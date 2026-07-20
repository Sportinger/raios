use crate::agent_protocol_module_service_slot_allocator_projection::latest_module_service_slot_allocator_readiness_projection;
use crate::agent_protocol_module_types::{
    ModuleLoaderArtifactByteIntakeBoundary, ModuleLoaderDescriptorIntakeBoundary,
    ModuleLoaderExecutionAuthorizationBoundary, ModuleLoaderLiveLoadBoundary,
    ModuleLoaderRuntimeCandidate, ModuleLoaderRuntimeExecutionCommitGate, ModuleLoaderRuntimeFact,
    ModuleLoaderRuntimeFactSource, ModuleLoaderServiceRegistryMutationBoundary,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_REASON,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_STATUS,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_REASON,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_STATUS, MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_REASON,
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_METHOD, MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_STATUS,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_REASON,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_STATUS, MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_REASON,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_STATUS,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_REASON,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_STATUS,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_REASON,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_STATUS, MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_REASON,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_METHOD, MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_STATUS,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_MISSING_STATUS,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_REASON,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_METHOD,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_STATUS, MODULE_LOADER_RUNTIME_FACT_SOURCES,
    MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT, MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_STATUS,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_STATUS,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_STATUS,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_STATUS, MODULE_LOADER_SERVICE_START_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_START_BOUNDARY_STATUS,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_STATUS,
};
use crate::event_log;

pub(super) fn module_loader_runtime_snapshot(
    manifest_reference_present: bool,
    artifact_reference_present: bool,
    vm_report_reference_present: bool,
    local_attestation_reference_present: bool,
    local_approval_reference_present: bool,
    computed_grant_reference_present: bool,
    audit_rollback_reference_present: bool,
    service_slot_reservation_present: bool,
    service_slot_reservation_event_id: Option<event_log::EventId>,
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
    execution_commit_gate_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderRuntimeExecutionCommitGateSourceEvidence,
    )>,
    descriptor_intake_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderDescriptorIntakeBoundarySourceEvidence,
    )>,
    artifact_byte_intake_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderArtifactByteIntakeBoundarySourceEvidence,
    )>,
    execution_authorization_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderExecutionAuthorizationBoundarySourceEvidence,
    )>,
    service_registry_mutation_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderServiceRegistryMutationBoundarySourceEvidence,
    )>,
    load_attempt_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    artifact_load_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    executable_mapping_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    entrypoint_transfer_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    service_start_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    service_health_binding_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    service_running_state_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    service_start_audit_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    service_unload_cleanup_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    live_load_commit_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    commit_audit_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    commit_rollback_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    commit_result_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    descriptor_acceptance_authority_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    descriptor_parser_contract_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    descriptor_parser_result_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    descriptor_schema_validation_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    descriptor_capability_validation_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    descriptor_load_plan_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    executable_load_plan_authority_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    executable_load_plan_result_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    executable_image_layout_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    executable_page_mapping_plan_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    executable_page_mapping_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    descriptor_executable_page_binding_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    executable_entrypoint_binding_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    executable_entrypoint_transfer_authorization_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    executable_entrypoint_transfer_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    executable_entrypoint_handoff_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    executable_entrypoint_invocation_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
) -> ModuleLoaderRuntimeCandidate {
    let service_slot_allocator = latest_module_service_slot_allocator_readiness_projection(
        service_slot_reservation_event_id,
    );
    ModuleLoaderRuntimeCandidate {
        manifest_reference_present,
        artifact_reference_present,
        vm_report_reference_present,
        local_attestation_reference_present,
        local_approval_reference_present,
        computed_grant_reference_present,
        audit_rollback_reference_present,
        service_slot_reservation_present,
        service_slot_allocator_readiness_present: service_slot_allocator.readiness_present,
        service_slot_allocator_ready: service_slot_allocator.ready,
        service_slot_allocator_unready_status: service_slot_allocator.unready_status,
        service_slot_allocator_unready_reason: service_slot_allocator.unready_reason,
        loader_identity: module_loader_runtime_loader_identity_fact(
            loader_identity_source_evidence,
        ),
        artifact_hash_binding: module_loader_runtime_artifact_hash_binding_fact(
            artifact_hash_binding_source_evidence,
        ),
        entrypoint_abi: module_loader_runtime_loader_fact_source_fact(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[2],
            entrypoint_abi_source_evidence,
        ),
        address_space_boundary: module_loader_runtime_loader_fact_source_fact(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[3],
            address_space_source_evidence,
        ),
        memory_map_constraints: module_loader_runtime_loader_fact_source_fact(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[4],
            memory_map_source_evidence,
        ),
        capability_import_table: module_loader_runtime_loader_fact_source_fact(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[5],
            capability_table_source_evidence,
        ),
        service_slot_binding: module_loader_runtime_loader_fact_source_fact(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[6],
            service_slot_source_evidence,
        ),
        health_state_hooks: module_loader_runtime_loader_fact_source_fact(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[7],
            health_source_evidence,
        ),
        rollback_hooks: module_loader_runtime_loader_fact_source_fact(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[8],
            rollback_source_evidence,
        ),
        audit_rollback_write_boundary_binding: module_loader_runtime_loader_fact_source_fact(
            MODULE_LOADER_RUNTIME_FACT_SOURCES[9],
            write_boundary_source_evidence,
        ),
        execution_commit_gate: module_loader_runtime_execution_commit_gate_from_source_evidence(
            execution_commit_gate_source_evidence,
        ),
        descriptor_intake_boundary: module_loader_descriptor_intake_boundary_from_source_evidence(
            descriptor_intake_boundary_source_evidence,
        ),
        artifact_byte_intake_boundary:
            module_loader_artifact_byte_intake_boundary_from_source_evidence(
                artifact_byte_intake_boundary_source_evidence,
            ),
        execution_authorization_boundary:
            module_loader_execution_authorization_boundary_from_source_evidence(
                execution_authorization_boundary_source_evidence,
            ),
        service_registry_mutation_boundary:
            module_loader_service_registry_mutation_boundary_from_source_evidence(
                service_registry_mutation_boundary_source_evidence,
            ),
        load_attempt_boundary: module_loader_live_load_boundary_from_source_evidence(
            load_attempt_boundary_source_evidence,
            MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        artifact_load_boundary: module_loader_live_load_boundary_from_source_evidence(
            artifact_load_boundary_source_evidence,
            MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_mapping_boundary: module_loader_live_load_boundary_from_source_evidence(
            executable_mapping_boundary_source_evidence,
            MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        entrypoint_transfer_boundary: module_loader_live_load_boundary_from_source_evidence(
            entrypoint_transfer_boundary_source_evidence,
            MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        service_start_boundary: module_loader_live_load_boundary_from_source_evidence(
            service_start_boundary_source_evidence,
            MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        service_health_binding_boundary: module_loader_live_load_boundary_from_source_evidence(
            service_health_binding_boundary_source_evidence,
            MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        service_running_state_boundary: module_loader_live_load_boundary_from_source_evidence(
            service_running_state_boundary_source_evidence,
            MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        service_start_audit_boundary: module_loader_live_load_boundary_from_source_evidence(
            service_start_audit_boundary_source_evidence,
            MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        service_unload_cleanup_boundary: module_loader_live_load_boundary_from_source_evidence(
            service_unload_cleanup_boundary_source_evidence,
            MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        live_load_commit_boundary: module_loader_live_load_boundary_from_source_evidence(
            live_load_commit_boundary_source_evidence,
            MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        commit_audit_boundary: module_loader_live_load_boundary_from_source_evidence(
            commit_audit_boundary_source_evidence,
            MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        commit_rollback_boundary: module_loader_live_load_boundary_from_source_evidence(
            commit_rollback_boundary_source_evidence,
            MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        commit_result_boundary: module_loader_live_load_boundary_from_source_evidence(
            commit_result_boundary_source_evidence,
            MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        descriptor_acceptance_authority_boundary:
            module_loader_live_load_boundary_from_source_evidence(
                descriptor_acceptance_authority_boundary_source_evidence,
                MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_METHOD,
                MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR,
            ),
        descriptor_parser_contract_boundary: module_loader_live_load_boundary_from_source_evidence(
            descriptor_parser_contract_boundary_source_evidence,
            MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        descriptor_parser_result_boundary: module_loader_live_load_boundary_from_source_evidence(
            descriptor_parser_result_boundary_source_evidence,
            MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        descriptor_schema_validation_boundary:
            module_loader_live_load_boundary_from_source_evidence(
                descriptor_schema_validation_boundary_source_evidence,
                MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_METHOD,
                MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR,
            ),
        descriptor_capability_validation_boundary:
            module_loader_live_load_boundary_from_source_evidence(
                descriptor_capability_validation_boundary_source_evidence,
                MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_METHOD,
                MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR,
            ),
        descriptor_load_plan_boundary: module_loader_live_load_boundary_from_source_evidence(
            descriptor_load_plan_boundary_source_evidence,
            MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_load_plan_authority_boundary:
            module_loader_live_load_boundary_from_source_evidence(
                executable_load_plan_authority_boundary_source_evidence,
                MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_METHOD,
                MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR,
            ),
        executable_load_plan_result_boundary: module_loader_live_load_boundary_from_source_evidence(
            executable_load_plan_result_boundary_source_evidence,
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_image_layout_boundary: module_loader_live_load_boundary_from_source_evidence(
            executable_image_layout_boundary_source_evidence,
            MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_page_mapping_plan_boundary:
            module_loader_live_load_boundary_from_source_evidence(
                executable_page_mapping_plan_boundary_source_evidence,
                MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_METHOD,
                MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR,
            ),
        executable_page_mapping_boundary: module_loader_live_load_boundary_from_source_evidence(
            executable_page_mapping_boundary_source_evidence,
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        descriptor_executable_page_binding_boundary:
            module_loader_live_load_boundary_from_source_evidence(
                descriptor_executable_page_binding_boundary_source_evidence,
                MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_METHOD,
                MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
            ),
        executable_entrypoint_binding_boundary:
            module_loader_live_load_boundary_from_source_evidence(
                executable_entrypoint_binding_boundary_source_evidence,
                MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_METHOD,
                MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
            ),
        executable_entrypoint_transfer_authorization_boundary:
            module_loader_live_load_boundary_from_source_evidence(
                executable_entrypoint_transfer_authorization_boundary_source_evidence,
                MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_METHOD,
                MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_FACT_LOCATOR,
            ),
        executable_entrypoint_transfer_boundary: module_loader_live_load_boundary_from_source_evidence(
            executable_entrypoint_transfer_boundary_source_evidence,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_entrypoint_handoff_boundary: module_loader_live_load_boundary_from_source_evidence(
            executable_entrypoint_handoff_boundary_source_evidence,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_entrypoint_invocation_boundary: module_loader_live_load_boundary_from_source_evidence(
            executable_entrypoint_invocation_boundary_source_evidence,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
    }
}

pub(super) fn module_loader_runtime_ready_snapshot() -> ModuleLoaderRuntimeCandidate {
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
        service_slot_allocator_unready_status: "available",
        service_slot_allocator_unready_reason: "service_slot_allocator_runtime_available",
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
        execution_commit_gate: module_loader_runtime_execution_commit_gate_available(),
        descriptor_intake_boundary: module_loader_descriptor_intake_boundary_available(),
        artifact_byte_intake_boundary: module_loader_artifact_byte_intake_boundary_available(),
        execution_authorization_boundary: module_loader_execution_authorization_boundary_available(
        ),
        service_registry_mutation_boundary:
            module_loader_service_registry_mutation_boundary_available(),
        load_attempt_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_STATUS,
            MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_REASON,
            MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        artifact_load_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_STATUS,
            MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_REASON,
            MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_mapping_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_STATUS,
            MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_REASON,
            MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        entrypoint_transfer_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_STATUS,
            MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_REASON,
            MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        service_start_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_SERVICE_START_BOUNDARY_STATUS,
            MODULE_LOADER_SERVICE_START_BOUNDARY_REASON,
            MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        service_health_binding_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_STATUS,
            MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_REASON,
            MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        service_running_state_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_STATUS,
            MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_REASON,
            MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        service_start_audit_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_STATUS,
            MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_REASON,
            MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        service_unload_cleanup_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_STATUS,
            MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_REASON,
            MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        live_load_commit_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_STATUS,
            MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_REASON,
            MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        commit_audit_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_STATUS,
            MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_REASON,
            MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        commit_rollback_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_STATUS,
            MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_REASON,
            MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        commit_result_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_COMMIT_RESULT_BOUNDARY_STATUS,
            MODULE_LOADER_COMMIT_RESULT_BOUNDARY_REASON,
            MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        descriptor_acceptance_authority_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_STATUS,
            MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_REASON,
            MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        descriptor_parser_contract_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_STATUS,
            MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_REASON,
            MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        descriptor_parser_result_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_STATUS,
            MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_REASON,
            MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        descriptor_schema_validation_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_STATUS,
            MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_REASON,
            MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        descriptor_capability_validation_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_STATUS,
            MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_REASON,
            MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        descriptor_load_plan_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_STATUS,
            MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_REASON,
            MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_load_plan_authority_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_STATUS,
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_REASON,
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_load_plan_result_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_STATUS,
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_REASON,
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_image_layout_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_STATUS,
            MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_REASON,
            MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_page_mapping_plan_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_STATUS,
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_REASON,
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_page_mapping_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_STATUS,
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_REASON,
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        descriptor_executable_page_binding_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_STATUS,
            MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_REASON,
            MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_entrypoint_binding_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_STATUS,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_REASON,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_entrypoint_transfer_authorization_boundary:
            module_loader_live_load_boundary_available(
                MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_STATUS,
                MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_REASON,
                MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_METHOD,
                MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_FACT_LOCATOR,
            ),
        executable_entrypoint_transfer_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_STATUS,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_REASON,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_entrypoint_handoff_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_STATUS,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_REASON,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_FACT_LOCATOR,
        ),
        executable_entrypoint_invocation_boundary: module_loader_live_load_boundary_available(
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_STATUS,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_REASON,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_METHOD,
            MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_FACT_LOCATOR,
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

fn module_loader_runtime_loader_fact_source_fact(
    source: ModuleLoaderRuntimeFactSource,
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
) -> ModuleLoaderRuntimeFact {
    let Some((event_id, evidence)) = source_evidence else {
        return module_loader_runtime_missing_fact_for(source);
    };

    ModuleLoaderRuntimeFact {
        present: evidence.fact_present,
        schema_ok: evidence.fact_schema_ok,
        scope: evidence.fact_scope,
        provenance_ok: evidence.fact_provenance_ok,
        classification: evidence.fact_classification,
        binds_retained_module_evidence: evidence.binds_retained_module_evidence,
        binds_service_slot_allocator: evidence.binds_service_slot_allocator,
        binds_audit_rollback_write_boundary: evidence.binds_audit_rollback_write_boundary,
        source_evidence_event_id: Some(event_id),
        source_evidence_schema: evidence.schema,
        source_evidence_state: if evidence.fact_present {
            "observed_current_boot_present"
        } else {
            "observed_current_boot_missing"
        },
        source_evidence_status: evidence.fact_status,
        source_evidence_reason: evidence.fact_reason,
        source_evidence_method: evidence.source_method,
        source_evidence_fact_locator: evidence.source_fact_locator,
    }
}

pub(super) fn module_loader_runtime_observed_loader_identity_missing_fact(
) -> ModuleLoaderRuntimeFact {
    ModuleLoaderRuntimeFact {
        source_evidence_event_id: Some(event_log::EventId { sequence: 42 }),
        source_evidence_state: "observed_current_boot_missing",
        source_evidence_status: "missing",
        source_evidence_reason: "module_loader_identity_missing",
        ..module_loader_runtime_missing_fact_for(MODULE_LOADER_RUNTIME_FACT_SOURCES[0])
    }
}

pub(super) fn module_loader_runtime_observed_artifact_hash_binding_missing_fact(
) -> ModuleLoaderRuntimeFact {
    ModuleLoaderRuntimeFact {
        source_evidence_event_id: Some(event_log::EventId { sequence: 43 }),
        source_evidence_state: "observed_current_boot_missing",
        source_evidence_status: "missing",
        source_evidence_reason: "module_loader_artifact_hash_binding_missing",
        ..module_loader_runtime_missing_fact_for(MODULE_LOADER_RUNTIME_FACT_SOURCES[1])
    }
}

pub(super) fn module_loader_runtime_observed_entrypoint_abi_missing_fact() -> ModuleLoaderRuntimeFact
{
    module_loader_runtime_observed_loader_fact_missing_fact(
        MODULE_LOADER_RUNTIME_FACT_SOURCES[2],
        44,
    )
}

pub(super) fn module_loader_runtime_observed_loader_fact_missing_fact(
    source: ModuleLoaderRuntimeFactSource,
    sequence: u64,
) -> ModuleLoaderRuntimeFact {
    ModuleLoaderRuntimeFact {
        source_evidence_event_id: Some(event_log::EventId { sequence }),
        source_evidence_state: "observed_current_boot_missing",
        source_evidence_status: "missing",
        source_evidence_reason: source.missing_reason,
        ..module_loader_runtime_missing_fact_for(source)
    }
}

pub(super) fn module_loader_runtime_missing_fact_for(
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

fn module_loader_runtime_execution_commit_gate_from_source_evidence(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderRuntimeExecutionCommitGateSourceEvidence,
    )>,
) -> ModuleLoaderRuntimeExecutionCommitGate {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleLoaderRuntimeExecutionCommitGate {
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
            loader_runtime_source_evidence_event_ids: evidence
                .loader_runtime_source_evidence_event_ids,
            loader_runtime_source_evidence_present: evidence.loader_runtime_source_evidence_present,
            loader_runtime_fact_present: evidence.loader_runtime_fact_present,
        };
    }
    module_loader_runtime_execution_commit_gate_missing()
}

pub(super) fn module_loader_runtime_execution_commit_gate_missing(
) -> ModuleLoaderRuntimeExecutionCommitGate {
    ModuleLoaderRuntimeExecutionCommitGate {
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema: MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_MISSING_STATUS,
        source_evidence_reason:
            MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_EVIDENCE_MISSING_REASON,
        source_evidence_method: MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_FACT_LOCATOR,
        source_chain_complete: false,
        authority_decision_present: false,
        loader_runtime_contract_present: false,
        loader_runtime_source_evidence_complete: false,
        service_slot_binding_source_evidence_present: false,
        service_slot_binding_fact_present: false,
        audit_rollback_write_boundary_source_evidence_present: false,
        audit_rollback_write_boundary_fact_present: false,
        retained_service_slot_reservation_present: false,
        loader_runtime_source_evidence_event_ids: [None; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_source_evidence_present: [false; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_fact_present: [false; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    }
}

fn module_loader_runtime_execution_commit_gate_available() -> ModuleLoaderRuntimeExecutionCommitGate
{
    ModuleLoaderRuntimeExecutionCommitGate {
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema: MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_STATUS,
        source_evidence_reason: MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_REASON,
        source_evidence_method: MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_FACT_LOCATOR,
        source_chain_complete: true,
        authority_decision_present: true,
        loader_runtime_contract_present: true,
        loader_runtime_source_evidence_complete: true,
        service_slot_binding_source_evidence_present: true,
        service_slot_binding_fact_present: true,
        audit_rollback_write_boundary_source_evidence_present: true,
        audit_rollback_write_boundary_fact_present: true,
        retained_service_slot_reservation_present: true,
        loader_runtime_source_evidence_event_ids: [None; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_source_evidence_present: [true; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_fact_present: [true; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    }
}

fn module_loader_descriptor_intake_boundary_from_source_evidence(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderDescriptorIntakeBoundarySourceEvidence,
    )>,
) -> ModuleLoaderDescriptorIntakeBoundary {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleLoaderDescriptorIntakeBoundary {
            present: evidence.boundary_present,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.boundary_present {
                "observed_current_boot_defined"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.boundary_status,
            source_evidence_reason: evidence.boundary_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
            source_chain_complete: evidence.source_chain_complete,
            registry_write_commit_gate_present: evidence.registry_write_commit_gate_present,
            execution_commit_gate_present: evidence.execution_commit_gate_present,
            loader_runtime_source_evidence_complete: evidence
                .loader_runtime_source_evidence_complete,
            retained_module_evidence_present: evidence.retained_module_evidence_present,
            retained_service_slot_reservation_present: evidence
                .retained_service_slot_reservation_present,
            loader_runtime_source_evidence_event_ids: evidence
                .loader_runtime_source_evidence_event_ids,
            loader_runtime_source_evidence_present: evidence.loader_runtime_source_evidence_present,
            loader_runtime_fact_present: evidence.loader_runtime_fact_present,
        };
    }
    module_loader_descriptor_intake_boundary_missing()
}

pub(super) fn module_loader_descriptor_intake_boundary_missing(
) -> ModuleLoaderDescriptorIntakeBoundary {
    ModuleLoaderDescriptorIntakeBoundary {
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_MISSING_STATUS,
        source_evidence_reason:
            MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
        source_evidence_method: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_METHOD,
        source_evidence_fact_locator: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_FACT_LOCATOR,
        source_chain_complete: false,
        registry_write_commit_gate_present: false,
        execution_commit_gate_present: false,
        loader_runtime_source_evidence_complete: false,
        retained_module_evidence_present: false,
        retained_service_slot_reservation_present: false,
        loader_runtime_source_evidence_event_ids: [None; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_source_evidence_present: [false; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_fact_present: [false; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    }
}

fn module_loader_descriptor_intake_boundary_available() -> ModuleLoaderDescriptorIntakeBoundary {
    ModuleLoaderDescriptorIntakeBoundary {
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_STATUS,
        source_evidence_reason: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_REASON,
        source_evidence_method: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_METHOD,
        source_evidence_fact_locator: MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_FACT_LOCATOR,
        source_chain_complete: true,
        registry_write_commit_gate_present: true,
        execution_commit_gate_present: true,
        loader_runtime_source_evidence_complete: true,
        retained_module_evidence_present: true,
        retained_service_slot_reservation_present: true,
        loader_runtime_source_evidence_event_ids: [None; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_source_evidence_present: [true; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_fact_present: [true; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    }
}

fn module_loader_artifact_byte_intake_boundary_from_source_evidence(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderArtifactByteIntakeBoundarySourceEvidence,
    )>,
) -> ModuleLoaderArtifactByteIntakeBoundary {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleLoaderArtifactByteIntakeBoundary {
            present: evidence.boundary_present,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.boundary_present {
                "observed_current_boot_defined"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.boundary_status,
            source_evidence_reason: evidence.boundary_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
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
            loader_runtime_source_evidence_event_ids: evidence
                .loader_runtime_source_evidence_event_ids,
            loader_runtime_source_evidence_present: evidence.loader_runtime_source_evidence_present,
            loader_runtime_fact_present: evidence.loader_runtime_fact_present,
        };
    }
    module_loader_artifact_byte_intake_boundary_missing()
}

pub(super) fn module_loader_artifact_byte_intake_boundary_missing(
) -> ModuleLoaderArtifactByteIntakeBoundary {
    ModuleLoaderArtifactByteIntakeBoundary {
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema: MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_MISSING_STATUS,
        source_evidence_reason:
            MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
        source_evidence_method: MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_FACT_LOCATOR,
        source_chain_complete: false,
        descriptor_intake_boundary_present: false,
        descriptor_intake_boundary_source_chain_complete: false,
        execution_commit_gate_present: false,
        artifact_hash_binding_present: false,
        retained_artifact_reference_present: false,
        retained_module_evidence_present: false,
        retained_service_slot_reservation_present: false,
        loader_runtime_source_evidence_event_ids: [None; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_source_evidence_present: [false; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_fact_present: [false; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    }
}

fn module_loader_artifact_byte_intake_boundary_available() -> ModuleLoaderArtifactByteIntakeBoundary
{
    ModuleLoaderArtifactByteIntakeBoundary {
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema: MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_STATUS,
        source_evidence_reason: MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_REASON,
        source_evidence_method: MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_FACT_LOCATOR,
        source_chain_complete: true,
        descriptor_intake_boundary_present: true,
        descriptor_intake_boundary_source_chain_complete: true,
        execution_commit_gate_present: true,
        artifact_hash_binding_present: true,
        retained_artifact_reference_present: true,
        retained_module_evidence_present: true,
        retained_service_slot_reservation_present: true,
        loader_runtime_source_evidence_event_ids: [None; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_source_evidence_present: [true; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_fact_present: [true; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    }
}

fn module_loader_execution_authorization_boundary_from_source_evidence(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderExecutionAuthorizationBoundarySourceEvidence,
    )>,
) -> ModuleLoaderExecutionAuthorizationBoundary {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleLoaderExecutionAuthorizationBoundary {
            present: evidence.boundary_present,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.boundary_present {
                "observed_current_boot_defined"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.boundary_status,
            source_evidence_reason: evidence.boundary_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
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
            loader_runtime_source_evidence_event_ids: evidence
                .loader_runtime_source_evidence_event_ids,
            loader_runtime_source_evidence_present: evidence.loader_runtime_source_evidence_present,
            loader_runtime_fact_present: evidence.loader_runtime_fact_present,
        };
    }
    module_loader_execution_authorization_boundary_missing()
}

pub(super) fn module_loader_execution_authorization_boundary_missing(
) -> ModuleLoaderExecutionAuthorizationBoundary {
    ModuleLoaderExecutionAuthorizationBoundary {
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema:
            MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_MISSING_STATUS,
        source_evidence_reason:
            MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
        source_evidence_method: MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_FACT_LOCATOR,
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
        loader_runtime_source_evidence_event_ids: [None; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_source_evidence_present: [false; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_fact_present: [false; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    }
}

fn module_loader_execution_authorization_boundary_available(
) -> ModuleLoaderExecutionAuthorizationBoundary {
    ModuleLoaderExecutionAuthorizationBoundary {
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema:
            MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_STATUS,
        source_evidence_reason: MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_REASON,
        source_evidence_method: MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_FACT_LOCATOR,
        source_chain_complete: true,
        artifact_byte_intake_boundary_present: true,
        artifact_byte_intake_boundary_source_chain_complete: true,
        descriptor_intake_boundary_present: true,
        descriptor_intake_boundary_source_chain_complete: true,
        execution_commit_gate_present: true,
        entrypoint_abi_source_evidence_present: true,
        address_space_source_evidence_present: true,
        memory_map_source_evidence_present: true,
        audit_rollback_write_boundary_source_evidence_present: true,
        retained_module_evidence_present: true,
        retained_service_slot_reservation_present: true,
        loader_runtime_source_evidence_event_ids: [None; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_source_evidence_present: [true; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_fact_present: [true; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    }
}

fn module_loader_service_registry_mutation_boundary_from_source_evidence(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderServiceRegistryMutationBoundarySourceEvidence,
    )>,
) -> ModuleLoaderServiceRegistryMutationBoundary {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleLoaderServiceRegistryMutationBoundary {
            present: evidence.boundary_present,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.boundary_present {
                "observed_current_boot_defined"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.boundary_status,
            source_evidence_reason: evidence.boundary_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
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
            loader_runtime_source_evidence_event_ids: evidence
                .loader_runtime_source_evidence_event_ids,
            loader_runtime_source_evidence_present: evidence.loader_runtime_source_evidence_present,
            loader_runtime_fact_present: evidence.loader_runtime_fact_present,
        };
    }
    module_loader_service_registry_mutation_boundary_missing()
}

pub(super) fn module_loader_service_registry_mutation_boundary_missing(
) -> ModuleLoaderServiceRegistryMutationBoundary {
    ModuleLoaderServiceRegistryMutationBoundary {
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema:
            MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_MISSING_STATUS,
        source_evidence_reason:
            MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_EVIDENCE_MISSING_REASON,
        source_evidence_method: MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_FACT_LOCATOR,
        source_chain_complete: false,
        execution_authorization_boundary_present: false,
        execution_authorization_boundary_source_chain_complete: false,
        registry_write_commit_gate_present: false,
        service_slot_binding_source_evidence_present: false,
        retained_module_evidence_present: false,
        retained_service_slot_reservation_present: false,
        loader_runtime_source_evidence_event_ids: [None; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_source_evidence_present: [false; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_fact_present: [false; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    }
}

fn module_loader_service_registry_mutation_boundary_available(
) -> ModuleLoaderServiceRegistryMutationBoundary {
    ModuleLoaderServiceRegistryMutationBoundary {
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema:
            MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_STATUS,
        source_evidence_reason: MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_REASON,
        source_evidence_method: MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_METHOD,
        source_evidence_fact_locator:
            MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_FACT_LOCATOR,
        source_chain_complete: true,
        execution_authorization_boundary_present: true,
        execution_authorization_boundary_source_chain_complete: true,
        registry_write_commit_gate_present: true,
        service_slot_binding_source_evidence_present: true,
        retained_module_evidence_present: true,
        retained_service_slot_reservation_present: true,
        loader_runtime_source_evidence_event_ids: [None; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_source_evidence_present: [true; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_fact_present: [true; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    }
}

fn module_loader_live_load_boundary_from_source_evidence(
    source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    )>,
    missing_schema: &'static str,
    missing_method: &'static str,
    missing_locator: &'static str,
) -> ModuleLoaderLiveLoadBoundary {
    if let Some((event_id, evidence)) = source_evidence {
        return ModuleLoaderLiveLoadBoundary {
            present: evidence.boundary_present,
            source_evidence_event_id: Some(event_id),
            source_evidence_schema: evidence.schema,
            source_evidence_state: if evidence.boundary_present {
                "observed_current_boot_defined"
            } else {
                "observed_current_boot_missing"
            },
            source_evidence_status: evidence.boundary_status,
            source_evidence_reason: evidence.boundary_reason,
            source_evidence_method: evidence.source_method,
            source_evidence_fact_locator: evidence.source_fact_locator,
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
            loader_runtime_source_evidence_event_ids: evidence
                .loader_runtime_source_evidence_event_ids,
            loader_runtime_source_evidence_present: evidence.loader_runtime_source_evidence_present,
            loader_runtime_fact_present: evidence.loader_runtime_fact_present,
        };
    }
    module_loader_live_load_boundary_missing(missing_schema, missing_method, missing_locator)
}

pub(super) fn module_loader_live_load_boundary_missing(
    source_evidence_schema: &'static str,
    source_method: &'static str,
    source_fact_locator: &'static str,
) -> ModuleLoaderLiveLoadBoundary {
    ModuleLoaderLiveLoadBoundary {
        present: false,
        source_evidence_event_id: None,
        source_evidence_schema,
        source_evidence_state: "addressable_not_observed",
        source_evidence_status: "missing",
        source_evidence_reason: "module_loader_live_load_boundary_source_evidence_missing",
        source_evidence_method: source_method,
        source_evidence_fact_locator: source_fact_locator,
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
        loader_runtime_source_evidence_event_ids: [None; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_source_evidence_present: [false; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_fact_present: [false; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    }
}

fn module_loader_live_load_boundary_available(
    source_evidence_schema: &'static str,
    status: &'static str,
    reason: &'static str,
    source_method: &'static str,
    source_fact_locator: &'static str,
) -> ModuleLoaderLiveLoadBoundary {
    ModuleLoaderLiveLoadBoundary {
        present: true,
        source_evidence_event_id: None,
        source_evidence_schema,
        source_evidence_state: "test_fixture_not_retained",
        source_evidence_status: status,
        source_evidence_reason: reason,
        source_evidence_method: source_method,
        source_evidence_fact_locator: source_fact_locator,
        source_chain_complete: true,
        load_attempt_boundary_present: true,
        load_attempt_boundary_source_chain_complete: true,
        artifact_load_boundary_present: true,
        artifact_load_boundary_source_chain_complete: true,
        executable_mapping_boundary_present: true,
        executable_mapping_boundary_source_chain_complete: true,
        entrypoint_transfer_boundary_present: true,
        entrypoint_transfer_boundary_source_chain_complete: true,
        service_start_boundary_present: true,
        service_start_boundary_source_chain_complete: true,
        service_health_binding_boundary_present: true,
        service_health_binding_boundary_source_chain_complete: true,
        service_running_state_boundary_present: true,
        service_running_state_boundary_source_chain_complete: true,
        service_start_audit_boundary_present: true,
        service_start_audit_boundary_source_chain_complete: true,
        service_unload_cleanup_boundary_present: true,
        service_unload_cleanup_boundary_source_chain_complete: true,
        live_load_commit_boundary_present: true,
        live_load_commit_boundary_source_chain_complete: true,
        commit_audit_boundary_present: true,
        commit_audit_boundary_source_chain_complete: true,
        commit_rollback_boundary_present: true,
        commit_rollback_boundary_source_chain_complete: true,
        commit_result_boundary_present: true,
        commit_result_boundary_source_chain_complete: true,
        descriptor_acceptance_authority_boundary_present: true,
        descriptor_acceptance_authority_boundary_source_chain_complete: true,
        descriptor_parser_contract_boundary_present: true,
        descriptor_parser_contract_boundary_source_chain_complete: true,
        descriptor_parser_result_boundary_present: true,
        descriptor_parser_result_boundary_source_chain_complete: true,
        descriptor_schema_validation_boundary_present: true,
        descriptor_schema_validation_boundary_source_chain_complete: true,
        descriptor_capability_validation_boundary_present: true,
        descriptor_capability_validation_boundary_source_chain_complete: true,
        descriptor_load_plan_boundary_present: true,
        descriptor_load_plan_boundary_source_chain_complete: true,
        executable_load_plan_authority_boundary_present: true,
        executable_load_plan_authority_boundary_source_chain_complete: true,
        executable_load_plan_result_boundary_present: true,
        executable_load_plan_result_boundary_source_chain_complete: true,
        executable_image_layout_boundary_present: true,
        executable_image_layout_boundary_source_chain_complete: true,
        executable_page_mapping_plan_boundary_present: true,
        executable_page_mapping_plan_boundary_source_chain_complete: true,
        executable_page_mapping_boundary_present: true,
        executable_page_mapping_boundary_source_chain_complete: true,
        descriptor_executable_page_binding_boundary_present: true,
        descriptor_executable_page_binding_boundary_source_chain_complete: true,
        executable_entrypoint_binding_boundary_present: true,
        executable_entrypoint_binding_boundary_source_chain_complete: true,
        executable_entrypoint_transfer_authorization_boundary_present: true,
        executable_entrypoint_transfer_authorization_boundary_source_chain_complete: true,
        executable_entrypoint_transfer_boundary_present: true,
        executable_entrypoint_transfer_boundary_source_chain_complete: true,
        executable_entrypoint_handoff_boundary_present: true,
        executable_entrypoint_handoff_boundary_source_chain_complete: true,
        artifact_byte_intake_boundary_present: true,
        artifact_byte_intake_boundary_source_chain_complete: true,
        execution_authorization_boundary_present: true,
        execution_authorization_boundary_source_chain_complete: true,
        service_registry_mutation_boundary_present: true,
        service_registry_mutation_boundary_source_chain_complete: true,
        service_slot_binding_source_evidence_present: true,
        health_state_hooks_source_evidence_present: true,
        artifact_hash_binding_present: true,
        entrypoint_abi_source_evidence_present: true,
        address_space_source_evidence_present: true,
        memory_map_source_evidence_present: true,
        capability_import_table_source_evidence_present: true,
        audit_rollback_write_boundary_source_evidence_present: true,
        retained_module_evidence_present: true,
        retained_artifact_reference_present: true,
        retained_service_slot_reservation_present: true,
        loader_runtime_source_evidence_event_ids: [None; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_source_evidence_present: [true; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
        loader_runtime_fact_present: [true; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    }
}
