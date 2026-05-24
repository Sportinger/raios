use crate::{
    agent_protocol_recovery_artifact_reference::*,
    agent_protocol_recovery_artifact_reference_emit::*,
    agent_protocol_recovery_artifact_selftest_emit::*,
    agent_protocol_recovery_command_admission_emit::*,
    agent_protocol_recovery_command_body_emit::*,
    agent_protocol_recovery_command_dispatch_emit::*,
    agent_protocol_recovery_command_effect_emit::*,
    agent_protocol_recovery_command_envelope_emit::*,
    agent_protocol_recovery_command_eval::*,
    agent_protocol_recovery_command_handler_emit::*,
    agent_protocol_recovery_command_reference_eval::*,
    agent_protocol_recovery_constants::*,
    agent_protocol_recovery_durable_write_emit::*,
    agent_protocol_recovery_execution::{
        RECOVERY_LIFELINE_COMMAND_EXECUTION_AUDIT_DENIAL_STAGE,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_COMMIT_GATE_STAGE,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_COMPLETION_DENIAL_STAGE,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_ENABLEMENT_STAGE,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_INTENT_STAGE,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_OBSERVATION_DENIAL_STAGE,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_PREFLIGHT_STAGE,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_RESULT_DENIAL_STAGE,
    },
    agent_protocol_recovery_lifeline::{
        recovery_lifeline_disable_module_command_spec,
        recovery_lifeline_load_artifact_by_hash_command_spec,
        recovery_lifeline_restart_last_good_command_spec,
        recovery_lifeline_rollback_apply_command_spec,
        recovery_lifeline_rollback_preview_command_spec, recovery_lifeline_status_command_spec,
    },
    agent_protocol_recovery_lifeline_command_vocabulary_emit::*,
    agent_protocol_recovery_lifeline_eval::*,
    agent_protocol_recovery_lifeline_protocol_emit::*,
    agent_protocol_recovery_lifeline_protocol_types::*,
    agent_protocol_recovery_load_binding::{
        evaluate_recovery_load_binding, recovery_load_binding_candidate_from_retained,
        recovery_load_binding_retained_loader_mismatch,
        recovery_load_binding_retained_local_approval_mismatch,
        recovery_load_binding_retained_rollback_evidence_mismatch,
        recovery_load_binding_retained_trust_mismatch,
        recovery_load_binding_retained_vm_test_mismatch, recovery_load_binding_selftest_cases,
    },
    agent_protocol_recovery_load_binding_emit::*,
    agent_protocol_recovery_loader_runtime_emit::*,
    agent_protocol_recovery_memory_provenance_emit::*,
    agent_protocol_recovery_memory_write_emit::*,
    agent_protocol_recovery_methods::{
        durable_audit_rollback_write_authority_diagnostic_arg,
        recovery_disable_module_target_binding_diagnostic_arg, recovery_identity_diagnostic_arg,
        recovery_lifeline_command_body_canonicalization_diagnostic_arg,
        recovery_lifeline_command_dispatch_behavior_diagnostic_arg,
        recovery_lifeline_command_envelope_diagnostic_arg,
        recovery_lifeline_command_executor_capability_table_diagnostic_arg,
        recovery_lifeline_command_handler_binding_diagnostic_arg,
        recovery_lifeline_command_side_effect_gate_diagnostic_arg,
        recovery_lifeline_request_diagnostic_arg,
        recovery_lifeline_status_read_handler_diagnostic_arg,
        recovery_load_artifact_by_hash_target_binding_diagnostic_arg,
        recovery_loader_diagnostic_arg, recovery_local_approval_diagnostic_arg,
        recovery_memory_write_authority_diagnostic_arg,
        recovery_restart_last_good_target_binding_diagnostic_arg,
        recovery_rollback_apply_authorization_diagnostic_arg,
        recovery_rollback_evidence_diagnostic_arg,
        recovery_rollback_preview_authorization_diagnostic_arg,
        recovery_service_inventory_side_effect_boundary_diagnostic_arg,
        recovery_trust_diagnostic_arg, recovery_vm_test_diagnostic_arg,
        RECOVERY_ARTIFACT_LOAD_BINDING_METHOD, RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD,
    },
    agent_protocol_recovery_persistence_emit::*,
    agent_protocol_recovery_rollback_apply_emit::*,
    agent_protocol_recovery_rollback_preview_emit::*,
    agent_protocol_recovery_rollback_transaction_emit::*,
    agent_protocol_recovery_service_inventory_effect_emit::*,
    agent_protocol_recovery_status_handler_emit::*,
    agent_protocol_recovery_target_binding_emit::*,
    agent_protocol_support::{
        begin_response, crlf, end_response, json_event_id, json_str, raw, raw_bool, raw_fmt,
        raw_line,
    },
    event_log, module_evidence, serial,
};

pub(crate) fn emit_recovery_artifact_identity_diagnostic(method: &str) {
    let check = parse_recovery_identity_reference(recovery_identity_diagnostic_arg(method));
    let recorded_event_id = if check.valid {
        recovery_identity_binding_from_check(&check)
            .map(event_log::record_recovery_artifact_identity_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_artifact_identity_reference();

    begin_response("recovery.identity_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_artifact_identity_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.identity_diagnostic <identity_reference_hash> <artifact_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"identity_schema\": \"raios.recovery_artifact_identity.v0\",");
    raw_line(
        "        \"identity_canonicalization\": \"raios.recovery_artifact_identity.canonical.v0\"",
    );
    raw_line("      },");
    emit_recovery_identity_reference_object(&check);
    raw_line(",");
    emit_recovery_identity_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"identity_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.identity_diagnostic");
}

pub(crate) fn emit_recovery_artifact_identity_diagnostic_selftest() {
    let cases = recovery_identity_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.identity_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_artifact_identity_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_identity_records\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_identity_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.identity_diagnostic_selftest");
}

pub(crate) fn emit_recovery_artifact_trust_diagnostic(method: &str) {
    let check = parse_recovery_trust_reference(recovery_trust_diagnostic_arg(method), true);
    let recorded_event_id = if check.valid {
        recovery_trust_binding_from_check(&check)
            .map(event_log::record_recovery_artifact_trust_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_artifact_trust_reference();

    begin_response("recovery.trust_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_artifact_trust_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.trust_diagnostic <trust_reference_hash> <retained_identity_event_id> <identity_reference_hash> <artifact_hash> <trust_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"trust_schema\": \"raios.recovery_artifact_trust.v0\",");
    raw_line("        \"trust_canonicalization\": \"raios.recovery_artifact_trust.canonical.v0\"");
    raw_line("      },");
    emit_recovery_trust_reference_object(&check);
    raw_line(",");
    emit_recovery_trust_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"trust_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.trust_diagnostic");
}

pub(crate) fn emit_recovery_artifact_trust_diagnostic_selftest() {
    let cases = recovery_trust_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.trust_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_artifact_trust_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_trust_records\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_trust_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.trust_diagnostic_selftest");
}

pub(crate) fn emit_recovery_artifact_vm_test_diagnostic(method: &str) {
    let check = parse_recovery_vm_test_reference(recovery_vm_test_diagnostic_arg(method), true);
    let recorded_event_id = if check.valid {
        recovery_vm_test_binding_from_check(&check)
            .map(event_log::record_recovery_artifact_vm_test_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_artifact_vm_test_reference();

    begin_response("recovery.vm_test_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_artifact_vm_test_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_vm_test_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.vm_test_diagnostic <vm_test_reference_hash> <retained_identity_event_id> <retained_trust_event_id> <identity_reference_hash> <trust_reference_hash> <artifact_hash> <trust_hash> <vm_test_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"vm_test_schema\": \"raios.recovery_artifact_vm_test.v0\",");
    raw_line(
        "        \"vm_test_canonicalization\": \"raios.recovery_artifact_vm_test.canonical.v0\"",
    );
    raw_line("      },");
    emit_recovery_vm_test_reference_object(&check);
    raw_line(",");
    emit_recovery_vm_test_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"vm_test_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.vm_test_diagnostic");
}

pub(crate) fn emit_recovery_artifact_vm_test_diagnostic_selftest() {
    let cases = recovery_vm_test_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.vm_test_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_artifact_vm_test_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_vm_test_records\": false,");
    raw_line("      \"accepts_vm_test_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_vm_test_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.vm_test_diagnostic_selftest");
}

pub(crate) fn emit_recovery_artifact_local_approval_diagnostic(method: &str) {
    let check = parse_recovery_local_approval_reference(
        recovery_local_approval_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_local_approval_binding_from_check(&check)
            .map(event_log::record_recovery_artifact_local_approval_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_artifact_local_approval_reference();

    begin_response("recovery.local_approval_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_artifact_local_approval_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_local_approval_text\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.local_approval_diagnostic <local_approval_reference_hash> <retained_identity_event_id> <retained_trust_event_id> <retained_vm_test_event_id> <identity_reference_hash> <trust_reference_hash> <vm_test_reference_hash> <artifact_hash> <trust_hash> <vm_test_hash> <local_approval_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"local_approval_schema\": \"raios.recovery_artifact_local_approval.v0\",");
    raw_line(
        "        \"local_approval_canonicalization\": \"raios.recovery_artifact_local_approval.canonical.v0\"",
    );
    raw_line("      },");
    emit_recovery_local_approval_reference_object(&check);
    raw_line(",");
    emit_recovery_local_approval_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"local_approval_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.local_approval_diagnostic");
}

pub(crate) fn emit_recovery_artifact_local_approval_diagnostic_selftest() {
    let cases = recovery_local_approval_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.local_approval_diagnostic_selftest");
    raw_line(
        "      \"schema\": \"raios.recovery_artifact_local_approval_diagnostic_selftest.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_local_approval_records\": false,");
    raw_line("      \"accepts_local_approval_text\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_local_approval_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.local_approval_diagnostic_selftest");
}

pub(crate) fn emit_recovery_artifact_loader_diagnostic(method: &str) {
    let check = parse_recovery_loader_reference(recovery_loader_diagnostic_arg(method), true);
    let recorded_event_id = if check.valid {
        recovery_loader_binding_from_check(&check)
            .map(event_log::record_recovery_artifact_loader_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_artifact_loader_reference();

    begin_response("recovery.loader_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_artifact_loader_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.loader_diagnostic <loader_reference_hash> <retained_identity_event_id> <retained_trust_event_id> <retained_vm_test_event_id> <retained_local_approval_event_id> <identity_reference_hash> <trust_reference_hash> <vm_test_reference_hash> <local_approval_reference_hash> <artifact_hash> <trust_hash> <vm_test_hash> <local_approval_hash> <loader_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"loader_schema\": \"raios.recovery_artifact_loader.v0\",");
    raw_line(
        "        \"loader_canonicalization\": \"raios.recovery_artifact_loader.canonical.v0\"",
    );
    raw_line("      },");
    emit_recovery_loader_reference_object(&check);
    raw_line(",");
    emit_recovery_loader_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"loader_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_loader\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.loader_diagnostic");
}

pub(crate) fn emit_recovery_artifact_loader_diagnostic_selftest() {
    let cases = recovery_loader_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.loader_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_artifact_loader_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_loader_records\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_loader_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.loader_diagnostic_selftest");
}

pub(crate) fn emit_recovery_artifact_rollback_evidence_diagnostic(method: &str) {
    let check = parse_recovery_rollback_evidence_reference(
        recovery_rollback_evidence_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_rollback_evidence_binding_from_check(&check)
            .map(event_log::record_recovery_artifact_rollback_evidence_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_artifact_rollback_evidence_reference();

    begin_response("recovery.rollback_evidence_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_artifact_rollback_evidence_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_rollback_evidence_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.rollback_evidence_diagnostic <rollback_evidence_reference_hash> <retained_identity_event_id> <retained_trust_event_id> <retained_vm_test_event_id> <retained_local_approval_event_id> <retained_loader_event_id> <identity_reference_hash> <trust_reference_hash> <vm_test_reference_hash> <local_approval_reference_hash> <loader_reference_hash> <artifact_hash> <trust_hash> <vm_test_hash> <local_approval_hash> <loader_hash> <rollback_evidence_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line(
        "        \"rollback_evidence_schema\": \"raios.recovery_artifact_rollback_evidence.v0\",",
    );
    raw_line(
        "        \"rollback_evidence_canonicalization\": \"raios.recovery_artifact_rollback_evidence.canonical.v0\"",
    );
    raw_line("      },");
    emit_recovery_rollback_evidence_reference_object(&check);
    raw_line(",");
    emit_recovery_rollback_evidence_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"rollback_evidence_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.rollback_evidence_diagnostic");
}

pub(crate) fn emit_recovery_artifact_rollback_evidence_diagnostic_selftest() {
    let cases = recovery_rollback_evidence_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.rollback_evidence_diagnostic_selftest");
    raw_line(
        "      \"schema\": \"raios.recovery_artifact_rollback_evidence_diagnostic_selftest.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_rollback_evidence_records\": false,");
    raw_line("      \"accepts_rollback_evidence_json\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_rollback_evidence_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.rollback_evidence_diagnostic_selftest");
}

pub(crate) fn emit_recovery_lifeline_request_diagnostic(method: &str) {
    let check = parse_recovery_lifeline_request_reference(
        recovery_lifeline_request_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_lifeline_request_binding_from_check(&check)
            .map(event_log::record_recovery_lifeline_request_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_lifeline_request_reference();

    begin_response("recovery.lifeline_request_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_lifeline_request_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.lifeline_request_diagnostic <lifeline_request_reference_hash> <retained_identity_event_id> <retained_trust_event_id> <retained_vm_test_event_id> <retained_local_approval_event_id> <retained_loader_event_id> <retained_rollback_evidence_event_id> <identity_reference_hash> <trust_reference_hash> <vm_test_reference_hash> <local_approval_reference_hash> <loader_reference_hash> <rollback_evidence_reference_hash> <artifact_hash> <trust_hash> <vm_test_hash> <local_approval_hash> <loader_hash> <rollback_evidence_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"lifeline_request_schema\": \"raios.recovery_lifeline_request.v0\",");
    raw_line(
        "        \"lifeline_request_canonicalization\": \"raios.recovery_lifeline_request.canonical.v0\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_request_reference_object(&check);
    raw_line(",");
    emit_recovery_lifeline_request_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"lifeline_request_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_loader\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"allocates_service_slot\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.lifeline_request_diagnostic");
}

pub(crate) fn emit_recovery_lifeline_request_diagnostic_selftest() {
    let cases = recovery_lifeline_request_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.lifeline_request_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_lifeline_request_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_request_records\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_lifeline_request_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.lifeline_request_diagnostic_selftest");
}

pub(crate) fn emit_recovery_lifeline_protocol_diagnostic() {
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let check =
        evaluate_recovery_lifeline_protocol(recovery_lifeline_protocol_candidate_from_retained(
            retained_request,
            retained_identity,
            retained_trust,
            retained_vm_test,
            retained_local_approval,
            retained_loader,
            retained_rollback_evidence,
        ));

    begin_response("recovery.lifeline_protocol_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_lifeline_protocol_state.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_protocol_state_records\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"lifeline_request_schema\": \"raios.recovery_lifeline_request.v0\",");
    raw_line(
        "        \"lifeline_protocol_state_schema\": \"raios.recovery_lifeline_protocol_state.v0\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_protocol_request_state(retained_request, &check, true);
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_load_vm_test_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        true,
    );
    emit_recovery_load_local_approval_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        true,
    );
    emit_recovery_load_loader_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        true,
    );
    emit_recovery_load_rollback_evidence_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
        false,
    );
    raw_line("      },");
    raw_line("      \"required_protocol_facts\": {");
    emit_recovery_lifeline_protocol_missing_fact(
        "lifeline_protocol_state",
        "raios.recovery_lifeline_protocol_state.v0",
        "recovery_lifeline_protocol_state_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "lifeline_command_vocabulary",
        "raios.recovery_lifeline_command_vocabulary.v0",
        "recovery_lifeline_command_vocabulary_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "loader_runtime_isolation",
        "raios.recovery_loader_runtime_isolation.v0",
        "recovery_loader_runtime_isolation_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "rollback_transaction_engine",
        "raios.recovery_rollback_transaction_engine.v0",
        "recovery_rollback_transaction_engine_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "durable_audit_rollback_persistence",
        "raios.durable_audit_rollback_persistence.v0",
        "durable_audit_rollback_persistence_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "recovery_memory_provenance",
        "raios.recovery_memory_provenance.v0",
        "recovery_memory_provenance_missing",
        false,
    );
    raw_line("      },");
    raw_line("      \"boundary\": {");
    emit_recovery_lifeline_protocol_check(&check);
    raw_line("      }");
    end_response("recovery.lifeline_protocol_diagnostic");
}

pub(crate) fn emit_recovery_lifeline_protocol_diagnostic_selftest() {
    let cases = recovery_lifeline_protocol_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.lifeline_protocol_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_lifeline_protocol_state_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_protocol_state_records\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_lifeline_protocol_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.lifeline_protocol_diagnostic_selftest");
}

pub(crate) fn emit_recovery_lifeline_command_vocabulary() {
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let protocol_candidate = recovery_lifeline_protocol_candidate_from_retained(
        retained_request,
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    );
    let protocol_check = evaluate_recovery_lifeline_protocol(protocol_candidate);
    let check = evaluate_recovery_lifeline_command_vocabulary(
        recovery_lifeline_command_vocabulary_candidate_from_protocol(protocol_candidate),
    );

    begin_response("recovery.lifeline_command_vocabulary");
    raw_line("      \"schema\": \"raios.recovery_lifeline_command_vocabulary.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_command_vocabulary_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"lifeline_request_schema\": \"raios.recovery_lifeline_request.v0\",");
    raw_line(
        "        \"lifeline_protocol_state_schema\": \"raios.recovery_lifeline_protocol_state.v0\",",
    );
    raw_line(
        "        \"lifeline_command_vocabulary_schema\": \"raios.recovery_lifeline_command_vocabulary.v0\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_protocol_request_state(retained_request, &protocol_check, true);
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_load_vm_test_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        true,
    );
    emit_recovery_load_local_approval_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        true,
    );
    emit_recovery_load_loader_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        true,
    );
    emit_recovery_load_rollback_evidence_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
        false,
    );
    raw_line("      },");
    raw_line("      \"required_execution_facts\": {");
    emit_recovery_lifeline_protocol_missing_fact(
        "lifeline_protocol_state",
        "raios.recovery_lifeline_protocol_state.v0",
        "recovery_lifeline_protocol_state_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "loader_runtime_isolation",
        "raios.recovery_loader_runtime_isolation.v0",
        "recovery_loader_runtime_isolation_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "rollback_transaction_engine",
        "raios.recovery_rollback_transaction_engine.v0",
        "recovery_rollback_transaction_engine_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "durable_audit_rollback_persistence",
        "raios.durable_audit_rollback_persistence.v0",
        "durable_audit_rollback_persistence_missing",
        true,
    );
    emit_recovery_lifeline_protocol_missing_fact(
        "recovery_memory_provenance",
        "raios.recovery_memory_provenance.v0",
        "recovery_memory_provenance_missing",
        false,
    );
    raw_line("      },");
    emit_recovery_lifeline_command_vocabulary_object(&check, true);
    raw_line("      \"boundary\": {");
    emit_recovery_lifeline_command_vocabulary_check(&check);
    raw_line("      }");
    end_response("recovery.lifeline_command_vocabulary");
}

pub(crate) fn emit_recovery_lifeline_command_vocabulary_selftest() {
    let cases = recovery_lifeline_command_vocabulary_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.lifeline_command_vocabulary_selftest");
    raw_line("      \"schema\": \"raios.recovery_lifeline_command_vocabulary_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_command_vocabulary_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_lifeline_command_vocabulary_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.lifeline_command_vocabulary_selftest");
}

pub(crate) fn emit_recovery_loader_runtime_isolation() {
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let protocol_candidate = recovery_lifeline_protocol_candidate_from_retained(
        retained_request,
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    );
    let protocol_check = evaluate_recovery_lifeline_protocol(protocol_candidate);
    let command_candidate =
        recovery_lifeline_command_vocabulary_candidate_from_protocol(protocol_candidate);
    let command_check = evaluate_recovery_lifeline_command_vocabulary(command_candidate);
    let isolation_candidate =
        recovery_loader_runtime_isolation_candidate_from_command_vocabulary(command_candidate);
    let check = evaluate_recovery_loader_runtime_isolation(isolation_candidate);

    begin_response("recovery.loader_runtime_isolation");
    raw_line("      \"schema\": \"raios.recovery_loader_runtime_isolation.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_loader_runtime_isolation_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"lifeline_request_schema\": \"raios.recovery_lifeline_request.v0\",");
    raw_line(
        "        \"lifeline_protocol_state_schema\": \"raios.recovery_lifeline_protocol_state.v0\",",
    );
    raw_line(
        "        \"lifeline_command_vocabulary_schema\": \"raios.recovery_lifeline_command_vocabulary.v0\",",
    );
    raw_line(
        "        \"loader_runtime_isolation_schema\": \"raios.recovery_loader_runtime_isolation.v0\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_protocol_request_state(retained_request, &protocol_check, true);
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_load_vm_test_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        true,
    );
    emit_recovery_load_local_approval_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        true,
    );
    emit_recovery_load_loader_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        true,
    );
    emit_recovery_load_rollback_evidence_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
        false,
    );
    raw_line("      },");
    emit_recovery_lifeline_command_vocabulary_object(&command_check, true);
    emit_recovery_loader_runtime_isolation_input_state(&isolation_candidate, &check, true);
    raw_line("      \"required_isolation_facts\": {");
    emit_recovery_loader_runtime_isolation_fact(
        "loader_address_space_boundary",
        "raios.recovery_loader_address_space_boundary.v0",
        isolation_candidate.loader_address_space_boundary_present,
        "recovery_loader_address_space_boundary_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "loader_entrypoint_abi",
        "raios.recovery_loader_entrypoint_abi.v0",
        isolation_candidate.loader_entrypoint_abi_present,
        "recovery_loader_entrypoint_abi_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "loader_memory_map_constraints",
        "raios.recovery_loader_memory_map_constraints.v0",
        isolation_candidate.loader_memory_map_constraints_present,
        "recovery_loader_memory_map_constraints_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "loader_capability_import_table",
        "raios.recovery_loader_capability_import_table.v0",
        isolation_candidate.loader_capability_import_table_present,
        "recovery_loader_capability_import_table_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "loader_artifact_hash_binding",
        "raios.recovery_loader_artifact_hash_binding.v0",
        isolation_candidate.loader_artifact_hash_binding_present,
        "recovery_loader_artifact_hash_binding_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "loader_provider_separation",
        "raios.recovery_loader_provider_separation.v0",
        isolation_candidate.loader_provider_separation_present,
        "recovery_loader_provider_separation_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "loader_normal_module_separation",
        "raios.recovery_loader_normal_module_separation.v0",
        isolation_candidate.loader_normal_module_separation_present,
        "recovery_loader_normal_module_separation_missing",
        false,
    );
    raw_line("      },");
    raw_line("      \"required_downstream_facts\": {");
    emit_recovery_loader_runtime_isolation_fact(
        "rollback_transaction_engine",
        "raios.recovery_rollback_transaction_engine.v0",
        isolation_candidate.rollback_transaction_engine_present,
        "recovery_rollback_transaction_engine_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "durable_audit_rollback_persistence",
        "raios.durable_audit_rollback_persistence.v0",
        isolation_candidate.durable_audit_rollback_persistence_present,
        "durable_audit_rollback_persistence_missing",
        true,
    );
    emit_recovery_loader_runtime_isolation_fact(
        "recovery_memory_provenance",
        "raios.recovery_memory_provenance.v0",
        isolation_candidate.recovery_memory_provenance_present,
        "recovery_memory_provenance_missing",
        false,
    );
    raw_line("      },");
    raw_line("      \"isolation_boundary\": {");
    emit_recovery_loader_runtime_isolation_boundary(&check);
    raw_line("      },");
    raw_line("      \"boundary\": {");
    emit_recovery_loader_runtime_isolation_check(&check);
    raw_line("      }");
    end_response("recovery.loader_runtime_isolation");
}

pub(crate) fn emit_recovery_loader_runtime_isolation_selftest() {
    let cases = recovery_loader_runtime_isolation_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.loader_runtime_isolation_selftest");
    raw_line("      \"schema\": \"raios.recovery_loader_runtime_isolation_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_loader_runtime_isolation_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_loader_runtime_isolation_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"loader_execution_enabled\": false,");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.loader_runtime_isolation_selftest");
}

pub(crate) fn emit_recovery_rollback_transaction_engine() {
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let protocol_candidate = recovery_lifeline_protocol_candidate_from_retained(
        retained_request,
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    );
    let protocol_check = evaluate_recovery_lifeline_protocol(protocol_candidate);
    let command_candidate =
        recovery_lifeline_command_vocabulary_candidate_from_protocol(protocol_candidate);
    let command_check = evaluate_recovery_lifeline_command_vocabulary(command_candidate);
    let isolation_candidate =
        recovery_loader_runtime_isolation_candidate_from_command_vocabulary(command_candidate);
    let isolation_check = evaluate_recovery_loader_runtime_isolation(isolation_candidate);
    let transaction_candidate =
        recovery_rollback_transaction_engine_candidate_from_loader(isolation_candidate);
    let check = evaluate_recovery_rollback_transaction_engine(transaction_candidate);

    begin_response("recovery.rollback_transaction_engine");
    raw_line("      \"schema\": \"raios.recovery_rollback_transaction_engine.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_rollback_transaction_engine_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_rollback_transaction_envelope\": false,");
    raw_line("      \"accepts_rollback_plan_json\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"rollback_read_capability\": \"cap.recovery.rollback.read\",");
    raw_line("        \"rollback_apply_capability\": \"cap.recovery.rollback\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"lifeline_request_schema\": \"raios.recovery_lifeline_request.v0\",");
    raw_line(
        "        \"lifeline_protocol_state_schema\": \"raios.recovery_lifeline_protocol_state.v0\",",
    );
    raw_line(
        "        \"lifeline_command_vocabulary_schema\": \"raios.recovery_lifeline_command_vocabulary.v0\",",
    );
    raw_line(
        "        \"loader_runtime_isolation_schema\": \"raios.recovery_loader_runtime_isolation.v0\",",
    );
    raw_line(
        "        \"rollback_transaction_engine_schema\": \"raios.recovery_rollback_transaction_engine.v0\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_protocol_request_state(retained_request, &protocol_check, true);
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_load_vm_test_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        true,
    );
    emit_recovery_load_local_approval_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        true,
    );
    emit_recovery_load_loader_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        true,
    );
    emit_recovery_load_rollback_evidence_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
        false,
    );
    raw_line("      },");
    emit_recovery_lifeline_command_vocabulary_object(&command_check, true);
    raw_line("      \"loader_runtime_isolation\": {");
    emit_recovery_loader_runtime_isolation_boundary(&isolation_check);
    raw_line("      },");
    emit_recovery_rollback_transaction_engine_input_state(
        &transaction_candidate,
        &isolation_check,
        &check,
        true,
    );
    raw_line("      \"required_transaction_facts\": {");
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_target_selection",
        "raios.recovery_rollback_target_selection.v0",
        transaction_candidate.rollback_target_selection_present,
        "recovery_rollback_target_selection_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_transaction_id_provenance",
        "raios.recovery_rollback_transaction_provenance.v0",
        transaction_candidate.rollback_transaction_id_provenance_present,
        "recovery_rollback_transaction_id_provenance_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_last_good_binding",
        "raios.recovery_rollback_last_good_binding.v0",
        transaction_candidate.rollback_last_good_binding_present,
        "recovery_rollback_last_good_binding_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_disabled_module_set_binding",
        "raios.recovery_rollback_disabled_module_set_binding.v0",
        transaction_candidate.rollback_disabled_module_set_binding_present,
        "recovery_rollback_disabled_module_set_binding_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_artifact_hash_binding",
        "raios.recovery_rollback_artifact_hash_binding.v0",
        transaction_candidate.rollback_artifact_hash_binding_present,
        "recovery_rollback_artifact_hash_binding_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_replay_preconditions",
        "raios.recovery_rollback_replay_preconditions.v0",
        transaction_candidate.rollback_replay_preconditions_present,
        "recovery_rollback_replay_preconditions_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_recovery_capability_import",
        "raios.recovery_rollback_recovery_capability_import.v0",
        transaction_candidate.rollback_recovery_capability_import_present,
        "recovery_rollback_recovery_capability_import_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "rollback_atomic_apply_abort_semantics",
        "raios.recovery_rollback_atomic_apply_abort_semantics.v0",
        transaction_candidate.rollback_atomic_apply_abort_semantics_present,
        "recovery_rollback_atomic_apply_abort_semantics_missing",
        false,
    );
    raw_line("      },");
    raw_line("      \"required_downstream_facts\": {");
    emit_recovery_rollback_transaction_engine_fact(
        "durable_audit_rollback_persistence",
        "raios.durable_audit_rollback_persistence.v0",
        transaction_candidate.durable_audit_rollback_persistence_present,
        "durable_audit_rollback_persistence_missing",
        true,
    );
    emit_recovery_rollback_transaction_engine_fact(
        "recovery_memory_provenance",
        "raios.recovery_memory_provenance.v0",
        transaction_candidate.recovery_memory_provenance_present,
        "recovery_memory_provenance_missing",
        false,
    );
    raw_line("      },");
    raw_line("      \"transaction_boundary\": {");
    emit_recovery_rollback_transaction_engine_boundary(&check);
    raw_line("      },");
    raw_line("      \"boundary\": {");
    emit_recovery_rollback_transaction_engine_check(&check);
    raw_line("      }");
    end_response("recovery.rollback_transaction_engine");
}

pub(crate) fn emit_recovery_rollback_transaction_engine_selftest() {
    let cases = recovery_rollback_transaction_engine_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.rollback_transaction_engine_selftest");
    raw_line("      \"schema\": \"raios.recovery_rollback_transaction_engine_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_rollback_transaction_engine_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_rollback_transaction_envelope\": false,");
    raw_line("      \"accepts_rollback_plan_json\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"rollback_preview_enabled\": false,");
    raw_line("      \"rollback_apply_enabled\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_rollback_transaction_engine_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"rollback_execution_enabled\": false,");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.rollback_transaction_engine_selftest");
}

pub(crate) fn emit_recovery_durable_audit_rollback_persistence() {
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let protocol_candidate = recovery_lifeline_protocol_candidate_from_retained(
        retained_request,
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    );
    let protocol_check = evaluate_recovery_lifeline_protocol(protocol_candidate);
    let command_candidate =
        recovery_lifeline_command_vocabulary_candidate_from_protocol(protocol_candidate);
    let command_check = evaluate_recovery_lifeline_command_vocabulary(command_candidate);
    let isolation_candidate =
        recovery_loader_runtime_isolation_candidate_from_command_vocabulary(command_candidate);
    let isolation_check = evaluate_recovery_loader_runtime_isolation(isolation_candidate);
    let transaction_candidate =
        recovery_rollback_transaction_engine_candidate_from_loader(isolation_candidate);
    let transaction_check = evaluate_recovery_rollback_transaction_engine(transaction_candidate);
    let persistence_candidate =
        recovery_durable_audit_rollback_persistence_candidate_from_transaction(
            transaction_candidate,
        );
    let check = evaluate_recovery_durable_audit_rollback_persistence(persistence_candidate);

    begin_response("recovery.durable_audit_rollback_persistence");
    raw_line("      \"schema\": \"raios.durable_audit_rollback_persistence.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_durable_audit_rollback_persistence_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_rollback_transaction_envelope\": false,");
    raw_line("      \"accepts_rollback_plan_json\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_persistence_device_inventory_json\": false,");
    raw_line("      \"accepts_storage_layout_json\": false,");
    raw_line("      \"accepts_recovery_memory_record\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"replays_rollback_transactions\": false,");
    raw_line("      \"updates_last_good_checkpoint\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"persistence_read_capability\": \"cap.recovery.persistence.read\",");
    raw_line("        \"persistence_write_capability\": \"cap.recovery.persistence\",");
    raw_line("        \"rollback_read_capability\": \"cap.recovery.rollback.read\",");
    raw_line("        \"rollback_apply_capability\": \"cap.recovery.rollback\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline\",");
    raw_line("        \"lifeline_request_schema\": \"raios.recovery_lifeline_request.v0\",");
    raw_line(
        "        \"lifeline_protocol_state_schema\": \"raios.recovery_lifeline_protocol_state.v0\",",
    );
    raw_line(
        "        \"lifeline_command_vocabulary_schema\": \"raios.recovery_lifeline_command_vocabulary.v0\",",
    );
    raw_line(
        "        \"loader_runtime_isolation_schema\": \"raios.recovery_loader_runtime_isolation.v0\",",
    );
    raw_line(
        "        \"rollback_transaction_engine_schema\": \"raios.recovery_rollback_transaction_engine.v0\",",
    );
    raw_line(
        "        \"durable_audit_rollback_persistence_schema\": \"raios.durable_audit_rollback_persistence.v0\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_protocol_request_state(retained_request, &protocol_check, true);
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_load_vm_test_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        true,
    );
    emit_recovery_load_local_approval_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        true,
    );
    emit_recovery_load_loader_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        true,
    );
    emit_recovery_load_rollback_evidence_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
        false,
    );
    raw_line("      },");
    emit_recovery_lifeline_command_vocabulary_object(&command_check, true);
    raw_line("      \"loader_runtime_isolation\": {");
    emit_recovery_loader_runtime_isolation_boundary(&isolation_check);
    raw_line("      },");
    raw_line("      \"rollback_transaction_engine\": {");
    emit_recovery_rollback_transaction_engine_boundary(&transaction_check);
    raw_line("      },");
    emit_recovery_durable_audit_rollback_persistence_input_state(
        &persistence_candidate,
        &transaction_check,
        &check,
        true,
    );
    raw_line("      \"required_persistence_facts\": {");
    emit_recovery_durable_audit_rollback_persistence_fact(
        "persistence_device_inventory",
        "raios.persistence_device_inventory.v0",
        persistence_candidate.persistence_device_inventory_present,
        "persistence_device_inventory_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "storage_layout_identity",
        "raios.durable_audit_rollback_storage_layout_identity.v0",
        persistence_candidate.storage_layout_identity_present,
        "durable_storage_layout_identity_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "audit_append_log_identity",
        "raios.durable_audit_append_log_identity.v0",
        persistence_candidate.audit_append_log_identity_present,
        "durable_audit_append_log_identity_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "rollback_store_identity",
        "raios.rollback_store_identity.v0",
        persistence_candidate.rollback_store_identity_present,
        "rollback_store_identity_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "transaction_replay_cursor",
        "raios.rollback_transaction_replay_cursor.v0",
        persistence_candidate.transaction_replay_cursor_present,
        "rollback_transaction_replay_cursor_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "last_good_checkpoint_binding",
        "raios.recovery_last_good_checkpoint_binding.v0",
        persistence_candidate.last_good_checkpoint_binding_present,
        "recovery_last_good_checkpoint_binding_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "write_ordering",
        "raios.durable_write_ordering.v0",
        persistence_candidate.write_ordering_present,
        "durable_write_ordering_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "crash_consistency",
        "raios.durable_crash_consistency.v0",
        persistence_candidate.crash_consistency_present,
        "durable_crash_consistency_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "integrity_root_hash_chain",
        "raios.durable_integrity_root_hash_chain.v0",
        persistence_candidate.integrity_root_hash_chain_present,
        "durable_integrity_root_hash_chain_missing",
        true,
    );
    emit_recovery_durable_audit_rollback_persistence_fact(
        "recovery_memory_provenance",
        "raios.recovery_memory_provenance.v0",
        persistence_candidate.recovery_memory_provenance_present,
        "recovery_memory_provenance_missing",
        false,
    );
    raw_line("      },");
    raw_line("      \"persistence_boundary\": {");
    emit_recovery_durable_audit_rollback_persistence_boundary(&check);
    raw_line("      },");
    raw_line("      \"boundary\": {");
    emit_recovery_durable_audit_rollback_persistence_check(&check);
    raw_line("      }");
    end_response("recovery.durable_audit_rollback_persistence");
}

pub(crate) fn emit_recovery_durable_audit_rollback_persistence_selftest() {
    let cases = recovery_durable_audit_rollback_persistence_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.durable_audit_rollback_persistence_selftest");
    raw_line("      \"schema\": \"raios.durable_audit_rollback_persistence_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_durable_audit_rollback_persistence_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_rollback_transaction_envelope\": false,");
    raw_line("      \"accepts_rollback_plan_json\": false,");
    raw_line("      \"accepts_lifeline_request_json\": false,");
    raw_line("      \"accepts_loader_descriptor\": false,");
    raw_line("      \"accepts_artifact_bytes\": false,");
    raw_line("      \"accepts_persistence_device_inventory_json\": false,");
    raw_line("      \"accepts_storage_layout_json\": false,");
    raw_line("      \"accepts_recovery_memory_record\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"rollback_preview_enabled\": false,");
    raw_line("      \"rollback_apply_enabled\": false,");
    raw_line("      \"durable_writes_enabled\": false,");
    raw_line("      \"rollback_replay_enabled\": false,");
    raw_line("      \"recovery_memory_writes_enabled\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_durable_audit_rollback_persistence_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"durable_write_enabled\": false,");
    raw_line("      \"rollback_execution_enabled\": false,");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.durable_audit_rollback_persistence_selftest");
}

pub(crate) fn emit_recovery_memory_provenance() {
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let protocol_candidate = recovery_lifeline_protocol_candidate_from_retained(
        retained_request,
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    );
    let protocol_check = evaluate_recovery_lifeline_protocol(protocol_candidate);
    let command_candidate =
        recovery_lifeline_command_vocabulary_candidate_from_protocol(protocol_candidate);
    let command_check = evaluate_recovery_lifeline_command_vocabulary(command_candidate);
    let isolation_candidate =
        recovery_loader_runtime_isolation_candidate_from_command_vocabulary(command_candidate);
    let isolation_check = evaluate_recovery_loader_runtime_isolation(isolation_candidate);
    let transaction_candidate =
        recovery_rollback_transaction_engine_candidate_from_loader(isolation_candidate);
    let transaction_check = evaluate_recovery_rollback_transaction_engine(transaction_candidate);
    let persistence_candidate =
        recovery_durable_audit_rollback_persistence_candidate_from_transaction(
            transaction_candidate,
        );
    let persistence_check =
        evaluate_recovery_durable_audit_rollback_persistence(persistence_candidate);
    let memory_candidate =
        recovery_memory_provenance_candidate_from_persistence(persistence_candidate);
    let check = evaluate_recovery_memory_provenance(memory_candidate);

    begin_response("recovery.memory_provenance");
    raw_line("      \"schema\": \"raios.recovery_memory_provenance.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_memory_provenance_records\": false,");
    raw_line("      \"accepts_memory_record_json\": false,");
    raw_line("      \"accepts_recovery_memory_record\": false,");
    raw_line("      \"accepts_provider_context_export\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_rollback_transaction_envelope\": false,");
    raw_line("      \"accepts_persistence_device_inventory_json\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"replays_rollback_transactions\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"memory_read_capability\": \"cap.recovery.memory.read\",");
    raw_line("        \"memory_write_capability\": \"cap.recovery.memory\",");
    raw_line("        \"persistence_read_capability\": \"cap.recovery.persistence.read\",");
    raw_line("        \"rollback_read_capability\": \"cap.recovery.rollback.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline_memory\",");
    raw_line("        \"lifeline_request_schema\": \"raios.recovery_lifeline_request.v0\",");
    raw_line(
        "        \"lifeline_protocol_state_schema\": \"raios.recovery_lifeline_protocol_state.v0\",",
    );
    raw_line(
        "        \"lifeline_command_vocabulary_schema\": \"raios.recovery_lifeline_command_vocabulary.v0\",",
    );
    raw_line(
        "        \"loader_runtime_isolation_schema\": \"raios.recovery_loader_runtime_isolation.v0\",",
    );
    raw_line(
        "        \"rollback_transaction_engine_schema\": \"raios.recovery_rollback_transaction_engine.v0\",",
    );
    raw_line(
        "        \"durable_audit_rollback_persistence_schema\": \"raios.durable_audit_rollback_persistence.v0\",",
    );
    raw_line("        \"memory_source_record_ids_schema\": \"raios.recovery_memory_source_record_ids.v0\",");
    raw_line("        \"memory_source_schema_hashes_schema\": \"raios.recovery_memory_source_schema_hashes.v0\",");
    raw_line(
        "        \"memory_classification_schema\": \"raios.recovery_memory_classification.v0\",",
    );
    raw_line(
        "        \"memory_authority_level_schema\": \"raios.recovery_memory_authority_level.v0\",",
    );
    raw_line("        \"memory_rollback_transaction_binding_schema\": \"raios.recovery_memory_rollback_transaction_binding.v0\",");
    raw_line("        \"memory_last_good_checkpoint_binding_schema\": \"raios.recovery_memory_last_good_checkpoint_binding.v0\",");
    raw_line(
        "        \"memory_export_profile_schema\": \"raios.recovery_memory_export_profile.v0\",",
    );
    raw_line(
        "        \"memory_redaction_state_schema\": \"raios.recovery_memory_redaction_state.v0\",",
    );
    raw_line(
        "        \"memory_replay_window_schema\": \"raios.recovery_memory_replay_window.v0\",",
    );
    raw_line("        \"memory_audit_linkage_schema\": \"raios.recovery_memory_audit_linkage.v0\"");
    raw_line("      },");
    emit_recovery_lifeline_protocol_request_state(retained_request, &protocol_check, true);
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_load_vm_test_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        true,
    );
    emit_recovery_load_local_approval_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        true,
    );
    emit_recovery_load_loader_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        true,
    );
    emit_recovery_load_rollback_evidence_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
        false,
    );
    raw_line("      },");
    emit_recovery_lifeline_command_vocabulary_object(&command_check, true);
    raw_line("      \"loader_runtime_isolation\": {");
    emit_recovery_loader_runtime_isolation_boundary(&isolation_check);
    raw_line("      },");
    raw_line("      \"rollback_transaction_engine\": {");
    emit_recovery_rollback_transaction_engine_boundary(&transaction_check);
    raw_line("      },");
    raw_line("      \"durable_audit_rollback_persistence\": {");
    emit_recovery_durable_audit_rollback_persistence_boundary(&persistence_check);
    raw_line("      },");
    emit_recovery_memory_provenance_input_state(
        &memory_candidate,
        &persistence_check,
        &check,
        true,
    );
    raw_line("      \"required_memory_provenance_facts\": {");
    emit_recovery_memory_provenance_fact(
        "source_record_ids",
        "raios.recovery_memory_source_record_ids.v0",
        memory_candidate.source_record_ids_present,
        "recovery_memory_source_record_ids_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "source_schema_hashes",
        "raios.recovery_memory_source_schema_hashes.v0",
        memory_candidate.source_schema_hashes_present,
        "recovery_memory_source_schema_hashes_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "memory_classification",
        "raios.recovery_memory_classification.v0",
        memory_candidate.memory_classification_present,
        "recovery_memory_classification_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "memory_authority_level",
        "raios.recovery_memory_authority_level.v0",
        memory_candidate.memory_authority_level_present,
        "recovery_memory_authority_level_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "memory_rollback_transaction_binding",
        "raios.recovery_memory_rollback_transaction_binding.v0",
        memory_candidate.memory_rollback_transaction_binding_present,
        "recovery_memory_rollback_transaction_binding_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "memory_last_good_checkpoint_binding",
        "raios.recovery_memory_last_good_checkpoint_binding.v0",
        memory_candidate.memory_last_good_checkpoint_binding_present,
        "recovery_memory_last_good_checkpoint_binding_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "recovery_only_export_profile",
        "raios.recovery_memory_export_profile.v0",
        memory_candidate.recovery_only_export_profile_present,
        "recovery_only_export_profile_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "memory_redaction_state",
        "raios.recovery_memory_redaction_state.v0",
        memory_candidate.memory_redaction_state_present,
        "recovery_memory_redaction_state_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "memory_replay_window",
        "raios.recovery_memory_replay_window.v0",
        memory_candidate.memory_replay_window_present,
        "recovery_memory_replay_window_missing",
        true,
    );
    emit_recovery_memory_provenance_fact(
        "memory_audit_linkage",
        "raios.recovery_memory_audit_linkage.v0",
        memory_candidate.memory_audit_linkage_present,
        "recovery_memory_audit_linkage_missing",
        false,
    );
    raw_line("      },");
    raw_line("      \"memory_provenance_boundary\": {");
    emit_recovery_memory_provenance_boundary(&check);
    raw_line("      },");
    raw_line("      \"boundary\": {");
    emit_recovery_memory_provenance_check(&check);
    raw_line("      }");
    end_response("recovery.memory_provenance");
}

pub(crate) fn emit_recovery_memory_provenance_selftest() {
    let cases = recovery_memory_provenance_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.memory_provenance_selftest");
    raw_line("      \"schema\": \"raios.recovery_memory_provenance_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_memory_provenance_records\": false,");
    raw_line("      \"accepts_memory_record_json\": false,");
    raw_line("      \"accepts_recovery_memory_record\": false,");
    raw_line("      \"accepts_provider_context_export\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_rollback_transaction_envelope\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"rollback_preview_enabled\": false,");
    raw_line("      \"rollback_apply_enabled\": false,");
    raw_line("      \"durable_writes_enabled\": false,");
    raw_line("      \"rollback_replay_enabled\": false,");
    raw_line("      \"recovery_memory_writes_enabled\": false,");
    raw_line("      \"memory_writes_enabled\": false,");
    raw_line("      \"provider_export_enabled\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_memory_provenance_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"memory_write_enabled\": false,");
    raw_line("      \"provider_export_enabled\": false,");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.memory_provenance_selftest");
}

pub(crate) fn emit_recovery_lifeline_command_admission() {
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let protocol_candidate = recovery_lifeline_protocol_candidate_from_retained(
        retained_request,
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    );
    let protocol_check = evaluate_recovery_lifeline_protocol(protocol_candidate);
    let command_candidate =
        recovery_lifeline_command_vocabulary_candidate_from_protocol(protocol_candidate);
    let command_check = evaluate_recovery_lifeline_command_vocabulary(command_candidate);
    let isolation_candidate =
        recovery_loader_runtime_isolation_candidate_from_command_vocabulary(command_candidate);
    let isolation_check = evaluate_recovery_loader_runtime_isolation(isolation_candidate);
    let transaction_candidate =
        recovery_rollback_transaction_engine_candidate_from_loader(isolation_candidate);
    let transaction_check = evaluate_recovery_rollback_transaction_engine(transaction_candidate);
    let persistence_candidate =
        recovery_durable_audit_rollback_persistence_candidate_from_transaction(
            transaction_candidate,
        );
    let persistence_check =
        evaluate_recovery_durable_audit_rollback_persistence(persistence_candidate);
    let memory_candidate =
        recovery_memory_provenance_candidate_from_persistence(persistence_candidate);
    let memory_check = evaluate_recovery_memory_provenance(memory_candidate);
    let admission_candidate =
        recovery_lifeline_command_admission_candidate_from_memory(memory_candidate);
    let check = evaluate_recovery_lifeline_command_admission(admission_candidate);

    begin_response("recovery.lifeline_command_admission");
    raw_line("      \"schema\": \"raios.recovery_lifeline_command_admission.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_command_admission_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_memory_record_json\": false,");
    raw_line("      \"accepts_provider_context_export\": false,");
    raw_line("      \"accepts_rollback_transaction_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"executes_disable_module\": false,");
    raw_line("      \"executes_restart_last_good\": false,");
    raw_line("      \"executes_load_recovery_artifact_by_hash\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"replays_rollback_transactions\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"request\": {");
    raw_line("        \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("        \"read_capability\": \"cap.recovery.load_artifact.read\",");
    raw_line("        \"command_read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"command_admission_capability\": \"cap.recovery.command.admit\",");
    raw_line("        \"lifeline_request_schema\": \"raios.recovery_lifeline_request.v0\",");
    raw_line("        \"lifeline_protocol_state_schema\": \"raios.recovery_lifeline_protocol_state.v0\",");
    raw_line("        \"lifeline_command_vocabulary_schema\": \"raios.recovery_lifeline_command_vocabulary.v0\",");
    raw_line("        \"loader_runtime_isolation_schema\": \"raios.recovery_loader_runtime_isolation.v0\",");
    raw_line("        \"rollback_transaction_engine_schema\": \"raios.recovery_rollback_transaction_engine.v0\",");
    raw_line("        \"durable_audit_rollback_persistence_schema\": \"raios.durable_audit_rollback_persistence.v0\",");
    raw_line(
        "        \"recovery_memory_provenance_schema\": \"raios.recovery_memory_provenance.v0\",",
    );
    raw_line(
        "        \"command_admission_schema\": \"raios.recovery_lifeline_command_admission.v0\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_protocol_request_state(retained_request, &protocol_check, true);
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_load_vm_test_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        true,
    );
    emit_recovery_load_local_approval_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        true,
    );
    emit_recovery_load_loader_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        true,
    );
    emit_recovery_load_rollback_evidence_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
        false,
    );
    raw_line("      },");
    emit_recovery_lifeline_command_vocabulary_object(&command_check, true);
    raw_line("      \"loader_runtime_isolation\": {");
    emit_recovery_loader_runtime_isolation_boundary(&isolation_check);
    raw_line("      },");
    raw_line("      \"rollback_transaction_engine\": {");
    emit_recovery_rollback_transaction_engine_boundary(&transaction_check);
    raw_line("      },");
    raw_line("      \"durable_audit_rollback_persistence\": {");
    emit_recovery_durable_audit_rollback_persistence_boundary(&persistence_check);
    raw_line("      },");
    raw_line("      \"recovery_memory_provenance\": {");
    emit_recovery_memory_provenance_boundary(&memory_check);
    raw_line("      },");
    emit_recovery_lifeline_command_admission_input_state(
        &admission_candidate,
        &memory_check,
        &check,
        true,
    );
    raw_line("      \"command_admission_requirements\": [");
    emit_recovery_lifeline_command_admission_requirement(
        "recovery.lifeline.status",
        "raios.recovery_lifeline_command.status_args.v0",
        "cap.recovery.load_artifact.read",
        "raios.recovery_lifeline_status_admission.v0",
        admission_candidate.lifeline_status_admission_present,
        "recovery_lifeline_status_command_admission_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_admission_requirement(
        "recovery.lifeline.rollback_preview",
        "raios.recovery_lifeline_command.rollback_preview_args.v0",
        "cap.recovery.rollback.read",
        "raios.recovery_rollback_preview_admission.v0",
        admission_candidate.rollback_preview_admission_present,
        "recovery_rollback_preview_command_admission_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_admission_requirement(
        "recovery.lifeline.rollback_apply",
        "raios.recovery_lifeline_command.rollback_apply_args.v0",
        "cap.recovery.rollback",
        "raios.recovery_rollback_apply_admission.v0",
        admission_candidate.rollback_apply_admission_present,
        "recovery_rollback_apply_command_admission_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_admission_requirement(
        "recovery.lifeline.disable_module",
        "raios.recovery_lifeline_command.disable_module_args.v0",
        "cap.recovery.module.disable",
        "raios.recovery_disable_module_admission.v0",
        admission_candidate.disable_module_admission_present,
        "recovery_disable_module_command_admission_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_admission_requirement(
        "recovery.lifeline.restart_last_good",
        "raios.recovery_lifeline_command.restart_last_good_args.v0",
        "cap.recovery.service.restart",
        "raios.recovery_restart_last_good_admission.v0",
        admission_candidate.restart_last_good_admission_present,
        "recovery_restart_last_good_command_admission_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_admission_requirement(
        "recovery.lifeline.load_artifact_by_hash",
        "raios.recovery_lifeline_command.load_artifact_by_hash_args.v0",
        "cap.recovery.load_artifact",
        "raios.recovery_load_artifact_by_hash_admission.v0",
        admission_candidate.load_recovery_artifact_by_hash_admission_present,
        "recovery_load_artifact_by_hash_command_admission_missing",
        &check,
        false,
    );
    raw_line("      ],");
    raw_line("      \"command_admission_boundary\": {");
    emit_recovery_lifeline_command_admission_boundary(&check);
    raw_line("      },");
    raw_line("      \"boundary\": {");
    emit_recovery_lifeline_command_admission_check(&check);
    raw_line("      }");
    end_response("recovery.lifeline_command_admission");
}

pub(crate) fn emit_recovery_lifeline_command_admission_selftest() {
    let cases = recovery_lifeline_command_admission_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.lifeline_command_admission_selftest");
    raw_line("      \"schema\": \"raios.recovery_lifeline_command_admission_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_command_admission_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"rollback_preview_enabled\": false,");
    raw_line("      \"rollback_apply_enabled\": false,");
    raw_line("      \"durable_writes_enabled\": false,");
    raw_line("      \"rollback_replay_enabled\": false,");
    raw_line("      \"recovery_memory_writes_enabled\": false,");
    raw_line("      \"memory_writes_enabled\": false,");
    raw_line("      \"provider_export_enabled\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_lifeline_command_admission_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.lifeline_command_admission_selftest");
}

pub(crate) fn emit_recovery_lifeline_command_envelope_diagnostic(method: &str) {
    let check = parse_recovery_lifeline_command_envelope_reference(
        recovery_lifeline_command_envelope_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_lifeline_command_envelope_binding_from_check(&check)
            .map(event_log::record_recovery_lifeline_command_envelope_reference)
    } else {
        None
    };
    let retained = event_log::latest_recovery_lifeline_command_envelope_reference();
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let protocol_candidate = recovery_lifeline_protocol_candidate_from_retained(
        retained_request,
        event_log::latest_recovery_artifact_identity_reference(),
        event_log::latest_recovery_artifact_trust_reference(),
        event_log::latest_recovery_artifact_vm_test_reference(),
        event_log::latest_recovery_artifact_local_approval_reference(),
        event_log::latest_recovery_artifact_loader_reference(),
        event_log::latest_recovery_artifact_rollback_evidence_reference(),
    );
    let protocol_check = evaluate_recovery_lifeline_protocol(protocol_candidate);

    begin_response("recovery.lifeline_command_envelope_diagnostic");
    raw_line(
        "      \"schema\": \"raios.recovery_lifeline_command_envelope_reference_diagnostic.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"executes_disable_module\": false,");
    raw_line("      \"executes_restart_last_good\": false,");
    raw_line("      \"executes_load_recovery_artifact_by_hash\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"replays_rollback_transactions\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.lifeline_command_envelope_diagnostic <command_envelope_reference_hash> <retained_lifeline_request_event_id> <command_id> <argument_schema> <argument_hash> <required_capability> <target_locator> <command_admission_boundary_id> <lifeline_request_reference_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline_command\",");
    raw_line("        \"command_reference_schema\": \"raios.recovery_lifeline_command_envelope_reference.v0\",");
    raw_line("        \"command_reference_canonicalization\": \"raios.recovery_lifeline_command_envelope_reference.canonical.v0\",");
    raw_line("        \"command_admission_boundary_id\": \"boundary.recovery_lifeline_command_admission.current_boot\"");
    raw_line("      },");
    emit_recovery_lifeline_protocol_request_state(retained_request, &protocol_check, true);
    raw_line("      \"command_admission_boundary\": {");
    raw_line("        \"schema\": \"raios.recovery_lifeline_command_admission.v0\",");
    raw_line("        \"state\": \"defined_read_only_boundary\",");
    raw_line(
        "        \"boundary_id\": \"boundary.recovery_lifeline_command_admission.current_boot\",",
    );
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"authorizes_command_execution\": false");
    raw_line("      },");
    raw_line("      \"allowed_command_ids\": [");
    emit_recovery_lifeline_command_envelope_allowed_command(
        recovery_lifeline_status_command_spec(),
        true,
    );
    emit_recovery_lifeline_command_envelope_allowed_command(
        recovery_lifeline_rollback_preview_command_spec(),
        true,
    );
    emit_recovery_lifeline_command_envelope_allowed_command(
        recovery_lifeline_rollback_apply_command_spec(),
        true,
    );
    emit_recovery_lifeline_command_envelope_allowed_command(
        recovery_lifeline_disable_module_command_spec(),
        true,
    );
    emit_recovery_lifeline_command_envelope_allowed_command(
        recovery_lifeline_restart_last_good_command_spec(),
        true,
    );
    emit_recovery_lifeline_command_envelope_allowed_command(
        recovery_lifeline_load_artifact_by_hash_command_spec(),
        false,
    );
    raw_line("      ],");
    emit_recovery_lifeline_command_envelope_reference_object(&check);
    raw_line(",");
    emit_recovery_lifeline_command_envelope_retained_reference(&check, recorded_event_id, retained);
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"command_envelope_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_lifeline_command_envelope\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"authorizes_recovery_load\": false,");
    raw_line("        \"can_move_beyond_denial\": false,");
    raw_line("        \"loads_recovery_artifact\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"installs_rollback_plan\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.lifeline_command_envelope_diagnostic");
}

pub(crate) fn emit_recovery_lifeline_command_envelope_diagnostic_selftest() {
    let cases = recovery_lifeline_command_envelope_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.lifeline_command_envelope_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_lifeline_command_envelope_reference_diagnostic_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_command_envelope_records\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"rollback_preview_enabled\": false,");
    raw_line("      \"rollback_apply_enabled\": false,");
    raw_line("      \"memory_writes_enabled\": false,");
    raw_line("      \"provider_export_enabled\": false,");
    raw_line("      \"durable_writes_enabled\": false,");
    raw_line("      \"rollback_replay_enabled\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_lifeline_command_envelope_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.lifeline_command_envelope_diagnostic_selftest");
}

pub(crate) fn emit_recovery_lifeline_command_dispatch_diagnostic() {
    let retained_envelope = event_log::latest_recovery_lifeline_command_envelope_reference();
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let retained_body =
        event_log::latest_recovery_lifeline_command_body_canonicalization_reference();
    let retained_handler = event_log::latest_recovery_lifeline_command_handler_binding_reference();
    let retained_status_handler =
        event_log::latest_recovery_lifeline_status_read_handler_reference();
    let retained_preview_authorization =
        event_log::latest_recovery_rollback_preview_authorization_reference();
    let retained_apply_authorization =
        event_log::latest_recovery_rollback_apply_authorization_reference();
    let retained_disable_module_target_binding =
        event_log::latest_recovery_disable_module_target_binding_reference();
    let retained_restart_last_good_target_binding =
        event_log::latest_recovery_restart_last_good_target_binding_reference();
    let retained_load_artifact_by_hash_target_binding =
        event_log::latest_recovery_load_artifact_by_hash_target_binding_reference();
    let retained_recovery_memory_write_authority =
        event_log::latest_recovery_memory_write_authority_reference();
    let retained_durable_audit_rollback_write_authority =
        event_log::latest_durable_audit_rollback_write_authority_reference();
    let retained_service_inventory_side_effect_boundary =
        event_log::latest_recovery_service_inventory_side_effect_boundary_reference();
    let retained_command_dispatch_behavior =
        event_log::latest_recovery_lifeline_command_dispatch_behavior_reference();
    let retained_executor_capability_table =
        event_log::latest_recovery_lifeline_command_executor_capability_table_reference();
    let retained_side_effect_gate =
        event_log::latest_recovery_lifeline_command_side_effect_gate_reference();
    let retained_execution_enablement =
        event_log::latest_recovery_lifeline_command_execution_stage_reference(
            RECOVERY_LIFELINE_COMMAND_EXECUTION_ENABLEMENT_STAGE.reference_schema,
        );
    let retained_execution_preflight =
        event_log::latest_recovery_lifeline_command_execution_stage_reference(
            RECOVERY_LIFELINE_COMMAND_EXECUTION_PREFLIGHT_STAGE.reference_schema,
        );
    let retained_execution_intent =
        event_log::latest_recovery_lifeline_command_execution_stage_reference(
            RECOVERY_LIFELINE_COMMAND_EXECUTION_INTENT_STAGE.reference_schema,
        );
    let retained_execution_commit_gate =
        event_log::latest_recovery_lifeline_command_execution_stage_reference(
            RECOVERY_LIFELINE_COMMAND_EXECUTION_COMMIT_GATE_STAGE.reference_schema,
        );
    let retained_execution_result_denial =
        event_log::latest_recovery_lifeline_command_execution_stage_reference(
            RECOVERY_LIFELINE_COMMAND_EXECUTION_RESULT_DENIAL_STAGE.reference_schema,
        );
    let retained_execution_audit_denial =
        event_log::latest_recovery_lifeline_command_execution_stage_reference(
            RECOVERY_LIFELINE_COMMAND_EXECUTION_AUDIT_DENIAL_STAGE.reference_schema,
        );
    let retained_execution_observation_denial =
        event_log::latest_recovery_lifeline_command_execution_stage_reference(
            RECOVERY_LIFELINE_COMMAND_EXECUTION_OBSERVATION_DENIAL_STAGE.reference_schema,
        );
    let retained_execution_completion_denial =
        event_log::latest_recovery_lifeline_command_execution_stage_reference(
            RECOVERY_LIFELINE_COMMAND_EXECUTION_COMPLETION_DENIAL_STAGE.reference_schema,
        );
    let candidate = recovery_lifeline_command_dispatch_candidate_from_retained(
        retained_envelope,
        retained_request,
        retained_body,
        retained_handler,
        retained_status_handler,
        retained_preview_authorization,
        retained_apply_authorization,
        retained_disable_module_target_binding,
        retained_restart_last_good_target_binding,
        retained_load_artifact_by_hash_target_binding,
        retained_recovery_memory_write_authority,
        retained_durable_audit_rollback_write_authority,
        retained_service_inventory_side_effect_boundary,
        retained_command_dispatch_behavior,
        retained_executor_capability_table,
        retained_side_effect_gate,
        retained_execution_enablement,
        retained_execution_preflight,
        retained_execution_intent,
        retained_execution_commit_gate,
        retained_execution_result_denial,
        retained_execution_audit_denial,
        retained_execution_observation_denial,
        retained_execution_completion_denial,
    );
    let check = evaluate_recovery_lifeline_command_dispatch(candidate);

    begin_response("recovery.lifeline_command_dispatch_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_lifeline_command_dispatch_denial.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_command_dispatch_records\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"executes_disable_module\": false,");
    raw_line("      \"executes_restart_last_good\": false,");
    raw_line("      \"executes_load_recovery_artifact_by_hash\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"replays_rollback_transactions\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"uses_direct_openai_recovery_path\": false,");
    raw_line("      \"provider_shortcut_used\": false,");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.dispatch\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline_command\",");
    raw_line("        \"command_envelope_reference_schema\": \"raios.recovery_lifeline_command_envelope_reference.v0\",");
    raw_line(
        "        \"command_admission_schema\": \"raios.recovery_lifeline_command_admission.v0\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_command_dispatch_retained_envelope(retained_envelope, retained_request);
    raw_line(",");
    raw_line("      \"command_dispatch_requirements\": [");
    emit_recovery_lifeline_command_dispatch_requirement(
        "command_body_canonicalization",
        "raios.recovery_lifeline_command_body_canonicalization.v0",
        candidate.command_body_canonicalization_present,
        "recovery_lifeline_command_body_canonicalization_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "command_handler_binding",
        "raios.recovery_lifeline_command_handler_binding.v0",
        candidate.command_handler_binding_present,
        "recovery_lifeline_command_handler_binding_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "status_read_handler",
        "raios.recovery_lifeline_status_read_handler.v0",
        candidate.status_read_handler_present,
        "recovery_lifeline_status_read_handler_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "rollback_preview_authorization",
        "raios.recovery_rollback_preview_authorization.v0",
        candidate.rollback_preview_authorization_present,
        "recovery_rollback_preview_authorization_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "rollback_apply_authorization",
        "raios.recovery_rollback_apply_authorization.v0",
        candidate.rollback_apply_authorization_present,
        "recovery_rollback_apply_authorization_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "disable_module_target_binding",
        "raios.recovery_disable_module_target_binding.v0",
        candidate.disable_module_target_binding_present,
        "recovery_disable_module_target_binding_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "restart_last_good_target_binding",
        "raios.recovery_restart_last_good_target_binding.v0",
        candidate.restart_last_good_target_binding_present,
        "recovery_restart_last_good_target_binding_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "load_artifact_by_hash_target_binding",
        "raios.recovery_load_artifact_by_hash_target_binding.v0",
        candidate.load_artifact_by_hash_target_binding_present,
        "recovery_load_artifact_by_hash_target_binding_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "recovery_memory_write_authority",
        "raios.recovery_memory_write_authority.v0",
        candidate.recovery_memory_write_authority_present,
        "recovery_memory_write_authority_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "durable_audit_rollback_write_authority",
        "raios.durable_audit_rollback_write_authority.v0",
        candidate.durable_audit_rollback_write_authority_present,
        "durable_audit_rollback_write_authority_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "service_inventory_side_effect_boundary",
        "raios.recovery_service_inventory_side_effect_boundary.v0",
        candidate.service_inventory_side_effect_boundary_present,
        "recovery_service_inventory_side_effect_boundary_missing",
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "command_dispatch_behavior",
        "raios.recovery_lifeline_command_dispatch_behavior.v0",
        candidate.command_dispatch_behavior_present,
        "recovery_lifeline_command_dispatch_behavior_not_implemented",
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "executor_capability_table",
        "raios.recovery_lifeline_command_executor_capability_table.v0",
        candidate.executor_capability_table_present,
        "recovery_lifeline_command_executor_capability_table_not_implemented",
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "side_effect_gate",
        "raios.recovery_lifeline_command_side_effect_gate.v0",
        candidate.side_effect_gate_present,
        "recovery_lifeline_command_side_effect_gate_not_implemented",
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "command_execution_enablement",
        "raios.recovery_lifeline_command_execution_enablement.v0",
        candidate.execution_enablement_present,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_ENABLEMENT_STAGE.not_implemented_reason,
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "command_execution_preflight",
        "raios.recovery_lifeline_command_execution_preflight.v0",
        candidate.execution_preflight_present,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_PREFLIGHT_STAGE.not_implemented_reason,
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "command_execution_intent",
        "raios.recovery_lifeline_command_execution_intent.v0",
        candidate.execution_intent_present,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_INTENT_STAGE.not_implemented_reason,
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "command_execution_commit_gate",
        "raios.recovery_lifeline_command_execution_commit_gate.v0",
        candidate.execution_commit_gate_present,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_COMMIT_GATE_STAGE.not_implemented_reason,
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "command_execution_result_denial",
        "raios.recovery_lifeline_command_execution_result_denial.v0",
        candidate.execution_result_denial_present,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_RESULT_DENIAL_STAGE.not_implemented_reason,
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "command_execution_audit_denial",
        "raios.recovery_lifeline_command_execution_audit_denial.v0",
        candidate.execution_audit_denial_present,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_AUDIT_DENIAL_STAGE.not_implemented_reason,
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "command_execution_observation_denial",
        "raios.recovery_lifeline_command_execution_observation_denial.v0",
        candidate.execution_observation_denial_present,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_OBSERVATION_DENIAL_STAGE.not_implemented_reason,
        &check,
        true,
    );
    emit_recovery_lifeline_command_dispatch_requirement(
        "command_execution_completion_denial",
        "raios.recovery_lifeline_command_execution_completion_denial.v0",
        candidate.execution_completion_denial_present,
        RECOVERY_LIFELINE_COMMAND_EXECUTION_COMPLETION_DENIAL_STAGE.not_implemented_reason,
        &check,
        false,
    );
    raw_line("      ],");
    raw_line("      \"boundary\": {");
    emit_recovery_lifeline_command_dispatch_boundary(&check);
    raw_line("      }");
    end_response("recovery.lifeline_command_dispatch_diagnostic");
}

pub(crate) fn emit_recovery_lifeline_command_dispatch_diagnostic_selftest() {
    let cases = recovery_lifeline_command_dispatch_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.lifeline_command_dispatch_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_lifeline_command_dispatch_denial_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_command_dispatch_records\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"rollback_preview_enabled\": false,");
    raw_line("      \"rollback_apply_enabled\": false,");
    raw_line("      \"memory_writes_enabled\": false,");
    raw_line("      \"provider_export_enabled\": false,");
    raw_line("      \"durable_writes_enabled\": false,");
    raw_line("      \"rollback_replay_enabled\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_lifeline_command_dispatch_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.lifeline_command_dispatch_diagnostic_selftest");
}

pub(crate) fn emit_recovery_lifeline_command_body_canonicalization_diagnostic(method: &str) {
    let check = parse_recovery_lifeline_command_body_canonicalization_reference(
        recovery_lifeline_command_body_canonicalization_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_lifeline_command_body_canonicalization_binding_from_check(&check)
            .map(event_log::record_recovery_lifeline_command_body_canonicalization_reference)
    } else {
        None
    };
    let retained_body =
        event_log::latest_recovery_lifeline_command_body_canonicalization_reference();
    let retained_envelope = event_log::latest_recovery_lifeline_command_envelope_reference();
    let retained_request = event_log::latest_recovery_lifeline_request_reference();
    let dispatch_candidate = recovery_lifeline_command_dispatch_candidate_from_retained(
        retained_envelope,
        retained_request,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    let dispatch_check = evaluate_recovery_lifeline_command_dispatch(dispatch_candidate);

    begin_response("recovery.lifeline_command_body_canonicalization_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_lifeline_command_body_canonicalization_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw("      \"creates_retained_recovery_lifeline_command_body_canonicalization_records\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"executes_disable_module\": false,");
    raw_line("      \"executes_restart_last_good\": false,");
    raw_line("      \"executes_load_recovery_artifact_by_hash\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"replays_rollback_transactions\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.lifeline_command_body_canonicalization_diagnostic <command_body_canonicalization_hash> <retained_command_envelope_reference_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_dispatch_boundary_id> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline_command_body\",");
    raw_line("        \"command_body_schema\": \"raios.recovery_lifeline_command_body_canonicalization.v0\",");
    raw_line("        \"command_body_canonicalization\": \"raios.recovery_lifeline_command_body_canonicalization.canonical.v0\",");
    raw_line(
        "        \"command_dispatch_boundary_id\": \"boundary.recovery_lifeline_command_dispatch_denial.current_boot\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_command_dispatch_retained_envelope(retained_envelope, retained_request);
    raw_line(",");
    raw_line("      \"command_dispatch_boundary\": {");
    raw_line("        \"schema\": \"raios.recovery_lifeline_command_dispatch_denial.v0\",");
    raw_line(
        "        \"boundary_id\": \"boundary.recovery_lifeline_command_dispatch_denial.current_boot\",",
    );
    raw("        \"status\": ");
    json_str(dispatch_check.status);
    raw_line(",");
    raw("        \"reason\": ");
    json_str(dispatch_check.reason);
    raw_line(",");
    raw_line("        \"expected_status_before_body_canonicalization\": \"denied_missing_lifeline_command_dispatch_boundary\",");
    raw_line("        \"expected_reason_before_body_canonicalization\": \"recovery_lifeline_command_body_canonicalization_missing\",");
    raw("        \"command_envelope_reference_accepted\": ");
    raw_bool(dispatch_check.command_envelope_reference_accepted);
    raw_line(",");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"command_execution_enabled\": false");
    raw_line("      },");
    emit_recovery_lifeline_command_body_canonicalization_reference_object(&check);
    raw_line(",");
    raw_line("      \"body_canonicalization_requirements\": [");
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "per_command_body_schema_canonicalization",
        "raios.recovery_lifeline_command_body_schema_canonicalization.v0",
        "recovery_lifeline_command_body_schema_canonicalization_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "body_redaction_classification",
        "raios.recovery_lifeline_command_body_redaction_classification.v0",
        "recovery_lifeline_command_body_redaction_classification_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "handler_input_binding",
        "raios.recovery_lifeline_command_handler_input_binding.v0",
        "recovery_lifeline_command_handler_input_binding_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "rollback_authorization_linkage",
        "raios.recovery_rollback_authorization_linkage.v0",
        "recovery_rollback_authorization_linkage_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "recovery_memory_write_linkage",
        "raios.recovery_memory_write_linkage.v0",
        "recovery_memory_write_linkage_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "durable_audit_rollback_write_linkage",
        "raios.durable_audit_rollback_write_linkage.v0",
        "durable_audit_rollback_write_linkage_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "service_inventory_side_effect_linkage",
        "raios.recovery_service_inventory_side_effect_linkage.v0",
        "recovery_service_inventory_side_effect_linkage_missing",
        false,
    );
    raw_line("      ],");
    emit_recovery_lifeline_command_body_canonicalization_retained_reference(
        &check,
        recorded_event_id,
        retained_body,
    );
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"command_body_canonicalization_reference_present\": ");
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
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.lifeline_command_body_canonicalization_diagnostic");
}

pub(crate) fn emit_recovery_lifeline_command_body_canonicalization_diagnostic_selftest() {
    let cases = recovery_lifeline_command_body_canonicalization_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.lifeline_command_body_canonicalization_diagnostic_selftest");
    raw_line(
        "      \"schema\": \"raios.recovery_lifeline_command_body_canonicalization_selftest.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_command_body_canonicalization_records\": false,");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"rollback_preview_enabled\": false,");
    raw_line("      \"rollback_apply_enabled\": false,");
    raw_line("      \"memory_writes_enabled\": false,");
    raw_line("      \"provider_export_enabled\": false,");
    raw_line("      \"durable_writes_enabled\": false,");
    raw_line("      \"rollback_replay_enabled\": false,");
    raw_line("      \"loads_recovery_loader\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_lifeline_command_body_canonicalization_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.lifeline_command_body_canonicalization_diagnostic_selftest");
}

pub(crate) fn emit_recovery_lifeline_command_handler_binding_diagnostic(method: &str) {
    let check = parse_recovery_lifeline_command_handler_binding_reference(
        recovery_lifeline_command_handler_binding_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_lifeline_command_handler_binding_from_check(&check)
            .map(event_log::record_recovery_lifeline_command_handler_binding_reference)
    } else {
        None
    };
    let retained_handler = event_log::latest_recovery_lifeline_command_handler_binding_reference();

    begin_response("recovery.lifeline_command_handler_binding_diagnostic");
    raw_line(
        "      \"schema\": \"raios.recovery_lifeline_command_handler_binding_diagnostic.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw("      \"creates_retained_recovery_lifeline_command_handler_binding_records\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.lifeline_command_handler_binding_diagnostic <handler_binding_hash> <retained_command_body_canonicalization_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <command_dispatch_boundary_id> <handler_id> <handler_input_binding_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline_command_handler\",");
    raw_line("        \"handler_binding_schema\": \"raios.recovery_lifeline_command_handler_binding.v0\",");
    raw_line("        \"handler_binding_canonicalization\": \"raios.recovery_lifeline_command_handler_binding.canonical.v0\",");
    raw_line(
        "        \"handler_binding_boundary_id\": \"boundary.recovery_lifeline_command_handler_binding.current_boot\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_command_handler_binding_reference_object(&check);
    raw_line(",");
    raw_line("      \"handler_binding_requirements\": [");
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "status_read_handler",
        "raios.recovery_lifeline_status_read_handler.v0",
        "recovery_lifeline_status_read_handler_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "rollback_preview_authorization",
        "raios.recovery_rollback_preview_authorization.v0",
        "recovery_rollback_preview_authorization_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "rollback_apply_authorization",
        "raios.recovery_rollback_apply_authorization.v0",
        "recovery_rollback_apply_authorization_missing",
        false,
    );
    raw_line("      ],");
    emit_recovery_lifeline_command_handler_binding_retained_reference(
        &check,
        recorded_event_id,
        retained_handler,
    );
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"command_handler_binding_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.lifeline_command_handler_binding_diagnostic");
}

pub(crate) fn emit_recovery_lifeline_command_handler_binding_diagnostic_selftest() {
    let cases = recovery_lifeline_command_handler_binding_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.lifeline_command_handler_binding_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_lifeline_command_handler_binding_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line(
        "      \"creates_retained_recovery_lifeline_command_handler_binding_records\": false,",
    );
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_lifeline_command_handler_binding_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.lifeline_command_handler_binding_diagnostic_selftest");
}

pub(crate) fn emit_recovery_lifeline_status_read_handler_diagnostic(method: &str) {
    let check = parse_recovery_lifeline_status_read_handler_reference(
        recovery_lifeline_status_read_handler_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_lifeline_status_read_handler_from_check(&check)
            .map(event_log::record_recovery_lifeline_status_read_handler_reference)
    } else {
        None
    };
    let retained_status_handler =
        event_log::latest_recovery_lifeline_status_read_handler_reference();

    begin_response("recovery.lifeline_status_read_handler_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_lifeline_status_read_handler_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw("      \"creates_retained_recovery_lifeline_status_read_handler_records\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.lifeline_status_read_handler_diagnostic <status_read_handler_hash> <retained_command_handler_binding_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <command_dispatch_boundary_id> <status_handler_id> <status_read_projection_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline_status_read_handler\",");
    raw_line("        \"status_read_handler_schema\": \"raios.recovery_lifeline_status_read_handler.v0\",");
    raw_line("        \"status_read_handler_canonicalization\": \"raios.recovery_lifeline_status_read_handler.canonical.v0\",");
    raw_line(
        "        \"status_read_handler_boundary_id\": \"boundary.recovery_lifeline_status_read_handler.current_boot\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_status_read_handler_reference_object(&check);
    raw_line(",");
    raw_line("      \"status_read_handler_requirements\": [");
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "rollback_preview_authorization",
        "raios.recovery_rollback_preview_authorization.v0",
        "recovery_rollback_preview_authorization_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "rollback_apply_authorization",
        "raios.recovery_rollback_apply_authorization.v0",
        "recovery_rollback_apply_authorization_missing",
        false,
    );
    raw_line("      ],");
    emit_recovery_lifeline_status_read_handler_retained_reference(
        &check,
        recorded_event_id,
        retained_status_handler,
    );
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"status_read_handler_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"executes_lifeline_status\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.lifeline_status_read_handler_diagnostic");
}

pub(crate) fn emit_recovery_lifeline_status_read_handler_diagnostic_selftest() {
    let cases = recovery_lifeline_status_read_handler_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.lifeline_status_read_handler_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_lifeline_status_read_handler_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_lifeline_status_read_handler_records\": false,");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_lifeline_status_read_handler_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.lifeline_status_read_handler_diagnostic_selftest");
}

pub(crate) fn emit_recovery_rollback_preview_authorization_diagnostic(method: &str) {
    let check = parse_recovery_rollback_preview_authorization_reference(
        recovery_rollback_preview_authorization_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_rollback_preview_authorization_from_check(&check)
            .map(event_log::record_recovery_rollback_preview_authorization_reference)
    } else {
        None
    };
    let retained_preview_authorization =
        event_log::latest_recovery_rollback_preview_authorization_reference();

    begin_response("recovery.rollback_preview_authorization_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_rollback_preview_authorization_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw("      \"creates_retained_recovery_rollback_preview_authorization_records\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.rollback_preview_authorization_diagnostic <rollback_preview_authorization_hash> <retained_status_read_handler_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <command_dispatch_boundary_id> <rollback_preview_authorization_id> <rollback_preview_projection_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_rollback_preview_authorization\",");
    raw_line("        \"rollback_preview_authorization_schema\": \"raios.recovery_rollback_preview_authorization.v0\",");
    raw_line("        \"rollback_preview_authorization_canonicalization\": \"raios.recovery_rollback_preview_authorization.canonical.v0\",");
    raw_line(
        "        \"rollback_preview_authorization_boundary_id\": \"boundary.recovery_rollback_preview_authorization.current_boot\"",
    );
    raw_line("      },");
    emit_recovery_rollback_preview_authorization_reference_object(&check);
    raw_line(",");
    raw_line("      \"rollback_preview_authorization_requirements\": [");
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "rollback_apply_authorization",
        "raios.recovery_rollback_apply_authorization.v0",
        "recovery_rollback_apply_authorization_missing",
        false,
    );
    raw_line("      ],");
    emit_recovery_rollback_preview_authorization_retained_reference(
        &check,
        recorded_event_id,
        retained_preview_authorization,
    );
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"rollback_preview_authorization_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"executes_rollback_preview\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.rollback_preview_authorization_diagnostic");
}

pub(crate) fn emit_recovery_rollback_preview_authorization_diagnostic_selftest() {
    let cases = recovery_rollback_preview_authorization_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.rollback_preview_authorization_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_rollback_preview_authorization_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_rollback_preview_authorization_records\": false,");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_rollback_preview_authorization_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.rollback_preview_authorization_diagnostic_selftest");
}

pub(crate) fn emit_recovery_rollback_apply_authorization_diagnostic(method: &str) {
    let check = parse_recovery_rollback_apply_authorization_reference(
        recovery_rollback_apply_authorization_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_rollback_apply_authorization_from_check(&check)
            .map(event_log::record_recovery_rollback_apply_authorization_reference)
    } else {
        None
    };
    let retained_apply_authorization =
        event_log::latest_recovery_rollback_apply_authorization_reference();

    begin_response("recovery.rollback_apply_authorization_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_rollback_apply_authorization_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw("      \"creates_retained_recovery_rollback_apply_authorization_records\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.rollback_apply_authorization_diagnostic <rollback_apply_authorization_hash> <retained_rollback_preview_authorization_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <command_dispatch_boundary_id> <rollback_apply_authorization_id> <rollback_apply_projection_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_rollback_apply_authorization\",");
    raw_line("        \"rollback_apply_authorization_schema\": \"raios.recovery_rollback_apply_authorization.v0\",");
    raw_line("        \"rollback_apply_authorization_canonicalization\": \"raios.recovery_rollback_apply_authorization.canonical.v0\",");
    raw_line(
        "        \"rollback_apply_authorization_boundary_id\": \"boundary.recovery_rollback_apply_authorization.current_boot\"",
    );
    raw_line("      },");
    emit_recovery_rollback_apply_authorization_reference_object(&check);
    raw_line(",");
    raw_line("      \"rollback_apply_authorization_requirements\": [");
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "disable_module_target_binding",
        "raios.recovery_disable_module_target_binding.v0",
        "recovery_disable_module_target_binding_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "restart_last_good_target_binding",
        "raios.recovery_restart_last_good_target_binding.v0",
        "recovery_restart_last_good_target_binding_missing",
        false,
    );
    raw_line("      ],");
    emit_recovery_rollback_apply_authorization_retained_reference(
        &check,
        recorded_event_id,
        retained_apply_authorization,
    );
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"rollback_apply_authorization_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"executes_rollback_apply\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.rollback_apply_authorization_diagnostic");
}

pub(crate) fn emit_recovery_rollback_apply_authorization_diagnostic_selftest() {
    let cases = recovery_rollback_apply_authorization_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.rollback_apply_authorization_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_rollback_apply_authorization_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_rollback_apply_authorization_records\": false,");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_rollback_apply_authorization_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.rollback_apply_authorization_diagnostic_selftest");
}

pub(crate) fn emit_recovery_disable_module_target_binding_diagnostic(method: &str) {
    let check = parse_recovery_disable_module_target_binding_reference(
        recovery_disable_module_target_binding_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_disable_module_target_binding_from_check(&check)
            .map(event_log::record_recovery_disable_module_target_binding_reference)
    } else {
        None
    };
    let retained_disable_module_target_binding =
        event_log::latest_recovery_disable_module_target_binding_reference();

    begin_response("recovery.disable_module_target_binding_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_disable_module_target_binding_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw("      \"creates_retained_recovery_disable_module_target_binding_records\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"executes_disable_module\": false,");
    raw_line("      \"disables_module\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.disable_module_target_binding_diagnostic <disable_module_target_binding_hash> <retained_rollback_apply_authorization_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <command_dispatch_boundary_id> <disable_module_target_id> <disable_module_target_projection_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_disable_module_target_binding\",");
    raw_line("        \"disable_module_target_binding_schema\": \"raios.recovery_disable_module_target_binding.v0\",");
    raw_line("        \"disable_module_target_binding_canonicalization\": \"raios.recovery_disable_module_target_binding.canonical.v0\",");
    raw_line(
        "        \"disable_module_target_binding_boundary_id\": \"boundary.recovery_disable_module_target_binding.current_boot\"",
    );
    raw_line("      },");
    emit_recovery_disable_module_target_binding_reference_object(&check);
    raw_line(",");
    raw_line("      \"disable_module_target_binding_requirements\": [");
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "restart_last_good_target_binding",
        "raios.recovery_restart_last_good_target_binding.v0",
        "recovery_restart_last_good_target_binding_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "load_artifact_by_hash_target_binding",
        "raios.recovery_load_artifact_by_hash_target_binding.v0",
        "recovery_load_artifact_by_hash_target_binding_missing",
        false,
    );
    raw_line("      ],");
    emit_recovery_disable_module_target_binding_retained_reference(
        &check,
        recorded_event_id,
        retained_disable_module_target_binding,
    );
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"disable_module_target_binding_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"disables_module\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.disable_module_target_binding_diagnostic");
}

pub(crate) fn emit_recovery_disable_module_target_binding_diagnostic_selftest() {
    let cases = recovery_disable_module_target_binding_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.disable_module_target_binding_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_disable_module_target_binding_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_disable_module_target_binding_records\": false,");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"disables_module\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_disable_module_target_binding_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.disable_module_target_binding_diagnostic_selftest");
}

pub(crate) fn emit_recovery_restart_last_good_target_binding_diagnostic(method: &str) {
    let check = parse_recovery_restart_last_good_target_binding_reference(
        recovery_restart_last_good_target_binding_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_restart_last_good_target_binding_from_check(&check)
            .map(event_log::record_recovery_restart_last_good_target_binding_reference)
    } else {
        None
    };
    let retained_restart_last_good_target_binding =
        event_log::latest_recovery_restart_last_good_target_binding_reference();

    begin_response("recovery.restart_last_good_target_binding_diagnostic");
    raw_line(
        "      \"schema\": \"raios.recovery_restart_last_good_target_binding_diagnostic.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw("      \"creates_retained_recovery_restart_last_good_target_binding_records\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"executes_disable_module\": false,");
    raw_line("      \"executes_restart_last_good\": false,");
    raw_line("      \"disables_module\": false,");
    raw_line("      \"restarts_last_good\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.restart_last_good_target_binding_diagnostic <restart_last_good_target_binding_hash> <retained_disable_module_target_binding_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <command_dispatch_boundary_id> <restart_last_good_target_id> <restart_last_good_target_projection_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_restart_last_good_target_binding\",");
    raw_line("        \"restart_last_good_target_binding_schema\": \"raios.recovery_restart_last_good_target_binding.v0\",");
    raw_line("        \"restart_last_good_target_binding_canonicalization\": \"raios.recovery_restart_last_good_target_binding.canonical.v0\",");
    raw_line(
        "        \"restart_last_good_target_binding_boundary_id\": \"boundary.recovery_restart_last_good_target_binding.current_boot\"",
    );
    raw_line("      },");
    emit_recovery_restart_last_good_target_binding_reference_object(&check);
    raw_line(",");
    raw_line("      \"restart_last_good_target_binding_requirements\": [");
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "load_artifact_by_hash_target_binding",
        "raios.recovery_load_artifact_by_hash_target_binding.v0",
        "recovery_load_artifact_by_hash_target_binding_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "recovery_memory_write_authority",
        "raios.recovery_memory_write_authority.v0",
        "recovery_memory_write_authority_missing",
        false,
    );
    raw_line("      ],");
    emit_recovery_restart_last_good_target_binding_retained_reference(
        &check,
        recorded_event_id,
        retained_restart_last_good_target_binding,
    );
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"restart_last_good_target_binding_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"restarts_last_good\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.restart_last_good_target_binding_diagnostic");
}

pub(crate) fn emit_recovery_restart_last_good_target_binding_diagnostic_selftest() {
    let cases = recovery_restart_last_good_target_binding_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.restart_last_good_target_binding_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_restart_last_good_target_binding_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line(
        "      \"creates_retained_recovery_restart_last_good_target_binding_records\": false,",
    );
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"restarts_last_good\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_restart_last_good_target_binding_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.restart_last_good_target_binding_diagnostic_selftest");
}

pub(crate) fn emit_recovery_load_artifact_by_hash_target_binding_diagnostic(method: &str) {
    let check = parse_recovery_load_artifact_by_hash_target_binding_reference(
        recovery_load_artifact_by_hash_target_binding_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_load_artifact_by_hash_target_binding_from_check(&check)
            .map(event_log::record_recovery_load_artifact_by_hash_target_binding_reference)
    } else {
        None
    };
    let retained_load_artifact_by_hash_target_binding =
        event_log::latest_recovery_load_artifact_by_hash_target_binding_reference();

    begin_response("recovery.load_artifact_by_hash_target_binding_diagnostic");
    raw_line(
        "      \"schema\": \"raios.recovery_load_artifact_by_hash_target_binding_diagnostic.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw("      \"creates_retained_recovery_load_artifact_by_hash_target_binding_records\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"executes_disable_module\": false,");
    raw_line("      \"executes_restart_last_good\": false,");
    raw_line("      \"executes_load_recovery_artifact_by_hash\": false,");
    raw_line("      \"disables_module\": false,");
    raw_line("      \"restarts_last_good\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"authorizes_recovery_load\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.load_artifact_by_hash_target_binding_diagnostic <load_artifact_by_hash_target_binding_hash> <retained_restart_last_good_target_binding_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <restart_last_good_target_binding_hash> <command_dispatch_boundary_id> <load_artifact_by_hash_target_id> <load_artifact_by_hash_target_artifact_hash> <load_artifact_by_hash_target_projection_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_load_artifact_by_hash_target_binding\",");
    raw_line("        \"load_artifact_by_hash_target_binding_schema\": \"raios.recovery_load_artifact_by_hash_target_binding.v0\",");
    raw_line("        \"load_artifact_by_hash_target_binding_canonicalization\": \"raios.recovery_load_artifact_by_hash_target_binding.canonical.v0\",");
    raw_line(
        "        \"load_artifact_by_hash_target_binding_boundary_id\": \"boundary.recovery_load_artifact_by_hash_target_binding.current_boot\"",
    );
    raw_line("      },");
    emit_recovery_load_artifact_by_hash_target_binding_reference_object(&check);
    raw_line(",");
    raw_line("      \"load_artifact_by_hash_target_binding_requirements\": [");
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "recovery_memory_write_authority",
        "raios.recovery_memory_write_authority.v0",
        "recovery_memory_write_authority_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "durable_audit_rollback_write_authority",
        "raios.durable_audit_rollback_write_authority.v0",
        "durable_audit_rollback_write_authority_missing",
        false,
    );
    raw_line("      ],");
    emit_recovery_load_artifact_by_hash_target_binding_retained_reference(
        &check,
        recorded_event_id,
        retained_load_artifact_by_hash_target_binding,
    );
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"load_artifact_by_hash_target_binding_reference_present\": ");
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
    raw_line("      }");
    end_response("recovery.load_artifact_by_hash_target_binding_diagnostic");
}

pub(crate) fn emit_recovery_load_artifact_by_hash_target_binding_diagnostic_selftest() {
    let cases = recovery_load_artifact_by_hash_target_binding_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.load_artifact_by_hash_target_binding_diagnostic_selftest");
    raw_line(
        "      \"schema\": \"raios.recovery_load_artifact_by_hash_target_binding_selftest.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line(
        "      \"creates_retained_recovery_load_artifact_by_hash_target_binding_records\": false,",
    );
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"authorizes_recovery_load\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_load_artifact_by_hash_target_binding_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.load_artifact_by_hash_target_binding_diagnostic_selftest");
}

pub(crate) fn emit_recovery_memory_write_authority_diagnostic(method: &str) {
    let check = parse_recovery_memory_write_authority_reference(
        recovery_memory_write_authority_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_memory_write_authority_from_check(&check)
            .map(event_log::record_recovery_memory_write_authority_reference)
    } else {
        None
    };
    let retained_memory_write_authority =
        event_log::latest_recovery_memory_write_authority_reference();

    begin_response("recovery.memory_write_authority_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_memory_write_authority_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw("      \"creates_retained_recovery_memory_write_authority_records\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"executes_disable_module\": false,");
    raw_line("      \"executes_restart_last_good\": false,");
    raw_line("      \"executes_load_recovery_artifact_by_hash\": false,");
    raw_line("      \"disables_module\": false,");
    raw_line("      \"restarts_last_good\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"authorizes_recovery_load\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.memory_write_authority_diagnostic <recovery_memory_write_authority_hash> <retained_load_artifact_by_hash_target_binding_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <restart_last_good_target_binding_hash> <load_artifact_by_hash_target_binding_hash> <command_dispatch_boundary_id> <recovery_memory_write_authority_id> <recovery_memory_projection_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_memory_write_authority\",");
    raw_line("        \"recovery_memory_write_authority_schema\": \"raios.recovery_memory_write_authority.v0\",");
    raw_line("        \"recovery_memory_write_authority_canonicalization\": \"raios.recovery_memory_write_authority.canonical.v0\",");
    raw_line(
        "        \"recovery_memory_write_authority_boundary_id\": \"boundary.recovery_memory_write_authority.current_boot\"",
    );
    raw_line("      },");
    emit_recovery_memory_write_authority_reference_object(&check);
    raw_line(",");
    raw_line("      \"recovery_memory_write_authority_requirements\": [");
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "durable_audit_rollback_write_authority",
        "raios.durable_audit_rollback_write_authority.v0",
        "durable_audit_rollback_write_authority_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "service_inventory_side_effect_boundary",
        "raios.recovery_service_inventory_side_effect_boundary.v0",
        "recovery_service_inventory_side_effect_boundary_missing",
        false,
    );
    raw_line("      ],");
    emit_recovery_memory_write_authority_retained_reference(
        &check,
        recorded_event_id,
        retained_memory_write_authority,
    );
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"recovery_memory_write_authority_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"writes_recovery_memory\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.memory_write_authority_diagnostic");
}

pub(crate) fn emit_recovery_memory_write_authority_diagnostic_selftest() {
    let cases = recovery_memory_write_authority_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.memory_write_authority_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_memory_write_authority_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_memory_write_authority_records\": false,");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_memory_write_authority_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.memory_write_authority_diagnostic_selftest");
}

pub(crate) fn emit_durable_audit_rollback_write_authority_diagnostic(method: &str) {
    let check = parse_durable_audit_rollback_write_authority_reference(
        durable_audit_rollback_write_authority_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        durable_audit_rollback_write_authority_from_check(&check)
            .map(event_log::record_durable_audit_rollback_write_authority_reference)
    } else {
        None
    };
    let retained_durable_write_authority =
        event_log::latest_durable_audit_rollback_write_authority_reference();

    begin_response("recovery.durable_audit_rollback_write_authority_diagnostic");
    raw_line("      \"schema\": \"raios.durable_audit_rollback_write_authority_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw("      \"creates_retained_durable_audit_rollback_write_authority_records\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"executes_disable_module\": false,");
    raw_line("      \"executes_restart_last_good\": false,");
    raw_line("      \"executes_load_recovery_artifact_by_hash\": false,");
    raw_line("      \"disables_module\": false,");
    raw_line("      \"restarts_last_good\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"authorizes_recovery_load\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.durable_audit_rollback_write_authority_diagnostic <durable_audit_rollback_write_authority_hash> <retained_recovery_memory_write_authority_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <restart_last_good_target_binding_hash> <load_artifact_by_hash_target_binding_hash> <recovery_memory_write_authority_hash> <command_dispatch_boundary_id> <durable_audit_rollback_write_authority_id> <durable_audit_rollback_projection_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"durable_audit_rollback_write_authority\",");
    raw_line("        \"durable_audit_rollback_write_authority_schema\": \"raios.durable_audit_rollback_write_authority.v0\",");
    raw_line("        \"durable_audit_rollback_write_authority_canonicalization\": \"raios.durable_audit_rollback_write_authority.canonical.v0\",");
    raw_line(
        "        \"durable_audit_rollback_write_authority_boundary_id\": \"boundary.durable_audit_rollback_write_authority.current_boot\"",
    );
    raw_line("      },");
    emit_durable_audit_rollback_write_authority_reference_object(&check);
    raw_line(",");
    raw_line("      \"durable_audit_rollback_write_authority_requirements\": [");
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "service_inventory_side_effect_boundary",
        "raios.recovery_service_inventory_side_effect_boundary.v0",
        "recovery_service_inventory_side_effect_boundary_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "recovery_lifeline_command_dispatch_behavior",
        "raios.recovery_lifeline_command_dispatch_behavior.v0",
        "recovery_lifeline_command_dispatch_behavior_not_implemented",
        false,
    );
    raw_line("      ],");
    emit_durable_audit_rollback_write_authority_retained_reference(
        &check,
        recorded_event_id,
        retained_durable_write_authority,
    );
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"durable_audit_rollback_write_authority_reference_present\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("        \"accepts_raw_command_body\": false,");
    raw_line("        \"accepts_lifeline_command_body\": false,");
    raw_line("        \"dispatches_lifeline_command\": false,");
    raw_line("        \"writes_durable_audit_log\": false,");
    raw_line("        \"writes_rollback_store\": false,");
    raw_line("        \"creates_durable_records\": false,");
    raw_line("        \"command_execution_enabled\": false,");
    raw_line("        \"service_inventory_change\": \"none\",");
    raw_line("        \"load_attempted\": false");
    raw_line("      }");
    end_response("recovery.durable_audit_rollback_write_authority_diagnostic");
}

pub(crate) fn emit_durable_audit_rollback_write_authority_diagnostic_selftest() {
    let cases = durable_audit_rollback_write_authority_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.durable_audit_rollback_write_authority_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.durable_audit_rollback_write_authority_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_durable_audit_rollback_write_authority_records\": false,");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_durable_audit_rollback_write_authority_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.durable_audit_rollback_write_authority_diagnostic_selftest");
}

pub(crate) fn emit_recovery_service_inventory_side_effect_boundary_diagnostic(method: &str) {
    let check = parse_recovery_service_inventory_side_effect_boundary_reference(
        recovery_service_inventory_side_effect_boundary_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_service_inventory_side_effect_boundary_from_check(&check)
            .map(event_log::record_recovery_service_inventory_side_effect_boundary_reference)
    } else {
        None
    };
    let retained_service_inventory_side_effect_boundary =
        event_log::latest_recovery_service_inventory_side_effect_boundary_reference();

    begin_response("recovery.service_inventory_side_effect_boundary_diagnostic");
    raw_line("      \"schema\": \"raios.recovery_service_inventory_side_effect_boundary_diagnostic.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw("      \"creates_retained_recovery_service_inventory_side_effect_boundary_records\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"executes_disable_module\": false,");
    raw_line("      \"executes_restart_last_good\": false,");
    raw_line("      \"executes_load_recovery_artifact_by_hash\": false,");
    raw_line("      \"disables_module\": false,");
    raw_line("      \"restarts_last_good\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"authorizes_recovery_load\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.service_inventory_side_effect_boundary_diagnostic <service_inventory_side_effect_boundary_hash> <retained_durable_audit_rollback_write_authority_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <restart_last_good_target_binding_hash> <load_artifact_by_hash_target_binding_hash> <recovery_memory_write_authority_hash> <durable_audit_rollback_write_authority_hash> <command_dispatch_boundary_id> <service_inventory_side_effect_boundary_id> <service_inventory_projection_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"service_inventory_side_effect_boundary\",");
    raw_line("        \"service_inventory_side_effect_boundary_schema\": \"raios.recovery_service_inventory_side_effect_boundary.v0\",");
    raw_line("        \"service_inventory_side_effect_boundary_canonicalization\": \"raios.recovery_service_inventory_side_effect_boundary.canonical.v0\",");
    raw_line(
        "        \"service_inventory_side_effect_boundary_id\": \"boundary.recovery_service_inventory_side_effect_boundary.current_boot\"",
    );
    raw_line("      },");
    emit_recovery_service_inventory_side_effect_boundary_reference_object(&check);
    raw_line(",");
    raw_line("      \"service_inventory_side_effect_boundary_requirements\": [");
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "recovery_lifeline_command_dispatch_behavior",
        "raios.recovery_lifeline_command_dispatch_behavior.v0",
        "recovery_lifeline_command_dispatch_behavior_not_implemented",
        false,
    );
    raw_line("      ],");
    emit_recovery_service_inventory_side_effect_boundary_retained_reference(
        &check,
        recorded_event_id,
        retained_service_inventory_side_effect_boundary,
    );
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"service_inventory_side_effect_boundary_reference_present\": ");
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
    raw_line("      }");
    end_response("recovery.service_inventory_side_effect_boundary_diagnostic");
}

pub(crate) fn emit_recovery_service_inventory_side_effect_boundary_diagnostic_selftest() {
    let cases = recovery_service_inventory_side_effect_boundary_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.service_inventory_side_effect_boundary_diagnostic_selftest");
    raw_line(
        "      \"schema\": \"raios.recovery_service_inventory_side_effect_boundary_selftest.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_service_inventory_side_effect_boundary_records\": false,");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"command_execution_enabled\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_service_inventory_side_effect_boundary_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.service_inventory_side_effect_boundary_diagnostic_selftest");
}

pub(crate) fn emit_recovery_lifeline_command_dispatch_behavior_diagnostic(method: &str) {
    let check = parse_recovery_lifeline_command_dispatch_behavior_reference(
        recovery_lifeline_command_dispatch_behavior_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_lifeline_command_dispatch_behavior_from_check(&check)
            .map(event_log::record_recovery_lifeline_command_dispatch_behavior_reference)
    } else {
        None
    };
    let retained_command_dispatch_behavior =
        event_log::latest_recovery_lifeline_command_dispatch_behavior_reference();

    begin_response("recovery.lifeline_command_dispatch_behavior_diagnostic");
    raw_line(
        "      \"schema\": \"raios.recovery_lifeline_command_dispatch_behavior_diagnostic.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw("      \"creates_retained_recovery_lifeline_command_dispatch_behavior_records\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"executes_disable_module\": false,");
    raw_line("      \"executes_restart_last_good\": false,");
    raw_line("      \"executes_load_recovery_artifact_by_hash\": false,");
    raw_line("      \"disables_module\": false,");
    raw_line("      \"restarts_last_good\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"authorizes_recovery_load\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.lifeline_command_dispatch_behavior_diagnostic <command_dispatch_behavior_hash> <retained_service_inventory_side_effect_boundary_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <restart_last_good_target_binding_hash> <load_artifact_by_hash_target_binding_hash> <recovery_memory_write_authority_hash> <durable_audit_rollback_write_authority_hash> <service_inventory_side_effect_boundary_hash> <command_dispatch_boundary_id> <command_dispatch_behavior_id> <command_dispatch_behavior_projection_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline_command_dispatch_behavior\",");
    raw_line("        \"command_dispatch_behavior_schema\": \"raios.recovery_lifeline_command_dispatch_behavior.v0\",");
    raw_line("        \"command_dispatch_behavior_canonicalization\": \"raios.recovery_lifeline_command_dispatch_behavior.canonical.v0\",");
    raw_line(
        "        \"command_dispatch_behavior_id\": \"boundary.recovery_lifeline_command_dispatch_behavior.current_boot\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_command_dispatch_behavior_reference_object(&check);
    raw_line(",");
    raw_line("      \"command_dispatch_behavior_requirements\": [");
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "executor_capability_table",
        "raios.recovery_lifeline_command_executor_capability_table.v0",
        "recovery_lifeline_command_executor_capability_table_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "side_effect_gate",
        "raios.recovery_lifeline_command_side_effect_gate.v0",
        "recovery_lifeline_command_side_effect_gate_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "command_execution_enablement",
        "raios.recovery_lifeline_command_execution_enablement.v0",
        "recovery_lifeline_command_execution_enablement_missing",
        false,
    );
    raw_line("      ],");
    emit_recovery_lifeline_command_dispatch_behavior_retained_reference(
        &check,
        recorded_event_id,
        retained_command_dispatch_behavior,
    );
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"command_dispatch_behavior_reference_present\": ");
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
    raw_line("      }");
    end_response("recovery.lifeline_command_dispatch_behavior_diagnostic");
}

pub(crate) fn emit_recovery_lifeline_command_dispatch_behavior_diagnostic_selftest() {
    let cases = recovery_lifeline_command_dispatch_behavior_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.lifeline_command_dispatch_behavior_diagnostic_selftest");
    raw_line(
        "      \"schema\": \"raios.recovery_lifeline_command_dispatch_behavior_selftest.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line(
        "      \"creates_retained_recovery_lifeline_command_dispatch_behavior_records\": false,",
    );
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_lifeline_command_dispatch_behavior_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.lifeline_command_dispatch_behavior_diagnostic_selftest");
}

pub(crate) fn emit_recovery_lifeline_command_executor_capability_table_diagnostic(method: &str) {
    let check = parse_recovery_lifeline_command_executor_capability_table_reference(
        recovery_lifeline_command_executor_capability_table_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_lifeline_command_executor_capability_table_from_check(&check)
            .map(event_log::record_recovery_lifeline_command_executor_capability_table_reference)
    } else {
        None
    };
    let retained_executor_capability_table =
        event_log::latest_recovery_lifeline_command_executor_capability_table_reference();

    begin_response("recovery.lifeline_command_executor_capability_table_diagnostic");
    raw_line(
        "      \"schema\": \"raios.recovery_lifeline_command_executor_capability_table_diagnostic.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw("      \"creates_retained_recovery_lifeline_command_executor_capability_table_records\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"executes_disable_module\": false,");
    raw_line("      \"executes_restart_last_good\": false,");
    raw_line("      \"executes_load_recovery_artifact_by_hash\": false,");
    raw_line("      \"disables_module\": false,");
    raw_line("      \"restarts_last_good\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"authorizes_recovery_load\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.lifeline_command_executor_capability_table_diagnostic <executor_capability_table_hash> <retained_command_dispatch_behavior_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <restart_last_good_target_binding_hash> <load_artifact_by_hash_target_binding_hash> <recovery_memory_write_authority_hash> <durable_audit_rollback_write_authority_hash> <service_inventory_side_effect_boundary_hash> <command_dispatch_behavior_hash> <command_dispatch_boundary_id> <executor_capability_table_id> <executor_capability_projection_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline_command_executor_capability_table\",");
    raw_line("        \"executor_capability_table_schema\": \"raios.recovery_lifeline_command_executor_capability_table.v0\",");
    raw_line("        \"executor_capability_table_canonicalization\": \"raios.recovery_lifeline_command_executor_capability_table.canonical.v0\",");
    raw_line(
        "        \"executor_capability_table_id\": \"boundary.recovery_lifeline_command_executor_capability_table.current_boot\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_command_executor_capability_table_reference_object(&check);
    raw_line(",");
    raw_line("      \"executor_capability_table_requirements\": [");
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "side_effect_gate",
        "raios.recovery_lifeline_command_side_effect_gate.v0",
        "recovery_lifeline_command_side_effect_gate_missing",
        true,
    );
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "command_execution_enablement",
        "raios.recovery_lifeline_command_execution_enablement.v0",
        "recovery_lifeline_command_execution_enablement_missing",
        false,
    );
    raw_line("      ],");
    emit_recovery_lifeline_command_executor_capability_table_retained_reference(
        &check,
        recorded_event_id,
        retained_executor_capability_table,
    );
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"executor_capability_table_reference_present\": ");
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
    raw_line("      }");
    end_response("recovery.lifeline_command_executor_capability_table_diagnostic");
}

pub(crate) fn emit_recovery_lifeline_command_executor_capability_table_diagnostic_selftest() {
    let cases = recovery_lifeline_command_executor_capability_table_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.lifeline_command_executor_capability_table_diagnostic_selftest");
    raw_line(
        "      \"schema\": \"raios.recovery_lifeline_command_executor_capability_table_selftest.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line(
        "      \"creates_retained_recovery_lifeline_command_executor_capability_table_records\": false,",
    );
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_lifeline_command_executor_capability_table_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.lifeline_command_executor_capability_table_diagnostic_selftest");
}

pub(crate) fn emit_recovery_lifeline_command_side_effect_gate_diagnostic(method: &str) {
    let check = parse_recovery_lifeline_command_side_effect_gate_reference(
        recovery_lifeline_command_side_effect_gate_diagnostic_arg(method),
        true,
    );
    let recorded_event_id = if check.valid {
        recovery_lifeline_command_side_effect_gate_from_check(&check)
            .map(event_log::record_recovery_lifeline_command_side_effect_gate_reference)
    } else {
        None
    };
    let retained_side_effect_gate =
        event_log::latest_recovery_lifeline_command_side_effect_gate_reference();

    begin_response("recovery.lifeline_command_side_effect_gate_diagnostic");
    raw_line(
        "      \"schema\": \"raios.recovery_lifeline_command_side_effect_gate_diagnostic.v0\",",
    );
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw("      \"status\": ");
    json_str(check.status);
    raw_line(",");
    raw("      \"reason\": ");
    json_str(check.reason);
    raw_line(",");
    raw_line("      \"test_infrastructure\": false,");
    raw("      \"mutates_global_event_log\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw("      \"global_event_log_mutation\": ");
    json_str(if check.valid {
        "valid_hash_reference_retention_only"
    } else {
        "none"
    });
    raw_line(",");
    raw("      \"creates_retained_recovery_lifeline_command_side_effect_gate_records\": ");
    raw_bool(check.valid);
    raw_line(",");
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"executes_lifeline_status\": false,");
    raw_line("      \"executes_rollback_preview\": false,");
    raw_line("      \"executes_rollback_apply\": false,");
    raw_line("      \"executes_disable_module\": false,");
    raw_line("      \"executes_restart_last_good\": false,");
    raw_line("      \"executes_load_recovery_artifact_by_hash\": false,");
    raw_line("      \"disables_module\": false,");
    raw_line("      \"restarts_last_good\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"authorizes_recovery_load\": false,");
    raw_line("      \"writes_recovery_memory\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"exports_provider_context\": false,");
    raw_line("      \"writes_durable_audit_log\": false,");
    raw_line("      \"writes_rollback_store\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"reference_format\": \"recovery.lifeline_command_side_effect_gate_diagnostic <side_effect_gate_hash> <retained_executor_capability_table_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <restart_last_good_target_binding_hash> <load_artifact_by_hash_target_binding_hash> <recovery_memory_write_authority_hash> <durable_audit_rollback_write_authority_hash> <service_inventory_side_effect_boundary_hash> <command_dispatch_behavior_hash> <executor_capability_table_hash> <command_dispatch_boundary_id> <side_effect_gate_id> <side_effect_projection_hash> [current_boot]\",");
    raw_line("      \"request\": {");
    raw_line("        \"read_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"requested_capability\": \"cap.recovery.command.read\",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"resource\": \"recovery_lifeline_command_side_effect_gate\",");
    raw_line("        \"side_effect_gate_schema\": \"raios.recovery_lifeline_command_side_effect_gate.v0\",");
    raw_line("        \"side_effect_gate_canonicalization\": \"raios.recovery_lifeline_command_side_effect_gate.canonical.v0\",");
    raw_line(
        "        \"side_effect_gate_id\": \"boundary.recovery_lifeline_command_side_effect_gate.current_boot\"",
    );
    raw_line("      },");
    emit_recovery_lifeline_command_side_effect_gate_reference_object(&check);
    raw_line(",");
    raw_line("      \"side_effect_gate_requirements\": [");
    emit_recovery_lifeline_command_body_canonicalization_requirement(
        "command_execution_enablement",
        "raios.recovery_lifeline_command_execution_enablement.v0",
        "recovery_lifeline_command_execution_enablement_missing",
        false,
    );
    raw_line("      ],");
    emit_recovery_lifeline_command_side_effect_gate_retained_reference(
        &check,
        recorded_event_id,
        retained_side_effect_gate,
    );
    raw_line(",");
    raw_line("      \"policy_result\": {");
    raw("        \"side_effect_gate_reference_present\": ");
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
    raw_line("      }");
    end_response("recovery.lifeline_command_side_effect_gate_diagnostic");
}

pub(crate) fn emit_recovery_lifeline_command_side_effect_gate_diagnostic_selftest() {
    let cases = recovery_lifeline_command_side_effect_gate_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response("recovery.lifeline_command_side_effect_gate_diagnostic_selftest");
    raw_line("      \"schema\": \"raios.recovery_lifeline_command_side_effect_gate_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line(
        "      \"creates_retained_recovery_lifeline_command_side_effect_gate_records\": false,",
    );
    raw_line("      \"accepts_raw_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_body\": false,");
    raw_line("      \"accepts_lifeline_command_envelope\": false,");
    raw_line("      \"dispatches_lifeline_command\": false,");
    raw_line("      \"command_execution_enabled\": false,");
    raw_line("      \"allocates_service_slot\": false,");
    raw_line("      \"creates_service_inventory_records\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_lifeline_command_side_effect_gate_selftest_case(
            &cases[idx],
            idx + 1 != cases.len(),
        );
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response("recovery.lifeline_command_side_effect_gate_diagnostic_selftest");
}

pub(crate) fn emit_recovery_artifact_load_binding() {
    let retained_identity = event_log::latest_recovery_artifact_identity_reference();
    let retained_trust = event_log::latest_recovery_artifact_trust_reference();
    let retained_vm_test = event_log::latest_recovery_artifact_vm_test_reference();
    let retained_local_approval = event_log::latest_recovery_artifact_local_approval_reference();
    let retained_loader = event_log::latest_recovery_artifact_loader_reference();
    let retained_rollback_evidence =
        event_log::latest_recovery_artifact_rollback_evidence_reference();
    let live = evaluate_recovery_load_binding(recovery_load_binding_candidate_from_retained(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    ));

    begin_response(RECOVERY_ARTIFACT_LOAD_BINDING_METHOD);
    raw_line("      \"schema\": \"raios.recovery_artifact_load_binding.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"status\": \"denied_missing_recovery_binding\",");
    raw_line("      \"test_infrastructure\": false,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_records\": false,");
    raw_line("      \"request\": {");
    raw("        \"requested_capability\": ");
    json_str(RECOVERY_ARTIFACT_LOAD_CAPABILITY);
    raw_line(",");
    raw("        \"read_capability\": ");
    json_str(RECOVERY_ARTIFACT_LOAD_READ_CAPABILITY);
    raw_line(",");
    raw_line("        \"load_mode\": \"recovery_only\",");
    raw_line("        \"risk\": \"recovery_modify_ram\",");
    raw_line("        \"target\": \"recovery_lifeline\",");
    raw_line("        \"subject\": \"agent.session.serial\",");
    raw_line("        \"recovery_only_capability_used\": true,");
    raw_line("        \"normal_module_capability_used\": false,");
    raw_line("        \"normal_module_load_path_used\": false,");
    raw_line("        \"separate_from\": \"cap.module.load_ephemeral\"");
    raw_line("      },");
    raw_line("      \"required_retained_evidence_ids\": [");
    raw_line("        \"recovery_artifact_identity_event_id\",");
    raw_line("        \"recovery_artifact_trust_event_id\",");
    raw_line("        \"recovery_vm_test_event_id\",");
    raw_line("        \"recovery_local_approval_event_id\",");
    raw_line("        \"recovery_loader_event_id\",");
    raw_line("        \"recovery_rollback_evidence_event_id\"");
    raw_line("      ],");
    raw_line("      \"required_retained_evidence\": {");
    emit_recovery_load_identity_binding_fact(retained_identity, true);
    emit_recovery_load_trust_binding_fact(retained_identity, retained_trust, true);
    emit_recovery_load_vm_test_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        true,
    );
    emit_recovery_load_local_approval_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        true,
    );
    emit_recovery_load_loader_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        true,
    );
    emit_recovery_load_rollback_evidence_binding_fact(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
        false,
    );
    raw_line("      },");
    raw_line("      \"normal_module_authority_rejected\": {");
    raw_line("        \"module_load_ephemeral_facts_used\": false,");
    raw_line("        \"module_append_intent_used\": false,");
    raw_line("        \"module_append_payload_hash_used_as_authority\": false,");
    raw_line("        \"module_writer_facts_used\": false,");
    raw_line("        \"module_service_slot_used\": false,");
    raw_line("        \"normal_module_capability_accepted\": false");
    raw_line("      },");
    raw_line("      \"append_payload_hash_envelopes\": {");
    raw_line("        \"schema\": \"raios.module_audit_rollback_append_payload_hash.v0\",");
    raw_line("        \"authority\": false,");
    raw_line("        \"non_authority_input_only\": true,");
    raw_line("        \"append_payload_hash_authority\": false");
    raw_line("      },");
    raw_line("      \"boundary\": {");
    emit_recovery_load_binding_check(&live, 8, true);
    raw_line("      },");
    raw_line("      \"blocked_by\": [");
    let mut wrote_blocker = false;
    if retained_identity.is_none() {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_artifact_identity_event_id",
            "missing",
            "recovery_artifact_identity_event_id_missing",
        );
    }
    if retained_trust.is_none() {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_artifact_trust_event_id",
            "missing",
            "recovery_artifact_trust_event_id_missing",
        );
    } else if let Some(reason) =
        recovery_load_binding_retained_trust_mismatch(retained_identity, retained_trust)
    {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_artifact_trust_event_id",
            "rejected",
            reason,
        );
    }
    if retained_vm_test.is_none() {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_vm_test_event_id",
            "missing",
            "recovery_vm_test_event_id_missing",
        );
    } else if let Some(reason) = recovery_load_binding_retained_vm_test_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
    ) {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_vm_test_event_id",
            "rejected",
            reason,
        );
    }
    if retained_local_approval.is_none() {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_local_approval_event_id",
            "missing",
            "recovery_local_approval_event_id_missing",
        );
    } else if let Some(reason) = recovery_load_binding_retained_local_approval_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
    ) {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_local_approval_event_id",
            "rejected",
            reason,
        );
    }
    if retained_loader.is_none() {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_loader_event_id",
            "missing",
            "recovery_loader_event_id_missing",
        );
    } else if let Some(reason) = recovery_load_binding_retained_loader_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
    ) {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_loader_event_id",
            "rejected",
            reason,
        );
    }
    if retained_rollback_evidence.is_none() {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_rollback_evidence_event_id",
            "missing",
            "recovery_rollback_evidence_event_id_missing",
        );
    } else if let Some(reason) = recovery_load_binding_retained_rollback_evidence_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    ) {
        emit_recovery_load_blocker(
            &mut wrote_blocker,
            "recovery_rollback_evidence_event_id",
            "rejected",
            reason,
        );
    }
    crlf();
    raw_line("      ]");
    end_response(RECOVERY_ARTIFACT_LOAD_BINDING_METHOD);
}

pub(crate) fn emit_recovery_artifact_load_binding_selftest() {
    let cases = recovery_load_binding_selftest_cases();
    let mut passed = true;
    let mut idx = 0usize;
    while idx < cases.len() {
        passed = passed && cases[idx].passed;
        idx += 1;
    }

    begin_response(RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD);
    raw_line("      \"schema\": \"raios.recovery_artifact_load_binding_selftest.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"test_infrastructure\": true,");
    raw_line("      \"mutates_global_event_log\": false,");
    raw_line("      \"creates_retained_recovery_records\": false,");
    raw_line("      \"creates_durable_records\": false,");
    raw_line("      \"installs_rollback_plan\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false,");
    raw_line("      \"requested_capability\": \"cap.recovery.load_artifact\",");
    raw_line("      \"normal_module_capability_accepted\": false,");
    raw_line("      \"append_payload_hash_authority\": false,");
    raw("      \"case_count\": ");
    raw_fmt(format_args!("{}", cases.len()));
    raw_line(",");
    raw("      \"passed\": ");
    raw_bool(passed);
    raw_line(",");
    raw_line("      \"required_retained_evidence_ids\": [");
    raw_line("        \"recovery_artifact_identity_event_id\",");
    raw_line("        \"recovery_artifact_trust_event_id\",");
    raw_line("        \"recovery_vm_test_event_id\",");
    raw_line("        \"recovery_local_approval_event_id\",");
    raw_line("        \"recovery_loader_event_id\",");
    raw_line("        \"recovery_rollback_evidence_event_id\"");
    raw_line("      ],");
    raw_line("      \"cases\": [");
    idx = 0;
    while idx < cases.len() {
        emit_recovery_load_binding_selftest_case(&cases[idx], idx + 1 != cases.len());
        idx += 1;
    }
    raw_line("      ],");
    raw_line("      \"can_move_beyond_denial\": false");
    end_response(RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD);
}

pub(crate) fn emit_recovery_artifact_load_denied(
    method: &'static str,
    event_id: event_log::EventId,
) {
    serial::write_raw_fmt(format_args!("RAIOS_AGENT_BEGIN {}\r\n", method));
    raw_line("{");
    raw_line("  \"v\": \"raios.agent.v0\",");
    raw_line("  \"t\": \"error\",");
    raw_line("  \"id\": \"serial\",");
    raw_line("  \"body\": {");
    raw("    \"method\": ");
    json_str(method);
    raw_line(",");
    raw("    \"event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw("    \"audit_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw_line("    \"code\": \"capability_denied\",");
    raw_line("    \"schema\": \"raios.recovery_artifact_load_boundary.v0\",");
    raw("    \"message\": ");
    json_str("recovery artifact loading is denied until recovery-only identity, trust, VM-test, approval, loader, and rollback evidence exist");
    raw_line(",");
    raw_line("    \"request\": {");
    raw_line("      \"load_mode\": \"recovery_only\",");
    raw("      \"requested_capability\": ");
    json_str(RECOVERY_ARTIFACT_LOAD_CAPABILITY);
    raw_line(",");
    raw_line("      \"risk\": \"recovery_modify_ram\",");
    raw_line("      \"target\": \"recovery_lifeline\",");
    raw_line("      \"subject\": \"agent.session.serial\",");
    raw_line("      \"normal_module_load_path_used\": false,");
    raw_line("      \"normal_module_capability_used\": false,");
    raw_line("      \"separate_from\": \"cap.module.load_ephemeral\"");
    raw_line("    },");
    raw_line("    \"boundary\": {");
    raw_line("      \"schema\": \"raios.recovery_artifact_load_denial_evidence.v0\",");
    raw_line("      \"scope\": \"current_boot\",");
    raw_line("      \"classification\": \"local_only\",");
    raw_line("      \"status\": \"denied_missing_recovery_artifact_evidence\",");
    raw_line("      \"recovery_artifact_identity\": \"missing\",");
    raw_line("      \"recovery_artifact_trust\": \"missing\",");
    raw_line("      \"recovery_vm_test\": \"missing\",");
    raw_line("      \"recovery_local_approval\": \"missing\",");
    raw_line("      \"recovery_loader\": \"missing\",");
    raw_line("      \"recovery_rollback_evidence\": \"missing\",");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"loads_normal_module\": false,");
    raw_line("      \"service_inventory_change\": \"none\",");
    raw_line("      \"load_attempted\": false");
    raw_line("    },");
    raw_line("    \"missing_facts\": {");
    emit_recovery_artifact_load_missing_fact(
        "recovery_artifact_identity",
        "raios.recovery_artifact_identity.v0",
        "recovery_artifact_identity_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_artifact_trust",
        "raios.recovery_artifact_trust.v0",
        "recovery_artifact_trust_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_vm_test",
        "raios.recovery_artifact_vm_test.v0",
        "recovery_vm_test_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_local_approval",
        "raios.recovery_artifact_local_approval.v0",
        "recovery_local_approval_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_loader",
        "raios.recovery_artifact_loader.v0",
        "recovery_loader_missing",
        true,
    );
    emit_recovery_artifact_load_missing_fact(
        "recovery_rollback_evidence",
        "raios.recovery_artifact_rollback_evidence.v0",
        "recovery_rollback_evidence_missing",
        false,
    );
    raw_line("    },");
    raw_line("    \"blocked_by\": [");
    raw_line("      {\"gate\": \"recovery_artifact_identity\", \"state\": \"missing\", \"reason\": \"recovery_artifact_identity_missing\"},");
    raw_line("      {\"gate\": \"recovery_artifact_trust\", \"state\": \"missing\", \"reason\": \"recovery_artifact_trust_missing\"},");
    raw_line("      {\"gate\": \"recovery_vm_test\", \"state\": \"missing\", \"reason\": \"recovery_vm_test_missing\"},");
    raw_line("      {\"gate\": \"recovery_local_approval\", \"state\": \"missing\", \"reason\": \"recovery_local_approval_missing\"},");
    raw_line("      {\"gate\": \"recovery_loader\", \"state\": \"missing\", \"reason\": \"recovery_loader_missing\"},");
    raw_line("      {\"gate\": \"recovery_rollback_evidence\", \"state\": \"missing\", \"reason\": \"recovery_rollback_evidence_missing\"}");
    raw_line("    ],");
    raw_line("    \"required\": [");
    raw_line("      \"raios.recovery_artifact_identity.v0\",");
    raw_line("      \"raios.recovery_artifact_trust.v0\",");
    raw_line("      \"raios.recovery_artifact_vm_test.v0\",");
    raw_line("      \"raios.recovery_artifact_local_approval.v0\",");
    raw_line("      \"raios.recovery_artifact_loader.v0\",");
    raw_line("      \"raios.recovery_artifact_rollback_evidence.v0\"");
    raw_line("    ],");
    raw_line("    \"evidence\": {");
    raw("      \"denial_event_id\": ");
    json_event_id(event_id);
    raw_line(",");
    raw_line("      \"event_scope\": \"current_boot\",");
    raw_line("      \"recovery_only_capability_id\": \"cap.recovery.load_artifact\",");
    raw_line("      \"normal_module_capability_id\": \"cap.module.load_ephemeral\",");
    raw_line("      \"normal_module_append_intent_used\": false,");
    raw_line("      \"append_payload_hash_authority\": false,");
    raw_line("      \"loads_recovery_artifact\": false,");
    raw_line("      \"load_attempted\": false");
    raw_line("    }");
    raw_line("  }");
    raw_line("}");
    serial::write_raw_fmt(format_args!("RAIOS_AGENT_END {}\r\n", method));
}

pub(crate) fn emit_recovery_artifact_load_denial_event_binding(
    binding: event_log::RecoveryArtifactLoadDenialBinding,
) {
    raw(", \"bindings\": {\"schema\": \"raios.recovery_artifact_load_denial_evidence.v0\", \"scope\": \"current_boot\", \"classification\": \"local_only\", \"status\": \"denied_missing_recovery_artifact_evidence\", \"requested_capability\": \"cap.recovery.load_artifact\", \"load_mode\": \"recovery_only\", \"separate_from\": \"cap.module.load_ephemeral\", \"normal_module_load_path_used\": false, \"normal_module_capability_used\": false, \"recovery_artifact_identity\": ");
    json_missing_state(binding.recovery_artifact_identity_missing);
    raw(", \"recovery_artifact_trust\": ");
    json_missing_state(binding.recovery_artifact_trust_missing);
    raw(", \"recovery_vm_test\": ");
    json_missing_state(binding.recovery_vm_test_missing);
    raw(", \"recovery_local_approval\": ");
    json_missing_state(binding.recovery_local_approval_missing);
    raw(", \"recovery_loader\": ");
    json_missing_state(binding.recovery_loader_missing);
    raw(", \"recovery_rollback_evidence\": ");
    json_missing_state(binding.recovery_rollback_evidence_missing);
    raw(", \"loads_recovery_artifact\": false, \"loads_normal_module\": false, \"service_inventory_change\": \"none\", \"load_attempted\": false, \"missing_fact_schemas\": [\"raios.recovery_artifact_identity.v0\", \"raios.recovery_artifact_trust.v0\", \"raios.recovery_artifact_vm_test.v0\", \"raios.recovery_artifact_local_approval.v0\", \"raios.recovery_artifact_loader.v0\", \"raios.recovery_artifact_rollback_evidence.v0\"]}");
}

fn recovery_lifeline_protocol_candidate_from_retained(
    retained_request: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineRequestReference,
    )>,
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained_vm_test: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    retained_local_approval: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
    retained_loader: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLoaderReference,
    )>,
    retained_rollback_evidence: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactRollbackEvidenceReference,
    )>,
) -> RecoveryLifelineProtocolCandidate {
    let mismatch = recovery_lifeline_protocol_retained_request_mismatch(
        retained_request,
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    );
    RecoveryLifelineProtocolCandidate {
        request_retained: retained_request.is_some(),
        request_current_boot: true,
        request_schema_ok: true,
        request_binding_ok: mismatch.is_none(),
        request_binding_reason: mismatch.unwrap_or("retained_recovery_lifeline_request_valid"),
        direct_openai_recovery_shortcut_used: false,
        lifeline_protocol_state_present: false,
        command_vocabulary_present: false,
        loader_runtime_isolation_present: false,
        rollback_transaction_engine_present: false,
        durable_audit_rollback_persistence_present: false,
        recovery_memory_provenance_present: false,
    }
}

fn recovery_lifeline_protocol_retained_request_mismatch(
    retained_request: Option<(
        event_log::EventId,
        event_log::RecoveryLifelineRequestReference,
    )>,
    retained_identity: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactIdentityReference,
    )>,
    retained_trust: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactTrustReference,
    )>,
    retained_vm_test: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactVmTestReference,
    )>,
    retained_local_approval: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLocalApprovalReference,
    )>,
    retained_loader: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactLoaderReference,
    )>,
    retained_rollback_evidence: Option<(
        event_log::EventId,
        event_log::RecoveryArtifactRollbackEvidenceReference,
    )>,
) -> Option<&'static str> {
    let Some((_request_event_id, request)) = retained_request else {
        return Some("recovery_lifeline_request_event_id_missing");
    };
    let Some((identity_event_id, identity_reference)) = retained_identity else {
        return Some("recovery_artifact_identity_reference_missing");
    };
    let Some((trust_event_id, _trust_reference)) = retained_trust else {
        return Some("recovery_artifact_trust_reference_missing");
    };
    let Some((vm_test_event_id, _vm_test_reference)) = retained_vm_test else {
        return Some("recovery_artifact_vm_test_reference_missing");
    };
    let Some((local_approval_event_id, _approval_reference)) = retained_local_approval else {
        return Some("recovery_artifact_local_approval_reference_missing");
    };
    let Some((loader_event_id, _loader_reference)) = retained_loader else {
        return Some("recovery_artifact_loader_reference_missing");
    };
    let Some((rollback_event_id, rollback_reference)) = retained_rollback_evidence else {
        return Some("recovery_artifact_rollback_evidence_reference_missing");
    };
    if request.retained_identity_reference_event_id != identity_event_id {
        return Some("recovery_lifeline_request_identity_event_id_mismatch");
    }
    if request.retained_trust_reference_event_id != trust_event_id {
        return Some("recovery_lifeline_request_trust_event_id_mismatch");
    }
    if request.retained_vm_test_reference_event_id != vm_test_event_id {
        return Some("recovery_lifeline_request_vm_test_event_id_mismatch");
    }
    if request.retained_local_approval_reference_event_id != local_approval_event_id {
        return Some("recovery_lifeline_request_local_approval_event_id_mismatch");
    }
    if request.retained_loader_reference_event_id != loader_event_id {
        return Some("recovery_lifeline_request_loader_event_id_mismatch");
    }
    if request.retained_rollback_evidence_reference_event_id != rollback_event_id {
        return Some("recovery_lifeline_request_rollback_evidence_event_id_mismatch");
    }
    if let Some(reason) = recovery_load_binding_retained_rollback_evidence_mismatch(
        retained_identity,
        retained_trust,
        retained_vm_test,
        retained_local_approval,
        retained_loader,
        retained_rollback_evidence,
    ) {
        return Some(reason);
    }
    if request.identity_reference_hash != identity_reference.identity_reference_hash {
        return Some("recovery_lifeline_request_identity_reference_hash_mismatch");
    }
    if request.identity_reference_hash != rollback_reference.identity_reference_hash {
        return Some("recovery_lifeline_request_rollback_identity_reference_hash_mismatch");
    }
    if request.trust_reference_hash != rollback_reference.trust_reference_hash {
        return Some("recovery_lifeline_request_trust_reference_hash_mismatch");
    }
    if request.vm_test_reference_hash != rollback_reference.vm_test_reference_hash {
        return Some("recovery_lifeline_request_vm_test_reference_hash_mismatch");
    }
    if request.local_approval_reference_hash != rollback_reference.local_approval_reference_hash {
        return Some("recovery_lifeline_request_local_approval_reference_hash_mismatch");
    }
    if request.loader_reference_hash != rollback_reference.loader_reference_hash {
        return Some("recovery_lifeline_request_loader_reference_hash_mismatch");
    }
    if request.rollback_evidence_reference_hash
        != rollback_reference.rollback_evidence_reference_hash
    {
        return Some("recovery_artifact_rollback_evidence_reference_hash_mismatch");
    }
    if request.artifact_hash != rollback_reference.artifact_hash {
        return Some("recovery_lifeline_request_artifact_hash_mismatch");
    }
    if request.trust_hash != rollback_reference.trust_hash {
        return Some("recovery_lifeline_request_trust_hash_mismatch");
    }
    if request.vm_test_hash != rollback_reference.vm_test_hash {
        return Some("recovery_lifeline_request_vm_test_hash_mismatch");
    }
    if request.local_approval_hash != rollback_reference.local_approval_hash {
        return Some("recovery_lifeline_request_local_approval_hash_mismatch");
    }
    if request.loader_hash != rollback_reference.loader_hash {
        return Some("recovery_lifeline_request_loader_hash_mismatch");
    }
    if request.rollback_evidence_hash != rollback_reference.rollback_evidence_hash {
        return Some("recovery_artifact_rollback_evidence_hash_mismatch");
    }

    let mut identity_event_id_text = [0u8; 27];
    let mut trust_event_id_text = [0u8; 27];
    let mut vm_test_event_id_text = [0u8; 27];
    let mut local_approval_event_id_text = [0u8; 27];
    let mut loader_event_id_text = [0u8; 27];
    let mut rollback_event_id_text = [0u8; 27];
    let expected = module_evidence::computed_recovery_lifeline_request_reference_hash(
        module_evidence::RecoveryLifelineRequestReferenceHashInput {
            retained_identity_reference_event_id: current_boot_event_id_text(
                request.retained_identity_reference_event_id,
                &mut identity_event_id_text,
            ),
            retained_trust_reference_event_id: current_boot_event_id_text(
                request.retained_trust_reference_event_id,
                &mut trust_event_id_text,
            ),
            retained_vm_test_reference_event_id: current_boot_event_id_text(
                request.retained_vm_test_reference_event_id,
                &mut vm_test_event_id_text,
            ),
            retained_local_approval_reference_event_id: current_boot_event_id_text(
                request.retained_local_approval_reference_event_id,
                &mut local_approval_event_id_text,
            ),
            retained_loader_reference_event_id: current_boot_event_id_text(
                request.retained_loader_reference_event_id,
                &mut loader_event_id_text,
            ),
            retained_rollback_evidence_reference_event_id: current_boot_event_id_text(
                request.retained_rollback_evidence_reference_event_id,
                &mut rollback_event_id_text,
            ),
            identity_reference_hash: request.identity_reference_hash,
            trust_reference_hash: request.trust_reference_hash,
            vm_test_reference_hash: request.vm_test_reference_hash,
            local_approval_reference_hash: request.local_approval_reference_hash,
            loader_reference_hash: request.loader_reference_hash,
            rollback_evidence_reference_hash: request.rollback_evidence_reference_hash,
            artifact_hash: request.artifact_hash,
            trust_hash: request.trust_hash,
            vm_test_hash: request.vm_test_hash,
            local_approval_hash: request.local_approval_hash,
            loader_hash: request.loader_hash,
            rollback_evidence_hash: request.rollback_evidence_hash,
        },
    );
    if request.lifeline_request_reference_hash != expected {
        return Some("recovery_lifeline_request_reference_hash_mismatch");
    }
    None
}

fn current_boot_event_id_text<'a>(event_id: event_log::EventId, out: &'a mut [u8; 27]) -> &'a str {
    let prefix = b"event.current_boot.";
    let mut idx = 0usize;
    while idx < prefix.len() {
        out[idx] = prefix[idx];
        idx += 1;
    }
    let mut sequence = event_id.sequence();
    let mut digit = 0usize;
    while digit < 8 {
        out[prefix.len() + 7 - digit] = b'0' + (sequence % 10) as u8;
        sequence /= 10;
        digit += 1;
    }
    unsafe { core::str::from_utf8_unchecked(out) }
}

fn json_missing_state(missing: bool) {
    json_str(if missing { "missing" } else { "available" });
}
