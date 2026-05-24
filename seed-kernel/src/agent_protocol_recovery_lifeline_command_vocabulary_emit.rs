use crate::{
    agent_protocol_recovery_lifeline_protocol_types::{
        RecoveryLifelineCommandVocabularyCheck, RecoveryLifelineCommandVocabularySelfTestCase,
    },
    agent_protocol_support::{crlf, json_str, raw, raw_bool, raw_fmt, raw_line},
};

pub(crate) fn emit_recovery_lifeline_command_vocabulary_object(
    check: &RecoveryLifelineCommandVocabularyCheck,
    comma: bool,
) {
    raw_line("      \"command_vocabulary\": {");
    raw_line("        \"schema\": \"raios.recovery_lifeline_command_vocabulary.v0\",");
    raw_line("        \"state\": \"defined_non_executable\",");
    raw_line("        \"scope\": \"current_boot\",");
    raw_line("        \"classification\": \"local_only\",");
    raw("        \"exposed\": ");
    raw_bool(check.command_vocabulary_exposed);
    raw_line(",");
    raw_line("        \"authority\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"dispatches_commands\": false,");
    raw_line(
        "        \"argument_envelope_schema\": \"raios.recovery_lifeline_command_envelope.v0\",",
    );
    raw("        \"primary_denial_reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"command_count\": ");
    raw_fmt(format_args!(
        "{}",
        if check.command_vocabulary_exposed {
            6usize
        } else {
            0usize
        }
    ));
    raw_line(",");
    raw_line("        \"commands\": [");
    if check.command_vocabulary_exposed {
        emit_recovery_lifeline_command_definition(
            "recovery.lifeline.status",
            "raios.recovery_lifeline_command.status_args.v0",
            "cap.recovery.load_artifact.read",
            "observe",
            "read recovery lifeline status",
            check.reason,
            true,
        );
        emit_recovery_lifeline_command_definition(
            "recovery.lifeline.rollback_preview",
            "raios.recovery_lifeline_command.rollback_preview_args.v0",
            "cap.recovery.rollback.read",
            "observe",
            "preview rollback target and required evidence",
            check.reason,
            true,
        );
        emit_recovery_lifeline_command_definition(
            "recovery.lifeline.rollback_apply",
            "raios.recovery_lifeline_command.rollback_apply_args.v0",
            "cap.recovery.rollback",
            "persist",
            "apply a rollback transaction",
            check.reason,
            true,
        );
        emit_recovery_lifeline_command_definition(
            "recovery.lifeline.disable_module",
            "raios.recovery_lifeline_command.disable_module_args.v0",
            "cap.recovery.module.disable",
            "persist",
            "disable a bad retained module id",
            check.reason,
            true,
        );
        emit_recovery_lifeline_command_definition(
            "recovery.lifeline.restart_last_good",
            "raios.recovery_lifeline_command.restart_last_good_args.v0",
            "cap.recovery.service.restart",
            "recovery_modify_ram",
            "restart the last-good service set",
            check.reason,
            true,
        );
        emit_recovery_lifeline_command_definition(
            "recovery.lifeline.load_artifact_by_hash",
            "raios.recovery_lifeline_command.load_artifact_by_hash_args.v0",
            "cap.recovery.load_artifact",
            "recovery_modify_ram",
            "load a recovery artifact by retained hash evidence",
            check.reason,
            false,
        );
    }
    raw_line("        ],");
    raw_line("        \"required_before_execution\": [");
    raw_line("          \"raios.recovery_lifeline_protocol_state.v0\",");
    raw_line("          \"raios.recovery_loader_runtime_isolation.v0\",");
    raw_line("          \"raios.recovery_rollback_transaction_engine.v0\",");
    raw_line("          \"raios.durable_audit_rollback_persistence.v0\",");
    raw_line("          \"raios.recovery_memory_provenance.v0\"");
    raw_line("        ]");
    raw("      }");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_lifeline_command_definition(
    command_id: &'static str,
    args_schema: &'static str,
    required_capability: &'static str,
    risk: &'static str,
    summary: &'static str,
    denial_reason: &'static str,
    comma: bool,
) {
    raw_line("          {");
    raw("            \"id\": ");
    json_str(command_id);
    raw_line(",");
    raw("            \"argument_schema\": ");
    json_str(args_schema);
    raw_line(",");
    raw("            \"required_capability\": ");
    json_str(required_capability);
    raw_line(",");
    raw("            \"risk\": ");
    json_str(risk);
    raw_line(",");
    raw("            \"summary\": ");
    json_str(summary);
    raw_line(",");
    raw_line("            \"status\": \"defined_non_executable\",");
    raw_line("            \"accepts_envelope\": false,");
    raw_line("            \"dispatches_command\": false,");
    raw_line("            \"authorizes_recovery_load\": false,");
    raw_line("            \"creates_durable_records\": false,");
    raw_line("            \"installs_rollback_plan\": false,");
    raw_line("            \"allocates_service_slot\": false,");
    raw("            \"denial_reason\": ");
    json_str(denial_reason);
    raw_line("");
    raw("          }");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_lifeline_command_vocabulary_check(
    check: &RecoveryLifelineCommandVocabularyCheck,
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
    raw("        \"command_vocabulary_exposed\": ");
    raw_bool(check.command_vocabulary_exposed);
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

pub(crate) fn emit_recovery_lifeline_command_vocabulary_selftest_case(
    case: &RecoveryLifelineCommandVocabularySelfTestCase,
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
    raw(", \"command_execution_enabled\": false, \"accepts_lifeline_command_envelope\": false, \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}
