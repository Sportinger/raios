use crate::{
    agent_protocol_module_load_gate_selftest_eval::{
        computed_module_audit_record_hash, computed_module_grant_hash,
        computed_module_rollback_plan_hash, computed_module_service_slot_reservation_hash,
        evaluate_module_load_gate_audit_rollback_candidate,
        evaluate_module_load_gate_loader_runtime_candidate,
        evaluate_module_load_gate_retained_candidate,
        evaluate_module_load_gate_service_slot_candidate,
    },
    agent_protocol_module_types::*,
    agent_protocol_support::{
        method_eq, parse_current_boot_event_id, run_selftest_cases_with, CaseSpec,
    },
    event_log,
    module_evidence::{ModuleAuditRecordHashInput, ModuleServiceSlotReservationHashInput},
};

#[derive(Clone, Copy)]
enum LoadGateSelftestMutation {
    None,
    Missing,
    Stale,
    PreviousBoot,
    WrongSchema,
    Substituted,
    RetainedReferenceHashMismatch,
    AuditComputedGrantHashMismatch,
    AuditRecordHashMismatch,
    RollbackPlanHashMismatch,
    AuditServiceSlotMismatch,
    MissingDurableAuditRecord,
    MissingRollbackPlan,
    DurableAuditSchemaMismatch,
    RollbackPlanSchemaMismatch,
    AuditRetainedGrantMismatch,
    AuditManifestMismatch,
    AuditArtifactMismatch,
    AuditVmReportMismatch,
    AuditLocalAttestationMismatch,
    AuditLocalApprovalMismatch,
    AuditRollbackPlanMismatch,
    RollbackArtifactMismatch,
    RollbackServiceSlotMismatch,
    ServiceSlotGrantWrongSchema,
    ServiceSlotAuditWrongSchema,
    ServiceSlotComputedGrantHashMismatch,
    ServiceSlotAuditRecordHashMismatch,
    ServiceSlotRollbackPlanHashMismatch,
    ServiceSlotInventoryHashMismatch,
    ServiceSlotIdMismatch,
    ServiceSlotReservationHashMismatch,
    LoaderMissingManifest,
    LoaderRejectedArtifact,
    LoaderMissingServiceSlot,
    LoaderRejectedServiceSlot,
}

const fn load_gate_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    mutation: LoadGateSelftestMutation,
) -> CaseSpec<LoadGateSelftestMutation> {
    CaseSpec {
        name,
        expected_status,
        expected_reason,
        mutation,
        require_live_retained: false,
    }
}
pub(crate) fn module_load_gate_retained_selftest_cases(
) -> [ModuleLoadGateRetainedSelfTestCase; MODULE_LOAD_GATE_RETAINED_SELFTEST_CASES] {
    run_selftest_cases_with(
        module_load_gate_retained_valid_candidate(),
        &RETAINED_CASES,
        apply_retained_case,
        evaluate_retained_case,
        module_load_gate_retained_selftest_case_from_spec,
    )
}

const RETAINED_CASES: [CaseSpec<LoadGateSelftestMutation>;
    MODULE_LOAD_GATE_RETAINED_SELFTEST_CASES] = [
    load_gate_case(
        "missing_retained_reference",
        "missing",
        "computed_capability_grant_reference_missing",
        LoadGateSelftestMutation::Missing,
    ),
    load_gate_case(
        "accepted_current_boot_reference_still_denied",
        "retained_hash_reference_only",
        "retained_computed_grant_reference_not_authorizing",
        LoadGateSelftestMutation::None,
    ),
    load_gate_case(
        "stale_dropped_retained_reference_event_id",
        "rejected",
        "retained_reference_stale_or_dropped_event_id",
        LoadGateSelftestMutation::Stale,
    ),
    load_gate_case(
        "previous_boot_or_unretained_reference",
        "rejected",
        "retained_reference_previous_boot_or_unretained",
        LoadGateSelftestMutation::PreviousBoot,
    ),
    load_gate_case(
        "wrong_schema_or_variant_substitution",
        "rejected",
        "retained_reference_wrong_schema_or_variant",
        LoadGateSelftestMutation::WrongSchema,
    ),
    load_gate_case(
        "substituted_retained_reference_record",
        "rejected",
        "retained_reference_substituted_record",
        LoadGateSelftestMutation::Substituted,
    ),
    load_gate_case(
        "mismatched_computed_grant_hash",
        "rejected",
        "retained_reference_hash_mismatch",
        LoadGateSelftestMutation::RetainedReferenceHashMismatch,
    ),
];

fn module_load_gate_retained_valid_candidate() -> ModuleLoadGateRetainedCandidate {
    let valid_reference = module_load_gate_test_reference(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    ModuleLoadGateRetainedCandidate {
        scope: "current_boot",
        retained: true,
        schema_ok: true,
        event_reference: Some(valid_reference),
        candidate_reference: Some(valid_reference),
    }
}

fn apply_retained_case(
    candidate: &mut ModuleLoadGateRetainedCandidate,
    mutation: LoadGateSelftestMutation,
) {
    match mutation {
        LoadGateSelftestMutation::None => {}
        LoadGateSelftestMutation::Missing => {
            candidate.retained = false;
            candidate.event_reference = None;
            candidate.candidate_reference = None;
        }
        LoadGateSelftestMutation::Stale => candidate.retained = false,
        LoadGateSelftestMutation::PreviousBoot => candidate.scope = "previous_boot",
        LoadGateSelftestMutation::WrongSchema => candidate.schema_ok = false,
        LoadGateSelftestMutation::Substituted => {
            candidate.candidate_reference = Some(module_load_gate_test_reference(
                MODULE_GRANT_MISMATCH_MANIFEST_HASH,
                MODULE_GRANT_TEST_ARTIFACT_HASH,
                MODULE_GRANT_TEST_VM_REPORT_HASH,
                MODULE_GRANT_TEST_ATTESTATION_HASH,
            ));
        }
        LoadGateSelftestMutation::RetainedReferenceHashMismatch => {
            let mismatched = event_log::ModuleComputedGrantReference {
                computed_grant_hash: [0x66; 32],
                manifest_hash: MODULE_GRANT_TEST_MANIFEST_HASH,
                artifact_hash: MODULE_GRANT_TEST_ARTIFACT_HASH,
                vm_report_hash: MODULE_GRANT_TEST_VM_REPORT_HASH,
                local_attestation_hash: MODULE_GRANT_TEST_ATTESTATION_HASH,
            };
            candidate.event_reference = Some(mismatched);
            candidate.candidate_reference = Some(mismatched);
        }
        _ => {}
    }
}

fn evaluate_retained_case(
    candidate: ModuleLoadGateRetainedCandidate,
    _require_live_retained: bool,
) -> ModuleLoadGateRetainedCheck {
    evaluate_module_load_gate_retained_candidate(candidate)
}

pub(crate) fn module_load_gate_audit_rollback_selftest_cases(
) -> [ModuleLoadGateAuditRollbackSelfTestCase; MODULE_LOAD_GATE_AUDIT_ROLLBACK_SELFTEST_CASES] {
    run_selftest_cases_with(
        module_load_gate_test_audit_rollback_candidate(),
        &AUDIT_ROLLBACK_CASES,
        apply_audit_rollback_case,
        evaluate_audit_rollback_case,
        module_load_gate_audit_rollback_selftest_case_from_spec,
    )
}

const AUDIT_ROLLBACK_CASES: [CaseSpec<LoadGateSelftestMutation>;
    MODULE_LOAD_GATE_AUDIT_ROLLBACK_SELFTEST_CASES] = [
    load_gate_case(
        "missing_retained_audit_rollback_reference",
        "missing",
        "retained_audit_rollback_reference_missing",
        LoadGateSelftestMutation::Missing,
    ),
    load_gate_case(
        "stale_dropped_retained_audit_rollback_reference_event_id",
        "rejected",
        "retained_audit_rollback_reference_stale_or_dropped_event_id",
        LoadGateSelftestMutation::Stale,
    ),
    load_gate_case(
        "previous_boot_or_unretained_audit_rollback_reference",
        "rejected",
        "retained_audit_rollback_reference_previous_boot_or_unretained",
        LoadGateSelftestMutation::PreviousBoot,
    ),
    load_gate_case(
        "retained_audit_rollback_wrong_schema_or_variant",
        "rejected",
        "retained_audit_rollback_reference_wrong_schema_or_variant",
        LoadGateSelftestMutation::WrongSchema,
    ),
    load_gate_case(
        "substituted_retained_audit_rollback_reference",
        "rejected",
        "retained_audit_rollback_reference_substituted_record",
        LoadGateSelftestMutation::Substituted,
    ),
    load_gate_case(
        "retained_audit_rollback_computed_grant_hash_mismatch",
        "rejected",
        "retained_audit_rollback_computed_grant_hash_mismatch",
        LoadGateSelftestMutation::AuditComputedGrantHashMismatch,
    ),
    load_gate_case(
        "retained_audit_record_hash_mismatch",
        "rejected",
        "retained_audit_record_hash_mismatch",
        LoadGateSelftestMutation::AuditRecordHashMismatch,
    ),
    load_gate_case(
        "retained_rollback_plan_hash_mismatch",
        "rejected",
        "retained_rollback_plan_hash_mismatch",
        LoadGateSelftestMutation::RollbackPlanHashMismatch,
    ),
    load_gate_case(
        "retained_audit_rollback_service_slot_mismatch",
        "rejected",
        "retained_audit_rollback_service_slot_mismatch",
        LoadGateSelftestMutation::AuditServiceSlotMismatch,
    ),
    load_gate_case(
        "missing_durable_audit_record",
        "missing",
        "durable_audit_write_missing",
        LoadGateSelftestMutation::MissingDurableAuditRecord,
    ),
    load_gate_case(
        "missing_rollback_plan",
        "missing",
        "rollback_install_missing",
        LoadGateSelftestMutation::MissingRollbackPlan,
    ),
    load_gate_case(
        "durable_audit_record_schema_mismatch",
        "rejected",
        "durable_audit_record_schema_mismatch",
        LoadGateSelftestMutation::DurableAuditSchemaMismatch,
    ),
    load_gate_case(
        "rollback_plan_schema_mismatch",
        "rejected",
        "rollback_plan_schema_mismatch",
        LoadGateSelftestMutation::RollbackPlanSchemaMismatch,
    ),
    load_gate_case(
        "valid_audit_and_rollback_still_denied",
        "validated_non_authorizing",
        "loader_and_service_slot_missing",
        LoadGateSelftestMutation::None,
    ),
    load_gate_case(
        "audit_retained_grant_hash_mismatch",
        "rejected",
        "audit_retained_grant_hash_mismatch",
        LoadGateSelftestMutation::AuditRetainedGrantMismatch,
    ),
    load_gate_case(
        "audit_manifest_hash_mismatch",
        "rejected",
        "audit_manifest_hash_mismatch",
        LoadGateSelftestMutation::AuditManifestMismatch,
    ),
    load_gate_case(
        "audit_artifact_hash_mismatch",
        "rejected",
        "audit_artifact_hash_mismatch",
        LoadGateSelftestMutation::AuditArtifactMismatch,
    ),
    load_gate_case(
        "audit_vm_report_hash_mismatch",
        "rejected",
        "audit_vm_test_report_hash_mismatch",
        LoadGateSelftestMutation::AuditVmReportMismatch,
    ),
    load_gate_case(
        "audit_local_attestation_hash_mismatch",
        "rejected",
        "audit_local_attestation_hash_mismatch",
        LoadGateSelftestMutation::AuditLocalAttestationMismatch,
    ),
    load_gate_case(
        "local_approval_mismatch",
        "rejected",
        "local_approval_missing_or_mismatch",
        LoadGateSelftestMutation::AuditLocalApprovalMismatch,
    ),
    load_gate_case(
        "audit_rollback_plan_hash_mismatch",
        "rejected",
        "audit_rollback_plan_hash_mismatch",
        LoadGateSelftestMutation::AuditRollbackPlanMismatch,
    ),
    load_gate_case(
        "rollback_artifact_hash_mismatch",
        "rejected",
        "rollback_artifact_hash_mismatch",
        LoadGateSelftestMutation::RollbackArtifactMismatch,
    ),
    load_gate_case(
        "rollback_service_slot_mismatch",
        "rejected",
        "rollback_service_slot_mismatch",
        LoadGateSelftestMutation::RollbackServiceSlotMismatch,
    ),
];

fn apply_audit_rollback_case(
    candidate: &mut ModuleLoadGateAuditRollbackCandidate,
    mutation: LoadGateSelftestMutation,
) {
    let valid_reference =
        module_load_gate_test_audit_rollback_reference(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID);
    match mutation {
        LoadGateSelftestMutation::None => {}
        LoadGateSelftestMutation::Missing => {
            candidate.retained_audit_rollback_reference.retained = false;
            candidate.retained_audit_rollback_reference.event_reference = None;
            candidate
                .retained_audit_rollback_reference
                .candidate_reference = None;
        }
        LoadGateSelftestMutation::Stale => {
            candidate.retained_audit_rollback_reference.retained = false;
            candidate.retained_audit_rollback_reference.event_reference = valid_reference;
            candidate
                .retained_audit_rollback_reference
                .candidate_reference = valid_reference;
        }
        LoadGateSelftestMutation::PreviousBoot => {
            candidate.retained_audit_rollback_reference.scope = "previous_boot";
        }
        LoadGateSelftestMutation::WrongSchema => {
            candidate.retained_audit_rollback_reference.schema_ok = false;
        }
        LoadGateSelftestMutation::Substituted => {
            candidate
                .retained_audit_rollback_reference
                .candidate_reference = module_load_gate_test_audit_rollback_reference_with_manifest(
                MODULE_GRANT_MISMATCH_MANIFEST_HASH,
                MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
            );
        }
        LoadGateSelftestMutation::AuditComputedGrantHashMismatch => {
            let reference = module_load_gate_test_audit_rollback_reference_with_override(
                MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
                Some([0x99; 32]),
                None,
                None,
            );
            candidate.retained_audit_rollback_reference.event_reference = reference;
            candidate
                .retained_audit_rollback_reference
                .candidate_reference = reference;
        }
        LoadGateSelftestMutation::AuditRecordHashMismatch => {
            let reference = module_load_gate_test_audit_rollback_reference_with_override(
                MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
                None,
                None,
                Some([0xaa; 32]),
            );
            candidate.retained_audit_rollback_reference.event_reference = reference;
            candidate
                .retained_audit_rollback_reference
                .candidate_reference = reference;
        }
        LoadGateSelftestMutation::RollbackPlanHashMismatch => {
            let reference = module_load_gate_test_audit_rollback_reference_with_override(
                MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
                None,
                Some([0xbb; 32]),
                None,
            );
            candidate.retained_audit_rollback_reference.event_reference = reference;
            candidate
                .retained_audit_rollback_reference
                .candidate_reference = reference;
        }
        LoadGateSelftestMutation::AuditServiceSlotMismatch => {
            let reference =
                module_load_gate_test_audit_rollback_reference("ram_only:svc.test.other");
            candidate.retained_audit_rollback_reference.event_reference = reference;
            candidate
                .retained_audit_rollback_reference
                .candidate_reference = reference;
        }
        LoadGateSelftestMutation::MissingDurableAuditRecord => {
            candidate.durable_audit_record = false
        }
        LoadGateSelftestMutation::MissingRollbackPlan => candidate.rollback_plan = false,
        LoadGateSelftestMutation::DurableAuditSchemaMismatch => candidate.audit_schema_ok = false,
        LoadGateSelftestMutation::RollbackPlanSchemaMismatch => {
            candidate.rollback_schema_ok = false
        }
        LoadGateSelftestMutation::AuditRetainedGrantMismatch => {
            candidate.audit_binds_retained_grant = false
        }
        LoadGateSelftestMutation::AuditManifestMismatch => candidate.audit_binds_manifest = false,
        LoadGateSelftestMutation::AuditArtifactMismatch => candidate.audit_binds_artifact = false,
        LoadGateSelftestMutation::AuditVmReportMismatch => candidate.audit_binds_vm_report = false,
        LoadGateSelftestMutation::AuditLocalAttestationMismatch => {
            candidate.audit_binds_local_attestation = false
        }
        LoadGateSelftestMutation::AuditLocalApprovalMismatch => {
            candidate.audit_binds_local_approval = false
        }
        LoadGateSelftestMutation::AuditRollbackPlanMismatch => {
            candidate.audit_binds_rollback_plan = false
        }
        LoadGateSelftestMutation::RollbackArtifactMismatch => {
            candidate.rollback_binds_artifact = false
        }
        LoadGateSelftestMutation::RollbackServiceSlotMismatch => {
            candidate.rollback_binds_service_slot = false
        }
        _ => {}
    }
}

fn evaluate_audit_rollback_case(
    candidate: ModuleLoadGateAuditRollbackCandidate,
    _require_live_retained: bool,
) -> ModuleLoadGateAuditRollbackEvaluation {
    evaluate_module_load_gate_audit_rollback_candidate(candidate)
}

pub(crate) fn module_load_gate_service_slot_selftest_cases(
) -> [ModuleLoadGateServiceSlotSelfTestCase; MODULE_LOAD_GATE_SERVICE_SLOT_SELFTEST_CASES] {
    run_selftest_cases_with(
        module_load_gate_test_service_slot_candidate(),
        &SERVICE_SLOT_CASES,
        apply_service_slot_case,
        evaluate_service_slot_case,
        module_load_gate_service_slot_selftest_case_from_spec,
    )
}

const SERVICE_SLOT_CASES: [CaseSpec<LoadGateSelftestMutation>;
    MODULE_LOAD_GATE_SERVICE_SLOT_SELFTEST_CASES] = [
    load_gate_case(
        "missing_retained_service_slot_reservation",
        "missing",
        "retained_service_slot_reservation_missing",
        LoadGateSelftestMutation::Missing,
    ),
    load_gate_case(
        "accepted_current_boot_reservation_still_denied",
        "retained_hash_reference_only_not_allocated",
        "retained_service_slot_reservation_not_allocated",
        LoadGateSelftestMutation::None,
    ),
    load_gate_case(
        "stale_dropped_retained_service_slot_reservation_event_id",
        "rejected",
        "retained_service_slot_reservation_stale_or_dropped_event_id",
        LoadGateSelftestMutation::Stale,
    ),
    load_gate_case(
        "retained_service_slot_wrong_schema_or_variant",
        "rejected",
        "retained_service_slot_reservation_wrong_schema_or_variant",
        LoadGateSelftestMutation::WrongSchema,
    ),
    load_gate_case(
        "substituted_retained_service_slot_reservation",
        "rejected",
        "retained_service_slot_reservation_substituted_record",
        LoadGateSelftestMutation::Substituted,
    ),
    load_gate_case(
        "retained_service_slot_grant_wrong_schema_or_variant",
        "rejected",
        "retained_service_slot_reservation_wrong_schema_or_variant",
        LoadGateSelftestMutation::ServiceSlotGrantWrongSchema,
    ),
    load_gate_case(
        "retained_service_slot_audit_rollback_wrong_schema_or_variant",
        "rejected",
        "retained_service_slot_reservation_wrong_schema_or_variant",
        LoadGateSelftestMutation::ServiceSlotAuditWrongSchema,
    ),
    load_gate_case(
        "retained_service_slot_computed_grant_hash_mismatch",
        "rejected",
        "retained_service_slot_reservation_computed_grant_hash_mismatch",
        LoadGateSelftestMutation::ServiceSlotComputedGrantHashMismatch,
    ),
    load_gate_case(
        "retained_service_slot_audit_record_hash_mismatch",
        "rejected",
        "retained_service_slot_reservation_audit_record_hash_mismatch",
        LoadGateSelftestMutation::ServiceSlotAuditRecordHashMismatch,
    ),
    load_gate_case(
        "retained_service_slot_rollback_plan_hash_mismatch",
        "rejected",
        "retained_service_slot_reservation_rollback_plan_hash_mismatch",
        LoadGateSelftestMutation::ServiceSlotRollbackPlanHashMismatch,
    ),
    load_gate_case(
        "retained_service_slot_inventory_hash_mismatch",
        "rejected",
        "retained_service_slot_reservation_pre_load_inventory_hash_mismatch",
        LoadGateSelftestMutation::ServiceSlotInventoryHashMismatch,
    ),
    load_gate_case(
        "retained_service_slot_service_slot_mismatch",
        "rejected",
        "retained_service_slot_reservation_service_slot_mismatch",
        LoadGateSelftestMutation::ServiceSlotIdMismatch,
    ),
    load_gate_case(
        "retained_service_slot_reservation_hash_mismatch",
        "rejected",
        "retained_service_slot_reservation_hash_mismatch",
        LoadGateSelftestMutation::ServiceSlotReservationHashMismatch,
    ),
];

fn apply_service_slot_case(
    candidate: &mut ModuleLoadGateServiceSlotCandidate,
    mutation: LoadGateSelftestMutation,
) {
    let valid_reservation = module_load_gate_test_service_slot_reservation();
    let reservation = match mutation {
        LoadGateSelftestMutation::Substituted => {
            module_load_gate_test_service_slot_reservation_with_override(
                Some([0x91; 32]),
                None,
                None,
                None,
                None,
                None,
            )
        }
        LoadGateSelftestMutation::ServiceSlotComputedGrantHashMismatch => {
            module_load_gate_test_service_slot_reservation_with_override(
                Some([0x92; 32]),
                None,
                None,
                None,
                None,
                None,
            )
        }
        LoadGateSelftestMutation::ServiceSlotAuditRecordHashMismatch => {
            module_load_gate_test_service_slot_reservation_with_override(
                None,
                Some([0x93; 32]),
                None,
                None,
                None,
                None,
            )
        }
        LoadGateSelftestMutation::ServiceSlotRollbackPlanHashMismatch => {
            module_load_gate_test_service_slot_reservation_with_override(
                None,
                None,
                Some([0x94; 32]),
                None,
                None,
                None,
            )
        }
        LoadGateSelftestMutation::ServiceSlotInventoryHashMismatch => {
            module_load_gate_test_service_slot_reservation_with_override(
                None,
                None,
                None,
                Some([0x95; 32]),
                None,
                None,
            )
        }
        LoadGateSelftestMutation::ServiceSlotIdMismatch => {
            module_load_gate_test_service_slot_reservation_with_override(
                None,
                None,
                None,
                None,
                Some("ram_only:svc.test.other"),
                None,
            )
        }
        LoadGateSelftestMutation::ServiceSlotReservationHashMismatch => {
            module_load_gate_test_service_slot_reservation_with_override(
                None,
                None,
                None,
                None,
                None,
                Some([0x96; 32]),
            )
        }
        _ => valid_reservation,
    };

    match mutation {
        LoadGateSelftestMutation::None => {}
        LoadGateSelftestMutation::Missing => {
            candidate.service_slot_reservation.event_reservation = None;
            candidate.service_slot_reservation.candidate_reservation = None;
        }
        LoadGateSelftestMutation::Stale => {
            candidate.service_slot_reservation.retained = false;
            candidate.service_slot_reservation.event_reservation = valid_reservation;
            candidate.service_slot_reservation.candidate_reservation = valid_reservation;
        }
        LoadGateSelftestMutation::WrongSchema => {
            candidate.service_slot_reservation.schema_ok = false;
        }
        LoadGateSelftestMutation::Substituted => {
            candidate.service_slot_reservation.event_reservation = valid_reservation;
            candidate.service_slot_reservation.candidate_reservation = reservation;
        }
        LoadGateSelftestMutation::ServiceSlotGrantWrongSchema => {
            candidate.service_slot_reservation.grant_event_schema_ok = false;
        }
        LoadGateSelftestMutation::ServiceSlotAuditWrongSchema => {
            candidate.service_slot_reservation.audit_event_schema_ok = false;
        }
        LoadGateSelftestMutation::ServiceSlotComputedGrantHashMismatch
        | LoadGateSelftestMutation::ServiceSlotAuditRecordHashMismatch
        | LoadGateSelftestMutation::ServiceSlotRollbackPlanHashMismatch
        | LoadGateSelftestMutation::ServiceSlotInventoryHashMismatch
        | LoadGateSelftestMutation::ServiceSlotIdMismatch
        | LoadGateSelftestMutation::ServiceSlotReservationHashMismatch => {
            candidate.service_slot_reservation.event_reservation = reservation;
            candidate.service_slot_reservation.candidate_reservation = reservation;
        }
        _ => {}
    }
}

fn evaluate_service_slot_case(
    candidate: ModuleLoadGateServiceSlotCandidate,
    _require_live_retained: bool,
) -> ModuleLoadGateServiceSlotEvaluation {
    evaluate_module_load_gate_service_slot_candidate(candidate)
}

pub(crate) fn module_load_gate_loader_runtime_selftest_cases(
) -> [ModuleLoadGateLoaderRuntimeSelfTestCase; MODULE_LOAD_GATE_LOADER_RUNTIME_SELFTEST_CASES] {
    run_selftest_cases_with(
        module_load_gate_loader_runtime_ready_candidate(),
        &LOADER_RUNTIME_CASES,
        apply_loader_runtime_case,
        evaluate_loader_runtime_case,
        module_load_gate_loader_runtime_selftest_case_from_spec,
    )
}

const LOADER_RUNTIME_CASES: [CaseSpec<LoadGateSelftestMutation>;
    MODULE_LOAD_GATE_LOADER_RUNTIME_SELFTEST_CASES] = [
    load_gate_case(
        "missing_manifest_reference",
        "denied_missing_retained_module_evidence",
        "retained_module_manifest_reference_missing",
        LoadGateSelftestMutation::LoaderMissingManifest,
    ),
    load_gate_case(
        "rejected_artifact_reference",
        "denied_missing_retained_module_evidence",
        "retained_candidate_artifact_reference_hash_mismatch",
        LoadGateSelftestMutation::LoaderRejectedArtifact,
    ),
    load_gate_case(
        "missing_service_slot_reservation",
        "denied_missing_retained_module_evidence",
        "ram_only_service_slot_unallocated",
        LoadGateSelftestMutation::LoaderMissingServiceSlot,
    ),
    load_gate_case(
        "rejected_service_slot_reservation",
        "denied_missing_retained_module_evidence",
        "retained_service_slot_reservation_hash_mismatch",
        LoadGateSelftestMutation::LoaderRejectedServiceSlot,
    ),
    load_gate_case(
        "all_retained_evidence_ready_allocator_authority_boundary_denied",
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
        LoadGateSelftestMutation::None,
    ),
];

fn apply_loader_runtime_case(
    candidate: &mut ModuleLoadGateLoaderRuntimeCandidate,
    mutation: LoadGateSelftestMutation,
) {
    match mutation {
        LoadGateSelftestMutation::None => {}
        LoadGateSelftestMutation::LoaderMissingManifest => {
            *candidate = ModuleLoadGateLoaderRuntimeCandidate {
                manifest_reference_state: "missing",
                manifest_reference_reason: "retained_module_manifest_reference_missing",
                artifact_reference_state: "missing",
                artifact_reference_reason: "retained_candidate_artifact_reference_missing",
                vm_report_reference_state: "missing",
                vm_report_reference_reason: "retained_vm_test_report_reference_missing",
                local_attestation_reference_state: "missing",
                local_attestation_reference_reason: "retained_local_attestation_reference_missing",
                local_approval_reference_state: "missing",
                local_approval_reference_reason: "retained_local_approval_reference_missing",
                computed_grant_reference_state: "missing",
                computed_grant_reference_reason: "computed_capability_grant_reference_missing",
                audit_rollback_reference_state: "missing",
                audit_rollback_reference_reason: "rollback_install_missing",
                service_slot_reservation_state: "missing",
                service_slot_reservation_reason: "ram_only_service_slot_unallocated",
            };
        }
        LoadGateSelftestMutation::LoaderRejectedArtifact => {
            candidate.artifact_reference_state = "rejected";
            candidate.artifact_reference_reason =
                "retained_candidate_artifact_reference_hash_mismatch";
            candidate.vm_report_reference_state = "missing";
            candidate.vm_report_reference_reason = "retained_vm_test_report_reference_missing";
            candidate.local_attestation_reference_state = "missing";
            candidate.local_attestation_reference_reason =
                "retained_local_attestation_reference_missing";
            candidate.local_approval_reference_state = "missing";
            candidate.local_approval_reference_reason = "retained_local_approval_reference_missing";
            candidate.computed_grant_reference_state = "missing";
            candidate.computed_grant_reference_reason =
                "computed_capability_grant_reference_missing";
            candidate.audit_rollback_reference_state = "missing";
            candidate.audit_rollback_reference_reason = "rollback_install_missing";
            candidate.service_slot_reservation_state = "missing";
            candidate.service_slot_reservation_reason = "ram_only_service_slot_unallocated";
        }
        LoadGateSelftestMutation::LoaderMissingServiceSlot => {
            candidate.service_slot_reservation_state = "missing";
            candidate.service_slot_reservation_reason = "ram_only_service_slot_unallocated";
        }
        LoadGateSelftestMutation::LoaderRejectedServiceSlot => {
            candidate.service_slot_reservation_state = "rejected";
            candidate.service_slot_reservation_reason =
                "retained_service_slot_reservation_hash_mismatch";
        }
        _ => {}
    }
}

fn evaluate_loader_runtime_case(
    candidate: ModuleLoadGateLoaderRuntimeCandidate,
    _require_live_retained: bool,
) -> ModuleLoadGateLoaderRuntimeEvaluation {
    evaluate_module_load_gate_loader_runtime_candidate(candidate)
}

fn loader_runtime_expected_states(
    mutation: LoadGateSelftestMutation,
) -> (&'static str, &'static str, &'static str) {
    match mutation {
        LoadGateSelftestMutation::None => (
            "available",
            "defined_non_authorizing",
            "blocked_by_service_slot_allocator_authority",
        ),
        LoadGateSelftestMutation::LoaderRejectedArtifact => (
            "rejected",
            "blocked_by_service_slot_reservation",
            "blocked_by_retained_module_evidence",
        ),
        LoadGateSelftestMutation::LoaderRejectedServiceSlot => (
            "rejected",
            "blocked_by_rejected_service_slot_reservation",
            "blocked_by_retained_module_evidence",
        ),
        LoadGateSelftestMutation::LoaderMissingServiceSlot => (
            "missing",
            "blocked_by_service_slot_reservation",
            "blocked_by_retained_module_evidence",
        ),
        _ => (
            "missing",
            "blocked_by_service_slot_reservation",
            "blocked_by_retained_module_evidence",
        ),
    }
}

fn module_load_gate_retained_selftest_case_from_spec(
    spec: &CaseSpec<LoadGateSelftestMutation>,
    actual: ModuleLoadGateRetainedCheck,
) -> ModuleLoadGateRetainedSelfTestCase {
    ModuleLoadGateRetainedSelfTestCase {
        name: spec.name,
        expected_status: spec.expected_status,
        expected_reason: spec.expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, spec.expected_status)
            && method_eq(actual.reason, spec.expected_reason)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_audit_rollback_selftest_case_from_spec(
    spec: &CaseSpec<LoadGateSelftestMutation>,
    actual: ModuleLoadGateAuditRollbackEvaluation,
) -> ModuleLoadGateAuditRollbackSelfTestCase {
    ModuleLoadGateAuditRollbackSelfTestCase {
        name: spec.name,
        expected_status: spec.expected_status,
        expected_reason: spec.expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, spec.expected_status)
            && method_eq(actual.reason, spec.expected_reason)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_service_slot_selftest_case_from_spec(
    spec: &CaseSpec<LoadGateSelftestMutation>,
    actual: ModuleLoadGateServiceSlotEvaluation,
) -> ModuleLoadGateServiceSlotSelfTestCase {
    let expected_hash_exposed = method_eq(
        spec.expected_status,
        "retained_hash_reference_only_not_allocated",
    );
    ModuleLoadGateServiceSlotSelfTestCase {
        name: spec.name,
        expected_status: spec.expected_status,
        expected_reason: spec.expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_service_slot_state: actual.service_slot_state,
        accepted_service_slot_reservation_hash: actual.accepted_service_slot_reservation_hash,
        passed: method_eq(actual.status, spec.expected_status)
            && method_eq(actual.reason, spec.expected_reason)
            && actual.accepted_service_slot_reservation_hash == expected_hash_exposed
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_loader_runtime_selftest_case_from_spec(
    spec: &CaseSpec<LoadGateSelftestMutation>,
    actual: ModuleLoadGateLoaderRuntimeEvaluation,
) -> ModuleLoadGateLoaderRuntimeSelfTestCase {
    let (
        expected_retained_module_evidence_state,
        expected_service_slot_allocator_state,
        expected_loader_runtime_state,
    ) = loader_runtime_expected_states(spec.mutation);
    let expected_retained_module_evidence_reason = if method_eq(
        spec.expected_status,
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
    ) {
        "retained_module_evidence_available"
    } else {
        spec.expected_reason
    };
    let expected_service_slot_allocator_status = if method_eq(
        expected_service_slot_allocator_state,
        "defined_non_authorizing",
    ) {
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS
    } else {
        "blocked"
    };
    let expected_service_slot_allocator_reason = if method_eq(
        expected_service_slot_allocator_state,
        "defined_non_authorizing",
    ) {
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON
    } else if method_eq(
        expected_service_slot_allocator_state,
        "blocked_by_rejected_service_slot_reservation",
    ) {
        spec.expected_reason
    } else {
        "retained_service_slot_reservation_missing"
    };
    ModuleLoadGateLoaderRuntimeSelfTestCase {
        name: spec.name,
        expected_status: spec.expected_status,
        expected_reason: spec.expected_reason,
        expected_retained_module_evidence_state,
        expected_service_slot_allocator_state,
        expected_loader_runtime_state,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_retained_module_evidence_state: actual.retained_module_evidence_state,
        actual_retained_module_evidence_reason: actual.retained_module_evidence_reason,
        actual_service_slot_allocator_state: actual.service_slot_allocator_state,
        actual_service_slot_allocator_status: actual.service_slot_allocator_status,
        actual_service_slot_allocator_reason: actual.service_slot_allocator_reason,
        actual_loader_runtime_state: actual.loader_runtime_state,
        passed: method_eq(actual.status, spec.expected_status)
            && method_eq(actual.reason, spec.expected_reason)
            && method_eq(
                actual.retained_module_evidence_state,
                expected_retained_module_evidence_state,
            )
            && method_eq(
                actual.service_slot_allocator_state,
                expected_service_slot_allocator_state,
            )
            && method_eq(
                actual.retained_module_evidence_reason,
                expected_retained_module_evidence_reason,
            )
            && method_eq(
                actual.service_slot_allocator_status,
                expected_service_slot_allocator_status,
            )
            && method_eq(
                actual.service_slot_allocator_reason,
                expected_service_slot_allocator_reason,
            )
            && method_eq(actual.loader_runtime_state, expected_loader_runtime_state)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_test_audit_rollback_candidate() -> ModuleLoadGateAuditRollbackCandidate {
    let audit_rollback_reference =
        module_load_gate_test_audit_rollback_reference(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID);
    ModuleLoadGateAuditRollbackCandidate {
        retained_reference: true,
        retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
            scope: "current_boot",
            retained: true,
            schema_ok: true,
            event_reference: audit_rollback_reference,
            candidate_reference: audit_rollback_reference,
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

fn module_load_gate_test_audit_rollback_reference(
    ram_only_service_slot_id: &'static str,
) -> Option<event_log::ModuleAuditRollbackReference> {
    module_load_gate_test_audit_rollback_reference_with_manifest(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        ram_only_service_slot_id,
    )
}

fn module_load_gate_test_audit_rollback_reference_with_manifest(
    manifest_hash: [u8; 32],
    ram_only_service_slot_id: &'static str,
) -> Option<event_log::ModuleAuditRollbackReference> {
    module_load_gate_test_audit_rollback_reference_with_override(
        ram_only_service_slot_id,
        None,
        None,
        None,
    )
    .map(|mut reference| {
        reference.manifest_hash = manifest_hash;
        reference.computed_grant_hash = computed_module_grant_hash(
            manifest_hash,
            reference.artifact_hash,
            reference.vm_report_hash,
            reference.local_attestation_hash,
        );
        reference.audit_record_hash =
            computed_module_audit_record_hash(ModuleAuditRecordHashInput {
                denial_event_id: MODULE_AUDIT_TEST_DENIAL_EVENT_ID,
                retained_reference_event_id: MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
                computed_grant_hash: reference.computed_grant_hash,
                manifest_hash: reference.manifest_hash,
                artifact_hash: reference.artifact_hash,
                vm_report_hash: reference.vm_report_hash,
                local_attestation_hash: reference.local_attestation_hash,
                local_approval_hash: reference.local_approval_hash,
                rollback_plan_hash: reference.rollback_plan_hash,
                ram_only_service_slot_id: reference.ram_only_service_slot_id.as_str(),
            });
        reference
    })
}

fn module_load_gate_test_audit_rollback_reference_with_override(
    ram_only_service_slot_id: &'static str,
    computed_grant_hash_override: Option<[u8; 32]>,
    rollback_plan_hash_override: Option<[u8; 32]>,
    audit_record_hash_override: Option<[u8; 32]>,
) -> Option<event_log::ModuleAuditRollbackReference> {
    let computed_grant_hash = computed_module_grant_hash(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let rollback_plan_hash = computed_module_rollback_plan_hash(
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_AUDIT_TEST_PRE_INVENTORY_HASH,
        ram_only_service_slot_id,
        MODULE_AUDIT_TEST_CLEANUP_HASH,
    );
    let audit_record_hash = computed_module_audit_record_hash(ModuleAuditRecordHashInput {
        denial_event_id: MODULE_AUDIT_TEST_DENIAL_EVENT_ID,
        retained_reference_event_id: MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
        computed_grant_hash,
        manifest_hash: MODULE_GRANT_TEST_MANIFEST_HASH,
        artifact_hash: MODULE_GRANT_TEST_ARTIFACT_HASH,
        vm_report_hash: MODULE_GRANT_TEST_VM_REPORT_HASH,
        local_attestation_hash: MODULE_GRANT_TEST_ATTESTATION_HASH,
        local_approval_hash: MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH,
        rollback_plan_hash,
        ram_only_service_slot_id,
    });

    Some(event_log::ModuleAuditRollbackReference {
        audit_record_hash: audit_record_hash_override.unwrap_or(audit_record_hash),
        rollback_plan_hash: rollback_plan_hash_override.unwrap_or(rollback_plan_hash),
        computed_grant_hash: computed_grant_hash_override.unwrap_or(computed_grant_hash),
        manifest_hash: MODULE_GRANT_TEST_MANIFEST_HASH,
        artifact_hash: MODULE_GRANT_TEST_ARTIFACT_HASH,
        vm_report_hash: MODULE_GRANT_TEST_VM_REPORT_HASH,
        local_attestation_hash: MODULE_GRANT_TEST_ATTESTATION_HASH,
        local_approval_hash: MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH,
        pre_load_service_inventory_hash: MODULE_AUDIT_TEST_PRE_INVENTORY_HASH,
        cleanup_actions_hash: MODULE_AUDIT_TEST_CLEANUP_HASH,
        denial_event_id: parse_current_boot_event_id(MODULE_AUDIT_TEST_DENIAL_EVENT_ID)?,
        retained_reference_event_id: parse_current_boot_event_id(
            MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
        )?,
        ram_only_service_slot_id: event_log::ModuleServiceSlotId::new(ram_only_service_slot_id)?,
    })
}

fn module_load_gate_test_service_slot_candidate() -> ModuleLoadGateServiceSlotCandidate {
    let retained_reference = Some(module_load_gate_test_reference(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    ));
    let audit_rollback_reference =
        module_load_gate_test_audit_rollback_reference(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID);
    let service_slot_reservation = module_load_gate_test_service_slot_reservation();

    ModuleLoadGateServiceSlotCandidate {
        retained_reference,
        audit_rollback_reference,
        audit_rollback_valid: true,
        service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
            scope: "current_boot",
            retained: true,
            schema_ok: true,
            grant_event_schema_ok: true,
            audit_event_schema_ok: true,
            grant_event_reference: retained_reference,
            audit_event_reference: audit_rollback_reference,
            event_reservation: service_slot_reservation,
            candidate_reservation: service_slot_reservation,
        },
    }
}

fn module_load_gate_test_service_slot_reservation(
) -> Option<event_log::ModuleServiceSlotReservation> {
    module_load_gate_test_service_slot_reservation_with_override(None, None, None, None, None, None)
}

fn module_load_gate_test_service_slot_reservation_with_override(
    computed_grant_hash_override: Option<[u8; 32]>,
    audit_record_hash_override: Option<[u8; 32]>,
    rollback_plan_hash_override: Option<[u8; 32]>,
    pre_load_service_inventory_hash_override: Option<[u8; 32]>,
    ram_only_service_slot_id_override: Option<&'static str>,
    reservation_hash_override: Option<[u8; 32]>,
) -> Option<event_log::ModuleServiceSlotReservation> {
    let audit_rollback_reference =
        module_load_gate_test_audit_rollback_reference(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID)?;
    let ram_only_service_slot_id =
        ram_only_service_slot_id_override.unwrap_or(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID);
    let computed_grant_hash =
        computed_grant_hash_override.unwrap_or(audit_rollback_reference.computed_grant_hash);
    let audit_record_hash =
        audit_record_hash_override.unwrap_or(audit_rollback_reference.audit_record_hash);
    let rollback_plan_hash =
        rollback_plan_hash_override.unwrap_or(audit_rollback_reference.rollback_plan_hash);
    let pre_load_service_inventory_hash = pre_load_service_inventory_hash_override
        .unwrap_or(audit_rollback_reference.pre_load_service_inventory_hash);
    let reservation_hash =
        computed_module_service_slot_reservation_hash(ModuleServiceSlotReservationHashInput {
            retained_reference_event_id: MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
            retained_audit_rollback_reference_event_id:
                MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID,
            computed_grant_hash,
            audit_record_hash,
            rollback_plan_hash,
            pre_load_service_inventory_hash,
            ram_only_service_slot_id,
        });

    Some(event_log::ModuleServiceSlotReservation {
        reservation_hash: reservation_hash_override.unwrap_or(reservation_hash),
        retained_reference_event_id: parse_current_boot_event_id(
            MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
        )?,
        retained_audit_rollback_reference_event_id: parse_current_boot_event_id(
            MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID,
        )?,
        computed_grant_hash,
        audit_record_hash,
        rollback_plan_hash,
        pre_load_service_inventory_hash,
        ram_only_service_slot_id: event_log::ModuleServiceSlotId::new(ram_only_service_slot_id)?,
    })
}

fn module_load_gate_loader_runtime_ready_candidate() -> ModuleLoadGateLoaderRuntimeCandidate {
    ModuleLoadGateLoaderRuntimeCandidate {
        manifest_reference_state: "available",
        manifest_reference_reason: "retained_module_manifest_reference_available",
        artifact_reference_state: "available",
        artifact_reference_reason: "retained_candidate_artifact_reference_available",
        vm_report_reference_state: "available",
        vm_report_reference_reason: "retained_vm_test_report_reference_available",
        local_attestation_reference_state: "available",
        local_attestation_reference_reason: "retained_local_attestation_reference_available",
        local_approval_reference_state: "available",
        local_approval_reference_reason: "retained_local_approval_reference_available",
        computed_grant_reference_state: "available",
        computed_grant_reference_reason: "computed_capability_grant_reference_available",
        audit_rollback_reference_state: "available",
        audit_rollback_reference_reason: "retained_audit_rollback_reference_available",
        service_slot_reservation_state: "available",
        service_slot_reservation_reason: "retained_service_slot_reservation_available",
    }
}

pub(crate) fn module_load_gate_test_reference(
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> event_log::ModuleComputedGrantReference {
    event_log::ModuleComputedGrantReference {
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
    }
}
