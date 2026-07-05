use alloc::vec;

use crate::{
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, crlf, emit_export_gate, emit_inline_record_object,
        emit_record_fields_trailing_comma, emit_record_property_line, end_response, method_eq,
        raw_line, record_bool as b, record_false as no, record_field as f, record_object as object,
        record_str as s,
    },
};
use raios_core::record::Value as V;

pub(crate) fn emit_module_audit_rollback_write_policy() {
    let policy = module_audit_rollback_write_policy_snapshot();
    let evaluation = evaluate_module_audit_rollback_write_policy_candidate(policy);

    begin_response("module.audit_rollback_write_policy");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.module_audit_rollback_write_policy.v0")),
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
    emit_module_write_policy_facts(policy, evaluation);
    raw_line(",");
    emit_record_property_line(
        "policy_result",
        vec![
            f("policy_status", s(evaluation.status)),
            f("policy_reason", s(evaluation.reason)),
            f(
                "durable_write_policy_missing",
                b(!method_eq(
                    evaluation.durable_write_policy_status,
                    "available",
                )),
            ),
            f(
                "rollback_install_policy_missing",
                b(!method_eq(
                    evaluation.rollback_install_policy_status,
                    "available",
                )),
            ),
            f("retained_hash_refs_are_policy_authority", no()),
            f("availability_facts_are_policy_authority", no()),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        true,
    );
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

#[rustfmt::skip]
pub(crate) fn emit_module_audit_rollback_write_policy_selftest() {
    let cases = module_audit_rollback_write_policy_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.audit_rollback_write_policy_selftest");
    emit_record_fields_trailing_comma(vec![f("schema", s("raios.module_audit_rollback_write_policy_selftest.v0")), f("scope", s("current_boot")), f("classification", s("local_only")), f("test_infrastructure", b(true)), f("mutates_global_event_log", no()), f("creates_durable_audit_records", no()), f("creates_rollback_plans", no()), f("installs_rollback_plan", no()), f("service_inventory_change", s("none")), f("load_attempted", no()), f("case_count", V::U64(cases.len() as u64)), f("passed", b(passed))], 6);
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

#[rustfmt::skip]
pub(crate) fn emit_module_audit_rollback_write_policy_selftest_case(
    case: &ModuleAuditRollbackWritePolicySelfTestCase,
    comma: bool,
) {
    emit_inline_record_object(vec![f("case", s(case.name)), f("expected_status", s(case.expected_status)), f("expected_reason", s(case.expected_reason)), f("actual_status", s(case.actual_status)), f("actual_reason", s(case.actual_reason)), f("passed", b(case.passed)), f("writes_enabled", no()), f("installs_rollback_plan", no()), f("can_load", no()), f("load_attempted", no())], comma);
}

pub(crate) fn module_audit_rollback_write_policy_snapshot(
) -> ModuleAuditRollbackWritePolicyCandidate {
    ModuleAuditRollbackWritePolicyCandidate {
        durable_write_policy: module_audit_rollback_missing_write_policy_fact(),
        rollback_install_policy: module_audit_rollback_missing_write_policy_fact(),
    }
}

pub(crate) fn module_audit_rollback_missing_write_policy_fact() -> ModuleAuditRollbackWritePolicyFact
{
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

pub(crate) fn module_audit_rollback_available_write_policy_fact(
) -> ModuleAuditRollbackWritePolicyFact {
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

pub(crate) fn evaluate_module_audit_rollback_write_policy_candidate(
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

pub(crate) fn evaluate_module_write_policy_fact(
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

pub(crate) fn emit_module_write_policy_facts(
    policy: ModuleAuditRollbackWritePolicyCandidate,
    evaluation: ModuleAuditRollbackWritePolicyEvaluation,
) {
    emit_record_property_line(
        "write_policy_facts",
        vec![
            f(
                "durable_write_policy",
                module_write_policy_fact_record(
                    "raios.durable_audit_write_policy.v0",
                    "policy.durable_audit_write.current_boot",
                    policy.durable_write_policy,
                    evaluation.durable_write_policy_status,
                    evaluation.durable_write_policy_reason,
                ),
            ),
            f(
                "rollback_install_policy",
                module_write_policy_fact_record(
                    "raios.rollback_install_policy.v0",
                    "policy.rollback_install.current_boot",
                    policy.rollback_install_policy,
                    evaluation.rollback_install_policy_status,
                    evaluation.rollback_install_policy_reason,
                ),
            ),
        ],
        false,
    );
}

fn module_write_policy_fact_record(
    schema: &'static str,
    id: &'static str,
    fact: ModuleAuditRollbackWritePolicyFact,
    status: &'static str,
    reason: &'static str,
) -> V<'static> {
    object(vec![
        f("schema", s(schema)),
        f("id", s(id)),
        f("scope", s(fact.scope)),
        f("classification", s(fact.classification)),
        f("status", s(status)),
        f("reason", s(reason)),
        f("present", b(fact.present)),
        f("schema_valid", b(fact.schema_ok)),
        f("provenance_valid", b(fact.provenance_ok)),
        f("binds_retained_evidence", b(fact.binds_retained_evidence)),
        f("binds_availability", b(fact.binds_availability)),
        f("authority", s("current_snapshot")),
        f("persistence", s("none")),
        f("durable", no()),
        f("authorizes_write", no()),
        f(
            "required_bindings",
            object(vec![
                f("module_manifest", s("raios.module_manifest_reference.v0")),
                f(
                    "candidate_artifact",
                    s("raios.module_candidate_artifact_reference.v0"),
                ),
                f(
                    "vm_test_report",
                    s("raios.module_vm_test_report_reference.v0"),
                ),
                f(
                    "computed_capability_grant",
                    s("raios.computed_capability_grant.v0"),
                ),
                f(
                    "local_attestation",
                    s("raios.module_local_attestation_reference.v0"),
                ),
                f(
                    "local_approval",
                    s("raios.module_local_approval_reference.v0"),
                ),
                f(
                    "audit_rollback",
                    s("raios.module_audit_rollback_reference.v0"),
                ),
                f(
                    "service_slot_reservation",
                    s("raios.module_service_slot_reservation.v0"),
                ),
                f("durable_audit_ledger", s("raios.durable_audit_ledger.v0")),
                f("rollback_store", s("raios.rollback_store.v0")),
            ]),
        ),
        f(
            "provenance",
            object(vec![
                f("source_method", s("module.audit_rollback_write_policy")),
                f("source_transport", s("serial-console")),
                f("event_scope", s("current_boot")),
                f("record_id", V::Null),
            ]),
        ),
    ])
}

pub(crate) fn module_audit_rollback_write_policy_selftest_cases(
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

pub(crate) fn module_audit_rollback_write_policy_selftest_case(
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
