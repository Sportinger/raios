if (-not $Network) { throw "m6d-rollback requires -Network" }

$activation = @(Invoke-W7M6PhysicalActivation)[-1]
$activationBinding = $activation.ActivationResponse.source_binding
$activationLifecycle = $activation.LastLifecycleResponse
$activationOk = $activation.CandidateSha256 -match '^sha256:[0-9a-f]{64}$' -and
    $activation.W7ReceiptSha256 -match '^sha256:[0-9a-f]{64}$' -and
    $activation.ActivationChallengeSha256 -match '^sha256:[0-9a-f]{64}$' -and
    $activationBinding.source_kind -eq "w7" -and
    $activationBinding.candidate_sha256 -eq $activation.CandidateSha256 -and
    $activationBinding.receipt_sha256 -eq $activation.W7ReceiptSha256 -and
    $activationBinding.receiver_identity.content_sha256 -eq $activation.CandidateSha256 -and
    $activationBinding.receiver_identity.retained_candidate_sha256 -eq $activation.CandidateSha256 -and
    $activationBinding.receiver_identity.catalog_finalize_candidate_sha256 -eq $activation.CandidateSha256 -and
    $activationBinding.receiver_identity.receiver_identity_complete -eq $true -and
    $activationBinding.receiver_identity.guest_signature_verification_performed -eq $true -and
    $activationBinding.receiver_identity.exact_candidate_catalog_binding -eq $true -and
    $activationBinding.computed_grant.event_id -eq $activation.M6GrantEventId -and
    $activationBinding.computed_grant.computed_grant_hash -eq $activation.M6ComputedGrantSha256 -and
    $activationBinding.computed_grant.manifest_hash -eq $activation.M6ManifestSha256 -and
    $activationBinding.computed_grant.artifact_hash -eq $activation.CandidateSha256 -and
    $activationBinding.computed_grant.vm_test_report_hash -eq $activation.M6VmReportSha256 -and
    $activationBinding.computed_grant.local_attestation_hash -eq $activation.M6LocalAttestationSha256 -and
    $activationBinding.local_attestation.event_id -eq $activation.M6AttestationEventId -and
    $activationBinding.local_attestation.manifest_reference_hash -eq $activation.M6ManifestReferenceSha256 -and
    $activationBinding.local_attestation.artifact_reference_hash -eq $activation.M6ArtifactReferenceSha256 -and
    $activationBinding.local_attestation.vm_report_reference_hash -eq $activation.M6VmReportReferenceSha256 -and
    $activationBinding.local_attestation.attestation_reference_hash -eq $activation.M6AttestationReferenceSha256 -and
    $activationBinding.local_attestation.signature_verified -eq $true -and
    $activationLifecycle.code -eq "capability_denied" -and
    $activationLifecycle.reason -eq "service_already_loaded" -and
    $activationLifecycle.loaded -eq $true -and $activationLifecycle.running -eq $true -and
    [int]$activationLifecycle.run_count -eq 1 -and
    $activationLifecycle.physical_approval_pending -eq $false -and
    $activationLifecycle.physical_approval_consumed -eq $true -and
    $activationLifecycle.durable_promotion_transaction.performed -eq $false -and
    $activationLifecycle.durable_artifact_persist.performed -eq $false
Add-Predicate -Name "m6d:01_b12b_w7_physical_run_established" -Expected "the network-acquisition prefix establishes the exact W7/receiver/M6 binding and one physically accepted current-boot run with run_count=1 and no durable write" -Passed $activationOk -Actual $(if ($activationOk) { "candidate=$($activation.CandidateSha256) activation=$($activation.ActivationChallengeSha256) run_count=1" } else { $activation | ConvertTo-Json -Compress -Depth 24 })
if (-not $activationOk) { throw "m6d B1.2b W7 physical run was not established" }

Send-AgentCommand -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "m6d:preinstall-reclog-before"
$preinstallReclogBeforeResponse = Get-LastAgentResponseJson -Method "durable.record_log_scan"
$preinstallReclogBefore = $preinstallReclogBeforeResponse.body.result
Send-AgentCommand -Command "service.rollback_preview svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.rollback_preview" -Name "m6d:preinstall-rollback-preview"
$preinstallPreviewResponse = Get-LastAgentResponseJson -Method "service.rollback_preview"
$preinstallPreview = $preinstallPreviewResponse.body.result
Send-AgentCommand -Command "service.rollback_apply svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.rollback_apply" -Name "m6d:preinstall-rollback-apply"
$preinstallApplyResponse = Get-LastAgentResponseJson -Method "service.rollback_apply"
$preinstallApply = $preinstallApplyResponse.body.result
Send-AgentCommand -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "m6d:preinstall-reclog-after"
$preinstallReclogAfterResponse = Get-LastAgentResponseJson -Method "durable.record_log_scan"
$preinstallReclogAfter = $preinstallReclogAfterResponse.body.result
$preinstallDeniedOk = $preinstallPreview.code -eq "capability_denied" -and
    $preinstallPreview.reason -eq "no_recorded_promotion_to_roll_back" -and
    $preinstallPreview.rollback_plan.present -eq $false -and
    $preinstallPreview.rollback_apply_authorized -eq $false -and
    $preinstallApply.code -eq "capability_denied" -and
    $preinstallApply.reason -eq "no_recorded_promotion_to_roll_back" -and
    $preinstallApply.rollback_apply_authorized -eq $false -and
    $preinstallApply.rollback_verification.durable_unpromote_transaction.performed -eq $false -and
    $preinstallApply.rollback_verification.durable_unpromote_transaction.reason -eq "no_recorded_promotion_to_roll_back" -and
    [int64]$preinstallReclogAfter.count -eq [int64]$preinstallReclogBefore.count -and
    $preinstallReclogAfter.head_frame_sha256 -eq $preinstallReclogBefore.head_frame_sha256 -and
    $preinstallReclogAfter.tail_frame_sha256 -eq $preinstallReclogBefore.tail_frame_sha256 -and
    $activationLifecycle.loaded -eq $true -and $activationLifecycle.running -eq $true -and
    [int]$activationLifecycle.run_count -eq 1
$preinstallDump = [ordered]@{ preview = $preinstallPreviewResponse; apply = $preinstallApplyResponse; reclog_before = $preinstallReclogBeforeResponse; reclog_after = $preinstallReclogAfterResponse }
Add-Predicate -Name "m6d:02_preinstall_rollback_denied" -Expected "rollback preview/apply both deny with no_recorded_promotion_to_roll_back before W6 and leave RECLOG and the one running service unchanged" -Passed $preinstallDeniedOk -Actual $(if ($preinstallDeniedOk) { "denied; RECLOG unchanged" } else { $preinstallDump | ConvertTo-Json -Compress -Depth 20 })
if (-not $preinstallDeniedOk) { throw "m6d preinstall rollback did not fail closed" }

$install = @(Invoke-SignedGrantedCandidateInstall -Activation $activation -NamePrefix "m6d")[-1]
$preview = $install.Preview
$prepareOk = $preview.status -eq "accepted" -and
    $preview.reason -eq "project_install_signature_required" -and
    $preview.phase -eq "pending_owner_signature" -and
    $preview.action_kind -eq "install" -and $preview.signature_verified -eq $false -and
    $preview.install_source -eq "granted_candidate" -and
    $preview.receipt_kind -eq "w7_acquisition" -and $preview.w4_project_receipt_present -eq $false -and
    $preview.candidate_sha256 -eq $activation.CandidateSha256 -and
    $preview.receipt_sha256 -eq $activation.W7ReceiptSha256 -and
    $preview.activation_approval_sha256 -eq $activation.ActivationChallengeSha256 -and
    $preview.install_envelope_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    $null -eq $preview.promotion_transaction_sha256 -and
    $null -eq $preview.artifact_persist_frame_sha256 -and
    $preview.writes_persistent_state -eq $false
Add-Predicate -Name "m6d:03_w6_prepare_exact_no_w4_receipt" -Expected "W6 prepare binds the same W7 candidate, receipt, and activation challenge with receipt_kind=w7_acquisition, no W4 receipt, signature required, and no write" -Passed $prepareOk -Actual $(if ($prepareOk) { "envelope=$($preview.install_envelope_sha256) signature required" } else { $install.PrepareResponse | ConvertTo-Json -Compress -Depth 20 })
if (-not $prepareOk) { throw "m6d W6 prepare did not bind the exact W7 activation" }

$signed = $install.SignedPreview
$signatureOk = $signed.status -eq "accepted" -and
    $signed.reason -eq "project_install_pending_physical_pointer_approval" -and
    $signed.phase -eq "pending_physical_pointer_approval" -and
    $signed.signature_verified -eq $true -and
    $signed.action_signature_message_sha256 -eq $preview.action_signature_message_sha256 -and
    $signed.action_signature_message_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    $signed.action_signature_message_sha256 -ne $activation.M6AttestationReferenceSha256 -and
    $signed.physical_approval_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    $signed.install_envelope_sha256 -eq $preview.install_envelope_sha256 -and
    $signed.writes_persistent_state -eq $false
Add-Predicate -Name "m6d:04_w6_dev_signature_verified" -Expected "the shared Rust dev signer verifies a distinct nonzero W6 action digest and arms a nonzero Genesis physical-approval hash without writing" -Passed $signatureOk -Actual $(if ($signatureOk) { "action=$($signed.action_signature_message_sha256) approval=$($signed.physical_approval_sha256)" } else { $install.SignatureResponse | ConvertTo-Json -Compress -Depth 20 })
if (-not $signatureOk) { throw "m6d W6 dev signature was not verified" }

$denial = $install.Denial
$serialDeniedOk = $denial.status -eq "denied" -and
    $denial.reason -eq "project_install_physical_pointer_approval_required" -and
    $denial.phase -eq "pending_physical_pointer_approval" -and
    $denial.signature_verified -eq $true -and
    $denial.action_signature_message_sha256 -eq $signed.action_signature_message_sha256 -and
    $denial.physical_approval_sha256 -eq $signed.physical_approval_sha256 -and
    $denial.install_envelope_sha256 -eq $signed.install_envelope_sha256 -and
    [int64]$install.DeniedReclogScan.count -eq [int64]$install.PreReclogScan.count -and
    $install.DeniedReclogScan.head_frame_sha256 -eq $install.PreReclogScan.head_frame_sha256 -and
    $install.DeniedReclogScan.tail_frame_sha256 -eq $install.PreReclogScan.tail_frame_sha256 -and
    [int64]$install.DeniedArtifactScan.artifact_persist_record_count -eq [int64]$install.PreArtifactScan.artifact_persist_record_count -and
    -not $install.BeforeClickLog.Contains("GRANTED_CANDIDATE_INSTALL_COMMIT")
$serialDeniedDump = [ordered]@{ denial = $install.DenialResponse; before_reclog = $install.PreReclogResponse; denied_reclog = $install.DeniedReclogResponse; before_artstor = $install.PreArtifactResponse; denied_artstor = $install.DeniedArtifactResponse }
Add-Predicate -Name "m6d:05_serial_install_approval_denied" -Expected "serial approval denies with project_install_physical_pointer_approval_required, preserves the signed pending preview, and leaves RECLOG/ARTSTOR unchanged" -Passed $serialDeniedOk -Actual $(if ($serialDeniedOk) { "signed preview pending; RECLOG/ARTSTOR unchanged" } else { $serialDeniedDump | ConvertTo-Json -Compress -Depth 20 })
if (-not $serialDeniedOk) { throw "m6d serial W6 approval crossed the pointer boundary" }

$commitOk = $install.CandidateSha256 -eq $activation.CandidateSha256 -and
    $install.W7ReceiptSha256 -eq $activation.W7ReceiptSha256 -and
    $install.ActivationApprovalSha256 -eq $activation.ActivationChallengeSha256 -and
    $install.InstallEnvelopeSha256 -eq $signed.install_envelope_sha256 -and
    $install.InstallActionSha256 -match '^sha256:[0-9a-f]{64}$' -and
    $install.PromotionTransactionSha256 -match '^sha256:[0-9a-f]{64}$' -and
    $install.ArtifactPersistFrameSha256 -match '^sha256:[0-9a-f]{64}$' -and
    $install.Generation -eq [int64]$signed.generation -and
    $install.Sequence -eq [int64]$signed.log_sequence -and
    $install.RunCount -eq 1 -and
    $install.TrustTier -eq "dev_key_not_owner_sealed" -and $install.DurableWrites -eq $true
Add-Predicate -Name "m6d:06_second_physical_click_commits" -Expected "the accepted install marker carries every exact W7/activation/W6 hash, generation/sequence, run_count=1, dev trust tier, and durable_writes=true" -Passed $commitOk -Actual $(if ($commitOk) { $install.MarkerLine } else { $install | ConvertTo-Json -Compress -Depth 20 })
if (-not $commitOk) { throw "m6d second physical approval did not commit the exact running candidate" }

Send-AgentCommand -Command "project.install_status" -ExpectedMarker "RAIOS_AGENT_END project.install_status" -Name "m6d:postinstall-preview-consumed"
$postInstallStatusResponse = Get-LastAgentResponseJson -Method "project.install_status"
$postInstallStatus = $postInstallStatusResponse.body.result
$approvalOk = $activation.ActivationChallengeSha256 -ne $install.PhysicalApprovalSha256 -and
    ($activation.ClickCount + $install.ClickCount) -eq 3 -and
    $activationLifecycle.physical_approval_consumed -eq $true -and
    $activationLifecycle.physical_approval_pending -eq $false -and
    $postInstallStatus.phase -eq "idle" -and
    $postInstallStatus.signature_verified -eq $false -and
    $null -eq $postInstallStatus.action_signature_message_sha256 -and
    $postInstallStatus.writes_persistent_state -eq $false
$approvalDump = [ordered]@{ activation = $activation; install = $install; post_install_status = $postInstallStatusResponse }
Add-Predicate -Name "m6d:07_two_distinct_consumed_approvals" -Expected "M6 activation challenge differs from W6 physical approval; three QMP clicks are honest (one stale denial plus two separately consumed approvals), and both consumable previews are gone" -Passed $approvalOk -Actual $(if ($approvalOk) { "qmp_clicks=3 consumable_approvals=2 activation=$($activation.ActivationChallengeSha256) install=$($install.PhysicalApprovalSha256)" } else { $approvalDump | ConvertTo-Json -Compress -Depth 24 })
if (-not $approvalOk) { throw "m6d did not consume two distinct physical approvals" }

$artifactRecord = @($install.PostArtifactScan.records | Where-Object {
    $_.artifact_sha256 -eq $activation.CandidateSha256 -and
    $_.promotion_transaction_sha256 -eq $install.PromotionTransactionSha256
})[0]
$m6Attestation = @($activation.M6AttestationResponse.evidence | Where-Object id -eq "local_attestation")[0]
$promoteOk = $install.PostReclogScan.status -eq "valid" -and
    $install.PostReclogScan.valid_prefix_chain -eq $true -and
    [int64]$install.PostReclogScan.count -eq ([int64]$install.PreReclogScan.count + 3) -and
    [int64]$install.Sequence -eq ([int64]$install.PreReclogScan.tail_seq + 2) -and
    [int64]$artifactRecord.seq -eq ([int64]$install.Sequence + 1) -and
    $install.PostReclogScan.tail_frame_sha256 -eq $install.ArtifactPersistFrameSha256 -and
    $m6Attestation.facts.signature_verified -eq $true -and
    $install.SignedPreview.signature_verified -eq $true -and
    $install.ActivationApprovalSha256 -eq $activation.ActivationChallengeSha256 -and
    $install.InstallEnvelopeSha256 -eq $preview.install_envelope_sha256 -and
    $install.InstallActionSha256 -match '^sha256:[0-9a-f]{64}$'
$promoteDump = [ordered]@{ before = $install.PreReclogResponse; after = $install.PostReclogResponse; marker = $install.MarkerLine; artifact_record = $artifactRecord; m6 = $activation.M6AttestationResponse; w6 = $install.SignatureResponse }
Add-Predicate -Name "m6d:08_durable_promote_provenance" -Expected "RECLOG adds the three consecutive frame-split records (install authorization at marker sequence, linked promote, artifact-persist tail); M6/W6 signatures and activation/envelope/action marker hashes are exact" -Passed $promoteOk -Actual $(if ($promoteOk) { "authorization_seq=$($install.Sequence) promote=$($install.PromotionTransactionSha256) artifact_persist=$($install.ArtifactPersistFrameSha256)" } else { $promoteDump | ConvertTo-Json -Compress -Depth 24 })
if (-not $promoteOk) { throw "m6d durable promote provenance was not exact" }

$artifactOk = $install.PostArtifactScan.reclog_read_completed -eq $true -and
    [int64]$install.PostArtifactScan.artifact_persist_record_count -eq ([int64]$install.PreArtifactScan.artifact_persist_record_count + 1) -and
    $artifactRecord.present -eq $true -and $artifactRecord.blob_hash_verified -eq $true -and
    $artifactRecord.artifact_sha256 -eq $activation.CandidateSha256 -and
    $artifactRecord.parsed_payload_sha256 -eq $activation.CandidateSha256 -and
    $artifactRecord.artstor_blob_frame_sha256 -eq $artifactRecord.computed_blob_frame_sha256 -and
    [int64]$artifactRecord.artstor_blob_len -gt 4205 -and
    $artifactRecord.promotion_transaction_sha256 -eq $install.PromotionTransactionSha256 -and
    $artifactRecord.record_authorizes_load -eq $false -and $artifactRecord.authorizes_load -eq $false
Add-Predicate -Name "m6d:09_artstor_persist_linked" -Expected "artifact.store_scan reads and hash-verifies the exact candidate blob and links its sole new record to the exact promote frame without granting load" -Passed $artifactOk -Actual $(if ($artifactOk) { "candidate=$($artifactRecord.artifact_sha256) promote=$($artifactRecord.promotion_transaction_sha256) blob_len=$($artifactRecord.artstor_blob_len)" } else { $install.PostArtifactResponse | ConvertTo-Json -Compress -Depth 20 })
if (-not $artifactOk) { throw "m6d ARTSTOR record did not link the exact candidate and promote" }

Send-AgentCommand -Command "agent module.loader_runtime" -ExpectedMarker "RAIOS_AGENT_END module.loader_runtime" -Name "m6d:loader-runtime"
$loaderResponse = Get-LastAgentResponseJson -Method "module.loader_runtime"
$loaderEvidence = @($loaderResponse.evidence)
$loaderApproval = @($loaderResponse.evidence | Where-Object id -eq "local_approval_reference")[0]
$loaderOk = $loaderResponse.schema -eq "raios.evidence_response.v1" -and
    $loaderResponse.family -eq "module.loader_runtime" -and
    $loaderResponse.scope -eq "current_boot" -and
    $loaderResponse.classification -eq "local_only" -and
    $loaderResponse.source_method -eq "module.loader_runtime" -and
    $null -eq $loaderResponse.event_id -and
    $loaderEvidence.Count -eq 54 -and
    $loaderEvidence[0].id -eq "manifest_reference" -and
    $loaderEvidence[53].id -eq "executable_entrypoint_invocation_boundary" -and
    $loaderResponse.PSObject.Properties.Name -notcontains "body" -and
    $loaderResponse.PSObject.Properties.Name -notcontains "live_granted_load_projection" -and
    @($loaderResponse.evidence | Where-Object id -eq "live_granted_load_projection").Count -eq 0 -and
    $loaderApproval.facts.present -eq $false -and $loaderApproval.facts.status_detail -eq "missing" -and
    $loaderResponse.decision.outcome -eq "denied" -and
    $loaderResponse.decision.reason -eq "retained_module_local_approval_reference_missing" -and
    @($loaderResponse.decision.grants).Count -eq 0 -and @($loaderResponse.decision.effects).Count -eq 0
Add-Predicate -Name "m6d:10_loader_runtime_root_denial_shape" -Expected "module.loader_runtime is the bare root v1 denial with event_id null, exactly 54 evidence entries, retained_module_local_approval_reference_missing, zero grants/effects, and no stale live projection" -Passed $loaderOk -Actual $(if ($loaderOk) { "bare denial; event_id=null evidence=54 projection=absent" } else { $loaderResponse | ConvertTo-Json -Compress -Depth 20 })
if (-not $loaderOk) { throw "m6d loader-runtime root denial shape drifted" }

Send-AgentCommand -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic" -Name "m6d:live-slot"
$slotResponse = Get-LastAgentResponseJson -Method "module.service_slot_diagnostic"
$liveSlot = @($slotResponse.evidence | Where-Object id -eq "live_granted_service_slot")[0].facts
$slotOk = $slotResponse.facts.runtime.live_granted_service_slot_present -eq $true -and
    $liveSlot.service_id -eq "svc.dev.granted_candidate" -and
    $liveSlot.ram_only_service_slot_id -eq "ram_only:svc.dev.granted_candidate" -and
    $liveSlot.service_slot_allocated -eq $true -and $liveSlot.running -eq $true -and
    $liveSlot.trust_tier -eq "dev_key_not_owner_sealed" -and
    $liveSlot.load_mechanism -eq "wasmi_interpreter_ram_only" -and
    $liveSlot.maps_executable_pages -eq $false -and $liveSlot.durable -eq $false -and
    $liveSlot.owner_sealed -eq $false -and $slotResponse.decision.outcome -eq "denied"
Add-Predicate -Name "m6d:11_service_slot_positive_shape" -Expected "the diagnostic runtime-presence fact is true and live_granted_service_slot reports the allocated/running RAM-only dev slot while the diagnostic decision remains denied" -Passed $slotOk -Actual $(if ($slotOk) { "runtime presence=true; allocated/running evidence present" } else { $slotResponse | ConvertTo-Json -Compress -Depth 20 })
if (-not $slotOk) { throw "m6d live service-slot shape drifted" }

Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "m6d:live-inventory"
$inventoryResponse = Get-LastAgentResponseJson -Method "service.inventory"
$inventoryRows = @($inventoryResponse.facts.services | Where-Object { $_.id -eq "svc.dev.granted_candidate" })
$inventoryRow = $inventoryRows[0]
$shapesOk = $inventoryRows.Count -eq 1 -and
    [int64]$inventoryRow.generation -eq $install.Generation -and
    $inventoryRow.last_run_outcome -eq "success" -and
    $inventoryRow.service_slot_activation_active -eq $true -and
    $inventoryRow.trust_tier -eq "dev_key_not_owner_sealed" -and
    $inventoryRow.PSObject.Properties.Name -notcontains "run_count" -and
    $activationLifecycle.PSObject.Properties.Name -contains "run_count" -and
    [int]$activationLifecycle.run_count -eq 1 -and
    $activationLifecycle.running -eq $true -and $activationLifecycle.last_run_evidence.run_outcome -eq "success"
$shapesDump = [ordered]@{ inventory = $inventoryResponse; lifecycle = $activationLifecycle }
Add-Predicate -Name "m6d:12_inventory_and_lifecycle_shapes" -Expected "inventory pins generation/last_run_outcome/service_slot_activation_active/trust_tier and has no run_count; the granted lifecycle body.result owns run_count=1" -Passed $shapesOk -Actual $(if ($shapesOk) { "inventory has no run_count; lifecycle run_count=1" } else { $shapesDump | ConvertTo-Json -Compress -Depth 20 })
if (-not $shapesOk) { throw "m6d inventory or lifecycle response shape drifted" }

Send-AgentCommand -Command "service.rollback_preview svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.rollback_preview" -Name "m6d:rollback-preview"
$rollbackPreviewResponse = Get-LastAgentResponseJson -Method "service.rollback_preview"
$rollbackPreview = $rollbackPreviewResponse.body.result
$rollbackOffset = Get-SerialLogOffset
Send-AgentCommand -Command "service.rollback_apply svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.rollback_apply" -Name "m6d:rollback-apply"
$rollbackApplyResponse = Get-LastAgentResponseJson -Method "service.rollback_apply"
$rollbackApply = $rollbackApplyResponse.body.result
$rollbackVerification = $rollbackApply.rollback_verification
$unpromote = $rollbackVerification.durable_unpromote_transaction
$rollbackOk = $rollbackPreview.code -eq "ok" -and
    $rollbackPreview.reason -eq "rollback_plan_recorded_ram_only" -and
    $rollbackPreview.rollback_plan.present -eq $true -and
    $rollbackPreview.rollback_plan.artifact_hash -eq $activation.CandidateSha256 -and
    $rollbackApply.code -eq "ok" -and
    $rollbackApply.reason -eq "unpromoted_dev_key_granted_external_wasm_current_boot" -and
    $rollbackApply.loaded -eq $false -and $rollbackApply.running -eq $false -and
    $rollbackApply.service_inventory_change -eq "removed_current_boot_service" -and
    $rollbackApply.rollback_apply_authorized -eq $true -and
    $rollbackVerification.stopped -eq $true -and
    $rollbackVerification.drop_clear_bytes -eq $true -and
    $rollbackVerification.free_slot -eq $true -and
    $rollbackVerification.remove_inventory -eq $true -and
    $rollbackVerification.restore_hash_verified -eq $true -and
    $rollbackVerification.pre_load_service_inventory_hash -match '^sha256:[0-9a-f]{64}$' -and
    $rollbackVerification.reprojected_service_inventory_hash -eq $rollbackVerification.pre_load_service_inventory_hash
Add-Predicate -Name "m6d:13_rollback_restores_exact_inventory" -Expected "rollback stops the service, drops/clears bytes, frees the slot, removes inventory, and reprojects exactly the captured pre-load inventory hash" -Passed $rollbackOk -Actual $(if ($rollbackOk) { "restored=$($rollbackVerification.reprojected_service_inventory_hash) cleanup=stop/drop/clear/free/remove" } else { [ordered]@{ preview = $rollbackPreviewResponse; apply = $rollbackApplyResponse } | ConvertTo-Json -Compress -Depth 24 })
if (-not $rollbackOk) { throw "m6d rollback did not restore the exact pre-load inventory" }

Send-AgentCommand -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "m6d:post-unpromote-reclog"
$postRollbackReclogResponse = Get-LastAgentResponseJson -Method "durable.record_log_scan"
$postRollbackReclog = $postRollbackReclogResponse.body.result
$unpromoteOk = $unpromote.transaction_kind -eq "unpromote" -and
    $unpromote.durable_append -eq "appended" -and $unpromote.code -eq "ok" -and
    $unpromote.performed -eq $true -and
    $unpromote.reason -eq "authorized_promotion_transaction_append_readback_reparse_verified" -and
    $unpromote.authority -eq "scoped_promotion_transaction_append_authorized" -and
    $unpromote.frame_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    $unpromote.readback_sha256 -eq $unpromote.frame_sha256 -and $unpromote.reparse_valid -eq $true -and
    $unpromote.signature_verified -eq $true -and $unpromote.grant_binds_capability -eq $true -and
    $unpromote.trust_tier -eq "dev_key_not_owner_sealed" -and
    [int64]$postRollbackReclog.count -eq ([int64]$install.PostReclogScan.count + 1) -and
    [int64]$unpromote.count_after -eq [int64]$postRollbackReclog.count -and
    [int64]$unpromote.tail_seq_after -eq [int64]$postRollbackReclog.tail_seq -and
    $postRollbackReclog.tail_frame_sha256 -eq $unpromote.frame_sha256 -and
    $rollbackVerification.restore_hash_verified -eq $true -and
    $rollbackPreview.rollback_plan.artifact_hash -eq $install.CandidateSha256
$unpromoteDump = [ordered]@{ rollback = $rollbackApplyResponse; reclog = $postRollbackReclogResponse; install = $install }
Add-Predicate -Name "m6d:14_durable_unpromote_readback" -Expected "the linked unpromote is appended/read back/reparsed as the new RECLOG tail with the same candidate, verified M6 signature/grant and retained W6 install provenance, plus the verified restore hash" -Passed $unpromoteOk -Actual $(if ($unpromoteOk) { "unpromote=$($unpromote.frame_sha256) tail=$($postRollbackReclog.tail_frame_sha256)" } else { $unpromoteDump | ConvertTo-Json -Compress -Depth 24 })
if (-not $unpromoteOk) { throw "m6d durable unpromote readback was not exact" }

Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "m6d:postrollback-inventory"
$postRollbackInventoryResponse = Get-LastAgentResponseJson -Method "service.inventory"
Send-AgentCommand -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic" -Name "m6d:postrollback-slot"
$postRollbackSlotResponse = Get-LastAgentResponseJson -Method "module.service_slot_diagnostic"
$postRollbackLog = (Get-SerialLogContent -Path $SerialLog).Substring([int]$rollbackOffset)
$absentOk = @($postRollbackInventoryResponse.facts.services | Where-Object { $_.id -eq "svc.dev.granted_candidate" }).Count -eq 0 -and
    $postRollbackSlotResponse.facts.runtime.live_granted_service_slot_present -eq $false -and
    @($postRollbackSlotResponse.evidence | Where-Object id -eq "live_granted_service_slot").Count -eq 0 -and
    -not $postRollbackLog.Contains("WASM_GUEST_LOG echo counter=")
$absentDump = [ordered]@{ inventory = $postRollbackInventoryResponse; slot = $postRollbackSlotResponse; log = $postRollbackLog }
Add-Predicate -Name "m6d:15_postrollback_service_absent" -Expected "after rollback the granted service has no inventory row, runtime slot presence is false with no positive slot evidence, and rollback emits no guest log" -Passed $absentOk -Actual $(if ($absentOk) { "inventory absent; slot absent; guest_log=0" } else { $absentDump | ConvertTo-Json -Compress -Depth 20 })
if (-not $absentOk) { throw "m6d service remained present after rollback" }

Send-AgentCommand -Command "service.rollback_apply svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.rollback_apply" -Name "m6d:second-rollback-apply"
$secondRollbackResponse = Get-LastAgentResponseJson -Method "service.rollback_apply"
$secondRollback = $secondRollbackResponse.body.result
Send-AgentCommand -Command "agent durable.record_log_scan" -ExpectedMarker "RAIOS_AGENT_END durable.record_log_scan" -Name "m6d:second-rollback-reclog"
$secondRollbackReclogResponse = Get-LastAgentResponseJson -Method "durable.record_log_scan"
$secondRollbackReclog = $secondRollbackReclogResponse.body.result
$secondDeniedOk = $secondRollback.code -eq "capability_denied" -and
    $secondRollback.reason -eq "no_recorded_promotion_to_roll_back" -and
    $secondRollback.rollback_apply_authorized -eq $false -and
    $secondRollback.rollback_verification.durable_unpromote_transaction.performed -eq $false -and
    $secondRollback.rollback_verification.durable_unpromote_transaction.reason -eq "no_recorded_promotion_to_roll_back" -and
    [int64]$secondRollbackReclog.count -eq [int64]$postRollbackReclog.count -and
    $secondRollbackReclog.head_frame_sha256 -eq $postRollbackReclog.head_frame_sha256 -and
    $secondRollbackReclog.tail_frame_sha256 -eq $postRollbackReclog.tail_frame_sha256
$secondDeniedDump = [ordered]@{ rollback = $secondRollbackResponse; before = $postRollbackReclogResponse; after = $secondRollbackReclogResponse }
Add-Predicate -Name "m6d:16_second_rollback_denied_one_shot" -Expected "a second rollback_apply denies with no_recorded_promotion_to_roll_back and appends no RECLOG frame" -Passed $secondDeniedOk -Actual $(if ($secondDeniedOk) { "denied one-shot; RECLOG unchanged" } else { $secondDeniedDump | ConvertTo-Json -Compress -Depth 20 })
if (-not $secondDeniedOk) { throw "m6d rollback was not one-shot" }

$repromotionOffset = Get-SerialLogOffset
Send-AgentCommand -Command "agent repromotion.run" -ExpectedMarker "RAIOS_AGENT_END repromotion.run" -Name "m6d:repromotion-after-unpromote"
$repromotionResponse = Get-LastAgentResponseJson -Method "repromotion.run"
$repromotion = $repromotionResponse.body.result
$repromotionLog = (Get-SerialLogContent -Path $SerialLog).Substring([int]$repromotionOffset)
Send-AgentCommand -Command "services" -ExpectedMarker "RAIOS_AGENT_END service.inventory" -Name "m6d:resolver-post-inventory"
$resolverInventoryResponse = Get-LastAgentResponseJson -Method "service.inventory"
Send-AgentCommand -Command "agent module.service_slot_diagnostic" -ExpectedMarker "RAIOS_AGENT_END module.service_slot_diagnostic" -Name "m6d:resolver-post-slot"
$resolverSlotResponse = Get-LastAgentResponseJson -Method "module.service_slot_diagnostic"
$resolverOk = $repromotion.status -eq "no_artifacts" -and
    $repromotion.performed -eq $true -and
    $repromotion.reason -eq "granted_candidate_install_rolled_back" -and
    [int]$repromotion.artifact_count -eq 0 -and @($repromotion.decisions).Count -eq 0 -and
    $repromotion.would_repromote -eq $false -and $repromotion.authorizes_load -eq $false -and
    $repromotion.cross_reboot_proven -eq $false -and
    -not $repromotionLog.Contains("RAIOS_AGENT_BEGIN service.load_ephemeral") -and
    -not $repromotionLog.Contains("RAIOS_AGENT_BEGIN service.start") -and
    -not $repromotionLog.Contains("WASM_GUEST_LOG echo counter=") -and
    @($resolverInventoryResponse.facts.services | Where-Object { $_.id -eq "svc.dev.granted_candidate" }).Count -eq 0 -and
    $resolverSlotResponse.facts.runtime.live_granted_service_slot_present -eq $false
$resolverDump = [ordered]@{ repromotion = $repromotionResponse; inventory = $resolverInventoryResponse; slot = $resolverSlotResponse; log = $repromotionLog }
Add-Predicate -Name "m6d:17_resolver_honors_newer_unpromote" -Expected "agent repromotion.run resolves the newer linked unpromote as rolled back/no artifacts, performs no retain/load/start or guest run, and cannot resurrect inventory or a slot" -Passed $resolverOk -Actual $(if ($resolverOk) { "rolled_back; artifacts=0; no load/start/resurrection" } else { $resolverDump | ConvertTo-Json -Compress -Depth 24 })
if (-not $resolverOk) { throw "m6d resolver resurrected or failed to honor the newer unpromote" }
