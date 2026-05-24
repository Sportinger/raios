use crate::{
    agent_protocol_recovery_command_effect_types::{
        RecoveryLifelineCommandDispatchBehaviorReferenceCheck,
        RecoveryLifelineCommandDispatchBehaviorSelfTestCase,
        RecoveryLifelineCommandExecutorCapabilityTableReferenceCheck,
        RecoveryLifelineCommandExecutorCapabilityTableSelfTestCase,
        RecoveryLifelineCommandSideEffectGateReferenceCheck,
        RecoveryLifelineCommandSideEffectGateSelfTestCase,
    },
    agent_protocol_support::{
        crlf, json_event_id, json_event_id_option, json_opt_str, json_sha256, json_sha256_option,
        json_str, raw, raw_bool, raw_line,
    },
    event_log,
};

pub(crate) fn emit_recovery_lifeline_command_dispatch_behavior_reference_object(
    check: &RecoveryLifelineCommandDispatchBehaviorReferenceCheck<'_>,
) {
    raw_line("      \"command_dispatch_behavior_reference\": {");
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
    raw("        \"command_dispatch_behavior_id\": ");
    json_opt_str(check.command_dispatch_behavior_id);
    raw_line(",");
    raw("        \"retained_service_inventory_side_effect_boundary_event_id\": ");
    json_opt_str(check.retained_service_inventory_side_effect_boundary_event_id);
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
    raw("        \"service_inventory_side_effect_boundary_hash\": ");
    json_sha256_option(check.service_inventory_side_effect_boundary_hash);
    raw_line(",");
    raw("        \"command_dispatch_behavior_projection_hash\": ");
    json_sha256_option(check.command_dispatch_behavior_projection_hash);
    raw_line(",");
    raw("        \"command_dispatch_behavior_hash\": ");
    json_sha256_option(check.command_dispatch_behavior_hash);
    raw_line(",");
    raw("        \"expected_command_dispatch_behavior_hash\": ");
    json_sha256_option(check.expected_command_dispatch_behavior_hash);
    raw_line(",");
    raw("        \"valid_hash_reference\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw("      }");
}

pub(crate) fn emit_recovery_lifeline_command_dispatch_behavior_retained_reference(
    check: &RecoveryLifelineCommandDispatchBehaviorReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandDispatchBehaviorReference,
    )>,
) {
    raw_line("      \"retained_recovery_lifeline_command_dispatch_behavior_reference\": {");
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
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw("        \"latest_event_id\": ");
    if let Some((event_id, _)) = retained {
        json_event_id(event_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_command_dispatch_behavior_id\": ");
    if let Some((_, reference)) = retained {
        json_str(reference.command_dispatch_behavior_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_command_dispatch_behavior_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.command_dispatch_behavior_hash);
    } else {
        raw("null");
    }
    raw_line("");
    raw("      }");
}

pub(crate) fn emit_recovery_lifeline_command_dispatch_behavior_selftest_case(
    case: &RecoveryLifelineCommandDispatchBehaviorSelfTestCase,
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
    raw(", \"accepts_raw_command_body\": false, \"accepts_lifeline_command_body\": false, \"accepts_lifeline_command_envelope\": false, \"dispatches_lifeline_command\": false, \"command_execution_enabled\": false, \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_recovery_lifeline_command_executor_capability_table_reference_object(
    check: &RecoveryLifelineCommandExecutorCapabilityTableReferenceCheck<'_>,
) {
    raw_line("      \"executor_capability_table_reference\": {");
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
    raw("        \"executor_capability_table_id\": ");
    json_opt_str(check.executor_capability_table_id);
    raw_line(",");
    raw("        \"retained_command_dispatch_behavior_event_id\": ");
    json_opt_str(check.retained_command_dispatch_behavior_event_id);
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
    raw("        \"service_inventory_side_effect_boundary_hash\": ");
    json_sha256_option(check.service_inventory_side_effect_boundary_hash);
    raw_line(",");
    raw("        \"command_dispatch_behavior_hash\": ");
    json_sha256_option(check.command_dispatch_behavior_hash);
    raw_line(",");
    raw("        \"executor_capability_projection_hash\": ");
    json_sha256_option(check.executor_capability_projection_hash);
    raw_line(",");
    raw("        \"executor_capability_table_hash\": ");
    json_sha256_option(check.executor_capability_table_hash);
    raw_line(",");
    raw("        \"expected_executor_capability_table_hash\": ");
    json_sha256_option(check.expected_executor_capability_table_hash);
    raw_line(",");
    raw("        \"valid_hash_reference\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw("      }");
}

pub(crate) fn emit_recovery_lifeline_command_executor_capability_table_retained_reference(
    check: &RecoveryLifelineCommandExecutorCapabilityTableReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandExecutorCapabilityTableReference,
    )>,
) {
    raw_line("      \"retained_recovery_lifeline_command_executor_capability_table_reference\": {");
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
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw("        \"latest_event_id\": ");
    if let Some((event_id, _)) = retained {
        json_event_id(event_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_executor_capability_table_id\": ");
    if let Some((_, reference)) = retained {
        json_str(reference.executor_capability_table_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_executor_capability_table_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.executor_capability_table_hash);
    } else {
        raw("null");
    }
    raw_line("");
    raw("      }");
}

pub(crate) fn emit_recovery_lifeline_command_executor_capability_table_selftest_case(
    case: &RecoveryLifelineCommandExecutorCapabilityTableSelfTestCase,
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
    raw(", \"accepts_raw_command_body\": false, \"accepts_lifeline_command_body\": false, \"accepts_lifeline_command_envelope\": false, \"dispatches_lifeline_command\": false, \"command_execution_enabled\": false, \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_recovery_lifeline_command_side_effect_gate_reference_object(
    check: &RecoveryLifelineCommandSideEffectGateReferenceCheck<'_>,
) {
    raw_line("      \"side_effect_gate_reference\": {");
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
    raw("        \"side_effect_gate_id\": ");
    json_opt_str(check.side_effect_gate_id);
    raw_line(",");
    raw("        \"retained_executor_capability_table_event_id\": ");
    json_opt_str(check.retained_executor_capability_table_event_id);
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
    raw("        \"service_inventory_side_effect_boundary_hash\": ");
    json_sha256_option(check.service_inventory_side_effect_boundary_hash);
    raw_line(",");
    raw("        \"command_dispatch_behavior_hash\": ");
    json_sha256_option(check.command_dispatch_behavior_hash);
    raw_line(",");
    raw("        \"executor_capability_table_hash\": ");
    json_sha256_option(check.executor_capability_table_hash);
    raw_line(",");
    raw("        \"side_effect_projection_hash\": ");
    json_sha256_option(check.side_effect_projection_hash);
    raw_line(",");
    raw("        \"side_effect_gate_hash\": ");
    json_sha256_option(check.side_effect_gate_hash);
    raw_line(",");
    raw("        \"expected_side_effect_gate_hash\": ");
    json_sha256_option(check.expected_side_effect_gate_hash);
    raw_line(",");
    raw("        \"valid_hash_reference\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw("      }");
}

pub(crate) fn emit_recovery_lifeline_command_side_effect_gate_retained_reference(
    check: &RecoveryLifelineCommandSideEffectGateReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineCommandSideEffectGateReference,
    )>,
) {
    raw_line("      \"retained_recovery_lifeline_command_side_effect_gate_reference\": {");
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
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"creates_service_inventory_records\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw("        \"latest_event_id\": ");
    if let Some((event_id, _)) = retained {
        json_event_id(event_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_side_effect_gate_id\": ");
    if let Some((_, reference)) = retained {
        json_str(reference.side_effect_gate_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_side_effect_gate_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.side_effect_gate_hash);
    } else {
        raw("null");
    }
    raw_line("");
    raw("      }");
}

pub(crate) fn emit_recovery_lifeline_command_side_effect_gate_selftest_case(
    case: &RecoveryLifelineCommandSideEffectGateSelfTestCase,
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
    raw(", \"accepts_raw_command_body\": false, \"accepts_lifeline_command_body\": false, \"accepts_lifeline_command_envelope\": false, \"dispatches_lifeline_command\": false, \"command_execution_enabled\": false, \"allocates_service_slot\": false, \"creates_service_inventory_records\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}
