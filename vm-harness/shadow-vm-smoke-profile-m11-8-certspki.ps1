Send-AgentCommand -Command "wasm.certspki_probe" -ExpectedMarker "RAIOS_AGENT_END wasm.certspki_probe" -Name "m11-8-certspki:probe"
$certspki = Get-LastAgentResponseJson -Method "wasm.certspki_probe"
$result = $certspki.body.result

$positiveOk = (
    $result.schema -eq "raios.wasm_certspki_probe.v0" -and
    $result.service_id -eq "svc.demo.certspki" -and
    $result.run_outcome -eq "success" -and
    [int]$result.authorized_import_count -eq 3 -and
    [int]$result.linked_host_import_count -eq 3 -and
    $result.module_imports_within_authorized_list -eq $true -and
    $result.input_sha256 -eq "sha256:baa2a6c3263fb8170aa2b4013046414a1e4760c2f5e7bfdf88c74f51742e0cb4" -and
    [int]$result.captured_output_len -eq 71 -and
    $result.guest_record_valid -eq $true -and
    $result.guest_parse_ok -eq $true -and
    $result.core_parse_ok -eq $true -and
    [int]$result.guest_spki_der_len -eq 91 -and
    [int]$result.core_spki_der_len -eq 91 -and
    [int]$result.guest_public_key_len -eq 65 -and
    [int]$result.core_public_key_len -eq 65 -and
    $result.guest_public_key_sha256 -eq "sha256:2a6dabe8ff66cfb03ab8f59d1bf7b450da7146abe5e441c4171ce1e78c2e5962" -and
    $result.core_public_key_sha256 -eq "sha256:2a6dabe8ff66cfb03ab8f59d1bf7b450da7146abe5e441c4171ce1e78c2e5962" -and
    $result.guest_matches_core -eq $true -and
    $result.output_bytes_match -eq $true -and
    $result.captured_output_sha256 -eq $result.core_output_sha256 -and
    [int64]$result.fuel_used -gt 0 -and
    [int64]$result.fuel_used -lt [int64]$result.fuel_budget
)
Add-Predicate -Name "m11-8-certspki:positive-crosscheck" -Expected "svc.demo.certspki extracts the P-256 SPKI public key and matches the independent core parser byte-for-byte" -Passed $positiveOk -Actual $(if ($positiveOk) { "matched" } else { ($result | ConvertTo-Json -Compress -Depth 8) })
if (-not $positiveOk) { throw "Expected svc.demo.certspki positive cross-check to match core SPKI parse and output SHA" }

$malformed = $result.malformed_case
$malformedOk = (
    $malformed.guest_record_valid -eq $true -and
    $malformed.guest_parse_ok -eq $false -and
    $malformed.core_parse_ok -eq $false -and
    [int]$malformed.guest_error_code -eq 1 -and
    [int]$malformed.core_error_code -eq 1 -and
    [int]$malformed.guest_spki_der_len -eq 0 -and
    $malformed.guest_matches_core -eq $true -and
    $malformed.capability_granted -eq $false
)
Add-Predicate -Name "m11-8-certspki:malformed-error-agree" -Expected "truncated cert returns matching guest/core no-P256-SPKI error and grants nothing" -Passed $malformedOk -Actual $(if ($malformedOk) { "matched" } else { ($malformed | ConvertTo-Json -Compress -Depth 6) })
if (-not $malformedOk) { throw "Expected svc.demo.certspki malformed case to agree with core no-P256-SPKI error" }

$negative = $result.negative
$negativeOk = (
    $negative.module_imports_within_authorized_list -eq $false -and
    $negative.run_outcome -eq "module_import_not_authorized" -and
    $negative.missing_import_module -eq "env" -and
    $negative.instantiation_ok -eq $false -and
    [int]$negative.captured_output_len -eq 0
)
Add-Predicate -Name "m11-8-certspki:refused-unless-granted" -Expected "svc.demo.certspki refuses the partial import grant before instantiation" -Passed $negativeOk -Actual $(if ($negativeOk) { "matched" } else { ($negative | ConvertTo-Json -Compress -Depth 6) })
if (-not $negativeOk) { throw "Expected svc.demo.certspki subset denial to fail closed before instantiation" }

$grantsNothingOk = (
    $result.guest_output_is_evidence_only -eq $true -and
    $result.core_is_authority -eq $true -and
    $result.policy_allows_beyond_env -eq $false -and
    $result.authorizes_provider_request -eq $false -and
    $result.authorizes_provider_export -eq $false -and
    $result.durable_write -eq $false -and
    $result.capability_granted -eq $false -and
    $result.validates_provider_spki -eq $false -and
    $result.owner_sealed -eq $false -and
    $result.trust_tier -eq "dev_key_not_owner_sealed"
)
Add-Predicate -Name "m11-8-certspki:grants-nothing" -Expected "certspki output is evidence only and every trust/authority/durable flag remains false" -Passed $grantsNothingOk -Actual $(if ($grantsNothingOk) { "matched" } else { ($result | ConvertTo-Json -Compress -Depth 8) })
if (-not $grantsNothingOk) { throw "Expected svc.demo.certspki to grant no trust, provider, durable, or capability authority" }
