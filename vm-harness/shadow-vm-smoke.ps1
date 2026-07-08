param(
    [int]$SerialTcpPort = 4565,
    [string]$Image = "",
    [string]$ArtifactPath = "",
    [string]$ManifestPath = "",
    [string]$PersistDiskPath = "",
    [string]$ReportDir = "$PSScriptRoot\..\release\vm-reports",
    [int]$TimeoutSeconds = 45,
    [switch]$Network,
    [switch]$KeepImage,
    [int]$SerialWriteChunkSize = 256,
    [int]$SerialWriteDelayMilliseconds = 0,
    [ValidateSet("full", "quick", "recovery", "hello-rollback-dry-run", "module-audit-rollback", "provider-memory", "provider-memory-full", "candidate-delivery", "m6c-promotion", "m6d-rollback", "m8-lifeline", "persistence", "memory-durable", "m11-wasm-import-grant")]
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
$ScratchImage = $null
$AuditRollbackTargetImage = $null
$PersistDiskImage = $null
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

. (Join-Path $PSScriptRoot "shadow-vm-smoke-support.ps1")

New-Item -ItemType Directory -Force -Path $RunDir | Out-Null

$ResolvedArtifact = Resolve-OptionalPath -Path $ArtifactPath
$ResolvedManifest = Resolve-OptionalPath -Path $ManifestPath

try {
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

    if ($Image) {
        $ResolvedImage = (Resolve-Path -LiteralPath $Image).Path
    }
    else {
        $ResolvedImage = Join-Path $RunDir "raios-stage0-shadow.img"
        $TempImage = $true
        & $PackageScript -Profile release -Image $ResolvedImage -UseTempEsp
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

    if ($Profile -eq "persistence" -or $Profile -eq "memory-durable" -or $PersistDiskPath) {
        $PersistDiskImage = Resolve-PersistDiskImage -PersistDiskPath $PersistDiskPath -RunDir $RunDir
    }
    elseif ($Profile -eq "m8-lifeline") {
        # M8B-1: recovery.disable_module writes a durable recovery-action record, which
        # (per the M7C-2a discipline) requires Normal boot posture. Boot m8-lifeline with
        # a valid-a BOOTCTL persist disk (Normal posture) + an empty reclog so the durable
        # append lands at seq 1 and live target-classification denials are reached.
        $m8PersistDisk = Assert-PersistDiskPathSafe -Path (Join-Path $RunDir "raios-persist-m8-lifeline.img")
        $null = & python (Join-Path $RepoRoot "scripts\make-gpt-persist-image.py") --self-check --seed-bootctl valid-a $m8PersistDisk 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "m8-lifeline persist disk build failed with exit code $LASTEXITCODE"
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
    if ($PersistDiskImage) {
        $QemuArgList += @(
            "-PersistDiskPath", $PersistDiskImage
        )
        $runParams.PersistDiskPath = $PersistDiskImage
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

    :SmokeProfileValidation while ($true) {
        if ($Profile -eq "persistence") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-persistence.ps1")
            break SmokeProfileValidation
        }

        if ($Profile -eq "memory-durable") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-memory-durable.ps1")
            break SmokeProfileValidation
        }

        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-common.ps1")

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

        if ($Profile -eq "m11-wasm-import-grant") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-m11-wasm-import-grant.ps1")
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

        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-recovery-artifact-evidence.ps1")
        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-recovery-lifeline-foundation.ps1")
        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-recovery-command-frontdoor.ps1")
        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-recovery-command-authority.ps1")
        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-recovery-command-effects.ps1")
        . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-recovery-execution-binding.ps1")

        if ($Profile -eq "recovery") {
            . (Join-Path $PSScriptRoot "shadow-vm-smoke-profile-recovery-audit.ps1")
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

    Write-Report `
        -FinalResult $Result `
        -ResolvedImage $ResolvedImage `
        -ResolvedArtifact $ResolvedArtifact `
        -ResolvedManifest $ResolvedManifest `
        -QemuArgList $QemuArgList `
        -HardwareProfile $HardwareProfile `
        -StartedAt $StartedAt

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
}
