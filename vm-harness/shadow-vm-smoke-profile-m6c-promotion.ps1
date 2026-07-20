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
    $negativeLoad.schema -eq "raios.evidence_response.v1" -and
    $negativeLoad.family -eq "module.load_gate" -and
    $negativeLoad.decision.outcome -eq "denied" -and
    @($negativeLoad.decision.grants).Count -eq 0 -and
    @($negativeLoad.decision.effects).Count -eq 0 -and
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
$moduleManifestRetainedReferenceEventId = [string]($moduleManifestResponse.evidence | Where-Object id -eq "module_manifest_retained").source_event_id

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
$moduleAuditRetainedReferenceEventId = [string]($moduleGrantResponse.evidence | Where-Object id -eq "computed_capability_grant_retained").source_event_id

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
$moduleArtifactRetainedReferenceEventId = [string]($moduleArtifactResponse.evidence | Where-Object id -eq "candidate_artifact_retained").source_event_id

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
$moduleVmReportRetainedReferenceEventId = [string]($moduleVmReportResponse.evidence | Where-Object id -eq "vm_test_report_retained").source_event_id

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
$signatureHex = New-ReliableDevPromotionSignatureHex -AttestationReferenceHash $moduleAttestationReferenceHash
$liveSignatureVerified = $false

if ($signatureHex) {
    $moduleAttestationCommand = "agent module.attestation_diagnostic $moduleAttestationReferenceHash $moduleManifestRetainedReferenceEventId $moduleArtifactRetainedReferenceEventId $moduleVmReportRetainedReferenceEventId $moduleAuditRetainedReferenceEventId $moduleManifestReferenceHash $moduleArtifactReferenceHash $moduleVmReportReferenceHash $moduleGrantManifestHash $moduleGrantArtifactHash $moduleGrantHash $moduleGrantReportHash $moduleGrantAttestationHash $signatureHex"
    Send-AgentCommand -Command $moduleAttestationCommand -ExpectedMarker "RAIOS_AGENT_END module.attestation_diagnostic" -Name "m6c:signed_attestation_reference"
    $moduleAttestationResponse = Get-LastAgentResponseJson -Method "module.attestation_diagnostic"
    $attestationResult = $moduleAttestationResponse
    $liveSignatureVerified = (
        ($attestationResult.evidence | Where-Object id -eq "local_attestation").facts.status_detail -eq "local_attestation_signature_verified_load_still_denied" -and
        ($attestationResult.evidence | Where-Object id -eq "local_attestation").facts.signature_verified -eq $true
    )
    Add-Predicate -Name "m6c:dev_signature_live_check_outcome" -Expected "the Rust dev signer produces a guest-verified exact attestation" -Passed $liveSignatureVerified -Actual $(if ($liveSignatureVerified) { "live_signature_verified_end_to_end" } else { "guest_rejected_rust_signer_signature" })
    if (-not $liveSignatureVerified) {
        throw "Expected the Rust dev signer attestation to verify in the guest"
    }
}

Send-AgentCommand -Command "agent module.granted_candidate_selftest" -ExpectedMarker "RAIOS_AGENT_END module.granted_candidate_selftest" -Name "m6c:granted_candidate_selftest"
$selftest = Get-LastAgentResponseJson -Method "module.granted_candidate_selftest"
$selftestOk = (
    $selftest.body.result.passed -eq $true -and
    [int]$selftest.body.result.case_count -eq 8
)
Add-Predicate -Name "m6c:in_guest_selftest_positive_and_negative_gates" -Expected "selftest proves granted run, ungranted denial, hash-mismatch denial, and live projection truth cases" -Passed $selftestOk -Actual $(if ($selftestOk) { "matched" } else { ($selftest.body.result | ConvertTo-Json -Compress -Depth 8) })
if (-not $selftestOk) {
    throw "Expected granted_candidate selftest to pass"
}
$selftestCases = @($selftest.body.result.cases)
$projectionLoadedCase = @($selftestCases | Where-Object { $_.case -eq "live_load_projection_loaded_snapshot" })[0]
$projectionLoaded = $projectionLoadedCase.live_load_projection
$projectionLoadedOk = (
    $projectionLoadedCase.passed -eq $true -and
    $projectionLoaded.present -eq $true -and
    $projectionLoaded.accepts_external_artifact_bytes -eq $true -and
    $projectionLoaded.loads_artifact -eq $true -and
    $projectionLoaded.can_load_now -eq $true -and
    $projectionLoaded.service_slot_allocated -eq $true -and
    $projectionLoaded.running -eq $true -and
    $projectionLoaded.run_outcome -eq "success" -and
    $projectionLoaded.trust_tier -eq "dev_key_not_owner_sealed" -and
    $projectionLoaded.load_mechanism -eq "wasmi_interpreter_ram_only" -and
    $projectionLoaded.maps_executable_pages -eq $false -and
    $projectionLoaded.durable -eq $false -and
    $projectionLoaded.owner_sealed -eq $false -and
    $projectionLoaded.authorizes_native_guest_load -eq $false
)
Add-Predicate -Name "m6c:projection_selftest_loaded_truth" -Expected "loaded projection positives true, guardrails false, trust_tier dev_key_not_owner_sealed" -Passed $projectionLoadedOk -Actual $(if ($projectionLoadedOk) { "matched" } else { ($projectionLoadedCase | ConvertTo-Json -Compress -Depth 8) })
if (-not $projectionLoadedOk) {
    throw "Expected loaded live projection selftest case"
}
$projectionAbsentCase = @($selftestCases | Where-Object { $_.case -eq "live_load_projection_not_loaded" })[0]
$projectionAbsent = $projectionAbsentCase.live_load_projection
$projectionAbsentOk = (
    $projectionAbsentCase.passed -eq $true -and
    $projectionAbsent.present -eq $false -and
    $projectionAbsent.accepts_external_artifact_bytes -eq $false -and
    $projectionAbsent.loads_artifact -eq $false -and
    $projectionAbsent.can_load_now -eq $false -and
    $projectionAbsent.service_slot_allocated -eq $false -and
    $projectionAbsent.running -eq $false -and
    $projectionAbsent.maps_executable_pages -eq $false -and
    $projectionAbsent.durable -eq $false -and
    $projectionAbsent.owner_sealed -eq $false -and
    $projectionAbsent.authorizes_native_guest_load -eq $false
)
Add-Predicate -Name "m6c:projection_selftest_not_loaded_false" -Expected "not-loaded projection keeps every projected boolean false" -Passed $projectionAbsentOk -Actual $(if ($projectionAbsentOk) { "matched" } else { ($projectionAbsentCase | ConvertTo-Json -Compress -Depth 8) })
if (-not $projectionAbsentOk) {
    throw "Expected not-loaded live projection selftest case"
}

if ($liveSignatureVerified) {
    Send-AgentCommand -Command $moduleGrantCommand -ExpectedMarker "RAIOS_AGENT_END module.grant_diagnostic" -Name "m6c:grant_can_load_now"
    $grantReady = Get-LastAgentResponseJson -Method "module.grant_diagnostic"
    $grantReadyResult = $grantReady
    $grantReadyOk = (
        $grantReadyResult.decision.outcome -eq "granted" -and
        @($grantReadyResult.decision.grants) -contains "cap.module.load_ephemeral" -and
        $grantReadyResult.facts.trust_tier -eq "dev_key_not_owner_sealed"
    )
    Add-Predicate -Name "m6c:grant_reports_dev_tier_can_load_now" -Expected "grants_capability=true trust_tier=dev_key_not_owner_sealed can_load_now=true" -Passed $grantReadyOk -Actual $(if ($grantReadyOk) { "matched" } else { ($grantReadyResult | ConvertTo-Json -Compress -Depth 6) })
    if (-not $grantReadyOk) {
        throw "Expected signed grant with retained bytes to report dev-tier can_load_now"
    }

    $loadOffset = Get-SerialLogOffset
    Send-AgentCommand -Command "module.load_ephemeral svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "m6c:granted_candidate_load"
    $load = Get-LastAgentResponseJson -Method "module.load_ephemeral"
    $loadResult = $load.body.result
    $loadBinding = $loadResult.source_binding
    $grantReadyEventId = [string]($grantReady.evidence | Where-Object id -eq "computed_capability_grant_retained").source_event_id
    $loadOk = (
        $load.t -eq "response" -and
        $loadResult.schema -eq "raios.ram_only_granted_candidate_service.lifecycle_response.v0" -and
        $loadResult.service_id -eq "svc.dev.granted_candidate" -and
        $loadResult.action -eq "load_prepare" -and
        $loadResult.code -eq "ok" -and
        $loadResult.reason -eq "physical_approval_pending" -and
        $loadResult.activation_authority -eq "genesis_pointer_required" -and
        $loadResult.physical_approval_pending -eq $true -and
        $loadResult.physical_approval_consumed -eq $false -and
        $loadResult.approval_challenge_sha256 -match '^sha256:[0-9a-f]{64}$' -and
        $loadResult.loaded -eq $false -and $loadResult.running -eq $false -and
        [int]$loadResult.run_count -eq 0 -and
        $loadResult.scope -eq "current_boot" -and
        $loadResult.trust_tier -eq "dev_key_not_owner_sealed" -and
        $loadBinding.present -eq $true -and $loadBinding.source_kind -eq "serial" -and
        $loadBinding.candidate_sha256 -eq "sha256:$expectedSha" -and
        $loadBinding.receipt_sha256 -eq $null -and $loadBinding.w7.present -eq $false -and
        $loadBinding.receiver_identity.present -eq $false -and
        $loadBinding.computed_grant.event_id -eq $grantReadyEventId -and
        $loadBinding.computed_grant.computed_grant_hash -eq "sha256:$moduleGrantHash" -and
        $loadBinding.computed_grant.artifact_hash -eq "sha256:$expectedSha" -and
        $loadBinding.local_attestation.event_id -match '^event\.current_boot\.[0-9]{8}$' -and
        $loadBinding.local_attestation.attestation_reference_hash -eq "sha256:$moduleAttestationReferenceHash" -and
        $loadBinding.local_attestation.signature_verified -eq $true -and
        $loadResult.grant_authority.evidence_authorized -eq $true -and
        $loadResult.grant_authority.physical_authority_satisfied -eq $false -and
        $loadResult.grant_authority.can_load_now -eq $false -and
        $loadResult.accepts_external_artifact_bytes -eq $true -and
        $loadResult.loads_external_artifact -eq $false -and
        $loadResult.maps_executable_pages -eq $false -and
        $loadResult.writes_persistent_state -eq $false -and
        $loadResult.authorizes_persistent_install -eq $false -and
        $loadResult.authorizes_rollback_install -eq $false
    )
    Add-Predicate -Name "m6c:granted_candidate_prepares_physical_activation" -Expected "exact M6 evidence creates one serial-source Genesis preview without load, slot, run, or durable effect" -Passed $loadOk -Actual $(if ($loadOk) { "challenge=$($loadResult.approval_challenge_sha256)" } else { ($loadResult | ConvertTo-Json -Compress -Depth 10) })
    if (-not $loadOk) {
        throw "Expected exact M6 evidence to prepare physical activation only"
    }

    $durablePromotion = $loadResult.durable_promotion_transaction
    $durablePromotionOk = (
        $durablePromotion.code -eq "capability_denied" -and
        $durablePromotion.performed -eq $false -and
        $durablePromotion.transaction_kind -eq "promote" -and
        $durablePromotion.persistence_claimed -eq $false -and
        $loadResult.durable_artifact_persist.performed -eq $false -and
        $loadResult.service_slot_activation.active -eq $false -and
        -not ((Get-SerialLogContent -Path $SerialLog).Substring([int]$loadOffset)).Contains("WASM_GUEST_LOG echo counter=") -and
        -not ((Get-SerialLogContent -Path $SerialLog).Substring([int]$loadOffset)).Contains("GRANTED_CANDIDATE_CURRENT_BOOT_ACTIVATION")
    )
    Add-Predicate -Name "m6c:pending_preview_has_zero_effect" -Expected "pending preview performs no slot, run, guest log, activation, promotion transaction, or artifact persist" -Passed $durablePromotionOk -Actual $(if ($durablePromotionOk) { "inert" } else { ($loadResult | ConvertTo-Json -Compress -Depth 8) })
    if (-not $durablePromotionOk) { throw "Expected the M6 physical preview to stay inert" }

    Send-AgentCommand -Command "service.start svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "m6c:serial_start_cannot_approve"
    $serialStart = (Get-LastAgentResponseJson -Method "service.start").body.result
    $serialStartOk = (
        $serialStart.code -eq "capability_denied" -and
        $serialStart.reason -eq "physical_approval_genesis_pointer_required" -and
        $serialStart.activation_authority -eq "genesis_pointer_required" -and
        $serialStart.physical_approval_pending -eq $true -and
        $serialStart.physical_approval_consumed -eq $false -and
        $serialStart.loaded -eq $false -and $serialStart.running -eq $false -and
        [int]$serialStart.run_count -eq 0 -and $serialStart.run_evidence.present -eq $false
    )
    Add-Predicate -Name "m6c:serial_start_cannot_substitute_for_pointer" -Expected "serial service.start leaves the same pending preview inert" -Passed $serialStartOk -Actual $(if ($serialStartOk) { "denied; pending" } else { $serialStart | ConvertTo-Json -Compress -Depth 8 })
    if (-not $serialStartOk) { throw "Expected serial service.start to remain denied before pointer approval" }

    $activationNeedle = "GRANTED_CANDIDATE_CURRENT_BOOT_ACTIVATION result=accepted physical_approval=genesis_pointer source_kind=serial candidate_sha256=sha256:$expectedSha receipt_sha256=none run_count=1 outcome=success durable_writes=false"
    Send-QemuAbsolutePointerClick -X 27017 -Y 6559
    $activated = Wait-ForLogTextAfterOffset -Path $SerialLog -Needle $activationNeedle -Offset $loadOffset -TimeoutSeconds $TimeoutSeconds
    $afterActivation = (Get-SerialLogContent -Path $SerialLog).Substring([int]$loadOffset)
    $activationCount = ([regex]::Matches($afterActivation, [regex]::Escape("GRANTED_CANDIDATE_CURRENT_BOOT_ACTIVATION"))).Count
    $guestLogCount = ([regex]::Matches($afterActivation, [regex]::Escape("WASM_GUEST_LOG echo counter="))).Count
    $physicalOk = $activated -and $activationCount -eq 1 -and $guestLogCount -eq 1
    Add-Predicate -Name "m6c:one_physical_click_runs_exactly_once" -Expected "one QMP Genesis click emits one accepted marker and one guest log" -Passed $physicalOk -Actual "activated=$activated activation_count=$activationCount guest_log_count=$guestLogCount"
    if (-not $physicalOk) { throw "Expected one Genesis pointer click to run the candidate exactly once" }

    Send-AgentCommand -Command "service.start svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "m6c:serial_start_replay"
    $startReplay = (Get-LastAgentResponseJson -Method "service.start").body.result
    Send-AgentCommand -Command "module.load_ephemeral svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "m6c:serial_load_replay"
    $loadReplay = (Get-LastAgentResponseJson -Method "module.load_ephemeral").body.result
    $afterReplay = (Get-SerialLogContent -Path $SerialLog).Substring([int]$loadOffset)
    $replayOk = $startReplay.code -eq "capability_denied" -and
        $startReplay.reason -eq "physical_approval_already_consumed" -and
        $startReplay.running -eq $true -and [int]$startReplay.run_count -eq 1 -and
        $startReplay.physical_approval_pending -eq $false -and $startReplay.physical_approval_consumed -eq $true -and
        $startReplay.last_run_evidence.run_outcome -eq "success" -and
        [int64]$startReplay.last_run_evidence.fuel_used -gt 0 -and $startReplay.last_run_evidence.log_line_emitted -eq $true -and
        $loadReplay.code -eq "capability_denied" -and $loadReplay.reason -eq "service_already_loaded" -and
        $loadReplay.running -eq $true -and [int]$loadReplay.run_count -eq 1 -and
        ([regex]::Matches($afterReplay, [regex]::Escape("GRANTED_CANDIDATE_CURRENT_BOOT_ACTIVATION"))).Count -eq 1 -and
        ([regex]::Matches($afterReplay, [regex]::Escape("WASM_GUEST_LOG echo counter="))).Count -eq 1
    Add-Predicate -Name "m6c:serial_replays_do_not_rerun" -Expected "serial start/load replays keep the physical approval consumed and run_count/marker/log at one" -Passed $replayOk -Actual $(if ($replayOk) { "run_count=1 marker=1 log=1" } else { @{ start = $startReplay; load = $loadReplay } | ConvertTo-Json -Compress -Depth 8 })
    if (-not $replayOk) { throw "Expected serial replays to remain non-executing after physical activation" }

    # P4-3b2 ruling: module.loader_runtime is a v1 denial envelope and the live
    # granted-candidate projection is intentionally ABSENT from loader responses
    # (it belongs to the granted-candidate family, verified by the replay/slot/
    # inventory predicates around this block). Native readiness must stay denied
    # with zero grants even while the physically approved candidate is running.
    Send-AgentCommand -Command "agent module.loader_runtime" -ExpectedMarker "RAIOS_AGENT_END module.loader_runtime" -Name "m6c:live_loader_runtime_projection"
    $loaderRuntime = Get-LastAgentResponseJson -Method "module.loader_runtime"
    $loaderEvidence = @($loaderRuntime.evidence)
    $loaderApproval = @($loaderRuntime.evidence | Where-Object id -eq "local_approval_reference")[0]
    $loaderProjectionOk = (
        $loaderRuntime.schema -eq "raios.evidence_response.v1" -and
        $loaderRuntime.family -eq "module.loader_runtime" -and
        $loaderRuntime.scope -eq "current_boot" -and
        $loaderRuntime.classification -eq "local_only" -and
        $loaderRuntime.source_method -eq "module.loader_runtime" -and
        $null -eq $loaderRuntime.event_id -and
        $loaderEvidence.Count -eq 54 -and
        $loaderEvidence[0].id -eq "manifest_reference" -and
        $loaderEvidence[53].id -eq "executable_entrypoint_invocation_boundary" -and
        $loaderRuntime.PSObject.Properties.Name -notcontains "live_granted_load_projection" -and
        @($loaderRuntime.evidence | Where-Object id -eq "live_granted_load_projection").Count -eq 0 -and
        @($loaderRuntime.evidence | Where-Object id -eq "manifest_reference")[0].facts.present -eq $true -and
        @($loaderRuntime.evidence | Where-Object id -eq "artifact_reference")[0].facts.present -eq $true -and
        @($loaderRuntime.evidence | Where-Object id -eq "vm_report_reference")[0].facts.present -eq $true -and
        @($loaderRuntime.evidence | Where-Object id -eq "local_attestation_reference")[0].facts.present -eq $true -and
        $loaderApproval.facts.present -eq $false -and
        $loaderApproval.facts.status_detail -eq "missing" -and
        $loaderRuntime.decision.outcome -eq "denied" -and
        $loaderRuntime.decision.reason -eq "retained_module_local_approval_reference_missing" -and
        $loaderRuntime.decision.requested_capability -eq "cap.module.load_ephemeral" -and
        @($loaderRuntime.decision.grants).Count -eq 0 -and
        @($loaderRuntime.decision.effects).Count -eq 0 -and
        @($loaderRuntime.decision.blocked_by)[0].evidence_id -eq "local_approval_reference"
    )
    Add-Predicate -Name "m6c:live_loader_runtime_projection_reflects_granted_run" -Expected "loader-runtime v1 envelope stays denied with zero grants and no live_granted_load_projection while the granted run is visible only in its own family (P4-3b2)" -Passed $loaderProjectionOk -Actual $(if ($loaderProjectionOk) { "denied; projection absent; evidence=54" } else { ($loaderRuntime | ConvertTo-Json -Compress -Depth 8) })
    if (-not $loaderProjectionOk) { throw "Expected the loader-runtime envelope to deny native readiness without a granted-load projection" }

    Send-AgentCommand -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic" -Name "m6c:live_service_slot_projection"
    $slotDiagnostic = Get-LastAgentResponseJson -Method "module.service_slot_diagnostic"
    $liveSlot = @($slotDiagnostic.evidence | Where-Object id -eq "live_granted_service_slot")[0].facts
    $liveSlotOk = (
        $liveSlot.service_id -eq "svc.dev.granted_candidate" -and
        $liveSlot.ram_only_service_slot_id -eq "ram_only:svc.dev.granted_candidate" -and
        $liveSlot.service_slot_allocated -eq $true -and
        $liveSlot.running -eq $true -and
        $liveSlot.trust_tier -eq "dev_key_not_owner_sealed" -and
        $liveSlot.load_mechanism -eq "wasmi_interpreter_ram_only" -and
        $liveSlot.maps_executable_pages -eq $false -and
        $liveSlot.durable -eq $false -and
        $liveSlot.owner_sealed -eq $false -and
        $slotDiagnostic.decision.outcome -eq "denied"
    )
    Add-Predicate -Name "m6c:live_service_slot_diagnostic_reports_allocated_granted_slot" -Expected "live_granted_service_slot evidence shows allocated RAM slot while decision remains denied" -Passed $liveSlotOk -Actual $(if ($liveSlotOk) { "matched" } else { ($slotDiagnostic | ConvertTo-Json -Compress -Depth 6) })

    Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "m6c:live_service_inventory_projection"
    $inventory = Get-LastAgentResponseJson -Method "service.inventory"
    $grantedInventory = @($inventory.facts.services | Where-Object { $_.id -eq "svc.dev.granted_candidate" })[0]
    $inventoryOk = (
        $grantedInventory.kind -eq "service" -and
        $grantedInventory.health -eq "healthy" -and
        $grantedInventory.scope -eq "current_boot" -and
        $grantedInventory.persistence -eq "none" -and
        $grantedInventory.classification -eq "local_only" -and
        $grantedInventory.trust_tier -eq "dev_key_not_owner_sealed" -and
        $grantedInventory.ram_only_service_slot_id -eq "ram_only:svc.dev.granted_candidate" -and
        $grantedInventory.service_slot_activation_active -eq $true -and
        $grantedInventory.running -eq $true -and
        $grantedInventory.last_run_outcome -eq "success"
    )
    Add-Predicate -Name "m6c:live_service_inventory_lists_granted_candidate" -Expected "service.inventory lists running dev-tier granted candidate" -Passed $inventoryOk -Actual $(if ($inventoryOk) { "matched" } else { ($inventory.facts | ConvertTo-Json -Compress -Depth 5) })

    Send-AgentCommand -Command "service.drop svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.drop" -Name "m6c:granted_candidate_drop"
}
else {
    Add-Predicate -Name "m6c:live_signature_positive_profile_gap" -Expected "live signed grant path available" -Passed $true -Actual "live signature did not verify; positive run proof is the in-guest selftest"
}

Send-AgentCommand -Command "module.load_ephemeral" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "m6c:generic_durable_gate_preserved"
$generic = Get-LastAgentResponseJson -Method "module.load_ephemeral"
$genericAudit = $generic.evidence | Where-Object { $_.id -eq "durable_audit_record" }
$genericRollback = $generic.evidence | Where-Object { $_.id -eq "rollback_plan" }
$genericOk = (
    $generic.schema -eq "raios.evidence_response.v1" -and
    $generic.decision.outcome -eq "denied" -and
    $genericAudit.facts.status_detail -eq "missing" -and
    $genericRollback.facts.status_detail -eq "missing" -and
    @($generic.decision.grants).Count -eq 0 -and
    @($generic.decision.effects).Count -eq 0
)
Add-Predicate -Name "m6c:generic_durable_load_gate_stays_denied" -Expected "durable/owner-sealed load gate remains denied with no grants or effects" -Passed $genericOk -Actual $(if ($genericOk) { "matched" } else { ($generic | ConvertTo-Json -Compress -Depth 8) })
if (-not $genericOk) {
    throw "Expected generic durable module.load_ephemeral gate to remain denied"
}
