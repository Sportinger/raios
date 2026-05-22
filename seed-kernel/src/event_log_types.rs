use core::str;

pub const EVENT_CAPACITY: usize = 256;
pub const DEFAULT_EVENT_LIMIT: usize = 32;
pub use crate::module_evidence::MODULE_SERVICE_SLOT_ID_MAX;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EventId {
    pub(crate) sequence: u64,
}

impl EventId {
    pub fn sequence(self) -> u64 {
        self.sequence
    }

    pub fn from_sequence(sequence: u64) -> Option<Self> {
        if sequence == 0 {
            None
        } else {
            Some(Self { sequence })
        }
    }
}

#[derive(Clone, Copy)]
pub struct ProviderContextHashes {
    pub projected_packet_hash: [u8; 32],
    pub exported_field_list_hash: [u8; 32],
    pub omitted_field_list_hash: [u8; 32],
}

#[derive(Clone, Copy)]
pub struct ProviderRequestEnvelopeBinding {
    pub request_id: u32,
    pub request_body_hash: [u8; 32],
    pub envelope_hash: [u8; 32],
    pub provider_trust_state: &'static str,
    pub provider_trust_positive: bool,
    pub development_tls_bypass: bool,
}

#[derive(Clone, Copy)]
pub struct ProviderRequestBinding {
    pub request_id: u32,
    pub request_envelope_event_id: EventId,
    pub request_body_hash: [u8; 32],
    pub request_envelope_hash: [u8; 32],
    pub request_binding_hash: [u8; 32],
    pub context: ProviderContextHashes,
    pub provider_trust_state: &'static str,
    pub development_tls_bypass: bool,
}

#[derive(Clone, Copy)]
pub struct ProviderExportAuditBinding {
    pub request_id: u32,
    pub request_envelope_event_id: EventId,
    pub request_binding_event_id: EventId,
    pub request_body_hash: [u8; 32],
    pub request_envelope_hash: [u8; 32],
    pub request_binding_hash: [u8; 32],
    pub export_audit_binding_hash: [u8; 32],
    pub context: ProviderContextHashes,
    pub provider_trust_state: &'static str,
    pub context_attached_to_provider_body: bool,
}

#[derive(Clone, Copy)]
pub struct ProviderBindingConsumption {
    pub request_id: u32,
    pub request_envelope_event_id: EventId,
    pub request_binding_event_id: EventId,
    pub export_audit_binding_event_id: EventId,
    pub request_binding_hash: [u8; 32],
    pub export_audit_binding_hash: [u8; 32],
    pub context: ProviderContextHashes,
}

#[derive(Clone, Copy)]
pub struct ProviderContextInjectionAuthorization {
    pub request_id: u32,
    pub request_envelope_event_id: EventId,
    pub request_binding_event_id: EventId,
    pub export_audit_binding_event_id: EventId,
    pub binding_consumption_event_id: EventId,
    pub request_body_hash: [u8; 32],
    pub request_envelope_hash: [u8; 32],
    pub request_binding_hash: [u8; 32],
    pub export_audit_binding_hash: [u8; 32],
    pub context: ProviderContextHashes,
    pub provider_trust_state: &'static str,
    pub final_authorization_hash: [u8; 32],
    pub context_attached_to_provider_body: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModuleManifestReference {
    pub manifest_reference_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModuleCandidateArtifactReference {
    pub artifact_reference_hash: [u8; 32],
    pub retained_manifest_reference_event_id: EventId,
    pub retained_reference_event_id: EventId,
    pub manifest_reference_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub computed_grant_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModuleVmTestReportReference {
    pub report_reference_hash: [u8; 32],
    pub retained_manifest_reference_event_id: EventId,
    pub retained_artifact_reference_event_id: EventId,
    pub retained_reference_event_id: EventId,
    pub manifest_reference_hash: [u8; 32],
    pub artifact_reference_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub computed_grant_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModuleLocalAttestationReference {
    pub attestation_reference_hash: [u8; 32],
    pub retained_manifest_reference_event_id: EventId,
    pub retained_artifact_reference_event_id: EventId,
    pub retained_vm_report_reference_event_id: EventId,
    pub retained_reference_event_id: EventId,
    pub manifest_reference_hash: [u8; 32],
    pub artifact_reference_hash: [u8; 32],
    pub vm_report_reference_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub computed_grant_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModuleLocalApprovalReference {
    pub approval_reference_hash: [u8; 32],
    pub retained_manifest_reference_event_id: EventId,
    pub retained_artifact_reference_event_id: EventId,
    pub retained_vm_report_reference_event_id: EventId,
    pub retained_local_attestation_reference_event_id: EventId,
    pub retained_reference_event_id: EventId,
    pub manifest_reference_hash: [u8; 32],
    pub artifact_reference_hash: [u8; 32],
    pub vm_report_reference_hash: [u8; 32],
    pub local_attestation_reference_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub computed_grant_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
    pub local_approval_hash: [u8; 32],
}

#[derive(Clone, Copy)]
pub struct ModuleComputedGrantReference {
    pub computed_grant_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
}

#[derive(Clone, Copy)]
pub struct ModuleServiceSlotId {
    bytes: [u8; MODULE_SERVICE_SLOT_ID_MAX],
    len: usize,
}

impl ModuleServiceSlotId {
    pub fn new(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        if bytes.is_empty() || bytes.len() > MODULE_SERVICE_SLOT_ID_MAX {
            return None;
        }
        let mut out = Self {
            bytes: [0; MODULE_SERVICE_SLOT_ID_MAX],
            len: bytes.len(),
        };
        out.bytes[..bytes.len()].copy_from_slice(bytes);
        Some(out)
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryCommandTargetLocator {
    bytes: [u8; MODULE_SERVICE_SLOT_ID_MAX],
    len: usize,
}

impl RecoveryCommandTargetLocator {
    pub fn new(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        if bytes.is_empty() || bytes.len() > MODULE_SERVICE_SLOT_ID_MAX {
            return None;
        }
        let mut out = Self {
            bytes: [0; MODULE_SERVICE_SLOT_ID_MAX],
            len: bytes.len(),
        };
        out.bytes[..bytes.len()].copy_from_slice(bytes);
        Some(out)
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }
}

#[derive(Clone, Copy)]
pub struct ModuleAuditRollbackReference {
    pub audit_record_hash: [u8; 32],
    pub rollback_plan_hash: [u8; 32],
    pub computed_grant_hash: [u8; 32],
    pub manifest_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub vm_report_hash: [u8; 32],
    pub local_attestation_hash: [u8; 32],
    pub local_approval_hash: [u8; 32],
    pub pre_load_service_inventory_hash: [u8; 32],
    pub cleanup_actions_hash: [u8; 32],
    pub denial_event_id: EventId,
    pub retained_reference_event_id: EventId,
    pub ram_only_service_slot_id: ModuleServiceSlotId,
}

#[derive(Clone, Copy)]
pub struct ModuleServiceSlotReservation {
    pub reservation_hash: [u8; 32],
    pub retained_reference_event_id: EventId,
    pub retained_audit_rollback_reference_event_id: EventId,
    pub computed_grant_hash: [u8; 32],
    pub audit_record_hash: [u8; 32],
    pub rollback_plan_hash: [u8; 32],
    pub pre_load_service_inventory_hash: [u8; 32],
    pub ram_only_service_slot_id: ModuleServiceSlotId,
}

#[derive(Clone, Copy)]
pub struct ModuleLoadGateBinding {
    pub manifest_reference_event_id: Option<EventId>,
    pub manifest_reference: Option<ModuleManifestReference>,
    pub manifest_reference_status: &'static str,
    pub manifest_reference_reason: &'static str,
    pub artifact_reference_event_id: Option<EventId>,
    pub artifact_reference: Option<ModuleCandidateArtifactReference>,
    pub artifact_reference_status: &'static str,
    pub artifact_reference_reason: &'static str,
    pub vm_report_reference_event_id: Option<EventId>,
    pub vm_report_reference: Option<ModuleVmTestReportReference>,
    pub vm_report_reference_status: &'static str,
    pub vm_report_reference_reason: &'static str,
    pub attestation_reference_event_id: Option<EventId>,
    pub attestation_reference: Option<ModuleLocalAttestationReference>,
    pub attestation_reference_status: &'static str,
    pub attestation_reference_reason: &'static str,
    pub approval_reference_event_id: Option<EventId>,
    pub approval_reference: Option<ModuleLocalApprovalReference>,
    pub approval_reference_status: &'static str,
    pub approval_reference_reason: &'static str,
    pub retained_reference_event_id: Option<EventId>,
    pub retained_reference: Option<ModuleComputedGrantReference>,
    pub audit_rollback_reference_event_id: Option<EventId>,
    pub audit_rollback_reference: Option<ModuleAuditRollbackReference>,
    pub audit_rollback_reference_status: &'static str,
    pub audit_rollback_reference_reason: &'static str,
    pub service_slot_reservation_event_id: Option<EventId>,
    pub service_slot_reservation: Option<ModuleServiceSlotReservation>,
    pub service_slot_reservation_status: &'static str,
    pub service_slot_reservation_reason: &'static str,
}

#[derive(Clone, Copy)]
pub struct RecoveryArtifactLoadDenialBinding {
    pub recovery_artifact_identity_missing: bool,
    pub recovery_artifact_trust_missing: bool,
    pub recovery_vm_test_missing: bool,
    pub recovery_local_approval_missing: bool,
    pub recovery_loader_missing: bool,
    pub recovery_rollback_evidence_missing: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryArtifactIdentityReference {
    pub identity_reference_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryArtifactTrustReference {
    pub trust_reference_hash: [u8; 32],
    pub retained_identity_reference_event_id: EventId,
    pub identity_reference_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub trust_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryArtifactVmTestReference {
    pub vm_test_reference_hash: [u8; 32],
    pub retained_identity_reference_event_id: EventId,
    pub retained_trust_reference_event_id: EventId,
    pub identity_reference_hash: [u8; 32],
    pub trust_reference_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub trust_hash: [u8; 32],
    pub vm_test_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryArtifactLocalApprovalReference {
    pub local_approval_reference_hash: [u8; 32],
    pub retained_identity_reference_event_id: EventId,
    pub retained_trust_reference_event_id: EventId,
    pub retained_vm_test_reference_event_id: EventId,
    pub identity_reference_hash: [u8; 32],
    pub trust_reference_hash: [u8; 32],
    pub vm_test_reference_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub trust_hash: [u8; 32],
    pub vm_test_hash: [u8; 32],
    pub local_approval_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryArtifactLoaderReference {
    pub loader_reference_hash: [u8; 32],
    pub retained_identity_reference_event_id: EventId,
    pub retained_trust_reference_event_id: EventId,
    pub retained_vm_test_reference_event_id: EventId,
    pub retained_local_approval_reference_event_id: EventId,
    pub identity_reference_hash: [u8; 32],
    pub trust_reference_hash: [u8; 32],
    pub vm_test_reference_hash: [u8; 32],
    pub local_approval_reference_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub trust_hash: [u8; 32],
    pub vm_test_hash: [u8; 32],
    pub local_approval_hash: [u8; 32],
    pub loader_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryArtifactRollbackEvidenceReference {
    pub rollback_evidence_reference_hash: [u8; 32],
    pub retained_identity_reference_event_id: EventId,
    pub retained_trust_reference_event_id: EventId,
    pub retained_vm_test_reference_event_id: EventId,
    pub retained_local_approval_reference_event_id: EventId,
    pub retained_loader_reference_event_id: EventId,
    pub identity_reference_hash: [u8; 32],
    pub trust_reference_hash: [u8; 32],
    pub vm_test_reference_hash: [u8; 32],
    pub local_approval_reference_hash: [u8; 32],
    pub loader_reference_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub trust_hash: [u8; 32],
    pub vm_test_hash: [u8; 32],
    pub local_approval_hash: [u8; 32],
    pub loader_hash: [u8; 32],
    pub rollback_evidence_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryLifelineRequestReference {
    pub lifeline_request_reference_hash: [u8; 32],
    pub retained_identity_reference_event_id: EventId,
    pub retained_trust_reference_event_id: EventId,
    pub retained_vm_test_reference_event_id: EventId,
    pub retained_local_approval_reference_event_id: EventId,
    pub retained_loader_reference_event_id: EventId,
    pub retained_rollback_evidence_reference_event_id: EventId,
    pub identity_reference_hash: [u8; 32],
    pub trust_reference_hash: [u8; 32],
    pub vm_test_reference_hash: [u8; 32],
    pub local_approval_reference_hash: [u8; 32],
    pub loader_reference_hash: [u8; 32],
    pub rollback_evidence_reference_hash: [u8; 32],
    pub artifact_hash: [u8; 32],
    pub trust_hash: [u8; 32],
    pub vm_test_hash: [u8; 32],
    pub local_approval_hash: [u8; 32],
    pub loader_hash: [u8; 32],
    pub rollback_evidence_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryLifelineCommandEnvelopeReference {
    pub command_envelope_reference_hash: [u8; 32],
    pub retained_lifeline_request_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub required_capability: &'static str,
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_admission_boundary_id: &'static str,
    pub lifeline_request_reference_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryLifelineCommandBodyCanonicalizationReference {
    pub command_body_canonicalization_hash: [u8; 32],
    pub retained_command_envelope_reference_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_envelope_reference_hash: [u8; 32],
    pub command_dispatch_boundary_id: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryLifelineCommandHandlerBindingReference {
    pub handler_binding_hash: [u8; 32],
    pub retained_command_body_canonicalization_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_envelope_reference_hash: [u8; 32],
    pub command_body_canonicalization_hash: [u8; 32],
    pub command_dispatch_boundary_id: &'static str,
    pub handler_id: &'static str,
    pub handler_input_binding_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryLifelineStatusReadHandlerReference {
    pub status_read_handler_hash: [u8; 32],
    pub retained_command_handler_binding_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_envelope_reference_hash: [u8; 32],
    pub command_body_canonicalization_hash: [u8; 32],
    pub handler_binding_hash: [u8; 32],
    pub command_dispatch_boundary_id: &'static str,
    pub status_handler_id: &'static str,
    pub status_read_projection_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryRollbackPreviewAuthorizationReference {
    pub rollback_preview_authorization_hash: [u8; 32],
    pub retained_status_read_handler_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_envelope_reference_hash: [u8; 32],
    pub command_body_canonicalization_hash: [u8; 32],
    pub handler_binding_hash: [u8; 32],
    pub status_read_handler_hash: [u8; 32],
    pub command_dispatch_boundary_id: &'static str,
    pub rollback_preview_authorization_id: &'static str,
    pub rollback_preview_projection_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryRollbackApplyAuthorizationReference {
    pub rollback_apply_authorization_hash: [u8; 32],
    pub retained_rollback_preview_authorization_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_envelope_reference_hash: [u8; 32],
    pub command_body_canonicalization_hash: [u8; 32],
    pub handler_binding_hash: [u8; 32],
    pub status_read_handler_hash: [u8; 32],
    pub rollback_preview_authorization_hash: [u8; 32],
    pub command_dispatch_boundary_id: &'static str,
    pub rollback_apply_authorization_id: &'static str,
    pub rollback_apply_projection_hash: [u8; 32],
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleManifestReferenceGateCheck {
    pub(crate) event_id: Option<EventId>,
    pub(crate) reference: Option<ModuleManifestReference>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleCandidateArtifactReferenceGateCheck {
    pub(crate) event_id: Option<EventId>,
    pub(crate) reference: Option<ModuleCandidateArtifactReference>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleVmTestReportReferenceGateCheck {
    pub(crate) event_id: Option<EventId>,
    pub(crate) reference: Option<ModuleVmTestReportReference>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLocalAttestationReferenceGateCheck {
    pub(crate) event_id: Option<EventId>,
    pub(crate) reference: Option<ModuleLocalAttestationReference>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleLocalApprovalReferenceGateCheck {
    pub(crate) event_id: Option<EventId>,
    pub(crate) reference: Option<ModuleLocalApprovalReference>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleAuditRollbackReferenceGateCheck {
    pub(crate) event_id: Option<EventId>,
    pub(crate) reference: Option<ModuleAuditRollbackReference>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ModuleServiceSlotReservationGateCheck {
    pub(crate) event_id: Option<EventId>,
    pub(crate) reservation: Option<ModuleServiceSlotReservation>,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct ConsumedProviderBinding {
    pub(crate) request_binding_event_id: EventId,
    pub(crate) export_audit_binding_event_id: EventId,
}

#[derive(Clone, Copy)]
pub struct ProviderBindingGateCheck {
    pub status: &'static str,
    pub reason: &'static str,
    pub request_binding_event_id: Option<EventId>,
    pub export_audit_binding_event_id: Option<EventId>,
    pub request_envelope_event_id: Option<EventId>,
    pub request_binding: Option<ProviderRequestBinding>,
    pub export_audit_binding: Option<ProviderExportAuditBinding>,
    pub consumed: bool,
    pub retained: bool,
}

#[derive(Clone, Copy)]
pub struct ProviderContextInjectionGateCheck {
    pub status: &'static str,
    pub reason: &'static str,
    pub authorization_event_id: Option<EventId>,
    pub binding_consumption_event_id: Option<EventId>,
    pub retained: bool,
    pub can_attach_context: bool,
    pub satisfies_current_boot_export_gate: bool,
}

pub const PROVIDER_BINDING_GATE_SELFTEST_CASES: usize = 16;
pub const PROVIDER_CONTEXT_INJECTION_GATE_SELFTEST_CASES: usize = 7;

#[derive(Clone, Copy)]
pub struct ProviderBindingGateSelfTestCase {
    pub name: &'static str,
    pub expected_status: &'static str,
    pub expected_reason: &'static str,
    pub actual_status: &'static str,
    pub actual_reason: &'static str,
    pub passed: bool,
}

#[derive(Clone, Copy)]
pub struct ProviderContextInjectionGateSelfTestCase {
    pub name: &'static str,
    pub expected_status: &'static str,
    pub expected_reason: &'static str,
    pub actual_status: &'static str,
    pub actual_reason: &'static str,
    pub passed: bool,
}

#[derive(Clone, Copy)]
pub enum EventBindings {
    None,
    ProviderRequestEnvelope(ProviderRequestEnvelopeBinding),
    ProviderRequestBound(ProviderRequestBinding),
    ProviderExportAuditBound(ProviderExportAuditBinding),
    ProviderBindingConsumption(ProviderBindingConsumption),
    ProviderContextInjectionAuthorization(ProviderContextInjectionAuthorization),
    ProviderRequestBindingDenied(ProviderContextHashes),
    ProviderExportDenialAudit(ProviderContextHashes),
    ModuleManifestReference(ModuleManifestReference),
    ModuleCandidateArtifactReference(ModuleCandidateArtifactReference),
    ModuleVmTestReportReference(ModuleVmTestReportReference),
    ModuleLocalAttestationReference(ModuleLocalAttestationReference),
    ModuleLocalApprovalReference(ModuleLocalApprovalReference),
    ModuleComputedGrantReference(ModuleComputedGrantReference),
    ModuleAuditRollbackReference(ModuleAuditRollbackReference),
    ModuleServiceSlotReservation(ModuleServiceSlotReservation),
    ModuleLoadGate(ModuleLoadGateBinding),
    RecoveryArtifactLoadDenied(RecoveryArtifactLoadDenialBinding),
    RecoveryArtifactIdentityReference(RecoveryArtifactIdentityReference),
    RecoveryArtifactTrustReference(RecoveryArtifactTrustReference),
    RecoveryArtifactVmTestReference(RecoveryArtifactVmTestReference),
    RecoveryArtifactLocalApprovalReference(RecoveryArtifactLocalApprovalReference),
    RecoveryArtifactLoaderReference(RecoveryArtifactLoaderReference),
    RecoveryArtifactRollbackEvidenceReference(RecoveryArtifactRollbackEvidenceReference),
    RecoveryLifelineRequestReference(RecoveryLifelineRequestReference),
    RecoveryLifelineCommandEnvelopeReference(RecoveryLifelineCommandEnvelopeReference),
    RecoveryLifelineCommandBodyCanonicalizationReference(
        RecoveryLifelineCommandBodyCanonicalizationReference,
    ),
    RecoveryLifelineCommandHandlerBindingReference(RecoveryLifelineCommandHandlerBindingReference),
    RecoveryLifelineStatusReadHandlerReference(RecoveryLifelineStatusReadHandlerReference),
    RecoveryRollbackPreviewAuthorizationReference(RecoveryRollbackPreviewAuthorizationReference),
    RecoveryRollbackApplyAuthorizationReference(RecoveryRollbackApplyAuthorizationReference),
}

#[derive(Clone, Copy)]
pub struct Event {
    pub sequence: u64,
    pub kind: &'static str,
    pub source_method: &'static str,
    pub source_transport: &'static str,
    pub classification: &'static str,
    pub outcome: &'static str,
    pub requested_capability: &'static str,
    pub risk: &'static str,
    pub subject: &'static str,
    pub resource: &'static str,
    pub reason: &'static str,
    pub evidence: &'static [&'static str],
    pub bindings: EventBindings,
}

#[derive(Clone, Copy)]
pub struct EventSnapshot {
    pub events: [Option<Event>; EVENT_CAPACITY],
    pub len: usize,
    pub limit: usize,
    pub capacity: usize,
    pub total_count: u64,
    pub dropped_before_sequence: u64,
}
