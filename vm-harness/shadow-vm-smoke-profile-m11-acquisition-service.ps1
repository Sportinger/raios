$method = "wasm.acquisition_service_probe"
Send-AgentCommand -Command $method -ExpectedMarker "RAIOS_AGENT_END $method" -Name "m11-acquisition-service:probe"
$probe = (Get-LastAgentResponseJson -Method $method).body.result

$identityOk = $probe.schema -eq "raios.wasm_acquisition_service_probe.v0" -and
    $probe.scope -eq "current_boot" -and $probe.classification -eq "local_only" -and
    $probe.service_id -eq "svc.net.acquire.w7" -and $probe.source_policy_id -eq "local.qemu.w7" -and
    $probe.host_import_abi -eq "raios.host_imports.v1" -and
    [int]$probe.requested_import_count -eq 16 -and [int]$probe.observed_import_count -eq 16 -and
    $probe.observed_imports_exact -eq $true -and $probe.artifact_valid -eq $true -and
    $probe.signatures_build_verified -eq $true -and
    [string]$probe.artifact_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    [string]$probe.descriptor_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    [string]$probe.load_descriptor_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    [string]$probe.import_list_sha256 -match '^sha256:[0-9a-f]{64}$'
Add-Predicate -Name "m11-acquisition-service:signed_exact_artifact" -Expected "the signed svc.net.acquire.w7 artifact validates and observes exactly the descriptor-bound 16-import v1 surface" -Passed $identityOk -Actual $(if ($identityOk) { "signed exact" } else { $probe | ConvertTo-Json -Compress -Depth 8 })
if (-not $identityOk) { throw "Expected the signed exact W7 artifact and import surface" }

$tlsPositive = $probe.client_hello_vector_positive -eq $true -and $probe.tls_sequence_vector_positive -eq $true
Add-Predicate -Name "m11-acquisition-service:tls_positive_vectors" -Expected "fixed-SNI ClientHello and ordered TLS 1.3 handshake vectors pass in the shared pure logic" -Passed $tlsPositive -Actual "client_hello=$($probe.client_hello_vector_positive) sequence=$($probe.tls_sequence_vector_positive)"
if (-not $tlsPositive) { throw "Expected positive ClientHello and TLS sequencing vectors" }

$tlsNegative = [int]$probe.malformed_tls_negative_count -eq 7
Add-Predicate -Name "m11-acquisition-service:malformed_tls_negatives" -Expected "all 7 malformed record/handshake/order/signature/Finished vectors fail closed" -Passed $tlsNegative -Actual $probe.malformed_tls_negative_count
if (-not $tlsNegative) { throw "Expected all malformed TLS vectors" }

$httpVectors = $probe.http_shape_vector_positive -eq $true -and [int]$probe.malformed_http_negative_count -eq 10
Add-Predicate -Name "m11-acquisition-service:http_shape_vectors" -Expected "200 + exact octet-stream Content-Length passes; redirect, missing/duplicate/malformed length, transfer/content encoding, wrong type, wrong length, folded header, and oversized header fail" -Passed $httpVectors -Actual "positive=$($probe.http_shape_vector_positive) negatives=$($probe.malformed_http_negative_count)"
if (-not $httpVectors) { throw "Expected positive and malformed W7 HTTP-shape vectors" }

$chunkOk = $probe.chunk_geometry_vector_positive -eq $true
Add-Predicate -Name "m11-acquisition-service:canonical_chunk_geometry" -Expected "the body driver uses at most four canonical 64-KiB chunks with an exact final tail" -Passed $chunkOk -Actual $probe.chunk_geometry_vector_positive
if (-not $chunkOk) { throw "Expected canonical W7 chunk geometry" }

$cleanupOk = $probe.guest_trap_cleanup -eq $true -and $probe.out_of_fuel_cleanup -eq $true
Add-Predicate -Name "m11-acquisition-service:fuel_trap_cleanup" -Expected "the reused NET-2R lifecycle tears down exactly once after guest trap and Wasm OutOfFuel" -Passed $cleanupOk -Actual "trap=$($probe.guest_trap_cleanup) fuel=$($probe.out_of_fuel_cleanup)"
if (-not $cleanupOk) { throw "Expected trap and fuel cleanup through NET-2R" }

# NET-8 arming (owner decision 2026-07-14): the exact signed W7 artifact is now
# owner-authorized, so this probe reports the ARMED reality instead of the pre-arming
# denial. policy_allows_beyond_env is true ONLY for this exact artifact+import-list+source
# conjunction (net_8_w7_policy_allows); the mismatch cases below still deny.
$armedOk = $probe.policy_allows_beyond_env -eq $true -and
    $probe.production_linker_armed -eq $true -and
    $probe.artifact_valid -eq $true -and $probe.signatures_build_verified -eq $true
Add-Predicate -Name "m11-acquisition-service:armed_for_exact_w7" -Expected "the exact signed W7 artifact is owner-armed (policy_allows_beyond_env true, production linker armed) after the NET-8 flip" -Passed $armedOk -Actual "policy=$($probe.policy_allows_beyond_env) armed=$($probe.production_linker_armed) valid=$($probe.artifact_valid)"
if (-not $armedOk) { throw "Expected the exact signed W7 artifact to be owner-armed after NET-8" }

# The mismatch guards must still keep the flip false: a wrong artifact hash, a wrong
# import-list hash, or a wrong source policy each deny even after arming.
$mismatchOk = $probe.artifact_hash_mismatch_denied -eq $true -and
    $probe.import_list_hash_mismatch_denied -eq $true -and
    $probe.source_policy_mismatch_denied -eq $true
Add-Predicate -Name "m11-acquisition-service:mismatch_still_denies" -Expected "artifact/import-list/source-policy mismatches independently keep the W7 flip false after arming" -Passed $mismatchOk -Actual "artifact=$($probe.artifact_hash_mismatch_denied) list=$($probe.import_list_hash_mismatch_denied) source=$($probe.source_policy_mismatch_denied)"
if (-not $mismatchOk) { throw "Expected artifact/import/source mismatches to keep W7 denied" }

# Armed to RUN, but still grants nothing DURABLE: no load, execute, install, durable
# write, or owner seal. Arming opened the linker; it did not open persistence.
$grantsNothingDurable = $probe.candidate_load_attempted -eq $false -and
    $probe.candidate_execution_attempted -eq $false -and
    $probe.candidate_install_attempted -eq $false -and $probe.durable_write_attempted -eq $false -and
    $probe.owner_sealed -eq $false -and $probe.trust_tier -eq "dev_key_not_owner_sealed" -and
    $probe.evidence_complete -eq $true -and
    $probe.all_fixture_vectors_positive -eq $true
Add-Predicate -Name "m11-acquisition-service:grants_nothing_durable" -Expected "even armed, W7 opens no load, execution, install, durable-write, or owner-seal authority" -Passed $grantsNothingDurable -Actual $(if ($grantsNothingDurable) { "grants nothing durable" } else { $probe | ConvertTo-Json -Compress -Depth 8 })
if (-not $grantsNothingDurable) { throw "Expected armed W7 to grant nothing durable (no load/execute/install/durable/owner-seal)" }
