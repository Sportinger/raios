use self::eval::evaluate_module_loader_runtime_candidate;
use self::evidence_core::{
    module_loader_artifact_byte_intake_boundary_source_evidence,
    module_loader_artifact_load_boundary_source_evidence,
    module_loader_descriptor_intake_boundary_source_evidence,
    module_loader_entrypoint_transfer_boundary_source_evidence,
    module_loader_executable_mapping_boundary_source_evidence,
    module_loader_execution_authorization_boundary_source_evidence,
    module_loader_load_attempt_boundary_source_evidence,
    module_loader_runtime_execution_commit_gate_source_evidence,
    module_loader_runtime_selftest_case_value, module_loader_runtime_source_fact_map,
    module_loader_service_registry_mutation_boundary_source_evidence,
};
use self::evidence_live_load::{
    module_loader_commit_audit_boundary_source_evidence,
    module_loader_commit_result_boundary_source_evidence,
    module_loader_commit_rollback_boundary_source_evidence,
    module_loader_descriptor_acceptance_authority_boundary_source_evidence,
    module_loader_descriptor_capability_validation_boundary_source_evidence,
    module_loader_descriptor_executable_page_binding_boundary_source_evidence,
    module_loader_descriptor_load_plan_boundary_source_evidence,
    module_loader_descriptor_parser_contract_boundary_source_evidence,
    module_loader_descriptor_parser_result_boundary_source_evidence,
    module_loader_descriptor_schema_validation_boundary_source_evidence,
    module_loader_executable_entrypoint_binding_boundary_source_evidence,
    module_loader_executable_entrypoint_handoff_boundary_source_evidence,
    module_loader_executable_entrypoint_invocation_boundary_source_evidence,
    module_loader_executable_entrypoint_transfer_authorization_boundary_source_evidence,
    module_loader_executable_entrypoint_transfer_boundary_source_evidence,
    module_loader_executable_image_layout_boundary_source_evidence,
    module_loader_executable_load_plan_authority_boundary_source_evidence,
    module_loader_executable_load_plan_result_boundary_source_evidence,
    module_loader_executable_page_mapping_boundary_source_evidence,
    module_loader_executable_page_mapping_plan_boundary_source_evidence,
    module_loader_live_load_commit_boundary_source_evidence,
    module_loader_service_health_binding_boundary_source_evidence,
    module_loader_service_running_state_boundary_source_evidence,
    module_loader_service_start_audit_boundary_source_evidence,
    module_loader_service_start_boundary_source_evidence,
    module_loader_service_unload_cleanup_boundary_source_evidence,
};
use self::render::{
    emit_module_loader_artifact_byte_intake_boundary,
    emit_module_loader_descriptor_intake_boundary,
    emit_module_loader_execution_authorization_boundary, emit_module_loader_live_load_boundary,
    emit_module_loader_runtime_execution_commit_gate, emit_module_loader_runtime_facts,
    emit_module_loader_runtime_retained_evidence,
    emit_module_loader_runtime_service_slot_allocator_readiness,
    emit_module_loader_service_registry_mutation_boundary, module_loader_runtime_blocked_by,
    module_loader_runtime_header_fields, module_loader_runtime_policy_result_fields,
};
use self::selftest::module_loader_runtime_selftest_cases;
use self::snapshot::module_loader_runtime_snapshot;
use crate::agent_protocol_module_types::{
    module_loader_runtime_source_fact_map_complete, MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_ID,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SCHEMA, MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_ID,
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SCHEMA, MODULE_LOADER_COMMIT_RESULT_BOUNDARY_ID,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SCHEMA, MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_ID,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_ID,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_ID,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_ID,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_ID,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_ID,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_ID,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_ID,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SCHEMA,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_ID,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_ID, MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_ID,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SCHEMA,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_ID, MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SCHEMA,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_ID, MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SCHEMA,
    MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT, MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_ID,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SCHEMA,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_ID,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SCHEMA,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_ID,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SCHEMA, MODULE_LOADER_SERVICE_START_BOUNDARY_ID,
    MODULE_LOADER_SERVICE_START_BOUNDARY_SCHEMA, MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_ID,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SCHEMA,
    MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES,
};
use crate::agent_protocol_support::{
    begin_response, emit_record_fields, emit_record_fields_trailing_comma,
    emit_record_property_line, emit_record_value_property_line, end_response, raw_line,
    record_bool as b, record_false as no, record_field as f, record_str as s,
};
use crate::{event_log, granted_candidate_service};
use alloc::{vec, vec::Vec};
use raios_core::record::Value as V;

mod eval;
mod evidence_core;
mod evidence_live_load;
mod render;
mod selftest;
mod snapshot;

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
    let entrypoint_abi_source_evidence =
        event_log::latest_module_loader_fact_source_evidence("module.loader_entrypoint_abi");
    let address_space_source_evidence = event_log::latest_module_loader_fact_source_evidence(
        "module.loader_address_space_boundary",
    );
    let memory_map_source_evidence = event_log::latest_module_loader_fact_source_evidence(
        "module.loader_memory_map_constraints",
    );
    let capability_table_source_evidence = event_log::latest_module_loader_fact_source_evidence(
        "module.loader_capability_import_table",
    );
    let service_slot_source_evidence =
        event_log::latest_module_loader_fact_source_evidence("module.loader_service_slot_binding");
    let health_source_evidence =
        event_log::latest_module_loader_fact_source_evidence("module.loader_health_state_hooks");
    let rollback_source_evidence =
        event_log::latest_module_loader_fact_source_evidence("module.loader_rollback_hooks");
    let write_boundary_source_evidence = event_log::latest_module_loader_fact_source_evidence(
        "module.loader_audit_rollback_write_boundary_binding",
    );
    let authority_decision_source_evidence =
        event_log::latest_module_service_slot_allocator_authority_decision_source_evidence();
    let loader_runtime_contract_source_evidence =
        event_log::latest_module_service_slot_authority_input_source_evidence(
            MODULE_SERVICE_SLOT_AUTHORITY_INPUT_SOURCES[2].source_fact_locator,
        );
    let execution_commit_gate_source_evidence =
        module_loader_runtime_execution_commit_gate_source_evidence(
            service_slot.is_some(),
            service_slot.as_ref().map(|(event_id, _)| *event_id),
            service_slot
                .as_ref()
                .map(|(_, reservation)| reservation.ram_only_service_slot_id),
            authority_decision_source_evidence,
            loader_runtime_contract_source_evidence,
            loader_identity_source_evidence,
            artifact_hash_binding_source_evidence,
            entrypoint_abi_source_evidence,
            address_space_source_evidence,
            memory_map_source_evidence,
            capability_table_source_evidence,
            service_slot_source_evidence,
            health_source_evidence,
            rollback_source_evidence,
            write_boundary_source_evidence,
        );
    let execution_commit_gate_source_evidence_event_id =
        event_log::record_module_loader_runtime_execution_commit_gate_source_evidence(
            execution_commit_gate_source_evidence,
        );
    let registry_write_commit_gate_source_evidence =
        event_log::latest_module_service_slot_registry_write_commit_gate_source_evidence();
    let descriptor_intake_boundary_source_evidence =
        module_loader_descriptor_intake_boundary_source_evidence(
            manifest.as_ref().map(|(event_id, _)| *event_id),
            artifact.as_ref().map(|(event_id, _)| *event_id),
            vm_report.as_ref().map(|(event_id, _)| *event_id),
            local_attestation.as_ref().map(|(event_id, _)| *event_id),
            local_approval.as_ref().map(|(event_id, _)| *event_id),
            computed_grant.as_ref().map(|(event_id, _)| *event_id),
            audit_rollback.as_ref().map(|(event_id, _)| *event_id),
            service_slot.as_ref().map(|(event_id, _)| *event_id),
            service_slot
                .as_ref()
                .map(|(_, reservation)| reservation.ram_only_service_slot_id),
            registry_write_commit_gate_source_evidence,
            (
                execution_commit_gate_source_evidence_event_id,
                execution_commit_gate_source_evidence,
            ),
        );
    let descriptor_intake_boundary_source_evidence_event_id =
        event_log::record_module_loader_descriptor_intake_boundary_source_evidence(
            descriptor_intake_boundary_source_evidence,
        );
    let artifact_byte_intake_boundary_source_evidence =
        module_loader_artifact_byte_intake_boundary_source_evidence(
            manifest.as_ref().map(|(event_id, _)| *event_id),
            artifact.as_ref().map(|(event_id, _)| *event_id),
            vm_report.as_ref().map(|(event_id, _)| *event_id),
            local_attestation.as_ref().map(|(event_id, _)| *event_id),
            local_approval.as_ref().map(|(event_id, _)| *event_id),
            computed_grant.as_ref().map(|(event_id, _)| *event_id),
            audit_rollback.as_ref().map(|(event_id, _)| *event_id),
            service_slot.as_ref().map(|(event_id, _)| *event_id),
            service_slot
                .as_ref()
                .map(|(_, reservation)| reservation.ram_only_service_slot_id),
            (
                execution_commit_gate_source_evidence_event_id,
                execution_commit_gate_source_evidence,
            ),
            (
                descriptor_intake_boundary_source_evidence_event_id,
                descriptor_intake_boundary_source_evidence,
            ),
            artifact_hash_binding_source_evidence,
        );
    let artifact_byte_intake_boundary_source_evidence_event_id =
        event_log::record_module_loader_artifact_byte_intake_boundary_source_evidence(
            artifact_byte_intake_boundary_source_evidence,
        );
    let execution_authorization_boundary_source_evidence =
        module_loader_execution_authorization_boundary_source_evidence(
            manifest.as_ref().map(|(event_id, _)| *event_id),
            artifact.as_ref().map(|(event_id, _)| *event_id),
            vm_report.as_ref().map(|(event_id, _)| *event_id),
            local_attestation.as_ref().map(|(event_id, _)| *event_id),
            local_approval.as_ref().map(|(event_id, _)| *event_id),
            computed_grant.as_ref().map(|(event_id, _)| *event_id),
            audit_rollback.as_ref().map(|(event_id, _)| *event_id),
            service_slot.as_ref().map(|(event_id, _)| *event_id),
            service_slot
                .as_ref()
                .map(|(_, reservation)| reservation.ram_only_service_slot_id),
            (
                execution_commit_gate_source_evidence_event_id,
                execution_commit_gate_source_evidence,
            ),
            (
                descriptor_intake_boundary_source_evidence_event_id,
                descriptor_intake_boundary_source_evidence,
            ),
            (
                artifact_byte_intake_boundary_source_evidence_event_id,
                artifact_byte_intake_boundary_source_evidence,
            ),
            entrypoint_abi_source_evidence,
            address_space_source_evidence,
            memory_map_source_evidence,
            write_boundary_source_evidence,
        );
    let execution_authorization_boundary_source_evidence_event_id =
        event_log::record_module_loader_execution_authorization_boundary_source_evidence(
            execution_authorization_boundary_source_evidence,
        );
    let service_registry_mutation_boundary_source_evidence =
        module_loader_service_registry_mutation_boundary_source_evidence(
            manifest.as_ref().map(|(event_id, _)| *event_id),
            artifact.as_ref().map(|(event_id, _)| *event_id),
            vm_report.as_ref().map(|(event_id, _)| *event_id),
            local_attestation.as_ref().map(|(event_id, _)| *event_id),
            local_approval.as_ref().map(|(event_id, _)| *event_id),
            computed_grant.as_ref().map(|(event_id, _)| *event_id),
            audit_rollback.as_ref().map(|(event_id, _)| *event_id),
            service_slot.as_ref().map(|(event_id, _)| *event_id),
            service_slot
                .as_ref()
                .map(|(_, reservation)| reservation.ram_only_service_slot_id),
            (
                execution_authorization_boundary_source_evidence_event_id,
                execution_authorization_boundary_source_evidence,
            ),
            registry_write_commit_gate_source_evidence,
            service_slot_source_evidence,
        );
    let service_registry_mutation_boundary_source_evidence_event_id =
        event_log::record_module_loader_service_registry_mutation_boundary_source_evidence(
            service_registry_mutation_boundary_source_evidence,
        );
    let load_attempt_boundary_source_evidence = module_loader_load_attempt_boundary_source_evidence(
        manifest.as_ref().map(|(event_id, _)| *event_id),
        artifact.as_ref().map(|(event_id, _)| *event_id),
        vm_report.as_ref().map(|(event_id, _)| *event_id),
        local_attestation.as_ref().map(|(event_id, _)| *event_id),
        local_approval.as_ref().map(|(event_id, _)| *event_id),
        computed_grant.as_ref().map(|(event_id, _)| *event_id),
        audit_rollback.as_ref().map(|(event_id, _)| *event_id),
        service_slot.as_ref().map(|(event_id, _)| *event_id),
        service_slot
            .as_ref()
            .map(|(_, reservation)| reservation.ram_only_service_slot_id),
        (
            artifact_byte_intake_boundary_source_evidence_event_id,
            artifact_byte_intake_boundary_source_evidence,
        ),
        (
            execution_authorization_boundary_source_evidence_event_id,
            execution_authorization_boundary_source_evidence,
        ),
        (
            service_registry_mutation_boundary_source_evidence_event_id,
            service_registry_mutation_boundary_source_evidence,
        ),
        write_boundary_source_evidence,
    );
    let load_attempt_boundary_source_evidence_event_id =
        event_log::record_module_loader_load_attempt_boundary_source_evidence(
            load_attempt_boundary_source_evidence,
        );
    let artifact_load_boundary_source_evidence =
        module_loader_artifact_load_boundary_source_evidence(
            manifest.as_ref().map(|(event_id, _)| *event_id),
            artifact.as_ref().map(|(event_id, _)| *event_id),
            vm_report.as_ref().map(|(event_id, _)| *event_id),
            local_attestation.as_ref().map(|(event_id, _)| *event_id),
            local_approval.as_ref().map(|(event_id, _)| *event_id),
            computed_grant.as_ref().map(|(event_id, _)| *event_id),
            audit_rollback.as_ref().map(|(event_id, _)| *event_id),
            service_slot.as_ref().map(|(event_id, _)| *event_id),
            service_slot
                .as_ref()
                .map(|(_, reservation)| reservation.ram_only_service_slot_id),
            (
                load_attempt_boundary_source_evidence_event_id,
                load_attempt_boundary_source_evidence,
            ),
            (
                artifact_byte_intake_boundary_source_evidence_event_id,
                artifact_byte_intake_boundary_source_evidence,
            ),
            artifact_hash_binding_source_evidence,
        );
    let artifact_load_boundary_source_evidence_event_id =
        event_log::record_module_loader_artifact_load_boundary_source_evidence(
            artifact_load_boundary_source_evidence,
        );
    let executable_mapping_boundary_source_evidence =
        module_loader_executable_mapping_boundary_source_evidence(
            manifest.as_ref().map(|(event_id, _)| *event_id),
            artifact.as_ref().map(|(event_id, _)| *event_id),
            vm_report.as_ref().map(|(event_id, _)| *event_id),
            local_attestation.as_ref().map(|(event_id, _)| *event_id),
            local_approval.as_ref().map(|(event_id, _)| *event_id),
            computed_grant.as_ref().map(|(event_id, _)| *event_id),
            audit_rollback.as_ref().map(|(event_id, _)| *event_id),
            service_slot.as_ref().map(|(event_id, _)| *event_id),
            service_slot
                .as_ref()
                .map(|(_, reservation)| reservation.ram_only_service_slot_id),
            (
                artifact_load_boundary_source_evidence_event_id,
                artifact_load_boundary_source_evidence,
            ),
            (
                execution_authorization_boundary_source_evidence_event_id,
                execution_authorization_boundary_source_evidence,
            ),
            address_space_source_evidence,
            memory_map_source_evidence,
        );
    let executable_mapping_boundary_source_evidence_event_id =
        event_log::record_module_loader_executable_mapping_boundary_source_evidence(
            executable_mapping_boundary_source_evidence,
        );
    let entrypoint_transfer_boundary_source_evidence =
        module_loader_entrypoint_transfer_boundary_source_evidence(
            manifest.as_ref().map(|(event_id, _)| *event_id),
            artifact.as_ref().map(|(event_id, _)| *event_id),
            vm_report.as_ref().map(|(event_id, _)| *event_id),
            local_attestation.as_ref().map(|(event_id, _)| *event_id),
            local_approval.as_ref().map(|(event_id, _)| *event_id),
            computed_grant.as_ref().map(|(event_id, _)| *event_id),
            audit_rollback.as_ref().map(|(event_id, _)| *event_id),
            service_slot.as_ref().map(|(event_id, _)| *event_id),
            service_slot
                .as_ref()
                .map(|(_, reservation)| reservation.ram_only_service_slot_id),
            (
                executable_mapping_boundary_source_evidence_event_id,
                executable_mapping_boundary_source_evidence,
            ),
            (
                execution_authorization_boundary_source_evidence_event_id,
                execution_authorization_boundary_source_evidence,
            ),
            entrypoint_abi_source_evidence,
            capability_table_source_evidence,
        );
    let entrypoint_transfer_boundary_source_evidence_event_id =
        event_log::record_module_loader_entrypoint_transfer_boundary_source_evidence(
            entrypoint_transfer_boundary_source_evidence,
        );
    let service_start_boundary_source_evidence =
        module_loader_service_start_boundary_source_evidence(
            manifest.as_ref().map(|(event_id, _)| *event_id),
            artifact.as_ref().map(|(event_id, _)| *event_id),
            vm_report.as_ref().map(|(event_id, _)| *event_id),
            local_attestation.as_ref().map(|(event_id, _)| *event_id),
            local_approval.as_ref().map(|(event_id, _)| *event_id),
            computed_grant.as_ref().map(|(event_id, _)| *event_id),
            audit_rollback.as_ref().map(|(event_id, _)| *event_id),
            service_slot.as_ref().map(|(event_id, _)| *event_id),
            service_slot
                .as_ref()
                .map(|(_, reservation)| reservation.ram_only_service_slot_id),
            (
                entrypoint_transfer_boundary_source_evidence_event_id,
                entrypoint_transfer_boundary_source_evidence,
            ),
            (
                service_registry_mutation_boundary_source_evidence_event_id,
                service_registry_mutation_boundary_source_evidence,
            ),
            service_slot_source_evidence,
            health_source_evidence,
            write_boundary_source_evidence,
        );
    let service_start_boundary_source_evidence_event_id =
        event_log::record_module_loader_service_start_boundary_source_evidence(
            service_start_boundary_source_evidence,
        );
    let service_health_binding_boundary_source_evidence =
        module_loader_service_health_binding_boundary_source_evidence(
            (
                service_start_boundary_source_evidence_event_id,
                service_start_boundary_source_evidence,
            ),
            service_slot_source_evidence,
            health_source_evidence,
            write_boundary_source_evidence,
        );
    let service_health_binding_boundary_source_evidence_event_id =
        event_log::record_module_loader_service_health_binding_boundary_source_evidence(
            service_health_binding_boundary_source_evidence,
        );
    let service_running_state_boundary_source_evidence =
        module_loader_service_running_state_boundary_source_evidence(
            (
                service_health_binding_boundary_source_evidence_event_id,
                service_health_binding_boundary_source_evidence,
            ),
            service_slot_source_evidence,
            health_source_evidence,
            write_boundary_source_evidence,
        );
    let service_running_state_boundary_source_evidence_event_id =
        event_log::record_module_loader_service_running_state_boundary_source_evidence(
            service_running_state_boundary_source_evidence,
        );
    let service_start_audit_boundary_source_evidence =
        module_loader_service_start_audit_boundary_source_evidence(
            (
                service_running_state_boundary_source_evidence_event_id,
                service_running_state_boundary_source_evidence,
            ),
            write_boundary_source_evidence,
        );
    let service_start_audit_boundary_source_evidence_event_id =
        event_log::record_module_loader_service_start_audit_boundary_source_evidence(
            service_start_audit_boundary_source_evidence,
        );
    let service_unload_cleanup_boundary_source_evidence =
        module_loader_service_unload_cleanup_boundary_source_evidence(
            (
                service_start_audit_boundary_source_evidence_event_id,
                service_start_audit_boundary_source_evidence,
            ),
            service_slot_source_evidence,
            rollback_source_evidence,
            write_boundary_source_evidence,
        );
    let service_unload_cleanup_boundary_source_evidence_event_id =
        event_log::record_module_loader_service_unload_cleanup_boundary_source_evidence(
            service_unload_cleanup_boundary_source_evidence,
        );
    let live_load_commit_boundary_source_evidence =
        module_loader_live_load_commit_boundary_source_evidence(
            (
                service_unload_cleanup_boundary_source_evidence_event_id,
                service_unload_cleanup_boundary_source_evidence,
            ),
            service_slot_source_evidence,
            rollback_source_evidence,
            write_boundary_source_evidence,
        );
    let live_load_commit_boundary_source_evidence_event_id =
        event_log::record_module_loader_live_load_commit_boundary_source_evidence(
            live_load_commit_boundary_source_evidence,
        );
    let commit_audit_boundary_source_evidence = module_loader_commit_audit_boundary_source_evidence(
        (
            live_load_commit_boundary_source_evidence_event_id,
            live_load_commit_boundary_source_evidence,
        ),
        write_boundary_source_evidence,
    );
    let commit_audit_boundary_source_evidence_event_id =
        event_log::record_module_loader_commit_audit_boundary_source_evidence(
            commit_audit_boundary_source_evidence,
        );
    let commit_rollback_boundary_source_evidence =
        module_loader_commit_rollback_boundary_source_evidence(
            (
                commit_audit_boundary_source_evidence_event_id,
                commit_audit_boundary_source_evidence,
            ),
            rollback_source_evidence,
            write_boundary_source_evidence,
        );
    let commit_rollback_boundary_source_evidence_event_id =
        event_log::record_module_loader_commit_rollback_boundary_source_evidence(
            commit_rollback_boundary_source_evidence,
        );
    let commit_result_boundary_source_evidence =
        module_loader_commit_result_boundary_source_evidence((
            commit_rollback_boundary_source_evidence_event_id,
            commit_rollback_boundary_source_evidence,
        ));
    let commit_result_boundary_source_evidence_event_id =
        event_log::record_module_loader_commit_result_boundary_source_evidence(
            commit_result_boundary_source_evidence,
        );
    let descriptor_acceptance_authority_boundary_source_evidence =
        module_loader_descriptor_acceptance_authority_boundary_source_evidence((
            commit_result_boundary_source_evidence_event_id,
            commit_result_boundary_source_evidence,
        ));
    let descriptor_acceptance_authority_boundary_source_evidence_event_id =
        event_log::record_module_loader_descriptor_acceptance_authority_boundary_source_evidence(
            descriptor_acceptance_authority_boundary_source_evidence,
        );
    let descriptor_parser_contract_boundary_source_evidence =
        module_loader_descriptor_parser_contract_boundary_source_evidence((
            descriptor_acceptance_authority_boundary_source_evidence_event_id,
            descriptor_acceptance_authority_boundary_source_evidence,
        ));
    let descriptor_parser_contract_boundary_source_evidence_event_id =
        event_log::record_module_loader_descriptor_parser_contract_boundary_source_evidence(
            descriptor_parser_contract_boundary_source_evidence,
        );
    let descriptor_parser_result_boundary_source_evidence =
        module_loader_descriptor_parser_result_boundary_source_evidence((
            descriptor_parser_contract_boundary_source_evidence_event_id,
            descriptor_parser_contract_boundary_source_evidence,
        ));
    let descriptor_parser_result_boundary_source_evidence_event_id =
        event_log::record_module_loader_descriptor_parser_result_boundary_source_evidence(
            descriptor_parser_result_boundary_source_evidence,
        );
    let descriptor_schema_validation_boundary_source_evidence =
        module_loader_descriptor_schema_validation_boundary_source_evidence((
            descriptor_parser_result_boundary_source_evidence_event_id,
            descriptor_parser_result_boundary_source_evidence,
        ));
    let descriptor_schema_validation_boundary_source_evidence_event_id =
        event_log::record_module_loader_descriptor_schema_validation_boundary_source_evidence(
            descriptor_schema_validation_boundary_source_evidence,
        );
    let descriptor_capability_validation_boundary_source_evidence =
        module_loader_descriptor_capability_validation_boundary_source_evidence((
            descriptor_schema_validation_boundary_source_evidence_event_id,
            descriptor_schema_validation_boundary_source_evidence,
        ));
    let descriptor_capability_validation_boundary_source_evidence_event_id =
        event_log::record_module_loader_descriptor_capability_validation_boundary_source_evidence(
            descriptor_capability_validation_boundary_source_evidence,
        );
    let descriptor_load_plan_boundary_source_evidence =
        module_loader_descriptor_load_plan_boundary_source_evidence((
            descriptor_capability_validation_boundary_source_evidence_event_id,
            descriptor_capability_validation_boundary_source_evidence,
        ));
    let descriptor_load_plan_boundary_source_evidence_event_id =
        event_log::record_module_loader_descriptor_load_plan_boundary_source_evidence(
            descriptor_load_plan_boundary_source_evidence,
        );
    let executable_load_plan_authority_boundary_source_evidence =
        module_loader_executable_load_plan_authority_boundary_source_evidence((
            descriptor_load_plan_boundary_source_evidence_event_id,
            descriptor_load_plan_boundary_source_evidence,
        ));
    let executable_load_plan_authority_boundary_source_evidence_event_id =
        event_log::record_module_loader_executable_load_plan_authority_boundary_source_evidence(
            executable_load_plan_authority_boundary_source_evidence,
        );
    let executable_load_plan_result_boundary_source_evidence =
        module_loader_executable_load_plan_result_boundary_source_evidence((
            executable_load_plan_authority_boundary_source_evidence_event_id,
            executable_load_plan_authority_boundary_source_evidence,
        ));
    let executable_load_plan_result_boundary_source_evidence_event_id =
        event_log::record_module_loader_executable_load_plan_result_boundary_source_evidence(
            executable_load_plan_result_boundary_source_evidence,
        );
    let executable_image_layout_boundary_source_evidence =
        module_loader_executable_image_layout_boundary_source_evidence((
            executable_load_plan_result_boundary_source_evidence_event_id,
            executable_load_plan_result_boundary_source_evidence,
        ));
    let executable_image_layout_boundary_source_evidence_event_id =
        event_log::record_module_loader_executable_image_layout_boundary_source_evidence(
            executable_image_layout_boundary_source_evidence,
        );
    let executable_page_mapping_plan_boundary_source_evidence =
        module_loader_executable_page_mapping_plan_boundary_source_evidence((
            executable_image_layout_boundary_source_evidence_event_id,
            executable_image_layout_boundary_source_evidence,
        ));
    let executable_page_mapping_plan_boundary_source_evidence_event_id =
        event_log::record_module_loader_executable_page_mapping_plan_boundary_source_evidence(
            executable_page_mapping_plan_boundary_source_evidence,
        );
    let executable_page_mapping_boundary_source_evidence =
        module_loader_executable_page_mapping_boundary_source_evidence((
            executable_page_mapping_plan_boundary_source_evidence_event_id,
            executable_page_mapping_plan_boundary_source_evidence,
        ));
    let executable_page_mapping_boundary_source_evidence_event_id =
        event_log::record_module_loader_executable_page_mapping_boundary_source_evidence(
            executable_page_mapping_boundary_source_evidence,
        );
    let descriptor_executable_page_binding_boundary_source_evidence =
        module_loader_descriptor_executable_page_binding_boundary_source_evidence((
            executable_page_mapping_boundary_source_evidence_event_id,
            executable_page_mapping_boundary_source_evidence,
        ));
    let descriptor_executable_page_binding_boundary_source_evidence_event_id =
        event_log::record_module_loader_descriptor_executable_page_binding_boundary_source_evidence(
            descriptor_executable_page_binding_boundary_source_evidence,
        );
    let executable_entrypoint_binding_boundary_source_evidence =
        module_loader_executable_entrypoint_binding_boundary_source_evidence((
            descriptor_executable_page_binding_boundary_source_evidence_event_id,
            descriptor_executable_page_binding_boundary_source_evidence,
        ));
    let executable_entrypoint_binding_boundary_source_evidence_event_id =
        event_log::record_module_loader_executable_entrypoint_binding_boundary_source_evidence(
            executable_entrypoint_binding_boundary_source_evidence,
        );
    let executable_entrypoint_transfer_authorization_boundary_source_evidence =
        module_loader_executable_entrypoint_transfer_authorization_boundary_source_evidence((
            executable_entrypoint_binding_boundary_source_evidence_event_id,
            executable_entrypoint_binding_boundary_source_evidence,
        ));
    let executable_entrypoint_transfer_authorization_boundary_source_evidence_event_id =
        event_log::record_module_loader_executable_entrypoint_transfer_authorization_boundary_source_evidence(
            executable_entrypoint_transfer_authorization_boundary_source_evidence,
        );
    let executable_entrypoint_transfer_boundary_source_evidence =
        module_loader_executable_entrypoint_transfer_boundary_source_evidence((
            executable_entrypoint_transfer_authorization_boundary_source_evidence_event_id,
            executable_entrypoint_transfer_authorization_boundary_source_evidence,
        ));
    let executable_entrypoint_transfer_boundary_source_evidence_event_id =
        event_log::record_module_loader_executable_entrypoint_transfer_boundary_source_evidence(
            executable_entrypoint_transfer_boundary_source_evidence,
        );
    let executable_entrypoint_handoff_boundary_source_evidence =
        module_loader_executable_entrypoint_handoff_boundary_source_evidence((
            executable_entrypoint_transfer_boundary_source_evidence_event_id,
            executable_entrypoint_transfer_boundary_source_evidence,
        ));
    let executable_entrypoint_handoff_boundary_source_evidence_event_id =
        event_log::record_module_loader_executable_entrypoint_handoff_boundary_source_evidence(
            executable_entrypoint_handoff_boundary_source_evidence,
        );
    let executable_entrypoint_invocation_boundary_source_evidence =
        module_loader_executable_entrypoint_invocation_boundary_source_evidence((
            executable_entrypoint_handoff_boundary_source_evidence_event_id,
            executable_entrypoint_handoff_boundary_source_evidence,
        ));
    let executable_entrypoint_invocation_boundary_source_evidence_event_id =
        event_log::record_module_loader_executable_entrypoint_invocation_boundary_source_evidence(
            executable_entrypoint_invocation_boundary_source_evidence,
        );
    let candidate = module_loader_runtime_snapshot(
        manifest.is_some(),
        artifact.is_some(),
        vm_report.is_some(),
        local_attestation.is_some(),
        local_approval.is_some(),
        computed_grant.is_some(),
        audit_rollback.is_some(),
        service_slot.is_some(),
        service_slot.as_ref().map(|(event_id, _)| *event_id),
        loader_identity_source_evidence,
        artifact_hash_binding_source_evidence,
        entrypoint_abi_source_evidence,
        address_space_source_evidence,
        memory_map_source_evidence,
        capability_table_source_evidence,
        service_slot_source_evidence,
        health_source_evidence,
        rollback_source_evidence,
        write_boundary_source_evidence,
        Some((
            execution_commit_gate_source_evidence_event_id,
            execution_commit_gate_source_evidence,
        )),
        Some((
            descriptor_intake_boundary_source_evidence_event_id,
            descriptor_intake_boundary_source_evidence,
        )),
        Some((
            artifact_byte_intake_boundary_source_evidence_event_id,
            artifact_byte_intake_boundary_source_evidence,
        )),
        Some((
            execution_authorization_boundary_source_evidence_event_id,
            execution_authorization_boundary_source_evidence,
        )),
        Some((
            service_registry_mutation_boundary_source_evidence_event_id,
            service_registry_mutation_boundary_source_evidence,
        )),
        Some((
            load_attempt_boundary_source_evidence_event_id,
            load_attempt_boundary_source_evidence,
        )),
        Some((
            artifact_load_boundary_source_evidence_event_id,
            artifact_load_boundary_source_evidence,
        )),
        Some((
            executable_mapping_boundary_source_evidence_event_id,
            executable_mapping_boundary_source_evidence,
        )),
        Some((
            entrypoint_transfer_boundary_source_evidence_event_id,
            entrypoint_transfer_boundary_source_evidence,
        )),
        Some((
            service_start_boundary_source_evidence_event_id,
            service_start_boundary_source_evidence,
        )),
        Some((
            service_health_binding_boundary_source_evidence_event_id,
            service_health_binding_boundary_source_evidence,
        )),
        Some((
            service_running_state_boundary_source_evidence_event_id,
            service_running_state_boundary_source_evidence,
        )),
        Some((
            service_start_audit_boundary_source_evidence_event_id,
            service_start_audit_boundary_source_evidence,
        )),
        Some((
            service_unload_cleanup_boundary_source_evidence_event_id,
            service_unload_cleanup_boundary_source_evidence,
        )),
        Some((
            live_load_commit_boundary_source_evidence_event_id,
            live_load_commit_boundary_source_evidence,
        )),
        Some((
            commit_audit_boundary_source_evidence_event_id,
            commit_audit_boundary_source_evidence,
        )),
        Some((
            commit_rollback_boundary_source_evidence_event_id,
            commit_rollback_boundary_source_evidence,
        )),
        Some((
            commit_result_boundary_source_evidence_event_id,
            commit_result_boundary_source_evidence,
        )),
        Some((
            descriptor_acceptance_authority_boundary_source_evidence_event_id,
            descriptor_acceptance_authority_boundary_source_evidence,
        )),
        Some((
            descriptor_parser_contract_boundary_source_evidence_event_id,
            descriptor_parser_contract_boundary_source_evidence,
        )),
        Some((
            descriptor_parser_result_boundary_source_evidence_event_id,
            descriptor_parser_result_boundary_source_evidence,
        )),
        Some((
            descriptor_schema_validation_boundary_source_evidence_event_id,
            descriptor_schema_validation_boundary_source_evidence,
        )),
        Some((
            descriptor_capability_validation_boundary_source_evidence_event_id,
            descriptor_capability_validation_boundary_source_evidence,
        )),
        Some((
            descriptor_load_plan_boundary_source_evidence_event_id,
            descriptor_load_plan_boundary_source_evidence,
        )),
        Some((
            executable_load_plan_authority_boundary_source_evidence_event_id,
            executable_load_plan_authority_boundary_source_evidence,
        )),
        Some((
            executable_load_plan_result_boundary_source_evidence_event_id,
            executable_load_plan_result_boundary_source_evidence,
        )),
        Some((
            executable_image_layout_boundary_source_evidence_event_id,
            executable_image_layout_boundary_source_evidence,
        )),
        Some((
            executable_page_mapping_plan_boundary_source_evidence_event_id,
            executable_page_mapping_plan_boundary_source_evidence,
        )),
        Some((
            executable_page_mapping_boundary_source_evidence_event_id,
            executable_page_mapping_boundary_source_evidence,
        )),
        Some((
            descriptor_executable_page_binding_boundary_source_evidence_event_id,
            descriptor_executable_page_binding_boundary_source_evidence,
        )),
        Some((
            executable_entrypoint_binding_boundary_source_evidence_event_id,
            executable_entrypoint_binding_boundary_source_evidence,
        )),
        Some((
            executable_entrypoint_transfer_authorization_boundary_source_evidence_event_id,
            executable_entrypoint_transfer_authorization_boundary_source_evidence,
        )),
        Some((
            executable_entrypoint_transfer_boundary_source_evidence_event_id,
            executable_entrypoint_transfer_boundary_source_evidence,
        )),
        Some((
            executable_entrypoint_handoff_boundary_source_evidence_event_id,
            executable_entrypoint_handoff_boundary_source_evidence,
        )),
        Some((
            executable_entrypoint_invocation_boundary_source_evidence_event_id,
            executable_entrypoint_invocation_boundary_source_evidence,
        )),
    );
    let evaluation = evaluate_module_loader_runtime_candidate(candidate);

    begin_response("module.loader_runtime");
    emit_record_fields_trailing_comma(module_loader_runtime_header_fields(), 6);
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
    emit_module_loader_runtime_execution_commit_gate(candidate, evaluation);
    raw_line(",");
    emit_module_loader_descriptor_intake_boundary(candidate, evaluation);
    raw_line(",");
    emit_module_loader_artifact_byte_intake_boundary(candidate, evaluation);
    raw_line(",");
    emit_module_loader_execution_authorization_boundary(candidate, evaluation);
    raw_line(",");
    emit_module_loader_service_registry_mutation_boundary(candidate, evaluation);
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "load_attempt_boundary",
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SCHEMA,
        MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_ID,
        candidate.load_attempt_boundary,
        evaluation.load_attempt_boundary_status,
        evaluation.load_attempt_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "artifact_load_boundary",
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SCHEMA,
        MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_ID,
        candidate.artifact_load_boundary,
        evaluation.artifact_load_boundary_status,
        evaluation.artifact_load_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "executable_mapping_boundary",
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_ID,
        candidate.executable_mapping_boundary,
        evaluation.executable_mapping_boundary_status,
        evaluation.executable_mapping_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "entrypoint_transfer_boundary",
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SCHEMA,
        MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_ID,
        candidate.entrypoint_transfer_boundary,
        evaluation.entrypoint_transfer_boundary_status,
        evaluation.entrypoint_transfer_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "service_start_boundary",
        MODULE_LOADER_SERVICE_START_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_START_BOUNDARY_ID,
        candidate.service_start_boundary,
        evaluation.service_start_boundary_status,
        evaluation.service_start_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "service_health_binding_boundary",
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_ID,
        candidate.service_health_binding_boundary,
        evaluation.service_health_binding_boundary_status,
        evaluation.service_health_binding_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "service_running_state_boundary",
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_ID,
        candidate.service_running_state_boundary,
        evaluation.service_running_state_boundary_status,
        evaluation.service_running_state_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "service_start_audit_boundary",
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_ID,
        candidate.service_start_audit_boundary,
        evaluation.service_start_audit_boundary_status,
        evaluation.service_start_audit_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "service_unload_cleanup_boundary",
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_ID,
        candidate.service_unload_cleanup_boundary,
        evaluation.service_unload_cleanup_boundary_status,
        evaluation.service_unload_cleanup_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "live_load_commit_boundary",
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SCHEMA,
        MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_ID,
        candidate.live_load_commit_boundary,
        evaluation.live_load_commit_boundary_status,
        evaluation.live_load_commit_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "commit_audit_boundary",
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SCHEMA,
        MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_ID,
        candidate.commit_audit_boundary,
        evaluation.commit_audit_boundary_status,
        evaluation.commit_audit_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "commit_rollback_boundary",
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SCHEMA,
        MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_ID,
        candidate.commit_rollback_boundary,
        evaluation.commit_rollback_boundary_status,
        evaluation.commit_rollback_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "commit_result_boundary",
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SCHEMA,
        MODULE_LOADER_COMMIT_RESULT_BOUNDARY_ID,
        candidate.commit_result_boundary,
        evaluation.commit_result_boundary_status,
        evaluation.commit_result_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "descriptor_acceptance_authority_boundary",
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_ID,
        candidate.descriptor_acceptance_authority_boundary,
        evaluation.descriptor_acceptance_authority_boundary_status,
        evaluation.descriptor_acceptance_authority_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "descriptor_parser_contract_boundary",
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_ID,
        candidate.descriptor_parser_contract_boundary,
        evaluation.descriptor_parser_contract_boundary_status,
        evaluation.descriptor_parser_contract_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "descriptor_parser_result_boundary",
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_ID,
        candidate.descriptor_parser_result_boundary,
        evaluation.descriptor_parser_result_boundary_status,
        evaluation.descriptor_parser_result_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "descriptor_schema_validation_boundary",
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_ID,
        candidate.descriptor_schema_validation_boundary,
        evaluation.descriptor_schema_validation_boundary_status,
        evaluation.descriptor_schema_validation_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "descriptor_capability_validation_boundary",
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_ID,
        candidate.descriptor_capability_validation_boundary,
        evaluation.descriptor_capability_validation_boundary_status,
        evaluation.descriptor_capability_validation_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "descriptor_load_plan_boundary",
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_ID,
        candidate.descriptor_load_plan_boundary,
        evaluation.descriptor_load_plan_boundary_status,
        evaluation.descriptor_load_plan_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "executable_load_plan_authority_boundary",
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_ID,
        candidate.executable_load_plan_authority_boundary,
        evaluation.executable_load_plan_authority_boundary_status,
        evaluation.executable_load_plan_authority_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "executable_load_plan_result_boundary",
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_ID,
        candidate.executable_load_plan_result_boundary,
        evaluation.executable_load_plan_result_boundary_status,
        evaluation.executable_load_plan_result_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "executable_image_layout_boundary",
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_ID,
        candidate.executable_image_layout_boundary,
        evaluation.executable_image_layout_boundary_status,
        evaluation.executable_image_layout_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "executable_page_mapping_plan_boundary",
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_ID,
        candidate.executable_page_mapping_plan_boundary,
        evaluation.executable_page_mapping_plan_boundary_status,
        evaluation.executable_page_mapping_plan_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "executable_page_mapping_boundary",
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_ID,
        candidate.executable_page_mapping_boundary,
        evaluation.executable_page_mapping_boundary_status,
        evaluation.executable_page_mapping_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "descriptor_executable_page_binding_boundary",
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_ID,
        candidate.descriptor_executable_page_binding_boundary,
        evaluation.descriptor_executable_page_binding_boundary_status,
        evaluation.descriptor_executable_page_binding_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "executable_entrypoint_binding_boundary",
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_ID,
        candidate.executable_entrypoint_binding_boundary,
        evaluation.executable_entrypoint_binding_boundary_status,
        evaluation.executable_entrypoint_binding_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "executable_entrypoint_transfer_authorization_boundary",
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_ID,
        candidate.executable_entrypoint_transfer_authorization_boundary,
        evaluation.executable_entrypoint_transfer_authorization_boundary_status,
        evaluation.executable_entrypoint_transfer_authorization_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "executable_entrypoint_transfer_boundary",
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_ID,
        candidate.executable_entrypoint_transfer_boundary,
        evaluation.executable_entrypoint_transfer_boundary_status,
        evaluation.executable_entrypoint_transfer_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "executable_entrypoint_handoff_boundary",
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_ID,
        candidate.executable_entrypoint_handoff_boundary,
        evaluation.executable_entrypoint_handoff_boundary_status,
        evaluation.executable_entrypoint_handoff_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_live_load_boundary(
        "executable_entrypoint_invocation_boundary",
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_ID,
        candidate.executable_entrypoint_invocation_boundary,
        evaluation.executable_entrypoint_invocation_boundary_status,
        evaluation.executable_entrypoint_invocation_boundary_reason,
    );
    raw_line(",");
    emit_module_loader_runtime_facts(candidate, evaluation);
    raw_line(",");
    emit_live_granted_load_projection();
    raw_line(",");
    emit_record_property_line(
        "policy_result",
        module_loader_runtime_policy_result_fields(candidate, evaluation),
        true,
    );
    emit_record_value_property_line(
        "blocked_by",
        module_loader_runtime_blocked_by(evaluation),
        false,
    );
    end_response("module.loader_runtime");
}

fn emit_live_granted_load_projection() {
    let projection = granted_candidate_service::live_load_projection();
    emit_record_value_property_line(
        "live_granted_load_projection",
        granted_candidate_service::record_live_load_projection(projection),
        false,
    );
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

    let mut case_values = Vec::new();
    idx = 0;
    while idx < cases.len() {
        case_values.push(module_loader_runtime_selftest_case_value(&cases[idx]));
        idx += 1;
    }

    begin_response("module.loader_runtime_selftest");
    emit_record_fields(
        vec![
            f(
                "schema",
                s("raios.module_loader_runtime_readiness_selftest.v0"),
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
            f("can_load_now", no()),
            f("load_attempted", no()),
            f("case_count", V::U64(cases.len() as u64)),
            f("passed", b(passed)),
            f(
                "source_fact_count",
                V::U64(MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT as u64),
            ),
            f("source_fact_map_complete", b(source_fact_map_complete)),
            f("source_fact_map", module_loader_runtime_source_fact_map()),
            f("cases", V::Array(case_values)),
            f("can_load", no()),
        ],
        6,
    );
    end_response("module.loader_runtime_selftest");
}
