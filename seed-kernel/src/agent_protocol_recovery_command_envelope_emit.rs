use crate::{
    agent_protocol_recovery_command_dispatch_types::{
        RecoveryLifelineCommandEnvelopeReferenceCheck, RecoveryLifelineCommandEnvelopeSelfTestCase,
    },
    agent_protocol_recovery_lifeline::RecoveryLifelineCommandSpec,
    agent_protocol_support::{
        crlf, json_event_id, json_event_id_option, json_opt_str, json_sha256, json_sha256_option,
        json_str, raw, raw_bool, raw_line,
    },
    event_log,
};

pub(crate) fn emit_recovery_lifeline_command_envelope_allowed_command(
    spec: RecoveryLifelineCommandSpec,
    comma: bool,
) {
    raw_line("        {");
    raw("          \"command_id\": ");
    json_str(spec.command_id);
    raw_line(",");
    raw("          \"argument_schema\": ");
    json_str(spec.argument_schema);
    raw_line(",");
    raw("          \"required_capability\": ");
    json_str(spec.required_capability);
    raw_line(",");
    raw_line("          \"accepts_command_body\": false,");
    raw_line("          \"dispatches_command\": false,");
    raw_line("          \"command_execution_enabled\": false");
    raw("        }");
    if comma {
        raw_line(",");
    } else {
        raw_line("");
    }
}

pub(crate) fn emit_recovery_lifeline_command_envelope_reference_object(
    check: &RecoveryLifelineCommandEnvelopeReferenceCheck<'_>,
) {
    raw_line("      \"command_envelope_reference\": {");
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
    raw("        \"required_capability\": ");
    json_opt_str(check.required_capability);
    raw_line(",");
    raw("        \"target_locator\": ");
    json_opt_str(check.target_locator);
    raw_line(",");
    raw("        \"command_admission_boundary_id\": ");
    json_opt_str(check.command_admission_boundary_id);
    raw_line(",");
    raw("        \"retained_recovery_lifeline_request_event_id\": ");
    json_opt_str(check.retained_lifeline_request_event_id);
    raw_line(",");
    raw("        \"lifeline_request_reference_hash\": ");
    json_sha256_option(check.lifeline_request_reference_hash);
    raw_line(",");
    raw("        \"argument_hash\": ");
    json_sha256_option(check.argument_hash);
    raw_line(",");
    raw("        \"command_envelope_reference_hash\": ");
    json_sha256_option(check.command_envelope_reference_hash);
    raw_line(",");
    raw("        \"expected_command_envelope_reference_hash\": ");
    json_sha256_option(check.expected_command_envelope_reference_hash);
    raw_line(",");
    raw("        \"valid_hash_reference\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
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

pub(crate) fn emit_recovery_lifeline_command_envelope_retained_reference(
    check: &RecoveryLifelineCommandEnvelopeReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandEnvelopeReference,
    )>,
) {
    raw_line("      \"retained_command_envelope_reference\": {");
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
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
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
    raw("        \"latest_command_envelope_reference_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.command_envelope_reference_hash);
    } else {
        raw("null");
    }
    raw_line("");
    raw("      }");
}

pub(crate) fn emit_recovery_lifeline_command_envelope_selftest_case(
    case: &RecoveryLifelineCommandEnvelopeSelfTestCase,
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
    raw(", \"actual_admission_status\": ");
    json_str(case.actual_admission_status);
    raw(", \"actual_admission_reason\": ");
    json_str(case.actual_admission_reason);
    raw(", \"command_admission_boundary_exposed\": ");
    raw_bool(case.command_admission_boundary_exposed);
    raw(", \"command_admission_accepted\": ");
    raw_bool(case.command_admission_accepted);
    raw(", \"command_envelope_reference_present\": ");
    raw_bool(case.command_envelope_reference_present);
    raw(", \"command_id_supported\": ");
    raw_bool(case.command_id_supported);
    raw(", \"argument_schema_matches\": ");
    raw_bool(case.argument_schema_matches);
    raw(", \"argument_hash_present\": ");
    raw_bool(case.argument_hash_present);
    raw(", \"required_capability_matches\": ");
    raw_bool(case.required_capability_matches);
    raw(", \"target_locator_present\": ");
    raw_bool(case.target_locator_present);
    raw(", \"reference_hash_matches\": ");
    raw_bool(case.reference_hash_matches);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"command_execution_enabled\": ");
    raw_bool(case.command_execution_enabled);
    raw(", \"accepts_lifeline_command_envelope\": ");
    raw_bool(case.accepts_lifeline_command_envelope);
    raw(", \"dispatches_lifeline_command\": ");
    raw_bool(case.dispatches_lifeline_command);
    raw(", \"memory_writes_enabled\": false, \"provider_export_enabled\": false, \"durable_writes_enabled\": false, \"rollback_replay_enabled\": false, \"rollback_preview_enabled\": false, \"rollback_apply_enabled\": false, \"authorizes_recovery_load\": ");
    raw_bool(case.authorizes_recovery_load);
    raw(", \"can_move_beyond_denial\": ");
    raw_bool(case.can_move_beyond_denial);
    raw(", \"loads_recovery_loader\": ");
    raw_bool(case.loads_recovery_loader);
    raw(", \"loads_recovery_artifact\": ");
    raw_bool(case.loads_recovery_artifact);
    raw(", \"creates_durable_records\": ");
    raw_bool(case.creates_durable_records);
    raw(", \"installs_rollback_plan\": ");
    raw_bool(case.installs_rollback_plan);
    raw(", \"allocates_service_slot\": ");
    raw_bool(case.allocates_service_slot);
    raw(", \"service_inventory_change\": ");
    json_str(case.service_inventory_change);
    raw(", \"load_attempted\": ");
    raw_bool(case.load_attempted);
    raw("}");
    if comma {
        raw(",");
    }
    crlf();
}
