param(
    [int]$SerialTcpPort = 4565,
    [string]$Image = "",
    [string]$ArtifactPath = "",
    [string]$ManifestPath = "",
    [string]$PersistDiskPath = "",
    [string]$ReportDir = "$PSScriptRoot\..\release\vm-reports",
    [int]$TimeoutSeconds = 90,
    [switch]$Network,
    [switch]$KeepImage,
    [int]$SerialWriteChunkSize = 256,
    [int]$SerialWriteDelayMilliseconds = 0,
    [ValidateSet("full", "quick", "recovery", "genesis-ui", "hello-rollback-dry-run", "module-audit-rollback", "provider-memory", "provider-memory-full", "candidate-delivery", "m6c-promotion", "m12-distribution-provenance", "m6d-rollback", "m8-lifeline", "persistence", "memory-durable", "structured-store", "project-workspace", "project-build", "project-app", "project-install", "secret-vault", "core-policy", "m11-wasm-import-grant", "m11-buffer-channel", "m11-6-certwindow", "m11-7-httphead", "m11-8-certspki", "m11-9-dnsparse", "m11-beyond-env-lifecycle", "m11-net-imports", "m11-crypto-imports", "m11-acquisition-service", "network-acquisition", "usb-hotplug")]
    [string]$Profile = "full"
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$RunScript = Join-Path $RepoRoot "scripts\run-stage0-qemu.ps1"
$PackageScript = Join-Path $RepoRoot "scripts\package-stage0.ps1"
$ValidateManifestScript = Join-Path $PSScriptRoot "validate-module-manifest.ps1"
$RunId = "shadow-{0:yyyyMMdd-HHmmss}-{1}" -f (Get-Date), $PID
$RunDir = Join-Path $env:TEMP "raios-$RunId"
$SerialLog = Join-Path $RunDir "serial.log"
$ReportPath = Join-Path $ReportDir "$RunId.json"
$ReportHashPath = "$ReportPath.sha256"
$QemuPid = $null
$TempImage = $false
$Result = "failed"
$Failures = New-Object System.Collections.Generic.List[object]
$Predicates = New-Object System.Collections.Generic.List[object]
$ExecutedCommands = New-Object System.Collections.Generic.List[object]
$StartedAt = [DateTime]::UtcNow
$script:SerialTcpClient = $null
$script:SerialTcpDrainStream = $null
$script:SerialTcpStream = $null
$QemuArgList = @()
$HardwareProfile = $null
$ResolvedImage = $null
$MonitorTcpPort = 0
$QmpTcpPort = 0
$ScratchImage = $null
$AuditRollbackTargetImage = $null
$PersistDiskImage = $null
$StructuredStoreDiskImage = $null
$StructuredStoreFixture = $null
$ResolvedArtifact = $null
$ResolvedManifest = $null
$ManifestValidation = $null
$script:SerialLogCachePath = $null
$script:SerialLogCacheLength = [int64]-1
$script:SerialLogCacheWriteTicks = [int64]-1
$script:SerialLogCacheContent = $null
$script:QemuProcess = $null
$script:QemuProcessBeforeTeardown = $null
$script:QemuProcessAfterTeardown = $null
$script:QemuTeardownAction = "not_started"
$script:SerialTransportFailure = $null
$script:VisualEvidence = New-Object System.Collections.Generic.List[object]
$script:ReportForbiddenDynamicValues = New-Object System.Collections.Generic.List[object]
$script:ReportSecurityTripwire = $null

. (Join-Path $PSScriptRoot "shadow-vm-smoke-support.ps1")

New-Item -ItemType Directory -Force -Path $RunDir | Out-Null

$ResolvedArtifact = Resolve-OptionalPath -Path $ArtifactPath
$ResolvedManifest = Resolve-OptionalPath -Path $ManifestPath

try {
    if ($Profile -in @("m11-net-imports", "network-acquisition") -and -not $Network) {
        throw "$Profile requires -Network (e1000 + DHCP + real TCP step)"
    }
    if ($ResolvedArtifact -and -not $ResolvedManifest) {
        throw "ArtifactPath requires ManifestPath; artifacts must not enter the evidence flow without a manifest"
    }
    if ($ResolvedManifest) {
        $validationParams = @{
            ManifestPath = $ResolvedManifest
        }
        if ($ResolvedArtifact) {
            $validationParams.ArtifactPath = $ResolvedArtifact
        }
        $validationJson = & $ValidateManifestScript @validationParams
        $ManifestValidation = ($validationJson -join [Environment]::NewLine) | ConvertFrom-Json
        if (-not $ManifestValidation.valid) {
            throw "Manifest validation failed"
        }
    }

    if ($Profile -in @("m6c-promotion", "network-acquisition")) {
        $env:CARGO_HOME = (Resolve-Path (Join-Path $RepoRoot ".cargo-home")).Path
        $env:CARGO_TARGET_DIR = Join-Path $RepoRoot "target"
        & cargo build --locked -p ota-tools --bin dev-promotion-signer
        if ($LASTEXITCODE -ne 0) {
            throw "dev-promotion-signer build failed with exit code $LASTEXITCODE"
        }
    }

    if ($Image) {
        if ($Profile -eq "network-acquisition") {
            throw "network-acquisition requires a per-run temporary pin-bearing image; do not pass -Image"
        }
        $ResolvedImage = (Resolve-Path -LiteralPath $Image).Path
    }
    else {
        $ResolvedImage = Join-Path $RunDir "raios-stage0-shadow.img"
        $TempImage = $true
        $packageParams = @{ Profile = "release"; Image = $ResolvedImage; UseTempEsp = $true }
        if ($Profile -eq "network-acquisition") {
            $fixtureWrapper = Join-Path $PSScriptRoot "net8-w7-tls-fixture.ps1"
            $fixtureReady = Join-Path $RunDir "w7-fixture-ready.json"
            $fixtureResult = Join-Path $RunDir "w7-fixture-result.json"
            $fixtureMode = Join-Path $RunDir "w7-fixture-mode.txt"
            $fixtureArtifact = Join-Path $RepoRoot "seed-kernel\artifacts\svc.demo.echo.wasm"
            $null = & $fixtureWrapper -Action Start -ArtifactPath $fixtureArtifact -ReadyPath $fixtureReady -ResultPath $fixtureResult -ModePath $fixtureMode
            $fixtureDeadline = [DateTime]::UtcNow.AddSeconds(60)
            while (-not (Test-Path -LiteralPath $fixtureReady -PathType Leaf) -and [DateTime]::UtcNow -lt $fixtureDeadline) {
                Start-Sleep -Milliseconds 50
            }
            if (-not (Test-Path -LiteralPath $fixtureReady -PathType Leaf)) {
                throw "NET-8 TLS fixture did not publish its atomic ready file"
            }
            $ready = Get-Content -LiteralPath $fixtureReady -Raw | ConvertFrom-Json
            if ($ready.schema -ne "raios.net8_w7_tls_fixture_ready.v0" -or
                $ready.sni -ne "w7.test.raios" -or [int]$ready.host_port -ne 8443 -or
                [int]$ready.artifact_length -ne 4205 -or
                $ready.artifact_sha256 -ne "f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2" -or
                $ready.path -ne "/raios/cas/sha256/f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2" -or
                $ready.spki_sha256 -notmatch '^[0-9a-f]{64}$') {
                throw "NET-8 TLS fixture ready file failed validation"
            }
            $pinEnvName = "RAIOS_NET8_W7_PIN_$PID"
            [Environment]::SetEnvironmentVariable($pinEnvName, [string]$ready.spki_sha256, "Process")
            $packageParams.EmbedNet8W7SpkiPinFromEnv = $true
            $packageParams.Net8W7SpkiPinEnvVar = $pinEnvName
        }
        & $PackageScript @packageParams
        if ($Profile -eq "network-acquisition") {
            [Environment]::SetEnvironmentVariable("RAIOS_NET8_W7_PIN_$PID", $null, "Process")
        }
        if ($LASTEXITCODE -ne 0) {
            throw "Image packaging failed with exit code $LASTEXITCODE"
        }
    }

    $ScratchImage = Join-Path $RunDir "raios-stage0-scratch.img"
    $scratchSector0 = New-Object byte[] 512
    $scratchMarker = [System.Text.Encoding]::ASCII.GetBytes("RAIOS_SCRATCH_V0")
    [Array]::Copy($scratchMarker, 0, $scratchSector0, 0, $scratchMarker.Length)
    $scratchStream = [System.IO.File]::Open($ScratchImage, [System.IO.FileMode]::Create, [System.IO.FileAccess]::ReadWrite, [System.IO.FileShare]::Read)
    try {
        $scratchStream.SetLength(1MB)
        $scratchStream.Write($scratchSector0, 0, $scratchSector0.Length)
    }
    finally {
        $scratchStream.Dispose()
    }

    $AuditRollbackTargetImage = Join-Path $RunDir "raios-stage0-audit-rollback-target.img"
    $auditRollbackTargetSector0 = New-Object byte[] 512
    $auditRollbackTargetMarker = [System.Text.Encoding]::ASCII.GetBytes("RAIOS_AUDITRB_V0")
    [Array]::Copy($auditRollbackTargetMarker, 0, $auditRollbackTargetSector0, 0, $auditRollbackTargetMarker.Length)
    $auditRollbackTargetStream = [System.IO.File]::Open($AuditRollbackTargetImage, [System.IO.FileMode]::Create, [System.IO.FileAccess]::ReadWrite, [System.IO.FileShare]::Read)
    try {
        $auditRollbackTargetStream.SetLength(1MB)
        $auditRollbackTargetStream.Write($auditRollbackTargetSector0, 0, $auditRollbackTargetSector0.Length)
    }
    finally {
        $auditRollbackTargetStream.Dispose()
    }

    if ($Profile -in @("structured-store", "project-workspace", "project-build", "project-app", "project-install", "secret-vault")) {
        if ($PersistDiskPath) {
            throw "$Profile profile creates its own exact valid-a SEED_DATA fixture"
        }
        $StructuredStoreDiskImage = Join-Path $RunDir "raios-structured-store-c1.img"
        $structuredStoreBuilder = Join-Path $RepoRoot "scripts\make-structured-store-image.py"
        $structuredStoreError = Join-Path $RunDir "structured-store-builder.err.txt"
        $fixtureJson = & python $structuredStoreBuilder create $StructuredStoreDiskImage --size-mib 16 --json 2> $structuredStoreError
        if ($LASTEXITCODE -ne 0) {
            $fixtureError = Get-Content -Raw -LiteralPath $structuredStoreError -ErrorAction SilentlyContinue
            throw "Structured-store fixture build failed: $fixtureError"
        }
        $StructuredStoreFixture = ($fixtureJson -join [Environment]::NewLine) | ConvertFrom-Json
        if (-not $StructuredStoreFixture.valid -or -not $StructuredStoreFixture.disposable_qemu_only -or
            $StructuredStoreFixture.store_state -ne "empty_unformatted") {
            throw "Structured-store fixture identity or empty-state validation failed"
        }
        $StructuredStoreDiskImage = (Resolve-Path -LiteralPath $StructuredStoreDiskImage).Path

        # BOOTCTL remains on its own SEED_DATA disk. The C1 Vault store keeps
        # its distinct controller-port/GPT identity and is never a fallback.
        $structuredStorePersist = Assert-PersistDiskPathSafe -Path (Join-Path $RunDir "raios-persist-structured-store.img")
        $structuredStorePersistError = Join-Path $RunDir "structured-store-persist-builder.err.txt"
        $null = & python (Join-Path $RepoRoot "scripts\make-gpt-persist-image.py") --self-check --seed-bootctl valid-a $structuredStorePersist 2> $structuredStorePersistError
        if ($LASTEXITCODE -ne 0) {
            throw "structured-store BOOTCTL fixture build failed with exit code $LASTEXITCODE"
        }
        $PersistDiskImage = (Resolve-Path -LiteralPath $structuredStorePersist).Path
    }
    elseif ($Profile -eq "network-acquisition") {
        if ($PersistDiskPath) {
            throw "network-acquisition creates its own exact valid-a disposable persist disk"
        }
        $PersistDiskImage = Resolve-PersistDiskImage -PersistDiskPath "" -RunDir $RunDir -SeedBootControl "valid-a"
    }
    elseif ($Profile -eq "persistence" -or $Profile -eq "memory-durable" -or $PersistDiskPath) {
        $PersistDiskImage = Resolve-PersistDiskImage -PersistDiskPath $PersistDiskPath -RunDir $RunDir
    }
    elseif ($Profile -eq "m8-lifeline" -or $Profile -eq "core-policy") {
        # Both profiles require a real Normal A/1 BOOTCTL decision. M8 also uses
        # the empty RECLOG for its bounded durable recovery-action append.
        $persistFixtureName = if ($Profile -eq "core-policy") { "raios-persist-core-policy.img" } else { "raios-persist-m8-lifeline.img" }
        $m8PersistDisk = Assert-PersistDiskPathSafe -Path (Join-Path $RunDir $persistFixtureName)
        $m8PersistError = Join-Path $RunDir "$Profile-persist-builder.err.txt"
        $null = & python (Join-Path $RepoRoot "scripts\make-gpt-persist-image.py") --self-check --seed-bootctl valid-a $m8PersistDisk 2> $m8PersistError
        if ($LASTEXITCODE -ne 0) {
            throw "$Profile persist disk build failed with exit code $LASTEXITCODE"
        }
        $PersistDiskImage = (Resolve-Path -LiteralPath $m8PersistDisk).Path
    }

    $Nic = if ($Network) { "e1000" } else { "none" }
    $HardwareProfile = New-HardwareProfile -Nic $Nic -ScratchDrive $true -AuditRollbackTargetDrive $true -PersistDrive ($null -ne $PersistDiskImage)
    $QemuArgList = @(
        "-StopExisting",
        "-Image", $ResolvedImage,
        "-ScratchImage", $ScratchImage,
        "-AuditRollbackTargetImage", $AuditRollbackTargetImage,
        "-SerialMode", "tcp",
        "-SerialTcpPort", "$SerialTcpPort",
        "-SerialLog", $SerialLog,
        "-Headless",
        "-UsbXhciInput",
        "-Cpu", "max",
        "-Nic", $Nic
    )
    $runParams = @{
        StopExisting = $true
        Image = $ResolvedImage
        ScratchImage = $ScratchImage
        AuditRollbackTargetImage = $AuditRollbackTargetImage
        SerialMode = "tcp"
        SerialTcpPort = $SerialTcpPort
        SerialLog = $SerialLog
        Headless = $true
        UsbXhciInput = $true
        Cpu = "max"
        Nic = $Nic
    }
    if ($Profile -eq "network-acquisition") {
        # Bridge guest 10.0.2.100:8443 to the host loopback TLS fixture. The guest's
        # W7 source policy targets 10.0.2.100 (not the slirp gateway 10.0.2.2, which
        # QEMU refuses to guestfwd). The fixture is already listening on 127.0.0.1:8443.
        $w7Fwd = "tcp:10.0.2.100:8443-tcp:127.0.0.1:8443"
        $QemuArgList += @("-GuestFwd", $w7Fwd)
        $runParams.GuestFwd = $w7Fwd
    }
    if ($PersistDiskImage) {
        $QemuArgList += @(
            "-PersistDiskPath", $PersistDiskImage
        )
        $runParams.PersistDiskPath = $PersistDiskImage
    }
    if ($StructuredStoreDiskImage) {
        $QemuArgList += @(
            "-StructuredStoreDiskPath", $StructuredStoreDiskImage
        )
        $runParams.StructuredStoreDiskPath = $StructuredStoreDiskImage
    }
    if ($Profile -in @("usb-hotplug", "genesis-ui", "project-app", "project-install", "secret-vault", "m11-beyond-env-lifecycle", "m11-net-imports", "network-acquisition")) {
        $MonitorTcpPort = $SerialTcpPort + 1000
        $QemuArgList += @(
            "-MonitorTcpPort", "$MonitorTcpPort"
        )
        $runParams.MonitorTcpPort = $MonitorTcpPort
    }
    if ($Profile -in @("genesis-ui", "project-app", "project-install", "m6c-promotion", "network-acquisition")) {
        $QmpTcpPort = $SerialTcpPort + 2000
        $QemuArgList += @(
            "-QmpTcpPort", "$QmpTcpPort"
        )
        $runParams.QmpTcpPort = $QmpTcpPort
    }

    $runOutput = & $RunScript @runParams
    foreach ($line in $runOutput) {
        if ($line -match '^qemu pid:\s*(\d+)') {
            $QemuPid = [int]$Matches[1]
        }
    }
    if (-not $QemuPid) {
        throw "Could not parse QEMU pid from runner output"
    }
    try {
        $script:QemuProcess = Get-Process -Id $QemuPid -ErrorAction Stop
    }
    catch {
        $script:QemuProcess = $null
    }

    Assert-LogContains -Name "boot:serial_console_ready" -Needle "SERIAL CONSOLE READY" -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains -Name "boot:framebuffer_ready" -Needle "status FRAMEBUFFER: READY" -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains -Name "boot:usb_xhci_ready" -Needle "status USB-XHCI: READY" -TimeoutSeconds $TimeoutSeconds
    if ($Network) {
        Assert-LogContains -Name "network:e1000_initialised" -Needle "e1000 network initialised; DHCP polling enabled" -TimeoutSeconds $TimeoutSeconds
        Assert-LogContains -Name "network:dhcp_lease_acquired" -Needle "DHCP lease acquired:" -TimeoutSeconds $TimeoutSeconds
    }

    :SmokeProfileValidation while ($true) {
        if ($Profile -eq "persistence") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-persistence.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "memory-durable") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-memory-durable.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "structured-store") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-structured-store.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "project-workspace") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-project-workspace.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "project-build") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-project-build.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "project-app") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-project-build.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "project-install") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-project-install.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "secret-vault") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-secret-vault.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "core-policy") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-core-policy.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "usb-hotplug") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-usb-hotplug.ps1")
            break SmokeProfileValidation
        }

        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-common.ps1")

        if ($Profile -eq "genesis-ui") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-genesis-ui.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "provider-memory") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-provider-memory.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "provider-memory-full") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-full-provider-memory.ps1")
            Invoke-ProviderContextGateSelftestProfile
            break SmokeProfileValidation
        }

        if ($Profile -eq "quick") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-quick.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "candidate-delivery") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-candidate-delivery.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "m6c-promotion") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-m6c-promotion.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "m12-distribution-provenance") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-m12-distribution-provenance.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "m11-wasm-import-grant") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-m11-wasm-import-grant.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "m11-buffer-channel") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-m11-4-buffer-channel.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "m11-6-certwindow") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-m11-6-certwindow.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "m11-7-httphead") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-m11-7-httphead.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "m11-8-certspki") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-m11-8-certspki.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "m11-9-dnsparse") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-m11-9-dnsparse.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "m11-beyond-env-lifecycle") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-m11-beyond-env-lifecycle.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "m11-net-imports") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-m11-net-imports.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "m11-crypto-imports") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-m11-crypto-imports.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "m11-acquisition-service") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-m11-acquisition-service.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "network-acquisition") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-network-acquisition.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "m6d-rollback") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-m6d-rollback.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "m8-lifeline") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-m8-lifeline.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "hello-rollback-dry-run") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-hello-rollback-dry-run.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "module-audit-rollback") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-full-module-evidence.ps1")
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-full-module-audit-rollback.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "full") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-full-provider-memory.ps1")
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-full-module-evidence.ps1")
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-full-module-audit-rollback.ps1")
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-full-module-selftests.ps1")
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-full-module-load-gate.ps1")
        }

        if ($Profile -eq "recovery") {
            break SmokeProfileValidation
        }

        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-full-audit.ps1")
        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-hello-rollback-dry-run.ps1")
        if ($Profile -eq "full") {
            Invoke-ProviderContextGateSelftestProfile
        }
        break SmokeProfileValidation
    }

    $Result = "passed"
}
catch {
    $Failures.Add($_.Exception.Message) | Out-Null
    throw
}
finally {
    Close-SerialTcpConnection

    $fixtureReadyForCleanup = Join-Path $RunDir "w7-fixture-ready.json"
    if (Test-Path -LiteralPath $fixtureReadyForCleanup -PathType Leaf) {
        try {
            $fixturePid = [int](Get-Content -LiteralPath $fixtureReadyForCleanup -Raw | ConvertFrom-Json).process_id
            & (Join-Path $PSScriptRoot "net8-w7-tls-fixture.ps1") -Action Stop -ProcessId $fixturePid
        }
        catch {
        }
    }
    [Environment]::SetEnvironmentVariable("RAIOS_NET8_W7_PIN_$PID", $null, "Process")

    $script:QemuProcessBeforeTeardown = Get-QemuProcessSnapshot -Observation "before_teardown"
    if ($QemuPid) {
        if ($script:QemuProcessBeforeTeardown.state -eq "running") {
            $script:QemuTeardownAction = "stop_process"
            Stop-Process -Id $QemuPid -Force -ErrorAction SilentlyContinue
            if ($null -ne $script:QemuProcess) {
                try {
                    $script:QemuProcess.WaitForExit(5000) | Out-Null
                }
                catch {
                }
            }
        }
        else {
            $script:QemuTeardownAction = "already_exited"
        }
    }
    $script:QemuProcessAfterTeardown = Get-QemuProcessSnapshot -Observation "after_teardown"

    $reportWriteException = $null
    try {
        Write-Report `
            -FinalResult $Result `
            -ResolvedImage $ResolvedImage `
            -ResolvedArtifact $ResolvedArtifact `
            -ResolvedManifest $ResolvedManifest `
            -QemuArgList $QemuArgList `
            -HardwareProfile $HardwareProfile `
            -StartedAt $StartedAt
    }
    catch {
        $reportWriteException = $_.Exception
        $Result = "failed"
    }
    finally {
        Clear-ReportForbiddenDynamicValues
    }

    if ($TempImage -and -not $KeepImage) {
        Remove-Item -LiteralPath $ResolvedImage -Force -ErrorAction SilentlyContinue
    }

    # Passed runs delete their temp dir (356 leftovers once filled the disk);
    # failed runs keep it so the serial log stays available for forensics.
    if ($Result -eq "passed" -and -not $KeepImage) {
        Remove-Item -LiteralPath $RunDir -Recurse -Force -ErrorAction SilentlyContinue
    }

    Write-Host "shadow vm result: $Result"
    Write-Host "report: $ReportPath"
    Write-Host "report sha256: $ReportHashPath"
    Write-Host "serial log: $SerialLog"

    if ($script:ReportSecurityTripwire) {
        throw "SECURITY_TRIPWIRE_REPORT_FORBIDDEN_DYNAMIC_VALUE labels=$($script:ReportSecurityTripwire)"
    }
    if ($reportWriteException) {
        throw $reportWriteException
    }
}
