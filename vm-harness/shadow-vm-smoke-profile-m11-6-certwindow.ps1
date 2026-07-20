function Test-CertWindowDateTime {
    param(
        $Value,
        [int]$Year,
        [int]$Month,
        [int]$Day,
        [int]$Hour,
        [int]$Minute,
        [int]$Second
    )

    return (
        [int]$Value.year -eq $Year -and
        [int]$Value.month -eq $Month -and
        [int]$Value.day -eq $Day -and
        [int]$Value.hour -eq $Hour -and
        [int]$Value.minute -eq $Minute -and
        [int]$Value.second -eq $Second
    )
}

Send-AgentCommand -Command "wasm.certwindow_probe" -ExpectedMarker "RAIOS_AGENT_END wasm.certwindow_probe" -Name "m11-6-certwindow:probe"
$certwindow = Get-LastAgentResponseJson -Method "wasm.certwindow_probe"
$result = $certwindow.body.result

$positiveOk = (
    $result.schema -eq "raios.wasm_certwindow_probe.v0" -and
    $result.service_id -eq "svc.demo.certwindow" -and
    $result.run_outcome -eq "success" -and
    [int]$result.authorized_import_count -eq 3 -and
    [int]$result.linked_host_import_count -eq 3 -and
    $result.module_imports_within_authorized_list -eq $true -and
    $result.input_sha256 -eq "sha256:baa2a6c3263fb8170aa2b4013046414a1e4760c2f5e7bfdf88c74f51742e0cb4" -and
    [int]$result.captured_output_len -eq 18 -and
    $result.guest_record_valid -eq $true -and
    $result.guest_parse_ok -eq $true -and
    $result.core_parse_ok -eq $true -and
    (Test-CertWindowDateTime -Value $result.guest_not_before -Year 2021 -Month 10 -Day 13 -Hour 8 -Minute 20 -Second 42) -and
    (Test-CertWindowDateTime -Value $result.core_not_before -Year 2021 -Month 10 -Day 13 -Hour 8 -Minute 20 -Second 42) -and
    (Test-CertWindowDateTime -Value $result.guest_not_after -Year 2031 -Month 10 -Day 11 -Hour 8 -Minute 20 -Second 42) -and
    (Test-CertWindowDateTime -Value $result.core_not_after -Year 2031 -Month 10 -Day 11 -Hour 8 -Minute 20 -Second 42) -and
    $result.guest_matches_core -eq $true -and
    $result.output_bytes_match -eq $true -and
    $result.captured_output_sha256 -eq $result.core_output_sha256 -and
    [int64]$result.fuel_used -lt [int64]$result.fuel_budget -and
    [int64]$result.fuel_used -gt 0
)
Add-Predicate -Name "m11-6-certwindow:positive-crosscheck" -Expected "svc.demo.certwindow parses the real cert and matches the independent core parser" -Passed $positiveOk -Actual $(if ($positiveOk) { "matched" } else { ($result | ConvertTo-Json -Compress -Depth 8) })
if (-not $positiveOk) {
    throw "Expected svc.demo.certwindow positive cross-check to match core parse and output SHA"
}

$malformed = $result.malformed_case
$malformedOk = (
    $malformed.guest_record_valid -eq $true -and
    $malformed.guest_parse_ok -eq $false -and
    $malformed.core_parse_ok -eq $false -and
    [int]$malformed.guest_error_code -eq 2 -and
    [int]$malformed.core_error_code -eq 2 -and
    $malformed.error_codes_match -eq $true -and
    $malformed.guest_matches_core -eq $true -and
    $malformed.guest_error_reason -eq "truncated" -and
    $malformed.capability_granted -eq $false
)
Add-Predicate -Name "m11-6-certwindow:malformed-error-agree" -Expected "truncated cert returns matching guest/core error code 2 and grants nothing" -Passed $malformedOk -Actual $(if ($malformedOk) { "matched" } else { ($malformed | ConvertTo-Json -Compress -Depth 6) })
if (-not $malformedOk) {
    throw "Expected svc.demo.certwindow malformed case to agree with core truncated error"
}

$negative = $result.negative
$negativeOk = (
    $negative.module_imports_within_authorized_list -eq $false -and
    $negative.run_outcome -eq "module_import_not_authorized" -and
    $negative.missing_import_module -eq "env" -and
    $negative.instantiation_ok -eq $false -and
    [int]$negative.captured_output_len -eq 0
)
Add-Predicate -Name "m11-6-certwindow:refused-unless-granted" -Expected "svc.demo.certwindow refuses the partial import grant before instantiation" -Passed $negativeOk -Actual $(if ($negativeOk) { "matched" } else { ($negative | ConvertTo-Json -Compress -Depth 6) })
if (-not $negativeOk) {
    throw "Expected svc.demo.certwindow subset denial to fail closed before instantiation"
}

$grantsNothingOk = (
    $result.guest_output_is_evidence_only -eq $true -and
    $result.core_is_authority -eq $true -and
    $result.policy_allows_beyond_env -eq $false -and
    $result.authorizes_provider_request -eq $false -and
    $result.authorizes_provider_export -eq $false -and
    $result.durable_write -eq $false -and
    $result.capability_granted -eq $false -and
    $result.validates_cert_time -eq $false -and
    $result.owner_sealed -eq $false -and
    $result.trust_tier -eq "dev_key_not_owner_sealed"
)
Add-Predicate -Name "m11-6-certwindow:grants-nothing" -Expected "certwindow output is evidence only and every trust/authority/durable flag remains false" -Passed $grantsNothingOk -Actual $(if ($grantsNothingOk) { "matched" } else { ($result | ConvertTo-Json -Compress -Depth 8) })
if (-not $grantsNothingOk) {
    throw "Expected svc.demo.certwindow to grant no trust, provider, durable, or capability authority"
}
