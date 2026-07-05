use alloc::vec;

use crate::{
    agent_protocol_module_types::*,
    agent_protocol_module_write_boundary_storage_layout::*,
    agent_protocol_support::{
        begin_response, crlf, emit_export_gate, emit_inline_record_object,
        emit_record_fields_trailing_comma, emit_record_property_line, end_response, method_eq,
        raw_line, record_bool as b, record_false as no, record_field as f, record_object as object,
        record_str as s,
    },
};
use raios_core::record::Value as V;

pub(crate) fn emit_module_audit_rollback_append_engine() {
    let storage = module_audit_rollback_storage_layout_snapshot();
    let storage_evaluation = evaluate_module_audit_rollback_storage_layout_candidate(storage);
    let engine = module_audit_rollback_append_engine_snapshot();
    let evaluation = evaluate_module_audit_rollback_append_engine_candidate(engine);

    begin_response("module.audit_rollback_append_engine");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.module_audit_rollback_append_engine.v0")),
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
    emit_module_append_engine_storage_layout_inputs(storage, storage_evaluation);
    raw_line(",");
    emit_module_append_engine_facts(engine, evaluation);
    raw_line(",");
    emit_record_property_line(
        "policy_result",
        vec![
            f("append_engine_status", s(evaluation.status)),
            f("append_engine_reason", s(evaluation.reason)),
            f(
                "audit_engine_available",
                b(evaluation.audit_engine_available),
            ),
            f(
                "rollback_engine_available",
                b(evaluation.rollback_engine_available),
            ),
            f(
                "append_engine_available",
                b(evaluation.append_engine_available),
            ),
            f(
                "append_engine_missing",
                b(!evaluation.append_engine_available),
            ),
            f(
                "storage_layout_available",
                b(storage_evaluation.storage_layout_available),
            ),
            f(
                "storage_layout_missing",
                b(!storage_evaluation.storage_layout_available),
            ),
            f("retained_hash_refs_are_append_engine_authority", no()),
            f("availability_facts_are_append_engine_authority", no()),
            f("write_policy_facts_are_append_engine_authority", no()),
            f("storage_layout_facts_are_append_engine_authority", no()),
            f("append_engine_facts_are_append_authority", no()),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        true,
    );
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
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_audit_rollback_append_engine_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_durable_audit_records", no()),
            f("creates_rollback_plans", no()),
            f("installs_rollback_plan", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
        ],
        6,
    );
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
    emit_inline_record_object(
        vec![
            f("case", s(case.name)),
            f("expected_status", s(case.expected_status)),
            f("expected_reason", s(case.expected_reason)),
            f("actual_status", s(case.actual_status)),
            f("actual_reason", s(case.actual_reason)),
            f("passed", b(case.passed)),
            f("writes_enabled", no()),
            f("installs_rollback_plan", no()),
            f("can_load", no()),
            f("load_attempted", no()),
        ],
        comma,
    );
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
    emit_record_property_line(
        "storage_layout_inputs",
        vec![
            f(
                "persistence_device_inventory",
                object(vec![
                    f("schema", s("raios.persistence_device_inventory.v0")),
                    f("status", s(evaluation.persistence_device_status)),
                    f("reason", s(evaluation.persistence_device_reason)),
                    f("present", b(storage.persistence_device_inventory.present)),
                    f(
                        "stable_identity",
                        b(storage.persistence_device_inventory.stable_identity),
                    ),
                    f(
                        "partition_inventory_available",
                        b(storage
                            .persistence_device_inventory
                            .partition_inventory_available),
                    ),
                    f(
                        "write_path_available",
                        b(storage.persistence_device_inventory.write_path_available),
                    ),
                    f("authorizes_append_engine", no()),
                ]),
            ),
            f(
                "audit_rollback_storage_layout",
                object(vec![
                    f("schema", s("raios.audit_rollback_storage_layout.v0")),
                    f("status", s(evaluation.storage_layout_status)),
                    f("reason", s(evaluation.storage_layout_reason)),
                    f("present", b(storage.audit_rollback_storage_layout.present)),
                    f(
                        "binds_persistence_device",
                        b(storage
                            .audit_rollback_storage_layout
                            .binds_persistence_device),
                    ),
                    f(
                        "append_slots_available",
                        b(storage.audit_rollback_storage_layout.append_slots_available),
                    ),
                    f(
                        "recovery_region_separated",
                        b(storage
                            .audit_rollback_storage_layout
                            .recovery_region_separated),
                    ),
                    f("authorizes_append_engine", no()),
                ]),
            ),
            f(
                "storage_layout_available",
                b(evaluation.storage_layout_available),
            ),
            f("storage_layout_facts_are_append_engine_authority", no()),
        ],
        false,
    );
}

pub(crate) fn emit_module_append_engine_facts(
    engine: ModuleAuditRollbackAppendEngineCandidate,
    evaluation: ModuleAuditRollbackAppendEngineEvaluation,
) {
    emit_record_property_line(
        "append_engine_facts",
        vec![
            f(
                "audit_ledger_append_engine",
                module_append_engine_fact_record(
                    "raios.audit_ledger_append_engine.v0",
                    "append_engine.audit_ledger.current_boot",
                    "raios.audit_record.v0",
                    engine.audit_ledger_append_engine,
                    evaluation.audit_engine_status,
                    evaluation.audit_engine_reason,
                ),
            ),
            f(
                "rollback_store_transaction_engine",
                module_append_engine_fact_record(
                    "raios.rollback_store_transaction_engine.v0",
                    "append_engine.rollback_store.current_boot",
                    "raios.rollback_plan.v0",
                    engine.rollback_store_transaction_engine,
                    evaluation.rollback_engine_status,
                    evaluation.rollback_engine_reason,
                ),
            ),
        ],
        false,
    );
}

fn module_append_engine_fact_record(
    schema: &'static str,
    id: &'static str,
    target_schema: &'static str,
    fact: ModuleAuditRollbackAppendEngineFact,
    status: &'static str,
    reason: &'static str,
) -> V<'static> {
    object(vec![
        f("schema", s(schema)),
        f("id", s(id)),
        f("target_schema", s(target_schema)),
        f(
            "storage_layout_schema",
            s("raios.audit_rollback_storage_layout.v0"),
        ),
        f(
            "write_policy_schema",
            s("raios.durable_audit_write_policy.v0"),
        ),
        f("scope", s(fact.scope)),
        f("classification", s(fact.classification)),
        f("status", s(status)),
        f("reason", s(reason)),
        f("present", b(fact.present)),
        f("schema_valid", b(fact.schema_ok)),
        f("provenance_valid", b(fact.provenance_ok)),
        f("binds_storage_layout", b(fact.binds_storage_layout)),
        f("binds_write_policy", b(fact.binds_write_policy)),
        f("supports_append_only", b(fact.supports_append_only)),
        f("supports_flush", b(fact.supports_flush)),
        f("supports_replay", b(fact.supports_replay)),
        f(
            "recovery_separation_respected",
            b(fact.recovery_separation_respected),
        ),
        f("authority", s("current_snapshot")),
        f("persistence", s("none")),
        f("durable", no()),
        f("authorizes_append", no()),
        f("authorizes_write", no()),
        f("write_attempted", no()),
        f(
            "provenance",
            object(vec![
                f("source_method", s("module.audit_rollback_append_engine")),
                f("source_transport", s("serial-console")),
                f("event_scope", s("current_boot")),
                f("record_id", V::Null),
            ]),
        ),
    ])
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
