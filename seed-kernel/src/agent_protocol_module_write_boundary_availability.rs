use alloc::vec;

use crate::{
    agent_protocol_module_types::*,
    agent_protocol_module_write_boundary_emit::*,
    agent_protocol_support::{
        begin_response, crlf, emit_export_gate, emit_inline_record_object,
        emit_record_fields_trailing_comma, emit_record_property_line, end_response, method_eq,
        raw_line, record_bool as b, record_false as no, record_field as f, record_str as s,
    },
};
use raios_core::record::Value as V;

pub(crate) fn emit_module_audit_rollback_availability() {
    let availability = module_audit_rollback_availability_snapshot();
    let evaluation = evaluate_module_audit_rollback_availability_candidate(availability);

    begin_response("module.audit_rollback_availability");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.module_audit_rollback_availability.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", no()),
            f("mutates_global_event_log", no()),
            f("global_event_log_mutation", s("none")),
            f("writes_enabled", no()),
            f("creates_durable_audit_records", no()),
            f("creates_rollback_plans", no()),
            f("installs_rollback_plan", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
        ],
        6,
    );
    emit_module_availability_facts(availability, evaluation);
    raw_line(",");
    emit_record_property_line(
        "policy_result",
        vec![
            f("availability_status", s(evaluation.status)),
            f("availability_reason", s(evaluation.reason)),
            f(
                "durable_audit_write_missing",
                b(!method_eq(
                    evaluation.durable_audit_ledger_status,
                    "available",
                )),
            ),
            f(
                "rollback_install_missing",
                b(!method_eq(evaluation.rollback_store_status, "available")),
            ),
            f(
                "durable_write_policy_available",
                b(evaluation.durable_write_policy_available),
            ),
            f(
                "rollback_install_policy_available",
                b(evaluation.rollback_install_policy_available),
            ),
            f("retained_hash_refs_are_durable_authority", no()),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        true,
    );
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

#[rustfmt::skip]
pub(crate) fn emit_module_audit_rollback_availability_selftest() {
    let cases = module_audit_rollback_availability_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.audit_rollback_availability_selftest");
    emit_record_fields_trailing_comma(vec![f("schema", s("raios.module_audit_rollback_availability_selftest.v0")), f("scope", s("current_boot")), f("classification", s("local_only")), f("test_infrastructure", b(true)), f("mutates_global_event_log", no()), f("creates_durable_audit_records", no()), f("creates_rollback_plans", no()), f("installs_rollback_plan", no()), f("service_inventory_change", s("none")), f("load_attempted", no()), f("case_count", V::U64(cases.len() as u64)), f("passed", b(passed))], 6);
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

#[rustfmt::skip]
pub(crate) fn emit_module_audit_rollback_availability_selftest_case(
    case: &ModuleAuditRollbackAvailabilitySelfTestCase,
    comma: bool,
) {
    emit_inline_record_object(vec![f("case", s(case.name)), f("expected_status", s(case.expected_status)), f("expected_reason", s(case.expected_reason)), f("actual_status", s(case.actual_status)), f("actual_reason", s(case.actual_reason)), f("passed", b(case.passed)), f("writes_enabled", no()), f("installs_rollback_plan", no()), f("can_load", no()), f("load_attempted", no())], comma);
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
