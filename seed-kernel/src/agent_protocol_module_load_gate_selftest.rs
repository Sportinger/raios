use crate::{
    agent_protocol_module_grant::{
        module_computed_grant_reference_hashes_consistent, module_computed_grant_reference_matches,
    },
    agent_protocol_module_types::*,
    agent_protocol_support::{method_eq, method_head_eq, parse_current_boot_event_id},
    event_log,
    module_evidence::{self, ModuleAuditRecordHashInput, ModuleServiceSlotReservationHashInput},
};
pub(crate) fn module_load_gate_manifest_selftest_cases(
) -> [ModuleLoadGateManifestSelfTestCase; MODULE_LOAD_GATE_MANIFEST_SELFTEST_CASES] {
    let valid_reference = module_load_gate_test_manifest_reference(MODULE_GRANT_TEST_MANIFEST_HASH);
    let substituted_reference =
        module_load_gate_test_manifest_reference(MODULE_GRANT_MISMATCH_MANIFEST_HASH);
    let mismatched_hash_reference = event_log::ModuleManifestReference {
        manifest_reference_hash: [0x99; 32],
        manifest_hash: MODULE_GRANT_TEST_MANIFEST_HASH,
    };

    [
        module_load_gate_manifest_selftest_case(
            "missing_retained_manifest_reference",
            "missing",
            "retained_module_manifest_reference_missing",
            ModuleLoadGateManifestReferenceCandidate {
                scope: "current_boot",
                retained: false,
                schema_ok: true,
                event_reference: None,
                candidate_reference: None,
            },
        ),
        module_load_gate_manifest_selftest_case(
            "accepted_current_boot_manifest_still_denied",
            "retained_hash_reference_only",
            "retained_module_manifest_reference_not_authorizing",
            ModuleLoadGateManifestReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_manifest_selftest_case(
            "stale_dropped_manifest_reference_event_id",
            "rejected",
            "retained_module_manifest_reference_stale_or_dropped_event_id",
            ModuleLoadGateManifestReferenceCandidate {
                scope: "current_boot",
                retained: false,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_manifest_selftest_case(
            "previous_boot_or_unretained_manifest_reference",
            "rejected",
            "retained_module_manifest_reference_previous_boot_or_unretained",
            ModuleLoadGateManifestReferenceCandidate {
                scope: "previous_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_manifest_selftest_case(
            "wrong_schema_or_variant",
            "rejected",
            "retained_module_manifest_reference_wrong_schema_or_variant",
            ModuleLoadGateManifestReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: false,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
            },
        ),
        module_load_gate_manifest_selftest_case(
            "substituted_manifest_reference_record",
            "rejected",
            "retained_module_manifest_reference_substituted_record",
            ModuleLoadGateManifestReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(substituted_reference),
            },
        ),
        module_load_gate_manifest_selftest_case(
            "manifest_reference_hash_mismatch",
            "rejected",
            "retained_module_manifest_reference_hash_mismatch",
            ModuleLoadGateManifestReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(mismatched_hash_reference),
                candidate_reference: Some(mismatched_hash_reference),
            },
        ),
    ]
}

fn module_load_gate_manifest_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoadGateManifestReferenceCandidate,
) -> ModuleLoadGateManifestSelfTestCase {
    let actual = evaluate_module_load_gate_manifest_candidate(candidate);
    ModuleLoadGateManifestSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_module_manifest_state: actual.module_manifest_state,
        accepted_manifest_hash: actual.accepted_manifest_hash,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_test_manifest_reference(
    manifest_hash: [u8; 32],
) -> event_log::ModuleManifestReference {
    event_log::ModuleManifestReference {
        manifest_reference_hash: computed_module_manifest_reference_hash(manifest_hash),
        manifest_hash,
    }
}

pub(crate) fn module_load_gate_artifact_selftest_cases(
) -> [ModuleLoadGateArtifactSelfTestCase; MODULE_LOAD_GATE_ARTIFACT_SELFTEST_CASES] {
    let valid_manifest_reference =
        module_load_gate_test_manifest_reference(MODULE_GRANT_TEST_MANIFEST_HASH);
    let valid_retained_reference = module_load_gate_test_reference(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let manifest_event_id = module_load_gate_test_event_id(26);
    let retained_event_id = module_load_gate_test_event_id(27);
    let valid_reference = module_load_gate_test_artifact_reference(
        manifest_event_id,
        retained_event_id,
        valid_manifest_reference,
        valid_retained_reference,
    );
    let substituted_reference = module_load_gate_test_artifact_reference(
        manifest_event_id,
        retained_event_id,
        valid_manifest_reference,
        module_load_gate_test_reference(
            MODULE_GRANT_TEST_MANIFEST_HASH,
            [0xbb; 32],
            MODULE_GRANT_TEST_VM_REPORT_HASH,
            MODULE_GRANT_TEST_ATTESTATION_HASH,
        ),
    );
    let mismatched_hash_reference = event_log::ModuleCandidateArtifactReference {
        artifact_reference_hash: [0x99; 32],
        ..valid_reference
    };

    [
        module_load_gate_artifact_selftest_case(
            "missing_retained_candidate_artifact_reference",
            "missing",
            "retained_candidate_artifact_reference_missing",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: false,
                schema_ok: true,
                event_reference: None,
                candidate_reference: None,
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "accepted_current_boot_artifact_still_denied",
            "retained_hash_reference_only",
            "retained_candidate_artifact_reference_not_authorizing",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "stale_dropped_retained_artifact_reference_event_id",
            "rejected",
            "retained_candidate_artifact_reference_stale_or_dropped_event_id",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: false,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "previous_boot_or_unretained_artifact_reference",
            "rejected",
            "retained_candidate_artifact_reference_previous_boot_or_unretained",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "previous_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "wrong_schema_or_variant",
            "rejected",
            "retained_candidate_artifact_reference_wrong_schema_or_variant",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: false,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "substituted_artifact_reference_record",
            "rejected",
            "retained_candidate_artifact_reference_substituted_record",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(substituted_reference),
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "artifact_reference_hash_mismatch",
            "rejected",
            "retained_candidate_artifact_reference_hash_mismatch",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(mismatched_hash_reference),
                candidate_reference: Some(mismatched_hash_reference),
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "manifest_reference_mismatch",
            "rejected",
            "retained_candidate_artifact_reference_manifest_reference_mismatch",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
                manifest_event_id: Some(module_load_gate_test_event_id(99)),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(retained_event_id),
                retained_reference: Some(valid_retained_reference),
            },
        ),
        module_load_gate_artifact_selftest_case(
            "computed_grant_reference_mismatch",
            "rejected",
            "retained_candidate_artifact_reference_computed_grant_reference_mismatch",
            ModuleLoadGateArtifactReferenceCandidate {
                scope: "current_boot",
                retained: true,
                schema_ok: true,
                event_reference: Some(valid_reference),
                candidate_reference: Some(valid_reference),
                manifest_event_id: Some(manifest_event_id),
                manifest_reference: Some(valid_manifest_reference),
                retained_event_id: Some(module_load_gate_test_event_id(98)),
                retained_reference: Some(valid_retained_reference),
            },
        ),
    ]
}

fn module_load_gate_artifact_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoadGateArtifactReferenceCandidate,
) -> ModuleLoadGateArtifactSelfTestCase {
    let actual = evaluate_module_load_gate_artifact_candidate(candidate);
    ModuleLoadGateArtifactSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_candidate_artifact_state: actual.candidate_artifact_state,
        accepted_artifact_hash: actual.accepted_artifact_hash,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_test_event_id(sequence: u64) -> event_log::EventId {
    let mut candidate = sequence;
    loop {
        if let Some(event_id) = event_log::EventId::from_sequence(candidate) {
            return event_id;
        }
        candidate = 1;
    }
}

fn module_load_gate_test_artifact_reference(
    retained_manifest_reference_event_id: event_log::EventId,
    retained_reference_event_id: event_log::EventId,
    manifest_reference: event_log::ModuleManifestReference,
    retained_reference: event_log::ModuleComputedGrantReference,
) -> event_log::ModuleCandidateArtifactReference {
    let artifact_reference_hash =
        module_evidence::computed_module_candidate_artifact_reference_hash_from_sequences(
            retained_manifest_reference_event_id.sequence(),
            retained_reference_event_id.sequence(),
            manifest_reference.manifest_reference_hash,
            manifest_reference.manifest_hash,
            retained_reference.computed_grant_hash,
            retained_reference.artifact_hash,
            retained_reference.vm_report_hash,
            retained_reference.local_attestation_hash,
        );
    event_log::ModuleCandidateArtifactReference {
        artifact_reference_hash,
        retained_manifest_reference_event_id,
        retained_reference_event_id,
        manifest_reference_hash: manifest_reference.manifest_reference_hash,
        manifest_hash: manifest_reference.manifest_hash,
        computed_grant_hash: retained_reference.computed_grant_hash,
        artifact_hash: retained_reference.artifact_hash,
        vm_report_hash: retained_reference.vm_report_hash,
        local_attestation_hash: retained_reference.local_attestation_hash,
    }
}

pub(crate) fn module_load_gate_vm_report_selftest_cases(
) -> [ModuleLoadGateVmReportSelfTestCase; MODULE_LOAD_GATE_VM_REPORT_SELFTEST_CASES] {
    let valid_manifest_reference =
        module_load_gate_test_manifest_reference(MODULE_GRANT_TEST_MANIFEST_HASH);
    let valid_retained_reference = module_load_gate_test_reference(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let manifest_event_id = module_load_gate_test_event_id(26);
    let artifact_event_id = module_load_gate_test_event_id(28);
    let retained_event_id = module_load_gate_test_event_id(27);
    let valid_artifact_reference = module_load_gate_test_artifact_reference(
        manifest_event_id,
        retained_event_id,
        valid_manifest_reference,
        valid_retained_reference,
    );
    let valid_reference = module_load_gate_test_vm_report_reference(
        manifest_event_id,
        artifact_event_id,
        retained_event_id,
        valid_manifest_reference,
        valid_artifact_reference,
        valid_retained_reference,
        None,
    );
    let substituted_reference = module_load_gate_test_vm_report_reference(
        manifest_event_id,
        artifact_event_id,
        retained_event_id,
        valid_manifest_reference,
        event_log::ModuleCandidateArtifactReference {
            artifact_hash: [0xbb; 32],
            ..valid_artifact_reference
        },
        valid_retained_reference,
        None,
    );
    let mismatched_hash_reference = event_log::ModuleVmTestReportReference {
        report_reference_hash: [0x99; 32],
        ..valid_reference
    };
    let mismatched_report_hash_reference = module_load_gate_test_vm_report_reference(
        manifest_event_id,
        artifact_event_id,
        retained_event_id,
        valid_manifest_reference,
        valid_artifact_reference,
        valid_retained_reference,
        Some([0xbb; 32]),
    );

    [
        module_load_gate_vm_report_selftest_case(
            "missing_retained_vm_test_report_reference",
            "missing",
            "retained_vm_test_report_reference_missing",
            module_load_gate_vm_report_candidate(
                false,
                true,
                "current_boot",
                None,
                None,
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "accepted_current_boot_report_still_denied",
            "retained_hash_reference_only",
            "retained_vm_test_report_reference_not_authorizing",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "current_boot",
                Some(valid_reference),
                Some(valid_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "stale_dropped_retained_vm_test_report_reference_event_id",
            "rejected",
            "retained_vm_test_report_reference_stale_or_dropped_event_id",
            module_load_gate_vm_report_candidate(
                false,
                true,
                "current_boot",
                Some(valid_reference),
                Some(valid_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "previous_boot_or_unretained_vm_test_report_reference",
            "rejected",
            "retained_vm_test_report_reference_previous_boot_or_unretained",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "previous_boot",
                Some(valid_reference),
                Some(valid_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "wrong_schema_or_variant",
            "rejected",
            "retained_vm_test_report_reference_wrong_schema_or_variant",
            module_load_gate_vm_report_candidate(
                true,
                false,
                "current_boot",
                Some(valid_reference),
                Some(valid_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "substituted_vm_test_report_reference_record",
            "rejected",
            "retained_vm_test_report_reference_substituted_record",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "current_boot",
                Some(valid_reference),
                Some(substituted_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "vm_test_report_reference_hash_mismatch",
            "rejected",
            "retained_vm_test_report_reference_hash_mismatch",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "current_boot",
                Some(mismatched_hash_reference),
                Some(mismatched_hash_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "manifest_reference_mismatch",
            "rejected",
            "retained_vm_test_report_reference_manifest_reference_mismatch",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "current_boot",
                Some(valid_reference),
                Some(valid_reference),
                module_load_gate_test_event_id(99),
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "artifact_reference_mismatch",
            "rejected",
            "retained_vm_test_report_reference_artifact_reference_mismatch",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "current_boot",
                Some(valid_reference),
                Some(valid_reference),
                manifest_event_id,
                valid_manifest_reference,
                module_load_gate_test_event_id(98),
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "computed_grant_reference_mismatch",
            "rejected",
            "retained_vm_test_report_reference_computed_grant_reference_mismatch",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "current_boot",
                Some(valid_reference),
                Some(valid_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                module_load_gate_test_event_id(97),
                valid_retained_reference,
            ),
        ),
        module_load_gate_vm_report_selftest_case(
            "vm_test_report_hash_mismatch",
            "rejected",
            "retained_vm_test_report_hash_mismatch",
            module_load_gate_vm_report_candidate(
                true,
                true,
                "current_boot",
                Some(mismatched_report_hash_reference),
                Some(mismatched_report_hash_reference),
                manifest_event_id,
                valid_manifest_reference,
                artifact_event_id,
                valid_artifact_reference,
                retained_event_id,
                valid_retained_reference,
            ),
        ),
    ]
}

fn module_load_gate_vm_report_candidate(
    retained: bool,
    schema_ok: bool,
    scope: &'static str,
    event_reference: Option<event_log::ModuleVmTestReportReference>,
    candidate_reference: Option<event_log::ModuleVmTestReportReference>,
    manifest_event_id: event_log::EventId,
    manifest_reference: event_log::ModuleManifestReference,
    artifact_event_id: event_log::EventId,
    artifact_reference: event_log::ModuleCandidateArtifactReference,
    retained_event_id: event_log::EventId,
    retained_reference: event_log::ModuleComputedGrantReference,
) -> ModuleLoadGateVmReportReferenceCandidate {
    ModuleLoadGateVmReportReferenceCandidate {
        scope,
        retained,
        schema_ok,
        event_reference,
        candidate_reference,
        manifest_event_id: Some(manifest_event_id),
        manifest_reference: Some(manifest_reference),
        artifact_event_id: Some(artifact_event_id),
        artifact_reference: Some(artifact_reference),
        retained_event_id: Some(retained_event_id),
        retained_reference: Some(retained_reference),
    }
}

fn module_load_gate_vm_report_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoadGateVmReportReferenceCandidate,
) -> ModuleLoadGateVmReportSelfTestCase {
    let actual = evaluate_module_load_gate_vm_report_candidate(candidate);
    ModuleLoadGateVmReportSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_vm_test_report_state: actual.vm_test_report_state,
        accepted_vm_report_hash: actual.accepted_vm_report_hash,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_test_vm_report_reference(
    retained_manifest_reference_event_id: event_log::EventId,
    retained_artifact_reference_event_id: event_log::EventId,
    retained_reference_event_id: event_log::EventId,
    manifest_reference: event_log::ModuleManifestReference,
    artifact_reference: event_log::ModuleCandidateArtifactReference,
    retained_reference: event_log::ModuleComputedGrantReference,
    vm_report_hash_override: Option<[u8; 32]>,
) -> event_log::ModuleVmTestReportReference {
    let vm_report_hash = vm_report_hash_override.unwrap_or(retained_reference.vm_report_hash);
    let report_reference_hash =
        module_evidence::computed_module_vm_test_report_reference_hash_from_sequences(
            retained_manifest_reference_event_id.sequence(),
            retained_artifact_reference_event_id.sequence(),
            retained_reference_event_id.sequence(),
            manifest_reference.manifest_reference_hash,
            artifact_reference.artifact_reference_hash,
            manifest_reference.manifest_hash,
            artifact_reference.artifact_hash,
            retained_reference.computed_grant_hash,
            vm_report_hash,
            retained_reference.local_attestation_hash,
        );
    event_log::ModuleVmTestReportReference {
        report_reference_hash,
        retained_manifest_reference_event_id,
        retained_artifact_reference_event_id,
        retained_reference_event_id,
        manifest_reference_hash: manifest_reference.manifest_reference_hash,
        artifact_reference_hash: artifact_reference.artifact_reference_hash,
        manifest_hash: manifest_reference.manifest_hash,
        artifact_hash: artifact_reference.artifact_hash,
        computed_grant_hash: retained_reference.computed_grant_hash,
        vm_report_hash,
        local_attestation_hash: retained_reference.local_attestation_hash,
    }
}

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
            "durable_audit_record_missing",
            ModuleLoadGateAuditRollbackCandidate {
                durable_audit_record: false,
                ..valid_requirements
            },
        ),
        module_load_gate_audit_rollback_selftest_case(
            "missing_rollback_plan",
            "missing",
            "rollback_plan_missing",
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

fn module_load_gate_test_reference(
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

fn evaluate_module_load_gate_manifest_candidate(
    candidate: ModuleLoadGateManifestReferenceCandidate,
) -> ModuleLoadGateManifestEvaluation {
    if candidate.candidate_reference.is_none() {
        return module_load_gate_manifest_check(
            "missing",
            "retained_module_manifest_reference_missing",
        );
    }
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_manifest_check(
            "rejected",
            "retained_module_manifest_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_manifest_check(
            "rejected",
            "retained_module_manifest_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_manifest_check(
            "rejected",
            "retained_module_manifest_reference_wrong_schema_or_variant",
        );
    }
    if candidate.event_reference != candidate.candidate_reference {
        return module_load_gate_manifest_check(
            "rejected",
            "retained_module_manifest_reference_substituted_record",
        );
    }
    let Some(reference) = candidate.candidate_reference else {
        return module_load_gate_manifest_check(
            "missing",
            "retained_module_manifest_reference_missing",
        );
    };
    if reference.manifest_reference_hash
        != computed_module_manifest_reference_hash(reference.manifest_hash)
    {
        return module_load_gate_manifest_check(
            "rejected",
            "retained_module_manifest_reference_hash_mismatch",
        );
    }
    module_load_gate_manifest_check(
        "retained_hash_reference_only",
        "retained_module_manifest_reference_not_authorizing",
    )
}

fn module_load_gate_manifest_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateManifestEvaluation {
    let accepted = method_eq(status, "retained_hash_reference_only");
    ModuleLoadGateManifestEvaluation {
        status,
        reason,
        module_manifest_state: if accepted {
            "retained_hash_reference_only"
        } else if method_eq(status, "rejected") {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_manifest_hash: accepted,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_load_gate_artifact_candidate(
    candidate: ModuleLoadGateArtifactReferenceCandidate,
) -> ModuleLoadGateArtifactEvaluation {
    let Some(candidate_reference) = candidate.candidate_reference else {
        return module_load_gate_artifact_check(
            "missing",
            "retained_candidate_artifact_reference_missing",
        );
    };
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_wrong_schema_or_variant",
        );
    }
    if candidate.event_reference != candidate.candidate_reference {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_substituted_record",
        );
    }
    if candidate_reference.artifact_reference_hash
        != module_evidence::computed_module_candidate_artifact_reference_hash_from_sequences(
            candidate_reference
                .retained_manifest_reference_event_id
                .sequence(),
            candidate_reference.retained_reference_event_id.sequence(),
            candidate_reference.manifest_reference_hash,
            candidate_reference.manifest_hash,
            candidate_reference.computed_grant_hash,
            candidate_reference.artifact_hash,
            candidate_reference.vm_report_hash,
            candidate_reference.local_attestation_hash,
        )
    {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_hash_mismatch",
        );
    }

    let (Some(manifest_event_id), Some(manifest_reference)) =
        (candidate.manifest_event_id, candidate.manifest_reference)
    else {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_manifest_reference_mismatch",
        );
    };
    if candidate_reference.retained_manifest_reference_event_id != manifest_event_id
        || candidate_reference.manifest_reference_hash != manifest_reference.manifest_reference_hash
        || candidate_reference.manifest_hash != manifest_reference.manifest_hash
    {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_manifest_reference_mismatch",
        );
    }

    let (Some(retained_event_id), Some(retained_reference)) =
        (candidate.retained_event_id, candidate.retained_reference)
    else {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_computed_grant_reference_mismatch",
        );
    };
    if candidate_reference.retained_reference_event_id != retained_event_id
        || candidate_reference.computed_grant_hash != retained_reference.computed_grant_hash
        || candidate_reference.manifest_hash != retained_reference.manifest_hash
        || candidate_reference.vm_report_hash != retained_reference.vm_report_hash
        || candidate_reference.local_attestation_hash != retained_reference.local_attestation_hash
    {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_reference_computed_grant_reference_mismatch",
        );
    }
    if candidate_reference.artifact_hash != retained_reference.artifact_hash {
        return module_load_gate_artifact_check(
            "rejected",
            "retained_candidate_artifact_hash_mismatch",
        );
    }

    module_load_gate_artifact_check(
        "retained_hash_reference_only",
        "retained_candidate_artifact_reference_not_authorizing",
    )
}

fn evaluate_module_load_gate_vm_report_candidate(
    candidate: ModuleLoadGateVmReportReferenceCandidate,
) -> ModuleLoadGateVmReportEvaluation {
    let Some(candidate_reference) = candidate.candidate_reference else {
        return module_load_gate_vm_report_check(
            "missing",
            "retained_vm_test_report_reference_missing",
        );
    };
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_wrong_schema_or_variant",
        );
    }
    if candidate.event_reference != candidate.candidate_reference {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_substituted_record",
        );
    }
    if candidate_reference.report_reference_hash
        != module_evidence::computed_module_vm_test_report_reference_hash_from_sequences(
            candidate_reference
                .retained_manifest_reference_event_id
                .sequence(),
            candidate_reference
                .retained_artifact_reference_event_id
                .sequence(),
            candidate_reference.retained_reference_event_id.sequence(),
            candidate_reference.manifest_reference_hash,
            candidate_reference.artifact_reference_hash,
            candidate_reference.manifest_hash,
            candidate_reference.artifact_hash,
            candidate_reference.computed_grant_hash,
            candidate_reference.vm_report_hash,
            candidate_reference.local_attestation_hash,
        )
    {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_hash_mismatch",
        );
    }

    let (Some(manifest_event_id), Some(manifest_reference)) =
        (candidate.manifest_event_id, candidate.manifest_reference)
    else {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_manifest_reference_mismatch",
        );
    };
    if candidate_reference.retained_manifest_reference_event_id != manifest_event_id
        || candidate_reference.manifest_reference_hash != manifest_reference.manifest_reference_hash
        || candidate_reference.manifest_hash != manifest_reference.manifest_hash
    {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_manifest_reference_mismatch",
        );
    }

    let (Some(artifact_event_id), Some(artifact_reference)) =
        (candidate.artifact_event_id, candidate.artifact_reference)
    else {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_artifact_reference_mismatch",
        );
    };
    if candidate_reference.retained_artifact_reference_event_id != artifact_event_id
        || candidate_reference.artifact_reference_hash != artifact_reference.artifact_reference_hash
        || candidate_reference.manifest_reference_hash != artifact_reference.manifest_reference_hash
        || candidate_reference.manifest_hash != artifact_reference.manifest_hash
        || candidate_reference.artifact_hash != artifact_reference.artifact_hash
        || candidate_reference.local_attestation_hash != artifact_reference.local_attestation_hash
    {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_artifact_reference_mismatch",
        );
    }
    if candidate_reference.vm_report_hash != artifact_reference.vm_report_hash {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_hash_mismatch",
        );
    }

    let (Some(retained_event_id), Some(retained_reference)) =
        (candidate.retained_event_id, candidate.retained_reference)
    else {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_computed_grant_reference_mismatch",
        );
    };
    if candidate_reference.retained_reference_event_id != retained_event_id
        || candidate_reference.computed_grant_hash != retained_reference.computed_grant_hash
        || candidate_reference.manifest_hash != retained_reference.manifest_hash
        || candidate_reference.artifact_hash != retained_reference.artifact_hash
        || candidate_reference.local_attestation_hash != retained_reference.local_attestation_hash
    {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_reference_computed_grant_reference_mismatch",
        );
    }
    if candidate_reference.vm_report_hash != retained_reference.vm_report_hash {
        return module_load_gate_vm_report_check(
            "rejected",
            "retained_vm_test_report_hash_mismatch",
        );
    }

    module_load_gate_vm_report_check(
        "retained_hash_reference_only",
        "retained_vm_test_report_reference_not_authorizing",
    )
}

fn module_load_gate_artifact_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateArtifactEvaluation {
    let accepted = method_eq(status, "retained_hash_reference_only");
    ModuleLoadGateArtifactEvaluation {
        status,
        reason,
        candidate_artifact_state: if accepted {
            "retained_hash_reference_only"
        } else if method_eq(status, "rejected") {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_artifact_hash: accepted,
        can_load: false,
        load_attempted: false,
    }
}

fn module_load_gate_vm_report_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateVmReportEvaluation {
    let accepted = method_eq(status, "retained_hash_reference_only");
    ModuleLoadGateVmReportEvaluation {
        status,
        reason,
        vm_test_report_state: if accepted {
            "retained_hash_reference_only"
        } else if method_eq(status, "rejected") {
            "rejected_retained_reference"
        } else {
            "missing"
        },
        accepted_vm_report_hash: accepted,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_load_gate_retained_candidate(
    candidate: ModuleLoadGateRetainedCandidate,
) -> ModuleLoadGateRetainedCheck {
    let Some(candidate_reference) = candidate.candidate_reference else {
        return module_load_gate_retained_check(
            "missing",
            "computed_capability_grant_reference_missing",
        );
    };
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_retained_check(
            "rejected",
            "retained_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_retained_check(
            "rejected",
            "retained_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_retained_check(
            "rejected",
            "retained_reference_wrong_schema_or_variant",
        );
    }
    let Some(event_reference) = candidate.event_reference else {
        return module_load_gate_retained_check(
            "rejected",
            "retained_reference_stale_or_dropped_event_id",
        );
    };
    if !module_computed_grant_reference_matches(event_reference, candidate_reference) {
        return module_load_gate_retained_check(
            "rejected",
            "retained_reference_substituted_record",
        );
    }
    if !module_computed_grant_reference_hashes_consistent(candidate_reference) {
        return module_load_gate_retained_check("rejected", "retained_reference_hash_mismatch");
    }
    module_load_gate_retained_check(
        "retained_hash_reference_only",
        "retained_computed_grant_reference_not_authorizing",
    )
}

fn module_load_gate_retained_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateRetainedCheck {
    ModuleLoadGateRetainedCheck {
        status,
        reason,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_load_gate_audit_rollback_candidate(
    candidate: ModuleLoadGateAuditRollbackCandidate,
) -> ModuleLoadGateAuditRollbackEvaluation {
    if !candidate.retained_reference {
        return module_load_gate_audit_rollback_check(
            "missing",
            "retained_computed_grant_reference_missing",
        );
    }
    let retained_audit_rollback_check =
        evaluate_module_load_gate_audit_rollback_reference_candidate(
            candidate.retained_audit_rollback_reference,
        );
    if !method_eq(
        retained_audit_rollback_check.status,
        "retained_hash_reference_only",
    ) {
        return module_load_gate_audit_rollback_check(
            retained_audit_rollback_check.status,
            retained_audit_rollback_check.reason,
        );
    }
    if !candidate.durable_audit_record {
        return module_load_gate_audit_rollback_check("missing", "durable_audit_record_missing");
    }
    if !candidate.rollback_plan {
        return module_load_gate_audit_rollback_check("missing", "rollback_plan_missing");
    }
    if !candidate.audit_schema_ok {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "durable_audit_record_schema_mismatch",
        );
    }
    if !candidate.rollback_schema_ok {
        return module_load_gate_audit_rollback_check("rejected", "rollback_plan_schema_mismatch");
    }
    if !candidate.audit_binds_retained_grant {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "audit_retained_grant_hash_mismatch",
        );
    }
    if !candidate.audit_binds_manifest {
        return module_load_gate_audit_rollback_check("rejected", "audit_manifest_hash_mismatch");
    }
    if !candidate.audit_binds_artifact {
        return module_load_gate_audit_rollback_check("rejected", "audit_artifact_hash_mismatch");
    }
    if !candidate.audit_binds_vm_report {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "audit_vm_test_report_hash_mismatch",
        );
    }
    if !candidate.audit_binds_local_attestation {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "audit_local_attestation_hash_mismatch",
        );
    }
    if !candidate.audit_binds_local_approval {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "local_approval_missing_or_mismatch",
        );
    }
    if !candidate.audit_binds_rollback_plan {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "audit_rollback_plan_hash_mismatch",
        );
    }
    if !candidate.rollback_binds_artifact {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "rollback_artifact_hash_mismatch",
        );
    }
    if !candidate.rollback_binds_service_slot {
        return module_load_gate_audit_rollback_check("rejected", "rollback_service_slot_mismatch");
    }
    if !candidate.ram_only_service_slot_allocated && !candidate.loader_available {
        return module_load_gate_audit_rollback_check(
            "validated_non_authorizing",
            "loader_and_service_slot_missing",
        );
    }
    if !candidate.ram_only_service_slot_allocated {
        return module_load_gate_audit_rollback_check(
            "rejected",
            "ram_only_service_slot_unallocated",
        );
    }
    if !candidate.loader_available {
        return module_load_gate_audit_rollback_check(
            "validated_non_authorizing",
            "module_loader_unimplemented",
        );
    }
    module_load_gate_audit_rollback_check("rejected", "positive_loader_path_unimplemented")
}

fn evaluate_module_load_gate_audit_rollback_reference_candidate(
    candidate: ModuleLoadGateAuditRollbackReferenceCandidate,
) -> ModuleLoadGateRetainedCheck {
    let Some(candidate_reference) = candidate.candidate_reference else {
        return module_load_gate_retained_check(
            "missing",
            "retained_audit_rollback_reference_missing",
        );
    };
    if !method_eq(candidate.scope, "current_boot") {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_reference_previous_boot_or_unretained",
        );
    }
    if !candidate.retained {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_reference_stale_or_dropped_event_id",
        );
    }
    if !candidate.schema_ok {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_reference_wrong_schema_or_variant",
        );
    }
    let Some(event_reference) = candidate.event_reference else {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_reference_stale_or_dropped_event_id",
        );
    };
    if !module_audit_rollback_event_reference_matches(event_reference, candidate_reference) {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_reference_substituted_record",
        );
    }
    if candidate_reference.ram_only_service_slot_id.as_str()
        != MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID
    {
        return module_load_gate_retained_check(
            "rejected",
            "retained_audit_rollback_service_slot_mismatch",
        );
    }
    if let Some(reason) = module_audit_rollback_reference_hash_mismatch(candidate_reference) {
        return module_load_gate_retained_check("rejected", reason);
    }
    module_load_gate_retained_check(
        "retained_hash_reference_only",
        "retained_audit_rollback_reference_not_authorizing",
    )
}

fn evaluate_module_load_gate_service_slot_candidate(
    candidate: ModuleLoadGateServiceSlotCandidate,
) -> ModuleLoadGateServiceSlotEvaluation {
    let Some(reservation) = candidate.service_slot_reservation.candidate_reservation else {
        return module_load_gate_service_slot_check(
            "missing",
            "retained_service_slot_reservation_missing",
        );
    };
    let Some(retained_reference) = candidate.retained_reference else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_computed_grant_reference_missing",
        );
    };
    let Some(audit_rollback_reference) = candidate.audit_rollback_reference else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_audit_rollback_reference_missing",
        );
    };
    if !candidate.audit_rollback_valid {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_audit_rollback_reference_not_valid_for_service_slot",
        );
    }

    let Some(retained_reference_event_id) =
        parse_current_boot_event_id(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID)
    else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_grant_reference_mismatch",
        );
    };
    let Some(audit_rollback_event_id) =
        parse_current_boot_event_id(MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID)
    else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_audit_rollback_reference_mismatch",
        );
    };

    if reservation.retained_reference_event_id != retained_reference_event_id {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_grant_reference_mismatch",
        );
    }
    if reservation.retained_audit_rollback_reference_event_id != audit_rollback_event_id {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_audit_rollback_reference_mismatch",
        );
    }

    let service_slot_candidate = candidate.service_slot_reservation;
    if !method_eq(service_slot_candidate.scope, "current_boot") || !service_slot_candidate.retained
    {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
        );
    }
    if !service_slot_candidate.schema_ok {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
        );
    }
    let Some(event_reservation) = service_slot_candidate.event_reservation else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
        );
    };
    if !module_service_slot_reservation_matches(event_reservation, reservation) {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_substituted_record",
        );
    }

    if !service_slot_candidate.grant_event_schema_ok {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
        );
    }
    let Some(grant_event_reference) = service_slot_candidate.grant_event_reference else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
        );
    };
    if !module_computed_grant_reference_matches(retained_reference, grant_event_reference) {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_substituted_record",
        );
    }

    if !service_slot_candidate.audit_event_schema_ok {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_wrong_schema_or_variant",
        );
    }
    let Some(audit_event_reference) = service_slot_candidate.audit_event_reference else {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_stale_or_dropped_event_id",
        );
    };
    if !module_audit_rollback_event_reference_matches(
        audit_rollback_reference,
        audit_event_reference,
    ) {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_substituted_record",
        );
    }

    if reservation.computed_grant_hash != retained_reference.computed_grant_hash
        || reservation.computed_grant_hash != audit_rollback_reference.computed_grant_hash
    {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_computed_grant_hash_mismatch",
        );
    }
    if reservation.audit_record_hash != audit_rollback_reference.audit_record_hash {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_audit_record_hash_mismatch",
        );
    }
    if reservation.rollback_plan_hash != audit_rollback_reference.rollback_plan_hash {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_rollback_plan_hash_mismatch",
        );
    }
    if reservation.pre_load_service_inventory_hash
        != audit_rollback_reference.pre_load_service_inventory_hash
    {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_pre_load_inventory_hash_mismatch",
        );
    }
    if reservation.ram_only_service_slot_id.as_str()
        != audit_rollback_reference.ram_only_service_slot_id.as_str()
        || !module_evidence::ram_only_service_slot_id_valid(
            reservation.ram_only_service_slot_id.as_str(),
        )
    {
        return module_load_gate_service_slot_check(
            "rejected",
            "retained_service_slot_reservation_service_slot_mismatch",
        );
    }
    if let Some(reason) = module_service_slot_reservation_hash_mismatch(reservation) {
        return module_load_gate_service_slot_check("rejected", reason);
    }

    module_load_gate_service_slot_check(
        "retained_hash_reference_only_not_allocated",
        "retained_service_slot_reservation_not_allocated",
    )
}

fn module_audit_rollback_event_reference_matches(
    event_reference: event_log::ModuleAuditRollbackReference,
    candidate_reference: event_log::ModuleAuditRollbackReference,
) -> bool {
    event_reference.audit_record_hash == candidate_reference.audit_record_hash
        && event_reference.rollback_plan_hash == candidate_reference.rollback_plan_hash
        && event_reference.computed_grant_hash == candidate_reference.computed_grant_hash
        && event_reference.manifest_hash == candidate_reference.manifest_hash
        && event_reference.artifact_hash == candidate_reference.artifact_hash
        && event_reference.vm_report_hash == candidate_reference.vm_report_hash
        && event_reference.local_attestation_hash == candidate_reference.local_attestation_hash
        && event_reference.local_approval_hash == candidate_reference.local_approval_hash
        && event_reference.pre_load_service_inventory_hash
            == candidate_reference.pre_load_service_inventory_hash
        && event_reference.cleanup_actions_hash == candidate_reference.cleanup_actions_hash
        && event_reference.denial_event_id == candidate_reference.denial_event_id
        && event_reference.retained_reference_event_id
            == candidate_reference.retained_reference_event_id
        && event_reference.ram_only_service_slot_id.as_str()
            == candidate_reference.ram_only_service_slot_id.as_str()
}

fn module_audit_rollback_reference_hash_mismatch(
    reference: event_log::ModuleAuditRollbackReference,
) -> Option<&'static str> {
    if parse_current_boot_event_id(MODULE_AUDIT_TEST_DENIAL_EVENT_ID)
        != Some(reference.denial_event_id)
        || parse_current_boot_event_id(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID)
            != Some(reference.retained_reference_event_id)
    {
        return Some("retained_audit_rollback_reference_substituted_record");
    }

    let expected_computed_grant_hash = computed_module_grant_hash(
        reference.manifest_hash,
        reference.artifact_hash,
        reference.vm_report_hash,
        reference.local_attestation_hash,
    );
    if reference.computed_grant_hash != expected_computed_grant_hash {
        return Some("retained_audit_rollback_computed_grant_hash_mismatch");
    }

    let expected_rollback_plan_hash = computed_module_rollback_plan_hash(
        reference.artifact_hash,
        reference.pre_load_service_inventory_hash,
        reference.ram_only_service_slot_id.as_str(),
        reference.cleanup_actions_hash,
    );
    if reference.rollback_plan_hash != expected_rollback_plan_hash {
        return Some("retained_rollback_plan_hash_mismatch");
    }

    let expected_audit_record_hash =
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
    if reference.audit_record_hash != expected_audit_record_hash {
        return Some("retained_audit_record_hash_mismatch");
    }

    None
}

fn module_service_slot_reservation_matches(
    left: event_log::ModuleServiceSlotReservation,
    right: event_log::ModuleServiceSlotReservation,
) -> bool {
    left.reservation_hash == right.reservation_hash
        && left.retained_reference_event_id == right.retained_reference_event_id
        && left.retained_audit_rollback_reference_event_id
            == right.retained_audit_rollback_reference_event_id
        && left.computed_grant_hash == right.computed_grant_hash
        && left.audit_record_hash == right.audit_record_hash
        && left.rollback_plan_hash == right.rollback_plan_hash
        && left.pre_load_service_inventory_hash == right.pre_load_service_inventory_hash
        && left.ram_only_service_slot_id.as_str() == right.ram_only_service_slot_id.as_str()
}

fn module_service_slot_reservation_hash_mismatch(
    reservation: event_log::ModuleServiceSlotReservation,
) -> Option<&'static str> {
    if parse_current_boot_event_id(MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID)
        != Some(reservation.retained_reference_event_id)
        || parse_current_boot_event_id(MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID)
            != Some(reservation.retained_audit_rollback_reference_event_id)
    {
        return Some("retained_service_slot_reservation_hash_mismatch");
    }

    let expected_reservation_hash =
        computed_module_service_slot_reservation_hash(ModuleServiceSlotReservationHashInput {
            retained_reference_event_id: MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID,
            retained_audit_rollback_reference_event_id:
                MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID,
            computed_grant_hash: reservation.computed_grant_hash,
            audit_record_hash: reservation.audit_record_hash,
            rollback_plan_hash: reservation.rollback_plan_hash,
            pre_load_service_inventory_hash: reservation.pre_load_service_inventory_hash,
            ram_only_service_slot_id: reservation.ram_only_service_slot_id.as_str(),
        });
    if reservation.reservation_hash != expected_reservation_hash {
        return Some("retained_service_slot_reservation_hash_mismatch");
    }

    None
}

fn module_load_gate_service_slot_check(
    status: &'static str,
    reason: &'static str,
) -> ModuleLoadGateServiceSlotEvaluation {
    let accepted = method_eq(status, "retained_hash_reference_only_not_allocated");
    let service_slot_state = if accepted {
        "retained_hash_reference_only_not_allocated"
    } else if method_eq(status, "rejected") {
        "rejected_retained_reference"
    } else {
        "unallocated"
    };
    ModuleLoadGateServiceSlotEvaluation {
        status,
        reason,
        service_slot_state,
        accepted_service_slot_reservation_hash: accepted,
        can_load: false,
        load_attempted: false,
    }
}

fn module_load_gate_audit_rollback_check(
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

fn computed_module_manifest_reference_hash(manifest_hash: [u8; 32]) -> [u8; 32] {
    module_evidence::computed_module_manifest_reference_hash(manifest_hash)
}

fn computed_module_grant_hash(
    manifest_hash: [u8; 32],
    artifact_hash: [u8; 32],
    vm_report_hash: [u8; 32],
    local_attestation_hash: [u8; 32],
) -> [u8; 32] {
    module_evidence::computed_module_grant_hash(
        manifest_hash,
        artifact_hash,
        vm_report_hash,
        local_attestation_hash,
    )
}

fn computed_module_rollback_plan_hash(
    artifact_hash: [u8; 32],
    pre_load_service_inventory_hash: [u8; 32],
    ram_only_service_slot_id: &str,
    cleanup_actions_hash: [u8; 32],
) -> [u8; 32] {
    module_evidence::computed_module_rollback_plan_hash(
        artifact_hash,
        pre_load_service_inventory_hash,
        ram_only_service_slot_id,
        cleanup_actions_hash,
    )
}

fn computed_module_audit_record_hash(input: ModuleAuditRecordHashInput<'_>) -> [u8; 32] {
    module_evidence::computed_module_audit_record_hash(input)
}

fn computed_module_service_slot_reservation_hash(
    input: ModuleServiceSlotReservationHashInput<'_>,
) -> [u8; 32] {
    module_evidence::computed_module_service_slot_reservation_hash(input)
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
