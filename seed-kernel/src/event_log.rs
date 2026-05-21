use core::str;

use spin::Mutex;

use crate::event_log_evidence::{
    DENIED_EVIDENCE, MODULE_AUDIT_ROLLBACK_REFERENCE_EVIDENCE,
    MODULE_CANDIDATE_ARTIFACT_REFERENCE_EVIDENCE, MODULE_COMPUTED_GRANT_REFERENCE_EVIDENCE,
    MODULE_LOAD_GATE_EVIDENCE, MODULE_MANIFEST_REFERENCE_EVIDENCE,
    MODULE_SERVICE_SLOT_RESERVATION_EVIDENCE, MODULE_VM_TEST_REPORT_REFERENCE_EVIDENCE,
    PROVIDER_BINDING_CONSUMPTION_EVIDENCE, PROVIDER_CONTEXT_INJECTION_AUTHORIZATION_EVIDENCE,
    PROVIDER_EXPORT_AUDIT_BINDING_EVIDENCE, PROVIDER_EXPORT_DENIAL_AUDIT_EVIDENCE,
    PROVIDER_REQUEST_BINDING_DENIAL_EVIDENCE, PROVIDER_REQUEST_BINDING_EVIDENCE,
    PROVIDER_REQUEST_ENVELOPE_EVIDENCE, READ_EVIDENCE,
};
use crate::event_log_types::{
    ConsumedProviderBinding, ModuleAuditRollbackReferenceGateCheck,
    ModuleCandidateArtifactReferenceGateCheck, ModuleManifestReferenceGateCheck,
    ModuleServiceSlotReservationGateCheck, ModuleVmTestReportReferenceGateCheck,
};
pub use crate::event_log_types::{
    Event, EventBindings, EventId, EventSnapshot, ModuleAuditRollbackReference,
    ModuleCandidateArtifactReference, ModuleComputedGrantReference, ModuleLoadGateBinding,
    ModuleManifestReference, ModuleServiceSlotId, ModuleServiceSlotReservation,
    ModuleVmTestReportReference, ProviderBindingConsumption, ProviderBindingGateCheck,
    ProviderBindingGateSelfTestCase, ProviderContextHashes, ProviderContextInjectionAuthorization,
    ProviderContextInjectionGateCheck, ProviderContextInjectionGateSelfTestCase,
    ProviderExportAuditBinding, ProviderRequestBinding, ProviderRequestEnvelopeBinding,
    DEFAULT_EVENT_LIMIT, EVENT_CAPACITY, PROVIDER_BINDING_GATE_SELFTEST_CASES,
    PROVIDER_CONTEXT_INJECTION_GATE_SELFTEST_CASES,
};
use crate::module_evidence;

static LOG: Mutex<EventLog> = Mutex::new(EventLog::new());

struct EventLog {
    events: [Option<Event>; EVENT_CAPACITY],
    consumed_bindings: [Option<ConsumedProviderBinding>; EVENT_CAPACITY],
    next_slot: usize,
    next_consumed_slot: usize,
    len: usize,
    consumed_len: usize,
    next_sequence: u64,
}

impl EventLog {
    const fn new() -> Self {
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

    fn record(&mut self, mut event: Event) -> EventId {
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

    fn check_provider_context_binding_gate(
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

    fn check_provider_context_injection_gate(
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

fn module_computed_grant_reference_matches(
    left: ModuleComputedGrantReference,
    right: ModuleComputedGrantReference,
) -> bool {
    left.computed_grant_hash == right.computed_grant_hash
        && left.manifest_hash == right.manifest_hash
        && left.artifact_hash == right.artifact_hash
        && left.vm_report_hash == right.vm_report_hash
        && left.local_attestation_hash == right.local_attestation_hash
}

fn module_manifest_reference_matches(
    left: ModuleManifestReference,
    right: ModuleManifestReference,
) -> bool {
    left.manifest_reference_hash == right.manifest_reference_hash
        && left.manifest_hash == right.manifest_hash
}

fn module_manifest_reference_hashes_consistent(reference: ModuleManifestReference) -> bool {
    reference.manifest_reference_hash
        == module_evidence::computed_module_manifest_reference_hash(reference.manifest_hash)
}

fn module_candidate_artifact_reference_matches(
    left: ModuleCandidateArtifactReference,
    right: ModuleCandidateArtifactReference,
) -> bool {
    left.artifact_reference_hash == right.artifact_reference_hash
        && left.retained_manifest_reference_event_id == right.retained_manifest_reference_event_id
        && left.retained_reference_event_id == right.retained_reference_event_id
        && left.manifest_reference_hash == right.manifest_reference_hash
        && left.manifest_hash == right.manifest_hash
        && left.computed_grant_hash == right.computed_grant_hash
        && left.artifact_hash == right.artifact_hash
        && left.vm_report_hash == right.vm_report_hash
        && left.local_attestation_hash == right.local_attestation_hash
}

fn module_candidate_artifact_reference_hashes_consistent(
    reference: ModuleCandidateArtifactReference,
) -> bool {
    reference.artifact_reference_hash
        == module_evidence::computed_module_candidate_artifact_reference_hash_from_sequences(
            reference.retained_manifest_reference_event_id.sequence(),
            reference.retained_reference_event_id.sequence(),
            reference.manifest_reference_hash,
            reference.manifest_hash,
            reference.computed_grant_hash,
            reference.artifact_hash,
            reference.vm_report_hash,
            reference.local_attestation_hash,
        )
}

fn module_vm_test_report_reference_matches(
    left: ModuleVmTestReportReference,
    right: ModuleVmTestReportReference,
) -> bool {
    left.report_reference_hash == right.report_reference_hash
        && left.retained_manifest_reference_event_id == right.retained_manifest_reference_event_id
        && left.retained_artifact_reference_event_id == right.retained_artifact_reference_event_id
        && left.retained_reference_event_id == right.retained_reference_event_id
        && left.manifest_reference_hash == right.manifest_reference_hash
        && left.artifact_reference_hash == right.artifact_reference_hash
        && left.manifest_hash == right.manifest_hash
        && left.artifact_hash == right.artifact_hash
        && left.computed_grant_hash == right.computed_grant_hash
        && left.vm_report_hash == right.vm_report_hash
        && left.local_attestation_hash == right.local_attestation_hash
}

fn module_vm_test_report_reference_hashes_consistent(
    reference: ModuleVmTestReportReference,
) -> bool {
    reference.report_reference_hash
        == module_evidence::computed_module_vm_test_report_reference_hash_from_sequences(
            reference.retained_manifest_reference_event_id.sequence(),
            reference.retained_artifact_reference_event_id.sequence(),
            reference.retained_reference_event_id.sequence(),
            reference.manifest_reference_hash,
            reference.artifact_reference_hash,
            reference.manifest_hash,
            reference.artifact_hash,
            reference.computed_grant_hash,
            reference.vm_report_hash,
            reference.local_attestation_hash,
        )
}

fn module_computed_grant_reference_hashes_consistent(
    reference: ModuleComputedGrantReference,
) -> bool {
    reference.computed_grant_hash
        == module_evidence::computed_module_grant_hash(
            reference.manifest_hash,
            reference.artifact_hash,
            reference.vm_report_hash,
            reference.local_attestation_hash,
        )
}

fn module_audit_rollback_reference_matches(
    left: ModuleAuditRollbackReference,
    right: ModuleAuditRollbackReference,
) -> bool {
    left.audit_record_hash == right.audit_record_hash
        && left.rollback_plan_hash == right.rollback_plan_hash
        && left.computed_grant_hash == right.computed_grant_hash
        && left.manifest_hash == right.manifest_hash
        && left.artifact_hash == right.artifact_hash
        && left.vm_report_hash == right.vm_report_hash
        && left.local_attestation_hash == right.local_attestation_hash
        && left.local_approval_hash == right.local_approval_hash
        && left.pre_load_service_inventory_hash == right.pre_load_service_inventory_hash
        && left.cleanup_actions_hash == right.cleanup_actions_hash
        && left.denial_event_id == right.denial_event_id
        && left.retained_reference_event_id == right.retained_reference_event_id
        && left.ram_only_service_slot_id.as_str() == right.ram_only_service_slot_id.as_str()
}

fn module_audit_rollback_binds_computed_grant(
    audit_rollback_reference: ModuleAuditRollbackReference,
    retained_reference: ModuleComputedGrantReference,
) -> bool {
    audit_rollback_reference.computed_grant_hash == retained_reference.computed_grant_hash
        && audit_rollback_reference.manifest_hash == retained_reference.manifest_hash
        && audit_rollback_reference.artifact_hash == retained_reference.artifact_hash
        && audit_rollback_reference.vm_report_hash == retained_reference.vm_report_hash
        && audit_rollback_reference.local_attestation_hash
            == retained_reference.local_attestation_hash
}

fn module_audit_rollback_reference_hash_mismatch(
    reference: ModuleAuditRollbackReference,
) -> Option<&'static str> {
    let expected_rollback_plan_hash = module_evidence::computed_module_rollback_plan_hash(
        reference.artifact_hash,
        reference.pre_load_service_inventory_hash,
        reference.ram_only_service_slot_id.as_str(),
        reference.cleanup_actions_hash,
    );
    if reference.rollback_plan_hash != expected_rollback_plan_hash {
        return Some("retained_rollback_plan_hash_mismatch");
    }

    let mut denial_event_id = [0u8; EVENT_ID_TEXT_LEN];
    let mut retained_reference_event_id = [0u8; EVENT_ID_TEXT_LEN];
    let denial_event_id = event_id_text(reference.denial_event_id, &mut denial_event_id);
    let retained_reference_event_id = event_id_text(
        reference.retained_reference_event_id,
        &mut retained_reference_event_id,
    );
    let expected_audit_record_hash = module_evidence::computed_module_audit_record_hash(
        module_evidence::ModuleAuditRecordHashInput {
            denial_event_id,
            retained_reference_event_id,
            computed_grant_hash: reference.computed_grant_hash,
            manifest_hash: reference.manifest_hash,
            artifact_hash: reference.artifact_hash,
            vm_report_hash: reference.vm_report_hash,
            local_attestation_hash: reference.local_attestation_hash,
            local_approval_hash: reference.local_approval_hash,
            rollback_plan_hash: reference.rollback_plan_hash,
            ram_only_service_slot_id: reference.ram_only_service_slot_id.as_str(),
        },
    );
    if reference.audit_record_hash != expected_audit_record_hash {
        return Some("retained_audit_record_hash_mismatch");
    }

    None
}

fn module_service_slot_reservation_hash_mismatch(
    reservation: ModuleServiceSlotReservation,
) -> Option<&'static str> {
    let mut retained_reference_event_id = [0u8; EVENT_ID_TEXT_LEN];
    let mut retained_audit_rollback_reference_event_id = [0u8; EVENT_ID_TEXT_LEN];
    let retained_reference_event_id = event_id_text(
        reservation.retained_reference_event_id,
        &mut retained_reference_event_id,
    );
    let retained_audit_rollback_reference_event_id = event_id_text(
        reservation.retained_audit_rollback_reference_event_id,
        &mut retained_audit_rollback_reference_event_id,
    );
    let expected_reservation_hash = module_evidence::computed_module_service_slot_reservation_hash(
        module_evidence::ModuleServiceSlotReservationHashInput {
            retained_reference_event_id,
            retained_audit_rollback_reference_event_id,
            computed_grant_hash: reservation.computed_grant_hash,
            audit_record_hash: reservation.audit_record_hash,
            rollback_plan_hash: reservation.rollback_plan_hash,
            pre_load_service_inventory_hash: reservation.pre_load_service_inventory_hash,
            ram_only_service_slot_id: reservation.ram_only_service_slot_id.as_str(),
        },
    );
    if reservation.reservation_hash != expected_reservation_hash {
        return Some("retained_service_slot_reservation_hash_mismatch");
    }

    None
}

const EVENT_ID_TEXT_LEN: usize = 27;

fn event_id_text<'a>(event_id: EventId, out: &'a mut [u8; EVENT_ID_TEXT_LEN]) -> &'a str {
    const PREFIX: &[u8] = b"event.current_boot.";
    out[..PREFIX.len()].copy_from_slice(PREFIX);
    let mut value = event_id.sequence();
    let mut idx = EVENT_ID_TEXT_LEN;
    while idx > PREFIX.len() {
        idx -= 1;
        out[idx] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    unsafe { str::from_utf8_unchecked(out) }
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
    let manifest_reference = log.latest_module_manifest_reference();
    let artifact_reference = log.latest_module_candidate_artifact_reference();
    let vm_report_reference = log.latest_module_vm_test_report_reference();
    let retained = log.latest_module_computed_grant_reference();
    let audit_rollback = log.latest_module_audit_rollback_reference();
    let service_slot = log.latest_module_service_slot_reservation();
    let manifest_check = log.check_module_manifest_reference_for_load(manifest_reference, retained);
    let artifact_check = log.check_module_candidate_artifact_reference_for_load(
        artifact_reference,
        manifest_reference,
        retained,
    );
    let vm_report_check = log.check_module_vm_test_report_reference_for_load(
        vm_report_reference,
        manifest_reference,
        artifact_reference,
        retained,
    );
    let audit_rollback_check =
        log.check_module_audit_rollback_reference_for_load(retained, audit_rollback);
    let service_slot_check = log.check_module_service_slot_reservation_for_load(
        retained,
        audit_rollback_check,
        service_slot,
    );
    let binding = ModuleLoadGateBinding {
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
    };
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
    [
        selftest_missing_export_audit_binding(context),
        selftest_denial_schema_substitution(context),
        selftest_stale_dropped_request_binding_event_id(context),
        selftest_stale_dropped_envelope_event_id(context),
        selftest_previous_boot_or_unretained_event_id(context),
        selftest_request_envelope_wrong_variant(context),
        selftest_positive_record_substitution(context),
        selftest_request_envelope_event_id_mismatch(context),
        selftest_request_id_mismatch(context),
        selftest_request_body_hash_mismatch(context),
        selftest_request_envelope_hash_mismatch(context),
        selftest_request_binding_hash_mismatch(context),
        selftest_provider_minimal_packet_hash_mismatch(context),
        selftest_exported_field_list_hash_mismatch(context),
        selftest_omitted_field_list_hash_mismatch(context),
        selftest_trust_bypass_record(context),
    ]
}

pub fn provider_context_injection_gate_selftest(
    context: ProviderContextHashes,
) -> [ProviderContextInjectionGateSelfTestCase; PROVIDER_CONTEXT_INJECTION_GATE_SELFTEST_CASES] {
    [
        selftest_missing_final_authorization(context),
        selftest_stale_dropped_final_authorization_event_id(context),
        selftest_final_authorization_schema_substitution(context),
        selftest_substituted_positive_final_authorization_record(context),
        selftest_final_authorization_body_hash_mismatch(context),
        selftest_final_authorization_trust_downgrade(context),
        selftest_body_attachment_without_final_authorization(context),
    ]
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

pub fn latest_module_candidate_artifact_reference(
) -> Option<(EventId, ModuleCandidateArtifactReference)> {
    LOG.lock().latest_module_candidate_artifact_reference()
}

pub fn latest_module_vm_test_report_reference() -> Option<(EventId, ModuleVmTestReportReference)> {
    LOG.lock().latest_module_vm_test_report_reference()
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

fn normalize_limit(limit: usize) -> usize {
    if limit == 0 {
        DEFAULT_EVENT_LIMIT
    } else {
        usize::min(limit, EVENT_CAPACITY)
    }
}

fn selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: ProviderBindingGateCheck,
) -> ProviderBindingGateSelfTestCase {
    ProviderBindingGateSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: check.status == expected_status && check.reason == expected_reason,
    }
}

fn injection_selftest_case(
    name: &'static str,
    expected_status: &'static str,
    expected_reason: &'static str,
    check: ProviderContextInjectionGateCheck,
) -> ProviderContextInjectionGateSelfTestCase {
    ProviderContextInjectionGateSelfTestCase {
        name,
        expected_status,
        expected_reason,
        actual_status: check.status,
        actual_reason: check.reason,
        passed: check.status == expected_status && check.reason == expected_reason,
    }
}

fn selftest_missing_export_audit_binding(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let log = EventLog::new();
    selftest_case(
        "missing_export_audit_binding",
        "missing",
        "provider_context_export_audit_binding_missing",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_denial_schema_substitution(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let denial_event_id = record_selftest_request_denial(&mut log, context);
    let export = selftest_export_binding(1, envelope_event_id, denial_event_id, context);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "denial_schema_substitution",
        "rejected",
        "binding_denied_schema_or_wrong_variant",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_stale_dropped_request_binding_event_id(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    record_selftest_filler(&mut log, EVENT_CAPACITY);
    let export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "stale_dropped_request_binding_event_id",
        "rejected",
        "binding_stale_or_dropped_event_id",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_stale_dropped_envelope_event_id(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request = selftest_request_binding(1, envelope_event_id, context);
    let request_event_id = record_selftest_request_binding(&mut log, request);
    record_selftest_filler(&mut log, EVENT_CAPACITY - 2);
    let export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "stale_dropped_envelope_event_id",
        "rejected",
        "binding_stale_or_dropped_event_id",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_previous_boot_or_unretained_event_id(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let previous_boot_like_id = EventId { sequence: u64::MAX };
    let export = selftest_export_binding(1, envelope_event_id, previous_boot_like_id, context);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "previous_boot_or_unretained_event_id",
        "rejected",
        "binding_stale_or_dropped_event_id",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_request_envelope_wrong_variant(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let wrong_envelope_event_id = record_selftest_request_denial(&mut log, context);
    let request = selftest_request_binding(1, wrong_envelope_event_id, context);
    let request_event_id = record_selftest_request_binding(&mut log, request);
    let export = selftest_export_binding(1, wrong_envelope_event_id, request_event_id, context);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "request_envelope_wrong_variant",
        "rejected",
        "request_envelope_wrong_variant",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_positive_record_substitution(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let substituted_envelope_event_id = record_selftest_envelope(&mut log, 2);
    let mut substituted = selftest_request_binding(2, substituted_envelope_event_id, context);
    substituted.request_body_hash = tagged_hash(42);
    let substituted_event_id = record_selftest_request_binding(&mut log, substituted);
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.request_binding_event_id = substituted_event_id;
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "positive_record_substitution",
        "rejected",
        "binding_request_envelope_event_id_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_request_envelope_event_id_mismatch(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.request_envelope_event_id = EventId {
        sequence: envelope_event_id.sequence().saturating_add(99),
    };
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "request_envelope_event_id_mismatch",
        "rejected",
        "binding_request_envelope_event_id_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_request_id_mismatch(context: ProviderContextHashes) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let export = selftest_export_binding(2, envelope_event_id, request_event_id, context);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "request_id_mismatch",
        "rejected",
        "binding_request_id_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_request_body_hash_mismatch(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.request_body_hash = tagged_hash(43);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "request_body_hash_mismatch",
        "rejected",
        "binding_request_body_hash_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_request_envelope_hash_mismatch(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.request_envelope_hash = tagged_hash(44);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "request_envelope_hash_mismatch",
        "rejected",
        "binding_request_envelope_hash_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_request_binding_hash_mismatch(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.request_binding_hash = tagged_hash(45);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "request_binding_hash_mismatch",
        "rejected",
        "binding_request_binding_hash_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_provider_minimal_packet_hash_mismatch(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.context.projected_packet_hash = tagged_hash(46);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "provider_minimal_packet_hash_mismatch",
        "rejected",
        "binding_provider_minimal_packet_hash_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_exported_field_list_hash_mismatch(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.context.exported_field_list_hash = tagged_hash(47);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "exported_field_list_hash_mismatch",
        "rejected",
        "binding_exported_field_list_hash_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_omitted_field_list_hash_mismatch(
    context: ProviderContextHashes,
) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let request_event_id = record_selftest_request_binding(
        &mut log,
        selftest_request_binding(1, envelope_event_id, context),
    );
    let mut export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    export.context.omitted_field_list_hash = tagged_hash(48);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "omitted_field_list_hash_mismatch",
        "rejected",
        "binding_omitted_field_list_hash_mismatch",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_trust_bypass_record(context: ProviderContextHashes) -> ProviderBindingGateSelfTestCase {
    let mut log = EventLog::new();
    let envelope_event_id = record_selftest_envelope(&mut log, 1);
    let mut request = selftest_request_binding(1, envelope_event_id, context);
    request.development_tls_bypass = true;
    let request_event_id = record_selftest_request_binding(&mut log, request);
    let export = selftest_export_binding(1, envelope_event_id, request_event_id, context);
    record_selftest_export_audit(&mut log, export);

    selftest_case(
        "trust_bypass_record",
        "rejected",
        "binding_trust_bypass_record",
        log.check_provider_context_binding_gate(context),
    )
}

fn selftest_missing_final_authorization(
    context: ProviderContextHashes,
) -> ProviderContextInjectionGateSelfTestCase {
    let log = EventLog::new();
    injection_selftest_case(
        "missing_final_authorization",
        "missing",
        "final_injection_authorization_missing",
        log.check_provider_context_injection_gate(context, "pinned_spki_verified"),
    )
}

fn selftest_stale_dropped_final_authorization_event_id(
    context: ProviderContextHashes,
) -> ProviderContextInjectionGateSelfTestCase {
    let mut log = EventLog::new();
    let chain = record_selftest_injection_chain(&mut log, context);
    record_selftest_filler(&mut log, EVENT_CAPACITY);
    record_selftest_injection_authorization(
        &mut log,
        selftest_injection_authorization(chain, context),
    );

    injection_selftest_case(
        "stale_dropped_final_authorization_event_id",
        "rejected",
        "final_injection_authorization_stale_or_dropped_event_id",
        log.check_provider_context_injection_gate(context, "pinned_spki_verified"),
    )
}

fn selftest_final_authorization_schema_substitution(
    context: ProviderContextHashes,
) -> ProviderContextInjectionGateSelfTestCase {
    let mut log = EventLog::new();
    let chain = record_selftest_injection_chain(&mut log, context);
    let wrong_consumption_event_id = record_selftest_request_denial(&mut log, context);
    let mut authorization = selftest_injection_authorization(chain, context);
    authorization.binding_consumption_event_id = wrong_consumption_event_id;
    record_selftest_injection_authorization(&mut log, authorization);

    injection_selftest_case(
        "final_authorization_schema_substitution",
        "rejected",
        "final_injection_authorization_wrong_schema_or_variant",
        log.check_provider_context_injection_gate(context, "pinned_spki_verified"),
    )
}

fn selftest_substituted_positive_final_authorization_record(
    context: ProviderContextHashes,
) -> ProviderContextInjectionGateSelfTestCase {
    let mut log = EventLog::new();
    let chain = record_selftest_injection_chain(&mut log, context);
    let mut authorization = selftest_injection_authorization(chain, context);
    authorization.request_id = 2;
    record_selftest_injection_authorization(&mut log, authorization);

    injection_selftest_case(
        "substituted_positive_final_authorization_record",
        "rejected",
        "final_injection_authorization_substituted_record",
        log.check_provider_context_injection_gate(context, "pinned_spki_verified"),
    )
}

fn selftest_final_authorization_body_hash_mismatch(
    context: ProviderContextHashes,
) -> ProviderContextInjectionGateSelfTestCase {
    let mut log = EventLog::new();
    let chain = record_selftest_injection_chain(&mut log, context);
    let mut authorization = selftest_injection_authorization(chain, context);
    authorization.request_body_hash = tagged_hash(90);
    record_selftest_injection_authorization(&mut log, authorization);

    injection_selftest_case(
        "final_authorization_body_hash_mismatch",
        "rejected",
        "final_prewrite_body_hash_mismatch",
        log.check_provider_context_injection_gate(context, "pinned_spki_verified"),
    )
}

fn selftest_final_authorization_trust_downgrade(
    context: ProviderContextHashes,
) -> ProviderContextInjectionGateSelfTestCase {
    let mut log = EventLog::new();
    let chain = record_selftest_injection_chain(&mut log, context);
    record_selftest_injection_authorization(
        &mut log,
        selftest_injection_authorization(chain, context),
    );

    injection_selftest_case(
        "final_authorization_trust_downgrade",
        "rejected",
        "final_provider_trust_downgraded_before_write",
        log.check_provider_context_injection_gate(context, "pin_config_missing"),
    )
}

fn selftest_body_attachment_without_final_authorization(
    context: ProviderContextHashes,
) -> ProviderContextInjectionGateSelfTestCase {
    let mut log = EventLog::new();
    let chain = record_selftest_injection_chain(&mut log, context);
    let mut authorization = selftest_injection_authorization(chain, context);
    authorization.context_attached_to_provider_body = true;
    record_selftest_injection_authorization(&mut log, authorization);

    injection_selftest_case(
        "body_attachment_without_final_authorization",
        "rejected",
        "body_attachment_without_final_authorization",
        log.check_provider_context_injection_gate(context, "pinned_spki_verified"),
    )
}

#[derive(Clone, Copy)]
struct ProviderContextInjectionSelfTestChain {
    request_binding: ProviderRequestBinding,
    export_binding: ProviderExportAuditBinding,
    consumption: ProviderBindingConsumption,
    binding_consumption_event_id: EventId,
}

fn record_selftest_envelope(log: &mut EventLog, request_id: u32) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.provider_request.envelope_created",
        source_method: "provider.context_gate_selftest",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "synthetic_not_exported",
        requested_capability: "cap.provider.context_export.read",
        risk: "observe",
        subject: "selftest",
        resource: "current_boot.synthetic",
        reason: "synthetic_gate_input_not_global_evidence",
        evidence: &[],
        bindings: EventBindings::ProviderRequestEnvelope(ProviderRequestEnvelopeBinding {
            request_id,
            request_body_hash: tagged_hash(1),
            envelope_hash: tagged_hash(2),
            provider_trust_state: "pinned_spki_verified",
            provider_trust_positive: true,
            development_tls_bypass: false,
        }),
    })
}

fn record_selftest_request_binding(log: &mut EventLog, binding: ProviderRequestBinding) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.provider_context_export.request_binding_bound",
        source_method: "provider.context_gate_selftest",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "synthetic_not_exported",
        requested_capability: "cap.provider.context_export.read",
        risk: "observe",
        subject: "selftest",
        resource: "current_boot.synthetic",
        reason: "synthetic_gate_input_not_global_evidence",
        evidence: &[],
        bindings: EventBindings::ProviderRequestBound(binding),
    })
}

fn record_selftest_export_audit(
    log: &mut EventLog,
    binding: ProviderExportAuditBinding,
) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.provider_context_export.audit_binding_bound",
        source_method: "provider.context_gate_selftest",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "synthetic_not_exported",
        requested_capability: "cap.provider.context_export.read",
        risk: "observe",
        subject: "selftest",
        resource: "current_boot.synthetic",
        reason: "synthetic_gate_input_not_global_evidence",
        evidence: &[],
        bindings: EventBindings::ProviderExportAuditBound(binding),
    })
}

fn record_selftest_binding_consumption(
    log: &mut EventLog,
    binding: ProviderBindingConsumption,
) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.provider_context_export.binding_consumption_checked",
        source_method: "provider.context_injection_gate_selftest",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "synthetic_not_exported",
        requested_capability: "cap.provider.context_injection.read",
        risk: "observe",
        subject: "selftest",
        resource: "current_boot.synthetic",
        reason: "synthetic_gate_input_not_global_evidence",
        evidence: PROVIDER_BINDING_CONSUMPTION_EVIDENCE,
        bindings: EventBindings::ProviderBindingConsumption(binding),
    })
}

fn record_selftest_injection_authorization(
    log: &mut EventLog,
    binding: ProviderContextInjectionAuthorization,
) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.provider_context_injection.authorization_bound",
        source_method: "provider.context_injection_gate_selftest",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "synthetic_not_exported",
        requested_capability: "cap.provider.context_injection.read",
        risk: "observe",
        subject: "selftest",
        resource: "current_boot.synthetic",
        reason: "synthetic_gate_input_not_global_evidence",
        evidence: PROVIDER_CONTEXT_INJECTION_AUTHORIZATION_EVIDENCE,
        bindings: EventBindings::ProviderContextInjectionAuthorization(binding),
    })
}

fn record_selftest_request_denial(log: &mut EventLog, context: ProviderContextHashes) -> EventId {
    log.record(Event {
        sequence: 0,
        kind: "selftest.provider_context_export.request_binding_denied",
        source_method: "provider.context_gate_selftest",
        source_transport: "serial-console",
        classification: "local_only",
        outcome: "synthetic_not_exported",
        requested_capability: "cap.provider.context_export.read",
        risk: "observe",
        subject: "selftest",
        resource: "current_boot.synthetic",
        reason: "synthetic_denial_variant",
        evidence: &[],
        bindings: EventBindings::ProviderRequestBindingDenied(context),
    })
}

fn record_selftest_filler(log: &mut EventLog, count: usize) {
    let mut idx = 0usize;
    while idx < count {
        log.record(Event {
            sequence: 0,
            kind: "selftest.filler",
            source_method: "provider.context_gate_selftest",
            source_transport: "serial-console",
            classification: "local_only",
            outcome: "synthetic_not_exported",
            requested_capability: "cap.provider.context_export.read",
            risk: "observe",
            subject: "selftest",
            resource: "current_boot.synthetic",
            reason: "fills_ram_ring_to_exercise_retention",
            evidence: &[],
            bindings: EventBindings::None,
        });
        idx += 1;
    }
}

fn record_selftest_injection_chain(
    log: &mut EventLog,
    context: ProviderContextHashes,
) -> ProviderContextInjectionSelfTestChain {
    let request_envelope_event_id = record_selftest_envelope(log, 1);
    let request_binding = selftest_request_binding(1, request_envelope_event_id, context);
    let request_binding_event_id = record_selftest_request_binding(log, request_binding);
    let export_binding = selftest_export_binding(
        1,
        request_envelope_event_id,
        request_binding_event_id,
        context,
    );
    let export_audit_binding_event_id = record_selftest_export_audit(log, export_binding);
    let consumption = ProviderBindingConsumption {
        request_id: 1,
        request_envelope_event_id,
        request_binding_event_id,
        export_audit_binding_event_id,
        request_binding_hash: request_binding.request_binding_hash,
        export_audit_binding_hash: export_binding.export_audit_binding_hash,
        context,
    };
    let binding_consumption_event_id = record_selftest_binding_consumption(log, consumption);

    ProviderContextInjectionSelfTestChain {
        request_binding,
        export_binding,
        consumption,
        binding_consumption_event_id,
    }
}

fn selftest_request_binding(
    request_id: u32,
    request_envelope_event_id: EventId,
    context: ProviderContextHashes,
) -> ProviderRequestBinding {
    ProviderRequestBinding {
        request_id,
        request_envelope_event_id,
        request_body_hash: tagged_hash(1),
        request_envelope_hash: tagged_hash(2),
        request_binding_hash: tagged_hash(3),
        context,
        provider_trust_state: "pinned_spki_verified",
        development_tls_bypass: false,
    }
}

fn selftest_export_binding(
    request_id: u32,
    request_envelope_event_id: EventId,
    request_binding_event_id: EventId,
    context: ProviderContextHashes,
) -> ProviderExportAuditBinding {
    ProviderExportAuditBinding {
        request_id,
        request_envelope_event_id,
        request_binding_event_id,
        request_body_hash: tagged_hash(1),
        request_envelope_hash: tagged_hash(2),
        request_binding_hash: tagged_hash(3),
        export_audit_binding_hash: tagged_hash(4),
        context,
        provider_trust_state: "pinned_spki_verified",
        context_attached_to_provider_body: false,
    }
}

fn selftest_injection_authorization(
    chain: ProviderContextInjectionSelfTestChain,
    context: ProviderContextHashes,
) -> ProviderContextInjectionAuthorization {
    ProviderContextInjectionAuthorization {
        request_id: chain.consumption.request_id,
        request_envelope_event_id: chain.consumption.request_envelope_event_id,
        request_binding_event_id: chain.consumption.request_binding_event_id,
        export_audit_binding_event_id: chain.consumption.export_audit_binding_event_id,
        binding_consumption_event_id: chain.binding_consumption_event_id,
        request_body_hash: chain.request_binding.request_body_hash,
        request_envelope_hash: chain.request_binding.request_envelope_hash,
        request_binding_hash: chain.consumption.request_binding_hash,
        export_audit_binding_hash: chain.consumption.export_audit_binding_hash,
        context,
        provider_trust_state: chain.export_binding.provider_trust_state,
        final_authorization_hash: tagged_hash(5),
        context_attached_to_provider_body: false,
    }
}

fn tagged_hash(tag: u8) -> [u8; 32] {
    let mut hash = [tag; 32];
    hash[31] = tag.wrapping_mul(17);
    hash
}
