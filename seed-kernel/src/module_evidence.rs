use sha2::{Digest, Sha256};

pub const MODULE_SERVICE_SLOT_ID_MAX: usize = 96;

pub struct ModuleAuditRecordHashInput<'a> {
    pub denial_event_id: &'a str,
    pub retained_reference_event_id: &'a str,
    pub computed_grant_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
    pub local_approval_hash: [u8; 32],
    pub rollback_plan_hash: [u8; 32],
    pub ram_only_service_slot_id: &'a str,
}

pub struct ModuleServiceSlotReservationHashInput<'a> {
    pub retained_reference_event_id: &'a str,
    pub retained_audit_rollback_reference_event_id: &'a str,
    pub computed_grant_hash: [u8; 32],
    pub audit_record_hash: [u8; 32],
    pub rollback_plan_hash: [u8; 32],
    pub pre_load_service_inventory_hash: [u8; 32],
    pub ram_only_service_slot_id: &'a str,
}

pub struct ModuleCandidateArtifactReferenceHashInput<'a> {
    pub retained_manifest_reference_event_id: &'a str,
    pub retained_reference_event_id: &'a str,
    pub manifest_reference_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub computed_grant_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
}

pub struct ModuleVmTestReportReferenceHashInput<'a> {
    pub retained_manifest_reference_event_id: &'a str,
    pub retained_artifact_reference_event_id: &'a str,
    pub retained_reference_event_id: &'a str,
    pub manifest_reference_hash: [u8; 32],
    pub artifact_reference_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub computed_grant_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
}

pub struct ModuleLocalAttestationReferenceHashInput<'a> {
    pub retained_manifest_reference_event_id: &'a str,
    pub retained_artifact_reference_event_id: &'a str,
    pub retained_vm_report_reference_event_id: &'a str,
    pub retained_reference_event_id: &'a str,
    pub manifest_reference_hash: [u8; 32],
    pub artifact_reference_hash: [u8; 32],
    pub vm_report_reference_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub computed_grant_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
}

pub struct ModuleLocalApprovalReferenceHashInput<'a> {
    pub retained_manifest_reference_event_id: &'a str,
    pub retained_artifact_reference_event_id: &'a str,
    pub retained_vm_report_reference_event_id: &'a str,
    pub retained_local_attestation_reference_event_id: &'a str,
    pub retained_reference_event_id: &'a str,
    pub manifest_reference_hash: [u8; 32],
    pub artifact_reference_hash: [u8; 32],
    pub vm_report_reference_hash: [u8; 32],
    pub local_attestation_reference_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub computed_grant_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
    pub local_approval_hash: [u8; 32],
}

pub struct RecoveryArtifactTrustReferenceHashInput<'a> {
    pub retained_identity_reference_event_id: &'a str,
    pub identity_reference_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub trust_hash: [u8; 32],
}

pub struct RecoveryArtifactVmTestReferenceHashInput<'a> {
    pub retained_identity_reference_event_id: &'a str,
    pub retained_trust_reference_event_id: &'a str,
    pub identity_reference_hash: [u8; 32],
    pub trust_reference_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub trust_hash: [u8; 32],
    pub vm_test_hash: [u8; 32],
}

pub struct RecoveryArtifactLocalApprovalReferenceHashInput<'a> {
    pub retained_identity_reference_event_id: &'a str,
    pub retained_trust_reference_event_id: &'a str,
    pub retained_vm_test_reference_event_id: &'a str,
    pub identity_reference_hash: [u8; 32],
    pub trust_reference_hash: [u8; 32],
    pub vm_test_reference_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub trust_hash: [u8; 32],
    pub vm_test_hash: [u8; 32],
    pub local_approval_hash: [u8; 32],
}

pub struct RecoveryArtifactLoaderReferenceHashInput<'a> {
    pub retained_identity_reference_event_id: &'a str,
    pub retained_trust_reference_event_id: &'a str,
    pub retained_vm_test_reference_event_id: &'a str,
    pub retained_local_approval_reference_event_id: &'a str,
    pub identity_reference_hash: [u8; 32],
    pub trust_reference_hash: [u8; 32],
    pub vm_test_reference_hash: [u8; 32],
    pub local_approval_reference_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub trust_hash: [u8; 32],
    pub vm_test_hash: [u8; 32],
    pub local_approval_hash: [u8; 32],
    pub loader_hash: [u8; 32],
}

pub struct RecoveryArtifactRollbackEvidenceReferenceHashInput<'a> {
    pub retained_identity_reference_event_id: &'a str,
    pub retained_trust_reference_event_id: &'a str,
    pub retained_vm_test_reference_event_id: &'a str,
    pub retained_local_approval_reference_event_id: &'a str,
    pub retained_loader_reference_event_id: &'a str,
    pub identity_reference_hash: [u8; 32],
    pub trust_reference_hash: [u8; 32],
    pub vm_test_reference_hash: [u8; 32],
    pub local_approval_reference_hash: [u8; 32],
    pub loader_reference_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub trust_hash: [u8; 32],
    pub vm_test_hash: [u8; 32],
    pub local_approval_hash: [u8; 32],
    pub loader_hash: [u8; 32],
    pub rollback_evidence_hash: [u8; 32],
}

pub struct RecoveryLifelineRequestReferenceHashInput<'a> {
    pub retained_identity_reference_event_id: &'a str,
    pub retained_trust_reference_event_id: &'a str,
    pub retained_vm_test_reference_event_id: &'a str,
    pub retained_local_approval_reference_event_id: &'a str,
    pub retained_loader_reference_event_id: &'a str,
    pub retained_rollback_evidence_reference_event_id: &'a str,
    pub identity_reference_hash: [u8; 32],
    pub trust_reference_hash: [u8; 32],
    pub vm_test_reference_hash: [u8; 32],
    pub local_approval_reference_hash: [u8; 32],
    pub loader_reference_hash: [u8; 32],
    pub rollback_evidence_reference_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub trust_hash: [u8; 32],
    pub vm_test_hash: [u8; 32],
    pub local_approval_hash: [u8; 32],
    pub loader_hash: [u8; 32],
    pub rollback_evidence_hash: [u8; 32],
}

pub fn computed_module_manifest_reference_hash(manifest_hash: [u8; 32]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.module_manifest_reference.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.module_manifest_reference.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.module.load_ephemeral",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=ram_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=live_service_graph", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_static_line(&mut hash, b"manifest_schema=raios.module_manifest.v0", true);
    hash_hash_line(&mut hash, b"manifest_sha256", manifest_hash, true);
    hash_static_line(&mut hash, b"authorizes_guest_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_recovery_artifact_identity_reference_hash(artifact_hash: [u8; 32]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.recovery_artifact_identity.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.recovery_artifact_identity.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.recovery.load_artifact",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=recovery_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=recovery_lifeline", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_hash_line(&mut hash, b"artifact_sha256", artifact_hash, true);
    hash_static_line(&mut hash, b"accepts_artifact_bytes=false", true);
    hash_static_line(&mut hash, b"loads_recovery_artifact=false", true);
    hash_static_line(&mut hash, b"authorizes_recovery_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_recovery_artifact_trust_reference_hash(
    input: RecoveryArtifactTrustReferenceHashInput<'_>,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.recovery_artifact_trust.canonical.v0",
        true,
    );
    hash_static_line(&mut hash, b"schema=raios.recovery_artifact_trust.v0", true);
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.recovery.load_artifact",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=recovery_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=recovery_lifeline", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_identity_event_id",
        input.retained_identity_reference_event_id,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"identity_reference_sha256",
        input.identity_reference_hash,
        true,
    );
    hash_hash_line(&mut hash, b"artifact_sha256", input.artifact_hash, true);
    hash_hash_line(&mut hash, b"trust_sha256", input.trust_hash, true);
    hash_static_line(&mut hash, b"accepts_artifact_bytes=false", true);
    hash_static_line(&mut hash, b"loads_recovery_artifact=false", true);
    hash_static_line(&mut hash, b"authorizes_recovery_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_recovery_artifact_vm_test_reference_hash(
    input: RecoveryArtifactVmTestReferenceHashInput<'_>,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.recovery_artifact_vm_test.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.recovery_artifact_vm_test.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.recovery.load_artifact",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=recovery_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=recovery_lifeline", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_identity_event_id",
        input.retained_identity_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_trust_event_id",
        input.retained_trust_reference_event_id,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"identity_reference_sha256",
        input.identity_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"trust_reference_sha256",
        input.trust_reference_hash,
        true,
    );
    hash_hash_line(&mut hash, b"artifact_sha256", input.artifact_hash, true);
    hash_hash_line(&mut hash, b"trust_sha256", input.trust_hash, true);
    hash_hash_line(&mut hash, b"vm_test_sha256", input.vm_test_hash, true);
    hash_static_line(&mut hash, b"accepts_vm_test_json=false", true);
    hash_static_line(&mut hash, b"accepts_artifact_bytes=false", true);
    hash_static_line(&mut hash, b"loads_recovery_artifact=false", true);
    hash_static_line(&mut hash, b"authorizes_recovery_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_recovery_artifact_local_approval_reference_hash(
    input: RecoveryArtifactLocalApprovalReferenceHashInput<'_>,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.recovery_artifact_local_approval.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.recovery_artifact_local_approval.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.recovery.load_artifact",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=recovery_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=recovery_lifeline", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_identity_event_id",
        input.retained_identity_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_trust_event_id",
        input.retained_trust_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_vm_test_event_id",
        input.retained_vm_test_reference_event_id,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"identity_reference_sha256",
        input.identity_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"trust_reference_sha256",
        input.trust_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"vm_test_reference_sha256",
        input.vm_test_reference_hash,
        true,
    );
    hash_hash_line(&mut hash, b"artifact_sha256", input.artifact_hash, true);
    hash_hash_line(&mut hash, b"trust_sha256", input.trust_hash, true);
    hash_hash_line(&mut hash, b"vm_test_sha256", input.vm_test_hash, true);
    hash_hash_line(
        &mut hash,
        b"local_approval_sha256",
        input.local_approval_hash,
        true,
    );
    hash_static_line(&mut hash, b"accepts_local_approval_text=false", true);
    hash_static_line(&mut hash, b"accepts_artifact_bytes=false", true);
    hash_static_line(&mut hash, b"loads_recovery_artifact=false", true);
    hash_static_line(&mut hash, b"authorizes_recovery_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_recovery_artifact_loader_reference_hash(
    input: RecoveryArtifactLoaderReferenceHashInput<'_>,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.recovery_artifact_loader.canonical.v0",
        true,
    );
    hash_static_line(&mut hash, b"schema=raios.recovery_artifact_loader.v0", true);
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.recovery.load_artifact",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=recovery_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=recovery_lifeline", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_identity_event_id",
        input.retained_identity_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_trust_event_id",
        input.retained_trust_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_vm_test_event_id",
        input.retained_vm_test_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_local_approval_event_id",
        input.retained_local_approval_reference_event_id,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"identity_reference_sha256",
        input.identity_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"trust_reference_sha256",
        input.trust_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"vm_test_reference_sha256",
        input.vm_test_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"local_approval_reference_sha256",
        input.local_approval_reference_hash,
        true,
    );
    hash_hash_line(&mut hash, b"artifact_sha256", input.artifact_hash, true);
    hash_hash_line(&mut hash, b"trust_sha256", input.trust_hash, true);
    hash_hash_line(&mut hash, b"vm_test_sha256", input.vm_test_hash, true);
    hash_hash_line(
        &mut hash,
        b"local_approval_sha256",
        input.local_approval_hash,
        true,
    );
    hash_hash_line(&mut hash, b"loader_sha256", input.loader_hash, true);
    hash_static_line(&mut hash, b"accepts_loader_descriptor=false", true);
    hash_static_line(&mut hash, b"accepts_artifact_bytes=false", true);
    hash_static_line(&mut hash, b"loads_recovery_loader=false", true);
    hash_static_line(&mut hash, b"loads_recovery_artifact=false", true);
    hash_static_line(&mut hash, b"authorizes_recovery_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_recovery_artifact_rollback_evidence_reference_hash(
    input: RecoveryArtifactRollbackEvidenceReferenceHashInput<'_>,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.recovery_artifact_rollback_evidence.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.recovery_artifact_rollback_evidence.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.recovery.load_artifact",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=recovery_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=recovery_lifeline", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_identity_event_id",
        input.retained_identity_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_trust_event_id",
        input.retained_trust_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_vm_test_event_id",
        input.retained_vm_test_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_local_approval_event_id",
        input.retained_local_approval_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_loader_event_id",
        input.retained_loader_reference_event_id,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"identity_reference_sha256",
        input.identity_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"trust_reference_sha256",
        input.trust_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"vm_test_reference_sha256",
        input.vm_test_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"local_approval_reference_sha256",
        input.local_approval_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"loader_reference_sha256",
        input.loader_reference_hash,
        true,
    );
    hash_hash_line(&mut hash, b"artifact_sha256", input.artifact_hash, true);
    hash_hash_line(&mut hash, b"trust_sha256", input.trust_hash, true);
    hash_hash_line(&mut hash, b"vm_test_sha256", input.vm_test_hash, true);
    hash_hash_line(
        &mut hash,
        b"local_approval_sha256",
        input.local_approval_hash,
        true,
    );
    hash_hash_line(&mut hash, b"loader_sha256", input.loader_hash, true);
    hash_hash_line(
        &mut hash,
        b"rollback_evidence_sha256",
        input.rollback_evidence_hash,
        true,
    );
    hash_static_line(&mut hash, b"accepts_rollback_evidence_json=false", true);
    hash_static_line(&mut hash, b"accepts_artifact_bytes=false", true);
    hash_static_line(&mut hash, b"loads_recovery_artifact=false", true);
    hash_static_line(&mut hash, b"authorizes_recovery_load=false", true);
    hash_static_line(&mut hash, b"creates_durable_records=false", true);
    hash_static_line(&mut hash, b"installs_rollback_plan=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_recovery_lifeline_request_reference_hash(
    input: RecoveryLifelineRequestReferenceHashInput<'_>,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.recovery_lifeline_request.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.recovery_lifeline_request.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.recovery.load_artifact",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=recovery_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=recovery_lifeline", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_identity_event_id",
        input.retained_identity_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_trust_event_id",
        input.retained_trust_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_vm_test_event_id",
        input.retained_vm_test_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_local_approval_event_id",
        input.retained_local_approval_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_loader_event_id",
        input.retained_loader_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_recovery_artifact_rollback_evidence_event_id",
        input.retained_rollback_evidence_reference_event_id,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"identity_reference_sha256",
        input.identity_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"trust_reference_sha256",
        input.trust_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"vm_test_reference_sha256",
        input.vm_test_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"local_approval_reference_sha256",
        input.local_approval_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"loader_reference_sha256",
        input.loader_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"rollback_evidence_reference_sha256",
        input.rollback_evidence_reference_hash,
        true,
    );
    hash_hash_line(&mut hash, b"artifact_sha256", input.artifact_hash, true);
    hash_hash_line(&mut hash, b"trust_sha256", input.trust_hash, true);
    hash_hash_line(&mut hash, b"vm_test_sha256", input.vm_test_hash, true);
    hash_hash_line(
        &mut hash,
        b"local_approval_sha256",
        input.local_approval_hash,
        true,
    );
    hash_hash_line(&mut hash, b"loader_sha256", input.loader_hash, true);
    hash_hash_line(
        &mut hash,
        b"rollback_evidence_sha256",
        input.rollback_evidence_hash,
        true,
    );
    hash_static_line(&mut hash, b"accepts_lifeline_request_json=false", true);
    hash_static_line(&mut hash, b"accepts_loader_descriptor=false", true);
    hash_static_line(&mut hash, b"accepts_artifact_bytes=false", true);
    hash_static_line(&mut hash, b"loads_recovery_loader=false", true);
    hash_static_line(&mut hash, b"loads_recovery_artifact=false", true);
    hash_static_line(&mut hash, b"authorizes_recovery_load=false", true);
    hash_static_line(&mut hash, b"creates_durable_records=false", true);
    hash_static_line(&mut hash, b"installs_rollback_plan=false", true);
    hash_static_line(&mut hash, b"allocates_service_slot=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_module_candidate_artifact_reference_hash(
    input: ModuleCandidateArtifactReferenceHashInput<'_>,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.module_candidate_artifact_reference.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.module_candidate_artifact_reference.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.module.load_ephemeral",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=ram_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=live_service_graph", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_str_line(
        &mut hash,
        b"retained_manifest_reference_event_id",
        input.retained_manifest_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_reference_event_id",
        input.retained_reference_event_id,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"manifest_reference_sha256",
        input.manifest_reference_hash,
        true,
    );
    hash_hash_line(&mut hash, b"manifest_sha256", input.manifest_hash, true);
    hash_hash_line(
        &mut hash,
        b"computed_capability_grant_sha256",
        input.computed_grant_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"candidate_artifact_sha256",
        input.artifact_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"vm_test_report_sha256",
        input.vm_report_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"local_attestation_sha256",
        input.local_attestation_hash,
        true,
    );
    hash_static_line(&mut hash, b"accepts_artifact_bytes=false", true);
    hash_static_line(&mut hash, b"loads_artifact=false", true);
    hash_static_line(&mut hash, b"authorizes_guest_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_module_candidate_artifact_reference_hash_from_sequences(
    retained_manifest_reference_event_sequence: u64,
    retained_reference_event_sequence: u64,
    manifest_reference_hash: [u8; 32],
    manifest_hash: [u8; 32],
    computed_grant_hash: [u8; 32],
    artifact_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.module_candidate_artifact_reference.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.module_candidate_artifact_reference.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.module.load_ephemeral",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=ram_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=live_service_graph", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_event_id_line(
        &mut hash,
        b"retained_manifest_reference_event_id",
        retained_manifest_reference_event_sequence,
        true,
    );
    hash_event_id_line(
        &mut hash,
        b"retained_reference_event_id",
        retained_reference_event_sequence,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"manifest_reference_sha256",
        manifest_reference_hash,
        true,
    );
    hash_hash_line(&mut hash, b"manifest_sha256", manifest_hash, true);
    hash_hash_line(
        &mut hash,
        b"computed_capability_grant_sha256",
        computed_grant_hash,
        true,
    );
    hash_hash_line(&mut hash, b"candidate_artifact_sha256", artifact_hash, true);
    hash_hash_line(&mut hash, b"vm_test_report_sha256", vm_report_hash, true);
    hash_hash_line(
        &mut hash,
        b"local_attestation_sha256",
        local_attestation_hash,
        true,
    );
    hash_static_line(&mut hash, b"accepts_artifact_bytes=false", true);
    hash_static_line(&mut hash, b"loads_artifact=false", true);
    hash_static_line(&mut hash, b"authorizes_guest_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_module_vm_test_report_reference_hash(
    input: ModuleVmTestReportReferenceHashInput<'_>,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.module_vm_test_report_reference.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.module_vm_test_report_reference.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.module.load_ephemeral",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=ram_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=live_service_graph", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_str_line(
        &mut hash,
        b"retained_manifest_reference_event_id",
        input.retained_manifest_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_artifact_reference_event_id",
        input.retained_artifact_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_reference_event_id",
        input.retained_reference_event_id,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"manifest_reference_sha256",
        input.manifest_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"artifact_reference_sha256",
        input.artifact_reference_hash,
        true,
    );
    hash_hash_line(&mut hash, b"manifest_sha256", input.manifest_hash, true);
    hash_hash_line(
        &mut hash,
        b"candidate_artifact_sha256",
        input.artifact_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"computed_capability_grant_sha256",
        input.computed_grant_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"vm_test_report_sha256",
        input.vm_report_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"local_attestation_sha256",
        input.local_attestation_hash,
        true,
    );
    hash_static_line(&mut hash, b"accepts_vm_report_json=false", true);
    hash_static_line(&mut hash, b"accepts_artifact_bytes=false", true);
    hash_static_line(&mut hash, b"loads_artifact=false", true);
    hash_static_line(&mut hash, b"authorizes_guest_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_module_vm_test_report_reference_hash_from_sequences(
    retained_manifest_reference_event_sequence: u64,
    retained_artifact_reference_event_sequence: u64,
    retained_reference_event_sequence: u64,
    manifest_reference_hash: [u8; 32],
    artifact_reference_hash: [u8; 32],
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    computed_grant_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.module_vm_test_report_reference.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.module_vm_test_report_reference.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.module.load_ephemeral",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=ram_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=live_service_graph", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_event_id_line(
        &mut hash,
        b"retained_manifest_reference_event_id",
        retained_manifest_reference_event_sequence,
        true,
    );
    hash_event_id_line(
        &mut hash,
        b"retained_artifact_reference_event_id",
        retained_artifact_reference_event_sequence,
        true,
    );
    hash_event_id_line(
        &mut hash,
        b"retained_reference_event_id",
        retained_reference_event_sequence,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"manifest_reference_sha256",
        manifest_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"artifact_reference_sha256",
        artifact_reference_hash,
        true,
    );
    hash_hash_line(&mut hash, b"manifest_sha256", manifest_hash, true);
    hash_hash_line(&mut hash, b"candidate_artifact_sha256", artifact_hash, true);
    hash_hash_line(
        &mut hash,
        b"computed_capability_grant_sha256",
        computed_grant_hash,
        true,
    );
    hash_hash_line(&mut hash, b"vm_test_report_sha256", vm_report_hash, true);
    hash_hash_line(
        &mut hash,
        b"local_attestation_sha256",
        local_attestation_hash,
        true,
    );
    hash_static_line(&mut hash, b"accepts_vm_report_json=false", true);
    hash_static_line(&mut hash, b"accepts_artifact_bytes=false", true);
    hash_static_line(&mut hash, b"loads_artifact=false", true);
    hash_static_line(&mut hash, b"authorizes_guest_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_module_local_attestation_reference_hash(
    input: ModuleLocalAttestationReferenceHashInput<'_>,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.module_local_attestation_reference.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.module_local_attestation_reference.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.module.load_ephemeral",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=ram_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=live_service_graph", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_str_line(
        &mut hash,
        b"retained_manifest_reference_event_id",
        input.retained_manifest_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_artifact_reference_event_id",
        input.retained_artifact_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_vm_report_reference_event_id",
        input.retained_vm_report_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_reference_event_id",
        input.retained_reference_event_id,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"manifest_reference_sha256",
        input.manifest_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"artifact_reference_sha256",
        input.artifact_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"vm_test_report_reference_sha256",
        input.vm_report_reference_hash,
        true,
    );
    hash_hash_line(&mut hash, b"manifest_sha256", input.manifest_hash, true);
    hash_hash_line(
        &mut hash,
        b"candidate_artifact_sha256",
        input.artifact_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"computed_capability_grant_sha256",
        input.computed_grant_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"vm_test_report_sha256",
        input.vm_report_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"local_attestation_sha256",
        input.local_attestation_hash,
        true,
    );
    hash_static_line(&mut hash, b"accepts_local_attestation_json=false", true);
    hash_static_line(&mut hash, b"accepts_artifact_bytes=false", true);
    hash_static_line(&mut hash, b"loads_artifact=false", true);
    hash_static_line(&mut hash, b"authorizes_guest_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_module_local_attestation_reference_hash_from_sequences(
    retained_manifest_reference_event_sequence: u64,
    retained_artifact_reference_event_sequence: u64,
    retained_vm_report_reference_event_sequence: u64,
    retained_reference_event_sequence: u64,
    manifest_reference_hash: [u8; 32],
    artifact_reference_hash: [u8; 32],
    vm_report_reference_hash: [u8; 32],
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    computed_grant_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.module_local_attestation_reference.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.module_local_attestation_reference.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.module.load_ephemeral",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=ram_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=live_service_graph", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_event_id_line(
        &mut hash,
        b"retained_manifest_reference_event_id",
        retained_manifest_reference_event_sequence,
        true,
    );
    hash_event_id_line(
        &mut hash,
        b"retained_artifact_reference_event_id",
        retained_artifact_reference_event_sequence,
        true,
    );
    hash_event_id_line(
        &mut hash,
        b"retained_vm_report_reference_event_id",
        retained_vm_report_reference_event_sequence,
        true,
    );
    hash_event_id_line(
        &mut hash,
        b"retained_reference_event_id",
        retained_reference_event_sequence,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"manifest_reference_sha256",
        manifest_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"artifact_reference_sha256",
        artifact_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"vm_test_report_reference_sha256",
        vm_report_reference_hash,
        true,
    );
    hash_hash_line(&mut hash, b"manifest_sha256", manifest_hash, true);
    hash_hash_line(&mut hash, b"candidate_artifact_sha256", artifact_hash, true);
    hash_hash_line(
        &mut hash,
        b"computed_capability_grant_sha256",
        computed_grant_hash,
        true,
    );
    hash_hash_line(&mut hash, b"vm_test_report_sha256", vm_report_hash, true);
    hash_hash_line(
        &mut hash,
        b"local_attestation_sha256",
        local_attestation_hash,
        true,
    );
    hash_static_line(&mut hash, b"accepts_local_attestation_json=false", true);
    hash_static_line(&mut hash, b"accepts_artifact_bytes=false", true);
    hash_static_line(&mut hash, b"loads_artifact=false", true);
    hash_static_line(&mut hash, b"authorizes_guest_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_module_local_approval_reference_hash(
    input: ModuleLocalApprovalReferenceHashInput<'_>,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.module_local_approval_reference.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.module_local_approval_reference.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.module.load_ephemeral",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=ram_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=live_service_graph", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_str_line(
        &mut hash,
        b"retained_manifest_reference_event_id",
        input.retained_manifest_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_artifact_reference_event_id",
        input.retained_artifact_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_vm_report_reference_event_id",
        input.retained_vm_report_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_local_attestation_reference_event_id",
        input.retained_local_attestation_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_reference_event_id",
        input.retained_reference_event_id,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"manifest_reference_sha256",
        input.manifest_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"artifact_reference_sha256",
        input.artifact_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"vm_test_report_reference_sha256",
        input.vm_report_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"local_attestation_reference_sha256",
        input.local_attestation_reference_hash,
        true,
    );
    hash_hash_line(&mut hash, b"manifest_sha256", input.manifest_hash, true);
    hash_hash_line(
        &mut hash,
        b"candidate_artifact_sha256",
        input.artifact_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"computed_capability_grant_sha256",
        input.computed_grant_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"vm_test_report_sha256",
        input.vm_report_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"local_attestation_sha256",
        input.local_attestation_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"local_approval_sha256",
        input.local_approval_hash,
        true,
    );
    hash_static_line(&mut hash, b"accepts_local_approval_text=false", true);
    hash_static_line(&mut hash, b"accepts_artifact_bytes=false", true);
    hash_static_line(&mut hash, b"loads_artifact=false", true);
    hash_static_line(&mut hash, b"authorizes_guest_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_module_local_approval_reference_hash_from_sequences(
    retained_manifest_reference_event_sequence: u64,
    retained_artifact_reference_event_sequence: u64,
    retained_vm_report_reference_event_sequence: u64,
    retained_local_attestation_reference_event_sequence: u64,
    retained_reference_event_sequence: u64,
    manifest_reference_hash: [u8; 32],
    artifact_reference_hash: [u8; 32],
    vm_report_reference_hash: [u8; 32],
    local_attestation_reference_hash: [u8; 32],
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    computed_grant_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
    local_approval_hash: [u8; 32],
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.module_local_approval_reference.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.module_local_approval_reference.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.module.load_ephemeral",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=ram_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=live_service_graph", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_event_id_line(
        &mut hash,
        b"retained_manifest_reference_event_id",
        retained_manifest_reference_event_sequence,
        true,
    );
    hash_event_id_line(
        &mut hash,
        b"retained_artifact_reference_event_id",
        retained_artifact_reference_event_sequence,
        true,
    );
    hash_event_id_line(
        &mut hash,
        b"retained_vm_report_reference_event_id",
        retained_vm_report_reference_event_sequence,
        true,
    );
    hash_event_id_line(
        &mut hash,
        b"retained_local_attestation_reference_event_id",
        retained_local_attestation_reference_event_sequence,
        true,
    );
    hash_event_id_line(
        &mut hash,
        b"retained_reference_event_id",
        retained_reference_event_sequence,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"manifest_reference_sha256",
        manifest_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"artifact_reference_sha256",
        artifact_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"vm_test_report_reference_sha256",
        vm_report_reference_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"local_attestation_reference_sha256",
        local_attestation_reference_hash,
        true,
    );
    hash_hash_line(&mut hash, b"manifest_sha256", manifest_hash, true);
    hash_hash_line(&mut hash, b"candidate_artifact_sha256", artifact_hash, true);
    hash_hash_line(
        &mut hash,
        b"computed_capability_grant_sha256",
        computed_grant_hash,
        true,
    );
    hash_hash_line(&mut hash, b"vm_test_report_sha256", vm_report_hash, true);
    hash_hash_line(
        &mut hash,
        b"local_attestation_sha256",
        local_attestation_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"local_approval_sha256",
        local_approval_hash,
        true,
    );
    hash_static_line(&mut hash, b"accepts_local_approval_text=false", true);
    hash_static_line(&mut hash, b"accepts_artifact_bytes=false", true);
    hash_static_line(&mut hash, b"loads_artifact=false", true);
    hash_static_line(&mut hash, b"authorizes_guest_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_module_grant_hash(
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.computed_capability_grant.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.computed_capability_grant.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.module.load_ephemeral",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=ram_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=live_service_graph", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_hash_line(&mut hash, b"manifest_sha256", manifest_hash, true);
    hash_hash_line(&mut hash, b"candidate_artifact_sha256", artifact_hash, true);
    hash_hash_line(&mut hash, b"vm_test_report_sha256", vm_report_hash, true);
    hash_hash_line(
        &mut hash,
        b"local_attestation_sha256",
        local_attestation_hash,
        true,
    );
    hash_static_line(&mut hash, b"grants_load_now=false", true);
    hash_static_line(&mut hash, b"authorizes_guest_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_module_rollback_plan_hash(
    artifact_hash: [u8; 32],
    pre_load_service_inventory_hash: [u8; 32],
    ram_only_service_slot_id: &str,
    cleanup_actions_hash: [u8; 32],
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.rollback_plan.canonical.v0",
        true,
    );
    hash_static_line(&mut hash, b"schema=raios.rollback_plan.v0", true);
    hash_static_line(&mut hash, b"load_mode=ram_only", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_hash_line(&mut hash, b"artifact_sha256", artifact_hash, true);
    hash_hash_line(
        &mut hash,
        b"pre_load_service_inventory_sha256",
        pre_load_service_inventory_hash,
        true,
    );
    hash_str_line(
        &mut hash,
        b"ram_only_service_slot_id",
        ram_only_service_slot_id,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"cleanup_actions_sha256",
        cleanup_actions_hash,
        true,
    );
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_module_audit_record_hash(input: ModuleAuditRecordHashInput<'_>) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.audit_record.canonical.v0",
        true,
    );
    hash_static_line(&mut hash, b"schema=raios.audit_record.v0", true);
    hash_static_line(
        &mut hash,
        b"requested_capability=cap.module.load_ephemeral",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=ram_only", true);
    hash_static_line(&mut hash, b"subject=agent.session.serial", true);
    hash_static_line(&mut hash, b"resource=live_service_graph", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_str_line(&mut hash, b"denial_event_id", input.denial_event_id, true);
    hash_str_line(
        &mut hash,
        b"retained_reference_event_id",
        input.retained_reference_event_id,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"computed_capability_grant_sha256",
        input.computed_grant_hash,
        true,
    );
    hash_hash_line(&mut hash, b"manifest_sha256", input.manifest_hash, true);
    hash_hash_line(
        &mut hash,
        b"candidate_artifact_sha256",
        input.artifact_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"vm_test_report_sha256",
        input.vm_report_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"local_attestation_sha256",
        input.local_attestation_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"local_approval_sha256",
        input.local_approval_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"rollback_plan_sha256",
        input.rollback_plan_hash,
        true,
    );
    hash_str_line(
        &mut hash,
        b"ram_only_service_slot_id",
        input.ram_only_service_slot_id,
        true,
    );
    hash_static_line(&mut hash, b"grants_load_now=false", true);
    hash_static_line(&mut hash, b"authorizes_guest_load=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_module_service_slot_reservation_hash(
    input: ModuleServiceSlotReservationHashInput<'_>,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.module_service_slot_reservation.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.module_service_slot_reservation.v0",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=ram_only", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_str_line(
        &mut hash,
        b"retained_reference_event_id",
        input.retained_reference_event_id,
        true,
    );
    hash_str_line(
        &mut hash,
        b"retained_audit_rollback_reference_event_id",
        input.retained_audit_rollback_reference_event_id,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"computed_capability_grant_sha256",
        input.computed_grant_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"audit_record_sha256",
        input.audit_record_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"rollback_plan_sha256",
        input.rollback_plan_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"pre_load_service_inventory_sha256",
        input.pre_load_service_inventory_hash,
        true,
    );
    hash_str_line(
        &mut hash,
        b"ram_only_service_slot_id",
        input.ram_only_service_slot_id,
        true,
    );
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_module_audit_append_payload_hash_from_sequences(
    retained_audit_rollback_reference_event_sequence: u64,
    retained_service_slot_reservation_event_sequence: u64,
    audit_record_hash: [u8; 32],
    rollback_plan_hash: [u8; 32],
    pre_load_service_inventory_hash: [u8; 32],
    service_slot_reservation_hash: [u8; 32],
    ram_only_service_slot_id: &str,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.append_payload_hash_envelope.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.audit_record_append_payload_hash_envelope.v0",
        true,
    );
    hash_static_line(&mut hash, b"target_schema=raios.audit_record.v0", true);
    hash_static_line(
        &mut hash,
        b"pre_load_write_request_schema=raios.module_pre_load_audit_rollback_write_request.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"append_contract_id=append.audit_ledger.current_boot",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=ram_only", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_event_id_line(
        &mut hash,
        b"retained_audit_rollback_reference_event_id",
        retained_audit_rollback_reference_event_sequence,
        true,
    );
    hash_event_id_line(
        &mut hash,
        b"retained_service_slot_reservation_event_id",
        retained_service_slot_reservation_event_sequence,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"audit_record_payload_sha256",
        audit_record_hash,
        true,
    );
    hash_hash_line(&mut hash, b"rollback_plan_sha256", rollback_plan_hash, true);
    hash_hash_line(
        &mut hash,
        b"pre_load_service_inventory_sha256",
        pre_load_service_inventory_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"service_slot_reservation_sha256",
        service_slot_reservation_hash,
        true,
    );
    hash_str_line(
        &mut hash,
        b"ram_only_service_slot_id",
        ram_only_service_slot_id,
        true,
    );
    hash_static_line(&mut hash, b"classification=local_only", true);
    hash_static_line(&mut hash, b"authorizes_append_intent=false", true);
    hash_static_line(&mut hash, b"authorizes_write=false", true);
    hash_static_line(&mut hash, b"durable=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn computed_module_rollback_append_payload_hash_from_sequences(
    retained_audit_rollback_reference_event_sequence: u64,
    retained_service_slot_reservation_event_sequence: u64,
    audit_record_hash: [u8; 32],
    rollback_plan_hash: [u8; 32],
    pre_load_service_inventory_hash: [u8; 32],
    service_slot_reservation_hash: [u8; 32],
    ram_only_service_slot_id: &str,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_static_line(
        &mut hash,
        b"canonicalization=raios.append_payload_hash_envelope.canonical.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"schema=raios.rollback_transaction_append_payload_hash_envelope.v0",
        true,
    );
    hash_static_line(&mut hash, b"target_schema=raios.rollback_plan.v0", true);
    hash_static_line(
        &mut hash,
        b"pre_load_write_request_schema=raios.module_pre_load_audit_rollback_write_request.v0",
        true,
    );
    hash_static_line(
        &mut hash,
        b"append_contract_id=append.rollback_store.current_boot",
        true,
    );
    hash_static_line(&mut hash, b"load_mode=ram_only", true);
    hash_static_line(&mut hash, b"scope=current_boot", true);
    hash_event_id_line(
        &mut hash,
        b"retained_audit_rollback_reference_event_id",
        retained_audit_rollback_reference_event_sequence,
        true,
    );
    hash_event_id_line(
        &mut hash,
        b"retained_service_slot_reservation_event_id",
        retained_service_slot_reservation_event_sequence,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"rollback_plan_payload_sha256",
        rollback_plan_hash,
        true,
    );
    hash_hash_line(&mut hash, b"audit_record_sha256", audit_record_hash, true);
    hash_hash_line(
        &mut hash,
        b"pre_load_service_inventory_sha256",
        pre_load_service_inventory_hash,
        true,
    );
    hash_hash_line(
        &mut hash,
        b"service_slot_reservation_sha256",
        service_slot_reservation_hash,
        true,
    );
    hash_str_line(
        &mut hash,
        b"ram_only_service_slot_id",
        ram_only_service_slot_id,
        true,
    );
    hash_static_line(&mut hash, b"classification=local_only", true);
    hash_static_line(&mut hash, b"authorizes_append_intent=false", true);
    hash_static_line(&mut hash, b"authorizes_write=false", true);
    hash_static_line(&mut hash, b"durable=false", true);
    hash_static_line(&mut hash, b"service_inventory_change=none", true);
    hash_static_line(&mut hash, b"load_attempted=false", false);
    finalize_sha256(hash)
}

pub fn ram_only_service_slot_id_valid(value: &str) -> bool {
    let Some(slot) = value.strip_prefix("ram_only:") else {
        return false;
    };
    !slot.is_empty()
        && value.len() <= MODULE_SERVICE_SLOT_ID_MAX
        && slot
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

fn finalize_sha256(hash: Sha256) -> [u8; 32] {
    let digest = hash.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn hash_static_line(hash: &mut Sha256, value: &'static [u8], newline: bool) {
    hash.update(value);
    if newline {
        hash.update(b"\n");
    }
}

fn hash_hash_line(hash: &mut Sha256, name: &'static [u8], value: [u8; 32], newline: bool) {
    hash.update(name);
    hash.update(b"=");
    hash_lower_hex(hash, value);
    if newline {
        hash.update(b"\n");
    }
}

fn hash_lower_hex(hash: &mut Sha256, value: [u8; 32]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut idx = 0usize;
    while idx < value.len() {
        let byte = value[idx];
        hash.update(&[HEX[(byte >> 4) as usize], HEX[(byte & 0x0f) as usize]]);
        idx += 1;
    }
}

fn hash_str_line(hash: &mut Sha256, name: &'static [u8], value: &str, newline: bool) {
    hash.update(name);
    hash.update(b"=");
    hash.update(value.as_bytes());
    if newline {
        hash.update(b"\n");
    }
}

fn hash_event_id_line(hash: &mut Sha256, name: &'static [u8], sequence: u64, newline: bool) {
    hash.update(name);
    hash.update(b"=event.current_boot.");
    let mut divisor = 10_000_000u64;
    while divisor > 0 {
        let digit = ((sequence / divisor) % 10) as u8;
        hash.update(&[b'0' + digit]);
        divisor /= 10;
    }
    if newline {
        hash.update(b"\n");
    }
}
