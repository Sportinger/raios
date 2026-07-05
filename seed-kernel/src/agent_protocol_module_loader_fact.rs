use alloc::vec;

use crate::{
    agent_protocol_module_service_slot_allocator_projection::latest_module_service_slot_allocator_readiness_projection,
    agent_protocol_support::{
        begin_response, crlf, emit_inline_record_object, emit_inline_record_object_fragment,
        emit_record_fields_trailing_comma, emit_record_property_line, end_response, method_eq,
        method_head_eq, raw_line, record_bool as b, record_event_or_null, record_false as no,
        record_field as f, record_str as s,
    },
    event_log,
};
use raios_core::record::Value as V;

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
    service_slot_allocator_unready_status: &'static str,
    service_slot_allocator_unready_reason: &'static str,
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

pub(crate) fn emit_module_loader_fact(method: &str) {
    let Some(spec) = module_loader_fact_spec(method) else {
        return;
    };
    let retained_module_evidence_present = module_loader_fact_retained_module_evidence_present();
    let service_slot = event_log::latest_module_service_slot_reservation();
    let service_slot_allocator = latest_module_service_slot_allocator_readiness_projection(
        service_slot.as_ref().map(|(event_id, _)| *event_id),
    );
    let dependency_source_evidence_event_id =
        module_loader_fact_dependency_source_evidence_event_id(spec);
    let candidate = ModuleLoaderFactCandidate {
        retained_module_evidence_present,
        service_slot_allocator_readiness_present: service_slot_allocator.readiness_present,
        service_slot_allocator_ready: service_slot_allocator.ready,
        service_slot_allocator_unready_status: service_slot_allocator.unready_status,
        service_slot_allocator_unready_reason: service_slot_allocator.unready_reason,
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
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s(spec.schema)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", no()),
            f("mutates_global_event_log", b(source_evidence.is_some())),
            f(
                "global_event_log_mutation",
                s(if source_evidence.is_some() {
                    "retained_current_boot_source_evidence_only"
                } else {
                    "none"
                }),
            ),
            f("accepts_loader_descriptor", no()),
            f("accepts_artifact_bytes", no()),
            f("loads_artifact", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        6,
    );
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
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s(spec.selftest_schema)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("accepts_loader_descriptor", no()),
            f("accepts_artifact_bytes", no()),
            f("loads_artifact", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
        ],
        6,
    );
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

fn module_loader_fact_dependency_bind_field(spec: ModuleLoaderFactSpec) -> &'static str {
    if method_eq(spec.dependency_gate, "artifact_hash_binding") {
        "binds_artifact_hash_binding"
    } else if method_eq(spec.dependency_gate, "entrypoint_abi") {
        "binds_entrypoint_abi"
    } else if method_eq(spec.dependency_gate, "address_space_boundary") {
        "binds_address_space_boundary"
    } else if method_eq(spec.dependency_gate, "memory_map_constraints") {
        "binds_memory_map_constraints"
    } else if method_eq(spec.dependency_gate, "capability_import_table") {
        "binds_capability_import_table"
    } else if method_eq(spec.dependency_gate, "service_slot_binding") {
        "binds_service_slot_binding"
    } else if method_eq(spec.dependency_gate, "health_state_hooks") {
        "binds_health_state_hooks"
    } else {
        "binds_rollback_hooks"
    }
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
    emit_record_property_line(
        "required_bindings",
        vec![
            f(
                "retained_module_evidence",
                s(evaluation.retained_module_evidence_status),
            ),
            f(
                "service_slot_allocator_readiness",
                s(evaluation.service_slot_allocator_readiness_status),
            ),
            f(
                "service_slot_allocator_runtime",
                s(evaluation.service_slot_allocator_runtime_status),
            ),
            f(
                "audit_rollback_write_boundary",
                s(evaluation.audit_rollback_write_boundary_status),
            ),
            f(spec.dependency_gate, s(evaluation.dependency_status)),
            f("fact_present", b(candidate.fact.present)),
            f("dependency_schema", s(spec.dependency_schema)),
            f("dependency_method", s(spec.dependency_method)),
        ],
        false,
    );
}

fn emit_module_loader_fact_source_evidence(
    event_id: event_log::EventId,
    evidence: event_log::ModuleLoaderFactSourceEvidence,
) {
    emit_record_property_line(
        "source_evidence",
        vec![
            f("schema", s(evidence.schema)),
            f("state", s("retained")),
            f("status", s("retained_current_boot_source_evidence")),
            f("reason", s("module_loader_fact_source_evidence_recorded")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("retention", s("current_boot_ram_event_log")),
            f("event_id", record_event_or_null(Some(event_id))),
            f("fact_schema", s(evidence.fact_schema)),
            f("fact_id", s(evidence.fact_id)),
            f("source_method", s(evidence.source_method)),
            f("source_fact_locator", s(evidence.source_fact_locator)),
            f("readiness_status", s(evidence.readiness_status)),
            f("readiness_reason", s(evidence.readiness_reason)),
            f("fact_status", s(evidence.fact_status)),
            f("fact_reason", s(evidence.fact_reason)),
            f("fact_present", b(evidence.fact_present)),
            f("dependency_gate", s(evidence.dependency_gate)),
            f("dependency_schema", s(evidence.dependency_schema)),
            f("dependency_method", s(evidence.dependency_method)),
            f("dependency_present", b(evidence.dependency_present)),
            f(
                "dependency_source_evidence_event_id",
                record_event_or_null(evidence.dependency_source_evidence_event_id),
            ),
            f("accepts_loader_descriptor", no()),
            f("accepts_artifact_bytes", no()),
            f("loads_artifact", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
            f("authorizes_load", no()),
        ],
        false,
    );
}

fn emit_module_loader_fact_object(
    spec: ModuleLoaderFactSpec,
    fact: ModuleLoaderFact,
    evaluation: ModuleLoaderFactEvaluation,
    source_evidence_event_id: Option<event_log::EventId>,
) {
    let mut fields = vec![
        f("schema", s(spec.schema)),
        f("state", s(if fact.present { "present" } else { "missing" })),
        f("status", s(evaluation.fact_status)),
        f("reason", s(evaluation.fact_reason)),
        f("scope", s("current_boot")),
        f("fact_scope", s(fact.scope)),
        f("schema_valid", b(fact.schema_ok)),
        f("classification", s(fact.classification)),
        f("provenance_valid", b(fact.provenance_ok)),
        f(
            "binds_retained_module_evidence",
            b(fact.binds_retained_module_evidence),
        ),
        f(
            "binds_service_slot_allocator",
            b(fact.binds_service_slot_allocator),
        ),
        f(
            "binds_audit_rollback_write_boundary",
            b(fact.binds_audit_rollback_write_boundary),
        ),
        f(
            module_loader_fact_dependency_bind_field(spec),
            b(fact.binds_dependency),
        ),
        f("fact_id", s(spec.fact_id)),
        f("source_method", s(spec.method)),
        f("source_fact_locator", s(spec.source_fact_locator)),
    ];
    if source_evidence_event_id.is_some() {
        fields.push(f(
            "source_evidence_event_id",
            record_event_or_null(source_evidence_event_id),
        ));
        fields.push(f("source_evidence_schema", s(spec.source_evidence_schema)));
        fields.push(f("source_evidence_state", s("retained_current_boot")));
    }
    fields.push(f("persistence", s("none")));
    fields.push(f("durable", no()));
    fields.push(f("loads_artifact", no()));
    fields.push(f("allocates_service_slot", no()));
    fields.push(f("creates_service_inventory_records", no()));
    fields.push(f("service_inventory_change", s("none")));
    fields.push(f("authorizes_load", no()));
    emit_record_property_line(spec.field, fields, false);
}

fn emit_module_loader_fact_policy_result(
    candidate: ModuleLoaderFactCandidate,
    evaluation: ModuleLoaderFactEvaluation,
) {
    emit_record_property_line(
        "policy_result",
        vec![
            f("readiness_status", s(evaluation.status)),
            f("readiness_reason", s(evaluation.reason)),
            f(
                "retained_module_evidence_present",
                b(candidate.retained_module_evidence_present),
            ),
            f(
                "service_slot_allocator_readiness_present",
                b(candidate.service_slot_allocator_readiness_present),
            ),
            f(
                "service_slot_allocator_ready",
                b(candidate.service_slot_allocator_ready),
            ),
            f(
                "audit_rollback_write_boundary_present",
                b(candidate.audit_rollback_write_boundary_present),
            ),
            f("dependency_present", b(candidate.dependency_present)),
            f(
                "fact_available",
                b(method_eq(evaluation.fact_status, "available")),
            ),
            f("loads_artifact", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        false,
    );
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
    emit_inline_record_object_fragment(
        vec![
            f("gate", s(gate)),
            f("state", s(state)),
            f("reason", s(reason)),
        ],
        8,
    );
}

fn emit_module_loader_fact_selftest_case(case: &ModuleLoaderFactSelfTestCase, comma: bool) {
    emit_inline_record_object(
        vec![
            f("case", s(case.name)),
            f("expected_status", s(case.expected_status)),
            f("expected_reason", s(case.expected_reason)),
            f("actual_status", s(case.actual_status)),
            f("actual_reason", s(case.actual_reason)),
            f("actual_fact_status", s(case.actual_fact_status)),
            f("actual_fact_reason", s(case.actual_fact_reason)),
            f("passed", b(case.passed)),
            f("loads_artifact", no()),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("can_load", no()),
            f("load_attempted", no()),
        ],
        comma,
    );
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
        if !candidate.service_slot_allocator_readiness_present {
            ("missing", "service_slot_allocator_readiness_missing")
        } else if candidate.service_slot_allocator_ready {
            ("available", "service_slot_allocator_readiness_available")
        } else {
            (
                candidate.service_slot_allocator_unready_status,
                candidate.service_slot_allocator_unready_reason,
            )
        };
    let (service_slot_allocator_runtime_status, service_slot_allocator_runtime_reason) =
        if candidate.service_slot_allocator_ready {
            ("available", "service_slot_allocator_runtime_available")
        } else if method_eq(
            candidate.service_slot_allocator_unready_status,
            "denied_missing_service_slot_allocator_runtime",
        ) {
            ("missing", candidate.service_slot_allocator_unready_reason)
        } else {
            ("available", "service_slot_allocator_runtime_available")
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
            candidate.service_slot_allocator_unready_status,
            candidate.service_slot_allocator_unready_reason,
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
                service_slot_allocator_unready_status:
                    "denied_missing_service_slot_allocator_runtime",
                service_slot_allocator_unready_reason: "service_slot_allocator_runtime_missing",
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
        service_slot_allocator_unready_status: "available",
        service_slot_allocator_unready_reason: "service_slot_allocator_runtime_available",
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
