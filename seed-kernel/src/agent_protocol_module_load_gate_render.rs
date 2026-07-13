use alloc::{vec, vec::Vec};
use core::sync::atomic::{AtomicU64, Ordering};

use crate::{
    agent_protocol,
    agent_protocol_memory::{
        define_direct_binding_fields, emit_binding_object_direct, BindingField,
    },
    agent_protocol_module_service_slot_allocator_projection::{
        latest_module_service_slot_allocator_readiness_projection,
        ModuleServiceSlotAuthorityInputProjection,
    },
    agent_protocol_module_types::*,
    agent_protocol_support::{
        crlf, emit_inline_record_object_fragment, emit_record_fields_trailing_comma,
        emit_record_property_at, emit_record_property_line_at, emit_record_value_fragment,
        json_event_id, json_event_id_option, json_sha256, json_str, method_eq, raw, raw_bool,
        raw_fmt, raw_line, record_bool as b, record_event_or_null, record_false as no,
        record_field as f, record_sha_or_null, record_str as s, record_str_or_null,
    },
    event_log, serial,
};
use raios_core::{
    evidence_response::{self as ev, Envelope},
    module_load_gate_projection::{
        project_load_gate_denial, LoadGateDisposition, LoadGateEvidenceStatus,
        LoadGatePrerequisite, LoadGateProjectionInput,
    },
    record::{Field, Value as V},
};

static NEXT_EVIDENCE_RESPONSE_ID: AtomicU64 = AtomicU64::new(1);
fn module_load_gate_manifest_state(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_manifest_reference_valid(binding) {
        "retained_hash_reference_only"
    } else if module_load_gate_manifest_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_manifest_reason(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_manifest_reference_valid(binding) {
        "retained_module_manifest_reference_not_authorizing"
    } else if module_load_gate_manifest_reference_rejected(binding) {
        binding.manifest_reference_reason
    } else {
        "module_manifest_missing"
    }
}

fn module_load_gate_candidate_artifact_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_candidate_artifact_reference_valid(binding) {
        "retained_hash_reference_only"
    } else if module_load_gate_candidate_artifact_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_candidate_artifact_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_candidate_artifact_reference_valid(binding) {
        "retained_candidate_artifact_reference_not_authorizing"
    } else if module_load_gate_candidate_artifact_reference_rejected(binding) {
        binding.artifact_reference_reason
    } else {
        "candidate_artifact_missing"
    }
}

fn module_load_gate_vm_test_report_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_vm_test_report_reference_valid(binding) {
        "retained_hash_reference_only"
    } else if module_load_gate_vm_test_report_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_vm_test_report_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_vm_test_report_reference_valid(binding) {
        "retained_vm_test_report_reference_not_authorizing"
    } else if module_load_gate_vm_test_report_reference_rejected(binding) {
        binding.vm_report_reference_reason
    } else {
        "vm_test_report_missing"
    }
}

fn module_load_gate_local_attestation_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_local_attestation_reference_valid(binding) {
        "retained_hash_reference_only"
    } else if module_load_gate_local_attestation_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_local_attestation_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_local_attestation_reference_valid(binding) {
        "retained_local_attestation_reference_not_authorizing"
    } else if module_load_gate_local_attestation_reference_rejected(binding) {
        binding.attestation_reference_reason
    } else {
        "local_attestation_missing"
    }
}

fn module_load_gate_local_approval_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_local_approval_reference_valid(binding) {
        "retained_hash_reference_only"
    } else if module_load_gate_local_approval_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_local_approval_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_local_approval_reference_valid(binding) {
        "retained_local_approval_reference_not_authorizing"
    } else if module_load_gate_local_approval_reference_rejected(binding) {
        binding.approval_reference_reason
    } else {
        "local_approval_missing"
    }
}

fn module_load_gate_computed_grant_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if binding.retained_reference.is_some() {
        "retained_hash_reference_only"
    } else {
        "missing"
    }
}

fn module_load_gate_computed_grant_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if binding.retained_reference.is_some() {
        "retained_computed_grant_reference_not_authorizing"
    } else {
        "computed_capability_grant_missing"
    }
}

fn module_load_gate_durable_audit_state(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_audit_rollback_reference_valid(binding) {
        "retained_hash_reference_only_not_durable"
    } else if module_load_gate_audit_rollback_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_durable_audit_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_audit_rollback_reference_valid(binding) {
        "durable_audit_write_missing"
    } else if module_load_gate_audit_rollback_reference_rejected(binding) {
        binding.audit_rollback_reference_reason
    } else {
        "durable_audit_write_missing"
    }
}

fn module_load_gate_rollback_state(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_audit_rollback_reference_valid(binding) {
        "retained_hash_reference_only_not_installed"
    } else if module_load_gate_audit_rollback_reference_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "missing"
    }
}

fn module_load_gate_rollback_reason(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_audit_rollback_reference_valid(binding) {
        "rollback_install_missing"
    } else if module_load_gate_audit_rollback_reference_rejected(binding) {
        binding.audit_rollback_reference_reason
    } else {
        "rollback_install_missing"
    }
}

fn module_load_gate_service_slot_state(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_service_slot_reservation_valid(binding) {
        "retained_hash_reference_only_not_allocated"
    } else if module_load_gate_service_slot_reservation_rejected(binding) {
        "rejected_retained_reference"
    } else {
        "unallocated"
    }
}

fn module_load_gate_service_slot_reason(binding: event_log::ModuleLoadGateBinding) -> &'static str {
    if module_load_gate_service_slot_reservation_valid(binding) {
        "retained_service_slot_reservation_not_allocated"
    } else if module_load_gate_service_slot_reservation_rejected(binding) {
        binding.service_slot_reservation_reason
    } else {
        "ram_only_service_slot_unallocated"
    }
}

fn module_load_gate_retained_module_evidence_complete(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    module_load_gate_manifest_reference_valid(binding)
        && module_load_gate_candidate_artifact_reference_valid(binding)
        && module_load_gate_vm_test_report_reference_valid(binding)
        && module_load_gate_local_attestation_reference_valid(binding)
        && module_load_gate_local_approval_reference_valid(binding)
        && binding.retained_reference.is_some()
        && module_load_gate_audit_rollback_reference_valid(binding)
        && module_load_gate_service_slot_reservation_valid(binding)
}

fn module_load_gate_retained_module_evidence_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    module_load_gate_manifest_reference_rejected(binding)
        || module_load_gate_candidate_artifact_reference_rejected(binding)
        || module_load_gate_vm_test_report_reference_rejected(binding)
        || module_load_gate_local_attestation_reference_rejected(binding)
        || module_load_gate_local_approval_reference_rejected(binding)
        || module_load_gate_audit_rollback_reference_rejected(binding)
        || module_load_gate_service_slot_reservation_rejected(binding)
}

fn module_load_gate_retained_module_evidence_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_retained_module_evidence_complete(binding) {
        "available"
    } else if module_load_gate_retained_module_evidence_rejected(binding) {
        "rejected"
    } else {
        "missing"
    }
}

fn module_load_gate_retained_module_evidence_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if !module_load_gate_manifest_reference_valid(binding) {
        return module_load_gate_manifest_reason(binding);
    }
    if !module_load_gate_candidate_artifact_reference_valid(binding) {
        return module_load_gate_candidate_artifact_reason(binding);
    }
    if !module_load_gate_vm_test_report_reference_valid(binding) {
        return module_load_gate_vm_test_report_reason(binding);
    }
    if !module_load_gate_local_attestation_reference_valid(binding) {
        return module_load_gate_local_attestation_reason(binding);
    }
    if !module_load_gate_local_approval_reference_valid(binding) {
        return module_load_gate_local_approval_reason(binding);
    }
    if binding.retained_reference.is_none() {
        return module_load_gate_computed_grant_reason(binding);
    }
    if !module_load_gate_audit_rollback_reference_valid(binding) {
        return module_load_gate_rollback_reason(binding);
    }
    if !module_load_gate_service_slot_reservation_valid(binding) {
        return module_load_gate_service_slot_reason(binding);
    }
    "retained_module_evidence_available"
}

fn module_load_gate_service_slot_allocator_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    module_load_gate_service_slot_allocator_projection(binding).state
}

fn module_load_gate_service_slot_allocator_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    module_load_gate_service_slot_allocator_projection(binding).reason
}

fn module_load_gate_loader_runtime_state(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if !module_load_gate_retained_module_evidence_complete(binding) {
        "blocked_by_retained_module_evidence"
    } else if method_eq(
        module_load_gate_service_slot_allocator_projection(binding).status,
        MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
    ) {
        "blocked_by_service_slot_allocator_authority"
    } else {
        "blocked_by_service_slot_allocator_readiness"
    }
}

fn module_load_gate_loader_runtime_status(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_retained_module_evidence_complete(binding) {
        module_load_gate_service_slot_allocator_projection(binding).status
    } else {
        "denied_missing_retained_module_evidence"
    }
}

fn module_load_gate_loader_runtime_reason(
    binding: event_log::ModuleLoadGateBinding,
) -> &'static str {
    if module_load_gate_retained_module_evidence_complete(binding) {
        module_load_gate_service_slot_allocator_projection(binding).reason
    } else {
        module_load_gate_retained_module_evidence_reason(binding)
    }
}

#[derive(Clone, Copy)]
struct ModuleLoadGateServiceSlotAllocatorProjection {
    state: &'static str,
    status: &'static str,
    reason: &'static str,
    ready: bool,
    authority_source_evidence_event_id: Option<event_log::EventId>,
    authority_status: &'static str,
    authority_reason: &'static str,
    allocation_intent_source_evidence_event_id: Option<event_log::EventId>,
    allocation_intent_status: &'static str,
    allocation_intent_reason: &'static str,
    allocation_intent_present: bool,
    authority_inputs:
        [ModuleServiceSlotAuthorityInputProjection; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
    authority_decision_source_evidence_event_id: Option<event_log::EventId>,
    authority_decision_status: &'static str,
    authority_decision_reason: &'static str,
    authority_decision_present: bool,
    registry_write_commit_gate_source_evidence_event_id: Option<event_log::EventId>,
    registry_write_commit_gate_status: &'static str,
    registry_write_commit_gate_reason: &'static str,
    registry_write_commit_gate_present: bool,
}

#[derive(Clone, Copy)]
struct ModuleLoadGateLoaderRuntimeExecutionCommitGateProjection {
    source_evidence_event_id: Option<event_log::EventId>,
    status: &'static str,
    reason: &'static str,
    present: bool,
    source_chain_complete: bool,
    authority_decision_present: bool,
    loader_runtime_contract_present: bool,
    loader_runtime_source_evidence_complete: bool,
    service_slot_binding_source_evidence_present: bool,
    service_slot_binding_fact_present: bool,
    audit_rollback_write_boundary_source_evidence_present: bool,
    audit_rollback_write_boundary_fact_present: bool,
    retained_service_slot_reservation_present: bool,
}

#[derive(Clone, Copy)]
struct ModuleLoadGateLoaderDescriptorIntakeBoundaryProjection {
    source_evidence_event_id: Option<event_log::EventId>,
    status: &'static str,
    reason: &'static str,
    present: bool,
    source_chain_complete: bool,
    registry_write_commit_gate_present: bool,
    execution_commit_gate_present: bool,
    loader_runtime_source_evidence_complete: bool,
    retained_module_evidence_present: bool,
    retained_service_slot_reservation_present: bool,
}

#[derive(Clone, Copy)]
struct ModuleLoadGateLoaderArtifactByteIntakeBoundaryProjection {
    source_evidence_event_id: Option<event_log::EventId>,
    status: &'static str,
    reason: &'static str,
    present: bool,
    source_chain_complete: bool,
    descriptor_intake_boundary_present: bool,
    descriptor_intake_boundary_source_chain_complete: bool,
    execution_commit_gate_present: bool,
    artifact_hash_binding_present: bool,
    retained_artifact_reference_present: bool,
    retained_module_evidence_present: bool,
    retained_service_slot_reservation_present: bool,
}

#[derive(Clone, Copy)]
struct ModuleLoadGateLoaderExecutionAuthorizationBoundaryProjection {
    source_evidence_event_id: Option<event_log::EventId>,
    status: &'static str,
    reason: &'static str,
    present: bool,
    source_chain_complete: bool,
    artifact_byte_intake_boundary_present: bool,
    artifact_byte_intake_boundary_source_chain_complete: bool,
    descriptor_intake_boundary_present: bool,
    descriptor_intake_boundary_source_chain_complete: bool,
    execution_commit_gate_present: bool,
    entrypoint_abi_source_evidence_present: bool,
    address_space_source_evidence_present: bool,
    memory_map_source_evidence_present: bool,
    audit_rollback_write_boundary_source_evidence_present: bool,
    retained_module_evidence_present: bool,
    retained_service_slot_reservation_present: bool,
}

#[derive(Clone, Copy)]
struct ModuleLoadGateLoaderServiceRegistryMutationBoundaryProjection {
    source_evidence_event_id: Option<event_log::EventId>,
    status: &'static str,
    reason: &'static str,
    present: bool,
    source_chain_complete: bool,
    execution_authorization_boundary_present: bool,
    execution_authorization_boundary_source_chain_complete: bool,
    registry_write_commit_gate_present: bool,
    service_slot_binding_source_evidence_present: bool,
    retained_module_evidence_present: bool,
    retained_service_slot_reservation_present: bool,
}

#[derive(Clone, Copy)]
struct ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    source_evidence_event_id: Option<event_log::EventId>,
    status: &'static str,
    reason: &'static str,
    present: bool,
    source_chain_complete: bool,
    load_attempt_boundary_present: bool,
    load_attempt_boundary_source_chain_complete: bool,
    artifact_load_boundary_present: bool,
    artifact_load_boundary_source_chain_complete: bool,
    executable_mapping_boundary_present: bool,
    executable_mapping_boundary_source_chain_complete: bool,
    entrypoint_transfer_boundary_present: bool,
    entrypoint_transfer_boundary_source_chain_complete: bool,
    service_start_boundary_present: bool,
    service_start_boundary_source_chain_complete: bool,
    service_health_binding_boundary_present: bool,
    service_health_binding_boundary_source_chain_complete: bool,
    service_running_state_boundary_present: bool,
    service_running_state_boundary_source_chain_complete: bool,
    service_start_audit_boundary_present: bool,
    service_start_audit_boundary_source_chain_complete: bool,
    service_unload_cleanup_boundary_present: bool,
    service_unload_cleanup_boundary_source_chain_complete: bool,
    live_load_commit_boundary_present: bool,
    live_load_commit_boundary_source_chain_complete: bool,
    commit_audit_boundary_present: bool,
    commit_audit_boundary_source_chain_complete: bool,
    commit_rollback_boundary_present: bool,
    commit_rollback_boundary_source_chain_complete: bool,
    commit_result_boundary_present: bool,
    commit_result_boundary_source_chain_complete: bool,
    descriptor_acceptance_authority_boundary_present: bool,
    descriptor_acceptance_authority_boundary_source_chain_complete: bool,
    descriptor_parser_contract_boundary_present: bool,
    descriptor_parser_contract_boundary_source_chain_complete: bool,
    descriptor_parser_result_boundary_present: bool,
    descriptor_parser_result_boundary_source_chain_complete: bool,
    descriptor_schema_validation_boundary_present: bool,
    descriptor_schema_validation_boundary_source_chain_complete: bool,
    descriptor_capability_validation_boundary_present: bool,
    descriptor_capability_validation_boundary_source_chain_complete: bool,
    descriptor_load_plan_boundary_present: bool,
    descriptor_load_plan_boundary_source_chain_complete: bool,
    executable_load_plan_authority_boundary_present: bool,
    executable_load_plan_authority_boundary_source_chain_complete: bool,
    executable_load_plan_result_boundary_present: bool,
    executable_load_plan_result_boundary_source_chain_complete: bool,
    executable_image_layout_boundary_present: bool,
    executable_image_layout_boundary_source_chain_complete: bool,
    executable_page_mapping_plan_boundary_present: bool,
    executable_page_mapping_plan_boundary_source_chain_complete: bool,
    executable_page_mapping_boundary_present: bool,
    executable_page_mapping_boundary_source_chain_complete: bool,
    descriptor_executable_page_binding_boundary_present: bool,
    descriptor_executable_page_binding_boundary_source_chain_complete: bool,
    executable_entrypoint_binding_boundary_present: bool,
    executable_entrypoint_binding_boundary_source_chain_complete: bool,
    executable_entrypoint_transfer_authorization_boundary_present: bool,
    executable_entrypoint_transfer_authorization_boundary_source_chain_complete: bool,
    executable_entrypoint_transfer_boundary_present: bool,
    executable_entrypoint_transfer_boundary_source_chain_complete: bool,
    executable_entrypoint_handoff_boundary_present: bool,
    executable_entrypoint_handoff_boundary_source_chain_complete: bool,
    artifact_byte_intake_boundary_present: bool,
    artifact_byte_intake_boundary_source_chain_complete: bool,
    execution_authorization_boundary_present: bool,
    execution_authorization_boundary_source_chain_complete: bool,
    service_registry_mutation_boundary_present: bool,
    service_registry_mutation_boundary_source_chain_complete: bool,
    service_slot_binding_source_evidence_present: bool,
    health_state_hooks_source_evidence_present: bool,
    artifact_hash_binding_present: bool,
    entrypoint_abi_source_evidence_present: bool,
    address_space_source_evidence_present: bool,
    memory_map_source_evidence_present: bool,
    capability_import_table_source_evidence_present: bool,
    audit_rollback_write_boundary_source_evidence_present: bool,
    retained_module_evidence_present: bool,
    retained_artifact_reference_present: bool,
    retained_service_slot_reservation_present: bool,
}

fn module_load_gate_loader_runtime_execution_commit_gate_projection(
) -> ModuleLoadGateLoaderRuntimeExecutionCommitGateProjection {
    let latest = event_log::latest_module_loader_runtime_execution_commit_gate_source_evidence();
    if let Some((event_id, evidence)) = latest {
        return ModuleLoadGateLoaderRuntimeExecutionCommitGateProjection {
            source_evidence_event_id: Some(event_id),
            status: evidence.gate_status,
            reason: evidence.gate_reason,
            present: evidence.gate_present,
            source_chain_complete: evidence.source_chain_complete,
            authority_decision_present: evidence.authority_decision_present,
            loader_runtime_contract_present: evidence.loader_runtime_contract_present,
            loader_runtime_source_evidence_complete: evidence
                .loader_runtime_source_evidence_complete,
            service_slot_binding_source_evidence_present: evidence
                .service_slot_binding_source_evidence_present,
            service_slot_binding_fact_present: evidence.service_slot_binding_fact_present,
            audit_rollback_write_boundary_source_evidence_present: evidence
                .audit_rollback_write_boundary_source_evidence_present,
            audit_rollback_write_boundary_fact_present: evidence
                .audit_rollback_write_boundary_fact_present,
            retained_service_slot_reservation_present: evidence
                .retained_service_slot_reservation_present,
        };
    }
    ModuleLoadGateLoaderRuntimeExecutionCommitGateProjection {
        source_evidence_event_id: None,
        status: MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_MISSING_STATUS,
        reason: MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
        present: false,
        source_chain_complete: false,
        authority_decision_present: false,
        loader_runtime_contract_present: false,
        loader_runtime_source_evidence_complete: false,
        service_slot_binding_source_evidence_present: false,
        service_slot_binding_fact_present: false,
        audit_rollback_write_boundary_source_evidence_present: false,
        audit_rollback_write_boundary_fact_present: false,
        retained_service_slot_reservation_present: false,
    }
}

fn module_load_gate_loader_descriptor_intake_boundary_projection(
) -> ModuleLoadGateLoaderDescriptorIntakeBoundaryProjection {
    let latest = event_log::latest_module_loader_descriptor_intake_boundary_source_evidence();
    if let Some((event_id, evidence)) = latest {
        return ModuleLoadGateLoaderDescriptorIntakeBoundaryProjection {
            source_evidence_event_id: Some(event_id),
            status: evidence.boundary_status,
            reason: evidence.boundary_reason,
            present: evidence.boundary_present,
            source_chain_complete: evidence.source_chain_complete,
            registry_write_commit_gate_present: evidence.registry_write_commit_gate_present,
            execution_commit_gate_present: evidence.execution_commit_gate_present,
            loader_runtime_source_evidence_complete: evidence
                .loader_runtime_source_evidence_complete,
            retained_module_evidence_present: evidence.retained_module_evidence_present,
            retained_service_slot_reservation_present: evidence
                .retained_service_slot_reservation_present,
        };
    }
    ModuleLoadGateLoaderDescriptorIntakeBoundaryProjection {
        source_evidence_event_id: None,
        status: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_MISSING_STATUS,
        reason: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,

        present: false,
        source_chain_complete: false,
        registry_write_commit_gate_present: false,
        execution_commit_gate_present: false,
        loader_runtime_source_evidence_complete: false,
        retained_module_evidence_present: false,
        retained_service_slot_reservation_present: false,
    }
}

fn module_load_gate_loader_artifact_byte_intake_boundary_projection(
) -> ModuleLoadGateLoaderArtifactByteIntakeBoundaryProjection {
    let latest = event_log::latest_module_loader_artifact_byte_intake_boundary_source_evidence();
    if let Some((event_id, evidence)) = latest {
        return ModuleLoadGateLoaderArtifactByteIntakeBoundaryProjection {
            source_evidence_event_id: Some(event_id),
            status: evidence.boundary_status,
            reason: evidence.boundary_reason,
            present: evidence.boundary_present,
            source_chain_complete: evidence.source_chain_complete,
            descriptor_intake_boundary_present: evidence.descriptor_intake_boundary_present,
            descriptor_intake_boundary_source_chain_complete: evidence
                .descriptor_intake_boundary_source_chain_complete,
            execution_commit_gate_present: evidence.execution_commit_gate_present,
            artifact_hash_binding_present: evidence.artifact_hash_binding_present,
            retained_artifact_reference_present: evidence.retained_artifact_reference_present,
            retained_module_evidence_present: evidence.retained_module_evidence_present,
            retained_service_slot_reservation_present: evidence
                .retained_service_slot_reservation_present,
        };
    }
    ModuleLoadGateLoaderArtifactByteIntakeBoundaryProjection {
        source_evidence_event_id: None,
        status: MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_MISSING_STATUS,
        reason: MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
        present: false,
        source_chain_complete: false,
        descriptor_intake_boundary_present: false,
        descriptor_intake_boundary_source_chain_complete: false,
        execution_commit_gate_present: false,
        artifact_hash_binding_present: false,
        retained_artifact_reference_present: false,
        retained_module_evidence_present: false,
        retained_service_slot_reservation_present: false,
    }
}

fn module_load_gate_loader_execution_authorization_boundary_projection(
) -> ModuleLoadGateLoaderExecutionAuthorizationBoundaryProjection {
    let latest = event_log::latest_module_loader_execution_authorization_boundary_source_evidence();
    if let Some((event_id, evidence)) = latest {
        return ModuleLoadGateLoaderExecutionAuthorizationBoundaryProjection {
            source_evidence_event_id: Some(event_id),
            status: evidence.boundary_status,
            reason: evidence.boundary_reason,
            present: evidence.boundary_present,
            source_chain_complete: evidence.source_chain_complete,
            artifact_byte_intake_boundary_present: evidence.artifact_byte_intake_boundary_present,
            artifact_byte_intake_boundary_source_chain_complete: evidence
                .artifact_byte_intake_boundary_source_chain_complete,
            descriptor_intake_boundary_present: evidence.descriptor_intake_boundary_present,
            descriptor_intake_boundary_source_chain_complete: evidence
                .descriptor_intake_boundary_source_chain_complete,
            execution_commit_gate_present: evidence.execution_commit_gate_present,
            entrypoint_abi_source_evidence_present: evidence.entrypoint_abi_source_evidence_present,
            address_space_source_evidence_present: evidence.address_space_source_evidence_present,
            memory_map_source_evidence_present: evidence.memory_map_source_evidence_present,
            audit_rollback_write_boundary_source_evidence_present: evidence
                .audit_rollback_write_boundary_source_evidence_present,
            retained_module_evidence_present: evidence.retained_module_evidence_present,
            retained_service_slot_reservation_present: evidence
                .retained_service_slot_reservation_present,
        };
    }
    ModuleLoadGateLoaderExecutionAuthorizationBoundaryProjection {
        source_evidence_event_id: None,
        status: MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_MISSING_STATUS,
        reason: MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
        present: false,
        source_chain_complete: false,
        artifact_byte_intake_boundary_present: false,
        artifact_byte_intake_boundary_source_chain_complete: false,
        descriptor_intake_boundary_present: false,
        descriptor_intake_boundary_source_chain_complete: false,
        execution_commit_gate_present: false,
        entrypoint_abi_source_evidence_present: false,
        address_space_source_evidence_present: false,
        memory_map_source_evidence_present: false,
        audit_rollback_write_boundary_source_evidence_present: false,
        retained_module_evidence_present: false,
        retained_service_slot_reservation_present: false,
    }
}

fn module_load_gate_loader_service_registry_mutation_boundary_projection(
) -> ModuleLoadGateLoaderServiceRegistryMutationBoundaryProjection {
    let latest =
        event_log::latest_module_loader_service_registry_mutation_boundary_source_evidence();
    if let Some((event_id, evidence)) = latest {
        return ModuleLoadGateLoaderServiceRegistryMutationBoundaryProjection {
            source_evidence_event_id: Some(event_id),
            status: evidence.boundary_status,
            reason: evidence.boundary_reason,
            present: evidence.boundary_present,
            source_chain_complete: evidence.source_chain_complete,
            execution_authorization_boundary_present: evidence
                .execution_authorization_boundary_present,
            execution_authorization_boundary_source_chain_complete: evidence
                .execution_authorization_boundary_source_chain_complete,
            registry_write_commit_gate_present: evidence.registry_write_commit_gate_present,
            service_slot_binding_source_evidence_present: evidence
                .service_slot_binding_source_evidence_present,
            retained_module_evidence_present: evidence.retained_module_evidence_present,
            retained_service_slot_reservation_present: evidence
                .retained_service_slot_reservation_present,
        };
    }
    ModuleLoadGateLoaderServiceRegistryMutationBoundaryProjection {
        source_evidence_event_id: None,
        status: MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_MISSING_STATUS,
        reason: MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
        present: false,
        source_chain_complete: false,
        execution_authorization_boundary_present: false,
        execution_authorization_boundary_source_chain_complete: false,
        registry_write_commit_gate_present: false,
        service_slot_binding_source_evidence_present: false,
        retained_module_evidence_present: false,
        retained_service_slot_reservation_present: false,
    }
}

fn module_load_gate_loader_load_attempt_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_load_attempt_boundary_source_evidence(),
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_artifact_load_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_artifact_load_boundary_source_evidence(),
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_executable_mapping_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_executable_mapping_boundary_source_evidence(),
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_entrypoint_transfer_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_entrypoint_transfer_boundary_source_evidence(),
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_service_start_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_service_start_boundary_source_evidence(),
        MODULE_LOADER_SERVICE_START_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_service_health_binding_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_service_health_binding_boundary_source_evidence(),
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_service_running_state_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_service_running_state_boundary_source_evidence(),
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_service_start_audit_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_service_start_audit_boundary_source_evidence(),
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_service_unload_cleanup_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_service_unload_cleanup_boundary_source_evidence(),
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_live_load_commit_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_live_load_commit_boundary_source_evidence(),
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_commit_audit_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_commit_audit_boundary_source_evidence(),
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_commit_rollback_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_commit_rollback_boundary_source_evidence(),
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_commit_result_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_commit_result_boundary_source_evidence(),
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_descriptor_acceptance_authority_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_descriptor_acceptance_authority_boundary_source_evidence(),
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_descriptor_parser_contract_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_descriptor_parser_contract_boundary_source_evidence(),
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_descriptor_parser_result_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_descriptor_parser_result_boundary_source_evidence(),
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_descriptor_schema_validation_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_descriptor_schema_validation_boundary_source_evidence(),
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_descriptor_capability_validation_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_descriptor_capability_validation_boundary_source_evidence(),
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}
fn module_load_gate_loader_descriptor_load_plan_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_descriptor_load_plan_boundary_source_evidence(),
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_executable_load_plan_authority_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_executable_load_plan_authority_boundary_source_evidence(),
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_executable_load_plan_result_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_executable_load_plan_result_boundary_source_evidence(),
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_executable_image_layout_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_executable_image_layout_boundary_source_evidence(),
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_executable_page_mapping_plan_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_executable_page_mapping_plan_boundary_source_evidence(),
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_executable_page_mapping_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_executable_page_mapping_boundary_source_evidence(),
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_descriptor_executable_page_binding_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_descriptor_executable_page_binding_boundary_source_evidence(
        ),
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_executable_entrypoint_binding_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_executable_entrypoint_binding_boundary_source_evidence(),
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_executable_entrypoint_transfer_authorization_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_executable_entrypoint_transfer_authorization_boundary_source_evidence(),
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_executable_entrypoint_transfer_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_executable_entrypoint_transfer_boundary_source_evidence(),
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_executable_entrypoint_handoff_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_executable_entrypoint_handoff_boundary_source_evidence(),
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_executable_entrypoint_invocation_boundary_projection(
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    module_load_gate_loader_live_load_boundary_projection(
        event_log::latest_module_loader_executable_entrypoint_invocation_boundary_source_evidence(),
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    )
}

fn module_load_gate_loader_live_load_boundary_projection(
    latest: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    missing_status: &'static str,
    missing_reason: &'static str,
) -> ModuleLoadGateLoaderLiveLoadBoundaryProjection {
    if let Some((event_id, evidence)) = latest {
        return ModuleLoadGateLoaderLiveLoadBoundaryProjection {
            source_evidence_event_id: Some(event_id),
            status: evidence.boundary_status,
            reason: evidence.boundary_reason,
            present: evidence.boundary_present,
            source_chain_complete: evidence.source_chain_complete,
            load_attempt_boundary_present: evidence.load_attempt_boundary_present,
            load_attempt_boundary_source_chain_complete: evidence
                .load_attempt_boundary_source_chain_complete,
            artifact_load_boundary_present: evidence.artifact_load_boundary_present,
            artifact_load_boundary_source_chain_complete: evidence
                .artifact_load_boundary_source_chain_complete,
            executable_mapping_boundary_present: evidence.executable_mapping_boundary_present,
            executable_mapping_boundary_source_chain_complete: evidence
                .executable_mapping_boundary_source_chain_complete,
            entrypoint_transfer_boundary_present: evidence.entrypoint_transfer_boundary_present,
            entrypoint_transfer_boundary_source_chain_complete: evidence
                .entrypoint_transfer_boundary_source_chain_complete,
            service_start_boundary_present: evidence.service_start_boundary_present,
            service_start_boundary_source_chain_complete: evidence
                .service_start_boundary_source_chain_complete,
            service_health_binding_boundary_present: evidence
                .service_health_binding_boundary_present,
            service_health_binding_boundary_source_chain_complete: evidence
                .service_health_binding_boundary_source_chain_complete,
            service_running_state_boundary_present: evidence.service_running_state_boundary_present,
            service_running_state_boundary_source_chain_complete: evidence
                .service_running_state_boundary_source_chain_complete,
            service_start_audit_boundary_present: evidence.service_start_audit_boundary_present,
            service_start_audit_boundary_source_chain_complete: evidence
                .service_start_audit_boundary_source_chain_complete,
            service_unload_cleanup_boundary_present: evidence
                .service_unload_cleanup_boundary_present,
            service_unload_cleanup_boundary_source_chain_complete: evidence
                .service_unload_cleanup_boundary_source_chain_complete,
            live_load_commit_boundary_present: evidence.live_load_commit_boundary_present,
            live_load_commit_boundary_source_chain_complete: evidence
                .live_load_commit_boundary_source_chain_complete,

            commit_audit_boundary_present: evidence.commit_audit_boundary_present,
            commit_audit_boundary_source_chain_complete: evidence
                .commit_audit_boundary_source_chain_complete,
            commit_rollback_boundary_present: evidence.commit_rollback_boundary_present,
            commit_rollback_boundary_source_chain_complete: evidence
                .commit_rollback_boundary_source_chain_complete,
            commit_result_boundary_present: evidence.commit_result_boundary_present,
            commit_result_boundary_source_chain_complete: evidence
                .commit_result_boundary_source_chain_complete,
            descriptor_acceptance_authority_boundary_present: evidence
                .descriptor_acceptance_authority_boundary_present,
            descriptor_acceptance_authority_boundary_source_chain_complete: evidence
                .descriptor_acceptance_authority_boundary_source_chain_complete,
            descriptor_parser_contract_boundary_present: evidence
                .descriptor_parser_contract_boundary_present,
            descriptor_parser_contract_boundary_source_chain_complete: evidence
                .descriptor_parser_contract_boundary_source_chain_complete,
            descriptor_parser_result_boundary_present: evidence
                .descriptor_parser_result_boundary_present,
            descriptor_parser_result_boundary_source_chain_complete: evidence
                .descriptor_parser_result_boundary_source_chain_complete,
            descriptor_schema_validation_boundary_present: evidence
                .descriptor_schema_validation_boundary_present,
            descriptor_schema_validation_boundary_source_chain_complete: evidence
                .descriptor_schema_validation_boundary_source_chain_complete,
            descriptor_capability_validation_boundary_present: evidence
                .descriptor_capability_validation_boundary_present,
            descriptor_capability_validation_boundary_source_chain_complete: evidence
                .descriptor_capability_validation_boundary_source_chain_complete,
            descriptor_load_plan_boundary_present: evidence.descriptor_load_plan_boundary_present,
            descriptor_load_plan_boundary_source_chain_complete: evidence
                .descriptor_load_plan_boundary_source_chain_complete,
            executable_load_plan_authority_boundary_present: evidence
                .executable_load_plan_authority_boundary_present,
            executable_load_plan_authority_boundary_source_chain_complete: evidence
                .executable_load_plan_authority_boundary_source_chain_complete,
            executable_load_plan_result_boundary_present: evidence
                .executable_load_plan_result_boundary_present,
            executable_load_plan_result_boundary_source_chain_complete: evidence
                .executable_load_plan_result_boundary_source_chain_complete,
            executable_image_layout_boundary_present: evidence
                .executable_image_layout_boundary_present,
            executable_image_layout_boundary_source_chain_complete: evidence
                .executable_image_layout_boundary_source_chain_complete,
            executable_page_mapping_plan_boundary_present: evidence
                .executable_page_mapping_plan_boundary_present,
            executable_page_mapping_plan_boundary_source_chain_complete: evidence
                .executable_page_mapping_plan_boundary_source_chain_complete,
            executable_page_mapping_boundary_present: evidence
                .executable_page_mapping_boundary_present,
            executable_page_mapping_boundary_source_chain_complete: evidence
                .executable_page_mapping_boundary_source_chain_complete,
            descriptor_executable_page_binding_boundary_present: evidence
                .descriptor_executable_page_binding_boundary_present,
            descriptor_executable_page_binding_boundary_source_chain_complete: evidence
                .descriptor_executable_page_binding_boundary_source_chain_complete,
            executable_entrypoint_binding_boundary_present: evidence
                .executable_entrypoint_binding_boundary_present,
            executable_entrypoint_binding_boundary_source_chain_complete: evidence
                .executable_entrypoint_binding_boundary_source_chain_complete,
            executable_entrypoint_transfer_authorization_boundary_present: evidence
                .executable_entrypoint_transfer_authorization_boundary_present,
            executable_entrypoint_transfer_authorization_boundary_source_chain_complete: evidence
                .executable_entrypoint_transfer_authorization_boundary_source_chain_complete,
            executable_entrypoint_transfer_boundary_present: evidence
                .executable_entrypoint_transfer_boundary_present,
            executable_entrypoint_transfer_boundary_source_chain_complete: evidence
                .executable_entrypoint_transfer_boundary_source_chain_complete,
            executable_entrypoint_handoff_boundary_present: evidence
                .executable_entrypoint_handoff_boundary_present,
            executable_entrypoint_handoff_boundary_source_chain_complete: evidence
                .executable_entrypoint_handoff_boundary_source_chain_complete,
            artifact_byte_intake_boundary_present: evidence.artifact_byte_intake_boundary_present,
            artifact_byte_intake_boundary_source_chain_complete: evidence
                .artifact_byte_intake_boundary_source_chain_complete,
            execution_authorization_boundary_present: evidence
                .execution_authorization_boundary_present,
            execution_authorization_boundary_source_chain_complete: evidence
                .execution_authorization_boundary_source_chain_complete,
            service_registry_mutation_boundary_present: evidence
                .service_registry_mutation_boundary_present,
            service_registry_mutation_boundary_source_chain_complete: evidence
                .service_registry_mutation_boundary_source_chain_complete,
            service_slot_binding_source_evidence_present: evidence
                .service_slot_binding_source_evidence_present,
            health_state_hooks_source_evidence_present: evidence
                .health_state_hooks_source_evidence_present,
            artifact_hash_binding_present: evidence.artifact_hash_binding_present,
            entrypoint_abi_source_evidence_present: evidence.entrypoint_abi_source_evidence_present,
            address_space_source_evidence_present: evidence.address_space_source_evidence_present,
            memory_map_source_evidence_present: evidence.memory_map_source_evidence_present,
            capability_import_table_source_evidence_present: evidence
                .capability_import_table_source_evidence_present,
            audit_rollback_write_boundary_source_evidence_present: evidence
                .audit_rollback_write_boundary_source_evidence_present,
            retained_module_evidence_present: evidence.retained_module_evidence_present,
            retained_artifact_reference_present: evidence.retained_artifact_reference_present,
            retained_service_slot_reservation_present: evidence
                .retained_service_slot_reservation_present,
        };
    }
    ModuleLoadGateLoaderLiveLoadBoundaryProjection {
        source_evidence_event_id: None,
        status: missing_status,
        reason: missing_reason,
        present: false,
        source_chain_complete: false,
        load_attempt_boundary_present: false,
        load_attempt_boundary_source_chain_complete: false,
        artifact_load_boundary_present: false,
        artifact_load_boundary_source_chain_complete: false,
        executable_mapping_boundary_present: false,
        executable_mapping_boundary_source_chain_complete: false,
        entrypoint_transfer_boundary_present: false,
        entrypoint_transfer_boundary_source_chain_complete: false,
        service_start_boundary_present: false,
        service_start_boundary_source_chain_complete: false,
        service_health_binding_boundary_present: false,
        service_health_binding_boundary_source_chain_complete: false,
        service_running_state_boundary_present: false,
        service_running_state_boundary_source_chain_complete: false,
        service_start_audit_boundary_present: false,
        service_start_audit_boundary_source_chain_complete: false,
        service_unload_cleanup_boundary_present: false,
        service_unload_cleanup_boundary_source_chain_complete: false,
        live_load_commit_boundary_present: false,
        live_load_commit_boundary_source_chain_complete: false,
        commit_audit_boundary_present: false,
        commit_audit_boundary_source_chain_complete: false,
        commit_rollback_boundary_present: false,
        commit_rollback_boundary_source_chain_complete: false,
        commit_result_boundary_present: false,
        commit_result_boundary_source_chain_complete: false,
        descriptor_acceptance_authority_boundary_present: false,
        descriptor_acceptance_authority_boundary_source_chain_complete: false,
        descriptor_parser_contract_boundary_present: false,
        descriptor_parser_contract_boundary_source_chain_complete: false,
        descriptor_parser_result_boundary_present: false,
        descriptor_parser_result_boundary_source_chain_complete: false,
        descriptor_schema_validation_boundary_present: false,
        descriptor_schema_validation_boundary_source_chain_complete: false,
        descriptor_capability_validation_boundary_present: false,
        descriptor_capability_validation_boundary_source_chain_complete: false,
        descriptor_load_plan_boundary_present: false,
        descriptor_load_plan_boundary_source_chain_complete: false,
        executable_load_plan_authority_boundary_present: false,
        executable_load_plan_authority_boundary_source_chain_complete: false,
        executable_load_plan_result_boundary_present: false,

        executable_load_plan_result_boundary_source_chain_complete: false,
        executable_image_layout_boundary_present: false,

        executable_image_layout_boundary_source_chain_complete: false,
        executable_page_mapping_plan_boundary_present: false,
        executable_page_mapping_plan_boundary_source_chain_complete: false,
        executable_page_mapping_boundary_present: false,
        executable_page_mapping_boundary_source_chain_complete: false,
        descriptor_executable_page_binding_boundary_present: false,
        descriptor_executable_page_binding_boundary_source_chain_complete: false,
        executable_entrypoint_binding_boundary_present: false,
        executable_entrypoint_binding_boundary_source_chain_complete: false,
        executable_entrypoint_transfer_authorization_boundary_present: false,
        executable_entrypoint_transfer_authorization_boundary_source_chain_complete: false,
        executable_entrypoint_transfer_boundary_present: false,
        executable_entrypoint_transfer_boundary_source_chain_complete: false,
        executable_entrypoint_handoff_boundary_present: false,
        executable_entrypoint_handoff_boundary_source_chain_complete: false,
        artifact_byte_intake_boundary_present: false,
        artifact_byte_intake_boundary_source_chain_complete: false,
        execution_authorization_boundary_present: false,
        execution_authorization_boundary_source_chain_complete: false,
        service_registry_mutation_boundary_present: false,
        service_registry_mutation_boundary_source_chain_complete: false,
        service_slot_binding_source_evidence_present: false,
        health_state_hooks_source_evidence_present: false,
        artifact_hash_binding_present: false,
        entrypoint_abi_source_evidence_present: false,
        address_space_source_evidence_present: false,
        memory_map_source_evidence_present: false,
        capability_import_table_source_evidence_present: false,
        audit_rollback_write_boundary_source_evidence_present: false,
        retained_module_evidence_present: false,
        retained_artifact_reference_present: false,
        retained_service_slot_reservation_present: false,
    }
}

fn module_load_gate_authority_input_projection_missing(
    source: ModuleServiceSlotAuthorityInputSpec,
) -> ModuleServiceSlotAuthorityInputProjection {
    ModuleServiceSlotAuthorityInputProjection {
        schema: source.schema,
        name: source.name,
        present: false,
        status: source.missing_status,
        reason: source.source_evidence_missing_reason,
        source_evidence_event_id: None,
    }
}

fn module_load_gate_authority_input_projections_missing(
) -> [ModuleServiceSlotAuthorityInputProjection; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT] {
    [
        module_load_gate_authority_input_projection_missing(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[0],
        ),
        module_load_gate_authority_input_projection_missing(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[1],
        ),
        module_load_gate_authority_input_projection_missing(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[2],
        ),
        module_load_gate_authority_input_projection_missing(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[3],
        ),
        module_load_gate_authority_input_projection_missing(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[4],
        ),
    ]
}

fn module_load_gate_service_slot_allocator_projection(
    binding: event_log::ModuleLoadGateBinding,
) -> ModuleLoadGateServiceSlotAllocatorProjection {
    if module_load_gate_service_slot_reservation_rejected(binding) {
        return ModuleLoadGateServiceSlotAllocatorProjection {
            state: "blocked_by_rejected_service_slot_reservation",
            status: "blocked",
            reason: binding.service_slot_reservation_reason,
            ready: false,
            authority_source_evidence_event_id: None,
            authority_status: "missing",
            authority_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_source_evidence_event_id: None,
            allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
            allocation_intent_reason:
                MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_present: false,
            authority_inputs: module_load_gate_authority_input_projections_missing(),
            authority_decision_source_evidence_event_id: None,
            authority_decision_status:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
            authority_decision_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
            authority_decision_present: false,
            registry_write_commit_gate_source_evidence_event_id: None,
            registry_write_commit_gate_status:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
            registry_write_commit_gate_present: false,
        };
    }
    if !module_load_gate_service_slot_reservation_valid(binding) {
        return ModuleLoadGateServiceSlotAllocatorProjection {
            state: "blocked_by_service_slot_reservation",
            status: "blocked",
            reason: "retained_service_slot_reservation_missing",
            ready: false,
            authority_source_evidence_event_id: None,
            authority_status: "missing",
            authority_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_source_evidence_event_id: None,
            allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
            allocation_intent_reason:
                MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_present: false,
            authority_inputs: module_load_gate_authority_input_projections_missing(),
            authority_decision_source_evidence_event_id: None,
            authority_decision_status:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
            authority_decision_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
            authority_decision_present: false,
            registry_write_commit_gate_source_evidence_event_id: None,
            registry_write_commit_gate_status:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
            registry_write_commit_gate_present: false,
        };
    }

    let allocator_runtime_source = MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[0];
    let allocator_runtime = event_log::latest_module_service_slot_allocator_fact_source_evidence(
        allocator_runtime_source.source_fact_locator,
    );
    let Some((allocator_runtime_event_id, allocator_runtime)) = allocator_runtime else {
        return module_load_gate_service_slot_allocator_missing_projection(
            "missing_runtime",
            allocator_runtime_source.source_evidence_missing_reason,
        );
    };
    if !module_load_gate_service_slot_allocator_fact_source_available(
        binding,
        allocator_runtime,
        None,
    ) {
        return module_load_gate_service_slot_allocator_missing_projection(
            "missing_runtime",
            module_load_gate_service_slot_allocator_fact_source_reason(
                binding,
                allocator_runtime,
                None,
            ),
        );
    }
    let registry_binding_source = MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[1];
    let registry_binding = event_log::latest_module_service_slot_allocator_fact_source_evidence(
        registry_binding_source.source_fact_locator,
    );
    let Some((registry_binding_event_id, registry_binding)) = registry_binding else {
        return module_load_gate_service_slot_allocator_missing_projection(
            "missing_registry_binding",
            registry_binding_source.source_evidence_missing_reason,
        );
    };
    if !module_load_gate_service_slot_allocator_fact_source_available(
        binding,
        registry_binding,
        Some(allocator_runtime_event_id),
    ) {
        return module_load_gate_service_slot_allocator_missing_projection(
            "missing_registry_binding",
            module_load_gate_service_slot_allocator_fact_source_reason(
                binding,
                registry_binding,
                Some(allocator_runtime_event_id),
            ),
        );
    }
    let health_state_source = MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[2];
    let health_state = event_log::latest_module_service_slot_allocator_fact_source_evidence(
        health_state_source.source_fact_locator,
    );
    let Some((health_state_event_id, health_state)) = health_state else {
        return module_load_gate_service_slot_allocator_missing_projection(
            "missing_health_state",
            health_state_source.source_evidence_missing_reason,
        );
    };
    if !module_load_gate_service_slot_allocator_fact_source_available(
        binding,
        health_state,
        Some(allocator_runtime_event_id),
    ) {
        return module_load_gate_service_slot_allocator_missing_projection(
            "missing_health_state",
            module_load_gate_service_slot_allocator_fact_source_reason(
                binding,
                health_state,
                Some(allocator_runtime_event_id),
            ),
        );
    }
    let unload_cleanup_source = MODULE_SERVICE_SLOT_ALLOCATOR_FACT_SOURCES[3];
    let unload_cleanup = event_log::latest_module_service_slot_allocator_fact_source_evidence(
        unload_cleanup_source.source_fact_locator,
    );
    let Some((unload_cleanup_event_id, unload_cleanup)) = unload_cleanup else {
        return module_load_gate_service_slot_allocator_missing_projection(
            "missing_unload_cleanup",
            unload_cleanup_source.source_evidence_missing_reason,
        );
    };
    if !module_load_gate_service_slot_allocator_fact_source_available(
        binding,
        unload_cleanup,
        Some(allocator_runtime_event_id),
    ) {
        return module_load_gate_service_slot_allocator_missing_projection(
            "missing_unload_cleanup",
            module_load_gate_service_slot_allocator_fact_source_reason(
                binding,
                unload_cleanup,
                Some(allocator_runtime_event_id),
            ),
        );
    }
    let durable_audit_source = MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[0];
    let durable_audit =
        event_log::latest_module_service_slot_allocator_prerequisite_source_evidence(
            durable_audit_source.source_fact_locator,
        );
    let Some((durable_audit_event_id, durable_audit)) = durable_audit else {
        return ModuleLoadGateServiceSlotAllocatorProjection {
            state: "missing_durable_audit_write",
            status: "denied_missing_durable_audit_write",
            reason: durable_audit_source.source_evidence_missing_reason,
            ready: false,
            authority_source_evidence_event_id: None,
            authority_status: "missing",
            authority_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_source_evidence_event_id: None,
            allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
            allocation_intent_reason:
                MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_present: false,
            authority_inputs: module_load_gate_authority_input_projections_missing(),
            authority_decision_source_evidence_event_id: None,
            authority_decision_status:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
            authority_decision_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
            authority_decision_present: false,
            registry_write_commit_gate_source_evidence_event_id: None,
            registry_write_commit_gate_status:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
            registry_write_commit_gate_present: false,
        };
    };
    if !module_load_gate_service_slot_allocator_prerequisite_source_available(
        durable_audit,
        allocator_runtime_event_id,
        registry_binding_event_id,
        health_state_event_id,
        unload_cleanup_event_id,
    ) {
        return ModuleLoadGateServiceSlotAllocatorProjection {
            state: "missing_durable_audit_write",
            status: "denied_missing_durable_audit_write",
            reason: module_load_gate_service_slot_allocator_prerequisite_source_reason(
                durable_audit,
                allocator_runtime_event_id,
                registry_binding_event_id,
                health_state_event_id,
                unload_cleanup_event_id,
            ),
            ready: false,
            authority_source_evidence_event_id: None,
            authority_status: "missing",
            authority_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_source_evidence_event_id: None,
            allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
            allocation_intent_reason:
                MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_present: false,
            authority_inputs: module_load_gate_authority_input_projections_missing(),
            authority_decision_source_evidence_event_id: None,
            authority_decision_status:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
            authority_decision_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,

            authority_decision_present: false,
            registry_write_commit_gate_source_evidence_event_id: None,
            registry_write_commit_gate_status:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
            registry_write_commit_gate_present: false,
        };
    }

    let rollback_install_source = MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[1];
    let rollback_install =
        event_log::latest_module_service_slot_allocator_prerequisite_source_evidence(
            rollback_install_source.source_fact_locator,
        );
    let Some((rollback_install_event_id, rollback_install)) = rollback_install else {
        return ModuleLoadGateServiceSlotAllocatorProjection {
            state: "missing_rollback_install",
            status: "denied_missing_rollback_install",
            reason: rollback_install_source.source_evidence_missing_reason,
            ready: false,
            authority_source_evidence_event_id: None,
            authority_status: "missing",
            authority_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_source_evidence_event_id: None,
            allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
            allocation_intent_reason:
                MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_present: false,
            authority_inputs: module_load_gate_authority_input_projections_missing(),
            authority_decision_source_evidence_event_id: None,
            authority_decision_status:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
            authority_decision_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
            authority_decision_present: false,
            registry_write_commit_gate_source_evidence_event_id: None,
            registry_write_commit_gate_status:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
            registry_write_commit_gate_present: false,
        };
    };
    if !module_load_gate_service_slot_allocator_prerequisite_source_available(
        rollback_install,
        allocator_runtime_event_id,
        registry_binding_event_id,
        health_state_event_id,
        unload_cleanup_event_id,
    ) {
        return ModuleLoadGateServiceSlotAllocatorProjection {
            state: "missing_rollback_install",
            status: "denied_missing_rollback_install",
            reason: module_load_gate_service_slot_allocator_prerequisite_source_reason(
                rollback_install,
                allocator_runtime_event_id,
                registry_binding_event_id,
                health_state_event_id,
                unload_cleanup_event_id,
            ),
            ready: false,
            authority_source_evidence_event_id: None,
            authority_status: "missing",
            authority_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_source_evidence_event_id: None,
            allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
            allocation_intent_reason:
                MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_present: false,
            authority_inputs: module_load_gate_authority_input_projections_missing(),
            authority_decision_source_evidence_event_id: None,
            authority_decision_status:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
            authority_decision_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
            authority_decision_present: false,
            registry_write_commit_gate_source_evidence_event_id: None,
            registry_write_commit_gate_status:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
            registry_write_commit_gate_present: false,
        };
    }

    let module_loader_source = MODULE_SERVICE_SLOT_ALLOCATOR_PREREQUISITE_SOURCES[2];
    let module_loader =
        event_log::latest_module_service_slot_allocator_prerequisite_source_evidence(
            module_loader_source.source_fact_locator,
        );
    let Some((module_loader_event_id, module_loader)) = module_loader else {
        return ModuleLoadGateServiceSlotAllocatorProjection {
            state: "module_loader_boundary_missing",
            status: "denied_loader_unimplemented",
            reason: module_loader_source.source_evidence_missing_reason,
            ready: false,
            authority_source_evidence_event_id: None,
            authority_status: "missing",
            authority_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_source_evidence_event_id: None,
            allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
            allocation_intent_reason:
                MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_present: false,
            authority_inputs: module_load_gate_authority_input_projections_missing(),
            authority_decision_source_evidence_event_id: None,
            authority_decision_status:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
            authority_decision_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
            authority_decision_present: false,
            registry_write_commit_gate_source_evidence_event_id: None,
            registry_write_commit_gate_status:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
            registry_write_commit_gate_present: false,
        };
    };
    if !module_load_gate_service_slot_allocator_prerequisite_source_available(
        module_loader,
        allocator_runtime_event_id,
        registry_binding_event_id,
        health_state_event_id,
        unload_cleanup_event_id,
    ) {
        return ModuleLoadGateServiceSlotAllocatorProjection {
            state: "module_loader_unimplemented",
            status: "denied_loader_unimplemented",
            reason: module_load_gate_service_slot_allocator_prerequisite_source_reason(
                module_loader,
                allocator_runtime_event_id,
                registry_binding_event_id,
                health_state_event_id,
                unload_cleanup_event_id,
            ),
            ready: false,
            authority_source_evidence_event_id: None,
            authority_status: "missing",
            authority_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_source_evidence_event_id: None,
            allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
            allocation_intent_reason:
                MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_present: false,

            authority_inputs: module_load_gate_authority_input_projections_missing(),
            authority_decision_source_evidence_event_id: None,
            authority_decision_status:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
            authority_decision_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
            authority_decision_present: false,
            registry_write_commit_gate_source_evidence_event_id: None,
            registry_write_commit_gate_status:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
            registry_write_commit_gate_present: false,
        };
    }

    let authority = event_log::latest_module_service_slot_allocator_authority_source_evidence();
    let Some((authority_event_id, authority)) = authority else {
        return ModuleLoadGateServiceSlotAllocatorProjection {
            state: "allocator_authority_missing",
            status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_MISSING_STATUS,
            reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
            ready: false,
            authority_source_evidence_event_id: None,
            authority_status: "missing",
            authority_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_source_evidence_event_id: None,
            allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
            allocation_intent_reason:
                MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_present: false,
            authority_inputs: module_load_gate_authority_input_projections_missing(),
            authority_decision_source_evidence_event_id: None,
            authority_decision_status:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
            authority_decision_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
            authority_decision_present: false,
            registry_write_commit_gate_source_evidence_event_id: None,
            registry_write_commit_gate_status:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
            registry_write_commit_gate_present: false,
        };
    };
    if !module_load_gate_service_slot_allocator_authority_source_available(
        authority,
        allocator_runtime_event_id,
        registry_binding_event_id,
        health_state_event_id,
        unload_cleanup_event_id,
        durable_audit_event_id,
        rollback_install_event_id,
        module_loader_event_id,
    ) {
        return ModuleLoadGateServiceSlotAllocatorProjection {
            state: "allocator_authority_missing",
            status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_MISSING_STATUS,
            reason: module_load_gate_service_slot_allocator_authority_source_reason(
                authority,
                allocator_runtime_event_id,
                registry_binding_event_id,
                health_state_event_id,
                unload_cleanup_event_id,
                durable_audit_event_id,
                rollback_install_event_id,
                module_loader_event_id,
            ),
            ready: false,
            authority_source_evidence_event_id: Some(authority_event_id),
            authority_status: authority.authority_status,
            authority_reason: authority.authority_reason,
            allocation_intent_source_evidence_event_id: None,
            allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
            allocation_intent_reason:
                MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
            allocation_intent_present: false,
            authority_inputs: module_load_gate_authority_input_projections_missing(),
            authority_decision_source_evidence_event_id: None,
            authority_decision_status:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
            authority_decision_reason:
                MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
            authority_decision_present: false,
            registry_write_commit_gate_source_evidence_event_id: None,
            registry_write_commit_gate_status:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
            registry_write_commit_gate_reason:
                MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
            registry_write_commit_gate_present: false,
        };
    }

    let allocation_intent =
        event_log::latest_module_service_slot_allocation_intent_source_evidence();
    let Some((allocation_intent_event_id, allocation_intent)) = allocation_intent else {
        return module_load_gate_service_slot_allocation_intent_missing_projection(
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
            None,
            authority_event_id,
            authority.authority_status,
            authority.authority_reason,
        );
    };
    if !module_load_gate_service_slot_allocation_intent_source_available(
        allocation_intent,
        binding,
        authority_event_id,
    ) {
        return module_load_gate_service_slot_allocation_intent_missing_projection(
            module_load_gate_service_slot_allocation_intent_source_reason(
                allocation_intent,
                binding,
                authority_event_id,
            ),
            Some(allocation_intent_event_id),
            authority_event_id,
            authority.authority_status,
            authority.authority_reason,
        );
    }

    let allocator_readiness_projection = latest_module_service_slot_allocator_readiness_projection(
        binding.service_slot_reservation_event_id,
    );
    let authority_inputs = allocator_readiness_projection.authority_inputs;
    ModuleLoadGateServiceSlotAllocatorProjection {
        state: "defined_non_authorizing",
        status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_STATUS,
        reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
        ready: false,
        authority_source_evidence_event_id: Some(authority_event_id),
        authority_status: "defined_non_authorizing",
        authority_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DENIED_REASON,
        allocation_intent_source_evidence_event_id: Some(allocation_intent_event_id),
        allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_STATUS,
        allocation_intent_reason: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_AVAILABLE_REASON,
        allocation_intent_present: true,
        authority_inputs,
        authority_decision_source_evidence_event_id: allocator_readiness_projection
            .authority_decision_source_evidence_event_id,
        authority_decision_status: allocator_readiness_projection.authority_decision_status,
        authority_decision_reason: allocator_readiness_projection.authority_decision_reason,
        authority_decision_present: allocator_readiness_projection.authority_decision_present,
        registry_write_commit_gate_source_evidence_event_id: allocator_readiness_projection
            .registry_write_commit_gate_source_evidence_event_id,
        registry_write_commit_gate_status: allocator_readiness_projection
            .registry_write_commit_gate_status,

        registry_write_commit_gate_reason: allocator_readiness_projection
            .registry_write_commit_gate_reason,
        registry_write_commit_gate_present: allocator_readiness_projection
            .registry_write_commit_gate_present,
    }
}

fn module_load_gate_service_slot_allocator_missing_projection(
    state: &'static str,
    reason: &'static str,
) -> ModuleLoadGateServiceSlotAllocatorProjection {
    ModuleLoadGateServiceSlotAllocatorProjection {
        state,
        status: "missing",
        reason,
        ready: false,
        authority_source_evidence_event_id: None,
        authority_status: "missing",
        authority_reason: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SOURCE_EVIDENCE_MISSING_REASON,
        allocation_intent_source_evidence_event_id: None,
        allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
        allocation_intent_reason:
            MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_EVIDENCE_MISSING_REASON,
        allocation_intent_present: false,
        authority_inputs: module_load_gate_authority_input_projections_missing(),
        authority_decision_source_evidence_event_id: None,
        authority_decision_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
        authority_decision_reason:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
        authority_decision_present: false,
        registry_write_commit_gate_source_evidence_event_id: None,
        registry_write_commit_gate_status:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
        registry_write_commit_gate_reason:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
        registry_write_commit_gate_present: false,
    }
}

fn module_load_gate_service_slot_allocation_intent_missing_projection(
    reason: &'static str,
    allocation_intent_source_evidence_event_id: Option<event_log::EventId>,
    authority_source_evidence_event_id: event_log::EventId,
    authority_status: &'static str,
    authority_reason: &'static str,
) -> ModuleLoadGateServiceSlotAllocatorProjection {
    ModuleLoadGateServiceSlotAllocatorProjection {
        state: "allocation_intent_missing",
        status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
        reason,
        ready: false,
        authority_source_evidence_event_id: Some(authority_source_evidence_event_id),
        authority_status,
        authority_reason,
        allocation_intent_source_evidence_event_id,
        allocation_intent_status: MODULE_SERVICE_SLOT_ALLOCATION_INTENT_MISSING_STATUS,
        allocation_intent_reason: reason,
        allocation_intent_present: false,
        authority_inputs: module_load_gate_authority_input_projections_missing(),
        authority_decision_source_evidence_event_id: None,
        authority_decision_status: MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_MISSING_STATUS,
        authority_decision_reason:
            MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SOURCE_EVIDENCE_MISSING_REASON,
        authority_decision_present: false,
        registry_write_commit_gate_source_evidence_event_id: None,
        registry_write_commit_gate_status:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_MISSING_STATUS,
        registry_write_commit_gate_reason:
            MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
        registry_write_commit_gate_present: false,
    }
}

fn module_load_gate_service_slot_allocator_fact_source_available(
    binding: event_log::ModuleLoadGateBinding,
    evidence: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    expected_allocator_runtime_source_evidence_event_id: Option<event_log::EventId>,
) -> bool {
    evidence.fact_present
        && evidence.fact_schema_ok
        && evidence.fact_provenance_ok
        && evidence.binds_retained_service_slot_reservation
        && evidence.binds_allocator_runtime
        && evidence.retained_service_slot_reservation_event_id
            == binding.service_slot_reservation_event_id
        && evidence.allocator_runtime_source_evidence_event_id
            == expected_allocator_runtime_source_evidence_event_id
}

fn module_load_gate_service_slot_allocator_fact_source_reason(
    binding: event_log::ModuleLoadGateBinding,
    evidence: event_log::ModuleServiceSlotAllocatorFactSourceEvidence,
    expected_allocator_runtime_source_evidence_event_id: Option<event_log::EventId>,
) -> &'static str {
    if evidence.retained_service_slot_reservation_event_id
        != binding.service_slot_reservation_event_id
    {
        "service_slot_allocator_retained_reservation_binding_mismatch"
    } else if evidence.allocator_runtime_source_evidence_event_id
        != expected_allocator_runtime_source_evidence_event_id
    {
        "service_slot_allocator_runtime_source_evidence_binding_mismatch"
    } else {
        evidence.fact_reason
    }
}

fn module_load_gate_service_slot_allocator_prerequisite_source_available(
    evidence: event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    allocator_runtime_event_id: event_log::EventId,
    registry_binding_event_id: event_log::EventId,
    health_state_event_id: event_log::EventId,
    unload_cleanup_event_id: event_log::EventId,
) -> bool {
    evidence.prerequisite_available
        && evidence.allocator_runtime_source_evidence_event_id == Some(allocator_runtime_event_id)
        && evidence.registry_binding_source_evidence_event_id == Some(registry_binding_event_id)
        && evidence.health_state_source_evidence_event_id == Some(health_state_event_id)
        && evidence.unload_cleanup_source_evidence_event_id == Some(unload_cleanup_event_id)
}

fn module_load_gate_service_slot_allocator_prerequisite_source_reason(
    evidence: event_log::ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    allocator_runtime_event_id: event_log::EventId,
    registry_binding_event_id: event_log::EventId,
    health_state_event_id: event_log::EventId,
    unload_cleanup_event_id: event_log::EventId,
) -> &'static str {
    if evidence.allocator_runtime_source_evidence_event_id != Some(allocator_runtime_event_id)
        || evidence.registry_binding_source_evidence_event_id != Some(registry_binding_event_id)
        || evidence.health_state_source_evidence_event_id != Some(health_state_event_id)
        || evidence.unload_cleanup_source_evidence_event_id != Some(unload_cleanup_event_id)
    {
        "service_slot_allocator_prerequisite_source_evidence_binding_mismatch"
    } else {
        evidence.prerequisite_reason
    }
}

fn module_load_gate_service_slot_allocator_authority_source_available(
    evidence: event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    allocator_runtime_event_id: event_log::EventId,
    registry_binding_event_id: event_log::EventId,
    health_state_event_id: event_log::EventId,
    unload_cleanup_event_id: event_log::EventId,
    durable_audit_event_id: event_log::EventId,
    rollback_install_event_id: event_log::EventId,
    module_loader_event_id: event_log::EventId,
) -> bool {
    evidence.authority_present
        && evidence.authority_schema_ok
        && evidence.authority_provenance_ok
        && evidence.source_chain_complete
        && evidence.allocator_runtime_source_evidence_event_id == Some(allocator_runtime_event_id)
        && evidence.registry_binding_source_evidence_event_id == Some(registry_binding_event_id)
        && evidence.health_state_source_evidence_event_id == Some(health_state_event_id)
        && evidence.unload_cleanup_source_evidence_event_id == Some(unload_cleanup_event_id)
        && evidence.durable_audit_source_evidence_event_id == Some(durable_audit_event_id)
        && evidence.rollback_install_source_evidence_event_id == Some(rollback_install_event_id)
        && evidence.module_loader_source_evidence_event_id == Some(module_loader_event_id)
}

fn module_load_gate_service_slot_allocator_authority_source_reason(
    evidence: event_log::ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    allocator_runtime_event_id: event_log::EventId,
    registry_binding_event_id: event_log::EventId,
    health_state_event_id: event_log::EventId,
    unload_cleanup_event_id: event_log::EventId,
    durable_audit_event_id: event_log::EventId,
    rollback_install_event_id: event_log::EventId,
    module_loader_event_id: event_log::EventId,
) -> &'static str {
    if evidence.allocator_runtime_source_evidence_event_id != Some(allocator_runtime_event_id)
        || evidence.registry_binding_source_evidence_event_id != Some(registry_binding_event_id)
        || evidence.health_state_source_evidence_event_id != Some(health_state_event_id)
        || evidence.unload_cleanup_source_evidence_event_id != Some(unload_cleanup_event_id)
        || evidence.durable_audit_source_evidence_event_id != Some(durable_audit_event_id)
        || evidence.rollback_install_source_evidence_event_id != Some(rollback_install_event_id)
        || evidence.module_loader_source_evidence_event_id != Some(module_loader_event_id)
    {
        "service_slot_allocator_authority_source_evidence_binding_mismatch"
    } else if !evidence.source_chain_complete {
        "service_slot_allocator_authority_source_chain_incomplete"
    } else {
        evidence.authority_reason
    }
}

fn module_load_gate_service_slot_allocation_intent_source_available(
    evidence: event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
    binding: event_log::ModuleLoadGateBinding,
    authority_source_evidence_event_id: event_log::EventId,
) -> bool {
    evidence.intent_present
        && evidence.intent_schema_ok
        && evidence.intent_provenance_ok
        && evidence.source_chain_complete
        && evidence.retained_module_evidence_present
        && evidence.retained_service_slot_reservation_present
        && evidence.allocator_authority_present
        && evidence.intent_scope == "current_boot"
        && evidence.intent_classification == "local_only"
        && evidence.requested_capability == "cap.module.load_ephemeral"
        && evidence.load_mode == "ram_only"
        && evidence.target == "live_service_graph"
        && evidence.service_slot_reservation_event_id == binding.service_slot_reservation_event_id
        && evidence.allocator_authority_source_evidence_event_id
            == Some(authority_source_evidence_event_id)
}

fn module_load_gate_service_slot_allocation_intent_source_reason(
    evidence: event_log::ModuleServiceSlotAllocationIntentSourceEvidence,
    binding: event_log::ModuleLoadGateBinding,
    authority_source_evidence_event_id: event_log::EventId,
) -> &'static str {
    if evidence.service_slot_reservation_event_id != binding.service_slot_reservation_event_id
        || evidence.allocator_authority_source_evidence_event_id
            != Some(authority_source_evidence_event_id)
    {
        "service_slot_allocation_intent_source_evidence_binding_mismatch"
    } else if !evidence.source_chain_complete {
        MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SOURCE_CHAIN_INCOMPLETE_REASON
    } else if evidence.intent_scope != "current_boot" {
        "service_slot_allocation_intent_scope_must_be_current_boot"
    } else if !evidence.intent_schema_ok {
        "service_slot_allocation_intent_schema_mismatch"
    } else if !evidence.intent_provenance_ok {
        "service_slot_allocation_intent_provenance_missing"
    } else {
        evidence.intent_reason
    }
}

fn module_load_gate_service_slot_allocator_ready(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    module_load_gate_service_slot_allocator_projection(binding).ready
}

fn module_load_gate_audit_rollback_reference_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.audit_rollback_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_audit_rollback_reference_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.audit_rollback_reference_status, "rejected")
}

fn module_load_gate_service_slot_reservation_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.service_slot_reservation_status,
        "retained_hash_reference_only_not_allocated",
    )
}

fn module_load_gate_service_slot_reservation_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.service_slot_reservation_status, "rejected")
}

fn module_load_gate_manifest_reference_valid(binding: event_log::ModuleLoadGateBinding) -> bool {
    method_eq(
        binding.manifest_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_manifest_reference_rejected(binding: event_log::ModuleLoadGateBinding) -> bool {
    method_eq(binding.manifest_reference_status, "rejected")
}

fn module_load_gate_candidate_artifact_reference_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.artifact_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_candidate_artifact_reference_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.artifact_reference_status, "rejected")
}

fn module_load_gate_vm_test_report_reference_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.vm_report_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_vm_test_report_reference_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.vm_report_reference_status, "rejected")
}

fn module_load_gate_local_attestation_reference_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.attestation_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_local_attestation_reference_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.attestation_reference_status, "rejected")
}

fn module_load_gate_local_approval_reference_valid(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(
        binding.approval_reference_status,
        "retained_hash_reference_only",
    )
}

fn module_load_gate_local_approval_reference_rejected(
    binding: event_log::ModuleLoadGateBinding,
) -> bool {
    method_eq(binding.approval_reference_status, "rejected")
}

fn module_load_gate_loader_runtime_source_fact_fields(
    source: ModuleLoaderRuntimeFactSource,
) -> Vec<Field<'static>> {
    vec![
        f("fact", s(source.name)),
        f("schema", s(source.schema)),
        f("id", s(source.id)),
        f("source_method", s(source.source_method)),
        f("source_fact_locator", s(source.source_fact_locator)),
        f("missing_reason", s(source.missing_reason)),
        f("status", s("missing")),
        f("present", no()),
    ]
}

fn module_load_gate_audit_rollback_requirement_fields<'a>(
    binding: &'a event_log::ModuleLoadGateBinding,
    compact: bool,
) -> Vec<Field<'a>> {
    vec![
        f(
            "record_schema",
            s("raios.module_load_gate_audit_rollback_requirements.v0"),
        ),
        f("classification", s("public")),
        f("status", s("required_missing")),
        f(
            "durable_audit_record",
            record_object_for(
                vec![
                    f("schema", s("raios.audit_record.v0")),
                    f("state", s(module_load_gate_durable_audit_state(*binding))),
                    f("durability", s("required_before_load")),
                    f(
                        "required_bindings",
                        static_str_array(
                            &[
                                "denial_event_id",
                                "retained_computed_grant_reference_event_id",
                                "computed_capability_grant_hash",
                                "manifest_hash",
                                "artifact_hash",
                                "vm_test_report_hash",
                                "local_attestation_hash",
                                "local_approval_hash",
                                "rollback_plan_hash",
                                "ram_only_service_slot_id",
                            ],
                            compact,
                        ),
                    ),
                ],
                compact,
            ),
        ),
        f(
            "rollback_plan",
            record_object_for(
                vec![
                    f("schema", s("raios.rollback_plan.v0")),
                    f("state", s(module_load_gate_rollback_state(*binding))),
                    f("must_preexist_load", b(true)),
                    f(
                        "required_bindings",
                        static_str_array(
                            &[
                                "artifact_hash",
                                "pre_load_service_inventory_hash",
                                "ram_only_service_slot_id",
                                "cleanup_actions_hash",
                            ],
                            compact,
                        ),
                    ),
                ],
                compact,
            ),
        ),
        f(
            "required_hashes",
            record_object_for(module_load_gate_hash_fields(binding), compact),
        ),
        f(
            "retained_reference_event_id",
            record_event_or_null(binding.retained_reference_event_id),
        ),
        f(
            "retained_manifest_reference_event_id",
            record_event_or_null(binding.manifest_reference_event_id),
        ),
        f(
            "retained_local_attestation_reference_event_id",
            record_event_or_null(binding.attestation_reference_event_id),
        ),
        f(
            "retained_local_approval_reference_event_id",
            record_event_or_null(binding.approval_reference_event_id),
        ),
        f(
            "retained_audit_rollback_reference_event_id",
            record_event_or_null(binding.audit_rollback_reference_event_id),
        ),
        f(
            "retained_service_slot_reservation_event_id",
            record_event_or_null(binding.service_slot_reservation_event_id),
        ),
        f(
            "local_approval",
            V::InlineObject(vec![
                f("state", s(module_load_gate_local_approval_state(*binding))),
                f(
                    "reason",
                    s(module_load_gate_local_approval_reason(*binding)),
                ),
                f("required", b(true)),
            ]),
        ),
        f(
            "ram_only_service_slot",
            V::InlineObject(vec![
                f("state", s(module_load_gate_service_slot_state(*binding))),
                f("reason", s(module_load_gate_service_slot_reason(*binding))),
                f("required", b(true)),
            ]),
        ),
    ]
}

fn module_load_gate_hash_fields<'a>(
    binding: &'a event_log::ModuleLoadGateBinding,
) -> Vec<Field<'a>> {
    let retained = binding.retained_reference.as_ref();
    let attestation = binding
        .attestation_reference
        .as_ref()
        .filter(|_| module_load_gate_local_attestation_reference_valid(*binding));
    let approval = binding
        .approval_reference
        .as_ref()
        .filter(|_| module_load_gate_local_approval_reference_valid(*binding));
    let report = binding
        .vm_report_reference
        .as_ref()
        .filter(|_| module_load_gate_vm_test_report_reference_valid(*binding));
    let artifact = binding
        .artifact_reference
        .as_ref()
        .filter(|_| module_load_gate_candidate_artifact_reference_valid(*binding));
    let manifest = binding
        .manifest_reference
        .as_ref()
        .filter(|_| module_load_gate_manifest_reference_valid(*binding));
    let audit = binding
        .audit_rollback_reference
        .as_ref()
        .filter(|_| module_load_gate_audit_rollback_reference_valid(*binding));
    let reservation = binding
        .service_slot_reservation
        .as_ref()
        .filter(|_| module_load_gate_service_slot_reservation_valid(*binding));

    vec![
        f(
            "computed_capability_grant_hash",
            record_sha_or_null(retained.map(|reference| reference.computed_grant_hash)),
        ),
        f(
            "local_attestation_reference_hash",
            record_sha_or_null(attestation.map(|reference| reference.attestation_reference_hash)),
        ),
        f(
            "local_attestation_hash",
            record_sha_or_null(attestation.map(|reference| reference.local_attestation_hash)),
        ),
        f(
            "local_approval_reference_hash",
            record_sha_or_null(approval.map(|reference| reference.approval_reference_hash)),
        ),
        f(
            "local_approval_hash",
            record_sha_or_null(approval.map(|reference| reference.local_approval_hash)),
        ),
        f(
            "vm_test_report_reference_hash",
            record_sha_or_null(report.map(|reference| reference.report_reference_hash)),
        ),
        f(
            "vm_test_report_hash",
            record_sha_or_null(report.map(|reference| reference.vm_report_hash)),
        ),
        f(
            "artifact_reference_hash",
            record_sha_or_null(artifact.map(|reference| reference.artifact_reference_hash)),
        ),
        f(
            "artifact_hash",
            record_sha_or_null(artifact.map(|reference| reference.artifact_hash)),
        ),
        f(
            "manifest_reference_hash",
            record_sha_or_null(manifest.map(|reference| reference.manifest_reference_hash)),
        ),
        f(
            "manifest_hash",
            record_sha_or_null(manifest.map(|reference| reference.manifest_hash)),
        ),
        f(
            "audit_record_hash",
            record_sha_or_null(audit.map(|reference| reference.audit_record_hash)),
        ),
        f(
            "rollback_plan_hash",
            record_sha_or_null(audit.map(|reference| reference.rollback_plan_hash)),
        ),
        f(
            "pre_load_service_inventory_hash",
            record_sha_or_null(audit.map(|reference| reference.pre_load_service_inventory_hash)),
        ),
        f(
            "cleanup_actions_hash",
            record_sha_or_null(audit.map(|reference| reference.cleanup_actions_hash)),
        ),
        f(
            "ram_only_service_slot_id",
            record_str_or_null(audit.map(|reference| reference.ram_only_service_slot_id.as_str())),
        ),
        f(
            "service_slot_reservation_hash",
            record_sha_or_null(reservation.map(|reservation| reservation.reservation_hash)),
        ),
    ]
}

fn record_object_for<'a>(fields: Vec<Field<'a>>, compact: bool) -> V<'a> {
    if compact {
        V::InlineObject(fields)
    } else {
        V::Object(fields)
    }
}

fn static_str_array(values: &[&'static str], compact: bool) -> V<'static> {
    let mut out = Vec::new();
    let mut idx = 0usize;
    while idx < values.len() {
        out.push(s(values[idx]));
        idx += 1;
    }
    if compact {
        V::InlineArray(out)
    } else {
        V::Array(out)
    }
}

const MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_FACT: &str = "receiver_identity_load_preflight";
const MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_SCHEMA: &str = "raios.module_load_gate.v0";
const MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_ID: &str =
    "module.load_ephemeral.receiver_identity_load_preflight.current_boot";
const MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_METHOD: &str =
    "module.distribution_receiver_identity_load_preflight";
const MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_LOCATOR: &str =
    "module.load_ephemeral.receiver_identity_load_preflight";

const MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_MISSING_REASON: &str =
    "distribution_receiver_identity_load_preflight_missing";
const MODULE_LOAD_GATE_M6_M7_REVERIFY_INPUT_CHECK_ID: &str =
    "module.load_ephemeral.m6_m7_reverify_input_check.current_boot";
const MODULE_LOAD_GATE_M6_M7_REVERIFY_INPUTS_MISSING_STATUS: &str =
    "denied_missing_m6_m7_reverify_inputs";
const MODULE_LOAD_GATE_M6_M7_REVERIFY_INPUTS_MISSING_REASON: &str = "m6_m7_reverify_inputs_missing";
const MODULE_LOAD_GATE_M6_M7_RECEIVER_PREFLIGHT_NOT_READY_STATUS: &str =
    "denied_receiver_preflight_source_fact_not_ready";
const MODULE_LOAD_GATE_M6_REVERIFY_INPUT_DIAGNOSTIC_ID: &str =
    "module.load_ephemeral.m6_reverify_input_diagnostic.current_boot";
const MODULE_LOAD_GATE_M6_REVERIFY_INPUT_MISSING_STATUS: &str =
    "denied_missing_m6_reverify_evidence";
const MODULE_LOAD_GATE_M6_REVERIFY_INPUT_MISSING_REASON: &str =
    "m6_reverification_evidence_missing";
const MODULE_LOAD_GATE_M7_LOADER_POLICY_INPUT_DIAGNOSTIC_ID: &str =
    "module.load_ephemeral.m7_loader_policy_input_diagnostic.current_boot";
const MODULE_LOAD_GATE_M7_LOADER_POLICY_BLOCKED_BY_M6_STATUS: &str =
    "denied_m6_reverify_input_not_ready_for_m7_loader_policy";
const MODULE_LOAD_GATE_M7_LOADER_POLICY_INPUT_MISSING_REASON: &str =
    "m7_loader_policy_evidence_missing";
const MODULE_LOAD_GATE_PROVIDER_TRUST_INPUT_DIAGNOSTIC_ID: &str =
    "module.load_ephemeral.provider_trust_input_diagnostic.current_boot";
const MODULE_LOAD_GATE_PROVIDER_TRUST_BLOCKED_BY_M7_STATUS: &str =
    "denied_m7_loader_policy_input_not_ready_for_provider_trust";
const MODULE_LOAD_GATE_PROVIDER_TRUST_INPUT_MISSING_REASON: &str =
    "provider_trust_evidence_missing";

fn module_load_gate_loader_runtime_source_fact_count() -> usize {
    MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT + 1
}

fn module_load_gate_loader_runtime_source_fact_map_complete() -> bool {
    module_loader_runtime_source_fact_map_complete()
        && !MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_FACT.is_empty()
        && !MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_SCHEMA.is_empty()
        && !MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_ID.is_empty()
        && !MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_METHOD.is_empty()
        && !MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_LOCATOR.is_empty()
        && !MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_MISSING_REASON.is_empty()
}

fn module_load_gate_receiver_identity_load_preflight_status(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> &'static str {
    if projection.present {
        projection.status
    } else {
        "missing"
    }
}

fn module_load_gate_receiver_identity_load_preflight_reason(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> &'static str {
    if projection.present {
        projection.reason
    } else {
        MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_MISSING_REASON
    }
}

fn module_load_gate_receiver_identity_load_preflight_source_fact_ready(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> bool {
    projection.present
        && projection.receiver_identity_complete
        && projection.retained_candidate_matches_catalog_finalize
        && projection.preflight_evaluated
        && projection.accepted
        && !projection.rejected
}

fn module_load_gate_m6_m7_reverify_input_check_status(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> &'static str {
    if module_load_gate_receiver_identity_load_preflight_source_fact_ready(projection) {
        MODULE_LOAD_GATE_M6_M7_REVERIFY_INPUTS_MISSING_STATUS
    } else {
        MODULE_LOAD_GATE_M6_M7_RECEIVER_PREFLIGHT_NOT_READY_STATUS
    }
}

fn module_load_gate_m6_m7_reverify_input_check_reason(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> &'static str {
    if module_load_gate_receiver_identity_load_preflight_source_fact_ready(projection) {
        MODULE_LOAD_GATE_M6_M7_REVERIFY_INPUTS_MISSING_REASON
    } else {
        module_load_gate_receiver_identity_load_preflight_reason(projection)
    }
}

fn module_load_gate_m6_reverify_input_diagnostic_status(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> &'static str {
    if module_load_gate_receiver_identity_load_preflight_source_fact_ready(projection) {
        MODULE_LOAD_GATE_M6_REVERIFY_INPUT_MISSING_STATUS
    } else {
        MODULE_LOAD_GATE_M6_M7_RECEIVER_PREFLIGHT_NOT_READY_STATUS
    }
}

fn module_load_gate_m6_reverify_input_diagnostic_reason(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> &'static str {
    if module_load_gate_receiver_identity_load_preflight_source_fact_ready(projection) {
        MODULE_LOAD_GATE_M6_REVERIFY_INPUT_MISSING_REASON
    } else {
        module_load_gate_receiver_identity_load_preflight_reason(projection)
    }
}

fn module_load_gate_m6_reverify_input_diagnostic_fields(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> Vec<Field<'static>> {
    let receiver_preflight_ready =
        module_load_gate_receiver_identity_load_preflight_source_fact_ready(projection);
    vec![
        f("id", s(MODULE_LOAD_GATE_M6_REVERIFY_INPUT_DIAGNOSTIC_ID)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f(
            "consumes_check",
            s(MODULE_LOAD_GATE_M6_M7_REVERIFY_INPUT_CHECK_ID),
        ),
        f(
            "source_fact",
            s(MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_FACT),
        ),
        f(
            "source_fact_locator",
            s(MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_LOCATOR),
        ),
        f(
            "receiver_preflight_input_ready",
            b(receiver_preflight_ready),
        ),
        f(
            "receiver_candidate_binding_absent",
            b(!receiver_preflight_ready),
        ),
        f(
            "receiver_preflight_status",
            s(module_load_gate_receiver_identity_load_preflight_status(
                projection,
            )),
        ),
        f(
            "receiver_preflight_reason",
            s(module_load_gate_receiver_identity_load_preflight_reason(
                projection,
            )),
        ),
        f(
            "status",
            s(module_load_gate_m6_reverify_input_diagnostic_status(
                projection,
            )),
        ),
        f(
            "reason",
            s(module_load_gate_m6_reverify_input_diagnostic_reason(
                projection,
            )),
        ),
        f("m6_reverification_evidence_present", no()),
        f(
            "m6_reverification_evidence_reason",
            s(MODULE_LOAD_GATE_M6_REVERIFY_INPUT_MISSING_REASON),
        ),
        f("can_enter_m6_reverify", no()),
    ]
}

fn module_load_gate_m7_loader_policy_input_diagnostic_status(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> &'static str {
    if module_load_gate_receiver_identity_load_preflight_source_fact_ready(projection) {
        MODULE_LOAD_GATE_M7_LOADER_POLICY_BLOCKED_BY_M6_STATUS
    } else {
        MODULE_LOAD_GATE_M6_M7_RECEIVER_PREFLIGHT_NOT_READY_STATUS
    }
}

fn module_load_gate_m7_loader_policy_input_diagnostic_reason(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> &'static str {
    if module_load_gate_receiver_identity_load_preflight_source_fact_ready(projection) {
        MODULE_LOAD_GATE_M6_REVERIFY_INPUT_MISSING_REASON
    } else {
        module_load_gate_receiver_identity_load_preflight_reason(projection)
    }
}

fn module_load_gate_m7_loader_policy_input_diagnostic_fields(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> Vec<Field<'static>> {
    vec![
        f(
            "id",
            s(MODULE_LOAD_GATE_M7_LOADER_POLICY_INPUT_DIAGNOSTIC_ID),
        ),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f(
            "consumes_diagnostic",
            s(MODULE_LOAD_GATE_M6_REVERIFY_INPUT_DIAGNOSTIC_ID),
        ),
        f(
            "consumes_check",
            s(MODULE_LOAD_GATE_M6_M7_REVERIFY_INPUT_CHECK_ID),
        ),
        f("m6_reverify_input_ready_for_loader_policy", no()),
        f(
            "m6_reverify_input_status",
            s(module_load_gate_m6_reverify_input_diagnostic_status(
                projection,
            )),
        ),
        f(
            "m6_reverify_input_reason",
            s(module_load_gate_m6_reverify_input_diagnostic_reason(
                projection,
            )),
        ),
        f("m6_reverification_evidence_present", no()),
        f(
            "m6_reverification_evidence_reason",
            s(MODULE_LOAD_GATE_M6_REVERIFY_INPUT_MISSING_REASON),
        ),
        f("m7_loader_policy_evidence_present", no()),
        f(
            "m7_loader_policy_evidence_reason",
            s(MODULE_LOAD_GATE_M7_LOADER_POLICY_INPUT_MISSING_REASON),
        ),
        f(
            "status",
            s(module_load_gate_m7_loader_policy_input_diagnostic_status(
                projection,
            )),
        ),
        f(
            "reason",
            s(module_load_gate_m7_loader_policy_input_diagnostic_reason(
                projection,
            )),
        ),
        f("can_enter_m7_loader_policy", no()),
    ]
}

fn module_load_gate_provider_trust_input_diagnostic_status(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> &'static str {
    if module_load_gate_receiver_identity_load_preflight_source_fact_ready(projection) {
        MODULE_LOAD_GATE_PROVIDER_TRUST_BLOCKED_BY_M7_STATUS
    } else {
        MODULE_LOAD_GATE_M6_M7_RECEIVER_PREFLIGHT_NOT_READY_STATUS
    }
}

fn module_load_gate_provider_trust_input_diagnostic_reason(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> &'static str {
    if module_load_gate_receiver_identity_load_preflight_source_fact_ready(projection) {
        MODULE_LOAD_GATE_M7_LOADER_POLICY_INPUT_MISSING_REASON
    } else {
        module_load_gate_receiver_identity_load_preflight_reason(projection)
    }
}

fn module_load_gate_provider_trust_input_diagnostic_fields(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> Vec<Field<'static>> {
    vec![
        f("id", s(MODULE_LOAD_GATE_PROVIDER_TRUST_INPUT_DIAGNOSTIC_ID)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f(
            "consumes_diagnostic",
            s(MODULE_LOAD_GATE_M7_LOADER_POLICY_INPUT_DIAGNOSTIC_ID),
        ),
        f(
            "consumes_check",
            s(MODULE_LOAD_GATE_M6_M7_REVERIFY_INPUT_CHECK_ID),
        ),
        f("m7_loader_policy_input_ready_for_provider_trust", no()),
        f(
            "m7_loader_policy_input_status",
            s(module_load_gate_m7_loader_policy_input_diagnostic_status(
                projection,
            )),
        ),
        f(
            "m7_loader_policy_input_reason",
            s(module_load_gate_m7_loader_policy_input_diagnostic_reason(
                projection,
            )),
        ),
        f("m7_loader_policy_evidence_present", no()),
        f(
            "m7_loader_policy_evidence_reason",
            s(MODULE_LOAD_GATE_M7_LOADER_POLICY_INPUT_MISSING_REASON),
        ),
        f("provider_trust_evidence_present", no()),
        f(
            "provider_trust_evidence_reason",
            s(MODULE_LOAD_GATE_PROVIDER_TRUST_INPUT_MISSING_REASON),
        ),
        f("provider_trust_positive", no()),
        f(
            "status",
            s(module_load_gate_provider_trust_input_diagnostic_status(
                projection,
            )),
        ),
        f(
            "reason",
            s(module_load_gate_provider_trust_input_diagnostic_reason(
                projection,
            )),
        ),
        f("can_enter_provider_trust", no()),
    ]
}

fn module_load_gate_m6_m7_reverify_input_check_fields(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> Vec<Field<'static>> {
    let receiver_preflight_ready =
        module_load_gate_receiver_identity_load_preflight_source_fact_ready(projection);
    vec![
        f("id", s(MODULE_LOAD_GATE_M6_M7_REVERIFY_INPUT_CHECK_ID)),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f(
            "source_fact",
            s(MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_FACT),
        ),
        f(
            "source_fact_locator",
            s(MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_LOCATOR),
        ),
        f("source_fact_present", b(projection.present)),
        f(
            "source_fact_ready_for_reverify",
            b(receiver_preflight_ready),
        ),
        f(
            "source_fact_status",
            s(module_load_gate_receiver_identity_load_preflight_status(
                projection,
            )),
        ),
        f(
            "source_fact_reason",
            s(module_load_gate_receiver_identity_load_preflight_reason(
                projection,
            )),
        ),
        f(
            "status",
            s(module_load_gate_m6_m7_reverify_input_check_status(
                projection,
            )),
        ),
        f(
            "reason",
            s(module_load_gate_m6_m7_reverify_input_check_reason(
                projection,
            )),
        ),
        f(
            "m6_reverify_input_diagnostic",
            V::InlineObject(module_load_gate_m6_reverify_input_diagnostic_fields(
                projection,
            )),
        ),
        f(
            "m7_loader_policy_input_diagnostic",
            V::InlineObject(module_load_gate_m7_loader_policy_input_diagnostic_fields(
                projection,
            )),
        ),
        f(
            "provider_trust_input_diagnostic",
            V::InlineObject(module_load_gate_provider_trust_input_diagnostic_fields(
                projection,
            )),
        ),
        f(
            "receiver_identity_complete",
            b(projection.receiver_identity_complete),
        ),
        f(
            "retained_candidate_matches_catalog_finalize",
            b(projection.retained_candidate_matches_catalog_finalize),
        ),
        f("preflight_evaluated", b(projection.preflight_evaluated)),
        f("m6_reverification_input_present", no()),
        f(
            "m6_reverification_input_reason",
            s("m6_reverification_evidence_missing"),
        ),
        f("m7_loader_policy_input_present", no()),
        f(
            "m7_loader_policy_input_reason",
            s("m7_loader_policy_evidence_missing"),
        ),
        f("m6_reverification_gate_satisfied", no()),
        f("m7_loader_policy_gate_satisfied", no()),
        f("can_enter_m6_reverify", no()),
        f("can_enter_m7_loader_policy", no()),
    ]
}

fn module_load_gate_receiver_identity_load_preflight_fields(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> Vec<Field<'static>> {
    vec![
        f("present", b(projection.present)),
        f("source_id", s("local.serial.catalog")),
        f("entry_id", s("local.catalog.distribution")),
        f("status", s(projection.status)),
        f("reason", s(projection.reason)),
        f(
            "content_sha256",
            record_sha_or_null(projection.content_sha256),
        ),
        f(
            "retained_part_count",
            V::U64(projection.retained_part_count as u64),
        ),
        f(
            "receiver_identity_retained",
            b(projection.receiver_identity_retained),
        ),
        f(
            "receiver_identity_complete",
            b(projection.receiver_identity_complete),
        ),
        f(
            "guest_signature_verification_performed",
            b(projection.guest_signature_verification_performed),
        ),
        f(
            "retained_candidate_sha256",
            record_sha_or_null(projection.retained_candidate_sha256),
        ),
        f(
            "retained_candidate_present",
            b(projection.retained_candidate_sha256.is_some()),
        ),
        f(
            "retained_candidate_wasm_valid",
            b(projection.retained_candidate_wasm_valid),
        ),
        f(
            "catalog_finalize_candidate_sha256",
            record_sha_or_null(projection.catalog_finalize_candidate_sha256),
        ),
        f(
            "retained_candidate_matches_catalog_finalize",
            b(projection.retained_candidate_matches_catalog_finalize),
        ),
        f("preflight_evaluated", b(projection.preflight_evaluated)),
        f("accepted", b(projection.accepted)),
        f("rejected", b(projection.rejected)),
        f(
            "missing_gate_count",
            V::U64(projection.missing_gate_count as u64),
        ),
        f("m6_reverification_gate_satisfied", no()),
        f("m7_loader_policy_gate_satisfied", no()),
        f("provider_trust_gate_satisfied", no()),
        f("owner_seal_gate_satisfied", no()),
        f("requires_m6_m7_reverify_for_load", b(true)),
        f("requires_provider_trust_for_load", b(true)),
        f("requires_owner_seal_for_load", b(true)),
        f("owner_sealed", no()),
        f("trust_tier", s("dev_key_not_owner_sealed")),
    ]
}

fn module_load_gate_receiver_identity_load_preflight_source_fact_fields(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> Vec<Field<'static>> {
    vec![
        f("fact", s(MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_FACT)),
        f(
            "schema",
            s(MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_SCHEMA),
        ),
        f("id", s(MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_ID)),
        f(
            "source_method",
            s(MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_METHOD),
        ),
        f(
            "source_fact_locator",
            s(MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_LOCATOR),
        ),
        f(
            "missing_reason",
            s(MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_MISSING_REASON),
        ),
        f(
            "status",
            s(module_load_gate_receiver_identity_load_preflight_status(
                projection,
            )),
        ),
        f(
            "reason",
            s(module_load_gate_receiver_identity_load_preflight_reason(
                projection,
            )),
        ),
        f("present", b(projection.present)),
        f(
            "receiver_identity_complete",
            b(projection.receiver_identity_complete),
        ),
        f(
            "retained_candidate_matches_catalog_finalize",
            b(projection.retained_candidate_matches_catalog_finalize),
        ),
        f("preflight_evaluated", b(projection.preflight_evaluated)),
        f("requires_m6_m7_reverify_for_load", b(true)),
    ]
}

fn module_load_gate_receiver_identity_load_preflight_runtime_fact_fields(
    projection: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> Vec<Field<'static>> {
    vec![
        f(
            "schema",
            s(MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_SCHEMA),
        ),
        f("id", s(MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_ID)),
        f(
            "source_method",
            s(MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_METHOD),
        ),
        f(
            "source_fact_locator",
            s(MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_LOCATOR),
        ),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f(
            "status",
            s(module_load_gate_receiver_identity_load_preflight_status(
                projection,
            )),
        ),
        f(
            "reason",
            s(module_load_gate_receiver_identity_load_preflight_reason(
                projection,
            )),
        ),
        f("present", b(projection.present)),
        f(
            "content_sha256",
            record_sha_or_null(projection.content_sha256),
        ),
        f(
            "receiver_identity_retained",
            b(projection.receiver_identity_retained),
        ),
        f(
            "receiver_identity_complete",
            b(projection.receiver_identity_complete),
        ),
        f(
            "guest_signature_verification_performed",
            b(projection.guest_signature_verification_performed),
        ),
        f(
            "retained_candidate_sha256",
            record_sha_or_null(projection.retained_candidate_sha256),
        ),
        f(
            "retained_candidate_wasm_valid",
            b(projection.retained_candidate_wasm_valid),
        ),
        f(
            "catalog_finalize_candidate_sha256",
            record_sha_or_null(projection.catalog_finalize_candidate_sha256),
        ),
        f(
            "retained_candidate_matches_catalog_finalize",
            b(projection.retained_candidate_matches_catalog_finalize),
        ),
        f("preflight_evaluated", b(projection.preflight_evaluated)),
        f("accepted", b(projection.accepted)),
        f("rejected", b(projection.rejected)),
        f(
            "missing_gate_count",
            V::U64(projection.missing_gate_count as u64),
        ),
        f("requires_m6_m7_reverify_for_load", b(true)),
    ]
}

const MODULE_LOAD_GATE_REQUIRED: &[&str] = &[
    "raios.module_manifest.v0",
    "candidate_artifact_sha256",
    "raios.vm_test_report.v0",
    "raios.local_attestation.v0",
    "computed_capability_grant",
    "local_approval",
    "raios.audit_record.v0",
    "rollback_plan",
    "ram_only_service_slot",
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
    "raios.module_loader_execution_authorization_boundary.v0",
    "raios.module_loader_service_registry_mutation_boundary.v0",
    "raios.module_loader_load_attempt_boundary.v0",
    "raios.module_loader_artifact_load_boundary.v0",
    "raios.module_loader_executable_mapping_boundary.v0",
    "raios.module_loader_entrypoint_transfer_boundary.v0",
    "raios.module_loader_service_start_boundary.v0",
    "raios.module_loader_service_health_binding_boundary.v0",
    "raios.module_loader_service_running_state_boundary.v0",
    "raios.module_loader_service_start_audit_boundary.v0",
    "raios.module_loader_service_unload_cleanup_boundary.v0",
    "raios.module_loader_live_load_commit_boundary.v0",
    "raios.module_loader_commit_audit_boundary.v0",
    "raios.module_loader_commit_rollback_boundary.v0",
    "raios.module_loader_commit_result_boundary.v0",
    "raios.module_loader_descriptor_acceptance_authority_boundary.v0",
    "raios.module_loader_descriptor_parser_contract_boundary.v0",
    "raios.module_loader_descriptor_parser_result_boundary.v0",
    "raios.module_loader_descriptor_schema_validation_boundary.v0",
    "raios.module_loader_descriptor_capability_validation_boundary.v0",
    "raios.module_loader_descriptor_load_plan_boundary.v0",
    "raios.module_loader_executable_load_plan_authority_boundary.v0",
    "raios.module_loader_executable_load_plan_result_boundary.v0",
    "raios.module_loader_executable_image_layout_boundary.v0",
    "raios.module_loader_executable_page_mapping_plan_boundary.v0",
    "raios.module_loader_executable_page_mapping_boundary.v0",
    "raios.module_loader_descriptor_executable_page_binding_boundary.v0",
    "raios.module_loader_executable_entrypoint_binding_boundary.v0",
    "raios.module_loader_executable_entrypoint_transfer_authorization_boundary.v0",
    "raios.module_loader_executable_entrypoint_transfer_boundary.v0",
    "raios.module_loader_executable_entrypoint_handoff_boundary.v0",
    "raios.module_loader_executable_entrypoint_invocation_boundary.v0",
    "raios.module_loader_runtime_readiness.v0",
];

fn prerequisite<'a>(
    status: LoadGateEvidenceStatus,
    status_detail: &'a str,
    reason: &'a str,
    source_event_id: Option<event_log::EventId>,
    facts: Vec<Field<'a>>,
) -> LoadGatePrerequisite<'a> {
    LoadGatePrerequisite {
        status,
        status_detail,
        reason,
        source_event_sequence: source_event_id.map(event_log::EventId::sequence),
        facts,
        disposition: LoadGateDisposition::Blocked,
    }
}

fn retained_status(valid: bool, rejected: bool) -> LoadGateEvidenceStatus {
    if valid {
        LoadGateEvidenceStatus::Verified
    } else if rejected {
        LoadGateEvidenceStatus::Rejected
    } else {
        LoadGateEvidenceStatus::Missing
    }
}

fn retained_base<'a>(state: &'static str, schema: &'static str) -> Vec<Field<'a>> {
    vec![
        f("state", s(state)),
        f("retention", s("current_boot_ram_event_log")),
        f("record_schema", s(schema)),
    ]
}

fn retained_record_state(present: bool, rejected: bool) -> &'static str {
    if rejected {
        "rejected"
    } else if present {
        "present"
    } else {
        "missing"
    }
}

fn module_manifest_facts(binding: event_log::ModuleLoadGateBinding) -> Vec<Field<'static>> {
    let mut out = retained_base(
        retained_record_state(
            binding.manifest_reference.is_some(),
            module_load_gate_manifest_reference_rejected(binding),
        ),
        "raios.module_manifest_reference.v0",
    );
    if let Some(reference) = binding.manifest_reference {
        out.extend([
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_unsigned_service_code", no()),
            f(
                "manifest_reference_hash",
                V::Sha256(reference.manifest_reference_hash),
            ),
            f("manifest_hash", V::Sha256(reference.manifest_hash)),
        ]);
    }
    out
}

fn candidate_artifact_facts(binding: event_log::ModuleLoadGateBinding) -> Vec<Field<'static>> {
    let mut out = retained_base(
        retained_record_state(
            binding.artifact_reference.is_some(),
            module_load_gate_candidate_artifact_reference_rejected(binding),
        ),
        "raios.module_candidate_artifact_reference.v0",
    );
    if let Some(reference) = binding.artifact_reference {
        out.extend([
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_unsigned_service_code", no()),
            f(
                "retained_manifest_reference_event_id",
                V::EventSequence(reference.retained_manifest_reference_event_id.sequence()),
            ),
            f(
                "retained_computed_grant_reference_event_id",
                V::EventSequence(reference.retained_reference_event_id.sequence()),
            ),
            f(
                "artifact_reference_hash",
                V::Sha256(reference.artifact_reference_hash),
            ),
            f(
                "manifest_reference_hash",
                V::Sha256(reference.manifest_reference_hash),
            ),
            f("manifest_hash", V::Sha256(reference.manifest_hash)),
            f(
                "computed_capability_grant_hash",
                V::Sha256(reference.computed_grant_hash),
            ),
            f("artifact_hash", V::Sha256(reference.artifact_hash)),
            f("vm_test_report_hash", V::Sha256(reference.vm_report_hash)),
            f(
                "local_attestation_hash",
                V::Sha256(reference.local_attestation_hash),
            ),
        ]);
    }
    out
}

fn vm_report_facts(binding: event_log::ModuleLoadGateBinding) -> Vec<Field<'static>> {
    let mut out = retained_base(
        retained_record_state(
            binding.vm_report_reference.is_some(),
            module_load_gate_vm_test_report_reference_rejected(binding),
        ),
        "raios.module_vm_test_report_reference.v0",
    );
    if let Some(reference) = binding.vm_report_reference {
        out.extend([
            f("accepts_manifest_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_vm_report_json", no()),
            f("accepts_unsigned_service_code", no()),
            f(
                "retained_manifest_reference_event_id",
                V::EventSequence(reference.retained_manifest_reference_event_id.sequence()),
            ),
            f(
                "retained_candidate_artifact_reference_event_id",
                V::EventSequence(reference.retained_artifact_reference_event_id.sequence()),
            ),
            f(
                "retained_computed_grant_reference_event_id",
                V::EventSequence(reference.retained_reference_event_id.sequence()),
            ),
            f(
                "vm_test_report_reference_hash",
                V::Sha256(reference.report_reference_hash),
            ),
            f(
                "manifest_reference_hash",
                V::Sha256(reference.manifest_reference_hash),
            ),
            f(
                "artifact_reference_hash",
                V::Sha256(reference.artifact_reference_hash),
            ),
            f("manifest_hash", V::Sha256(reference.manifest_hash)),
            f("artifact_hash", V::Sha256(reference.artifact_hash)),
            f(
                "computed_capability_grant_hash",
                V::Sha256(reference.computed_grant_hash),
            ),
            f("vm_test_report_hash", V::Sha256(reference.vm_report_hash)),
            f(
                "local_attestation_hash",
                V::Sha256(reference.local_attestation_hash),
            ),
        ]);
    }
    out
}

fn local_attestation_facts(binding: event_log::ModuleLoadGateBinding) -> Vec<Field<'static>> {
    let mut out = retained_base(
        retained_record_state(
            binding.attestation_reference.is_some(),
            module_load_gate_local_attestation_reference_rejected(binding),
        ),
        "raios.module_local_attestation_reference.v0",
    );
    if let Some(reference) = binding.attestation_reference {
        out.extend([
            f("accepts_local_attestation_json", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_unsigned_service_code", no()),
            f(
                "retained_manifest_reference_event_id",
                V::EventSequence(reference.retained_manifest_reference_event_id.sequence()),
            ),
            f(
                "retained_candidate_artifact_reference_event_id",
                V::EventSequence(reference.retained_artifact_reference_event_id.sequence()),
            ),
            f(
                "retained_vm_test_report_reference_event_id",
                V::EventSequence(reference.retained_vm_report_reference_event_id.sequence()),
            ),
            f(
                "retained_computed_grant_reference_event_id",
                V::EventSequence(reference.retained_reference_event_id.sequence()),
            ),
            f(
                "local_attestation_reference_hash",
                V::Sha256(reference.attestation_reference_hash),
            ),
            f(
                "manifest_reference_hash",
                V::Sha256(reference.manifest_reference_hash),
            ),
            f(
                "artifact_reference_hash",
                V::Sha256(reference.artifact_reference_hash),
            ),
            f(
                "vm_test_report_reference_hash",
                V::Sha256(reference.vm_report_reference_hash),
            ),
            f("manifest_hash", V::Sha256(reference.manifest_hash)),
            f("artifact_hash", V::Sha256(reference.artifact_hash)),
            f(
                "computed_capability_grant_hash",
                V::Sha256(reference.computed_grant_hash),
            ),
            f("vm_test_report_hash", V::Sha256(reference.vm_report_hash)),
            f(
                "local_attestation_hash",
                V::Sha256(reference.local_attestation_hash),
            ),
        ]);
    }
    out
}

fn local_approval_facts(binding: event_log::ModuleLoadGateBinding) -> Vec<Field<'static>> {
    let mut out = retained_base(
        retained_record_state(
            binding.approval_reference.is_some(),
            module_load_gate_local_approval_reference_rejected(binding),
        ),
        "raios.module_local_approval_reference.v0",
    );
    if let Some(reference) = binding.approval_reference {
        out.extend([
            f("accepts_local_approval_text", no()),
            f("accepts_artifact_bytes", no()),
            f("accepts_unsigned_service_code", no()),
            f(
                "retained_manifest_reference_event_id",
                V::EventSequence(reference.retained_manifest_reference_event_id.sequence()),
            ),
            f(
                "retained_candidate_artifact_reference_event_id",
                V::EventSequence(reference.retained_artifact_reference_event_id.sequence()),
            ),
            f(
                "retained_vm_test_report_reference_event_id",
                V::EventSequence(reference.retained_vm_report_reference_event_id.sequence()),
            ),
            f(
                "retained_local_attestation_reference_event_id",
                V::EventSequence(
                    reference
                        .retained_local_attestation_reference_event_id
                        .sequence(),
                ),
            ),
            f(
                "retained_computed_grant_reference_event_id",
                V::EventSequence(reference.retained_reference_event_id.sequence()),
            ),
            f(
                "local_approval_reference_hash",
                V::Sha256(reference.approval_reference_hash),
            ),
            f(
                "manifest_reference_hash",
                V::Sha256(reference.manifest_reference_hash),
            ),
            f(
                "artifact_reference_hash",
                V::Sha256(reference.artifact_reference_hash),
            ),
            f(
                "vm_test_report_reference_hash",
                V::Sha256(reference.vm_report_reference_hash),
            ),
            f(
                "local_attestation_reference_hash",
                V::Sha256(reference.local_attestation_reference_hash),
            ),
            f("manifest_hash", V::Sha256(reference.manifest_hash)),
            f("artifact_hash", V::Sha256(reference.artifact_hash)),
            f(
                "computed_capability_grant_hash",
                V::Sha256(reference.computed_grant_hash),
            ),
            f("vm_test_report_hash", V::Sha256(reference.vm_report_hash)),
            f(
                "local_attestation_hash",
                V::Sha256(reference.local_attestation_hash),
            ),
            f(
                "local_approval_hash",
                V::Sha256(reference.local_approval_hash),
            ),
        ]);
    }
    out
}

fn computed_grant_facts(binding: event_log::ModuleLoadGateBinding) -> Vec<Field<'static>> {
    let mut out = retained_base(
        if binding.retained_reference.is_some() {
            "present"
        } else {
            "missing"
        },
        "raios.module_computed_grant_reference.v0",
    );
    if let Some(reference) = binding.retained_reference {
        out.extend([
            f(
                "computed_capability_grant_hash",
                V::Sha256(reference.computed_grant_hash),
            ),
            f("manifest_hash", V::Sha256(reference.manifest_hash)),
            f("artifact_hash", V::Sha256(reference.artifact_hash)),
            f("vm_test_report_hash", V::Sha256(reference.vm_report_hash)),
            f(
                "local_attestation_hash",
                V::Sha256(reference.local_attestation_hash),
            ),
        ]);
    }
    out
}

fn audit_rollback_facts<'a>(
    binding: &'a event_log::ModuleLoadGateBinding,
    rollback: bool,
) -> Vec<Field<'a>> {
    let mut out = retained_base(
        retained_record_state(
            binding.audit_rollback_reference.is_some(),
            module_load_gate_audit_rollback_reference_rejected(*binding),
        ),
        if rollback {
            "raios.rollback_plan.v0"
        } else {
            "raios.audit_record.v0"
        },
    );
    if let Some(reference) = binding.audit_rollback_reference.as_ref() {
        out.extend([
            f(
                "reference_schema",
                s("raios.module_audit_rollback_reference.v0"),
            ),
            f(
                "denial_event_id",
                V::EventSequence(reference.denial_event_id.sequence()),
            ),
            f(
                "retained_computed_grant_reference_event_id",
                V::EventSequence(reference.retained_reference_event_id.sequence()),
            ),
            f(
                "ram_only_service_slot_id",
                s(reference.ram_only_service_slot_id.as_str()),
            ),
            f("audit_record_hash", V::Sha256(reference.audit_record_hash)),
            f(
                "rollback_plan_hash",
                V::Sha256(reference.rollback_plan_hash),
            ),
            f(
                "computed_capability_grant_hash",
                V::Sha256(reference.computed_grant_hash),
            ),
            f("manifest_hash", V::Sha256(reference.manifest_hash)),
            f("artifact_hash", V::Sha256(reference.artifact_hash)),
            f("vm_test_report_hash", V::Sha256(reference.vm_report_hash)),
            f(
                "local_attestation_hash",
                V::Sha256(reference.local_attestation_hash),
            ),
            f(
                "local_approval_hash",
                V::Sha256(reference.local_approval_hash),
            ),
            f(
                "pre_load_service_inventory_hash",
                V::Sha256(reference.pre_load_service_inventory_hash),
            ),
            f(
                "cleanup_actions_hash",
                V::Sha256(reference.cleanup_actions_hash),
            ),
            f(
                "requirements",
                V::InlineObject(module_load_gate_audit_rollback_requirement_fields(
                    binding, true,
                )),
            ),
        ]);
    }
    out
}

fn service_slot_facts<'a>(binding: &'a event_log::ModuleLoadGateBinding) -> Vec<Field<'a>> {
    let mut out = retained_base(
        retained_record_state(
            binding.service_slot_reservation.is_some(),
            module_load_gate_service_slot_reservation_rejected(*binding),
        ),
        "raios.module_service_slot_reservation.v0",
    );
    if let Some(reference) = binding.service_slot_reservation.as_ref() {
        out.extend([
            f(
                "retained_computed_grant_reference_event_id",
                V::EventSequence(reference.retained_reference_event_id.sequence()),
            ),
            f(
                "retained_audit_rollback_reference_event_id",
                V::EventSequence(
                    reference
                        .retained_audit_rollback_reference_event_id
                        .sequence(),
                ),
            ),
            f(
                "ram_only_service_slot_id",
                s(reference.ram_only_service_slot_id.as_str()),
            ),
            f("reservation_hash", V::Sha256(reference.reservation_hash)),
            f(
                "computed_capability_grant_hash",
                V::Sha256(reference.computed_grant_hash),
            ),
            f("audit_record_hash", V::Sha256(reference.audit_record_hash)),
            f(
                "rollback_plan_hash",
                V::Sha256(reference.rollback_plan_hash),
            ),
            f(
                "pre_load_service_inventory_hash",
                V::Sha256(reference.pre_load_service_inventory_hash),
            ),
        ]);
    }
    out
}

fn allocator_authority_inputs(
    inputs: [ModuleServiceSlotAuthorityInputProjection; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
) -> V<'static> {
    V::InlineObject(
        inputs
            .into_iter()
            .map(|input| {
                f(
                    input.name,
                    V::InlineObject(vec![
                        f("record_schema", s(input.schema)),
                        f(
                            "source_event_id",
                            record_event_or_null(input.source_evidence_event_id),
                        ),
                        f("status_detail", s(input.status)),
                        f("reason", s(input.reason)),
                        f("present", b(input.present)),
                    ]),
                )
            })
            .collect(),
    )
}

fn allocator_facts(
    binding: event_log::ModuleLoadGateBinding,
    projection: ModuleLoadGateServiceSlotAllocatorProjection,
) -> Vec<Field<'static>> {
    vec![
        f(
            "record_schema",
            s("raios.module_service_slot_allocator_readiness.v0"),
        ),
        f("source_method", s("module.service_slot_allocator")),
        f("state", s(projection.state)),
        f("readiness_status", s(projection.status)),
        f("readiness_reason", s(projection.reason)),
        f(
            "allocator_authority_boundary",
            V::InlineObject(vec![
                f(
                    "record_schema",
                    s(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SCHEMA),
                ),
                f(
                    "source_event_id",
                    record_event_or_null(projection.authority_source_evidence_event_id),
                ),
                f("status_detail", s(projection.authority_status)),
                f("reason", s(projection.authority_reason)),
            ]),
        ),
        f(
            "allocation_intent_boundary",
            V::InlineObject(vec![
                f(
                    "record_schema",
                    s(MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SCHEMA),
                ),
                f("record_id", s(MODULE_SERVICE_SLOT_ALLOCATION_INTENT_ID)),
                f(
                    "source_event_id",
                    record_event_or_null(projection.allocation_intent_source_evidence_event_id),
                ),
                f("status_detail", s(projection.allocation_intent_status)),
                f("reason", s(projection.allocation_intent_reason)),
                f("present", b(projection.allocation_intent_present)),
            ]),
        ),
        f(
            "authority_input_boundaries",
            allocator_authority_inputs(projection.authority_inputs),
        ),
        f(
            "authority_decision",
            V::InlineObject(vec![
                f(
                    "record_schema",
                    s(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SCHEMA),
                ),
                f(
                    "record_id",
                    s(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_ID),
                ),
                f(
                    "source_event_id",
                    record_event_or_null(projection.authority_decision_source_evidence_event_id),
                ),
                f("status_detail", s(projection.authority_decision_status)),
                f("reason", s(projection.authority_decision_reason)),
                f("present", b(projection.authority_decision_present)),
            ]),
        ),
        f(
            "registry_write_commit_gate",
            V::InlineObject(vec![
                f(
                    "record_schema",
                    s(MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SCHEMA),
                ),
                f(
                    "record_id",
                    s(MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_ID),
                ),
                f(
                    "source_event_id",
                    record_event_or_null(
                        projection.registry_write_commit_gate_source_evidence_event_id,
                    ),
                ),
                f(
                    "status_detail",
                    s(projection.registry_write_commit_gate_status),
                ),
                f("reason", s(projection.registry_write_commit_gate_reason)),
                f("present", b(projection.registry_write_commit_gate_present)),
            ]),
        ),
        f(
            "retained_service_slot_reservation_status",
            s(module_load_gate_service_slot_state(binding)),
        ),
        f(
            "retained_service_slot_reservation_reason",
            s(module_load_gate_service_slot_reason(binding)),
        ),
        f("service_slot_allocator_ready", b(projection.ready)),
    ]
}

fn execution_commit_gate_facts(
    p: ModuleLoadGateLoaderRuntimeExecutionCommitGateProjection,
) -> V<'static> {
    V::InlineObject(vec![
        f(
            "record_schema",
            s(MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SCHEMA),
        ),
        f(
            "record_id",
            s(MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_ID),
        ),
        f(
            "source_event_id",
            record_event_or_null(p.source_evidence_event_id),
        ),
        f("status_detail", s(p.status)),
        f("reason", s(p.reason)),
        f("present", b(p.present)),
        f("source_chain_complete", b(p.source_chain_complete)),
        f(
            "authority_decision_present",
            b(p.authority_decision_present),
        ),
        f(
            "loader_runtime_contract_present",
            b(p.loader_runtime_contract_present),
        ),
        f(
            "loader_runtime_source_evidence_complete",
            b(p.loader_runtime_source_evidence_complete),
        ),
        f(
            "service_slot_binding_source_evidence_present",
            b(p.service_slot_binding_source_evidence_present),
        ),
        f(
            "service_slot_binding_fact_present",
            b(p.service_slot_binding_fact_present),
        ),
        f(
            "audit_rollback_write_boundary_source_evidence_present",
            b(p.audit_rollback_write_boundary_source_evidence_present),
        ),
        f(
            "audit_rollback_write_boundary_fact_present",
            b(p.audit_rollback_write_boundary_fact_present),
        ),
        f(
            "retained_service_slot_reservation_present",
            b(p.retained_service_slot_reservation_present),
        ),
    ])
}

fn descriptor_intake_facts(
    p: ModuleLoadGateLoaderDescriptorIntakeBoundaryProjection,
) -> V<'static> {
    V::InlineObject(vec![
        f(
            "record_schema",
            s(MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SCHEMA),
        ),
        f("record_id", s(MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_ID)),
        f(
            "source_event_id",
            record_event_or_null(p.source_evidence_event_id),
        ),
        f("status_detail", s(p.status)),
        f("reason", s(p.reason)),
        f("present", b(p.present)),
        f("source_chain_complete", b(p.source_chain_complete)),
        f(
            "registry_write_commit_gate_present",
            b(p.registry_write_commit_gate_present),
        ),
        f(
            "execution_commit_gate_present",
            b(p.execution_commit_gate_present),
        ),
        f(
            "loader_runtime_source_evidence_complete",
            b(p.loader_runtime_source_evidence_complete),
        ),
        f(
            "retained_module_evidence_present",
            b(p.retained_module_evidence_present),
        ),
        f(
            "retained_service_slot_reservation_present",
            b(p.retained_service_slot_reservation_present),
        ),
    ])
}

fn artifact_intake_facts(
    p: ModuleLoadGateLoaderArtifactByteIntakeBoundaryProjection,
) -> V<'static> {
    V::InlineObject(vec![
        f(
            "record_schema",
            s(MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SCHEMA),
        ),
        f(
            "record_id",
            s(MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_ID),
        ),
        f(
            "source_event_id",
            record_event_or_null(p.source_evidence_event_id),
        ),
        f("status_detail", s(p.status)),
        f("reason", s(p.reason)),
        f("present", b(p.present)),
        f("source_chain_complete", b(p.source_chain_complete)),
        f(
            "descriptor_intake_boundary_present",
            b(p.descriptor_intake_boundary_present),
        ),
        f(
            "descriptor_intake_boundary_source_chain_complete",
            b(p.descriptor_intake_boundary_source_chain_complete),
        ),
        f(
            "execution_commit_gate_present",
            b(p.execution_commit_gate_present),
        ),
        f(
            "artifact_hash_binding_present",
            b(p.artifact_hash_binding_present),
        ),
        f(
            "retained_artifact_reference_present",
            b(p.retained_artifact_reference_present),
        ),
        f(
            "retained_module_evidence_present",
            b(p.retained_module_evidence_present),
        ),
        f(
            "retained_service_slot_reservation_present",
            b(p.retained_service_slot_reservation_present),
        ),
    ])
}

fn execution_authorization_facts(
    p: ModuleLoadGateLoaderExecutionAuthorizationBoundaryProjection,
) -> V<'static> {
    V::InlineObject(vec![
        f(
            "record_schema",
            s(MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SCHEMA),
        ),
        f(
            "record_id",
            s(MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_ID),
        ),
        f(
            "source_event_id",
            record_event_or_null(p.source_evidence_event_id),
        ),
        f("status_detail", s(p.status)),
        f("reason", s(p.reason)),
        f("present", b(p.present)),
        f("source_chain_complete", b(p.source_chain_complete)),
        f(
            "artifact_byte_intake_boundary_present",
            b(p.artifact_byte_intake_boundary_present),
        ),
        f(
            "artifact_byte_intake_boundary_source_chain_complete",
            b(p.artifact_byte_intake_boundary_source_chain_complete),
        ),
        f(
            "descriptor_intake_boundary_present",
            b(p.descriptor_intake_boundary_present),
        ),
        f(
            "descriptor_intake_boundary_source_chain_complete",
            b(p.descriptor_intake_boundary_source_chain_complete),
        ),
        f(
            "execution_commit_gate_present",
            b(p.execution_commit_gate_present),
        ),
        f(
            "entrypoint_abi_source_evidence_present",
            b(p.entrypoint_abi_source_evidence_present),
        ),
        f(
            "address_space_source_evidence_present",
            b(p.address_space_source_evidence_present),
        ),
        f(
            "memory_map_source_evidence_present",
            b(p.memory_map_source_evidence_present),
        ),
        f(
            "audit_rollback_write_boundary_source_evidence_present",
            b(p.audit_rollback_write_boundary_source_evidence_present),
        ),
        f(
            "retained_module_evidence_present",
            b(p.retained_module_evidence_present),
        ),
        f(
            "retained_service_slot_reservation_present",
            b(p.retained_service_slot_reservation_present),
        ),
    ])
}

fn registry_mutation_facts(
    p: ModuleLoadGateLoaderServiceRegistryMutationBoundaryProjection,
) -> V<'static> {
    V::InlineObject(vec![
        f(
            "record_schema",
            s(MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SCHEMA),
        ),
        f(
            "record_id",
            s(MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_ID),
        ),
        f(
            "source_event_id",
            record_event_or_null(p.source_evidence_event_id),
        ),
        f("status_detail", s(p.status)),
        f("reason", s(p.reason)),
        f("present", b(p.present)),
        f("source_chain_complete", b(p.source_chain_complete)),
        f(
            "execution_authorization_boundary_present",
            b(p.execution_authorization_boundary_present),
        ),
        f(
            "execution_authorization_boundary_source_chain_complete",
            b(p.execution_authorization_boundary_source_chain_complete),
        ),
        f(
            "registry_write_commit_gate_present",
            b(p.registry_write_commit_gate_present),
        ),
        f(
            "service_slot_binding_source_evidence_present",
            b(p.service_slot_binding_source_evidence_present),
        ),
        f(
            "retained_module_evidence_present",
            b(p.retained_module_evidence_present),
        ),
        f(
            "retained_service_slot_reservation_present",
            b(p.retained_service_slot_reservation_present),
        ),
    ])
}

fn live_boundary_facts(
    schema: &'static str,
    id: &'static str,
    p: ModuleLoadGateLoaderLiveLoadBoundaryProjection,
) -> V<'static> {
    V::InlineObject(vec![
        f("record_schema", s(schema)),
        f("record_id", s(id)),
        f(
            "source_event_id",
            record_event_or_null(p.source_evidence_event_id),
        ),
        f("status_detail", s(p.status)),
        f("reason", s(p.reason)),
        f("present", b(p.present)),
        f("source_chain_complete", b(p.source_chain_complete)),
        f(
            "load_attempt_boundary_present",
            b(p.load_attempt_boundary_present),
        ),
        f(
            "load_attempt_boundary_source_chain_complete",
            b(p.load_attempt_boundary_source_chain_complete),
        ),
        f(
            "artifact_load_boundary_present",
            b(p.artifact_load_boundary_present),
        ),
        f(
            "artifact_load_boundary_source_chain_complete",
            b(p.artifact_load_boundary_source_chain_complete),
        ),
        f(
            "executable_mapping_boundary_present",
            b(p.executable_mapping_boundary_present),
        ),
        f(
            "executable_mapping_boundary_source_chain_complete",
            b(p.executable_mapping_boundary_source_chain_complete),
        ),
        f(
            "entrypoint_transfer_boundary_present",
            b(p.entrypoint_transfer_boundary_present),
        ),
        f(
            "entrypoint_transfer_boundary_source_chain_complete",
            b(p.entrypoint_transfer_boundary_source_chain_complete),
        ),
        f(
            "service_start_boundary_present",
            b(p.service_start_boundary_present),
        ),
        f(
            "service_start_boundary_source_chain_complete",
            b(p.service_start_boundary_source_chain_complete),
        ),
        f(
            "service_health_binding_boundary_present",
            b(p.service_health_binding_boundary_present),
        ),
        f(
            "service_health_binding_boundary_source_chain_complete",
            b(p.service_health_binding_boundary_source_chain_complete),
        ),
        f(
            "service_running_state_boundary_present",
            b(p.service_running_state_boundary_present),
        ),
        f(
            "service_running_state_boundary_source_chain_complete",
            b(p.service_running_state_boundary_source_chain_complete),
        ),
        f(
            "service_start_audit_boundary_present",
            b(p.service_start_audit_boundary_present),
        ),
        f(
            "service_start_audit_boundary_source_chain_complete",
            b(p.service_start_audit_boundary_source_chain_complete),
        ),
        f(
            "service_unload_cleanup_boundary_present",
            b(p.service_unload_cleanup_boundary_present),
        ),
        f(
            "service_unload_cleanup_boundary_source_chain_complete",
            b(p.service_unload_cleanup_boundary_source_chain_complete),
        ),
        f(
            "live_load_commit_boundary_present",
            b(p.live_load_commit_boundary_present),
        ),
        f(
            "live_load_commit_boundary_source_chain_complete",
            b(p.live_load_commit_boundary_source_chain_complete),
        ),
        f(
            "commit_audit_boundary_present",
            b(p.commit_audit_boundary_present),
        ),
        f(
            "commit_audit_boundary_source_chain_complete",
            b(p.commit_audit_boundary_source_chain_complete),
        ),
        f(
            "commit_rollback_boundary_present",
            b(p.commit_rollback_boundary_present),
        ),
        f(
            "commit_rollback_boundary_source_chain_complete",
            b(p.commit_rollback_boundary_source_chain_complete),
        ),
        f(
            "commit_result_boundary_present",
            b(p.commit_result_boundary_present),
        ),
        f(
            "commit_result_boundary_source_chain_complete",
            b(p.commit_result_boundary_source_chain_complete),
        ),
        f(
            "descriptor_acceptance_authority_boundary_present",
            b(p.descriptor_acceptance_authority_boundary_present),
        ),
        f(
            "descriptor_acceptance_authority_boundary_source_chain_complete",
            b(p.descriptor_acceptance_authority_boundary_source_chain_complete),
        ),
        f(
            "descriptor_parser_contract_boundary_present",
            b(p.descriptor_parser_contract_boundary_present),
        ),
        f(
            "descriptor_parser_contract_boundary_source_chain_complete",
            b(p.descriptor_parser_contract_boundary_source_chain_complete),
        ),
        f(
            "descriptor_parser_result_boundary_present",
            b(p.descriptor_parser_result_boundary_present),
        ),
        f(
            "descriptor_parser_result_boundary_source_chain_complete",
            b(p.descriptor_parser_result_boundary_source_chain_complete),
        ),
        f(
            "descriptor_schema_validation_boundary_present",
            b(p.descriptor_schema_validation_boundary_present),
        ),
        f(
            "descriptor_schema_validation_boundary_source_chain_complete",
            b(p.descriptor_schema_validation_boundary_source_chain_complete),
        ),
        f(
            "descriptor_capability_validation_boundary_present",
            b(p.descriptor_capability_validation_boundary_present),
        ),
        f(
            "descriptor_capability_validation_boundary_source_chain_complete",
            b(p.descriptor_capability_validation_boundary_source_chain_complete),
        ),
        f(
            "descriptor_load_plan_boundary_present",
            b(p.descriptor_load_plan_boundary_present),
        ),
        f(
            "descriptor_load_plan_boundary_source_chain_complete",
            b(p.descriptor_load_plan_boundary_source_chain_complete),
        ),
        f(
            "executable_load_plan_authority_boundary_present",
            b(p.executable_load_plan_authority_boundary_present),
        ),
        f(
            "executable_load_plan_authority_boundary_source_chain_complete",
            b(p.executable_load_plan_authority_boundary_source_chain_complete),
        ),
        f(
            "executable_load_plan_result_boundary_present",
            b(p.executable_load_plan_result_boundary_present),
        ),
        f(
            "executable_load_plan_result_boundary_source_chain_complete",
            b(p.executable_load_plan_result_boundary_source_chain_complete),
        ),
        f(
            "executable_image_layout_boundary_present",
            b(p.executable_image_layout_boundary_present),
        ),
        f(
            "executable_image_layout_boundary_source_chain_complete",
            b(p.executable_image_layout_boundary_source_chain_complete),
        ),
        f(
            "executable_page_mapping_plan_boundary_present",
            b(p.executable_page_mapping_plan_boundary_present),
        ),
        f(
            "executable_page_mapping_plan_boundary_source_chain_complete",
            b(p.executable_page_mapping_plan_boundary_source_chain_complete),
        ),
        f(
            "executable_page_mapping_boundary_present",
            b(p.executable_page_mapping_boundary_present),
        ),
        f(
            "executable_page_mapping_boundary_source_chain_complete",
            b(p.executable_page_mapping_boundary_source_chain_complete),
        ),
        f(
            "descriptor_executable_page_binding_boundary_present",
            b(p.descriptor_executable_page_binding_boundary_present),
        ),
        f(
            "descriptor_executable_page_binding_boundary_source_chain_complete",
            b(p.descriptor_executable_page_binding_boundary_source_chain_complete),
        ),
        f(
            "executable_entrypoint_binding_boundary_present",
            b(p.executable_entrypoint_binding_boundary_present),
        ),
        f(
            "executable_entrypoint_binding_boundary_source_chain_complete",
            b(p.executable_entrypoint_binding_boundary_source_chain_complete),
        ),
        f(
            "executable_entrypoint_transfer_authorization_boundary_present",
            b(p.executable_entrypoint_transfer_authorization_boundary_present),
        ),
        f(
            "executable_entrypoint_transfer_authorization_boundary_source_chain_complete",
            b(p.executable_entrypoint_transfer_authorization_boundary_source_chain_complete),
        ),
        f(
            "executable_entrypoint_transfer_boundary_present",
            b(p.executable_entrypoint_transfer_boundary_present),
        ),
        f(
            "executable_entrypoint_transfer_boundary_source_chain_complete",
            b(p.executable_entrypoint_transfer_boundary_source_chain_complete),
        ),
        f(
            "executable_entrypoint_handoff_boundary_present",
            b(p.executable_entrypoint_handoff_boundary_present),
        ),
        f(
            "executable_entrypoint_handoff_boundary_source_chain_complete",
            b(p.executable_entrypoint_handoff_boundary_source_chain_complete),
        ),
        f(
            "artifact_byte_intake_boundary_present",
            b(p.artifact_byte_intake_boundary_present),
        ),
        f(
            "artifact_byte_intake_boundary_source_chain_complete",
            b(p.artifact_byte_intake_boundary_source_chain_complete),
        ),
        f(
            "execution_authorization_boundary_present",
            b(p.execution_authorization_boundary_present),
        ),
        f(
            "execution_authorization_boundary_source_chain_complete",
            b(p.execution_authorization_boundary_source_chain_complete),
        ),
        f(
            "service_registry_mutation_boundary_present",
            b(p.service_registry_mutation_boundary_present),
        ),
        f(
            "service_registry_mutation_boundary_source_chain_complete",
            b(p.service_registry_mutation_boundary_source_chain_complete),
        ),
        f(
            "service_slot_binding_source_evidence_present",
            b(p.service_slot_binding_source_evidence_present),
        ),
        f(
            "health_state_hooks_source_evidence_present",
            b(p.health_state_hooks_source_evidence_present),
        ),
        f(
            "artifact_hash_binding_present",
            b(p.artifact_hash_binding_present),
        ),
        f(
            "entrypoint_abi_source_evidence_present",
            b(p.entrypoint_abi_source_evidence_present),
        ),
        f(
            "address_space_source_evidence_present",
            b(p.address_space_source_evidence_present),
        ),
        f(
            "memory_map_source_evidence_present",
            b(p.memory_map_source_evidence_present),
        ),
        f(
            "capability_import_table_source_evidence_present",
            b(p.capability_import_table_source_evidence_present),
        ),
        f(
            "audit_rollback_write_boundary_source_evidence_present",
            b(p.audit_rollback_write_boundary_source_evidence_present),
        ),
        f(
            "retained_module_evidence_present",
            b(p.retained_module_evidence_present),
        ),
        f(
            "retained_artifact_reference_present",
            b(p.retained_artifact_reference_present),
        ),
        f(
            "retained_service_slot_reservation_present",
            b(p.retained_service_slot_reservation_present),
        ),
    ])
}

fn loader_runtime_facts(
    binding: event_log::ModuleLoadGateBinding,
    receiver: agent_protocol::DistributionReceiverIdentityLoadPreflightProjection,
) -> Vec<Field<'static>> {
    let mut source_map = Vec::with_capacity(module_load_gate_loader_runtime_source_fact_count());
    let mut runtime_facts = Vec::with_capacity(module_load_gate_loader_runtime_source_fact_count());
    for source in MODULE_LOADER_RUNTIME_FACT_SOURCES {
        source_map.push(V::InlineObject(
            module_load_gate_loader_runtime_source_fact_fields(source),
        ));
        runtime_facts.push(f(
            source.name,
            V::InlineObject(vec![
                f("record_schema", s(source.schema)),
                f("record_id", s(source.id)),
                f("source_method", s(source.source_method)),
                f("source_fact_locator", s(source.source_fact_locator)),
                f("scope", s("current_boot")),
                f("classification", s("local_only")),
                f("status_detail", s("missing")),
                f("reason", s(source.missing_reason)),
                f("present", no()),
            ]),
        ));
    }
    source_map.push(V::InlineObject(
        module_load_gate_receiver_identity_load_preflight_source_fact_fields(receiver),
    ));
    runtime_facts.push(f(
        MODULE_LOAD_GATE_RECEIVER_PREFLIGHT_SOURCE_FACT,
        V::InlineObject(
            module_load_gate_receiver_identity_load_preflight_runtime_fact_fields(receiver),
        ),
    ));

    let mut out = vec![
        f(
            "record_schema",
            s("raios.module_loader_runtime_readiness.v0"),
        ),
        f("state", s(module_load_gate_loader_runtime_state(binding))),
        f(
            "readiness_status",
            s(module_load_gate_loader_runtime_status(binding)),
        ),
        f(
            "readiness_reason",
            s(module_load_gate_loader_runtime_reason(binding)),
        ),
        f(
            "retained_module_evidence_state",
            s(module_load_gate_retained_module_evidence_state(binding)),
        ),
        f(
            "retained_module_evidence_reason",
            s(module_load_gate_retained_module_evidence_reason(binding)),
        ),
        f(
            "service_slot_allocator_ready",
            b(module_load_gate_service_slot_allocator_ready(binding)),
        ),
        f(
            "receiver_identity_load_preflight",
            V::InlineObject(module_load_gate_receiver_identity_load_preflight_fields(
                receiver,
            )),
        ),
        f(
            "m6_m7_reverify_input_check",
            V::InlineObject(module_load_gate_m6_m7_reverify_input_check_fields(receiver)),
        ),
        f(
            "execution_commit_gate",
            execution_commit_gate_facts(
                module_load_gate_loader_runtime_execution_commit_gate_projection(),
            ),
        ),
        f(
            "descriptor_intake_boundary",
            descriptor_intake_facts(
                module_load_gate_loader_descriptor_intake_boundary_projection(),
            ),
        ),
        f(
            "artifact_byte_intake_boundary",
            artifact_intake_facts(
                module_load_gate_loader_artifact_byte_intake_boundary_projection(),
            ),
        ),
        f(
            "execution_authorization_boundary",
            execution_authorization_facts(
                module_load_gate_loader_execution_authorization_boundary_projection(),
            ),
        ),
        f(
            "service_registry_mutation_boundary",
            registry_mutation_facts(
                module_load_gate_loader_service_registry_mutation_boundary_projection(),
            ),
        ),
    ];
    macro_rules! boundary {
        ($name:literal, $schema:expr, $id:expr, $projection:expr) => {
            out.push(f($name, live_boundary_facts($schema, $id, $projection)));
        };
    }
    boundary!(
        "load_attempt_boundary",
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SCHEMA,
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_ID,
        module_load_gate_loader_load_attempt_boundary_projection()
    );
    boundary!(
        "artifact_load_boundary",
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SCHEMA,
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_ID,
        module_load_gate_loader_artifact_load_boundary_projection()
    );
    boundary!(
        "executable_mapping_boundary",
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_ID,
        module_load_gate_loader_executable_mapping_boundary_projection()
    );

    boundary!(
        "entrypoint_transfer_boundary",
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SCHEMA,
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_ID,
        module_load_gate_loader_entrypoint_transfer_boundary_projection()
    );
    boundary!(
        "service_start_boundary",
        MODULE_LOADER_SERVICE_START_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_START_BOUNDARY_ID,
        module_load_gate_loader_service_start_boundary_projection()
    );
    boundary!(
        "service_health_binding_boundary",
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_ID,
        module_load_gate_loader_service_health_binding_boundary_projection()
    );
    boundary!(
        "service_running_state_boundary",
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_ID,
        module_load_gate_loader_service_running_state_boundary_projection()
    );
    boundary!(
        "service_start_audit_boundary",
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_ID,
        module_load_gate_loader_service_start_audit_boundary_projection()
    );
    boundary!(
        "service_unload_cleanup_boundary",
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_ID,
        module_load_gate_loader_service_unload_cleanup_boundary_projection()
    );
    boundary!(
        "live_load_commit_boundary",
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SCHEMA,
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_ID,
        module_load_gate_loader_live_load_commit_boundary_projection()
    );
    boundary!(
        "commit_audit_boundary",
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SCHEMA,
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_ID,
        module_load_gate_loader_commit_audit_boundary_projection()
    );
    boundary!(
        "commit_rollback_boundary",
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SCHEMA,
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_ID,
        module_load_gate_loader_commit_rollback_boundary_projection()
    );
    boundary!(
        "commit_result_boundary",
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SCHEMA,
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_ID,
        module_load_gate_loader_commit_result_boundary_projection()
    );
    boundary!(
        "descriptor_acceptance_authority_boundary",
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_ID,
        module_load_gate_loader_descriptor_acceptance_authority_boundary_projection()
    );
    boundary!(
        "descriptor_parser_contract_boundary",
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_ID,
        module_load_gate_loader_descriptor_parser_contract_boundary_projection()
    );
    boundary!(
        "descriptor_parser_result_boundary",
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_ID,
        module_load_gate_loader_descriptor_parser_result_boundary_projection()
    );
    boundary!(
        "descriptor_schema_validation_boundary",
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_ID,
        module_load_gate_loader_descriptor_schema_validation_boundary_projection()
    );
    boundary!(
        "descriptor_capability_validation_boundary",
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_ID,
        module_load_gate_loader_descriptor_capability_validation_boundary_projection()
    );
    boundary!(
        "descriptor_load_plan_boundary",
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_ID,
        module_load_gate_loader_descriptor_load_plan_boundary_projection()
    );
    boundary!(
        "executable_load_plan_authority_boundary",
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_ID,
        module_load_gate_loader_executable_load_plan_authority_boundary_projection()
    );
    boundary!(
        "executable_load_plan_result_boundary",
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_ID,
        module_load_gate_loader_executable_load_plan_result_boundary_projection()
    );
    boundary!(
        "executable_image_layout_boundary",
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_ID,
        module_load_gate_loader_executable_image_layout_boundary_projection()
    );
    boundary!(
        "executable_page_mapping_plan_boundary",
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_ID,
        module_load_gate_loader_executable_page_mapping_plan_boundary_projection()
    );
    boundary!(
        "executable_page_mapping_boundary",
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_ID,
        module_load_gate_loader_executable_page_mapping_boundary_projection()
    );
    boundary!(
        "descriptor_executable_page_binding_boundary",
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_ID,
        module_load_gate_loader_descriptor_executable_page_binding_boundary_projection()
    );
    boundary!(
        "executable_entrypoint_binding_boundary",
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_ID,
        module_load_gate_loader_executable_entrypoint_binding_boundary_projection()
    );
    boundary!(
        "executable_entrypoint_transfer_authorization_boundary",
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_ID,
        module_load_gate_loader_executable_entrypoint_transfer_authorization_boundary_projection()
    );
    boundary!(
        "executable_entrypoint_transfer_boundary",
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_ID,
        module_load_gate_loader_executable_entrypoint_transfer_boundary_projection()
    );

    boundary!(
        "executable_entrypoint_handoff_boundary",
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_ID,
        module_load_gate_loader_executable_entrypoint_handoff_boundary_projection()
    );
    boundary!(
        "executable_entrypoint_invocation_boundary",
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_ID,
        module_load_gate_loader_executable_entrypoint_invocation_boundary_projection()
    );
    out.extend([
        f(
            "missing_facts",
            V::InlineArray(
                MODULE_LOADER_RUNTIME_FACT_SOURCES
                    .into_iter()
                    .map(|source| s(source.name))
                    .collect(),
            ),
        ),
        f(
            "source_fact_count",
            V::U64(module_load_gate_loader_runtime_source_fact_count() as u64),
        ),
        f(
            "source_fact_map_complete",
            b(module_load_gate_loader_runtime_source_fact_map_complete()),
        ),
        f("source_fact_map", V::InlineArray(source_map)),
        f("loader_runtime_facts", V::InlineObject(runtime_facts)),
    ]);
    out
}

fn module_load_gate_projection_input(
    binding: &event_log::ModuleLoadGateBinding,
) -> LoadGateProjectionInput<'_> {
    // `binding` (the reference) is what the fact builders borrow from, so their
    // Vec<Field<'a>> outlives this function. `copy` is the by-value form the
    // Copy-taking state/validity accessors want. Shadowing the reference with the
    // copy would make the returned facts borrow a local: E0515.
    let copy = *binding;
    let receiver = agent_protocol::receiver_identity_load_preflight_projection();
    let allocator = module_load_gate_service_slot_allocator_projection(copy);
    let manifest_valid = module_load_gate_manifest_reference_valid(copy);
    let manifest_rejected = module_load_gate_manifest_reference_rejected(copy);
    let artifact_valid = module_load_gate_candidate_artifact_reference_valid(copy);
    let artifact_rejected = module_load_gate_candidate_artifact_reference_rejected(copy);
    let report_valid = module_load_gate_vm_test_report_reference_valid(copy);
    let report_rejected = module_load_gate_vm_test_report_reference_rejected(copy);
    let attestation_valid = module_load_gate_local_attestation_reference_valid(copy);
    let attestation_rejected = module_load_gate_local_attestation_reference_rejected(copy);
    let approval_valid = module_load_gate_local_approval_reference_valid(copy);
    let approval_rejected = module_load_gate_local_approval_reference_rejected(copy);
    let audit_valid = module_load_gate_audit_rollback_reference_valid(copy);
    let audit_rejected = module_load_gate_audit_rollback_reference_rejected(copy);
    let slot_valid = module_load_gate_service_slot_reservation_valid(copy);
    let slot_rejected = module_load_gate_service_slot_reservation_rejected(copy);

    LoadGateProjectionInput {
        module_manifest: prerequisite(
            retained_status(manifest_valid, manifest_rejected),
            module_load_gate_manifest_state(copy),
            module_load_gate_manifest_reason(copy),
            binding.manifest_reference_event_id,
            module_manifest_facts(copy),
        ),
        candidate_artifact: prerequisite(
            retained_status(artifact_valid, artifact_rejected),
            module_load_gate_candidate_artifact_state(copy),
            module_load_gate_candidate_artifact_reason(copy),
            binding.artifact_reference_event_id,
            candidate_artifact_facts(copy),
        ),
        vm_test_report: prerequisite(
            retained_status(report_valid, report_rejected),
            module_load_gate_vm_test_report_state(copy),
            module_load_gate_vm_test_report_reason(copy),
            binding.vm_report_reference_event_id,
            vm_report_facts(copy),
        ),
        local_attestation: prerequisite(
            retained_status(attestation_valid, attestation_rejected),
            module_load_gate_local_attestation_state(copy),
            module_load_gate_local_attestation_reason(copy),
            binding.attestation_reference_event_id,
            local_attestation_facts(copy),
        ),
        local_approval: prerequisite(
            retained_status(approval_valid, approval_rejected),
            module_load_gate_local_approval_state(copy),
            module_load_gate_local_approval_reason(copy),
            binding.approval_reference_event_id,
            local_approval_facts(copy),
        ),
        computed_capability_grant: prerequisite(
            if binding.retained_reference.is_some() {
                LoadGateEvidenceStatus::Verified
            } else {
                LoadGateEvidenceStatus::Missing
            },
            module_load_gate_computed_grant_state(copy),
            module_load_gate_computed_grant_reason(copy),
            binding.retained_reference_event_id,
            computed_grant_facts(copy),
        ),
        durable_audit_record: prerequisite(
            retained_status(audit_valid, audit_rejected),
            module_load_gate_durable_audit_state(copy),
            module_load_gate_durable_audit_reason(copy),
            binding.audit_rollback_reference_event_id,
            audit_rollback_facts(binding, false),
        ),
        rollback_plan: prerequisite(
            retained_status(audit_valid, audit_rejected),
            module_load_gate_rollback_state(copy),
            module_load_gate_rollback_reason(copy),
            binding.audit_rollback_reference_event_id,
            audit_rollback_facts(binding, true),
        ),
        service_slot: prerequisite(
            retained_status(slot_valid, slot_rejected),
            module_load_gate_service_slot_state(copy),
            module_load_gate_service_slot_reason(copy),
            binding.service_slot_reservation_event_id,
            service_slot_facts(binding),
        ),
        service_slot_allocator: prerequisite(
            LoadGateEvidenceStatus::Present,
            allocator.state,
            allocator.reason,
            allocator.authority_source_evidence_event_id,
            allocator_facts(copy, allocator),
        ),
        loader_runtime: prerequisite(
            LoadGateEvidenceStatus::Present,
            module_load_gate_loader_runtime_state(copy),
            module_load_gate_loader_runtime_reason(copy),
            None,
            loader_runtime_facts(copy, receiver),
        ),
        loader: prerequisite(
            LoadGateEvidenceStatus::Unavailable,
            "unavailable",
            "module_loader_unimplemented",
            None,
            vec![],
        ),
    }
}

pub(crate) fn project_module_load_gate_event_binding(
    binding: &event_log::ModuleLoadGateBinding,
) -> raios_core::event_evidence_projection::ClassifiedEventFacts<'_> {
    raios_core::event_evidence_projection::project_load_gate_event_binding(
        module_load_gate_projection_input(binding),
    )
}

fn emit_load_gate_v1(
    method: &'static str,
    event_id: Option<event_log::EventId>,
    binding: event_log::ModuleLoadGateBinding,
    framed: bool,
) {
    let projection = project_load_gate_denial(module_load_gate_projection_input(&binding));

    let facts = V::InlineObject(vec![
        f(
            "message",
            s("ephemeral module loading is denied until all load-gate evidence is authoritative"),
        ),
        f(
            "request",
            V::InlineObject(vec![
                f("load_mode", s("ram_only")),
                f("requested_capability", s("cap.module.load_ephemeral")),
                f("risk", s("modify_ram")),
                f("target", s("live_service_graph")),
                f("subject", s("agent.session.serial")),
            ]),
        ),
        f(
            "runtime",
            V::InlineObject(vec![f("persistence", s("none"))]),
        ),
        f(
            "required",
            V::InlineArray(
                MODULE_LOAD_GATE_REQUIRED
                    .iter()
                    .map(|item| s(item))
                    .collect(),
            ),
        ),
    ]);
    let envelope = Envelope {
        response_sequence: NEXT_EVIDENCE_RESPONSE_ID.fetch_add(1, Ordering::Relaxed),
        family: "module.load_gate",
        scope: "current_boot",
        classification: "local_only",
        source_method: method,
        event_sequence: event_id.map(event_log::EventId::sequence),
        facts,
        evidence: projection
            .evidence
            .into_iter()
            .map(ev::evidence_value)
            .collect(),
        decision: projection.decision,
    };
    let response = ev::response_value(&envelope);
    if framed {
        raw_fmt(format_args!("RAIOS_AGENT_BEGIN {}\r\n", method));
    }
    emit_record_value_fragment(response, 0);
    if framed {
        crlf();
        raw_fmt(format_args!("RAIOS_AGENT_END {}\r\n", method));
    }
}

pub(crate) fn emit_module_load_ephemeral_denied(
    method: &'static str,
    event_id: event_log::EventId,
    binding: event_log::ModuleLoadGateBinding,
) {
    emit_load_gate_v1(method, Some(event_id), binding, true);
}
// Event-binding rendering stays on the pre-v1 compact vocabulary: memory.recent_events event records are EVENT-family output and convert in P4-4, not P4-2.
pub(crate) fn emit_module_load_gate_event_binding(binding: event_log::ModuleLoadGateBinding) {
    emit_binding_object_direct(
        &binding,
        "",
        MODULE_LOAD_GATE_EVENT_BINDING_FIELDS,
        emit_module_load_gate_event_binding_value,
    );
}

#[rustfmt::skip]
fn module_load_gate_blocker_value(
    gate: &'static str,
    state: &'static str,
    reason: &'static str,
) -> V<'static> {
    V::InlineObject(vec![f("gate", s(gate)), f("state", s(state)), f("reason", s(reason))])
}

#[rustfmt::skip]
fn module_load_gate_gate_state_value(binding: event_log::ModuleLoadGateBinding) -> V<'static> {
    V::InlineObject(vec![
        f("module_manifest", s(module_load_gate_manifest_state(binding))),
        f("candidate_artifact", s(module_load_gate_candidate_artifact_state(binding))),
        f("vm_test_report", s(module_load_gate_vm_test_report_state(binding))),
        f("local_attestation", s(module_load_gate_local_attestation_state(binding))),
        f("computed_capability_grant", s(module_load_gate_computed_grant_state(binding))),
        f("local_approval", s(module_load_gate_local_approval_state(binding))),
        f("rollback_plan", s(module_load_gate_rollback_state(binding))),
        f("durable_audit_record", s(module_load_gate_durable_audit_state(binding))),
        f("service_slot", s(module_load_gate_service_slot_state(binding))),
        f("service_slot_allocator", s(module_load_gate_service_slot_allocator_state(binding))),
        f("loader_runtime", s(module_load_gate_loader_runtime_state(binding))),
        f("loader", s("unavailable")),
        f("artifact_loaded", no()),
        f("service_started", no()),
        f("persistence", s("none")),
        f("can_load", no()),
    ])
}

#[rustfmt::skip]
fn module_load_gate_blocked_by_value(binding: event_log::ModuleLoadGateBinding) -> V<'static> {
    V::InlineArray(vec![
        module_load_gate_blocker_value("module_manifest", module_load_gate_manifest_state(binding), module_load_gate_manifest_reason(binding)),
        module_load_gate_blocker_value("candidate_artifact", module_load_gate_candidate_artifact_state(binding), module_load_gate_candidate_artifact_reason(binding)),
        module_load_gate_blocker_value("vm_test_report", module_load_gate_vm_test_report_state(binding), module_load_gate_vm_test_report_reason(binding)),
        module_load_gate_blocker_value("local_attestation", module_load_gate_local_attestation_state(binding), module_load_gate_local_attestation_reason(binding)),
        module_load_gate_blocker_value("local_approval", module_load_gate_local_approval_state(binding), module_load_gate_local_approval_reason(binding)),
        module_load_gate_blocker_value("computed_capability_grant", module_load_gate_computed_grant_state(binding), module_load_gate_computed_grant_reason(binding)),
        module_load_gate_blocker_value("durable_audit_record", module_load_gate_durable_audit_state(binding), module_load_gate_durable_audit_reason(binding)),
        module_load_gate_blocker_value("rollback_plan", module_load_gate_rollback_state(binding), module_load_gate_rollback_reason(binding)),
        module_load_gate_blocker_value("service_slot", module_load_gate_service_slot_state(binding), module_load_gate_service_slot_reason(binding)),
        module_load_gate_blocker_value("service_slot_allocator", module_load_gate_service_slot_allocator_state(binding), module_load_gate_service_slot_allocator_reason(binding)),
        module_load_gate_blocker_value("loader_runtime", module_load_gate_loader_runtime_state(binding), module_load_gate_loader_runtime_reason(binding)),
        module_load_gate_blocker_value("loader", "unavailable", "module_loader_unimplemented"),
    ])
}

fn module_load_gate_required_value() -> V<'static> {
    V::InlineArray(
        MODULE_LOAD_GATE_REQUIRED
            .iter()
            .map(|item| {
                s(if *item == "computed_capability_grant" {
                    "raios.computed_capability_grant.v0"
                } else {
                    *item
                })
            })
            .collect(),
    )
}

define_direct_binding_fields! { ModuleLoadGateEventBindingField, MODULE_LOAD_GATE_EVENT_BINDING_FIELDS, emit_module_load_gate_event_binding_value, event_log::ModuleLoadGateBinding, binding, _kind;
    (Schema, "schema", { emit_record_value_fragment(s("raios.module_load_gate.v0"), 0); }), (Status, "status", { emit_record_value_fragment(s("denied_missing_evidence"), 0); }), (LoadMode, "load_mode", { emit_record_value_fragment(s("ram_only"), 0); }),
    (RequestedCapability, "requested_capability", { emit_record_value_fragment(s("cap.module.load_ephemeral"), 0); }), (Risk, "risk", { emit_record_value_fragment(s("modify_ram"), 0); }), (Target, "target", { emit_record_value_fragment(s("live_service_graph"), 0); }), (Subject, "subject", { emit_record_value_fragment(s("agent.session.serial"), 0); }),
    (GateState, "gate_state", {
            emit_record_value_fragment(module_load_gate_gate_state_value(*binding), 0);
        }),
    (RetainedModuleManifestReference, "retained_module_manifest_reference", { emit_module_load_gate_manifest_reference_compact(*binding); }), (RetainedCandidateArtifactReference, "retained_candidate_artifact_reference", { emit_module_load_gate_artifact_reference_compact(*binding); }),
    (RetainedVmTestReportReference, "retained_vm_test_report_reference", { emit_module_load_gate_vm_report_reference_compact(*binding); }), (RetainedLocalAttestationReference, "retained_local_attestation_reference", { emit_module_load_gate_local_attestation_reference_compact(*binding); }),
    (RetainedLocalApprovalReference, "retained_local_approval_reference", { emit_module_load_gate_local_approval_reference_compact(*binding); }), (RetainedComputedGrantReference, "retained_computed_grant_reference", { emit_module_load_gate_retained_reference_compact(*binding); }),
    (RetainedAuditRollbackReference, "retained_audit_rollback_reference", { emit_module_load_gate_audit_rollback_reference_compact(*binding); }), (RetainedServiceSlotReservation, "retained_service_slot_reservation", { emit_module_load_gate_service_slot_reservation_compact(*binding); }),
    (ServiceSlotAllocatorReadiness, "service_slot_allocator_readiness", { emit_module_load_gate_service_slot_allocator_readiness_compact(*binding); }),
    (LoaderRuntimeReadiness, "loader_runtime_readiness", { emit_module_load_gate_loader_runtime_readiness_compact(*binding); }),
    (AuditRollbackRequirements, "audit_rollback_requirements", { emit_module_load_gate_audit_rollback_requirements_compact(*binding); }),
    (BlockedBy, "blocked_by", {
            emit_record_value_fragment(module_load_gate_blocked_by_value(*binding), 0);
        }),
    (Required, "required", {
        emit_record_value_fragment(module_load_gate_required_value(), 0);
    }),
    (Evidence, "evidence", {
            raw("{\"event_scope\": \"current_boot\", ");
            emit_module_load_gate_evidence_hashes_compact(*binding);
            raw(", \"service_inventory_change\": \"none\", \"load_attempted\": false}");
        }),
}

fn emit_module_load_gate_manifest_reference_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.manifest_reference {
        if module_load_gate_manifest_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.manifest_reference_event_id);
            raw(", \"schema\": \"raios.module_manifest_reference.v0\", \"status\": ");
            json_str(binding.manifest_reference_status);
            raw(", \"reason\": ");
            json_str(binding.manifest_reference_reason);
            raw(", \"classification\": \"local_only\", \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.manifest_reference_event_id);
        raw(concat!(
            ", \"schema\": \"raios.module_manifest_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\"",
            ", \"accepts_manifest_json\": false, \"accepts_artifact_bytes\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false, \"can_load_now\": false",
            ", \"service_inventory_change\": \"none\", \"load_attempted\": false, \"hashes\": {\"manifest_reference_hash\": ",
        ));
        json_sha256(reference.manifest_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_manifest_reference.v0\", \"status\": ");
        json_str(binding.manifest_reference_status);
        raw(", \"reason\": ");
        json_str(binding.manifest_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_artifact_reference_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.artifact_reference {
        if module_load_gate_candidate_artifact_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.artifact_reference_event_id);
            raw(", \"schema\": \"raios.module_candidate_artifact_reference.v0\", \"status\": ");
            json_str(binding.artifact_reference_status);
            raw(", \"reason\": ");
            json_str(binding.artifact_reference_reason);
            raw(", \"classification\": \"local_only\", \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.artifact_reference_event_id);
        raw(concat!(
            ", \"schema\": \"raios.module_candidate_artifact_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\"",
            ", \"accepts_manifest_json\": false, \"accepts_artifact_bytes\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false, \"can_load_now\": false",
            ", \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_manifest_reference_event_id\": ",
        ));
        json_event_id(reference.retained_manifest_reference_event_id);
        raw(", \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw(", \"hashes\": {\"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw(", \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_candidate_artifact_reference.v0\", \"status\": ");
        json_str(binding.artifact_reference_status);
        raw(", \"reason\": ");
        json_str(binding.artifact_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_vm_report_reference_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.vm_report_reference {
        if module_load_gate_vm_test_report_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.vm_report_reference_event_id);
            raw(", \"schema\": \"raios.module_vm_test_report_reference.v0\", \"status\": ");
            json_str(binding.vm_report_reference_status);
            raw(", \"reason\": ");
            json_str(binding.vm_report_reference_reason);
            raw(", \"classification\": \"local_only\", \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.vm_report_reference_event_id);
        raw(concat!(
            ", \"schema\": \"raios.module_vm_test_report_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\"",
            ", \"accepts_manifest_json\": false, \"accepts_artifact_bytes\": false, \"accepts_vm_report_json\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false",
            ", \"can_load_now\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_manifest_reference_event_id\": ",
        ));
        json_event_id(reference.retained_manifest_reference_event_id);
        raw(", \"retained_candidate_artifact_reference_event_id\": ");
        json_event_id(reference.retained_artifact_reference_event_id);
        raw(", \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw(", \"hashes\": {\"vm_test_report_reference_hash\": ");
        json_sha256(reference.report_reference_hash);
        raw(", \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_vm_test_report_reference.v0\", \"status\": ");
        json_str(binding.vm_report_reference_status);
        raw(", \"reason\": ");
        json_str(binding.vm_report_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_local_attestation_reference_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    if let Some(reference) = binding.attestation_reference {
        if module_load_gate_local_attestation_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.attestation_reference_event_id);
            raw(", \"schema\": \"raios.module_local_attestation_reference.v0\", \"status\": ");
            json_str(binding.attestation_reference_status);
            raw(", \"reason\": ");
            json_str(binding.attestation_reference_reason);
            raw(", \"classification\": \"local_only\", \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.attestation_reference_event_id);
        raw(concat!(
            ", \"schema\": \"raios.module_local_attestation_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\"",
            ", \"accepts_local_attestation_json\": false, \"accepts_artifact_bytes\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false, \"can_load_now\": false",
            ", \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_manifest_reference_event_id\": ",
        ));
        json_event_id(reference.retained_manifest_reference_event_id);
        raw(", \"retained_candidate_artifact_reference_event_id\": ");
        json_event_id(reference.retained_artifact_reference_event_id);
        raw(", \"retained_vm_test_report_reference_event_id\": ");
        json_event_id(reference.retained_vm_report_reference_event_id);
        raw(", \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw(", \"hashes\": {\"local_attestation_reference_hash\": ");
        json_sha256(reference.attestation_reference_hash);
        raw(", \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw(", \"vm_test_report_reference_hash\": ");
        json_sha256(reference.vm_report_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_local_attestation_reference.v0\", \"status\": ");
        json_str(binding.attestation_reference_status);
        raw(", \"reason\": ");
        json_str(binding.attestation_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_local_approval_reference_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    if let Some(reference) = binding.approval_reference {
        if module_load_gate_local_approval_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.approval_reference_event_id);
            raw(", \"schema\": \"raios.module_local_approval_reference.v0\", \"status\": ");
            json_str(binding.approval_reference_status);
            raw(", \"reason\": ");
            json_str(binding.approval_reference_reason);
            raw(", \"classification\": \"local_only\", \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.approval_reference_event_id);
        raw(concat!(
            ", \"schema\": \"raios.module_local_approval_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\"",
            ", \"accepts_local_approval_text\": false, \"accepts_artifact_bytes\": false, \"accepts_unsigned_service_code\": false, \"authorizes_guest_load\": false, \"can_load_now\": false",
            ", \"service_inventory_change\": \"none\", \"load_attempted\": false, \"retained_manifest_reference_event_id\": ",
        ));
        json_event_id(reference.retained_manifest_reference_event_id);
        raw(", \"retained_candidate_artifact_reference_event_id\": ");
        json_event_id(reference.retained_artifact_reference_event_id);
        raw(", \"retained_vm_test_report_reference_event_id\": ");
        json_event_id(reference.retained_vm_report_reference_event_id);
        raw(", \"retained_local_attestation_reference_event_id\": ");
        json_event_id(reference.retained_local_attestation_reference_event_id);
        raw(", \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw(", \"hashes\": {\"local_approval_reference_hash\": ");
        json_sha256(reference.approval_reference_hash);
        raw(", \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw(", \"vm_test_report_reference_hash\": ");
        json_sha256(reference.vm_report_reference_hash);
        raw(", \"local_attestation_reference_hash\": ");
        json_sha256(reference.local_attestation_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_local_approval_reference.v0\", \"status\": ");
        json_str(binding.approval_reference_status);
        raw(", \"reason\": ");
        json_str(binding.approval_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_retained_reference_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.retained_reference {
        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.retained_reference_event_id);
        raw(", \"schema\": \"raios.module_computed_grant_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\", \"grants_capability\": false, \"grants_load_now\": false, \"authorizes_guest_load\": false, \"can_load_now\": false, \"load_attempted\": false, \"hashes\": {\"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_computed_grant_reference.v0\", \"status\": \"missing\", \"reason\": \"no_valid_computed_grant_reference_retained\", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_audit_rollback_reference_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    if let Some(reference) = binding.audit_rollback_reference {
        if module_load_gate_audit_rollback_reference_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.audit_rollback_reference_event_id);
            raw(", \"schema\": \"raios.module_audit_rollback_reference.v0\", \"status\": ");
            json_str(binding.audit_rollback_reference_status);
            raw(", \"reason\": ");
            json_str(binding.audit_rollback_reference_reason);
            raw(", \"classification\": \"local_only\", \"durable_audit_written\": false, \"rollback_plan_installed\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.audit_rollback_reference_event_id);
        raw(concat!(
            ", \"schema\": \"raios.module_audit_rollback_reference.v0\", \"status\": \"retained_hash_reference_load_still_denied\", \"classification\": \"local_only\"",
            ", \"durable_audit_written\": false, \"rollback_plan_installed\": false, \"grants_capability\": false, \"grants_load_now\": false, \"authorizes_guest_load\": false",
            ", \"can_load_now\": false, \"load_attempted\": false, \"denial_event_id\": ",
        ));
        json_event_id(reference.denial_event_id);
        raw(", \"retained_computed_grant_reference_event_id\": ");
        json_event_id(reference.retained_reference_event_id);
        raw(", \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
        raw(", \"hashes\": {\"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw(", \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
        raw(", \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        raw(", \"cleanup_actions_hash\": ");
        json_sha256(reference.cleanup_actions_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_audit_rollback_reference.v0\", \"status\": ");
        json_str(binding.audit_rollback_reference_status);
        raw(", \"reason\": ");
        json_str(binding.audit_rollback_reference_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_service_slot_reservation_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    if let Some(reservation) = binding.service_slot_reservation {
        if module_load_gate_service_slot_reservation_rejected(binding) {
            raw("{\"state\": \"rejected\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
            json_event_id_option(binding.service_slot_reservation_event_id);
            raw(", \"schema\": \"raios.module_service_slot_reservation.v0\", \"status\": ");
            json_str(binding.service_slot_reservation_status);
            raw(", \"reason\": ");
            json_str(binding.service_slot_reservation_reason);
            raw(", \"classification\": \"local_only\", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"can_load_now\": false, \"load_attempted\": false}");
            return;
        }

        raw("{\"state\": \"present\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": ");
        json_event_id_option(binding.service_slot_reservation_event_id);
        raw(concat!(
            ", \"schema\": \"raios.module_service_slot_reservation.v0\", \"status\": \"retained_hash_reference_only_not_allocated\", \"classification\": \"local_only\"",
            ", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"grants_capability\": false, \"grants_load_now\": false, \"authorizes_guest_load\": false",
            ", \"can_load_now\": false, \"load_attempted\": false, \"retained_computed_grant_reference_event_id\": ",
        ));
        json_event_id(reservation.retained_reference_event_id);
        raw(", \"retained_audit_rollback_reference_event_id\": ");
        json_event_id(reservation.retained_audit_rollback_reference_event_id);
        raw(", \"ram_only_service_slot_id\": ");
        json_str(reservation.ram_only_service_slot_id.as_str());
        raw(", \"hashes\": {\"reservation_hash\": ");
        json_sha256(reservation.reservation_hash);
        raw(", \"computed_capability_grant_hash\": ");
        json_sha256(reservation.computed_grant_hash);
        raw(", \"audit_record_hash\": ");
        json_sha256(reservation.audit_record_hash);
        raw(", \"rollback_plan_hash\": ");
        json_sha256(reservation.rollback_plan_hash);
        raw(", \"pre_load_service_inventory_hash\": ");
        json_sha256(reservation.pre_load_service_inventory_hash);
        raw("}}");
    } else {
        raw("{\"state\": \"missing\", \"retention\": \"current_boot_ram_event_log\", \"event_id\": null, \"schema\": \"raios.module_service_slot_reservation.v0\", \"status\": ");
        json_str(binding.service_slot_reservation_status);
        raw(", \"reason\": ");
        json_str(binding.service_slot_reservation_reason);
        raw(", \"can_load_now\": false, \"load_attempted\": false}");
    }
}

fn emit_module_load_gate_service_slot_allocator_readiness_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    let projection = module_load_gate_service_slot_allocator_projection(binding);
    raw("{\"schema\": \"raios.module_service_slot_allocator_readiness.v0\", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"source_method\": \"module.service_slot_allocator\", \"state\": ");
    json_str(projection.state);
    raw(", \"readiness_status\": ");
    json_str(projection.status);
    raw(", \"readiness_reason\": ");
    json_str(projection.reason);
    raw(", \"allocator_authority_boundary\": {\"schema\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SCHEMA);
    raw(", \"source_evidence_event_id\": ");
    json_event_id_option(projection.authority_source_evidence_event_id);
    raw(", \"status\": ");
    json_str(projection.authority_status);
    raw(", \"reason\": ");
    json_str(projection.authority_reason);
    raw(", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    raw(", \"allocation_intent_boundary\": {\"schema\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SCHEMA);
    raw(", \"id\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATION_INTENT_ID);
    raw(", \"source_evidence_event_id\": ");
    json_event_id_option(projection.allocation_intent_source_evidence_event_id);
    raw(", \"status\": ");
    json_str(projection.allocation_intent_status);
    raw(", \"reason\": ");
    json_str(projection.allocation_intent_reason);
    raw(", \"present\": ");
    raw_bool(projection.allocation_intent_present);
    raw(", \"requested_capability\": \"cap.module.load_ephemeral\", \"load_mode\": \"ram_only\", \"target\": \"live_service_graph\", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    raw(", \"authority_input_boundaries\": ");
    emit_module_load_gate_authority_input_boundaries_compact(projection.authority_inputs);
    raw(", \"authority_decision\": ");
    emit_module_load_gate_authority_decision_compact(projection);
    raw(", \"registry_write_commit_gate\": ");
    emit_module_load_gate_registry_write_commit_gate_compact(projection);
    raw(", \"retained_service_slot_reservation_status\": ");
    json_str(module_load_gate_service_slot_state(binding));
    raw(", \"service_slot_allocator_ready\": ");
    raw_bool(module_load_gate_service_slot_allocator_ready(binding));
    raw(", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"can_load_now\": false, \"load_attempted\": false}");
}

fn emit_module_load_gate_loader_runtime_readiness_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    raw("{\"schema\": \"raios.module_loader_runtime_readiness.v0\", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"state\": ");
    json_str(module_load_gate_loader_runtime_state(binding));
    raw(", \"readiness_status\": ");
    json_str(module_load_gate_loader_runtime_status(binding));
    raw(", \"readiness_reason\": ");
    json_str(module_load_gate_loader_runtime_reason(binding));
    raw(", \"retained_module_evidence_state\": ");
    json_str(module_load_gate_retained_module_evidence_state(binding));
    raw(", \"retained_module_evidence_reason\": ");
    json_str(module_load_gate_retained_module_evidence_reason(binding));
    raw(", \"service_slot_allocator_ready\": ");
    raw_bool(module_load_gate_service_slot_allocator_ready(binding));
    raw(", \"m6_m7_reverify_input_check\": ");
    emit_inline_record_object_fragment(
        module_load_gate_m6_m7_reverify_input_check_fields(
            agent_protocol::receiver_identity_load_preflight_projection(),
        ),
        0,
    );
    raw(", \"execution_commit_gate\": ");
    emit_module_load_gate_loader_runtime_execution_commit_gate_compact();
    raw(", \"descriptor_intake_boundary\": ");
    emit_module_load_gate_loader_descriptor_intake_boundary_compact();
    raw(", \"artifact_byte_intake_boundary\": ");
    emit_module_load_gate_loader_artifact_byte_intake_boundary_compact();
    raw(", \"execution_authorization_boundary\": ");
    emit_module_load_gate_loader_execution_authorization_boundary_compact();
    raw(", \"service_registry_mutation_boundary\": ");
    emit_module_load_gate_loader_service_registry_mutation_boundary_compact();
    raw(", \"load_attempt_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SCHEMA,
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_ID,
        module_load_gate_loader_load_attempt_boundary_projection(),
    );
    raw(", \"artifact_load_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SCHEMA,
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_ID,
        module_load_gate_loader_artifact_load_boundary_projection(),
    );
    raw(", \"executable_mapping_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_ID,
        module_load_gate_loader_executable_mapping_boundary_projection(),
    );
    raw(", \"entrypoint_transfer_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SCHEMA,
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_ID,
        module_load_gate_loader_entrypoint_transfer_boundary_projection(),
    );
    raw(", \"service_start_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_SERVICE_START_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_START_BOUNDARY_ID,
        module_load_gate_loader_service_start_boundary_projection(),
    );
    raw(", \"service_health_binding_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_ID,
        module_load_gate_loader_service_health_binding_boundary_projection(),
    );
    raw(", \"service_running_state_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_ID,
        module_load_gate_loader_service_running_state_boundary_projection(),
    );
    raw(", \"service_start_audit_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_ID,
        module_load_gate_loader_service_start_audit_boundary_projection(),
    );
    raw(", \"service_unload_cleanup_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_ID,
        module_load_gate_loader_service_unload_cleanup_boundary_projection(),
    );
    raw(", \"live_load_commit_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SCHEMA,
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_ID,
        module_load_gate_loader_live_load_commit_boundary_projection(),
    );
    raw(", \"commit_audit_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SCHEMA,
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_ID,
        module_load_gate_loader_commit_audit_boundary_projection(),
    );
    raw(", \"commit_rollback_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SCHEMA,
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_ID,
        module_load_gate_loader_commit_rollback_boundary_projection(),
    );
    raw(", \"commit_result_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SCHEMA,
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_ID,
        module_load_gate_loader_commit_result_boundary_projection(),
    );
    raw(", \"descriptor_acceptance_authority_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_ID,
        module_load_gate_loader_descriptor_acceptance_authority_boundary_projection(),
    );
    raw(", \"descriptor_parser_contract_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_ID,
        module_load_gate_loader_descriptor_parser_contract_boundary_projection(),
    );
    raw(", \"descriptor_parser_result_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_ID,
        module_load_gate_loader_descriptor_parser_result_boundary_projection(),
    );
    raw(", \"descriptor_schema_validation_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_ID,
        module_load_gate_loader_descriptor_schema_validation_boundary_projection(),
    );
    raw(", \"descriptor_capability_validation_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_ID,
        module_load_gate_loader_descriptor_capability_validation_boundary_projection(),
    );
    raw(", \"descriptor_load_plan_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_ID,
        module_load_gate_loader_descriptor_load_plan_boundary_projection(),
    );
    raw(", \"executable_load_plan_authority_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_ID,
        module_load_gate_loader_executable_load_plan_authority_boundary_projection(),
    );
    raw(", \"executable_load_plan_result_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_ID,
        module_load_gate_loader_executable_load_plan_result_boundary_projection(),
    );
    raw(", \"executable_image_layout_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_ID,
        module_load_gate_loader_executable_image_layout_boundary_projection(),
    );
    raw(", \"executable_page_mapping_plan_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_ID,
        module_load_gate_loader_executable_page_mapping_plan_boundary_projection(),
    );
    raw(", \"executable_page_mapping_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_ID,
        module_load_gate_loader_executable_page_mapping_boundary_projection(),
    );
    raw(", \"descriptor_executable_page_binding_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_ID,
        module_load_gate_loader_descriptor_executable_page_binding_boundary_projection(),
    );
    raw(", \"executable_entrypoint_binding_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_ID,
        module_load_gate_loader_executable_entrypoint_binding_boundary_projection(),
    );
    raw(", \"executable_entrypoint_transfer_authorization_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_ID,
        module_load_gate_loader_executable_entrypoint_transfer_authorization_boundary_projection(),
    );
    raw(", \"executable_entrypoint_transfer_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_ID,
        module_load_gate_loader_executable_entrypoint_transfer_boundary_projection(),
    );
    raw(", \"executable_entrypoint_handoff_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_ID,
        module_load_gate_loader_executable_entrypoint_handoff_boundary_projection(),
    );
    raw(", \"executable_entrypoint_invocation_boundary\": ");
    emit_module_load_gate_loader_live_load_boundary_compact(
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_ID,
        module_load_gate_loader_executable_entrypoint_invocation_boundary_projection(),
    );
    raw(concat!(
        ", \"accepts_descriptor_bytes\": false, \"produces_parsed_descriptor\": false, \"validates_descriptor_schema\": false, \"produces_validated_descriptor\": false",
        ", \"validates_descriptor_capabilities\": false, \"produces_capability_validated_descriptor\": false, \"authorizes_executable_load_plan\": false",
        ", \"produces_executable_load_plan\": false, \"produces_executable_image_layout\": false, \"produces_executable_page_mapping_plan\": false",
        ", \"binds_capability_validated_descriptor_to_executable_pages\": false, \"parses_descriptor_bytes\": false, \"loads_artifact\": false, \"allocates_service_slot\": false",
        ", \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"starts_service\": false, \"marks_service_running\": false",
        ", \"creates_service_health_records\": false, \"writes_service_start_audit_record\": false, \"unloads_service\": false, \"cleans_up_service_slot\": false",
        ", \"commits_live_load\": false, \"writes_load_commit_audit_record\": false, \"installs_commit_rollback_record\": false, \"records_load_result\": false, \"can_load_now\": false",
        ", \"load_attempted\": false, \"missing_facts\": [\"raios.module_loader_identity.v0\", \"raios.module_loader_artifact_hash_binding.v0\", \"raios.module_loader_entrypoint_abi.v0\"",
        ", \"raios.module_loader_address_space_boundary.v0\", \"raios.module_loader_memory_map_constraints.v0\", \"raios.module_loader_capability_import_table.v0\"",
        ", \"raios.module_loader_service_slot_binding.v0\", \"raios.module_loader_health_state_hooks.v0\", \"raios.module_loader_rollback_hooks.v0\"",
        ", \"raios.module_loader_audit_rollback_write_boundary_binding.v0\"]",
        ));
    raw(", \"source_fact_count\": ");
    raw_fmt(format_args!(
        "{}",
        module_load_gate_loader_runtime_source_fact_count()
    ));
    raw(", \"source_fact_map_complete\": ");
    raw_bool(module_load_gate_loader_runtime_source_fact_map_complete());
    raw(", \"source_fact_map\": [");
    emit_module_load_gate_loader_runtime_source_fact_map_compact();
    raw("]}");
}

fn emit_module_load_gate_authority_input_boundaries_compact(
    inputs: [ModuleServiceSlotAuthorityInputProjection; MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT],
) {
    raw("{");
    let mut idx = 0usize;
    while idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        let input = inputs[idx];
        json_str(input.name);
        raw(": {\"schema\": ");
        json_str(input.schema);
        raw(", \"source_evidence_event_id\": ");
        json_event_id_option(input.source_evidence_event_id);
        raw(", \"status\": ");
        json_str(input.status);
        raw(", \"reason\": ");
        json_str(input.reason);
        raw(", \"present\": ");
        raw_bool(input.present);
        raw(", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
        if idx + 1 != MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
            raw(", ");
        }
        idx += 1;
    }
    raw("}");
}

fn emit_module_load_gate_authority_decision_compact(
    projection: ModuleLoadGateServiceSlotAllocatorProjection,
) {
    raw("{\"schema\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SCHEMA);
    raw(", \"id\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_ID);
    raw(", \"source_evidence_event_id\": ");
    json_event_id_option(projection.authority_decision_source_evidence_event_id);
    raw(", \"status\": ");
    json_str(projection.authority_decision_status);
    raw(", \"reason\": ");
    json_str(projection.authority_decision_reason);
    raw(", \"present\": ");
    raw_bool(projection.authority_decision_present);
    raw(", \"requested_capability\": \"cap.module.load_ephemeral\", \"load_mode\": \"ram_only\", \"target\": \"live_service_graph\", \"authorizes_allocation\": false, \"authorizes_load\": false, \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
}

fn emit_module_load_gate_registry_write_commit_gate_compact(
    projection: ModuleLoadGateServiceSlotAllocatorProjection,
) {
    raw("{\"schema\": ");
    json_str(MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SCHEMA);
    raw(", \"id\": ");
    json_str(MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_ID);
    raw(", \"source_evidence_event_id\": ");
    json_event_id_option(projection.registry_write_commit_gate_source_evidence_event_id);
    raw(", \"status\": ");
    json_str(projection.registry_write_commit_gate_status);
    raw(", \"reason\": ");
    json_str(projection.registry_write_commit_gate_reason);
    raw(", \"present\": ");
    raw_bool(projection.registry_write_commit_gate_present);
    raw(concat!(
        ", \"requested_capability\": \"cap.module.load_ephemeral\", \"load_mode\": \"ram_only\", \"target\": \"live_service_graph\", \"authorizes_registry_write\": false",
        ", \"mutates_service_registry\": false, \"writes_durable_audit_state\": false, \"installs_rollback_state\": false, \"authorizes_allocation\": false, \"authorizes_load\": false",
        ", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"loads_artifact\": false, \"load_attempted\": false}",
        ));
}

fn emit_module_load_gate_loader_runtime_execution_commit_gate_compact() {
    let projection = module_load_gate_loader_runtime_execution_commit_gate_projection();
    raw("{\"schema\": ");
    json_str(MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SCHEMA);
    raw(", \"id\": ");
    json_str(MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_ID);
    raw(", \"source_evidence_event_id\": ");
    json_event_id_option(projection.source_evidence_event_id);
    raw(", \"status\": ");
    json_str(projection.status);
    raw(", \"reason\": ");
    json_str(projection.reason);
    raw(", \"present\": ");
    raw_bool(projection.present);
    raw(", \"source_chain_complete\": ");
    raw_bool(projection.source_chain_complete);
    raw(", \"authority_decision_present\": ");
    raw_bool(projection.authority_decision_present);
    raw(", \"loader_runtime_contract_present\": ");
    raw_bool(projection.loader_runtime_contract_present);
    raw(", \"loader_runtime_source_evidence_complete\": ");
    raw_bool(projection.loader_runtime_source_evidence_complete);
    raw(", \"service_slot_binding_source_evidence_present\": ");
    raw_bool(projection.service_slot_binding_source_evidence_present);
    raw(", \"service_slot_binding_fact_present\": ");
    raw_bool(projection.service_slot_binding_fact_present);
    raw(", \"audit_rollback_write_boundary_source_evidence_present\": ");
    raw_bool(projection.audit_rollback_write_boundary_source_evidence_present);
    raw(", \"audit_rollback_write_boundary_fact_present\": ");
    raw_bool(projection.audit_rollback_write_boundary_fact_present);
    raw(", \"retained_service_slot_reservation_present\": ");
    raw_bool(projection.retained_service_slot_reservation_present);
    raw(concat!(
        ", \"requested_capability\": \"cap.module.load_ephemeral\", \"load_mode\": \"ram_only\", \"target\": \"live_service_graph\", \"accepts_loader_descriptor\": false",
        ", \"accepts_artifact_bytes\": false, \"authorizes_execution\": false, \"mutates_service_registry\": false, \"writes_durable_audit_state\": false, \"installs_rollback_state\": false",
        ", \"authorizes_load\": false, \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"loads_artifact\": false",
        ", \"load_attempted\": false}",
        ));
}

fn emit_module_load_gate_loader_descriptor_intake_boundary_compact() {
    let projection = module_load_gate_loader_descriptor_intake_boundary_projection();
    raw("{\"schema\": ");
    json_str(MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SCHEMA);
    raw(", \"id\": ");
    json_str(MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_ID);
    raw(", \"source_evidence_event_id\": ");
    json_event_id_option(projection.source_evidence_event_id);
    raw(", \"status\": ");
    json_str(projection.status);
    raw(", \"reason\": ");
    json_str(projection.reason);
    raw(", \"present\": ");
    raw_bool(projection.present);
    raw(", \"source_chain_complete\": ");
    raw_bool(projection.source_chain_complete);
    raw(", \"registry_write_commit_gate_present\": ");
    raw_bool(projection.registry_write_commit_gate_present);
    raw(", \"execution_commit_gate_present\": ");
    raw_bool(projection.execution_commit_gate_present);
    raw(", \"loader_runtime_source_evidence_complete\": ");
    raw_bool(projection.loader_runtime_source_evidence_complete);
    raw(", \"retained_module_evidence_present\": ");
    raw_bool(projection.retained_module_evidence_present);
    raw(", \"retained_service_slot_reservation_present\": ");
    raw_bool(projection.retained_service_slot_reservation_present);
    raw(concat!(
        ", \"requested_capability\": \"cap.module.load_ephemeral\", \"load_mode\": \"ram_only\", \"target\": \"live_service_graph\", \"accepts_loader_descriptor\": false",
        ", \"accepts_descriptor_bytes\": false, \"produces_parsed_descriptor\": false, \"parses_descriptor_bytes\": false, \"accepts_artifact_bytes\": false",
        ", \"authorizes_descriptor_intake\": false, \"authorizes_execution\": false, \"mutates_service_registry\": false, \"writes_durable_audit_state\": false",
        ", \"installs_rollback_state\": false, \"authorizes_load\": false, \"allocates_service_slot\": false, \"creates_service_inventory_records\": false",
        ", \"service_inventory_change\": \"none\", \"loads_artifact\": false, \"load_attempted\": false}",
        ));
}

fn emit_module_load_gate_loader_artifact_byte_intake_boundary_compact() {
    let projection = module_load_gate_loader_artifact_byte_intake_boundary_projection();
    raw("{\"schema\": ");
    json_str(MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SCHEMA);
    raw(", \"id\": ");
    json_str(MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_ID);
    raw(", \"source_evidence_event_id\": ");
    json_event_id_option(projection.source_evidence_event_id);
    raw(", \"status\": ");
    json_str(projection.status);
    raw(", \"reason\": ");
    json_str(projection.reason);
    raw(", \"present\": ");
    raw_bool(projection.present);
    raw(", \"source_chain_complete\": ");
    raw_bool(projection.source_chain_complete);
    raw(", \"descriptor_intake_boundary_present\": ");
    raw_bool(projection.descriptor_intake_boundary_present);
    raw(", \"descriptor_intake_boundary_source_chain_complete\": ");
    raw_bool(projection.descriptor_intake_boundary_source_chain_complete);
    raw(", \"execution_commit_gate_present\": ");
    raw_bool(projection.execution_commit_gate_present);
    raw(", \"artifact_hash_binding_present\": ");
    raw_bool(projection.artifact_hash_binding_present);
    raw(", \"retained_artifact_reference_present\": ");
    raw_bool(projection.retained_artifact_reference_present);
    raw(", \"retained_module_evidence_present\": ");
    raw_bool(projection.retained_module_evidence_present);
    raw(", \"retained_service_slot_reservation_present\": ");
    raw_bool(projection.retained_service_slot_reservation_present);
    raw(concat!(
        ", \"requested_capability\": \"cap.module.load_ephemeral\", \"load_mode\": \"ram_only\", \"target\": \"live_service_graph\", \"accepts_loader_descriptor\": false",
        ", \"accepts_descriptor_bytes\": false, \"produces_parsed_descriptor\": false, \"parses_descriptor_bytes\": false, \"accepts_artifact_bytes\": false",
        ", \"authorizes_descriptor_intake\": false, \"authorizes_artifact_byte_intake\": false, \"authorizes_execution\": false, \"mutates_service_registry\": false",
        ", \"writes_durable_audit_state\": false, \"installs_rollback_state\": false, \"authorizes_load\": false, \"allocates_service_slot\": false",
        ", \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"loads_artifact\": false, \"load_attempted\": false}",
        ));
}

fn emit_module_load_gate_loader_execution_authorization_boundary_compact() {
    let projection = module_load_gate_loader_execution_authorization_boundary_projection();
    raw("{\"schema\": ");
    json_str(MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SCHEMA);
    raw(", \"id\": ");
    json_str(MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_ID);
    raw(", \"source_evidence_event_id\": ");
    json_event_id_option(projection.source_evidence_event_id);
    raw(", \"status\": ");
    json_str(projection.status);
    raw(", \"reason\": ");
    json_str(projection.reason);
    raw(", \"present\": ");
    raw_bool(projection.present);
    raw(", \"source_chain_complete\": ");
    raw_bool(projection.source_chain_complete);
    raw(", \"artifact_byte_intake_boundary_present\": ");
    raw_bool(projection.artifact_byte_intake_boundary_present);
    raw(", \"artifact_byte_intake_boundary_source_chain_complete\": ");
    raw_bool(projection.artifact_byte_intake_boundary_source_chain_complete);
    raw(", \"descriptor_intake_boundary_present\": ");
    raw_bool(projection.descriptor_intake_boundary_present);
    raw(", \"descriptor_intake_boundary_source_chain_complete\": ");
    raw_bool(projection.descriptor_intake_boundary_source_chain_complete);
    raw(", \"execution_commit_gate_present\": ");
    raw_bool(projection.execution_commit_gate_present);
    raw(", \"entrypoint_abi_source_evidence_present\": ");
    raw_bool(projection.entrypoint_abi_source_evidence_present);
    raw(", \"address_space_source_evidence_present\": ");
    raw_bool(projection.address_space_source_evidence_present);
    raw(", \"memory_map_source_evidence_present\": ");
    raw_bool(projection.memory_map_source_evidence_present);
    raw(", \"audit_rollback_write_boundary_source_evidence_present\": ");
    raw_bool(projection.audit_rollback_write_boundary_source_evidence_present);
    raw(", \"retained_module_evidence_present\": ");
    raw_bool(projection.retained_module_evidence_present);
    raw(", \"retained_service_slot_reservation_present\": ");
    raw_bool(projection.retained_service_slot_reservation_present);
    raw(concat!(
        ", \"requested_capability\": \"cap.module.load_ephemeral\", \"load_mode\": \"ram_only\", \"target\": \"live_service_graph\", \"accepts_loader_descriptor\": false",
        ", \"accepts_descriptor_bytes\": false, \"produces_parsed_descriptor\": false, \"parses_descriptor_bytes\": false, \"accepts_artifact_bytes\": false",
        ", \"authorizes_descriptor_intake\": false, \"authorizes_artifact_byte_intake\": false, \"maps_executable_pages\": false, \"jumps_to_entrypoint\": false",
        ", \"authorizes_execution\": false, \"mutates_service_registry\": false, \"writes_durable_audit_state\": false, \"installs_rollback_state\": false, \"authorizes_load\": false",
        ", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"loads_artifact\": false, \"load_attempted\": false}",
        ));
}

fn emit_module_load_gate_loader_service_registry_mutation_boundary_compact() {
    let projection = module_load_gate_loader_service_registry_mutation_boundary_projection();
    raw("{\"schema\": ");
    json_str(MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SCHEMA);
    raw(", \"id\": ");
    json_str(MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_ID);
    raw(", \"source_evidence_event_id\": ");
    json_event_id_option(projection.source_evidence_event_id);
    raw(", \"status\": ");
    json_str(projection.status);
    raw(", \"reason\": ");
    json_str(projection.reason);
    raw(", \"present\": ");
    raw_bool(projection.present);
    raw(", \"source_chain_complete\": ");
    raw_bool(projection.source_chain_complete);
    raw(", \"execution_authorization_boundary_present\": ");
    raw_bool(projection.execution_authorization_boundary_present);
    raw(", \"execution_authorization_boundary_source_chain_complete\": ");
    raw_bool(projection.execution_authorization_boundary_source_chain_complete);
    raw(", \"registry_write_commit_gate_present\": ");
    raw_bool(projection.registry_write_commit_gate_present);
    raw(", \"service_slot_binding_source_evidence_present\": ");
    raw_bool(projection.service_slot_binding_source_evidence_present);
    raw(", \"retained_module_evidence_present\": ");
    raw_bool(projection.retained_module_evidence_present);
    raw(", \"retained_service_slot_reservation_present\": ");
    raw_bool(projection.retained_service_slot_reservation_present);
    raw(concat!(
        ", \"requested_capability\": \"cap.module.load_ephemeral\", \"load_mode\": \"ram_only\", \"target\": \"live_service_graph\", \"accepts_loader_descriptor\": false",
        ", \"accepts_descriptor_bytes\": false, \"produces_parsed_descriptor\": false, \"parses_descriptor_bytes\": false, \"accepts_artifact_bytes\": false",
        ", \"authorizes_descriptor_intake\": false, \"authorizes_artifact_byte_intake\": false, \"maps_executable_pages\": false, \"jumps_to_entrypoint\": false",
        ", \"authorizes_execution\": false, \"mutates_service_registry\": false, \"writes_durable_audit_state\": false, \"installs_rollback_state\": false, \"authorizes_load\": false",
        ", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"loads_artifact\": false, \"load_attempted\": false}",
        ));
}

fn emit_module_load_gate_loader_live_load_boundary_compact(
    boundary_schema: &'static str,
    boundary_id: &'static str,
    projection: ModuleLoadGateLoaderLiveLoadBoundaryProjection,
) {
    raw("{\"schema\": ");
    json_str(boundary_schema);
    raw(", \"id\": ");
    json_str(boundary_id);
    raw(", \"source_evidence_event_id\": ");
    json_event_id_option(projection.source_evidence_event_id);
    raw(", \"status\": ");
    json_str(projection.status);
    raw(", \"reason\": ");
    json_str(projection.reason);
    raw(", \"present\": ");
    raw_bool(projection.present);
    raw(", \"source_chain_complete\": ");
    raw_bool(projection.source_chain_complete);
    raw(", \"load_attempt_boundary_present\": ");
    raw_bool(projection.load_attempt_boundary_present);
    raw(", \"artifact_load_boundary_present\": ");
    raw_bool(projection.artifact_load_boundary_present);
    raw(", \"executable_mapping_boundary_present\": ");
    raw_bool(projection.executable_mapping_boundary_present);
    raw(", \"entrypoint_transfer_boundary_present\": ");
    raw_bool(projection.entrypoint_transfer_boundary_present);
    raw(", \"service_start_boundary_present\": ");
    raw_bool(projection.service_start_boundary_present);
    raw(", \"service_health_binding_boundary_present\": ");
    raw_bool(projection.service_health_binding_boundary_present);
    raw(", \"service_running_state_boundary_present\": ");
    raw_bool(projection.service_running_state_boundary_present);
    raw(", \"service_start_audit_boundary_present\": ");
    raw_bool(projection.service_start_audit_boundary_present);
    raw(", \"service_start_audit_boundary_source_chain_complete\": ");
    raw_bool(projection.service_start_audit_boundary_source_chain_complete);
    raw(", \"service_unload_cleanup_boundary_present\": ");
    raw_bool(projection.service_unload_cleanup_boundary_present);
    raw(", \"service_unload_cleanup_boundary_source_chain_complete\": ");
    raw_bool(projection.service_unload_cleanup_boundary_source_chain_complete);
    raw(", \"live_load_commit_boundary_present\": ");
    raw_bool(projection.live_load_commit_boundary_present);
    raw(", \"live_load_commit_boundary_source_chain_complete\": ");
    raw_bool(projection.live_load_commit_boundary_source_chain_complete);
    raw(", \"commit_audit_boundary_present\": ");
    raw_bool(projection.commit_audit_boundary_present);
    raw(", \"commit_audit_boundary_source_chain_complete\": ");
    raw_bool(projection.commit_audit_boundary_source_chain_complete);
    raw(", \"commit_rollback_boundary_present\": ");
    raw_bool(projection.commit_rollback_boundary_present);
    raw(", \"commit_rollback_boundary_source_chain_complete\": ");
    raw_bool(projection.commit_rollback_boundary_source_chain_complete);
    raw(", \"commit_result_boundary_present\": ");
    raw_bool(projection.commit_result_boundary_present);
    raw(", \"commit_result_boundary_source_chain_complete\": ");
    raw_bool(projection.commit_result_boundary_source_chain_complete);
    raw(", \"descriptor_acceptance_authority_boundary_present\": ");
    raw_bool(projection.descriptor_acceptance_authority_boundary_present);
    raw(", \"descriptor_acceptance_authority_boundary_source_chain_complete\": ");
    raw_bool(projection.descriptor_acceptance_authority_boundary_source_chain_complete);
    raw(", \"descriptor_parser_contract_boundary_present\": ");
    raw_bool(projection.descriptor_parser_contract_boundary_present);
    raw(", \"descriptor_parser_contract_boundary_source_chain_complete\": ");
    raw_bool(projection.descriptor_parser_contract_boundary_source_chain_complete);
    raw(", \"descriptor_parser_result_boundary_present\": ");
    raw_bool(projection.descriptor_parser_result_boundary_present);
    raw(", \"descriptor_parser_result_boundary_source_chain_complete\": ");
    raw_bool(projection.descriptor_parser_result_boundary_source_chain_complete);
    raw(", \"descriptor_schema_validation_boundary_present\": ");
    raw_bool(projection.descriptor_schema_validation_boundary_present);
    raw(", \"descriptor_schema_validation_boundary_source_chain_complete\": ");
    raw_bool(projection.descriptor_schema_validation_boundary_source_chain_complete);
    raw(", \"artifact_byte_intake_boundary_present\": ");
    raw_bool(projection.artifact_byte_intake_boundary_present);
    raw(", \"artifact_byte_intake_boundary_source_chain_complete\": ");
    raw_bool(projection.artifact_byte_intake_boundary_source_chain_complete);
    raw(", \"execution_authorization_boundary_present\": ");
    raw_bool(projection.execution_authorization_boundary_present);
    raw(", \"execution_authorization_boundary_source_chain_complete\": ");
    raw_bool(projection.execution_authorization_boundary_source_chain_complete);
    raw(", \"service_registry_mutation_boundary_present\": ");
    raw_bool(projection.service_registry_mutation_boundary_present);
    raw(", \"service_registry_mutation_boundary_source_chain_complete\": ");
    raw_bool(projection.service_registry_mutation_boundary_source_chain_complete);
    raw(", \"descriptor_capability_validation_boundary_present\": ");
    raw_bool(projection.descriptor_capability_validation_boundary_present);
    raw(", \"descriptor_capability_validation_boundary_source_chain_complete\": ");
    raw_bool(projection.descriptor_capability_validation_boundary_source_chain_complete);
    raw(", \"descriptor_load_plan_boundary_present\": ");
    raw_bool(projection.descriptor_load_plan_boundary_present);
    raw(", \"descriptor_load_plan_boundary_source_chain_complete\": ");
    raw_bool(projection.descriptor_load_plan_boundary_source_chain_complete);
    raw(", \"executable_load_plan_authority_boundary_present\": ");
    raw_bool(projection.executable_load_plan_authority_boundary_present);
    raw(", \"executable_load_plan_authority_boundary_source_chain_complete\": ");
    raw_bool(projection.executable_load_plan_authority_boundary_source_chain_complete);
    raw(", \"executable_load_plan_result_boundary_present\": ");
    raw_bool(projection.executable_load_plan_result_boundary_present);
    raw(", \"executable_load_plan_result_boundary_source_chain_complete\": ");
    raw_bool(projection.executable_load_plan_result_boundary_source_chain_complete);
    raw(", \"executable_image_layout_boundary_present\": ");
    raw_bool(projection.executable_image_layout_boundary_present);
    raw(", \"executable_image_layout_boundary_source_chain_complete\": ");
    raw_bool(projection.executable_image_layout_boundary_source_chain_complete);
    raw(", \"executable_page_mapping_plan_boundary_present\": ");
    raw_bool(projection.executable_page_mapping_plan_boundary_present);
    raw(", \"executable_page_mapping_plan_boundary_source_chain_complete\": ");
    raw_bool(projection.executable_page_mapping_plan_boundary_source_chain_complete);
    raw(", \"executable_page_mapping_boundary_present\": ");
    raw_bool(projection.executable_page_mapping_boundary_present);
    raw(", \"executable_page_mapping_boundary_source_chain_complete\": ");
    raw_bool(projection.executable_page_mapping_boundary_source_chain_complete);
    raw(", \"descriptor_executable_page_binding_boundary_present\": ");
    raw_bool(projection.descriptor_executable_page_binding_boundary_present);
    raw(", \"descriptor_executable_page_binding_boundary_source_chain_complete\": ");
    raw_bool(projection.descriptor_executable_page_binding_boundary_source_chain_complete);
    raw(", \"executable_entrypoint_binding_boundary_present\": ");
    raw_bool(projection.executable_entrypoint_binding_boundary_present);
    raw(", \"executable_entrypoint_binding_boundary_source_chain_complete\": ");
    raw_bool(projection.executable_entrypoint_binding_boundary_source_chain_complete);
    raw(", \"executable_entrypoint_transfer_authorization_boundary_present\": ");
    raw_bool(projection.executable_entrypoint_transfer_authorization_boundary_present);
    raw(", \"executable_entrypoint_transfer_authorization_boundary_source_chain_complete\": ");
    raw_bool(
        projection.executable_entrypoint_transfer_authorization_boundary_source_chain_complete,
    );
    raw(", \"executable_entrypoint_transfer_boundary_present\": ");
    raw_bool(projection.executable_entrypoint_transfer_boundary_present);
    raw(", \"executable_entrypoint_transfer_boundary_source_chain_complete\": ");
    raw_bool(projection.executable_entrypoint_transfer_boundary_source_chain_complete);
    raw(", \"executable_entrypoint_handoff_boundary_present\": ");
    raw_bool(projection.executable_entrypoint_handoff_boundary_present);
    raw(", \"executable_entrypoint_handoff_boundary_source_chain_complete\": ");
    raw_bool(projection.executable_entrypoint_handoff_boundary_source_chain_complete);
    raw(", \"service_slot_binding_source_evidence_present\": ");
    raw_bool(projection.service_slot_binding_source_evidence_present);
    raw(", \"health_state_hooks_source_evidence_present\": ");
    raw_bool(projection.health_state_hooks_source_evidence_present);
    raw(", \"artifact_hash_binding_present\": ");
    raw_bool(projection.artifact_hash_binding_present);
    raw(", \"entrypoint_abi_source_evidence_present\": ");
    raw_bool(projection.entrypoint_abi_source_evidence_present);
    raw(", \"address_space_source_evidence_present\": ");
    raw_bool(projection.address_space_source_evidence_present);
    raw(", \"memory_map_source_evidence_present\": ");
    raw_bool(projection.memory_map_source_evidence_present);
    raw(", \"capability_import_table_source_evidence_present\": ");
    raw_bool(projection.capability_import_table_source_evidence_present);
    raw(", \"audit_rollback_write_boundary_source_evidence_present\": ");
    raw_bool(projection.audit_rollback_write_boundary_source_evidence_present);
    raw(", \"retained_module_evidence_present\": ");
    raw_bool(projection.retained_module_evidence_present);
    raw(", \"retained_artifact_reference_present\": ");
    raw_bool(projection.retained_artifact_reference_present);
    raw(", \"retained_service_slot_reservation_present\": ");
    raw_bool(projection.retained_service_slot_reservation_present);
    raw(concat!(
        ", \"requested_capability\": \"cap.module.load_ephemeral\", \"load_mode\": \"ram_only\", \"target\": \"live_service_graph\", \"accepts_loader_descriptor\": false",
        ", \"accepts_descriptor_bytes\": false, \"produces_parsed_descriptor\": false, \"validates_descriptor_schema\": false, \"produces_validated_descriptor\": false",
        ", \"validates_descriptor_capabilities\": false, \"produces_capability_validated_descriptor\": false, \"authorizes_executable_load_plan\": false",
        ", \"produces_executable_load_plan\": false, \"produces_executable_image_layout\": false, \"produces_executable_page_mapping_plan\": false",
        ", \"binds_capability_validated_descriptor_to_executable_pages\": false, \"parses_descriptor_bytes\": false, \"accepts_artifact_bytes\": false",
        ", \"authorizes_descriptor_intake\": false, \"authorizes_artifact_byte_intake\": false, \"maps_executable_pages\": false, \"jumps_to_entrypoint\": false",
        ", \"authorizes_execution\": false, \"mutates_service_registry\": false, \"writes_durable_audit_state\": false, \"installs_rollback_state\": false, \"authorizes_load\": false",
        ", \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"loads_artifact\": false, \"starts_service\": false",
        ", \"marks_service_running\": false, \"creates_service_health_records\": false, \"writes_service_start_audit_record\": false, \"unloads_service\": false",
        ", \"cleans_up_service_slot\": false, \"commits_live_load\": false, \"writes_load_commit_audit_record\": false, \"installs_commit_rollback_record\": false",
        ", \"records_load_result\": false, \"load_attempted\": false}",
        ));
}
fn emit_module_load_gate_loader_runtime_source_fact_map_compact() {
    let mut idx = 0usize;
    while idx < MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
        emit_inline_record_object_fragment(
            module_load_gate_loader_runtime_source_fact_fields(
                MODULE_LOADER_RUNTIME_FACT_SOURCES[idx],
            ),
            0,
        );
        raw(", ");
        idx += 1;
    }
    emit_inline_record_object_fragment(
        module_load_gate_receiver_identity_load_preflight_source_fact_fields(
            agent_protocol::receiver_identity_load_preflight_projection(),
        ),
        0,
    );
}

fn emit_module_load_gate_evidence_hashes_compact(binding: event_log::ModuleLoadGateBinding) {
    if let Some(reference) = binding.retained_reference {
        raw("\"computed_capability_grant_hash\": ");
        json_sha256(reference.computed_grant_hash);
    } else {
        raw("\"computed_capability_grant_hash\": null");
    }
    if let Some(reference) = binding
        .attestation_reference
        .filter(|_| module_load_gate_local_attestation_reference_valid(binding))
    {
        raw(", \"local_attestation_reference_hash\": ");
        json_sha256(reference.attestation_reference_hash);
        raw(", \"local_attestation_hash\": ");
        json_sha256(reference.local_attestation_hash);
    } else {
        raw(", \"local_attestation_reference_hash\": null, \"local_attestation_hash\": null");
    }
    if let Some(reference) = binding
        .approval_reference
        .filter(|_| module_load_gate_local_approval_reference_valid(binding))
    {
        raw(", \"local_approval_reference_hash\": ");
        json_sha256(reference.approval_reference_hash);
        raw(", \"local_approval_hash\": ");
        json_sha256(reference.local_approval_hash);
    } else {
        raw(", \"local_approval_reference_hash\": null, \"local_approval_hash\": null");
    }
    if let Some(reference) = binding
        .vm_report_reference
        .filter(|_| module_load_gate_vm_test_report_reference_valid(binding))
    {
        raw(", \"vm_test_report_reference_hash\": ");
        json_sha256(reference.report_reference_hash);
        raw(", \"vm_test_report_hash\": ");
        json_sha256(reference.vm_report_hash);
    } else {
        raw(", \"vm_test_report_reference_hash\": null, \"vm_test_report_hash\": null");
    }
    if let Some(reference) = binding
        .artifact_reference
        .filter(|_| module_load_gate_candidate_artifact_reference_valid(binding))
    {
        raw(", \"artifact_reference_hash\": ");
        json_sha256(reference.artifact_reference_hash);
        raw(", \"artifact_hash\": ");
        json_sha256(reference.artifact_hash);
    } else {
        raw(", \"artifact_reference_hash\": null, \"artifact_hash\": null");
    }
    if let Some(reference) = binding
        .manifest_reference
        .filter(|_| module_load_gate_manifest_reference_valid(binding))
    {
        raw(", \"manifest_reference_hash\": ");
        json_sha256(reference.manifest_reference_hash);
        raw(", \"manifest_hash\": ");
        json_sha256(reference.manifest_hash);
    } else {
        raw(", \"manifest_reference_hash\": null, \"manifest_hash\": null");
    }
    if let Some(reference) = binding
        .audit_rollback_reference
        .filter(|_| module_load_gate_audit_rollback_reference_valid(binding))
    {
        raw(", \"audit_record_hash\": ");
        json_sha256(reference.audit_record_hash);
        raw(", \"rollback_plan_hash\": ");
        json_sha256(reference.rollback_plan_hash);
        raw(", \"pre_load_service_inventory_hash\": ");
        json_sha256(reference.pre_load_service_inventory_hash);
        raw(", \"cleanup_actions_hash\": ");
        json_sha256(reference.cleanup_actions_hash);
        raw(", \"ram_only_service_slot_id\": ");
        json_str(reference.ram_only_service_slot_id.as_str());
    } else {
        raw(", \"audit_record_hash\": null, \"rollback_plan_hash\": null, \"pre_load_service_inventory_hash\": null, \"cleanup_actions_hash\": null, \"ram_only_service_slot_id\": null");
    }
    if let Some(reservation) = binding
        .service_slot_reservation
        .filter(|_| module_load_gate_service_slot_reservation_valid(binding))
    {
        raw(", \"service_slot_reservation_hash\": ");
        json_sha256(reservation.reservation_hash);
    } else {
        raw(", \"service_slot_reservation_hash\": null");
    }
}

fn emit_module_load_gate_audit_rollback_requirements_compact(
    binding: event_log::ModuleLoadGateBinding,
) {
    emit_inline_record_object_fragment(
        module_load_gate_audit_rollback_requirement_fields(&binding, true),
        0,
    );
}
