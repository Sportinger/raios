use crate::{agent_protocol_module_types::*, agent_protocol_support::*, event_log};

pub(crate) fn module_service_slot_allocator_method(method: &str) -> bool {
    method_head_eq(method, "module.service_slot_allocator")
}

pub(crate) fn module_service_slot_allocator_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.service_slot_allocator_selftest")
}

pub(crate) fn emit_module_service_slot_allocator() {
    let retained = event_log::latest_module_service_slot_reservation();
    let retained_event_id = retained.as_ref().map(|(event_id, _)| *event_id);
    let allocator_runtime_source_evidence = module_service_slot_allocator_runtime_source_evidence(
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0],
        retained_event_id,
        None,
    );
    let allocator_runtime_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_fact_source_evidence(
            allocator_runtime_source_evidence,
        );
    let registry_binding_source_evidence = module_service_slot_allocator_bound_fact_source_evidence(
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1],
        retained_event_id,
        Some(allocator_runtime_source_evidence_event_id),
        allocator_runtime_source_evidence,
        "service_slot_registry_binding_available",
    );
    let registry_binding_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_fact_source_evidence(
            registry_binding_source_evidence,
        );
    let health_state_source_evidence = module_service_slot_allocator_bound_fact_source_evidence(
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2],
        retained_event_id,
        Some(allocator_runtime_source_evidence_event_id),
        allocator_runtime_source_evidence,
        "service_health_state_model_available",
    );
    let health_state_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_fact_source_evidence(
            health_state_source_evidence,
        );
    let unload_cleanup_source_evidence = module_service_slot_allocator_bound_fact_source_evidence(
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3],
        retained_event_id,
        Some(allocator_runtime_source_evidence_event_id),
        allocator_runtime_source_evidence,
        "service_unload_cleanup_plan_available",
    );
    let unload_cleanup_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_fact_source_evidence(
            unload_cleanup_source_evidence,
        );
    let allocator_runtime_observed_source_evidence =
        event_log::latest_module_service_slot_allocator_fact_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0].source_fact_locator,
        )
        .unwrap_or((
            allocator_runtime_source_evidence_event_id,
            allocator_runtime_source_evidence,
        ));
    let registry_binding_observed_source_evidence =
        event_log::latest_module_service_slot_allocator_fact_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1].source_fact_locator,
        )
        .unwrap_or((
            registry_binding_source_evidence_event_id,
            registry_binding_source_evidence,
        ));
    let health_state_observed_source_evidence =
        event_log::latest_module_service_slot_allocator_fact_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2].source_fact_locator,
        )
        .unwrap_or((
            health_state_source_evidence_event_id,
            health_state_source_evidence,
        ));
    let unload_cleanup_observed_source_evidence =
        event_log::latest_module_service_slot_allocator_fact_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3].source_fact_locator,
        )
        .unwrap_or((
            unload_cleanup_source_evidence_event_id,
            unload_cleanup_source_evidence,
        ));
    let durable_audit_source_evidence =
        module_service_slot_allocator_bound_prerequisite_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0],
            retained.is_some(),
            allocator_runtime_observed_source_evidence,
            registry_binding_observed_source_evidence,
            health_state_observed_source_evidence,
            unload_cleanup_observed_source_evidence,
            "durable_audit_write_available",
        );
    let durable_audit_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_prerequisite_source_evidence(
            durable_audit_source_evidence,
        );
    let rollback_install_source_evidence =
        module_service_slot_allocator_bound_prerequisite_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1],
            retained.is_some(),
            allocator_runtime_observed_source_evidence,
            registry_binding_observed_source_evidence,
            health_state_observed_source_evidence,
            unload_cleanup_observed_source_evidence,
            "rollback_plan_install_available",
        );
    let rollback_install_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_prerequisite_source_evidence(
            rollback_install_source_evidence,
        );
    let module_loader_source_evidence =
        module_service_slot_allocator_module_loader_prerequisite_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2],
            retained.is_some(),
            allocator_runtime_observed_source_evidence,
            registry_binding_observed_source_evidence,
            health_state_observed_source_evidence,
            unload_cleanup_observed_source_evidence,
            durable_audit_source_evidence,
            rollback_install_source_evidence,
        );
    let module_loader_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_prerequisite_source_evidence(
            module_loader_source_evidence,
        );
    let durable_audit_observed_source_evidence =
        event_log::latest_module_service_slot_allocator_prerequisite_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0].source_fact_locator,
        )
        .unwrap_or((
            durable_audit_source_evidence_event_id,
            durable_audit_source_evidence,
        ));
    let rollback_install_observed_source_evidence =
        event_log::latest_module_service_slot_allocator_prerequisite_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1].source_fact_locator,
        )
        .unwrap_or((
            rollback_install_source_evidence_event_id,
            rollback_install_source_evidence,
        ));
    let module_loader_observed_source_evidence =
        event_log::latest_module_service_slot_allocator_prerequisite_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2].source_fact_locator,
        )
        .unwrap_or((
            module_loader_source_evidence_event_id,
            module_loader_source_evidence,
        ));
    let allocator_authority_source_evidence =
        module_service_slot_allocator_authority_source_evidence(
            retained.is_some(),
            allocator_runtime_observed_source_evidence,
            registry_binding_observed_source_evidence,
            health_state_observed_source_evidence,
            unload_cleanup_observed_source_evidence,
            durable_audit_observed_source_evidence,
            rollback_install_observed_source_evidence,
            module_loader_observed_source_evidence,
        );
    let allocator_authority_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_authority_source_evidence(
            allocator_authority_source_evidence,
        );
    let load_gate_binding = event_log::module_load_gate_binding_snapshot();
    let allocation_intent_source_evidence = module_service_slot_allocation_intent_source_evidence(
        load_gate_binding,
        retained.is_some(),
        (
            allocator_authority_source_evidence_event_id,
            allocator_authority_source_evidence,
        ),
    );
    let allocation_intent_source_evidence_event_id =
        event_log::record_module_service_slot_allocation_intent_source_evidence(
            allocation_intent_source_evidence,
        );
    let policy_decision_source_evidence = module_service_slot_authority_input_source_evidence(
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0],
        load_gate_binding,
        (
            allocation_intent_source_evidence_event_id,
            allocation_intent_source_evidence,
        ),
        MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SCHEMA,
        Some(allocation_intent_source_evidence_event_id),
        allocation_intent_source_evidence.intent_present
            && allocation_intent_source_evidence.source_chain_complete,
    );
    let policy_decision_source_evidence_event_id =
        event_log::record_module_service_slot_authority_input_source_evidence(
            policy_decision_source_evidence,
        );
    let registry_write_authority_source_evidence =
        module_service_slot_authority_input_source_evidence(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[1],
            load_gate_binding,
            (
                allocation_intent_source_evidence_event_id,
                allocation_intent_source_evidence,
            ),
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0].schema,
            Some(policy_decision_source_evidence_event_id),
            policy_decision_source_evidence.input_present
                && policy_decision_source_evidence.source_chain_complete,
        );
    let registry_write_authority_source_evidence_event_id =
        event_log::record_module_service_slot_authority_input_source_evidence(
            registry_write_authority_source_evidence,
        );
    let loader_runtime_contract_source_evidence =
        module_service_slot_authority_input_source_evidence(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[2],
            load_gate_binding,
            (
                allocation_intent_source_evidence_event_id,
                allocation_intent_source_evidence,
            ),
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[1].schema,
            Some(registry_write_authority_source_evidence_event_id),
            registry_write_authority_source_evidence.input_present
                && registry_write_authority_source_evidence.source_chain_complete,
        );
    let loader_runtime_contract_source_evidence_event_id =
        event_log::record_module_service_slot_authority_input_source_evidence(
            loader_runtime_contract_source_evidence,
        );
    let health_monitor_binding_source_evidence =
        module_service_slot_authority_input_source_evidence(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[3],
            load_gate_binding,
            (
                allocation_intent_source_evidence_event_id,
                allocation_intent_source_evidence,
            ),
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[2].schema,
            Some(loader_runtime_contract_source_evidence_event_id),
            loader_runtime_contract_source_evidence.input_present
                && loader_runtime_contract_source_evidence.source_chain_complete,
        );
    let health_monitor_binding_source_evidence_event_id =
        event_log::record_module_service_slot_authority_input_source_evidence(
            health_monitor_binding_source_evidence,
        );
    let unload_cleanup_authority_source_evidence =
        module_service_slot_authority_input_source_evidence(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[4],
            load_gate_binding,
            (
                allocation_intent_source_evidence_event_id,
                allocation_intent_source_evidence,
            ),
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[3].schema,
            Some(health_monitor_binding_source_evidence_event_id),
            health_monitor_binding_source_evidence.input_present
                && health_monitor_binding_source_evidence.source_chain_complete,
        );
    let unload_cleanup_authority_source_evidence_event_id =
        event_log::record_module_service_slot_authority_input_source_evidence(
            unload_cleanup_authority_source_evidence,
        );
    let authority_input_source_evidence = [
        (
            policy_decision_source_evidence_event_id,
            policy_decision_source_evidence,
        ),
        (
            registry_write_authority_source_evidence_event_id,
            registry_write_authority_source_evidence,
        ),
        (
            loader_runtime_contract_source_evidence_event_id,
            loader_runtime_contract_source_evidence,
        ),
        (
            health_monitor_binding_source_evidence_event_id,
            health_monitor_binding_source_evidence,
        ),
        (
            unload_cleanup_authority_source_evidence_event_id,
            unload_cleanup_authority_source_evidence,
        ),
    ];
    let authority_decision_source_evidence =
        module_service_slot_allocator_authority_decision_source_evidence(
            load_gate_binding,
            (
                allocator_authority_source_evidence_event_id,
                allocator_authority_source_evidence,
            ),
            (
                allocation_intent_source_evidence_event_id,
                allocation_intent_source_evidence,
            ),
            authority_input_source_evidence,
        );
    let authority_decision_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_authority_decision_source_evidence(
            authority_decision_source_evidence,
        );
    let registry_write_commit_gate_source_evidence =
        module_service_slot_registry_write_commit_gate_source_evidence(
            load_gate_binding,
            (
                authority_decision_source_evidence_event_id,
                authority_decision_source_evidence,
            ),
            (
                registry_write_authority_source_evidence_event_id,
                registry_write_authority_source_evidence,
            ),
            registry_binding_observed_source_evidence,
            durable_audit_observed_source_evidence,
            rollback_install_observed_source_evidence,
        );
    let registry_write_commit_gate_source_evidence_event_id =
        event_log::record_module_service_slot_registry_write_commit_gate_source_evidence(
            registry_write_commit_gate_source_evidence,
        );
    let candidate = module_service_slot_allocator_snapshot(
        retained.is_some(),
        Some(allocator_runtime_observed_source_evidence),
        Some(registry_binding_observed_source_evidence),
        Some(health_state_observed_source_evidence),
        Some(unload_cleanup_observed_source_evidence),
        Some(durable_audit_observed_source_evidence),
        Some(rollback_install_observed_source_evidence),
        Some(module_loader_observed_source_evidence),
        Some((
            allocator_authority_source_evidence_event_id,
            allocator_authority_source_evidence,
        )),
        Some((
            allocation_intent_source_evidence_event_id,
            allocation_intent_source_evidence,
        )),
        Some(authority_input_source_evidence),
        Some((
            authority_decision_source_evidence_event_id,
            authority_decision_source_evidence,
        )),
        Some((
            registry_write_commit_gate_source_evidence_event_id,
            registry_write_commit_gate_source_evidence,
        )),
    );
    let evaluation = evaluate_module_service_slot_allocator_candidate(candidate);

    begin_response("module.service_slot_allocator");
    raw_line("      \"schema\": \"raios.module_service_slot_allocator_readiness.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": true,");
    raw_line(
        "      \"global_event_log_mutation\": \"retained_current_boot_source_evidence_only\",",
    );
    raw_line("      \"creates_service_slot_reservation_records\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"can_allocate\": false,");
    raw_line("      \"can_load_now\": false,");
    raw_line("      \"load_attempted\": false,");
    emit_module_service_slot_allocator_source_evidence(
        allocator_runtime_source_evidence_event_id,
        allocator_runtime_source_evidence,
        registry_binding_source_evidence_event_id,
        registry_binding_source_evidence,
        health_state_source_evidence_event_id,
        health_state_source_evidence,
        unload_cleanup_source_evidence_event_id,
        unload_cleanup_source_evidence,
        durable_audit_source_evidence_event_id,
        durable_audit_source_evidence,
        rollback_install_source_evidence_event_id,
        rollback_install_source_evidence,
        module_loader_source_evidence_event_id,
        module_loader_source_evidence,
        allocator_authority_source_evidence_event_id,
        allocator_authority_source_evidence,
        allocation_intent_source_evidence_event_id,
        allocation_intent_source_evidence,
        authority_input_source_evidence,
        authority_decision_source_evidence_event_id,
        authority_decision_source_evidence,
        registry_write_commit_gate_source_evidence_event_id,
        registry_write_commit_gate_source_evidence,
    );
    raw_line(",");
    emit_module_service_slot_allocator_retained_reservation(retained);
    raw_line(",");
    emit_module_service_slot_allocator_facts(candidate, evaluation);
    raw_line(",");
    emit_module_service_slot_allocator_prerequisites(candidate, evaluation);
    raw_line(",");
    emit_module_service_slot_allocator_authority(candidate, evaluation);
    raw_line(",");
    emit_module_service_slot_allocation_intent(candidate, evaluation);
    raw_line(",");
    emit_module_service_slot_authority_inputs(candidate, evaluation);
    raw_line(",");
    emit_module_service_slot_allocator_authority_decision(candidate, evaluation);
    raw_line(",");
    emit_module_service_slot_registry_write_commit_gate(candidate, evaluation);
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
    raw_bool(candidate.durable_audit_write.available);
    raw_line(",");
    raw("        \"rollback_plan_installed\": ");
    raw_bool(candidate.rollback_plan_install.available);
    raw_line(",");
    raw("        \"module_loader_available\": ");
    raw_bool(candidate.module_loader.available);
    raw_line(",");
    raw("        \"allocator_authority_status\": ");
    json_str(evaluation.authority_status);
    raw_line(",");
    raw("        \"allocator_authority_reason\": ");
    json_str(evaluation.authority_reason);
    raw_line(",");
    raw("        \"allocation_intent_status\": ");
    json_str(evaluation.allocation_intent_status);
    raw_line(",");
    raw("        \"allocation_intent_reason\": ");
    json_str(evaluation.allocation_intent_reason);
    raw_line(",");
    raw_line("        \"authority_input_statuses\": {");
    let mut input_idx = 0usize;
    while input_idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        raw("          ");
        json_str(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[input_idx].name);
        raw(": {\"schema\": ");
        json_str(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[input_idx].schema);
        raw(", \"status\": ");
        json_str(evaluation.authority_input_statuses[input_idx]);
        raw(", \"reason\": ");
        json_str(evaluation.authority_input_reasons[input_idx]);
        raw("}");
        if input_idx + 1 != MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
            raw(",");
        }
        crlf();
        input_idx += 1;
    }
    raw_line("        },");
    raw("        \"authority_decision_status\": ");
    json_str(evaluation.authority_decision_status);
    raw_line(",");
    raw("        \"authority_decision_reason\": ");
    json_str(evaluation.authority_decision_reason);
    raw_line(",");
    raw("        \"registry_write_commit_gate_status\": ");
    json_str(evaluation.registry_write_commit_gate_status);
    raw_line(",");
    raw("        \"registry_write_commit_gate_reason\": ");
    json_str(evaluation.registry_write_commit_gate_reason);
    raw_line(",");
    raw_line("        \"service_slot_reserved\": false,");
    raw_line("        \"registry_write_committed\": false,");
    raw_line("        \"mutates_service_registry\": false,");
    raw_line("        \"writes_durable_audit_state\": false,");
    raw_line("        \"installs_rollback_state\": false,");
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
    emit_module_service_slot_allocator_gate(
        &mut wrote,
        "service_slot_allocator_authority",
        evaluation.authority_status,
        evaluation.authority_reason,
    );
    emit_module_service_slot_allocator_gate(
        &mut wrote,
        "service_slot_allocation_intent",
        evaluation.allocation_intent_status,
        evaluation.allocation_intent_reason,
    );
    let mut input_idx = 0usize;
    while input_idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        emit_module_service_slot_allocator_gate(
            &mut wrote,
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[input_idx].name,
            evaluation.authority_input_statuses[input_idx],
            evaluation.authority_input_reasons[input_idx],
        );
        input_idx += 1;
    }
    emit_module_service_slot_allocator_gate(
        &mut wrote,
        "service_slot_allocator_authority_decision",
        evaluation.authority_decision_status,
        evaluation.authority_decision_reason,
    );
    emit_module_service_slot_allocator_gate(
        &mut wrote,
        "service_slot_registry_write_commit_gate",
        evaluation.registry_write_commit_gate_status,
        evaluation.registry_write_commit_gate_reason,
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

fn emit_module_service_slot_allocator_source_evidence(
    allocator_runtime_event_id: event_log::EventId,
    allocator_runtime: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    registry_binding_event_id: event_log::EventId,
    registry_binding: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    health_state_event_id: event_log::EventId,
    health_state: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    unload_cleanup_event_id: event_log::EventId,
    unload_cleanup: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    durable_audit_event_id: event_log::EventId,
    durable_audit: event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    rollback_install_event_id: event_log::EventId,
    rollback_install: event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    module_loader_event_id: event_log::EventId,
    module_loader: event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    allocator_authority_event_id: event_log::EventId,
    allocator_authority: event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    allocation_intent_event_id: event_log::EventId,
    allocation_intent: event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
    authority_inputs: [(
        event_log::EventId,
        event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
    ); MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
    authority_decision_event_id: event_log::EventId,
    authority_decision: event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
    registry_write_commit_gate_event_id: event_log::EventId,
    registry_write_commit_gate: event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence,
) {
    raw_line("      \"source_evidence\": [");
    emit_module_service_slot_allocator_source_evidence_item(
        allocator_runtime_event_id,
        allocator_runtime,
        true,
    );
    emit_module_service_slot_allocator_source_evidence_item(
        registry_binding_event_id,
        registry_binding,
        true,
    );
    emit_module_service_slot_allocator_source_evidence_item(
        health_state_event_id,
        health_state,
        true,
    );
    emit_module_service_slot_allocator_source_evidence_item(
        unload_cleanup_event_id,
        unload_cleanup,
        true,
    );
    emit_module_service_slot_allocator_prerequisite_source_evidence_item(
        durable_audit_event_id,
        durable_audit,
        true,
    );
    emit_module_service_slot_allocator_prerequisite_source_evidence_item(
        rollback_install_event_id,
        rollback_install,
        true,
    );
    emit_module_service_slot_allocator_prerequisite_source_evidence_item(
        module_loader_event_id,
        module_loader,
        true,
    );
    emit_module_service_slot_allocator_authority_source_evidence_item(
        allocator_authority_event_id,
        allocator_authority,
        true,
    );
    emit_module_service_slot_allocation_intent_source_evidence_item(
        allocation_intent_event_id,
        allocation_intent,
        true,
    );
    let mut idx = 0usize;
    while idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        let (event_id, evidence) = authority_inputs[idx];
        emit_module_service_slot_authority_input_source_evidence_item(event_id, evidence, true);
        idx += 1;
    }
    emit_module_service_slot_allocator_authority_decision_source_evidence_item(
        authority_decision_event_id,
        authority_decision,
        true,
    );
    emit_module_service_slot_registry_write_commit_gate_source_evidence_item(
        registry_write_commit_gate_event_id,
        registry_write_commit_gate,
        false,
    );
    raw_line("      ]");
}

fn emit_module_service_slot_allocator_source_evidence_item(
    event_id: event_log::EventId,
    evidence: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    comma: bool,
) {
    raw_line("        {");
    raw_line("          \"kind\": \"allocator_fact\",");
    raw("          \"event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("          \"schema\": ");
    json_str(evidence.schema);
    raw_line(",");
    raw_line("          \"status\": \"retained_current_boot_source_evidence\",");
    raw_line(
        "          \"reason\": \"module_service_slot_allocator_fact_source_evidence_recorded\",",
    );
    raw("          \"fact_schema\": ");
    json_str(evidence.fact_schema);
    raw_line(",");
    raw("          \"fact_id\": ");
    json_str(evidence.fact_id);
    raw_line(",");
    raw("          \"source_method\": ");
    json_str(evidence.source_method);
    raw_line(",");
    raw("          \"source_fact_locator\": ");
    json_str(evidence.source_fact_locator);
    raw_line(",");
    raw("          \"fact_status\": ");
    json_str(evidence.fact_status);
    raw_line(",");
    raw("          \"fact_reason\": ");
    json_str(evidence.fact_reason);
    raw_line(",");
    raw("          \"fact_present\": ");
    raw_bool(evidence.fact_present);
    raw_line(",");
    raw("          \"retained_service_slot_reservation_event_id\": ");
    json_event_id_option(evidence.retained_service_slot_reservation_event_id);
    raw_line(",");
    raw("          \"allocator_runtime_source_evidence_event_id\": ");
    json_event_id_option(evidence.allocator_runtime_source_evidence_event_id);
    raw_line(",");
    raw("          \"source_evidence_retained\": true,");
    raw_line("          \"retention\": \"current_boot_ram_event_log\",");
    raw_line("          \"allocates_service_slot\": false,");
    raw_line("          \"creates_service_inventory_records\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"can_load_now\": false,");
    raw_line("          \"load_attempted\": false");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_service_slot_allocator_prerequisite_source_evidence_item(
    event_id: event_log::EventId,
    evidence: event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    comma: bool,
) {
    raw_line("        {");
    raw_line("          \"kind\": \"allocator_prerequisite\",");
    raw("          \"event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("          \"schema\": ");
    json_str(evidence.schema);
    raw_line(",");
    raw_line("          \"status\": \"retained_current_boot_source_evidence\",");
    raw_line(
        "          \"reason\": \"module_service_slot_allocator_prerequisite_source_evidence_recorded\",",
    );
    raw("          \"prerequisite_schema\": ");
    json_str(evidence.prerequisite_schema);
    raw_line(",");
    raw("          \"prerequisite_id\": ");
    json_str(evidence.prerequisite_id);
    raw_line(",");
    raw("          \"source_method\": ");
    json_str(evidence.source_method);
    raw_line(",");
    raw("          \"source_fact_locator\": ");
    json_str(evidence.source_fact_locator);
    raw_line(",");
    raw("          \"prerequisite_status\": ");
    json_str(evidence.prerequisite_status);
    raw_line(",");
    raw("          \"prerequisite_reason\": ");
    json_str(evidence.prerequisite_reason);
    raw_line(",");
    raw("          \"prerequisite_available\": ");
    raw_bool(evidence.prerequisite_available);
    raw_line(",");
    raw("          \"allocator_runtime_source_evidence_event_id\": ");
    json_event_id_option(evidence.allocator_runtime_source_evidence_event_id);
    raw_line(",");
    raw("          \"registry_binding_source_evidence_event_id\": ");
    json_event_id_option(evidence.registry_binding_source_evidence_event_id);
    raw_line(",");
    raw("          \"health_state_source_evidence_event_id\": ");
    json_event_id_option(evidence.health_state_source_evidence_event_id);
    raw_line(",");
    raw("          \"unload_cleanup_source_evidence_event_id\": ");
    json_event_id_option(evidence.unload_cleanup_source_evidence_event_id);
    raw_line(",");
    raw("          \"source_evidence_retained\": true,");
    raw_line("          \"retention\": \"current_boot_ram_event_log\",");
    raw_line("          \"allocates_service_slot\": false,");
    raw_line("          \"creates_service_inventory_records\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"can_load_now\": false,");
    raw_line("          \"load_attempted\": false");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_service_slot_allocator_authority_source_evidence_item(
    event_id: event_log::EventId,
    evidence: event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    comma: bool,
) {
    raw_line("        {");
    raw_line("          \"kind\": \"allocator_authority\",");
    raw("          \"event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("          \"schema\": ");
    json_str(evidence.schema);
    raw_line(",");
    raw_line("          \"status\": \"retained_current_boot_source_evidence\",");
    raw_line(
        "          \"reason\": \"module_service_slot_allocator_authority_source_evidence_recorded\",",
    );
    raw("          \"authority_schema\": ");
    json_str(evidence.authority_schema);
    raw_line(",");
    raw("          \"authority_id\": ");
    json_str(evidence.authority_id);
    raw_line(",");
    raw("          \"source_method\": ");
    json_str(evidence.source_method);
    raw_line(",");
    raw("          \"source_fact_locator\": ");
    json_str(evidence.source_fact_locator);
    raw_line(",");
    raw("          \"authority_status\": ");
    json_str(evidence.authority_status);
    raw_line(",");
    raw("          \"authority_reason\": ");
    json_str(evidence.authority_reason);
    raw_line(",");
    raw("          \"authority_scope\": ");
    json_str(evidence.authority_scope);
    raw_line(",");
    raw("          \"authority_classification\": ");
    json_str(evidence.authority_classification);
    raw_line(",");
    raw("          \"authority_present\": ");
    raw_bool(evidence.authority_present);
    raw_line(",");
    raw("          \"retained_service_slot_reservation_present\": ");
    raw_bool(evidence.retained_service_slot_reservation_present);
    raw_line(",");
    raw("          \"allocator_runtime_available\": ");
    raw_bool(evidence.allocator_runtime_available);
    raw_line(",");
    raw("          \"registry_binding_available\": ");
    raw_bool(evidence.registry_binding_available);
    raw_line(",");
    raw("          \"health_state_available\": ");
    raw_bool(evidence.health_state_available);
    raw_line(",");
    raw("          \"unload_cleanup_available\": ");
    raw_bool(evidence.unload_cleanup_available);
    raw_line(",");
    raw("          \"durable_audit_write_available\": ");
    raw_bool(evidence.durable_audit_write_available);
    raw_line(",");
    raw("          \"rollback_plan_install_available\": ");
    raw_bool(evidence.rollback_plan_install_available);
    raw_line(",");
    raw("          \"module_loader_available\": ");
    raw_bool(evidence.module_loader_available);
    raw_line(",");
    raw("          \"source_chain_complete\": ");
    raw_bool(evidence.source_chain_complete);
    raw_line(",");
    raw("          \"allocator_runtime_source_evidence_event_id\": ");
    json_event_id_option(evidence.allocator_runtime_source_evidence_event_id);
    raw_line(",");
    raw("          \"registry_binding_source_evidence_event_id\": ");
    json_event_id_option(evidence.registry_binding_source_evidence_event_id);
    raw_line(",");
    raw("          \"health_state_source_evidence_event_id\": ");
    json_event_id_option(evidence.health_state_source_evidence_event_id);
    raw_line(",");
    raw("          \"unload_cleanup_source_evidence_event_id\": ");
    json_event_id_option(evidence.unload_cleanup_source_evidence_event_id);
    raw_line(",");
    raw("          \"durable_audit_source_evidence_event_id\": ");
    json_event_id_option(evidence.durable_audit_source_evidence_event_id);
    raw_line(",");
    raw("          \"rollback_install_source_evidence_event_id\": ");
    json_event_id_option(evidence.rollback_install_source_evidence_event_id);
    raw_line(",");
    raw("          \"module_loader_source_evidence_event_id\": ");
    json_event_id_option(evidence.module_loader_source_evidence_event_id);
    raw_line(",");
    raw_line("          \"source_evidence_retained\": true,");
    raw_line("          \"retention\": \"current_boot_ram_event_log\",");
    raw_line("          \"allocates_service_slot\": false,");
    raw_line("          \"creates_service_inventory_records\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"can_load_now\": false,");
    raw_line("          \"load_attempted\": false");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_service_slot_allocation_intent_source_evidence_item(
    event_id: event_log::EventId,
    evidence: event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
    comma: bool,
) {
    raw_line("        {");
    raw_line("          \"kind\": \"allocation_intent\",");
    raw("          \"event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("          \"schema\": ");
    json_str(evidence.schema);
    raw_line(",");
    raw_line("          \"status\": \"retained_current_boot_source_evidence\",");
    raw_line(
        "          \"reason\": \"module_service_slot_allocation_intent_source_evidence_recorded\",",
    );
    raw("          \"intent_schema\": ");
    json_str(evidence.intent_schema);
    raw_line(",");
    raw("          \"intent_id\": ");
    json_str(evidence.intent_id);
    raw_line(",");
    raw("          \"source_method\": ");
    json_str(evidence.source_method);
    raw_line(",");
    raw("          \"source_fact_locator\": ");
    json_str(evidence.source_fact_locator);
    raw_line(",");
    raw("          \"intent_status\": ");
    json_str(evidence.intent_status);
    raw_line(",");
    raw("          \"intent_reason\": ");
    json_str(evidence.intent_reason);
    raw_line(",");
    raw("          \"intent_present\": ");
    raw_bool(evidence.intent_present);
    raw_line(",");
    raw("          \"intent_scope\": ");
    json_str(evidence.intent_scope);
    raw_line(",");
    raw("          \"requested_capability\": ");
    json_str(evidence.requested_capability);
    raw_line(",");
    raw("          \"load_mode\": ");
    json_str(evidence.load_mode);
    raw_line(",");
    raw("          \"target\": ");
    json_str(evidence.target);
    raw_line(",");
    raw("          \"retained_module_evidence_present\": ");
    raw_bool(evidence.retained_module_evidence_present);
    raw_line(",");
    raw("          \"retained_service_slot_reservation_present\": ");
    raw_bool(evidence.retained_service_slot_reservation_present);
    raw_line(",");
    raw("          \"allocator_authority_present\": ");
    raw_bool(evidence.allocator_authority_present);
    raw_line(",");
    raw("          \"source_chain_complete\": ");
    raw_bool(evidence.source_chain_complete);
    raw_line(",");
    raw("          \"manifest_reference_event_id\": ");
    json_event_id_option(evidence.manifest_reference_event_id);
    raw_line(",");
    raw("          \"candidate_artifact_reference_event_id\": ");
    json_event_id_option(evidence.artifact_reference_event_id);
    raw_line(",");
    raw("          \"vm_test_report_reference_event_id\": ");
    json_event_id_option(evidence.vm_report_reference_event_id);
    raw_line(",");
    raw("          \"local_attestation_reference_event_id\": ");
    json_event_id_option(evidence.local_attestation_reference_event_id);
    raw_line(",");
    raw("          \"local_approval_reference_event_id\": ");
    json_event_id_option(evidence.local_approval_reference_event_id);
    raw_line(",");
    raw("          \"computed_grant_reference_event_id\": ");
    json_event_id_option(evidence.computed_grant_reference_event_id);
    raw_line(",");
    raw("          \"audit_rollback_reference_event_id\": ");
    json_event_id_option(evidence.audit_rollback_reference_event_id);
    raw_line(",");
    raw("          \"service_slot_reservation_event_id\": ");
    json_event_id_option(evidence.service_slot_reservation_event_id);
    raw_line(",");
    raw("          \"allocator_authority_source_evidence_event_id\": ");
    json_event_id_option(evidence.allocator_authority_source_evidence_event_id);
    raw_line(",");
    raw("          \"ram_only_service_slot_id\": ");
    if let Some(id) = evidence.ram_only_service_slot_id {
        json_str(id.as_str());
    } else {
        raw("null");
    }
    raw_line(",");
    raw_line("          \"source_evidence_retained\": true,");
    raw_line("          \"retention\": \"current_boot_ram_event_log\",");
    raw_line("          \"allocates_service_slot\": false,");
    raw_line("          \"creates_service_inventory_records\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"can_load_now\": false,");
    raw_line("          \"load_attempted\": false");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_service_slot_authority_input_source_evidence_item(
    event_id: event_log::EventId,
    evidence: event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
    comma: bool,
) {
    raw_line("        {");
    raw_line("          \"kind\": \"authority_input\",");
    raw("          \"event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("          \"schema\": ");
    json_str(evidence.schema);
    raw_line(",");
    raw_line("          \"status\": \"retained_current_boot_source_evidence\",");
    raw_line(
        "          \"reason\": \"module_service_slot_authority_input_source_evidence_recorded\",",
    );
    raw("          \"input_schema\": ");
    json_str(evidence.input_schema);
    raw_line(",");
    raw("          \"input_id\": ");
    json_str(evidence.input_id);
    raw_line(",");
    raw("          \"input_name\": ");
    json_str(evidence.input_name);
    raw_line(",");
    raw("          \"source_method\": ");
    json_str(evidence.source_method);
    raw_line(",");
    raw("          \"source_fact_locator\": ");
    json_str(evidence.source_fact_locator);
    raw_line(",");
    raw("          \"input_status\": ");
    json_str(evidence.input_status);
    raw_line(",");
    raw("          \"input_reason\": ");
    json_str(evidence.input_reason);
    raw_line(",");
    raw("          \"input_present\": ");
    raw_bool(evidence.input_present);
    raw_line(",");
    raw("          \"input_scope\": ");
    json_str(evidence.input_scope);
    raw_line(",");
    raw("          \"dependency_schema\": ");
    json_str(evidence.dependency_schema);
    raw_line(",");
    raw("          \"dependency_source_evidence_event_id\": ");
    json_event_id_option(evidence.dependency_source_evidence_event_id);
    raw_line(",");
    raw("          \"dependency_present\": ");
    raw_bool(evidence.dependency_present);
    raw_line(",");
    raw("          \"requested_capability\": ");
    json_str(evidence.requested_capability);
    raw_line(",");
    raw("          \"load_mode\": ");
    json_str(evidence.load_mode);
    raw_line(",");
    raw("          \"target\": ");
    json_str(evidence.target);
    raw_line(",");
    raw("          \"retained_module_evidence_present\": ");
    raw_bool(evidence.retained_module_evidence_present);
    raw_line(",");
    raw("          \"retained_service_slot_reservation_present\": ");
    raw_bool(evidence.retained_service_slot_reservation_present);
    raw_line(",");
    raw("          \"allocator_authority_present\": ");
    raw_bool(evidence.allocator_authority_present);
    raw_line(",");
    raw("          \"allocation_intent_source_evidence_event_id\": ");
    json_event_id_option(evidence.allocation_intent_source_evidence_event_id);
    raw_line(",");
    raw("          \"source_chain_complete\": ");
    raw_bool(evidence.source_chain_complete);
    raw_line(",");
    raw("          \"service_slot_reservation_event_id\": ");
    json_event_id_option(evidence.service_slot_reservation_event_id);
    raw_line(",");
    raw("          \"allocator_authority_source_evidence_event_id\": ");
    json_event_id_option(evidence.allocator_authority_source_evidence_event_id);
    raw_line(",");
    raw("          \"ram_only_service_slot_id\": ");
    if let Some(id) = evidence.ram_only_service_slot_id {
        json_str(id.as_str());
    } else {
        raw("null");
    }
    raw_line(",");
    raw_line("          \"source_evidence_retained\": true,");
    raw_line("          \"retention\": \"current_boot_ram_event_log\",");
    raw_line("          \"allocates_service_slot\": false,");
    raw_line("          \"creates_service_inventory_records\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"can_load_now\": false,");
    raw_line("          \"load_attempted\": false");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_service_slot_allocator_authority_decision_source_evidence_item(
    event_id: event_log::EventId,
    evidence: event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
    comma: bool,
) {
    raw_line("        {");
    raw_line("          \"kind\": \"authority_decision\",");
    raw("          \"event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("          \"schema\": ");
    json_str(evidence.schema);
    raw_line(",");
    raw_line("          \"status\": \"retained_current_boot_source_evidence\",");
    raw_line(
        "          \"reason\": \"module_service_slot_allocator_authority_decision_source_evidence_recorded\",",
    );
    raw("          \"decision_schema\": ");
    json_str(evidence.decision_schema);
    raw_line(",");
    raw("          \"decision_id\": ");
    json_str(evidence.decision_id);
    raw_line(",");
    raw("          \"source_method\": ");
    json_str(evidence.source_method);
    raw_line(",");
    raw("          \"source_fact_locator\": ");
    json_str(evidence.source_fact_locator);
    raw_line(",");
    raw("          \"decision_status\": ");
    json_str(evidence.decision_status);
    raw_line(",");
    raw("          \"decision_reason\": ");
    json_str(evidence.decision_reason);
    raw_line(",");
    raw("          \"decision_present\": ");
    raw_bool(evidence.decision_present);
    raw_line(",");
    raw("          \"decision_scope\": ");
    json_str(evidence.decision_scope);
    raw_line(",");
    raw("          \"requested_capability\": ");
    json_str(evidence.requested_capability);
    raw_line(",");
    raw("          \"load_mode\": ");
    json_str(evidence.load_mode);
    raw_line(",");
    raw("          \"target\": ");
    json_str(evidence.target);
    raw_line(",");
    raw("          \"allocator_authority_present\": ");
    raw_bool(evidence.allocator_authority_present);
    raw_line(",");
    raw("          \"allocation_intent_present\": ");
    raw_bool(evidence.allocation_intent_present);
    raw_line(",");
    raw("          \"authority_inputs_complete\": ");
    raw_bool(evidence.authority_inputs_complete);
    raw_line(",");
    raw("          \"source_chain_complete\": ");
    raw_bool(evidence.source_chain_complete);
    raw_line(",");
    raw("          \"allocator_authority_source_evidence_event_id\": ");
    json_event_id_option(evidence.allocator_authority_source_evidence_event_id);
    raw_line(",");
    raw("          \"allocation_intent_source_evidence_event_id\": ");
    json_event_id_option(evidence.allocation_intent_source_evidence_event_id);
    raw_line(",");
    raw("          \"authority_input_source_evidence_event_ids\": [");
    let mut idx = 0usize;
    while idx < evidence.authority_input_source_evidence_event_ids.len() {
        json_event_id_option(evidence.authority_input_source_evidence_event_ids[idx]);
        if idx + 1 != evidence.authority_input_source_evidence_event_ids.len() {
            raw(", ");
        }
        idx += 1;
    }
    raw_line("],");
    raw("          \"authority_input_present\": [");
    idx = 0;
    while idx < evidence.authority_input_present.len() {
        raw_bool(evidence.authority_input_present[idx]);
        if idx + 1 != evidence.authority_input_present.len() {
            raw(", ");
        }
        idx += 1;
    }
    raw_line("],");
    raw("          \"retained_module_evidence_present\": ");
    raw_bool(evidence.retained_module_evidence_present);
    raw_line(",");
    raw("          \"retained_service_slot_reservation_present\": ");
    raw_bool(evidence.retained_service_slot_reservation_present);
    raw_line(",");
    raw("          \"service_slot_reservation_event_id\": ");
    json_event_id_option(evidence.service_slot_reservation_event_id);
    raw_line(",");
    raw("          \"ram_only_service_slot_id\": ");
    if let Some(id) = evidence.ram_only_service_slot_id {
        json_str(id.as_str());
    } else {
        raw("null");
    }
    raw_line(",");
    raw_line("          \"source_evidence_retained\": true,");
    raw_line("          \"retention\": \"current_boot_ram_event_log\",");
    raw_line("          \"allocates_service_slot\": false,");
    raw_line("          \"creates_service_inventory_records\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"can_load_now\": false,");
    raw_line("          \"load_attempted\": false");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_service_slot_registry_write_commit_gate_source_evidence_item(
    event_id: event_log::EventId,
    evidence: event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence,
    comma: bool,
) {
    raw_line("        {");
    raw_line("          \"kind\": \"registry_write_commit_gate\",");
    raw("          \"event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("          \"schema\": ");
    json_str(evidence.schema);
    raw_line(",");
    raw_line("          \"status\": \"retained_current_boot_source_evidence\",");
    raw_line(
        "          \"reason\": \"module_service_slot_registry_write_commit_gate_source_evidence_recorded\",",
    );
    raw("          \"gate_schema\": ");
    json_str(evidence.gate_schema);
    raw_line(",");
    raw("          \"gate_id\": ");
    json_str(evidence.gate_id);
    raw_line(",");
    raw("          \"source_method\": ");
    json_str(evidence.source_method);
    raw_line(",");
    raw("          \"source_fact_locator\": ");
    json_str(evidence.source_fact_locator);
    raw_line(",");
    raw("          \"gate_status\": ");
    json_str(evidence.gate_status);
    raw_line(",");
    raw("          \"gate_reason\": ");
    json_str(evidence.gate_reason);
    raw_line(",");
    raw("          \"gate_present\": ");
    raw_bool(evidence.gate_present);
    raw_line(",");
    raw("          \"gate_scope\": ");
    json_str(evidence.gate_scope);
    raw_line(",");
    raw("          \"requested_capability\": ");
    json_str(evidence.requested_capability);
    raw_line(",");
    raw("          \"load_mode\": ");
    json_str(evidence.load_mode);
    raw_line(",");
    raw("          \"target\": ");
    json_str(evidence.target);
    raw_line(",");
    raw("          \"authority_decision_present\": ");
    raw_bool(evidence.authority_decision_present);
    raw_line(",");
    raw("          \"registry_write_authority_present\": ");
    raw_bool(evidence.registry_write_authority_present);
    raw_line(",");
    raw("          \"registry_binding_available\": ");
    raw_bool(evidence.registry_binding_available);
    raw_line(",");
    raw("          \"durable_audit_write_available\": ");
    raw_bool(evidence.durable_audit_write_available);
    raw_line(",");
    raw("          \"rollback_plan_install_available\": ");
    raw_bool(evidence.rollback_plan_install_available);
    raw_line(",");
    raw("          \"retained_service_slot_reservation_present\": ");
    raw_bool(evidence.retained_service_slot_reservation_present);
    raw_line(",");
    raw("          \"source_chain_complete\": ");
    raw_bool(evidence.source_chain_complete);
    raw_line(",");
    raw("          \"authority_decision_source_evidence_event_id\": ");
    json_event_id_option(evidence.authority_decision_source_evidence_event_id);
    raw_line(",");
    raw("          \"registry_write_authority_source_evidence_event_id\": ");
    json_event_id_option(evidence.registry_write_authority_source_evidence_event_id);
    raw_line(",");
    raw("          \"registry_binding_source_evidence_event_id\": ");
    json_event_id_option(evidence.registry_binding_source_evidence_event_id);
    raw_line(",");
    raw("          \"durable_audit_source_evidence_event_id\": ");
    json_event_id_option(evidence.durable_audit_source_evidence_event_id);
    raw_line(",");
    raw("          \"rollback_install_source_evidence_event_id\": ");
    json_event_id_option(evidence.rollback_install_source_evidence_event_id);
    raw_line(",");
    raw("          \"service_slot_reservation_event_id\": ");
    json_event_id_option(evidence.service_slot_reservation_event_id);
    raw_line(",");
    raw("          \"ram_only_service_slot_id\": ");
    if let Some(id) = evidence.ram_only_service_slot_id {
        json_str(id.as_str());
    } else {
        raw("null");
    }
    raw_line(",");
    raw_line("          \"source_evidence_retained\": true,");
    raw_line("          \"retention\": \"current_boot_ram_event_log\",");
    raw("          \"authorizes_registry_write\": ");
    raw_bool(evidence.authorizes_registry_write);
    raw_line(",");
    raw("          \"mutates_service_registry\": ");
    raw_bool(evidence.mutates_service_registry);
    raw_line(",");
    raw("          \"writes_durable_audit_state\": ");
    raw_bool(evidence.writes_durable_audit_state);
    raw_line(",");
    raw("          \"installs_rollback_state\": ");
    raw_bool(evidence.installs_rollback_state);
    raw_line(",");
    raw("          \"allocates_service_slot\": ");
    raw_bool(evidence.allocates_service_slot);
    raw_line(",");
    raw_line("          \"creates_service_inventory_records\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"can_load_now\": false,");
    raw("          \"loads_artifact\": ");
    raw_bool(evidence.loads_artifact);
    raw_line(",");
    raw("          \"load_attempted\": ");
    raw_bool(evidence.loads_artifact);
    crlf();
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
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
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0],
        candidate.allocator_runtime,
        evaluation.allocator_runtime_status,
        evaluation.allocator_runtime_reason,
        true,
    );
    emit_module_service_slot_allocator_fact(
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1],
        candidate.registry_binding,
        evaluation.registry_binding_status,
        evaluation.registry_binding_reason,
        true,
    );
    emit_module_service_slot_allocator_fact(
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2],
        candidate.health_state,
        evaluation.health_state_status,
        evaluation.health_state_reason,
        true,
    );
    emit_module_service_slot_allocator_fact(
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3],
        candidate.unload_cleanup,
        evaluation.unload_cleanup_status,
        evaluation.unload_cleanup_reason,
        false,
    );
    raw_line("      }");
}

fn emit_module_service_slot_allocator_fact(
    source: ModuleServiceSlotAllocatorFactSource,
    fact: ModuleServiceSlotAllocatorFact,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(source.name);
    raw_line(": {");
    raw("          \"schema\": ");
    json_str(source.schema);
    raw_line(",");
    raw("          \"id\": ");
    json_str(source.id);
    raw_line(",");
    raw("          \"source_method\": ");
    json_str(source.source_method);
    raw_line(",");
    raw("          \"source_fact_locator\": ");
    json_str(source.source_fact_locator);
    raw_line(",");
    raw("          \"source_evidence_event_id\": ");
    json_event_id_option(fact.source_evidence_event_id);
    raw_line(",");
    raw("          \"source_evidence_schema\": ");
    json_str(fact.source_evidence_schema);
    raw_line(",");
    raw("          \"source_evidence_state\": ");
    json_str(fact.source_evidence_state);
    raw_line(",");
    raw("          \"source_evidence_status\": ");
    json_str(fact.source_evidence_status);
    raw_line(",");
    raw("          \"source_evidence_reason\": ");
    json_str(fact.source_evidence_reason);
    raw_line(",");
    raw("          \"source_evidence_method\": ");
    json_str(fact.source_evidence_method);
    raw_line(",");
    raw("          \"source_evidence_fact_locator\": ");
    json_str(fact.source_evidence_fact_locator);
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
    raw("            \"source_method\": ");
    json_str(source.source_method);
    raw_line(",");
    raw("            \"source_fact_locator\": ");
    json_str(source.source_fact_locator);
    raw_line(",");
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

fn emit_module_service_slot_allocator_prerequisites(
    candidate: ModuleServiceSlotAllocatorCandidate,
    evaluation: ModuleServiceSlotAllocatorEvaluation,
) {
    raw_line("      \"allocator_prerequisite_gates\": {");
    emit_module_service_slot_allocator_prerequisite(
        MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0],
        candidate.durable_audit_write,
        evaluation.durable_audit_status,
        evaluation.durable_audit_reason,
        true,
    );
    emit_module_service_slot_allocator_prerequisite(
        MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1],
        candidate.rollback_plan_install,
        evaluation.rollback_status,
        evaluation.rollback_reason,
        true,
    );
    emit_module_service_slot_allocator_prerequisite(
        MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2],
        candidate.module_loader,
        evaluation.module_loader_status,
        evaluation.module_loader_reason,
        false,
    );
    raw_line("      }");
}

fn emit_module_service_slot_allocator_prerequisite(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
    prerequisite: ModuleServiceSlotAllocatorPrerequisite,
    status: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(source.name);
    raw_line(": {");
    raw("          \"schema\": ");
    json_str(source.schema);
    raw_line(",");
    raw("          \"id\": ");
    json_str(source.id);
    raw_line(",");
    raw("          \"source_method\": ");
    json_str(source.source_method);
    raw_line(",");
    raw("          \"source_fact_locator\": ");
    json_str(source.source_fact_locator);
    raw_line(",");
    raw("          \"source_evidence_event_id\": ");
    json_event_id_option(prerequisite.source_evidence_event_id);
    raw_line(",");
    raw("          \"source_evidence_schema\": ");
    json_str(prerequisite.source_evidence_schema);
    raw_line(",");
    raw("          \"source_evidence_state\": ");
    json_str(prerequisite.source_evidence_state);
    raw_line(",");
    raw("          \"source_evidence_status\": ");
    json_str(prerequisite.source_evidence_status);
    raw_line(",");
    raw("          \"source_evidence_reason\": ");
    json_str(prerequisite.source_evidence_reason);
    raw_line(",");
    raw("          \"source_evidence_method\": ");
    json_str(prerequisite.source_evidence_method);
    raw_line(",");
    raw("          \"source_evidence_fact_locator\": ");
    json_str(prerequisite.source_evidence_fact_locator);
    raw_line(",");
    raw("          \"status\": ");
    json_str(status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("          \"available\": ");
    raw_bool(prerequisite.available);
    raw_line(",");
    raw_line("          \"scope\": \"current_boot\",");
    raw_line("          \"classification\": \"local_only\",");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"allocates_service_slot\": false,");
    raw_line("          \"creates_service_inventory_records\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"authorizes_load\": false,");
    raw_line("          \"provenance\": {");
    raw("            \"source_method\": ");
    json_str(source.source_method);
    raw_line(",");
    raw("            \"source_fact_locator\": ");
    json_str(source.source_fact_locator);
    raw_line(",");
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

fn emit_module_service_slot_allocator_authority(
    candidate: ModuleServiceSlotAllocatorCandidate,
    evaluation: ModuleServiceSlotAllocatorEvaluation,
) {
    let authority = candidate.allocator_authority;
    raw_line("      \"allocator_authority_boundary\": {");
    raw("        \"schema\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SCHEMA);
    raw_line(",");
    raw("        \"id\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_ID);
    raw_line(",");
    raw_line("        \"scope\": \"current_boot\",");
    raw_line("        \"classification\": \"local_only\",");
    raw("        \"source_method\": ");
    json_str(authority.source_evidence_method);
    raw_line(",");
    raw("        \"source_fact_locator\": ");
    json_str(authority.source_evidence_fact_locator);
    raw_line(",");
    raw("        \"source_evidence_event_id\": ");
    json_event_id_option(authority.source_evidence_event_id);
    raw_line(",");
    raw("        \"source_evidence_schema\": ");
    json_str(authority.source_evidence_schema);
    raw_line(",");
    raw("        \"source_evidence_state\": ");
    json_str(authority.source_evidence_state);
    raw_line(",");
    raw("        \"source_evidence_status\": ");
    json_str(authority.source_evidence_status);
    raw_line(",");
    raw("        \"source_evidence_reason\": ");
    json_str(authority.source_evidence_reason);
    raw_line(",");
    raw("        \"status\": ");
    json_str(evaluation.authority_status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(evaluation.authority_reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(authority.present);
    raw_line(",");
    raw("        \"source_chain_complete\": ");
    raw_bool(authority.source_chain_complete);
    raw_line(",");
    raw_line("        \"future_authority_inputs\": [");
    emit_module_service_slot_allocator_authority_required_input(
        "raios.service_slot_allocation_intent.v0",
        evaluation.allocation_intent_status,
        evaluation.allocation_intent_reason,
        true,
    );
    emit_module_service_slot_allocator_authority_required_input(
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0].schema,
        evaluation.authority_input_statuses[0],
        evaluation.authority_input_reasons[0],
        true,
    );
    let mut idx = 1usize;
    while idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        emit_module_service_slot_allocator_authority_required_input(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[idx].schema,
            evaluation.authority_input_statuses[idx],
            evaluation.authority_input_reasons[idx],
            idx + 1 != MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT,
        );
        idx += 1;
    }
    raw_line("        ],");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"can_allocate\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
}

fn emit_module_service_slot_allocation_intent(
    candidate: ModuleServiceSlotAllocatorCandidate,
    evaluation: ModuleServiceSlotAllocatorEvaluation,
) {
    let intent = candidate.allocation_intent;
    raw_line("      \"allocation_intent_boundary\": {");
    raw("        \"schema\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SCHEMA);
    raw_line(",");
    raw("        \"id\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATION_INTENT_ID);
    raw_line(",");
    raw_line("        \"scope\": \"current_boot\",");
    raw_line("        \"classification\": \"local_only\",");
    raw("        \"source_method\": ");
    json_str(intent.source_evidence_method);
    raw_line(",");
    raw("        \"source_fact_locator\": ");
    json_str(intent.source_evidence_fact_locator);
    raw_line(",");
    raw("        \"source_evidence_event_id\": ");
    json_event_id_option(intent.source_evidence_event_id);
    raw_line(",");
    raw("        \"source_evidence_schema\": ");
    json_str(intent.source_evidence_schema);
    raw_line(",");
    raw("        \"source_evidence_state\": ");
    json_str(intent.source_evidence_state);
    raw_line(",");
    raw("        \"source_evidence_status\": ");
    json_str(intent.source_evidence_status);
    raw_line(",");
    raw("        \"source_evidence_reason\": ");
    json_str(intent.source_evidence_reason);
    raw_line(",");
    raw("        \"status\": ");
    json_str(evaluation.allocation_intent_status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(evaluation.allocation_intent_reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(intent.present);
    raw_line(",");
    raw("        \"source_chain_complete\": ");
    raw_bool(intent.source_chain_complete);
    raw_line(",");
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"target\": \"live_service_graph\",");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"can_allocate\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
}

fn emit_module_service_slot_authority_inputs(
    candidate: ModuleServiceSlotAllocatorCandidate,
    evaluation: ModuleServiceSlotAllocatorEvaluation,
) {
    raw_line("      \"authority_input_boundaries\": {");
    let mut idx = 0usize;
    while idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        let input = candidate.authority_inputs[idx];
        raw("        ");
        json_str(input.spec.name);
        raw_line(": {");
        raw("          \"schema\": ");
        json_str(input.spec.schema);
        raw_line(",");
        raw("          \"id\": ");
        json_str(input.spec.id);
        raw_line(",");
        raw_line("          \"scope\": \"current_boot\",");
        raw_line("          \"classification\": \"local_only\",");
        raw("          \"source_method\": ");
        json_str(input.source_evidence_method);
        raw_line(",");
        raw("          \"source_fact_locator\": ");
        json_str(input.source_evidence_fact_locator);
        raw_line(",");
        raw("          \"source_evidence_event_id\": ");
        json_event_id_option(input.source_evidence_event_id);
        raw_line(",");
        raw("          \"source_evidence_schema\": ");
        json_str(input.source_evidence_schema);
        raw_line(",");
        raw("          \"source_evidence_state\": ");
        json_str(input.source_evidence_state);
        raw_line(",");
        raw("          \"source_evidence_status\": ");
        json_str(input.source_evidence_status);
        raw_line(",");
        raw("          \"source_evidence_reason\": ");
        json_str(input.source_evidence_reason);
        raw_line(",");
        raw("          \"dependency_source_evidence_event_id\": ");
        json_event_id_option(input.dependency_source_evidence_event_id);
        raw_line(",");
        raw("          \"status\": ");
        json_str(evaluation.authority_input_statuses[idx]);
        raw_line(",");
        raw("          \"reason\": ");
        json_str(evaluation.authority_input_reasons[idx]);
        raw_line(",");
        raw("          \"present\": ");
        raw_bool(input.present);
        raw_line(",");
        raw("          \"source_chain_complete\": ");
        raw_bool(input.source_chain_complete);
        raw_line(",");
        raw_line("          \"requested_capability\": \"cap.module.load_ephemeral\",");
        raw_line("          \"load_mode\": \"ram_only\",");
        raw_line("          \"target\": \"live_service_graph\",");
        raw_line("          \"accepts_loader_descriptor\": false,");
        raw_line("          \"accepts_artifact_bytes\": false,");
        raw_line("          \"allocates_service_slot\": false,");
        raw_line("          \"creates_service_inventory_records\": false,");
        raw_line("          \"service_inventory_change\": \"none\",");
        raw_line("          \"can_allocate\": false,");
        raw_line("          \"can_load_now\": false,");
        raw_line("          \"load_attempted\": false");
        raw("        }");
        if idx + 1 != MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
            raw(",");
        }
        crlf();
        idx += 1;
    }
    raw_line("      }");
}

fn emit_module_service_slot_allocator_authority_decision(
    candidate: ModuleServiceSlotAllocatorCandidate,
    evaluation: ModuleServiceSlotAllocatorEvaluation,
) {
    let decision = candidate.authority_decision;
    raw_line("      \"authority_decision\": {");
    raw("        \"schema\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SCHEMA);
    raw_line(",");
    raw("        \"id\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_ID);
    raw_line(",");
    raw_line("        \"scope\": \"current_boot\",");
    raw_line("        \"classification\": \"local_only\",");
    raw("        \"source_method\": ");
    json_str(decision.source_evidence_method);
    raw_line(",");
    raw("        \"source_fact_locator\": ");
    json_str(decision.source_evidence_fact_locator);
    raw_line(",");
    raw("        \"source_evidence_event_id\": ");
    json_event_id_option(decision.source_evidence_event_id);
    raw_line(",");
    raw("        \"source_evidence_schema\": ");
    json_str(decision.source_evidence_schema);
    raw_line(",");
    raw("        \"source_evidence_state\": ");
    json_str(decision.source_evidence_state);
    raw_line(",");
    raw("        \"source_evidence_status\": ");
    json_str(decision.source_evidence_status);
    raw_line(",");
    raw("        \"source_evidence_reason\": ");
    json_str(decision.source_evidence_reason);
    raw_line(",");
    raw("        \"status\": ");
    json_str(evaluation.authority_decision_status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(evaluation.authority_decision_reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(decision.present);
    raw_line(",");
    raw("        \"input_chain_complete\": ");
    raw_bool(decision.input_chain_complete);
    raw_line(",");
    raw("        \"source_chain_complete\": ");
    raw_bool(decision.source_chain_complete);
    raw_line(",");
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"target\": \"live_service_graph\",");
    raw_line("        \"authorizes_allocation\": false,");
    raw_line("        \"authorizes_load\": false,");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"can_allocate\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
}

fn emit_module_service_slot_registry_write_commit_gate(
    candidate: ModuleServiceSlotAllocatorCandidate,
    evaluation: ModuleServiceSlotAllocatorEvaluation,
) {
    let gate = candidate.registry_write_commit_gate;
    raw_line("      \"registry_write_commit_gate\": {");
    raw("        \"schema\": ");
    json_str(MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SCHEMA);
    raw_line(",");
    raw("        \"id\": ");
    json_str(MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_ID);
    raw_line(",");
    raw_line("        \"scope\": \"current_boot\",");
    raw_line("        \"classification\": \"local_only\",");
    raw("        \"source_method\": ");
    json_str(gate.source_evidence_method);
    raw_line(",");
    raw("        \"source_fact_locator\": ");
    json_str(gate.source_evidence_fact_locator);
    raw_line(",");
    raw("        \"source_evidence_event_id\": ");
    json_event_id_option(gate.source_evidence_event_id);
    raw_line(",");
    raw("        \"source_evidence_schema\": ");
    json_str(gate.source_evidence_schema);
    raw_line(",");
    raw("        \"source_evidence_state\": ");
    json_str(gate.source_evidence_state);
    raw_line(",");
    raw("        \"source_evidence_status\": ");
    json_str(gate.source_evidence_status);
    raw_line(",");
    raw("        \"source_evidence_reason\": ");
    json_str(gate.source_evidence_reason);
    raw_line(",");
    raw("        \"status\": ");
    json_str(evaluation.registry_write_commit_gate_status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(evaluation.registry_write_commit_gate_reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(gate.present);
    raw_line(",");
    raw("        \"source_chain_complete\": ");
    raw_bool(gate.source_chain_complete);
    raw_line(",");
    raw("        \"authority_decision_present\": ");
    raw_bool(gate.authority_decision_present);
    raw_line(",");
    raw("        \"registry_write_authority_present\": ");
    raw_bool(gate.registry_write_authority_present);
    raw_line(",");
    raw("        \"registry_binding_available\": ");
    raw_bool(gate.registry_binding_available);
    raw_line(",");
    raw("        \"durable_audit_write_available\": ");
    raw_bool(gate.durable_audit_write_available);
    raw_line(",");
    raw("        \"rollback_plan_install_available\": ");
    raw_bool(gate.rollback_plan_install_available);
    raw_line(",");
    raw("        \"retained_service_slot_reservation_present\": ");
    raw_bool(gate.retained_service_slot_reservation_present);
    raw_line(",");
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"target\": \"live_service_graph\",");
    raw_line("        \"authorizes_registry_write\": false,");
    raw_line("        \"authorizes_allocation\": false,");
    raw_line("        \"authorizes_load\": false,");
    raw_line("        \"mutates_service_registry\": false,");
    raw_line("        \"writes_durable_audit_state\": false,");
    raw_line("        \"installs_rollback_state\": false,");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"can_allocate\": false,");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
}

fn emit_module_service_slot_allocator_authority_required_input(
    schema: &'static str,
    state: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("          {\"schema\": ");
    json_str(schema);
    raw(", \"state\": ");
    json_str(state);
    raw(", \"reason\": ");
    json_str(reason);
    raw(", \"required_before_allocation\": true, \"classification\": \"local_only\"}");
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
    raw(", \"actual_allocator_runtime_source_evidence_present\": ");
    raw_bool(case.actual_allocator_runtime_source_evidence_present);
    raw(", \"actual_allocator_runtime_source_evidence_state\": ");
    json_str(case.actual_allocator_runtime_source_evidence_state);
    raw(", \"actual_allocator_runtime_source_evidence_status\": ");
    json_str(case.actual_allocator_runtime_source_evidence_status);
    raw(", \"actual_allocator_runtime_source_evidence_reason\": ");
    json_str(case.actual_allocator_runtime_source_evidence_reason);
    raw(", \"actual_registry_binding_source_evidence_present\": ");
    raw_bool(case.actual_registry_binding_source_evidence_present);
    raw(", \"actual_registry_binding_source_evidence_state\": ");
    json_str(case.actual_registry_binding_source_evidence_state);
    raw(", \"actual_registry_binding_source_evidence_status\": ");
    json_str(case.actual_registry_binding_source_evidence_status);
    raw(", \"actual_registry_binding_source_evidence_reason\": ");
    json_str(case.actual_registry_binding_source_evidence_reason);
    raw(", \"actual_health_state_source_evidence_present\": ");
    raw_bool(case.actual_health_state_source_evidence_present);
    raw(", \"actual_health_state_source_evidence_state\": ");
    json_str(case.actual_health_state_source_evidence_state);
    raw(", \"actual_health_state_source_evidence_status\": ");
    json_str(case.actual_health_state_source_evidence_status);
    raw(", \"actual_health_state_source_evidence_reason\": ");
    json_str(case.actual_health_state_source_evidence_reason);
    raw(", \"actual_unload_cleanup_source_evidence_present\": ");
    raw_bool(case.actual_unload_cleanup_source_evidence_present);
    raw(", \"actual_unload_cleanup_source_evidence_state\": ");
    json_str(case.actual_unload_cleanup_source_evidence_state);
    raw(", \"actual_unload_cleanup_source_evidence_status\": ");
    json_str(case.actual_unload_cleanup_source_evidence_status);
    raw(", \"actual_unload_cleanup_source_evidence_reason\": ");
    json_str(case.actual_unload_cleanup_source_evidence_reason);
    raw(", \"actual_durable_audit_source_evidence_present\": ");
    raw_bool(case.actual_durable_audit_source_evidence_present);
    raw(", \"actual_durable_audit_source_evidence_state\": ");
    json_str(case.actual_durable_audit_source_evidence_state);
    raw(", \"actual_durable_audit_source_evidence_status\": ");
    json_str(case.actual_durable_audit_source_evidence_status);
    raw(", \"actual_durable_audit_source_evidence_reason\": ");
    json_str(case.actual_durable_audit_source_evidence_reason);
    raw(", \"actual_rollback_install_source_evidence_present\": ");
    raw_bool(case.actual_rollback_install_source_evidence_present);
    raw(", \"actual_rollback_install_source_evidence_state\": ");
    json_str(case.actual_rollback_install_source_evidence_state);
    raw(", \"actual_rollback_install_source_evidence_status\": ");
    json_str(case.actual_rollback_install_source_evidence_status);
    raw(", \"actual_rollback_install_source_evidence_reason\": ");
    json_str(case.actual_rollback_install_source_evidence_reason);
    raw(", \"actual_module_loader_source_evidence_present\": ");
    raw_bool(case.actual_module_loader_source_evidence_present);
    raw(", \"actual_module_loader_source_evidence_state\": ");
    json_str(case.actual_module_loader_source_evidence_state);
    raw(", \"actual_module_loader_source_evidence_status\": ");
    json_str(case.actual_module_loader_source_evidence_status);
    raw(", \"actual_module_loader_source_evidence_reason\": ");
    json_str(case.actual_module_loader_source_evidence_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"can_allocate\": false, \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn module_service_slot_allocator_fact_source_evidence(
    source: ModuleServiceSlotAllocatorFactSource,
    retained_service_slot_reservation_event_id: Option<event_log::EventId>,
    allocator_runtime_source_evidence_event_id: Option<event_log::EventId>,
) -> event_log::ModuleServiceSlotAllocatorFactSourceEvidence {
    event_log::ModuleServiceSlotAllocatorFactSourceEvidence {
        schema: source.source_evidence_schema,
        fact_schema: source.schema,
        fact_id: source.id,
        source_method: source.source_method,
        source_fact_locator: source.source_fact_locator,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_service_slot_allocator_fact_source_evidence_recorded",
        fact_status: "missing",
        fact_reason: source.missing_reason,
        fact_present: false,
        fact_scope: "current_boot",
        fact_schema_ok: true,
        fact_provenance_ok: false,
        fact_classification: "local_only",
        retained_service_slot_reservation_present: retained_service_slot_reservation_event_id
            .is_some(),
        allocator_runtime_source_evidence_present: allocator_runtime_source_evidence_event_id
            .is_some(),
        binds_retained_service_slot_reservation: false,
        binds_allocator_runtime: false,
        retained_service_slot_reservation_event_id,
        allocator_runtime_source_evidence_event_id,
    }
}

fn module_service_slot_allocator_runtime_source_evidence(
    source: ModuleServiceSlotAllocatorFactSource,
    retained_service_slot_reservation_event_id: Option<event_log::EventId>,
    allocator_runtime_source_evidence_event_id: Option<event_log::EventId>,
) -> event_log::ModuleServiceSlotAllocatorFactSourceEvidence {
    let mut evidence = module_service_slot_allocator_fact_source_evidence(
        source,
        retained_service_slot_reservation_event_id,
        allocator_runtime_source_evidence_event_id,
    );
    if retained_service_slot_reservation_event_id.is_some() {
        evidence.fact_status = "available";
        evidence.fact_reason = "service_slot_allocator_runtime_available";
        evidence.fact_present = true;
        evidence.fact_provenance_ok = true;
        evidence.binds_retained_service_slot_reservation = true;
        evidence.binds_allocator_runtime = true;
    }
    evidence
}

fn module_service_slot_allocator_bound_fact_source_evidence(
    source: ModuleServiceSlotAllocatorFactSource,
    retained_service_slot_reservation_event_id: Option<event_log::EventId>,
    allocator_runtime_source_evidence_event_id: Option<event_log::EventId>,
    allocator_runtime_source_evidence: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    available_reason: &'static str,
) -> event_log::ModuleServiceSlotAllocatorFactSourceEvidence {
    let mut evidence = module_service_slot_allocator_fact_source_evidence(
        source,
        retained_service_slot_reservation_event_id,
        allocator_runtime_source_evidence_event_id,
    );
    if retained_service_slot_reservation_event_id.is_some()
        && allocator_runtime_source_evidence.fact_present
        && allocator_runtime_source_evidence.fact_schema_ok
        && allocator_runtime_source_evidence.fact_provenance_ok
        && allocator_runtime_source_evidence.binds_retained_service_slot_reservation
    {
        evidence.fact_status = "available";
        evidence.fact_reason = available_reason;
        evidence.fact_present = true;
        evidence.fact_provenance_ok = true;
        evidence.binds_retained_service_slot_reservation = true;
        evidence.binds_allocator_runtime = true;
    }
    evidence
}

fn module_service_slot_allocator_prerequisite_source_evidence(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
    retained_service_slot_reservation_present: bool,
    allocator_runtime_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    registry_binding_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    health_state_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    unload_cleanup_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
) -> event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence {
    event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence {
        schema: source.source_evidence_schema,
        prerequisite_schema: source.schema,
        prerequisite_id: source.id,
        source_method: source.source_method,
        source_fact_locator: source.source_fact_locator,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_service_slot_allocator_prerequisite_source_evidence_recorded",
        prerequisite_status: source.missing_status,
        prerequisite_reason: source.missing_reason,
        prerequisite_available: false,
        retained_service_slot_reservation_present,
        allocator_runtime_available: allocator_runtime_source_evidence.1.fact_present,
        registry_binding_available: registry_binding_source_evidence.1.fact_present,
        health_state_available: health_state_source_evidence.1.fact_present,
        unload_cleanup_available: unload_cleanup_source_evidence.1.fact_present,
        allocator_runtime_source_evidence_event_id: Some(allocator_runtime_source_evidence.0),
        registry_binding_source_evidence_event_id: Some(registry_binding_source_evidence.0),
        health_state_source_evidence_event_id: Some(health_state_source_evidence.0),
        unload_cleanup_source_evidence_event_id: Some(unload_cleanup_source_evidence.0),
    }
}

fn module_service_slot_allocator_bound_prerequisite_source_evidence(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
    retained_service_slot_reservation_present: bool,
    allocator_runtime_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    registry_binding_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    health_state_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    unload_cleanup_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    available_reason: &'static str,
) -> event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence {
    let mut evidence = module_service_slot_allocator_prerequisite_source_evidence(
        source,
        retained_service_slot_reservation_present,
        allocator_runtime_source_evidence,
        registry_binding_source_evidence,
        health_state_source_evidence,
        unload_cleanup_source_evidence,
    );
    if retained_service_slot_reservation_present
        && module_service_slot_allocator_fact_source_evidence_available(
            allocator_runtime_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            registry_binding_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            health_state_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            unload_cleanup_source_evidence.1,
        )
    {
        evidence.prerequisite_status = "available";
        evidence.prerequisite_reason = available_reason;
        evidence.prerequisite_available = true;
    }
    evidence
}

fn module_service_slot_allocator_fact_source_evidence_available(
    evidence: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
) -> bool {
    evidence.fact_present
        && evidence.fact_schema_ok
        && evidence.fact_provenance_ok
        && evidence.binds_retained_service_slot_reservation
        && evidence.binds_allocator_runtime
}

fn module_service_slot_allocator_module_loader_prerequisite_source_evidence(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
    retained_service_slot_reservation_present: bool,
    allocator_runtime_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    registry_binding_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    health_state_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    unload_cleanup_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    durable_audit_source_evidence: event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    rollback_install_source_evidence: event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
) -> event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence {
    let mut evidence = module_service_slot_allocator_prerequisite_source_evidence(
        source,
        retained_service_slot_reservation_present,
        allocator_runtime_source_evidence,
        registry_binding_source_evidence,
        health_state_source_evidence,
        unload_cleanup_source_evidence,
    );
    if retained_service_slot_reservation_present
        && module_service_slot_allocator_fact_source_evidence_available(
            allocator_runtime_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            registry_binding_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            health_state_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            unload_cleanup_source_evidence.1,
        )
        && module_service_slot_allocator_prerequisite_source_evidence_available(
            durable_audit_source_evidence,
        )
        && module_service_slot_allocator_prerequisite_source_evidence_available(
            rollback_install_source_evidence,
        )
    {
        evidence.prerequisite_status = "available";
        evidence.prerequisite_reason = "module_loader_boundary_available_non_authorizing";
        evidence.prerequisite_available = true;
    }
    evidence
}

fn module_service_slot_allocator_prerequisite_source_evidence_available(
    evidence: event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
) -> bool {
    evidence.prerequisite_available
}

fn module_service_slot_allocator_authority_source_evidence(
    retained_service_slot_reservation_present: bool,
    allocator_runtime_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    registry_binding_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    health_state_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    unload_cleanup_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    durable_audit_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    ),
    rollback_install_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    ),
    module_loader_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    ),
) -> event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence {
    let source_chain_complete = retained_service_slot_reservation_present
        && module_service_slot_allocator_fact_source_evidence_available(
            allocator_runtime_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            registry_binding_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            health_state_source_evidence.1,
        )
        && module_service_slot_allocator_fact_source_evidence_available(
            unload_cleanup_source_evidence.1,
        )
        && module_service_slot_allocator_prerequisite_source_evidence_available(
            durable_audit_source_evidence.1,
        )
        && module_service_slot_allocator_prerequisite_source_evidence_available(
            rollback_install_source_evidence.1,
        )
        && module_service_slot_allocator_prerequisite_source_evidence_available(
            module_loader_source_evidence.1,
        );
    event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence {
        schema: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_SCHEMA,
        authority_schema: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SCHEMA,
        authority_id: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_ID,
        source_method: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_METHOD,
        source_fact_locator: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_FACT_LOCATOR,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_service_slot_allocator_authority_source_evidence_recorded",
        authority_status: if source_chain_complete {
            "defined_non_authorizing"
        } else {
            "missing"
        },
        authority_reason: if source_chain_complete {
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON
        } else {
            "service_slot_allocator_authority_source_chain_incomplete"
        },
        authority_present: source_chain_complete,
        authority_scope: "current_boot",
        authority_schema_ok: true,
        authority_provenance_ok: source_chain_complete,
        authority_classification: "local_only",
        retained_service_slot_reservation_present,
        allocator_runtime_available: allocator_runtime_source_evidence.1.fact_present,
        registry_binding_available: registry_binding_source_evidence.1.fact_present,
        health_state_available: health_state_source_evidence.1.fact_present,
        unload_cleanup_available: unload_cleanup_source_evidence.1.fact_present,
        durable_audit_write_available: durable_audit_source_evidence.1.prerequisite_available,
        rollback_plan_install_available: rollback_install_source_evidence.1.prerequisite_available,
        module_loader_available: module_loader_source_evidence.1.prerequisite_available,
        source_chain_complete,
        allocator_runtime_source_evidence_event_id: Some(allocator_runtime_source_evidence.0),
        registry_binding_source_evidence_event_id: Some(registry_binding_source_evidence.0),
        health_state_source_evidence_event_id: Some(health_state_source_evidence.0),
        unload_cleanup_source_evidence_event_id: Some(unload_cleanup_source_evidence.0),
        durable_audit_source_evidence_event_id: Some(durable_audit_source_evidence.0),
        rollback_install_source_evidence_event_id: Some(rollback_install_source_evidence.0),
        module_loader_source_evidence_event_id: Some(module_loader_source_evidence.0),
    }
}

fn module_service_slot_allocation_intent_source_evidence(
    binding: event_log::ModuleLoadGateBinding,
    retained_service_slot_reservation_present: bool,
    allocator_authority_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    ),
) -> event_log::ModuleServiceSlotAllocationIntentSourceEvidence {
    let retained_module_evidence_present =
        module_service_slot_allocation_intent_retained_module_evidence_present(binding);
    let source_chain_complete = retained_module_evidence_present
        && retained_service_slot_reservation_present
        && allocator_authority_source_evidence.1.authority_present
        && allocator_authority_source_evidence.1.source_chain_complete;
    event_log::ModuleServiceSlotAllocationIntentSourceEvidence {
        schema: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_SCHEMA,
        intent_schema: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SCHEMA,
        intent_id: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_ID,
        source_method: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_METHOD,
        source_fact_locator: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_FACT_LOCATOR,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_service_slot_allocation_intent_source_evidence_recorded",
        intent_status: if source_chain_complete {
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_STATUS
        } else {
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS
        },
        intent_reason: if source_chain_complete {
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_REASON
        } else {
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_CHAIN_INCOMPLETE_REASON
        },
        intent_present: source_chain_complete,
        intent_scope: "current_boot",
        intent_schema_ok: true,
        intent_provenance_ok: source_chain_complete,
        intent_classification: "local_only",
        requested_capability: "cap.module.load_ephemeral",
        load_mode: "ram_only",
        target: "live_service_graph",
        retained_module_evidence_present,
        retained_service_slot_reservation_present,
        allocator_authority_present: allocator_authority_source_evidence.1.authority_present,
        source_chain_complete,
        manifest_reference_event_id: binding.manifest_reference_event_id,
        artifact_reference_event_id: binding.artifact_reference_event_id,
        vm_report_reference_event_id: binding.vm_report_reference_event_id,
        local_attestation_reference_event_id: binding.attestation_reference_event_id,
        local_approval_reference_event_id: binding.approval_reference_event_id,
        computed_grant_reference_event_id: binding.retained_reference_event_id,
        audit_rollback_reference_event_id: binding.audit_rollback_reference_event_id,
        service_slot_reservation_event_id: binding.service_slot_reservation_event_id,
        allocator_authority_source_evidence_event_id: Some(allocator_authority_source_evidence.0),
        ram_only_service_slot_id: binding
            .service_slot_reservation
            .map(|reservation| reservation.ram_only_service_slot_id),
    }
}

fn module_service_slot_allocation_intent_retained_module_evidence_present(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.manifest_reference_status,
        "retained_hash_reference_only",
    ) && method_eq(
        binding.artifact_reference_status,
        "retained_hash_reference_only",
    ) && method_eq(
        binding.vm_report_reference_status,
        "retained_hash_reference_only",
    ) && method_eq(
        binding.attestation_reference_status,
        "retained_hash_reference_only",
    ) && method_eq(
        binding.approval_reference_status,
        "retained_hash_reference_only",
    ) && binding.retained_reference_event_id.is_some()
        && method_eq(
            binding.audit_rollback_reference_status,
            "retained_hash_reference_only",
        )
}

fn module_service_slot_authority_input_source_evidence(
    source: ModuleServiceSlotAuthorityInputSpec,
    binding: event_log::ModuleLoadGateBinding,
    allocation_intent_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
    ),
    dependency_schema: &'static str,
    dependency_source_evidence_event_id: Option<event_log::EventId>,
    dependency_present: bool,
) -> event_log::ModuleServiceSlotAuthorityInputSourceEvidence {
    let source_chain_complete = allocation_intent_source_evidence.1.source_chain_complete
        && allocation_intent_source_evidence.1.intent_present
        && dependency_present
        && dependency_source_evidence_event_id.is_some();
    event_log::ModuleServiceSlotAuthorityInputSourceEvidence {
        schema: source.source_evidence_schema,
        input_schema: source.schema,
        input_id: source.id,
        input_name: source.name,
        source_method: source.source_method,
        source_fact_locator: source.source_fact_locator,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_service_slot_authority_input_source_evidence_recorded",
        input_status: if source_chain_complete {
            source.available_status
        } else {
            source.missing_status
        },
        input_reason: if source_chain_complete {
            source.available_reason
        } else {
            source.source_chain_incomplete_reason
        },
        input_present: source_chain_complete,
        input_scope: "current_boot",
        input_schema_ok: true,
        input_provenance_ok: source_chain_complete,
        input_classification: "local_only",
        dependency_schema,
        dependency_source_evidence_event_id,
        dependency_present,
        requested_capability: "cap.module.load_ephemeral",
        load_mode: "ram_only",
        target: "live_service_graph",
        retained_module_evidence_present: allocation_intent_source_evidence
            .1
            .retained_module_evidence_present,
        retained_service_slot_reservation_present: allocation_intent_source_evidence
            .1
            .retained_service_slot_reservation_present,
        allocator_authority_present: allocation_intent_source_evidence
            .1
            .allocator_authority_present,
        allocation_intent_source_evidence_event_id: Some(allocation_intent_source_evidence.0),
        source_chain_complete,
        service_slot_reservation_event_id: binding.service_slot_reservation_event_id,
        allocator_authority_source_evidence_event_id: allocation_intent_source_evidence
            .1
            .allocator_authority_source_evidence_event_id,
        ram_only_service_slot_id: binding
            .service_slot_reservation
            .map(|reservation| reservation.ram_only_service_slot_id),
    }
}

fn module_service_slot_allocator_authority_decision_source_evidence(
    binding: event_log::ModuleLoadGateBinding,
    allocator_authority_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    ),
    allocation_intent_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
    ),
    authority_input_source_evidence: [(
        event_log::EventId,
        event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
    ); MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
) -> event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence {
    let mut authority_inputs_complete = true;
    let mut authority_input_event_ids = [None; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT];
    let mut authority_input_present = [false; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT];
    let mut idx = 0usize;
    while idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        authority_input_event_ids[idx] = Some(authority_input_source_evidence[idx].0);
        authority_input_present[idx] = authority_input_source_evidence[idx].1.input_present
            && authority_input_source_evidence[idx].1.source_chain_complete;
        authority_inputs_complete = authority_inputs_complete && authority_input_present[idx];
        idx += 1;
    }
    let source_chain_complete = allocator_authority_source_evidence.1.authority_present
        && allocator_authority_source_evidence.1.source_chain_complete
        && allocation_intent_source_evidence.1.intent_present
        && allocation_intent_source_evidence.1.source_chain_complete
        && authority_inputs_complete;
    event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence {
        schema: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_SCHEMA,
        decision_schema: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SCHEMA,
        decision_id: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_ID,
        source_method: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_METHOD,
        source_fact_locator: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_FACT_LOCATOR,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason:
            "module_service_slot_allocator_authority_decision_source_evidence_recorded",
        decision_status: if source_chain_complete {
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_STATUS
        } else {
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS
        },
        decision_reason: if source_chain_complete {
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_REASON
        } else {
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_CHAIN_INCOMPLETE_REASON
        },
        decision_present: source_chain_complete,
        decision_scope: "current_boot",
        decision_schema_ok: true,
        decision_provenance_ok: source_chain_complete,
        decision_classification: "local_only",
        requested_capability: "cap.module.load_ephemeral",
        load_mode: "ram_only",
        target: "live_service_graph",
        allocator_authority_present: allocator_authority_source_evidence.1.authority_present,
        allocation_intent_present: allocation_intent_source_evidence.1.intent_present,
        authority_inputs_complete,
        source_chain_complete,
        allocator_authority_source_evidence_event_id: Some(allocator_authority_source_evidence.0),
        allocation_intent_source_evidence_event_id: Some(allocation_intent_source_evidence.0),
        authority_input_source_evidence_event_ids: authority_input_event_ids,
        authority_input_present,
        retained_module_evidence_present: allocation_intent_source_evidence
            .1
            .retained_module_evidence_present,
        retained_service_slot_reservation_present: allocation_intent_source_evidence
            .1
            .retained_service_slot_reservation_present,
        service_slot_reservation_event_id: binding.service_slot_reservation_event_id,
        ram_only_service_slot_id: binding
            .service_slot_reservation
            .map(|reservation| reservation.ram_only_service_slot_id),
    }
}

fn module_service_slot_registry_write_commit_gate_source_evidence(
    binding: event_log::ModuleLoadGateBinding,
    authority_decision_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
    ),
    registry_write_authority_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
    ),
    registry_binding_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    ),
    durable_audit_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    ),
    rollback_install_source_evidence: (
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    ),
) -> event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence {
    let retained_service_slot_reservation_present = binding.service_slot_reservation.is_some()
        && binding.service_slot_reservation_event_id.is_some();
    let authority_decision_present = authority_decision_source_evidence.1.decision_present
        && authority_decision_source_evidence.1.source_chain_complete;
    let registry_write_authority_present = registry_write_authority_source_evidence.1.input_present
        && registry_write_authority_source_evidence
            .1
            .source_chain_complete;
    let registry_binding_available = registry_binding_source_evidence.1.fact_present
        && registry_binding_source_evidence.1.fact_schema_ok
        && registry_binding_source_evidence.1.fact_provenance_ok
        && registry_binding_source_evidence
            .1
            .binds_retained_service_slot_reservation
        && registry_binding_source_evidence.1.binds_allocator_runtime;
    let durable_audit_write_available = durable_audit_source_evidence.1.prerequisite_available;
    let rollback_plan_install_available = rollback_install_source_evidence.1.prerequisite_available;
    let source_chain_complete = authority_decision_present
        && registry_write_authority_present
        && registry_binding_available
        && durable_audit_write_available
        && rollback_plan_install_available
        && retained_service_slot_reservation_present;
    event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence {
        schema: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_SCHEMA,
        gate_schema: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SCHEMA,
        gate_id: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_ID,
        source_method: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_METHOD,
        source_fact_locator: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_FACT_LOCATOR,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_service_slot_registry_write_commit_gate_source_evidence_recorded",
        gate_status: if source_chain_complete {
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_STATUS
        } else {
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS
        },
        gate_reason: if source_chain_complete {
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_REASON
        } else {
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_CHAIN_INCOMPLETE_REASON
        },
        gate_present: source_chain_complete,
        gate_scope: "current_boot",
        gate_schema_ok: true,
        gate_provenance_ok: source_chain_complete,
        gate_classification: "local_only",
        requested_capability: "cap.module.load_ephemeral",
        load_mode: "ram_only",
        target: "live_service_graph",
        authority_decision_present,
        registry_write_authority_present,
        registry_binding_available,
        durable_audit_write_available,
        rollback_plan_install_available,
        retained_service_slot_reservation_present,
        source_chain_complete,
        authority_decision_source_evidence_event_id: Some(authority_decision_source_evidence.0),
        registry_write_authority_source_evidence_event_id: Some(
            registry_write_authority_source_evidence.0,
        ),
        registry_binding_source_evidence_event_id: Some(registry_binding_source_evidence.0),
        durable_audit_source_evidence_event_id: Some(durable_audit_source_evidence.0),
        rollback_install_source_evidence_event_id: Some(rollback_install_source_evidence.0),
        service_slot_reservation_event_id: binding.service_slot_reservation_event_id,
        ram_only_service_slot_id: binding
            .service_slot_reservation
            .map(|reservation| reservation.ram_only_service_slot_id),
        authorizes_registry_write: false,
        mutates_service_registry: false,
        writes_durable_audit_state: false,
        installs_rollback_state: false,
        allocates_service_slot: false,
        loads_artifact: false,
    }
}

fn module_service_slot_allocator_snapshot(
    retained_reservation_present: bool,
    allocator_runtime_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    )>,
    registry_binding_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    )>,
    health_state_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    )>,
    unload_cleanup_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    )>,
    durable_audit_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    )>,
    rollback_install_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    )>,
    module_loader_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    )>,
    allocator_authority_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    )>,
    allocation_intent_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
    )>,
    authority_input_source_evidence: Option<
        [(
            event_log::EventId,
            event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
        ); MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
    >,
    authority_decision_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
    )>,
    registry_write_commit_gate_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence,
    )>,
) -> ModuleServiceSlotAllocatorCandidate {
    ModuleServiceSlotAllocatorCandidate {
        retained_reservation_present,
        allocator_runtime: module_service_slot_allocator_fact_from_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0],
            allocator_runtime_source_evidence,
        ),
        registry_binding: module_service_slot_allocator_fact_from_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1],
            registry_binding_source_evidence,
        ),
        health_state: module_service_slot_allocator_fact_from_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2],
            health_state_source_evidence,
        ),
        unload_cleanup: module_service_slot_allocator_fact_from_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3],
            unload_cleanup_source_evidence,
        ),
        durable_audit_write: module_service_slot_allocator_prerequisite_from_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0],
            durable_audit_source_evidence,
        ),
        rollback_plan_install: module_service_slot_allocator_prerequisite_from_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1],
            rollback_install_source_evidence,
        ),
        module_loader: module_service_slot_allocator_prerequisite_from_source_evidence(
            MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2],
            module_loader_source_evidence,
        ),
        allocator_authority: module_service_slot_allocator_authority_from_source_evidence(
            allocator_authority_source_evidence,
        ),
        allocation_intent: module_service_slot_allocation_intent_from_source_evidence(
            allocation_intent_source_evidence,
        ),
        authority_inputs: module_service_slot_authority_inputs_from_source_evidence(
            authority_input_source_evidence,
        ),
        authority_decision: module_service_slot_allocator_authority_decision_from_source_evidence(
            authority_decision_source_evidence,
        ),
        registry_write_commit_gate:
            module_service_slot_registry_write_commit_gate_from_source_evidence(
                registry_write_commit_gate_source_evidence,
            ),
    }
}

fn module_service_slot_allocator_empty_snapshot(
    retained_reservation_present: bool,
) -> ModuleServiceSlotAllocatorCandidate {
    module_service_slot_allocator_snapshot(
        retained_reservation_present,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
}

fn module_service_slot_allocator_authority_from_source_evidence(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    )>,
) -> ModuleServiceSlotAllocatorAuthority {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleServiceSlotAllocatorAuthority {
            present: evidence.authority_present,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.authority_present {
                "observed_current_boot_defined"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.authority_status,
            source_evidence_reason: evidence.authority_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
            source_chain_complete: evidence.source_chain_complete,
        };
    }
    module_service_slot_allocator_missing_authority()
}

fn module_service_slot_allocator_missing_authority() -> ModuleServiceSlotAllocatorAuthority {
    ModuleServiceSlotAllocatorAuthority {
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: "missing",
        source_evidence_reason:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_METHOD,
        source_evidence_fact_locator: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_FACT_LOCATOR,
        source_chain_complete: false,
    }
}

fn module_service_slot_allocation_intent_from_source_evidence(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
    )>,
) -> ModuleServiceSlotAllocationIntent {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleServiceSlotAllocationIntent {
            present: evidence.intent_present,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.intent_present {
                "observed_current_boot_defined"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.intent_status,
            source_evidence_reason: evidence.intent_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
            source_chain_complete: evidence.source_chain_complete,
        };
    }
    module_service_slot_allocation_intent_missing()
}

fn module_service_slot_allocation_intent_missing() -> ModuleServiceSlotAllocationIntent {
    ModuleServiceSlotAllocationIntent {
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
        source_evidence_reason:
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_METHOD,
        source_evidence_fact_locator: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_FACT_LOCATOR,
        source_chain_complete: false,
    }
}

fn module_service_slot_authority_inputs_from_source_evidence(
    source_evidence: Option<
        [(
            event_log::EventId,
            event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
        ); MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
    >,
) -> [ModuleServiceSlotAuthorityInput; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT] {
    if let Some(source_evidence) = source_evidence {
        return [
            module_service_slot_authority_input_from_source_evidence(
                MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0],
                Some(source_evidence[0]),
            ),
            module_service_slot_authority_input_from_source_evidence(
                MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[1],
                Some(source_evidence[1]),
            ),
            module_service_slot_authority_input_from_source_evidence(
                MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[2],
                Some(source_evidence[2]),
            ),
            module_service_slot_authority_input_from_source_evidence(
                MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[3],
                Some(source_evidence[3]),
            ),
            module_service_slot_authority_input_from_source_evidence(
                MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[4],
                Some(source_evidence[4]),
            ),
        ];
    }
    module_service_slot_authority_inputs_missing()
}

fn module_service_slot_authority_input_from_source_evidence(
    source: ModuleServiceSlotAuthorityInputSpec,
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
    )>,
) -> ModuleServiceSlotAuthorityInput {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleServiceSlotAuthorityInput {
            spec: source,
            present: evidence.input_present,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.input_present {
                "observed_current_boot_defined"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.input_status,
            source_evidence_reason: evidence.input_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
            dependency_source_evidence_event_id: evidence.dependency_source_evidence_event_id,
            source_chain_complete: evidence.source_chain_complete,
        };
    }
    module_service_slot_authority_input_missing(source)
}

fn module_service_slot_authority_inputs_missing(
) -> [ModuleServiceSlotAuthorityInput; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT] {
    [
        module_service_slot_authority_input_missing(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0]),
        module_service_slot_authority_input_missing(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[1]),
        module_service_slot_authority_input_missing(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[2]),
        module_service_slot_authority_input_missing(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[3]),
        module_service_slot_authority_input_missing(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[4]),
    ]
}

fn module_service_slot_authority_input_missing(
    source: ModuleServiceSlotAuthorityInputSpec,
) -> ModuleServiceSlotAuthorityInput {
    ModuleServiceSlotAuthorityInput {
        spec: source,
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema: source.source_evidence_schema,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: source.missing_status,
        source_evidence_reason: source.source_evidence_missing_reason,
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
        dependency_source_evidence_event_id: None,
        source_chain_complete: false,
    }
}

fn module_service_slot_allocator_authority_decision_from_source_evidence(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
    )>,
) -> ModuleServiceSlotAllocatorAuthorityDecision {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleServiceSlotAllocatorAuthorityDecision {
            present: evidence.decision_present,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.decision_present {
                "observed_current_boot_defined"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.decision_status,
            source_evidence_reason: evidence.decision_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
            source_chain_complete: evidence.source_chain_complete,
            input_chain_complete: evidence.authority_inputs_complete,
        };
    }
    module_service_slot_allocator_authority_decision_missing()
}

fn module_service_slot_allocator_authority_decision_missing(
) -> ModuleServiceSlotAllocatorAuthorityDecision {
    ModuleServiceSlotAllocatorAuthorityDecision {
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
        source_evidence_reason:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_FACT_LOCATOR,
        source_chain_complete: false,
        input_chain_complete: false,
    }
}

fn module_service_slot_registry_write_commit_gate_from_source_evidence(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence,
    )>,
) -> ModuleServiceSlotRegistryWriteCommitGate {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleServiceSlotRegistryWriteCommitGate {
            present: evidence.gate_present,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.gate_present {
                "observed_current_boot_defined"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.gate_status,
            source_evidence_reason: evidence.gate_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
            source_chain_complete: evidence.source_chain_complete,
            authority_decision_present: evidence.authority_decision_present,
            registry_write_authority_present: evidence.registry_write_authority_present,
            registry_binding_available: evidence.registry_binding_available,
            durable_audit_write_available: evidence.durable_audit_write_available,
            rollback_plan_install_available: evidence.rollback_plan_install_available,
            retained_service_slot_reservation_present: evidence
                .retained_service_slot_reservation_present,
        };
    }
    module_service_slot_registry_write_commit_gate_missing()
}

fn module_service_slot_registry_write_commit_gate_missing(
) -> ModuleServiceSlotRegistryWriteCommitGate {
    ModuleServiceSlotRegistryWriteCommitGate {
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
        source_evidence_reason:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_FACT_LOCATOR,
        source_chain_complete: false,
        authority_decision_present: false,
        registry_write_authority_present: false,
        registry_binding_available: false,
        durable_audit_write_available: false,
        rollback_plan_install_available: false,
        retained_service_slot_reservation_present: false,
    }
}

fn module_service_slot_allocator_fact_from_source_evidence(
    source: ModuleServiceSlotAllocatorFactSource,
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    )>,
) -> ModuleServiceSlotAllocatorFact {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleServiceSlotAllocatorFact {
            present: evidence.fact_present,
            schema_ok: evidence.fact_schema_ok,
            scope: evidence.fact_scope,
            provenance_ok: evidence.fact_provenance_ok,
            classification: evidence.fact_classification,
            binds_retained_reservation: evidence.binds_retained_service_slot_reservation,
            binds_allocator_runtime: evidence.binds_allocator_runtime,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.fact_present {
                "observed_current_boot_available"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.fact_status,
            source_evidence_reason: evidence.fact_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
        };
    }
    module_service_slot_allocator_missing_fact(source)
}

fn module_service_slot_allocator_missing_fact(
    source: ModuleServiceSlotAllocatorFactSource,
) -> ModuleServiceSlotAllocatorFact {
    ModuleServiceSlotAllocatorFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_retained_reservation: false,
        binds_allocator_runtime: false,
        source_evidence_event_id: None,
        source_evidence_schema: source.source_evidence_schema,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: "missing",
        source_evidence_reason: source.source_evidence_missing_reason,
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
    }
}

fn module_service_slot_allocator_available_fact(
    source: ModuleServiceSlotAllocatorFactSource,
) -> ModuleServiceSlotAllocatorFact {
    ModuleServiceSlotAllocatorFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_retained_reservation: true,
        binds_allocator_runtime: true,
        source_evidence_event_id: None,
        source_evidence_schema: source.source_evidence_schema,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: "available",
        source_evidence_reason: "service_slot_allocator_fact_available",
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
    }
}

fn module_service_slot_allocator_prerequisite_from_source_evidence(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    )>,
) -> ModuleServiceSlotAllocatorPrerequisite {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleServiceSlotAllocatorPrerequisite {
            available: evidence.prerequisite_available,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.prerequisite_available {
                "observed_current_boot_available"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.prerequisite_status,
            source_evidence_reason: evidence.prerequisite_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
        };
    }
    module_service_slot_allocator_missing_prerequisite(source)
}

fn module_service_slot_allocator_missing_prerequisite(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
) -> ModuleServiceSlotAllocatorPrerequisite {
    ModuleServiceSlotAllocatorPrerequisite {
        available: false,
        source_evidence_event_id: None,
        source_evidence_schema: source.source_evidence_schema,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: source.missing_status,
        source_evidence_reason: source.source_evidence_missing_reason,
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
    }
}

fn module_service_slot_allocator_available_prerequisite(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
) -> ModuleServiceSlotAllocatorPrerequisite {
    ModuleServiceSlotAllocatorPrerequisite {
        available: true,
        source_evidence_event_id: None,
        source_evidence_schema: source.source_evidence_schema,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: "available",
        source_evidence_reason: "service_slot_allocator_prerequisite_available",
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
    }
}

fn module_service_slot_allocator_available_authority() -> ModuleServiceSlotAllocatorAuthority {
    ModuleServiceSlotAllocatorAuthority {
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: "defined_non_authorizing",
        source_evidence_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_METHOD,
        source_evidence_fact_locator: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_FACT_LOCATOR,
        source_chain_complete: true,
    }
}

fn module_service_slot_allocation_intent_available() -> ModuleServiceSlotAllocationIntent {
    ModuleServiceSlotAllocationIntent {
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_STATUS,
        source_evidence_reason: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_METHOD,
        source_evidence_fact_locator: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_FACT_LOCATOR,
        source_chain_complete: true,
    }
}

fn module_service_slot_authority_inputs_available(
) -> [ModuleServiceSlotAuthorityInput; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT] {
    [
        module_service_slot_authority_input_available(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0],
        ),
        module_service_slot_authority_input_available(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[1],
        ),
        module_service_slot_authority_input_available(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[2],
        ),
        module_service_slot_authority_input_available(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[3],
        ),
        module_service_slot_authority_input_available(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[4],
        ),
    ]
}

fn module_service_slot_authority_input_available(
    source: ModuleServiceSlotAuthorityInputSpec,
) -> ModuleServiceSlotAuthorityInput {
    ModuleServiceSlotAuthorityInput {
        spec: source,
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema: source.source_evidence_schema,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: source.available_status,
        source_evidence_reason: source.available_reason,
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
        dependency_source_evidence_event_id: None,
        source_chain_complete: true,
    }
}

fn module_service_slot_allocator_authority_decision_available(
) -> ModuleServiceSlotAllocatorAuthorityDecision {
    ModuleServiceSlotAllocatorAuthorityDecision {
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_STATUS,
        source_evidence_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_FACT_LOCATOR,
        source_chain_complete: true,
        input_chain_complete: true,
    }
}

fn module_service_slot_registry_write_commit_gate_available(
) -> ModuleServiceSlotRegistryWriteCommitGate {
    ModuleServiceSlotRegistryWriteCommitGate {
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_STATUS,
        source_evidence_reason: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_REASON,
        source_evidence_method: MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_FACT_LOCATOR,
        source_chain_complete: true,
        authority_decision_present: true,
        registry_write_authority_present: true,
        registry_binding_available: true,
        durable_audit_write_available: true,
        rollback_plan_install_available: true,
        retained_service_slot_reservation_present: true,
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

    let durable_audit_status = if candidate.durable_audit_write.available {
        "available"
    } else {
        "missing"
    };
    let durable_audit_reason = if candidate.durable_audit_write.available {
        "durable_audit_write_available"
    } else {
        "durable_audit_write_missing"
    };
    let rollback_status = if candidate.rollback_plan_install.available {
        "available"
    } else {
        "missing"
    };
    let rollback_reason = if candidate.rollback_plan_install.available {
        "rollback_plan_install_available"
    } else {
        "rollback_install_missing"
    };
    let module_loader_status = if candidate.module_loader.available {
        "available"
    } else {
        "unavailable"
    };
    let module_loader_reason = if candidate.module_loader.available {
        "module_loader_boundary_available_non_authorizing"
    } else {
        "module_loader_unimplemented"
    };
    let authority_status = if candidate.allocator_authority.present
        && candidate.allocator_authority.source_chain_complete
    {
        "defined_non_authorizing"
    } else {
        "missing"
    };
    let authority_reason = if candidate.allocator_authority.present
        && candidate.allocator_authority.source_chain_complete
    {
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON
    } else {
        "service_slot_allocator_authority_source_chain_incomplete"
    };
    let allocation_intent_status = if candidate.allocation_intent.present
        && candidate.allocation_intent.source_chain_complete
    {
        MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_STATUS
    } else {
        MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS
    };
    let allocation_intent_reason = if candidate.allocation_intent.present
        && candidate.allocation_intent.source_chain_complete
    {
        MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_REASON
    } else {
        MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_CHAIN_INCOMPLETE_REASON
    };
    let authority_input_statuses = [
        module_service_slot_authority_input_status(candidate.authority_inputs[0]),
        module_service_slot_authority_input_status(candidate.authority_inputs[1]),
        module_service_slot_authority_input_status(candidate.authority_inputs[2]),
        module_service_slot_authority_input_status(candidate.authority_inputs[3]),
        module_service_slot_authority_input_status(candidate.authority_inputs[4]),
    ];
    let authority_input_reasons = [
        module_service_slot_authority_input_reason(candidate.authority_inputs[0]),
        module_service_slot_authority_input_reason(candidate.authority_inputs[1]),
        module_service_slot_authority_input_reason(candidate.authority_inputs[2]),
        module_service_slot_authority_input_reason(candidate.authority_inputs[3]),
        module_service_slot_authority_input_reason(candidate.authority_inputs[4]),
    ];
    let authority_decision_status = if candidate.authority_decision.present
        && candidate.authority_decision.source_chain_complete
        && candidate.authority_decision.input_chain_complete
    {
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_STATUS
    } else {
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS
    };
    let authority_decision_reason = if candidate.authority_decision.present
        && candidate.authority_decision.source_chain_complete
        && candidate.authority_decision.input_chain_complete
    {
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_REASON
    } else {
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_CHAIN_INCOMPLETE_REASON
    };
    let registry_write_commit_gate_status = if candidate.registry_write_commit_gate.present
        && candidate.registry_write_commit_gate.source_chain_complete
    {
        MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_STATUS
    } else {
        MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS
    };
    let registry_write_commit_gate_reason = if candidate.registry_write_commit_gate.present
        && candidate.registry_write_commit_gate.source_chain_complete
    {
        MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_REASON
    } else {
        MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_CHAIN_INCOMPLETE_REASON
    };
    let mut authority_inputs_complete = true;
    let mut first_authority_input_reason =
        MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0].source_chain_incomplete_reason;
    let mut authority_input_idx = 0usize;
    while authority_input_idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        if !candidate.authority_inputs[authority_input_idx].present
            || !candidate.authority_inputs[authority_input_idx].source_chain_complete
        {
            authority_inputs_complete = false;
            first_authority_input_reason = authority_input_reasons[authority_input_idx];
            break;
        }
        authority_input_idx += 1;
    }

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
    } else if !candidate.durable_audit_write.available {
        ("denied_missing_durable_audit_write", durable_audit_reason)
    } else if !candidate.rollback_plan_install.available {
        ("denied_missing_rollback_install", rollback_reason)
    } else if !candidate.module_loader.available {
        ("denied_loader_unimplemented", module_loader_reason)
    } else if !candidate.allocator_authority.present {
        (
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_MISSING_STATUS,
            authority_reason,
        )
    } else if !candidate.allocation_intent.present
        || !candidate.allocation_intent.source_chain_complete
    {
        (
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
            allocation_intent_reason,
        )
    } else if !authority_inputs_complete {
        ("missing", first_authority_input_reason)
    } else if !candidate.authority_decision.present
        || !candidate.authority_decision.source_chain_complete
        || !candidate.authority_decision.input_chain_complete
    {
        (
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
            authority_decision_reason,
        )
    } else if !candidate.registry_write_commit_gate.present
        || !candidate.registry_write_commit_gate.source_chain_complete
    {
        (
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason,
        )
    } else {
        (
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
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
        authority_status,
        authority_reason,
        allocation_intent_status,
        allocation_intent_reason,
        authority_input_statuses,
        authority_input_reasons,
        authority_decision_status,
        authority_decision_reason,
        registry_write_commit_gate_status,
        registry_write_commit_gate_reason,
        allocates_service_slot: false,
        creates_service_inventory_records: false,
        can_allocate: false,
        can_load: false,
        load_attempted: false,
    }
}

fn module_service_slot_authority_input_status(
    input: ModuleServiceSlotAuthorityInput,
) -> &'static str {
    if input.present && input.source_chain_complete {
        input.spec.available_status
    } else {
        input.spec.missing_status
    }
}

fn module_service_slot_authority_input_reason(
    input: ModuleServiceSlotAuthorityInput,
) -> &'static str {
    if input.present && input.source_chain_complete {
        input.spec.available_reason
    } else {
        input.spec.source_chain_incomplete_reason
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

fn module_service_slot_allocator_observed_missing_fact(
    source: ModuleServiceSlotAllocatorFactSource,
    sequence: u64,
) -> ModuleServiceSlotAllocatorFact {
    ModuleServiceSlotAllocatorFact {
        source_evidence_event_id: Some(event_log::EventId { sequence }),
        source_evidence_state: "observed_current_boot_missing",
        source_evidence_status: "missing",
        source_evidence_reason: source.missing_reason,
        ..module_service_slot_allocator_missing_fact(source)
    }
}

fn module_service_slot_allocator_observed_available_fact(
    source: ModuleServiceSlotAllocatorFactSource,
    sequence: u64,
    available_reason: &'static str,
) -> ModuleServiceSlotAllocatorFact {
    ModuleServiceSlotAllocatorFact {
        source_evidence_event_id: Some(event_log::EventId { sequence }),
        source_evidence_state: "observed_current_boot_available",
        source_evidence_status: "available",
        source_evidence_reason: available_reason,
        ..module_service_slot_allocator_available_fact(source)
    }
}

fn module_service_slot_allocator_observed_missing_prerequisite(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
    sequence: u64,
) -> ModuleServiceSlotAllocatorPrerequisite {
    ModuleServiceSlotAllocatorPrerequisite {
        source_evidence_event_id: Some(event_log::EventId { sequence }),
        source_evidence_state: "observed_current_boot_missing",
        source_evidence_status: source.missing_status,
        source_evidence_reason: source.missing_reason,
        ..module_service_slot_allocator_missing_prerequisite(source)
    }
}

fn module_service_slot_allocator_observed_available_prerequisite(
    source: ModuleServiceSlotAllocatorPrerequisiteSource,
    sequence: u64,
    available_reason: &'static str,
) -> ModuleServiceSlotAllocatorPrerequisite {
    ModuleServiceSlotAllocatorPrerequisite {
        available: true,
        source_evidence_event_id: Some(event_log::EventId { sequence }),
        source_evidence_state: "observed_current_boot_available",
        source_evidence_status: "available",
        source_evidence_reason: available_reason,
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
        source_evidence_schema: source.source_evidence_schema,
    }
}

fn module_service_slot_allocator_selftest_cases(
) -> [ModuleServiceSlotAllocatorSelfTestCase; MODULE_SERVICE_SLOT_ALLOCATOR_SELFTEST_CASES] {
    let missing = module_service_slot_allocator_empty_snapshot(false);
    let allocator_available =
        module_service_slot_allocator_available_fact(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0]);
    let registry_available =
        module_service_slot_allocator_available_fact(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1]);
    let health_available =
        module_service_slot_allocator_available_fact(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2]);
    let unload_available =
        module_service_slot_allocator_available_fact(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3]);
    let durable_available = module_service_slot_allocator_available_prerequisite(
        MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0],
    );
    let rollback_available = module_service_slot_allocator_available_prerequisite(
        MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1],
    );
    let module_loader_available = module_service_slot_allocator_available_prerequisite(
        MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2],
    );
    let allocator_authority_available = module_service_slot_allocator_available_authority();
    let allocation_intent_available = module_service_slot_allocation_intent_available();
    let authority_inputs_available = module_service_slot_authority_inputs_available();
    let authority_decision_available = module_service_slot_allocator_authority_decision_available();
    let registry_write_commit_gate_available =
        module_service_slot_registry_write_commit_gate_available();
    let ready = ModuleServiceSlotAllocatorCandidate {
        retained_reservation_present: true,
        allocator_runtime: allocator_available,
        registry_binding: registry_available,
        health_state: health_available,
        unload_cleanup: unload_available,
        durable_audit_write: durable_available,
        rollback_plan_install: rollback_available,
        module_loader: module_loader_available,
        allocator_authority: allocator_authority_available,
        allocation_intent: allocation_intent_available,
        authority_inputs: authority_inputs_available,
        authority_decision: authority_decision_available,
        registry_write_commit_gate: registry_write_commit_gate_available,
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
                    ..allocator_available
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
                    ..allocator_available
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
                    ..allocator_available
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
                    ..allocator_available
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
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_allocator_runtime_observed_source_evidence_missing",
            "missing",
            "service_slot_allocator_runtime_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: module_service_slot_allocator_observed_missing_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0],
                    42,
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_allocator_runtime_observed_source_evidence_available_registry_missing",
            "missing",
            "service_slot_registry_binding_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: module_service_slot_allocator_observed_available_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0],
                    49,
                    "service_slot_allocator_runtime_available",
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_registry_binding_missing",
            "missing",
            "service_slot_registry_binding_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_registry_binding_observed_source_evidence_missing",
            "missing",
            "service_slot_registry_binding_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                registry_binding: module_service_slot_allocator_observed_missing_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1],
                    43,
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_registry_binding_observed_source_evidence_available_health_missing",
            "missing",
            "service_health_state_model_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                registry_binding: module_service_slot_allocator_observed_available_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1],
                    50,
                    "service_slot_registry_binding_available",
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_registry_allocator_runtime_binding_missing",
            "rejected",
            "service_slot_registry_allocator_runtime_binding_missing",
            ModuleServiceSlotAllocatorCandidate {
                registry_binding: ModuleServiceSlotAllocatorFact {
                    binds_allocator_runtime: false,
                    ..registry_available
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
                allocator_runtime: allocator_available,
                registry_binding: registry_available,
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_health_state_model_observed_source_evidence_missing",
            "missing",
            "service_health_state_model_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                registry_binding: registry_available,
                health_state: module_service_slot_allocator_observed_missing_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2],
                    44,
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_health_state_model_observed_source_evidence_available_unload_missing",
            "missing",
            "service_unload_cleanup_plan_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                registry_binding: registry_available,
                health_state: module_service_slot_allocator_observed_available_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2],
                    51,
                    "service_health_state_model_available",
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_unload_cleanup_plan_missing",
            "missing",
            "service_unload_cleanup_plan_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                registry_binding: registry_available,
                health_state: health_available,
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_unload_cleanup_plan_observed_source_evidence_missing",
            "missing",
            "service_unload_cleanup_plan_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                registry_binding: registry_available,
                health_state: health_available,
                unload_cleanup: module_service_slot_allocator_observed_missing_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3],
                    45,
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_unload_cleanup_plan_observed_source_evidence_available_durable_missing",
            "denied_missing_durable_audit_write",
            "durable_audit_write_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                registry_binding: registry_available,
                health_state: health_available,
                unload_cleanup: module_service_slot_allocator_observed_available_fact(
                    MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3],
                    52,
                    "service_unload_cleanup_plan_available",
                ),
                ..module_service_slot_allocator_empty_snapshot(true)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "durable_audit_write_missing",
            "denied_missing_durable_audit_write",
            "durable_audit_write_missing",
            ModuleServiceSlotAllocatorCandidate {
                durable_audit_write: module_service_slot_allocator_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0],
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "durable_audit_write_observed_source_evidence_missing",
            "denied_missing_durable_audit_write",
            "durable_audit_write_missing",
            ModuleServiceSlotAllocatorCandidate {
                durable_audit_write: module_service_slot_allocator_observed_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0],
                    46,
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "durable_audit_write_observed_source_evidence_available_rollback_missing",
            "denied_missing_rollback_install",
            "rollback_install_missing",
            ModuleServiceSlotAllocatorCandidate {
                durable_audit_write: module_service_slot_allocator_observed_available_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0],
                    53,
                    "durable_audit_write_available",
                ),
                rollback_plan_install: module_service_slot_allocator_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1],
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "rollback_install_missing",
            "denied_missing_rollback_install",
            "rollback_install_missing",
            ModuleServiceSlotAllocatorCandidate {
                rollback_plan_install: module_service_slot_allocator_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1],
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "rollback_install_observed_source_evidence_missing",
            "denied_missing_rollback_install",
            "rollback_install_missing",
            ModuleServiceSlotAllocatorCandidate {
                rollback_plan_install: module_service_slot_allocator_observed_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1],
                    47,
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "rollback_install_observed_source_evidence_available_module_loader_unimplemented",
            "denied_loader_unimplemented",
            "module_loader_unimplemented",
            ModuleServiceSlotAllocatorCandidate {
                durable_audit_write: durable_available,
                rollback_plan_install:
                    module_service_slot_allocator_observed_available_prerequisite(
                        MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1],
                        54,
                        "rollback_plan_install_available",
                    ),
                module_loader: module_service_slot_allocator_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2],
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "module_loader_missing",
            "denied_loader_unimplemented",
            "module_loader_unimplemented",
            ModuleServiceSlotAllocatorCandidate {
                module_loader: module_service_slot_allocator_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2],
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "module_loader_observed_source_evidence_missing",
            "denied_loader_unimplemented",
            "module_loader_unimplemented",
            ModuleServiceSlotAllocatorCandidate {
                module_loader: module_service_slot_allocator_observed_missing_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2],
                    48,
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "module_loader_observed_source_evidence_available_allocator_authority_boundary",
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
            ModuleServiceSlotAllocatorCandidate {
                module_loader: module_service_slot_allocator_observed_available_prerequisite(
                    MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2],
                    55,
                    "module_loader_boundary_available_non_authorizing",
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "registry_write_commit_gate_missing",
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_CHAIN_INCOMPLETE_REASON,
            ModuleServiceSlotAllocatorCandidate {
                registry_write_commit_gate: module_service_slot_registry_write_commit_gate_missing(
                ),
                ..ready
            },
        ),
        module_service_slot_allocator_selftest_case(
            "all_inputs_ready_still_non_authorizing",
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
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
        actual_allocator_runtime_source_evidence_present: candidate
            .allocator_runtime
            .source_evidence_event_id
            .is_some(),
        actual_allocator_runtime_source_evidence_state: candidate
            .allocator_runtime
            .source_evidence_state,
        actual_allocator_runtime_source_evidence_status: candidate
            .allocator_runtime
            .source_evidence_status,
        actual_allocator_runtime_source_evidence_reason: candidate
            .allocator_runtime
            .source_evidence_reason,
        actual_registry_binding_source_evidence_present: candidate
            .registry_binding
            .source_evidence_event_id
            .is_some(),
        actual_registry_binding_source_evidence_state: candidate
            .registry_binding
            .source_evidence_state,
        actual_registry_binding_source_evidence_status: candidate
            .registry_binding
            .source_evidence_status,
        actual_registry_binding_source_evidence_reason: candidate
            .registry_binding
            .source_evidence_reason,
        actual_health_state_source_evidence_present: candidate
            .health_state
            .source_evidence_event_id
            .is_some(),
        actual_health_state_source_evidence_state: candidate.health_state.source_evidence_state,
        actual_health_state_source_evidence_status: candidate.health_state.source_evidence_status,
        actual_health_state_source_evidence_reason: candidate.health_state.source_evidence_reason,
        actual_unload_cleanup_source_evidence_present: candidate
            .unload_cleanup
            .source_evidence_event_id
            .is_some(),
        actual_unload_cleanup_source_evidence_state: candidate.unload_cleanup.source_evidence_state,
        actual_unload_cleanup_source_evidence_status: candidate
            .unload_cleanup
            .source_evidence_status,
        actual_unload_cleanup_source_evidence_reason: candidate
            .unload_cleanup
            .source_evidence_reason,
        actual_durable_audit_source_evidence_present: candidate
            .durable_audit_write
            .source_evidence_event_id
            .is_some(),
        actual_durable_audit_source_evidence_state: candidate
            .durable_audit_write
            .source_evidence_state,
        actual_durable_audit_source_evidence_status: candidate
            .durable_audit_write
            .source_evidence_status,
        actual_durable_audit_source_evidence_reason: candidate
            .durable_audit_write
            .source_evidence_reason,
        actual_rollback_install_source_evidence_present: candidate
            .rollback_plan_install
            .source_evidence_event_id
            .is_some(),
        actual_rollback_install_source_evidence_state: candidate
            .rollback_plan_install
            .source_evidence_state,
        actual_rollback_install_source_evidence_status: candidate
            .rollback_plan_install
            .source_evidence_status,
        actual_rollback_install_source_evidence_reason: candidate
            .rollback_plan_install
            .source_evidence_reason,
        actual_module_loader_source_evidence_present: candidate
            .module_loader
            .source_evidence_event_id
            .is_some(),
        actual_module_loader_source_evidence_state: candidate.module_loader.source_evidence_state,
        actual_module_loader_source_evidence_status: candidate.module_loader.source_evidence_status,
        actual_module_loader_source_evidence_reason: candidate.module_loader.source_evidence_reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && module_service_slot_allocator_source_fact_map_complete()
            && module_service_slot_allocator_prerequisite_source_map_complete()
            && !actual.allocates_service_slot
            && !actual.creates_service_inventory_records
            && !actual.can_allocate
            && !actual.can_load
            && !actual.load_attempted,
    }
}
