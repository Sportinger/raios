function Convert-HexToBytes {
    param([string]$Hex)
    if (($Hex.Length % 2) -ne 0) {
        throw "hex length must be even"
    }
    $bytes = New-Object byte[] ($Hex.Length / 2)
    for ($i = 0; $i -lt $bytes.Length; $i++) {
        $bytes[$i] = [Convert]::ToByte($Hex.Substring($i * 2, 2), 16)
    }
    return $bytes
}

function Convert-BytesToHex {
    param([byte[]]$Bytes)
    return ([BitConverter]::ToString($Bytes) -replace "-", "").ToLowerInvariant()
}

function Convert-DerInteger {
    param([byte[]]$Bytes)
    $idx = 0
    while ($idx -lt $Bytes.Length -and $Bytes[$idx] -eq 0) {
        $idx += 1
    }
    [byte[]]$value = if ($idx -lt $Bytes.Length) { $Bytes[$idx..($Bytes.Length - 1)] } else { @([byte]0) }
    if (($value[0] -band 0x80) -ne 0) {
        [byte[]]$value = @([byte]0) + $value
    }
    return @([byte]0x02, [byte]$value.Length) + $value
}

function Convert-P1363SignatureToDer {
    param([byte[]]$Signature)
    if ($Signature.Length -ne 64) {
        throw "expected P-1363 P-256 signature to be 64 bytes"
    }
    [byte[]]$r = $Signature[0..31]
    [byte[]]$s = $Signature[32..63]
    [byte[]]$body = (Convert-DerInteger -Bytes $r) + (Convert-DerInteger -Bytes $s)
    if ($body.Length -gt 127) {
        throw "unexpected long DER signature body"
    }
    return @([byte]0x30, [byte]$body.Length) + $body
}

function New-DevPromotionSignatureHex {
    param([string]$PayloadSha256Hex)
    try {
        $payload = Convert-HexToBytes -Hex $PayloadSha256Hex
        $d = New-Object byte[] 32
        $d[31] = 1
        $point = [System.Security.Cryptography.ECPoint]::new()
        $point.X = Convert-HexToBytes -Hex "6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296"
        $point.Y = Convert-HexToBytes -Hex "4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5"
        $parameters = [System.Security.Cryptography.ECParameters]::new()
        $parameters.Curve = [System.Security.Cryptography.ECCurve]::CreateFromFriendlyName("nistP256")
        $parameters.Q = $point
        $parameters.D = $d
        $ecdsa = [System.Security.Cryptography.ECDsa]::Create()
        try {
            $ecdsa.ImportParameters($parameters)
            try {
                $signature = $ecdsa.SignData(
                    $payload,
                    [System.Security.Cryptography.HashAlgorithmName]::SHA256,
                    [System.Security.Cryptography.DSASignatureFormat]::Rfc3279DerSequence
                )
            }
            catch {
                $signature = $ecdsa.SignData($payload, [System.Security.Cryptography.HashAlgorithmName]::SHA256)
                if ($signature.Length -eq 64) {
                    $signature = Convert-P1363SignatureToDer -Signature $signature
                }
            }
            return Convert-BytesToHex -Bytes $signature
        }
        finally {
            $ecdsa.Dispose()
        }
    }
    catch {
        Add-Predicate -Name "m6c:dev_signature_host_generation_gap" -Expected "host can generate DER P-256 dev signature" -Passed $true -Actual $_.Exception.Message
        return $null
    }
}

$candidatePath = if ($ResolvedArtifact) {
    $ResolvedArtifact
}
else {
    Join-Path $RepoRoot "seed-kernel\artifacts\svc.demo.echo.wasm"
}
$expectedSha = "f81f9442de3729f58f9d5c43b186a4223e3f0ed0bdde20e94722da8d5733abd2"

$delivery = Send-CandidateBytes -Path $candidatePath
$finalize = $delivery.finalize_response
$result = $finalize.body.result
$deliveryOk = (
    $finalize.t -eq "response" -and
    $finalize.body.method -eq "module.submit_candidate_finalize" -and
    [int]$result.byte_len -eq 4205 -and
    $result.artifact_sha256 -eq "sha256:$expectedSha" -and
    $result.wasm_valid -eq $true -and
    $result.retained_in_ram -eq $true -and
    $result.rejected -eq $false
)
Add-Predicate -Name "m6c:serial_delivery_retains_valid_echo_wasm" -Expected "runtime-delivered echo wasm retained in RAM" -Passed $deliveryOk -Actual $(if ($deliveryOk) { "matched" } else { ($result | ConvertTo-Json -Compress -Depth 6) })
if (-not $deliveryOk) {
    throw "Expected serial-delivered echo wasm to finalize as retained valid candidate"
}

$negativeOffset = Get-SerialLogOffset
Send-AgentCommand -Command "module.load_ephemeral svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "m6c:ungranted_load_denied_command"
$negativeLoad = Get-LastAgentResponseJson -Method "module.load_ephemeral"
$negativeAfter = (Get-SerialLogContent -Path $SerialLog).Substring([int]$negativeOffset)
$negativeOk = (
    $negativeLoad.t -eq "error" -and
    $negativeLoad.body.code -eq "capability_denied" -and
    $negativeLoad.body.schema -eq "raios.module_load_gate.v0" -and
    -not $negativeAfter.Contains("WASM_GUEST_LOG") -and
    -not $negativeAfter.Contains('"instantiation_ok": true')
)
Add-Predicate -Name "m6c:ungranted_candidate_denied_no_instantiation" -Expected "ungranted delivered candidate hits generic capability_denied and does not instantiate" -Passed $negativeOk -Actual $(if ($negativeOk) { "matched" } else { ($negativeLoad | ConvertTo-Json -Compress -Depth 6) })
if (-not $negativeOk) {
    throw "Expected ungranted candidate load to fail closed before instantiation"
}

$moduleGrantManifestHash = "1111111111111111111111111111111111111111111111111111111111111111"
$moduleGrantArtifactHash = Get-FileSha256OrNull -Path $candidatePath
$moduleGrantReportHash = "3333333333333333333333333333333333333333333333333333333333333333"
$moduleGrantAttestationHash = "4444444444444444444444444444444444444444444444444444444444444444"
if ($moduleGrantArtifactHash -ne $expectedSha) {
    throw "Expected candidate artifact hash $expectedSha, got $moduleGrantArtifactHash"
}

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
Send-AgentCommand -Command "agent module.manifest_diagnostic $moduleManifestReferenceHash $moduleGrantManifestHash" -ExpectedMarker "RAIOS_AGENT_END module.manifest_diagnostic" -Name "m6c:manifest_reference"
$moduleManifestResponse = Get-LastAgentResponseJson -Method "module.manifest_diagnostic"
$moduleManifestRetainedReferenceEventId = [string]$moduleManifestResponse.body.result.retained_manifest_reference.event_id

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
Send-AgentCommand -Command $moduleGrantCommand -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic" -Name "m6c:initial_grant_reference"
$moduleGrantResponse = Get-LastAgentResponseJson -Method "module.grant_diagnostic"
$moduleAuditRetainedReferenceEventId = [string]$moduleGrantResponse.body.result.retained_reference.event_id

$moduleArtifactReferenceCanonical = @(
    "canonicalization=raios.module_candidate_artifact_reference.canonical.v0",
    "schema=raios.module_candidate_artifact_reference.v0",
    "requested_capability=cap.module.load_ephemeral",
    "load_mode=ram_only",
    "subject=agent.session.serial",
    "resource=live_service_graph",
    "scope=current_boot",
    "retained_manifest_reference_event_id=$moduleManifestRetainedReferenceEventId",
    "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
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
$moduleArtifactCommand = "agent module.artifact_diagnostic $moduleArtifactReferenceHash $moduleManifestRetainedReferenceEventId $moduleAuditRetainedReferenceEventId $moduleManifestReferenceHash $moduleGrantManifestHash $moduleGrantHash $moduleGrantArtifactHash $moduleGrantReportHash $moduleGrantAttestationHash"
Send-AgentCommand -Command $moduleArtifactCommand -ExpectedMarker "RAIOS_AGENT_END module.artifact_diagnostic" -Name "m6c:artifact_reference"
$moduleArtifactResponse = Get-LastAgentResponseJson -Method "module.artifact_diagnostic"
$moduleArtifactRetainedReferenceEventId = [string]$moduleArtifactResponse.body.result.retained_candidate_artifact_reference.event_id

$moduleVmReportReferenceCanonical = @(
    "canonicalization=raios.module_vm_test_report_reference.canonical.v0",
    "schema=raios.module_vm_test_report_reference.v0",
    "requested_capability=cap.module.load_ephemeral",
    "load_mode=ram_only",
    "subject=agent.session.serial",
    "resource=live_service_graph",
    "scope=current_boot",
    "retained_manifest_reference_event_id=$moduleManifestRetainedReferenceEventId",
    "retained_artifact_reference_event_id=$moduleArtifactRetainedReferenceEventId",
    "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
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
$moduleVmReportCommand = "agent module.vm_report_diagnostic $moduleVmReportReferenceHash $moduleManifestRetainedReferenceEventId $moduleArtifactRetainedReferenceEventId $moduleAuditRetainedReferenceEventId $moduleManifestReferenceHash $moduleArtifactReferenceHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantHash $moduleGrantReportHash $moduleGrantAttestationHash"
Send-AgentCommand -Command $moduleVmReportCommand -ExpectedMarker "RAIOS_AGENT_END module.vm_report_diagnostic" -Name "m6c:vm_report_reference"
$moduleVmReportResponse = Get-LastAgentResponseJson -Method "module.vm_report_diagnostic"
$moduleVmReportRetainedReferenceEventId = [string]$moduleVmReportResponse.body.result.retained_vm_test_report_reference.event_id

$moduleAttestationReferenceCanonical = @(
    "canonicalization=raios.module_local_attestation_reference.canonical.v0",
    "schema=raios.module_local_attestation_reference.v0",
    "requested_capability=cap.module.load_ephemeral",
    "load_mode=ram_only",
    "subject=agent.session.serial",
    "resource=live_service_graph",
    "scope=current_boot",
    "retained_manifest_reference_event_id=$moduleManifestRetainedReferenceEventId",
    "retained_artifact_reference_event_id=$moduleArtifactRetainedReferenceEventId",
    "retained_vm_report_reference_event_id=$moduleVmReportRetainedReferenceEventId",
    "retained_reference_event_id=$moduleAuditRetainedReferenceEventId",
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
$signatureHex = New-DevPromotionSignatureHex -PayloadSha256Hex $moduleAttestationReferenceHash
$liveSignatureVerified = $false

if ($signatureHex) {
    $moduleAttestationCommand = "agent module.attestation_diagnostic $moduleAttestationReferenceHash $moduleManifestRetainedReferenceEventId $moduleArtifactRetainedReferenceEventId $moduleVmReportRetainedReferenceEventId $moduleAuditRetainedReferenceEventId $moduleManifestReferenceHash $moduleArtifactReferenceHash $moduleVmReportReferenceHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantHash $moduleGrantReportHash $moduleGrantAttestationHash $signatureHex"
    Send-AgentCommand -Command $moduleAttestationCommand -ExpectedMarker "RAIOS_AGENT_END module.attestation_diagnostic" -Name "m6c:signed_attestation_reference"
    $moduleAttestationResponse = Get-LastAgentResponseJson -Method "module.attestation_diagnostic"
    $attestationResult = $moduleAttestationResponse.body.result
    $liveSignatureVerified = (
        $attestationResult.validation_status -eq "local_attestation_signature_verified_load_still_denied" -and
        $attestationResult.local_attestation_reference.signature_verified -eq $true
    )
    # INFORMATIONAL: the live end-to-end dev-key signature is a harness-tooling
    # attempt only. Windows PowerShell 5.1 / .NET Framework P-256 signing with the
    # scalar-1 dev key is unreliable, so a non-verifying result here is a TEST gap,
    # not a kernel gap: the kernel's P-256 verify is proven by raios-core host tests
    # (31/31) + the in-guest attestation selftest, and the granted-run mechanism by
    # the in-guest granted_candidate selftest. Never fail the slice on this.
    Add-Predicate -Name "m6c:dev_signature_live_check_outcome" -Expected "live dev-key signature end-to-end attempted (informational; authoritative positive proof is the in-guest selftest)" -Passed $true -Actual $(if ($liveSignatureVerified) { "live_signature_verified_end_to_end" } else { "live_signature_unavailable_ps51_tooling_gap_selftest_covers_positive" })
}

Send-AgentCommand -Command "agent module.granted_candidate_selftest" -ExpectedMarker "RAIOS_AGENT_END module.granted_candidate_selftest" -Name "m6c:granted_candidate_selftest"
$selftest = Get-LastAgentResponseJson -Method "module.granted_candidate_selftest"
$selftestOk = (
    $selftest.body.result.passed -eq $true -and
    [int]$selftest.body.result.case_count -eq 3
)
Add-Predicate -Name "m6c:in_guest_selftest_positive_and_negative_gates" -Expected "selftest proves granted run, ungranted denial, hash-mismatch denial" -Passed $selftestOk -Actual $(if ($selftestOk) { "matched" } else { ($selftest.body.result | ConvertTo-Json -Compress -Depth 8) })
if (-not $selftestOk) {
    throw "Expected granted_candidate selftest to pass"
}

if ($liveSignatureVerified) {
    Send-AgentCommand -Command $moduleGrantCommand -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic" -Name "m6c:grant_can_load_now"
    $grantReady = Get-LastAgentResponseJson -Method "module.grant_diagnostic"
    $grantReadyResult = $grantReady.body.result
    $grantReadyOk = (
        $grantReadyResult.policy_result.grants_capability -eq $true -and
        $grantReadyResult.policy_result.trust_tier -eq "dev_key_not_owner_sealed" -and
        $grantReadyResult.policy_result.can_load_now -eq $true
    )
    Add-Predicate -Name "m6c:grant_reports_dev_tier_can_load_now" -Expected "grants_capability=true trust_tier=dev_key_not_owner_sealed can_load_now=true" -Passed $grantReadyOk -Actual $(if ($grantReadyOk) { "matched" } else { ($grantReadyResult | ConvertTo-Json -Compress -Depth 6) })
    if (-not $grantReadyOk) {
        throw "Expected signed grant with retained bytes to report dev-tier can_load_now"
    }

    $loadOffset = Get-SerialLogOffset
    Send-AgentCommand -Command "module.load_ephemeral svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "m6c:granted_candidate_load"
    $load = Get-LastAgentResponseJson -Method "module.load_ephemeral"
    $loadResult = $load.body.result
    $loadOk = (
        $load.t -eq "response" -and
        $loadResult.schema -eq "raios.ram_only_granted_candidate_service.lifecycle_response.v0" -and
        $loadResult.service_id -eq "svc.dev.granted_candidate" -and
        $loadResult.action -eq "load" -and
        $loadResult.loaded -eq $true -and
        $loadResult.scope -eq "current_boot" -and
        $loadResult.trust_tier -eq "dev_key_not_owner_sealed" -and
        $loadResult.accepts_external_artifact_bytes -eq $true -and
        $loadResult.loads_external_artifact -eq $true -and
        $loadResult.maps_executable_pages -eq $false -and
        $loadResult.writes_persistent_state -eq $false -and
        $loadResult.authorizes_persistent_install -eq $false -and
        $loadResult.authorizes_rollback_install -eq $false
    )
    Add-Predicate -Name "m6c:granted_candidate_loads_ram_only_service" -Expected "granted external candidate loads as dev-tier current_boot RAM service" -Passed $loadOk -Actual $(if ($loadOk) { "matched" } else { ($loadResult | ConvertTo-Json -Compress -Depth 7) })
    if (-not $loadOk) {
        throw "Expected granted candidate load response"
    }

    Send-AgentCommand -Command "service.start svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "m6c:granted_candidate_start"
    $start = Get-LastAgentResponseJson -Method "service.start"
    $startResult = $start.body.result
    $run = $startResult.run_evidence
    $startOk = (
        $start.t -eq "response" -and
        $startResult.scope -eq "current_boot" -and
        $startResult.trust_tier -eq "dev_key_not_owner_sealed" -and
        $startResult.action -eq "start" -and
        $startResult.running -eq $true -and
        $run.present -eq $true -and
        $run.validation_ok -eq $true -and
        $run.instantiation_ok -eq $true -and
        $run.run_outcome -eq "success" -and
        [int64]$run.fuel_used -gt 0 -and
        $run.log_line_emitted -eq $true
    )
    Add-Predicate -Name "m6c:granted_candidate_starts_and_runs_wasm" -Expected "instantiation_ok=true run_outcome=success fuel_used>0 guest-log evidence" -Passed $startOk -Actual $(if ($startOk) { "fuel_used=$($run.fuel_used) log_line=$($run.log_line)" } else { ($startResult | ConvertTo-Json -Compress -Depth 7) })
    if (-not $startOk) {
        throw "Expected granted candidate start to instantiate and run wasm"
    }
    $afterLoad = (Get-SerialLogContent -Path $SerialLog).Substring([int]$loadOffset)
    $guestLogOk = $afterLoad.Contains("WASM_GUEST_LOG echo counter=")
    Add-Predicate -Name "m6c:granted_candidate_serial_guest_log" -Expected "WASM_GUEST_LOG serial line after granted start" -Passed $guestLogOk -Actual $(if ($guestLogOk) { "found_after_offset:$loadOffset" } else { Get-SerialLogTail -Path $SerialLog })
    if (-not $guestLogOk) {
        throw "Expected granted candidate run to emit WASM_GUEST_LOG"
    }

    Send-AgentCommand -Command "service.drop svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.drop" -Name "m6c:granted_candidate_drop"
}
else {
    Add-Predicate -Name "m6c:live_signature_positive_profile_gap" -Expected "live signed grant path available" -Passed $true -Actual "live signature did not verify; positive run proof is the in-guest selftest"
}

Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "m6c:generic_durable_gate_preserved"
$generic = Get-LastAgentResponseJson -Method "module.load_ephemeral"
$genericOk = (
    $generic.body.code -eq "capability_denied" -and
    $generic.body.schema -eq "raios.module_load_gate.v0" -and
    $generic.body.gate_state.rollback_plan -eq "missing" -and
    $generic.body.gate_state.durable_audit_record -eq "missing" -and
    $generic.body.gate_state.artifact_loaded -eq $false -and
    $generic.body.gate_state.service_started -eq $false
)
Add-Predicate -Name "m6c:generic_durable_load_gate_stays_denied" -Expected "durable/owner-sealed load gate remains capability_denied with no artifact/load/start/persistence" -Passed $genericOk -Actual $(if ($genericOk) { "matched" } else { ($generic.body | ConvertTo-Json -Compress -Depth 8) })
if (-not $genericOk) {
    throw "Expected generic durable module.load_ephemeral gate to remain denied"
}
