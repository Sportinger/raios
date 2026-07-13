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
        project_loader_identity_denial, LoaderAllocatorDisposition, LoaderAllocatorEvidenceInput,
        LoaderAllocatorEvidenceStatus, LoaderIdentityProjectionInput,
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

pub(crate) fn emit_module_loader_identity() {
    let manifest = event_log::latest_module_manifest_reference();
    let artifact = event_log::latest_module_candidate_artifact_reference();
    let vm_report = event_log::latest_module_vm_test_report_reference();
    let local_attestation = event_log::latest_module_local_attestation_reference();
    let local_approval = event_log::latest_module_local_approval_reference();
    let computed_grant = event_log::latest_module_computed_grant_reference();
    let audit_rollback = event_log::latest_module_audit_rollback_reference();
    let service_slot = event_log::latest_module_service_slot_reservation();
    let retained_module_evidence_present = manifest.is_some()
        && artifact.is_some()
        && vm_report.is_some()
        && local_attestation.is_some()
        && local_approval.is_some()
        && computed_grant.is_some()
        && audit_rollback.is_some()
        && service_slot.is_some();
    let service_slot_allocator = latest_module_service_slot_allocator_readiness_projection(
        service_slot.as_ref().map(|(event_id, _)| *event_id),
    );
    let candidate = ModuleLoaderIdentityCandidate {
        retained_module_evidence_present,
        service_slot_allocator_readiness_present: service_slot_allocator.readiness_present,
        service_slot_allocator_ready: service_slot_allocator.ready,
        service_slot_allocator_unready_status: service_slot_allocator.unready_status,
        service_slot_allocator_unready_reason: service_slot_allocator.unready_reason,
        audit_rollback_write_boundary_present: false,
        identity: module_loader_identity_missing_fact(),
    };
    let evaluation = evaluate_module_loader_identity_candidate(candidate);
    let source_evidence = module_loader_identity_source_evidence(
        candidate,
        evaluation,
        manifest.as_ref().map(|(event_id, _)| *event_id),
        artifact.as_ref().map(|(event_id, _)| *event_id),
        vm_report.as_ref().map(|(event_id, _)| *event_id),
        local_attestation.as_ref().map(|(event_id, _)| *event_id),
        local_approval.as_ref().map(|(event_id, _)| *event_id),
        computed_grant.as_ref().map(|(event_id, _)| *event_id),
        audit_rollback.as_ref().map(|(event_id, _)| *event_id),
        service_slot.as_ref().map(|(event_id, _)| *event_id),
    );
    let source_evidence_event_id =
        event_log::record_module_loader_identity_source_evidence(source_evidence);

    let projection = project_loader_identity_denial(LoaderIdentityProjectionInput {
        retained_module_evidence: evidence_input(
            evaluation.retained_module_evidence_status,
            evaluation.retained_module_evidence_reason,
            None,
            vec![
                f(
                    "manifest_reference_event_id",
                    record_event_or_null(manifest.map(|v| v.0)),
                ),
                f(
                    "candidate_artifact_reference_event_id",
                    record_event_or_null(artifact.map(|v| v.0)),
                ),
                f(
                    "vm_test_report_reference_event_id",
                    record_event_or_null(vm_report.map(|v| v.0)),
                ),
                f(
                    "local_attestation_reference_event_id",
                    record_event_or_null(local_attestation.map(|v| v.0)),
                ),
                f(
                    "local_approval_reference_event_id",
                    record_event_or_null(local_approval.map(|v| v.0)),
                ),
                f(
                    "computed_grant_reference_event_id",
                    record_event_or_null(computed_grant.map(|v| v.0)),
                ),
                f(
                    "audit_rollback_reference_event_id",
                    record_event_or_null(audit_rollback.map(|v| v.0)),
                ),
                f(
                    "service_slot_reservation_event_id",
                    record_event_or_null(service_slot.map(|v| v.0)),
                ),
            ],
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
        service_slot_allocator_runtime: evidence_input(
            evaluation.service_slot_allocator_runtime_status,
            evaluation.service_slot_allocator_runtime_reason,
            service_slot_allocator.authority_source_evidence_event_id,
            vec![f("ready", b(candidate.service_slot_allocator_ready))],
        ),
        audit_rollback_write_boundary: evidence_input(
            evaluation.audit_rollback_write_boundary_status,
            evaluation.audit_rollback_write_boundary_reason,
            None,
            vec![f(
                "present",
                b(candidate.audit_rollback_write_boundary_present),
            )],
        ),
        loader_identity: evidence_input(
            evaluation.identity_status,
            evaluation.identity_reason,
            Some(source_evidence_event_id),
            vec![
                f("record_schema", s(source_evidence.fact_schema)),
                f("record_id", s(source_evidence.fact_id)),
                f("source_method", s(source_evidence.source_method)),
                f(
                    "source_fact_locator",
                    s(source_evidence.source_fact_locator),
                ),
                f("present", b(candidate.identity.present)),
                f("schema_valid", b(candidate.identity.schema_ok)),
                f("provenance_valid", b(candidate.identity.provenance_ok)),
                f(
                    "binds_retained_module_evidence",
                    b(candidate.identity.binds_retained_module_evidence),
                ),
                f(
                    "binds_service_slot_allocator",
                    b(candidate.identity.binds_service_slot_allocator),
                ),
                f(
                    "binds_audit_rollback_write_boundary",
                    b(candidate.identity.binds_audit_rollback_write_boundary),
                ),
            ],
        ),
    });
    emit_evidence_v1_response(
        "module.loader_identity",
        "module.loader_identity",
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

pub(crate) fn emit_module_loader_identity_selftest() {
    let cases = module_loader_identity_selftest_cases();
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
        "module.loader_identity_selftest",
        "module.loader_identity_selftest",
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

fn module_loader_identity_source_evidence(
    candidate: ModuleLoaderIdentityCandidate,
    evaluation: ModuleLoaderIdentityEvaluation,
    manifest_event_id: Option<event_log::EventId>,
    artifact_event_id: Option<event_log::EventId>,
    vm_report_event_id: Option<event_log::EventId>,
    local_attestation_event_id: Option<event_log::EventId>,
    local_approval_event_id: Option<event_log::EventId>,
    computed_grant_event_id: Option<event_log::EventId>,
    audit_rollback_event_id: Option<event_log::EventId>,
    service_slot_event_id: Option<event_log::EventId>,
) -> event_log::ModuleLoaderIdentitySourceEvidence {
    event_log::ModuleLoaderIdentitySourceEvidence {
        schema: "raios.module_loader_identity_source_evidence.v0",
        fact_schema: "raios.module_loader_identity.v0",
        fact_id: "module.loader_runtime.identity.current_boot",
        source_method: "module.loader_identity",
        source_fact_locator: "module.loader_identity.loader_identity",
        readiness_status: evaluation.status,
        readiness_reason: evaluation.reason,
        identity_status: evaluation.identity_status,
        identity_reason: evaluation.identity_reason,
        identity_present: candidate.identity.present,
        identity_scope: candidate.identity.scope,
        identity_schema_ok: candidate.identity.schema_ok,
        identity_provenance_ok: candidate.identity.provenance_ok,
        identity_classification: candidate.identity.classification,
        retained_module_evidence_present: candidate.retained_module_evidence_present,
        service_slot_allocator_readiness_present: candidate
            .service_slot_allocator_readiness_present,
        service_slot_allocator_ready: candidate.service_slot_allocator_ready,
        audit_rollback_write_boundary_present: candidate.audit_rollback_write_boundary_present,
        binds_retained_module_evidence: candidate.identity.binds_retained_module_evidence,
        binds_service_slot_allocator: candidate.identity.binds_service_slot_allocator,
        binds_audit_rollback_write_boundary: candidate.identity.binds_audit_rollback_write_boundary,
        manifest_reference_event_id: manifest_event_id,
        artifact_reference_event_id: artifact_event_id,
        vm_test_report_reference_event_id: vm_report_event_id,
        local_attestation_reference_event_id: local_attestation_event_id,
        local_approval_reference_event_id: local_approval_event_id,
        computed_grant_reference_event_id: computed_grant_event_id,
        audit_rollback_reference_event_id: audit_rollback_event_id,
        service_slot_reservation_event_id: service_slot_event_id,
    }
}

fn evaluate_module_loader_identity_candidate(
    candidate: ModuleLoaderIdentityCandidate,
) -> ModuleLoaderIdentityEvaluation {
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
    let (identity_status, identity_reason) =
        evaluate_module_loader_identity_fact(candidate.identity);

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
    } else if method_eq(identity_status, "rejected") {
        ("rejected", identity_reason)
    } else if method_eq(identity_status, "missing") {
        ("denied_missing_loader_identity", identity_reason)
    } else {
        (
            "available_non_authorizing",
            "module_loader_identity_not_load_authority",
        )
    };

    ModuleLoaderIdentityEvaluation {
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
        identity_status,
        identity_reason,
        loads_artifact: false,
        allocates_service_slot: false,
        creates_service_inventory_records: false,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_loader_identity_fact(
    fact: ModuleLoaderRuntimeFact,
) -> (&'static str, &'static str) {
    if !method_eq(fact.scope, "current_boot") {
        return (
            "rejected",
            "module_loader_identity_scope_must_be_current_boot",
        );
    }
    if !fact.schema_ok {
        return ("rejected", "module_loader_identity_schema_mismatch");
    }
    if !fact.present {
        return ("missing", "module_loader_identity_missing");
    }
    if !fact.provenance_ok {
        return ("rejected", "module_loader_identity_provenance_missing");
    }
    if !fact.binds_retained_module_evidence {
        return (
            "rejected",
            "module_loader_identity_retained_evidence_binding_missing",
        );
    }
    if !fact.binds_service_slot_allocator {
        return (
            "rejected",
            "module_loader_identity_service_slot_allocator_binding_missing",
        );
    }
    if !fact.binds_audit_rollback_write_boundary {
        return (
            "rejected",
            "module_loader_identity_audit_write_boundary_binding_missing",
        );
    }
    ("available", "module_loader_identity_available")
}

fn module_loader_identity_selftest_cases(
) -> [ModuleLoaderIdentitySelfTestCase; MODULE_LOADER_IDENTITY_SELFTEST_CASES] {
    let ready = module_loader_identity_ready_candidate();
    let missing_fact = module_loader_identity_missing_fact();
    [
        module_loader_identity_selftest_case(
            "missing_retained_module_evidence",
            "denied_missing_retained_module_evidence",
            "retained_module_evidence_missing",
            ModuleLoaderIdentityCandidate {
                retained_module_evidence_present: false,
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "missing_service_slot_allocator_readiness",
            "denied_missing_service_slot_allocator_readiness",
            "service_slot_allocator_readiness_missing",
            ModuleLoaderIdentityCandidate {
                service_slot_allocator_readiness_present: false,
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "service_slot_allocator_runtime_missing",
            "denied_missing_service_slot_allocator_runtime",
            "service_slot_allocator_runtime_missing",
            ModuleLoaderIdentityCandidate {
                service_slot_allocator_ready: false,
                service_slot_allocator_unready_status:
                    "denied_missing_service_slot_allocator_runtime",
                service_slot_allocator_unready_reason: "service_slot_allocator_runtime_missing",
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "audit_write_boundary_missing",
            "denied_missing_audit_rollback_write_boundary",
            "module_audit_rollback_write_boundary_binding_missing",
            ModuleLoaderIdentityCandidate {
                audit_rollback_write_boundary_present: false,
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "loader_identity_previous_boot",
            "rejected",
            "module_loader_identity_scope_must_be_current_boot",
            ModuleLoaderIdentityCandidate {
                identity: ModuleLoaderRuntimeFact {
                    scope: "previous_boot",
                    ..ready.identity
                },
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "loader_identity_schema_mismatch",
            "rejected",
            "module_loader_identity_schema_mismatch",
            ModuleLoaderIdentityCandidate {
                identity: ModuleLoaderRuntimeFact {
                    schema_ok: false,
                    ..ready.identity
                },
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "loader_identity_provenance_missing",
            "rejected",
            "module_loader_identity_provenance_missing",
            ModuleLoaderIdentityCandidate {
                identity: ModuleLoaderRuntimeFact {
                    provenance_ok: false,
                    ..ready.identity
                },
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "loader_identity_retained_evidence_binding_missing",
            "rejected",
            "module_loader_identity_retained_evidence_binding_missing",
            ModuleLoaderIdentityCandidate {
                identity: ModuleLoaderRuntimeFact {
                    binds_retained_module_evidence: false,
                    ..ready.identity
                },
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "loader_identity_service_slot_allocator_binding_missing",
            "rejected",
            "module_loader_identity_service_slot_allocator_binding_missing",
            ModuleLoaderIdentityCandidate {
                identity: ModuleLoaderRuntimeFact {
                    binds_service_slot_allocator: false,
                    ..ready.identity
                },
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "loader_identity_audit_write_boundary_binding_missing",
            "rejected",
            "module_loader_identity_audit_write_boundary_binding_missing",
            ModuleLoaderIdentityCandidate {
                identity: ModuleLoaderRuntimeFact {
                    binds_audit_rollback_write_boundary: false,
                    ..ready.identity
                },
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "loader_identity_missing",
            "denied_missing_loader_identity",
            "module_loader_identity_missing",
            ModuleLoaderIdentityCandidate {
                identity: missing_fact,
                ..ready
            },
        ),
        module_loader_identity_selftest_case(
            "all_inputs_present_identity_non_authorizing",
            "available_non_authorizing",
            "module_loader_identity_not_load_authority",
            ready,
        ),
    ]
}

fn module_loader_identity_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoaderIdentityCandidate,
) -> ModuleLoaderIdentitySelfTestCase {
    let actual = evaluate_module_loader_identity_candidate(candidate);
    ModuleLoaderIdentitySelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_identity_status: actual.identity_status,
        actual_identity_reason: actual.identity_reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.loads_artifact
            && !actual.allocates_service_slot
            && !actual.creates_service_inventory_records
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_loader_identity_ready_candidate() -> ModuleLoaderIdentityCandidate {
    ModuleLoaderIdentityCandidate {
        retained_module_evidence_present: true,
        service_slot_allocator_readiness_present: true,
        service_slot_allocator_ready: true,
        service_slot_allocator_unready_status: "available",
        service_slot_allocator_unready_reason: "service_slot_allocator_runtime_available",
        audit_rollback_write_boundary_present: true,
        identity: module_loader_identity_available_fact(),
    }
}

fn module_loader_identity_missing_fact() -> ModuleLoaderRuntimeFact {
    ModuleLoaderRuntimeFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_retained_module_evidence: false,
        binds_service_slot_allocator: false,
        binds_audit_rollback_write_boundary: false,
        source_evidence_event_id: None,
        source_evidence_schema: "raios.module_loader_identity_source_evidence.v0",
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: "missing",
        source_evidence_reason: "module_loader_identity_source_evidence_missing",
        source_evidence_method: "module.loader_identity",
        source_evidence_fact_locator: "module.loader_identity.loader_identity",
    }
}

fn module_loader_identity_available_fact() -> ModuleLoaderRuntimeFact {
    ModuleLoaderRuntimeFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_retained_module_evidence: true,
        binds_service_slot_allocator: true,
        binds_audit_rollback_write_boundary: true,
        source_evidence_event_id: None,
        source_evidence_schema: "raios.module_loader_identity_source_evidence.v0",
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: "available",
        source_evidence_reason: "module_loader_identity_available",
        source_evidence_method: "module.loader_identity",
        source_evidence_fact_locator: "module.loader_identity.loader_identity",
    }
}
