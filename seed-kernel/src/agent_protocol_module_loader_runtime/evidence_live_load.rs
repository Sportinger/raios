use crate::agent_protocol_module_types::{
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_ID, MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_REASON, MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SCHEMA,
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_METHOD, MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_STATUS,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_ID, MODULE_LOADER_COMMIT_RESULT_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_REASON, MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SCHEMA,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_STATUS, MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_ID,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_REASON, MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SCHEMA,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_ID,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_ID,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_ID,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_ID,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_ID,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_ID,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_ID,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_STATUS,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_ID,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_REASON, MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SCHEMA,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_STATUS, MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_ID,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SCHEMA,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_STATUS,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_ID,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SCHEMA,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_STATUS,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_ID,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SCHEMA,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_STATUS, MODULE_LOADER_SERVICE_START_BOUNDARY_ID,
    MODULE_LOADER_SERVICE_START_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_SERVICE_START_BOUNDARY_REASON, MODULE_LOADER_SERVICE_START_BOUNDARY_SCHEMA,
    MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_START_BOUNDARY_STATUS, MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_ID,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SCHEMA,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_STATUS,
};
use crate::event_log;

pub(super) fn module_loader_service_start_boundary_source_evidence(
    manifest_reference_event_id: Option<event_log::EventId>,
    artifact_reference_event_id: Option<event_log::EventId>,
    vm_test_report_reference_event_id: Option<event_log::EventId>,
    local_attestation_reference_event_id: Option<event_log::EventId>,
    local_approval_reference_event_id: Option<event_log::EventId>,
    computed_grant_reference_event_id: Option<event_log::EventId>,
    audit_rollback_reference_event_id: Option<event_log::EventId>,
    service_slot_reservation_event_id: Option<event_log::EventId>,
    ram_only_service_slot_id: Option<event_log::ModuleServiceSlotId>,
    entrypoint_transfer_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    service_registry_mutation_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderServiceRegistryMutationBoundarySourceEvidence,
    ),
    service_slot_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    health_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
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
    let entrypoint_transfer_boundary_present = entrypoint_transfer_boundary_source_evidence
        .1
        .boundary_present
        && entrypoint_transfer_boundary_source_evidence
            .1
            .source_chain_complete;
    let service_registry_mutation_boundary_present =
        service_registry_mutation_boundary_source_evidence
            .1
            .boundary_present
            && service_registry_mutation_boundary_source_evidence
                .1
                .source_chain_complete;
    let service_slot_binding_source_evidence_present = service_slot_source_evidence.is_some()
        || service_registry_mutation_boundary_source_evidence
            .1
            .service_slot_binding_source_evidence_present
        || entrypoint_transfer_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_present[6];
    let health_state_hooks_source_evidence_present = health_source_evidence.is_some()
        || entrypoint_transfer_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_present[7];
    let audit_rollback_write_boundary_source_evidence_present = write_boundary_source_evidence
        .is_some()
        || entrypoint_transfer_boundary_source_evidence
            .1
            .audit_rollback_write_boundary_source_evidence_present;
    let source_chain_complete = retained_module_evidence_present
        && entrypoint_transfer_boundary_present
        && service_registry_mutation_boundary_present
        && service_slot_binding_source_evidence_present
        && health_state_hooks_source_evidence_present
        && audit_rollback_write_boundary_source_evidence_present
        && service_slot_reservation_event_id.is_some();
    module_loader_live_load_boundary_source_evidence_record(
        MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_SERVICE_START_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_START_BOUNDARY_ID,
        MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_service_start_boundary_source_evidence_recorded",
        MODULE_LOADER_SERVICE_START_BOUNDARY_STATUS,
        MODULE_LOADER_SERVICE_START_BOUNDARY_REASON,
        MODULE_LOADER_SERVICE_START_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        source_chain_complete,
        entrypoint_transfer_boundary_source_evidence
            .1
            .load_attempt_boundary_present,
        entrypoint_transfer_boundary_source_evidence
            .1
            .load_attempt_boundary_source_chain_complete,
        entrypoint_transfer_boundary_source_evidence
            .1
            .artifact_load_boundary_present,
        entrypoint_transfer_boundary_source_evidence
            .1
            .artifact_load_boundary_source_chain_complete,
        entrypoint_transfer_boundary_source_evidence
            .1
            .executable_mapping_boundary_present,
        entrypoint_transfer_boundary_source_evidence
            .1
            .executable_mapping_boundary_source_chain_complete,
        entrypoint_transfer_boundary_present,
        entrypoint_transfer_boundary_source_evidence
            .1
            .source_chain_complete,
        entrypoint_transfer_boundary_source_evidence
            .1
            .artifact_byte_intake_boundary_present,
        entrypoint_transfer_boundary_source_evidence
            .1
            .artifact_byte_intake_boundary_source_chain_complete,
        entrypoint_transfer_boundary_source_evidence
            .1
            .execution_authorization_boundary_present,
        entrypoint_transfer_boundary_source_evidence
            .1
            .execution_authorization_boundary_source_chain_complete,
        service_registry_mutation_boundary_present,
        service_registry_mutation_boundary_source_evidence
            .1
            .source_chain_complete,
        service_slot_binding_source_evidence_present,
        health_state_hooks_source_evidence_present,
        entrypoint_transfer_boundary_source_evidence
            .1
            .artifact_hash_binding_present,
        entrypoint_transfer_boundary_source_evidence
            .1
            .entrypoint_abi_source_evidence_present,
        entrypoint_transfer_boundary_source_evidence
            .1
            .address_space_source_evidence_present,
        entrypoint_transfer_boundary_source_evidence
            .1
            .memory_map_source_evidence_present,
        entrypoint_transfer_boundary_source_evidence
            .1
            .capability_import_table_source_evidence_present,
        audit_rollback_write_boundary_source_evidence_present,
        retained_module_evidence_present,
        artifact_reference_event_id.is_some(),
        service_slot_reservation_event_id.is_some(),
        entrypoint_transfer_boundary_source_evidence
            .1
            .load_attempt_boundary_source_evidence_event_id,
        entrypoint_transfer_boundary_source_evidence
            .1
            .artifact_load_boundary_source_evidence_event_id,
        entrypoint_transfer_boundary_source_evidence
            .1
            .executable_mapping_boundary_source_evidence_event_id,
        Some(entrypoint_transfer_boundary_source_evidence.0),
        entrypoint_transfer_boundary_source_evidence
            .1
            .artifact_byte_intake_boundary_source_evidence_event_id,
        entrypoint_transfer_boundary_source_evidence
            .1
            .execution_authorization_boundary_source_evidence_event_id,
        Some(service_registry_mutation_boundary_source_evidence.0),
        service_slot_source_evidence
            .map(|(event_id, _)| event_id)
            .or(service_registry_mutation_boundary_source_evidence
                .1
                .service_slot_binding_source_evidence_event_id)
            .or(entrypoint_transfer_boundary_source_evidence
                .1
                .loader_runtime_source_evidence_event_ids[6]),
        health_source_evidence.map(|(event_id, _)| event_id).or(
            entrypoint_transfer_boundary_source_evidence
                .1
                .loader_runtime_source_evidence_event_ids[7],
        ),
        entrypoint_transfer_boundary_source_evidence
            .1
            .artifact_hash_binding_source_evidence_event_id,
        entrypoint_transfer_boundary_source_evidence
            .1
            .entrypoint_abi_source_evidence_event_id,
        entrypoint_transfer_boundary_source_evidence
            .1
            .address_space_source_evidence_event_id,
        entrypoint_transfer_boundary_source_evidence
            .1
            .memory_map_source_evidence_event_id,
        entrypoint_transfer_boundary_source_evidence
            .1
            .capability_import_table_source_evidence_event_id,
        write_boundary_source_evidence
            .map(|(event_id, _)| event_id)
            .or(entrypoint_transfer_boundary_source_evidence
                .1
                .audit_rollback_write_boundary_source_evidence_event_id),
        entrypoint_transfer_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_event_ids,
        entrypoint_transfer_boundary_source_evidence
            .1
            .loader_runtime_source_evidence_present,
        entrypoint_transfer_boundary_source_evidence
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

pub(super) fn module_loader_service_health_binding_boundary_source_evidence(
    service_start_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    service_slot_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    health_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    write_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_ID,
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_service_health_binding_boundary_source_evidence_recorded",
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_STATUS,
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_REASON,
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        service_start_boundary_source_evidence,
        true,
        true,
        false,
        true,
        service_slot_source_evidence,
        health_source_evidence,
        None,
        write_boundary_source_evidence,
    );
    let prior = service_start_boundary_source_evidence.1;
    evidence.service_start_boundary_present = prior.boundary_present && prior.source_chain_complete;
    evidence.service_start_boundary_source_chain_complete = prior.source_chain_complete;
    evidence.service_start_boundary_source_evidence_event_id =
        Some(service_start_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_service_running_state_boundary_source_evidence(
    service_health_binding_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    service_slot_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    health_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    write_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_ID,
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_service_running_state_boundary_source_evidence_recorded",
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_STATUS,
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_REASON,
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        service_health_binding_boundary_source_evidence,
        true,
        true,
        false,
        true,
        service_slot_source_evidence,
        health_source_evidence,
        None,
        write_boundary_source_evidence,
    );
    let prior = service_health_binding_boundary_source_evidence.1;
    evidence.service_health_binding_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.service_health_binding_boundary_source_chain_complete = prior.source_chain_complete;
    evidence.service_health_binding_boundary_source_evidence_event_id =
        Some(service_health_binding_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_service_start_audit_boundary_source_evidence(
    service_running_state_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    write_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_ID,
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_service_start_audit_boundary_source_evidence_recorded",
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_STATUS,
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_REASON,
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        service_running_state_boundary_source_evidence,
        false,
        false,
        false,
        true,
        None,
        None,
        None,
        write_boundary_source_evidence,
    );
    let prior = service_running_state_boundary_source_evidence.1;
    evidence.service_running_state_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.service_running_state_boundary_source_chain_complete = prior.source_chain_complete;
    evidence.service_running_state_boundary_source_evidence_event_id =
        Some(service_running_state_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_service_unload_cleanup_boundary_source_evidence(
    service_start_audit_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    service_slot_source_evidence: Option<(
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
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_ID,
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_service_unload_cleanup_boundary_source_evidence_recorded",
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_STATUS,
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_REASON,
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        service_start_audit_boundary_source_evidence,
        true,
        false,
        true,
        true,
        service_slot_source_evidence,
        None,
        rollback_source_evidence,
        write_boundary_source_evidence,
    );
    let prior = service_start_audit_boundary_source_evidence.1;
    evidence.service_start_audit_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.service_start_audit_boundary_source_chain_complete = prior.source_chain_complete;
    evidence.service_start_audit_boundary_source_evidence_event_id =
        Some(service_start_audit_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_live_load_commit_boundary_source_evidence(
    service_unload_cleanup_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    service_slot_source_evidence: Option<(
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
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SCHEMA,
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_ID,
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_live_load_commit_boundary_source_evidence_recorded",
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_STATUS,
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_REASON,
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        service_unload_cleanup_boundary_source_evidence,
        true,
        false,
        true,
        true,
        service_slot_source_evidence,
        None,
        rollback_source_evidence,
        write_boundary_source_evidence,
    );
    let prior = service_unload_cleanup_boundary_source_evidence.1;
    evidence.service_unload_cleanup_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.service_unload_cleanup_boundary_source_chain_complete = prior.source_chain_complete;
    evidence.service_unload_cleanup_boundary_source_evidence_event_id =
        Some(service_unload_cleanup_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_commit_audit_boundary_source_evidence(
    live_load_commit_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    write_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SCHEMA,
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_ID,
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_commit_audit_boundary_source_evidence_recorded",
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_STATUS,
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_REASON,
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        live_load_commit_boundary_source_evidence,
        false,
        false,
        false,
        true,
        None,
        None,
        None,
        write_boundary_source_evidence,
    );
    let prior = live_load_commit_boundary_source_evidence.1;
    evidence.live_load_commit_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.live_load_commit_boundary_source_chain_complete = prior.source_chain_complete;
    evidence.live_load_commit_boundary_source_evidence_event_id =
        Some(live_load_commit_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_commit_rollback_boundary_source_evidence(
    commit_audit_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    rollback_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
    write_boundary_source_evidence: Option<(
        event_log::EventId,
        event_log::ModuleLoaderFactSourceEvidence,
    )>,
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SCHEMA,
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_ID,
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_commit_rollback_boundary_source_evidence_recorded",
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_STATUS,
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_REASON,
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        commit_audit_boundary_source_evidence,
        false,
        false,
        true,
        true,
        None,
        None,
        rollback_source_evidence,
        write_boundary_source_evidence,
    );
    let prior = commit_audit_boundary_source_evidence.1;
    evidence.commit_audit_boundary_present = prior.boundary_present && prior.source_chain_complete;
    evidence.commit_audit_boundary_source_chain_complete = prior.source_chain_complete;
    evidence.commit_audit_boundary_source_evidence_event_id =
        Some(commit_audit_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_commit_result_boundary_source_evidence(
    commit_rollback_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SCHEMA,
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_ID,
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_commit_result_boundary_source_evidence_recorded",
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_STATUS,
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_REASON,
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        commit_rollback_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = commit_rollback_boundary_source_evidence.1;
    evidence.commit_rollback_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.commit_rollback_boundary_source_chain_complete = prior.source_chain_complete;
    evidence.commit_rollback_boundary_source_evidence_event_id =
        Some(commit_rollback_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_descriptor_acceptance_authority_boundary_source_evidence(
    commit_result_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_ID,
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_descriptor_acceptance_authority_boundary_source_evidence_recorded",
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_STATUS,
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_REASON,
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        commit_result_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = commit_result_boundary_source_evidence.1;
    evidence.commit_result_boundary_present = prior.boundary_present && prior.source_chain_complete;
    evidence.commit_result_boundary_source_chain_complete = prior.source_chain_complete;
    evidence.commit_result_boundary_source_evidence_event_id =
        Some(commit_result_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_descriptor_parser_contract_boundary_source_evidence(
    descriptor_acceptance_authority_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_ID,
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_descriptor_parser_contract_boundary_source_evidence_recorded",
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_STATUS,
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_REASON,
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        descriptor_acceptance_authority_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = descriptor_acceptance_authority_boundary_source_evidence.1;
    evidence.descriptor_acceptance_authority_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.descriptor_acceptance_authority_boundary_source_chain_complete =
        prior.source_chain_complete;
    evidence.descriptor_acceptance_authority_boundary_source_evidence_event_id =
        Some(descriptor_acceptance_authority_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_descriptor_parser_result_boundary_source_evidence(
    descriptor_parser_contract_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_ID,
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_descriptor_parser_result_boundary_source_evidence_recorded",
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_STATUS,
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_REASON,
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        descriptor_parser_contract_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = descriptor_parser_contract_boundary_source_evidence.1;
    evidence.descriptor_parser_contract_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.descriptor_parser_contract_boundary_source_chain_complete =
        prior.source_chain_complete;
    evidence.descriptor_parser_contract_boundary_source_evidence_event_id =
        Some(descriptor_parser_contract_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_descriptor_schema_validation_boundary_source_evidence(
    descriptor_parser_result_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_ID,
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_descriptor_schema_validation_boundary_source_evidence_recorded",
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_STATUS,
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_REASON,
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        descriptor_parser_result_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = descriptor_parser_result_boundary_source_evidence.1;
    evidence.descriptor_parser_result_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.descriptor_parser_result_boundary_source_chain_complete = prior.source_chain_complete;
    evidence.descriptor_parser_result_boundary_source_evidence_event_id =
        Some(descriptor_parser_result_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_descriptor_capability_validation_boundary_source_evidence(
    descriptor_schema_validation_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_ID,
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_descriptor_capability_validation_boundary_source_evidence_recorded",
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_STATUS,
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_REASON,
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        descriptor_schema_validation_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = descriptor_schema_validation_boundary_source_evidence.1;
    evidence.descriptor_schema_validation_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.descriptor_schema_validation_boundary_source_chain_complete =
        prior.source_chain_complete;
    evidence.descriptor_schema_validation_boundary_source_evidence_event_id =
        Some(descriptor_schema_validation_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_descriptor_load_plan_boundary_source_evidence(
    descriptor_capability_validation_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_ID,
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_descriptor_load_plan_boundary_source_evidence_recorded",
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_STATUS,
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_REASON,
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        descriptor_capability_validation_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = descriptor_capability_validation_boundary_source_evidence.1;
    evidence.descriptor_capability_validation_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.descriptor_capability_validation_boundary_source_chain_complete =
        prior.source_chain_complete;
    evidence.descriptor_capability_validation_boundary_source_evidence_event_id =
        Some(descriptor_capability_validation_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_executable_load_plan_authority_boundary_source_evidence(
    descriptor_load_plan_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_ID,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_executable_load_plan_authority_boundary_source_evidence_recorded",
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        descriptor_load_plan_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = descriptor_load_plan_boundary_source_evidence.1;
    evidence.descriptor_load_plan_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.descriptor_load_plan_boundary_source_chain_complete = prior.source_chain_complete;
    evidence.descriptor_load_plan_boundary_source_evidence_event_id =
        Some(descriptor_load_plan_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_executable_load_plan_result_boundary_source_evidence(
    executable_load_plan_authority_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_ID,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_executable_load_plan_result_boundary_source_evidence_recorded",
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        executable_load_plan_authority_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = executable_load_plan_authority_boundary_source_evidence.1;
    evidence.executable_load_plan_authority_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.executable_load_plan_authority_boundary_source_chain_complete =
        prior.source_chain_complete;
    evidence.executable_load_plan_authority_boundary_source_evidence_event_id =
        Some(executable_load_plan_authority_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_executable_image_layout_boundary_source_evidence(
    executable_load_plan_result_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_ID,
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_executable_image_layout_boundary_source_evidence_recorded",
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        executable_load_plan_result_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = executable_load_plan_result_boundary_source_evidence.1;
    evidence.executable_load_plan_result_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.executable_load_plan_result_boundary_source_chain_complete =
        prior.source_chain_complete;
    evidence.executable_load_plan_result_boundary_source_evidence_event_id =
        Some(executable_load_plan_result_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_executable_page_mapping_plan_boundary_source_evidence(
    executable_image_layout_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_ID,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_executable_page_mapping_plan_boundary_source_evidence_recorded",
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        executable_image_layout_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = executable_image_layout_boundary_source_evidence.1;
    evidence.executable_image_layout_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.executable_image_layout_boundary_source_chain_complete = prior.source_chain_complete;
    evidence.executable_image_layout_boundary_source_evidence_event_id =
        Some(executable_image_layout_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_executable_page_mapping_boundary_source_evidence(
    executable_page_mapping_plan_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_ID,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_executable_page_mapping_boundary_source_evidence_recorded",
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        executable_page_mapping_plan_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = executable_page_mapping_plan_boundary_source_evidence.1;
    evidence.executable_page_mapping_plan_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.executable_page_mapping_plan_boundary_source_chain_complete =
        prior.source_chain_complete;
    evidence.executable_page_mapping_plan_boundary_source_evidence_event_id =
        Some(executable_page_mapping_plan_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_descriptor_executable_page_binding_boundary_source_evidence(
    executable_page_mapping_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_ID,
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_descriptor_executable_page_binding_boundary_source_evidence_recorded",
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_STATUS,
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_REASON,
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        executable_page_mapping_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = executable_page_mapping_boundary_source_evidence.1;
    evidence.executable_page_mapping_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.executable_page_mapping_boundary_source_chain_complete = prior.source_chain_complete;
    evidence.executable_page_mapping_boundary_source_evidence_event_id =
        Some(executable_page_mapping_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_executable_entrypoint_binding_boundary_source_evidence(
    descriptor_executable_page_binding_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_ID,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_executable_entrypoint_binding_boundary_source_evidence_recorded",
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        descriptor_executable_page_binding_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = descriptor_executable_page_binding_boundary_source_evidence.1;
    evidence.descriptor_executable_page_binding_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.descriptor_executable_page_binding_boundary_source_chain_complete =
        prior.source_chain_complete;
    evidence.descriptor_executable_page_binding_boundary_source_evidence_event_id =
        Some(descriptor_executable_page_binding_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_executable_entrypoint_transfer_authorization_boundary_source_evidence(
    executable_entrypoint_binding_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_ID,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_executable_entrypoint_transfer_authorization_boundary_source_evidence_recorded",
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        executable_entrypoint_binding_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = executable_entrypoint_binding_boundary_source_evidence.1;
    evidence.executable_entrypoint_binding_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.executable_entrypoint_binding_boundary_source_chain_complete =
        prior.source_chain_complete;
    evidence.executable_entrypoint_binding_boundary_source_evidence_event_id =
        Some(executable_entrypoint_binding_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_executable_entrypoint_transfer_boundary_source_evidence(
    executable_entrypoint_transfer_authorization_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_ID,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_executable_entrypoint_transfer_boundary_source_evidence_recorded",
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        executable_entrypoint_transfer_authorization_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = executable_entrypoint_transfer_authorization_boundary_source_evidence.1;
    evidence.executable_entrypoint_transfer_authorization_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.executable_entrypoint_transfer_authorization_boundary_source_chain_complete =
        prior.source_chain_complete;
    evidence.executable_entrypoint_transfer_authorization_boundary_source_evidence_event_id =
        Some(executable_entrypoint_transfer_authorization_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_executable_entrypoint_handoff_boundary_source_evidence(
    executable_entrypoint_transfer_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_ID,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_executable_entrypoint_handoff_boundary_source_evidence_recorded",
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        executable_entrypoint_transfer_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = executable_entrypoint_transfer_boundary_source_evidence.1;
    evidence.executable_entrypoint_transfer_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.executable_entrypoint_transfer_boundary_source_chain_complete =
        prior.source_chain_complete;
    evidence.executable_entrypoint_transfer_boundary_source_evidence_event_id =
        Some(executable_entrypoint_transfer_boundary_source_evidence.0);
    evidence
}

pub(super) fn module_loader_executable_entrypoint_invocation_boundary_source_evidence(
    executable_entrypoint_handoff_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let mut evidence = module_loader_follow_on_live_load_boundary_source_evidence(
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_ID,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_METHOD,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_FACT_LOCATOR,
        "module_loader_executable_entrypoint_invocation_boundary_source_evidence_recorded",
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        executable_entrypoint_handoff_boundary_source_evidence,
        false,
        false,
        false,
        false,
        None,
        None,
        None,
        None,
    );
    let prior = executable_entrypoint_handoff_boundary_source_evidence.1;
    evidence.executable_entrypoint_handoff_boundary_present =
        prior.boundary_present && prior.source_chain_complete;
    evidence.executable_entrypoint_handoff_boundary_source_chain_complete =
        prior.source_chain_complete;
    evidence.executable_entrypoint_handoff_boundary_source_evidence_event_id =
        Some(executable_entrypoint_handoff_boundary_source_evidence.0);
    evidence
}

#[allow(clippy::too_many_arguments)]
fn module_loader_follow_on_live_load_boundary_source_evidence(
    schema: &'static str,
    boundary_schema: &'static str,
    boundary_id: &'static str,
    source_method: &'static str,
    source_fact_locator: &'static str,
    readiness_reason: &'static str,
    boundary_available_status: &'static str,
    boundary_available_reason: &'static str,
    boundary_missing_status: &'static str,
    boundary_missing_reason: &'static str,
    prior_boundary_source_evidence: (
        event_log::EventId,
        event_log::ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    require_service_slot_binding: bool,
    require_health_hooks: bool,
    require_rollback_hooks: bool,
    require_audit_write_boundary: bool,
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
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    let prior = prior_boundary_source_evidence.1;
    let prior_boundary_present = prior.boundary_present && prior.source_chain_complete;
    let service_slot_binding_source_evidence_present = service_slot_source_evidence.is_some()
        || prior.service_slot_binding_source_evidence_present
        || prior.loader_runtime_source_evidence_present[6];
    let health_state_hooks_source_evidence_present = health_source_evidence.is_some()
        || prior.health_state_hooks_source_evidence_present
        || prior.loader_runtime_source_evidence_present[7];
    let rollback_hooks_source_evidence_present =
        rollback_source_evidence.is_some() || prior.loader_runtime_source_evidence_present[8];
    let audit_rollback_write_boundary_source_evidence_present = write_boundary_source_evidence
        .is_some()
        || prior.audit_rollback_write_boundary_source_evidence_present
        || prior.loader_runtime_source_evidence_present[9];
    let source_chain_complete = prior.retained_module_evidence_present
        && prior.retained_service_slot_reservation_present
        && prior_boundary_present
        && (!require_service_slot_binding || service_slot_binding_source_evidence_present)
        && (!require_health_hooks || health_state_hooks_source_evidence_present)
        && (!require_rollback_hooks || rollback_hooks_source_evidence_present)
        && (!require_audit_write_boundary || audit_rollback_write_boundary_source_evidence_present);

    let mut evidence = module_loader_live_load_boundary_source_evidence_record(
        schema,
        boundary_schema,
        boundary_id,
        source_method,
        source_fact_locator,
        readiness_reason,
        boundary_available_status,
        boundary_available_reason,
        boundary_missing_status,
        boundary_missing_reason,
        source_chain_complete,
        prior.load_attempt_boundary_present,
        prior.load_attempt_boundary_source_chain_complete,
        prior.artifact_load_boundary_present,
        prior.artifact_load_boundary_source_chain_complete,
        prior.executable_mapping_boundary_present,
        prior.executable_mapping_boundary_source_chain_complete,
        prior.entrypoint_transfer_boundary_present,
        prior.entrypoint_transfer_boundary_source_chain_complete,
        prior.artifact_byte_intake_boundary_present,
        prior.artifact_byte_intake_boundary_source_chain_complete,
        prior.execution_authorization_boundary_present,
        prior.execution_authorization_boundary_source_chain_complete,
        prior.service_registry_mutation_boundary_present,
        prior.service_registry_mutation_boundary_source_chain_complete,
        service_slot_binding_source_evidence_present,
        health_state_hooks_source_evidence_present,
        prior.artifact_hash_binding_present,
        prior.entrypoint_abi_source_evidence_present,
        prior.address_space_source_evidence_present,
        prior.memory_map_source_evidence_present,
        prior.capability_import_table_source_evidence_present,
        audit_rollback_write_boundary_source_evidence_present,
        prior.retained_module_evidence_present,
        prior.retained_artifact_reference_present,
        prior.retained_service_slot_reservation_present,
        prior.load_attempt_boundary_source_evidence_event_id,
        prior.artifact_load_boundary_source_evidence_event_id,
        prior.executable_mapping_boundary_source_evidence_event_id,
        prior.entrypoint_transfer_boundary_source_evidence_event_id,
        prior.artifact_byte_intake_boundary_source_evidence_event_id,
        prior.execution_authorization_boundary_source_evidence_event_id,
        prior.service_registry_mutation_boundary_source_evidence_event_id,
        service_slot_source_evidence
            .map(|(event_id, _)| event_id)
            .or(prior.service_slot_binding_source_evidence_event_id)
            .or(prior.loader_runtime_source_evidence_event_ids[6]),
        health_source_evidence
            .map(|(event_id, _)| event_id)
            .or(prior.health_state_hooks_source_evidence_event_id)
            .or(prior.loader_runtime_source_evidence_event_ids[7]),
        prior.artifact_hash_binding_source_evidence_event_id,
        prior.entrypoint_abi_source_evidence_event_id,
        prior.address_space_source_evidence_event_id,
        prior.memory_map_source_evidence_event_id,
        prior.capability_import_table_source_evidence_event_id,
        write_boundary_source_evidence
            .map(|(event_id, _)| event_id)
            .or(prior.audit_rollback_write_boundary_source_evidence_event_id)
            .or(prior.loader_runtime_source_evidence_event_ids[9]),
        prior.loader_runtime_source_evidence_event_ids,
        prior.loader_runtime_source_evidence_present,
        prior.loader_runtime_fact_present,
        prior.manifest_reference_event_id,
        prior.artifact_reference_event_id,
        prior.vm_test_report_reference_event_id,
        prior.local_attestation_reference_event_id,
        prior.local_approval_reference_event_id,
        prior.computed_grant_reference_event_id,
        prior.audit_rollback_reference_event_id,
        prior.service_slot_reservation_event_id,
        prior.ram_only_service_slot_id,
    );
    evidence.service_start_boundary_present = prior.service_start_boundary_present;
    evidence.service_start_boundary_source_chain_complete =
        prior.service_start_boundary_source_chain_complete;
    evidence.service_health_binding_boundary_present =
        prior.service_health_binding_boundary_present;
    evidence.service_health_binding_boundary_source_chain_complete =
        prior.service_health_binding_boundary_source_chain_complete;
    evidence.service_running_state_boundary_present = prior.service_running_state_boundary_present;
    evidence.service_running_state_boundary_source_chain_complete =
        prior.service_running_state_boundary_source_chain_complete;
    evidence.service_start_audit_boundary_present = prior.service_start_audit_boundary_present;
    evidence.service_start_audit_boundary_source_chain_complete =
        prior.service_start_audit_boundary_source_chain_complete;
    evidence.service_unload_cleanup_boundary_present =
        prior.service_unload_cleanup_boundary_present;
    evidence.service_unload_cleanup_boundary_source_chain_complete =
        prior.service_unload_cleanup_boundary_source_chain_complete;
    evidence.live_load_commit_boundary_present = prior.live_load_commit_boundary_present;
    evidence.live_load_commit_boundary_source_chain_complete =
        prior.live_load_commit_boundary_source_chain_complete;
    evidence.commit_audit_boundary_present = prior.commit_audit_boundary_present;
    evidence.commit_audit_boundary_source_chain_complete =
        prior.commit_audit_boundary_source_chain_complete;
    evidence.commit_rollback_boundary_present = prior.commit_rollback_boundary_present;
    evidence.commit_rollback_boundary_source_chain_complete =
        prior.commit_rollback_boundary_source_chain_complete;
    evidence.commit_result_boundary_present = prior.commit_result_boundary_present;
    evidence.commit_result_boundary_source_chain_complete =
        prior.commit_result_boundary_source_chain_complete;
    evidence.descriptor_acceptance_authority_boundary_present =
        prior.descriptor_acceptance_authority_boundary_present;
    evidence.descriptor_acceptance_authority_boundary_source_chain_complete =
        prior.descriptor_acceptance_authority_boundary_source_chain_complete;
    evidence.descriptor_parser_contract_boundary_present =
        prior.descriptor_parser_contract_boundary_present;
    evidence.descriptor_parser_contract_boundary_source_chain_complete =
        prior.descriptor_parser_contract_boundary_source_chain_complete;
    evidence.descriptor_parser_result_boundary_present =
        prior.descriptor_parser_result_boundary_present;
    evidence.descriptor_parser_result_boundary_source_chain_complete =
        prior.descriptor_parser_result_boundary_source_chain_complete;
    evidence.descriptor_schema_validation_boundary_present =
        prior.descriptor_schema_validation_boundary_present;
    evidence.descriptor_schema_validation_boundary_source_chain_complete =
        prior.descriptor_schema_validation_boundary_source_chain_complete;
    evidence.descriptor_capability_validation_boundary_present =
        prior.descriptor_capability_validation_boundary_present;
    evidence.descriptor_capability_validation_boundary_source_chain_complete =
        prior.descriptor_capability_validation_boundary_source_chain_complete;
    evidence.descriptor_load_plan_boundary_present = prior.descriptor_load_plan_boundary_present;
    evidence.descriptor_load_plan_boundary_source_chain_complete =
        prior.descriptor_load_plan_boundary_source_chain_complete;
    evidence.executable_load_plan_authority_boundary_present =
        prior.executable_load_plan_authority_boundary_present;
    evidence.executable_load_plan_authority_boundary_source_chain_complete =
        prior.executable_load_plan_authority_boundary_source_chain_complete;
    evidence.executable_load_plan_result_boundary_present =
        prior.executable_load_plan_result_boundary_present;
    evidence.executable_load_plan_result_boundary_source_chain_complete =
        prior.executable_load_plan_result_boundary_source_chain_complete;
    evidence.executable_image_layout_boundary_present =
        prior.executable_image_layout_boundary_present;
    evidence.executable_image_layout_boundary_source_chain_complete =
        prior.executable_image_layout_boundary_source_chain_complete;
    evidence.executable_page_mapping_plan_boundary_present =
        prior.executable_page_mapping_plan_boundary_present;
    evidence.executable_page_mapping_plan_boundary_source_chain_complete =
        prior.executable_page_mapping_plan_boundary_source_chain_complete;
    evidence.executable_page_mapping_boundary_present =
        prior.executable_page_mapping_boundary_present;
    evidence.executable_page_mapping_boundary_source_chain_complete =
        prior.executable_page_mapping_boundary_source_chain_complete;
    evidence.descriptor_executable_page_binding_boundary_present =
        prior.descriptor_executable_page_binding_boundary_present;
    evidence.descriptor_executable_page_binding_boundary_source_chain_complete =
        prior.descriptor_executable_page_binding_boundary_source_chain_complete;
    evidence.executable_entrypoint_binding_boundary_present =
        prior.executable_entrypoint_binding_boundary_present;
    evidence.executable_entrypoint_binding_boundary_source_chain_complete =
        prior.executable_entrypoint_binding_boundary_source_chain_complete;
    evidence.executable_entrypoint_transfer_authorization_boundary_present =
        prior.executable_entrypoint_transfer_authorization_boundary_present;
    evidence.executable_entrypoint_transfer_authorization_boundary_source_chain_complete =
        prior.executable_entrypoint_transfer_authorization_boundary_source_chain_complete;
    evidence.executable_entrypoint_transfer_boundary_present =
        prior.executable_entrypoint_transfer_boundary_present;
    evidence.executable_entrypoint_transfer_boundary_source_chain_complete =
        prior.executable_entrypoint_transfer_boundary_source_chain_complete;
    evidence.executable_entrypoint_handoff_boundary_present =
        prior.executable_entrypoint_handoff_boundary_present;
    evidence.executable_entrypoint_handoff_boundary_source_chain_complete =
        prior.executable_entrypoint_handoff_boundary_source_chain_complete;
    evidence.service_start_boundary_source_evidence_event_id =
        prior.service_start_boundary_source_evidence_event_id;
    evidence.service_health_binding_boundary_source_evidence_event_id =
        prior.service_health_binding_boundary_source_evidence_event_id;
    evidence.service_running_state_boundary_source_evidence_event_id =
        prior.service_running_state_boundary_source_evidence_event_id;
    evidence.service_start_audit_boundary_source_evidence_event_id =
        prior.service_start_audit_boundary_source_evidence_event_id;
    evidence.service_unload_cleanup_boundary_source_evidence_event_id =
        prior.service_unload_cleanup_boundary_source_evidence_event_id;
    evidence.live_load_commit_boundary_source_evidence_event_id =
        prior.live_load_commit_boundary_source_evidence_event_id;
    evidence.commit_audit_boundary_source_evidence_event_id =
        prior.commit_audit_boundary_source_evidence_event_id;
    evidence.commit_rollback_boundary_source_evidence_event_id =
        prior.commit_rollback_boundary_source_evidence_event_id;
    evidence.commit_result_boundary_source_evidence_event_id =
        prior.commit_result_boundary_source_evidence_event_id;
    evidence.descriptor_acceptance_authority_boundary_source_evidence_event_id =
        prior.descriptor_acceptance_authority_boundary_source_evidence_event_id;
    evidence.descriptor_parser_contract_boundary_source_evidence_event_id =
        prior.descriptor_parser_contract_boundary_source_evidence_event_id;
    evidence.descriptor_parser_result_boundary_source_evidence_event_id =
        prior.descriptor_parser_result_boundary_source_evidence_event_id;
    evidence.descriptor_schema_validation_boundary_source_evidence_event_id =
        prior.descriptor_schema_validation_boundary_source_evidence_event_id;
    evidence.descriptor_capability_validation_boundary_source_evidence_event_id =
        prior.descriptor_capability_validation_boundary_source_evidence_event_id;
    evidence.descriptor_load_plan_boundary_source_evidence_event_id =
        prior.descriptor_load_plan_boundary_source_evidence_event_id;
    evidence.executable_load_plan_authority_boundary_source_evidence_event_id =
        prior.executable_load_plan_authority_boundary_source_evidence_event_id;
    evidence.executable_load_plan_result_boundary_source_evidence_event_id =
        prior.executable_load_plan_result_boundary_source_evidence_event_id;
    evidence.executable_image_layout_boundary_source_evidence_event_id =
        prior.executable_image_layout_boundary_source_evidence_event_id;
    evidence.executable_page_mapping_plan_boundary_source_evidence_event_id =
        prior.executable_page_mapping_plan_boundary_source_evidence_event_id;
    evidence.executable_page_mapping_boundary_source_evidence_event_id =
        prior.executable_page_mapping_boundary_source_evidence_event_id;
    evidence.descriptor_executable_page_binding_boundary_source_evidence_event_id =
        prior.descriptor_executable_page_binding_boundary_source_evidence_event_id;
    evidence.executable_entrypoint_binding_boundary_source_evidence_event_id =
        prior.executable_entrypoint_binding_boundary_source_evidence_event_id;
    evidence.executable_entrypoint_transfer_authorization_boundary_source_evidence_event_id =
        prior.executable_entrypoint_transfer_authorization_boundary_source_evidence_event_id;
    evidence.executable_entrypoint_transfer_boundary_source_evidence_event_id =
        prior.executable_entrypoint_transfer_boundary_source_evidence_event_id;
    evidence.executable_entrypoint_handoff_boundary_source_evidence_event_id =
        prior.executable_entrypoint_handoff_boundary_source_evidence_event_id;
    evidence
}

pub(super) fn module_loader_retained_module_evidence_present(
    manifest_reference_event_id: Option<event_log::EventId>,
    artifact_reference_event_id: Option<event_log::EventId>,
    vm_test_report_reference_event_id: Option<event_log::EventId>,
    local_attestation_reference_event_id: Option<event_log::EventId>,
    local_approval_reference_event_id: Option<event_log::EventId>,
    computed_grant_reference_event_id: Option<event_log::EventId>,
    audit_rollback_reference_event_id: Option<event_log::EventId>,
    service_slot_reservation_event_id: Option<event_log::EventId>,
) -> bool {
    manifest_reference_event_id.is_some()
        && artifact_reference_event_id.is_some()
        && vm_test_report_reference_event_id.is_some()
        && local_attestation_reference_event_id.is_some()
        && local_approval_reference_event_id.is_some()
        && computed_grant_reference_event_id.is_some()
        && audit_rollback_reference_event_id.is_some()
        && service_slot_reservation_event_id.is_some()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn module_loader_live_load_boundary_source_evidence_record(
    schema: &'static str,
    boundary_schema: &'static str,
    boundary_id: &'static str,
    source_method: &'static str,
    source_fact_locator: &'static str,
    readiness_reason: &'static str,
    boundary_available_status: &'static str,
    boundary_available_reason: &'static str,
    boundary_missing_status: &'static str,
    boundary_missing_reason: &'static str,
    source_chain_complete: bool,
    load_attempt_boundary_present: bool,
    load_attempt_boundary_source_chain_complete: bool,
    artifact_load_boundary_present: bool,
    artifact_load_boundary_source_chain_complete: bool,
    executable_mapping_boundary_present: bool,
    executable_mapping_boundary_source_chain_complete: bool,
    entrypoint_transfer_boundary_present: bool,
    entrypoint_transfer_boundary_source_chain_complete: bool,
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
    load_attempt_boundary_source_evidence_event_id: Option<event_log::EventId>,
    artifact_load_boundary_source_evidence_event_id: Option<event_log::EventId>,
    executable_mapping_boundary_source_evidence_event_id: Option<event_log::EventId>,
    entrypoint_transfer_boundary_source_evidence_event_id: Option<event_log::EventId>,
    artifact_byte_intake_boundary_source_evidence_event_id: Option<event_log::EventId>,
    execution_authorization_boundary_source_evidence_event_id: Option<event_log::EventId>,
    service_registry_mutation_boundary_source_evidence_event_id: Option<event_log::EventId>,
    service_slot_binding_source_evidence_event_id: Option<event_log::EventId>,
    health_state_hooks_source_evidence_event_id: Option<event_log::EventId>,
    artifact_hash_binding_source_evidence_event_id: Option<event_log::EventId>,
    entrypoint_abi_source_evidence_event_id: Option<event_log::EventId>,
    address_space_source_evidence_event_id: Option<event_log::EventId>,
    memory_map_source_evidence_event_id: Option<event_log::EventId>,
    capability_import_table_source_evidence_event_id: Option<event_log::EventId>,
    audit_rollback_write_boundary_source_evidence_event_id: Option<event_log::EventId>,
    loader_runtime_source_evidence_event_ids: [Option<event_log::EventId>;
        MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    loader_runtime_source_evidence_present: [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    loader_runtime_fact_present: [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    manifest_reference_event_id: Option<event_log::EventId>,
    artifact_reference_event_id: Option<event_log::EventId>,
    vm_test_report_reference_event_id: Option<event_log::EventId>,
    local_attestation_reference_event_id: Option<event_log::EventId>,
    local_approval_reference_event_id: Option<event_log::EventId>,
    computed_grant_reference_event_id: Option<event_log::EventId>,
    audit_rollback_reference_event_id: Option<event_log::EventId>,
    service_slot_reservation_event_id: Option<event_log::EventId>,
    ram_only_service_slot_id: Option<event_log::ModuleServiceSlotId>,
) -> event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
    event_log::ModuleLoaderLiveLoadBoundarySourceEvidence {
        schema,
        boundary_schema,
        boundary_id,
        source_method,
        source_fact_locator,
        readiness_status: "retained_current_boot_source_evidence",
        readiness_reason,
        boundary_status: if source_chain_complete {
            boundary_available_status
        } else {
            boundary_missing_status
        },
        boundary_reason: if source_chain_complete {
            boundary_available_reason
        } else {
            boundary_missing_reason
        },
        boundary_present: source_chain_complete,
        boundary_scope: "current_boot",
        boundary_schema_ok: true,
        boundary_provenance_ok: source_chain_complete,
        boundary_classification: "local_only",
        requested_capability: "cap.module.load_ephemeral",
        load_mode: "ram_only",
        target: "live_service_graph",
        load_attempt_boundary_present,
        load_attempt_boundary_source_chain_complete,
        artifact_load_boundary_present,
        artifact_load_boundary_source_chain_complete,
        executable_mapping_boundary_present,
        executable_mapping_boundary_source_chain_complete,
        entrypoint_transfer_boundary_present,
        entrypoint_transfer_boundary_source_chain_complete,
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
        artifact_byte_intake_boundary_present,
        artifact_byte_intake_boundary_source_chain_complete,
        execution_authorization_boundary_present,
        execution_authorization_boundary_source_chain_complete,
        service_registry_mutation_boundary_present,
        service_registry_mutation_boundary_source_chain_complete,
        service_slot_binding_source_evidence_present,
        health_state_hooks_source_evidence_present,
        artifact_hash_binding_present,
        entrypoint_abi_source_evidence_present,
        address_space_source_evidence_present,
        memory_map_source_evidence_present,
        capability_import_table_source_evidence_present,
        audit_rollback_write_boundary_source_evidence_present,
        retained_module_evidence_present,
        retained_artifact_reference_present,
        retained_service_slot_reservation_present,
        source_chain_complete,
        load_attempt_boundary_source_evidence_event_id,
        artifact_load_boundary_source_evidence_event_id,
        executable_mapping_boundary_source_evidence_event_id,
        entrypoint_transfer_boundary_source_evidence_event_id,
        service_start_boundary_source_evidence_event_id: None,
        service_health_binding_boundary_source_evidence_event_id: None,
        service_running_state_boundary_source_evidence_event_id: None,
        service_start_audit_boundary_source_evidence_event_id: None,
        service_unload_cleanup_boundary_source_evidence_event_id: None,
        live_load_commit_boundary_source_evidence_event_id: None,
        commit_audit_boundary_source_evidence_event_id: None,
        commit_rollback_boundary_source_evidence_event_id: None,
        commit_result_boundary_source_evidence_event_id: None,
        descriptor_acceptance_authority_boundary_source_evidence_event_id: None,
        descriptor_parser_contract_boundary_source_evidence_event_id: None,
        descriptor_parser_result_boundary_source_evidence_event_id: None,
        descriptor_schema_validation_boundary_source_evidence_event_id: None,
        descriptor_capability_validation_boundary_source_evidence_event_id: None,
        descriptor_load_plan_boundary_source_evidence_event_id: None,
        executable_load_plan_authority_boundary_source_evidence_event_id: None,
        executable_load_plan_result_boundary_source_evidence_event_id: None,
        executable_image_layout_boundary_source_evidence_event_id: None,
        executable_page_mapping_plan_boundary_source_evidence_event_id: None,
        executable_page_mapping_boundary_source_evidence_event_id: None,
        descriptor_executable_page_binding_boundary_source_evidence_event_id: None,
        executable_entrypoint_binding_boundary_source_evidence_event_id: None,
        executable_entrypoint_transfer_authorization_boundary_source_evidence_event_id: None,
        executable_entrypoint_transfer_boundary_source_evidence_event_id: None,
        executable_entrypoint_handoff_boundary_source_evidence_event_id: None,
        artifact_byte_intake_boundary_source_evidence_event_id,
        execution_authorization_boundary_source_evidence_event_id,
        service_registry_mutation_boundary_source_evidence_event_id,
        service_slot_binding_source_evidence_event_id,
        health_state_hooks_source_evidence_event_id,
        artifact_hash_binding_source_evidence_event_id,
        entrypoint_abi_source_evidence_event_id,
        address_space_source_evidence_event_id,
        memory_map_source_evidence_event_id,
        capability_import_table_source_evidence_event_id,
        audit_rollback_write_boundary_source_evidence_event_id,
        loader_runtime_source_evidence_event_ids,
        loader_runtime_source_evidence_present,
        loader_runtime_fact_present,
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
        starts_service: false,
        marks_service_running: false,
        creates_service_health_records: false,
        writes_service_start_audit_record: false,
        unloads_service: false,
        cleans_up_service_slot: false,
        commits_live_load: false,
        writes_load_commit_audit_record: false,
        installs_commit_rollback_record: false,
        records_load_result: false,
        load_attempted: false,
    }
}
