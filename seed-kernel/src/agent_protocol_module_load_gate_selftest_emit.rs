use alloc::{vec, vec::Vec};

use crate::{
    agent_protocol_module_load_gate_selftest::{
        module_load_gate_audit_rollback_selftest_cases,
        module_load_gate_loader_runtime_selftest_cases, module_load_gate_retained_selftest_cases,
        module_load_gate_service_slot_selftest_cases,
    },
    agent_protocol_module_load_gate_selftest_reference_cases::{
        module_load_gate_approval_selftest_cases, module_load_gate_artifact_selftest_cases,
        module_load_gate_attestation_selftest_cases, module_load_gate_manifest_selftest_cases,
        module_load_gate_vm_report_selftest_cases,
    },
    agent_protocol_module_types::*,
    agent_protocol_support::{
        begin_response, emit_record_fields_trailing_comma, emit_record_value_property_line,
        end_response, record_bool as b, record_false as no, record_field as f, record_str as s,
    },
};
use raios_core::record::Value as V;

pub(crate) fn emit_module_load_gate_manifest_selftest() {
    let cases = module_load_gate_manifest_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);
    let case_records = cases
        .iter()
        .map(module_load_gate_manifest_selftest_case_record)
        .collect();

    begin_response("module.load_gate_manifest_selftest");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.module_load_gate_manifest_selftest.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_retained_manifest_reference_records", no()),
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_unsigned_service_code", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("loader", s("unavailable")),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f(
                "required_bindings",
                static_str_array(&["manifest_reference_hash", "manifest_hash"]),
            ),
        ],
        6,
    );
    emit_record_value_property_line("cases", V::Array(case_records), true);
    emit_record_value_property_line("can_load", no(), false);
    end_response("module.load_gate_manifest_selftest");
}

fn module_load_gate_manifest_selftest_case_record(
    case: &ModuleLoadGateManifestSelfTestCase,
) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f(
            "actual_module_manifest_state",
            s(case.actual_module_manifest_state),
        ),
        f("accepted_manifest_hash", b(case.accepted_manifest_hash)),
        f("passed", b(case.passed)),
        f("can_load", no()),
        f("load_attempted", no()),
    ])
}

pub(crate) fn emit_module_load_gate_artifact_selftest() {
    let cases = module_load_gate_artifact_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);
    let case_records = cases
        .iter()
        .map(module_load_gate_artifact_selftest_case_record)
        .collect();

    begin_response("module.load_gate_artifact_selftest");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.module_load_gate_artifact_selftest.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f(
                "creates_retained_candidate_artifact_reference_records",
                no(),
            ),
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_unsigned_service_code", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("loader", s("unavailable")),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f(
                "required_bindings",
                static_str_array(&[
                    "artifact_reference_hash",
                    "retained_manifest_reference_event_id",
                    "retained_computed_grant_reference_event_id",
                    "manifest_reference_hash",
                    "manifest_hash",
                    "computed_capability_grant_hash",
                    "artifact_hash",
                    "vm_test_report_hash",
                    "local_attestation_hash",
                ]),
            ),
        ],
        6,
    );
    emit_record_value_property_line("cases", V::Array(case_records), true);
    emit_record_value_property_line("can_load", no(), false);
    end_response("module.load_gate_artifact_selftest");
}

fn module_load_gate_artifact_selftest_case_record(
    case: &ModuleLoadGateArtifactSelfTestCase,
) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f(
            "actual_candidate_artifact_state",
            s(case.actual_candidate_artifact_state),
        ),
        f("accepted_artifact_hash", b(case.accepted_artifact_hash)),
        f("passed", b(case.passed)),
        f("can_load", no()),
        f("load_attempted", no()),
    ])
}

pub(crate) fn emit_module_load_gate_vm_report_selftest() {
    let cases = module_load_gate_vm_report_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);
    let case_records = cases
        .iter()
        .map(module_load_gate_vm_report_selftest_case_record)
        .collect();

    begin_response("module.load_gate_vm_report_selftest");
    emit_record_fields_trailing_comma(
        vec![
            f("schema", s("raios.module_load_gate_vm_report_selftest.v0")),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_retained_vm_test_report_reference_records", no()),
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_vm_report_json", no()),
            f("accepts_unsigned_service_code", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("loader", s("unavailable")),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f(
                "required_bindings",
                static_str_array(&[
                    "vm_test_report_reference_hash",
                    "retained_manifest_reference_event_id",
                    "retained_candidate_artifact_reference_event_id",
                    "retained_computed_grant_reference_event_id",
                    "manifest_reference_hash",
                    "artifact_reference_hash",
                    "manifest_hash",
                    "artifact_hash",
                    "computed_capability_grant_hash",
                    "vm_test_report_hash",
                    "local_attestation_hash",
                ]),
            ),
        ],
        6,
    );
    emit_record_value_property_line("cases", V::Array(case_records), true);
    emit_record_value_property_line("can_load", no(), false);
    end_response("module.load_gate_vm_report_selftest");
}

pub(crate) fn emit_module_load_gate_attestation_selftest() {
    let cases = module_load_gate_attestation_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);
    let case_records = cases
        .iter()
        .map(module_load_gate_attestation_selftest_case_record)
        .collect();

    begin_response("module.load_gate_attestation_selftest");
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_load_gate_local_attestation_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_retained_local_attestation_reference_records", no()),
            f("accepts_local_attestation_json", no()),
            f("accepts_artifact_bytes", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
        ],
        6,
    );
    emit_record_value_property_line("cases", V::Array(case_records), true);
    emit_record_value_property_line("can_load", no(), false);
    end_response("module.load_gate_attestation_selftest");
}

pub(crate) fn emit_module_load_gate_approval_selftest() {
    let cases = module_load_gate_approval_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);
    let case_records = cases
        .iter()
        .map(module_load_gate_approval_selftest_case_record)
        .collect();

    begin_response("module.load_gate_approval_selftest");
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_load_gate_local_approval_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_retained_local_approval_reference_records", no()),
            f("accepts_local_approval_text", no()),
            f("accepts_artifact_bytes", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
        ],
        6,
    );
    emit_record_value_property_line("cases", V::Array(case_records), true);
    emit_record_value_property_line("can_load", no(), false);
    end_response("module.load_gate_approval_selftest");
}

fn module_load_gate_attestation_selftest_case_record(
    case: &ModuleLoadGateLocalAttestationSelfTestCase,
) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f(
            "actual_local_attestation_state",
            s(case.actual_local_attestation_state),
        ),
        f(
            "accepted_local_attestation_hash",
            b(case.accepted_local_attestation_hash),
        ),
        f("passed", b(case.passed)),
        f("can_load", no()),
        f("load_attempted", no()),
    ])
}

fn module_load_gate_approval_selftest_case_record(
    case: &ModuleLoadGateLocalApprovalSelfTestCase,
) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f(
            "actual_local_approval_state",
            s(case.actual_local_approval_state),
        ),
        f(
            "accepted_local_approval_hash",
            b(case.accepted_local_approval_hash),
        ),
        f("passed", b(case.passed)),
        f("can_load", no()),
        f("load_attempted", no()),
    ])
}

fn module_load_gate_vm_report_selftest_case_record(
    case: &ModuleLoadGateVmReportSelfTestCase,
) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f(
            "actual_vm_test_report_state",
            s(case.actual_vm_test_report_state),
        ),
        f(
            "accepted_vm_test_report_hash",
            b(case.accepted_vm_report_hash),
        ),
        f("passed", b(case.passed)),
        f("can_load", no()),
        f("load_attempted", no()),
    ])
}

pub(crate) fn emit_module_load_gate_retained_selftest() {
    let cases = module_load_gate_retained_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);
    let case_records = cases
        .iter()
        .map(module_load_gate_retained_selftest_case_record)
        .collect();

    begin_response("module.load_gate_retained_selftest");
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_load_gate_retained_reference_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_retained_reference_records", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("loader", s("unavailable")),
            f("service_slot", s("unallocated")),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
        ],
        6,
    );
    emit_record_value_property_line("cases", V::Array(case_records), true);
    emit_record_value_property_line("can_load", no(), false);
    end_response("module.load_gate_retained_selftest");
}

fn module_load_gate_retained_selftest_case_record(
    case: &ModuleLoadGateRetainedSelfTestCase,
) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f("passed", b(case.passed)),
        f("can_load", no()),
        f("load_attempted", no()),
    ])
}

pub(crate) fn emit_module_load_gate_audit_rollback_selftest() {
    let cases = module_load_gate_audit_rollback_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);
    let case_records = cases
        .iter()
        .map(module_load_gate_audit_rollback_selftest_case_record)
        .collect();

    begin_response("module.load_gate_audit_rollback_selftest");
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_load_gate_audit_rollback_selftest.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("test_infrastructure", b(true)),
            f("mutates_global_event_log", no()),
            f("creates_durable_audit_records", no()),
            f("creates_rollback_plans", no()),
            f("allocates_service_slot", no()),
            f("loads_artifact", no()),
            f("service_inventory_change", s("none")),
            f("load_attempted", no()),
            f("loader", s("unavailable")),
            f("service_slot", s("unallocated")),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f(
                "required_bindings",
                static_str_array(&[
                    "retained_computed_grant_reference_event_id",
                    "retained_audit_rollback_reference_event_id",
                    "audit_record_hash",
                    "computed_capability_grant_hash",
                    "manifest_hash",
                    "artifact_hash",
                    "vm_test_report_hash",
                    "local_attestation_hash",
                    "local_approval",
                    "rollback_plan_hash",
                    "ram_only_service_slot_id",
                ]),
            ),
        ],
        6,
    );
    emit_record_value_property_line("cases", V::Array(case_records), true);
    emit_record_value_property_line("can_load", no(), false);
    end_response("module.load_gate_audit_rollback_selftest");
}

fn module_load_gate_audit_rollback_selftest_case_record(
    case: &ModuleLoadGateAuditRollbackSelfTestCase,
) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f("passed", b(case.passed)),
        f("can_load", no()),
        f("load_attempted", no()),
    ])
}

pub(crate) fn emit_module_load_gate_service_slot_selftest() {
    let cases = module_load_gate_service_slot_selftest_cases();
    let passed = cases.iter().all(|case| case.passed);
    let case_records = cases
        .iter()
        .map(module_load_gate_service_slot_selftest_case_record)
        .collect();

    begin_response("module.load_gate_service_slot_selftest");
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_load_gate_service_slot_selftest.v0"),
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
            f("load_attempted", no()),
            f("loader", s("unavailable")),
            f("service_slot", s("non_authorizing")),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f(
                "required_bindings",
                static_str_array(&[
                    "retained_computed_grant_reference_event_id",
                    "retained_audit_rollback_reference_event_id",
                    "reservation_hash",
                    "computed_capability_grant_hash",
                    "audit_record_hash",
                    "rollback_plan_hash",
                    "pre_load_service_inventory_hash",
                    "ram_only_service_slot_id",
                ]),
            ),
        ],
        6,
    );
    emit_record_value_property_line("cases", V::Array(case_records), true);
    emit_record_value_property_line("can_load", no(), false);
    end_response("module.load_gate_service_slot_selftest");
}

fn module_load_gate_service_slot_selftest_case_record(
    case: &ModuleLoadGateServiceSlotSelfTestCase,
) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f(
            "actual_service_slot_state",
            s(case.actual_service_slot_state),
        ),
        f(
            "accepted_service_slot_reservation_hash",
            b(case.accepted_service_slot_reservation_hash),
        ),
        f("passed", b(case.passed)),
        f("allocates_service_slot", no()),
        f("can_load", no()),
        f("load_attempted", no()),
    ])
}

pub(crate) fn emit_module_load_gate_loader_runtime_selftest() {
    let cases = module_load_gate_loader_runtime_selftest_cases();
    let source_fact_map_complete = module_loader_runtime_source_fact_map_complete();
    let passed = source_fact_map_complete && cases.iter().all(|case| case.passed);
    let source_fact_records = MODULE_LOADER_RUNTIME_FACT_SOURCES
        .iter()
        .copied()
        .map(module_load_gate_loader_runtime_source_fact_record)
        .collect();
    let case_records = cases
        .iter()
        .map(module_load_gate_loader_runtime_selftest_case_record)
        .collect();

    begin_response("module.load_gate_loader_runtime_selftest");
    emit_record_fields_trailing_comma(
        vec![
            f(
                "schema",
                s("raios.module_load_gate_loader_runtime_selftest.v0"),
            ),
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
            f("load_attempted", no()),
            f("service_slot_allocator_ready", no()),
            f("loader", s("unavailable")),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f(
                "required_bindings",
                static_str_array(&[
                    "retained_module_evidence",
                    "raios.module_service_slot_allocator_readiness.v0",
                    "raios.module_service_slot_allocator_authority.v0",
                    "raios.service_slot_allocation_intent.v0",
                    "raios.service_slot_allocator_policy_decision.v0",
                    "raios.service_slot_registry_write_authority.v0",
                    "raios.module_loader_runtime_contract.v0",
                    "raios.service_health_monitor_binding.v0",
                    "raios.service_unload_cleanup_authority.v0",
                    "raios.module_service_slot_allocator_authority_decision.v0",
                    "raios.service_slot_registry_write_commit_gate.v0",
                    "raios.module_loader_runtime_execution_commit_gate.v0",
                    "raios.module_loader_descriptor_intake_boundary.v0",
                    "raios.module_loader_artifact_byte_intake_boundary.v0",
                    "raios.module_loader_identity.v0",
                    "raios.module_loader_artifact_hash_binding.v0",
                    "raios.module_loader_entrypoint_abi.v0",
                    "raios.module_loader_address_space_boundary.v0",
                    "raios.module_loader_memory_map_constraints.v0",
                    "raios.module_loader_capability_import_table.v0",
                    "raios.module_loader_service_slot_binding.v0",
                    "raios.module_loader_health_state_hooks.v0",
                    "raios.module_loader_rollback_hooks.v0",
                    "raios.module_loader_audit_rollback_write_boundary_binding.v0",
                ]),
            ),
            f(
                "missing_runtime_facts",
                static_str_array(&[
                    "raios.module_loader_identity.v0",
                    "raios.module_loader_artifact_hash_binding.v0",
                    "raios.module_loader_entrypoint_abi.v0",
                    "raios.module_loader_address_space_boundary.v0",
                    "raios.module_loader_memory_map_constraints.v0",
                    "raios.module_loader_capability_import_table.v0",
                    "raios.module_loader_service_slot_binding.v0",
                    "raios.module_loader_health_state_hooks.v0",
                    "raios.module_loader_rollback_hooks.v0",
                    "raios.module_loader_audit_rollback_write_boundary_binding.v0",
                ]),
            ),
            f(
                "source_fact_count",
                V::U64(MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT as u64),
            ),
            f("source_fact_map_complete", b(source_fact_map_complete)),
            f("source_fact_map", V::Array(source_fact_records)),
        ],
        6,
    );
    emit_record_value_property_line("cases", V::Array(case_records), true);
    emit_record_value_property_line("can_load", no(), false);
    end_response("module.load_gate_loader_runtime_selftest");
}

fn module_load_gate_loader_runtime_source_fact_record(
    source: ModuleLoaderRuntimeFactSource,
) -> V<'static> {
    V::InlineObject(vec![
        f("fact", s(source.name)),
        f("schema", s(source.schema)),
        f("id", s(source.id)),
        f("source_method", s(source.source_method)),
        f("source_fact_locator", s(source.source_fact_locator)),
        f("missing_reason", s(source.missing_reason)),
        f("status", s("missing")),
        f("present", no()),
        f("authorizes_load", no()),
    ])
}

fn module_load_gate_loader_runtime_selftest_case_record(
    case: &ModuleLoadGateLoaderRuntimeSelfTestCase,
) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f(
            "expected_retained_module_evidence_state",
            s(case.expected_retained_module_evidence_state),
        ),
        f(
            "expected_service_slot_allocator_state",
            s(case.expected_service_slot_allocator_state),
        ),
        f(
            "expected_loader_runtime_state",
            s(case.expected_loader_runtime_state),
        ),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f(
            "actual_retained_module_evidence_state",
            s(case.actual_retained_module_evidence_state),
        ),
        f(
            "actual_retained_module_evidence_reason",
            s(case.actual_retained_module_evidence_reason),
        ),
        f(
            "actual_service_slot_allocator_state",
            s(case.actual_service_slot_allocator_state),
        ),
        f(
            "actual_service_slot_allocator_status",
            s(case.actual_service_slot_allocator_status),
        ),
        f(
            "actual_service_slot_allocator_reason",
            s(case.actual_service_slot_allocator_reason),
        ),
        f(
            "actual_loader_runtime_state",
            s(case.actual_loader_runtime_state),
        ),
        f("passed", b(case.passed)),
        f("loads_artifact", no()),
        f("allocates_service_slot", no()),
        f("creates_service_inventory_records", no()),
        f("can_load", no()),
        f("load_attempted", no()),
    ])
}

fn static_str_array(values: &[&'static str]) -> V<'static> {
    let mut out = Vec::new();
    let mut idx = 0usize;
    while idx < values.len() {
        out.push(s(values[idx]));
        idx += 1;
    }
    V::Array(out)
}
