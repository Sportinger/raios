//! Neutral module-reference records shared by the host and kernel adapters.
//!
//! These records carry evidence only.  They do not load modules, consume
//! authority, write storage, or inspect kernel state.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleManifestReferenceCheck<'a> {
    pub has_reference: bool,
    pub arity_valid: bool,
    pub scope: &'a str,
    pub manifest_reference_hash: Option<[u8; 32]>,
    pub manifest_hash: Option<[u8; 32]>,
    pub expected_manifest_reference_hash: Option<[u8; 32]>,
    pub status: &'static str,
    pub reason: &'static str,
    pub valid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleManifestSelfTestCase {
    pub name: &'static str,
    pub expected_status: &'static str,
    pub expected_reason: &'static str,
    pub actual_status: &'static str,
    pub actual_reason: &'static str,
    pub passed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleArtifactReferenceInput<'a> {
    pub has_reference: bool,
    pub arity_valid: bool,
    pub scope: &'a str,
    pub artifact_reference_hash: Option<[u8; 32]>,
    pub retained_manifest_reference_event_id: Option<u64>,
    pub retained_reference_event_id: Option<u64>,
    pub manifest_reference_hash: Option<[u8; 32]>,
    pub manifest_hash: Option<[u8; 32]>,
    pub computed_grant_hash: Option<[u8; 32]>,
    pub artifact_hash: Option<[u8; 32]>,
    pub vm_report_hash: Option<[u8; 32]>,
    pub local_attestation_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleArtifactReferenceCheck<'a> {
    pub has_reference: bool,
    pub arity_valid: bool,
    pub scope: &'a str,
    pub artifact_reference_hash: Option<[u8; 32]>,
    pub retained_manifest_reference_event_id: Option<u64>,
    pub retained_reference_event_id: Option<u64>,
    pub manifest_reference_hash: Option<[u8; 32]>,
    pub manifest_hash: Option<[u8; 32]>,
    pub computed_grant_hash: Option<[u8; 32]>,
    pub artifact_hash: Option<[u8; 32]>,
    pub vm_report_hash: Option<[u8; 32]>,
    pub local_attestation_hash: Option<[u8; 32]>,
    pub expected_artifact_reference_hash: Option<[u8; 32]>,
    pub expected_computed_grant_hash: Option<[u8; 32]>,
    pub status: &'static str,
    pub reason: &'static str,
    pub valid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleArtifactSelfTestCase {
    pub name: &'static str,
    pub expected_status: &'static str,
    pub expected_reason: &'static str,
    pub actual_status: &'static str,
    pub actual_reason: &'static str,
    pub passed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleVmReportReferenceInput<'a> {
    pub has_reference: bool,
    pub arity_valid: bool,
    pub scope: &'a str,
    pub report_reference_hash: Option<[u8; 32]>,
    pub retained_manifest_reference_event_id: Option<u64>,
    pub retained_artifact_reference_event_id: Option<u64>,
    pub retained_reference_event_id: Option<u64>,
    pub manifest_reference_hash: Option<[u8; 32]>,
    pub artifact_reference_hash: Option<[u8; 32]>,
    pub manifest_hash: Option<[u8; 32]>,
    pub artifact_hash: Option<[u8; 32]>,
    pub computed_grant_hash: Option<[u8; 32]>,
    pub vm_report_hash: Option<[u8; 32]>,
    pub local_attestation_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleVmReportReferenceCheck<'a> {
    pub has_reference: bool,
    pub arity_valid: bool,
    pub scope: &'a str,
    pub report_reference_hash: Option<[u8; 32]>,
    pub retained_manifest_reference_event_id: Option<u64>,
    pub retained_artifact_reference_event_id: Option<u64>,
    pub retained_reference_event_id: Option<u64>,
    pub manifest_reference_hash: Option<[u8; 32]>,
    pub artifact_reference_hash: Option<[u8; 32]>,
    pub manifest_hash: Option<[u8; 32]>,
    pub artifact_hash: Option<[u8; 32]>,
    pub computed_grant_hash: Option<[u8; 32]>,
    pub vm_report_hash: Option<[u8; 32]>,
    pub local_attestation_hash: Option<[u8; 32]>,
    pub expected_report_reference_hash: Option<[u8; 32]>,
    pub expected_computed_grant_hash: Option<[u8; 32]>,
    pub status: &'static str,
    pub reason: &'static str,
    pub valid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleVmReportSelfTestCase {
    pub name: &'static str,
    pub expected_status: &'static str,
    pub expected_reason: &'static str,
    pub actual_status: &'static str,
    pub actual_reason: &'static str,
    pub passed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleLocalAttestationReferenceInput<'a> {
    pub has_reference: bool,
    pub arity_valid: bool,
    pub scope: &'a str,
    pub attestation_reference_hash: Option<[u8; 32]>,
    pub retained_manifest_reference_event_id: Option<u64>,
    pub retained_artifact_reference_event_id: Option<u64>,
    pub retained_vm_report_reference_event_id: Option<u64>,
    pub retained_reference_event_id: Option<u64>,
    pub manifest_reference_hash: Option<[u8; 32]>,
    pub artifact_reference_hash: Option<[u8; 32]>,
    pub vm_report_reference_hash: Option<[u8; 32]>,
    pub manifest_hash: Option<[u8; 32]>,
    pub artifact_hash: Option<[u8; 32]>,
    pub computed_grant_hash: Option<[u8; 32]>,
    pub vm_report_hash: Option<[u8; 32]>,
    pub local_attestation_hash: Option<[u8; 32]>,
    pub promotion_signature_der: Option<&'a [u8]>,
    pub signature_verified: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleLocalAttestationReferenceCheck<'a> {
    pub has_reference: bool,
    pub arity_valid: bool,
    pub scope: &'a str,
    pub attestation_reference_hash: Option<[u8; 32]>,
    pub retained_manifest_reference_event_id: Option<u64>,
    pub retained_artifact_reference_event_id: Option<u64>,
    pub retained_vm_report_reference_event_id: Option<u64>,
    pub retained_reference_event_id: Option<u64>,
    pub manifest_reference_hash: Option<[u8; 32]>,
    pub artifact_reference_hash: Option<[u8; 32]>,
    pub vm_report_reference_hash: Option<[u8; 32]>,
    pub manifest_hash: Option<[u8; 32]>,
    pub artifact_hash: Option<[u8; 32]>,
    pub computed_grant_hash: Option<[u8; 32]>,
    pub vm_report_hash: Option<[u8; 32]>,
    pub local_attestation_hash: Option<[u8; 32]>,
    pub promotion_signature_der: Option<&'a [u8]>,
    pub signature_verified: bool,
    pub expected_attestation_reference_hash: Option<[u8; 32]>,
    pub expected_computed_grant_hash: Option<[u8; 32]>,
    pub status: &'static str,
    pub reason: &'static str,
    pub valid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleLocalAttestationSelfTestCase {
    pub name: &'static str,
    pub expected_status: &'static str,
    pub expected_reason: &'static str,
    pub actual_status: &'static str,
    pub actual_reason: &'static str,
    pub passed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleLocalApprovalReferenceInput<'a> {
    pub has_reference: bool,
    pub arity_valid: bool,
    pub scope: &'a str,
    pub approval_reference_hash: Option<[u8; 32]>,
    pub retained_manifest_reference_event_id: Option<u64>,
    pub retained_artifact_reference_event_id: Option<u64>,
    pub retained_vm_report_reference_event_id: Option<u64>,
    pub retained_local_attestation_reference_event_id: Option<u64>,
    pub retained_reference_event_id: Option<u64>,
    pub manifest_reference_hash: Option<[u8; 32]>,
    pub artifact_reference_hash: Option<[u8; 32]>,
    pub vm_report_reference_hash: Option<[u8; 32]>,
    pub local_attestation_reference_hash: Option<[u8; 32]>,
    pub manifest_hash: Option<[u8; 32]>,
    pub artifact_hash: Option<[u8; 32]>,
    pub computed_grant_hash: Option<[u8; 32]>,
    pub vm_report_hash: Option<[u8; 32]>,
    pub local_attestation_hash: Option<[u8; 32]>,
    pub local_approval_hash: Option<[u8; 32]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleLocalApprovalReferenceCheck<'a> {
    pub has_reference: bool,
    pub arity_valid: bool,
    pub scope: &'a str,
    pub approval_reference_hash: Option<[u8; 32]>,
    pub retained_manifest_reference_event_id: Option<u64>,
    pub retained_artifact_reference_event_id: Option<u64>,
    pub retained_vm_report_reference_event_id: Option<u64>,
    pub retained_local_attestation_reference_event_id: Option<u64>,
    pub retained_reference_event_id: Option<u64>,
    pub manifest_reference_hash: Option<[u8; 32]>,
    pub artifact_reference_hash: Option<[u8; 32]>,
    pub vm_report_reference_hash: Option<[u8; 32]>,
    pub local_attestation_reference_hash: Option<[u8; 32]>,
    pub manifest_hash: Option<[u8; 32]>,
    pub artifact_hash: Option<[u8; 32]>,
    pub computed_grant_hash: Option<[u8; 32]>,
    pub vm_report_hash: Option<[u8; 32]>,
    pub local_attestation_hash: Option<[u8; 32]>,
    pub local_approval_hash: Option<[u8; 32]>,
    pub expected_approval_reference_hash: Option<[u8; 32]>,
    pub expected_computed_grant_hash: Option<[u8; 32]>,
    pub status: &'static str,
    pub reason: &'static str,
    pub valid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleLocalApprovalSelfTestCase {
    pub name: &'static str,
    pub expected_status: &'static str,
    pub expected_reason: &'static str,
    pub actual_status: &'static str,
    pub actual_reason: &'static str,
    pub passed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleGrantReferenceCheck<'a> {
    pub has_reference: bool,
    pub arity_valid: bool,
    pub scope: &'a str,
    pub grant_hash: Option<[u8; 32]>,
    pub manifest_hash: Option<[u8; 32]>,
    pub artifact_hash: Option<[u8; 32]>,
    pub vm_report_hash: Option<[u8; 32]>,
    pub local_attestation_hash: Option<[u8; 32]>,
    pub expected_grant_hash: Option<[u8; 32]>,
    pub status: &'static str,
    pub reason: &'static str,
    pub valid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleGrantSelfTestCase {
    pub name: &'static str,
    pub expected_status: &'static str,
    pub expected_reason: &'static str,
    pub actual_status: &'static str,
    pub actual_reason: &'static str,
    pub passed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleAuditRollbackReferenceInput<'a> {
    pub has_reference: bool,
    pub arity_valid: bool,
    pub scope: &'a str,
    pub audit_schema_ok: bool,
    pub rollback_schema_ok: bool,
    pub audit_record_hash: Option<[u8; 32]>,
    pub rollback_plan_hash: Option<[u8; 32]>,
    pub computed_grant_hash: Option<[u8; 32]>,
    pub manifest_hash: Option<[u8; 32]>,
    pub artifact_hash: Option<[u8; 32]>,
    pub vm_report_hash: Option<[u8; 32]>,
    pub local_attestation_hash: Option<[u8; 32]>,
    pub local_approval_hash: Option<[u8; 32]>,
    pub pre_load_service_inventory_hash: Option<[u8; 32]>,
    pub cleanup_actions_hash: Option<[u8; 32]>,
    pub denial_event_id: Option<u64>,
    pub retained_reference_event_id: Option<u64>,
    pub ram_only_service_slot_id: Option<&'a str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleAuditRollbackReferenceCheck<'a> {
    pub has_reference: bool,
    pub arity_valid: bool,
    pub scope: &'a str,
    pub audit_record_hash: Option<[u8; 32]>,
    pub rollback_plan_hash: Option<[u8; 32]>,
    pub computed_grant_hash: Option<[u8; 32]>,
    pub manifest_hash: Option<[u8; 32]>,
    pub artifact_hash: Option<[u8; 32]>,
    pub vm_report_hash: Option<[u8; 32]>,
    pub local_attestation_hash: Option<[u8; 32]>,
    pub local_approval_hash: Option<[u8; 32]>,
    pub pre_load_service_inventory_hash: Option<[u8; 32]>,
    pub cleanup_actions_hash: Option<[u8; 32]>,
    pub denial_event_id: Option<u64>,
    pub retained_reference_event_id: Option<u64>,
    pub ram_only_service_slot_id: Option<&'a str>,
    pub expected_computed_grant_hash: Option<[u8; 32]>,
    pub expected_rollback_plan_hash: Option<[u8; 32]>,
    pub expected_audit_record_hash: Option<[u8; 32]>,
    pub status: &'static str,
    pub reason: &'static str,
    pub valid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleAuditRollbackSelfTestCase {
    pub name: &'static str,
    pub expected_status: &'static str,
    pub expected_reason: &'static str,
    pub actual_status: &'static str,
    pub actual_reason: &'static str,
    pub passed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleServiceSlotReservationInput<'a> {
    pub has_reference: bool,
    pub arity_valid: bool,
    pub scope: &'a str,
    pub reservation_hash: Option<[u8; 32]>,
    pub retained_reference_event_id: Option<u64>,
    pub retained_audit_rollback_reference_event_id: Option<u64>,
    pub computed_grant_hash: Option<[u8; 32]>,
    pub audit_record_hash: Option<[u8; 32]>,
    pub rollback_plan_hash: Option<[u8; 32]>,
    pub pre_load_service_inventory_hash: Option<[u8; 32]>,
    pub ram_only_service_slot_id: Option<&'a str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleServiceSlotReservationCheck<'a> {
    pub has_reference: bool,
    pub arity_valid: bool,
    pub scope: &'a str,
    pub reservation_hash: Option<[u8; 32]>,
    pub retained_reference_event_id: Option<u64>,
    pub retained_audit_rollback_reference_event_id: Option<u64>,
    pub computed_grant_hash: Option<[u8; 32]>,
    pub audit_record_hash: Option<[u8; 32]>,
    pub rollback_plan_hash: Option<[u8; 32]>,
    pub pre_load_service_inventory_hash: Option<[u8; 32]>,
    pub ram_only_service_slot_id: Option<&'a str>,
    pub expected_reservation_hash: Option<[u8; 32]>,
    pub status: &'static str,
    pub reason: &'static str,
    pub valid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModuleServiceSlotSelfTestCase {
    pub name: &'static str,
    pub expected_status: &'static str,
    pub expected_reason: &'static str,
    pub actual_status: &'static str,
    pub actual_reason: &'static str,
    pub passed: bool,
}

pub const MODULE_LOAD_GATE_MANIFEST_SELFTEST_CASES: usize = 7;
pub const MODULE_LOAD_GATE_ARTIFACT_SELFTEST_CASES: usize = 9;
pub const MODULE_LOAD_GATE_VM_REPORT_SELFTEST_CASES: usize = 11;
pub const MODULE_LOAD_GATE_LOCAL_ATTESTATION_SELFTEST_CASES: usize = 11;
pub const MODULE_LOAD_GATE_LOCAL_APPROVAL_SELFTEST_CASES: usize = 12;
pub const MODULE_LOAD_GATE_RETAINED_SELFTEST_CASES: usize = 7;
pub const MODULE_LOAD_GATE_AUDIT_ROLLBACK_SELFTEST_CASES: usize = 23;
pub const MODULE_LOAD_GATE_SERVICE_SLOT_SELFTEST_CASES: usize = 13;

pub const MODULE_GRANT_TEST_MANIFEST_HASH: [u8; 32] = [0x11; 32];
pub const MODULE_GRANT_TEST_ARTIFACT_HASH: [u8; 32] = [0x22; 32];
pub const MODULE_GRANT_TEST_VM_REPORT_HASH: [u8; 32] = [0x33; 32];
pub const MODULE_GRANT_TEST_ATTESTATION_HASH: [u8; 32] = [0x44; 32];
pub const MODULE_GRANT_MISMATCH_MANIFEST_HASH: [u8; 32] = [0x55; 32];
pub const MODULE_AUDIT_TEST_LOCAL_APPROVAL_HASH: [u8; 32] = [0x66; 32];
pub const MODULE_AUDIT_TEST_PRE_INVENTORY_HASH: [u8; 32] = [0x77; 32];
pub const MODULE_AUDIT_TEST_CLEANUP_HASH: [u8; 32] = [0x88; 32];
pub const MODULE_ARTIFACT_TEST_RETAINED_MANIFEST_REFERENCE_EVENT_ID: &str =
    "event.current_boot.00000026";
pub const MODULE_VM_REPORT_TEST_RETAINED_ARTIFACT_REFERENCE_EVENT_ID: &str =
    "event.current_boot.00000028";
pub const MODULE_LOCAL_ATTESTATION_TEST_RETAINED_VM_REPORT_REFERENCE_EVENT_ID: &str =
    "event.current_boot.00000029";
pub const MODULE_LOCAL_APPROVAL_TEST_RETAINED_ATTESTATION_REFERENCE_EVENT_ID: &str =
    "event.current_boot.00000030";
pub const MODULE_AUDIT_TEST_DENIAL_EVENT_ID: &str = "event.current_boot.00000031";
pub const MODULE_AUDIT_TEST_RETAINED_REFERENCE_EVENT_ID: &str = "event.current_boot.00000027";
pub const MODULE_AUDIT_TEST_RAM_ONLY_SERVICE_SLOT_ID: &str = "ram_only:svc.test.0001";
pub const MODULE_SERVICE_SLOT_TEST_RETAINED_AUDIT_ROLLBACK_EVENT_ID: &str =
    "event.current_boot.00000033";
