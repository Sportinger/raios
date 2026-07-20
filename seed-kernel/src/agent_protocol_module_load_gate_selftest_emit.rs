use alloc::vec;

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
    agent_protocol_module_reference::{emit_evidence_v1_response, selftest_case, selftest_facts},
    agent_protocol_module_types::*,
    agent_protocol_support::{
        record_bool as b, record_false as no, record_field as f, record_str as s,
    },
};
use raios_core::{evidence_response as ev, record::Value as V};

macro_rules! emit_selftest {
    ($function:ident, $method:literal, $family:literal, $cases:ident, $case_type:ty, $extras:expr) => {
        pub(crate) fn $function() {
            let cases = $cases();
            let passed = cases.iter().all(|case| case.passed);
            let records = cases
                .iter()
                .map(|case: &$case_type| {
                    selftest_case(
                        case.name,
                        case.expected_status,
                        case.expected_reason,
                        case.actual_status,
                        case.actual_reason,
                        case.passed,
                    )
                })
                .collect();
            let mut facts = selftest_facts(V::Array(records), cases.len(), passed);
            if let V::InlineObject(fields) = &mut facts {
                fields.extend($extras);
            }
            emit_evidence_v1_response(
                $method,
                $family,
                None,
                facts,
                vec![],
                ev::observed("selftest_completed"),
            );
        }
    };
}

emit_selftest!(
    emit_module_load_gate_manifest_selftest,
    "module.load_gate_manifest_selftest",
    "module.load_gate.selftest",
    module_load_gate_manifest_selftest_cases,
    ModuleLoadGateManifestSelfTestCase,
    vec![f(
        "required_bindings",
        strings(&["manifest_reference_hash", "manifest_hash"])
    )]
);
emit_selftest!(
    emit_module_load_gate_artifact_selftest,
    "module.load_gate_artifact_selftest",
    "module.load_gate.selftest",
    module_load_gate_artifact_selftest_cases,
    ModuleLoadGateArtifactSelfTestCase,
    vec![f(
        "required_bindings",
        strings(&[
            "artifact_reference_hash",
            "retained_manifest_reference_event_id",
            "retained_computed_grant_reference_event_id",
            "manifest_reference_hash",
            "manifest_hash",
            "computed_capability_grant_hash",
            "artifact_hash",
            "vm_test_report_hash",
            "local_attestation_hash",
        ])
    )]
);
emit_selftest!(
    emit_module_load_gate_vm_report_selftest,
    "module.load_gate_vm_report_selftest",
    "module.load_gate.selftest",
    module_load_gate_vm_report_selftest_cases,
    ModuleLoadGateVmReportSelfTestCase,
    vec![f(
        "required_bindings",
        strings(&[
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
        ])
    )]
);
emit_selftest!(
    emit_module_load_gate_attestation_selftest,
    "module.load_gate_attestation_selftest",
    "module.load_gate.selftest",
    module_load_gate_attestation_selftest_cases,
    ModuleLoadGateLocalAttestationSelfTestCase,
    vec![]
);
emit_selftest!(
    emit_module_load_gate_approval_selftest,
    "module.load_gate_approval_selftest",
    "module.load_gate.selftest",
    module_load_gate_approval_selftest_cases,
    ModuleLoadGateLocalApprovalSelfTestCase,
    vec![]
);
emit_selftest!(
    emit_module_load_gate_retained_selftest,
    "module.load_gate_retained_selftest",
    "module.load_gate.selftest",
    module_load_gate_retained_selftest_cases,
    ModuleLoadGateRetainedSelfTestCase,
    vec![]
);
emit_selftest!(
    emit_module_load_gate_audit_rollback_selftest,
    "module.load_gate_audit_rollback_selftest",
    "module.load_gate.selftest",
    module_load_gate_audit_rollback_selftest_cases,
    ModuleLoadGateAuditRollbackSelfTestCase,
    vec![f(
        "required_bindings",
        strings(&[
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
        ])
    )]
);
emit_selftest!(
    emit_module_load_gate_service_slot_selftest,
    "module.load_gate_service_slot_selftest",
    "module.load_gate.selftest",
    module_load_gate_service_slot_selftest_cases,
    ModuleLoadGateServiceSlotSelfTestCase,
    vec![f(
        "required_bindings",
        strings(&[
            "retained_computed_grant_reference_event_id",
            "retained_audit_rollback_reference_event_id",
            "reservation_hash",
            "computed_capability_grant_hash",
            "audit_record_hash",
            "rollback_plan_hash",
            "pre_load_service_inventory_hash",
            "ram_only_service_slot_id",
        ])
    )]
);
emit_selftest!(
    emit_module_load_gate_loader_runtime_selftest,
    "module.load_gate_loader_runtime_selftest",
    "module.load_gate.selftest",
    module_load_gate_loader_runtime_selftest_cases,
    ModuleLoadGateLoaderRuntimeSelfTestCase,
    loader_runtime_facts()
);

fn strings(values: &[&'static str]) -> V<'static> {
    V::Array(values.iter().copied().map(s).collect())
}

fn loader_runtime_facts() -> alloc::vec::Vec<raios_core::record::Field<'static>> {
    let source_fact_map_complete = module_loader_runtime_source_fact_map_complete();
    let source_fact_map = MODULE_LOADER_RUNTIME_FACT_SOURCES
        .iter()
        .map(|source| {
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
        })
        .collect();
    vec![
        f(
            "required_bindings",
            strings(&[
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
            strings(&[
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
        f("source_fact_map", V::Array(source_fact_map)),
    ]
}
