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
pub(crate) const MODULE_GRANT_SELFTEST_CASES: usize = 5;
pub(crate) const MODULE_AUDIT_ROLLBACK_SELFTEST_CASES: usize = 10;
pub(crate) const MODULE_SERVICE_SLOT_SELFTEST_CASES: usize = 5;
pub(crate) const MODULE_LOAD_GATE_MANIFEST_SELFTEST_CASES: usize = 7;
pub(crate) const MODULE_LOAD_GATE_ARTIFACT_SELFTEST_CASES: usize = 9;
pub(crate) const MODULE_LOAD_GATE_VM_REPORT_SELFTEST_CASES: usize = 11;
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
pub(crate) const MODULE_AUDIT_TEST_DENIAL_EVENT_ID: &str = "event.current_boot.00000031";
pub(crate) const MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID: &str =
    "event.current_boot.00000027";
pub(crate) const MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID: &str = "ram_only:svc.test.0001";
pub(crate) const MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID: &str =
    "event.current_boot.00000033";
