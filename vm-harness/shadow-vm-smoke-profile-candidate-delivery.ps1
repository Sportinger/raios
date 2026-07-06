$candidatePath = if ($ResolvedArtifact) {
    $ResolvedArtifact
}
else {
    Join-Path $RepoRoot "seed-kernel\artifacts\svc.demo.echo.wasm"
}

$delivery = Send-CandidateBytes -Path $candidatePath
$finalize = $delivery.finalize_response
$result = $finalize.body.result
$expectedSha = "sha256:f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2"

$positiveOk = (
    $finalize.t -eq "response" -and
    $finalize.body.method -eq "module.submit_candidate_finalize" -and
    [int]$result.byte_len -eq 4205 -and
    [int]$result.delivered_byte_len -eq 4205 -and
    $result.artifact_sha256 -eq $expectedSha -and
    $result.wasm_valid -eq $true -and
    $result.retained_in_ram -eq $true -and
    $result.rejected -eq $false -and
    $result.external_delivery_channel -eq "serial_console_base64_chunks_v0" -and
    $result.load_attempted -eq $false -and
    $result.authorizes_load -eq $false -and
    $result.execution_attempted -eq $false -and
    $result.authorizes_execution -eq $false -and
    $result.writes_persistent_state -eq $false
)
Add-Predicate -Name "candidate_delivery:finalize_accepts_external_echo_wasm" -Expected "external echo wasm accepted as inert current-boot candidate" -Passed $positiveOk -Actual $(if ($positiveOk) { "matched" } else { ($result | ConvertTo-Json -Compress -Depth 6) })
if (-not $positiveOk) {
    throw "Expected delivered candidate artifact to be retained as a valid inert wasm candidate"
}

Assert-LogContains -Name "candidate_delivery:serial_byte_len" -Needle '"byte_len": 4205' -TimeoutSeconds 1
Assert-LogContains -Name "candidate_delivery:serial_artifact_sha256" -Needle "`"artifact_sha256`": `"$expectedSha`"" -TimeoutSeconds 1
Assert-LogContains -Name "candidate_delivery:serial_wasm_valid" -Needle '"wasm_valid": true' -TimeoutSeconds 1
Assert-LogContains -Name "candidate_delivery:serial_retained" -Needle '"retained_in_ram": true' -TimeoutSeconds 1
Assert-LogContains -Name "candidate_delivery:serial_not_rejected" -Needle '"rejected": false' -TimeoutSeconds 1
Assert-LogContains -Name "candidate_delivery:serial_delivery_channel" -Needle '"external_delivery_channel": "serial_console_base64_chunks_v0"' -TimeoutSeconds 1
Assert-LogContains -Name "candidate_delivery:serial_load_not_attempted" -Needle '"load_attempted": false' -TimeoutSeconds 1
Assert-LogContains -Name "candidate_delivery:serial_load_not_authorized" -Needle '"authorizes_load": false' -TimeoutSeconds 1
Assert-LogContains -Name "candidate_delivery:serial_execution_not_attempted" -Needle '"execution_attempted": false' -TimeoutSeconds 1
Assert-LogContains -Name "candidate_delivery:serial_execution_not_authorized" -Needle '"authorizes_execution": false' -TimeoutSeconds 1
Assert-LogContains -Name "candidate_delivery:serial_persistence_denied" -Needle '"writes_persistent_state": false' -TimeoutSeconds 1

Send-AgentCommand -Command "module.submit_candidate_chunk !!!!" -ExpectedMarker "RAIOS_AGENT_END module.submit_candidate_chunk" -Name "candidate_delivery:malformed_chunk"
$badChunk = Get-LastAgentResponseJson -Method "module.submit_candidate_chunk"
$badChunkResult = $badChunk.body.result
$badChunkOk = (
    $badChunkResult.rejected -eq $true -and
    $badChunkResult.accepted -eq $false -and
    $badChunkResult.discarded_pending_delivery -eq $true -and
    [int]$badChunkResult.pending_byte_len -eq 0
)
Add-Predicate -Name "candidate_delivery:malformed_chunk_discards_pending" -Expected "malformed base64 rejects and clears pending delivery" -Passed $badChunkOk -Actual $(if ($badChunkOk) { "matched" } else { ($badChunkResult | ConvertTo-Json -Compress -Depth 4) })
if (-not $badChunkOk) {
    throw "Expected malformed base64 chunk to reject and discard pending delivery"
}

Send-AgentCommand -Command "module.submit_candidate_finalize" -ExpectedMarker "RAIOS_AGENT_END module.submit_candidate_finalize" -Name "candidate_delivery:finalize_after_malformed"
$badFinalize = Get-LastAgentResponseJson -Method "module.submit_candidate_finalize"
$badFinalizeResult = $badFinalize.body.result
$badFinalizeOk = (
    $badFinalizeResult.rejected -eq $true -and
    $badFinalizeResult.retained_in_ram -eq $false -and
    [int]$badFinalizeResult.byte_len -eq 0 -and
    $badFinalizeResult.reason -eq "rejected_empty_candidate_delivery" -and
    $badFinalizeResult.load_attempted -eq $false -and
    $badFinalizeResult.authorizes_load -eq $false -and
    $badFinalizeResult.execution_attempted -eq $false -and
    $badFinalizeResult.authorizes_execution -eq $false -and
    $badFinalizeResult.writes_persistent_state -eq $false
)
Add-Predicate -Name "candidate_delivery:finalize_after_malformed_rejected" -Expected "empty finalize after malformed chunk rejects without panic" -Passed $badFinalizeOk -Actual $(if ($badFinalizeOk) { "matched" } else { ($badFinalizeResult | ConvertTo-Json -Compress -Depth 5) })
if (-not $badFinalizeOk) {
    throw "Expected finalize after malformed chunk to reject empty delivery"
}

Send-AgentCommand -Command "caps" -ExpectedMarker "RAIOS_AGENT_END system.capabilities" -Name "candidate_delivery:followup_caps_after_bad_finalize"
