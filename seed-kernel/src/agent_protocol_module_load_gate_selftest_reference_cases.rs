use crate::{
    agent_protocol_module_load_gate::{
        ModuleLoadGateArtifactEvaluation, ModuleLoadGateLocalApprovalEvaluation,
        ModuleLoadGateLocalAttestationEvaluation, ModuleLoadGateManifestEvaluation,
        ModuleLoadGateVmReportEvaluation,
    },
    agent_protocol_module_load_gate_selftest::module_load_gate_test_reference,
    agent_protocol_module_load_gate_selftest_eval::{
        computed_module_manifest_reference_hash, evaluate_module_load_gate_artifact_candidate,
        evaluate_module_load_gate_local_approval_candidate,
        evaluate_module_load_gate_local_attestation_candidate,
        evaluate_module_load_gate_manifest_candidate,
        evaluate_module_load_gate_vm_report_candidate,
    },
    agent_protocol_module_types::*,
    agent_protocol_support::{method_eq, run_selftest_cases_with, CaseSpec},
    event_log, module_evidence,
};

#[derive(Clone, Copy)]
enum LoadGateReferenceMutation {
    None,
    Missing,
    Stale,
    PreviousBoot,
    WrongSchema,
    Substituted,
    ReferenceHashMismatch,
    ManifestReferenceMismatch,
    ArtifactReferenceMismatch,
    VmReportReferenceMismatch,
    ComputedGrantReferenceMismatch,
    VmReportHashMismatch,
    LocalAttestationReferenceMismatch,
}

const fn reference_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    mutation: LoadGateReferenceMutation,
) -> CaseSpec<LoadGateReferenceMutation> {
    CaseSpec {
        name,
        expected_status,
        expected_reason,
        mutation,
        require_live_retained: false,
    }
}

#[derive(Clone, Copy)]
struct ModuleLoadGateReferenceChain {
    manifest_event_id: event_log::EventId,
    artifact_event_id: event_log::EventId,
    vm_report_event_id: event_log::EventId,
    attestation_event_id: event_log::EventId,
    retained_event_id: event_log::EventId,
    manifest_reference: event_log::ModuleManifestReference,
    retained_reference: event_log::ModuleComputedGrantReference,
    artifact_reference: event_log::ModuleCandidateArtifactReference,
    vm_report_reference: event_log::ModuleVmTestReportReference,
    attestation_reference: event_log::ModuleLocalAttestationReference,
    approval_reference: event_log::ModuleLocalApprovalReference,
}

pub(crate) fn module_load_gate_manifest_selftest_cases(
) -> [ModuleLoadGateManifestSelfTestCase; MODULE_LOAD_GATE_MANIFEST_SELFTEST_CASES] {
    run_selftest_cases_with(
        module_load_gate_manifest_valid_candidate(),
        &MANIFEST_CASES,
        apply_manifest_reference_case,
        evaluate_manifest_reference_case,
        module_load_gate_manifest_selftest_case_from_spec,
    )
}

const MANIFEST_CASES: [CaseSpec<LoadGateReferenceMutation>;
    MODULE_LOAD_GATE_MANIFEST_SELFTEST_CASES] = [
    reference_case(
        "missing_retained_manifest_reference",
        "missing",
        "retained_module_manifest_reference_missing",
        LoadGateReferenceMutation::Missing,
    ),
    reference_case(
        "accepted_current_boot_manifest_still_denied",
        "retained_hash_reference_only",
        "retained_module_manifest_reference_not_authorizing",
        LoadGateReferenceMutation::None,
    ),
    reference_case(
        "stale_dropped_manifest_reference_event_id",
        "rejected",
        "retained_module_manifest_reference_stale_or_dropped_event_id",
        LoadGateReferenceMutation::Stale,
    ),
    reference_case(
        "previous_boot_or_unretained_manifest_reference",
        "rejected",
        "retained_module_manifest_reference_previous_boot_or_unretained",
        LoadGateReferenceMutation::PreviousBoot,
    ),
    reference_case(
        "wrong_schema_or_variant",
        "rejected",
        "retained_module_manifest_reference_wrong_schema_or_variant",
        LoadGateReferenceMutation::WrongSchema,
    ),
    reference_case(
        "substituted_manifest_reference_record",
        "rejected",
        "retained_module_manifest_reference_substituted_record",
        LoadGateReferenceMutation::Substituted,
    ),
    reference_case(
        "manifest_reference_hash_mismatch",
        "rejected",
        "retained_module_manifest_reference_hash_mismatch",
        LoadGateReferenceMutation::ReferenceHashMismatch,
    ),
];

fn module_load_gate_manifest_valid_candidate() -> ModuleLoadGateManifestReferenceCandidate {
    let reference = module_load_gate_test_manifest_reference(MODULE_GRANT_TEST_MANIFEST_HASH);
    ModuleLoadGateManifestReferenceCandidate {
        scope: "current_boot",
        retained: true,
        schema_ok: true,
        event_reference: Some(reference),
        candidate_reference: Some(reference),
    }
}

fn apply_manifest_reference_case(
    candidate: &mut ModuleLoadGateManifestReferenceCandidate,
    mutation: LoadGateReferenceMutation,
) {
    let valid_reference = module_load_gate_test_manifest_reference(MODULE_GRANT_TEST_MANIFEST_HASH);
    match mutation {
        LoadGateReferenceMutation::None => {}
        LoadGateReferenceMutation::Missing => {
            candidate.retained = false;
            candidate.event_reference = None;
            candidate.candidate_reference = None;
        }
        LoadGateReferenceMutation::Stale => candidate.retained = false,
        LoadGateReferenceMutation::PreviousBoot => candidate.scope = "previous_boot",
        LoadGateReferenceMutation::WrongSchema => candidate.schema_ok = false,
        LoadGateReferenceMutation::Substituted => {
            candidate.candidate_reference = Some(module_load_gate_test_manifest_reference(
                MODULE_GRANT_MISMATCH_MANIFEST_HASH,
            ));
        }
        LoadGateReferenceMutation::ReferenceHashMismatch => {
            let mismatched = event_log::ModuleManifestReference {
                manifest_reference_hash: [0x99; 32],
                manifest_hash: MODULE_GRANT_TEST_MANIFEST_HASH,
            };
            candidate.event_reference = Some(mismatched);
            candidate.candidate_reference = Some(mismatched);
        }
        _ => {
            candidate.event_reference = Some(valid_reference);
            candidate.candidate_reference = Some(valid_reference);
        }
    }
}

fn evaluate_manifest_reference_case(
    candidate: ModuleLoadGateManifestReferenceCandidate,
    _require_live_retained: bool,
) -> ModuleLoadGateManifestEvaluation {
    evaluate_module_load_gate_manifest_candidate(candidate)
}

fn module_load_gate_manifest_selftest_case_from_spec(
    spec: &CaseSpec<LoadGateReferenceMutation>,
    actual: ModuleLoadGateManifestEvaluation,
) -> ModuleLoadGateManifestSelfTestCase {
    ModuleLoadGateManifestSelfTestCase {
        name: spec.name,
        expected_status: spec.expected_status,
        expected_reason: spec.expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_module_manifest_state: actual.module_manifest_state,
        accepted_manifest_hash: actual.accepted_manifest_hash,
        passed: method_eq(actual.status, spec.expected_status)
            && method_eq(actual.reason, spec.expected_reason)
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
    run_selftest_cases_with(
        module_load_gate_artifact_valid_candidate(),
        &ARTIFACT_CASES,
        apply_artifact_reference_case,
        evaluate_artifact_reference_case,
        module_load_gate_artifact_selftest_case_from_spec,
    )
}

const ARTIFACT_CASES: [CaseSpec<LoadGateReferenceMutation>;
    MODULE_LOAD_GATE_ARTIFACT_SELFTEST_CASES] = [
    reference_case(
        "missing_retained_candidate_artifact_reference",
        "missing",
        "retained_candidate_artifact_reference_missing",
        LoadGateReferenceMutation::Missing,
    ),
    reference_case(
        "accepted_current_boot_artifact_still_denied",
        "retained_hash_reference_only",
        "retained_candidate_artifact_reference_not_authorizing",
        LoadGateReferenceMutation::None,
    ),
    reference_case(
        "stale_dropped_retained_artifact_reference_event_id",
        "rejected",
        "retained_candidate_artifact_reference_stale_or_dropped_event_id",
        LoadGateReferenceMutation::Stale,
    ),
    reference_case(
        "previous_boot_or_unretained_artifact_reference",
        "rejected",
        "retained_candidate_artifact_reference_previous_boot_or_unretained",
        LoadGateReferenceMutation::PreviousBoot,
    ),
    reference_case(
        "wrong_schema_or_variant",
        "rejected",
        "retained_candidate_artifact_reference_wrong_schema_or_variant",
        LoadGateReferenceMutation::WrongSchema,
    ),
    reference_case(
        "substituted_artifact_reference_record",
        "rejected",
        "retained_candidate_artifact_reference_substituted_record",
        LoadGateReferenceMutation::Substituted,
    ),
    reference_case(
        "artifact_reference_hash_mismatch",
        "rejected",
        "retained_candidate_artifact_reference_hash_mismatch",
        LoadGateReferenceMutation::ReferenceHashMismatch,
    ),
    reference_case(
        "manifest_reference_mismatch",
        "rejected",
        "retained_candidate_artifact_reference_manifest_reference_mismatch",
        LoadGateReferenceMutation::ManifestReferenceMismatch,
    ),
    reference_case(
        "computed_grant_reference_mismatch",
        "rejected",
        "retained_candidate_artifact_reference_computed_grant_reference_mismatch",
        LoadGateReferenceMutation::ComputedGrantReferenceMismatch,
    ),
];

fn module_load_gate_artifact_valid_candidate() -> ModuleLoadGateArtifactReferenceCandidate {
    let chain = module_load_gate_reference_chain();
    ModuleLoadGateArtifactReferenceCandidate {
        scope: "current_boot",
        retained: true,
        schema_ok: true,
        event_reference: Some(chain.artifact_reference),
        candidate_reference: Some(chain.artifact_reference),
        manifest_event_id: Some(chain.manifest_event_id),
        manifest_reference: Some(chain.manifest_reference),
        retained_event_id: Some(chain.retained_event_id),
        retained_reference: Some(chain.retained_reference),
    }
}

fn apply_artifact_reference_case(
    candidate: &mut ModuleLoadGateArtifactReferenceCandidate,
    mutation: LoadGateReferenceMutation,
) {
    let chain = module_load_gate_reference_chain();
    match mutation {
        LoadGateReferenceMutation::None => {}
        LoadGateReferenceMutation::Missing => {
            candidate.retained = false;
            candidate.event_reference = None;
            candidate.candidate_reference = None;
        }
        LoadGateReferenceMutation::Stale => candidate.retained = false,
        LoadGateReferenceMutation::PreviousBoot => candidate.scope = "previous_boot",
        LoadGateReferenceMutation::WrongSchema => candidate.schema_ok = false,
        LoadGateReferenceMutation::Substituted => {
            candidate.candidate_reference = Some(module_load_gate_test_artifact_reference(
                chain.manifest_event_id,
                chain.retained_event_id,
                chain.manifest_reference,
                module_load_gate_test_reference(
                    MODULE_GRANT_TEST_MANIFEST_HASH,
                    [0xbb; 32],
                    MODULE_GRANT_TEST_VM_REPORT_HASH,
                    MODULE_GRANT_TEST_ATTESTATION_HASH,
                ),
            ));
        }
        LoadGateReferenceMutation::ReferenceHashMismatch => {
            let mismatched = event_log::ModuleCandidateArtifactReference {
                artifact_reference_hash: [0x99; 32],
                ..chain.artifact_reference
            };
            candidate.event_reference = Some(mismatched);
            candidate.candidate_reference = Some(mismatched);
        }
        LoadGateReferenceMutation::ManifestReferenceMismatch => {
            candidate.manifest_event_id = Some(module_load_gate_test_event_id(99));
        }
        LoadGateReferenceMutation::ComputedGrantReferenceMismatch => {
            candidate.retained_event_id = Some(module_load_gate_test_event_id(98));
        }
        _ => {}
    }
}

fn evaluate_artifact_reference_case(
    candidate: ModuleLoadGateArtifactReferenceCandidate,
    _require_live_retained: bool,
) -> ModuleLoadGateArtifactEvaluation {
    evaluate_module_load_gate_artifact_candidate(candidate)
}

fn module_load_gate_artifact_selftest_case_from_spec(
    spec: &CaseSpec<LoadGateReferenceMutation>,
    actual: ModuleLoadGateArtifactEvaluation,
) -> ModuleLoadGateArtifactSelfTestCase {
    ModuleLoadGateArtifactSelfTestCase {
        name: spec.name,
        expected_status: spec.expected_status,
        expected_reason: spec.expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_candidate_artifact_state: actual.candidate_artifact_state,
        accepted_artifact_hash: actual.accepted_artifact_hash,
        passed: method_eq(actual.status, spec.expected_status)
            && method_eq(actual.reason, spec.expected_reason)
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

fn module_load_gate_reference_chain() -> ModuleLoadGateReferenceChain {
    let manifest_reference =
        module_load_gate_test_manifest_reference(MODULE_GRANT_TEST_MANIFEST_HASH);
    let retained_reference = module_load_gate_test_reference(
        MODULE_GRANT_TEST_MANIFEST_HASH,
        MODULE_GRANT_TEST_ARTIFACT_HASH,
        MODULE_GRANT_TEST_VM_REPORT_HASH,
        MODULE_GRANT_TEST_ATTESTATION_HASH,
    );
    let manifest_event_id = module_load_gate_test_event_id(26);
    let artifact_event_id = module_load_gate_test_event_id(28);
    let vm_report_event_id = module_load_gate_test_event_id(29);
    let attestation_event_id = module_load_gate_test_event_id(30);
    let retained_event_id = module_load_gate_test_event_id(27);
    let artifact_reference = module_load_gate_test_artifact_reference(
        manifest_event_id,
        retained_event_id,
        manifest_reference,
        retained_reference,
    );
    let vm_report_reference = module_load_gate_test_vm_report_reference(
        manifest_event_id,
        artifact_event_id,
        retained_event_id,
        manifest_reference,
        artifact_reference,
        retained_reference,
        None,
    );
    let attestation_reference = module_load_gate_test_local_attestation_reference(
        manifest_event_id,
        artifact_event_id,
        vm_report_event_id,
        retained_event_id,
        manifest_reference,
        artifact_reference,
        vm_report_reference,
        retained_reference,
    );
    let approval_reference = module_load_gate_test_local_approval_reference(
        manifest_event_id,
        artifact_event_id,
        vm_report_event_id,
        attestation_event_id,
        retained_event_id,
        manifest_reference,
        artifact_reference,
        vm_report_reference,
        attestation_reference,
        retained_reference,
    );
    ModuleLoadGateReferenceChain {
        manifest_event_id,
        artifact_event_id,
        vm_report_event_id,
        attestation_event_id,
        retained_event_id,
        manifest_reference,
        retained_reference,
        artifact_reference,
        vm_report_reference,
        attestation_reference,
        approval_reference,
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
    run_selftest_cases_with(
        module_load_gate_vm_report_valid_candidate(),
        &VM_REPORT_CASES,
        apply_vm_report_reference_case,
        evaluate_vm_report_reference_case,
        module_load_gate_vm_report_selftest_case_from_spec,
    )
}

const VM_REPORT_CASES: [CaseSpec<LoadGateReferenceMutation>;
    MODULE_LOAD_GATE_VM_REPORT_SELFTEST_CASES] = [
    reference_case(
        "missing_retained_vm_test_report_reference",
        "missing",
        "retained_vm_test_report_reference_missing",
        LoadGateReferenceMutation::Missing,
    ),
    reference_case(
        "accepted_current_boot_report_still_denied",
        "retained_hash_reference_only",
        "retained_vm_test_report_reference_not_authorizing",
        LoadGateReferenceMutation::None,
    ),
    reference_case(
        "stale_dropped_retained_vm_test_report_reference_event_id",
        "rejected",
        "retained_vm_test_report_reference_stale_or_dropped_event_id",
        LoadGateReferenceMutation::Stale,
    ),
    reference_case(
        "previous_boot_or_unretained_vm_test_report_reference",
        "rejected",
        "retained_vm_test_report_reference_previous_boot_or_unretained",
        LoadGateReferenceMutation::PreviousBoot,
    ),
    reference_case(
        "wrong_schema_or_variant",
        "rejected",
        "retained_vm_test_report_reference_wrong_schema_or_variant",
        LoadGateReferenceMutation::WrongSchema,
    ),
    reference_case(
        "substituted_vm_test_report_reference_record",
        "rejected",
        "retained_vm_test_report_reference_substituted_record",
        LoadGateReferenceMutation::Substituted,
    ),
    reference_case(
        "vm_test_report_reference_hash_mismatch",
        "rejected",
        "retained_vm_test_report_reference_hash_mismatch",
        LoadGateReferenceMutation::ReferenceHashMismatch,
    ),
    reference_case(
        "manifest_reference_mismatch",
        "rejected",
        "retained_vm_test_report_reference_manifest_reference_mismatch",
        LoadGateReferenceMutation::ManifestReferenceMismatch,
    ),
    reference_case(
        "artifact_reference_mismatch",
        "rejected",
        "retained_vm_test_report_reference_artifact_reference_mismatch",
        LoadGateReferenceMutation::ArtifactReferenceMismatch,
    ),
    reference_case(
        "computed_grant_reference_mismatch",
        "rejected",
        "retained_vm_test_report_reference_computed_grant_reference_mismatch",
        LoadGateReferenceMutation::ComputedGrantReferenceMismatch,
    ),
    reference_case(
        "vm_test_report_hash_mismatch",
        "rejected",
        "retained_vm_test_report_hash_mismatch",
        LoadGateReferenceMutation::VmReportHashMismatch,
    ),
];

fn module_load_gate_vm_report_valid_candidate() -> ModuleLoadGateVmReportReferenceCandidate {
    let chain = module_load_gate_reference_chain();
    module_load_gate_vm_report_candidate(
        true,
        true,
        "current_boot",
        Some(chain.vm_report_reference),
        Some(chain.vm_report_reference),
        chain.manifest_event_id,
        chain.manifest_reference,
        chain.artifact_event_id,
        chain.artifact_reference,
        chain.retained_event_id,
        chain.retained_reference,
    )
}

fn apply_vm_report_reference_case(
    candidate: &mut ModuleLoadGateVmReportReferenceCandidate,
    mutation: LoadGateReferenceMutation,
) {
    let chain = module_load_gate_reference_chain();
    match mutation {
        LoadGateReferenceMutation::None => {}
        LoadGateReferenceMutation::Missing => {
            candidate.retained = false;
            candidate.event_reference = None;
            candidate.candidate_reference = None;
        }
        LoadGateReferenceMutation::Stale => candidate.retained = false,
        LoadGateReferenceMutation::PreviousBoot => candidate.scope = "previous_boot",
        LoadGateReferenceMutation::WrongSchema => candidate.schema_ok = false,
        LoadGateReferenceMutation::Substituted => {
            candidate.candidate_reference = Some(module_load_gate_test_vm_report_reference(
                chain.manifest_event_id,
                chain.artifact_event_id,
                chain.retained_event_id,
                chain.manifest_reference,
                event_log::ModuleCandidateArtifactReference {
                    artifact_hash: [0xbb; 32],
                    ..chain.artifact_reference
                },
                chain.retained_reference,
                None,
            ));
        }
        LoadGateReferenceMutation::ReferenceHashMismatch => {
            let mismatched = event_log::ModuleVmTestReportReference {
                report_reference_hash: [0x99; 32],
                ..chain.vm_report_reference
            };
            candidate.event_reference = Some(mismatched);
            candidate.candidate_reference = Some(mismatched);
        }
        LoadGateReferenceMutation::ManifestReferenceMismatch => {
            candidate.manifest_event_id = Some(module_load_gate_test_event_id(99));
        }
        LoadGateReferenceMutation::ArtifactReferenceMismatch => {
            candidate.artifact_event_id = Some(module_load_gate_test_event_id(98));
        }
        LoadGateReferenceMutation::ComputedGrantReferenceMismatch => {
            candidate.retained_event_id = Some(module_load_gate_test_event_id(97));
        }
        LoadGateReferenceMutation::VmReportHashMismatch => {
            let mismatched = module_load_gate_test_vm_report_reference(
                chain.manifest_event_id,
                chain.artifact_event_id,
                chain.retained_event_id,
                chain.manifest_reference,
                chain.artifact_reference,
                chain.retained_reference,
                Some([0xbb; 32]),
            );
            candidate.event_reference = Some(mismatched);
            candidate.candidate_reference = Some(mismatched);
        }
        _ => {}
    }
}

fn evaluate_vm_report_reference_case(
    candidate: ModuleLoadGateVmReportReferenceCandidate,
    _require_live_retained: bool,
) -> ModuleLoadGateVmReportEvaluation {
    evaluate_module_load_gate_vm_report_candidate(candidate)
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

fn module_load_gate_vm_report_selftest_case_from_spec(
    spec: &CaseSpec<LoadGateReferenceMutation>,
    actual: ModuleLoadGateVmReportEvaluation,
) -> ModuleLoadGateVmReportSelfTestCase {
    ModuleLoadGateVmReportSelfTestCase {
        name: spec.name,
        expected_status: spec.expected_status,
        expected_reason: spec.expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_vm_test_report_state: actual.vm_test_report_state,
        accepted_vm_report_hash: actual.accepted_vm_report_hash,
        passed: method_eq(actual.status, spec.expected_status)
            && method_eq(actual.reason, spec.expected_reason)
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

pub(crate) fn module_load_gate_attestation_selftest_cases(
) -> [ModuleLoadGateLocalAttestationSelfTestCase; MODULE_LOAD_GATE_LOCAL_ATTESTATION_SELFTEST_CASES]
{
    run_selftest_cases_with(
        module_load_gate_attestation_valid_candidate(),
        &ATTESTATION_CASES,
        apply_attestation_reference_case,
        evaluate_attestation_reference_case,
        module_load_gate_attestation_selftest_case_from_spec,
    )
}

const ATTESTATION_CASES: [CaseSpec<LoadGateReferenceMutation>;
    MODULE_LOAD_GATE_LOCAL_ATTESTATION_SELFTEST_CASES] = [
    reference_case(
        "missing_retained_local_attestation_reference",
        "missing",
        "retained_local_attestation_reference_missing",
        LoadGateReferenceMutation::Missing,
    ),
    reference_case(
        "accepted_current_boot_attestation_still_denied",
        "retained_hash_reference_only",
        "retained_local_attestation_reference_not_authorizing",
        LoadGateReferenceMutation::None,
    ),
    reference_case(
        "stale_dropped_retained_local_attestation_reference_event_id",
        "rejected",
        "retained_local_attestation_reference_stale_or_dropped_event_id",
        LoadGateReferenceMutation::Stale,
    ),
    reference_case(
        "previous_boot_or_unretained_local_attestation_reference",
        "rejected",
        "retained_local_attestation_reference_previous_boot_or_unretained",
        LoadGateReferenceMutation::PreviousBoot,
    ),
    reference_case(
        "wrong_schema_or_variant",
        "rejected",
        "retained_local_attestation_reference_wrong_schema_or_variant",
        LoadGateReferenceMutation::WrongSchema,
    ),
    reference_case(
        "substituted_local_attestation_reference_record",
        "rejected",
        "retained_local_attestation_reference_substituted_record",
        LoadGateReferenceMutation::Substituted,
    ),
    reference_case(
        "local_attestation_reference_hash_mismatch",
        "rejected",
        "retained_local_attestation_reference_hash_mismatch",
        LoadGateReferenceMutation::ReferenceHashMismatch,
    ),
    reference_case(
        "manifest_reference_mismatch",
        "rejected",
        "retained_local_attestation_reference_manifest_reference_mismatch",
        LoadGateReferenceMutation::ManifestReferenceMismatch,
    ),
    reference_case(
        "artifact_reference_mismatch",
        "rejected",
        "retained_local_attestation_reference_artifact_reference_mismatch",
        LoadGateReferenceMutation::ArtifactReferenceMismatch,
    ),
    reference_case(
        "vm_report_reference_mismatch",
        "rejected",
        "retained_local_attestation_reference_vm_report_reference_mismatch",
        LoadGateReferenceMutation::VmReportReferenceMismatch,
    ),
    reference_case(
        "computed_grant_reference_mismatch",
        "rejected",
        "retained_local_attestation_reference_computed_grant_reference_mismatch",
        LoadGateReferenceMutation::ComputedGrantReferenceMismatch,
    ),
];

fn module_load_gate_attestation_valid_candidate() -> ModuleLoadGateLocalAttestationReferenceCandidate
{
    let chain = module_load_gate_reference_chain();
    module_load_gate_attestation_candidate(
        true,
        true,
        "current_boot",
        Some(chain.attestation_reference),
        Some(chain.attestation_reference),
        chain.manifest_event_id,
        chain.manifest_reference,
        chain.artifact_event_id,
        chain.artifact_reference,
        chain.vm_report_event_id,
        chain.vm_report_reference,
        chain.retained_event_id,
        chain.retained_reference,
    )
}

fn apply_attestation_reference_case(
    candidate: &mut ModuleLoadGateLocalAttestationReferenceCandidate,
    mutation: LoadGateReferenceMutation,
) {
    let chain = module_load_gate_reference_chain();
    match mutation {
        LoadGateReferenceMutation::None => {}
        LoadGateReferenceMutation::Missing => {
            candidate.retained = false;
            candidate.event_reference = None;
            candidate.candidate_reference = None;
        }
        LoadGateReferenceMutation::Stale => candidate.retained = false,
        LoadGateReferenceMutation::PreviousBoot => candidate.scope = "previous_boot",
        LoadGateReferenceMutation::WrongSchema => candidate.schema_ok = false,
        LoadGateReferenceMutation::Substituted => {
            candidate.candidate_reference =
                Some(module_load_gate_test_local_attestation_reference(
                    chain.manifest_event_id,
                    chain.artifact_event_id,
                    chain.vm_report_event_id,
                    chain.retained_event_id,
                    chain.manifest_reference,
                    event_log::ModuleCandidateArtifactReference {
                        artifact_hash: [0xbb; 32],
                        ..chain.artifact_reference
                    },
                    chain.vm_report_reference,
                    chain.retained_reference,
                ));
        }
        LoadGateReferenceMutation::ReferenceHashMismatch => {
            let mismatched = event_log::ModuleLocalAttestationReference {
                attestation_reference_hash: [0x99; 32],
                signature_verified: false,
                ..chain.attestation_reference
            };
            candidate.event_reference = Some(mismatched);
            candidate.candidate_reference = Some(mismatched);
        }
        LoadGateReferenceMutation::ManifestReferenceMismatch => {
            candidate.manifest_event_id = Some(module_load_gate_test_event_id(99));
        }
        LoadGateReferenceMutation::ArtifactReferenceMismatch => {
            candidate.artifact_event_id = Some(module_load_gate_test_event_id(98));
        }
        LoadGateReferenceMutation::VmReportReferenceMismatch => {
            candidate.vm_report_event_id = Some(module_load_gate_test_event_id(97));
        }
        LoadGateReferenceMutation::ComputedGrantReferenceMismatch => {
            candidate.retained_event_id = Some(module_load_gate_test_event_id(96));
        }
        _ => {}
    }
}

fn evaluate_attestation_reference_case(
    candidate: ModuleLoadGateLocalAttestationReferenceCandidate,
    _require_live_retained: bool,
) -> ModuleLoadGateLocalAttestationEvaluation {
    evaluate_module_load_gate_local_attestation_candidate(candidate)
}

fn module_load_gate_attestation_candidate(
    retained: bool,
    schema_ok: bool,
    scope: &'static str,
    event_reference: Option<event_log::ModuleLocalAttestationReference>,
    candidate_reference: Option<event_log::ModuleLocalAttestationReference>,
    manifest_event_id: event_log::EventId,
    manifest_reference: event_log::ModuleManifestReference,
    artifact_event_id: event_log::EventId,
    artifact_reference: event_log::ModuleCandidateArtifactReference,
    vm_report_event_id: event_log::EventId,
    vm_report_reference: event_log::ModuleVmTestReportReference,
    retained_event_id: event_log::EventId,
    retained_reference: event_log::ModuleComputedGrantReference,
) -> ModuleLoadGateLocalAttestationReferenceCandidate {
    ModuleLoadGateLocalAttestationReferenceCandidate {
        scope,
        retained,
        schema_ok,
        event_reference,
        candidate_reference,
        manifest_event_id: Some(manifest_event_id),
        manifest_reference: Some(manifest_reference),
        artifact_event_id: Some(artifact_event_id),
        artifact_reference: Some(artifact_reference),
        vm_report_event_id: Some(vm_report_event_id),
        vm_report_reference: Some(vm_report_reference),
        retained_event_id: Some(retained_event_id),
        retained_reference: Some(retained_reference),
    }
}

fn module_load_gate_attestation_selftest_case_from_spec(
    spec: &CaseSpec<LoadGateReferenceMutation>,
    actual: ModuleLoadGateLocalAttestationEvaluation,
) -> ModuleLoadGateLocalAttestationSelfTestCase {
    ModuleLoadGateLocalAttestationSelfTestCase {
        name: spec.name,
        expected_status: spec.expected_status,
        expected_reason: spec.expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_local_attestation_state: actual.local_attestation_state,
        accepted_local_attestation_hash: actual.accepted_local_attestation_hash,
        passed: method_eq(actual.status, spec.expected_status)
            && method_eq(actual.reason, spec.expected_reason)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_test_local_attestation_reference(
    retained_manifest_reference_event_id: event_log::EventId,
    retained_artifact_reference_event_id: event_log::EventId,
    retained_vm_report_reference_event_id: event_log::EventId,
    retained_reference_event_id: event_log::EventId,
    manifest_reference: event_log::ModuleManifestReference,
    artifact_reference: event_log::ModuleCandidateArtifactReference,
    vm_report_reference: event_log::ModuleVmTestReportReference,
    retained_reference: event_log::ModuleComputedGrantReference,
) -> event_log::ModuleLocalAttestationReference {
    let attestation_reference_hash =
        module_evidence::computed_module_local_attestation_reference_hash_from_sequences(
            retained_manifest_reference_event_id.sequence(),
            retained_artifact_reference_event_id.sequence(),
            retained_vm_report_reference_event_id.sequence(),
            retained_reference_event_id.sequence(),
            manifest_reference.manifest_reference_hash,
            artifact_reference.artifact_reference_hash,
            vm_report_reference.report_reference_hash,
            manifest_reference.manifest_hash,
            artifact_reference.artifact_hash,
            retained_reference.computed_grant_hash,
            vm_report_reference.vm_report_hash,
            retained_reference.local_attestation_hash,
        );
    event_log::ModuleLocalAttestationReference {
        attestation_reference_hash,
        retained_manifest_reference_event_id,
        retained_artifact_reference_event_id,
        retained_vm_report_reference_event_id,
        retained_reference_event_id,
        manifest_reference_hash: manifest_reference.manifest_reference_hash,
        artifact_reference_hash: artifact_reference.artifact_reference_hash,
        vm_report_reference_hash: vm_report_reference.report_reference_hash,
        manifest_hash: manifest_reference.manifest_hash,
        artifact_hash: artifact_reference.artifact_hash,
        computed_grant_hash: retained_reference.computed_grant_hash,
        vm_report_hash: vm_report_reference.vm_report_hash,
        local_attestation_hash: retained_reference.local_attestation_hash,
        signature_verified: false,
    }
}

pub(crate) fn module_load_gate_approval_selftest_cases(
) -> [ModuleLoadGateLocalApprovalSelfTestCase; MODULE_LOAD_GATE_LOCAL_APPROVAL_SELFTEST_CASES] {
    run_selftest_cases_with(
        module_load_gate_approval_valid_candidate(),
        &APPROVAL_CASES,
        apply_approval_reference_case,
        evaluate_approval_reference_case,
        module_load_gate_approval_selftest_case_from_spec,
    )
}

const APPROVAL_CASES: [CaseSpec<LoadGateReferenceMutation>;
    MODULE_LOAD_GATE_LOCAL_APPROVAL_SELFTEST_CASES] = [
    reference_case(
        "missing_retained_local_approval_reference",
        "missing",
        "retained_local_approval_reference_missing",
        LoadGateReferenceMutation::Missing,
    ),
    reference_case(
        "accepted_current_boot_approval_still_denied",
        "retained_hash_reference_only",
        "retained_local_approval_reference_not_authorizing",
        LoadGateReferenceMutation::None,
    ),
    reference_case(
        "stale_dropped_retained_local_approval_reference_event_id",
        "rejected",
        "retained_local_approval_reference_stale_or_dropped_event_id",
        LoadGateReferenceMutation::Stale,
    ),
    reference_case(
        "previous_boot_or_unretained_local_approval_reference",
        "rejected",
        "retained_local_approval_reference_previous_boot_or_unretained",
        LoadGateReferenceMutation::PreviousBoot,
    ),
    reference_case(
        "wrong_schema_or_variant",
        "rejected",
        "retained_local_approval_reference_wrong_schema_or_variant",
        LoadGateReferenceMutation::WrongSchema,
    ),
    reference_case(
        "substituted_local_approval_reference_record",
        "rejected",
        "retained_local_approval_reference_substituted_record",
        LoadGateReferenceMutation::Substituted,
    ),
    reference_case(
        "local_approval_reference_hash_mismatch",
        "rejected",
        "retained_local_approval_reference_hash_mismatch",
        LoadGateReferenceMutation::ReferenceHashMismatch,
    ),
    reference_case(
        "manifest_reference_mismatch",
        "rejected",
        "retained_local_approval_reference_manifest_reference_mismatch",
        LoadGateReferenceMutation::ManifestReferenceMismatch,
    ),
    reference_case(
        "artifact_reference_mismatch",
        "rejected",
        "retained_local_approval_reference_artifact_reference_mismatch",
        LoadGateReferenceMutation::ArtifactReferenceMismatch,
    ),
    reference_case(
        "vm_report_reference_mismatch",
        "rejected",
        "retained_local_approval_reference_vm_report_reference_mismatch",
        LoadGateReferenceMutation::VmReportReferenceMismatch,
    ),
    reference_case(
        "local_attestation_reference_mismatch",
        "rejected",
        "retained_local_approval_reference_local_attestation_reference_mismatch",
        LoadGateReferenceMutation::LocalAttestationReferenceMismatch,
    ),
    reference_case(
        "computed_grant_reference_mismatch",
        "rejected",
        "retained_local_approval_reference_computed_grant_reference_mismatch",
        LoadGateReferenceMutation::ComputedGrantReferenceMismatch,
    ),
];

fn module_load_gate_approval_valid_candidate() -> ModuleLoadGateLocalApprovalReferenceCandidate {
    let chain = module_load_gate_reference_chain();
    module_load_gate_approval_candidate(
        true,
        true,
        "current_boot",
        Some(chain.approval_reference),
        Some(chain.approval_reference),
        chain.manifest_event_id,
        chain.manifest_reference,
        chain.artifact_event_id,
        chain.artifact_reference,
        chain.vm_report_event_id,
        chain.vm_report_reference,
        chain.attestation_event_id,
        chain.attestation_reference,
        chain.retained_event_id,
        chain.retained_reference,
    )
}

fn apply_approval_reference_case(
    candidate: &mut ModuleLoadGateLocalApprovalReferenceCandidate,
    mutation: LoadGateReferenceMutation,
) {
    let chain = module_load_gate_reference_chain();
    match mutation {
        LoadGateReferenceMutation::None => {}
        LoadGateReferenceMutation::Missing => {
            candidate.retained = false;
            candidate.event_reference = None;
            candidate.candidate_reference = None;
        }
        LoadGateReferenceMutation::Stale => candidate.retained = false,
        LoadGateReferenceMutation::PreviousBoot => candidate.scope = "previous_boot",
        LoadGateReferenceMutation::WrongSchema => candidate.schema_ok = false,
        LoadGateReferenceMutation::Substituted => {
            candidate.candidate_reference = Some(module_load_gate_test_local_approval_reference(
                chain.manifest_event_id,
                chain.artifact_event_id,
                chain.vm_report_event_id,
                chain.attestation_event_id,
                chain.retained_event_id,
                chain.manifest_reference,
                chain.artifact_reference,
                chain.vm_report_reference,
                event_log::ModuleLocalAttestationReference {
                    local_attestation_hash: [0xbc; 32],
                    signature_verified: false,
                    ..chain.attestation_reference
                },
                chain.retained_reference,
            ));
        }
        LoadGateReferenceMutation::ReferenceHashMismatch => {
            let mismatched = event_log::ModuleLocalApprovalReference {
                approval_reference_hash: [0x99; 32],
                ..chain.approval_reference
            };
            candidate.event_reference = Some(mismatched);
            candidate.candidate_reference = Some(mismatched);
        }
        LoadGateReferenceMutation::ManifestReferenceMismatch => {
            candidate.manifest_event_id = Some(module_load_gate_test_event_id(99));
        }
        LoadGateReferenceMutation::ArtifactReferenceMismatch => {
            candidate.artifact_event_id = Some(module_load_gate_test_event_id(98));
        }
        LoadGateReferenceMutation::VmReportReferenceMismatch => {
            candidate.vm_report_event_id = Some(module_load_gate_test_event_id(97));
        }
        LoadGateReferenceMutation::LocalAttestationReferenceMismatch => {
            candidate.attestation_event_id = Some(module_load_gate_test_event_id(96));
        }
        LoadGateReferenceMutation::ComputedGrantReferenceMismatch => {
            candidate.retained_event_id = Some(module_load_gate_test_event_id(95));
        }
        _ => {}
    }
}

fn evaluate_approval_reference_case(
    candidate: ModuleLoadGateLocalApprovalReferenceCandidate,
    _require_live_retained: bool,
) -> ModuleLoadGateLocalApprovalEvaluation {
    evaluate_module_load_gate_local_approval_candidate(candidate)
}

fn module_load_gate_approval_candidate(
    retained: bool,
    schema_ok: bool,
    scope: &'static str,
    event_reference: Option<event_log::ModuleLocalApprovalReference>,
    candidate_reference: Option<event_log::ModuleLocalApprovalReference>,
    manifest_event_id: event_log::EventId,
    manifest_reference: event_log::ModuleManifestReference,
    artifact_event_id: event_log::EventId,
    artifact_reference: event_log::ModuleCandidateArtifactReference,
    vm_report_event_id: event_log::EventId,
    vm_report_reference: event_log::ModuleVmTestReportReference,
    attestation_event_id: event_log::EventId,
    attestation_reference: event_log::ModuleLocalAttestationReference,
    retained_event_id: event_log::EventId,
    retained_reference: event_log::ModuleComputedGrantReference,
) -> ModuleLoadGateLocalApprovalReferenceCandidate {
    ModuleLoadGateLocalApprovalReferenceCandidate {
        scope,
        retained,
        schema_ok,
        event_reference,
        candidate_reference,
        manifest_event_id: Some(manifest_event_id),
        manifest_reference: Some(manifest_reference),
        artifact_event_id: Some(artifact_event_id),
        artifact_reference: Some(artifact_reference),
        vm_report_event_id: Some(vm_report_event_id),
        vm_report_reference: Some(vm_report_reference),
        attestation_event_id: Some(attestation_event_id),
        attestation_reference: Some(attestation_reference),
        retained_event_id: Some(retained_event_id),
        retained_reference: Some(retained_reference),
    }
}

fn module_load_gate_approval_selftest_case_from_spec(
    spec: &CaseSpec<LoadGateReferenceMutation>,
    actual: ModuleLoadGateLocalApprovalEvaluation,
) -> ModuleLoadGateLocalApprovalSelfTestCase {
    ModuleLoadGateLocalApprovalSelfTestCase {
        name: spec.name,
        expected_status: spec.expected_status,
        expected_reason: spec.expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_local_approval_state: actual.local_approval_state,
        accepted_local_approval_hash: actual.accepted_local_approval_hash,
        passed: method_eq(actual.status, spec.expected_status)
            && method_eq(actual.reason, spec.expected_reason)
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_load_gate_test_local_approval_reference(
    retained_manifest_reference_event_id: event_log::EventId,
    retained_artifact_reference_event_id: event_log::EventId,
    retained_vm_report_reference_event_id: event_log::EventId,
    retained_local_attestation_reference_event_id: event_log::EventId,
    retained_reference_event_id: event_log::EventId,
    manifest_reference: event_log::ModuleManifestReference,
    artifact_reference: event_log::ModuleCandidateArtifactReference,
    vm_report_reference: event_log::ModuleVmTestReportReference,
    attestation_reference: event_log::ModuleLocalAttestationReference,
    retained_reference: event_log::ModuleComputedGrantReference,
) -> event_log::ModuleLocalApprovalReference {
    let approval_reference_hash =
        module_evidence::computed_module_local_approval_reference_hash_from_sequences(
            retained_manifest_reference_event_id.sequence(),
            retained_artifact_reference_event_id.sequence(),
            retained_vm_report_reference_event_id.sequence(),
            retained_local_attestation_reference_event_id.sequence(),
            retained_reference_event_id.sequence(),
            manifest_reference.manifest_reference_hash,
            artifact_reference.artifact_reference_hash,
            vm_report_reference.report_reference_hash,
            attestation_reference.attestation_reference_hash,
            manifest_reference.manifest_hash,
            artifact_reference.artifact_hash,
            retained_reference.computed_grant_hash,
            vm_report_reference.vm_report_hash,
            retained_reference.local_attestation_hash,
            MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH,
        );
    event_log::ModuleLocalApprovalReference {
        approval_reference_hash,
        retained_manifest_reference_event_id,
        retained_artifact_reference_event_id,
        retained_vm_report_reference_event_id,
        retained_local_attestation_reference_event_id,
        retained_reference_event_id,
        manifest_reference_hash: manifest_reference.manifest_reference_hash,
        artifact_reference_hash: artifact_reference.artifact_reference_hash,
        vm_report_reference_hash: vm_report_reference.report_reference_hash,
        local_attestation_reference_hash: attestation_reference.attestation_reference_hash,
        manifest_hash: manifest_reference.manifest_hash,
        artifact_hash: artifact_reference.artifact_hash,
        computed_grant_hash: retained_reference.computed_grant_hash,
        vm_report_hash: vm_report_reference.vm_report_hash,
        local_attestation_hash: retained_reference.local_attestation_hash,
        local_approval_hash: MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH,
    }
}
