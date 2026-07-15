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
$provenanceSignatureHex = "304402201fd9aa3e26579ab9852a1ea61a7fe23f79c39badd13e2c74dbdf9d957a25449b02204f783191894cfb609d35c5babc9fb3208e77d9c712d2645a633120db6dcdd89b"

function Get-W7BytesSha256Hex {
    param([byte[]]$Bytes)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try { return (($sha.ComputeHash($Bytes) | ForEach-Object { $_.ToString("x2") }) -join "") }
    finally { $sha.Dispose() }
}

function Convert-W7HexToBytes {
    param([string]$Hex)
    $trimmed = $Hex.Trim()
    if (($trimmed.Length % 2) -ne 0 -or $trimmed -notmatch '^[0-9a-fA-F]+$') {
        throw "W7 receiver evidence is not bounded even-length hex"
    }
    $bytes = New-Object byte[] ($trimmed.Length / 2)
    for ($index = 0; $index -lt $bytes.Length; $index++) {
        $bytes[$index] = [Convert]::ToByte($trimmed.Substring($index * 2, 2), 16)
    }
    return $bytes
}

function Get-W7ReceiverEvidence {
    param([string]$Kind, [string]$Path, [switch]$Hex)
    $bytes = if ($Hex) {
        Convert-W7HexToBytes -Hex ([System.IO.File]::ReadAllText($Path))
    }
    else {
        [System.IO.File]::ReadAllBytes($Path)
    }
    return [pscustomobject]@{
        kind = $Kind
        bytes = $bytes
        sha256 = Get-W7BytesSha256Hex -Bytes $bytes
    }
}

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

$missingIdentityOffset = Get-SerialLogOffset
Send-AgentCommand -Command "$startMethod start_w7" -ExpectedMarker "RAIOS_AGENT_END $startMethod" -Name "network-acquisition:missing-receiver-identity"
$missingIdentityFinished = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_W7_ACQUISITION outcome=guest_denied source_tls_evidence=true candidate_retained=false" -Offset $missingIdentityOffset -TimeoutSeconds 5
Send-AgentCommand -Command $method -ExpectedMarker "RAIOS_AGENT_END $method" -Name "network-acquisition:missing-receiver-identity-status"
$missingIdentity = (Get-LastAgentResponseJson -Method $method).body.result
$missingIdentityOk = $missingIdentityFinished -and $missingIdentity.outcome -eq "guest_denied" -and
    $missingIdentity.request_status -eq "complete" -and [int]$missingIdentity.run_count -eq 1 -and
    [int]$missingIdentity.success_count -eq 0 -and [int]$missingIdentity.tx_bytes -gt 0 -and
    [int]$missingIdentity.rx_bytes -gt 0 -and $missingIdentity.network_effect -eq $true -and
    $missingIdentity.crypto_effect -eq $true -and $missingIdentity.acquisition_effect -eq $false -and
    $missingIdentity.source_tls_evidence -eq $true -and $null -eq $missingIdentity.candidate_sha256 -and
    $null -eq $missingIdentity.receipt_sha256 -and $missingIdentity.pending_acquisition_present -eq $false -and
    $missingIdentity.candidate_load_attempted -eq $false -and
    $missingIdentity.candidate_install_attempted -eq $false -and
    $missingIdentity.candidate_execution_attempted -eq $false -and
    $missingIdentity.durable_write_attempted -eq $false -and
    $missingIdentity.teardown_complete -eq $true -and [int]$missingIdentity.teardown_count -eq 1
Add-Predicate -Name "network-acquisition:8_missing_receiver_identity_denies_candidate_accept" -Expected "without an exact guest-complete catalog receiver identity, the bounded authorized TLS fetch may finish but shared candidate acceptance denies with no candidate, receipt, load, install, execution, or durable effect" -Passed $missingIdentityOk -Actual $(if ($missingIdentityOk) { "TLS bytes discarded; candidate acceptance denied" } else { $missingIdentity | ConvertTo-Json -Compress -Depth 8 })
if (-not $missingIdentityOk) { throw "B1.2a W7 did not fail closed without a complete receiver identity" }

$descriptorRoot = Join-Path $RepoRoot "seed-kernel\descriptors"
$receiverEvidence = @(
    Get-W7ReceiverEvidence -Kind artifact_identity_descriptor -Path (Join-Path $descriptorRoot "svc.demo.echo.wasm_artifact_identity.desc")
    Get-W7ReceiverEvidence -Kind artifact_identity_public_key -Path (Join-Path $descriptorRoot "svc.demo.echo.wasm_artifact_identity.p256.pub.hex") -Hex
    Get-W7ReceiverEvidence -Kind artifact_identity_signature -Path (Join-Path $descriptorRoot "svc.demo.echo.wasm_artifact_identity.p256.sig.der.hex") -Hex
    Get-W7ReceiverEvidence -Kind load_descriptor -Path (Join-Path $descriptorRoot "svc.demo.echo.current_boot_load.desc")
    Get-W7ReceiverEvidence -Kind load_descriptor_public_key -Path (Join-Path $descriptorRoot "svc.demo.echo.current_boot_load.p256.pub.hex") -Hex
    Get-W7ReceiverEvidence -Kind load_descriptor_signature -Path (Join-Path $descriptorRoot "svc.demo.echo.current_boot_load.p256.sig.der.hex") -Hex
)
$artifactIdentityDescriptorSha = $receiverEvidence[0].sha256
$artifactIdentityPublicKeySha = $receiverEvidence[1].sha256
$artifactIdentitySignatureSha = $receiverEvidence[2].sha256
$loadDescriptorSha = $receiverEvidence[3].sha256
$loadDescriptorPublicKeySha = $receiverEvidence[4].sha256
$loadDescriptorSignatureSha = $receiverEvidence[5].sha256

Send-AgentCommand -Command "module.submit_distribution_catalog_entry $expectedPayload 4205 1 sig:$provenanceSignatureHex" -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_catalog_entry" -Name "network-acquisition:catalog-entry"
$catalogEntry = (Get-LastAgentResponseJson -Method "module.submit_distribution_catalog_entry").body.result
$catalogEntryOk = $catalogEntry.accepted -eq $true -and $catalogEntry.rejected -eq $false -and
    $catalogEntry.content_sha256 -eq $expectedPayload -and [int]$catalogEntry.total_length -eq 4205 -and
    [int]$catalogEntry.chunk_count -eq 1 -and $catalogEntry.receiver_identity_retained -eq $false -and
    $catalogEntry.authorizes_load -eq $false -and $catalogEntry.authorizes_install -eq $false -and
    $catalogEntry.authorizes_execute -eq $false -and $catalogEntry.writes_persistent_state -eq $false
Add-Predicate -Name "network-acquisition:9_exact_catalog_entry" -Expected "the exact W7 hash/length/one-chunk shape is retained in the existing local catalog without authority" -Passed $catalogEntryOk -Actual $(if ($catalogEntryOk) { "exact inert catalog entry retained" } else { $catalogEntry | ConvertTo-Json -Compress -Depth 8 })
if (-not $catalogEntryOk) { throw "B1.2a exact W7 catalog entry failed" }

$receiverIdentityCommand = "module.submit_distribution_receiver_identity $expectedPayload sha256:$artifactIdentityDescriptorSha sha256:$artifactIdentityPublicKeySha sha256:$artifactIdentitySignatureSha sha256:$loadDescriptorSha sha256:$loadDescriptorPublicKeySha sha256:$loadDescriptorSignatureSha classification:local_only artifact_identity_signature_verified:true load_descriptor_signature_verified:true artifact_hash_bound_by_identity:true artifact_hash_bound_by_load_descriptor:true load_descriptor_binds_artifact_identity:true load_descriptor_authorizes_current_boot_wasm_execution:true export_authorizes_load:false export_authorizes_install:false export_authorizes_execute:false export_writes_persistent_state:false requires_m6_m7_reverify_for_load:true"
Send-AgentCommand -Command $receiverIdentityCommand -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_receiver_identity" -Name "network-acquisition:receiver-identity"
$receiverIdentity = (Get-LastAgentResponseJson -Method "module.submit_distribution_receiver_identity").body.result
$receiverIdentityOk = $receiverIdentity.accepted -eq $true -and $receiverIdentity.rejected -eq $false -and
    $receiverIdentity.content_sha256 -eq $expectedPayload -and $receiverIdentity.retained_in_catalog -eq $true -and
    $receiverIdentity.receiver_identity.receiver_identity_complete -eq $false -and
    $receiverIdentity.guest_signature_verification_performed -eq $false -and
    $receiverIdentity.requires_m6_m7_reverify_for_load -eq $true -and
    $receiverIdentity.authorizes_load -eq $false -and $receiverIdentity.authorizes_install -eq $false -and
    $receiverIdentity.authorizes_execute -eq $false -and $receiverIdentity.writes_persistent_state -eq $false
Add-Predicate -Name "network-acquisition:10_receiver_identity_metadata" -Expected "the existing echo receiver identity metadata is catalog-bound but remains incomplete and non-authorizing before guest evidence verification" -Passed $receiverIdentityOk -Actual $(if ($receiverIdentityOk) { "receiver metadata retained inert" } else { $receiverIdentity | ConvertTo-Json -Compress -Depth 9 })
if (-not $receiverIdentityOk) { throw "B1.2a receiver identity metadata failed" }

$receiverEvidenceOk = $true
foreach ($part in $receiverEvidence) {
    $payload = [Convert]::ToBase64String($part.bytes)
    $command = "module.submit_distribution_receiver_identity_evidence $expectedPayload $($part.kind) sha256:$($part.sha256) $payload"
    Send-AgentCommand -Command $command -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_receiver_identity_evidence" -Name "network-acquisition:receiver-evidence-$($part.kind)"
    $evidencePartResult = (Get-LastAgentResponseJson -Method "module.submit_distribution_receiver_identity_evidence").body.result
    $receiverEvidenceOk = $receiverEvidenceOk -and $evidencePartResult.accepted -eq $true -and $evidencePartResult.rejected -eq $false -and
        $evidencePartResult.content_sha256 -eq $expectedPayload -and $evidencePartResult.evidence_kind -eq $part.kind -and
        [int]$evidencePartResult.decoded_byte_len -eq $part.bytes.Length -and $evidencePartResult.receiver_identity_complete -eq $false -and
        $evidencePartResult.guest_signature_verification_performed -eq $false -and $evidencePartResult.authorizes_load -eq $false -and
        $evidencePartResult.authorizes_install -eq $false -and $evidencePartResult.authorizes_execute -eq $false -and
        $evidencePartResult.writes_persistent_state -eq $false
}
Add-Predicate -Name "network-acquisition:11_receiver_identity_evidence" -Expected "all six exact descriptor/key/signature payloads are retained in RAM without load, execute, install, or durable authority" -Passed $receiverEvidenceOk -Actual $(if ($receiverEvidenceOk) { "six inert evidence parts retained" } else { "receiver evidence rejected or authorized an effect" })
if (-not $receiverEvidenceOk) { throw "B1.2a receiver identity evidence failed" }

Send-AgentCommand -Command "module.submit_distribution_receiver_identity_finalize $expectedPayload" -ExpectedMarker "RAIOS_AGENT_END module.submit_distribution_receiver_identity_finalize" -Name "network-acquisition:receiver-identity-finalize"
$receiverFinalize = (Get-LastAgentResponseJson -Method "module.submit_distribution_receiver_identity_finalize").body.result
$receiverFinalizeIdentity = $receiverFinalize.receiver_identity
$receiverFinalizeOk = $receiverFinalize.accepted -eq $true -and $receiverFinalize.rejected -eq $false -and
    $receiverFinalize.content_sha256 -eq $expectedPayload -and [int]$receiverFinalize.retained_part_count -eq 6 -and
    $receiverFinalize.receiver_identity_complete -eq $true -and
    $receiverFinalize.guest_signature_verification_performed -eq $true -and
    $receiverFinalizeIdentity.receiver_identity_complete -eq $true -and
    $receiverFinalizeIdentity.artifact_identity_signature_verified_by_guest -eq $true -and
    $receiverFinalizeIdentity.load_descriptor_signature_verified_by_guest -eq $true -and
    $receiverFinalize.authorizes_load -eq $false -and $receiverFinalize.authorizes_install -eq $false -and
    $receiverFinalize.authorizes_execute -eq $false -and $receiverFinalize.writes_persistent_state -eq $false
Add-Predicate -Name "network-acquisition:12_receiver_identity_guest_verified" -Expected "the guest verifies all six receiver-identity payloads and marks the exact identity complete without granting effects" -Passed $receiverFinalizeOk -Actual $(if ($receiverFinalizeOk) { "guest-complete identity; authority still false" } else { $receiverFinalize | ConvertTo-Json -Compress -Depth 10 })
if (-not $receiverFinalizeOk) { throw "B1.2a guest receiver identity verification failed" }

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
    [int]$fixturePositive.host_connection -eq 1 -and [int]$fixturePositive.session -eq 2 -and
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
Add-Predicate -Name "network-acquisition:1_positive_live_fetch" -Expected "persistent host connection 1/session 2 performs the real e1000/DHCP/10.0.2.100:8443/TLS1.3-0x1301-P256 pinned-SPKI/server-Finished/exact-HTTP fetch and verifies every chunk and whole hash" -Passed $positiveOk -Actual $(if ($positiveOk) { "live fetch verified on persistent relay session 2" } else { @{ fixture = $fixturePositive; guest = $positive } | ConvertTo-Json -Compress -Depth 10 })
if (-not $positiveOk) { throw "NET-8 positive live fetch failed" }

Send-AgentCommand -Command "module.load_ephemeral svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "network-acquisition:retained-preflight"
$load = Get-LastAgentResponseJson -Method "module.load_ephemeral"
$loadRuntime = ($load.evidence | Where-Object id -eq "loader_runtime").facts
$preflight = $loadRuntime.receiver_identity_load_preflight
$preflightOk = $load.schema -eq "raios.evidence_response.v1" -and $load.decision.outcome -eq "denied" -and
    @($load.decision.grants).Count -eq 0 -and @($load.decision.effects).Count -eq 0 -and
    $preflight.status -eq "denied" -and
    $preflight.reason -eq "distribution_receiver_identity_load_preflight_missing_required_gates" -and
    $preflight.present -eq $true -and
    $preflight.receiver_identity_retained -eq $true -and
    $preflight.receiver_identity_complete -eq $true -and
    $preflight.guest_signature_verification_performed -eq $true -and
    $preflight.retained_candidate_sha256 -eq $expectedPayload -and
    $preflight.retained_candidate_present -eq $true -and
    $preflight.retained_candidate_wasm_valid -eq $true -and
    $preflight.catalog_finalize_candidate_sha256 -eq $expectedPayload -and
    $preflight.retained_candidate_matches_catalog_finalize -eq $true -and
    $preflight.preflight_evaluated -eq $true -and
    [int]$preflight.missing_gate_count -eq 4 -and
    $preflight.m6_reverification_gate_satisfied -eq $false -and
    $preflight.m7_loader_policy_gate_satisfied -eq $false -and
    $preflight.provider_trust_gate_satisfied -eq $false -and
    $preflight.owner_seal_gate_satisfied -eq $false -and
    $preflight.requires_m6_m7_reverify_for_load -eq $true -and
    $positive.candidate_load_attempted -eq $false -and
    $positive.candidate_install_attempted -eq $false -and
    $positive.candidate_execution_attempted -eq $false -and
    $positive.durable_write_attempted -eq $false -and
    $positive.rollback_mutation_attempted -eq $false -and
    $positive.provider_auto_load_attempted -eq $false
Add-Predicate -Name "network-acquisition:3_retained_candidate_preflight_denial" -Expected "the W7-finalized candidate exactly matches its guest-complete catalog receiver identity; preflight names all four still-missing M6/M7/provider/owner gates and grants no effect" -Passed $preflightOk -Actual $(if ($preflightOk) { "exact receiver/candidate binding; four gates remain closed" } else { $load | ConvertTo-Json -Compress -Depth 10 })
if (-not $preflightOk) { throw "B1.2a retained candidate did not bind to the complete receiver identity while remaining inert" }

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
# The shared wait helper samples on a 200 ms grid and cannot resolve a fast
# cancel; poll this one marker at 15 ms. $killMs is the HOST-OBSERVED bound
# (HMP + QEMU input + guest kill + UART + TCP drain), not a guest-internal
# latency; the security truths are the fact pins below, which do not depend
# on wall clock.
$killDeadline = [DateTime]::UtcNow.AddSeconds(2)
$killed = $false
do {
    Drain-SerialTcpOutput -Stream $script:SerialTcpDrainStream
    $killContent = Get-SerialLogContent -Path $SerialLog
    if ($null -ne $killContent -and [int64]$killContent.Length -gt [int64]$silentOffset -and
        $killContent.Substring([int]$silentOffset).Contains("RAIOS_W7_ACQUISITION outcome=killed")) {
        $killed = $true
        break
    }
    Start-Sleep -Milliseconds 15
} while ([DateTime]::UtcNow -lt $killDeadline)
$killMs = [int][Math]::Round(([DateTime]::UtcNow - $killSentAt).TotalMilliseconds)
Send-AgentCommand -Command $method -ExpectedMarker "RAIOS_AGENT_END $method" -Name "network-acquisition:killed-status"
$killStatus = (Get-LastAgentResponseJson -Method $method).body.result
$killOk = $killed -and $killMs -le 750 -and $killStatus.outcome -eq "killed" -and
    $killStatus.no_resume_after_kill -eq $true -and [int]$killStatus.teardown_count -eq 1 -and
    $killStatus.crypto_session_zeroized -eq $true -and $killStatus.transport_lease_held -eq $false -and
    $killStatus.pending_acquisition_present -eq $false -and $killStatus.prior_candidate_preserved -eq $true
Add-Predicate -Name "network-acquisition:4_f12_silent_peer_cleanup" -Expected "monitor F12 cancels a silent peer within 750 ms host-observed (15 ms sampling), never resumes, zeroizes crypto, completes guest/local socket and lease cleanup, drops incomplete bytes, and preserves the prior candidate" -Passed $killOk -Actual $(if ($killOk) { "killed in ${killMs}ms; cleaned once" } else { $killStatus | ConvertTo-Json -Compress -Depth 8 })
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
            [int]$fixtureTransition.session -eq 3 -and $fixtureTransition.relay_session_advanced -eq $true -and
            $fixtureTransition.reason -eq "mode_transition" -and
            $fixtureTransition.phase -eq "after_raw_relay_before_tls" -and
            [int]$fixtureTransition.drained_bytes -gt 0) { break }
    }
    Start-Sleep -Milliseconds 20
}
$transitionOk = $fixtureTransition.mode -eq "silent" -and [int]$fixtureTransition.host_connection -eq 1 -and
    [int]$fixtureTransition.session -eq 3 -and $fixtureTransition.relay_session_advanced -eq $true -and
    $fixtureTransition.reason -eq "mode_transition" -and
    $fixtureTransition.phase -eq "after_raw_relay_before_tls" -and
    [int]$fixtureTransition.drained_bytes -gt 0
Add-Predicate -Name "network-acquisition:4b_silent_relay_session_advance" -Expected "persistent host connection 1 drains silent relay session 3 and advances it on the observed mode transition before malformed acquisition starts" -Passed $transitionOk -Actual $(if ($transitionOk) { "silent relay session 3 drained and advanced" } elseif ($null -eq $fixtureTransition) { "transition result absent" } else { $fixtureTransition | ConvertTo-Json -Compress -Depth 6 })
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

# F12 opened the recovery view while cancelling the silent W7 request. Close it
# before the one physical approval below so the pointer reaches Genesis.
$recoveryCloseOffset = Get-SerialLogOffset
Send-QemuMonitorCommand -Command "sendkey f12 60" -ReplyWaitMilliseconds 0 | Out-Null
$recoveryClosed = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "GENESIS_RECOVERY_VIEW_CLOSED current_boot=true" -Offset $recoveryCloseOffset -TimeoutSeconds 2
Add-Predicate -Name "network-acquisition:13_recovery_view_closed" -Expected "F12 closes the prior recovery view before Genesis approval" -Passed $recoveryClosed -Actual $(if ($recoveryClosed) { "closed" } else { Get-SerialLogTail -Path $SerialLog })
if (-not $recoveryClosed) { throw "B1.2b could not close the Genesis recovery view" }

$moduleGrantManifestHash = "1111111111111111111111111111111111111111111111111111111111111111"
$moduleGrantArtifactHash = $expectedPayload.Substring(7)
$moduleGrantReportHash = "3333333333333333333333333333333333333333333333333333333333333333"
$moduleGrantAttestationHash = "4444444444444444444444444444444444444444444444444444444444444444"
$wrongModuleGrantArtifactHash = "2222222222222222222222222222222222222222222222222222222222222222"
$wrongModuleGrantCanonical = @(
    "canonicalization=raios.computed_capability_grant.canonical.v0",
    "schema=raios.computed_capability_grant.v0",
    "requested_capability=cap.module.load_ephemeral",
    "load_mode=ram_only",
    "subject=agent.session.serial",
    "resource=live_service_graph",
    "scope=current_boot",
    "manifest_sha256=$moduleGrantManifestHash",
    "candidate_artifact_sha256=$wrongModuleGrantArtifactHash",
    "vm_test_report_sha256=$moduleGrantReportHash",
    "local_attestation_sha256=$moduleGrantAttestationHash",
    "grants_load_now=false",
    "authorizes_guest_load=false",
    "service_inventory_change=none",
    "load_attempted=false"
) -join "`n"
$wrongModuleGrantHash = Get-TextSha256 -Text $wrongModuleGrantCanonical
$wrongM6Offset = Get-SerialLogOffset
Send-AgentCommand -Command "agent module.grant_diagnostic $wrongModuleGrantHash $moduleGrantManifestHash $wrongModuleGrantArtifactHash $moduleGrantReportHash $moduleGrantAttestationHash" -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic" -Name "network-acquisition:wrong-m6-artifact-grant"
$wrongM6GrantResponse = Get-LastAgentResponseJson -Method "module.grant_diagnostic"
$wrongM6GrantEvidence = @($wrongM6GrantResponse.evidence | Where-Object id -eq "computed_capability_grant_retained")[0]
Send-AgentCommand -Command "module.load_ephemeral svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "network-acquisition:wrong-m6-load-denied"
$wrongM6Load = Get-LastAgentResponseJson -Method "module.load_ephemeral"
Send-AgentCommand -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic" -Name "network-acquisition:wrong-m6-slot-absent"
$wrongM6Slot = Get-LastAgentResponseJson -Method "module.service_slot_diagnostic"
$wrongM6Log = (Get-SerialLogContent -Path $SerialLog).Substring([int]$wrongM6Offset)
$wrongM6Ok = $wrongM6GrantEvidence.facts.matches_current_reference -eq $true -and
    $wrongM6GrantEvidence.facts.computed_capability_grant_hash -eq "sha256:$wrongModuleGrantHash" -and
    $wrongM6GrantEvidence.facts.artifact_hash -eq "sha256:$wrongModuleGrantArtifactHash" -and
    $wrongM6Load.schema -eq "raios.evidence_response.v1" -and
    $wrongM6Load.decision.outcome -eq "denied" -and
    @($wrongM6Load.decision.grants).Count -eq 0 -and @($wrongM6Load.decision.effects).Count -eq 0 -and
    $wrongM6Load.PSObject.Properties.Name -notcontains "physical_approval_pending" -and
    $wrongM6Slot.facts.runtime.live_granted_service_slot_present -eq $false -and
    -not $wrongM6Log.Contains("GRANTED_CANDIDATE_CURRENT_BOOT_ACTIVATION") -and
    -not $wrongM6Log.Contains("WASM_GUEST_LOG echo counter=")
Add-Predicate -Name "network-acquisition:14_wrong_m6_hash_denied" -Expected "a correctly hashed and retained M6 grant for the wrong artifact remains a generic denial with no pending preview, grants, effects, slot, marker, or run" -Passed $wrongM6Ok -Actual $(if ($wrongM6Ok) { "grant=sha256:$wrongModuleGrantHash wrong_artifact=sha256:$wrongModuleGrantArtifactHash denied inert" } else { @{ grant = $wrongM6GrantResponse; load = $wrongM6Load; slot = $wrongM6Slot.facts } | ConvertTo-Json -Compress -Depth 9 })
if (-not $wrongM6Ok) { throw "B1.2b wrong-artifact M6 grant did not fail closed" }

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
Send-AgentCommand -Command "agent module.manifest_diagnostic $moduleManifestReferenceHash $moduleGrantManifestHash" -ExpectedMarker "RAIOS_AGENT_END module.manifest_diagnostic" -Name "network-acquisition:m6-manifest-reference"
$moduleManifestResponse = Get-LastAgentResponseJson -Method "module.manifest_diagnostic"
$moduleManifestEventId = [string]($moduleManifestResponse.evidence | Where-Object id -eq "module_manifest_retained").source_event_id

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
Send-AgentCommand -Command $moduleGrantCommand -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic" -Name "network-acquisition:m6-initial-grant-reference"
$moduleGrantResponse = Get-LastAgentResponseJson -Method "module.grant_diagnostic"
$moduleInitialGrantEventId = [string]($moduleGrantResponse.evidence | Where-Object id -eq "computed_capability_grant_retained").source_event_id

$moduleArtifactReferenceCanonical = @(
    "canonicalization=raios.module_candidate_artifact_reference.canonical.v0",
    "schema=raios.module_candidate_artifact_reference.v0",
    "requested_capability=cap.module.load_ephemeral",
    "load_mode=ram_only",
    "subject=agent.session.serial",
    "resource=live_service_graph",
    "scope=current_boot",
    "retained_manifest_reference_event_id=$moduleManifestEventId",
    "retained_reference_event_id=$moduleInitialGrantEventId",
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
$moduleArtifactCommand = "agent module.artifact_diagnostic $moduleArtifactReferenceHash $moduleManifestEventId $moduleInitialGrantEventId $moduleManifestReferenceHash $moduleGrantManifestHash $moduleGrantHash $moduleGrantArtifactHash $moduleGrantReportHash $moduleGrantAttestationHash"
Send-AgentCommand -Command $moduleArtifactCommand -ExpectedMarker "RAIOS_AGENT_END module.artifact_diagnostic" -Name "network-acquisition:m6-artifact-reference"
$moduleArtifactResponse = Get-LastAgentResponseJson -Method "module.artifact_diagnostic"
$moduleArtifactEventId = [string]($moduleArtifactResponse.evidence | Where-Object id -eq "candidate_artifact_retained").source_event_id

$moduleVmReportReferenceCanonical = @(
    "canonicalization=raios.module_vm_test_report_reference.canonical.v0",
    "schema=raios.module_vm_test_report_reference.v0",
    "requested_capability=cap.module.load_ephemeral",
    "load_mode=ram_only",
    "subject=agent.session.serial",
    "resource=live_service_graph",
    "scope=current_boot",
    "retained_manifest_reference_event_id=$moduleManifestEventId",
    "retained_artifact_reference_event_id=$moduleArtifactEventId",
    "retained_reference_event_id=$moduleInitialGrantEventId",
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
$moduleVmReportCommand = "agent module.vm_report_diagnostic $moduleVmReportReferenceHash $moduleManifestEventId $moduleArtifactEventId $moduleInitialGrantEventId $moduleManifestReferenceHash $moduleArtifactReferenceHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantHash $moduleGrantReportHash $moduleGrantAttestationHash"
Send-AgentCommand -Command $moduleVmReportCommand -ExpectedMarker "RAIOS_AGENT_END module.vm_report_diagnostic" -Name "network-acquisition:m6-vm-report-reference"
$moduleVmReportResponse = Get-LastAgentResponseJson -Method "module.vm_report_diagnostic"
$moduleVmReportEventId = [string]($moduleVmReportResponse.evidence | Where-Object id -eq "vm_test_report_retained").source_event_id

$moduleAttestationReferenceCanonical = @(
    "canonicalization=raios.module_local_attestation_reference.canonical.v0",
    "schema=raios.module_local_attestation_reference.v0",
    "requested_capability=cap.module.load_ephemeral",
    "load_mode=ram_only",
    "subject=agent.session.serial",
    "resource=live_service_graph",
    "scope=current_boot",
    "retained_manifest_reference_event_id=$moduleManifestEventId",
    "retained_artifact_reference_event_id=$moduleArtifactEventId",
    "retained_vm_report_reference_event_id=$moduleVmReportEventId",
    "retained_reference_event_id=$moduleInitialGrantEventId",
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
$moduleSignatureHex = New-ReliableDevPromotionSignatureHex -AttestationReferenceHash $moduleAttestationReferenceHash
$moduleAttestationCommand = "agent module.attestation_diagnostic $moduleAttestationReferenceHash $moduleManifestEventId $moduleArtifactEventId $moduleVmReportEventId $moduleInitialGrantEventId $moduleManifestReferenceHash $moduleArtifactReferenceHash $moduleVmReportReferenceHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantHash $moduleGrantReportHash $moduleGrantAttestationHash $moduleSignatureHex"
Send-AgentCommand -Command $moduleAttestationCommand -ExpectedMarker "RAIOS_AGENT_END module.attestation_diagnostic" -Name "network-acquisition:m6-signed-attestation-reference"
$moduleAttestationResponse = Get-LastAgentResponseJson -Method "module.attestation_diagnostic"
$moduleAttestationEvidence = @($moduleAttestationResponse.evidence | Where-Object id -eq "local_attestation")[0]
$moduleAttestationOk = $moduleAttestationEvidence.facts.signature_verified -eq $true -and
    $moduleAttestationEvidence.facts.status_detail -eq "local_attestation_signature_verified_load_still_denied" -and
    $null -eq $moduleAttestationEvidence.source_event_id
Add-Predicate -Name "network-acquisition:15_exact_m6_attestation" -Expected "the reliable Rust signer closes the exact manifest/artifact/report/attestation chain for the W7 candidate; the diagnostic remains non-authorizing and leaves its retained event id null" -Passed $moduleAttestationOk -Actual $(if ($moduleAttestationOk) { "reference=sha256:$moduleAttestationReferenceHash diagnostic_source_event=null" } else { $moduleAttestationResponse | ConvertTo-Json -Compress -Depth 8 })
if (-not $moduleAttestationOk) { throw "B1.2b exact M6 attestation did not verify" }

Send-AgentCommand -Command $moduleGrantCommand -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic" -Name "network-acquisition:m6-current-grant"
$moduleCurrentGrantResponse = Get-LastAgentResponseJson -Method "module.grant_diagnostic"
$moduleCurrentGrantEvidence = @($moduleCurrentGrantResponse.evidence | Where-Object id -eq "computed_capability_grant_retained")[0]
$moduleCurrentGrantEventId = [string]$moduleCurrentGrantEvidence.source_event_id
$moduleGrantReady = $moduleCurrentGrantResponse.decision.outcome -eq "granted" -and
    @($moduleCurrentGrantResponse.decision.grants) -contains "cap.module.load_ephemeral"
Add-Predicate -Name "network-acquisition:16_exact_m6_grant" -Expected "the latest current-boot M6 grant binds the exact W7 artifact and signed attestation" -Passed $moduleGrantReady -Actual $(if ($moduleGrantReady) { "event=$moduleCurrentGrantEventId grant=sha256:$moduleGrantHash" } else { $moduleCurrentGrantResponse | ConvertTo-Json -Compress -Depth 8 })
if (-not $moduleGrantReady) { throw "B1.2b exact M6 grant did not become ready" }

$activationOffset = Get-SerialLogOffset
Send-AgentCommand -Command "module.load_ephemeral svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "network-acquisition:w7-m6-physical-preview"
$activationPrepare = (Get-LastAgentResponseJson -Method "module.load_ephemeral").body.result
$sourceBinding = $activationPrepare.source_binding
$receiverBinding = $sourceBinding.receiver_identity
$grantBinding = $sourceBinding.computed_grant
$attestationBinding = $sourceBinding.local_attestation
$boundAttestationEventId = [string]$attestationBinding.event_id
$pendingBindingOk = $activationPrepare.action -eq "load_prepare" -and $activationPrepare.code -eq "ok" -and
    $activationPrepare.reason -eq "physical_approval_pending" -and
    $activationPrepare.activation_authority -eq "genesis_pointer_required" -and
    $activationPrepare.physical_approval_pending -eq $true -and
    $activationPrepare.physical_approval_consumed -eq $false -and
    $activationPrepare.approval_challenge_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    $activationPrepare.loaded -eq $false -and $activationPrepare.running -eq $false -and
    [int]$activationPrepare.run_count -eq 0 -and
    $activationPrepare.grant_authority.evidence_authorized -eq $true -and
    $activationPrepare.grant_authority.physical_authority_satisfied -eq $false -and
    $activationPrepare.grant_authority.can_load_now -eq $false -and
    $sourceBinding.present -eq $true -and $sourceBinding.source_kind -eq "w7" -and
    $sourceBinding.candidate_sha256 -eq $expectedPayload -and
    $sourceBinding.receipt_sha256 -eq $retry.receipt_sha256 -and
    $sourceBinding.w7.present -eq $true -and
    [int64]$sourceBinding.w7.invocation_id -eq [int64]$retry.invocation_id -and
    $sourceBinding.w7.candidate_sha256 -eq $expectedPayload -and
    $sourceBinding.w7.receipt_sha256 -eq $retry.receipt_sha256 -and
    $receiverBinding.present -eq $true -and $receiverBinding.content_sha256 -eq $expectedPayload -and
    $receiverBinding.retained_candidate_sha256 -eq $expectedPayload -and
    $receiverBinding.catalog_finalize_candidate_sha256 -eq $expectedPayload -and
    $receiverBinding.receiver_identity_complete -eq $true -and
    $receiverBinding.guest_signature_verification_performed -eq $true -and
    $receiverBinding.exact_candidate_catalog_binding -eq $true -and
    $grantBinding.present -eq $true -and $grantBinding.event_id -eq $moduleCurrentGrantEventId -and
    $grantBinding.computed_grant_hash -eq "sha256:$moduleGrantHash" -and
    $grantBinding.manifest_hash -eq "sha256:$moduleGrantManifestHash" -and
    $grantBinding.artifact_hash -eq $expectedPayload -and
    $grantBinding.vm_test_report_hash -eq "sha256:$moduleGrantReportHash" -and
    $grantBinding.local_attestation_hash -eq "sha256:$moduleGrantAttestationHash" -and
    $attestationBinding.present -eq $true -and $boundAttestationEventId -match '^event\.current_boot\.[0-9]{8}$' -and
    $attestationBinding.attestation_reference_hash -eq "sha256:$moduleAttestationReferenceHash" -and
    $attestationBinding.retained_manifest_reference_event_id -eq $moduleManifestEventId -and
    $attestationBinding.retained_artifact_reference_event_id -eq $moduleArtifactEventId -and
    $attestationBinding.retained_vm_report_reference_event_id -eq $moduleVmReportEventId -and
    $attestationBinding.retained_grant_reference_event_id -eq $moduleInitialGrantEventId -and
    $attestationBinding.manifest_reference_hash -eq "sha256:$moduleManifestReferenceHash" -and
    $attestationBinding.artifact_reference_hash -eq "sha256:$moduleArtifactReferenceHash" -and
    $attestationBinding.vm_report_reference_hash -eq "sha256:$moduleVmReportReferenceHash" -and
    $attestationBinding.manifest_hash -eq "sha256:$moduleGrantManifestHash" -and
    $attestationBinding.artifact_hash -eq $expectedPayload -and
    $attestationBinding.computed_grant_hash -eq "sha256:$moduleGrantHash" -and
    $attestationBinding.vm_test_report_hash -eq "sha256:$moduleGrantReportHash" -and
    $attestationBinding.local_attestation_hash -eq "sha256:$moduleGrantAttestationHash" -and
    $attestationBinding.signature_verified -eq $true
Add-Predicate -Name "network-acquisition:17_w7_receiver_m6_exact_pending_binding" -Expected "the inert Genesis preview freezes exact W7 invocation/receipt, guest-complete receiver/catalog identity, and every current M6 event/hash" -Passed $pendingBindingOk -Actual $(if ($pendingBindingOk) { "challenge=$($activationPrepare.approval_challenge_sha256)" } else { $activationPrepare | ConvertTo-Json -Compress -Depth 12 })
if (-not $pendingBindingOk) { throw "B1.2b physical preview did not bind exact W7, receiver, and M6 evidence" }

Send-AgentCommand -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic" -Name "network-acquisition:pre-click-slot-absent"
$preClickSlot = Get-LastAgentResponseJson -Method "module.service_slot_diagnostic"
Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "network-acquisition:pre-click-inventory-absent"
$preClickInventory = Get-LastAgentResponseJson -Method "service.inventory"
$preClickLog = (Get-SerialLogContent -Path $SerialLog).Substring([int]$activationOffset)
$preClickInertOk = $preClickSlot.facts.runtime.live_granted_service_slot_present -eq $false -and
    @($preClickInventory.facts.services | Where-Object { $_.id -eq "svc.dev.granted_candidate" }).Count -eq 0 -and
    $activationPrepare.service_slot_activation.active -eq $false -and
    $activationPrepare.run_evidence.present -eq $false -and
    $activationPrepare.loads_external_artifact -eq $false -and
    $activationPrepare.writes_persistent_state -eq $false -and
    $activationPrepare.durable_writes_enabled -eq $false -and
    $activationPrepare.durable_promotion_transaction.performed -eq $false -and
    $activationPrepare.durable_artifact_persist.performed -eq $false -and
    -not $preClickLog.Contains("WASM_GUEST_LOG echo counter=") -and
    -not $preClickLog.Contains("GRANTED_CANDIDATE_CURRENT_BOOT_ACTIVATION") -and
    -not $preClickLog.Contains("PROJECT_INSTALL_COMMIT") -and
    -not $preClickLog.Contains("PROJECT_APP_AUTOLOAD result=accepted")
Add-Predicate -Name "network-acquisition:18_pending_preview_zero_effect" -Expected "before the click there is no slot, inventory row, run/log/activation, durable persist, rollback, install, or autoload effect" -Passed $preClickInertOk -Actual $(if ($preClickInertOk) { "inert" } else { @{ prepare = $activationPrepare; slot = $preClickSlot.facts; inventory = $preClickInventory.facts } | ConvertTo-Json -Compress -Depth 9 })
if (-not $preClickInertOk) { throw "B1.2b pending preview was not inert" }

Send-AgentCommand -Command "service.start svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "network-acquisition:serial-start-cannot-approve"
$serialStart = (Get-LastAgentResponseJson -Method "service.start").body.result
$serialStartOk = $serialStart.code -eq "capability_denied" -and
    $serialStart.reason -eq "physical_approval_genesis_pointer_required" -and
    $serialStart.physical_approval_pending -eq $true -and
    $serialStart.physical_approval_consumed -eq $false -and
    $serialStart.loaded -eq $false -and $serialStart.running -eq $false -and
    [int]$serialStart.run_count -eq 0 -and $serialStart.run_evidence.present -eq $false -and
    $serialStart.source_binding.source_kind -eq "w7" -and
    $serialStart.source_binding.candidate_sha256 -eq $expectedPayload -and
    $serialStart.source_binding.receipt_sha256 -eq $retry.receipt_sha256
Add-Predicate -Name "network-acquisition:19_serial_start_denied" -Expected "serial service.start cannot substitute for physical approval and preserves the exact W7 binding" -Passed $serialStartOk -Actual $(if ($serialStartOk) { "denied; pending binding unchanged" } else { $serialStart | ConvertTo-Json -Compress -Depth 10 })
if (-not $serialStartOk) { throw "B1.2b serial service.start crossed the physical boundary" }

$stalePreviewChallenge = [string]$activationPrepare.approval_challenge_sha256
Send-AgentCommand -Command $moduleGrantCommand -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic" -Name "network-acquisition:advance-m6-grant-event"
$advancedGrantResponse = Get-LastAgentResponseJson -Method "module.grant_diagnostic"
$advancedGrantEventId = [string]($advancedGrantResponse.evidence | Where-Object id -eq "computed_capability_grant_retained").source_event_id
$advancedGrantOk = $advancedGrantResponse.decision.outcome -eq "granted" -and
    $advancedGrantEventId -ne $moduleCurrentGrantEventId
if (-not $advancedGrantOk) { throw "B1.2b could not advance the exact M6 grant event for stale-preview proof" }

$staleMarker = "GRANTED_CANDIDATE_CURRENT_BOOT_ACTIVATION result=denied physical_approval=genesis_pointer source_kind=w7 candidate_sha256=$expectedPayload receipt_sha256=$($retry.receipt_sha256) run_count=0 outcome=not_run durable_writes=false reason=activation_binding_stale"
Send-QemuAbsolutePointerClick -X 27017 -Y 6559
$staleDenied = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle $staleMarker -Offset $activationOffset -TimeoutSeconds $TimeoutSeconds
Send-AgentCommand -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic" -Name "network-acquisition:stale-click-slot-absent"
$staleSlot = Get-LastAgentResponseJson -Method "module.service_slot_diagnostic"
Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "network-acquisition:stale-click-inventory-absent"
$staleInventory = Get-LastAgentResponseJson -Method "service.inventory"
$afterStaleLog = (Get-SerialLogContent -Path $SerialLog).Substring([int]$activationOffset)
$staleOk = $staleDenied -and
    ([regex]::Matches($afterStaleLog, [regex]::Escape("GRANTED_CANDIDATE_CURRENT_BOOT_ACTIVATION result=denied"))).Count -eq 1 -and
    -not $afterStaleLog.Contains("GRANTED_CANDIDATE_CURRENT_BOOT_ACTIVATION result=accepted") -and
    -not $afterStaleLog.Contains("WASM_GUEST_LOG echo counter=") -and
    $staleSlot.facts.runtime.live_granted_service_slot_present -eq $false -and
    @($staleInventory.facts.services | Where-Object { $_.id -eq "svc.dev.granted_candidate" }).Count -eq 0
Add-Predicate -Name "network-acquisition:20_stale_m6_binding_click_denied" -Expected "advancing the latest exact M6 grant event makes the frozen preview stale; its click emits one denied marker and no slot, inventory, run, log, or durable effect" -Passed $staleOk -Actual $(if ($staleOk) { "old_grant=$moduleCurrentGrantEventId new_grant=$advancedGrantEventId denied inert" } else { @{ log = $afterStaleLog; slot = $staleSlot.facts; inventory = $staleInventory.facts } | ConvertTo-Json -Compress -Depth 8 })
if (-not $staleOk) { throw "B1.2b stale M6 preview did not fail closed" }

Send-AgentCommand -Command "module.load_ephemeral svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "network-acquisition:fresh-preview-after-stale"
$freshPrepare = (Get-LastAgentResponseJson -Method "module.load_ephemeral").body.result
$freshPrepareOk = $freshPrepare.action -eq "load_prepare" -and $freshPrepare.code -eq "ok" -and
    $freshPrepare.reason -eq "physical_approval_pending" -and
    $freshPrepare.physical_approval_pending -eq $true -and $freshPrepare.physical_approval_consumed -eq $false -and
    $freshPrepare.loaded -eq $false -and $freshPrepare.running -eq $false -and [int]$freshPrepare.run_count -eq 0 -and
    $freshPrepare.approval_challenge_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    $freshPrepare.approval_challenge_sha256 -ne $stalePreviewChallenge -and
    $freshPrepare.source_binding.source_kind -eq "w7" -and
    $freshPrepare.source_binding.candidate_sha256 -eq $expectedPayload -and
    $freshPrepare.source_binding.receipt_sha256 -eq $retry.receipt_sha256 -and
    $freshPrepare.source_binding.computed_grant.event_id -eq $advancedGrantEventId -and
    $freshPrepare.source_binding.computed_grant.computed_grant_hash -eq "sha256:$moduleGrantHash" -and
    $freshPrepare.source_binding.local_attestation.event_id -eq $boundAttestationEventId -and
    $freshPrepare.run_evidence.present -eq $false -and
    $freshPrepare.durable_promotion_transaction.performed -eq $false -and
    $freshPrepare.durable_artifact_persist.performed -eq $false
Add-Predicate -Name "network-acquisition:21_fresh_preview_after_stale" -Expected "a fresh prepare binds the new latest M6 grant event, resets consumed=false, and remains inert" -Passed $freshPrepareOk -Actual $(if ($freshPrepareOk) { "challenge=$($freshPrepare.approval_challenge_sha256) grant=$advancedGrantEventId" } else { $freshPrepare | ConvertTo-Json -Compress -Depth 10 })
if (-not $freshPrepareOk) { throw "B1.2b could not prepare a fresh exact preview after stale denial" }

$activationNeedle = "GRANTED_CANDIDATE_CURRENT_BOOT_ACTIVATION result=accepted physical_approval=genesis_pointer source_kind=w7 candidate_sha256=$expectedPayload receipt_sha256=$($retry.receipt_sha256) run_count=1 outcome=success durable_writes=false reason=wasm_run_success"
Send-QemuAbsolutePointerClick -X 27017 -Y 6559
$activationAccepted = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle $activationNeedle -Offset $activationOffset -TimeoutSeconds $TimeoutSeconds
$afterClickLog = (Get-SerialLogContent -Path $SerialLog).Substring([int]$activationOffset)
$activationCount = ([regex]::Matches($afterClickLog, [regex]::Escape("GRANTED_CANDIDATE_CURRENT_BOOT_ACTIVATION"))).Count
$acceptedActivationCount = ([regex]::Matches($afterClickLog, [regex]::Escape("GRANTED_CANDIDATE_CURRENT_BOOT_ACTIVATION result=accepted"))).Count
$guestLogCount = ([regex]::Matches($afterClickLog, [regex]::Escape("WASM_GUEST_LOG echo counter="))).Count
$clickOk = $activationAccepted -and $activationCount -eq 2 -and $acceptedActivationCount -eq 1 -and $guestLogCount -eq 1
Add-Predicate -Name "network-acquisition:22_one_fresh_physical_click_one_run" -Expected "after one stale denial, one fresh QMP Genesis click emits the sole accepted activation marker and sole guest log" -Passed $clickOk -Actual "accepted=$activationAccepted total_markers=$activationCount accepted_markers=$acceptedActivationCount guest_log_count=$guestLogCount"
if (-not $clickOk) { throw "B1.2b physical click did not activate exactly once" }

Send-AgentCommand -Command "service.start svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "network-acquisition:post-click-start-replay"
$postClickStart = (Get-LastAgentResponseJson -Method "service.start").body.result
Send-AgentCommand -Command "module.load_ephemeral svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "network-acquisition:post-click-load-replay"
$postClickLoad = (Get-LastAgentResponseJson -Method "module.load_ephemeral").body.result
Send-AgentCommand -Command "agent module.loader_runtime" -ExpectedMarker "RAIOS_AGENT_END module.loader_runtime" -Name "network-acquisition:post-click-loader-runtime"
$postClickLoader = Get-LastAgentResponseJson -Method "module.loader_runtime"
Send-AgentCommand -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic" -Name "network-acquisition:post-click-slot-live"
$postClickSlotLive = @((Get-LastAgentResponseJson -Method "module.service_slot_diagnostic").evidence | Where-Object id -eq "live_granted_service_slot")[0].facts
Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "network-acquisition:post-click-inventory"
$postClickInventoryResponse = Get-LastAgentResponseJson -Method "service.inventory"
$postClickInventory = @($postClickInventoryResponse.facts.services | Where-Object { $_.id -eq "svc.dev.granted_candidate" })
$afterReplayLog = (Get-SerialLogContent -Path $SerialLog).Substring([int]$activationOffset)
$runningOnceOk = $postClickStart.code -eq "capability_denied" -and
    $postClickStart.reason -eq "physical_approval_already_consumed" -and
    $postClickStart.physical_approval_pending -eq $false -and $postClickStart.physical_approval_consumed -eq $true -and
    $postClickStart.running -eq $true -and [int]$postClickStart.run_count -eq 1 -and
    $postClickStart.last_run_evidence.run_outcome -eq "success" -and
    [int64]$postClickStart.last_run_evidence.fuel_used -gt 0 -and $postClickStart.last_run_evidence.log_line_emitted -eq $true -and
    $postClickLoad.code -eq "capability_denied" -and $postClickLoad.reason -eq "service_already_loaded" -and
    $postClickLoad.physical_approval_pending -eq $false -and $postClickLoad.physical_approval_consumed -eq $true -and
    $postClickLoad.running -eq $true -and [int]$postClickLoad.run_count -eq 1 -and
    $postClickSlotLive.service_id -eq "svc.dev.granted_candidate" -and
    $postClickSlotLive.ram_only_service_slot_id -eq "ram_only:svc.dev.granted_candidate" -and
    $postClickSlotLive.service_slot_allocated -eq $true -and $postClickSlotLive.running -eq $true -and
    $postClickSlotLive.trust_tier -eq "dev_key_not_owner_sealed" -and
    $postClickSlotLive.load_mechanism -eq "wasmi_interpreter_ram_only" -and
    $postClickSlotLive.maps_executable_pages -eq $false -and
    $postClickSlotLive.durable -eq $false -and $postClickSlotLive.owner_sealed -eq $false -and
    $postClickInventory.Count -eq 1 -and $postClickInventory[0].scope -eq "current_boot" -and
    $postClickInventory[0].persistence -eq "none" -and $postClickInventory[0].running -eq $true -and
    $postClickInventory[0].health -eq "healthy" -and
    $postClickInventory[0].trust_tier -eq "dev_key_not_owner_sealed" -and
    $postClickInventory[0].ram_only_service_slot_id -eq "ram_only:svc.dev.granted_candidate" -and
    $postClickInventory[0].service_slot_activation_active -eq $true -and
    $postClickInventory[0].last_run_outcome -eq "success" -and
    ([regex]::Matches($afterReplayLog, [regex]::Escape("GRANTED_CANDIDATE_CURRENT_BOOT_ACTIVATION"))).Count -eq 2 -and
    ([regex]::Matches($afterReplayLog, [regex]::Escape("GRANTED_CANDIDATE_CURRENT_BOOT_ACTIVATION result=accepted"))).Count -eq 1 -and
    ([regex]::Matches($afterReplayLog, [regex]::Escape("WASM_GUEST_LOG echo counter="))).Count -eq 1
Add-Predicate -Name "network-acquisition:23_current_boot_run_and_replay_one_shot" -Expected "the current-boot W7 service is healthy at run_count=1 and serial load/start replays keep total markers=2, accepted=1, log=1" -Passed $runningOnceOk -Actual $(if ($runningOnceOk) { "running current_boot; total_markers=2 accepted=1 log=1" } else { @{ start = $postClickStart; load = $postClickLoad; slot = $postClickSlotLive; inventory = $postClickInventory } | ConvertTo-Json -Compress -Depth 9 })
if (-not $runningOnceOk) { throw "B1.2b current-boot activation or one-shot replay proof failed" }

# P4-3b2 ruling: the loader-runtime family renders a v1 denial envelope and the
# live granted-candidate projection is intentionally absent (it lives in the
# granted-candidate family, proven above). Native readiness must stay denied
# with zero grants even while the physically approved W7 candidate is running.
$loaderRuntimeEvidence = @($postClickLoader.evidence)
$loaderRuntimeApproval = @($postClickLoader.evidence | Where-Object id -eq "local_approval_reference")[0]
$loaderRuntimeDeniedOk = (
    $postClickLoader.schema -eq "raios.evidence_response.v1" -and
    $postClickLoader.family -eq "module.loader_runtime" -and
    $postClickLoader.scope -eq "current_boot" -and
    $postClickLoader.classification -eq "local_only" -and
    $postClickLoader.source_method -eq "module.loader_runtime" -and
    $null -eq $postClickLoader.event_id -and
    $loaderRuntimeEvidence.Count -eq 54 -and
    $loaderRuntimeEvidence[0].id -eq "manifest_reference" -and
    $loaderRuntimeEvidence[53].id -eq "executable_entrypoint_invocation_boundary" -and
    $postClickLoader.PSObject.Properties.Name -notcontains "live_granted_load_projection" -and
    @($postClickLoader.evidence | Where-Object id -eq "live_granted_load_projection").Count -eq 0 -and
    @($postClickLoader.evidence | Where-Object id -eq "manifest_reference")[0].facts.present -eq $true -and
    @($postClickLoader.evidence | Where-Object id -eq "artifact_reference")[0].facts.present -eq $true -and
    @($postClickLoader.evidence | Where-Object id -eq "vm_report_reference")[0].facts.present -eq $true -and
    @($postClickLoader.evidence | Where-Object id -eq "local_attestation_reference")[0].facts.present -eq $true -and
    $loaderRuntimeApproval.facts.present -eq $false -and
    $loaderRuntimeApproval.facts.status_detail -eq "missing" -and
    $postClickLoader.decision.outcome -eq "denied" -and
    $postClickLoader.decision.reason -eq "retained_module_local_approval_reference_missing" -and
    $postClickLoader.decision.requested_capability -eq "cap.module.load_ephemeral" -and
    @($postClickLoader.decision.grants).Count -eq 0 -and
    @($postClickLoader.decision.effects).Count -eq 0 -and
    @($postClickLoader.decision.blocked_by)[0].evidence_id -eq "local_approval_reference"
)
Add-Predicate -Name "network-acquisition:23b_loader_runtime_native_stays_denied" -Expected "loader-runtime v1 envelope stays denied with zero grants and no live_granted_load_projection while the W7 candidate runs (P4-3b2)" -Passed $loaderRuntimeDeniedOk -Actual $(if ($loaderRuntimeDeniedOk) { "denied; projection absent; evidence=54" } else { ($postClickLoader | ConvertTo-Json -Compress -Depth 8) })
if (-not $loaderRuntimeDeniedOk) { throw "B1.2b loader-runtime envelope did not deny native readiness during the granted run" }

$secondByteCommands = @($ExecutedCommands | Where-Object { $_.command -match '^module\.submit_candidate_(chunk|finalize)(\s|$)' -or $_.command -match '^module\.submit_distribution_(begin|chunk|finalize)(\s|$)' })
$postureOk = $secondByteCommands.Count -eq 0 -and
    -not $afterReplayLog.Contains("PROJECT_INSTALL_COMMIT") -and
    -not $afterReplayLog.Contains("PROJECT_APP_AUTOLOAD result=accepted") -and
    $postClickStart.writes_persistent_state -eq $false -and
    $postClickStart.durable_writes_enabled -eq $false -and
    $postClickStart.authorizes_persistent_install -eq $false -and
    $postClickStart.authorizes_rollback_install -eq $false -and
    $postClickStart.durable_promotion_transaction.performed -eq $false -and
    $postClickStart.durable_artifact_persist.performed -eq $false
Add-Predicate -Name "network-acquisition:24_one_w7_byte_path_no_durable_side_effect" -Expected "W7 is the only candidate byte route and activation performs no RECLOG/ARTSTOR/project install/provider autoload/rollback mutation" -Passed $postureOk -Actual $(if ($postureOk) { "serial_byte_commands=0 durable/install/autoload=false" } else { @{ second_byte_commands = $secondByteCommands; start = $postClickStart } | ConvertTo-Json -Compress -Depth 8 })
if (-not $postureOk) { throw "B1.2b introduced a second candidate path or durable/install effect" }
