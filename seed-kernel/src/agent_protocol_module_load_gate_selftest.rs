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
    agent_protocol_support::{method_eq, method_head_eq, parse_current_boot_event_id},
    event_log,
    module_evidence::{ModuleAuditRecordHashInput, ModuleServiceSlotReservationHashInput},
};
pub(crate) fn module_load_gate_retained_selftest_cases(
) -> [ModuleLoadGateRetainedSelfTestCase; MODULE_LOAD_GATE_RETAINED_SELFTEST_CASES] {
    let valid_reference = module_load_gate_test_reference(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let substituted_reference = module_load_gate_test_reference(
        MODULE_GRANT_MISMATCH_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let mismatched_hash_reference = event_log::ModuleComputedGrantReference {
        computed_grant_hash: [0x66; 32],
        manifest_hash: MODULE_GRANT_TEST_MANIFEST_HASH,
        artifact_hash: MODULE_GRANT_TEST_ARTIFACT_HASH,
        vm_report_hash: MODULE_GRANT_TEST_VM_REPORT_HASH,
        local_attestation_hash: MODULE_GRANT_TEST_ATTESTATION_HASH,
    };

    [
        module_load_gate_retained_selftest_case(
            "missing_retained_reference",
            "missing",
            "computed_capability_grant_reference_missing",
            ModuleLoadGateRetainedCandidate {
                scope: "current_boot",
                retained: false,
                schema_ok: true,
                event_reference: None,
                candidate_reference: None,
            },
        ),
        module_load_gate_retained_selftest_case(
            "accepted_current_boot_reference_still_denied",
            "retained_hash_reference_only",
            "retained_computed_grant_reference_not_authorizing",
            ModuleLoadGateRetainedCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_retained_selftest_case(
            "stale_dropped_retained_reference_event_id",
            "rejected",
            "retained_reference_stale_or_dropped_event_id",
            ModuleLoadGateRetainedCandidate {
                scope: "current_boot",
                retained: false,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_retained_selftest_case(
            "previous_boot_or_unretained_reference",
            "rejected",
            "retained_reference_previous_boot_or_unretained",
            ModuleLoadGateRetainedCandidate {
                scope: "previous_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_retained_selftest_case(
            "wrong_schema_or_variant_substitution",
            "rejected",
            "retained_reference_wrong_schema_or_variant",
            ModuleLoadGateRetainedCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: false,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_retained_selftest_case(
            "substituted_retained_reference_record",
            "rejected",
            "retained_reference_substituted_record",
            ModuleLoadGateRetainedCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(substituted_reference),
            },
        ),
        module_load_gate_retained_selftest_case(
            "mismatched_computed_grant_hash",
            "rejected",
            "retained_reference_hash_mismatch",
            ModuleLoadGateRetainedCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(mismatched_hash_reference),
                candidate_reference: Some(mismatched_hash_reference),
            },
        ),
    ]
}

pub(crate) fn module_load_gate_audit_rollback_selftest_cases(
) -> [ModuleLoadGateAuditRollbackSelfTestCase; MODULE_LOAD_GATE_AUDIT_ROLLBACK_SELFTEST_CASES] {
    let valid_requirements = module_load_gate_test_audit_rollback_candidate();
    let valid_audit_rollback_reference =
        module_load_gate_test_audit_rollback_reference(MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID);
    let substituted_audit_rollback_reference =
        module_load_gate_test_audit_rollback_reference_with_manifest(
            MODULE_GRANT_MISMATCH_MANIFEST_HASH,
            MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
        );
    let computed_grant_mismatch_reference =
        module_load_gate_test_audit_rollback_reference_with_override(
            MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
            Some([0x99; 32]),
            None,
            None,
        );
    let audit_hash_mismatch_reference =
        module_load_gate_test_audit_rollback_reference_with_override(
            MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
            None,
            None,
            Some([0xaa; 32]),
        );
    let rollback_hash_mismatch_reference =
        module_load_gate_test_audit_rollback_reference_with_override(
            MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID,
            None,
            Some([0xbb; 32]),
            None,
        );
    let service_slot_mismatch_reference =
        module_load_gate_test_audit_rollback_reference("ram_only:svc.test.other");
    [
        module_load_gate_audit_rollback_selftest_case(
            "missing_retained_audit_rollback_reference",
            "missing",
            "retained_audit_rollback_reference_missing",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: false,
                    schema_ok: true,
                    event_reference: None,
                    candidate_reference: None,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "stale_dropped_retained_audit_rollback_reference_event_id",
            "rejected",
            "retained_audit_rollback_reference_stale_or_dropped_event_id",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: false,
                    schema_ok: true,
                    event_reference: valid_audit_rollback_reference,
                    candidate_reference: valid_audit_rollback_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "previous_boot_or_unretained_audit_rollback_reference",
            "rejected",
            "retained_audit_rollback_reference_previous_boot_or_unretained",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "previous_boot",
                    retained: true,
                    schema_ok: true,
                    event_reference: valid_audit_rollback_reference,
                    candidate_reference: valid_audit_rollback_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "retained_audit_rollback_wrong_schema_or_variant",
            "rejected",
            "retained_audit_rollback_reference_wrong_schema_or_variant",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: true,
                    schema_ok: false,
                    event_reference: valid_audit_rollback_reference,
                    candidate_reference: valid_audit_rollback_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "substituted_retained_audit_rollback_reference",
            "rejected",
            "retained_audit_rollback_reference_substituted_record",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: true,
                    schema_ok: true,
                    event_reference: valid_audit_rollback_reference,
                    candidate_reference: substituted_audit_rollback_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "retained_audit_rollback_computed_grant_hash_mismatch",
            "rejected",
            "retained_audit_rollback_computed_grant_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: true,
                    schema_ok: true,
                    event_reference: computed_grant_mismatch_reference,
                    candidate_reference: computed_grant_mismatch_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "retained_audit_record_hash_mismatch",
            "rejected",
            "retained_audit_record_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: true,
                    schema_ok: true,
                    event_reference: audit_hash_mismatch_reference,
                    candidate_reference: audit_hash_mismatch_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "retained_rollback_plan_hash_mismatch",
            "rejected",
            "retained_rollback_plan_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: true,
                    schema_ok: true,
                    event_reference: rollback_hash_mismatch_reference,
                    candidate_reference: rollback_hash_mismatch_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "retained_audit_rollback_service_slot_mismatch",
            "rejected",
            "retained_audit_rollback_service_slot_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate {
                    scope: "current_boot",
                    retained: true,
                    schema_ok: true,
                    event_reference: service_slot_mismatch_reference,
                    candidate_reference: service_slot_mismatch_reference,
                },
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "missing_durable_audit_record",
            "missing",
            "durable_audit_write_missing",
            ModuleLoadGateAuditRollbackCandidate {
                durable_audit_record: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "missing_rollback_plan",
            "missing",
            "rollback_install_missing",
            ModuleLoadGateAuditRollbackCandidate {
                rollback_plan: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "durable_audit_record_schema_mismatch",
            "rejected",
            "durable_audit_record_schema_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_schema_ok: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "rollback_plan_schema_mismatch",
            "rejected",
            "rollback_plan_schema_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                rollback_schema_ok: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "valid_audit_and_rollback_still_denied",
            "validated_non_authorizing",
            "loader_and_service_slot_missing",
            valid_requirements,
        ),
        module_load_gate_audit_rollback_selftest_case(
            "audit_retained_grant_hash_mismatch",
            "rejected",
            "audit_retained_grant_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_binds_retained_grant: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "audit_manifest_hash_mismatch",
            "rejected",
            "audit_manifest_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_binds_manifest: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "audit_artifact_hash_mismatch",
            "rejected",
            "audit_artifact_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_binds_artifact: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "audit_vm_report_hash_mismatch",
            "rejected",
            "audit_vm_test_report_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_binds_vm_report: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "audit_local_attestation_hash_mismatch",
            "rejected",
            "audit_local_attestation_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_binds_local_attestation: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "local_approval_mismatch",
            "rejected",
            "local_approval_missing_or_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_binds_local_approval: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "audit_rollback_plan_hash_mismatch",
            "rejected",
            "audit_rollback_plan_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                audit_binds_rollback_plan: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "rollback_artifact_hash_mismatch",
            "rejected",
            "rollback_artifact_hash_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                rollback_binds_artifact: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "rollback_service_slot_mismatch",
            "rejected",
            "rollback_service_slot_mismatch",
            ModuleLoadGateAuditRollbackCandidate {
                rollback_binds_service_slot: false,
                ..valid_requirements
            },
        ),
    ]
}

pub(crate) fn module_load_gate_service_slot_selftest_cases(
) -> [ModuleLoadGateServiceSlotSelfTestCase; MODULE_LOAD_GATE_SERVICE_SLOT_SELFTEST_CASES] {
    let valid_gate = module_load_gate_test_service_slot_candidate();
    let valid_reservation = module_load_gate_test_service_slot_reservation();
    let substituted_reservation = module_load_gate_test_service_slot_reservation_with_override(
        Some([0x91; 32]),
        None,
        None,
        None,
        None,
        None,
    );
    let computed_grant_mismatch_reservation =
        module_load_gate_test_service_slot_reservation_with_override(
            Some([0x92; 32]),
            None,
            None,
            None,
            None,
            None,
        );
    let audit_hash_mismatch_reservation =
        module_load_gate_test_service_slot_reservation_with_override(
            None,
            Some([0x93; 32]),
            None,
            None,
            None,
            None,
        );
    let rollback_hash_mismatch_reservation =
        module_load_gate_test_service_slot_reservation_with_override(
            None,
            None,
            Some([0x94; 32]),
            None,
            None,
            None,
        );
    let inventory_hash_mismatch_reservation =
        module_load_gate_test_service_slot_reservation_with_override(
            None,
            None,
            None,
            Some([0x95; 32]),
            None,
            None,
        );
    let service_slot_mismatch_reservation =
        module_load_gate_test_service_slot_reservation_with_override(
            None,
            None,
            None,
            None,
            Some("ram_only:svc.test.other"),
            None,
        );
    let reservation_hash_mismatch = module_load_gate_test_service_slot_reservation_with_override(
        None,
        None,
        None,
        None,
        None,
        Some([0x96; 32]),
    );

    [
        module_load_gate_service_slot_selftest_case(
            "missing_retained_service_slot_reservation",
            "missing",
            "retained_service_slot_reservation_missing",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: None,
                    candidate_reservation: None,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "accepted_current_boot_reservation_still_denied",
            "retained_hash_reference_only_not_allocated",
            "retained_service_slot_reservation_not_allocated",
            valid_gate,
        ),
        module_load_gate_service_slot_selftest_case(
            "stale_dropped_retained_service_slot_reservation_event_id",
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    retained: false,
                    event_reservation: valid_reservation,
                    candidate_reservation: valid_reservation,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_wrong_schema_or_variant",
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    schema_ok: false,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "substituted_retained_service_slot_reservation",
            "rejected",
            "retained_service_slot_reservation_substituted_record",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: valid_reservation,
                    candidate_reservation: substituted_reservation,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_grant_wrong_schema_or_variant",
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    grant_event_schema_ok: false,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_audit_rollback_wrong_schema_or_variant",
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    audit_event_schema_ok: false,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_computed_grant_hash_mismatch",
            "rejected",
            "retained_service_slot_reservation_computed_grant_hash_mismatch",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: computed_grant_mismatch_reservation,
                    candidate_reservation: computed_grant_mismatch_reservation,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_audit_record_hash_mismatch",
            "rejected",
            "retained_service_slot_reservation_audit_record_hash_mismatch",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: audit_hash_mismatch_reservation,
                    candidate_reservation: audit_hash_mismatch_reservation,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_rollback_plan_hash_mismatch",
            "rejected",
            "retained_service_slot_reservation_rollback_plan_hash_mismatch",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: rollback_hash_mismatch_reservation,
                    candidate_reservation: rollback_hash_mismatch_reservation,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_inventory_hash_mismatch",
            "rejected",
            "retained_service_slot_reservation_pre_load_inventory_hash_mismatch",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: inventory_hash_mismatch_reservation,
                    candidate_reservation: inventory_hash_mismatch_reservation,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_service_slot_mismatch",
            "rejected",
            "retained_service_slot_reservation_service_slot_mismatch",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: service_slot_mismatch_reservation,
                    candidate_reservation: service_slot_mismatch_reservation,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
        module_load_gate_service_slot_selftest_case(
            "retained_service_slot_reservation_hash_mismatch",
            "rejected",
            "retained_service_slot_reservation_hash_mismatch",
            ModuleLoadGateServiceSlotCandidate {
                service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate {
                    event_reservation: reservation_hash_mismatch,
                    candidate_reservation: reservation_hash_mismatch,
                    ..valid_gate.service_slot_reservation
                },
                ..valid_gate
            },
        ),
    ]
}

pub(crate) fn module_load_gate_loader_runtime_selftest_cases(
) -> [ModuleLoadGateLoaderRuntimeSelfTestCase; MODULE_LOAD_GATE_LOADER_RUNTIME_SELFTEST_CASES] {
    let ready = module_load_gate_loader_runtime_ready_candidate();
    [
        module_load_gate_loader_runtime_selftest_case(
            "missing_manifest_reference",
            "denied_missing_retained_module_evidence",
            "retained_module_manifest_reference_missing",
            "missing",
            "blocked_by_service_slot_reservation",
            "blocked_by_retained_module_evidence",
            ModuleLoadGateLoaderRuntimeCandidate {
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
            },
        ),
        module_load_gate_loader_runtime_selftest_case(
            "rejected_artifact_reference",
            "denied_missing_retained_module_evidence",
            "retained_candidate_artifact_reference_hash_mismatch",
            "rejected",
            "blocked_by_service_slot_reservation",
            "blocked_by_retained_module_evidence",
            ModuleLoadGateLoaderRuntimeCandidate {
                artifact_reference_state: "rejected",
                artifact_reference_reason: "retained_candidate_artifact_reference_hash_mismatch",
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
                ..ready
            },
        ),
        module_load_gate_loader_runtime_selftest_case(
            "missing_service_slot_reservation",
            "denied_missing_retained_module_evidence",
            "ram_only_service_slot_unallocated",
            "missing",
            "blocked_by_service_slot_reservation",
            "blocked_by_retained_module_evidence",
            ModuleLoadGateLoaderRuntimeCandidate {
                service_slot_reservation_state: "missing",
                service_slot_reservation_reason: "ram_only_service_slot_unallocated",
                ..ready
            },
        ),
        module_load_gate_loader_runtime_selftest_case(
            "rejected_service_slot_reservation",
            "denied_missing_retained_module_evidence",
            "retained_service_slot_reservation_hash_mismatch",
            "rejected",
            "blocked_by_rejected_service_slot_reservation",
            "blocked_by_retained_module_evidence",
            ModuleLoadGateLoaderRuntimeCandidate {
                service_slot_reservation_state: "rejected",
                service_slot_reservation_reason: "retained_service_slot_reservation_hash_mismatch",
                ..ready
            },
        ),
        module_load_gate_loader_runtime_selftest_case(
            "all_retained_evidence_ready_allocator_runtime_missing",
            "denied_missing_service_slot_allocator_runtime",
            "service_slot_allocator_runtime_missing",
            "available",
            "missing_runtime",
            "blocked_by_service_slot_allocator_runtime",
            ready,
        ),
    ]
}

fn module_load_gate_retained_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoadGateRetainedCandidate,
) -> ModuleLoadGateRetainedSelfTestCase {
    let actual = evaluate_module_load_gate_retained_candidate(candidate);
    ModuleLoadGateRetainedSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_audit_rollback_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoadGateAuditRollbackCandidate,
) -> ModuleLoadGateAuditRollbackSelfTestCase {
    let actual = evaluate_module_load_gate_audit_rollback_candidate(candidate);
    ModuleLoadGateAuditRollbackSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_service_slot_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoadGateServiceSlotCandidate,
) -> ModuleLoadGateServiceSlotSelfTestCase {
    let actual = evaluate_module_load_gate_service_slot_candidate(candidate);
    let expected_hash_exposed = method_eq(
        expected_status,
        "retained_hash_reference_only_not_allocated",
    );
    ModuleLoadGateServiceSlotSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_service_slot_state: actual.service_slot_state,
        accepted_service_slot_reservation_hash: actual.accepted_service_slot_reservation_hash,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && actual.accepted_service_slot_reservation_hash == expected_hash_exposed
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_loader_runtime_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    expected_retained_module_evidence_state: &'static str,
    expected_service_slot_allocator_state: &'static str,
    expected_loader_runtime_state: &'static str,
    candidate: ModuleLoadGateLoaderRuntimeCandidate,
) -> ModuleLoadGateLoaderRuntimeSelfTestCase {
    let actual = evaluate_module_load_gate_loader_runtime_candidate(candidate);
    let expected_retained_module_evidence_reason = if method_eq(
        expected_status,
        "denied_missing_service_slot_allocator_runtime",
    ) {
        "retained_module_evidence_available"
    } else {
        expected_reason
    };
    let expected_service_slot_allocator_status =
        if method_eq(expected_service_slot_allocator_state, "missing_runtime") {
            "missing"
        } else {
            "blocked"
        };
    let expected_service_slot_allocator_reason =
        if method_eq(expected_service_slot_allocator_state, "missing_runtime") {
            "service_slot_allocator_runtime_missing"
        } else if method_eq(
            expected_service_slot_allocator_state,
            "blocked_by_rejected_service_slot_reservation",
        ) {
            expected_reason
        } else {
            "retained_service_slot_reservation_missing"
        };
    ModuleLoadGateLoaderRuntimeSelfTestCase {
        name,
        expected_status,
        expected_reason,
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
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
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

pub(crate) fn module_load_gate_manifest_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.load_gate_manifest_selftest")
        || method_head_eq(method, "module.manifest_gate_selftest")
}

pub(crate) fn module_load_gate_artifact_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.load_gate_artifact_selftest")
        || method_head_eq(method, "module.artifact_gate_selftest")
}

pub(crate) fn module_load_gate_vm_report_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.load_gate_vm_report_selftest")
        || method_head_eq(method, "module.vm_report_gate_selftest")
}

pub(crate) fn module_load_gate_attestation_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.load_gate_attestation_selftest")
        || method_head_eq(method, "module.attestation_gate_selftest")
}

pub(crate) fn module_load_gate_approval_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.load_gate_approval_selftest")
        || method_head_eq(method, "module.approval_gate_selftest")
}

pub(crate) fn module_load_gate_retained_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.load_gate_retained_selftest")
        || method_head_eq(method, "module.retained_grant_gate_selftest")
}

pub(crate) fn module_load_gate_audit_rollback_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.load_gate_audit_rollback_selftest")
        || method_head_eq(method, "module.audit_rollback_gate_selftest")
}

pub(crate) fn module_load_gate_service_slot_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.load_gate_service_slot_selftest")
        || method_head_eq(method, "module.service_slot_gate_selftest")
}

pub(crate) fn module_load_gate_loader_runtime_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.load_gate_loader_runtime_selftest")
        || method_head_eq(method, "module.loader_runtime_gate_selftest")
}
