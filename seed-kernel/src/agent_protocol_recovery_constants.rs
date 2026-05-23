pub(crate) const RECOVERY_ARTIFACT_LOAD_CAPABILITY: &str = "cap.recovery.load_artifact";
pub(crate) const RECOVERY_ARTIFACT_LOAD_READ_CAPABILITY: &str = "cap.recovery.load_artifact.read";

pub(crate) const RECOVERY_LOAD_BINDING_SELFTEST_CASES: usize = 14;
pub(crate) const RECOVERY_IDENTITY_SELFTEST_CASES: usize = 6;
pub(crate) const RECOVERY_TRUST_SELFTEST_CASES: usize = 8;
pub(crate) const RECOVERY_VM_TEST_SELFTEST_CASES: usize = 10;
pub(crate) const RECOVERY_LOCAL_APPROVAL_SELFTEST_CASES: usize = 11;
pub(crate) const RECOVERY_LOADER_SELFTEST_CASES: usize = 10;
pub(crate) const RECOVERY_ROLLBACK_EVIDENCE_SELFTEST_CASES: usize = 10;
pub(crate) const RECOVERY_LIFELINE_REQUEST_SELFTEST_CASES: usize = 11;
pub(crate) const RECOVERY_LIFELINE_PROTOCOL_SELFTEST_CASES: usize = 15;
pub(crate) const RECOVERY_LIFELINE_COMMAND_VOCABULARY_SELFTEST_CASES: usize = 16;
pub(crate) const RECOVERY_LOADER_RUNTIME_ISOLATION_SELFTEST_CASES: usize = 27;
pub(crate) const RECOVERY_ROLLBACK_TRANSACTION_ENGINE_SELFTEST_CASES: usize = 38;
pub(crate) const RECOVERY_DURABLE_AUDIT_ROLLBACK_PERSISTENCE_SELFTEST_CASES: usize = 51;
pub(crate) const RECOVERY_MEMORY_PROVENANCE_SELFTEST_CASES: usize = 65;
pub(crate) const RECOVERY_LIFELINE_COMMAND_ADMISSION_SELFTEST_CASES: usize = 45;
pub(crate) const RECOVERY_LIFELINE_COMMAND_ENVELOPE_SELFTEST_CASES: usize = 47;
pub(crate) const RECOVERY_LIFELINE_COMMAND_DISPATCH_SELFTEST_CASES: usize = 51;
pub(crate) const RECOVERY_LIFELINE_COMMAND_BODY_CANONICALIZATION_SELFTEST_CASES: usize = 43;
pub(crate) const RECOVERY_LIFELINE_COMMAND_HANDLER_BINDING_SELFTEST_CASES: usize = 10;
pub(crate) const RECOVERY_LIFELINE_STATUS_READ_HANDLER_SELFTEST_CASES: usize = 10;
pub(crate) const RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_SELFTEST_CASES: usize = 10;
pub(crate) const RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_SELFTEST_CASES: usize = 10;
pub(crate) const RECOVERY_DISABLE_MODULE_TARGET_BINDING_SELFTEST_CASES: usize = 10;
pub(crate) const RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_SELFTEST_CASES: usize = 10;
pub(crate) const RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_SELFTEST_CASES: usize = 10;
pub(crate) const RECOVERY_MEMORY_WRITE_AUTHORITY_SELFTEST_CASES: usize = 10;
pub(crate) const DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_SELFTEST_CASES: usize = 10;
pub(crate) const RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_SELFTEST_CASES: usize = 10;
pub(crate) const RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_SELFTEST_CASES: usize = 10;
pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_SELFTEST_CASES: usize = 10;
pub(crate) const RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_SELFTEST_CASES: usize = 10;

pub(crate) const RECOVERY_COMMAND_ADMISSION_BOUNDARY_ID: &str =
    "boundary.recovery_lifeline_command_admission.current_boot";
pub(crate) const RECOVERY_COMMAND_HANDLER_BINDING_BOUNDARY_ID: &str =
    "boundary.recovery_lifeline_command_handler_binding.current_boot";
pub(crate) const RECOVERY_STATUS_READ_HANDLER_BOUNDARY_ID: &str =
    "boundary.recovery_lifeline_status_read_handler.current_boot";
pub(crate) const RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_BOUNDARY_ID: &str =
    "boundary.recovery_rollback_preview_authorization.current_boot";
pub(crate) const RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_BOUNDARY_ID: &str =
    "boundary.recovery_rollback_apply_authorization.current_boot";
pub(crate) const RECOVERY_DISABLE_MODULE_TARGET_BINDING_BOUNDARY_ID: &str =
    "boundary.recovery_disable_module_target_binding.current_boot";
pub(crate) const RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_BOUNDARY_ID: &str =
    "boundary.recovery_restart_last_good_target_binding.current_boot";
pub(crate) const RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_BOUNDARY_ID: &str =
    "boundary.recovery_load_artifact_by_hash_target_binding.current_boot";
pub(crate) const RECOVERY_MEMORY_WRITE_AUTHORITY_BOUNDARY_ID: &str =
    "boundary.recovery_memory_write_authority.current_boot";
pub(crate) const DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_BOUNDARY_ID: &str =
    "boundary.durable_audit_rollback_write_authority.current_boot";
pub(crate) const RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_ID: &str =
    "boundary.recovery_service_inventory_side_effect_boundary.current_boot";
pub(crate) const RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_BOUNDARY_ID: &str =
    "boundary.recovery_lifeline_command_dispatch_behavior.current_boot";
pub(crate) const RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_BOUNDARY_ID: &str =
    "boundary.recovery_lifeline_command_executor_capability_table.current_boot";
pub(crate) const RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_BOUNDARY_ID: &str =
    "boundary.recovery_lifeline_command_side_effect_gate.current_boot";
