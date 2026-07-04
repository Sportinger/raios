use crate::{
    agent_protocol_recovery_command_authorization_types::{
        RecoveryDisableModuleTargetBindingReferenceCheck,
        RecoveryDisableModuleTargetBindingSelfTestCase,
        RecoveryLoadArtifactByHashTargetBindingReferenceCheck,
        RecoveryLoadArtifactByHashTargetBindingSelfTestCase,
        RecoveryRestartLastGoodTargetBindingReferenceCheck,
        RecoveryRestartLastGoodTargetBindingSelfTestCase,
    },
    agent_protocol_support::{
        crlf, json_event_id, json_event_id_option, json_opt_str, json_sha256, json_sha256_option,
        json_str, raw, raw_bool, raw_line,
    },
    event_log,
};

pub(crate) fn emit_recovery_artifact_load_denial_source_evidence() {
    let latest = event_log::latest_recovery_artifact_load_denied_binding();
    let source_present = latest.is_some();

    raw_line("      \"recovery_artifact_load_denial_source\": {");
    raw_line("        \"schema\": \"raios.recovery_artifact_load_denial_source.v0\",");
    raw_line("        \"scope\": \"current_boot\",");
    raw_line("        \"classification\": \"local_only\",");
    raw("        \"status\": ");
    if let Some((_, binding)) = latest {
        json_str(binding.recovery_load_binding_status);
    } else {
        json_str("missing");
    }
    raw_line(",");
    raw("        \"reason\": ");
    if let Some((_, binding)) = latest {
        json_str(binding.recovery_load_binding_reason);
    } else {
        json_str("recovery_artifact_load_denial_event_missing");
    }
    raw_line(",");
    raw("        \"source_evidence_present\": ");
    raw_bool(source_present);
    raw_line(",");
    raw_line("        \"source_method\": \"recovery.load_artifact\",");
    raw_line("        \"read_only\": true,");
    raw_line("        \"mutates_global_event_log\": false,");
    raw_line("        \"creates_retained_records\": false,");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"writes_recovery_memory\": false,");
    raw_line("        \"writes_durable_audit_log\": false,");
    raw_line("        \"writes_rollback_store\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false,");
    raw("        \"retained_recovery_artifact_load_denied_event_id\": ");
    if let Some((event_id, _)) = latest {
        json_event_id(event_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw_line("        \"recovery_load_binding\": {");
    raw("          \"status\": ");
    if let Some((_, binding)) = latest {
        json_str(binding.recovery_load_binding_status);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("          \"reason\": ");
    if let Some((_, binding)) = latest {
        json_str(binding.recovery_load_binding_reason);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("          \"retained_execution_completion_denial_event_id\": ");
    if let Some((_, binding)) = latest {
        json_event_id_option(binding.retained_execution_completion_denial_event_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("          \"execution_completion_denial_hash\": ");
    if let Some((_, binding)) = latest {
        json_sha256_option(binding.execution_completion_denial_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("          \"side_effect_gate_hash\": ");
    if let Some((_, binding)) = latest {
        json_sha256_option(binding.side_effect_gate_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("          \"source_rollback_apply_denial_hash\": ");
    if let Some((_, binding)) = latest {
        json_sha256_option(binding.source_rollback_apply_denial_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("          \"source_durable_policy_write_authority_decision_hash\": ");
    if let Some((_, binding)) = latest {
        json_sha256_option(binding.source_durable_policy_write_authority_decision_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("          \"source_recovery_rollback_inspect_source_reference_hash\": ");
    if let Some((_, binding)) = latest {
        json_sha256_option(binding.source_recovery_rollback_inspect_source_reference_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw_line("          \"loads_recovery_artifact\": false,");
    raw_line("          \"dispatches_lifeline_command\": false,");
    raw_line("          \"command_execution_enabled\": false,");
    raw_line("          \"load_attempted\": false");
    raw_line("        }");
    raw("      }");
}

pub(crate) fn emit_recovery_disable_module_target_binding_reference_object(
    check: &RecoveryDisableModuleTargetBindingReferenceCheck<'_>,
) {
    raw_line("      \"disable_module_target_binding_reference\": {");
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
    raw("        \"disable_module_target_id\": ");
    json_opt_str(check.disable_module_target_id);
    raw_line(",");
    raw("        \"retained_recovery_rollback_apply_authorization_event_id\": ");
    json_opt_str(check.retained_rollback_apply_authorization_event_id);
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
    raw("        \"source_rollback_apply_denial_hash\": ");
    json_sha256_option(check.source_rollback_apply_denial_hash);
    raw_line(",");
    raw("        \"source_durable_policy_write_authority_decision_hash\": ");
    json_sha256_option(check.source_durable_policy_write_authority_decision_hash);
    raw_line(",");
    raw("        \"source_recovery_rollback_inspect_source_reference_hash\": ");
    json_sha256_option(check.source_recovery_rollback_inspect_source_reference_hash);
    raw_line(",");
    raw("        \"disable_module_target_projection_hash\": ");
    json_sha256_option(check.disable_module_target_projection_hash);
    raw_line(",");
    raw("        \"disable_module_target_binding_hash\": ");
    json_sha256_option(check.disable_module_target_binding_hash);
    raw_line(",");
    raw("        \"expected_disable_module_target_binding_hash\": ");
    json_sha256_option(check.expected_disable_module_target_binding_hash);
    raw_line(",");
    raw("        \"valid_hash_reference\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"disables_module\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw("      }");
}

pub(crate) fn emit_recovery_disable_module_target_binding_retained_reference(
    check: &RecoveryDisableModuleTargetBindingReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryDisableModuleTargetBindingReference,
    )>,
) {
    raw_line("      \"retained_disable_module_target_binding_reference\": {");
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
    raw_line("        \"disables_module\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"load_attempted\": false,");
    raw("        \"latest_event_id\": ");
    if let Some((event_id, _)) = retained {
        json_event_id(event_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_disable_module_target_id\": ");
    if let Some((_, reference)) = retained {
        json_str(reference.disable_module_target_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_disable_module_target_binding_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.disable_module_target_binding_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_source_rollback_apply_denial_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.source_rollback_apply_denial_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_source_durable_policy_write_authority_decision_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.source_durable_policy_write_authority_decision_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_source_recovery_rollback_inspect_source_reference_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.source_recovery_rollback_inspect_source_reference_hash);
    } else {
        raw("null");
    }
    raw_line("");
    raw("      }");
}

pub(crate) fn emit_recovery_disable_module_target_binding_selftest_case(
    case: &RecoveryDisableModuleTargetBindingSelfTestCase,
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
    raw(", \"accepts_raw_command_body\": false, \"dispatches_lifeline_command\": false, \"disables_module\": false, \"command_execution_enabled\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_recovery_restart_last_good_target_binding_reference_object(
    check: &RecoveryRestartLastGoodTargetBindingReferenceCheck<'_>,
) {
    raw_line("      \"restart_last_good_target_binding_reference\": {");
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
    raw("        \"restart_last_good_target_id\": ");
    json_opt_str(check.restart_last_good_target_id);
    raw_line(",");
    raw("        \"retained_recovery_disable_module_target_binding_event_id\": ");
    json_opt_str(check.retained_disable_module_target_binding_event_id);
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
    raw("        \"source_rollback_apply_denial_hash\": ");
    json_sha256_option(check.source_rollback_apply_denial_hash);
    raw_line(",");
    raw("        \"source_durable_policy_write_authority_decision_hash\": ");
    json_sha256_option(check.source_durable_policy_write_authority_decision_hash);
    raw_line(",");
    raw("        \"source_recovery_rollback_inspect_source_reference_hash\": ");
    json_sha256_option(check.source_recovery_rollback_inspect_source_reference_hash);
    raw_line(",");
    raw("        \"restart_last_good_target_projection_hash\": ");
    json_sha256_option(check.restart_last_good_target_projection_hash);
    raw_line(",");
    raw("        \"restart_last_good_target_binding_hash\": ");
    json_sha256_option(check.restart_last_good_target_binding_hash);
    raw_line(",");
    raw("        \"expected_restart_last_good_target_binding_hash\": ");
    json_sha256_option(check.expected_restart_last_good_target_binding_hash);
    raw_line(",");
    raw("        \"valid_hash_reference\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"restarts_last_good\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw("      }");
}

pub(crate) fn emit_recovery_restart_last_good_target_binding_retained_reference(
    check: &RecoveryRestartLastGoodTargetBindingReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryRestartLastGoodTargetBindingReference,
    )>,
) {
    raw_line("      \"retained_restart_last_good_target_binding_reference\": {");
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
    raw_line("        \"restarts_last_good\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"load_attempted\": false,");
    raw("        \"latest_event_id\": ");
    if let Some((event_id, _)) = retained {
        json_event_id(event_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_restart_last_good_target_id\": ");
    if let Some((_, reference)) = retained {
        json_str(reference.restart_last_good_target_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_restart_last_good_target_binding_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.restart_last_good_target_binding_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_source_rollback_apply_denial_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.source_rollback_apply_denial_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_source_durable_policy_write_authority_decision_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.source_durable_policy_write_authority_decision_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_source_recovery_rollback_inspect_source_reference_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.source_recovery_rollback_inspect_source_reference_hash);
    } else {
        raw("null");
    }
    raw_line("");
    raw("      }");
}

pub(crate) fn emit_recovery_restart_last_good_target_binding_selftest_case(
    case: &RecoveryRestartLastGoodTargetBindingSelfTestCase,
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
    raw(", \"accepts_raw_command_body\": false, \"dispatches_lifeline_command\": false, \"restarts_last_good\": false, \"command_execution_enabled\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}

pub(crate) fn emit_recovery_load_artifact_by_hash_target_binding_reference_object(
    check: &RecoveryLoadArtifactByHashTargetBindingReferenceCheck<'_>,
) {
    raw_line("      \"load_artifact_by_hash_target_binding_reference\": {");
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
    raw("        \"load_artifact_by_hash_target_id\": ");
    json_opt_str(check.load_artifact_by_hash_target_id);
    raw_line(",");
    raw("        \"retained_recovery_restart_last_good_target_binding_event_id\": ");
    json_opt_str(check.retained_restart_last_good_target_binding_event_id);
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
    raw("        \"source_rollback_apply_denial_hash\": ");
    json_sha256_option(check.source_rollback_apply_denial_hash);
    raw_line(",");
    raw("        \"source_durable_policy_write_authority_decision_hash\": ");
    json_sha256_option(check.source_durable_policy_write_authority_decision_hash);
    raw_line(",");
    raw("        \"source_recovery_rollback_inspect_source_reference_hash\": ");
    json_sha256_option(check.source_recovery_rollback_inspect_source_reference_hash);
    raw_line(",");
    raw("        \"load_artifact_by_hash_target_artifact_hash\": ");
    json_sha256_option(check.load_artifact_by_hash_target_artifact_hash);
    raw_line(",");
    raw("        \"load_artifact_by_hash_target_projection_hash\": ");
    json_sha256_option(check.load_artifact_by_hash_target_projection_hash);
    raw_line(",");
    raw("        \"load_artifact_by_hash_target_binding_hash\": ");
    json_sha256_option(check.load_artifact_by_hash_target_binding_hash);
    raw_line(",");
    raw("        \"expected_load_artifact_by_hash_target_binding_hash\": ");
    json_sha256_option(check.expected_load_artifact_by_hash_target_binding_hash);
    raw_line(",");
    raw("        \"valid_hash_reference\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw("      }");
}

pub(crate) fn emit_recovery_load_artifact_by_hash_target_binding_retained_reference(
    check: &RecoveryLoadArtifactByHashTargetBindingReferenceCheck<'_>,
    recorded_event_id: Option<event_log::EventId>,
    retained: Option<(
        event_log::EventId,
        event_log::RecoveryLoadArtifactByHashTargetBindingReference,
    )>,
) {
    raw_line("      \"retained_load_artifact_by_hash_target_binding_reference\": {");
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
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"load_attempted\": false,");
    raw("        \"latest_event_id\": ");
    if let Some((event_id, _)) = retained {
        json_event_id(event_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_load_artifact_by_hash_target_id\": ");
    if let Some((_, reference)) = retained {
        json_str(reference.load_artifact_by_hash_target_id);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_load_artifact_by_hash_target_binding_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.load_artifact_by_hash_target_binding_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_source_rollback_apply_denial_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.source_rollback_apply_denial_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_source_durable_policy_write_authority_decision_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.source_durable_policy_write_authority_decision_hash);
    } else {
        raw("null");
    }
    raw_line(",");
    raw("        \"latest_source_recovery_rollback_inspect_source_reference_hash\": ");
    if let Some((_, reference)) = retained {
        json_sha256(reference.source_recovery_rollback_inspect_source_reference_hash);
    } else {
        raw("null");
    }
    raw_line("");
    raw("      }");
}

pub(crate) fn emit_recovery_load_artifact_by_hash_target_binding_selftest_case(
    case: &RecoveryLoadArtifactByHashTargetBindingSelfTestCase,
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
    raw(", \"accepts_raw_command_body\": false, \"dispatches_lifeline_command\": false, \"loads_recovery_artifact\": false, \"authorizes_recovery_load\": false, \"command_execution_enabled\": false, \"load_attempted\": false}");
    if comma {
        raw(",");
    }
    crlf();
}
