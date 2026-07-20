if (-not $Network) {
    throw "m11-net-imports requires -Network (e1000 + DHCP + real TCP step)"
}

$method = "network.transport_lease_probe"
Send-AgentCommand -Command $method -ExpectedMarker "RAIOS_AGENT_END $method" -Name "m11-net-imports:transport_lease_probe"
$probe = (Get-LastAgentResponseJson -Method $method).body.result

$nativeSuccess = $probe.schema -eq "raios.transport_lease_probe.v0" -and
    $probe.test_infrastructure -eq $true -and
    $probe.network_configured -eq $true -and
    $probe.native_owner -eq "svc.provider.openai_direct" -and
    $probe.native_tcp_action -eq "started" -and
    [int]$probe.native_tcp_poll_steps -eq 1 -and
    $probe.native_tcp_action_bounded_one_step -eq $true
Add-Predicate -Name "m11-net-imports:native_owner_tcp_step" -Expected "DHCP-configured native OpenAI owner claims the lease and starts one bounded real TCP action" -Passed $nativeSuccess -Actual $(if ($nativeSuccess) { "started" } else { $probe | ConvertTo-Json -Compress -Depth 6 })
if (-not $nativeSuccess) { throw "Expected the native owner to complete one bounded TCP start step" }

$testBlocksNative = $probe.test_blocks_native_reason -eq "network_transport_busy"
Add-Predicate -Name "m11-net-imports:test_blocks_native" -Expected "the test owner makes the native claimant receive network_transport_busy" -Passed $testBlocksNative -Actual $probe.test_blocks_native_reason
if (-not $testBlocksNative) { throw "Expected test-to-native busy denial" }

$nativeBlocksTest = $probe.native_blocks_test_reason -eq "network_transport_busy"
Add-Predicate -Name "m11-net-imports:native_blocks_test" -Expected "the native owner makes the test claimant receive network_transport_busy" -Passed $nativeBlocksTest -Actual $probe.native_blocks_test_reason
if (-not $nativeBlocksTest) { throw "Expected native-to-test busy denial" }

$foreignAbortDenied = $probe.foreign_abort_reason -eq "transport_lease_foreign_owner" -and
    $probe.active_owner_survived_foreign_abort -eq $true
Add-Predicate -Name "m11-net-imports:foreign_abort_denied" -Expected "a foreign generation cannot abort or disturb the active owner's socket" -Passed $foreignAbortDenied -Actual $(if ($foreignAbortDenied) { "foreign denied; owner intact" } else { $probe | ConvertTo-Json -Compress -Depth 6 })
if (-not $foreignAbortDenied) { throw "Expected owner-only abort with no cross-talk" }

$ownerAbort = $probe.owner_abort_released -eq $true
Add-Predicate -Name "m11-net-imports:owner_abort_releases" -Expected "the current owner aborts and releases its own lease" -Passed $ownerAbort -Actual $probe.owner_abort_released
if (-not $ownerAbort) { throw "Expected owner abort to release" }

$timeoutRelease = $probe.timeout_released -eq $true
Add-Predicate -Name "m11-net-imports:timeout_releases" -Expected "deadline expiry releases the singleton lease" -Passed $timeoutRelease -Actual $probe.timeout_released
if (-not $timeoutRelease) { throw "Expected timeout release" }

$retryGrantsNothing = $probe.retry_succeeded -eq $true -and
    $probe.generation_advanced -eq $true -and
    $probe.idempotent_owner_teardown -eq $true -and
    $probe.policy_allows_beyond_env -eq $false -and
    $probe.wasm_net_import_linked -eq $false -and
    $probe.production_linker_armed -eq $false -and
    $probe.capability_granted -eq $false -and
    $probe.durable_effect -eq $false -and
    $probe.evidence_complete -eq $true
Add-Predicate -Name "m11-net-imports:retry_generation_grants_nothing" -Expected "retry gets a fresh generation; teardown is idempotent; no Wasm net linker or beyond-env authority exists" -Passed $retryGrantsNothing -Actual $(if ($retryGrantsNothing) { "fresh retry; grants nothing" } else { $probe | ConvertTo-Json -Compress -Depth 6 })
if (-not $retryGrantsNothing) { throw "Expected fresh-generation retry with no authority grant" }

# NET-4 uses only QEMU user-net endpoints: slirp's built-in DNS server at
# 10.0.2.3:53 is the responsive TCP peer; unused same-subnet 10.0.2.254:53
# stays in connect/ARP progress and is deterministic without external network.
$netMethod = "network.transport_lease_probe"
Send-AgentCommand -Command "$netMethod status" -ExpectedMarker "RAIOS_AGENT_END $netMethod" -Name "m11-net-imports:net4_grant_negatives"
$netStatus = (Get-LastAgentResponseJson -Method $netMethod).body.result
$netDenied = $netStatus.schema -eq "raios.transport_lease_probe.v0" -and
    $netStatus.denied_service_id -eq "test.fixture.net_shims.signed" -and
    $netStatus.host_import_abi -eq "raios.host_imports.v1" -and
    $netStatus.denied_artifact_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    $netStatus.denied_import_list_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    [int]$netStatus.requested_net_import_count -eq 4 -and
    $netStatus.net_policy_denial_reason -eq "import_beyond_env_not_owner_authorized" -and
    $netStatus.net_exact_list_drift_reason -eq "observed_import_list_mismatch" -and
    $netStatus.net_linker_drift_reason -eq "linker_implementation_list_mismatch" -and
    $netStatus.net_denied_before_instantiation -eq $true -and
    $netStatus.policy_allows_beyond_env -eq $false -and
    $netStatus.production_linker_armed -eq $false -and
    [int]$netStatus.production_net_shim_call_count -eq 0 -and
    $netStatus.capability_granted -eq $false
Add-Predicate -Name "m11-net-imports:net4_denied_before_instantiation" -Expected "all four evidence-bound net.* requests deny before instantiation; exact-list and linker drift remain fail-closed" -Passed $netDenied -Actual $(if ($netDenied) { "policy/list/linker denied" } else { $netStatus | ConvertTo-Json -Compress -Depth 8 })
if (-not $netDenied) { throw "Expected NET-1 negatives and the production net.* policy denial before instantiation" }

$responsiveOffset = Get-SerialLogOffset
Send-AgentCommand -Command "$netMethod start_responsive" -ExpectedMarker "RAIOS_AGENT_END $netMethod" -Name "m11-net-imports:net4_start_responsive"
$responsiveStart = (Get-LastAgentResponseJson -Method $netMethod).body.result
$responsiveSuspended = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_NET_SHIM suspended=true scenario=responsive operation=tcp_open" -Offset $responsiveOffset -TimeoutSeconds 2
Add-Predicate -Name "m11-net-imports:net4_responsive_real_suspend" -Expected "tcp_open reaches a real typed HostSuspend before the main-loop task touches TCP" -Passed $responsiveSuspended -Actual $(if ($responsiveSuspended) { "suspended" } else { Get-SerialLogTail -Path $SerialLog })
if (-not $responsiveSuspended) { throw "Expected responsive net fixture to suspend at tcp_open" }
$responsiveFinished = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_NET_SHIM outcome=finished scenario=responsive teardown_complete=true" -Offset $responsiveOffset -TimeoutSeconds 8
if (-not $responsiveFinished) { throw "Expected TCP DNS fixture to finish against 10.0.2.3:53" }
Send-AgentCommand -Command "$netMethod status" -ExpectedMarker "RAIOS_AGENT_END $netMethod" -Name "m11-net-imports:net4_responsive_status"
$responsive = (Get-LastAgentResponseJson -Method $netMethod).body.result
$responsiveBytes = $responsiveStart.request_status -eq "accepted_pending" -and
    $responsive.outcome -eq "finished" -and $responsive.active -eq $false -and
    $responsive.responsive_peer -eq "10.0.2.3:53" -and
    [int]$responsive.tx_bytes -eq 31 -and [int]$responsive.rx_bytes -ge 2 -and
    $responsive.dns_tcp_length_prefix_present -eq $true -and
    [int]$responsive.received_prefix_le -gt 0 -and
    [int]$responsive.guest_return_value -eq [int]$responsive.received_prefix_le -and
    $responsive.rx_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    $responsive.teardown_complete -eq $true -and $responsive.singleton_lease_held -eq $false
Add-Predicate -Name "m11-net-imports:net4_responsive_dns_bytes" -Expected "main-loop writes real TCP DNS response bytes before resume; the guest reads the retained prefix and the lease releases" -Passed $responsiveBytes -Actual $(if ($responsiveBytes) { "tx=31 rx=$($responsive.rx_bytes) prefix=$($responsive.guest_return_value)" } else { $responsive | ConvertTo-Json -Compress -Depth 8 })
if (-not $responsiveBytes) { throw "Expected real TCP DNS bytes through suspend/resume" }
$callEvidence = [int]$responsive.suspension_count -eq 3 -and [int]$responsive.resume_count -eq 3 -and
    [int]$responsive.open_operation_id -eq 1 -and [int]$responsive.send_operation_id -eq 2 -and
    [int]$responsive.recv_operation_id -eq 3 -and [int]$responsive.close_call_count -eq 2 -and
    [int]$responsive.first_close_operation_id -eq 4 -and [int]$responsive.second_close_operation_id -eq 5 -and
    [int]$responsive.fixed_pending_buffer_capacity -eq 4096 -and
    $responsive.would_block_guest_visible -eq $false
Add-Predicate -Name "m11-net-imports:net4_one_op_per_suspend" -Expected "open/send/recv each own one ordered pending operation, both closes are immediate/idempotent, and would_block never reaches the guest" -Passed $callEvidence -Actual $(if ($callEvidence) { "ops=1,2,3 suspends=3 closes=2" } else { $responsive | ConvertTo-Json -Compress -Depth 8 })
if (-not $callEvidence) { throw "Expected one pending operation per suspension and no guest-visible would_block" }

$timeoutOffset = Get-SerialLogOffset
Send-AgentCommand -Command "$netMethod start_timeout" -ExpectedMarker "RAIOS_AGENT_END $netMethod" -Name "m11-net-imports:net4_start_timeout"
$timeoutSuspended = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_NET_SHIM suspended=true scenario=silent_timeout operation=tcp_open" -Offset $timeoutOffset -TimeoutSeconds 2
if (-not $timeoutSuspended) { throw "Expected silent timeout fixture to suspend" }
$timeoutFinished = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_NET_SHIM outcome=timed_out scenario=silent_timeout teardown_complete=true" -Offset $timeoutOffset -TimeoutSeconds 7
Send-AgentCommand -Command "$netMethod status" -ExpectedMarker "RAIOS_AGENT_END $netMethod" -Name "m11-net-imports:net4_timeout_status"
$timedOut = (Get-LastAgentResponseJson -Method $netMethod).body.result
$timeoutOk = $timeoutFinished -and $timedOut.outcome -eq "timed_out" -and
    $timedOut.silent_peer -eq "10.0.2.254:53" -and [int]$timedOut.suspension_count -eq 1 -and
    [int]$timedOut.resume_count -eq 1 -and [int]$timedOut.timeout_run_count -eq 1 -and
    $timedOut.singleton_lease_held -eq $false -and $timedOut.would_block_guest_visible -eq $false
Add-Predicate -Name "m11-net-imports:net4_silent_timeout" -Expected "the unused slirp-subnet peer stays suspended until the 5-second typed timeout, then releases without exposing would_block" -Passed $timeoutOk -Actual $(if ($timeoutOk) { "typed timeout" } else { $timedOut | ConvertTo-Json -Compress -Depth 8 })
if (-not $timeoutOk) { throw "Expected deterministic silent-peer timeout and lease release" }

$killOffset = Get-SerialLogOffset
Send-AgentCommand -Command "$netMethod start_kill" -ExpectedMarker "RAIOS_AGENT_END $netMethod" -Name "m11-net-imports:net4_start_kill"
$killSuspended = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_NET_SHIM suspended=true scenario=silent_kill operation=tcp_open" -Offset $killOffset -TimeoutSeconds 2
if (-not $killSuspended) { throw "Expected silent kill fixture to suspend before F12" }
Send-AgentCommand -Command "$netMethod status" -ExpectedMarker "RAIOS_AGENT_END $netMethod" -Name "m11-net-imports:net4_busy_while_suspended"
$busy = (Get-LastAgentResponseJson -Method $netMethod).body.result
$busyOk = $busy.active -eq $true -and $busy.suspended -eq $true -and
    $busy.representative_run_outcome -eq "wasm_execution_busy" -and [int]$busy.resume_count -eq 0
Add-Predicate -Name "m11-net-imports:net4_central_reentrancy_busy" -Expected "wasm_runtime refuses a representative echo instance with typed wasm_execution_busy while the net continuation exists" -Passed $busyOk -Actual $(if ($busyOk) { "busy denied" } else { $busy | ConvertTo-Json -Compress -Depth 8 })
if (-not $busyOk) { throw "Expected central Wasm execution busy denial while suspended" }

$f12Offset = Get-SerialLogOffset
$killSentAt = [DateTime]::UtcNow
Send-QemuMonitorCommand -Command "sendkey f12 60" -ReplyWaitMilliseconds 0 | Out-Null
$monitorAcknowledgedAt = [DateTime]::UtcNow
$killNeedle = "RAIOS_NET_SHIM outcome=killed scenario=silent_kill teardown_complete=true"
$killFound = $false
$killDeadline = $killSentAt.AddMilliseconds(250)
do {
    $content = Get-SerialLogContent -Path $SerialLog
    if ($null -ne $content -and $f12Offset -lt $content.Length -and $content.Substring([int]$f12Offset) -clike "*$killNeedle*") {
        $killFound = $true
        break
    }
    Start-Sleep -Milliseconds 5
} while ([DateTime]::UtcNow -lt $killDeadline)
$killObservedAt = [DateTime]::UtcNow
$sendToMarkerMs = [int][Math]::Round(($killObservedAt - $killSentAt).TotalMilliseconds)
$ackToMarkerMs = [Math]::Max(0, [int][Math]::Round(($killObservedAt - $monitorAcknowledgedAt).TotalMilliseconds))
Add-Predicate -Name "m11-net-imports:net4_physical_f12_monitor" -Expected "QEMU HMP sendkey f12 kills the silent-peer invocation before its timeout" -Passed $killFound -Actual $(if ($killFound) { "monitor path" } else { Get-SerialLogTail -Path $SerialLog })
$killBound = $killFound -and $sendToMarkerMs -le 250 -and $ackToMarkerMs -le 250
Add-Predicate -Name "m11-net-imports:net4_f12_host_bound" -Expected "F12 send/ack to net teardown marker is at most 250 ms" -Passed $killBound -Actual "send_to_marker_ms=$sendToMarkerMs ack_to_marker_ms=$ackToMarkerMs"
if (-not $killBound) { throw "Expected F12 net teardown within 250 ms and before timeout" }

Send-AgentCommand -Command "$netMethod status" -ExpectedMarker "RAIOS_AGENT_END $netMethod" -Name "m11-net-imports:net4_killed_status"
$killed = (Get-LastAgentResponseJson -Method $netMethod).body.result
$guestKillDelta = [int64]$killed.terminal_boot_ms - [int64]$killed.kill_observed_boot_ms
$killedOk = $killed.outcome -eq "killed" -and $killed.active -eq $false -and
    $killed.no_resume_after_kill -eq $true -and [int]$killed.resume_count -eq 0 -and
    [int]$killed.teardown_count -eq 1 -and $killed.teardown_complete -eq $true -and
    $killed.singleton_lease_held -eq $false -and [int]$killed.killed_run_count -eq 1 -and
    $guestKillDelta -ge 0 -and $guestKillDelta -le 16
Add-Predicate -Name "m11-net-imports:net4_kill_no_resume_cleanup" -Expected "kill prevents resume, tears down exactly once, releases the lease, and meets the 16 ms guest boundary" -Passed $killedOk -Actual $(if ($killedOk) { "guest_delta_ms=$guestKillDelta" } else { $killed | ConvertTo-Json -Compress -Depth 8 })
if (-not $killedOk) { throw "Expected kill-before-resume and exactly-once net teardown" }
$reopened = $killed.representative_run_outcome -eq "success"
Add-Predicate -Name "m11-net-imports:net4_execution_reopens_after_teardown" -Expected "the representative Wasm entry succeeds again after active-task teardown" -Passed $reopened -Actual $killed.representative_run_outcome
if (-not $reopened) { throw "Expected central Wasm execution gate to reopen after teardown" }

$retryOffset = Get-SerialLogOffset
Send-AgentCommand -Command "$netMethod start_responsive" -ExpectedMarker "RAIOS_AGENT_END $netMethod" -Name "m11-net-imports:net4_retry_after_kill"
$retryFinished = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_NET_SHIM outcome=finished scenario=responsive teardown_complete=true" -Offset $retryOffset -TimeoutSeconds 8
Send-AgentCommand -Command "$netMethod status" -ExpectedMarker "RAIOS_AGENT_END $netMethod" -Name "m11-net-imports:net4_retry_status"
$retry = (Get-LastAgentResponseJson -Method $netMethod).body.result
$retryOk = $retryFinished -and $retry.outcome -eq "finished" -and
    [int]$retry.responsive_run_count -eq 2 -and $retry.singleton_lease_held -eq $false
Add-Predicate -Name "m11-net-imports:net4_lease_retry_after_kill" -Expected "a fresh responsive invocation claims a new lease and completes after F12 teardown" -Passed $retryOk -Actual $(if ($retryOk) { "responsive_run_count=2" } else { $retry | ConvertTo-Json -Compress -Depth 8 })
if (-not $retryOk) { throw "Expected clean net fixture retry after F12 teardown" }
