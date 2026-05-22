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

function Get-SerialLogOffset {
    if (Test-Path -LiteralPath $SerialLog) {
        $content = Get-Content -Raw -LiteralPath $SerialLog -ErrorAction SilentlyContinue
        if ($null -ne $content) {
            return [int64]$content.Length
        }
    }
    return [int64]0
}

function Wait-ForLogTextAfterOffset {
    param(
        [string]$Path,
        [string]$Needle,
        [int64]$Offset,
        [int]$TimeoutSeconds
    )

    $deadline = [DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
    do {
        if (Test-Path -LiteralPath $Path) {
            $content = Get-Content -Raw -LiteralPath $Path -ErrorAction SilentlyContinue
            if ($null -ne $content) {
                $start = [int64]$Offset
                if ($start -lt 0) {
                    $start = [int64]0
                }
                if ($start -gt [int64]$content.Length) {
                    $start = [int64]$content.Length
                }
                $after = if ($start -eq [int64]0) {
                    $content
                }
                elseif ($start -lt [int64]$content.Length) {
                    $content.Substring([int]$start)
                }
                else {
                    ""
                }
                if ($after -clike "*$Needle*") {
                    return $true
                }
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

function Assert-LogContainsFields {
    param(
        [string]$NamePrefix,
        [object[]]$Fields,
        [int]$TimeoutSeconds
    )

    foreach ($field in $Fields) {
        Assert-LogContains -Name "$NamePrefix$($field.Suffix)" -Needle $field.Needle -TimeoutSeconds $TimeoutSeconds
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

    $startOffset = Get-SerialLogOffset
    Send-SerialText -Port $SerialTcpPort -Text "$Command`r" -TimeoutSeconds $TimeoutSeconds
    $predicateName = if ($Name.Length -gt 0) { $Name } else { "command:$Command" }
    $passed = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle $ExpectedMarker -Offset $startOffset -TimeoutSeconds $TimeoutSeconds
    $actual = if ($passed) { "found_after_offset:$startOffset" } else { Get-SerialLogTail -Path $SerialLog }
    Add-Predicate -Name $predicateName -Expected "serial_contains_after_offset:$ExpectedMarker" -Passed $passed -Actual $actual
    if (-not $passed) {
        throw "Timed out waiting for '$ExpectedMarker' in $SerialLog after offset $startOffset"
    }
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
            "agent module.manifest_diagnostic",
            "agent module.manifest_diagnostic <valid hash reference>",
            "agent module.manifest_diagnostic_selftest",
            "agent module.artifact_diagnostic",
            "agent module.artifact_diagnostic <valid hash reference>",
            "agent module.artifact_diagnostic_selftest",
            "agent module.vm_report_diagnostic",
            "agent module.vm_report_diagnostic <valid hash reference>",
            "agent module.vm_report_diagnostic_selftest",
            "agent module.attestation_diagnostic",
            "agent module.attestation_diagnostic <valid hash reference>",
            "agent module.attestation_diagnostic_selftest",
            "agent module.approval_diagnostic",
            "agent module.approval_diagnostic <valid hash reference>",
            "agent module.approval_diagnostic_selftest",
            "agent module.grant_diagnostic",
            "agent module.grant_diagnostic <valid hash reference>",
            "agent module.grant_diagnostic_selftest",
            "agent module.audit_rollback_diagnostic",
            "agent module.audit_rollback_diagnostic <valid hash reference>",
            "agent module.audit_rollback_diagnostic_selftest",
            "agent module.service_slot_diagnostic",
            "agent module.service_slot_diagnostic <valid hash reference>",
            "agent module.service_slot_diagnostic_selftest",
            "agent module.audit_rollback_availability",
            "agent module.audit_rollback_availability_selftest",
            "agent module.audit_rollback_write_policy",
            "agent module.audit_rollback_write_policy_selftest",
            "agent module.audit_rollback_storage_layout",
            "agent module.audit_rollback_storage_layout_selftest",
            "agent module.audit_rollback_append_engine",
            "agent module.audit_rollback_append_engine_selftest",
            "agent module.audit_rollback_append_contract",
            "agent module.audit_rollback_append_contract_selftest",
            "agent module.audit_rollback_append_payload_hash",
            "agent module.audit_rollback_append_payload_hash_selftest",
            "agent module.audit_rollback_append_intent",
            "agent module.audit_rollback_append_intent_selftest",
            "agent module.audit_rollback_write_boundary",
            "agent module.audit_rollback_write_boundary_selftest",
            "agent module.load_gate_manifest_selftest",
            "agent module.load_gate_artifact_selftest",
            "agent module.load_gate_vm_report_selftest",
            "agent module.load_gate_attestation_selftest",
            "agent module.load_gate_approval_selftest",
            "agent module.load_gate_retained_selftest",
            "agent module.load_gate_audit_rollback_selftest",
            "agent module.load_gate_service_slot_selftest",
            "module.load_ephemeral",
            "recovery.load_artifact",
            "agent recovery.identity_diagnostic",
            "agent recovery.identity_diagnostic <valid hash reference>",
            "agent recovery.identity_diagnostic_selftest",
            "agent recovery.trust_diagnostic",
            "agent recovery.trust_diagnostic <valid hash reference>",
            "agent recovery.trust_diagnostic_selftest",
            "agent recovery.vm_test_diagnostic",
            "agent recovery.vm_test_diagnostic <valid hash reference>",
            "agent recovery.vm_test_diagnostic_selftest",
            "agent recovery.local_approval_diagnostic",
            "agent recovery.local_approval_diagnostic <valid hash reference>",
            "agent recovery.local_approval_diagnostic_selftest",
            "agent recovery.loader_diagnostic",
            "agent recovery.loader_diagnostic <valid hash reference>",
            "agent recovery.loader_diagnostic_selftest",
            "agent recovery.rollback_evidence_diagnostic",
            "agent recovery.rollback_evidence_diagnostic <valid hash reference>",
            "agent recovery.rollback_evidence_diagnostic_selftest",
            "agent recovery.lifeline_request_diagnostic",
            "agent recovery.lifeline_request_diagnostic <valid hash reference>",
            "agent recovery.lifeline_request_diagnostic_selftest",
            "agent recovery.lifeline_protocol_diagnostic",
            "agent recovery.lifeline_protocol_diagnostic_selftest",
            "agent recovery.load_binding",
            "agent recovery.load_binding_selftest",
            "module.load_recovery_artifact",
            "agent audit.events 128"
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
    Assert-LogContains -Name "protocol:recovery_load_read_capability" -Needle '"id": "cap.recovery.load_artifact.read"' -TimeoutSeconds 1
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

    Send-AgentCommand -Command "agent module.manifest_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.manifest_diagnostic"
    Assert-LogContains -Name "protocol:module_manifest_diag_schema" -Needle '"schema": "raios.module_manifest_reference_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_no_manifest_json" -Needle '"accepts_manifest_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_absent_reason" -Needle '"validation_reason": "module_manifest_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_manifest_missing" -Needle '"module_manifest": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_can_load_false" -Needle '"can_load_now": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    $moduleGrantManifestHash = "1111111111111111111111111111111111111111111111111111111111111111"
    $moduleManifestReferenceCanonical = @(
        "canonicalization=raios.module_manifest_reference.canonical.v0",
        "schema=raios.module_manifest_reference.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "manifest_schema=raios.module_manifest.v0",
        "manifest_sha256=$moduleGrantManifestHash",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleManifestReferenceHash = Get-TextSha256 -Text $moduleManifestReferenceCanonical
    $moduleManifestCommand = "agent module.manifest_diagnostic $moduleManifestReferenceHash $moduleGrantManifestHash"

    Send-AgentCommand -Command $moduleManifestCommand -ExpectedMarker "RAIOS_AGENT_END module.manifest_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:module_manifest_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"validation_reason": "module_manifest_reference_valid_but_loader_and_evidence_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "manifest_present"; Needle = '"manifest_reference_present": true' },
        @{ Suffix = "ref_hash_echo"; Needle = "`"manifest_reference_hash`": `"sha256:$moduleManifestReferenceHash`"" },
        @{ Suffix = "manifest_hash_echo"; Needle = "`"manifest_hash`": `"sha256:$moduleGrantManifestHash`"" },
        @{ Suffix = "still_no_load"; Needle = '"can_load_now": false' }
    )

    $moduleManifestResponse = Get-LastAgentResponseJson -Method "module.manifest_diagnostic"
    $moduleManifestRetainedReferenceEventId = [string]$moduleManifestResponse.body.result.retained_manifest_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:module_manifest_retained_reference_event_id_captured" -Value $moduleManifestRetainedReferenceEventId

    Send-AgentCommand -Command "agent module.manifest_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.manifest_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_manifest_selftest_schema" -Needle '"schema": "raios.module_manifest_reference_diagnostic_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_no_records" -Needle '"creates_retained_manifest_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_count" -Needle '"case_count": 5' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_valid_case" -Needle '"case": "accepted_current_boot_manifest_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_stale_case" -Needle '"case": "stale_previous_boot_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_selftest_mismatch_case" -Needle '"case": "mismatched_manifest_hash_reference"' -TimeoutSeconds 1

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
    Assert-LogContainsFields -NamePrefix "protocol:module_grant_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_retained"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "candidate_present"; Needle = '"computed_candidate_present": true' },
        @{ Suffix = "valid_still_no_capability"; Needle = '"grants_capability": false' },
        @{ Suffix = "valid_still_no_load"; Needle = '"can_load_now": false' },
        @{ Suffix = "valid_hash_echo"; Needle = "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" }
    )

    $moduleGrantResponse = Get-LastAgentResponseJson -Method "module.grant_diagnostic"
    $moduleAuditRetainedReferenceEventId = [string]$moduleGrantResponse.body.result.retained_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:module_grant_retained_reference_event_id_captured" -Value $moduleAuditRetainedReferenceEventId

    Send-AgentCommand -Command "agent module.artifact_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.artifact_diagnostic"
    Assert-LogContains -Name "protocol:module_artifact_diag_schema" -Needle '"schema": "raios.module_candidate_artifact_reference_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_no_manifest_json" -Needle '"accepts_manifest_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_absent_reason" -Needle '"validation_reason": "candidate_artifact_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_artifact_missing" -Needle '"candidate_artifact": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    $moduleArtifactReferenceCanonical = @(
        "canonicalization=raios.module_candidate_artifact_reference.canonical.v0",
        "schema=raios.module_candidate_artifact_reference.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "retained_manifest_reference_event_id=$moduleManifestRetainedReferenceEventId",
        "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
        "manifest_reference_sha256=$moduleManifestReferenceHash",
        "manifest_sha256=$moduleGrantManifestHash",
        "computed_capability_grant_sha256=$moduleGrantHash",
        "candidate_artifact_sha256=$moduleGrantArtifactHash",
        "vm_test_report_sha256=$moduleGrantReportHash",
        "local_attestation_sha256=$moduleGrantAttestationHash",
        "accepts_artifact_bytes=false",
        "loads_artifact=false",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleArtifactReferenceHash = Get-TextSha256 -Text $moduleArtifactReferenceCanonical
    $moduleArtifactCommand = "agent module.artifact_diagnostic $moduleArtifactReferenceHash $moduleManifestRetainedReferenceEventId $moduleAuditRetainedReferenceEventId $moduleManifestReferenceHash $moduleGrantManifestHash $moduleGrantHash $moduleGrantArtifactHash $moduleGrantReportHash $moduleGrantAttestationHash"

    Send-AgentCommand -Command $moduleArtifactCommand -ExpectedMarker "RAIOS_AGENT_END module.artifact_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:module_artifact_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"validation_reason": "candidate_artifact_reference_valid_but_loader_and_evidence_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "present"; Needle = '"candidate_artifact_reference_present": true' },
        @{ Suffix = "ref_hash_echo"; Needle = "`"artifact_reference_hash`": `"sha256:$moduleArtifactReferenceHash`"" },
        @{ Suffix = "artifact_hash_echo"; Needle = "`"artifact_hash`": `"sha256:$moduleGrantArtifactHash`"" },
        @{ Suffix = "still_no_load"; Needle = '"can_load_now": false' }
    )

    $moduleArtifactResponse = Get-LastAgentResponseJson -Method "module.artifact_diagnostic"
    $moduleArtifactRetainedReferenceEventId = [string]$moduleArtifactResponse.body.result.retained_candidate_artifact_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:module_artifact_retained_reference_event_id_captured" -Value $moduleArtifactRetainedReferenceEventId

    Send-AgentCommand -Command "agent module.artifact_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.artifact_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_artifact_selftest_schema" -Needle '"schema": "raios.module_candidate_artifact_reference_diagnostic_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_no_records" -Needle '"creates_retained_candidate_artifact_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_count" -Needle '"case_count": 7' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_valid_case" -Needle '"case": "accepted_current_boot_artifact_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_selftest_mismatch_case" -Needle '"case": "mismatched_artifact_reference_hash"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.vm_report_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.vm_report_diagnostic"
    Assert-LogContains -Name "protocol:module_vm_report_diag_schema" -Needle '"schema": "raios.module_vm_test_report_reference_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_no_manifest_json" -Needle '"accepts_manifest_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_no_vm_report_json" -Needle '"accepts_vm_report_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_absent_reason" -Needle '"validation_reason": "vm_test_report_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_report_missing" -Needle '"vm_test_report": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    $moduleVmReportReferenceCanonical = @(
        "canonicalization=raios.module_vm_test_report_reference.canonical.v0",
        "schema=raios.module_vm_test_report_reference.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "retained_manifest_reference_event_id=$moduleManifestRetainedReferenceEventId",
        "retained_artifact_reference_event_id=$moduleArtifactRetainedReferenceEventId",
        "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
        "manifest_reference_sha256=$moduleManifestReferenceHash",
        "artifact_reference_sha256=$moduleArtifactReferenceHash",
        "manifest_sha256=$moduleGrantManifestHash",
        "candidate_artifact_sha256=$moduleGrantArtifactHash",
        "computed_capability_grant_sha256=$moduleGrantHash",
        "vm_test_report_sha256=$moduleGrantReportHash",
        "local_attestation_sha256=$moduleGrantAttestationHash",
        "accepts_vm_report_json=false",
        "accepts_artifact_bytes=false",
        "loads_artifact=false",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleVmReportReferenceHash = Get-TextSha256 -Text $moduleVmReportReferenceCanonical
    $moduleVmReportCommand = "agent module.vm_report_diagnostic $moduleVmReportReferenceHash $moduleManifestRetainedReferenceEventId $moduleArtifactRetainedReferenceEventId $moduleAuditRetainedReferenceEventId $moduleManifestReferenceHash $moduleArtifactReferenceHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantHash $moduleGrantReportHash $moduleGrantAttestationHash"

    Send-AgentCommand -Command $moduleVmReportCommand -ExpectedMarker "RAIOS_AGENT_END module.vm_report_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:module_vm_report_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"validation_reason": "vm_test_report_reference_valid_but_loader_and_evidence_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "present"; Needle = '"vm_test_report_reference_present": true' },
        @{ Suffix = "ref_hash_echo"; Needle = "`"vm_test_report_reference_hash`": `"sha256:$moduleVmReportReferenceHash`"" },
        @{ Suffix = "report_hash_echo"; Needle = "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" },
        @{ Suffix = "artifact_ref_hash_echo"; Needle = "`"artifact_reference_hash`": `"sha256:$moduleArtifactReferenceHash`"" },
        @{ Suffix = "still_no_load"; Needle = '"can_load_now": false' }
    )

    $moduleVmReportResponse = Get-LastAgentResponseJson -Method "module.vm_report_diagnostic"
    $moduleVmReportRetainedReferenceEventId = [string]$moduleVmReportResponse.body.result.retained_vm_test_report_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:module_vm_report_retained_reference_event_id_captured" -Value $moduleVmReportRetainedReferenceEventId

    Send-AgentCommand -Command "agent module.vm_report_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.vm_report_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_vm_report_selftest_schema" -Needle '"schema": "raios.module_vm_test_report_reference_diagnostic_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_no_records" -Needle '"creates_retained_vm_test_report_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_no_vm_report_json" -Needle '"accepts_vm_report_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_count" -Needle '"case_count": 8' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_valid_case" -Needle '"case": "accepted_current_boot_report_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_mismatch_case" -Needle '"case": "vm_report_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_selftest_grant_mismatch_case" -Needle '"case": "computed_grant_hash_mismatch"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.attestation_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.attestation_diagnostic"
    Assert-LogContains -Name "protocol:module_attestation_diag_schema" -Needle '"schema": "raios.module_local_attestation_reference_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_no_attestation_json" -Needle '"accepts_local_attestation_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_absent_reason" -Needle '"validation_reason": "local_attestation_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_attestation_missing" -Needle '"local_attestation": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    $moduleAttestationReferenceCanonical = @(
        "canonicalization=raios.module_local_attestation_reference.canonical.v0",
        "schema=raios.module_local_attestation_reference.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "retained_manifest_reference_event_id=$moduleManifestRetainedReferenceEventId",
        "retained_artifact_reference_event_id=$moduleArtifactRetainedReferenceEventId",
        "retained_vm_report_reference_event_id=$moduleVmReportRetainedReferenceEventId",
        "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
        "manifest_reference_sha256=$moduleManifestReferenceHash",
        "artifact_reference_sha256=$moduleArtifactReferenceHash",
        "vm_test_report_reference_sha256=$moduleVmReportReferenceHash",
        "manifest_sha256=$moduleGrantManifestHash",
        "candidate_artifact_sha256=$moduleGrantArtifactHash",
        "computed_capability_grant_sha256=$moduleGrantHash",
        "vm_test_report_sha256=$moduleGrantReportHash",
        "local_attestation_sha256=$moduleGrantAttestationHash",
        "accepts_local_attestation_json=false",
        "accepts_artifact_bytes=false",
        "loads_artifact=false",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleAttestationReferenceHash = Get-TextSha256 -Text $moduleAttestationReferenceCanonical
    $moduleAttestationCommand = "agent module.attestation_diagnostic $moduleAttestationReferenceHash $moduleManifestRetainedReferenceEventId $moduleArtifactRetainedReferenceEventId $moduleVmReportRetainedReferenceEventId $moduleAuditRetainedReferenceEventId $moduleManifestReferenceHash $moduleArtifactReferenceHash $moduleVmReportReferenceHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantHash $moduleGrantReportHash $moduleGrantAttestationHash"

    Send-AgentCommand -Command $moduleAttestationCommand -ExpectedMarker "RAIOS_AGENT_END module.attestation_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:module_attestation_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"validation_reason": "local_attestation_reference_valid_but_loader_and_evidence_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "present"; Needle = '"local_attestation_reference_present": true' },
        @{ Suffix = "ref_hash_echo"; Needle = "`"local_attestation_reference_hash`": `"sha256:$moduleAttestationReferenceHash`"" },
        @{ Suffix = "attestation_hash_echo"; Needle = "`"local_attestation_hash`": `"sha256:$moduleGrantAttestationHash`"" },
        @{ Suffix = "vm_ref_hash_echo"; Needle = "`"vm_test_report_reference_hash`": `"sha256:$moduleVmReportReferenceHash`"" },
        @{ Suffix = "still_no_load"; Needle = '"can_load_now": false' }
    )

    $moduleAttestationResponse = Get-LastAgentResponseJson -Method "module.attestation_diagnostic"
    $moduleAttestationRetainedReferenceEventId = [string]$moduleAttestationResponse.body.result.retained_local_attestation_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:module_attestation_retained_reference_event_id_captured" -Value $moduleAttestationRetainedReferenceEventId

    Send-AgentCommand -Command "agent module.attestation_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.attestation_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_attestation_selftest_schema" -Needle '"schema": "raios.module_local_attestation_reference_diagnostic_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_no_records" -Needle '"creates_retained_local_attestation_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_no_attestation_json" -Needle '"accepts_local_attestation_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_count" -Needle '"case_count": 9' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_valid_case" -Needle '"case": "accepted_current_boot_attestation_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_selftest_mismatch_case" -Needle '"case": "local_attestation_reference_hash_mismatch"' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.approval_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.approval_diagnostic"
    Assert-LogContains -Name "protocol:module_approval_diag_schema" -Needle '"schema": "raios.module_local_approval_reference_diagnostic.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_no_approval_text" -Needle '"accepts_local_approval_text": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_absent" -Needle '"validation_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_absent_reason" -Needle '"validation_reason": "local_approval_reference_absent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_approval_missing" -Needle '"local_approval": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_diag_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    $moduleAuditLocalApprovalHash = "6666666666666666666666666666666666666666666666666666666666666666"
    $moduleApprovalReferenceCanonical = @(
        "canonicalization=raios.module_local_approval_reference.canonical.v0",
        "schema=raios.module_local_approval_reference.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "retained_manifest_reference_event_id=$moduleManifestRetainedReferenceEventId",
        "retained_artifact_reference_event_id=$moduleArtifactRetainedReferenceEventId",
        "retained_vm_report_reference_event_id=$moduleVmReportRetainedReferenceEventId",
        "retained_local_attestation_reference_event_id=$moduleAttestationRetainedReferenceEventId",
        "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
        "manifest_reference_sha256=$moduleManifestReferenceHash",
        "artifact_reference_sha256=$moduleArtifactReferenceHash",
        "vm_test_report_reference_sha256=$moduleVmReportReferenceHash",
        "local_attestation_reference_sha256=$moduleAttestationReferenceHash",
        "manifest_sha256=$moduleGrantManifestHash",
        "candidate_artifact_sha256=$moduleGrantArtifactHash",
        "computed_capability_grant_sha256=$moduleGrantHash",
        "vm_test_report_sha256=$moduleGrantReportHash",
        "local_attestation_sha256=$moduleGrantAttestationHash",
        "local_approval_sha256=$moduleAuditLocalApprovalHash",
        "accepts_local_approval_text=false",
        "accepts_artifact_bytes=false",
        "loads_artifact=false",
        "authorizes_guest_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $moduleApprovalReferenceHash = Get-TextSha256 -Text $moduleApprovalReferenceCanonical
    $moduleApprovalCommand = "agent module.approval_diagnostic $moduleApprovalReferenceHash $moduleManifestRetainedReferenceEventId $moduleArtifactRetainedReferenceEventId $moduleVmReportRetainedReferenceEventId $moduleAttestationRetainedReferenceEventId $moduleAuditRetainedReferenceEventId $moduleManifestReferenceHash $moduleArtifactReferenceHash $moduleVmReportReferenceHash $moduleAttestationReferenceHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantHash $moduleGrantReportHash $moduleGrantAttestationHash $moduleAuditLocalApprovalHash"

    Send-AgentCommand -Command $moduleApprovalCommand -ExpectedMarker "RAIOS_AGENT_END module.approval_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:module_approval_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"validation_reason": "local_approval_reference_valid_but_loader_and_evidence_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "present"; Needle = '"local_approval_reference_present": true' },
        @{ Suffix = "ref_hash_echo"; Needle = "`"local_approval_reference_hash`": `"sha256:$moduleApprovalReferenceHash`"" },
        @{ Suffix = "approval_hash_echo"; Needle = "`"local_approval_hash`": `"sha256:$moduleAuditLocalApprovalHash`"" },
        @{ Suffix = "attestation_ref_hash_echo"; Needle = "`"local_attestation_reference_hash`": `"sha256:$moduleAttestationReferenceHash`"" },
        @{ Suffix = "still_no_load"; Needle = '"can_load_now": false' }
    )

    $moduleApprovalResponse = Get-LastAgentResponseJson -Method "module.approval_diagnostic"
    $moduleApprovalRetainedReferenceEventId = [string]$moduleApprovalResponse.body.result.retained_local_approval_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:module_approval_retained_reference_event_id_captured" -Value $moduleApprovalRetainedReferenceEventId

    Send-AgentCommand -Command "agent module.approval_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END module.approval_diagnostic_selftest"
    Assert-LogContains -Name "protocol:module_approval_selftest_schema" -Needle '"schema": "raios.module_local_approval_reference_diagnostic_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_no_records" -Needle '"creates_retained_local_approval_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_no_approval_text" -Needle '"accepts_local_approval_text": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_count" -Needle '"case_count": 10' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_absent_case" -Needle '"case": "absent_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_valid_case" -Needle '"case": "accepted_current_boot_approval_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_selftest_mismatch_case" -Needle '"case": "local_approval_reference_hash_mismatch"' -TimeoutSeconds 1

    Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "command:module.load_ephemeral.pre_audit"
    $modulePreAuditLoadResponse = Get-LastAgentResponseJson -Method "module.load_ephemeral"
    $moduleAuditDenialEventId = [string]$modulePreAuditLoadResponse.body.event_id
    Assert-CurrentBootEventId -Name "protocol:module_audit_denial_event_id_captured" -Value $moduleAuditDenialEventId
    Assert-LogContains -Name "policy:module_pre_audit_load_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_artifact_retained" -Needle '"candidate_artifact": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_vm_report_retained" -Needle '"vm_test_report": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_attestation_retained" -Needle '"local_attestation": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_grant_retained" -Needle '"computed_capability_grant": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_pre_audit_approval_retained" -Needle '"local_approval": "retained_hash_reference_only"' -TimeoutSeconds 1
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
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"validation_reason": "audit_rollback_reference_valid_but_loader_and_slot_missing"' },
        @{ Suffix = "audit_hash_echo"; Needle = "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" },
        @{ Suffix = "rollback_hash_echo"; Needle = "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" },
        @{ Suffix = "grant_hash_echo"; Needle = "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" },
        @{ Suffix = "audit_ref_present"; Needle = '"audit_record_hash_reference_present": true' },
        @{ Suffix = "rollback_ref_present"; Needle = '"rollback_plan_hash_reference_present": true' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "retained_recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "no_durable_write"; Needle = '"durable_audit_written": false' },
        @{ Suffix = "not_installed"; Needle = '"rollback_plan_installed": false' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "inventory_none"; Needle = '"service_inventory_change": "none"' }
    )

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
    Assert-LogContainsFields -NamePrefix "protocol:module_service_slot_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "valid_status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "valid_reason"; Needle = '"validation_reason": "service_slot_reservation_valid_but_allocator_and_loader_missing"' },
        @{ Suffix = "reservation_hash_echo"; Needle = "`"reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" },
        @{ Suffix = "grant_hash_echo"; Needle = "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" },
        @{ Suffix = "audit_hash_echo"; Needle = "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" },
        @{ Suffix = "rollback_hash_echo"; Needle = "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" },
        @{ Suffix = "inventory_hash_echo"; Needle = "`"pre_load_service_inventory_hash`": `"sha256:$moduleAuditPreInventoryHash`"" },
        @{ Suffix = "slot_echo"; Needle = "`"ram_only_service_slot_id`": `"$moduleAuditRamOnlyServiceSlotId`"" },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "retained_recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "policy_present"; Needle = '"reservation_reference_present": true' },
        @{ Suffix = "policy_no_reserved_slot"; Needle = '"service_slot_reserved": false' },
        @{ Suffix = "policy_can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "policy_inventory_none"; Needle = '"service_inventory_change": "none"' }
    )

    Send-AgentCommand -Command "agent module.audit_rollback_availability" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_availability"
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_availability_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_availability.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "no_audit_records"; Needle = '"creates_durable_audit_records": false' },
        @{ Suffix = "no_rollback_plans"; Needle = '"creates_rollback_plans": false' },
        @{ Suffix = "no_rollback_install"; Needle = '"installs_rollback_plan": false' },
        @{ Suffix = "ledger_schema"; Needle = '"schema": "raios.durable_audit_ledger.v0"' },
        @{ Suffix = "ledger_id"; Needle = '"id": "availability.durable_audit_ledger.current_boot"' },
        @{ Suffix = "ledger_reason"; Needle = '"reason": "durable_audit_ledger_missing"' },
        @{ Suffix = "ledger_present_false"; Needle = '"present": false' },
        @{ Suffix = "ledger_provenance_false"; Needle = '"provenance_valid": false' },
        @{ Suffix = "rollback_schema"; Needle = '"schema": "raios.rollback_store.v0"' },
        @{ Suffix = "rollback_id"; Needle = '"id": "availability.rollback_store.current_boot"' },
        @{ Suffix = "rollback_reason"; Needle = '"reason": "rollback_store_missing"' },
        @{ Suffix = "availability_status"; Needle = '"availability_status": "missing"' },
        @{ Suffix = "availability_reason"; Needle = '"availability_reason": "durable_audit_ledger_missing_and_rollback_store_missing"' },
        @{ Suffix = "audit_write_missing"; Needle = '"durable_audit_write_missing": true' },
        @{ Suffix = "rollback_install_missing"; Needle = '"rollback_install_missing": true' },
        @{ Suffix = "policy_missing"; Needle = '"durable_write_policy_available": false' },
        @{ Suffix = "rollback_policy_missing"; Needle = '"rollback_install_policy_available": false' },
        @{ Suffix = "not_durable_authority"; Needle = '"retained_hash_refs_are_durable_authority": false' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent module.audit_rollback_write_policy" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_write_policy"
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_write_policy_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_write_policy.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "no_audit_records"; Needle = '"creates_durable_audit_records": false' },
        @{ Suffix = "no_rollback_plans"; Needle = '"creates_rollback_plans": false' },
        @{ Suffix = "no_rollback_install"; Needle = '"installs_rollback_plan": false' },
        @{ Suffix = "durable_policy_schema"; Needle = '"schema": "raios.durable_audit_write_policy.v0"' },
        @{ Suffix = "durable_policy_id"; Needle = '"id": "policy.durable_audit_write.current_boot"' },
        @{ Suffix = "durable_policy_reason"; Needle = '"reason": "durable_write_policy_missing"' },
        @{ Suffix = "durable_policy_present_false"; Needle = '"present": false' },
        @{ Suffix = "durable_policy_binding_false"; Needle = '"binds_retained_evidence": false' },
        @{ Suffix = "durable_policy_availability_false"; Needle = '"binds_availability": false' },
        @{ Suffix = "rollback_policy_schema"; Needle = '"schema": "raios.rollback_install_policy.v0"' },
        @{ Suffix = "rollback_policy_id"; Needle = '"id": "policy.rollback_install.current_boot"' },
        @{ Suffix = "rollback_policy_reason"; Needle = '"reason": "rollback_install_policy_missing"' },
        @{ Suffix = "policy_status"; Needle = '"policy_status": "missing"' },
        @{ Suffix = "policy_reason"; Needle = '"policy_reason": "durable_write_policy_missing_and_rollback_install_policy_missing"' },
        @{ Suffix = "policy_missing"; Needle = '"durable_write_policy_missing": true' },
        @{ Suffix = "rollback_policy_missing"; Needle = '"rollback_install_policy_missing": true' },
        @{ Suffix = "policy_authority_false"; Needle = '"retained_hash_refs_are_policy_authority": false' },
        @{ Suffix = "availability_authority_false"; Needle = '"availability_facts_are_policy_authority": false' },
        @{ Suffix = "required_manifest"; Needle = '"module_manifest": "raios.module_manifest_reference.v0"' },
        @{ Suffix = "required_availability"; Needle = '"durable_audit_ledger": "raios.durable_audit_ledger.v0"' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent module.audit_rollback_storage_layout" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_storage_layout"
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_storage_layout_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_storage_layout.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "no_audit_records"; Needle = '"creates_durable_audit_records": false' },
        @{ Suffix = "no_rollback_plans"; Needle = '"creates_rollback_plans": false' },
        @{ Suffix = "device_schema"; Needle = '"schema": "raios.persistence_device_inventory.v0"' },
        @{ Suffix = "device_id"; Needle = '"id": "storage.persistence_device_inventory.current_boot"' },
        @{ Suffix = "device_reason"; Needle = '"reason": "persistence_device_inventory_missing"' },
        @{ Suffix = "device_present_false"; Needle = '"present": false' },
        @{ Suffix = "device_stable_false"; Needle = '"stable_identity": false' },
        @{ Suffix = "partition_inventory_false"; Needle = '"partition_inventory_available": false' },
        @{ Suffix = "write_path_false"; Needle = '"write_path_available": false' },
        @{ Suffix = "layout_schema"; Needle = '"schema": "raios.audit_rollback_storage_layout.v0"' },
        @{ Suffix = "layout_id"; Needle = '"id": "storage.audit_rollback_layout.current_boot"' },
        @{ Suffix = "layout_reason"; Needle = '"reason": "audit_rollback_storage_layout_missing"' },
        @{ Suffix = "layout_binding_false"; Needle = '"binds_persistence_device": false' },
        @{ Suffix = "audit_region_false"; Needle = '"has_audit_ledger_region": false' },
        @{ Suffix = "rollback_region_false"; Needle = '"has_rollback_store_region": false' },
        @{ Suffix = "append_slots_false"; Needle = '"append_slots_available": false' },
        @{ Suffix = "storage_status"; Needle = '"storage_layout_status": "missing"' },
        @{ Suffix = "storage_reason"; Needle = '"storage_layout_reason": "persistence_device_inventory_missing_and_storage_layout_missing"' },
        @{ Suffix = "storage_available_false"; Needle = '"storage_layout_available": false' },
        @{ Suffix = "storage_missing_true"; Needle = '"storage_layout_missing": true' },
        @{ Suffix = "append_engine_missing"; Needle = '"append_engine_missing": true' },
        @{ Suffix = "not_storage_authority"; Needle = '"retained_hash_refs_are_storage_authority": false' },
        @{ Suffix = "storage_not_append_authority"; Needle = '"storage_layout_facts_are_append_authority": false' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent module.audit_rollback_append_engine" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_engine"
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_append_engine_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_append_engine.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "storage_inputs"; Needle = '"storage_layout_inputs": {' },
        @{ Suffix = "storage_available_false"; Needle = '"storage_layout_available": false' },
        @{ Suffix = "storage_missing_true"; Needle = '"storage_layout_missing": true' },
        @{ Suffix = "storage_not_engine_authority"; Needle = '"storage_layout_facts_are_append_engine_authority": false' },
        @{ Suffix = "audit_engine_schema"; Needle = '"schema": "raios.audit_ledger_append_engine.v0"' },
        @{ Suffix = "audit_engine_id"; Needle = '"id": "append_engine.audit_ledger.current_boot"' },
        @{ Suffix = "audit_engine_target"; Needle = '"target_schema": "raios.audit_record.v0"' },
        @{ Suffix = "audit_engine_reason"; Needle = '"reason": "audit_ledger_append_engine_missing"' },
        @{ Suffix = "audit_engine_present_false"; Needle = '"present": false' },
        @{ Suffix = "audit_engine_storage_binding_false"; Needle = '"binds_storage_layout": false' },
        @{ Suffix = "audit_engine_policy_binding_false"; Needle = '"binds_write_policy": false' },
        @{ Suffix = "audit_engine_append_only_false"; Needle = '"supports_append_only": false' },
        @{ Suffix = "rollback_engine_schema"; Needle = '"schema": "raios.rollback_store_transaction_engine.v0"' },
        @{ Suffix = "rollback_engine_id"; Needle = '"id": "append_engine.rollback_store.current_boot"' },
        @{ Suffix = "rollback_engine_target"; Needle = '"target_schema": "raios.rollback_plan.v0"' },
        @{ Suffix = "rollback_engine_reason"; Needle = '"reason": "rollback_store_transaction_engine_missing"' },
        @{ Suffix = "append_engine_status"; Needle = '"append_engine_status": "missing"' },
        @{ Suffix = "append_engine_reason"; Needle = '"append_engine_reason": "audit_ledger_append_engine_missing_and_rollback_store_transaction_engine_missing"' },
        @{ Suffix = "append_engine_available_false"; Needle = '"append_engine_available": false' },
        @{ Suffix = "append_engine_missing_true"; Needle = '"append_engine_missing": true' },
        @{ Suffix = "retained_not_engine_authority"; Needle = '"retained_hash_refs_are_append_engine_authority": false' },
        @{ Suffix = "engine_not_append_authority"; Needle = '"append_engine_facts_are_append_authority": false' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent module.audit_rollback_append_contract" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_contract"
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_append_contract_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_append_contract.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "storage_inputs"; Needle = '"storage_layout_inputs": {' },
        @{ Suffix = "storage_input_device"; Needle = '"persistence_device_inventory": {' },
        @{ Suffix = "storage_input_layout"; Needle = '"audit_rollback_storage_layout": {' },
        @{ Suffix = "storage_input_available_false"; Needle = '"storage_layout_available": false' },
        @{ Suffix = "storage_input_authority_false"; Needle = '"storage_layout_facts_are_append_authority": false' },
        @{ Suffix = "append_engine_inputs"; Needle = '"append_engine_inputs": {' },
        @{ Suffix = "append_engine_input_audit"; Needle = '"audit_ledger_append_engine": {' },
        @{ Suffix = "append_engine_input_rollback"; Needle = '"rollback_store_transaction_engine": {' },
        @{ Suffix = "append_engine_input_available_false"; Needle = '"append_engine_available": false' },
        @{ Suffix = "append_engine_input_authority_false"; Needle = '"append_engine_facts_are_append_authority": false' },
        @{ Suffix = "audit_append_schema"; Needle = '"schema": "raios.audit_ledger_append_envelope.v0"' },
        @{ Suffix = "audit_append_id"; Needle = '"id": "append.audit_ledger.current_boot"' },
        @{ Suffix = "audit_append_target"; Needle = '"target_schema": "raios.audit_record.v0"' },
        @{ Suffix = "audit_append_storage_schema"; Needle = '"storage_layout_schema": "raios.audit_rollback_storage_layout.v0"' },
        @{ Suffix = "audit_append_storage_id"; Needle = '"storage_layout_id": "storage.audit_rollback_layout.current_boot"' },
        @{ Suffix = "audit_append_engine_id"; Needle = '"append_engine_id": "append_engine.audit_ledger.current_boot"' },
        @{ Suffix = "audit_append_policy_id"; Needle = '"write_policy_id": "policy.durable_audit_write.current_boot"' },
        @{ Suffix = "audit_append_availability_id"; Needle = '"availability_id": "availability.durable_audit_ledger.current_boot"' },
        @{ Suffix = "audit_append_provenance_schema"; Needle = '"append_contract_provenance_schema": "raios.append_contract_envelope_provenance.v0"' },
        @{ Suffix = "audit_append_reason"; Needle = '"reason": "audit_append_envelope_missing"' },
        @{ Suffix = "audit_append_storage_binding_false"; Needle = '"binds_storage_layout_id": false' },
        @{ Suffix = "audit_append_engine_binding_false"; Needle = '"binds_append_engine_id": false' },
        @{ Suffix = "audit_append_policy_id_binding_false"; Needle = '"binds_write_policy_id": false' },
        @{ Suffix = "audit_append_availability_id_binding_false"; Needle = '"binds_availability_id": false' },
        @{ Suffix = "audit_append_provenance_binding_false"; Needle = '"binds_envelope_provenance": false' },
        @{ Suffix = "storage_layout_false"; Needle = '"storage_layout_available": false' },
        @{ Suffix = "append_engine_false"; Needle = '"append_engine_available": false' },
        @{ Suffix = "rollback_transaction_schema"; Needle = '"schema": "raios.rollback_store_transaction_envelope.v0"' },
        @{ Suffix = "rollback_transaction_id"; Needle = '"id": "append.rollback_store.current_boot"' },
        @{ Suffix = "rollback_transaction_target"; Needle = '"target_schema": "raios.rollback_plan.v0"' },
        @{ Suffix = "rollback_transaction_engine_id"; Needle = '"append_engine_id": "append_engine.rollback_store.current_boot"' },
        @{ Suffix = "rollback_transaction_policy_id"; Needle = '"write_policy_id": "policy.rollback_install.current_boot"' },
        @{ Suffix = "rollback_transaction_availability_id"; Needle = '"availability_id": "availability.rollback_store.current_boot"' },
        @{ Suffix = "rollback_transaction_reason"; Needle = '"reason": "rollback_transaction_envelope_missing"' },
        @{ Suffix = "contract_status"; Needle = '"append_contract_status": "missing"' },
        @{ Suffix = "contract_reason"; Needle = '"append_contract_reason": "audit_append_envelope_missing_and_rollback_transaction_envelope_missing"' },
        @{ Suffix = "storage_layout_missing"; Needle = '"storage_layout_missing": true' },
        @{ Suffix = "append_engine_missing"; Needle = '"append_engine_missing": true' },
        @{ Suffix = "not_append_authority"; Needle = '"retained_hash_refs_are_append_authority": false' },
        @{ Suffix = "policy_not_append_authority"; Needle = '"policy_facts_are_append_authority": false' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent module.audit_rollback_append_payload_hash" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_payload_hash"
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_append_payload_hash_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_append_payload_hash.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "retained_inputs"; Needle = '"retained_payload_inputs": {' },
        @{ Suffix = "audit_event"; Needle = '"audit_rollback": {"event_id": "event.current_boot.' },
        @{ Suffix = "service_slot_event"; Needle = '"service_slot_reservation": {"event_id": "event.current_boot.' },
        @{ Suffix = "audit_hash_input"; Needle = "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" },
        @{ Suffix = "rollback_hash_input"; Needle = "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" },
        @{ Suffix = "inventory_hash_input"; Needle = "`"pre_load_service_inventory_hash`": `"sha256:$moduleAuditPreInventoryHash`"" },
        @{ Suffix = "reservation_hash_input"; Needle = "`"service_slot_reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" },
        @{ Suffix = "slot_id_input"; Needle = "`"ram_only_service_slot_id`": `"$moduleAuditRamOnlyServiceSlotId`"" },
        @{ Suffix = "append_contract_inputs"; Needle = '"append_contract_inputs": {' },
        @{ Suffix = "contract_available_false"; Needle = '"append_contract_available": false' },
        @{ Suffix = "payload_facts"; Needle = '"append_payload_hash_facts": {' },
        @{ Suffix = "audit_payload_schema"; Needle = '"schema": "raios.audit_record_append_payload_hash_envelope.v0"' },
        @{ Suffix = "audit_payload_id"; Needle = '"id": "append_payload.audit_record.current_boot"' },
        @{ Suffix = "audit_payload_target"; Needle = '"target_schema": "raios.audit_record.v0"' },
        @{ Suffix = "audit_payload_contract"; Needle = '"append_contract_id": "append.audit_ledger.current_boot"' },
        @{ Suffix = "payload_hash_present"; Needle = '"payload_hash": "sha256:' },
        @{ Suffix = "audit_source_payload_hash"; Needle = "`"source_payload_hash`": `"sha256:$moduleAuditHash`"" },
        @{ Suffix = "audit_payload_reason"; Needle = '"reason": "audit_record_append_contract_missing"' },
        @{ Suffix = "binds_retained"; Needle = '"binds_retained_audit_rollback": true' },
        @{ Suffix = "binds_slot"; Needle = '"binds_service_slot_reservation": true' },
        @{ Suffix = "binds_request"; Needle = '"binds_pre_load_write_request": true' },
        @{ Suffix = "binds_contract_id"; Needle = '"binds_append_contract_id": true' },
        @{ Suffix = "binds_payload"; Needle = '"binds_payload_hash": true' },
        @{ Suffix = "retained_available"; Needle = '"retained_audit_rollback_available": true' },
        @{ Suffix = "service_slot_available"; Needle = '"service_slot_reservation_available": true' },
        @{ Suffix = "append_contract_available_false"; Needle = '"append_contract_available": false' },
        @{ Suffix = "rollback_payload_schema"; Needle = '"schema": "raios.rollback_transaction_append_payload_hash_envelope.v0"' },
        @{ Suffix = "rollback_payload_id"; Needle = '"id": "append_payload.rollback_transaction.current_boot"' },
        @{ Suffix = "rollback_payload_target"; Needle = '"target_schema": "raios.rollback_plan.v0"' },
        @{ Suffix = "rollback_source_payload_hash"; Needle = "`"source_payload_hash`": `"sha256:$moduleRollbackHash`"" },
        @{ Suffix = "rollback_payload_reason"; Needle = '"reason": "rollback_transaction_append_contract_missing"' },
        @{ Suffix = "payload_status"; Needle = '"payload_hash_status": "missing"' },
        @{ Suffix = "payload_reason"; Needle = '"payload_hash_reason": "audit_record_append_contract_missing"' },
        @{ Suffix = "retained_available_result"; Needle = '"retained_evidence_available": true' },
        @{ Suffix = "payload_available_false"; Needle = '"payload_hash_available": false' },
        @{ Suffix = "payload_missing_true"; Needle = '"payload_hash_missing": true' },
        @{ Suffix = "not_payload_authority"; Needle = '"payload_hash_envelopes_are_append_intent_authority": false' },
        @{ Suffix = "retained_not_payload_authority"; Needle = '"retained_hash_refs_are_payload_authority": false' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent module.audit_rollback_append_intent" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_intent"
    Assert-LogContainsFields -NamePrefix "protocol:module_audit_rollback_append_intent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_append_intent.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "append_contract_inputs"; Needle = '"append_contract_inputs": {' },
        @{ Suffix = "audit_contract_input"; Needle = '"audit_append_envelope": {"schema": "raios.audit_ledger_append_envelope.v0", "status": "missing", "reason": "audit_append_envelope_missing"' },
        @{ Suffix = "rollback_contract_input"; Needle = '"rollback_transaction_envelope": {"schema": "raios.rollback_store_transaction_envelope.v0", "status": "missing", "reason": "rollback_transaction_envelope_missing"' },
        @{ Suffix = "contract_available_false"; Needle = '"append_contract_available": false' },
        @{ Suffix = "contract_not_intent_authority"; Needle = '"append_contract_facts_are_append_intent_authority": false' },
        @{ Suffix = "payload_inputs"; Needle = '"append_payload_hash_inputs": {' },
        @{ Suffix = "audit_payload_input"; Needle = '"audit_record_append_payload_hash": {"schema": "raios.audit_record_append_payload_hash_envelope.v0", "status": "missing", "reason": "audit_record_append_contract_missing"' },
        @{ Suffix = "rollback_payload_input"; Needle = '"rollback_transaction_append_payload_hash": {"schema": "raios.rollback_transaction_append_payload_hash_envelope.v0", "status": "missing", "reason": "rollback_transaction_append_contract_missing"' },
        @{ Suffix = "payload_hash_available_false"; Needle = '"payload_hash_available": false' },
        @{ Suffix = "payload_not_intent_authority"; Needle = '"payload_hash_envelopes_are_append_intent_authority": false' },
        @{ Suffix = "append_intent_facts"; Needle = '"append_intent_facts": {' },
        @{ Suffix = "audit_intent_schema"; Needle = '"schema": "raios.audit_record_append_intent.v0"' },
        @{ Suffix = "audit_intent_id"; Needle = '"id": "append_intent.audit_record.current_boot"' },
        @{ Suffix = "audit_intent_target"; Needle = '"target_schema": "raios.audit_record.v0"' },
        @{ Suffix = "audit_intent_contract_id"; Needle = '"append_contract_id": "append.audit_ledger.current_boot"' },
        @{ Suffix = "audit_intent_engine_id"; Needle = '"append_engine_id": "append_engine.audit_ledger.current_boot"' },
        @{ Suffix = "audit_intent_storage_id"; Needle = '"storage_layout_id": "storage.audit_rollback_layout.current_boot"' },
        @{ Suffix = "audit_intent_policy_id"; Needle = '"write_policy_id": "policy.durable_audit_write.current_boot"' },
        @{ Suffix = "audit_intent_availability_id"; Needle = '"availability_id": "availability.durable_audit_ledger.current_boot"' },
        @{ Suffix = "intent_provenance_schema"; Needle = '"intent_provenance_schema": "raios.append_intent_provenance.v0"' },
        @{ Suffix = "payload_hash_schema"; Needle = '"payload_hash_schema": "raios.append_intent_payload_hash.v0"' },
        @{ Suffix = "payload_hash_envelope_schema"; Needle = '"payload_hash_envelope_schema": "raios.append_payload_hash_envelope.canonical.v0"' },
        @{ Suffix = "audit_intent_reason"; Needle = '"reason": "audit_record_append_intent_missing"' },
        @{ Suffix = "audit_contract_binding_false"; Needle = '"binds_append_contract_id": false' },
        @{ Suffix = "audit_engine_binding_false"; Needle = '"binds_append_engine_id": false' },
        @{ Suffix = "audit_storage_binding_false"; Needle = '"binds_storage_layout_id": false' },
        @{ Suffix = "audit_policy_binding_false"; Needle = '"binds_write_policy_id": false' },
        @{ Suffix = "audit_availability_binding_false"; Needle = '"binds_availability_id": false' },
        @{ Suffix = "audit_payload_binding_false"; Needle = '"binds_payload_hash": false' },
        @{ Suffix = "audit_provenance_binding_false"; Needle = '"binds_intent_provenance": false' },
        @{ Suffix = "audit_payload_available_false"; Needle = '"payload_hash_available": false' },
        @{ Suffix = "rollback_intent_schema"; Needle = '"schema": "raios.rollback_transaction_append_intent.v0"' },
        @{ Suffix = "rollback_intent_id"; Needle = '"id": "append_intent.rollback_transaction.current_boot"' },
        @{ Suffix = "rollback_intent_target"; Needle = '"target_schema": "raios.rollback_plan.v0"' },
        @{ Suffix = "rollback_intent_contract_id"; Needle = '"append_contract_id": "append.rollback_store.current_boot"' },
        @{ Suffix = "rollback_intent_engine_id"; Needle = '"append_engine_id": "append_engine.rollback_store.current_boot"' },
        @{ Suffix = "rollback_intent_policy_id"; Needle = '"write_policy_id": "policy.rollback_install.current_boot"' },
        @{ Suffix = "rollback_intent_availability_id"; Needle = '"availability_id": "availability.rollback_store.current_boot"' },
        @{ Suffix = "rollback_intent_reason"; Needle = '"reason": "rollback_transaction_append_intent_missing"' },
        @{ Suffix = "intent_status"; Needle = '"append_intent_status": "missing"' },
        @{ Suffix = "intent_reason"; Needle = '"append_intent_reason": "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing"' },
        @{ Suffix = "intent_available_false"; Needle = '"append_intent_available": false' },
        @{ Suffix = "intent_missing_true"; Needle = '"append_intent_missing": true' },
        @{ Suffix = "payload_missing_true"; Needle = '"payload_hash_missing": true' },
        @{ Suffix = "intent_not_writer_authority"; Needle = '"append_intent_facts_are_writer_authority": false' },
        @{ Suffix = "retained_not_intent_authority"; Needle = '"retained_hash_refs_are_append_intent_authority": false' },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent module.audit_rollback_write_boundary" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_write_boundary"
    Assert-LogContainsFields -NamePrefix "protocol:module_write_boundary_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.module_audit_rollback_write_boundary.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "writes_disabled"; Needle = '"writes_enabled": false' },
        @{ Suffix = "no_audit_records"; Needle = '"creates_durable_audit_records": false' },
        @{ Suffix = "no_rollback_plans"; Needle = '"creates_rollback_plans": false' },
        @{ Suffix = "no_rollback_install"; Needle = '"installs_rollback_plan": false' },
        @{ Suffix = "no_recovery_artifact"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "request_schema"; Needle = '"schema": "raios.module_pre_load_audit_rollback_write_request.v0"' },
        @{ Suffix = "availability_inputs"; Needle = '"availability_inputs": {' },
        @{ Suffix = "ledger_availability_input"; Needle = '"durable_audit_ledger": {"schema": "raios.durable_audit_ledger.v0", "status": "missing", "reason": "durable_audit_ledger_missing"' },
        @{ Suffix = "rollback_availability_input"; Needle = '"rollback_store": {"schema": "raios.rollback_store.v0", "status": "missing", "reason": "rollback_store_missing"' },
        @{ Suffix = "policy_inputs"; Needle = '"policy_inputs": {' },
        @{ Suffix = "durable_policy_input"; Needle = '"durable_write_policy": {"schema": "raios.durable_audit_write_policy.v0", "status": "missing", "reason": "durable_write_policy_missing"' },
        @{ Suffix = "rollback_policy_input"; Needle = '"rollback_install_policy": {"schema": "raios.rollback_install_policy.v0", "status": "missing", "reason": "rollback_install_policy_missing"' },
        @{ Suffix = "append_contract_inputs"; Needle = '"append_contract_inputs": {' },
        @{ Suffix = "audit_append_input"; Needle = '"audit_append_envelope": {"schema": "raios.audit_ledger_append_envelope.v0", "status": "missing", "reason": "audit_append_envelope_missing"' },
        @{ Suffix = "rollback_transaction_input"; Needle = '"rollback_transaction_envelope": {"schema": "raios.rollback_store_transaction_envelope.v0", "status": "missing", "reason": "rollback_transaction_envelope_missing"' },
        @{ Suffix = "append_payload_hash_inputs"; Needle = '"append_payload_hash_inputs": {' },
        @{ Suffix = "audit_payload_hash_input"; Needle = '"audit_record_append_payload_hash": {"schema": "raios.audit_record_append_payload_hash_envelope.v0", "status": "missing", "reason": "audit_record_append_contract_missing"' },
        @{ Suffix = "rollback_payload_hash_input"; Needle = '"rollback_transaction_append_payload_hash": {"schema": "raios.rollback_transaction_append_payload_hash_envelope.v0", "status": "missing", "reason": "rollback_transaction_append_contract_missing"' },
        @{ Suffix = "append_payload_available_false"; Needle = '"payload_hash_available": false' },
        @{ Suffix = "append_payload_not_writer_authority"; Needle = '"payload_hash_envelopes_are_writer_authority": false' },
        @{ Suffix = "append_intent_inputs"; Needle = '"append_intent_inputs": {' },
        @{ Suffix = "audit_append_intent_input"; Needle = '"audit_record_append_intent": {"schema": "raios.audit_record_append_intent.v0", "status": "missing", "reason": "audit_record_append_intent_missing"' },
        @{ Suffix = "rollback_append_intent_input"; Needle = '"rollback_transaction_append_intent": {"schema": "raios.rollback_transaction_append_intent.v0", "status": "missing", "reason": "rollback_transaction_append_intent_missing"' },
        @{ Suffix = "append_intent_available_false"; Needle = '"append_intent_available": false' },
        @{ Suffix = "append_intent_not_writer_authority"; Needle = '"append_intent_facts_are_writer_authority": false' },
        @{ Suffix = "denial_schema"; Needle = '"schema": "raios.module_audit_rollback_write_denial_evidence.v0"' },
        @{ Suffix = "validation_status"; Needle = '"validation_status": "denied_missing_durable_write_boundary"' },
        @{ Suffix = "validation_reason"; Needle = '"validation_reason": "durable_audit_write_missing_and_rollback_install_missing"' },
        @{ Suffix = "audit_write_missing"; Needle = '"reason": "durable_audit_write_missing"' },
        @{ Suffix = "rollback_install_missing"; Needle = '"reason": "rollback_install_missing"' },
        @{ Suffix = "durable_policy_missing"; Needle = '"durable_write_policy_missing": true' },
        @{ Suffix = "rollback_policy_missing"; Needle = '"rollback_install_policy_missing": true' },
        @{ Suffix = "audit_append_missing"; Needle = '"audit_append_status": "missing"' },
        @{ Suffix = "rollback_transaction_missing"; Needle = '"rollback_transaction_status": "missing"' },
        @{ Suffix = "audit_payload_hash_missing"; Needle = '"audit_append_payload_hash_status": "missing"' },
        @{ Suffix = "rollback_payload_hash_missing"; Needle = '"rollback_transaction_append_payload_hash_status": "missing"' },
        @{ Suffix = "audit_append_intent_missing"; Needle = '"audit_append_intent_status": "missing"' },
        @{ Suffix = "rollback_transaction_intent_missing"; Needle = '"rollback_transaction_append_intent_status": "missing"' },
        @{ Suffix = "append_intent_missing"; Needle = '"append_intent_missing": true' },
        @{ Suffix = "payload_missing"; Needle = '"payload_hash_missing": true' },
        @{ Suffix = "storage_layout_missing"; Needle = '"storage_layout_missing": true' },
        @{ Suffix = "append_engine_missing"; Needle = '"append_engine_missing": true' },
        @{ Suffix = "not_append_authority"; Needle = '"retained_hash_refs_are_append_authority": false' },
        @{ Suffix = "not_payload_authority"; Needle = '"retained_hash_refs_are_payload_authority": false' },
        @{ Suffix = "not_append_intent_authority"; Needle = '"retained_hash_refs_are_append_intent_authority": false' },
        @{ Suffix = "not_durable_authority"; Needle = '"retained_hash_refs_are_durable_authority": false' },
        @{ Suffix = "manifest_event"; Needle = '"module_manifest": {"event_id": "event.current_boot.' },
        @{ Suffix = "artifact_event"; Needle = '"candidate_artifact": {"event_id": "event.current_boot.' },
        @{ Suffix = "vm_report_event"; Needle = '"vm_test_report": {"event_id": "event.current_boot.' },
        @{ Suffix = "grant_event"; Needle = '"computed_capability_grant": {"event_id": "event.current_boot.' },
        @{ Suffix = "attestation_event"; Needle = '"local_attestation": {"event_id": "event.current_boot.' },
        @{ Suffix = "approval_event"; Needle = '"local_approval": {"event_id": "event.current_boot.' },
        @{ Suffix = "audit_event"; Needle = '"audit_rollback": {"event_id": "event.current_boot.' },
        @{ Suffix = "service_slot_event"; Needle = '"service_slot_reservation": {"event_id": "event.current_boot.' },
        @{ Suffix = "manifest_hash"; Needle = "`"manifest_hash`": `"sha256:$moduleGrantManifestHash`"" },
        @{ Suffix = "artifact_hash"; Needle = "`"candidate_artifact_hash`": `"sha256:$moduleGrantArtifactHash`"" },
        @{ Suffix = "vm_report_hash"; Needle = "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" },
        @{ Suffix = "attestation_hash"; Needle = "`"local_attestation_hash`": `"sha256:$moduleGrantAttestationHash`"" },
        @{ Suffix = "approval_hash"; Needle = "`"local_approval_hash`": `"sha256:$moduleAuditLocalApprovalHash`"" },
        @{ Suffix = "audit_hash"; Needle = "`"audit_record_hash`": `"sha256:$moduleAuditHash`"" },
        @{ Suffix = "rollback_hash"; Needle = "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" },
        @{ Suffix = "service_slot_hash"; Needle = "`"service_slot_reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" },
        @{ Suffix = "slot_id"; Needle = "`"ram_only_service_slot_id`": `"$moduleAuditRamOnlyServiceSlotId`"" },
        @{ Suffix = "can_load_false"; Needle = '"can_load_now": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

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

    Send-AgentCommand -Command "agent module.audit_rollback_availability_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_availability_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_availability_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_count" -Needle '"case_count": 8' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_missing_case" -Needle '"case": "missing_ledger_and_store_current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_ledger_scope_case" -Needle '"case": "durable_audit_ledger_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_ledger_schema_case" -Needle '"case": "durable_audit_ledger_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_ledger_provenance_case" -Needle '"case": "durable_audit_ledger_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_rollback_scope_case" -Needle '"case": "rollback_store_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_rollback_schema_case" -Needle '"case": "rollback_store_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_rollback_provenance_case" -Needle '"case": "rollback_store_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_policy_case" -Needle '"case": "available_facts_policy_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_policy_status" -Needle '"actual_status": "denied_missing_durable_write_policy"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_policy_reason" -Needle '"actual_reason": "durable_write_policy_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_availability_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_write_policy_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_write_policy_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_write_policy_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_count" -Needle '"case_count": 12' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_missing_case" -Needle '"case": "missing_policy_pair_current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_durable_scope_case" -Needle '"case": "durable_write_policy_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_durable_schema_case" -Needle '"case": "durable_write_policy_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_durable_provenance_case" -Needle '"case": "durable_write_policy_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_durable_binding_case" -Needle '"case": "durable_write_policy_retained_evidence_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_durable_availability_case" -Needle '"case": "durable_write_policy_availability_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_rollback_scope_case" -Needle '"case": "rollback_install_policy_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_rollback_schema_case" -Needle '"case": "rollback_install_policy_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_rollback_provenance_case" -Needle '"case": "rollback_install_policy_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_rollback_binding_case" -Needle '"case": "rollback_install_policy_retained_evidence_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_rollback_availability_case" -Needle '"case": "rollback_install_policy_availability_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_writer_case" -Needle '"case": "available_policy_facts_writer_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_writer_status" -Needle '"actual_status": "denied_write_path_unimplemented"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_writer_reason" -Needle '"actual_reason": "durable_audit_rollback_writer_unimplemented"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_write_policy_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_storage_layout_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_storage_layout_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_storage_layout_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_count" -Needle '"case_count": 15' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_missing_case" -Needle '"case": "missing_storage_inputs_current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_missing_reason" -Needle '"actual_reason": "persistence_device_inventory_missing_and_storage_layout_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_device_scope_case" -Needle '"case": "persistence_device_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_device_schema_case" -Needle '"case": "persistence_device_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_device_provenance_case" -Needle '"case": "persistence_device_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_device_identity_case" -Needle '"case": "persistence_device_stable_identity_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_partition_case" -Needle '"case": "persistence_partition_inventory_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_layout_scope_case" -Needle '"case": "audit_rollback_storage_layout_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_layout_schema_case" -Needle '"case": "audit_rollback_storage_layout_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_layout_provenance_case" -Needle '"case": "audit_rollback_storage_layout_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_binding_case" -Needle '"case": "storage_layout_device_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_audit_region_case" -Needle '"case": "audit_ledger_layout_region_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_rollback_region_case" -Needle '"case": "rollback_store_layout_region_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_append_slots_case" -Needle '"case": "storage_layout_append_slots_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_recovery_case" -Needle '"case": "storage_layout_recovery_boundary_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_available_case" -Needle '"case": "available_storage_layout_still_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_available_status" -Needle '"actual_status": "available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_available_reason" -Needle '"actual_reason": "audit_rollback_storage_layout_available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_storage_layout_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_append_engine_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_engine_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_append_engine_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_count" -Needle '"case_count": 16' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_missing_case" -Needle '"case": "missing_append_engine_pair_current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_missing_reason" -Needle '"actual_reason": "audit_ledger_append_engine_missing_and_rollback_store_transaction_engine_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_scope_case" -Needle '"case": "audit_ledger_append_engine_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_schema_case" -Needle '"case": "audit_ledger_append_engine_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_provenance_case" -Needle '"case": "audit_ledger_append_engine_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_storage_case" -Needle '"case": "audit_ledger_append_engine_storage_layout_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_policy_case" -Needle '"case": "audit_ledger_append_engine_write_policy_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_append_only_case" -Needle '"case": "audit_ledger_append_engine_append_only_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_flush_case" -Needle '"case": "audit_ledger_append_engine_flush_support_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_audit_recovery_case" -Needle '"case": "audit_ledger_append_engine_recovery_boundary_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_rollback_scope_case" -Needle '"case": "rollback_store_transaction_engine_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_rollback_schema_case" -Needle '"case": "rollback_store_transaction_engine_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_rollback_provenance_case" -Needle '"case": "rollback_store_transaction_engine_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_rollback_storage_case" -Needle '"case": "rollback_store_transaction_engine_storage_layout_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_rollback_policy_case" -Needle '"case": "rollback_store_transaction_engine_write_policy_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_rollback_replay_case" -Needle '"case": "rollback_store_transaction_engine_replay_support_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_available_case" -Needle '"case": "available_append_engines_still_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_available_status" -Needle '"actual_status": "available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_available_reason" -Needle '"actual_reason": "audit_rollback_append_engine_available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_engine_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_append_contract_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_contract_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_append_contract_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_count" -Needle '"case_count": 24' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_missing_case" -Needle '"case": "missing_append_envelope_pair_current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_missing_reason" -Needle '"actual_reason": "audit_append_envelope_missing_and_rollback_transaction_envelope_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_scope_case" -Needle '"case": "audit_append_envelope_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_schema_case" -Needle '"case": "audit_append_envelope_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_provenance_case" -Needle '"case": "audit_append_envelope_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_provenance_binding_case" -Needle '"case": "audit_append_envelope_provenance_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_policy_binding_case" -Needle '"case": "audit_append_envelope_policy_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_policy_id_case" -Needle '"case": "audit_append_envelope_write_policy_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_availability_case" -Needle '"case": "audit_append_envelope_availability_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_availability_id_case" -Needle '"case": "audit_append_envelope_availability_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_storage_id_case" -Needle '"case": "audit_append_envelope_storage_layout_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_engine_id_case" -Needle '"case": "audit_append_envelope_append_engine_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_audit_storage_case" -Needle '"case": "audit_ledger_storage_layout_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_scope_case" -Needle '"case": "rollback_transaction_envelope_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_schema_case" -Needle '"case": "rollback_transaction_envelope_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_provenance_case" -Needle '"case": "rollback_transaction_envelope_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_provenance_binding_case" -Needle '"case": "rollback_transaction_envelope_provenance_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_policy_binding_case" -Needle '"case": "rollback_transaction_envelope_policy_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_policy_id_case" -Needle '"case": "rollback_transaction_envelope_write_policy_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_availability_case" -Needle '"case": "rollback_transaction_envelope_availability_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_availability_id_case" -Needle '"case": "rollback_transaction_envelope_availability_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_storage_id_case" -Needle '"case": "rollback_transaction_envelope_storage_layout_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_engine_id_case" -Needle '"case": "rollback_transaction_envelope_append_engine_id_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_rollback_storage_case" -Needle '"case": "rollback_store_storage_layout_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_engine_case" -Needle '"case": "available_envelopes_append_engine_still_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_engine_status" -Needle '"actual_status": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_engine_reason" -Needle '"actual_reason": "audit_ledger_append_engine_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_contract_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_append_payload_hash_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_payload_hash_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_append_payload_hash_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_count" -Needle '"case_count": 20' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_missing_case" -Needle '"case": "missing_payload_hash_pair_current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_missing_reason" -Needle '"actual_reason": "audit_record_append_payload_hash_missing_and_rollback_transaction_append_payload_hash_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_scope_case" -Needle '"case": "audit_record_payload_hash_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_schema_case" -Needle '"case": "audit_record_payload_hash_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_provenance_case" -Needle '"case": "audit_record_payload_hash_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_retained_binding_case" -Needle '"case": "audit_record_payload_hash_retained_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_slot_binding_case" -Needle '"case": "audit_record_payload_hash_service_slot_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_request_binding_case" -Needle '"case": "audit_record_payload_hash_write_request_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_contract_binding_case" -Needle '"case": "audit_record_payload_hash_append_contract_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_target_binding_case" -Needle '"case": "audit_record_payload_hash_target_schema_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_payload_case" -Needle '"case": "audit_record_payload_hash_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_retained_missing_case" -Needle '"case": "audit_record_retained_audit_rollback_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_slot_missing_case" -Needle '"case": "audit_record_service_slot_reservation_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_audit_contract_missing_case" -Needle '"case": "audit_record_append_contract_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_rollback_scope_case" -Needle '"case": "rollback_transaction_payload_hash_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_rollback_schema_case" -Needle '"case": "rollback_transaction_payload_hash_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_rollback_provenance_case" -Needle '"case": "rollback_transaction_payload_hash_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_rollback_contract_binding_case" -Needle '"case": "rollback_transaction_payload_hash_append_contract_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_rollback_payload_case" -Needle '"case": "rollback_transaction_payload_hash_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_rollback_contract_missing_case" -Needle '"case": "rollback_transaction_append_contract_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_available_case" -Needle '"case": "available_payload_hashes_still_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_available_status" -Needle '"actual_status": "available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_available_reason" -Needle '"actual_reason": "audit_rollback_append_payload_hash_available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_payload_hash_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_append_intent_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_append_intent_selftest"
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_append_intent_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_count" -Needle '"case_count": 20' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_missing_case" -Needle '"case": "missing_append_intent_pair_current_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_missing_reason" -Needle '"actual_reason": "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_scope_case" -Needle '"case": "audit_record_append_intent_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_schema_case" -Needle '"case": "audit_record_append_intent_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_provenance_case" -Needle '"case": "audit_record_append_intent_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_provenance_binding_case" -Needle '"case": "audit_record_append_intent_provenance_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_contract_binding_case" -Needle '"case": "audit_record_append_intent_append_contract_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_engine_binding_case" -Needle '"case": "audit_record_append_intent_append_engine_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_storage_binding_case" -Needle '"case": "audit_record_append_intent_storage_layout_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_policy_binding_case" -Needle '"case": "audit_record_append_intent_write_policy_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_availability_binding_case" -Needle '"case": "audit_record_append_intent_availability_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_payload_case" -Needle '"case": "audit_record_append_intent_payload_hash_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_contract_missing_case" -Needle '"case": "audit_record_append_intent_append_contract_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_audit_payload_envelope_case" -Needle '"case": "audit_record_append_intent_payload_hash_envelope_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_rollback_scope_case" -Needle '"case": "rollback_transaction_append_intent_previous_boot"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_rollback_schema_case" -Needle '"case": "rollback_transaction_append_intent_wrong_schema"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_rollback_provenance_case" -Needle '"case": "rollback_transaction_append_intent_provenance_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_rollback_contract_binding_case" -Needle '"case": "rollback_transaction_append_intent_append_contract_binding_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_rollback_payload_case" -Needle '"case": "rollback_transaction_append_intent_payload_hash_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_rollback_payload_envelope_case" -Needle '"case": "rollback_transaction_append_intent_payload_hash_envelope_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_available_case" -Needle '"case": "available_append_intents_still_non_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_available_status" -Needle '"actual_status": "available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_available_reason" -Needle '"actual_reason": "audit_rollback_append_intent_available"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_audit_rollback_append_intent_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.audit_rollback_write_boundary_selftest" -ExpectedMarker "RAIOS_AGENT_END module.audit_rollback_write_boundary_selftest"
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_schema" -Needle '"schema": "raios.module_audit_rollback_write_boundary_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_no_audit_records" -Needle '"creates_durable_audit_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_no_rollback_plans" -Needle '"creates_rollback_plans": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_no_recovery_artifact" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_count" -Needle '"case_count": 22' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_missing_manifest" -Needle '"case": "missing_manifest_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_stale_artifact" -Needle '"case": "stale_artifact_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_substituted_vm_report" -Needle '"case": "substituted_vm_report_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_previous_boot" -Needle '"case": "previous_boot_write_request"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_schema_mismatch" -Needle '"case": "write_request_schema_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_missing_grant" -Needle '"case": "missing_computed_grant_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_attestation_mismatch" -Needle '"case": "local_attestation_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_approval_mismatch" -Needle '"case": "local_approval_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_audit_hash_mismatch" -Needle '"case": "audit_record_service_slot_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_rollback_hash_mismatch" -Needle '"case": "rollback_plan_service_slot_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_service_slot_substituted" -Needle '"case": "substituted_service_slot_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_recovery_separate" -Needle '"case": "recovery_artifact_loader_requested"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_ledger_available_case" -Needle '"case": "durable_audit_ledger_available_rollback_store_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_rollback_available_case" -Needle '"case": "rollback_store_available_durable_audit_ledger_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_policy_denied_case" -Needle '"case": "availability_facts_present_policy_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_policy_status" -Needle '"actual_status": "denied_missing_durable_write_policy"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_policy_reason" -Needle '"actual_reason": "durable_write_policy_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_rollback_policy_case" -Needle '"case": "durable_write_policy_available_rollback_policy_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_rollback_policy_status" -Needle '"actual_status": "denied_missing_rollback_install_policy"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_rollback_policy_reason" -Needle '"actual_reason": "rollback_install_policy_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_append_missing_case" -Needle '"case": "policy_facts_available_append_contract_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_append_missing_status" -Needle '"actual_status": "denied_missing_append_contract"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_append_missing_reason" -Needle '"actual_reason": "audit_append_envelope_missing_and_rollback_transaction_envelope_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_rollback_transaction_missing_case" -Needle '"case": "audit_append_available_rollback_transaction_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_rollback_transaction_missing_reason" -Needle '"actual_reason": "rollback_transaction_envelope_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_intent_missing_case" -Needle '"case": "append_contract_available_append_intent_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_intent_missing_status" -Needle '"actual_status": "denied_missing_append_intent"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_intent_missing_reason" -Needle '"actual_reason": "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_payload_envelope_missing_case" -Needle '"case": "append_intent_payload_hash_envelope_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_payload_envelope_missing_reason" -Needle '"actual_reason": "audit_record_append_payload_hash_envelope_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_writer_case" -Needle '"case": "append_intents_available_writer_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_writer_status" -Needle '"actual_status": "denied_write_path_unimplemented"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_writer_reason" -Needle '"actual_reason": "durable_audit_rollback_writer_unimplemented"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_valid_denied" -Needle '"case": "accepted_current_boot_preconditions_write_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_valid_status" -Needle '"actual_status": "denied_missing_durable_write_boundary"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_valid_reason" -Needle '"actual_reason": "durable_audit_write_missing_and_rollback_install_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_write_boundary_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_manifest_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_manifest_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_schema" -Needle '"schema": "raios.module_load_gate_manifest_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_no_records" -Needle '"creates_retained_manifest_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_no_manifest_json" -Needle '"accepts_manifest_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_no_load" -Needle '"loads_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_count" -Needle '"case_count": 7' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_missing_case" -Needle '"case": "missing_retained_manifest_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_missing_reason" -Needle '"actual_reason": "retained_module_manifest_reference_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_valid_case" -Needle '"case": "accepted_current_boot_manifest_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_valid_status" -Needle '"actual_status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_valid_state" -Needle '"actual_module_manifest_state": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_valid_hash_exposed" -Needle '"accepted_manifest_hash": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_stale_case" -Needle '"case": "stale_dropped_manifest_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_stale_reason" -Needle '"actual_reason": "retained_module_manifest_reference_stale_or_dropped_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_previous_case" -Needle '"case": "previous_boot_or_unretained_manifest_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_previous_reason" -Needle '"actual_reason": "retained_module_manifest_reference_previous_boot_or_unretained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_wrong_schema_case" -Needle '"case": "wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_wrong_schema_reason" -Needle '"actual_reason": "retained_module_manifest_reference_wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_substituted_case" -Needle '"case": "substituted_manifest_reference_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_substituted_reason" -Needle '"actual_reason": "retained_module_manifest_reference_substituted_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_hash_case" -Needle '"case": "manifest_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_hash_reason" -Needle '"actual_reason": "retained_module_manifest_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_rejected_state" -Needle '"actual_module_manifest_state": "rejected_retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_rejected_hash_not_exposed" -Needle '"accepted_manifest_hash": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_manifest_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_artifact_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_artifact_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_schema" -Needle '"schema": "raios.module_load_gate_artifact_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_no_records" -Needle '"creates_retained_candidate_artifact_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_no_manifest_json" -Needle '"accepts_manifest_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_no_load" -Needle '"loads_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_count" -Needle '"case_count": 9' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_missing_case" -Needle '"case": "missing_retained_candidate_artifact_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_missing_reason" -Needle '"actual_reason": "retained_candidate_artifact_reference_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_valid_case" -Needle '"case": "accepted_current_boot_artifact_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_valid_status" -Needle '"actual_status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_valid_state" -Needle '"actual_candidate_artifact_state": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_valid_hash_exposed" -Needle '"accepted_artifact_hash": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_stale_case" -Needle '"case": "stale_dropped_retained_artifact_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_wrong_schema_case" -Needle '"case": "wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_substituted_case" -Needle '"case": "substituted_artifact_reference_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_hash_case" -Needle '"case": "artifact_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_manifest_case" -Needle '"case": "manifest_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_grant_case" -Needle '"case": "computed_grant_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_rejected_state" -Needle '"actual_candidate_artifact_state": "rejected_retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_rejected_hash_not_exposed" -Needle '"accepted_artifact_hash": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_artifact_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_vm_report_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_vm_report_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_schema" -Needle '"schema": "raios.module_load_gate_vm_report_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_no_records" -Needle '"creates_retained_vm_test_report_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_no_manifest_json" -Needle '"accepts_manifest_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_no_vm_report_json" -Needle '"accepts_vm_report_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_no_unsigned_code" -Needle '"accepts_unsigned_service_code": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_no_load" -Needle '"loads_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_count" -Needle '"case_count": 11' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_missing_case" -Needle '"case": "missing_retained_vm_test_report_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_missing_reason" -Needle '"actual_reason": "retained_vm_test_report_reference_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_valid_case" -Needle '"case": "accepted_current_boot_report_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_valid_status" -Needle '"actual_status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_valid_reason" -Needle '"actual_reason": "retained_vm_test_report_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_valid_state" -Needle '"actual_vm_test_report_state": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_valid_hash_exposed" -Needle '"accepted_vm_test_report_hash": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_stale_case" -Needle '"case": "stale_dropped_retained_vm_test_report_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_wrong_schema_case" -Needle '"case": "wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_substituted_case" -Needle '"case": "substituted_vm_test_report_reference_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_hash_case" -Needle '"case": "vm_test_report_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_hash_reason" -Needle '"actual_reason": "retained_vm_test_report_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_manifest_case" -Needle '"case": "manifest_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_artifact_case" -Needle '"case": "artifact_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_grant_case" -Needle '"case": "computed_grant_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_report_hash_case" -Needle '"case": "vm_test_report_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_report_hash_reason" -Needle '"actual_reason": "retained_vm_test_report_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_rejected_state" -Needle '"actual_vm_test_report_state": "rejected_retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_rejected_hash_not_exposed" -Needle '"accepted_vm_test_report_hash": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_vm_report_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_attestation_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_attestation_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_schema" -Needle '"schema": "raios.module_load_gate_local_attestation_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_no_records" -Needle '"creates_retained_local_attestation_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_no_attestation_json" -Needle '"accepts_local_attestation_json": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_no_load" -Needle '"loads_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_count" -Needle '"case_count": 11' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_missing_case" -Needle '"case": "missing_retained_local_attestation_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_missing_reason" -Needle '"actual_reason": "retained_local_attestation_reference_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_valid_case" -Needle '"case": "accepted_current_boot_attestation_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_valid_status" -Needle '"actual_status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_valid_reason" -Needle '"actual_reason": "retained_local_attestation_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_valid_state" -Needle '"actual_local_attestation_state": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_valid_hash_exposed" -Needle '"accepted_local_attestation_hash": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_stale_case" -Needle '"case": "stale_dropped_retained_local_attestation_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_wrong_schema_case" -Needle '"case": "wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_substituted_case" -Needle '"case": "substituted_local_attestation_reference_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_hash_case" -Needle '"case": "local_attestation_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_hash_reason" -Needle '"actual_reason": "retained_local_attestation_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_manifest_case" -Needle '"case": "manifest_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_artifact_case" -Needle '"case": "artifact_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_vm_report_case" -Needle '"case": "vm_report_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_grant_case" -Needle '"case": "computed_grant_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_rejected_state" -Needle '"actual_local_attestation_state": "rejected_retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_rejected_hash_not_exposed" -Needle '"accepted_local_attestation_hash": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_attestation_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent module.load_gate_approval_selftest" -ExpectedMarker "RAIOS_AGENT_END module.load_gate_approval_selftest"
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_schema" -Needle '"schema": "raios.module_load_gate_local_approval_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_no_records" -Needle '"creates_retained_local_approval_reference_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_no_approval_text" -Needle '"accepts_local_approval_text": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_no_artifact_bytes" -Needle '"accepts_artifact_bytes": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_no_load" -Needle '"loads_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_count" -Needle '"case_count": 12' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_missing_case" -Needle '"case": "missing_retained_local_approval_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_missing_reason" -Needle '"actual_reason": "retained_local_approval_reference_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_valid_case" -Needle '"case": "accepted_current_boot_approval_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_valid_status" -Needle '"actual_status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_valid_reason" -Needle '"actual_reason": "retained_local_approval_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_valid_state" -Needle '"actual_local_approval_state": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_valid_hash_exposed" -Needle '"accepted_local_approval_hash": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_stale_case" -Needle '"case": "stale_dropped_retained_local_approval_reference_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_wrong_schema_case" -Needle '"case": "wrong_schema_or_variant"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_substituted_case" -Needle '"case": "substituted_local_approval_reference_record"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_hash_case" -Needle '"case": "local_approval_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_hash_reason" -Needle '"actual_reason": "retained_local_approval_reference_hash_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_attestation_case" -Needle '"case": "local_attestation_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_grant_case" -Needle '"case": "computed_grant_reference_mismatch"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_rejected_state" -Needle '"actual_local_approval_state": "rejected_retained_reference"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_rejected_hash_not_exposed" -Needle '"accepted_local_approval_hash": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_can_load_false" -Needle '"can_load": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_approval_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

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
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_missing_audit_reason" -Needle '"actual_reason": "durable_audit_write_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_missing_rollback" -Needle '"case": "missing_rollback_plan"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_gate_audit_rollback_selftest_missing_rollback_reason" -Needle '"actual_reason": "rollback_install_missing"' -TimeoutSeconds 1
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
    $moduleFinalLoadResponse = Get-LastAgentResponseJson -Method "module.load_ephemeral"
    Assert-LogContains -Name "policy:mutating_load_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_gate_schema" -Needle '"schema": "raios.module_load_gate.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:mutating_load_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_mode_ram_only" -Needle '"load_mode": "ram_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_capability" -Needle '"requested_capability": "cap.module.load_ephemeral"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_load_target" -Needle '"target": "live_service_graph"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_manifest_retained" -Needle '"module_manifest": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_manifest_reference" -Needle '"retained_module_manifest_reference": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_manifest_present" -Needle '"schema": "raios.module_manifest_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_manifest_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_manifest_event_id" -Needle '"retained_manifest_reference_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_manifest_ref_hash" -Needle "`"manifest_reference_hash`": `"sha256:$moduleManifestReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:candidate_artifact_retained" -Needle '"candidate_artifact": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_artifact_reference" -Needle '"retained_candidate_artifact_reference": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_artifact_present" -Needle '"schema": "raios.module_candidate_artifact_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_artifact_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    $moduleLoadArtifactEventId = [string]$moduleFinalLoadResponse.body.retained_candidate_artifact_reference.event_id
    $moduleLoadArtifactEventIdMatches = $moduleLoadArtifactEventId -eq $moduleArtifactRetainedReferenceEventId
    Add-Predicate -Name "policy:module_retained_artifact_event_id" -Expected $moduleArtifactRetainedReferenceEventId -Passed $moduleLoadArtifactEventIdMatches -Actual $moduleLoadArtifactEventId
    if (-not $moduleLoadArtifactEventIdMatches) {
        throw "Expected retained artifact event id $moduleArtifactRetainedReferenceEventId in module.load_ephemeral, got $moduleLoadArtifactEventId"
    }
    Assert-LogContains -Name "policy:module_retained_artifact_ref_hash" -Needle "`"artifact_reference_hash`": `"sha256:$moduleArtifactReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_vm_report_retained" -Needle '"vm_test_report": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_vm_report_reference" -Needle '"retained_vm_test_report_reference": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_vm_report_present" -Needle '"schema": "raios.module_vm_test_report_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_vm_report_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    $moduleLoadVmReportEventId = [string]$moduleFinalLoadResponse.body.retained_vm_test_report_reference.event_id
    $moduleLoadVmReportEventIdMatches = $moduleLoadVmReportEventId -eq $moduleVmReportRetainedReferenceEventId
    Add-Predicate -Name "policy:module_retained_vm_report_event_id" -Expected $moduleVmReportRetainedReferenceEventId -Passed $moduleLoadVmReportEventIdMatches -Actual $moduleLoadVmReportEventId
    if (-not $moduleLoadVmReportEventIdMatches) {
        throw "Expected retained VM report event id $moduleVmReportRetainedReferenceEventId in module.load_ephemeral, got $moduleLoadVmReportEventId"
    }
    Assert-LogContains -Name "policy:module_retained_vm_report_ref_hash" -Needle "`"vm_test_report_reference_hash`": `"sha256:$moduleVmReportReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_vm_report_hash" -Needle "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_attestation_retained" -Needle '"local_attestation": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_attestation_reference" -Needle '"retained_local_attestation_reference": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_attestation_present" -Needle '"schema": "raios.module_local_attestation_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_attestation_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_attestation_event_id" -Needle '"retained_local_attestation_reference_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_attestation_ref_hash" -Needle "`"local_attestation_reference_hash`": `"sha256:$moduleAttestationReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_computed_grant_retained" -Needle '"computed_capability_grant": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_reference" -Needle '"retained_computed_grant_reference": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_present" -Needle '"state": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_grant_hash" -Needle "`"computed_capability_grant_hash`": `"sha256:$moduleGrantHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_approval_retained" -Needle '"local_approval": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_approval_reference" -Needle '"retained_local_approval_reference": {' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_approval_present" -Needle '"schema": "raios.module_local_approval_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_approval_status" -Needle '"status": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    $moduleLoadApprovalEventId = [string]$moduleFinalLoadResponse.body.retained_local_approval_reference.event_id
    $moduleLoadApprovalEventIdMatches = $moduleLoadApprovalEventId -eq $moduleApprovalRetainedReferenceEventId
    Add-Predicate -Name "policy:module_retained_approval_event_id" -Expected $moduleApprovalRetainedReferenceEventId -Passed $moduleLoadApprovalEventIdMatches -Actual $moduleLoadApprovalEventId
    if (-not $moduleLoadApprovalEventIdMatches) {
        throw "Expected retained approval event id $moduleApprovalRetainedReferenceEventId in module.load_ephemeral, got $moduleLoadApprovalEventId"
    }
    Assert-LogContains -Name "policy:module_retained_approval_reason" -Needle '"reason": "retained_local_approval_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_approval_ref_hash" -Needle "`"local_approval_reference_hash`": `"sha256:$moduleApprovalReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_retained_approval_hash" -Needle "`"local_approval_hash`": `"sha256:$moduleAuditLocalApprovalHash`"" -TimeoutSeconds 1
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
    Assert-LogContains -Name "policy:module_manifest_retained_reason" -Needle '"reason": "retained_module_manifest_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:candidate_artifact_retained_reason" -Needle '"reason": "retained_candidate_artifact_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_vm_report_retained_reason" -Needle '"reason": "retained_vm_test_report_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_attestation_retained_reason" -Needle '"reason": "retained_local_attestation_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_approval_retained_reason" -Needle '"reason": "retained_local_approval_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_computed_grant_retained_not_authorizing" -Needle '"reason": "retained_computed_grant_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_reference_reason" -Needle '"reason": "durable_audit_write_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_rollback_reference_reason" -Needle '"reason": "rollback_install_missing"' -TimeoutSeconds 1
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
    Assert-LogContains -Name "policy:module_audit_retained_approval_reference_required" -Needle '"retained_local_approval_reference_event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_rollback_hash_retained" -Needle "`"rollback_plan_hash`": `"sha256:$moduleRollbackHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_audit_slot_id_retained" -Needle "`"ram_only_service_slot_id`": `"$moduleAuditRamOnlyServiceSlotId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_hash_retained" -Needle "`"service_slot_reservation_hash`": `"sha256:$moduleServiceSlotReservationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_slot_requirement_retained" -Needle '"ram_only_service_slot": {"state": "retained_hash_reference_only_not_allocated", "reason": "retained_service_slot_reservation_not_allocated", "required": true, "allocates_service_slot": false}' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_manifest_hash_retained" -Needle "`"manifest_hash`": `"sha256:$moduleGrantManifestHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_artifact_hash_retained" -Needle "`"artifact_hash`": `"sha256:$moduleGrantArtifactHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_vm_report_hash_retained" -Needle "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_attestation_hash_retained" -Needle "`"local_attestation_hash`": `"sha256:$moduleGrantAttestationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_approval_reference_hash_retained" -Needle "`"local_approval_reference_hash`": `"sha256:$moduleApprovalReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_local_approval_hash_retained" -Needle "`"local_approval_hash`": `"sha256:$moduleAuditLocalApprovalHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "policy:module_service_inventory_unchanged" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1

    Send-AgentCommand -Command "recovery.load_artifact" -ExpectedMarker "RAIOS_AGENT_END recovery.load_artifact"
    $recoveryLoadResponse = Get-LastAgentResponseJson -Method "recovery.load_artifact"
    Assert-LogContains -Name "policy:recovery_load_denied" -Needle '"code": "capability_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_schema" -Needle '"schema": "raios.recovery_artifact_load_boundary.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_event_id" -Needle '"event_id": "event.current_boot.' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_capability" -Needle '"requested_capability": "cap.recovery.load_artifact"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_separate_capability" -Needle '"separate_from": "cap.module.load_ephemeral"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_normal_path_not_used" -Needle '"normal_module_load_path_used": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_normal_cap_not_used" -Needle '"normal_module_capability_used": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_denial_evidence_schema" -Needle '"schema": "raios.recovery_artifact_load_denial_evidence.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_status" -Needle '"status": "denied_missing_recovery_artifact_evidence"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_identity_missing" -Needle '"recovery_artifact_identity": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_trust_missing" -Needle '"recovery_artifact_trust": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_vm_test_missing" -Needle '"recovery_vm_test": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_approval_missing" -Needle '"recovery_local_approval": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_loader_missing" -Needle '"recovery_loader": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_rollback_missing" -Needle '"recovery_rollback_evidence": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_identity_schema" -Needle '"schema": "raios.recovery_artifact_identity.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_trust_schema" -Needle '"schema": "raios.recovery_artifact_trust.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_vm_test_schema" -Needle '"schema": "raios.recovery_artifact_vm_test.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_approval_schema" -Needle '"schema": "raios.recovery_artifact_local_approval.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_loader_schema" -Needle '"schema": "raios.recovery_artifact_loader.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_rollback_schema" -Needle '"schema": "raios.recovery_artifact_rollback_evidence.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_no_normal_load" -Needle '"loads_normal_module": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_append_payload_not_authority" -Needle '"append_payload_hash_authority": false' -TimeoutSeconds 1
    Assert-LogContains -Name "policy:recovery_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1
    $recoveryLoadEventId = [string]$recoveryLoadResponse.body.event_id
    $recoveryLoadEventIdPresent = $recoveryLoadEventId.StartsWith("event.current_boot.")
    Add-Predicate -Name "policy:recovery_load_current_boot_event_id" -Expected "event.current_boot.*" -Passed $recoveryLoadEventIdPresent -Actual $recoveryLoadEventId
    if (-not $recoveryLoadEventIdPresent) {
        throw "Expected current-boot event id for recovery.load_artifact, got $recoveryLoadEventId"
    }

    Send-AgentCommand -Command "agent recovery.identity_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.identity_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_identity_diag_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_identity_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_recovery_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "no_normal_load"; Needle = '"loads_normal_module": false' },
        @{ Suffix = "status"; Needle = '"validation_status": "missing"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_identity_reference_absent"' },
        @{ Suffix = "retained_missing"; Needle = '"reason": "no_valid_recovery_artifact_identity_reference_retained"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryArtifactHash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    $recoveryIdentityCanonical = @(
        "canonicalization=raios.recovery_artifact_identity.canonical.v0",
        "schema=raios.recovery_artifact_identity.v0",
        "requested_capability=cap.recovery.load_artifact",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline",
        "scope=current_boot",
        "artifact_sha256=$recoveryArtifactHash",
        "accepts_artifact_bytes=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryIdentityReferenceHash = Get-TextSha256 -Text $recoveryIdentityCanonical
    $recoveryIdentityCommand = "agent recovery.identity_diagnostic $recoveryIdentityReferenceHash $recoveryArtifactHash"

    Send-AgentCommand -Command $recoveryIdentityCommand -ExpectedMarker "RAIOS_AGENT_END recovery.identity_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_identity_diag_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_identity_reference_valid_but_trust_and_loader_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "identity_hash_echo"; Needle = "`"identity_reference_hash`": `"sha256:$recoveryIdentityReferenceHash`"" },
        @{ Suffix = "artifact_hash_echo"; Needle = "`"artifact_hash`": `"sha256:$recoveryArtifactHash`"" },
        @{ Suffix = "still_denied"; Needle = '"can_move_beyond_denial": false' },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryIdentityResponse = Get-LastAgentResponseJson -Method "recovery.identity_diagnostic"
    $recoveryIdentityEventId = [string]$recoveryIdentityResponse.body.result.retained_recovery_artifact_identity_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_identity_retained_reference_event_id_captured" -Value $recoveryIdentityEventId

    Send-AgentCommand -Command "agent recovery.identity_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.identity_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_identity_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_identity_diagnostic_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_identity_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 6' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "absent_reference"' },
        @{ Suffix = "valid_case"; Needle = '"case": "accepted_current_boot_identity_still_denied"' },
        @{ Suffix = "stale_case"; Needle = '"case": "stale_previous_boot_reference"' },
        @{ Suffix = "wrong_schema_case"; Needle = '"case": "wrong_schema_identity_reference"' },
        @{ Suffix = "substituted_case"; Needle = '"case": "substituted_identity_reference_record"' },
        @{ Suffix = "mismatch_case"; Needle = '"case": "identity_reference_hash_mismatch"' },
        @{ Suffix = "valid_reason"; Needle = '"actual_reason": "recovery_artifact_identity_reference_valid_but_trust_and_loader_missing"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.trust_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.trust_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_trust_diag_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_trust_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_recovery_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "status"; Needle = '"validation_status": "missing"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_trust_reference_absent"' },
        @{ Suffix = "retained_missing"; Needle = '"reason": "no_valid_recovery_artifact_trust_reference_retained"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryTrustHash = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    $recoveryTrustCanonical = @(
        "canonicalization=raios.recovery_artifact_trust.canonical.v0",
        "schema=raios.recovery_artifact_trust.v0",
        "requested_capability=cap.recovery.load_artifact",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline",
        "scope=current_boot",
        "retained_recovery_artifact_identity_event_id=$recoveryIdentityEventId",
        "identity_reference_sha256=$recoveryIdentityReferenceHash",
        "artifact_sha256=$recoveryArtifactHash",
        "trust_sha256=$recoveryTrustHash",
        "accepts_artifact_bytes=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryTrustReferenceHash = Get-TextSha256 -Text $recoveryTrustCanonical
    $recoveryTrustCommand = "agent recovery.trust_diagnostic $recoveryTrustReferenceHash $recoveryIdentityEventId $recoveryIdentityReferenceHash $recoveryArtifactHash $recoveryTrustHash"

    Send-AgentCommand -Command $recoveryTrustCommand -ExpectedMarker "RAIOS_AGENT_END recovery.trust_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_trust_diag_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_trust_reference_valid_but_vm_test_and_loader_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "identity_event"; Needle = "`"retained_recovery_artifact_identity_event_id`": `"$recoveryIdentityEventId`"" },
        @{ Suffix = "trust_hash_echo"; Needle = "`"trust_reference_hash`": `"sha256:$recoveryTrustReferenceHash`"" },
        @{ Suffix = "identity_hash_echo"; Needle = "`"identity_reference_hash`": `"sha256:$recoveryIdentityReferenceHash`"" },
        @{ Suffix = "artifact_hash_echo"; Needle = "`"artifact_hash`": `"sha256:$recoveryArtifactHash`"" },
        @{ Suffix = "trust_material_hash_echo"; Needle = "`"trust_hash`": `"sha256:$recoveryTrustHash`"" },
        @{ Suffix = "still_denied"; Needle = '"can_move_beyond_denial": false' },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryTrustResponse = Get-LastAgentResponseJson -Method "recovery.trust_diagnostic"
    $recoveryTrustEventId = [string]$recoveryTrustResponse.body.result.retained_recovery_artifact_trust_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_trust_retained_reference_event_id_captured" -Value $recoveryTrustEventId

    Send-AgentCommand -Command "agent recovery.trust_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.trust_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_trust_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_trust_diagnostic_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_trust_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 8' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "absent_reference"' },
        @{ Suffix = "valid_case"; Needle = '"case": "accepted_current_boot_trust_still_denied"' },
        @{ Suffix = "stale_case"; Needle = '"case": "stale_previous_boot_reference"' },
        @{ Suffix = "identity_not_current_case"; Needle = '"case": "retained_identity_event_not_current_boot"' },
        @{ Suffix = "identity_missing_case"; Needle = '"case": "retained_identity_missing"' },
        @{ Suffix = "identity_schema_case"; Needle = '"case": "retained_identity_wrong_schema_or_variant"' },
        @{ Suffix = "substituted_case"; Needle = '"case": "substituted_identity_reference_record"' },
        @{ Suffix = "mismatch_case"; Needle = '"case": "trust_reference_hash_mismatch"' },
        @{ Suffix = "valid_reason"; Needle = '"actual_reason": "recovery_artifact_trust_reference_valid_but_vm_test_and_loader_missing"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.vm_test_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.vm_test_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_vm_test_diag_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_vm_test_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_vm_test_json"; Needle = '"accepts_vm_test_json": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_recovery_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "status"; Needle = '"validation_status": "missing"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_vm_test_reference_absent"' },
        @{ Suffix = "retained_missing"; Needle = '"reason": "no_valid_recovery_artifact_vm_test_reference_retained"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryVmTestHash = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
    $recoveryVmTestCanonical = @(
        "canonicalization=raios.recovery_artifact_vm_test.canonical.v0",
        "schema=raios.recovery_artifact_vm_test.v0",
        "requested_capability=cap.recovery.load_artifact",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline",
        "scope=current_boot",
        "retained_recovery_artifact_identity_event_id=$recoveryIdentityEventId",
        "retained_recovery_artifact_trust_event_id=$recoveryTrustEventId",
        "identity_reference_sha256=$recoveryIdentityReferenceHash",
        "trust_reference_sha256=$recoveryTrustReferenceHash",
        "artifact_sha256=$recoveryArtifactHash",
        "trust_sha256=$recoveryTrustHash",
        "vm_test_sha256=$recoveryVmTestHash",
        "accepts_vm_test_json=false",
        "accepts_artifact_bytes=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryVmTestReferenceHash = Get-TextSha256 -Text $recoveryVmTestCanonical
    $recoveryVmTestCommand = "agent recovery.vm_test_diagnostic $recoveryVmTestReferenceHash $recoveryIdentityEventId $recoveryTrustEventId $recoveryIdentityReferenceHash $recoveryTrustReferenceHash $recoveryArtifactHash $recoveryTrustHash $recoveryVmTestHash"

    Send-AgentCommand -Command $recoveryVmTestCommand -ExpectedMarker "RAIOS_AGENT_END recovery.vm_test_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_vm_test_diag_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_vm_test_reference_valid_but_local_approval_and_loader_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "identity_event"; Needle = "`"retained_recovery_artifact_identity_event_id`": `"$recoveryIdentityEventId`"" },
        @{ Suffix = "trust_event"; Needle = "`"retained_recovery_artifact_trust_event_id`": `"$recoveryTrustEventId`"" },
        @{ Suffix = "vm_test_hash_echo"; Needle = "`"vm_test_reference_hash`": `"sha256:$recoveryVmTestReferenceHash`"" },
        @{ Suffix = "trust_hash_echo"; Needle = "`"trust_reference_hash`": `"sha256:$recoveryTrustReferenceHash`"" },
        @{ Suffix = "identity_hash_echo"; Needle = "`"identity_reference_hash`": `"sha256:$recoveryIdentityReferenceHash`"" },
        @{ Suffix = "vm_test_material_hash_echo"; Needle = "`"vm_test_hash`": `"sha256:$recoveryVmTestHash`"" },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryVmTestResponse = Get-LastAgentResponseJson -Method "recovery.vm_test_diagnostic"
    $recoveryVmTestEventId = [string]$recoveryVmTestResponse.body.result.retained_recovery_artifact_vm_test_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_vm_test_retained_reference_event_id_captured" -Value $recoveryVmTestEventId

    Send-AgentCommand -Command "agent recovery.vm_test_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.vm_test_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_vm_test_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_vm_test_diagnostic_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_vm_test_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "absent_reference"' },
        @{ Suffix = "valid_case"; Needle = '"case": "accepted_current_boot_vm_test_still_denied"' },
        @{ Suffix = "stale_case"; Needle = '"case": "stale_previous_boot_reference"' },
        @{ Suffix = "trust_not_current_case"; Needle = '"case": "retained_trust_event_not_current_boot"' },
        @{ Suffix = "identity_missing_case"; Needle = '"case": "retained_identity_missing"' },
        @{ Suffix = "trust_schema_case"; Needle = '"case": "retained_trust_wrong_schema_or_variant"' },
        @{ Suffix = "substituted_case"; Needle = '"case": "substituted_trust_reference_record"' },
        @{ Suffix = "mismatch_case"; Needle = '"case": "vm_test_reference_hash_mismatch"' },
        @{ Suffix = "binding_mismatch_case"; Needle = '"case": "trust_binding_mismatch"' },
        @{ Suffix = "valid_reason"; Needle = '"actual_reason": "recovery_artifact_vm_test_reference_valid_but_local_approval_and_loader_missing"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.local_approval_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.local_approval_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_local_approval_diag_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_local_approval_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_approval_text"; Needle = '"accepts_local_approval_text": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_recovery_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "status"; Needle = '"validation_status": "missing"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_local_approval_reference_absent"' },
        @{ Suffix = "retained_missing"; Needle = '"reason": "no_valid_recovery_artifact_local_approval_reference_retained"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLocalApprovalHash = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
    $recoveryLocalApprovalCanonical = @(
        "canonicalization=raios.recovery_artifact_local_approval.canonical.v0",
        "schema=raios.recovery_artifact_local_approval.v0",
        "requested_capability=cap.recovery.load_artifact",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline",
        "scope=current_boot",
        "retained_recovery_artifact_identity_event_id=$recoveryIdentityEventId",
        "retained_recovery_artifact_trust_event_id=$recoveryTrustEventId",
        "retained_recovery_artifact_vm_test_event_id=$recoveryVmTestEventId",
        "identity_reference_sha256=$recoveryIdentityReferenceHash",
        "trust_reference_sha256=$recoveryTrustReferenceHash",
        "vm_test_reference_sha256=$recoveryVmTestReferenceHash",
        "artifact_sha256=$recoveryArtifactHash",
        "trust_sha256=$recoveryTrustHash",
        "vm_test_sha256=$recoveryVmTestHash",
        "local_approval_sha256=$recoveryLocalApprovalHash",
        "accepts_local_approval_text=false",
        "accepts_artifact_bytes=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryLocalApprovalReferenceHash = Get-TextSha256 -Text $recoveryLocalApprovalCanonical
    $recoveryLocalApprovalCommand = "agent recovery.local_approval_diagnostic $recoveryLocalApprovalReferenceHash $recoveryIdentityEventId $recoveryTrustEventId $recoveryVmTestEventId $recoveryIdentityReferenceHash $recoveryTrustReferenceHash $recoveryVmTestReferenceHash $recoveryArtifactHash $recoveryTrustHash $recoveryVmTestHash $recoveryLocalApprovalHash"

    Send-AgentCommand -Command $recoveryLocalApprovalCommand -ExpectedMarker "RAIOS_AGENT_END recovery.local_approval_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_local_approval_diag_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_local_approval_reference_valid_but_loader_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "identity_event"; Needle = "`"retained_recovery_artifact_identity_event_id`": `"$recoveryIdentityEventId`"" },
        @{ Suffix = "trust_event"; Needle = "`"retained_recovery_artifact_trust_event_id`": `"$recoveryTrustEventId`"" },
        @{ Suffix = "vm_test_event"; Needle = "`"retained_recovery_artifact_vm_test_event_id`": `"$recoveryVmTestEventId`"" },
        @{ Suffix = "approval_hash_echo"; Needle = "`"local_approval_reference_hash`": `"sha256:$recoveryLocalApprovalReferenceHash`"" },
        @{ Suffix = "vm_test_hash_echo"; Needle = "`"vm_test_reference_hash`": `"sha256:$recoveryVmTestReferenceHash`"" },
        @{ Suffix = "local_approval_material_hash_echo"; Needle = "`"local_approval_hash`": `"sha256:$recoveryLocalApprovalHash`"" },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLocalApprovalResponse = Get-LastAgentResponseJson -Method "recovery.local_approval_diagnostic"
    $recoveryLocalApprovalEventId = [string]$recoveryLocalApprovalResponse.body.result.retained_recovery_artifact_local_approval_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_local_approval_retained_reference_event_id_captured" -Value $recoveryLocalApprovalEventId

    Send-AgentCommand -Command "agent recovery.local_approval_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.local_approval_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_local_approval_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_local_approval_diagnostic_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_local_approval_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 11' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "absent_reference"' },
        @{ Suffix = "valid_case"; Needle = '"case": "accepted_current_boot_local_approval_still_denied"' },
        @{ Suffix = "stale_case"; Needle = '"case": "stale_previous_boot_reference"' },
        @{ Suffix = "vm_not_current_case"; Needle = '"case": "retained_vm_test_event_not_current_boot"' },
        @{ Suffix = "vm_missing_case"; Needle = '"case": "retained_vm_test_missing"' },
        @{ Suffix = "vm_schema_case"; Needle = '"case": "retained_vm_test_wrong_schema_or_variant"' },
        @{ Suffix = "substituted_case"; Needle = '"case": "substituted_vm_test_reference_record"' },
        @{ Suffix = "approval_mismatch_case"; Needle = '"case": "local_approval_reference_hash_mismatch"' },
        @{ Suffix = "vm_hash_mismatch_case"; Needle = '"case": "vm_test_reference_hash_mismatch"' },
        @{ Suffix = "chain_mismatch_case"; Needle = '"case": "retained_chain_mismatch"' },
        @{ Suffix = "valid_reason"; Needle = '"actual_reason": "recovery_artifact_local_approval_reference_valid_but_loader_missing"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.loader_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.loader_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_loader_diag_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_loader_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_loader_descriptor"; Needle = '"accepts_loader_descriptor": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_loader_load"; Needle = '"loads_recovery_loader": false' },
        @{ Suffix = "status"; Needle = '"validation_status": "missing"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_loader_reference_absent"' },
        @{ Suffix = "retained_missing"; Needle = '"reason": "no_valid_recovery_artifact_loader_reference_retained"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLoaderHash = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
    $recoveryLoaderCanonical = @(
        "canonicalization=raios.recovery_artifact_loader.canonical.v0",
        "schema=raios.recovery_artifact_loader.v0",
        "requested_capability=cap.recovery.load_artifact",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline",
        "scope=current_boot",
        "retained_recovery_artifact_identity_event_id=$recoveryIdentityEventId",
        "retained_recovery_artifact_trust_event_id=$recoveryTrustEventId",
        "retained_recovery_artifact_vm_test_event_id=$recoveryVmTestEventId",
        "retained_recovery_artifact_local_approval_event_id=$recoveryLocalApprovalEventId",
        "identity_reference_sha256=$recoveryIdentityReferenceHash",
        "trust_reference_sha256=$recoveryTrustReferenceHash",
        "vm_test_reference_sha256=$recoveryVmTestReferenceHash",
        "local_approval_reference_sha256=$recoveryLocalApprovalReferenceHash",
        "artifact_sha256=$recoveryArtifactHash",
        "trust_sha256=$recoveryTrustHash",
        "vm_test_sha256=$recoveryVmTestHash",
        "local_approval_sha256=$recoveryLocalApprovalHash",
        "loader_sha256=$recoveryLoaderHash",
        "accepts_loader_descriptor=false",
        "accepts_artifact_bytes=false",
        "loads_recovery_loader=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryLoaderReferenceHash = Get-TextSha256 -Text $recoveryLoaderCanonical
    $recoveryLoaderCommand = "agent recovery.loader_diagnostic $recoveryLoaderReferenceHash $recoveryIdentityEventId $recoveryTrustEventId $recoveryVmTestEventId $recoveryLocalApprovalEventId $recoveryIdentityReferenceHash $recoveryTrustReferenceHash $recoveryVmTestReferenceHash $recoveryLocalApprovalReferenceHash $recoveryArtifactHash $recoveryTrustHash $recoveryVmTestHash $recoveryLocalApprovalHash $recoveryLoaderHash"

    Send-AgentCommand -Command $recoveryLoaderCommand -ExpectedMarker "RAIOS_AGENT_END recovery.loader_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_loader_diag_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_loader_reference_valid_but_rollback_evidence_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "approval_event"; Needle = "`"retained_recovery_artifact_local_approval_event_id`": `"$recoveryLocalApprovalEventId`"" },
        @{ Suffix = "loader_ref_hash"; Needle = "`"loader_reference_hash`": `"sha256:$recoveryLoaderReferenceHash`"" },
        @{ Suffix = "approval_ref_hash"; Needle = "`"local_approval_reference_hash`": `"sha256:$recoveryLocalApprovalReferenceHash`"" },
        @{ Suffix = "loader_hash"; Needle = "`"loader_hash`": `"sha256:$recoveryLoaderHash`"" },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLoaderResponse = Get-LastAgentResponseJson -Method "recovery.loader_diagnostic"
    $recoveryLoaderEventId = [string]$recoveryLoaderResponse.body.result.retained_recovery_artifact_loader_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_loader_retained_reference_event_id_captured" -Value $recoveryLoaderEventId

    Send-AgentCommand -Command "agent recovery.loader_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.loader_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_loader_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_loader_diagnostic_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_loader_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "absent_reference"' },
        @{ Suffix = "valid_case"; Needle = '"case": "accepted_current_boot_loader_still_denied"' },
        @{ Suffix = "stale_case"; Needle = '"case": "stale_previous_boot_reference"' },
        @{ Suffix = "approval_not_current_case"; Needle = '"case": "retained_local_approval_event_not_current_boot"' },
        @{ Suffix = "approval_missing_case"; Needle = '"case": "retained_local_approval_missing"' },
        @{ Suffix = "approval_schema_case"; Needle = '"case": "retained_local_approval_wrong_schema_or_variant"' },
        @{ Suffix = "substituted_case"; Needle = '"case": "substituted_local_approval_reference_record"' },
        @{ Suffix = "mismatch_case"; Needle = '"case": "loader_reference_hash_mismatch"' },
        @{ Suffix = "chain_mismatch_case"; Needle = '"case": "retained_chain_mismatch"' },
        @{ Suffix = "valid_reason"; Needle = '"actual_reason": "recovery_artifact_loader_reference_valid_but_rollback_evidence_missing"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.rollback_evidence_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_evidence_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_evidence_diag_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_rollback_evidence_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_rollback_json"; Needle = '"accepts_rollback_evidence_json": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_durable"; Needle = '"creates_durable_records": false' },
        @{ Suffix = "status"; Needle = '"validation_status": "missing"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_rollback_evidence_reference_absent"' },
        @{ Suffix = "retained_missing"; Needle = '"reason": "no_valid_recovery_artifact_rollback_evidence_reference_retained"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRollbackEvidenceHash = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    $recoveryRollbackEvidenceCanonical = @(
        "canonicalization=raios.recovery_artifact_rollback_evidence.canonical.v0",
        "schema=raios.recovery_artifact_rollback_evidence.v0",
        "requested_capability=cap.recovery.load_artifact",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline",
        "scope=current_boot",
        "retained_recovery_artifact_identity_event_id=$recoveryIdentityEventId",
        "retained_recovery_artifact_trust_event_id=$recoveryTrustEventId",
        "retained_recovery_artifact_vm_test_event_id=$recoveryVmTestEventId",
        "retained_recovery_artifact_local_approval_event_id=$recoveryLocalApprovalEventId",
        "retained_recovery_artifact_loader_event_id=$recoveryLoaderEventId",
        "identity_reference_sha256=$recoveryIdentityReferenceHash",
        "trust_reference_sha256=$recoveryTrustReferenceHash",
        "vm_test_reference_sha256=$recoveryVmTestReferenceHash",
        "local_approval_reference_sha256=$recoveryLocalApprovalReferenceHash",
        "loader_reference_sha256=$recoveryLoaderReferenceHash",
        "artifact_sha256=$recoveryArtifactHash",
        "trust_sha256=$recoveryTrustHash",
        "vm_test_sha256=$recoveryVmTestHash",
        "local_approval_sha256=$recoveryLocalApprovalHash",
        "loader_sha256=$recoveryLoaderHash",
        "rollback_evidence_sha256=$recoveryRollbackEvidenceHash",
        "accepts_rollback_evidence_json=false",
        "accepts_artifact_bytes=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryRollbackEvidenceReferenceHash = Get-TextSha256 -Text $recoveryRollbackEvidenceCanonical
    $recoveryRollbackEvidenceCommand = "agent recovery.rollback_evidence_diagnostic $recoveryRollbackEvidenceReferenceHash $recoveryIdentityEventId $recoveryTrustEventId $recoveryVmTestEventId $recoveryLocalApprovalEventId $recoveryLoaderEventId $recoveryIdentityReferenceHash $recoveryTrustReferenceHash $recoveryVmTestReferenceHash $recoveryLocalApprovalReferenceHash $recoveryLoaderReferenceHash $recoveryArtifactHash $recoveryTrustHash $recoveryVmTestHash $recoveryLocalApprovalHash $recoveryLoaderHash $recoveryRollbackEvidenceHash"

    Send-AgentCommand -Command $recoveryRollbackEvidenceCommand -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_evidence_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_evidence_diag_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_artifact_rollback_evidence_reference_valid_but_lifeline_protocol_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "loader_event"; Needle = "`"retained_recovery_artifact_loader_event_id`": `"$recoveryLoaderEventId`"" },
        @{ Suffix = "rollback_ref_hash"; Needle = "`"rollback_evidence_reference_hash`": `"sha256:$recoveryRollbackEvidenceReferenceHash`"" },
        @{ Suffix = "loader_ref_hash"; Needle = "`"loader_reference_hash`": `"sha256:$recoveryLoaderReferenceHash`"" },
        @{ Suffix = "rollback_hash"; Needle = "`"rollback_evidence_hash`": `"sha256:$recoveryRollbackEvidenceHash`"" },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRollbackEvidenceResponse = Get-LastAgentResponseJson -Method "recovery.rollback_evidence_diagnostic"
    $recoveryRollbackEvidenceEventId = [string]$recoveryRollbackEvidenceResponse.body.result.retained_recovery_artifact_rollback_evidence_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_rollback_evidence_retained_reference_event_id_captured" -Value $recoveryRollbackEvidenceEventId

    Send-AgentCommand -Command "agent recovery.rollback_evidence_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_evidence_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_evidence_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_artifact_rollback_evidence_diagnostic_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_rollback_evidence_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "absent_reference"' },
        @{ Suffix = "valid_case"; Needle = '"case": "accepted_current_boot_rollback_evidence_still_denied"' },
        @{ Suffix = "stale_case"; Needle = '"case": "stale_previous_boot_reference"' },
        @{ Suffix = "loader_not_current_case"; Needle = '"case": "retained_loader_event_not_current_boot"' },
        @{ Suffix = "loader_missing_case"; Needle = '"case": "retained_loader_missing"' },
        @{ Suffix = "loader_schema_case"; Needle = '"case": "retained_loader_wrong_schema_or_variant"' },
        @{ Suffix = "substituted_case"; Needle = '"case": "substituted_loader_reference_record"' },
        @{ Suffix = "mismatch_case"; Needle = '"case": "rollback_evidence_reference_hash_mismatch"' },
        @{ Suffix = "chain_mismatch_case"; Needle = '"case": "retained_chain_mismatch"' },
        @{ Suffix = "valid_reason"; Needle = '"actual_reason": "recovery_artifact_rollback_evidence_reference_valid_but_lifeline_protocol_missing"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_request_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_request_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_request_diag_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_request_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_request_json"; Needle = '"accepts_lifeline_request_json": false' },
        @{ Suffix = "no_loader_descriptor"; Needle = '"accepts_loader_descriptor": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "status"; Needle = '"validation_status": "missing"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_lifeline_request_reference_absent"' },
        @{ Suffix = "retained_missing"; Needle = '"reason": "no_valid_recovery_lifeline_request_reference_retained"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLifelineRequestCanonical = @(
        "canonicalization=raios.recovery_lifeline_request.canonical.v0",
        "schema=raios.recovery_lifeline_request.v0",
        "requested_capability=cap.recovery.load_artifact",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline",
        "scope=current_boot",
        "retained_recovery_artifact_identity_event_id=$recoveryIdentityEventId",
        "retained_recovery_artifact_trust_event_id=$recoveryTrustEventId",
        "retained_recovery_artifact_vm_test_event_id=$recoveryVmTestEventId",
        "retained_recovery_artifact_local_approval_event_id=$recoveryLocalApprovalEventId",
        "retained_recovery_artifact_loader_event_id=$recoveryLoaderEventId",
        "retained_recovery_artifact_rollback_evidence_event_id=$recoveryRollbackEvidenceEventId",
        "identity_reference_sha256=$recoveryIdentityReferenceHash",
        "trust_reference_sha256=$recoveryTrustReferenceHash",
        "vm_test_reference_sha256=$recoveryVmTestReferenceHash",
        "local_approval_reference_sha256=$recoveryLocalApprovalReferenceHash",
        "loader_reference_sha256=$recoveryLoaderReferenceHash",
        "rollback_evidence_reference_sha256=$recoveryRollbackEvidenceReferenceHash",
        "artifact_sha256=$recoveryArtifactHash",
        "trust_sha256=$recoveryTrustHash",
        "vm_test_sha256=$recoveryVmTestHash",
        "local_approval_sha256=$recoveryLocalApprovalHash",
        "loader_sha256=$recoveryLoaderHash",
        "rollback_evidence_sha256=$recoveryRollbackEvidenceHash",
        "accepts_lifeline_request_json=false",
        "accepts_loader_descriptor=false",
        "accepts_artifact_bytes=false",
        "loads_recovery_loader=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryLifelineRequestReferenceHash = Get-TextSha256 -Text $recoveryLifelineRequestCanonical
    $recoveryLifelineRequestCommand = "agent recovery.lifeline_request_diagnostic $recoveryLifelineRequestReferenceHash $recoveryIdentityEventId $recoveryTrustEventId $recoveryVmTestEventId $recoveryLocalApprovalEventId $recoveryLoaderEventId $recoveryRollbackEvidenceEventId $recoveryIdentityReferenceHash $recoveryTrustReferenceHash $recoveryVmTestReferenceHash $recoveryLocalApprovalReferenceHash $recoveryLoaderReferenceHash $recoveryRollbackEvidenceReferenceHash $recoveryArtifactHash $recoveryTrustHash $recoveryVmTestHash $recoveryLocalApprovalHash $recoveryLoaderHash $recoveryRollbackEvidenceHash"

    Send-AgentCommand -Command $recoveryLifelineRequestCommand -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_request_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_request_diag_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"validation_status": "valid_hash_reference_load_still_denied"' },
        @{ Suffix = "reason"; Needle = '"validation_reason": "recovery_lifeline_request_reference_valid_but_lifeline_protocol_missing"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_load_still_denied"' },
        @{ Suffix = "retained_event_id"; Needle = '"event_id": "event.current_boot.' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_matches"; Needle = '"matches_current_reference": true' },
        @{ Suffix = "rollback_event"; Needle = "`"retained_recovery_artifact_rollback_evidence_event_id`": `"$recoveryRollbackEvidenceEventId`"" },
        @{ Suffix = "request_ref_hash"; Needle = "`"lifeline_request_reference_hash`": `"sha256:$recoveryLifelineRequestReferenceHash`"" },
        @{ Suffix = "rollback_ref_hash"; Needle = "`"rollback_evidence_reference_hash`": `"sha256:$recoveryRollbackEvidenceReferenceHash`"" },
        @{ Suffix = "rollback_hash"; Needle = "`"rollback_evidence_hash`": `"sha256:$recoveryRollbackEvidenceHash`"" },
        @{ Suffix = "no_loader_load"; Needle = '"loads_recovery_loader": false' },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "no_durable"; Needle = '"creates_durable_records": false' },
        @{ Suffix = "no_install"; Needle = '"installs_rollback_plan": false' },
        @{ Suffix = "no_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLifelineRequestResponse = Get-LastAgentResponseJson -Method "recovery.lifeline_request_diagnostic"
    $recoveryLifelineRequestEventId = [string]$recoveryLifelineRequestResponse.body.result.retained_recovery_lifeline_request_reference.event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_lifeline_request_retained_reference_event_id_captured" -Value $recoveryLifelineRequestEventId

    Send-AgentCommand -Command "agent recovery.lifeline_request_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_request_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_request_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_request_diagnostic_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_request_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 11' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "absent_reference"' },
        @{ Suffix = "valid_case"; Needle = '"case": "accepted_current_boot_lifeline_request_still_denied"' },
        @{ Suffix = "stale_case"; Needle = '"case": "stale_previous_boot_reference"' },
        @{ Suffix = "identity_not_current_case"; Needle = '"case": "retained_identity_event_not_current_boot"' },
        @{ Suffix = "rollback_not_current_case"; Needle = '"case": "retained_rollback_evidence_event_not_current_boot"' },
        @{ Suffix = "identity_missing_case"; Needle = '"case": "retained_identity_missing"' },
        @{ Suffix = "rollback_schema_case"; Needle = '"case": "retained_rollback_evidence_wrong_schema_or_variant"' },
        @{ Suffix = "substituted_case"; Needle = '"case": "substituted_rollback_evidence_reference_record"' },
        @{ Suffix = "request_mismatch_case"; Needle = '"case": "lifeline_request_reference_hash_mismatch"' },
        @{ Suffix = "rollback_mismatch_case"; Needle = '"case": "rollback_evidence_reference_hash_mismatch"' },
        @{ Suffix = "chain_mismatch_case"; Needle = '"case": "retained_chain_mismatch"' },
        @{ Suffix = "valid_reason"; Needle = '"actual_reason": "recovery_lifeline_request_reference_valid_but_lifeline_protocol_missing"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_protocol_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_protocol_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_protocol_diag_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_protocol_state.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_protocol_state"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_protocol_state_missing"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_protocol_state_records": false' },
        @{ Suffix = "no_request_json"; Needle = '"accepts_lifeline_request_json": false' },
        @{ Suffix = "no_loader_descriptor"; Needle = '"accepts_loader_descriptor": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_openai_shortcut"; Needle = '"uses_direct_openai_recovery_path": false' },
        @{ Suffix = "provider_shortcut_false"; Needle = '"provider_shortcut_used": false' },
        @{ Suffix = "request_valid"; Needle = '"request_chain_valid": true' },
        @{ Suffix = "can_report_gaps"; Needle = '"can_report_protocol_gaps": true' },
        @{ Suffix = "request_event"; Needle = "`"event_id`": `"$recoveryLifelineRequestEventId`"" },
        @{ Suffix = "identity_event"; Needle = "`"retained_recovery_artifact_identity_event_id`": `"$recoveryIdentityEventId`"" },
        @{ Suffix = "trust_event"; Needle = "`"retained_recovery_artifact_trust_event_id`": `"$recoveryTrustEventId`"" },
        @{ Suffix = "vm_event"; Needle = "`"retained_recovery_artifact_vm_test_event_id`": `"$recoveryVmTestEventId`"" },
        @{ Suffix = "approval_event"; Needle = "`"retained_recovery_artifact_local_approval_event_id`": `"$recoveryLocalApprovalEventId`"" },
        @{ Suffix = "loader_event"; Needle = "`"retained_recovery_artifact_loader_event_id`": `"$recoveryLoaderEventId`"" },
        @{ Suffix = "rollback_event"; Needle = "`"retained_recovery_artifact_rollback_evidence_event_id`": `"$recoveryRollbackEvidenceEventId`"" },
        @{ Suffix = "request_hash"; Needle = "`"lifeline_request_reference_hash`": `"sha256:$recoveryLifelineRequestReferenceHash`"" },
        @{ Suffix = "protocol_state_missing"; Needle = '"reason": "recovery_lifeline_protocol_state_missing"' },
        @{ Suffix = "command_vocab_missing"; Needle = '"reason": "recovery_lifeline_command_vocabulary_missing"' },
        @{ Suffix = "isolation_missing"; Needle = '"reason": "recovery_loader_runtime_isolation_missing"' },
        @{ Suffix = "rollback_engine_missing"; Needle = '"reason": "recovery_rollback_transaction_engine_missing"' },
        @{ Suffix = "durable_missing"; Needle = '"reason": "durable_audit_rollback_persistence_missing"' },
        @{ Suffix = "memory_provenance_missing"; Needle = '"reason": "recovery_memory_provenance_missing"' },
        @{ Suffix = "no_loader_load"; Needle = '"loads_recovery_loader": false' },
        @{ Suffix = "no_artifact_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "no_durable"; Needle = '"creates_durable_records": false' },
        @{ Suffix = "no_install"; Needle = '"installs_rollback_plan": false' },
        @{ Suffix = "no_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    $recoveryLifelineProtocolResponse = Get-LastAgentResponseJson -Method "recovery.lifeline_protocol_diagnostic"
    $recoveryLifelineProtocolRequestEventId = [string]$recoveryLifelineProtocolResponse.body.result.retained_recovery_lifeline_request.event_id
    $recoveryLifelineProtocolRequestMatches = $recoveryLifelineProtocolRequestEventId -eq $recoveryLifelineRequestEventId
    Add-Predicate -Name "protocol:recovery_lifeline_protocol_request_event_id_matches_retained" -Expected $recoveryLifelineRequestEventId -Passed $recoveryLifelineProtocolRequestMatches -Actual $recoveryLifelineProtocolRequestEventId
    if (-not $recoveryLifelineProtocolRequestMatches) {
        throw "Expected recovery lifeline protocol request event id $recoveryLifelineRequestEventId, got $recoveryLifelineProtocolRequestEventId"
    }

    Send-AgentCommand -Command "agent recovery.lifeline_protocol_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_protocol_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_protocol_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_protocol_state_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_protocol_state_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 15' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "missing_case"; Needle = '"case": "missing_lifeline_request_event_id"' },
        @{ Suffix = "stale_case"; Needle = '"case": "stale_dropped_lifeline_request_event_id"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_lifeline_request_event_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "wrong_schema_lifeline_request_event_id"' },
        @{ Suffix = "substituted_case"; Needle = '"case": "substituted_lifeline_request_record"' },
        @{ Suffix = "request_hash_case"; Needle = '"case": "lifeline_request_reference_hash_mismatch"' },
        @{ Suffix = "identity_event_case"; Needle = '"case": "retained_identity_event_id_mismatch"' },
        @{ Suffix = "rollback_hash_case"; Needle = '"case": "rollback_evidence_reference_hash_mismatch"' },
        @{ Suffix = "direct_openai_case"; Needle = '"case": "direct_openai_recovery_shortcut_rejected"' },
        @{ Suffix = "protocol_state_case"; Needle = '"case": "accepted_current_boot_request_protocol_state_missing"' },
        @{ Suffix = "command_vocab_case"; Needle = '"case": "command_vocabulary_missing_after_protocol_state"' },
        @{ Suffix = "isolation_case"; Needle = '"case": "loader_runtime_isolation_missing_after_command_vocabulary"' },
        @{ Suffix = "rollback_engine_case"; Needle = '"case": "rollback_transaction_engine_missing_after_isolation"' },
        @{ Suffix = "durable_case"; Needle = '"case": "durable_audit_rollback_persistence_missing_after_engine"' },
        @{ Suffix = "memory_provenance_case"; Needle = '"case": "recovery_memory_provenance_missing_after_persistence"' },
        @{ Suffix = "valid_reason"; Needle = '"actual_reason": "recovery_lifeline_protocol_state_missing"' },
        @{ Suffix = "no_loader_load"; Needle = '"loads_recovery_loader": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.load_binding" -ExpectedMarker "RAIOS_AGENT_END recovery.load_binding"
    $recoveryBindingResponse = Get-LastAgentResponseJson -Method "recovery.load_binding"
    Assert-LogContains -Name "protocol:recovery_binding_schema" -Needle '"schema": "raios.recovery_artifact_load_binding.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_status" -Needle '"status": "denied_missing_recovery_binding"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_no_records" -Needle '"creates_retained_recovery_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_load_capability" -Needle '"requested_capability": "cap.recovery.load_artifact"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_read_capability" -Needle '"read_capability": "cap.recovery.load_artifact.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_recovery_capability" -Needle '"recovery_only_capability_used": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_normal_capability_false" -Needle '"normal_module_capability_used": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_separate_from_module" -Needle '"separate_from": "cap.module.load_ephemeral"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_identity_id_required" -Needle '"recovery_artifact_identity_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_trust_id_required" -Needle '"recovery_artifact_trust_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_vm_test_id_required" -Needle '"recovery_vm_test_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_approval_id_required" -Needle '"recovery_local_approval_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_loader_id_required" -Needle '"recovery_loader_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_rollback_id_required" -Needle '"recovery_rollback_evidence_event_id"' -TimeoutSeconds 1
    $recoveryBindingIdentityEventId = [string]$recoveryBindingResponse.body.result.required_retained_evidence.recovery_artifact_identity_event_id.event_id
    $recoveryBindingIdentityEventIdMatches = $recoveryBindingIdentityEventId -eq $recoveryIdentityEventId
    Add-Predicate -Name "protocol:recovery_binding_identity_event_id_matches_retained" -Expected $recoveryIdentityEventId -Passed $recoveryBindingIdentityEventIdMatches -Actual $recoveryBindingIdentityEventId
    if (-not $recoveryBindingIdentityEventIdMatches) {
        throw "Expected recovery binding identity event id $recoveryIdentityEventId, got $recoveryBindingIdentityEventId"
    }
    $recoveryBindingTrustEventId = [string]$recoveryBindingResponse.body.result.required_retained_evidence.recovery_artifact_trust_event_id.event_id
    $recoveryBindingTrustEventIdMatches = $recoveryBindingTrustEventId -eq $recoveryTrustEventId
    Add-Predicate -Name "protocol:recovery_binding_trust_event_id_matches_retained" -Expected $recoveryTrustEventId -Passed $recoveryBindingTrustEventIdMatches -Actual $recoveryBindingTrustEventId
    if (-not $recoveryBindingTrustEventIdMatches) {
        throw "Expected recovery binding trust event id $recoveryTrustEventId, got $recoveryBindingTrustEventId"
    }
    $recoveryBindingVmTestEventId = [string]$recoveryBindingResponse.body.result.required_retained_evidence.recovery_vm_test_event_id.event_id
    $recoveryBindingVmTestEventIdMatches = $recoveryBindingVmTestEventId -eq $recoveryVmTestEventId
    Add-Predicate -Name "protocol:recovery_binding_vm_test_event_id_matches_retained" -Expected $recoveryVmTestEventId -Passed $recoveryBindingVmTestEventIdMatches -Actual $recoveryBindingVmTestEventId
    if (-not $recoveryBindingVmTestEventIdMatches) {
        throw "Expected recovery binding VM-test event id $recoveryVmTestEventId, got $recoveryBindingVmTestEventId"
    }
    $recoveryBindingLocalApprovalEventId = [string]$recoveryBindingResponse.body.result.required_retained_evidence.recovery_local_approval_event_id.event_id
    $recoveryBindingLocalApprovalEventIdMatches = $recoveryBindingLocalApprovalEventId -eq $recoveryLocalApprovalEventId
    Add-Predicate -Name "protocol:recovery_binding_local_approval_event_id_matches_retained" -Expected $recoveryLocalApprovalEventId -Passed $recoveryBindingLocalApprovalEventIdMatches -Actual $recoveryBindingLocalApprovalEventId
    if (-not $recoveryBindingLocalApprovalEventIdMatches) {
        throw "Expected recovery binding local-approval event id $recoveryLocalApprovalEventId, got $recoveryBindingLocalApprovalEventId"
    }
    $recoveryBindingLoaderEventId = [string]$recoveryBindingResponse.body.result.required_retained_evidence.recovery_loader_event_id.event_id
    $recoveryBindingLoaderEventIdMatches = $recoveryBindingLoaderEventId -eq $recoveryLoaderEventId
    Add-Predicate -Name "protocol:recovery_binding_loader_event_id_matches_retained" -Expected $recoveryLoaderEventId -Passed $recoveryBindingLoaderEventIdMatches -Actual $recoveryBindingLoaderEventId
    if (-not $recoveryBindingLoaderEventIdMatches) {
        throw "Expected recovery binding loader event id $recoveryLoaderEventId, got $recoveryBindingLoaderEventId"
    }
    $recoveryBindingRollbackEvidenceEventId = [string]$recoveryBindingResponse.body.result.required_retained_evidence.recovery_rollback_evidence_event_id.event_id
    $recoveryBindingRollbackEvidenceEventIdMatches = $recoveryBindingRollbackEvidenceEventId -eq $recoveryRollbackEvidenceEventId
    Add-Predicate -Name "protocol:recovery_binding_rollback_evidence_event_id_matches_retained" -Expected $recoveryRollbackEvidenceEventId -Passed $recoveryBindingRollbackEvidenceEventIdMatches -Actual $recoveryBindingRollbackEvidenceEventId
    if (-not $recoveryBindingRollbackEvidenceEventIdMatches) {
        throw "Expected recovery binding rollback-evidence event id $recoveryRollbackEvidenceEventId, got $recoveryBindingRollbackEvidenceEventId"
    }
    Assert-LogContains -Name "protocol:recovery_binding_identity_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_trust_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_vm_test_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_local_approval_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_loader_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_rollback_evidence_retained_status" -Needle '"status": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_identity_retained_reason" -Needle '"reason": "retained_recovery_artifact_identity_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_trust_retained_reason" -Needle '"reason": "retained_recovery_artifact_trust_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_vm_test_retained_reason" -Needle '"reason": "retained_recovery_artifact_vm_test_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_local_approval_retained_reason" -Needle '"reason": "retained_recovery_artifact_local_approval_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_loader_retained_reason" -Needle '"reason": "retained_recovery_artifact_loader_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_rollback_evidence_retained_reason" -Needle '"reason": "retained_recovery_artifact_rollback_evidence_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_identity_hash" -Needle "`"identity_reference_hash`": `"sha256:$recoveryIdentityReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_trust_hash" -Needle "`"trust_reference_hash`": `"sha256:$recoveryTrustReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_vm_test_hash" -Needle "`"vm_test_reference_hash`": `"sha256:$recoveryVmTestReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_local_approval_hash" -Needle "`"local_approval_reference_hash`": `"sha256:$recoveryLocalApprovalReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_loader_hash" -Needle "`"loader_reference_hash`": `"sha256:$recoveryLoaderReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_rollback_evidence_hash" -Needle "`"rollback_evidence_reference_hash`": `"sha256:$recoveryRollbackEvidenceReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_artifact_hash" -Needle "`"artifact_hash`": `"sha256:$recoveryArtifactHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_trust_material_hash" -Needle "`"trust_hash`": `"sha256:$recoveryTrustHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_vm_test_material_hash" -Needle "`"vm_test_hash`": `"sha256:$recoveryVmTestHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_local_approval_material_hash" -Needle "`"local_approval_hash`": `"sha256:$recoveryLocalApprovalHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_loader_material_hash" -Needle "`"loader_hash`": `"sha256:$recoveryLoaderHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_rollback_evidence_material_hash" -Needle "`"rollback_evidence_hash`": `"sha256:$recoveryRollbackEvidenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_boundary_lifeline_missing" -Needle '"reason": "recovery_lifeline_protocol_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_module_intent_rejected" -Needle '"module_append_intent_used": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_module_payload_not_authority" -Needle '"module_append_payload_hash_used_as_authority": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_module_writer_rejected" -Needle '"module_writer_facts_used": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_module_slot_rejected" -Needle '"module_service_slot_used": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_payload_non_authority" -Needle '"non_authority_input_only": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_payload_authority_false" -Needle '"append_payload_hash_authority": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_no_beyond_denial" -Needle '"can_move_beyond_denial": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_no_recovery_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_no_normal_load" -Needle '"loads_normal_module": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_no_durable_records" -Needle '"creates_durable_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_no_rollback_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_service_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent recovery.load_binding_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.load_binding_selftest"
    Assert-LogContains -Name "protocol:recovery_binding_selftest_schema" -Needle '"schema": "raios.recovery_artifact_load_binding_selftest.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_local_only" -Needle '"classification": "local_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_no_mutation" -Needle '"mutates_global_event_log": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_no_records" -Needle '"creates_retained_recovery_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_no_durable" -Needle '"creates_durable_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_no_install" -Needle '"installs_rollback_plan": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_no_recovery_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_no_normal_load" -Needle '"loads_normal_module": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_inventory_none" -Needle '"service_inventory_change": "none"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_count" -Needle '"case_count": 14' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_passed" -Needle '"passed": true' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_identity" -Needle '"case": "missing_recovery_artifact_identity_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_previous_identity" -Needle '"case": "previous_boot_recovery_artifact_identity_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_wrong_identity_schema" -Needle '"case": "wrong_schema_recovery_artifact_identity_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_trust" -Needle '"case": "missing_recovery_artifact_trust_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_vm_test" -Needle '"case": "missing_recovery_vm_test_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_approval" -Needle '"case": "missing_recovery_local_approval_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_loader" -Needle '"case": "missing_recovery_loader_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_missing_rollback" -Needle '"case": "missing_recovery_rollback_evidence_event_id"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_module_capability" -Needle '"case": "module_load_ephemeral_capability_substituted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_module_intent" -Needle '"case": "normal_module_append_intent_substituted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_payload_authority" -Needle '"case": "append_payload_hash_claimed_as_authority"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_writer" -Needle '"case": "normal_module_writer_facts_substituted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_service_slot" -Needle '"case": "normal_module_service_slot_substituted"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_available_denied" -Needle '"case": "available_recovery_binding_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_module_capability_reason" -Needle '"actual_reason": "recovery_load_requires_cap_recovery_load_artifact"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_module_intent_reason" -Needle '"actual_reason": "normal_module_append_intent_not_recovery_authority"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_payload_reason" -Needle '"actual_reason": "append_payload_hash_not_recovery_authority"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_writer_reason" -Needle '"actual_reason": "normal_module_writer_facts_not_recovery_authority"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_service_slot_reason" -Needle '"actual_reason": "normal_module_service_slot_not_recovery_authority"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_available_reason" -Needle '"actual_reason": "recovery_lifeline_protocol_missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_can_move_false" -Needle '"can_move_beyond_denial": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_load_attempted_false" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_module_cap_not_accepted" -Needle '"normal_module_capability_accepted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_payload_authority_false" -Needle '"append_payload_hash_authority": false' -TimeoutSeconds 1

    Send-AgentCommand -Command "agent audit.events 128" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events"
    Assert-LogContains -Name "protocol:module_manifest_audit_source" -Needle '"source_method": "module.manifest_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_kind" -Needle '"kind": "module.manifest_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_manifest_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_ref_hash" -Needle "`"manifest_reference_hash`": `"sha256:$moduleManifestReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_hash" -Needle "`"manifest_hash`": `"sha256:$moduleGrantManifestHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_manifest_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_source" -Needle '"source_method": "module.artifact_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_kind" -Needle '"kind": "module.artifact_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_candidate_artifact_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_ref_hash" -Needle "`"artifact_reference_hash`": `"sha256:$moduleArtifactReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_hash" -Needle "`"artifact_hash`": `"sha256:$moduleGrantArtifactHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_artifact_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_source" -Needle '"source_method": "module.vm_report_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_kind" -Needle '"kind": "module.vm_test_report_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_vm_test_report_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_ref_hash" -Needle "`"vm_test_report_reference_hash`": `"sha256:$moduleVmReportReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_hash" -Needle "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_vm_report_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_source" -Needle '"source_method": "module.attestation_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_kind" -Needle '"kind": "module.local_attestation_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_local_attestation_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_ref_hash" -Needle "`"local_attestation_reference_hash`": `"sha256:$moduleAttestationReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_hash" -Needle "`"local_attestation_hash`": `"sha256:$moduleGrantAttestationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_attestation_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_source" -Needle '"source_method": "module.approval_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_kind" -Needle '"kind": "module.local_approval_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_binding_schema" -Needle '"bindings": {"schema": "raios.module_local_approval_reference.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_ref_hash" -Needle "`"local_approval_reference_hash`": `"sha256:$moduleApprovalReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_hash" -Needle "`"local_approval_hash`": `"sha256:$moduleAuditLocalApprovalHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_approval_audit_no_load" -Needle '"load_attempted": false' -TimeoutSeconds 1
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
    Assert-LogContains -Name "protocol:module_load_audit_evidence_attestation_checked" -Needle '"local_attestation_reference_checked"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_evidence_approval_checked" -Needle '"local_approval_reference_checked"' -TimeoutSeconds 1
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
    Assert-LogContains -Name "protocol:module_load_audit_retained_vm_report_state" -Needle '"vm_test_report": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_vm_report_binding" -Needle '"retained_vm_test_report_reference": {"state": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_vm_report_reason" -Needle '"reason": "retained_vm_test_report_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_attestation_state" -Needle '"local_attestation": "retained_hash_reference_only"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_attestation_binding" -Needle '"retained_local_attestation_reference": {"state": "present"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_attestation_reason" -Needle '"reason": "retained_local_attestation_reference_not_authorizing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_attestation_hash" -Needle "`"local_attestation_reference_hash`": `"sha256:$moduleAttestationReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_vm_report_ref_hash" -Needle "`"vm_test_report_reference_hash`": `"sha256:$moduleVmReportReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:module_load_audit_retained_vm_report_hash" -Needle "`"vm_test_report_hash`": `"sha256:$moduleGrantReportHash`"" -TimeoutSeconds 1
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
    Assert-LogContains -Name "protocol:recovery_identity_audit_source" -Needle '"source_method": "recovery.identity_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_identity_audit_kind" -Needle '"kind": "recovery.artifact_identity_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_identity_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_identity_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_artifact_identity.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_identity_audit_capability" -Needle '"requested_capability": "cap.recovery.load_artifact.read"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_identity_audit_hash" -Needle "`"identity_reference_hash`": `"sha256:$recoveryIdentityReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_identity_audit_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_trust_audit_source" -Needle '"source_method": "recovery.trust_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_trust_audit_kind" -Needle '"kind": "recovery.artifact_trust_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_trust_audit_outcome" -Needle '"outcome": "retained_hash_reference_load_still_denied"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_trust_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_artifact_trust.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_trust_audit_identity_event" -Needle "`"retained_recovery_artifact_identity_event_id`": `"$recoveryIdentityEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_trust_audit_hash" -Needle "`"trust_reference_hash`": `"sha256:$recoveryTrustReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_trust_audit_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_vm_test_audit_source" -Needle '"source_method": "recovery.vm_test_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_vm_test_audit_kind" -Needle '"kind": "recovery.artifact_vm_test_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_vm_test_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_artifact_vm_test.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_vm_test_audit_trust_event" -Needle "`"retained_recovery_artifact_trust_event_id`": `"$recoveryTrustEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_vm_test_audit_hash" -Needle "`"vm_test_reference_hash`": `"sha256:$recoveryVmTestReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_vm_test_audit_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_local_approval_audit_source" -Needle '"source_method": "recovery.local_approval_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_local_approval_audit_kind" -Needle '"kind": "recovery.artifact_local_approval_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_local_approval_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_artifact_local_approval.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_local_approval_audit_vm_event" -Needle "`"retained_recovery_artifact_vm_test_event_id`": `"$recoveryVmTestEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_local_approval_audit_hash" -Needle "`"local_approval_reference_hash`": `"sha256:$recoveryLocalApprovalReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_local_approval_audit_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_audit_source" -Needle '"source_method": "recovery.loader_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_audit_kind" -Needle '"kind": "recovery.artifact_loader_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_artifact_loader.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_audit_approval_event" -Needle "`"retained_recovery_artifact_local_approval_event_id`": `"$recoveryLocalApprovalEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_audit_hash" -Needle "`"loader_reference_hash`": `"sha256:$recoveryLoaderReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_audit_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_evidence_audit_source" -Needle '"source_method": "recovery.rollback_evidence_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_evidence_audit_kind" -Needle '"kind": "recovery.artifact_rollback_evidence_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_evidence_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_artifact_rollback_evidence.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_evidence_audit_loader_event" -Needle "`"retained_recovery_artifact_loader_event_id`": `"$recoveryLoaderEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_evidence_audit_hash" -Needle "`"rollback_evidence_reference_hash`": `"sha256:$recoveryRollbackEvidenceReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_evidence_audit_no_durable" -Needle '"creates_durable_records": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_request_audit_source" -Needle '"source_method": "recovery.lifeline_request_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_request_audit_kind" -Needle '"kind": "recovery.lifeline_request_reference.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_request_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_request.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_request_audit_rollback_event" -Needle "`"retained_recovery_artifact_rollback_evidence_event_id`": `"$recoveryRollbackEvidenceEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_request_audit_hash" -Needle "`"lifeline_request_reference_hash`": `"sha256:$recoveryLifelineRequestReferenceHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_request_audit_no_loader" -Needle '"loads_recovery_loader": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_request_audit_no_slot" -Needle '"allocates_service_slot": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_protocol_audit_source" -Needle '"source_method": "recovery.lifeline_protocol_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_protocol_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_protocol_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_source" -Needle '"source_method": "recovery.load_artifact"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_capability" -Needle '"requested_capability": "cap.recovery.load_artifact"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_risk" -Needle '"risk": "recovery_modify_ram"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_resource" -Needle '"resource": "recovery_lifeline"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_reason" -Needle '"reason": "missing_recovery_artifact_evidence"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_evidence_boundary" -Needle '"recovery_artifact_load_boundary_evaluated"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_evidence_identity" -Needle '"recovery_artifact_identity_required"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_evidence_normal_path" -Needle '"normal_module_load_path_not_used"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_artifact_load_denial_evidence.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_binding_status" -Needle '"status": "denied_missing_recovery_artifact_evidence"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_binding_normal_path" -Needle '"normal_module_load_path_used": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_binding_missing_identity" -Needle '"recovery_artifact_identity": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_binding_missing_rollback" -Needle '"recovery_rollback_evidence": "missing"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_audit_binding_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_audit_source" -Needle '"source_method": "recovery.load_binding"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_selftest_audit_source" -Needle '"source_method": "recovery.load_binding_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_binding_audit_capability" -Needle '"requested_capability": "cap.recovery.load_artifact.read"' -TimeoutSeconds 1

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
