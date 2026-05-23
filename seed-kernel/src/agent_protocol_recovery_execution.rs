use crate::{
    agent_protocol_recovery::{
        recovery_lifeline_command_spec, RecoveryLifelineCommandSpec,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    },
    agent_protocol_support::{
        current_boot_event_id_str, method_eq, method_head_eq, parse_current_boot_event_id,
        parse_sha256_ref,
    },
    event_log, module_evidence,
};

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

#[derive(Clone, Copy)]
pub(crate) struct RecoveryLifelineCommandExecutionStageReferenceCheck<'a> {
    pub(crate) descriptor: RecoveryLifelineCommandExecutionStageDescriptor,
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) execution_stage_hash: Option<[u8; 32]>,
    pub(crate) expected_execution_stage_hash: Option<[u8; 32]>,
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
    pub(crate) normalized_spec: Option<RecoveryLifelineCommandSpec>,
    pub(crate) target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
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

pub(crate) fn parse_recovery_lifeline_command_execution_stage_reference(
    arg: &str,
    descriptor: RecoveryLifelineCommandExecutionStageDescriptor,
    require_live_retained: bool,
) -> RecoveryLifelineCommandExecutionStageReferenceCheck<'_> {
    let mut parts = arg.split_whitespace();
    let execution_stage_hash = parts.next();
    let retained_previous_stage_event_id = parts.next();
    let command_id = parts.next();
    let argument_schema = parts.next();
    let argument_hash = parts.next();
    let target_locator = parts.next();
    let command_envelope_reference_hash = parts.next();
    let command_body_canonicalization_hash = parts.next();
    let handler_binding_hash = parts.next();
    let status_read_handler_hash = parts.next();
    let rollback_preview_authorization_hash = parts.next();
    let rollback_apply_authorization_hash = parts.next();
    let disable_module_target_binding_hash = parts.next();
    let restart_last_good_target_binding_hash = parts.next();
    let load_artifact_by_hash_target_binding_hash = parts.next();
    let recovery_memory_write_authority_hash = parts.next();
    let durable_audit_rollback_write_authority_hash = parts.next();
    let service_inventory_side_effect_boundary_hash = parts.next();
    let command_dispatch_behavior_hash = parts.next();
    let executor_capability_table_hash = parts.next();
    let side_effect_gate_hash = parts.next();
    let execution_enablement_hash = if descriptor.index >= 1 {
        parts.next()
    } else {
        None
    };
    let execution_preflight_hash = if descriptor.index >= 2 {
        parts.next()
    } else {
        None
    };
    let execution_intent_hash = if descriptor.index >= 3 {
        parts.next()
    } else {
        None
    };
    let execution_commit_gate_hash = if descriptor.index >= 4 {
        parts.next()
    } else {
        None
    };
    let execution_result_denial_hash = if descriptor.index >= 5 {
        parts.next()
    } else {
        None
    };
    let execution_audit_denial_hash = if descriptor.index >= 6 {
        parts.next()
    } else {
        None
    };
    let execution_observation_denial_hash = if descriptor.index >= 7 {
        parts.next()
    } else {
        None
    };
    let command_dispatch_boundary_id = parts.next();
    let execution_stage_id = parts.next();
    let execution_stage_projection_hash = parts.next();
    let scope = parts.next().unwrap_or("current_boot");
    let extra = parts.next();
    let input = RecoveryLifelineCommandExecutionStageInput {
        descriptor,
        has_reference: execution_stage_hash.is_some(),
        arity_valid: execution_stage_hash.is_some()
            && retained_previous_stage_event_id.is_some()
            && command_id.is_some()
            && argument_schema.is_some()
            && argument_hash.is_some()
            && target_locator.is_some()
            && command_envelope_reference_hash.is_some()
            && command_body_canonicalization_hash.is_some()
            && handler_binding_hash.is_some()
            && status_read_handler_hash.is_some()
            && rollback_preview_authorization_hash.is_some()
            && rollback_apply_authorization_hash.is_some()
            && disable_module_target_binding_hash.is_some()
            && restart_last_good_target_binding_hash.is_some()
            && load_artifact_by_hash_target_binding_hash.is_some()
            && recovery_memory_write_authority_hash.is_some()
            && durable_audit_rollback_write_authority_hash.is_some()
            && service_inventory_side_effect_boundary_hash.is_some()
            && command_dispatch_behavior_hash.is_some()
            && executor_capability_table_hash.is_some()
            && side_effect_gate_hash.is_some()
            && (descriptor.index < 1 || execution_enablement_hash.is_some())
            && (descriptor.index < 2 || execution_preflight_hash.is_some())
            && (descriptor.index < 3 || execution_intent_hash.is_some())
            && (descriptor.index < 4 || execution_commit_gate_hash.is_some())
            && (descriptor.index < 5 || execution_result_denial_hash.is_some())
            && (descriptor.index < 6 || execution_audit_denial_hash.is_some())
            && (descriptor.index < 7 || execution_observation_denial_hash.is_some())
            && command_dispatch_boundary_id.is_some()
            && execution_stage_id.is_some()
            && execution_stage_projection_hash.is_some()
            && extra.is_none(),
        scope,
        execution_stage_hash: execution_stage_hash.and_then(parse_sha256_ref),
        retained_previous_stage_event_id,
        command_id,
        argument_schema,
        argument_hash: argument_hash.and_then(parse_sha256_ref),
        target_locator,
        command_envelope_reference_hash: command_envelope_reference_hash.and_then(parse_sha256_ref),
        command_body_canonicalization_hash: command_body_canonicalization_hash
            .and_then(parse_sha256_ref),
        handler_binding_hash: handler_binding_hash.and_then(parse_sha256_ref),
        status_read_handler_hash: status_read_handler_hash.and_then(parse_sha256_ref),
        rollback_preview_authorization_hash: rollback_preview_authorization_hash
            .and_then(parse_sha256_ref),
        rollback_apply_authorization_hash: rollback_apply_authorization_hash
            .and_then(parse_sha256_ref),
        disable_module_target_binding_hash: disable_module_target_binding_hash
            .and_then(parse_sha256_ref),
        restart_last_good_target_binding_hash: restart_last_good_target_binding_hash
            .and_then(parse_sha256_ref),
        load_artifact_by_hash_target_binding_hash: load_artifact_by_hash_target_binding_hash
            .and_then(parse_sha256_ref),
        recovery_memory_write_authority_hash: recovery_memory_write_authority_hash
            .and_then(parse_sha256_ref),
        durable_audit_rollback_write_authority_hash: durable_audit_rollback_write_authority_hash
            .and_then(parse_sha256_ref),
        service_inventory_side_effect_boundary_hash: service_inventory_side_effect_boundary_hash
            .and_then(parse_sha256_ref),
        command_dispatch_behavior_hash: command_dispatch_behavior_hash.and_then(parse_sha256_ref),
        executor_capability_table_hash: executor_capability_table_hash.and_then(parse_sha256_ref),
        side_effect_gate_hash: side_effect_gate_hash.and_then(parse_sha256_ref),
        execution_enablement_hash: execution_enablement_hash.and_then(parse_sha256_ref),
        execution_preflight_hash: execution_preflight_hash.and_then(parse_sha256_ref),
        execution_intent_hash: execution_intent_hash.and_then(parse_sha256_ref),
        execution_commit_gate_hash: execution_commit_gate_hash.and_then(parse_sha256_ref),
        execution_result_denial_hash: execution_result_denial_hash.and_then(parse_sha256_ref),
        execution_audit_denial_hash: execution_audit_denial_hash.and_then(parse_sha256_ref),
        execution_observation_denial_hash: execution_observation_denial_hash
            .and_then(parse_sha256_ref),
        command_dispatch_boundary_id,
        execution_stage_id,
        execution_stage_projection_hash: execution_stage_projection_hash.and_then(parse_sha256_ref),
    };
    evaluate_recovery_lifeline_command_execution_stage_reference(input, require_live_retained)
}

pub(crate) fn evaluate_recovery_lifeline_command_execution_stage_reference(
    input: RecoveryLifelineCommandExecutionStageInput<'_>,
    require_live_retained: bool,
) -> RecoveryLifelineCommandExecutionStageReferenceCheck<'_> {
    let descriptor = input.descriptor;
    if !input.has_reference {
        return recovery_lifeline_command_execution_stage_reference_check(
            input,
            None,
            None,
            None,
            "missing",
            descriptor.absent_reason,
            false,
        );
    }
    let Some(retained_previous_stage_event_id) = input.retained_previous_stage_event_id else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(command_id) = input.command_id else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(argument_schema) = input.argument_schema else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(argument_hash) = input.argument_hash else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(target_locator) = input.target_locator else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(command_envelope_reference_hash) = input.command_envelope_reference_hash else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(command_body_canonicalization_hash) = input.command_body_canonicalization_hash else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(handler_binding_hash) = input.handler_binding_hash else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(status_read_handler_hash) = input.status_read_handler_hash else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(rollback_preview_authorization_hash) = input.rollback_preview_authorization_hash
    else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(rollback_apply_authorization_hash) = input.rollback_apply_authorization_hash else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(disable_module_target_binding_hash) = input.disable_module_target_binding_hash else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(restart_last_good_target_binding_hash) = input.restart_last_good_target_binding_hash
    else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(load_artifact_by_hash_target_binding_hash) =
        input.load_artifact_by_hash_target_binding_hash
    else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(recovery_memory_write_authority_hash) = input.recovery_memory_write_authority_hash
    else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(durable_audit_rollback_write_authority_hash) =
        input.durable_audit_rollback_write_authority_hash
    else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(service_inventory_side_effect_boundary_hash) =
        input.service_inventory_side_effect_boundary_hash
    else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(command_dispatch_behavior_hash) = input.command_dispatch_behavior_hash else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(executor_capability_table_hash) = input.executor_capability_table_hash else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(side_effect_gate_hash) = input.side_effect_gate_hash else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    if descriptor.index >= 1 && input.execution_enablement_hash.is_none() {
        return recovery_lifeline_command_execution_stage_invalid(input);
    }
    if descriptor.index >= 2 && input.execution_preflight_hash.is_none() {
        return recovery_lifeline_command_execution_stage_invalid(input);
    }
    if descriptor.index >= 3 && input.execution_intent_hash.is_none() {
        return recovery_lifeline_command_execution_stage_invalid(input);
    }
    if descriptor.index >= 4 && input.execution_commit_gate_hash.is_none() {
        return recovery_lifeline_command_execution_stage_invalid(input);
    }
    if descriptor.index >= 5 && input.execution_result_denial_hash.is_none() {
        return recovery_lifeline_command_execution_stage_invalid(input);
    }
    if descriptor.index >= 6 && input.execution_audit_denial_hash.is_none() {
        return recovery_lifeline_command_execution_stage_invalid(input);
    }
    if descriptor.index >= 7 && input.execution_observation_denial_hash.is_none() {
        return recovery_lifeline_command_execution_stage_invalid(input);
    }
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(execution_stage_id) = input.execution_stage_id else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    let Some(execution_stage_projection_hash) = input.execution_stage_projection_hash else {
        return recovery_lifeline_command_execution_stage_invalid(input);
    };
    if !input.arity_valid {
        return recovery_lifeline_command_execution_stage_reference_check(
            input,
            None,
            None,
            None,
            "invalid_reference",
            descriptor.arity_reason,
            false,
        );
    }
    if !method_eq(input.scope, "current_boot") {
        return recovery_lifeline_command_execution_stage_reference_check(
            input,
            None,
            None,
            None,
            "stale_or_non_current_boot_reference",
            descriptor.scope_reason,
            false,
        );
    }
    if !current_boot_event_id_str(retained_previous_stage_event_id) {
        return recovery_lifeline_command_execution_stage_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "retained_recovery_lifeline_command_execution_stage_event_id_not_current_boot",
            false,
        );
    }
    let Some(spec) = recovery_lifeline_command_spec(command_id) else {
        return recovery_lifeline_command_execution_stage_reference_check(
            input,
            None,
            None,
            None,
            "rejected",
            "recovery_lifeline_command_id_unsupported",
            false,
        );
    };
    if !method_eq(argument_schema, spec.argument_schema) {
        return recovery_lifeline_command_execution_stage_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_argument_schema_mismatch",
            false,
        );
    }
    if !method_eq(
        command_dispatch_boundary_id,
        RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
    ) {
        return recovery_lifeline_command_execution_stage_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            "recovery_lifeline_command_dispatch_boundary_mismatch",
            false,
        );
    }
    if !method_eq(execution_stage_id, descriptor.stage_id) {
        return recovery_lifeline_command_execution_stage_reference_check(
            input,
            Some(spec),
            None,
            None,
            "rejected",
            descriptor.id_mismatch_reason,
            false,
        );
    }
    let Some(target_locator_value) = event_log::RecoveryCommandTargetLocator::new(target_locator)
    else {
        return recovery_lifeline_command_execution_stage_reference_check(
            input,
            Some(spec),
            None,
            None,
            "invalid_reference",
            "recovery_lifeline_command_target_locator_invalid",
            false,
        );
    };
    let expected = module_evidence::computed_recovery_lifeline_command_execution_stage_hash(
        module_evidence::RecoveryLifelineCommandExecutionStageHashInput {
            canonicalization: descriptor.canonicalization,
            schema: descriptor.reference_schema,
            resource: descriptor.resource,
            retained_previous_stage_event_id_field: input
                .descriptor
                .retained_previous_stage_event_id_field,
            retained_previous_stage_event_id,
            command_id: spec.command_id,
            argument_schema: spec.argument_schema,
            argument_hash,
            target_locator,
            command_envelope_reference_hash,
            command_body_canonicalization_hash,
            handler_binding_hash,
            status_read_handler_hash,
            rollback_preview_authorization_hash,
            rollback_apply_authorization_hash,
            disable_module_target_binding_hash,
            restart_last_good_target_binding_hash,
            load_artifact_by_hash_target_binding_hash,
            recovery_memory_write_authority_hash,
            durable_audit_rollback_write_authority_hash,
            service_inventory_side_effect_boundary_hash,
            command_dispatch_behavior_hash,
            executor_capability_table_hash,
            side_effect_gate_hash,
            execution_enablement_hash: input.execution_enablement_hash,
            execution_preflight_hash: input.execution_preflight_hash,
            execution_intent_hash: input.execution_intent_hash,
            execution_commit_gate_hash: input.execution_commit_gate_hash,
            execution_result_denial_hash: input.execution_result_denial_hash,
            execution_audit_denial_hash: input.execution_audit_denial_hash,
            execution_observation_denial_hash: input.execution_observation_denial_hash,
            command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
            execution_stage_id_field: descriptor.stage_id_field,
            execution_stage_id: descriptor.stage_id,
            execution_stage_projection_hash_field: descriptor.stage_projection_field,
            execution_stage_projection_hash,
        },
    );
    if input.execution_stage_hash != Some(expected) {
        return recovery_lifeline_command_execution_stage_reference_check(
            input,
            Some(spec),
            Some(target_locator_value),
            Some(expected),
            descriptor.hash_mismatch_status,
            descriptor.hash_mismatch_reason,
            false,
        );
    }
    if require_live_retained {
        if let Some(reason) = recovery_lifeline_command_execution_stage_live_chain_mismatch(&input)
        {
            return recovery_lifeline_command_execution_stage_reference_check(
                input,
                Some(spec),
                Some(target_locator_value),
                Some(expected),
                "rejected",
                reason,
                false,
            );
        }
    }
    recovery_lifeline_command_execution_stage_reference_check(
        input,
        Some(spec),
        Some(target_locator_value),
        Some(expected),
        "valid_hash_reference_command_still_denied",
        descriptor.valid_reason,
        true,
    )
}

fn recovery_lifeline_command_execution_stage_invalid(
    input: RecoveryLifelineCommandExecutionStageInput<'_>,
) -> RecoveryLifelineCommandExecutionStageReferenceCheck<'_> {
    let reason = input.descriptor.invalid_hash_reason;
    recovery_lifeline_command_execution_stage_reference_check(
        input,
        None,
        None,
        None,
        "invalid_reference",
        reason,
        false,
    )
}

fn recovery_lifeline_command_execution_stage_reference_check<'a>(
    input: RecoveryLifelineCommandExecutionStageInput<'a>,
    normalized_spec: Option<RecoveryLifelineCommandSpec>,
    target_locator_value: Option<event_log::RecoveryCommandTargetLocator>,
    expected_execution_stage_hash: Option<[u8; 32]>,
    status: &'static str,
    reason: &'static str,
    valid: bool,
) -> RecoveryLifelineCommandExecutionStageReferenceCheck<'a> {
    RecoveryLifelineCommandExecutionStageReferenceCheck {
        descriptor: input.descriptor,
        has_reference: input.has_reference,
        arity_valid: input.arity_valid,
        scope: input.scope,
        execution_stage_hash: input.execution_stage_hash,
        expected_execution_stage_hash,
        retained_previous_stage_event_id: input.retained_previous_stage_event_id,
        command_id: input.command_id,
        argument_schema: input.argument_schema,
        argument_hash: input.argument_hash,
        target_locator: input.target_locator,
        command_envelope_reference_hash: input.command_envelope_reference_hash,
        command_body_canonicalization_hash: input.command_body_canonicalization_hash,
        handler_binding_hash: input.handler_binding_hash,
        status_read_handler_hash: input.status_read_handler_hash,
        rollback_preview_authorization_hash: input.rollback_preview_authorization_hash,
        rollback_apply_authorization_hash: input.rollback_apply_authorization_hash,
        disable_module_target_binding_hash: input.disable_module_target_binding_hash,
        restart_last_good_target_binding_hash: input.restart_last_good_target_binding_hash,
        load_artifact_by_hash_target_binding_hash: input.load_artifact_by_hash_target_binding_hash,
        recovery_memory_write_authority_hash: input.recovery_memory_write_authority_hash,
        durable_audit_rollback_write_authority_hash: input
            .durable_audit_rollback_write_authority_hash,
        service_inventory_side_effect_boundary_hash: input
            .service_inventory_side_effect_boundary_hash,
        command_dispatch_behavior_hash: input.command_dispatch_behavior_hash,
        executor_capability_table_hash: input.executor_capability_table_hash,
        side_effect_gate_hash: input.side_effect_gate_hash,
        execution_enablement_hash: input.execution_enablement_hash,
        execution_preflight_hash: input.execution_preflight_hash,
        execution_intent_hash: input.execution_intent_hash,
        execution_commit_gate_hash: input.execution_commit_gate_hash,
        execution_result_denial_hash: input.execution_result_denial_hash,
        execution_audit_denial_hash: input.execution_audit_denial_hash,
        execution_observation_denial_hash: input.execution_observation_denial_hash,
        command_dispatch_boundary_id: input.command_dispatch_boundary_id,
        execution_stage_id: input.execution_stage_id,
        execution_stage_projection_hash: input.execution_stage_projection_hash,
        normalized_spec,
        target_locator_value,
        status,
        reason,
        valid,
    }
}

fn recovery_lifeline_command_execution_stage_live_chain_mismatch(
    input: &RecoveryLifelineCommandExecutionStageInput<'_>,
) -> Option<&'static str> {
    let retained_event_id = parse_current_boot_event_id(input.retained_previous_stage_event_id?)?;
    if input.descriptor.index == 0 {
        let Some((latest_event_id, latest_reference)) =
            event_log::latest_recovery_lifeline_command_side_effect_gate_reference()
        else {
            return Some(input.descriptor.retained_previous_missing_reason);
        };
        if latest_event_id != retained_event_id {
            return Some(input.descriptor.retained_previous_stale_reason);
        }
        if !recovery_lifeline_command_execution_stage_matches_side_effect(input, latest_reference) {
            return Some(input.descriptor.retained_previous_mismatch_reason);
        }
        return None;
    }
    let Some(previous_descriptor) =
        recovery_lifeline_command_previous_execution_stage_descriptor(input.descriptor)
    else {
        return Some(input.descriptor.retained_previous_missing_reason);
    };
    let Some((latest_event_id, latest_reference)) =
        event_log::latest_recovery_lifeline_command_execution_stage_reference(
            previous_descriptor.reference_schema,
        )
    else {
        return Some(input.descriptor.retained_previous_missing_reason);
    };
    if latest_event_id != retained_event_id {
        return Some(input.descriptor.retained_previous_stale_reason);
    }
    if !recovery_lifeline_command_execution_stage_matches_previous_stage(input, latest_reference) {
        return Some(input.descriptor.retained_previous_mismatch_reason);
    }
    None
}

fn recovery_lifeline_command_execution_stage_matches_side_effect(
    input: &RecoveryLifelineCommandExecutionStageInput<'_>,
    previous: event_log::RecoveryLifelineCommandSideEffectGateReference,
) -> bool {
    let Some(command_id) = input.command_id else {
        return false;
    };
    let Some(argument_schema) = input.argument_schema else {
        return false;
    };
    let Some(target_locator) = input.target_locator else {
        return false;
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return false;
    };
    method_eq(command_id, previous.command_id)
        && method_eq(argument_schema, previous.argument_schema)
        && input.argument_hash == Some(previous.argument_hash)
        && input.command_envelope_reference_hash == Some(previous.command_envelope_reference_hash)
        && input.command_body_canonicalization_hash
            == Some(previous.command_body_canonicalization_hash)
        && input.handler_binding_hash == Some(previous.handler_binding_hash)
        && input.status_read_handler_hash == Some(previous.status_read_handler_hash)
        && input.rollback_preview_authorization_hash
            == Some(previous.rollback_preview_authorization_hash)
        && input.rollback_apply_authorization_hash
            == Some(previous.rollback_apply_authorization_hash)
        && input.disable_module_target_binding_hash
            == Some(previous.disable_module_target_binding_hash)
        && input.restart_last_good_target_binding_hash
            == Some(previous.restart_last_good_target_binding_hash)
        && input.load_artifact_by_hash_target_binding_hash
            == Some(previous.load_artifact_by_hash_target_binding_hash)
        && input.recovery_memory_write_authority_hash
            == Some(previous.recovery_memory_write_authority_hash)
        && input.durable_audit_rollback_write_authority_hash
            == Some(previous.durable_audit_rollback_write_authority_hash)
        && input.service_inventory_side_effect_boundary_hash
            == Some(previous.service_inventory_side_effect_boundary_hash)
        && input.command_dispatch_behavior_hash == Some(previous.command_dispatch_behavior_hash)
        && input.executor_capability_table_hash == Some(previous.executor_capability_table_hash)
        && input.side_effect_gate_hash == Some(previous.side_effect_gate_hash)
        && method_eq(target_locator, previous.target_locator.as_str())
        && method_eq(
            command_dispatch_boundary_id,
            previous.command_dispatch_boundary_id,
        )
}

fn recovery_lifeline_command_execution_stage_matches_previous_stage(
    input: &RecoveryLifelineCommandExecutionStageInput<'_>,
    previous: event_log::RecoveryLifelineCommandExecutionStageReference,
) -> bool {
    let Some(command_id) = input.command_id else {
        return false;
    };
    let Some(argument_schema) = input.argument_schema else {
        return false;
    };
    let Some(target_locator) = input.target_locator else {
        return false;
    };
    let Some(command_dispatch_boundary_id) = input.command_dispatch_boundary_id else {
        return false;
    };
    method_eq(command_id, previous.command_id)
        && method_eq(argument_schema, previous.argument_schema)
        && input.argument_hash == Some(previous.argument_hash)
        && input.command_envelope_reference_hash == Some(previous.command_envelope_reference_hash)
        && input.command_body_canonicalization_hash
            == Some(previous.command_body_canonicalization_hash)
        && input.handler_binding_hash == Some(previous.handler_binding_hash)
        && input.status_read_handler_hash == Some(previous.status_read_handler_hash)
        && input.rollback_preview_authorization_hash
            == Some(previous.rollback_preview_authorization_hash)
        && input.rollback_apply_authorization_hash
            == Some(previous.rollback_apply_authorization_hash)
        && input.disable_module_target_binding_hash
            == Some(previous.disable_module_target_binding_hash)
        && input.restart_last_good_target_binding_hash
            == Some(previous.restart_last_good_target_binding_hash)
        && input.load_artifact_by_hash_target_binding_hash
            == Some(previous.load_artifact_by_hash_target_binding_hash)
        && input.recovery_memory_write_authority_hash
            == Some(previous.recovery_memory_write_authority_hash)
        && input.durable_audit_rollback_write_authority_hash
            == Some(previous.durable_audit_rollback_write_authority_hash)
        && input.service_inventory_side_effect_boundary_hash
            == Some(previous.service_inventory_side_effect_boundary_hash)
        && input.command_dispatch_behavior_hash == Some(previous.command_dispatch_behavior_hash)
        && input.executor_capability_table_hash == Some(previous.executor_capability_table_hash)
        && input.side_effect_gate_hash == Some(previous.side_effect_gate_hash)
        && input.execution_enablement_hash == previous.execution_enablement_hash
        && input.execution_preflight_hash == previous.execution_preflight_hash
        && input.execution_intent_hash == previous.execution_intent_hash
        && input.execution_commit_gate_hash == previous.execution_commit_gate_hash
        && input.execution_result_denial_hash == previous.execution_result_denial_hash
        && input.execution_audit_denial_hash == previous.execution_audit_denial_hash
        && input.execution_observation_denial_hash == previous.execution_observation_denial_hash
        && method_eq(target_locator, previous.target_locator.as_str())
        && method_eq(
            command_dispatch_boundary_id,
            previous.command_dispatch_boundary_id,
        )
}

fn recovery_lifeline_command_previous_execution_stage_descriptor(
    descriptor: RecoveryLifelineCommandExecutionStageDescriptor,
) -> Option<RecoveryLifelineCommandExecutionStageDescriptor> {
    if descriptor.index == 1 {
        Some(RECOVERY_LIFELINE_COMMAND_EXECUTION_ENABLEMENT_STAGE)
    } else if descriptor.index == 2 {
        Some(RECOVERY_LIFELINE_COMMAND_EXECUTION_PREFLIGHT_STAGE)
    } else if descriptor.index == 3 {
        Some(RECOVERY_LIFELINE_COMMAND_EXECUTION_INTENT_STAGE)
    } else if descriptor.index == 4 {
        Some(RECOVERY_LIFELINE_COMMAND_EXECUTION_COMMIT_GATE_STAGE)
    } else if descriptor.index == 5 {
        Some(RECOVERY_LIFELINE_COMMAND_EXECUTION_RESULT_DENIAL_STAGE)
    } else if descriptor.index == 6 {
        Some(RECOVERY_LIFELINE_COMMAND_EXECUTION_AUDIT_DENIAL_STAGE)
    } else if descriptor.index == 7 {
        Some(RECOVERY_LIFELINE_COMMAND_EXECUTION_OBSERVATION_DENIAL_STAGE)
    } else {
        None
    }
}

pub(crate) fn recovery_lifeline_command_execution_stage_from_check(
    check: &RecoveryLifelineCommandExecutionStageReferenceCheck<'_>,
) -> Option<event_log::RecoveryLifelineCommandExecutionStageReference> {
    let spec = check.normalized_spec?;
    let execution_stage_hash = check.execution_stage_hash?;
    let execution_enablement_hash = if check.descriptor.index == 0 {
        Some(execution_stage_hash)
    } else {
        check.execution_enablement_hash
    };
    let execution_preflight_hash = if check.descriptor.index == 1 {
        Some(execution_stage_hash)
    } else {
        check.execution_preflight_hash
    };
    let execution_intent_hash = if check.descriptor.index == 2 {
        Some(execution_stage_hash)
    } else {
        check.execution_intent_hash
    };
    let execution_commit_gate_hash = if check.descriptor.index == 3 {
        Some(execution_stage_hash)
    } else {
        check.execution_commit_gate_hash
    };
    let execution_result_denial_hash = if check.descriptor.index == 4 {
        Some(execution_stage_hash)
    } else {
        check.execution_result_denial_hash
    };
    let execution_audit_denial_hash = if check.descriptor.index == 5 {
        Some(execution_stage_hash)
    } else {
        check.execution_audit_denial_hash
    };
    let execution_observation_denial_hash = if check.descriptor.index == 6 {
        Some(execution_stage_hash)
    } else {
        check.execution_observation_denial_hash
    };
    Some(event_log::RecoveryLifelineCommandExecutionStageReference {
        schema: check.descriptor.reference_schema,
        stage_name: check.descriptor.stage_name,
        execution_stage_hash,
        retained_previous_stage_event_id: parse_current_boot_event_id(
            check.retained_previous_stage_event_id?,
        )?,
        command_id: spec.command_id,
        argument_schema: spec.argument_schema,
        argument_hash: check.argument_hash?,
        target_locator: check.target_locator_value?,
        command_envelope_reference_hash: check.command_envelope_reference_hash?,
        command_body_canonicalization_hash: check.command_body_canonicalization_hash?,
        handler_binding_hash: check.handler_binding_hash?,
        status_read_handler_hash: check.status_read_handler_hash?,
        rollback_preview_authorization_hash: check.rollback_preview_authorization_hash?,
        rollback_apply_authorization_hash: check.rollback_apply_authorization_hash?,
        disable_module_target_binding_hash: check.disable_module_target_binding_hash?,
        restart_last_good_target_binding_hash: check.restart_last_good_target_binding_hash?,
        load_artifact_by_hash_target_binding_hash: check
            .load_artifact_by_hash_target_binding_hash?,
        recovery_memory_write_authority_hash: check.recovery_memory_write_authority_hash?,
        durable_audit_rollback_write_authority_hash: check
            .durable_audit_rollback_write_authority_hash?,
        service_inventory_side_effect_boundary_hash: check
            .service_inventory_side_effect_boundary_hash?,
        command_dispatch_behavior_hash: check.command_dispatch_behavior_hash?,
        executor_capability_table_hash: check.executor_capability_table_hash?,
        side_effect_gate_hash: check.side_effect_gate_hash?,
        execution_enablement_hash,
        execution_preflight_hash,
        execution_intent_hash,
        execution_commit_gate_hash,
        execution_result_denial_hash,
        execution_audit_denial_hash,
        execution_observation_denial_hash,
        command_dispatch_boundary_id: RECOVERY_COMMAND_DISPATCH_BOUNDARY_ID,
        execution_stage_id: check.descriptor.stage_id,
        execution_stage_projection_hash: check.execution_stage_projection_hash?,
    })
}

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
