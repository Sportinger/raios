use crate::{
    agent_protocol_recovery_command_dispatch_types::{
        RecoveryLifelineCommandBodyCanonicalizationReferenceCheck,
        RecoveryLifelineCommandBodyCanonicalizationSelfTestCase,
    },
    agent_protocol_support::{
        crlf, json_event_id, json_event_id_option, json_opt_str, json_sha256, json_sha256_option,
        json_str, raw, raw_bool, raw_line,
    },
    event_log,
};

pub(crate) fn emit_recovery_lifeline_command_body_canonicalization_reference_object(
    check: &RecoveryLifelineCommandBodyCanonicalizationReferenceCheck<'_>,
) {
    raw_line("      \"command_body_canonicalization_reference\": {");
    raw("        \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"has_reference\": ");
    raw_bool(check.has_reference);
    raw_line(",");
    raw("        \"arity_valid\": ");
    raw_bool(check.arity_valid);
    raw_line(",");
    raw("        \"scope\": ");
    json_str(check.scope);
    raw_line(",");
    raw("        \"command_id\": ");
    json_opt_str(check.command_id);
    raw_line(",");
    raw("        \"argument_schema\": ");
    json_opt_str(check.argument_schema);
    raw_line(",");
    raw("        \"target_locator\": ");
    json_opt_str(check.target_locator);
    raw_line(",");
    raw("        \"command_dispatch_boundary_id\": ");
    json_opt_str(check.command_dispatch_boundary_id);
    raw_line(",");
    raw("        \"retained_recovery_lifeline_command_envelope_event_id\": ");
    json_opt_str(check.retained_command_envelope_reference_event_id);
    raw_line(",");
    raw("        \"argument_hash\": ");
    json_sha256_option(check.argument_hash);
    raw_line(",");
    raw("        \"command_envelope_reference_hash\": ");
    json_sha256_option(check.command_envelope_reference_hash);
    raw_line(",");
    raw("        \"command_body_canonicalization_hash\": ");
    json_sha256_option(check.command_body_canonicalization_hash);
    raw_line(",");
    raw("        \"expected_command_body_canonicalization_hash\": ");
    json_sha256_option(check.expected_command_body_canonicalization_hash);
    raw_line(",");
    raw("        \"valid_hash_reference\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw("      }");
}

pub(crate) fn emit_recovery_lifeline_command_body_canonicalization_requirement(
    name: &'static str,
    schema: &'static str,
    reason: &'static str,
    comma: bool,
) {
    raw("        {\"fact\": ");
    json_str(name);
    raw(", \"schema\": ");
    json_str(schema);
    raw(", \"status\": \"missing\", \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
    json_str(reason);
    raw(", \"accepts_raw_command_body\": false, \"accepts_lifeline_command_body\": false, \"dispatches_lifeline_command\": false, \"command_execution_enabled\": false, \"rollback_preview_enabled\": false, \"rollback_apply_enabled\": false, \"recovery_memory_writes_enabled\": false, \"durable_writes_enabled\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_recovery_lifeline_command_body_canonicalization_retained_reference(
    check: &RecoveryLifelineCommandBodyCanonicalizationReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandBodyCanonicalizationReference,
    )>,
) {
    raw_line("      \"retained_command_body_canonicalization_reference\": {");
    raw("        \"status\": ");
    json_str(if check.valid {
        "retained_hash_reference_command_still_denied"
    } else if retained.is_some() {
        "previous_retained_hash_reference_present"
    } else {
        "missing"
    });
    raw_line(",");
    raw("        \"recorded_event_id\": ");
    json_event_id_option(recorded_event_id);
    raw_line(",");
    raw_line("        \"scope\": \"current_boot\",");
    raw_line("        \"classification\": \"local_only\",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw("        \"latest_event_id\": ");
    if let Some((event_id, _)) = retained {
        json_event_id(event_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_command_id\": ");
    if let Some((_, reference)) = retained {
        json_str(reference.command_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_command_body_canonicalization_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.command_body_canonicalization_hash);
    } else {
        raw("null");
    }
    raw_line("");
    raw("      }");
}

pub(crate) fn emit_recovery_lifeline_command_body_canonicalization_selftest_case(
    case: &RecoveryLifelineCommandBodyCanonicalizationSelfTestCase,
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
    raw(", \"actual_dispatch_status\": ");
    json_str(case.actual_dispatch_status);
    raw(", \"actual_dispatch_reason\": ");
    json_str(case.actual_dispatch_reason);
    raw(", \"command_body_reference_accepted\": ");
    raw_bool(case.command_body_reference_accepted);
    raw(", \"body_hash_matches\": ");
    raw_bool(case.body_hash_matches);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"accepts_raw_command_body\": false, \"accepts_lifeline_command_body\": false, \"accepts_lifeline_command_envelope\": false, \"dispatches_lifeline_command\": ");
    raw_bool(case.dispatches_lifeline_command);
    raw(", \"command_execution_enabled\": ");
    raw_bool(case.command_execution_enabled);
    raw(", \"memory_writes_enabled\": false, \"provider_export_enabled\": false, \"durable_writes_enabled\": false, \"rollback_replay_enabled\": false, \"rollback_preview_enabled\": false, \"rollback_apply_enabled\": false, \"authorizes_recovery_load\": false, \"can_move_beyond_denial\": false, \"loads_recovery_loader\": false, \"loads_recovery_artifact\": false, \"creates_durable_records\": false, \"installs_rollback_plan\": false, \"allocates_service_slot\": false, \"service_inventory_change\": \"none\", \"load_attempted\": ");
    raw_bool(case.load_attempted);
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}
