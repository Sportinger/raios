use crate::{agent_protocol_module_types::*, agent_protocol_support::*};

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

pub(crate) fn emit_module_audit_rollback_storage_layout_selftest_case(
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

pub(crate) fn module_audit_rollback_storage_layout_snapshot(
) -> ModuleAuditRollbackStorageLayoutCandidate {
    ModuleAuditRollbackStorageLayoutCandidate {
        persistence_device_inventory: module_audit_rollback_missing_persistence_device_fact(),
        audit_rollback_storage_layout: module_audit_rollback_missing_storage_layout_fact(),
    }
}

pub(crate) fn module_audit_rollback_missing_persistence_device_fact(
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

pub(crate) fn module_audit_rollback_available_persistence_device_fact(
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

pub(crate) fn module_audit_rollback_missing_storage_layout_fact(
) -> ModuleAuditRollbackStorageLayoutFact {
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

pub(crate) fn module_audit_rollback_available_storage_layout_fact(
) -> ModuleAuditRollbackStorageLayoutFact {
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

pub(crate) fn evaluate_module_audit_rollback_storage_layout_candidate(
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

pub(crate) fn evaluate_module_persistence_device_fact(
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

pub(crate) fn evaluate_module_storage_layout_fact(
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

pub(crate) fn emit_module_storage_layout_facts(
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

pub(crate) fn emit_module_persistence_device_fact(
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

pub(crate) fn emit_module_storage_layout_fact(
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

pub(crate) fn module_audit_rollback_storage_layout_selftest_cases(
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

pub(crate) fn module_audit_rollback_storage_layout_selftest_case(
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
