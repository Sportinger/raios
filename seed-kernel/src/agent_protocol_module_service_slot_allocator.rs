use alloc::{vec, vec::Vec};

use crate::agent_protocol_support::{
    emit_inline_record_object, emit_inline_record_object_fragment,
    emit_record_fields_trailing_comma, emit_record_object, emit_record_property_line,
    record_bool as b, record_event_or_null, record_false as no, record_field as f,
    record_inline as inline, record_null as null, record_object as object, record_sha as sha,
    record_str as s, record_str_or_null,
};
use crate::{agent_protocol_module_types::*, agent_protocol_support::*, event_log};
use raios_core::record::{Field, Value as V};

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
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_service_slot_allocator_readiness.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", no()),
            f("mutates_global_event_log", b(true)),
            f(
                "global_event_log_mutation",
                s("retained_current_boot_source_evidence_only"),
            ),
            f("creates_service_slot_reservation_records", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("can_allocate", no()),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        6,
    );
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
    emit_record_property_line(
        "policy_result",
        vec![
            f("readiness_status", s(evaluation.status)),
            f("readiness_reason", s(evaluation.reason)),
            f(
                "retained_service_slot_reservation_present",
                b(candidate.retained_reservation_present),
            ),
            f("retained_hash_reference_allocates_slot", no()),
            f(
                "allocator_runtime_available",
                b(method_eq(evaluation.allocator_runtime_status, "available")),
            ),
            f(
                "registry_binding_available",
                b(method_eq(evaluation.registry_binding_status, "available")),
            ),
            f(
                "health_state_available",
                b(method_eq(evaluation.health_state_status, "available")),
            ),
            f(
                "unload_cleanup_available",
                b(method_eq(evaluation.unload_cleanup_status, "available")),
            ),
            f(
                "durable_audit_written",
                b(candidate.durable_audit_write.available),
            ),
            f(
                "rollback_plan_installed",
                b(candidate.rollback_plan_install.available),
            ),
            f(
                "module_loader_available",
                b(candidate.module_loader.available),
            ),
            f("allocator_authority_status", s(evaluation.authority_status)),
            f("allocator_authority_reason", s(evaluation.authority_reason)),
            f(
                "allocation_intent_status",
                s(evaluation.allocation_intent_status),
            ),
            f(
                "allocation_intent_reason",
                s(evaluation.allocation_intent_reason),
            ),
            f(
                "authority_input_statuses",
                object(module_service_slot_authority_input_status_fields(
                    &evaluation,
                )),
            ),
            f(
                "authority_decision_status",
                s(evaluation.authority_decision_status),
            ),
            f(
                "authority_decision_reason",
                s(evaluation.authority_decision_reason),
            ),
            f(
                "registry_write_commit_gate_status",
                s(evaluation.registry_write_commit_gate_status),
            ),
            f(
                "registry_write_commit_gate_reason",
                s(evaluation.registry_write_commit_gate_reason),
            ),
            f("service_slot_reserved", no()),
            f("registry_write_committed", no()),
            f("mutates_service_registry", no()),
            f("writes_durable_audit_state", no()),
            f("installs_rollback_state", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("can_allocate", no()),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        true,
    );
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
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_service_slot_allocator_readiness_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_service_slot_reservation_records", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("can_allocate", no()),
            f("load_attempted", no()),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
        ],
        6,
    );
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

fn module_service_slot_authority_input_status_fields(
    evaluation: &ModuleServiceSlotAllocatorEvaluation,
) -> Vec<Field<'static>> {
    let mut fields = Vec::new();
    let mut input_idx = 0usize;
    while input_idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        fields.push(f(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[input_idx].name,
            inline(vec![
                f(
                    "schema",
                    s(MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[input_idx].schema),
                ),
                f("status", s(evaluation.authority_input_statuses[input_idx])),
                f("reason", s(evaluation.authority_input_reasons[input_idx])),
            ]),
        ));
        input_idx += 1;
    }
    fields
}

fn inline_event_id_options(values: &[Option<event_log::EventId>]) -> V<'static> {
    let mut out = Vec::new();
    let mut idx = 0usize;
    while idx < values.len() {
        out.push(record_event_or_null(values[idx]));
        idx += 1;
    }
    V::InlineArray(out)
}

fn inline_bools(values: &[bool]) -> V<'static> {
    let mut out = Vec::new();
    let mut idx = 0usize;
    while idx < values.len() {
        out.push(b(values[idx]));
        idx += 1;
    }
    V::InlineArray(out)
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
    emit_record_object(
        vec![
            f("kind", s("allocator_fact")),
            f("event_id", record_event_or_null(Some(event_id))),
            f("schema", s(evidence.schema)),
            f("status", s("retained_current_boot_source_evidence")),
            f(
                "reason",
                s("module_service_slot_allocator_fact_source_evidence_recorded"),
            ),
            f("fact_schema", s(evidence.fact_schema)),
            f("fact_id", s(evidence.fact_id)),
            f("source_method", s(evidence.source_method)),
            f("source_fact_locator", s(evidence.source_fact_locator)),
            f("fact_status", s(evidence.fact_status)),
            f("fact_reason", s(evidence.fact_reason)),
            f("fact_present", b(evidence.fact_present)),
            f(
                "retained_service_slot_reservation_event_id",
                record_event_or_null(evidence.retained_service_slot_reservation_event_id),
            ),
            f(
                "allocator_runtime_source_evidence_event_id",
                record_event_or_null(evidence.allocator_runtime_source_evidence_event_id),
            ),
            f("source_evidence_retained", b(true)),
            f("retention", s("current_boot_ram_event_log")),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        8,
        comma,
    );
}

fn emit_module_service_slot_allocator_prerequisite_source_evidence_item(
    event_id: event_log::EventId,
    evidence: event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    comma: bool,
) {
    emit_record_object(
        vec![
            f("kind", s("allocator_prerequisite")),
            f("event_id", record_event_or_null(Some(event_id))),
            f("schema", s(evidence.schema)),
            f("status", s("retained_current_boot_source_evidence")),
            f(
                "reason",
                s("module_service_slot_allocator_prerequisite_source_evidence_recorded"),
            ),
            f("prerequisite_schema", s(evidence.prerequisite_schema)),
            f("prerequisite_id", s(evidence.prerequisite_id)),
            f("source_method", s(evidence.source_method)),
            f("source_fact_locator", s(evidence.source_fact_locator)),
            f("prerequisite_status", s(evidence.prerequisite_status)),
            f("prerequisite_reason", s(evidence.prerequisite_reason)),
            f("prerequisite_available", b(evidence.prerequisite_available)),
            f(
                "allocator_runtime_source_evidence_event_id",
                record_event_or_null(evidence.allocator_runtime_source_evidence_event_id),
            ),
            f(
                "registry_binding_source_evidence_event_id",
                record_event_or_null(evidence.registry_binding_source_evidence_event_id),
            ),
            f(
                "health_state_source_evidence_event_id",
                record_event_or_null(evidence.health_state_source_evidence_event_id),
            ),
            f(
                "unload_cleanup_source_evidence_event_id",
                record_event_or_null(evidence.unload_cleanup_source_evidence_event_id),
            ),
            f("source_evidence_retained", b(true)),
            f("retention", s("current_boot_ram_event_log")),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        8,
        comma,
    );
}

fn emit_module_service_slot_allocator_authority_source_evidence_item(
    event_id: event_log::EventId,
    evidence: event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    comma: bool,
) {
    emit_record_object(
        vec![
            f("kind", s("allocator_authority")),
            f("event_id", record_event_or_null(Some(event_id))),
            f("schema", s(evidence.schema)),
            f("status", s("retained_current_boot_source_evidence")),
            f(
                "reason",
                s("module_service_slot_allocator_authority_source_evidence_recorded"),
            ),
            f("authority_schema", s(evidence.authority_schema)),
            f("authority_id", s(evidence.authority_id)),
            f("source_method", s(evidence.source_method)),
            f("source_fact_locator", s(evidence.source_fact_locator)),
            f("authority_status", s(evidence.authority_status)),
            f("authority_reason", s(evidence.authority_reason)),
            f("authority_scope", s(evidence.authority_scope)),
            f(
                "authority_classification",
                s(evidence.authority_classification),
            ),
            f("authority_present", b(evidence.authority_present)),
            f(
                "retained_service_slot_reservation_present",
                b(evidence.retained_service_slot_reservation_present),
            ),
            f(
                "allocator_runtime_available",
                b(evidence.allocator_runtime_available),
            ),
            f(
                "registry_binding_available",
                b(evidence.registry_binding_available),
            ),
            f("health_state_available", b(evidence.health_state_available)),
            f(
                "unload_cleanup_available",
                b(evidence.unload_cleanup_available),
            ),
            f(
                "durable_audit_write_available",
                b(evidence.durable_audit_write_available),
            ),
            f(
                "rollback_plan_install_available",
                b(evidence.rollback_plan_install_available),
            ),
            f(
                "module_loader_available",
                b(evidence.module_loader_available),
            ),
            f("source_chain_complete", b(evidence.source_chain_complete)),
            f(
                "allocator_runtime_source_evidence_event_id",
                record_event_or_null(evidence.allocator_runtime_source_evidence_event_id),
            ),
            f(
                "registry_binding_source_evidence_event_id",
                record_event_or_null(evidence.registry_binding_source_evidence_event_id),
            ),
            f(
                "health_state_source_evidence_event_id",
                record_event_or_null(evidence.health_state_source_evidence_event_id),
            ),
            f(
                "unload_cleanup_source_evidence_event_id",
                record_event_or_null(evidence.unload_cleanup_source_evidence_event_id),
            ),
            f(
                "durable_audit_source_evidence_event_id",
                record_event_or_null(evidence.durable_audit_source_evidence_event_id),
            ),
            f(
                "rollback_install_source_evidence_event_id",
                record_event_or_null(evidence.rollback_install_source_evidence_event_id),
            ),
            f(
                "module_loader_source_evidence_event_id",
                record_event_or_null(evidence.module_loader_source_evidence_event_id),
            ),
            f("source_evidence_retained", b(true)),
            f("retention", s("current_boot_ram_event_log")),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        8,
        comma,
    );
}

fn emit_module_service_slot_allocation_intent_source_evidence_item(
    event_id: event_log::EventId,
    evidence: event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
    comma: bool,
) {
    emit_record_object(
        vec![
            f("kind", s("allocation_intent")),
            f("event_id", record_event_or_null(Some(event_id))),
            f("schema", s(evidence.schema)),
            f("status", s("retained_current_boot_source_evidence")),
            f(
                "reason",
                s("module_service_slot_allocation_intent_source_evidence_recorded"),
            ),
            f("intent_schema", s(evidence.intent_schema)),
            f("intent_id", s(evidence.intent_id)),
            f("source_method", s(evidence.source_method)),
            f("source_fact_locator", s(evidence.source_fact_locator)),
            f("intent_status", s(evidence.intent_status)),
            f("intent_reason", s(evidence.intent_reason)),
            f("intent_present", b(evidence.intent_present)),
            f("intent_scope", s(evidence.intent_scope)),
            f("requested_capability", s(evidence.requested_capability)),
            f("load_mode", s(evidence.load_mode)),
            f("target", s(evidence.target)),
            f(
                "retained_module_evidence_present",
                b(evidence.retained_module_evidence_present),
            ),
            f(
                "retained_service_slot_reservation_present",
                b(evidence.retained_service_slot_reservation_present),
            ),
            f(
                "allocator_authority_present",
                b(evidence.allocator_authority_present),
            ),
            f("source_chain_complete", b(evidence.source_chain_complete)),
            f(
                "manifest_reference_event_id",
                record_event_or_null(evidence.manifest_reference_event_id),
            ),
            f(
                "candidate_artifact_reference_event_id",
                record_event_or_null(evidence.artifact_reference_event_id),
            ),
            f(
                "vm_test_report_reference_event_id",
                record_event_or_null(evidence.vm_report_reference_event_id),
            ),
            f(
                "local_attestation_reference_event_id",
                record_event_or_null(evidence.local_attestation_reference_event_id),
            ),
            f(
                "local_approval_reference_event_id",
                record_event_or_null(evidence.local_approval_reference_event_id),
            ),
            f(
                "computed_grant_reference_event_id",
                record_event_or_null(evidence.computed_grant_reference_event_id),
            ),
            f(
                "audit_rollback_reference_event_id",
                record_event_or_null(evidence.audit_rollback_reference_event_id),
            ),
            f(
                "service_slot_reservation_event_id",
                record_event_or_null(evidence.service_slot_reservation_event_id),
            ),
            f(
                "allocator_authority_source_evidence_event_id",
                record_event_or_null(evidence.allocator_authority_source_evidence_event_id),
            ),
            f(
                "ram_only_service_slot_id",
                record_str_or_null(
                    evidence
                        .ram_only_service_slot_id
                        .as_ref()
                        .map(|id| id.as_str()),
                ),
            ),
            f("source_evidence_retained", b(true)),
            f("retention", s("current_boot_ram_event_log")),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        8,
        comma,
    );
}

fn emit_module_service_slot_authority_input_source_evidence_item(
    event_id: event_log::EventId,
    evidence: event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
    comma: bool,
) {
    emit_record_object(
        vec![
            f("kind", s("authority_input")),
            f("event_id", record_event_or_null(Some(event_id))),
            f("schema", s(evidence.schema)),
            f("status", s("retained_current_boot_source_evidence")),
            f(
                "reason",
                s("module_service_slot_authority_input_source_evidence_recorded"),
            ),
            f("input_schema", s(evidence.input_schema)),
            f("input_id", s(evidence.input_id)),
            f("input_name", s(evidence.input_name)),
            f("source_method", s(evidence.source_method)),
            f("source_fact_locator", s(evidence.source_fact_locator)),
            f("input_status", s(evidence.input_status)),
            f("input_reason", s(evidence.input_reason)),
            f("input_present", b(evidence.input_present)),
            f("input_scope", s(evidence.input_scope)),
            f("dependency_schema", s(evidence.dependency_schema)),
            f(
                "dependency_source_evidence_event_id",
                record_event_or_null(evidence.dependency_source_evidence_event_id),
            ),
            f("dependency_present", b(evidence.dependency_present)),
            f("requested_capability", s(evidence.requested_capability)),
            f("load_mode", s(evidence.load_mode)),
            f("target", s(evidence.target)),
            f(
                "retained_module_evidence_present",
                b(evidence.retained_module_evidence_present),
            ),
            f(
                "retained_service_slot_reservation_present",
                b(evidence.retained_service_slot_reservation_present),
            ),
            f(
                "allocator_authority_present",
                b(evidence.allocator_authority_present),
            ),
            f(
                "allocation_intent_source_evidence_event_id",
                record_event_or_null(evidence.allocation_intent_source_evidence_event_id),
            ),
            f("source_chain_complete", b(evidence.source_chain_complete)),
            f(
                "service_slot_reservation_event_id",
                record_event_or_null(evidence.service_slot_reservation_event_id),
            ),
            f(
                "allocator_authority_source_evidence_event_id",
                record_event_or_null(evidence.allocator_authority_source_evidence_event_id),
            ),
            f(
                "ram_only_service_slot_id",
                record_str_or_null(
                    evidence
                        .ram_only_service_slot_id
                        .as_ref()
                        .map(|id| id.as_str()),
                ),
            ),
            f("source_evidence_retained", b(true)),
            f("retention", s("current_boot_ram_event_log")),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        8,
        comma,
    );
}

fn emit_module_service_slot_allocator_authority_decision_source_evidence_item(
    event_id: event_log::EventId,
    evidence: event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
    comma: bool,
) {
    emit_record_object(
        vec![
            f("kind", s("authority_decision")),
            f("event_id", record_event_or_null(Some(event_id))),
            f("schema", s(evidence.schema)),
            f("status", s("retained_current_boot_source_evidence")),
            f(
                "reason",
                s("module_service_slot_allocator_authority_decision_source_evidence_recorded"),
            ),
            f("decision_schema", s(evidence.decision_schema)),
            f("decision_id", s(evidence.decision_id)),
            f("source_method", s(evidence.source_method)),
            f("source_fact_locator", s(evidence.source_fact_locator)),
            f("decision_status", s(evidence.decision_status)),
            f("decision_reason", s(evidence.decision_reason)),
            f("decision_present", b(evidence.decision_present)),
            f("decision_scope", s(evidence.decision_scope)),
            f("requested_capability", s(evidence.requested_capability)),
            f("load_mode", s(evidence.load_mode)),
            f("target", s(evidence.target)),
            f(
                "allocator_authority_present",
                b(evidence.allocator_authority_present),
            ),
            f(
                "allocation_intent_present",
                b(evidence.allocation_intent_present),
            ),
            f(
                "authority_inputs_complete",
                b(evidence.authority_inputs_complete),
            ),
            f("source_chain_complete", b(evidence.source_chain_complete)),
            f(
                "allocator_authority_source_evidence_event_id",
                record_event_or_null(evidence.allocator_authority_source_evidence_event_id),
            ),
            f(
                "allocation_intent_source_evidence_event_id",
                record_event_or_null(evidence.allocation_intent_source_evidence_event_id),
            ),
            f(
                "authority_input_source_evidence_event_ids",
                inline_event_id_options(&evidence.authority_input_source_evidence_event_ids),
            ),
            f(
                "authority_input_present",
                inline_bools(&evidence.authority_input_present),
            ),
            f(
                "retained_module_evidence_present",
                b(evidence.retained_module_evidence_present),
            ),
            f(
                "retained_service_slot_reservation_present",
                b(evidence.retained_service_slot_reservation_present),
            ),
            f(
                "service_slot_reservation_event_id",
                record_event_or_null(evidence.service_slot_reservation_event_id),
            ),
            f(
                "ram_only_service_slot_id",
                record_str_or_null(
                    evidence
                        .ram_only_service_slot_id
                        .as_ref()
                        .map(|id| id.as_str()),
                ),
            ),
            f("source_evidence_retained", b(true)),
            f("retention", s("current_boot_ram_event_log")),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        8,
        comma,
    );
}

fn emit_module_service_slot_registry_write_commit_gate_source_evidence_item(
    event_id: event_log::EventId,
    evidence: event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence,
    comma: bool,
) {
    emit_record_object(
        vec![
            f("kind", s("registry_write_commit_gate")),
            f("event_id", record_event_or_null(Some(event_id))),
            f("schema", s(evidence.schema)),
            f("status", s("retained_current_boot_source_evidence")),
            f(
                "reason",
                s("module_service_slot_registry_write_commit_gate_source_evidence_recorded"),
            ),
            f("gate_schema", s(evidence.gate_schema)),
            f("gate_id", s(evidence.gate_id)),
            f("source_method", s(evidence.source_method)),
            f("source_fact_locator", s(evidence.source_fact_locator)),
            f("gate_status", s(evidence.gate_status)),
            f("gate_reason", s(evidence.gate_reason)),
            f("gate_present", b(evidence.gate_present)),
            f("gate_scope", s(evidence.gate_scope)),
            f("requested_capability", s(evidence.requested_capability)),
            f("load_mode", s(evidence.load_mode)),
            f("target", s(evidence.target)),
            f(
                "authority_decision_present",
                b(evidence.authority_decision_present),
            ),
            f(
                "registry_write_authority_present",
                b(evidence.registry_write_authority_present),
            ),
            f(
                "registry_binding_available",
                b(evidence.registry_binding_available),
            ),
            f(
                "durable_audit_write_available",
                b(evidence.durable_audit_write_available),
            ),
            f(
                "rollback_plan_install_available",
                b(evidence.rollback_plan_install_available),
            ),
            f(
                "retained_service_slot_reservation_present",
                b(evidence.retained_service_slot_reservation_present),
            ),
            f("source_chain_complete", b(evidence.source_chain_complete)),
            f(
                "authority_decision_source_evidence_event_id",
                record_event_or_null(evidence.authority_decision_source_evidence_event_id),
            ),
            f(
                "registry_write_authority_source_evidence_event_id",
                record_event_or_null(evidence.registry_write_authority_source_evidence_event_id),
            ),
            f(
                "registry_binding_source_evidence_event_id",
                record_event_or_null(evidence.registry_binding_source_evidence_event_id),
            ),
            f(
                "durable_audit_source_evidence_event_id",
                record_event_or_null(evidence.durable_audit_source_evidence_event_id),
            ),
            f(
                "rollback_install_source_evidence_event_id",
                record_event_or_null(evidence.rollback_install_source_evidence_event_id),
            ),
            f(
                "service_slot_reservation_event_id",
                record_event_or_null(evidence.service_slot_reservation_event_id),
            ),
            f(
                "ram_only_service_slot_id",
                record_str_or_null(
                    evidence
                        .ram_only_service_slot_id
                        .as_ref()
                        .map(|id| id.as_str()),
                ),
            ),
            f("source_evidence_retained", b(true)),
            f("retention", s("current_boot_ram_event_log")),
            f(
                "authorizes_registry_write",
                b(evidence.authorizes_registry_write),
            ),
            f(
                "mutates_service_registry",
                b(evidence.mutates_service_registry),
            ),
            f(
                "writes_durable_audit_state",
                b(evidence.writes_durable_audit_state),
            ),
            f(
                "installs_rollback_state",
                b(evidence.installs_rollback_state),
            ),
            f("allocates_service_slot", b(evidence.allocates_service_slot)),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("loads_artifact", b(evidence.loads_artifact)),
            f("load_attempted", b(evidence.loads_artifact)),
        ],
        8,
        comma,
    );
}

fn emit_module_service_slot_allocator_retained_reservation(
    retained: Option<(event_log::EventId, event_log::ModuleServiceSlotReservation)>,
) {
    if let Some((event_id, reference)) = retained {
        emit_record_property_line(
            "retained_service_slot_reservation",
            vec![
                f("state", s("present")),
                f("schema", s("raios.module_service_slot_reservation.v0")),
                f("event_id", record_event_or_null(Some(event_id))),
                f("status", s("retained_hash_reference_only_not_allocated")),
                f(
                    "reason",
                    s("service_slot_reservation_is_evidence_not_allocator_state"),
                ),
                f("classification", s("local_only")),
                f("allocates_service_slot", no()),
                f("creates_service_inventory_records", no()),
                f("service_inventory_change", s("none")),
                f("can_allocate", no()),
                f("can_load_now", no()),
                f("load_attempted", no()),
                f(
                    "retained_computed_grant_reference_event_id",
                    record_event_or_null(Some(reference.retained_reference_event_id)),
                ),
                f(
                    "retained_audit_rollback_reference_event_id",
                    record_event_or_null(Some(
                        reference.retained_audit_rollback_reference_event_id,
                    )),
                ),
                f(
                    "ram_only_service_slot_id",
                    s(reference.ram_only_service_slot_id.as_str()),
                ),
                f(
                    "hashes",
                    object(vec![
                        f("reservation_hash", sha(reference.reservation_hash)),
                        f(
                            "computed_capability_grant_hash",
                            sha(reference.computed_grant_hash),
                        ),
                        f("audit_record_hash", sha(reference.audit_record_hash)),
                        f("rollback_plan_hash", sha(reference.rollback_plan_hash)),
                        f(
                            "pre_load_service_inventory_hash",
                            sha(reference.pre_load_service_inventory_hash),
                        ),
                    ]),
                ),
            ],
            false,
        );
    } else {
        emit_record_property_line(
            "retained_service_slot_reservation",
            vec![
                f("state", s("missing")),
                f("schema", s("raios.module_service_slot_reservation.v0")),
                f("event_id", null()),
                f("status", s("missing")),
                f("reason", s("retained_service_slot_reservation_missing")),
                f("classification", s("local_only")),
                f("allocates_service_slot", no()),
                f("creates_service_inventory_records", no()),
                f("can_allocate", no()),
                f("can_load_now", no()),
                f("load_attempted", no()),
            ],
            false,
        );
    }
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
    emit_record_property_line_at(
        source.name,
        vec![
            f("schema", s(source.schema)),
            f("id", s(source.id)),
            f("source_method", s(source.source_method)),
            f("source_fact_locator", s(source.source_fact_locator)),
            f(
                "source_evidence_event_id",
                record_event_or_null(fact.source_evidence_event_id),
            ),
            f("source_evidence_schema", s(fact.source_evidence_schema)),
            f("source_evidence_state", s(fact.source_evidence_state)),
            f("source_evidence_status", s(fact.source_evidence_status)),
            f("source_evidence_reason", s(fact.source_evidence_reason)),
            f("source_evidence_method", s(fact.source_evidence_method)),
            f(
                "source_evidence_fact_locator",
                s(fact.source_evidence_fact_locator),
            ),
            f("scope", s(fact.scope)),
            f("classification", s(fact.classification)),
            f("status", s(status)),
            f("reason", s(reason)),
            f("present", b(fact.present)),
            f("schema_valid", b(fact.schema_ok)),
            f("provenance_valid", b(fact.provenance_ok)),
            f(
                "binds_retained_service_slot_reservation",
                b(fact.binds_retained_reservation),
            ),
            f("binds_allocator_runtime", b(fact.binds_allocator_runtime)),
            f("authority", s("current_snapshot")),
            f("persistence", s("none")),
            f("durable", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("authorizes_load", no()),
            f(
                "required_bindings",
                object(vec![
                    f(
                        "service_slot_reservation",
                        s("raios.module_service_slot_reservation.v0"),
                    ),
                    f(
                        "audit_write_boundary",
                        s("raios.module_audit_rollback_write_boundary.v0"),
                    ),
                    f("durable_audit_record", s("raios.audit_record.v0")),
                    f("rollback_plan", s("raios.rollback_plan.v0")),
                    f("module_loader", s("raios.module_loader.v0")),
                ]),
            ),
            f(
                "provenance",
                object(vec![
                    f("source_method", s(source.source_method)),
                    f("source_fact_locator", s(source.source_fact_locator)),
                    f("source_transport", s("serial-console")),
                    f("event_scope", s("current_boot")),
                    f("record_id", null()),
                ]),
            ),
        ],
        8,
        comma,
    );
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
    emit_record_property_line_at(
        source.name,
        vec![
            f("schema", s(source.schema)),
            f("id", s(source.id)),
            f("source_method", s(source.source_method)),
            f("source_fact_locator", s(source.source_fact_locator)),
            f(
                "source_evidence_event_id",
                record_event_or_null(prerequisite.source_evidence_event_id),
            ),
            f(
                "source_evidence_schema",
                s(prerequisite.source_evidence_schema),
            ),
            f(
                "source_evidence_state",
                s(prerequisite.source_evidence_state),
            ),
            f(
                "source_evidence_status",
                s(prerequisite.source_evidence_status),
            ),
            f(
                "source_evidence_reason",
                s(prerequisite.source_evidence_reason),
            ),
            f(
                "source_evidence_method",
                s(prerequisite.source_evidence_method),
            ),
            f(
                "source_evidence_fact_locator",
                s(prerequisite.source_evidence_fact_locator),
            ),
            f("status", s(status)),
            f("reason", s(reason)),
            f("available", b(prerequisite.available)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("authority", s("current_snapshot")),
            f("persistence", s("none")),
            f("durable", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("authorizes_load", no()),
            f(
                "provenance",
                object(vec![
                    f("source_method", s(source.source_method)),
                    f("source_fact_locator", s(source.source_fact_locator)),
                    f("source_transport", s("serial-console")),
                    f("event_scope", s("current_boot")),
                    f("record_id", null()),
                ]),
            ),
        ],
        8,
        comma,
    );
}

fn emit_module_service_slot_allocator_authority(
    candidate: ModuleServiceSlotAllocatorCandidate,
    evaluation: ModuleServiceSlotAllocatorEvaluation,
) {
    let authority = candidate.allocator_authority;
    emit_record_property_line(
        "allocator_authority_boundary",
        vec![
            f("schema", s(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SCHEMA)),
            f("id", s(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_ID)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("source_method", s(authority.source_evidence_method)),
            f(
                "source_fact_locator",
                s(authority.source_evidence_fact_locator),
            ),
            f(
                "source_evidence_event_id",
                record_event_or_null(authority.source_evidence_event_id),
            ),
            f(
                "source_evidence_schema",
                s(authority.source_evidence_schema),
            ),
            f("source_evidence_state", s(authority.source_evidence_state)),
            f(
                "source_evidence_status",
                s(authority.source_evidence_status),
            ),
            f(
                "source_evidence_reason",
                s(authority.source_evidence_reason),
            ),
            f("status", s(evaluation.authority_status)),
            f("reason", s(evaluation.authority_reason)),
            f("present", b(authority.present)),
            f("source_chain_complete", b(authority.source_chain_complete)),
            f(
                "future_authority_inputs",
                module_service_slot_allocator_authority_required_inputs(&evaluation),
            ),
            f("accepts_loader_descriptor", no()),
            f("accepts_artifact_bytes", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_allocate", no()),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        false,
    );
}

fn module_service_slot_allocator_authority_required_inputs(
    evaluation: &ModuleServiceSlotAllocatorEvaluation,
) -> V<'static> {
    let mut values = vec![module_service_slot_allocator_authority_required_input(
        "raios.service_slot_allocation_intent.v0",
        evaluation.allocation_intent_status,
        evaluation.allocation_intent_reason,
    )];
    let mut idx = 0usize;
    while idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        values.push(module_service_slot_allocator_authority_required_input(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[idx].schema,
            evaluation.authority_input_statuses[idx],
            evaluation.authority_input_reasons[idx],
        ));
        idx += 1;
    }
    V::Array(values)
}

fn module_service_slot_allocator_authority_required_input(
    schema: &'static str,
    state: &'static str,
    reason: &'static str,
) -> V<'static> {
    inline(vec![
        f("schema", s(schema)),
        f("state", s(state)),
        f("reason", s(reason)),
        f("required_before_allocation", b(true)),
        f("classification", s("local_only")),
    ])
}

fn emit_module_service_slot_allocation_intent(
    candidate: ModuleServiceSlotAllocatorCandidate,
    evaluation: ModuleServiceSlotAllocatorEvaluation,
) {
    let intent = candidate.allocation_intent;
    emit_record_property_line(
        "allocation_intent_boundary",
        vec![
            f("schema", s(MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SCHEMA)),
            f("id", s(MODULE_SERVICE_SLOT_ALLOCATION_INTENT_ID)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("source_method", s(intent.source_evidence_method)),
            f(
                "source_fact_locator",
                s(intent.source_evidence_fact_locator),
            ),
            f(
                "source_evidence_event_id",
                record_event_or_null(intent.source_evidence_event_id),
            ),
            f("source_evidence_schema", s(intent.source_evidence_schema)),
            f("source_evidence_state", s(intent.source_evidence_state)),
            f("source_evidence_status", s(intent.source_evidence_status)),
            f("source_evidence_reason", s(intent.source_evidence_reason)),
            f("status", s(evaluation.allocation_intent_status)),
            f("reason", s(evaluation.allocation_intent_reason)),
            f("present", b(intent.present)),
            f("source_chain_complete", b(intent.source_chain_complete)),
            f("requested_capability", s("cap.module.load_ephemeral")),
            f("load_mode", s("ram_only")),
            f("target", s("live_service_graph")),
            f("accepts_loader_descriptor", no()),
            f("accepts_artifact_bytes", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_allocate", no()),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        false,
    );
}

fn emit_module_service_slot_authority_inputs(
    candidate: ModuleServiceSlotAllocatorCandidate,
    evaluation: ModuleServiceSlotAllocatorEvaluation,
) {
    raw_line("      \"authority_input_boundaries\": {");
    let mut idx = 0usize;
    while idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        let input = candidate.authority_inputs[idx];
        emit_record_property_line_at(
            input.spec.name,
            vec![
                f("schema", s(input.spec.schema)),
                f("id", s(input.spec.id)),
                f("scope", s("current_boot")),
                f("classification", s("local_only")),
                f("source_method", s(input.source_evidence_method)),
                f("source_fact_locator", s(input.source_evidence_fact_locator)),
                f(
                    "source_evidence_event_id",
                    record_event_or_null(input.source_evidence_event_id),
                ),
                f("source_evidence_schema", s(input.source_evidence_schema)),
                f("source_evidence_state", s(input.source_evidence_state)),
                f("source_evidence_status", s(input.source_evidence_status)),
                f("source_evidence_reason", s(input.source_evidence_reason)),
                f(
                    "dependency_source_evidence_event_id",
                    record_event_or_null(input.dependency_source_evidence_event_id),
                ),
                f("status", s(evaluation.authority_input_statuses[idx])),
                f("reason", s(evaluation.authority_input_reasons[idx])),
                f("present", b(input.present)),
                f("source_chain_complete", b(input.source_chain_complete)),
                f("requested_capability", s("cap.module.load_ephemeral")),
                f("load_mode", s("ram_only")),
                f("target", s("live_service_graph")),
                f("accepts_loader_descriptor", no()),
                f("accepts_artifact_bytes", no()),
                f("allocates_service_slot", no()),
                f("creates_service_inventory_records", no()),
                f("service_inventory_change", s("none")),
                f("can_allocate", no()),
                f("can_load_now", no()),
                f("load_attempted", no()),
            ],
            8,
            idx + 1 != MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT,
        );
        idx += 1;
    }
    raw_line("      }");
}

fn emit_module_service_slot_allocator_authority_decision(
    candidate: ModuleServiceSlotAllocatorCandidate,
    evaluation: ModuleServiceSlotAllocatorEvaluation,
) {
    let decision = candidate.authority_decision;
    emit_record_property_line(
        "authority_decision",
        vec![
            f(
                "schema",
                s(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SCHEMA),
            ),
            f("id", s(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_ID)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("source_method", s(decision.source_evidence_method)),
            f(
                "source_fact_locator",
                s(decision.source_evidence_fact_locator),
            ),
            f(
                "source_evidence_event_id",
                record_event_or_null(decision.source_evidence_event_id),
            ),
            f("source_evidence_schema", s(decision.source_evidence_schema)),
            f("source_evidence_state", s(decision.source_evidence_state)),
            f("source_evidence_status", s(decision.source_evidence_status)),
            f("source_evidence_reason", s(decision.source_evidence_reason)),
            f("status", s(evaluation.authority_decision_status)),
            f("reason", s(evaluation.authority_decision_reason)),
            f("present", b(decision.present)),
            f("input_chain_complete", b(decision.input_chain_complete)),
            f("source_chain_complete", b(decision.source_chain_complete)),
            f("requested_capability", s("cap.module.load_ephemeral")),
            f("load_mode", s("ram_only")),
            f("target", s("live_service_graph")),
            f("authorizes_allocation", no()),
            f("authorizes_load", no()),
            f("accepts_loader_descriptor", no()),
            f("accepts_artifact_bytes", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_allocate", no()),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        false,
    );
}

fn emit_module_service_slot_registry_write_commit_gate(
    candidate: ModuleServiceSlotAllocatorCandidate,
    evaluation: ModuleServiceSlotAllocatorEvaluation,
) {
    let gate = candidate.registry_write_commit_gate;
    emit_record_property_line(
        "registry_write_commit_gate",
        vec![
            f(
                "schema",
                s(MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SCHEMA),
            ),
            f("id", s(MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_ID)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("source_method", s(gate.source_evidence_method)),
            f("source_fact_locator", s(gate.source_evidence_fact_locator)),
            f(
                "source_evidence_event_id",
                record_event_or_null(gate.source_evidence_event_id),
            ),
            f("source_evidence_schema", s(gate.source_evidence_schema)),
            f("source_evidence_state", s(gate.source_evidence_state)),
            f("source_evidence_status", s(gate.source_evidence_status)),
            f("source_evidence_reason", s(gate.source_evidence_reason)),
            f("status", s(evaluation.registry_write_commit_gate_status)),
            f("reason", s(evaluation.registry_write_commit_gate_reason)),
            f("present", b(gate.present)),
            f("source_chain_complete", b(gate.source_chain_complete)),
            f(
                "authority_decision_present",
                b(gate.authority_decision_present),
            ),
            f(
                "registry_write_authority_present",
                b(gate.registry_write_authority_present),
            ),
            f(
                "registry_binding_available",
                b(gate.registry_binding_available),
            ),
            f(
                "durable_audit_write_available",
                b(gate.durable_audit_write_available),
            ),
            f(
                "rollback_plan_install_available",
                b(gate.rollback_plan_install_available),
            ),
            f(
                "retained_service_slot_reservation_present",
                b(gate.retained_service_slot_reservation_present),
            ),
            f("requested_capability", s("cap.module.load_ephemeral")),
            f("load_mode", s("ram_only")),
            f("target", s("live_service_graph")),
            f("authorizes_registry_write", no()),
            f("authorizes_allocation", no()),
            f("authorizes_load", no()),
            f("mutates_service_registry", no()),
            f("writes_durable_audit_state", no()),
            f("installs_rollback_state", no()),
            f("accepts_loader_descriptor", no()),
            f("accepts_artifact_bytes", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_allocate", no()),
            f("can_load_now", no()),
            f("loads_artifact", no()),
            f("load_attempted", no()),
        ],
        false,
    );
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
    emit_inline_record_object_fragment(
        vec![
            f("gate", s(gate)),
            f("state", s(state)),
            f("reason", s(reason)),
        ],
        8,
    );
}

fn emit_module_service_slot_allocator_selftest_case(
    case: &ModuleServiceSlotAllocatorSelfTestCase,
    comma: bool,
) {
    emit_inline_record_object(
        vec![
            f("case", s(case.name)),
            f("expected_status", s(case.expected_status)),
            f("expected_reason", s(case.expected_reason)),
            f("actual_status", s(case.actual_status)),
            f("actual_reason", s(case.actual_reason)),
            f(
                "actual_allocator_runtime_source_evidence_present",
                b(case.actual_allocator_runtime_source_evidence_present),
            ),
            f(
                "actual_allocator_runtime_source_evidence_state",
                s(case.actual_allocator_runtime_source_evidence_state),
            ),
            f(
                "actual_allocator_runtime_source_evidence_status",
                s(case.actual_allocator_runtime_source_evidence_status),
            ),
            f(
                "actual_allocator_runtime_source_evidence_reason",
                s(case.actual_allocator_runtime_source_evidence_reason),
            ),
            f(
                "actual_registry_binding_source_evidence_present",
                b(case.actual_registry_binding_source_evidence_present),
            ),
            f(
                "actual_registry_binding_source_evidence_state",
                s(case.actual_registry_binding_source_evidence_state),
            ),
            f(
                "actual_registry_binding_source_evidence_status",
                s(case.actual_registry_binding_source_evidence_status),
            ),
            f(
                "actual_registry_binding_source_evidence_reason",
                s(case.actual_registry_binding_source_evidence_reason),
            ),
            f(
                "actual_health_state_source_evidence_present",
                b(case.actual_health_state_source_evidence_present),
            ),
            f(
                "actual_health_state_source_evidence_state",
                s(case.actual_health_state_source_evidence_state),
            ),
            f(
                "actual_health_state_source_evidence_status",
                s(case.actual_health_state_source_evidence_status),
            ),
            f(
                "actual_health_state_source_evidence_reason",
                s(case.actual_health_state_source_evidence_reason),
            ),
            f(
                "actual_unload_cleanup_source_evidence_present",
                b(case.actual_unload_cleanup_source_evidence_present),
            ),
            f(
                "actual_unload_cleanup_source_evidence_state",
                s(case.actual_unload_cleanup_source_evidence_state),
            ),
            f(
                "actual_unload_cleanup_source_evidence_status",
                s(case.actual_unload_cleanup_source_evidence_status),
            ),
            f(
                "actual_unload_cleanup_source_evidence_reason",
                s(case.actual_unload_cleanup_source_evidence_reason),
            ),
            f(
                "actual_durable_audit_source_evidence_present",
                b(case.actual_durable_audit_source_evidence_present),
            ),
            f(
                "actual_durable_audit_source_evidence_state",
                s(case.actual_durable_audit_source_evidence_state),
            ),
            f(
                "actual_durable_audit_source_evidence_status",
                s(case.actual_durable_audit_source_evidence_status),
            ),
            f(
                "actual_durable_audit_source_evidence_reason",
                s(case.actual_durable_audit_source_evidence_reason),
            ),
            f(
                "actual_rollback_install_source_evidence_present",
                b(case.actual_rollback_install_source_evidence_present),
            ),
            f(
                "actual_rollback_install_source_evidence_state",
                s(case.actual_rollback_install_source_evidence_state),
            ),
            f(
                "actual_rollback_install_source_evidence_status",
                s(case.actual_rollback_install_source_evidence_status),
            ),
            f(
                "actual_rollback_install_source_evidence_reason",
                s(case.actual_rollback_install_source_evidence_reason),
            ),
            f(
                "actual_module_loader_source_evidence_present",
                b(case.actual_module_loader_source_evidence_present),
            ),
            f(
                "actual_module_loader_source_evidence_state",
                s(case.actual_module_loader_source_evidence_state),
            ),
            f(
                "actual_module_loader_source_evidence_status",
                s(case.actual_module_loader_source_evidence_status),
            ),
            f(
                "actual_module_loader_source_evidence_reason",
                s(case.actual_module_loader_source_evidence_reason),
            ),
            f("passed", b(case.passed)),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("can_allocate", no()),
            f("can_load", no()),
            f("load_attempted", no()),
        ],
        comma,
    );
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
