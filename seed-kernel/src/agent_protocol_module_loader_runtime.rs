use crate::{agent_protocol_module_types::*, agent_protocol_support::*, event_log};

pub(crate) fn module_loader_runtime_method(method: &str) -> bool {
    method_head_eq(method, "module.loader_runtime")
}

pub(crate) fn module_loader_runtime_selftest_method(method: &str) -> bool {
    method_head_eq(method, "module.loader_runtime_selftest")
}

pub(crate) fn emit_module_loader_runtime() {
    let manifest = event_log::latest_module_manifest_reference();
    let artifact = event_log::latest_module_candidate_artifact_reference();
    let vm_report = event_log::latest_module_vm_test_report_reference();
    let local_attestation = event_log::latest_module_local_attestation_reference();
    let local_approval = event_log::latest_module_local_approval_reference();
    let computed_grant = event_log::latest_module_computed_grant_reference();
    let audit_rollback = event_log::latest_module_audit_rollback_reference();
    let service_slot = event_log::latest_module_service_slot_reservation();
    let loader_identity_source_evidence =
        event_log::latest_module_loader_identity_source_evidence();
    let artifact_hash_binding_source_evidence =
        event_log::latest_module_loader_artifact_hash_binding_source_evidence();
    let candidate = module_loader_runtime_snapshot(
        manifest.is_some(),
        artifact.is_some(),
        vm_report.is_some(),
        local_attestation.is_some(),
        local_approval.is_some(),
        computed_grant.is_some(),
        audit_rollback.is_some(),
        service_slot.is_some(),
        loader_identity_source_evidence,
        artifact_hash_binding_source_evidence,
    );
    let evaluation = evaluate_module_loader_runtime_candidate(candidate);

    begin_response("module.loader_runtime");
    raw_line("      \"schema\": \"raios.module_loader_runtime_readiness.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"global_event_log_mutation\": \"none\",");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"can_load_now\": false,");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"authorizes_guest_load\": false,");
    emit_module_loader_runtime_retained_evidence(
        manifest.as_ref().map(|(event_id, _)| *event_id),
        artifact.as_ref().map(|(event_id, _)| *event_id),
        vm_report.as_ref().map(|(event_id, _)| *event_id),
        local_attestation.as_ref().map(|(event_id, _)| *event_id),
        local_approval.as_ref().map(|(event_id, _)| *event_id),
        computed_grant.as_ref().map(|(event_id, _)| *event_id),
        audit_rollback.as_ref().map(|(event_id, _)| *event_id),
        service_slot.as_ref().map(|(event_id, _)| *event_id),
    );
    raw_line(",");
    emit_module_loader_runtime_service_slot_allocator_readiness(candidate, evaluation);
    raw_line(",");
    emit_module_loader_runtime_facts(candidate, evaluation);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"readiness_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"readiness_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"retained_module_evidence_complete\": ");
    raw_bool(module_loader_runtime_retained_evidence_complete(candidate));
    raw_line(",");
    raw("        \"service_slot_allocator_readiness_present\": ");
    raw_bool(candidate.service_slot_allocator_readiness_present);
    raw_line(",");
    raw("        \"service_slot_allocator_ready\": ");
    raw_bool(candidate.service_slot_allocator_ready);
    raw_line(",");
    raw("        \"loader_runtime_facts_complete\": ");
    raw_bool(module_loader_runtime_facts_complete(evaluation));
    raw_line(",");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false,");
    raw_line("        \"authorizes_guest_load\": false");
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote = false;
    emit_module_loader_runtime_gate(
        &mut wrote,
        "retained_module_manifest_reference",
        evaluation.manifest_reference_status,
        evaluation.manifest_reference_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "retained_module_candidate_artifact_reference",
        evaluation.artifact_reference_status,
        evaluation.artifact_reference_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "retained_module_vm_test_report_reference",
        evaluation.vm_report_reference_status,
        evaluation.vm_report_reference_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "retained_module_local_attestation_reference",
        evaluation.local_attestation_reference_status,
        evaluation.local_attestation_reference_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "retained_module_local_approval_reference",
        evaluation.local_approval_reference_status,
        evaluation.local_approval_reference_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "retained_module_computed_grant_reference",
        evaluation.computed_grant_reference_status,
        evaluation.computed_grant_reference_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "retained_module_audit_rollback_reference",
        evaluation.audit_rollback_reference_status,
        evaluation.audit_rollback_reference_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "retained_module_service_slot_reservation",
        evaluation.service_slot_reservation_status,
        evaluation.service_slot_reservation_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "service_slot_allocator_readiness",
        evaluation.service_slot_allocator_readiness_status,
        evaluation.service_slot_allocator_readiness_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "service_slot_allocator_runtime",
        evaluation.service_slot_allocator_runtime_status,
        evaluation.service_slot_allocator_runtime_reason,
    );
    emit_module_loader_runtime_fact_gate(
        &mut wrote,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[0],
        evaluation.loader_identity_status,
        evaluation.loader_identity_reason,
    );
    emit_module_loader_runtime_fact_gate(
        &mut wrote,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[1],
        evaluation.artifact_hash_binding_status,
        evaluation.artifact_hash_binding_reason,
    );
    emit_module_loader_runtime_fact_gate(
        &mut wrote,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[2],
        evaluation.entrypoint_abi_status,
        evaluation.entrypoint_abi_reason,
    );
    emit_module_loader_runtime_fact_gate(
        &mut wrote,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[3],
        evaluation.address_space_boundary_status,
        evaluation.address_space_boundary_reason,
    );
    emit_module_loader_runtime_fact_gate(
        &mut wrote,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[4],
        evaluation.memory_map_constraints_status,
        evaluation.memory_map_constraints_reason,
    );
    emit_module_loader_runtime_fact_gate(
        &mut wrote,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[5],
        evaluation.capability_import_table_status,
        evaluation.capability_import_table_reason,
    );
    emit_module_loader_runtime_fact_gate(
        &mut wrote,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[6],
        evaluation.service_slot_binding_status,
        evaluation.service_slot_binding_reason,
    );
    emit_module_loader_runtime_fact_gate(
        &mut wrote,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[7],
        evaluation.health_state_hooks_status,
        evaluation.health_state_hooks_reason,
    );
    emit_module_loader_runtime_fact_gate(
        &mut wrote,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[8],
        evaluation.rollback_hooks_status,
        evaluation.rollback_hooks_reason,
    );
    emit_module_loader_runtime_fact_gate(
        &mut wrote,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[9],
        evaluation.audit_rollback_write_boundary_binding_status,
        evaluation.audit_rollback_write_boundary_binding_reason,
    );
    crlf();
    raw_line("      ]");
    end_response("module.loader_runtime");
}

pub(crate) fn emit_module_loader_runtime_selftest() {
    let cases = module_loader_runtime_selftest_cases();
    let source_fact_map_complete = module_loader_runtime_source_fact_map_complete();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }
    passed = passed && source_fact_map_complete;

    begin_response("module.loader_runtime_selftest");
    raw_line("      \"schema\": \"raios.module_loader_runtime_readiness_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"can_load_now\": false,");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw("      \"source_fact_count\": ");
    raw_fmt(format_args!("{}", MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT));
    raw_line(",");
    raw("      \"source_fact_map_complete\": ");
    raw_bool(source_fact_map_complete);
    raw_line(",");
    raw_line("      \"source_fact_map\": [");
    emit_module_loader_runtime_source_fact_map();
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_loader_runtime_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response("module.loader_runtime_selftest");
}

fn emit_module_loader_runtime_retained_evidence(
    manifest_event_id: Option<event_log::EventId>,
    artifact_event_id: Option<event_log::EventId>,
    vm_report_event_id: Option<event_log::EventId>,
    local_attestation_event_id: Option<event_log::EventId>,
    local_approval_event_id: Option<event_log::EventId>,
    computed_grant_event_id: Option<event_log::EventId>,
    audit_rollback_event_id: Option<event_log::EventId>,
    service_slot_event_id: Option<event_log::EventId>,
) {
    raw_line("      \"retained_module_evidence\": {");
    emit_module_loader_runtime_retained_evidence_item(
        "manifest_reference",
        "raios.module_manifest_reference.v0",
        manifest_event_id,
        "retained_module_manifest_reference_available",
        "retained_module_manifest_reference_missing",
        true,
    );
    emit_module_loader_runtime_retained_evidence_item(
        "candidate_artifact_reference",
        "raios.module_candidate_artifact_reference.v0",
        artifact_event_id,
        "retained_module_candidate_artifact_reference_available",
        "retained_module_candidate_artifact_reference_missing",
        true,
    );
    emit_module_loader_runtime_retained_evidence_item(
        "vm_test_report_reference",
        "raios.module_vm_test_report_reference.v0",
        vm_report_event_id,
        "retained_module_vm_test_report_reference_available",
        "retained_module_vm_test_report_reference_missing",
        true,
    );
    emit_module_loader_runtime_retained_evidence_item(
        "local_attestation_reference",
        "raios.module_local_attestation_reference.v0",
        local_attestation_event_id,
        "retained_module_local_attestation_reference_available",
        "retained_module_local_attestation_reference_missing",
        true,
    );
    emit_module_loader_runtime_retained_evidence_item(
        "local_approval_reference",
        "raios.module_local_approval_reference.v0",
        local_approval_event_id,
        "retained_module_local_approval_reference_available",
        "retained_module_local_approval_reference_missing",
        true,
    );
    emit_module_loader_runtime_retained_evidence_item(
        "computed_grant_reference",
        "raios.module_computed_grant_reference.v0",
        computed_grant_event_id,
        "retained_module_computed_grant_reference_available",
        "retained_module_computed_grant_reference_missing",
        true,
    );
    emit_module_loader_runtime_retained_evidence_item(
        "audit_rollback_reference",
        "raios.module_audit_rollback_reference.v0",
        audit_rollback_event_id,
        "retained_module_audit_rollback_reference_available",
        "retained_module_audit_rollback_reference_missing",
        true,
    );
    emit_module_loader_runtime_retained_evidence_item(
        "service_slot_reservation",
        "raios.module_service_slot_reservation.v0",
        service_slot_event_id,
        "retained_module_service_slot_reservation_available",
        "retained_module_service_slot_reservation_missing",
        false,
    );
    raw_line("      }");
}

fn emit_module_loader_runtime_retained_evidence_item(
    name: &'static str,
    schema: &'static str,
    event_id: Option<event_log::EventId>,
    available_reason: &'static str,
    missing_reason: &'static str,
    comma: bool,
) {
    raw("        ");
    json_str(name);
    raw_line(": {");
    raw("          \"schema\": ");
    json_str(schema);
    raw_line(",");
    raw_line("          \"scope\": \"current_boot\",");
    raw_line("          \"classification\": \"local_only\",");
    raw("          \"state\": ");
    json_str(if event_id.is_some() {
        "present"
    } else {
        "missing"
    });
    raw_line(",");
    raw("          \"event_id\": ");
    json_event_id_option(event_id);
    raw_line(",");
    raw("          \"status\": ");
    json_str(if event_id.is_some() {
        "available"
    } else {
        "missing"
    });
    raw_line(",");
    raw("          \"reason\": ");
    json_str(if event_id.is_some() {
        available_reason
    } else {
        missing_reason
    });
    raw_line(",");
    raw_line("          \"authority\": \"retained_hash_reference_only\",");
    raw_line("          \"loads_artifact\": false,");
    raw_line("          \"allocates_service_slot\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"can_load_now\": false,");
    raw_line("          \"load_attempted\": false");
    raw("        }");
    if comma {
        raw(",");
    }
    crlf();
}

fn emit_module_loader_runtime_service_slot_allocator_readiness(
    candidate: ModuleLoaderRuntimeCandidate,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    raw_line("      \"service_slot_allocator_readiness\": {");
    raw_line("        \"schema\": \"raios.module_service_slot_allocator_readiness.v0\",");
    raw_line("        \"scope\": \"current_boot\",");
    raw_line("        \"classification\": \"local_only\",");
    raw_line("        \"source_method\": \"module.service_slot_allocator\",");
    raw_line("        \"state\": \"read_only_diagnostic_defined\",");
    raw("        \"retained_service_slot_reservation_present\": ");
    raw_bool(candidate.service_slot_reservation_present);
    raw_line(",");
    raw("        \"readiness_present\": ");
    raw_bool(candidate.service_slot_allocator_readiness_present);
    raw_line(",");
    raw("        \"readiness_status\": ");
    json_str(evaluation.service_slot_allocator_readiness_status);
    raw_line(",");
    raw("        \"readiness_reason\": ");
    json_str(evaluation.service_slot_allocator_readiness_reason);
    raw_line(",");
    raw("        \"runtime_status\": ");
    json_str(evaluation.service_slot_allocator_runtime_status);
    raw_line(",");
    raw("        \"runtime_reason\": ");
    json_str(evaluation.service_slot_allocator_runtime_reason);
    raw_line(",");
    raw("        \"service_slot_allocator_ready\": ");
    raw_bool(candidate.service_slot_allocator_ready);
    raw_line(",");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
}

fn emit_module_loader_runtime_facts(
    candidate: ModuleLoaderRuntimeCandidate,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    raw_line("      \"loader_runtime_facts\": {");
    emit_module_loader_runtime_fact(
        MODULE_LOADER_RUNTIME_FACT_SOURCES[0],
        candidate.loader_identity,
        evaluation.loader_identity_status,
        evaluation.loader_identity_reason,
        true,
    );
    emit_module_loader_runtime_fact(
        MODULE_LOADER_RUNTIME_FACT_SOURCES[1],
        candidate.artifact_hash_binding,
        evaluation.artifact_hash_binding_status,
        evaluation.artifact_hash_binding_reason,
        true,
    );
    emit_module_loader_runtime_fact(
        MODULE_LOADER_RUNTIME_FACT_SOURCES[2],
        candidate.entrypoint_abi,
        evaluation.entrypoint_abi_status,
        evaluation.entrypoint_abi_reason,
        true,
    );
    emit_module_loader_runtime_fact(
        MODULE_LOADER_RUNTIME_FACT_SOURCES[3],
        candidate.address_space_boundary,
        evaluation.address_space_boundary_status,
        evaluation.address_space_boundary_reason,
        true,
    );
    emit_module_loader_runtime_fact(
        MODULE_LOADER_RUNTIME_FACT_SOURCES[4],
        candidate.memory_map_constraints,
        evaluation.memory_map_constraints_status,
        evaluation.memory_map_constraints_reason,
        true,
    );
    emit_module_loader_runtime_fact(
        MODULE_LOADER_RUNTIME_FACT_SOURCES[5],
        candidate.capability_import_table,
        evaluation.capability_import_table_status,
        evaluation.capability_import_table_reason,
        true,
    );
    emit_module_loader_runtime_fact(
        MODULE_LOADER_RUNTIME_FACT_SOURCES[6],
        candidate.service_slot_binding,
        evaluation.service_slot_binding_status,
        evaluation.service_slot_binding_reason,
        true,
    );
    emit_module_loader_runtime_fact(
        MODULE_LOADER_RUNTIME_FACT_SOURCES[7],
        candidate.health_state_hooks,
        evaluation.health_state_hooks_status,
        evaluation.health_state_hooks_reason,
        true,
    );
    emit_module_loader_runtime_fact(
        MODULE_LOADER_RUNTIME_FACT_SOURCES[8],
        candidate.rollback_hooks,
        evaluation.rollback_hooks_status,
        evaluation.rollback_hooks_reason,
        true,
    );
    emit_module_loader_runtime_fact(
        MODULE_LOADER_RUNTIME_FACT_SOURCES[9],
        candidate.audit_rollback_write_boundary_binding,
        evaluation.audit_rollback_write_boundary_binding_status,
        evaluation.audit_rollback_write_boundary_binding_reason,
        false,
    );
    raw_line("      }");
}

fn emit_module_loader_runtime_fact(
    source: ModuleLoaderRuntimeFactSource,
    fact: ModuleLoaderRuntimeFact,
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
    if module_loader_runtime_fact_source_evidence_visible(source) {
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
    }
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
    raw("          \"binds_retained_module_evidence\": ");
    raw_bool(fact.binds_retained_module_evidence);
    raw_line(",");
    raw("          \"binds_service_slot_allocator\": ");
    raw_bool(fact.binds_service_slot_allocator);
    raw_line(",");
    raw("          \"binds_audit_rollback_write_boundary\": ");
    raw_bool(fact.binds_audit_rollback_write_boundary);
    raw_line(",");
    raw_line("          \"authority\": \"current_snapshot\",");
    raw_line("          \"persistence\": \"none\",");
    raw_line("          \"durable\": false,");
    raw_line("          \"loads_artifact\": false,");
    raw_line("          \"allocates_service_slot\": false,");
    raw_line("          \"creates_service_inventory_records\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"authorizes_load\": false,");
    raw_line("          \"required_bindings\": {");
    raw_line("            \"retained_module_evidence\": \"current_boot_hash_references\",");
    raw_line(
        "            \"service_slot_allocator_readiness\": \"raios.module_service_slot_allocator_readiness.v0\",",
    );
    raw_line(
        "            \"audit_write_boundary\": \"raios.module_audit_rollback_write_boundary.v0\",",
    );
    raw_line("            \"module_loader_runtime\": \"raios.module_loader_runtime_readiness.v0\"");
    raw_line("          },");
    raw_line("          \"provenance\": {");
    raw("            \"source_method\": ");
    json_str(source.source_method);
    raw_line(",");
    raw("            \"source_fact_locator\": ");
    json_str(source.source_fact_locator);
    raw_line(",");
    raw_line("            \"aggregate_method\": \"module.loader_runtime\",");
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

fn module_loader_runtime_fact_source_evidence_visible(
    source: ModuleLoaderRuntimeFactSource,
) -> bool {
    method_eq(source.name, "loader_identity") || method_eq(source.name, "artifact_hash_binding")
}

fn emit_module_loader_runtime_gate(
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

fn emit_module_loader_runtime_fact_gate(
    wrote: &mut bool,
    source: ModuleLoaderRuntimeFactSource,
    state: &'static str,
    reason: &'static str,
) {
    if *wrote {
        raw_line(",");
    } else {
        *wrote = true;
    }
    raw("        {\"gate\": ");
    json_str(source.name);
    raw(", \"state\": ");
    json_str(state);
    raw(", \"reason\": ");
    json_str(reason);
    raw(", \"schema\": ");
    json_str(source.schema);
    raw(", \"fact_id\": ");
    json_str(source.id);
    raw(", \"source_method\": ");
    json_str(source.source_method);
    raw(", \"source_fact_locator\": ");
    json_str(source.source_fact_locator);
    raw("}");
}

fn emit_module_loader_runtime_source_fact_map() {
    let mut idx = 0usize;
    while idx < MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
        let source = MODULE_LOADER_RUNTIME_FACT_SOURCES[idx];
        raw("        {\"fact\": ");
        json_str(source.name);
        raw(", \"schema\": ");
        json_str(source.schema);
        raw(", \"aggregate_fact_id\": ");
        json_str(source.id);
        raw(", \"source_method\": ");
        json_str(source.source_method);
        raw(", \"source_fact_locator\": ");
        json_str(source.source_fact_locator);
        raw(", \"source_evidence_schema\": ");
        json_str(source.source_evidence_schema);
        raw(", \"source_evidence_missing_reason\": ");
        json_str(source.source_evidence_missing_reason);
        raw(", \"addressable\": true, \"included_in_required_fact_list\": true}");
        if idx + 1 != MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
            raw(",");
        }
        crlf();
        idx += 1;
    }
}

fn emit_module_loader_runtime_selftest_case(case: &ModuleLoaderRuntimeSelfTestCase, comma: bool) {
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
    raw(", \"actual_loader_identity_source_evidence_present\": ");
    raw_bool(case.actual_loader_identity_source_evidence_present);
    raw(", \"actual_loader_identity_source_evidence_state\": ");
    json_str(case.actual_loader_identity_source_evidence_state);
    raw(", \"actual_loader_identity_source_evidence_status\": ");
    json_str(case.actual_loader_identity_source_evidence_status);
    raw(", \"actual_loader_identity_source_evidence_reason\": ");
    json_str(case.actual_loader_identity_source_evidence_reason);
    raw(", \"actual_artifact_hash_source_evidence_present\": ");
    raw_bool(case.actual_artifact_hash_source_evidence_present);
    raw(", \"actual_artifact_hash_source_evidence_state\": ");
    json_str(case.actual_artifact_hash_source_evidence_state);
    raw(", \"actual_artifact_hash_source_evidence_status\": ");
    json_str(case.actual_artifact_hash_source_evidence_status);
    raw(", \"actual_artifact_hash_source_evidence_reason\": ");
    json_str(case.actual_artifact_hash_source_evidence_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"loads_artifact\": false, \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn module_loader_runtime_snapshot(
    manifest_reference_present: bool,
    artifact_reference_present: bool,
    vm_report_reference_present: bool,
    local_attestation_reference_present: bool,
    local_approval_reference_present: bool,
    computed_grant_reference_present: bool,
    audit_rollback_reference_present: bool,
    service_slot_reservation_present: bool,
    loader_identity_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderIdentitySourceEvidence,
    )>,
    artifact_hash_binding_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderArtifactHashBindingSourceEvidence,
    )>,
) -> ModuleLoaderRuntimeCandidate {
    ModuleLoaderRuntimeCandidate {
        manifest_reference_present,
        artifact_reference_present,
        vm_report_reference_present,
        local_attestation_reference_present,
        local_approval_reference_present,
        computed_grant_reference_present,
        audit_rollback_reference_present,
        service_slot_reservation_present,
        service_slot_allocator_readiness_present: true,
        service_slot_allocator_ready: false,
        loader_identity: module_loader_runtime_loader_identity_fact(
            loader_identity_source_evidence,
        ),
        artifact_hash_binding: module_loader_runtime_artifact_hash_binding_fact(
            artifact_hash_binding_source_evidence,
        ),
        entrypoint_abi: module_loader_runtime_missing_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[2],
        ),
        address_space_boundary: module_loader_runtime_missing_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[3],
        ),
        memory_map_constraints: module_loader_runtime_missing_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[4],
        ),
        capability_import_table: module_loader_runtime_missing_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[5],
        ),
        service_slot_binding: module_loader_runtime_missing_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[6],
        ),
        health_state_hooks: module_loader_runtime_missing_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[7],
        ),
        rollback_hooks: module_loader_runtime_missing_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[8],
        ),
        audit_rollback_write_boundary_binding: module_loader_runtime_missing_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[9],
        ),
    }
}

fn module_loader_runtime_ready_snapshot() -> ModuleLoaderRuntimeCandidate {
    ModuleLoaderRuntimeCandidate {
        manifest_reference_present: true,
        artifact_reference_present: true,
        vm_report_reference_present: true,
        local_attestation_reference_present: true,
        local_approval_reference_present: true,
        computed_grant_reference_present: true,
        audit_rollback_reference_present: true,
        service_slot_reservation_present: true,
        service_slot_allocator_readiness_present: true,
        service_slot_allocator_ready: true,
        loader_identity: module_loader_runtime_available_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[0],
        ),
        artifact_hash_binding: module_loader_runtime_available_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[1],
        ),
        entrypoint_abi: module_loader_runtime_available_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[2],
        ),
        address_space_boundary: module_loader_runtime_available_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[3],
        ),
        memory_map_constraints: module_loader_runtime_available_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[4],
        ),
        capability_import_table: module_loader_runtime_available_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[5],
        ),
        service_slot_binding: module_loader_runtime_available_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[6],
        ),
        health_state_hooks: module_loader_runtime_available_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[7],
        ),
        rollback_hooks: module_loader_runtime_available_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[8],
        ),
        audit_rollback_write_boundary_binding: module_loader_runtime_available_fact_for(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[9],
        ),
    }
}

fn module_loader_runtime_loader_identity_fact(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderIdentitySourceEvidence,
    )>,
) -> ModuleLoaderRuntimeFact {
    let Some((event_id, evidence)) = source_evidence else {
        return module_loader_runtime_missing_fact_for(MODULE_LOADER_RUNTIME_FACT_SOURCES[0]);
    };

    ModuleLoaderRuntimeFact {
        present: evidence.identity_present,
        schema_ok: evidence.identity_schema_ok,
        scope: evidence.identity_scope,
        provenance_ok: evidence.identity_provenance_ok,
        classification: evidence.identity_classification,
        binds_retained_module_evidence: evidence.binds_retained_module_evidence,
        binds_service_slot_allocator: evidence.binds_service_slot_allocator,
        binds_audit_rollback_write_boundary: evidence.binds_audit_rollback_write_boundary,
        source_evidence_event_id: Some(event_id),
        source_evidence_schema: evidence.schema,
        source_evidence_state: if evidence.identity_present {
            "observed_current_boot_present"
        } else {
            "observed_current_boot_missing"
        },
        source_evidence_status: evidence.identity_status,
        source_evidence_reason: evidence.identity_reason,
        source_evidence_method: evidence.source_method,
        source_evidence_fact_locator: evidence.source_fact_locator,
    }
}

fn module_loader_runtime_artifact_hash_binding_fact(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderArtifactHashBindingSourceEvidence,
    )>,
) -> ModuleLoaderRuntimeFact {
    let Some((event_id, evidence)) = source_evidence else {
        return module_loader_runtime_missing_fact_for(MODULE_LOADER_RUNTIME_FACT_SOURCES[1]);
    };

    ModuleLoaderRuntimeFact {
        present: evidence.artifact_hash_binding_present,
        schema_ok: evidence.artifact_hash_binding_schema_ok,
        scope: evidence.artifact_hash_binding_scope,
        provenance_ok: evidence.artifact_hash_binding_provenance_ok,
        classification: evidence.artifact_hash_binding_classification,
        binds_retained_module_evidence: evidence.binds_retained_module_evidence,
        binds_service_slot_allocator: evidence.binds_service_slot_allocator,
        binds_audit_rollback_write_boundary: evidence.binds_audit_rollback_write_boundary,
        source_evidence_event_id: Some(event_id),
        source_evidence_schema: evidence.schema,
        source_evidence_state: if evidence.artifact_hash_binding_present {
            "observed_current_boot_present"
        } else {
            "observed_current_boot_missing"
        },
        source_evidence_status: evidence.artifact_hash_binding_status,
        source_evidence_reason: evidence.artifact_hash_binding_reason,
        source_evidence_method: evidence.source_method,
        source_evidence_fact_locator: evidence.source_fact_locator,
    }
}

fn module_loader_runtime_observed_loader_identity_missing_fact() -> ModuleLoaderRuntimeFact {
    ModuleLoaderRuntimeFact {
        source_evidence_event_id: Some(event_log::EventId { sequence: 42 }),
        source_evidence_state: "observed_current_boot_missing",
        source_evidence_status: "missing",
        source_evidence_reason: "module_loader_identity_missing",
        ..module_loader_runtime_missing_fact_for(MODULE_LOADER_RUNTIME_FACT_SOURCES[0])
    }
}

fn module_loader_runtime_observed_artifact_hash_binding_missing_fact() -> ModuleLoaderRuntimeFact {
    ModuleLoaderRuntimeFact {
        source_evidence_event_id: Some(event_log::EventId { sequence: 43 }),
        source_evidence_state: "observed_current_boot_missing",
        source_evidence_status: "missing",
        source_evidence_reason: "module_loader_artifact_hash_binding_missing",
        ..module_loader_runtime_missing_fact_for(MODULE_LOADER_RUNTIME_FACT_SOURCES[1])
    }
}

fn module_loader_runtime_missing_fact_for(
    source: ModuleLoaderRuntimeFactSource,
) -> ModuleLoaderRuntimeFact {
    ModuleLoaderRuntimeFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_retained_module_evidence: false,
        binds_service_slot_allocator: false,
        binds_audit_rollback_write_boundary: false,
        source_evidence_event_id: None,
        source_evidence_schema: source.source_evidence_schema,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: "missing",
        source_evidence_reason: source.source_evidence_missing_reason,
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
    }
}

fn module_loader_runtime_available_fact_for(
    source: ModuleLoaderRuntimeFactSource,
) -> ModuleLoaderRuntimeFact {
    ModuleLoaderRuntimeFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_retained_module_evidence: true,
        binds_service_slot_allocator: true,
        binds_audit_rollback_write_boundary: true,
        source_evidence_event_id: None,
        source_evidence_schema: source.source_evidence_schema,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: "available",
        source_evidence_reason: "module_loader_runtime_source_evidence_test_fixture_not_retained",
        source_evidence_method: source.source_method,
        source_evidence_fact_locator: source.source_fact_locator,
    }
}

fn evaluate_module_loader_runtime_candidate(
    candidate: ModuleLoaderRuntimeCandidate,
) -> ModuleLoaderRuntimeEvaluation {
    let (manifest_reference_status, manifest_reference_reason) =
        evaluate_module_loader_runtime_evidence(
            candidate.manifest_reference_present,
            "retained_module_manifest_reference_available",
            "retained_module_manifest_reference_missing",
        );
    let (artifact_reference_status, artifact_reference_reason) =
        evaluate_module_loader_runtime_evidence(
            candidate.artifact_reference_present,
            "retained_module_candidate_artifact_reference_available",
            "retained_module_candidate_artifact_reference_missing",
        );
    let (vm_report_reference_status, vm_report_reference_reason) =
        evaluate_module_loader_runtime_evidence(
            candidate.vm_report_reference_present,
            "retained_module_vm_test_report_reference_available",
            "retained_module_vm_test_report_reference_missing",
        );
    let (local_attestation_reference_status, local_attestation_reference_reason) =
        evaluate_module_loader_runtime_evidence(
            candidate.local_attestation_reference_present,
            "retained_module_local_attestation_reference_available",
            "retained_module_local_attestation_reference_missing",
        );
    let (local_approval_reference_status, local_approval_reference_reason) =
        evaluate_module_loader_runtime_evidence(
            candidate.local_approval_reference_present,
            "retained_module_local_approval_reference_available",
            "retained_module_local_approval_reference_missing",
        );
    let (computed_grant_reference_status, computed_grant_reference_reason) =
        evaluate_module_loader_runtime_evidence(
            candidate.computed_grant_reference_present,
            "retained_module_computed_grant_reference_available",
            "retained_module_computed_grant_reference_missing",
        );
    let (audit_rollback_reference_status, audit_rollback_reference_reason) =
        evaluate_module_loader_runtime_evidence(
            candidate.audit_rollback_reference_present,
            "retained_module_audit_rollback_reference_available",
            "retained_module_audit_rollback_reference_missing",
        );
    let (service_slot_reservation_status, service_slot_reservation_reason) =
        evaluate_module_loader_runtime_evidence(
            candidate.service_slot_reservation_present,
            "retained_module_service_slot_reservation_available",
            "retained_module_service_slot_reservation_missing",
        );
    let (service_slot_allocator_readiness_status, service_slot_allocator_readiness_reason) =
        evaluate_module_loader_runtime_evidence(
            candidate.service_slot_allocator_readiness_present,
            "service_slot_allocator_readiness_available",
            "service_slot_allocator_readiness_missing",
        );
    let (service_slot_allocator_runtime_status, service_slot_allocator_runtime_reason) =
        evaluate_module_loader_runtime_evidence(
            candidate.service_slot_allocator_ready,
            "service_slot_allocator_runtime_available",
            "service_slot_allocator_runtime_missing",
        );

    let (loader_identity_status, loader_identity_reason) = evaluate_module_loader_runtime_fact(
        candidate.loader_identity,
        "module_loader_identity_scope_must_be_current_boot",
        "module_loader_identity_schema_mismatch",
        "module_loader_identity_missing",
        "module_loader_identity_provenance_missing",
        "module_loader_identity_retained_evidence_binding_missing",
        "module_loader_identity_service_slot_allocator_binding_missing",
        "module_loader_identity_audit_write_boundary_binding_missing",
        "module_loader_identity_available",
    );
    let (artifact_hash_binding_status, artifact_hash_binding_reason) =
        evaluate_module_loader_runtime_fact(
            candidate.artifact_hash_binding,
            "module_loader_artifact_hash_binding_scope_must_be_current_boot",
            "module_loader_artifact_hash_binding_schema_mismatch",
            "module_loader_artifact_hash_binding_missing",
            "module_loader_artifact_hash_binding_provenance_missing",
            "module_loader_artifact_hash_binding_retained_evidence_binding_missing",
            "module_loader_artifact_hash_binding_service_slot_allocator_binding_missing",
            "module_loader_artifact_hash_binding_audit_write_boundary_binding_missing",
            "module_loader_artifact_hash_binding_available",
        );
    let (entrypoint_abi_status, entrypoint_abi_reason) = evaluate_module_loader_runtime_fact(
        candidate.entrypoint_abi,
        "module_loader_entrypoint_abi_scope_must_be_current_boot",
        "module_loader_entrypoint_abi_schema_mismatch",
        "module_loader_entrypoint_abi_missing",
        "module_loader_entrypoint_abi_provenance_missing",
        "module_loader_entrypoint_abi_retained_evidence_binding_missing",
        "module_loader_entrypoint_abi_service_slot_allocator_binding_missing",
        "module_loader_entrypoint_abi_audit_write_boundary_binding_missing",
        "module_loader_entrypoint_abi_available",
    );
    let (address_space_boundary_status, address_space_boundary_reason) =
        evaluate_module_loader_runtime_fact(
            candidate.address_space_boundary,
            "module_loader_address_space_boundary_scope_must_be_current_boot",
            "module_loader_address_space_boundary_schema_mismatch",
            "module_loader_address_space_boundary_missing",
            "module_loader_address_space_boundary_provenance_missing",
            "module_loader_address_space_boundary_retained_evidence_binding_missing",
            "module_loader_address_space_boundary_service_slot_allocator_binding_missing",
            "module_loader_address_space_boundary_audit_write_boundary_binding_missing",
            "module_loader_address_space_boundary_available",
        );
    let (memory_map_constraints_status, memory_map_constraints_reason) =
        evaluate_module_loader_runtime_fact(
            candidate.memory_map_constraints,
            "module_loader_memory_map_constraints_scope_must_be_current_boot",
            "module_loader_memory_map_constraints_schema_mismatch",
            "module_loader_memory_map_constraints_missing",
            "module_loader_memory_map_constraints_provenance_missing",
            "module_loader_memory_map_constraints_retained_evidence_binding_missing",
            "module_loader_memory_map_constraints_service_slot_allocator_binding_missing",
            "module_loader_memory_map_constraints_audit_write_boundary_binding_missing",
            "module_loader_memory_map_constraints_available",
        );
    let (capability_import_table_status, capability_import_table_reason) =
        evaluate_module_loader_runtime_fact(
            candidate.capability_import_table,
            "module_loader_capability_import_table_scope_must_be_current_boot",
            "module_loader_capability_import_table_schema_mismatch",
            "module_loader_capability_import_table_missing",
            "module_loader_capability_import_table_provenance_missing",
            "module_loader_capability_import_table_retained_evidence_binding_missing",
            "module_loader_capability_import_table_service_slot_allocator_binding_missing",
            "module_loader_capability_import_table_audit_write_boundary_binding_missing",
            "module_loader_capability_import_table_available",
        );
    let (service_slot_binding_status, service_slot_binding_reason) =
        evaluate_module_loader_runtime_fact(
            candidate.service_slot_binding,
            "module_loader_service_slot_binding_scope_must_be_current_boot",
            "module_loader_service_slot_binding_schema_mismatch",
            "module_loader_service_slot_binding_missing",
            "module_loader_service_slot_binding_provenance_missing",
            "module_loader_service_slot_binding_retained_evidence_binding_missing",
            "module_loader_service_slot_binding_service_slot_allocator_binding_missing",
            "module_loader_service_slot_binding_audit_write_boundary_binding_missing",
            "module_loader_service_slot_binding_available",
        );
    let (health_state_hooks_status, health_state_hooks_reason) =
        evaluate_module_loader_runtime_fact(
            candidate.health_state_hooks,
            "module_loader_health_state_hooks_scope_must_be_current_boot",
            "module_loader_health_state_hooks_schema_mismatch",
            "module_loader_health_state_hooks_missing",
            "module_loader_health_state_hooks_provenance_missing",
            "module_loader_health_state_hooks_retained_evidence_binding_missing",
            "module_loader_health_state_hooks_service_slot_allocator_binding_missing",
            "module_loader_health_state_hooks_audit_write_boundary_binding_missing",
            "module_loader_health_state_hooks_available",
        );
    let (rollback_hooks_status, rollback_hooks_reason) = evaluate_module_loader_runtime_fact(
        candidate.rollback_hooks,
        "module_loader_rollback_hooks_scope_must_be_current_boot",
        "module_loader_rollback_hooks_schema_mismatch",
        "module_loader_rollback_hooks_missing",
        "module_loader_rollback_hooks_provenance_missing",
        "module_loader_rollback_hooks_retained_evidence_binding_missing",
        "module_loader_rollback_hooks_service_slot_allocator_binding_missing",
        "module_loader_rollback_hooks_audit_write_boundary_binding_missing",
        "module_loader_rollback_hooks_available",
    );
    let (
        audit_rollback_write_boundary_binding_status,
        audit_rollback_write_boundary_binding_reason,
    ) = evaluate_module_loader_runtime_fact(
        candidate.audit_rollback_write_boundary_binding,
        "module_loader_audit_rollback_write_boundary_binding_scope_must_be_current_boot",
        "module_loader_audit_rollback_write_boundary_binding_schema_mismatch",
        "module_loader_audit_rollback_write_boundary_binding_missing",
        "module_loader_audit_rollback_write_boundary_binding_provenance_missing",
        "module_loader_audit_rollback_write_boundary_binding_retained_evidence_binding_missing",
        "module_loader_audit_rollback_write_boundary_binding_service_slot_allocator_binding_missing",
        "module_loader_audit_rollback_write_boundary_binding_audit_write_boundary_binding_missing",
        "module_loader_audit_rollback_write_boundary_binding_available",
    );

    let (status, reason) = if !candidate.manifest_reference_present {
        (
            "denied_missing_retained_module_evidence",
            manifest_reference_reason,
        )
    } else if !candidate.artifact_reference_present {
        (
            "denied_missing_retained_module_evidence",
            artifact_reference_reason,
        )
    } else if !candidate.vm_report_reference_present {
        (
            "denied_missing_retained_module_evidence",
            vm_report_reference_reason,
        )
    } else if !candidate.local_attestation_reference_present {
        (
            "denied_missing_retained_module_evidence",
            local_attestation_reference_reason,
        )
    } else if !candidate.local_approval_reference_present {
        (
            "denied_missing_retained_module_evidence",
            local_approval_reference_reason,
        )
    } else if !candidate.computed_grant_reference_present {
        (
            "denied_missing_retained_module_evidence",
            computed_grant_reference_reason,
        )
    } else if !candidate.audit_rollback_reference_present {
        (
            "denied_missing_retained_module_evidence",
            audit_rollback_reference_reason,
        )
    } else if !candidate.service_slot_reservation_present {
        (
            "denied_missing_retained_module_evidence",
            service_slot_reservation_reason,
        )
    } else if !candidate.service_slot_allocator_readiness_present {
        (
            "denied_missing_service_slot_allocator_readiness",
            service_slot_allocator_readiness_reason,
        )
    } else if !candidate.service_slot_allocator_ready {
        (
            "denied_missing_service_slot_allocator_runtime",
            service_slot_allocator_runtime_reason,
        )
    } else if method_eq(loader_identity_status, "rejected") {
        ("rejected", loader_identity_reason)
    } else if method_eq(loader_identity_status, "missing") {
        ("denied_missing_loader_runtime_fact", loader_identity_reason)
    } else if method_eq(artifact_hash_binding_status, "rejected") {
        ("rejected", artifact_hash_binding_reason)
    } else if method_eq(artifact_hash_binding_status, "missing") {
        (
            "denied_missing_loader_runtime_fact",
            artifact_hash_binding_reason,
        )
    } else if method_eq(entrypoint_abi_status, "rejected") {
        ("rejected", entrypoint_abi_reason)
    } else if method_eq(entrypoint_abi_status, "missing") {
        ("denied_missing_loader_runtime_fact", entrypoint_abi_reason)
    } else if method_eq(address_space_boundary_status, "rejected") {
        ("rejected", address_space_boundary_reason)
    } else if method_eq(address_space_boundary_status, "missing") {
        (
            "denied_missing_loader_runtime_fact",
            address_space_boundary_reason,
        )
    } else if method_eq(memory_map_constraints_status, "rejected") {
        ("rejected", memory_map_constraints_reason)
    } else if method_eq(memory_map_constraints_status, "missing") {
        (
            "denied_missing_loader_runtime_fact",
            memory_map_constraints_reason,
        )
    } else if method_eq(capability_import_table_status, "rejected") {
        ("rejected", capability_import_table_reason)
    } else if method_eq(capability_import_table_status, "missing") {
        (
            "denied_missing_loader_runtime_fact",
            capability_import_table_reason,
        )
    } else if method_eq(service_slot_binding_status, "rejected") {
        ("rejected", service_slot_binding_reason)
    } else if method_eq(service_slot_binding_status, "missing") {
        (
            "denied_missing_loader_runtime_fact",
            service_slot_binding_reason,
        )
    } else if method_eq(health_state_hooks_status, "rejected") {
        ("rejected", health_state_hooks_reason)
    } else if method_eq(health_state_hooks_status, "missing") {
        (
            "denied_missing_loader_runtime_fact",
            health_state_hooks_reason,
        )
    } else if method_eq(rollback_hooks_status, "rejected") {
        ("rejected", rollback_hooks_reason)
    } else if method_eq(rollback_hooks_status, "missing") {
        ("denied_missing_loader_runtime_fact", rollback_hooks_reason)
    } else if method_eq(audit_rollback_write_boundary_binding_status, "rejected") {
        ("rejected", audit_rollback_write_boundary_binding_reason)
    } else if method_eq(audit_rollback_write_boundary_binding_status, "missing") {
        (
            "denied_missing_loader_runtime_fact",
            audit_rollback_write_boundary_binding_reason,
        )
    } else {
        (
            "defined_non_executable",
            "module_loader_runtime_behavior_not_implemented",
        )
    };

    ModuleLoaderRuntimeEvaluation {
        status,
        reason,
        manifest_reference_status,
        manifest_reference_reason,
        artifact_reference_status,
        artifact_reference_reason,
        vm_report_reference_status,
        vm_report_reference_reason,
        local_attestation_reference_status,
        local_attestation_reference_reason,
        local_approval_reference_status,
        local_approval_reference_reason,
        computed_grant_reference_status,
        computed_grant_reference_reason,
        audit_rollback_reference_status,
        audit_rollback_reference_reason,
        service_slot_reservation_status,
        service_slot_reservation_reason,
        service_slot_allocator_readiness_status,
        service_slot_allocator_readiness_reason,
        service_slot_allocator_runtime_status,
        service_slot_allocator_runtime_reason,
        loader_identity_status,
        loader_identity_reason,
        artifact_hash_binding_status,
        artifact_hash_binding_reason,
        entrypoint_abi_status,
        entrypoint_abi_reason,
        address_space_boundary_status,
        address_space_boundary_reason,
        memory_map_constraints_status,
        memory_map_constraints_reason,
        capability_import_table_status,
        capability_import_table_reason,
        service_slot_binding_status,
        service_slot_binding_reason,
        health_state_hooks_status,
        health_state_hooks_reason,
        rollback_hooks_status,
        rollback_hooks_reason,
        audit_rollback_write_boundary_binding_status,
        audit_rollback_write_boundary_binding_reason,
        loads_artifact: false,
        allocates_service_slot: false,
        creates_service_inventory_records: false,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_loader_runtime_evidence(
    present: bool,
    available_reason: &'static str,
    missing_reason: &'static str,
) -> (&'static str, &'static str) {
    if present {
        ("available", available_reason)
    } else {
        ("missing", missing_reason)
    }
}

fn evaluate_module_loader_runtime_fact(
    fact: ModuleLoaderRuntimeFact,
    scope_reason: &'static str,
    schema_reason: &'static str,
    missing_reason: &'static str,
    provenance_reason: &'static str,
    retained_evidence_reason: &'static str,
    service_slot_allocator_reason: &'static str,
    audit_write_boundary_reason: &'static str,
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
    if !fact.binds_retained_module_evidence {
        return ("rejected", retained_evidence_reason);
    }
    if !fact.binds_service_slot_allocator {
        return ("rejected", service_slot_allocator_reason);
    }
    if !fact.binds_audit_rollback_write_boundary {
        return ("rejected", audit_write_boundary_reason);
    }
    ("available", available_reason)
}

fn module_loader_runtime_retained_evidence_complete(
    candidate: ModuleLoaderRuntimeCandidate,
) -> bool {
    candidate.manifest_reference_present
        && candidate.artifact_reference_present
        && candidate.vm_report_reference_present
        && candidate.local_attestation_reference_present
        && candidate.local_approval_reference_present
        && candidate.computed_grant_reference_present
        && candidate.audit_rollback_reference_present
        && candidate.service_slot_reservation_present
}

fn module_loader_runtime_facts_complete(evaluation: ModuleLoaderRuntimeEvaluation) -> bool {
    method_eq(evaluation.loader_identity_status, "available")
        && method_eq(evaluation.artifact_hash_binding_status, "available")
        && method_eq(evaluation.entrypoint_abi_status, "available")
        && method_eq(evaluation.address_space_boundary_status, "available")
        && method_eq(evaluation.memory_map_constraints_status, "available")
        && method_eq(evaluation.capability_import_table_status, "available")
        && method_eq(evaluation.service_slot_binding_status, "available")
        && method_eq(evaluation.health_state_hooks_status, "available")
        && method_eq(evaluation.rollback_hooks_status, "available")
        && method_eq(
            evaluation.audit_rollback_write_boundary_binding_status,
            "available",
        )
}

fn module_loader_runtime_selftest_cases(
) -> [ModuleLoaderRuntimeSelfTestCase; MODULE_LOADER_RUNTIME_SELFTEST_CASES] {
    let ready = module_loader_runtime_ready_snapshot();
    [
        module_loader_runtime_selftest_case(
            "missing_manifest_reference",
            "denied_missing_retained_module_evidence",
            "retained_module_manifest_reference_missing",
            ModuleLoaderRuntimeCandidate {
                manifest_reference_present: false,
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "missing_artifact_reference",
            "denied_missing_retained_module_evidence",
            "retained_module_candidate_artifact_reference_missing",
            ModuleLoaderRuntimeCandidate {
                artifact_reference_present: false,
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "missing_vm_report_reference",
            "denied_missing_retained_module_evidence",
            "retained_module_vm_test_report_reference_missing",
            ModuleLoaderRuntimeCandidate {
                vm_report_reference_present: false,
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "missing_local_attestation_reference",
            "denied_missing_retained_module_evidence",
            "retained_module_local_attestation_reference_missing",
            ModuleLoaderRuntimeCandidate {
                local_attestation_reference_present: false,
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "missing_local_approval_reference",
            "denied_missing_retained_module_evidence",
            "retained_module_local_approval_reference_missing",
            ModuleLoaderRuntimeCandidate {
                local_approval_reference_present: false,
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "missing_computed_grant_reference",
            "denied_missing_retained_module_evidence",
            "retained_module_computed_grant_reference_missing",
            ModuleLoaderRuntimeCandidate {
                computed_grant_reference_present: false,
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "missing_audit_rollback_reference",
            "denied_missing_retained_module_evidence",
            "retained_module_audit_rollback_reference_missing",
            ModuleLoaderRuntimeCandidate {
                audit_rollback_reference_present: false,
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "missing_service_slot_reservation",
            "denied_missing_retained_module_evidence",
            "retained_module_service_slot_reservation_missing",
            ModuleLoaderRuntimeCandidate {
                service_slot_reservation_present: false,
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "missing_service_slot_allocator_readiness",
            "denied_missing_service_slot_allocator_readiness",
            "service_slot_allocator_readiness_missing",
            ModuleLoaderRuntimeCandidate {
                service_slot_allocator_readiness_present: false,
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "service_slot_allocator_runtime_missing",
            "denied_missing_service_slot_allocator_runtime",
            "service_slot_allocator_runtime_missing",
            ModuleLoaderRuntimeCandidate {
                service_slot_allocator_ready: false,
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "loader_identity_previous_boot",
            "rejected",
            "module_loader_identity_scope_must_be_current_boot",
            ModuleLoaderRuntimeCandidate {
                loader_identity: ModuleLoaderRuntimeFact {
                    scope: "previous_boot",
                    ..ready.loader_identity
                },
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "loader_identity_wrong_schema",
            "rejected",
            "module_loader_identity_schema_mismatch",
            ModuleLoaderRuntimeCandidate {
                loader_identity: ModuleLoaderRuntimeFact {
                    schema_ok: false,
                    ..ready.loader_identity
                },
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "loader_identity_provenance_missing",
            "rejected",
            "module_loader_identity_provenance_missing",
            ModuleLoaderRuntimeCandidate {
                loader_identity: ModuleLoaderRuntimeFact {
                    provenance_ok: false,
                    ..ready.loader_identity
                },
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "loader_identity_retained_evidence_binding_missing",
            "rejected",
            "module_loader_identity_retained_evidence_binding_missing",
            ModuleLoaderRuntimeCandidate {
                loader_identity: ModuleLoaderRuntimeFact {
                    binds_retained_module_evidence: false,
                    ..ready.loader_identity
                },
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "loader_identity_service_slot_allocator_binding_missing",
            "rejected",
            "module_loader_identity_service_slot_allocator_binding_missing",
            ModuleLoaderRuntimeCandidate {
                loader_identity: ModuleLoaderRuntimeFact {
                    binds_service_slot_allocator: false,
                    ..ready.loader_identity
                },
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "loader_identity_audit_write_boundary_binding_missing",
            "rejected",
            "module_loader_identity_audit_write_boundary_binding_missing",
            ModuleLoaderRuntimeCandidate {
                loader_identity: ModuleLoaderRuntimeFact {
                    binds_audit_rollback_write_boundary: false,
                    ..ready.loader_identity
                },
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "loader_identity_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_identity_missing",
            ModuleLoaderRuntimeCandidate {
                loader_identity: module_loader_runtime_missing_fact_for(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[0],
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "loader_identity_observed_source_evidence_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_identity_missing",
            ModuleLoaderRuntimeCandidate {
                loader_identity: module_loader_runtime_observed_loader_identity_missing_fact(),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "artifact_hash_binding_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_artifact_hash_binding_missing",
            ModuleLoaderRuntimeCandidate {
                artifact_hash_binding: module_loader_runtime_missing_fact_for(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[1],
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "artifact_hash_binding_observed_source_evidence_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_artifact_hash_binding_missing",
            ModuleLoaderRuntimeCandidate {
                artifact_hash_binding:
                    module_loader_runtime_observed_artifact_hash_binding_missing_fact(),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "entrypoint_abi_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_entrypoint_abi_missing",
            ModuleLoaderRuntimeCandidate {
                entrypoint_abi: module_loader_runtime_missing_fact_for(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[2],
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "address_space_boundary_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_address_space_boundary_missing",
            ModuleLoaderRuntimeCandidate {
                address_space_boundary: module_loader_runtime_missing_fact_for(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[3],
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "memory_map_constraints_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_memory_map_constraints_missing",
            ModuleLoaderRuntimeCandidate {
                memory_map_constraints: module_loader_runtime_missing_fact_for(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[4],
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "capability_import_table_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_capability_import_table_missing",
            ModuleLoaderRuntimeCandidate {
                capability_import_table: module_loader_runtime_missing_fact_for(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[5],
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "service_slot_binding_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_service_slot_binding_missing",
            ModuleLoaderRuntimeCandidate {
                service_slot_binding: module_loader_runtime_missing_fact_for(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[6],
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "health_state_hooks_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_health_state_hooks_missing",
            ModuleLoaderRuntimeCandidate {
                health_state_hooks: module_loader_runtime_missing_fact_for(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[7],
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "rollback_hooks_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_rollback_hooks_missing",
            ModuleLoaderRuntimeCandidate {
                rollback_hooks: module_loader_runtime_missing_fact_for(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[8],
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "audit_rollback_write_boundary_binding_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_audit_rollback_write_boundary_binding_missing",
            ModuleLoaderRuntimeCandidate {
                audit_rollback_write_boundary_binding: module_loader_runtime_missing_fact_for(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[9],
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "all_inputs_ready_defined_non_executable",
            "defined_non_executable",
            "module_loader_runtime_behavior_not_implemented",
            ready,
        ),
    ]
}

fn module_loader_runtime_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoaderRuntimeCandidate,
) -> ModuleLoaderRuntimeSelfTestCase {
    let actual = evaluate_module_loader_runtime_candidate(candidate);
    ModuleLoaderRuntimeSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_loader_identity_source_evidence_present: candidate
            .loader_identity
            .source_evidence_event_id
            .is_some(),
        actual_loader_identity_source_evidence_state: candidate
            .loader_identity
            .source_evidence_state,
        actual_loader_identity_source_evidence_status: candidate
            .loader_identity
            .source_evidence_status,
        actual_loader_identity_source_evidence_reason: candidate
            .loader_identity
            .source_evidence_reason,
        actual_artifact_hash_source_evidence_present: candidate
            .artifact_hash_binding
            .source_evidence_event_id
            .is_some(),
        actual_artifact_hash_source_evidence_state: candidate
            .artifact_hash_binding
            .source_evidence_state,
        actual_artifact_hash_source_evidence_status: candidate
            .artifact_hash_binding
            .source_evidence_status,
        actual_artifact_hash_source_evidence_reason: candidate
            .artifact_hash_binding
            .source_evidence_reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.loads_artifact
            && !actual.allocates_service_slot
            && !actual.creates_service_inventory_records
            && !actual.can_load
            && !actual.load_attempted,
    }
}
