use crate::{
    agent_protocol_recovery_command_effect_types::{
        RecoveryServiceInventorySideEffectBoundaryReferenceCheck,
        RecoveryServiceInventorySideEffectBoundarySelfTestCase,
    },
    agent_protocol_support::{
        crlf, json_event_id, json_event_id_option, json_opt_str, json_sha256, json_sha256_option,
        json_str, raw, raw_bool, raw_line,
    },
    event_log,
};

pub(crate) fn emit_recovery_service_inventory_side_effect_boundary_reference_object(
    check: &RecoveryServiceInventorySideEffectBoundaryReferenceCheck<'_>,
) {
    raw_line("      \"service_inventory_side_effect_boundary_reference\": {");
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
    raw("        \"service_inventory_side_effect_boundary_id\": ");
    json_opt_str(check.service_inventory_side_effect_boundary_id);
    raw_line(",");
    raw("        \"retained_durable_audit_rollback_write_authority_event_id\": ");
    json_opt_str(check.retained_durable_audit_rollback_write_authority_event_id);
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
    raw("        \"handler_binding_hash\": ");
    json_sha256_option(check.handler_binding_hash);
    raw_line(",");
    raw("        \"status_read_handler_hash\": ");
    json_sha256_option(check.status_read_handler_hash);
    raw_line(",");
    raw("        \"rollback_preview_authorization_hash\": ");
    json_sha256_option(check.rollback_preview_authorization_hash);
    raw_line(",");
    raw("        \"rollback_apply_authorization_hash\": ");
    json_sha256_option(check.rollback_apply_authorization_hash);
    raw_line(",");
    raw("        \"disable_module_target_binding_hash\": ");
    json_sha256_option(check.disable_module_target_binding_hash);
    raw_line(",");
    raw("        \"restart_last_good_target_binding_hash\": ");
    json_sha256_option(check.restart_last_good_target_binding_hash);
    raw_line(",");
    raw("        \"load_artifact_by_hash_target_binding_hash\": ");
    json_sha256_option(check.load_artifact_by_hash_target_binding_hash);
    raw_line(",");
    raw("        \"recovery_memory_write_authority_hash\": ");
    json_sha256_option(check.recovery_memory_write_authority_hash);
    raw_line(",");
    raw("        \"durable_audit_rollback_write_authority_hash\": ");
    json_sha256_option(check.durable_audit_rollback_write_authority_hash);
    raw_line(",");
    raw("        \"service_inventory_projection_hash\": ");
    json_sha256_option(check.service_inventory_projection_hash);
    raw_line(",");
    raw("        \"service_inventory_side_effect_boundary_hash\": ");
    json_sha256_option(check.service_inventory_side_effect_boundary_hash);
    raw_line(",");
    raw("        \"expected_service_inventory_side_effect_boundary_hash\": ");
    json_sha256_option(check.expected_service_inventory_side_effect_boundary_hash);
    raw_line(",");
    raw("        \"valid_hash_reference\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"load_attempted\": false");
    raw("      }");
}

pub(crate) fn emit_recovery_service_inventory_side_effect_boundary_retained_reference(
    check: &RecoveryServiceInventorySideEffectBoundaryReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryServiceInventorySideEffectBoundaryReference,
    )>,
) {
    raw_line("      \"retained_service_inventory_side_effect_boundary_reference\": {");
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
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"load_attempted\": false,");
    raw("        \"latest_event_id\": ");
    if let Some((event_id, _)) = retained {
        json_event_id(event_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_service_inventory_side_effect_boundary_id\": ");
    if let Some((_, reference)) = retained {
        json_str(reference.service_inventory_side_effect_boundary_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_service_inventory_side_effect_boundary_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.service_inventory_side_effect_boundary_hash);
    } else {
        raw("null");
    }
    raw_line("");
    raw("      }");
}

pub(crate) fn emit_recovery_service_inventory_side_effect_boundary_selftest_case(
    case: &RecoveryServiceInventorySideEffectBoundarySelfTestCase,
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
    raw(", \"accepts_raw_command_body\": false, \"dispatches_lifeline_command\": false, \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"command_execution_enabled\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}
