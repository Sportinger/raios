use crate::event_log;

#[derive(Clone, Copy)]
pub(crate) struct ModuleManifestReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) expected_manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleManifestSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleArtifactReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleArtifactReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) expected_artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) expected_computed_grant_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleArtifactSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleVmReportReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) report_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_artifact_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleVmReportReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) report_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_artifact_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) expected_report_reference_hash: Option<[u8; 32]>,
    pub(crate) expected_computed_grant_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleVmReportSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLocalAttestationReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) attestation_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_artifact_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_report_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_report_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLocalAttestationReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) attestation_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_artifact_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_report_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_report_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) expected_attestation_reference_hash: Option<[u8; 32]>,
    pub(crate) expected_computed_grant_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleLocalAttestationSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLocalApprovalReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) approval_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_artifact_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_report_reference_event_id: Option<&'a str>,
    pub(crate) retained_local_attestation_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_report_reference_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLocalApprovalReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) approval_reference_hash: Option<[u8; 32]>,
    pub(crate) retained_manifest_reference_event_id: Option<&'a str>,
    pub(crate) retained_artifact_reference_event_id: Option<&'a str>,
    pub(crate) retained_vm_report_reference_event_id: Option<&'a str>,
    pub(crate) retained_local_attestation_reference_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) manifest_reference_hash: Option<[u8; 32]>,
    pub(crate) artifact_reference_hash: Option<[u8; 32]>,
    pub(crate) vm_report_reference_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_reference_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
    pub(crate) expected_approval_reference_hash: Option<[u8; 32]>,
    pub(crate) expected_computed_grant_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleLocalApprovalSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleGrantReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) grant_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) expected_grant_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleGrantSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackReferenceInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) audit_schema_ok: bool,
    pub(crate) rollback_schema_ok: bool,
    pub(crate) audit_record_hash: Option<[u8; 32]>,
    pub(crate) rollback_plan_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
    pub(crate) pre_load_service_inventory_hash: Option<[u8; 32]>,
    pub(crate) cleanup_actions_hash: Option<[u8; 32]>,
    pub(crate) denial_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) ram_only_service_slot_id: Option<&'a str>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackReferenceCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) audit_record_hash: Option<[u8; 32]>,
    pub(crate) rollback_plan_hash: Option<[u8; 32]>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) manifest_hash: Option<[u8; 32]>,
    pub(crate) artifact_hash: Option<[u8; 32]>,
    pub(crate) vm_report_hash: Option<[u8; 32]>,
    pub(crate) local_attestation_hash: Option<[u8; 32]>,
    pub(crate) local_approval_hash: Option<[u8; 32]>,
    pub(crate) pre_load_service_inventory_hash: Option<[u8; 32]>,
    pub(crate) cleanup_actions_hash: Option<[u8; 32]>,
    pub(crate) denial_event_id: Option<&'a str>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) ram_only_service_slot_id: Option<&'a str>,
    pub(crate) expected_computed_grant_hash: Option<[u8; 32]>,
    pub(crate) expected_rollback_plan_hash: Option<[u8; 32]>,
    pub(crate) expected_audit_record_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleAuditRollbackSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotReservationInput<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) reservation_hash: Option<[u8; 32]>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) retained_audit_rollback_reference_event_id: Option<&'a str>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) audit_record_hash: Option<[u8; 32]>,
    pub(crate) rollback_plan_hash: Option<[u8; 32]>,
    pub(crate) pre_load_service_inventory_hash: Option<[u8; 32]>,
    pub(crate) ram_only_service_slot_id: Option<&'a str>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotReservationCheck<'a> {
    pub(crate) has_reference: bool,
    pub(crate) arity_valid: bool,
    pub(crate) scope: &'a str,
    pub(crate) reservation_hash: Option<[u8; 32]>,
    pub(crate) retained_reference_event_id: Option<&'a str>,
    pub(crate) retained_audit_rollback_reference_event_id: Option<&'a str>,
    pub(crate) computed_grant_hash: Option<[u8; 32]>,
    pub(crate) audit_record_hash: Option<[u8; 32]>,
    pub(crate) rollback_plan_hash: Option<[u8; 32]>,
    pub(crate) pre_load_service_inventory_hash: Option<[u8; 32]>,
    pub(crate) ram_only_service_slot_id: Option<&'a str>,
    pub(crate) expected_reservation_hash: Option<[u8; 32]>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) valid: bool,
}

pub(crate) struct ModuleServiceSlotSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAvailabilityFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAvailabilityCandidate {
    pub(crate) durable_audit_ledger: ModuleAuditRollbackAvailabilityFact,
    pub(crate) rollback_store: ModuleAuditRollbackAvailabilityFact,
    pub(crate) durable_write_policy_available: bool,
    pub(crate) rollback_install_policy_available: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAvailabilityEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) durable_audit_ledger_status: &'static str,
    pub(crate) durable_audit_ledger_reason: &'static str,
    pub(crate) rollback_store_status: &'static str,
    pub(crate) rollback_store_reason: &'static str,
    pub(crate) durable_write_policy_available: bool,
    pub(crate) rollback_install_policy_available: bool,
    pub(crate) writes_enabled: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackAvailabilitySelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackWritePolicyFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) binds_retained_evidence: bool,
    pub(crate) binds_availability: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackWritePolicyCandidate {
    pub(crate) durable_write_policy: ModuleAuditRollbackWritePolicyFact,
    pub(crate) rollback_install_policy: ModuleAuditRollbackWritePolicyFact,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackWritePolicyEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) durable_write_policy_status: &'static str,
    pub(crate) durable_write_policy_reason: &'static str,
    pub(crate) rollback_install_policy_status: &'static str,
    pub(crate) rollback_install_policy_reason: &'static str,
    pub(crate) writes_enabled: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackWritePolicySelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackPersistenceDeviceFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) stable_identity: bool,
    pub(crate) partition_inventory_available: bool,
    pub(crate) write_path_available: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackStorageLayoutFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) binds_persistence_device: bool,
    pub(crate) has_audit_ledger_region: bool,
    pub(crate) has_rollback_store_region: bool,
    pub(crate) append_slots_available: bool,
    pub(crate) recovery_region_separated: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackStorageLayoutCandidate {
    pub(crate) persistence_device_inventory: ModuleAuditRollbackPersistenceDeviceFact,
    pub(crate) audit_rollback_storage_layout: ModuleAuditRollbackStorageLayoutFact,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackStorageLayoutEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) persistence_device_status: &'static str,
    pub(crate) persistence_device_reason: &'static str,
    pub(crate) storage_layout_status: &'static str,
    pub(crate) storage_layout_reason: &'static str,
    pub(crate) persistence_device_available: bool,
    pub(crate) storage_layout_available: bool,
    pub(crate) append_engine_available: bool,
    pub(crate) writes_enabled: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackStorageLayoutSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendEngineFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) binds_storage_layout: bool,
    pub(crate) binds_write_policy: bool,
    pub(crate) supports_append_only: bool,
    pub(crate) supports_flush: bool,
    pub(crate) supports_replay: bool,
    pub(crate) recovery_separation_respected: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendEngineCandidate {
    pub(crate) audit_ledger_append_engine: ModuleAuditRollbackAppendEngineFact,
    pub(crate) rollback_store_transaction_engine: ModuleAuditRollbackAppendEngineFact,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendEngineEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) audit_engine_status: &'static str,
    pub(crate) audit_engine_reason: &'static str,
    pub(crate) rollback_engine_status: &'static str,
    pub(crate) rollback_engine_reason: &'static str,
    pub(crate) audit_engine_available: bool,
    pub(crate) rollback_engine_available: bool,
    pub(crate) append_engine_available: bool,
    pub(crate) writes_enabled: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackAppendEngineSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendContractFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) binds_write_policy: bool,
    pub(crate) binds_availability: bool,
    pub(crate) binds_storage_layout_id: bool,
    pub(crate) binds_append_engine_id: bool,
    pub(crate) binds_write_policy_id: bool,
    pub(crate) binds_availability_id: bool,
    pub(crate) binds_envelope_provenance: bool,
    pub(crate) storage_layout_available: bool,
    pub(crate) append_engine_available: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendContractCandidate {
    pub(crate) audit_append_envelope: ModuleAuditRollbackAppendContractFact,
    pub(crate) rollback_transaction_envelope: ModuleAuditRollbackAppendContractFact,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendContractEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) audit_append_status: &'static str,
    pub(crate) audit_append_reason: &'static str,
    pub(crate) rollback_transaction_status: &'static str,
    pub(crate) rollback_transaction_reason: &'static str,
    pub(crate) storage_layout_available: bool,
    pub(crate) append_engine_available: bool,
    pub(crate) writes_enabled: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackAppendContractSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendPayloadHashFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) binds_retained_audit_rollback: bool,
    pub(crate) binds_service_slot_reservation: bool,
    pub(crate) binds_pre_load_write_request: bool,
    pub(crate) binds_append_contract_id: bool,
    pub(crate) binds_target_schema: bool,
    pub(crate) binds_payload_hash: bool,
    pub(crate) binds_payload_provenance: bool,
    pub(crate) retained_audit_rollback_available: bool,
    pub(crate) service_slot_reservation_available: bool,
    pub(crate) append_contract_available: bool,
    pub(crate) retained_audit_rollback_event_id: Option<event_log::EventId>,
    pub(crate) service_slot_reservation_event_id: Option<event_log::EventId>,
    pub(crate) payload_hash: Option<[u8; 32]>,
    pub(crate) source_payload_hash: Option<[u8; 32]>,
    pub(crate) pre_load_service_inventory_hash: Option<[u8; 32]>,
    pub(crate) service_slot_reservation_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendPayloadHashCandidate {
    pub(crate) audit_record_payload_hash: ModuleAuditRollbackAppendPayloadHashFact,
    pub(crate) rollback_transaction_payload_hash: ModuleAuditRollbackAppendPayloadHashFact,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendPayloadHashEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) audit_payload_status: &'static str,
    pub(crate) audit_payload_reason: &'static str,
    pub(crate) rollback_payload_status: &'static str,
    pub(crate) rollback_payload_reason: &'static str,
    pub(crate) retained_evidence_available: bool,
    pub(crate) service_slot_reservation_available: bool,
    pub(crate) append_contract_available: bool,
    pub(crate) payload_hash_available: bool,
    pub(crate) writes_enabled: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackAppendPayloadHashSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendIntentFact {
    pub(crate) present: bool,
    pub(crate) schema_ok: bool,
    pub(crate) scope: &'static str,
    pub(crate) provenance_ok: bool,
    pub(crate) classification: &'static str,
    pub(crate) binds_append_contract: bool,
    pub(crate) binds_append_contract_id: bool,
    pub(crate) binds_append_engine_id: bool,
    pub(crate) binds_storage_layout_id: bool,
    pub(crate) binds_write_policy_id: bool,
    pub(crate) binds_availability_id: bool,
    pub(crate) binds_payload_hash: bool,
    pub(crate) binds_intent_provenance: bool,
    pub(crate) append_contract_available: bool,
    pub(crate) payload_hash_available: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendIntentCandidate {
    pub(crate) audit_record_append_intent: ModuleAuditRollbackAppendIntentFact,
    pub(crate) rollback_transaction_append_intent: ModuleAuditRollbackAppendIntentFact,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackAppendIntentEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) audit_intent_status: &'static str,
    pub(crate) audit_intent_reason: &'static str,
    pub(crate) rollback_intent_status: &'static str,
    pub(crate) rollback_intent_reason: &'static str,
    pub(crate) append_contract_available: bool,
    pub(crate) payload_hash_available: bool,
    pub(crate) append_intent_available: bool,
    pub(crate) writes_enabled: bool,
    pub(crate) installs_rollback_plan: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackAppendIntentSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackWriteBoundaryCandidate {
    pub(crate) scope: &'static str,
    pub(crate) request_schema_ok: bool,
    pub(crate) manifest_status: &'static str,
    pub(crate) manifest_reason: &'static str,
    pub(crate) artifact_status: &'static str,
    pub(crate) artifact_reason: &'static str,
    pub(crate) vm_report_status: &'static str,
    pub(crate) vm_report_reason: &'static str,
    pub(crate) computed_grant_status: &'static str,
    pub(crate) computed_grant_reason: &'static str,
    pub(crate) local_attestation_status: &'static str,
    pub(crate) local_attestation_reason: &'static str,
    pub(crate) local_approval_status: &'static str,
    pub(crate) local_approval_reason: &'static str,
    pub(crate) audit_rollback_status: &'static str,
    pub(crate) audit_rollback_reason: &'static str,
    pub(crate) service_slot_status: &'static str,
    pub(crate) service_slot_reason: &'static str,
    pub(crate) manifest_hash_matches_grant: bool,
    pub(crate) artifact_hash_matches_grant: bool,
    pub(crate) vm_report_hash_matches_grant: bool,
    pub(crate) local_attestation_hash_matches_grant: bool,
    pub(crate) local_approval_hash_matches_audit: bool,
    pub(crate) audit_record_hash_matches_service_slot: bool,
    pub(crate) rollback_plan_hash_matches_service_slot: bool,
    pub(crate) service_slot_binds_audit_rollback: bool,
    pub(crate) durable_audit_ledger_status: &'static str,
    pub(crate) durable_audit_ledger_reason: &'static str,
    pub(crate) rollback_store_status: &'static str,
    pub(crate) rollback_store_reason: &'static str,
    pub(crate) durable_write_policy_status: &'static str,
    pub(crate) durable_write_policy_reason: &'static str,
    pub(crate) rollback_install_policy_status: &'static str,
    pub(crate) rollback_install_policy_reason: &'static str,
    pub(crate) audit_append_status: &'static str,
    pub(crate) audit_append_reason: &'static str,
    pub(crate) rollback_transaction_status: &'static str,
    pub(crate) rollback_transaction_reason: &'static str,
    pub(crate) audit_append_intent_status: &'static str,
    pub(crate) audit_append_intent_reason: &'static str,
    pub(crate) rollback_transaction_append_intent_status: &'static str,
    pub(crate) rollback_transaction_append_intent_reason: &'static str,
    pub(crate) recovery_artifact_loader_requested: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackWriteBoundaryEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) durable_audit_write_state: &'static str,
    pub(crate) durable_audit_write_reason: &'static str,
    pub(crate) rollback_install_state: &'static str,
    pub(crate) rollback_install_reason: &'static str,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleAuditRollbackWriteBoundarySelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateRetainedCheck {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateManifestReferenceCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) event_reference: Option<event_log::ModuleManifestReference>,
    pub(crate) candidate_reference: Option<event_log::ModuleManifestReference>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateManifestEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) module_manifest_state: &'static str,
    pub(crate) accepted_manifest_hash: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoadGateManifestSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_module_manifest_state: &'static str,
    pub(crate) accepted_manifest_hash: bool,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateArtifactReferenceCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) event_reference: Option<event_log::ModuleCandidateArtifactReference>,
    pub(crate) candidate_reference: Option<event_log::ModuleCandidateArtifactReference>,
    pub(crate) manifest_event_id: Option<event_log::EventId>,
    pub(crate) manifest_reference: Option<event_log::ModuleManifestReference>,
    pub(crate) retained_event_id: Option<event_log::EventId>,
    pub(crate) retained_reference: Option<event_log::ModuleComputedGrantReference>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateArtifactEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) candidate_artifact_state: &'static str,
    pub(crate) accepted_artifact_hash: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoadGateArtifactSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_candidate_artifact_state: &'static str,
    pub(crate) accepted_artifact_hash: bool,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateVmReportReferenceCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) event_reference: Option<event_log::ModuleVmTestReportReference>,
    pub(crate) candidate_reference: Option<event_log::ModuleVmTestReportReference>,
    pub(crate) manifest_event_id: Option<event_log::EventId>,
    pub(crate) manifest_reference: Option<event_log::ModuleManifestReference>,
    pub(crate) artifact_event_id: Option<event_log::EventId>,
    pub(crate) artifact_reference: Option<event_log::ModuleCandidateArtifactReference>,
    pub(crate) retained_event_id: Option<event_log::EventId>,
    pub(crate) retained_reference: Option<event_log::ModuleComputedGrantReference>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateVmReportEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) vm_test_report_state: &'static str,
    pub(crate) accepted_vm_report_hash: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoadGateVmReportSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_vm_test_report_state: &'static str,
    pub(crate) accepted_vm_report_hash: bool,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateLocalAttestationReferenceCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) event_reference: Option<event_log::ModuleLocalAttestationReference>,
    pub(crate) candidate_reference: Option<event_log::ModuleLocalAttestationReference>,
    pub(crate) manifest_event_id: Option<event_log::EventId>,
    pub(crate) manifest_reference: Option<event_log::ModuleManifestReference>,
    pub(crate) artifact_event_id: Option<event_log::EventId>,
    pub(crate) artifact_reference: Option<event_log::ModuleCandidateArtifactReference>,
    pub(crate) vm_report_event_id: Option<event_log::EventId>,
    pub(crate) vm_report_reference: Option<event_log::ModuleVmTestReportReference>,
    pub(crate) retained_event_id: Option<event_log::EventId>,
    pub(crate) retained_reference: Option<event_log::ModuleComputedGrantReference>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateLocalAttestationEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) local_attestation_state: &'static str,
    pub(crate) accepted_local_attestation_hash: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoadGateLocalAttestationSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_local_attestation_state: &'static str,
    pub(crate) accepted_local_attestation_hash: bool,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateLocalApprovalReferenceCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) event_reference: Option<event_log::ModuleLocalApprovalReference>,
    pub(crate) candidate_reference: Option<event_log::ModuleLocalApprovalReference>,
    pub(crate) manifest_event_id: Option<event_log::EventId>,
    pub(crate) manifest_reference: Option<event_log::ModuleManifestReference>,
    pub(crate) artifact_event_id: Option<event_log::EventId>,
    pub(crate) artifact_reference: Option<event_log::ModuleCandidateArtifactReference>,
    pub(crate) vm_report_event_id: Option<event_log::EventId>,
    pub(crate) vm_report_reference: Option<event_log::ModuleVmTestReportReference>,
    pub(crate) attestation_event_id: Option<event_log::EventId>,
    pub(crate) attestation_reference: Option<event_log::ModuleLocalAttestationReference>,
    pub(crate) retained_event_id: Option<event_log::EventId>,
    pub(crate) retained_reference: Option<event_log::ModuleComputedGrantReference>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateLocalApprovalEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) local_approval_state: &'static str,
    pub(crate) accepted_local_approval_hash: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoadGateLocalApprovalSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_local_approval_state: &'static str,
    pub(crate) accepted_local_approval_hash: bool,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateRetainedCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) event_reference: Option<event_log::ModuleComputedGrantReference>,
    pub(crate) candidate_reference: Option<event_log::ModuleComputedGrantReference>,
}

pub(crate) struct ModuleLoadGateRetainedSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateAuditRollbackReferenceCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) event_reference: Option<event_log::ModuleAuditRollbackReference>,
    pub(crate) candidate_reference: Option<event_log::ModuleAuditRollbackReference>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateAuditRollbackCandidate {
    pub(crate) retained_reference: bool,
    pub(crate) retained_audit_rollback_reference: ModuleLoadGateAuditRollbackReferenceCandidate,
    pub(crate) durable_audit_record: bool,
    pub(crate) rollback_plan: bool,
    pub(crate) audit_schema_ok: bool,
    pub(crate) rollback_schema_ok: bool,
    pub(crate) audit_binds_retained_grant: bool,
    pub(crate) audit_binds_manifest: bool,
    pub(crate) audit_binds_artifact: bool,
    pub(crate) audit_binds_vm_report: bool,
    pub(crate) audit_binds_local_attestation: bool,
    pub(crate) audit_binds_local_approval: bool,
    pub(crate) audit_binds_rollback_plan: bool,
    pub(crate) rollback_binds_artifact: bool,
    pub(crate) rollback_binds_service_slot: bool,
    pub(crate) ram_only_service_slot_allocated: bool,
    pub(crate) loader_available: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateAuditRollbackEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoadGateAuditRollbackSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) passed: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateServiceSlotReservationCandidate {
    pub(crate) scope: &'static str,
    pub(crate) retained: bool,
    pub(crate) schema_ok: bool,
    pub(crate) grant_event_schema_ok: bool,
    pub(crate) audit_event_schema_ok: bool,
    pub(crate) grant_event_reference: Option<event_log::ModuleComputedGrantReference>,
    pub(crate) audit_event_reference: Option<event_log::ModuleAuditRollbackReference>,
    pub(crate) event_reservation: Option<event_log::ModuleServiceSlotReservation>,
    pub(crate) candidate_reservation: Option<event_log::ModuleServiceSlotReservation>,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateServiceSlotCandidate {
    pub(crate) retained_reference: Option<event_log::ModuleComputedGrantReference>,
    pub(crate) audit_rollback_reference: Option<event_log::ModuleAuditRollbackReference>,
    pub(crate) audit_rollback_valid: bool,
    pub(crate) service_slot_reservation: ModuleLoadGateServiceSlotReservationCandidate,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLoadGateServiceSlotEvaluation {
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
    pub(crate) service_slot_state: &'static str,
    pub(crate) accepted_service_slot_reservation_hash: bool,
    pub(crate) can_load: bool,
    pub(crate) load_attempted: bool,
}

pub(crate) struct ModuleLoadGateServiceSlotSelfTestCase {
    pub(crate) name: &'static str,
    pub(crate) expected_status: &'static str,
    pub(crate) expected_reason: &'static str,
    pub(crate) actual_status: &'static str,
    pub(crate) actual_reason: &'static str,
    pub(crate) actual_service_slot_state: &'static str,
    pub(crate) accepted_service_slot_reservation_hash: bool,
    pub(crate) passed: bool,
}

pub(crate) const MODULE_MANIFEST_SELFTEST_CASES: usize = 5;
pub(crate) const MODULE_ARTIFACT_SELFTEST_CASES: usize = 7;
pub(crate) const MODULE_VM_REPORT_SELFTEST_CASES: usize = 8;
pub(crate) const MODULE_LOCAL_ATTESTATION_SELFTEST_CASES: usize = 9;
pub(crate) const MODULE_LOCAL_APPROVAL_SELFTEST_CASES: usize = 10;
pub(crate) const MODULE_GRANT_SELFTEST_CASES: usize = 5;
pub(crate) const MODULE_AUDIT_ROLLBACK_SELFTEST_CASES: usize = 10;
pub(crate) const MODULE_SERVICE_SLOT_SELFTEST_CASES: usize = 5;
pub(crate) const MODULE_AUDIT_ROLLBACK_AVAILABILITY_SELFTEST_CASES: usize = 8;
pub(crate) const MODULE_AUDIT_ROLLBACK_WRITE_POLICY_SELFTEST_CASES: usize = 12;
pub(crate) const MODULE_AUDIT_ROLLBACK_STORAGE_LAYOUT_SELFTEST_CASES: usize = 15;
pub(crate) const MODULE_AUDIT_ROLLBACK_APPEND_ENGINE_SELFTEST_CASES: usize = 16;
pub(crate) const MODULE_AUDIT_ROLLBACK_APPEND_CONTRACT_SELFTEST_CASES: usize = 24;
pub(crate) const MODULE_AUDIT_ROLLBACK_APPEND_PAYLOAD_HASH_SELFTEST_CASES: usize = 20;
pub(crate) const MODULE_AUDIT_ROLLBACK_APPEND_INTENT_SELFTEST_CASES: usize = 20;
pub(crate) const MODULE_AUDIT_ROLLBACK_WRITE_BOUNDARY_SELFTEST_CASES: usize = 22;
pub(crate) const MODULE_LOAD_GATE_MANIFEST_SELFTEST_CASES: usize = 7;
pub(crate) const MODULE_LOAD_GATE_ARTIFACT_SELFTEST_CASES: usize = 9;
pub(crate) const MODULE_LOAD_GATE_VM_REPORT_SELFTEST_CASES: usize = 11;
pub(crate) const MODULE_LOAD_GATE_LOCAL_ATTESTATION_SELFTEST_CASES: usize = 11;
pub(crate) const MODULE_LOAD_GATE_LOCAL_APPROVAL_SELFTEST_CASES: usize = 12;
pub(crate) const MODULE_LOAD_GATE_RETAINED_SELFTEST_CASES: usize = 7;
pub(crate) const MODULE_LOAD_GATE_AUDIT_ROLLBACK_SELFTEST_CASES: usize = 23;
pub(crate) const MODULE_LOAD_GATE_SERVICE_SLOT_SELFTEST_CASES: usize = 13;
pub(crate) const MODULE_GRANT_TEST_MANIFEST_HASH: [u8; 32] = [0x11; 32];
pub(crate) const MODULE_GRANT_TEST_ARTIFACT_HASH: [u8; 32] = [0x22; 32];
pub(crate) const MODULE_GRANT_TEST_VM_REPORT_HASH: [u8; 32] = [0x33; 32];
pub(crate) const MODULE_GRANT_TEST_ATTESTATION_HASH: [u8; 32] = [0x44; 32];
pub(crate) const MODULE_GRANT_MISMATCH_MANIFEST_HASH: [u8; 32] = [0x55; 32];
pub(crate) const MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH: [u8; 32] = [0x66; 32];
pub(crate) const MODULE_AUDIT_TEST_PRE_INVENTORY_HASH: [u8; 32] = [0x77; 32];
pub(crate) const MODULE_AUDIT_TEST_CLEANUP_HASH: [u8; 32] = [0x88; 32];
pub(crate) const MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID: &str =
    "event.current_boot.00000026";
pub(crate) const MODULE_VM_REPORT_TEST_RETAINED_ARTIFACT_REFERENCE_EVENT_ID: &str =
    "event.current_boot.00000028";
pub(crate) const MODULE_LOCAL_ATTESTATION_TEST_RETAINED_VM_REPORT_REFERENCE_EVENT_ID: &str =
    "event.current_boot.00000029";
pub(crate) const MODULE_LOCAL_APPROVAL_TEST_RETAINED_ATTESTATION_REFERENCE_EVENT_ID: &str =
    "event.current_boot.00000030";
pub(crate) const MODULE_AUDIT_TEST_DENIAL_EVENT_ID: &str = "event.current_boot.00000031";
pub(crate) const MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID: &str =
    "event.current_boot.00000027";
pub(crate) const MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID: &str = "ram_only:svc.test.0001";
pub(crate) const MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID: &str =
    "event.current_boot.00000033";
