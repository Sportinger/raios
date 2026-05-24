use spin::Mutex;

use crate::event_log_evidence::{
    DENIED_EVIDENCE, DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_EVIDENCE,
    MODULE_AUDIT_ROLLBACK_REFERENCE_EVIDENCE, MODULE_CANDIDATE_ARTIFACT_REFERENCE_EVIDENCE,
    MODULE_COMPUTED_GRANT_REFERENCE_EVIDENCE, MODULE_LOADER_IDENTITY_SOURCE_EVIDENCE,
    MODULE_LOAD_GATE_EVIDENCE, MODULE_LOCAL_APPROVAL_REFERENCE_EVIDENCE,
    MODULE_LOCAL_ATTESTATION_REFERENCE_EVIDENCE, MODULE_MANIFEST_REFERENCE_EVIDENCE,
    MODULE_SERVICE_SLOT_RESERVATION_EVIDENCE, MODULE_VM_TEST_REPORT_REFERENCE_EVIDENCE,
    PROVIDER_BINDING_CONSUMPTION_EVIDENCE, PROVIDER_EXPORT_AUDIT_BINDING_EVIDENCE,
    PROVIDER_EXPORT_DENIAL_AUDIT_EVIDENCE, PROVIDER_REQUEST_BINDING_DENIAL_EVIDENCE,
    PROVIDER_REQUEST_BINDING_EVIDENCE, PROVIDER_REQUEST_ENVELOPE_EVIDENCE, READ_EVIDENCE,
    RECOVERY_ARTIFACT_IDENTITY_REFERENCE_EVIDENCE, RECOVERY_ARTIFACT_LOADER_REFERENCE_EVIDENCE,
    RECOVERY_ARTIFACT_LOAD_DENIAL_EVIDENCE, RECOVERY_ARTIFACT_LOCAL_APPROVAL_REFERENCE_EVIDENCE,
    RECOVERY_ARTIFACT_ROLLBACK_EVIDENCE_REFERENCE_EVIDENCE,
    RECOVERY_ARTIFACT_TRUST_REFERENCE_EVIDENCE, RECOVERY_ARTIFACT_VM_TEST_REFERENCE_EVIDENCE,
    RECOVERY_DISABLE_MODULE_TARGET_BINDING_EVIDENCE,
    RECOVERY_LIFELINE_COMMAND_BODY_CANONICALIZATION_EVIDENCE,
    RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_EVIDENCE,
    RECOVERY_LIFELINE_COMMAND_ENVELOPE_REFERENCE_EVIDENCE,
    RECOVERY_LIFELINE_COMMAND_EXECUTION_STAGE_EVIDENCE,
    RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_EVIDENCE,
    RECOVERY_LIFELINE_COMMAND_HANDLER_BINDING_EVIDENCE,
    RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_EVIDENCE,
    RECOVERY_LIFELINE_REQUEST_REFERENCE_EVIDENCE, RECOVERY_LIFELINE_STATUS_READ_HANDLER_EVIDENCE,
    RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_EVIDENCE,
    RECOVERY_MEMORY_WRITE_AUTHORITY_EVIDENCE, RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_EVIDENCE,
    RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_EVIDENCE,
    RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_EVIDENCE,
    RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_EVIDENCE,
};
use crate::event_log_module_checks::{
    module_audit_rollback_binds_computed_grant, module_audit_rollback_reference_hash_mismatch,
    module_audit_rollback_reference_matches, module_candidate_artifact_reference_hashes_consistent,
    module_candidate_artifact_reference_matches, module_computed_grant_reference_hashes_consistent,
    module_computed_grant_reference_matches, module_local_approval_reference_hashes_consistent,
    module_local_approval_reference_matches, module_local_attestation_reference_hashes_consistent,
    module_local_attestation_reference_matches, module_manifest_reference_hashes_consistent,
    module_manifest_reference_matches, module_service_slot_reservation_hash_mismatch,
    module_vm_test_report_reference_hashes_consistent, module_vm_test_report_reference_matches,
};
use crate::event_log_types::{
    ConsumedProviderBinding, ModuleAuditRollbackReferenceGateCheck,
    ModuleCandidateArtifactReferenceGateCheck, ModuleLocalApprovalReferenceGateCheck,
    ModuleLocalAttestationReferenceGateCheck, ModuleManifestReferenceGateCheck,
    ModuleServiceSlotReservationGateCheck, ModuleVmTestReportReferenceGateCheck,
};
pub use crate::event_log_types::{
    DurableAuditRollbackWriteAuthorityReference, Event, EventBindings, EventId, EventSnapshot,
    ModuleAuditRollbackReference, ModuleCandidateArtifactReference, ModuleComputedGrantReference,
    ModuleLoadGateBinding, ModuleLoaderIdentitySourceEvidence, ModuleLocalApprovalReference,
    ModuleLocalAttestationReference, ModuleManifestReference, ModuleServiceSlotId,
    ModuleServiceSlotReservation, ModuleVmTestReportReference, ProviderBindingConsumption,
    ProviderBindingGateCheck, ProviderBindingGateSelfTestCase, ProviderContextHashes,
    ProviderContextInjectionAuthorization, ProviderContextInjectionGateCheck,
    ProviderContextInjectionGateSelfTestCase, ProviderExportAuditBinding, ProviderRequestBinding,
    ProviderRequestEnvelopeBinding, RecoveryArtifactIdentityReference,
    RecoveryArtifactLoadDenialBinding, RecoveryArtifactLoaderReference,
    RecoveryArtifactLocalApprovalReference, RecoveryArtifactRollbackEvidenceReference,
    RecoveryArtifactTrustReference, RecoveryArtifactVmTestReference, RecoveryCommandTargetLocator,
    RecoveryDisableModuleTargetBindingReference,
    RecoveryLifelineCommandBodyCanonicalizationReference,
    RecoveryLifelineCommandDispatchBehaviorReference, RecoveryLifelineCommandEnvelopeReference,
    RecoveryLifelineCommandExecutionStageReference,
    RecoveryLifelineCommandExecutorCapabilityTableReference,
    RecoveryLifelineCommandHandlerBindingReference, RecoveryLifelineCommandSideEffectGateReference,
    RecoveryLifelineRequestReference, RecoveryLifelineStatusReadHandlerReference,
    RecoveryLoadArtifactByHashTargetBindingReference, RecoveryMemoryWriteAuthorityReference,
    RecoveryRestartLastGoodTargetBindingReference, RecoveryRollbackApplyAuthorizationReference,
    RecoveryRollbackPreviewAuthorizationReference,
    RecoveryServiceInventorySideEffectBoundaryReference, DEFAULT_EVENT_LIMIT, EVENT_CAPACITY,
    PROVIDER_BINDING_GATE_SELFTEST_CASES, PROVIDER_CONTEXT_INJECTION_GATE_SELFTEST_CASES,
};
use crate::module_evidence;

static LOG: Mutex<EventLog> = Mutex::new(EventLog::new());

pub(crate) struct EventLog {
    events: [Option<Event>; EVENT_CAPACITY],
    consumed_bindings: [Option<ConsumedProviderBinding>; EVENT_CAPACITY],
    next_slot: usize,
    next_consumed_slot: usize,
    len: usize,
    consumed_len: usize,
    next_sequence: u64,
}

impl EventLog {
    pub(crate) const fn new() -> Self {
        Self {
            events: [None; EVENT_CAPACITY],
            consumed_bindings: [None; EVENT_CAPACITY],
            next_slot: 0,
            next_consumed_slot: 0,
            len: 0,
            consumed_len: 0,
            next_sequence: 1,
        }
    }

    pub(crate) fn record(&mut self, mut event: Event) -> EventId {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        event.sequence = sequence;

        self.events[self.next_slot] = Some(event);
        self.next_slot = (self.next_slot + 1) % EVENT_CAPACITY;
        self.len = usize::min(self.len + 1, EVENT_CAPACITY);

        EventId { sequence }
    }

    fn snapshot_recent(&self, requested_limit: usize) -> EventSnapshot {
        let limit = normalize_limit(requested_limit);
        let want = usize::min(self.len, limit);
        let skip = self.len.saturating_sub(want);
        let oldest = if self.len == EVENT_CAPACITY {
            self.next_slot
        } else {
            0
        };

        let mut events = [None; EVENT_CAPACITY];
        let mut out_idx = 0usize;
        let mut idx = skip;
        while idx < self.len {
            let source = (oldest + idx) % EVENT_CAPACITY;
            events[out_idx] = self.events[source];
            out_idx += 1;
            idx += 1;
        }

        let total_count = self.next_sequence.saturating_sub(1);
        let dropped_before_sequence = if total_count > self.len as u64 {
            total_count - self.len as u64 + 1
        } else {
            0
        };

        EventSnapshot {
            events,
            len: out_idx,
            limit,
            capacity: EVENT_CAPACITY,
            total_count,
            dropped_before_sequence,
        }
    }

    pub(crate) fn check_provider_context_binding_gate(
        &self,
        _context: ProviderContextHashes,
    ) -> ProviderBindingGateCheck {
        let Some((export_event_id, export_binding)) = self.latest_export_audit_binding() else {
            return ProviderBindingGateCheck::rejected(
                "missing",
                "provider_context_export_audit_binding_missing",
            );
        };

        if self.binding_consumed(export_binding.request_binding_event_id, export_event_id) {
            return ProviderBindingGateCheck {
                status: "rejected",
                reason: "binding_already_consumed",
                request_binding_event_id: Some(export_binding.request_binding_event_id),
                export_audit_binding_event_id: Some(export_event_id),
                request_envelope_event_id: Some(export_binding.request_envelope_event_id),
                request_binding: None,
                export_audit_binding: Some(export_binding),
                consumed: true,
                retained: true,
            };
        }

        let Some(request_event) = self.event_by_sequence(export_binding.request_binding_event_id)
        else {
            return ProviderBindingGateCheck::with_export(
                "rejected",
                "binding_stale_or_dropped_event_id",
                export_event_id,
                export_binding,
            );
        };

        let EventBindings::ProviderRequestBound(request_binding) = request_event.bindings else {
            return ProviderBindingGateCheck::with_export(
                "rejected",
                "binding_denied_schema_or_wrong_variant",
                export_event_id,
                export_binding,
            );
        };

        let Some(envelope_event) =
            self.event_by_sequence(request_binding.request_envelope_event_id)
        else {
            return ProviderBindingGateCheck::with_pair(
                "rejected",
                "binding_stale_or_dropped_event_id",
                request_event.sequence,
                request_binding,
                export_event_id,
                export_binding,
            );
        };

        let EventBindings::ProviderRequestEnvelope(envelope_binding) = envelope_event.bindings
        else {
            return ProviderBindingGateCheck::with_pair(
                "rejected",
                "request_envelope_wrong_variant",
                request_event.sequence,
                request_binding,
                export_event_id,
                export_binding,
            );
        };

        if export_binding.request_envelope_event_id.sequence() != envelope_event.sequence
            || request_binding.request_envelope_event_id.sequence() != envelope_event.sequence
        {
            return ProviderBindingGateCheck::with_pair(
                "rejected",
                "binding_request_envelope_event_id_mismatch",
                request_event.sequence,
                request_binding,
                export_event_id,
                export_binding,
            );
        }
        if request_binding.request_id != export_binding.request_id
            || request_binding.request_id != envelope_binding.request_id
        {
            return ProviderBindingGateCheck::with_pair(
                "rejected",
                "binding_request_id_mismatch",
                request_event.sequence,
                request_binding,
                export_event_id,
                export_binding,
            );
        }
        if request_binding.request_body_hash != export_binding.request_body_hash
            || request_binding.request_body_hash != envelope_binding.request_body_hash
        {
            return ProviderBindingGateCheck::with_pair(
                "rejected",
                "binding_request_body_hash_mismatch",
                request_event.sequence,
                request_binding,
                export_event_id,
                export_binding,
            );
        }
        if request_binding.request_envelope_hash != export_binding.request_envelope_hash
            || request_binding.request_envelope_hash != envelope_binding.envelope_hash
        {
            return ProviderBindingGateCheck::with_pair(
                "rejected",
                "binding_request_envelope_hash_mismatch",
                request_event.sequence,
                request_binding,
                export_event_id,
                export_binding,
            );
        }
        if request_binding.request_binding_hash != export_binding.request_binding_hash {
            return ProviderBindingGateCheck::with_pair(
                "rejected",
                "binding_request_binding_hash_mismatch",
                request_event.sequence,
                request_binding,
                export_event_id,
                export_binding,
            );
        }
        if request_binding.context.projected_packet_hash
            != export_binding.context.projected_packet_hash
        {
            return ProviderBindingGateCheck::with_pair(
                "rejected",
                "binding_provider_minimal_packet_hash_mismatch",
                request_event.sequence,
                request_binding,
                export_event_id,
                export_binding,
            );
        }
        if request_binding.context.exported_field_list_hash
            != export_binding.context.exported_field_list_hash
        {
            return ProviderBindingGateCheck::with_pair(
                "rejected",
                "binding_exported_field_list_hash_mismatch",
                request_event.sequence,
                request_binding,
                export_event_id,
                export_binding,
            );
        }
        if request_binding.context.omitted_field_list_hash
            != export_binding.context.omitted_field_list_hash
        {
            return ProviderBindingGateCheck::with_pair(
                "rejected",
                "binding_omitted_field_list_hash_mismatch",
                request_event.sequence,
                request_binding,
                export_event_id,
                export_binding,
            );
        }
        if request_binding.development_tls_bypass {
            return ProviderBindingGateCheck::with_pair(
                "rejected",
                "binding_trust_bypass_record",
                request_event.sequence,
                request_binding,
                export_event_id,
                export_binding,
            );
        }
        if !positive_provider_trust(request_binding.provider_trust_state)
            || !positive_provider_trust(export_binding.provider_trust_state)
        {
            return ProviderBindingGateCheck::with_pair(
                "rejected",
                "binding_provider_trust_not_positive",
                request_event.sequence,
                request_binding,
                export_event_id,
                export_binding,
            );
        }
        if export_binding.context_attached_to_provider_body {
            return ProviderBindingGateCheck::with_pair(
                "rejected",
                "binding_context_already_attached",
                request_event.sequence,
                request_binding,
                export_event_id,
                export_binding,
            );
        }

        ProviderBindingGateCheck {
            status: "valid",
            reason: "binding_pair_valid_for_gate_evaluation",
            request_binding_event_id: Some(EventId {
                sequence: request_event.sequence,
            }),
            export_audit_binding_event_id: Some(export_event_id),
            request_envelope_event_id: Some(EventId {
                sequence: envelope_event.sequence,
            }),
            request_binding: Some(request_binding),
            export_audit_binding: Some(export_binding),
            consumed: false,
            retained: true,
        }
    }

    fn consume_provider_context_binding_gate(
        &mut self,
        context: ProviderContextHashes,
    ) -> (ProviderBindingGateCheck, Option<EventId>) {
        let check = self.check_provider_context_binding_gate(context);
        if check.status != "valid" {
            return (check, None);
        }

        let Some(request_binding) = check.request_binding else {
            return (check, None);
        };
        let Some(export_binding) = check.export_audit_binding else {
            return (check, None);
        };
        let Some(request_binding_event_id) = check.request_binding_event_id else {
            return (check, None);
        };
        let Some(export_audit_binding_event_id) = check.export_audit_binding_event_id else {
            return (check, None);
        };

        self.consumed_bindings[self.next_consumed_slot] = Some(ConsumedProviderBinding {
            request_binding_event_id,
            export_audit_binding_event_id,
        });
        self.next_consumed_slot = (self.next_consumed_slot + 1) % EVENT_CAPACITY;
        self.consumed_len = usize::min(self.consumed_len + 1, EVENT_CAPACITY);

        let event_id = self.record(Event {
            sequence: 0,
            kind: "provider_context_export.binding_consumption_checked",
            source_method: "provider.context_export",
            source_transport: "serial-console",
            classification: "local_only",
            outcome: "checked_not_exported",
            requested_capability: "cap.provider.context_export",
            risk: "export",
            subject: "agent.session.serial",
            resource: "svc.provider.openai_direct",
            reason: "provider_binding_consumed_without_body_attachment",
            evidence: PROVIDER_BINDING_CONSUMPTION_EVIDENCE,
            bindings: EventBindings::ProviderBindingConsumption(ProviderBindingConsumption {
                request_id: export_binding.request_id,
                request_envelope_event_id: export_binding.request_envelope_event_id,
                request_binding_event_id,
                export_audit_binding_event_id,
                request_binding_hash: request_binding.request_binding_hash,
                export_audit_binding_hash: export_binding.export_audit_binding_hash,
                context: export_binding.context,
            }),
        });
        (check, Some(event_id))
    }

    pub(crate) fn check_provider_context_injection_gate(
        &self,
        _context: ProviderContextHashes,
        current_provider_trust_state: &'static str,
    ) -> ProviderContextInjectionGateCheck {
        let Some((authorization_event_id, authorization)) =
            self.latest_context_injection_authorization()
        else {
            return ProviderContextInjectionGateCheck::missing();
        };

        let Some(consumption_event) =
            self.event_by_sequence(authorization.binding_consumption_event_id)
        else {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_stale_or_dropped_event_id",
                authorization_event_id,
                authorization,
            );
        };

        let EventBindings::ProviderBindingConsumption(consumption) = consumption_event.bindings
        else {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_wrong_schema_or_variant",
                authorization_event_id,
                authorization,
            );
        };

        if authorization.request_id != consumption.request_id {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_substituted_record",
                authorization_event_id,
                authorization,
            );
        }
        if authorization.request_envelope_event_id.sequence()
            != consumption.request_envelope_event_id.sequence()
            || authorization.request_binding_event_id.sequence()
                != consumption.request_binding_event_id.sequence()
            || authorization.export_audit_binding_event_id.sequence()
                != consumption.export_audit_binding_event_id.sequence()
        {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_substituted_record",
                authorization_event_id,
                authorization,
            );
        }
        if authorization.request_binding_hash != consumption.request_binding_hash {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_substituted_record",
                authorization_event_id,
                authorization,
            );
        }
        if authorization.export_audit_binding_hash != consumption.export_audit_binding_hash {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_substituted_record",
                authorization_event_id,
                authorization,
            );
        }
        if authorization.context.projected_packet_hash != consumption.context.projected_packet_hash
        {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_substituted_record",
                authorization_event_id,
                authorization,
            );
        }
        if authorization.context.exported_field_list_hash
            != consumption.context.exported_field_list_hash
        {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_substituted_record",
                authorization_event_id,
                authorization,
            );
        }
        if authorization.context.omitted_field_list_hash
            != consumption.context.omitted_field_list_hash
        {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_substituted_record",
                authorization_event_id,
                authorization,
            );
        }

        let Some(request_event) = self.event_by_sequence(authorization.request_binding_event_id)
        else {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_stale_or_dropped_event_id",
                authorization_event_id,
                authorization,
            );
        };
        let EventBindings::ProviderRequestBound(request_binding) = request_event.bindings else {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_wrong_schema_or_variant",
                authorization_event_id,
                authorization,
            );
        };
        let Some(export_event) =
            self.event_by_sequence(authorization.export_audit_binding_event_id)
        else {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_stale_or_dropped_event_id",
                authorization_event_id,
                authorization,
            );
        };
        let EventBindings::ProviderExportAuditBound(export_binding) = export_event.bindings else {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_wrong_schema_or_variant",
                authorization_event_id,
                authorization,
            );
        };

        if authorization.request_body_hash != request_binding.request_body_hash
            || authorization.request_body_hash != export_binding.request_body_hash
        {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_prewrite_body_hash_mismatch",
                authorization_event_id,
                authorization,
            );
        }
        if authorization.request_envelope_hash != request_binding.request_envelope_hash
            || authorization.request_envelope_hash != export_binding.request_envelope_hash
        {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_substituted_record",
                authorization_event_id,
                authorization,
            );
        }
        if authorization.context.projected_packet_hash
            != request_binding.context.projected_packet_hash
            || authorization.context.projected_packet_hash
                != export_binding.context.projected_packet_hash
        {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_substituted_record",
                authorization_event_id,
                authorization,
            );
        }
        if authorization.context.exported_field_list_hash
            != request_binding.context.exported_field_list_hash
            || authorization.context.exported_field_list_hash
                != export_binding.context.exported_field_list_hash
        {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_substituted_record",
                authorization_event_id,
                authorization,
            );
        }
        if authorization.context.omitted_field_list_hash
            != request_binding.context.omitted_field_list_hash
            || authorization.context.omitted_field_list_hash
                != export_binding.context.omitted_field_list_hash
        {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_injection_authorization_substituted_record",
                authorization_event_id,
                authorization,
            );
        }
        if authorization.context_attached_to_provider_body
            || export_binding.context_attached_to_provider_body
        {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "body_attachment_without_final_authorization",
                authorization_event_id,
                authorization,
            );
        }
        if request_binding.development_tls_bypass
            || !positive_provider_trust(request_binding.provider_trust_state)
            || !positive_provider_trust(export_binding.provider_trust_state)
            || !positive_provider_trust(authorization.provider_trust_state)
        {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_provider_trust_downgraded_before_write",
                authorization_event_id,
                authorization,
            );
        }
        if !positive_provider_trust(current_provider_trust_state)
            || current_provider_trust_state != authorization.provider_trust_state
        {
            return ProviderContextInjectionGateCheck::with_authorization(
                "rejected",
                "final_provider_trust_downgraded_before_write",
                authorization_event_id,
                authorization,
            );
        }

        ProviderContextInjectionGateCheck {
            status: "blocked",
            reason: "automatic_context_injection_disabled",
            authorization_event_id: Some(authorization_event_id),
            binding_consumption_event_id: Some(authorization.binding_consumption_event_id),
            retained: true,
            can_attach_context: false,
            satisfies_current_boot_export_gate: false,
        }
    }

    fn latest_export_audit_binding(&self) -> Option<(EventId, ProviderExportAuditBinding)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::ProviderExportAuditBound(binding) = event.bindings {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_context_injection_authorization(
        &self,
    ) -> Option<(EventId, ProviderContextInjectionAuthorization)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::ProviderContextInjectionAuthorization(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_module_manifest_reference(&self) -> Option<(EventId, ModuleManifestReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::ModuleManifestReference(binding) = event.bindings {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_artifact_identity_reference(
        &self,
    ) -> Option<(EventId, RecoveryArtifactIdentityReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryArtifactIdentityReference(binding) = event.bindings {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_artifact_trust_reference(
        &self,
    ) -> Option<(EventId, RecoveryArtifactTrustReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryArtifactTrustReference(binding) = event.bindings {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_artifact_vm_test_reference(
        &self,
    ) -> Option<(EventId, RecoveryArtifactVmTestReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryArtifactVmTestReference(binding) = event.bindings {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_artifact_local_approval_reference(
        &self,
    ) -> Option<(EventId, RecoveryArtifactLocalApprovalReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryArtifactLocalApprovalReference(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_artifact_loader_reference(
        &self,
    ) -> Option<(EventId, RecoveryArtifactLoaderReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryArtifactLoaderReference(binding) = event.bindings {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_artifact_rollback_evidence_reference(
        &self,
    ) -> Option<(EventId, RecoveryArtifactRollbackEvidenceReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryArtifactRollbackEvidenceReference(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_lifeline_request_reference(
        &self,
    ) -> Option<(EventId, RecoveryLifelineRequestReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryLifelineRequestReference(binding) = event.bindings {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_lifeline_command_envelope_reference(
        &self,
    ) -> Option<(EventId, RecoveryLifelineCommandEnvelopeReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryLifelineCommandEnvelopeReference(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_lifeline_command_body_canonicalization_reference(
        &self,
    ) -> Option<(
        EventId,
        RecoveryLifelineCommandBodyCanonicalizationReference,
    )> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryLifelineCommandBodyCanonicalizationReference(
                    binding,
                ) = event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_lifeline_command_handler_binding_reference(
        &self,
    ) -> Option<(EventId, RecoveryLifelineCommandHandlerBindingReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryLifelineCommandHandlerBindingReference(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_lifeline_status_read_handler_reference(
        &self,
    ) -> Option<(EventId, RecoveryLifelineStatusReadHandlerReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryLifelineStatusReadHandlerReference(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_rollback_preview_authorization_reference(
        &self,
    ) -> Option<(EventId, RecoveryRollbackPreviewAuthorizationReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryRollbackPreviewAuthorizationReference(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_rollback_apply_authorization_reference(
        &self,
    ) -> Option<(EventId, RecoveryRollbackApplyAuthorizationReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryRollbackApplyAuthorizationReference(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_disable_module_target_binding_reference(
        &self,
    ) -> Option<(EventId, RecoveryDisableModuleTargetBindingReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryDisableModuleTargetBindingReference(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_restart_last_good_target_binding_reference(
        &self,
    ) -> Option<(EventId, RecoveryRestartLastGoodTargetBindingReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryRestartLastGoodTargetBindingReference(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_load_artifact_by_hash_target_binding_reference(
        &self,
    ) -> Option<(EventId, RecoveryLoadArtifactByHashTargetBindingReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryLoadArtifactByHashTargetBindingReference(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_memory_write_authority_reference(
        &self,
    ) -> Option<(EventId, RecoveryMemoryWriteAuthorityReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryMemoryWriteAuthorityReference(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_durable_audit_rollback_write_authority_reference(
        &self,
    ) -> Option<(EventId, DurableAuditRollbackWriteAuthorityReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::DurableAuditRollbackWriteAuthorityReference(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_service_inventory_side_effect_boundary_reference(
        &self,
    ) -> Option<(EventId, RecoveryServiceInventorySideEffectBoundaryReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryServiceInventorySideEffectBoundaryReference(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_lifeline_command_dispatch_behavior_reference(
        &self,
    ) -> Option<(EventId, RecoveryLifelineCommandDispatchBehaviorReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryLifelineCommandDispatchBehaviorReference(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_lifeline_command_executor_capability_table_reference(
        &self,
    ) -> Option<(
        EventId,
        RecoveryLifelineCommandExecutorCapabilityTableReference,
    )> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryLifelineCommandExecutorCapabilityTableReference(
                    binding,
                ) = event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_lifeline_command_side_effect_gate_reference(
        &self,
    ) -> Option<(EventId, RecoveryLifelineCommandSideEffectGateReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryLifelineCommandSideEffectGateReference(binding) =
                    event.bindings
                {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_recovery_lifeline_command_execution_stage_reference(
        &self,
        schema: &'static str,
    ) -> Option<(EventId, RecoveryLifelineCommandExecutionStageReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::RecoveryLifelineCommandExecutionStageReference(binding) =
                    event.bindings
                {
                    if binding.schema == schema {
                        return Some((
                            EventId {
                                sequence: event.sequence,
                            },
                            binding,
                        ));
                    }
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_module_candidate_artifact_reference(
        &self,
    ) -> Option<(EventId, ModuleCandidateArtifactReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::ModuleCandidateArtifactReference(binding) = event.bindings {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_module_vm_test_report_reference(
        &self,
    ) -> Option<(EventId, ModuleVmTestReportReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::ModuleVmTestReportReference(binding) = event.bindings {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_module_local_attestation_reference(
        &self,
    ) -> Option<(EventId, ModuleLocalAttestationReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::ModuleLocalAttestationReference(binding) = event.bindings {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_module_local_approval_reference(
        &self,
    ) -> Option<(EventId, ModuleLocalApprovalReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::ModuleLocalApprovalReference(binding) = event.bindings {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_module_computed_grant_reference(
        &self,
    ) -> Option<(EventId, ModuleComputedGrantReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::ModuleComputedGrantReference(binding) = event.bindings {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_module_audit_rollback_reference(
        &self,
    ) -> Option<(EventId, ModuleAuditRollbackReference)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::ModuleAuditRollbackReference(binding) = event.bindings {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_module_service_slot_reservation(
        &self,
    ) -> Option<(EventId, ModuleServiceSlotReservation)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::ModuleServiceSlotReservation(binding) = event.bindings {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn latest_module_loader_identity_source_evidence(
        &self,
    ) -> Option<(EventId, ModuleLoaderIdentitySourceEvidence)> {
        let mut idx = 0usize;
        while idx < self.len {
            let source = if self.next_slot > idx {
                self.next_slot - idx - 1
            } else {
                EVENT_CAPACITY + self.next_slot - idx - 1
            };
            if let Some(event) = self.events[source] {
                if let EventBindings::ModuleLoaderIdentitySourceEvidence(binding) = event.bindings {
                    return Some((
                        EventId {
                            sequence: event.sequence,
                        },
                        binding,
                    ));
                }
            }
            idx += 1;
        }
        None
    }

    fn check_module_manifest_reference_for_load(
        &self,
        manifest: Option<(EventId, ModuleManifestReference)>,
        retained: Option<(EventId, ModuleComputedGrantReference)>,
    ) -> ModuleManifestReferenceGateCheck {
        let Some((manifest_event_id, manifest_reference)) = manifest else {
            return ModuleManifestReferenceGateCheck {
                event_id: None,
                reference: None,
                status: "missing",
                reason: "retained_module_manifest_reference_missing",
            };
        };

        let Some(manifest_event) = self.event_by_sequence(manifest_event_id) else {
            return ModuleManifestReferenceGateCheck {
                event_id: Some(manifest_event_id),
                reference: Some(manifest_reference),
                status: "rejected",
                reason: "retained_module_manifest_reference_stale_or_dropped_event_id",
            };
        };
        let EventBindings::ModuleManifestReference(manifest_event_reference) =
            manifest_event.bindings
        else {
            return ModuleManifestReferenceGateCheck {
                event_id: Some(manifest_event_id),
                reference: Some(manifest_reference),
                status: "rejected",
                reason: "retained_module_manifest_reference_wrong_schema_or_variant",
            };
        };
        if !module_manifest_reference_matches(manifest_reference, manifest_event_reference) {
            return ModuleManifestReferenceGateCheck {
                event_id: Some(manifest_event_id),
                reference: Some(manifest_reference),
                status: "rejected",
                reason: "retained_module_manifest_reference_substituted_record",
            };
        }
        if !module_manifest_reference_hashes_consistent(manifest_reference) {
            return ModuleManifestReferenceGateCheck {
                event_id: Some(manifest_event_id),
                reference: Some(manifest_reference),
                status: "rejected",
                reason: "retained_module_manifest_reference_hash_mismatch",
            };
        }
        if let Some((_, retained_reference)) = retained {
            if manifest_reference.manifest_hash != retained_reference.manifest_hash {
                return ModuleManifestReferenceGateCheck {
                    event_id: Some(manifest_event_id),
                    reference: Some(manifest_reference),
                    status: "rejected",
                    reason: "retained_module_manifest_reference_computed_grant_mismatch",
                };
            }
        }

        ModuleManifestReferenceGateCheck {
            event_id: Some(manifest_event_id),
            reference: Some(manifest_reference),
            status: "retained_hash_reference_only",
            reason: "retained_module_manifest_reference_not_authorizing",
        }
    }

    fn check_module_candidate_artifact_reference_for_load(
        &self,
        artifact: Option<(EventId, ModuleCandidateArtifactReference)>,
        manifest: Option<(EventId, ModuleManifestReference)>,
        retained: Option<(EventId, ModuleComputedGrantReference)>,
    ) -> ModuleCandidateArtifactReferenceGateCheck {
        let Some((artifact_event_id, artifact_reference)) = artifact else {
            return ModuleCandidateArtifactReferenceGateCheck {
                event_id: None,
                reference: None,
                status: "missing",
                reason: "retained_candidate_artifact_reference_missing",
            };
        };

        let Some(artifact_event) = self.event_by_sequence(artifact_event_id) else {
            return ModuleCandidateArtifactReferenceGateCheck {
                event_id: Some(artifact_event_id),
                reference: Some(artifact_reference),
                status: "rejected",
                reason: "retained_candidate_artifact_reference_stale_or_dropped_event_id",
            };
        };
        let EventBindings::ModuleCandidateArtifactReference(artifact_event_reference) =
            artifact_event.bindings
        else {
            return ModuleCandidateArtifactReferenceGateCheck {
                event_id: Some(artifact_event_id),
                reference: Some(artifact_reference),
                status: "rejected",
                reason: "retained_candidate_artifact_reference_wrong_schema_or_variant",
            };
        };
        if !module_candidate_artifact_reference_matches(
            artifact_reference,
            artifact_event_reference,
        ) {
            return ModuleCandidateArtifactReferenceGateCheck {
                event_id: Some(artifact_event_id),
                reference: Some(artifact_reference),
                status: "rejected",
                reason: "retained_candidate_artifact_reference_substituted_record",
            };
        }
        if !module_candidate_artifact_reference_hashes_consistent(artifact_reference) {
            return ModuleCandidateArtifactReferenceGateCheck {
                event_id: Some(artifact_event_id),
                reference: Some(artifact_reference),
                status: "rejected",
                reason: "retained_candidate_artifact_reference_hash_mismatch",
            };
        }
        let Some((manifest_event_id, manifest_reference)) = manifest else {
            return ModuleCandidateArtifactReferenceGateCheck {
                event_id: Some(artifact_event_id),
                reference: Some(artifact_reference),
                status: "rejected",
                reason: "retained_candidate_artifact_reference_manifest_reference_mismatch",
            };
        };
        if artifact_reference.retained_manifest_reference_event_id != manifest_event_id
            || artifact_reference.manifest_reference_hash
                != manifest_reference.manifest_reference_hash
            || artifact_reference.manifest_hash != manifest_reference.manifest_hash
        {
            return ModuleCandidateArtifactReferenceGateCheck {
                event_id: Some(artifact_event_id),
                reference: Some(artifact_reference),
                status: "rejected",
                reason: "retained_candidate_artifact_reference_manifest_reference_mismatch",
            };
        }
        let Some((retained_event_id, retained_reference)) = retained else {
            return ModuleCandidateArtifactReferenceGateCheck {
                event_id: Some(artifact_event_id),
                reference: Some(artifact_reference),
                status: "rejected",
                reason: "retained_candidate_artifact_reference_computed_grant_reference_mismatch",
            };
        };
        if artifact_reference.retained_reference_event_id != retained_event_id
            || artifact_reference.computed_grant_hash != retained_reference.computed_grant_hash
            || artifact_reference.manifest_hash != retained_reference.manifest_hash
            || artifact_reference.vm_report_hash != retained_reference.vm_report_hash
            || artifact_reference.local_attestation_hash
                != retained_reference.local_attestation_hash
        {
            return ModuleCandidateArtifactReferenceGateCheck {
                event_id: Some(artifact_event_id),
                reference: Some(artifact_reference),
                status: "rejected",
                reason: "retained_candidate_artifact_reference_computed_grant_reference_mismatch",
            };
        }
        if artifact_reference.artifact_hash != retained_reference.artifact_hash {
            return ModuleCandidateArtifactReferenceGateCheck {
                event_id: Some(artifact_event_id),
                reference: Some(artifact_reference),
                status: "rejected",
                reason: "retained_candidate_artifact_hash_mismatch",
            };
        }

        ModuleCandidateArtifactReferenceGateCheck {
            event_id: Some(artifact_event_id),
            reference: Some(artifact_reference),
            status: "retained_hash_reference_only",
            reason: "retained_candidate_artifact_reference_not_authorizing",
        }
    }

    fn check_module_vm_test_report_reference_for_load(
        &self,
        report: Option<(EventId, ModuleVmTestReportReference)>,
        manifest: Option<(EventId, ModuleManifestReference)>,
        artifact: Option<(EventId, ModuleCandidateArtifactReference)>,
        retained: Option<(EventId, ModuleComputedGrantReference)>,
    ) -> ModuleVmTestReportReferenceGateCheck {
        let Some((report_event_id, report_reference)) = report else {
            return ModuleVmTestReportReferenceGateCheck {
                event_id: None,
                reference: None,
                status: "missing",
                reason: "retained_vm_test_report_reference_missing",
            };
        };

        let Some(report_event) = self.event_by_sequence(report_event_id) else {
            return ModuleVmTestReportReferenceGateCheck {
                event_id: Some(report_event_id),
                reference: Some(report_reference),
                status: "rejected",
                reason: "retained_vm_test_report_reference_stale_or_dropped_event_id",
            };
        };
        let EventBindings::ModuleVmTestReportReference(report_event_reference) =
            report_event.bindings
        else {
            return ModuleVmTestReportReferenceGateCheck {
                event_id: Some(report_event_id),
                reference: Some(report_reference),
                status: "rejected",
                reason: "retained_vm_test_report_reference_wrong_schema_or_variant",
            };
        };
        if !module_vm_test_report_reference_matches(report_reference, report_event_reference) {
            return ModuleVmTestReportReferenceGateCheck {
                event_id: Some(report_event_id),
                reference: Some(report_reference),
                status: "rejected",
                reason: "retained_vm_test_report_reference_substituted_record",
            };
        }
        if !module_vm_test_report_reference_hashes_consistent(report_reference) {
            return ModuleVmTestReportReferenceGateCheck {
                event_id: Some(report_event_id),
                reference: Some(report_reference),
                status: "rejected",
                reason: "retained_vm_test_report_reference_hash_mismatch",
            };
        }
        let Some((manifest_event_id, manifest_reference)) = manifest else {
            return ModuleVmTestReportReferenceGateCheck {
                event_id: Some(report_event_id),
                reference: Some(report_reference),
                status: "rejected",
                reason: "retained_vm_test_report_reference_manifest_reference_mismatch",
            };
        };
        if report_reference.retained_manifest_reference_event_id != manifest_event_id
            || report_reference.manifest_reference_hash
                != manifest_reference.manifest_reference_hash
            || report_reference.manifest_hash != manifest_reference.manifest_hash
        {
            return ModuleVmTestReportReferenceGateCheck {
                event_id: Some(report_event_id),
                reference: Some(report_reference),
                status: "rejected",
                reason: "retained_vm_test_report_reference_manifest_reference_mismatch",
            };
        }
        let Some((artifact_event_id, artifact_reference)) = artifact else {
            return ModuleVmTestReportReferenceGateCheck {
                event_id: Some(report_event_id),
                reference: Some(report_reference),
                status: "rejected",
                reason: "retained_vm_test_report_reference_artifact_reference_mismatch",
            };
        };
        if report_reference.retained_artifact_reference_event_id != artifact_event_id
            || report_reference.artifact_reference_hash
                != artifact_reference.artifact_reference_hash
            || report_reference.manifest_reference_hash
                != artifact_reference.manifest_reference_hash
            || report_reference.manifest_hash != artifact_reference.manifest_hash
            || report_reference.artifact_hash != artifact_reference.artifact_hash
            || report_reference.local_attestation_hash != artifact_reference.local_attestation_hash
        {
            return ModuleVmTestReportReferenceGateCheck {
                event_id: Some(report_event_id),
                reference: Some(report_reference),
                status: "rejected",
                reason: "retained_vm_test_report_reference_artifact_reference_mismatch",
            };
        }
        if report_reference.vm_report_hash != artifact_reference.vm_report_hash {
            return ModuleVmTestReportReferenceGateCheck {
                event_id: Some(report_event_id),
                reference: Some(report_reference),
                status: "rejected",
                reason: "retained_vm_test_report_hash_mismatch",
            };
        }
        let Some((retained_event_id, retained_reference)) = retained else {
            return ModuleVmTestReportReferenceGateCheck {
                event_id: Some(report_event_id),
                reference: Some(report_reference),
                status: "rejected",
                reason: "retained_vm_test_report_reference_computed_grant_reference_mismatch",
            };
        };
        if report_reference.retained_reference_event_id != retained_event_id
            || report_reference.computed_grant_hash != retained_reference.computed_grant_hash
            || report_reference.manifest_hash != retained_reference.manifest_hash
            || report_reference.artifact_hash != retained_reference.artifact_hash
            || report_reference.local_attestation_hash != retained_reference.local_attestation_hash
        {
            return ModuleVmTestReportReferenceGateCheck {
                event_id: Some(report_event_id),
                reference: Some(report_reference),
                status: "rejected",
                reason: "retained_vm_test_report_reference_computed_grant_reference_mismatch",
            };
        }
        if report_reference.vm_report_hash != retained_reference.vm_report_hash {
            return ModuleVmTestReportReferenceGateCheck {
                event_id: Some(report_event_id),
                reference: Some(report_reference),
                status: "rejected",
                reason: "retained_vm_test_report_hash_mismatch",
            };
        }

        ModuleVmTestReportReferenceGateCheck {
            event_id: Some(report_event_id),
            reference: Some(report_reference),
            status: "retained_hash_reference_only",
            reason: "retained_vm_test_report_reference_not_authorizing",
        }
    }

    fn check_module_local_attestation_reference_for_load(
        &self,
        attestation: Option<(EventId, ModuleLocalAttestationReference)>,
        manifest: Option<(EventId, ModuleManifestReference)>,
        artifact: Option<(EventId, ModuleCandidateArtifactReference)>,
        report: Option<(EventId, ModuleVmTestReportReference)>,
        retained: Option<(EventId, ModuleComputedGrantReference)>,
    ) -> ModuleLocalAttestationReferenceGateCheck {
        let Some((attestation_event_id, attestation_reference)) = attestation else {
            return ModuleLocalAttestationReferenceGateCheck {
                event_id: None,
                reference: None,
                status: "missing",
                reason: "retained_local_attestation_reference_missing",
            };
        };

        let Some(attestation_event) = self.event_by_sequence(attestation_event_id) else {
            return ModuleLocalAttestationReferenceGateCheck {
                event_id: Some(attestation_event_id),
                reference: Some(attestation_reference),
                status: "rejected",
                reason: "retained_local_attestation_reference_stale_or_dropped_event_id",
            };
        };
        let EventBindings::ModuleLocalAttestationReference(attestation_event_reference) =
            attestation_event.bindings
        else {
            return ModuleLocalAttestationReferenceGateCheck {
                event_id: Some(attestation_event_id),
                reference: Some(attestation_reference),
                status: "rejected",
                reason: "retained_local_attestation_reference_wrong_schema_or_variant",
            };
        };
        if !module_local_attestation_reference_matches(
            attestation_reference,
            attestation_event_reference,
        ) {
            return ModuleLocalAttestationReferenceGateCheck {
                event_id: Some(attestation_event_id),
                reference: Some(attestation_reference),
                status: "rejected",
                reason: "retained_local_attestation_reference_substituted_record",
            };
        }
        if !module_local_attestation_reference_hashes_consistent(attestation_reference) {
            return ModuleLocalAttestationReferenceGateCheck {
                event_id: Some(attestation_event_id),
                reference: Some(attestation_reference),
                status: "rejected",
                reason: "retained_local_attestation_reference_hash_mismatch",
            };
        }

        let Some((manifest_event_id, manifest_reference)) = manifest else {
            return ModuleLocalAttestationReferenceGateCheck {
                event_id: Some(attestation_event_id),
                reference: Some(attestation_reference),
                status: "rejected",
                reason: "retained_local_attestation_reference_manifest_reference_mismatch",
            };
        };
        if attestation_reference.retained_manifest_reference_event_id != manifest_event_id
            || attestation_reference.manifest_reference_hash
                != manifest_reference.manifest_reference_hash
            || attestation_reference.manifest_hash != manifest_reference.manifest_hash
        {
            return ModuleLocalAttestationReferenceGateCheck {
                event_id: Some(attestation_event_id),
                reference: Some(attestation_reference),
                status: "rejected",
                reason: "retained_local_attestation_reference_manifest_reference_mismatch",
            };
        }

        let Some((artifact_event_id, artifact_reference)) = artifact else {
            return ModuleLocalAttestationReferenceGateCheck {
                event_id: Some(attestation_event_id),
                reference: Some(attestation_reference),
                status: "rejected",
                reason: "retained_local_attestation_reference_artifact_reference_mismatch",
            };
        };
        if attestation_reference.retained_artifact_reference_event_id != artifact_event_id
            || attestation_reference.artifact_reference_hash
                != artifact_reference.artifact_reference_hash
            || attestation_reference.manifest_reference_hash
                != artifact_reference.manifest_reference_hash
            || attestation_reference.manifest_hash != artifact_reference.manifest_hash
            || attestation_reference.artifact_hash != artifact_reference.artifact_hash
            || attestation_reference.local_attestation_hash
                != artifact_reference.local_attestation_hash
        {
            return ModuleLocalAttestationReferenceGateCheck {
                event_id: Some(attestation_event_id),
                reference: Some(attestation_reference),
                status: "rejected",
                reason: "retained_local_attestation_reference_artifact_reference_mismatch",
            };
        }

        let Some((report_event_id, report_reference)) = report else {
            return ModuleLocalAttestationReferenceGateCheck {
                event_id: Some(attestation_event_id),
                reference: Some(attestation_reference),
                status: "rejected",
                reason: "retained_local_attestation_reference_vm_report_reference_mismatch",
            };
        };
        if attestation_reference.retained_vm_report_reference_event_id != report_event_id
            || attestation_reference.vm_report_reference_hash
                != report_reference.report_reference_hash
            || attestation_reference.manifest_reference_hash
                != report_reference.manifest_reference_hash
            || attestation_reference.artifact_reference_hash
                != report_reference.artifact_reference_hash
            || attestation_reference.manifest_hash != report_reference.manifest_hash
            || attestation_reference.artifact_hash != report_reference.artifact_hash
            || attestation_reference.vm_report_hash != report_reference.vm_report_hash
            || attestation_reference.local_attestation_hash
                != report_reference.local_attestation_hash
        {
            return ModuleLocalAttestationReferenceGateCheck {
                event_id: Some(attestation_event_id),
                reference: Some(attestation_reference),
                status: "rejected",
                reason: "retained_local_attestation_reference_vm_report_reference_mismatch",
            };
        }

        let Some((retained_event_id, retained_reference)) = retained else {
            return ModuleLocalAttestationReferenceGateCheck {
                event_id: Some(attestation_event_id),
                reference: Some(attestation_reference),
                status: "rejected",
                reason: "retained_local_attestation_reference_computed_grant_reference_mismatch",
            };
        };
        if attestation_reference.retained_reference_event_id != retained_event_id
            || attestation_reference.computed_grant_hash != retained_reference.computed_grant_hash
            || attestation_reference.manifest_hash != retained_reference.manifest_hash
            || attestation_reference.artifact_hash != retained_reference.artifact_hash
            || attestation_reference.vm_report_hash != retained_reference.vm_report_hash
            || attestation_reference.local_attestation_hash
                != retained_reference.local_attestation_hash
        {
            return ModuleLocalAttestationReferenceGateCheck {
                event_id: Some(attestation_event_id),
                reference: Some(attestation_reference),
                status: "rejected",
                reason: "retained_local_attestation_reference_computed_grant_reference_mismatch",
            };
        }

        ModuleLocalAttestationReferenceGateCheck {
            event_id: Some(attestation_event_id),
            reference: Some(attestation_reference),
            status: "retained_hash_reference_only",
            reason: "retained_local_attestation_reference_not_authorizing",
        }
    }

    fn check_module_local_approval_reference_for_load(
        &self,
        approval: Option<(EventId, ModuleLocalApprovalReference)>,
        manifest: Option<(EventId, ModuleManifestReference)>,
        artifact: Option<(EventId, ModuleCandidateArtifactReference)>,
        report: Option<(EventId, ModuleVmTestReportReference)>,
        attestation: ModuleLocalAttestationReferenceGateCheck,
        retained: Option<(EventId, ModuleComputedGrantReference)>,
    ) -> ModuleLocalApprovalReferenceGateCheck {
        let Some((approval_event_id, approval_reference)) = approval else {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: None,
                reference: None,
                status: "missing",
                reason: "retained_local_approval_reference_missing",
            };
        };

        let Some(approval_event) = self.event_by_sequence(approval_event_id) else {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: Some(approval_event_id),
                reference: Some(approval_reference),
                status: "rejected",
                reason: "retained_local_approval_reference_stale_or_dropped_event_id",
            };
        };
        let EventBindings::ModuleLocalApprovalReference(approval_event_reference) =
            approval_event.bindings
        else {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: Some(approval_event_id),
                reference: Some(approval_reference),
                status: "rejected",
                reason: "retained_local_approval_reference_wrong_schema_or_variant",
            };
        };
        if !module_local_approval_reference_matches(approval_reference, approval_event_reference) {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: Some(approval_event_id),
                reference: Some(approval_reference),
                status: "rejected",
                reason: "retained_local_approval_reference_substituted_record",
            };
        }
        if !module_local_approval_reference_hashes_consistent(approval_reference) {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: Some(approval_event_id),
                reference: Some(approval_reference),
                status: "rejected",
                reason: "retained_local_approval_reference_hash_mismatch",
            };
        }

        let Some((manifest_event_id, manifest_reference)) = manifest else {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: Some(approval_event_id),
                reference: Some(approval_reference),
                status: "rejected",
                reason: "retained_local_approval_reference_manifest_reference_mismatch",
            };
        };
        if approval_reference.retained_manifest_reference_event_id != manifest_event_id
            || approval_reference.manifest_reference_hash
                != manifest_reference.manifest_reference_hash
            || approval_reference.manifest_hash != manifest_reference.manifest_hash
        {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: Some(approval_event_id),
                reference: Some(approval_reference),
                status: "rejected",
                reason: "retained_local_approval_reference_manifest_reference_mismatch",
            };
        }

        let Some((artifact_event_id, artifact_reference)) = artifact else {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: Some(approval_event_id),
                reference: Some(approval_reference),
                status: "rejected",
                reason: "retained_local_approval_reference_artifact_reference_mismatch",
            };
        };
        if approval_reference.retained_artifact_reference_event_id != artifact_event_id
            || approval_reference.artifact_reference_hash
                != artifact_reference.artifact_reference_hash
            || approval_reference.manifest_reference_hash
                != artifact_reference.manifest_reference_hash
            || approval_reference.manifest_hash != artifact_reference.manifest_hash
            || approval_reference.artifact_hash != artifact_reference.artifact_hash
            || approval_reference.local_attestation_hash
                != artifact_reference.local_attestation_hash
        {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: Some(approval_event_id),
                reference: Some(approval_reference),
                status: "rejected",
                reason: "retained_local_approval_reference_artifact_reference_mismatch",
            };
        }

        let Some((report_event_id, report_reference)) = report else {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: Some(approval_event_id),
                reference: Some(approval_reference),
                status: "rejected",
                reason: "retained_local_approval_reference_vm_report_reference_mismatch",
            };
        };
        if approval_reference.retained_vm_report_reference_event_id != report_event_id
            || approval_reference.vm_report_reference_hash != report_reference.report_reference_hash
            || approval_reference.artifact_reference_hash
                != report_reference.artifact_reference_hash
            || approval_reference.vm_report_hash != report_reference.vm_report_hash
            || approval_reference.local_attestation_hash != report_reference.local_attestation_hash
        {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: Some(approval_event_id),
                reference: Some(approval_reference),
                status: "rejected",
                reason: "retained_local_approval_reference_vm_report_reference_mismatch",
            };
        }

        if attestation.status != "retained_hash_reference_only" {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: Some(approval_event_id),
                reference: Some(approval_reference),
                status: "rejected",
                reason: "retained_local_approval_reference_local_attestation_reference_mismatch",
            };
        }
        let (Some(attestation_event_id), Some(attestation_reference)) =
            (attestation.event_id, attestation.reference)
        else {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: Some(approval_event_id),
                reference: Some(approval_reference),
                status: "rejected",
                reason: "retained_local_approval_reference_local_attestation_reference_mismatch",
            };
        };
        if approval_reference.retained_local_attestation_reference_event_id != attestation_event_id
            || approval_reference.local_attestation_reference_hash
                != attestation_reference.attestation_reference_hash
            || approval_reference.vm_report_reference_hash
                != attestation_reference.vm_report_reference_hash
            || approval_reference.local_attestation_hash
                != attestation_reference.local_attestation_hash
        {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: Some(approval_event_id),
                reference: Some(approval_reference),
                status: "rejected",
                reason: "retained_local_approval_reference_local_attestation_reference_mismatch",
            };
        }

        let Some((retained_event_id, retained_reference)) = retained else {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: Some(approval_event_id),
                reference: Some(approval_reference),
                status: "rejected",
                reason: "retained_local_approval_reference_computed_grant_reference_mismatch",
            };
        };
        if approval_reference.retained_reference_event_id != retained_event_id
            || approval_reference.computed_grant_hash != retained_reference.computed_grant_hash
            || approval_reference.manifest_hash != retained_reference.manifest_hash
            || approval_reference.artifact_hash != retained_reference.artifact_hash
            || approval_reference.vm_report_hash != retained_reference.vm_report_hash
            || approval_reference.local_attestation_hash
                != retained_reference.local_attestation_hash
        {
            return ModuleLocalApprovalReferenceGateCheck {
                event_id: Some(approval_event_id),
                reference: Some(approval_reference),
                status: "rejected",
                reason: "retained_local_approval_reference_computed_grant_reference_mismatch",
            };
        }

        ModuleLocalApprovalReferenceGateCheck {
            event_id: Some(approval_event_id),
            reference: Some(approval_reference),
            status: "retained_hash_reference_only",
            reason: "retained_local_approval_reference_not_authorizing",
        }
    }

    fn check_module_audit_rollback_reference_for_load(
        &self,
        retained: Option<(EventId, ModuleComputedGrantReference)>,
        audit_rollback: Option<(EventId, ModuleAuditRollbackReference)>,
    ) -> ModuleAuditRollbackReferenceGateCheck {
        let Some((audit_rollback_event_id, audit_rollback_reference)) = audit_rollback else {
            return ModuleAuditRollbackReferenceGateCheck {
                event_id: None,
                reference: None,
                status: "missing",
                reason: "retained_audit_rollback_reference_missing",
            };
        };

        let Some((retained_reference_event_id, retained_reference)) = retained else {
            return ModuleAuditRollbackReferenceGateCheck {
                event_id: Some(audit_rollback_event_id),
                reference: Some(audit_rollback_reference),
                status: "rejected",
                reason: "retained_computed_grant_reference_missing",
            };
        };

        if audit_rollback_reference.retained_reference_event_id != retained_reference_event_id {
            return ModuleAuditRollbackReferenceGateCheck {
                event_id: Some(audit_rollback_event_id),
                reference: Some(audit_rollback_reference),
                status: "rejected",
                reason: "retained_audit_rollback_reference_substituted_record",
            };
        }

        let Some(retained_event) =
            self.event_by_sequence(audit_rollback_reference.retained_reference_event_id)
        else {
            return ModuleAuditRollbackReferenceGateCheck {
                event_id: Some(audit_rollback_event_id),
                reference: Some(audit_rollback_reference),
                status: "rejected",
                reason: "retained_audit_rollback_reference_stale_or_dropped_event_id",
            };
        };
        let EventBindings::ModuleComputedGrantReference(retained_event_reference) =
            retained_event.bindings
        else {
            return ModuleAuditRollbackReferenceGateCheck {
                event_id: Some(audit_rollback_event_id),
                reference: Some(audit_rollback_reference),
                status: "rejected",
                reason: "retained_audit_rollback_reference_wrong_schema_or_variant",
            };
        };
        if !module_computed_grant_reference_matches(retained_reference, retained_event_reference) {
            return ModuleAuditRollbackReferenceGateCheck {
                event_id: Some(audit_rollback_event_id),
                reference: Some(audit_rollback_reference),
                status: "rejected",
                reason: "retained_audit_rollback_reference_substituted_record",
            };
        }
        if !module_computed_grant_reference_hashes_consistent(retained_reference) {
            return ModuleAuditRollbackReferenceGateCheck {
                event_id: Some(audit_rollback_event_id),
                reference: Some(audit_rollback_reference),
                status: "rejected",
                reason: "retained_audit_rollback_computed_grant_hash_mismatch",
            };
        }
        if !module_audit_rollback_binds_computed_grant(audit_rollback_reference, retained_reference)
        {
            return ModuleAuditRollbackReferenceGateCheck {
                event_id: Some(audit_rollback_event_id),
                reference: Some(audit_rollback_reference),
                status: "rejected",
                reason: "retained_audit_rollback_computed_grant_hash_mismatch",
            };
        }
        if !module_evidence::ram_only_service_slot_id_valid(
            audit_rollback_reference.ram_only_service_slot_id.as_str(),
        ) {
            return ModuleAuditRollbackReferenceGateCheck {
                event_id: Some(audit_rollback_event_id),
                reference: Some(audit_rollback_reference),
                status: "rejected",
                reason: "retained_audit_rollback_service_slot_mismatch",
            };
        }
        if let Some(reason) =
            module_audit_rollback_reference_hash_mismatch(audit_rollback_reference)
        {
            return ModuleAuditRollbackReferenceGateCheck {
                event_id: Some(audit_rollback_event_id),
                reference: Some(audit_rollback_reference),
                status: "rejected",
                reason,
            };
        }

        if audit_rollback_reference.denial_event_id.sequence() >= audit_rollback_event_id.sequence()
        {
            return ModuleAuditRollbackReferenceGateCheck {
                event_id: Some(audit_rollback_event_id),
                reference: Some(audit_rollback_reference),
                status: "rejected",
                reason: "retained_audit_rollback_reference_substituted_record",
            };
        }

        let Some(denial_event) = self.event_by_sequence(audit_rollback_reference.denial_event_id)
        else {
            return ModuleAuditRollbackReferenceGateCheck {
                event_id: Some(audit_rollback_event_id),
                reference: Some(audit_rollback_reference),
                status: "rejected",
                reason: "retained_audit_rollback_reference_stale_or_dropped_event_id",
            };
        };
        let EventBindings::ModuleLoadGate(denial_binding) = denial_event.bindings else {
            return ModuleAuditRollbackReferenceGateCheck {
                event_id: Some(audit_rollback_event_id),
                reference: Some(audit_rollback_reference),
                status: "rejected",
                reason: "retained_audit_rollback_reference_wrong_schema_or_variant",
            };
        };
        if denial_binding.retained_reference_event_id != Some(retained_reference_event_id) {
            return ModuleAuditRollbackReferenceGateCheck {
                event_id: Some(audit_rollback_event_id),
                reference: Some(audit_rollback_reference),
                status: "rejected",
                reason: "retained_audit_rollback_reference_substituted_record",
            };
        }

        ModuleAuditRollbackReferenceGateCheck {
            event_id: Some(audit_rollback_event_id),
            reference: Some(audit_rollback_reference),
            status: "retained_hash_reference_only",
            reason: "retained_audit_rollback_reference_not_authorizing",
        }
    }

    fn check_module_service_slot_reservation_for_load(
        &self,
        retained: Option<(EventId, ModuleComputedGrantReference)>,
        audit_rollback_check: ModuleAuditRollbackReferenceGateCheck,
        service_slot: Option<(EventId, ModuleServiceSlotReservation)>,
    ) -> ModuleServiceSlotReservationGateCheck {
        let Some((reservation_event_id, reservation)) = service_slot else {
            return ModuleServiceSlotReservationGateCheck {
                event_id: None,
                reservation: None,
                status: "missing",
                reason: "retained_service_slot_reservation_missing",
            };
        };

        let Some((retained_reference_event_id, retained_reference)) = retained else {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_computed_grant_reference_missing",
            };
        };

        if audit_rollback_check.status != "retained_hash_reference_only" {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: if audit_rollback_check.event_id.is_some() {
                    "retained_audit_rollback_reference_not_valid_for_service_slot"
                } else {
                    "retained_audit_rollback_reference_missing"
                },
            };
        }

        let Some(audit_rollback_event_id) = audit_rollback_check.event_id else {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_audit_rollback_reference_missing",
            };
        };
        let Some(audit_rollback_reference) = audit_rollback_check.reference else {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_audit_rollback_reference_missing",
            };
        };

        if reservation.retained_reference_event_id != retained_reference_event_id {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_service_slot_reservation_grant_reference_mismatch",
            };
        }
        if reservation.retained_audit_rollback_reference_event_id != audit_rollback_event_id {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_service_slot_reservation_audit_rollback_reference_mismatch",
            };
        }

        let Some(retained_event) = self.event_by_sequence(reservation.retained_reference_event_id)
        else {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_service_slot_reservation_stale_or_dropped_event_id",
            };
        };
        let EventBindings::ModuleComputedGrantReference(retained_event_reference) =
            retained_event.bindings
        else {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_service_slot_reservation_wrong_schema_or_variant",
            };
        };
        if !module_computed_grant_reference_matches(retained_reference, retained_event_reference) {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_service_slot_reservation_substituted_record",
            };
        }

        let Some(audit_event) =
            self.event_by_sequence(reservation.retained_audit_rollback_reference_event_id)
        else {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_service_slot_reservation_stale_or_dropped_event_id",
            };
        };
        let EventBindings::ModuleAuditRollbackReference(audit_event_reference) =
            audit_event.bindings
        else {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_service_slot_reservation_wrong_schema_or_variant",
            };
        };
        if !module_audit_rollback_reference_matches(audit_rollback_reference, audit_event_reference)
        {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_service_slot_reservation_substituted_record",
            };
        }

        if reservation.computed_grant_hash != retained_reference.computed_grant_hash
            || reservation.computed_grant_hash != audit_rollback_reference.computed_grant_hash
        {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_service_slot_reservation_computed_grant_hash_mismatch",
            };
        }
        if reservation.audit_record_hash != audit_rollback_reference.audit_record_hash {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_service_slot_reservation_audit_record_hash_mismatch",
            };
        }
        if reservation.rollback_plan_hash != audit_rollback_reference.rollback_plan_hash {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_service_slot_reservation_rollback_plan_hash_mismatch",
            };
        }
        if reservation.pre_load_service_inventory_hash
            != audit_rollback_reference.pre_load_service_inventory_hash
        {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_service_slot_reservation_pre_load_inventory_hash_mismatch",
            };
        }
        if reservation.ram_only_service_slot_id.as_str()
            != audit_rollback_reference.ram_only_service_slot_id.as_str()
            || !module_evidence::ram_only_service_slot_id_valid(
                reservation.ram_only_service_slot_id.as_str(),
            )
        {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason: "retained_service_slot_reservation_service_slot_mismatch",
            };
        }
        if let Some(reason) = module_service_slot_reservation_hash_mismatch(reservation) {
            return ModuleServiceSlotReservationGateCheck {
                event_id: Some(reservation_event_id),
                reservation: Some(reservation),
                status: "rejected",
                reason,
            };
        }

        ModuleServiceSlotReservationGateCheck {
            event_id: Some(reservation_event_id),
            reservation: Some(reservation),
            status: "retained_hash_reference_only_not_allocated",
            reason: "retained_service_slot_reservation_not_allocated",
        }
    }

    fn module_load_gate_binding(&self) -> ModuleLoadGateBinding {
        let manifest_reference = self.latest_module_manifest_reference();
        let artifact_reference = self.latest_module_candidate_artifact_reference();
        let vm_report_reference = self.latest_module_vm_test_report_reference();
        let attestation_reference = self.latest_module_local_attestation_reference();
        let approval_reference = self.latest_module_local_approval_reference();
        let retained = self.latest_module_computed_grant_reference();
        let audit_rollback = self.latest_module_audit_rollback_reference();
        let service_slot = self.latest_module_service_slot_reservation();
        let manifest_check =
            self.check_module_manifest_reference_for_load(manifest_reference, retained);
        let artifact_check = self.check_module_candidate_artifact_reference_for_load(
            artifact_reference,
            manifest_reference,
            retained,
        );
        let vm_report_check = self.check_module_vm_test_report_reference_for_load(
            vm_report_reference,
            manifest_reference,
            artifact_reference,
            retained,
        );
        let attestation_check = self.check_module_local_attestation_reference_for_load(
            attestation_reference,
            manifest_reference,
            artifact_reference,
            vm_report_reference,
            retained,
        );
        let approval_check = self.check_module_local_approval_reference_for_load(
            approval_reference,
            manifest_reference,
            artifact_reference,
            vm_report_reference,
            attestation_check,
            retained,
        );
        let audit_rollback_check =
            self.check_module_audit_rollback_reference_for_load(retained, audit_rollback);
        let service_slot_check = self.check_module_service_slot_reservation_for_load(
            retained,
            audit_rollback_check,
            service_slot,
        );

        ModuleLoadGateBinding {
            manifest_reference_event_id: manifest_check.event_id,
            manifest_reference: manifest_check.reference,
            manifest_reference_status: manifest_check.status,
            manifest_reference_reason: manifest_check.reason,
            artifact_reference_event_id: artifact_check.event_id,
            artifact_reference: artifact_check.reference,
            artifact_reference_status: artifact_check.status,
            artifact_reference_reason: artifact_check.reason,
            vm_report_reference_event_id: vm_report_check.event_id,
            vm_report_reference: vm_report_check.reference,
            vm_report_reference_status: vm_report_check.status,
            vm_report_reference_reason: vm_report_check.reason,
            attestation_reference_event_id: attestation_check.event_id,
            attestation_reference: attestation_check.reference,
            attestation_reference_status: attestation_check.status,
            attestation_reference_reason: attestation_check.reason,
            approval_reference_event_id: approval_check.event_id,
            approval_reference: approval_check.reference,
            approval_reference_status: approval_check.status,
            approval_reference_reason: approval_check.reason,
            retained_reference_event_id: retained.map(|(event_id, _)| event_id),
            retained_reference: retained.map(|(_, reference)| reference),
            audit_rollback_reference_event_id: audit_rollback_check.event_id,
            audit_rollback_reference: audit_rollback_check.reference,
            audit_rollback_reference_status: audit_rollback_check.status,
            audit_rollback_reference_reason: audit_rollback_check.reason,
            service_slot_reservation_event_id: service_slot_check.event_id,
            service_slot_reservation: service_slot_check.reservation,
            service_slot_reservation_status: service_slot_check.status,
            service_slot_reservation_reason: service_slot_check.reason,
        }
    }

    fn event_by_sequence(&self, event_id: EventId) -> Option<Event> {
        let mut idx = 0usize;
        while idx < EVENT_CAPACITY {
            if let Some(event) = self.events[idx] {
                if event.sequence == event_id.sequence() {
                    return Some(event);
                }
            }
            idx += 1;
        }
        None
    }

    fn binding_consumed(
        &self,
        request_binding_event_id: EventId,
        export_audit_binding_event_id: EventId,
    ) -> bool {
        let mut idx = 0usize;
        while idx < self.consumed_len {
            if let Some(consumed) = self.consumed_bindings[idx] {
                if consumed.request_binding_event_id.sequence()
                    == request_binding_event_id.sequence()
                    && consumed.export_audit_binding_event_id.sequence()
                        == export_audit_binding_event_id.sequence()
                {
                    return true;
                }
            }
            idx += 1;
        }
        false
    }
}

impl ProviderBindingGateCheck {
    const fn rejected(status: &'static str, reason: &'static str) -> Self {
        Self {
            status,
            reason,
            request_binding_event_id: None,
            export_audit_binding_event_id: None,
            request_envelope_event_id: None,
            request_binding: None,
            export_audit_binding: None,
            consumed: false,
            retained: false,
        }
    }

    fn with_export(
        status: &'static str,
        reason: &'static str,
        export_audit_binding_event_id: EventId,
        export_audit_binding: ProviderExportAuditBinding,
    ) -> Self {
        Self {
            status,
            reason,
            request_binding_event_id: Some(export_audit_binding.request_binding_event_id),
            export_audit_binding_event_id: Some(export_audit_binding_event_id),
            request_envelope_event_id: Some(export_audit_binding.request_envelope_event_id),
            request_binding: None,
            export_audit_binding: Some(export_audit_binding),
            consumed: false,
            retained: true,
        }
    }

    fn with_pair(
        status: &'static str,
        reason: &'static str,
        request_binding_sequence: u64,
        request_binding: ProviderRequestBinding,
        export_audit_binding_event_id: EventId,
        export_audit_binding: ProviderExportAuditBinding,
    ) -> Self {
        Self {
            status,
            reason,
            request_binding_event_id: Some(EventId {
                sequence: request_binding_sequence,
            }),
            export_audit_binding_event_id: Some(export_audit_binding_event_id),
            request_envelope_event_id: Some(request_binding.request_envelope_event_id),
            request_binding: Some(request_binding),
            export_audit_binding: Some(export_audit_binding),
            consumed: false,
            retained: true,
        }
    }
}

impl ProviderContextInjectionGateCheck {
    const fn missing() -> Self {
        Self {
            status: "missing",
            reason: "final_injection_authorization_missing",
            authorization_event_id: None,
            binding_consumption_event_id: None,
            retained: false,
            can_attach_context: false,
            satisfies_current_boot_export_gate: false,
        }
    }

    fn with_authorization(
        status: &'static str,
        reason: &'static str,
        authorization_event_id: EventId,
        authorization: ProviderContextInjectionAuthorization,
    ) -> Self {
        Self {
            status,
            reason,
            authorization_event_id: Some(authorization_event_id),
            binding_consumption_event_id: Some(authorization.binding_consumption_event_id),
            retained: true,
            can_attach_context: false,
            satisfies_current_boot_export_gate: false,
        }
    }
}

fn positive_provider_trust(trust_state: &str) -> bool {
    matches!(
        trust_state,
        "pinned_cert_verified" | "pinned_spki_verified" | "webpki_verified"
    )
}

pub fn record_agent_read(
    source_method: &'static str,
    requested_capability: &'static str,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "agent_protocol.read_response",
        source_method,
        source_transport: "serial-console",
        classification: "public",
        outcome: "response",
        requested_capability,
        risk: "observe",
        subject: "agent.session.serial",
        resource: "current_boot",
        reason: "granted_read",
        evidence: READ_EVIDENCE,
        bindings: EventBindings::None,
    })
}

pub fn record_capability_denied(
    source_method: &'static str,
    requested_capability: &'static str,
    risk: &'static str,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "agent_protocol.capability_denied",
        source_method,
        source_transport: "serial-console",
        classification: "public",
        outcome: "capability_denied",
        requested_capability,
        risk,
        subject: "agent.session.serial",
        resource: "current_boot",
        reason: "missing_evidence",
        evidence: DENIED_EVIDENCE,
        bindings: EventBindings::None,
    })
}

pub fn record_module_load_ephemeral_denied(
    source_method: &'static str,
) -> (EventId, ModuleLoadGateBinding) {
    let mut log = LOG.lock();
    let binding = log.module_load_gate_binding();
    let event_id = log.record(Event {
        sequence: 0,
        kind: "agent_protocol.capability_denied",
        source_method,
        source_transport: "serial-console",
        classification: "public",
        outcome: "capability_denied",
        requested_capability: "cap.module.load_ephemeral",
        risk: "modify_ram",
        subject: "agent.session.serial",
        resource: "live_service_graph",
        reason: "missing_evidence",
        evidence: MODULE_LOAD_GATE_EVIDENCE,
        bindings: EventBindings::ModuleLoadGate(binding),
    });
    (event_id, binding)
}

pub fn record_recovery_artifact_load_denied(source_method: &'static str) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "agent_protocol.capability_denied",
        source_method,
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "capability_denied",
        requested_capability: "cap.recovery.load_artifact",
        risk: "recovery_modify_ram",
        subject: "agent.session.serial",
        resource: "recovery_lifeline",
        reason: "missing_recovery_artifact_evidence",
        evidence: RECOVERY_ARTIFACT_LOAD_DENIAL_EVIDENCE,
        bindings: EventBindings::RecoveryArtifactLoadDenied(RecoveryArtifactLoadDenialBinding {
            recovery_artifact_identity_missing: true,
            recovery_artifact_trust_missing: true,
            recovery_vm_test_missing: true,
            recovery_local_approval_missing: true,
            recovery_loader_missing: true,
            recovery_rollback_evidence_missing: true,
        }),
    })
}

pub fn module_load_gate_binding_snapshot() -> ModuleLoadGateBinding {
    LOG.lock().module_load_gate_binding()
}

pub fn record_recovery_artifact_identity_reference(
    binding: RecoveryArtifactIdentityReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.artifact_identity_reference.retained",
        source_method: "recovery.identity_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_load_still_denied",
        requested_capability: "cap.recovery.load_artifact.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_lifeline",
        reason: "recovery_artifact_identity_reference_valid_for_current_boot",
        evidence: RECOVERY_ARTIFACT_IDENTITY_REFERENCE_EVIDENCE,
        bindings: EventBindings::RecoveryArtifactIdentityReference(binding),
    })
}

pub fn record_recovery_artifact_trust_reference(
    binding: RecoveryArtifactTrustReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.artifact_trust_reference.retained",
        source_method: "recovery.trust_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_load_still_denied",
        requested_capability: "cap.recovery.load_artifact.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_lifeline",
        reason: "recovery_artifact_trust_reference_valid_for_current_boot",
        evidence: RECOVERY_ARTIFACT_TRUST_REFERENCE_EVIDENCE,
        bindings: EventBindings::RecoveryArtifactTrustReference(binding),
    })
}

pub fn record_recovery_artifact_vm_test_reference(
    binding: RecoveryArtifactVmTestReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.artifact_vm_test_reference.retained",
        source_method: "recovery.vm_test_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_load_still_denied",
        requested_capability: "cap.recovery.load_artifact.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_lifeline",
        reason: "recovery_artifact_vm_test_reference_valid_for_current_boot",
        evidence: RECOVERY_ARTIFACT_VM_TEST_REFERENCE_EVIDENCE,
        bindings: EventBindings::RecoveryArtifactVmTestReference(binding),
    })
}

pub fn record_recovery_artifact_local_approval_reference(
    binding: RecoveryArtifactLocalApprovalReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.artifact_local_approval_reference.retained",
        source_method: "recovery.local_approval_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_load_still_denied",
        requested_capability: "cap.recovery.load_artifact.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_lifeline",
        reason: "recovery_artifact_local_approval_reference_valid_for_current_boot",
        evidence: RECOVERY_ARTIFACT_LOCAL_APPROVAL_REFERENCE_EVIDENCE,
        bindings: EventBindings::RecoveryArtifactLocalApprovalReference(binding),
    })
}

pub fn record_recovery_artifact_loader_reference(
    binding: RecoveryArtifactLoaderReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.artifact_loader_reference.retained",
        source_method: "recovery.loader_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_load_still_denied",
        requested_capability: "cap.recovery.load_artifact.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_lifeline",
        reason: "recovery_artifact_loader_reference_valid_for_current_boot",
        evidence: RECOVERY_ARTIFACT_LOADER_REFERENCE_EVIDENCE,
        bindings: EventBindings::RecoveryArtifactLoaderReference(binding),
    })
}

pub fn record_recovery_artifact_rollback_evidence_reference(
    binding: RecoveryArtifactRollbackEvidenceReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.artifact_rollback_evidence_reference.retained",
        source_method: "recovery.rollback_evidence_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_load_still_denied",
        requested_capability: "cap.recovery.load_artifact.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_lifeline",
        reason: "recovery_artifact_rollback_evidence_reference_valid_for_current_boot",
        evidence: RECOVERY_ARTIFACT_ROLLBACK_EVIDENCE_REFERENCE_EVIDENCE,
        bindings: EventBindings::RecoveryArtifactRollbackEvidenceReference(binding),
    })
}

pub fn record_recovery_lifeline_request_reference(
    binding: RecoveryLifelineRequestReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.lifeline_request_reference.retained",
        source_method: "recovery.lifeline_request_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_load_still_denied",
        requested_capability: "cap.recovery.load_artifact.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_lifeline",
        reason: "recovery_lifeline_request_reference_valid_for_current_boot",
        evidence: RECOVERY_LIFELINE_REQUEST_REFERENCE_EVIDENCE,
        bindings: EventBindings::RecoveryLifelineRequestReference(binding),
    })
}

pub fn record_recovery_lifeline_command_envelope_reference(
    binding: RecoveryLifelineCommandEnvelopeReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.lifeline_command_envelope_reference.retained",
        source_method: "recovery.lifeline_command_envelope_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_lifeline_command",
        reason: "recovery_lifeline_command_envelope_reference_valid_for_current_boot",
        evidence: RECOVERY_LIFELINE_COMMAND_ENVELOPE_REFERENCE_EVIDENCE,
        bindings: EventBindings::RecoveryLifelineCommandEnvelopeReference(binding),
    })
}

pub fn record_recovery_lifeline_command_body_canonicalization_reference(
    binding: RecoveryLifelineCommandBodyCanonicalizationReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.lifeline_command_body_canonicalization.retained",
        source_method: "recovery.lifeline_command_body_canonicalization_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_lifeline_command_body",
        reason: "recovery_lifeline_command_body_canonicalization_valid_for_current_boot",
        evidence: RECOVERY_LIFELINE_COMMAND_BODY_CANONICALIZATION_EVIDENCE,
        bindings: EventBindings::RecoveryLifelineCommandBodyCanonicalizationReference(binding),
    })
}

pub fn record_recovery_lifeline_command_handler_binding_reference(
    binding: RecoveryLifelineCommandHandlerBindingReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.lifeline_command_handler_binding.retained",
        source_method: "recovery.lifeline_command_handler_binding_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_lifeline_command_handler",
        reason: "recovery_lifeline_command_handler_binding_valid_for_current_boot",
        evidence: RECOVERY_LIFELINE_COMMAND_HANDLER_BINDING_EVIDENCE,
        bindings: EventBindings::RecoveryLifelineCommandHandlerBindingReference(binding),
    })
}

pub fn record_recovery_lifeline_status_read_handler_reference(
    binding: RecoveryLifelineStatusReadHandlerReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.lifeline_status_read_handler.retained",
        source_method: "recovery.lifeline_status_read_handler_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_lifeline_status_read_handler",
        reason: "recovery_lifeline_status_read_handler_valid_for_current_boot",
        evidence: RECOVERY_LIFELINE_STATUS_READ_HANDLER_EVIDENCE,
        bindings: EventBindings::RecoveryLifelineStatusReadHandlerReference(binding),
    })
}

pub fn record_recovery_rollback_preview_authorization_reference(
    binding: RecoveryRollbackPreviewAuthorizationReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.rollback_preview_authorization.retained",
        source_method: "recovery.rollback_preview_authorization_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_rollback_preview_authorization",
        reason: "recovery_rollback_preview_authorization_valid_for_current_boot",
        evidence: RECOVERY_ROLLBACK_PREVIEW_AUTHORIZATION_EVIDENCE,
        bindings: EventBindings::RecoveryRollbackPreviewAuthorizationReference(binding),
    })
}

pub fn record_recovery_rollback_apply_authorization_reference(
    binding: RecoveryRollbackApplyAuthorizationReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.rollback_apply_authorization.retained",
        source_method: "recovery.rollback_apply_authorization_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_rollback_apply_authorization",
        reason: "recovery_rollback_apply_authorization_valid_for_current_boot",
        evidence: RECOVERY_ROLLBACK_APPLY_AUTHORIZATION_EVIDENCE,
        bindings: EventBindings::RecoveryRollbackApplyAuthorizationReference(binding),
    })
}

pub fn record_recovery_disable_module_target_binding_reference(
    binding: RecoveryDisableModuleTargetBindingReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.disable_module_target_binding.retained",
        source_method: "recovery.disable_module_target_binding_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_disable_module_target_binding",
        reason: "recovery_disable_module_target_binding_valid_for_current_boot",
        evidence: RECOVERY_DISABLE_MODULE_TARGET_BINDING_EVIDENCE,
        bindings: EventBindings::RecoveryDisableModuleTargetBindingReference(binding),
    })
}

pub fn record_recovery_restart_last_good_target_binding_reference(
    binding: RecoveryRestartLastGoodTargetBindingReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.restart_last_good_target_binding.retained",
        source_method: "recovery.restart_last_good_target_binding_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_restart_last_good_target_binding",
        reason: "recovery_restart_last_good_target_binding_valid_for_current_boot",
        evidence: RECOVERY_RESTART_LAST_GOOD_TARGET_BINDING_EVIDENCE,
        bindings: EventBindings::RecoveryRestartLastGoodTargetBindingReference(binding),
    })
}

pub fn record_recovery_load_artifact_by_hash_target_binding_reference(
    binding: RecoveryLoadArtifactByHashTargetBindingReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.load_artifact_by_hash_target_binding.retained",
        source_method: "recovery.load_artifact_by_hash_target_binding_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_load_artifact_by_hash_target_binding",
        reason: "recovery_load_artifact_by_hash_target_binding_valid_for_current_boot",
        evidence: RECOVERY_LOAD_ARTIFACT_BY_HASH_TARGET_BINDING_EVIDENCE,
        bindings: EventBindings::RecoveryLoadArtifactByHashTargetBindingReference(binding),
    })
}

pub fn record_recovery_memory_write_authority_reference(
    binding: RecoveryMemoryWriteAuthorityReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.memory_write_authority.retained",
        source_method: "recovery.memory_write_authority_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_memory_write_authority",
        reason: "recovery_memory_write_authority_valid_for_current_boot",
        evidence: RECOVERY_MEMORY_WRITE_AUTHORITY_EVIDENCE,
        bindings: EventBindings::RecoveryMemoryWriteAuthorityReference(binding),
    })
}

pub fn record_durable_audit_rollback_write_authority_reference(
    binding: DurableAuditRollbackWriteAuthorityReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.durable_audit_rollback_write_authority.retained",
        source_method: "recovery.durable_audit_rollback_write_authority_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "durable_audit_rollback_write_authority",
        reason: "durable_audit_rollback_write_authority_valid_for_current_boot",
        evidence: DURABLE_AUDIT_ROLLBACK_WRITE_AUTHORITY_EVIDENCE,
        bindings: EventBindings::DurableAuditRollbackWriteAuthorityReference(binding),
    })
}

pub fn record_recovery_service_inventory_side_effect_boundary_reference(
    binding: RecoveryServiceInventorySideEffectBoundaryReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.service_inventory_side_effect_boundary.retained",
        source_method: "recovery.service_inventory_side_effect_boundary_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "service_inventory_side_effect_boundary",
        reason: "recovery_service_inventory_side_effect_boundary_valid_for_current_boot",
        evidence: RECOVERY_SERVICE_INVENTORY_SIDE_EFFECT_BOUNDARY_EVIDENCE,
        bindings: EventBindings::RecoveryServiceInventorySideEffectBoundaryReference(binding),
    })
}

pub fn record_recovery_lifeline_command_dispatch_behavior_reference(
    binding: RecoveryLifelineCommandDispatchBehaviorReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.lifeline_command_dispatch_behavior.retained",
        source_method: "recovery.lifeline_command_dispatch_behavior_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_lifeline_command_dispatch_behavior",
        reason: "recovery_lifeline_command_dispatch_behavior_valid_for_current_boot",
        evidence: RECOVERY_LIFELINE_COMMAND_DISPATCH_BEHAVIOR_EVIDENCE,
        bindings: EventBindings::RecoveryLifelineCommandDispatchBehaviorReference(binding),
    })
}

pub fn record_recovery_lifeline_command_executor_capability_table_reference(
    binding: RecoveryLifelineCommandExecutorCapabilityTableReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.lifeline_command_executor_capability_table.retained",
        source_method: "recovery.lifeline_command_executor_capability_table_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_lifeline_command_executor_capability_table",
        reason: "recovery_lifeline_command_executor_capability_table_valid_for_current_boot",
        evidence: RECOVERY_LIFELINE_COMMAND_EXECUTOR_CAPABILITY_TABLE_EVIDENCE,
        bindings: EventBindings::RecoveryLifelineCommandExecutorCapabilityTableReference(binding),
    })
}

pub fn record_recovery_lifeline_command_side_effect_gate_reference(
    binding: RecoveryLifelineCommandSideEffectGateReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "recovery.lifeline_command_side_effect_gate.retained",
        source_method: "recovery.lifeline_command_side_effect_gate_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "recovery_lifeline_command_side_effect_gate",
        reason: "recovery_lifeline_command_side_effect_gate_valid_for_current_boot",
        evidence: RECOVERY_LIFELINE_COMMAND_SIDE_EFFECT_GATE_EVIDENCE,
        bindings: EventBindings::RecoveryLifelineCommandSideEffectGateReference(binding),
    })
}

pub fn record_recovery_lifeline_command_execution_stage_reference(
    binding: RecoveryLifelineCommandExecutionStageReference,
) -> EventId {
    let (kind, source_method, resource, reason) =
        recovery_lifeline_command_execution_stage_event_metadata(binding);
    LOG.lock().record(Event {
        sequence: 0,
        kind,
        source_method,
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_command_still_denied",
        requested_capability: "cap.recovery.command.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource,
        reason,
        evidence: RECOVERY_LIFELINE_COMMAND_EXECUTION_STAGE_EVIDENCE,
        bindings: EventBindings::RecoveryLifelineCommandExecutionStageReference(binding),
    })
}

fn recovery_lifeline_command_execution_stage_event_metadata(
    binding: RecoveryLifelineCommandExecutionStageReference,
) -> (&'static str, &'static str, &'static str, &'static str) {
    if binding.stage_name == "execution_preflight" {
        (
            "recovery.lifeline_command_execution_preflight.retained",
            "recovery.lifeline_command_execution_preflight_diagnostic",
            "recovery_lifeline_command_execution_preflight",
            "recovery_lifeline_command_execution_preflight_valid_for_current_boot",
        )
    } else if binding.stage_name == "execution_intent" {
        (
            "recovery.lifeline_command_execution_intent.retained",
            "recovery.lifeline_command_execution_intent_diagnostic",
            "recovery_lifeline_command_execution_intent",
            "recovery_lifeline_command_execution_intent_valid_for_current_boot",
        )
    } else if binding.stage_name == "execution_commit_gate" {
        (
            "recovery.lifeline_command_execution_commit_gate.retained",
            "recovery.lifeline_command_execution_commit_gate_diagnostic",
            "recovery_lifeline_command_execution_commit_gate",
            "recovery_lifeline_command_execution_commit_gate_valid_for_current_boot",
        )
    } else if binding.stage_name == "execution_result_denial" {
        (
            "recovery.lifeline_command_execution_result_denial.retained",
            "recovery.lifeline_command_execution_result_denial_diagnostic",
            "recovery_lifeline_command_execution_result_denial",
            "recovery_lifeline_command_execution_result_denial_valid_for_current_boot",
        )
    } else if binding.stage_name == "execution_audit_denial" {
        (
            "recovery.lifeline_command_execution_audit_denial.retained",
            "recovery.lifeline_command_execution_audit_denial_diagnostic",
            "recovery_lifeline_command_execution_audit_denial",
            "recovery_lifeline_command_execution_audit_denial_valid_for_current_boot",
        )
    } else if binding.stage_name == "execution_observation_denial" {
        (
            "recovery.lifeline_command_execution_observation_denial.retained",
            "recovery.lifeline_command_execution_observation_denial_diagnostic",
            "recovery_lifeline_command_execution_observation_denial",
            "recovery_lifeline_command_execution_observation_denial_valid_for_current_boot",
        )
    } else if binding.stage_name == "execution_completion_denial" {
        (
            "recovery.lifeline_command_execution_completion_denial.retained",
            "recovery.lifeline_command_execution_completion_denial_diagnostic",
            "recovery_lifeline_command_execution_completion_denial",
            "recovery_lifeline_command_execution_completion_denial_valid_for_current_boot",
        )
    } else {
        (
            "recovery.lifeline_command_execution_enablement.retained",
            "recovery.lifeline_command_execution_enablement_diagnostic",
            "recovery_lifeline_command_execution_enablement",
            "recovery_lifeline_command_execution_enablement_valid_for_current_boot",
        )
    }
}

pub fn record_module_manifest_reference(binding: ModuleManifestReference) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "module.manifest_reference.retained",
        source_method: "module.manifest_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_load_still_denied",
        requested_capability: "cap.module.grant_diagnostic.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "live_service_graph",
        reason: "module_manifest_reference_valid_for_current_boot",
        evidence: MODULE_MANIFEST_REFERENCE_EVIDENCE,
        bindings: EventBindings::ModuleManifestReference(binding),
    })
}

pub fn record_module_candidate_artifact_reference(
    binding: ModuleCandidateArtifactReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "module.artifact_reference.retained",
        source_method: "module.artifact_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_load_still_denied",
        requested_capability: "cap.module.grant_diagnostic.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "live_service_graph",
        reason: "candidate_artifact_reference_valid_for_current_boot",
        evidence: MODULE_CANDIDATE_ARTIFACT_REFERENCE_EVIDENCE,
        bindings: EventBindings::ModuleCandidateArtifactReference(binding),
    })
}

pub fn record_module_vm_test_report_reference(binding: ModuleVmTestReportReference) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "module.vm_test_report_reference.retained",
        source_method: "module.vm_report_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_load_still_denied",
        requested_capability: "cap.module.grant_diagnostic.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "live_service_graph",
        reason: "vm_test_report_reference_valid_for_current_boot",
        evidence: MODULE_VM_TEST_REPORT_REFERENCE_EVIDENCE,
        bindings: EventBindings::ModuleVmTestReportReference(binding),
    })
}

pub fn record_module_local_attestation_reference(
    binding: ModuleLocalAttestationReference,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "module.local_attestation_reference.retained",
        source_method: "module.attestation_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_load_still_denied",
        requested_capability: "cap.module.grant_diagnostic.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "live_service_graph",
        reason: "local_attestation_reference_valid_for_current_boot",
        evidence: MODULE_LOCAL_ATTESTATION_REFERENCE_EVIDENCE,
        bindings: EventBindings::ModuleLocalAttestationReference(binding),
    })
}

pub fn record_module_local_approval_reference(binding: ModuleLocalApprovalReference) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "module.local_approval_reference.retained",
        source_method: "module.approval_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_load_still_denied",
        requested_capability: "cap.module.grant_diagnostic.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "live_service_graph",
        reason: "local_approval_reference_valid_for_current_boot",
        evidence: MODULE_LOCAL_APPROVAL_REFERENCE_EVIDENCE,
        bindings: EventBindings::ModuleLocalApprovalReference(binding),
    })
}

pub fn record_module_computed_grant_reference(binding: ModuleComputedGrantReference) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "module.computed_grant_reference.retained",
        source_method: "module.grant_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_load_still_denied",
        requested_capability: "cap.module.grant_diagnostic.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "live_service_graph",
        reason: "computed_grant_reference_valid_for_current_boot",
        evidence: MODULE_COMPUTED_GRANT_REFERENCE_EVIDENCE,
        bindings: EventBindings::ModuleComputedGrantReference(binding),
    })
}

pub fn record_module_audit_rollback_reference(binding: ModuleAuditRollbackReference) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "module.audit_rollback_reference.retained",
        source_method: "module.audit_rollback_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_load_still_denied",
        requested_capability: "cap.module.grant_diagnostic.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "live_service_graph",
        reason: "audit_rollback_reference_valid_for_current_boot",
        evidence: MODULE_AUDIT_ROLLBACK_REFERENCE_EVIDENCE,
        bindings: EventBindings::ModuleAuditRollbackReference(binding),
    })
}

pub fn record_module_service_slot_reservation(binding: ModuleServiceSlotReservation) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "module.service_slot_reservation.retained",
        source_method: "module.service_slot_diagnostic",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "retained_hash_reference_load_still_denied",
        requested_capability: "cap.module.grant_diagnostic.read",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "live_service_graph",
        reason: "service_slot_reservation_valid_for_current_boot",
        evidence: MODULE_SERVICE_SLOT_RESERVATION_EVIDENCE,
        bindings: EventBindings::ModuleServiceSlotReservation(binding),
    })
}

pub fn record_module_loader_identity_source_evidence(
    binding: ModuleLoaderIdentitySourceEvidence,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "module.loader_identity.source_evidence.retained",
        source_method: "module.loader_identity",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: binding.readiness_status,
        requested_capability: "cap.module.load_ephemeral",
        risk: "observe",
        subject: "agent.session.serial",
        resource: "module.loader_runtime.identity.current_boot",
        reason: binding.readiness_reason,
        evidence: MODULE_LOADER_IDENTITY_SOURCE_EVIDENCE,
        bindings: EventBindings::ModuleLoaderIdentitySourceEvidence(binding),
    })
}

pub fn record_provider_request_binding_denied(hashes: ProviderContextHashes) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "provider_context_export.request_binding_denied",
        source_method: "provider.context_export",
        source_transport: "serial-console",
        classification: "public",
        outcome: "denied_not_bound",
        requested_capability: "cap.provider.context_export",
        risk: "export",
        subject: "agent.session.serial",
        resource: "current_boot",
        reason: "provider_request_binding_requires_real_request_envelope",
        evidence: PROVIDER_REQUEST_BINDING_DENIAL_EVIDENCE,
        bindings: EventBindings::ProviderRequestBindingDenied(hashes),
    })
}

pub fn record_provider_request_envelope_created(
    binding: ProviderRequestEnvelopeBinding,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "provider_request.envelope_created",
        source_method: "ask",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "local_prewrite_envelope",
        requested_capability: "cap.provider.request",
        risk: "export",
        subject: "agent.session.serial",
        resource: "svc.provider.openai_direct",
        reason: "provider_request_envelope_created_before_write",
        evidence: PROVIDER_REQUEST_ENVELOPE_EVIDENCE,
        bindings: EventBindings::ProviderRequestEnvelope(binding),
    })
}

pub fn record_provider_request_binding_bound(binding: ProviderRequestBinding) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "provider_context_export.request_binding_bound",
        source_method: "ask",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "bound",
        requested_capability: "cap.provider.context_export",
        risk: "export",
        subject: "agent.session.serial",
        resource: "svc.provider.openai_direct",
        reason: "provider_minimal_context_bound_to_real_request_envelope",
        evidence: PROVIDER_REQUEST_BINDING_EVIDENCE,
        bindings: EventBindings::ProviderRequestBound(binding),
    })
}

pub fn record_provider_context_export_audit_binding_bound(
    binding: ProviderExportAuditBinding,
) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "provider_context_export.audit_binding_bound",
        source_method: "ask",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "authorized_for_single_provider_request",
        requested_capability: "cap.provider.context_export",
        risk: "export",
        subject: "agent.session.serial",
        resource: "svc.provider.openai_direct",
        reason: "provider_minimal_context_export_audit_bound_without_body_attachment",
        evidence: PROVIDER_EXPORT_AUDIT_BINDING_EVIDENCE,
        bindings: EventBindings::ProviderExportAuditBound(binding),
    })
}

pub fn check_provider_context_binding_gate(
    context: ProviderContextHashes,
) -> ProviderBindingGateCheck {
    LOG.lock().check_provider_context_binding_gate(context)
}

pub fn consume_provider_context_binding_gate(
    context: ProviderContextHashes,
) -> (ProviderBindingGateCheck, Option<EventId>) {
    LOG.lock().consume_provider_context_binding_gate(context)
}

pub fn check_provider_context_injection_gate(
    context: ProviderContextHashes,
    current_provider_trust_state: &'static str,
) -> ProviderContextInjectionGateCheck {
    LOG.lock()
        .check_provider_context_injection_gate(context, current_provider_trust_state)
}

pub fn provider_context_binding_gate_selftest(
    context: ProviderContextHashes,
) -> [ProviderBindingGateSelfTestCase; PROVIDER_BINDING_GATE_SELFTEST_CASES] {
    crate::event_log_provider_selftest::provider_context_binding_gate_selftest(context)
}

pub fn provider_context_injection_gate_selftest(
    context: ProviderContextHashes,
) -> [ProviderContextInjectionGateSelfTestCase; PROVIDER_CONTEXT_INJECTION_GATE_SELFTEST_CASES] {
    crate::event_log_provider_selftest::provider_context_injection_gate_selftest(context)
}

pub fn record_provider_context_export_denial_audit(hashes: ProviderContextHashes) -> EventId {
    LOG.lock().record(Event {
        sequence: 0,
        kind: "provider_context_export.denial_audit",
        source_method: "provider.context_export",
        source_transport: "serial-console",
        classification: "public",
        outcome: "denied_no_provider_write",
        requested_capability: "cap.provider.context_export",
        risk: "export",
        subject: "agent.session.serial",
        resource: "current_boot",
        reason: "provider_context_export_not_authorized",
        evidence: PROVIDER_EXPORT_DENIAL_AUDIT_EVIDENCE,
        bindings: EventBindings::ProviderExportDenialAudit(hashes),
    })
}

pub fn snapshot_recent(limit: usize) -> EventSnapshot {
    LOG.lock().snapshot_recent(limit)
}

pub fn latest_module_manifest_reference() -> Option<(EventId, ModuleManifestReference)> {
    LOG.lock().latest_module_manifest_reference()
}

pub fn latest_recovery_artifact_identity_reference(
) -> Option<(EventId, RecoveryArtifactIdentityReference)> {
    LOG.lock().latest_recovery_artifact_identity_reference()
}

pub fn latest_recovery_artifact_trust_reference(
) -> Option<(EventId, RecoveryArtifactTrustReference)> {
    LOG.lock().latest_recovery_artifact_trust_reference()
}

pub fn latest_recovery_artifact_vm_test_reference(
) -> Option<(EventId, RecoveryArtifactVmTestReference)> {
    LOG.lock().latest_recovery_artifact_vm_test_reference()
}

pub fn latest_recovery_artifact_local_approval_reference(
) -> Option<(EventId, RecoveryArtifactLocalApprovalReference)> {
    LOG.lock()
        .latest_recovery_artifact_local_approval_reference()
}

pub fn latest_recovery_artifact_loader_reference(
) -> Option<(EventId, RecoveryArtifactLoaderReference)> {
    LOG.lock().latest_recovery_artifact_loader_reference()
}

pub fn latest_recovery_artifact_rollback_evidence_reference(
) -> Option<(EventId, RecoveryArtifactRollbackEvidenceReference)> {
    LOG.lock()
        .latest_recovery_artifact_rollback_evidence_reference()
}

pub fn latest_recovery_lifeline_request_reference(
) -> Option<(EventId, RecoveryLifelineRequestReference)> {
    LOG.lock().latest_recovery_lifeline_request_reference()
}

pub fn latest_recovery_lifeline_command_envelope_reference(
) -> Option<(EventId, RecoveryLifelineCommandEnvelopeReference)> {
    LOG.lock()
        .latest_recovery_lifeline_command_envelope_reference()
}

pub fn latest_recovery_lifeline_command_body_canonicalization_reference() -> Option<(
    EventId,
    RecoveryLifelineCommandBodyCanonicalizationReference,
)> {
    LOG.lock()
        .latest_recovery_lifeline_command_body_canonicalization_reference()
}

pub fn latest_recovery_lifeline_command_handler_binding_reference(
) -> Option<(EventId, RecoveryLifelineCommandHandlerBindingReference)> {
    LOG.lock()
        .latest_recovery_lifeline_command_handler_binding_reference()
}

pub fn latest_recovery_lifeline_status_read_handler_reference(
) -> Option<(EventId, RecoveryLifelineStatusReadHandlerReference)> {
    LOG.lock()
        .latest_recovery_lifeline_status_read_handler_reference()
}

pub fn latest_recovery_rollback_preview_authorization_reference(
) -> Option<(EventId, RecoveryRollbackPreviewAuthorizationReference)> {
    LOG.lock()
        .latest_recovery_rollback_preview_authorization_reference()
}

pub fn latest_recovery_rollback_apply_authorization_reference(
) -> Option<(EventId, RecoveryRollbackApplyAuthorizationReference)> {
    LOG.lock()
        .latest_recovery_rollback_apply_authorization_reference()
}

pub fn latest_recovery_disable_module_target_binding_reference(
) -> Option<(EventId, RecoveryDisableModuleTargetBindingReference)> {
    LOG.lock()
        .latest_recovery_disable_module_target_binding_reference()
}

pub fn latest_recovery_restart_last_good_target_binding_reference(
) -> Option<(EventId, RecoveryRestartLastGoodTargetBindingReference)> {
    LOG.lock()
        .latest_recovery_restart_last_good_target_binding_reference()
}

pub fn latest_recovery_load_artifact_by_hash_target_binding_reference(
) -> Option<(EventId, RecoveryLoadArtifactByHashTargetBindingReference)> {
    LOG.lock()
        .latest_recovery_load_artifact_by_hash_target_binding_reference()
}

pub fn latest_recovery_memory_write_authority_reference(
) -> Option<(EventId, RecoveryMemoryWriteAuthorityReference)> {
    LOG.lock()
        .latest_recovery_memory_write_authority_reference()
}

pub fn latest_durable_audit_rollback_write_authority_reference(
) -> Option<(EventId, DurableAuditRollbackWriteAuthorityReference)> {
    LOG.lock()
        .latest_durable_audit_rollback_write_authority_reference()
}

pub fn latest_recovery_service_inventory_side_effect_boundary_reference(
) -> Option<(EventId, RecoveryServiceInventorySideEffectBoundaryReference)> {
    LOG.lock()
        .latest_recovery_service_inventory_side_effect_boundary_reference()
}

pub fn latest_recovery_lifeline_command_dispatch_behavior_reference(
) -> Option<(EventId, RecoveryLifelineCommandDispatchBehaviorReference)> {
    LOG.lock()
        .latest_recovery_lifeline_command_dispatch_behavior_reference()
}

pub fn latest_recovery_lifeline_command_executor_capability_table_reference() -> Option<(
    EventId,
    RecoveryLifelineCommandExecutorCapabilityTableReference,
)> {
    LOG.lock()
        .latest_recovery_lifeline_command_executor_capability_table_reference()
}

pub fn latest_recovery_lifeline_command_side_effect_gate_reference(
) -> Option<(EventId, RecoveryLifelineCommandSideEffectGateReference)> {
    LOG.lock()
        .latest_recovery_lifeline_command_side_effect_gate_reference()
}

pub fn latest_recovery_lifeline_command_execution_stage_reference(
    schema: &'static str,
) -> Option<(EventId, RecoveryLifelineCommandExecutionStageReference)> {
    LOG.lock()
        .latest_recovery_lifeline_command_execution_stage_reference(schema)
}

pub fn latest_module_candidate_artifact_reference(
) -> Option<(EventId, ModuleCandidateArtifactReference)> {
    LOG.lock().latest_module_candidate_artifact_reference()
}

pub fn latest_module_vm_test_report_reference() -> Option<(EventId, ModuleVmTestReportReference)> {
    LOG.lock().latest_module_vm_test_report_reference()
}

pub fn latest_module_local_attestation_reference(
) -> Option<(EventId, ModuleLocalAttestationReference)> {
    LOG.lock().latest_module_local_attestation_reference()
}

pub fn latest_module_local_approval_reference() -> Option<(EventId, ModuleLocalApprovalReference)> {
    LOG.lock().latest_module_local_approval_reference()
}

pub fn latest_module_computed_grant_reference() -> Option<(EventId, ModuleComputedGrantReference)> {
    LOG.lock().latest_module_computed_grant_reference()
}

pub fn latest_module_audit_rollback_reference() -> Option<(EventId, ModuleAuditRollbackReference)> {
    LOG.lock().latest_module_audit_rollback_reference()
}

pub fn latest_module_service_slot_reservation() -> Option<(EventId, ModuleServiceSlotReservation)> {
    LOG.lock().latest_module_service_slot_reservation()
}

pub fn latest_module_loader_identity_source_evidence(
) -> Option<(EventId, ModuleLoaderIdentitySourceEvidence)> {
    LOG.lock().latest_module_loader_identity_source_evidence()
}

fn normalize_limit(limit: usize) -> usize {
    if limit == 0 {
        DEFAULT_EVENT_LIMIT
    } else {
        usize::min(limit, EVENT_CAPACITY)
    }
}
