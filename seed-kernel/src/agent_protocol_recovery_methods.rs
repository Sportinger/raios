use crate::agent_protocol_support::{method_eq, method_head_eq};

pub(crate) const RECOVERY_ARTIFACT_LOAD_METHOD: &str = "recovery.load_artifact";
pub(crate) const MODULE_RECOVERY_ARTIFACT_LOAD_METHOD: &str = "module.load_recovery_artifact";
pub(crate) const RECOVERY_ARTIFACT_LOAD_BINDING_METHOD: &str = "recovery.load_binding";
pub(crate) const MODULE_RECOVERY_ARTIFACT_LOAD_BINDING_METHOD: &str =
    "module.recovery_load_binding";
pub(crate) const RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD: &str =
    "recovery.load_binding_selftest";
pub(crate) const MODULE_RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD: &str =
    "module.recovery_load_binding_selftest";

pub(crate) fn recovery_artifact_load_method(method: &str) -> bool {
    method_eq(method, RECOVERY_ARTIFACT_LOAD_METHOD)
        || method_eq(method, MODULE_RECOVERY_ARTIFACT_LOAD_METHOD)
}

pub(crate) fn recovery_artifact_identity_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.identity_diagnostic")
}

pub(crate) fn recovery_artifact_identity_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.identity_diagnostic_selftest")
}

pub(crate) fn recovery_artifact_trust_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.trust_diagnostic")
}

pub(crate) fn recovery_artifact_trust_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.trust_diagnostic_selftest")
}

pub(crate) fn recovery_artifact_vm_test_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.vm_test_diagnostic")
}

pub(crate) fn recovery_artifact_vm_test_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.vm_test_diagnostic_selftest")
}

pub(crate) fn recovery_artifact_local_approval_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.local_approval_diagnostic")
}

pub(crate) fn recovery_artifact_local_approval_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.local_approval_diagnostic_selftest")
}

pub(crate) fn recovery_artifact_loader_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.loader_diagnostic")
}

pub(crate) fn recovery_artifact_loader_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.loader_diagnostic_selftest")
}

pub(crate) fn recovery_artifact_rollback_evidence_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.rollback_evidence_diagnostic")
}

pub(crate) fn recovery_artifact_rollback_evidence_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.rollback_evidence_diagnostic_selftest")
}

pub(crate) fn recovery_lifeline_request_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_request_diagnostic")
}

pub(crate) fn recovery_lifeline_request_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_request_diagnostic_selftest")
}

pub(crate) fn recovery_lifeline_protocol_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_protocol_diagnostic")
}

pub(crate) fn recovery_lifeline_protocol_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_protocol_diagnostic_selftest")
}

pub(crate) fn recovery_lifeline_command_vocabulary_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_command_vocabulary")
        || method_head_eq(method, "recovery.lifeline_command_vocabulary_diagnostic")
}

pub(crate) fn recovery_lifeline_command_vocabulary_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_command_vocabulary_selftest")
        || method_head_eq(
            method,
            "recovery.lifeline_command_vocabulary_diagnostic_selftest",
        )
}

pub(crate) fn recovery_loader_runtime_isolation_method(method: &str) -> bool {
    method_head_eq(method, "recovery.loader_runtime_isolation")
        || method_head_eq(method, "recovery.loader_runtime_isolation_diagnostic")
}

pub(crate) fn recovery_loader_runtime_isolation_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.loader_runtime_isolation_selftest")
        || method_head_eq(
            method,
            "recovery.loader_runtime_isolation_diagnostic_selftest",
        )
}

pub(crate) fn recovery_rollback_transaction_engine_method(method: &str) -> bool {
    method_head_eq(method, "recovery.rollback_transaction_engine")
        || method_head_eq(method, "recovery.rollback_transaction_engine_diagnostic")
}

pub(crate) fn recovery_rollback_transaction_engine_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.rollback_transaction_engine_selftest")
        || method_head_eq(
            method,
            "recovery.rollback_transaction_engine_diagnostic_selftest",
        )
}

pub(crate) fn recovery_durable_audit_rollback_persistence_method(method: &str) -> bool {
    method_head_eq(method, "recovery.durable_audit_rollback_persistence")
        || method_head_eq(
            method,
            "recovery.durable_audit_rollback_persistence_diagnostic",
        )
}

pub(crate) fn recovery_durable_audit_rollback_persistence_selftest_method(method: &str) -> bool {
    method_head_eq(
        method,
        "recovery.durable_audit_rollback_persistence_selftest",
    ) || method_head_eq(
        method,
        "recovery.durable_audit_rollback_persistence_diagnostic_selftest",
    )
}

pub(crate) fn recovery_memory_provenance_method(method: &str) -> bool {
    method_head_eq(method, "recovery.memory_provenance")
        || method_head_eq(method, "recovery.memory_provenance_diagnostic")
}

pub(crate) fn recovery_memory_provenance_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.memory_provenance_selftest")
        || method_head_eq(method, "recovery.memory_provenance_diagnostic_selftest")
}

pub(crate) fn recovery_lifeline_command_admission_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_command_admission")
        || method_head_eq(method, "recovery.lifeline_command_admission_diagnostic")
}

pub(crate) fn recovery_lifeline_command_admission_selftest_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_command_admission_selftest")
        || method_head_eq(
            method,
            "recovery.lifeline_command_admission_diagnostic_selftest",
        )
}

pub(crate) fn recovery_lifeline_command_envelope_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_command_envelope_diagnostic")
        || method_head_eq(method, "recovery.lifeline_command_envelope_reference")
}

pub(crate) fn recovery_lifeline_command_envelope_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(
        method,
        "recovery.lifeline_command_envelope_diagnostic_selftest",
    ) || method_head_eq(
        method,
        "recovery.lifeline_command_envelope_reference_selftest",
    )
}

pub(crate) fn recovery_lifeline_command_dispatch_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_command_dispatch_diagnostic")
        || method_head_eq(method, "recovery.lifeline_command_dispatch_denial")
}

pub(crate) fn recovery_lifeline_command_dispatch_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(
        method,
        "recovery.lifeline_command_dispatch_diagnostic_selftest",
    ) || method_head_eq(method, "recovery.lifeline_command_dispatch_denial_selftest")
}

pub(crate) fn recovery_lifeline_command_dispatch_behavior_diagnostic_method(method: &str) -> bool {
    method_head_eq(
        method,
        "recovery.lifeline_command_dispatch_behavior_diagnostic",
    ) || method_head_eq(method, "recovery.lifeline_command_dispatch_behavior")
}

pub(crate) fn recovery_lifeline_command_dispatch_behavior_diagnostic_selftest_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.lifeline_command_dispatch_behavior_diagnostic_selftest",
    ) || method_head_eq(
        method,
        "recovery.lifeline_command_dispatch_behavior_selftest",
    )
}

pub(crate) fn recovery_lifeline_command_executor_capability_table_diagnostic_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.lifeline_command_executor_capability_table_diagnostic",
    ) || method_head_eq(
        method,
        "recovery.lifeline_command_executor_capability_table",
    )
}

pub(crate) fn recovery_lifeline_command_executor_capability_table_diagnostic_selftest_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.lifeline_command_executor_capability_table_diagnostic_selftest",
    ) || method_head_eq(
        method,
        "recovery.lifeline_command_executor_capability_table_selftest",
    )
}

pub(crate) fn recovery_lifeline_command_side_effect_gate_diagnostic_method(method: &str) -> bool {
    method_head_eq(
        method,
        "recovery.lifeline_command_side_effect_gate_diagnostic",
    ) || method_head_eq(method, "recovery.lifeline_command_side_effect_gate")
}

pub(crate) fn recovery_lifeline_command_side_effect_gate_diagnostic_selftest_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.lifeline_command_side_effect_gate_diagnostic_selftest",
    ) || method_head_eq(
        method,
        "recovery.lifeline_command_side_effect_gate_selftest",
    )
}

pub(crate) fn recovery_lifeline_command_body_canonicalization_diagnostic_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.lifeline_command_body_canonicalization_diagnostic",
    ) || method_head_eq(method, "recovery.lifeline_command_body_canonicalization")
}

pub(crate) fn recovery_lifeline_command_body_canonicalization_diagnostic_selftest_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.lifeline_command_body_canonicalization_diagnostic_selftest",
    ) || method_head_eq(
        method,
        "recovery.lifeline_command_body_canonicalization_selftest",
    )
}

pub(crate) fn recovery_lifeline_command_handler_binding_diagnostic_method(method: &str) -> bool {
    method_head_eq(
        method,
        "recovery.lifeline_command_handler_binding_diagnostic",
    ) || method_head_eq(method, "recovery.lifeline_command_handler_binding")
}

pub(crate) fn recovery_lifeline_command_handler_binding_diagnostic_selftest_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.lifeline_command_handler_binding_diagnostic_selftest",
    ) || method_head_eq(method, "recovery.lifeline_command_handler_binding_selftest")
}

pub(crate) fn recovery_lifeline_status_read_handler_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.lifeline_status_read_handler_diagnostic")
        || method_head_eq(method, "recovery.lifeline_status_read_handler")
}

pub(crate) fn recovery_lifeline_status_read_handler_diagnostic_selftest_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.lifeline_status_read_handler_diagnostic_selftest",
    ) || method_head_eq(method, "recovery.lifeline_status_read_handler_selftest")
}

pub(crate) fn recovery_rollback_preview_authorization_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.rollback_preview_authorization_diagnostic")
        || method_head_eq(method, "recovery.rollback_preview_authorization")
}

pub(crate) fn recovery_rollback_preview_authorization_diagnostic_selftest_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.rollback_preview_authorization_diagnostic_selftest",
    ) || method_head_eq(method, "recovery.rollback_preview_authorization_selftest")
}

pub(crate) fn recovery_rollback_apply_authorization_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.rollback_apply_authorization_diagnostic")
        || method_head_eq(method, "recovery.rollback_apply_authorization")
}

pub(crate) fn recovery_rollback_apply_authorization_diagnostic_selftest_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.rollback_apply_authorization_diagnostic_selftest",
    ) || method_head_eq(method, "recovery.rollback_apply_authorization_selftest")
}

pub(crate) fn recovery_disable_module_target_binding_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.disable_module_target_binding_diagnostic")
        || method_head_eq(method, "recovery.disable_module_target_binding")
}

pub(crate) fn recovery_disable_module_target_binding_diagnostic_selftest_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.disable_module_target_binding_diagnostic_selftest",
    ) || method_head_eq(method, "recovery.disable_module_target_binding_selftest")
}

pub(crate) fn recovery_restart_last_good_target_binding_diagnostic_method(method: &str) -> bool {
    method_head_eq(
        method,
        "recovery.restart_last_good_target_binding_diagnostic",
    ) || method_head_eq(method, "recovery.restart_last_good_target_binding")
}

pub(crate) fn recovery_restart_last_good_target_binding_diagnostic_selftest_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.restart_last_good_target_binding_diagnostic_selftest",
    ) || method_head_eq(method, "recovery.restart_last_good_target_binding_selftest")
}

pub(crate) fn recovery_load_artifact_by_hash_target_binding_diagnostic_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.load_artifact_by_hash_target_binding_diagnostic",
    ) || method_head_eq(method, "recovery.load_artifact_by_hash_target_binding")
}

pub(crate) fn recovery_load_artifact_by_hash_target_binding_diagnostic_selftest_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.load_artifact_by_hash_target_binding_diagnostic_selftest",
    ) || method_head_eq(
        method,
        "recovery.load_artifact_by_hash_target_binding_selftest",
    )
}

pub(crate) fn recovery_memory_write_authority_diagnostic_method(method: &str) -> bool {
    method_head_eq(method, "recovery.memory_write_authority_diagnostic")
        || method_head_eq(method, "recovery.memory_write_authority")
}

pub(crate) fn recovery_memory_write_authority_diagnostic_selftest_method(method: &str) -> bool {
    method_head_eq(
        method,
        "recovery.memory_write_authority_diagnostic_selftest",
    ) || method_head_eq(method, "recovery.memory_write_authority_selftest")
}

pub(crate) fn durable_audit_rollback_write_authority_diagnostic_method(method: &str) -> bool {
    method_head_eq(
        method,
        "recovery.durable_audit_rollback_write_authority_diagnostic",
    ) || method_head_eq(method, "recovery.durable_audit_rollback_write_authority")
}

pub(crate) fn durable_audit_rollback_write_authority_diagnostic_selftest_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.durable_audit_rollback_write_authority_diagnostic_selftest",
    ) || method_head_eq(
        method,
        "recovery.durable_audit_rollback_write_authority_selftest",
    )
}

pub(crate) fn recovery_service_inventory_side_effect_boundary_diagnostic_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.service_inventory_side_effect_boundary_diagnostic",
    ) || method_head_eq(method, "recovery.service_inventory_side_effect_boundary")
}

pub(crate) fn recovery_service_inventory_side_effect_boundary_diagnostic_selftest_method(
    method: &str,
) -> bool {
    method_head_eq(
        method,
        "recovery.service_inventory_side_effect_boundary_diagnostic_selftest",
    ) || method_head_eq(
        method,
        "recovery.service_inventory_side_effect_boundary_selftest",
    )
}

pub(crate) fn recovery_artifact_load_binding_method(method: &str) -> bool {
    method_head_eq(method, RECOVERY_ARTIFACT_LOAD_BINDING_METHOD)
        || method_head_eq(method, MODULE_RECOVERY_ARTIFACT_LOAD_BINDING_METHOD)
}

pub(crate) fn recovery_artifact_load_binding_selftest_method(method: &str) -> bool {
    method_head_eq(method, RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD)
        || method_head_eq(
            method,
            MODULE_RECOVERY_ARTIFACT_LOAD_BINDING_SELFTEST_METHOD,
        )
}

pub(crate) fn canonical_recovery_artifact_load_method(method: &str) -> &'static str {
    if method_eq(method, MODULE_RECOVERY_ARTIFACT_LOAD_METHOD) {
        MODULE_RECOVERY_ARTIFACT_LOAD_METHOD
    } else {
        RECOVERY_ARTIFACT_LOAD_METHOD
    }
}

pub(crate) fn recovery_identity_diagnostic_arg(method: &str) -> &str {
    method_arg(method, "recovery.identity_diagnostic")
}

pub(crate) fn recovery_trust_diagnostic_arg(method: &str) -> &str {
    method_arg(method, "recovery.trust_diagnostic")
}

pub(crate) fn recovery_vm_test_diagnostic_arg(method: &str) -> &str {
    method_arg(method, "recovery.vm_test_diagnostic")
}

pub(crate) fn recovery_local_approval_diagnostic_arg(method: &str) -> &str {
    method_arg(method, "recovery.local_approval_diagnostic")
}

pub(crate) fn recovery_loader_diagnostic_arg(method: &str) -> &str {
    method_arg(method, "recovery.loader_diagnostic")
}

pub(crate) fn recovery_rollback_evidence_diagnostic_arg(method: &str) -> &str {
    method_arg(method, "recovery.rollback_evidence_diagnostic")
}

pub(crate) fn recovery_lifeline_request_diagnostic_arg(method: &str) -> &str {
    method_arg(method, "recovery.lifeline_request_diagnostic")
}

pub(crate) fn recovery_lifeline_command_envelope_diagnostic_arg(method: &str) -> &str {
    method_arg_with_alias(
        method,
        "recovery.lifeline_command_envelope_diagnostic",
        "recovery.lifeline_command_envelope_reference",
    )
}

pub(crate) fn recovery_lifeline_command_body_canonicalization_diagnostic_arg(method: &str) -> &str {
    method_arg_with_alias(
        method,
        "recovery.lifeline_command_body_canonicalization_diagnostic",
        "recovery.lifeline_command_body_canonicalization",
    )
}

pub(crate) fn recovery_lifeline_command_handler_binding_diagnostic_arg(method: &str) -> &str {
    method_arg_with_alias(
        method,
        "recovery.lifeline_command_handler_binding_diagnostic",
        "recovery.lifeline_command_handler_binding",
    )
}

pub(crate) fn recovery_lifeline_status_read_handler_diagnostic_arg(method: &str) -> &str {
    method_arg_with_alias(
        method,
        "recovery.lifeline_status_read_handler_diagnostic",
        "recovery.lifeline_status_read_handler",
    )
}

pub(crate) fn recovery_rollback_preview_authorization_diagnostic_arg(method: &str) -> &str {
    method_arg_with_alias(
        method,
        "recovery.rollback_preview_authorization_diagnostic",
        "recovery.rollback_preview_authorization",
    )
}

pub(crate) fn recovery_rollback_apply_authorization_diagnostic_arg(method: &str) -> &str {
    method_arg_with_alias(
        method,
        "recovery.rollback_apply_authorization_diagnostic",
        "recovery.rollback_apply_authorization",
    )
}

pub(crate) fn recovery_disable_module_target_binding_diagnostic_arg(method: &str) -> &str {
    method_arg_with_alias(
        method,
        "recovery.disable_module_target_binding_diagnostic",
        "recovery.disable_module_target_binding",
    )
}

pub(crate) fn recovery_restart_last_good_target_binding_diagnostic_arg(method: &str) -> &str {
    method_arg_with_alias(
        method,
        "recovery.restart_last_good_target_binding_diagnostic",
        "recovery.restart_last_good_target_binding",
    )
}

pub(crate) fn recovery_load_artifact_by_hash_target_binding_diagnostic_arg(method: &str) -> &str {
    method_arg_with_alias(
        method,
        "recovery.load_artifact_by_hash_target_binding_diagnostic",
        "recovery.load_artifact_by_hash_target_binding",
    )
}

pub(crate) fn recovery_memory_write_authority_diagnostic_arg(method: &str) -> &str {
    method_arg_with_alias(
        method,
        "recovery.memory_write_authority_diagnostic",
        "recovery.memory_write_authority",
    )
}

pub(crate) fn durable_audit_rollback_write_authority_diagnostic_arg(method: &str) -> &str {
    method_arg_with_alias(
        method,
        "recovery.durable_audit_rollback_write_authority_diagnostic",
        "recovery.durable_audit_rollback_write_authority",
    )
}

pub(crate) fn recovery_service_inventory_side_effect_boundary_diagnostic_arg(method: &str) -> &str {
    method_arg_with_alias(
        method,
        "recovery.service_inventory_side_effect_boundary_diagnostic",
        "recovery.service_inventory_side_effect_boundary",
    )
}

pub(crate) fn recovery_lifeline_command_dispatch_behavior_diagnostic_arg(method: &str) -> &str {
    method_arg_with_alias(
        method,
        "recovery.lifeline_command_dispatch_behavior_diagnostic",
        "recovery.lifeline_command_dispatch_behavior",
    )
}

pub(crate) fn recovery_lifeline_command_executor_capability_table_diagnostic_arg(
    method: &str,
) -> &str {
    method_arg_with_alias(
        method,
        "recovery.lifeline_command_executor_capability_table_diagnostic",
        "recovery.lifeline_command_executor_capability_table",
    )
}

pub(crate) fn recovery_lifeline_command_side_effect_gate_diagnostic_arg(method: &str) -> &str {
    method_arg_with_alias(
        method,
        "recovery.lifeline_command_side_effect_gate_diagnostic",
        "recovery.lifeline_command_side_effect_gate",
    )
}

fn method_arg<'a>(method: &'a str, head: &str) -> &'a str {
    let method = method.trim();
    if method_head_eq(method, head) {
        method[head.len()..].trim()
    } else {
        ""
    }
}

fn method_arg_with_alias<'a>(method: &'a str, head: &str, alias: &str) -> &'a str {
    let method = method.trim();
    if method_head_eq(method, head) {
        method[head.len()..].trim()
    } else if method_head_eq(method, alias) {
        method[alias.len()..].trim()
    } else {
        ""
    }
}
