//! Pure module-evidence reference evaluators.
//!
//! Event-log records are represented by neutral values here.  A kernel adapter
//! may translate its event ids and records into these values, but these
//! functions never read or consume the event log and never authorize a load.

use sha2::{Digest, Sha256};

use crate::method_eq;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModuleManifestReference {
    pub manifest_reference_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModuleCandidateArtifactReference {
    pub artifact_reference_hash: [u8; 32],
    pub retained_manifest_reference_event_id: u64,
    pub retained_reference_event_id: u64,
    pub manifest_reference_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub computed_grant_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModuleVmTestReportReference {
    pub report_reference_hash: [u8; 32],
    pub retained_manifest_reference_event_id: u64,
    pub retained_artifact_reference_event_id: u64,
    pub retained_reference_event_id: u64,
    pub manifest_reference_hash: [u8; 32],
    pub artifact_reference_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub computed_grant_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModuleLocalAttestationReference {
    pub attestation_reference_hash: [u8; 32],
    pub retained_manifest_reference_event_id: u64,
    pub retained_artifact_reference_event_id: u64,
    pub retained_vm_report_reference_event_id: u64,
    pub retained_reference_event_id: u64,
    pub manifest_reference_hash: [u8; 32],
    pub artifact_reference_hash: [u8; 32],
    pub vm_report_reference_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub computed_grant_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
    pub signature_verified: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModuleLocalApprovalReference {
    pub approval_reference_hash: [u8; 32],
    pub retained_manifest_reference_event_id: u64,
    pub retained_artifact_reference_event_id: u64,
    pub retained_vm_report_reference_event_id: u64,
    pub retained_local_attestation_reference_event_id: u64,
    pub retained_reference_event_id: u64,
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

#[derive(Clone, Copy)]
pub struct ModuleComputedGrantReference {
    pub computed_grant_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
}

#[derive(Clone, Copy)]
pub struct ModuleAuditRollbackReference<'a> {
    pub audit_record_hash: [u8; 32],
    pub rollback_plan_hash: [u8; 32],
    pub computed_grant_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
    pub local_approval_hash: [u8; 32],
    pub pre_load_service_inventory_hash: [u8; 32],
    pub cleanup_actions_hash: [u8; 32],
    pub denial_event_id: u64,
    pub retained_reference_event_id: u64,
    pub ram_only_service_slot_id: &'a str,
}

#[derive(Clone, Copy)]
pub struct ModuleServiceSlotReservation<'a> {
    pub reservation_hash: [u8; 32],
    pub retained_reference_event_id: u64,
    pub retained_audit_rollback_reference_event_id: u64,
    pub computed_grant_hash: [u8; 32],
    pub audit_record_hash: [u8; 32],
    pub rollback_plan_hash: [u8; 32],
    pub pre_load_service_inventory_hash: [u8; 32],
    pub ram_only_service_slot_id: &'a str,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateManifestReferenceCandidate {
    pub scope: &'static str,
    pub retained: bool,
    pub schema_ok: bool,
    pub event_reference: Option<ModuleManifestReference>,
    pub candidate_reference: Option<ModuleManifestReference>,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateArtifactReferenceCandidate {
    pub scope: &'static str,
    pub retained: bool,
    pub schema_ok: bool,
    pub event_reference: Option<ModuleCandidateArtifactReference>,
    pub candidate_reference: Option<ModuleCandidateArtifactReference>,
    pub manifest_event_id: Option<u64>,
    pub manifest_reference: Option<ModuleManifestReference>,
    pub retained_event_id: Option<u64>,
    pub retained_reference: Option<ModuleComputedGrantReference>,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateVmReportReferenceCandidate {
    pub scope: &'static str,
    pub retained: bool,
    pub schema_ok: bool,
    pub event_reference: Option<ModuleVmTestReportReference>,
    pub candidate_reference: Option<ModuleVmTestReportReference>,
    pub manifest_event_id: Option<u64>,
    pub manifest_reference: Option<ModuleManifestReference>,
    pub artifact_event_id: Option<u64>,
    pub artifact_reference: Option<ModuleCandidateArtifactReference>,
    pub retained_event_id: Option<u64>,
    pub retained_reference: Option<ModuleComputedGrantReference>,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateLocalAttestationReferenceCandidate {
    pub scope: &'static str,
    pub retained: bool,
    pub schema_ok: bool,
    pub event_reference: Option<ModuleLocalAttestationReference>,
    pub candidate_reference: Option<ModuleLocalAttestationReference>,
    pub manifest_event_id: Option<u64>,
    pub manifest_reference: Option<ModuleManifestReference>,
    pub artifact_event_id: Option<u64>,
    pub artifact_reference: Option<ModuleCandidateArtifactReference>,
    pub vm_report_event_id: Option<u64>,
    pub vm_report_reference: Option<ModuleVmTestReportReference>,
    pub retained_event_id: Option<u64>,
    pub retained_reference: Option<ModuleComputedGrantReference>,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateLocalApprovalReferenceCandidate {
    pub scope: &'static str,
    pub retained: bool,
    pub schema_ok: bool,
    pub event_reference: Option<ModuleLocalApprovalReference>,
    pub candidate_reference: Option<ModuleLocalApprovalReference>,
    pub manifest_event_id: Option<u64>,
    pub manifest_reference: Option<ModuleManifestReference>,
    pub artifact_event_id: Option<u64>,
    pub artifact_reference: Option<ModuleCandidateArtifactReference>,
    pub vm_report_event_id: Option<u64>,
    pub vm_report_reference: Option<ModuleVmTestReportReference>,
    pub attestation_event_id: Option<u64>,
    pub attestation_reference: Option<ModuleLocalAttestationReference>,
    pub retained_event_id: Option<u64>,
    pub retained_reference: Option<ModuleComputedGrantReference>,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateManifestEvaluation {
    pub status: &'static str,
    pub reason: &'static str,
    pub module_manifest_state: &'static str,
    pub accepted_manifest_hash: bool,
    pub can_load: bool,
    pub load_attempted: bool,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateArtifactEvaluation {
    pub status: &'static str,
    pub reason: &'static str,
    pub candidate_artifact_state: &'static str,
    pub accepted_artifact_hash: bool,
    pub can_load: bool,
    pub load_attempted: bool,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateVmReportEvaluation {
    pub status: &'static str,
    pub reason: &'static str,
    pub vm_test_report_state: &'static str,
    pub accepted_vm_report_hash: bool,
    pub can_load: bool,
    pub load_attempted: bool,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateLocalAttestationEvaluation {
    pub status: &'static str,
    pub reason: &'static str,
    pub local_attestation_state: &'static str,
    pub accepted_local_attestation_hash: bool,
    pub can_load: bool,
    pub load_attempted: bool,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateLocalApprovalEvaluation {
    pub status: &'static str,
    pub reason: &'static str,
    pub local_approval_state: &'static str,
    pub accepted_local_approval_hash: bool,
    pub can_load: bool,
    pub load_attempted: bool,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateRetainedCandidate {
    pub scope: &'static str,
    pub retained: bool,
    pub schema_ok: bool,
    pub event_reference: Option<ModuleComputedGrantReference>,
    pub candidate_reference: Option<ModuleComputedGrantReference>,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateRetainedCheck {
    pub status: &'static str,
    pub reason: &'static str,
    pub can_load: bool,
    pub load_attempted: bool,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateAuditRollbackReferenceCandidate<'a> {
    pub scope: &'a str,
    pub retained: bool,
    pub schema_ok: bool,
    pub event_reference: Option<ModuleAuditRollbackReference<'a>>,
    pub candidate_reference: Option<ModuleAuditRollbackReference<'a>>,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateAuditRollbackCandidate<'a> {
    pub retained_reference: bool,
    pub retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate<'a>,
    pub durable_audit_record: bool,
    pub rollback_plan: bool,
    pub audit_schema_ok: bool,
    pub rollback_schema_ok: bool,
    pub audit_binds_retained_grant: bool,
    pub audit_binds_manifest: bool,
    pub audit_binds_artifact: bool,
    pub audit_binds_vm_report: bool,
    pub audit_binds_local_attestation: bool,
    pub audit_binds_local_approval: bool,
    pub audit_binds_rollback_plan: bool,
    pub rollback_binds_artifact: bool,
    pub rollback_binds_service_slot: bool,
    pub ram_only_service_slot_allocated: bool,
    pub loader_available: bool,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateAuditRollbackEvaluation {
    pub status: &'static str,
    pub reason: &'static str,
    pub can_load: bool,
    pub load_attempted: bool,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateServiceSlotReservationCandidate<'a> {
    pub scope: &'a str,
    pub retained: bool,
    pub schema_ok: bool,
    pub grant_event_schema_ok: bool,
    pub audit_event_schema_ok: bool,
    pub grant_event_reference: Option<ModuleComputedGrantReference>,
    pub audit_event_reference: Option<ModuleAuditRollbackReference<'a>>,
    pub event_reservation: Option<ModuleServiceSlotReservation<'a>>,
    pub candidate_reservation: Option<ModuleServiceSlotReservation<'a>>,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateServiceSlotCandidate<'a> {
    pub retained_reference: Option<ModuleComputedGrantReference>,
    pub audit_rollback_reference: Option<ModuleAuditRollbackReference<'a>>,
    pub audit_rollback_valid: bool,
    pub service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate<'a>,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateServiceSlotEvaluation {
    pub status: &'static str,
    pub reason: &'static str,
    pub service_slot_state: &'static str,
    pub accepted_service_slot_reservation_hash: bool,
    pub can_load: bool,
    pub load_attempted: bool,
}

pub fn evaluate_manifest_reference(
    candidate: ModuleLoadGateManifestReferenceCandidate,
) -> ModuleLoadGateManifestEvaluation {
    if candidate.candidate_reference.is_none() {
        return manifest_result("missing", "retained_module_manifest_reference_missing");
    }
    if !method_eq(candidate.scope, "current_boot") {
        return manifest_result(
            "rejected",
            "retained_module_manifest_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return manifest_result(
            "rejected",
            "retained_module_manifest_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return manifest_result(
            "rejected",
            "retained_module_manifest_reference_wrong_schema_or_variant",
        );
    }
    if candidate.event_reference != candidate.candidate_reference {
        return manifest_result(
            "rejected",
            "retained_module_manifest_reference_substituted_record",
        );
    }
    let reference = candidate.candidate_reference.unwrap();
    if reference.manifest_reference_hash
        != computed_module_manifest_reference_hash(reference.manifest_hash)
    {
        return manifest_result(
            "rejected",
            "retained_module_manifest_reference_hash_mismatch",
        );
    }
    manifest_result(
        "retained_hash_reference_only",
        "retained_module_manifest_reference_not_authorizing",
    )
}

pub fn evaluate_artifact_reference(
    candidate: ModuleLoadGateArtifactReferenceCandidate,
) -> ModuleLoadGateArtifactEvaluation {
    let Some(reference) = candidate.candidate_reference else {
        return artifact_result("missing", "retained_candidate_artifact_reference_missing");
    };
    if let Some(reason) = common_reference_reason(
        candidate.scope,
        candidate.retained,
        candidate.schema_ok,
        candidate.event_reference == candidate.candidate_reference,
        "retained_candidate_artifact_reference",
    ) {
        return artifact_result("rejected", reason);
    }
    if reference.artifact_reference_hash
        != computed_module_candidate_artifact_reference_hash_from_sequences(
            reference.retained_manifest_reference_event_id,
            reference.retained_reference_event_id,
            reference.manifest_reference_hash,
            reference.manifest_hash,
            reference.computed_grant_hash,
            reference.artifact_hash,
            reference.vm_report_hash,
            reference.local_attestation_hash,
        )
    {
        return artifact_result(
            "rejected",
            "retained_candidate_artifact_reference_hash_mismatch",
        );
    }
    let (Some(event_id), Some(manifest)) =
        (candidate.manifest_event_id, candidate.manifest_reference)
    else {
        return artifact_result(
            "rejected",
            "retained_candidate_artifact_reference_manifest_reference_mismatch",
        );
    };
    if reference.retained_manifest_reference_event_id != event_id
        || reference.manifest_reference_hash != manifest.manifest_reference_hash
        || reference.manifest_hash != manifest.manifest_hash
    {
        return artifact_result(
            "rejected",
            "retained_candidate_artifact_reference_manifest_reference_mismatch",
        );
    }
    let (Some(event_id), Some(retained)) =
        (candidate.retained_event_id, candidate.retained_reference)
    else {
        return artifact_result(
            "rejected",
            "retained_candidate_artifact_reference_computed_grant_reference_mismatch",
        );
    };
    if reference.retained_reference_event_id != event_id
        || reference.computed_grant_hash != retained.computed_grant_hash
        || reference.manifest_hash != retained.manifest_hash
        || reference.vm_report_hash != retained.vm_report_hash
        || reference.local_attestation_hash != retained.local_attestation_hash
    {
        return artifact_result(
            "rejected",
            "retained_candidate_artifact_reference_computed_grant_reference_mismatch",
        );
    }
    if reference.artifact_hash != retained.artifact_hash {
        return artifact_result("rejected", "retained_candidate_artifact_hash_mismatch");
    }
    artifact_result(
        "retained_hash_reference_only",
        "retained_candidate_artifact_reference_not_authorizing",
    )
}

pub fn evaluate_vm_report_reference(
    candidate: ModuleLoadGateVmReportReferenceCandidate,
) -> ModuleLoadGateVmReportEvaluation {
    let Some(reference) = candidate.candidate_reference else {
        return vm_report_result("missing", "retained_vm_test_report_reference_missing");
    };
    if let Some(reason) = common_reference_reason(
        candidate.scope,
        candidate.retained,
        candidate.schema_ok,
        candidate.event_reference == candidate.candidate_reference,
        "retained_vm_test_report_reference",
    ) {
        return vm_report_result("rejected", reason);
    }
    if reference.report_reference_hash
        != computed_module_vm_test_report_reference_hash_from_sequences(
            reference.retained_manifest_reference_event_id,
            reference.retained_artifact_reference_event_id,
            reference.retained_reference_event_id,
            reference.manifest_reference_hash,
            reference.artifact_reference_hash,
            reference.manifest_hash,
            reference.artifact_hash,
            reference.computed_grant_hash,
            reference.vm_report_hash,
            reference.local_attestation_hash,
        )
    {
        return vm_report_result(
            "rejected",
            "retained_vm_test_report_reference_hash_mismatch",
        );
    }
    if !vm_manifest_matches(
        reference,
        candidate.manifest_event_id,
        candidate.manifest_reference,
    ) {
        return vm_report_result(
            "rejected",
            "retained_vm_test_report_reference_manifest_reference_mismatch",
        );
    }
    let (Some(event_id), Some(artifact)) =
        (candidate.artifact_event_id, candidate.artifact_reference)
    else {
        return vm_report_result(
            "rejected",
            "retained_vm_test_report_reference_artifact_reference_mismatch",
        );
    };
    if reference.retained_artifact_reference_event_id != event_id
        || reference.artifact_reference_hash != artifact.artifact_reference_hash
        || reference.manifest_reference_hash != artifact.manifest_reference_hash
        || reference.manifest_hash != artifact.manifest_hash
        || reference.artifact_hash != artifact.artifact_hash
        || reference.local_attestation_hash != artifact.local_attestation_hash
    {
        return vm_report_result(
            "rejected",
            "retained_vm_test_report_reference_artifact_reference_mismatch",
        );
    }
    if reference.vm_report_hash != artifact.vm_report_hash {
        return vm_report_result("rejected", "retained_vm_test_report_hash_mismatch");
    }
    let (Some(event_id), Some(retained)) =
        (candidate.retained_event_id, candidate.retained_reference)
    else {
        return vm_report_result(
            "rejected",
            "retained_vm_test_report_reference_computed_grant_reference_mismatch",
        );
    };
    if reference.retained_reference_event_id != event_id
        || reference.computed_grant_hash != retained.computed_grant_hash
        || reference.manifest_hash != retained.manifest_hash
        || reference.artifact_hash != retained.artifact_hash
        || reference.local_attestation_hash != retained.local_attestation_hash
    {
        return vm_report_result(
            "rejected",
            "retained_vm_test_report_reference_computed_grant_reference_mismatch",
        );
    }
    if reference.vm_report_hash != retained.vm_report_hash {
        return vm_report_result("rejected", "retained_vm_test_report_hash_mismatch");
    }
    vm_report_result(
        "retained_hash_reference_only",
        "retained_vm_test_report_reference_not_authorizing",
    )
}

pub fn evaluate_local_attestation_reference(
    candidate: ModuleLoadGateLocalAttestationReferenceCandidate,
) -> ModuleLoadGateLocalAttestationEvaluation {
    let Some(reference) = candidate.candidate_reference else {
        return attestation_result("missing", "retained_local_attestation_reference_missing");
    };
    if let Some(reason) = common_reference_reason(
        candidate.scope,
        candidate.retained,
        candidate.schema_ok,
        candidate.event_reference == candidate.candidate_reference,
        "retained_local_attestation_reference",
    ) {
        return attestation_result("rejected", reason);
    }
    if reference.attestation_reference_hash
        != computed_module_local_attestation_reference_hash_from_sequences(
            reference.retained_manifest_reference_event_id,
            reference.retained_artifact_reference_event_id,
            reference.retained_vm_report_reference_event_id,
            reference.retained_reference_event_id,
            reference.manifest_reference_hash,
            reference.artifact_reference_hash,
            reference.vm_report_reference_hash,
            reference.manifest_hash,
            reference.artifact_hash,
            reference.computed_grant_hash,
            reference.vm_report_hash,
            reference.local_attestation_hash,
        )
    {
        return attestation_result(
            "rejected",
            "retained_local_attestation_reference_hash_mismatch",
        );
    }
    if !attestation_manifest_matches(
        reference,
        candidate.manifest_event_id,
        candidate.manifest_reference,
    ) {
        return attestation_result(
            "rejected",
            "retained_local_attestation_reference_manifest_reference_mismatch",
        );
    }
    let (Some(event_id), Some(artifact)) =
        (candidate.artifact_event_id, candidate.artifact_reference)
    else {
        return attestation_result(
            "rejected",
            "retained_local_attestation_reference_artifact_reference_mismatch",
        );
    };
    if reference.retained_artifact_reference_event_id != event_id
        || reference.artifact_reference_hash != artifact.artifact_reference_hash
        || reference.manifest_reference_hash != artifact.manifest_reference_hash
        || reference.manifest_hash != artifact.manifest_hash
        || reference.artifact_hash != artifact.artifact_hash
        || reference.local_attestation_hash != artifact.local_attestation_hash
    {
        return attestation_result(
            "rejected",
            "retained_local_attestation_reference_artifact_reference_mismatch",
        );
    }
    let (Some(event_id), Some(report)) =
        (candidate.vm_report_event_id, candidate.vm_report_reference)
    else {
        return attestation_result(
            "rejected",
            "retained_local_attestation_reference_vm_report_reference_mismatch",
        );
    };
    if reference.retained_vm_report_reference_event_id != event_id
        || reference.vm_report_reference_hash != report.report_reference_hash
        || reference.artifact_reference_hash != report.artifact_reference_hash
        || reference.vm_report_hash != report.vm_report_hash
        || reference.local_attestation_hash != report.local_attestation_hash
    {
        return attestation_result(
            "rejected",
            "retained_local_attestation_reference_vm_report_reference_mismatch",
        );
    }
    let (Some(event_id), Some(retained)) =
        (candidate.retained_event_id, candidate.retained_reference)
    else {
        return attestation_result(
            "rejected",
            "retained_local_attestation_reference_computed_grant_reference_mismatch",
        );
    };
    if reference.retained_reference_event_id != event_id
        || reference.computed_grant_hash != retained.computed_grant_hash
        || reference.manifest_hash != retained.manifest_hash
        || reference.artifact_hash != retained.artifact_hash
        || reference.vm_report_hash != retained.vm_report_hash
        || reference.local_attestation_hash != retained.local_attestation_hash
    {
        return attestation_result(
            "rejected",
            "retained_local_attestation_reference_computed_grant_reference_mismatch",
        );
    }
    attestation_result(
        "retained_hash_reference_only",
        "retained_local_attestation_reference_not_authorizing",
    )
}

pub fn evaluate_local_approval_reference(
    candidate: ModuleLoadGateLocalApprovalReferenceCandidate,
) -> ModuleLoadGateLocalApprovalEvaluation {
    let Some(reference) = candidate.candidate_reference else {
        return approval_result("missing", "retained_local_approval_reference_missing");
    };
    if let Some(reason) = common_reference_reason(
        candidate.scope,
        candidate.retained,
        candidate.schema_ok,
        candidate.event_reference == candidate.candidate_reference,
        "retained_local_approval_reference",
    ) {
        return approval_result("rejected", reason);
    }
    if reference.approval_reference_hash
        != computed_module_local_approval_reference_hash_from_sequences(
            reference.retained_manifest_reference_event_id,
            reference.retained_artifact_reference_event_id,
            reference.retained_vm_report_reference_event_id,
            reference.retained_local_attestation_reference_event_id,
            reference.retained_reference_event_id,
            reference.manifest_reference_hash,
            reference.artifact_reference_hash,
            reference.vm_report_reference_hash,
            reference.local_attestation_reference_hash,
            reference.manifest_hash,
            reference.artifact_hash,
            reference.computed_grant_hash,
            reference.vm_report_hash,
            reference.local_attestation_hash,
            reference.local_approval_hash,
        )
    {
        return approval_result(
            "rejected",
            "retained_local_approval_reference_hash_mismatch",
        );
    }
    if !approval_manifest_matches(
        reference,
        candidate.manifest_event_id,
        candidate.manifest_reference,
    ) {
        return approval_result(
            "rejected",
            "retained_local_approval_reference_manifest_reference_mismatch",
        );
    }
    let (Some(event_id), Some(artifact)) =
        (candidate.artifact_event_id, candidate.artifact_reference)
    else {
        return approval_result(
            "rejected",
            "retained_local_approval_reference_artifact_reference_mismatch",
        );
    };
    if reference.retained_artifact_reference_event_id != event_id
        || reference.artifact_reference_hash != artifact.artifact_reference_hash
        || reference.manifest_reference_hash != artifact.manifest_reference_hash
        || reference.manifest_hash != artifact.manifest_hash
        || reference.artifact_hash != artifact.artifact_hash
        || reference.local_attestation_hash != artifact.local_attestation_hash
    {
        return approval_result(
            "rejected",
            "retained_local_approval_reference_artifact_reference_mismatch",
        );
    }
    let (Some(event_id), Some(report)) =
        (candidate.vm_report_event_id, candidate.vm_report_reference)
    else {
        return approval_result(
            "rejected",
            "retained_local_approval_reference_vm_report_reference_mismatch",
        );
    };
    if reference.retained_vm_report_reference_event_id != event_id
        || reference.vm_report_reference_hash != report.report_reference_hash
        || reference.artifact_reference_hash != report.artifact_reference_hash
        || reference.vm_report_hash != report.vm_report_hash
        || reference.local_attestation_hash != report.local_attestation_hash
    {
        return approval_result(
            "rejected",
            "retained_local_approval_reference_vm_report_reference_mismatch",
        );
    }
    let (Some(event_id), Some(attestation)) = (
        candidate.attestation_event_id,
        candidate.attestation_reference,
    ) else {
        return approval_result(
            "rejected",
            "retained_local_approval_reference_local_attestation_reference_mismatch",
        );
    };
    if reference.retained_local_attestation_reference_event_id != event_id
        || reference.local_attestation_reference_hash != attestation.attestation_reference_hash
        || reference.vm_report_reference_hash != attestation.vm_report_reference_hash
        || reference.local_attestation_hash != attestation.local_attestation_hash
    {
        return approval_result(
            "rejected",
            "retained_local_approval_reference_local_attestation_reference_mismatch",
        );
    }
    let (Some(event_id), Some(retained)) =
        (candidate.retained_event_id, candidate.retained_reference)
    else {
        return approval_result(
            "rejected",
            "retained_local_approval_reference_computed_grant_reference_mismatch",
        );
    };
    if reference.retained_reference_event_id != event_id
        || reference.computed_grant_hash != retained.computed_grant_hash
        || reference.manifest_hash != retained.manifest_hash
        || reference.artifact_hash != retained.artifact_hash
        || reference.vm_report_hash != retained.vm_report_hash
        || reference.local_attestation_hash != retained.local_attestation_hash
    {
        return approval_result(
            "rejected",
            "retained_local_approval_reference_computed_grant_reference_mismatch",
        );
    }
    approval_result(
        "retained_hash_reference_only",
        "retained_local_approval_reference_not_authorizing",
    )
}

pub fn evaluate_retained_candidate(
    candidate: ModuleLoadGateRetainedCandidate,
) -> ModuleLoadGateRetainedCheck {
    let Some(reference) = candidate.candidate_reference else {
        return retained_result("missing", "computed_capability_grant_reference_missing");
    };
    if !method_eq(candidate.scope, "current_boot") {
        return retained_result("rejected", "retained_reference_previous_boot_or_unretained");
    }
    if !candidate.retained {
        return retained_result("rejected", "retained_reference_stale_or_dropped_event_id");
    }
    if !candidate.schema_ok {
        return retained_result("rejected", "retained_reference_wrong_schema_or_variant");
    }
    let Some(event_reference) = candidate.event_reference else {
        return retained_result("rejected", "retained_reference_stale_or_dropped_event_id");
    };
    if !computed_grant_reference_matches(event_reference, reference) {
        return retained_result("rejected", "retained_reference_substituted_record");
    }
    if !module_computed_grant_reference_hashes_consistent(reference) {
        return retained_result("rejected", "retained_reference_hash_mismatch");
    }
    retained_result(
        "retained_hash_reference_only",
        "retained_computed_grant_reference_not_authorizing",
    )
}

pub fn evaluate_audit_rollback_candidate(
    candidate: ModuleLoadGateAuditRollbackCandidate<'_>,
) -> ModuleLoadGateAuditRollbackEvaluation {
    if !candidate.retained_reference {
        return audit_result("missing", "retained_computed_grant_reference_missing");
    }
    let check = evaluate_audit_rollback_reference(candidate.retained_audit_rollback_reference);
    if check.status != "retained_hash_reference_only" {
        return audit_result(check.status, check.reason);
    }
    let checks = [
        (
            candidate.durable_audit_record,
            "missing",
            "durable_audit_write_missing",
        ),
        (
            candidate.rollback_plan,
            "missing",
            "rollback_install_missing",
        ),
        (
            candidate.audit_schema_ok,
            "rejected",
            "durable_audit_record_schema_mismatch",
        ),
        (
            candidate.rollback_schema_ok,
            "rejected",
            "rollback_plan_schema_mismatch",
        ),
        (
            candidate.audit_binds_retained_grant,
            "rejected",
            "audit_retained_grant_hash_mismatch",
        ),
        (
            candidate.audit_binds_manifest,
            "rejected",
            "audit_manifest_hash_mismatch",
        ),
        (
            candidate.audit_binds_artifact,
            "rejected",
            "audit_artifact_hash_mismatch",
        ),
        (
            candidate.audit_binds_vm_report,
            "rejected",
            "audit_vm_test_report_hash_mismatch",
        ),
        (
            candidate.audit_binds_local_attestation,
            "rejected",
            "audit_local_attestation_hash_mismatch",
        ),
        (
            candidate.audit_binds_local_approval,
            "rejected",
            "local_approval_missing_or_mismatch",
        ),
        (
            candidate.audit_binds_rollback_plan,
            "rejected",
            "audit_rollback_plan_hash_mismatch",
        ),
        (
            candidate.rollback_binds_artifact,
            "rejected",
            "rollback_artifact_hash_mismatch",
        ),
        (
            candidate.rollback_binds_service_slot,
            "rejected",
            "rollback_service_slot_mismatch",
        ),
    ];
    for (ok, status, reason) in checks {
        if !ok {
            return audit_result(status, reason);
        }
    }
    if !candidate.ram_only_service_slot_allocated && !candidate.loader_available {
        return audit_result(
            "validated_non_authorizing",
            "loader_and_service_slot_missing",
        );
    }
    if !candidate.ram_only_service_slot_allocated {
        return audit_result("rejected", "ram_only_service_slot_unallocated");
    }
    if !candidate.loader_available {
        return audit_result("validated_non_authorizing", "module_loader_unimplemented");
    }
    audit_result("rejected", "positive_loader_path_unimplemented")
}

pub fn evaluate_service_slot_candidate(
    candidate: ModuleLoadGateServiceSlotCandidate<'_>,
) -> ModuleLoadGateServiceSlotEvaluation {
    let slot = candidate.service_slot_reservation;
    let Some(reservation) = slot.candidate_reservation else {
        return service_slot_result("missing", "retained_service_slot_reservation_missing");
    };
    let Some(grant) = candidate.retained_reference else {
        return service_slot_result("rejected", "retained_computed_grant_reference_missing");
    };
    let Some(audit) = candidate.audit_rollback_reference else {
        return service_slot_result("rejected", "retained_audit_rollback_reference_missing");
    };
    if !candidate.audit_rollback_valid {
        return service_slot_result(
            "rejected",
            "retained_audit_rollback_reference_not_valid_for_service_slot",
        );
    }
    if reservation.retained_reference_event_id != 27 {
        return service_slot_result(
            "rejected",
            "retained_service_slot_reservation_grant_reference_mismatch",
        );
    }
    if reservation.retained_audit_rollback_reference_event_id != 33 {
        return service_slot_result(
            "rejected",
            "retained_service_slot_reservation_audit_rollback_reference_mismatch",
        );
    }
    if !method_eq(slot.scope, "current_boot") || !slot.retained {
        return service_slot_result(
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
        );
    }
    if !slot.schema_ok {
        return service_slot_result(
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
        );
    }
    let Some(event_reservation) = slot.event_reservation else {
        return service_slot_result(
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
        );
    };
    if !service_slot_reservation_matches(event_reservation, reservation) {
        return service_slot_result(
            "rejected",
            "retained_service_slot_reservation_substituted_record",
        );
    }
    if !slot.grant_event_schema_ok || !slot.audit_event_schema_ok {
        return service_slot_result(
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
        );
    }
    let (Some(grant_event), Some(audit_event)) =
        (slot.grant_event_reference, slot.audit_event_reference)
    else {
        return service_slot_result(
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
        );
    };
    if !computed_grant_reference_matches(grant_event, grant)
        || !audit_rollback_reference_matches(audit_event, audit)
    {
        return service_slot_result(
            "rejected",
            "retained_service_slot_reservation_substituted_record",
        );
    }
    if reservation.computed_grant_hash != grant.computed_grant_hash
        || reservation.computed_grant_hash != audit.computed_grant_hash
    {
        return service_slot_result(
            "rejected",
            "retained_service_slot_reservation_computed_grant_hash_mismatch",
        );
    }
    if reservation.audit_record_hash != audit.audit_record_hash {
        return service_slot_result(
            "rejected",
            "retained_service_slot_reservation_audit_record_hash_mismatch",
        );
    }
    if reservation.rollback_plan_hash != audit.rollback_plan_hash {
        return service_slot_result(
            "rejected",
            "retained_service_slot_reservation_rollback_plan_hash_mismatch",
        );
    }
    if reservation.pre_load_service_inventory_hash != audit.pre_load_service_inventory_hash {
        return service_slot_result(
            "rejected",
            "retained_service_slot_reservation_pre_load_inventory_hash_mismatch",
        );
    }
    if reservation.ram_only_service_slot_id != audit.ram_only_service_slot_id
        || !ram_only_service_slot_id_valid(reservation.ram_only_service_slot_id)
    {
        return service_slot_result(
            "rejected",
            "retained_service_slot_reservation_service_slot_mismatch",
        );
    }
    if reservation.reservation_hash != computed_module_service_slot_reservation_hash(reservation) {
        return service_slot_result(
            "rejected",
            "retained_service_slot_reservation_hash_mismatch",
        );
    }
    service_slot_result(
        "retained_hash_reference_only_not_allocated",
        "retained_service_slot_reservation_not_allocated",
    )
}

fn common_reference_reason(
    scope: &str,
    retained: bool,
    schema_ok: bool,
    equal: bool,
    prefix: &str,
) -> Option<&'static str> {
    if !method_eq(scope, "current_boot") {
        return Some(match prefix {
            "retained_candidate_artifact_reference" => {
                "retained_candidate_artifact_reference_previous_boot_or_unretained"
            }
            "retained_vm_test_report_reference" => {
                "retained_vm_test_report_reference_previous_boot_or_unretained"
            }
            "retained_local_attestation_reference" => {
                "retained_local_attestation_reference_previous_boot_or_unretained"
            }
            _ => "retained_local_approval_reference_previous_boot_or_unretained",
        });
    }
    if !retained {
        return Some(match prefix {
            "retained_candidate_artifact_reference" => {
                "retained_candidate_artifact_reference_stale_or_dropped_event_id"
            }
            "retained_vm_test_report_reference" => {
                "retained_vm_test_report_reference_stale_or_dropped_event_id"
            }
            "retained_local_attestation_reference" => {
                "retained_local_attestation_reference_stale_or_dropped_event_id"
            }
            _ => "retained_local_approval_reference_stale_or_dropped_event_id",
        });
    }
    if !schema_ok {
        return Some(match prefix {
            "retained_candidate_artifact_reference" => {
                "retained_candidate_artifact_reference_wrong_schema_or_variant"
            }
            "retained_vm_test_report_reference" => {
                "retained_vm_test_report_reference_wrong_schema_or_variant"
            }
            "retained_local_attestation_reference" => {
                "retained_local_attestation_reference_wrong_schema_or_variant"
            }
            _ => "retained_local_approval_reference_wrong_schema_or_variant",
        });
    }
    if !equal {
        return Some(match prefix {
            "retained_candidate_artifact_reference" => {
                "retained_candidate_artifact_reference_substituted_record"
            }
            "retained_vm_test_report_reference" => {
                "retained_vm_test_report_reference_substituted_record"
            }
            "retained_local_attestation_reference" => {
                "retained_local_attestation_reference_substituted_record"
            }
            _ => "retained_local_approval_reference_substituted_record",
        });
    }
    None
}

fn vm_manifest_matches(
    r: ModuleVmTestReportReference,
    id: Option<u64>,
    value: Option<ModuleManifestReference>,
) -> bool {
    value.is_some_and(|v| {
        id == Some(r.retained_manifest_reference_event_id)
            && v.manifest_reference_hash == r.manifest_reference_hash
            && v.manifest_hash == r.manifest_hash
    })
}
fn attestation_manifest_matches(
    r: ModuleLocalAttestationReference,
    id: Option<u64>,
    value: Option<ModuleManifestReference>,
) -> bool {
    value.is_some_and(|v| {
        id == Some(r.retained_manifest_reference_event_id)
            && v.manifest_reference_hash == r.manifest_reference_hash
            && v.manifest_hash == r.manifest_hash
    })
}
fn approval_manifest_matches(
    r: ModuleLocalApprovalReference,
    id: Option<u64>,
    value: Option<ModuleManifestReference>,
) -> bool {
    value.is_some_and(|v| {
        id == Some(r.retained_manifest_reference_event_id)
            && v.manifest_reference_hash == r.manifest_reference_hash
            && v.manifest_hash == r.manifest_hash
    })
}
fn module_computed_grant_reference_hashes_consistent(r: ModuleComputedGrantReference) -> bool {
    r.computed_grant_hash
        == computed_module_grant_hash(
            r.manifest_hash,
            r.artifact_hash,
            r.vm_report_hash,
            r.local_attestation_hash,
        )
}
fn computed_grant_reference_matches(
    left: ModuleComputedGrantReference,
    right: ModuleComputedGrantReference,
) -> bool {
    left.computed_grant_hash == right.computed_grant_hash
        && left.manifest_hash == right.manifest_hash
        && left.artifact_hash == right.artifact_hash
        && left.vm_report_hash == right.vm_report_hash
        && left.local_attestation_hash == right.local_attestation_hash
}
fn audit_rollback_reference_matches(
    left: ModuleAuditRollbackReference<'_>,
    right: ModuleAuditRollbackReference<'_>,
) -> bool {
    left.audit_record_hash == right.audit_record_hash
        && left.rollback_plan_hash == right.rollback_plan_hash
        && left.computed_grant_hash == right.computed_grant_hash
        && left.manifest_hash == right.manifest_hash
        && left.artifact_hash == right.artifact_hash
        && left.vm_report_hash == right.vm_report_hash
        && left.local_attestation_hash == right.local_attestation_hash
        && left.local_approval_hash == right.local_approval_hash
        && left.pre_load_service_inventory_hash == right.pre_load_service_inventory_hash
        && left.cleanup_actions_hash == right.cleanup_actions_hash
        && left.denial_event_id == right.denial_event_id
        && left.retained_reference_event_id == right.retained_reference_event_id
        && left.ram_only_service_slot_id == right.ram_only_service_slot_id
}
fn service_slot_reservation_matches(
    left: ModuleServiceSlotReservation<'_>,
    right: ModuleServiceSlotReservation<'_>,
) -> bool {
    left.reservation_hash == right.reservation_hash
        && left.retained_reference_event_id == right.retained_reference_event_id
        && left.retained_audit_rollback_reference_event_id
            == right.retained_audit_rollback_reference_event_id
        && left.computed_grant_hash == right.computed_grant_hash
        && left.audit_record_hash == right.audit_record_hash
        && left.rollback_plan_hash == right.rollback_plan_hash
        && left.pre_load_service_inventory_hash == right.pre_load_service_inventory_hash
        && left.ram_only_service_slot_id == right.ram_only_service_slot_id
}
fn evaluate_audit_rollback_reference(
    c: ModuleLoadGateAuditRollbackReferenceCandidate<'_>,
) -> ModuleLoadGateRetainedCheck {
    let Some(r) = c.candidate_reference else {
        return retained_result("missing", "retained_audit_rollback_reference_missing");
    };
    if !method_eq(c.scope, "current_boot") {
        return retained_result(
            "rejected",
            "retained_audit_rollback_reference_previous_boot_or_unretained",
        );
    };
    if !c.retained {
        return retained_result(
            "rejected",
            "retained_audit_rollback_reference_stale_or_dropped_event_id",
        );
    };
    if !c.schema_ok {
        return retained_result(
            "rejected",
            "retained_audit_rollback_reference_wrong_schema_or_variant",
        );
    };
    let Some(event_reference) = c.event_reference else {
        return retained_result(
            "rejected",
            "retained_audit_rollback_reference_stale_or_dropped_event_id",
        );
    };
    if !audit_rollback_reference_matches(event_reference, r) {
        return retained_result(
            "rejected",
            "retained_audit_rollback_reference_substituted_record",
        );
    };
    if r.ram_only_service_slot_id != crate::module_types::MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID
    {
        return retained_result("rejected", "retained_audit_rollback_service_slot_mismatch");
    }
    if r.denial_event_id != 31 || r.retained_reference_event_id != 27 {
        return retained_result(
            "rejected",
            "retained_audit_rollback_reference_substituted_record",
        );
    };
    if r.computed_grant_hash
        != computed_module_grant_hash(
            r.manifest_hash,
            r.artifact_hash,
            r.vm_report_hash,
            r.local_attestation_hash,
        )
    {
        return retained_result(
            "rejected",
            "retained_audit_rollback_computed_grant_hash_mismatch",
        );
    };
    if r.rollback_plan_hash
        != computed_module_rollback_plan_hash(
            r.artifact_hash,
            r.pre_load_service_inventory_hash,
            r.ram_only_service_slot_id,
            r.cleanup_actions_hash,
        )
    {
        return retained_result("rejected", "retained_rollback_plan_hash_mismatch");
    };
    if r.audit_record_hash != computed_module_audit_record_hash(r) {
        return retained_result("rejected", "retained_audit_record_hash_mismatch");
    };
    retained_result(
        "retained_hash_reference_only",
        "retained_audit_rollback_reference_not_authorizing",
    )
}
fn manifest_result(status: &'static str, reason: &'static str) -> ModuleLoadGateManifestEvaluation {
    ModuleLoadGateManifestEvaluation {
        status,
        reason,
        module_manifest_state: if status == "retained_hash_reference_only" {
            "retained_hash_reference_only"
        } else if status == "rejected" {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_manifest_hash: status == "retained_hash_reference_only",
        can_load: false,
        load_attempted: false,
    }
}
fn artifact_result(status: &'static str, reason: &'static str) -> ModuleLoadGateArtifactEvaluation {
    ModuleLoadGateArtifactEvaluation {
        status,
        reason,
        candidate_artifact_state: if status == "retained_hash_reference_only" {
            "retained_hash_reference_only"
        } else if status == "rejected" {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_artifact_hash: status == "retained_hash_reference_only",
        can_load: false,
        load_attempted: false,
    }
}
fn vm_report_result(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateVmReportEvaluation {
    ModuleLoadGateVmReportEvaluation {
        status,
        reason,
        vm_test_report_state: if status == "retained_hash_reference_only" {
            "retained_hash_reference_only"
        } else if status == "rejected" {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_vm_report_hash: status == "retained_hash_reference_only",
        can_load: false,
        load_attempted: false,
    }
}
fn attestation_result(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateLocalAttestationEvaluation {
    ModuleLoadGateLocalAttestationEvaluation {
        status,
        reason,
        local_attestation_state: if status == "retained_hash_reference_only" {
            "retained_hash_reference_only"
        } else if status == "rejected" {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_local_attestation_hash: status == "retained_hash_reference_only",
        can_load: false,
        load_attempted: false,
    }
}
fn approval_result(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateLocalApprovalEvaluation {
    ModuleLoadGateLocalApprovalEvaluation {
        status,
        reason,
        local_approval_state: if status == "retained_hash_reference_only" {
            "retained_hash_reference_only"
        } else if status == "rejected" {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_local_approval_hash: status == "retained_hash_reference_only",
        can_load: false,
        load_attempted: false,
    }
}
fn retained_result(status: &'static str, reason: &'static str) -> ModuleLoadGateRetainedCheck {
    ModuleLoadGateRetainedCheck {
        status,
        reason,
        can_load: false,
        load_attempted: false,
    }
}
fn audit_result(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateAuditRollbackEvaluation {
    ModuleLoadGateAuditRollbackEvaluation {
        status,
        reason,
        can_load: false,
        load_attempted: false,
    }
}
fn service_slot_result(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateServiceSlotEvaluation {
    ModuleLoadGateServiceSlotEvaluation {
        status,
        reason,
        service_slot_state: if status == "retained_hash_reference_only_not_allocated" {
            "retained_hash_reference_only_not_allocated"
        } else if status == "rejected" {
            "rejected_retained_reference"
        } else {
            "unallocated"
        },
        accepted_service_slot_reservation_hash: status
            == "retained_hash_reference_only_not_allocated",
        can_load: false,
        load_attempted: false,
    }
}

pub fn ram_only_service_slot_id_valid(value: &str) -> bool {
    let Some(slot) = value.strip_prefix("ram_only:") else {
        return false;
    };
    !slot.is_empty()
        && value.len() <= 96
        && slot
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
}

struct HashLines(Sha256);

fn current_boot_event_id(value: u64) -> [u8; 27] {
    const PREFIX: &[u8; 19] = b"event.current_boot.";
    let mut text = [0u8; 27];
    text[..PREFIX.len()].copy_from_slice(PREFIX);
    let mut divisor = 10_000_000u64;
    for digit in &mut text[PREFIX.len()..] {
        *digit = b'0' + ((value / divisor) % 10) as u8;
        divisor /= 10;
    }
    text
}

impl HashLines {
    fn new() -> Self {
        Self(Sha256::new())
    }
    fn line(&mut self, name: &[u8], value: &str, newline: bool) {
        self.0.update(name);
        self.0.update(b"=");
        self.0.update(value.as_bytes());
        if newline {
            self.0.update(b"\n");
        }
    }
    fn raw(&mut self, value: &'static [u8], newline: bool) {
        self.0.update(value);
        if newline {
            self.0.update(b"\n");
        }
    }
    fn hash(&mut self, name: &'static [u8], value: [u8; 32], newline: bool) {
        self.0.update(name);
        self.0.update(b"=");
        let mut hex = [0u8; 64];
        for (i, b) in value.iter().enumerate() {
            hex[i * 2] = b"0123456789abcdef"[(b >> 4) as usize];
            hex[i * 2 + 1] = b"0123456789abcdef"[(b & 15) as usize];
        }
        self.0.update(hex);
        if newline {
            self.0.update(b"\n");
        }
    }
    fn event(&mut self, name: &'static [u8], value: u64, newline: bool) {
        self.0.update(name);
        self.0.update(b"=");
        self.0.update(current_boot_event_id(value));
        if newline {
            self.0.update(b"\n");
        }
    }
    fn finish(self) -> [u8; 32] {
        let digest = self.0.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        out
    }
}

fn base(hash: &mut HashLines, canonical: &'static [u8], schema: &'static [u8]) {
    hash.raw(b"canonicalization=", false);
    hash.raw(canonical, true);
    hash.raw(b"schema=", false);
    hash.raw(schema, true);
    hash.raw(b"requested_capability=cap.module.load_ephemeral", true);
    hash.raw(b"load_mode=ram_only", true);
    hash.raw(b"subject=agent.session.serial", true);
    hash.raw(b"resource=live_service_graph", true);
    hash.raw(b"scope=current_boot", true);
}
pub fn computed_module_manifest_reference_hash(manifest_hash: [u8; 32]) -> [u8; 32] {
    let mut h = HashLines::new();
    base(
        &mut h,
        b"raios.module_manifest_reference.canonical.v0",
        b"raios.module_manifest_reference.v0",
    );
    h.raw(b"manifest_schema=raios.module_manifest.v0", true);
    h.hash(b"manifest_sha256", manifest_hash, true);
    h.raw(b"authorizes_guest_load=false", true);
    h.raw(b"service_inventory_change=none", true);
    h.raw(b"load_attempted=false", false);
    h.finish()
}
pub fn computed_module_grant_hash(m: [u8; 32], a: [u8; 32], v: [u8; 32], l: [u8; 32]) -> [u8; 32] {
    let mut h = HashLines::new();
    base(
        &mut h,
        b"raios.computed_capability_grant.canonical.v0",
        b"raios.computed_capability_grant.v0",
    );
    h.hash(b"manifest_sha256", m, true);
    h.hash(b"candidate_artifact_sha256", a, true);
    h.hash(b"vm_test_report_sha256", v, true);
    h.hash(b"local_attestation_sha256", l, true);
    h.raw(b"grants_load_now=false", true);
    h.raw(b"authorizes_guest_load=false", true);
    h.raw(b"service_inventory_change=none", true);
    h.raw(b"load_attempted=false", false);
    h.finish()
}
pub fn computed_module_candidate_artifact_reference_hash_from_sequences(
    mi: u64,
    ri: u64,
    mr: [u8; 32],
    m: [u8; 32],
    g: [u8; 32],
    a: [u8; 32],
    v: [u8; 32],
    l: [u8; 32],
) -> [u8; 32] {
    let mut h = HashLines::new();
    base(
        &mut h,
        b"raios.module_candidate_artifact_reference.canonical.v0",
        b"raios.module_candidate_artifact_reference.v0",
    );
    h.event(b"retained_manifest_reference_event_id", mi, true);
    h.event(b"retained_reference_event_id", ri, true);
    h.hash(b"manifest_reference_sha256", mr, true);
    h.hash(b"manifest_sha256", m, true);
    h.hash(b"computed_capability_grant_sha256", g, true);
    h.hash(b"candidate_artifact_sha256", a, true);
    h.hash(b"vm_test_report_sha256", v, true);
    h.hash(b"local_attestation_sha256", l, true);
    h.raw(b"accepts_artifact_bytes=false", true);
    h.raw(b"loads_artifact=false", true);
    h.raw(b"authorizes_guest_load=false", true);
    h.raw(b"service_inventory_change=none", true);
    h.raw(b"load_attempted=false", false);
    h.finish()
}
pub fn computed_module_vm_test_report_reference_hash_from_sequences(
    mi: u64,
    ai: u64,
    ri: u64,
    mr: [u8; 32],
    ar: [u8; 32],
    m: [u8; 32],
    a: [u8; 32],
    g: [u8; 32],
    v: [u8; 32],
    l: [u8; 32],
) -> [u8; 32] {
    let mut h = HashLines::new();
    base(
        &mut h,
        b"raios.module_vm_test_report_reference.canonical.v0",
        b"raios.module_vm_test_report_reference.v0",
    );
    h.event(b"retained_manifest_reference_event_id", mi, true);
    h.event(b"retained_artifact_reference_event_id", ai, true);
    h.event(b"retained_reference_event_id", ri, true);
    h.hash(b"manifest_reference_sha256", mr, true);
    h.hash(b"artifact_reference_sha256", ar, true);
    h.hash(b"manifest_sha256", m, true);
    h.hash(b"candidate_artifact_sha256", a, true);
    h.hash(b"computed_capability_grant_sha256", g, true);
    h.hash(b"vm_test_report_sha256", v, true);
    h.hash(b"local_attestation_sha256", l, true);
    h.raw(b"accepts_vm_report_json=false", true);
    h.raw(b"accepts_artifact_bytes=false", true);
    h.raw(b"loads_artifact=false", true);
    h.raw(b"authorizes_guest_load=false", true);
    h.raw(b"service_inventory_change=none", true);
    h.raw(b"load_attempted=false", false);
    h.finish()
}
pub fn computed_module_local_attestation_reference_hash_from_sequences(
    mi: u64,
    ai: u64,
    vi: u64,
    ri: u64,
    mr: [u8; 32],
    ar: [u8; 32],
    vr: [u8; 32],
    m: [u8; 32],
    a: [u8; 32],
    g: [u8; 32],
    v: [u8; 32],
    l: [u8; 32],
) -> [u8; 32] {
    let mut h = HashLines::new();
    base(
        &mut h,
        b"raios.module_local_attestation_reference.canonical.v0",
        b"raios.module_local_attestation_reference.v0",
    );
    h.event(b"retained_manifest_reference_event_id", mi, true);
    h.event(b"retained_artifact_reference_event_id", ai, true);
    h.event(b"retained_vm_report_reference_event_id", vi, true);
    h.event(b"retained_reference_event_id", ri, true);
    h.hash(b"manifest_reference_sha256", mr, true);
    h.hash(b"artifact_reference_sha256", ar, true);
    h.hash(b"vm_test_report_reference_sha256", vr, true);
    h.hash(b"manifest_sha256", m, true);
    h.hash(b"candidate_artifact_sha256", a, true);
    h.hash(b"computed_capability_grant_sha256", g, true);
    h.hash(b"vm_test_report_sha256", v, true);
    h.hash(b"local_attestation_sha256", l, true);
    h.raw(b"accepts_local_attestation_json=false", true);
    h.raw(b"accepts_artifact_bytes=false", true);
    h.raw(b"loads_artifact=false", true);
    h.raw(b"authorizes_guest_load=false", true);
    h.raw(b"service_inventory_change=none", true);
    h.raw(b"load_attempted=false", false);
    h.finish()
}
pub fn computed_module_local_approval_reference_hash_from_sequences(
    mi: u64,
    ai: u64,
    vi: u64,
    li: u64,
    ri: u64,
    mr: [u8; 32],
    ar: [u8; 32],
    vr: [u8; 32],
    lr: [u8; 32],
    m: [u8; 32],
    a: [u8; 32],
    g: [u8; 32],
    v: [u8; 32],
    l: [u8; 32],
    p: [u8; 32],
) -> [u8; 32] {
    let mut h = HashLines::new();
    base(
        &mut h,
        b"raios.module_local_approval_reference.canonical.v0",
        b"raios.module_local_approval_reference.v0",
    );
    h.event(b"retained_manifest_reference_event_id", mi, true);
    h.event(b"retained_artifact_reference_event_id", ai, true);
    h.event(b"retained_vm_report_reference_event_id", vi, true);
    h.event(b"retained_local_attestation_reference_event_id", li, true);
    h.event(b"retained_reference_event_id", ri, true);
    h.hash(b"manifest_reference_sha256", mr, true);
    h.hash(b"artifact_reference_sha256", ar, true);
    h.hash(b"vm_test_report_reference_sha256", vr, true);
    h.hash(b"local_attestation_reference_sha256", lr, true);
    h.hash(b"manifest_sha256", m, true);
    h.hash(b"candidate_artifact_sha256", a, true);
    h.hash(b"computed_capability_grant_sha256", g, true);
    h.hash(b"vm_test_report_sha256", v, true);
    h.hash(b"local_attestation_sha256", l, true);
    h.hash(b"local_approval_sha256", p, true);
    h.raw(b"accepts_local_approval_text=false", true);
    h.raw(b"accepts_artifact_bytes=false", true);
    h.raw(b"loads_artifact=false", true);
    h.raw(b"authorizes_guest_load=false", true);
    h.raw(b"service_inventory_change=none", true);
    h.raw(b"load_attempted=false", false);
    h.finish()
}
pub fn computed_module_rollback_plan_hash(
    a: [u8; 32],
    i: [u8; 32],
    slot: &str,
    c: [u8; 32],
) -> [u8; 32] {
    let mut h = HashLines::new();
    h.raw(b"canonicalization=raios.rollback_plan.canonical.v0", true);
    h.raw(b"schema=raios.rollback_plan.v0", true);
    h.raw(b"load_mode=ram_only", true);
    h.raw(b"scope=current_boot", true);
    h.hash(b"artifact_sha256", a, true);
    h.hash(b"pre_load_service_inventory_sha256", i, true);
    h.line(b"ram_only_service_slot_id", slot, true);
    h.hash(b"cleanup_actions_sha256", c, true);
    h.raw(b"service_inventory_change=none", true);
    h.raw(b"load_attempted=false", false);
    h.finish()
}
pub fn computed_module_audit_record_hash(r: ModuleAuditRollbackReference<'_>) -> [u8; 32] {
    let mut h = HashLines::new();
    base(
        &mut h,
        b"raios.audit_record.canonical.v0",
        b"raios.audit_record.v0",
    );
    h.event(b"denial_event_id", r.denial_event_id, true);
    h.event(
        b"retained_reference_event_id",
        r.retained_reference_event_id,
        true,
    );
    h.hash(
        b"computed_capability_grant_sha256",
        r.computed_grant_hash,
        true,
    );
    h.hash(b"manifest_sha256", r.manifest_hash, true);
    h.hash(b"candidate_artifact_sha256", r.artifact_hash, true);
    h.hash(b"vm_test_report_sha256", r.vm_report_hash, true);
    h.hash(b"local_attestation_sha256", r.local_attestation_hash, true);
    h.hash(b"local_approval_sha256", r.local_approval_hash, true);
    h.hash(b"rollback_plan_sha256", r.rollback_plan_hash, true);
    h.line(
        b"ram_only_service_slot_id",
        r.ram_only_service_slot_id,
        true,
    );
    h.raw(b"grants_load_now=false", true);
    h.raw(b"authorizes_guest_load=false", true);
    h.raw(b"service_inventory_change=none", true);
    h.raw(b"load_attempted=false", false);
    h.finish()
}
pub fn computed_module_service_slot_reservation_hash(
    r: ModuleServiceSlotReservation<'_>,
) -> [u8; 32] {
    let mut h = HashLines::new();
    h.raw(
        b"canonicalization=raios.module_service_slot_reservation.canonical.v0",
        true,
    );
    h.raw(b"schema=raios.module_service_slot_reservation.v0", true);
    h.raw(b"load_mode=ram_only", true);
    h.raw(b"scope=current_boot", true);
    h.event(
        b"retained_reference_event_id",
        r.retained_reference_event_id,
        true,
    );
    h.event(
        b"retained_audit_rollback_reference_event_id",
        r.retained_audit_rollback_reference_event_id,
        true,
    );
    h.hash(
        b"computed_capability_grant_sha256",
        r.computed_grant_hash,
        true,
    );
    h.hash(b"audit_record_sha256", r.audit_record_hash, true);
    h.hash(b"rollback_plan_sha256", r.rollback_plan_hash, true);
    h.hash(
        b"pre_load_service_inventory_sha256",
        r.pre_load_service_inventory_hash,
        true,
    );
    h.line(
        b"ram_only_service_slot_id",
        r.ram_only_service_slot_id,
        true,
    );
    h.raw(b"service_inventory_change=none", true);
    h.raw(b"load_attempted=false", false);
    h.finish()
}

const fn load_case(
    name: &'static str,
    status: &'static str,
    reason: &'static str,
) -> (&'static str, &'static str, &'static str) {
    (name, status, reason)
}
pub const MODULE_LOAD_GATE_REFERENCE_CASES: &[(&str, &str, &str)] = &[
    load_case(
        "missing_retained_manifest_reference",
        "missing",
        "retained_module_manifest_reference_missing",
    ),
    load_case(
        "accepted_current_boot_manifest_still_denied",
        "retained_hash_reference_only",
        "retained_module_manifest_reference_not_authorizing",
    ),
    load_case(
        "stale_dropped_manifest_reference_event_id",
        "rejected",
        "retained_module_manifest_reference_stale_or_dropped_event_id",
    ),
    load_case(
        "previous_boot_or_unretained_manifest_reference",
        "rejected",
        "retained_module_manifest_reference_previous_boot_or_unretained",
    ),
    load_case(
        "wrong_schema_or_variant",
        "rejected",
        "retained_module_manifest_reference_wrong_schema_or_variant",
    ),
    load_case(
        "substituted_manifest_reference_record",
        "rejected",
        "retained_module_manifest_reference_substituted_record",
    ),
    load_case(
        "manifest_reference_hash_mismatch",
        "rejected",
        "retained_module_manifest_reference_hash_mismatch",
    ),
    load_case(
        "missing_retained_candidate_artifact_reference",
        "missing",
        "retained_candidate_artifact_reference_missing",
    ),
    load_case(
        "accepted_current_boot_artifact_still_denied",
        "retained_hash_reference_only",
        "retained_candidate_artifact_reference_not_authorizing",
    ),
    load_case(
        "stale_dropped_retained_artifact_reference_event_id",
        "rejected",
        "retained_candidate_artifact_reference_stale_or_dropped_event_id",
    ),
    load_case(
        "previous_boot_or_unretained_artifact_reference",
        "rejected",
        "retained_candidate_artifact_reference_previous_boot_or_unretained",
    ),
    load_case(
        "wrong_schema_or_variant",
        "rejected",
        "retained_candidate_artifact_reference_wrong_schema_or_variant",
    ),
    load_case(
        "substituted_artifact_reference_record",
        "rejected",
        "retained_candidate_artifact_reference_substituted_record",
    ),
    load_case(
        "artifact_reference_hash_mismatch",
        "rejected",
        "retained_candidate_artifact_reference_hash_mismatch",
    ),
    load_case(
        "manifest_reference_mismatch",
        "rejected",
        "retained_candidate_artifact_reference_manifest_reference_mismatch",
    ),
    load_case(
        "computed_grant_reference_mismatch",
        "rejected",
        "retained_candidate_artifact_reference_computed_grant_reference_mismatch",
    ),
    load_case(
        "missing_retained_vm_test_report_reference",
        "missing",
        "retained_vm_test_report_reference_missing",
    ),
    load_case(
        "accepted_current_boot_report_still_denied",
        "retained_hash_reference_only",
        "retained_vm_test_report_reference_not_authorizing",
    ),
    load_case(
        "stale_dropped_retained_vm_test_report_reference_event_id",
        "rejected",
        "retained_vm_test_report_reference_stale_or_dropped_event_id",
    ),
    load_case(
        "previous_boot_or_unretained_vm_test_report_reference",
        "rejected",
        "retained_vm_test_report_reference_previous_boot_or_unretained",
    ),
    load_case(
        "wrong_schema_or_variant",
        "rejected",
        "retained_vm_test_report_reference_wrong_schema_or_variant",
    ),
    load_case(
        "substituted_vm_test_report_reference_record",
        "rejected",
        "retained_vm_test_report_reference_substituted_record",
    ),
    load_case(
        "vm_test_report_reference_hash_mismatch",
        "rejected",
        "retained_vm_test_report_reference_hash_mismatch",
    ),
    load_case(
        "manifest_reference_mismatch",
        "rejected",
        "retained_vm_test_report_reference_manifest_reference_mismatch",
    ),
    load_case(
        "artifact_reference_mismatch",
        "rejected",
        "retained_vm_test_report_reference_artifact_reference_mismatch",
    ),
    load_case(
        "computed_grant_reference_mismatch",
        "rejected",
        "retained_vm_test_report_reference_computed_grant_reference_mismatch",
    ),
    load_case(
        "vm_test_report_hash_mismatch",
        "rejected",
        "retained_vm_test_report_hash_mismatch",
    ),
    load_case(
        "missing_retained_local_attestation_reference",
        "missing",
        "retained_local_attestation_reference_missing",
    ),
    load_case(
        "accepted_current_boot_attestation_still_denied",
        "retained_hash_reference_only",
        "retained_local_attestation_reference_not_authorizing",
    ),
    load_case(
        "stale_dropped_retained_local_attestation_reference_event_id",
        "rejected",
        "retained_local_attestation_reference_stale_or_dropped_event_id",
    ),
    load_case(
        "previous_boot_or_unretained_local_attestation_reference",
        "rejected",
        "retained_local_attestation_reference_previous_boot_or_unretained",
    ),
    load_case(
        "wrong_schema_or_variant",
        "rejected",
        "retained_local_attestation_reference_wrong_schema_or_variant",
    ),
    load_case(
        "substituted_local_attestation_reference_record",
        "rejected",
        "retained_local_attestation_reference_substituted_record",
    ),
    load_case(
        "local_attestation_reference_hash_mismatch",
        "rejected",
        "retained_local_attestation_reference_hash_mismatch",
    ),
    load_case(
        "manifest_reference_mismatch",
        "rejected",
        "retained_local_attestation_reference_manifest_reference_mismatch",
    ),
    load_case(
        "artifact_reference_mismatch",
        "rejected",
        "retained_local_attestation_reference_artifact_reference_mismatch",
    ),
    load_case(
        "vm_report_reference_mismatch",
        "rejected",
        "retained_local_attestation_reference_vm_report_reference_mismatch",
    ),
    load_case(
        "computed_grant_reference_mismatch",
        "rejected",
        "retained_local_attestation_reference_computed_grant_reference_mismatch",
    ),
    load_case(
        "missing_retained_local_approval_reference",
        "missing",
        "retained_local_approval_reference_missing",
    ),
    load_case(
        "accepted_current_boot_approval_still_denied",
        "retained_hash_reference_only",
        "retained_local_approval_reference_not_authorizing",
    ),
    load_case(
        "stale_dropped_retained_local_approval_reference_event_id",
        "rejected",
        "retained_local_approval_reference_stale_or_dropped_event_id",
    ),
    load_case(
        "previous_boot_or_unretained_local_approval_reference",
        "rejected",
        "retained_local_approval_reference_previous_boot_or_unretained",
    ),
    load_case(
        "wrong_schema_or_variant",
        "rejected",
        "retained_local_approval_reference_wrong_schema_or_variant",
    ),
    load_case(
        "substituted_local_approval_reference_record",
        "rejected",
        "retained_local_approval_reference_substituted_record",
    ),
    load_case(
        "local_approval_reference_hash_mismatch",
        "rejected",
        "retained_local_approval_reference_hash_mismatch",
    ),
    load_case(
        "manifest_reference_mismatch",
        "rejected",
        "retained_local_approval_reference_manifest_reference_mismatch",
    ),
    load_case(
        "artifact_reference_mismatch",
        "rejected",
        "retained_local_approval_reference_artifact_reference_mismatch",
    ),
    load_case(
        "vm_report_reference_mismatch",
        "rejected",
        "retained_local_approval_reference_vm_report_reference_mismatch",
    ),
    load_case(
        "local_attestation_reference_mismatch",
        "rejected",
        "retained_local_approval_reference_local_attestation_reference_mismatch",
    ),
    load_case(
        "computed_grant_reference_mismatch",
        "rejected",
        "retained_local_approval_reference_computed_grant_reference_mismatch",
    ),
];

pub const MODULE_LOAD_GATE_RETAINED_CASES: &[(&str, &str, &str)] = &[
    load_case(
        "missing_retained_reference",
        "missing",
        "computed_capability_grant_reference_missing",
    ),
    load_case(
        "accepted_current_boot_reference_still_denied",
        "retained_hash_reference_only",
        "retained_computed_grant_reference_not_authorizing",
    ),
    load_case(
        "stale_dropped_retained_reference_event_id",
        "rejected",
        "retained_reference_stale_or_dropped_event_id",
    ),
    load_case(
        "previous_boot_or_unretained_reference",
        "rejected",
        "retained_reference_previous_boot_or_unretained",
    ),
    load_case(
        "wrong_schema_or_variant_substitution",
        "rejected",
        "retained_reference_wrong_schema_or_variant",
    ),
    load_case(
        "substituted_retained_reference_record",
        "rejected",
        "retained_reference_substituted_record",
    ),
    load_case(
        "mismatched_computed_grant_hash",
        "rejected",
        "retained_reference_hash_mismatch",
    ),
];

pub const MODULE_LOAD_GATE_AUDIT_ROLLBACK_CASES: &[(&str, &str, &str)] = &[
    load_case(
        "missing_retained_audit_rollback_reference",
        "missing",
        "retained_audit_rollback_reference_missing",
    ),
    load_case(
        "stale_dropped_retained_audit_rollback_reference_event_id",
        "rejected",
        "retained_audit_rollback_reference_stale_or_dropped_event_id",
    ),
    load_case(
        "previous_boot_or_unretained_audit_rollback_reference",
        "rejected",
        "retained_audit_rollback_reference_previous_boot_or_unretained",
    ),
    load_case(
        "retained_audit_rollback_wrong_schema_or_variant",
        "rejected",
        "retained_audit_rollback_reference_wrong_schema_or_variant",
    ),
    load_case(
        "substituted_retained_audit_rollback_reference",
        "rejected",
        "retained_audit_rollback_reference_substituted_record",
    ),
    load_case(
        "retained_audit_rollback_computed_grant_hash_mismatch",
        "rejected",
        "retained_audit_rollback_computed_grant_hash_mismatch",
    ),
    load_case(
        "retained_audit_record_hash_mismatch",
        "rejected",
        "retained_audit_record_hash_mismatch",
    ),
    load_case(
        "retained_rollback_plan_hash_mismatch",
        "rejected",
        "retained_rollback_plan_hash_mismatch",
    ),
    load_case(
        "retained_audit_rollback_service_slot_mismatch",
        "rejected",
        "retained_audit_rollback_service_slot_mismatch",
    ),
    load_case(
        "missing_durable_audit_record",
        "missing",
        "durable_audit_write_missing",
    ),
    load_case(
        "missing_rollback_plan",
        "missing",
        "rollback_install_missing",
    ),
    load_case(
        "durable_audit_record_schema_mismatch",
        "rejected",
        "durable_audit_record_schema_mismatch",
    ),
    load_case(
        "rollback_plan_schema_mismatch",
        "rejected",
        "rollback_plan_schema_mismatch",
    ),
    load_case(
        "valid_audit_and_rollback_still_denied",
        "validated_non_authorizing",
        "loader_and_service_slot_missing",
    ),
    load_case(
        "audit_retained_grant_hash_mismatch",
        "rejected",
        "audit_retained_grant_hash_mismatch",
    ),
    load_case(
        "audit_manifest_hash_mismatch",
        "rejected",
        "audit_manifest_hash_mismatch",
    ),
    load_case(
        "audit_artifact_hash_mismatch",
        "rejected",
        "audit_artifact_hash_mismatch",
    ),
    load_case(
        "audit_vm_report_hash_mismatch",
        "rejected",
        "audit_vm_test_report_hash_mismatch",
    ),
    load_case(
        "audit_local_attestation_hash_mismatch",
        "rejected",
        "audit_local_attestation_hash_mismatch",
    ),
    load_case(
        "local_approval_mismatch",
        "rejected",
        "local_approval_missing_or_mismatch",
    ),
    load_case(
        "audit_rollback_plan_hash_mismatch",
        "rejected",
        "audit_rollback_plan_hash_mismatch",
    ),
    load_case(
        "rollback_artifact_hash_mismatch",
        "rejected",
        "rollback_artifact_hash_mismatch",
    ),
    load_case(
        "rollback_service_slot_mismatch",
        "rejected",
        "rollback_service_slot_mismatch",
    ),
];

pub const MODULE_LOAD_GATE_SERVICE_SLOT_CASES: &[(&str, &str, &str)] = &[
    load_case(
        "missing_retained_service_slot_reservation",
        "missing",
        "retained_service_slot_reservation_missing",
    ),
    load_case(
        "accepted_current_boot_reservation_still_denied",
        "retained_hash_reference_only_not_allocated",
        "retained_service_slot_reservation_not_allocated",
    ),
    load_case(
        "stale_dropped_retained_service_slot_reservation_event_id",
        "rejected",
        "retained_service_slot_reservation_stale_or_dropped_event_id",
    ),
    load_case(
        "retained_service_slot_wrong_schema_or_variant",
        "rejected",
        "retained_service_slot_reservation_wrong_schema_or_variant",
    ),
    load_case(
        "substituted_retained_service_slot_reservation",
        "rejected",
        "retained_service_slot_reservation_substituted_record",
    ),
    load_case(
        "retained_service_slot_grant_wrong_schema_or_variant",
        "rejected",
        "retained_service_slot_reservation_wrong_schema_or_variant",
    ),
    load_case(
        "retained_service_slot_audit_rollback_wrong_schema_or_variant",
        "rejected",
        "retained_service_slot_reservation_wrong_schema_or_variant",
    ),
    load_case(
        "retained_service_slot_computed_grant_hash_mismatch",
        "rejected",
        "retained_service_slot_reservation_computed_grant_hash_mismatch",
    ),
    load_case(
        "retained_service_slot_audit_record_hash_mismatch",
        "rejected",
        "retained_service_slot_reservation_audit_record_hash_mismatch",
    ),
    load_case(
        "retained_service_slot_rollback_plan_hash_mismatch",
        "rejected",
        "retained_service_slot_reservation_rollback_plan_hash_mismatch",
    ),
    load_case(
        "retained_service_slot_inventory_hash_mismatch",
        "rejected",
        "retained_service_slot_reservation_pre_load_inventory_hash_mismatch",
    ),
    load_case(
        "retained_service_slot_service_slot_mismatch",
        "rejected",
        "retained_service_slot_reservation_service_slot_mismatch",
    ),
    load_case(
        "retained_service_slot_reservation_hash_mismatch",
        "rejected",
        "retained_service_slot_reservation_hash_mismatch",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_types::{
        MODULE_AUDIT_TEST_CLEANUP_HASH, MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH,
        MODULE_AUDIT_TEST_PRE_INVENTORY_HASH, MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
        MODULE_GRANT_MISMATCH_MANIFEST_HASH, MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH, MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH, MODULE_LOAD_GATE_AUDIT_ROLLBACK_SELFTEST_CASES,
        MODULE_LOAD_GATE_RETAINED_SELFTEST_CASES, MODULE_LOAD_GATE_SERVICE_SLOT_SELFTEST_CASES,
    };

    #[derive(Clone, Copy)]
    struct ReferenceChain {
        manifest: ModuleManifestReference,
        retained: ModuleComputedGrantReference,
        artifact: ModuleCandidateArtifactReference,
        report: ModuleVmTestReportReference,
        attestation: ModuleLocalAttestationReference,
        approval: ModuleLocalApprovalReference,
    }

    fn reference_chain() -> ReferenceChain {
        let m = MODULE_GRANT_TEST_MANIFEST_HASH;
        let a = MODULE_GRANT_TEST_ARTIFACT_HASH;
        let v = MODULE_GRANT_TEST_VM_REPORT_HASH;
        let l = MODULE_GRANT_TEST_ATTESTATION_HASH;
        let p = MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH;
        let manifest = ModuleManifestReference {
            manifest_reference_hash: computed_module_manifest_reference_hash(m),
            manifest_hash: m,
        };
        let retained = ModuleComputedGrantReference {
            computed_grant_hash: computed_module_grant_hash(m, a, v, l),
            manifest_hash: m,
            artifact_hash: a,
            vm_report_hash: v,
            local_attestation_hash: l,
        };
        let artifact = ModuleCandidateArtifactReference {
            artifact_reference_hash:
                computed_module_candidate_artifact_reference_hash_from_sequences(
                    26,
                    27,
                    manifest.manifest_reference_hash,
                    m,
                    retained.computed_grant_hash,
                    a,
                    v,
                    l,
                ),
            retained_manifest_reference_event_id: 26,
            retained_reference_event_id: 27,
            manifest_reference_hash: manifest.manifest_reference_hash,
            manifest_hash: m,
            computed_grant_hash: retained.computed_grant_hash,
            artifact_hash: a,
            vm_report_hash: v,
            local_attestation_hash: l,
        };
        let report = ModuleVmTestReportReference {
            report_reference_hash: computed_module_vm_test_report_reference_hash_from_sequences(
                26,
                28,
                27,
                manifest.manifest_reference_hash,
                artifact.artifact_reference_hash,
                m,
                a,
                retained.computed_grant_hash,
                v,
                l,
            ),
            retained_manifest_reference_event_id: 26,
            retained_artifact_reference_event_id: 28,
            retained_reference_event_id: 27,
            manifest_reference_hash: manifest.manifest_reference_hash,
            artifact_reference_hash: artifact.artifact_reference_hash,
            manifest_hash: m,
            artifact_hash: a,
            computed_grant_hash: retained.computed_grant_hash,
            vm_report_hash: v,
            local_attestation_hash: l,
        };
        let attestation = ModuleLocalAttestationReference {
            attestation_reference_hash:
                computed_module_local_attestation_reference_hash_from_sequences(
                    26,
                    28,
                    29,
                    27,
                    manifest.manifest_reference_hash,
                    artifact.artifact_reference_hash,
                    report.report_reference_hash,
                    m,
                    a,
                    retained.computed_grant_hash,
                    v,
                    l,
                ),
            retained_manifest_reference_event_id: 26,
            retained_artifact_reference_event_id: 28,
            retained_vm_report_reference_event_id: 29,
            retained_reference_event_id: 27,
            manifest_reference_hash: manifest.manifest_reference_hash,
            artifact_reference_hash: artifact.artifact_reference_hash,
            vm_report_reference_hash: report.report_reference_hash,
            manifest_hash: m,
            artifact_hash: a,
            computed_grant_hash: retained.computed_grant_hash,
            vm_report_hash: v,
            local_attestation_hash: l,
            signature_verified: true,
        };
        let approval = ModuleLocalApprovalReference {
            approval_reference_hash: computed_module_local_approval_reference_hash_from_sequences(
                26,
                28,
                29,
                30,
                27,
                manifest.manifest_reference_hash,
                artifact.artifact_reference_hash,
                report.report_reference_hash,
                attestation.attestation_reference_hash,
                m,
                a,
                retained.computed_grant_hash,
                v,
                l,
                p,
            ),
            retained_manifest_reference_event_id: 26,
            retained_artifact_reference_event_id: 28,
            retained_vm_report_reference_event_id: 29,
            retained_local_attestation_reference_event_id: 30,
            retained_reference_event_id: 27,
            manifest_reference_hash: manifest.manifest_reference_hash,
            artifact_reference_hash: artifact.artifact_reference_hash,
            vm_report_reference_hash: report.report_reference_hash,
            local_attestation_reference_hash: attestation.attestation_reference_hash,
            manifest_hash: m,
            artifact_hash: a,
            computed_grant_hash: retained.computed_grant_hash,
            vm_report_hash: v,
            local_attestation_hash: l,
            local_approval_hash: p,
        };
        ReferenceChain {
            manifest,
            retained,
            artifact,
            report,
            attestation,
            approval,
        }
    }
    #[test]
    fn reference_case_table_preserves_names_statuses_and_reasons() {
        assert_eq!(MODULE_LOAD_GATE_REFERENCE_CASES.len(), 50);
        assert_eq!(
            MODULE_LOAD_GATE_REFERENCE_CASES[0],
            (
                "missing_retained_manifest_reference",
                "missing",
                "retained_module_manifest_reference_missing"
            )
        );
        assert_eq!(
            MODULE_LOAD_GATE_RETAINED_CASES.len(),
            MODULE_LOAD_GATE_RETAINED_SELFTEST_CASES
        );
        assert_eq!(
            MODULE_LOAD_GATE_AUDIT_ROLLBACK_CASES.len(),
            MODULE_LOAD_GATE_AUDIT_ROLLBACK_SELFTEST_CASES
        );
        assert_eq!(
            MODULE_LOAD_GATE_SERVICE_SLOT_CASES.len(),
            MODULE_LOAD_GATE_SERVICE_SLOT_SELFTEST_CASES
        );
        assert_eq!(
            MODULE_LOAD_GATE_AUDIT_ROLLBACK_CASES[13],
            (
                "valid_audit_and_rollback_still_denied",
                "validated_non_authorizing",
                "loader_and_service_slot_missing"
            )
        );
        assert_eq!(
            MODULE_LOAD_GATE_SERVICE_SLOT_CASES[12],
            (
                "retained_service_slot_reservation_hash_mismatch",
                "rejected",
                "retained_service_slot_reservation_hash_mismatch"
            )
        );
        assert_eq!(
            MODULE_LOAD_GATE_REFERENCE_CASES[49],
            (
                "computed_grant_reference_mismatch",
                "rejected",
                "retained_local_approval_reference_computed_grant_reference_mismatch"
            )
        );
    }
    #[test]
    fn manifest_valid_reference_is_non_authorizing() {
        let m = MODULE_GRANT_TEST_MANIFEST_HASH;
        let r = ModuleManifestReference {
            manifest_reference_hash: computed_module_manifest_reference_hash(m),
            manifest_hash: m,
        };
        let e = evaluate_manifest_reference(ModuleLoadGateManifestReferenceCandidate {
            scope: "current_boot",
            retained: true,
            schema_ok: true,
            event_reference: Some(r),
            candidate_reference: Some(r),
        });
        assert_eq!(e.status, "retained_hash_reference_only");
        assert!(!e.can_load && !e.load_attempted);
    }
    #[test]
    fn malformed_manifest_reference_fails_closed() {
        let r = ModuleManifestReference {
            manifest_reference_hash: [0; 32],
            manifest_hash: MODULE_GRANT_TEST_MANIFEST_HASH,
        };
        let e = evaluate_manifest_reference(ModuleLoadGateManifestReferenceCandidate {
            scope: "current_boot",
            retained: true,
            schema_ok: true,
            event_reference: Some(r),
            candidate_reference: Some(r),
        });
        assert_eq!(e.reason, "retained_module_manifest_reference_hash_mismatch");
    }

    #[test]
    fn event_hash_input_matches_kernel_current_boot_format() {
        for value in [0, 1, 26, 12_345_678, 99_999_999] {
            assert_eq!(
                current_boot_event_id(value).as_slice(),
                format!("event.current_boot.{value:08}").as_bytes()
            );
        }
        for value in [100_000_000, 123_456_789, u64::MAX] {
            assert_eq!(
                current_boot_event_id(value).as_slice(),
                format!("event.current_boot.{:08}", value % 100_000_000).as_bytes()
            );
        }
    }

    #[test]
    fn artifact_reference_case_table_matches_kernel_fixtures() {
        let manifest_hash = MODULE_GRANT_TEST_MANIFEST_HASH;
        let artifact_hash = [0x22; 32];
        let vm_report_hash = [0x33; 32];
        let local_attestation_hash = [0x44; 32];
        let manifest_reference = ModuleManifestReference {
            manifest_reference_hash: computed_module_manifest_reference_hash(manifest_hash),
            manifest_hash,
        };
        let retained_reference = ModuleComputedGrantReference {
            computed_grant_hash: computed_module_grant_hash(
                manifest_hash,
                artifact_hash,
                vm_report_hash,
                local_attestation_hash,
            ),
            manifest_hash,
            artifact_hash,
            vm_report_hash,
            local_attestation_hash,
        };
        let reference = ModuleCandidateArtifactReference {
            artifact_reference_hash:
                computed_module_candidate_artifact_reference_hash_from_sequences(
                    26,
                    27,
                    manifest_reference.manifest_reference_hash,
                    manifest_hash,
                    retained_reference.computed_grant_hash,
                    artifact_hash,
                    vm_report_hash,
                    local_attestation_hash,
                ),
            retained_manifest_reference_event_id: 26,
            retained_reference_event_id: 27,
            manifest_reference_hash: manifest_reference.manifest_reference_hash,
            manifest_hash,
            computed_grant_hash: retained_reference.computed_grant_hash,
            artifact_hash,
            vm_report_hash,
            local_attestation_hash,
        };

        let valid = ModuleLoadGateArtifactReferenceCandidate {
            scope: "current_boot",
            retained: true,
            schema_ok: true,
            event_reference: Some(reference),
            candidate_reference: Some(reference),
            manifest_event_id: Some(26),
            manifest_reference: Some(manifest_reference),
            retained_event_id: Some(27),
            retained_reference: Some(retained_reference),
        };

        for (index, &(name, expected_status, expected_reason)) in
            MODULE_LOAD_GATE_REFERENCE_CASES[7..16].iter().enumerate()
        {
            let mut candidate = valid;
            match index {
                0 => {
                    candidate.retained = false;
                    candidate.event_reference = None;
                    candidate.candidate_reference = None;
                }
                1 => {}
                2 => candidate.retained = false,
                3 => candidate.scope = "previous_boot",
                4 => candidate.schema_ok = false,
                5 => {
                    let substituted_retained = ModuleComputedGrantReference {
                        artifact_hash: [0xbb; 32],
                        ..retained_reference
                    };
                    candidate.candidate_reference = Some(ModuleCandidateArtifactReference {
                        artifact_reference_hash:
                            computed_module_candidate_artifact_reference_hash_from_sequences(
                                26,
                                27,
                                manifest_reference.manifest_reference_hash,
                                manifest_hash,
                                substituted_retained.computed_grant_hash,
                                substituted_retained.artifact_hash,
                                vm_report_hash,
                                local_attestation_hash,
                            ),
                        artifact_hash: substituted_retained.artifact_hash,
                        ..reference
                    });
                }
                6 => {
                    let mismatched = ModuleCandidateArtifactReference {
                        artifact_reference_hash: [0x99; 32],
                        ..reference
                    };
                    candidate.event_reference = Some(mismatched);
                    candidate.candidate_reference = Some(mismatched);
                }
                7 => candidate.manifest_event_id = Some(99),
                8 => candidate.retained_event_id = Some(98),
                _ => unreachable!(),
            }
            let result = evaluate_artifact_reference(candidate);
            assert_eq!(
                (result.status, result.reason),
                (expected_status, expected_reason),
                "{name}"
            );
        }
    }

    #[test]
    fn downstream_reference_case_tables_match_kernel_fixtures() {
        let chain = reference_chain();

        for (index, &(name, status, reason)) in
            MODULE_LOAD_GATE_REFERENCE_CASES[16..27].iter().enumerate()
        {
            let mut candidate = ModuleLoadGateVmReportReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(chain.report),
                candidate_reference: Some(chain.report),
                manifest_event_id: Some(26),
                manifest_reference: Some(chain.manifest),
                artifact_event_id: Some(28),
                artifact_reference: Some(chain.artifact),
                retained_event_id: Some(27),
                retained_reference: Some(chain.retained),
            };
            match index {
                0 => {
                    candidate.retained = false;
                    candidate.event_reference = None;
                    candidate.candidate_reference = None;
                }
                1 => {}
                2 => candidate.retained = false,
                3 => candidate.scope = "previous_boot",
                4 => candidate.schema_ok = false,
                5 => {
                    candidate.candidate_reference = Some(ModuleVmTestReportReference {
                        artifact_hash: [0xbb; 32],
                        ..chain.report
                    })
                }
                6 => {
                    let r = ModuleVmTestReportReference {
                        report_reference_hash: [0x99; 32],
                        ..chain.report
                    };
                    candidate.event_reference = Some(r);
                    candidate.candidate_reference = Some(r);
                }
                7 => candidate.manifest_event_id = Some(99),
                8 => candidate.artifact_event_id = Some(98),
                9 => candidate.retained_event_id = Some(97),
                10 => {
                    let r = ModuleVmTestReportReference {
                        report_reference_hash:
                            computed_module_vm_test_report_reference_hash_from_sequences(
                                26,
                                28,
                                27,
                                chain.manifest.manifest_reference_hash,
                                chain.artifact.artifact_reference_hash,
                                chain.report.manifest_hash,
                                chain.report.artifact_hash,
                                chain.report.computed_grant_hash,
                                [0xbb; 32],
                                chain.report.local_attestation_hash,
                            ),
                        vm_report_hash: [0xbb; 32],
                        ..chain.report
                    };
                    candidate.event_reference = Some(r);
                    candidate.candidate_reference = Some(r);
                }
                _ => unreachable!(),
            }
            let actual = evaluate_vm_report_reference(candidate);
            assert_eq!((actual.status, actual.reason), (status, reason), "{name}");
        }

        for (index, &(name, status, reason)) in
            MODULE_LOAD_GATE_REFERENCE_CASES[27..38].iter().enumerate()
        {
            let mut candidate = ModuleLoadGateLocalAttestationReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(chain.attestation),
                candidate_reference: Some(chain.attestation),
                manifest_event_id: Some(26),
                manifest_reference: Some(chain.manifest),
                artifact_event_id: Some(28),
                artifact_reference: Some(chain.artifact),
                vm_report_event_id: Some(29),
                vm_report_reference: Some(chain.report),
                retained_event_id: Some(27),
                retained_reference: Some(chain.retained),
            };
            match index {
                0 => {
                    candidate.retained = false;
                    candidate.event_reference = None;
                    candidate.candidate_reference = None;
                }
                1 => {}
                2 => candidate.retained = false,
                3 => candidate.scope = "previous_boot",
                4 => candidate.schema_ok = false,
                5 => {
                    candidate.candidate_reference = Some(ModuleLocalAttestationReference {
                        artifact_hash: [0xbb; 32],
                        ..chain.attestation
                    })
                }
                6 => {
                    let r = ModuleLocalAttestationReference {
                        attestation_reference_hash: [0x99; 32],
                        signature_verified: false,
                        ..chain.attestation
                    };
                    candidate.event_reference = Some(r);
                    candidate.candidate_reference = Some(r);
                }
                7 => candidate.manifest_event_id = Some(99),
                8 => candidate.artifact_event_id = Some(98),
                9 => candidate.vm_report_event_id = Some(97),
                10 => candidate.retained_event_id = Some(96),
                _ => unreachable!(),
            }
            let actual = evaluate_local_attestation_reference(candidate);
            assert_eq!((actual.status, actual.reason), (status, reason), "{name}");
        }

        for (index, &(name, status, reason)) in
            MODULE_LOAD_GATE_REFERENCE_CASES[38..50].iter().enumerate()
        {
            let mut candidate = ModuleLoadGateLocalApprovalReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(chain.approval),
                candidate_reference: Some(chain.approval),
                manifest_event_id: Some(26),
                manifest_reference: Some(chain.manifest),
                artifact_event_id: Some(28),
                artifact_reference: Some(chain.artifact),
                vm_report_event_id: Some(29),
                vm_report_reference: Some(chain.report),
                attestation_event_id: Some(30),
                attestation_reference: Some(chain.attestation),
                retained_event_id: Some(27),
                retained_reference: Some(chain.retained),
            };
            match index {
                0 => {
                    candidate.retained = false;
                    candidate.event_reference = None;
                    candidate.candidate_reference = None;
                }
                1 => {}
                2 => candidate.retained = false,
                3 => candidate.scope = "previous_boot",
                4 => candidate.schema_ok = false,
                5 => {
                    candidate.candidate_reference = Some(ModuleLocalApprovalReference {
                        local_attestation_reference_hash: [0xbc; 32],
                        ..chain.approval
                    })
                }
                6 => {
                    let r = ModuleLocalApprovalReference {
                        approval_reference_hash: [0x99; 32],
                        ..chain.approval
                    };
                    candidate.event_reference = Some(r);
                    candidate.candidate_reference = Some(r);
                }
                7 => candidate.manifest_event_id = Some(99),
                8 => candidate.artifact_event_id = Some(98),
                9 => candidate.vm_report_event_id = Some(97),
                10 => candidate.attestation_event_id = Some(96),
                11 => candidate.retained_event_id = Some(95),
                _ => unreachable!(),
            }
            let actual = evaluate_local_approval_reference(candidate);
            assert_eq!((actual.status, actual.reason), (status, reason), "{name}");
        }
    }

    fn audit_reference(
        slot: &'static str,
        grant_override: Option<[u8; 32]>,
        rollback_override: Option<[u8; 32]>,
        audit_override: Option<[u8; 32]>,
    ) -> ModuleAuditRollbackReference<'static> {
        let grant = computed_module_grant_hash(
            MODULE_GRANT_TEST_MANIFEST_HASH,
            MODULE_GRANT_TEST_ARTIFACT_HASH,
            MODULE_GRANT_TEST_VM_REPORT_HASH,
            MODULE_GRANT_TEST_ATTESTATION_HASH,
        );
        let rollback = computed_module_rollback_plan_hash(
            MODULE_GRANT_TEST_ARTIFACT_HASH,
            MODULE_AUDIT_TEST_PRE_INVENTORY_HASH,
            slot,
            MODULE_AUDIT_TEST_CLEANUP_HASH,
        );
        let mut reference = ModuleAuditRollbackReference {
            audit_record_hash: [0; 32],
            rollback_plan_hash: rollback_override.unwrap_or(rollback),
            computed_grant_hash: grant_override.unwrap_or(grant),
            manifest_hash: MODULE_GRANT_TEST_MANIFEST_HASH,
            artifact_hash: MODULE_GRANT_TEST_ARTIFACT_HASH,
            vm_report_hash: MODULE_GRANT_TEST_VM_REPORT_HASH,
            local_attestation_hash: MODULE_GRANT_TEST_ATTESTATION_HASH,
            local_approval_hash: MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH,
            pre_load_service_inventory_hash: MODULE_AUDIT_TEST_PRE_INVENTORY_HASH,
            cleanup_actions_hash: MODULE_AUDIT_TEST_CLEANUP_HASH,
            denial_event_id: 31,
            retained_reference_event_id: 27,
            ram_only_service_slot_id: slot,
        };
        reference.audit_record_hash = audit_override.unwrap_or_else(|| {
            let mut hash_input = reference;
            hash_input.computed_grant_hash = grant;
            hash_input.rollback_plan_hash = rollback;
            computed_module_audit_record_hash(hash_input)
        });
        reference
    }

    fn audit_candidate() -> ModuleLoadGateAuditRollbackCandidate<'static> {
        let reference =
            audit_reference(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID, None, None, None);
        ModuleLoadGateAuditRollbackCandidate {
            retained_reference: true,
            retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(reference),
                candidate_reference: Some(reference),
            },
            durable_audit_record: true,
            rollback_plan: true,
            audit_schema_ok: true,
            rollback_schema_ok: true,
            audit_binds_retained_grant: true,
            audit_binds_manifest: true,
            audit_binds_artifact: true,
            audit_binds_vm_report: true,
            audit_binds_local_attestation: true,
            audit_binds_local_approval: true,
            audit_binds_rollback_plan: true,
            rollback_binds_artifact: true,
            rollback_binds_service_slot: true,
            ram_only_service_slot_allocated: false,
            loader_available: false,
        }
    }

    fn slot_reservation(
        grant: Option<[u8; 32]>,
        audit_hash: Option<[u8; 32]>,
        rollback: Option<[u8; 32]>,
        inventory: Option<[u8; 32]>,
        slot: Option<&'static str>,
        reservation_hash: Option<[u8; 32]>,
    ) -> ModuleServiceSlotReservation<'static> {
        let audit = audit_reference(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID, None, None, None);
        let mut reservation = ModuleServiceSlotReservation {
            reservation_hash: [0; 32],
            retained_reference_event_id: 27,
            retained_audit_rollback_reference_event_id: 33,
            computed_grant_hash: grant.unwrap_or(audit.computed_grant_hash),
            audit_record_hash: audit_hash.unwrap_or(audit.audit_record_hash),
            rollback_plan_hash: rollback.unwrap_or(audit.rollback_plan_hash),
            pre_load_service_inventory_hash: inventory
                .unwrap_or(audit.pre_load_service_inventory_hash),
            ram_only_service_slot_id: slot.unwrap_or(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID),
        };
        reservation.reservation_hash = reservation_hash
            .unwrap_or_else(|| computed_module_service_slot_reservation_hash(reservation));
        reservation
    }

    #[test]
    fn retained_case_table_matches_kernel_fixtures() {
        let valid = reference_chain().retained;
        for (index, &(name, status, reason)) in MODULE_LOAD_GATE_RETAINED_CASES.iter().enumerate() {
            let mut candidate = ModuleLoadGateRetainedCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid),
                candidate_reference: Some(valid),
            };
            match index {
                0 => {
                    candidate.retained = false;
                    candidate.event_reference = None;
                    candidate.candidate_reference = None;
                }
                1 => {}
                2 => candidate.retained = false,
                3 => candidate.scope = "previous_boot",
                4 => candidate.schema_ok = false,
                5 => {
                    let manifest_hash = MODULE_GRANT_MISMATCH_MANIFEST_HASH;
                    candidate.candidate_reference = Some(ModuleComputedGrantReference {
                        computed_grant_hash: computed_module_grant_hash(
                            manifest_hash,
                            valid.artifact_hash,
                            valid.vm_report_hash,
                            valid.local_attestation_hash,
                        ),
                        manifest_hash,
                        ..valid
                    });
                }
                6 => {
                    let mismatch = ModuleComputedGrantReference {
                        computed_grant_hash: [0x66; 32],
                        ..valid
                    };
                    candidate.event_reference = Some(mismatch);
                    candidate.candidate_reference = Some(mismatch);
                }
                _ => unreachable!(),
            }
            let actual = evaluate_retained_candidate(candidate);
            assert_eq!((actual.status, actual.reason), (status, reason), "{name}");
        }
    }

    #[test]
    fn audit_rollback_case_table_matches_kernel_fixtures() {
        for (index, &(name, status, reason)) in
            MODULE_LOAD_GATE_AUDIT_ROLLBACK_CASES.iter().enumerate()
        {
            let mut candidate = audit_candidate();
            let reference = match index {
                4 => {
                    let mut r = audit_reference(
                        MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
                        None,
                        None,
                        None,
                    );
                    r.manifest_hash = MODULE_GRANT_MISMATCH_MANIFEST_HASH;
                    r.computed_grant_hash = computed_module_grant_hash(
                        r.manifest_hash,
                        r.artifact_hash,
                        r.vm_report_hash,
                        r.local_attestation_hash,
                    );
                    r.audit_record_hash = computed_module_audit_record_hash(r);
                    Some(r)
                }
                5 => Some(audit_reference(
                    MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
                    Some([0x99; 32]),
                    None,
                    None,
                )),
                6 => Some(audit_reference(
                    MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
                    None,
                    None,
                    Some([0xaa; 32]),
                )),
                7 => Some(audit_reference(
                    MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
                    None,
                    Some([0xbb; 32]),
                    None,
                )),
                8 => Some(audit_reference("ram_only:svc.test.0002", None, None, None)),
                _ => None,
            };
            match index {
                0 => {
                    candidate.retained_audit_rollback_reference.retained = false;
                    candidate.retained_audit_rollback_reference.event_reference = None;
                    candidate
                        .retained_audit_rollback_reference
                        .candidate_reference = None;
                }
                1 => candidate.retained_audit_rollback_reference.retained = false,
                2 => candidate.retained_audit_rollback_reference.scope = "previous_boot",
                3 => candidate.retained_audit_rollback_reference.schema_ok = false,
                4 => {
                    candidate
                        .retained_audit_rollback_reference
                        .candidate_reference = reference
                }
                5..=8 => {
                    candidate.retained_audit_rollback_reference.event_reference = reference;
                    candidate
                        .retained_audit_rollback_reference
                        .candidate_reference = reference;
                }
                9 => candidate.durable_audit_record = false,
                10 => candidate.rollback_plan = false,
                11 => candidate.audit_schema_ok = false,
                12 => candidate.rollback_schema_ok = false,
                13 => {}
                14 => candidate.audit_binds_retained_grant = false,
                15 => candidate.audit_binds_manifest = false,
                16 => candidate.audit_binds_artifact = false,
                17 => candidate.audit_binds_vm_report = false,
                18 => candidate.audit_binds_local_attestation = false,
                19 => candidate.audit_binds_local_approval = false,
                20 => candidate.audit_binds_rollback_plan = false,
                21 => candidate.rollback_binds_artifact = false,
                22 => candidate.rollback_binds_service_slot = false,
                _ => unreachable!(),
            }
            let actual = evaluate_audit_rollback_candidate(candidate);
            assert_eq!((actual.status, actual.reason), (status, reason), "{name}");
        }
    }

    #[test]
    fn service_slot_case_table_matches_kernel_fixtures() {
        let retained = reference_chain().retained;
        let audit = audit_reference(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID, None, None, None);
        let valid = slot_reservation(None, None, None, None, None, None);
        for (index, &(name, status, reason)) in
            MODULE_LOAD_GATE_SERVICE_SLOT_CASES.iter().enumerate()
        {
            let reservation = match index {
                4 => slot_reservation(Some([0x91; 32]), None, None, None, None, None),
                7 => slot_reservation(Some([0x92; 32]), None, None, None, None, None),
                8 => slot_reservation(None, Some([0x93; 32]), None, None, None, None),
                9 => slot_reservation(None, None, Some([0x94; 32]), None, None, None),
                10 => slot_reservation(None, None, None, Some([0x95; 32]), None, None),
                11 => slot_reservation(
                    None,
                    None,
                    None,
                    None,
                    Some("ram_only:svc.test.other"),
                    None,
                ),
                12 => slot_reservation(None, None, None, None, None, Some([0x96; 32])),
                _ => valid,
            };
            let mut candidate = ModuleLoadGateServiceSlotCandidate {
                retained_reference: Some(retained),
                audit_rollback_reference: Some(audit),
                audit_rollback_valid: true,
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    scope: "current_boot",
                    retained: true,
                    schema_ok: true,
                    grant_event_schema_ok: true,
                    audit_event_schema_ok: true,
                    grant_event_reference: Some(retained),
                    audit_event_reference: Some(audit),
                    event_reservation: Some(valid),
                    candidate_reservation: Some(valid),
                },
            };
            match index {
                0 => {
                    candidate.service_slot_reservation.event_reservation = None;
                    candidate.service_slot_reservation.candidate_reservation = None;
                }
                1 => {}
                2 => candidate.service_slot_reservation.retained = false,
                3 => candidate.service_slot_reservation.schema_ok = false,
                4 => candidate.service_slot_reservation.candidate_reservation = Some(reservation),
                5 => candidate.service_slot_reservation.grant_event_schema_ok = false,
                6 => candidate.service_slot_reservation.audit_event_schema_ok = false,
                7..=12 => {
                    candidate.service_slot_reservation.event_reservation = Some(reservation);
                    candidate.service_slot_reservation.candidate_reservation = Some(reservation);
                }
                _ => unreachable!(),
            }
            let actual = evaluate_service_slot_candidate(candidate);
            assert_eq!((actual.status, actual.reason), (status, reason), "{name}");
        }
    }
}
