if (-not $Network) { throw "network-acquisition requires -Network" }

$activation = @(Invoke-W7M6PhysicalActivation)[-1]
$install = @(Invoke-SignedGrantedCandidateInstall -Activation $activation -NamePrefix "network-acquisition")[-1]

$preview = $install.Preview
$prepareOk = $preview.status -eq "accepted" -and
    $preview.reason -eq "project_install_signature_required" -and
    $preview.accepted -eq $true -and $preview.rejected -eq $false -and
    $preview.phase -eq "pending_owner_signature" -and
    $preview.action_kind -eq "install" -and $preview.signature_verified -eq $false -and
    $preview.install_source -eq "granted_candidate" -and
    $preview.receipt_kind -eq "w7_acquisition" -and $preview.w4_project_receipt_present -eq $false -and
    $preview.candidate_sha256 -eq $activation.CandidateSha256 -and
    $preview.receipt_sha256 -eq $activation.W7ReceiptSha256 -and
    $preview.activation_approval_sha256 -eq $activation.ActivationChallengeSha256 -and
    $preview.install_envelope_sha256 -match '^sha256:[0-9a-f]{64}$' -and
    $preview.promotion_transaction_sha256 -eq $null -and
    $preview.artifact_persist_frame_sha256 -eq $null -and
    $preview.writes_persistent_state -eq $false
Add-Predicate -Name "network-acquisition:25_w6_prepare_binds_b12b_w7_activation" -Expected "source granted_candidate binds the exact B1.2b candidate, W7 receipt, and activation challenge; receipt_kind=w7_acquisition, w4_project_receipt_present=false, no write" -Passed $prepareOk -Actual $(if ($prepareOk) { "exact W7 activation bound; signature required" } else { $install.PrepareResponse | ConvertTo-Json -Compress -Depth 12 })
if (-not $prepareOk) { throw "B1.2c W6 prepare did not bind the exact B1.2b W7 activation" }

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
Add-Predicate -Name "network-acquisition:26_w6_signature_is_separate_authority" -Expected "W6 action digest is signed, differs from the M6 attestation-reference digest, and remains pending Genesis approval" -Passed $signatureOk -Actual $(if ($signatureOk) { "w6=$($signed.action_signature_message_sha256) m6=$($activation.M6AttestationReferenceSha256)" } else { $install.SignatureResponse | ConvertTo-Json -Compress -Depth 12 })
if (-not $signatureOk) { throw "B1.2c W6 signature did not remain a separate pending authority" }

$denial = $install.Denial
$serialDenialOk = $denial.status -eq "denied" -and
    $denial.reason -eq "project_install_physical_pointer_approval_required" -and
    $denial.accepted -eq $false -and $denial.rejected -eq $true -and
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
Add-Predicate -Name "network-acquisition:27_serial_install_approval_denied_zero_effect" -Expected "project.install_approve returns project_install_physical_pointer_approval_required with the same signed preview and unchanged RECLOG/ARTSTOR evidence; denials emit no install marker" -Passed $serialDenialOk -Actual $(if ($serialDenialOk) { "denied response pinned; RECLOG/ARTSTOR unchanged" } else { [ordered]@{ denial = $install.DenialResponse; before_reclog = $install.PreReclogResponse; denied_reclog = $install.DeniedReclogResponse; before_artstor = $install.PreArtifactResponse; denied_artstor = $install.DeniedArtifactResponse } | ConvertTo-Json -Compress -Depth 12 })
if (-not $serialDenialOk) { throw "B1.2c serial install approval did not deny with zero effect" }

$markerOk = $install.ClickCount -eq 1 -and
    $install.CandidateSha256 -eq $activation.CandidateSha256 -and
    $install.W7ReceiptSha256 -eq $activation.W7ReceiptSha256 -and
    $install.ActivationApprovalSha256 -eq $activation.ActivationChallengeSha256 -and
    $install.InstallEnvelopeSha256 -eq $signed.install_envelope_sha256 -and
    $install.InstallActionSha256 -match '^sha256:[0-9a-f]{64}$' -and
    $install.PromotionTransactionSha256 -match '^sha256:[0-9a-f]{64}$' -and
    $install.ArtifactPersistFrameSha256 -match '^sha256:[0-9a-f]{64}$' -and
    $install.Generation -eq [int64]$signed.generation -and
    $install.Sequence -eq [int64]$signed.log_sequence -and
    $install.RunCount -eq 1 -and $install.TrustTier -eq "dev_key_not_owner_sealed" -and
    $install.DurableWrites -eq $true
Add-Predicate -Name "network-acquisition:28_second_click_installs_exact_running_candidate" -Expected "one W6 QMP click emits the accepted merged-kernel marker in exact field order for the same W7 candidate/receipt/activation, Genesis approval, run_count=1, dev trust tier, and durable writes" -Passed $markerOk -Actual $(if ($markerOk) { $install.MarkerLine } else { $install | ConvertTo-Json -Compress -Depth 12 })
if (-not $markerOk) { throw "B1.2c second physical approval did not install the exact running candidate" }

Send-AgentCommand -Command "service.start svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END service.start" -Name "network-acquisition:post-install-start-replay"
$postInstallStartResponse = Get-LastAgentResponseJson -Method "service.start"
$postInstallStart = $postInstallStartResponse.body.result
Send-AgentCommand -Command "module.load_ephemeral svc.dev.granted_candidate" -ExpectedMarker "RAIOS_AGENT_END module.load_ephemeral" -Name "network-acquisition:post-install-load-replay"
$postInstallLoadResponse = Get-LastAgentResponseJson -Method "module.load_ephemeral"
$postInstallLoad = $postInstallLoadResponse.body.result
$postInstallLog = (Get-SerialLogContent -Path $SerialLog).Substring([int]$activation.ActivationOffset)
$noRerunOk = $install.RunCount -eq 1 -and
    [int]$activation.LastLifecycleResponse.run_count -eq 1 -and
    $activation.LastLifecycleResponse.running -eq $true -and
    $postInstallStart.code -eq "capability_denied" -and
    $postInstallStart.reason -eq "physical_approval_already_consumed" -and
    [int]$postInstallStart.run_count -eq 1 -and $postInstallStart.running -eq $true -and
    $postInstallLoad.code -eq "capability_denied" -and
    $postInstallLoad.reason -eq "service_already_loaded" -and
    [int]$postInstallLoad.run_count -eq 1 -and $postInstallLoad.running -eq $true -and
    ([regex]::Matches($postInstallLog, [regex]::Escape("WASM_GUEST_LOG echo counter="))).Count -eq 1
Add-Predicate -Name "network-acquisition:29_install_does_not_rerun" -Expected "boot-1 lifecycle and install marker remain run_count=1, one guest log total, and serial load/start replays stay denied" -Passed $noRerunOk -Actual $(if ($noRerunOk) { "run_count=1 guest_log_count=1 replays denied" } else { [ordered]@{ activation = $activation.LastLifecycleResponse; marker = $install.MarkerLine; start = $postInstallStartResponse; load = $postInstallLoadResponse } | ConvertTo-Json -Compress -Depth 12 })
if (-not $noRerunOk) { throw "B1.2c durable install reran the already running candidate" }

$artifactRecord = @($install.PostArtifactScan.records | Where-Object {
    $_.artifact_sha256 -eq $activation.CandidateSha256 -and
    $_.promotion_transaction_sha256 -eq $install.PromotionTransactionSha256
})[0]
$m6AttestationEvidence = @($activation.M6AttestationResponse.evidence | Where-Object id -eq "local_attestation")[0]
$provenanceOk = [int64]$install.PostReclogScan.count -eq ([int64]$install.PreReclogScan.count + 3) -and
    $install.PostReclogScan.status -eq "valid" -and
    $install.PostReclogScan.valid_prefix_chain -eq $true -and
    $install.PostReclogScan.head_frame_sha256 -eq $install.PreReclogScan.head_frame_sha256 -and
    $install.PostReclogScan.tail_frame_sha256 -eq $install.ArtifactPersistFrameSha256 -and
    [int64]$install.PostArtifactScan.artifact_persist_record_count -eq ([int64]$install.PreArtifactScan.artifact_persist_record_count + 1) -and
    $install.PostArtifactScan.reclog_read_completed -eq $true -and
    $artifactRecord.present -eq $true -and $artifactRecord.blob_hash_verified -eq $true -and
    $artifactRecord.artifact_sha256 -eq $activation.CandidateSha256 -and
    $artifactRecord.promotion_transaction_sha256 -eq $install.PromotionTransactionSha256 -and
    $m6AttestationEvidence.facts.signature_verified -eq $true -and
    $install.SignedPreview.signature_verified -eq $true -and
    $install.Preview.w4_project_receipt_present -eq $false -and
    -not $postInstallLog.Contains("PROJECT_INSTALL_COMMIT kind=install")
Add-Predicate -Name "network-acquisition:30_artstor_reclog_w6_provenance" -Expected "RECLOG promote then artifact-persist readbacks form the marker head/tail, ARTSTOR verifies the exact candidate and promote link, M6/W6 signatures are verified, and no W4 project commit/receipt is fabricated" -Passed $provenanceOk -Actual $(if ($provenanceOk) { "promote=$($install.PromotionTransactionSha256) artifact_persist=$($install.ArtifactPersistFrameSha256) signatures=m6+w6" } else { [ordered]@{ reclog = $install.PostReclogResponse; artstor = $install.PostArtifactResponse; artifact_record = $artifactRecord; m6_attestation = $activation.M6AttestationResponse; w6_signature = $install.SignatureResponse; marker = $install.MarkerLine } | ConvertTo-Json -Compress -Depth 14 })
if (-not $provenanceOk) { throw "B1.2c ARTSTOR/RECLOG W6 provenance did not match the install marker" }
