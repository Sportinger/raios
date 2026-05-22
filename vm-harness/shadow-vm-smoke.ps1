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
            "agent recovery.lifeline_command_vocabulary",
            "agent recovery.lifeline_command_vocabulary_selftest",
            "agent recovery.loader_runtime_isolation",
            "agent recovery.loader_runtime_isolation_selftest",
            "agent recovery.rollback_transaction_engine",
            "agent recovery.rollback_transaction_engine_selftest",
            "agent recovery.durable_audit_rollback_persistence",
            "agent recovery.durable_audit_rollback_persistence_selftest",
            "agent recovery.memory_provenance",
            "agent recovery.memory_provenance_selftest",
            "agent recovery.lifeline_command_admission",
            "agent recovery.lifeline_command_admission_selftest",
            "agent recovery.lifeline_command_envelope_diagnostic",
            "agent recovery.lifeline_command_envelope_diagnostic_selftest",
            "agent recovery.lifeline_command_dispatch_diagnostic",
            "agent recovery.lifeline_command_dispatch_diagnostic_selftest",
            "agent recovery.lifeline_command_body_canonicalization_diagnostic",
            "agent recovery.lifeline_command_body_canonicalization_diagnostic_selftest",
            "agent recovery.lifeline_command_handler_binding_diagnostic",
            "agent recovery.lifeline_command_handler_binding_diagnostic_selftest",
            "agent recovery.lifeline_status_read_handler_diagnostic",
            "agent recovery.lifeline_status_read_handler_diagnostic_selftest",
            "agent recovery.rollback_preview_authorization_diagnostic",
            "agent recovery.rollback_preview_authorization_diagnostic_selftest",
            "agent recovery.rollback_apply_authorization_diagnostic",
            "agent recovery.rollback_apply_authorization_diagnostic_selftest",
            "agent recovery.disable_module_target_binding_diagnostic",
            "agent recovery.disable_module_target_binding_diagnostic_selftest",
            "agent recovery.restart_last_good_target_binding_diagnostic",
            "agent recovery.restart_last_good_target_binding_diagnostic_selftest",
            "agent recovery.load_artifact_by_hash_target_binding_diagnostic",
            "agent recovery.load_artifact_by_hash_target_binding_diagnostic_selftest",
            "agent recovery.memory_write_authority_diagnostic",
            "agent recovery.memory_write_authority_diagnostic_selftest",
            "agent recovery.load_binding",
            "agent recovery.load_binding_selftest",
            "module.load_recovery_artifact",
            "agent audit.events 256"
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

    Send-AgentCommand -Command "agent recovery.lifeline_command_vocabulary" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_vocabulary"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_vocab_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_vocabulary.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_protocol_state"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_protocol_state_missing"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_vocabulary_records": false' },
        @{ Suffix = "no_command_envelope"; Needle = '"accepts_lifeline_command_envelope": false' },
        @{ Suffix = "no_request_json"; Needle = '"accepts_lifeline_request_json": false' },
        @{ Suffix = "no_loader_descriptor"; Needle = '"accepts_loader_descriptor": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_openai_shortcut"; Needle = '"uses_direct_openai_recovery_path": false' },
        @{ Suffix = "request_valid"; Needle = '"request_chain_valid": true' },
        @{ Suffix = "vocab_exposed"; Needle = '"command_vocabulary_exposed": true' },
        @{ Suffix = "commands_defined"; Needle = '"state": "defined_non_executable"' },
        @{ Suffix = "command_count"; Needle = '"command_count": 6' },
        @{ Suffix = "envelope_schema"; Needle = '"argument_envelope_schema": "raios.recovery_lifeline_command_envelope.v0"' },
        @{ Suffix = "status_command"; Needle = '"id": "recovery.lifeline.status"' },
        @{ Suffix = "rollback_preview_command"; Needle = '"id": "recovery.lifeline.rollback_preview"' },
        @{ Suffix = "rollback_apply_command"; Needle = '"id": "recovery.lifeline.rollback_apply"' },
        @{ Suffix = "disable_module_command"; Needle = '"id": "recovery.lifeline.disable_module"' },
        @{ Suffix = "restart_last_good_command"; Needle = '"id": "recovery.lifeline.restart_last_good"' },
        @{ Suffix = "load_artifact_command"; Needle = '"id": "recovery.lifeline.load_artifact_by_hash"' },
        @{ Suffix = "rollback_cap"; Needle = '"required_capability": "cap.recovery.rollback"' },
        @{ Suffix = "module_disable_cap"; Needle = '"required_capability": "cap.recovery.module.disable"' },
        @{ Suffix = "service_restart_cap"; Needle = '"required_capability": "cap.recovery.service.restart"' },
        @{ Suffix = "load_cap"; Needle = '"required_capability": "cap.recovery.load_artifact"' },
        @{ Suffix = "request_event"; Needle = "`"event_id`": `"$recoveryLifelineRequestEventId`"" },
        @{ Suffix = "identity_event"; Needle = "`"retained_recovery_artifact_identity_event_id`": `"$recoveryIdentityEventId`"" },
        @{ Suffix = "rollback_event"; Needle = "`"retained_recovery_artifact_rollback_evidence_event_id`": `"$recoveryRollbackEvidenceEventId`"" },
        @{ Suffix = "request_hash"; Needle = "`"lifeline_request_reference_hash`": `"sha256:$recoveryLifelineRequestReferenceHash`"" },
        @{ Suffix = "execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_commands": false' },
        @{ Suffix = "no_loader_load"; Needle = '"loads_recovery_loader": false' },
        @{ Suffix = "no_artifact_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "no_durable"; Needle = '"creates_durable_records": false' },
        @{ Suffix = "no_install"; Needle = '"installs_rollback_plan": false' },
        @{ Suffix = "no_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    $recoveryLifelineCommandVocabularyResponse = Get-LastAgentResponseJson -Method "recovery.lifeline_command_vocabulary"
    $recoveryLifelineCommandVocabularyRequestEventId = [string]$recoveryLifelineCommandVocabularyResponse.body.result.retained_recovery_lifeline_request.event_id
    $recoveryLifelineCommandVocabularyRequestMatches = $recoveryLifelineCommandVocabularyRequestEventId -eq $recoveryLifelineRequestEventId
    Add-Predicate -Name "protocol:recovery_lifeline_command_vocab_request_event_id_matches_retained" -Expected $recoveryLifelineRequestEventId -Passed $recoveryLifelineCommandVocabularyRequestMatches -Actual $recoveryLifelineCommandVocabularyRequestEventId
    if (-not $recoveryLifelineCommandVocabularyRequestMatches) {
        throw "Expected recovery lifeline command vocabulary request event id $recoveryLifelineRequestEventId, got $recoveryLifelineCommandVocabularyRequestEventId"
    }

    Send-AgentCommand -Command "agent recovery.lifeline_command_vocabulary_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_vocabulary_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_vocab_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_vocabulary_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_vocabulary_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 16' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "missing_request_case"; Needle = '"case": "missing_lifeline_request_event_id"' },
        @{ Suffix = "stale_request_case"; Needle = '"case": "stale_dropped_lifeline_request_event_id"' },
        @{ Suffix = "previous_request_case"; Needle = '"case": "previous_boot_lifeline_request_event_id"' },
        @{ Suffix = "wrong_request_schema_case"; Needle = '"case": "wrong_schema_lifeline_request_event_id"' },
        @{ Suffix = "substituted_request_case"; Needle = '"case": "substituted_lifeline_request_record"' },
        @{ Suffix = "request_hash_case"; Needle = '"case": "lifeline_request_reference_hash_mismatch"' },
        @{ Suffix = "missing_protocol_state_case"; Needle = '"case": "protocol_state_missing_after_valid_request"' },
        @{ Suffix = "previous_protocol_state_case"; Needle = '"case": "previous_boot_lifeline_protocol_state"' },
        @{ Suffix = "wrong_protocol_state_schema_case"; Needle = '"case": "wrong_schema_lifeline_protocol_state"' },
        @{ Suffix = "substituted_protocol_state_case"; Needle = '"case": "substituted_lifeline_protocol_state"' },
        @{ Suffix = "direct_openai_case"; Needle = '"case": "direct_openai_recovery_shortcut_rejected"' },
        @{ Suffix = "isolation_missing_case"; Needle = '"case": "loader_runtime_isolation_missing"' },
        @{ Suffix = "rollback_engine_missing_case"; Needle = '"case": "rollback_transaction_engine_missing"' },
        @{ Suffix = "durable_missing_case"; Needle = '"case": "durable_audit_rollback_persistence_missing"' },
        @{ Suffix = "memory_provenance_case"; Needle = '"case": "recovery_memory_provenance_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_commands_still_non_executable"' },
        @{ Suffix = "non_executable_reason"; Needle = '"actual_reason": "recovery_lifeline_command_behavior_not_implemented"' },
        @{ Suffix = "execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.loader_runtime_isolation" -ExpectedMarker "RAIOS_AGENT_END recovery.loader_runtime_isolation"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_loader_runtime_isolation_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_loader_runtime_isolation.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_protocol_state"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_protocol_state_missing"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_loader_runtime_isolation_records": false' },
        @{ Suffix = "no_command_envelope"; Needle = '"accepts_lifeline_command_envelope": false' },
        @{ Suffix = "no_request_json"; Needle = '"accepts_lifeline_request_json": false' },
        @{ Suffix = "no_loader_descriptor"; Needle = '"accepts_loader_descriptor": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_openai_shortcut"; Needle = '"uses_direct_openai_recovery_path": false' },
        @{ Suffix = "request_valid"; Needle = '"request_chain_valid": true' },
        @{ Suffix = "vocab_exposed"; Needle = '"command_vocabulary_envelope_exposed": true' },
        @{ Suffix = "vocab_not_accepted"; Needle = '"command_vocabulary_accepted": false' },
        @{ Suffix = "requirements_exposed"; Needle = '"isolation_requirements_exposed": true' },
        @{ Suffix = "isolation_not_ready"; Needle = '"loader_runtime_isolation_ready": false' },
        @{ Suffix = "request_event"; Needle = "`"event_id`": `"$recoveryLifelineRequestEventId`"" },
        @{ Suffix = "identity_event"; Needle = "`"retained_recovery_artifact_identity_event_id`": `"$recoveryIdentityEventId`"" },
        @{ Suffix = "rollback_event"; Needle = "`"retained_recovery_artifact_rollback_evidence_event_id`": `"$recoveryRollbackEvidenceEventId`"" },
        @{ Suffix = "request_hash"; Needle = "`"lifeline_request_reference_hash`": `"sha256:$recoveryLifelineRequestReferenceHash`"" },
        @{ Suffix = "command_vocab_schema"; Needle = '"schema": "raios.recovery_lifeline_command_vocabulary.v0"' },
        @{ Suffix = "command_vocab_envelope"; Needle = '"state": "defined_read_only_envelope"' },
        @{ Suffix = "loader_boundary_schema"; Needle = '"loader_runtime_isolation_schema": "raios.recovery_loader_runtime_isolation.v0"' },
        @{ Suffix = "address_space_schema"; Needle = '"schema": "raios.recovery_loader_address_space_boundary.v0"' },
        @{ Suffix = "entrypoint_schema"; Needle = '"schema": "raios.recovery_loader_entrypoint_abi.v0"' },
        @{ Suffix = "memory_map_schema"; Needle = '"schema": "raios.recovery_loader_memory_map_constraints.v0"' },
        @{ Suffix = "import_table_schema"; Needle = '"schema": "raios.recovery_loader_capability_import_table.v0"' },
        @{ Suffix = "artifact_hash_schema"; Needle = '"schema": "raios.recovery_loader_artifact_hash_binding.v0"' },
        @{ Suffix = "provider_sep_schema"; Needle = '"schema": "raios.recovery_loader_provider_separation.v0"' },
        @{ Suffix = "normal_module_sep_schema"; Needle = '"schema": "raios.recovery_loader_normal_module_separation.v0"' },
        @{ Suffix = "address_space_missing"; Needle = '"reason": "recovery_loader_address_space_boundary_missing"' },
        @{ Suffix = "entrypoint_missing"; Needle = '"reason": "recovery_loader_entrypoint_abi_missing"' },
        @{ Suffix = "memory_map_missing"; Needle = '"reason": "recovery_loader_memory_map_constraints_missing"' },
        @{ Suffix = "import_table_missing"; Needle = '"reason": "recovery_loader_capability_import_table_missing"' },
        @{ Suffix = "artifact_hash_missing"; Needle = '"reason": "recovery_loader_artifact_hash_binding_missing"' },
        @{ Suffix = "provider_sep_missing"; Needle = '"reason": "recovery_loader_provider_separation_missing"' },
        @{ Suffix = "normal_module_sep_missing"; Needle = '"reason": "recovery_loader_normal_module_separation_missing"' },
        @{ Suffix = "rollback_engine_missing"; Needle = '"reason": "recovery_rollback_transaction_engine_missing"' },
        @{ Suffix = "durable_missing"; Needle = '"reason": "durable_audit_rollback_persistence_missing"' },
        @{ Suffix = "memory_provenance_missing"; Needle = '"reason": "recovery_memory_provenance_missing"' },
        @{ Suffix = "loader_execution_false"; Needle = '"loader_execution_enabled": false' },
        @{ Suffix = "dispatch_false"; Needle = '"recovery_lifeline_command_dispatch_enabled": false' },
        @{ Suffix = "normal_path_false"; Needle = '"normal_module_load_path_used": false' },
        @{ Suffix = "no_loader_load"; Needle = '"loads_recovery_loader": false' },
        @{ Suffix = "no_artifact_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "no_durable"; Needle = '"creates_durable_records": false' },
        @{ Suffix = "no_install"; Needle = '"installs_rollback_plan": false' },
        @{ Suffix = "no_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    $recoveryLoaderRuntimeIsolationResponse = Get-LastAgentResponseJson -Method "recovery.loader_runtime_isolation"
    $recoveryLoaderRuntimeIsolationRequestEventId = [string]$recoveryLoaderRuntimeIsolationResponse.body.result.retained_recovery_lifeline_request.event_id
    $recoveryLoaderRuntimeIsolationRequestMatches = $recoveryLoaderRuntimeIsolationRequestEventId -eq $recoveryLifelineRequestEventId
    Add-Predicate -Name "protocol:recovery_loader_runtime_isolation_request_event_id_matches_retained" -Expected $recoveryLifelineRequestEventId -Passed $recoveryLoaderRuntimeIsolationRequestMatches -Actual $recoveryLoaderRuntimeIsolationRequestEventId
    if (-not $recoveryLoaderRuntimeIsolationRequestMatches) {
        throw "Expected recovery loader runtime isolation request event id $recoveryLifelineRequestEventId, got $recoveryLoaderRuntimeIsolationRequestEventId"
    }

    Send-AgentCommand -Command "agent recovery.loader_runtime_isolation_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.loader_runtime_isolation_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_loader_runtime_isolation_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_loader_runtime_isolation_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_loader_runtime_isolation_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 27' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "missing_request_case"; Needle = '"case": "missing_lifeline_request_event_id"' },
        @{ Suffix = "stale_request_case"; Needle = '"case": "stale_dropped_lifeline_request_event_id"' },
        @{ Suffix = "previous_request_case"; Needle = '"case": "previous_boot_lifeline_request_event_id"' },
        @{ Suffix = "wrong_request_schema_case"; Needle = '"case": "wrong_schema_lifeline_request_event_id"' },
        @{ Suffix = "substituted_request_case"; Needle = '"case": "substituted_lifeline_request_record"' },
        @{ Suffix = "request_hash_case"; Needle = '"case": "lifeline_request_reference_hash_mismatch"' },
        @{ Suffix = "missing_protocol_state_case"; Needle = '"case": "protocol_state_missing_after_valid_request"' },
        @{ Suffix = "previous_protocol_state_case"; Needle = '"case": "previous_boot_lifeline_protocol_state"' },
        @{ Suffix = "wrong_protocol_state_schema_case"; Needle = '"case": "wrong_schema_lifeline_protocol_state"' },
        @{ Suffix = "substituted_protocol_state_case"; Needle = '"case": "substituted_lifeline_protocol_state"' },
        @{ Suffix = "missing_command_vocab_case"; Needle = '"case": "command_vocabulary_missing_after_protocol_state"' },
        @{ Suffix = "previous_command_vocab_case"; Needle = '"case": "previous_boot_lifeline_command_vocabulary"' },
        @{ Suffix = "wrong_command_vocab_case"; Needle = '"case": "wrong_schema_lifeline_command_vocabulary"' },
        @{ Suffix = "substituted_command_vocab_case"; Needle = '"case": "substituted_lifeline_command_vocabulary"' },
        @{ Suffix = "direct_openai_case"; Needle = '"case": "direct_openai_recovery_shortcut_rejected"' },
        @{ Suffix = "all_isolation_missing_case"; Needle = '"case": "loader_runtime_isolation_missing"' },
        @{ Suffix = "address_space_case"; Needle = '"case": "loader_address_space_boundary_missing"' },
        @{ Suffix = "entrypoint_case"; Needle = '"case": "loader_entrypoint_abi_missing"' },
        @{ Suffix = "memory_map_case"; Needle = '"case": "loader_memory_map_constraints_missing"' },
        @{ Suffix = "import_table_case"; Needle = '"case": "loader_capability_import_table_missing"' },
        @{ Suffix = "artifact_hash_case"; Needle = '"case": "loader_artifact_hash_binding_missing"' },
        @{ Suffix = "provider_sep_case"; Needle = '"case": "loader_provider_separation_missing"' },
        @{ Suffix = "normal_module_sep_case"; Needle = '"case": "loader_normal_module_separation_missing"' },
        @{ Suffix = "rollback_engine_case"; Needle = '"case": "rollback_transaction_engine_missing"' },
        @{ Suffix = "durable_case"; Needle = '"case": "durable_audit_rollback_persistence_missing"' },
        @{ Suffix = "memory_provenance_case"; Needle = '"case": "recovery_memory_provenance_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_loader_still_non_executable"' },
        @{ Suffix = "non_executable_reason"; Needle = '"actual_reason": "recovery_loader_runtime_behavior_not_implemented"' },
        @{ Suffix = "loader_execution_false"; Needle = '"loader_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.rollback_transaction_engine" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_transaction_engine"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_transaction_engine_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_rollback_transaction_engine.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_protocol_state"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_protocol_state_missing"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_rollback_transaction_engine_records": false' },
        @{ Suffix = "no_command_envelope"; Needle = '"accepts_lifeline_command_envelope": false' },
        @{ Suffix = "no_transaction_envelope"; Needle = '"accepts_rollback_transaction_envelope": false' },
        @{ Suffix = "no_plan_json"; Needle = '"accepts_rollback_plan_json": false' },
        @{ Suffix = "no_request_json"; Needle = '"accepts_lifeline_request_json": false' },
        @{ Suffix = "no_loader_descriptor"; Needle = '"accepts_loader_descriptor": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_openai_shortcut"; Needle = '"uses_direct_openai_recovery_path": false' },
        @{ Suffix = "request_valid"; Needle = '"request_chain_valid": true' },
        @{ Suffix = "vocab_exposed"; Needle = '"command_vocabulary_envelope_exposed": true' },
        @{ Suffix = "vocab_not_accepted"; Needle = '"command_vocabulary_accepted": false' },
        @{ Suffix = "loader_boundary_exposed"; Needle = '"loader_runtime_isolation_boundary_exposed": true' },
        @{ Suffix = "loader_not_accepted"; Needle = '"loader_runtime_isolation_accepted": false' },
        @{ Suffix = "requirements_exposed"; Needle = '"transaction_requirements_exposed": true' },
        @{ Suffix = "engine_not_ready"; Needle = '"rollback_transaction_engine_ready": false' },
        @{ Suffix = "request_event"; Needle = "`"event_id`": `"$recoveryLifelineRequestEventId`"" },
        @{ Suffix = "identity_event"; Needle = "`"retained_recovery_artifact_identity_event_id`": `"$recoveryIdentityEventId`"" },
        @{ Suffix = "rollback_event"; Needle = "`"retained_recovery_artifact_rollback_evidence_event_id`": `"$recoveryRollbackEvidenceEventId`"" },
        @{ Suffix = "request_hash"; Needle = "`"lifeline_request_reference_hash`": `"sha256:$recoveryLifelineRequestReferenceHash`"" },
        @{ Suffix = "command_vocab_schema"; Needle = '"schema": "raios.recovery_lifeline_command_vocabulary.v0"' },
        @{ Suffix = "command_vocab_envelope"; Needle = '"state": "defined_read_only_envelope"' },
        @{ Suffix = "loader_boundary_schema"; Needle = '"schema": "raios.recovery_loader_runtime_isolation.v0"' },
        @{ Suffix = "loader_state"; Needle = '"state": "defined_non_executable"' },
        @{ Suffix = "target_selection_schema"; Needle = '"schema": "raios.recovery_rollback_target_selection.v0"' },
        @{ Suffix = "transaction_provenance_schema"; Needle = '"schema": "raios.recovery_rollback_transaction_provenance.v0"' },
        @{ Suffix = "last_good_schema"; Needle = '"schema": "raios.recovery_rollback_last_good_binding.v0"' },
        @{ Suffix = "disabled_set_schema"; Needle = '"schema": "raios.recovery_rollback_disabled_module_set_binding.v0"' },
        @{ Suffix = "artifact_hash_schema"; Needle = '"schema": "raios.recovery_rollback_artifact_hash_binding.v0"' },
        @{ Suffix = "replay_schema"; Needle = '"schema": "raios.recovery_rollback_replay_preconditions.v0"' },
        @{ Suffix = "capability_import_schema"; Needle = '"schema": "raios.recovery_rollback_recovery_capability_import.v0"' },
        @{ Suffix = "atomic_semantics_schema"; Needle = '"schema": "raios.recovery_rollback_atomic_apply_abort_semantics.v0"' },
        @{ Suffix = "target_selection_missing"; Needle = '"reason": "recovery_rollback_target_selection_missing"' },
        @{ Suffix = "transaction_provenance_missing"; Needle = '"reason": "recovery_rollback_transaction_id_provenance_missing"' },
        @{ Suffix = "last_good_missing"; Needle = '"reason": "recovery_rollback_last_good_binding_missing"' },
        @{ Suffix = "disabled_set_missing"; Needle = '"reason": "recovery_rollback_disabled_module_set_binding_missing"' },
        @{ Suffix = "artifact_hash_missing"; Needle = '"reason": "recovery_rollback_artifact_hash_binding_missing"' },
        @{ Suffix = "replay_missing"; Needle = '"reason": "recovery_rollback_replay_preconditions_missing"' },
        @{ Suffix = "capability_import_missing"; Needle = '"reason": "recovery_rollback_recovery_capability_import_missing"' },
        @{ Suffix = "atomic_semantics_missing"; Needle = '"reason": "recovery_rollback_atomic_apply_abort_semantics_missing"' },
        @{ Suffix = "durable_schema"; Needle = '"schema": "raios.durable_audit_rollback_persistence.v0"' },
        @{ Suffix = "durable_missing"; Needle = '"reason": "durable_audit_rollback_persistence_missing"' },
        @{ Suffix = "memory_schema"; Needle = '"schema": "raios.recovery_memory_provenance.v0"' },
        @{ Suffix = "memory_missing"; Needle = '"reason": "recovery_memory_provenance_missing"' },
        @{ Suffix = "preview_false"; Needle = '"rollback_preview_enabled": false' },
        @{ Suffix = "apply_false"; Needle = '"rollback_apply_enabled": false' },
        @{ Suffix = "execute_preview_false"; Needle = '"executes_rollback_preview": false' },
        @{ Suffix = "execute_apply_false"; Needle = '"executes_rollback_apply": false' },
        @{ Suffix = "no_loader_load"; Needle = '"loads_recovery_loader": false' },
        @{ Suffix = "no_artifact_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "no_authority"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "no_durable"; Needle = '"creates_durable_records": false' },
        @{ Suffix = "no_install"; Needle = '"installs_rollback_plan": false' },
        @{ Suffix = "no_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    $recoveryRollbackTransactionEngineResponse = Get-LastAgentResponseJson -Method "recovery.rollback_transaction_engine"
    $recoveryRollbackTransactionEngineRequestEventId = [string]$recoveryRollbackTransactionEngineResponse.body.result.retained_recovery_lifeline_request.event_id
    $recoveryRollbackTransactionEngineRequestMatches = $recoveryRollbackTransactionEngineRequestEventId -eq $recoveryLifelineRequestEventId
    Add-Predicate -Name "protocol:recovery_rollback_transaction_engine_request_event_id_matches_retained" -Expected $recoveryLifelineRequestEventId -Passed $recoveryRollbackTransactionEngineRequestMatches -Actual $recoveryRollbackTransactionEngineRequestEventId
    if (-not $recoveryRollbackTransactionEngineRequestMatches) {
        throw "Expected recovery rollback transaction engine request event id $recoveryLifelineRequestEventId, got $recoveryRollbackTransactionEngineRequestEventId"
    }

    Send-AgentCommand -Command "agent recovery.rollback_transaction_engine_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_transaction_engine_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_transaction_engine_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_rollback_transaction_engine_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_rollback_transaction_engine_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 38' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "missing_request_case"; Needle = '"case": "missing_lifeline_request_event_id"' },
        @{ Suffix = "stale_request_case"; Needle = '"case": "stale_dropped_lifeline_request_event_id"' },
        @{ Suffix = "previous_request_case"; Needle = '"case": "previous_boot_lifeline_request_event_id"' },
        @{ Suffix = "wrong_request_schema_case"; Needle = '"case": "wrong_schema_lifeline_request_event_id"' },
        @{ Suffix = "substituted_request_case"; Needle = '"case": "substituted_lifeline_request_record"' },
        @{ Suffix = "request_hash_case"; Needle = '"case": "lifeline_request_reference_hash_mismatch"' },
        @{ Suffix = "missing_protocol_state_case"; Needle = '"case": "protocol_state_missing_after_valid_request"' },
        @{ Suffix = "previous_protocol_state_case"; Needle = '"case": "previous_boot_lifeline_protocol_state"' },
        @{ Suffix = "wrong_protocol_state_schema_case"; Needle = '"case": "wrong_schema_lifeline_protocol_state"' },
        @{ Suffix = "substituted_protocol_state_case"; Needle = '"case": "substituted_lifeline_protocol_state"' },
        @{ Suffix = "missing_command_vocab_case"; Needle = '"case": "command_vocabulary_missing_after_protocol_state"' },
        @{ Suffix = "previous_command_vocab_case"; Needle = '"case": "previous_boot_lifeline_command_vocabulary"' },
        @{ Suffix = "wrong_command_vocab_case"; Needle = '"case": "wrong_schema_lifeline_command_vocabulary"' },
        @{ Suffix = "substituted_command_vocab_case"; Needle = '"case": "substituted_lifeline_command_vocabulary"' },
        @{ Suffix = "direct_openai_case"; Needle = '"case": "direct_openai_recovery_shortcut_rejected"' },
        @{ Suffix = "isolation_missing_case"; Needle = '"case": "loader_runtime_isolation_missing_after_command_vocabulary"' },
        @{ Suffix = "previous_isolation_case"; Needle = '"case": "previous_boot_loader_runtime_isolation"' },
        @{ Suffix = "wrong_isolation_schema_case"; Needle = '"case": "wrong_schema_loader_runtime_isolation"' },
        @{ Suffix = "substituted_isolation_case"; Needle = '"case": "substituted_loader_runtime_isolation"' },
        @{ Suffix = "address_space_case"; Needle = '"case": "loader_address_space_boundary_missing"' },
        @{ Suffix = "entrypoint_case"; Needle = '"case": "loader_entrypoint_abi_missing"' },
        @{ Suffix = "memory_map_case"; Needle = '"case": "loader_memory_map_constraints_missing"' },
        @{ Suffix = "import_table_case"; Needle = '"case": "loader_capability_import_table_missing"' },
        @{ Suffix = "loader_artifact_hash_case"; Needle = '"case": "loader_artifact_hash_binding_missing"' },
        @{ Suffix = "provider_sep_case"; Needle = '"case": "loader_provider_separation_missing"' },
        @{ Suffix = "normal_module_sep_case"; Needle = '"case": "loader_normal_module_separation_missing"' },
        @{ Suffix = "engine_missing_case"; Needle = '"case": "rollback_transaction_engine_missing"' },
        @{ Suffix = "target_selection_case"; Needle = '"case": "rollback_target_selection_missing"' },
        @{ Suffix = "transaction_provenance_case"; Needle = '"case": "rollback_transaction_id_provenance_missing"' },
        @{ Suffix = "last_good_case"; Needle = '"case": "rollback_last_good_binding_missing"' },
        @{ Suffix = "disabled_set_case"; Needle = '"case": "rollback_disabled_module_set_binding_missing"' },
        @{ Suffix = "artifact_hash_case"; Needle = '"case": "rollback_artifact_hash_binding_missing"' },
        @{ Suffix = "replay_case"; Needle = '"case": "rollback_replay_preconditions_missing"' },
        @{ Suffix = "capability_import_case"; Needle = '"case": "rollback_recovery_capability_import_missing"' },
        @{ Suffix = "atomic_semantics_case"; Needle = '"case": "rollback_atomic_apply_abort_semantics_missing"' },
        @{ Suffix = "durable_case"; Needle = '"case": "durable_audit_rollback_persistence_missing"' },
        @{ Suffix = "memory_provenance_case"; Needle = '"case": "recovery_memory_provenance_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_rollback_still_non_executable"' },
        @{ Suffix = "non_executable_reason"; Needle = '"actual_reason": "recovery_rollback_transaction_behavior_not_implemented"' },
        @{ Suffix = "preview_false"; Needle = '"rollback_preview_enabled": false' },
        @{ Suffix = "apply_false"; Needle = '"rollback_apply_enabled": false' },
        @{ Suffix = "execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.durable_audit_rollback_persistence" -ExpectedMarker "RAIOS_AGENT_END recovery.durable_audit_rollback_persistence"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_durable_audit_rollback_persistence_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.durable_audit_rollback_persistence.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_protocol_state"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_protocol_state_missing"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_durable_audit_rollback_persistence_records": false' },
        @{ Suffix = "no_command_envelope"; Needle = '"accepts_lifeline_command_envelope": false' },
        @{ Suffix = "no_transaction_envelope"; Needle = '"accepts_rollback_transaction_envelope": false' },
        @{ Suffix = "no_plan_json"; Needle = '"accepts_rollback_plan_json": false' },
        @{ Suffix = "no_request_json"; Needle = '"accepts_lifeline_request_json": false' },
        @{ Suffix = "no_loader_descriptor"; Needle = '"accepts_loader_descriptor": false' },
        @{ Suffix = "no_artifact_bytes"; Needle = '"accepts_artifact_bytes": false' },
        @{ Suffix = "no_device_json"; Needle = '"accepts_persistence_device_inventory_json": false' },
        @{ Suffix = "no_layout_json"; Needle = '"accepts_storage_layout_json": false' },
        @{ Suffix = "no_memory_record"; Needle = '"accepts_recovery_memory_record": false' },
        @{ Suffix = "no_openai_shortcut"; Needle = '"uses_direct_openai_recovery_path": false' },
        @{ Suffix = "request_valid"; Needle = '"request_chain_valid": true' },
        @{ Suffix = "vocab_exposed"; Needle = '"command_vocabulary_envelope_exposed": true' },
        @{ Suffix = "vocab_not_accepted"; Needle = '"command_vocabulary_accepted": false' },
        @{ Suffix = "loader_boundary_exposed"; Needle = '"loader_runtime_isolation_boundary_exposed": true' },
        @{ Suffix = "loader_not_accepted"; Needle = '"loader_runtime_isolation_accepted": false' },
        @{ Suffix = "engine_boundary_exposed"; Needle = '"rollback_transaction_engine_boundary_exposed": true' },
        @{ Suffix = "engine_not_accepted"; Needle = '"rollback_transaction_engine_accepted": false' },
        @{ Suffix = "requirements_exposed"; Needle = '"persistence_requirements_exposed": true' },
        @{ Suffix = "persistence_not_ready"; Needle = '"durable_audit_rollback_persistence_ready": false' },
        @{ Suffix = "request_event"; Needle = "`"event_id`": `"$recoveryLifelineRequestEventId`"" },
        @{ Suffix = "identity_event"; Needle = "`"retained_recovery_artifact_identity_event_id`": `"$recoveryIdentityEventId`"" },
        @{ Suffix = "rollback_event"; Needle = "`"retained_recovery_artifact_rollback_evidence_event_id`": `"$recoveryRollbackEvidenceEventId`"" },
        @{ Suffix = "request_hash"; Needle = "`"lifeline_request_reference_hash`": `"sha256:$recoveryLifelineRequestReferenceHash`"" },
        @{ Suffix = "command_vocab_schema"; Needle = '"schema": "raios.recovery_lifeline_command_vocabulary.v0"' },
        @{ Suffix = "loader_boundary_schema"; Needle = '"schema": "raios.recovery_loader_runtime_isolation.v0"' },
        @{ Suffix = "transaction_boundary_schema"; Needle = '"schema": "raios.recovery_rollback_transaction_engine.v0"' },
        @{ Suffix = "persistence_device_schema"; Needle = '"schema": "raios.persistence_device_inventory.v0"' },
        @{ Suffix = "storage_layout_schema"; Needle = '"schema": "raios.durable_audit_rollback_storage_layout_identity.v0"' },
        @{ Suffix = "audit_log_schema"; Needle = '"schema": "raios.durable_audit_append_log_identity.v0"' },
        @{ Suffix = "rollback_store_schema"; Needle = '"schema": "raios.rollback_store_identity.v0"' },
        @{ Suffix = "replay_cursor_schema"; Needle = '"schema": "raios.rollback_transaction_replay_cursor.v0"' },
        @{ Suffix = "checkpoint_schema"; Needle = '"schema": "raios.recovery_last_good_checkpoint_binding.v0"' },
        @{ Suffix = "write_ordering_schema"; Needle = '"schema": "raios.durable_write_ordering.v0"' },
        @{ Suffix = "crash_schema"; Needle = '"schema": "raios.durable_crash_consistency.v0"' },
        @{ Suffix = "integrity_schema"; Needle = '"schema": "raios.durable_integrity_root_hash_chain.v0"' },
        @{ Suffix = "memory_schema"; Needle = '"schema": "raios.recovery_memory_provenance.v0"' },
        @{ Suffix = "persistence_device_missing"; Needle = '"reason": "persistence_device_inventory_missing"' },
        @{ Suffix = "storage_layout_missing"; Needle = '"reason": "durable_storage_layout_identity_missing"' },
        @{ Suffix = "audit_log_missing"; Needle = '"reason": "durable_audit_append_log_identity_missing"' },
        @{ Suffix = "rollback_store_missing"; Needle = '"reason": "rollback_store_identity_missing"' },
        @{ Suffix = "replay_cursor_missing"; Needle = '"reason": "rollback_transaction_replay_cursor_missing"' },
        @{ Suffix = "checkpoint_missing"; Needle = '"reason": "recovery_last_good_checkpoint_binding_missing"' },
        @{ Suffix = "write_ordering_missing"; Needle = '"reason": "durable_write_ordering_missing"' },
        @{ Suffix = "crash_missing"; Needle = '"reason": "durable_crash_consistency_missing"' },
        @{ Suffix = "integrity_missing"; Needle = '"reason": "durable_integrity_root_hash_chain_missing"' },
        @{ Suffix = "memory_missing"; Needle = '"reason": "recovery_memory_provenance_missing"' },
        @{ Suffix = "durable_writes_false"; Needle = '"durable_writes_enabled": false' },
        @{ Suffix = "replay_false"; Needle = '"rollback_replay_enabled": false' },
        @{ Suffix = "memory_writes_false"; Needle = '"recovery_memory_writes_enabled": false' },
        @{ Suffix = "preview_false"; Needle = '"rollback_preview_enabled": false' },
        @{ Suffix = "apply_false"; Needle = '"rollback_apply_enabled": false' },
        @{ Suffix = "no_durable"; Needle = '"creates_durable_records": false' },
        @{ Suffix = "no_install"; Needle = '"installs_rollback_plan": false' },
        @{ Suffix = "no_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    $recoveryDurablePersistenceResponse = Get-LastAgentResponseJson -Method "recovery.durable_audit_rollback_persistence"
    $recoveryDurablePersistenceRequestEventId = [string]$recoveryDurablePersistenceResponse.body.result.retained_recovery_lifeline_request.event_id
    $recoveryDurablePersistenceRequestMatches = $recoveryDurablePersistenceRequestEventId -eq $recoveryLifelineRequestEventId
    Add-Predicate -Name "protocol:recovery_durable_audit_rollback_persistence_request_event_id_matches_retained" -Expected $recoveryLifelineRequestEventId -Passed $recoveryDurablePersistenceRequestMatches -Actual $recoveryDurablePersistenceRequestEventId
    if (-not $recoveryDurablePersistenceRequestMatches) {
        throw "Expected recovery durable persistence request event id $recoveryLifelineRequestEventId, got $recoveryDurablePersistenceRequestEventId"
    }

    Send-AgentCommand -Command "agent recovery.durable_audit_rollback_persistence_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.durable_audit_rollback_persistence_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_durable_audit_rollback_persistence_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.durable_audit_rollback_persistence_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_durable_audit_rollback_persistence_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 51' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "missing_request_case"; Needle = '"case": "missing_lifeline_request_event_id"' },
        @{ Suffix = "request_hash_case"; Needle = '"case": "lifeline_request_reference_hash_mismatch"' },
        @{ Suffix = "protocol_state_missing_case"; Needle = '"case": "protocol_state_missing_after_valid_request"' },
        @{ Suffix = "missing_command_vocab_case"; Needle = '"case": "command_vocabulary_missing_after_protocol_state"' },
        @{ Suffix = "direct_openai_case"; Needle = '"case": "direct_openai_recovery_shortcut_rejected"' },
        @{ Suffix = "isolation_missing_case"; Needle = '"case": "loader_runtime_isolation_missing_after_command_vocabulary"' },
        @{ Suffix = "substituted_isolation_case"; Needle = '"case": "substituted_loader_runtime_isolation"' },
        @{ Suffix = "address_space_case"; Needle = '"case": "loader_address_space_boundary_missing"' },
        @{ Suffix = "engine_boundary_missing_case"; Needle = '"case": "rollback_transaction_engine_boundary_missing_after_loader"' },
        @{ Suffix = "previous_engine_case"; Needle = '"case": "previous_boot_rollback_transaction_engine"' },
        @{ Suffix = "wrong_engine_schema_case"; Needle = '"case": "wrong_schema_rollback_transaction_engine"' },
        @{ Suffix = "substituted_engine_case"; Needle = '"case": "substituted_rollback_transaction_engine"' },
        @{ Suffix = "mismatched_engine_case"; Needle = '"case": "mismatched_rollback_transaction_engine"' },
        @{ Suffix = "target_selection_case"; Needle = '"case": "rollback_target_selection_missing"' },
        @{ Suffix = "transaction_provenance_case"; Needle = '"case": "rollback_transaction_id_provenance_missing"' },
        @{ Suffix = "atomic_semantics_case"; Needle = '"case": "rollback_atomic_apply_abort_semantics_missing"' },
        @{ Suffix = "persistence_missing_case"; Needle = '"case": "durable_audit_rollback_persistence_missing"' },
        @{ Suffix = "device_case"; Needle = '"case": "persistence_device_inventory_missing"' },
        @{ Suffix = "storage_layout_case"; Needle = '"case": "storage_layout_identity_missing"' },
        @{ Suffix = "audit_log_case"; Needle = '"case": "audit_append_log_identity_missing"' },
        @{ Suffix = "rollback_store_case"; Needle = '"case": "rollback_store_identity_missing"' },
        @{ Suffix = "replay_cursor_case"; Needle = '"case": "transaction_replay_cursor_missing"' },
        @{ Suffix = "checkpoint_case"; Needle = '"case": "last_good_checkpoint_binding_missing"' },
        @{ Suffix = "write_ordering_case"; Needle = '"case": "write_ordering_missing"' },
        @{ Suffix = "crash_case"; Needle = '"case": "crash_consistency_missing"' },
        @{ Suffix = "integrity_case"; Needle = '"case": "integrity_root_hash_chain_missing"' },
        @{ Suffix = "memory_provenance_case"; Needle = '"case": "recovery_memory_provenance_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_persistence_still_non_executable"' },
        @{ Suffix = "non_executable_reason"; Needle = '"actual_reason": "durable_audit_rollback_persistence_behavior_not_implemented"' },
        @{ Suffix = "durable_writes_false"; Needle = '"durable_writes_enabled": false' },
        @{ Suffix = "replay_false"; Needle = '"rollback_replay_enabled": false' },
        @{ Suffix = "memory_writes_false"; Needle = '"recovery_memory_writes_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.memory_provenance" -ExpectedMarker "RAIOS_AGENT_END recovery.memory_provenance"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_memory_provenance_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_memory_provenance.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_protocol_state"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_protocol_state_missing"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_memory_provenance_records": false' },
        @{ Suffix = "no_memory_json"; Needle = '"accepts_memory_record_json": false' },
        @{ Suffix = "no_memory_record"; Needle = '"accepts_recovery_memory_record": false' },
        @{ Suffix = "no_provider_export_accept"; Needle = '"accepts_provider_context_export": false' },
        @{ Suffix = "no_command_envelope"; Needle = '"accepts_lifeline_command_envelope": false' },
        @{ Suffix = "no_transaction_envelope"; Needle = '"accepts_rollback_transaction_envelope": false' },
        @{ Suffix = "no_device_json"; Needle = '"accepts_persistence_device_inventory_json": false' },
        @{ Suffix = "no_memory_write"; Needle = '"writes_recovery_memory": false' },
        @{ Suffix = "no_provider_export"; Needle = '"exports_provider_context": false' },
        @{ Suffix = "no_durable_write"; Needle = '"writes_durable_audit_log": false' },
        @{ Suffix = "no_rollback_store"; Needle = '"writes_rollback_store": false' },
        @{ Suffix = "no_replay"; Needle = '"replays_rollback_transactions": false' },
        @{ Suffix = "no_openai_shortcut"; Needle = '"uses_direct_openai_recovery_path": false' },
        @{ Suffix = "request_valid"; Needle = '"request_chain_valid": true' },
        @{ Suffix = "vocab_exposed"; Needle = '"command_vocabulary_envelope_exposed": true' },
        @{ Suffix = "vocab_not_accepted"; Needle = '"command_vocabulary_accepted": false' },
        @{ Suffix = "loader_boundary_exposed"; Needle = '"loader_runtime_isolation_boundary_exposed": true' },
        @{ Suffix = "loader_not_accepted"; Needle = '"loader_runtime_isolation_accepted": false' },
        @{ Suffix = "engine_boundary_exposed"; Needle = '"rollback_transaction_engine_boundary_exposed": true' },
        @{ Suffix = "engine_not_accepted"; Needle = '"rollback_transaction_engine_accepted": false' },
        @{ Suffix = "durable_boundary_exposed"; Needle = '"durable_audit_rollback_persistence_boundary_exposed": true' },
        @{ Suffix = "durable_not_accepted"; Needle = '"durable_audit_rollback_persistence_accepted": false' },
        @{ Suffix = "requirements_exposed"; Needle = '"memory_provenance_requirements_exposed": true' },
        @{ Suffix = "memory_not_ready"; Needle = '"recovery_memory_provenance_ready": false' },
        @{ Suffix = "request_event"; Needle = "`"event_id`": `"$recoveryLifelineRequestEventId`"" },
        @{ Suffix = "identity_event"; Needle = "`"retained_recovery_artifact_identity_event_id`": `"$recoveryIdentityEventId`"" },
        @{ Suffix = "rollback_event"; Needle = "`"retained_recovery_artifact_rollback_evidence_event_id`": `"$recoveryRollbackEvidenceEventId`"" },
        @{ Suffix = "request_hash"; Needle = "`"lifeline_request_reference_hash`": `"sha256:$recoveryLifelineRequestReferenceHash`"" },
        @{ Suffix = "command_vocab_schema"; Needle = '"schema": "raios.recovery_lifeline_command_vocabulary.v0"' },
        @{ Suffix = "loader_boundary_schema"; Needle = '"schema": "raios.recovery_loader_runtime_isolation.v0"' },
        @{ Suffix = "transaction_boundary_schema"; Needle = '"schema": "raios.recovery_rollback_transaction_engine.v0"' },
        @{ Suffix = "durable_boundary_schema"; Needle = '"schema": "raios.durable_audit_rollback_persistence.v0"' },
        @{ Suffix = "source_ids_schema"; Needle = '"schema": "raios.recovery_memory_source_record_ids.v0"' },
        @{ Suffix = "source_hashes_schema"; Needle = '"schema": "raios.recovery_memory_source_schema_hashes.v0"' },
        @{ Suffix = "classification_schema"; Needle = '"schema": "raios.recovery_memory_classification.v0"' },
        @{ Suffix = "authority_schema"; Needle = '"schema": "raios.recovery_memory_authority_level.v0"' },
        @{ Suffix = "rollback_binding_schema"; Needle = '"schema": "raios.recovery_memory_rollback_transaction_binding.v0"' },
        @{ Suffix = "checkpoint_binding_schema"; Needle = '"schema": "raios.recovery_memory_last_good_checkpoint_binding.v0"' },
        @{ Suffix = "export_profile_schema"; Needle = '"schema": "raios.recovery_memory_export_profile.v0"' },
        @{ Suffix = "redaction_schema"; Needle = '"schema": "raios.recovery_memory_redaction_state.v0"' },
        @{ Suffix = "replay_window_schema"; Needle = '"schema": "raios.recovery_memory_replay_window.v0"' },
        @{ Suffix = "audit_linkage_schema"; Needle = '"schema": "raios.recovery_memory_audit_linkage.v0"' },
        @{ Suffix = "source_ids_missing"; Needle = '"reason": "recovery_memory_source_record_ids_missing"' },
        @{ Suffix = "source_hashes_missing"; Needle = '"reason": "recovery_memory_source_schema_hashes_missing"' },
        @{ Suffix = "classification_missing"; Needle = '"reason": "recovery_memory_classification_missing"' },
        @{ Suffix = "authority_missing"; Needle = '"reason": "recovery_memory_authority_level_missing"' },
        @{ Suffix = "rollback_binding_missing"; Needle = '"reason": "recovery_memory_rollback_transaction_binding_missing"' },
        @{ Suffix = "checkpoint_binding_missing"; Needle = '"reason": "recovery_memory_last_good_checkpoint_binding_missing"' },
        @{ Suffix = "export_profile_missing"; Needle = '"reason": "recovery_only_export_profile_missing"' },
        @{ Suffix = "redaction_missing"; Needle = '"reason": "recovery_memory_redaction_state_missing"' },
        @{ Suffix = "replay_window_missing"; Needle = '"reason": "recovery_memory_replay_window_missing"' },
        @{ Suffix = "audit_linkage_missing"; Needle = '"reason": "recovery_memory_audit_linkage_missing"' },
        @{ Suffix = "memory_writes_false"; Needle = '"memory_writes_enabled": false' },
        @{ Suffix = "provider_export_false"; Needle = '"provider_export_enabled": false' },
        @{ Suffix = "durable_writes_false"; Needle = '"durable_writes_enabled": false' },
        @{ Suffix = "replay_false"; Needle = '"rollback_replay_enabled": false' },
        @{ Suffix = "recovery_memory_writes_false"; Needle = '"recovery_memory_writes_enabled": false' },
        @{ Suffix = "preview_false"; Needle = '"rollback_preview_enabled": false' },
        @{ Suffix = "apply_false"; Needle = '"rollback_apply_enabled": false' },
        @{ Suffix = "no_durable"; Needle = '"creates_durable_records": false' },
        @{ Suffix = "no_install"; Needle = '"installs_rollback_plan": false' },
        @{ Suffix = "no_slot"; Needle = '"allocates_service_slot": false' },
        @{ Suffix = "no_service_change"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    $recoveryMemoryProvenanceResponse = Get-LastAgentResponseJson -Method "recovery.memory_provenance"
    $recoveryMemoryProvenanceRequestEventId = [string]$recoveryMemoryProvenanceResponse.body.result.retained_recovery_lifeline_request.event_id
    $recoveryMemoryProvenanceRequestMatches = $recoveryMemoryProvenanceRequestEventId -eq $recoveryLifelineRequestEventId
    Add-Predicate -Name "protocol:recovery_memory_provenance_request_event_id_matches_retained" -Expected $recoveryLifelineRequestEventId -Passed $recoveryMemoryProvenanceRequestMatches -Actual $recoveryMemoryProvenanceRequestEventId
    if (-not $recoveryMemoryProvenanceRequestMatches) {
        throw "Expected recovery memory provenance request event id $recoveryLifelineRequestEventId, got $recoveryMemoryProvenanceRequestEventId"
    }

    Send-AgentCommand -Command "agent recovery.memory_provenance_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.memory_provenance_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_memory_provenance_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_memory_provenance_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_memory_provenance_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 65' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "missing_request_case"; Needle = '"case": "missing_lifeline_request_event_id"' },
        @{ Suffix = "request_hash_case"; Needle = '"case": "lifeline_request_reference_hash_mismatch"' },
        @{ Suffix = "protocol_state_missing_case"; Needle = '"case": "protocol_state_missing_after_valid_request"' },
        @{ Suffix = "missing_command_vocab_case"; Needle = '"case": "command_vocabulary_missing_after_protocol_state"' },
        @{ Suffix = "direct_openai_case"; Needle = '"case": "direct_openai_recovery_shortcut_rejected"' },
        @{ Suffix = "isolation_missing_case"; Needle = '"case": "loader_runtime_isolation_missing_after_command_vocabulary"' },
        @{ Suffix = "substituted_isolation_case"; Needle = '"case": "substituted_loader_runtime_isolation"' },
        @{ Suffix = "engine_boundary_missing_case"; Needle = '"case": "rollback_transaction_engine_boundary_missing_after_loader"' },
        @{ Suffix = "mismatched_engine_case"; Needle = '"case": "mismatched_rollback_transaction_engine"' },
        @{ Suffix = "durable_missing_case"; Needle = '"case": "durable_persistence_boundary_missing_after_rollback_engine"' },
        @{ Suffix = "previous_durable_case"; Needle = '"case": "previous_boot_durable_persistence"' },
        @{ Suffix = "mismatched_durable_case"; Needle = '"case": "mismatched_durable_persistence"' },
        @{ Suffix = "device_case"; Needle = '"case": "persistence_device_inventory_missing"' },
        @{ Suffix = "integrity_case"; Needle = '"case": "integrity_root_hash_chain_missing"' },
        @{ Suffix = "memory_provenance_case"; Needle = '"case": "recovery_memory_provenance_missing"' },
        @{ Suffix = "source_ids_case"; Needle = '"case": "source_record_ids_missing"' },
        @{ Suffix = "schema_hashes_case"; Needle = '"case": "source_schema_hashes_missing"' },
        @{ Suffix = "classification_case"; Needle = '"case": "memory_classification_missing"' },
        @{ Suffix = "authority_case"; Needle = '"case": "memory_authority_level_missing"' },
        @{ Suffix = "rollback_binding_case"; Needle = '"case": "memory_rollback_transaction_binding_missing"' },
        @{ Suffix = "checkpoint_binding_case"; Needle = '"case": "memory_last_good_checkpoint_binding_missing"' },
        @{ Suffix = "export_profile_case"; Needle = '"case": "recovery_only_export_profile_missing"' },
        @{ Suffix = "redaction_case"; Needle = '"case": "memory_redaction_state_missing"' },
        @{ Suffix = "replay_window_case"; Needle = '"case": "memory_replay_window_missing"' },
        @{ Suffix = "audit_linkage_case"; Needle = '"case": "memory_audit_linkage_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_memory_still_non_executable"' },
        @{ Suffix = "non_executable_reason"; Needle = '"actual_reason": "recovery_memory_provenance_behavior_not_implemented"' },
        @{ Suffix = "memory_writes_false"; Needle = '"memory_writes_enabled": false' },
        @{ Suffix = "provider_export_false"; Needle = '"provider_export_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_admission" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_admission"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_admission_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_admission.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_protocol_state"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_protocol_state_missing"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_admission_records": false' },
        @{ Suffix = "no_command_envelope"; Needle = '"accepts_lifeline_command_envelope": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "no_status_exec"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "no_preview"; Needle = '"executes_rollback_preview": false' },
        @{ Suffix = "no_apply"; Needle = '"executes_rollback_apply": false' },
        @{ Suffix = "no_disable"; Needle = '"executes_disable_module": false' },
        @{ Suffix = "no_restart"; Needle = '"executes_restart_last_good": false' },
        @{ Suffix = "no_load_by_hash"; Needle = '"executes_load_recovery_artifact_by_hash": false' },
        @{ Suffix = "request_valid"; Needle = '"request_chain_valid": true' },
        @{ Suffix = "vocab_exposed"; Needle = '"command_vocabulary_envelope_exposed": true' },
        @{ Suffix = "loader_boundary_exposed"; Needle = '"loader_runtime_isolation_boundary_exposed": true' },
        @{ Suffix = "engine_boundary_exposed"; Needle = '"rollback_transaction_engine_boundary_exposed": true' },
        @{ Suffix = "durable_boundary_exposed"; Needle = '"durable_audit_rollback_persistence_boundary_exposed": true' },
        @{ Suffix = "memory_boundary_exposed"; Needle = '"recovery_memory_provenance_boundary_exposed": true' },
        @{ Suffix = "memory_not_accepted"; Needle = '"recovery_memory_provenance_accepted": false' },
        @{ Suffix = "requirements_not_exposed"; Needle = '"command_admission_requirements_exposed": false' },
        @{ Suffix = "admission_not_ready"; Needle = '"command_admission_ready": false' },
        @{ Suffix = "status_command"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "preview_command"; Needle = '"command_id": "recovery.lifeline.rollback_preview"' },
        @{ Suffix = "apply_command"; Needle = '"command_id": "recovery.lifeline.rollback_apply"' },
        @{ Suffix = "disable_command"; Needle = '"command_id": "recovery.lifeline.disable_module"' },
        @{ Suffix = "restart_command"; Needle = '"command_id": "recovery.lifeline.restart_last_good"' },
        @{ Suffix = "load_hash_command"; Needle = '"command_id": "recovery.lifeline.load_artifact_by_hash"' },
        @{ Suffix = "status_schema"; Needle = '"admission_schema": "raios.recovery_lifeline_status_admission.v0"' },
        @{ Suffix = "preview_schema"; Needle = '"admission_schema": "raios.recovery_rollback_preview_admission.v0"' },
        @{ Suffix = "apply_schema"; Needle = '"admission_schema": "raios.recovery_rollback_apply_admission.v0"' },
        @{ Suffix = "disable_schema"; Needle = '"admission_schema": "raios.recovery_disable_module_admission.v0"' },
        @{ Suffix = "restart_schema"; Needle = '"admission_schema": "raios.recovery_restart_last_good_admission.v0"' },
        @{ Suffix = "load_hash_schema"; Needle = '"admission_schema": "raios.recovery_load_artifact_by_hash_admission.v0"' },
        @{ Suffix = "request_event"; Needle = "`"event_id`": `"$recoveryLifelineRequestEventId`"" },
        @{ Suffix = "request_hash"; Needle = "`"lifeline_request_reference_hash`": `"sha256:$recoveryLifelineRequestReferenceHash`"" },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "provider_export_false"; Needle = '"provider_export_enabled": false' },
        @{ Suffix = "durable_writes_false"; Needle = '"durable_writes_enabled": false' },
        @{ Suffix = "memory_writes_false"; Needle = '"memory_writes_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )
    $recoveryCommandAdmissionResponse = Get-LastAgentResponseJson -Method "recovery.lifeline_command_admission"
    $recoveryCommandAdmissionRequestEventId = [string]$recoveryCommandAdmissionResponse.body.result.retained_recovery_lifeline_request.event_id
    $recoveryCommandAdmissionRequestMatches = $recoveryCommandAdmissionRequestEventId -eq $recoveryLifelineRequestEventId
    Add-Predicate -Name "protocol:recovery_lifeline_command_admission_request_event_id_matches_retained" -Expected $recoveryLifelineRequestEventId -Passed $recoveryCommandAdmissionRequestMatches -Actual $recoveryCommandAdmissionRequestEventId
    if (-not $recoveryCommandAdmissionRequestMatches) {
        throw "Expected recovery command-admission request event id $recoveryLifelineRequestEventId, got $recoveryCommandAdmissionRequestEventId"
    }

    Send-AgentCommand -Command "agent recovery.lifeline_command_admission_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_admission_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_admission_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_admission_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_admission_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 45' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "missing_request_case"; Needle = '"case": "missing_lifeline_request_event_id"' },
        @{ Suffix = "request_hash_case"; Needle = '"case": "lifeline_request_reference_hash_mismatch"' },
        @{ Suffix = "protocol_state_missing_case"; Needle = '"case": "protocol_state_missing_after_valid_request"' },
        @{ Suffix = "command_vocab_missing_case"; Needle = '"case": "command_vocabulary_missing_after_protocol_state"' },
        @{ Suffix = "direct_openai_case"; Needle = '"case": "direct_openai_recovery_shortcut_rejected"' },
        @{ Suffix = "loader_mismatch_case"; Needle = '"case": "mismatched_loader_runtime_isolation"' },
        @{ Suffix = "engine_mismatch_case"; Needle = '"case": "mismatched_rollback_transaction_engine"' },
        @{ Suffix = "durable_mismatch_case"; Needle = '"case": "mismatched_durable_persistence"' },
        @{ Suffix = "memory_boundary_case"; Needle = '"case": "recovery_memory_provenance_boundary_missing"' },
        @{ Suffix = "memory_mismatch_case"; Needle = '"case": "mismatched_recovery_memory_provenance"' },
        @{ Suffix = "memory_facts_case"; Needle = '"case": "recovery_memory_provenance_facts_missing"' },
        @{ Suffix = "admission_missing_case"; Needle = '"case": "command_admission_requirements_missing"' },
        @{ Suffix = "status_missing_case"; Needle = '"case": "lifeline_status_command_admission_missing"' },
        @{ Suffix = "preview_missing_case"; Needle = '"case": "rollback_preview_command_admission_missing"' },
        @{ Suffix = "apply_missing_case"; Needle = '"case": "rollback_apply_command_admission_missing"' },
        @{ Suffix = "disable_missing_case"; Needle = '"case": "disable_module_command_admission_missing"' },
        @{ Suffix = "restart_missing_case"; Needle = '"case": "restart_last_good_command_admission_missing"' },
        @{ Suffix = "load_hash_missing_case"; Needle = '"case": "load_artifact_by_hash_command_admission_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_command_admission_still_non_executable"' },
        @{ Suffix = "non_executable_reason"; Needle = '"actual_reason": "recovery_lifeline_command_admission_behavior_not_implemented"' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_envelope_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_envelope_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_envelope_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_envelope_reference_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "mutation_none"; Needle = '"global_event_log_mutation": "none"' },
        @{ Suffix = "no_command_envelope"; Needle = '"accepts_lifeline_command_envelope": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "no_status_exec"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "no_preview"; Needle = '"executes_rollback_preview": false' },
        @{ Suffix = "no_apply"; Needle = '"executes_rollback_apply": false' },
        @{ Suffix = "no_disable"; Needle = '"executes_disable_module": false' },
        @{ Suffix = "no_restart"; Needle = '"executes_restart_last_good": false' },
        @{ Suffix = "no_load_by_hash"; Needle = '"executes_load_recovery_artifact_by_hash": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.lifeline_command_envelope_diagnostic' },
        @{ Suffix = "reference_hash_arg"; Needle = '<command_envelope_reference_hash>' },
        @{ Suffix = "request_event_arg"; Needle = '<retained_lifeline_request_event_id>' },
        @{ Suffix = "admission_boundary_arg"; Needle = '<command_admission_boundary_id>' },
        @{ Suffix = "request_hash_arg"; Needle = '<lifeline_request_reference_hash>' },
        @{ Suffix = "reference_schema"; Needle = '"command_reference_schema": "raios.recovery_lifeline_command_envelope_reference.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"command_reference_canonicalization": "raios.recovery_lifeline_command_envelope_reference.canonical.v0"' },
        @{ Suffix = "boundary_id"; Needle = '"command_admission_boundary_id": "boundary.recovery_lifeline_command_admission.current_boot"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_envelope_reference_absent"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "status_command"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "preview_command"; Needle = '"command_id": "recovery.lifeline.rollback_preview"' },
        @{ Suffix = "apply_command"; Needle = '"command_id": "recovery.lifeline.rollback_apply"' },
        @{ Suffix = "disable_command"; Needle = '"command_id": "recovery.lifeline.disable_module"' },
        @{ Suffix = "restart_command"; Needle = '"command_id": "recovery.lifeline.restart_last_good"' },
        @{ Suffix = "load_hash_command"; Needle = '"command_id": "recovery.lifeline.load_artifact_by_hash"' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "provider_export_false"; Needle = '"exports_provider_context": false' },
        @{ Suffix = "durable_writes_false"; Needle = '"writes_durable_audit_log": false' },
        @{ Suffix = "memory_writes_false"; Needle = '"writes_recovery_memory": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_envelope_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_envelope_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_envelope_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_envelope_reference_diagnostic_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_envelope_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 47' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "missing_request_case"; Needle = '"case": "missing_lifeline_request_event_id"' },
        @{ Suffix = "request_hash_case"; Needle = '"case": "lifeline_request_reference_hash_mismatch"' },
        @{ Suffix = "protocol_state_missing_case"; Needle = '"case": "protocol_state_missing_after_valid_request"' },
        @{ Suffix = "command_vocab_missing_case"; Needle = '"case": "command_vocabulary_missing_after_protocol_state"' },
        @{ Suffix = "direct_openai_case"; Needle = '"case": "direct_openai_recovery_shortcut_rejected"' },
        @{ Suffix = "loader_mismatch_case"; Needle = '"case": "mismatched_loader_runtime_isolation"' },
        @{ Suffix = "engine_mismatch_case"; Needle = '"case": "mismatched_rollback_transaction_engine"' },
        @{ Suffix = "durable_mismatch_case"; Needle = '"case": "mismatched_durable_persistence"' },
        @{ Suffix = "memory_mismatch_case"; Needle = '"case": "mismatched_recovery_memory_provenance"' },
        @{ Suffix = "admission_missing_case"; Needle = '"case": "recovery_lifeline_command_admission_missing"' },
        @{ Suffix = "previous_admission_case"; Needle = '"case": "previous_boot_recovery_lifeline_command_admission"' },
        @{ Suffix = "wrong_admission_case"; Needle = '"case": "wrong_schema_recovery_lifeline_command_admission"' },
        @{ Suffix = "substituted_admission_case"; Needle = '"case": "substituted_recovery_lifeline_command_admission"' },
        @{ Suffix = "mismatched_admission_case"; Needle = '"case": "mismatched_recovery_lifeline_command_admission"' },
        @{ Suffix = "unsupported_command_case"; Needle = '"case": "unsupported_lifeline_command_id"' },
        @{ Suffix = "schema_mismatch_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "capability_mismatch_case"; Needle = '"case": "required_capability_mismatch"' },
        @{ Suffix = "argument_hash_missing_case"; Needle = '"case": "argument_hash_missing"' },
        @{ Suffix = "target_locator_missing_case"; Needle = '"case": "target_locator_missing"' },
        @{ Suffix = "reference_hash_case"; Needle = '"case": "command_envelope_reference_hash_mismatch"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_command_envelope_still_non_executable"' },
        @{ Suffix = "non_executable_reason"; Needle = '"actual_reason": "recovery_lifeline_command_envelope_reference_behavior_not_implemented"' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryCommandAdmissionBoundaryId = "boundary.recovery_lifeline_command_admission.current_boot"
    $recoveryCommandTargetLocator = "recovery.lifeline.status.current_boot"
    $recoveryLifelineStatusArgumentCanonical = @(
        "schema=raios.recovery_lifeline_command.status_args.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "body_present=false"
    ) -join "`n"
    $recoveryLifelineStatusArgumentHash = Get-TextSha256 -Text $recoveryLifelineStatusArgumentCanonical
    $recoveryLifelineCommandEnvelopeCanonical = @(
        "canonicalization=raios.recovery_lifeline_command_envelope_reference.canonical.v0",
        "schema=raios.recovery_lifeline_command_envelope_reference.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline_command",
        "scope=current_boot",
        "retained_recovery_lifeline_request_event_id=$recoveryLifelineRequestEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "required_capability=cap.recovery.load_artifact.read",
        "target_locator=$recoveryCommandTargetLocator",
        "command_admission_boundary_id=$recoveryCommandAdmissionBoundaryId",
        "lifeline_request_reference_sha256=$recoveryLifelineRequestReferenceHash",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "writes_recovery_memory=false",
        "exports_provider_context=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryLifelineCommandEnvelopeReferenceHash = Get-TextSha256 -Text $recoveryLifelineCommandEnvelopeCanonical
    $recoveryLifelineCommandEnvelopeCommand = "agent recovery.lifeline_command_envelope_diagnostic $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineRequestEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash cap.recovery.load_artifact.read $recoveryCommandTargetLocator $recoveryCommandAdmissionBoundaryId $recoveryLifelineRequestReferenceHash"

    Send-AgentCommand -Command $recoveryLifelineCommandEnvelopeCommand -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_envelope_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_envelope_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_envelope_reference_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_command_still_denied"' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_latest_event"; Needle = '"latest_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "argument_schema"; Needle = '"argument_schema": "raios.recovery_lifeline_command.status_args.v0"' },
        @{ Suffix = "required_capability"; Needle = '"required_capability": "cap.recovery.load_artifact.read"' },
        @{ Suffix = "target_locator"; Needle = "`"target_locator`": `"$recoveryCommandTargetLocator`"" },
        @{ Suffix = "boundary_id"; Needle = "`"command_admission_boundary_id`": `"$recoveryCommandAdmissionBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "request_hash"; Needle = "`"lifeline_request_reference_hash`": `"sha256:$recoveryLifelineRequestReferenceHash`"" },
        @{ Suffix = "command_reference_hash"; Needle = "`"command_envelope_reference_hash`": `"sha256:$recoveryLifelineCommandEnvelopeReferenceHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLifelineCommandEnvelopeResponse = Get-LastAgentResponseJson -Method "recovery.lifeline_command_envelope_diagnostic"
    $recoveryLifelineCommandEnvelopeEventId = [string]$recoveryLifelineCommandEnvelopeResponse.body.result.retained_command_envelope_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_lifeline_command_envelope_retained_reference_event_id_captured" -Value $recoveryLifelineCommandEnvelopeEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_body_canonicalization_missing"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_dispatch_records": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_command_envelope"; Needle = '"accepts_lifeline_command_envelope": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "no_status_exec"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "no_preview"; Needle = '"executes_rollback_preview": false' },
        @{ Suffix = "no_apply"; Needle = '"executes_rollback_apply": false' },
        @{ Suffix = "no_disable"; Needle = '"executes_disable_module": false' },
        @{ Suffix = "no_restart"; Needle = '"executes_restart_last_good": false' },
        @{ Suffix = "no_load_by_hash"; Needle = '"executes_load_recovery_artifact_by_hash": false' },
        @{ Suffix = "reference_schema"; Needle = '"command_envelope_reference_schema": "raios.recovery_lifeline_command_envelope_reference.v0"' },
        @{ Suffix = "admission_schema"; Needle = '"command_admission_schema": "raios.recovery_lifeline_command_admission.v0"' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_command_still_denied"' },
        @{ Suffix = "retained_event"; Needle = "`"event_id`": `"$recoveryLifelineCommandEnvelopeEventId`"" },
        @{ Suffix = "retained_matches"; Needle = '"matches_latest_lifeline_request": true' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "target_locator"; Needle = "`"target_locator`": `"$recoveryCommandTargetLocator`"" },
        @{ Suffix = "command_reference_hash"; Needle = "`"command_envelope_reference_hash`": `"sha256:$recoveryLifelineCommandEnvelopeReferenceHash`"" },
        @{ Suffix = "body_fact"; Needle = '"fact": "command_body_canonicalization"' },
        @{ Suffix = "body_schema"; Needle = '"schema": "raios.recovery_lifeline_command_body_canonicalization.v0"' },
        @{ Suffix = "handler_fact"; Needle = '"fact": "command_handler_binding"' },
        @{ Suffix = "status_handler_fact"; Needle = '"fact": "status_read_handler"' },
        @{ Suffix = "preview_auth_fact"; Needle = '"fact": "rollback_preview_authorization"' },
        @{ Suffix = "apply_auth_fact"; Needle = '"fact": "rollback_apply_authorization"' },
        @{ Suffix = "disable_target_fact"; Needle = '"fact": "disable_module_target_binding"' },
        @{ Suffix = "restart_target_fact"; Needle = '"fact": "restart_last_good_target_binding"' },
        @{ Suffix = "load_hash_target_fact"; Needle = '"fact": "load_artifact_by_hash_target_binding"' },
        @{ Suffix = "memory_authority_fact"; Needle = '"fact": "recovery_memory_write_authority"' },
        @{ Suffix = "durable_authority_fact"; Needle = '"fact": "durable_audit_rollback_write_authority"' },
        @{ Suffix = "service_effect_fact"; Needle = '"fact": "service_inventory_side_effect_boundary"' },
        @{ Suffix = "envelope_accepted"; Needle = '"command_envelope_reference_accepted": true' },
        @{ Suffix = "body_missing"; Needle = '"command_body_canonicalization_present": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "memory_writes_false"; Needle = '"recovery_memory_writes_enabled": false' },
        @{ Suffix = "provider_export_false"; Needle = '"provider_export_enabled": false' },
        @{ Suffix = "durable_writes_false"; Needle = '"durable_writes_enabled": false' },
        @{ Suffix = "service_inventory_none"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_dispatch_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 40' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "request_missing_case"; Needle = '"case": "missing_lifeline_request_event_id"' },
        @{ Suffix = "protocol_missing_case"; Needle = '"case": "protocol_state_missing_after_valid_request"' },
        @{ Suffix = "command_vocab_case"; Needle = '"case": "command_vocabulary_missing_after_protocol_state"' },
        @{ Suffix = "loader_mismatch_case"; Needle = '"case": "mismatched_loader_runtime_isolation"' },
        @{ Suffix = "engine_mismatch_case"; Needle = '"case": "mismatched_rollback_transaction_engine"' },
        @{ Suffix = "durable_mismatch_case"; Needle = '"case": "mismatched_durable_persistence"' },
        @{ Suffix = "memory_mismatch_case"; Needle = '"case": "mismatched_recovery_memory_provenance"' },
        @{ Suffix = "admission_mismatch_case"; Needle = '"case": "mismatched_recovery_lifeline_command_admission"' },
        @{ Suffix = "envelope_missing_case"; Needle = '"case": "command_envelope_reference_missing"' },
        @{ Suffix = "envelope_previous_case"; Needle = '"case": "previous_boot_command_envelope_reference"' },
        @{ Suffix = "envelope_wrong_case"; Needle = '"case": "wrong_schema_command_envelope_reference"' },
        @{ Suffix = "envelope_sub_case"; Needle = '"case": "substituted_command_envelope_reference"' },
        @{ Suffix = "envelope_mismatch_case"; Needle = '"case": "mismatched_command_envelope_reference"' },
        @{ Suffix = "body_missing_case"; Needle = '"case": "command_body_canonicalization_missing"' },
        @{ Suffix = "handler_missing_case"; Needle = '"case": "command_handler_binding_missing"' },
        @{ Suffix = "status_handler_missing_case"; Needle = '"case": "status_read_handler_missing"' },
        @{ Suffix = "preview_auth_missing_case"; Needle = '"case": "rollback_preview_authorization_missing"' },
        @{ Suffix = "apply_auth_missing_case"; Needle = '"case": "rollback_apply_authorization_missing"' },
        @{ Suffix = "disable_target_missing_case"; Needle = '"case": "disable_module_target_binding_missing"' },
        @{ Suffix = "restart_target_missing_case"; Needle = '"case": "restart_last_good_target_binding_missing"' },
        @{ Suffix = "load_hash_target_missing_case"; Needle = '"case": "load_artifact_by_hash_target_binding_missing"' },
        @{ Suffix = "memory_write_missing_case"; Needle = '"case": "recovery_memory_write_authority_missing"' },
        @{ Suffix = "durable_write_missing_case"; Needle = '"case": "durable_audit_rollback_write_authority_missing"' },
        @{ Suffix = "service_effect_missing_case"; Needle = '"case": "service_inventory_side_effect_boundary_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_command_dispatch_still_non_executable"' },
        @{ Suffix = "non_executable_reason"; Needle = '"actual_reason": "recovery_lifeline_command_dispatch_behavior_not_implemented"' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_body_canonicalization_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_body_canonicalization_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_body_canonicalization_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_body_canonicalization_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_body_canonicalization_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_body_canonicalization_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_command_envelope"; Needle = '"accepts_lifeline_command_envelope": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.lifeline_command_body_canonicalization_diagnostic' },
        @{ Suffix = "body_schema"; Needle = '"command_body_schema": "raios.recovery_lifeline_command_body_canonicalization.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"command_body_canonicalization": "raios.recovery_lifeline_command_body_canonicalization.canonical.v0"' },
        @{ Suffix = "dispatch_boundary"; Needle = '"command_dispatch_boundary_id": "boundary.recovery_lifeline_command_dispatch_denial.current_boot"' },
        @{ Suffix = "dispatch_expected_reason"; Needle = '"expected_reason_before_body_canonicalization": "recovery_lifeline_command_body_canonicalization_missing"' },
        @{ Suffix = "retained_event"; Needle = "`"event_id`": `"$recoveryLifelineCommandEnvelopeEventId`"" },
        @{ Suffix = "envelope_accepted"; Needle = '"command_envelope_reference_accepted": true' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "schema_fact"; Needle = '"fact": "per_command_body_schema_canonicalization"' },
        @{ Suffix = "redaction_fact"; Needle = '"fact": "body_redaction_classification"' },
        @{ Suffix = "handler_fact"; Needle = '"fact": "handler_input_binding"' },
        @{ Suffix = "rollback_link_fact"; Needle = '"fact": "rollback_authorization_linkage"' },
        @{ Suffix = "memory_link_fact"; Needle = '"fact": "recovery_memory_write_linkage"' },
        @{ Suffix = "durable_link_fact"; Needle = '"fact": "durable_audit_rollback_write_linkage"' },
        @{ Suffix = "service_link_fact"; Needle = '"fact": "service_inventory_side_effect_linkage"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_body_canonicalization_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_body_canonicalization_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_body_canonicalization_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_body_canonicalization_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_body_canonicalization_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 43' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "request_missing_case"; Needle = '"case": "missing_lifeline_request_event_id"' },
        @{ Suffix = "protocol_missing_case"; Needle = '"case": "protocol_state_missing_after_valid_request"' },
        @{ Suffix = "command_vocab_case"; Needle = '"case": "command_vocabulary_missing_after_protocol_state"' },
        @{ Suffix = "loader_mismatch_case"; Needle = '"case": "mismatched_loader_runtime_isolation"' },
        @{ Suffix = "engine_mismatch_case"; Needle = '"case": "mismatched_rollback_transaction_engine"' },
        @{ Suffix = "durable_mismatch_case"; Needle = '"case": "mismatched_durable_persistence"' },
        @{ Suffix = "memory_mismatch_case"; Needle = '"case": "mismatched_recovery_memory_provenance"' },
        @{ Suffix = "admission_mismatch_case"; Needle = '"case": "mismatched_recovery_lifeline_command_admission"' },
        @{ Suffix = "envelope_missing_case"; Needle = '"case": "command_envelope_reference_missing"' },
        @{ Suffix = "envelope_mismatch_case"; Needle = '"case": "mismatched_command_envelope_reference"' },
        @{ Suffix = "dispatch_boundary_case"; Needle = '"case": "dispatch_boundary_not_body_missing"' },
        @{ Suffix = "body_missing_case"; Needle = '"case": "command_body_canonicalization_missing"' },
        @{ Suffix = "body_wrong_schema_case"; Needle = '"case": "wrong_schema_command_body_canonicalization_reference"' },
        @{ Suffix = "body_sub_case"; Needle = '"case": "substituted_command_body_canonicalization_reference"' },
        @{ Suffix = "body_mismatch_case"; Needle = '"case": "mismatched_command_body_canonicalization_reference"' },
        @{ Suffix = "unsupported_command_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_mismatch_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "argument_hash_missing_case"; Needle = '"case": "argument_hash_missing"' },
        @{ Suffix = "target_locator_missing_case"; Needle = '"case": "target_locator_missing"' },
        @{ Suffix = "envelope_hash_case"; Needle = '"case": "command_envelope_reference_hash_mismatch"' },
        @{ Suffix = "dispatch_id_case"; Needle = '"case": "command_dispatch_boundary_id_mismatch"' },
        @{ Suffix = "body_hash_case"; Needle = '"case": "command_body_canonicalization_hash_mismatch"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_command_body_canonicalization_still_non_executable"' },
        @{ Suffix = "non_executable_reason"; Needle = '"actual_reason": "recovery_lifeline_command_body_canonicalization_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryCommandDispatchBoundaryId = "boundary.recovery_lifeline_command_dispatch_denial.current_boot"
    $recoveryLifelineCommandBodyCanonical = @(
        "canonicalization=raios.recovery_lifeline_command_body_canonicalization.canonical.v0",
        "schema=raios.recovery_lifeline_command_body_canonicalization.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline_command_body",
        "scope=current_boot",
        "retained_recovery_lifeline_command_envelope_event_id=$recoveryLifelineCommandEnvelopeEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryLifelineCommandBodyCanonicalizationHash = Get-TextSha256 -Text $recoveryLifelineCommandBodyCanonical
    $recoveryLifelineCommandBodyCommand = "agent recovery.lifeline_command_body_canonicalization_diagnostic $recoveryLifelineCommandBodyCanonicalizationHash $recoveryLifelineCommandEnvelopeEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryCommandDispatchBoundaryId"

    Send-AgentCommand -Command $recoveryLifelineCommandBodyCommand -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_body_canonicalization_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_body_canonicalization_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_body_canonicalization_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_lifeline_command_body_canonicalization_records": true' },
        @{ Suffix = "retained_status"; Needle = '"status": "retained_hash_reference_command_still_denied"' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "retained_latest_event"; Needle = '"latest_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "argument_schema"; Needle = '"argument_schema": "raios.recovery_lifeline_command.status_args.v0"' },
        @{ Suffix = "target_locator"; Needle = "`"target_locator`": `"$recoveryCommandTargetLocator`"" },
        @{ Suffix = "boundary_id"; Needle = "`"command_dispatch_boundary_id`": `"$recoveryCommandDispatchBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "command_reference_hash"; Needle = "`"command_envelope_reference_hash`": `"sha256:$recoveryLifelineCommandEnvelopeReferenceHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLifelineCommandBodyResponse = Get-LastAgentResponseJson -Method "recovery.lifeline_command_body_canonicalization_diagnostic"
    $recoveryLifelineCommandBodyEventId = [string]$recoveryLifelineCommandBodyResponse.body.result.retained_command_body_canonicalization_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_lifeline_command_body_retained_reference_event_id_captured" -Value $recoveryLifelineCommandBodyEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_body_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_handler_binding_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_missing"; Needle = '"command_handler_binding_present": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "memory_writes_false"; Needle = '"recovery_memory_writes_enabled": false' },
        @{ Suffix = "durable_writes_false"; Needle = '"durable_writes_enabled": false' },
        @{ Suffix = "service_inventory_none"; Needle = '"service_inventory_change": "none"' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_handler_binding_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_handler_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_handler_binding_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_handler_binding_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_handler_binding_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_handler_binding_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.lifeline_command_handler_binding_diagnostic' },
        @{ Suffix = "handler_schema"; Needle = '"handler_binding_schema": "raios.recovery_lifeline_command_handler_binding.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"handler_binding_canonicalization": "raios.recovery_lifeline_command_handler_binding.canonical.v0"' },
        @{ Suffix = "handler_boundary"; Needle = '"handler_binding_boundary_id": "boundary.recovery_lifeline_command_handler_binding.current_boot"' },
        @{ Suffix = "status_handler_fact"; Needle = '"fact": "status_read_handler"' },
        @{ Suffix = "preview_auth_fact"; Needle = '"fact": "rollback_preview_authorization"' },
        @{ Suffix = "apply_auth_fact"; Needle = '"fact": "rollback_apply_authorization"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_command_handler_binding_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_handler_binding_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_handler_binding_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_handler_binding_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_command_handler_binding_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "handler_binding_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "handler_binding_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_handler_binding"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "handler_case"; Needle = '"case": "handler_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "handler_binding_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_body_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_handler_binding_still_non_executable"' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryCommandHandlerBindingBoundaryId = "boundary.recovery_lifeline_command_handler_binding.current_boot"
    $recoveryCommandHandlerInputCanonical = @(
        "schema=raios.recovery_lifeline_command_handler_input_binding.v0",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "handler_id=$recoveryCommandHandlerBindingBoundaryId",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryCommandHandlerInputBindingHash = Get-TextSha256 -Text $recoveryCommandHandlerInputCanonical
    $recoveryCommandHandlerBindingCanonical = @(
        "canonicalization=raios.recovery_lifeline_command_handler_binding.canonical.v0",
        "schema=raios.recovery_lifeline_command_handler_binding.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline_command_handler",
        "scope=current_boot",
        "retained_recovery_lifeline_command_body_canonicalization_event_id=$recoveryLifelineCommandBodyEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "handler_id=$recoveryCommandHandlerBindingBoundaryId",
        "handler_input_binding_sha256=$recoveryCommandHandlerInputBindingHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryCommandHandlerBindingHash = Get-TextSha256 -Text $recoveryCommandHandlerBindingCanonical
    $recoveryCommandHandlerBindingCommand = "agent recovery.lifeline_command_handler_binding_diagnostic $recoveryCommandHandlerBindingHash $recoveryLifelineCommandBodyEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandDispatchBoundaryId $recoveryCommandHandlerBindingBoundaryId $recoveryCommandHandlerInputBindingHash"

    Send-AgentCommand -Command $recoveryCommandHandlerBindingCommand -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_handler_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_handler_binding_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_command_handler_binding_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_lifeline_command_handler_binding_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "handler_id"; Needle = "`"handler_id`": `"$recoveryCommandHandlerBindingBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "input_hash"; Needle = "`"handler_input_binding_hash`": `"sha256:$recoveryCommandHandlerInputBindingHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryCommandHandlerBindingResponse = Get-LastAgentResponseJson -Method "recovery.lifeline_command_handler_binding_diagnostic"
    $recoveryCommandHandlerBindingEventId = [string]$recoveryCommandHandlerBindingResponse.body.result.retained_command_handler_binding_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_lifeline_command_handler_retained_reference_event_id_captured" -Value $recoveryCommandHandlerBindingEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_handler_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_status_read_handler_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_missing"; Needle = '"status_read_handler_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_status_read_handler_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_status_read_handler_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_status_read_handler_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_status_read_handler_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_status_read_handler_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_status_read_handler_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_status_execute"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.lifeline_status_read_handler_diagnostic' },
        @{ Suffix = "status_handler_schema"; Needle = '"status_read_handler_schema": "raios.recovery_lifeline_status_read_handler.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"status_read_handler_canonicalization": "raios.recovery_lifeline_status_read_handler.canonical.v0"' },
        @{ Suffix = "status_handler_boundary"; Needle = '"status_read_handler_boundary_id": "boundary.recovery_lifeline_status_read_handler.current_boot"' },
        @{ Suffix = "preview_auth_fact"; Needle = '"fact": "rollback_preview_authorization"' },
        @{ Suffix = "apply_auth_fact"; Needle = '"fact": "rollback_apply_authorization"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.lifeline_status_read_handler_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_status_read_handler_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_status_read_handler_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_status_read_handler_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_lifeline_status_read_handler_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "status_read_handler_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "status_read_handler_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_status_read_handler"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "handler_case"; Needle = '"case": "status_handler_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "status_read_handler_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_handler_binding_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_status_read_handler_still_non_executable"' },
        @{ Suffix = "status_execute_false"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryStatusReadHandlerBoundaryId = "boundary.recovery_lifeline_status_read_handler.current_boot"
    $recoveryStatusReadProjectionCanonical = @(
        "schema=raios.recovery_lifeline_status_read_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryStatusReadProjectionHash = Get-TextSha256 -Text $recoveryStatusReadProjectionCanonical
    $recoveryStatusReadHandlerCanonical = @(
        "canonicalization=raios.recovery_lifeline_status_read_handler.canonical.v0",
        "schema=raios.recovery_lifeline_status_read_handler.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_lifeline_status_read_handler",
        "scope=current_boot",
        "retained_recovery_lifeline_command_handler_binding_event_id=$recoveryCommandHandlerBindingEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "handler_binding_sha256=$recoveryCommandHandlerBindingHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "status_handler_id=$recoveryStatusReadHandlerBoundaryId",
        "status_read_projection_sha256=$recoveryStatusReadProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryStatusReadHandlerHash = Get-TextSha256 -Text $recoveryStatusReadHandlerCanonical
    $recoveryStatusReadHandlerCommand = "agent recovery.lifeline_status_read_handler_diagnostic $recoveryStatusReadHandlerHash $recoveryCommandHandlerBindingEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryCommandDispatchBoundaryId $recoveryStatusReadHandlerBoundaryId $recoveryStatusReadProjectionHash"

    Send-AgentCommand -Command $recoveryStatusReadHandlerCommand -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_status_read_handler_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_status_read_handler_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_lifeline_status_read_handler_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_lifeline_status_read_handler_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "status_handler_id"; Needle = "`"status_handler_id`": `"$recoveryStatusReadHandlerBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"status_read_projection_hash`": `"sha256:$recoveryStatusReadProjectionHash`"" },
        @{ Suffix = "status_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "status_execute_false"; Needle = '"executes_lifeline_status": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryStatusReadHandlerResponse = Get-LastAgentResponseJson -Method "recovery.lifeline_status_read_handler_diagnostic"
    $recoveryStatusReadHandlerEventId = [string]$recoveryStatusReadHandlerResponse.body.result.retained_status_read_handler_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_lifeline_status_read_handler_retained_reference_event_id_captured" -Value $recoveryStatusReadHandlerEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_status_handler_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_rollback_preview_authorization_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_present"; Needle = '"status_read_handler_present": true' },
        @{ Suffix = "preview_auth_missing"; Needle = '"rollback_preview_authorization_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.rollback_preview_authorization_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_preview_authorization_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_preview_authorization_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_rollback_preview_authorization_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_rollback_preview_authorization_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_rollback_preview_authorization_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_preview"; Needle = '"executes_rollback_preview": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.rollback_preview_authorization_diagnostic' },
        @{ Suffix = "preview_schema"; Needle = '"rollback_preview_authorization_schema": "raios.recovery_rollback_preview_authorization.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"rollback_preview_authorization_canonicalization": "raios.recovery_rollback_preview_authorization.canonical.v0"' },
        @{ Suffix = "preview_boundary"; Needle = '"rollback_preview_authorization_boundary_id": "boundary.recovery_rollback_preview_authorization.current_boot"' },
        @{ Suffix = "apply_auth_fact"; Needle = '"fact": "rollback_apply_authorization"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.rollback_preview_authorization_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_preview_authorization_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_preview_authorization_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_rollback_preview_authorization_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_rollback_preview_authorization_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "rollback_preview_authorization_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "rollback_preview_authorization_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_rollback_preview_authorization"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "authorization_case"; Needle = '"case": "rollback_preview_authorization_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "rollback_preview_authorization_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_status_read_handler_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_rollback_preview_authorization_still_non_executable"' },
        @{ Suffix = "preview_false"; Needle = '"executes_rollback_preview": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRollbackPreviewAuthorizationBoundaryId = "boundary.recovery_rollback_preview_authorization.current_boot"
    $recoveryRollbackPreviewProjectionCanonical = @(
        "schema=raios.recovery_rollback_preview_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryRollbackPreviewProjectionHash = Get-TextSha256 -Text $recoveryRollbackPreviewProjectionCanonical
    $recoveryRollbackPreviewAuthorizationCanonical = @(
        "canonicalization=raios.recovery_rollback_preview_authorization.canonical.v0",
        "schema=raios.recovery_rollback_preview_authorization.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_rollback_preview_authorization",
        "scope=current_boot",
        "retained_recovery_lifeline_status_read_handler_event_id=$recoveryStatusReadHandlerEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "handler_binding_sha256=$recoveryCommandHandlerBindingHash",
        "status_read_handler_sha256=$recoveryStatusReadHandlerHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "rollback_preview_authorization_id=$recoveryRollbackPreviewAuthorizationBoundaryId",
        "rollback_preview_projection_sha256=$recoveryRollbackPreviewProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryRollbackPreviewAuthorizationHash = Get-TextSha256 -Text $recoveryRollbackPreviewAuthorizationCanonical
    $recoveryRollbackPreviewAuthorizationCommand = "agent recovery.rollback_preview_authorization_diagnostic $recoveryRollbackPreviewAuthorizationHash $recoveryStatusReadHandlerEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryCommandDispatchBoundaryId $recoveryRollbackPreviewAuthorizationBoundaryId $recoveryRollbackPreviewProjectionHash"

    Send-AgentCommand -Command $recoveryRollbackPreviewAuthorizationCommand -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_preview_authorization_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_preview_authorization_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_rollback_preview_authorization_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_rollback_preview_authorization_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "preview_authorization_id"; Needle = "`"rollback_preview_authorization_id`": `"$recoveryRollbackPreviewAuthorizationBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "status_handler_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"rollback_preview_projection_hash`": `"sha256:$recoveryRollbackPreviewProjectionHash`"" },
        @{ Suffix = "preview_hash"; Needle = "`"rollback_preview_authorization_hash`": `"sha256:$recoveryRollbackPreviewAuthorizationHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "preview_false"; Needle = '"executes_rollback_preview": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRollbackPreviewAuthorizationResponse = Get-LastAgentResponseJson -Method "recovery.rollback_preview_authorization_diagnostic"
    $recoveryRollbackPreviewAuthorizationEventId = [string]$recoveryRollbackPreviewAuthorizationResponse.body.result.retained_rollback_preview_authorization_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_rollback_preview_authorization_retained_reference_event_id_captured" -Value $recoveryRollbackPreviewAuthorizationEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_preview_auth_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_rollback_apply_authorization_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_present"; Needle = '"status_read_handler_present": true' },
        @{ Suffix = "preview_auth_present"; Needle = '"rollback_preview_authorization_present": true' },
        @{ Suffix = "apply_auth_missing"; Needle = '"rollback_apply_authorization_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.rollback_apply_authorization_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_apply_authorization_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_apply_authorization_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_rollback_apply_authorization_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_rollback_apply_authorization_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_rollback_apply_authorization_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_apply"; Needle = '"executes_rollback_apply": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.rollback_apply_authorization_diagnostic' },
        @{ Suffix = "apply_schema"; Needle = '"rollback_apply_authorization_schema": "raios.recovery_rollback_apply_authorization.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"rollback_apply_authorization_canonicalization": "raios.recovery_rollback_apply_authorization.canonical.v0"' },
        @{ Suffix = "apply_boundary"; Needle = '"rollback_apply_authorization_boundary_id": "boundary.recovery_rollback_apply_authorization.current_boot"' },
        @{ Suffix = "disable_target_fact"; Needle = '"fact": "disable_module_target_binding"' },
        @{ Suffix = "restart_target_fact"; Needle = '"fact": "restart_last_good_target_binding"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.rollback_apply_authorization_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_apply_authorization_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_apply_authorization_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_rollback_apply_authorization_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_rollback_apply_authorization_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "rollback_apply_authorization_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "rollback_apply_authorization_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_rollback_apply_authorization"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "authorization_case"; Needle = '"case": "rollback_apply_authorization_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "rollback_apply_authorization_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_rollback_preview_authorization_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_rollback_apply_authorization_still_non_executable"' },
        @{ Suffix = "apply_false"; Needle = '"executes_rollback_apply": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRollbackApplyAuthorizationBoundaryId = "boundary.recovery_rollback_apply_authorization.current_boot"
    $recoveryRollbackApplyProjectionCanonical = @(
        "schema=raios.recovery_rollback_apply_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "rollback_preview_authorization_hash=$recoveryRollbackPreviewAuthorizationHash",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryRollbackApplyProjectionHash = Get-TextSha256 -Text $recoveryRollbackApplyProjectionCanonical
    $recoveryRollbackApplyAuthorizationCanonical = @(
        "canonicalization=raios.recovery_rollback_apply_authorization.canonical.v0",
        "schema=raios.recovery_rollback_apply_authorization.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_rollback_apply_authorization",
        "scope=current_boot",
        "retained_recovery_rollback_preview_authorization_event_id=$recoveryRollbackPreviewAuthorizationEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "handler_binding_sha256=$recoveryCommandHandlerBindingHash",
        "status_read_handler_sha256=$recoveryStatusReadHandlerHash",
        "rollback_preview_authorization_sha256=$recoveryRollbackPreviewAuthorizationHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "rollback_apply_authorization_id=$recoveryRollbackApplyAuthorizationBoundaryId",
        "rollback_apply_projection_sha256=$recoveryRollbackApplyProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryRollbackApplyAuthorizationHash = Get-TextSha256 -Text $recoveryRollbackApplyAuthorizationCanonical
    $recoveryRollbackApplyAuthorizationCommand = "agent recovery.rollback_apply_authorization_diagnostic $recoveryRollbackApplyAuthorizationHash $recoveryRollbackPreviewAuthorizationEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryRollbackPreviewAuthorizationHash $recoveryCommandDispatchBoundaryId $recoveryRollbackApplyAuthorizationBoundaryId $recoveryRollbackApplyProjectionHash"

    Send-AgentCommand -Command $recoveryRollbackApplyAuthorizationCommand -ExpectedMarker "RAIOS_AGENT_END recovery.rollback_apply_authorization_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_rollback_apply_authorization_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_rollback_apply_authorization_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_rollback_apply_authorization_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "apply_authorization_id"; Needle = "`"rollback_apply_authorization_id`": `"$recoveryRollbackApplyAuthorizationBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "status_handler_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "preview_hash"; Needle = "`"rollback_preview_authorization_hash`": `"sha256:$recoveryRollbackPreviewAuthorizationHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"rollback_apply_projection_hash`": `"sha256:$recoveryRollbackApplyProjectionHash`"" },
        @{ Suffix = "apply_hash"; Needle = "`"rollback_apply_authorization_hash`": `"sha256:$recoveryRollbackApplyAuthorizationHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "apply_false"; Needle = '"executes_rollback_apply": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRollbackApplyAuthorizationResponse = Get-LastAgentResponseJson -Method "recovery.rollback_apply_authorization_diagnostic"
    $recoveryRollbackApplyAuthorizationEventId = [string]$recoveryRollbackApplyAuthorizationResponse.body.result.retained_rollback_apply_authorization_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_rollback_apply_authorization_retained_reference_event_id_captured" -Value $recoveryRollbackApplyAuthorizationEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_apply_auth_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_disable_module_target_binding_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_present"; Needle = '"status_read_handler_present": true' },
        @{ Suffix = "preview_auth_present"; Needle = '"rollback_preview_authorization_present": true' },
        @{ Suffix = "apply_auth_present"; Needle = '"rollback_apply_authorization_present": true' },
        @{ Suffix = "disable_target_missing"; Needle = '"disable_module_target_binding_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.disable_module_target_binding_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.disable_module_target_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_disable_module_target_binding_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_disable_module_target_binding_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_disable_module_target_binding_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_disable_module_target_binding_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_disable"; Needle = '"disables_module": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.disable_module_target_binding_diagnostic' },
        @{ Suffix = "disable_schema"; Needle = '"disable_module_target_binding_schema": "raios.recovery_disable_module_target_binding.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"disable_module_target_binding_canonicalization": "raios.recovery_disable_module_target_binding.canonical.v0"' },
        @{ Suffix = "disable_boundary"; Needle = '"disable_module_target_binding_boundary_id": "boundary.recovery_disable_module_target_binding.current_boot"' },
        @{ Suffix = "restart_target_fact"; Needle = '"fact": "restart_last_good_target_binding"' },
        @{ Suffix = "load_target_fact"; Needle = '"fact": "load_artifact_by_hash_target_binding"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.disable_module_target_binding_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.disable_module_target_binding_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_disable_module_target_binding_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_disable_module_target_binding_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_disable_module_target_binding_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "disable_module_target_binding_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "disable_module_target_binding_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_disable_module_target_binding"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "target_case"; Needle = '"case": "disable_module_target_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "disable_module_target_binding_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_rollback_apply_authorization_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_disable_module_target_binding_still_non_executable"' },
        @{ Suffix = "disable_false"; Needle = '"disables_module": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryDisableModuleTargetBindingBoundaryId = "boundary.recovery_disable_module_target_binding.current_boot"
    $recoveryDisableModuleTargetProjectionCanonical = @(
        "schema=raios.recovery_disable_module_target_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "rollback_apply_authorization_hash=$recoveryRollbackApplyAuthorizationHash",
        "rollback_preview_authorization_hash=$recoveryRollbackPreviewAuthorizationHash",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryDisableModuleTargetProjectionHash = Get-TextSha256 -Text $recoveryDisableModuleTargetProjectionCanonical
    $recoveryDisableModuleTargetBindingCanonical = @(
        "canonicalization=raios.recovery_disable_module_target_binding.canonical.v0",
        "schema=raios.recovery_disable_module_target_binding.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_disable_module_target_binding",
        "scope=current_boot",
        "retained_recovery_rollback_apply_authorization_event_id=$recoveryRollbackApplyAuthorizationEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "handler_binding_sha256=$recoveryCommandHandlerBindingHash",
        "status_read_handler_sha256=$recoveryStatusReadHandlerHash",
        "rollback_preview_authorization_sha256=$recoveryRollbackPreviewAuthorizationHash",
        "rollback_apply_authorization_sha256=$recoveryRollbackApplyAuthorizationHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "disable_module_target_id=$recoveryDisableModuleTargetBindingBoundaryId",
        "disable_module_target_projection_sha256=$recoveryDisableModuleTargetProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "disables_module=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryDisableModuleTargetBindingHash = Get-TextSha256 -Text $recoveryDisableModuleTargetBindingCanonical
    $recoveryDisableModuleTargetBindingCommand = "agent recovery.disable_module_target_binding_diagnostic $recoveryDisableModuleTargetBindingHash $recoveryRollbackApplyAuthorizationEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryRollbackPreviewAuthorizationHash $recoveryRollbackApplyAuthorizationHash $recoveryCommandDispatchBoundaryId $recoveryDisableModuleTargetBindingBoundaryId $recoveryDisableModuleTargetProjectionHash"

    Send-AgentCommand -Command $recoveryDisableModuleTargetBindingCommand -ExpectedMarker "RAIOS_AGENT_END recovery.disable_module_target_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_disable_module_target_binding_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_disable_module_target_binding_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_disable_module_target_binding_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "disable_target_id"; Needle = "`"disable_module_target_id`": `"$recoveryDisableModuleTargetBindingBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "status_handler_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "preview_hash"; Needle = "`"rollback_preview_authorization_hash`": `"sha256:$recoveryRollbackPreviewAuthorizationHash`"" },
        @{ Suffix = "apply_hash"; Needle = "`"rollback_apply_authorization_hash`": `"sha256:$recoveryRollbackApplyAuthorizationHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"disable_module_target_projection_hash`": `"sha256:$recoveryDisableModuleTargetProjectionHash`"" },
        @{ Suffix = "binding_hash"; Needle = "`"disable_module_target_binding_hash`": `"sha256:$recoveryDisableModuleTargetBindingHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "disable_false"; Needle = '"disables_module": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryDisableModuleTargetBindingResponse = Get-LastAgentResponseJson -Method "recovery.disable_module_target_binding_diagnostic"
    $recoveryDisableModuleTargetBindingEventId = [string]$recoveryDisableModuleTargetBindingResponse.body.result.retained_disable_module_target_binding_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_disable_module_target_binding_retained_reference_event_id_captured" -Value $recoveryDisableModuleTargetBindingEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_disable_target_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_restart_last_good_target_binding_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_present"; Needle = '"status_read_handler_present": true' },
        @{ Suffix = "preview_auth_present"; Needle = '"rollback_preview_authorization_present": true' },
        @{ Suffix = "apply_auth_present"; Needle = '"rollback_apply_authorization_present": true' },
        @{ Suffix = "disable_target_present"; Needle = '"disable_module_target_binding_present": true' },
        @{ Suffix = "restart_target_missing"; Needle = '"restart_last_good_target_binding_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.restart_last_good_target_binding_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.restart_last_good_target_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_restart_last_good_target_binding_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_restart_last_good_target_binding_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_restart_last_good_target_binding_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_restart_last_good_target_binding_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_restart"; Needle = '"restarts_last_good": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.restart_last_good_target_binding_diagnostic' },
        @{ Suffix = "restart_schema"; Needle = '"restart_last_good_target_binding_schema": "raios.recovery_restart_last_good_target_binding.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"restart_last_good_target_binding_canonicalization": "raios.recovery_restart_last_good_target_binding.canonical.v0"' },
        @{ Suffix = "restart_boundary"; Needle = '"restart_last_good_target_binding_boundary_id": "boundary.recovery_restart_last_good_target_binding.current_boot"' },
        @{ Suffix = "load_target_fact"; Needle = '"fact": "load_artifact_by_hash_target_binding"' },
        @{ Suffix = "memory_authority_fact"; Needle = '"fact": "recovery_memory_write_authority"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.restart_last_good_target_binding_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.restart_last_good_target_binding_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_restart_last_good_target_binding_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_restart_last_good_target_binding_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_restart_last_good_target_binding_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "restart_last_good_target_binding_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "restart_last_good_target_binding_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_restart_last_good_target_binding"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "target_case"; Needle = '"case": "restart_last_good_target_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "restart_last_good_target_binding_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_disable_module_target_binding_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_restart_last_good_target_binding_still_non_executable"' },
        @{ Suffix = "restart_false"; Needle = '"restarts_last_good": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRestartLastGoodTargetBindingBoundaryId = "boundary.recovery_restart_last_good_target_binding.current_boot"
    $recoveryRestartLastGoodTargetProjectionCanonical = @(
        "schema=raios.recovery_restart_last_good_target_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "disable_module_target_binding_hash=$recoveryDisableModuleTargetBindingHash",
        "rollback_apply_authorization_hash=$recoveryRollbackApplyAuthorizationHash",
        "rollback_preview_authorization_hash=$recoveryRollbackPreviewAuthorizationHash",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryRestartLastGoodTargetProjectionHash = Get-TextSha256 -Text $recoveryRestartLastGoodTargetProjectionCanonical
    $recoveryRestartLastGoodTargetBindingCanonical = @(
        "canonicalization=raios.recovery_restart_last_good_target_binding.canonical.v0",
        "schema=raios.recovery_restart_last_good_target_binding.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_restart_last_good_target_binding",
        "scope=current_boot",
        "retained_recovery_disable_module_target_binding_event_id=$recoveryDisableModuleTargetBindingEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "handler_binding_sha256=$recoveryCommandHandlerBindingHash",
        "status_read_handler_sha256=$recoveryStatusReadHandlerHash",
        "rollback_preview_authorization_sha256=$recoveryRollbackPreviewAuthorizationHash",
        "rollback_apply_authorization_sha256=$recoveryRollbackApplyAuthorizationHash",
        "disable_module_target_binding_sha256=$recoveryDisableModuleTargetBindingHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "restart_last_good_target_id=$recoveryRestartLastGoodTargetBindingBoundaryId",
        "restart_last_good_target_projection_sha256=$recoveryRestartLastGoodTargetProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "restarts_last_good=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "disables_module=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "loads_recovery_artifact=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryRestartLastGoodTargetBindingHash = Get-TextSha256 -Text $recoveryRestartLastGoodTargetBindingCanonical
    $recoveryRestartLastGoodTargetBindingCommand = "agent recovery.restart_last_good_target_binding_diagnostic $recoveryRestartLastGoodTargetBindingHash $recoveryDisableModuleTargetBindingEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryRollbackPreviewAuthorizationHash $recoveryRollbackApplyAuthorizationHash $recoveryDisableModuleTargetBindingHash $recoveryCommandDispatchBoundaryId $recoveryRestartLastGoodTargetBindingBoundaryId $recoveryRestartLastGoodTargetProjectionHash"

    Send-AgentCommand -Command $recoveryRestartLastGoodTargetBindingCommand -ExpectedMarker "RAIOS_AGENT_END recovery.restart_last_good_target_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_restart_last_good_target_binding_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_restart_last_good_target_binding_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_restart_last_good_target_binding_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "restart_target_id"; Needle = "`"restart_last_good_target_id`": `"$recoveryRestartLastGoodTargetBindingBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "status_handler_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "preview_hash"; Needle = "`"rollback_preview_authorization_hash`": `"sha256:$recoveryRollbackPreviewAuthorizationHash`"" },
        @{ Suffix = "apply_hash"; Needle = "`"rollback_apply_authorization_hash`": `"sha256:$recoveryRollbackApplyAuthorizationHash`"" },
        @{ Suffix = "disable_hash"; Needle = "`"disable_module_target_binding_hash`": `"sha256:$recoveryDisableModuleTargetBindingHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"restart_last_good_target_projection_hash`": `"sha256:$recoveryRestartLastGoodTargetProjectionHash`"" },
        @{ Suffix = "binding_hash"; Needle = "`"restart_last_good_target_binding_hash`": `"sha256:$recoveryRestartLastGoodTargetBindingHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "restart_false"; Needle = '"restarts_last_good": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryRestartLastGoodTargetBindingResponse = Get-LastAgentResponseJson -Method "recovery.restart_last_good_target_binding_diagnostic"
    $recoveryRestartLastGoodTargetBindingEventId = [string]$recoveryRestartLastGoodTargetBindingResponse.body.result.retained_restart_last_good_target_binding_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_restart_last_good_target_binding_retained_reference_event_id_captured" -Value $recoveryRestartLastGoodTargetBindingEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_restart_target_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_load_artifact_by_hash_target_binding_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_present"; Needle = '"status_read_handler_present": true' },
        @{ Suffix = "preview_auth_present"; Needle = '"rollback_preview_authorization_present": true' },
        @{ Suffix = "apply_auth_present"; Needle = '"rollback_apply_authorization_present": true' },
        @{ Suffix = "disable_target_present"; Needle = '"disable_module_target_binding_present": true' },
        @{ Suffix = "restart_target_present"; Needle = '"restart_last_good_target_binding_present": true' },
        @{ Suffix = "load_hash_target_missing"; Needle = '"load_artifact_by_hash_target_binding_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.load_artifact_by_hash_target_binding_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.load_artifact_by_hash_target_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_load_artifact_by_hash_target_binding_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_load_artifact_by_hash_target_binding_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_load_artifact_by_hash_target_binding_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_load_artifact_by_hash_target_binding_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_load"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "no_authorize"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.load_artifact_by_hash_target_binding_diagnostic' },
        @{ Suffix = "load_schema"; Needle = '"load_artifact_by_hash_target_binding_schema": "raios.recovery_load_artifact_by_hash_target_binding.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"load_artifact_by_hash_target_binding_canonicalization": "raios.recovery_load_artifact_by_hash_target_binding.canonical.v0"' },
        @{ Suffix = "load_boundary"; Needle = '"load_artifact_by_hash_target_binding_boundary_id": "boundary.recovery_load_artifact_by_hash_target_binding.current_boot"' },
        @{ Suffix = "memory_authority_fact"; Needle = '"fact": "recovery_memory_write_authority"' },
        @{ Suffix = "durable_authority_fact"; Needle = '"fact": "durable_audit_rollback_write_authority"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.load_artifact_by_hash_target_binding_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.load_artifact_by_hash_target_binding_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_load_artifact_by_hash_target_binding_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_load_artifact_by_hash_target_binding_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_load_artifact_by_hash_target_binding_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "load_artifact_by_hash_target_binding_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "load_artifact_by_hash_target_binding_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_load_artifact_by_hash_target_binding"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "target_case"; Needle = '"case": "load_artifact_by_hash_target_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "load_artifact_by_hash_target_binding_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_restart_last_good_target_binding_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_load_artifact_by_hash_target_binding_still_non_executable"' },
        @{ Suffix = "load_false"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLoadArtifactByHashTargetBindingBoundaryId = "boundary.recovery_load_artifact_by_hash_target_binding.current_boot"
    $recoveryLoadArtifactByHashTargetProjectionCanonical = @(
        "schema=raios.recovery_load_artifact_by_hash_target_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "artifact_hash=$recoveryArtifactHash",
        "restart_last_good_target_binding_hash=$recoveryRestartLastGoodTargetBindingHash",
        "disable_module_target_binding_hash=$recoveryDisableModuleTargetBindingHash",
        "rollback_apply_authorization_hash=$recoveryRollbackApplyAuthorizationHash",
        "rollback_preview_authorization_hash=$recoveryRollbackPreviewAuthorizationHash",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryLoadArtifactByHashTargetProjectionHash = Get-TextSha256 -Text $recoveryLoadArtifactByHashTargetProjectionCanonical
    $recoveryLoadArtifactByHashTargetBindingCanonical = @(
        "canonicalization=raios.recovery_load_artifact_by_hash_target_binding.canonical.v0",
        "schema=raios.recovery_load_artifact_by_hash_target_binding.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_load_artifact_by_hash_target_binding",
        "scope=current_boot",
        "retained_recovery_restart_last_good_target_binding_event_id=$recoveryRestartLastGoodTargetBindingEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "handler_binding_sha256=$recoveryCommandHandlerBindingHash",
        "status_read_handler_sha256=$recoveryStatusReadHandlerHash",
        "rollback_preview_authorization_sha256=$recoveryRollbackPreviewAuthorizationHash",
        "rollback_apply_authorization_sha256=$recoveryRollbackApplyAuthorizationHash",
        "disable_module_target_binding_sha256=$recoveryDisableModuleTargetBindingHash",
        "restart_last_good_target_binding_sha256=$recoveryRestartLastGoodTargetBindingHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "load_artifact_by_hash_target_id=$recoveryLoadArtifactByHashTargetBindingBoundaryId",
        "load_artifact_by_hash_target_artifact_sha256=$recoveryArtifactHash",
        "load_artifact_by_hash_target_projection_sha256=$recoveryLoadArtifactByHashTargetProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "loads_recovery_artifact=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "disables_module=false",
        "restarts_last_good=false",
        "writes_recovery_memory=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryLoadArtifactByHashTargetBindingHash = Get-TextSha256 -Text $recoveryLoadArtifactByHashTargetBindingCanonical
    $recoveryLoadArtifactByHashTargetBindingCommand = "agent recovery.load_artifact_by_hash_target_binding_diagnostic $recoveryLoadArtifactByHashTargetBindingHash $recoveryRestartLastGoodTargetBindingEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryRollbackPreviewAuthorizationHash $recoveryRollbackApplyAuthorizationHash $recoveryDisableModuleTargetBindingHash $recoveryRestartLastGoodTargetBindingHash $recoveryCommandDispatchBoundaryId $recoveryLoadArtifactByHashTargetBindingBoundaryId $recoveryArtifactHash $recoveryLoadArtifactByHashTargetProjectionHash"

    Send-AgentCommand -Command $recoveryLoadArtifactByHashTargetBindingCommand -ExpectedMarker "RAIOS_AGENT_END recovery.load_artifact_by_hash_target_binding_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_load_artifact_by_hash_target_binding_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_load_artifact_by_hash_target_binding_valid_but_command_dispatch_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_load_artifact_by_hash_target_binding_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "load_target_id"; Needle = "`"load_artifact_by_hash_target_id`": `"$recoveryLoadArtifactByHashTargetBindingBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "status_handler_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "preview_hash"; Needle = "`"rollback_preview_authorization_hash`": `"sha256:$recoveryRollbackPreviewAuthorizationHash`"" },
        @{ Suffix = "apply_hash"; Needle = "`"rollback_apply_authorization_hash`": `"sha256:$recoveryRollbackApplyAuthorizationHash`"" },
        @{ Suffix = "disable_hash"; Needle = "`"disable_module_target_binding_hash`": `"sha256:$recoveryDisableModuleTargetBindingHash`"" },
        @{ Suffix = "restart_hash"; Needle = "`"restart_last_good_target_binding_hash`": `"sha256:$recoveryRestartLastGoodTargetBindingHash`"" },
        @{ Suffix = "artifact_hash"; Needle = "`"load_artifact_by_hash_target_artifact_hash`": `"sha256:$recoveryArtifactHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"load_artifact_by_hash_target_projection_hash`": `"sha256:$recoveryLoadArtifactByHashTargetProjectionHash`"" },
        @{ Suffix = "binding_hash"; Needle = "`"load_artifact_by_hash_target_binding_hash`": `"sha256:$recoveryLoadArtifactByHashTargetBindingHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "load_false"; Needle = '"loads_recovery_artifact": false' },
        @{ Suffix = "no_authorize"; Needle = '"authorizes_recovery_load": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryLoadArtifactByHashTargetBindingResponse = Get-LastAgentResponseJson -Method "recovery.load_artifact_by_hash_target_binding_diagnostic"
    $recoveryLoadArtifactByHashTargetBindingEventId = [string]$recoveryLoadArtifactByHashTargetBindingResponse.body.result.retained_load_artifact_by_hash_target_binding_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_load_artifact_by_hash_target_binding_retained_reference_event_id_captured" -Value $recoveryLoadArtifactByHashTargetBindingEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_load_hash_target_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_memory_write_authority_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_present"; Needle = '"status_read_handler_present": true' },
        @{ Suffix = "preview_auth_present"; Needle = '"rollback_preview_authorization_present": true' },
        @{ Suffix = "apply_auth_present"; Needle = '"rollback_apply_authorization_present": true' },
        @{ Suffix = "disable_target_present"; Needle = '"disable_module_target_binding_present": true' },
        @{ Suffix = "restart_target_present"; Needle = '"restart_last_good_target_binding_present": true' },
        @{ Suffix = "load_hash_target_present"; Needle = '"load_artifact_by_hash_target_binding_present": true' },
        @{ Suffix = "memory_authority_missing"; Needle = '"recovery_memory_write_authority_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.memory_write_authority_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.memory_write_authority_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_memory_write_authority_absent_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_memory_write_authority_diagnostic.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "status"; Needle = '"status": "missing"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_memory_write_authority_absent"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_memory_write_authority_records": false' },
        @{ Suffix = "no_raw_body"; Needle = '"accepts_raw_command_body": false' },
        @{ Suffix = "no_command_body"; Needle = '"accepts_lifeline_command_body": false' },
        @{ Suffix = "no_write"; Needle = '"writes_recovery_memory": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "reference_format"; Needle = '"reference_format": "recovery.memory_write_authority_diagnostic' },
        @{ Suffix = "memory_schema"; Needle = '"recovery_memory_write_authority_schema": "raios.recovery_memory_write_authority.v0"' },
        @{ Suffix = "canonicalization"; Needle = '"recovery_memory_write_authority_canonicalization": "raios.recovery_memory_write_authority.canonical.v0"' },
        @{ Suffix = "memory_boundary"; Needle = '"recovery_memory_write_authority_boundary_id": "boundary.recovery_memory_write_authority.current_boot"' },
        @{ Suffix = "durable_authority_fact"; Needle = '"fact": "durable_audit_rollback_write_authority"' },
        @{ Suffix = "service_boundary_fact"; Needle = '"fact": "service_inventory_side_effect_boundary"' },
        @{ Suffix = "valid_false"; Needle = '"valid_hash_reference": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    Send-AgentCommand -Command "agent recovery.memory_write_authority_diagnostic_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.memory_write_authority_diagnostic_selftest"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_memory_write_authority_selftest_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_memory_write_authority_selftest.v0"' },
        @{ Suffix = "local_only"; Needle = '"classification": "local_only"' },
        @{ Suffix = "no_mutation"; Needle = '"mutates_global_event_log": false' },
        @{ Suffix = "no_records"; Needle = '"creates_retained_recovery_memory_write_authority_records": false' },
        @{ Suffix = "case_count"; Needle = '"case_count": 10' },
        @{ Suffix = "passed"; Needle = '"passed": true' },
        @{ Suffix = "absent_case"; Needle = '"case": "recovery_memory_write_authority_absent"' },
        @{ Suffix = "arity_case"; Needle = '"case": "recovery_memory_write_authority_arity_invalid"' },
        @{ Suffix = "previous_case"; Needle = '"case": "previous_boot_recovery_memory_write_authority"' },
        @{ Suffix = "unsupported_case"; Needle = '"case": "unsupported_command_id"' },
        @{ Suffix = "schema_case"; Needle = '"case": "argument_schema_mismatch"' },
        @{ Suffix = "boundary_case"; Needle = '"case": "dispatch_boundary_mismatch"' },
        @{ Suffix = "authority_case"; Needle = '"case": "recovery_memory_write_authority_id_mismatch"' },
        @{ Suffix = "hash_case"; Needle = '"case": "recovery_memory_write_authority_hash_mismatch"' },
        @{ Suffix = "live_missing_case"; Needle = '"case": "retained_load_artifact_by_hash_target_binding_reference_missing"' },
        @{ Suffix = "non_executable_case"; Needle = '"case": "all_inputs_present_recovery_memory_write_authority_still_non_executable"' },
        @{ Suffix = "write_false"; Needle = '"writes_recovery_memory": false' },
        @{ Suffix = "dispatch_false"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryMemoryWriteAuthorityBoundaryId = "boundary.recovery_memory_write_authority.current_boot"
    $recoveryMemoryProjectionCanonical = @(
        "schema=raios.recovery_memory_write_projection.v0",
        "command_id=recovery.lifeline.status",
        "target_locator=$recoveryCommandTargetLocator",
        "load_artifact_by_hash_target_binding_hash=$recoveryLoadArtifactByHashTargetBindingHash",
        "restart_last_good_target_binding_hash=$recoveryRestartLastGoodTargetBindingHash",
        "disable_module_target_binding_hash=$recoveryDisableModuleTargetBindingHash",
        "rollback_apply_authorization_hash=$recoveryRollbackApplyAuthorizationHash",
        "rollback_preview_authorization_hash=$recoveryRollbackPreviewAuthorizationHash",
        "status_read_handler_hash=$recoveryStatusReadHandlerHash",
        "handler_binding_hash=$recoveryCommandHandlerBindingHash",
        "body_hash=$recoveryLifelineCommandBodyCanonicalizationHash"
    ) -join "`n"
    $recoveryMemoryProjectionHash = Get-TextSha256 -Text $recoveryMemoryProjectionCanonical
    $recoveryMemoryWriteAuthorityCanonical = @(
        "canonicalization=raios.recovery_memory_write_authority.canonical.v0",
        "schema=raios.recovery_memory_write_authority.v0",
        "load_mode=recovery_only",
        "subject=agent.session.serial",
        "resource=recovery_memory_write_authority",
        "scope=current_boot",
        "retained_recovery_load_artifact_by_hash_target_binding_event_id=$recoveryLoadArtifactByHashTargetBindingEventId",
        "command_id=recovery.lifeline.status",
        "argument_schema=raios.recovery_lifeline_command.status_args.v0",
        "argument_sha256=$recoveryLifelineStatusArgumentHash",
        "target_locator=$recoveryCommandTargetLocator",
        "command_envelope_reference_sha256=$recoveryLifelineCommandEnvelopeReferenceHash",
        "command_body_canonicalization_sha256=$recoveryLifelineCommandBodyCanonicalizationHash",
        "handler_binding_sha256=$recoveryCommandHandlerBindingHash",
        "status_read_handler_sha256=$recoveryStatusReadHandlerHash",
        "rollback_preview_authorization_sha256=$recoveryRollbackPreviewAuthorizationHash",
        "rollback_apply_authorization_sha256=$recoveryRollbackApplyAuthorizationHash",
        "disable_module_target_binding_sha256=$recoveryDisableModuleTargetBindingHash",
        "restart_last_good_target_binding_sha256=$recoveryRestartLastGoodTargetBindingHash",
        "load_artifact_by_hash_target_binding_sha256=$recoveryLoadArtifactByHashTargetBindingHash",
        "command_dispatch_boundary_id=$recoveryCommandDispatchBoundaryId",
        "recovery_memory_write_authority_id=$recoveryMemoryWriteAuthorityBoundaryId",
        "recovery_memory_projection_sha256=$recoveryMemoryProjectionHash",
        "accepts_raw_command_body=false",
        "accepts_lifeline_command_body=false",
        "accepts_lifeline_command_envelope=false",
        "dispatches_lifeline_command=false",
        "writes_recovery_memory=false",
        "loads_recovery_artifact=false",
        "executes_lifeline_status=false",
        "executes_rollback_preview=false",
        "executes_rollback_apply=false",
        "disables_module=false",
        "restarts_last_good=false",
        "writes_durable_audit_log=false",
        "writes_rollback_store=false",
        "exports_provider_context=false",
        "authorizes_recovery_load=false",
        "creates_durable_records=false",
        "installs_rollback_plan=false",
        "allocates_service_slot=false",
        "service_inventory_change=none",
        "load_attempted=false"
    ) -join "`n"
    $recoveryMemoryWriteAuthorityHash = Get-TextSha256 -Text $recoveryMemoryWriteAuthorityCanonical
    $recoveryMemoryWriteAuthorityCommand = "agent recovery.memory_write_authority_diagnostic $recoveryMemoryWriteAuthorityHash $recoveryLoadArtifactByHashTargetBindingEventId recovery.lifeline.status raios.recovery_lifeline_command.status_args.v0 $recoveryLifelineStatusArgumentHash $recoveryCommandTargetLocator $recoveryLifelineCommandEnvelopeReferenceHash $recoveryLifelineCommandBodyCanonicalizationHash $recoveryCommandHandlerBindingHash $recoveryStatusReadHandlerHash $recoveryRollbackPreviewAuthorizationHash $recoveryRollbackApplyAuthorizationHash $recoveryDisableModuleTargetBindingHash $recoveryRestartLastGoodTargetBindingHash $recoveryLoadArtifactByHashTargetBindingHash $recoveryCommandDispatchBoundaryId $recoveryMemoryWriteAuthorityBoundaryId $recoveryMemoryProjectionHash"

    Send-AgentCommand -Command $recoveryMemoryWriteAuthorityCommand -ExpectedMarker "RAIOS_AGENT_END recovery.memory_write_authority_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_memory_write_authority_valid_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "status"; Needle = '"status": "valid_hash_reference_command_still_denied"' },
        @{ Suffix = "reason"; Needle = '"reason": "recovery_memory_write_authority_valid_but_memory_writes_disabled"' },
        @{ Suffix = "retention_mutation"; Needle = '"global_event_log_mutation": "valid_hash_reference_retention_only"' },
        @{ Suffix = "creates_record"; Needle = '"creates_retained_recovery_memory_write_authority_records": true' },
        @{ Suffix = "recorded_event_id"; Needle = '"recorded_event_id": "event.current_boot.' },
        @{ Suffix = "command_id"; Needle = '"command_id": "recovery.lifeline.status"' },
        @{ Suffix = "memory_authority_id"; Needle = "`"recovery_memory_write_authority_id`": `"$recoveryMemoryWriteAuthorityBoundaryId`"" },
        @{ Suffix = "argument_hash"; Needle = "`"argument_hash`": `"sha256:$recoveryLifelineStatusArgumentHash`"" },
        @{ Suffix = "body_hash"; Needle = "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" },
        @{ Suffix = "handler_hash"; Needle = "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" },
        @{ Suffix = "status_handler_hash"; Needle = "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" },
        @{ Suffix = "preview_hash"; Needle = "`"rollback_preview_authorization_hash`": `"sha256:$recoveryRollbackPreviewAuthorizationHash`"" },
        @{ Suffix = "apply_hash"; Needle = "`"rollback_apply_authorization_hash`": `"sha256:$recoveryRollbackApplyAuthorizationHash`"" },
        @{ Suffix = "disable_hash"; Needle = "`"disable_module_target_binding_hash`": `"sha256:$recoveryDisableModuleTargetBindingHash`"" },
        @{ Suffix = "restart_hash"; Needle = "`"restart_last_good_target_binding_hash`": `"sha256:$recoveryRestartLastGoodTargetBindingHash`"" },
        @{ Suffix = "load_hash"; Needle = "`"load_artifact_by_hash_target_binding_hash`": `"sha256:$recoveryLoadArtifactByHashTargetBindingHash`"" },
        @{ Suffix = "projection_hash"; Needle = "`"recovery_memory_projection_hash`": `"sha256:$recoveryMemoryProjectionHash`"" },
        @{ Suffix = "authority_hash"; Needle = "`"recovery_memory_write_authority_hash`": `"sha256:$recoveryMemoryWriteAuthorityHash`"" },
        @{ Suffix = "valid_hash"; Needle = '"valid_hash_reference": true' },
        @{ Suffix = "write_false"; Needle = '"writes_recovery_memory": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
        @{ Suffix = "load_attempted_false"; Needle = '"load_attempted": false' }
    )

    $recoveryMemoryWriteAuthorityResponse = Get-LastAgentResponseJson -Method "recovery.memory_write_authority_diagnostic"
    $recoveryMemoryWriteAuthorityEventId = [string]$recoveryMemoryWriteAuthorityResponse.body.result.retained_recovery_memory_write_authority_reference.recorded_event_id
    Assert-CurrentBootEventId -Name "protocol:recovery_memory_write_authority_retained_reference_event_id_captured" -Value $recoveryMemoryWriteAuthorityEventId

    Send-AgentCommand -Command "agent recovery.lifeline_command_dispatch_diagnostic" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_command_dispatch_diagnostic"
    Assert-LogContainsFields -NamePrefix "protocol:recovery_lifeline_command_dispatch_after_memory_authority_" -TimeoutSeconds 1 -Fields @(
        @{ Suffix = "schema"; Needle = '"schema": "raios.recovery_lifeline_command_dispatch_denial.v0"' },
        @{ Suffix = "status"; Needle = '"status": "denied_missing_lifeline_command_dispatch_boundary"' },
        @{ Suffix = "reason"; Needle = '"reason": "durable_audit_rollback_write_authority_missing"' },
        @{ Suffix = "body_present"; Needle = '"command_body_canonicalization_present": true' },
        @{ Suffix = "handler_present"; Needle = '"command_handler_binding_present": true' },
        @{ Suffix = "status_handler_present"; Needle = '"status_read_handler_present": true' },
        @{ Suffix = "preview_auth_present"; Needle = '"rollback_preview_authorization_present": true' },
        @{ Suffix = "apply_auth_present"; Needle = '"rollback_apply_authorization_present": true' },
        @{ Suffix = "disable_target_present"; Needle = '"disable_module_target_binding_present": true' },
        @{ Suffix = "restart_target_present"; Needle = '"restart_last_good_target_binding_present": true' },
        @{ Suffix = "load_hash_target_present"; Needle = '"load_artifact_by_hash_target_binding_present": true' },
        @{ Suffix = "memory_authority_present"; Needle = '"recovery_memory_write_authority_present": true' },
        @{ Suffix = "durable_authority_missing"; Needle = '"durable_audit_rollback_write_authority_present": false' },
        @{ Suffix = "no_dispatch"; Needle = '"dispatches_lifeline_command": false' },
        @{ Suffix = "command_execution_false"; Needle = '"command_execution_enabled": false' },
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

    Send-AgentCommand -Command "agent audit.events 256" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events"
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
    Assert-LogContains -Name "protocol:recovery_lifeline_command_vocab_audit_source" -Needle '"source_method": "recovery.lifeline_command_vocabulary"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_vocab_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_vocabulary_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_runtime_isolation_audit_source" -Needle '"source_method": "recovery.loader_runtime_isolation"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_loader_runtime_isolation_selftest_audit_source" -Needle '"source_method": "recovery.loader_runtime_isolation_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_transaction_engine_audit_source" -Needle '"source_method": "recovery.rollback_transaction_engine"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_transaction_engine_selftest_audit_source" -Needle '"source_method": "recovery.rollback_transaction_engine_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_durable_audit_rollback_persistence_audit_source" -Needle '"source_method": "recovery.durable_audit_rollback_persistence"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_durable_audit_rollback_persistence_selftest_audit_source" -Needle '"source_method": "recovery.durable_audit_rollback_persistence_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_provenance_audit_source" -Needle '"source_method": "recovery.memory_provenance"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_provenance_selftest_audit_source" -Needle '"source_method": "recovery.memory_provenance_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_admission_audit_source" -Needle '"source_method": "recovery.lifeline_command_admission"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_admission_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_admission_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_envelope_audit_source" -Needle '"source_method": "recovery.lifeline_command_envelope_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_envelope_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_envelope_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_dispatch_audit_source" -Needle '"source_method": "recovery.lifeline_command_dispatch_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_dispatch_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_dispatch_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_body_audit_source" -Needle '"source_method": "recovery.lifeline_command_body_canonicalization_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_body_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_body_canonicalization_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_body_audit_kind" -Needle '"kind": "recovery.lifeline_command_body_canonicalization.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_body_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_command_body_canonicalization.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_body_audit_envelope_event" -Needle "`"retained_recovery_lifeline_command_envelope_event_id`": `"$recoveryLifelineCommandEnvelopeEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_body_audit_hash" -Needle "`"command_body_canonicalization_hash`": `"sha256:$recoveryLifelineCommandBodyCanonicalizationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_body_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_handler_audit_source" -Needle '"source_method": "recovery.lifeline_command_handler_binding_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_handler_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_command_handler_binding_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_handler_audit_kind" -Needle '"kind": "recovery.lifeline_command_handler_binding.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_handler_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_command_handler_binding.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_handler_audit_body_event" -Needle "`"retained_recovery_lifeline_command_body_canonicalization_event_id`": `"$recoveryLifelineCommandBodyEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_handler_audit_hash" -Needle "`"handler_binding_hash`": `"sha256:$recoveryCommandHandlerBindingHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_command_handler_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_audit_source" -Needle '"source_method": "recovery.lifeline_status_read_handler_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_selftest_audit_source" -Needle '"source_method": "recovery.lifeline_status_read_handler_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_audit_kind" -Needle '"kind": "recovery.lifeline_status_read_handler.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_lifeline_status_read_handler.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_audit_handler_event" -Needle "`"retained_recovery_lifeline_command_handler_binding_event_id`": `"$recoveryCommandHandlerBindingEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_audit_hash" -Needle "`"status_read_handler_hash`": `"sha256:$recoveryStatusReadHandlerHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_lifeline_status_read_handler_audit_no_status_execute" -Needle '"executes_lifeline_status": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_audit_source" -Needle '"source_method": "recovery.rollback_preview_authorization_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_selftest_audit_source" -Needle '"source_method": "recovery.rollback_preview_authorization_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_audit_kind" -Needle '"kind": "recovery.rollback_preview_authorization.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_rollback_preview_authorization.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_audit_status_event" -Needle "`"retained_recovery_lifeline_status_read_handler_event_id`": `"$recoveryStatusReadHandlerEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_audit_hash" -Needle "`"rollback_preview_authorization_hash`": `"sha256:$recoveryRollbackPreviewAuthorizationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_preview_authorization_audit_no_preview" -Needle '"executes_rollback_preview": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_audit_source" -Needle '"source_method": "recovery.rollback_apply_authorization_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_selftest_audit_source" -Needle '"source_method": "recovery.rollback_apply_authorization_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_audit_kind" -Needle '"kind": "recovery.rollback_apply_authorization.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_rollback_apply_authorization.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_audit_preview_event" -Needle "`"retained_recovery_rollback_preview_authorization_event_id`": `"$recoveryRollbackPreviewAuthorizationEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_audit_hash" -Needle "`"rollback_apply_authorization_hash`": `"sha256:$recoveryRollbackApplyAuthorizationHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_rollback_apply_authorization_audit_no_apply" -Needle '"executes_rollback_apply": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_audit_source" -Needle '"source_method": "recovery.disable_module_target_binding_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_selftest_audit_source" -Needle '"source_method": "recovery.disable_module_target_binding_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_audit_kind" -Needle '"kind": "recovery.disable_module_target_binding.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_disable_module_target_binding.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_audit_apply_event" -Needle "`"retained_recovery_rollback_apply_authorization_event_id`": `"$recoveryRollbackApplyAuthorizationEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_audit_hash" -Needle "`"disable_module_target_binding_hash`": `"sha256:$recoveryDisableModuleTargetBindingHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_disable_module_target_binding_audit_no_disable" -Needle '"disables_module": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_audit_source" -Needle '"source_method": "recovery.restart_last_good_target_binding_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_selftest_audit_source" -Needle '"source_method": "recovery.restart_last_good_target_binding_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_audit_kind" -Needle '"kind": "recovery.restart_last_good_target_binding.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_restart_last_good_target_binding.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_audit_disable_event" -Needle "`"retained_recovery_disable_module_target_binding_event_id`": `"$recoveryDisableModuleTargetBindingEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_audit_hash" -Needle "`"restart_last_good_target_binding_hash`": `"sha256:$recoveryRestartLastGoodTargetBindingHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_restart_last_good_target_binding_audit_no_restart" -Needle '"restarts_last_good": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_audit_source" -Needle '"source_method": "recovery.load_artifact_by_hash_target_binding_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_selftest_audit_source" -Needle '"source_method": "recovery.load_artifact_by_hash_target_binding_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_audit_kind" -Needle '"kind": "recovery.load_artifact_by_hash_target_binding.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_load_artifact_by_hash_target_binding.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_audit_restart_event" -Needle "`"retained_recovery_restart_last_good_target_binding_event_id`": `"$recoveryRestartLastGoodTargetBindingEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_audit_hash" -Needle "`"load_artifact_by_hash_target_binding_hash`": `"sha256:$recoveryLoadArtifactByHashTargetBindingHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_load_artifact_by_hash_target_binding_audit_no_load" -Needle '"loads_recovery_artifact": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_audit_source" -Needle '"source_method": "recovery.memory_write_authority_diagnostic"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_selftest_audit_source" -Needle '"source_method": "recovery.memory_write_authority_diagnostic_selftest"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_audit_kind" -Needle '"kind": "recovery.memory_write_authority.retained"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_audit_binding_schema" -Needle '"bindings": {"schema": "raios.recovery_memory_write_authority.v0"' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_audit_load_event" -Needle "`"retained_recovery_load_artifact_by_hash_target_binding_event_id`": `"$recoveryLoadArtifactByHashTargetBindingEventId`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_audit_hash" -Needle "`"recovery_memory_write_authority_hash`": `"sha256:$recoveryMemoryWriteAuthorityHash`"" -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_audit_no_dispatch" -Needle '"dispatches_lifeline_command": false' -TimeoutSeconds 1
    Assert-LogContains -Name "protocol:recovery_memory_write_authority_audit_no_write" -Needle '"writes_recovery_memory": false' -TimeoutSeconds 1
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
