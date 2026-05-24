use crate::{agent_protocol_module_types::*, agent_protocol_support::*, event_log};

pub(crate) fn module_service_slot_allocator_method(method: &str) -> bool {
    method_head_eq(method, "module.service_slot_allocator")
}

pub(crate) fn module_service_slot_allocator_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.service_slot_allocator_selftest")
}

pub(crate) fn emit_module_service_slot_allocator() {
    let retained = event_log::latest_module_service_slot_reservation();
    let candidate = module_service_slot_allocator_snapshot(retained.is_some());
    let evaluation = evaluate_module_service_slot_allocator_candidate(candidate);

    begin_response("module.service_slot_allocator");
    raw_line("      \"schema\": \"raios.module_service_slot_allocator_readiness.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"global_event_log_mutation\": \"none\",");
    raw_line("      \"creates_service_slot_reservation_records\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"can_allocate\": false,");
    raw_line("      \"can_load_now\": false,");
    raw_line("      \"load_attempted\": false,");
    emit_module_service_slot_allocator_retained_reservation(retained);
    raw_line(",");
    emit_module_service_slot_allocator_facts(candidate, evaluation);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"readiness_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"readiness_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"retained_service_slot_reservation_present\": ");
    raw_bool(candidate.retained_reservation_present);
    raw_line(",");
    raw_line("        \"retained_hash_reference_allocates_slot\": false,");
    raw("        \"allocator_runtime_available\": ");
    raw_bool(method_eq(evaluation.allocator_runtime_status, "available"));
    raw_line(",");
    raw("        \"registry_binding_available\": ");
    raw_bool(method_eq(evaluation.registry_binding_status, "available"));
    raw_line(",");
    raw("        \"health_state_available\": ");
    raw_bool(method_eq(evaluation.health_state_status, "available"));
    raw_line(",");
    raw("        \"unload_cleanup_available\": ");
    raw_bool(method_eq(evaluation.unload_cleanup_status, "available"));
    raw_line(",");
    raw("        \"durable_audit_written\": ");
    raw_bool(candidate.durable_audit_written);
    raw_line(",");
    raw("        \"rollback_plan_installed\": ");
    raw_bool(candidate.rollback_plan_installed);
    raw_line(",");
    raw("        \"module_loader_available\": ");
    raw_bool(candidate.module_loader_available);
    raw_line(",");
    raw_line("        \"service_slot_reserved\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"can_allocate\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    emit_module_service_slot_allocator_gate(
        &mut wrote,
        "retained_service_slot_reservation",
        evaluation.retained_reservation_status,
        evaluation.retained_reservation_reason,
    );
    emit_module_service_slot_allocator_gate(
        &mut wrote,
        "service_slot_allocator_runtime",
        evaluation.allocator_runtime_status,
        evaluation.allocator_runtime_reason,
    );
    emit_module_service_slot_allocator_gate(
        &mut wrote,
        "service_slot_registry_binding",
        evaluation.registry_binding_status,
        evaluation.registry_binding_reason,
    );
    emit_module_service_slot_allocator_gate(
        &mut wrote,
        "service_health_state_model",
        evaluation.health_state_status,
        evaluation.health_state_reason,
    );
    emit_module_service_slot_allocator_gate(
        &mut wrote,
        "service_unload_cleanup_plan",
        evaluation.unload_cleanup_status,
        evaluation.unload_cleanup_reason,
    );
    emit_module_service_slot_allocator_gate(
        &mut wrote,
        "durable_audit_write",
        evaluation.durable_audit_status,
        evaluation.durable_audit_reason,
    );
    emit_module_service_slot_allocator_gate(
        &mut wrote,
        "rollback_plan_install",
        evaluation.rollback_status,
        evaluation.rollback_reason,
    );
    emit_module_service_slot_allocator_gate(
        &mut wrote,
        "module_loader",
        evaluation.module_loader_status,
        evaluation.module_loader_reason,
    );
    crlf();
    raw_line("      ]");
    end_response("module.service_slot_allocator");
}

pub(crate) fn emit_module_service_slot_allocator_selftest() {
    let cases = module_service_slot_allocator_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("module.service_slot_allocator_selftest");
    raw_line("      \"schema\": \"raios.module_service_slot_allocator_readiness_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_service_slot_reservation_records\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"can_allocate\": false,");
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
        emit_module_service_slot_allocator_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.service_slot_allocator_selftest");
}

fn emit_module_service_slot_allocator_retained_reservation(
    retained: Option<(event_log::EventId, event_log::ModuleServiceSlotReservation)>,
) {
    raw_line("      \"retained_service_slot_reservation\": {");
    if let Some((event_id, reference)) = retained {
        raw_line("        \"state\": \"present\",");
        raw_line("        \"schema\": \"raios.module_service_slot_reservation.v0\",");
        raw("        \"event_id\": ");
        json_event_id(event_id);
        raw_line(",");
        raw_line("        \"status\": \"retained_hash_reference_only_not_allocated\",");
        raw_line(
            "        \"reason\": \"service_slot_reservation_is_evidence_not_allocator_state\",",
        );
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"allocates_service_slot\": false,");
        raw_line("        \"creates_service_inventory_records\": false,");
        raw_line("        \"service_inventory_change\": \"none\",");
        raw_line("        \"can_allocate\": false,");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"load_attempted\": false,");
        raw("        \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw_line(",");
        raw("        \"retained_audit_rollback_reference_event_id\": ");
        json_event_id(reference.retained_audit_rollback_reference_event_id);
        raw_line(",");
        raw("        \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
        raw_line(",");
        raw_line("        \"hashes\": {");
        raw("          \"reservation_hash\": ");
        json_sha256(reference.reservation_hash);
        raw_line(",");
        raw("          \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw_line(",");
        raw("          \"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw_line(",");
        raw("          \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw_line(",");
        raw("          \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        crlf();
        raw_line("        }");
    } else {
        raw_line("        \"state\": \"missing\",");
        raw_line("        \"schema\": \"raios.module_service_slot_reservation.v0\",");
        raw_line("        \"event_id\": null,");
        raw_line("        \"status\": \"missing\",");
        raw_line("        \"reason\": \"retained_service_slot_reservation_missing\",");
        raw_line("        \"classification\": \"local_only\",");
        raw_line("        \"allocates_service_slot\": false,");
        raw_line("        \"creates_service_inventory_records\": false,");
        raw_line("        \"can_allocate\": false,");
        raw_line("        \"can_load_now\": false,");
        raw_line("        \"load_attempted\": false");
    }
    raw_line("      }");
}

fn emit_module_service_slot_allocator_facts(
    candidate: ModuleServiceSlotAllocatorCandidate,
    evaluation: ModuleServiceSlotAllocatorEvaluation,
) {
    raw_line("      \"allocator_readiness_facts\": {");
    emit_module_service_slot_allocator_fact(
        "service_slot_allocator_runtime",
        "raios.ram_only_service_slot_allocator.v0",
        "module.service_slot_allocator.runtime.current_boot",
        candidate.allocator_runtime,
        evaluation.allocator_runtime_status,
        evaluation.allocator_runtime_reason,
        true,
    );
    emit_module_service_slot_allocator_fact(
        "service_slot_registry_binding",
        "raios.service_slot_registry_binding.v0",
        "module.service_slot_registry.binding.current_boot",
        candidate.registry_binding,
        evaluation.registry_binding_status,
        evaluation.registry_binding_reason,
        true,
    );
    emit_module_service_slot_allocator_fact(
        "service_health_state_model",
        "raios.service_health_state_model.v0",
        "module.service_health_state.model.current_boot",
        candidate.health_state,
        evaluation.health_state_status,
        evaluation.health_state_reason,
        true,
    );
    emit_module_service_slot_allocator_fact(
        "service_unload_cleanup_plan",
        "raios.service_unload_cleanup_plan.v0",
        "module.service_unload.cleanup.current_boot",
        candidate.unload_cleanup,
        evaluation.unload_cleanup_status,
        evaluation.unload_cleanup_reason,
        false,
    );
    raw_line("      }");
}

fn emit_module_service_slot_allocator_fact(
    name: &'static str,
    schema: &'static str,
    id: &'static str,
    fact: ModuleServiceSlotAllocatorFact,
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
    raw("          \"binds_retained_service_slot_reservation\": ");
    raw_bool(fact.binds_retained_reservation);
    raw_line(",");
    raw("          \"binds_allocator_runtime\": ");
    raw_bool(fact.binds_allocator_runtime);
    raw_line(",");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"allocates_service_slot\": false,");
    raw_line("          \"creates_service_inventory_records\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"authorizes_load\": false,");
    raw_line("          \"required_bindings\": {");
    raw_line(
        "            \"service_slot_reservation\": \"raios.module_service_slot_reservation.v0\",",
    );
    raw_line(
        "            \"audit_write_boundary\": \"raios.module_audit_rollback_write_boundary.v0\",",
    );
    raw_line("            \"durable_audit_record\": \"raios.audit_record.v0\",");
    raw_line("            \"rollback_plan\": \"raios.rollback_plan.v0\",");
    raw_line("            \"module_loader\": \"raios.module_loader.v0\"");
    raw_line("          },");
    raw_line("          \"provenance\": {");
    raw_line("            \"source_method\": \"module.service_slot_allocator\",");
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

fn emit_module_service_slot_allocator_gate(
    wrote: &mut bool,
    gate: &'static str,
    state: &'static str,
    reason: &'static str,
) {
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

fn emit_module_service_slot_allocator_selftest_case(
    case: &ModuleServiceSlotAllocatorSelfTestCase,
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
    raw(", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"can_allocate\": false, \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn module_service_slot_allocator_snapshot(
    retained_reservation_present: bool,
) -> ModuleServiceSlotAllocatorCandidate {
    ModuleServiceSlotAllocatorCandidate {
        retained_reservation_present,
        allocator_runtime: module_service_slot_allocator_missing_fact(),
        registry_binding: module_service_slot_allocator_missing_fact(),
        health_state: module_service_slot_allocator_missing_fact(),
        unload_cleanup: module_service_slot_allocator_missing_fact(),
        durable_audit_written: false,
        rollback_plan_installed: false,
        module_loader_available: false,
    }
}

fn module_service_slot_allocator_missing_fact() -> ModuleServiceSlotAllocatorFact {
    ModuleServiceSlotAllocatorFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_retained_reservation: false,
        binds_allocator_runtime: false,
    }
}

fn module_service_slot_allocator_available_fact() -> ModuleServiceSlotAllocatorFact {
    ModuleServiceSlotAllocatorFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_retained_reservation: true,
        binds_allocator_runtime: true,
    }
}

fn evaluate_module_service_slot_allocator_candidate(
    candidate: ModuleServiceSlotAllocatorCandidate,
) -> ModuleServiceSlotAllocatorEvaluation {
    let retained_reservation_status = if candidate.retained_reservation_present {
        "available"
    } else {
        "missing"
    };
    let retained_reservation_reason = if candidate.retained_reservation_present {
        "retained_service_slot_reservation_available"
    } else {
        "retained_service_slot_reservation_missing"
    };

    let (allocator_runtime_status, allocator_runtime_reason) =
        evaluate_module_service_slot_allocator_fact(
            candidate.allocator_runtime,
            "service_slot_allocator_scope_must_be_current_boot",
            "service_slot_allocator_schema_mismatch",
            "service_slot_allocator_runtime_missing",
            "service_slot_allocator_provenance_missing",
            "service_slot_allocator_retained_reservation_binding_missing",
            None,
            "service_slot_allocator_runtime_available",
        );
    let (registry_binding_status, registry_binding_reason) =
        evaluate_module_service_slot_allocator_fact(
            candidate.registry_binding,
            "service_slot_registry_binding_scope_must_be_current_boot",
            "service_slot_registry_binding_schema_mismatch",
            "service_slot_registry_binding_missing",
            "service_slot_registry_binding_provenance_missing",
            "service_slot_registry_retained_reservation_binding_missing",
            Some("service_slot_registry_allocator_runtime_binding_missing"),
            "service_slot_registry_binding_available",
        );
    let (health_state_status, health_state_reason) = evaluate_module_service_slot_allocator_fact(
        candidate.health_state,
        "service_health_state_scope_must_be_current_boot",
        "service_health_state_schema_mismatch",
        "service_health_state_model_missing",
        "service_health_state_provenance_missing",
        "service_health_state_retained_reservation_binding_missing",
        Some("service_health_state_allocator_runtime_binding_missing"),
        "service_health_state_model_available",
    );
    let (unload_cleanup_status, unload_cleanup_reason) =
        evaluate_module_service_slot_allocator_fact(
            candidate.unload_cleanup,
            "service_unload_cleanup_scope_must_be_current_boot",
            "service_unload_cleanup_schema_mismatch",
            "service_unload_cleanup_plan_missing",
            "service_unload_cleanup_provenance_missing",
            "service_unload_cleanup_retained_reservation_binding_missing",
            Some("service_unload_cleanup_allocator_runtime_binding_missing"),
            "service_unload_cleanup_plan_available",
        );

    let durable_audit_status = if candidate.durable_audit_written {
        "available"
    } else {
        "missing"
    };
    let durable_audit_reason = if candidate.durable_audit_written {
        "durable_audit_write_available"
    } else {
        "durable_audit_write_missing"
    };
    let rollback_status = if candidate.rollback_plan_installed {
        "available"
    } else {
        "missing"
    };
    let rollback_reason = if candidate.rollback_plan_installed {
        "rollback_plan_install_available"
    } else {
        "rollback_install_missing"
    };
    let module_loader_status = if candidate.module_loader_available {
        "available"
    } else {
        "unavailable"
    };
    let module_loader_reason = if candidate.module_loader_available {
        "module_loader_available"
    } else {
        "module_loader_unimplemented"
    };

    let (status, reason) = if !candidate.retained_reservation_present {
        ("missing", retained_reservation_reason)
    } else if method_eq(allocator_runtime_status, "rejected") {
        ("rejected", allocator_runtime_reason)
    } else if method_eq(allocator_runtime_status, "missing") {
        ("missing", allocator_runtime_reason)
    } else if method_eq(registry_binding_status, "rejected") {
        ("rejected", registry_binding_reason)
    } else if method_eq(registry_binding_status, "missing") {
        ("missing", registry_binding_reason)
    } else if method_eq(health_state_status, "rejected") {
        ("rejected", health_state_reason)
    } else if method_eq(health_state_status, "missing") {
        ("missing", health_state_reason)
    } else if method_eq(unload_cleanup_status, "rejected") {
        ("rejected", unload_cleanup_reason)
    } else if method_eq(unload_cleanup_status, "missing") {
        ("missing", unload_cleanup_reason)
    } else if !candidate.durable_audit_written {
        ("denied_missing_durable_audit_write", durable_audit_reason)
    } else if !candidate.rollback_plan_installed {
        ("denied_missing_rollback_install", rollback_reason)
    } else if !candidate.module_loader_available {
        ("denied_loader_unimplemented", module_loader_reason)
    } else {
        (
            "denied_allocator_authority_unimplemented",
            "service_slot_allocator_authority_unimplemented",
        )
    };

    ModuleServiceSlotAllocatorEvaluation {
        status,
        reason,
        retained_reservation_status,
        retained_reservation_reason,
        allocator_runtime_status,
        allocator_runtime_reason,
        registry_binding_status,
        registry_binding_reason,
        health_state_status,
        health_state_reason,
        unload_cleanup_status,
        unload_cleanup_reason,
        durable_audit_status,
        durable_audit_reason,
        rollback_status,
        rollback_reason,
        module_loader_status,
        module_loader_reason,
        allocates_service_slot: false,
        creates_service_inventory_records: false,
        can_allocate: false,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_service_slot_allocator_fact(
    fact: ModuleServiceSlotAllocatorFact,
    scope_reason: &'static str,
    schema_reason: &'static str,
    missing_reason: &'static str,
    provenance_reason: &'static str,
    retained_reservation_reason: &'static str,
    allocator_runtime_reason: Option<&'static str>,
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
    if !fact.binds_retained_reservation {
        return ("rejected", retained_reservation_reason);
    }
    if let Some(reason) = allocator_runtime_reason {
        if !fact.binds_allocator_runtime {
            return ("rejected", reason);
        }
    }
    ("available", available_reason)
}

fn module_service_slot_allocator_selftest_cases(
) -> [ModuleServiceSlotAllocatorSelfTestCase; MODULE_SERVICE_SLOT_ALLOCATOR_SELFTEST_CASES] {
    let missing = module_service_slot_allocator_snapshot(false);
    let available = module_service_slot_allocator_available_fact();
    let ready = ModuleServiceSlotAllocatorCandidate {
        retained_reservation_present: true,
        allocator_runtime: available,
        registry_binding: available,
        health_state: available,
        unload_cleanup: available,
        durable_audit_written: true,
        rollback_plan_installed: true,
        module_loader_available: true,
    };
    [
        module_service_slot_allocator_selftest_case(
            "missing_retained_service_slot_reservation",
            "missing",
            "retained_service_slot_reservation_missing",
            missing,
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_allocator_previous_boot",
            "rejected",
            "service_slot_allocator_scope_must_be_current_boot",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: ModuleServiceSlotAllocatorFact {
                    scope: "previous_boot",
                    ..available
                },
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_allocator_wrong_schema",
            "rejected",
            "service_slot_allocator_schema_mismatch",
            ModuleServiceSlotAllocatorCandidate {
                allocator_runtime: ModuleServiceSlotAllocatorFact {
                    schema_ok: false,
                    ..available
                },
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_allocator_provenance_missing",
            "rejected",
            "service_slot_allocator_provenance_missing",
            ModuleServiceSlotAllocatorCandidate {
                allocator_runtime: ModuleServiceSlotAllocatorFact {
                    provenance_ok: false,
                    ..available
                },
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_allocator_retained_reservation_binding_missing",
            "rejected",
            "service_slot_allocator_retained_reservation_binding_missing",
            ModuleServiceSlotAllocatorCandidate {
                allocator_runtime: ModuleServiceSlotAllocatorFact {
                    binds_retained_reservation: false,
                    ..available
                },
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_allocator_runtime_missing",
            "missing",
            "service_slot_allocator_runtime_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                ..module_service_slot_allocator_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_registry_binding_missing",
            "missing",
            "service_slot_registry_binding_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: available,
                ..module_service_slot_allocator_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_registry_allocator_runtime_binding_missing",
            "rejected",
            "service_slot_registry_allocator_runtime_binding_missing",
            ModuleServiceSlotAllocatorCandidate {
                registry_binding: ModuleServiceSlotAllocatorFact {
                    binds_allocator_runtime: false,
                    ..available
                },
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_health_state_model_missing",
            "missing",
            "service_health_state_model_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: available,
                registry_binding: available,
                ..module_service_slot_allocator_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_unload_cleanup_plan_missing",
            "missing",
            "service_unload_cleanup_plan_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: available,
                registry_binding: available,
                health_state: available,
                ..module_service_slot_allocator_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "durable_audit_write_missing",
            "denied_missing_durable_audit_write",
            "durable_audit_write_missing",
            ModuleServiceSlotAllocatorCandidate {
                durable_audit_written: false,
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "rollback_install_missing",
            "denied_missing_rollback_install",
            "rollback_install_missing",
            ModuleServiceSlotAllocatorCandidate {
                rollback_plan_installed: false,
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "module_loader_missing",
            "denied_loader_unimplemented",
            "module_loader_unimplemented",
            ModuleServiceSlotAllocatorCandidate {
                module_loader_available: false,
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "all_inputs_ready_still_non_authorizing",
            "denied_allocator_authority_unimplemented",
            "service_slot_allocator_authority_unimplemented",
            ready,
        ),
    ]
}

fn module_service_slot_allocator_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleServiceSlotAllocatorCandidate,
) -> ModuleServiceSlotAllocatorSelfTestCase {
    let actual = evaluate_module_service_slot_allocator_candidate(candidate);
    ModuleServiceSlotAllocatorSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.allocates_service_slot
            && !actual.creates_service_inventory_records
            && !actual.can_allocate
            && !actual.can_load
            && !actual.load_attempted,
    }
}
