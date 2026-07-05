use alloc::vec;

use crate::{
    agent_protocol_module_service_slot_allocator_projection::latest_module_service_slot_allocator_readiness_projection,
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, crlf, emit_inline_record_object, emit_inline_record_object_fragment,
        emit_record_fields_trailing_comma, emit_record_property_line, end_response, method_eq,
        method_head_eq, raw_line, record_bool as b, record_event_or_null, record_false as no,
        record_field as f, record_str as s,
    },
    event_log,
};
use raios_core::record::Value as V;

pub(crate) fn module_loader_artifact_hash_binding_method(method: &str) -> bool {
    method_head_eq(method, "module.loader_artifact_hash_binding")
}

pub(crate) fn module_loader_artifact_hash_binding_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.loader_artifact_hash_binding_selftest")
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

    begin_response("module.loader_artifact_hash_binding");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.module_loader_artifact_hash_binding.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", no()),
            f("mutates_global_event_log", b(true)),
            f(
                "global_event_log_mutation",
                s("retained_current_boot_source_evidence_only"),
            ),
            f("accepts_loader_descriptor", no()),
            f("accepts_artifact_bytes", no()),
            f("loads_artifact", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        6,
    );
    emit_module_loader_artifact_hash_binding_source_evidence(
        source_evidence_event_id,
        source_evidence,
    );
    raw_line(",");
    emit_module_loader_artifact_hash_binding_required_bindings(
        candidate,
        evaluation,
        loader_identity_source_evidence_present,
    );
    raw_line(",");
    emit_module_loader_artifact_hash_binding_fact(
        candidate.artifact_hash_binding,
        evaluation,
        source_evidence_event_id,
    );
    raw_line(",");
    emit_module_loader_artifact_hash_binding_policy_result(candidate, evaluation);
    raw_line(",");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    emit_module_loader_artifact_hash_binding_gate(
        &mut wrote,
        "retained_module_evidence",
        evaluation.retained_module_evidence_status,
        evaluation.retained_module_evidence_reason,
    );
    emit_module_loader_artifact_hash_binding_gate(
        &mut wrote,
        "service_slot_allocator_readiness",
        evaluation.service_slot_allocator_readiness_status,
        evaluation.service_slot_allocator_readiness_reason,
    );
    emit_module_loader_artifact_hash_binding_gate(
        &mut wrote,
        "service_slot_allocator_runtime",
        evaluation.service_slot_allocator_runtime_status,
        evaluation.service_slot_allocator_runtime_reason,
    );
    emit_module_loader_artifact_hash_binding_gate(
        &mut wrote,
        "audit_rollback_write_boundary",
        evaluation.audit_rollback_write_boundary_status,
        evaluation.audit_rollback_write_boundary_reason,
    );
    emit_module_loader_artifact_hash_binding_gate(
        &mut wrote,
        "loader_identity",
        evaluation.loader_identity_status,
        evaluation.loader_identity_reason,
    );
    emit_module_loader_artifact_hash_binding_gate(
        &mut wrote,
        "artifact_hash_binding",
        evaluation.artifact_hash_binding_status,
        evaluation.artifact_hash_binding_reason,
    );
    crlf();
    raw_line("      ]");
    end_response("module.loader_artifact_hash_binding");
}

pub(crate) fn emit_module_loader_artifact_hash_binding_selftest() {
    let cases = module_loader_artifact_hash_binding_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.loader_artifact_hash_binding_selftest");
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_loader_artifact_hash_binding_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("accepts_loader_descriptor", no()),
            f("accepts_artifact_bytes", no()),
            f("loads_artifact", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
        ],
        6,
    );
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_loader_artifact_hash_binding_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.loader_artifact_hash_binding_selftest");
}

fn emit_module_loader_artifact_hash_binding_required_bindings(
    candidate: ModuleLoaderArtifactHashBindingCandidate,
    evaluation: ModuleLoaderArtifactHashBindingEvaluation,
    loader_identity_source_evidence_present: bool,
) {
    emit_record_property_line(
        "required_bindings",
        vec![
            f(
                "retained_module_evidence",
                s(evaluation.retained_module_evidence_status),
            ),
            f(
                "service_slot_allocator_readiness",
                s(evaluation.service_slot_allocator_readiness_status),
            ),
            f(
                "service_slot_allocator_runtime",
                s(evaluation.service_slot_allocator_runtime_status),
            ),
            f(
                "audit_rollback_write_boundary",
                s(evaluation.audit_rollback_write_boundary_status),
            ),
            f("loader_identity", s(evaluation.loader_identity_status)),
            f(
                "loader_identity_source_evidence_present",
                b(loader_identity_source_evidence_present),
            ),
            f(
                "loader_identity_available",
                b(candidate.loader_identity_present),
            ),
            f(
                "artifact_hash_binding_fact_present",
                b(candidate.artifact_hash_binding.present),
            ),
        ],
        false,
    );
}

fn emit_module_loader_artifact_hash_binding_source_evidence(
    event_id: event_log::EventId,
    evidence: event_log::ModuleLoaderArtifactHashBindingSourceEvidence,
) {
    emit_record_property_line(
        "source_evidence",
        vec![
            f("schema", s(evidence.schema)),
            f("state", s("retained")),
            f("status", s("retained_current_boot_source_evidence")),
            f(
                "reason",
                s("module_loader_artifact_hash_binding_source_evidence_recorded"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("retention", s("current_boot_ram_event_log")),
            f("event_id", record_event_or_null(Some(event_id))),
            f("fact_schema", s(evidence.fact_schema)),
            f("fact_id", s(evidence.fact_id)),
            f("source_method", s(evidence.source_method)),
            f("source_fact_locator", s(evidence.source_fact_locator)),
            f("readiness_status", s(evidence.readiness_status)),
            f("readiness_reason", s(evidence.readiness_reason)),
            f(
                "artifact_hash_binding_status",
                s(evidence.artifact_hash_binding_status),
            ),
            f(
                "artifact_hash_binding_reason",
                s(evidence.artifact_hash_binding_reason),
            ),
            f(
                "artifact_hash_binding_present",
                b(evidence.artifact_hash_binding_present),
            ),
            f(
                "retained_module_evidence_present",
                b(evidence.retained_module_evidence_present),
            ),
            f(
                "service_slot_allocator_readiness_present",
                b(evidence.service_slot_allocator_readiness_present),
            ),
            f(
                "service_slot_allocator_ready",
                b(evidence.service_slot_allocator_ready),
            ),
            f(
                "audit_rollback_write_boundary_present",
                b(evidence.audit_rollback_write_boundary_present),
            ),
            f(
                "loader_identity_present",
                b(evidence.loader_identity_present),
            ),
            f("binds_loader_identity", b(evidence.binds_loader_identity)),
            f(
                "loader_identity_source_evidence_event_id",
                record_event_or_null(evidence.loader_identity_source_evidence_event_id),
            ),
            f("accepts_loader_descriptor", no()),
            f("accepts_artifact_bytes", no()),
            f("loads_artifact", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
            f("authorizes_load", no()),
        ],
        false,
    );
}

fn emit_module_loader_artifact_hash_binding_fact(
    fact: ModuleLoaderArtifactHashBindingFact,
    evaluation: ModuleLoaderArtifactHashBindingEvaluation,
    source_evidence_event_id: event_log::EventId,
) {
    emit_record_property_line(
        "artifact_hash_binding",
        vec![
            f("schema", s("raios.module_loader_artifact_hash_binding.v0")),
            f("state", s(if fact.present { "present" } else { "missing" })),
            f("status", s(evaluation.artifact_hash_binding_status)),
            f("reason", s(evaluation.artifact_hash_binding_reason)),
            f("scope", s("current_boot")),
            f("fact_scope", s(fact.scope)),
            f("schema_valid", b(fact.schema_ok)),
            f("classification", s(fact.classification)),
            f("provenance_valid", b(fact.provenance_ok)),
            f(
                "binds_retained_module_evidence",
                b(fact.binds_retained_module_evidence),
            ),
            f(
                "binds_service_slot_allocator",
                b(fact.binds_service_slot_allocator),
            ),
            f(
                "binds_audit_rollback_write_boundary",
                b(fact.binds_audit_rollback_write_boundary),
            ),
            f("binds_loader_identity", b(fact.binds_loader_identity)),
            f(
                "fact_id",
                s("module.loader_runtime.artifact_hash_binding.current_boot"),
            ),
            f("source_method", s("module.loader_artifact_hash_binding")),
            f(
                "source_fact_locator",
                s("module.loader_artifact_hash_binding.artifact_hash_binding"),
            ),
            f(
                "source_evidence_event_id",
                record_event_or_null(Some(source_evidence_event_id)),
            ),
            f(
                "source_evidence_schema",
                s("raios.module_loader_artifact_hash_binding_source_evidence.v0"),
            ),
            f("source_evidence_state", s("retained_current_boot")),
            f("persistence", s("none")),
            f("durable", no()),
            f("loads_artifact", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("authorizes_load", no()),
        ],
        false,
    );
}

fn emit_module_loader_artifact_hash_binding_policy_result(
    candidate: ModuleLoaderArtifactHashBindingCandidate,
    evaluation: ModuleLoaderArtifactHashBindingEvaluation,
) {
    emit_record_property_line(
        "policy_result",
        vec![
            f("readiness_status", s(evaluation.status)),
            f("readiness_reason", s(evaluation.reason)),
            f(
                "retained_module_evidence_present",
                b(candidate.retained_module_evidence_present),
            ),
            f(
                "service_slot_allocator_readiness_present",
                b(candidate.service_slot_allocator_readiness_present),
            ),
            f(
                "service_slot_allocator_ready",
                b(candidate.service_slot_allocator_ready),
            ),
            f(
                "audit_rollback_write_boundary_present",
                b(candidate.audit_rollback_write_boundary_present),
            ),
            f(
                "loader_identity_present",
                b(candidate.loader_identity_present),
            ),
            f(
                "artifact_hash_binding_available",
                b(method_eq(
                    evaluation.artifact_hash_binding_status,
                    "available",
                )),
            ),
            f("loads_artifact", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        false,
    );
}

#[rustfmt::skip]
fn emit_module_loader_artifact_hash_binding_gate(
    wrote: &mut bool,
    gate: &'static str,
    state: &'static str,
    reason: &'static str,
) {
    if method_eq(state, "available") {
        return;
    }
    if *wrote {
        raw_line(",");
    } else {
        *wrote = true;
    }
    emit_inline_record_object_fragment(vec![f("gate", s(gate)), f("state", s(state)), f("reason", s(reason))], 8);
}

#[rustfmt::skip]
fn emit_module_loader_artifact_hash_binding_selftest_case(
    case: &ModuleLoaderArtifactHashBindingSelfTestCase,
    comma: bool,
) {
    emit_inline_record_object(vec![f("case", s(case.name)), f("expected_status", s(case.expected_status)), f("expected_reason", s(case.expected_reason)), f("actual_status", s(case.actual_status)), f("actual_reason", s(case.actual_reason)), f("actual_artifact_hash_binding_status", s(case.actual_artifact_hash_binding_status)), f("actual_artifact_hash_binding_reason", s(case.actual_artifact_hash_binding_reason)), f("passed", b(case.passed)), f("loads_artifact", no()), f("allocates_service_slot", no()), f("creates_service_inventory_records", no()), f("can_load", no()), f("load_attempted", no())], comma);
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
