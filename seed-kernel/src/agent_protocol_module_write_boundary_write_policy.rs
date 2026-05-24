use crate::{agent_protocol_module_types::*, agent_protocol_support::*};

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

pub(crate) fn emit_module_audit_rollback_write_policy_selftest_case(
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

pub(crate) fn emit_module_write_policy_fact(
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
