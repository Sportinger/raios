use crate::{
    agent_protocol_support::{
        begin_response, crlf, end_response, json_event_id_option, json_str, method_eq,
        method_head_eq, raw, raw_bool, raw_fmt, raw_line,
    },
    event_log,
};

const MODULE_LOADER_FACT_SELFTEST_CASES: usize = 14;

#[derive(Clone, Copy)]
struct ModuleLoaderFactSpec {
    method: &'static str,
    selftest_method: &'static str,
    schema: &'static str,
    selftest_schema: &'static str,
    field: &'static str,
    fact_id: &'static str,
    source_fact_locator: &'static str,
    source_evidence_schema: &'static str,
    scope_reason: &'static str,
    schema_reason: &'static str,
    missing_reason: &'static str,
    provenance_reason: &'static str,
    retained_binding_reason: &'static str,
    allocator_binding_reason: &'static str,
    audit_binding_reason: &'static str,
    available_reason: &'static str,
    non_authorizing_reason: &'static str,
    dependency_gate: &'static str,
    dependency_schema: &'static str,
    dependency_method: &'static str,
    dependency_missing_reason: &'static str,
    dependency_binding_reason: &'static str,
}

#[derive(Clone, Copy)]
struct ModuleLoaderFact {
    present: bool,
    schema_ok: bool,
    scope: &'static str,
    provenance_ok: bool,
    classification: &'static str,
    binds_retained_module_evidence: bool,
    binds_service_slot_allocator: bool,
    binds_audit_rollback_write_boundary: bool,
    binds_dependency: bool,
}

#[derive(Clone, Copy)]
struct ModuleLoaderFactCandidate {
    retained_module_evidence_present: bool,
    service_slot_allocator_readiness_present: bool,
    service_slot_allocator_ready: bool,
    audit_rollback_write_boundary_present: bool,
    dependency_present: bool,
    fact: ModuleLoaderFact,
}

#[derive(Clone, Copy)]
struct ModuleLoaderFactEvaluation {
    status: &'static str,
    reason: &'static str,
    retained_module_evidence_status: &'static str,
    retained_module_evidence_reason: &'static str,
    service_slot_allocator_readiness_status: &'static str,
    service_slot_allocator_readiness_reason: &'static str,
    service_slot_allocator_runtime_status: &'static str,
    service_slot_allocator_runtime_reason: &'static str,
    audit_rollback_write_boundary_status: &'static str,
    audit_rollback_write_boundary_reason: &'static str,
    dependency_status: &'static str,
    dependency_reason: &'static str,
    fact_status: &'static str,
    fact_reason: &'static str,
    loads_artifact: bool,
    allocates_service_slot: bool,
    creates_service_inventory_records: bool,
    can_load: bool,
    load_attempted: bool,
}

struct ModuleLoaderFactSelfTestCase {
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    actual_status: &'static str,
    actual_reason: &'static str,
    actual_fact_status: &'static str,
    actual_fact_reason: &'static str,
    passed: bool,
}

const MODULE_LOADER_FACT_SPECS: [ModuleLoaderFactSpec; 8] = [
    ModuleLoaderFactSpec {
        method: "module.loader_entrypoint_abi",
        selftest_method: "module.loader_entrypoint_abi_selftest",
        schema: "raios.module_loader_entrypoint_abi.v0",
        selftest_schema: "raios.module_loader_entrypoint_abi_selftest.v0",
        field: "entrypoint_abi",
        fact_id: "module.loader_runtime.entrypoint_abi.current_boot",
        source_fact_locator: "module.loader_entrypoint_abi.entrypoint_abi",
        source_evidence_schema: "raios.module_loader_entrypoint_abi_source_evidence.v0",
        scope_reason: "module_loader_entrypoint_abi_scope_must_be_current_boot",
        schema_reason: "module_loader_entrypoint_abi_schema_mismatch",
        missing_reason: "module_loader_entrypoint_abi_missing",
        provenance_reason: "module_loader_entrypoint_abi_provenance_missing",
        retained_binding_reason: "module_loader_entrypoint_abi_retained_evidence_binding_missing",
        allocator_binding_reason:
            "module_loader_entrypoint_abi_service_slot_allocator_binding_missing",
        audit_binding_reason: "module_loader_entrypoint_abi_audit_write_boundary_binding_missing",
        available_reason: "module_loader_entrypoint_abi_available",
        non_authorizing_reason: "module_loader_entrypoint_abi_not_load_authority",
        dependency_gate: "artifact_hash_binding",
        dependency_schema: "raios.module_loader_artifact_hash_binding.v0",
        dependency_method: "module.loader_artifact_hash_binding",
        dependency_missing_reason: "module_loader_artifact_hash_binding_missing",
        dependency_binding_reason:
            "module_loader_entrypoint_abi_artifact_hash_binding_binding_missing",
    },
    ModuleLoaderFactSpec {
        method: "module.loader_address_space_boundary",
        selftest_method: "module.loader_address_space_boundary_selftest",
        schema: "raios.module_loader_address_space_boundary.v0",
        selftest_schema: "raios.module_loader_address_space_boundary_selftest.v0",
        field: "address_space_boundary",
        fact_id: "module.loader_runtime.address_space_boundary.current_boot",
        source_fact_locator: "module.loader_address_space_boundary.address_space_boundary",
        source_evidence_schema: "raios.module_loader_address_space_boundary_source_evidence.v0",
        scope_reason: "module_loader_address_space_boundary_scope_must_be_current_boot",
        schema_reason: "module_loader_address_space_boundary_schema_mismatch",
        missing_reason: "module_loader_address_space_boundary_missing",
        provenance_reason: "module_loader_address_space_boundary_provenance_missing",
        retained_binding_reason:
            "module_loader_address_space_boundary_retained_evidence_binding_missing",
        allocator_binding_reason:
            "module_loader_address_space_boundary_service_slot_allocator_binding_missing",
        audit_binding_reason:
            "module_loader_address_space_boundary_audit_write_boundary_binding_missing",
        available_reason: "module_loader_address_space_boundary_available",
        non_authorizing_reason: "module_loader_address_space_boundary_not_load_authority",
        dependency_gate: "entrypoint_abi",
        dependency_schema: "raios.module_loader_entrypoint_abi.v0",
        dependency_method: "module.loader_entrypoint_abi",
        dependency_missing_reason: "module_loader_entrypoint_abi_missing",
        dependency_binding_reason:
            "module_loader_address_space_boundary_entrypoint_abi_binding_missing",
    },
    ModuleLoaderFactSpec {
        method: "module.loader_memory_map_constraints",
        selftest_method: "module.loader_memory_map_constraints_selftest",
        schema: "raios.module_loader_memory_map_constraints.v0",
        selftest_schema: "raios.module_loader_memory_map_constraints_selftest.v0",
        field: "memory_map_constraints",
        fact_id: "module.loader_runtime.memory_map_constraints.current_boot",
        source_fact_locator: "module.loader_memory_map_constraints.memory_map_constraints",
        source_evidence_schema:
            "raios.module_loader_memory_map_constraints_source_evidence.v0",
        scope_reason: "module_loader_memory_map_constraints_scope_must_be_current_boot",
        schema_reason: "module_loader_memory_map_constraints_schema_mismatch",
        missing_reason: "module_loader_memory_map_constraints_missing",
        provenance_reason: "module_loader_memory_map_constraints_provenance_missing",
        retained_binding_reason:
            "module_loader_memory_map_constraints_retained_evidence_binding_missing",
        allocator_binding_reason:
            "module_loader_memory_map_constraints_service_slot_allocator_binding_missing",
        audit_binding_reason:
            "module_loader_memory_map_constraints_audit_write_boundary_binding_missing",
        available_reason: "module_loader_memory_map_constraints_available",
        non_authorizing_reason: "module_loader_memory_map_constraints_not_load_authority",
        dependency_gate: "address_space_boundary",
        dependency_schema: "raios.module_loader_address_space_boundary.v0",
        dependency_method: "module.loader_address_space_boundary",
        dependency_missing_reason: "module_loader_address_space_boundary_missing",
        dependency_binding_reason:
            "module_loader_memory_map_constraints_address_space_boundary_binding_missing",
    },
    ModuleLoaderFactSpec {
        method: "module.loader_capability_import_table",
        selftest_method: "module.loader_capability_import_table_selftest",
        schema: "raios.module_loader_capability_import_table.v0",
        selftest_schema: "raios.module_loader_capability_import_table_selftest.v0",
        field: "capability_import_table",
        fact_id: "module.loader_runtime.capability_import_table.current_boot",
        source_fact_locator: "module.loader_capability_import_table.capability_import_table",
        source_evidence_schema:
            "raios.module_loader_capability_import_table_source_evidence.v0",
        scope_reason: "module_loader_capability_import_table_scope_must_be_current_boot",
        schema_reason: "module_loader_capability_import_table_schema_mismatch",
        missing_reason: "module_loader_capability_import_table_missing",
        provenance_reason: "module_loader_capability_import_table_provenance_missing",
        retained_binding_reason:
            "module_loader_capability_import_table_retained_evidence_binding_missing",
        allocator_binding_reason:
            "module_loader_capability_import_table_service_slot_allocator_binding_missing",
        audit_binding_reason:
            "module_loader_capability_import_table_audit_write_boundary_binding_missing",
        available_reason: "module_loader_capability_import_table_available",
        non_authorizing_reason: "module_loader_capability_import_table_not_load_authority",
        dependency_gate: "memory_map_constraints",
        dependency_schema: "raios.module_loader_memory_map_constraints.v0",
        dependency_method: "module.loader_memory_map_constraints",
        dependency_missing_reason: "module_loader_memory_map_constraints_missing",
        dependency_binding_reason:
            "module_loader_capability_import_table_memory_map_constraints_binding_missing",
    },
    ModuleLoaderFactSpec {
        method: "module.loader_service_slot_binding",
        selftest_method: "module.loader_service_slot_binding_selftest",
        schema: "raios.module_loader_service_slot_binding.v0",
        selftest_schema: "raios.module_loader_service_slot_binding_selftest.v0",
        field: "service_slot_binding",
        fact_id: "module.loader_runtime.service_slot_binding.current_boot",
        source_fact_locator: "module.loader_service_slot_binding.service_slot_binding",
        source_evidence_schema: "raios.module_loader_service_slot_binding_source_evidence.v0",
        scope_reason: "module_loader_service_slot_binding_scope_must_be_current_boot",
        schema_reason: "module_loader_service_slot_binding_schema_mismatch",
        missing_reason: "module_loader_service_slot_binding_missing",
        provenance_reason: "module_loader_service_slot_binding_provenance_missing",
        retained_binding_reason: "module_loader_service_slot_binding_retained_evidence_binding_missing",
        allocator_binding_reason:
            "module_loader_service_slot_binding_service_slot_allocator_binding_missing",
        audit_binding_reason:
            "module_loader_service_slot_binding_audit_write_boundary_binding_missing",
        available_reason: "module_loader_service_slot_binding_available",
        non_authorizing_reason: "module_loader_service_slot_binding_not_load_authority",
        dependency_gate: "capability_import_table",
        dependency_schema: "raios.module_loader_capability_import_table.v0",
        dependency_method: "module.loader_capability_import_table",
        dependency_missing_reason: "module_loader_capability_import_table_missing",
        dependency_binding_reason:
            "module_loader_service_slot_binding_capability_import_table_binding_missing",
    },
    ModuleLoaderFactSpec {
        method: "module.loader_health_state_hooks",
        selftest_method: "module.loader_health_state_hooks_selftest",
        schema: "raios.module_loader_health_state_hooks.v0",
        selftest_schema: "raios.module_loader_health_state_hooks_selftest.v0",
        field: "health_state_hooks",
        fact_id: "module.loader_runtime.health_state_hooks.current_boot",
        source_fact_locator: "module.loader_health_state_hooks.health_state_hooks",
        source_evidence_schema: "raios.module_loader_health_state_hooks_source_evidence.v0",
        scope_reason: "module_loader_health_state_hooks_scope_must_be_current_boot",
        schema_reason: "module_loader_health_state_hooks_schema_mismatch",
        missing_reason: "module_loader_health_state_hooks_missing",
        provenance_reason: "module_loader_health_state_hooks_provenance_missing",
        retained_binding_reason: "module_loader_health_state_hooks_retained_evidence_binding_missing",
        allocator_binding_reason:
            "module_loader_health_state_hooks_service_slot_allocator_binding_missing",
        audit_binding_reason:
            "module_loader_health_state_hooks_audit_write_boundary_binding_missing",
        available_reason: "module_loader_health_state_hooks_available",
        non_authorizing_reason: "module_loader_health_state_hooks_not_load_authority",
        dependency_gate: "service_slot_binding",
        dependency_schema: "raios.module_loader_service_slot_binding.v0",
        dependency_method: "module.loader_service_slot_binding",
        dependency_missing_reason: "module_loader_service_slot_binding_missing",
        dependency_binding_reason:
            "module_loader_health_state_hooks_service_slot_binding_binding_missing",
    },
    ModuleLoaderFactSpec {
        method: "module.loader_rollback_hooks",
        selftest_method: "module.loader_rollback_hooks_selftest",
        schema: "raios.module_loader_rollback_hooks.v0",
        selftest_schema: "raios.module_loader_rollback_hooks_selftest.v0",
        field: "rollback_hooks",
        fact_id: "module.loader_runtime.rollback_hooks.current_boot",
        source_fact_locator: "module.loader_rollback_hooks.rollback_hooks",
        source_evidence_schema: "raios.module_loader_rollback_hooks_source_evidence.v0",
        scope_reason: "module_loader_rollback_hooks_scope_must_be_current_boot",
        schema_reason: "module_loader_rollback_hooks_schema_mismatch",
        missing_reason: "module_loader_rollback_hooks_missing",
        provenance_reason: "module_loader_rollback_hooks_provenance_missing",
        retained_binding_reason: "module_loader_rollback_hooks_retained_evidence_binding_missing",
        allocator_binding_reason:
            "module_loader_rollback_hooks_service_slot_allocator_binding_missing",
        audit_binding_reason: "module_loader_rollback_hooks_audit_write_boundary_binding_missing",
        available_reason: "module_loader_rollback_hooks_available",
        non_authorizing_reason: "module_loader_rollback_hooks_not_load_authority",
        dependency_gate: "health_state_hooks",
        dependency_schema: "raios.module_loader_health_state_hooks.v0",
        dependency_method: "module.loader_health_state_hooks",
        dependency_missing_reason: "module_loader_health_state_hooks_missing",
        dependency_binding_reason: "module_loader_rollback_hooks_health_state_hooks_binding_missing",
    },
    ModuleLoaderFactSpec {
        method: "module.loader_audit_rollback_write_boundary_binding",
        selftest_method: "module.loader_audit_rollback_write_boundary_binding_selftest",
        schema: "raios.module_loader_audit_rollback_write_boundary_binding.v0",
        selftest_schema: "raios.module_loader_audit_rollback_write_boundary_binding_selftest.v0",
        field: "audit_rollback_write_boundary_binding",
        fact_id: "module.loader_runtime.audit_rollback_write_boundary_binding.current_boot",
        source_fact_locator:
            "module.loader_audit_rollback_write_boundary_binding.audit_rollback_write_boundary_binding",
        source_evidence_schema:
            "raios.module_loader_audit_rollback_write_boundary_binding_source_evidence.v0",
        scope_reason:
            "module_loader_audit_rollback_write_boundary_binding_scope_must_be_current_boot",
        schema_reason: "module_loader_audit_rollback_write_boundary_binding_schema_mismatch",
        missing_reason: "module_loader_audit_rollback_write_boundary_binding_missing",
        provenance_reason:
            "module_loader_audit_rollback_write_boundary_binding_provenance_missing",
        retained_binding_reason:
            "module_loader_audit_rollback_write_boundary_binding_retained_evidence_binding_missing",
        allocator_binding_reason:
            "module_loader_audit_rollback_write_boundary_binding_service_slot_allocator_binding_missing",
        audit_binding_reason:
            "module_loader_audit_rollback_write_boundary_binding_audit_write_boundary_binding_missing",
        available_reason: "module_loader_audit_rollback_write_boundary_binding_available",
        non_authorizing_reason:
            "module_loader_audit_rollback_write_boundary_binding_not_load_authority",
        dependency_gate: "rollback_hooks",
        dependency_schema: "raios.module_loader_rollback_hooks.v0",
        dependency_method: "module.loader_rollback_hooks",
        dependency_missing_reason: "module_loader_rollback_hooks_missing",
        dependency_binding_reason:
            "module_loader_audit_rollback_write_boundary_binding_rollback_hooks_binding_missing",
    },
];

pub(crate) fn module_loader_fact_method(method: &str) -> bool {
    module_loader_fact_spec(method).is_some()
}

pub(crate) fn module_loader_fact_selftest_method(method: &str) -> bool {
    module_loader_fact_selftest_spec(method).is_some()
}

pub(crate) fn canonical_module_loader_fact_method(method: &str) -> &'static str {
    module_loader_fact_spec(method)
        .map(|spec| spec.method)
        .unwrap_or("module.loader_entrypoint_abi")
}

pub(crate) fn canonical_module_loader_fact_selftest_method(method: &str) -> &'static str {
    module_loader_fact_selftest_spec(method)
        .map(|spec| spec.selftest_method)
        .unwrap_or("module.loader_entrypoint_abi_selftest")
}

pub(crate) fn emit_module_loader_fact(method: &str) {
    let Some(spec) = module_loader_fact_spec(method) else {
        return;
    };
    let retained_module_evidence_present = module_loader_fact_retained_module_evidence_present();
    let dependency_source_evidence_event_id =
        module_loader_fact_dependency_source_evidence_event_id(spec);
    let candidate = ModuleLoaderFactCandidate {
        retained_module_evidence_present,
        service_slot_allocator_readiness_present: true,
        service_slot_allocator_ready: false,
        audit_rollback_write_boundary_present: false,
        dependency_present: module_loader_fact_dependency_present(spec),
        fact: module_loader_fact_missing_fact(),
    };
    let evaluation = evaluate_module_loader_fact_candidate(spec, candidate);
    let source_evidence = if module_loader_fact_source_evidence_enabled(spec) {
        let evidence = module_loader_fact_source_evidence(
            spec,
            candidate,
            evaluation,
            dependency_source_evidence_event_id,
        );
        Some((
            event_log::record_module_loader_fact_source_evidence(evidence),
            evidence,
        ))
    } else {
        None
    };

    begin_response(spec.method);
    raw("      \"schema\": ");
    json_str(spec.schema);
    raw_line(",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(source_evidence.is_some());
    raw_line(",");
    if source_evidence.is_some() {
        raw_line(
            "      \"global_event_log_mutation\": \"retained_current_boot_source_evidence_only\",",
        );
    } else {
        raw_line("      \"global_event_log_mutation\": \"none\",");
    }
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"can_load_now\": false,");
    raw_line("      \"load_attempted\": false,");
    if let Some((event_id, evidence)) = source_evidence {
        emit_module_loader_fact_source_evidence(event_id, evidence);
        raw_line(",");
    }
    emit_module_loader_fact_required_bindings(spec, candidate, evaluation);
    raw_line(",");
    emit_module_loader_fact_object(
        spec,
        candidate.fact,
        evaluation,
        source_evidence.map(|(event_id, _)| event_id),
    );
    raw_line(",");
    emit_module_loader_fact_policy_result(candidate, evaluation);
    raw_line(",");
    raw_line("      \"blocked_by\": [");
    emit_module_loader_fact_blocked_by(spec, evaluation);
    crlf();
    raw_line("      ]");
    end_response(spec.method);
}

pub(crate) fn emit_module_loader_fact_selftest(method: &str) {
    let Some(spec) = module_loader_fact_selftest_spec(method) else {
        return;
    };
    let cases = module_loader_fact_selftest_cases(spec);
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response(spec.selftest_method);
    raw("      \"schema\": ");
    json_str(spec.selftest_schema);
    raw_line(",");
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
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_module_loader_fact_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_load\": false");
    end_response(spec.selftest_method);
}

fn module_loader_fact_spec(method: &str) -> Option<ModuleLoaderFactSpec> {
    let mut idx = 0usize;
    while idx < MODULE_LOADER_FACT_SPECS.len() {
        let spec = MODULE_LOADER_FACT_SPECS[idx];
        if method_head_eq(method, spec.method) {
            return Some(spec);
        }
        idx += 1;
    }
    None
}

fn module_loader_fact_selftest_spec(method: &str) -> Option<ModuleLoaderFactSpec> {
    let mut idx = 0usize;
    while idx < MODULE_LOADER_FACT_SPECS.len() {
        let spec = MODULE_LOADER_FACT_SPECS[idx];
        if method_head_eq(method, spec.selftest_method) {
            return Some(spec);
        }
        idx += 1;
    }
    None
}

fn module_loader_fact_retained_module_evidence_present() -> bool {
    event_log::latest_module_manifest_reference().is_some()
        && event_log::latest_module_candidate_artifact_reference().is_some()
        && event_log::latest_module_vm_test_report_reference().is_some()
        && event_log::latest_module_local_attestation_reference().is_some()
        && event_log::latest_module_local_approval_reference().is_some()
        && event_log::latest_module_computed_grant_reference().is_some()
        && event_log::latest_module_audit_rollback_reference().is_some()
        && event_log::latest_module_service_slot_reservation().is_some()
}

fn module_loader_fact_source_evidence_enabled(spec: ModuleLoaderFactSpec) -> bool {
    method_eq(spec.method, "module.loader_entrypoint_abi")
        || method_eq(spec.method, "module.loader_address_space_boundary")
        || method_eq(spec.method, "module.loader_memory_map_constraints")
        || method_eq(spec.method, "module.loader_capability_import_table")
        || method_eq(spec.method, "module.loader_service_slot_binding")
        || method_eq(spec.method, "module.loader_health_state_hooks")
        || method_eq(spec.method, "module.loader_rollback_hooks")
        || method_eq(
            spec.method,
            "module.loader_audit_rollback_write_boundary_binding",
        )
}

fn module_loader_fact_dependency_source_evidence_event_id(
    spec: ModuleLoaderFactSpec,
) -> Option<event_log::EventId> {
    if method_eq(spec.method, "module.loader_entrypoint_abi") {
        event_log::latest_module_loader_artifact_hash_binding_source_evidence()
            .map(|(event_id, _)| event_id)
    } else {
        event_log::latest_module_loader_fact_source_evidence(spec.dependency_method)
            .map(|(event_id, _)| event_id)
    }
}

fn module_loader_fact_dependency_present(spec: ModuleLoaderFactSpec) -> bool {
    if method_eq(spec.method, "module.loader_entrypoint_abi") {
        event_log::latest_module_loader_artifact_hash_binding_source_evidence()
            .map(|(_, evidence)| {
                evidence.artifact_hash_binding_present
                    && method_eq(evidence.artifact_hash_binding_status, "available")
            })
            .unwrap_or(false)
    } else {
        event_log::latest_module_loader_fact_source_evidence(spec.dependency_method)
            .map(|(_, evidence)| {
                evidence.fact_present && method_eq(evidence.fact_status, "available")
            })
            .unwrap_or(false)
    }
}

fn emit_module_loader_fact_required_bindings(
    spec: ModuleLoaderFactSpec,
    candidate: ModuleLoaderFactCandidate,
    evaluation: ModuleLoaderFactEvaluation,
) {
    raw_line("      \"required_bindings\": {");
    raw("        \"retained_module_evidence\": ");
    json_str(evaluation.retained_module_evidence_status);
    raw_line(",");
    raw("        \"service_slot_allocator_readiness\": ");
    json_str(evaluation.service_slot_allocator_readiness_status);
    raw_line(",");
    raw("        \"service_slot_allocator_runtime\": ");
    json_str(evaluation.service_slot_allocator_runtime_status);
    raw_line(",");
    raw("        \"audit_rollback_write_boundary\": ");
    json_str(evaluation.audit_rollback_write_boundary_status);
    raw_line(",");
    json_str(spec.dependency_gate);
    raw(": ");
    json_str(evaluation.dependency_status);
    raw_line(",");
    raw("        \"fact_present\": ");
    raw_bool(candidate.fact.present);
    raw_line(",");
    raw("        \"dependency_schema\": ");
    json_str(spec.dependency_schema);
    raw_line(",");
    raw("        \"dependency_method\": ");
    json_str(spec.dependency_method);
    crlf();
    raw_line("      }");
}

fn emit_module_loader_fact_source_evidence(
    event_id: event_log::EventId,
    evidence: event_log::ModuleLoaderFactSourceEvidence,
) {
    raw_line("      \"source_evidence\": {");
    raw("        \"schema\": ");
    json_str(evidence.schema);
    raw_line(",");
    raw_line("        \"state\": \"retained\",");
    raw_line("        \"status\": \"retained_current_boot_source_evidence\",");
    raw_line("        \"reason\": \"module_loader_fact_source_evidence_recorded\",");
    raw_line("        \"scope\": \"current_boot\",");
    raw_line("        \"classification\": \"local_only\",");
    raw_line("        \"retention\": \"current_boot_ram_event_log\",");
    raw("        \"event_id\": ");
    json_event_id_option(Some(event_id));
    raw_line(",");
    raw("        \"fact_schema\": ");
    json_str(evidence.fact_schema);
    raw_line(",");
    raw("        \"fact_id\": ");
    json_str(evidence.fact_id);
    raw_line(",");
    raw("        \"source_method\": ");
    json_str(evidence.source_method);
    raw_line(",");
    raw("        \"source_fact_locator\": ");
    json_str(evidence.source_fact_locator);
    raw_line(",");
    raw("        \"readiness_status\": ");
    json_str(evidence.readiness_status);
    raw_line(",");
    raw("        \"readiness_reason\": ");
    json_str(evidence.readiness_reason);
    raw_line(",");
    raw("        \"fact_status\": ");
    json_str(evidence.fact_status);
    raw_line(",");
    raw("        \"fact_reason\": ");
    json_str(evidence.fact_reason);
    raw_line(",");
    raw("        \"fact_present\": ");
    raw_bool(evidence.fact_present);
    raw_line(",");
    raw("        \"dependency_gate\": ");
    json_str(evidence.dependency_gate);
    raw_line(",");
    raw("        \"dependency_schema\": ");
    json_str(evidence.dependency_schema);
    raw_line(",");
    raw("        \"dependency_method\": ");
    json_str(evidence.dependency_method);
    raw_line(",");
    raw("        \"dependency_present\": ");
    raw_bool(evidence.dependency_present);
    raw_line(",");
    raw("        \"dependency_source_evidence_event_id\": ");
    json_event_id_option(evidence.dependency_source_evidence_event_id);
    raw_line(",");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false,");
    raw_line("        \"authorizes_load\": false");
    raw_line("      }");
}

fn emit_module_loader_fact_object(
    spec: ModuleLoaderFactSpec,
    fact: ModuleLoaderFact,
    evaluation: ModuleLoaderFactEvaluation,
    source_evidence_event_id: Option<event_log::EventId>,
) {
    raw("      ");
    json_str(spec.field);
    raw_line(": {");
    raw("        \"schema\": ");
    json_str(spec.schema);
    raw_line(",");
    raw("        \"state\": ");
    json_str(if fact.present { "present" } else { "missing" });
    raw_line(",");
    raw("        \"status\": ");
    json_str(evaluation.fact_status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(evaluation.fact_reason);
    raw_line(",");
    raw_line("        \"scope\": \"current_boot\",");
    raw("        \"fact_scope\": ");
    json_str(fact.scope);
    raw_line(",");
    raw("        \"schema_valid\": ");
    raw_bool(fact.schema_ok);
    raw_line(",");
    raw("        \"classification\": ");
    json_str(fact.classification);
    raw_line(",");
    raw("        \"provenance_valid\": ");
    raw_bool(fact.provenance_ok);
    raw_line(",");
    raw("        \"binds_retained_module_evidence\": ");
    raw_bool(fact.binds_retained_module_evidence);
    raw_line(",");
    raw("        \"binds_service_slot_allocator\": ");
    raw_bool(fact.binds_service_slot_allocator);
    raw_line(",");
    raw("        \"binds_audit_rollback_write_boundary\": ");
    raw_bool(fact.binds_audit_rollback_write_boundary);
    raw_line(",");
    raw("        \"binds_");
    raw(spec.dependency_gate);
    raw("\": ");
    raw_bool(fact.binds_dependency);
    raw_line(",");
    raw("        \"fact_id\": ");
    json_str(spec.fact_id);
    raw_line(",");
    raw("        \"source_method\": ");
    json_str(spec.method);
    raw_line(",");
    raw("        \"source_fact_locator\": ");
    json_str(spec.source_fact_locator);
    raw_line(",");
    if source_evidence_event_id.is_some() {
        raw("        \"source_evidence_event_id\": ");
        json_event_id_option(source_evidence_event_id);
        raw_line(",");
        raw("        \"source_evidence_schema\": ");
        json_str(spec.source_evidence_schema);
        raw_line(",");
        raw_line("        \"source_evidence_state\": \"retained_current_boot\",");
    }
    raw_line("        \"persistence\": \"none\",");
    raw_line("        \"durable\": false,");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"authorizes_load\": false");
    raw_line("      }");
}

fn emit_module_loader_fact_policy_result(
    candidate: ModuleLoaderFactCandidate,
    evaluation: ModuleLoaderFactEvaluation,
) {
    raw_line("      \"policy_result\": {");
    raw("        \"readiness_status\": ");
    json_str(evaluation.status);
    raw_line(",");
    raw("        \"readiness_reason\": ");
    json_str(evaluation.reason);
    raw_line(",");
    raw("        \"retained_module_evidence_present\": ");
    raw_bool(candidate.retained_module_evidence_present);
    raw_line(",");
    raw("        \"service_slot_allocator_readiness_present\": ");
    raw_bool(candidate.service_slot_allocator_readiness_present);
    raw_line(",");
    raw("        \"service_slot_allocator_ready\": ");
    raw_bool(candidate.service_slot_allocator_ready);
    raw_line(",");
    raw("        \"audit_rollback_write_boundary_present\": ");
    raw_bool(candidate.audit_rollback_write_boundary_present);
    raw_line(",");
    raw("        \"dependency_present\": ");
    raw_bool(candidate.dependency_present);
    raw_line(",");
    raw("        \"fact_available\": ");
    raw_bool(method_eq(evaluation.fact_status, "available"));
    raw_line(",");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"can_load_now\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
}

fn emit_module_loader_fact_blocked_by(
    spec: ModuleLoaderFactSpec,
    evaluation: ModuleLoaderFactEvaluation,
) {
    let mut wrote = false;
    emit_module_loader_fact_gate(
        &mut wrote,
        "retained_module_evidence",
        evaluation.retained_module_evidence_status,
        evaluation.retained_module_evidence_reason,
    );
    emit_module_loader_fact_gate(
        &mut wrote,
        "service_slot_allocator_readiness",
        evaluation.service_slot_allocator_readiness_status,
        evaluation.service_slot_allocator_readiness_reason,
    );
    emit_module_loader_fact_gate(
        &mut wrote,
        "service_slot_allocator_runtime",
        evaluation.service_slot_allocator_runtime_status,
        evaluation.service_slot_allocator_runtime_reason,
    );
    emit_module_loader_fact_gate(
        &mut wrote,
        "audit_rollback_write_boundary",
        evaluation.audit_rollback_write_boundary_status,
        evaluation.audit_rollback_write_boundary_reason,
    );
    emit_module_loader_fact_gate(
        &mut wrote,
        spec.dependency_gate,
        evaluation.dependency_status,
        evaluation.dependency_reason,
    );
    emit_module_loader_fact_gate(
        &mut wrote,
        spec.field,
        evaluation.fact_status,
        evaluation.fact_reason,
    );
}

fn emit_module_loader_fact_gate(
    wrote: &mut bool,
    gate: &'static str,
    state: &'static str,
    reason: &'static str,
) {
    if method_eq(state, "available") {
        return;
    }
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

fn emit_module_loader_fact_selftest_case(case: &ModuleLoaderFactSelfTestCase, comma: bool) {
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
    raw(", \"actual_fact_status\": ");
    json_str(case.actual_fact_status);
    raw(", \"actual_fact_reason\": ");
    json_str(case.actual_fact_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"loads_artifact\": false, \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn evaluate_module_loader_fact_candidate(
    spec: ModuleLoaderFactSpec,
    candidate: ModuleLoaderFactCandidate,
) -> ModuleLoaderFactEvaluation {
    let (retained_module_evidence_status, retained_module_evidence_reason) =
        if candidate.retained_module_evidence_present {
            ("available", "retained_module_evidence_available")
        } else {
            ("missing", "retained_module_evidence_missing")
        };
    let (service_slot_allocator_readiness_status, service_slot_allocator_readiness_reason) =
        if candidate.service_slot_allocator_readiness_present {
            ("available", "service_slot_allocator_readiness_available")
        } else {
            ("missing", "service_slot_allocator_readiness_missing")
        };
    let (service_slot_allocator_runtime_status, service_slot_allocator_runtime_reason) =
        if candidate.service_slot_allocator_ready {
            ("available", "service_slot_allocator_runtime_available")
        } else {
            ("missing", "service_slot_allocator_runtime_missing")
        };
    let (audit_rollback_write_boundary_status, audit_rollback_write_boundary_reason) =
        if candidate.audit_rollback_write_boundary_present {
            (
                "available",
                "module_audit_rollback_write_boundary_binding_available",
            )
        } else {
            (
                "missing",
                "module_audit_rollback_write_boundary_binding_missing",
            )
        };
    let (dependency_status, dependency_reason) = if candidate.dependency_present {
        ("available", spec.available_reason)
    } else {
        ("missing", spec.dependency_missing_reason)
    };
    let (fact_status, fact_reason) = evaluate_module_loader_fact(spec, candidate.fact);

    let (status, reason) = if !candidate.retained_module_evidence_present {
        (
            "denied_missing_retained_module_evidence",
            retained_module_evidence_reason,
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
    } else if !candidate.audit_rollback_write_boundary_present {
        (
            "denied_missing_audit_rollback_write_boundary",
            audit_rollback_write_boundary_reason,
        )
    } else if !candidate.dependency_present {
        ("denied_missing_loader_fact_dependency", dependency_reason)
    } else if method_eq(fact_status, "rejected") {
        ("rejected", fact_reason)
    } else if method_eq(fact_status, "missing") {
        ("denied_missing_loader_runtime_fact", fact_reason)
    } else {
        ("available_non_authorizing", spec.non_authorizing_reason)
    };

    ModuleLoaderFactEvaluation {
        status,
        reason,
        retained_module_evidence_status,
        retained_module_evidence_reason,
        service_slot_allocator_readiness_status,
        service_slot_allocator_readiness_reason,
        service_slot_allocator_runtime_status,
        service_slot_allocator_runtime_reason,
        audit_rollback_write_boundary_status,
        audit_rollback_write_boundary_reason,
        dependency_status,
        dependency_reason,
        fact_status,
        fact_reason,
        loads_artifact: false,
        allocates_service_slot: false,
        creates_service_inventory_records: false,
        can_load: false,
        load_attempted: false,
    }
}

fn evaluate_module_loader_fact(
    spec: ModuleLoaderFactSpec,
    fact: ModuleLoaderFact,
) -> (&'static str, &'static str) {
    if !method_eq(fact.scope, "current_boot") {
        return ("rejected", spec.scope_reason);
    }
    if !fact.schema_ok {
        return ("rejected", spec.schema_reason);
    }
    if !fact.present {
        return ("missing", spec.missing_reason);
    }
    if !fact.provenance_ok {
        return ("rejected", spec.provenance_reason);
    }
    if !fact.binds_retained_module_evidence {
        return ("rejected", spec.retained_binding_reason);
    }
    if !fact.binds_service_slot_allocator {
        return ("rejected", spec.allocator_binding_reason);
    }
    if !fact.binds_audit_rollback_write_boundary {
        return ("rejected", spec.audit_binding_reason);
    }
    if !fact.binds_dependency {
        return ("rejected", spec.dependency_binding_reason);
    }
    ("available", spec.available_reason)
}

fn module_loader_fact_source_evidence(
    spec: ModuleLoaderFactSpec,
    candidate: ModuleLoaderFactCandidate,
    evaluation: ModuleLoaderFactEvaluation,
    dependency_source_evidence_event_id: Option<event_log::EventId>,
) -> event_log::ModuleLoaderFactSourceEvidence {
    event_log::ModuleLoaderFactSourceEvidence {
        schema: spec.source_evidence_schema,
        fact_schema: spec.schema,
        fact_id: spec.fact_id,
        source_method: spec.method,
        source_fact_locator: spec.source_fact_locator,
        readiness_status: evaluation.status,
        readiness_reason: evaluation.reason,
        fact_status: evaluation.fact_status,
        fact_reason: evaluation.fact_reason,
        fact_present: candidate.fact.present,
        fact_scope: candidate.fact.scope,
        fact_schema_ok: candidate.fact.schema_ok,
        fact_provenance_ok: candidate.fact.provenance_ok,
        fact_classification: candidate.fact.classification,
        retained_module_evidence_present: candidate.retained_module_evidence_present,
        service_slot_allocator_readiness_present: candidate
            .service_slot_allocator_readiness_present,
        service_slot_allocator_ready: candidate.service_slot_allocator_ready,
        audit_rollback_write_boundary_present: candidate.audit_rollback_write_boundary_present,
        dependency_present: candidate.dependency_present,
        dependency_gate: spec.dependency_gate,
        dependency_schema: spec.dependency_schema,
        dependency_method: spec.dependency_method,
        dependency_source_evidence_event_id,
        binds_retained_module_evidence: candidate.fact.binds_retained_module_evidence,
        binds_service_slot_allocator: candidate.fact.binds_service_slot_allocator,
        binds_audit_rollback_write_boundary: candidate.fact.binds_audit_rollback_write_boundary,
        binds_dependency: candidate.fact.binds_dependency,
    }
}

fn module_loader_fact_selftest_cases(
    spec: ModuleLoaderFactSpec,
) -> [ModuleLoaderFactSelfTestCase; MODULE_LOADER_FACT_SELFTEST_CASES] {
    let ready = module_loader_fact_ready_candidate();
    let missing_fact = module_loader_fact_missing_fact();
    [
        module_loader_fact_selftest_case(
            spec,
            "missing_retained_module_evidence",
            "denied_missing_retained_module_evidence",
            "retained_module_evidence_missing",
            ModuleLoaderFactCandidate {
                retained_module_evidence_present: false,
                ..ready
            },
        ),
        module_loader_fact_selftest_case(
            spec,
            "missing_service_slot_allocator_readiness",
            "denied_missing_service_slot_allocator_readiness",
            "service_slot_allocator_readiness_missing",
            ModuleLoaderFactCandidate {
                service_slot_allocator_readiness_present: false,
                ..ready
            },
        ),
        module_loader_fact_selftest_case(
            spec,
            "service_slot_allocator_runtime_missing",
            "denied_missing_service_slot_allocator_runtime",
            "service_slot_allocator_runtime_missing",
            ModuleLoaderFactCandidate {
                service_slot_allocator_ready: false,
                ..ready
            },
        ),
        module_loader_fact_selftest_case(
            spec,
            "audit_write_boundary_missing",
            "denied_missing_audit_rollback_write_boundary",
            "module_audit_rollback_write_boundary_binding_missing",
            ModuleLoaderFactCandidate {
                audit_rollback_write_boundary_present: false,
                ..ready
            },
        ),
        module_loader_fact_selftest_case(
            spec,
            "previous_loader_fact_missing",
            "denied_missing_loader_fact_dependency",
            spec.dependency_missing_reason,
            ModuleLoaderFactCandidate {
                dependency_present: false,
                ..ready
            },
        ),
        module_loader_fact_selftest_case(
            spec,
            "loader_fact_previous_boot",
            "rejected",
            spec.scope_reason,
            ModuleLoaderFactCandidate {
                fact: ModuleLoaderFact {
                    scope: "previous_boot",
                    ..ready.fact
                },
                ..ready
            },
        ),
        module_loader_fact_selftest_case(
            spec,
            "loader_fact_schema_mismatch",
            "rejected",
            spec.schema_reason,
            ModuleLoaderFactCandidate {
                fact: ModuleLoaderFact {
                    schema_ok: false,
                    ..ready.fact
                },
                ..ready
            },
        ),
        module_loader_fact_selftest_case(
            spec,
            "loader_fact_provenance_missing",
            "rejected",
            spec.provenance_reason,
            ModuleLoaderFactCandidate {
                fact: ModuleLoaderFact {
                    provenance_ok: false,
                    ..ready.fact
                },
                ..ready
            },
        ),
        module_loader_fact_selftest_case(
            spec,
            "loader_fact_retained_evidence_binding_missing",
            "rejected",
            spec.retained_binding_reason,
            ModuleLoaderFactCandidate {
                fact: ModuleLoaderFact {
                    binds_retained_module_evidence: false,
                    ..ready.fact
                },
                ..ready
            },
        ),
        module_loader_fact_selftest_case(
            spec,
            "loader_fact_service_slot_allocator_binding_missing",
            "rejected",
            spec.allocator_binding_reason,
            ModuleLoaderFactCandidate {
                fact: ModuleLoaderFact {
                    binds_service_slot_allocator: false,
                    ..ready.fact
                },
                ..ready
            },
        ),
        module_loader_fact_selftest_case(
            spec,
            "loader_fact_audit_write_boundary_binding_missing",
            "rejected",
            spec.audit_binding_reason,
            ModuleLoaderFactCandidate {
                fact: ModuleLoaderFact {
                    binds_audit_rollback_write_boundary: false,
                    ..ready.fact
                },
                ..ready
            },
        ),
        module_loader_fact_selftest_case(
            spec,
            "loader_fact_previous_loader_fact_binding_missing",
            "rejected",
            spec.dependency_binding_reason,
            ModuleLoaderFactCandidate {
                fact: ModuleLoaderFact {
                    binds_dependency: false,
                    ..ready.fact
                },
                ..ready
            },
        ),
        module_loader_fact_selftest_case(
            spec,
            "loader_fact_missing",
            "denied_missing_loader_runtime_fact",
            spec.missing_reason,
            ModuleLoaderFactCandidate {
                fact: missing_fact,
                ..ready
            },
        ),
        module_loader_fact_selftest_case(
            spec,
            "all_inputs_present_fact_non_authorizing",
            "available_non_authorizing",
            spec.non_authorizing_reason,
            ready,
        ),
    ]
}

fn module_loader_fact_selftest_case(
    spec: ModuleLoaderFactSpec,
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    candidate: ModuleLoaderFactCandidate,
) -> ModuleLoaderFactSelfTestCase {
    let actual = evaluate_module_loader_fact_candidate(spec, candidate);
    ModuleLoaderFactSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: actual.status,
        actual_reason: actual.reason,
        actual_fact_status: actual.fact_status,
        actual_fact_reason: actual.fact_reason,
        passed: method_eq(actual.status, expected_status)
            && method_eq(actual.reason, expected_reason)
            && !actual.loads_artifact
            && !actual.allocates_service_slot
            && !actual.creates_service_inventory_records
            && !actual.can_load
            && !actual.load_attempted,
    }
}

fn module_loader_fact_ready_candidate() -> ModuleLoaderFactCandidate {
    ModuleLoaderFactCandidate {
        retained_module_evidence_present: true,
        service_slot_allocator_readiness_present: true,
        service_slot_allocator_ready: true,
        audit_rollback_write_boundary_present: true,
        dependency_present: true,
        fact: module_loader_fact_available_fact(),
    }
}

fn module_loader_fact_missing_fact() -> ModuleLoaderFact {
    ModuleLoaderFact {
        present: false,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: false,
        classification: "local_only",
        binds_retained_module_evidence: false,
        binds_service_slot_allocator: false,
        binds_audit_rollback_write_boundary: false,
        binds_dependency: false,
    }
}

fn module_loader_fact_available_fact() -> ModuleLoaderFact {
    ModuleLoaderFact {
        present: true,
        schema_ok: true,
        scope: "current_boot",
        provenance_ok: true,
        classification: "local_only",
        binds_retained_module_evidence: true,
        binds_service_slot_allocator: true,
        binds_audit_rollback_write_boundary: true,
        binds_dependency: true,
    }
}
