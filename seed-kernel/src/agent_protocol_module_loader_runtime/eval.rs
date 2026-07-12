use crate::agent_protocol_module_types::{
    ModuleLoaderLiveLoadBoundary, ModuleLoaderRuntimeCandidate, ModuleLoaderRuntimeEvaluation,
    ModuleLoaderRuntimeFact, MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_REASON,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_STATUS,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_REASON,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_STATUS,
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_MISSING_STATUS, MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_REASON,
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_STATUS,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_REASON,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_STATUS,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_REASON,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_STATUS,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_REASON,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_STATUS,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_REASON,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_STATUS,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_REASON,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_STATUS,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_REASON,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_STATUS,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_MISSING_STATUS, MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_REASON,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_STATUS,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_MISSING_STATUS,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_REASON,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_STATUS,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_STATUS,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_STATUS,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_STATUS,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_STATUS,
    MODULE_LOADER_SERVICE_START_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_SERVICE_START_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_SERVICE_START_BOUNDARY_STATUS,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_MISSING_STATUS,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_REASON,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_STATUS,
};
use crate::agent_protocol_support::method_eq;

pub(super) fn evaluate_module_loader_runtime_candidate(
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
    let (
        executable_entrypoint_invocation_boundary_status,
        executable_entrypoint_invocation_boundary_reason,
    ) = evaluate_module_loader_live_load_boundary(
        candidate.executable_entrypoint_invocation_boundary,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_REASON,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_MISSING_STATUS,
        MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_CHAIN_INCOMPLETE_REASON,
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
    } else if !candidate.executable_entrypoint_invocation_boundary.present
        || !candidate
            .executable_entrypoint_invocation_boundary
            .source_chain_complete
    {
        (
            "denied_missing_module_loader_executable_entrypoint_invocation_boundary",
            executable_entrypoint_invocation_boundary_reason,
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
        executable_entrypoint_invocation_boundary_status,
        executable_entrypoint_invocation_boundary_reason,
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

pub(super) fn module_loader_runtime_retained_evidence_complete(
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

pub(super) fn module_loader_runtime_facts_complete(
    evaluation: ModuleLoaderRuntimeEvaluation,
) -> bool {
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
