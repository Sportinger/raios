use alloc::{vec, vec::Vec};

use crate::{
    agent_protocol_module_reference::emit_evidence_v1_response,
    agent_protocol_module_service_slot_allocator_projection::latest_module_service_slot_allocator_readiness_projection,
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, crlf, emit_inline_record_object, emit_inline_record_object_fragment,
        emit_record_fields_trailing_comma, emit_record_property_line, end_response, method_eq,
        raw_line, record_bool as b, record_event_or_null, record_false as no, record_field as f,
        record_str as s,
    },
    event_log,
};
use raios_core::{
    evidence_response::{self as ev, SelftestFacts},
    module_loader_allocator_projection::{
        project_loader_artifact_hash_binding_denial, LoaderAllocatorDisposition,
        LoaderAllocatorEvidenceInput, LoaderAllocatorEvidenceStatus,
        LoaderArtifactHashBindingProjectionInput,
    },
    record::Value as V,
};

fn evidence_input<'a>(
    status: &'static str,
    reason: &'a str,
    source_event_id: Option<event_log::EventId>,
    facts: Vec<raios_core::record::Field<'a>>,
) -> LoaderAllocatorEvidenceInput<'a> {
    LoaderAllocatorEvidenceInput {
        status: match status {
            "available" => LoaderAllocatorEvidenceStatus::Verified,
            "missing" => LoaderAllocatorEvidenceStatus::Missing,
            "rejected" => LoaderAllocatorEvidenceStatus::Rejected,
            _ => LoaderAllocatorEvidenceStatus::Unavailable,
        },
        status_detail: status,
        reason,
        source_event_sequence: source_event_id.map(event_log::EventId::sequence),
        facts,
        disposition: if method_eq(status, "available") {
            LoaderAllocatorDisposition::Satisfied
        } else {
            LoaderAllocatorDisposition::Blocked
        },
    }
}

pub(crate) fn emit_module_loader_artifact_hash_binding() {
    let manifest = event_log::latest_module_manifest_reference();
    let artifact = event_log::latest_module_candidate_artifact_reference();
    let vm_report = event_log::latest_module_vm_test_report_reference();
    let local_attestation = event_log::latest_module_local_attestation_reference();
    let local_approval = event_log::latest_module_local_approval_reference();
    let computed_grant = event_log::latest_module_computed_grant_reference();
    let audit_rollback = event_log::latest_module_audit_rollback_reference();
    let service_slot = event_log::latest_module_service_slot_reservation();
    let loader_identity_source_evidence =
        event_log::latest_module_loader_identity_source_evidence();
    let loader_identity_source_evidence_present = loader_identity_source_evidence.is_some();
    let loader_identity_source_evidence_event_id =
        loader_identity_source_evidence.map(|(event_id, _)| event_id);
    let retained_module_evidence_present = manifest.is_some()
        && artifact.is_some()
        && vm_report.is_some()
        && local_attestation.is_some()
        && local_approval.is_some()
        && computed_grant.is_some()
        && audit_rollback.is_some()
        && service_slot.is_some();
    let loader_identity_present = loader_identity_source_evidence
        .map(|(_, evidence)| {
            evidence.identity_present && method_eq(evidence.identity_status, "available")
        })
        .unwrap_or(false);
    let service_slot_allocator = latest_module_service_slot_allocator_readiness_projection(
        service_slot.as_ref().map(|(event_id, _)| *event_id),
    );
    let candidate = ModuleLoaderArtifactHashBindingCandidate {
        retained_module_evidence_present,
        service_slot_allocator_readiness_present: service_slot_allocator.readiness_present,
        service_slot_allocator_ready: service_slot_allocator.ready,
        service_slot_allocator_unready_status: service_slot_allocator.unready_status,
        service_slot_allocator_unready_reason: service_slot_allocator.unready_reason,
        audit_rollback_write_boundary_present: false,
        loader_identity_present,
        artifact_hash_binding: module_loader_artifact_hash_binding_missing_fact(),
    };
    let evaluation = evaluate_module_loader_artifact_hash_binding_candidate(candidate);
    let source_evidence = module_loader_artifact_hash_binding_source_evidence(
        candidate,
        evaluation,
        loader_identity_source_evidence_event_id,
    );
    let source_evidence_event_id =
        event_log::record_module_loader_artifact_hash_binding_source_evidence(source_evidence);

    let common = |status, reason, facts| evidence_input(status, reason, None, facts);
    let projection =
        project_loader_artifact_hash_binding_denial(LoaderArtifactHashBindingProjectionInput {
            retained_module_evidence: common(
                evaluation.retained_module_evidence_status,
                evaluation.retained_module_evidence_reason,
                vec![f("present", b(candidate.retained_module_evidence_present))],
            ),
            service_slot_allocator_readiness: evidence_input(
                evaluation.service_slot_allocator_readiness_status,
                evaluation.service_slot_allocator_readiness_reason,
                service_slot_allocator.authority_source_evidence_event_id,
                vec![
                    f(
                        "present",
                        b(candidate.service_slot_allocator_readiness_present),
                    ),
                    f("ready", b(candidate.service_slot_allocator_ready)),
                ],
            ),
            service_slot_allocator_runtime: common(
                evaluation.service_slot_allocator_runtime_status,
                evaluation.service_slot_allocator_runtime_reason,
                vec![f("ready", b(candidate.service_slot_allocator_ready))],
            ),
            audit_rollback_write_boundary: common(
                evaluation.audit_rollback_write_boundary_status,
                evaluation.audit_rollback_write_boundary_reason,
                vec![f(
                    "present",
                    b(candidate.audit_rollback_write_boundary_present),
                )],
            ),
            loader_identity: evidence_input(
                evaluation.loader_identity_status,
                evaluation.loader_identity_reason,
                loader_identity_source_evidence_event_id,
                vec![f("present", b(candidate.loader_identity_present))],
            ),
            artifact_hash_binding: evidence_input(
                evaluation.artifact_hash_binding_status,
                evaluation.artifact_hash_binding_reason,
                Some(source_evidence_event_id),
                vec![
                    f("record_schema", s(source_evidence.fact_schema)),
                    f("record_id", s(source_evidence.fact_id)),
                    f("source_method", s(source_evidence.source_method)),
                    f(
                        "source_fact_locator",
                        s(source_evidence.source_fact_locator),
                    ),
                    f("present", b(candidate.artifact_hash_binding.present)),
                    f("schema_valid", b(candidate.artifact_hash_binding.schema_ok)),
                    f(
                        "provenance_valid",
                        b(candidate.artifact_hash_binding.provenance_ok),
                    ),
                    f(
                        "binds_retained_module_evidence",
                        b(candidate
                            .artifact_hash_binding
                            .binds_retained_module_evidence),
                    ),
                    f(
                        "binds_service_slot_allocator",
                        b(candidate.artifact_hash_binding.binds_service_slot_allocator),
                    ),
                    f(
                        "binds_audit_rollback_write_boundary",
                        b(candidate
                            .artifact_hash_binding
                            .binds_audit_rollback_write_boundary),
                    ),
                    f(
                        "binds_loader_identity",
                        b(candidate.artifact_hash_binding.binds_loader_identity),
                    ),
                ],
            ),
        });
    emit_evidence_v1_response(
        "module.loader_artifact_hash_binding",
        "module.loader_artifact_hash_binding",
        None,
        V::InlineObject(vec![f("test_infrastructure", no())]),
        projection
            .evidence
            .into_iter()
            .map(ev::evidence_value)
            .collect(),
        projection.decision,
    );
}

pub(crate) fn emit_module_loader_artifact_hash_binding_selftest() {
    let cases = module_loader_artifact_hash_binding_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    let values = cases
        .iter()
        .map(|case| {
            ev::selftest_case(
                case.name,
                case.expected_status,
                case.expected_reason,
                case.actual_status,
                case.actual_reason,
                case.passed,
            )
        })
        .collect();
    emit_evidence_v1_response(
        "module.loader_artifact_hash_binding_selftest",
        "module.loader_artifact_hash_binding_selftest",
        None,
        ev::selftest_facts_value(SelftestFacts {
            case_count: cases.len() as u64,
            passed,
            safety: ev::selftest_safety_value(),
            cases: V::Array(values),
        }),
        vec![],
        ev::observed("selftest_completed"),
    );
}

fn module_loader_artifact_hash_binding_source_evidence(
    candidate: ModuleLoaderArtifactHashBindingCandidate,
    evaluation: ModuleLoaderArtifactHashBindingEvaluation,
    loader_identity_source_evidence_event_id: Option<event_log::EventId>,
) -> event_log::ModuleLoaderArtifactHashBindingSourceEvidence {
    event_log::ModuleLoaderArtifactHashBindingSourceEvidence {
        schema: "raios.module_loader_artifact_hash_binding_source_evidence.v0",
        fact_schema: "raios.module_loader_artifact_hash_binding.v0",
        fact_id: "module.loader_runtime.artifact_hash_binding.current_boot",
        source_method: "module.loader_artifact_hash_binding",
        source_fact_locator: "module.loader_artifact_hash_binding.artifact_hash_binding",
        readiness_status: evaluation.status,
        readiness_reason: evaluation.reason,
        artifact_hash_binding_status: evaluation.artifact_hash_binding_status,
        artifact_hash_binding_reason: evaluation.artifact_hash_binding_reason,
        artifact_hash_binding_present: candidate.artifact_hash_binding.present,
        artifact_hash_binding_scope: candidate.artifact_hash_binding.scope,
        artifact_hash_binding_schema_ok: candidate.artifact_hash_binding.schema_ok,
        artifact_hash_binding_provenance_ok: candidate.artifact_hash_binding.provenance_ok,
        artifact_hash_binding_classification: candidate.artifact_hash_binding.classification,
        retained_module_evidence_present: candidate.retained_module_evidence_present,
        service_slot_allocator_readiness_present: candidate
            .service_slot_allocator_readiness_present,
        service_slot_allocator_ready: candidate.service_slot_allocator_ready,
        audit_rollback_write_boundary_present: candidate.audit_rollback_write_boundary_present,
        loader_identity_present: candidate.loader_identity_present,
        binds_retained_module_evidence: candidate
            .artifact_hash_binding
            .binds_retained_module_evidence,
        binds_service_slot_allocator: candidate.artifact_hash_binding.binds_service_slot_allocator,
        binds_audit_rollback_write_boundary: candidate
            .artifact_hash_binding
            .binds_audit_rollback_write_boundary,
        binds_loader_identity: candidate.artifact_hash_binding.binds_loader_identity,
        loader_identity_source_evidence_event_id,
    }
}

fn evaluate_module_loader_artifact_hash_binding_candidate(
    candidate: ModuleLoaderArtifactHashBindingCandidate,
) -> ModuleLoaderArtifactHashBindingEvaluation {
    let (retained_module_evidence_status, retained_module_evidence_reason) =
        if candidate.retained_module_evidence_present {
            ("available", "retained_module_evidence_available")
        } else {
            ("missing", "retained_module_evidence_missing")
        };
    let (service_slot_allocator_readiness_status, service_slot_allocator_readiness_reason) =
        if !candidate.service_slot_allocator_readiness_present {
            ("missing", "service_slot_allocator_readiness_missing")
        } else if candidate.service_slot_allocator_ready {
            ("available", "service_slot_allocator_readiness_available")
        } else {
            (
                candidate.service_slot_allocator_unready_status,
                candidate.service_slot_allocator_unready_reason,
            )
        };
    let (service_slot_allocator_runtime_status, service_slot_allocator_runtime_reason) =
        if candidate.service_slot_allocator_ready {
            ("available", "service_slot_allocator_runtime_available")
        } else if method_eq(
            candidate.service_slot_allocator_unready_status,
            "denied_missing_service_slot_allocator_runtime",
        ) {
            ("missing", candidate.service_slot_allocator_unready_reason)
        } else {
            ("available", "service_slot_allocator_runtime_available")
        };
    let (audit_rollback_write_boundary_status, audit_rollback_write_boundary_reason) =
        if candidate.audit_rollback_write_boundary_present {
            (
                "available",
                "module_audit_rollback_write_boundary_binding_available",
            )
        } else {
            (
                "missing",
                "module_audit_rollback_write_boundary_binding_missing",
            )
        };
    let (loader_identity_status, loader_identity_reason) = if candidate.loader_identity_present {
        ("available", "module_loader_identity_available")
    } else {
        ("missing", "module_loader_identity_missing")
    };
    let (artifact_hash_binding_status, artifact_hash_binding_reason) =
        evaluate_module_loader_artifact_hash_binding_fact(candidate.artifact_hash_binding);

    let (status, reason) = if !candidate.retained_module_evidence_present {
        (
            "denied_missing_retained_module_evidence",
            retained_module_evidence_reason,
        )
    } else if !candidate.service_slot_allocator_readiness_present {
        (
            "denied_missing_service_slot_allocator_readiness",
            service_slot_allocator_readiness_reason,
        )
    } else if !candidate.service_slot_allocator_ready {
        (
            candidate.service_slot_allocator_unready_status,
            candidate.service_slot_allocator_unready_reason,
        )
    } else if !candidate.audit_rollback_write_boundary_present {
        (
            "denied_missing_audit_rollback_write_boundary",
            audit_rollback_write_boundary_reason,
        )
    } else if !candidate.loader_identity_present {
        ("denied_missing_loader_identity", loader_identity_reason)
    } else if method_eq(artifact_hash_binding_status, "rejected") {
        ("rejected", artifact_hash_binding_reason)
    } else if method_eq(artifact_hash_binding_status, "missing") {
        (
            "denied_missing_loader_artifact_hash_binding",
            artifact_hash_binding_reason,
        )
    } else {
        (
            "available_non_authorizing",
            "module_loader_artifact_hash_binding_not_load_authority",
        )
    };

    ModuleLoaderArtifactHashBindingEvaluation {
        status,
        reason,
        retained_module_evidence_status,
        retained_module_evidence_reason,
        service_slot_allocator_readiness_status,
        service_slot_allocator_readiness_reason,
        service_slot_allocator_runtime_status,
        service_slot_allocator_runtime_reason,
        audit_rollback_write_boundary_status,
        audit_rollback_write_boundary_reason,
        loader_identity_status,
        loader_identity_reason,
        artifact_hash_binding_status,
        artifact_hash_binding_reason,
        loads_artifact: false,
        allocates_service_slot: false,
        creates_service_inventory_records: false,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_loader_artifact_hash_binding_fact(
    fact: ModuleLoaderArtifactHashBindingFact,
) -> (&'static str, &'static str) {
    if !method_eq(fact.scope, "current_boot") {
        return (
            "rejected",
            "module_loader_artifact_hash_binding_scope_must_be_current_boot",
        );
    }
    if !fact.schema_ok {
        return (
            "rejected",
            "module_loader_artifact_hash_binding_schema_mismatch",
        );
    }
    if !fact.present {
        return ("missing", "module_loader_artifact_hash_binding_missing");
    }
    if !fact.provenance_ok {
        return (
            "rejected",
            "module_loader_artifact_hash_binding_provenance_missing",
        );
    }
    if !fact.binds_retained_module_evidence {
        return (
            "rejected",
            "module_loader_artifact_hash_binding_retained_evidence_binding_missing",
        );
    }
    if !fact.binds_service_slot_allocator {
        return (
            "rejected",
            "module_loader_artifact_hash_binding_service_slot_allocator_binding_missing",
        );
    }
    if !fact.binds_audit_rollback_write_boundary {
        return (
            "rejected",
            "module_loader_artifact_hash_binding_audit_write_boundary_binding_missing",
        );
    }
    if !fact.binds_loader_identity {
        return (
            "rejected",
            "module_loader_artifact_hash_binding_loader_identity_binding_missing",
        );
    }
    ("available", "module_loader_artifact_hash_binding_available")
}

fn module_loader_artifact_hash_binding_selftest_cases(
) -> [ModuleLoaderArtifactHashBindingSelfTestCase; MODULE_LOADER_ARTIFACT_HASH_BINDING_SELFTEST_CASES]
{
    let ready = module_loader_artifact_hash_binding_ready_candidate();
    let missing_fact = module_loader_artifact_hash_binding_missing_fact();
    [
        module_loader_artifact_hash_binding_selftest_case(
            "missing_retained_module_evidence",
            "denied_missing_retained_module_evidence",
            "retained_module_evidence_missing",
            ModuleLoaderArtifactHashBindingCandidate {
                retained_module_evidence_present: false,
                ..ready
            },
        ),
        module_loader_artifact_hash_binding_selftest_case(
            "missing_service_slot_allocator_readiness",
            "denied_missing_service_slot_allocator_readiness",
            "service_slot_allocator_readiness_missing",
            ModuleLoaderArtifactHashBindingCandidate {
                service_slot_allocator_readiness_present: false,
                ..ready
            },
        ),
        module_loader_artifact_hash_binding_selftest_case(
            "service_slot_allocator_runtime_missing",
            "denied_missing_service_slot_allocator_runtime",
            "service_slot_allocator_runtime_missing",
            ModuleLoaderArtifactHashBindingCandidate {
                service_slot_allocator_ready: false,
                service_slot_allocator_unready_status:
                    "denied_missing_service_slot_allocator_runtime",
                service_slot_allocator_unready_reason: "service_slot_allocator_runtime_missing",
                ..ready
            },
        ),
        module_loader_artifact_hash_binding_selftest_case(
            "audit_write_boundary_missing",
            "denied_missing_audit_rollback_write_boundary",
            "module_audit_rollback_write_boundary_binding_missing",
            ModuleLoaderArtifactHashBindingCandidate {
                audit_rollback_write_boundary_present: false,
                ..ready
            },
        ),
        module_loader_artifact_hash_binding_selftest_case(
            "loader_identity_missing",
            "denied_missing_loader_identity",
            "module_loader_identity_missing",
            ModuleLoaderArtifactHashBindingCandidate {
                loader_identity_present: false,
                ..ready
            },
        ),
        module_loader_artifact_hash_binding_selftest_case(
            "artifact_hash_binding_previous_boot",
            "rejected",
            "module_loader_artifact_hash_binding_scope_must_be_current_boot",
            ModuleLoaderArtifactHashBindingCandidate {
                artifact_hash_binding: ModuleLoaderArtifactHashBindingFact {
                    scope: "previous_boot",
                    ..ready.artifact_hash_binding
                },
                ..ready
            },
        ),
        module_loader_artifact_hash_binding_selftest_case(
            "artifact_hash_binding_schema_mismatch",
            "rejected",
            "module_loader_artifact_hash_binding_schema_mismatch",
            ModuleLoaderArtifactHashBindingCandidate {
                artifact_hash_binding: ModuleLoaderArtifactHashBindingFact {
                    schema_ok: false,
                    ..ready.artifact_hash_binding
                },
                ..ready
            },
        ),
        module_loader_artifact_hash_binding_selftest_case(
            "artifact_hash_binding_provenance_missing",
            "rejected",
            "module_loader_artifact_hash_binding_provenance_missing",
            ModuleLoaderArtifactHashBindingCandidate {
                artifact_hash_binding: ModuleLoaderArtifactHashBindingFact {
                    provenance_ok: false,
                    ..ready.artifact_hash_binding
                },
                ..ready
            },
        ),
        module_loader_artifact_hash_binding_selftest_case(
            "artifact_hash_binding_retained_evidence_binding_missing",
            "rejected",
            "module_loader_artifact_hash_binding_retained_evidence_binding_missing",
            ModuleLoaderArtifactHashBindingCandidate {
                artifact_hash_binding: ModuleLoaderArtifactHashBindingFact {
                    binds_retained_module_evidence: false,
                    ..ready.artifact_hash_binding
                },
                ..ready
            },
        ),
        module_loader_artifact_hash_binding_selftest_case(
            "artifact_hash_binding_service_slot_allocator_binding_missing",
            "rejected",
            "module_loader_artifact_hash_binding_service_slot_allocator_binding_missing",
            ModuleLoaderArtifactHashBindingCandidate {
                artifact_hash_binding: ModuleLoaderArtifactHashBindingFact {
                    binds_service_slot_allocator: false,
                    ..ready.artifact_hash_binding
                },
                ..ready
            },
        ),
        module_loader_artifact_hash_binding_selftest_case(
            "artifact_hash_binding_audit_write_boundary_binding_missing",
            "rejected",
            "module_loader_artifact_hash_binding_audit_write_boundary_binding_missing",
            ModuleLoaderArtifactHashBindingCandidate {
                artifact_hash_binding: ModuleLoaderArtifactHashBindingFact {
                    binds_audit_rollback_write_boundary: false,
                    ..ready.artifact_hash_binding
                },
                ..ready
            },
        ),
        module_loader_artifact_hash_binding_selftest_case(
            "artifact_hash_binding_loader_identity_binding_missing",
            "rejected",
            "module_loader_artifact_hash_binding_loader_identity_binding_missing",
            ModuleLoaderArtifactHashBindingCandidate {
                artifact_hash_binding: ModuleLoaderArtifactHashBindingFact {
                    binds_loader_identity: false,
                    ..ready.artifact_hash_binding
                },
                ..ready
            },
        ),
        module_loader_artifact_hash_binding_selftest_case(
            "artifact_hash_binding_missing",
            "denied_missing_loader_artifact_hash_binding",
            "module_loader_artifact_hash_binding_missing",
            ModuleLoaderArtifactHashBindingCandidate {
                artifact_hash_binding: missing_fact,
                ..ready
            },
        ),
        module_loader_artifact_hash_binding_selftest_case(
            "all_inputs_present_artifact_hash_binding_non_authorizing",
            "available_non_authorizing",
            "module_loader_artifact_hash_binding_not_load_authority",
            ready,
        ),
    ]
}

fn module_loader_artifact_hash_binding_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoaderArtifactHashBindingCandidate,
) -> ModuleLoaderArtifactHashBindingSelfTestCase {
    let actual = evaluate_module_loader_artifact_hash_binding_candidate(candidate);
    ModuleLoaderArtifactHashBindingSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_artifact_hash_binding_status: actual.artifact_hash_binding_status,
        actual_artifact_hash_binding_reason: actual.artifact_hash_binding_reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.loads_artifact
            && !actual.allocates_service_slot
            && !actual.creates_service_inventory_records
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_loader_artifact_hash_binding_ready_candidate() -> ModuleLoaderArtifactHashBindingCandidate
{
    ModuleLoaderArtifactHashBindingCandidate {
        retained_module_evidence_present: true,
        service_slot_allocator_readiness_present: true,
        service_slot_allocator_ready: true,
        service_slot_allocator_unready_status: "available",
        service_slot_allocator_unready_reason: "service_slot_allocator_runtime_available",
        audit_rollback_write_boundary_present: true,
        loader_identity_present: true,
        artifact_hash_binding: module_loader_artifact_hash_binding_available_fact(),
    }
}

fn module_loader_artifact_hash_binding_missing_fact() -> ModuleLoaderArtifactHashBindingFact {
    ModuleLoaderArtifactHashBindingFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_retained_module_evidence: false,
        binds_service_slot_allocator: false,
        binds_audit_rollback_write_boundary: false,
        binds_loader_identity: false,
    }
}

fn module_loader_artifact_hash_binding_available_fact() -> ModuleLoaderArtifactHashBindingFact {
    ModuleLoaderArtifactHashBindingFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_retained_module_evidence: true,
        binds_service_slot_allocator: true,
        binds_audit_rollback_write_boundary: true,
        binds_loader_identity: true,
    }
}
