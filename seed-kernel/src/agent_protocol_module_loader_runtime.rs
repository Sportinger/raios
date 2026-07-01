use crate::{
    agent_protocol_module_service_slot_allocator_projection::latest_module_service_slot_allocator_readiness_projection,
    agent_protocol_module_types::*, agent_protocol_support::*, event_log,
};

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
    );
    let evaluation = evaluate_module_loader_runtime_candidate(candidate);

    begin_response("module.loader_runtime");
    raw_line("      \"schema\": \"raios.module_loader_runtime_readiness.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": true,");
    raw_line(
        "      \"global_event_log_mutation\": \"retained_current_boot_source_evidence_only\",",
    );
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_descriptor_bytes\": false,");
    raw_line("      \"produces_parsed_descriptor\": false,");
    raw_line("      \"validates_descriptor_schema\": false,");
    raw_line("      \"produces_validated_descriptor\": false,");
    raw_line("      \"validates_descriptor_capabilities\": false,");
    raw_line("      \"produces_capability_validated_descriptor\": false,");
    raw_line("      \"authorizes_executable_load_plan\": false,");
    raw_line("      \"produces_executable_load_plan\": false,");
    raw_line("      \"produces_executable_image_layout\": false,");
    raw_line("      \"produces_executable_page_mapping_plan\": false,");
    raw_line("      \"maps_executable_pages\": false,");
    raw_line("      \"binds_capability_validated_descriptor_to_executable_pages\": false,");
    raw_line("      \"parses_descriptor_bytes\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_artifact\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"starts_service\": false,");
    raw_line("      \"marks_service_running\": false,");
    raw_line("      \"creates_service_health_records\": false,");
    raw_line("      \"writes_service_start_audit_record\": false,");
    raw_line("      \"unloads_service\": false,");
    raw_line("      \"cleans_up_service_slot\": false,");
    raw_line("      \"commits_live_load\": false,");
    raw_line("      \"writes_load_commit_audit_record\": false,");
    raw_line("      \"installs_commit_rollback_record\": false,");
    raw_line("      \"records_load_result\": false,");
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
    raw("        \"execution_commit_gate_status\": ");
    json_str(evaluation.execution_commit_gate_status);
    raw_line(",");
    raw("        \"execution_commit_gate_reason\": ");
    json_str(evaluation.execution_commit_gate_reason);
    raw_line(",");
    raw("        \"descriptor_intake_boundary_status\": ");
    json_str(evaluation.descriptor_intake_boundary_status);
    raw_line(",");
    raw("        \"descriptor_intake_boundary_reason\": ");
    json_str(evaluation.descriptor_intake_boundary_reason);
    raw_line(",");
    raw("        \"artifact_byte_intake_boundary_status\": ");
    json_str(evaluation.artifact_byte_intake_boundary_status);
    raw_line(",");
    raw("        \"artifact_byte_intake_boundary_reason\": ");
    json_str(evaluation.artifact_byte_intake_boundary_reason);
    raw_line(",");
    raw("        \"execution_authorization_boundary_status\": ");
    json_str(evaluation.execution_authorization_boundary_status);
    raw_line(",");
    raw("        \"execution_authorization_boundary_reason\": ");
    json_str(evaluation.execution_authorization_boundary_reason);
    raw_line(",");
    raw("        \"service_registry_mutation_boundary_status\": ");
    json_str(evaluation.service_registry_mutation_boundary_status);
    raw_line(",");
    raw("        \"service_registry_mutation_boundary_reason\": ");
    json_str(evaluation.service_registry_mutation_boundary_reason);
    raw_line(",");
    raw("        \"load_attempt_boundary_status\": ");
    json_str(evaluation.load_attempt_boundary_status);
    raw_line(",");
    raw("        \"load_attempt_boundary_reason\": ");
    json_str(evaluation.load_attempt_boundary_reason);
    raw_line(",");
    raw("        \"artifact_load_boundary_status\": ");
    json_str(evaluation.artifact_load_boundary_status);
    raw_line(",");
    raw("        \"artifact_load_boundary_reason\": ");
    json_str(evaluation.artifact_load_boundary_reason);
    raw_line(",");
    raw("        \"executable_mapping_boundary_status\": ");
    json_str(evaluation.executable_mapping_boundary_status);
    raw_line(",");
    raw("        \"executable_mapping_boundary_reason\": ");
    json_str(evaluation.executable_mapping_boundary_reason);
    raw_line(",");
    raw("        \"entrypoint_transfer_boundary_status\": ");
    json_str(evaluation.entrypoint_transfer_boundary_status);
    raw_line(",");
    raw("        \"entrypoint_transfer_boundary_reason\": ");
    json_str(evaluation.entrypoint_transfer_boundary_reason);
    raw_line(",");
    raw("        \"service_start_boundary_status\": ");
    json_str(evaluation.service_start_boundary_status);
    raw_line(",");
    raw("        \"service_start_boundary_reason\": ");
    json_str(evaluation.service_start_boundary_reason);
    raw_line(",");
    raw("        \"service_health_binding_boundary_status\": ");
    json_str(evaluation.service_health_binding_boundary_status);
    raw_line(",");
    raw("        \"service_health_binding_boundary_reason\": ");
    json_str(evaluation.service_health_binding_boundary_reason);
    raw_line(",");
    raw("        \"service_running_state_boundary_status\": ");
    json_str(evaluation.service_running_state_boundary_status);
    raw_line(",");
    raw("        \"service_running_state_boundary_reason\": ");
    json_str(evaluation.service_running_state_boundary_reason);
    raw_line(",");
    raw("        \"service_start_audit_boundary_status\": ");
    json_str(evaluation.service_start_audit_boundary_status);
    raw_line(",");
    raw("        \"service_start_audit_boundary_reason\": ");
    json_str(evaluation.service_start_audit_boundary_reason);
    raw_line(",");
    raw("        \"service_unload_cleanup_boundary_status\": ");
    json_str(evaluation.service_unload_cleanup_boundary_status);
    raw_line(",");
    raw("        \"service_unload_cleanup_boundary_reason\": ");
    json_str(evaluation.service_unload_cleanup_boundary_reason);
    raw_line(",");
    raw("        \"live_load_commit_boundary_status\": ");
    json_str(evaluation.live_load_commit_boundary_status);
    raw_line(",");
    raw("        \"live_load_commit_boundary_reason\": ");
    json_str(evaluation.live_load_commit_boundary_reason);
    raw_line(",");
    raw("        \"commit_audit_boundary_status\": ");
    json_str(evaluation.commit_audit_boundary_status);
    raw_line(",");
    raw("        \"commit_audit_boundary_reason\": ");
    json_str(evaluation.commit_audit_boundary_reason);
    raw_line(",");
    raw("        \"commit_rollback_boundary_status\": ");
    json_str(evaluation.commit_rollback_boundary_status);
    raw_line(",");
    raw("        \"commit_rollback_boundary_reason\": ");
    json_str(evaluation.commit_rollback_boundary_reason);
    raw_line(",");
    raw("        \"commit_result_boundary_status\": ");
    json_str(evaluation.commit_result_boundary_status);
    raw_line(",");
    raw("        \"commit_result_boundary_reason\": ");
    json_str(evaluation.commit_result_boundary_reason);
    raw_line(",");
    raw("        \"descriptor_acceptance_authority_boundary_status\": ");
    json_str(evaluation.descriptor_acceptance_authority_boundary_status);
    raw_line(",");
    raw("        \"descriptor_acceptance_authority_boundary_reason\": ");
    json_str(evaluation.descriptor_acceptance_authority_boundary_reason);
    raw_line(",");
    raw("        \"descriptor_parser_contract_boundary_status\": ");
    json_str(evaluation.descriptor_parser_contract_boundary_status);
    raw_line(",");
    raw("        \"descriptor_parser_contract_boundary_reason\": ");
    json_str(evaluation.descriptor_parser_contract_boundary_reason);
    raw_line(",");
    raw("        \"descriptor_parser_result_boundary_status\": ");
    json_str(evaluation.descriptor_parser_result_boundary_status);
    raw_line(",");
    raw("        \"descriptor_parser_result_boundary_reason\": ");
    json_str(evaluation.descriptor_parser_result_boundary_reason);
    raw_line(",");
    raw("        \"descriptor_schema_validation_boundary_status\": ");
    json_str(evaluation.descriptor_schema_validation_boundary_status);
    raw_line(",");
    raw("        \"descriptor_schema_validation_boundary_reason\": ");
    json_str(evaluation.descriptor_schema_validation_boundary_reason);
    raw_line(",");
    raw("        \"descriptor_capability_validation_boundary_status\": ");
    json_str(evaluation.descriptor_capability_validation_boundary_status);
    raw_line(",");
    raw("        \"descriptor_capability_validation_boundary_reason\": ");
    json_str(evaluation.descriptor_capability_validation_boundary_reason);
    raw_line(",");
    raw("        \"descriptor_load_plan_boundary_status\": ");
    json_str(evaluation.descriptor_load_plan_boundary_status);
    raw_line(",");
    raw("        \"descriptor_load_plan_boundary_reason\": ");
    json_str(evaluation.descriptor_load_plan_boundary_reason);
    raw_line(",");
    raw("        \"executable_load_plan_authority_boundary_status\": ");
    json_str(evaluation.executable_load_plan_authority_boundary_status);
    raw_line(",");
    raw("        \"executable_load_plan_authority_boundary_reason\": ");
    json_str(evaluation.executable_load_plan_authority_boundary_reason);
    raw_line(",");
    raw("        \"executable_load_plan_result_boundary_status\": ");
    json_str(evaluation.executable_load_plan_result_boundary_status);
    raw_line(",");
    raw("        \"executable_load_plan_result_boundary_reason\": ");
    json_str(evaluation.executable_load_plan_result_boundary_reason);
    raw_line(",");
    raw("        \"executable_image_layout_boundary_status\": ");
    json_str(evaluation.executable_image_layout_boundary_status);
    raw_line(",");
    raw("        \"executable_image_layout_boundary_reason\": ");
    json_str(evaluation.executable_image_layout_boundary_reason);
    raw_line(",");
    raw("        \"executable_page_mapping_plan_boundary_status\": ");
    json_str(evaluation.executable_page_mapping_plan_boundary_status);
    raw_line(",");
    raw("        \"executable_page_mapping_plan_boundary_reason\": ");
    json_str(evaluation.executable_page_mapping_plan_boundary_reason);
    raw_line(",");
    raw("        \"executable_page_mapping_boundary_status\": ");
    json_str(evaluation.executable_page_mapping_boundary_status);
    raw_line(",");
    raw("        \"executable_page_mapping_boundary_reason\": ");
    json_str(evaluation.executable_page_mapping_boundary_reason);
    raw_line(",");
    raw("        \"descriptor_executable_page_binding_boundary_status\": ");
    json_str(evaluation.descriptor_executable_page_binding_boundary_status);
    raw_line(",");
    raw("        \"descriptor_executable_page_binding_boundary_reason\": ");
    json_str(evaluation.descriptor_executable_page_binding_boundary_reason);
    raw_line(",");
    raw("        \"executable_entrypoint_binding_boundary_status\": ");
    json_str(evaluation.executable_entrypoint_binding_boundary_status);
    raw_line(",");
    raw("        \"executable_entrypoint_binding_boundary_reason\": ");
    json_str(evaluation.executable_entrypoint_binding_boundary_reason);
    raw_line(",");
    raw("        \"executable_entrypoint_transfer_authorization_boundary_status\": ");
    json_str(evaluation.executable_entrypoint_transfer_authorization_boundary_status);
    raw_line(",");
    raw("        \"executable_entrypoint_transfer_authorization_boundary_reason\": ");
    json_str(evaluation.executable_entrypoint_transfer_authorization_boundary_reason);
    raw_line(",");
    raw("        \"executable_entrypoint_transfer_boundary_status\": ");
    json_str(evaluation.executable_entrypoint_transfer_boundary_status);
    raw_line(",");
    raw("        \"executable_entrypoint_transfer_boundary_reason\": ");
    json_str(evaluation.executable_entrypoint_transfer_boundary_reason);
    raw_line(",");
    raw("        \"executable_entrypoint_handoff_boundary_status\": ");
    json_str(evaluation.executable_entrypoint_handoff_boundary_status);
    raw_line(",");
    raw("        \"executable_entrypoint_handoff_boundary_reason\": ");
    json_str(evaluation.executable_entrypoint_handoff_boundary_reason);
    raw_line(",");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"starts_service\": false,");
    raw_line("        \"marks_service_running\": false,");
    raw_line("        \"creates_service_health_records\": false,");
    raw_line("        \"writes_service_start_audit_record\": false,");
    raw_line("        \"unloads_service\": false,");
    raw_line("        \"cleans_up_service_slot\": false,");
    raw_line("        \"commits_live_load\": false,");
    raw_line("        \"writes_load_commit_audit_record\": false,");
    raw_line("        \"installs_commit_rollback_record\": false,");
    raw_line("        \"records_load_result\": false,");
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
    emit_module_loader_runtime_gate(
        &mut wrote,
        "execution_commit_gate",
        evaluation.execution_commit_gate_status,
        evaluation.execution_commit_gate_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "descriptor_intake_boundary",
        evaluation.descriptor_intake_boundary_status,
        evaluation.descriptor_intake_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "artifact_byte_intake_boundary",
        evaluation.artifact_byte_intake_boundary_status,
        evaluation.artifact_byte_intake_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "execution_authorization_boundary",
        evaluation.execution_authorization_boundary_status,
        evaluation.execution_authorization_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "service_registry_mutation_boundary",
        evaluation.service_registry_mutation_boundary_status,
        evaluation.service_registry_mutation_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "load_attempt_boundary",
        evaluation.load_attempt_boundary_status,
        evaluation.load_attempt_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "artifact_load_boundary",
        evaluation.artifact_load_boundary_status,
        evaluation.artifact_load_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "executable_mapping_boundary",
        evaluation.executable_mapping_boundary_status,
        evaluation.executable_mapping_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "entrypoint_transfer_boundary",
        evaluation.entrypoint_transfer_boundary_status,
        evaluation.entrypoint_transfer_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "service_start_boundary",
        evaluation.service_start_boundary_status,
        evaluation.service_start_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "service_health_binding_boundary",
        evaluation.service_health_binding_boundary_status,
        evaluation.service_health_binding_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "service_running_state_boundary",
        evaluation.service_running_state_boundary_status,
        evaluation.service_running_state_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "service_start_audit_boundary",
        evaluation.service_start_audit_boundary_status,
        evaluation.service_start_audit_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "service_unload_cleanup_boundary",
        evaluation.service_unload_cleanup_boundary_status,
        evaluation.service_unload_cleanup_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "live_load_commit_boundary",
        evaluation.live_load_commit_boundary_status,
        evaluation.live_load_commit_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "commit_audit_boundary",
        evaluation.commit_audit_boundary_status,
        evaluation.commit_audit_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "commit_rollback_boundary",
        evaluation.commit_rollback_boundary_status,
        evaluation.commit_rollback_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "commit_result_boundary",
        evaluation.commit_result_boundary_status,
        evaluation.commit_result_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "descriptor_acceptance_authority_boundary",
        evaluation.descriptor_acceptance_authority_boundary_status,
        evaluation.descriptor_acceptance_authority_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "descriptor_parser_contract_boundary",
        evaluation.descriptor_parser_contract_boundary_status,
        evaluation.descriptor_parser_contract_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "descriptor_parser_result_boundary",
        evaluation.descriptor_parser_result_boundary_status,
        evaluation.descriptor_parser_result_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "descriptor_schema_validation_boundary",
        evaluation.descriptor_schema_validation_boundary_status,
        evaluation.descriptor_schema_validation_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "descriptor_capability_validation_boundary",
        evaluation.descriptor_capability_validation_boundary_status,
        evaluation.descriptor_capability_validation_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "descriptor_load_plan_boundary",
        evaluation.descriptor_load_plan_boundary_status,
        evaluation.descriptor_load_plan_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "executable_load_plan_authority_boundary",
        evaluation.executable_load_plan_authority_boundary_status,
        evaluation.executable_load_plan_authority_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "executable_load_plan_result_boundary",
        evaluation.executable_load_plan_result_boundary_status,
        evaluation.executable_load_plan_result_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "executable_image_layout_boundary",
        evaluation.executable_image_layout_boundary_status,
        evaluation.executable_image_layout_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "executable_page_mapping_plan_boundary",
        evaluation.executable_page_mapping_plan_boundary_status,
        evaluation.executable_page_mapping_plan_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "executable_page_mapping_boundary",
        evaluation.executable_page_mapping_boundary_status,
        evaluation.executable_page_mapping_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "descriptor_executable_page_binding_boundary",
        evaluation.descriptor_executable_page_binding_boundary_status,
        evaluation.descriptor_executable_page_binding_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "executable_entrypoint_binding_boundary",
        evaluation.executable_entrypoint_binding_boundary_status,
        evaluation.executable_entrypoint_binding_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "executable_entrypoint_transfer_authorization_boundary",
        evaluation.executable_entrypoint_transfer_authorization_boundary_status,
        evaluation.executable_entrypoint_transfer_authorization_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "executable_entrypoint_transfer_boundary",
        evaluation.executable_entrypoint_transfer_boundary_status,
        evaluation.executable_entrypoint_transfer_boundary_reason,
    );
    emit_module_loader_runtime_gate(
        &mut wrote,
        "executable_entrypoint_handoff_boundary",
        evaluation.executable_entrypoint_handoff_boundary_status,
        evaluation.executable_entrypoint_handoff_boundary_reason,
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
    let service_slot = event_log::latest_module_service_slot_reservation();
    let allocator_projection = latest_module_service_slot_allocator_readiness_projection(
        service_slot.as_ref().map(|(event_id, _)| *event_id),
    );
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
    raw_line("        \"allocator_authority_boundary\": {");
    raw("          \"schema\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SCHEMA);
    raw_line(",");
    raw("          \"source_evidence_event_id\": ");
    json_event_id_option(allocator_projection.authority_source_evidence_event_id);
    raw_line(",");
    raw("          \"status\": ");
    json_str(allocator_projection.authority_status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(allocator_projection.authority_reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(allocator_projection.authority_present);
    raw_line(",");
    raw_line("          \"allocates_service_slot\": false,");
    raw_line("          \"creates_service_inventory_records\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"load_attempted\": false");
    raw_line("        },");
    raw_line("        \"allocation_intent_boundary\": {");
    raw("          \"schema\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SCHEMA);
    raw_line(",");
    raw("          \"id\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATION_INTENT_ID);
    raw_line(",");
    raw("          \"source_evidence_event_id\": ");
    json_event_id_option(allocator_projection.allocation_intent_source_evidence_event_id);
    raw_line(",");
    raw("          \"status\": ");
    json_str(allocator_projection.allocation_intent_status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(allocator_projection.allocation_intent_reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(allocator_projection.allocation_intent_present);
    raw_line(",");
    raw_line("          \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("          \"load_mode\": \"ram_only\",");
    raw_line("          \"target\": \"live_service_graph\",");
    raw_line("          \"allocates_service_slot\": false,");
    raw_line("          \"creates_service_inventory_records\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"load_attempted\": false");
    raw_line("        },");
    raw_line("        \"authority_input_boundaries\": {");
    let mut input_idx = 0usize;
    while input_idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        let input = allocator_projection.authority_inputs[input_idx];
        raw("          ");
        json_str(input.name);
        raw_line(": {");
        raw("            \"schema\": ");
        json_str(input.schema);
        raw_line(",");
        raw("            \"source_evidence_event_id\": ");
        json_event_id_option(input.source_evidence_event_id);
        raw_line(",");
        raw("            \"status\": ");
        json_str(input.status);
        raw_line(",");
        raw("            \"reason\": ");
        json_str(input.reason);
        raw_line(",");
        raw("            \"present\": ");
        raw_bool(input.present);
        raw_line(",");
        raw_line("            \"allocates_service_slot\": false,");
        raw_line("            \"creates_service_inventory_records\": false,");
        raw_line("            \"service_inventory_change\": \"none\",");
        raw_line("            \"load_attempted\": false");
        raw("          }");
        if input_idx + 1 != MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
            raw(",");
        }
        crlf();
        input_idx += 1;
    }
    raw_line("        },");
    raw_line("        \"authority_decision\": {");
    raw("          \"schema\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SCHEMA);
    raw_line(",");
    raw("          \"id\": ");
    json_str(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_ID);
    raw_line(",");
    raw("          \"source_evidence_event_id\": ");
    json_event_id_option(allocator_projection.authority_decision_source_evidence_event_id);
    raw_line(",");
    raw("          \"status\": ");
    json_str(allocator_projection.authority_decision_status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(allocator_projection.authority_decision_reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(allocator_projection.authority_decision_present);
    raw_line(",");
    raw_line("          \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("          \"load_mode\": \"ram_only\",");
    raw_line("          \"target\": \"live_service_graph\",");
    raw_line("          \"authorizes_allocation\": false,");
    raw_line("          \"authorizes_load\": false,");
    raw_line("          \"allocates_service_slot\": false,");
    raw_line("          \"creates_service_inventory_records\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"load_attempted\": false");
    raw_line("        },");
    raw_line("        \"registry_write_commit_gate\": {");
    raw("          \"schema\": ");
    json_str(MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SCHEMA);
    raw_line(",");
    raw("          \"id\": ");
    json_str(MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_ID);
    raw_line(",");
    raw("          \"source_evidence_event_id\": ");
    json_event_id_option(allocator_projection.registry_write_commit_gate_source_evidence_event_id);
    raw_line(",");
    raw("          \"status\": ");
    json_str(allocator_projection.registry_write_commit_gate_status);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(allocator_projection.registry_write_commit_gate_reason);
    raw_line(",");
    raw("          \"present\": ");
    raw_bool(allocator_projection.registry_write_commit_gate_present);
    raw_line(",");
    raw_line("          \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("          \"load_mode\": \"ram_only\",");
    raw_line("          \"target\": \"live_service_graph\",");
    raw_line("          \"authorizes_registry_write\": false,");
    raw_line("          \"mutates_service_registry\": false,");
    raw_line("          \"writes_durable_audit_state\": false,");
    raw_line("          \"installs_rollback_state\": false,");
    raw_line("          \"authorizes_allocation\": false,");
    raw_line("          \"authorizes_load\": false,");
    raw_line("          \"allocates_service_slot\": false,");
    raw_line("          \"creates_service_inventory_records\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"loads_artifact\": false,");
    raw_line("          \"load_attempted\": false");
    raw_line("        },");
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

fn emit_module_loader_runtime_execution_commit_gate(
    candidate: ModuleLoaderRuntimeCandidate,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    let gate = candidate.execution_commit_gate;
    raw_line("      \"execution_commit_gate\": {");
    raw("        \"schema\": ");
    json_str(MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SCHEMA);
    raw_line(",");
    raw("        \"id\": ");
    json_str(MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_ID);
    raw_line(",");
    raw("        \"source_evidence_schema\": ");
    json_str(gate.source_evidence_schema);
    raw_line(",");
    raw("        \"source_evidence_event_id\": ");
    json_event_id_option(gate.source_evidence_event_id);
    raw_line(",");
    raw("        \"source_evidence_state\": ");
    json_str(gate.source_evidence_state);
    raw_line(",");
    raw("        \"source_evidence_status\": ");
    json_str(gate.source_evidence_status);
    raw_line(",");
    raw("        \"source_evidence_reason\": ");
    json_str(gate.source_evidence_reason);
    raw_line(",");
    raw("        \"source_method\": ");
    json_str(gate.source_evidence_method);
    raw_line(",");
    raw("        \"source_fact_locator\": ");
    json_str(gate.source_evidence_fact_locator);
    raw_line(",");
    raw("        \"status\": ");
    json_str(evaluation.execution_commit_gate_status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(evaluation.execution_commit_gate_reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(gate.present);
    raw_line(",");
    raw("        \"source_chain_complete\": ");
    raw_bool(gate.source_chain_complete);
    raw_line(",");
    raw("        \"authority_decision_present\": ");
    raw_bool(gate.authority_decision_present);
    raw_line(",");
    raw("        \"loader_runtime_contract_present\": ");
    raw_bool(gate.loader_runtime_contract_present);
    raw_line(",");
    raw("        \"loader_runtime_source_evidence_complete\": ");
    raw_bool(gate.loader_runtime_source_evidence_complete);
    raw_line(",");
    raw("        \"service_slot_binding_source_evidence_present\": ");
    raw_bool(gate.service_slot_binding_source_evidence_present);
    raw_line(",");
    raw("        \"service_slot_binding_fact_present\": ");
    raw_bool(gate.service_slot_binding_fact_present);
    raw_line(",");
    raw("        \"audit_rollback_write_boundary_source_evidence_present\": ");
    raw_bool(gate.audit_rollback_write_boundary_source_evidence_present);
    raw_line(",");
    raw("        \"audit_rollback_write_boundary_fact_present\": ");
    raw_bool(gate.audit_rollback_write_boundary_fact_present);
    raw_line(",");
    raw("        \"retained_service_slot_reservation_present\": ");
    raw_bool(gate.retained_service_slot_reservation_present);
    raw_line(",");
    raw_line("        \"loader_runtime_source_evidence\": [");
    let mut idx = 0usize;
    while idx < MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
        let source = MODULE_LOADER_RUNTIME_FACT_SOURCES[idx];
        raw("          {\"fact\": ");
        json_str(source.name);
        raw(", \"schema\": ");
        json_str(source.schema);
        raw(", \"source_evidence_event_id\": ");
        json_event_id_option(gate.loader_runtime_source_evidence_event_ids[idx]);
        raw(", \"source_evidence_present\": ");
        raw_bool(gate.loader_runtime_source_evidence_present[idx]);
        raw(", \"fact_present\": ");
        raw_bool(gate.loader_runtime_fact_present[idx]);
        raw("}");
        if idx + 1 != MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
            raw(",");
        }
        crlf();
        idx += 1;
    }
    raw_line("        ],");
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"target\": \"live_service_graph\",");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"authorizes_execution\": false,");
    raw_line("        \"mutates_service_registry\": false,");
    raw_line("        \"writes_durable_audit_state\": false,");
    raw_line("        \"installs_rollback_state\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
}

fn emit_module_loader_descriptor_intake_boundary(
    candidate: ModuleLoaderRuntimeCandidate,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    let boundary = candidate.descriptor_intake_boundary;
    raw_line("      \"descriptor_intake_boundary\": {");
    raw("        \"schema\": ");
    json_str(MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SCHEMA);
    raw_line(",");
    raw("        \"id\": ");
    json_str(MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_ID);
    raw_line(",");
    raw("        \"source_evidence_schema\": ");
    json_str(boundary.source_evidence_schema);
    raw_line(",");
    raw("        \"source_evidence_event_id\": ");
    json_event_id_option(boundary.source_evidence_event_id);
    raw_line(",");
    raw("        \"source_evidence_state\": ");
    json_str(boundary.source_evidence_state);
    raw_line(",");
    raw("        \"source_evidence_status\": ");
    json_str(boundary.source_evidence_status);
    raw_line(",");
    raw("        \"source_evidence_reason\": ");
    json_str(boundary.source_evidence_reason);
    raw_line(",");
    raw("        \"source_method\": ");
    json_str(boundary.source_evidence_method);
    raw_line(",");
    raw("        \"source_fact_locator\": ");
    json_str(boundary.source_evidence_fact_locator);
    raw_line(",");
    raw("        \"status\": ");
    json_str(evaluation.descriptor_intake_boundary_status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(evaluation.descriptor_intake_boundary_reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(boundary.present);
    raw_line(",");
    raw("        \"source_chain_complete\": ");
    raw_bool(boundary.source_chain_complete);
    raw_line(",");
    raw("        \"registry_write_commit_gate_present\": ");
    raw_bool(boundary.registry_write_commit_gate_present);
    raw_line(",");
    raw("        \"execution_commit_gate_present\": ");
    raw_bool(boundary.execution_commit_gate_present);
    raw_line(",");
    raw("        \"loader_runtime_source_evidence_complete\": ");
    raw_bool(boundary.loader_runtime_source_evidence_complete);
    raw_line(",");
    raw("        \"retained_module_evidence_present\": ");
    raw_bool(boundary.retained_module_evidence_present);
    raw_line(",");
    raw("        \"retained_service_slot_reservation_present\": ");
    raw_bool(boundary.retained_service_slot_reservation_present);
    raw_line(",");
    raw_line("        \"loader_runtime_source_evidence\": [");
    let mut idx = 0usize;
    while idx < MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
        let source = MODULE_LOADER_RUNTIME_FACT_SOURCES[idx];
        raw("          {\"fact\": ");
        json_str(source.name);
        raw(", \"schema\": ");
        json_str(source.schema);
        raw(", \"source_evidence_event_id\": ");
        json_event_id_option(boundary.loader_runtime_source_evidence_event_ids[idx]);
        raw(", \"source_evidence_present\": ");
        raw_bool(boundary.loader_runtime_source_evidence_present[idx]);
        raw(", \"fact_present\": ");
        raw_bool(boundary.loader_runtime_fact_present[idx]);
        raw("}");
        if idx + 1 != MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
            raw(",");
        }
        crlf();
        idx += 1;
    }
    raw_line("        ],");
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"target\": \"live_service_graph\",");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_descriptor_bytes\": false,");
    raw_line("        \"produces_parsed_descriptor\": false,");
    raw_line("        \"validates_descriptor_schema\": false,");
    raw_line("        \"produces_validated_descriptor\": false,");
    raw_line("        \"validates_descriptor_capabilities\": false,");
    raw_line("        \"produces_capability_validated_descriptor\": false,");
    raw_line("        \"authorizes_executable_load_plan\": false,");
    raw_line("        \"produces_executable_load_plan\": false,");
    raw_line("        \"produces_executable_image_layout\": false,");
    raw_line("        \"produces_executable_page_mapping_plan\": false,");
    raw_line("        \"binds_capability_validated_descriptor_to_executable_pages\": false,");
    raw_line("        \"parses_descriptor_bytes\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"authorizes_descriptor_intake\": false,");
    raw_line("        \"authorizes_execution\": false,");
    raw_line("        \"mutates_service_registry\": false,");
    raw_line("        \"writes_durable_audit_state\": false,");
    raw_line("        \"installs_rollback_state\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
}

fn emit_module_loader_artifact_byte_intake_boundary(
    candidate: ModuleLoaderRuntimeCandidate,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    let boundary = candidate.artifact_byte_intake_boundary;
    raw_line("      \"artifact_byte_intake_boundary\": {");
    raw("        \"schema\": ");
    json_str(MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SCHEMA);
    raw_line(",");
    raw("        \"id\": ");
    json_str(MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_ID);
    raw_line(",");
    raw("        \"source_evidence_schema\": ");
    json_str(boundary.source_evidence_schema);
    raw_line(",");
    raw("        \"source_evidence_event_id\": ");
    json_event_id_option(boundary.source_evidence_event_id);
    raw_line(",");
    raw("        \"source_evidence_state\": ");
    json_str(boundary.source_evidence_state);
    raw_line(",");
    raw("        \"source_evidence_status\": ");
    json_str(boundary.source_evidence_status);
    raw_line(",");
    raw("        \"source_evidence_reason\": ");
    json_str(boundary.source_evidence_reason);
    raw_line(",");
    raw("        \"source_method\": ");
    json_str(boundary.source_evidence_method);
    raw_line(",");
    raw("        \"source_fact_locator\": ");
    json_str(boundary.source_evidence_fact_locator);
    raw_line(",");
    raw("        \"status\": ");
    json_str(evaluation.artifact_byte_intake_boundary_status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(evaluation.artifact_byte_intake_boundary_reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(boundary.present);
    raw_line(",");
    raw("        \"source_chain_complete\": ");
    raw_bool(boundary.source_chain_complete);
    raw_line(",");
    raw("        \"descriptor_intake_boundary_present\": ");
    raw_bool(boundary.descriptor_intake_boundary_present);
    raw_line(",");
    raw("        \"descriptor_intake_boundary_source_chain_complete\": ");
    raw_bool(boundary.descriptor_intake_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"execution_commit_gate_present\": ");
    raw_bool(boundary.execution_commit_gate_present);
    raw_line(",");
    raw("        \"artifact_hash_binding_present\": ");
    raw_bool(boundary.artifact_hash_binding_present);
    raw_line(",");
    raw("        \"retained_artifact_reference_present\": ");
    raw_bool(boundary.retained_artifact_reference_present);
    raw_line(",");
    raw("        \"retained_module_evidence_present\": ");
    raw_bool(boundary.retained_module_evidence_present);
    raw_line(",");
    raw("        \"retained_service_slot_reservation_present\": ");
    raw_bool(boundary.retained_service_slot_reservation_present);
    raw_line(",");
    raw_line("        \"loader_runtime_source_evidence\": [");
    let mut idx = 0usize;
    while idx < MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
        let source = MODULE_LOADER_RUNTIME_FACT_SOURCES[idx];
        raw("          {\"fact\": ");
        json_str(source.name);
        raw(", \"schema\": ");
        json_str(source.schema);
        raw(", \"source_evidence_event_id\": ");
        json_event_id_option(boundary.loader_runtime_source_evidence_event_ids[idx]);
        raw(", \"source_evidence_present\": ");
        raw_bool(boundary.loader_runtime_source_evidence_present[idx]);
        raw(", \"fact_present\": ");
        raw_bool(boundary.loader_runtime_fact_present[idx]);
        raw("}");
        if idx + 1 != MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
            raw(",");
        }
        crlf();
        idx += 1;
    }
    raw_line("        ],");
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"target\": \"live_service_graph\",");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_descriptor_bytes\": false,");
    raw_line("        \"produces_parsed_descriptor\": false,");
    raw_line("        \"validates_descriptor_schema\": false,");
    raw_line("        \"produces_validated_descriptor\": false,");
    raw_line("        \"validates_descriptor_capabilities\": false,");
    raw_line("        \"produces_capability_validated_descriptor\": false,");
    raw_line("        \"authorizes_executable_load_plan\": false,");
    raw_line("        \"produces_executable_load_plan\": false,");
    raw_line("        \"produces_executable_image_layout\": false,");
    raw_line("        \"produces_executable_page_mapping_plan\": false,");
    raw_line("        \"binds_capability_validated_descriptor_to_executable_pages\": false,");
    raw_line("        \"parses_descriptor_bytes\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"authorizes_descriptor_intake\": false,");
    raw_line("        \"authorizes_artifact_byte_intake\": false,");
    raw_line("        \"authorizes_execution\": false,");
    raw_line("        \"mutates_service_registry\": false,");
    raw_line("        \"writes_durable_audit_state\": false,");
    raw_line("        \"installs_rollback_state\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
}

fn emit_module_loader_execution_authorization_boundary(
    candidate: ModuleLoaderRuntimeCandidate,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    let boundary = candidate.execution_authorization_boundary;
    raw_line("      \"execution_authorization_boundary\": {");
    raw("        \"schema\": ");
    json_str(MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SCHEMA);
    raw_line(",");
    raw("        \"id\": ");
    json_str(MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_ID);
    raw_line(",");
    raw("        \"source_evidence_schema\": ");
    json_str(boundary.source_evidence_schema);
    raw_line(",");
    raw("        \"source_evidence_event_id\": ");
    json_event_id_option(boundary.source_evidence_event_id);
    raw_line(",");
    raw("        \"source_evidence_state\": ");
    json_str(boundary.source_evidence_state);
    raw_line(",");
    raw("        \"source_evidence_status\": ");
    json_str(boundary.source_evidence_status);
    raw_line(",");
    raw("        \"source_evidence_reason\": ");
    json_str(boundary.source_evidence_reason);
    raw_line(",");
    raw("        \"source_method\": ");
    json_str(boundary.source_evidence_method);
    raw_line(",");
    raw("        \"source_fact_locator\": ");
    json_str(boundary.source_evidence_fact_locator);
    raw_line(",");
    raw("        \"status\": ");
    json_str(evaluation.execution_authorization_boundary_status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(evaluation.execution_authorization_boundary_reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(boundary.present);
    raw_line(",");
    raw("        \"source_chain_complete\": ");
    raw_bool(boundary.source_chain_complete);
    raw_line(",");
    raw("        \"artifact_byte_intake_boundary_present\": ");
    raw_bool(boundary.artifact_byte_intake_boundary_present);
    raw_line(",");
    raw("        \"artifact_byte_intake_boundary_source_chain_complete\": ");
    raw_bool(boundary.artifact_byte_intake_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"descriptor_intake_boundary_present\": ");
    raw_bool(boundary.descriptor_intake_boundary_present);
    raw_line(",");
    raw("        \"descriptor_intake_boundary_source_chain_complete\": ");
    raw_bool(boundary.descriptor_intake_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"execution_commit_gate_present\": ");
    raw_bool(boundary.execution_commit_gate_present);
    raw_line(",");
    raw("        \"entrypoint_abi_source_evidence_present\": ");
    raw_bool(boundary.entrypoint_abi_source_evidence_present);
    raw_line(",");
    raw("        \"address_space_source_evidence_present\": ");
    raw_bool(boundary.address_space_source_evidence_present);
    raw_line(",");
    raw("        \"memory_map_source_evidence_present\": ");
    raw_bool(boundary.memory_map_source_evidence_present);
    raw_line(",");
    raw("        \"audit_rollback_write_boundary_source_evidence_present\": ");
    raw_bool(boundary.audit_rollback_write_boundary_source_evidence_present);
    raw_line(",");
    raw("        \"retained_module_evidence_present\": ");
    raw_bool(boundary.retained_module_evidence_present);
    raw_line(",");
    raw("        \"retained_service_slot_reservation_present\": ");
    raw_bool(boundary.retained_service_slot_reservation_present);
    raw_line(",");
    raw_line("        \"loader_runtime_source_evidence\": [");
    let mut idx = 0usize;
    while idx < MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
        let source = MODULE_LOADER_RUNTIME_FACT_SOURCES[idx];
        raw("          {\"fact\": ");
        json_str(source.name);
        raw(", \"schema\": ");
        json_str(source.schema);
        raw(", \"source_evidence_event_id\": ");
        json_event_id_option(boundary.loader_runtime_source_evidence_event_ids[idx]);
        raw(", \"source_evidence_present\": ");
        raw_bool(boundary.loader_runtime_source_evidence_present[idx]);
        raw(", \"fact_present\": ");
        raw_bool(boundary.loader_runtime_fact_present[idx]);
        raw("}");
        if idx + 1 != MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
            raw(",");
        }
        crlf();
        idx += 1;
    }
    raw_line("        ],");
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"target\": \"live_service_graph\",");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_descriptor_bytes\": false,");
    raw_line("        \"produces_parsed_descriptor\": false,");
    raw_line("        \"validates_descriptor_schema\": false,");
    raw_line("        \"produces_validated_descriptor\": false,");
    raw_line("        \"validates_descriptor_capabilities\": false,");
    raw_line("        \"produces_capability_validated_descriptor\": false,");
    raw_line("        \"authorizes_executable_load_plan\": false,");
    raw_line("        \"produces_executable_load_plan\": false,");
    raw_line("        \"produces_executable_image_layout\": false,");
    raw_line("        \"produces_executable_page_mapping_plan\": false,");
    raw_line("        \"binds_capability_validated_descriptor_to_executable_pages\": false,");
    raw_line("        \"parses_descriptor_bytes\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"authorizes_descriptor_intake\": false,");
    raw_line("        \"authorizes_artifact_byte_intake\": false,");
    raw_line("        \"maps_executable_pages\": false,");
    raw_line("        \"jumps_to_entrypoint\": false,");
    raw_line("        \"authorizes_execution\": false,");
    raw_line("        \"mutates_service_registry\": false,");
    raw_line("        \"writes_durable_audit_state\": false,");
    raw_line("        \"installs_rollback_state\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
}

fn emit_module_loader_service_registry_mutation_boundary(
    candidate: ModuleLoaderRuntimeCandidate,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    let boundary = candidate.service_registry_mutation_boundary;
    raw_line("      \"service_registry_mutation_boundary\": {");
    raw("        \"schema\": ");
    json_str(MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SCHEMA);
    raw_line(",");
    raw("        \"id\": ");
    json_str(MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_ID);
    raw_line(",");
    raw("        \"source_evidence_schema\": ");
    json_str(boundary.source_evidence_schema);
    raw_line(",");
    raw("        \"source_evidence_event_id\": ");
    json_event_id_option(boundary.source_evidence_event_id);
    raw_line(",");
    raw("        \"source_evidence_state\": ");
    json_str(boundary.source_evidence_state);
    raw_line(",");
    raw("        \"source_evidence_status\": ");
    json_str(boundary.source_evidence_status);
    raw_line(",");
    raw("        \"source_evidence_reason\": ");
    json_str(boundary.source_evidence_reason);
    raw_line(",");
    raw("        \"source_method\": ");
    json_str(boundary.source_evidence_method);
    raw_line(",");
    raw("        \"source_fact_locator\": ");
    json_str(boundary.source_evidence_fact_locator);
    raw_line(",");
    raw("        \"status\": ");
    json_str(evaluation.service_registry_mutation_boundary_status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(evaluation.service_registry_mutation_boundary_reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(boundary.present);
    raw_line(",");
    raw("        \"source_chain_complete\": ");
    raw_bool(boundary.source_chain_complete);
    raw_line(",");
    raw("        \"execution_authorization_boundary_present\": ");
    raw_bool(boundary.execution_authorization_boundary_present);
    raw_line(",");
    raw("        \"execution_authorization_boundary_source_chain_complete\": ");
    raw_bool(boundary.execution_authorization_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"registry_write_commit_gate_present\": ");
    raw_bool(boundary.registry_write_commit_gate_present);
    raw_line(",");
    raw("        \"service_slot_binding_source_evidence_present\": ");
    raw_bool(boundary.service_slot_binding_source_evidence_present);
    raw_line(",");
    raw("        \"retained_module_evidence_present\": ");
    raw_bool(boundary.retained_module_evidence_present);
    raw_line(",");
    raw("        \"retained_service_slot_reservation_present\": ");
    raw_bool(boundary.retained_service_slot_reservation_present);
    raw_line(",");
    raw_line("        \"loader_runtime_source_evidence\": [");
    let mut idx = 0usize;
    while idx < MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
        let source = MODULE_LOADER_RUNTIME_FACT_SOURCES[idx];
        raw("          {\"fact\": ");
        json_str(source.name);
        raw(", \"schema\": ");
        json_str(source.schema);
        raw(", \"source_evidence_event_id\": ");
        json_event_id_option(boundary.loader_runtime_source_evidence_event_ids[idx]);
        raw(", \"source_evidence_present\": ");
        raw_bool(boundary.loader_runtime_source_evidence_present[idx]);
        raw(", \"fact_present\": ");
        raw_bool(boundary.loader_runtime_fact_present[idx]);
        raw("}");
        if idx + 1 != MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
            raw(",");
        }
        crlf();
        idx += 1;
    }
    raw_line("        ],");
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"target\": \"live_service_graph\",");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_descriptor_bytes\": false,");
    raw_line("        \"produces_parsed_descriptor\": false,");
    raw_line("        \"validates_descriptor_schema\": false,");
    raw_line("        \"produces_validated_descriptor\": false,");
    raw_line("        \"validates_descriptor_capabilities\": false,");
    raw_line("        \"produces_capability_validated_descriptor\": false,");
    raw_line("        \"authorizes_executable_load_plan\": false,");
    raw_line("        \"produces_executable_load_plan\": false,");
    raw_line("        \"produces_executable_image_layout\": false,");
    raw_line("        \"produces_executable_page_mapping_plan\": false,");
    raw_line("        \"binds_capability_validated_descriptor_to_executable_pages\": false,");
    raw_line("        \"parses_descriptor_bytes\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"authorizes_descriptor_intake\": false,");
    raw_line("        \"authorizes_artifact_byte_intake\": false,");
    raw_line("        \"maps_executable_pages\": false,");
    raw_line("        \"jumps_to_entrypoint\": false,");
    raw_line("        \"authorizes_execution\": false,");
    raw_line("        \"mutates_service_registry\": false,");
    raw_line("        \"writes_durable_audit_state\": false,");
    raw_line("        \"installs_rollback_state\": false,");
    raw_line("        \"authorizes_load\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
}

fn emit_module_loader_live_load_boundary(
    json_name: &'static str,
    boundary_schema: &'static str,
    boundary_id: &'static str,
    boundary: ModuleLoaderLiveLoadBoundary,
    status: &'static str,
    reason: &'static str,
) {
    raw("      ");
    json_str(json_name);
    raw_line(": {");
    raw("        \"schema\": ");
    json_str(boundary_schema);
    raw_line(",");
    raw("        \"id\": ");
    json_str(boundary_id);
    raw_line(",");
    raw("        \"source_evidence_schema\": ");
    json_str(boundary.source_evidence_schema);
    raw_line(",");
    raw("        \"source_evidence_event_id\": ");
    json_event_id_option(boundary.source_evidence_event_id);
    raw_line(",");
    raw("        \"source_evidence_state\": ");
    json_str(boundary.source_evidence_state);
    raw_line(",");
    raw("        \"source_evidence_status\": ");
    json_str(boundary.source_evidence_status);
    raw_line(",");
    raw("        \"source_evidence_reason\": ");
    json_str(boundary.source_evidence_reason);
    raw_line(",");
    raw("        \"source_method\": ");
    json_str(boundary.source_evidence_method);
    raw_line(",");
    raw("        \"source_fact_locator\": ");
    json_str(boundary.source_evidence_fact_locator);
    raw_line(",");
    raw("        \"status\": ");
    json_str(status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(reason);
    raw_line(",");
    raw("        \"present\": ");
    raw_bool(boundary.present);
    raw_line(",");
    raw("        \"source_chain_complete\": ");
    raw_bool(boundary.source_chain_complete);
    raw_line(",");
    raw("        \"load_attempt_boundary_present\": ");
    raw_bool(boundary.load_attempt_boundary_present);
    raw_line(",");
    raw("        \"load_attempt_boundary_source_chain_complete\": ");
    raw_bool(boundary.load_attempt_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"artifact_load_boundary_present\": ");
    raw_bool(boundary.artifact_load_boundary_present);
    raw_line(",");
    raw("        \"artifact_load_boundary_source_chain_complete\": ");
    raw_bool(boundary.artifact_load_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"executable_mapping_boundary_present\": ");
    raw_bool(boundary.executable_mapping_boundary_present);
    raw_line(",");
    raw("        \"executable_mapping_boundary_source_chain_complete\": ");
    raw_bool(boundary.executable_mapping_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"entrypoint_transfer_boundary_present\": ");
    raw_bool(boundary.entrypoint_transfer_boundary_present);
    raw_line(",");
    raw("        \"entrypoint_transfer_boundary_source_chain_complete\": ");
    raw_bool(boundary.entrypoint_transfer_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"service_start_boundary_present\": ");
    raw_bool(boundary.service_start_boundary_present);
    raw_line(",");
    raw("        \"service_start_boundary_source_chain_complete\": ");
    raw_bool(boundary.service_start_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"service_health_binding_boundary_present\": ");
    raw_bool(boundary.service_health_binding_boundary_present);
    raw_line(",");
    raw("        \"service_health_binding_boundary_source_chain_complete\": ");
    raw_bool(boundary.service_health_binding_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"service_running_state_boundary_present\": ");
    raw_bool(boundary.service_running_state_boundary_present);
    raw_line(",");
    raw("        \"service_running_state_boundary_source_chain_complete\": ");
    raw_bool(boundary.service_running_state_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"service_start_audit_boundary_present\": ");
    raw_bool(boundary.service_start_audit_boundary_present);
    raw_line(",");
    raw("        \"service_start_audit_boundary_source_chain_complete\": ");
    raw_bool(boundary.service_start_audit_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"service_unload_cleanup_boundary_present\": ");
    raw_bool(boundary.service_unload_cleanup_boundary_present);
    raw_line(",");
    raw("        \"service_unload_cleanup_boundary_source_chain_complete\": ");
    raw_bool(boundary.service_unload_cleanup_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"live_load_commit_boundary_present\": ");
    raw_bool(boundary.live_load_commit_boundary_present);
    raw_line(",");
    raw("        \"live_load_commit_boundary_source_chain_complete\": ");
    raw_bool(boundary.live_load_commit_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"commit_audit_boundary_present\": ");
    raw_bool(boundary.commit_audit_boundary_present);
    raw_line(",");
    raw("        \"commit_audit_boundary_source_chain_complete\": ");
    raw_bool(boundary.commit_audit_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"commit_rollback_boundary_present\": ");
    raw_bool(boundary.commit_rollback_boundary_present);
    raw_line(",");
    raw("        \"commit_rollback_boundary_source_chain_complete\": ");
    raw_bool(boundary.commit_rollback_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"commit_result_boundary_present\": ");
    raw_bool(boundary.commit_result_boundary_present);
    raw_line(",");
    raw("        \"commit_result_boundary_source_chain_complete\": ");
    raw_bool(boundary.commit_result_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"descriptor_acceptance_authority_boundary_present\": ");
    raw_bool(boundary.descriptor_acceptance_authority_boundary_present);
    raw_line(",");
    raw("        \"descriptor_acceptance_authority_boundary_source_chain_complete\": ");
    raw_bool(boundary.descriptor_acceptance_authority_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"descriptor_parser_contract_boundary_present\": ");
    raw_bool(boundary.descriptor_parser_contract_boundary_present);
    raw_line(",");
    raw("        \"descriptor_parser_contract_boundary_source_chain_complete\": ");
    raw_bool(boundary.descriptor_parser_contract_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"descriptor_parser_result_boundary_present\": ");
    raw_bool(boundary.descriptor_parser_result_boundary_present);
    raw_line(",");
    raw("        \"descriptor_parser_result_boundary_source_chain_complete\": ");
    raw_bool(boundary.descriptor_parser_result_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"descriptor_schema_validation_boundary_present\": ");
    raw_bool(boundary.descriptor_schema_validation_boundary_present);
    raw_line(",");
    raw("        \"descriptor_schema_validation_boundary_source_chain_complete\": ");
    raw_bool(boundary.descriptor_schema_validation_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"descriptor_capability_validation_boundary_present\": ");
    raw_bool(boundary.descriptor_capability_validation_boundary_present);
    raw_line(",");
    raw("        \"descriptor_capability_validation_boundary_source_chain_complete\": ");
    raw_bool(boundary.descriptor_capability_validation_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"descriptor_load_plan_boundary_present\": ");
    raw_bool(boundary.descriptor_load_plan_boundary_present);
    raw_line(",");
    raw("        \"descriptor_load_plan_boundary_source_chain_complete\": ");
    raw_bool(boundary.descriptor_load_plan_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"executable_load_plan_authority_boundary_present\": ");
    raw_bool(boundary.executable_load_plan_authority_boundary_present);
    raw_line(",");
    raw("        \"executable_load_plan_authority_boundary_source_chain_complete\": ");
    raw_bool(boundary.executable_load_plan_authority_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"executable_load_plan_result_boundary_present\": ");
    raw_bool(boundary.executable_load_plan_result_boundary_present);
    raw_line(",");
    raw("        \"executable_load_plan_result_boundary_source_chain_complete\": ");
    raw_bool(boundary.executable_load_plan_result_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"executable_image_layout_boundary_present\": ");
    raw_bool(boundary.executable_image_layout_boundary_present);
    raw_line(",");
    raw("        \"executable_image_layout_boundary_source_chain_complete\": ");
    raw_bool(boundary.executable_image_layout_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"executable_page_mapping_plan_boundary_present\": ");
    raw_bool(boundary.executable_page_mapping_plan_boundary_present);
    raw_line(",");
    raw("        \"executable_page_mapping_plan_boundary_source_chain_complete\": ");
    raw_bool(boundary.executable_page_mapping_plan_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"executable_page_mapping_boundary_present\": ");
    raw_bool(boundary.executable_page_mapping_boundary_present);
    raw_line(",");
    raw("        \"executable_page_mapping_boundary_source_chain_complete\": ");
    raw_bool(boundary.executable_page_mapping_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"descriptor_executable_page_binding_boundary_present\": ");
    raw_bool(boundary.descriptor_executable_page_binding_boundary_present);
    raw_line(",");
    raw("        \"descriptor_executable_page_binding_boundary_source_chain_complete\": ");
    raw_bool(boundary.descriptor_executable_page_binding_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"executable_entrypoint_binding_boundary_present\": ");
    raw_bool(boundary.executable_entrypoint_binding_boundary_present);
    raw_line(",");
    raw("        \"executable_entrypoint_binding_boundary_source_chain_complete\": ");
    raw_bool(boundary.executable_entrypoint_binding_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"executable_entrypoint_transfer_authorization_boundary_present\": ");
    raw_bool(boundary.executable_entrypoint_transfer_authorization_boundary_present);
    raw_line(",");
    raw(
        "        \"executable_entrypoint_transfer_authorization_boundary_source_chain_complete\": ",
    );
    raw_bool(boundary.executable_entrypoint_transfer_authorization_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"executable_entrypoint_transfer_boundary_present\": ");
    raw_bool(boundary.executable_entrypoint_transfer_boundary_present);
    raw_line(",");
    raw("        \"executable_entrypoint_transfer_boundary_source_chain_complete\": ");
    raw_bool(boundary.executable_entrypoint_transfer_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"artifact_byte_intake_boundary_present\": ");
    raw_bool(boundary.artifact_byte_intake_boundary_present);
    raw_line(",");
    raw("        \"artifact_byte_intake_boundary_source_chain_complete\": ");
    raw_bool(boundary.artifact_byte_intake_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"execution_authorization_boundary_present\": ");
    raw_bool(boundary.execution_authorization_boundary_present);
    raw_line(",");
    raw("        \"execution_authorization_boundary_source_chain_complete\": ");
    raw_bool(boundary.execution_authorization_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"service_registry_mutation_boundary_present\": ");
    raw_bool(boundary.service_registry_mutation_boundary_present);
    raw_line(",");
    raw("        \"service_registry_mutation_boundary_source_chain_complete\": ");
    raw_bool(boundary.service_registry_mutation_boundary_source_chain_complete);
    raw_line(",");
    raw("        \"service_slot_binding_source_evidence_present\": ");
    raw_bool(boundary.service_slot_binding_source_evidence_present);
    raw_line(",");
    raw("        \"health_state_hooks_source_evidence_present\": ");
    raw_bool(boundary.health_state_hooks_source_evidence_present);
    raw_line(",");
    raw("        \"artifact_hash_binding_present\": ");
    raw_bool(boundary.artifact_hash_binding_present);
    raw_line(",");
    raw("        \"entrypoint_abi_source_evidence_present\": ");
    raw_bool(boundary.entrypoint_abi_source_evidence_present);
    raw_line(",");
    raw("        \"address_space_source_evidence_present\": ");
    raw_bool(boundary.address_space_source_evidence_present);
    raw_line(",");
    raw("        \"memory_map_source_evidence_present\": ");
    raw_bool(boundary.memory_map_source_evidence_present);
    raw_line(",");
    raw("        \"capability_import_table_source_evidence_present\": ");
    raw_bool(boundary.capability_import_table_source_evidence_present);
    raw_line(",");
    raw("        \"audit_rollback_write_boundary_source_evidence_present\": ");
    raw_bool(boundary.audit_rollback_write_boundary_source_evidence_present);
    raw_line(",");
    raw("        \"retained_module_evidence_present\": ");
    raw_bool(boundary.retained_module_evidence_present);
    raw_line(",");
    raw("        \"retained_artifact_reference_present\": ");
    raw_bool(boundary.retained_artifact_reference_present);
    raw_line(",");
    raw("        \"retained_service_slot_reservation_present\": ");
    raw_bool(boundary.retained_service_slot_reservation_present);
    raw_line(",");
    raw_line("        \"loader_runtime_source_evidence\": [");
    let mut idx = 0usize;
    while idx < MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
        let source = MODULE_LOADER_RUNTIME_FACT_SOURCES[idx];
        raw("          {\"fact\": ");
        json_str(source.name);
        raw(", \"schema\": ");
        json_str(source.schema);
        raw(", \"source_evidence_event_id\": ");
        json_event_id_option(boundary.loader_runtime_source_evidence_event_ids[idx]);
        raw(", \"source_evidence_present\": ");
        raw_bool(boundary.loader_runtime_source_evidence_present[idx]);
        raw(", \"fact_present\": ");
        raw_bool(boundary.loader_runtime_fact_present[idx]);
        raw("}");
        if idx + 1 != MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
            raw(",");
        }
        crlf();
        idx += 1;
    }
    raw_line("        ],");
    raw_line("        \"requested_capability\": \"cap.module.load_ephemeral\",");
    raw_line("        \"load_mode\": \"ram_only\",");
    raw_line("        \"target\": \"live_service_graph\",");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_descriptor_bytes\": false,");
    raw_line("        \"produces_parsed_descriptor\": false,");
    raw_line("        \"validates_descriptor_schema\": false,");
    raw_line("        \"produces_validated_descriptor\": false,");
    raw_line("        \"validates_descriptor_capabilities\": false,");
    raw_line("        \"produces_capability_validated_descriptor\": false,");
    raw_line("        \"authorizes_executable_load_plan\": false,");
    raw_line("        \"produces_executable_load_plan\": false,");
    raw_line("        \"produces_executable_image_layout\": false,");
    raw_line("        \"produces_executable_page_mapping_plan\": false,");
    raw_line("        \"binds_capability_validated_descriptor_to_executable_pages\": false,");
    raw_line("        \"parses_descriptor_bytes\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"authorizes_descriptor_intake\": false,");
    raw_line("        \"authorizes_artifact_byte_intake\": false,");
    raw_line("        \"maps_executable_pages\": false,");
    raw_line("        \"jumps_to_entrypoint\": false,");
    raw_line("        \"authorizes_execution\": false,");
    raw_line("        \"mutates_service_registry\": false,");
    raw_line("        \"writes_durable_audit_state\": false,");
    raw_line("        \"installs_rollback_state\": false,");
    raw_line("        \"authorizes_load\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"loads_artifact\": false,");
    raw_line("        \"starts_service\": false,");
    raw_line("        \"marks_service_running\": false,");
    raw_line("        \"creates_service_health_records\": false,");
    raw_line("        \"writes_service_start_audit_record\": false,");
    raw_line("        \"unloads_service\": false,");
    raw_line("        \"cleans_up_service_slot\": false,");
    raw_line("        \"commits_live_load\": false,");
    raw_line("        \"writes_load_commit_audit_record\": false,");
    raw_line("        \"installs_commit_rollback_record\": false,");
    raw_line("        \"records_load_result\": false,");
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
    raw_line(
        "            \"execution_commit_gate\": \"raios.module_loader_runtime_execution_commit_gate.v0\",",
    );
    raw_line(
        "            \"descriptor_intake_boundary\": \"raios.module_loader_descriptor_intake_boundary.v0\",",
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
    method_eq(source.name, "loader_identity")
        || method_eq(source.name, "artifact_hash_binding")
        || method_eq(source.name, "entrypoint_abi")
        || method_eq(source.name, "address_space_boundary")
        || method_eq(source.name, "memory_map_constraints")
        || method_eq(source.name, "capability_import_table")
        || method_eq(source.name, "service_slot_binding")
        || method_eq(source.name, "health_state_hooks")
        || method_eq(source.name, "rollback_hooks")
        || method_eq(source.name, "audit_rollback_write_boundary_binding")
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
    raw(", \"actual_entrypoint_abi_source_evidence_present\": ");
    raw_bool(case.actual_entrypoint_abi_source_evidence_present);
    raw(", \"actual_entrypoint_abi_source_evidence_state\": ");
    json_str(case.actual_entrypoint_abi_source_evidence_state);
    raw(", \"actual_entrypoint_abi_source_evidence_status\": ");
    json_str(case.actual_entrypoint_abi_source_evidence_status);
    raw(", \"actual_entrypoint_abi_source_evidence_reason\": ");
    json_str(case.actual_entrypoint_abi_source_evidence_reason);
    raw(", \"actual_address_space_source_evidence_present\": ");
    raw_bool(case.actual_address_space_source_evidence_present);
    raw(", \"actual_address_space_source_evidence_state\": ");
    json_str(case.actual_address_space_source_evidence_state);
    raw(", \"actual_address_space_source_evidence_status\": ");
    json_str(case.actual_address_space_source_evidence_status);
    raw(", \"actual_address_space_source_evidence_reason\": ");
    json_str(case.actual_address_space_source_evidence_reason);
    raw(", \"actual_memory_map_source_evidence_present\": ");
    raw_bool(case.actual_memory_map_source_evidence_present);
    raw(", \"actual_memory_map_source_evidence_state\": ");
    json_str(case.actual_memory_map_source_evidence_state);
    raw(", \"actual_memory_map_source_evidence_status\": ");
    json_str(case.actual_memory_map_source_evidence_status);
    raw(", \"actual_memory_map_source_evidence_reason\": ");
    json_str(case.actual_memory_map_source_evidence_reason);
    raw(", \"actual_capability_table_source_evidence_present\": ");
    raw_bool(case.actual_capability_table_source_evidence_present);
    raw(", \"actual_capability_table_source_evidence_state\": ");
    json_str(case.actual_capability_table_source_evidence_state);
    raw(", \"actual_capability_table_source_evidence_status\": ");
    json_str(case.actual_capability_table_source_evidence_status);
    raw(", \"actual_capability_table_source_evidence_reason\": ");
    json_str(case.actual_capability_table_source_evidence_reason);
    raw(", \"actual_service_slot_source_evidence_present\": ");
    raw_bool(case.actual_service_slot_source_evidence_present);
    raw(", \"actual_service_slot_source_evidence_state\": ");
    json_str(case.actual_service_slot_source_evidence_state);
    raw(", \"actual_service_slot_source_evidence_status\": ");
    json_str(case.actual_service_slot_source_evidence_status);
    raw(", \"actual_service_slot_source_evidence_reason\": ");
    json_str(case.actual_service_slot_source_evidence_reason);
    raw(", \"actual_health_source_evidence_present\": ");
    raw_bool(case.actual_health_source_evidence_present);
    raw(", \"actual_health_source_evidence_state\": ");
    json_str(case.actual_health_source_evidence_state);
    raw(", \"actual_health_source_evidence_status\": ");
    json_str(case.actual_health_source_evidence_status);
    raw(", \"actual_health_source_evidence_reason\": ");
    json_str(case.actual_health_source_evidence_reason);
    raw(", \"actual_rollback_source_evidence_present\": ");
    raw_bool(case.actual_rollback_source_evidence_present);
    raw(", \"actual_rollback_source_evidence_state\": ");
    json_str(case.actual_rollback_source_evidence_state);
    raw(", \"actual_rollback_source_evidence_status\": ");
    json_str(case.actual_rollback_source_evidence_status);
    raw(", \"actual_rollback_source_evidence_reason\": ");
    json_str(case.actual_rollback_source_evidence_reason);
    raw(", \"actual_write_boundary_source_evidence_present\": ");
    raw_bool(case.actual_write_boundary_source_evidence_present);
    raw(", \"actual_write_boundary_source_evidence_state\": ");
    json_str(case.actual_write_boundary_source_evidence_state);
    raw(", \"actual_write_boundary_source_evidence_status\": ");
    json_str(case.actual_write_boundary_source_evidence_status);
    raw(", \"actual_write_boundary_source_evidence_reason\": ");
    json_str(case.actual_write_boundary_source_evidence_reason);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"loads_artifact\": false, \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"starts_service\": false, \"marks_service_running\": false, \"can_load\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

fn module_loader_runtime_execution_commit_gate_source_evidence(
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

fn module_loader_descriptor_intake_boundary_source_evidence(
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

fn module_loader_artifact_byte_intake_boundary_source_evidence(
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

fn module_loader_execution_authorization_boundary_source_evidence(
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

fn module_loader_service_registry_mutation_boundary_source_evidence(
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

fn module_loader_load_attempt_boundary_source_evidence(
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

fn module_loader_artifact_load_boundary_source_evidence(
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

fn module_loader_executable_mapping_boundary_source_evidence(
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

fn module_loader_entrypoint_transfer_boundary_source_evidence(
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

fn module_loader_service_start_boundary_source_evidence(
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

fn module_loader_service_health_binding_boundary_source_evidence(
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

fn module_loader_service_running_state_boundary_source_evidence(
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

fn module_loader_service_start_audit_boundary_source_evidence(
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

fn module_loader_service_unload_cleanup_boundary_source_evidence(
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

fn module_loader_live_load_commit_boundary_source_evidence(
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

fn module_loader_commit_audit_boundary_source_evidence(
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

fn module_loader_commit_rollback_boundary_source_evidence(
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

fn module_loader_commit_result_boundary_source_evidence(
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

fn module_loader_descriptor_acceptance_authority_boundary_source_evidence(
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

fn module_loader_descriptor_parser_contract_boundary_source_evidence(
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

fn module_loader_descriptor_parser_result_boundary_source_evidence(
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

fn module_loader_descriptor_schema_validation_boundary_source_evidence(
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

fn module_loader_descriptor_capability_validation_boundary_source_evidence(
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

fn module_loader_descriptor_load_plan_boundary_source_evidence(
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

fn module_loader_executable_load_plan_authority_boundary_source_evidence(
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

fn module_loader_executable_load_plan_result_boundary_source_evidence(
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

fn module_loader_executable_image_layout_boundary_source_evidence(
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

fn module_loader_executable_page_mapping_plan_boundary_source_evidence(
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

fn module_loader_executable_page_mapping_boundary_source_evidence(
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

fn module_loader_descriptor_executable_page_binding_boundary_source_evidence(
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

fn module_loader_executable_entrypoint_binding_boundary_source_evidence(
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

fn module_loader_executable_entrypoint_transfer_authorization_boundary_source_evidence(
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

fn module_loader_executable_entrypoint_transfer_boundary_source_evidence(
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

fn module_loader_executable_entrypoint_handoff_boundary_source_evidence(
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
    evidence
}

fn module_loader_retained_module_evidence_present(
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
fn module_loader_live_load_boundary_source_evidence_record(
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

fn module_loader_runtime_snapshot(
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

fn module_loader_runtime_observed_entrypoint_abi_missing_fact() -> ModuleLoaderRuntimeFact {
    module_loader_runtime_observed_loader_fact_missing_fact(
        MODULE_LOADER_RUNTIME_FACT_SOURCES[2],
        44,
    )
}

fn module_loader_runtime_observed_loader_fact_missing_fact(
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

fn module_loader_runtime_execution_commit_gate_missing() -> ModuleLoaderRuntimeExecutionCommitGate {
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

fn module_loader_descriptor_intake_boundary_missing() -> ModuleLoaderDescriptorIntakeBoundary {
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

fn module_loader_artifact_byte_intake_boundary_missing() -> ModuleLoaderArtifactByteIntakeBoundary {
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

fn module_loader_execution_authorization_boundary_missing(
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

fn module_loader_service_registry_mutation_boundary_missing(
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

fn module_loader_live_load_boundary_missing(
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
    let execution_commit_gate_status = if candidate.execution_commit_gate.present
        && candidate.execution_commit_gate.source_chain_complete
    {
        MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_STATUS
    } else {
        MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_MISSING_STATUS
    };
    let execution_commit_gate_reason = if candidate.execution_commit_gate.present
        && candidate.execution_commit_gate.source_chain_complete
    {
        MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_REASON
    } else {
        MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_CHAIN_INCOMPLETE_REASON
    };
    let descriptor_intake_boundary_status = if candidate.descriptor_intake_boundary.present
        && candidate.descriptor_intake_boundary.source_chain_complete
    {
        MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_STATUS
    } else {
        MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_MISSING_STATUS
    };
    let descriptor_intake_boundary_reason = if candidate.descriptor_intake_boundary.present
        && candidate.descriptor_intake_boundary.source_chain_complete
    {
        MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_REASON
    } else {
        MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON
    };
    let artifact_byte_intake_boundary_status = if candidate.artifact_byte_intake_boundary.present
        && candidate
            .artifact_byte_intake_boundary
            .source_chain_complete
    {
        MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_STATUS
    } else {
        MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_MISSING_STATUS
    };
    let artifact_byte_intake_boundary_reason = if candidate.artifact_byte_intake_boundary.present
        && candidate
            .artifact_byte_intake_boundary
            .source_chain_complete
    {
        MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_REASON
    } else {
        MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON
    };
    let execution_authorization_boundary_status =
        if candidate.execution_authorization_boundary.present
            && candidate
                .execution_authorization_boundary
                .source_chain_complete
        {
            MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_STATUS
        } else {
            MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_MISSING_STATUS
        };
    let execution_authorization_boundary_reason =
        if candidate.execution_authorization_boundary.present
            && candidate
                .execution_authorization_boundary
                .source_chain_complete
        {
            MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_REASON
        } else {
            MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON
        };
    let service_registry_mutation_boundary_status =
        if candidate.service_registry_mutation_boundary.present
            && candidate
                .service_registry_mutation_boundary
                .source_chain_complete
        {
            MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_STATUS
        } else {
            MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_MISSING_STATUS
        };
    let service_registry_mutation_boundary_reason =
        if candidate.service_registry_mutation_boundary.present
            && candidate
                .service_registry_mutation_boundary
                .source_chain_complete
        {
            MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_REASON
        } else {
            MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON
        };
    let (load_attempt_boundary_status, load_attempt_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.load_attempt_boundary,
            MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_STATUS,
            MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_REASON,
            MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (artifact_load_boundary_status, artifact_load_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.artifact_load_boundary,
            MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_STATUS,
            MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_REASON,
            MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (executable_mapping_boundary_status, executable_mapping_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.executable_mapping_boundary,
            MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_STATUS,
            MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_REASON,
            MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (entrypoint_transfer_boundary_status, entrypoint_transfer_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.entrypoint_transfer_boundary,
            MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_STATUS,
            MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_REASON,
            MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (service_start_boundary_status, service_start_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.service_start_boundary,
            MODULE_LOADER_SERVICE_START_BOUNDARY_STATUS,
            MODULE_LOADER_SERVICE_START_BOUNDARY_REASON,
            MODULE_LOADER_SERVICE_START_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (service_health_binding_boundary_status, service_health_binding_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.service_health_binding_boundary,
            MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_STATUS,
            MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_REASON,
            MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (service_running_state_boundary_status, service_running_state_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.service_running_state_boundary,
            MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_STATUS,
            MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_REASON,
            MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (service_start_audit_boundary_status, service_start_audit_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.service_start_audit_boundary,
            MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_STATUS,
            MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_REASON,
            MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (service_unload_cleanup_boundary_status, service_unload_cleanup_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.service_unload_cleanup_boundary,
            MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_STATUS,
            MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_REASON,
            MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (live_load_commit_boundary_status, live_load_commit_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.live_load_commit_boundary,
            MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_STATUS,
            MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_REASON,
            MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (commit_audit_boundary_status, commit_audit_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.commit_audit_boundary,
            MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_STATUS,
            MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_REASON,
            MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (commit_rollback_boundary_status, commit_rollback_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.commit_rollback_boundary,
            MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_STATUS,
            MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_REASON,
            MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (commit_result_boundary_status, commit_result_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.commit_result_boundary,
            MODULE_LOADER_COMMIT_RESULT_BOUNDARY_STATUS,
            MODULE_LOADER_COMMIT_RESULT_BOUNDARY_REASON,
            MODULE_LOADER_COMMIT_RESULT_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (
        descriptor_acceptance_authority_boundary_status,
        descriptor_acceptance_authority_boundary_reason,
    ) = evaluate_module_loader_live_load_boundary(
        candidate.descriptor_acceptance_authority_boundary,
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_STATUS,
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_REASON,
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    );
    let (descriptor_parser_contract_boundary_status, descriptor_parser_contract_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.descriptor_parser_contract_boundary,
            MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_STATUS,
            MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_REASON,
            MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (descriptor_parser_result_boundary_status, descriptor_parser_result_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.descriptor_parser_result_boundary,
            MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_STATUS,
            MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_REASON,
            MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (
        descriptor_schema_validation_boundary_status,
        descriptor_schema_validation_boundary_reason,
    ) = evaluate_module_loader_live_load_boundary(
        candidate.descriptor_schema_validation_boundary,
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_STATUS,
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_REASON,
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    );
    let (
        descriptor_capability_validation_boundary_status,
        descriptor_capability_validation_boundary_reason,
    ) = evaluate_module_loader_live_load_boundary(
        candidate.descriptor_capability_validation_boundary,
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_STATUS,
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_REASON,
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    );
    let (descriptor_load_plan_boundary_status, descriptor_load_plan_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.descriptor_load_plan_boundary,
            MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_STATUS,
            MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_REASON,
            MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (
        executable_load_plan_authority_boundary_status,
        executable_load_plan_authority_boundary_reason,
    ) = evaluate_module_loader_live_load_boundary(
        candidate.executable_load_plan_authority_boundary,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    );
    let (executable_load_plan_result_boundary_status, executable_load_plan_result_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.executable_load_plan_result_boundary,
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_STATUS,
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_REASON,
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (executable_image_layout_boundary_status, executable_image_layout_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.executable_image_layout_boundary,
            MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_STATUS,
            MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_REASON,
            MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (
        executable_page_mapping_plan_boundary_status,
        executable_page_mapping_plan_boundary_reason,
    ) = evaluate_module_loader_live_load_boundary(
        candidate.executable_page_mapping_plan_boundary,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    );
    let (executable_page_mapping_boundary_status, executable_page_mapping_boundary_reason) =
        evaluate_module_loader_live_load_boundary(
            candidate.executable_page_mapping_boundary,
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_STATUS,
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_REASON,
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_MISSING_STATUS,
            MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
        );
    let (
        descriptor_executable_page_binding_boundary_status,
        descriptor_executable_page_binding_boundary_reason,
    ) = evaluate_module_loader_live_load_boundary(
        candidate.descriptor_executable_page_binding_boundary,
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_STATUS,
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_REASON,
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    );
    let (
        executable_entrypoint_binding_boundary_status,
        executable_entrypoint_binding_boundary_reason,
    ) = evaluate_module_loader_live_load_boundary(
        candidate.executable_entrypoint_binding_boundary,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    );
    let (
        executable_entrypoint_transfer_authorization_boundary_status,
        executable_entrypoint_transfer_authorization_boundary_reason,
    ) = evaluate_module_loader_live_load_boundary(
        candidate.executable_entrypoint_transfer_authorization_boundary,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    );
    let (
        executable_entrypoint_transfer_boundary_status,
        executable_entrypoint_transfer_boundary_reason,
    ) = evaluate_module_loader_live_load_boundary(
        candidate.executable_entrypoint_transfer_boundary,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    );
    let (
        executable_entrypoint_handoff_boundary_status,
        executable_entrypoint_handoff_boundary_reason,
    ) = evaluate_module_loader_live_load_boundary(
        candidate.executable_entrypoint_handoff_boundary,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
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
            candidate.service_slot_allocator_unready_status,
            candidate.service_slot_allocator_unready_reason,
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
    } else if !candidate.execution_commit_gate.present
        || !candidate.execution_commit_gate.source_chain_complete
    {
        (
            "denied_missing_module_loader_runtime_execution_commit_gate",
            execution_commit_gate_reason,
        )
    } else if !candidate.descriptor_intake_boundary.present
        || !candidate.descriptor_intake_boundary.source_chain_complete
    {
        (
            "denied_missing_module_loader_descriptor_intake_boundary",
            descriptor_intake_boundary_reason,
        )
    } else if !candidate.artifact_byte_intake_boundary.present
        || !candidate
            .artifact_byte_intake_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_artifact_byte_intake_boundary",
            artifact_byte_intake_boundary_reason,
        )
    } else if !candidate.execution_authorization_boundary.present
        || !candidate
            .execution_authorization_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_execution_authorization_boundary",
            execution_authorization_boundary_reason,
        )
    } else if !candidate.service_registry_mutation_boundary.present
        || !candidate
            .service_registry_mutation_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_service_registry_mutation_boundary",
            service_registry_mutation_boundary_reason,
        )
    } else if !candidate.load_attempt_boundary.present
        || !candidate.load_attempt_boundary.source_chain_complete
    {
        (
            "denied_missing_module_loader_load_attempt_boundary",
            load_attempt_boundary_reason,
        )
    } else if !candidate.artifact_load_boundary.present
        || !candidate.artifact_load_boundary.source_chain_complete
    {
        (
            "denied_missing_module_loader_artifact_load_boundary",
            artifact_load_boundary_reason,
        )
    } else if !candidate.executable_mapping_boundary.present
        || !candidate.executable_mapping_boundary.source_chain_complete
    {
        (
            "denied_missing_module_loader_executable_mapping_boundary",
            executable_mapping_boundary_reason,
        )
    } else if !candidate.entrypoint_transfer_boundary.present
        || !candidate.entrypoint_transfer_boundary.source_chain_complete
    {
        (
            "denied_missing_module_loader_entrypoint_transfer_boundary",
            entrypoint_transfer_boundary_reason,
        )
    } else if !candidate.service_start_boundary.present
        || !candidate.service_start_boundary.source_chain_complete
    {
        (
            "denied_missing_module_loader_service_start_boundary",
            service_start_boundary_reason,
        )
    } else if !candidate.service_health_binding_boundary.present
        || !candidate
            .service_health_binding_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_service_health_binding_boundary",
            service_health_binding_boundary_reason,
        )
    } else if !candidate.service_running_state_boundary.present
        || !candidate
            .service_running_state_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_service_running_state_boundary",
            service_running_state_boundary_reason,
        )
    } else if !candidate.service_start_audit_boundary.present
        || !candidate.service_start_audit_boundary.source_chain_complete
    {
        (
            "denied_missing_module_loader_service_start_audit_boundary",
            service_start_audit_boundary_reason,
        )
    } else if !candidate.service_unload_cleanup_boundary.present
        || !candidate
            .service_unload_cleanup_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_service_unload_cleanup_boundary",
            service_unload_cleanup_boundary_reason,
        )
    } else if !candidate.live_load_commit_boundary.present
        || !candidate.live_load_commit_boundary.source_chain_complete
    {
        (
            "denied_missing_module_loader_live_load_commit_boundary",
            live_load_commit_boundary_reason,
        )
    } else if !candidate.commit_audit_boundary.present
        || !candidate.commit_audit_boundary.source_chain_complete
    {
        (
            "denied_missing_module_loader_commit_audit_boundary",
            commit_audit_boundary_reason,
        )
    } else if !candidate.commit_rollback_boundary.present
        || !candidate.commit_rollback_boundary.source_chain_complete
    {
        (
            "denied_missing_module_loader_commit_rollback_boundary",
            commit_rollback_boundary_reason,
        )
    } else if !candidate.commit_result_boundary.present
        || !candidate.commit_result_boundary.source_chain_complete
    {
        (
            "denied_missing_module_loader_commit_result_boundary",
            commit_result_boundary_reason,
        )
    } else if !candidate.descriptor_acceptance_authority_boundary.present
        || !candidate
            .descriptor_acceptance_authority_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_descriptor_acceptance_authority_boundary",
            descriptor_acceptance_authority_boundary_reason,
        )
    } else if !candidate.descriptor_parser_contract_boundary.present
        || !candidate
            .descriptor_parser_contract_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_descriptor_parser_contract_boundary",
            descriptor_parser_contract_boundary_reason,
        )
    } else if !candidate.descriptor_parser_result_boundary.present
        || !candidate
            .descriptor_parser_result_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_descriptor_parser_result_boundary",
            descriptor_parser_result_boundary_reason,
        )
    } else if !candidate.descriptor_schema_validation_boundary.present
        || !candidate
            .descriptor_schema_validation_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_descriptor_schema_validation_boundary",
            descriptor_schema_validation_boundary_reason,
        )
    } else if !candidate.descriptor_capability_validation_boundary.present
        || !candidate
            .descriptor_capability_validation_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_descriptor_capability_validation_boundary",
            descriptor_capability_validation_boundary_reason,
        )
    } else if !candidate.descriptor_load_plan_boundary.present
        || !candidate
            .descriptor_load_plan_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_descriptor_load_plan_boundary",
            descriptor_load_plan_boundary_reason,
        )
    } else if !candidate.executable_load_plan_authority_boundary.present
        || !candidate
            .executable_load_plan_authority_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_executable_load_plan_authority_boundary",
            executable_load_plan_authority_boundary_reason,
        )
    } else if !candidate.executable_load_plan_result_boundary.present
        || !candidate
            .executable_load_plan_result_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_executable_load_plan_result_boundary",
            executable_load_plan_result_boundary_reason,
        )
    } else if !candidate.executable_image_layout_boundary.present
        || !candidate
            .executable_image_layout_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_executable_image_layout_boundary",
            executable_image_layout_boundary_reason,
        )
    } else if !candidate.executable_page_mapping_plan_boundary.present
        || !candidate
            .executable_page_mapping_plan_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_executable_page_mapping_plan_boundary",
            executable_page_mapping_plan_boundary_reason,
        )
    } else if !candidate.executable_page_mapping_boundary.present
        || !candidate
            .executable_page_mapping_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_executable_page_mapping_boundary",
            executable_page_mapping_boundary_reason,
        )
    } else if !candidate
        .descriptor_executable_page_binding_boundary
        .present
        || !candidate
            .descriptor_executable_page_binding_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_descriptor_executable_page_binding_boundary",
            descriptor_executable_page_binding_boundary_reason,
        )
    } else if !candidate.executable_entrypoint_binding_boundary.present
        || !candidate
            .executable_entrypoint_binding_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_executable_entrypoint_binding_boundary",
            executable_entrypoint_binding_boundary_reason,
        )
    } else if !candidate
        .executable_entrypoint_transfer_authorization_boundary
        .present
        || !candidate
            .executable_entrypoint_transfer_authorization_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_executable_entrypoint_transfer_authorization_boundary",
            executable_entrypoint_transfer_authorization_boundary_reason,
        )
    } else if !candidate.executable_entrypoint_transfer_boundary.present
        || !candidate
            .executable_entrypoint_transfer_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_executable_entrypoint_transfer_boundary",
            executable_entrypoint_transfer_boundary_reason,
        )
    } else if !candidate.executable_entrypoint_handoff_boundary.present
        || !candidate
            .executable_entrypoint_handoff_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_executable_entrypoint_handoff_boundary",
            executable_entrypoint_handoff_boundary_reason,
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
        execution_commit_gate_status,
        execution_commit_gate_reason,
        descriptor_intake_boundary_status,
        descriptor_intake_boundary_reason,
        artifact_byte_intake_boundary_status,
        artifact_byte_intake_boundary_reason,
        execution_authorization_boundary_status,
        execution_authorization_boundary_reason,
        service_registry_mutation_boundary_status,
        service_registry_mutation_boundary_reason,
        load_attempt_boundary_status,
        load_attempt_boundary_reason,
        artifact_load_boundary_status,
        artifact_load_boundary_reason,
        executable_mapping_boundary_status,
        executable_mapping_boundary_reason,
        entrypoint_transfer_boundary_status,
        entrypoint_transfer_boundary_reason,
        service_start_boundary_status,
        service_start_boundary_reason,
        service_health_binding_boundary_status,
        service_health_binding_boundary_reason,
        service_running_state_boundary_status,
        service_running_state_boundary_reason,
        service_start_audit_boundary_status,
        service_start_audit_boundary_reason,
        service_unload_cleanup_boundary_status,
        service_unload_cleanup_boundary_reason,
        live_load_commit_boundary_status,
        live_load_commit_boundary_reason,
        commit_audit_boundary_status,
        commit_audit_boundary_reason,
        commit_rollback_boundary_status,
        commit_rollback_boundary_reason,
        commit_result_boundary_status,
        commit_result_boundary_reason,
        descriptor_acceptance_authority_boundary_status,
        descriptor_acceptance_authority_boundary_reason,
        descriptor_parser_contract_boundary_status,
        descriptor_parser_contract_boundary_reason,
        descriptor_parser_result_boundary_status,
        descriptor_parser_result_boundary_reason,
        descriptor_schema_validation_boundary_status,
        descriptor_schema_validation_boundary_reason,
        descriptor_capability_validation_boundary_status,
        descriptor_capability_validation_boundary_reason,
        descriptor_load_plan_boundary_status,
        descriptor_load_plan_boundary_reason,
        executable_load_plan_authority_boundary_status,
        executable_load_plan_authority_boundary_reason,
        executable_load_plan_result_boundary_status,
        executable_load_plan_result_boundary_reason,
        executable_image_layout_boundary_status,
        executable_image_layout_boundary_reason,
        executable_page_mapping_plan_boundary_status,
        executable_page_mapping_plan_boundary_reason,
        executable_page_mapping_boundary_status,
        executable_page_mapping_boundary_reason,
        descriptor_executable_page_binding_boundary_status,
        descriptor_executable_page_binding_boundary_reason,
        executable_entrypoint_binding_boundary_status,
        executable_entrypoint_binding_boundary_reason,
        executable_entrypoint_transfer_authorization_boundary_status,
        executable_entrypoint_transfer_authorization_boundary_reason,
        executable_entrypoint_transfer_boundary_status,
        executable_entrypoint_transfer_boundary_reason,
        executable_entrypoint_handoff_boundary_status,
        executable_entrypoint_handoff_boundary_reason,
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

fn evaluate_module_loader_live_load_boundary(
    boundary: ModuleLoaderLiveLoadBoundary,
    available_status: &'static str,
    available_reason: &'static str,
    missing_status: &'static str,
    missing_reason: &'static str,
) -> (&'static str, &'static str) {
    if boundary.present && boundary.source_chain_complete {
        (available_status, available_reason)
    } else {
        (missing_status, missing_reason)
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
                service_slot_allocator_unready_status:
                    "denied_missing_service_slot_allocator_runtime",
                service_slot_allocator_unready_reason: "service_slot_allocator_runtime_missing",
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
            "entrypoint_abi_observed_source_evidence_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_entrypoint_abi_missing",
            ModuleLoaderRuntimeCandidate {
                entrypoint_abi: module_loader_runtime_observed_entrypoint_abi_missing_fact(),
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
            "address_space_boundary_observed_source_evidence_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_address_space_boundary_missing",
            ModuleLoaderRuntimeCandidate {
                address_space_boundary: module_loader_runtime_observed_loader_fact_missing_fact(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[3],
                    45,
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
            "memory_map_constraints_observed_source_evidence_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_memory_map_constraints_missing",
            ModuleLoaderRuntimeCandidate {
                memory_map_constraints: module_loader_runtime_observed_loader_fact_missing_fact(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[4],
                    46,
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
            "capability_import_table_observed_source_evidence_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_capability_import_table_missing",
            ModuleLoaderRuntimeCandidate {
                capability_import_table: module_loader_runtime_observed_loader_fact_missing_fact(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[5],
                    47,
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
            "service_slot_binding_observed_source_evidence_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_service_slot_binding_missing",
            ModuleLoaderRuntimeCandidate {
                service_slot_binding: module_loader_runtime_observed_loader_fact_missing_fact(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[6],
                    48,
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
            "health_state_hooks_observed_source_evidence_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_health_state_hooks_missing",
            ModuleLoaderRuntimeCandidate {
                health_state_hooks: module_loader_runtime_observed_loader_fact_missing_fact(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[7],
                    49,
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
            "rollback_hooks_observed_source_evidence_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_rollback_hooks_missing",
            ModuleLoaderRuntimeCandidate {
                rollback_hooks: module_loader_runtime_observed_loader_fact_missing_fact(
                    MODULE_LOADER_RUNTIME_FACT_SOURCES[8],
                    50,
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
            "audit_rollback_write_boundary_binding_observed_source_evidence_missing",
            "denied_missing_loader_runtime_fact",
            "module_loader_audit_rollback_write_boundary_binding_missing",
            ModuleLoaderRuntimeCandidate {
                audit_rollback_write_boundary_binding:
                    module_loader_runtime_observed_loader_fact_missing_fact(
                        MODULE_LOADER_RUNTIME_FACT_SOURCES[9],
                        51,
                    ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "execution_commit_gate_missing",
            "denied_missing_module_loader_runtime_execution_commit_gate",
            "module_loader_runtime_execution_commit_gate_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                execution_commit_gate: module_loader_runtime_execution_commit_gate_missing(),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "descriptor_intake_boundary_missing",
            "denied_missing_module_loader_descriptor_intake_boundary",
            "module_loader_descriptor_intake_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                descriptor_intake_boundary: module_loader_descriptor_intake_boundary_missing(),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "artifact_byte_intake_boundary_missing",
            "denied_missing_module_loader_artifact_byte_intake_boundary",
            "module_loader_artifact_byte_intake_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                artifact_byte_intake_boundary: module_loader_artifact_byte_intake_boundary_missing(
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "execution_authorization_boundary_missing",
            "denied_missing_module_loader_execution_authorization_boundary",
            "module_loader_execution_authorization_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                execution_authorization_boundary:
                    module_loader_execution_authorization_boundary_missing(),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "service_registry_mutation_boundary_missing",
            "denied_missing_module_loader_service_registry_mutation_boundary",
            "module_loader_service_registry_mutation_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                service_registry_mutation_boundary:
                    module_loader_service_registry_mutation_boundary_missing(),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "load_attempt_boundary_missing",
            "denied_missing_module_loader_load_attempt_boundary",
            "module_loader_load_attempt_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                load_attempt_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "artifact_load_boundary_missing",
            "denied_missing_module_loader_artifact_load_boundary",
            "module_loader_artifact_load_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                artifact_load_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "executable_mapping_boundary_missing",
            "denied_missing_module_loader_executable_mapping_boundary",
            "module_loader_executable_mapping_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                executable_mapping_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "entrypoint_transfer_boundary_missing",
            "denied_missing_module_loader_entrypoint_transfer_boundary",
            "module_loader_entrypoint_transfer_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                entrypoint_transfer_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "service_start_boundary_missing",
            "denied_missing_module_loader_service_start_boundary",
            "module_loader_service_start_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                service_start_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "service_health_binding_boundary_missing",
            "denied_missing_module_loader_service_health_binding_boundary",
            "module_loader_service_health_binding_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                service_health_binding_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "service_running_state_boundary_missing",
            "denied_missing_module_loader_service_running_state_boundary",
            "module_loader_service_running_state_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                service_running_state_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "service_start_audit_boundary_missing",
            "denied_missing_module_loader_service_start_audit_boundary",
            "module_loader_service_start_audit_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                service_start_audit_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "service_unload_cleanup_boundary_missing",
            "denied_missing_module_loader_service_unload_cleanup_boundary",
            "module_loader_service_unload_cleanup_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                service_unload_cleanup_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "live_load_commit_boundary_missing",
            "denied_missing_module_loader_live_load_commit_boundary",
            "module_loader_live_load_commit_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                live_load_commit_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "commit_audit_boundary_missing",
            "denied_missing_module_loader_commit_audit_boundary",
            "module_loader_commit_audit_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                commit_audit_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "commit_rollback_boundary_missing",
            "denied_missing_module_loader_commit_rollback_boundary",
            "module_loader_commit_rollback_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                commit_rollback_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "commit_result_boundary_missing",
            "denied_missing_module_loader_commit_result_boundary",
            "module_loader_commit_result_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                commit_result_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "descriptor_acceptance_authority_boundary_missing",
            "denied_missing_module_loader_descriptor_acceptance_authority_boundary",
            "module_loader_descriptor_acceptance_authority_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                descriptor_acceptance_authority_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "descriptor_parser_contract_boundary_missing",
            "denied_missing_module_loader_descriptor_parser_contract_boundary",
            "module_loader_descriptor_parser_contract_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                descriptor_parser_contract_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "descriptor_parser_result_boundary_missing",
            "denied_missing_module_loader_descriptor_parser_result_boundary",
            "module_loader_descriptor_parser_result_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                descriptor_parser_result_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "descriptor_schema_validation_boundary_missing",
            "denied_missing_module_loader_descriptor_schema_validation_boundary",
            "module_loader_descriptor_schema_validation_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                descriptor_schema_validation_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "descriptor_capability_validation_boundary_missing",
            "denied_missing_module_loader_descriptor_capability_validation_boundary",
            "module_loader_descriptor_capability_validation_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                descriptor_capability_validation_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "descriptor_load_plan_boundary_missing",
            "denied_missing_module_loader_descriptor_load_plan_boundary",
            "module_loader_descriptor_load_plan_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                descriptor_load_plan_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "executable_load_plan_authority_boundary_missing",
            "denied_missing_module_loader_executable_load_plan_authority_boundary",
            "module_loader_executable_load_plan_authority_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                executable_load_plan_authority_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "executable_load_plan_result_boundary_missing",
            "denied_missing_module_loader_executable_load_plan_result_boundary",
            "module_loader_executable_load_plan_result_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                executable_load_plan_result_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "executable_image_layout_boundary_missing",
            "denied_missing_module_loader_executable_image_layout_boundary",
            "module_loader_executable_image_layout_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                executable_image_layout_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "executable_page_mapping_plan_boundary_missing",
            "denied_missing_module_loader_executable_page_mapping_plan_boundary",
            "module_loader_executable_page_mapping_plan_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                executable_page_mapping_plan_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "executable_page_mapping_boundary_missing",
            "denied_missing_module_loader_executable_page_mapping_boundary",
            "module_loader_executable_page_mapping_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                executable_page_mapping_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "descriptor_executable_page_binding_boundary_missing",
            "denied_missing_module_loader_descriptor_executable_page_binding_boundary",
            "module_loader_descriptor_executable_page_binding_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                descriptor_executable_page_binding_boundary:
                    module_loader_live_load_boundary_missing(
                        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_METHOD,
                        MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
                    ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "executable_entrypoint_binding_boundary_missing",
            "denied_missing_module_loader_executable_entrypoint_binding_boundary",
            "module_loader_executable_entrypoint_binding_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                executable_entrypoint_binding_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "executable_entrypoint_transfer_authorization_boundary_missing",
            "denied_missing_module_loader_executable_entrypoint_transfer_authorization_boundary",
            "module_loader_executable_entrypoint_transfer_authorization_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                executable_entrypoint_transfer_authorization_boundary:
                    module_loader_live_load_boundary_missing(
                        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_METHOD,
                        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_FACT_LOCATOR,
                    ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "executable_entrypoint_transfer_boundary_missing",
            "denied_missing_module_loader_executable_entrypoint_transfer_boundary",
            "module_loader_executable_entrypoint_transfer_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                executable_entrypoint_transfer_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_FACT_LOCATOR,
                ),
                ..ready
            },
        ),
        module_loader_runtime_selftest_case(
            "executable_entrypoint_handoff_boundary_missing",
            "denied_missing_module_loader_executable_entrypoint_handoff_boundary",
            "module_loader_executable_entrypoint_handoff_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                executable_entrypoint_handoff_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_FACT_LOCATOR,
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
        actual_entrypoint_abi_source_evidence_present: candidate
            .entrypoint_abi
            .source_evidence_event_id
            .is_some(),
        actual_entrypoint_abi_source_evidence_state: candidate.entrypoint_abi.source_evidence_state,
        actual_entrypoint_abi_source_evidence_status: candidate
            .entrypoint_abi
            .source_evidence_status,
        actual_entrypoint_abi_source_evidence_reason: candidate
            .entrypoint_abi
            .source_evidence_reason,
        actual_address_space_source_evidence_present: candidate
            .address_space_boundary
            .source_evidence_event_id
            .is_some(),
        actual_address_space_source_evidence_state: candidate
            .address_space_boundary
            .source_evidence_state,
        actual_address_space_source_evidence_status: candidate
            .address_space_boundary
            .source_evidence_status,
        actual_address_space_source_evidence_reason: candidate
            .address_space_boundary
            .source_evidence_reason,
        actual_memory_map_source_evidence_present: candidate
            .memory_map_constraints
            .source_evidence_event_id
            .is_some(),
        actual_memory_map_source_evidence_state: candidate
            .memory_map_constraints
            .source_evidence_state,
        actual_memory_map_source_evidence_status: candidate
            .memory_map_constraints
            .source_evidence_status,
        actual_memory_map_source_evidence_reason: candidate
            .memory_map_constraints
            .source_evidence_reason,
        actual_capability_table_source_evidence_present: candidate
            .capability_import_table
            .source_evidence_event_id
            .is_some(),
        actual_capability_table_source_evidence_state: candidate
            .capability_import_table
            .source_evidence_state,
        actual_capability_table_source_evidence_status: candidate
            .capability_import_table
            .source_evidence_status,
        actual_capability_table_source_evidence_reason: candidate
            .capability_import_table
            .source_evidence_reason,
        actual_service_slot_source_evidence_present: candidate
            .service_slot_binding
            .source_evidence_event_id
            .is_some(),
        actual_service_slot_source_evidence_state: candidate
            .service_slot_binding
            .source_evidence_state,
        actual_service_slot_source_evidence_status: candidate
            .service_slot_binding
            .source_evidence_status,
        actual_service_slot_source_evidence_reason: candidate
            .service_slot_binding
            .source_evidence_reason,
        actual_health_source_evidence_present: candidate
            .health_state_hooks
            .source_evidence_event_id
            .is_some(),
        actual_health_source_evidence_state: candidate.health_state_hooks.source_evidence_state,
        actual_health_source_evidence_status: candidate.health_state_hooks.source_evidence_status,
        actual_health_source_evidence_reason: candidate.health_state_hooks.source_evidence_reason,
        actual_rollback_source_evidence_present: candidate
            .rollback_hooks
            .source_evidence_event_id
            .is_some(),
        actual_rollback_source_evidence_state: candidate.rollback_hooks.source_evidence_state,
        actual_rollback_source_evidence_status: candidate.rollback_hooks.source_evidence_status,
        actual_rollback_source_evidence_reason: candidate.rollback_hooks.source_evidence_reason,
        actual_write_boundary_source_evidence_present: candidate
            .audit_rollback_write_boundary_binding
            .source_evidence_event_id
            .is_some(),
        actual_write_boundary_source_evidence_state: candidate
            .audit_rollback_write_boundary_binding
            .source_evidence_state,
        actual_write_boundary_source_evidence_status: candidate
            .audit_rollback_write_boundary_binding
            .source_evidence_status,
        actual_write_boundary_source_evidence_reason: candidate
            .audit_rollback_write_boundary_binding
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
