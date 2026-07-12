use super::eval::{
    module_loader_runtime_facts_complete, module_loader_runtime_retained_evidence_complete,
};
use crate::agent_protocol_module_service_slot_allocator_projection::latest_module_service_slot_allocator_readiness_projection;
use crate::agent_protocol_module_types::{
    ModuleLoaderLiveLoadBoundary, ModuleLoaderRuntimeCandidate, ModuleLoaderRuntimeEvaluation,
    ModuleLoaderRuntimeFact, ModuleLoaderRuntimeFactSource,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_ID,
    MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SCHEMA,
    MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_ID, MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SCHEMA,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_ID,
    MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SCHEMA,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_ID,
    MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SCHEMA, MODULE_LOADER_RUNTIME_FACT_SOURCES,
    MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT, MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_ID,
    MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SCHEMA,
    MODULE_SERVICE_SLOT_ALLOCATION_INTENT_ID, MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SCHEMA,
    MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_ID,
    MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SCHEMA,
    MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SCHEMA, MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT,
    MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_ID,
    MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SCHEMA,
};
use crate::agent_protocol_support::method_eq;
use crate::agent_protocol_support::{
    emit_record_property_line, record_bool as b, record_event_or_null, record_false as no,
    record_field as f, record_null as null, record_str as s,
};
use crate::event_log;
use alloc::{vec, vec::Vec};
use raios_core::record::{Field, Value as V};

macro_rules! push_status_reason {
    ($fields:expr, $prefix:literal, $status:expr, $reason:expr $(,)?) => {{
        $fields.push(f(concat!($prefix, "_status"), s($status)));
        $fields.push(f(concat!($prefix, "_reason"), s($reason)));
    }};
}

pub(super) fn emit_module_loader_runtime_retained_evidence(
    manifest_event_id: Option<event_log::EventId>,
    artifact_event_id: Option<event_log::EventId>,
    vm_report_event_id: Option<event_log::EventId>,
    local_attestation_event_id: Option<event_log::EventId>,
    local_approval_event_id: Option<event_log::EventId>,
    computed_grant_event_id: Option<event_log::EventId>,
    audit_rollback_event_id: Option<event_log::EventId>,
    service_slot_event_id: Option<event_log::EventId>,
) {
    emit_record_property_line(
        "retained_module_evidence",
        vec![
            module_loader_runtime_retained_evidence_item(
                "manifest_reference",
                "raios.module_manifest_reference.v0",
                manifest_event_id,
                "retained_module_manifest_reference_available",
                "retained_module_manifest_reference_missing",
            ),
            module_loader_runtime_retained_evidence_item(
                "candidate_artifact_reference",
                "raios.module_candidate_artifact_reference.v0",
                artifact_event_id,
                "retained_module_candidate_artifact_reference_available",
                "retained_module_candidate_artifact_reference_missing",
            ),
            module_loader_runtime_retained_evidence_item(
                "vm_test_report_reference",
                "raios.module_vm_test_report_reference.v0",
                vm_report_event_id,
                "retained_module_vm_test_report_reference_available",
                "retained_module_vm_test_report_reference_missing",
            ),
            module_loader_runtime_retained_evidence_item(
                "local_attestation_reference",
                "raios.module_local_attestation_reference.v0",
                local_attestation_event_id,
                "retained_module_local_attestation_reference_available",
                "retained_module_local_attestation_reference_missing",
            ),
            module_loader_runtime_retained_evidence_item(
                "local_approval_reference",
                "raios.module_local_approval_reference.v0",
                local_approval_event_id,
                "retained_module_local_approval_reference_available",
                "retained_module_local_approval_reference_missing",
            ),
            module_loader_runtime_retained_evidence_item(
                "computed_grant_reference",
                "raios.module_computed_grant_reference.v0",
                computed_grant_event_id,
                "retained_module_computed_grant_reference_available",
                "retained_module_computed_grant_reference_missing",
            ),
            module_loader_runtime_retained_evidence_item(
                "audit_rollback_reference",
                "raios.module_audit_rollback_reference.v0",
                audit_rollback_event_id,
                "retained_module_audit_rollback_reference_available",
                "retained_module_audit_rollback_reference_missing",
            ),
            module_loader_runtime_retained_evidence_item(
                "service_slot_reservation",
                "raios.module_service_slot_reservation.v0",
                service_slot_event_id,
                "retained_module_service_slot_reservation_available",
                "retained_module_service_slot_reservation_missing",
            ),
        ],
        false,
    );
}

fn module_loader_runtime_retained_evidence_item(
    name: &'static str,
    schema: &'static str,
    event_id: Option<event_log::EventId>,
    available_reason: &'static str,
    missing_reason: &'static str,
) -> Field<'static> {
    f(
        name,
        V::Object(vec![
            f("schema", s(schema)),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f(
                "state",
                s(if event_id.is_some() {
                    "present"
                } else {
                    "missing"
                }),
            ),
            f("event_id", record_event_or_null(event_id)),
            f(
                "status",
                s(if event_id.is_some() {
                    "available"
                } else {
                    "missing"
                }),
            ),
            f(
                "reason",
                s(if event_id.is_some() {
                    available_reason
                } else {
                    missing_reason
                }),
            ),
            f("authority", s("retained_hash_reference_only")),
            f("loads_artifact", no()),
            f("allocates_service_slot", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ]),
    )
}

fn push_false_fields(fields: &mut Vec<Field<'static>>, names: &[&'static str]) {
    let mut idx = 0usize;
    while idx < names.len() {
        fields.push(f(names[idx], no()));
        idx += 1;
    }
}

macro_rules! bf {
    ($fields:expr, $key:literal => $value:expr;) => {{
        $fields.push(f($key, b($value)));
    }};
}

fn push_request_target_fields(fields: &mut Vec<Field<'static>>) {
    fields.push(f("requested_capability", s("cap.module.load_ephemeral")));
    fields.push(f("load_mode", s("ram_only")));
    fields.push(f("target", s("live_service_graph")));
}

pub(super) fn module_loader_runtime_header_fields() -> Vec<Field<'static>> {
    let mut fields = vec![
        f("schema", s("raios.module_loader_runtime_readiness.v0")),
        f("scope", s("current_boot")),
        f("classification", s("local_only")),
        f("test_infrastructure", no()),
        f("mutates_global_event_log", b(true)),
        f(
            "global_event_log_mutation",
            s("retained_current_boot_source_evidence_only"),
        ),
    ];
    push_false_fields(
        &mut fields,
        &[
            "accepts_loader_descriptor",
            "accepts_descriptor_bytes",
            "produces_parsed_descriptor",
            "validates_descriptor_schema",
            "produces_validated_descriptor",
            "validates_descriptor_capabilities",
            "produces_capability_validated_descriptor",
            "authorizes_executable_load_plan",
            "produces_executable_load_plan",
            "produces_executable_image_layout",
            "produces_executable_page_mapping_plan",
            "maps_executable_pages",
            "binds_capability_validated_descriptor_to_executable_pages",
            "parses_descriptor_bytes",
            "accepts_artifact_bytes",
            "loads_artifact",
            "allocates_service_slot",
            "creates_service_inventory_records",
        ],
    );
    fields.push(f("service_inventory_change", s("none")));
    push_false_fields(
        &mut fields,
        &[
            "starts_service",
            "marks_service_running",
            "creates_service_health_records",
            "writes_service_start_audit_record",
            "unloads_service",
            "cleans_up_service_slot",
            "commits_live_load",
            "writes_load_commit_audit_record",
            "installs_commit_rollback_record",
            "records_load_result",
            "can_load_now",
            "load_attempted",
            "authorizes_guest_load",
        ],
    );
    fields
}

pub(super) fn module_loader_runtime_policy_result_fields(
    candidate: ModuleLoaderRuntimeCandidate,
    evaluation: ModuleLoaderRuntimeEvaluation,
) -> Vec<Field<'static>> {
    let mut fields = vec![
        f("readiness_status", s(evaluation.status)),
        f("readiness_reason", s(evaluation.reason)),
        f(
            "retained_module_evidence_complete",
            b(module_loader_runtime_retained_evidence_complete(candidate)),
        ),
        f(
            "service_slot_allocator_readiness_present",
            b(candidate.service_slot_allocator_readiness_present),
        ),
        f(
            "service_slot_allocator_ready",
            b(candidate.service_slot_allocator_ready),
        ),
        f(
            "loader_runtime_facts_complete",
            b(module_loader_runtime_facts_complete(evaluation)),
        ),
    ];
    push_status_reason!(
        &mut fields,
        "execution_commit_gate",
        evaluation.execution_commit_gate_status,
        evaluation.execution_commit_gate_reason,
    );
    push_status_reason!(
        &mut fields,
        "descriptor_intake_boundary",
        evaluation.descriptor_intake_boundary_status,
        evaluation.descriptor_intake_boundary_reason,
    );
    push_status_reason!(
        &mut fields,
        "artifact_byte_intake_boundary",
        evaluation.artifact_byte_intake_boundary_status,
        evaluation.artifact_byte_intake_boundary_reason,
    );
    push_status_reason!(
        &mut fields,
        "execution_authorization_boundary",
        evaluation.execution_authorization_boundary_status,
        evaluation.execution_authorization_boundary_reason,
    );
    push_status_reason!(
        &mut fields,
        "service_registry_mutation_boundary",
        evaluation.service_registry_mutation_boundary_status,
        evaluation.service_registry_mutation_boundary_reason,
    );
    push_live_load_status_reason_fields(&mut fields, evaluation);
    push_false_fields(
        &mut fields,
        &[
            "loads_artifact",
            "allocates_service_slot",
            "creates_service_inventory_records",
        ],
    );
    fields.push(f("service_inventory_change", s("none")));
    push_false_fields(
        &mut fields,
        &[
            "starts_service",
            "marks_service_running",
            "creates_service_health_records",
            "writes_service_start_audit_record",
            "unloads_service",
            "cleans_up_service_slot",
            "commits_live_load",
            "writes_load_commit_audit_record",
            "installs_commit_rollback_record",
            "records_load_result",
            "can_load_now",
            "load_attempted",
            "authorizes_guest_load",
        ],
    );
    fields
}

fn push_live_load_status_reason_fields(
    fields: &mut Vec<Field<'static>>,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    push_status_reason!(
        fields,
        "load_attempt_boundary",
        evaluation.load_attempt_boundary_status,
        evaluation.load_attempt_boundary_reason,
    );
    push_status_reason!(
        fields,
        "artifact_load_boundary",
        evaluation.artifact_load_boundary_status,
        evaluation.artifact_load_boundary_reason,
    );
    push_status_reason!(
        fields,
        "executable_mapping_boundary",
        evaluation.executable_mapping_boundary_status,
        evaluation.executable_mapping_boundary_reason,
    );
    push_status_reason!(
        fields,
        "entrypoint_transfer_boundary",
        evaluation.entrypoint_transfer_boundary_status,
        evaluation.entrypoint_transfer_boundary_reason,
    );
    push_status_reason!(
        fields,
        "service_start_boundary",
        evaluation.service_start_boundary_status,
        evaluation.service_start_boundary_reason,
    );
    push_status_reason!(
        fields,
        "service_health_binding_boundary",
        evaluation.service_health_binding_boundary_status,
        evaluation.service_health_binding_boundary_reason,
    );
    push_status_reason!(
        fields,
        "service_running_state_boundary",
        evaluation.service_running_state_boundary_status,
        evaluation.service_running_state_boundary_reason,
    );
    push_status_reason!(
        fields,
        "service_start_audit_boundary",
        evaluation.service_start_audit_boundary_status,
        evaluation.service_start_audit_boundary_reason,
    );
    push_status_reason!(
        fields,
        "service_unload_cleanup_boundary",
        evaluation.service_unload_cleanup_boundary_status,
        evaluation.service_unload_cleanup_boundary_reason,
    );
    push_status_reason!(
        fields,
        "live_load_commit_boundary",
        evaluation.live_load_commit_boundary_status,
        evaluation.live_load_commit_boundary_reason,
    );
    push_status_reason!(
        fields,
        "commit_audit_boundary",
        evaluation.commit_audit_boundary_status,
        evaluation.commit_audit_boundary_reason,
    );
    push_status_reason!(
        fields,
        "commit_rollback_boundary",
        evaluation.commit_rollback_boundary_status,
        evaluation.commit_rollback_boundary_reason,
    );
    push_status_reason!(
        fields,
        "commit_result_boundary",
        evaluation.commit_result_boundary_status,
        evaluation.commit_result_boundary_reason,
    );
    push_status_reason!(
        fields,
        "descriptor_acceptance_authority_boundary",
        evaluation.descriptor_acceptance_authority_boundary_status,
        evaluation.descriptor_acceptance_authority_boundary_reason,
    );
    push_status_reason!(
        fields,
        "descriptor_parser_contract_boundary",
        evaluation.descriptor_parser_contract_boundary_status,
        evaluation.descriptor_parser_contract_boundary_reason,
    );
    push_status_reason!(
        fields,
        "descriptor_parser_result_boundary",
        evaluation.descriptor_parser_result_boundary_status,
        evaluation.descriptor_parser_result_boundary_reason,
    );
    push_status_reason!(
        fields,
        "descriptor_schema_validation_boundary",
        evaluation.descriptor_schema_validation_boundary_status,
        evaluation.descriptor_schema_validation_boundary_reason,
    );
    push_status_reason!(
        fields,
        "descriptor_capability_validation_boundary",
        evaluation.descriptor_capability_validation_boundary_status,
        evaluation.descriptor_capability_validation_boundary_reason,
    );
    push_status_reason!(
        fields,
        "descriptor_load_plan_boundary",
        evaluation.descriptor_load_plan_boundary_status,
        evaluation.descriptor_load_plan_boundary_reason,
    );
    push_status_reason!(
        fields,
        "executable_load_plan_authority_boundary",
        evaluation.executable_load_plan_authority_boundary_status,
        evaluation.executable_load_plan_authority_boundary_reason,
    );
    push_status_reason!(
        fields,
        "executable_load_plan_result_boundary",
        evaluation.executable_load_plan_result_boundary_status,
        evaluation.executable_load_plan_result_boundary_reason,
    );
    push_status_reason!(
        fields,
        "executable_image_layout_boundary",
        evaluation.executable_image_layout_boundary_status,
        evaluation.executable_image_layout_boundary_reason,
    );
    push_status_reason!(
        fields,
        "executable_page_mapping_plan_boundary",
        evaluation.executable_page_mapping_plan_boundary_status,
        evaluation.executable_page_mapping_plan_boundary_reason,
    );
    push_status_reason!(
        fields,
        "executable_page_mapping_boundary",
        evaluation.executable_page_mapping_boundary_status,
        evaluation.executable_page_mapping_boundary_reason,
    );
    push_status_reason!(
        fields,
        "descriptor_executable_page_binding_boundary",
        evaluation.descriptor_executable_page_binding_boundary_status,
        evaluation.descriptor_executable_page_binding_boundary_reason,
    );
    push_status_reason!(
        fields,
        "executable_entrypoint_binding_boundary",
        evaluation.executable_entrypoint_binding_boundary_status,
        evaluation.executable_entrypoint_binding_boundary_reason,
    );
    push_status_reason!(
        fields,
        "executable_entrypoint_transfer_authorization_boundary",
        evaluation.executable_entrypoint_transfer_authorization_boundary_status,
        evaluation.executable_entrypoint_transfer_authorization_boundary_reason,
    );
    push_status_reason!(
        fields,
        "executable_entrypoint_transfer_boundary",
        evaluation.executable_entrypoint_transfer_boundary_status,
        evaluation.executable_entrypoint_transfer_boundary_reason,
    );
    push_status_reason!(
        fields,
        "executable_entrypoint_handoff_boundary",
        evaluation.executable_entrypoint_handoff_boundary_status,
        evaluation.executable_entrypoint_handoff_boundary_reason,
    );
    push_status_reason!(
        fields,
        "executable_entrypoint_invocation_boundary",
        evaluation.executable_entrypoint_invocation_boundary_status,
        evaluation.executable_entrypoint_invocation_boundary_reason,
    );
}

pub(super) fn module_loader_runtime_blocked_by(
    evaluation: ModuleLoaderRuntimeEvaluation,
) -> V<'static> {
    let mut values = Vec::new();
    push_block_gate(
        &mut values,
        "retained_module_manifest_reference",
        evaluation.manifest_reference_status,
        evaluation.manifest_reference_reason,
    );
    push_block_gate(
        &mut values,
        "retained_module_candidate_artifact_reference",
        evaluation.artifact_reference_status,
        evaluation.artifact_reference_reason,
    );
    push_block_gate(
        &mut values,
        "retained_module_vm_test_report_reference",
        evaluation.vm_report_reference_status,
        evaluation.vm_report_reference_reason,
    );
    push_block_gate(
        &mut values,
        "retained_module_local_attestation_reference",
        evaluation.local_attestation_reference_status,
        evaluation.local_attestation_reference_reason,
    );
    push_block_gate(
        &mut values,
        "retained_module_local_approval_reference",
        evaluation.local_approval_reference_status,
        evaluation.local_approval_reference_reason,
    );
    push_block_gate(
        &mut values,
        "retained_module_computed_grant_reference",
        evaluation.computed_grant_reference_status,
        evaluation.computed_grant_reference_reason,
    );
    push_block_gate(
        &mut values,
        "retained_module_audit_rollback_reference",
        evaluation.audit_rollback_reference_status,
        evaluation.audit_rollback_reference_reason,
    );
    push_block_gate(
        &mut values,
        "retained_module_service_slot_reservation",
        evaluation.service_slot_reservation_status,
        evaluation.service_slot_reservation_reason,
    );
    push_block_gate(
        &mut values,
        "service_slot_allocator_readiness",
        evaluation.service_slot_allocator_readiness_status,
        evaluation.service_slot_allocator_readiness_reason,
    );
    push_block_gate(
        &mut values,
        "service_slot_allocator_runtime",
        evaluation.service_slot_allocator_runtime_status,
        evaluation.service_slot_allocator_runtime_reason,
    );
    push_block_fact_gate(
        &mut values,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[0],
        evaluation.loader_identity_status,
        evaluation.loader_identity_reason,
    );
    push_block_fact_gate(
        &mut values,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[1],
        evaluation.artifact_hash_binding_status,
        evaluation.artifact_hash_binding_reason,
    );
    push_block_fact_gate(
        &mut values,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[2],
        evaluation.entrypoint_abi_status,
        evaluation.entrypoint_abi_reason,
    );
    push_block_fact_gate(
        &mut values,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[3],
        evaluation.address_space_boundary_status,
        evaluation.address_space_boundary_reason,
    );
    push_block_fact_gate(
        &mut values,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[4],
        evaluation.memory_map_constraints_status,
        evaluation.memory_map_constraints_reason,
    );
    push_block_fact_gate(
        &mut values,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[5],
        evaluation.capability_import_table_status,
        evaluation.capability_import_table_reason,
    );
    push_block_fact_gate(
        &mut values,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[6],
        evaluation.service_slot_binding_status,
        evaluation.service_slot_binding_reason,
    );
    push_block_fact_gate(
        &mut values,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[7],
        evaluation.health_state_hooks_status,
        evaluation.health_state_hooks_reason,
    );
    push_block_fact_gate(
        &mut values,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[8],
        evaluation.rollback_hooks_status,
        evaluation.rollback_hooks_reason,
    );
    push_block_fact_gate(
        &mut values,
        MODULE_LOADER_RUNTIME_FACT_SOURCES[9],
        evaluation.audit_rollback_write_boundary_binding_status,
        evaluation.audit_rollback_write_boundary_binding_reason,
    );
    push_block_gate(
        &mut values,
        "execution_commit_gate",
        evaluation.execution_commit_gate_status,
        evaluation.execution_commit_gate_reason,
    );
    push_block_gate(
        &mut values,
        "descriptor_intake_boundary",
        evaluation.descriptor_intake_boundary_status,
        evaluation.descriptor_intake_boundary_reason,
    );
    push_block_gate(
        &mut values,
        "artifact_byte_intake_boundary",
        evaluation.artifact_byte_intake_boundary_status,
        evaluation.artifact_byte_intake_boundary_reason,
    );
    push_block_gate(
        &mut values,
        "execution_authorization_boundary",
        evaluation.execution_authorization_boundary_status,
        evaluation.execution_authorization_boundary_reason,
    );
    push_block_gate(
        &mut values,
        "service_registry_mutation_boundary",
        evaluation.service_registry_mutation_boundary_status,
        evaluation.service_registry_mutation_boundary_reason,
    );
    push_live_load_block_gates(&mut values, evaluation);
    V::Array(values)
}

fn push_block_gate(
    values: &mut Vec<V<'static>>,
    gate: &'static str,
    state: &'static str,
    reason: &'static str,
) {
    values.push(V::InlineObject(vec![
        f("gate", s(gate)),
        f("state", s(state)),
        f("reason", s(reason)),
    ]));
}

fn push_block_fact_gate(
    values: &mut Vec<V<'static>>,
    source: ModuleLoaderRuntimeFactSource,
    state: &'static str,
    reason: &'static str,
) {
    values.push(V::InlineObject(vec![
        f("gate", s(source.name)),
        f("state", s(state)),
        f("reason", s(reason)),
        f("schema", s(source.schema)),
        f("fact_id", s(source.id)),
        f("source_method", s(source.source_method)),
        f("source_fact_locator", s(source.source_fact_locator)),
    ]));
}

fn push_live_load_block_gates(
    values: &mut Vec<V<'static>>,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    push_block_gate(
        values,
        "load_attempt_boundary",
        evaluation.load_attempt_boundary_status,
        evaluation.load_attempt_boundary_reason,
    );
    push_block_gate(
        values,
        "artifact_load_boundary",
        evaluation.artifact_load_boundary_status,
        evaluation.artifact_load_boundary_reason,
    );
    push_block_gate(
        values,
        "executable_mapping_boundary",
        evaluation.executable_mapping_boundary_status,
        evaluation.executable_mapping_boundary_reason,
    );
    push_block_gate(
        values,
        "entrypoint_transfer_boundary",
        evaluation.entrypoint_transfer_boundary_status,
        evaluation.entrypoint_transfer_boundary_reason,
    );
    push_block_gate(
        values,
        "service_start_boundary",
        evaluation.service_start_boundary_status,
        evaluation.service_start_boundary_reason,
    );
    push_block_gate(
        values,
        "service_health_binding_boundary",
        evaluation.service_health_binding_boundary_status,
        evaluation.service_health_binding_boundary_reason,
    );
    push_block_gate(
        values,
        "service_running_state_boundary",
        evaluation.service_running_state_boundary_status,
        evaluation.service_running_state_boundary_reason,
    );
    push_block_gate(
        values,
        "service_start_audit_boundary",
        evaluation.service_start_audit_boundary_status,
        evaluation.service_start_audit_boundary_reason,
    );
    push_block_gate(
        values,
        "service_unload_cleanup_boundary",
        evaluation.service_unload_cleanup_boundary_status,
        evaluation.service_unload_cleanup_boundary_reason,
    );
    push_block_gate(
        values,
        "live_load_commit_boundary",
        evaluation.live_load_commit_boundary_status,
        evaluation.live_load_commit_boundary_reason,
    );
    push_block_gate(
        values,
        "commit_audit_boundary",
        evaluation.commit_audit_boundary_status,
        evaluation.commit_audit_boundary_reason,
    );
    push_block_gate(
        values,
        "commit_rollback_boundary",
        evaluation.commit_rollback_boundary_status,
        evaluation.commit_rollback_boundary_reason,
    );
    push_block_gate(
        values,
        "commit_result_boundary",
        evaluation.commit_result_boundary_status,
        evaluation.commit_result_boundary_reason,
    );
    push_block_gate(
        values,
        "descriptor_acceptance_authority_boundary",
        evaluation.descriptor_acceptance_authority_boundary_status,
        evaluation.descriptor_acceptance_authority_boundary_reason,
    );
    push_block_gate(
        values,
        "descriptor_parser_contract_boundary",
        evaluation.descriptor_parser_contract_boundary_status,
        evaluation.descriptor_parser_contract_boundary_reason,
    );
    push_block_gate(
        values,
        "descriptor_parser_result_boundary",
        evaluation.descriptor_parser_result_boundary_status,
        evaluation.descriptor_parser_result_boundary_reason,
    );
    push_block_gate(
        values,
        "descriptor_schema_validation_boundary",
        evaluation.descriptor_schema_validation_boundary_status,
        evaluation.descriptor_schema_validation_boundary_reason,
    );
    push_block_gate(
        values,
        "descriptor_capability_validation_boundary",
        evaluation.descriptor_capability_validation_boundary_status,
        evaluation.descriptor_capability_validation_boundary_reason,
    );
    push_block_gate(
        values,
        "descriptor_load_plan_boundary",
        evaluation.descriptor_load_plan_boundary_status,
        evaluation.descriptor_load_plan_boundary_reason,
    );
    push_block_gate(
        values,
        "executable_load_plan_authority_boundary",
        evaluation.executable_load_plan_authority_boundary_status,
        evaluation.executable_load_plan_authority_boundary_reason,
    );
    push_block_gate(
        values,
        "executable_load_plan_result_boundary",
        evaluation.executable_load_plan_result_boundary_status,
        evaluation.executable_load_plan_result_boundary_reason,
    );
    push_block_gate(
        values,
        "executable_image_layout_boundary",
        evaluation.executable_image_layout_boundary_status,
        evaluation.executable_image_layout_boundary_reason,
    );
    push_block_gate(
        values,
        "executable_page_mapping_plan_boundary",
        evaluation.executable_page_mapping_plan_boundary_status,
        evaluation.executable_page_mapping_plan_boundary_reason,
    );
    push_block_gate(
        values,
        "executable_page_mapping_boundary",
        evaluation.executable_page_mapping_boundary_status,
        evaluation.executable_page_mapping_boundary_reason,
    );
    push_block_gate(
        values,
        "descriptor_executable_page_binding_boundary",
        evaluation.descriptor_executable_page_binding_boundary_status,
        evaluation.descriptor_executable_page_binding_boundary_reason,
    );
    push_block_gate(
        values,
        "executable_entrypoint_binding_boundary",
        evaluation.executable_entrypoint_binding_boundary_status,
        evaluation.executable_entrypoint_binding_boundary_reason,
    );
    push_block_gate(
        values,
        "executable_entrypoint_transfer_authorization_boundary",
        evaluation.executable_entrypoint_transfer_authorization_boundary_status,
        evaluation.executable_entrypoint_transfer_authorization_boundary_reason,
    );
    push_block_gate(
        values,
        "executable_entrypoint_transfer_boundary",
        evaluation.executable_entrypoint_transfer_boundary_status,
        evaluation.executable_entrypoint_transfer_boundary_reason,
    );
    push_block_gate(
        values,
        "executable_entrypoint_handoff_boundary",
        evaluation.executable_entrypoint_handoff_boundary_status,
        evaluation.executable_entrypoint_handoff_boundary_reason,
    );
    push_block_gate(
        values,
        "executable_entrypoint_invocation_boundary",
        evaluation.executable_entrypoint_invocation_boundary_status,
        evaluation.executable_entrypoint_invocation_boundary_reason,
    );
}

fn module_loader_runtime_source_evidence_array(
    event_ids: [Option<event_log::EventId>; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    source_present: [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
    fact_present: [bool; MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT],
) -> V<'static> {
    let mut values = Vec::new();
    let mut idx = 0usize;
    while idx < MODULE_LOADER_RUNTIME_FACT_SOURCE_COUNT {
        let source = MODULE_LOADER_RUNTIME_FACT_SOURCES[idx];
        values.push(V::InlineObject(vec![
            f("fact", s(source.name)),
            f("schema", s(source.schema)),
            f(
                "source_evidence_event_id",
                record_event_or_null(event_ids[idx]),
            ),
            f("source_evidence_present", b(source_present[idx])),
            f("fact_present", b(fact_present[idx])),
        ]));
        idx += 1;
    }
    V::Array(values)
}

fn module_loader_runtime_common_boundary_fields(
    schema: &'static str,
    id: &'static str,
    source_evidence_schema: &'static str,
    source_evidence_event_id: Option<event_log::EventId>,
    source_evidence_state: &'static str,
    source_evidence_status: &'static str,
    source_evidence_reason: &'static str,
    source_method: &'static str,
    source_fact_locator: &'static str,
    status: &'static str,
    reason: &'static str,
    present: bool,
    source_chain_complete: bool,
) -> Vec<Field<'static>> {
    vec![
        f("schema", s(schema)),
        f("id", s(id)),
        f("source_evidence_schema", s(source_evidence_schema)),
        f(
            "source_evidence_event_id",
            record_event_or_null(source_evidence_event_id),
        ),
        f("source_evidence_state", s(source_evidence_state)),
        f("source_evidence_status", s(source_evidence_status)),
        f("source_evidence_reason", s(source_evidence_reason)),
        f("source_method", s(source_method)),
        f("source_fact_locator", s(source_fact_locator)),
        f("status", s(status)),
        f("reason", s(reason)),
        f("present", b(present)),
        f("source_chain_complete", b(source_chain_complete)),
    ]
}

fn push_live_load_presence_fields(
    fields: &mut Vec<Field<'static>>,
    boundary: ModuleLoaderLiveLoadBoundary,
) {
    bf! { fields, "load_attempt_boundary_present" => boundary.load_attempt_boundary_present; }
    bf! { fields, "load_attempt_boundary_source_chain_complete" => boundary.load_attempt_boundary_source_chain_complete; }
    bf! { fields, "artifact_load_boundary_present" => boundary.artifact_load_boundary_present; }
    bf! { fields, "artifact_load_boundary_source_chain_complete" => boundary.artifact_load_boundary_source_chain_complete; }
    bf! { fields, "executable_mapping_boundary_present" => boundary.executable_mapping_boundary_present; }
    bf! { fields, "executable_mapping_boundary_source_chain_complete" => boundary.executable_mapping_boundary_source_chain_complete; }
    bf! { fields, "entrypoint_transfer_boundary_present" => boundary.entrypoint_transfer_boundary_present; }
    bf! { fields, "entrypoint_transfer_boundary_source_chain_complete" => boundary.entrypoint_transfer_boundary_source_chain_complete; }
    bf! { fields, "service_start_boundary_present" => boundary.service_start_boundary_present; }
    bf! { fields, "service_start_boundary_source_chain_complete" => boundary.service_start_boundary_source_chain_complete; }
    bf! { fields, "service_health_binding_boundary_present" => boundary.service_health_binding_boundary_present; }
    bf! { fields, "service_health_binding_boundary_source_chain_complete" => boundary.service_health_binding_boundary_source_chain_complete; }
    bf! { fields, "service_running_state_boundary_present" => boundary.service_running_state_boundary_present; }
    bf! { fields, "service_running_state_boundary_source_chain_complete" => boundary.service_running_state_boundary_source_chain_complete; }
    bf! { fields, "service_start_audit_boundary_present" => boundary.service_start_audit_boundary_present; }
    bf! { fields, "service_start_audit_boundary_source_chain_complete" => boundary.service_start_audit_boundary_source_chain_complete; }
    bf! { fields, "service_unload_cleanup_boundary_present" => boundary.service_unload_cleanup_boundary_present; }
    bf! { fields, "service_unload_cleanup_boundary_source_chain_complete" => boundary.service_unload_cleanup_boundary_source_chain_complete; }
    bf! { fields, "live_load_commit_boundary_present" => boundary.live_load_commit_boundary_present; }
    bf! { fields, "live_load_commit_boundary_source_chain_complete" => boundary.live_load_commit_boundary_source_chain_complete; }
    bf! { fields, "commit_audit_boundary_present" => boundary.commit_audit_boundary_present; }
    bf! { fields, "commit_audit_boundary_source_chain_complete" => boundary.commit_audit_boundary_source_chain_complete; }
    bf! { fields, "commit_rollback_boundary_present" => boundary.commit_rollback_boundary_present; }
    bf! { fields, "commit_rollback_boundary_source_chain_complete" => boundary.commit_rollback_boundary_source_chain_complete; }
    bf! { fields, "commit_result_boundary_present" => boundary.commit_result_boundary_present; }
    bf! { fields, "commit_result_boundary_source_chain_complete" => boundary.commit_result_boundary_source_chain_complete; }
    bf! { fields, "descriptor_acceptance_authority_boundary_present" => boundary.descriptor_acceptance_authority_boundary_present; }
    bf! { fields, "descriptor_acceptance_authority_boundary_source_chain_complete" => boundary.descriptor_acceptance_authority_boundary_source_chain_complete; }
    bf! { fields, "descriptor_parser_contract_boundary_present" => boundary.descriptor_parser_contract_boundary_present; }
    bf! { fields, "descriptor_parser_contract_boundary_source_chain_complete" => boundary.descriptor_parser_contract_boundary_source_chain_complete; }
    bf! { fields, "descriptor_parser_result_boundary_present" => boundary.descriptor_parser_result_boundary_present; }
    bf! { fields, "descriptor_parser_result_boundary_source_chain_complete" => boundary.descriptor_parser_result_boundary_source_chain_complete; }
    bf! { fields, "descriptor_schema_validation_boundary_present" => boundary.descriptor_schema_validation_boundary_present; }
    bf! { fields, "descriptor_schema_validation_boundary_source_chain_complete" => boundary.descriptor_schema_validation_boundary_source_chain_complete; }
    bf! { fields, "descriptor_capability_validation_boundary_present" => boundary.descriptor_capability_validation_boundary_present; }
    bf! { fields, "descriptor_capability_validation_boundary_source_chain_complete" => boundary.descriptor_capability_validation_boundary_source_chain_complete; }
    bf! { fields, "descriptor_load_plan_boundary_present" => boundary.descriptor_load_plan_boundary_present; }
    bf! { fields, "descriptor_load_plan_boundary_source_chain_complete" => boundary.descriptor_load_plan_boundary_source_chain_complete; }
    bf! { fields, "executable_load_plan_authority_boundary_present" => boundary.executable_load_plan_authority_boundary_present; }
    bf! { fields, "executable_load_plan_authority_boundary_source_chain_complete" => boundary.executable_load_plan_authority_boundary_source_chain_complete; }
    bf! { fields, "executable_load_plan_result_boundary_present" => boundary.executable_load_plan_result_boundary_present; }
    bf! { fields, "executable_load_plan_result_boundary_source_chain_complete" => boundary.executable_load_plan_result_boundary_source_chain_complete; }
    bf! { fields, "executable_image_layout_boundary_present" => boundary.executable_image_layout_boundary_present; }
    bf! { fields, "executable_image_layout_boundary_source_chain_complete" => boundary.executable_image_layout_boundary_source_chain_complete; }
    bf! { fields, "executable_page_mapping_plan_boundary_present" => boundary.executable_page_mapping_plan_boundary_present; }
    bf! { fields, "executable_page_mapping_plan_boundary_source_chain_complete" => boundary.executable_page_mapping_plan_boundary_source_chain_complete; }
    bf! { fields, "executable_page_mapping_boundary_present" => boundary.executable_page_mapping_boundary_present; }
    bf! { fields, "executable_page_mapping_boundary_source_chain_complete" => boundary.executable_page_mapping_boundary_source_chain_complete; }
    bf! { fields, "descriptor_executable_page_binding_boundary_present" => boundary.descriptor_executable_page_binding_boundary_present; }
    bf! { fields, "descriptor_executable_page_binding_boundary_source_chain_complete" => boundary.descriptor_executable_page_binding_boundary_source_chain_complete; }
    bf! { fields, "executable_entrypoint_binding_boundary_present" => boundary.executable_entrypoint_binding_boundary_present; }
    bf! { fields, "executable_entrypoint_binding_boundary_source_chain_complete" => boundary.executable_entrypoint_binding_boundary_source_chain_complete; }
    bf! { fields, "executable_entrypoint_transfer_authorization_boundary_present" => boundary.executable_entrypoint_transfer_authorization_boundary_present; }
    bf! { fields, "executable_entrypoint_transfer_authorization_boundary_source_chain_complete" => boundary.executable_entrypoint_transfer_authorization_boundary_source_chain_complete; }
    bf! { fields, "executable_entrypoint_transfer_boundary_present" => boundary.executable_entrypoint_transfer_boundary_present; }
    bf! { fields, "executable_entrypoint_transfer_boundary_source_chain_complete" => boundary.executable_entrypoint_transfer_boundary_source_chain_complete; }
    bf! { fields, "executable_entrypoint_handoff_boundary_present" => boundary.executable_entrypoint_handoff_boundary_present; }
    bf! { fields, "executable_entrypoint_handoff_boundary_source_chain_complete" => boundary.executable_entrypoint_handoff_boundary_source_chain_complete; }
    bf! { fields, "artifact_byte_intake_boundary_present" => boundary.artifact_byte_intake_boundary_present; }
    bf! { fields, "artifact_byte_intake_boundary_source_chain_complete" => boundary.artifact_byte_intake_boundary_source_chain_complete; }
    bf! { fields, "execution_authorization_boundary_present" => boundary.execution_authorization_boundary_present; }
    bf! { fields, "execution_authorization_boundary_source_chain_complete" => boundary.execution_authorization_boundary_source_chain_complete; }
    bf! { fields, "service_registry_mutation_boundary_present" => boundary.service_registry_mutation_boundary_present; }
    bf! { fields, "service_registry_mutation_boundary_source_chain_complete" => boundary.service_registry_mutation_boundary_source_chain_complete; }
    bf! { fields, "service_slot_binding_source_evidence_present" => boundary.service_slot_binding_source_evidence_present; }
    bf! { fields, "health_state_hooks_source_evidence_present" => boundary.health_state_hooks_source_evidence_present; }
    bf! { fields, "artifact_hash_binding_present" => boundary.artifact_hash_binding_present; }
    bf! { fields, "entrypoint_abi_source_evidence_present" => boundary.entrypoint_abi_source_evidence_present; }
    bf! { fields, "address_space_source_evidence_present" => boundary.address_space_source_evidence_present; }
    bf! { fields, "memory_map_source_evidence_present" => boundary.memory_map_source_evidence_present; }
    bf! { fields, "capability_import_table_source_evidence_present" => boundary.capability_import_table_source_evidence_present; }
    bf! { fields, "audit_rollback_write_boundary_source_evidence_present" => boundary.audit_rollback_write_boundary_source_evidence_present; }
    bf! { fields, "retained_module_evidence_present" => boundary.retained_module_evidence_present; }
    bf! { fields, "retained_artifact_reference_present" => boundary.retained_artifact_reference_present; }
    bf! { fields, "retained_service_slot_reservation_present" => boundary.retained_service_slot_reservation_present; }
}

pub(super) fn emit_module_loader_runtime_service_slot_allocator_readiness(
    candidate: ModuleLoaderRuntimeCandidate,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    let service_slot = event_log::latest_module_service_slot_reservation();
    let allocator_projection = latest_module_service_slot_allocator_readiness_projection(
        service_slot.as_ref().map(|(event_id, _)| *event_id),
    );
    let mut authority_inputs = Vec::new();
    let mut input_idx = 0usize;
    while input_idx < MODULE_SERVICE_SLOT_AUTHORITY_INPUT_COUNT {
        let input = allocator_projection.authority_inputs[input_idx];
        authority_inputs.push(f(
            input.name,
            V::Object(vec![
                f("schema", s(input.schema)),
                f(
                    "source_evidence_event_id",
                    record_event_or_null(input.source_evidence_event_id),
                ),
                f("status", s(input.status)),
                f("reason", s(input.reason)),
                f("present", b(input.present)),
                f("allocates_service_slot", no()),
                f("creates_service_inventory_records", no()),
                f("service_inventory_change", s("none")),
                f("load_attempted", no()),
            ]),
        ));
        input_idx += 1;
    }
    emit_record_property_line(
        "service_slot_allocator_readiness",
        vec![
            f(
                "schema",
                s("raios.module_service_slot_allocator_readiness.v0"),
            ),
            f("scope", s("current_boot")),
            f("classification", s("local_only")),
            f("source_method", s("module.service_slot_allocator")),
            f("state", s("read_only_diagnostic_defined")),
            f(
                "retained_service_slot_reservation_present",
                b(candidate.service_slot_reservation_present),
            ),
            f(
                "readiness_present",
                b(candidate.service_slot_allocator_readiness_present),
            ),
            f(
                "readiness_status",
                s(evaluation.service_slot_allocator_readiness_status),
            ),
            f(
                "readiness_reason",
                s(evaluation.service_slot_allocator_readiness_reason),
            ),
            f(
                "allocator_authority_boundary",
                V::Object(vec![
                    f("schema", s(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_SCHEMA)),
                    f(
                        "source_evidence_event_id",
                        record_event_or_null(
                            allocator_projection.authority_source_evidence_event_id,
                        ),
                    ),
                    f("status", s(allocator_projection.authority_status)),
                    f("reason", s(allocator_projection.authority_reason)),
                    f("present", b(allocator_projection.authority_present)),
                    f("allocates_service_slot", no()),
                    f("creates_service_inventory_records", no()),
                    f("service_inventory_change", s("none")),
                    f("load_attempted", no()),
                ]),
            ),
            f(
                "allocation_intent_boundary",
                V::Object(vec![
                    f("schema", s(MODULE_SERVICE_SLOT_ALLOCATION_INTENT_SCHEMA)),
                    f("id", s(MODULE_SERVICE_SLOT_ALLOCATION_INTENT_ID)),
                    f(
                        "source_evidence_event_id",
                        record_event_or_null(
                            allocator_projection.allocation_intent_source_evidence_event_id,
                        ),
                    ),
                    f("status", s(allocator_projection.allocation_intent_status)),
                    f("reason", s(allocator_projection.allocation_intent_reason)),
                    f("present", b(allocator_projection.allocation_intent_present)),
                    f("requested_capability", s("cap.module.load_ephemeral")),
                    f("load_mode", s("ram_only")),
                    f("target", s("live_service_graph")),
                    f("allocates_service_slot", no()),
                    f("creates_service_inventory_records", no()),
                    f("service_inventory_change", s("none")),
                    f("load_attempted", no()),
                ]),
            ),
            f("authority_input_boundaries", V::Object(authority_inputs)),
            f(
                "authority_decision",
                V::Object(vec![
                    f(
                        "schema",
                        s(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_SCHEMA),
                    ),
                    f("id", s(MODULE_SERVICE_SLOT_ALLOCATOR_AUTHORITY_DECISION_ID)),
                    f(
                        "source_evidence_event_id",
                        record_event_or_null(
                            allocator_projection.authority_decision_source_evidence_event_id,
                        ),
                    ),
                    f("status", s(allocator_projection.authority_decision_status)),
                    f("reason", s(allocator_projection.authority_decision_reason)),
                    f(
                        "present",
                        b(allocator_projection.authority_decision_present),
                    ),
                    f("requested_capability", s("cap.module.load_ephemeral")),
                    f("load_mode", s("ram_only")),
                    f("target", s("live_service_graph")),
                    f("authorizes_allocation", no()),
                    f("authorizes_load", no()),
                    f("allocates_service_slot", no()),
                    f("creates_service_inventory_records", no()),
                    f("service_inventory_change", s("none")),
                    f("load_attempted", no()),
                ]),
            ),
            f(
                "registry_write_commit_gate",
                V::Object(vec![
                    f(
                        "schema",
                        s(MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_SCHEMA),
                    ),
                    f("id", s(MODULE_SERVICE_SLOT_REGISTRY_WRITE_COMMIT_GATE_ID)),
                    f(
                        "source_evidence_event_id",
                        record_event_or_null(
                            allocator_projection
                                .registry_write_commit_gate_source_evidence_event_id,
                        ),
                    ),
                    f(
                        "status",
                        s(allocator_projection.registry_write_commit_gate_status),
                    ),
                    f(
                        "reason",
                        s(allocator_projection.registry_write_commit_gate_reason),
                    ),
                    f(
                        "present",
                        b(allocator_projection.registry_write_commit_gate_present),
                    ),
                    f("requested_capability", s("cap.module.load_ephemeral")),
                    f("load_mode", s("ram_only")),
                    f("target", s("live_service_graph")),
                    f("authorizes_registry_write", no()),
                    f("mutates_service_registry", no()),
                    f("writes_durable_audit_state", no()),
                    f("installs_rollback_state", no()),
                    f("authorizes_allocation", no()),
                    f("authorizes_load", no()),
                    f("allocates_service_slot", no()),
                    f("creates_service_inventory_records", no()),
                    f("service_inventory_change", s("none")),
                    f("loads_artifact", no()),
                    f("load_attempted", no()),
                ]),
            ),
            f(
                "runtime_status",
                s(evaluation.service_slot_allocator_runtime_status),
            ),
            f(
                "runtime_reason",
                s(evaluation.service_slot_allocator_runtime_reason),
            ),
            f(
                "service_slot_allocator_ready",
                b(candidate.service_slot_allocator_ready),
            ),
            f("allocates_service_slot", no()),
            f("creates_service_inventory_records", no()),
            f("service_inventory_change", s("none")),
            f("can_load_now", no()),
            f("load_attempted", no()),
        ],
        false,
    );
}

pub(super) fn emit_module_loader_runtime_execution_commit_gate(
    candidate: ModuleLoaderRuntimeCandidate,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    let gate = candidate.execution_commit_gate;
    let mut fields = module_loader_runtime_common_boundary_fields(
        MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_SCHEMA,
        MODULE_LOADER_RUNTIME_EXECUTION_COMMIT_GATE_ID,
        gate.source_evidence_schema,
        gate.source_evidence_event_id,
        gate.source_evidence_state,
        gate.source_evidence_status,
        gate.source_evidence_reason,
        gate.source_evidence_method,
        gate.source_evidence_fact_locator,
        evaluation.execution_commit_gate_status,
        evaluation.execution_commit_gate_reason,
        gate.present,
        gate.source_chain_complete,
    );
    bf! { fields, "authority_decision_present" => gate.authority_decision_present; }
    bf! { fields, "loader_runtime_contract_present" => gate.loader_runtime_contract_present; }
    bf! { fields, "loader_runtime_source_evidence_complete" => gate.loader_runtime_source_evidence_complete; }
    bf! { fields, "service_slot_binding_source_evidence_present" => gate.service_slot_binding_source_evidence_present; }
    bf! { fields, "service_slot_binding_fact_present" => gate.service_slot_binding_fact_present; }
    bf! { fields, "audit_rollback_write_boundary_source_evidence_present" => gate.audit_rollback_write_boundary_source_evidence_present; }
    bf! { fields, "audit_rollback_write_boundary_fact_present" => gate.audit_rollback_write_boundary_fact_present; }
    bf! { fields, "retained_service_slot_reservation_present" => gate.retained_service_slot_reservation_present; }
    fields.push(f(
        "loader_runtime_source_evidence",
        module_loader_runtime_source_evidence_array(
            gate.loader_runtime_source_evidence_event_ids,
            gate.loader_runtime_source_evidence_present,
            gate.loader_runtime_fact_present,
        ),
    ));
    push_request_target_fields(&mut fields);
    push_false_fields(
        &mut fields,
        &[
            "accepts_loader_descriptor",
            "accepts_artifact_bytes",
            "authorizes_execution",
            "mutates_service_registry",
            "writes_durable_audit_state",
            "installs_rollback_state",
            "allocates_service_slot",
            "creates_service_inventory_records",
        ],
    );
    fields.push(f("service_inventory_change", s("none")));
    push_false_fields(&mut fields, &["loads_artifact", "load_attempted"]);
    emit_record_property_line("execution_commit_gate", fields, false);
}

pub(super) fn emit_module_loader_descriptor_intake_boundary(
    candidate: ModuleLoaderRuntimeCandidate,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    let boundary = candidate.descriptor_intake_boundary;
    let mut fields = module_loader_runtime_common_boundary_fields(
        MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_SCHEMA,
        MODULE_LOADER_DESCRIPTOR_INTAKE_BOUNDARY_ID,
        boundary.source_evidence_schema,
        boundary.source_evidence_event_id,
        boundary.source_evidence_state,
        boundary.source_evidence_status,
        boundary.source_evidence_reason,
        boundary.source_evidence_method,
        boundary.source_evidence_fact_locator,
        evaluation.descriptor_intake_boundary_status,
        evaluation.descriptor_intake_boundary_reason,
        boundary.present,
        boundary.source_chain_complete,
    );
    bf! { fields, "registry_write_commit_gate_present" => boundary.registry_write_commit_gate_present; }
    bf! { fields, "execution_commit_gate_present" => boundary.execution_commit_gate_present; }
    bf! { fields, "loader_runtime_source_evidence_complete" => boundary.loader_runtime_source_evidence_complete; }
    bf! { fields, "retained_module_evidence_present" => boundary.retained_module_evidence_present; }
    bf! { fields, "retained_service_slot_reservation_present" => boundary.retained_service_slot_reservation_present; }
    fields.push(f(
        "loader_runtime_source_evidence",
        module_loader_runtime_source_evidence_array(
            boundary.loader_runtime_source_evidence_event_ids,
            boundary.loader_runtime_source_evidence_present,
            boundary.loader_runtime_fact_present,
        ),
    ));
    push_request_target_fields(&mut fields);
    push_false_fields(
        &mut fields,
        &[
            "accepts_loader_descriptor",
            "accepts_descriptor_bytes",
            "produces_parsed_descriptor",
            "validates_descriptor_schema",
            "produces_validated_descriptor",
            "validates_descriptor_capabilities",
            "produces_capability_validated_descriptor",
            "authorizes_executable_load_plan",
            "produces_executable_load_plan",
            "produces_executable_image_layout",
            "produces_executable_page_mapping_plan",
            "binds_capability_validated_descriptor_to_executable_pages",
            "parses_descriptor_bytes",
            "accepts_artifact_bytes",
            "authorizes_descriptor_intake",
            "authorizes_execution",
            "mutates_service_registry",
            "writes_durable_audit_state",
            "installs_rollback_state",
            "allocates_service_slot",
            "creates_service_inventory_records",
        ],
    );
    fields.push(f("service_inventory_change", s("none")));
    push_false_fields(&mut fields, &["loads_artifact", "load_attempted"]);
    emit_record_property_line("descriptor_intake_boundary", fields, false);
}

pub(super) fn emit_module_loader_artifact_byte_intake_boundary(
    candidate: ModuleLoaderRuntimeCandidate,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    let boundary = candidate.artifact_byte_intake_boundary;
    let mut fields = module_loader_runtime_common_boundary_fields(
        MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_SCHEMA,
        MODULE_LOADER_ARTIFACT_BYTE_INTAKE_BOUNDARY_ID,
        boundary.source_evidence_schema,
        boundary.source_evidence_event_id,
        boundary.source_evidence_state,
        boundary.source_evidence_status,
        boundary.source_evidence_reason,
        boundary.source_evidence_method,
        boundary.source_evidence_fact_locator,
        evaluation.artifact_byte_intake_boundary_status,
        evaluation.artifact_byte_intake_boundary_reason,
        boundary.present,
        boundary.source_chain_complete,
    );
    bf! { fields, "descriptor_intake_boundary_present" => boundary.descriptor_intake_boundary_present; }
    bf! { fields, "descriptor_intake_boundary_source_chain_complete" => boundary.descriptor_intake_boundary_source_chain_complete; }
    bf! { fields, "execution_commit_gate_present" => boundary.execution_commit_gate_present; }
    bf! { fields, "artifact_hash_binding_present" => boundary.artifact_hash_binding_present; }
    bf! { fields, "retained_artifact_reference_present" => boundary.retained_artifact_reference_present; }
    bf! { fields, "retained_module_evidence_present" => boundary.retained_module_evidence_present; }
    bf! { fields, "retained_service_slot_reservation_present" => boundary.retained_service_slot_reservation_present; }
    fields.push(f(
        "loader_runtime_source_evidence",
        module_loader_runtime_source_evidence_array(
            boundary.loader_runtime_source_evidence_event_ids,
            boundary.loader_runtime_source_evidence_present,
            boundary.loader_runtime_fact_present,
        ),
    ));
    push_request_target_fields(&mut fields);
    push_false_fields(
        &mut fields,
        &[
            "accepts_loader_descriptor",
            "accepts_descriptor_bytes",
            "produces_parsed_descriptor",
            "validates_descriptor_schema",
            "produces_validated_descriptor",
            "validates_descriptor_capabilities",
            "produces_capability_validated_descriptor",
            "authorizes_executable_load_plan",
            "produces_executable_load_plan",
            "produces_executable_image_layout",
            "produces_executable_page_mapping_plan",
            "binds_capability_validated_descriptor_to_executable_pages",
            "parses_descriptor_bytes",
            "accepts_artifact_bytes",
            "authorizes_descriptor_intake",
            "authorizes_artifact_byte_intake",
            "authorizes_execution",
            "mutates_service_registry",
            "writes_durable_audit_state",
            "installs_rollback_state",
            "allocates_service_slot",
            "creates_service_inventory_records",
        ],
    );
    fields.push(f("service_inventory_change", s("none")));
    push_false_fields(&mut fields, &["loads_artifact", "load_attempted"]);
    emit_record_property_line("artifact_byte_intake_boundary", fields, false);
}

pub(super) fn emit_module_loader_execution_authorization_boundary(
    candidate: ModuleLoaderRuntimeCandidate,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    let boundary = candidate.execution_authorization_boundary;
    let mut fields = module_loader_runtime_common_boundary_fields(
        MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_EXECUTION_AUTHORIZATION_BOUNDARY_ID,
        boundary.source_evidence_schema,
        boundary.source_evidence_event_id,
        boundary.source_evidence_state,
        boundary.source_evidence_status,
        boundary.source_evidence_reason,
        boundary.source_evidence_method,
        boundary.source_evidence_fact_locator,
        evaluation.execution_authorization_boundary_status,
        evaluation.execution_authorization_boundary_reason,
        boundary.present,
        boundary.source_chain_complete,
    );
    bf! { fields, "artifact_byte_intake_boundary_present" => boundary.artifact_byte_intake_boundary_present; }
    bf! { fields, "artifact_byte_intake_boundary_source_chain_complete" => boundary.artifact_byte_intake_boundary_source_chain_complete; }
    bf! { fields, "descriptor_intake_boundary_present" => boundary.descriptor_intake_boundary_present; }
    bf! { fields, "descriptor_intake_boundary_source_chain_complete" => boundary.descriptor_intake_boundary_source_chain_complete; }
    bf! { fields, "execution_commit_gate_present" => boundary.execution_commit_gate_present; }
    bf! { fields, "entrypoint_abi_source_evidence_present" => boundary.entrypoint_abi_source_evidence_present; }
    bf! { fields, "address_space_source_evidence_present" => boundary.address_space_source_evidence_present; }
    bf! { fields, "memory_map_source_evidence_present" => boundary.memory_map_source_evidence_present; }
    bf! { fields, "audit_rollback_write_boundary_source_evidence_present" => boundary.audit_rollback_write_boundary_source_evidence_present; }
    bf! { fields, "retained_module_evidence_present" => boundary.retained_module_evidence_present; }
    bf! { fields, "retained_service_slot_reservation_present" => boundary.retained_service_slot_reservation_present; }
    fields.push(f(
        "loader_runtime_source_evidence",
        module_loader_runtime_source_evidence_array(
            boundary.loader_runtime_source_evidence_event_ids,
            boundary.loader_runtime_source_evidence_present,
            boundary.loader_runtime_fact_present,
        ),
    ));
    push_request_target_fields(&mut fields);
    push_false_fields(
        &mut fields,
        &[
            "accepts_loader_descriptor",
            "accepts_descriptor_bytes",
            "produces_parsed_descriptor",
            "validates_descriptor_schema",
            "produces_validated_descriptor",
            "validates_descriptor_capabilities",
            "produces_capability_validated_descriptor",
            "authorizes_executable_load_plan",
            "produces_executable_load_plan",
            "produces_executable_image_layout",
            "produces_executable_page_mapping_plan",
            "binds_capability_validated_descriptor_to_executable_pages",
            "parses_descriptor_bytes",
            "accepts_artifact_bytes",
            "authorizes_descriptor_intake",
            "authorizes_artifact_byte_intake",
            "maps_executable_pages",
            "jumps_to_entrypoint",
            "authorizes_execution",
            "mutates_service_registry",
            "writes_durable_audit_state",
            "installs_rollback_state",
            "allocates_service_slot",
            "creates_service_inventory_records",
        ],
    );
    fields.push(f("service_inventory_change", s("none")));
    push_false_fields(&mut fields, &["loads_artifact", "load_attempted"]);
    emit_record_property_line("execution_authorization_boundary", fields, false);
}

pub(super) fn emit_module_loader_service_registry_mutation_boundary(
    candidate: ModuleLoaderRuntimeCandidate,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    let boundary = candidate.service_registry_mutation_boundary;
    let mut fields = module_loader_runtime_common_boundary_fields(
        MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_SCHEMA,
        MODULE_LOADER_SERVICE_REGISTRY_MUTATION_BOUNDARY_ID,
        boundary.source_evidence_schema,
        boundary.source_evidence_event_id,
        boundary.source_evidence_state,
        boundary.source_evidence_status,
        boundary.source_evidence_reason,
        boundary.source_evidence_method,
        boundary.source_evidence_fact_locator,
        evaluation.service_registry_mutation_boundary_status,
        evaluation.service_registry_mutation_boundary_reason,
        boundary.present,
        boundary.source_chain_complete,
    );
    bf! { fields, "execution_authorization_boundary_present" => boundary.execution_authorization_boundary_present; }
    bf! { fields, "execution_authorization_boundary_source_chain_complete" => boundary.execution_authorization_boundary_source_chain_complete; }
    bf! { fields, "registry_write_commit_gate_present" => boundary.registry_write_commit_gate_present; }
    bf! { fields, "service_slot_binding_source_evidence_present" => boundary.service_slot_binding_source_evidence_present; }
    bf! { fields, "retained_module_evidence_present" => boundary.retained_module_evidence_present; }
    bf! { fields, "retained_service_slot_reservation_present" => boundary.retained_service_slot_reservation_present; }
    fields.push(f(
        "loader_runtime_source_evidence",
        module_loader_runtime_source_evidence_array(
            boundary.loader_runtime_source_evidence_event_ids,
            boundary.loader_runtime_source_evidence_present,
            boundary.loader_runtime_fact_present,
        ),
    ));
    push_request_target_fields(&mut fields);
    push_false_fields(
        &mut fields,
        &[
            "accepts_loader_descriptor",
            "accepts_descriptor_bytes",
            "produces_parsed_descriptor",
            "validates_descriptor_schema",
            "produces_validated_descriptor",
            "validates_descriptor_capabilities",
            "produces_capability_validated_descriptor",
            "authorizes_executable_load_plan",
            "produces_executable_load_plan",
            "produces_executable_image_layout",
            "produces_executable_page_mapping_plan",
            "binds_capability_validated_descriptor_to_executable_pages",
            "parses_descriptor_bytes",
            "accepts_artifact_bytes",
            "authorizes_descriptor_intake",
            "authorizes_artifact_byte_intake",
            "maps_executable_pages",
            "jumps_to_entrypoint",
            "authorizes_execution",
            "mutates_service_registry",
            "writes_durable_audit_state",
            "installs_rollback_state",
            "authorizes_load",
            "allocates_service_slot",
            "creates_service_inventory_records",
        ],
    );
    fields.push(f("service_inventory_change", s("none")));
    push_false_fields(&mut fields, &["loads_artifact", "load_attempted"]);
    emit_record_property_line("service_registry_mutation_boundary", fields, false);
}

pub(super) fn emit_module_loader_live_load_boundary(
    json_name: &'static str,
    boundary_schema: &'static str,
    boundary_id: &'static str,
    boundary: ModuleLoaderLiveLoadBoundary,
    status: &'static str,
    reason: &'static str,
) {
    let mut fields = module_loader_runtime_common_boundary_fields(
        boundary_schema,
        boundary_id,
        boundary.source_evidence_schema,
        boundary.source_evidence_event_id,
        boundary.source_evidence_state,
        boundary.source_evidence_status,
        boundary.source_evidence_reason,
        boundary.source_evidence_method,
        boundary.source_evidence_fact_locator,
        status,
        reason,
        boundary.present,
        boundary.source_chain_complete,
    );
    push_live_load_presence_fields(&mut fields, boundary);
    fields.push(f(
        "loader_runtime_source_evidence",
        module_loader_runtime_source_evidence_array(
            boundary.loader_runtime_source_evidence_event_ids,
            boundary.loader_runtime_source_evidence_present,
            boundary.loader_runtime_fact_present,
        ),
    ));
    push_request_target_fields(&mut fields);
    push_false_fields(
        &mut fields,
        &[
            "accepts_loader_descriptor",
            "accepts_descriptor_bytes",
            "produces_parsed_descriptor",
            "validates_descriptor_schema",
            "produces_validated_descriptor",
            "validates_descriptor_capabilities",
            "produces_capability_validated_descriptor",
            "authorizes_executable_load_plan",
            "produces_executable_load_plan",
            "produces_executable_image_layout",
            "produces_executable_page_mapping_plan",
            "binds_capability_validated_descriptor_to_executable_pages",
            "parses_descriptor_bytes",
            "accepts_artifact_bytes",
            "authorizes_descriptor_intake",
            "authorizes_artifact_byte_intake",
            "maps_executable_pages",
            "jumps_to_entrypoint",
            "authorizes_execution",
            "mutates_service_registry",
            "writes_durable_audit_state",
            "installs_rollback_state",
            "authorizes_load",
            "allocates_service_slot",
            "creates_service_inventory_records",
        ],
    );
    fields.push(f("service_inventory_change", s("none")));
    push_false_fields(
        &mut fields,
        &[
            "loads_artifact",
            "starts_service",
            "marks_service_running",
            "creates_service_health_records",
            "writes_service_start_audit_record",
            "unloads_service",
            "cleans_up_service_slot",
            "commits_live_load",
            "writes_load_commit_audit_record",
            "installs_commit_rollback_record",
            "records_load_result",
            "load_attempted",
        ],
    );
    emit_record_property_line(json_name, fields, false);
}

pub(super) fn emit_module_loader_runtime_facts(
    candidate: ModuleLoaderRuntimeCandidate,
    evaluation: ModuleLoaderRuntimeEvaluation,
) {
    emit_record_property_line(
        "loader_runtime_facts",
        vec![
            module_loader_runtime_fact(
                MODULE_LOADER_RUNTIME_FACT_SOURCES[0],
                candidate.loader_identity,
                evaluation.loader_identity_status,
                evaluation.loader_identity_reason,
            ),
            module_loader_runtime_fact(
                MODULE_LOADER_RUNTIME_FACT_SOURCES[1],
                candidate.artifact_hash_binding,
                evaluation.artifact_hash_binding_status,
                evaluation.artifact_hash_binding_reason,
            ),
            module_loader_runtime_fact(
                MODULE_LOADER_RUNTIME_FACT_SOURCES[2],
                candidate.entrypoint_abi,
                evaluation.entrypoint_abi_status,
                evaluation.entrypoint_abi_reason,
            ),
            module_loader_runtime_fact(
                MODULE_LOADER_RUNTIME_FACT_SOURCES[3],
                candidate.address_space_boundary,
                evaluation.address_space_boundary_status,
                evaluation.address_space_boundary_reason,
            ),
            module_loader_runtime_fact(
                MODULE_LOADER_RUNTIME_FACT_SOURCES[4],
                candidate.memory_map_constraints,
                evaluation.memory_map_constraints_status,
                evaluation.memory_map_constraints_reason,
            ),
            module_loader_runtime_fact(
                MODULE_LOADER_RUNTIME_FACT_SOURCES[5],
                candidate.capability_import_table,
                evaluation.capability_import_table_status,
                evaluation.capability_import_table_reason,
            ),
            module_loader_runtime_fact(
                MODULE_LOADER_RUNTIME_FACT_SOURCES[6],
                candidate.service_slot_binding,
                evaluation.service_slot_binding_status,
                evaluation.service_slot_binding_reason,
            ),
            module_loader_runtime_fact(
                MODULE_LOADER_RUNTIME_FACT_SOURCES[7],
                candidate.health_state_hooks,
                evaluation.health_state_hooks_status,
                evaluation.health_state_hooks_reason,
            ),
            module_loader_runtime_fact(
                MODULE_LOADER_RUNTIME_FACT_SOURCES[8],
                candidate.rollback_hooks,
                evaluation.rollback_hooks_status,
                evaluation.rollback_hooks_reason,
            ),
            module_loader_runtime_fact(
                MODULE_LOADER_RUNTIME_FACT_SOURCES[9],
                candidate.audit_rollback_write_boundary_binding,
                evaluation.audit_rollback_write_boundary_binding_status,
                evaluation.audit_rollback_write_boundary_binding_reason,
            ),
        ],
        false,
    );
}

fn module_loader_runtime_fact(
    source: ModuleLoaderRuntimeFactSource,
    fact: ModuleLoaderRuntimeFact,
    status: &'static str,
    reason: &'static str,
) -> Field<'static> {
    let mut fields = vec![
        f("schema", s(source.schema)),
        f("id", s(source.id)),
        f("source_method", s(source.source_method)),
        f("source_fact_locator", s(source.source_fact_locator)),
    ];
    if module_loader_runtime_fact_source_evidence_visible(source) {
        fields.push(f(
            "source_evidence_event_id",
            record_event_or_null(fact.source_evidence_event_id),
        ));
        fields.push(f("source_evidence_schema", s(fact.source_evidence_schema)));
        fields.push(f("source_evidence_state", s(fact.source_evidence_state)));
        fields.push(f("source_evidence_status", s(fact.source_evidence_status)));
        fields.push(f("source_evidence_reason", s(fact.source_evidence_reason)));
        fields.push(f("source_evidence_method", s(fact.source_evidence_method)));
        fields.push(f(
            "source_evidence_fact_locator",
            s(fact.source_evidence_fact_locator),
        ));
    }
    fields.push(f("scope", s(fact.scope)));
    fields.push(f("classification", s(fact.classification)));
    fields.push(f("status", s(status)));
    fields.push(f("reason", s(reason)));
    fields.push(f("present", b(fact.present)));
    fields.push(f("schema_valid", b(fact.schema_ok)));
    fields.push(f("provenance_valid", b(fact.provenance_ok)));
    bf! { fields, "binds_retained_module_evidence" => fact.binds_retained_module_evidence; }
    bf! { fields, "binds_service_slot_allocator" => fact.binds_service_slot_allocator; }
    bf! { fields, "binds_audit_rollback_write_boundary" => fact.binds_audit_rollback_write_boundary; }
    fields.push(f("authority", s("current_snapshot")));
    fields.push(f("persistence", s("none")));
    fields.push(f("durable", no()));
    fields.push(f("loads_artifact", no()));
    fields.push(f("allocates_service_slot", no()));
    fields.push(f("creates_service_inventory_records", no()));
    fields.push(f("service_inventory_change", s("none")));
    fields.push(f("authorizes_load", no()));
    fields.push(f(
        "required_bindings",
        V::Object(vec![
            f(
                "retained_module_evidence",
                s("current_boot_hash_references"),
            ),
            f(
                "service_slot_allocator_readiness",
                s("raios.module_service_slot_allocator_readiness.v0"),
            ),
            f(
                "audit_write_boundary",
                s("raios.module_audit_rollback_write_boundary.v0"),
            ),
            f(
                "execution_commit_gate",
                s("raios.module_loader_runtime_execution_commit_gate.v0"),
            ),
            f(
                "descriptor_intake_boundary",
                s("raios.module_loader_descriptor_intake_boundary.v0"),
            ),
            f(
                "module_loader_runtime",
                s("raios.module_loader_runtime_readiness.v0"),
            ),
        ]),
    ));
    fields.push(f(
        "provenance",
        V::Object(vec![
            f("source_method", s(source.source_method)),
            f("source_fact_locator", s(source.source_fact_locator)),
            f("aggregate_method", s("module.loader_runtime")),
            f("source_transport", s("serial-console")),
            f("event_scope", s("current_boot")),
            f("record_id", null()),
        ]),
    ));
    f(source.name, V::Object(fields))
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
