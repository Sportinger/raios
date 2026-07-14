Send-AgentCommand -Command "wasm.dnsparse_probe" -ExpectedMarker "RAIOS_AGENT_END wasm.dnsparse_probe" -Name "m11-9-dnsparse:probe"
$dnsparse = Get-LastAgentResponseJson -Method "wasm.dnsparse_probe"
$result = $dnsparse.body.result

$positiveOk = (
    $result.schema -eq "raios.wasm_dnsparse_probe.v0" -and
    $result.service_id -eq "svc.demo.dnsparse" -and
    [int]$result.authorized_import_count -eq 3 -and
    @($result.authorized_host_imports).Count -eq 3 -and
    $result.authorized_host_imports[0] -eq "env.input_len" -and
    $result.authorized_host_imports[1] -eq "env.input_read" -and
    $result.authorized_host_imports[2] -eq "env.output_write" -and
    $result.run_outcome -eq "success" -and
    $result.input_sha256 -eq "sha256:249c19391d1dec57f30ba590c45650aa6b8b6dbddd69866a29ad8449cab5184c" -and
    [int]$result.captured_output_len -eq 16 -and
    $result.guest_answer.present -eq $true -and
    $result.core_answer.present -eq $true -and
    (@($result.guest_answer.address) -join ".") -eq "104.18.33.45" -and
    (@($result.core_answer.address) -join ".") -eq "104.18.33.45" -and
    [int]$result.guest_answer.ttl -eq 60 -and
    [int]$result.core_answer.ttl -eq 60 -and
    $result.guest_matches_core -eq $true -and
    $result.output_bytes_match -eq $true -and
    $result.crosscheck_passed -eq $true -and
    $result.fuel_bounded -eq $true
)
Add-Predicate -Name "m11-9-dnsparse:positive-crosscheck" -Expected "svc.demo.dnsparse parses the pinned DNS A answer through three exact imports and matches the independent core parser byte-for-byte" -Passed $positiveOk -Actual $(if ($positiveOk) { "matched" } else { ($result | ConvertTo-Json -Compress -Depth 8) })
if (-not $positiveOk) { throw "Expected svc.demo.dnsparse positive cross-check to match the core DNS answer and output bytes" }

$truncated = $result.truncated_case
$truncatedOk = (
    $truncated.canonical_no_answer -eq $true -and
    $truncated.crosscheck_passed -eq $true
)
Add-Predicate -Name "m11-9-dnsparse:truncated-agree" -Expected "truncated DNS input returns canonical no-answer in both guest and core" -Passed $truncatedOk -Actual $(if ($truncatedOk) { "matched" } else { ($truncated | ConvertTo-Json -Compress -Depth 6) })
if (-not $truncatedOk) { throw "Expected svc.demo.dnsparse truncated case to agree with the core canonical no-answer" }

$pointerLoop = $result.pointer_loop_case
$pointerLoopOk = (
    $pointerLoop.canonical_no_answer -eq $true -and
    $pointerLoop.crosscheck_passed -eq $true
)
Add-Predicate -Name "m11-9-dnsparse:pointer-loop-agree" -Expected "DNS compression pointer loop returns bounded canonical no-answer in both guest and core" -Passed $pointerLoopOk -Actual $(if ($pointerLoopOk) { "matched" } else { ($pointerLoop | ConvertTo-Json -Compress -Depth 6) })
if (-not $pointerLoopOk) { throw "Expected svc.demo.dnsparse pointer-loop case to agree with the core canonical no-answer" }

$negative = $result.negative
$negativeOk = (
    $negative.run_outcome -eq "module_import_not_authorized" -and
    $negative.instantiation_ok -eq $false -and
    [int]$negative.captured_output_len -eq 0 -and
    $negative.denied_before_instantiation -eq $true -and
    $negative.capability_granted -eq $false
)
Add-Predicate -Name "m11-9-dnsparse:refused-unless-granted" -Expected "svc.demo.dnsparse refuses the partial import grant before instantiation with zero output" -Passed $negativeOk -Actual $(if ($negativeOk) { "matched" } else { ($negative | ConvertTo-Json -Compress -Depth 6) })
if (-not $negativeOk) { throw "Expected svc.demo.dnsparse subset denial to fail closed before instantiation" }

$grantsNothingOk = (
    $result.guest_output_is_evidence_only -eq $true -and
    $result.core_is_authority -eq $true -and
    $result.policy_allows_beyond_env -eq $false -and
    $result.owner_sealed -eq $false -and
    $result.trust_tier -eq "dev_key_not_owner_sealed" -and
    $result.authorizes_provider_request -eq $false -and
    $result.authorizes_provider_export -eq $false -and
    $result.authorizes_dns_cache_update -eq $false -and
    $result.durable_write -eq $false -and
    $result.capability_granted -eq $false -and
    $result.evidence_complete -eq $true
)
Add-Predicate -Name "m11-9-dnsparse:grants-nothing" -Expected "dnsparse output is evidence only and every trust, provider, cache, durable, and capability authority remains false" -Passed $grantsNothingOk -Actual $(if ($grantsNothingOk) { "matched" } else { ($result | ConvertTo-Json -Compress -Depth 8) })
if (-not $grantsNothingOk) { throw "Expected svc.demo.dnsparse to grant no trust, provider, DNS-cache, durable, or capability authority" }
