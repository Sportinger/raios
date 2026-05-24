use crate::{
    agent_protocol_recovery_runtime_types::{
        RecoveryLifelineCommandAdmissionCandidate, RecoveryLifelineCommandAdmissionCheck,
        RecoveryLifelineCommandAdmissionSelfTestCase, RecoveryMemoryProvenanceCheck,
    },
    agent_protocol_support::{crlf, json_str, raw, raw_bool, raw_line},
};

pub(crate) fn emit_recovery_lifeline_command_admission_input_state(
    candidate: &RecoveryLifelineCommandAdmissionCandidate,
    memory_check: &RecoveryMemoryProvenanceCheck,
    check: &RecoveryLifelineCommandAdmissionCheck,
    comma: bool,
) {
    raw_line("      \"command_admission_inputs\": {");
    raw_line("        \"recovery_memory_provenance\": {");
    raw_line("          \"schema\": \"raios.recovery_memory_provenance.v0\",");
    raw("          \"state\": ");
    json_str(if candidate.recovery_memory_provenance_available {
        "defined_read_only_boundary"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("          \"retention\": \"current_boot_read_only_diagnostic\",");
    raw_line("          \"event_id\": null,");
    raw("          \"boundary_exposed\": ");
    raw_bool(check.recovery_memory_provenance_boundary_exposed);
    raw_line(",");
    raw("          \"accepted_for_command_admission\": ");
    raw_bool(check.recovery_memory_provenance_accepted);
    raw_line(",");
    raw("          \"recovery_memory_provenance_ready\": ");
    raw_bool(memory_check.recovery_memory_provenance_ready);
    raw_line(",");
    raw("          \"current_boot\": ");
    raw_bool(candidate.recovery_memory_provenance_current_boot);
    raw_line(",");
    raw("          \"schema_ok\": ");
    raw_bool(candidate.recovery_memory_provenance_schema_ok);
    raw_line(",");
    raw("          \"binding_ok\": ");
    raw_bool(candidate.recovery_memory_provenance_binding_ok);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(candidate.recovery_memory_provenance_binding_reason);
    raw_line(",");
    raw_line("          \"writes_recovery_memory\": false,");
    raw_line("          \"exports_provider_context\": false,");
    raw_line("          \"accepts_lifeline_command_envelope\": false");
    raw_line("        }");
    raw("      }");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_lifeline_command_admission_requirement(
    command_id: &'static str,
    args_schema: &'static str,
    required_capability: &'static str,
    readiness_schema: &'static str,
    present: bool,
    missing_reason: &'static str,
    check: &RecoveryLifelineCommandAdmissionCheck,
    comma: bool,
) {
    raw_line("        {");
    raw("          \"command_id\": ");
    json_str(command_id);
    raw_line(",");
    raw("          \"argument_schema\": ");
    json_str(args_schema);
    raw_line(",");
    raw("          \"required_capability\": ");
    json_str(required_capability);
    raw_line(",");
    raw("          \"admission_schema\": ");
    json_str(readiness_schema);
    raw_line(",");
    raw("          \"status\": ");
    json_str(if present { "present" } else { "missing" });
    raw_line(",");
    raw_line("          \"scope\": \"current_boot\",");
    raw_line("          \"classification\": \"local_only\",");
    raw_line("          \"required\": true,");
    raw_line("          \"event_id\": null,");
    raw("          \"reason\": ");
    json_str(if present {
        "command_admission_requirement_defined"
    } else {
        missing_reason
    });
    raw_line(",");
    raw("          \"admission_ready\": ");
    raw_bool(check.command_admission_ready && present);
    raw_line(",");
    raw_line("          \"accepts_envelope\": false,");
    raw_line("          \"dispatches_command\": false,");
    raw_line("          \"command_execution_enabled\": false,");
    raw_line("          \"rollback_preview_enabled\": false,");
    raw_line("          \"rollback_apply_enabled\": false,");
    raw_line("          \"memory_writes_enabled\": false,");
    raw_line("          \"provider_export_enabled\": false,");
    raw_line("          \"durable_writes_enabled\": false,");
    raw_line("          \"loads_recovery_artifact\": false,");
    raw_line("          \"creates_durable_records\": false,");
    raw_line("          \"installs_rollback_plan\": false,");
    raw_line("          \"allocates_service_slot\": false,");
    raw_line("          \"service_inventory_change\": \"none\",");
    raw_line("          \"load_attempted\": false");
    raw("        }");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_lifeline_command_admission_boundary(
    check: &RecoveryLifelineCommandAdmissionCheck,
) {
    raw_line("        \"schema\": \"raios.recovery_lifeline_command_admission.v0\",");
    raw_line("        \"state\": \"defined_non_executable\",");
    raw("        \"requirements_exposed\": ");
    raw_bool(check.command_admission_requirements_exposed);
    raw_line(",");
    raw("        \"command_admission_ready\": ");
    raw_bool(check.command_admission_ready);
    raw_line(",");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"rollback_preview_enabled\": false,");
    raw_line("        \"rollback_apply_enabled\": false,");
    raw_line("        \"memory_writes_enabled\": false,");
    raw_line("        \"provider_export_enabled\": false,");
    raw_line("        \"durable_writes_enabled\": false,");
    raw_line("        \"rollback_replay_enabled\": false,");
    raw_line("        \"recovery_memory_writes_enabled\": false,");
    raw_line("        \"writes_durable_audit_log\": false,");
    raw_line("        \"writes_rollback_store\": false,");
    raw_line("        \"loads_recovery_loader\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw_line("        \"required_before_admission\": [");
    raw_line("          \"raios.recovery_lifeline_request.v0\",");
    raw_line("          \"raios.recovery_lifeline_protocol_state.v0\",");
    raw_line("          \"raios.recovery_lifeline_command_vocabulary.v0\",");
    raw_line("          \"raios.recovery_loader_runtime_isolation.v0\",");
    raw_line("          \"raios.recovery_rollback_transaction_engine.v0\",");
    raw_line("          \"raios.durable_audit_rollback_persistence.v0\",");
    raw_line("          \"raios.recovery_memory_provenance.v0\"");
    raw_line("        ]");
}

pub(crate) fn emit_recovery_lifeline_command_admission_check(
    check: &RecoveryLifelineCommandAdmissionCheck,
) {
    raw("        \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"request_chain_valid\": ");
    raw_bool(check.memory_check.request_chain_valid);
    raw_line(",");
    raw("        \"command_vocabulary_envelope_exposed\": ");
    raw_bool(check.memory_check.command_vocabulary_envelope_exposed);
    raw_line(",");
    raw("        \"command_vocabulary_accepted\": ");
    raw_bool(check.memory_check.command_vocabulary_accepted);
    raw_line(",");
    raw("        \"loader_runtime_isolation_boundary_exposed\": ");
    raw_bool(check.memory_check.loader_runtime_isolation_boundary_exposed);
    raw_line(",");
    raw("        \"loader_runtime_isolation_accepted\": ");
    raw_bool(check.memory_check.loader_runtime_isolation_accepted);
    raw_line(",");
    raw("        \"rollback_transaction_engine_boundary_exposed\": ");
    raw_bool(
        check
            .memory_check
            .rollback_transaction_engine_boundary_exposed,
    );
    raw_line(",");
    raw("        \"rollback_transaction_engine_accepted\": ");
    raw_bool(check.memory_check.rollback_transaction_engine_accepted);
    raw_line(",");
    raw("        \"durable_audit_rollback_persistence_boundary_exposed\": ");
    raw_bool(
        check
            .memory_check
            .durable_audit_rollback_persistence_boundary_exposed,
    );
    raw_line(",");
    raw("        \"durable_audit_rollback_persistence_accepted\": ");
    raw_bool(
        check
            .memory_check
            .durable_audit_rollback_persistence_accepted,
    );
    raw_line(",");
    raw("        \"recovery_memory_provenance_boundary_exposed\": ");
    raw_bool(check.recovery_memory_provenance_boundary_exposed);
    raw_line(",");
    raw("        \"recovery_memory_provenance_accepted\": ");
    raw_bool(check.recovery_memory_provenance_accepted);
    raw_line(",");
    raw("        \"command_admission_requirements_exposed\": ");
    raw_bool(check.command_admission_requirements_exposed);
    raw_line(",");
    raw("        \"command_admission_ready\": ");
    raw_bool(check.command_admission_ready);
    raw_line(",");
    raw("        \"lifeline_status_admission_present\": ");
    raw_bool(check.lifeline_status_admission_present);
    raw_line(",");
    raw("        \"rollback_preview_admission_present\": ");
    raw_bool(check.rollback_preview_admission_present);
    raw_line(",");
    raw("        \"rollback_apply_admission_present\": ");
    raw_bool(check.rollback_apply_admission_present);
    raw_line(",");
    raw("        \"disable_module_admission_present\": ");
    raw_bool(check.disable_module_admission_present);
    raw_line(",");
    raw("        \"restart_last_good_admission_present\": ");
    raw_bool(check.restart_last_good_admission_present);
    raw_line(",");
    raw("        \"load_recovery_artifact_by_hash_admission_present\": ");
    raw_bool(check.load_recovery_artifact_by_hash_admission_present);
    raw_line(",");
    raw("        \"command_execution_enabled\": ");
    raw_bool(check.command_execution_enabled);
    raw_line(",");
    raw("        \"accepts_lifeline_command_envelope\": ");
    raw_bool(check.accepts_lifeline_command_envelope);
    raw_line(",");
    raw("        \"dispatches_lifeline_command\": ");
    raw_bool(check.dispatches_lifeline_command);
    raw_line(",");
    raw("        \"authorizes_recovery_load\": ");
    raw_bool(check.authorizes_recovery_load);
    raw_line(",");
    raw("        \"can_move_beyond_denial\": ");
    raw_bool(check.can_move_beyond_denial);
    raw_line(",");
    raw("        \"loads_recovery_loader\": ");
    raw_bool(check.loads_recovery_loader);
    raw_line(",");
    raw("        \"loads_recovery_artifact\": ");
    raw_bool(check.loads_recovery_artifact);
    raw_line(",");
    raw("        \"creates_durable_records\": ");
    raw_bool(check.creates_durable_records);
    raw_line(",");
    raw("        \"installs_rollback_plan\": ");
    raw_bool(check.installs_rollback_plan);
    raw_line(",");
    raw("        \"allocates_service_slot\": ");
    raw_bool(check.allocates_service_slot);
    raw_line(",");
    raw("        \"service_inventory_change\": ");
    json_str(check.service_inventory_change);
    raw_line(",");
    raw_line("        \"durable_audit_write_attempted\": false,");
    raw_line("        \"rollback_install_attempted\": false,");
    raw_line("        \"rollback_replay_attempted\": false,");
    raw_line("        \"recovery_memory_write_attempted\": false,");
    raw_line("        \"provider_export_attempted\": false,");
    raw_line("        \"lifeline_command_dispatch_attempted\": false,");
    raw_line("        \"service_slot_allocation_attempted\": false,");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw("        \"load_attempted\": ");
    raw_bool(check.load_attempted);
    crlf();
}

pub(crate) fn emit_recovery_lifeline_command_admission_selftest_case(
    case: &RecoveryLifelineCommandAdmissionSelfTestCase,
    comma: bool,
) {
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
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"command_execution_enabled\": false, \"accepts_lifeline_command_envelope\": false, \"dispatches_lifeline_command\": false, \"memory_writes_enabled\": false, \"provider_export_enabled\": false, \"durable_writes_enabled\": false, \"rollback_replay_enabled\": false, \"recovery_memory_writes_enabled\": false, \"rollback_preview_enabled\": false, \"rollback_apply_enabled\": false, \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}
