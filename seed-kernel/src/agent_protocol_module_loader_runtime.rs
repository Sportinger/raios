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
    module_loader_runtime_source_fact_map,
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
use self::selftest::module_loader_runtime_selftest_cases;
use self::snapshot::module_loader_runtime_snapshot;
use crate::agent_protocol_module_types::{
    module_loader_runtime_source_fact_map_complete, ModuleLoaderLiveLoadBoundary,
    ModuleLoaderRuntimeFact, MODULE_LOADER_ARTIFACT_LOAD_BOUNDARY_ID,
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
    MODULE_LOADER_RUNTIME_FACT_SOURCES, MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT,
    MODULE_LOADER_SERVICE_HEALTH_BINDING_BOUNDARY_ID,
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
use crate::{agent_protocol_module_reference::emit_evidence_v1_response, event_log};
use alloc::{vec, vec::Vec};
use raios_core::{
    evidence_response::{self as ev, SelftestFacts},
    module_loader_allocator_projection::{
        project_loader_runtime_denial, LoaderAllocatorDisposition, LoaderAllocatorEvidenceInput,
        LoaderAllocatorEvidenceStatus, LoaderRuntimeProjectionInput,
    },
    record::{Field, Value as V},
};

mod eval;
mod evidence_core;
mod evidence_live_load;
mod selftest;
mod snapshot;

fn runtime_evidence<'a>(
    status: &'static str,
    reason: &'a str,
    source_event_id: Option<event_log::EventId>,
    facts: Vec<Field<'a>>,
) -> LoaderAllocatorEvidenceInput<'a> {
    LoaderAllocatorEvidenceInput {
        status: match status {
            "available" => LoaderAllocatorEvidenceStatus::Verified,
            "missing" => LoaderAllocatorEvidenceStatus::Missing,
            "rejected" => LoaderAllocatorEvidenceStatus::Rejected,
            _ => LoaderAllocatorEvidenceStatus::Unavailable,
        },
        status_detail: status,
        reason,
        source_event_sequence: source_event_id.map(event_log::EventId::sequence),
        facts,
        disposition: if status == "available" {
            LoaderAllocatorDisposition::Satisfied
        } else {
            LoaderAllocatorDisposition::Blocked
        },
    }
}

fn runtime_fact<'a>(
    index: usize,
    fact: ModuleLoaderRuntimeFact,
    status: &'static str,
    reason: &'a str,
) -> LoaderAllocatorEvidenceInput<'a> {
    let source = MODULE_LOADER_RUNTIME_FACT_SOURCES[index];
    runtime_evidence(
        status,
        reason,
        fact.source_evidence_event_id,
        vec![
            f("record_schema", s(source.schema)),
            f("record_id", s(source.id)),
            f("source_method", s(source.source_method)),
            f("source_fact_locator", s(source.source_fact_locator)),
            f("source_record_schema", s(fact.source_evidence_schema)),
            f("source_state", s(fact.source_evidence_state)),
            f("source_status_detail", s(fact.source_evidence_status)),
            f("source_reason", s(fact.source_evidence_reason)),
            f("present", b(fact.present)),
            f("schema_valid", b(fact.schema_ok)),
            f("provenance_valid", b(fact.provenance_ok)),
            f(
                "binds_retained_module_evidence",
                b(fact.binds_retained_module_evidence),
            ),
            f(
                "binds_service_slot_allocator",
                b(fact.binds_service_slot_allocator),
            ),
            f(
                "binds_audit_rollback_write_boundary",
                b(fact.binds_audit_rollback_write_boundary),
            ),
        ],
    )
}

fn live_boundary<'a>(
    boundary: ModuleLoaderLiveLoadBoundary,
    status: &'static str,
    reason: &'a str,
) -> LoaderAllocatorEvidenceInput<'a> {
    runtime_evidence(
        status,
        reason,
        boundary.source_evidence_event_id,
        vec![
            f("source_record_schema", s(boundary.source_evidence_schema)),
            f("source_state", s(boundary.source_evidence_state)),
            f("source_status_detail", s(boundary.source_evidence_status)),
            f("source_reason", s(boundary.source_evidence_reason)),
            f("source_method", s(boundary.source_evidence_method)),
            f(
                "source_fact_locator",
                s(boundary.source_evidence_fact_locator),
            ),
            f("present", b(boundary.present)),
            f("source_chain_complete", b(boundary.source_chain_complete)),
        ],
    )
}

macro_rules! runtime_boundary {
    ($boundary:expr, $status:expr, $reason:expr) => {{
        let boundary = $boundary;
        runtime_evidence(
            $status,
            $reason,
            boundary.source_evidence_event_id,
            vec![
                f("source_record_schema", s(boundary.source_evidence_schema)),
                f("source_state", s(boundary.source_evidence_state)),
                f("source_status_detail", s(boundary.source_evidence_status)),
                f("source_reason", s(boundary.source_evidence_reason)),
                f("source_method", s(boundary.source_evidence_method)),
                f(
                    "source_fact_locator",
                    s(boundary.source_evidence_fact_locator),
                ),
                f("present", b(boundary.present)),
                f("source_chain_complete", b(boundary.source_chain_complete)),
            ],
        )
    }};
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

    let projection = project_loader_runtime_denial(LoaderRuntimeProjectionInput {
        manifest_reference: runtime_evidence(
            evaluation.manifest_reference_status,
            evaluation.manifest_reference_reason,
            manifest.as_ref().map(|v| v.0),
            vec![f("present", b(candidate.manifest_reference_present))],
        ),
        artifact_reference: runtime_evidence(
            evaluation.artifact_reference_status,
            evaluation.artifact_reference_reason,
            artifact.as_ref().map(|v| v.0),
            vec![f("present", b(candidate.artifact_reference_present))],
        ),
        vm_report_reference: runtime_evidence(
            evaluation.vm_report_reference_status,
            evaluation.vm_report_reference_reason,
            vm_report.as_ref().map(|v| v.0),
            vec![f("present", b(candidate.vm_report_reference_present))],
        ),
        local_attestation_reference: runtime_evidence(
            evaluation.local_attestation_reference_status,
            evaluation.local_attestation_reference_reason,
            local_attestation.as_ref().map(|v| v.0),
            vec![f(
                "present",
                b(candidate.local_attestation_reference_present),
            )],
        ),
        local_approval_reference: runtime_evidence(
            evaluation.local_approval_reference_status,
            evaluation.local_approval_reference_reason,
            local_approval.as_ref().map(|v| v.0),
            vec![f("present", b(candidate.local_approval_reference_present))],
        ),
        computed_grant_reference: runtime_evidence(
            evaluation.computed_grant_reference_status,
            evaluation.computed_grant_reference_reason,
            computed_grant.as_ref().map(|v| v.0),
            vec![f("present", b(candidate.computed_grant_reference_present))],
        ),
        audit_rollback_reference: runtime_evidence(
            evaluation.audit_rollback_reference_status,
            evaluation.audit_rollback_reference_reason,
            audit_rollback.as_ref().map(|v| v.0),
            vec![f("present", b(candidate.audit_rollback_reference_present))],
        ),
        service_slot_reservation: runtime_evidence(
            evaluation.service_slot_reservation_status,
            evaluation.service_slot_reservation_reason,
            service_slot.as_ref().map(|v| v.0),
            vec![f("present", b(candidate.service_slot_reservation_present))],
        ),
        service_slot_allocator_readiness: runtime_evidence(
            evaluation.service_slot_allocator_readiness_status,
            evaluation.service_slot_allocator_readiness_reason,
            None,
            vec![
                f(
                    "present",
                    b(candidate.service_slot_allocator_readiness_present),
                ),
                f("ready", b(candidate.service_slot_allocator_ready)),
            ],
        ),
        loader_identity: runtime_fact(
            0,
            candidate.loader_identity,
            evaluation.loader_identity_status,
            evaluation.loader_identity_reason,
        ),
        artifact_hash_binding: runtime_fact(
            1,
            candidate.artifact_hash_binding,
            evaluation.artifact_hash_binding_status,
            evaluation.artifact_hash_binding_reason,
        ),
        entrypoint_abi: runtime_fact(
            2,
            candidate.entrypoint_abi,
            evaluation.entrypoint_abi_status,
            evaluation.entrypoint_abi_reason,
        ),
        address_space_boundary: runtime_fact(
            3,
            candidate.address_space_boundary,
            evaluation.address_space_boundary_status,
            evaluation.address_space_boundary_reason,
        ),
        memory_map_constraints: runtime_fact(
            4,
            candidate.memory_map_constraints,
            evaluation.memory_map_constraints_status,
            evaluation.memory_map_constraints_reason,
        ),
        capability_import_table: runtime_fact(
            5,
            candidate.capability_import_table,
            evaluation.capability_import_table_status,
            evaluation.capability_import_table_reason,
        ),
        service_slot_binding: runtime_fact(
            6,
            candidate.service_slot_binding,
            evaluation.service_slot_binding_status,
            evaluation.service_slot_binding_reason,
        ),
        health_state_hooks: runtime_fact(
            7,
            candidate.health_state_hooks,
            evaluation.health_state_hooks_status,
            evaluation.health_state_hooks_reason,
        ),
        rollback_hooks: runtime_fact(
            8,
            candidate.rollback_hooks,
            evaluation.rollback_hooks_status,
            evaluation.rollback_hooks_reason,
        ),
        audit_rollback_write_boundary_binding: runtime_fact(
            9,
            candidate.audit_rollback_write_boundary_binding,
            evaluation.audit_rollback_write_boundary_binding_status,
            evaluation.audit_rollback_write_boundary_binding_reason,
        ),
        execution_commit_gate: runtime_boundary!(
            candidate.execution_commit_gate,
            evaluation.execution_commit_gate_status,
            evaluation.execution_commit_gate_reason
        ),
        descriptor_intake_boundary: runtime_boundary!(
            candidate.descriptor_intake_boundary,
            evaluation.descriptor_intake_boundary_status,
            evaluation.descriptor_intake_boundary_reason
        ),
        artifact_byte_intake_boundary: runtime_boundary!(
            candidate.artifact_byte_intake_boundary,
            evaluation.artifact_byte_intake_boundary_status,
            evaluation.artifact_byte_intake_boundary_reason
        ),
        execution_authorization_boundary: runtime_boundary!(
            candidate.execution_authorization_boundary,
            evaluation.execution_authorization_boundary_status,
            evaluation.execution_authorization_boundary_reason
        ),
        service_registry_mutation_boundary: runtime_boundary!(
            candidate.service_registry_mutation_boundary,
            evaluation.service_registry_mutation_boundary_status,
            evaluation.service_registry_mutation_boundary_reason
        ),
        load_attempt_boundary: live_boundary(
            candidate.load_attempt_boundary,
            evaluation.load_attempt_boundary_status,
            evaluation.load_attempt_boundary_reason,
        ),
        artifact_load_boundary: live_boundary(
            candidate.artifact_load_boundary,
            evaluation.artifact_load_boundary_status,
            evaluation.artifact_load_boundary_reason,
        ),
        executable_mapping_boundary: live_boundary(
            candidate.executable_mapping_boundary,
            evaluation.executable_mapping_boundary_status,
            evaluation.executable_mapping_boundary_reason,
        ),
        entrypoint_transfer_boundary: live_boundary(
            candidate.entrypoint_transfer_boundary,
            evaluation.entrypoint_transfer_boundary_status,
            evaluation.entrypoint_transfer_boundary_reason,
        ),
        service_start_boundary: live_boundary(
            candidate.service_start_boundary,
            evaluation.service_start_boundary_status,
            evaluation.service_start_boundary_reason,
        ),
        service_health_binding_boundary: live_boundary(
            candidate.service_health_binding_boundary,
            evaluation.service_health_binding_boundary_status,
            evaluation.service_health_binding_boundary_reason,
        ),
        service_running_state_boundary: live_boundary(
            candidate.service_running_state_boundary,
            evaluation.service_running_state_boundary_status,
            evaluation.service_running_state_boundary_reason,
        ),
        service_start_audit_boundary: live_boundary(
            candidate.service_start_audit_boundary,
            evaluation.service_start_audit_boundary_status,
            evaluation.service_start_audit_boundary_reason,
        ),
        service_unload_cleanup_boundary: live_boundary(
            candidate.service_unload_cleanup_boundary,
            evaluation.service_unload_cleanup_boundary_status,
            evaluation.service_unload_cleanup_boundary_reason,
        ),
        live_load_commit_boundary: live_boundary(
            candidate.live_load_commit_boundary,
            evaluation.live_load_commit_boundary_status,
            evaluation.live_load_commit_boundary_reason,
        ),
        commit_audit_boundary: live_boundary(
            candidate.commit_audit_boundary,
            evaluation.commit_audit_boundary_status,
            evaluation.commit_audit_boundary_reason,
        ),
        commit_rollback_boundary: live_boundary(
            candidate.commit_rollback_boundary,
            evaluation.commit_rollback_boundary_status,
            evaluation.commit_rollback_boundary_reason,
        ),
        commit_result_boundary: live_boundary(
            candidate.commit_result_boundary,
            evaluation.commit_result_boundary_status,
            evaluation.commit_result_boundary_reason,
        ),
        descriptor_acceptance_authority_boundary: live_boundary(
            candidate.descriptor_acceptance_authority_boundary,
            evaluation.descriptor_acceptance_authority_boundary_status,
            evaluation.descriptor_acceptance_authority_boundary_reason,
        ),
        descriptor_parser_contract_boundary: live_boundary(
            candidate.descriptor_parser_contract_boundary,
            evaluation.descriptor_parser_contract_boundary_status,
            evaluation.descriptor_parser_contract_boundary_reason,
        ),
        descriptor_parser_result_boundary: live_boundary(
            candidate.descriptor_parser_result_boundary,
            evaluation.descriptor_parser_result_boundary_status,
            evaluation.descriptor_parser_result_boundary_reason,
        ),
        descriptor_schema_validation_boundary: live_boundary(
            candidate.descriptor_schema_validation_boundary,
            evaluation.descriptor_schema_validation_boundary_status,
            evaluation.descriptor_schema_validation_boundary_reason,
        ),
        descriptor_capability_validation_boundary: live_boundary(
            candidate.descriptor_capability_validation_boundary,
            evaluation.descriptor_capability_validation_boundary_status,
            evaluation.descriptor_capability_validation_boundary_reason,
        ),
        descriptor_load_plan_boundary: live_boundary(
            candidate.descriptor_load_plan_boundary,
            evaluation.descriptor_load_plan_boundary_status,
            evaluation.descriptor_load_plan_boundary_reason,
        ),
        executable_load_plan_authority_boundary: live_boundary(
            candidate.executable_load_plan_authority_boundary,
            evaluation.executable_load_plan_authority_boundary_status,
            evaluation.executable_load_plan_authority_boundary_reason,
        ),
        executable_load_plan_result_boundary: live_boundary(
            candidate.executable_load_plan_result_boundary,
            evaluation.executable_load_plan_result_boundary_status,
            evaluation.executable_load_plan_result_boundary_reason,
        ),
        executable_image_layout_boundary: live_boundary(
            candidate.executable_image_layout_boundary,
            evaluation.executable_image_layout_boundary_status,
            evaluation.executable_image_layout_boundary_reason,
        ),
        executable_page_mapping_plan_boundary: live_boundary(
            candidate.executable_page_mapping_plan_boundary,
            evaluation.executable_page_mapping_plan_boundary_status,
            evaluation.executable_page_mapping_plan_boundary_reason,
        ),
        executable_page_mapping_boundary: live_boundary(
            candidate.executable_page_mapping_boundary,
            evaluation.executable_page_mapping_boundary_status,
            evaluation.executable_page_mapping_boundary_reason,
        ),
        descriptor_executable_page_binding_boundary: live_boundary(
            candidate.descriptor_executable_page_binding_boundary,
            evaluation.descriptor_executable_page_binding_boundary_status,
            evaluation.descriptor_executable_page_binding_boundary_reason,
        ),
        executable_entrypoint_binding_boundary: live_boundary(
            candidate.executable_entrypoint_binding_boundary,
            evaluation.executable_entrypoint_binding_boundary_status,
            evaluation.executable_entrypoint_binding_boundary_reason,
        ),
        executable_entrypoint_transfer_authorization_boundary: live_boundary(
            candidate.executable_entrypoint_transfer_authorization_boundary,
            evaluation.executable_entrypoint_transfer_authorization_boundary_status,
            evaluation.executable_entrypoint_transfer_authorization_boundary_reason,
        ),
        executable_entrypoint_transfer_boundary: live_boundary(
            candidate.executable_entrypoint_transfer_boundary,
            evaluation.executable_entrypoint_transfer_boundary_status,
            evaluation.executable_entrypoint_transfer_boundary_reason,
        ),
        executable_entrypoint_handoff_boundary: live_boundary(
            candidate.executable_entrypoint_handoff_boundary,
            evaluation.executable_entrypoint_handoff_boundary_status,
            evaluation.executable_entrypoint_handoff_boundary_reason,
        ),
        executable_entrypoint_invocation_boundary: live_boundary(
            candidate.executable_entrypoint_invocation_boundary,
            evaluation.executable_entrypoint_invocation_boundary_status,
            evaluation.executable_entrypoint_invocation_boundary_reason,
        ),
    });
    emit_evidence_v1_response(
        "module.loader_runtime",
        "module.loader_runtime",
        None,
        V::InlineObject(vec![f("test_infrastructure", no())]),
        projection
            .evidence
            .into_iter()
            .map(ev::evidence_value)
            .collect(),
        projection.decision,
    );
}

pub(crate) fn emit_module_loader_runtime_selftest() {
    let cases = module_loader_runtime_selftest_cases();
    let source_fact_map_complete = module_loader_runtime_source_fact_map_complete();
    let passed = cases.iter().all(|case| case.passed) && source_fact_map_complete;
    let values = cases
        .iter()
        .map(|case| {
            ev::selftest_case(
                case.name,
                case.expected_status,
                case.expected_reason,
                case.actual_status,
                case.actual_reason,
                case.passed,
            )
        })
        .collect();
    let facts = ev::selftest_facts_value(SelftestFacts {
        case_count: cases.len() as u64,
        passed,
        safety: ev::selftest_safety_value(),
        cases: V::Array(values),
    });
    let mut fields = match facts {
        V::InlineObject(fields) => fields,
        _ => unreachable!(),
    };
    fields.push(f(
        "source_fact_count",
        V::U64(MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT as u64),
    ));
    fields.push(f("source_fact_map_complete", b(source_fact_map_complete)));
    fields.push(f(
        "source_fact_map",
        module_loader_runtime_source_fact_map(),
    ));
    emit_evidence_v1_response(
        "module.loader_runtime_selftest",
        "module.loader_runtime_selftest",
        None,
        V::InlineObject(fields),
        vec![],
        ev::observed("selftest_completed"),
    );
}
