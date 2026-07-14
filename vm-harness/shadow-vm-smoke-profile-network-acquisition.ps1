if (-not $Network) { throw "network-acquisition requires -Network" }

$method = "wasm.acquisition_service_probe"
$startMethod = "network.transport_lease_probe"
$fixtureWrapper = Join-Path $PSScriptRoot "net8-w7-tls-fixture.ps1"
$fixtureReadyPath = Join-Path $RunDir "w7-fixture-ready.json"
$fixtureResultPath = Join-Path $RunDir "w7-fixture-result.json"
$fixtureModePath = Join-Path $RunDir "w7-fixture-mode.txt"
$expectedArtifact = "sha256:32a018b0c730a4f85210ca820483ca68f8a4d0715021a1dda97951fe305e9e54"
$expectedImports = "sha256:eb390ec5c2dfde5ac632b127515c5101c812ed6ca209191846bc762409bf4345"
$expectedPayload = "sha256:f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2"

Send-AgentCommand -Command $method -ExpectedMarker "RAIOS_AGENT_END $method" -Name "network-acquisition:arming-boundary"
$armed = (Get-LastAgentResponseJson -Method $method).body.result
$boundaryOk = $armed.service_id -eq "svc.net.acquire.w7" -and
    $armed.source_policy_id -eq "local.qemu.w7" -and
    $armed.artifact_sha256 -eq $expectedArtifact -and $armed.import_list_sha256 -eq $expectedImports -and
    $armed.policy_allows_beyond_env -eq $true -and $armed.production_linker_armed -eq $true -and
    $armed.live_pin_configured -eq $true -and
    $armed.different_service_denied_before_instantiation -eq $true -and
    $armed.artifact_hash_mismatch_denied -eq $true -and
    $armed.import_list_hash_mismatch_denied -eq $true -and
    $armed.source_policy_mismatch_denied -eq $true -and [int]$armed.run_count -eq 0 -and
    $armed.instantiation_attempted -eq $false -and $armed.network_effect -eq $false -and
    $armed.crypto_effect -eq $false -and $armed.acquisition_effect -eq $false -and
    $armed.candidate_load_attempted -eq $false -and $armed.durable_write_attempted -eq $false
Add-Predicate -Name "network-acquisition:7_exact_identity_only" -Expected "only the literal W7 artifact/import-list/local.qemu.w7 binding is armed; another service and each independent mismatch deny before instantiation with zero effects" -Passed $boundaryOk -Actual $(if ($boundaryOk) { "exact binding only" } else { $armed | ConvertTo-Json -Compress -Depth 8 })
if (-not $boundaryOk) { throw "NET-8 exact-identity arming boundary failed" }

& $fixtureWrapper -Action SetMode -ModePath $fixtureModePath -Mode serve
Remove-Item -LiteralPath $fixtureResultPath -Force -ErrorAction SilentlyContinue
$positiveOffset = Get-SerialLogOffset
Send-AgentCommand -Command "$startMethod start_w7" -ExpectedMarker "RAIOS_AGENT_END $startMethod" -Name "network-acquisition:start-live"
$positiveStart = (Get-LastAgentResponseJson -Method $startMethod).body.result
$positiveFinished = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_W7_ACQUISITION outcome=finished source_tls_evidence=true candidate_retained=true" -Offset $positiveOffset -TimeoutSeconds 30
$fixtureDeadline = [DateTime]::UtcNow.AddSeconds(5)
while (-not (Test-Path -LiteralPath $fixtureResultPath -PathType Leaf) -and [DateTime]::UtcNow -lt $fixtureDeadline) { Start-Sleep -Milliseconds 20 }
if (-not (Test-Path -LiteralPath $fixtureResultPath -PathType Leaf)) { throw "positive TLS fixture result missing" }
$fixturePositive = Get-Content -LiteralPath $fixtureResultPath -Raw | ConvertFrom-Json
Send-AgentCommand -Command $method -ExpectedMarker "RAIOS_AGENT_END $method" -Name "network-acquisition:positive-status"
$positive = (Get-LastAgentResponseJson -Method $method).body.result
$positiveOk = $positiveFinished -and $positiveStart.request_status -eq "accepted_pending" -and
    [int]$fixturePositive.host_connection -eq 1 -and [int]$fixturePositive.session -eq 1 -and
    $fixturePositive.mode -eq "serve" -and $fixturePositive.request_exact -eq $true -and
    $fixturePositive.tls_protocol -eq "Tls13" -and
    $fixturePositive.cipher_suite -eq "TLS_AES_128_GCM_SHA256" -and
    $fixturePositive.request_path -eq "/raios/cas/sha256/f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2" -and
    $positive.outcome -eq "finished" -and $positive.source_tls_evidence -eq $true -and
    $positive.tls_protocol -eq "TLS1.3" -and $positive.tls_cipher_suite -eq "0x1301" -and
    $positive.tls_key_exchange_group -eq "P-256" -and $positive.certificate_verify_math -eq $true -and
    $positive.ephemeral_spki_pin_match -eq $true -and $positive.server_finished_valid -eq $true -and
    [int]$positive.http_status -eq 200 -and $positive.http_content_type -eq "application/octet-stream" -and
    [int]$positive.http_content_length -eq 4205 -and $positive.every_chunk_hash_valid -eq $true -and
    $positive.whole_hash_valid -eq $true -and $positive.candidate_sha256 -eq $expectedPayload
Add-Predicate -Name "network-acquisition:1_positive_live_fetch" -Expected "persistent host connection 1/session 1 performs the real e1000/DHCP/10.0.2.100:8443/TLS1.3-0x1301-P256 pinned-SPKI/server-Finished/exact-HTTP fetch and verifies every chunk and whole hash" -Passed $positiveOk -Actual $(if ($positiveOk) { "live fetch verified on persistent relay session 1" } else { @{ fixture = $fixturePositive; guest = $positive } | ConvertTo-Json -Compress -Depth 10 })
if (-not $positiveOk) { throw "NET-8 positive live fetch failed" }

Send-AgentCommand -Command "module.load_ephemeral svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "network-acquisition:retained-preflight"
$load = Get-LastAgentResponseJson -Method "module.load_ephemeral"
$loadRuntime = ($load.evidence | Where-Object id -eq "loader_runtime").facts
$preflight = $loadRuntime.receiver_identity_load_preflight
$preflightOk = $load.schema -eq "raios.evidence_response.v1" -and $load.decision.outcome -eq "denied" -and
    @($load.decision.grants).Count -eq 0 -and @($load.decision.effects).Count -eq 0 -and
    $preflight.status -eq "denied" -and
    $preflight.reason -eq "receiver_identity_catalog_entry_not_found" -and
    $preflight.present -eq $false -and
    $preflight.receiver_identity_retained -eq $false -and
    $preflight.receiver_identity_complete -eq $false -and
    $preflight.retained_candidate_present -eq $false -and
    $preflight.retained_candidate_matches_catalog_finalize -eq $false -and
    $preflight.preflight_evaluated -eq $false -and
    [int]$preflight.missing_gate_count -eq 0 -and
    $preflight.requires_m6_m7_reverify_for_load -eq $true -and
    $positive.candidate_load_attempted -eq $false -and
    $positive.candidate_install_attempted -eq $false -and
    $positive.candidate_execution_attempted -eq $false -and
    $positive.durable_write_attempted -eq $false -and
    $positive.rollback_mutation_attempted -eq $false -and
    $positive.provider_auto_load_attempted -eq $false
Add-Predicate -Name "network-acquisition:3_retained_candidate_preflight_denial" -Expected "the retained current_boot candidate has no distinct catalog receiver identity or evaluated load preflight, requires M6/M7 reverify, and cannot load, install, execute, persist, mutate rollback, or auto-load for a provider" -Passed $preflightOk -Actual $(if ($preflightOk) { "receiver identity and load preflight missing; M6/M7 reverify required" } else { $load | ConvertTo-Json -Compress -Depth 10 })
if (-not $preflightOk) { throw "NET-8 retained candidate did not remain inert without a receiver identity load preflight" }

Send-AgentCommand -Command "wasm.acquire_import_probe" -ExpectedMarker "RAIOS_AGENT_END wasm.acquire_import_probe" -Name "network-acquisition:shared-finalizer"
$shared = (Get-LastAgentResponseJson -Method "wasm.acquire_import_probe").body.result
$sharedOk = $shared.fixture_complete -eq $true -and $shared.candidate_hash_converged -eq $true -and
    $shared.receipt_hash_converged -eq $false -and $null -eq $shared.serial_receipt_sha256 -and
    $positive.candidate_sha256 -eq $shared.service_candidate_sha256 -and
    $positive.receipt_sha256 -eq $shared.service_receipt_sha256 -and
    $positive.candidate_scope -eq "current_boot" -and $positive.candidate_inert -eq $true -and
    $positive.same_shared_acquire_finalizer -eq $true -and $positive.w7_private_success_store -eq $false
Add-Predicate -Name "network-acquisition:2_shared_finalize_convergence" -Expected "live W7 and the acquire service converge on the same inert candidate and byte-identical receipt through one shared finalizer, with no W7-private store and no unrelated serial receipt" -Passed $sharedOk -Actual $(if ($sharedOk) { "live W7 and acquire service hashes converged; serial receipt honestly absent" } else { @{ live = $positive; shared = $shared } | ConvertTo-Json -Compress -Depth 10 })
if (-not $sharedOk) { throw "NET-8 did not converge on the shared finalizer" }

$busyOffset = Get-SerialLogOffset
Send-AgentCommand -Command "$startMethod start_w7_provider_busy" -ExpectedMarker "RAIOS_AGENT_END $startMethod" -Name "network-acquisition:provider-blocks-w7"
$busyFinished = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_W7_ACQUISITION outcome=resource_busy" -Offset $busyOffset -TimeoutSeconds 5
Send-AgentCommand -Command $method -ExpectedMarker "RAIOS_AGENT_END $method" -Name "network-acquisition:provider-busy-status"
$providerBusy = (Get-LastAgentResponseJson -Method $method).body.result

& $fixtureWrapper -Action SetMode -ModePath $fixtureModePath -Mode silent
$silentOffset = Get-SerialLogOffset
Send-AgentCommand -Command "$startMethod start_w7" -ExpectedMarker "RAIOS_AGENT_END $startMethod" -Name "network-acquisition:start-silent"
$silentWaiting = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_NET_SHIM suspended=true scenario=w7_live operation=tcp_recv" -Offset $silentOffset -TimeoutSeconds 5
Send-AgentCommand -Command $method -ExpectedMarker "RAIOS_AGENT_END $method" -Name "network-acquisition:w7-blocks-provider"
$activeW7 = (Get-LastAgentResponseJson -Method $method).body.result
$bothBusyOk = $busyFinished -and $providerBusy.outcome -eq "resource_busy" -and
    $silentWaiting -and $activeW7.active -eq $true -and
    $activeW7.w7_blocks_provider_reason -eq "network_transport_busy"
Add-Predicate -Name "network-acquisition:5_provider_acquisition_busy_both_directions" -Expected "the native provider owner blocks W7 and an active W7 lease blocks the native provider with the same busy class; neither steals the lease" -Passed $bothBusyOk -Actual $(if ($bothBusyOk) { "network_transport_busy both directions" } else { @{ provider_blocks_w7 = $providerBusy; w7_blocks_provider = $activeW7 } | ConvertTo-Json -Compress -Depth 8 })
if (-not $bothBusyOk) { throw "NET-8 shared lease did not deny both directions" }

$killSentAt = [DateTime]::UtcNow
Send-QemuMonitorCommand -Command "sendkey f12 60" -ReplyWaitMilliseconds 0 | Out-Null
$killed = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_W7_ACQUISITION outcome=killed" -Offset $silentOffset -TimeoutSeconds 1
$killMs = [int][Math]::Round(([DateTime]::UtcNow - $killSentAt).TotalMilliseconds)
Send-AgentCommand -Command $method -ExpectedMarker "RAIOS_AGENT_END $method" -Name "network-acquisition:killed-status"
$killStatus = (Get-LastAgentResponseJson -Method $method).body.result
$killOk = $killed -and $killMs -le 250 -and $killStatus.outcome -eq "killed" -and
    $killStatus.no_resume_after_kill -eq $true -and [int]$killStatus.teardown_count -eq 1 -and
    $killStatus.crypto_session_zeroized -eq $true -and $killStatus.transport_lease_held -eq $false -and
    $killStatus.pending_acquisition_present -eq $false -and $killStatus.prior_candidate_preserved -eq $true
Add-Predicate -Name "network-acquisition:4_f12_silent_peer_cleanup" -Expected "monitor F12 cancels a silent peer within 250 ms, never resumes, zeroizes crypto, completes guest/local socket and lease cleanup, drops incomplete bytes, and preserves the prior candidate" -Passed $killOk -Actual $(if ($killOk) { "killed in ${killMs}ms; cleaned once" } else { $killStatus | ConvertTo-Json -Compress -Depth 8 })
if (-not $killOk) { throw "NET-8 silent-peer F12 cleanup failed" }

& $fixtureWrapper -Action SetMode -ModePath $fixtureModePath -Mode malformed
$transitionDeadline = [DateTime]::UtcNow.AddSeconds(2)
$fixtureTransition = $null
while ([DateTime]::UtcNow -lt $transitionDeadline)
{
    if (Test-Path -LiteralPath $fixtureResultPath -PathType Leaf)
    {
        try { $fixtureTransition = Get-Content -LiteralPath $fixtureResultPath -Raw | ConvertFrom-Json }
        catch { $fixtureTransition = $null }
        if ($fixtureTransition.mode -eq "silent" -and [int]$fixtureTransition.host_connection -eq 1 -and
            [int]$fixtureTransition.session -eq 2 -and $fixtureTransition.relay_session_advanced -eq $true -and
            $fixtureTransition.reason -eq "mode_transition" -and
            $fixtureTransition.phase -eq "after_raw_relay_before_tls" -and
            [int]$fixtureTransition.drained_bytes -gt 0) { break }
    }
    Start-Sleep -Milliseconds 20
}
$transitionOk = $fixtureTransition.mode -eq "silent" -and [int]$fixtureTransition.host_connection -eq 1 -and
    [int]$fixtureTransition.session -eq 2 -and $fixtureTransition.relay_session_advanced -eq $true -and
    $fixtureTransition.reason -eq "mode_transition" -and
    $fixtureTransition.phase -eq "after_raw_relay_before_tls" -and
    [int]$fixtureTransition.drained_bytes -gt 0
Add-Predicate -Name "network-acquisition:4b_silent_relay_session_advance" -Expected "persistent host connection 1 drains silent relay session 2 and advances it on the observed mode transition before malformed acquisition starts" -Passed $transitionOk -Actual $(if ($transitionOk) { "silent relay session 2 drained and advanced" } elseif ($null -eq $fixtureTransition) { "transition result absent" } else { $fixtureTransition | ConvertTo-Json -Compress -Depth 6 })
if (-not $transitionOk) { throw "NET-8 silent relay session did not advance before malformed acquisition" }
Remove-Item -LiteralPath $fixtureResultPath -Force -ErrorAction SilentlyContinue
$malformedOffset = Get-SerialLogOffset
Send-AgentCommand -Command "$startMethod start_w7" -ExpectedMarker "RAIOS_AGENT_END $startMethod" -Name "network-acquisition:malformed-response"
$malformedFinished = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_W7_ACQUISITION outcome=guest_denied" -Offset $malformedOffset -TimeoutSeconds 15
Send-AgentCommand -Command $method -ExpectedMarker "RAIOS_AGENT_END $method" -Name "network-acquisition:malformed-status"
$malformedStatus = (Get-LastAgentResponseJson -Method $method).body.result
& $fixtureWrapper -Action SetMode -ModePath $fixtureModePath -Mode serve
Remove-Item -LiteralPath $fixtureResultPath -Force -ErrorAction SilentlyContinue
$retryOffset = Get-SerialLogOffset
Send-AgentCommand -Command "$startMethod start_w7" -ExpectedMarker "RAIOS_AGENT_END $startMethod" -Name "network-acquisition:valid-retry"
$retryFinished = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_W7_ACQUISITION outcome=finished source_tls_evidence=true candidate_retained=true" -Offset $retryOffset -TimeoutSeconds 30
Send-AgentCommand -Command $method -ExpectedMarker "RAIOS_AGENT_END $method" -Name "network-acquisition:retry-status"
$retry = (Get-LastAgentResponseJson -Method $method).body.result
$cleanupRetryOk = $malformedFinished -and $malformedStatus.outcome -eq "guest_denied" -and
    [int]$malformedStatus.teardown_count -eq 1 -and $malformedStatus.teardown_complete -eq $true -and
    $malformedStatus.crypto_session_zeroized -eq $true -and $malformedStatus.transport_lease_held -eq $false -and
    $retryFinished -and [int]$retry.success_count -ge 2 -and
    [int]$retry.teardown_count -eq 1 -and $retry.teardown_complete -eq $true -and
    $retry.crypto_session_zeroized -eq $true -and $retry.transport_lease_held -eq $false -and
    $armed.guest_trap_cleanup -eq $true -and $armed.out_of_fuel_cleanup -eq $true
Add-Predicate -Name "network-acquisition:6_cleanup_and_retry" -Expected "silence, malformed response, guest trap, OutOfFuel, and cancellation share exactly-once teardown, then a valid request succeeds in the same boot" -Passed $cleanupRetryOk -Actual $(if ($cleanupRetryOk) { "all cleanup classes; same-boot retry succeeded" } else { $retry | ConvertTo-Json -Compress -Depth 8 })
if (-not $cleanupRetryOk) { throw "NET-8 cleanup-and-retry proof failed" }
