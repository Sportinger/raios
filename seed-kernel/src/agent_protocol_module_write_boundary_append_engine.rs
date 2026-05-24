use crate::{
    agent_protocol_module_types::*, agent_protocol_module_write_boundary_storage_layout::*,
    agent_protocol_support::*,
};

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

pub(crate) fn emit_module_audit_rollback_append_engine_selftest_case(
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

pub(crate) fn module_audit_rollback_append_engine_snapshot(
) -> ModuleAuditRollbackAppendEngineCandidate {
    ModuleAuditRollbackAppendEngineCandidate {
        audit_ledger_append_engine: module_audit_rollback_missing_append_engine_fact(),
        rollback_store_transaction_engine: module_audit_rollback_missing_append_engine_fact(),
    }
}

pub(crate) fn module_audit_rollback_missing_append_engine_fact(
) -> ModuleAuditRollbackAppendEngineFact {
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

pub(crate) fn module_audit_rollback_available_append_engine_fact(
) -> ModuleAuditRollbackAppendEngineFact {
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

pub(crate) fn evaluate_module_audit_rollback_append_engine_candidate(
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

pub(crate) fn evaluate_module_append_engine_fact(
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

pub(crate) fn emit_module_append_engine_storage_layout_inputs(
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

pub(crate) fn emit_module_append_engine_facts(
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

pub(crate) fn emit_module_append_engine_fact(
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

pub(crate) fn module_audit_rollback_append_engine_selftest_cases(
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

pub(crate) fn module_audit_rollback_append_engine_selftest_case(
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
