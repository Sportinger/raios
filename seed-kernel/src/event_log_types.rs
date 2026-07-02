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

#[derive(Clone, Copy)]
pub struct HelloServiceLifecycleBinding {
    pub descriptor_schema: &'static str,
    pub descriptor_id: &'static str,
    pub descriptor_source_locator: &'static str,
    pub descriptor_source_kind: &'static str,
    pub descriptor_source_hash: [u8; 32],
    pub descriptor_source_envelope_id: Option<&'static str>,
    pub descriptor_source_envelope_hash: Option<[u8; 32]>,
    pub descriptor_source_envelope_payload_hash: Option<[u8; 32]>,
    pub descriptor_source_envelope_trust_scope: Option<&'static str>,
    pub descriptor_source_signature_algorithm: Option<&'static str>,
    pub descriptor_source_signature_public_key_hash: Option<[u8; 32]>,
    pub descriptor_source_signature_hash: Option<[u8; 32]>,
    pub descriptor_source_signature_verified: bool,
    pub artifact_identity_id: &'static str,
    pub artifact_identity_hash: [u8; 32],
    pub artifact_identity_envelope_id: &'static str,
    pub artifact_identity_envelope_hash: [u8; 32],
    pub artifact_identity_envelope_payload_hash: [u8; 32],
    pub artifact_identity_envelope_trust_scope: &'static str,
    pub artifact_identity_signature_algorithm: &'static str,
    pub artifact_identity_signature_public_key_hash: [u8; 32],
    pub artifact_identity_signature_hash: [u8; 32],
    pub artifact_identity_signature_verified: bool,
    pub artifact_identity_validated: bool,
    pub artifact_content_binding_id: &'static str,
    pub artifact_content_binding_hash: [u8; 32],
    pub artifact_content_source_locator: &'static str,
    pub artifact_content_source_hash: [u8; 32],
    pub artifact_content_trust_envelope_id: &'static str,
    pub artifact_content_trust_envelope_hash: [u8; 32],
    pub artifact_content_trust_signature_verified: bool,
    pub artifact_content_validated: bool,
    pub artifact_reference_id: &'static str,
    pub artifact_reference_hash: [u8; 32],
    pub artifact_reference_bytes_hash: [u8; 32],
    pub artifact_reference_content_binding_hash: [u8; 32],
    pub artifact_reference_trust_envelope_id: &'static str,
    pub artifact_reference_trust_envelope_hash: [u8; 32],
    pub artifact_reference_trust_signature_verified: bool,
    pub artifact_reference_validated: bool,
    pub artifact_load_plan_preflight_id: &'static str,
    pub artifact_load_plan_preflight_hash: [u8; 32],
    pub artifact_load_plan_preflight_status: &'static str,
    pub artifact_load_plan_preflight_accepted: bool,
    pub service_slot_intent_id: &'static str,
    pub ram_only_service_slot_id: &'static str,
    pub binds_source_locator: Option<&'static str>,
    pub binds_source_kind: Option<&'static str>,
    pub binds_source_hash: Option<[u8; 32]>,
    pub descriptor_source_validated: bool,
    pub service_inventory_change: &'static str,
    pub persistence: &'static str,
    pub accepts_external_artifact_bytes: bool,
    pub loads_external_artifact: bool,
    pub maps_executable_pages: bool,
    pub writes_persistent_state: bool,
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
pub struct ModuleServiceSlotAllocatorFactSourceEvidence {
    pub schema: &'static str,
    pub fact_schema: &'static str,
    pub fact_id: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub fact_status: &'static str,
    pub fact_reason: &'static str,
    pub fact_present: bool,
    pub fact_scope: &'static str,
    pub fact_schema_ok: bool,
    pub fact_provenance_ok: bool,
    pub fact_classification: &'static str,
    pub retained_service_slot_reservation_present: bool,
    pub allocator_runtime_source_evidence_present: bool,
    pub binds_retained_service_slot_reservation: bool,
    pub binds_allocator_runtime: bool,
    pub retained_service_slot_reservation_event_id: Option<EventId>,
    pub allocator_runtime_source_evidence_event_id: Option<EventId>,
}

#[derive(Clone, Copy)]
pub struct ModuleServiceSlotAllocatorPrerequisiteSourceEvidence {
    pub schema: &'static str,
    pub prerequisite_schema: &'static str,
    pub prerequisite_id: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub prerequisite_status: &'static str,
    pub prerequisite_reason: &'static str,
    pub prerequisite_available: bool,
    pub retained_service_slot_reservation_present: bool,
    pub allocator_runtime_available: bool,
    pub registry_binding_available: bool,
    pub health_state_available: bool,
    pub unload_cleanup_available: bool,
    pub allocator_runtime_source_evidence_event_id: Option<EventId>,
    pub registry_binding_source_evidence_event_id: Option<EventId>,
    pub health_state_source_evidence_event_id: Option<EventId>,
    pub unload_cleanup_source_evidence_event_id: Option<EventId>,
}

#[derive(Clone, Copy)]
pub struct ModuleServiceSlotAllocatorAuthoritySourceEvidence {
    pub schema: &'static str,
    pub authority_schema: &'static str,
    pub authority_id: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub authority_status: &'static str,
    pub authority_reason: &'static str,
    pub authority_present: bool,
    pub authority_scope: &'static str,
    pub authority_schema_ok: bool,
    pub authority_provenance_ok: bool,
    pub authority_classification: &'static str,
    pub retained_service_slot_reservation_present: bool,
    pub allocator_runtime_available: bool,
    pub registry_binding_available: bool,
    pub health_state_available: bool,
    pub unload_cleanup_available: bool,
    pub durable_audit_write_available: bool,
    pub rollback_plan_install_available: bool,
    pub module_loader_available: bool,
    pub source_chain_complete: bool,
    pub allocator_runtime_source_evidence_event_id: Option<EventId>,
    pub registry_binding_source_evidence_event_id: Option<EventId>,
    pub health_state_source_evidence_event_id: Option<EventId>,
    pub unload_cleanup_source_evidence_event_id: Option<EventId>,
    pub durable_audit_source_evidence_event_id: Option<EventId>,
    pub rollback_install_source_evidence_event_id: Option<EventId>,
    pub module_loader_source_evidence_event_id: Option<EventId>,
}

#[derive(Clone, Copy)]
pub struct ModuleServiceSlotAllocationIntentSourceEvidence {
    pub schema: &'static str,
    pub intent_schema: &'static str,
    pub intent_id: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub intent_status: &'static str,
    pub intent_reason: &'static str,
    pub intent_present: bool,
    pub intent_scope: &'static str,
    pub intent_schema_ok: bool,
    pub intent_provenance_ok: bool,
    pub intent_classification: &'static str,
    pub requested_capability: &'static str,
    pub load_mode: &'static str,
    pub target: &'static str,
    pub retained_module_evidence_present: bool,
    pub retained_service_slot_reservation_present: bool,
    pub allocator_authority_present: bool,
    pub source_chain_complete: bool,
    pub manifest_reference_event_id: Option<EventId>,
    pub artifact_reference_event_id: Option<EventId>,
    pub vm_report_reference_event_id: Option<EventId>,
    pub local_attestation_reference_event_id: Option<EventId>,
    pub local_approval_reference_event_id: Option<EventId>,
    pub computed_grant_reference_event_id: Option<EventId>,
    pub audit_rollback_reference_event_id: Option<EventId>,
    pub service_slot_reservation_event_id: Option<EventId>,
    pub allocator_authority_source_evidence_event_id: Option<EventId>,
    pub ram_only_service_slot_id: Option<ModuleServiceSlotId>,
}

#[derive(Clone, Copy)]
pub struct ModuleServiceSlotAuthorityInputSourceEvidence {
    pub schema: &'static str,
    pub input_schema: &'static str,
    pub input_id: &'static str,
    pub input_name: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub input_status: &'static str,
    pub input_reason: &'static str,
    pub input_present: bool,
    pub input_scope: &'static str,
    pub input_schema_ok: bool,
    pub input_provenance_ok: bool,
    pub input_classification: &'static str,
    pub dependency_schema: &'static str,
    pub dependency_source_evidence_event_id: Option<EventId>,
    pub dependency_present: bool,
    pub requested_capability: &'static str,
    pub load_mode: &'static str,
    pub target: &'static str,
    pub retained_module_evidence_present: bool,
    pub retained_service_slot_reservation_present: bool,
    pub allocator_authority_present: bool,
    pub allocation_intent_source_evidence_event_id: Option<EventId>,
    pub source_chain_complete: bool,
    pub service_slot_reservation_event_id: Option<EventId>,
    pub allocator_authority_source_evidence_event_id: Option<EventId>,
    pub ram_only_service_slot_id: Option<ModuleServiceSlotId>,
}

#[derive(Clone, Copy)]
pub struct ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence {
    pub schema: &'static str,
    pub decision_schema: &'static str,
    pub decision_id: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub decision_status: &'static str,
    pub decision_reason: &'static str,
    pub decision_present: bool,
    pub decision_scope: &'static str,
    pub decision_schema_ok: bool,
    pub decision_provenance_ok: bool,
    pub decision_classification: &'static str,
    pub requested_capability: &'static str,
    pub load_mode: &'static str,
    pub target: &'static str,
    pub allocator_authority_present: bool,
    pub allocation_intent_present: bool,
    pub authority_inputs_complete: bool,
    pub source_chain_complete: bool,
    pub allocator_authority_source_evidence_event_id: Option<EventId>,
    pub allocation_intent_source_evidence_event_id: Option<EventId>,
    pub authority_input_source_evidence_event_ids: [Option<EventId>; 5],
    pub authority_input_present: [bool; 5],
    pub retained_module_evidence_present: bool,
    pub retained_service_slot_reservation_present: bool,
    pub service_slot_reservation_event_id: Option<EventId>,
    pub ram_only_service_slot_id: Option<ModuleServiceSlotId>,
}

#[derive(Clone, Copy)]
pub struct ModuleServiceSlotRegistryWriteCommitGateSourceEvidence {
    pub schema: &'static str,
    pub gate_schema: &'static str,
    pub gate_id: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub gate_status: &'static str,
    pub gate_reason: &'static str,
    pub gate_present: bool,
    pub gate_scope: &'static str,
    pub gate_schema_ok: bool,
    pub gate_provenance_ok: bool,
    pub gate_classification: &'static str,
    pub requested_capability: &'static str,
    pub load_mode: &'static str,
    pub target: &'static str,
    pub authority_decision_present: bool,
    pub registry_write_authority_present: bool,
    pub registry_binding_available: bool,
    pub durable_audit_write_available: bool,
    pub rollback_plan_install_available: bool,
    pub retained_service_slot_reservation_present: bool,
    pub source_chain_complete: bool,
    pub authority_decision_source_evidence_event_id: Option<EventId>,
    pub registry_write_authority_source_evidence_event_id: Option<EventId>,
    pub registry_binding_source_evidence_event_id: Option<EventId>,
    pub durable_audit_source_evidence_event_id: Option<EventId>,
    pub rollback_install_source_evidence_event_id: Option<EventId>,
    pub service_slot_reservation_event_id: Option<EventId>,
    pub ram_only_service_slot_id: Option<ModuleServiceSlotId>,
    pub authorizes_registry_write: bool,
    pub mutates_service_registry: bool,
    pub writes_durable_audit_state: bool,
    pub installs_rollback_state: bool,
    pub allocates_service_slot: bool,
    pub loads_artifact: bool,
}

#[derive(Clone, Copy)]
pub struct ModuleLoaderIdentitySourceEvidence {
    pub schema: &'static str,
    pub fact_schema: &'static str,
    pub fact_id: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub identity_status: &'static str,
    pub identity_reason: &'static str,
    pub identity_present: bool,
    pub identity_scope: &'static str,
    pub identity_schema_ok: bool,
    pub identity_provenance_ok: bool,
    pub identity_classification: &'static str,
    pub retained_module_evidence_present: bool,
    pub service_slot_allocator_readiness_present: bool,
    pub service_slot_allocator_ready: bool,
    pub audit_rollback_write_boundary_present: bool,
    pub binds_retained_module_evidence: bool,
    pub binds_service_slot_allocator: bool,
    pub binds_audit_rollback_write_boundary: bool,
    pub manifest_reference_event_id: Option<EventId>,
    pub artifact_reference_event_id: Option<EventId>,
    pub vm_test_report_reference_event_id: Option<EventId>,
    pub local_attestation_reference_event_id: Option<EventId>,
    pub local_approval_reference_event_id: Option<EventId>,
    pub computed_grant_reference_event_id: Option<EventId>,
    pub audit_rollback_reference_event_id: Option<EventId>,
    pub service_slot_reservation_event_id: Option<EventId>,
}

#[derive(Clone, Copy)]
pub struct ModuleLoaderArtifactHashBindingSourceEvidence {
    pub schema: &'static str,
    pub fact_schema: &'static str,
    pub fact_id: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub artifact_hash_binding_status: &'static str,
    pub artifact_hash_binding_reason: &'static str,
    pub artifact_hash_binding_present: bool,
    pub artifact_hash_binding_scope: &'static str,
    pub artifact_hash_binding_schema_ok: bool,
    pub artifact_hash_binding_provenance_ok: bool,
    pub artifact_hash_binding_classification: &'static str,
    pub retained_module_evidence_present: bool,
    pub service_slot_allocator_readiness_present: bool,
    pub service_slot_allocator_ready: bool,
    pub audit_rollback_write_boundary_present: bool,
    pub loader_identity_present: bool,
    pub binds_retained_module_evidence: bool,
    pub binds_service_slot_allocator: bool,
    pub binds_audit_rollback_write_boundary: bool,
    pub binds_loader_identity: bool,
    pub loader_identity_source_evidence_event_id: Option<EventId>,
}

#[derive(Clone, Copy)]
pub struct ModuleLoaderFactSourceEvidence {
    pub schema: &'static str,
    pub fact_schema: &'static str,
    pub fact_id: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub fact_status: &'static str,
    pub fact_reason: &'static str,
    pub fact_present: bool,
    pub fact_scope: &'static str,
    pub fact_schema_ok: bool,
    pub fact_provenance_ok: bool,
    pub fact_classification: &'static str,
    pub retained_module_evidence_present: bool,
    pub service_slot_allocator_readiness_present: bool,
    pub service_slot_allocator_ready: bool,
    pub audit_rollback_write_boundary_present: bool,
    pub dependency_present: bool,
    pub dependency_gate: &'static str,
    pub dependency_schema: &'static str,
    pub dependency_method: &'static str,
    pub dependency_source_evidence_event_id: Option<EventId>,
    pub binds_retained_module_evidence: bool,
    pub binds_service_slot_allocator: bool,
    pub binds_audit_rollback_write_boundary: bool,
    pub binds_dependency: bool,
}

#[derive(Clone, Copy)]
pub struct ModuleLoaderRuntimeExecutionCommitGateSourceEvidence {
    pub schema: &'static str,
    pub gate_schema: &'static str,
    pub gate_id: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub gate_status: &'static str,
    pub gate_reason: &'static str,
    pub gate_present: bool,
    pub gate_scope: &'static str,
    pub gate_schema_ok: bool,
    pub gate_provenance_ok: bool,
    pub gate_classification: &'static str,
    pub requested_capability: &'static str,
    pub load_mode: &'static str,
    pub target: &'static str,
    pub authority_decision_present: bool,
    pub loader_runtime_contract_present: bool,
    pub loader_runtime_source_evidence_complete: bool,
    pub service_slot_binding_source_evidence_present: bool,
    pub service_slot_binding_fact_present: bool,
    pub audit_rollback_write_boundary_source_evidence_present: bool,
    pub audit_rollback_write_boundary_fact_present: bool,
    pub retained_service_slot_reservation_present: bool,
    pub source_chain_complete: bool,
    pub authority_decision_source_evidence_event_id: Option<EventId>,
    pub loader_runtime_contract_source_evidence_event_id: Option<EventId>,
    pub loader_runtime_source_evidence_event_ids: [Option<EventId>; 10],
    pub loader_runtime_source_evidence_present: [bool; 10],
    pub loader_runtime_fact_present: [bool; 10],
    pub service_slot_reservation_event_id: Option<EventId>,
    pub ram_only_service_slot_id: Option<ModuleServiceSlotId>,
    pub accepts_loader_descriptor: bool,
    pub accepts_artifact_bytes: bool,
    pub authorizes_execution: bool,
    pub mutates_service_registry: bool,
    pub writes_durable_audit_state: bool,
    pub installs_rollback_state: bool,
    pub allocates_service_slot: bool,
    pub loads_artifact: bool,
}

#[derive(Clone, Copy)]
pub struct ModuleLoaderDescriptorIntakeBoundarySourceEvidence {
    pub schema: &'static str,
    pub boundary_schema: &'static str,
    pub boundary_id: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub boundary_status: &'static str,
    pub boundary_reason: &'static str,
    pub boundary_present: bool,
    pub boundary_scope: &'static str,
    pub boundary_schema_ok: bool,
    pub boundary_provenance_ok: bool,
    pub boundary_classification: &'static str,
    pub requested_capability: &'static str,
    pub load_mode: &'static str,
    pub target: &'static str,
    pub registry_write_commit_gate_present: bool,
    pub execution_commit_gate_present: bool,
    pub loader_runtime_source_evidence_complete: bool,
    pub retained_module_evidence_present: bool,
    pub retained_service_slot_reservation_present: bool,
    pub source_chain_complete: bool,
    pub registry_write_commit_gate_source_evidence_event_id: Option<EventId>,
    pub execution_commit_gate_source_evidence_event_id: Option<EventId>,
    pub loader_runtime_source_evidence_event_ids: [Option<EventId>; 10],
    pub loader_runtime_source_evidence_present: [bool; 10],
    pub loader_runtime_fact_present: [bool; 10],
    pub manifest_reference_event_id: Option<EventId>,
    pub artifact_reference_event_id: Option<EventId>,
    pub vm_test_report_reference_event_id: Option<EventId>,
    pub local_attestation_reference_event_id: Option<EventId>,
    pub local_approval_reference_event_id: Option<EventId>,
    pub computed_grant_reference_event_id: Option<EventId>,
    pub audit_rollback_reference_event_id: Option<EventId>,
    pub service_slot_reservation_event_id: Option<EventId>,
    pub ram_only_service_slot_id: Option<ModuleServiceSlotId>,
    pub accepts_loader_descriptor: bool,
    pub accepts_descriptor_bytes: bool,
    pub accepts_artifact_bytes: bool,
    pub authorizes_descriptor_intake: bool,
    pub authorizes_execution: bool,
    pub mutates_service_registry: bool,
    pub writes_durable_audit_state: bool,
    pub installs_rollback_state: bool,
    pub allocates_service_slot: bool,
    pub loads_artifact: bool,
}

#[derive(Clone, Copy)]
pub struct ModuleLoaderArtifactByteIntakeBoundarySourceEvidence {
    pub schema: &'static str,
    pub boundary_schema: &'static str,
    pub boundary_id: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub boundary_status: &'static str,
    pub boundary_reason: &'static str,
    pub boundary_present: bool,
    pub boundary_scope: &'static str,
    pub boundary_schema_ok: bool,
    pub boundary_provenance_ok: bool,
    pub boundary_classification: &'static str,
    pub requested_capability: &'static str,
    pub load_mode: &'static str,
    pub target: &'static str,
    pub descriptor_intake_boundary_present: bool,
    pub descriptor_intake_boundary_source_chain_complete: bool,
    pub execution_commit_gate_present: bool,
    pub artifact_hash_binding_present: bool,
    pub retained_artifact_reference_present: bool,
    pub retained_module_evidence_present: bool,
    pub retained_service_slot_reservation_present: bool,
    pub source_chain_complete: bool,
    pub descriptor_intake_boundary_source_evidence_event_id: Option<EventId>,
    pub execution_commit_gate_source_evidence_event_id: Option<EventId>,
    pub artifact_hash_binding_source_evidence_event_id: Option<EventId>,
    pub loader_runtime_source_evidence_event_ids: [Option<EventId>; 10],
    pub loader_runtime_source_evidence_present: [bool; 10],
    pub loader_runtime_fact_present: [bool; 10],
    pub manifest_reference_event_id: Option<EventId>,
    pub artifact_reference_event_id: Option<EventId>,
    pub vm_test_report_reference_event_id: Option<EventId>,
    pub local_attestation_reference_event_id: Option<EventId>,
    pub local_approval_reference_event_id: Option<EventId>,
    pub computed_grant_reference_event_id: Option<EventId>,
    pub audit_rollback_reference_event_id: Option<EventId>,
    pub service_slot_reservation_event_id: Option<EventId>,
    pub ram_only_service_slot_id: Option<ModuleServiceSlotId>,
    pub accepts_loader_descriptor: bool,
    pub accepts_descriptor_bytes: bool,
    pub accepts_artifact_bytes: bool,
    pub authorizes_descriptor_intake: bool,
    pub authorizes_artifact_byte_intake: bool,
    pub authorizes_execution: bool,
    pub mutates_service_registry: bool,
    pub writes_durable_audit_state: bool,
    pub installs_rollback_state: bool,
    pub allocates_service_slot: bool,
    pub loads_artifact: bool,
}

#[derive(Clone, Copy)]
pub struct ModuleLoaderExecutionAuthorizationBoundarySourceEvidence {
    pub schema: &'static str,
    pub boundary_schema: &'static str,
    pub boundary_id: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub boundary_status: &'static str,
    pub boundary_reason: &'static str,
    pub boundary_present: bool,
    pub boundary_scope: &'static str,
    pub boundary_schema_ok: bool,
    pub boundary_provenance_ok: bool,
    pub boundary_classification: &'static str,
    pub requested_capability: &'static str,
    pub load_mode: &'static str,
    pub target: &'static str,
    pub artifact_byte_intake_boundary_present: bool,
    pub artifact_byte_intake_boundary_source_chain_complete: bool,
    pub descriptor_intake_boundary_present: bool,
    pub descriptor_intake_boundary_source_chain_complete: bool,
    pub execution_commit_gate_present: bool,
    pub entrypoint_abi_source_evidence_present: bool,
    pub address_space_source_evidence_present: bool,
    pub memory_map_source_evidence_present: bool,
    pub audit_rollback_write_boundary_source_evidence_present: bool,
    pub retained_module_evidence_present: bool,
    pub retained_service_slot_reservation_present: bool,
    pub source_chain_complete: bool,
    pub artifact_byte_intake_boundary_source_evidence_event_id: Option<EventId>,
    pub descriptor_intake_boundary_source_evidence_event_id: Option<EventId>,
    pub execution_commit_gate_source_evidence_event_id: Option<EventId>,
    pub entrypoint_abi_source_evidence_event_id: Option<EventId>,
    pub address_space_source_evidence_event_id: Option<EventId>,
    pub memory_map_source_evidence_event_id: Option<EventId>,
    pub audit_rollback_write_boundary_source_evidence_event_id: Option<EventId>,
    pub loader_runtime_source_evidence_event_ids: [Option<EventId>; 10],
    pub loader_runtime_source_evidence_present: [bool; 10],
    pub loader_runtime_fact_present: [bool; 10],
    pub manifest_reference_event_id: Option<EventId>,
    pub artifact_reference_event_id: Option<EventId>,
    pub vm_test_report_reference_event_id: Option<EventId>,
    pub local_attestation_reference_event_id: Option<EventId>,
    pub local_approval_reference_event_id: Option<EventId>,
    pub computed_grant_reference_event_id: Option<EventId>,
    pub audit_rollback_reference_event_id: Option<EventId>,
    pub service_slot_reservation_event_id: Option<EventId>,
    pub ram_only_service_slot_id: Option<ModuleServiceSlotId>,
    pub accepts_loader_descriptor: bool,
    pub accepts_descriptor_bytes: bool,
    pub accepts_artifact_bytes: bool,
    pub authorizes_descriptor_intake: bool,
    pub authorizes_artifact_byte_intake: bool,
    pub maps_executable_pages: bool,
    pub jumps_to_entrypoint: bool,
    pub authorizes_execution: bool,
    pub mutates_service_registry: bool,
    pub writes_durable_audit_state: bool,
    pub installs_rollback_state: bool,
    pub allocates_service_slot: bool,
    pub loads_artifact: bool,
}

#[derive(Clone, Copy)]
pub struct ModuleLoaderServiceRegistryMutationBoundarySourceEvidence {
    pub schema: &'static str,
    pub boundary_schema: &'static str,
    pub boundary_id: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub boundary_status: &'static str,
    pub boundary_reason: &'static str,
    pub boundary_present: bool,
    pub boundary_scope: &'static str,
    pub boundary_schema_ok: bool,
    pub boundary_provenance_ok: bool,
    pub boundary_classification: &'static str,
    pub requested_capability: &'static str,
    pub load_mode: &'static str,
    pub target: &'static str,
    pub execution_authorization_boundary_present: bool,
    pub execution_authorization_boundary_source_chain_complete: bool,
    pub registry_write_commit_gate_present: bool,
    pub service_slot_binding_source_evidence_present: bool,
    pub retained_module_evidence_present: bool,
    pub retained_service_slot_reservation_present: bool,
    pub source_chain_complete: bool,
    pub execution_authorization_boundary_source_evidence_event_id: Option<EventId>,
    pub registry_write_commit_gate_source_evidence_event_id: Option<EventId>,
    pub service_slot_binding_source_evidence_event_id: Option<EventId>,
    pub loader_runtime_source_evidence_event_ids: [Option<EventId>; 10],
    pub loader_runtime_source_evidence_present: [bool; 10],
    pub loader_runtime_fact_present: [bool; 10],
    pub manifest_reference_event_id: Option<EventId>,
    pub artifact_reference_event_id: Option<EventId>,
    pub vm_test_report_reference_event_id: Option<EventId>,
    pub local_attestation_reference_event_id: Option<EventId>,
    pub local_approval_reference_event_id: Option<EventId>,
    pub computed_grant_reference_event_id: Option<EventId>,
    pub audit_rollback_reference_event_id: Option<EventId>,
    pub service_slot_reservation_event_id: Option<EventId>,
    pub ram_only_service_slot_id: Option<ModuleServiceSlotId>,
    pub accepts_loader_descriptor: bool,
    pub accepts_descriptor_bytes: bool,
    pub accepts_artifact_bytes: bool,
    pub authorizes_descriptor_intake: bool,
    pub authorizes_artifact_byte_intake: bool,
    pub maps_executable_pages: bool,
    pub jumps_to_entrypoint: bool,
    pub authorizes_execution: bool,
    pub mutates_service_registry: bool,
    pub writes_durable_audit_state: bool,
    pub installs_rollback_state: bool,
    pub allocates_service_slot: bool,
    pub creates_service_inventory_records: bool,
    pub loads_artifact: bool,
}

#[derive(Clone, Copy)]
pub struct ModuleLoaderLiveLoadBoundarySourceEvidence {
    pub schema: &'static str,
    pub boundary_schema: &'static str,
    pub boundary_id: &'static str,
    pub source_method: &'static str,
    pub source_fact_locator: &'static str,
    pub readiness_status: &'static str,
    pub readiness_reason: &'static str,
    pub boundary_status: &'static str,
    pub boundary_reason: &'static str,
    pub boundary_present: bool,
    pub boundary_scope: &'static str,
    pub boundary_schema_ok: bool,
    pub boundary_provenance_ok: bool,
    pub boundary_classification: &'static str,
    pub requested_capability: &'static str,
    pub load_mode: &'static str,
    pub target: &'static str,
    pub load_attempt_boundary_present: bool,
    pub load_attempt_boundary_source_chain_complete: bool,
    pub artifact_load_boundary_present: bool,
    pub artifact_load_boundary_source_chain_complete: bool,
    pub executable_mapping_boundary_present: bool,
    pub executable_mapping_boundary_source_chain_complete: bool,
    pub entrypoint_transfer_boundary_present: bool,
    pub entrypoint_transfer_boundary_source_chain_complete: bool,
    pub service_start_boundary_present: bool,
    pub service_start_boundary_source_chain_complete: bool,
    pub service_health_binding_boundary_present: bool,
    pub service_health_binding_boundary_source_chain_complete: bool,
    pub service_running_state_boundary_present: bool,
    pub service_running_state_boundary_source_chain_complete: bool,
    pub service_start_audit_boundary_present: bool,
    pub service_start_audit_boundary_source_chain_complete: bool,
    pub service_unload_cleanup_boundary_present: bool,
    pub service_unload_cleanup_boundary_source_chain_complete: bool,
    pub live_load_commit_boundary_present: bool,
    pub live_load_commit_boundary_source_chain_complete: bool,
    pub commit_audit_boundary_present: bool,
    pub commit_audit_boundary_source_chain_complete: bool,
    pub commit_rollback_boundary_present: bool,
    pub commit_rollback_boundary_source_chain_complete: bool,
    pub commit_result_boundary_present: bool,
    pub commit_result_boundary_source_chain_complete: bool,
    pub descriptor_acceptance_authority_boundary_present: bool,
    pub descriptor_acceptance_authority_boundary_source_chain_complete: bool,
    pub descriptor_parser_contract_boundary_present: bool,
    pub descriptor_parser_contract_boundary_source_chain_complete: bool,
    pub descriptor_parser_result_boundary_present: bool,
    pub descriptor_parser_result_boundary_source_chain_complete: bool,
    pub descriptor_schema_validation_boundary_present: bool,
    pub descriptor_schema_validation_boundary_source_chain_complete: bool,
    pub descriptor_capability_validation_boundary_present: bool,
    pub descriptor_capability_validation_boundary_source_chain_complete: bool,
    pub descriptor_load_plan_boundary_present: bool,
    pub descriptor_load_plan_boundary_source_chain_complete: bool,
    pub executable_load_plan_authority_boundary_present: bool,
    pub executable_load_plan_authority_boundary_source_chain_complete: bool,
    pub executable_load_plan_result_boundary_present: bool,
    pub executable_load_plan_result_boundary_source_chain_complete: bool,
    pub executable_image_layout_boundary_present: bool,
    pub executable_image_layout_boundary_source_chain_complete: bool,
    pub executable_page_mapping_plan_boundary_present: bool,
    pub executable_page_mapping_plan_boundary_source_chain_complete: bool,
    pub executable_page_mapping_boundary_present: bool,
    pub executable_page_mapping_boundary_source_chain_complete: bool,
    pub descriptor_executable_page_binding_boundary_present: bool,
    pub descriptor_executable_page_binding_boundary_source_chain_complete: bool,
    pub executable_entrypoint_binding_boundary_present: bool,
    pub executable_entrypoint_binding_boundary_source_chain_complete: bool,
    pub executable_entrypoint_transfer_authorization_boundary_present: bool,
    pub executable_entrypoint_transfer_authorization_boundary_source_chain_complete: bool,
    pub executable_entrypoint_transfer_boundary_present: bool,
    pub executable_entrypoint_transfer_boundary_source_chain_complete: bool,
    pub executable_entrypoint_handoff_boundary_present: bool,
    pub executable_entrypoint_handoff_boundary_source_chain_complete: bool,
    pub artifact_byte_intake_boundary_present: bool,
    pub artifact_byte_intake_boundary_source_chain_complete: bool,
    pub execution_authorization_boundary_present: bool,
    pub execution_authorization_boundary_source_chain_complete: bool,
    pub service_registry_mutation_boundary_present: bool,
    pub service_registry_mutation_boundary_source_chain_complete: bool,
    pub service_slot_binding_source_evidence_present: bool,
    pub health_state_hooks_source_evidence_present: bool,
    pub artifact_hash_binding_present: bool,
    pub entrypoint_abi_source_evidence_present: bool,
    pub address_space_source_evidence_present: bool,
    pub memory_map_source_evidence_present: bool,
    pub capability_import_table_source_evidence_present: bool,
    pub audit_rollback_write_boundary_source_evidence_present: bool,
    pub retained_module_evidence_present: bool,
    pub retained_artifact_reference_present: bool,
    pub retained_service_slot_reservation_present: bool,
    pub source_chain_complete: bool,
    pub load_attempt_boundary_source_evidence_event_id: Option<EventId>,
    pub artifact_load_boundary_source_evidence_event_id: Option<EventId>,
    pub executable_mapping_boundary_source_evidence_event_id: Option<EventId>,
    pub entrypoint_transfer_boundary_source_evidence_event_id: Option<EventId>,
    pub service_start_boundary_source_evidence_event_id: Option<EventId>,
    pub service_health_binding_boundary_source_evidence_event_id: Option<EventId>,
    pub service_running_state_boundary_source_evidence_event_id: Option<EventId>,
    pub service_start_audit_boundary_source_evidence_event_id: Option<EventId>,
    pub service_unload_cleanup_boundary_source_evidence_event_id: Option<EventId>,
    pub live_load_commit_boundary_source_evidence_event_id: Option<EventId>,
    pub commit_audit_boundary_source_evidence_event_id: Option<EventId>,
    pub commit_rollback_boundary_source_evidence_event_id: Option<EventId>,
    pub commit_result_boundary_source_evidence_event_id: Option<EventId>,
    pub descriptor_acceptance_authority_boundary_source_evidence_event_id: Option<EventId>,
    pub descriptor_parser_contract_boundary_source_evidence_event_id: Option<EventId>,
    pub descriptor_parser_result_boundary_source_evidence_event_id: Option<EventId>,
    pub descriptor_schema_validation_boundary_source_evidence_event_id: Option<EventId>,
    pub descriptor_capability_validation_boundary_source_evidence_event_id: Option<EventId>,
    pub descriptor_load_plan_boundary_source_evidence_event_id: Option<EventId>,
    pub executable_load_plan_authority_boundary_source_evidence_event_id: Option<EventId>,
    pub executable_load_plan_result_boundary_source_evidence_event_id: Option<EventId>,
    pub executable_image_layout_boundary_source_evidence_event_id: Option<EventId>,
    pub executable_page_mapping_plan_boundary_source_evidence_event_id: Option<EventId>,
    pub executable_page_mapping_boundary_source_evidence_event_id: Option<EventId>,
    pub descriptor_executable_page_binding_boundary_source_evidence_event_id: Option<EventId>,
    pub executable_entrypoint_binding_boundary_source_evidence_event_id: Option<EventId>,
    pub executable_entrypoint_transfer_authorization_boundary_source_evidence_event_id:
        Option<EventId>,
    pub executable_entrypoint_transfer_boundary_source_evidence_event_id: Option<EventId>,
    pub executable_entrypoint_handoff_boundary_source_evidence_event_id: Option<EventId>,
    pub artifact_byte_intake_boundary_source_evidence_event_id: Option<EventId>,
    pub execution_authorization_boundary_source_evidence_event_id: Option<EventId>,
    pub service_registry_mutation_boundary_source_evidence_event_id: Option<EventId>,
    pub service_slot_binding_source_evidence_event_id: Option<EventId>,
    pub health_state_hooks_source_evidence_event_id: Option<EventId>,
    pub artifact_hash_binding_source_evidence_event_id: Option<EventId>,
    pub entrypoint_abi_source_evidence_event_id: Option<EventId>,
    pub address_space_source_evidence_event_id: Option<EventId>,
    pub memory_map_source_evidence_event_id: Option<EventId>,
    pub capability_import_table_source_evidence_event_id: Option<EventId>,
    pub audit_rollback_write_boundary_source_evidence_event_id: Option<EventId>,
    pub loader_runtime_source_evidence_event_ids: [Option<EventId>; 10],
    pub loader_runtime_source_evidence_present: [bool; 10],
    pub loader_runtime_fact_present: [bool; 10],
    pub manifest_reference_event_id: Option<EventId>,
    pub artifact_reference_event_id: Option<EventId>,
    pub vm_test_report_reference_event_id: Option<EventId>,
    pub local_attestation_reference_event_id: Option<EventId>,
    pub local_approval_reference_event_id: Option<EventId>,
    pub computed_grant_reference_event_id: Option<EventId>,
    pub audit_rollback_reference_event_id: Option<EventId>,
    pub service_slot_reservation_event_id: Option<EventId>,
    pub ram_only_service_slot_id: Option<ModuleServiceSlotId>,
    pub accepts_loader_descriptor: bool,
    pub accepts_descriptor_bytes: bool,
    pub accepts_artifact_bytes: bool,
    pub authorizes_descriptor_intake: bool,
    pub authorizes_artifact_byte_intake: bool,
    pub maps_executable_pages: bool,
    pub jumps_to_entrypoint: bool,
    pub authorizes_execution: bool,
    pub mutates_service_registry: bool,
    pub writes_durable_audit_state: bool,
    pub installs_rollback_state: bool,
    pub allocates_service_slot: bool,
    pub creates_service_inventory_records: bool,
    pub loads_artifact: bool,
    pub starts_service: bool,
    pub marks_service_running: bool,
    pub creates_service_health_records: bool,
    pub writes_service_start_audit_record: bool,
    pub unloads_service: bool,
    pub cleans_up_service_slot: bool,
    pub commits_live_load: bool,
    pub writes_load_commit_audit_record: bool,
    pub installs_commit_rollback_record: bool,
    pub records_load_result: bool,
    pub load_attempted: bool,
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryDisableModuleTargetBindingReference {
    pub disable_module_target_binding_hash: [u8; 32],
    pub retained_rollback_apply_authorization_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_envelope_reference_hash: [u8; 32],
    pub command_body_canonicalization_hash: [u8; 32],
    pub handler_binding_hash: [u8; 32],
    pub status_read_handler_hash: [u8; 32],
    pub rollback_preview_authorization_hash: [u8; 32],
    pub rollback_apply_authorization_hash: [u8; 32],
    pub command_dispatch_boundary_id: &'static str,
    pub disable_module_target_id: &'static str,
    pub disable_module_target_projection_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryRestartLastGoodTargetBindingReference {
    pub restart_last_good_target_binding_hash: [u8; 32],
    pub retained_disable_module_target_binding_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_envelope_reference_hash: [u8; 32],
    pub command_body_canonicalization_hash: [u8; 32],
    pub handler_binding_hash: [u8; 32],
    pub status_read_handler_hash: [u8; 32],
    pub rollback_preview_authorization_hash: [u8; 32],
    pub rollback_apply_authorization_hash: [u8; 32],
    pub disable_module_target_binding_hash: [u8; 32],
    pub command_dispatch_boundary_id: &'static str,
    pub restart_last_good_target_id: &'static str,
    pub restart_last_good_target_projection_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryLoadArtifactByHashTargetBindingReference {
    pub load_artifact_by_hash_target_binding_hash: [u8; 32],
    pub retained_restart_last_good_target_binding_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_envelope_reference_hash: [u8; 32],
    pub command_body_canonicalization_hash: [u8; 32],
    pub handler_binding_hash: [u8; 32],
    pub status_read_handler_hash: [u8; 32],
    pub rollback_preview_authorization_hash: [u8; 32],
    pub rollback_apply_authorization_hash: [u8; 32],
    pub disable_module_target_binding_hash: [u8; 32],
    pub restart_last_good_target_binding_hash: [u8; 32],
    pub command_dispatch_boundary_id: &'static str,
    pub load_artifact_by_hash_target_id: &'static str,
    pub load_artifact_by_hash_target_artifact_hash: [u8; 32],
    pub load_artifact_by_hash_target_projection_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryMemoryWriteAuthorityReference {
    pub recovery_memory_write_authority_hash: [u8; 32],
    pub retained_load_artifact_by_hash_target_binding_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_envelope_reference_hash: [u8; 32],
    pub command_body_canonicalization_hash: [u8; 32],
    pub handler_binding_hash: [u8; 32],
    pub status_read_handler_hash: [u8; 32],
    pub rollback_preview_authorization_hash: [u8; 32],
    pub rollback_apply_authorization_hash: [u8; 32],
    pub disable_module_target_binding_hash: [u8; 32],
    pub restart_last_good_target_binding_hash: [u8; 32],
    pub load_artifact_by_hash_target_binding_hash: [u8; 32],
    pub command_dispatch_boundary_id: &'static str,
    pub recovery_memory_write_authority_id: &'static str,
    pub recovery_memory_projection_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DurableAuditRollbackWriteAuthorityReference {
    pub durable_audit_rollback_write_authority_hash: [u8; 32],
    pub retained_recovery_memory_write_authority_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_envelope_reference_hash: [u8; 32],
    pub command_body_canonicalization_hash: [u8; 32],
    pub handler_binding_hash: [u8; 32],
    pub status_read_handler_hash: [u8; 32],
    pub rollback_preview_authorization_hash: [u8; 32],
    pub rollback_apply_authorization_hash: [u8; 32],
    pub disable_module_target_binding_hash: [u8; 32],
    pub restart_last_good_target_binding_hash: [u8; 32],
    pub load_artifact_by_hash_target_binding_hash: [u8; 32],
    pub recovery_memory_write_authority_hash: [u8; 32],
    pub command_dispatch_boundary_id: &'static str,
    pub durable_audit_rollback_write_authority_id: &'static str,
    pub durable_audit_rollback_projection_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryServiceInventorySideEffectBoundaryReference {
    pub service_inventory_side_effect_boundary_hash: [u8; 32],
    pub retained_durable_audit_rollback_write_authority_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_envelope_reference_hash: [u8; 32],
    pub command_body_canonicalization_hash: [u8; 32],
    pub handler_binding_hash: [u8; 32],
    pub status_read_handler_hash: [u8; 32],
    pub rollback_preview_authorization_hash: [u8; 32],
    pub rollback_apply_authorization_hash: [u8; 32],
    pub disable_module_target_binding_hash: [u8; 32],
    pub restart_last_good_target_binding_hash: [u8; 32],
    pub load_artifact_by_hash_target_binding_hash: [u8; 32],
    pub recovery_memory_write_authority_hash: [u8; 32],
    pub durable_audit_rollback_write_authority_hash: [u8; 32],
    pub command_dispatch_boundary_id: &'static str,
    pub service_inventory_side_effect_boundary_id: &'static str,
    pub service_inventory_projection_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryLifelineCommandDispatchBehaviorReference {
    pub command_dispatch_behavior_hash: [u8; 32],
    pub retained_service_inventory_side_effect_boundary_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_envelope_reference_hash: [u8; 32],
    pub command_body_canonicalization_hash: [u8; 32],
    pub handler_binding_hash: [u8; 32],
    pub status_read_handler_hash: [u8; 32],
    pub rollback_preview_authorization_hash: [u8; 32],
    pub rollback_apply_authorization_hash: [u8; 32],
    pub disable_module_target_binding_hash: [u8; 32],
    pub restart_last_good_target_binding_hash: [u8; 32],
    pub load_artifact_by_hash_target_binding_hash: [u8; 32],
    pub recovery_memory_write_authority_hash: [u8; 32],
    pub durable_audit_rollback_write_authority_hash: [u8; 32],
    pub service_inventory_side_effect_boundary_hash: [u8; 32],
    pub command_dispatch_boundary_id: &'static str,
    pub command_dispatch_behavior_id: &'static str,
    pub command_dispatch_behavior_projection_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryLifelineCommandExecutorCapabilityTableReference {
    pub executor_capability_table_hash: [u8; 32],
    pub retained_command_dispatch_behavior_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_envelope_reference_hash: [u8; 32],
    pub command_body_canonicalization_hash: [u8; 32],
    pub handler_binding_hash: [u8; 32],
    pub status_read_handler_hash: [u8; 32],
    pub rollback_preview_authorization_hash: [u8; 32],
    pub rollback_apply_authorization_hash: [u8; 32],
    pub disable_module_target_binding_hash: [u8; 32],
    pub restart_last_good_target_binding_hash: [u8; 32],
    pub load_artifact_by_hash_target_binding_hash: [u8; 32],
    pub recovery_memory_write_authority_hash: [u8; 32],
    pub durable_audit_rollback_write_authority_hash: [u8; 32],
    pub service_inventory_side_effect_boundary_hash: [u8; 32],
    pub command_dispatch_behavior_hash: [u8; 32],
    pub command_dispatch_boundary_id: &'static str,
    pub executor_capability_table_id: &'static str,
    pub executor_capability_projection_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryLifelineCommandSideEffectGateReference {
    pub side_effect_gate_hash: [u8; 32],
    pub retained_executor_capability_table_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_envelope_reference_hash: [u8; 32],
    pub command_body_canonicalization_hash: [u8; 32],
    pub handler_binding_hash: [u8; 32],
    pub status_read_handler_hash: [u8; 32],
    pub rollback_preview_authorization_hash: [u8; 32],
    pub rollback_apply_authorization_hash: [u8; 32],
    pub disable_module_target_binding_hash: [u8; 32],
    pub restart_last_good_target_binding_hash: [u8; 32],
    pub load_artifact_by_hash_target_binding_hash: [u8; 32],
    pub recovery_memory_write_authority_hash: [u8; 32],
    pub durable_audit_rollback_write_authority_hash: [u8; 32],
    pub service_inventory_side_effect_boundary_hash: [u8; 32],
    pub command_dispatch_behavior_hash: [u8; 32],
    pub executor_capability_table_hash: [u8; 32],
    pub command_dispatch_boundary_id: &'static str,
    pub side_effect_gate_id: &'static str,
    pub side_effect_projection_hash: [u8; 32],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryLifelineCommandExecutionStageReference {
    pub schema: &'static str,
    pub stage_name: &'static str,
    pub execution_stage_hash: [u8; 32],
    pub retained_previous_stage_event_id: EventId,
    pub command_id: &'static str,
    pub argument_schema: &'static str,
    pub argument_hash: [u8; 32],
    pub target_locator: RecoveryCommandTargetLocator,
    pub command_envelope_reference_hash: [u8; 32],
    pub command_body_canonicalization_hash: [u8; 32],
    pub handler_binding_hash: [u8; 32],
    pub status_read_handler_hash: [u8; 32],
    pub rollback_preview_authorization_hash: [u8; 32],
    pub rollback_apply_authorization_hash: [u8; 32],
    pub disable_module_target_binding_hash: [u8; 32],
    pub restart_last_good_target_binding_hash: [u8; 32],
    pub load_artifact_by_hash_target_binding_hash: [u8; 32],
    pub recovery_memory_write_authority_hash: [u8; 32],
    pub durable_audit_rollback_write_authority_hash: [u8; 32],
    pub service_inventory_side_effect_boundary_hash: [u8; 32],
    pub command_dispatch_behavior_hash: [u8; 32],
    pub executor_capability_table_hash: [u8; 32],
    pub side_effect_gate_hash: [u8; 32],
    pub execution_enablement_hash: Option<[u8; 32]>,
    pub execution_preflight_hash: Option<[u8; 32]>,
    pub execution_intent_hash: Option<[u8; 32]>,
    pub execution_commit_gate_hash: Option<[u8; 32]>,
    pub execution_result_denial_hash: Option<[u8; 32]>,
    pub execution_audit_denial_hash: Option<[u8; 32]>,
    pub execution_observation_denial_hash: Option<[u8; 32]>,
    pub command_dispatch_boundary_id: &'static str,
    pub execution_stage_id: &'static str,
    pub execution_stage_projection_hash: [u8; 32],
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
    HelloServiceLifecycle(HelloServiceLifecycleBinding),
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
    ModuleServiceSlotAllocatorFactSourceEvidence(ModuleServiceSlotAllocatorFactSourceEvidence),
    ModuleServiceSlotAllocatorPrerequisiteSourceEvidence(
        ModuleServiceSlotAllocatorPrerequisiteSourceEvidence,
    ),
    ModuleServiceSlotAllocatorAuthoritySourceEvidence(
        ModuleServiceSlotAllocatorAuthoritySourceEvidence,
    ),
    ModuleServiceSlotAllocationIntentSourceEvidence(
        ModuleServiceSlotAllocationIntentSourceEvidence,
    ),
    ModuleServiceSlotAuthorityInputSourceEvidence(ModuleServiceSlotAuthorityInputSourceEvidence),
    ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence(
        ModuleServiceSlotAllocatorAuthorityDecisionSourceEvidence,
    ),
    ModuleServiceSlotRegistryWriteCommitGateSourceEvidence(
        ModuleServiceSlotRegistryWriteCommitGateSourceEvidence,
    ),
    ModuleLoaderIdentitySourceEvidence(ModuleLoaderIdentitySourceEvidence),
    ModuleLoaderArtifactHashBindingSourceEvidence(ModuleLoaderArtifactHashBindingSourceEvidence),
    ModuleLoaderFactSourceEvidence(ModuleLoaderFactSourceEvidence),
    ModuleLoaderRuntimeExecutionCommitGateSourceEvidence(
        ModuleLoaderRuntimeExecutionCommitGateSourceEvidence,
    ),
    ModuleLoaderDescriptorIntakeBoundarySourceEvidence(
        ModuleLoaderDescriptorIntakeBoundarySourceEvidence,
    ),
    ModuleLoaderArtifactByteIntakeBoundarySourceEvidence(
        ModuleLoaderArtifactByteIntakeBoundarySourceEvidence,
    ),
    ModuleLoaderExecutionAuthorizationBoundarySourceEvidence(
        ModuleLoaderExecutionAuthorizationBoundarySourceEvidence,
    ),
    ModuleLoaderServiceRegistryMutationBoundarySourceEvidence(
        ModuleLoaderServiceRegistryMutationBoundarySourceEvidence,
    ),
    ModuleLoaderLoadAttemptBoundarySourceEvidence(ModuleLoaderLiveLoadBoundarySourceEvidence),
    ModuleLoaderArtifactLoadBoundarySourceEvidence(ModuleLoaderLiveLoadBoundarySourceEvidence),
    ModuleLoaderExecutableMappingBoundarySourceEvidence(ModuleLoaderLiveLoadBoundarySourceEvidence),
    ModuleLoaderEntrypointTransferBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderServiceStartBoundarySourceEvidence(ModuleLoaderLiveLoadBoundarySourceEvidence),
    ModuleLoaderServiceHealthBindingBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderServiceRunningStateBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderServiceStartAuditBoundarySourceEvidence(ModuleLoaderLiveLoadBoundarySourceEvidence),
    ModuleLoaderServiceUnloadCleanupBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderLiveLoadCommitBoundarySourceEvidence(ModuleLoaderLiveLoadBoundarySourceEvidence),
    ModuleLoaderCommitAuditBoundarySourceEvidence(ModuleLoaderLiveLoadBoundarySourceEvidence),
    ModuleLoaderCommitRollbackBoundarySourceEvidence(ModuleLoaderLiveLoadBoundarySourceEvidence),
    ModuleLoaderCommitResultBoundarySourceEvidence(ModuleLoaderLiveLoadBoundarySourceEvidence),
    ModuleLoaderDescriptorAcceptanceAuthorityBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderDescriptorParserContractBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderDescriptorParserResultBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderDescriptorSchemaValidationBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderDescriptorCapabilityValidationBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderDescriptorLoadPlanBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderExecutableLoadPlanAuthorityBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderExecutableLoadPlanResultBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderExecutableImageLayoutBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderExecutablePageMappingPlanBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderExecutablePageMappingBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderDescriptorExecutablePageBindingBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderExecutableEntrypointBindingBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderExecutableEntrypointTransferAuthorizationBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderExecutableEntrypointTransferBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderExecutableEntrypointHandoffBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
    ModuleLoaderExecutableEntrypointInvocationBoundarySourceEvidence(
        ModuleLoaderLiveLoadBoundarySourceEvidence,
    ),
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
    RecoveryDisableModuleTargetBindingReference(RecoveryDisableModuleTargetBindingReference),
    RecoveryRestartLastGoodTargetBindingReference(RecoveryRestartLastGoodTargetBindingReference),
    RecoveryLoadArtifactByHashTargetBindingReference(
        RecoveryLoadArtifactByHashTargetBindingReference,
    ),
    RecoveryMemoryWriteAuthorityReference(RecoveryMemoryWriteAuthorityReference),
    DurableAuditRollbackWriteAuthorityReference(DurableAuditRollbackWriteAuthorityReference),
    RecoveryServiceInventorySideEffectBoundaryReference(
        RecoveryServiceInventorySideEffectBoundaryReference,
    ),
    RecoveryLifelineCommandDispatchBehaviorReference(
        RecoveryLifelineCommandDispatchBehaviorReference,
    ),
    RecoveryLifelineCommandExecutorCapabilityTableReference(
        RecoveryLifelineCommandExecutorCapabilityTableReference,
    ),
    RecoveryLifelineCommandSideEffectGateReference(RecoveryLifelineCommandSideEffectGateReference),
    RecoveryLifelineCommandExecutionStageReference(RecoveryLifelineCommandExecutionStageReference),
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
