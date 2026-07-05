use super::*;

pub(crate) struct ServiceDescriptor {
    pub(crate) service_id: &'static str,
    pub(crate) artifact_id: &'static str,
    pub(crate) artifact_kind: &'static str,
    pub(crate) scope: &'static str,
    pub(crate) classification: &'static str,
    pub(crate) persistence: &'static str,
    pub(crate) service_capability: &'static str,
    pub(crate) health_capability: &'static str,
    pub(crate) rollback_preview_capability: &'static str,
    pub(crate) rollback_apply_capability: &'static str,
    pub(crate) rollback_materialize_capability: &'static str,
    pub(crate) rollback_inspect_capability: &'static str,
    pub(crate) primary_alias: &'static str,
    pub(crate) host_bound_alias: &'static str,
    pub(crate) replacement_service_id: &'static str,
    pub(crate) replacement_alias: &'static str,
    pub(crate) replacement_artifact_identity_id: &'static str,
    pub(crate) reset_state_service_id: &'static str,
    pub(crate) reset_state_alias: &'static str,
    pub(crate) artifact_load_plan_preflight_id: &'static str,
    pub(crate) artifact_load_plan_preflight_status: &'static str,
    pub(crate) service_slot_intent_id: &'static str,
    pub(crate) ram_only_service_slot_id: &'static str,
    pub(crate) service_slot_activation_id: &'static str,
    pub(crate) service_slot_activation_active_status: &'static str,
    pub(crate) service_slot_activation_stopped_status: &'static str,
    pub(crate) service_slot_activation_cleared_status: &'static str,
    pub(crate) service_slot_activation_missing_status: &'static str,
    pub(crate) inventory_kind: &'static str,
    pub(crate) inventory_replaceable: bool,
    pub(crate) inventory_core_owned: bool,
    pub(crate) inventory_health_running: &'static str,
    pub(crate) inventory_health_stopped: &'static str,
    pub(crate) inventory_health_missing: &'static str,
    pub(crate) event_lifecycle_kind: &'static str,
    pub(crate) event_health_kind: &'static str,
    pub(crate) event_rollback_preview_kind: &'static str,
    pub(crate) event_rollback_apply_kind: &'static str,
}

pub(crate) const HELLO_SERVICE_DESCRIPTOR: ServiceDescriptor = ServiceDescriptor {
    service_id: descriptor_sources::HELLO_SERVICE_ID,
    artifact_id: descriptor_sources::HELLO_ARTIFACT_ID,
    artifact_kind: "builtin_stage0_test_service",
    scope: "current_boot",
    classification: "local_only",
    persistence: "none",
    service_capability: "cap.service.hello_demo.current_boot",
    health_capability: "cap.service.health.read",
    rollback_preview_capability: "cap.service.rollback_preview.read",
    rollback_apply_capability: "cap.service.rollback_apply.current_boot",
    rollback_materialize_capability: "cap.recovery.rollback_materialize_dry_run.current_boot",
    rollback_inspect_capability: "cap.recovery.rollback_inspect.read",
    primary_alias: "hello",
    host_bound_alias: "host_bound:svc.demo.hello",
    replacement_service_id: "svc.demo.hello.v2",
    replacement_alias: "hello.v2",
    replacement_artifact_identity_id: descriptor_sources::HELLO_BUILTIN_ARTIFACT_IDENTITY_V2_ID,
    reset_state_service_id: "svc.demo.hello.reset_state",
    reset_state_alias: "hello.reset_state",
    artifact_load_plan_preflight_id: "artifact_load_plan_preflight.current_boot.svc.demo.hello.v0",
    artifact_load_plan_preflight_status: "accepted_builtin_current_boot_only",
    service_slot_intent_id: "service_slot_intent.current_boot.svc.demo.hello.v0",
    ram_only_service_slot_id: "ram_only:svc.demo.hello",
    service_slot_activation_id: "service_slot_activation.current_boot.svc.demo.hello.v0",
    service_slot_activation_active_status: "active_current_boot",
    service_slot_activation_stopped_status: "stopped_current_boot",
    service_slot_activation_cleared_status: "cleared_current_boot",
    service_slot_activation_missing_status: "missing_current_boot",
    inventory_kind: "service",
    inventory_replaceable: true,
    inventory_core_owned: false,
    inventory_health_running: "healthy",
    inventory_health_stopped: "stopped",
    inventory_health_missing: "missing",
    event_lifecycle_kind: "raios.ram_only_hello_service.lifecycle",
    event_health_kind: "raios.ram_only_hello_service.health",
    event_rollback_preview_kind: "raios.ram_only_hello_service.rollback_preview",
    event_rollback_apply_kind: "raios.ram_only_hello_service.rollback_apply",
};

pub(crate) const SERVICE_ID: &str = HELLO_SERVICE_DESCRIPTOR.service_id;
pub(crate) const ARTIFACT_ID: &str = HELLO_SERVICE_DESCRIPTOR.artifact_id;
pub(crate) const CAPABILITIES: &[&str] = &[HELLO_SERVICE_DESCRIPTOR.service_capability];
pub(crate) const LOAD_DESCRIPTOR_SCHEMA: &str = descriptor_sources::HELLO_LOAD_DESCRIPTOR_SCHEMA;
pub(crate) const LOAD_DESCRIPTOR_ID: &str = descriptor_sources::HELLO_LOAD_DESCRIPTOR_ID;
pub(crate) const LOAD_DESCRIPTOR_CANONICALIZATION: &str =
    descriptor_sources::HELLO_LOAD_DESCRIPTOR_CANONICALIZATION;
pub(crate) const LOAD_DESCRIPTOR_SOURCE_LOCATOR: &str =
    descriptor_sources::HELLO_LOAD_DESCRIPTOR_SOURCE_LOCATOR;
pub(crate) const LOAD_DESCRIPTOR_SOURCE_KIND: &str =
    descriptor_sources::HELLO_LOAD_DESCRIPTOR_SOURCE_KIND;
pub(crate) const ARTIFACT_LOAD_PLAN_PREFLIGHT_SCHEMA: &str =
    "raios.current_boot_artifact_load_plan_preflight.v0";
pub(crate) const ARTIFACT_LOAD_PLAN_PREFLIGHT_ID: &str =
    HELLO_SERVICE_DESCRIPTOR.artifact_load_plan_preflight_id;
pub(crate) const ARTIFACT_LOAD_PLAN_PREFLIGHT_STATUS: &str =
    HELLO_SERVICE_DESCRIPTOR.artifact_load_plan_preflight_status;
pub(crate) const SERVICE_SLOT_INTENT_SCHEMA: &str = "raios.ram_only_service_slot_intent.v0";
pub(crate) const SERVICE_SLOT_INTENT_ID: &str = HELLO_SERVICE_DESCRIPTOR.service_slot_intent_id;
pub(crate) const RAM_ONLY_SERVICE_SLOT_ID: &str = HELLO_SERVICE_DESCRIPTOR.ram_only_service_slot_id;
pub(crate) const SERVICE_SLOT_ACTIVATION_SCHEMA: &str = "raios.ram_only_service_slot_activation.v0";
pub(crate) const SERVICE_SLOT_ACTIVATION_ID: &str =
    HELLO_SERVICE_DESCRIPTOR.service_slot_activation_id;
pub(crate) const SERVICE_SLOT_ACTIVATION_ACTIVE_STATUS: &str =
    HELLO_SERVICE_DESCRIPTOR.service_slot_activation_active_status;
pub(crate) const SERVICE_SLOT_ACTIVATION_STOPPED_STATUS: &str =
    HELLO_SERVICE_DESCRIPTOR.service_slot_activation_stopped_status;
pub(crate) const SERVICE_SLOT_ACTIVATION_CLEARED_STATUS: &str =
    HELLO_SERVICE_DESCRIPTOR.service_slot_activation_cleared_status;
pub(crate) const SERVICE_SLOT_ACTIVATION_MISSING_STATUS: &str =
    HELLO_SERVICE_DESCRIPTOR.service_slot_activation_missing_status;
pub(crate) const HELLO_STATE_SCHEMA: &str = "raios.ram_only_hello_service_state.v0";
pub(crate) const HELLO_STATE_ID: &str = "hello_state.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_STATE_MIGRATION_SCHEMA: &str =
    "raios.ram_only_hello_service_state_migration.v0";
pub(crate) const HELLO_STATE_MIGRATION_ID: &str =
    "hello_state_migration.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_HOT_SWAP_PROBATION_SCHEMA: &str =
    "raios.ram_only_hello_service_hot_swap_probation.v0";
pub(crate) const HELLO_HOT_SWAP_PROBATION_ID: &str =
    "hello_hot_swap_probation.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_HOT_SWAP_PROBATION_STATUS: &str = "active_current_boot_probation";
pub(crate) const HELLO_ROLLBACK_PREVIEW_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_preview.v0";
pub(crate) const HELLO_ROLLBACK_PREVIEW_ID: &str =
    "hello_rollback_preview.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_PREVIEW_STATUS: &str = "preview_only_current_boot";
pub(crate) const HELLO_ROLLBACK_APPLY_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_apply.v0";
pub(crate) const HELLO_ROLLBACK_APPLY_ID: &str =
    "hello_rollback_apply.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_APPLY_STATUS: &str = "denied_missing_rollback_apply_authority";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_transaction_preflight.v0";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_ID: &str =
    "hello_rollback_transaction_preflight.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_PREFLIGHT_STATUS: &str =
    "denied_missing_write_authorities";
pub(crate) const HELLO_ROLLBACK_WRITE_AUTHORITY_GATE_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_write_authority_gate.v0";
pub(crate) const HELLO_ROLLBACK_WRITE_AUTHORITY_GATE_ID: &str =
    "hello_rollback_write_authority_gate.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_WRITE_AUTHORITY_GATE_STATUS: &str =
    "denied_missing_durable_write_authority";
pub(crate) const HELLO_ROLLBACK_APPEND_INTENT_GATE_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_append_intent_gate.v0";
pub(crate) const HELLO_ROLLBACK_APPEND_INTENT_GATE_ID: &str =
    "hello_rollback_append_intent_gate.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_APPEND_INTENT_GATE_STATUS: &str =
    "denied_missing_rollback_transaction_append_authority";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_PAYLOAD_SCHEMA: &str = "raios.rollback_transaction.v0";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_PAYLOAD_ID: &str =
    "rollback_transaction.proposed.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_PAYLOAD_STATUS: &str =
    "proposed_not_appended_current_boot";
pub(crate) const HELLO_ROLLBACK_PAYLOAD_ENVELOPE_GATE_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_payload_envelope_gate.v0";
pub(crate) const HELLO_ROLLBACK_PAYLOAD_ENVELOPE_GATE_ID: &str =
    "hello_rollback_payload_envelope_gate.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_PAYLOAD_ENVELOPE_GATE_STATUS: &str =
    "denied_missing_rollback_transaction_writer";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_AUTHORITY_GATE_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_transaction_writer_storage_authority_gate.v0";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_AUTHORITY_GATE_ID: &str =
    "hello_rollback_transaction_writer_storage_authority_gate.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_AUTHORITY_GATE_STATUS: &str =
    "denied_missing_transaction_writer_storage_authority";
pub(crate) const HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_append_record_dry_run.v0";
pub(crate) const HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_ID: &str =
    "hello_rollback_append_record_dry_run.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_STATUS: &str =
    "scratch_record_image_ready_not_durable_authority";
pub(crate) const HELLO_ROLLBACK_APPEND_RECORD_DRY_RUN_REASON: &str =
    "scratch_writer_dry_run_verified_no_append_authority";
pub(crate) const HELLO_ROLLBACK_APPEND_RECORD_CANONICALIZATION: &str =
    "raios.rollback_append_record_image.canonical.v0";
pub(crate) const HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_append_sector_plan_dry_run.v0";
pub(crate) const HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_ID: &str =
    "hello_rollback_append_sector_plan_dry_run.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_STATUS: &str =
    "scratch_sector_image_ready_not_durable_authority";
pub(crate) const HELLO_ROLLBACK_APPEND_SECTOR_PLAN_DRY_RUN_REASON: &str =
    "append_record_images_fit_scratch_sector_without_write_authority";
pub(crate) const HELLO_ROLLBACK_APPEND_SECTOR_PLAN_CANONICALIZATION: &str =
    "raios.rollback_append_sector_plan.canonical.v0";
pub(crate) const HELLO_ROLLBACK_APPEND_SECTOR_PADDING_POLICY: &str = "zero_pad_to_logical_sector";
pub(crate) const HELLO_ROLLBACK_APPEND_SECTOR_WRITE_READBACK_DRY_RUN_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_append_sector_write_readback_dry_run.v0";
pub(crate) const HELLO_ROLLBACK_APPEND_SECTOR_WRITE_READBACK_DRY_RUN_ID: &str =
    "hello_rollback_append_sector_write_readback_dry_run.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_APPEND_SECTOR_WRITE_READBACK_DRY_RUN_STATUS: &str =
    "scratch_sector_image_readback_verified_not_durable_authority";
pub(crate) const HELLO_ROLLBACK_APPEND_SECTOR_WRITE_READBACK_DRY_RUN_REASON: &str =
    "planned_sector_image_written_and_read_back_on_scratch_only";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_append_authority_preflight.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_ID: &str =
    "hello_rollback_durable_append_authority_preflight.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_STATUS: &str =
    "denied_missing_durable_append_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_PREFLIGHT_REASON: &str =
    "durable_append_authority_not_granted";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_REMAINING_DENIAL_REASON: &str =
    "durable_append_authority_missing";
pub(crate) const HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_writer_policy_preflight.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_ID: &str =
    "hello_rollback_durable_writer_policy_preflight.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_STATUS: &str =
    "candidate_ready_not_append_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_WRITER_POLICY_PREFLIGHT_REASON: &str =
    "writer_candidates_verified_without_append_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_append_transaction_authorization_gate.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_ID: &str =
    "hello_rollback_durable_append_transaction_authorization_gate.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_STATUS: &str =
    "denied_missing_durable_append_transaction_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_TRANSACTION_AUTHORIZATION_GATE_REASON: &str =
    "writer_policy_ready_durable_append_authority_missing";
pub(crate) const HELLO_ROLLBACK_APPEND_ENGINE_READINESS_DECISION_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_append_engine_readiness_decision.v0";
pub(crate) const HELLO_ROLLBACK_APPEND_ENGINE_READINESS_DECISION_ID: &str =
    "hello_rollback_append_engine_readiness_decision.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_media_write_authority_gate.v0";
pub(crate) const HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_ID: &str =
    "hello_rollback_media_write_authority_gate.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_STATUS: &str =
    "denied_missing_durable_audit_policy";
pub(crate) const HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_GATE_REASON: &str =
    "target_region_test_media_write_verified_durable_audit_policy_missing";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_append_authority_decision.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_ID: &str =
    "hello_rollback_durable_append_authority_decision.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_STATUS: &str =
    "denied_missing_durable_append_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_DECISION_REASON: &str =
    "append_engine_ready_durable_audit_policy_missing";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_audit_policy_decision.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_ID: &str =
    "hello_rollback_durable_audit_policy_decision.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_STATUS: &str =
    "denied_missing_durable_audit_policy";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_DECISION_REASON: &str =
    "durable_append_authority_blocked_by_missing_durable_audit_policy";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_audit_policy_candidate.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_ID: &str =
    "hello_rollback_durable_audit_policy_candidate.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_STATUS: &str =
    "candidate_ready_not_durable_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_CANDIDATE_REASON: &str =
    "audit_record_image_and_media_policy_verified_without_write_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_audit_policy_acceptance_gate.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_ID: &str =
    "hello_rollback_durable_audit_policy_acceptance_gate.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_STATUS: &str =
    "denied_missing_durable_policy_ledger_or_write_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_ACCEPTANCE_GATE_REASON: &str =
    "candidate_verified_without_durable_policy_ledger_or_write_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_audit_policy_ledger_candidate.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_ID: &str =
    "hello_rollback_durable_audit_policy_ledger_candidate.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_STATUS: &str =
    "candidate_ready_read_only_not_write_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_CANDIDATE_REASON: &str =
    "acceptance_gate_bound_without_durable_write_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_audit_policy_ledger_aware_acceptance_result.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_ID: &str =
    "hello_rollback_durable_audit_policy_ledger_aware_acceptance_result.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_STATUS: &str =
    "denied_missing_write_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_LEDGER_AWARE_ACCEPTANCE_RESULT_REASON: &str =
    "read_only_ledger_candidate_without_durable_write_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_audit_policy_write_authority_availability.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_ID: &str =
    "hello_rollback_durable_audit_policy_write_authority_availability.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_STATUS: &str =
    "denied_missing_durable_write_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_WRITE_AUTHORITY_AVAILABILITY_REASON: &str =
    "ledger_aware_acceptance_result_without_durable_write_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_policy_ledger_availability.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_ID: &str =
    "hello_rollback_durable_policy_ledger_availability.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_STATUS: &str =
    "denied_missing_durable_policy_ledger";
pub(crate) const HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_REASON: &str =
    "write_authority_availability_without_durable_policy_ledger";
pub(crate) const HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_policy_ledger_availability_dry_run.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_ID: &str =
    "hello_rollback_durable_policy_ledger_availability_dry_run.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_STATUS: &str =
    "test_media_policy_ledger_availability_verified_not_durable_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_POLICY_LEDGER_AVAILABILITY_DRY_RUN_REASON: &str =
    "policy_ledger_availability_bound_to_transaction_denial_current_boot";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_audit_policy_availability.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_ID: &str =
    "hello_rollback_durable_audit_policy_availability.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_STATUS: &str =
    "denied_missing_durable_audit_policy";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_REASON: &str =
    "durable_policy_ledger_availability_without_durable_audit_policy";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_audit_policy_availability_dry_run.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_ID: &str =
    "hello_rollback_durable_audit_policy_availability_dry_run.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_STATUS: &str =
    "test_media_audit_policy_availability_verified_not_durable_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_AVAILABILITY_DRY_RUN_REASON: &str =
    "audit_policy_availability_bound_to_transaction_denial_current_boot";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_append_authority_availability.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_ID: &str =
    "hello_rollback_durable_append_authority_availability.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_STATUS: &str =
    "denied_missing_durable_append_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_REASON: &str =
    "durable_audit_policy_availability_without_durable_append_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_append_authority_availability_dry_run.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_ID: &str =
    "hello_rollback_durable_append_authority_availability_dry_run.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_STATUS: &str =
    "test_media_append_authority_availability_verified_not_durable_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_APPEND_AUTHORITY_AVAILABILITY_DRY_RUN_REASON: &str =
    "append_authority_availability_bound_to_transaction_denial_current_boot";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_transaction_append_availability_decision.v0";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_ID: &str =
    "hello_rollback_transaction_append_availability_decision.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_STATUS: &str =
    "denied_missing_rollback_transaction_append_authority";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_APPEND_AVAILABILITY_DECISION_REASON: &str =
    "durable_append_authority_availability_without_transaction_append_authority";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_transaction_append_authority_denial_gate.v0";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_ID: &str =
    "hello_rollback_transaction_append_authority_denial_gate.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_STATUS: &str =
    "denied_missing_rollback_transaction_append_authority";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_APPEND_AUTHORITY_DENIAL_GATE_REASON: &str =
    "transaction_append_availability_decision_without_append_authority";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_transaction_append_dry_run.v0";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_ID: &str =
    "hello_rollback_transaction_append_dry_run.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_STATUS: &str =
    "blocked_by_transaction_append_authority_denial_gate";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_APPEND_DRY_RUN_REASON: &str =
    "missing_transaction_append_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_durable_policy_write_authority_decision.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_ID: &str =
    "hello_rollback_durable_policy_write_authority_decision.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_STATUS: &str =
    "denied_missing_durable_policy_write_authority";
pub(crate) const HELLO_ROLLBACK_DURABLE_POLICY_WRITE_AUTHORITY_DECISION_REASON: &str =
    "transaction_append_dry_run_verified_without_durable_policy_write_authority";
pub(crate) const HELLO_ROLLBACK_MEDIA_WRITE_AUTHORITY_MISSING_REASON: &str =
    "media_write_authority_missing";
pub(crate) const HELLO_ROLLBACK_TEST_MEDIA_WRITE_AUTHORITY_REASON: &str =
    "test_infrastructure_media_write_authority_verified_current_boot";
pub(crate) const HELLO_ROLLBACK_DURABLE_AUDIT_POLICY_MISSING_REASON: &str =
    "durable_audit_policy_missing";
pub(crate) const HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_target_region_write_readback_dry_run.v0";
pub(crate) const HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_ID: &str =
    "hello_rollback_target_region_write_readback_dry_run.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_STATUS: &str =
    "target_region_sector_image_readback_verified_not_durable_authority";
pub(crate) const HELLO_ROLLBACK_TARGET_REGION_WRITE_READBACK_DRY_RUN_REASON: &str =
    "planned_sector_image_written_and_read_back_on_target_region_current_boot";
pub(crate) const HELLO_ROLLBACK_TARGET_REGION_MATERIALIZER_MISSING_REASON: &str =
    "recovery_rollback_materialize_dry_run_missing";
pub(crate) const HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_SCHEMA: &str =
    "raios.ram_only_hello_service_rollback_target_region_sector_inspection.v0";
pub(crate) const HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_ID: &str =
    "hello_rollback_target_region_sector_inspection.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_STATUS: &str =
    "target_region_sector_read_parsed_not_durable_authority";
pub(crate) const HELLO_ROLLBACK_TARGET_REGION_SECTOR_INSPECTION_REASON: &str =
    "target_region_sector_read_and_append_offsets_verified_current_boot";
pub(crate) const HELLO_ROLLBACK_TARGET_REGION_INSPECTION_MISSING_REASON: &str =
    "recovery_rollback_inspect_missing";
pub(crate) const HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_SCHEMA: &str =
    "raios.recovery_rollback_inspect_source_reference.v0";
pub(crate) const HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_ID: &str =
    "recovery_rollback_inspect_source.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_STATUS: &str =
    "retained_sector_inspection_source_reference";
pub(crate) const HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_REASON: &str =
    "retained_recovery_rollback_inspect_source_matches_sector_inspection";
pub(crate) const HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_MISSING_REASON: &str =
    "recovery_rollback_inspect_source_missing";
pub(crate) const HELLO_RECOVERY_ROLLBACK_INSPECT_SCHEMA: &str =
    "raios.recovery_rollback_inspect.v0";
pub(crate) const HELLO_RECOVERY_ROLLBACK_INSPECT_ID: &str =
    "recovery_rollback_inspect.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_RECOVERY_ROLLBACK_INSPECT_STATUS: &str =
    "target_region_sector_inspected_current_boot";
pub(crate) const HELLO_RECOVERY_ROLLBACK_MATERIALIZE_DRY_RUN_SCHEMA: &str =
    "raios.recovery_rollback_materialize_dry_run.v0";
pub(crate) const HELLO_RECOVERY_ROLLBACK_MATERIALIZE_DRY_RUN_ID: &str =
    "recovery_rollback_materialize_dry_run.current_boot.svc.demo.hello.v0";
pub(crate) const HELLO_RECOVERY_ROLLBACK_MATERIALIZE_DRY_RUN_STATUS: &str =
    "target_region_sector_materialized_current_boot";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_FOUNDATION_SCHEMA: &str =
    "raios.module_audit_rollback_append_contract.v0";
pub(crate) const HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_FOUNDATION_OWNER: &str =
    rollback_storage_layout::AUDIT_ROLLBACK_TRANSACTION_WRITER_OWNER;
pub(crate) const HELLO_ROLLBACK_TRANSACTION_WRITER_STORAGE_TARGET_ID: &str =
    rollback_storage_layout::AUDIT_ROLLBACK_ROLLBACK_APPEND_TARGET_ID;
pub(crate) const HELLO_ROLLBACK_APPLY_CAPABILITY: &str =
    HELLO_SERVICE_DESCRIPTOR.rollback_apply_capability;
pub(crate) const ARTIFACT_LOAD_PLAN_PREFLIGHT_SELFTEST_SCHEMA: &str =
    "raios.current_boot_artifact_load_plan_preflight_selftest.v0";
pub(crate) const ARTIFACT_LOAD_PLAN_PREFLIGHT_SELFTEST_ID: &str =
    "artifact_load_plan_preflight_selftest.current_boot.svc.demo.hello.v0";
pub(crate) const ARTIFACT_LOAD_PLAN_PREFLIGHT_SELFTEST_CASES: usize = 8;
pub(crate) const HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_SELFTEST_SCHEMA: &str =
    "raios.recovery_rollback_inspect_source_reference_selftest.v0";
pub(crate) const HELLO_RECOVERY_ROLLBACK_INSPECT_SOURCE_REFERENCE_SELFTEST_ID: &str =
    "recovery_rollback_inspect_source_reference_selftest.current_boot.svc.demo.hello.v0";
