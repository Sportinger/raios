use crate::{
    agent_protocol_module_load_gate_selftest::module_load_gate_test_reference,
    agent_protocol_module_load_gate_selftest_eval::{
        computed_module_manifest_reference_hash, evaluate_module_load_gate_artifact_candidate,
        evaluate_module_load_gate_manifest_candidate,
        evaluate_module_load_gate_vm_report_candidate,
    },
    agent_protocol_module_types::*,
    agent_protocol_support::method_eq,
    event_log, module_evidence,
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
