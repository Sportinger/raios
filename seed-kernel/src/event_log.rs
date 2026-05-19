use spin::Mutex;

pub const EVENT_CAPACITY: usize = 64;
pub const DEFAULT_EVENT_LIMIT: usize = 32;

const READ_EVIDENCE: &[&str] = &["computed_capability_grant"];
const DENIED_EVIDENCE: &[&str] = &["missing_required_evidence", "capability_denied"];
const PROVIDER_REQUEST_BINDING_DENIAL_EVIDENCE: &[&str] = &[
    "provider_request_binding_denied",
    "projected_packet_hash",
    "provider_write_not_attempted",
];
const PROVIDER_EXPORT_DENIAL_AUDIT_EVIDENCE: &[&str] = &[
    "provider_request_binding_denied",
    "projected_packet_hash",
    "exported_field_list_hash",
    "omitted_field_list_hash",
    "provider_write_not_attempted",
];
const PROVIDER_REQUEST_ENVELOPE_EVIDENCE: &[&str] = &[
    "provider_request_envelope_created",
    "request_body_hash",
    "envelope_hash",
    "provider_write_not_attempted",
];
const PROVIDER_REQUEST_BINDING_EVIDENCE: &[&str] = &[
    "provider_request_binding",
    "request_envelope_hash",
    "request_body_hash",
    "projected_packet_hash",
    "exported_field_list_hash",
    "omitted_field_list_hash",
    "positive_provider_trust",
    "provider_write_not_attempted",
];
const PROVIDER_EXPORT_AUDIT_BINDING_EVIDENCE: &[&str] = &[
    "provider_context_export_audit_binding",
    "provider_request_binding",
    "request_envelope_hash",
    "request_body_hash",
    "projected_packet_hash",
    "exported_field_list_hash",
    "omitted_field_list_hash",
    "positive_provider_trust",
    "context_injection_disabled",
];

static LOG: Mutex<EventLog> = Mutex::new(EventLog::new());

#[derive(Clone, Copy)]
pub struct EventId {
    sequence: u64,
}

impl EventId {
    pub fn sequence(self) -> u64 {
        self.sequence
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
pub enum EventBindings {
    None,
    ProviderRequestEnvelope(ProviderRequestEnvelopeBinding),
    ProviderRequestBound(ProviderRequestBinding),
    ProviderExportAuditBound(ProviderExportAuditBinding),
    ProviderRequestBindingDenied(ProviderContextHashes),
    ProviderExportDenialAudit(ProviderContextHashes),
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

struct EventLog {
    events: [Option<Event>; EVENT_CAPACITY],
    next_slot: usize,
    len: usize,
    next_sequence: u64,
}

impl EventLog {
    const fn new() -> Self {
        Self {
            events: [None; EVENT_CAPACITY],
            next_slot: 0,
            len: 0,
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

fn normalize_limit(limit: usize) -> usize {
    if limit == 0 {
        DEFAULT_EVENT_LIMIT
    } else {
        usize::min(limit, EVENT_CAPACITY)
    }
}
