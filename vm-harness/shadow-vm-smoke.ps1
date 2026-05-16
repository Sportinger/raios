param(
    [int]$SerialTcpPort = 4565,
    [string]$Image = "",
    [string]$ArtifactPath = "",
    [string]$ManifestPath = "",
    [string]$ReportDir = "$PSScriptRoot\..\release\vm-reports",
    [int]$TimeoutSeconds = 45,
    [switch]$Network,
    [switch]$KeepImage
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$RunScript = Join-Path $RepoRoot "scripts\run-stage0-qemu.ps1"
$PackageScript = Join-Path $RepoRoot "scripts\package-stage0.ps1"
$ValidateManifestScript = Join-Path $PSScriptRoot "validate-module-manifest.ps1"
$RunId = "shadow-{0:yyyyMMdd-HHmmss}-{1}" -f (Get-Date), $PID
$RunDir = Join-Path $env:TEMP "seedos-$RunId"
$SerialLog = Join-Path $RunDir "serial.log"
$ReportPath = Join-Path $ReportDir "$RunId.json"
$ReportHashPath = "$ReportPath.sha256"
$QemuPid = $null
$TempImage = $false
$Result = "failed"
$Failures = New-Object System.Collections.Generic.List[string]
$Predicates = New-Object System.Collections.Generic.List[object]
$StartedAt = [DateTime]::UtcNow
$QemuArgList = @()
$HardwareProfile = $null
$ResolvedImage = $null
$ResolvedArtifact = $null
$ResolvedManifest = $null
$ManifestValidation = $null

function Resolve-OptionalPath {
    param([string]$Path)
    if (-not $Path) {
        return $null
    }
    if (-not (Test-Path -LiteralPath $Path)) {
        throw "Path does not exist: $Path"
    }
    return (Resolve-Path -LiteralPath $Path).Path
}

function Get-FileSha256OrNull {
    param([string]$Path)
    if (-not $Path) {
        return $null
    }
    if (-not (Test-Path -LiteralPath $Path)) {
        return $null
    }
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Get-TextSha256 {
    param([string]$Text)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($Text)
        $hash = $sha.ComputeHash($bytes)
        return ([BitConverter]::ToString($hash) -replace "-", "").ToLowerInvariant()
    }
    finally {
        $sha.Dispose()
    }
}

function ConvertTo-ReportJson {
    param([object]$Value)
    return ($Value | ConvertTo-Json -Depth 20 -Compress)
}

function Get-NullablePath {
    param([string]$Path)
    if ([string]::IsNullOrWhiteSpace($Path)) {
        return $null
    }
    return $Path
}

function Get-SerialLogTail {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) {
        return "serial log not created"
    }

    $content = Get-Content -Raw -LiteralPath $Path -ErrorAction SilentlyContinue
    if ($null -eq $content) {
        return "serial log unreadable"
    }

    $limit = 1600
    if ($content.Length -le $limit) {
        return $content
    }
    return $content.Substring($content.Length - $limit)
}

function New-HardwareProfile {
    param([string]$Nic)

    $networkDevice = if ($Nic -eq "e1000") {
        "e1000_user"
    }
    else {
        "none"
    }

    return [ordered]@{
        profile = "seedos.shadow_vm.q35_xhci.v0"
        machine = "q35"
        memory = "512M"
        cpu = "max"
        firmware = "edk2-x86_64"
        boot_drive = "ide_raw_image"
        display = "none"
        serial = "tcp_chardev_with_log"
        input = @(
            "qemu-xhci",
            "usb-kbd",
            "usb-tablet"
        )
        network = $networkDevice
    }
}

function Wait-ForLogText {
    param(
        [string]$Path,
        [string]$Needle,
        [int]$TimeoutSeconds
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        if (Test-Path -LiteralPath $Path) {
            $content = Get-Content -Raw -LiteralPath $Path -ErrorAction SilentlyContinue
            if ($content -clike "*$Needle*") {
                return $true
            }
        }
        Start-Sleep -Milliseconds 200
    } while ([DateTime]::UtcNow -lt $deadline)

    return $false
}

function Add-Predicate {
    param(
        [string]$Name,
        [string]$Expected,
        [bool]$Passed,
        [string]$Actual = ""
    )

    $Predicates.Add([ordered]@{
        name = $Name
        expected = $Expected
        passed = $Passed
        actual = $Actual
    }) | Out-Null

    if (-not $Passed) {
        $Failures.Add($Name) | Out-Null
    }
}

function Assert-LogContains {
    param(
        [string]$Name,
        [string]$Needle,
        [int]$TimeoutSeconds
    )

    $passed = Wait-ForLogText -Path $SerialLog -Needle $Needle -TimeoutSeconds $TimeoutSeconds
    $actual = if ($passed) { "found" } else { Get-SerialLogTail -Path $SerialLog }
    Add-Predicate -Name $Name -Expected "serial_contains:$Needle" -Passed $passed -Actual $actual
    if (-not $passed) {
        throw "Timed out waiting for '$Needle' in $SerialLog"
    }
}

function Send-SerialText {
    param(
        [int]$Port,
        [string]$Text,
        [int]$TimeoutSeconds
    )

    $client = [System.Net.Sockets.TcpClient]::new()
    $client.NoDelay = $true
    $connect = $client.BeginConnect("127.0.0.1", $Port, $null, $null)
    if (-not $connect.AsyncWaitHandle.WaitOne([TimeSpan]::FromSeconds($TimeoutSeconds))) {
        $client.Close()
        throw "Timed out connecting to QEMU serial TCP port $Port"
    }
    $client.EndConnect($connect)

    try {
        $stream = $client.GetStream()
        $bytes = [System.Text.Encoding]::ASCII.GetBytes($Text)
        foreach ($byte in $bytes) {
            $stream.WriteByte($byte)
            Start-Sleep -Milliseconds 8
        }
        $stream.Flush()
        Start-Sleep -Milliseconds 250
    }
    finally {
        $client.Close()
    }
}

function Send-AgentCommand {
    param(
        [string]$Command,
        [string]$ExpectedMarker
    )

    Send-SerialText -Port $SerialTcpPort -Text "$Command`r" -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains -Name "command:$Command" -Needle $ExpectedMarker -TimeoutSeconds $TimeoutSeconds
}

function Write-Report {
    param(
        [string]$FinalResult,
        [string]$ResolvedImage,
        [string]$ResolvedArtifact,
        [string]$ResolvedManifest,
        [string[]]$QemuArgList,
        [object]$HardwareProfile,
        [DateTime]$StartedAt
    )

    New-Item -ItemType Directory -Force -Path $ReportDir | Out-Null

    $serialHash = Get-FileSha256OrNull -Path $SerialLog
    $errLog = [System.IO.Path]::ChangeExtension($SerialLog, ".err.txt")
    $endedAt = [DateTime]::UtcNow
    $networkMode = if ($Network) { "e1000_user_enabled" } else { "disabled" }
    $baseImageSha256 = Get-FileSha256OrNull -Path $ResolvedImage
    $artifactSha256 = Get-FileSha256OrNull -Path $ResolvedArtifact
    $manifestSha256 = Get-FileSha256OrNull -Path $ResolvedManifest
    $qemuArgsCanonical = ConvertTo-ReportJson -Value @($QemuArgList)
    $qemuArgsSha256 = Get-TextSha256 -Text $qemuArgsCanonical
    $hardwareProfileSha256 = Get-TextSha256 -Text (ConvertTo-ReportJson -Value $HardwareProfile)

    $report = [ordered]@{
        schema = "seedos.vm_test_report.v0"
        result = $FinalResult
        generated_at_utc = ($endedAt.ToString("o"))
        started_at_utc = ($StartedAt.ToString("o"))
        duration_ms = ([int][Math]::Round(($endedAt - $StartedAt).TotalMilliseconds))
        run_id = $RunId
        sandbox_policy = [ordered]@{
            hypervisor = "qemu-system-x86_64"
            headless = $true
            shared_folders = "none"
            host_filesystem_mounts = "none"
            network = $networkMode
            boot_media = "temporary_or_explicit_image"
            qemu_killed_after_run = $true
        }
        base_image = [ordered]@{
            path = (Get-NullablePath -Path $ResolvedImage)
            sha256 = $baseImageSha256
            temporary = $TempImage
        }
        candidate_artifact = [ordered]@{
            path = (Get-NullablePath -Path $ResolvedArtifact)
            sha256 = $artifactSha256
        }
        candidate_manifest = [ordered]@{
            path = (Get-NullablePath -Path $ResolvedManifest)
            sha256 = $manifestSha256
            validation = $ManifestValidation
        }
        hardware_profile = $HardwareProfile
        qemu = [ordered]@{
            script = $RunScript
            args = @($QemuArgList)
            args_canonical_json = $qemuArgsCanonical
            args_sha256 = $qemuArgsSha256
            serial_tcp_port = $SerialTcpPort
            pid = $QemuPid
        }
        evidence_binding = [ordered]@{
            base_image_sha256 = $baseImageSha256
            candidate_artifact_sha256 = $artifactSha256
            candidate_manifest_sha256 = $manifestSha256
            hardware_profile_sha256 = $hardwareProfileSha256
            qemu_args_sha256 = $qemuArgsSha256
            serial_log_sha256 = $serialHash
            predicate_count = $Predicates.Count
            predicate_passed_count = @($Predicates.ToArray() | Where-Object { $_.passed }).Count
            predicate_failed_count = @($Predicates.ToArray() | Where-Object { -not $_.passed }).Count
            result = $FinalResult
        }
        commands = @(
            "describe",
            "snapshot",
            "services",
            "problems",
            "module.load_ephemeral"
        )
        predicates = @($Predicates.ToArray())
        serial_log = [ordered]@{
            path = $SerialLog
            sha256 = $serialHash
        }
        stderr_log = [ordered]@{
            path = $errLog
            sha256 = (Get-FileSha256OrNull -Path $errLog)
        }
        failures = @($Failures.ToArray())
    }

    $json = $report | ConvertTo-Json -Depth 20
    Set-Content -LiteralPath $ReportPath -Value $json -Encoding UTF8
    $reportHash = Get-FileSha256OrNull -Path $ReportPath
    Set-Content -LiteralPath $ReportHashPath -Value "$reportHash  $ReportPath" -Encoding ASCII
}

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
        $ResolvedImage = Join-Path $RunDir "seedos-stage0-shadow.img"
        $TempImage = $true
        & $PackageScript -Profile release -Image $ResolvedImage -UseTempEsp
        if ($LASTEXITCODE -ne 0) {
            throw "Image packaging failed with exit code $LASTEXITCODE"
        }
    }

    $Nic = if ($Network) { "e1000" } else { "none" }
    $HardwareProfile = New-HardwareProfile -Nic $Nic
    $QemuArgList = @(
        "-StopExisting",
        "-Image", $ResolvedImage,
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
        SerialMode = "tcp"
        SerialTcpPort = $SerialTcpPort
        SerialLog = $SerialLog
        Headless = $true
        UsbXhciInput = $true
        Cpu = "max"
        Nic = $Nic
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

    Assert-LogContains -Name "boot:serial_console_ready" -Needle "SERIAL CONSOLE READY" -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains -Name "boot:framebuffer_ready" -Needle "status FRAMEBUFFER: READY" -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains -Name "boot:usb_xhci_ready" -Needle "status USB-XHCI: READY" -TimeoutSeconds $TimeoutSeconds

    Send-AgentCommand -Command "describe" -ExpectedMarker "SEEDOS_AGENT_END system.describe"
    Assert-LogContains -Name "protocol:describe_schema" -Needle '"schema": "system.describe.v0"' -TimeoutSeconds 1

    Send-AgentCommand -Command "snapshot" -ExpectedMarker "SEEDOS_AGENT_END system.snapshot"
    Assert-LogContains -Name "protocol:snapshot_schema" -Needle '"schema": "system.snapshot.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_trust_problem" -Needle "provider.tls_pin_config_missing" -TimeoutSeconds 1

    Send-AgentCommand -Command "services" -ExpectedMarker "SEEDOS_AGENT_END service.inventory"
    Assert-LogContains -Name "protocol:service_inventory_schema" -Needle '"schema": "service.inventory.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:openai_service_listed" -Needle "svc.provider.openai_direct" -TimeoutSeconds 1

    Send-AgentCommand -Command "problems" -ExpectedMarker "SEEDOS_AGENT_END problem.list"
    Assert-LogContains -Name "protocol:problem_list_schema" -Needle '"schema": "problem.list.v0"' -TimeoutSeconds 1

    Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "SEEDOS_AGENT_END module.load_ephemeral"
    Assert-LogContains -Name "policy:mutating_load_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:vm_report_required" -Needle "seedos.vm_test_report.v0" -TimeoutSeconds 1

    $Result = "passed"
}
catch {
    $Failures.Add($_.Exception.Message) | Out-Null
    throw
}
finally {
    if ($QemuPid) {
        Stop-Process -Id $QemuPid -Force -ErrorAction SilentlyContinue
    }

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

    Write-Host "shadow vm result: $Result"
    Write-Host "report: $ReportPath"
    Write-Host "report sha256: $ReportHashPath"
    Write-Host "serial log: $SerialLog"
}
