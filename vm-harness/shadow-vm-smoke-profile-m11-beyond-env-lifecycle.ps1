$method = "wasm.beyond_env_lifecycle_probe"
$LifelineVocabularySha256 = "sha256:7488a1abb0791a9e278d6883d6b45d993001148c7e0ffc03a50347399af3cc56"

$suspendOffset = Get-SerialLogOffset
Send-AgentCommand -Command "$method start_kill" -ExpectedMarker "RAIOS_AGENT_END $method" -Name "m11-beyond-env-lifecycle:start_kill"
$start = (Get-LastAgentResponseJson -Method $method).body.result
$suspendedMarker = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_BEYOND_ENV_LIFECYCLE suspended=true" -Offset $suspendOffset -TimeoutSeconds 2
Add-Predicate -Name "m11-beyond-env-lifecycle:real_resumable_marker" -Expected "the labeled fixture reaches ResumableCall::Resumable" -Passed $suspendedMarker -Actual $(if ($suspendedMarker) { "suspended" } else { Get-SerialLogTail -Path $SerialLog })
if (-not $suspendedMarker) { throw "Expected the labeled test.suspend_once fixture to retain a real wasmi continuation" }

Send-AgentCommand -Command "$method status" -ExpectedMarker "RAIOS_AGENT_END $method" -Name "m11-beyond-env-lifecycle:suspended_status"
$suspended = (Get-LastAgentResponseJson -Method $method).body.result
$suspendedOk = (
    $start.request_status -eq "accepted_pending" -and
    $suspended.schema -eq "raios.wasm_beyond_env_lifecycle_probe.v0" -and
    $suspended.test_infrastructure -eq $true -and
    $suspended.fixture_import -eq "test.suspend_once" -and
    $suspended.fixture_linker_scope -eq "labeled_fixture_runner_only" -and
    $suspended.active -eq $true -and $suspended.suspended -eq $true -and
    [int]$suspended.suspension_count -eq 1 -and [int]$suspended.resume_count -eq 0 -and
    $suspended.network_effect -eq $false -and $suspended.secret_effect -eq $false -and
    $suspended.acquisition_effect -eq $false -and $suspended.durable_effect -eq $false
)
Add-Predicate -Name "m11-beyond-env-lifecycle:suspended_grants_nothing" -Expected "real suspension is retained with zero network, secret, acquisition, or durable effect" -Passed $suspendedOk -Actual $(if ($suspendedOk) { "retained grants-nothing" } else { $suspended | ConvertTo-Json -Compress -Depth 8 })
if (-not $suspendedOk) { throw "Expected the retained fixture continuation to remain test-only and grants-nothing" }

Send-AgentCommand -Command "agent recovery.snapshot" -ExpectedMarker "RAIOS_AGENT_END recovery.snapshot" -Name "m11-beyond-env-lifecycle:recovery_snapshot_while_suspended"
$recoverySnapshot = (Get-LastAgentResponseJson -Method "recovery.snapshot").body.result
$recoverySnapshotOk = $recoverySnapshot.lifeline_available -eq $true -and $recoverySnapshot.routes_through_wasm -eq $false -and $recoverySnapshot.routes_through_provider -eq $false
Add-Predicate -Name "m11-beyond-env-lifecycle:recovery_snapshot_responsive" -Expected "recovery.snapshot answers outside Wasm while the continuation is retained" -Passed $recoverySnapshotOk -Actual $(if ($recoverySnapshotOk) { "responsive" } else { $recoverySnapshot | ConvertTo-Json -Compress -Depth 6 })
if (-not $recoverySnapshotOk) { throw "Expected recovery.snapshot to remain responsive during suspension" }

Send-AgentCommand -Command "agent recovery.lifeline_table" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_table" -Name "m11-beyond-env-lifecycle:lifeline_table_while_suspended"
$lifeline = (Get-LastAgentResponseJson -Method "recovery.lifeline_table").body.result
$lifelineOk = $lifeline.schema -eq "raios.recovery_lifeline_table.v0" -and [int]$lifeline.method_count -eq 6 -and $lifeline.routes_through_wasm -eq $false -and $lifeline.routes_through_provider -eq $false -and $lifeline.lifeline_vocabulary_sha256 -eq $LifelineVocabularySha256
Add-Predicate -Name "m11-beyond-env-lifecycle:lifeline_table_responsive" -Expected "the unchanged six-method recovery lifeline answers outside Wasm during suspension" -Passed $lifelineOk -Actual $(if ($lifelineOk) { "responsive unchanged" } else { $lifeline | ConvertTo-Json -Compress -Depth 6 })
if (-not $lifelineOk) { throw "Expected recovery.lifeline_table to remain unchanged and responsive" }

Send-AgentCommand -Command "$method start_finish" -ExpectedMarker "RAIOS_AGENT_END $method" -Name "m11-beyond-env-lifecycle:second_start_busy"
$busy = (Get-LastAgentResponseJson -Method $method).body.result
$busyOk = $busy.request_status -eq "resource_busy_active_invocation" -and $busy.active -eq $true -and [int]$busy.invocation_id -eq [int]$suspended.invocation_id
Add-Predicate -Name "m11-beyond-env-lifecycle:reentrancy_denied" -Expected "a second invocation start returns resource_busy_active_invocation without replacing the owner" -Passed $busyOk -Actual $(if ($busyOk) { "busy denied" } else { $busy | ConvertTo-Json -Compress -Depth 6 })
if (-not $busyOk) { throw "Expected a second fixture start to deny while one continuation is active" }

$killOffset = Get-SerialLogOffset
$killSentAt = [DateTime]::UtcNow
Send-QemuMonitorCommand -Command "sendkey f12 60" -ReplyWaitMilliseconds 0 | Out-Null
$monitorAcknowledgedAt = [DateTime]::UtcNow
$killNeedle = "RAIOS_BEYOND_ENV_LIFECYCLE outcome=killed teardown_complete=true"
$killFound = $false
$killDeadline = $killSentAt.AddMilliseconds(250)
do {
    $content = Get-SerialLogContent -Path $SerialLog
    if ($null -ne $content -and $killOffset -lt $content.Length -and $content.Substring([int]$killOffset) -clike "*$killNeedle*") {
        $killFound = $true
        break
    }
    Start-Sleep -Milliseconds 5
} while ([DateTime]::UtcNow -lt $killDeadline)
$killObservedAt = [DateTime]::UtcNow
$sendToMarkerMs = [int][Math]::Round(($killObservedAt - $killSentAt).TotalMilliseconds)
$ackToMarkerMs = [Math]::Max(0, [int][Math]::Round(($killObservedAt - $monitorAcknowledgedAt).TotalMilliseconds))
$physicalKillOk = $killFound -and $sendToMarkerMs -le 250 -and $ackToMarkerMs -le 250
Add-Predicate -Name "m11-beyond-env-lifecycle:physical_f12_monitor_path" -Expected "QEMU HMP sendkey f12 60, never serial, produces the killed marker" -Passed $killFound -Actual $(if ($killFound) { "monitor path" } else { Get-SerialLogTail -Path $SerialLog })
Add-Predicate -Name "m11-beyond-env-lifecycle:f12_host_bound" -Expected "F12 send/ack to killed marker is at most 250 ms host-side" -Passed $physicalKillOk -Actual "send_to_marker_ms=$sendToMarkerMs ack_to_marker_ms=$ackToMarkerMs"
if (-not $physicalKillOk) { throw "Expected physical F12 kill and teardown within 250 ms; send=$sendToMarkerMs ack=$ackToMarkerMs" }

Send-AgentCommand -Command "$method status" -ExpectedMarker "RAIOS_AGENT_END $method" -Name "m11-beyond-env-lifecycle:killed_status"
$killed = (Get-LastAgentResponseJson -Method $method).body.result
$guestKillDelta = [int64]$killed.terminal_boot_ms - [int64]$killed.kill_observed_boot_ms
$killedOk = (
    $killed.outcome -eq "killed" -and $killed.active -eq $false -and
    $killed.no_resume_after_kill -eq $true -and [int]$killed.resume_count -eq 0 -and
    $killed.teardown_complete -eq $true -and [int]$killed.teardown_count -eq 1 -and
    $killed.handles_invalid -eq $true -and $killed.singleton_lease_held -eq $false -and
    $killed.pending_acquisition_present -eq $false -and $killed.prior_candidate_unchanged -eq $true -and
    [int]$killed.killed_run_count -eq 1 -and $guestKillDelta -ge 0 -and $guestKillDelta -le 16
)
Add-Predicate -Name "m11-beyond-env-lifecycle:killed_cleanup_guest_bound" -Expected "no resume follows kill; exactly-once teardown completes within the 16 ms guest scheduling bound and releases every modeled resource" -Passed $killedOk -Actual $(if ($killedOk) { "guest_delta_ms=$guestKillDelta cleanup complete" } else { $killed | ConvertTo-Json -Compress -Depth 8 })
if (-not $killedOk) { throw "Expected killed outcome, no later resume, exactly-once teardown, and guest boot-relative bound" }

$finishOffset = Get-SerialLogOffset
Send-AgentCommand -Command "$method start_finish" -ExpectedMarker "RAIOS_AGENT_END $method" -Name "m11-beyond-env-lifecycle:second_run_start"
$finishedMarker = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_BEYOND_ENV_LIFECYCLE outcome=finished teardown_complete=true" -Offset $finishOffset -TimeoutSeconds 2
Add-Predicate -Name "m11-beyond-env-lifecycle:second_run_finished_marker" -Expected "a fresh invocation suspends, resumes once, and finishes after the killed owner released" -Passed $finishedMarker -Actual $(if ($finishedMarker) { "finished" } else { Get-SerialLogTail -Path $SerialLog })
if (-not $finishedMarker) { throw "Expected a second fixture run to finish after F12 teardown" }
Send-AgentCommand -Command "$method status" -ExpectedMarker "RAIOS_AGENT_END $method" -Name "m11-beyond-env-lifecycle:second_run_status"
$finished = (Get-LastAgentResponseJson -Method $method).body.result
$secondRunOk = $finished.outcome -eq "finished" -and [int]$finished.run_count -eq 2 -and [int]$finished.finished_run_count -eq 1 -and [int]$finished.killed_run_count -eq 1 -and [int]$finished.resume_count -eq 1 -and [int]$finished.teardown_count -eq 1 -and $finished.teardown_complete -eq $true
Add-Predicate -Name "m11-beyond-env-lifecycle:second_run_after_kill" -Expected "the second run resumes exactly once and tears down exactly once after the killed generation" -Passed $secondRunOk -Actual $(if ($secondRunOk) { "second run succeeded" } else { $finished | ConvertTo-Json -Compress -Depth 8 })
if (-not $secondRunOk) { throw "Expected task and handle generation release to permit a clean second run" }

$cases = @($finished.cases)
$expectedCases = [ordered]@{
    normal_return = "finished"
    guest_trap = "guest_trap"
    unrecognized_host_error = "host_error"
    host_fuel_error = "out_of_fuel"
    wasm_out_of_fuel = "out_of_fuel"
    suspend_marker_mismatch = "host_error"
    abandoned_drop_guard = "abandoned"
}
$caseMatrixOk = $cases.Count -eq $expectedCases.Count
foreach ($entry in $expectedCases.GetEnumerator()) {
    $row = @($cases | Where-Object { $_.name -eq $entry.Key })[0]
    $caseMatrixOk = $caseMatrixOk -and $null -ne $row -and $row.outcome -eq $entry.Value -and [int]$row.teardown_count -eq 1 -and $row.teardown_complete -eq $true
}
$caseMatrixOk = $caseMatrixOk -and $finished.tail_call_terminal -eq $true
Add-Predicate -Name "m11-beyond-env-lifecycle:terminal_matrix_exactly_once" -Expected "normal, guest trap, unrecognized host error, host fuel, Wasm fuel, marker mismatch, abandoned Drop, and tail-call edge are terminal with exactly-once cleanup" -Passed $caseMatrixOk -Actual $(if ($caseMatrixOk) { "all terminal routes exact" } else { $cases | ConvertTo-Json -Compress -Depth 6 })
if (-not $caseMatrixOk) { throw "Expected strict suspension classification and exactly-once teardown for every terminal fixture" }

$busyLoopOk = $finished.busy_loop_out_of_fuel -eq $true -and [int64]$finished.fuel_budget -eq 1000000 -and [int64]$finished.busy_loop_wall_ms -le 250
Add-Predicate -Name "m11-beyond-env-lifecycle:max_fuel_busy_loop_bound" -Expected "the 1,000,000-fuel busy loop terminates as OutOfFuel within 250 ms" -Passed $busyLoopOk -Actual "out_of_fuel=$($finished.busy_loop_out_of_fuel) wall_ms=$($finished.busy_loop_wall_ms)"
if (-not $busyLoopOk) { throw "Expected maximum-fuel busy loop to terminate within the cooperative 250 ms ceiling" }

$grantsNothingOk = $finished.policy_allows_beyond_env -eq $false -and $finished.production_linker_armed -eq $false -and $finished.network_effect -eq $false -and $finished.crypto_effect -eq $false -and $finished.secret_effect -eq $false -and $finished.acquisition_effect -eq $false -and $finished.durable_effect -eq $false -and $finished.capability_granted -eq $false -and $finished.evidence_complete -eq $true
Add-Predicate -Name "m11-beyond-env-lifecycle:grants_nothing" -Expected "policy stays false and no production linker, network, crypto, secret, acquisition, durable, or capability effect exists" -Passed $grantsNothingOk -Actual $(if ($grantsNothingOk) { "grants nothing" } else { $finished | ConvertTo-Json -Compress -Depth 8 })
if (-not $grantsNothingOk) { throw "Expected NET-2 lifecycle evidence to grant nothing" }

# Copy the m8-lifeline trap/fuel/recovery assertions into this focused profile.
Send-AgentCommand -Command "module.load_ephemeral svc.demo.echo" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "m11-beyond-env-lifecycle:m8_echo_load"
$m8Load = (Get-LastAgentResponseJson -Method "module.load_ephemeral").body.result
$m8LoadOk = $m8Load.schema -eq "raios.ram_only_echo_service.lifecycle_response.v0" -and $m8Load.service_id -eq "svc.demo.echo" -and $m8Load.loaded -eq $true
Add-Predicate -Name "m11-beyond-env-lifecycle:m8_echo_load" -Expected "svc.demo.echo loads as a current-boot Wasm service" -Passed $m8LoadOk -Actual $(if ($m8LoadOk) { "loaded" } else { $m8Load | ConvertTo-Json -Compress -Depth 6 })
if (-not $m8LoadOk) { throw "Expected the copied m8 echo load assertion to pass" }

Send-AgentCommand -Command "service.start svc.demo.echo" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "m11-beyond-env-lifecycle:m8_echo_start"
$m8Start = (Get-LastAgentResponseJson -Method "service.start").body.result
$m8StartOk = $m8Start.action -eq "start" -and $m8Start.loaded -eq $true -and $m8Start.running -eq $true -and $m8Start.health -eq "healthy"
Add-Predicate -Name "m11-beyond-env-lifecycle:m8_echo_start_healthy" -Expected "svc.demo.echo starts healthy before the copied fuel wedge" -Passed $m8StartOk -Actual $(if ($m8StartOk) { "healthy" } else { $m8Start | ConvertTo-Json -Compress -Depth 6 })
if (-not $m8StartOk) { throw "Expected copied m8 healthy start assertion" }

Send-AgentCommand -Command "agent echo.invoke_fuel_starved" -ExpectedMarker "RAIOS_AGENT_END echo.invoke_fuel_starved" -Name "m11-beyond-env-lifecycle:m8_fuel_wedge"
$m8Wedge = (Get-LastAgentResponseJson -Method "echo.invoke_fuel_starved").body.result
$m8WedgeOk = $m8Wedge.test_infrastructure -eq $true -and $m8Wedge.out_of_fuel -eq $true -and $m8Wedge.run_outcome -eq "fuel_exhausted" -and $m8Wedge.health -eq "crashed" -and $m8Wedge.crashed -eq $true -and $m8Wedge.running -eq $false
Add-Predicate -Name "m11-beyond-env-lifecycle:m8_wedge_out_of_fuel_crashed" -Expected "the real m8 echo fixture traps OutOfFuel and marks echo crashed without wedging the loop" -Passed $m8WedgeOk -Actual $(if ($m8WedgeOk) { "crashed_out_of_fuel" } else { $m8Wedge | ConvertTo-Json -Compress -Depth 6 })
if (-not $m8WedgeOk) { throw "Expected copied m8 OutOfFuel wedge assertion" }

Send-AgentCommand -Command "agent recovery.lifeline_table" -ExpectedMarker "RAIOS_AGENT_END recovery.lifeline_table" -Name "m11-beyond-env-lifecycle:m8_post_wedge_lifeline"
$m8Lifeline = (Get-LastAgentResponseJson -Method "recovery.lifeline_table").body.result
$m8LifelineOk = [int]$m8Lifeline.method_count -eq 6 -and $m8Lifeline.routes_through_wasm -eq $false -and $m8Lifeline.routes_through_provider -eq $false -and $m8Lifeline.lifeline_vocabulary_sha256 -eq $LifelineVocabularySha256
Add-Predicate -Name "m11-beyond-env-lifecycle:m8_post_wedge_lifeline_unchanged" -Expected "the unchanged six-method lifeline answers after real OutOfFuel" -Passed $m8LifelineOk -Actual $(if ($m8LifelineOk) { "unchanged" } else { $m8Lifeline | ConvertTo-Json -Compress -Depth 6 })
if (-not $m8LifelineOk) { throw "Expected copied m8 post-wedge lifeline assertion" }

Send-AgentCommand -Command "agent recovery.snapshot" -ExpectedMarker "RAIOS_AGENT_END recovery.snapshot" -Name "m11-beyond-env-lifecycle:m8_post_wedge_snapshot"
$m8Snapshot = (Get-LastAgentResponseJson -Method "recovery.snapshot").body.result
$m8EchoCrash = @(@($m8Snapshot.crashed_services) | Where-Object { $_.id -eq "svc.demo.echo" })[0]
$m8SnapshotOk = $m8Snapshot.lifeline_available -eq $true -and $m8Snapshot.routes_through_wasm -eq $false -and $m8Snapshot.routes_through_provider -eq $false -and $null -ne $m8EchoCrash -and $m8EchoCrash.health -eq "crashed" -and ([string]$m8EchoCrash.last_error_id -like "event.current_boot.*")
Add-Predicate -Name "m11-beyond-env-lifecycle:m8_snapshot_lists_crashed_echo" -Expected "recovery.snapshot lists the fuel-wedged echo with its stable current-boot error id" -Passed $m8SnapshotOk -Actual $(if ($m8SnapshotOk) { "listed crashed" } else { $m8Snapshot | ConvertTo-Json -Compress -Depth 6 })
if (-not $m8SnapshotOk) { throw "Expected copied m8 recovery.snapshot assertion" }
