if (-not $Network) {
    throw "m11-crypto-imports requires -Network (real fixture-owned TCP lease)"
}

$method = "wasm.crypto_import_probe"
$fixtureOffset = Get-SerialLogOffset
Send-AgentCommand -Command $method -ExpectedMarker "RAIOS_AGENT_END $method" -Name "m11-crypto-imports:probe"
$probe = (Get-LastAgentResponseJson -Method $method).body.result

$primaryFinished = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_CRYPTO_SHIM outcome=finished scenario=positive_and_negatives teardown_complete=true" -Offset $fixtureOffset -TimeoutSeconds 8
if (-not $primaryFinished) { throw "Expected the in-guest crypto positive/negative fixture to finish" }

$oobOffset = Get-SerialLogOffset
Send-AgentCommand -Command $method -ExpectedMarker "RAIOS_AGENT_END $method" -Name "m11-crypto-imports:start_oob"
$oobFinished = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_CRYPTO_SHIM outcome=host_error scenario=guest_memory_fault teardown_complete=true" -Offset $oobOffset -TimeoutSeconds 8
if (-not $oobFinished) { throw "Expected the in-guest memory-fault fixture to trap without crashing the VM" }

$foreignOffset = Get-SerialLogOffset
Send-AgentCommand -Command $method -ExpectedMarker "RAIOS_AGENT_END $method" -Name "m11-crypto-imports:start_foreign"
$foreignFinished = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle "RAIOS_CRYPTO_SHIM outcome=finished scenario=foreign_session teardown_complete=true" -Offset $foreignOffset -TimeoutSeconds 8
if (-not $foreignFinished) { throw "Expected the fresh-invocation foreign-handle fixture to finish" }

Send-AgentCommand -Command $method -ExpectedMarker "RAIOS_AGENT_END $method" -Name "m11-crypto-imports:fixture_complete"
$probe = (Get-LastAgentResponseJson -Method $method).body.result

$schemaOk = $probe.schema -eq "raios.wasm_crypto_import_probe.v0" -and $probe.scope -eq "current_boot" -and $probe.classification -eq "local_only" -and $probe.test_infrastructure -eq $true
Add-Predicate -Name "m11-crypto-imports:typed_probe" -Expected "the read-only local probe reports the fixed current-boot crypto surface" -Passed $schemaOk -Actual $(if ($schemaOk) { "typed" } else { $probe | ConvertTo-Json -Compress -Depth 8 })
if (-not $schemaOk) { throw "Expected the typed crypto-import probe" }

$outcomes = @($probe.per_call_outcomes)
$outcomesOk = $probe.all_eight_host_vectors_positive -eq $true -and $outcomes.Count -eq 8 -and ($outcomes -join ",") -eq "session_handle_and_public_bytes,digest32,math_valid,handshake_keys_opaque,application_keys_opaque,server_verified_client_proof,ciphertext,plaintext"
Add-Predicate -Name "m11-crypto-imports:eight_positive_outcomes" -Expected "all eight fixed imports have an observed positive host-vector outcome" -Passed $outcomesOk -Actual ($outcomes -join ",")
if (-not $outcomesOk) { throw "Expected all eight fixed crypto host vectors" }

$rfcOk = $probe.rfc8448_key_schedule_valid -eq $true
Add-Predicate -Name "m11-crypto-imports:rfc8448_schedule" -Expected "the TLS 1.3 HKDF schedule matches the RFC 8448 traffic-key vector" -Passed $rfcOk -Actual $probe.rfc8448_key_schedule_valid
if (-not $rfcOk) { throw "Expected the RFC 8448 key-schedule vector" }

$recordOk = $probe.record_framing_valid -eq $true
Add-Predicate -Name "m11-crypto-imports:record_framing" -Expected "fixed TLS 1.3 record framing seals and opens through AES-128-GCM" -Passed $recordOk -Actual $probe.record_framing_valid
if (-not $recordOk) { throw "Expected TLS record framing vectors" }

$stateOk = $probe.state_machine_valid -eq $true -and $probe.final_state -eq "application_traffic" -and [int]$probe.transition_count -ge 5
Add-Predicate -Name "m11-crypto-imports:state_transitions" -Expected "the opaque session reaches application traffic only through the fixed transitions" -Passed $stateOk -Actual "state=$($probe.final_state) transitions=$($probe.transition_count)"
if (-not $stateOk) { throw "Expected fixed TLS session transitions" }

$sequenceOk = $probe.sequence_rules_valid -eq $true -and [int64]$probe.send_sequence -eq 1 -and [int64]$probe.receive_sequence -eq 1
Add-Predicate -Name "m11-crypto-imports:sequence_success" -Expected "send and receive counters advance only for successful application records" -Passed $sequenceOk -Actual "send=$($probe.send_sequence) receive=$($probe.receive_sequence)"
if (-not $sequenceOk) { throw "Expected successful sequence increments" }

$tagOk = $probe.tag_failure_preserved_receive_sequence -eq $true
Add-Predicate -Name "m11-crypto-imports:tag_failure_sequence" -Expected "a failed GCM tag leaves the receive sequence unchanged" -Passed $tagOk -Actual $probe.tag_failure_preserved_receive_sequence
if (-not $tagOk) { throw "Expected tag failure to preserve sequence" }

$ownershipOk = $probe.foreign_stale_state_size_negatives -eq $true
Add-Predicate -Name "m11-crypto-imports:foreign_stale_state_size" -Expected "duplicate, foreign, stale, state and size vectors all fail closed" -Passed $ownershipOk -Actual $probe.foreign_stale_state_size_negatives
if (-not $ownershipOk) { throw "Expected ownership/state/size negatives" }

$coreNegativesOk = $probe.core_negative_vectors_positive -eq $true
Add-Predicate -Name "m11-crypto-imports:core_negative_vectors" -Expected "peer key, transcript, header, length, record and tag negatives are observed" -Passed $coreNegativesOk -Actual $probe.core_negative_vectors_positive
if (-not $coreNegativesOk) { throw "Expected core crypto negative vectors" }

$inGuestCalls = @($probe.in_guest_call_evidence)
$positiveImports = @(
    "crypto.tls13_session_open", "crypto.sha256", "crypto.tls13_handshake_keys",
    "crypto.p256_verify", "crypto.tls13_application_keys", "crypto.tls13_finished",
    "crypto.tls13_aead_seal", "crypto.tls13_aead_open"
)
$positiveImportsObserved = @($positiveImports | Where-Object {
    $import = $_
    -not @($inGuestCalls | Where-Object { $_.import -eq $import -and $_.outcome -eq "ok" }).Count
}).Count -eq 0
$positiveShape = @($inGuestCalls | Where-Object { $_.import -eq "crypto.tls13_finished" -and $_.outcome -eq "ok" }).Count -eq 2 -and
    @($inGuestCalls | Where-Object { $_.import -eq "crypto.tls13_application_keys" -and $_.outcome -eq "ok" }).Count -eq 1 -and
    @($inGuestCalls | Where-Object { $_.import -eq "crypto.tls13_aead_seal" -and $_.outcome -eq "ok" }).Count -eq 1 -and
    @($inGuestCalls | Where-Object { $_.import -eq "crypto.tls13_aead_open" -and $_.outcome -eq "ok" }).Count -eq 1
$inGuestPositive = $probe.fixture_complete -eq $true -and
    $probe.in_guest_positive_sequence_complete -eq $true -and
    $probe.sealed_opened_roundtrip -eq $true -and
    $probe.fixture_math_valid -eq $true -and
    $probe.fixture_pin_match -eq $true -and
    $probe.fixture_server_finished_valid -eq $true -and
    $probe.fixture_session_zeroized -eq $true -and
    $positiveImportsObserved -and $positiveShape
Add-Predicate -Name "m11-crypto-imports:in_guest_positive_sequence" -Expected "the labeled Wasm fixture calls all eight shims and guest-checks the sealed/plaintext-opened roundtrip" -Passed $inGuestPositive -Actual $(if ($inGuestPositive) { "eight imports; roundtrip equal" } else { $probe | ConvertTo-Json -Compress -Depth 10 })
if (-not $inGuestPositive) { throw "Expected the full in-guest TLS-shaped crypto sequence" }

$foreignOk = @($inGuestCalls | Where-Object { $_.import -eq "crypto.sha256" -and $_.denial -eq "crypto_foreign_session" }).Count -eq 1
Add-Predicate -Name "m11-crypto-imports:guest_foreign_session" -Expected "a prior-invocation/fabricated session handle receives crypto_foreign_session" -Passed $foreignOk -Actual (@($inGuestCalls | Where-Object denial -eq "crypto_foreign_session") | ConvertTo-Json -Compress)
if (-not $foreignOk) { throw "Expected the kernel shim foreign-session denial" }

$staleOk = @($inGuestCalls | Where-Object { $_.import -eq "crypto.sha256" -and $_.denial -eq "crypto_stale_session" }).Count -eq 1
Add-Predicate -Name "m11-crypto-imports:guest_stale_session" -Expected "net.tcp_close closes the opaque slot and the old handle receives crypto_stale_session" -Passed $staleOk -Actual (@($inGuestCalls | Where-Object denial -eq "crypto_stale_session") | ConvertTo-Json -Compress)
if (-not $staleOk) { throw "Expected the kernel shim stale-session denial" }

$oversizeOk = @($inGuestCalls | Where-Object { $_.import -eq "crypto.sha256" -and $_.denial -eq "crypto_hash_input_too_large" }).Count -eq 1
Add-Predicate -Name "m11-crypto-imports:guest_hash_oversize" -Expected "a 16385-byte hash request receives crypto_hash_input_too_large" -Passed $oversizeOk -Actual (@($inGuestCalls | Where-Object denial -eq "crypto_hash_input_too_large") | ConvertTo-Json -Compress)
if (-not $oversizeOk) { throw "Expected the kernel shim hash-size denial" }

$smallOutputOk = @($inGuestCalls | Where-Object { $_.import -eq "crypto.tls13_aead_seal" -and $_.denial -eq "crypto_output_too_small" }).Count -eq 1
Add-Predicate -Name "m11-crypto-imports:guest_seal_output_small" -Expected "a 41-byte buffer for a 42-byte sealed record receives crypto_output_too_small" -Passed $smallOutputOk -Actual (@($inGuestCalls | Where-Object denial -eq "crypto_output_too_small") | ConvertTo-Json -Compress)
if (-not $smallOutputOk) { throw "Expected the kernel shim seal-capacity denial" }

$memoryFaultOk = @($inGuestCalls | Where-Object { $_.import -eq "crypto.sha256" -and $_.denial -eq "crypto_guest_memory_fault" }).Count -eq 1
Add-Predicate -Name "m11-crypto-imports:guest_memory_fault" -Expected "an input range past the guest memory end receives crypto_guest_memory_fault and only traps its fixture" -Passed $memoryFaultOk -Actual (@($inGuestCalls | Where-Object denial -eq "crypto_guest_memory_fault") | ConvertTo-Json -Compress)
if (-not $memoryFaultOk) { throw "Expected the kernel shim guest-memory denial" }

$invalidModeOk = @($inGuestCalls | Where-Object { $_.import -eq "crypto.tls13_finished" -and $_.denial -eq "crypto_invalid_finished_mode" }).Count -eq 1
Add-Predicate -Name "m11-crypto-imports:guest_finished_mode" -Expected "Finished mode 2 receives crypto_invalid_finished_mode" -Passed $invalidModeOk -Actual (@($inGuestCalls | Where-Object denial -eq "crypto_invalid_finished_mode") | ConvertTo-Json -Compress)
if (-not $invalidModeOk) { throw "Expected the kernel shim Finished-mode denial" }

$sequenceOkGuest = @($inGuestCalls | Where-Object { $_.import -eq "crypto.tls13_aead_seal" -and $_.denial -eq "crypto_sequence_exhausted" }).Count -eq 1
Add-Predicate -Name "m11-crypto-imports:guest_sequence_exhausted" -Expected "the fixture-only bound assertion maps the core-proven u64::MAX seal limit to crypto_sequence_exhausted" -Passed $sequenceOkGuest -Actual (@($inGuestCalls | Where-Object denial -eq "crypto_sequence_exhausted") | ConvertTo-Json -Compress)
if (-not $sequenceOkGuest) { throw "Expected the kernel shim sequence-exhaustion denial" }

$selfBlessOk = @($inGuestCalls | Where-Object { $_.import -eq "crypto.tls13_application_keys" -and $_.denial -eq "crypto_guest_trust_self_assertion_denied" }).Count -eq 1
Add-Predicate -Name "m11-crypto-imports:guest_self_bless_denied" -Expected "application_keys before positive server-Finished evidence receives crypto_guest_trust_self_assertion_denied" -Passed $selfBlessOk -Actual (@($inGuestCalls | Where-Object denial -eq "crypto_guest_trust_self_assertion_denied") | ConvertTo-Json -Compress)
if (-not $selfBlessOk) { throw "Expected the critical guest self-bless denial" }

$denials = @($probe.typed_denials)
$distinctDenials = @($denials | Sort-Object -Unique)
$denialsDistinct = $denials.Count -eq 35 -and $distinctDenials.Count -eq 35
Add-Predicate -Name "m11-crypto-imports:typed_denials_distinct" -Expected "all 35 refusal reasons are pairwise-distinct typed denials" -Passed $denialsDistinct -Actual "total=$($denials.Count) unique=$($distinctDenials.Count)"
if (-not $denialsDistinct) { throw "Expected pairwise-distinct crypto denial reasons" }

$sessionDenials = @("crypto_foreign_tcp_owner", "crypto_entropy_not_ready", "crypto_duplicate_session", "crypto_wrong_public_output_size", "crypto_guest_memory_fault")
$sessionDenialsOk = @($sessionDenials | Where-Object { $_ -notin $denials }).Count -eq 0
Add-Predicate -Name "m11-crypto-imports:session_open_denials" -Expected "session-open ownership, entropy, duplicate, size and memory refusals stay distinct" -Passed $sessionDenialsOk -Actual ($sessionDenials -join ",")
if (-not $sessionDenialsOk) { throw "Expected all session-open denial reasons" }

$verifyDenials = @("crypto_unsupported_spki_encoding", "crypto_unsupported_signature_encoding", "crypto_verify_input_out_of_bounds", "crypto_source_pin_absent", "crypto_certificate_verify_math_invalid", "crypto_source_pin_mismatch")
$verifyDenialsOk = @($verifyDenials | Where-Object { $_ -notin $denials }).Count -eq 0
Add-Predicate -Name "m11-crypto-imports:verify_denials" -Expected "P-256 encoding, bounds, math and pin failures remain separate" -Passed $verifyDenialsOk -Actual ($verifyDenials -join ",")
if (-not $verifyDenialsOk) { throw "Expected all verification denial reasons" }

$transitionDenials = @("crypto_wrong_peer_key_encoding", "crypto_repeated_or_out_of_order_transition", "crypto_zero_transcript_hash", "crypto_guest_trust_self_assertion_denied", "crypto_certificate_verify_evidence_missing", "crypto_server_finished_evidence_missing", "crypto_server_finished_evidence_invalid", "crypto_invalid_finished_mode", "crypto_wrong_finished_proof_size", "crypto_finished_wrong_state")
$transitionDenialsOk = @($transitionDenials | Where-Object { $_ -notin $denials }).Count -eq 0
Add-Predicate -Name "m11-crypto-imports:transition_denials" -Expected "handshake/application/Finished refusals are distinct and guest trust cannot self-authorize" -Passed $transitionDenialsOk -Actual ($transitionDenials -join ",")
if (-not $transitionDenialsOk) { throw "Expected all transition denial reasons" }

$aeadDenials = @("crypto_invalid_record_header", "crypto_record_length_mismatch", "crypto_record_too_large", "crypto_sequence_exhausted", "crypto_output_too_small", "crypto_tag_authentication_failed", "crypto_aead_wrong_state")
$aeadDenialsOk = @($aeadDenials | Where-Object { $_ -notin $denials }).Count -eq 0
Add-Predicate -Name "m11-crypto-imports:aead_denials" -Expected "header, length, bound, sequence, output, tag and state AEAD refusals stay distinct" -Passed $aeadDenialsOk -Actual ($aeadDenials -join ",")
if (-not $aeadDenialsOk) { throw "Expected all AEAD denial reasons" }

$trustFactsOk = $probe.math_valid -eq $true -and $probe.pin_match -eq $true -and $probe.server_finished_valid -eq $true -and $probe.trust_label_granted_by_guest -eq $false
Add-Predicate -Name "m11-crypto-imports:math_pin_separate" -Expected "math_valid and pin_match are separate positive facts while no guest call grants a trust label" -Passed $trustFactsOk -Actual "math=$($probe.math_valid) pin=$($probe.pin_match) finished=$($probe.server_finished_valid) guest_trust=$($probe.trust_label_granted_by_guest)"
if (-not $trustFactsOk) { throw "Expected separate math/pin/Finished facts" }

$hashes = @($probe.public_output_sha256, $probe.hello_transcript_sha256, $probe.application_transcript_sha256, $probe.spki_sha256, $probe.verify_message_sha256, $probe.signature_sha256)
$hashesOk = @($hashes | Where-Object { $_ -notmatch '^sha256:[0-9a-f]{64}$' -or $_ -eq ('sha256:' + ('0' * 64)) }).Count -eq 0
Add-Predicate -Name "m11-crypto-imports:public_evidence_hashes" -Expected "only nonzero hashes of public output/transcripts/SPKI/message/signature are exposed" -Passed $hashesOk -Actual ($hashes -join ",")
if (-not $hashesOk) { throw "Expected bounded public evidence hashes" }

$grantOk = $probe.policy_denial_reason -eq "import_beyond_env_not_owner_authorized" -and $probe.denied_before_instantiation -eq $true -and [int]$probe.requested_crypto_import_count -eq 8
Add-Predicate -Name "m11-crypto-imports:denied_before_instantiation" -Expected "the signed eight-import artifact denies before instantiation with import_beyond_env_not_owner_authorized" -Passed $grantOk -Actual "reason=$($probe.policy_denial_reason) denied=$($probe.denied_before_instantiation)"
if (-not $grantOk) { throw "Expected ungranted crypto artifact denial" }

$immediateOk = $probe.crypto_calls_suspend -eq $false -and [int]$probe.synchronous_wall_limit_ms -eq 25 -and [int]$probe.host_vector_wall_ms -le 25
Add-Predicate -Name "m11-crypto-imports:immediate_not_suspending" -Expected "all fixed crypto calls complete synchronously under the 25 ms host-call ceiling" -Passed $immediateOk -Actual "suspend=$($probe.crypto_calls_suspend) limit=$($probe.synchronous_wall_limit_ms)"
if (-not $immediateOk) { throw "Expected immediate crypto calls" }

$secretsOk = [int]$probe.private_material_guest_writes -eq 0 -and $probe.opaque_session_zeroized_after_vectors -eq $true
Add-Predicate -Name "m11-crypto-imports:opaque_secret_custody" -Expected "no private material is written to guest memory and the core-owned session zeroizes" -Passed $secretsOk -Actual "guest_writes=$($probe.private_material_guest_writes) zeroized=$($probe.opaque_session_zeroized_after_vectors)"
if (-not $secretsOk) { throw "Expected opaque key custody and zeroization" }

$postureOk = $probe.policy_allows_beyond_env -eq $false -and $probe.production_linker_armed -eq $false -and [int]$probe.production_crypto_shim_call_count -eq 0 -and $probe.owner_sealed -eq $false -and $probe.authority_tier -eq "dev_key_not_owner_sealed" -and $probe.capability_granted -eq $false -and $probe.durable_effect -eq $false
Add-Predicate -Name "m11-crypto-imports:grants_nothing" -Expected "policy remains false, production linker stays unarmed, and dev-key evidence grants no capability or durable effect" -Passed $postureOk -Actual $(if ($postureOk) { "grants nothing" } else { $probe | ConvertTo-Json -Compress -Depth 8 })
if (-not $postureOk) { throw "Expected NET-5 to grant nothing" }
