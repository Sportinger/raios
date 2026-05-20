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
        [string]$ExpectedMarker,
        [string]$Name = ""
    )

    Send-SerialText -Port $SerialTcpPort -Text "$Command`r" -TimeoutSeconds $TimeoutSeconds
    $predicateName = if ($Name.Length -gt 0) { $Name } else { "command:$Command" }
    Assert-LogContains -Name $predicateName -Needle $ExpectedMarker -TimeoutSeconds $TimeoutSeconds
}

function Get-LastAgentResponseJson {
    param(
        [string]$Method
    )

    $content = Get-Content -Raw -LiteralPath $SerialLog -ErrorAction Stop
    $begin = "RAIOS_AGENT_BEGIN $Method"
    $end = "RAIOS_AGENT_END $Method"
    $beginIndex = $content.LastIndexOf($begin, [System.StringComparison]::Ordinal)
    if ($beginIndex -lt 0) {
        throw "No agent response for method '$Method' found in $SerialLog"
    }
    $jsonStart = $content.IndexOf("{", $beginIndex, [System.StringComparison]::Ordinal)
    $endIndex = $content.IndexOf($end, $jsonStart, [System.StringComparison]::Ordinal)
    if ($jsonStart -lt 0 -or $endIndex -lt 0) {
        throw "Incomplete agent response for method '$Method' found in $SerialLog"
    }
    $json = $content.Substring($jsonStart, $endIndex - $jsonStart).Trim()
    return $json | ConvertFrom-Json
}

function Assert-CurrentBootEventId {
    param(
        [string]$Name,
        [string]$Value
    )

    $passed = $Value -match '^event\.current_boot\.[0-9]{8}$'
    Add-Predicate -Name $Name -Expected "current_boot_event_id" -Passed $passed -Actual $Value
    if (-not $passed) {
        throw "Expected current-boot event id for '$Name', got '$Value'"
    }
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
            "agent module.grant_diagnostic",
            "agent module.grant_diagnostic <valid hash reference>",
            "agent module.grant_diagnostic_selftest",
            "agent module.audit_rollback_diagnostic",
            "agent module.audit_rollback_diagnostic <valid hash reference>",
            "agent module.audit_rollback_diagnostic_selftest",
            "agent module.service_slot_diagnostic",
            "agent module.service_slot_diagnostic <valid hash reference>",
            "agent module.service_slot_diagnostic_selftest",
            "agent module.load_gate_retained_selftest",
            "agent module.load_gate_audit_rollback_selftest",
            "agent module.load_gate_service_slot_selftest",
            "module.load_ephemeral",
            "agent audit.events 32"
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

    Send-AgentCommand -Command "agent module.grant_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic"
    Assert-LogContains -Name "protocol:module_grant_diag_schema" -Needle '"schema": "raios.module_computed_grant_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_absent_reason" -Needle '"validation_reason": "computed_capability_grant_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_computed_missing" -Needle '"computed_capability_grant": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_no_capability" -Needle '"grants_capability": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_no_load_grant" -Needle '"grants_load_now": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_no_guest_load" -Needle '"authorizes_guest_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_can_load_false" -Needle '"can_load_now": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_loader_unavailable" -Needle '"loader", "state": "unavailable"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_slot_unallocated" -Needle '"service_slot", "state": "unallocated"' -TimeoutSeconds 1

    $moduleGrantManifestHash = "1111111111111111111111111111111111111111111111111111111111111111"
    $moduleGrantArtifactHash = "2222222222222222222222222222222222222222222222222222222222222222"
    $moduleGrantReportHash = "3333333333333333333333333333333333333333333333333333333333333333"
    $moduleGrantAttestationHash = "4444444444444444444444444444444444444444444444444444444444444444"
    $moduleGrantCanonical = @(
        "canonicalization=raios.computed_capability_grant.canonical.v0",
        "schema=raios.computed_capability_grant.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "manifest_sha256=$moduleGrantManifestHash",
        "candidate_artifact_sha256=$moduleGrantArtifactHash",
        "vm_test_report_sha256=$moduleGrantReportHash",
        "local_attestation_sha256=$moduleGrantAttestationHash",
        "grants_load_now=false",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleGrantHash = Get-TextSha256 -Text $moduleGrantCanonical
    $moduleGrantCommand = "agent module.grant_diagnostic $moduleGrantHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantReportHash $moduleGrantAttestationHash"

    Send-AgentCommand -Command $moduleGrantCommand -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic"
    Assert-LogContains -Name "protocol:module_grant_diag_valid_status" -Needle '"validation_status": "valid_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_valid_retained" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_retained_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_recorded_event_id" -Needle '"recorded_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_retained_matches" -Needle '"matches_current_reference": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_candidate_present" -Needle '"computed_candidate_present": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_valid_still_no_capability" -Needle '"grants_capability": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_valid_still_no_load" -Needle '"can_load_now": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_diag_valid_hash_echo" -Needle "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" -TimeoutSeconds 1

    $moduleGrantResponse = Get-LastAgentResponseJson -Method "module.grant_diagnostic"
    $moduleAuditRetainedReferenceEventId = [string]$moduleGrantResponse.body.result.retained_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:module_grant_retained_reference_event_id_captured" -Value $moduleAuditRetainedReferenceEventId

    Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "command:module.load_ephemeral.pre_audit"
    $modulePreAuditLoadResponse = Get-LastAgentResponseJson -Method "module.load_ephemeral"
    $moduleAuditDenialEventId = [string]$modulePreAuditLoadResponse.body.event_id
    Assert-CurrentBootEventId -Name "protocol:module_audit_denial_event_id_captured" -Value $moduleAuditDenialEventId
    Assert-LogContains -Name "policy:module_pre_audit_load_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_grant_retained" -Needle '"computed_capability_grant": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_audit_missing" -Needle '"durable_audit_record": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_rollback_missing" -Needle '"rollback_plan": "missing"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_diagnostic"
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_schema" -Needle '"schema": "raios.module_audit_rollback_reference_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_no_slots" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_absent_reason" -Needle '"validation_reason": "audit_rollback_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    $moduleAuditLocalApprovalHash = "6666666666666666666666666666666666666666666666666666666666666666"
    $moduleAuditPreInventoryHash = "7777777777777777777777777777777777777777777777777777777777777777"
    $moduleAuditCleanupHash = "8888888888888888888888888888888888888888888888888888888888888888"
    $moduleAuditRamOnlyServiceSlotId = "ram_only:svc.test.0001"
    $moduleRollbackCanonical = @(
        "canonicalization=raios.rollback_plan.canonical.v0",
        "schema=raios.rollback_plan.v0",
        "load_mode=ram_only",
        "scope=current_boot",
        "artifact_sha256=$moduleGrantArtifactHash",
        "pre_load_service_inventory_sha256=$moduleAuditPreInventoryHash",
        "ram_only_service_slot_id=$moduleAuditRamOnlyServiceSlotId",
        "cleanup_actions_sha256=$moduleAuditCleanupHash",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleRollbackHash = Get-TextSha256 -Text $moduleRollbackCanonical

    $moduleWrongAuditDenialEventId = $moduleAuditRetainedReferenceEventId
    $moduleWrongAuditCanonical = @(
        "canonicalization=raios.audit_record.canonical.v0",
        "schema=raios.audit_record.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "denial_event_id=$moduleWrongAuditDenialEventId",
        "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
        "computed_capability_grant_sha256=$moduleGrantHash",
        "manifest_sha256=$moduleGrantManifestHash",
        "candidate_artifact_sha256=$moduleGrantArtifactHash",
        "vm_test_report_sha256=$moduleGrantReportHash",
        "local_attestation_sha256=$moduleGrantAttestationHash",
        "local_approval_sha256=$moduleAuditLocalApprovalHash",
        "rollback_plan_sha256=$moduleRollbackHash",
        "ram_only_service_slot_id=$moduleAuditRamOnlyServiceSlotId",
        "grants_load_now=false",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleWrongAuditHash = Get-TextSha256 -Text $moduleWrongAuditCanonical
    $moduleWrongAuditCommand = "agent module.audit_rollback_diagnostic $moduleWrongAuditHash $moduleRollbackHash $moduleGrantHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantReportHash $moduleGrantAttestationHash $moduleAuditLocalApprovalHash $moduleAuditPreInventoryHash $moduleAuditCleanupHash $moduleWrongAuditDenialEventId $moduleAuditRetainedReferenceEventId $moduleAuditRamOnlyServiceSlotId"

    Send-AgentCommand -Command $moduleWrongAuditCommand -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_diagnostic" -Name "command:module.audit_rollback_diagnostic.wrong_denial"
    Assert-LogContains -Name "protocol:module_wrong_audit_rollback_diag_valid_status" -Needle '"validation_status": "valid_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "command:module.load_ephemeral.rejected_audit_ref"
    Assert-LogContains -Name "policy:module_rejected_audit_reference_state" -Needle '"state": "rejected"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rejected_audit_reference_status" -Needle '"status": "rejected"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rejected_audit_reference_reason" -Needle '"reason": "retained_audit_rollback_reference_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rejected_audit_state" -Needle '"durable_audit_record": "rejected_retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rejected_rollback_state" -Needle '"rollback_plan": "rejected_retained_reference"' -TimeoutSeconds 1

    $moduleAuditCanonical = @(
        "canonicalization=raios.audit_record.canonical.v0",
        "schema=raios.audit_record.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "denial_event_id=$moduleAuditDenialEventId",
        "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
        "computed_capability_grant_sha256=$moduleGrantHash",
        "manifest_sha256=$moduleGrantManifestHash",
        "candidate_artifact_sha256=$moduleGrantArtifactHash",
        "vm_test_report_sha256=$moduleGrantReportHash",
        "local_attestation_sha256=$moduleGrantAttestationHash",
        "local_approval_sha256=$moduleAuditLocalApprovalHash",
        "rollback_plan_sha256=$moduleRollbackHash",
        "ram_only_service_slot_id=$moduleAuditRamOnlyServiceSlotId",
        "grants_load_now=false",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleAuditHash = Get-TextSha256 -Text $moduleAuditCanonical
    $moduleAuditCommand = "agent module.audit_rollback_diagnostic $moduleAuditHash $moduleRollbackHash $moduleGrantHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantReportHash $moduleGrantAttestationHash $moduleAuditLocalApprovalHash $moduleAuditPreInventoryHash $moduleAuditCleanupHash $moduleAuditDenialEventId $moduleAuditRetainedReferenceEventId $moduleAuditRamOnlyServiceSlotId"

    Send-AgentCommand -Command $moduleAuditCommand -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_diagnostic"
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_valid_status" -Needle '"validation_status": "valid_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_valid_reason" -Needle '"validation_reason": "audit_rollback_reference_valid_but_loader_and_slot_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_audit_hash_echo" -Needle "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_rollback_hash_echo" -Needle "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_grant_hash_echo" -Needle "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_audit_ref_present" -Needle '"audit_record_hash_reference_present": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_rollback_ref_present" -Needle '"rollback_plan_hash_reference_present": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_retention_mutation" -Needle '"global_event_log_mutation": "valid_hash_reference_retention_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_retained_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_retained_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_retained_recorded_event_id" -Needle '"recorded_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_retained_matches" -Needle '"matches_current_reference": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_no_durable_write" -Needle '"durable_audit_written": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_not_installed" -Needle '"rollback_plan_installed": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_can_load_false" -Needle '"can_load_now": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_diag_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1

    $moduleAuditResponse = Get-LastAgentResponseJson -Method "module.audit_rollback_diagnostic"
    $moduleServiceSlotRetainedAuditEventId = [string]$moduleAuditResponse.body.result.retained_audit_rollback_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:module_service_slot_retained_audit_reference_event_id_captured" -Value $moduleServiceSlotRetainedAuditEventId

    Send-AgentCommand -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic"
    Assert-LogContains -Name "protocol:module_service_slot_diag_schema" -Needle '"schema": "raios.module_service_slot_reservation_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_no_allocation" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_no_inventory_records" -Needle '"creates_service_inventory_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_absent_reason" -Needle '"validation_reason": "service_slot_reservation_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    $moduleServiceSlotCanonical = @(
        "canonicalization=raios.module_service_slot_reservation.canonical.v0",
        "schema=raios.module_service_slot_reservation.v0",
        "load_mode=ram_only",
        "scope=current_boot",
        "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
        "retained_audit_rollback_reference_event_id=$moduleServiceSlotRetainedAuditEventId",
        "computed_capability_grant_sha256=$moduleGrantHash",
        "audit_record_sha256=$moduleAuditHash",
        "rollback_plan_sha256=$moduleRollbackHash",
        "pre_load_service_inventory_sha256=$moduleAuditPreInventoryHash",
        "ram_only_service_slot_id=$moduleAuditRamOnlyServiceSlotId",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleServiceSlotReservationHash = Get-TextSha256 -Text $moduleServiceSlotCanonical
    $moduleServiceSlotCommand = "agent module.service_slot_diagnostic $moduleServiceSlotReservationHash $moduleAuditRetainedReferenceEventId $moduleServiceSlotRetainedAuditEventId $moduleGrantHash $moduleAuditHash $moduleRollbackHash $moduleAuditPreInventoryHash $moduleAuditRamOnlyServiceSlotId"

    Send-AgentCommand -Command $moduleServiceSlotCommand -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic"
    Assert-LogContains -Name "protocol:module_service_slot_diag_valid_status" -Needle '"validation_status": "valid_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_valid_reason" -Needle '"validation_reason": "service_slot_reservation_valid_but_allocator_and_loader_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_reservation_hash_echo" -Needle "`"reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_grant_hash_echo" -Needle "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_audit_hash_echo" -Needle "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_rollback_hash_echo" -Needle "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_inventory_hash_echo" -Needle "`"pre_load_service_inventory_hash`": `"sha256:$moduleAuditPreInventoryHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_slot_echo" -Needle "`"ram_only_service_slot_id`": `"$moduleAuditRamOnlyServiceSlotId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_retention_mutation" -Needle '"global_event_log_mutation": "valid_hash_reference_retention_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_retained_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_retained_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_retained_recorded_event_id" -Needle '"recorded_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_retained_matches" -Needle '"matches_current_reference": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_policy_present" -Needle '"reservation_reference_present": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_policy_no_reserved_slot" -Needle '"service_slot_reserved": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_policy_can_load_false" -Needle '"can_load_now": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_diag_policy_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.grant_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_grant_selftest_schema" -Needle '"schema": "raios.module_computed_grant_diagnostic_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_no_artifacts" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_no_load" -Needle '"loads_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_case_count" -Needle '"case_count": 5' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_valid_case" -Needle '"case": "accepted_current_boot_reference_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_valid_status" -Needle '"actual_status": "valid_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_stale_case" -Needle '"case": "stale_previous_boot_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_stale_status" -Needle '"actual_status": "stale_or_non_current_boot_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_mismatch_case" -Needle '"case": "mismatched_manifest_hash_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_mismatch_status" -Needle '"actual_status": "mismatched_computed_grant_hash"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_wrong_policy_case" -Needle '"case": "grants_load_now_or_wrong_policy_hash"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_reference_diagnostic_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_no_artifacts" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_no_slots" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_count" -Needle '"case_count": 10' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_valid_case" -Needle '"case": "accepted_current_boot_reference_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_valid_status" -Needle '"actual_status": "valid_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_stale_case" -Needle '"case": "stale_previous_boot_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_event_case" -Needle '"case": "previous_boot_denial_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_audit_schema_case" -Needle '"case": "audit_record_schema_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_rollback_schema_case" -Needle '"case": "rollback_plan_schema_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_substituted_case" -Needle '"case": "substituted_audit_record_hash"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_rollback_hash_case" -Needle '"case": "mismatched_rollback_plan_hash"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_grant_hash_case" -Needle '"case": "mismatched_computed_grant_hash"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_slot_case" -Needle '"case": "invalid_ram_only_service_slot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.service_slot_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_service_slot_selftest_schema" -Needle '"schema": "raios.module_service_slot_reservation_diagnostic_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_no_records" -Needle '"creates_service_slot_reservation_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_no_slots" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_no_load" -Needle '"loads_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_count" -Needle '"case_count": 5' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_valid_case" -Needle '"case": "accepted_current_boot_reservation_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_valid_status" -Needle '"actual_status": "valid_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_valid_reason" -Needle '"actual_reason": "service_slot_reservation_valid_but_allocator_and_loader_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_stale_case" -Needle '"case": "stale_previous_boot_reservation"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_stale_status" -Needle '"actual_status": "stale_or_non_current_boot_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_mismatch_case" -Needle '"case": "mismatched_reservation_hash"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_mismatch_reason" -Needle '"actual_reason": "service_slot_reservation_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_slot_case" -Needle '"case": "invalid_ram_only_service_slot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_slot_reason" -Needle '"actual_reason": "ram_only_service_slot_id_invalid"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_retained_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_retained_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_schema" -Needle '"schema": "raios.module_load_gate_retained_reference_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_no_records" -Needle '"creates_retained_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_no_load" -Needle '"loads_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_count" -Needle '"case_count": 7' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_missing_case" -Needle '"case": "missing_retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_valid_case" -Needle '"case": "accepted_current_boot_reference_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_valid_status" -Needle '"actual_status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_valid_reason" -Needle '"actual_reason": "retained_computed_grant_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_stale_case" -Needle '"case": "stale_dropped_retained_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_stale_reason" -Needle '"actual_reason": "retained_reference_stale_or_dropped_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_previous_boot_case" -Needle '"case": "previous_boot_or_unretained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_previous_boot_reason" -Needle '"actual_reason": "retained_reference_previous_boot_or_unretained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_wrong_schema_case" -Needle '"case": "wrong_schema_or_variant_substitution"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_wrong_schema_reason" -Needle '"actual_reason": "retained_reference_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_substituted_case" -Needle '"case": "substituted_retained_reference_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_substituted_reason" -Needle '"actual_reason": "retained_reference_substituted_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_hash_case" -Needle '"case": "mismatched_computed_grant_hash"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_hash_reason" -Needle '"actual_reason": "retained_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_retained_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_audit_rollback_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_audit_rollback_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_schema" -Needle '"schema": "raios.module_load_gate_audit_rollback_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_no_retained_records" -Needle '"creates_retained_audit_rollback_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_no_slots" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_no_load" -Needle '"loads_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_count" -Needle '"case_count": 23' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_missing_retained_audit_ref" -Needle '"case": "missing_retained_audit_rollback_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_missing_retained_audit_ref_reason" -Needle '"actual_reason": "retained_audit_rollback_reference_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_stale_retained_audit_ref" -Needle '"case": "stale_dropped_retained_audit_rollback_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_stale_retained_audit_ref_reason" -Needle '"actual_reason": "retained_audit_rollback_reference_stale_or_dropped_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_previous_retained_audit_ref" -Needle '"case": "previous_boot_or_unretained_audit_rollback_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_previous_retained_audit_ref_reason" -Needle '"actual_reason": "retained_audit_rollback_reference_previous_boot_or_unretained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_wrong_schema_retained_audit_ref" -Needle '"case": "retained_audit_rollback_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_wrong_schema_retained_audit_ref_reason" -Needle '"actual_reason": "retained_audit_rollback_reference_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_substituted_retained_audit_ref" -Needle '"case": "substituted_retained_audit_rollback_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_substituted_retained_audit_ref_reason" -Needle '"actual_reason": "retained_audit_rollback_reference_substituted_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_grant_hash_mismatch" -Needle '"case": "retained_audit_rollback_computed_grant_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_grant_hash_mismatch_reason" -Needle '"actual_reason": "retained_audit_rollback_computed_grant_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_audit_hash_mismatch" -Needle '"case": "retained_audit_record_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_audit_hash_mismatch_reason" -Needle '"actual_reason": "retained_audit_record_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_rollback_hash_mismatch" -Needle '"case": "retained_rollback_plan_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_rollback_hash_mismatch_reason" -Needle '"actual_reason": "retained_rollback_plan_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_slot_mismatch" -Needle '"case": "retained_audit_rollback_service_slot_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_retained_slot_mismatch_reason" -Needle '"actual_reason": "retained_audit_rollback_service_slot_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_missing_audit" -Needle '"case": "missing_durable_audit_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_missing_audit_reason" -Needle '"actual_reason": "durable_audit_record_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_missing_rollback" -Needle '"case": "missing_rollback_plan"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_missing_rollback_reason" -Needle '"actual_reason": "rollback_plan_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_audit_schema_mismatch" -Needle '"case": "durable_audit_record_schema_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_audit_schema_reason" -Needle '"actual_reason": "durable_audit_record_schema_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_rollback_schema_mismatch" -Needle '"case": "rollback_plan_schema_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_rollback_schema_reason" -Needle '"actual_reason": "rollback_plan_schema_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_valid_case" -Needle '"case": "valid_audit_and_rollback_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_valid_status" -Needle '"actual_status": "validated_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_valid_reason" -Needle '"actual_reason": "loader_and_service_slot_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_grant_mismatch" -Needle '"case": "audit_retained_grant_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_manifest_mismatch" -Needle '"case": "audit_manifest_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_artifact_mismatch" -Needle '"case": "audit_artifact_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_report_mismatch" -Needle '"case": "audit_vm_report_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_attestation_mismatch" -Needle '"case": "audit_local_attestation_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_approval_mismatch" -Needle '"case": "local_approval_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_rollback_hash_mismatch" -Needle '"case": "audit_rollback_plan_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_rollback_artifact_mismatch" -Needle '"case": "rollback_artifact_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_rollback_slot_mismatch" -Needle '"case": "rollback_service_slot_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_rollback_slot_reason" -Needle '"actual_reason": "rollback_service_slot_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_bindings" -Needle '"required_bindings":' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_service_slot_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_service_slot_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_schema" -Needle '"schema": "raios.module_load_gate_service_slot_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_no_records" -Needle '"creates_service_slot_reservation_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_no_slots" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_no_inventory_records" -Needle '"creates_service_inventory_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_no_load" -Needle '"loads_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_count" -Needle '"case_count": 13' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_missing_case" -Needle '"case": "missing_retained_service_slot_reservation"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_missing_reason" -Needle '"actual_reason": "retained_service_slot_reservation_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_valid_case" -Needle '"case": "accepted_current_boot_reservation_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_valid_status" -Needle '"actual_status": "retained_hash_reference_only_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_valid_state" -Needle '"actual_service_slot_state": "retained_hash_reference_only_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_valid_hash_exposed" -Needle '"accepted_service_slot_reservation_hash": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_stale_case" -Needle '"case": "stale_dropped_retained_service_slot_reservation_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_stale_reason" -Needle '"actual_reason": "retained_service_slot_reservation_stale_or_dropped_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_wrong_schema_case" -Needle '"case": "retained_service_slot_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_wrong_schema_reason" -Needle '"actual_reason": "retained_service_slot_reservation_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_substituted_case" -Needle '"case": "substituted_retained_service_slot_reservation"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_substituted_reason" -Needle '"actual_reason": "retained_service_slot_reservation_substituted_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_grant_schema_case" -Needle '"case": "retained_service_slot_grant_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_audit_schema_case" -Needle '"case": "retained_service_slot_audit_rollback_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_grant_hash_mismatch" -Needle '"case": "retained_service_slot_computed_grant_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_grant_hash_reason" -Needle '"actual_reason": "retained_service_slot_reservation_computed_grant_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_audit_hash_mismatch" -Needle '"case": "retained_service_slot_audit_record_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_audit_hash_reason" -Needle '"actual_reason": "retained_service_slot_reservation_audit_record_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_rollback_hash_mismatch" -Needle '"case": "retained_service_slot_rollback_plan_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_rollback_hash_reason" -Needle '"actual_reason": "retained_service_slot_reservation_rollback_plan_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_inventory_mismatch" -Needle '"case": "retained_service_slot_inventory_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_inventory_reason" -Needle '"actual_reason": "retained_service_slot_reservation_pre_load_inventory_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_slot_mismatch" -Needle '"case": "retained_service_slot_service_slot_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_slot_reason" -Needle '"actual_reason": "retained_service_slot_reservation_service_slot_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_hash_mismatch" -Needle '"case": "retained_service_slot_reservation_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_hash_reason" -Needle '"actual_reason": "retained_service_slot_reservation_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_rejected_state" -Needle '"actual_service_slot_state": "rejected_retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_rejected_hash_not_exposed" -Needle '"accepted_service_slot_reservation_hash": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_service_slot_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral"
    Assert-LogContains -Name "policy:mutating_load_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_gate_schema" -Needle '"schema": "raios.module_load_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:mutating_load_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_mode_ram_only" -Needle '"load_mode": "ram_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_capability" -Needle '"requested_capability": "cap.module.load_ephemeral"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_target" -Needle '"target": "live_service_graph"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_manifest_missing" -Needle '"module_manifest": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:candidate_artifact_missing" -Needle '"candidate_artifact": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_vm_report_missing" -Needle '"vm_test_report": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_attestation_missing" -Needle '"local_attestation": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_computed_grant_retained" -Needle '"computed_capability_grant": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_reference" -Needle '"retained_computed_grant_reference": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_present" -Needle '"state": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_hash" -Needle "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_approval_missing" -Needle '"local_approval": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rollback_reference_retained" -Needle '"rollback_plan": "retained_hash_reference_only_not_installed"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_reference_retained" -Needle '"durable_audit_record": "retained_hash_reference_only_not_durable"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_audit_rollback_reference" -Needle '"retained_audit_rollback_reference": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_audit_rollback_present" -Needle '"schema": "raios.module_audit_rollback_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_audit_rollback_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_audit_rollback_event_id" -Needle '"retained_audit_rollback_reference_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_audit_hash" -Needle "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_rollback_hash" -Needle "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_unavailable" -Needle '"loader": "unavailable"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_retained" -Needle '"service_slot": "retained_hash_reference_only_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_reference" -Needle '"retained_service_slot_reservation": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_present" -Needle '"schema": "raios.module_service_slot_reservation.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_status" -Needle '"status": "retained_hash_reference_only_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_event_id" -Needle '"retained_service_slot_reservation_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_reason" -Needle '"reason": "retained_service_slot_reservation_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_hash" -Needle "`"reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_no_allocation" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_service_slot_no_inventory" -Needle '"creates_service_inventory_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_artifact_not_loaded" -Needle '"artifact_loaded": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_not_started" -Needle '"service_started": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_manifest_missing_reason" -Needle '"reason": "module_manifest_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:candidate_artifact_missing_reason" -Needle '"reason": "candidate_artifact_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_vm_report_missing_reason" -Needle '"reason": "vm_test_report_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_attestation_missing_reason" -Needle '"reason": "local_attestation_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_computed_grant_retained_not_authorizing" -Needle '"reason": "retained_computed_grant_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_reference_reason" -Needle '"reason": "retained_audit_record_reference_not_durable"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rollback_reference_reason" -Needle '"reason": "retained_rollback_plan_reference_not_installed"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_loader_unimplemented_reason" -Needle '"reason": "module_loader_unimplemented"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_manifest_required" -Needle "raios.module_manifest.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:candidate_artifact_sha256_required" -Needle "candidate_artifact_sha256" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:vm_report_required" -Needle "raios.vm_test_report.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:local_attestation_required" -Needle "raios.local_attestation.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_computed_grant_required" -Needle "computed_capability_grant" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_approval_required" -Needle "local_approval" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_record_required" -Needle "raios.audit_record.v0" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rollback_required" -Needle "rollback_plan" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_required" -Needle "ram_only_service_slot" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_rollback_requirements_schema" -Needle '"schema": "raios.module_load_gate_audit_rollback_requirements.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_rollback_requirements_status" -Needle '"status": "required_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_rollback_requirements_no_writes" -Needle '"writes_enabled": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_rollback_requirements_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_rollback_requirements_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_record_schema" -Needle '"schema": "raios.audit_record.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rollback_plan_schema" -Needle '"schema": "raios.rollback_plan.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_retained_reference_required" -Needle '"retained_computed_grant_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_rollback_hash_retained" -Needle "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_slot_id_retained" -Needle "`"ram_only_service_slot_id`": `"$moduleAuditRamOnlyServiceSlotId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_hash_retained" -Needle "`"service_slot_reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_requirement_retained" -Needle '"ram_only_service_slot": {"state": "retained_hash_reference_only_not_allocated", "reason": "retained_service_slot_reservation_not_allocated", "required": true, "allocates_service_slot": false}' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_manifest_hash_retained" -Needle "`"manifest_hash`": `"sha256:$moduleGrantManifestHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_artifact_hash_retained" -Needle "`"artifact_hash`": `"sha256:$moduleGrantArtifactHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_vm_report_hash_retained" -Needle "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_attestation_hash_retained" -Needle "`"local_attestation_hash`": `"sha256:$moduleGrantAttestationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_inventory_unchanged" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent audit.events 32" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events"
    Assert-LogContains -Name "protocol:module_grant_audit_source" -Needle '"source_method": "module.grant_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_kind" -Needle '"kind": "module.computed_grant_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_capability" -Needle '"requested_capability": "cap.module.grant_diagnostic.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_computed_grant_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_binding_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_no_capability" -Needle '"grants_capability": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_grant_audit_hash" -Needle "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_source" -Needle '"source_method": "module.audit_rollback_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_kind" -Needle '"kind": "module.audit_rollback_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_audit_rollback_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_hash" -Needle "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_source" -Needle '"source_method": "module.service_slot_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_kind" -Needle '"kind": "module.service_slot_reservation.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_service_slot_reservation.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_reservation_hash" -Needle "`"reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_no_allocation" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_no_inventory" -Needle '"creates_service_inventory_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_service_slot_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_source" -Needle '"source_method": "module.load_ephemeral"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_capability" -Needle '"requested_capability": "cap.module.load_ephemeral"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_risk" -Needle '"risk": "modify_ram"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_resource" -Needle '"resource": "live_service_graph"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_reason" -Needle '"reason": "missing_evidence"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_evidence_gate" -Needle '"module_load_gate_evaluated"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_evidence_audit_required" -Needle '"durable_audit_record_required"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_evidence_rollback_required" -Needle '"rollback_plan_required"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_evidence_rollback_bindings" -Needle '"rollback_bindings_required"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_evidence_inventory" -Needle '"service_inventory_unchanged"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_evidence_no_load" -Needle '"load_not_attempted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_load_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_binding_status" -Needle '"status": "denied_missing_evidence"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_requirements_schema" -Needle '"audit_rollback_requirements": {"schema": "raios.module_load_gate_audit_rollback_requirements.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_requirements_no_load" -Needle '"audit_rollback_requirements": {"schema": "raios.module_load_gate_audit_rollback_requirements.v0", "classification": "public", "status": "required_missing", "writes_enabled": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_grant_state" -Needle '"computed_capability_grant": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_grant_binding" -Needle '"retained_computed_grant_reference": {"state": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_grant_reason" -Needle '"reason": "retained_computed_grant_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_grant_hash" -Needle "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_audit_rollback_binding" -Needle '"retained_audit_rollback_reference": {"state": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_audit_state" -Needle '"durable_audit_record": "retained_hash_reference_only_not_durable"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_rollback_state" -Needle '"rollback_plan": "retained_hash_reference_only_not_installed"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_state" -Needle '"service_slot": "retained_hash_reference_only_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_binding" -Needle '"retained_service_slot_reservation": {"state": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_status" -Needle '"status": "retained_hash_reference_only_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_reason" -Needle '"reason": "retained_service_slot_reservation_not_allocated"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_audit_hash" -Needle "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_rollback_hash" -Needle "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_hash" -Needle "`"reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_evidence_hash" -Needle "`"service_slot_reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_event_id" -Needle '"retained_service_slot_reservation_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_no_allocation" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_service_slot_no_inventory" -Needle '"creates_service_inventory_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_binding_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1

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
