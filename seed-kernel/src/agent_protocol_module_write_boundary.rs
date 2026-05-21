use crate::{
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, crlf, emit_export_gate, end_response, json_event_id_option,
        json_sha256_option, json_str, method_eq, method_head_eq, raw, raw_bool, raw_fmt, raw_line,
    },
    event_log,
};

pub(crate) fn module_audit_rollback_availability_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_availability")
        || method_head_eq(method, "module.audit_rollback_store_availability")
}

pub(crate) fn module_audit_rollback_availability_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_availability_selftest")
        || method_head_eq(method, "module.audit_rollback_store_availability_selftest")
}

pub(crate) fn module_audit_rollback_write_policy_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_write_policy")
        || method_head_eq(method, "module.audit_rollback_policy")
}

pub(crate) fn module_audit_rollback_write_policy_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_write_policy_selftest")
        || method_head_eq(method, "module.audit_rollback_policy_selftest")
}

pub(crate) fn module_audit_rollback_storage_layout_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_storage_layout")
        || method_head_eq(method, "module.audit_rollback_persistence_layout")
}

pub(crate) fn module_audit_rollback_storage_layout_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_storage_layout_selftest")
        || method_head_eq(method, "module.audit_rollback_persistence_layout_selftest")
}

pub(crate) fn module_audit_rollback_append_engine_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_append_engine")
        || method_head_eq(method, "module.audit_rollback_append_engine_readiness")
}

pub(crate) fn module_audit_rollback_append_engine_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_append_engine_selftest")
        || method_head_eq(
            method,
            "module.audit_rollback_append_engine_readiness_selftest",
        )
}

pub(crate) fn module_audit_rollback_append_contract_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_append_contract")
        || method_head_eq(method, "module.audit_rollback_storage_contract")
}

pub(crate) fn module_audit_rollback_append_contract_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_append_contract_selftest")
        || method_head_eq(method, "module.audit_rollback_storage_contract_selftest")
}

pub(crate) fn module_audit_rollback_append_intent_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_append_intent")
        || method_head_eq(method, "module.audit_rollback_append_request")
}

pub(crate) fn module_audit_rollback_append_intent_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_append_intent_selftest")
        || method_head_eq(method, "module.audit_rollback_append_request_selftest")
}

pub(crate) fn module_audit_rollback_write_boundary_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_write_boundary")
        || method_head_eq(method, "module.audit_rollback_write_gate")
}

pub(crate) fn module_audit_rollback_write_boundary_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.audit_rollback_write_boundary_selftest")
        || method_head_eq(method, "module.audit_rollback_write_gate_selftest")
}

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

fn emit_module_audit_rollback_availability_selftest_case(
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

fn module_audit_rollback_availability_snapshot() -> ModuleAuditRollbackAvailabilityCandidate {
    ModuleAuditRollbackAvailabilityCandidate {
        durable_audit_ledger: module_audit_rollback_missing_availability_fact(),
        rollback_store: module_audit_rollback_missing_availability_fact(),
        durable_write_policy_available: false,
        rollback_install_policy_available: false,
    }
}

fn module_audit_rollback_missing_availability_fact() -> ModuleAuditRollbackAvailabilityFact {
    ModuleAuditRollbackAvailabilityFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
    }
}

fn module_audit_rollback_available_availability_fact() -> ModuleAuditRollbackAvailabilityFact {
    ModuleAuditRollbackAvailabilityFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
    }
}

fn evaluate_module_audit_rollback_availability_candidate(
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

fn evaluate_module_availability_fact(
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

fn module_audit_rollback_availability_selftest_cases(
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

fn module_audit_rollback_availability_selftest_case(
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

pub(crate) fn emit_module_audit_rollback_write_policy() {
    let policy = module_audit_rollback_write_policy_snapshot();
    let evaluation = evaluate_module_audit_rollback_write_policy_candidate(policy);

    begin_response("module.audit_rollback_write_policy");
    raw_line("      \"schema\": \"raios.module_audit_rollback_write_policy.v0\",");
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
    emit_module_write_policy_facts(policy, evaluation);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"policy_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"policy_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"durable_write_policy_missing\": ");
    raw_bool(!method_eq(
        evaluation.durable_write_policy_status,
        "available",
    ));
    raw_line(",");
    raw("        \"rollback_install_policy_missing\": ");
    raw_bool(!method_eq(
        evaluation.rollback_install_policy_status,
        "available",
    ));
    raw_line(",");
    raw_line("        \"retained_hash_refs_are_policy_authority\": false,");
    raw_line("        \"availability_facts_are_policy_authority\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    emit_export_gate(
        &mut wrote,
        "durable_write_policy",
        evaluation.durable_write_policy_status,
        evaluation.durable_write_policy_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_install_policy",
        evaluation.rollback_install_policy_status,
        evaluation.rollback_install_policy_reason,
    );
    crlf();
    raw_line("      ]");
    end_response("module.audit_rollback_write_policy");
}

pub(crate) fn emit_module_audit_rollback_write_policy_selftest() {
    let cases = module_audit_rollback_write_policy_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.audit_rollback_write_policy_selftest");
    raw_line("      \"schema\": \"raios.module_audit_rollback_write_policy_selftest.v0\",");
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
        emit_module_audit_rollback_write_policy_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.audit_rollback_write_policy_selftest");
}

fn emit_module_audit_rollback_write_policy_selftest_case(
    case: &ModuleAuditRollbackWritePolicySelfTestCase,
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

fn module_audit_rollback_write_policy_snapshot() -> ModuleAuditRollbackWritePolicyCandidate {
    ModuleAuditRollbackWritePolicyCandidate {
        durable_write_policy: module_audit_rollback_missing_write_policy_fact(),
        rollback_install_policy: module_audit_rollback_missing_write_policy_fact(),
    }
}

fn module_audit_rollback_missing_write_policy_fact() -> ModuleAuditRollbackWritePolicyFact {
    ModuleAuditRollbackWritePolicyFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_retained_evidence: false,
        binds_availability: false,
    }
}

fn module_audit_rollback_available_write_policy_fact() -> ModuleAuditRollbackWritePolicyFact {
    ModuleAuditRollbackWritePolicyFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_retained_evidence: true,
        binds_availability: true,
    }
}

fn evaluate_module_audit_rollback_write_policy_candidate(
    candidate: ModuleAuditRollbackWritePolicyCandidate,
) -> ModuleAuditRollbackWritePolicyEvaluation {
    let (durable_write_policy_status, durable_write_policy_reason) =
        evaluate_module_write_policy_fact(
            candidate.durable_write_policy,
            "durable_write_policy_scope_must_be_current_boot",
            "durable_write_policy_schema_mismatch",
            "durable_write_policy_missing",
            "durable_write_policy_provenance_missing",
            "durable_write_policy_retained_evidence_binding_missing",
            "durable_write_policy_availability_binding_missing",
            "durable_write_policy_available",
        );
    let (rollback_install_policy_status, rollback_install_policy_reason) =
        evaluate_module_write_policy_fact(
            candidate.rollback_install_policy,
            "rollback_install_policy_scope_must_be_current_boot",
            "rollback_install_policy_schema_mismatch",
            "rollback_install_policy_missing",
            "rollback_install_policy_provenance_missing",
            "rollback_install_policy_retained_evidence_binding_missing",
            "rollback_install_policy_availability_binding_missing",
            "rollback_install_policy_available",
        );

    let (status, reason) = if method_eq(durable_write_policy_status, "rejected") {
        ("rejected", durable_write_policy_reason)
    } else if method_eq(rollback_install_policy_status, "rejected") {
        ("rejected", rollback_install_policy_reason)
    } else if method_eq(durable_write_policy_status, "missing")
        && method_eq(rollback_install_policy_status, "missing")
    {
        (
            "missing",
            "durable_write_policy_missing_and_rollback_install_policy_missing",
        )
    } else if method_eq(durable_write_policy_status, "missing") {
        ("missing", durable_write_policy_reason)
    } else if method_eq(rollback_install_policy_status, "missing") {
        ("missing", rollback_install_policy_reason)
    } else {
        (
            "denied_write_path_unimplemented",
            "durable_audit_rollback_writer_unimplemented",
        )
    };

    ModuleAuditRollbackWritePolicyEvaluation {
        status,
        reason,
        durable_write_policy_status,
        durable_write_policy_reason,
        rollback_install_policy_status,
        rollback_install_policy_reason,
        writes_enabled: false,
        installs_rollback_plan: false,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_write_policy_fact(
    fact: ModuleAuditRollbackWritePolicyFact,
    scope_reason: &'static str,
    schema_reason: &'static str,
    missing_reason: &'static str,
    provenance_reason: &'static str,
    retained_evidence_reason: &'static str,
    availability_reason: &'static str,
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
    if !fact.binds_retained_evidence {
        return ("rejected", retained_evidence_reason);
    }
    if !fact.binds_availability {
        return ("rejected", availability_reason);
    }
    ("available", available_reason)
}

fn emit_module_write_policy_facts(
    policy: ModuleAuditRollbackWritePolicyCandidate,
    evaluation: ModuleAuditRollbackWritePolicyEvaluation,
) {
    raw_line("      \"write_policy_facts\": {");
    emit_module_write_policy_fact(
        "durable_write_policy",
        "raios.durable_audit_write_policy.v0",
        "policy.durable_audit_write.current_boot",
        policy.durable_write_policy,
        evaluation.durable_write_policy_status,
        evaluation.durable_write_policy_reason,
        true,
    );
    emit_module_write_policy_fact(
        "rollback_install_policy",
        "raios.rollback_install_policy.v0",
        "policy.rollback_install.current_boot",
        policy.rollback_install_policy,
        evaluation.rollback_install_policy_status,
        evaluation.rollback_install_policy_reason,
        false,
    );
    raw_line("      }");
}

fn emit_module_write_policy_fact(
    name: &'static str,
    schema: &'static str,
    id: &'static str,
    fact: ModuleAuditRollbackWritePolicyFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw_line(": {");
    raw("          \"schema\": ");
    json_str(schema);
    raw_line(",");
    raw("          \"id\": ");
    json_str(id);
    raw_line(",");
    raw("          \"scope\": ");
    json_str(fact.scope);
    raw_line(",");
    raw("          \"classification\": ");
    json_str(fact.classification);
    raw_line(",");
    raw("          \"status\": ");
    json_str(status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(fact.present);
    raw_line(",");
    raw("          \"schema_valid\": ");
    raw_bool(fact.schema_ok);
    raw_line(",");
    raw("          \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw_line(",");
    raw("          \"binds_retained_evidence\": ");
    raw_bool(fact.binds_retained_evidence);
    raw_line(",");
    raw("          \"binds_availability\": ");
    raw_bool(fact.binds_availability);
    raw_line(",");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"authorizes_write\": false,");
    raw_line("          \"required_bindings\": {");
    raw_line("            \"module_manifest\": \"raios.module_manifest_reference.v0\",");
    raw_line(
        "            \"candidate_artifact\": \"raios.module_candidate_artifact_reference.v0\",",
    );
    raw_line("            \"vm_test_report\": \"raios.module_vm_test_report_reference.v0\",");
    raw_line("            \"computed_capability_grant\": \"raios.computed_capability_grant.v0\",");
    raw_line("            \"local_attestation\": \"raios.module_local_attestation_reference.v0\",");
    raw_line("            \"local_approval\": \"raios.module_local_approval_reference.v0\",");
    raw_line("            \"audit_rollback\": \"raios.module_audit_rollback_reference.v0\",");
    raw_line(
        "            \"service_slot_reservation\": \"raios.module_service_slot_reservation.v0\",",
    );
    raw_line("            \"durable_audit_ledger\": \"raios.durable_audit_ledger.v0\",");
    raw_line("            \"rollback_store\": \"raios.rollback_store.v0\"");
    raw_line("          },");
    raw_line("          \"provenance\": {");
    raw_line("            \"source_method\": \"module.audit_rollback_write_policy\",");
    raw_line("            \"source_transport\": \"serial-console\",");
    raw_line("            \"event_scope\": \"current_boot\",");
    raw_line("            \"record_id\": null");
    raw_line("          }");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

fn module_audit_rollback_write_policy_selftest_cases(
) -> [ModuleAuditRollbackWritePolicySelfTestCase; MODULE_AUDIT_ROLLBACK_WRITE_POLICY_SELFTEST_CASES]
{
    let missing = module_audit_rollback_write_policy_snapshot();
    let available = module_audit_rollback_available_write_policy_fact();
    [
        module_audit_rollback_write_policy_selftest_case(
            "missing_policy_pair_current_boot",
            "missing",
            "durable_write_policy_missing_and_rollback_install_policy_missing",
            missing,
        ),
        module_audit_rollback_write_policy_selftest_case(
            "durable_write_policy_previous_boot",
            "rejected",
            "durable_write_policy_scope_must_be_current_boot",
            ModuleAuditRollbackWritePolicyCandidate {
                durable_write_policy: ModuleAuditRollbackWritePolicyFact {
                    scope: "previous_boot",
                    ..available
                },
                rollback_install_policy: available,
            },
        ),
        module_audit_rollback_write_policy_selftest_case(
            "durable_write_policy_wrong_schema",
            "rejected",
            "durable_write_policy_schema_mismatch",
            ModuleAuditRollbackWritePolicyCandidate {
                durable_write_policy: ModuleAuditRollbackWritePolicyFact {
                    schema_ok: false,
                    ..available
                },
                rollback_install_policy: available,
            },
        ),
        module_audit_rollback_write_policy_selftest_case(
            "durable_write_policy_provenance_missing",
            "rejected",
            "durable_write_policy_provenance_missing",
            ModuleAuditRollbackWritePolicyCandidate {
                durable_write_policy: ModuleAuditRollbackWritePolicyFact {
                    provenance_ok: false,
                    ..available
                },
                rollback_install_policy: available,
            },
        ),
        module_audit_rollback_write_policy_selftest_case(
            "durable_write_policy_retained_evidence_binding_missing",
            "rejected",
            "durable_write_policy_retained_evidence_binding_missing",
            ModuleAuditRollbackWritePolicyCandidate {
                durable_write_policy: ModuleAuditRollbackWritePolicyFact {
                    binds_retained_evidence: false,
                    ..available
                },
                rollback_install_policy: available,
            },
        ),
        module_audit_rollback_write_policy_selftest_case(
            "durable_write_policy_availability_binding_missing",
            "rejected",
            "durable_write_policy_availability_binding_missing",
            ModuleAuditRollbackWritePolicyCandidate {
                durable_write_policy: ModuleAuditRollbackWritePolicyFact {
                    binds_availability: false,
                    ..available
                },
                rollback_install_policy: available,
            },
        ),
        module_audit_rollback_write_policy_selftest_case(
            "rollback_install_policy_previous_boot",
            "rejected",
            "rollback_install_policy_scope_must_be_current_boot",
            ModuleAuditRollbackWritePolicyCandidate {
                durable_write_policy: available,
                rollback_install_policy: ModuleAuditRollbackWritePolicyFact {
                    scope: "previous_boot",
                    ..available
                },
            },
        ),
        module_audit_rollback_write_policy_selftest_case(
            "rollback_install_policy_wrong_schema",
            "rejected",
            "rollback_install_policy_schema_mismatch",
            ModuleAuditRollbackWritePolicyCandidate {
                durable_write_policy: available,
                rollback_install_policy: ModuleAuditRollbackWritePolicyFact {
                    schema_ok: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_write_policy_selftest_case(
            "rollback_install_policy_provenance_missing",
            "rejected",
            "rollback_install_policy_provenance_missing",
            ModuleAuditRollbackWritePolicyCandidate {
                durable_write_policy: available,
                rollback_install_policy: ModuleAuditRollbackWritePolicyFact {
                    provenance_ok: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_write_policy_selftest_case(
            "rollback_install_policy_retained_evidence_binding_missing",
            "rejected",
            "rollback_install_policy_retained_evidence_binding_missing",
            ModuleAuditRollbackWritePolicyCandidate {
                durable_write_policy: available,
                rollback_install_policy: ModuleAuditRollbackWritePolicyFact {
                    binds_retained_evidence: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_write_policy_selftest_case(
            "rollback_install_policy_availability_binding_missing",
            "rejected",
            "rollback_install_policy_availability_binding_missing",
            ModuleAuditRollbackWritePolicyCandidate {
                durable_write_policy: available,
                rollback_install_policy: ModuleAuditRollbackWritePolicyFact {
                    binds_availability: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_write_policy_selftest_case(
            "available_policy_facts_writer_still_denied",
            "denied_write_path_unimplemented",
            "durable_audit_rollback_writer_unimplemented",
            ModuleAuditRollbackWritePolicyCandidate {
                durable_write_policy: available,
                rollback_install_policy: available,
            },
        ),
    ]
}

fn module_audit_rollback_write_policy_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleAuditRollbackWritePolicyCandidate,
) -> ModuleAuditRollbackWritePolicySelfTestCase {
    let actual = evaluate_module_audit_rollback_write_policy_candidate(candidate);
    ModuleAuditRollbackWritePolicySelfTestCase {
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

pub(crate) fn emit_module_audit_rollback_storage_layout() {
    let storage = module_audit_rollback_storage_layout_snapshot();
    let evaluation = evaluate_module_audit_rollback_storage_layout_candidate(storage);

    begin_response("module.audit_rollback_storage_layout");
    raw_line("      \"schema\": \"raios.module_audit_rollback_storage_layout.v0\",");
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
    emit_module_storage_layout_facts(storage, evaluation);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"storage_layout_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"storage_layout_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"persistence_device_available\": ");
    raw_bool(evaluation.persistence_device_available);
    raw_line(",");
    raw("        \"storage_layout_available\": ");
    raw_bool(evaluation.storage_layout_available);
    raw_line(",");
    raw("        \"storage_layout_missing\": ");
    raw_bool(!evaluation.storage_layout_available);
    raw_line(",");
    raw("        \"append_engine_available\": ");
    raw_bool(evaluation.append_engine_available);
    raw_line(",");
    raw("        \"append_engine_missing\": ");
    raw_bool(!evaluation.append_engine_available);
    raw_line(",");
    raw_line("        \"retained_hash_refs_are_storage_authority\": false,");
    raw_line("        \"availability_facts_are_storage_authority\": false,");
    raw_line("        \"write_policy_facts_are_storage_authority\": false,");
    raw_line("        \"storage_layout_facts_are_append_authority\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    emit_export_gate(
        &mut wrote,
        "persistence_device_inventory",
        evaluation.persistence_device_status,
        evaluation.persistence_device_reason,
    );
    emit_export_gate(
        &mut wrote,
        "audit_rollback_storage_layout",
        evaluation.storage_layout_status,
        evaluation.storage_layout_reason,
    );
    emit_export_gate(
        &mut wrote,
        "append_engine",
        "missing",
        "append_engine_unimplemented",
    );
    crlf();
    raw_line("      ]");
    end_response("module.audit_rollback_storage_layout");
}

pub(crate) fn emit_module_audit_rollback_storage_layout_selftest() {
    let cases = module_audit_rollback_storage_layout_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.audit_rollback_storage_layout_selftest");
    raw_line("      \"schema\": \"raios.module_audit_rollback_storage_layout_selftest.v0\",");
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
        emit_module_audit_rollback_storage_layout_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.audit_rollback_storage_layout_selftest");
}

fn emit_module_audit_rollback_storage_layout_selftest_case(
    case: &ModuleAuditRollbackStorageLayoutSelfTestCase,
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

fn module_audit_rollback_storage_layout_snapshot() -> ModuleAuditRollbackStorageLayoutCandidate {
    ModuleAuditRollbackStorageLayoutCandidate {
        persistence_device_inventory: module_audit_rollback_missing_persistence_device_fact(),
        audit_rollback_storage_layout: module_audit_rollback_missing_storage_layout_fact(),
    }
}

fn module_audit_rollback_missing_persistence_device_fact(
) -> ModuleAuditRollbackPersistenceDeviceFact {
    ModuleAuditRollbackPersistenceDeviceFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        stable_identity: false,
        partition_inventory_available: false,
        write_path_available: false,
    }
}

fn module_audit_rollback_available_persistence_device_fact(
) -> ModuleAuditRollbackPersistenceDeviceFact {
    ModuleAuditRollbackPersistenceDeviceFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        stable_identity: true,
        partition_inventory_available: true,
        write_path_available: true,
    }
}

fn module_audit_rollback_missing_storage_layout_fact() -> ModuleAuditRollbackStorageLayoutFact {
    ModuleAuditRollbackStorageLayoutFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_persistence_device: false,
        has_audit_ledger_region: false,
        has_rollback_store_region: false,
        append_slots_available: false,
        recovery_region_separated: false,
    }
}

fn module_audit_rollback_available_storage_layout_fact() -> ModuleAuditRollbackStorageLayoutFact {
    ModuleAuditRollbackStorageLayoutFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_persistence_device: true,
        has_audit_ledger_region: true,
        has_rollback_store_region: true,
        append_slots_available: true,
        recovery_region_separated: true,
    }
}

fn evaluate_module_audit_rollback_storage_layout_candidate(
    candidate: ModuleAuditRollbackStorageLayoutCandidate,
) -> ModuleAuditRollbackStorageLayoutEvaluation {
    let (persistence_device_status, persistence_device_reason) =
        evaluate_module_persistence_device_fact(candidate.persistence_device_inventory);
    let (storage_layout_status, storage_layout_reason) =
        evaluate_module_storage_layout_fact(candidate.audit_rollback_storage_layout);

    let (status, reason) = if method_eq(persistence_device_status, "rejected") {
        ("rejected", persistence_device_reason)
    } else if method_eq(storage_layout_status, "rejected") {
        ("rejected", storage_layout_reason)
    } else if method_eq(persistence_device_status, "missing")
        && method_eq(
            persistence_device_reason,
            "persistence_device_inventory_missing",
        )
        && method_eq(storage_layout_status, "missing")
        && method_eq(
            storage_layout_reason,
            "audit_rollback_storage_layout_missing",
        )
    {
        (
            "missing",
            "persistence_device_inventory_missing_and_storage_layout_missing",
        )
    } else if method_eq(persistence_device_status, "missing") {
        ("missing", persistence_device_reason)
    } else if method_eq(storage_layout_status, "missing") {
        ("missing", storage_layout_reason)
    } else {
        ("available", "audit_rollback_storage_layout_available")
    };

    let persistence_device_available = method_eq(persistence_device_status, "available");
    let storage_layout_available =
        persistence_device_available && method_eq(storage_layout_status, "available");
    ModuleAuditRollbackStorageLayoutEvaluation {
        status,
        reason,
        persistence_device_status,
        persistence_device_reason,
        storage_layout_status,
        storage_layout_reason,
        persistence_device_available,
        storage_layout_available,
        append_engine_available: false,
        writes_enabled: false,
        installs_rollback_plan: false,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_persistence_device_fact(
    fact: ModuleAuditRollbackPersistenceDeviceFact,
) -> (&'static str, &'static str) {
    if !method_eq(fact.scope, "current_boot") {
        return ("rejected", "persistence_device_scope_must_be_current_boot");
    }
    if !fact.schema_ok {
        return ("rejected", "persistence_device_schema_mismatch");
    }
    if !fact.present {
        return ("missing", "persistence_device_inventory_missing");
    }
    if !fact.provenance_ok {
        return ("rejected", "persistence_device_provenance_missing");
    }
    if !fact.stable_identity {
        return ("rejected", "persistence_device_stable_identity_missing");
    }
    if !fact.partition_inventory_available {
        return ("missing", "persistence_partition_inventory_missing");
    }
    if !fact.write_path_available {
        return ("missing", "persistence_device_write_path_missing");
    }
    ("available", "persistence_device_inventory_available")
}

fn evaluate_module_storage_layout_fact(
    fact: ModuleAuditRollbackStorageLayoutFact,
) -> (&'static str, &'static str) {
    if !method_eq(fact.scope, "current_boot") {
        return (
            "rejected",
            "audit_rollback_storage_layout_scope_must_be_current_boot",
        );
    }
    if !fact.schema_ok {
        return ("rejected", "audit_rollback_storage_layout_schema_mismatch");
    }
    if !fact.present {
        return ("missing", "audit_rollback_storage_layout_missing");
    }
    if !fact.provenance_ok {
        return (
            "rejected",
            "audit_rollback_storage_layout_provenance_missing",
        );
    }
    if !fact.binds_persistence_device {
        return ("rejected", "storage_layout_device_binding_missing");
    }
    if !fact.has_audit_ledger_region {
        return ("missing", "audit_ledger_layout_region_missing");
    }
    if !fact.has_rollback_store_region {
        return ("missing", "rollback_store_layout_region_missing");
    }
    if !fact.append_slots_available {
        return ("missing", "storage_layout_append_slots_missing");
    }
    if !fact.recovery_region_separated {
        return ("rejected", "storage_layout_recovery_boundary_missing");
    }
    ("available", "audit_rollback_storage_layout_available")
}

fn emit_module_storage_layout_facts(
    storage: ModuleAuditRollbackStorageLayoutCandidate,
    evaluation: ModuleAuditRollbackStorageLayoutEvaluation,
) {
    raw_line("      \"storage_layout_facts\": {");
    emit_module_persistence_device_fact(
        storage.persistence_device_inventory,
        evaluation.persistence_device_status,
        evaluation.persistence_device_reason,
        true,
    );
    emit_module_storage_layout_fact(
        storage.audit_rollback_storage_layout,
        evaluation.storage_layout_status,
        evaluation.storage_layout_reason,
        false,
    );
    raw_line("      }");
}

fn emit_module_persistence_device_fact(
    fact: ModuleAuditRollbackPersistenceDeviceFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw_line("        \"persistence_device_inventory\": {");
    raw_line("          \"schema\": \"raios.persistence_device_inventory.v0\",");
    raw_line("          \"id\": \"storage.persistence_device_inventory.current_boot\",");
    raw_line("          \"device_class\": \"persistence_device\",");
    raw_line("          \"device_id\": null,");
    raw("          \"scope\": ");
    json_str(fact.scope);
    raw_line(",");
    raw("          \"classification\": ");
    json_str(fact.classification);
    raw_line(",");
    raw("          \"status\": ");
    json_str(status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(fact.present);
    raw_line(",");
    raw("          \"schema_valid\": ");
    raw_bool(fact.schema_ok);
    raw_line(",");
    raw("          \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw_line(",");
    raw("          \"stable_identity\": ");
    raw_bool(fact.stable_identity);
    raw_line(",");
    raw("          \"partition_inventory_available\": ");
    raw_bool(fact.partition_inventory_available);
    raw_line(",");
    raw("          \"write_path_available\": ");
    raw_bool(fact.write_path_available);
    raw_line(",");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"authorizes_layout\": false,");
    raw_line("          \"write_attempted\": false,");
    raw_line("          \"provenance\": {");
    raw_line("            \"source_method\": \"module.audit_rollback_storage_layout\",");
    raw_line("            \"source_transport\": \"serial-console\",");
    raw_line("            \"event_scope\": \"current_boot\",");
    raw_line("            \"record_id\": null");
    raw_line("          }");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_storage_layout_fact(
    fact: ModuleAuditRollbackStorageLayoutFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw_line("        \"audit_rollback_storage_layout\": {");
    raw_line("          \"schema\": \"raios.audit_rollback_storage_layout.v0\",");
    raw_line("          \"id\": \"storage.audit_rollback_layout.current_boot\",");
    raw("          \"scope\": ");
    json_str(fact.scope);
    raw_line(",");
    raw("          \"classification\": ");
    json_str(fact.classification);
    raw_line(",");
    raw("          \"status\": ");
    json_str(status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(fact.present);
    raw_line(",");
    raw("          \"schema_valid\": ");
    raw_bool(fact.schema_ok);
    raw_line(",");
    raw("          \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw_line(",");
    raw("          \"binds_persistence_device\": ");
    raw_bool(fact.binds_persistence_device);
    raw_line(",");
    raw("          \"has_audit_ledger_region\": ");
    raw_bool(fact.has_audit_ledger_region);
    raw_line(",");
    raw("          \"has_rollback_store_region\": ");
    raw_bool(fact.has_rollback_store_region);
    raw_line(",");
    raw("          \"append_slots_available\": ");
    raw_bool(fact.append_slots_available);
    raw_line(",");
    raw("          \"recovery_region_separated\": ");
    raw_bool(fact.recovery_region_separated);
    raw_line(",");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"authorizes_append\": false,");
    raw_line("          \"write_attempted\": false,");
    raw_line("          \"provenance\": {");
    raw_line("            \"source_method\": \"module.audit_rollback_storage_layout\",");
    raw_line("            \"source_transport\": \"serial-console\",");
    raw_line("            \"event_scope\": \"current_boot\",");
    raw_line("            \"record_id\": null");
    raw_line("          }");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

fn module_audit_rollback_storage_layout_selftest_cases(
) -> [ModuleAuditRollbackStorageLayoutSelfTestCase;
       MODULE_AUDIT_ROLLBACK_STORAGE_LAYOUT_SELFTEST_CASES] {
    let missing = module_audit_rollback_storage_layout_snapshot();
    let available_device = module_audit_rollback_available_persistence_device_fact();
    let available_layout = module_audit_rollback_available_storage_layout_fact();
    [
        module_audit_rollback_storage_layout_selftest_case(
            "missing_storage_inputs_current_boot",
            "missing",
            "persistence_device_inventory_missing_and_storage_layout_missing",
            missing,
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "persistence_device_previous_boot",
            "rejected",
            "persistence_device_scope_must_be_current_boot",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: ModuleAuditRollbackPersistenceDeviceFact {
                    scope: "previous_boot",
                    ..available_device
                },
                audit_rollback_storage_layout: available_layout,
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "persistence_device_wrong_schema",
            "rejected",
            "persistence_device_schema_mismatch",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: ModuleAuditRollbackPersistenceDeviceFact {
                    schema_ok: false,
                    ..available_device
                },
                audit_rollback_storage_layout: available_layout,
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "persistence_device_provenance_missing",
            "rejected",
            "persistence_device_provenance_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: ModuleAuditRollbackPersistenceDeviceFact {
                    provenance_ok: false,
                    ..available_device
                },
                audit_rollback_storage_layout: available_layout,
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "persistence_device_stable_identity_missing",
            "rejected",
            "persistence_device_stable_identity_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: ModuleAuditRollbackPersistenceDeviceFact {
                    stable_identity: false,
                    ..available_device
                },
                audit_rollback_storage_layout: available_layout,
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "persistence_partition_inventory_missing",
            "missing",
            "persistence_partition_inventory_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: ModuleAuditRollbackPersistenceDeviceFact {
                    partition_inventory_available: false,
                    ..available_device
                },
                audit_rollback_storage_layout: available_layout,
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "audit_rollback_storage_layout_previous_boot",
            "rejected",
            "audit_rollback_storage_layout_scope_must_be_current_boot",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    scope: "previous_boot",
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "audit_rollback_storage_layout_wrong_schema",
            "rejected",
            "audit_rollback_storage_layout_schema_mismatch",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    schema_ok: false,
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "audit_rollback_storage_layout_provenance_missing",
            "rejected",
            "audit_rollback_storage_layout_provenance_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    provenance_ok: false,
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "storage_layout_device_binding_missing",
            "rejected",
            "storage_layout_device_binding_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    binds_persistence_device: false,
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "audit_ledger_layout_region_missing",
            "missing",
            "audit_ledger_layout_region_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    has_audit_ledger_region: false,
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "rollback_store_layout_region_missing",
            "missing",
            "rollback_store_layout_region_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    has_rollback_store_region: false,
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "storage_layout_append_slots_missing",
            "missing",
            "storage_layout_append_slots_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    append_slots_available: false,
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "storage_layout_recovery_boundary_missing",
            "rejected",
            "storage_layout_recovery_boundary_missing",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact {
                    recovery_region_separated: false,
                    ..available_layout
                },
            },
        ),
        module_audit_rollback_storage_layout_selftest_case(
            "available_storage_layout_still_non_authorizing",
            "available",
            "audit_rollback_storage_layout_available",
            ModuleAuditRollbackStorageLayoutCandidate {
                persistence_device_inventory: available_device,
                audit_rollback_storage_layout: available_layout,
            },
        ),
    ]
}

fn module_audit_rollback_storage_layout_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleAuditRollbackStorageLayoutCandidate,
) -> ModuleAuditRollbackStorageLayoutSelfTestCase {
    let actual = evaluate_module_audit_rollback_storage_layout_candidate(candidate);
    ModuleAuditRollbackStorageLayoutSelfTestCase {
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

pub(crate) fn emit_module_audit_rollback_append_engine() {
    let storage = module_audit_rollback_storage_layout_snapshot();
    let storage_evaluation = evaluate_module_audit_rollback_storage_layout_candidate(storage);
    let engine = module_audit_rollback_append_engine_snapshot();
    let evaluation = evaluate_module_audit_rollback_append_engine_candidate(engine);

    begin_response("module.audit_rollback_append_engine");
    raw_line("      \"schema\": \"raios.module_audit_rollback_append_engine.v0\",");
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
    emit_module_append_engine_storage_layout_inputs(storage, storage_evaluation);
    raw_line(",");
    emit_module_append_engine_facts(engine, evaluation);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"append_engine_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"append_engine_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"audit_engine_available\": ");
    raw_bool(evaluation.audit_engine_available);
    raw_line(",");
    raw("        \"rollback_engine_available\": ");
    raw_bool(evaluation.rollback_engine_available);
    raw_line(",");
    raw("        \"append_engine_available\": ");
    raw_bool(evaluation.append_engine_available);
    raw_line(",");
    raw("        \"append_engine_missing\": ");
    raw_bool(!evaluation.append_engine_available);
    raw_line(",");
    raw("        \"storage_layout_available\": ");
    raw_bool(storage_evaluation.storage_layout_available);
    raw_line(",");
    raw("        \"storage_layout_missing\": ");
    raw_bool(!storage_evaluation.storage_layout_available);
    raw_line(",");
    raw_line("        \"retained_hash_refs_are_append_engine_authority\": false,");
    raw_line("        \"availability_facts_are_append_engine_authority\": false,");
    raw_line("        \"write_policy_facts_are_append_engine_authority\": false,");
    raw_line("        \"storage_layout_facts_are_append_engine_authority\": false,");
    raw_line("        \"append_engine_facts_are_append_authority\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    emit_export_gate(
        &mut wrote,
        "audit_rollback_storage_layout",
        storage_evaluation.status,
        storage_evaluation.reason,
    );
    emit_export_gate(
        &mut wrote,
        "audit_ledger_append_engine",
        evaluation.audit_engine_status,
        evaluation.audit_engine_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_store_transaction_engine",
        evaluation.rollback_engine_status,
        evaluation.rollback_engine_reason,
    );
    crlf();
    raw_line("      ]");
    end_response("module.audit_rollback_append_engine");
}

pub(crate) fn emit_module_audit_rollback_append_engine_selftest() {
    let cases = module_audit_rollback_append_engine_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.audit_rollback_append_engine_selftest");
    raw_line("      \"schema\": \"raios.module_audit_rollback_append_engine_selftest.v0\",");
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
        emit_module_audit_rollback_append_engine_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.audit_rollback_append_engine_selftest");
}

fn emit_module_audit_rollback_append_engine_selftest_case(
    case: &ModuleAuditRollbackAppendEngineSelfTestCase,
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

fn module_audit_rollback_append_engine_snapshot() -> ModuleAuditRollbackAppendEngineCandidate {
    ModuleAuditRollbackAppendEngineCandidate {
        audit_ledger_append_engine: module_audit_rollback_missing_append_engine_fact(),
        rollback_store_transaction_engine: module_audit_rollback_missing_append_engine_fact(),
    }
}

fn module_audit_rollback_missing_append_engine_fact() -> ModuleAuditRollbackAppendEngineFact {
    ModuleAuditRollbackAppendEngineFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_storage_layout: false,
        binds_write_policy: false,
        supports_append_only: false,
        supports_flush: false,
        supports_replay: false,
        recovery_separation_respected: false,
    }
}

fn module_audit_rollback_available_append_engine_fact() -> ModuleAuditRollbackAppendEngineFact {
    ModuleAuditRollbackAppendEngineFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_storage_layout: true,
        binds_write_policy: true,
        supports_append_only: true,
        supports_flush: true,
        supports_replay: true,
        recovery_separation_respected: true,
    }
}

fn evaluate_module_audit_rollback_append_engine_candidate(
    candidate: ModuleAuditRollbackAppendEngineCandidate,
) -> ModuleAuditRollbackAppendEngineEvaluation {
    let (audit_engine_status, audit_engine_reason) = evaluate_module_append_engine_fact(
        candidate.audit_ledger_append_engine,
        "audit_ledger_append_engine_scope_must_be_current_boot",
        "audit_ledger_append_engine_schema_mismatch",
        "audit_ledger_append_engine_missing",
        "audit_ledger_append_engine_provenance_missing",
        "audit_ledger_append_engine_storage_layout_binding_missing",
        "audit_ledger_append_engine_write_policy_binding_missing",
        "audit_ledger_append_engine_append_only_contract_missing",
        "audit_ledger_append_engine_flush_support_missing",
        "audit_ledger_append_engine_replay_support_missing",
        "audit_ledger_append_engine_recovery_boundary_missing",
        "audit_ledger_append_engine_available",
    );
    let (rollback_engine_status, rollback_engine_reason) = evaluate_module_append_engine_fact(
        candidate.rollback_store_transaction_engine,
        "rollback_store_transaction_engine_scope_must_be_current_boot",
        "rollback_store_transaction_engine_schema_mismatch",
        "rollback_store_transaction_engine_missing",
        "rollback_store_transaction_engine_provenance_missing",
        "rollback_store_transaction_engine_storage_layout_binding_missing",
        "rollback_store_transaction_engine_write_policy_binding_missing",
        "rollback_store_transaction_engine_append_only_contract_missing",
        "rollback_store_transaction_engine_flush_support_missing",
        "rollback_store_transaction_engine_replay_support_missing",
        "rollback_store_transaction_engine_recovery_boundary_missing",
        "rollback_store_transaction_engine_available",
    );

    let (status, reason) = if method_eq(audit_engine_status, "rejected") {
        ("rejected", audit_engine_reason)
    } else if method_eq(rollback_engine_status, "rejected") {
        ("rejected", rollback_engine_reason)
    } else if method_eq(audit_engine_status, "missing")
        && method_eq(audit_engine_reason, "audit_ledger_append_engine_missing")
        && method_eq(rollback_engine_status, "missing")
        && method_eq(
            rollback_engine_reason,
            "rollback_store_transaction_engine_missing",
        )
    {
        (
            "missing",
            "audit_ledger_append_engine_missing_and_rollback_store_transaction_engine_missing",
        )
    } else if method_eq(audit_engine_status, "missing") {
        ("missing", audit_engine_reason)
    } else if method_eq(rollback_engine_status, "missing") {
        ("missing", rollback_engine_reason)
    } else {
        ("available", "audit_rollback_append_engine_available")
    };

    let audit_engine_available = method_eq(audit_engine_status, "available");
    let rollback_engine_available = method_eq(rollback_engine_status, "available");
    ModuleAuditRollbackAppendEngineEvaluation {
        status,
        reason,
        audit_engine_status,
        audit_engine_reason,
        rollback_engine_status,
        rollback_engine_reason,
        audit_engine_available,
        rollback_engine_available,
        append_engine_available: audit_engine_available && rollback_engine_available,
        writes_enabled: false,
        installs_rollback_plan: false,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_append_engine_fact(
    fact: ModuleAuditRollbackAppendEngineFact,
    scope_reason: &'static str,
    schema_reason: &'static str,
    missing_reason: &'static str,
    provenance_reason: &'static str,
    storage_layout_reason: &'static str,
    write_policy_reason: &'static str,
    append_only_reason: &'static str,
    flush_reason: &'static str,
    replay_reason: &'static str,
    recovery_reason: &'static str,
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
    if !fact.binds_storage_layout {
        return ("rejected", storage_layout_reason);
    }
    if !fact.binds_write_policy {
        return ("rejected", write_policy_reason);
    }
    if !fact.supports_append_only {
        return ("missing", append_only_reason);
    }
    if !fact.supports_flush {
        return ("missing", flush_reason);
    }
    if !fact.supports_replay {
        return ("missing", replay_reason);
    }
    if !fact.recovery_separation_respected {
        return ("rejected", recovery_reason);
    }
    ("available", available_reason)
}

fn emit_module_append_engine_storage_layout_inputs(
    storage: ModuleAuditRollbackStorageLayoutCandidate,
    evaluation: ModuleAuditRollbackStorageLayoutEvaluation,
) {
    raw_line("      \"storage_layout_inputs\": {");
    raw_line("        \"persistence_device_inventory\": {");
    raw_line("          \"schema\": \"raios.persistence_device_inventory.v0\",");
    raw("          \"status\": ");
    json_str(evaluation.persistence_device_status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(evaluation.persistence_device_reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(storage.persistence_device_inventory.present);
    raw_line(",");
    raw("          \"stable_identity\": ");
    raw_bool(storage.persistence_device_inventory.stable_identity);
    raw_line(",");
    raw("          \"partition_inventory_available\": ");
    raw_bool(
        storage
            .persistence_device_inventory
            .partition_inventory_available,
    );
    raw_line(",");
    raw("          \"write_path_available\": ");
    raw_bool(storage.persistence_device_inventory.write_path_available);
    raw_line(",");
    raw_line("          \"authorizes_append_engine\": false");
    raw_line("        },");
    raw_line("        \"audit_rollback_storage_layout\": {");
    raw_line("          \"schema\": \"raios.audit_rollback_storage_layout.v0\",");
    raw("          \"status\": ");
    json_str(evaluation.storage_layout_status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(evaluation.storage_layout_reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(storage.audit_rollback_storage_layout.present);
    raw_line(",");
    raw("          \"binds_persistence_device\": ");
    raw_bool(
        storage
            .audit_rollback_storage_layout
            .binds_persistence_device,
    );
    raw_line(",");
    raw("          \"append_slots_available\": ");
    raw_bool(storage.audit_rollback_storage_layout.append_slots_available);
    raw_line(",");
    raw("          \"recovery_region_separated\": ");
    raw_bool(
        storage
            .audit_rollback_storage_layout
            .recovery_region_separated,
    );
    raw_line(",");
    raw_line("          \"authorizes_append_engine\": false");
    raw_line("        },");
    raw("        \"storage_layout_available\": ");
    raw_bool(evaluation.storage_layout_available);
    raw_line(",");
    raw_line("        \"storage_layout_facts_are_append_engine_authority\": false");
    raw_line("      }");
}

fn emit_module_append_engine_facts(
    engine: ModuleAuditRollbackAppendEngineCandidate,
    evaluation: ModuleAuditRollbackAppendEngineEvaluation,
) {
    raw_line("      \"append_engine_facts\": {");
    emit_module_append_engine_fact(
        "audit_ledger_append_engine",
        "raios.audit_ledger_append_engine.v0",
        "append_engine.audit_ledger.current_boot",
        "raios.audit_record.v0",
        engine.audit_ledger_append_engine,
        evaluation.audit_engine_status,
        evaluation.audit_engine_reason,
        true,
    );
    emit_module_append_engine_fact(
        "rollback_store_transaction_engine",
        "raios.rollback_store_transaction_engine.v0",
        "append_engine.rollback_store.current_boot",
        "raios.rollback_plan.v0",
        engine.rollback_store_transaction_engine,
        evaluation.rollback_engine_status,
        evaluation.rollback_engine_reason,
        false,
    );
    raw_line("      }");
}

fn emit_module_append_engine_fact(
    name: &'static str,
    schema: &'static str,
    id: &'static str,
    target_schema: &'static str,
    fact: ModuleAuditRollbackAppendEngineFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw_line(": {");
    raw("          \"schema\": ");
    json_str(schema);
    raw_line(",");
    raw("          \"id\": ");
    json_str(id);
    raw_line(",");
    raw("          \"target_schema\": ");
    json_str(target_schema);
    raw_line(",");
    raw_line("          \"storage_layout_schema\": \"raios.audit_rollback_storage_layout.v0\",");
    raw_line("          \"write_policy_schema\": \"raios.durable_audit_write_policy.v0\",");
    raw("          \"scope\": ");
    json_str(fact.scope);
    raw_line(",");
    raw("          \"classification\": ");
    json_str(fact.classification);
    raw_line(",");
    raw("          \"status\": ");
    json_str(status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(fact.present);
    raw_line(",");
    raw("          \"schema_valid\": ");
    raw_bool(fact.schema_ok);
    raw_line(",");
    raw("          \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw_line(",");
    raw("          \"binds_storage_layout\": ");
    raw_bool(fact.binds_storage_layout);
    raw_line(",");
    raw("          \"binds_write_policy\": ");
    raw_bool(fact.binds_write_policy);
    raw_line(",");
    raw("          \"supports_append_only\": ");
    raw_bool(fact.supports_append_only);
    raw_line(",");
    raw("          \"supports_flush\": ");
    raw_bool(fact.supports_flush);
    raw_line(",");
    raw("          \"supports_replay\": ");
    raw_bool(fact.supports_replay);
    raw_line(",");
    raw("          \"recovery_separation_respected\": ");
    raw_bool(fact.recovery_separation_respected);
    raw_line(",");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"authorizes_append\": false,");
    raw_line("          \"authorizes_write\": false,");
    raw_line("          \"write_attempted\": false,");
    raw_line("          \"provenance\": {");
    raw_line("            \"source_method\": \"module.audit_rollback_append_engine\",");
    raw_line("            \"source_transport\": \"serial-console\",");
    raw_line("            \"event_scope\": \"current_boot\",");
    raw_line("            \"record_id\": null");
    raw_line("          }");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

fn module_audit_rollback_append_engine_selftest_cases(
) -> [ModuleAuditRollbackAppendEngineSelfTestCase; MODULE_AUDIT_ROLLBACK_APPEND_ENGINE_SELFTEST_CASES]
{
    let missing = module_audit_rollback_append_engine_snapshot();
    let available = module_audit_rollback_available_append_engine_fact();
    [
        module_audit_rollback_append_engine_selftest_case(
            "missing_append_engine_pair_current_boot",
            "missing",
            "audit_ledger_append_engine_missing_and_rollback_store_transaction_engine_missing",
            missing,
        ),
        module_audit_rollback_append_engine_selftest_case(
            "audit_ledger_append_engine_previous_boot",
            "rejected",
            "audit_ledger_append_engine_scope_must_be_current_boot",
            ModuleAuditRollbackAppendEngineCandidate {
                audit_ledger_append_engine: ModuleAuditRollbackAppendEngineFact {
                    scope: "previous_boot",
                    ..available
                },
                rollback_store_transaction_engine: available,
            },
        ),
        module_audit_rollback_append_engine_selftest_case(
            "audit_ledger_append_engine_wrong_schema",
            "rejected",
            "audit_ledger_append_engine_schema_mismatch",
            ModuleAuditRollbackAppendEngineCandidate {
                audit_ledger_append_engine: ModuleAuditRollbackAppendEngineFact {
                    schema_ok: false,
                    ..available
                },
                rollback_store_transaction_engine: available,
            },
        ),
        module_audit_rollback_append_engine_selftest_case(
            "audit_ledger_append_engine_provenance_missing",
            "rejected",
            "audit_ledger_append_engine_provenance_missing",
            ModuleAuditRollbackAppendEngineCandidate {
                audit_ledger_append_engine: ModuleAuditRollbackAppendEngineFact {
                    provenance_ok: false,
                    ..available
                },
                rollback_store_transaction_engine: available,
            },
        ),
        module_audit_rollback_append_engine_selftest_case(
            "audit_ledger_append_engine_storage_layout_binding_missing",
            "rejected",
            "audit_ledger_append_engine_storage_layout_binding_missing",
            ModuleAuditRollbackAppendEngineCandidate {
                audit_ledger_append_engine: ModuleAuditRollbackAppendEngineFact {
                    binds_storage_layout: false,
                    ..available
                },
                rollback_store_transaction_engine: available,
            },
        ),
        module_audit_rollback_append_engine_selftest_case(
            "audit_ledger_append_engine_write_policy_binding_missing",
            "rejected",
            "audit_ledger_append_engine_write_policy_binding_missing",
            ModuleAuditRollbackAppendEngineCandidate {
                audit_ledger_append_engine: ModuleAuditRollbackAppendEngineFact {
                    binds_write_policy: false,
                    ..available
                },
                rollback_store_transaction_engine: available,
            },
        ),
        module_audit_rollback_append_engine_selftest_case(
            "audit_ledger_append_engine_append_only_missing",
            "missing",
            "audit_ledger_append_engine_append_only_contract_missing",
            ModuleAuditRollbackAppendEngineCandidate {
                audit_ledger_append_engine: ModuleAuditRollbackAppendEngineFact {
                    supports_append_only: false,
                    ..available
                },
                rollback_store_transaction_engine: available,
            },
        ),
        module_audit_rollback_append_engine_selftest_case(
            "audit_ledger_append_engine_flush_support_missing",
            "missing",
            "audit_ledger_append_engine_flush_support_missing",
            ModuleAuditRollbackAppendEngineCandidate {
                audit_ledger_append_engine: ModuleAuditRollbackAppendEngineFact {
                    supports_flush: false,
                    ..available
                },
                rollback_store_transaction_engine: available,
            },
        ),
        module_audit_rollback_append_engine_selftest_case(
            "audit_ledger_append_engine_recovery_boundary_missing",
            "rejected",
            "audit_ledger_append_engine_recovery_boundary_missing",
            ModuleAuditRollbackAppendEngineCandidate {
                audit_ledger_append_engine: ModuleAuditRollbackAppendEngineFact {
                    recovery_separation_respected: false,
                    ..available
                },
                rollback_store_transaction_engine: available,
            },
        ),
        module_audit_rollback_append_engine_selftest_case(
            "rollback_store_transaction_engine_previous_boot",
            "rejected",
            "rollback_store_transaction_engine_scope_must_be_current_boot",
            ModuleAuditRollbackAppendEngineCandidate {
                audit_ledger_append_engine: available,
                rollback_store_transaction_engine: ModuleAuditRollbackAppendEngineFact {
                    scope: "previous_boot",
                    ..available
                },
            },
        ),
        module_audit_rollback_append_engine_selftest_case(
            "rollback_store_transaction_engine_wrong_schema",
            "rejected",
            "rollback_store_transaction_engine_schema_mismatch",
            ModuleAuditRollbackAppendEngineCandidate {
                audit_ledger_append_engine: available,
                rollback_store_transaction_engine: ModuleAuditRollbackAppendEngineFact {
                    schema_ok: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_engine_selftest_case(
            "rollback_store_transaction_engine_provenance_missing",
            "rejected",
            "rollback_store_transaction_engine_provenance_missing",
            ModuleAuditRollbackAppendEngineCandidate {
                audit_ledger_append_engine: available,
                rollback_store_transaction_engine: ModuleAuditRollbackAppendEngineFact {
                    provenance_ok: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_engine_selftest_case(
            "rollback_store_transaction_engine_storage_layout_binding_missing",
            "rejected",
            "rollback_store_transaction_engine_storage_layout_binding_missing",
            ModuleAuditRollbackAppendEngineCandidate {
                audit_ledger_append_engine: available,
                rollback_store_transaction_engine: ModuleAuditRollbackAppendEngineFact {
                    binds_storage_layout: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_engine_selftest_case(
            "rollback_store_transaction_engine_write_policy_binding_missing",
            "rejected",
            "rollback_store_transaction_engine_write_policy_binding_missing",
            ModuleAuditRollbackAppendEngineCandidate {
                audit_ledger_append_engine: available,
                rollback_store_transaction_engine: ModuleAuditRollbackAppendEngineFact {
                    binds_write_policy: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_engine_selftest_case(
            "rollback_store_transaction_engine_replay_support_missing",
            "missing",
            "rollback_store_transaction_engine_replay_support_missing",
            ModuleAuditRollbackAppendEngineCandidate {
                audit_ledger_append_engine: available,
                rollback_store_transaction_engine: ModuleAuditRollbackAppendEngineFact {
                    supports_replay: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_engine_selftest_case(
            "available_append_engines_still_non_authorizing",
            "available",
            "audit_rollback_append_engine_available",
            ModuleAuditRollbackAppendEngineCandidate {
                audit_ledger_append_engine: available,
                rollback_store_transaction_engine: available,
            },
        ),
    ]
}

fn module_audit_rollback_append_engine_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleAuditRollbackAppendEngineCandidate,
) -> ModuleAuditRollbackAppendEngineSelfTestCase {
    let actual = evaluate_module_audit_rollback_append_engine_candidate(candidate);
    ModuleAuditRollbackAppendEngineSelfTestCase {
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

pub(crate) fn emit_module_audit_rollback_append_contract() {
    let storage = module_audit_rollback_storage_layout_snapshot();
    let storage_evaluation = evaluate_module_audit_rollback_storage_layout_candidate(storage);
    let engine = module_audit_rollback_append_engine_snapshot();
    let engine_evaluation = evaluate_module_audit_rollback_append_engine_candidate(engine);
    let append = module_audit_rollback_append_contract_snapshot_from_storage_and_engine(
        storage_evaluation,
        engine_evaluation,
    );
    let evaluation = evaluate_module_audit_rollback_append_contract_candidate(append);

    begin_response("module.audit_rollback_append_contract");
    raw_line("      \"schema\": \"raios.module_audit_rollback_append_contract.v0\",");
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
    emit_module_append_contract_storage_layout_inputs(storage, storage_evaluation);
    raw_line(",");
    emit_module_append_contract_append_engine_inputs(engine, engine_evaluation);
    raw_line(",");
    emit_module_append_contract_facts(append, evaluation);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"append_contract_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"append_contract_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"audit_append_missing\": ");
    raw_bool(!method_eq(evaluation.audit_append_status, "available"));
    raw_line(",");
    raw("        \"rollback_transaction_missing\": ");
    raw_bool(!method_eq(
        evaluation.rollback_transaction_status,
        "available",
    ));
    raw_line(",");
    raw("        \"storage_layout_available\": ");
    raw_bool(evaluation.storage_layout_available);
    raw_line(",");
    raw("        \"append_engine_available\": ");
    raw_bool(evaluation.append_engine_available);
    raw_line(",");
    raw("        \"storage_layout_missing\": ");
    raw_bool(!evaluation.storage_layout_available);
    raw_line(",");
    raw("        \"append_engine_missing\": ");
    raw_bool(!evaluation.append_engine_available);
    raw_line(",");
    raw_line("        \"append_envelopes_are_durable_authority\": false,");
    raw_line("        \"retained_hash_refs_are_append_authority\": false,");
    raw_line("        \"policy_facts_are_append_authority\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    emit_export_gate(
        &mut wrote,
        "audit_append_envelope",
        evaluation.audit_append_status,
        evaluation.audit_append_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_transaction_envelope",
        evaluation.rollback_transaction_status,
        evaluation.rollback_transaction_reason,
    );
    crlf();
    raw_line("      ]");
    end_response("module.audit_rollback_append_contract");
}

pub(crate) fn emit_module_audit_rollback_append_contract_selftest() {
    let cases = module_audit_rollback_append_contract_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.audit_rollback_append_contract_selftest");
    raw_line("      \"schema\": \"raios.module_audit_rollback_append_contract_selftest.v0\",");
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
        emit_module_audit_rollback_append_contract_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.audit_rollback_append_contract_selftest");
}

fn emit_module_audit_rollback_append_contract_selftest_case(
    case: &ModuleAuditRollbackAppendContractSelfTestCase,
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

fn module_audit_rollback_append_contract_snapshot() -> ModuleAuditRollbackAppendContractCandidate {
    let storage = module_audit_rollback_storage_layout_snapshot();
    let storage_evaluation = evaluate_module_audit_rollback_storage_layout_candidate(storage);
    let engine = module_audit_rollback_append_engine_snapshot();
    let engine_evaluation = evaluate_module_audit_rollback_append_engine_candidate(engine);
    module_audit_rollback_append_contract_snapshot_from_storage_and_engine(
        storage_evaluation,
        engine_evaluation,
    )
}

fn module_audit_rollback_append_contract_snapshot_from_storage_and_engine(
    storage: ModuleAuditRollbackStorageLayoutEvaluation,
    engine: ModuleAuditRollbackAppendEngineEvaluation,
) -> ModuleAuditRollbackAppendContractCandidate {
    ModuleAuditRollbackAppendContractCandidate {
        audit_append_envelope: module_audit_rollback_missing_append_contract_fact(storage, engine),
        rollback_transaction_envelope: module_audit_rollback_missing_append_contract_fact(
            storage, engine,
        ),
    }
}

fn module_audit_rollback_missing_append_contract_fact(
    storage: ModuleAuditRollbackStorageLayoutEvaluation,
    engine: ModuleAuditRollbackAppendEngineEvaluation,
) -> ModuleAuditRollbackAppendContractFact {
    ModuleAuditRollbackAppendContractFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_write_policy: false,
        binds_availability: false,
        binds_storage_layout_id: false,
        binds_append_engine_id: false,
        binds_write_policy_id: false,
        binds_availability_id: false,
        binds_envelope_provenance: false,
        storage_layout_available: storage.storage_layout_available,
        append_engine_available: engine.append_engine_available,
    }
}

fn module_audit_rollback_available_append_contract_fact() -> ModuleAuditRollbackAppendContractFact {
    ModuleAuditRollbackAppendContractFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_write_policy: true,
        binds_availability: true,
        binds_storage_layout_id: true,
        binds_append_engine_id: true,
        binds_write_policy_id: true,
        binds_availability_id: true,
        binds_envelope_provenance: true,
        storage_layout_available: true,
        append_engine_available: true,
    }
}

fn evaluate_module_audit_rollback_append_contract_candidate(
    candidate: ModuleAuditRollbackAppendContractCandidate,
) -> ModuleAuditRollbackAppendContractEvaluation {
    let (audit_append_status, audit_append_reason) = evaluate_module_append_contract_fact(
        candidate.audit_append_envelope,
        "audit_append_envelope_scope_must_be_current_boot",
        "audit_append_envelope_schema_mismatch",
        "audit_append_envelope_missing",
        "audit_append_envelope_provenance_missing",
        "audit_append_envelope_write_policy_binding_missing",
        "audit_append_envelope_availability_binding_missing",
        "audit_append_envelope_storage_layout_binding_missing",
        "audit_append_envelope_append_engine_binding_missing",
        "audit_append_envelope_provenance_binding_missing",
        "audit_ledger_storage_layout_missing",
        "audit_ledger_append_engine_missing",
        "audit_append_envelope_available",
    );
    let (rollback_transaction_status, rollback_transaction_reason) =
        evaluate_module_append_contract_fact(
            candidate.rollback_transaction_envelope,
            "rollback_transaction_envelope_scope_must_be_current_boot",
            "rollback_transaction_envelope_schema_mismatch",
            "rollback_transaction_envelope_missing",
            "rollback_transaction_envelope_provenance_missing",
            "rollback_transaction_envelope_write_policy_binding_missing",
            "rollback_transaction_envelope_availability_binding_missing",
            "rollback_transaction_envelope_storage_layout_binding_missing",
            "rollback_transaction_envelope_append_engine_binding_missing",
            "rollback_transaction_envelope_provenance_binding_missing",
            "rollback_store_storage_layout_missing",
            "rollback_store_append_engine_missing",
            "rollback_transaction_envelope_available",
        );

    let (status, reason) = if method_eq(audit_append_status, "rejected") {
        ("rejected", audit_append_reason)
    } else if method_eq(rollback_transaction_status, "rejected") {
        ("rejected", rollback_transaction_reason)
    } else if method_eq(audit_append_status, "missing")
        && method_eq(audit_append_reason, "audit_append_envelope_missing")
        && method_eq(rollback_transaction_status, "missing")
        && method_eq(
            rollback_transaction_reason,
            "rollback_transaction_envelope_missing",
        )
    {
        (
            "missing",
            "audit_append_envelope_missing_and_rollback_transaction_envelope_missing",
        )
    } else if method_eq(audit_append_status, "missing") {
        ("missing", audit_append_reason)
    } else if method_eq(rollback_transaction_status, "missing") {
        ("missing", rollback_transaction_reason)
    } else {
        (
            "denied_write_path_unimplemented",
            "durable_audit_rollback_writer_unimplemented",
        )
    };

    ModuleAuditRollbackAppendContractEvaluation {
        status,
        reason,
        audit_append_status,
        audit_append_reason,
        rollback_transaction_status,
        rollback_transaction_reason,
        storage_layout_available: candidate.audit_append_envelope.storage_layout_available
            && candidate
                .rollback_transaction_envelope
                .storage_layout_available,
        append_engine_available: candidate.audit_append_envelope.append_engine_available
            && candidate
                .rollback_transaction_envelope
                .append_engine_available,
        writes_enabled: false,
        installs_rollback_plan: false,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_append_contract_fact(
    fact: ModuleAuditRollbackAppendContractFact,
    scope_reason: &'static str,
    schema_reason: &'static str,
    missing_reason: &'static str,
    provenance_reason: &'static str,
    write_policy_reason: &'static str,
    availability_reason: &'static str,
    storage_layout_binding_reason: &'static str,
    append_engine_binding_reason: &'static str,
    provenance_binding_reason: &'static str,
    storage_layout_reason: &'static str,
    append_engine_reason: &'static str,
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
    if !fact.binds_envelope_provenance {
        return ("rejected", provenance_binding_reason);
    }
    if !fact.binds_write_policy {
        return ("rejected", write_policy_reason);
    }
    if !fact.binds_availability {
        return ("rejected", availability_reason);
    }
    if !fact.binds_write_policy_id {
        return ("rejected", write_policy_reason);
    }
    if !fact.binds_availability_id {
        return ("rejected", availability_reason);
    }
    if !fact.binds_storage_layout_id {
        return ("rejected", storage_layout_binding_reason);
    }
    if !fact.binds_append_engine_id {
        return ("rejected", append_engine_binding_reason);
    }
    if !fact.storage_layout_available {
        return ("missing", storage_layout_reason);
    }
    if !fact.append_engine_available {
        return ("missing", append_engine_reason);
    }
    ("available", available_reason)
}

fn emit_module_append_contract_facts(
    append: ModuleAuditRollbackAppendContractCandidate,
    evaluation: ModuleAuditRollbackAppendContractEvaluation,
) {
    raw_line("      \"append_contract_facts\": {");
    emit_module_append_contract_fact(
        "audit_append_envelope",
        "raios.audit_ledger_append_envelope.v0",
        "append.audit_ledger.current_boot",
        "raios.audit_record.v0",
        "storage.audit_rollback_layout.current_boot",
        "append_engine.audit_ledger.current_boot",
        "policy.durable_audit_write.current_boot",
        "availability.durable_audit_ledger.current_boot",
        append.audit_append_envelope,
        evaluation.audit_append_status,
        evaluation.audit_append_reason,
        true,
    );
    emit_module_append_contract_fact(
        "rollback_transaction_envelope",
        "raios.rollback_store_transaction_envelope.v0",
        "append.rollback_store.current_boot",
        "raios.rollback_plan.v0",
        "storage.audit_rollback_layout.current_boot",
        "append_engine.rollback_store.current_boot",
        "policy.rollback_install.current_boot",
        "availability.rollback_store.current_boot",
        append.rollback_transaction_envelope,
        evaluation.rollback_transaction_status,
        evaluation.rollback_transaction_reason,
        false,
    );
    raw_line("      }");
}

fn emit_module_append_contract_storage_layout_inputs(
    storage: ModuleAuditRollbackStorageLayoutCandidate,
    evaluation: ModuleAuditRollbackStorageLayoutEvaluation,
) {
    raw_line("      \"storage_layout_inputs\": {");
    raw_line("        \"persistence_device_inventory\": {");
    raw_line("          \"schema\": \"raios.persistence_device_inventory.v0\",");
    raw("          \"status\": ");
    json_str(evaluation.persistence_device_status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(evaluation.persistence_device_reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(storage.persistence_device_inventory.present);
    raw_line(",");
    raw("          \"stable_identity\": ");
    raw_bool(storage.persistence_device_inventory.stable_identity);
    raw_line(",");
    raw("          \"partition_inventory_available\": ");
    raw_bool(
        storage
            .persistence_device_inventory
            .partition_inventory_available,
    );
    raw_line(",");
    raw("          \"write_path_available\": ");
    raw_bool(storage.persistence_device_inventory.write_path_available);
    raw_line(",");
    raw_line("          \"authorizes_append\": false");
    raw_line("        },");
    raw_line("        \"audit_rollback_storage_layout\": {");
    raw_line("          \"schema\": \"raios.audit_rollback_storage_layout.v0\",");
    raw("          \"status\": ");
    json_str(evaluation.storage_layout_status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(evaluation.storage_layout_reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(storage.audit_rollback_storage_layout.present);
    raw_line(",");
    raw("          \"binds_persistence_device\": ");
    raw_bool(
        storage
            .audit_rollback_storage_layout
            .binds_persistence_device,
    );
    raw_line(",");
    raw("          \"has_audit_ledger_region\": ");
    raw_bool(
        storage
            .audit_rollback_storage_layout
            .has_audit_ledger_region,
    );
    raw_line(",");
    raw("          \"has_rollback_store_region\": ");
    raw_bool(
        storage
            .audit_rollback_storage_layout
            .has_rollback_store_region,
    );
    raw_line(",");
    raw("          \"append_slots_available\": ");
    raw_bool(storage.audit_rollback_storage_layout.append_slots_available);
    raw_line(",");
    raw("          \"authorizes_append\": false");
    crlf();
    raw_line("        },");
    raw("        \"storage_layout_available\": ");
    raw_bool(evaluation.storage_layout_available);
    raw_line(",");
    raw_line("        \"storage_layout_facts_are_append_authority\": false");
    raw_line("      }");
}

fn emit_module_append_contract_append_engine_inputs(
    engine: ModuleAuditRollbackAppendEngineCandidate,
    evaluation: ModuleAuditRollbackAppendEngineEvaluation,
) {
    raw_line("      \"append_engine_inputs\": {");
    raw_line("        \"audit_ledger_append_engine\": {");
    raw_line("          \"schema\": \"raios.audit_ledger_append_engine.v0\",");
    raw("          \"status\": ");
    json_str(evaluation.audit_engine_status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(evaluation.audit_engine_reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(engine.audit_ledger_append_engine.present);
    raw_line(",");
    raw("          \"binds_storage_layout\": ");
    raw_bool(engine.audit_ledger_append_engine.binds_storage_layout);
    raw_line(",");
    raw("          \"binds_write_policy\": ");
    raw_bool(engine.audit_ledger_append_engine.binds_write_policy);
    raw_line(",");
    raw("          \"supports_append_only\": ");
    raw_bool(engine.audit_ledger_append_engine.supports_append_only);
    raw_line(",");
    raw_line("          \"authorizes_append_contract\": false");
    raw_line("        },");
    raw_line("        \"rollback_store_transaction_engine\": {");
    raw_line("          \"schema\": \"raios.rollback_store_transaction_engine.v0\",");
    raw("          \"status\": ");
    json_str(evaluation.rollback_engine_status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(evaluation.rollback_engine_reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(engine.rollback_store_transaction_engine.present);
    raw_line(",");
    raw("          \"binds_storage_layout\": ");
    raw_bool(
        engine
            .rollback_store_transaction_engine
            .binds_storage_layout,
    );
    raw_line(",");
    raw("          \"binds_write_policy\": ");
    raw_bool(engine.rollback_store_transaction_engine.binds_write_policy);
    raw_line(",");
    raw("          \"supports_replay\": ");
    raw_bool(engine.rollback_store_transaction_engine.supports_replay);
    raw_line(",");
    raw_line("          \"authorizes_append_contract\": false");
    raw_line("        },");
    raw("        \"append_engine_available\": ");
    raw_bool(evaluation.append_engine_available);
    raw_line(",");
    raw_line("        \"append_engine_facts_are_append_authority\": false");
    raw_line("      }");
}

fn emit_module_append_contract_fact(
    name: &'static str,
    schema: &'static str,
    id: &'static str,
    target_schema: &'static str,
    storage_layout_id: &'static str,
    append_engine_id: &'static str,
    write_policy_id: &'static str,
    availability_id: &'static str,
    fact: ModuleAuditRollbackAppendContractFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw_line(": {");
    raw("          \"schema\": ");
    json_str(schema);
    raw_line(",");
    raw("          \"id\": ");
    json_str(id);
    raw_line(",");
    raw("          \"target_schema\": ");
    json_str(target_schema);
    raw_line(",");
    raw_line("          \"storage_layout_schema\": \"raios.audit_rollback_storage_layout.v0\",");
    raw("          \"storage_layout_id\": ");
    json_str(storage_layout_id);
    raw_line(",");
    raw("          \"append_engine_id\": ");
    json_str(append_engine_id);
    raw_line(",");
    raw("          \"write_policy_id\": ");
    json_str(write_policy_id);
    raw_line(",");
    raw("          \"availability_id\": ");
    json_str(availability_id);
    raw_line(",");
    raw_line("          \"append_contract_provenance_schema\": \"raios.append_contract_envelope_provenance.v0\",");
    raw("          \"scope\": ");
    json_str(fact.scope);
    raw_line(",");
    raw("          \"classification\": ");
    json_str(fact.classification);
    raw_line(",");
    raw("          \"status\": ");
    json_str(status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(fact.present);
    raw_line(",");
    raw("          \"schema_valid\": ");
    raw_bool(fact.schema_ok);
    raw_line(",");
    raw("          \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw_line(",");
    raw("          \"binds_write_policy\": ");
    raw_bool(fact.binds_write_policy);
    raw_line(",");
    raw("          \"binds_availability\": ");
    raw_bool(fact.binds_availability);
    raw_line(",");
    raw("          \"binds_storage_layout_id\": ");
    raw_bool(fact.binds_storage_layout_id);
    raw_line(",");
    raw("          \"binds_append_engine_id\": ");
    raw_bool(fact.binds_append_engine_id);
    raw_line(",");
    raw("          \"binds_write_policy_id\": ");
    raw_bool(fact.binds_write_policy_id);
    raw_line(",");
    raw("          \"binds_availability_id\": ");
    raw_bool(fact.binds_availability_id);
    raw_line(",");
    raw("          \"binds_envelope_provenance\": ");
    raw_bool(fact.binds_envelope_provenance);
    raw_line(",");
    raw("          \"storage_layout_available\": ");
    raw_bool(fact.storage_layout_available);
    raw_line(",");
    raw("          \"append_engine_available\": ");
    raw_bool(fact.append_engine_available);
    raw_line(",");
    raw_line("          \"required_bindings\": {");
    raw("            \"storage_layout_id\": ");
    json_str(storage_layout_id);
    raw_line(",");
    raw("            \"append_engine_id\": ");
    json_str(append_engine_id);
    raw_line(",");
    raw("            \"write_policy_id\": ");
    json_str(write_policy_id);
    raw_line(",");
    raw("            \"availability_id\": ");
    json_str(availability_id);
    raw_line(",");
    raw_line("            \"provenance_schema\": \"raios.append_contract_envelope_provenance.v0\"");
    raw_line("          },");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"write_attempted\": false,");
    raw_line("          \"provenance\": {");
    raw_line("            \"source_method\": \"module.audit_rollback_append_contract\",");
    raw_line("            \"source_transport\": \"serial-console\",");
    raw_line("            \"event_scope\": \"current_boot\",");
    raw_line("            \"record_id\": null");
    raw_line("          }");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

fn module_audit_rollback_append_contract_selftest_cases(
) -> [ModuleAuditRollbackAppendContractSelfTestCase;
       MODULE_AUDIT_ROLLBACK_APPEND_CONTRACT_SELFTEST_CASES] {
    let missing = module_audit_rollback_append_contract_snapshot();
    let available = module_audit_rollback_available_append_contract_fact();
    [
        module_audit_rollback_append_contract_selftest_case(
            "missing_append_envelope_pair_current_boot",
            "missing",
            "audit_append_envelope_missing_and_rollback_transaction_envelope_missing",
            missing,
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_previous_boot",
            "rejected",
            "audit_append_envelope_scope_must_be_current_boot",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    scope: "previous_boot",
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_wrong_schema",
            "rejected",
            "audit_append_envelope_schema_mismatch",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    schema_ok: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_provenance_missing",
            "rejected",
            "audit_append_envelope_provenance_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    provenance_ok: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_provenance_binding_missing",
            "rejected",
            "audit_append_envelope_provenance_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_envelope_provenance: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_policy_binding_missing",
            "rejected",
            "audit_append_envelope_write_policy_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_write_policy: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_write_policy_id_missing",
            "rejected",
            "audit_append_envelope_write_policy_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_write_policy_id: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_availability_binding_missing",
            "rejected",
            "audit_append_envelope_availability_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_availability: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_availability_id_missing",
            "rejected",
            "audit_append_envelope_availability_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_availability_id: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_storage_layout_id_missing",
            "rejected",
            "audit_append_envelope_storage_layout_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_storage_layout_id: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_append_envelope_append_engine_id_missing",
            "rejected",
            "audit_append_envelope_append_engine_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_append_engine_id: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "audit_ledger_storage_layout_missing",
            "missing",
            "audit_ledger_storage_layout_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    storage_layout_available: false,
                    ..available
                },
                rollback_transaction_envelope: available,
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_previous_boot",
            "rejected",
            "rollback_transaction_envelope_scope_must_be_current_boot",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    scope: "previous_boot",
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_wrong_schema",
            "rejected",
            "rollback_transaction_envelope_schema_mismatch",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    schema_ok: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_provenance_missing",
            "rejected",
            "rollback_transaction_envelope_provenance_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    provenance_ok: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_provenance_binding_missing",
            "rejected",
            "rollback_transaction_envelope_provenance_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_envelope_provenance: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_policy_binding_missing",
            "rejected",
            "rollback_transaction_envelope_write_policy_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_write_policy: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_write_policy_id_missing",
            "rejected",
            "rollback_transaction_envelope_write_policy_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_write_policy_id: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_availability_binding_missing",
            "rejected",
            "rollback_transaction_envelope_availability_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_availability: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_availability_id_missing",
            "rejected",
            "rollback_transaction_envelope_availability_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_availability_id: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_storage_layout_id_missing",
            "rejected",
            "rollback_transaction_envelope_storage_layout_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_storage_layout_id: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_transaction_envelope_append_engine_id_missing",
            "rejected",
            "rollback_transaction_envelope_append_engine_binding_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    binds_append_engine_id: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "rollback_store_storage_layout_missing",
            "missing",
            "rollback_store_storage_layout_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: available,
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    storage_layout_available: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_contract_selftest_case(
            "available_envelopes_append_engine_still_missing",
            "missing",
            "audit_ledger_append_engine_missing",
            ModuleAuditRollbackAppendContractCandidate {
                audit_append_envelope: ModuleAuditRollbackAppendContractFact {
                    append_engine_available: false,
                    ..available
                },
                rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact {
                    append_engine_available: false,
                    ..available
                },
            },
        ),
    ]
}

fn module_audit_rollback_append_contract_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleAuditRollbackAppendContractCandidate,
) -> ModuleAuditRollbackAppendContractSelfTestCase {
    let actual = evaluate_module_audit_rollback_append_contract_candidate(candidate);
    ModuleAuditRollbackAppendContractSelfTestCase {
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

pub(crate) fn emit_module_audit_rollback_append_intent() {
    let append_contract = module_audit_rollback_append_contract_snapshot();
    let append_evaluation =
        evaluate_module_audit_rollback_append_contract_candidate(append_contract);
    let intent =
        module_audit_rollback_append_intent_snapshot_from_append_contract(append_evaluation);
    let evaluation = evaluate_module_audit_rollback_append_intent_candidate(intent);

    begin_response("module.audit_rollback_append_intent");
    raw_line("      \"schema\": \"raios.module_audit_rollback_append_intent.v0\",");
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
    emit_module_append_intent_append_contract_inputs(append_contract, append_evaluation);
    raw_line(",");
    emit_module_append_intent_facts(intent, evaluation);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"append_intent_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"append_intent_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"audit_intent_missing\": ");
    raw_bool(!method_eq(evaluation.audit_intent_status, "available"));
    raw_line(",");
    raw("        \"rollback_intent_missing\": ");
    raw_bool(!method_eq(evaluation.rollback_intent_status, "available"));
    raw_line(",");
    raw("        \"append_contract_available\": ");
    raw_bool(evaluation.append_contract_available);
    raw_line(",");
    raw("        \"append_contract_missing\": ");
    raw_bool(!evaluation.append_contract_available);
    raw_line(",");
    raw("        \"append_intent_available\": ");
    raw_bool(evaluation.append_intent_available);
    raw_line(",");
    raw("        \"append_intent_missing\": ");
    raw_bool(!evaluation.append_intent_available);
    raw_line(",");
    raw_line("        \"append_intent_facts_are_writer_authority\": false,");
    raw_line("        \"append_contract_facts_are_append_intent_authority\": false,");
    raw_line("        \"retained_hash_refs_are_append_intent_authority\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    let append_contract_available = evaluation.append_contract_available;
    emit_export_gate(
        &mut wrote,
        "append_contract",
        if append_contract_available {
            "available"
        } else {
            append_evaluation.status
        },
        if append_contract_available {
            "append_contract_available"
        } else {
            append_evaluation.reason
        },
    );
    emit_export_gate(
        &mut wrote,
        "audit_record_append_intent",
        evaluation.audit_intent_status,
        evaluation.audit_intent_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_transaction_append_intent",
        evaluation.rollback_intent_status,
        evaluation.rollback_intent_reason,
    );
    crlf();
    raw_line("      ]");
    end_response("module.audit_rollback_append_intent");
}

pub(crate) fn emit_module_audit_rollback_append_intent_selftest() {
    let cases = module_audit_rollback_append_intent_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.audit_rollback_append_intent_selftest");
    raw_line("      \"schema\": \"raios.module_audit_rollback_append_intent_selftest.v0\",");
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
        emit_module_audit_rollback_append_intent_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.audit_rollback_append_intent_selftest");
}

fn emit_module_audit_rollback_append_intent_selftest_case(
    case: &ModuleAuditRollbackAppendIntentSelfTestCase,
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

fn module_audit_rollback_append_intent_snapshot() -> ModuleAuditRollbackAppendIntentCandidate {
    let append_contract = module_audit_rollback_append_contract_snapshot();
    let append_evaluation =
        evaluate_module_audit_rollback_append_contract_candidate(append_contract);
    module_audit_rollback_append_intent_snapshot_from_append_contract(append_evaluation)
}

fn module_audit_rollback_append_intent_snapshot_from_append_contract(
    append: ModuleAuditRollbackAppendContractEvaluation,
) -> ModuleAuditRollbackAppendIntentCandidate {
    ModuleAuditRollbackAppendIntentCandidate {
        audit_record_append_intent: module_audit_rollback_missing_append_intent_fact(method_eq(
            append.audit_append_status,
            "available",
        )),
        rollback_transaction_append_intent: module_audit_rollback_missing_append_intent_fact(
            method_eq(append.rollback_transaction_status, "available"),
        ),
    }
}

fn module_audit_rollback_missing_append_intent_fact(
    append_contract_available: bool,
) -> ModuleAuditRollbackAppendIntentFact {
    ModuleAuditRollbackAppendIntentFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_append_contract: false,
        binds_append_contract_id: false,
        binds_append_engine_id: false,
        binds_storage_layout_id: false,
        binds_write_policy_id: false,
        binds_availability_id: false,
        binds_payload_hash: false,
        binds_intent_provenance: false,
        append_contract_available,
    }
}

fn module_audit_rollback_available_append_intent_fact() -> ModuleAuditRollbackAppendIntentFact {
    ModuleAuditRollbackAppendIntentFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_append_contract: true,
        binds_append_contract_id: true,
        binds_append_engine_id: true,
        binds_storage_layout_id: true,
        binds_write_policy_id: true,
        binds_availability_id: true,
        binds_payload_hash: true,
        binds_intent_provenance: true,
        append_contract_available: true,
    }
}

fn evaluate_module_audit_rollback_append_intent_candidate(
    candidate: ModuleAuditRollbackAppendIntentCandidate,
) -> ModuleAuditRollbackAppendIntentEvaluation {
    let (audit_intent_status, audit_intent_reason) = evaluate_module_append_intent_fact(
        candidate.audit_record_append_intent,
        "audit_record_append_intent_scope_must_be_current_boot",
        "audit_record_append_intent_schema_mismatch",
        "audit_record_append_intent_missing",
        "audit_record_append_intent_provenance_missing",
        "audit_record_append_intent_provenance_binding_missing",
        "audit_record_append_intent_append_contract_binding_missing",
        "audit_record_append_intent_append_engine_binding_missing",
        "audit_record_append_intent_storage_layout_binding_missing",
        "audit_record_append_intent_write_policy_binding_missing",
        "audit_record_append_intent_availability_binding_missing",
        "audit_record_append_intent_payload_hash_missing",
        "audit_record_append_contract_missing",
        "audit_record_append_intent_available",
    );
    let (rollback_intent_status, rollback_intent_reason) = evaluate_module_append_intent_fact(
        candidate.rollback_transaction_append_intent,
        "rollback_transaction_append_intent_scope_must_be_current_boot",
        "rollback_transaction_append_intent_schema_mismatch",
        "rollback_transaction_append_intent_missing",
        "rollback_transaction_append_intent_provenance_missing",
        "rollback_transaction_append_intent_provenance_binding_missing",
        "rollback_transaction_append_intent_append_contract_binding_missing",
        "rollback_transaction_append_intent_append_engine_binding_missing",
        "rollback_transaction_append_intent_storage_layout_binding_missing",
        "rollback_transaction_append_intent_write_policy_binding_missing",
        "rollback_transaction_append_intent_availability_binding_missing",
        "rollback_transaction_append_intent_payload_hash_missing",
        "rollback_transaction_append_contract_missing",
        "rollback_transaction_append_intent_available",
    );

    let (status, reason) = if method_eq(audit_intent_status, "rejected") {
        ("rejected", audit_intent_reason)
    } else if method_eq(rollback_intent_status, "rejected") {
        ("rejected", rollback_intent_reason)
    } else if method_eq(audit_intent_status, "missing")
        && method_eq(audit_intent_reason, "audit_record_append_intent_missing")
        && method_eq(rollback_intent_status, "missing")
        && method_eq(
            rollback_intent_reason,
            "rollback_transaction_append_intent_missing",
        )
    {
        (
            "missing",
            "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing",
        )
    } else if method_eq(audit_intent_status, "missing") {
        ("missing", audit_intent_reason)
    } else if method_eq(rollback_intent_status, "missing") {
        ("missing", rollback_intent_reason)
    } else {
        ("available", "audit_rollback_append_intent_available")
    };

    let append_contract_available = candidate
        .audit_record_append_intent
        .append_contract_available
        && candidate
            .rollback_transaction_append_intent
            .append_contract_available;
    let append_intent_available = method_eq(audit_intent_status, "available")
        && method_eq(rollback_intent_status, "available");
    ModuleAuditRollbackAppendIntentEvaluation {
        status,
        reason,
        audit_intent_status,
        audit_intent_reason,
        rollback_intent_status,
        rollback_intent_reason,
        append_contract_available,
        append_intent_available,
        writes_enabled: false,
        installs_rollback_plan: false,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_append_intent_fact(
    fact: ModuleAuditRollbackAppendIntentFact,
    scope_reason: &'static str,
    schema_reason: &'static str,
    missing_reason: &'static str,
    provenance_reason: &'static str,
    provenance_binding_reason: &'static str,
    append_contract_binding_reason: &'static str,
    append_engine_binding_reason: &'static str,
    storage_layout_binding_reason: &'static str,
    write_policy_binding_reason: &'static str,
    availability_binding_reason: &'static str,
    payload_hash_reason: &'static str,
    append_contract_reason: &'static str,
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
    if !fact.binds_intent_provenance {
        return ("rejected", provenance_binding_reason);
    }
    if !fact.binds_append_contract || !fact.binds_append_contract_id {
        return ("rejected", append_contract_binding_reason);
    }
    if !fact.binds_append_engine_id {
        return ("rejected", append_engine_binding_reason);
    }
    if !fact.binds_storage_layout_id {
        return ("rejected", storage_layout_binding_reason);
    }
    if !fact.binds_write_policy_id {
        return ("rejected", write_policy_binding_reason);
    }
    if !fact.binds_availability_id {
        return ("rejected", availability_binding_reason);
    }
    if !fact.binds_payload_hash {
        return ("rejected", payload_hash_reason);
    }
    if !fact.append_contract_available {
        return ("missing", append_contract_reason);
    }
    ("available", available_reason)
}

fn emit_module_append_intent_append_contract_inputs(
    append: ModuleAuditRollbackAppendContractCandidate,
    evaluation: ModuleAuditRollbackAppendContractEvaluation,
) {
    raw_line("      \"append_contract_inputs\": {");
    emit_module_append_intent_append_contract_input(
        "audit_append_envelope",
        "raios.audit_ledger_append_envelope.v0",
        append.audit_append_envelope,
        evaluation.audit_append_status,
        evaluation.audit_append_reason,
        true,
    );
    emit_module_append_intent_append_contract_input(
        "rollback_transaction_envelope",
        "raios.rollback_store_transaction_envelope.v0",
        append.rollback_transaction_envelope,
        evaluation.rollback_transaction_status,
        evaluation.rollback_transaction_reason,
        true,
    );
    raw("        \"append_contract_available\": ");
    raw_bool(
        method_eq(evaluation.audit_append_status, "available")
            && method_eq(evaluation.rollback_transaction_status, "available"),
    );
    raw_line(",");
    raw_line("        \"append_contract_facts_are_append_intent_authority\": false");
    raw_line("      }");
}

fn emit_module_append_intent_append_contract_input(
    name: &'static str,
    schema: &'static str,
    fact: ModuleAuditRollbackAppendContractFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw(": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": ");
    json_str(status);
    raw(", \"reason\": ");
    json_str(reason);
    raw(", \"scope\": ");
    json_str(fact.scope);
    raw(", \"classification\": ");
    json_str(fact.classification);
    raw(", \"present\": ");
    raw_bool(fact.present);
    raw(", \"binds_storage_layout_id\": ");
    raw_bool(fact.binds_storage_layout_id);
    raw(", \"binds_append_engine_id\": ");
    raw_bool(fact.binds_append_engine_id);
    raw(", \"binds_write_policy_id\": ");
    raw_bool(fact.binds_write_policy_id);
    raw(", \"binds_availability_id\": ");
    raw_bool(fact.binds_availability_id);
    raw(", \"binds_envelope_provenance\": ");
    raw_bool(fact.binds_envelope_provenance);
    raw(", \"authorizes_append_intent\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_append_intent_facts(
    intent: ModuleAuditRollbackAppendIntentCandidate,
    evaluation: ModuleAuditRollbackAppendIntentEvaluation,
) {
    raw_line("      \"append_intent_facts\": {");
    emit_module_append_intent_fact(
        "audit_record_append_intent",
        "raios.audit_record_append_intent.v0",
        "append_intent.audit_record.current_boot",
        "raios.audit_record.v0",
        "append.audit_ledger.current_boot",
        "append_engine.audit_ledger.current_boot",
        "storage.audit_rollback_layout.current_boot",
        "policy.durable_audit_write.current_boot",
        "availability.durable_audit_ledger.current_boot",
        intent.audit_record_append_intent,
        evaluation.audit_intent_status,
        evaluation.audit_intent_reason,
        true,
    );
    emit_module_append_intent_fact(
        "rollback_transaction_append_intent",
        "raios.rollback_transaction_append_intent.v0",
        "append_intent.rollback_transaction.current_boot",
        "raios.rollback_plan.v0",
        "append.rollback_store.current_boot",
        "append_engine.rollback_store.current_boot",
        "storage.audit_rollback_layout.current_boot",
        "policy.rollback_install.current_boot",
        "availability.rollback_store.current_boot",
        intent.rollback_transaction_append_intent,
        evaluation.rollback_intent_status,
        evaluation.rollback_intent_reason,
        false,
    );
    raw_line("      }");
}

fn emit_module_append_intent_fact(
    name: &'static str,
    schema: &'static str,
    id: &'static str,
    target_schema: &'static str,
    append_contract_id: &'static str,
    append_engine_id: &'static str,
    storage_layout_id: &'static str,
    write_policy_id: &'static str,
    availability_id: &'static str,
    fact: ModuleAuditRollbackAppendIntentFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw_line(": {");
    raw("          \"schema\": ");
    json_str(schema);
    raw_line(",");
    raw("          \"id\": ");
    json_str(id);
    raw_line(",");
    raw("          \"target_schema\": ");
    json_str(target_schema);
    raw_line(",");
    raw("          \"append_contract_id\": ");
    json_str(append_contract_id);
    raw_line(",");
    raw("          \"append_engine_id\": ");
    json_str(append_engine_id);
    raw_line(",");
    raw("          \"storage_layout_id\": ");
    json_str(storage_layout_id);
    raw_line(",");
    raw("          \"write_policy_id\": ");
    json_str(write_policy_id);
    raw_line(",");
    raw("          \"availability_id\": ");
    json_str(availability_id);
    raw_line(",");
    raw_line("          \"intent_provenance_schema\": \"raios.append_intent_provenance.v0\",");
    raw_line("          \"payload_hash_schema\": \"raios.append_intent_payload_hash.v0\",");
    raw_line("          \"payload_hash\": null,");
    raw("          \"scope\": ");
    json_str(fact.scope);
    raw_line(",");
    raw("          \"classification\": ");
    json_str(fact.classification);
    raw_line(",");
    raw("          \"status\": ");
    json_str(status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(fact.present);
    raw_line(",");
    raw("          \"schema_valid\": ");
    raw_bool(fact.schema_ok);
    raw_line(",");
    raw("          \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw_line(",");
    raw("          \"binds_append_contract\": ");
    raw_bool(fact.binds_append_contract);
    raw_line(",");
    raw("          \"binds_append_contract_id\": ");
    raw_bool(fact.binds_append_contract_id);
    raw_line(",");
    raw("          \"binds_append_engine_id\": ");
    raw_bool(fact.binds_append_engine_id);
    raw_line(",");
    raw("          \"binds_storage_layout_id\": ");
    raw_bool(fact.binds_storage_layout_id);
    raw_line(",");
    raw("          \"binds_write_policy_id\": ");
    raw_bool(fact.binds_write_policy_id);
    raw_line(",");
    raw("          \"binds_availability_id\": ");
    raw_bool(fact.binds_availability_id);
    raw_line(",");
    raw("          \"binds_payload_hash\": ");
    raw_bool(fact.binds_payload_hash);
    raw_line(",");
    raw("          \"binds_intent_provenance\": ");
    raw_bool(fact.binds_intent_provenance);
    raw_line(",");
    raw("          \"append_contract_available\": ");
    raw_bool(fact.append_contract_available);
    raw_line(",");
    raw_line("          \"required_bindings\": {");
    raw("            \"append_contract_id\": ");
    json_str(append_contract_id);
    raw_line(",");
    raw("            \"append_engine_id\": ");
    json_str(append_engine_id);
    raw_line(",");
    raw("            \"storage_layout_id\": ");
    json_str(storage_layout_id);
    raw_line(",");
    raw("            \"write_policy_id\": ");
    json_str(write_policy_id);
    raw_line(",");
    raw("            \"availability_id\": ");
    json_str(availability_id);
    raw_line(",");
    raw_line("            \"payload_hash\": \"required\",");
    raw_line("            \"provenance_schema\": \"raios.append_intent_provenance.v0\"");
    raw_line("          },");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"write_attempted\": false,");
    raw_line("          \"install_attempted\": false,");
    raw_line("          \"provenance\": {");
    raw_line("            \"source_method\": \"module.audit_rollback_append_intent\",");
    raw_line("            \"source_transport\": \"serial-console\",");
    raw_line("            \"event_scope\": \"current_boot\",");
    raw_line("            \"record_id\": null");
    raw_line("          }");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

fn module_audit_rollback_append_intent_selftest_cases(
) -> [ModuleAuditRollbackAppendIntentSelfTestCase; MODULE_AUDIT_ROLLBACK_APPEND_INTENT_SELFTEST_CASES]
{
    let missing = module_audit_rollback_append_intent_snapshot();
    let available = module_audit_rollback_available_append_intent_fact();
    [
        module_audit_rollback_append_intent_selftest_case(
            "missing_append_intent_pair_current_boot",
            "missing",
            "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing",
            missing,
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_previous_boot",
            "rejected",
            "audit_record_append_intent_scope_must_be_current_boot",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    scope: "previous_boot",
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_wrong_schema",
            "rejected",
            "audit_record_append_intent_schema_mismatch",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    schema_ok: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_provenance_missing",
            "rejected",
            "audit_record_append_intent_provenance_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    provenance_ok: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_provenance_binding_missing",
            "rejected",
            "audit_record_append_intent_provenance_binding_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_intent_provenance: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_append_contract_binding_missing",
            "rejected",
            "audit_record_append_intent_append_contract_binding_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_append_contract: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_append_engine_binding_missing",
            "rejected",
            "audit_record_append_intent_append_engine_binding_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_append_engine_id: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_storage_layout_binding_missing",
            "rejected",
            "audit_record_append_intent_storage_layout_binding_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_storage_layout_id: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_write_policy_binding_missing",
            "rejected",
            "audit_record_append_intent_write_policy_binding_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_write_policy_id: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_availability_binding_missing",
            "rejected",
            "audit_record_append_intent_availability_binding_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_availability_id: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_payload_hash_missing",
            "rejected",
            "audit_record_append_intent_payload_hash_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_payload_hash: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "audit_record_append_intent_append_contract_missing",
            "missing",
            "audit_record_append_contract_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: ModuleAuditRollbackAppendIntentFact {
                    append_contract_available: false,
                    ..available
                },
                rollback_transaction_append_intent: available,
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "rollback_transaction_append_intent_previous_boot",
            "rejected",
            "rollback_transaction_append_intent_scope_must_be_current_boot",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    scope: "previous_boot",
                    ..available
                },
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "rollback_transaction_append_intent_wrong_schema",
            "rejected",
            "rollback_transaction_append_intent_schema_mismatch",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    schema_ok: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "rollback_transaction_append_intent_provenance_missing",
            "rejected",
            "rollback_transaction_append_intent_provenance_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    provenance_ok: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "rollback_transaction_append_intent_append_contract_binding_missing",
            "rejected",
            "rollback_transaction_append_intent_append_contract_binding_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_append_contract_id: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "rollback_transaction_append_intent_payload_hash_missing",
            "rejected",
            "rollback_transaction_append_intent_payload_hash_missing",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact {
                    binds_payload_hash: false,
                    ..available
                },
            },
        ),
        module_audit_rollback_append_intent_selftest_case(
            "available_append_intents_still_non_authorizing",
            "available",
            "audit_rollback_append_intent_available",
            ModuleAuditRollbackAppendIntentCandidate {
                audit_record_append_intent: available,
                rollback_transaction_append_intent: available,
            },
        ),
    ]
}

fn module_audit_rollback_append_intent_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleAuditRollbackAppendIntentCandidate,
) -> ModuleAuditRollbackAppendIntentSelfTestCase {
    let actual = evaluate_module_audit_rollback_append_intent_candidate(candidate);
    ModuleAuditRollbackAppendIntentSelfTestCase {
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

pub(crate) fn emit_module_audit_rollback_write_boundary() {
    let binding = event_log::module_load_gate_binding_snapshot();
    let availability = module_audit_rollback_availability_snapshot();
    let availability_evaluation =
        evaluate_module_audit_rollback_availability_candidate(availability);
    let policy = module_audit_rollback_write_policy_snapshot();
    let policy_evaluation = evaluate_module_audit_rollback_write_policy_candidate(policy);
    let append_contract = module_audit_rollback_append_contract_snapshot();
    let append_evaluation =
        evaluate_module_audit_rollback_append_contract_candidate(append_contract);
    let append_intent =
        module_audit_rollback_append_intent_snapshot_from_append_contract(append_evaluation);
    let append_intent_evaluation =
        evaluate_module_audit_rollback_append_intent_candidate(append_intent);
    let candidate = module_audit_rollback_write_boundary_candidate_from_binding(
        binding,
        availability_evaluation,
        policy_evaluation,
        append_evaluation,
        append_intent_evaluation,
    );
    let evaluation = evaluate_module_audit_rollback_write_boundary_candidate(candidate);

    begin_response("module.audit_rollback_write_boundary");
    raw_line("      \"schema\": \"raios.module_audit_rollback_write_boundary.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"global_event_log_mutation\": \"none\",");
    raw_line("      \"writes_enabled\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"pre_load_write_request\": {");
    raw_line("        \"schema\": \"raios.module_pre_load_audit_rollback_write_request.v0\",");
    raw_line(
        "        \"canonicalization\": \"raios.module_pre_load_audit_rollback_write_request.canonical.v0\",",
    );
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"live_service_graph\",");
    raw_line("        \"requested_writes\": [");
    raw_line(
        "          {\"target\": \"durable_audit_ledger\", \"schema\": \"raios.audit_record.v0\"},",
    );
    raw_line("          {\"target\": \"rollback_store\", \"schema\": \"raios.rollback_plan.v0\"}");
    raw_line("        ],");
    raw_line("        \"required_retained_references\": [");
    raw_line("          \"raios.module_manifest_reference.v0\",");
    raw_line("          \"raios.module_candidate_artifact_reference.v0\",");
    raw_line("          \"raios.module_vm_test_report_reference.v0\",");
    raw_line("          \"raios.computed_capability_grant.v0\",");
    raw_line("          \"raios.module_local_attestation_reference.v0\",");
    raw_line("          \"raios.module_local_approval_reference.v0\",");
    raw_line("          \"raios.module_audit_rollback_reference.v0\",");
    raw_line("          \"raios.module_service_slot_reservation.v0\"");
    raw_line("        ],");
    raw_line("        \"recovery_artifact_loading\": \"separate_capability\"");
    raw_line("      },");
    emit_module_write_boundary_inputs(binding);
    raw_line(",");
    emit_module_write_boundary_availability_inputs(availability, availability_evaluation);
    raw_line(",");
    emit_module_write_boundary_policy_inputs(policy, policy_evaluation);
    raw_line(",");
    emit_module_write_boundary_append_contract_inputs(append_contract, append_evaluation);
    raw_line(",");
    emit_module_write_boundary_append_intent_inputs(append_intent, append_intent_evaluation);
    raw_line(",");
    emit_module_write_boundary_denial_evidence(evaluation);
    raw_line(",");
    emit_module_write_boundary_policy_result(
        evaluation,
        policy_evaluation,
        append_evaluation,
        append_intent_evaluation,
    );
    raw_line(",");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    if !method_eq(evaluation.status, "denied_missing_durable_write_boundary") {
        emit_export_gate(
            &mut wrote,
            "write_boundary_preconditions",
            evaluation.status,
            evaluation.reason,
        );
    }
    emit_export_gate(
        &mut wrote,
        "durable_audit_write",
        evaluation.durable_audit_write_state,
        evaluation.durable_audit_write_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_install",
        evaluation.rollback_install_state,
        evaluation.rollback_install_reason,
    );
    emit_export_gate(
        &mut wrote,
        "audit_append_envelope",
        append_evaluation.audit_append_status,
        append_evaluation.audit_append_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_transaction_envelope",
        append_evaluation.rollback_transaction_status,
        append_evaluation.rollback_transaction_reason,
    );
    emit_export_gate(
        &mut wrote,
        "audit_record_append_intent",
        append_intent_evaluation.audit_intent_status,
        append_intent_evaluation.audit_intent_reason,
    );
    emit_export_gate(
        &mut wrote,
        "rollback_transaction_append_intent",
        append_intent_evaluation.rollback_intent_status,
        append_intent_evaluation.rollback_intent_reason,
    );
    emit_export_gate(
        &mut wrote,
        "module_loader",
        "unavailable",
        "module_loader_unimplemented",
    );
    crlf();
    raw_line("      ]");
    end_response("module.audit_rollback_write_boundary");
}

fn emit_module_write_boundary_inputs(binding: event_log::ModuleLoadGateBinding) {
    raw_line("      \"retained_reference_inputs\": {");
    emit_module_write_boundary_input_ref(
        "module_manifest",
        binding.manifest_reference_event_id,
        binding.manifest_reference_status,
        binding.manifest_reference_reason,
        "raios.module_manifest_reference.v0",
        true,
    );
    emit_module_write_boundary_input_ref(
        "candidate_artifact",
        binding.artifact_reference_event_id,
        binding.artifact_reference_status,
        binding.artifact_reference_reason,
        "raios.module_candidate_artifact_reference.v0",
        true,
    );
    emit_module_write_boundary_input_ref(
        "vm_test_report",
        binding.vm_report_reference_event_id,
        binding.vm_report_reference_status,
        binding.vm_report_reference_reason,
        "raios.module_vm_test_report_reference.v0",
        true,
    );
    emit_module_write_boundary_input_ref(
        "computed_capability_grant",
        binding.retained_reference_event_id,
        computed_grant_status(binding),
        computed_grant_reason(binding),
        "raios.computed_capability_grant.v0",
        true,
    );
    emit_module_write_boundary_input_ref(
        "local_attestation",
        binding.attestation_reference_event_id,
        binding.attestation_reference_status,
        binding.attestation_reference_reason,
        "raios.module_local_attestation_reference.v0",
        true,
    );
    emit_module_write_boundary_input_ref(
        "local_approval",
        binding.approval_reference_event_id,
        binding.approval_reference_status,
        binding.approval_reference_reason,
        "raios.module_local_approval_reference.v0",
        true,
    );
    emit_module_write_boundary_input_ref(
        "audit_rollback",
        binding.audit_rollback_reference_event_id,
        binding.audit_rollback_reference_status,
        binding.audit_rollback_reference_reason,
        "raios.module_audit_rollback_reference.v0",
        true,
    );
    emit_module_write_boundary_input_ref(
        "service_slot_reservation",
        binding.service_slot_reservation_event_id,
        binding.service_slot_reservation_status,
        binding.service_slot_reservation_reason,
        "raios.module_service_slot_reservation.v0",
        false,
    );
    raw_line("      },");
    raw_line("      \"hash_inputs\": {");
    let retained = binding.retained_reference;
    let approval = binding.approval_reference;
    let audit = binding.audit_rollback_reference;
    let service_slot = binding.service_slot_reservation;
    raw("        \"manifest_hash\": ");
    json_sha256_option(retained.map(|reference| reference.manifest_hash));
    raw_line(",");
    raw("        \"candidate_artifact_hash\": ");
    json_sha256_option(retained.map(|reference| reference.artifact_hash));
    raw_line(",");
    raw("        \"vm_test_report_hash\": ");
    json_sha256_option(retained.map(|reference| reference.vm_report_hash));
    raw_line(",");
    raw("        \"local_attestation_hash\": ");
    json_sha256_option(retained.map(|reference| reference.local_attestation_hash));
    raw_line(",");
    raw("        \"local_approval_hash\": ");
    json_sha256_option(approval.map(|reference| reference.local_approval_hash));
    raw_line(",");
    raw("        \"computed_capability_grant_hash\": ");
    json_sha256_option(retained.map(|reference| reference.computed_grant_hash));
    raw_line(",");
    raw("        \"audit_record_hash\": ");
    json_sha256_option(audit.map(|reference| reference.audit_record_hash));
    raw_line(",");
    raw("        \"rollback_plan_hash\": ");
    json_sha256_option(audit.map(|reference| reference.rollback_plan_hash));
    raw_line(",");
    raw("        \"pre_load_service_inventory_hash\": ");
    json_sha256_option(audit.map(|reference| reference.pre_load_service_inventory_hash));
    raw_line(",");
    raw("        \"cleanup_actions_hash\": ");
    json_sha256_option(audit.map(|reference| reference.cleanup_actions_hash));
    raw_line(",");
    raw("        \"service_slot_reservation_hash\": ");
    json_sha256_option(service_slot.map(|reservation| reservation.reservation_hash));
    raw_line(",");
    raw("        \"ram_only_service_slot_id\": ");
    if let Some(reference) = audit {
        json_str(reference.ram_only_service_slot_id.as_str());
    } else {
        raw("null");
    }
    crlf();
    raw_line("      }");
}

fn emit_module_write_boundary_input_ref(
    name: &'static str,
    event_id: Option<event_log::EventId>,
    status: &'static str,
    reason: &'static str,
    schema: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw(": {\"event_id\": ");
    json_event_id_option(event_id);
    raw(", \"schema\": ");
    json_str(schema);
    raw(", \"status\": ");
    json_str(status);
    raw(", \"reason\": ");
    json_str(reason);
    raw(", \"classification\": \"local_only\", \"authorizes_guest_load\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_availability_facts(
    availability: ModuleAuditRollbackAvailabilityCandidate,
    evaluation: ModuleAuditRollbackAvailabilityEvaluation,
) {
    raw_line("      \"availability_facts\": {");
    emit_module_availability_fact(
        "durable_audit_ledger",
        "raios.durable_audit_ledger.v0",
        "availability.durable_audit_ledger.current_boot",
        availability.durable_audit_ledger,
        evaluation.durable_audit_ledger_status,
        evaluation.durable_audit_ledger_reason,
        true,
    );
    emit_module_availability_fact(
        "rollback_store",
        "raios.rollback_store.v0",
        "availability.rollback_store.current_boot",
        availability.rollback_store,
        evaluation.rollback_store_status,
        evaluation.rollback_store_reason,
        false,
    );
    raw_line("      }");
}

fn emit_module_availability_fact(
    name: &'static str,
    schema: &'static str,
    id: &'static str,
    fact: ModuleAuditRollbackAvailabilityFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw_line(": {");
    raw("          \"schema\": ");
    json_str(schema);
    raw_line(",");
    raw("          \"id\": ");
    json_str(id);
    raw_line(",");
    raw("          \"scope\": ");
    json_str(fact.scope);
    raw_line(",");
    raw("          \"classification\": ");
    json_str(fact.classification);
    raw_line(",");
    raw("          \"status\": ");
    json_str(status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(fact.present);
    raw_line(",");
    raw("          \"schema_valid\": ");
    raw_bool(fact.schema_ok);
    raw_line(",");
    raw("          \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw_line(",");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"write_attempted\": false,");
    raw_line("          \"install_attempted\": false,");
    raw_line("          \"provenance\": {");
    raw_line("            \"source_method\": \"module.audit_rollback_availability\",");
    raw_line("            \"source_transport\": \"serial-console\",");
    raw_line("            \"event_scope\": \"current_boot\",");
    raw_line("            \"record_id\": null");
    raw_line("          }");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_write_boundary_availability_inputs(
    availability: ModuleAuditRollbackAvailabilityCandidate,
    evaluation: ModuleAuditRollbackAvailabilityEvaluation,
) {
    raw_line("      \"availability_inputs\": {");
    emit_module_write_boundary_availability_input(
        "durable_audit_ledger",
        "raios.durable_audit_ledger.v0",
        availability.durable_audit_ledger,
        evaluation.durable_audit_ledger_status,
        evaluation.durable_audit_ledger_reason,
        true,
    );
    emit_module_write_boundary_availability_input(
        "rollback_store",
        "raios.rollback_store.v0",
        availability.rollback_store,
        evaluation.rollback_store_status,
        evaluation.rollback_store_reason,
        false,
    );
    raw_line("      }");
}

fn emit_module_write_boundary_availability_input(
    name: &'static str,
    schema: &'static str,
    fact: ModuleAuditRollbackAvailabilityFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw(": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": ");
    json_str(status);
    raw(", \"reason\": ");
    json_str(reason);
    raw(", \"scope\": ");
    json_str(fact.scope);
    raw(", \"classification\": ");
    json_str(fact.classification);
    raw(", \"present\": ");
    raw_bool(fact.present);
    raw(", \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw(", \"authorizes_write\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_write_boundary_policy_inputs(
    policy: ModuleAuditRollbackWritePolicyCandidate,
    evaluation: ModuleAuditRollbackWritePolicyEvaluation,
) {
    raw_line("      \"policy_inputs\": {");
    emit_module_write_boundary_policy_input(
        "durable_write_policy",
        "raios.durable_audit_write_policy.v0",
        policy.durable_write_policy,
        evaluation.durable_write_policy_status,
        evaluation.durable_write_policy_reason,
        true,
    );
    emit_module_write_boundary_policy_input(
        "rollback_install_policy",
        "raios.rollback_install_policy.v0",
        policy.rollback_install_policy,
        evaluation.rollback_install_policy_status,
        evaluation.rollback_install_policy_reason,
        false,
    );
    raw_line("      }");
}

fn emit_module_write_boundary_policy_input(
    name: &'static str,
    schema: &'static str,
    fact: ModuleAuditRollbackWritePolicyFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw(": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": ");
    json_str(status);
    raw(", \"reason\": ");
    json_str(reason);
    raw(", \"scope\": ");
    json_str(fact.scope);
    raw(", \"classification\": ");
    json_str(fact.classification);
    raw(", \"present\": ");
    raw_bool(fact.present);
    raw(", \"binds_retained_evidence\": ");
    raw_bool(fact.binds_retained_evidence);
    raw(", \"binds_availability\": ");
    raw_bool(fact.binds_availability);
    raw(", \"authorizes_write\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_write_boundary_append_contract_inputs(
    append: ModuleAuditRollbackAppendContractCandidate,
    evaluation: ModuleAuditRollbackAppendContractEvaluation,
) {
    raw_line("      \"append_contract_inputs\": {");
    emit_module_write_boundary_append_contract_input(
        "audit_append_envelope",
        "raios.audit_ledger_append_envelope.v0",
        append.audit_append_envelope,
        evaluation.audit_append_status,
        evaluation.audit_append_reason,
        true,
    );
    emit_module_write_boundary_append_contract_input(
        "rollback_transaction_envelope",
        "raios.rollback_store_transaction_envelope.v0",
        append.rollback_transaction_envelope,
        evaluation.rollback_transaction_status,
        evaluation.rollback_transaction_reason,
        false,
    );
    raw_line("      }");
}

fn emit_module_write_boundary_append_contract_input(
    name: &'static str,
    schema: &'static str,
    fact: ModuleAuditRollbackAppendContractFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw(": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": ");
    json_str(status);
    raw(", \"reason\": ");
    json_str(reason);
    raw(", \"scope\": ");
    json_str(fact.scope);
    raw(", \"classification\": ");
    json_str(fact.classification);
    raw(", \"present\": ");
    raw_bool(fact.present);
    raw(", \"binds_write_policy\": ");
    raw_bool(fact.binds_write_policy);
    raw(", \"binds_availability\": ");
    raw_bool(fact.binds_availability);
    raw(", \"binds_storage_layout_id\": ");
    raw_bool(fact.binds_storage_layout_id);
    raw(", \"binds_append_engine_id\": ");
    raw_bool(fact.binds_append_engine_id);
    raw(", \"binds_write_policy_id\": ");
    raw_bool(fact.binds_write_policy_id);
    raw(", \"binds_availability_id\": ");
    raw_bool(fact.binds_availability_id);
    raw(", \"binds_envelope_provenance\": ");
    raw_bool(fact.binds_envelope_provenance);
    raw(", \"storage_layout_available\": ");
    raw_bool(fact.storage_layout_available);
    raw(", \"append_engine_available\": ");
    raw_bool(fact.append_engine_available);
    raw(", \"authorizes_write\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_write_boundary_append_intent_inputs(
    intent: ModuleAuditRollbackAppendIntentCandidate,
    evaluation: ModuleAuditRollbackAppendIntentEvaluation,
) {
    raw_line("      \"append_intent_inputs\": {");
    emit_module_write_boundary_append_intent_input(
        "audit_record_append_intent",
        "raios.audit_record_append_intent.v0",
        intent.audit_record_append_intent,
        evaluation.audit_intent_status,
        evaluation.audit_intent_reason,
        true,
    );
    emit_module_write_boundary_append_intent_input(
        "rollback_transaction_append_intent",
        "raios.rollback_transaction_append_intent.v0",
        intent.rollback_transaction_append_intent,
        evaluation.rollback_intent_status,
        evaluation.rollback_intent_reason,
        true,
    );
    raw("        \"append_contract_available\": ");
    raw_bool(evaluation.append_contract_available);
    raw_line(",");
    raw("        \"append_intent_available\": ");
    raw_bool(evaluation.append_intent_available);
    raw_line(",");
    raw_line("        \"append_intent_facts_are_writer_authority\": false");
    raw_line("      }");
}

fn emit_module_write_boundary_append_intent_input(
    name: &'static str,
    schema: &'static str,
    fact: ModuleAuditRollbackAppendIntentFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw(": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": ");
    json_str(status);
    raw(", \"reason\": ");
    json_str(reason);
    raw(", \"scope\": ");
    json_str(fact.scope);
    raw(", \"classification\": ");
    json_str(fact.classification);
    raw(", \"present\": ");
    raw_bool(fact.present);
    raw(", \"binds_append_contract_id\": ");
    raw_bool(fact.binds_append_contract_id);
    raw(", \"binds_append_engine_id\": ");
    raw_bool(fact.binds_append_engine_id);
    raw(", \"binds_storage_layout_id\": ");
    raw_bool(fact.binds_storage_layout_id);
    raw(", \"binds_write_policy_id\": ");
    raw_bool(fact.binds_write_policy_id);
    raw(", \"binds_availability_id\": ");
    raw_bool(fact.binds_availability_id);
    raw(", \"binds_payload_hash\": ");
    raw_bool(fact.binds_payload_hash);
    raw(", \"binds_intent_provenance\": ");
    raw_bool(fact.binds_intent_provenance);
    raw(", \"append_contract_available\": ");
    raw_bool(fact.append_contract_available);
    raw(", \"authorizes_write\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_write_boundary_denial_evidence(
    evaluation: ModuleAuditRollbackWriteBoundaryEvaluation,
) {
    raw_line("      \"denial_evidence\": {");
    raw_line("        \"schema\": \"raios.module_audit_rollback_write_denial_evidence.v0\",");
    raw("        \"validation_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"validation_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw_line("        \"durable_audit_write\": {");
    raw_line("          \"schema\": \"raios.audit_record.v0\",");
    raw("          \"state\": ");
    json_str(evaluation.durable_audit_write_state);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(evaluation.durable_audit_write_reason);
    raw_line(",");
    raw("          \"ledger\": ");
    json_str(evaluation.durable_audit_write_state);
    raw_line(",");
    raw_line("          \"write_attempted\": false");
    raw_line("        },");
    raw_line("        \"rollback_install\": {");
    raw_line("          \"schema\": \"raios.rollback_plan.v0\",");
    raw("          \"state\": ");
    json_str(evaluation.rollback_install_state);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(evaluation.rollback_install_reason);
    raw_line(",");
    raw("          \"store\": ");
    json_str(evaluation.rollback_install_state);
    raw_line(",");
    raw_line("          \"install_attempted\": false");
    raw_line("        },");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"recovery_artifact_loading\": \"separate_capability\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
}

fn emit_module_write_boundary_policy_result(
    evaluation: ModuleAuditRollbackWriteBoundaryEvaluation,
    policy: ModuleAuditRollbackWritePolicyEvaluation,
    append: ModuleAuditRollbackAppendContractEvaluation,
    append_intent: ModuleAuditRollbackAppendIntentEvaluation,
) {
    raw_line("      \"policy_result\": {");
    raw("        \"preconditions_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"preconditions_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"durable_write_policy_status\": ");
    json_str(policy.durable_write_policy_status);
    raw_line(",");
    raw("        \"durable_write_policy_reason\": ");
    json_str(policy.durable_write_policy_reason);
    raw_line(",");
    raw("        \"rollback_install_policy_status\": ");
    json_str(policy.rollback_install_policy_status);
    raw_line(",");
    raw("        \"rollback_install_policy_reason\": ");
    json_str(policy.rollback_install_policy_reason);
    raw_line(",");
    raw("        \"audit_append_status\": ");
    json_str(append.audit_append_status);
    raw_line(",");
    raw("        \"audit_append_reason\": ");
    json_str(append.audit_append_reason);
    raw_line(",");
    raw("        \"rollback_transaction_status\": ");
    json_str(append.rollback_transaction_status);
    raw_line(",");
    raw("        \"rollback_transaction_reason\": ");
    json_str(append.rollback_transaction_reason);
    raw_line(",");
    raw("        \"audit_append_intent_status\": ");
    json_str(append_intent.audit_intent_status);
    raw_line(",");
    raw("        \"audit_append_intent_reason\": ");
    json_str(append_intent.audit_intent_reason);
    raw_line(",");
    raw("        \"rollback_transaction_append_intent_status\": ");
    json_str(append_intent.rollback_intent_status);
    raw_line(",");
    raw("        \"rollback_transaction_append_intent_reason\": ");
    json_str(append_intent.rollback_intent_reason);
    raw_line(",");
    raw_line("        \"durable_audit_written\": false,");
    raw_line("        \"rollback_plan_installed\": false,");
    raw("        \"durable_audit_write_missing\": ");
    raw_bool(method_eq(evaluation.durable_audit_write_state, "missing"));
    raw_line(",");
    raw("        \"rollback_install_missing\": ");
    raw_bool(method_eq(evaluation.rollback_install_state, "missing"));
    raw_line(",");
    raw("        \"durable_write_policy_missing\": ");
    raw_bool(!method_eq(policy.durable_write_policy_status, "available"));
    raw_line(",");
    raw("        \"rollback_install_policy_missing\": ");
    raw_bool(!method_eq(
        policy.rollback_install_policy_status,
        "available",
    ));
    raw_line(",");
    raw("        \"storage_layout_missing\": ");
    raw_bool(!append.storage_layout_available);
    raw_line(",");
    raw("        \"append_engine_missing\": ");
    raw_bool(!append.append_engine_available);
    raw_line(",");
    raw("        \"append_contract_available\": ");
    raw_bool(append_intent.append_contract_available);
    raw_line(",");
    raw("        \"append_intent_available\": ");
    raw_bool(append_intent.append_intent_available);
    raw_line(",");
    raw("        \"append_intent_missing\": ");
    raw_bool(!append_intent.append_intent_available);
    raw_line(",");
    raw_line("        \"retained_hash_refs_are_durable_authority\": false,");
    raw_line("        \"retained_hash_refs_are_append_authority\": false,");
    raw_line("        \"retained_hash_refs_are_append_intent_authority\": false,");
    raw_line("        \"policy_facts_are_append_authority\": false,");
    raw_line("        \"append_contract_facts_are_append_intent_authority\": false,");
    raw_line("        \"append_intent_facts_are_writer_authority\": false,");
    raw_line("        \"recovery_artifact_loading_separate\": true,");
    raw_line("        \"grants_capability\": false,");
    raw_line("        \"grants_load_now\": false,");
    raw_line("        \"authorizes_guest_load\": false,");
    raw("        \"can_load_now\": ");
    raw_bool(evaluation.can_load);
    raw_line(",");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw("        \"load_attempted\": ");
    raw_bool(evaluation.load_attempted);
    crlf();
    raw_line("      }");
}

pub(crate) fn emit_module_audit_rollback_write_boundary_selftest() {
    let cases = module_audit_rollback_write_boundary_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.audit_rollback_write_boundary_selftest");
    raw_line("      \"schema\": \"raios.module_audit_rollback_write_boundary_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_durable_audit_records\": false,");
    raw_line("      \"creates_rollback_plans\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
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
        emit_module_audit_rollback_write_boundary_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.audit_rollback_write_boundary_selftest");
}

fn emit_module_audit_rollback_write_boundary_selftest_case(
    case: &ModuleAuditRollbackWriteBoundarySelfTestCase,
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
    raw(", \"creates_durable_audit_records\": false, \"installs_rollback_plan\": false, \"loads_artifact\": false, \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn module_audit_rollback_write_boundary_candidate_from_binding(
    binding: event_log::ModuleLoadGateBinding,
    availability: ModuleAuditRollbackAvailabilityEvaluation,
    policy: ModuleAuditRollbackWritePolicyEvaluation,
    append: ModuleAuditRollbackAppendContractEvaluation,
    append_intent: ModuleAuditRollbackAppendIntentEvaluation,
) -> ModuleAuditRollbackWriteBoundaryCandidate {
    ModuleAuditRollbackWriteBoundaryCandidate {
        scope: "current_boot",
        request_schema_ok: true,
        manifest_status: binding.manifest_reference_status,
        manifest_reason: binding.manifest_reference_reason,
        artifact_status: binding.artifact_reference_status,
        artifact_reason: binding.artifact_reference_reason,
        vm_report_status: binding.vm_report_reference_status,
        vm_report_reason: binding.vm_report_reference_reason,
        computed_grant_status: computed_grant_status(binding),
        computed_grant_reason: computed_grant_reason(binding),
        local_attestation_status: binding.attestation_reference_status,
        local_attestation_reason: binding.attestation_reference_reason,
        local_approval_status: binding.approval_reference_status,
        local_approval_reason: binding.approval_reference_reason,
        audit_rollback_status: binding.audit_rollback_reference_status,
        audit_rollback_reason: binding.audit_rollback_reference_reason,
        service_slot_status: binding.service_slot_reservation_status,
        service_slot_reason: binding.service_slot_reservation_reason,
        manifest_hash_matches_grant: manifest_hash_matches_grant(binding),
        artifact_hash_matches_grant: artifact_hash_matches_grant(binding),
        vm_report_hash_matches_grant: vm_report_hash_matches_grant(binding),
        local_attestation_hash_matches_grant: local_attestation_hash_matches_grant(binding),
        local_approval_hash_matches_audit: local_approval_hash_matches_audit(binding),
        audit_record_hash_matches_service_slot: audit_record_hash_matches_service_slot(binding),
        rollback_plan_hash_matches_service_slot: rollback_plan_hash_matches_service_slot(binding),
        service_slot_binds_audit_rollback: service_slot_binds_audit_rollback(binding),
        durable_audit_ledger_status: availability.durable_audit_ledger_status,
        durable_audit_ledger_reason: availability.durable_audit_ledger_reason,
        rollback_store_status: availability.rollback_store_status,
        rollback_store_reason: availability.rollback_store_reason,
        durable_write_policy_status: policy.durable_write_policy_status,
        durable_write_policy_reason: policy.durable_write_policy_reason,
        rollback_install_policy_status: policy.rollback_install_policy_status,
        rollback_install_policy_reason: policy.rollback_install_policy_reason,
        audit_append_status: append.audit_append_status,
        audit_append_reason: append.audit_append_reason,
        rollback_transaction_status: append.rollback_transaction_status,
        rollback_transaction_reason: append.rollback_transaction_reason,
        audit_append_intent_status: append_intent.audit_intent_status,
        audit_append_intent_reason: append_intent.audit_intent_reason,
        rollback_transaction_append_intent_status: append_intent.rollback_intent_status,
        rollback_transaction_append_intent_reason: append_intent.rollback_intent_reason,
        recovery_artifact_loader_requested: false,
    }
}

fn evaluate_module_audit_rollback_write_boundary_candidate(
    candidate: ModuleAuditRollbackWriteBoundaryCandidate,
) -> ModuleAuditRollbackWriteBoundaryEvaluation {
    if !method_eq(candidate.scope, "current_boot") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_scope_must_be_current_boot",
            candidate,
        );
    }
    if !candidate.request_schema_ok {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "pre_load_audit_rollback_write_request_schema_mismatch",
            candidate,
        );
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.manifest_status,
        candidate.manifest_reason,
        "retained_hash_reference_only",
        candidate,
    ) {
        return evaluation;
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.artifact_status,
        candidate.artifact_reason,
        "retained_hash_reference_only",
        candidate,
    ) {
        return evaluation;
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.vm_report_status,
        candidate.vm_report_reason,
        "retained_hash_reference_only",
        candidate,
    ) {
        return evaluation;
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.computed_grant_status,
        candidate.computed_grant_reason,
        "retained_hash_reference_only",
        candidate,
    ) {
        return evaluation;
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.local_attestation_status,
        candidate.local_attestation_reason,
        "retained_hash_reference_only",
        candidate,
    ) {
        return evaluation;
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.local_approval_status,
        candidate.local_approval_reason,
        "retained_hash_reference_only",
        candidate,
    ) {
        return evaluation;
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.audit_rollback_status,
        candidate.audit_rollback_reason,
        "retained_hash_reference_only",
        candidate,
    ) {
        return evaluation;
    }
    if let Some(evaluation) = write_boundary_require_status(
        candidate.service_slot_status,
        candidate.service_slot_reason,
        "retained_hash_reference_only_not_allocated",
        candidate,
    ) {
        return evaluation;
    }
    if !candidate.manifest_hash_matches_grant {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_manifest_hash_mismatch",
            candidate,
        );
    }
    if !candidate.artifact_hash_matches_grant {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_artifact_hash_mismatch",
            candidate,
        );
    }
    if !candidate.vm_report_hash_matches_grant {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_vm_test_report_hash_mismatch",
            candidate,
        );
    }
    if !candidate.local_attestation_hash_matches_grant {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_local_attestation_hash_mismatch",
            candidate,
        );
    }
    if !candidate.local_approval_hash_matches_audit {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_local_approval_hash_mismatch",
            candidate,
        );
    }
    if !candidate.audit_record_hash_matches_service_slot {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_audit_record_hash_mismatch",
            candidate,
        );
    }
    if !candidate.rollback_plan_hash_matches_service_slot {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_rollback_plan_hash_mismatch",
            candidate,
        );
    }
    if !candidate.service_slot_binds_audit_rollback {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "write_boundary_service_slot_reference_mismatch",
            candidate,
        );
    }
    if candidate.recovery_artifact_loader_requested {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            "recovery_artifact_loading_is_separate",
            candidate,
        );
    }
    if method_eq(candidate.durable_audit_ledger_status, "rejected") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.durable_audit_ledger_reason,
            candidate,
        );
    }
    if method_eq(candidate.rollback_store_status, "rejected") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.rollback_store_reason,
            candidate,
        );
    }

    let durable_audit_ledger_available =
        method_eq(candidate.durable_audit_ledger_status, "available");
    let rollback_store_available = method_eq(candidate.rollback_store_status, "available");
    if !durable_audit_ledger_available && !rollback_store_available {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_durable_write_boundary",
            "durable_audit_write_missing_and_rollback_install_missing",
            candidate,
        );
    }
    if !durable_audit_ledger_available {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_durable_write_boundary",
            "durable_audit_write_missing",
            candidate,
        );
    }
    if !rollback_store_available {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_durable_write_boundary",
            "rollback_install_missing",
            candidate,
        );
    }
    if method_eq(candidate.durable_write_policy_status, "rejected") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.durable_write_policy_reason,
            candidate,
        );
    }
    if method_eq(candidate.rollback_install_policy_status, "rejected") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.rollback_install_policy_reason,
            candidate,
        );
    }
    if !method_eq(candidate.durable_write_policy_status, "available") {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_durable_write_policy",
            candidate.durable_write_policy_reason,
            candidate,
        );
    }
    if !method_eq(candidate.rollback_install_policy_status, "available") {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_rollback_install_policy",
            candidate.rollback_install_policy_reason,
            candidate,
        );
    }
    if method_eq(candidate.audit_append_status, "rejected") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.audit_append_reason,
            candidate,
        );
    }
    if method_eq(candidate.rollback_transaction_status, "rejected") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.rollback_transaction_reason,
            candidate,
        );
    }

    let audit_append_available = method_eq(candidate.audit_append_status, "available");
    let rollback_transaction_available =
        method_eq(candidate.rollback_transaction_status, "available");
    if !audit_append_available
        && method_eq(
            candidate.audit_append_reason,
            "audit_append_envelope_missing",
        )
        && !rollback_transaction_available
        && method_eq(
            candidate.rollback_transaction_reason,
            "rollback_transaction_envelope_missing",
        )
    {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_append_contract",
            "audit_append_envelope_missing_and_rollback_transaction_envelope_missing",
            candidate,
        );
    }
    if !audit_append_available {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_append_contract",
            candidate.audit_append_reason,
            candidate,
        );
    }
    if !rollback_transaction_available {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_append_contract",
            candidate.rollback_transaction_reason,
            candidate,
        );
    }
    if method_eq(candidate.audit_append_intent_status, "rejected") {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.audit_append_intent_reason,
            candidate,
        );
    }
    if method_eq(
        candidate.rollback_transaction_append_intent_status,
        "rejected",
    ) {
        return module_audit_rollback_write_boundary_evaluation(
            "rejected",
            candidate.rollback_transaction_append_intent_reason,
            candidate,
        );
    }

    let audit_append_intent_available =
        method_eq(candidate.audit_append_intent_status, "available");
    let rollback_transaction_append_intent_available = method_eq(
        candidate.rollback_transaction_append_intent_status,
        "available",
    );
    if !audit_append_intent_available
        && method_eq(
            candidate.audit_append_intent_reason,
            "audit_record_append_intent_missing",
        )
        && !rollback_transaction_append_intent_available
        && method_eq(
            candidate.rollback_transaction_append_intent_reason,
            "rollback_transaction_append_intent_missing",
        )
    {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_append_intent",
            "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing",
            candidate,
        );
    }
    if !audit_append_intent_available {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_append_intent",
            candidate.audit_append_intent_reason,
            candidate,
        );
    }
    if !rollback_transaction_append_intent_available {
        return module_audit_rollback_write_boundary_evaluation(
            "denied_missing_append_intent",
            candidate.rollback_transaction_append_intent_reason,
            candidate,
        );
    }
    module_audit_rollback_write_boundary_evaluation(
        "denied_write_path_unimplemented",
        "durable_audit_rollback_writer_unimplemented",
        candidate,
    )
}

fn write_boundary_require_status(
    status: &'static str,
    reason: &'static str,
    expected: &'static str,
    candidate: ModuleAuditRollbackWriteBoundaryCandidate,
) -> Option<ModuleAuditRollbackWriteBoundaryEvaluation> {
    if method_eq(status, expected) {
        None
    } else {
        Some(module_audit_rollback_write_boundary_evaluation(
            if method_eq(status, "missing") {
                "missing"
            } else {
                "rejected"
            },
            reason,
            candidate,
        ))
    }
}

fn module_audit_rollback_write_boundary_evaluation(
    status: &'static str,
    reason: &'static str,
    candidate: ModuleAuditRollbackWriteBoundaryCandidate,
) -> ModuleAuditRollbackWriteBoundaryEvaluation {
    let durable_audit_ledger_available =
        method_eq(candidate.durable_audit_ledger_status, "available");
    let rollback_store_available = method_eq(candidate.rollback_store_status, "available");
    let durable_write_policy_available =
        method_eq(candidate.durable_write_policy_status, "available");
    let rollback_install_policy_available =
        method_eq(candidate.rollback_install_policy_status, "available");
    let audit_append_available = method_eq(candidate.audit_append_status, "available");
    let rollback_transaction_available =
        method_eq(candidate.rollback_transaction_status, "available");
    let audit_append_intent_available =
        method_eq(candidate.audit_append_intent_status, "available");
    let rollback_transaction_append_intent_available = method_eq(
        candidate.rollback_transaction_append_intent_status,
        "available",
    );
    ModuleAuditRollbackWriteBoundaryEvaluation {
        status,
        reason,
        durable_audit_write_state: if durable_audit_ledger_available {
            "available_but_write_disabled"
        } else {
            "missing"
        },
        durable_audit_write_reason: if durable_audit_ledger_available
            && !durable_write_policy_available
        {
            candidate.durable_write_policy_reason
        } else if durable_audit_ledger_available && !audit_append_available {
            candidate.audit_append_reason
        } else if durable_audit_ledger_available && !audit_append_intent_available {
            candidate.audit_append_intent_reason
        } else if durable_audit_ledger_available {
            "durable_audit_write_disabled_until_writer_exists"
        } else {
            "durable_audit_write_missing"
        },
        rollback_install_state: if rollback_store_available {
            "available_but_install_disabled"
        } else {
            "missing"
        },
        rollback_install_reason: if rollback_store_available && !rollback_install_policy_available {
            candidate.rollback_install_policy_reason
        } else if rollback_store_available && !rollback_transaction_available {
            candidate.rollback_transaction_reason
        } else if rollback_store_available && !rollback_transaction_append_intent_available {
            candidate.rollback_transaction_append_intent_reason
        } else if rollback_store_available {
            "rollback_install_disabled_until_installer_exists"
        } else {
            "rollback_install_missing"
        },
        can_load: false,
        load_attempted: false,
    }
}

fn module_audit_rollback_write_boundary_selftest_cases(
) -> [ModuleAuditRollbackWriteBoundarySelfTestCase;
       MODULE_AUDIT_ROLLBACK_WRITE_BOUNDARY_SELFTEST_CASES] {
    let valid = module_audit_rollback_write_boundary_valid_candidate();
    [
        module_audit_rollback_write_boundary_selftest_case(
            "missing_manifest_reference",
            "missing",
            "retained_module_manifest_reference_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                manifest_status: "missing",
                manifest_reason: "retained_module_manifest_reference_missing",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "stale_artifact_reference",
            "rejected",
            "retained_candidate_artifact_reference_stale_or_dropped_event_id",
            ModuleAuditRollbackWriteBoundaryCandidate {
                artifact_status: "rejected",
                artifact_reason: "retained_candidate_artifact_reference_stale_or_dropped_event_id",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "substituted_vm_report_reference",
            "rejected",
            "retained_vm_test_report_reference_substituted_record",
            ModuleAuditRollbackWriteBoundaryCandidate {
                vm_report_status: "rejected",
                vm_report_reason: "retained_vm_test_report_reference_substituted_record",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "previous_boot_write_request",
            "rejected",
            "write_boundary_scope_must_be_current_boot",
            ModuleAuditRollbackWriteBoundaryCandidate {
                scope: "previous_boot",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "write_request_schema_mismatch",
            "rejected",
            "pre_load_audit_rollback_write_request_schema_mismatch",
            ModuleAuditRollbackWriteBoundaryCandidate {
                request_schema_ok: false,
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "missing_computed_grant_reference",
            "missing",
            "retained_computed_grant_reference_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                computed_grant_status: "missing",
                computed_grant_reason: "retained_computed_grant_reference_missing",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "local_attestation_hash_mismatch",
            "rejected",
            "write_boundary_local_attestation_hash_mismatch",
            ModuleAuditRollbackWriteBoundaryCandidate {
                local_attestation_hash_matches_grant: false,
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "local_approval_hash_mismatch",
            "rejected",
            "write_boundary_local_approval_hash_mismatch",
            ModuleAuditRollbackWriteBoundaryCandidate {
                local_approval_hash_matches_audit: false,
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "audit_record_service_slot_hash_mismatch",
            "rejected",
            "write_boundary_audit_record_hash_mismatch",
            ModuleAuditRollbackWriteBoundaryCandidate {
                audit_record_hash_matches_service_slot: false,
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "rollback_plan_service_slot_hash_mismatch",
            "rejected",
            "write_boundary_rollback_plan_hash_mismatch",
            ModuleAuditRollbackWriteBoundaryCandidate {
                rollback_plan_hash_matches_service_slot: false,
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "substituted_service_slot_reference",
            "rejected",
            "write_boundary_service_slot_reference_mismatch",
            ModuleAuditRollbackWriteBoundaryCandidate {
                service_slot_binds_audit_rollback: false,
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "recovery_artifact_loader_requested",
            "rejected",
            "recovery_artifact_loading_is_separate",
            ModuleAuditRollbackWriteBoundaryCandidate {
                recovery_artifact_loader_requested: true,
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "durable_audit_ledger_available_rollback_store_missing",
            "denied_missing_durable_write_boundary",
            "rollback_install_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "rollback_store_available_durable_audit_ledger_missing",
            "denied_missing_durable_write_boundary",
            "durable_audit_write_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "availability_facts_present_policy_still_denied",
            "denied_missing_durable_write_policy",
            "durable_write_policy_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "durable_write_policy_available_rollback_policy_missing",
            "denied_missing_rollback_install_policy",
            "rollback_install_policy_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                durable_write_policy_status: "available",
                durable_write_policy_reason: "durable_write_policy_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "policy_facts_available_append_contract_missing",
            "denied_missing_append_contract",
            "audit_append_envelope_missing_and_rollback_transaction_envelope_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                durable_write_policy_status: "available",
                durable_write_policy_reason: "durable_write_policy_available",
                rollback_install_policy_status: "available",
                rollback_install_policy_reason: "rollback_install_policy_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "audit_append_available_rollback_transaction_missing",
            "denied_missing_append_contract",
            "rollback_transaction_envelope_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                durable_write_policy_status: "available",
                durable_write_policy_reason: "durable_write_policy_available",
                rollback_install_policy_status: "available",
                rollback_install_policy_reason: "rollback_install_policy_available",
                audit_append_status: "available",
                audit_append_reason: "audit_append_envelope_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "append_contract_available_append_intent_missing",
            "denied_missing_append_intent",
            "audit_record_append_intent_missing_and_rollback_transaction_append_intent_missing",
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                durable_write_policy_status: "available",
                durable_write_policy_reason: "durable_write_policy_available",
                rollback_install_policy_status: "available",
                rollback_install_policy_reason: "rollback_install_policy_available",
                audit_append_status: "available",
                audit_append_reason: "audit_append_envelope_available",
                rollback_transaction_status: "available",
                rollback_transaction_reason: "rollback_transaction_envelope_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "append_intents_available_writer_still_denied",
            "denied_write_path_unimplemented",
            "durable_audit_rollback_writer_unimplemented",
            ModuleAuditRollbackWriteBoundaryCandidate {
                durable_audit_ledger_status: "available",
                durable_audit_ledger_reason: "durable_audit_ledger_available",
                rollback_store_status: "available",
                rollback_store_reason: "rollback_store_available",
                durable_write_policy_status: "available",
                durable_write_policy_reason: "durable_write_policy_available",
                rollback_install_policy_status: "available",
                rollback_install_policy_reason: "rollback_install_policy_available",
                audit_append_status: "available",
                audit_append_reason: "audit_append_envelope_available",
                rollback_transaction_status: "available",
                rollback_transaction_reason: "rollback_transaction_envelope_available",
                audit_append_intent_status: "available",
                audit_append_intent_reason: "audit_record_append_intent_available",
                rollback_transaction_append_intent_status: "available",
                rollback_transaction_append_intent_reason:
                    "rollback_transaction_append_intent_available",
                ..valid
            },
        ),
        module_audit_rollback_write_boundary_selftest_case(
            "accepted_current_boot_preconditions_write_still_denied",
            "denied_missing_durable_write_boundary",
            "durable_audit_write_missing_and_rollback_install_missing",
            valid,
        ),
    ]
}

fn module_audit_rollback_write_boundary_valid_candidate(
) -> ModuleAuditRollbackWriteBoundaryCandidate {
    ModuleAuditRollbackWriteBoundaryCandidate {
        scope: "current_boot",
        request_schema_ok: true,
        manifest_status: "retained_hash_reference_only",
        manifest_reason: "retained_module_manifest_reference_not_authorizing",
        artifact_status: "retained_hash_reference_only",
        artifact_reason: "retained_candidate_artifact_reference_not_authorizing",
        vm_report_status: "retained_hash_reference_only",
        vm_report_reason: "retained_vm_test_report_reference_not_authorizing",
        computed_grant_status: "retained_hash_reference_only",
        computed_grant_reason: "retained_computed_grant_reference_not_authorizing",
        local_attestation_status: "retained_hash_reference_only",
        local_attestation_reason: "retained_local_attestation_reference_not_authorizing",
        local_approval_status: "retained_hash_reference_only",
        local_approval_reason: "retained_local_approval_reference_not_authorizing",
        audit_rollback_status: "retained_hash_reference_only",
        audit_rollback_reason: "retained_audit_rollback_reference_not_authorizing",
        service_slot_status: "retained_hash_reference_only_not_allocated",
        service_slot_reason: "retained_service_slot_reservation_not_allocated",
        manifest_hash_matches_grant: true,
        artifact_hash_matches_grant: true,
        vm_report_hash_matches_grant: true,
        local_attestation_hash_matches_grant: true,
        local_approval_hash_matches_audit: true,
        audit_record_hash_matches_service_slot: true,
        rollback_plan_hash_matches_service_slot: true,
        service_slot_binds_audit_rollback: true,
        durable_audit_ledger_status: "missing",
        durable_audit_ledger_reason: "durable_audit_ledger_missing",
        rollback_store_status: "missing",
        rollback_store_reason: "rollback_store_missing",
        durable_write_policy_status: "missing",
        durable_write_policy_reason: "durable_write_policy_missing",
        rollback_install_policy_status: "missing",
        rollback_install_policy_reason: "rollback_install_policy_missing",
        audit_append_status: "missing",
        audit_append_reason: "audit_append_envelope_missing",
        rollback_transaction_status: "missing",
        rollback_transaction_reason: "rollback_transaction_envelope_missing",
        audit_append_intent_status: "missing",
        audit_append_intent_reason: "audit_record_append_intent_missing",
        rollback_transaction_append_intent_status: "missing",
        rollback_transaction_append_intent_reason: "rollback_transaction_append_intent_missing",
        recovery_artifact_loader_requested: false,
    }
}

fn module_audit_rollback_write_boundary_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleAuditRollbackWriteBoundaryCandidate,
) -> ModuleAuditRollbackWriteBoundarySelfTestCase {
    let actual = evaluate_module_audit_rollback_write_boundary_candidate(candidate);
    ModuleAuditRollbackWriteBoundarySelfTestCase {
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

fn computed_grant_status(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if binding.retained_reference.is_some() {
        "retained_hash_reference_only"
    } else {
        "missing"
    }
}

fn computed_grant_reason(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if binding.retained_reference.is_some() {
        "retained_computed_grant_reference_not_authorizing"
    } else {
        "retained_computed_grant_reference_missing"
    }
}

fn manifest_hash_matches_grant(binding: event_log::ModuleLoadGateBinding) -> bool {
    match (binding.manifest_reference, binding.retained_reference) {
        (Some(manifest), Some(grant)) => manifest.manifest_hash == grant.manifest_hash,
        _ => false,
    }
}

fn artifact_hash_matches_grant(binding: event_log::ModuleLoadGateBinding) -> bool {
    match (binding.artifact_reference, binding.retained_reference) {
        (Some(artifact), Some(grant)) => artifact.artifact_hash == grant.artifact_hash,
        _ => false,
    }
}

fn vm_report_hash_matches_grant(binding: event_log::ModuleLoadGateBinding) -> bool {
    match (binding.vm_report_reference, binding.retained_reference) {
        (Some(report), Some(grant)) => report.vm_report_hash == grant.vm_report_hash,
        _ => false,
    }
}

fn local_attestation_hash_matches_grant(binding: event_log::ModuleLoadGateBinding) -> bool {
    match (binding.attestation_reference, binding.retained_reference) {
        (Some(attestation), Some(grant)) => {
            attestation.local_attestation_hash == grant.local_attestation_hash
        }
        _ => false,
    }
}

fn local_approval_hash_matches_audit(binding: event_log::ModuleLoadGateBinding) -> bool {
    match (binding.approval_reference, binding.audit_rollback_reference) {
        (Some(approval), Some(audit)) => approval.local_approval_hash == audit.local_approval_hash,
        _ => false,
    }
}

fn audit_record_hash_matches_service_slot(binding: event_log::ModuleLoadGateBinding) -> bool {
    match (
        binding.audit_rollback_reference,
        binding.service_slot_reservation,
    ) {
        (Some(audit), Some(reservation)) => {
            audit.audit_record_hash == reservation.audit_record_hash
        }
        _ => false,
    }
}

fn rollback_plan_hash_matches_service_slot(binding: event_log::ModuleLoadGateBinding) -> bool {
    match (
        binding.audit_rollback_reference,
        binding.service_slot_reservation,
    ) {
        (Some(audit), Some(reservation)) => {
            audit.rollback_plan_hash == reservation.rollback_plan_hash
        }
        _ => false,
    }
}

fn service_slot_binds_audit_rollback(binding: event_log::ModuleLoadGateBinding) -> bool {
    match (
        binding.audit_rollback_reference_event_id,
        binding.audit_rollback_reference,
        binding.service_slot_reservation,
    ) {
        (Some(audit_event_id), Some(audit), Some(reservation)) => {
            reservation.retained_audit_rollback_reference_event_id == audit_event_id
                && reservation.computed_grant_hash == audit.computed_grant_hash
                && reservation.audit_record_hash == audit.audit_record_hash
                && reservation.rollback_plan_hash == audit.rollback_plan_hash
                && reservation.pre_load_service_inventory_hash
                    == audit.pre_load_service_inventory_hash
                && reservation.ram_only_service_slot_id.as_str()
                    == audit.ram_only_service_slot_id.as_str()
        }
        _ => false,
    }
}
