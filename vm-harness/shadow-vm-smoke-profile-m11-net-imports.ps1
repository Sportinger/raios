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
