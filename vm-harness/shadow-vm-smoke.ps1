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
$RunDir = Join-Path $env:TEMP "raios-$RunId"
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
        profile = "raios.shadow_vm.q35_xhci.v0"
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

function Assert-LogDoesNotContain {
    param(
        [string]$Name,
        [string]$Needle
    )

    $content = if (Test-Path -LiteralPath $SerialLog) {
        Get-Content -Raw -LiteralPath $SerialLog -ErrorAction SilentlyContinue
    }
    else {
        ""
    }
    $passed = -not ($content.Contains($Needle))
    $actual = if ($passed) { "absent" } else { "found" }
    Add-Predicate -Name $Name -Expected "serial_not_contains:$Needle" -Passed $passed -Actual $actual
    if (-not $passed) {
        throw "Unexpected '$Needle' in $SerialLog"
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
        schema = "raios.vm_test_report.v0"
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
            "caps",
            "services",
            "problems",
            "agent memory.profile",
            "agent memory.context diagnostic",
            "agent memory.context provider_minimal",
            "agent provider.context_export provider_minimal",
            "agent provider.context_gate provider_minimal",
            "agent provider.context_gate_selftest provider_minimal",
            "agent provider.context_injection_gate provider_minimal",
            "agent provider.context_injection_gate_selftest provider_minimal",
            "agent memory.query",
            "agent memory.trace snapshot.current",
            "agent memory.recent_events",
            "agent audit.events 8",
            "agent memory.record_observation",
            "agent memory.propose_policy",
            "agent memory.supersede_fact",
            "agent memory.redact",
            "agent memory.compact",
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
        $ResolvedImage = Join-Path $RunDir "raios-stage0-shadow.img"
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

    Send-AgentCommand -Command "describe" -ExpectedMarker "RAIOS_AGENT_END system.describe"
    Assert-LogContains -Name "protocol:describe_schema" -Needle '"schema": "system.describe.v0"' -TimeoutSeconds 1

    Send-AgentCommand -Command "snapshot" -ExpectedMarker "RAIOS_AGENT_END system.snapshot"
    Assert-LogContains -Name "protocol:snapshot_schema" -Needle '"schema": "system.snapshot.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_trust_problem" -Needle "provider.tls_pin_config_missing" -TimeoutSeconds 1

    Send-AgentCommand -Command "caps" -ExpectedMarker "RAIOS_AGENT_END system.capabilities"
    Assert-LogContains -Name "protocol:capabilities_schema" -Needle '"schema": "system.capabilities.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_capability" -Needle '"id": "cap.memory.recent_events.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:audit_events_capability" -Needle '"id": "cap.audit.events.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_read_capability" -Needle '"id": "cap.provider.context_export.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_read_capability" -Needle '"id": "cap.provider.context_injection.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_capability_listed" -Needle '"id": "cap.provider.context_export"' -TimeoutSeconds 1

    Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory"
    Assert-LogContains -Name "protocol:service_inventory_schema" -Needle '"schema": "service.inventory.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:openai_service_listed" -Needle "svc.provider.openai_direct" -TimeoutSeconds 1

    Send-AgentCommand -Command "problems" -ExpectedMarker "RAIOS_AGENT_END problem.list"
    Assert-LogContains -Name "protocol:problem_list_schema" -Needle '"schema": "problem.list.v0"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.profile" -ExpectedMarker "RAIOS_AGENT_END memory.profile"
    Assert-LogContains -Name "protocol:memory_profile_schema" -Needle '"schema": "memory.profile.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_profile_scope" -Needle '"scope": "current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_profile_provider_minimal" -Needle '"provider_minimal"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_profile_provider_local_projection" -Needle '"local_projection": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_profile_diagnostic" -Needle '"diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_profile_planning" -Needle '"planning"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.context diagnostic" -ExpectedMarker "RAIOS_AGENT_END memory.context"
    Assert-LogContains -Name "protocol:memory_context_schema" -Needle '"schema": "raios.agent_context.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_profile" -Needle '"profile": "diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_scope" -Needle '"scope": "current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_event_id" -Needle '"context_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_audit_event_id" -Needle '"audit_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_snapshot_source" -Needle "system.snapshot.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_service_source" -Needle "service.inventory.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_problem_source" -Needle "problem.list.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_trust_problem" -Needle "provider.tls_pin_config_missing" -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.context provider_minimal" -ExpectedMarker "RAIOS_AGENT_END memory.context"
    Assert-LogContains -Name "protocol:memory_context_provider_profile" -Needle '"profile": "provider_minimal"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_export_disabled" -Needle '"provider_export": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_projection_schema" -Needle '"schema": "raios.provider_context_projection.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_projection_mode" -Needle '"mode": "local_read_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_projection_present" -Needle '"redaction_projection": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_classification_default" -Needle '"classification_default": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_projection_event" -Needle '"local_projection_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_can_export" -Needle '"can_export": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_trust_gate" -Needle '"reason": "provider_trust_not_positive"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_audit_gate" -Needle '"reason": "provider_context_export_audit_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_packet_evidence" -Needle '"packet_evidence":' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_packet_canonicalization" -Needle '"canonicalization": "raios.provider_minimal.packet.canonical.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_packet_hash" -Needle '"projected_packet_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_exported_fields_hash" -Needle '"exported_field_list_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_omitted_fields_hash" -Needle '"omitted_field_list_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_included_status" -Needle '"field": "current.status.*"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_included_key_state" -Needle '"field": "current.provider.api_key_state"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_omits_raw_snapshot" -Needle '"field": "system.snapshot.raw"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_omits_secret_prompt" -Needle '"field": "provider.direct_last_prompt"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_packet_purpose" -Needle '"purpose": "current_boot_provider_context"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_context_provider_snapshot_projection_record" -Needle "snapshot.current.provider_minimal" -TimeoutSeconds 1

    Send-AgentCommand -Command "agent provider.context_export provider_minimal" -ExpectedMarker "RAIOS_AGENT_END provider.context_export"
    Assert-LogContains -Name "protocol:provider_context_export_schema" -Needle '"schema": "raios.provider_context_export.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_profile" -Needle '"profile": "provider_minimal"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_capability" -Needle '"requested_capability": "cap.provider.context_export"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_no_write" -Needle '"provider_write": "not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_can_export_false" -Needle '"can_export": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_trust_block" -Needle '"reason": "provider_trust_not_positive"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_packet_binding_present" -Needle '"packet_evidence_binding": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_exported_fields_binding_present" -Needle '"exported_field_list_binding": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_omitted_fields_binding_present" -Needle '"omitted_field_list_binding": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_packet_canonicalization" -Needle '"packet_canonicalization": "raios.provider_minimal.packet.canonical.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_packet_hash" -Needle '"projected_packet_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_exported_fields_hash" -Needle '"exported_field_list_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_omitted_fields_hash" -Needle '"omitted_field_list_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_missing" -Needle '"provider_request_binding": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_denial_state" -Needle '"provider_request_binding_denial": "present_denied_not_bound"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_audit_binding_missing" -Needle '"provider_export_audit_binding": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_state" -Needle '"provider_export_denial_audit": "present_denied_no_provider_write"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_denial_schema" -Needle '"schema": "raios.provider_request_binding_denial.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_denial_id" -Needle '"id": "provider_request_binding_denial.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_attempt_id" -Needle '"attempted_request_id": "provider_request_attempt.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_denial_status" -Needle '"status": "denied_not_bound"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_denial_not_gate" -Needle '"satisfies_export_gate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_schema" -Needle '"schema": "raios.provider_context_export_denial_audit.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_id" -Needle '"id": "provider_context_export_denial_audit.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_status" -Needle '"status": "denied_no_provider_write"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_not_gate" -Needle '"satisfies_export_gate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_write_path_disabled" -Needle '"reason": "automatic_context_injection_disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_required" -Needle '"reason": "provider_request_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_audit_binding_required" -Needle '"reason": "provider_context_export_audit_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_denial_evidence" -Needle '"provider_request_binding_denial_id": "provider_request_binding_denial.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_binding_denial_event_evidence" -Needle '"provider_request_binding_denial_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_request_attempt_evidence" -Needle '"provider_request_attempt_id": "provider_request_attempt.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_audit_binding_status_evidence" -Needle '"export_audit_binding_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_evidence" -Needle '"export_denial_audit_id": "provider_context_export_denial_audit.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_event_evidence" -Needle '"export_denial_audit_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_audit_not_gate" -Needle '"export_denial_audit_satisfies_export_gate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_denial_not_binding" -Needle '"denial_event_is_export_binding": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_export_projection_locator" -Needle '"local_projection_locator": "snapshot.current.provider_minimal"' -TimeoutSeconds 1
    Assert-LogDoesNotContain -Name "protocol:provider_context_export_did_not_fake_request_envelope" -Needle "raios.provider_request_envelope.v0"

    Send-AgentCommand -Command "agent provider.context_gate provider_minimal" -ExpectedMarker "RAIOS_AGENT_END provider.context_gate"
    Assert-LogContains -Name "protocol:provider_context_gate_schema" -Needle '"schema": "raios.provider_context_export_gate_state.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_export_disabled" -Needle '"provider_export": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_injection_disabled" -Needle '"automatic_context_injection": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_no_body_attachment" -Needle '"context_attached_to_provider_body": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_no_write" -Needle '"provider_write": "not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_binding_missing" -Needle '"binding_validation_reason": "provider_context_export_audit_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_request_binding_missing" -Needle '"provider_request_binding": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_audit_binding_missing" -Needle '"provider_export_audit_binding": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_current_boot_gate_false" -Needle '"satisfies_current_boot_export_gate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_can_export_false" -Needle '"can_export": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent provider.context_gate_selftest provider_minimal" -ExpectedMarker "RAIOS_AGENT_END provider.context_gate_selftest"
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_schema" -Needle '"schema": "raios.provider_context_gate_negative_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_test_infrastructure" -Needle '"test_infrastructure": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_no_global_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_no_request_envelope" -Needle '"creates_provider_request_envelope": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_no_positive_bindings" -Needle '"creates_positive_binding_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_no_write" -Needle '"provider_write": "not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_injection_disabled" -Needle '"automatic_context_injection": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_no_body_attachment" -Needle '"context_attached_to_provider_body": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_count" -Needle '"case_count": 16' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_stale_request" -Needle '"case": "stale_dropped_request_binding_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_stale_envelope" -Needle '"case": "stale_dropped_envelope_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_previous_boot" -Needle '"case": "previous_boot_or_unretained_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_denial_schema" -Needle '"case": "denial_schema_substitution"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_positive_substitution" -Needle '"case": "positive_record_substitution"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_body_hash" -Needle '"case": "request_body_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_envelope_hash" -Needle '"case": "request_envelope_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_binding_hash" -Needle '"case": "request_binding_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_packet_hash" -Needle '"case": "provider_minimal_packet_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_exported_hash" -Needle '"case": "exported_field_list_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_omitted_hash" -Needle '"case": "omitted_field_list_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_stale_reason" -Needle '"actual_reason": "binding_stale_or_dropped_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_denial_reason" -Needle '"actual_reason": "binding_denied_schema_or_wrong_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_context_hash_reason" -Needle '"actual_reason": "binding_provider_minimal_packet_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_gate_false" -Needle '"satisfies_current_boot_export_gate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_gate_selftest_can_export_false" -Needle '"can_export": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent provider.context_injection_gate provider_minimal" -ExpectedMarker "RAIOS_AGENT_END provider.context_injection_gate"
    Assert-LogContains -Name "protocol:provider_context_injection_gate_schema" -Needle '"schema": "raios.provider_context_injection_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_export_disabled" -Needle '"provider_export": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_injection_disabled" -Needle '"automatic_context_injection": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_no_body_attachment" -Needle '"context_attached_to_provider_body": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_no_write" -Needle '"provider_write": "not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_final_schema" -Needle '"final_authorization_schema": "raios.provider_context_injection_authorization.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_final_missing" -Needle '"final_authorization": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_body_check_not_attempted" -Needle '"final_prewrite_body_check": "not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_can_attach_false" -Needle '"can_attach_context": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_current_boot_gate_false" -Needle '"satisfies_current_boot_export_gate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_blocked_final" -Needle '"reason": "final_injection_authorization_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_blocked_disabled" -Needle '"reason": "automatic_context_injection_disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_gate_required_authorization" -Needle '"raios.provider_context_injection_authorization.v0"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent provider.context_injection_gate_selftest provider_minimal" -ExpectedMarker "RAIOS_AGENT_END provider.context_injection_gate_selftest"
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_schema" -Needle '"schema": "raios.provider_context_injection_gate_negative_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_test_infrastructure" -Needle '"test_infrastructure": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_no_global_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_no_request_envelope" -Needle '"creates_provider_request_envelope": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_no_positive_bindings" -Needle '"creates_positive_binding_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_no_final_records" -Needle '"creates_final_authorization_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_no_write" -Needle '"provider_write": "not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_injection_disabled" -Needle '"automatic_context_injection": "disabled"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_no_attachment" -Needle '"context_attached_to_provider_body": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_can_attach_false" -Needle '"can_attach_context": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_count" -Needle '"case_count": 7' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_missing" -Needle '"case": "missing_final_authorization"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_stale" -Needle '"case": "stale_dropped_final_authorization_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_schema_substitution" -Needle '"case": "final_authorization_schema_substitution"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_positive_substitution" -Needle '"case": "substituted_positive_final_authorization_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_body_hash" -Needle '"case": "final_authorization_body_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_trust_downgrade" -Needle '"case": "final_authorization_trust_downgrade"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_body_attachment_attempt" -Needle '"case": "body_attachment_without_final_authorization"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_missing_reason" -Needle '"actual_reason": "final_injection_authorization_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_stale_reason" -Needle '"actual_reason": "final_injection_authorization_stale_or_dropped_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_schema_reason" -Needle '"actual_reason": "final_injection_authorization_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_substitution_reason" -Needle '"actual_reason": "final_injection_authorization_substituted_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_body_hash_reason" -Needle '"actual_reason": "final_prewrite_body_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_trust_reason" -Needle '"actual_reason": "final_provider_trust_downgraded_before_write"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:provider_context_injection_selftest_attachment_reason" -Needle '"actual_reason": "body_attachment_without_final_authorization"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.query" -ExpectedMarker "RAIOS_AGENT_END memory.query"
    Assert-LogContains -Name "protocol:memory_query_schema" -Needle '"schema": "memory.query.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_query_snapshot_record" -Needle "snapshot.current" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_query_projection_record" -Needle "snapshot.current.provider_minimal" -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.trace snapshot.current" -ExpectedMarker "RAIOS_AGENT_END memory.trace"
    Assert-LogContains -Name "protocol:memory_trace_schema" -Needle '"schema": "memory.trace.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_trace_snapshot_source" -Needle '"source_method": "system.snapshot"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.recent_events" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events"
    Assert-LogContains -Name "protocol:memory_recent_events_schema" -Needle '"schema": "event.log.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_record_schema" -Needle '"record_schema": "audit.event.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_scope" -Needle '"scope": "current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_bounded" -Needle '"bounded": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_record_id" -Needle '"id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_sequence" -Needle '"sequence":' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_classification" -Needle '"classification": "public"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_read_outcome" -Needle '"outcome": "response"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_read_kind" -Needle '"kind": "agent_protocol.read_response"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_snapshot_source" -Needle '"source_method": "system.snapshot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_export_source" -Needle '"source_method": "provider.context_export"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_export_capability" -Needle '"requested_capability": "cap.provider.context_export"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_export_risk" -Needle '"risk": "export"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_request_binding_denied_kind" -Needle '"kind": "provider_context_export.request_binding_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_request_binding_denied_outcome" -Needle '"outcome": "denied_not_bound"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_export_audit_kind" -Needle '"kind": "provider_context_export.denial_audit"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_export_audit_outcome" -Needle '"outcome": "denied_no_provider_write"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_provider_write_not_attempted" -Needle "provider_write_not_attempted" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_evidence" -Needle '"evidence":' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_request_denial_bindings" -Needle '"bindings": {"schema": "raios.provider_request_binding_denial.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_export_denial_bindings" -Needle '"bindings": {"schema": "raios.provider_context_export_denial_audit.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_binding_gate_false" -Needle '"satisfies_current_boot_export_gate": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_binding_packet_hash" -Needle '"hashes": {"packet_canonicalization": "raios.provider_minimal.packet.canonical.v0", "projected_packet_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_binding_exported_fields_hash" -Needle '"exported_field_list_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:memory_recent_events_binding_omitted_fields_hash" -Needle '"omitted_field_list_hash": "sha256:' -TimeoutSeconds 1
    Assert-LogDoesNotContain -Name "protocol:no_positive_request_binding_schema" -Needle '"schema": "raios.provider_request_binding.v0"'
    Assert-LogDoesNotContain -Name "protocol:no_positive_export_audit_binding_schema" -Needle '"schema": "raios.provider_context_export_audit_binding.v0"'
    Assert-LogDoesNotContain -Name "protocol:no_positive_current_boot_export_gate" -Needle '"satisfies_current_boot_export_gate": true'
    Assert-LogDoesNotContain -Name "protocol:no_positive_export_authorization" -Needle '"positive_export_authorization": true'
    Assert-LogContains -Name "protocol:memory_recent_events_ram_only" -Needle '"persistence": "none"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.record_observation" -ExpectedMarker "RAIOS_AGENT_END memory.record_observation"
    Assert-LogContains -Name "policy:memory_record_observation_method" -Needle '"method": "memory.record_observation"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:memory_record_observation_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:memory_record_observation_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:memory_record_observation_audit_event_id" -Needle '"audit_event_id": "event.current_boot.' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent audit.events 8" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events"
    Assert-LogContains -Name "protocol:audit_events_alias_schema" -Needle '"schema": "event.log.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:audit_events_limit" -Needle '"limit": 8' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:audit_events_denied_outcome" -Needle '"outcome": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:audit_events_denied_kind" -Needle '"kind": "agent_protocol.capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:audit_events_denied_source" -Needle '"source_method": "memory.record_observation"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:audit_events_denied_capability" -Needle '"requested_capability": "cap.memory.mutate"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.propose_policy" -ExpectedMarker "RAIOS_AGENT_END memory.propose_policy"
    Assert-LogContains -Name "policy:memory_propose_policy_method" -Needle '"method": "memory.propose_policy"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.supersede_fact" -ExpectedMarker "RAIOS_AGENT_END memory.supersede_fact"
    Assert-LogContains -Name "policy:memory_supersede_fact_method" -Needle '"method": "memory.supersede_fact"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.redact" -ExpectedMarker "RAIOS_AGENT_END memory.redact"
    Assert-LogContains -Name "policy:memory_redact_method" -Needle '"method": "memory.redact"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent memory.compact" -ExpectedMarker "RAIOS_AGENT_END memory.compact"
    Assert-LogContains -Name "policy:memory_compact_method" -Needle '"method": "memory.compact"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:memory_audit_required" -Needle "raios.audit_record.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:memory_persistence_required" -Needle "raios.memory_persistence.v0" -TimeoutSeconds 1

    Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral"
    Assert-LogContains -Name "policy:mutating_load_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:mutating_load_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:vm_report_required" -Needle "raios.vm_test_report.v0" -TimeoutSeconds 1

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
