use core::str;

use crate::event_log_types::{
    EventId, ModuleAuditRollbackReference, ModuleCandidateArtifactReference,
    ModuleComputedGrantReference, ModuleLocalApprovalReference, ModuleLocalAttestationReference,
    ModuleManifestReference, ModuleServiceSlotReservation, ModuleVmTestReportReference,
};
use crate::module_evidence;

pub(crate) fn module_computed_grant_reference_matches(
    left: ModuleComputedGrantReference,
    right: ModuleComputedGrantReference,
) -> bool {
    left.computed_grant_hash == right.computed_grant_hash
        && left.manifest_hash == right.manifest_hash
        && left.artifact_hash == right.artifact_hash
        && left.vm_report_hash == right.vm_report_hash
        && left.local_attestation_hash == right.local_attestation_hash
}

pub(crate) fn module_manifest_reference_matches(
    left: ModuleManifestReference,
    right: ModuleManifestReference,
) -> bool {
    left.manifest_reference_hash == right.manifest_reference_hash
        && left.manifest_hash == right.manifest_hash
}

pub(crate) fn module_manifest_reference_hashes_consistent(
    reference: ModuleManifestReference,
) -> bool {
    reference.manifest_reference_hash
        == module_evidence::computed_module_manifest_reference_hash(reference.manifest_hash)
}

pub(crate) fn module_candidate_artifact_reference_matches(
    left: ModuleCandidateArtifactReference,
    right: ModuleCandidateArtifactReference,
) -> bool {
    left.artifact_reference_hash == right.artifact_reference_hash
        && left.retained_manifest_reference_event_id == right.retained_manifest_reference_event_id
        && left.retained_reference_event_id == right.retained_reference_event_id
        && left.manifest_reference_hash == right.manifest_reference_hash
        && left.manifest_hash == right.manifest_hash
        && left.computed_grant_hash == right.computed_grant_hash
        && left.artifact_hash == right.artifact_hash
        && left.vm_report_hash == right.vm_report_hash
        && left.local_attestation_hash == right.local_attestation_hash
}

pub(crate) fn module_candidate_artifact_reference_hashes_consistent(
    reference: ModuleCandidateArtifactReference,
) -> bool {
    reference.artifact_reference_hash
        == module_evidence::computed_module_candidate_artifact_reference_hash_from_sequences(
            reference.retained_manifest_reference_event_id.sequence(),
            reference.retained_reference_event_id.sequence(),
            reference.manifest_reference_hash,
            reference.manifest_hash,
            reference.computed_grant_hash,
            reference.artifact_hash,
            reference.vm_report_hash,
            reference.local_attestation_hash,
        )
}

pub(crate) fn module_vm_test_report_reference_matches(
    left: ModuleVmTestReportReference,
    right: ModuleVmTestReportReference,
) -> bool {
    left.report_reference_hash == right.report_reference_hash
        && left.retained_manifest_reference_event_id == right.retained_manifest_reference_event_id
        && left.retained_artifact_reference_event_id == right.retained_artifact_reference_event_id
        && left.retained_reference_event_id == right.retained_reference_event_id
        && left.manifest_reference_hash == right.manifest_reference_hash
        && left.artifact_reference_hash == right.artifact_reference_hash
        && left.manifest_hash == right.manifest_hash
        && left.artifact_hash == right.artifact_hash
        && left.computed_grant_hash == right.computed_grant_hash
        && left.vm_report_hash == right.vm_report_hash
        && left.local_attestation_hash == right.local_attestation_hash
}

pub(crate) fn module_vm_test_report_reference_hashes_consistent(
    reference: ModuleVmTestReportReference,
) -> bool {
    reference.report_reference_hash
        == module_evidence::computed_module_vm_test_report_reference_hash_from_sequences(
            reference.retained_manifest_reference_event_id.sequence(),
            reference.retained_artifact_reference_event_id.sequence(),
            reference.retained_reference_event_id.sequence(),
            reference.manifest_reference_hash,
            reference.artifact_reference_hash,
            reference.manifest_hash,
            reference.artifact_hash,
            reference.computed_grant_hash,
            reference.vm_report_hash,
            reference.local_attestation_hash,
        )
}

pub(crate) fn module_local_attestation_reference_matches(
    left: ModuleLocalAttestationReference,
    right: ModuleLocalAttestationReference,
) -> bool {
    left.attestation_reference_hash == right.attestation_reference_hash
        && left.retained_manifest_reference_event_id == right.retained_manifest_reference_event_id
        && left.retained_artifact_reference_event_id == right.retained_artifact_reference_event_id
        && left.retained_vm_report_reference_event_id == right.retained_vm_report_reference_event_id
        && left.retained_reference_event_id == right.retained_reference_event_id
        && left.manifest_reference_hash == right.manifest_reference_hash
        && left.artifact_reference_hash == right.artifact_reference_hash
        && left.vm_report_reference_hash == right.vm_report_reference_hash
        && left.manifest_hash == right.manifest_hash
        && left.artifact_hash == right.artifact_hash
        && left.computed_grant_hash == right.computed_grant_hash
        && left.vm_report_hash == right.vm_report_hash
        && left.local_attestation_hash == right.local_attestation_hash
}

pub(crate) fn module_local_attestation_reference_hashes_consistent(
    reference: ModuleLocalAttestationReference,
) -> bool {
    reference.attestation_reference_hash
        == module_evidence::computed_module_local_attestation_reference_hash_from_sequences(
            reference.retained_manifest_reference_event_id.sequence(),
            reference.retained_artifact_reference_event_id.sequence(),
            reference.retained_vm_report_reference_event_id.sequence(),
            reference.retained_reference_event_id.sequence(),
            reference.manifest_reference_hash,
            reference.artifact_reference_hash,
            reference.vm_report_reference_hash,
            reference.manifest_hash,
            reference.artifact_hash,
            reference.computed_grant_hash,
            reference.vm_report_hash,
            reference.local_attestation_hash,
        )
}

pub(crate) fn module_local_approval_reference_matches(
    left: ModuleLocalApprovalReference,
    right: ModuleLocalApprovalReference,
) -> bool {
    left.approval_reference_hash == right.approval_reference_hash
        && left.retained_manifest_reference_event_id == right.retained_manifest_reference_event_id
        && left.retained_artifact_reference_event_id == right.retained_artifact_reference_event_id
        && left.retained_vm_report_reference_event_id == right.retained_vm_report_reference_event_id
        && left.retained_local_attestation_reference_event_id
            == right.retained_local_attestation_reference_event_id
        && left.retained_reference_event_id == right.retained_reference_event_id
        && left.manifest_reference_hash == right.manifest_reference_hash
        && left.artifact_reference_hash == right.artifact_reference_hash
        && left.vm_report_reference_hash == right.vm_report_reference_hash
        && left.local_attestation_reference_hash == right.local_attestation_reference_hash
        && left.manifest_hash == right.manifest_hash
        && left.artifact_hash == right.artifact_hash
        && left.computed_grant_hash == right.computed_grant_hash
        && left.vm_report_hash == right.vm_report_hash
        && left.local_attestation_hash == right.local_attestation_hash
        && left.local_approval_hash == right.local_approval_hash
}

pub(crate) fn module_local_approval_reference_hashes_consistent(
    reference: ModuleLocalApprovalReference,
) -> bool {
    reference.approval_reference_hash
        == module_evidence::computed_module_local_approval_reference_hash_from_sequences(
            reference.retained_manifest_reference_event_id.sequence(),
            reference.retained_artifact_reference_event_id.sequence(),
            reference.retained_vm_report_reference_event_id.sequence(),
            reference
                .retained_local_attestation_reference_event_id
                .sequence(),
            reference.retained_reference_event_id.sequence(),
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
}

pub(crate) fn module_computed_grant_reference_hashes_consistent(
    reference: ModuleComputedGrantReference,
) -> bool {
    reference.computed_grant_hash
        == module_evidence::computed_module_grant_hash(
            reference.manifest_hash,
            reference.artifact_hash,
            reference.vm_report_hash,
            reference.local_attestation_hash,
        )
}

pub(crate) fn module_audit_rollback_reference_matches(
    left: ModuleAuditRollbackReference,
    right: ModuleAuditRollbackReference,
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
        && left.ram_only_service_slot_id.as_str() == right.ram_only_service_slot_id.as_str()
}

pub(crate) fn module_audit_rollback_binds_computed_grant(
    audit_rollback_reference: ModuleAuditRollbackReference,
    retained_reference: ModuleComputedGrantReference,
) -> bool {
    audit_rollback_reference.computed_grant_hash == retained_reference.computed_grant_hash
        && audit_rollback_reference.manifest_hash == retained_reference.manifest_hash
        && audit_rollback_reference.artifact_hash == retained_reference.artifact_hash
        && audit_rollback_reference.vm_report_hash == retained_reference.vm_report_hash
        && audit_rollback_reference.local_attestation_hash
            == retained_reference.local_attestation_hash
}

pub(crate) fn module_audit_rollback_reference_hash_mismatch(
    reference: ModuleAuditRollbackReference,
) -> Option<&'static str> {
    let expected_rollback_plan_hash = module_evidence::computed_module_rollback_plan_hash(
        reference.artifact_hash,
        reference.pre_load_service_inventory_hash,
        reference.ram_only_service_slot_id.as_str(),
        reference.cleanup_actions_hash,
    );
    if reference.rollback_plan_hash != expected_rollback_plan_hash {
        return Some("retained_rollback_plan_hash_mismatch");
    }

    let mut denial_event_id = [0u8; EVENT_ID_TEXT_LEN];
    let mut retained_reference_event_id = [0u8; EVENT_ID_TEXT_LEN];
    let denial_event_id = event_id_text(reference.denial_event_id, &mut denial_event_id);
    let retained_reference_event_id = event_id_text(
        reference.retained_reference_event_id,
        &mut retained_reference_event_id,
    );
    let expected_audit_record_hash = module_evidence::computed_module_audit_record_hash(
        module_evidence::ModuleAuditRecordHashInput {
            denial_event_id,
            retained_reference_event_id,
            computed_grant_hash: reference.computed_grant_hash,
            manifest_hash: reference.manifest_hash,
            artifact_hash: reference.artifact_hash,
            vm_report_hash: reference.vm_report_hash,
            local_attestation_hash: reference.local_attestation_hash,
            local_approval_hash: reference.local_approval_hash,
            rollback_plan_hash: reference.rollback_plan_hash,
            ram_only_service_slot_id: reference.ram_only_service_slot_id.as_str(),
        },
    );
    if reference.audit_record_hash != expected_audit_record_hash {
        return Some("retained_audit_record_hash_mismatch");
    }

    None
}

pub(crate) fn module_service_slot_reservation_hash_mismatch(
    reservation: ModuleServiceSlotReservation,
) -> Option<&'static str> {
    let mut retained_reference_event_id = [0u8; EVENT_ID_TEXT_LEN];
    let mut retained_audit_rollback_reference_event_id = [0u8; EVENT_ID_TEXT_LEN];
    let retained_reference_event_id = event_id_text(
        reservation.retained_reference_event_id,
        &mut retained_reference_event_id,
    );
    let retained_audit_rollback_reference_event_id = event_id_text(
        reservation.retained_audit_rollback_reference_event_id,
        &mut retained_audit_rollback_reference_event_id,
    );
    let expected_reservation_hash = module_evidence::computed_module_service_slot_reservation_hash(
        module_evidence::ModuleServiceSlotReservationHashInput {
            retained_reference_event_id,
            retained_audit_rollback_reference_event_id,
            computed_grant_hash: reservation.computed_grant_hash,
            audit_record_hash: reservation.audit_record_hash,
            rollback_plan_hash: reservation.rollback_plan_hash,
            pre_load_service_inventory_hash: reservation.pre_load_service_inventory_hash,
            ram_only_service_slot_id: reservation.ram_only_service_slot_id.as_str(),
        },
    );
    if reservation.reservation_hash != expected_reservation_hash {
        return Some("retained_service_slot_reservation_hash_mismatch");
    }

    None
}

const EVENT_ID_TEXT_LEN: usize = 27;

fn event_id_text<'a>(event_id: EventId, out: &'a mut [u8; EVENT_ID_TEXT_LEN]) -> &'a str {
    const PREFIX: &[u8] = b"event.current_boot.";
    out[..PREFIX.len()].copy_from_slice(PREFIX);
    let mut value = event_id.sequence();
    let mut idx = EVENT_ID_TEXT_LEN;
    while idx > PREFIX.len() {
        idx -= 1;
        out[idx] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    unsafe { str::from_utf8_unchecked(out) }
}
