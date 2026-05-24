use crate::{
    agent_protocol_recovery_runtime_types::{
        RecoveryDurableAuditRollbackPersistenceCheck, RecoveryMemoryProvenanceCandidate,
        RecoveryMemoryProvenanceCheck, RecoveryMemoryProvenanceSelfTestCase,
    },
    agent_protocol_support::{crlf, json_str, raw, raw_bool, raw_line},
};

pub(crate) fn emit_recovery_memory_provenance_input_state(
    candidate: &RecoveryMemoryProvenanceCandidate,
    persistence_check: &RecoveryDurableAuditRollbackPersistenceCheck,
    check: &RecoveryMemoryProvenanceCheck,
    comma: bool,
) {
    raw_line("      \"memory_provenance_inputs\": {");
    raw_line("        \"durable_audit_rollback_persistence\": {");
    raw_line("          \"schema\": \"raios.durable_audit_rollback_persistence.v0\",");
    raw("          \"state\": ");
    json_str(if candidate.durable_audit_rollback_persistence_available {
        "defined_read_only_boundary"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("          \"retention\": \"current_boot_read_only_diagnostic\",");
    raw_line("          \"event_id\": null,");
    raw("          \"boundary_exposed\": ");
    raw_bool(check.durable_audit_rollback_persistence_boundary_exposed);
    raw_line(",");
    raw("          \"accepted_for_memory_readiness\": ");
    raw_bool(check.durable_audit_rollback_persistence_accepted);
    raw_line(",");
    raw("          \"durable_audit_rollback_persistence_ready\": ");
    raw_bool(persistence_check.durable_audit_rollback_persistence_ready);
    raw_line(",");
    raw("          \"current_boot\": ");
    raw_bool(candidate.durable_audit_rollback_persistence_current_boot);
    raw_line(",");
    raw("          \"schema_ok\": ");
    raw_bool(candidate.durable_audit_rollback_persistence_schema_ok);
    raw_line(",");
    raw("          \"binding_ok\": ");
    raw_bool(candidate.durable_audit_rollback_persistence_binding_ok);
    raw_line(",");
    raw("          \"reason\": ");
    json_str(candidate.durable_audit_rollback_persistence_binding_reason);
    raw_line(",");
    raw_line("          \"writes_durable_audit_log\": false,");
    raw_line("          \"writes_rollback_store\": false,");
    raw_line("          \"replays_rollback_transactions\": false,");
    raw_line("          \"recovery_memory_writes_enabled\": false");
    raw_line("        }");
    raw("      }");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_memory_provenance_fact(
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
    raw(", \"authorizes_recovery_load\": false, \"memory_writes_enabled\": false, \"provider_export_enabled\": false, \"durable_writes_enabled\": false, \"rollback_replay_enabled\": false, \"recovery_memory_writes_enabled\": false, \"rollback_preview_enabled\": false, \"rollback_apply_enabled\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_memory_provenance_boundary(check: &RecoveryMemoryProvenanceCheck) {
    raw_line("        \"schema\": \"raios.recovery_memory_provenance.v0\",");
    raw_line("        \"state\": \"defined_non_executable\",");
    raw("        \"requirements_exposed\": ");
    raw_bool(check.memory_provenance_requirements_exposed);
    raw_line(",");
    raw("        \"recovery_memory_provenance_ready\": ");
    raw_bool(check.recovery_memory_provenance_ready);
    raw_line(",");
    raw_line("        \"memory_writes_enabled\": false,");
    raw_line("        \"provider_export_enabled\": false,");
    raw_line("        \"durable_writes_enabled\": false,");
    raw_line("        \"rollback_replay_enabled\": false,");
    raw_line("        \"recovery_memory_writes_enabled\": false,");
    raw_line("        \"rollback_preview_enabled\": false,");
    raw_line("        \"rollback_apply_enabled\": false,");
    raw_line("        \"accepts_memory_record_json\": false,");
    raw_line("        \"accepts_provider_context_export\": false,");
    raw_line("        \"accepts_rollback_transaction_envelope\": false,");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"writes_durable_audit_log\": false,");
    raw_line("        \"writes_rollback_store\": false,");
    raw_line("        \"exports_provider_context\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw_line("        \"required_before_memory_write\": [");
    raw_line("          \"raios.recovery_lifeline_protocol_state.v0\",");
    raw_line("          \"raios.recovery_lifeline_command_vocabulary.v0\",");
    raw_line("          \"raios.recovery_loader_runtime_isolation.v0\",");
    raw_line("          \"raios.recovery_rollback_transaction_engine.v0\",");
    raw_line("          \"raios.durable_audit_rollback_persistence.v0\",");
    raw_line("          \"raios.recovery_memory_source_record_ids.v0\",");
    raw_line("          \"raios.recovery_memory_source_schema_hashes.v0\",");
    raw_line("          \"raios.recovery_memory_classification.v0\",");
    raw_line("          \"raios.recovery_memory_authority_level.v0\",");
    raw_line("          \"raios.recovery_memory_rollback_transaction_binding.v0\",");
    raw_line("          \"raios.recovery_memory_last_good_checkpoint_binding.v0\",");
    raw_line("          \"raios.recovery_memory_export_profile.v0\",");
    raw_line("          \"raios.recovery_memory_redaction_state.v0\",");
    raw_line("          \"raios.recovery_memory_replay_window.v0\",");
    raw_line("          \"raios.recovery_memory_audit_linkage.v0\"");
    raw_line("        ]");
}

pub(crate) fn emit_recovery_memory_provenance_check(check: &RecoveryMemoryProvenanceCheck) {
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
    raw("        \"loader_runtime_isolation_boundary_exposed\": ");
    raw_bool(check.loader_runtime_isolation_boundary_exposed);
    raw_line(",");
    raw("        \"loader_runtime_isolation_accepted\": ");
    raw_bool(check.loader_runtime_isolation_accepted);
    raw_line(",");
    raw("        \"rollback_transaction_engine_boundary_exposed\": ");
    raw_bool(check.rollback_transaction_engine_boundary_exposed);
    raw_line(",");
    raw("        \"rollback_transaction_engine_accepted\": ");
    raw_bool(check.rollback_transaction_engine_accepted);
    raw_line(",");
    raw("        \"durable_audit_rollback_persistence_boundary_exposed\": ");
    raw_bool(check.durable_audit_rollback_persistence_boundary_exposed);
    raw_line(",");
    raw("        \"durable_audit_rollback_persistence_accepted\": ");
    raw_bool(check.durable_audit_rollback_persistence_accepted);
    raw_line(",");
    raw("        \"memory_provenance_requirements_exposed\": ");
    raw_bool(check.memory_provenance_requirements_exposed);
    raw_line(",");
    raw("        \"recovery_memory_provenance_ready\": ");
    raw_bool(check.recovery_memory_provenance_ready);
    raw_line(",");
    raw("        \"memory_writes_enabled\": ");
    raw_bool(check.memory_writes_enabled);
    raw_line(",");
    raw("        \"provider_export_enabled\": ");
    raw_bool(check.provider_export_enabled);
    raw_line(",");
    raw("        \"durable_writes_enabled\": ");
    raw_bool(check.durable_writes_enabled);
    raw_line(",");
    raw("        \"rollback_replay_enabled\": ");
    raw_bool(check.rollback_replay_enabled);
    raw_line(",");
    raw("        \"recovery_memory_writes_enabled\": ");
    raw_bool(check.recovery_memory_writes_enabled);
    raw_line(",");
    raw("        \"rollback_preview_enabled\": ");
    raw_bool(check.rollback_preview_enabled);
    raw_line(",");
    raw("        \"rollback_apply_enabled\": ");
    raw_bool(check.rollback_apply_enabled);
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
    raw_line("        \"rollback_replay_attempted\": false,");
    raw_line("        \"recovery_memory_write_attempted\": false,");
    raw_line("        \"provider_export_attempted\": false,");
    raw_line("        \"service_slot_allocation_attempted\": false,");
    raw_line("        \"direct_openai_recovery_shortcut_accepted\": false,");
    raw("        \"load_attempted\": ");
    raw_bool(check.load_attempted);
    crlf();
}

pub(crate) fn emit_recovery_memory_provenance_selftest_case(
    case: &RecoveryMemoryProvenanceSelfTestCase,
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
    raw(", \"memory_writes_enabled\": false, \"provider_export_enabled\": false, \"durable_writes_enabled\": false, \"rollback_replay_enabled\": false, \"recovery_memory_writes_enabled\": false, \"rollback_preview_enabled\": false, \"rollback_apply_enabled\": false, \"command_execution_enabled\": false, \"accepts_lifeline_command_envelope\": false, \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}
