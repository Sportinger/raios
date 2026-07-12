use crate::{
    agent_protocol_module_load_gate::{
        ModuleLoadGateArtifactEvaluation, ModuleLoadGateAuditRollbackEvaluation,
        ModuleLoadGateLocalApprovalEvaluation, ModuleLoadGateLocalAttestationEvaluation,
        ModuleLoadGateManifestEvaluation, ModuleLoadGateRetainedCheck,
        ModuleLoadGateServiceSlotEvaluation, ModuleLoadGateVmReportEvaluation,
    },
    agent_protocol_module_types::*,
    agent_protocol_support::{method_eq, parse_current_boot_event_id},
    event_log,
    module_evidence::{ModuleAuditRecordHashInput, ModuleServiceSlotReservationHashInput},
};

use raios_core::module_load_gate as core_load_gate;

fn core_manifest_reference(
    reference: event_log::ModuleManifestReference,
) -> core_load_gate::ModuleManifestReference {
    core_load_gate::ModuleManifestReference {
        manifest_reference_hash: reference.manifest_reference_hash,
        manifest_hash: reference.manifest_hash,
    }
}

fn core_artifact_reference(
    reference: event_log::ModuleCandidateArtifactReference,
) -> core_load_gate::ModuleCandidateArtifactReference {
    core_load_gate::ModuleCandidateArtifactReference {
        artifact_reference_hash: reference.artifact_reference_hash,
        retained_manifest_reference_event_id: reference
            .retained_manifest_reference_event_id
            .sequence(),
        retained_reference_event_id: reference.retained_reference_event_id.sequence(),
        manifest_reference_hash: reference.manifest_reference_hash,
        manifest_hash: reference.manifest_hash,
        computed_grant_hash: reference.computed_grant_hash,
        artifact_hash: reference.artifact_hash,
        vm_report_hash: reference.vm_report_hash,
        local_attestation_hash: reference.local_attestation_hash,
    }
}

fn core_vm_report_reference(
    reference: event_log::ModuleVmTestReportReference,
) -> core_load_gate::ModuleVmTestReportReference {
    core_load_gate::ModuleVmTestReportReference {
        report_reference_hash: reference.report_reference_hash,
        retained_manifest_reference_event_id: reference
            .retained_manifest_reference_event_id
            .sequence(),
        retained_artifact_reference_event_id: reference
            .retained_artifact_reference_event_id
            .sequence(),
        retained_reference_event_id: reference.retained_reference_event_id.sequence(),
        manifest_reference_hash: reference.manifest_reference_hash,
        artifact_reference_hash: reference.artifact_reference_hash,
        manifest_hash: reference.manifest_hash,
        artifact_hash: reference.artifact_hash,
        computed_grant_hash: reference.computed_grant_hash,
        vm_report_hash: reference.vm_report_hash,
        local_attestation_hash: reference.local_attestation_hash,
    }
}

fn core_attestation_reference(
    reference: event_log::ModuleLocalAttestationReference,
) -> core_load_gate::ModuleLocalAttestationReference {
    core_load_gate::ModuleLocalAttestationReference {
        attestation_reference_hash: reference.attestation_reference_hash,
        retained_manifest_reference_event_id: reference
            .retained_manifest_reference_event_id
            .sequence(),
        retained_artifact_reference_event_id: reference
            .retained_artifact_reference_event_id
            .sequence(),
        retained_vm_report_reference_event_id: reference
            .retained_vm_report_reference_event_id
            .sequence(),
        retained_reference_event_id: reference.retained_reference_event_id.sequence(),
        manifest_reference_hash: reference.manifest_reference_hash,
        artifact_reference_hash: reference.artifact_reference_hash,
        vm_report_reference_hash: reference.vm_report_reference_hash,
        manifest_hash: reference.manifest_hash,
        artifact_hash: reference.artifact_hash,
        computed_grant_hash: reference.computed_grant_hash,
        vm_report_hash: reference.vm_report_hash,
        local_attestation_hash: reference.local_attestation_hash,
        signature_verified: reference.signature_verified,
    }
}

fn core_approval_reference(
    reference: event_log::ModuleLocalApprovalReference,
) -> core_load_gate::ModuleLocalApprovalReference {
    core_load_gate::ModuleLocalApprovalReference {
        approval_reference_hash: reference.approval_reference_hash,
        retained_manifest_reference_event_id: reference
            .retained_manifest_reference_event_id
            .sequence(),
        retained_artifact_reference_event_id: reference
            .retained_artifact_reference_event_id
            .sequence(),
        retained_vm_report_reference_event_id: reference
            .retained_vm_report_reference_event_id
            .sequence(),
        retained_local_attestation_reference_event_id: reference
            .retained_local_attestation_reference_event_id
            .sequence(),
        retained_reference_event_id: reference.retained_reference_event_id.sequence(),
        manifest_reference_hash: reference.manifest_reference_hash,
        artifact_reference_hash: reference.artifact_reference_hash,
        vm_report_reference_hash: reference.vm_report_reference_hash,
        local_attestation_reference_hash: reference.local_attestation_reference_hash,
        manifest_hash: reference.manifest_hash,
        artifact_hash: reference.artifact_hash,
        computed_grant_hash: reference.computed_grant_hash,
        vm_report_hash: reference.vm_report_hash,
        local_attestation_hash: reference.local_attestation_hash,
        local_approval_hash: reference.local_approval_hash,
    }
}

fn core_grant_reference(
    reference: event_log::ModuleComputedGrantReference,
) -> core_load_gate::ModuleComputedGrantReference {
    core_load_gate::ModuleComputedGrantReference {
        computed_grant_hash: reference.computed_grant_hash,
        manifest_hash: reference.manifest_hash,
        artifact_hash: reference.artifact_hash,
        vm_report_hash: reference.vm_report_hash,
        local_attestation_hash: reference.local_attestation_hash,
    }
}

fn core_manifest_candidate(
    candidate: ModuleLoadGateManifestReferenceCandidate,
) -> core_load_gate::ModuleLoadGateManifestReferenceCandidate {
    core_load_gate::ModuleLoadGateManifestReferenceCandidate {
        scope: candidate.scope,
        retained: candidate.retained,
        schema_ok: candidate.schema_ok,
        event_reference: candidate.event_reference.map(core_manifest_reference),
        candidate_reference: candidate.candidate_reference.map(core_manifest_reference),
    }
}

fn core_artifact_candidate(
    candidate: ModuleLoadGateArtifactReferenceCandidate,
) -> core_load_gate::ModuleLoadGateArtifactReferenceCandidate {
    core_load_gate::ModuleLoadGateArtifactReferenceCandidate {
        scope: candidate.scope,
        retained: candidate.retained,
        schema_ok: candidate.schema_ok,
        event_reference: candidate.event_reference.map(core_artifact_reference),
        candidate_reference: candidate.candidate_reference.map(core_artifact_reference),
        manifest_event_id: candidate
            .manifest_event_id
            .map(|event_id| event_id.sequence()),
        manifest_reference: candidate.manifest_reference.map(core_manifest_reference),
        retained_event_id: candidate
            .retained_event_id
            .map(|event_id| event_id.sequence()),
        retained_reference: candidate.retained_reference.map(core_grant_reference),
    }
}

fn core_vm_report_candidate(
    candidate: ModuleLoadGateVmReportReferenceCandidate,
) -> core_load_gate::ModuleLoadGateVmReportReferenceCandidate {
    core_load_gate::ModuleLoadGateVmReportReferenceCandidate {
        scope: candidate.scope,
        retained: candidate.retained,
        schema_ok: candidate.schema_ok,
        event_reference: candidate.event_reference.map(core_vm_report_reference),
        candidate_reference: candidate.candidate_reference.map(core_vm_report_reference),
        manifest_event_id: candidate
            .manifest_event_id
            .map(|event_id| event_id.sequence()),
        manifest_reference: candidate.manifest_reference.map(core_manifest_reference),
        artifact_event_id: candidate
            .artifact_event_id
            .map(|event_id| event_id.sequence()),
        artifact_reference: candidate.artifact_reference.map(core_artifact_reference),
        retained_event_id: candidate
            .retained_event_id
            .map(|event_id| event_id.sequence()),
        retained_reference: candidate.retained_reference.map(core_grant_reference),
    }
}

fn core_attestation_candidate(
    candidate: ModuleLoadGateLocalAttestationReferenceCandidate,
) -> core_load_gate::ModuleLoadGateLocalAttestationReferenceCandidate {
    core_load_gate::ModuleLoadGateLocalAttestationReferenceCandidate {
        scope: candidate.scope,
        retained: candidate.retained,
        schema_ok: candidate.schema_ok,
        event_reference: candidate.event_reference.map(core_attestation_reference),
        candidate_reference: candidate
            .candidate_reference
            .map(core_attestation_reference),
        manifest_event_id: candidate
            .manifest_event_id
            .map(|event_id| event_id.sequence()),
        manifest_reference: candidate.manifest_reference.map(core_manifest_reference),
        artifact_event_id: candidate
            .artifact_event_id
            .map(|event_id| event_id.sequence()),
        artifact_reference: candidate.artifact_reference.map(core_artifact_reference),
        vm_report_event_id: candidate
            .vm_report_event_id
            .map(|event_id| event_id.sequence()),
        vm_report_reference: candidate.vm_report_reference.map(core_vm_report_reference),
        retained_event_id: candidate
            .retained_event_id
            .map(|event_id| event_id.sequence()),
        retained_reference: candidate.retained_reference.map(core_grant_reference),
    }
}

fn core_approval_candidate(
    candidate: ModuleLoadGateLocalApprovalReferenceCandidate,
) -> core_load_gate::ModuleLoadGateLocalApprovalReferenceCandidate {
    core_load_gate::ModuleLoadGateLocalApprovalReferenceCandidate {
        scope: candidate.scope,
        retained: candidate.retained,
        schema_ok: candidate.schema_ok,
        event_reference: candidate.event_reference.map(core_approval_reference),
        candidate_reference: candidate.candidate_reference.map(core_approval_reference),
        manifest_event_id: candidate
            .manifest_event_id
            .map(|event_id| event_id.sequence()),
        manifest_reference: candidate.manifest_reference.map(core_manifest_reference),
        artifact_event_id: candidate
            .artifact_event_id
            .map(|event_id| event_id.sequence()),
        artifact_reference: candidate.artifact_reference.map(core_artifact_reference),
        vm_report_event_id: candidate
            .vm_report_event_id
            .map(|event_id| event_id.sequence()),
        vm_report_reference: candidate.vm_report_reference.map(core_vm_report_reference),
        attestation_event_id: candidate
            .attestation_event_id
            .map(|event_id| event_id.sequence()),
        attestation_reference: candidate
            .attestation_reference
            .map(core_attestation_reference),
        retained_event_id: candidate
            .retained_event_id
            .map(|event_id| event_id.sequence()),
        retained_reference: candidate.retained_reference.map(core_grant_reference),
    }
}

fn core_retained_candidate(
    candidate: ModuleLoadGateRetainedCandidate,
) -> core_load_gate::ModuleLoadGateRetainedCandidate {
    core_load_gate::ModuleLoadGateRetainedCandidate {
        scope: candidate.scope,
        retained: candidate.retained,
        schema_ok: candidate.schema_ok,
        event_reference: candidate.event_reference.map(core_grant_reference),
        candidate_reference: candidate.candidate_reference.map(core_grant_reference),
    }
}

fn core_audit_reference<'a>(
    reference: &'a event_log::ModuleAuditRollbackReference,
) -> core_load_gate::ModuleAuditRollbackReference<'a> {
    core_load_gate::ModuleAuditRollbackReference {
        audit_record_hash: reference.audit_record_hash,
        rollback_plan_hash: reference.rollback_plan_hash,
        computed_grant_hash: reference.computed_grant_hash,
        manifest_hash: reference.manifest_hash,
        artifact_hash: reference.artifact_hash,
        vm_report_hash: reference.vm_report_hash,
        local_attestation_hash: reference.local_attestation_hash,
        local_approval_hash: reference.local_approval_hash,
        pre_load_service_inventory_hash: reference.pre_load_service_inventory_hash,
        cleanup_actions_hash: reference.cleanup_actions_hash,
        denial_event_id: reference.denial_event_id.sequence(),
        retained_reference_event_id: reference.retained_reference_event_id.sequence(),
        ram_only_service_slot_id: reference.ram_only_service_slot_id.as_str(),
    }
}

fn core_audit_reference_candidate<'a>(
    candidate: &'a ModuleLoadGateAuditRollbackReferenceCandidate,
) -> core_load_gate::ModuleLoadGateAuditRollbackReferenceCandidate<'a> {
    core_load_gate::ModuleLoadGateAuditRollbackReferenceCandidate {
        scope: candidate.scope,
        retained: candidate.retained,
        schema_ok: candidate.schema_ok,
        event_reference: candidate.event_reference.as_ref().map(core_audit_reference),
        candidate_reference: candidate
            .candidate_reference
            .as_ref()
            .map(core_audit_reference),
    }
}

fn core_audit_candidate<'a>(
    candidate: &'a ModuleLoadGateAuditRollbackCandidate,
) -> core_load_gate::ModuleLoadGateAuditRollbackCandidate<'a> {
    core_load_gate::ModuleLoadGateAuditRollbackCandidate {
        retained_reference: candidate.retained_reference,
        retained_audit_rollback_reference: core_audit_reference_candidate(
            &candidate.retained_audit_rollback_reference,
        ),
        durable_audit_record: candidate.durable_audit_record,
        rollback_plan: candidate.rollback_plan,
        audit_schema_ok: candidate.audit_schema_ok,
        rollback_schema_ok: candidate.rollback_schema_ok,
        audit_binds_retained_grant: candidate.audit_binds_retained_grant,
        audit_binds_manifest: candidate.audit_binds_manifest,
        audit_binds_artifact: candidate.audit_binds_artifact,
        audit_binds_vm_report: candidate.audit_binds_vm_report,
        audit_binds_local_attestation: candidate.audit_binds_local_attestation,
        audit_binds_local_approval: candidate.audit_binds_local_approval,
        audit_binds_rollback_plan: candidate.audit_binds_rollback_plan,
        rollback_binds_artifact: candidate.rollback_binds_artifact,
        rollback_binds_service_slot: candidate.rollback_binds_service_slot,
        ram_only_service_slot_allocated: candidate.ram_only_service_slot_allocated,
        loader_available: candidate.loader_available,
    }
}

fn core_slot_reservation<'a>(
    reservation: &'a event_log::ModuleServiceSlotReservation,
) -> core_load_gate::ModuleServiceSlotReservation<'a> {
    core_load_gate::ModuleServiceSlotReservation {
        reservation_hash: reservation.reservation_hash,
        retained_reference_event_id: reservation.retained_reference_event_id.sequence(),
        retained_audit_rollback_reference_event_id: reservation
            .retained_audit_rollback_reference_event_id
            .sequence(),
        computed_grant_hash: reservation.computed_grant_hash,
        audit_record_hash: reservation.audit_record_hash,
        rollback_plan_hash: reservation.rollback_plan_hash,
        pre_load_service_inventory_hash: reservation.pre_load_service_inventory_hash,
        ram_only_service_slot_id: reservation.ram_only_service_slot_id.as_str(),
    }
}

fn core_slot_reservation_candidate<'a>(
    candidate: &'a ModuleLoadGateServiceSlotReservationCandidate,
) -> core_load_gate::ModuleLoadGateServiceSlotReservationCandidate<'a> {
    core_load_gate::ModuleLoadGateServiceSlotReservationCandidate {
        scope: candidate.scope,
        retained: candidate.retained,
        schema_ok: candidate.schema_ok,
        grant_event_schema_ok: candidate.grant_event_schema_ok,
        audit_event_schema_ok: candidate.audit_event_schema_ok,
        grant_event_reference: candidate.grant_event_reference.map(core_grant_reference),
        audit_event_reference: candidate
            .audit_event_reference
            .as_ref()
            .map(core_audit_reference),
        event_reservation: candidate
            .event_reservation
            .as_ref()
            .map(core_slot_reservation),
        candidate_reservation: candidate
            .candidate_reservation
            .as_ref()
            .map(core_slot_reservation),
    }
}

fn core_slot_candidate<'a>(
    candidate: &'a ModuleLoadGateServiceSlotCandidate,
) -> core_load_gate::ModuleLoadGateServiceSlotCandidate<'a> {
    core_load_gate::ModuleLoadGateServiceSlotCandidate {
        retained_reference: candidate.retained_reference.map(core_grant_reference),
        audit_rollback_reference: candidate
            .audit_rollback_reference
            .as_ref()
            .map(core_audit_reference),
        audit_rollback_valid: candidate.audit_rollback_valid,
        service_slot_reservation: core_slot_reservation_candidate(
            &candidate.service_slot_reservation,
        ),
    }
}

pub(crate) fn evaluate_module_load_gate_manifest_candidate(
    candidate: ModuleLoadGateManifestReferenceCandidate,
) -> ModuleLoadGateManifestEvaluation {
    let result = core_load_gate::evaluate_manifest_reference(core_manifest_candidate(candidate));
    ModuleLoadGateManifestEvaluation {
        status: result.status,
        reason: result.reason,
        module_manifest_state: result.module_manifest_state,
        accepted_manifest_hash: result.accepted_manifest_hash,
        can_load: result.can_load,
        load_attempted: result.load_attempted,
    }
}

pub(crate) fn evaluate_module_load_gate_artifact_candidate(
    candidate: ModuleLoadGateArtifactReferenceCandidate,
) -> ModuleLoadGateArtifactEvaluation {
    let result = core_load_gate::evaluate_artifact_reference(core_artifact_candidate(candidate));
    ModuleLoadGateArtifactEvaluation {
        status: result.status,
        reason: result.reason,
        candidate_artifact_state: result.candidate_artifact_state,
        accepted_artifact_hash: result.accepted_artifact_hash,
        can_load: result.can_load,
        load_attempted: result.load_attempted,
    }
}

pub(crate) fn evaluate_module_load_gate_vm_report_candidate(
    candidate: ModuleLoadGateVmReportReferenceCandidate,
) -> ModuleLoadGateVmReportEvaluation {
    let result = core_load_gate::evaluate_vm_report_reference(core_vm_report_candidate(candidate));
    ModuleLoadGateVmReportEvaluation {
        status: result.status,
        reason: result.reason,
        vm_test_report_state: result.vm_test_report_state,
        accepted_vm_report_hash: result.accepted_vm_report_hash,
        can_load: result.can_load,
        load_attempted: result.load_attempted,
    }
}

pub(crate) fn evaluate_module_load_gate_local_attestation_candidate(
    candidate: ModuleLoadGateLocalAttestationReferenceCandidate,
) -> ModuleLoadGateLocalAttestationEvaluation {
    let result =
        core_load_gate::evaluate_local_attestation_reference(core_attestation_candidate(candidate));
    ModuleLoadGateLocalAttestationEvaluation {
        status: result.status,
        reason: result.reason,
        local_attestation_state: result.local_attestation_state,
        accepted_local_attestation_hash: result.accepted_local_attestation_hash,
        can_load: result.can_load,
        load_attempted: result.load_attempted,
    }
}

pub(crate) fn evaluate_module_load_gate_local_approval_candidate(
    candidate: ModuleLoadGateLocalApprovalReferenceCandidate,
) -> ModuleLoadGateLocalApprovalEvaluation {
    let result =
        core_load_gate::evaluate_local_approval_reference(core_approval_candidate(candidate));
    ModuleLoadGateLocalApprovalEvaluation {
        status: result.status,
        reason: result.reason,
        local_approval_state: result.local_approval_state,
        accepted_local_approval_hash: result.accepted_local_approval_hash,
        can_load: result.can_load,
        load_attempted: result.load_attempted,
    }
}

pub(crate) fn evaluate_module_load_gate_retained_candidate(
    candidate: ModuleLoadGateRetainedCandidate,
) -> ModuleLoadGateRetainedCheck {
    let result = core_load_gate::evaluate_retained_candidate(core_retained_candidate(candidate));
    ModuleLoadGateRetainedCheck {
        status: result.status,
        reason: result.reason,
        can_load: result.can_load,
        load_attempted: result.load_attempted,
    }
}

pub(crate) fn evaluate_module_load_gate_audit_rollback_candidate(
    candidate: ModuleLoadGateAuditRollbackCandidate,
) -> ModuleLoadGateAuditRollbackEvaluation {
    let result =
        core_load_gate::evaluate_audit_rollback_candidate(core_audit_candidate(&candidate));
    ModuleLoadGateAuditRollbackEvaluation {
        status: result.status,
        reason: result.reason,
        can_load: result.can_load,
        load_attempted: result.load_attempted,
    }
}

pub(crate) fn evaluate_module_load_gate_service_slot_candidate(
    candidate: ModuleLoadGateServiceSlotCandidate,
) -> ModuleLoadGateServiceSlotEvaluation {
    let result = core_load_gate::evaluate_service_slot_candidate(core_slot_candidate(&candidate));
    ModuleLoadGateServiceSlotEvaluation {
        status: result.status,
        reason: result.reason,
        service_slot_state: result.service_slot_state,
        accepted_service_slot_reservation_hash: result.accepted_service_slot_reservation_hash,
        can_load: result.can_load,
        load_attempted: result.load_attempted,
    }
}

pub(crate) fn computed_module_manifest_reference_hash(manifest_hash: [u8; 32]) -> [u8; 32] {
    core_load_gate::computed_module_manifest_reference_hash(manifest_hash)
}

pub(crate) fn computed_module_grant_hash(
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> [u8; 32] {
    core_load_gate::computed_module_grant_hash(
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    )
}

pub(crate) fn computed_module_rollback_plan_hash(
    artifact_hash: [u8; 32],
    pre_load_service_inventory_hash: [u8; 32],
    ram_only_service_slot_id: &str,
    cleanup_actions_hash: [u8; 32],
) -> [u8; 32] {
    core_load_gate::computed_module_rollback_plan_hash(
        artifact_hash,
        pre_load_service_inventory_hash,
        ram_only_service_slot_id,
        cleanup_actions_hash,
    )
}

pub(crate) fn computed_module_audit_record_hash(input: ModuleAuditRecordHashInput<'_>) -> [u8; 32] {
    let denial_event_id = parse_current_boot_event_id(input.denial_event_id)
        .map(|event_id| event_id.sequence())
        .unwrap_or(0);
    let retained_reference_event_id =
        parse_current_boot_event_id(input.retained_reference_event_id)
            .map(|event_id| event_id.sequence())
            .unwrap_or(0);
    core_load_gate::computed_module_audit_record_hash(
        core_load_gate::ModuleAuditRollbackReference {
            audit_record_hash: [0; 32],
            rollback_plan_hash: input.rollback_plan_hash,
            computed_grant_hash: input.computed_grant_hash,
            manifest_hash: input.manifest_hash,
            artifact_hash: input.artifact_hash,
            vm_report_hash: input.vm_report_hash,
            local_attestation_hash: input.local_attestation_hash,
            local_approval_hash: input.local_approval_hash,
            pre_load_service_inventory_hash: [0; 32],
            cleanup_actions_hash: [0; 32],
            denial_event_id,
            retained_reference_event_id,
            ram_only_service_slot_id: input.ram_only_service_slot_id,
        },
    )
}

pub(crate) fn computed_module_service_slot_reservation_hash(
    input: ModuleServiceSlotReservationHashInput<'_>,
) -> [u8; 32] {
    let retained_reference_event_id =
        parse_current_boot_event_id(input.retained_reference_event_id)
            .map(|event_id| event_id.sequence())
            .unwrap_or(0);
    let retained_audit_rollback_reference_event_id =
        parse_current_boot_event_id(input.retained_audit_rollback_reference_event_id)
            .map(|event_id| event_id.sequence())
            .unwrap_or(0);
    core_load_gate::computed_module_service_slot_reservation_hash(
        core_load_gate::ModuleServiceSlotReservation {
            reservation_hash: [0; 32],
            retained_reference_event_id,
            retained_audit_rollback_reference_event_id,
            computed_grant_hash: input.computed_grant_hash,
            audit_record_hash: input.audit_record_hash,
            rollback_plan_hash: input.rollback_plan_hash,
            pre_load_service_inventory_hash: input.pre_load_service_inventory_hash,
            ram_only_service_slot_id: input.ram_only_service_slot_id,
        },
    )
}

pub(crate) fn evaluate_module_load_gate_loader_runtime_candidate(
    candidate: ModuleLoadGateLoaderRuntimeCandidate,
) -> ModuleLoadGateLoaderRuntimeEvaluation {
    let retained_module_evidence_complete =
        module_load_gate_loader_runtime_retained_evidence_complete(candidate);
    let retained_module_evidence_state = if retained_module_evidence_complete {
        "available"
    } else if module_load_gate_loader_runtime_retained_evidence_rejected(candidate) {
        "rejected"
    } else {
        "missing"
    };
    let retained_module_evidence_reason =
        module_load_gate_loader_runtime_retained_evidence_reason(candidate);
    let service_slot_allocator_state = if module_load_gate_loader_runtime_reference_available(
        candidate.service_slot_reservation_state,
    ) {
        "defined_non_authorizing"
    } else if module_load_gate_loader_runtime_reference_rejected(
        candidate.service_slot_reservation_state,
    ) {
        "blocked_by_rejected_service_slot_reservation"
    } else {
        "blocked_by_service_slot_reservation"
    };
    let service_slot_allocator_status = if module_load_gate_loader_runtime_reference_available(
        candidate.service_slot_reservation_state,
    ) {
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS
    } else {
        "blocked"
    };
    let service_slot_allocator_reason = if module_load_gate_loader_runtime_reference_available(
        candidate.service_slot_reservation_state,
    ) {
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON
    } else if module_load_gate_loader_runtime_reference_rejected(
        candidate.service_slot_reservation_state,
    ) {
        candidate.service_slot_reservation_reason
    } else {
        "retained_service_slot_reservation_missing"
    };
    let (loader_runtime_state, status, reason) = if retained_module_evidence_complete {
        (
            "blocked_by_service_slot_allocator_authority",
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
        )
    } else {
        (
            "blocked_by_retained_module_evidence",
            "denied_missing_retained_module_evidence",
            retained_module_evidence_reason,
        )
    };

    ModuleLoadGateLoaderRuntimeEvaluation {
        status,
        reason,
        retained_module_evidence_state,
        retained_module_evidence_reason,
        service_slot_allocator_state,
        service_slot_allocator_status,
        service_slot_allocator_reason,
        loader_runtime_state,
        can_load: false,
        load_attempted: false,
    }
}

fn module_load_gate_loader_runtime_retained_evidence_complete(
    candidate: ModuleLoadGateLoaderRuntimeCandidate,
) -> bool {
    module_load_gate_loader_runtime_reference_available(candidate.manifest_reference_state)
        && module_load_gate_loader_runtime_reference_available(candidate.artifact_reference_state)
        && module_load_gate_loader_runtime_reference_available(candidate.vm_report_reference_state)
        && module_load_gate_loader_runtime_reference_available(
            candidate.local_attestation_reference_state,
        )
        && module_load_gate_loader_runtime_reference_available(
            candidate.local_approval_reference_state,
        )
        && module_load_gate_loader_runtime_reference_available(
            candidate.computed_grant_reference_state,
        )
        && module_load_gate_loader_runtime_reference_available(
            candidate.audit_rollback_reference_state,
        )
        && module_load_gate_loader_runtime_reference_available(
            candidate.service_slot_reservation_state,
        )
}

fn module_load_gate_loader_runtime_retained_evidence_rejected(
    candidate: ModuleLoadGateLoaderRuntimeCandidate,
) -> bool {
    module_load_gate_loader_runtime_reference_rejected(candidate.manifest_reference_state)
        || module_load_gate_loader_runtime_reference_rejected(candidate.artifact_reference_state)
        || module_load_gate_loader_runtime_reference_rejected(candidate.vm_report_reference_state)
        || module_load_gate_loader_runtime_reference_rejected(
            candidate.local_attestation_reference_state,
        )
        || module_load_gate_loader_runtime_reference_rejected(
            candidate.local_approval_reference_state,
        )
        || module_load_gate_loader_runtime_reference_rejected(
            candidate.computed_grant_reference_state,
        )
        || module_load_gate_loader_runtime_reference_rejected(
            candidate.audit_rollback_reference_state,
        )
        || module_load_gate_loader_runtime_reference_rejected(
            candidate.service_slot_reservation_state,
        )
}

fn module_load_gate_loader_runtime_retained_evidence_reason(
    candidate: ModuleLoadGateLoaderRuntimeCandidate,
) -> &'static str {
    if !module_load_gate_loader_runtime_reference_available(candidate.manifest_reference_state) {
        return candidate.manifest_reference_reason;
    }
    if !module_load_gate_loader_runtime_reference_available(candidate.artifact_reference_state) {
        return candidate.artifact_reference_reason;
    }
    if !module_load_gate_loader_runtime_reference_available(candidate.vm_report_reference_state) {
        return candidate.vm_report_reference_reason;
    }
    if !module_load_gate_loader_runtime_reference_available(
        candidate.local_attestation_reference_state,
    ) {
        return candidate.local_attestation_reference_reason;
    }
    if !module_load_gate_loader_runtime_reference_available(
        candidate.local_approval_reference_state,
    ) {
        return candidate.local_approval_reference_reason;
    }
    if !module_load_gate_loader_runtime_reference_available(
        candidate.computed_grant_reference_state,
    ) {
        return candidate.computed_grant_reference_reason;
    }
    if !module_load_gate_loader_runtime_reference_available(
        candidate.audit_rollback_reference_state,
    ) {
        return candidate.audit_rollback_reference_reason;
    }
    if !module_load_gate_loader_runtime_reference_available(
        candidate.service_slot_reservation_state,
    ) {
        return candidate.service_slot_reservation_reason;
    }
    "retained_module_evidence_available"
}

fn module_load_gate_loader_runtime_reference_available(state: &'static str) -> bool {
    method_eq(state, "available")
}

fn module_load_gate_loader_runtime_reference_rejected(state: &'static str) -> bool {
    method_eq(state, "rejected")
}
