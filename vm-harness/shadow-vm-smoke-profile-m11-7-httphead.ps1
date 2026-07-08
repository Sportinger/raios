Send-AgentCommand -Command "wasm.httphead_probe" -ExpectedMarker "RAIOS_AGENT_END wasm.httphead_probe" -Name "m11-7-httphead:probe"
$httphead = Get-LastAgentResponseJson -Method "wasm.httphead_probe"
$result = $httphead.body.result

$positiveOk = (
    $result.schema -eq "raios.wasm_httphead_probe.v0" -and
    $result.service_id -eq "svc.demo.httphead" -and
    $result.run_outcome -eq "success" -and
    [int]$result.authorized_import_count -eq 3 -and
    [int]$result.linked_host_import_count -eq 3 -and
    $result.module_imports_within_authorized_list -eq $true -and
    $result.input_sha256 -eq "sha256:f49383a81b679f0569a28430340963e08c898b581b9f910be129ccfe60e54f9a" -and
    [int]$result.captured_output_len -eq 16 -and
    $result.guest_record_valid -eq $true -and
    $result.guest_status_present -eq $true -and
    [int]$result.guest_status_code -eq 200 -and
    $result.guest_content_length_present -eq $false -and
    $result.guest_response_complete -eq $true -and
    $result.guest_chunked -eq $true -and
    $result.core_status_present -eq $true -and
    [int]$result.core_status_code -eq 200 -and
    $result.core_content_length_present -eq $false -and
    $result.core_response_complete -eq $true -and
    $result.core_chunked -eq $true -and
    $result.guest_matches_core -eq $true -and
    $result.output_bytes_match -eq $true -and
    $result.captured_output_sha256 -eq $result.core_output_sha256 -and
    [int64]$result.fuel_used -gt 0 -and
    [int64]$result.fuel_used -lt [int64]$result.fuel_budget
)
Add-Predicate -Name "m11-7-httphead:positive-crosscheck" -Expected "svc.demo.httphead parses the chunked response and matches the independent core parser byte-for-byte" -Passed $positiveOk -Actual $(if ($positiveOk) { "matched" } else { ($result | ConvertTo-Json -Compress -Depth 8) })
if (-not $positiveOk) { throw "Expected svc.demo.httphead positive cross-check to match core parse and output SHA" }

$malformed = $result.malformed_case
$malformedOk = (
    $malformed.guest_record_valid -eq $true -and
    $malformed.guest_status_present -eq $true -and
    [int]$malformed.guest_status_code -eq 200 -and
    $malformed.guest_content_length_present -eq $false -and
    $malformed.guest_response_complete -eq $false -and
    $malformed.guest_chunked -eq $true -and
    $malformed.core_response_complete -eq $false -and
    $malformed.guest_matches_core -eq $true -and
    $malformed.capability_granted -eq $false
)
Add-Predicate -Name "m11-7-httphead:malformed-truncated-agree" -Expected "truncated chunked stream: guest and core agree it is NOT complete (response_complete 1->0), chunked stays 1" -Passed $malformedOk -Actual $(if ($malformedOk) { "matched" } else { ($malformed | ConvertTo-Json -Compress -Depth 6) })
if (-not $malformedOk) { throw "Expected svc.demo.httphead malformed case to agree with core truncated (not complete)" }

$contentLength = $result.content_length_case
$contentLengthOk = (
    $contentLength.guest_record_valid -eq $true -and
    $contentLength.guest_status_present -eq $true -and
    [int]$contentLength.guest_status_code -eq 200 -and
    $contentLength.guest_content_length_present -eq $false -and
    $contentLength.guest_response_complete -eq $false -and
    $contentLength.guest_chunked -eq $false -and
    $contentLength.core_content_length_present -eq $false -and
    $contentLength.guest_matches_core -eq $true -and
    $contentLength.capability_granted -eq $false
)
Add-Predicate -Name "m11-7-httphead:content-length-short-circuit-evidence" -Expected "guest and core BOTH report content_length_present=false for a response literally containing Content-Length: 11 (preserved status-line short-circuit)" -Passed $contentLengthOk -Actual $(if ($contentLengthOk) { "matched" } else { ($contentLength | ConvertTo-Json -Compress -Depth 6) })
if (-not $contentLengthOk) { throw "Expected svc.demo.httphead content-length case to record the preserved short-circuit as typed evidence" }

$negative = $result.negative
$negativeOk = (
    $negative.module_imports_within_authorized_list -eq $false -and
    $negative.run_outcome -eq "module_import_not_authorized" -and
    $negative.missing_import_module -eq "env" -and
    $negative.instantiation_ok -eq $false -and
    [int]$negative.captured_output_len -eq 0
)
Add-Predicate -Name "m11-7-httphead:refused-unless-granted" -Expected "svc.demo.httphead refuses the partial import grant before instantiation" -Passed $negativeOk -Actual $(if ($negativeOk) { "matched" } else { ($negative | ConvertTo-Json -Compress -Depth 6) })
if (-not $negativeOk) { throw "Expected svc.demo.httphead subset denial to fail closed before instantiation" }

$grantsNothingOk = (
    $result.guest_output_is_evidence_only -eq $true -and
    $result.core_is_authority -eq $true -and
    $result.policy_allows_beyond_env -eq $false -and
    $result.authorizes_provider_request -eq $false -and
    $result.authorizes_provider_export -eq $false -and
    $result.durable_write -eq $false -and
    $result.capability_granted -eq $false -and
    $result.owner_sealed -eq $false -and
    $result.trust_tier -eq "dev_key_not_owner_sealed"
)
Add-Predicate -Name "m11-7-httphead:grants-nothing" -Expected "httphead output is evidence only and every trust/authority/durable flag remains false" -Passed $grantsNothingOk -Actual $(if ($grantsNothingOk) { "matched" } else { ($result | ConvertTo-Json -Compress -Depth 8) })
if (-not $grantsNothingOk) { throw "Expected svc.demo.httphead to grant no trust, provider, durable, or capability authority" }
