use crate::{
    agent_protocol_module_types::*, agent_protocol_module_write_boundary_emit::*,
    agent_protocol_support::*,
};

pub(crate) fn emit_module_audit_rollback_availability() {
    let availability = module_audit_rollback_availability_snapshot();
    let evaluation = evaluate_module_audit_rollback_availability_candidate(availability);

    begin_response("module.audit_rollback_availability");
    raw_line("      \"schema\": \"raios.module_audit_rollback_availability.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"global_event_log_mutation\": \"none\",");
    raw_line("      \"writes_enabled\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    emit_module_availability_facts(availability, evaluation);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"availability_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"availability_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"durable_audit_write_missing\": ");
    raw_bool(!method_eq(
        evaluation.durable_audit_ledger_status,
        "available",
    ));
    raw_line(",");
    raw("        \"rollback_install_missing\": ");
    raw_bool(!method_eq(evaluation.rollback_store_status, "available"));
    raw_line(",");
    raw("        \"durable_write_policy_available\": ");
    raw_bool(evaluation.durable_write_policy_available);
    raw_line(",");
    raw("        \"rollback_install_policy_available\": ");
    raw_bool(evaluation.rollback_install_policy_available);
    raw_line(",");
    raw_line("        \"retained_hash_refs_are_durable_authority\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    emit_export_gate(
        &mut wrote,
        "durable_audit_ledger",
        evaluation.durable_audit_ledger_status,
        evaluation.durable_audit_ledger_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_store",
        evaluation.rollback_store_status,
        evaluation.rollback_store_reason,
    );
    crlf();
    raw_line("      ]");
    end_response("module.audit_rollback_availability");
}

pub(crate) fn emit_module_audit_rollback_availability_selftest() {
    let cases = module_audit_rollback_availability_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.audit_rollback_availability_selftest");
    raw_line("      \"schema\": \"raios.module_audit_rollback_availability_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
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
        emit_module_audit_rollback_availability_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.audit_rollback_availability_selftest");
}

pub(crate) fn emit_module_audit_rollback_availability_selftest_case(
    case: &ModuleAuditRollbackAvailabilitySelfTestCase,
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
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"writes_enabled\": false, \"installs_rollback_plan\": false, \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn module_audit_rollback_availability_snapshot(
) -> ModuleAuditRollbackAvailabilityCandidate {
    ModuleAuditRollbackAvailabilityCandidate {
        durable_audit_ledger: module_audit_rollback_missing_availability_fact(),
        rollback_store: module_audit_rollback_missing_availability_fact(),
        durable_write_policy_available: false,
        rollback_install_policy_available: false,
    }
}

pub(crate) fn module_audit_rollback_missing_availability_fact(
) -> ModuleAuditRollbackAvailabilityFact {
    ModuleAuditRollbackAvailabilityFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
    }
}

pub(crate) fn module_audit_rollback_available_availability_fact(
) -> ModuleAuditRollbackAvailabilityFact {
    ModuleAuditRollbackAvailabilityFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
    }
}

pub(crate) fn evaluate_module_audit_rollback_availability_candidate(
    candidate: ModuleAuditRollbackAvailabilityCandidate,
) -> ModuleAuditRollbackAvailabilityEvaluation {
    let (durable_audit_ledger_status, durable_audit_ledger_reason) =
        evaluate_module_availability_fact(
            candidate.durable_audit_ledger,
            "durable_audit_ledger_scope_must_be_current_boot",
            "durable_audit_ledger_schema_mismatch",
            "durable_audit_ledger_provenance_missing",
            "durable_audit_ledger_missing",
            "durable_audit_ledger_available",
        );
    let (rollback_store_status, rollback_store_reason) = evaluate_module_availability_fact(
        candidate.rollback_store,
        "rollback_store_scope_must_be_current_boot",
        "rollback_store_schema_mismatch",
        "rollback_store_provenance_missing",
        "rollback_store_missing",
        "rollback_store_available",
    );

    let (status, reason) = if method_eq(durable_audit_ledger_status, "rejected") {
        ("rejected", durable_audit_ledger_reason)
    } else if method_eq(rollback_store_status, "rejected") {
        ("rejected", rollback_store_reason)
    } else if method_eq(durable_audit_ledger_status, "missing")
        && method_eq(rollback_store_status, "missing")
    {
        (
            "missing",
            "durable_audit_ledger_missing_and_rollback_store_missing",
        )
    } else if method_eq(durable_audit_ledger_status, "missing") {
        ("missing", durable_audit_ledger_reason)
    } else if method_eq(rollback_store_status, "missing") {
        ("missing", rollback_store_reason)
    } else if !candidate.durable_write_policy_available {
        (
            "denied_missing_durable_write_policy",
            "durable_write_policy_missing",
        )
    } else if !candidate.rollback_install_policy_available {
        (
            "denied_missing_rollback_install_policy",
            "rollback_install_policy_missing",
        )
    } else {
        (
            "denied_write_path_unimplemented",
            "durable_audit_rollback_writer_unimplemented",
        )
    };

    ModuleAuditRollbackAvailabilityEvaluation {
        status,
        reason,
        durable_audit_ledger_status,
        durable_audit_ledger_reason,
        rollback_store_status,
        rollback_store_reason,
        durable_write_policy_available: candidate.durable_write_policy_available,
        rollback_install_policy_available: candidate.rollback_install_policy_available,
        writes_enabled: false,
        installs_rollback_plan: false,
        can_load: false,
        load_attempted: false,
    }
}

pub(crate) fn evaluate_module_availability_fact(
    fact: ModuleAuditRollbackAvailabilityFact,
    scope_reason: &'static str,
    schema_reason: &'static str,
    provenance_reason: &'static str,
    missing_reason: &'static str,
    available_reason: &'static str,
) -> (&'static str, &'static str) {
    if !method_eq(fact.scope, "current_boot") {
        return ("rejected", scope_reason);
    }
    if !fact.schema_ok {
        return ("rejected", schema_reason);
    }
    if !fact.present {
        return ("missing", missing_reason);
    }
    if !fact.provenance_ok {
        return ("rejected", provenance_reason);
    }
    ("available", available_reason)
}

pub(crate) fn module_audit_rollback_availability_selftest_cases(
) -> [ModuleAuditRollbackAvailabilitySelfTestCase; MODULE_AUDIT_ROLLBACK_AVAILABILITY_SELFTEST_CASES]
{
    let missing = module_audit_rollback_availability_snapshot();
    let available_fact = module_audit_rollback_available_availability_fact();
    [
        module_audit_rollback_availability_selftest_case(
            "missing_ledger_and_store_current_boot",
            "missing",
            "durable_audit_ledger_missing_and_rollback_store_missing",
            missing,
        ),
        module_audit_rollback_availability_selftest_case(
            "durable_audit_ledger_previous_boot",
            "rejected",
            "durable_audit_ledger_scope_must_be_current_boot",
            ModuleAuditRollbackAvailabilityCandidate {
                durable_audit_ledger: ModuleAuditRollbackAvailabilityFact {
                    scope: "previous_boot",
                    ..available_fact
                },
                rollback_store: available_fact,
                ..missing
            },
        ),
        module_audit_rollback_availability_selftest_case(
            "durable_audit_ledger_wrong_schema",
            "rejected",
            "durable_audit_ledger_schema_mismatch",
            ModuleAuditRollbackAvailabilityCandidate {
                durable_audit_ledger: ModuleAuditRollbackAvailabilityFact {
                    schema_ok: false,
                    ..available_fact
                },
                rollback_store: available_fact,
                ..missing
            },
        ),
        module_audit_rollback_availability_selftest_case(
            "durable_audit_ledger_provenance_missing",
            "rejected",
            "durable_audit_ledger_provenance_missing",
            ModuleAuditRollbackAvailabilityCandidate {
                durable_audit_ledger: ModuleAuditRollbackAvailabilityFact {
                    provenance_ok: false,
                    ..available_fact
                },
                rollback_store: available_fact,
                ..missing
            },
        ),
        module_audit_rollback_availability_selftest_case(
            "rollback_store_previous_boot",
            "rejected",
            "rollback_store_scope_must_be_current_boot",
            ModuleAuditRollbackAvailabilityCandidate {
                durable_audit_ledger: available_fact,
                rollback_store: ModuleAuditRollbackAvailabilityFact {
                    scope: "previous_boot",
                    ..available_fact
                },
                ..missing
            },
        ),
        module_audit_rollback_availability_selftest_case(
            "rollback_store_wrong_schema",
            "rejected",
            "rollback_store_schema_mismatch",
            ModuleAuditRollbackAvailabilityCandidate {
                durable_audit_ledger: available_fact,
                rollback_store: ModuleAuditRollbackAvailabilityFact {
                    schema_ok: false,
                    ..available_fact
                },
                ..missing
            },
        ),
        module_audit_rollback_availability_selftest_case(
            "rollback_store_provenance_missing",
            "rejected",
            "rollback_store_provenance_missing",
            ModuleAuditRollbackAvailabilityCandidate {
                durable_audit_ledger: available_fact,
                rollback_store: ModuleAuditRollbackAvailabilityFact {
                    provenance_ok: false,
                    ..available_fact
                },
                ..missing
            },
        ),
        module_audit_rollback_availability_selftest_case(
            "available_facts_policy_still_denied",
            "denied_missing_durable_write_policy",
            "durable_write_policy_missing",
            ModuleAuditRollbackAvailabilityCandidate {
                durable_audit_ledger: available_fact,
                rollback_store: available_fact,
                ..missing
            },
        ),
    ]
}

pub(crate) fn module_audit_rollback_availability_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleAuditRollbackAvailabilityCandidate,
) -> ModuleAuditRollbackAvailabilitySelfTestCase {
    let actual = evaluate_module_audit_rollback_availability_candidate(candidate);
    ModuleAuditRollbackAvailabilitySelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.writes_enabled
            && !actual.installs_rollback_plan
            && !actual.can_load
            && !actual.load_attempted,
    }
}
