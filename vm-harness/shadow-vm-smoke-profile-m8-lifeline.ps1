# M8A-3 focused profile: recovery lifeline survives a REAL wedged Wasm service.
#
# Proves the fault-injection story end to end: load+start echo, wedge it with a
# genuine wasmi OutOfFuel trap (echo.invoke_fuel_starved), then show the pinned
# lifeline still answers while echo is crashed — the lifeline_table vocabulary
# hash is UNCHANGED, recovery.snapshot lists svc.demo.echo under crashed_services,
# and all four recovery mutators still deny. HARD-STOP detection is baked in: the
# wedge's OWN end-marker and both post-wedge lifeline end-markers are asserted with
# bounded timeouts (a missing marker means the wedge took down the dispatcher loop,
# which changes M8's shape — stop and report).
#
# This profile is standalone and dot-sourced AFTER shadow-vm-smoke-profile-common.ps1,
# so baseline agent boot needles already ran. It touches only echo + the lifeline;
# it grants nothing.

$LifelineVocabularySha256 = "sha256:03d3985c104de4d025c7b38aa6d484bbe68e0eb3512bc034993663c7b7a112a3"

# --- (a) baseline: echo loads, starts, and is healthy; snapshot is clean -----------
Send-AgentCommand -Command "module.load_ephemeral svc.demo.echo" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "m8-lifeline:echo_load"
$m8EchoLoad = Get-LastAgentResponseJson -Method "module.load_ephemeral"
$m8EchoLoadResult = $m8EchoLoad.body.result
$m8EchoLoadOk = (
    $m8EchoLoad.t -eq "response" -and
    $m8EchoLoadResult.schema -eq "raios.ram_only_echo_service.lifecycle_response.v0" -and
    $m8EchoLoadResult.service_id -eq "svc.demo.echo" -and
    $m8EchoLoadResult.loaded -eq $true
)
Add-Predicate -Name "m8-lifeline:echo_load" -Expected "svc.demo.echo loads as a current-boot wasm service" -Passed $m8EchoLoadOk -Actual $(if ($m8EchoLoadOk) { "loaded" } else { ($m8EchoLoadResult | ConvertTo-Json -Compress -Depth 6) })
if (-not $m8EchoLoadOk) {
    throw "Expected module.load_ephemeral svc.demo.echo to load the current-boot wasm service"
}

Send-AgentCommand -Command "service.start svc.demo.echo" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "m8-lifeline:echo_start"
$m8EchoStart = Get-LastAgentResponseJson -Method "service.start"
$m8EchoStartResult = $m8EchoStart.body.result
$m8EchoStartOk = (
    $m8EchoStartResult.action -eq "start" -and
    $m8EchoStartResult.loaded -eq $true -and
    $m8EchoStartResult.running -eq $true -and
    $m8EchoStartResult.health -eq "healthy"
)
Add-Predicate -Name "m8-lifeline:echo_start_healthy" -Expected "svc.demo.echo starts, running=true, health=healthy" -Passed $m8EchoStartOk -Actual $(if ($m8EchoStartOk) { "healthy" } else { ($m8EchoStartResult | ConvertTo-Json -Compress -Depth 6) })
if (-not $m8EchoStartOk) {
    throw "Expected service.start svc.demo.echo to run and report healthy"
}

Send-AgentCommand -Command "agent recovery.snapshot" -ExpectedMarker "RAIOS_AGENT_END recovery.snapshot" -Name "m8-lifeline:baseline_snapshot"
$m8BaselineSnapshot = Get-LastAgentResponseJson -Method "recovery.snapshot"
$m8bs = $m8BaselineSnapshot.body.result
$m8BaselineCrashed = @($m8bs.crashed_services)
$m8BaselineSnapshotOk = (
    $m8bs.schema -eq "raios.recovery_snapshot.v0" -and
    $m8bs.scope -eq "current_boot" -and
    $m8bs.lifeline_available -eq $true -and
    $m8bs.routes_through_wasm -eq $false -and
    [int]$m8bs.crashed_service_count -eq 0 -and
    $m8BaselineCrashed.Count -eq 0
)
Add-Predicate -Name "m8-lifeline:baseline_snapshot_no_crash" -Expected "baseline recovery.snapshot renders with empty crashed_services" -Passed $m8BaselineSnapshotOk -Actual $(if ($m8BaselineSnapshotOk) { "clean" } else { ($m8bs | ConvertTo-Json -Compress -Depth 6) })
if (-not $m8BaselineSnapshotOk) {
    throw "Expected baseline recovery.snapshot to render with no crashed services"
}

# --- (b) REAL wedge: fuel-starved echo invoke traps OutOfFuel and flips crashed ----
# The ExpectedMarker assertion below is the HARD-STOP tripwire for the wedge itself:
# if RAIOS_AGENT_END echo.invoke_fuel_starved never appears, the trap took down the
# cooperative loop — Send-AgentCommand throws and the profile fails loudly.
Send-AgentCommand -Command "agent echo.invoke_fuel_starved" -ExpectedMarker "RAIOS_AGENT_END echo.invoke_fuel_starved" -Name "m8-lifeline:wedge_marker"
$m8Wedge = Get-LastAgentResponseJson -Method "echo.invoke_fuel_starved"
$m8WedgeResult = $m8Wedge.body.result
$m8WedgeEventId = [string]$m8WedgeResult.last_error_id
$m8WedgeOk = (
    $m8Wedge.t -eq "response" -and
    $m8WedgeResult.schema -eq "raios.ram_only_echo_service.fuel_starved_response.v0" -and
    $m8WedgeResult.service_id -eq "svc.demo.echo" -and
    $m8WedgeResult.test_infrastructure -eq $true -and
    $m8WedgeResult.scope -eq "current_boot" -and
    $m8WedgeResult.out_of_fuel -eq $true -and
    $m8WedgeResult.run_outcome -eq "fuel_exhausted" -and
    $m8WedgeResult.health -eq "crashed" -and
    $m8WedgeResult.crashed -eq $true -and
    $m8WedgeResult.running -eq $false -and
    ($m8WedgeEventId -like "event.current_boot.*")
)
Add-Predicate -Name "m8-lifeline:wedge_out_of_fuel_crashed" -Expected "echo.invoke_fuel_starved traps OutOfFuel and marks echo crashed (test_infrastructure, current_boot)" -Passed $m8WedgeOk -Actual $(if ($m8WedgeOk) { "crashed_out_of_fuel event=$m8WedgeEventId" } else { ($m8WedgeResult | ConvertTo-Json -Compress -Depth 6) })
if (-not $m8WedgeOk) {
    throw "Expected echo.invoke_fuel_starved to trap OutOfFuel and mark echo crashed"
}

Send-AgentCommand -Command "agent audit.events 12" -ExpectedMarker "RAIOS_AGENT_END memory.recent_events" -Name "m8-lifeline:wedge_audit_events"
Assert-LogContains -Name "m8-lifeline:wedge_audit_reason" -Needle '"reason": "wedged_out_of_fuel_test_infrastructure"' -TimeoutSeconds 2
Assert-LogContains -Name "m8-lifeline:wedge_audit_resource" -Needle '"resource": "svc.demo.echo"' -TimeoutSeconds 2

# --- (c) LIFELINE STILL ANSWERS post-wedge -----------------------------------------
# Bounded end-marker assertion = HARD-STOP tripwire for the lifeline path.
Send-AgentCommand -Command "agent recovery.lifeline_table" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_table" -Name "m8-lifeline:post_wedge_lifeline_table"
$m8LifelineTable = Get-LastAgentResponseJson -Method "recovery.lifeline_table"
$m8lt = $m8LifelineTable.body.result
$m8LifelineTableOk = (
    $m8lt.schema -eq "raios.recovery_lifeline_table.v0" -and
    [int]$m8lt.method_count -eq 6 -and
    $m8lt.routes_through_wasm -eq $false -and
    $m8lt.lifeline_vocabulary_sha256 -eq $LifelineVocabularySha256
)
Add-Predicate -Name "m8-lifeline:post_wedge_lifeline_table_unchanged" -Expected "recovery.lifeline_table still renders 6 methods with the UNCHANGED pinned vocabulary hash while echo is wedged" -Passed $m8LifelineTableOk -Actual $(if ($m8LifelineTableOk) { "unchanged" } else { ($m8lt | ConvertTo-Json -Compress -Depth 6) })
if (-not $m8LifelineTableOk) {
    throw "Expected recovery.lifeline_table to answer with the unchanged pinned vocabulary while echo is wedged"
}

$m8SnapshotOffset = Get-SerialLogOffset
Send-AgentCommand -Command "agent recovery.snapshot" -ExpectedMarker "RAIOS_AGENT_END recovery.snapshot" -Name "m8-lifeline:post_wedge_snapshot"
$m8PostSnapshot = Get-LastAgentResponseJson -Method "recovery.snapshot"
$m8ps = $m8PostSnapshot.body.result
$m8PostCrashed = @($m8ps.crashed_services)
$m8EchoCrashRow = @($m8PostCrashed | Where-Object { $_.id -eq "svc.demo.echo" })[0]
$m8PostSnapshotOk = (
    $m8ps.schema -eq "raios.recovery_snapshot.v0" -and
    $m8ps.lifeline_available -eq $true -and
    $m8ps.routes_through_wasm -eq $false -and
    [int]$m8ps.crashed_service_count -ge 1 -and
    ($null -ne $m8EchoCrashRow) -and
    $m8EchoCrashRow.health -eq "crashed" -and
    ([string]$m8EchoCrashRow.last_error_id -eq $m8WedgeEventId) -and
    ([string]$m8EchoCrashRow.last_error_id -like "event.current_boot.*")
)
Add-Predicate -Name "m8-lifeline:post_wedge_snapshot_lists_crashed_echo" -Expected "recovery.snapshot lists svc.demo.echo under crashed_services with health=crashed and the stable last_error_id" -Passed $m8PostSnapshotOk -Actual $(if ($m8PostSnapshotOk) { "listed last_error_id=$([string]$m8EchoCrashRow.last_error_id)" } else { ($m8ps | ConvertTo-Json -Compress -Depth 6) })
if (-not $m8PostSnapshotOk) {
    throw "Expected recovery.snapshot to list svc.demo.echo under crashed_services while it is wedged"
}

# --- (d) mutators still deny while wedged -------------------------------------------
# recovery.disable_module is NO LONGER in this loop: it is an implemented executor as of
# M8B-1 and is exercised (positive + fail-closed) in section (g) below. The OTHER three
# mutators must still fail closed (needle vi).
foreach ($m8Mutator in @("recovery.restart_last_good", "recovery.rollback", "recovery.load_artifact_by_hash")) {
    Send-AgentCommand -Command "agent $m8Mutator" -ExpectedMarker "RAIOS_AGENT_END $m8Mutator" -Name "m8-lifeline:mutator_$m8Mutator"
    $m8MutatorResponse = Get-LastAgentResponseJson -Method $m8Mutator
    $m8mr = $m8MutatorResponse.body.result
    $m8MutatorDenied = (
        $m8mr.status -eq "capability_denied" -and
        $m8mr.method -eq $m8Mutator -and
        $m8mr.implemented -eq $false -and
        $m8mr.mutates_state -eq $false
    )
    Add-Predicate -Name "m8-lifeline:mutator_denied_$m8Mutator" -Expected "$m8Mutator stays capability_denied (implemented=false, mutates_state=false) while echo is wedged" -Passed $m8MutatorDenied -Actual $(if ($m8MutatorDenied) { "denied" } else { ($m8mr | ConvertTo-Json -Compress -Depth 6) })
    if (-not $m8MutatorDenied) {
        throw "Expected $m8Mutator to return typed capability_denied while echo is wedged"
    }
}

# --- (e) redaction negative: no secret text in the post-wedge snapshot window -------
$m8SnapshotWindow = (Get-SerialLogContent -Path $SerialLog)
if ($null -eq $m8SnapshotWindow) { $m8SnapshotWindow = "" }
if ($m8SnapshotWindow.Length -gt $m8SnapshotOffset) {
    $m8SnapshotWindow = $m8SnapshotWindow.Substring($m8SnapshotOffset)
}
$m8RedactionOk = $true
foreach ($m8Secret in @("sk-", "OPENAI_API_KEY", "api_key", "passphrase", "wifi_password", "PSK:")) {
    if ($m8SnapshotWindow.Contains($m8Secret)) { $m8RedactionOk = $false }
}
Add-Predicate -Name "m8-lifeline:snapshot_redaction_no_secrets" -Expected "recovery.snapshot output window contains no api-key/passphrase text" -Passed $m8RedactionOk -Actual $(if ($m8RedactionOk) { "redacted" } else { "secret_marker_present" })
if (-not $m8RedactionOk) {
    throw "Expected recovery.snapshot output to contain no secret markers"
}

# --- (f) restart recovery: a successful restart clears the crashed latch (ADR 0004 honesty) -------
# The obvious operator recovery action (restart the crashed service) must make the crashed
# report go away; a healthy, running, just-executed service must NEVER be reported crashed.
Send-AgentCommand -Command "service.start svc.demo.echo" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "m8-lifeline:restart"
$m8Restart = Get-LastAgentResponseJson -Method "service.start"
$m8RestartResult = $m8Restart.body.result
$m8RestartOk = (
    $m8RestartResult.action -eq "start" -and
    $m8RestartResult.loaded -eq $true -and
    $m8RestartResult.running -eq $true -and
    $m8RestartResult.health -eq "healthy"
)
Add-Predicate -Name "m8-lifeline:restart_recovers_healthy" -Expected "restarting the wedged echo runs it healthy again (crashed latch cleared)" -Passed $m8RestartOk -Actual $(if ($m8RestartOk) { "recovered_healthy" } else { ($m8RestartResult | ConvertTo-Json -Compress -Depth 6) })
if (-not $m8RestartOk) {
    throw "Expected restarting svc.demo.echo to recover to healthy/running"
}

Send-AgentCommand -Command "agent recovery.snapshot" -ExpectedMarker "RAIOS_AGENT_END recovery.snapshot" -Name "m8-lifeline:post_restart_snapshot"
$m8PostRestart = Get-LastAgentResponseJson -Method "recovery.snapshot"
$m8prs = $m8PostRestart.body.result
$m8PostRestartClean = (
    $m8prs.schema -eq "raios.recovery_snapshot.v0" -and
    [int]$m8prs.crashed_service_count -eq 0 -and
    @($m8prs.crashed_services).Count -eq 0
)
Add-Predicate -Name "m8-lifeline:post_restart_snapshot_no_false_crash" -Expected "after a successful restart, recovery.snapshot no longer lists echo as crashed (no false crashed report)" -Passed $m8PostRestartClean -Actual $(if ($m8PostRestartClean) { "cleared" } else { ($m8prs | ConvertTo-Json -Compress -Depth 6) })
if (-not $m8PostRestartClean) {
    throw "Expected recovery.snapshot to report NO crashed services after a successful restart"
}

# --- (g) M8B-1 recovery.disable_module executor: fail-closed denials (mutate nothing) [needles ii-iv] ---
# Every invalid target must fail closed in the deny-before-mutate PREFLIGHT: core-owned,
# the lifeline endpoint itself, an unknown id, and "disable all". None writes a durable
# record, none mutates live state. Bounded end-marker on every call = hard-stop tripwire.
$m8DisableDenials = @(
    @{ Target = "core.serial"; Reason = "target_core_owned"; Label = "core_owned_core_serial" },
    @{ Target = "recovery.disable_module"; Reason = "target_is_lifeline_endpoint"; Label = "lifeline_endpoint_self" },
    @{ Target = "svc.nope"; Reason = "target_unknown"; Label = "unknown_id_svc_nope" },
    @{ Target = "*"; Reason = "target_unknown"; Label = "disable_all_star" }
)
foreach ($m8Deny in $m8DisableDenials) {
    Send-AgentCommand -Command "agent recovery.disable_module $($m8Deny.Target)" -ExpectedMarker "RAIOS_AGENT_END recovery.disable_module" -Name "m8-lifeline:disable_deny_$($m8Deny.Label)"
    $m8DenyResp = Get-LastAgentResponseJson -Method "recovery.disable_module"
    $m8dr = $m8DenyResp.body.result
    $m8DenyOk = (
        $m8dr.schema -eq "raios.recovery_action.v0" -and
        $m8dr.action_kind -eq "disable_module" -and
        $m8dr.method -eq "recovery.disable_module" -and
        $m8dr.status -eq "capability_denied" -and
        $m8dr.reason -eq $m8Deny.Reason -and
        $m8dr.performed -eq $false -and
        $m8dr.mutates_live_state -eq $false -and
        $m8dr.durable_append -ne "appended" -and
        $m8dr.owner_sealed -eq $false -and
        $m8dr.grants_new_capability -eq $false -and
        $m8dr.persistence_claimed -eq $false -and
        $m8dr.promotion_authority_is_placeholder -eq $true
    )
    Add-Predicate -Name "m8-lifeline:disable_denied_$($m8Deny.Label)" -Expected "recovery.disable_module $($m8Deny.Target) fails closed ($($m8Deny.Reason)) and mutates nothing" -Passed $m8DenyOk -Actual $(if ($m8DenyOk) { "denied $($m8Deny.Reason)" } else { ($m8dr | ConvertTo-Json -Compress -Depth 6) })
    if (-not $m8DenyOk) {
        throw "Expected recovery.disable_module $($m8Deny.Target) to fail closed with $($m8Deny.Reason) and mutate nothing"
    }
}

# --- (g2) SAFE-mode + full truth table via the recovery_action selftest [needle v] ----
# The m8-lifeline VM boots Normal, so SAFE is proven here (not by forcing SAFE): the
# selftest's safe_posture_denied case plus one preflight case per classification denial.
Send-AgentCommand -Command "agent recovery.disable_module_selftest" -ExpectedMarker "RAIOS_AGENT_END recovery.disable_module_selftest" -Name "m8-lifeline:disable_selftest"
$m8Selftest = Get-LastAgentResponseJson -Method "recovery.disable_module_selftest"
$m8st = $m8Selftest.body.result
$m8SelftestCases = @($m8st.cases)
$m8SafeCase = @($m8SelftestCases | Where-Object { $_.case -eq "safe_posture_denied" })[0]
$m8SelftestOk = (
    $m8st.schema -eq "raios.recovery_action_append_selftest.v0" -and
    $m8st.passed -eq $true -and
    [int]$m8st.case_count -ge 12 -and
    ($null -ne $m8SafeCase) -and
    $m8SafeCase.actual_reason -eq "boot_control_safe_mode" -and
    $m8SafeCase.passed -eq $true -and
    (@($m8SelftestCases | Where-Object { $_.case -eq "preflight_core_owned_denied" }).Count -eq 1) -and
    (@($m8SelftestCases | Where-Object { $_.case -eq "preflight_lifeline_endpoint_denied" }).Count -eq 1) -and
    (@($m8SelftestCases | Where-Object { $_.case -eq "preflight_target_unknown_denied" }).Count -eq 1)
)
Add-Predicate -Name "m8-lifeline:disable_selftest_truth_table" -Expected "recovery.disable_module_selftest passes the full truth table incl. safe_posture_denied + one preflight case per classification denial" -Passed $m8SelftestOk -Actual $(if ($m8SelftestOk) { "passed case_count=$([int]$m8st.case_count)" } else { ($m8st | ConvertTo-Json -Compress -Depth 6) })
if (-not $m8SelftestOk) {
    throw "Expected recovery.disable_module_selftest to pass the full truth table including safe_posture_denied"
}

# --- (g3) lifeline_table: disable_module now implemented+mutating, vocab re-pinned [needle vii] ---
Send-AgentCommand -Command "agent recovery.lifeline_table" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_table" -Name "m8-lifeline:disable_lifeline_table"
$m8Table2 = Get-LastAgentResponseJson -Method "recovery.lifeline_table"
$m8t2 = $m8Table2.body.result
$m8DisableRow = @($m8t2.methods | Where-Object { $_.name -eq "recovery.disable_module" })[0]
$m8Table2Ok = (
    $m8t2.schema -eq "raios.recovery_lifeline_table.v0" -and
    [int]$m8t2.method_count -eq 6 -and
    $m8t2.lifeline_vocabulary_sha256 -eq $LifelineVocabularySha256 -and
    ($null -ne $m8DisableRow) -and
    $m8DisableRow.implemented -eq $true -and
    $m8DisableRow.mutating -eq $true
)
Add-Predicate -Name "m8-lifeline:disable_module_row_implemented" -Expected "recovery.lifeline_table shows method_count 6, disable_module implemented+mutating, and the re-pinned vocabulary hash" -Passed $m8Table2Ok -Actual $(if ($m8Table2Ok) { "implemented+mutating vocab=$($m8t2.lifeline_vocabulary_sha256)" } else { ($m8t2 | ConvertTo-Json -Compress -Depth 6) })
if (-not $m8Table2Ok) {
    throw "Expected recovery.lifeline_table to show disable_module implemented+mutating with the re-pinned vocabulary"
}

# --- (g4) POSITIVE: disable svc.demo.echo for the current boot [needle i] -------------
# Durable recovery_action record FIRST (appended+readback+reparse-verified), THEN echo
# is stopped+disabled. The append is the mutating tripwire: a missing/denied append here
# means the disable never ran.
$m8DisableOffset = Get-SerialLogOffset
Send-AgentCommand -Command "agent recovery.disable_module svc.demo.echo" -ExpectedMarker "RAIOS_AGENT_END recovery.disable_module" -Name "m8-lifeline:disable_echo"
$m8Disable = Get-LastAgentResponseJson -Method "recovery.disable_module"
$m8d = $m8Disable.body.result
$m8DisableOk = (
    $m8d.schema -eq "raios.recovery_action.v0" -and
    $m8d.action_kind -eq "disable_module" -and
    $m8d.disable_target_id -eq "svc.demo.echo" -and
    $m8d.disable_target_known_disablable -eq $true -and
    $m8d.status -eq "ok" -and
    $m8d.durable_append -eq "appended" -and
    $m8d.performed -eq $true -and
    $m8d.reason -eq "authorized_recovery_action_append_readback_reparse_verified" -and
    $m8d.reparse_valid -eq $true -and
    ([int]$m8d.count_after -eq ([int]$m8d.count_before + 1)) -and
    $m8d.mutates_live_state -eq $true -and
    $m8d.owner_sealed -eq $false -and
    $m8d.persistence_claimed -eq $false -and
    $m8d.grants_new_capability -eq $false -and
    $m8d.promotion_authority_is_placeholder -eq $true -and
    ([string]$m8d.disable_event_id -like "event.current_boot.*")
)
Add-Predicate -Name "m8-lifeline:disable_echo_appended_and_disabled" -Expected "recovery.disable_module svc.demo.echo appends a durable recovery_action record (reparse-verified, count+1) and disables echo for the current boot" -Passed $m8DisableOk -Actual $(if ($m8DisableOk) { "appended+disabled count $([int]$m8d.count_before)->$([int]$m8d.count_after)" } else { ($m8d | ConvertTo-Json -Compress -Depth 6) })
if (-not $m8DisableOk) {
    throw "Expected recovery.disable_module svc.demo.echo to durably append then disable echo for the current boot"
}

Send-AgentCommand -Command "agent recovery.snapshot" -ExpectedMarker "RAIOS_AGENT_END recovery.snapshot" -Name "m8-lifeline:post_disable_snapshot"
$m8DisSnap = Get-LastAgentResponseJson -Method "recovery.snapshot"
$m8dsnap = $m8DisSnap.body.result
$m8DisabledList = @($m8dsnap.disabled_modules)
$m8EchoDisRow = @($m8DisabledList | Where-Object { $_.id -eq "svc.demo.echo" })[0]
$m8DisSnapOk = (
    $m8dsnap.schema -eq "raios.recovery_snapshot.v0" -and
    $m8dsnap.routes_through_wasm -eq $false -and
    [int]$m8dsnap.disabled_module_count -ge 1 -and
    ($null -ne $m8EchoDisRow) -and
    $m8EchoDisRow.health -eq "disabled"
)
Add-Predicate -Name "m8-lifeline:post_disable_snapshot_lists_disabled_echo" -Expected "recovery.snapshot lists svc.demo.echo under disabled_modules with health=disabled" -Passed $m8DisSnapOk -Actual $(if ($m8DisSnapOk) { "listed disabled" } else { ($m8dsnap | ConvertTo-Json -Compress -Depth 6) })
if (-not $m8DisSnapOk) {
    throw "Expected recovery.snapshot to list svc.demo.echo under disabled_modules with health=disabled"
}

Send-AgentCommand -Command "service.start svc.demo.echo" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "m8-lifeline:disabled_start_denied"
$m8DisStart = Get-LastAgentResponseJson -Method "service.start"
$m8dst = $m8DisStart.body.result
$m8DisStartOk = (
    $m8dst.action -eq "start" -and
    $m8dst.running -eq $false -and
    $m8dst.reason -eq "service_disabled" -and
    $m8dst.health -eq "disabled"
)
Add-Predicate -Name "m8-lifeline:disabled_start_refused" -Expected "service.start svc.demo.echo is refused while disabled (reason service_disabled, running=false, health=disabled)" -Passed $m8DisStartOk -Actual $(if ($m8DisStartOk) { "refused service_disabled" } else { ($m8dst | ConvertTo-Json -Compress -Depth 6) })
if (-not $m8DisStartOk) {
    throw "Expected service.start svc.demo.echo to be refused with reason service_disabled while disabled"
}

# --- (g5) the OTHER three mutators still deny after a real disable happened [needle vi] ---
foreach ($m8StillDenied in @("recovery.restart_last_good", "recovery.rollback", "recovery.load_artifact_by_hash")) {
    Send-AgentCommand -Command "agent $m8StillDenied" -ExpectedMarker "RAIOS_AGENT_END $m8StillDenied" -Name "m8-lifeline:post_disable_mutator_$m8StillDenied"
    $m8SdResp = Get-LastAgentResponseJson -Method $m8StillDenied
    $m8sd = $m8SdResp.body.result
    $m8SdDenied = (
        $m8sd.status -eq "capability_denied" -and
        $m8sd.method -eq $m8StillDenied -and
        $m8sd.implemented -eq $false -and
        $m8sd.mutates_state -eq $false
    )
    Add-Predicate -Name "m8-lifeline:post_disable_mutator_denied_$m8StillDenied" -Expected "$m8StillDenied stays capability_denied (implemented=false) even after a real disable happened" -Passed $m8SdDenied -Actual $(if ($m8SdDenied) { "denied" } else { ($m8sd | ConvertTo-Json -Compress -Depth 6) })
    if (-not $m8SdDenied) {
        throw "Expected $m8StillDenied to stay capability_denied after a real disable"
    }
}

# --- (g6) redaction negative over the whole disable window [needle viii] ---------------
$m8DisableWindow = (Get-SerialLogContent -Path $SerialLog)
if ($null -eq $m8DisableWindow) { $m8DisableWindow = "" }
if ($m8DisableWindow.Length -gt $m8DisableOffset) {
    $m8DisableWindow = $m8DisableWindow.Substring($m8DisableOffset)
}
$m8DisableRedactionOk = $true
foreach ($m8Secret in @("sk-", "OPENAI_API_KEY", "api_key", "passphrase", "wifi_password", "PSK:")) {
    if ($m8DisableWindow.Contains($m8Secret)) { $m8DisableRedactionOk = $false }
}
Add-Predicate -Name "m8-lifeline:disable_window_redaction_no_secrets" -Expected "the recovery.disable_module output window contains no api-key/passphrase text" -Passed $m8DisableRedactionOk -Actual $(if ($m8DisableRedactionOk) { "redacted" } else { "secret_marker_present" })
if (-not $m8DisableRedactionOk) {
    throw "Expected the recovery.disable_module output window to contain no secret markers"
}
