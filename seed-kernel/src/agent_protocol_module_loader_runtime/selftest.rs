use super::eval::evaluate_module_loader_runtime_candidate;
use super::snapshot::{
    module_loader_artifact_byte_intake_boundary_missing,
    module_loader_descriptor_intake_boundary_missing,
    module_loader_execution_authorization_boundary_missing,
    module_loader_live_load_boundary_missing, module_loader_runtime_execution_commit_gate_missing,
    module_loader_runtime_missing_fact_for,
    module_loader_runtime_observed_artifact_hash_binding_missing_fact,
    module_loader_runtime_observed_entrypoint_abi_missing_fact,
    module_loader_runtime_observed_loader_fact_missing_fact,
    module_loader_runtime_observed_loader_identity_missing_fact,
    module_loader_runtime_ready_snapshot, module_loader_service_registry_mutation_boundary_missing,
};
use crate::agent_protocol_module_types::{
    ModuleLoaderRuntimeCandidate, ModuleLoaderRuntimeFact, ModuleLoaderRuntimeSelfTestCase,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_COMMIT_AUDIT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_COMMIT_RESULT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_COMMIT_ROLLBACK_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_ACCEPTANCE_AUTHORITY_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_CAPABILITY_VALIDATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_EXECUTABLE_PAGE_BINDING_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_LOAD_PLAN_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_PARSER_CONTRACT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_PARSER_RESULT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_DESCRIPTOR_SCHEMA_VALIDATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_BINDING_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_HANDOFF_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_AUTHORIZATION_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_TRANSFER_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_IMAGE_LAYOUT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_AUTHORITY_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_LOAD_PLAN_RESULT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_MAPPING_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_EXECUTABLE_PAGE_MAPPING_PLAN_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_LIVE_LOAD_COMMIT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_LOAD_ATTEMPT_BOUNDARY_SOURCE_METHOD, MODULE_LOADER_RUNTIME_FACT_SOURCES,
    MODULE_LOADER_RUNTIME_SELFTEST_CASES,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_RUNNING_STATE_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_START_AUDIT_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_START_BOUNDARY_SOURCE_METHOD,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_FACT_LOCATOR,
    MODULE_LOADER_SERVICE_UNLOAD_CLEANUP_BOUNDARY_SOURCE_METHOD,
};
use crate::agent_protocol_support::method_eq;

pub(super) fn module_loader_runtime_selftest_cases(
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
            "executable_entrypoint_invocation_boundary_missing",
            "denied_missing_module_loader_executable_entrypoint_invocation_boundary",
            "module_loader_executable_entrypoint_invocation_boundary_source_chain_incomplete",
            ModuleLoaderRuntimeCandidate {
                executable_entrypoint_invocation_boundary: module_loader_live_load_boundary_missing(
                    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_EVIDENCE_SCHEMA,
                    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_METHOD,
                    MODULE_LOADER_EXECUTABLE_ENTRYPOINT_INVOCATION_BOUNDARY_SOURCE_FACT_LOCATOR,
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
