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
