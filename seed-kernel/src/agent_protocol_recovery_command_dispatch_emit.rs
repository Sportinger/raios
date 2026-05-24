use crate::{
    agent_protocol_recovery_command_dispatch_types::{
        RecoveryLifelineCommandDispatchCheck, RecoveryLifelineCommandDispatchSelfTestCase,
    },
    agent_protocol_support::{crlf, json_event_id, json_sha256, json_str, raw, raw_bool, raw_line},
    event_log,
};

pub(crate) fn emit_recovery_lifeline_command_dispatch_retained_envelope(
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandEnvelopeReference,
    )>,
    retained_request: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineRequestReference,
    )>,
) {
    raw_line("      \"retained_command_envelope_reference\": {");
    raw("        \"status\": ");
    json_str(if retained.is_some() {
        "retained_hash_reference_command_still_denied"
    } else {
        "missing"
    });
    raw_line(",");
    raw_line("        \"scope\": \"current_boot\",");
    raw_line("        \"classification\": \"local_only\",");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw("        \"event_id\": ");
    if let Some((event_id, _)) = retained {
        json_event_id(event_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"command_id\": ");
    if let Some((_, reference)) = retained {
        json_str(reference.command_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"argument_schema\": ");
    if let Some((_, reference)) = retained {
        json_str(reference.argument_schema);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"required_capability\": ");
    if let Some((_, reference)) = retained {
        json_str(reference.required_capability);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"target_locator\": ");
    if let Some((_, reference)) = retained {
        json_str(reference.target_locator.as_str());
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"command_admission_boundary_id\": ");
    if let Some((_, reference)) = retained {
        json_str(reference.command_admission_boundary_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"retained_recovery_lifeline_request_event_id\": ");
    if let Some((_, reference)) = retained {
        json_event_id(reference.retained_lifeline_request_event_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"matches_latest_lifeline_request\": ");
    raw_bool(
        if let (Some((_, reference)), Some((request_event_id, request))) =
            (retained, retained_request)
        {
            reference.retained_lifeline_request_event_id == request_event_id
                && reference.lifeline_request_reference_hash
                    == request.lifeline_request_reference_hash
        } else {
            false
        },
    );
    raw_line(",");
    raw("        \"command_envelope_reference_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.command_envelope_reference_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"argument_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.argument_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"lifeline_request_reference_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.lifeline_request_reference_hash);
    } else {
        raw("null");
    }
    raw_line("");
    raw("      }");
}

pub(crate) fn emit_recovery_lifeline_command_dispatch_requirement(
    name: &'static str,
    schema: &'static str,
    present: bool,
    reason: &'static str,
    check: &RecoveryLifelineCommandDispatchCheck,
    comma: bool,
) {
    raw("        {\"fact\": ");
    json_str(name);
    raw(", \"schema\": ");
    json_str(schema);
    raw(", \"status\": ");
    json_str(if present { "present" } else { "missing" });
    raw(", \"required\": true, \"scope\": \"current_boot\", \"classification\": \"local_only\", \"reason\": ");
    json_str(if present {
        "present_for_selftest_only"
    } else {
        reason
    });
    raw(", \"command_envelope_reference_accepted\": ");
    raw_bool(check.command_envelope_reference_accepted);
    raw(", \"accepts_lifeline_command_body\": false, \"dispatches_lifeline_command\": false, \"command_execution_enabled\": false, \"rollback_preview_enabled\": false, \"rollback_apply_enabled\": false, \"recovery_memory_writes_enabled\": false, \"durable_writes_enabled\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_recovery_lifeline_command_dispatch_boundary(
    check: &RecoveryLifelineCommandDispatchCheck,
) {
    raw("        \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw("        \"command_envelope_reference_available\": ");
    raw_bool(check.command_envelope_reference_available);
    raw_line(",");
    raw("        \"command_envelope_reference_accepted\": ");
    raw_bool(check.command_envelope_reference_accepted);
    raw_line(",");
    raw("        \"command_body_canonicalization_present\": ");
    raw_bool(check.command_body_canonicalization_present);
    raw_line(",");
    raw("        \"command_handler_binding_present\": ");
    raw_bool(check.command_handler_binding_present);
    raw_line(",");
    raw("        \"status_read_handler_present\": ");
    raw_bool(check.status_read_handler_present);
    raw_line(",");
    raw("        \"rollback_preview_authorization_present\": ");
    raw_bool(check.rollback_preview_authorization_present);
    raw_line(",");
    raw("        \"rollback_apply_authorization_present\": ");
    raw_bool(check.rollback_apply_authorization_present);
    raw_line(",");
    raw("        \"disable_module_target_binding_present\": ");
    raw_bool(check.disable_module_target_binding_present);
    raw_line(",");
    raw("        \"restart_last_good_target_binding_present\": ");
    raw_bool(check.restart_last_good_target_binding_present);
    raw_line(",");
    raw("        \"load_artifact_by_hash_target_binding_present\": ");
    raw_bool(check.load_artifact_by_hash_target_binding_present);
    raw_line(",");
    raw("        \"recovery_memory_write_authority_present\": ");
    raw_bool(check.recovery_memory_write_authority_present);
    raw_line(",");
    raw("        \"durable_audit_rollback_write_authority_present\": ");
    raw_bool(check.durable_audit_rollback_write_authority_present);
    raw_line(",");
    raw("        \"service_inventory_side_effect_boundary_present\": ");
    raw_bool(check.service_inventory_side_effect_boundary_present);
    raw_line(",");
    raw("        \"command_dispatch_behavior_present\": ");
    raw_bool(check.command_dispatch_behavior_present);
    raw_line(",");
    raw("        \"executor_capability_table_present\": ");
    raw_bool(check.executor_capability_table_present);
    raw_line(",");
    raw("        \"side_effect_gate_present\": ");
    raw_bool(check.side_effect_gate_present);
    raw_line(",");
    raw("        \"execution_enablement_present\": ");
    raw_bool(check.execution_enablement_present);
    raw_line(",");
    raw("        \"execution_preflight_present\": ");
    raw_bool(check.execution_preflight_present);
    raw_line(",");
    raw("        \"execution_intent_present\": ");
    raw_bool(check.execution_intent_present);
    raw_line(",");
    raw("        \"execution_commit_gate_present\": ");
    raw_bool(check.execution_commit_gate_present);
    raw_line(",");
    raw("        \"execution_result_denial_present\": ");
    raw_bool(check.execution_result_denial_present);
    raw_line(",");
    raw("        \"execution_audit_denial_present\": ");
    raw_bool(check.execution_audit_denial_present);
    raw_line(",");
    raw("        \"execution_observation_denial_present\": ");
    raw_bool(check.execution_observation_denial_present);
    raw_line(",");
    raw("        \"execution_completion_denial_present\": ");
    raw_bool(check.execution_completion_denial_present);
    raw_line(",");
    raw("        \"accepts_lifeline_command_body\": ");
    raw_bool(check.accepts_lifeline_command_body);
    raw_line(",");
    raw("        \"accepts_lifeline_command_envelope\": ");
    raw_bool(check.accepts_lifeline_command_envelope);
    raw_line(",");
    raw("        \"dispatches_lifeline_command\": ");
    raw_bool(check.dispatches_lifeline_command);
    raw_line(",");
    raw("        \"command_execution_enabled\": ");
    raw_bool(check.command_execution_enabled);
    raw_line(",");
    raw("        \"rollback_preview_enabled\": ");
    raw_bool(check.rollback_preview_enabled);
    raw_line(",");
    raw("        \"rollback_apply_enabled\": ");
    raw_bool(check.rollback_apply_enabled);
    raw_line(",");
    raw("        \"recovery_memory_writes_enabled\": ");
    raw_bool(check.recovery_memory_writes_enabled);
    raw_line(",");
    raw("        \"durable_writes_enabled\": ");
    raw_bool(check.durable_writes_enabled);
    raw_line(",");
    raw("        \"rollback_replay_enabled\": ");
    raw_bool(check.rollback_replay_enabled);
    raw_line(",");
    raw("        \"provider_export_enabled\": ");
    raw_bool(check.provider_export_enabled);
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
    raw("        \"load_attempted\": ");
    raw_bool(check.load_attempted);
    raw_line("");
}

pub(crate) fn emit_recovery_lifeline_command_dispatch_selftest_case(
    case: &RecoveryLifelineCommandDispatchSelfTestCase,
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
    raw(", \"actual_envelope_status\": ");
    json_str(case.actual_envelope_status);
    raw(", \"actual_envelope_reason\": ");
    json_str(case.actual_envelope_reason);
    raw(", \"command_envelope_reference_accepted\": ");
    raw_bool(case.command_envelope_reference_accepted);
    raw(", \"command_body_canonicalization_present\": ");
    raw_bool(case.command_body_canonicalization_present);
    raw(", \"command_handler_binding_present\": ");
    raw_bool(case.command_handler_binding_present);
    raw(", \"passed\": ");
    raw_bool(case.passed);
    raw(", \"accepts_lifeline_command_body\": false, \"accepts_lifeline_command_envelope\": false, \"dispatches_lifeline_command\": ");
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
