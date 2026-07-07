param(
    [int]$SerialTcpPort = 4665,
    [string]$Image = "",
    [string]$ReportDir = "$PSScriptRoot\..\release\vm-reports",
    [int]$TimeoutSeconds = 180,
    [int]$SerialWriteChunkSize = 256,
    [int]$SerialWriteDelayMilliseconds = 0,
    [switch]$KeepRunDir
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$RunScript = Join-Path $RepoRoot "scripts\run-stage0-qemu.ps1"
$PackageScript = Join-Path $RepoRoot "scripts\package-stage0.ps1"
$Builder = Join-Path $RepoRoot "scripts\make-gpt-persist-image.py"
$RunId = "shadow-persistence-reboot-{0:yyyyMMdd-HHmmss}-{1}" -f (Get-Date), $PID
$RunDir = Join-Path $env:TEMP "raios-$RunId"
$SerialLog = Join-Path $RunDir "serial-merged.log"
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
$Profile = "persistence-reboot"
$ResolvedImage = $null
$ScratchImage = $null
$AuditRollbackTargetImage = $null
$PersistDiskImage = $null
$ResolvedArtifact = $null
$ResolvedManifest = $null
$ManifestValidation = $null
$Network = $false
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

function Convert-CompactJson {
    param(
        [object]$Value,
        [int]$Depth = 8
    )
    if ($null -eq $Value) {
        return "null"
    }
    return ($Value | ConvertTo-Json -Compress -Depth $Depth)
}

function Assert-ReportPredicate {
    param(
        [string]$Prefix,
        [string]$Name,
        [string]$Expected,
        [bool]$Passed,
        [string]$Actual,
        [string]$FailureMessage
    )
    Add-Predicate -Name "${Prefix}:$Name" -Expected $Expected -Passed $Passed -Actual $Actual
    if (-not $Passed) {
        throw $FailureMessage
    }
}

function Invoke-NativeCommandForReport {
    param(
        [string]$FilePath,
        [string[]]$Arguments
    )
    $oldErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        $output = & $FilePath @Arguments 2>&1 | ForEach-Object { "$_" }
        $exitCode = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $oldErrorActionPreference
    }
    return [pscustomobject]@{
        ExitCode = $exitCode
        Output = @($output)
    }
}

function Prefix-NewReportEntries {
    param(
        [int]$PredicateStart,
        [int]$CommandStart,
        [string]$Prefix
    )
    for ($i = $PredicateStart; $i -lt $Predicates.Count; $i++) {
        $entry = $Predicates[$i]
        $name = [string]$entry["name"]
        if (-not $name.StartsWith("${Prefix}:", [System.StringComparison]::Ordinal)) {
            $entry["name"] = "${Prefix}:$name"
        }
    }
    for ($i = $CommandStart; $i -lt $ExecutedCommands.Count; $i++) {
        $entry = $ExecutedCommands[$i]
        $name = [string]$entry["name"]
        if (-not $name.StartsWith("${Prefix}:", [System.StringComparison]::Ordinal)) {
            $entry["name"] = "${Prefix}:$name"
        }
    }
}

function Send-AgentCommandTagged {
    param(
        [string]$Prefix,
        [string]$Command,
        [string]$ExpectedMarker,
        [string]$Name
    )
    Send-AgentCommand -Command $Command -ExpectedMarker $ExpectedMarker -Name "${Prefix}:$Name"
}

function Send-CandidateBytesTagged {
    param(
        [string]$Prefix,
        [string]$Path
    )
    $predicateStart = $Predicates.Count
    $commandStart = $ExecutedCommands.Count
    try {
        return Send-CandidateBytes -Path $Path
    }
    finally {
        Prefix-NewReportEntries -PredicateStart $predicateStart -CommandStart $commandStart -Prefix $Prefix
    }
}

function New-MarkerImage {
    param(
        [string]$Path,
        [string]$Marker
    )
    $sector0 = New-Object byte[] 512
    $markerBytes = [System.Text.Encoding]::ASCII.GetBytes($Marker)
    [Array]::Copy($markerBytes, 0, $sector0, 0, $markerBytes.Length)
    $stream = [System.IO.File]::Open($Path, [System.IO.FileMode]::Create, [System.IO.FileAccess]::ReadWrite, [System.IO.FileShare]::Read)
    try {
        $stream.SetLength(1MB)
        $stream.Write($sector0, 0, $sector0.Length)
    }
    finally {
        $stream.Dispose()
    }
}

function Get-PersistInspection {
    param([string]$Path)
    $json = & python $Builder --inspect-json $Path
    if ($LASTEXITCODE -ne 0) {
        throw "Persist image inspection failed: $Path"
    }
    return ($json -join [Environment]::NewLine) | ConvertFrom-Json
}

function Invoke-PersistTool {
    param([string[]]$Arguments)
    $toolArguments = @($Builder) + $Arguments
    $result = Invoke-NativeCommandForReport -FilePath "python" -Arguments $toolArguments
    if ($result.ExitCode -ne 0) {
        throw "Persist image tool failed ($($Arguments -join ' ')): $($result.Output -join [Environment]::NewLine)"
    }
    return ($result.Output -join [Environment]::NewLine) | ConvertFrom-Json
}

function Start-RaiosVm {
    param(
        [string]$Label,
        [string]$PredicatePrefix,
        [int]$Port,
        [int]$MonitorPort,
        [string]$PersistPath
    )

    $stageImage = Join-Path $RunDir "raios-stage0-$Label.img"
    $scratch = Join-Path $RunDir "raios-stage0-$Label-scratch.img"
    $audit = Join-Path $RunDir "raios-stage0-$Label-audit-rollback-target.img"
    $log = Join-Path $RunDir "serial-$Label.log"
    Copy-Item -LiteralPath $ResolvedImage -Destination $stageImage -Force
    New-MarkerImage -Path $scratch -Marker "RAIOS_SCRATCH_V0"
    New-MarkerImage -Path $audit -Marker "RAIOS_AUDITRB_V0"

    $script:SerialLog = $log
    $script:SerialTcpPort = $Port
    $script:SerialLogCachePath = $null
    $script:SerialLogCacheLength = [int64]-1
    $script:SerialLogCacheWriteTicks = [int64]-1
    $script:SerialLogCacheContent = $null

    $runParams = @{
        Image = $stageImage
        ScratchImage = $scratch
        AuditRollbackTargetImage = $audit
        PersistDiskPath = $PersistPath
        PersistCacheMode = "writethrough"
        SerialMode = "tcp"
        SerialTcpPort = $Port
        SerialLog = $log
        Headless = $true
        UsbXhciInput = $true
        Cpu = "max"
        Nic = "none"
    }
    $argList = @(
        "-Image", $stageImage,
        "-ScratchImage", $scratch,
        "-AuditRollbackTargetImage", $audit,
        "-PersistDiskPath", $PersistPath,
        "-PersistCacheMode", "writethrough",
        "-SerialMode", "tcp",
        "-SerialTcpPort", "$Port",
        "-SerialLog", $log,
        "-Headless",
        "-UsbXhciInput",
        "-Cpu", "max",
        "-Nic", "none"
    )
    if ($MonitorPort -gt 0) {
        $runParams.MonitorTcpPort = $MonitorPort
        $argList += @("-MonitorTcpPort", "$MonitorPort")
    }
    $script:QemuArgList += @("${Label}:") + $argList

    $runOutput = & $RunScript @runParams
    $qemuPid = $null
    foreach ($line in $runOutput) {
        if ($line -match '^qemu pid:\s*(\d+)') {
            $qemuPid = [int]$Matches[1]
        }
    }
    if (-not $qemuPid) {
        throw "Could not parse QEMU pid from runner output for $Label"
    }
    $script:QemuPid = $qemuPid
    try {
        $script:QemuProcess = Get-Process -Id $qemuPid -ErrorAction Stop
    }
    catch {
        $script:QemuProcess = $null
    }
    $script:QemuTeardownAction = "running"

    Assert-LogContains -Name "${PredicatePrefix}:$Label-serial-console-ready" -Needle "SERIAL CONSOLE READY" -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains -Name "${PredicatePrefix}:$Label-framebuffer-ready" -Needle "status FRAMEBUFFER: READY" -TimeoutSeconds $TimeoutSeconds
    Assert-LogContains -Name "${PredicatePrefix}:$Label-usb-xhci-ready" -Needle "status USB-XHCI: READY" -TimeoutSeconds $TimeoutSeconds

    return [pscustomobject]@{
        Label = $Label
        PredicatePrefix = $PredicatePrefix
        SerialLog = $log
        SerialTcpPort = $Port
        MonitorPort = $MonitorPort
        Pid = $qemuPid
        Process = $script:QemuProcess
        StageImage = $stageImage
        ScratchImage = $scratch
        AuditRollbackTargetImage = $audit
        PersistDisk = $PersistPath
        CleanupPaths = @($stageImage, $scratch, $audit)
    }
}

function Stop-RaiosVmCleanly {
    param(
        [object]$Vm,
        [string]$Name
    )
    Close-SerialTcpConnection
    $script:QemuPid = $Vm.Pid
    $script:QemuProcess = $Vm.Process
    $passed = $false
    $actual = ""
    $client = $null
    try {
        $client = [System.Net.Sockets.TcpClient]::new()
        $connect = $client.BeginConnect("127.0.0.1", $Vm.MonitorPort, $null, $null)
        if (-not $connect.AsyncWaitHandle.WaitOne([TimeSpan]::FromSeconds(5))) {
            throw "timed out connecting to QEMU monitor port $($Vm.MonitorPort)"
        }
        $client.EndConnect($connect)
        $stream = $client.GetStream()
        $bytes = [System.Text.Encoding]::ASCII.GetBytes("quit`n")
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush()
        if ($null -ne $Vm.Process) {
            $passed = $Vm.Process.WaitForExit(15000)
        }
        else {
            $deadline = [DateTime]::UtcNow.AddSeconds(15)
            do {
                try {
                    Get-Process -Id $Vm.Pid -ErrorAction Stop | Out-Null
                    Start-Sleep -Milliseconds 200
                }
                catch {
                    $passed = $true
                    break
                }
            } while ([DateTime]::UtcNow -lt $deadline)
        }
        $actual = if ($passed) { "qmp_quit_exited" } else { "qmp_quit_sent_but_process_still_running" }
    }
    catch {
        $actual = $_.Exception.Message
    }
    finally {
        if ($client) {
            $client.Close()
        }
    }
    $script:QemuProcessBeforeTeardown = Get-QemuProcessSnapshot -Observation "${Name}_after_qmp_quit"
    $script:QemuTeardownAction = "qmp_quit"
    $script:QemuProcessAfterTeardown = Get-QemuProcessSnapshot -Observation "${Name}_after_qmp_quit_wait"
    Add-Predicate -Name "$($Vm.PredicatePrefix):$Name-clean-qmp-quit" -Expected "qmp_monitor_quit_exits_qemu" -Passed $passed -Actual $actual
    if (-not $passed) {
        Stop-RaiosVmForce -Vm $Vm
        throw "QEMU did not exit cleanly for ${Name}: $actual"
    }
}

function Stop-RaiosVmForce {
    param([object]$Vm)
    Close-SerialTcpConnection
    if ($Vm -and $Vm.Pid) {
        try {
            $proc = Get-Process -Id $Vm.Pid -ErrorAction Stop
            Stop-Process -Id $Vm.Pid -Force -ErrorAction SilentlyContinue
            $proc.WaitForExit(5000) | Out-Null
        }
        catch {
        }
    }
}

function Remove-RunImages {
    param([object]$Vm)
    if ($Vm -and $Vm.CleanupPaths) {
        Remove-Item -LiteralPath $Vm.CleanupPaths -Force -ErrorAction SilentlyContinue
    }
}

function Invoke-RealDevKeyPromotion {
    param([string]$Prefix)

    $expectedSha = "f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2"
    $delivery = Send-CandidateBytesTagged -Prefix $Prefix -Path $ResolvedArtifact
    $finalize = $delivery.finalize_response
    $result = $finalize.body.result
    $deliveryOk = (
        $finalize.t -eq "response" -and
        $finalize.body.method -eq "module.submit_candidate_finalize" -and
        [int]$result.byte_len -eq 4205 -and
        $result.artifact_sha256 -eq "sha256:$expectedSha" -and
        $result.wasm_valid -eq $true -and
        $result.retained_in_ram -eq $true -and
        $result.rejected -eq $false
    )
    Assert-ReportPredicate -Prefix $Prefix -Name "serial-delivery-retains-valid-echo-wasm" -Expected "runtime-delivered echo wasm retained in RAM" -Passed $deliveryOk -Actual $(if ($deliveryOk) { "matched" } else { Convert-CompactJson $result }) -FailureMessage "Expected serial-delivered echo wasm to finalize as retained valid candidate"

    $moduleGrantManifestHash = "1111111111111111111111111111111111111111111111111111111111111111"
    $moduleGrantArtifactHash = Get-FileSha256OrNull -Path $ResolvedArtifact
    $moduleGrantReportHash = "3333333333333333333333333333333333333333333333333333333333333333"
    $moduleGrantAttestationHash = "4444444444444444444444444444444444444444444444444444444444444444"
    if ($moduleGrantArtifactHash -ne $expectedSha) {
        throw "Expected candidate artifact hash $expectedSha, got $moduleGrantArtifactHash"
    }

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
    Send-AgentCommandTagged -Prefix $Prefix -Command "agent module.manifest_diagnostic $moduleManifestReferenceHash $moduleGrantManifestHash" -ExpectedMarker "RAIOS_AGENT_END module.manifest_diagnostic" -Name "manifest-reference"
    $moduleManifestResponse = Get-LastAgentResponseJson -Method "module.manifest_diagnostic"
    $moduleManifestRetainedReferenceEventId = [string]$moduleManifestResponse.body.result.retained_manifest_reference.event_id
    Assert-CurrentBootEventId -Name "${Prefix}:manifest-reference-event-id" -Value $moduleManifestRetainedReferenceEventId

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
    Send-AgentCommandTagged -Prefix $Prefix -Command $moduleGrantCommand -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic" -Name "initial-grant-reference"
    $moduleGrantResponse = Get-LastAgentResponseJson -Method "module.grant_diagnostic"
    $moduleGrantRetainedReferenceEventId = [string]$moduleGrantResponse.body.result.retained_reference.event_id
    Assert-CurrentBootEventId -Name "${Prefix}:grant-reference-event-id" -Value $moduleGrantRetainedReferenceEventId

    $moduleArtifactReferenceCanonical = @(
        "canonicalization=raios.module_candidate_artifact_reference.canonical.v0",
        "schema=raios.module_candidate_artifact_reference.v0",
        "requested_capability=cap.module.load_ephemeral",
        "load_mode=ram_only",
        "subject=agent.session.serial",
        "resource=live_service_graph",
        "scope=current_boot",
        "retained_manifest_reference_event_id=$moduleManifestRetainedReferenceEventId",
        "retained_reference_event_id=$moduleGrantRetainedReferenceEventId",
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
    $moduleArtifactCommand = "agent module.artifact_diagnostic $moduleArtifactReferenceHash $moduleManifestRetainedReferenceEventId $moduleGrantRetainedReferenceEventId $moduleManifestReferenceHash $moduleGrantManifestHash $moduleGrantHash $moduleGrantArtifactHash $moduleGrantReportHash $moduleGrantAttestationHash"
    Send-AgentCommandTagged -Prefix $Prefix -Command $moduleArtifactCommand -ExpectedMarker "RAIOS_AGENT_END module.artifact_diagnostic" -Name "artifact-reference"
    $moduleArtifactResponse = Get-LastAgentResponseJson -Method "module.artifact_diagnostic"
    $moduleArtifactRetainedReferenceEventId = [string]$moduleArtifactResponse.body.result.retained_candidate_artifact_reference.event_id
    Assert-CurrentBootEventId -Name "${Prefix}:artifact-reference-event-id" -Value $moduleArtifactRetainedReferenceEventId

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
        "retained_reference_event_id=$moduleGrantRetainedReferenceEventId",
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
    $moduleVmReportCommand = "agent module.vm_report_diagnostic $moduleVmReportReferenceHash $moduleManifestRetainedReferenceEventId $moduleArtifactRetainedReferenceEventId $moduleGrantRetainedReferenceEventId $moduleManifestReferenceHash $moduleArtifactReferenceHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantHash $moduleGrantReportHash $moduleGrantAttestationHash"
    Send-AgentCommandTagged -Prefix $Prefix -Command $moduleVmReportCommand -ExpectedMarker "RAIOS_AGENT_END module.vm_report_diagnostic" -Name "vm-report-reference"
    $moduleVmReportResponse = Get-LastAgentResponseJson -Method "module.vm_report_diagnostic"
    $moduleVmReportRetainedReferenceEventId = [string]$moduleVmReportResponse.body.result.retained_vm_test_report_reference.event_id
    Assert-CurrentBootEventId -Name "${Prefix}:vm-report-reference-event-id" -Value $moduleVmReportRetainedReferenceEventId

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
        "retained_reference_event_id=$moduleGrantRetainedReferenceEventId",
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
    $signatureHex = New-ReliableDevPromotionSignatureHex -AttestationReferenceHash $moduleAttestationReferenceHash
    Assert-ReportPredicate -Prefix $Prefix -Name "rust-signer-produced-der" -Expected "dev-promotion-signer returns DER hex" -Passed ($signatureHex -match '^[0-9a-f]+$') -Actual "len=$($signatureHex.Length)" -FailureMessage "Rust dev signer did not produce DER hex"

    $moduleAttestationCommand = "agent module.attestation_diagnostic $moduleAttestationReferenceHash $moduleManifestRetainedReferenceEventId $moduleArtifactRetainedReferenceEventId $moduleVmReportRetainedReferenceEventId $moduleGrantRetainedReferenceEventId $moduleManifestReferenceHash $moduleArtifactReferenceHash $moduleVmReportReferenceHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantHash $moduleGrantReportHash $moduleGrantAttestationHash $signatureHex"
    Send-AgentCommandTagged -Prefix $Prefix -Command $moduleAttestationCommand -ExpectedMarker "RAIOS_AGENT_END module.attestation_diagnostic" -Name "signed-attestation-reference"
    $moduleAttestationResponse = Get-LastAgentResponseJson -Method "module.attestation_diagnostic"
    $attestationResult = $moduleAttestationResponse.body.result
    $signatureVerifiedOk = (
        $attestationResult.local_attestation_reference.validation_status -eq "local_attestation_signature_verified_load_still_denied" -and
        $attestationResult.local_attestation_reference.signature_verified -eq $true
    )
    Assert-ReportPredicate -Prefix $Prefix -Name "signature-verified-true" -Expected "module.attestation_diagnostic reports signature_verified true" -Passed $signatureVerifiedOk -Actual $(if ($signatureVerifiedOk) { "matched" } else { Convert-CompactJson $attestationResult 10 }) -FailureMessage "Expected real dev-key signature to verify"

    Send-AgentCommandTagged -Prefix $Prefix -Command $moduleGrantCommand -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic" -Name "grant-can-load-now"
    $grantReady = Get-LastAgentResponseJson -Method "module.grant_diagnostic"
    $grantReadyResult = $grantReady.body.result
    $grantReadyOk = (
        $grantReadyResult.policy_result.grants_capability -eq $true -and
        $grantReadyResult.policy_result.trust_tier -eq "dev_key_not_owner_sealed" -and
        $grantReadyResult.policy_result.can_load_now -eq $true
    )
    Assert-ReportPredicate -Prefix $Prefix -Name "grant-reports-dev-tier-can-load-now" -Expected "grants_capability=true trust_tier=dev_key_not_owner_sealed can_load_now=true" -Passed $grantReadyOk -Actual $(if ($grantReadyOk) { "matched" } else { Convert-CompactJson $grantReadyResult 8 }) -FailureMessage "Expected signed grant with retained bytes to report can_load_now"

    $loadOffset = Get-SerialLogOffset
    Send-AgentCommandTagged -Prefix $Prefix -Command "module.load_ephemeral svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "granted-candidate-load"
    $load = Get-LastAgentResponseJson -Method "module.load_ephemeral"
    $loadResult = $load.body.result
    $loadOk = (
        $load.t -eq "response" -and
        $loadResult.schema -eq "raios.ram_only_granted_candidate_service.lifecycle_response.v0" -and
        $loadResult.service_id -eq "svc.dev.granted_candidate" -and
        $loadResult.action -eq "load" -and
        $loadResult.loaded -eq $true -and
        $loadResult.scope -eq "current_boot" -and
        $loadResult.trust_tier -eq "dev_key_not_owner_sealed"
    )
    Assert-ReportPredicate -Prefix $Prefix -Name "granted-candidate-loads-service" -Expected "granted external candidate loads as dev-tier current_boot service" -Passed $loadOk -Actual $(if ($loadOk) { "matched" } else { Convert-CompactJson $loadResult 8 }) -FailureMessage "Expected granted candidate load response"

    $durablePromotion = $loadResult.durable_promotion_transaction
    $durablePromotionOk = (
        $durablePromotion.performed -eq $true -and
        $durablePromotion.durable_append -eq "appended" -and
        $durablePromotion.transaction_kind -eq "promote" -and
        $durablePromotion.signature_verified -eq $true -and
        $durablePromotion.grant_binds_capability -eq $true -and
        $durablePromotion.trust_tier -eq "dev_key_not_owner_sealed" -and
        $durablePromotion.owner_sealed -eq $false -and
        $durablePromotion.persistence_claimed -eq $false
    )
    Assert-ReportPredicate -Prefix $Prefix -Name "durable-promotion-performed" -Expected "durable_promotion_transaction.performed true" -Passed $durablePromotionOk -Actual $(if ($durablePromotionOk) { "matched" } else { Convert-CompactJson $durablePromotion 10 }) -FailureMessage "Expected boot 1 to append durable promotion transaction"

    $artifactPersist = $loadResult.durable_artifact_persist
    $artifactPersistOk = (
        $artifactPersist.performed -eq $true -and
        $artifactPersist.status -eq "appended" -and
        $artifactPersist.blob_written_to_disk -eq $true -and
        $artifactPersist.artstor_blob_frame_sha256 -ne $null -and
        $artifactPersist.artifact_persist_frame_sha256 -ne $null -and
        $artifactPersist.authorizes_load -eq $false -and
        $artifactPersist.owner_sealed -eq $false -and
        $artifactPersist.cross_reboot_proven -eq $false -and
        $artifactPersist.persistence_claimed -eq $false
    )
    Assert-ReportPredicate -Prefix $Prefix -Name "artifact-persisted" -Expected "durable_artifact_persist.performed true and blob_written_to_disk true" -Passed $artifactPersistOk -Actual $(if ($artifactPersistOk) { "matched" } else { Convert-CompactJson $artifactPersist 10 }) -FailureMessage "Expected boot 1 to persist artifact blob and artifact_persist record"

    Send-AgentCommandTagged -Prefix $Prefix -Command "service.start svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "granted-candidate-start"
    $start = Get-LastAgentResponseJson -Method "service.start"
    $startResult = $start.body.result
    $run = $startResult.run_evidence
    $startOk = (
        $start.t -eq "response" -and
        $startResult.running -eq $true -and
        $run.present -eq $true -and
        $run.validation_ok -eq $true -and
        $run.instantiation_ok -eq $true -and
        $run.run_outcome -eq "success" -and
        $run.log_line_emitted -eq $true
    )
    $afterLoad = (Get-SerialLogContent -Path $SerialLog).Substring([int]$loadOffset)
    $guestLogOk = $afterLoad.Contains("WASM_GUEST_LOG echo counter=")
    Assert-ReportPredicate -Prefix $Prefix -Name "service-answers-before-reboot" -Expected "service.start emits live WASM_GUEST_LOG" -Passed ($startOk -and $guestLogOk) -Actual $(if ($startOk -and $guestLogOk) { "matched" } else { Convert-CompactJson $startResult 8 }) -FailureMessage "Expected boot 1 granted candidate to answer live"
}

function Assert-Boot2Repromotion {
    param([string]$Prefix)

    Send-AgentCommandTagged -Prefix $Prefix -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "reclog-scan-before-repromotion"
    $scan = Get-LastAgentResponseJson -Method "durable.record_log_scan"
    $scanResult = $scan.body.result
    $nonEmpty = [int]$scanResult.count -gt 0
    Assert-ReportPredicate -Prefix $Prefix -Name "reclog-non-empty-before-repromotion" -Expected "boot 2 reads non-empty boot-1 RECLOG" -Passed $nonEmpty -Actual $(Convert-CompactJson $scanResult 8) -FailureMessage "Boot 2 saw an empty RECLOG; refusing to mask a durability loss"

    Send-AgentCommandTagged -Prefix $Prefix -Command "agent repromotion.run" -ExpectedMarker "RAIOS_AGENT_END repromotion.run" -Name "repromotion-run"
    $repromotion = Get-LastAgentResponseJson -Method "repromotion.run"
    $body = $repromotion.body.result
    $decisions = @()
    if ($null -ne $body.decisions) {
        $decisions = @($body.decisions)
    }
    $grant = @($decisions | Where-Object { $_.status -eq "repromoted" })[0]
    $granted = (
        $body.schema -eq "raios.repromotion.v0" -and
        $body.status -eq "completed" -and
        $body.performed -eq $true -and
        [int]$body.artifact_count -ge 1 -and
        $grant -ne $null -and
        $grant.performed -eq $true -and
        $grant.authorizes_load -eq $true -and
        $grant.cross_reboot_proven -eq $true -and
        $grant.trust_tier -eq "dev_key_not_owner_sealed" -and
        $grant.owner_sealed -eq $false -and
        $grant.persistence_claimed -eq $false -and
        $grant.recorded_signature_verified -eq $true -and
        $grant.grant_binds_capability -eq $true
    )
    Assert-ReportPredicate -Prefix $Prefix -Name "repromotion-granted" -Expected "repromotion.run re-verifies persisted chain and grants dev-tier load" -Passed $granted -Actual $(if ($granted) { Convert-CompactJson $grant 10 } else { Convert-CompactJson $body 12 }) -FailureMessage "Expected boot 2 repromotion to grant and load the persisted service"

    $startOffset = Get-SerialLogOffset
    Send-AgentCommandTagged -Prefix $Prefix -Command "service.start svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "service-start-after-repromotion"
    $start = Get-LastAgentResponseJson -Method "service.start"
    $startResult = $start.body.result
    $afterStart = (Get-SerialLogContent -Path $SerialLog).Substring([int]$startOffset)
    $answers = (
        $start.t -eq "response" -and
        $startResult.running -eq $true -and
        $afterStart.Contains("WASM_GUEST_LOG echo counter=")
    )
    Assert-ReportPredicate -Prefix $Prefix -Name "service-answers-after-reboot" -Expected "re-promoted service answers live after reboot" -Passed $answers -Actual $(if ($answers) { "matched" } else { Convert-CompactJson $startResult 8 }) -FailureMessage "Expected re-promoted service to answer after reboot"
}

function Assert-RepromotionDeniedChild {
    param(
        [string]$Label,
        [string]$MutationFlag,
        [string]$ExpectedMismatch,
        [int]$Port
    )
    $prefix = "boot2-denyhash"
    $childVm = $null
    $childPersist = Join-Path $RunDir "persist-$Label.img"
    Copy-Item -LiteralPath $Boot1PersistSnapshot -Destination $childPersist -Force
    try {
        Invoke-PersistTool -Arguments @($MutationFlag, $childPersist) | Out-Null
        $inspection = Get-PersistInspection -Path $childPersist
        $records = @($inspection.artifact_persist_records)
        $mismatches = if ($records.Count -gt 0) { @($records[0].binding_mismatches) } else { @() }
        $landed = $mismatches -contains $ExpectedMismatch
        Assert-ReportPredicate -Prefix $prefix -Name "$Label-tamper-landed" -Expected "$ExpectedMismatch mismatch visible in inspect-json" -Passed $landed -Actual $(Convert-CompactJson $records 10) -FailureMessage "$Label tamper did not land"

        $childVm = Start-RaiosVm -Label "denyhash-$Label" -PredicatePrefix $prefix -Port $Port -MonitorPort 0 -PersistPath $childPersist
        Send-AgentCommandTagged -Prefix $prefix -Command "agent repromotion.run" -ExpectedMarker "RAIOS_AGENT_END repromotion.run" -Name "$Label-repromotion-run"
        $repromotion = Get-LastAgentResponseJson -Method "repromotion.run"
        $body = $repromotion.body.result
        $decisions = @()
        if ($null -ne $body.decisions) {
            $decisions = @($body.decisions)
        }
        $denied = @($decisions | Where-Object { $_.status -eq "repromotion_denied" -and $_.reason -match "hash_mismatch|sha_mismatch" })[0]
        $deniedOk = (
            $body.schema -eq "raios.repromotion.v0" -and
            [int]$body.artifact_count -ge 1 -and
            $denied -ne $null -and
            $denied.authorizes_load -eq $false -and
            $denied.cross_reboot_proven -eq $false
        )
        Assert-ReportPredicate -Prefix $prefix -Name "$Label-repromotion-denied-hash-mismatch" -Expected "repromotion_denied hash mismatch and no load authority" -Passed $deniedOk -Actual $(if ($deniedOk) { Convert-CompactJson $denied 10 } else { Convert-CompactJson $body 12 }) -FailureMessage "$Label did not produce repromotion_denied hash mismatch"

        $startOffset = Get-SerialLogOffset
        Send-AgentCommandTagged -Prefix $prefix -Command "service.start svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "$Label-service-start-after-denial"
        $afterStart = (Get-SerialLogContent -Path $SerialLog).Substring([int]$startOffset)
        $noAnswer = -not $afterStart.Contains("WASM_GUEST_LOG echo counter=")
        Assert-ReportPredicate -Prefix $prefix -Name "$Label-no-service-answer" -Expected "denied repromotion leaves service unable to answer" -Passed $noAnswer -Actual $(if ($noAnswer) { "no_guest_log_after_offset:$startOffset" } else { Get-SerialLogTail -Path $SerialLog }) -FailureMessage "$Label denial still allowed a service answer"
    }
    finally {
        Stop-RaiosVmForce -Vm $childVm
        Remove-RunImages -Vm $childVm
        Remove-Item -LiteralPath $childPersist -Force -ErrorAction SilentlyContinue
    }
}

function Assert-SafeChild {
    param([int]$Port)
    $prefix = "boot2-safe"
    $safeVm = $null
    $safePersist = Join-Path $RunDir "persist-safe.img"
    Copy-Item -LiteralPath $Boot1PersistSnapshot -Destination $safePersist -Force
    try {
        Invoke-PersistTool -Arguments @("--image", $safePersist, "--seed-bootctl", "both-invalid") | Out-Null
        $inspection = Get-PersistInspection -Path $safePersist
        $safePosture = $inspection.bootctl_read.decision.posture -eq "Safe"
        Assert-ReportPredicate -Prefix $prefix -Name "bootctl-safe-posture-landed" -Expected "both-invalid bootctl inspect-json posture Safe" -Passed $safePosture -Actual $(Convert-CompactJson $inspection.bootctl_read.decision 6) -FailureMessage "SAFE child bootctl patch did not land"

        $safeVm = Start-RaiosVm -Label "safe" -PredicatePrefix $prefix -Port $Port -MonitorPort 0 -PersistPath $safePersist
        Send-AgentCommandTagged -Prefix $prefix -Command "agent repromotion.run" -ExpectedMarker "RAIOS_AGENT_END repromotion.run" -Name "repromotion-run"
        $repromotion = Get-LastAgentResponseJson -Method "repromotion.run"
        $body = $repromotion.body.result
        $skipped = (
            $body.schema -eq "raios.repromotion.v0" -and
            $body.status -eq "repromotion_denied" -and
            $body.reason -eq "repromotion_skipped_in_safe" -and
            $body.performed -eq $false -and
            $body.authorizes_load -eq $false -and
            $body.cross_reboot_proven -eq $false
        )
        Assert-ReportPredicate -Prefix $prefix -Name "repromotion-skipped-in-safe" -Expected "SAFE posture skips repromotion with no load authority" -Passed $skipped -Actual $(if ($skipped) { "matched" } else { Convert-CompactJson $body 10 }) -FailureMessage "SAFE child did not skip repromotion"

        $startOffset = Get-SerialLogOffset
        Send-AgentCommandTagged -Prefix $prefix -Command "service.start svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "service-start-after-safe-skip"
        $afterStart = (Get-SerialLogContent -Path $SerialLog).Substring([int]$startOffset)
        $noAnswer = -not $afterStart.Contains("WASM_GUEST_LOG echo counter=")
        Assert-ReportPredicate -Prefix $prefix -Name "no-service-answer" -Expected "SAFE skipped repromotion leaves service unable to answer" -Passed $noAnswer -Actual $(if ($noAnswer) { "no_guest_log_after_offset:$startOffset" } else { Get-SerialLogTail -Path $SerialLog }) -FailureMessage "SAFE skipped repromotion still allowed a service answer"
    }
    finally {
        Stop-RaiosVmForce -Vm $safeVm
        Remove-RunImages -Vm $safeVm
        Remove-Item -LiteralPath $safePersist -Force -ErrorAction SilentlyContinue
    }
}

function Merge-SerialLogs {
    $merged = Join-Path $RunDir "serial-merged.log"
    Remove-Item -LiteralPath $merged -Force -ErrorAction SilentlyContinue
    $logs = @(Get-ChildItem -LiteralPath $RunDir -Filter "serial-*.log" -ErrorAction SilentlyContinue | Sort-Object Name)
    foreach ($log in $logs) {
        Add-Content -LiteralPath $merged -Encoding UTF8 -Value "===== $($log.Name) ====="
        if (Test-Path -LiteralPath $log.FullName) {
            Add-Content -LiteralPath $merged -Encoding UTF8 -Value (Get-Content -Raw -LiteralPath $log.FullName)
        }
    }
    $script:SerialLog = $merged
}

New-Item -ItemType Directory -Force -Path $RunDir | Out-Null

$Boot1PersistSnapshot = Join-Path $RunDir "persist-after-boot1.img"
$boot1Vm = $null
$boot2Vm = $null

try {
    $ResolvedArtifact = Join-Path $RepoRoot "seed-kernel\artifacts\svc.demo.echo.wasm"
    if (-not (Test-Path -LiteralPath $ResolvedArtifact)) {
        throw "Candidate artifact missing: $ResolvedArtifact"
    }

    $buildResult = Invoke-NativeCommandForReport -FilePath "cargo" -Arguments @("build", "-p", "ota-tools", "--bin", "dev-promotion-signer")
    Assert-ReportPredicate -Prefix "boot1" -Name "rust-signer-built" -Expected "cargo build -p ota-tools --bin dev-promotion-signer exits 0" -Passed ($buildResult.ExitCode -eq 0) -Actual $(if ($buildResult.ExitCode -eq 0) { "built" } else { $buildResult.Output -join [Environment]::NewLine }) -FailureMessage "dev-promotion-signer build failed"

    if ($Image) {
        $ResolvedImage = (Resolve-Path -LiteralPath $Image).Path
    }
    else {
        $ResolvedImage = Join-Path $RunDir "raios-stage0-persistence-reboot-base.img"
        $TempImage = $true
        & $PackageScript -Profile release -Image $ResolvedImage -UseTempEsp
        if ($LASTEXITCODE -ne 0) {
            throw "Image packaging failed with exit code $LASTEXITCODE"
        }
    }

    $PersistDiskImage = Join-Path $RunDir "kept-persist.img"
    $builderResult = Invoke-NativeCommandForReport -FilePath "python" -Arguments @($Builder, "--self-check", "--seed-bootctl", "valid-a", $PersistDiskImage)
    Assert-ReportPredicate -Prefix "boot1" -Name "kept-persist-disk-built" -Expected "self-check GPT persist disk with valid-a bootctl" -Passed ($builderResult.ExitCode -eq 0) -Actual $(if ($builderResult.ExitCode -eq 0) { "built" } else { $builderResult.Output -join [Environment]::NewLine }) -FailureMessage "Persist disk build failed"
    $PersistDiskImage = Assert-PersistDiskPathSafe -Path $PersistDiskImage
    $initialInspection = Get-PersistInspection -Path $PersistDiskImage
    $initialOk = (
        [bool]$initialInspection.gpt_header_valid -and
        [bool]$initialInspection.gpt_crc_checked -and
        [bool]$initialInspection.data_superblock_valid -and
        $initialInspection.bootctl_read.decision.posture -eq "Normal" -and
        [int]$initialInspection.reclog_scan.count -eq 0
    )
    Assert-ReportPredicate -Prefix "boot1" -Name "kept-persist-disk-empty-normal" -Expected "valid GPT, Normal bootctl, empty RECLOG" -Passed $initialOk -Actual $(Convert-CompactJson $initialInspection 8) -FailureMessage "Initial kept persist disk was not empty Normal posture"

    $HardwareProfile = New-HardwareProfile -Nic "none" -ScratchDrive $true -AuditRollbackTargetDrive $true -PersistDrive $true

    $boot1Vm = Start-RaiosVm -Label "boot1" -PredicatePrefix "boot1" -Port $SerialTcpPort -MonitorPort ($SerialTcpPort + 100) -PersistPath $PersistDiskImage
    Invoke-RealDevKeyPromotion -Prefix "boot1"
    Stop-RaiosVmCleanly -Vm $boot1Vm -Name "boot1"

    $postBoot1Inspection = Get-PersistInspection -Path $PersistDiskImage
    $postBoot1Records = @($postBoot1Inspection.artifact_persist_records)
    $postBoot1Ok = [int]$postBoot1Inspection.reclog_scan.count -gt 0 -and $postBoot1Records.Count -ge 1
    Assert-ReportPredicate -Prefix "boot1" -Name "persist-disk-has-records-after-clean-quit" -Expected "host inspect sees non-empty RECLOG and artifact_persist after boot 1 quit" -Passed $postBoot1Ok -Actual $(Convert-CompactJson $postBoot1Inspection 10) -FailureMessage "Boot 1 did not leave a non-empty persisted store"
    Copy-Item -LiteralPath $PersistDiskImage -Destination $Boot1PersistSnapshot -Force

    $boot2Vm = Start-RaiosVm -Label "boot2" -PredicatePrefix "boot2" -Port ($SerialTcpPort + 1) -MonitorPort ($SerialTcpPort + 101) -PersistPath $PersistDiskImage
    Assert-Boot2Repromotion -Prefix "boot2"
    Stop-RaiosVmCleanly -Vm $boot2Vm -Name "boot2"

    Assert-RepromotionDeniedChild -Label "corrupt-artstor-blob" -MutationFlag "--corrupt-artstor-blob" -ExpectedMismatch "artstor_blob_frame_sha256" -Port ($SerialTcpPort + 10)
    Assert-RepromotionDeniedChild -Label "tamper-persist-record" -MutationFlag "--tamper-persist-record" -ExpectedMismatch "artifact_sha256" -Port ($SerialTcpPort + 11)
    Assert-SafeChild -Port ($SerialTcpPort + 12)

    $Result = "passed"
}
catch {
    $Failures.Add($_.Exception.Message) | Out-Null
    throw
}
finally {
    Close-SerialTcpConnection
    Stop-RaiosVmForce -Vm $boot1Vm
    Stop-RaiosVmForce -Vm $boot2Vm
    Merge-SerialLogs

    Write-Report `
        -FinalResult $Result `
        -ResolvedImage $ResolvedImage `
        -ResolvedArtifact $ResolvedArtifact `
        -ResolvedManifest $ResolvedManifest `
        -QemuArgList $QemuArgList `
        -HardwareProfile $HardwareProfile `
        -StartedAt $StartedAt

    if ($Result -eq "passed" -and -not $KeepRunDir) {
        Remove-Item -LiteralPath $RunDir -Recurse -Force -ErrorAction SilentlyContinue
    }

    Write-Host "persistence reboot result: $Result"
    Write-Host "report: $ReportPath"
    Write-Host "report sha256: $ReportHashPath"
    Write-Host "run dir: $RunDir"
}
