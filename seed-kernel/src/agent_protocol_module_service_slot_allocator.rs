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
    let allocator_runtime_source_evidence = module_service_slot_allocator_fact_source_evidence(
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0],
        retained_event_id,
        None,
    );
    let allocator_runtime_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_fact_source_evidence(
            allocator_runtime_source_evidence,
        );
    let registry_binding_source_evidence = module_service_slot_allocator_fact_source_evidence(
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1],
        retained_event_id,
        Some(allocator_runtime_source_evidence_event_id),
    );
    let registry_binding_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_fact_source_evidence(
            registry_binding_source_evidence,
        );
    let health_state_source_evidence = module_service_slot_allocator_fact_source_evidence(
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2],
        retained_event_id,
        Some(allocator_runtime_source_evidence_event_id),
    );
    let health_state_source_evidence_event_id =
        event_log::record_module_service_slot_allocator_fact_source_evidence(
            health_state_source_evidence,
        );
    let unload_cleanup_source_evidence = module_service_slot_allocator_fact_source_evidence(
        MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3],
        retained_event_id,
        Some(allocator_runtime_source_evidence_event_id),
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
    let candidate = module_service_slot_allocator_snapshot(
        retained.is_some(),
        Some(allocator_runtime_observed_source_evidence),
        Some(registry_binding_observed_source_evidence),
        Some(health_state_observed_source_evidence),
        Some(unload_cleanup_observed_source_evidence),
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
    );
    raw_line(",");
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

fn emit_module_service_slot_allocator_source_evidence(
    allocator_runtime_event_id: event_log::EventId,
    allocator_runtime: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    registry_binding_event_id: event_log::EventId,
    registry_binding: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    health_state_event_id: event_log::EventId,
    health_state: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    unload_cleanup_event_id: event_log::EventId,
    unload_cleanup: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
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
        durable_audit_written: false,
        rollback_plan_installed: false,
        module_loader_available: false,
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

fn module_service_slot_allocator_selftest_cases(
) -> [ModuleServiceSlotAllocatorSelfTestCase; MODULE_SERVICE_SLOT_ALLOCATOR_SELFTEST_CASES] {
    let missing = module_service_slot_allocator_snapshot(false, None, None, None, None);
    let allocator_available =
        module_service_slot_allocator_available_fact(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0]);
    let registry_available =
        module_service_slot_allocator_available_fact(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1]);
    let health_available =
        module_service_slot_allocator_available_fact(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2]);
    let unload_available =
        module_service_slot_allocator_available_fact(MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3]);
    let ready = ModuleServiceSlotAllocatorCandidate {
        retained_reservation_present: true,
        allocator_runtime: allocator_available,
        registry_binding: registry_available,
        health_state: health_available,
        unload_cleanup: unload_available,
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
                ..module_service_slot_allocator_snapshot(true, None, None, None, None)
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
                ..module_service_slot_allocator_snapshot(true, None, None, None, None)
            },
        ),
        module_service_slot_allocator_selftest_case(
            "service_slot_registry_binding_missing",
            "missing",
            "service_slot_registry_binding_missing",
            ModuleServiceSlotAllocatorCandidate {
                retained_reservation_present: true,
                allocator_runtime: allocator_available,
                ..module_service_slot_allocator_snapshot(true, None, None, None, None)
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
                ..module_service_slot_allocator_snapshot(true, None, None, None, None)
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
                ..module_service_slot_allocator_snapshot(true, None, None, None, None)
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
                ..module_service_slot_allocator_snapshot(true, None, None, None, None)
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
                ..module_service_slot_allocator_snapshot(true, None, None, None, None)
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
                ..module_service_slot_allocator_snapshot(true, None, None, None, None)
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
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && module_service_slot_allocator_source_fact_map_complete()
            && !actual.allocates_service_slot
            && !actual.creates_service_inventory_records
            && !actual.can_allocate
            && !actual.can_load
            && !actual.load_attempted,
    }
}
