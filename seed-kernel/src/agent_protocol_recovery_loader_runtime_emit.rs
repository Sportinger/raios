use crate::{
    agent_protocol_recovery_runtime_types::{
        RecoveryLoaderRuntimeIsolationCandidate, RecoveryLoaderRuntimeIsolationCheck,
        RecoveryLoaderRuntimeIsolationSelfTestCase,
    },
    agent_protocol_support::{crlf, json_str, raw, raw_bool, raw_line},
};

pub(crate) fn emit_recovery_loader_runtime_isolation_input_state(
    candidate: &RecoveryLoaderRuntimeIsolationCandidate,
    check: &RecoveryLoaderRuntimeIsolationCheck,
    comma: bool,
) {
    raw_line("      \"loader_isolation_inputs\": {");
    raw_line("        \"lifeline_protocol_state\": {");
    raw_line("          \"schema\": \"raios.recovery_lifeline_protocol_state.v0\",");
    raw("          \"state\": ");
    json_str(if candidate.command_candidate.protocol_state_retained {
        "present"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("          \"retention\": \"current_boot_ram_event_log\",");
    raw_line("          \"event_id\": null,");
    raw("          \"current_boot\": ");
    raw_bool(candidate.command_candidate.protocol_state_current_boot);
    raw_line(",");
    raw("          \"schema_ok\": ");
    raw_bool(candidate.command_candidate.protocol_state_schema_ok);
    raw_line(",");
    raw("          \"binding_ok\": ");
    raw_bool(candidate.command_candidate.protocol_state_binding_ok);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(candidate.command_candidate.protocol_state_binding_reason);
    raw_line(",");
    raw_line("          \"authorizes_recovery_load\": false");
    raw_line("        },");
    raw_line("        \"lifeline_command_vocabulary\": {");
    raw_line("          \"schema\": \"raios.recovery_lifeline_command_vocabulary.v0\",");
    raw("          \"state\": ");
    json_str(if candidate.command_vocabulary_available {
        "defined_read_only_envelope"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("          \"retention\": \"current_boot_read_only_diagnostic\",");
    raw_line("          \"event_id\": null,");
    raw("          \"envelope_exposed\": ");
    raw_bool(check.command_vocabulary_envelope_exposed);
    raw_line(",");
    raw("          \"accepted_for_loader_readiness\": ");
    raw_bool(check.command_vocabulary_accepted);
    raw_line(",");
    raw("          \"current_boot\": ");
    raw_bool(candidate.command_vocabulary_current_boot);
    raw_line(",");
    raw("          \"schema_ok\": ");
    raw_bool(candidate.command_vocabulary_schema_ok);
    raw_line(",");
    raw("          \"binding_ok\": ");
    raw_bool(candidate.command_vocabulary_binding_ok);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(candidate.command_vocabulary_binding_reason);
    raw_line(",");
    raw_line("          \"accepts_lifeline_command_envelope\": false,");
    raw_line("          \"dispatches_commands\": false,");
    raw_line("          \"authorizes_recovery_load\": false");
    raw_line("        }");
    raw("      }");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_loader_runtime_isolation_fact(
    field: &'static str,
    schema: &'static str,
    present: bool,
    missing_reason: &'static str,
    comma: bool,
) {
    raw("        \"");
    raw(field);
    raw("\": {\"schema\": ");
    json_str(schema);
    raw(", \"status\": ");
    json_str(if present { "present" } else { "missing" });
    raw(", \"event_id\": null, \"retained\": false, \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
    json_str(if present {
        "current_boot_fact_available"
    } else {
        missing_reason
    });
    raw(", \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_loader_runtime_isolation_boundary(
    check: &RecoveryLoaderRuntimeIsolationCheck,
) {
    raw_line("        \"schema\": \"raios.recovery_loader_runtime_isolation.v0\",");
    raw_line("        \"state\": \"defined_non_executable\",");
    raw("        \"requirements_exposed\": ");
    raw_bool(check.isolation_requirements_exposed);
    raw_line(",");
    raw("        \"loader_runtime_isolation_ready\": ");
    raw_bool(check.loader_runtime_isolation_ready);
    raw_line(",");
    raw_line("        \"loader_execution_enabled\": false,");
    raw_line("        \"accepts_loader_descriptor\": false,");
    raw_line("        \"accepts_artifact_bytes\": false,");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"loads_recovery_loader\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"normal_module_load_path_used\": false,");
    raw_line("        \"recovery_lifeline_command_dispatch_enabled\": false,");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw_line("        \"required_before_execution\": [");
    raw_line("          \"raios.recovery_lifeline_protocol_state.v0\",");
    raw_line("          \"raios.recovery_lifeline_command_vocabulary.v0\",");
    raw_line("          \"raios.recovery_loader_address_space_boundary.v0\",");
    raw_line("          \"raios.recovery_loader_entrypoint_abi.v0\",");
    raw_line("          \"raios.recovery_loader_memory_map_constraints.v0\",");
    raw_line("          \"raios.recovery_loader_capability_import_table.v0\",");
    raw_line("          \"raios.recovery_loader_artifact_hash_binding.v0\",");
    raw_line("          \"raios.recovery_loader_provider_separation.v0\",");
    raw_line("          \"raios.recovery_loader_normal_module_separation.v0\",");
    raw_line("          \"raios.recovery_rollback_transaction_engine.v0\",");
    raw_line("          \"raios.durable_audit_rollback_persistence.v0\",");
    raw_line("          \"raios.recovery_memory_provenance.v0\"");
    raw_line("        ]");
}

pub(crate) fn emit_recovery_loader_runtime_isolation_check(
    check: &RecoveryLoaderRuntimeIsolationCheck,
) {
    raw("        \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"request_chain_valid\": ");
    raw_bool(check.request_chain_valid);
    raw_line(",");
    raw("        \"command_vocabulary_envelope_exposed\": ");
    raw_bool(check.command_vocabulary_envelope_exposed);
    raw_line(",");
    raw("        \"command_vocabulary_accepted\": ");
    raw_bool(check.command_vocabulary_accepted);
    raw_line(",");
    raw("        \"isolation_requirements_exposed\": ");
    raw_bool(check.isolation_requirements_exposed);
    raw_line(",");
    raw("        \"loader_runtime_isolation_ready\": ");
    raw_bool(check.loader_runtime_isolation_ready);
    raw_line(",");
    raw("        \"command_execution_enabled\": ");
    raw_bool(check.command_execution_enabled);
    raw_line(",");
    raw("        \"accepts_lifeline_command_envelope\": ");
    raw_bool(check.accepts_lifeline_command_envelope);
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
    raw_line("        \"service_slot_allocation_attempted\": false,");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw("        \"load_attempted\": ");
    raw_bool(check.load_attempted);
    crlf();
}

pub(crate) fn emit_recovery_loader_runtime_isolation_selftest_case(
    case: &RecoveryLoaderRuntimeIsolationSelfTestCase,
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
    raw(", \"loader_execution_enabled\": false, \"command_execution_enabled\": false, \"accepts_lifeline_command_envelope\": false, \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}
