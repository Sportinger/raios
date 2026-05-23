use crate::agent_protocol_support::method_head_eq;

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLifelineCommandExecutionStageDescriptor {
    pub(crate) index: u8,
    pub(crate) method_name: &'static str,
    pub(crate) method_alias: &'static str,
    pub(crate) selftest_method_name: &'static str,
    pub(crate) selftest_alias: &'static str,
    pub(crate) response_method: &'static str,
    pub(crate) selftest_response_method: &'static str,
    pub(crate) diagnostic_schema: &'static str,
    pub(crate) selftest_schema: &'static str,
    pub(crate) reference_schema: &'static str,
    pub(crate) canonicalization: &'static str,
    pub(crate) resource: &'static str,
    pub(crate) stage_name: &'static str,
    pub(crate) stage_hash_field: &'static str,
    pub(crate) stage_id_field: &'static str,
    pub(crate) stage_id: &'static str,
    pub(crate) stage_projection_field: &'static str,
    pub(crate) retained_previous_stage_event_id_field: &'static str,
    pub(crate) reference_format: &'static str,
    pub(crate) absent_reason: &'static str,
    pub(crate) arity_reason: &'static str,
    pub(crate) scope_reason: &'static str,
    pub(crate) invalid_hash_reason: &'static str,
    pub(crate) id_mismatch_reason: &'static str,
    pub(crate) hash_mismatch_status: &'static str,
    pub(crate) hash_mismatch_reason: &'static str,
    pub(crate) retained_previous_missing_reason: &'static str,
    pub(crate) retained_previous_stale_reason: &'static str,
    pub(crate) retained_previous_mismatch_reason: &'static str,
    pub(crate) valid_reason: &'static str,
    pub(crate) not_implemented_reason: &'static str,
    pub(crate) next_requirement_fact: Option<&'static str>,
    pub(crate) next_requirement_schema: Option<&'static str>,
    pub(crate) next_requirement_reason: Option<&'static str>,
}

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLifelineCommandExecutionStageInput<'a> {
    pub(crate) descriptor: RecoveryLifelineCommandExecutionStageDescriptor,
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) execution_stage_hash: Option<[u8; 32]>,
    pub(crate) retained_previous_stage_event_id: Option<&'a str>,
    pub(crate) command_id: Option<&'a str>,
    pub(crate) argument_schema: Option<&'a str>,
    pub(crate) argument_hash: Option<[u8; 32]>,
    pub(crate) target_locator: Option<&'a str>,
    pub(crate) command_envelope_reference_hash: Option<[u8; 32]>,
    pub(crate) command_body_canonicalization_hash: Option<[u8; 32]>,
    pub(crate) handler_binding_hash: Option<[u8; 32]>,
    pub(crate) status_read_handler_hash: Option<[u8; 32]>,
    pub(crate) rollback_preview_authorization_hash: Option<[u8; 32]>,
    pub(crate) rollback_apply_authorization_hash: Option<[u8; 32]>,
    pub(crate) disable_module_target_binding_hash: Option<[u8; 32]>,
    pub(crate) restart_last_good_target_binding_hash: Option<[u8; 32]>,
    pub(crate) load_artifact_by_hash_target_binding_hash: Option<[u8; 32]>,
    pub(crate) recovery_memory_write_authority_hash: Option<[u8; 32]>,
    pub(crate) durable_audit_rollback_write_authority_hash: Option<[u8; 32]>,
    pub(crate) service_inventory_side_effect_boundary_hash: Option<[u8; 32]>,
    pub(crate) command_dispatch_behavior_hash: Option<[u8; 32]>,
    pub(crate) executor_capability_table_hash: Option<[u8; 32]>,
    pub(crate) side_effect_gate_hash: Option<[u8; 32]>,
    pub(crate) execution_enablement_hash: Option<[u8; 32]>,
    pub(crate) execution_preflight_hash: Option<[u8; 32]>,
    pub(crate) execution_intent_hash: Option<[u8; 32]>,
    pub(crate) execution_commit_gate_hash: Option<[u8; 32]>,
    pub(crate) execution_result_denial_hash: Option<[u8; 32]>,
    pub(crate) execution_audit_denial_hash: Option<[u8; 32]>,
    pub(crate) execution_observation_denial_hash: Option<[u8; 32]>,
    pub(crate) command_dispatch_boundary_id: Option<&'a str>,
    pub(crate) execution_stage_id: Option<&'a str>,
    pub(crate) execution_stage_projection_hash: Option<[u8; 32]>,
}

pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_ENABLEMENT_BOUNDARY_ID: &str =
    "boundary.recovery_lifeline_command_execution_enablement.current_boot";
pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_PREFLIGHT_BOUNDARY_ID: &str =
    "boundary.recovery_lifeline_command_execution_preflight.current_boot";
pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_INTENT_BOUNDARY_ID: &str =
    "boundary.recovery_lifeline_command_execution_intent.current_boot";
pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_COMMIT_GATE_BOUNDARY_ID: &str =
    "boundary.recovery_lifeline_command_execution_commit_gate.current_boot";
pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_RESULT_DENIAL_BOUNDARY_ID: &str =
    "boundary.recovery_lifeline_command_execution_result_denial.current_boot";
pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_AUDIT_DENIAL_BOUNDARY_ID: &str =
    "boundary.recovery_lifeline_command_execution_audit_denial.current_boot";
pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_OBSERVATION_DENIAL_BOUNDARY_ID: &str =
    "boundary.recovery_lifeline_command_execution_observation_denial.current_boot";
pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_COMPLETION_DENIAL_BOUNDARY_ID: &str =
    "boundary.recovery_lifeline_command_execution_completion_denial.current_boot";

pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_ENABLEMENT_STAGE:
    RecoveryLifelineCommandExecutionStageDescriptor =
    RecoveryLifelineCommandExecutionStageDescriptor {
        index: 0,
        method_name: "recovery.lifeline_command_execution_enablement_diagnostic",
        method_alias: "recovery.lifeline_command_execution_enablement",
        selftest_method_name: "recovery.lifeline_command_execution_enablement_diagnostic_selftest",
        selftest_alias: "recovery.lifeline_command_execution_enablement_selftest",
        response_method: "recovery.lifeline_command_execution_enablement_diagnostic",
        selftest_response_method: "recovery.lifeline_command_execution_enablement_diagnostic_selftest",
        diagnostic_schema: "raios.recovery_lifeline_command_execution_enablement_diagnostic.v0",
        selftest_schema: "raios.recovery_lifeline_command_execution_enablement_selftest.v0",
        reference_schema: "raios.recovery_lifeline_command_execution_enablement.v0",
        canonicalization: "raios.recovery_lifeline_command_execution_enablement.canonical.v0",
        resource: "recovery_lifeline_command_execution_enablement",
        stage_name: "execution_enablement",
        stage_hash_field: "execution_enablement_hash",
        stage_id_field: "execution_enablement_id",
        stage_id: RECOVERY_LIFELINE_COMMAND_EXECUTION_ENABLEMENT_BOUNDARY_ID,
        stage_projection_field: "execution_projection_sha256",
        retained_previous_stage_event_id_field: "retained_side_effect_gate_event_id",
        reference_format: "recovery.lifeline_command_execution_enablement_diagnostic <execution_enablement_hash> <retained_side_effect_gate_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <restart_last_good_target_binding_hash> <load_artifact_by_hash_target_binding_hash> <recovery_memory_write_authority_hash> <durable_audit_rollback_write_authority_hash> <service_inventory_side_effect_boundary_hash> <command_dispatch_behavior_hash> <executor_capability_table_hash> <side_effect_gate_hash> <command_dispatch_boundary_id> <execution_enablement_id> <execution_projection_hash> [current_boot]",
        absent_reason: "recovery_lifeline_command_execution_enablement_absent",
        arity_reason: "recovery_lifeline_command_execution_enablement_arity_invalid",
        scope_reason: "recovery_lifeline_command_execution_enablement_scope_must_be_current_boot",
        invalid_hash_reason: "recovery_lifeline_command_execution_enablement_invalid_hash",
        id_mismatch_reason: "recovery_lifeline_command_execution_enablement_id_mismatch",
        hash_mismatch_status: "mismatched_recovery_lifeline_command_execution_enablement_hash",
        hash_mismatch_reason: "recovery_lifeline_command_execution_enablement_hash_mismatch",
        retained_previous_missing_reason: "retained_recovery_lifeline_command_side_effect_gate_missing",
        retained_previous_stale_reason: "retained_recovery_lifeline_command_side_effect_gate_event_id_stale_or_dropped",
        retained_previous_mismatch_reason: "recovery_lifeline_command_side_effect_gate_mismatch",
        valid_reason: "recovery_lifeline_command_execution_enablement_valid_but_execution_disabled",
        not_implemented_reason: "recovery_lifeline_command_execution_enablement_not_implemented",
        next_requirement_fact: Some("command_execution_preflight"),
        next_requirement_schema: Some("raios.recovery_lifeline_command_execution_preflight.v0"),
        next_requirement_reason: Some("recovery_lifeline_command_execution_preflight_missing"),
    };

pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_PREFLIGHT_STAGE:
    RecoveryLifelineCommandExecutionStageDescriptor =
    RecoveryLifelineCommandExecutionStageDescriptor {
        index: 1,
        method_name: "recovery.lifeline_command_execution_preflight_diagnostic",
        method_alias: "recovery.lifeline_command_execution_preflight",
        selftest_method_name: "recovery.lifeline_command_execution_preflight_diagnostic_selftest",
        selftest_alias: "recovery.lifeline_command_execution_preflight_selftest",
        response_method: "recovery.lifeline_command_execution_preflight_diagnostic",
        selftest_response_method: "recovery.lifeline_command_execution_preflight_diagnostic_selftest",
        diagnostic_schema: "raios.recovery_lifeline_command_execution_preflight_diagnostic.v0",
        selftest_schema: "raios.recovery_lifeline_command_execution_preflight_selftest.v0",
        reference_schema: "raios.recovery_lifeline_command_execution_preflight.v0",
        canonicalization: "raios.recovery_lifeline_command_execution_preflight.canonical.v0",
        resource: "recovery_lifeline_command_execution_preflight",
        stage_name: "execution_preflight",
        stage_hash_field: "execution_preflight_hash",
        stage_id_field: "execution_preflight_id",
        stage_id: RECOVERY_LIFELINE_COMMAND_EXECUTION_PREFLIGHT_BOUNDARY_ID,
        stage_projection_field: "execution_preflight_projection_sha256",
        retained_previous_stage_event_id_field: "retained_execution_enablement_event_id",
        reference_format: "recovery.lifeline_command_execution_preflight_diagnostic <execution_preflight_hash> <retained_execution_enablement_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <restart_last_good_target_binding_hash> <load_artifact_by_hash_target_binding_hash> <recovery_memory_write_authority_hash> <durable_audit_rollback_write_authority_hash> <service_inventory_side_effect_boundary_hash> <command_dispatch_behavior_hash> <executor_capability_table_hash> <side_effect_gate_hash> <execution_enablement_hash> <command_dispatch_boundary_id> <execution_preflight_id> <execution_preflight_projection_hash> [current_boot]",
        absent_reason: "recovery_lifeline_command_execution_preflight_absent",
        arity_reason: "recovery_lifeline_command_execution_preflight_arity_invalid",
        scope_reason: "recovery_lifeline_command_execution_preflight_scope_must_be_current_boot",
        invalid_hash_reason: "recovery_lifeline_command_execution_preflight_invalid_hash",
        id_mismatch_reason: "recovery_lifeline_command_execution_preflight_id_mismatch",
        hash_mismatch_status: "mismatched_recovery_lifeline_command_execution_preflight_hash",
        hash_mismatch_reason: "recovery_lifeline_command_execution_preflight_hash_mismatch",
        retained_previous_missing_reason: "retained_recovery_lifeline_command_execution_enablement_missing",
        retained_previous_stale_reason: "retained_recovery_lifeline_command_execution_enablement_event_id_stale_or_dropped",
        retained_previous_mismatch_reason: "recovery_lifeline_command_execution_enablement_mismatch",
        valid_reason: "recovery_lifeline_command_execution_preflight_valid_but_execution_disabled",
        not_implemented_reason: "recovery_lifeline_command_execution_preflight_not_implemented",
        next_requirement_fact: Some("command_execution_intent"),
        next_requirement_schema: Some("raios.recovery_lifeline_command_execution_intent.v0"),
        next_requirement_reason: Some("recovery_lifeline_command_execution_intent_missing"),
    };

pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_INTENT_STAGE:
    RecoveryLifelineCommandExecutionStageDescriptor =
    RecoveryLifelineCommandExecutionStageDescriptor {
        index: 2,
        method_name: "recovery.lifeline_command_execution_intent_diagnostic",
        method_alias: "recovery.lifeline_command_execution_intent",
        selftest_method_name: "recovery.lifeline_command_execution_intent_diagnostic_selftest",
        selftest_alias: "recovery.lifeline_command_execution_intent_selftest",
        response_method: "recovery.lifeline_command_execution_intent_diagnostic",
        selftest_response_method: "recovery.lifeline_command_execution_intent_diagnostic_selftest",
        diagnostic_schema: "raios.recovery_lifeline_command_execution_intent_diagnostic.v0",
        selftest_schema: "raios.recovery_lifeline_command_execution_intent_selftest.v0",
        reference_schema: "raios.recovery_lifeline_command_execution_intent.v0",
        canonicalization: "raios.recovery_lifeline_command_execution_intent.canonical.v0",
        resource: "recovery_lifeline_command_execution_intent",
        stage_name: "execution_intent",
        stage_hash_field: "execution_intent_hash",
        stage_id_field: "execution_intent_id",
        stage_id: RECOVERY_LIFELINE_COMMAND_EXECUTION_INTENT_BOUNDARY_ID,
        stage_projection_field: "execution_intent_projection_sha256",
        retained_previous_stage_event_id_field: "retained_execution_preflight_event_id",
        reference_format: "recovery.lifeline_command_execution_intent_diagnostic <execution_intent_hash> <retained_execution_preflight_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <restart_last_good_target_binding_hash> <load_artifact_by_hash_target_binding_hash> <recovery_memory_write_authority_hash> <durable_audit_rollback_write_authority_hash> <service_inventory_side_effect_boundary_hash> <command_dispatch_behavior_hash> <executor_capability_table_hash> <side_effect_gate_hash> <execution_enablement_hash> <execution_preflight_hash> <command_dispatch_boundary_id> <execution_intent_id> <execution_intent_projection_hash> [current_boot]",
        absent_reason: "recovery_lifeline_command_execution_intent_absent",
        arity_reason: "recovery_lifeline_command_execution_intent_arity_invalid",
        scope_reason: "recovery_lifeline_command_execution_intent_scope_must_be_current_boot",
        invalid_hash_reason: "recovery_lifeline_command_execution_intent_invalid_hash",
        id_mismatch_reason: "recovery_lifeline_command_execution_intent_id_mismatch",
        hash_mismatch_status: "mismatched_recovery_lifeline_command_execution_intent_hash",
        hash_mismatch_reason: "recovery_lifeline_command_execution_intent_hash_mismatch",
        retained_previous_missing_reason: "retained_recovery_lifeline_command_execution_preflight_missing",
        retained_previous_stale_reason: "retained_recovery_lifeline_command_execution_preflight_event_id_stale_or_dropped",
        retained_previous_mismatch_reason: "recovery_lifeline_command_execution_preflight_mismatch",
        valid_reason: "recovery_lifeline_command_execution_intent_valid_but_execution_disabled",
        not_implemented_reason: "recovery_lifeline_command_execution_intent_not_implemented",
        next_requirement_fact: Some("command_execution_commit_gate"),
        next_requirement_schema: Some("raios.recovery_lifeline_command_execution_commit_gate.v0"),
        next_requirement_reason: Some("recovery_lifeline_command_execution_commit_gate_missing"),
    };

pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_COMMIT_GATE_STAGE:
    RecoveryLifelineCommandExecutionStageDescriptor =
    RecoveryLifelineCommandExecutionStageDescriptor {
        index: 3,
        method_name: "recovery.lifeline_command_execution_commit_gate_diagnostic",
        method_alias: "recovery.lifeline_command_execution_commit_gate",
        selftest_method_name: "recovery.lifeline_command_execution_commit_gate_diagnostic_selftest",
        selftest_alias: "recovery.lifeline_command_execution_commit_gate_selftest",
        response_method: "recovery.lifeline_command_execution_commit_gate_diagnostic",
        selftest_response_method: "recovery.lifeline_command_execution_commit_gate_diagnostic_selftest",
        diagnostic_schema: "raios.recovery_lifeline_command_execution_commit_gate_diagnostic.v0",
        selftest_schema: "raios.recovery_lifeline_command_execution_commit_gate_selftest.v0",
        reference_schema: "raios.recovery_lifeline_command_execution_commit_gate.v0",
        canonicalization: "raios.recovery_lifeline_command_execution_commit_gate.canonical.v0",
        resource: "recovery_lifeline_command_execution_commit_gate",
        stage_name: "execution_commit_gate",
        stage_hash_field: "execution_commit_gate_hash",
        stage_id_field: "execution_commit_gate_id",
        stage_id: RECOVERY_LIFELINE_COMMAND_EXECUTION_COMMIT_GATE_BOUNDARY_ID,
        stage_projection_field: "execution_commit_gate_projection_sha256",
        retained_previous_stage_event_id_field: "retained_execution_intent_event_id",
        reference_format: "recovery.lifeline_command_execution_commit_gate_diagnostic <execution_commit_gate_hash> <retained_execution_intent_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <restart_last_good_target_binding_hash> <load_artifact_by_hash_target_binding_hash> <recovery_memory_write_authority_hash> <durable_audit_rollback_write_authority_hash> <service_inventory_side_effect_boundary_hash> <command_dispatch_behavior_hash> <executor_capability_table_hash> <side_effect_gate_hash> <execution_enablement_hash> <execution_preflight_hash> <execution_intent_hash> <command_dispatch_boundary_id> <execution_commit_gate_id> <execution_commit_gate_projection_hash> [current_boot]",
        absent_reason: "recovery_lifeline_command_execution_commit_gate_absent",
        arity_reason: "recovery_lifeline_command_execution_commit_gate_arity_invalid",
        scope_reason: "recovery_lifeline_command_execution_commit_gate_scope_must_be_current_boot",
        invalid_hash_reason: "recovery_lifeline_command_execution_commit_gate_invalid_hash",
        id_mismatch_reason: "recovery_lifeline_command_execution_commit_gate_id_mismatch",
        hash_mismatch_status: "mismatched_recovery_lifeline_command_execution_commit_gate_hash",
        hash_mismatch_reason: "recovery_lifeline_command_execution_commit_gate_hash_mismatch",
        retained_previous_missing_reason: "retained_recovery_lifeline_command_execution_intent_missing",
        retained_previous_stale_reason: "retained_recovery_lifeline_command_execution_intent_event_id_stale_or_dropped",
        retained_previous_mismatch_reason: "recovery_lifeline_command_execution_intent_mismatch",
        valid_reason: "recovery_lifeline_command_execution_commit_gate_valid_but_execution_disabled",
        not_implemented_reason: "recovery_lifeline_command_execution_commit_gate_not_implemented",
        next_requirement_fact: Some("command_execution_result_denial"),
        next_requirement_schema: Some("raios.recovery_lifeline_command_execution_result_denial.v0"),
        next_requirement_reason: Some("recovery_lifeline_command_execution_result_denial_missing"),
    };

pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_RESULT_DENIAL_STAGE:
    RecoveryLifelineCommandExecutionStageDescriptor =
    RecoveryLifelineCommandExecutionStageDescriptor {
        index: 4,
        method_name: "recovery.lifeline_command_execution_result_denial_diagnostic",
        method_alias: "recovery.lifeline_command_execution_result_denial",
        selftest_method_name:
            "recovery.lifeline_command_execution_result_denial_diagnostic_selftest",
        selftest_alias: "recovery.lifeline_command_execution_result_denial_selftest",
        response_method: "recovery.lifeline_command_execution_result_denial_diagnostic",
        selftest_response_method:
            "recovery.lifeline_command_execution_result_denial_diagnostic_selftest",
        diagnostic_schema:
            "raios.recovery_lifeline_command_execution_result_denial_diagnostic.v0",
        selftest_schema: "raios.recovery_lifeline_command_execution_result_denial_selftest.v0",
        reference_schema: "raios.recovery_lifeline_command_execution_result_denial.v0",
        canonicalization: "raios.recovery_lifeline_command_execution_result_denial.canonical.v0",
        resource: "recovery_lifeline_command_execution_result_denial",
        stage_name: "execution_result_denial",
        stage_hash_field: "execution_result_denial_hash",
        stage_id_field: "execution_result_denial_id",
        stage_id: RECOVERY_LIFELINE_COMMAND_EXECUTION_RESULT_DENIAL_BOUNDARY_ID,
        stage_projection_field: "execution_result_projection_sha256",
        retained_previous_stage_event_id_field: "retained_execution_commit_gate_event_id",
        reference_format: "recovery.lifeline_command_execution_result_denial_diagnostic <execution_result_denial_hash> <retained_execution_commit_gate_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <restart_last_good_target_binding_hash> <load_artifact_by_hash_target_binding_hash> <recovery_memory_write_authority_hash> <durable_audit_rollback_write_authority_hash> <service_inventory_side_effect_boundary_hash> <command_dispatch_behavior_hash> <executor_capability_table_hash> <side_effect_gate_hash> <execution_enablement_hash> <execution_preflight_hash> <execution_intent_hash> <execution_commit_gate_hash> <command_dispatch_boundary_id> <execution_result_denial_id> <execution_result_projection_hash> [current_boot]",
        absent_reason: "recovery_lifeline_command_execution_result_denial_absent",
        arity_reason: "recovery_lifeline_command_execution_result_denial_arity_invalid",
        scope_reason: "recovery_lifeline_command_execution_result_denial_scope_must_be_current_boot",
        invalid_hash_reason: "recovery_lifeline_command_execution_result_denial_invalid_hash",
        id_mismatch_reason: "recovery_lifeline_command_execution_result_denial_id_mismatch",
        hash_mismatch_status:
            "mismatched_recovery_lifeline_command_execution_result_denial_hash",
        hash_mismatch_reason: "recovery_lifeline_command_execution_result_denial_hash_mismatch",
        retained_previous_missing_reason:
            "retained_recovery_lifeline_command_execution_commit_gate_missing",
        retained_previous_stale_reason:
            "retained_recovery_lifeline_command_execution_commit_gate_event_id_stale_or_dropped",
        retained_previous_mismatch_reason: "recovery_lifeline_command_execution_commit_gate_mismatch",
        valid_reason: "recovery_lifeline_command_execution_result_denial_valid_but_execution_disabled",
        not_implemented_reason: "recovery_lifeline_command_execution_result_denial_not_implemented",
        next_requirement_fact: Some("command_execution_audit_denial"),
        next_requirement_schema: Some("raios.recovery_lifeline_command_execution_audit_denial.v0"),
        next_requirement_reason: Some("recovery_lifeline_command_execution_audit_denial_missing"),
    };

pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_AUDIT_DENIAL_STAGE:
    RecoveryLifelineCommandExecutionStageDescriptor =
    RecoveryLifelineCommandExecutionStageDescriptor {
        index: 5,
        method_name: "recovery.lifeline_command_execution_audit_denial_diagnostic",
        method_alias: "recovery.lifeline_command_execution_audit_denial",
        selftest_method_name:
            "recovery.lifeline_command_execution_audit_denial_diagnostic_selftest",
        selftest_alias: "recovery.lifeline_command_execution_audit_denial_selftest",
        response_method: "recovery.lifeline_command_execution_audit_denial_diagnostic",
        selftest_response_method:
            "recovery.lifeline_command_execution_audit_denial_diagnostic_selftest",
        diagnostic_schema:
            "raios.recovery_lifeline_command_execution_audit_denial_diagnostic.v0",
        selftest_schema: "raios.recovery_lifeline_command_execution_audit_denial_selftest.v0",
        reference_schema: "raios.recovery_lifeline_command_execution_audit_denial.v0",
        canonicalization: "raios.recovery_lifeline_command_execution_audit_denial.canonical.v0",
        resource: "recovery_lifeline_command_execution_audit_denial",
        stage_name: "execution_audit_denial",
        stage_hash_field: "execution_audit_denial_hash",
        stage_id_field: "execution_audit_denial_id",
        stage_id: RECOVERY_LIFELINE_COMMAND_EXECUTION_AUDIT_DENIAL_BOUNDARY_ID,
        stage_projection_field: "execution_audit_projection_sha256",
        retained_previous_stage_event_id_field: "retained_execution_result_denial_event_id",
        reference_format: "recovery.lifeline_command_execution_audit_denial_diagnostic <execution_audit_denial_hash> <retained_execution_result_denial_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <restart_last_good_target_binding_hash> <load_artifact_by_hash_target_binding_hash> <recovery_memory_write_authority_hash> <durable_audit_rollback_write_authority_hash> <service_inventory_side_effect_boundary_hash> <command_dispatch_behavior_hash> <executor_capability_table_hash> <side_effect_gate_hash> <execution_enablement_hash> <execution_preflight_hash> <execution_intent_hash> <execution_commit_gate_hash> <execution_result_denial_hash> <command_dispatch_boundary_id> <execution_audit_denial_id> <execution_audit_projection_hash> [current_boot]",
        absent_reason: "recovery_lifeline_command_execution_audit_denial_absent",
        arity_reason: "recovery_lifeline_command_execution_audit_denial_arity_invalid",
        scope_reason: "recovery_lifeline_command_execution_audit_denial_scope_must_be_current_boot",
        invalid_hash_reason: "recovery_lifeline_command_execution_audit_denial_invalid_hash",
        id_mismatch_reason: "recovery_lifeline_command_execution_audit_denial_id_mismatch",
        hash_mismatch_status:
            "mismatched_recovery_lifeline_command_execution_audit_denial_hash",
        hash_mismatch_reason: "recovery_lifeline_command_execution_audit_denial_hash_mismatch",
        retained_previous_missing_reason:
            "retained_recovery_lifeline_command_execution_result_denial_missing",
        retained_previous_stale_reason:
            "retained_recovery_lifeline_command_execution_result_denial_event_id_stale_or_dropped",
        retained_previous_mismatch_reason: "recovery_lifeline_command_execution_result_denial_mismatch",
        valid_reason: "recovery_lifeline_command_execution_audit_denial_valid_but_execution_disabled",
        not_implemented_reason: "recovery_lifeline_command_execution_audit_denial_not_implemented",
        next_requirement_fact: Some("command_execution_observation_denial"),
        next_requirement_schema: Some(
            "raios.recovery_lifeline_command_execution_observation_denial.v0",
        ),
        next_requirement_reason: Some(
            "recovery_lifeline_command_execution_observation_denial_missing",
        ),
    };

pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_OBSERVATION_DENIAL_STAGE:
    RecoveryLifelineCommandExecutionStageDescriptor =
    RecoveryLifelineCommandExecutionStageDescriptor {
        index: 6,
        method_name: "recovery.lifeline_command_execution_observation_denial_diagnostic",
        method_alias: "recovery.lifeline_command_execution_observation_denial",
        selftest_method_name:
            "recovery.lifeline_command_execution_observation_denial_diagnostic_selftest",
        selftest_alias: "recovery.lifeline_command_execution_observation_denial_selftest",
        response_method: "recovery.lifeline_command_execution_observation_denial_diagnostic",
        selftest_response_method:
            "recovery.lifeline_command_execution_observation_denial_diagnostic_selftest",
        diagnostic_schema:
            "raios.recovery_lifeline_command_execution_observation_denial_diagnostic.v0",
        selftest_schema:
            "raios.recovery_lifeline_command_execution_observation_denial_selftest.v0",
        reference_schema: "raios.recovery_lifeline_command_execution_observation_denial.v0",
        canonicalization:
            "raios.recovery_lifeline_command_execution_observation_denial.canonical.v0",
        resource: "recovery_lifeline_command_execution_observation_denial",
        stage_name: "execution_observation_denial",
        stage_hash_field: "execution_observation_denial_hash",
        stage_id_field: "execution_observation_denial_id",
        stage_id: RECOVERY_LIFELINE_COMMAND_EXECUTION_OBSERVATION_DENIAL_BOUNDARY_ID,
        stage_projection_field: "execution_observation_projection_sha256",
        retained_previous_stage_event_id_field: "retained_execution_audit_denial_event_id",
        reference_format: "recovery.lifeline_command_execution_observation_denial_diagnostic <execution_observation_denial_hash> <retained_execution_audit_denial_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <restart_last_good_target_binding_hash> <load_artifact_by_hash_target_binding_hash> <recovery_memory_write_authority_hash> <durable_audit_rollback_write_authority_hash> <service_inventory_side_effect_boundary_hash> <command_dispatch_behavior_hash> <executor_capability_table_hash> <side_effect_gate_hash> <execution_enablement_hash> <execution_preflight_hash> <execution_intent_hash> <execution_commit_gate_hash> <execution_result_denial_hash> <execution_audit_denial_hash> <command_dispatch_boundary_id> <execution_observation_denial_id> <execution_observation_projection_hash> [current_boot]",
        absent_reason: "recovery_lifeline_command_execution_observation_denial_absent",
        arity_reason: "recovery_lifeline_command_execution_observation_denial_arity_invalid",
        scope_reason:
            "recovery_lifeline_command_execution_observation_denial_scope_must_be_current_boot",
        invalid_hash_reason:
            "recovery_lifeline_command_execution_observation_denial_invalid_hash",
        id_mismatch_reason:
            "recovery_lifeline_command_execution_observation_denial_id_mismatch",
        hash_mismatch_status:
            "mismatched_recovery_lifeline_command_execution_observation_denial_hash",
        hash_mismatch_reason:
            "recovery_lifeline_command_execution_observation_denial_hash_mismatch",
        retained_previous_missing_reason:
            "retained_recovery_lifeline_command_execution_audit_denial_missing",
        retained_previous_stale_reason:
            "retained_recovery_lifeline_command_execution_audit_denial_event_id_stale_or_dropped",
        retained_previous_mismatch_reason:
            "recovery_lifeline_command_execution_audit_denial_mismatch",
        valid_reason:
            "recovery_lifeline_command_execution_observation_denial_valid_but_execution_disabled",
        not_implemented_reason:
            "recovery_lifeline_command_execution_observation_denial_not_implemented",
        next_requirement_fact: Some("command_execution_completion_denial"),
        next_requirement_schema: Some(
            "raios.recovery_lifeline_command_execution_completion_denial.v0",
        ),
        next_requirement_reason: Some(
            "recovery_lifeline_command_execution_completion_denial_missing",
        ),
    };

pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTION_COMPLETION_DENIAL_STAGE:
    RecoveryLifelineCommandExecutionStageDescriptor =
    RecoveryLifelineCommandExecutionStageDescriptor {
        index: 7,
        method_name: "recovery.lifeline_command_execution_completion_denial_diagnostic",
        method_alias: "recovery.lifeline_command_execution_completion_denial",
        selftest_method_name:
            "recovery.lifeline_command_execution_completion_denial_diagnostic_selftest",
        selftest_alias: "recovery.lifeline_command_execution_completion_denial_selftest",
        response_method: "recovery.lifeline_command_execution_completion_denial_diagnostic",
        selftest_response_method:
            "recovery.lifeline_command_execution_completion_denial_diagnostic_selftest",
        diagnostic_schema:
            "raios.recovery_lifeline_command_execution_completion_denial_diagnostic.v0",
        selftest_schema:
            "raios.recovery_lifeline_command_execution_completion_denial_selftest.v0",
        reference_schema: "raios.recovery_lifeline_command_execution_completion_denial.v0",
        canonicalization:
            "raios.recovery_lifeline_command_execution_completion_denial.canonical.v0",
        resource: "recovery_lifeline_command_execution_completion_denial",
        stage_name: "execution_completion_denial",
        stage_hash_field: "execution_completion_denial_hash",
        stage_id_field: "execution_completion_denial_id",
        stage_id: RECOVERY_LIFELINE_COMMAND_EXECUTION_COMPLETION_DENIAL_BOUNDARY_ID,
        stage_projection_field: "execution_completion_projection_sha256",
        retained_previous_stage_event_id_field: "retained_execution_observation_denial_event_id",
        reference_format: "recovery.lifeline_command_execution_completion_denial_diagnostic <execution_completion_denial_hash> <retained_execution_observation_denial_event_id> <command_id> <argument_schema> <argument_hash> <target_locator> <command_envelope_reference_hash> <command_body_canonicalization_hash> <handler_binding_hash> <status_read_handler_hash> <rollback_preview_authorization_hash> <rollback_apply_authorization_hash> <disable_module_target_binding_hash> <restart_last_good_target_binding_hash> <load_artifact_by_hash_target_binding_hash> <recovery_memory_write_authority_hash> <durable_audit_rollback_write_authority_hash> <service_inventory_side_effect_boundary_hash> <command_dispatch_behavior_hash> <executor_capability_table_hash> <side_effect_gate_hash> <execution_enablement_hash> <execution_preflight_hash> <execution_intent_hash> <execution_commit_gate_hash> <execution_result_denial_hash> <execution_audit_denial_hash> <execution_observation_denial_hash> <command_dispatch_boundary_id> <execution_completion_denial_id> <execution_completion_projection_hash> [current_boot]",
        absent_reason: "recovery_lifeline_command_execution_completion_denial_absent",
        arity_reason: "recovery_lifeline_command_execution_completion_denial_arity_invalid",
        scope_reason:
            "recovery_lifeline_command_execution_completion_denial_scope_must_be_current_boot",
        invalid_hash_reason:
            "recovery_lifeline_command_execution_completion_denial_invalid_hash",
        id_mismatch_reason:
            "recovery_lifeline_command_execution_completion_denial_id_mismatch",
        hash_mismatch_status:
            "mismatched_recovery_lifeline_command_execution_completion_denial_hash",
        hash_mismatch_reason:
            "recovery_lifeline_command_execution_completion_denial_hash_mismatch",
        retained_previous_missing_reason:
            "retained_recovery_lifeline_command_execution_observation_denial_missing",
        retained_previous_stale_reason:
            "retained_recovery_lifeline_command_execution_observation_denial_event_id_stale_or_dropped",
        retained_previous_mismatch_reason:
            "recovery_lifeline_command_execution_observation_denial_mismatch",
        valid_reason:
            "recovery_lifeline_command_execution_completion_denial_valid_but_execution_disabled",
        not_implemented_reason:
            "recovery_lifeline_command_execution_completion_denial_not_implemented",
        next_requirement_fact: None,
        next_requirement_schema: None,
        next_requirement_reason: None,
    };

pub(crate) fn recovery_lifeline_command_execution_stage_diagnostic_method(
    method: &str,
    descriptor: RecoveryLifelineCommandExecutionStageDescriptor,
) -> bool {
    method_head_eq(method, descriptor.method_name)
        || method_head_eq(method, descriptor.method_alias)
}

pub(crate) fn recovery_lifeline_command_execution_stage_diagnostic_selftest_method(
    method: &str,
    descriptor: RecoveryLifelineCommandExecutionStageDescriptor,
) -> bool {
    method_head_eq(method, descriptor.selftest_method_name)
        || method_head_eq(method, descriptor.selftest_alias)
}

pub(crate) fn recovery_lifeline_command_execution_stage_diagnostic_arg(
    method: &str,
    descriptor: RecoveryLifelineCommandExecutionStageDescriptor,
) -> &str {
    let method = method.trim();
    let head_len = if method_head_eq(method, descriptor.method_name) {
        descriptor.method_name.len()
    } else if method_head_eq(method, descriptor.method_alias) {
        descriptor.method_alias.len()
    } else {
        return "";
    };
    method[head_len..].trim()
}
