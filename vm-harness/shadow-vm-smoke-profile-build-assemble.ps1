if (-not $StructuredStoreDiskImage -or -not $StructuredStoreFixture -or -not $PersistDiskImage -or
    -not $QmpTcpPort -or -not $MonitorTcpPort) {
    throw 'build-assemble requires C1 source store, valid-a BOOTCTL, QMP pointer, and monitor shutdown'
}
if (-not $StructuredStoreFixture.valid -or -not $StructuredStoreFixture.disposable_qemu_only -or
    $StructuredStoreFixture.store_state -ne 'empty_unformatted') {
    throw "build-assemble requires the dedicated empty disposable C1 fixture: $($StructuredStoreFixture | ConvertTo-Json -Compress -Depth 16)"
}

# Assembly and Genesis markers arrive late in boot on release images.
if ($TimeoutSeconds -lt 240) { $TimeoutSeconds = 240 }

function Add-BuildAssemblePredicate {
    param([string]$Name, [string]$Expected, [bool]$Passed, [object]$Actual)
    $actualText = if ($Passed -and $Actual -is [string]) {
        $Actual
    } else {
        $Actual | ConvertTo-Json -Compress -Depth 32
    }
    Add-Predicate -Name "build-assemble:$Name" -Expected $Expected -Passed $Passed -Actual $actualText
    if (-not $Passed) { throw "Build-assemble predicate failed: $Name" }
}

function Send-BuildAssembleCommand {
    param([string]$Command, [string]$Method, [string]$Name)
    Send-AgentCommand -Command $Command -ExpectedMarker "RAIOS_AGENT_END $Method" -Name "build-assemble:$Name"
    return (Get-LastAgentResponseJson -Method $Method)
}

function Get-BuildAssembleSha256Hex {
    param([byte[]]$Bytes)
    $sha = [Security.Cryptography.SHA256]::Create()
    try { return ([BitConverter]::ToString($sha.ComputeHash($Bytes)) -replace '-', '').ToLowerInvariant() }
    finally { $sha.Dispose() }
}

function Convert-BuildAssembleHexToBytes {
    param([string]$Hex)
    $bytes = [byte[]]::new($Hex.Length / 2)
    for ($index = 0; $index -lt $bytes.Length; $index++) {
        $bytes[$index] = [Convert]::ToByte($Hex.Substring($index * 2, 2), 16)
    }
    return $bytes
}

function Write-BuildAssembleCanonicalString {
    param([IO.BinaryWriter]$Writer, [string]$Value)
    $bytes = [Text.Encoding]::UTF8.GetBytes($Value)
    $Writer.Write([uint16]$bytes.Length)
    $Writer.Write($bytes)
}

function Get-BuildAssembleProjectId {
    param([byte[]]$RequestBytes)
    $stream = [IO.MemoryStream]::new()
    $writer = [IO.BinaryWriter]::new($stream, [Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([Text.Encoding]::ASCII.GetBytes('raios.agent_project_id.v1'))
        $writer.Write([uint64]$RequestBytes.Length)
        $writer.Write($RequestBytes)
        $writer.Flush()
        return (Get-BuildAssembleSha256Hex $stream.ToArray()).Substring(0, 32)
    }
    finally { $writer.Dispose(); $stream.Dispose() }
}

function Get-BuildAssembleTreeSha256 {
    param([object[]]$Files)
    $stream = [IO.MemoryStream]::new()
    $writer = [IO.BinaryWriter]::new($stream, [Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([Text.Encoding]::ASCII.GetBytes('RAIOSTR1'))
        $writer.Write([uint16]$Files.Count)
        foreach ($file in $Files) {
            Write-BuildAssembleCanonicalString $writer $file.path
            $writer.Write([byte]$(if ($file.classification -eq 'public') { 1 } else { 2 }))
            Write-BuildAssembleCanonicalString $writer $file.media_type
            $writer.Write([uint32]$file.bytes.Length)
            $writer.Write([byte[]](Convert-BuildAssembleHexToBytes $file.sha256))
        }
        $writer.Flush()
        return Get-BuildAssembleSha256Hex $stream.ToArray()
    }
    finally { $writer.Dispose(); $stream.Dispose() }
}

function Get-BuildAssembleRevisionSha256 {
    param([string]$ProjectId, [string]$TreeSha256)
    $stream = [IO.MemoryStream]::new()
    $writer = [IO.BinaryWriter]::new($stream, [Text.Encoding]::UTF8, $true)
    try {
        $writer.Write([Text.Encoding]::ASCII.GetBytes('RAIOSREV1'))
        $writer.Write([byte[]](Convert-BuildAssembleHexToBytes $ProjectId))
        $writer.Write([byte]0)
        $writer.Write([byte[]]::new(32))
        $writer.Write([byte[]](Convert-BuildAssembleHexToBytes $TreeSha256))
        Write-BuildAssembleCanonicalString $writer 'agent_answer'
        Write-BuildAssembleCanonicalString $writer 'none_deterministic'
        $writer.Flush()
        return Get-BuildAssembleSha256Hex $stream.ToArray()
    }
    finally { $writer.Dispose(); $stream.Dispose() }
}

function Test-BuildAssembleShaRef {
    param([object]$Value)
    return $Value -is [string] -and $Value -match '^sha256:[0-9a-f]{64}$' -and
        $Value -ne ('sha256:' + ('0' * 64))
}

function Test-BuildAssembleFileFacts {
    param([object[]]$Actual, [object[]]$Expected)
    if ($Actual.Count -ne $Expected.Count) { return $false }
    for ($index = 0; $index -lt $Expected.Count; $index++) {
        if ($Actual[$index].path -cne $Expected[$index].path -or
            $Actual[$index].classification -ne $Expected[$index].classification -or
            $Actual[$index].media_type -ne $Expected[$index].media_type -or
            [int]$Actual[$index].byte_len -ne $Expected[$index].bytes.Length -or
            $Actual[$index].blob_sha256 -ne "sha256:$($Expected[$index].sha256)") {
            return $false
        }
    }
    return $true
}

function Test-BuildAssembleAllFalse {
    param([object]$Result, [string[]]$Fields)
    foreach ($field in $Fields) {
        if ($null -eq $Result.PSObject.Properties[$field] -or $Result.$field -ne $false) {
            return $false
        }
    }
    return $true
}

function Get-BuildAssembleMarkerLine {
    param([string]$Prefix)
    if (-not (Wait-ForLogText -Path $SerialLog -Needle $Prefix -TimeoutSeconds $TimeoutSeconds)) {
        throw "Timed out waiting for build-assemble marker '$Prefix'"
    }
    $matching = @((Get-SerialLogContent -Path $SerialLog) -split "`n" | Where-Object { $_.Contains($Prefix) })
    if ($matching.Count -eq 0) { throw "Build-assemble marker '$Prefix' disappeared from serial log" }
    $line = [string]$matching[-1]
    $offset = $line.IndexOf($Prefix, [StringComparison]::Ordinal)
    return $line.Substring($offset).TrimEnd([char[]]@("`r", "`n"))
}

function Get-BuildAssembleMarkerCount {
    param([string]$Prefix)
    if (-not (Test-Path -LiteralPath $SerialLog -PathType Leaf)) { return 0 }
    return [regex]::Matches((Get-SerialLogContent -Path $SerialLog), [regex]::Escape($Prefix)).Count
}

function Stop-BuildAssembleBoot {
    if (-not $QemuPid) { throw 'build-assemble cannot stop a boot without a QEMU pid' }
    Close-SerialTcpConnection
    $pidToStop = $QemuPid
    Send-QemuMonitorCommand -Command 'quit' | Out-Null
    $process = Get-Process -Id $pidToStop -ErrorAction SilentlyContinue
    if ($process) { $process.WaitForExit(5000) | Out-Null }
    if (Get-Process -Id $pidToStop -ErrorAction SilentlyContinue) {
        throw 'build-assemble QEMU did not complete clean monitor shutdown'
    }
    $script:QemuPid = $null
    $script:QemuProcess = $null
}

function Start-BuildAssembleBoot {
    param([string]$Name, [string]$StorePath)
    $log = Join-Path $RunDir "serial-build-assemble-$Name.log"
    $params = $runParams.Clone()
    $params.StopExisting = $false
    $params.StructuredStoreDiskPath = $StorePath
    $params.SerialLog = $log
    $output = & $RunScript @params
    $script:SerialLogCachePath = $null
    $script:SerialLogCacheLength = [int64]-1
    $script:SerialLogCacheWriteTicks = [int64]-1
    $script:SerialLogCacheContent = $null
    $script:SerialTcpClient = $null
    $script:SerialTcpDrainStream = $null
    $script:SerialTcpStream = $null
    $script:QemuProcess = $null
    $script:QemuProcessBeforeTeardown = $null
    $script:QemuProcessAfterTeardown = $null
    $script:QemuTeardownAction = 'running'
    $script:SerialLog = $log
    $script:QemuPid = $null
    foreach ($line in $output) {
        if ($line -match '^qemu pid:\s*(\d+)') { $script:QemuPid = [int]$Matches[1] }
    }
    if (-not $QemuPid) { throw "build-assemble $Name boot did not return a QEMU pid" }
    try { $script:QemuProcess = Get-Process -Id $QemuPid -ErrorAction Stop } catch { $script:QemuProcess = $null }
    if (-not (Wait-ForLogText -Path $SerialLog -Needle 'SERIAL CONSOLE READY' -TimeoutSeconds $TimeoutSeconds)) {
        throw "build-assemble $Name serial console did not become ready"
    }
    return $log
}

$rwirRequest = 'Build the fixed B3A-1c RAIOS_WASM_IR_V1 source fixture.'
$rwirContentBase64 = 'UkFJT1NfV0FTTV9JUl9WMQpmdW5jIHJhaW9zX3NlcnZpY2VfbWFpbiAtPiBpMzIKY29uc3QuaTMyIDQyCnJldHVybgplbmQK'
$rwirAnswer = @(
    'RAIOS_SOURCE_FILES_V1'
    "file bWFpbi5yd2ly $rwirContentBase64"
    'end'
) -join "`n"
$rwirBytes = [Convert]::FromBase64String($rwirContentBase64)
$rwirFile = [pscustomobject]@{
    path = 'main.rwir'
    classification = 'local_only'
    media_type = 'text/raios-wasm-ir'
    bytes = $rwirBytes
    sha256 = Get-BuildAssembleSha256Hex $rwirBytes
}
$rwirFiles = @($rwirFile)
$rwirRequestBytes = [Text.Encoding]::UTF8.GetBytes($rwirRequest)
$rwirAnswerBytes = [Text.Encoding]::UTF8.GetBytes($rwirAnswer)
$rwirProjectId = Get-BuildAssembleProjectId $rwirRequestBytes
$rwirRequestSha256 = Get-BuildAssembleSha256Hex $rwirRequestBytes
$rwirAnswerSha256 = Get-BuildAssembleSha256Hex $rwirAnswerBytes
$rwirTreeSha256 = Get-BuildAssembleTreeSha256 $rwirFiles
$rwirRevisionSha256 = Get-BuildAssembleRevisionSha256 $rwirProjectId $rwirTreeSha256
$rwirInputSha256 = $rwirFile.sha256

# Host-independent output pin copied from the literal 52-byte `expected` array in
# crates/raios-wasm-ir/src/lib.rs; this profile never runs the encoder to derive it.
$expectedWasmSha256 = '37b6dae3dbb05625f90dc108f74875b299c943a8ce6e11535ed6e14a9c4bfde2'
$expectedWasmByteLen = 52
$workspaceImportCanonical = "{`r`n  `"service_id`": `"svc.workspace.current_boot`",`r`n  `"imports`": []`r`n}"
$workspaceImportListSha256 = Get-BuildAssembleSha256Hex ([Text.Encoding]::UTF8.GetBytes($workspaceImportCanonical))

# The first denial proves assembly cannot invent a source revision.
$noRevisionResponse = Send-BuildAssembleCommand 'build.assemble_revision' 'build.assemble_revision' 'negative_no_revision'
$noRevision = $noRevisionResponse.body.result

Send-AgentCommand -Command 'services' -ExpectedMarker 'RAIOS_AGENT_END service.inventory' -Name 'build-assemble:services_before'
$servicesBeforeResponse = Get-LastAgentResponseJson -Method 'service.inventory'
Send-AgentCommand -Command 'agent durable.record_log_scan' -ExpectedMarker 'RAIOS_AGENT_END durable.record_log_scan' -Name 'build-assemble:reclog_before'
$reclogBeforeResponse = Get-LastAgentResponseJson -Method 'durable.record_log_scan'
Send-AgentCommand -Command 'agent artifact.store_scan' -ExpectedMarker 'RAIOS_AGENT_END artifact.store_scan' -Name 'build-assemble:artstor_before'
$artstorBeforeResponse = Get-LastAgentResponseJson -Method 'artifact.store_scan'

$fixtureResponse = Send-BuildAssembleCommand 'project.rwir_answer_fixture' 'project.rwir_answer_fixture' 'fixture'
$fixture = $fixtureResponse.body.result
$workspaceResponse = Send-BuildAssembleCommand 'project.workspace' 'project.workspace' 'workspace'
$workspace = $workspaceResponse.body.result
$inspectResponse = Send-BuildAssembleCommand "project.inspect $rwirProjectId" 'project.inspect' 'inspect'
$inspect = $inspectResponse.body.result
$replayResponse = Send-BuildAssembleCommand 'project.rwir_answer_fixture' 'project.rwir_answer_fixture' 'fixture_replay'
$replay = $replayResponse.body.result
$fixtureFilesExact = Test-BuildAssembleFileFacts @($fixture.files) $rwirFiles
$workspaceFilesExact = Test-BuildAssembleFileFacts @($workspace.files) $rwirFiles
$inspectFilesExact = Test-BuildAssembleFileFacts @($inspect.files) $rwirFiles
$fixtureOk = (
    $fixtureResponse.v -eq 'raios.agent.v0' -and $workspaceResponse.v -eq 'raios.agent.v0' -and
    $inspectResponse.v -eq 'raios.agent.v0' -and $replayResponse.v -eq 'raios.agent.v0' -and
    $fixture.method -eq 'project.rwir_answer_fixture' -and $fixture.status -eq 'accepted' -and
    $fixture.reason -eq 'agent_answer_revision_committed' -and $fixture.phase -eq 'source_ready' -and
    $fixture.accepted -eq $true -and $fixture.rejected -eq $false -and
    [int]$fixture.latest_request_id -eq 733212 -and $null -eq $fixture.pending_request_id -and
    $fixture.project_id -eq $rwirProjectId -and $fixture.request_sha256 -eq "sha256:$rwirRequestSha256" -and
    $fixture.answer_sha256 -eq "sha256:$rwirAnswerSha256" -and [int]$fixture.answer_byte_len -eq $rwirAnswerBytes.Length -and
    $fixture.tree_sha256 -eq "sha256:$rwirTreeSha256" -and $fixture.revision_sha256 -eq "sha256:$rwirRevisionSha256" -and
    $fixture.revision_action -eq 'agent_answer' -and $null -eq $fixture.parent_revision_sha256 -and
    [int]$fixture.file_count -eq 1 -and [int]$fixture.total_byte_len -eq 72 -and $fixtureFilesExact -and
    $fixture.answer_origin -eq 'test_fixture' -and $fixture.provider_trust_positive -eq $false -and
    $fixture.test_infrastructure -eq $true -and $fixture.source_authority -eq 'untrusted_agent_candidate' -and
    $fixture.persistence_posture -eq 'qemu_disposable_structured_store_only' -and
    $fixture.committed_files_retention -eq 'qemu_disposable_structured_store' -and $fixture.qemu_only -eq $true -and
    $fixture.storage_write_attempted -eq $true -and $fixture.writes_persistent_state -eq $true -and
    $workspace.method -eq 'project.workspace' -and $workspace.phase -eq 'source_ready' -and
    $workspace.project_id -eq $rwirProjectId -and $workspace.tree_sha256 -eq "sha256:$rwirTreeSha256" -and
    $workspace.revision_sha256 -eq "sha256:$rwirRevisionSha256" -and [int]$workspace.file_count -eq 1 -and
    [int]$workspace.total_byte_len -eq 72 -and $workspaceFilesExact -and
    $inspect.status -eq 'present' -and $inspect.reason -eq 'project_revision_verified' -and
    $inspect.project_id -eq $rwirProjectId -and $inspect.tree_sha256 -eq "sha256:$rwirTreeSha256" -and
    $inspect.revision_sha256 -eq "sha256:$rwirRevisionSha256" -and [int]$inspect.file_count -eq 1 -and
    [int]$inspect.total_byte_len -eq 72 -and $inspectFilesExact -and
    $replay.status -eq 'denied' -and $replay.reason -eq 'agent_answer_request_not_tracked' -and
    $replay.storage_write_attempted -eq $false -and $replay.writes_persistent_state -eq $false -and
    $replay.revision_sha256 -eq "sha256:$rwirRevisionSha256" -and [int]$replay.file_count -eq 1
)
$fixtureDump = [ordered]@{
    fixture = $fixtureResponse; workspace = $workspaceResponse; inspect = $inspectResponse; replay = $replayResponse
    host = [ordered]@{ project_id = $rwirProjectId; request_sha256 = $rwirRequestSha256; answer_sha256 = $rwirAnswerSha256; tree_sha256 = $rwirTreeSha256; revision_sha256 = $rwirRevisionSha256; file = $rwirFile }
}
Add-BuildAssemblePredicate 'fixture-revision-committed' 'fixed test fixture commits exactly local-only root main.rwir, host-recomputed source hashes agree, and replay is untracked' $fixtureOk $(if ($fixtureOk) { "revision=sha256:$rwirRevisionSha256 input=sha256:$rwirInputSha256" } else { $fixtureDump })

$assembleResponse = Send-BuildAssembleCommand 'build.assemble_revision' 'build.assemble_revision' 'assemble_revision'
$assembled = $assembleResponse.body.result
$assembleInert = Test-BuildAssembleAllFalse $assembled @(
    'executed', 'accepts_external_artifact_bytes', 'candidate_intake_attempted',
    'executable_candidate_created', 'maps_produced_artifact_executable_pages', 'load_attempted',
    'load_authorized', 'execution_attempted', 'execution_authorized', 'install_attempted',
    'install_authorized', 'promotion_attempted', 'promotion_authorized', 'service_started',
    'w5_preview_created', 'w6_preview_created', 'reclog_executable_record_written',
    'artstor_executable_record_written', 'writes_persistent_state', 'network_access',
    'secret_access', 'rollback_effect'
)
$assembleOk = (
    $assembleResponse.v -eq 'raios.agent.v0' -and $assembled.method -eq 'build.assemble_revision' -and
    $assembled.status -eq 'accepted' -and $assembled.assemble_outcome -eq 'passed' -and
    $assembled.source_path -eq 'main.rwir' -and $assembled.source_media_type -eq 'text/raios-wasm-ir' -and
    $assembled.source_classification -eq 'local_only' -and $assembled.project_id -eq $rwirProjectId -and
    $assembled.revision_sha256 -eq "sha256:$rwirRevisionSha256" -and
    $assembled.tree_sha256 -eq "sha256:$rwirTreeSha256" -and [int]$assembled.input_byte_len -eq 72 -and
    $assembled.input_sha256 -eq "sha256:$rwirInputSha256" -and $assembled.input_binding_verified_before_assembly -eq $true -and
    $assembled.service_id -eq 'svc.build.assembler' -and
    (Test-BuildAssembleShaRef $assembled.guest_artifact_sha256) -and
    (Test-BuildAssembleShaRef $assembled.guest_descriptor_sha256) -and
    (Test-BuildAssembleShaRef $assembled.guest_signature_envelope_sha256) -and
    $assembled.signed_guest_valid -eq $true -and $assembled.assembler_guest_run_outcome -eq 'success' -and
    $assembled.assembler_guest_executed -eq $true -and $assembled.assembler_guest_run2_outcome -eq 'success' -and
    $assembled.assembler_guest_run2_executed -eq $true -and [int]$assembled.authorized_host_import_count -eq 3 -and
    $assembled.authorized_host_import_abi -eq 'raios.host_imports.v1' -and
    (Test-BuildAssembleShaRef $assembled.authorized_host_import_list_sha256) -and
    [int]$assembled.linked_host_import_count -eq 3 -and $assembled.module_imports_within_authorized_list -eq $true -and
    [int64]$assembled.fuel_budget -eq 1000000 -and [int64]$assembled.run1_fuel_used -gt 0 -and
    [int64]$assembled.run1_fuel_used -lt 1000000 -and [int64]$assembled.run2_fuel_used -gt 0 -and
    [int64]$assembled.run2_fuel_used -lt 1000000 -and [int64]$assembled.memory_limit_bytes -eq 2097152 -and
    $assembled.run1_input_sha256 -eq "sha256:$rwirInputSha256" -and $assembled.run2_input_sha256 -eq "sha256:$rwirInputSha256" -and
    $assembled.guest_output_sha256 -eq "sha256:$expectedWasmSha256" -and
    $assembled.run1_output_sha256 -eq "sha256:$expectedWasmSha256" -and
    $assembled.run2_output_sha256 -eq "sha256:$expectedWasmSha256" -and
    $assembled.kernel_recompute_sha256 -eq "sha256:$expectedWasmSha256" -and
    $assembled.output_sha256 -eq "sha256:$expectedWasmSha256" -and [int]$assembled.output_byte_len -eq $expectedWasmByteLen -and
    $assembled.guest_kernel_byte_identical -eq $true -and $assembled.fresh_builds_byte_identical -eq $true -and
    $assembled.byte_identical -eq $true -and $assembled.wasmi_module_valid -eq $true -and
    [int]$assembled.produced_module_import_count -eq 0 -and $assembled.produced_module_imports_empty -eq $true -and
    $assembled.entrypoint -eq 'raios_service_main' -and $assembled.exact_entrypoint_export -eq $true -and
    $assembled.double_build_scope -eq 'two_independent_guest_instantiations_same_boot' -and
    $assembled.two_fresh_guest_instantiations -eq $true -and $assembled.fresh_store_double_build_across_reboot -eq $false -and
    $assembled.output_retained_in_ram -eq $true -and $assembled.retention_scope -eq 'current_boot' -and
    $assembled.guest_output_inert -eq $true -and $assembled.service_inventory_mutation -eq 'none' -and $assembleInert
)
Add-BuildAssemblePredicate 'revision-assembled-deterministic' 'exact revision and input produce two byte-identical, independently recomputed, wasmi-valid 52-byte zero-import modules matching the literal golden hash while inert' $assembleOk $(if ($assembleOk) { "sha256:$expectedWasmSha256 bytes=$expectedWasmByteLen" } else { $assembleResponse })

$prepareResponse = Send-BuildAssembleCommand 'build.run_prepare' 'build.run_prepare' 'run_prepare'
$prepare = $prepareResponse.body.result
$pendingMarkerExpected = "BUILD_RUN_PENDING result=accepted revision_sha256=sha256:$rwirRevisionSha256 tree_sha256=sha256:$rwirTreeSha256 input_sha256=sha256:$rwirInputSha256 output_sha256=sha256:$expectedWasmSha256 imports=0 entrypoint=raios_service_main fuel_budget=250000 memory_limit=4194304 scope=current_boot approval=genesis_pointer_required"
$pendingMarkerLine = Get-BuildAssembleMarkerLine 'BUILD_RUN_PENDING result='
$challengeSha256 = [string]$prepare.approval_challenge_sha256
$prepareOk = (
    $prepareResponse.v -eq 'raios.agent.v0' -and $prepare.status -eq 'accepted' -and
    $prepare.reason -eq 'build_run_pending_physical_approval' -and $prepare.accepted -eq $true -and
    $prepare.rejected -eq $false -and $prepare.project_id -eq $rwirProjectId -and
    $prepare.revision_sha256 -eq "sha256:$rwirRevisionSha256" -and $prepare.tree_sha256 -eq "sha256:$rwirTreeSha256" -and
    $prepare.input_sha256 -eq "sha256:$rwirInputSha256" -and [int]$prepare.input_byte_len -eq 72 -and
    $prepare.output_sha256 -eq "sha256:$expectedWasmSha256" -and [int]$prepare.output_byte_len -eq 52 -and
    $prepare.entrypoint -eq 'raios_service_main' -and [int64]$prepare.fuel_budget -eq 250000 -and
    [int64]$prepare.memory_limit_bytes -eq 4194304 -and [int]$prepare.instance_limit -eq 1 -and
    [int]$prepare.memory_count_limit -eq 1 -and [int]$prepare.table_limit -eq 0 -and
    [int]$prepare.import_count -eq 0 -and $prepare.produced_module_imports_empty -eq $true -and
    $prepare.import_list_sha256 -eq "sha256:$workspaceImportListSha256" -and
    (Test-BuildAssembleShaRef $challengeSha256) -and $prepare.phase -eq 'pending_physical_approval' -and
    $prepare.w5_preview_created -eq $true -and $prepare.physical_approval_present -eq $false -and
    $prepare.approval_source -eq 'genesis_pointer' -and $null -eq $prepare.approval_sha256 -and
    $prepare.serial_approval_authorized -eq $false -and
    $prepare.serial_approval_reason -eq 'workspace_physical_pointer_approval_required' -and
    [int]$prepare.run_count -eq 0 -and $prepare.run_outcome -eq 'not_run' -and
    $null -eq $prepare.return_value_i32_bits -and [int64]$prepare.fuel_used -eq 0 -and
    $prepare.executed -eq $false -and $prepare.one_shot_requires_fresh_prepare -eq $true -and
    $pendingMarkerLine -ceq $pendingMarkerExpected
)
$prepareDump = [ordered]@{ response = $prepareResponse; expected_marker = $pendingMarkerExpected; actual_marker = $pendingMarkerLine; import_list_sha256 = $workspaceImportListSha256 }
Add-BuildAssemblePredicate 'w5-preview-bound' 'W5 preview and trimmed BUILD_RUN_PENDING marker bind revision, tree, input, output, empty imports, entrypoint, envelope, and one-shot Genesis approval' $prepareOk $(if ($prepareOk) { "challenge=$challengeSha256" } else { $prepareDump })

$serialApprovalResponse = Send-BuildAssembleCommand 'project.run_approve' 'project.run_approve' 'serial_approval'
$serialApproval = $serialApprovalResponse.body.result
$serialStartResponse = Send-BuildAssembleCommand 'service.start svc.workspace.current_boot' 'service.start' 'serial_start_before_approval'
$serialStart = $serialStartResponse.body.result
$serialDeniedOk = (
    $serialApprovalResponse.v -eq 'raios.agent.v0' -and $serialApproval.status -eq 'denied' -and
    $serialApproval.reason -eq 'workspace_physical_pointer_approval_required' -and
    $serialApproval.phase -eq 'pending_physical_approval' -and $serialApproval.physical_approval_present -eq $false -and
    $serialApproval.serial_approval_authorized -eq $false -and [int]$serialApproval.run_count -eq 0 -and
    $serialApproval.writes_persistent_state -eq $false -and $serialApproval.storage_write -eq $false -and
    $serialStartResponse.v -eq 'raios.agent.v0' -and $serialStart.status -eq 'denied' -and
    $serialStart.reason -eq 'workspace_physical_approval_required' -and
    $serialStart.phase -eq 'pending_physical_approval' -and [int]$serialStart.run_count -eq 0 -and
    $serialStart.writes_persistent_state -eq $false
)
$serialDeniedDump = [ordered]@{ approve = $serialApprovalResponse; start = $serialStartResponse }
Add-BuildAssemblePredicate 'serial-approve-denied' 'project.run_approve and serial service.start cannot substitute for the pending Genesis pointer' $serialDeniedOk $(if ($serialDeniedOk) { 'workspace_physical_pointer_approval_required; run_count=0' } else { $serialDeniedDump })

Save-QemuScreendump -Name 'build-assemble-awaiting-physical-approval' | Out-Null
$activationPrefix = 'BUILD_RUN_ACTIVATION physical_approval=pointer'
$activationCountBefore = Get-BuildAssembleMarkerCount $activationPrefix
Send-QemuAbsolutePointerClick -X 27017 -Y 6559
$activationMarkerExpected = "BUILD_RUN_ACTIVATION physical_approval=pointer result=accepted revision_sha256=sha256:$rwirRevisionSha256 tree_sha256=sha256:$rwirTreeSha256 input_sha256=sha256:$rwirInputSha256 output_sha256=sha256:$expectedWasmSha256 imports=0 entrypoint=raios_service_main fuel_budget=250000 memory_limit=4194304 run_outcome=success return_value=42 reason=workspace_physical_pointer_approved install=false persistence=false"
$activationMarkerLine = Get-BuildAssembleMarkerLine $activationPrefix
$activationCountAfter = Get-BuildAssembleMarkerCount $activationPrefix
$postRunResponse = Send-BuildAssembleCommand 'build.run_prepare' 'build.run_prepare' 'post_run_status'
$postRun = $postRunResponse.body.result
$physicalOk = (
    $activationMarkerLine -ceq $activationMarkerExpected -and $activationCountAfter -eq ($activationCountBefore + 1) -and
    $postRunResponse.v -eq 'raios.agent.v0' -and $postRun.status -eq 'denied' -and
    $postRun.reason -eq 'workspace_service_slot_occupied' -and $postRun.phase -eq 'running' -and
    $postRun.revision_sha256 -eq "sha256:$rwirRevisionSha256" -and $postRun.tree_sha256 -eq "sha256:$rwirTreeSha256" -and
    $postRun.input_sha256 -eq "sha256:$rwirInputSha256" -and $postRun.output_sha256 -eq "sha256:$expectedWasmSha256" -and
    [int]$postRun.import_count -eq 0 -and $postRun.entrypoint -eq 'raios_service_main' -and
    $postRun.physical_approval_present -eq $true -and $postRun.approval_source -eq 'genesis_pointer' -and
    (Test-BuildAssembleShaRef $postRun.approval_sha256) -and [int]$postRun.run_count -eq 1 -and
    $postRun.run_outcome -eq 'success' -and [int]$postRun.return_value_i32_bits -eq 42 -and
    [int64]$postRun.fuel_used -gt 0 -and [int64]$postRun.fuel_used -lt 250000 -and
    $postRun.executed -eq $true -and $postRun.install_attempted -eq $false -and
    $postRun.install_authorized -eq $false -and $postRun.promotion_attempted -eq $false -and
    $postRun.promotion_authorized -eq $false -and $postRun.w6_preview_created -eq $false -and
    $postRun.reclog_executable_record_written -eq $false -and $postRun.artstor_executable_record_written -eq $false -and
    $postRun.writes_persistent_state -eq $false -and $postRun.network_access -eq $false -and
    $postRun.secret_access -eq $false -and $postRun.rollback_effect -eq $false
)
$physicalDump = [ordered]@{ expected_marker = $activationMarkerExpected; actual_marker = $activationMarkerLine; marker_count_before = $activationCountBefore; marker_count_after = $activationCountAfter; status = $postRunResponse }
Add-BuildAssemblePredicate 'physical-click-runs-42' 'one QMP Genesis click emits one exact accepted activation and the produced module runs once to i32 42 with no install or persistence effect' $physicalOk $(if ($physicalOk) { "run_count=1 return=42 fuel=$($postRun.fuel_used)" } else { $physicalDump })

# Stop the one-shot slot first, then capture the drift scans at the exact
# post-build-run state. The stale-click experiments run AFTER the scans because
# a consumed click legitimately falls through to the personal-shell surface
# (existing Genesis behavior) and would otherwise start svc.user.shell before
# the drift measurement.
$stopResponse = Send-BuildAssembleCommand 'service.stop svc.workspace.current_boot' 'service.stop' 'stop_after_run'
$stopped = $stopResponse.body.result

Send-AgentCommand -Command 'services' -ExpectedMarker 'RAIOS_AGENT_END service.inventory' -Name 'build-assemble:services_after'
$servicesAfterResponse = Get-LastAgentResponseJson -Method 'service.inventory'
Send-AgentCommand -Command 'agent durable.record_log_scan' -ExpectedMarker 'RAIOS_AGENT_END durable.record_log_scan' -Name 'build-assemble:reclog_after'
$reclogAfterResponse = Get-LastAgentResponseJson -Method 'durable.record_log_scan'
Send-AgentCommand -Command 'agent artifact.store_scan' -ExpectedMarker 'RAIOS_AGENT_END artifact.store_scan' -Name 'build-assemble:artstor_after'
$artstorAfterResponse = Get-LastAgentResponseJson -Method 'artifact.store_scan'
$servicesBefore = @($servicesBeforeResponse.facts.services)
$servicesAfter = @($servicesAfterResponse.facts.services)
$workspaceRows = @($servicesAfter | Where-Object { $_.id -eq 'svc.workspace.current_boot' })
$servicesWithoutWorkspace = @($servicesAfter | Where-Object { $_.id -ne 'svc.workspace.current_boot' })
$serviceBaseUnchanged = (($servicesBefore | ConvertTo-Json -Compress -Depth 16) -ceq
    ($servicesWithoutWorkspace | ConvertTo-Json -Compress -Depth 16))
$workspaceCurrentBootExact = (
    $workspaceRows.Count -eq 1 -and $workspaceRows[0].kind -eq 'wasm_service' -and
    $workspaceRows[0].scope -eq 'current_boot' -and $workspaceRows[0].persistence -eq 'none' -and
    $workspaceRows[0].phase -eq 'stopped' -and $workspaceRows[0].running -eq $false -and
    [int]$workspaceRows[0].generation -eq 1 -and [int]$workspaceRows[0].host_import_count -eq 0 -and
    @($workspaceRows[0].granted_host_imports).Count -eq 0 -and $null -eq $workspaceRows[0].candidate_sha256 -and
    $null -eq $workspaceRows[0].receipt_sha256 -and [int]$workspaceRows[0].run_count -eq 1 -and
    [int]$workspaceRows[0].last_return_value_i32 -eq 42 -and $workspaceRows[0].last_action -eq 'success'
)
$zeroDriftOk = (
    $serviceBaseUnchanged -and $workspaceCurrentBootExact -and
    (($reclogBeforeResponse.body.result | ConvertTo-Json -Compress -Depth 24) -ceq
        ($reclogAfterResponse.body.result | ConvertTo-Json -Compress -Depth 24)) -and
    (($artstorBeforeResponse.body.result | ConvertTo-Json -Compress -Depth 24) -ceq
        ($artstorAfterResponse.body.result | ConvertTo-Json -Compress -Depth 24))
)
$zeroDriftDump = [ordered]@{ services_before = $servicesBeforeResponse; services_after = $servicesAfterResponse; reclog_before = $reclogBeforeResponse; reclog_after = $reclogAfterResponse; artstor_before = $artstorBeforeResponse; artstor_after = $artstorAfterResponse }

# Stale-click experiments AFTER the drift scans: the second click may
# legitimately open the empty personal shell; nothing measures drift afterwards.
Send-QemuAbsolutePointerClick -X 27017 -Y 6559
Start-Sleep -Milliseconds 500
$activationCountAfterSecondClick = Get-BuildAssembleMarkerCount $activationPrefix
$restartResponse = Send-BuildAssembleCommand 'service.start svc.workspace.current_boot' 'service.start' 'restart_without_prepare'
$restart = $restartResponse.body.result
$stoppedReplayResponse = Send-BuildAssembleCommand 'build.run_prepare' 'build.run_prepare' 'stopped_replay_prepare'
$stoppedReplay = $stoppedReplayResponse.body.result
$secondActivationOk = (
    $activationCountAfterSecondClick -eq $activationCountAfter -and
    $stopResponse.v -eq 'raios.agent.v0' -and $stopped.status -eq 'accepted' -and
    $stopped.reason -eq 'workspace_service_stopped' -and $stopped.phase -eq 'stopped' -and
    [int]$stopped.run_count -eq 1 -and [int]$stopped.return_value_i32_bits -eq 42 -and
    $restartResponse.v -eq 'raios.agent.v0' -and $restart.status -eq 'denied' -and
    $restart.reason -eq 'build_run_fresh_prepare_required' -and $restart.phase -eq 'stopped' -and
    [int]$restart.run_count -eq 1 -and [int]$restart.return_value_i32_bits -eq 42 -and
    $stoppedReplayResponse.v -eq 'raios.agent.v0' -and $stoppedReplay.status -eq 'denied' -and
    $stoppedReplay.reason -eq 'workspace_service_slot_occupied' -and $stoppedReplay.phase -eq 'stopped' -and
    [int]$stoppedReplay.run_count -eq 1 -and [int]$stoppedReplay.return_value_i32_bits -eq 42
)
$secondActivationDump = [ordered]@{ marker_count_after_run = $activationCountAfter; marker_count_after_second_click = $activationCountAfterSecondClick; stop = $stopResponse; restart = $restartResponse; replay_prepare = $stoppedReplayResponse }
Add-BuildAssemblePredicate 'second-activation-stale-denied' 'a consumed click emits no second activation; after stop, prepare-less start is build_run_fresh_prepare_required and replay prepare sees the occupied one-shot slot' $secondActivationOk $(if ($secondActivationOk) { 'one activation; build_run_fresh_prepare_required; workspace_service_slot_occupied' } else { $secondActivationDump })

# A fresh disposable store makes the existing two-file B2 fixture a real
# wrong-shape revision without changing the kernel or corrupting the positive proof.
$positiveLog = $SerialLog
Stop-BuildAssembleBoot
$negativeStorePath = Join-Path $RunDir 'raios-structured-store-build-assemble-negative.img'
$negativeStoreError = Join-Path $RunDir 'build-assemble-negative-store.err.txt'
$negativeStoreJson = & python (Join-Path $RepoRoot 'scripts\make-structured-store-image.py') create $negativeStorePath --size-mib 16 --json 2> $negativeStoreError
if ($LASTEXITCODE -ne 0) {
    $negativeStoreErrorText = Get-Content -LiteralPath $negativeStoreError -Raw -ErrorAction SilentlyContinue
    throw "build-assemble negative structured-store creation failed: $negativeStoreErrorText"
}
$negativeStore = ($negativeStoreJson -join [Environment]::NewLine) | ConvertFrom-Json
if (-not $negativeStore.valid -or -not $negativeStore.disposable_qemu_only -or
    $negativeStore.store_state -ne 'empty_unformatted') {
    throw "build-assemble negative structured-store fixture invalid: $($negativeStore | ConvertTo-Json -Compress -Depth 16)"
}
$negativeStorePath = (Resolve-Path -LiteralPath $negativeStorePath).Path
$negativeLog = Start-BuildAssembleBoot 'negative-main-rwir' $negativeStorePath
$wrongFixtureResponse = Send-BuildAssembleCommand 'project.agent_answer_fixture' 'project.agent_answer_fixture' 'negative_wrong_fixture'
$wrongFixture = $wrongFixtureResponse.body.result
$missingMainResponse = Send-BuildAssembleCommand 'build.assemble_revision' 'build.assemble_revision' 'negative_missing_main_rwir'
$missingMain = $missingMainResponse.body.result
$negativeTableOk = (
    $noRevisionResponse.v -eq 'raios.agent.v0' -and $noRevision.status -eq 'denied' -and
    $noRevision.assemble_outcome -eq 'build_assemble_project_missing' -and
    $noRevision.input_binding_verified_before_assembly -eq $false -and $noRevision.output_retained_in_ram -eq $false -and
    $wrongFixtureResponse.v -eq 'raios.agent.v0' -and $wrongFixture.status -eq 'accepted' -and
    $wrongFixture.reason -eq 'agent_answer_revision_committed' -and [int]$wrongFixture.file_count -eq 2 -and
    $wrongFixture.answer_origin -eq 'test_fixture' -and $wrongFixture.provider_trust_positive -eq $false -and
    $wrongFixture.test_infrastructure -eq $true -and
    $missingMainResponse.v -eq 'raios.agent.v0' -and $missingMain.status -eq 'denied' -and
    $missingMain.assemble_outcome -eq 'build_assemble_main_rwir_missing' -and
    $missingMain.input_binding_verified_before_assembly -eq $false -and $missingMain.output_retained_in_ram -eq $false
)
$negativeDump = [ordered]@{ no_revision = $noRevisionResponse; wrong_fixture = $wrongFixtureResponse; missing_main_rwir = $missingMainResponse }
Add-BuildAssemblePredicate 'negative-table' 'assembly before any agent revision denies build_assemble_project_missing, and a fresh-store two-file B2 revision denies build_assemble_main_rwir_missing' $negativeTableOk $(if ($negativeTableOk) { 'build_assemble_project_missing; build_assemble_main_rwir_missing' } else { $negativeDump })

Add-BuildAssemblePredicate 'zero-executable-drift' 'only the expected stopped current-boot workspace fact appears; all other services plus RECLOG and ARTSTOR scans remain byte-identical' $zeroDriftOk $(if ($zeroDriftOk) { 'one stopped current_boot row; durable scans unchanged' } else { $zeroDriftDump })

# Stop the negative boot before combining: QEMU holds an exclusive write handle
# on its serial log, so ReadAllText would fail while the VM is still running.
Stop-BuildAssembleBoot
$combinedLog = Join-Path $RunDir 'serial-build-assemble-combined.log'
$combinedContent = [IO.File]::ReadAllText($positiveLog) + "`n" + [IO.File]::ReadAllText($negativeLog)
[IO.File]::WriteAllText($combinedLog, $combinedContent, [Text.UTF8Encoding]::new($false))
$SerialLog = $combinedLog
