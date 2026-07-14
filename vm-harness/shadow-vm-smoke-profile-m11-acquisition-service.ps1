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

$denialOk = $probe.policy_denial_reason -eq "import_beyond_env_not_owner_authorized" -and
    $probe.denied_before_instantiation -eq $true -and $probe.instantiation_attempted -eq $false -and
    $probe.policy_allows_beyond_env -eq $false
Add-Predicate -Name "m11-acquisition-service:denied_before_instantiation" -Expected "the exact signed artifact denies before instantiation with import_beyond_env_not_owner_authorized" -Passed $denialOk -Actual "reason=$($probe.policy_denial_reason) denied=$($probe.denied_before_instantiation) instantiated=$($probe.instantiation_attempted)"
if (-not $denialOk) { throw "Expected the explicit beyond-env owner-authorization denial before instantiation" }

$grantsNothing = $probe.production_linker_armed -eq $false -and $probe.network_effect -eq $false -and
    $probe.crypto_effect -eq $false -and $probe.acquisition_effect -eq $false -and
    $probe.candidate_load_attempted -eq $false -and $probe.candidate_execution_attempted -eq $false -and
    $probe.candidate_install_attempted -eq $false -and $probe.durable_write_attempted -eq $false -and
    $probe.owner_sealed -eq $false -and $probe.trust_tier -eq "dev_key_not_owner_sealed" -and
    $probe.capability_granted -eq $false -and $probe.evidence_complete -eq $true -and
    $probe.all_fixture_vectors_positive -eq $true
Add-Predicate -Name "m11-acquisition-service:grants_nothing" -Expected "NET-7 opens no linker, network, crypto, acquisition, load, execution, install, durable, owner-seal, or capability effect" -Passed $grantsNothing -Actual $(if ($grantsNothing) { "grants nothing" } else { $probe | ConvertTo-Json -Compress -Depth 8 })
if (-not $grantsNothing) { throw "Expected NET-7 to grant and effect nothing" }
