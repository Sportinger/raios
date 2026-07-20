function Send-WorkspaceCommand {
    param([string]$Command, [string]$Method, [string]$Name)
    Send-AgentCommand -Command $Command -ExpectedMarker "RAIOS_AGENT_END $Method" -Name "project-app:$Name"
    return (Get-LastAgentResponseJson -Method $Method).body.result
}

function Assert-WorkspacePosture {
    param([object]$Result, [string]$Name)
    Add-BuildPredicate -Name "app_$Name`:posture" -Expected 'local-only current-boot Wasm execution without install, promotion, native load or persistence authority' -Passed (
        $Result.scope -eq 'current_boot' -and $Result.classification -eq 'local_only' -and
        $Result.persistence_posture -eq 'ram_only' -and
        $Result.service_id -eq 'svc.workspace.current_boot' -and
        $Result.trust_tier -eq 'owner_workstation_build_plus_physical_current_boot_not_owner_sealed' -and
        $Result.serial_approval_authorized -eq $false -and
        $Result.writes_persistent_state -eq $false -and $Result.storage_write -eq $false -and
        $Result.authorizes_promotion -eq $false -and $Result.authorizes_install -eq $false -and
        $Result.authorizes_native_load -eq $false -and $Result.authorizes_persistence -eq $false
    ) -Actual $Result
}

function Assert-WorkspaceDenied {
    param([object]$Result, [string]$Reason, [string]$Name)
    Add-BuildPredicate -Name "app_$Name`:denied" -Expected "$Reason with no execution or authority" -Passed (
        $Result.status -eq 'denied' -and $Result.reason -eq $Reason -and
        $Result.accepted -eq $false -and $Result.rejected -eq $true -and
        [int]$Result.run_count -eq 0 -and $Result.physical_approval_present -eq $false
    ) -Actual $Result
    Assert-WorkspacePosture $Result $Name
}

function Get-WorkspaceApprovalSha256 {
    param([string]$ChallengeSha256)
    $bytes = [byte[]](@(
        [Text.Encoding]::ASCII.GetBytes('raios.workspace_physical_approval.v1')
        Convert-HexToBytes $ChallengeSha256.Substring(7)
        [Text.Encoding]::ASCII.GetBytes('genesis_pointer')
    ) | ForEach-Object { $_ })
    return Get-ByteSha256Hex $bytes
}

function Get-WorkspaceMarkerCount {
    param([string]$Marker)
    if (-not (Test-Path -LiteralPath $SerialLog -PathType Leaf)) { return 0 }
    return [regex]::Matches((Get-SerialLogContent -Path $SerialLog), [regex]::Escape($Marker)).Count
}

function Get-CanonicalWorkspaceImportListSha256 {
    # Mirrors raios-core record::Value::Object serialization independently on
    # the host. CRLF, indentation and field order are part of that canonical
    # byte contract; an empty observed list is still service-id bound.
    $canonical = "{`r`n  `"service_id`": `"svc.workspace.current_boot`",`r`n  `"imports`": []`r`n}"
    return Get-ByteSha256Hex ([Text.Encoding]::UTF8.GetBytes($canonical))
}

$workspaceImportListSha256 = Get-CanonicalWorkspaceImportListSha256
$workspacePrepare = "project.run_prepare $projectId sha256:$projectRevisionSha256 sha256:$receiptSha256 sha256:$candidateSha256"

$initial = Send-WorkspaceCommand 'project.run_status' 'project.run_status' 'initial_status'
Add-BuildPredicate -Name 'app_initial:missing' -Expected 'workspace service absent before exact prepare' -Passed (
    $initial.status -eq 'accepted' -and $initial.reason -eq 'workspace_status_read' -and
    $initial.phase -eq 'missing' -and [int]$initial.run_count -eq 0 -and
    $initial.physical_approval_present -eq $false
) -Actual $initial
Assert-WorkspacePosture $initial 'initial'

$staleRevision = Send-WorkspaceCommand `
    "project.run_prepare $projectId sha256:$('8' * 64) sha256:$receiptSha256 sha256:$candidateSha256" `
    'project.run_prepare' 'negative_stale_revision'
Assert-WorkspaceDenied $staleRevision 'build_project_revision_stale' 'negative_stale_revision'

$tamperedReceipt = Send-WorkspaceCommand `
    "project.run_prepare $projectId sha256:$projectRevisionSha256 sha256:$('9' * 64) sha256:$candidateSha256" `
    'project.run_prepare' 'negative_tampered_receipt'
Assert-WorkspaceDenied $tamperedReceipt 'workspace_receipt_not_found' 'negative_tampered_receipt'

$tamperedCandidate = Send-WorkspaceCommand `
    "project.run_prepare $projectId sha256:$projectRevisionSha256 sha256:$receiptSha256 sha256:$('a' * 64)" `
    'project.run_prepare' 'negative_tampered_candidate'
Assert-WorkspaceDenied $tamperedCandidate 'workspace_candidate_mismatch' 'negative_tampered_candidate'

$prepared = Send-WorkspaceCommand $workspacePrepare 'project.run_prepare' 'prepare'
$challengeSha256 = [string]$prepared.approval_challenge_sha256
Add-BuildPredicate -Name 'app_prepare:exact_binding' -Expected 'exact W4 revision, receipt, candidate, observed empty import surface and bounded runtime await physical approval' -Passed (
    $prepared.status -eq 'accepted' -and $prepared.reason -eq 'workspace_run_pending_physical_approval' -and
    $prepared.accepted -eq $true -and $prepared.rejected -eq $false -and
    $prepared.phase -eq 'pending_physical_approval' -and $prepared.health -eq 'starting' -and
    $prepared.project_id -eq $projectId -and
    $prepared.project_revision_sha256 -eq "sha256:$projectRevisionSha256" -and
    $prepared.source_tree_sha256 -eq "sha256:$projectTreeSha256" -and
    $prepared.cargo_lock_sha256 -eq "sha256:$cargoLockSha256" -and
    $prepared.input_manifest_sha256 -eq "sha256:$inputManifestSha256" -and
    $prepared.receipt_sha256 -eq "sha256:$receiptSha256" -and
    $prepared.candidate_sha256 -eq "sha256:$candidateSha256" -and
    [int]$prepared.candidate_byte_len -eq $candidateByteLen -and
    $prepared.import_list_observed -eq $true -and [int]$prepared.import_count -eq 0 -and
    $prepared.import_list_sha256 -eq "sha256:$workspaceImportListSha256" -and
    $prepared.entrypoint -eq 'raios_service_main' -and [int64]$prepared.fuel_budget -eq 250000 -and
    [int64]$prepared.memory_limit_bytes -eq 4194304 -and [int]$prepared.instance_limit -eq 1 -and
    [int]$prepared.memory_count_limit -eq 1 -and [int]$prepared.table_limit -eq 0 -and
    $challengeSha256 -match '^sha256:[0-9a-f]{64}$' -and
    $prepared.physical_approval_present -eq $false -and [int]$prepared.generation -eq 0 -and
    [int]$prepared.run_count -eq 0 -and $prepared.run_outcome -eq 'not_run'
) -Actual $prepared
Assert-WorkspacePosture $prepared 'prepare'
$pendingMarker = "WORKSPACE_CURRENT_BOOT_PENDING result=accepted candidate_sha256=sha256:$candidateSha256 receipt_sha256=sha256:$receiptSha256 imports=0 memory_limit=4194304 scope=current_boot approval=genesis_pointer_required"
Assert-LogContains -Name 'project-app:visible_exact_pending_preview' -Needle $pendingMarker -TimeoutSeconds $TimeoutSeconds

$serialApproval = Send-WorkspaceCommand 'project.run_approve' 'project.run_approve' 'negative_serial_approval'
Add-BuildPredicate -Name 'app_negative_serial_approval:denied' -Expected 'serial can inspect but cannot approve the exact pending run' -Passed (
    $serialApproval.status -eq 'denied' -and
    $serialApproval.reason -eq 'workspace_physical_pointer_approval_required' -and
    $serialApproval.phase -eq 'pending_physical_approval' -and
    $serialApproval.approval_challenge_sha256 -eq $challengeSha256 -and
    $serialApproval.physical_approval_present -eq $false -and [int]$serialApproval.run_count -eq 0
) -Actual $serialApproval
Assert-WorkspacePosture $serialApproval 'negative_serial_approval'

$serialStart = Send-WorkspaceCommand 'service.start svc.workspace.current_boot' 'service.start' 'negative_serial_start_before_approval'
Add-BuildPredicate -Name 'app_negative_serial_start:denied' -Expected 'service.start cannot substitute for physical pointer approval' -Passed (
    $serialStart.status -eq 'denied' -and
    $serialStart.reason -eq 'workspace_physical_approval_required' -and
    $serialStart.phase -eq 'pending_physical_approval' -and [int]$serialStart.run_count -eq 0
) -Actual $serialStart
Assert-WorkspacePosture $serialStart 'negative_serial_start'

Save-QemuScreendump -Name 'workspace-app-awaiting-physical-approval' | Out-Null
$activationCountBefore = Get-WorkspaceMarkerCount 'WORKSPACE_CURRENT_BOOT_ACTIVATION physical_approval=pointer'
Send-QemuAbsolutePointerClick -X 27017 -Y 6559
$activationMarker = "WORKSPACE_CURRENT_BOOT_ACTIVATION physical_approval=pointer result=accepted candidate_sha256=sha256:$candidateSha256 receipt_sha256=sha256:$receiptSha256 imports=0 entrypoint=raios_service_main fuel_budget=250000 memory_limit=4194304 run_outcome=success return_value=42"
Assert-LogContains -Name 'project-app:physical_activation_result_42' -Needle $activationMarker -TimeoutSeconds $TimeoutSeconds
$activationCountAfter = Get-WorkspaceMarkerCount 'WORKSPACE_CURRENT_BOOT_ACTIVATION physical_approval=pointer'
Add-BuildPredicate -Name 'app_physical_activation:exactly_once' -Expected 'one QMP USB-tablet pointer approval produces exactly one activation marker' -Passed (
    $activationCountAfter -eq ($activationCountBefore + 1)
) -Actual "before=$activationCountBefore after=$activationCountAfter"
Save-QemuScreendump -Name 'workspace-app-result-42' | Out-Null

$running = Send-WorkspaceCommand 'project.run_status' 'project.run_status' 'running_status'
$expectedApprovalSha256 = Get-WorkspaceApprovalSha256 $challengeSha256
Add-BuildPredicate -Name 'app_running:result_and_approval' -Expected 'pointer-bound approval runs exact candidate once to result 42 under zero imports and bounded fuel' -Passed (
    $running.status -eq 'accepted' -and $running.reason -eq 'workspace_status_read' -and
    $running.phase -eq 'running' -and $running.health -eq 'healthy' -and
    $running.receipt_sha256 -eq "sha256:$receiptSha256" -and
    $running.candidate_sha256 -eq "sha256:$candidateSha256" -and
    $running.approval_challenge_sha256 -eq $challengeSha256 -and
    $running.physical_approval_present -eq $true -and $running.approval_source -eq 'genesis_pointer' -and
    $running.approval_sha256 -eq "sha256:$expectedApprovalSha256" -and
    [int]$running.generation -eq 1 -and [int]$running.run_count -eq 1 -and
    $running.run_outcome -eq 'success' -and [int]$running.return_value_i32 -eq 42 -and
    [int64]$running.fuel_used -gt 0 -and [int64]$running.fuel_used -lt 250000 -and
    $running.import_list_observed -eq $true -and [int]$running.import_count -eq 0 -and
    $running.import_list_sha256 -eq "sha256:$workspaceImportListSha256"
) -Actual $running
Assert-WorkspacePosture $running 'running'

$health = Send-WorkspaceCommand 'service.health svc.workspace.current_boot' 'service.health' 'health'
Add-BuildPredicate -Name 'app_health:exact' -Expected 'workspace health exposes the same running exact-hash result-42 lifecycle' -Passed (
    $health.status -eq 'accepted' -and $health.reason -eq 'workspace_health_read' -and
    $health.phase -eq 'running' -and $health.health -eq 'healthy' -and
    $health.candidate_sha256 -eq "sha256:$candidateSha256" -and
    $health.receipt_sha256 -eq "sha256:$receiptSha256" -and
    [int]$health.run_count -eq 1 -and [int]$health.return_value_i32 -eq 42
) -Actual $health
Assert-WorkspacePosture $health 'health'

Send-AgentCommand -Command 'services' -ExpectedMarker 'RAIOS_AGENT_END service.inventory' -Name 'project-app:running_inventory'
$inventory = (Get-LastAgentResponseJson -Method 'service.inventory').facts
$workspaceRows = @($inventory.services | Where-Object { $_.id -eq 'svc.workspace.current_boot' })
Add-BuildPredicate -Name 'app_inventory:running' -Expected 'one current-boot zero-import workspace service row binds the exact candidate and receipt' -Passed (
    $workspaceRows.Count -eq 1 -and $workspaceRows[0].kind -eq 'wasm_service' -and
    $workspaceRows[0].scope -eq 'current_boot' -and $workspaceRows[0].persistence -eq 'none' -and
    $workspaceRows[0].phase -eq 'running' -and $workspaceRows[0].running -eq $true -and
    [int]$workspaceRows[0].host_import_count -eq 0 -and @($workspaceRows[0].granted_host_imports).Count -eq 0 -and
    $workspaceRows[0].candidate_sha256 -eq "sha256:$candidateSha256" -and
    $workspaceRows[0].receipt_sha256 -eq "sha256:$receiptSha256" -and
    [int]$workspaceRows[0].run_count -eq 1 -and [int]$workspaceRows[0].last_return_value_i32 -eq 42
) -Actual $workspaceRows

# The project-install profile continues from this exact healthy W5 binding.
# Its separate signed install preview and second physical approval must happen
# before the W5-only replay and F12 teardown below clear the candidate.
if ($Profile -eq 'project-install') {
    return
}

$replayPrepare = Send-WorkspaceCommand $workspacePrepare 'project.run_prepare' 'negative_replay_prepare'
Add-BuildPredicate -Name 'app_negative_replay_prepare:denied' -Expected 'the running one-shot slot rejects a replay without a second run' -Passed (
    $replayPrepare.status -eq 'denied' -and $replayPrepare.reason -eq 'workspace_service_slot_occupied' -and
    $replayPrepare.phase -eq 'running' -and [int]$replayPrepare.run_count -eq 1 -and
    [int]$replayPrepare.return_value_i32 -eq 42
) -Actual $replayPrepare
Assert-WorkspacePosture $replayPrepare 'negative_replay_prepare'

$replayStart = Send-WorkspaceCommand 'service.start svc.workspace.current_boot' 'service.start' 'replay_start'
Add-BuildPredicate -Name 'app_replay_start:no_second_execution' -Expected 'start on an already-running current-boot app is idempotent and does not execute twice' -Passed (
    $replayStart.status -eq 'accepted' -and $replayStart.reason -eq 'workspace_service_already_running' -and
    [int]$replayStart.run_count -eq 1 -and [int]$replayStart.return_value_i32 -eq 42 -and
    (Get-WorkspaceMarkerCount 'WORKSPACE_CURRENT_BOOT_ACTIVATION physical_approval=pointer') -eq $activationCountAfter
) -Actual $replayStart
Assert-WorkspacePosture $replayStart 'replay_start'

Send-QemuMonitorCommand -Command 'sendkey f12' | Out-Null
Assert-LogContains -Name 'project-app:f12_drop' -Needle 'WORKSPACE_CURRENT_BOOT_RECOVERY secure_attention=F12 action=drop_to_genesis result=accepted' -TimeoutSeconds $TimeoutSeconds
Assert-LogContains -Name 'project-app:f12_recovery_open' -Needle 'GENESIS_RECOVERY_VIEW_OPENED current_boot=true' -TimeoutSeconds $TimeoutSeconds
Save-QemuScreendump -Name 'workspace-app-f12-core-recovery' | Out-Null

$afterF12 = Send-WorkspaceCommand 'project.run_status' 'project.run_status' 'after_f12_status'
Add-BuildPredicate -Name 'app_f12:cleared' -Expected 'secure attention drops the current-boot service and clears its approval, binding and run state' -Passed (
    $afterF12.status -eq 'accepted' -and $afterF12.phase -eq 'missing' -and
    $null -eq $afterF12.candidate_sha256 -and $null -eq $afterF12.receipt_sha256 -and
    $afterF12.physical_approval_present -eq $false -and [int]$afterF12.generation -eq 0 -and
    [int]$afterF12.run_count -eq 0 -and $afterF12.run_outcome -eq 'not_run'
) -Actual $afterF12
Assert-WorkspacePosture $afterF12 'after_f12'

Send-AgentCommand -Command 'services' -ExpectedMarker 'RAIOS_AGENT_END service.inventory' -Name 'project-app:after_f12_inventory'
$afterF12Inventory = (Get-LastAgentResponseJson -Method 'service.inventory').facts
$afterF12Workspace = @($afterF12Inventory.services | Where-Object { $_.id -eq 'svc.workspace.current_boot' })
$afterF12Genesis = @($afterF12Inventory.services | Where-Object { $_.id -eq 'core.ui.genesis' })
Add-BuildPredicate -Name 'app_f12:inventory_removed' -Expected 'workspace row is absent and immutable core Genesis remains after F12' -Passed (
    $afterF12Workspace.Count -eq 0 -and $afterF12Genesis.Count -eq 1 -and
    $afterF12Genesis[0].core_owned -eq $true -and $afterF12Genesis[0].replaceable -eq $false
) -Actual $afterF12Inventory.services

$candidateCleared = Send-WorkspaceCommand $workspacePrepare 'project.run_prepare' 'negative_candidate_cleared_after_f12'
Assert-WorkspaceDenied $candidateCleared 'workspace_candidate_missing' 'negative_candidate_cleared_after_f12'

Send-AgentCommand -Command 'agent recovery.snapshot' -ExpectedMarker 'RAIOS_AGENT_END recovery.snapshot' -Name 'project-app:recovery_after_f12'
$recovery = (Get-LastAgentResponseJson -Method 'recovery.snapshot').body.result
Add-BuildPredicate -Name 'app_f12:recovery_lifeline' -Expected 'core Recovery remains read-only and callable after workspace app cleanup' -Passed (
    $recovery.lifeline_available -eq $true -and $recovery.mutates_state -eq $false
) -Actual $recovery
