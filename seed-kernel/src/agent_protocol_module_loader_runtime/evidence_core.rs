use super::evidence_live_load::{
    module_loader_live_load_boundary_source_evidence_record,
    module_loader_retained_module_evidence_present,
};
use crate::agent_protocol_module_types::{
    ModuleLoaderRuntimeSelfTestCase, MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_ID,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_REASON,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SCHEMA,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_STATUS, MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_ID,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_REASON, MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SCHEMA,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_STATUS, MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_ID,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_STATUS, MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_ID,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_REASON,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SCHEMA,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_ID,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_STATUS, MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_ID,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_MISSING_STATUS, MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_REASON,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SCHEMA,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_METHOD, MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_STATUS,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_ID,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_MISSING_STATUS,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_REASON,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SCHEMA,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_METHOD,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_STATUS, MODULE_LOADER_RUNTIME_FACT_SOURCES,
    MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT, MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_ID,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SCHEMA,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_STATUS,
};
use crate::agent_protocol_support::{
    record_bool as b, record_false as no, record_field as f, record_str as s,
};
use crate::event_log;
use alloc::{vec, vec::Vec};
use raios_core::record::Value as V;

pub(super) fn module_loader_runtime_source_fact_map() -> V<'static> {
    let mut values = Vec::new();
    let mut idx = 0usize;
    while idx < MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
        let source = MODULE_LOADER_RUNTIME_FACT_SOURCES[idx];
        values.push(V::InlineObject(vec![
            f("fact", s(source.name)),
            f("schema", s(source.schema)),
            f("aggregate_fact_id", s(source.id)),
            f("source_method", s(source.source_method)),
            f("source_fact_locator", s(source.source_fact_locator)),
            f("source_evidence_schema", s(source.source_evidence_schema)),
            f(
                "source_evidence_missing_reason",
                s(source.source_evidence_missing_reason),
            ),
            f("addressable", b(true)),
            f("included_in_required_fact_list", b(true)),
        ]));
        idx += 1;
    }
    V::Array(values)
}

pub(super) fn module_loader_runtime_selftest_case_value(
    case: &ModuleLoaderRuntimeSelfTestCase,
) -> V<'static> {
    V::InlineObject(vec![
        f("case", s(case.name)),
        f("expected_status", s(case.expected_status)),
        f("expected_reason", s(case.expected_reason)),
        f("actual_status", s(case.actual_status)),
        f("actual_reason", s(case.actual_reason)),
        f(
            "actual_loader_identity_source_evidence_present",
            b(case.actual_loader_identity_source_evidence_present),
        ),
        f(
            "actual_loader_identity_source_evidence_state",
            s(case.actual_loader_identity_source_evidence_state),
        ),
        f(
            "actual_loader_identity_source_evidence_status",
            s(case.actual_loader_identity_source_evidence_status),
        ),
        f(
            "actual_loader_identity_source_evidence_reason",
            s(case.actual_loader_identity_source_evidence_reason),
        ),
        f(
            "actual_artifact_hash_source_evidence_present",
            b(case.actual_artifact_hash_source_evidence_present),
        ),
        f(
            "actual_artifact_hash_source_evidence_state",
            s(case.actual_artifact_hash_source_evidence_state),
        ),
        f(
            "actual_artifact_hash_source_evidence_status",
            s(case.actual_artifact_hash_source_evidence_status),
        ),
        f(
            "actual_artifact_hash_source_evidence_reason",
            s(case.actual_artifact_hash_source_evidence_reason),
        ),
        f(
            "actual_entrypoint_abi_source_evidence_present",
            b(case.actual_entrypoint_abi_source_evidence_present),
        ),
        f(
            "actual_entrypoint_abi_source_evidence_state",
            s(case.actual_entrypoint_abi_source_evidence_state),
        ),
        f(
            "actual_entrypoint_abi_source_evidence_status",
            s(case.actual_entrypoint_abi_source_evidence_status),
        ),
        f(
            "actual_entrypoint_abi_source_evidence_reason",
            s(case.actual_entrypoint_abi_source_evidence_reason),
        ),
        f(
            "actual_address_space_source_evidence_present",
            b(case.actual_address_space_source_evidence_present),
        ),
        f(
            "actual_address_space_source_evidence_state",
            s(case.actual_address_space_source_evidence_state),
        ),
        f(
            "actual_address_space_source_evidence_status",
            s(case.actual_address_space_source_evidence_status),
        ),
        f(
            "actual_address_space_source_evidence_reason",
            s(case.actual_address_space_source_evidence_reason),
        ),
        f(
            "actual_memory_map_source_evidence_present",
            b(case.actual_memory_map_source_evidence_present),
        ),
        f(
            "actual_memory_map_source_evidence_state",
            s(case.actual_memory_map_source_evidence_state),
        ),
        f(
            "actual_memory_map_source_evidence_status",
            s(case.actual_memory_map_source_evidence_status),
        ),
        f(
            "actual_memory_map_source_evidence_reason",
            s(case.actual_memory_map_source_evidence_reason),
        ),
        f(
            "actual_capability_table_source_evidence_present",
            b(case.actual_capability_table_source_evidence_present),
        ),
        f(
            "actual_capability_table_source_evidence_state",
            s(case.actual_capability_table_source_evidence_state),
        ),
        f(
            "actual_capability_table_source_evidence_status",
            s(case.actual_capability_table_source_evidence_status),
        ),
        f(
            "actual_capability_table_source_evidence_reason",
            s(case.actual_capability_table_source_evidence_reason),
        ),
        f(
            "actual_service_slot_source_evidence_present",
            b(case.actual_service_slot_source_evidence_present),
        ),
        f(
            "actual_service_slot_source_evidence_state",
            s(case.actual_service_slot_source_evidence_state),
        ),
        f(
            "actual_service_slot_source_evidence_status",
            s(case.actual_service_slot_source_evidence_status),
        ),
        f(
            "actual_service_slot_source_evidence_reason",
            s(case.actual_service_slot_source_evidence_reason),
        ),
        f(
            "actual_health_source_evidence_present",
            b(case.actual_health_source_evidence_present),
        ),
        f(
            "actual_health_source_evidence_state",
            s(case.actual_health_source_evidence_state),
        ),
        f(
            "actual_health_source_evidence_status",
            s(case.actual_health_source_evidence_status),
        ),
        f(
            "actual_health_source_evidence_reason",
            s(case.actual_health_source_evidence_reason),
        ),
        f(
            "actual_rollback_source_evidence_present",
            b(case.actual_rollback_source_evidence_present),
        ),
        f(
            "actual_rollback_source_evidence_state",
            s(case.actual_rollback_source_evidence_state),
        ),
        f(
            "actual_rollback_source_evidence_status",
            s(case.actual_rollback_source_evidence_status),
        ),
        f(
            "actual_rollback_source_evidence_reason",
            s(case.actual_rollback_source_evidence_reason),
        ),
        f(
            "actual_write_boundary_source_evidence_present",
            b(case.actual_write_boundary_source_evidence_present),
        ),
        f(
            "actual_write_boundary_source_evidence_state",
            s(case.actual_write_boundary_source_evidence_state),
        ),
        f(
            "actual_write_boundary_source_evidence_status",
            s(case.actual_write_boundary_source_evidence_status),
        ),
        f(
            "actual_write_boundary_source_evidence_reason",
            s(case.actual_write_boundary_source_evidence_reason),
        ),
        f("passed", b(case.passed)),
        f("loads_artifact", no()),
        f("allocates_service_slot", no()),
        f("creates_service_inventory_records", no()),
        f("starts_service", no()),
        f("marks_service_running", no()),
        f("can_load", no()),
        f("load_attempted", no()),
    ])
}

pub(super) fn module_loader_runtime_execution_commit_gate_source_evidence(
    retained_service_slot_reservation_present: bool,
    service_slot_reservation_event_id: Option<event_log::EventId>,
    ram_only_service_slot_id: Option<event_log::ModuleServiceSlotId>,
    authority_decision_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
    )>,
    loader_runtime_contract_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotAuthorityInputSourceEvidence,
    )>,
    loader_identity_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderIdentitySourceEvidence,
    )>,
    artifact_hash_binding_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderArtifactHashBindingSourceEvidence,
    )>,
    entrypoint_abi_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    address_space_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    memory_map_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    capability_table_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    service_slot_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    health_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    rollback_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    write_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
) -> event_log::ModuleLoaderRuntimeExecutionCommitGateSourceEvidence {
    let authority_decision_present = authority_decision_source_evidence
        .map(|(_, evidence)| evidence.decision_present && evidence.source_chain_complete)
        .unwrap_or(false);
    let loader_runtime_contract_present = loader_runtime_contract_source_evidence
        .map(|(_, evidence)| evidence.input_present && evidence.source_chain_complete)
        .unwrap_or(false);
    let loader_runtime_source_evidence_event_ids = [
        loader_identity_source_evidence.map(|(event_id, _)| event_id),
        artifact_hash_binding_source_evidence.map(|(event_id, _)| event_id),
        entrypoint_abi_source_evidence.map(|(event_id, _)| event_id),
        address_space_source_evidence.map(|(event_id, _)| event_id),
        memory_map_source_evidence.map(|(event_id, _)| event_id),
        capability_table_source_evidence.map(|(event_id, _)| event_id),
        service_slot_source_evidence.map(|(event_id, _)| event_id),
        health_source_evidence.map(|(event_id, _)| event_id),
        rollback_source_evidence.map(|(event_id, _)| event_id),
        write_boundary_source_evidence.map(|(event_id, _)| event_id),
    ];
    let loader_runtime_source_evidence_present = [
        loader_identity_source_evidence.is_some(),
        artifact_hash_binding_source_evidence.is_some(),
        entrypoint_abi_source_evidence.is_some(),
        address_space_source_evidence.is_some(),
        memory_map_source_evidence.is_some(),
        capability_table_source_evidence.is_some(),
        service_slot_source_evidence.is_some(),
        health_source_evidence.is_some(),
        rollback_source_evidence.is_some(),
        write_boundary_source_evidence.is_some(),
    ];
    let loader_runtime_fact_present = [
        loader_identity_source_evidence
            .map(|(_, evidence)| evidence.identity_present)
            .unwrap_or(false),
        artifact_hash_binding_source_evidence
            .map(|(_, evidence)| evidence.artifact_hash_binding_present)
            .unwrap_or(false),
        entrypoint_abi_source_evidence
            .map(|(_, evidence)| evidence.fact_present)
            .unwrap_or(false),
        address_space_source_evidence
            .map(|(_, evidence)| evidence.fact_present)
            .unwrap_or(false),
        memory_map_source_evidence
            .map(|(_, evidence)| evidence.fact_present)
            .unwrap_or(false),
        capability_table_source_evidence
            .map(|(_, evidence)| evidence.fact_present)
            .unwrap_or(false),
        service_slot_source_evidence
            .map(|(_, evidence)| evidence.fact_present)
            .unwrap_or(false),
        health_source_evidence
            .map(|(_, evidence)| evidence.fact_present)
            .unwrap_or(false),
        rollback_source_evidence
            .map(|(_, evidence)| evidence.fact_present)
            .unwrap_or(false),
        write_boundary_source_evidence
            .map(|(_, evidence)| evidence.fact_present)
            .unwrap_or(false),
    ];
    let mut loader_runtime_source_evidence_complete = true;
    let mut idx = 0usize;
    while idx < MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
        if !loader_runtime_source_evidence_present[idx] {
            loader_runtime_source_evidence_complete = false;
            break;
        }
        idx += 1;
    }
    let source_chain_complete = authority_decision_present
        && loader_runtime_contract_present
        && loader_runtime_source_evidence_complete
        && retained_service_slot_reservation_present;
    event_log::ModuleLoaderRuntimeExecutionCommitGateSourceEvidence {
        schema: MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_EVIDENCE_SCHEMA,
        gate_schema: MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SCHEMA,
        gate_id: MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_ID,
        source_method: MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_METHOD,
        source_fact_locator: MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_FACT_LOCATOR,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_loader_runtime_execution_commit_gate_source_evidence_recorded",
        gate_status: if source_chain_complete {
            MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_STATUS
        } else {
            MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_MISSING_STATUS
        },
        gate_reason: if source_chain_complete {
            MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_REASON
        } else {
            MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_CHAIN_INCOMPLETE_REASON
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
        loader_runtime_contract_present,
        loader_runtime_source_evidence_complete,
        service_slot_binding_source_evidence_present: service_slot_source_evidence.is_some(),
        service_slot_binding_fact_present: service_slot_source_evidence
            .map(|(_, evidence)| evidence.fact_present)
            .unwrap_or(false),
        audit_rollback_write_boundary_source_evidence_present: write_boundary_source_evidence
            .is_some(),
        audit_rollback_write_boundary_fact_present: write_boundary_source_evidence
            .map(|(_, evidence)| evidence.fact_present)
            .unwrap_or(false),
        retained_service_slot_reservation_present,
        source_chain_complete,
        authority_decision_source_evidence_event_id: authority_decision_source_evidence
            .map(|(event_id, _)| event_id),
        loader_runtime_contract_source_evidence_event_id: loader_runtime_contract_source_evidence
            .map(|(event_id, _)| event_id),
        loader_runtime_source_evidence_event_ids,
        loader_runtime_source_evidence_present,
        loader_runtime_fact_present,
        service_slot_reservation_event_id,
        ram_only_service_slot_id,
        accepts_loader_descriptor: false,
        accepts_artifact_bytes: false,
        authorizes_execution: false,
        mutates_service_registry: false,
        writes_durable_audit_state: false,
        installs_rollback_state: false,
        allocates_service_slot: false,
        loads_artifact: false,
    }
}

pub(super) fn module_loader_descriptor_intake_boundary_source_evidence(
    manifest_reference_event_id: Option<event_log::EventId>,
    artifact_reference_event_id: Option<event_log::EventId>,
    vm_test_report_reference_event_id: Option<event_log::EventId>,
    local_attestation_reference_event_id: Option<event_log::EventId>,
    local_approval_reference_event_id: Option<event_log::EventId>,
    computed_grant_reference_event_id: Option<event_log::EventId>,
    audit_rollback_reference_event_id: Option<event_log::EventId>,
    service_slot_reservation_event_id: Option<event_log::EventId>,
    ram_only_service_slot_id: Option<event_log::ModuleServiceSlotId>,
    registry_write_commit_gate_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence,
    )>,
    execution_commit_gate_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderRuntimeExecutionCommitGateSourceEvidence,
    ),
) -> event_log::ModuleLoaderDescriptorIntakeBoundarySourceEvidence {
    let retained_module_evidence_present = manifest_reference_event_id.is_some()
        && artifact_reference_event_id.is_some()
        && vm_test_report_reference_event_id.is_some()
        && local_attestation_reference_event_id.is_some()
        && local_approval_reference_event_id.is_some()
        && computed_grant_reference_event_id.is_some()
        && audit_rollback_reference_event_id.is_some()
        && service_slot_reservation_event_id.is_some();
    let registry_write_commit_gate_present = registry_write_commit_gate_source_evidence
        .map(|(_, evidence)| evidence.gate_present && evidence.source_chain_complete)
        .unwrap_or(false);
    let execution_commit_gate_present = execution_commit_gate_source_evidence.1.gate_present
        && execution_commit_gate_source_evidence
            .1
            .source_chain_complete;
    let loader_runtime_source_evidence_complete = execution_commit_gate_source_evidence
        .1
        .loader_runtime_source_evidence_complete;
    let source_chain_complete = retained_module_evidence_present
        && registry_write_commit_gate_present
        && execution_commit_gate_present
        && loader_runtime_source_evidence_complete
        && service_slot_reservation_event_id.is_some();
    event_log::ModuleLoaderDescriptorIntakeBoundarySourceEvidence {
        schema: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        boundary_schema: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SCHEMA,
        boundary_id: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_ID,
        source_method: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_METHOD,
        source_fact_locator: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_FACT_LOCATOR,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_loader_descriptor_intake_boundary_source_evidence_recorded",
        boundary_status: if source_chain_complete {
            MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_STATUS
        } else {
            MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_MISSING_STATUS
        },
        boundary_reason: if source_chain_complete {
            MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_REASON
        } else {
            MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON
        },
        boundary_present: source_chain_complete,
        boundary_scope: "current_boot",
        boundary_schema_ok: true,
        boundary_provenance_ok: source_chain_complete,
        boundary_classification: "local_only",
        requested_capability: "cap.module.load_ephemeral",
        load_mode: "ram_only",
        target: "live_service_graph",
        registry_write_commit_gate_present,
        execution_commit_gate_present,
        loader_runtime_source_evidence_complete,
        retained_module_evidence_present,
        retained_service_slot_reservation_present: service_slot_reservation_event_id.is_some(),
        source_chain_complete,
        registry_write_commit_gate_source_evidence_event_id:
            registry_write_commit_gate_source_evidence.map(|(event_id, _)| event_id),
        execution_commit_gate_source_evidence_event_id: Some(
            execution_commit_gate_source_evidence.0,
        ),
        loader_runtime_source_evidence_event_ids: execution_commit_gate_source_evidence
            .1
            .loader_runtime_source_evidence_event_ids,
        loader_runtime_source_evidence_present: execution_commit_gate_source_evidence
            .1
            .loader_runtime_source_evidence_present,
        loader_runtime_fact_present: execution_commit_gate_source_evidence
            .1
            .loader_runtime_fact_present,
        manifest_reference_event_id,
        artifact_reference_event_id,
        vm_test_report_reference_event_id,
        local_attestation_reference_event_id,
        local_approval_reference_event_id,
        computed_grant_reference_event_id,
        audit_rollback_reference_event_id,
        service_slot_reservation_event_id,
        ram_only_service_slot_id,
        accepts_loader_descriptor: false,
        accepts_descriptor_bytes: false,
        accepts_artifact_bytes: false,
        authorizes_descriptor_intake: false,
        authorizes_execution: false,
        mutates_service_registry: false,
        writes_durable_audit_state: false,
        installs_rollback_state: false,
        allocates_service_slot: false,
        loads_artifact: false,
    }
}

pub(super) fn module_loader_artifact_byte_intake_boundary_source_evidence(
    manifest_reference_event_id: Option<event_log::EventId>,
    artifact_reference_event_id: Option<event_log::EventId>,
    vm_test_report_reference_event_id: Option<event_log::EventId>,
    local_attestation_reference_event_id: Option<event_log::EventId>,
    local_approval_reference_event_id: Option<event_log::EventId>,
    computed_grant_reference_event_id: Option<event_log::EventId>,
    audit_rollback_reference_event_id: Option<event_log::EventId>,
    service_slot_reservation_event_id: Option<event_log::EventId>,
    ram_only_service_slot_id: Option<event_log::ModuleServiceSlotId>,
    execution_commit_gate_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderRuntimeExecutionCommitGateSourceEvidence,
    ),
    descriptor_intake_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderDescriptorIntakeBoundarySourceEvidence,
    ),
    artifact_hash_binding_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderArtifactHashBindingSourceEvidence,
    )>,
) -> event_log::ModuleLoaderArtifactByteIntakeBoundarySourceEvidence {
    let retained_module_evidence_present = manifest_reference_event_id.is_some()
        && artifact_reference_event_id.is_some()
        && vm_test_report_reference_event_id.is_some()
        && local_attestation_reference_event_id.is_some()
        && local_approval_reference_event_id.is_some()
        && computed_grant_reference_event_id.is_some()
        && audit_rollback_reference_event_id.is_some()
        && service_slot_reservation_event_id.is_some();
    let descriptor_intake_boundary_present = descriptor_intake_boundary_source_evidence
        .1
        .boundary_present
        && descriptor_intake_boundary_source_evidence
            .1
            .source_chain_complete;
    let execution_commit_gate_present = execution_commit_gate_source_evidence.1.gate_present
        && execution_commit_gate_source_evidence
            .1
            .source_chain_complete;
    let artifact_hash_binding_present = execution_commit_gate_source_evidence
        .1
        .loader_runtime_source_evidence_present[1];
    let source_chain_complete = retained_module_evidence_present
        && artifact_reference_event_id.is_some()
        && descriptor_intake_boundary_present
        && execution_commit_gate_present
        && artifact_hash_binding_present
        && service_slot_reservation_event_id.is_some();
    event_log::ModuleLoaderArtifactByteIntakeBoundarySourceEvidence {
        schema: MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        boundary_schema: MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SCHEMA,
        boundary_id: MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_ID,
        source_method: MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_METHOD,
        source_fact_locator: MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_FACT_LOCATOR,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_loader_artifact_byte_intake_boundary_source_evidence_recorded",
        boundary_status: if source_chain_complete {
            MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_STATUS
        } else {
            MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_MISSING_STATUS
        },
        boundary_reason: if source_chain_complete {
            MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_REASON
        } else {
            MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON
        },
        boundary_present: source_chain_complete,
        boundary_scope: "current_boot",
        boundary_schema_ok: true,
        boundary_provenance_ok: source_chain_complete,
        boundary_classification: "local_only",
        requested_capability: "cap.module.load_ephemeral",
        load_mode: "ram_only",
        target: "live_service_graph",
        descriptor_intake_boundary_present,
        descriptor_intake_boundary_source_chain_complete:
            descriptor_intake_boundary_source_evidence
                .1
                .source_chain_complete,
        execution_commit_gate_present,
        artifact_hash_binding_present,
        retained_artifact_reference_present: artifact_reference_event_id.is_some(),
        retained_module_evidence_present,
        retained_service_slot_reservation_present: service_slot_reservation_event_id.is_some(),
        source_chain_complete,
        descriptor_intake_boundary_source_evidence_event_id: Some(
            descriptor_intake_boundary_source_evidence.0,
        ),
        execution_commit_gate_source_evidence_event_id: Some(
            execution_commit_gate_source_evidence.0,
        ),
        artifact_hash_binding_source_evidence_event_id: artifact_hash_binding_source_evidence
            .map(|(event_id, _)| event_id),
        loader_runtime_source_evidence_event_ids: execution_commit_gate_source_evidence
            .1
            .loader_runtime_source_evidence_event_ids,
        loader_runtime_source_evidence_present: execution_commit_gate_source_evidence
            .1
            .loader_runtime_source_evidence_present,
        loader_runtime_fact_present: execution_commit_gate_source_evidence
            .1
            .loader_runtime_fact_present,
        manifest_reference_event_id,
        artifact_reference_event_id,
        vm_test_report_reference_event_id,
        local_attestation_reference_event_id,
        local_approval_reference_event_id,
        computed_grant_reference_event_id,
        audit_rollback_reference_event_id,
        service_slot_reservation_event_id,
        ram_only_service_slot_id,
        accepts_loader_descriptor: false,
        accepts_descriptor_bytes: false,
        accepts_artifact_bytes: false,
        authorizes_descriptor_intake: false,
        authorizes_artifact_byte_intake: false,
        authorizes_execution: false,
        mutates_service_registry: false,
        writes_durable_audit_state: false,
        installs_rollback_state: false,
        allocates_service_slot: false,
        loads_artifact: false,
    }
}

pub(super) fn module_loader_execution_authorization_boundary_source_evidence(
    manifest_reference_event_id: Option<event_log::EventId>,
    artifact_reference_event_id: Option<event_log::EventId>,
    vm_test_report_reference_event_id: Option<event_log::EventId>,
    local_attestation_reference_event_id: Option<event_log::EventId>,
    local_approval_reference_event_id: Option<event_log::EventId>,
    computed_grant_reference_event_id: Option<event_log::EventId>,
    audit_rollback_reference_event_id: Option<event_log::EventId>,
    service_slot_reservation_event_id: Option<event_log::EventId>,
    ram_only_service_slot_id: Option<event_log::ModuleServiceSlotId>,
    execution_commit_gate_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderRuntimeExecutionCommitGateSourceEvidence,
    ),
    descriptor_intake_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderDescriptorIntakeBoundarySourceEvidence,
    ),
    artifact_byte_intake_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderArtifactByteIntakeBoundarySourceEvidence,
    ),
    entrypoint_abi_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    address_space_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    memory_map_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    write_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
) -> event_log::ModuleLoaderExecutionAuthorizationBoundarySourceEvidence {
    let retained_module_evidence_present = manifest_reference_event_id.is_some()
        && artifact_reference_event_id.is_some()
        && vm_test_report_reference_event_id.is_some()
        && local_attestation_reference_event_id.is_some()
        && local_approval_reference_event_id.is_some()
        && computed_grant_reference_event_id.is_some()
        && audit_rollback_reference_event_id.is_some()
        && service_slot_reservation_event_id.is_some();
    let artifact_byte_intake_boundary_present = artifact_byte_intake_boundary_source_evidence
        .1
        .boundary_present
        && artifact_byte_intake_boundary_source_evidence
            .1
            .source_chain_complete;
    let descriptor_intake_boundary_present = descriptor_intake_boundary_source_evidence
        .1
        .boundary_present
        && descriptor_intake_boundary_source_evidence
            .1
            .source_chain_complete;
    let execution_commit_gate_present = execution_commit_gate_source_evidence.1.gate_present
        && execution_commit_gate_source_evidence
            .1
            .source_chain_complete;
    let entrypoint_abi_source_evidence_present = entrypoint_abi_source_evidence.is_some()
        || execution_commit_gate_source_evidence
            .1
            .loader_runtime_source_evidence_present[2];
    let address_space_source_evidence_present = address_space_source_evidence.is_some()
        || execution_commit_gate_source_evidence
            .1
            .loader_runtime_source_evidence_present[3];
    let memory_map_source_evidence_present = memory_map_source_evidence.is_some()
        || execution_commit_gate_source_evidence
            .1
            .loader_runtime_source_evidence_present[4];
    let audit_rollback_write_boundary_source_evidence_present = write_boundary_source_evidence
        .is_some()
        || execution_commit_gate_source_evidence
            .1
            .loader_runtime_source_evidence_present[9];
    let source_chain_complete = retained_module_evidence_present
        && artifact_byte_intake_boundary_present
        && descriptor_intake_boundary_present
        && execution_commit_gate_present
        && entrypoint_abi_source_evidence_present
        && address_space_source_evidence_present
        && memory_map_source_evidence_present
        && audit_rollback_write_boundary_source_evidence_present
        && service_slot_reservation_event_id.is_some();
    event_log::ModuleLoaderExecutionAuthorizationBoundarySourceEvidence {
        schema: MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        boundary_schema: MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SCHEMA,
        boundary_id: MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_ID,
        source_method: MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_METHOD,
        source_fact_locator: MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_FACT_LOCATOR,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason: "module_loader_execution_authorization_boundary_source_evidence_recorded",
        boundary_status: if source_chain_complete {
            MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_STATUS
        } else {
            MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_MISSING_STATUS
        },
        boundary_reason: if source_chain_complete {
            MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_REASON
        } else {
            MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON
        },
        boundary_present: source_chain_complete,
        boundary_scope: "current_boot",
        boundary_schema_ok: true,
        boundary_provenance_ok: source_chain_complete,
        boundary_classification: "local_only",
        requested_capability: "cap.module.load_ephemeral",
        load_mode: "ram_only",
        target: "live_service_graph",
        artifact_byte_intake_boundary_present,
        artifact_byte_intake_boundary_source_chain_complete:
            artifact_byte_intake_boundary_source_evidence
                .1
                .source_chain_complete,
        descriptor_intake_boundary_present,
        descriptor_intake_boundary_source_chain_complete:
            descriptor_intake_boundary_source_evidence
                .1
                .source_chain_complete,
        execution_commit_gate_present,
        entrypoint_abi_source_evidence_present,
        address_space_source_evidence_present,
        memory_map_source_evidence_present,
        audit_rollback_write_boundary_source_evidence_present,
        retained_module_evidence_present,
        retained_service_slot_reservation_present: service_slot_reservation_event_id.is_some(),
        source_chain_complete,
        artifact_byte_intake_boundary_source_evidence_event_id: Some(
            artifact_byte_intake_boundary_source_evidence.0,
        ),
        descriptor_intake_boundary_source_evidence_event_id: Some(
            descriptor_intake_boundary_source_evidence.0,
        ),
        execution_commit_gate_source_evidence_event_id: Some(
            execution_commit_gate_source_evidence.0,
        ),
        entrypoint_abi_source_evidence_event_id: entrypoint_abi_source_evidence
            .map(|(event_id, _)| event_id)
            .or(execution_commit_gate_source_evidence
                .1
                .loader_runtime_source_evidence_event_ids[2]),
        address_space_source_evidence_event_id: address_space_source_evidence
            .map(|(event_id, _)| event_id)
            .or(execution_commit_gate_source_evidence
                .1
                .loader_runtime_source_evidence_event_ids[3]),
        memory_map_source_evidence_event_id: memory_map_source_evidence
            .map(|(event_id, _)| event_id)
            .or(execution_commit_gate_source_evidence
                .1
                .loader_runtime_source_evidence_event_ids[4]),
        audit_rollback_write_boundary_source_evidence_event_id: write_boundary_source_evidence
            .map(|(event_id, _)| event_id)
            .or(execution_commit_gate_source_evidence
                .1
                .loader_runtime_source_evidence_event_ids[9]),
        loader_runtime_source_evidence_event_ids: execution_commit_gate_source_evidence
            .1
            .loader_runtime_source_evidence_event_ids,
        loader_runtime_source_evidence_present: execution_commit_gate_source_evidence
            .1
            .loader_runtime_source_evidence_present,
        loader_runtime_fact_present: execution_commit_gate_source_evidence
            .1
            .loader_runtime_fact_present,
        manifest_reference_event_id,
        artifact_reference_event_id,
        vm_test_report_reference_event_id,
        local_attestation_reference_event_id,
        local_approval_reference_event_id,
        computed_grant_reference_event_id,
        audit_rollback_reference_event_id,
        service_slot_reservation_event_id,
        ram_only_service_slot_id,
        accepts_loader_descriptor: false,
        accepts_descriptor_bytes: false,
        accepts_artifact_bytes: false,
        authorizes_descriptor_intake: false,
        authorizes_artifact_byte_intake: false,
        maps_executable_pages: false,
        jumps_to_entrypoint: false,
        authorizes_execution: false,
        mutates_service_registry: false,
        writes_durable_audit_state: false,
        installs_rollback_state: false,
        allocates_service_slot: false,
        loads_artifact: false,
    }
}

pub(super) fn module_loader_service_registry_mutation_boundary_source_evidence(
    manifest_reference_event_id: Option<event_log::EventId>,
    artifact_reference_event_id: Option<event_log::EventId>,
    vm_test_report_reference_event_id: Option<event_log::EventId>,
    local_attestation_reference_event_id: Option<event_log::EventId>,
    local_approval_reference_event_id: Option<event_log::EventId>,
    computed_grant_reference_event_id: Option<event_log::EventId>,
    audit_rollback_reference_event_id: Option<event_log::EventId>,
    service_slot_reservation_event_id: Option<event_log::EventId>,
    ram_only_service_slot_id: Option<event_log::ModuleServiceSlotId>,
    execution_authorization_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderExecutionAuthorizationBoundarySourceEvidence,
    ),
    registry_write_commit_gate_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleServiceSlotRegistryWriteCommitGateSourceEvidence,
    )>,
    service_slot_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
) -> event_log::ModuleLoaderServiceRegistryMutationBoundarySourceEvidence {
    let retained_module_evidence_present = manifest_reference_event_id.is_some()
        && artifact_reference_event_id.is_some()
        && vm_test_report_reference_event_id.is_some()
        && local_attestation_reference_event_id.is_some()
        && local_approval_reference_event_id.is_some()
        && computed_grant_reference_event_id.is_some()
        && audit_rollback_reference_event_id.is_some()
        && service_slot_reservation_event_id.is_some();
    let execution_authorization_boundary_present = execution_authorization_boundary_source_evidence
        .1
        .boundary_present
        && execution_authorization_boundary_source_evidence
            .1
            .source_chain_complete;
    let registry_write_commit_gate_present = registry_write_commit_gate_source_evidence
        .map(|(_, evidence)| evidence.gate_present && evidence.source_chain_complete)
        .unwrap_or(false);
    let service_slot_binding_source_evidence_present = service_slot_source_evidence.is_some()
        || execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_present[6];
    let source_chain_complete = retained_module_evidence_present
        && execution_authorization_boundary_present
        && registry_write_commit_gate_present
        && service_slot_binding_source_evidence_present
        && service_slot_reservation_event_id.is_some();
    event_log::ModuleLoaderServiceRegistryMutationBoundarySourceEvidence {
        schema: MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        boundary_schema: MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SCHEMA,
        boundary_id: MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_ID,
        source_method: MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_METHOD,
        source_fact_locator: MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_FACT_LOCATOR,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason:
            "module_loader_service_registry_mutation_boundary_source_evidence_recorded",
        boundary_status: if source_chain_complete {
            MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_STATUS
        } else {
            MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_MISSING_STATUS
        },
        boundary_reason: if source_chain_complete {
            MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_REASON
        } else {
            MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON
        },
        boundary_present: source_chain_complete,
        boundary_scope: "current_boot",
        boundary_schema_ok: true,
        boundary_provenance_ok: source_chain_complete,
        boundary_classification: "local_only",
        requested_capability: "cap.module.load_ephemeral",
        load_mode: "ram_only",
        target: "live_service_graph",
        execution_authorization_boundary_present,
        execution_authorization_boundary_source_chain_complete:
            execution_authorization_boundary_source_evidence
                .1
                .source_chain_complete,
        registry_write_commit_gate_present,
        service_slot_binding_source_evidence_present,
        retained_module_evidence_present,
        retained_service_slot_reservation_present: service_slot_reservation_event_id.is_some(),
        source_chain_complete,
        execution_authorization_boundary_source_evidence_event_id: Some(
            execution_authorization_boundary_source_evidence.0,
        ),
        registry_write_commit_gate_source_evidence_event_id:
            registry_write_commit_gate_source_evidence.map(|(event_id, _)| event_id),
        service_slot_binding_source_evidence_event_id: service_slot_source_evidence
            .map(|(event_id, _)| event_id)
            .or(execution_authorization_boundary_source_evidence
                .1
                .loader_runtime_source_evidence_event_ids[6]),
        loader_runtime_source_evidence_event_ids: execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_event_ids,
        loader_runtime_source_evidence_present: execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_present,
        loader_runtime_fact_present: execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_fact_present,
        manifest_reference_event_id,
        artifact_reference_event_id,
        vm_test_report_reference_event_id,
        local_attestation_reference_event_id,
        local_approval_reference_event_id,
        computed_grant_reference_event_id,
        audit_rollback_reference_event_id,
        service_slot_reservation_event_id,
        ram_only_service_slot_id,
        accepts_loader_descriptor: false,
        accepts_descriptor_bytes: false,
        accepts_artifact_bytes: false,
        authorizes_descriptor_intake: false,
        authorizes_artifact_byte_intake: false,
        maps_executable_pages: false,
        jumps_to_entrypoint: false,
        authorizes_execution: false,
        mutates_service_registry: false,
        writes_durable_audit_state: false,
        installs_rollback_state: false,
        allocates_service_slot: false,
        creates_service_inventory_records: false,
        loads_artifact: false,
    }
}

pub(super) fn module_loader_load_attempt_boundary_source_evidence(
    manifest_reference_event_id: Option<event_log::EventId>,
    artifact_reference_event_id: Option<event_log::EventId>,
    vm_test_report_reference_event_id: Option<event_log::EventId>,
    local_attestation_reference_event_id: Option<event_log::EventId>,
    local_approval_reference_event_id: Option<event_log::EventId>,
    computed_grant_reference_event_id: Option<event_log::EventId>,
    audit_rollback_reference_event_id: Option<event_log::EventId>,
    service_slot_reservation_event_id: Option<event_log::EventId>,
    ram_only_service_slot_id: Option<event_log::ModuleServiceSlotId>,
    artifact_byte_intake_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderArtifactByteIntakeBoundarySourceEvidence,
    ),
    execution_authorization_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderExecutionAuthorizationBoundarySourceEvidence,
    ),
    service_registry_mutation_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderServiceRegistryMutationBoundarySourceEvidence,
    ),
    write_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let retained_module_evidence_present = module_loader_retained_module_evidence_present(
        manifest_reference_event_id,
        artifact_reference_event_id,
        vm_test_report_reference_event_id,
        local_attestation_reference_event_id,
        local_approval_reference_event_id,
        computed_grant_reference_event_id,
        audit_rollback_reference_event_id,
        service_slot_reservation_event_id,
    );
    let artifact_byte_intake_boundary_present = artifact_byte_intake_boundary_source_evidence
        .1
        .boundary_present
        && artifact_byte_intake_boundary_source_evidence
            .1
            .source_chain_complete;
    let execution_authorization_boundary_present = execution_authorization_boundary_source_evidence
        .1
        .boundary_present
        && execution_authorization_boundary_source_evidence
            .1
            .source_chain_complete;
    let service_registry_mutation_boundary_present =
        service_registry_mutation_boundary_source_evidence
            .1
            .boundary_present
            && service_registry_mutation_boundary_source_evidence
                .1
                .source_chain_complete;
    let audit_rollback_write_boundary_source_evidence_present = write_boundary_source_evidence
        .is_some()
        || execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_present[9];
    let source_chain_complete = retained_module_evidence_present
        && artifact_byte_intake_boundary_present
        && execution_authorization_boundary_present
        && service_registry_mutation_boundary_present
        && audit_rollback_write_boundary_source_evidence_present
        && service_slot_reservation_event_id.is_some();
    module_loader_live_load_boundary_source_evidence_record(
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SCHEMA,
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_ID,
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_load_attempt_boundary_source_evidence_recorded",
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_STATUS,
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_REASON,
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        source_chain_complete,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        artifact_byte_intake_boundary_present,
        artifact_byte_intake_boundary_source_evidence
            .1
            .source_chain_complete,
        execution_authorization_boundary_present,
        execution_authorization_boundary_source_evidence
            .1
            .source_chain_complete,
        service_registry_mutation_boundary_present,
        service_registry_mutation_boundary_source_evidence
            .1
            .source_chain_complete,
        service_registry_mutation_boundary_source_evidence
            .1
            .service_slot_binding_source_evidence_present,
        execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_present[7],
        false,
        execution_authorization_boundary_source_evidence
            .1
            .entrypoint_abi_source_evidence_present,
        execution_authorization_boundary_source_evidence
            .1
            .address_space_source_evidence_present,
        execution_authorization_boundary_source_evidence
            .1
            .memory_map_source_evidence_present,
        execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_present[5],
        audit_rollback_write_boundary_source_evidence_present,
        retained_module_evidence_present,
        artifact_reference_event_id.is_some(),
        service_slot_reservation_event_id.is_some(),
        None,
        None,
        None,
        None,
        Some(artifact_byte_intake_boundary_source_evidence.0),
        Some(execution_authorization_boundary_source_evidence.0),
        Some(service_registry_mutation_boundary_source_evidence.0),
        service_registry_mutation_boundary_source_evidence
            .1
            .service_slot_binding_source_evidence_event_id,
        execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_event_ids[7],
        None,
        execution_authorization_boundary_source_evidence
            .1
            .entrypoint_abi_source_evidence_event_id,
        execution_authorization_boundary_source_evidence
            .1
            .address_space_source_evidence_event_id,
        execution_authorization_boundary_source_evidence
            .1
            .memory_map_source_evidence_event_id,
        execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_event_ids[5],
        write_boundary_source_evidence
            .map(|(event_id, _)| event_id)
            .or(execution_authorization_boundary_source_evidence
                .1
                .audit_rollback_write_boundary_source_evidence_event_id),
        execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_event_ids,
        execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_present,
        execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_fact_present,
        manifest_reference_event_id,
        artifact_reference_event_id,
        vm_test_report_reference_event_id,
        local_attestation_reference_event_id,
        local_approval_reference_event_id,
        computed_grant_reference_event_id,
        audit_rollback_reference_event_id,
        service_slot_reservation_event_id,
        ram_only_service_slot_id,
    )
}

pub(super) fn module_loader_artifact_load_boundary_source_evidence(
    manifest_reference_event_id: Option<event_log::EventId>,
    artifact_reference_event_id: Option<event_log::EventId>,
    vm_test_report_reference_event_id: Option<event_log::EventId>,
    local_attestation_reference_event_id: Option<event_log::EventId>,
    local_approval_reference_event_id: Option<event_log::EventId>,
    computed_grant_reference_event_id: Option<event_log::EventId>,
    audit_rollback_reference_event_id: Option<event_log::EventId>,
    service_slot_reservation_event_id: Option<event_log::EventId>,
    ram_only_service_slot_id: Option<event_log::ModuleServiceSlotId>,
    load_attempt_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    artifact_byte_intake_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderArtifactByteIntakeBoundarySourceEvidence,
    ),
    artifact_hash_binding_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderArtifactHashBindingSourceEvidence,
    )>,
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let retained_module_evidence_present = module_loader_retained_module_evidence_present(
        manifest_reference_event_id,
        artifact_reference_event_id,
        vm_test_report_reference_event_id,
        local_attestation_reference_event_id,
        local_approval_reference_event_id,
        computed_grant_reference_event_id,
        audit_rollback_reference_event_id,
        service_slot_reservation_event_id,
    );
    let load_attempt_boundary_present = load_attempt_boundary_source_evidence.1.boundary_present
        && load_attempt_boundary_source_evidence
            .1
            .source_chain_complete;
    let artifact_byte_intake_boundary_present = artifact_byte_intake_boundary_source_evidence
        .1
        .boundary_present
        && artifact_byte_intake_boundary_source_evidence
            .1
            .source_chain_complete;
    let artifact_hash_binding_present = artifact_hash_binding_source_evidence.is_some()
        || load_attempt_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_present[1];
    let source_chain_complete = retained_module_evidence_present
        && load_attempt_boundary_present
        && artifact_byte_intake_boundary_present
        && artifact_hash_binding_present
        && artifact_reference_event_id.is_some()
        && service_slot_reservation_event_id.is_some();
    module_loader_live_load_boundary_source_evidence_record(
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SCHEMA,
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_ID,
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_artifact_load_boundary_source_evidence_recorded",
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_STATUS,
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_REASON,
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        source_chain_complete,
        load_attempt_boundary_present,
        load_attempt_boundary_source_evidence
            .1
            .source_chain_complete,
        false,
        false,
        false,
        false,
        false,
        false,
        artifact_byte_intake_boundary_present,
        artifact_byte_intake_boundary_source_evidence
            .1
            .source_chain_complete,
        false,
        false,
        false,
        false,
        load_attempt_boundary_source_evidence
            .1
            .service_slot_binding_source_evidence_present,
        load_attempt_boundary_source_evidence
            .1
            .health_state_hooks_source_evidence_present,
        artifact_hash_binding_present,
        false,
        false,
        false,
        false,
        load_attempt_boundary_source_evidence
            .1
            .audit_rollback_write_boundary_source_evidence_present,
        retained_module_evidence_present,
        artifact_reference_event_id.is_some(),
        service_slot_reservation_event_id.is_some(),
        Some(load_attempt_boundary_source_evidence.0),
        None,
        None,
        None,
        Some(artifact_byte_intake_boundary_source_evidence.0),
        None,
        None,
        load_attempt_boundary_source_evidence
            .1
            .service_slot_binding_source_evidence_event_id,
        load_attempt_boundary_source_evidence
            .1
            .health_state_hooks_source_evidence_event_id,
        artifact_hash_binding_source_evidence.map(|(event_id, _)| event_id),
        None,
        None,
        None,
        None,
        load_attempt_boundary_source_evidence
            .1
            .audit_rollback_write_boundary_source_evidence_event_id,
        load_attempt_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_event_ids,
        load_attempt_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_present,
        load_attempt_boundary_source_evidence
            .1
            .loader_runtime_fact_present,
        manifest_reference_event_id,
        artifact_reference_event_id,
        vm_test_report_reference_event_id,
        local_attestation_reference_event_id,
        local_approval_reference_event_id,
        computed_grant_reference_event_id,
        audit_rollback_reference_event_id,
        service_slot_reservation_event_id,
        ram_only_service_slot_id,
    )
}

pub(super) fn module_loader_executable_mapping_boundary_source_evidence(
    manifest_reference_event_id: Option<event_log::EventId>,
    artifact_reference_event_id: Option<event_log::EventId>,
    vm_test_report_reference_event_id: Option<event_log::EventId>,
    local_attestation_reference_event_id: Option<event_log::EventId>,
    local_approval_reference_event_id: Option<event_log::EventId>,
    computed_grant_reference_event_id: Option<event_log::EventId>,
    audit_rollback_reference_event_id: Option<event_log::EventId>,
    service_slot_reservation_event_id: Option<event_log::EventId>,
    ram_only_service_slot_id: Option<event_log::ModuleServiceSlotId>,
    artifact_load_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    execution_authorization_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderExecutionAuthorizationBoundarySourceEvidence,
    ),
    address_space_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    memory_map_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let retained_module_evidence_present = module_loader_retained_module_evidence_present(
        manifest_reference_event_id,
        artifact_reference_event_id,
        vm_test_report_reference_event_id,
        local_attestation_reference_event_id,
        local_approval_reference_event_id,
        computed_grant_reference_event_id,
        audit_rollback_reference_event_id,
        service_slot_reservation_event_id,
    );
    let artifact_load_boundary_present = artifact_load_boundary_source_evidence.1.boundary_present
        && artifact_load_boundary_source_evidence
            .1
            .source_chain_complete;
    let execution_authorization_boundary_present = execution_authorization_boundary_source_evidence
        .1
        .boundary_present
        && execution_authorization_boundary_source_evidence
            .1
            .source_chain_complete;
    let address_space_source_evidence_present = address_space_source_evidence.is_some()
        || execution_authorization_boundary_source_evidence
            .1
            .address_space_source_evidence_present;
    let memory_map_source_evidence_present = memory_map_source_evidence.is_some()
        || execution_authorization_boundary_source_evidence
            .1
            .memory_map_source_evidence_present;
    let source_chain_complete = retained_module_evidence_present
        && artifact_load_boundary_present
        && execution_authorization_boundary_present
        && address_space_source_evidence_present
        && memory_map_source_evidence_present
        && service_slot_reservation_event_id.is_some();
    module_loader_live_load_boundary_source_evidence_record(
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_ID,
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_executable_mapping_boundary_source_evidence_recorded",
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        source_chain_complete,
        artifact_load_boundary_source_evidence
            .1
            .load_attempt_boundary_present,
        artifact_load_boundary_source_evidence
            .1
            .load_attempt_boundary_source_chain_complete,
        artifact_load_boundary_present,
        artifact_load_boundary_source_evidence
            .1
            .source_chain_complete,
        false,
        false,
        false,
        false,
        artifact_load_boundary_source_evidence
            .1
            .artifact_byte_intake_boundary_present,
        artifact_load_boundary_source_evidence
            .1
            .artifact_byte_intake_boundary_source_chain_complete,
        execution_authorization_boundary_present,
        execution_authorization_boundary_source_evidence
            .1
            .source_chain_complete,
        false,
        false,
        artifact_load_boundary_source_evidence
            .1
            .service_slot_binding_source_evidence_present,
        artifact_load_boundary_source_evidence
            .1
            .health_state_hooks_source_evidence_present,
        artifact_load_boundary_source_evidence
            .1
            .artifact_hash_binding_present,
        execution_authorization_boundary_source_evidence
            .1
            .entrypoint_abi_source_evidence_present,
        address_space_source_evidence_present,
        memory_map_source_evidence_present,
        execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_present[5],
        execution_authorization_boundary_source_evidence
            .1
            .audit_rollback_write_boundary_source_evidence_present,
        retained_module_evidence_present,
        artifact_reference_event_id.is_some(),
        service_slot_reservation_event_id.is_some(),
        artifact_load_boundary_source_evidence
            .1
            .load_attempt_boundary_source_evidence_event_id,
        Some(artifact_load_boundary_source_evidence.0),
        None,
        None,
        artifact_load_boundary_source_evidence
            .1
            .artifact_byte_intake_boundary_source_evidence_event_id,
        Some(execution_authorization_boundary_source_evidence.0),
        None,
        artifact_load_boundary_source_evidence
            .1
            .service_slot_binding_source_evidence_event_id,
        artifact_load_boundary_source_evidence
            .1
            .health_state_hooks_source_evidence_event_id,
        artifact_load_boundary_source_evidence
            .1
            .artifact_hash_binding_source_evidence_event_id,
        execution_authorization_boundary_source_evidence
            .1
            .entrypoint_abi_source_evidence_event_id,
        address_space_source_evidence
            .map(|(event_id, _)| event_id)
            .or(execution_authorization_boundary_source_evidence
                .1
                .address_space_source_evidence_event_id),
        memory_map_source_evidence.map(|(event_id, _)| event_id).or(
            execution_authorization_boundary_source_evidence
                .1
                .memory_map_source_evidence_event_id,
        ),
        execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_event_ids[5],
        execution_authorization_boundary_source_evidence
            .1
            .audit_rollback_write_boundary_source_evidence_event_id,
        execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_event_ids,
        execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_present,
        execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_fact_present,
        manifest_reference_event_id,
        artifact_reference_event_id,
        vm_test_report_reference_event_id,
        local_attestation_reference_event_id,
        local_approval_reference_event_id,
        computed_grant_reference_event_id,
        audit_rollback_reference_event_id,
        service_slot_reservation_event_id,
        ram_only_service_slot_id,
    )
}

pub(super) fn module_loader_entrypoint_transfer_boundary_source_evidence(
    manifest_reference_event_id: Option<event_log::EventId>,
    artifact_reference_event_id: Option<event_log::EventId>,
    vm_test_report_reference_event_id: Option<event_log::EventId>,
    local_attestation_reference_event_id: Option<event_log::EventId>,
    local_approval_reference_event_id: Option<event_log::EventId>,
    computed_grant_reference_event_id: Option<event_log::EventId>,
    audit_rollback_reference_event_id: Option<event_log::EventId>,
    service_slot_reservation_event_id: Option<event_log::EventId>,
    ram_only_service_slot_id: Option<event_log::ModuleServiceSlotId>,
    executable_mapping_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    execution_authorization_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderExecutionAuthorizationBoundarySourceEvidence,
    ),
    entrypoint_abi_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    capability_table_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let retained_module_evidence_present = module_loader_retained_module_evidence_present(
        manifest_reference_event_id,
        artifact_reference_event_id,
        vm_test_report_reference_event_id,
        local_attestation_reference_event_id,
        local_approval_reference_event_id,
        computed_grant_reference_event_id,
        audit_rollback_reference_event_id,
        service_slot_reservation_event_id,
    );
    let executable_mapping_boundary_present = executable_mapping_boundary_source_evidence
        .1
        .boundary_present
        && executable_mapping_boundary_source_evidence
            .1
            .source_chain_complete;
    let execution_authorization_boundary_present = execution_authorization_boundary_source_evidence
        .1
        .boundary_present
        && execution_authorization_boundary_source_evidence
            .1
            .source_chain_complete;
    let entrypoint_abi_source_evidence_present = entrypoint_abi_source_evidence.is_some()
        || execution_authorization_boundary_source_evidence
            .1
            .entrypoint_abi_source_evidence_present;
    let capability_import_table_source_evidence_present = capability_table_source_evidence
        .is_some()
        || execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_present[5];
    let source_chain_complete = retained_module_evidence_present
        && executable_mapping_boundary_present
        && execution_authorization_boundary_present
        && entrypoint_abi_source_evidence_present
        && capability_import_table_source_evidence_present
        && service_slot_reservation_event_id.is_some();
    module_loader_live_load_boundary_source_evidence_record(
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SCHEMA,
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_ID,
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_entrypoint_transfer_boundary_source_evidence_recorded",
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_STATUS,
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_REASON,
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        source_chain_complete,
        executable_mapping_boundary_source_evidence
            .1
            .load_attempt_boundary_present,
        executable_mapping_boundary_source_evidence
            .1
            .load_attempt_boundary_source_chain_complete,
        executable_mapping_boundary_source_evidence
            .1
            .artifact_load_boundary_present,
        executable_mapping_boundary_source_evidence
            .1
            .artifact_load_boundary_source_chain_complete,
        executable_mapping_boundary_present,
        executable_mapping_boundary_source_evidence
            .1
            .source_chain_complete,
        false,
        false,
        executable_mapping_boundary_source_evidence
            .1
            .artifact_byte_intake_boundary_present,
        executable_mapping_boundary_source_evidence
            .1
            .artifact_byte_intake_boundary_source_chain_complete,
        execution_authorization_boundary_present,
        execution_authorization_boundary_source_evidence
            .1
            .source_chain_complete,
        false,
        false,
        executable_mapping_boundary_source_evidence
            .1
            .service_slot_binding_source_evidence_present,
        executable_mapping_boundary_source_evidence
            .1
            .health_state_hooks_source_evidence_present,
        executable_mapping_boundary_source_evidence
            .1
            .artifact_hash_binding_present,
        entrypoint_abi_source_evidence_present,
        executable_mapping_boundary_source_evidence
            .1
            .address_space_source_evidence_present,
        executable_mapping_boundary_source_evidence
            .1
            .memory_map_source_evidence_present,
        capability_import_table_source_evidence_present,
        executable_mapping_boundary_source_evidence
            .1
            .audit_rollback_write_boundary_source_evidence_present,
        retained_module_evidence_present,
        artifact_reference_event_id.is_some(),
        service_slot_reservation_event_id.is_some(),
        executable_mapping_boundary_source_evidence
            .1
            .load_attempt_boundary_source_evidence_event_id,
        executable_mapping_boundary_source_evidence
            .1
            .artifact_load_boundary_source_evidence_event_id,
        Some(executable_mapping_boundary_source_evidence.0),
        None,
        executable_mapping_boundary_source_evidence
            .1
            .artifact_byte_intake_boundary_source_evidence_event_id,
        Some(execution_authorization_boundary_source_evidence.0),
        None,
        executable_mapping_boundary_source_evidence
            .1
            .service_slot_binding_source_evidence_event_id,
        executable_mapping_boundary_source_evidence
            .1
            .health_state_hooks_source_evidence_event_id,
        executable_mapping_boundary_source_evidence
            .1
            .artifact_hash_binding_source_evidence_event_id,
        entrypoint_abi_source_evidence
            .map(|(event_id, _)| event_id)
            .or(execution_authorization_boundary_source_evidence
                .1
                .entrypoint_abi_source_evidence_event_id),
        executable_mapping_boundary_source_evidence
            .1
            .address_space_source_evidence_event_id,
        executable_mapping_boundary_source_evidence
            .1
            .memory_map_source_evidence_event_id,
        capability_table_source_evidence
            .map(|(event_id, _)| event_id)
            .or(execution_authorization_boundary_source_evidence
                .1
                .loader_runtime_source_evidence_event_ids[5]),
        executable_mapping_boundary_source_evidence
            .1
            .audit_rollback_write_boundary_source_evidence_event_id,
        execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_event_ids,
        execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_present,
        execution_authorization_boundary_source_evidence
            .1
            .loader_runtime_fact_present,
        manifest_reference_event_id,
        artifact_reference_event_id,
        vm_test_report_reference_event_id,
        local_attestation_reference_event_id,
        local_approval_reference_event_id,
        computed_grant_reference_event_id,
        audit_rollback_reference_event_id,
        service_slot_reservation_event_id,
        ram_only_service_slot_id,
    )
}
