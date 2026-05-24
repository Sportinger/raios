use crate::{
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, crlf, end_response, json_str, method_eq, method_head_eq, raw, raw_bool,
        raw_fmt, raw_line,
    },
};

pub(crate) fn module_loader_artifact_hash_binding_method(method: &str) -> bool {
    method_head_eq(method, "module.loader_artifact_hash_binding")
}

pub(crate) fn module_loader_artifact_hash_binding_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.loader_artifact_hash_binding_selftest")
}

pub(crate) fn emit_module_loader_artifact_hash_binding() {
    let candidate = ModuleLoaderArtifactHashBindingCandidate {
        retained_module_evidence_present: false,
        service_slot_allocator_readiness_present: true,
        service_slot_allocator_ready: false,
        audit_rollback_write_boundary_present: false,
        loader_identity_present: false,
        artifact_hash_binding: module_loader_artifact_hash_binding_missing_fact(),
    };
    let evaluation = evaluate_module_loader_artifact_hash_binding_candidate(candidate);

    begin_response("module.loader_artifact_hash_binding");
    raw_line("      \"schema\": \"raios.module_loader_artifact_hash_binding.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"global_event_log_mutation\": \"none\",");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"can_load_now\": false,");
    raw_line("      \"load_attempted\": false,");
    emit_module_loader_artifact_hash_binding_required_bindings(candidate, evaluation);
    raw_line(",");
    emit_module_loader_artifact_hash_binding_fact(candidate.artifact_hash_binding, evaluation);
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
    raw_line("      \"schema\": \"raios.module_loader_artifact_hash_binding_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"can_load_now\": false,");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
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
) {
    raw_line("      \"required_bindings\": {");
    raw("        \"retained_module_evidence\": ");
    json_str(evaluation.retained_module_evidence_status);
    raw_line(",");
    raw("        \"service_slot_allocator_readiness\": ");
    json_str(evaluation.service_slot_allocator_readiness_status);
    raw_line(",");
    raw("        \"service_slot_allocator_runtime\": ");
    json_str(evaluation.service_slot_allocator_runtime_status);
    raw_line(",");
    raw("        \"audit_rollback_write_boundary\": ");
    json_str(evaluation.audit_rollback_write_boundary_status);
    raw_line(",");
    raw("        \"loader_identity\": ");
    json_str(evaluation.loader_identity_status);
    raw_line(",");
    raw("        \"artifact_hash_binding_fact_present\": ");
    raw_bool(candidate.artifact_hash_binding.present);
    crlf();
    raw_line("      }");
}

fn emit_module_loader_artifact_hash_binding_fact(
    fact: ModuleLoaderArtifactHashBindingFact,
    evaluation: ModuleLoaderArtifactHashBindingEvaluation,
) {
    raw_line("      \"artifact_hash_binding\": {");
    raw_line("        \"schema\": \"raios.module_loader_artifact_hash_binding.v0\",");
    raw("        \"state\": ");
    json_str(if fact.present { "present" } else { "missing" });
    raw_line(",");
    raw("        \"status\": ");
    json_str(evaluation.artifact_hash_binding_status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(evaluation.artifact_hash_binding_reason);
    raw_line(",");
    raw_line("        \"scope\": \"current_boot\",");
    raw("        \"fact_scope\": ");
    json_str(fact.scope);
    raw_line(",");
    raw("        \"schema_valid\": ");
    raw_bool(fact.schema_ok);
    raw_line(",");
    raw("        \"classification\": ");
    json_str(fact.classification);
    raw_line(",");
    raw("        \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw_line(",");
    raw("        \"binds_retained_module_evidence\": ");
    raw_bool(fact.binds_retained_module_evidence);
    raw_line(",");
    raw("        \"binds_service_slot_allocator\": ");
    raw_bool(fact.binds_service_slot_allocator);
    raw_line(",");
    raw("        \"binds_audit_rollback_write_boundary\": ");
    raw_bool(fact.binds_audit_rollback_write_boundary);
    raw_line(",");
    raw("        \"binds_loader_identity\": ");
    raw_bool(fact.binds_loader_identity);
    raw_line(",");
    raw_line("        \"fact_id\": \"module.loader_runtime.artifact_hash_binding.current_boot\",");
    raw_line("        \"persistence\": \"none\",");
    raw_line("        \"durable\": false,");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"authorizes_load\": false");
    raw_line("      }");
}

fn emit_module_loader_artifact_hash_binding_policy_result(
    candidate: ModuleLoaderArtifactHashBindingCandidate,
    evaluation: ModuleLoaderArtifactHashBindingEvaluation,
) {
    raw_line("      \"policy_result\": {");
    raw("        \"readiness_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"readiness_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"retained_module_evidence_present\": ");
    raw_bool(candidate.retained_module_evidence_present);
    raw_line(",");
    raw("        \"service_slot_allocator_readiness_present\": ");
    raw_bool(candidate.service_slot_allocator_readiness_present);
    raw_line(",");
    raw("        \"service_slot_allocator_ready\": ");
    raw_bool(candidate.service_slot_allocator_ready);
    raw_line(",");
    raw("        \"audit_rollback_write_boundary_present\": ");
    raw_bool(candidate.audit_rollback_write_boundary_present);
    raw_line(",");
    raw("        \"loader_identity_present\": ");
    raw_bool(candidate.loader_identity_present);
    raw_line(",");
    raw("        \"artifact_hash_binding_available\": ");
    raw_bool(method_eq(
        evaluation.artifact_hash_binding_status,
        "available",
    ));
    raw_line(",");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
}

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
    raw("        {\"gate\": ");
    json_str(gate);
    raw(", \"state\": ");
    json_str(state);
    raw(", \"reason\": ");
    json_str(reason);
    raw("}");
}

fn emit_module_loader_artifact_hash_binding_selftest_case(
    case: &ModuleLoaderArtifactHashBindingSelfTestCase,
    comma: bool,
) {
    raw("        {\"case\": ");
    json_str(case.name);
    raw(", \"expected_status\": ");
    json_str(case.expected_status);
    raw(", \"expected_reason\": ");
    json_str(case.expected_reason);
    raw(", \"actual_status\": ");
    json_str(case.actual_status);
    raw(", \"actual_reason\": ");
    json_str(case.actual_reason);
    raw(", \"actual_artifact_hash_binding_status\": ");
    json_str(case.actual_artifact_hash_binding_status);
    raw(", \"actual_artifact_hash_binding_reason\": ");
    json_str(case.actual_artifact_hash_binding_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"loads_artifact\": false, \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
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
        if candidate.service_slot_allocator_readiness_present {
            ("available", "service_slot_allocator_readiness_available")
        } else {
            ("missing", "service_slot_allocator_readiness_missing")
        };
    let (service_slot_allocator_runtime_status, service_slot_allocator_runtime_reason) =
        if candidate.service_slot_allocator_ready {
            ("available", "service_slot_allocator_runtime_available")
        } else {
            ("missing", "service_slot_allocator_runtime_missing")
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
            "denied_missing_service_slot_allocator_runtime",
            service_slot_allocator_runtime_reason,
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
