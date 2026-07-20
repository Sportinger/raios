extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{self, Write};
use core::str;

use embedded_io::Write as IoWrite;
use embedded_tls::blocking::{Aes128GcmSha256, NoVerify, TlsConfig, TlsConnection, TlsContext};
use raios_core::http_response_parse::{
    find_subslice, header_contains, http_response_complete, parse_hex_usize, parse_status,
    trim_ascii,
};
use raios_core::scoped_provider_export::{
    evaluate_provider_export_gate, ProviderExportGateDecision, ProviderExportGateInput,
};
use rand_core::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};
use spin::Mutex;

use crate::{
    agent_build_loop::{self, FeedbackPacket}, agent_protocol, entropy, event_log, memory_store, net,
    openai_trust::OpenAiPinnedCertVerifier, provider, provider_config, provider_trust, secret_vault,
    serial, time, tls_io::KernelTcpStream, ui,
};

const API_HOST: &str = provider_trust::OPENAI_PINNED_TLS_VERIFIER_METADATA.host;
const API_PORT: u16 = 443;
const API_PATH: &str = "/v1/responses";
const MODEL: &str = "gpt-5.4";
pub(crate) const DEFAULT_MAX_OUTPUT_TOKENS: u16 = 512;
pub(crate) const MAX_OUTPUT_TOKENS: u16 = 8_192;
const LINE_CAPACITY: usize = 2048;
const DNS_TIMEOUT_MS: u64 = 6_000;
const TCP_TIMEOUT_MS: u64 = 8_000;
const HTTPS_TIMEOUT_MS: u64 = 60_000;
const TRANSPORT_LEASE_TIMEOUT_MS: u64 = 90_000;
const TLS_RECORD_BUFFER_SIZE: usize = 16_640;
const TLS_WRITE_BUFFER_SIZE: usize = 4_096;
const HTTP_RESPONSE_LIMIT: usize = 128 * 1024;
const SCOPED_FEEDBACK_DESTINATION: &str = "provider.openai.responses";
const SCOPED_FEEDBACK_PROFILE: &str = "provider_minimal";
const SCOPED_FEEDBACK_BUDGET_TOKENS: u64 = 1_000;
const SCOPED_FEEDBACK_RECORD_COUNT: u64 = 4;
const SCOPED_PROJECT_FEEDBACK_PROMPT: &str =
    "Return one complete corrected RAIOS_SOURCE_FILES_V1 revision using only the attached system feedback.";

static STATE: Mutex<OpenAiState> = Mutex::new(OpenAiState::new());

#[derive(Clone, Copy)]
pub enum SubmitError {
    Empty,
    InvalidMaxOutput,
    Busy(u32),
}

#[derive(Clone, Copy)]
pub struct FixedLine {
    bytes: [u8; LINE_CAPACITY],
    len: usize,
}

impl FixedLine {
    const fn empty() -> Self {
        Self {
            bytes: [0; LINE_CAPACITY],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn set_from_bytes(&mut self, bytes: &[u8]) {
        self.clear();
        let value = str::from_utf8(bytes).unwrap_or("?");
        self.set_from_str(value);
    }

    fn set_from_str(&mut self, value: &str) {
        self.clear();
        push_str_truncated(&mut self.bytes, &mut self.len, value);
    }
}

fn push_str_truncated(bytes: &mut [u8], len: &mut usize, value: &str) {
    for ch in value.chars() {
        let char_len = ch.len_utf8();
        if (*len).saturating_add(char_len) > bytes.len() {
            break;
        }
        ch.encode_utf8(&mut bytes[*len..*len + char_len]);
        *len += char_len;
    }
}

impl Write for FixedLine {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        push_str_truncated(&mut self.bytes, &mut self.len, s);
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct Snapshot {
    pub phase: &'static str,
    pub pending_id: Option<u32>,
    pub last_request_id: Option<u32>,
    pub last_prompt: FixedLine,
    pub last_event: FixedLine,
    pub last_error: FixedLine,
    pub endpoint: &'static str,
    pub model: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Resolving,
    Connecting,
    Requesting,
}

struct PendingRequest {
    id: u32,
    phase: Phase,
    address: Option<smoltcp::wire::Ipv4Address>,
    transport_lease: Option<net::TransportLeaseToken>,
    phase_started_ms: u64,
    envelope: Option<ProviderRequestEnvelope>,
    envelope_event_id: Option<event_log::EventId>,
    body: String,
    scoped_feedback: ScopedFeedbackState,
    target: provider::RequestTarget,
    runtime: ui::RuntimeStatus,
}

enum ScopedFeedbackState {
    NotRequested,
    Ready(ScopedFeedbackAuthorization),
    Taken,
}

struct ScopedFeedbackAuthorization {
    packet: FeedbackPacket,
    consumed: bool,
}

#[derive(Clone, Copy)]
struct ProviderRequestEnvelope {
    request_id: u32,
    max_output_tokens: u16,
    source_method: &'static str,
    request_body_hash: [u8; 32],
    envelope_hash: [u8; 32],
    provider_trust_state: &'static str,
    provider_trust_positive: bool,
    development_tls_bypass: bool,
    context_attached_to_provider_body: bool,
}

struct AuthorizedScopedRequest {
    body: String,
    envelope: ProviderRequestEnvelope,
    context: event_log::ProviderContextHashes,
    gate_decision: ProviderExportGateDecision,
    audit_binding_hash: [u8; 32],
    packet_hash: [u8; 32],
    packet_estimated_tokens: u64,
    audit: memory_store::LiveProviderExportAuditOutcome,
}

pub(crate) struct Event {
    pub(crate) id: u32,
    pub(crate) target: provider::RequestTarget,
    pub(crate) kind: EventKind,
}

pub(crate) enum EventKind {
    Answer(String, provider::AnswerProvenance),
    Error(FixedLine),
}

impl ProviderRequestEnvelope {
    fn event_binding(self) -> event_log::ProviderRequestEnvelopeBinding {
        event_log::ProviderRequestEnvelopeBinding {
            request_id: self.request_id,
            request_body_hash: self.request_body_hash,
            envelope_hash: self.envelope_hash,
            provider_trust_state: self.provider_trust_state,
            provider_trust_positive: self.provider_trust_positive,
            development_tls_bypass: self.development_tls_bypass,
        }
    }
}

struct OpenAiState {
    next_id: u32,
    pending: Option<PendingRequest>,
    last_request_id: Option<u32>,
    last_prompt: FixedLine,
    last_event: FixedLine,
    last_error: FixedLine,
}

impl OpenAiState {
    const fn new() -> Self {
        Self {
            next_id: 1,
            pending: None,
            last_request_id: None,
            last_prompt: FixedLine::empty(),
            last_event: FixedLine::empty(),
            last_error: FixedLine::empty(),
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            phase: self
                .pending
                .as_ref()
                .map(|pending| phase_name(pending.phase))
                .unwrap_or("IDLE"),
            pending_id: self.pending.as_ref().map(|pending| pending.id),
            last_request_id: self.last_request_id,
            last_prompt: self.last_prompt,
            last_event: self.last_event,
            last_error: self.last_error,
            endpoint: "https://api.openai.com/v1/responses",
            model: MODEL,
        }
    }
}

pub(crate) const fn model() -> &'static str {
    MODEL
}

pub fn submit_request(
    prompt: &str,
    max_output_tokens: u16,
    target: provider::RequestTarget,
    runtime: ui::RuntimeStatus,
) -> Result<u32, SubmitError> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err(SubmitError::Empty);
    }
    if !(1..=MAX_OUTPUT_TOKENS).contains(&max_output_tokens) {
        return Err(SubmitError::InvalidMaxOutput);
    }

    let mut state = STATE.lock();
    if let Some(pending) = state.pending.as_ref() {
        return Err(SubmitError::Busy(pending.id));
    }

    let id = state.next_id;
    state.next_id = state.next_id.wrapping_add(1).max(1);
    let body = build_request_body(prompt, max_output_tokens);
    let envelope = build_provider_request_envelope(
        id,
        max_output_tokens,
        target,
        hash_bytes(body.as_bytes()),
        provider_trust::snapshot(),
        false,
    );
    let envelope_event_id =
        event_log::record_provider_request_envelope_created(envelope.event_binding());
    state.pending = Some(PendingRequest {
        id,
        phase: Phase::Resolving,
        address: None,
        transport_lease: None,
        phase_started_ms: now_ms(),
        envelope: Some(envelope),
        envelope_event_id: Some(envelope_event_id),
        body,
        scoped_feedback: ScopedFeedbackState::NotRequested,
        target,
        runtime,
    });
    state.last_request_id = Some(id);
    state.last_prompt.set_from_str(prompt);
    state
        .last_event
        .set_from_bytes(b"OPENAI DIRECT: RESOLVING api.openai.com");
    state.last_error.clear();
    emit_provider_request_envelope(envelope, runtime);

    serial::write_fmt(format_args!(
        "OPENAI_DIRECT_REQ {} {} {}\r\n",
        id, API_HOST, API_PATH
    ));
    Ok(id)
}

pub(crate) fn submit_scoped_project_feedback(
    packet: FeedbackPacket,
    runtime: ui::RuntimeStatus,
) -> Result<u32, SubmitError> {
    let mut state = STATE.lock();
    if let Some(pending) = state.pending.as_ref() {
        return Err(SubmitError::Busy(pending.id));
    }

    let id = state.next_id;
    state.next_id = state.next_id.wrapping_add(1).max(1);
    state.pending = Some(PendingRequest {
        id,
        phase: Phase::Resolving,
        address: None,
        transport_lease: None,
        phase_started_ms: now_ms(),
        envelope: None,
        envelope_event_id: None,
        body: build_request_body(SCOPED_PROJECT_FEEDBACK_PROMPT, MAX_OUTPUT_TOKENS),
        scoped_feedback: ScopedFeedbackState::Ready(ScopedFeedbackAuthorization {
            packet,
            consumed: false,
        }),
        target: provider::RequestTarget::ProjectFeedbackWorkspace,
        runtime,
    });
    state.last_request_id = Some(id);
    state
        .last_prompt
        .set_from_str(SCOPED_PROJECT_FEEDBACK_PROMPT);
    state
        .last_event
        .set_from_bytes(b"OPENAI DIRECT: RESOLVING api.openai.com");
    state.last_error.clear();
    serial::write_fmt(format_args!(
        "OPENAI_DIRECT_REQ {} {} {}\r\n",
        id, API_HOST, API_PATH
    ));
    Ok(id)
}

pub fn poll() -> Option<Event> {
    let now = now_ms();
    let (phase, phase_started_ms, address, transport_lease) = {
        let state = STATE.lock();
        let pending = state.pending.as_ref()?;
        (
            pending.phase,
            pending.phase_started_ms,
            pending.address,
            pending.transport_lease,
        )
    };

    match phase {
        Phase::Resolving => {
            let resolved = net::resolve_hostname(API_HOST);
            let mut state = STATE.lock();
            let Some(pending) = state.pending.as_mut() else {
                return None;
            };
            if pending.phase != Phase::Resolving {
                return None;
            }
            if let Some(address) = resolved {
                pending.address = Some(address);
                pending.phase = Phase::Connecting;
                pending.phase_started_ms = now;
                serial::write_fmt(format_args!(
                    "openai: {} resolved to {}; connecting tcp {}\r\n",
                    API_HOST, address, API_PORT
                ));
                let lease = match net::tcp_claim(
                    net::NATIVE_OPENAI_TRANSPORT_OWNER,
                    now,
                    TRANSPORT_LEASE_TIMEOUT_MS,
                ) {
                    Ok(lease) => lease,
                    Err(net::TransportLeaseError::NetworkTransportBusy) => {
                        return complete_error(&mut state, b"OPENAI DIRECT NETWORK TRANSPORT BUSY")
                    }
                    Err(_) => {
                        return complete_error(
                            &mut state,
                            b"OPENAI DIRECT NETWORK TRANSPORT LEASE DENIED",
                        )
                    }
                };
                pending.transport_lease = Some(lease);
                let connect = match net::tcp_connect_step(lease, address, API_PORT, now) {
                    Ok(result) => result,
                    Err(_) => {
                        return complete_error(
                            &mut state,
                            b"OPENAI DIRECT NETWORK TRANSPORT LEASE DENIED",
                        )
                    }
                };
                match handle_tcp_result(&mut state, connect) {
                    TcpAction::None => None,
                    TcpAction::StartHttps {
                        scoped_feedback, ..
                    } => {
                        if let Some(pending) = state.pending.as_mut() {
                            pending.scoped_feedback = match scoped_feedback {
                                Some(authorization) => ScopedFeedbackState::Ready(authorization),
                                None => ScopedFeedbackState::NotRequested,
                            };
                        }
                        None
                    }
                    TcpAction::Event(line) => Some(line),
                }
            } else if now.saturating_sub(phase_started_ms) >= DNS_TIMEOUT_MS {
                complete_error(&mut state, b"OPENAI DIRECT DNS TIMEOUT")
            } else {
                None
            }
        }
        Phase::Connecting => {
            let Some(address) = address else {
                let mut state = STATE.lock();
                return complete_error(&mut state, b"OPENAI DIRECT LOST DNS RESULT");
            };
            let Some(lease) = transport_lease else {
                let mut state = STATE.lock();
                return complete_error(&mut state, b"OPENAI DIRECT TRANSPORT LEASE LOST");
            };
            let mut state = STATE.lock();
            if now.saturating_sub(phase_started_ms) >= TCP_TIMEOUT_MS {
                return complete_error(&mut state, b"OPENAI DIRECT TCP TIMEOUT");
            }
            let connect = match net::tcp_connect_step(lease, address, API_PORT, now) {
                Ok(result) => result,
                Err(net::TransportLeaseError::LeaseTimedOut) => {
                    return complete_error(&mut state, b"OPENAI DIRECT TCP TIMEOUT")
                }
                Err(_) => {
                    return complete_error(
                        &mut state,
                        b"OPENAI DIRECT NETWORK TRANSPORT LEASE DENIED",
                    )
                }
            };
            match handle_tcp_result(&mut state, connect) {
                TcpAction::None => None,
                TcpAction::Event(line) => Some(line),
                TcpAction::StartHttps {
                    id,
                    target,
                    body,
                    envelope,
                    envelope_event_id,
                    scoped_feedback,
                    runtime,
                    transport_lease,
                } => {
                    if let Some(pending) = state.pending.as_mut() {
                        pending.phase = Phase::Requesting;
                        pending.phase_started_ms = now;
                    }
                    state
                        .last_event
                        .set_from_bytes(b"OPENAI DIRECT: TLS HANDSHAKE STARTED");
                    drop(state);
                    let result = perform_https_request(
                        id,
                        target,
                        body.as_str(),
                        envelope,
                        envelope_event_id,
                        scoped_feedback,
                        runtime,
                        transport_lease,
                    );
                    let _ = net::tcp_abort(transport_lease, now_ms());
                    let mut state = STATE.lock();
                    if let Some(pending) = state.pending.as_mut() {
                        pending.transport_lease = None;
                    }
                    match result {
                        HttpsResult::Answer(answer, provenance) => {
                            state
                                .last_event
                                .set_from_bytes(b"OPENAI DIRECT: RESPONSE COMPLETED");
                            state.pending = None;
                            Some(Event {
                                id,
                                target,
                                kind: EventKind::Answer(answer, provenance),
                            })
                        }
                        HttpsResult::Status(status, detail) => {
                            let mut line = FixedLine::empty();
                            let _ = write!(line, "OPENAI HTTP {} {}", status, detail.as_str());
                            state.last_error = line;
                            state.last_event = line;
                            state.pending = None;
                            Some(Event {
                                id,
                                target,
                                kind: EventKind::Error(line),
                            })
                        }
                        HttpsResult::Error(error) => complete_error(&mut state, error),
                    }
                }
            }
        }
        Phase::Requesting => {
            let mut state = STATE.lock();
            if now.saturating_sub(phase_started_ms) >= HTTPS_TIMEOUT_MS {
                complete_error(&mut state, b"OPENAI DIRECT HTTPS TIMEOUT")
            } else {
                None
            }
        }
    }
}

pub fn snapshot() -> Snapshot {
    STATE.lock().snapshot()
}

enum TcpAction {
    None,
    Event(Event),
    StartHttps {
        id: u32,
        target: provider::RequestTarget,
        body: String,
        envelope: Option<ProviderRequestEnvelope>,
        envelope_event_id: Option<event_log::EventId>,
        scoped_feedback: Option<ScopedFeedbackAuthorization>,
        runtime: ui::RuntimeStatus,
        transport_lease: net::TransportLeaseToken,
    },
}

fn handle_tcp_result(state: &mut OpenAiState, result: net::TcpConnectResult) -> TcpAction {
    match result {
        net::TcpConnectResult::Connected => {
            let Some(pending) = state.pending.as_mut() else {
                return TcpAction::None;
            };
            let Some(transport_lease) = pending.transport_lease else {
                return complete_error(state, b"OPENAI DIRECT TRANSPORT LEASE LOST")
                    .map(TcpAction::Event)
                    .unwrap_or(TcpAction::None);
            };
            let scoped_feedback = match core::mem::replace(
                &mut pending.scoped_feedback,
                ScopedFeedbackState::Taken,
            ) {
                ScopedFeedbackState::NotRequested => None,
                ScopedFeedbackState::Ready(authorization) => Some(authorization),
                ScopedFeedbackState::Taken => {
                    return complete_error(
                        state,
                        b"OPENAI SCOPED FEEDBACK AUTHORIZATION ALREADY CONSUMED",
                    )
                    .map(TcpAction::Event)
                    .unwrap_or(TcpAction::None)
                }
            };
            TcpAction::StartHttps {
                id: pending.id,
                target: pending.target,
                body: pending.body.clone(),
                envelope: pending.envelope,
                envelope_event_id: pending.envelope_event_id,
                scoped_feedback,
                runtime: pending.runtime,
                transport_lease,
            }
        }
        net::TcpConnectResult::Started => {
            state
                .last_event
                .set_from_bytes(b"OPENAI DIRECT: TCP CONNECT STARTED");
            TcpAction::None
        }
        net::TcpConnectResult::Connecting(tcp_state) => {
            let mut line = FixedLine::empty();
            let _ = write!(line, "OPENAI DIRECT: TCP {}", tcp_state);
            state.last_event = line;
            TcpAction::None
        }
        net::TcpConnectResult::NetworkUnavailable => {
            complete_error(state, b"OPENAI DIRECT NETWORK UNAVAILABLE")
                .map(TcpAction::Event)
                .unwrap_or(TcpAction::None)
        }
        net::TcpConnectResult::NetworkUnconfigured => {
            complete_error(state, b"OPENAI DIRECT NETWORK UNCONFIGURED")
                .map(TcpAction::Event)
                .unwrap_or(TcpAction::None)
        }
        net::TcpConnectResult::ConnectError => complete_error(state, b"OPENAI DIRECT TCP ERROR")
            .map(TcpAction::Event)
            .unwrap_or(TcpAction::None),
    }
}

fn complete_error(state: &mut OpenAiState, message: &[u8]) -> Option<Event> {
    let pending = state.pending.take()?;
    let id = pending.id;
    let target = pending.target;
    if let Some(lease) = pending.transport_lease {
        let _ = net::tcp_abort(lease, now_ms());
    }
    let mut line = FixedLine::empty();
    line.set_from_bytes(message);
    state.last_error = line;
    state.last_event = line;
    Some(Event {
        id,
        target,
        kind: EventKind::Error(line),
    })
}

fn phase_name(phase: Phase) -> &'static str {
    match phase {
        Phase::Resolving => "RESOLVING DNS",
        Phase::Connecting => "CONNECTING TCP",
        Phase::Requesting => "HTTPS REQUEST",
    }
}

fn now_ms() -> u64 {
    let per_ms = time::tsc_per_ms().max(1);
    time::rdtsc() / per_ms
}

fn build_provider_request_envelope(
    request_id: u32,
    max_output_tokens: u16,
    target: provider::RequestTarget,
    request_body_hash: [u8; 32],
    trust: provider_trust::Snapshot,
    context_attached_to_provider_body: bool,
) -> ProviderRequestEnvelope {
    let provider_trust_state = trust.state.as_protocol();
    let provider_trust_positive = provider_trust_positive(trust.state);
    let development_tls_bypass = trust.development_bypass;
    let source_method = match target {
        provider::RequestTarget::Conversation => "ask",
        provider::RequestTarget::ProgramWorkspace => "program.ask",
        provider::RequestTarget::ProjectWorkspace => "project.ask",
        provider::RequestTarget::ProjectFeedbackWorkspace => "project.feedback_export",
    };
    let envelope_hash = provider_request_envelope_hash(
        request_id,
        max_output_tokens,
        source_method,
        request_body_hash,
        provider_trust_state,
        provider_trust_positive,
        development_tls_bypass,
        context_attached_to_provider_body,
    );

    ProviderRequestEnvelope {
        request_id,
        max_output_tokens,
        source_method,
        request_body_hash,
        envelope_hash,
        provider_trust_state,
        provider_trust_positive,
        development_tls_bypass,
        context_attached_to_provider_body,
    }
}

fn provider_trust_positive(state: provider_trust::TrustState) -> bool {
    matches!(
        state,
        provider_trust::TrustState::PinnedCertVerified
            | provider_trust::TrustState::PinnedSpkiVerified
            | provider_trust::TrustState::WebPkiVerified
    )
}

fn provider_request_envelope_hash(
    request_id: u32,
    max_output_tokens: u16,
    source_method: &'static str,
    request_body_hash: [u8; 32],
    provider_trust_state: &'static str,
    provider_trust_positive: bool,
    development_tls_bypass: bool,
    context_attached_to_provider_body: bool,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_field(
        &mut hash,
        "domain",
        "raios.provider_request_envelope.canonical.v0",
    );
    hash_field(&mut hash, "schema", "raios.provider_request_envelope.v0");
    hash_field(&mut hash, "scope", "current_boot");
    hash_field(&mut hash, "classification", "local_only");
    hash_field(&mut hash, "persistence", "none");
    hash_field(&mut hash, "status", "local_prewrite_envelope");
    hash_field(&mut hash, "provider_write", "not_attempted");
    hash_field(&mut hash, "source.method", source_method);
    hash_field(&mut hash, "source.capability", "cap.provider.request");
    hash_field(&mut hash, "source.risk", "export");
    hash_field(&mut hash, "source.code_path", "seed-kernel/src/openai.rs");
    hash_field(&mut hash, "provider.selected", "OPENAI");
    hash_field(&mut hash, "provider.route", "OPENAI DIRECT");
    hash_field(&mut hash, "provider.host", API_HOST);
    hash_field(&mut hash, "provider.port", "443");
    hash_field(&mut hash, "provider.method", "POST");
    hash_field(&mut hash, "provider.path", API_PATH);
    hash_field(&mut hash, "provider.model", MODEL);
    hash_field(&mut hash, "request.id", format!("{}", request_id).as_str());
    hash_field(
        &mut hash,
        "request_body.schema",
        "openai.responses.request.redacted.v0",
    );
    if context_attached_to_provider_body {
        hash_field(
            &mut hash,
            "request_body.instructions",
            "fixed_scoped_project_feedback",
        );
        hash_field(
            &mut hash,
            "request_body.input",
            "provider_minimal_packet_redacted",
        );
    } else {
        hash_field(&mut hash, "request_body.user_prompt", "present_redacted");
    }
    hash_field(
        &mut hash,
        "request_body.max_output_tokens",
        format!("{}", max_output_tokens).as_str(),
    );
    hash_field(&mut hash, "request_body.store", "false");
    hash_field(
        &mut hash,
        "request_body.context_attached_to_provider_body",
        bool_str(context_attached_to_provider_body),
    );
    hash_hash_field(&mut hash, "request_body.body_sha256", request_body_hash);
    hash_field(
        &mut hash,
        "secret_state.api_key_state",
        if provider_credential_configured() {
            "set"
        } else {
            "missing"
        },
    );
    hash_field(&mut hash, "secret_state.authorization_header", "redacted");
    hash_field(&mut hash, "secret_state.api_key_value", "not_recorded");
    hash_field(
        &mut hash,
        "provider_minimal_context.attached",
        bool_str(context_attached_to_provider_body),
    );
    hash_field(
        &mut hash,
        "provider_minimal_context.binding_status",
        if context_attached_to_provider_body {
            "bound"
        } else {
            "not_bound"
        },
    );
    hash_field(
        &mut hash,
        "trust_snapshot.provider_trust_state",
        provider_trust_state,
    );
    hash_field(
        &mut hash,
        "trust_snapshot.provider_trust_positive",
        bool_str(provider_trust_positive),
    );
    hash_field(
        &mut hash,
        "trust_snapshot.development_tls_bypass",
        bool_str(development_tls_bypass),
    );
    hash.finalize().into()
}

fn hash_bytes(bytes: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(bytes);
    hash.finalize().into()
}

fn hash_field(hash: &mut Sha256, name: &str, value: &str) {
    hash.update(name.as_bytes());
    hash.update(b"=");
    hash.update(value.as_bytes());
    hash.update(b"\n");
}

fn hash_hash_field(hash: &mut Sha256, name: &str, value: [u8; 32]) {
    hash.update(name.as_bytes());
    hash.update(b"=sha256:");
    hash.update(value);
    hash.update(b"\n");
}

fn emit_provider_request_envelope(envelope: ProviderRequestEnvelope, _runtime: ui::RuntimeStatus) {
    serial::write_raw_str("OPENAI_PROVIDER_REQUEST_ENVELOPE {\"schema\":\"raios.provider_request_envelope.v0\",\"id\":\"provider_request_envelope.current_boot.");
    serial::write_raw_fmt(format_args!("{:08}", envelope.request_id));
    serial::write_raw_str("\",\"scope\":\"current_boot\",\"classification\":\"local_only\",\"persistence\":\"none\",\"status\":\"local_prewrite_envelope\",\"provider_write\":\"not_attempted\",\"source\":{\"method\":\"");
    serial::write_raw_str(envelope.source_method);
    serial::write_raw_str("\",\"capability\":\"cap.provider.request\",\"risk\":\"export\",\"code_path\":\"seed-kernel/src/openai.rs\"},\"provider\":{\"selected\":\"OPENAI\",\"route\":\"OPENAI DIRECT\",\"host\":\"");
    serial::write_raw_str(API_HOST);
    serial::write_raw_str("\",\"port\":");
    serial::write_raw_fmt(format_args!("{}", API_PORT));
    serial::write_raw_str(",\"method\":\"POST\",\"path\":\"");
    serial::write_raw_str(API_PATH);
    serial::write_raw_str("\",\"model\":\"");
    serial::write_raw_str(MODEL);
    serial::write_raw_str("\"},\"request_body\":{\"schema\":\"openai.responses.request.redacted.v0\",");
    if envelope.context_attached_to_provider_body {
        serial::write_raw_str("\"instructions\":\"fixed_scoped_project_feedback\",\"input\":\"provider_minimal_packet_redacted\",");
    } else {
        serial::write_raw_str("\"user_prompt\":\"present_redacted\",");
    }
    serial::write_raw_str("\"max_output_tokens\":");
    serial::write_raw_fmt(format_args!("{}", envelope.max_output_tokens));
    serial::write_raw_str(",\"store\":false,\"context_attached_to_provider_body\":");
    serial::write_raw_str(bool_str(envelope.context_attached_to_provider_body));
    serial::write_raw_str(",\"body_sha256\":");
    write_raw_sha256(envelope.request_body_hash);
    serial::write_raw_str("},\"secret_state\":{\"api_key_state\":\"");
    serial::write_raw_str(if provider_credential_configured() {
        "set"
    } else {
        "missing"
    });
    serial::write_raw_str("\",\"authorization_header\":\"redacted\",\"api_key_value\":\"not_recorded\"},\"provider_minimal_context\":{\"attached\":");
    serial::write_raw_str(bool_str(envelope.context_attached_to_provider_body));
    serial::write_raw_str(",\"binding_status\":\"");
    serial::write_raw_str(if envelope.context_attached_to_provider_body {
        "bound"
    } else {
        "not_bound"
    });
    serial::write_raw_str("\"},\"trust_snapshot\":{\"provider_trust_state\":\"");
    serial::write_raw_str(envelope.provider_trust_state);
    serial::write_raw_str("\",\"provider_trust_positive\":");
    serial::write_raw_str(bool_str(envelope.provider_trust_positive));
    serial::write_raw_str(",\"development_tls_bypass\":");
    serial::write_raw_str(bool_str(envelope.development_tls_bypass));
    serial::write_raw_str("},\"evidence\":{\"canonicalization\":\"raios.provider_request_envelope.canonical.v0\",\"envelope_hash\":");
    write_raw_sha256(envelope.envelope_hash);
    serial::write_raw_str("}}\r\n");
}

fn write_raw_sha256(hash: [u8; 32]) {
    serial::write_raw_str("\"sha256:");
    let mut idx = 0usize;
    while idx < hash.len() {
        serial::write_raw_fmt(format_args!("{:02x}", hash[idx]));
        idx += 1;
    }
    serial::write_raw_str("\"");
}

fn bool_str(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn record_positive_provider_context_bindings(
    envelope: ProviderRequestEnvelope,
    envelope_event_id: event_log::EventId,
    runtime: ui::RuntimeStatus,
    trust: provider_trust::Snapshot,
    scoped_context: Option<event_log::ProviderContextHashes>,
    context_attached_to_provider_body: bool,
) {
    if trust.development_bypass || !provider_trust_positive(trust.state) {
        return;
    }

    let context_hashes = scoped_context.unwrap_or_else(|| {
        agent_protocol::provider_minimal_context_evidence_for_runtime(runtime).event_hashes()
    });
    let provider_trust_state = trust.state.as_protocol();
    let trust_evidence_hash = provider_trust_evidence_hash(trust);
    let request_binding_hash = provider_request_binding_hash(
        envelope,
        envelope_event_id,
        context_hashes,
        provider_trust_state,
        trust_evidence_hash,
        context_attached_to_provider_body,
    );
    let request_binding = event_log::ProviderRequestBinding {
        request_id: envelope.request_id,
        request_envelope_event_id: envelope_event_id,
        request_body_hash: envelope.request_body_hash,
        request_envelope_hash: envelope.envelope_hash,
        request_binding_hash,
        context: context_hashes,
        provider_trust_state,
        provider_trust_pin_kind: trust.pin_kind,
        provider_trust_pin_id: trust.pin_id,
        provider_trust_pin_slot: trust.pin_slot,
        provider_trust_pin_rotation_policy: trust.pin_rotation_policy,
        provider_trust_pin_rotation_id: trust.pin_rotation_id,
        provider_trust_verifier: trust.verifier,
        provider_trust_verifier_decision: trust.verifier_decision,
        provider_trust_evidence_hash: trust_evidence_hash,
        development_tls_bypass: trust.development_bypass,
    };
    let request_binding_event_id =
        event_log::record_provider_request_binding_bound(request_binding);
    emit_provider_request_binding(
        request_binding,
        request_binding_event_id,
        context_attached_to_provider_body,
    );

    let export_audit_binding_hash = provider_export_audit_binding_hash(
        request_binding,
        request_binding_event_id,
        provider_trust_state,
        trust_evidence_hash,
        context_attached_to_provider_body,
    );
    let export_binding = event_log::ProviderExportAuditBinding {
        request_id: envelope.request_id,
        request_envelope_event_id: envelope_event_id,
        request_binding_event_id,
        request_body_hash: envelope.request_body_hash,
        request_envelope_hash: envelope.envelope_hash,
        request_binding_hash,
        export_audit_binding_hash,
        context: context_hashes,
        provider_trust_state,
        provider_trust_pin_kind: trust.pin_kind,
        provider_trust_pin_id: trust.pin_id,
        provider_trust_pin_slot: trust.pin_slot,
        provider_trust_pin_rotation_policy: trust.pin_rotation_policy,
        provider_trust_pin_rotation_id: trust.pin_rotation_id,
        provider_trust_verifier: trust.verifier,
        provider_trust_verifier_decision: trust.verifier_decision,
        provider_trust_evidence_hash: trust_evidence_hash,
        context_attached_to_provider_body,
    };
    let export_audit_event_id =
        event_log::record_provider_context_export_audit_binding_bound(export_binding);
    emit_provider_export_audit_binding(export_binding, export_audit_event_id);
}

fn emit_provider_context_injection_gate_blocked(
    envelope: ProviderRequestEnvelope,
    runtime: ui::RuntimeStatus,
    trust: provider_trust::Snapshot,
) {
    if trust.development_bypass || !provider_trust_positive(trust.state) {
        return;
    }

    let context = agent_protocol::provider_minimal_context_evidence_for_runtime(runtime);
    serial::write_raw_str("OPENAI_PROVIDER_CONTEXT_INJECTION_GATE {\"schema\":\"raios.provider_context_injection_gate.v0\",\"scope\":\"current_boot\",\"classification\":\"local_only\",\"status\":\"blocked\",\"reason\":\"automatic_context_injection_disabled\",\"request_id\":");
    serial::write_raw_fmt(format_args!("{}", envelope.request_id));
    serial::write_raw_str(",\"request_body_hash\":");
    write_raw_sha256(envelope.request_body_hash);
    serial::write_raw_str(",\"request_envelope_hash\":");
    write_raw_sha256(envelope.envelope_hash);
    serial::write_raw_str(",\"provider_trust_state\":\"");
    serial::write_raw_str(trust.state.as_protocol());
    serial::write_raw_str("\",\"provider_trust_positive\":true,\"provider_trust_evidence_hash\":");
    write_raw_sha256(provider_trust_evidence_hash(trust));
    serial::write_raw_str(",\"provider_trust_verifier\":");
    write_raw_provider_trust_verifier(trust.verifier);
    serial::write_raw_str(",\"provider_trust_verifier_decision\":");
    write_raw_provider_trust_verifier_decision(trust.verifier_decision);
    serial::write_raw_str(",\"final_authorization_schema\":\"raios.provider_context_injection_authorization.v0\",\"final_authorization\":\"missing\",\"satisfies_current_boot_export_gate\":false,\"automatic_context_injection\":\"disabled\",\"context_attached_to_provider_body\":false,\"provider_write\":\"not_attempted\",\"can_attach_context\":false,\"hashes\":");
    write_raw_context_hashes(context.event_hashes(), false);
    serial::write_raw_str("}\r\n");
}

fn provider_trust_evidence_hash(trust: provider_trust::Snapshot) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_field(
        &mut hash,
        "domain",
        "raios.provider_trust_evidence.canonical.v0",
    );
    hash_field(&mut hash, "schema", "raios.provider_trust_evidence.v0");
    hash_field(&mut hash, "scope", "single_provider_request");
    hash_field(&mut hash, "classification", "local_only");
    hash_field(&mut hash, "provider.selected", "OPENAI");
    hash_field(&mut hash, "provider.host", API_HOST);
    hash_field(&mut hash, "provider.port", "443");
    hash_field(&mut hash, "provider_trust_state", trust.state.as_protocol());
    hash_field(
        &mut hash,
        "provider_trust_positive",
        bool_str(provider_trust_positive(trust.state)),
    );
    hash_field(
        &mut hash,
        "provider_trust_pin_kind",
        trust.pin_kind.unwrap_or("none"),
    );
    hash_field(
        &mut hash,
        "provider_trust_pin_id",
        trust.pin_id.unwrap_or("none"),
    );
    hash_field(
        &mut hash,
        "provider_trust_pin_slot",
        trust.pin_slot.unwrap_or("none"),
    );
    hash_field(
        &mut hash,
        "provider_trust_pin_rotation_policy",
        trust.pin_rotation_policy,
    );
    hash_field(
        &mut hash,
        "provider_trust_pin_rotation_id",
        trust.pin_rotation_id.unwrap_or("none"),
    );
    hash_field(&mut hash, "verifier.schema", trust.verifier.schema);
    hash_field(&mut hash, "verifier.id", trust.verifier.id);
    hash_field(&mut hash, "verifier.host", trust.verifier.host);
    hash_field(&mut hash, "verifier.port", trust.verifier.port);
    hash_field(&mut hash, "verifier.transport", trust.verifier.transport);
    hash_field(
        &mut hash,
        "verifier.hostname_policy",
        trust.verifier.hostname_policy,
    );
    hash_field(&mut hash, "verifier.pin_policy", trust.verifier.pin_policy);
    hash_field(
        &mut hash,
        "verifier.chain_policy",
        trust.verifier.chain_policy,
    );
    hash_field(
        &mut hash,
        "verifier.time_policy",
        trust.verifier.time_policy,
    );
    hash_field(
        &mut hash,
        "verifier.certificate_verify_policy",
        trust.verifier.certificate_verify_policy,
    );
    hash_field(
        &mut hash,
        "verifier_decision.schema",
        trust.verifier_decision.schema,
    );
    hash_field(
        &mut hash,
        "verifier_decision.verifier_id",
        trust.verifier_decision.verifier_id,
    );
    hash_field(
        &mut hash,
        "verifier_decision.stage",
        trust.verifier_decision.stage,
    );
    hash_field(
        &mut hash,
        "verifier_decision.outcome",
        trust.verifier_decision.outcome,
    );
    hash_field(
        &mut hash,
        "verifier_decision.reason",
        trust.verifier_decision.reason,
    );
    hash_field(
        &mut hash,
        "development_tls_bypass",
        bool_str(trust.development_bypass),
    );
    hash.finalize().into()
}

fn provider_request_binding_hash(
    envelope: ProviderRequestEnvelope,
    envelope_event_id: event_log::EventId,
    context: event_log::ProviderContextHashes,
    provider_trust_state: &'static str,
    provider_trust_evidence_hash: [u8; 32],
    context_attached_to_provider_body: bool,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_field(
        &mut hash,
        "domain",
        "raios.provider_request_binding.canonical.v0",
    );
    hash_field(&mut hash, "schema", "raios.provider_request_binding.v0");
    hash_field(&mut hash, "status", "bound");
    hash_field(&mut hash, "scope", "current_boot");
    hash_field(
        &mut hash,
        "request.id",
        format!("{}", envelope.request_id).as_str(),
    );
    hash_field(
        &mut hash,
        "request_envelope_event.sequence",
        format!("{}", envelope_event_id.sequence()).as_str(),
    );
    hash_hash_field(&mut hash, "request_envelope_hash", envelope.envelope_hash);
    hash_hash_field(&mut hash, "request_body_hash", envelope.request_body_hash);
    hash_hash_field(
        &mut hash,
        "projected_packet_hash",
        context.projected_packet_hash,
    );
    hash_hash_field(
        &mut hash,
        "exported_field_list_hash",
        context.exported_field_list_hash,
    );
    hash_hash_field(
        &mut hash,
        "omitted_field_list_hash",
        context.omitted_field_list_hash,
    );
    hash_hash_field(
        &mut hash,
        "redaction_policy_hash",
        context.redaction_policy_hash,
    );
    hash_hash_field(
        &mut hash,
        "field_classification_hash",
        context.field_classification_hash,
    );
    hash_hash_field(&mut hash, "token_budget_hash", context.token_budget_hash);
    hash_field(
        &mut hash,
        "provider_trust_state_at_binding",
        provider_trust_state,
    );
    hash_hash_field(
        &mut hash,
        "provider_trust_evidence_hash",
        provider_trust_evidence_hash,
    );
    hash_field(&mut hash, "development_tls_bypass", "false");
    hash_field(&mut hash, "provider_write_at_binding", "not_attempted");
    hash_field(
        &mut hash,
        "context_attached_to_provider_body",
        bool_str(context_attached_to_provider_body),
    );
    hash.finalize().into()
}

fn provider_export_audit_binding_hash(
    request_binding: event_log::ProviderRequestBinding,
    request_binding_event_id: event_log::EventId,
    provider_trust_state: &'static str,
    provider_trust_evidence_hash: [u8; 32],
    context_attached_to_provider_body: bool,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_field(
        &mut hash,
        "domain",
        "raios.provider_context_export_audit_binding.canonical.v0",
    );
    hash_field(
        &mut hash,
        "schema",
        "raios.provider_context_export_audit_binding.v0",
    );
    hash_field(
        &mut hash,
        "status",
        "authorized_for_single_provider_request",
    );
    hash_field(&mut hash, "scope", "current_boot");
    hash_field(
        &mut hash,
        "request.id",
        format!("{}", request_binding.request_id).as_str(),
    );
    hash_field(
        &mut hash,
        "request_binding_event.sequence",
        format!("{}", request_binding_event_id.sequence()).as_str(),
    );
    hash_hash_field(
        &mut hash,
        "request_binding_hash",
        request_binding.request_binding_hash,
    );
    hash_hash_field(
        &mut hash,
        "request_envelope_hash",
        request_binding.request_envelope_hash,
    );
    hash_hash_field(
        &mut hash,
        "request_body_hash",
        request_binding.request_body_hash,
    );
    hash_hash_field(
        &mut hash,
        "projected_packet_hash",
        request_binding.context.projected_packet_hash,
    );
    hash_hash_field(
        &mut hash,
        "exported_field_list_hash",
        request_binding.context.exported_field_list_hash,
    );
    hash_hash_field(
        &mut hash,
        "omitted_field_list_hash",
        request_binding.context.omitted_field_list_hash,
    );
    hash_hash_field(
        &mut hash,
        "redaction_policy_hash",
        request_binding.context.redaction_policy_hash,
    );
    hash_hash_field(
        &mut hash,
        "field_classification_hash",
        request_binding.context.field_classification_hash,
    );
    hash_hash_field(
        &mut hash,
        "token_budget_hash",
        request_binding.context.token_budget_hash,
    );
    hash_field(
        &mut hash,
        "provider_trust_state_at_binding",
        provider_trust_state,
    );
    hash_hash_field(
        &mut hash,
        "provider_trust_evidence_hash",
        provider_trust_evidence_hash,
    );
    hash_field(&mut hash, "positive_export_authorization", "true");
    hash_field(
        &mut hash,
        "context_attached_to_provider_body",
        bool_str(context_attached_to_provider_body),
    );
    hash_field(
        &mut hash,
        "automatic_context_injection",
        if context_attached_to_provider_body {
            "deliberate_scoped_feedback_only"
        } else {
            "disabled"
        },
    );
    hash.finalize().into()
}

fn emit_provider_request_binding(
    binding: event_log::ProviderRequestBinding,
    event_id: event_log::EventId,
    context_attached_to_provider_body: bool,
) {
    serial::write_raw_str("OPENAI_PROVIDER_REQUEST_BINDING {\"schema\":\"raios.provider_request_binding.v0\",\"id\":\"provider_request_binding.current_boot.");
    serial::write_raw_fmt(format_args!("{:08}", event_id.sequence()));
    serial::write_raw_str("\",\"event_id\":\"event.current_boot.");
    serial::write_raw_fmt(format_args!("{:08}", event_id.sequence()));
    serial::write_raw_str("\",\"status\":\"bound\",\"satisfies_request_binding_gate\":true,\"satisfies_current_boot_export_gate\":");
    serial::write_raw_str(bool_str(context_attached_to_provider_body));
    serial::write_raw_str(",\"provider_write_at_binding\":\"not_attempted\",\"context_attached_to_provider_body\":");
    serial::write_raw_str(bool_str(context_attached_to_provider_body));
    serial::write_raw_str(",\"request_id\":");
    serial::write_raw_fmt(format_args!("{}", binding.request_id));
    serial::write_raw_str(",\"request_envelope_event_id\":\"event.current_boot.");
    serial::write_raw_fmt(format_args!(
        "{:08}",
        binding.request_envelope_event_id.sequence()
    ));
    serial::write_raw_str("\",\"request_body_hash\":");
    write_raw_sha256(binding.request_body_hash);
    serial::write_raw_str(",\"request_envelope_hash\":");
    write_raw_sha256(binding.request_envelope_hash);
    serial::write_raw_str(",\"request_binding_hash\":");
    write_raw_sha256(binding.request_binding_hash);
    serial::write_raw_str(",\"trust_snapshot\":{\"provider_trust_state\":\"");
    serial::write_raw_str(binding.provider_trust_state);
    serial::write_raw_str("\",\"provider_trust_positive\":true,\"provider_trust_pin_kind\":");
    write_raw_opt_str(binding.provider_trust_pin_kind);
    serial::write_raw_str(",\"provider_trust_pin_id\":");
    write_raw_opt_str(binding.provider_trust_pin_id);
    serial::write_raw_str(",\"provider_trust_pin_slot\":");
    write_raw_opt_str(binding.provider_trust_pin_slot);
    serial::write_raw_str(",\"provider_trust_pin_rotation_policy\":\"");
    serial::write_raw_str(binding.provider_trust_pin_rotation_policy);
    serial::write_raw_str("\",\"provider_trust_pin_rotation_id\":");
    write_raw_opt_str(binding.provider_trust_pin_rotation_id);
    serial::write_raw_str(",\"provider_trust_verifier\":");
    write_raw_provider_trust_verifier(binding.provider_trust_verifier);
    serial::write_raw_str(",\"provider_trust_verifier_decision\":");
    write_raw_provider_trust_verifier_decision(binding.provider_trust_verifier_decision);
    serial::write_raw_str(",\"provider_trust_evidence_hash\":");
    write_raw_sha256(binding.provider_trust_evidence_hash);
    serial::write_raw_str(",\"development_tls_bypass\":");
    serial::write_raw_str(bool_str(binding.development_tls_bypass));
    serial::write_raw_str("},\"hashes\":");
    write_raw_context_hashes(binding.context, context_attached_to_provider_body);
    serial::write_raw_str("}\r\n");
}

fn emit_provider_export_audit_binding(
    binding: event_log::ProviderExportAuditBinding,
    event_id: event_log::EventId,
) {
    serial::write_raw_str("OPENAI_PROVIDER_EXPORT_AUDIT_BINDING {\"schema\":\"raios.provider_context_export_audit_binding.v0\",\"id\":\"provider_context_export_audit_binding.current_boot.");
    serial::write_raw_fmt(format_args!("{:08}", event_id.sequence()));
    serial::write_raw_str("\",\"event_id\":\"event.current_boot.");
    serial::write_raw_fmt(format_args!("{:08}", event_id.sequence()));
    serial::write_raw_str("\",\"status\":\"authorized_for_single_provider_request\",\"satisfies_export_audit_binding_gate\":true,\"satisfies_current_boot_export_gate\":");
    serial::write_raw_str(bool_str(binding.context_attached_to_provider_body));
    serial::write_raw_str(",\"positive_export_authorization\":true,\"automatic_context_injection\":\"");
    serial::write_raw_str(if binding.context_attached_to_provider_body {
        "deliberate_scoped_feedback_only"
    } else {
        "disabled"
    });
    serial::write_raw_str("\",\"provider_write_at_binding\":\"not_attempted\",\"context_attached_to_provider_body\":");
    serial::write_raw_str(bool_str(binding.context_attached_to_provider_body));
    serial::write_raw_str(",\"request_id\":");
    serial::write_raw_fmt(format_args!("{}", binding.request_id));
    serial::write_raw_str(",\"request_envelope_event_id\":\"event.current_boot.");
    serial::write_raw_fmt(format_args!(
        "{:08}",
        binding.request_envelope_event_id.sequence()
    ));
    serial::write_raw_str("\",\"request_binding_event_id\":\"event.current_boot.");
    serial::write_raw_fmt(format_args!(
        "{:08}",
        binding.request_binding_event_id.sequence()
    ));
    serial::write_raw_str("\",\"request_body_hash\":");
    write_raw_sha256(binding.request_body_hash);
    serial::write_raw_str(",\"request_envelope_hash\":");
    write_raw_sha256(binding.request_envelope_hash);
    serial::write_raw_str(",\"request_binding_hash\":");
    write_raw_sha256(binding.request_binding_hash);
    serial::write_raw_str(",\"export_audit_binding_hash\":");
    write_raw_sha256(binding.export_audit_binding_hash);
    serial::write_raw_str(",\"trust_snapshot\":{\"provider_trust_state\":\"");
    serial::write_raw_str(binding.provider_trust_state);
    serial::write_raw_str("\",\"provider_trust_positive\":true,\"provider_trust_pin_kind\":");
    write_raw_opt_str(binding.provider_trust_pin_kind);
    serial::write_raw_str(",\"provider_trust_pin_id\":");
    write_raw_opt_str(binding.provider_trust_pin_id);
    serial::write_raw_str(",\"provider_trust_pin_slot\":");
    write_raw_opt_str(binding.provider_trust_pin_slot);
    serial::write_raw_str(",\"provider_trust_pin_rotation_policy\":\"");
    serial::write_raw_str(binding.provider_trust_pin_rotation_policy);
    serial::write_raw_str("\",\"provider_trust_pin_rotation_id\":");
    write_raw_opt_str(binding.provider_trust_pin_rotation_id);
    serial::write_raw_str(",\"provider_trust_verifier\":");
    write_raw_provider_trust_verifier(binding.provider_trust_verifier);
    serial::write_raw_str(",\"provider_trust_verifier_decision\":");
    write_raw_provider_trust_verifier_decision(binding.provider_trust_verifier_decision);
    serial::write_raw_str(",\"provider_trust_evidence_hash\":");
    write_raw_sha256(binding.provider_trust_evidence_hash);
    serial::write_raw_str(",\"development_tls_bypass\":false},\"hashes\":");
    write_raw_context_hashes(
        binding.context,
        binding.context_attached_to_provider_body,
    );
    serial::write_raw_str("}\r\n");
}

fn write_raw_opt_str(value: Option<&str>) {
    match value {
        Some(value) => {
            serial::write_raw_str("\"");
            serial::write_raw_str(value);
            serial::write_raw_str("\"");
        }
        None => serial::write_raw_str("null"),
    }
}

fn write_raw_provider_trust_verifier(metadata: provider_trust::ProviderTrustVerifierMetadata) {
    serial::write_raw_str("{\"schema\":\"");
    serial::write_raw_str(metadata.schema);
    serial::write_raw_str("\",\"id\":\"");
    serial::write_raw_str(metadata.id);
    serial::write_raw_str("\",\"host\":\"");
    serial::write_raw_str(metadata.host);
    serial::write_raw_str("\",\"port\":\"");
    serial::write_raw_str(metadata.port);
    serial::write_raw_str("\",\"transport\":\"");
    serial::write_raw_str(metadata.transport);
    serial::write_raw_str("\",\"hostname_policy\":\"");
    serial::write_raw_str(metadata.hostname_policy);
    serial::write_raw_str("\",\"pin_policy\":\"");
    serial::write_raw_str(metadata.pin_policy);
    serial::write_raw_str("\",\"chain_policy\":\"");
    serial::write_raw_str(metadata.chain_policy);
    serial::write_raw_str("\",\"time_policy\":\"");
    serial::write_raw_str(metadata.time_policy);
    serial::write_raw_str("\",\"certificate_verify_policy\":\"");
    serial::write_raw_str(metadata.certificate_verify_policy);
    serial::write_raw_str("\"}");
}

fn write_raw_provider_trust_verifier_decision(
    decision: provider_trust::ProviderTrustVerifierDecision,
) {
    serial::write_raw_str("{\"schema\":\"");
    serial::write_raw_str(decision.schema);
    serial::write_raw_str("\",\"verifier_id\":\"");
    serial::write_raw_str(decision.verifier_id);
    serial::write_raw_str("\",\"stage\":\"");
    serial::write_raw_str(decision.stage);
    serial::write_raw_str("\",\"outcome\":\"");
    serial::write_raw_str(decision.outcome);
    serial::write_raw_str("\",\"reason\":\"");
    serial::write_raw_str(decision.reason);
    serial::write_raw_str("\"}");
}

fn write_raw_context_hashes(
    context: event_log::ProviderContextHashes,
    scoped_feedback: bool,
) {
    serial::write_raw_str("{\"packet_canonicalization\":\"");
    serial::write_raw_str(if scoped_feedback {
        "raios.scoped_project_feedback.packet.canonical_json.v1"
    } else {
        "raios.provider_minimal.packet.canonical.v0"
    });
    serial::write_raw_str("\",\"projected_packet_hash\":");
    write_raw_sha256(context.projected_packet_hash);
    serial::write_raw_str(",\"exported_field_list_hash\":");
    write_raw_sha256(context.exported_field_list_hash);
    serial::write_raw_str(",\"omitted_field_list_hash\":");
    write_raw_sha256(context.omitted_field_list_hash);
    serial::write_raw_str(",\"redaction_policy_hash\":");
    write_raw_sha256(context.redaction_policy_hash);
    serial::write_raw_str(",\"field_classification_hash\":");
    write_raw_sha256(context.field_classification_hash);
    serial::write_raw_str(",\"token_budget_hash\":");
    write_raw_sha256(context.token_budget_hash);
    serial::write_raw_str("}");
}

fn build_request_body_with_context(
    request_id: u32,
    max_output_tokens: u16,
    target: provider::RequestTarget,
    trust: provider_trust::Snapshot,
    authorization: &mut ScopedFeedbackAuthorization,
) -> Result<AuthorizedScopedRequest, &'static str> {
    if authorization.consumed {
        return Err("single_use_export_authorization_consumed");
    }

    let packet_json = build_feedback_packet_json(authorization.packet);
    let packet_hash = hash_bytes(packet_json.as_bytes());
    let packet_estimated_tokens = (packet_json.len() as u64).saturating_add(3) / 4;
    let packet_all_records_public =
        agent_build_loop::feedback_packet_is_public(authorization.packet);
    let body = build_request_body_with_packet(
        SCOPED_PROJECT_FEEDBACK_PROMPT,
        max_output_tokens,
        packet_json.as_str(),
    );
    let request_body_hash = hash_bytes(body.as_bytes());
    let provider_trust_state = trust.state.as_protocol();
    let (source_method, export_method) = match target {
        provider::RequestTarget::ProjectFeedbackWorkspace => {
            ("project.feedback_export", "provider.context_export")
        }
        _ => (
            "provider.context_export.invalid_target",
            "provider.context_export.invalid_target",
        ),
    };
    let request_envelope_hash = provider_request_envelope_hash(
        request_id,
        max_output_tokens,
        source_method,
        request_body_hash,
        provider_trust_state,
        provider_trust_positive(trust.state),
        trust.development_bypass,
        true,
    );
    let audit_binding_hash = scoped_feedback_audit_binding_hash(
        request_id,
        packet_hash,
        request_body_hash,
        request_envelope_hash,
        trust,
    );
    let gate_input = ProviderExportGateInput {
        method: Some(export_method),
        profile: Some(SCOPED_FEEDBACK_PROFILE),
        trust_state: Some(provider_trust_state),
        tls_certificate_verification_bypassed: trust.development_bypass,
        packet_all_records_public,
        packet_record_count: Some(SCOPED_FEEDBACK_RECORD_COUNT),
        budget_tokens: Some(SCOPED_FEEDBACK_BUDGET_TOKENS),
        packet_estimated_tokens: Some(packet_estimated_tokens),
        audit_packet_hash: Some(packet_hash),
        audit_destination: Some(SCOPED_FEEDBACK_DESTINATION),
        audit_trust_snapshot_present: scoped_feedback_trust_snapshot_present(trust),
        audit_binding_hash: Some(audit_binding_hash),
    };
    let gate_decision = evaluate_provider_export_gate(&gate_input);
    if !gate_decision.performed {
        let _ = memory_store::record_provider_export_denial_audit(
            gate_decision.reason,
            SCOPED_FEEDBACK_PROFILE,
            provider_trust_state,
        );
        emit_scoped_feedback_export_denied(
            request_id,
            gate_decision.reason,
            request_body_hash,
            packet_hash,
            false,
        );
        return Err(gate_decision.reason);
    }

    let audit = memory_store::record_live_provider_export_audit(
        request_id,
        packet_hash,
        SCOPED_FEEDBACK_RECORD_COUNT,
        SCOPED_FEEDBACK_DESTINATION,
        provider_trust_state,
        SCOPED_FEEDBACK_BUDGET_TOKENS,
        packet_estimated_tokens,
        request_body_hash,
        request_envelope_hash,
        audit_binding_hash,
        provider_trust_evidence_hash(trust),
    );
    if !audit.performed
        || !audit.reparse_valid
        || audit.seq.is_none()
        || audit.payload_sha256.is_none()
        || audit.frame_sha256.is_none()
        || audit.frame_sha256 != audit.readback_sha256
    {
        emit_scoped_feedback_export_denied(
            request_id,
            "durable_export_audit_append_failed",
            request_body_hash,
            packet_hash,
            false,
        );
        return Err("durable_export_audit_append_failed");
    }

    authorization.consumed = true;
    Ok(AuthorizedScopedRequest {
        body,
        envelope: ProviderRequestEnvelope {
            request_id,
            max_output_tokens,
            source_method,
            request_body_hash,
            envelope_hash: request_envelope_hash,
            provider_trust_state,
            provider_trust_positive: true,
            development_tls_bypass: false,
            context_attached_to_provider_body: true,
        },
        context: scoped_feedback_context_hashes(packet_hash, packet_estimated_tokens),
        gate_decision,
        audit_binding_hash,
        packet_hash,
        packet_estimated_tokens,
        audit,
    })
}

fn build_feedback_packet_json(packet: FeedbackPacket) -> String {
    let mut out = String::from("{\"profile\":\"provider_minimal\",\"records\":[{\"field\":\"check_id\",\"classification\":\"public\",\"value\":\"");
    push_json_string(&mut out, packet.check_id);
    out.push_str("\"},{\"field\":\"revision_sha256\",\"classification\":\"public\",\"value\":\"");
    push_sha256_string(&mut out, packet.revision_sha256);
    out.push_str("\"},{\"field\":\"tree_sha256\",\"classification\":\"public\",\"value\":\"");
    push_sha256_string(&mut out, packet.tree_sha256);
    out.push_str("\"},{\"field\":\"reason\",\"classification\":\"public\",\"value\":\"");
    push_json_string(&mut out, packet.reason);
    out.push_str("\"}]}");
    out
}

fn build_request_body_with_packet(
    instructions: &str,
    max_output_tokens: u16,
    packet_json: &str,
) -> String {
    let mut body = String::new();
    body.push_str("{\"model\":\"");
    body.push_str(MODEL);
    body.push_str("\",\"instructions\":\"");
    push_json_string(&mut body, instructions);
    body.push_str("\",\"input\":\"");
    push_json_string(&mut body, packet_json);
    body.push_str("\",\"max_output_tokens\":");
    body.push_str(format!("{}", max_output_tokens).as_str());
    body.push_str(",\"store\":false}");
    body
}

fn push_sha256_string(out: &mut String, value: [u8; 32]) {
    out.push_str("sha256:");
    let mut idx = 0usize;
    while idx < value.len() {
        let _ = write!(out, "{:02x}", value[idx]);
        idx += 1;
    }
}

fn scoped_feedback_trust_snapshot_present(trust: provider_trust::Snapshot) -> bool {
    !trust.verifier.schema.is_empty()
        && !trust.verifier.id.is_empty()
        && trust.verifier.host == API_HOST
        && trust.verifier.port == "443"
        && !trust.verifier.transport.is_empty()
        && !trust.verifier.hostname_policy.is_empty()
        && !trust.verifier.pin_policy.is_empty()
        && !trust.verifier.chain_policy.is_empty()
        && !trust.verifier.time_policy.is_empty()
        && !trust.verifier.certificate_verify_policy.is_empty()
        && !trust.verifier_decision.schema.is_empty()
        && trust.verifier_decision.verifier_id == trust.verifier.id
        && !trust.verifier_decision.stage.is_empty()
        && trust.verifier_decision.outcome == "verified"
        && !trust.verifier_decision.reason.is_empty()
}

fn scoped_feedback_audit_binding_hash(
    request_id: u32,
    packet_hash: [u8; 32],
    request_body_hash: [u8; 32],
    request_envelope_hash: [u8; 32],
    trust: provider_trust::Snapshot,
) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash_field(
        &mut hash,
        "domain",
        "raios.scoped_project_feedback_export_audit.canonical.v0",
    );
    hash_field(&mut hash, "request.id", format!("{}", request_id).as_str());
    hash_hash_field(&mut hash, "packet_hash", packet_hash);
    hash_hash_field(&mut hash, "request_body_hash", request_body_hash);
    hash_hash_field(&mut hash, "request_envelope_hash", request_envelope_hash);
    hash_field(&mut hash, "destination", SCOPED_FEEDBACK_DESTINATION);
    hash_field(&mut hash, "provider_trust_state", trust.state.as_protocol());
    hash_hash_field(
        &mut hash,
        "provider_trust_evidence_hash",
        provider_trust_evidence_hash(trust),
    );
    hash_field(
        &mut hash,
        "development_tls_bypass",
        bool_str(trust.development_bypass),
    );
    hash.finalize().into()
}

fn scoped_feedback_context_hashes(
    packet_hash: [u8; 32],
    packet_estimated_tokens: u64,
) -> event_log::ProviderContextHashes {
    let mut budget = Sha256::new();
    hash_field(
        &mut budget,
        "budget_tokens",
        format!("{}", SCOPED_FEEDBACK_BUDGET_TOKENS).as_str(),
    );
    hash_field(
        &mut budget,
        "packet_estimated_tokens",
        format!("{}", packet_estimated_tokens).as_str(),
    );
    event_log::ProviderContextHashes {
        projected_packet_hash: packet_hash,
        exported_field_list_hash: hash_bytes(
            b"check_id\nrevision_sha256\ntree_sha256\nreason\n",
        ),
        omitted_field_list_hash: hash_bytes(
            b"source_bytes\nfile_paths\nlogs\nstdout\nsecrets\nhardware_facts\nunclassified_text\n",
        ),
        redaction_policy_hash: hash_bytes(b"explicit_four_field_allowlist_public_only.v1"),
        field_classification_hash: hash_bytes(
            b"check_id=public\nrevision_sha256=public\ntree_sha256=public\nreason=public\n",
        ),
        token_budget_hash: budget.finalize().into(),
    }
}

fn emit_scoped_feedback_export_denied(
    request_id: u32,
    reason: &'static str,
    request_body_hash: [u8; 32],
    packet_hash: [u8; 32],
    authorization_consumed: bool,
) {
    serial::write_raw_str("OPENAI_SCOPED_PROJECT_FEEDBACK_EXPORT {\"status\":\"denied\",\"reason\":\"");
    serial::write_raw_str(reason);
    serial::write_raw_str("\",\"request_id\":");
    serial::write_raw_fmt(format_args!("{}", request_id));
    serial::write_raw_str(",\"request_body_hash\":");
    write_raw_sha256(request_body_hash);
    serial::write_raw_str(",\"packet_hash\":");
    write_raw_sha256(packet_hash);
    serial::write_raw_str(",\"context_attached_to_provider_body\":false,\"provider_write\":\"not_attempted\",\"authorization_consumed\":");
    serial::write_raw_str(bool_str(authorization_consumed));
    serial::write_raw_str("}\r\n");
}

fn emit_scoped_feedback_export_authorized(request: &AuthorizedScopedRequest) {
    serial::write_raw_str("OPENAI_SCOPED_PROJECT_FEEDBACK_EXPORT {\"status\":\"authorized\",\"reason\":\"");
    serial::write_raw_str(request.gate_decision.reason);
    serial::write_raw_str("\",\"request_id\":");
    serial::write_raw_fmt(format_args!("{}", request.envelope.request_id));
    serial::write_raw_str(",\"packet_record_count\":4,\"packet_all_records_public\":true,\"budget_tokens\":1000,\"packet_estimated_tokens\":");
    serial::write_raw_fmt(format_args!("{}", request.packet_estimated_tokens));
    serial::write_raw_str(",\"packet_hash\":");
    write_raw_sha256(request.packet_hash);
    serial::write_raw_str(",\"request_body_hash\":");
    write_raw_sha256(request.envelope.request_body_hash);
    serial::write_raw_str(",\"request_envelope_hash\":");
    write_raw_sha256(request.envelope.envelope_hash);
    serial::write_raw_str(",\"audit_binding_hash\":");
    write_raw_sha256(request.audit_binding_hash);
    serial::write_raw_str(",\"durable_audit_payload_sha256\":");
    match request.audit.payload_sha256 {
        Some(hash) => write_raw_sha256(hash),
        None => serial::write_raw_str("null"),
    }
    serial::write_raw_str(",\"durable_audit_seq\":");
    match request.audit.seq {
        Some(seq) => serial::write_raw_fmt(format_args!("{}", seq)),
        None => serial::write_raw_str("null"),
    }
    serial::write_raw_str(",\"durable_audit_reason\":\"");
    serial::write_raw_str(request.audit.reason);
    serial::write_raw_str("\",\"context_attached_to_provider_body\":true,\"provider_write\":\"not_attempted\",\"authorization_consumed\":true}\r\n");
}

enum HttpsResult {
    Answer(String, provider::AnswerProvenance),
    Status(u16, FixedLine),
    Error(&'static [u8]),
}

fn perform_https_request(
    request_id: u32,
    target: provider::RequestTarget,
    request_body: &str,
    envelope: Option<ProviderRequestEnvelope>,
    envelope_event_id: Option<event_log::EventId>,
    mut scoped_feedback: Option<ScopedFeedbackAuthorization>,
    runtime: ui::RuntimeStatus,
    transport_lease: net::TransportLeaseToken,
) -> HttpsResult {
    let trust = provider_trust::snapshot();
    if !provider_credential_usable() {
        return HttpsResult::Error(b"OPENAI DIRECT API KEY MISSING");
    }
    if !trust.allows_provider_request() && !provider_trust::can_attempt_openai_tls() {
        serial::write_fmt(format_args!(
            "openai: TLS trust denied before API key copy: {}\r\n",
            trust.state.as_protocol()
        ));
        return HttpsResult::Error(trust.state.openai_error());
    }

    let mut read_record_buffer = vec![0u8; TLS_RECORD_BUFFER_SIZE];
    let mut write_record_buffer = vec![0u8; TLS_WRITE_BUFFER_SIZE];
    let stream = KernelTcpStream::new(transport_lease);
    let config = TlsConfig::<Aes128GcmSha256>::new()
        .with_server_name(API_HOST)
        .enable_rsa_signatures();
    let mut tls = TlsConnection::new(stream, &mut read_record_buffer, &mut write_record_buffer);

    if trust.development_bypass {
        serial::write_line("openai: TLS 1.3 handshake starting (unverified development override)");
    } else {
        serial::write_line("openai: TLS 1.3 handshake starting (pinned provider verifier)");
    }
    let mut rng = KernelRng;
    let tls_opened = if trust.development_bypass {
        tls.open::<KernelRng, NoVerify>(TlsContext::new(&config, &mut rng))
    } else {
        tls.open::<KernelRng, OpenAiPinnedCertVerifier>(TlsContext::new(&config, &mut rng))
    };
    if tls_opened.is_err() {
        let trust = provider_trust::snapshot();
        return HttpsResult::Error(if trust.allows_provider_request() {
            b"OPENAI DIRECT TLS HANDSHAKE FAILED"
        } else {
            trust.state.openai_error()
        });
    }
    serial::write_line("openai: TLS 1.3 established");
    let trust = provider_trust::snapshot();
    if scoped_feedback.is_none() && !trust.allows_provider_request() {
        serial::write_fmt(format_args!(
            "openai: TLS trust denied before API key copy: {}\r\n",
            trust.state.as_protocol()
        ));
        return HttpsResult::Error(trust.state.openai_error());
    }
    if trust.development_bypass {
        serial::write_line(
            "openai: TLS provider trust state: tls_certificate_verification_bypassed",
        );
    } else if provider_trust_positive(trust.state) {
        let trust_label = match trust.state {
            provider_trust::TrustState::PinnedCertVerified => "pinned_cert",
            provider_trust::TrustState::PinnedSpkiVerified => "pinned_spki",
            provider_trust::TrustState::WebPkiVerified => "webpki",
            _ => "verified",
        };
        if let Some(pin_id) = trust.pin_id {
            serial::write_fmt(format_args!(
                "openai: TLS provider trust verified: {} sha256:{}\r\n",
                trust_label, pin_id
            ));
        } else {
            serial::write_fmt(format_args!(
                "openai: TLS provider trust verified: {}\r\n",
                trust_label
            ));
        }
    } else {
        serial::write_fmt(format_args!(
            "openai: TLS trust pending scoped export gate: {}\r\n",
            trust.state.as_protocol()
        ));
    }

    let mut scoped_body = None;
    let (envelope, _envelope_event_id) = match scoped_feedback.as_mut() {
        Some(authorization) => {
            let authorized = match build_request_body_with_context(
                request_id,
                MAX_OUTPUT_TOKENS,
                target,
                trust,
                authorization,
            ) {
                Ok(authorized) => authorized,
                Err(_) => {
                    return HttpsResult::Error(b"OPENAI SCOPED FEEDBACK EXPORT DENIED")
                }
            };
            let scoped_envelope = authorized.envelope;
            let scoped_envelope_event_id = event_log::record_provider_request_envelope_created(
                scoped_envelope.event_binding(),
            );
            emit_provider_request_envelope(scoped_envelope, runtime);
            record_positive_provider_context_bindings(
                scoped_envelope,
                scoped_envelope_event_id,
                runtime,
                trust,
                Some(authorized.context),
                true,
            );
            emit_scoped_feedback_export_authorized(&authorized);
            scoped_body = Some(authorized.body);
            (scoped_envelope, scoped_envelope_event_id)
        }
        None => {
            let (Some(envelope), Some(envelope_event_id)) = (envelope, envelope_event_id) else {
                return HttpsResult::Error(b"OPENAI DIRECT REQUEST ENVELOPE MISSING");
            };
            if hash_bytes(request_body.as_bytes()) != envelope.request_body_hash {
                return HttpsResult::Error(b"OPENAI DIRECT REQUEST ENVELOPE BODY HASH MISMATCH");
            }
            record_positive_provider_context_bindings(
                envelope,
                envelope_event_id,
                runtime,
                trust,
                None,
                false,
            );
            emit_provider_context_injection_gate_blocked(envelope, runtime, trust);
            (envelope, envelope_event_id)
        }
    };
    let request_body = scoped_body.as_deref().unwrap_or(request_body);
    if hash_bytes(request_body.as_bytes()) != envelope.request_body_hash {
        return HttpsResult::Error(b"OPENAI DIRECT FINAL BODY HASH MISMATCH");
    }
    let final_trust = provider_trust::snapshot();
    if envelope.context_attached_to_provider_body
        && (final_trust.development_bypass
            || !provider_trust_positive(final_trust.state)
            || final_trust.state.as_protocol() != envelope.provider_trust_state)
    {
        emit_scoped_feedback_export_denied(
            envelope.request_id,
            "final_provider_trust_downgraded_before_write",
            envelope.request_body_hash,
            scoped_feedback
                .as_ref()
                .map(|authorization| {
                    hash_bytes(build_feedback_packet_json(authorization.packet).as_bytes())
                })
                .unwrap_or([0; 32]),
            true,
        );
        return HttpsResult::Error(b"OPENAI DIRECT FINAL PROVIDER TRUST DOWNGRADED");
    }

    let vault_credential = matches!(
        secret_vault::provider_status(),
        secret_vault::VaultSecretStatus::Available { .. }
    );
    let vault_lease = if vault_credential {
        let Some(verified) = provider_trust::verified_openai_provider_trust() else {
            return HttpsResult::Error(b"OPENAI DIRECT VAULT TRUST EVIDENCE MISSING");
        };
        match secret_vault::provider_lease_for_verified_request(verified) {
            Ok(lease) => Some(lease),
            Err(_) => return HttpsResult::Error(b"OPENAI DIRECT VAULT USE DENIED"),
        }
    } else {
        None
    };
    let legacy_credential = if vault_credential {
        None
    } else {
        provider_config::legacy_openai_credential()
    };
    if vault_lease.is_none() && legacy_credential.is_none() {
        return HttpsResult::Error(b"OPENAI DIRECT API KEY MISSING");
    }

    let preamble = build_http_preamble();
    if tls.write_all(preamble.as_bytes()).is_err() {
        return HttpsResult::Error(b"OPENAI DIRECT HTTPS WRITE FAILED");
    }
    let authorization_written = match (vault_lease, legacy_credential) {
        (Some(lease), None) => lease.into_openai_authorization_header(&mut tls).is_ok(),
        (None, Some(credential)) => credential
            .into_openai_authorization_header(&mut tls)
            .is_ok(),
        _ => false,
    };

    if !authorization_written
        || tls.write_all(b"Content-Type: application/json\r\nContent-Length: ").is_err()
        || tls
            .write_all(format!("{}", request_body.len()).as_bytes())
            .is_err()
        || tls
            .write_all(b"\r\nAccept: application/json\r\nAccept-Encoding: identity\r\nConnection: close\r\n\r\n")
            .is_err()
        || tls.write_all(request_body.as_bytes()).is_err()
        || tls.flush().is_err()
    {
        return HttpsResult::Error(b"OPENAI DIRECT HTTPS WRITE FAILED");
    }
    serial::write_line("openai: HTTPS request sent");
    if envelope.context_attached_to_provider_body {
        serial::write_raw_str("OPENAI_SCOPED_PROJECT_FEEDBACK_SENT {\"request_id\":");
        serial::write_raw_fmt(format_args!("{}", envelope.request_id));
        serial::write_raw_str(",\"request_body_hash\":");
        write_raw_sha256(envelope.request_body_hash);
        serial::write_raw_str(",\"request_envelope_hash\":");
        write_raw_sha256(envelope.envelope_hash);
        serial::write_raw_str(",\"context_attached_to_provider_body\":true,\"provider_write\":\"performed\",\"authorization_consumed\":true}\r\n");
    }

    let response = match read_http_response(&mut tls) {
        Ok(response) => response,
        Err(error) => return HttpsResult::Error(error),
    };

    let status = parse_status(&response).unwrap_or(0);
    if status != 200 {
        let mut detail = FixedLine::empty();
        if let Some(message) = extract_json_string_after(&response, 0, "message") {
            detail.set_from_bytes(message.as_bytes());
        } else {
            detail.set_from_bytes(b"OPENAI API ERROR");
        }
        return HttpsResult::Status(status, detail);
    }

    let body_start = match find_subslice(&response, b"\r\n\r\n") {
        Some(index) => index + 4,
        None => return HttpsResult::Error(b"OPENAI DIRECT HTTP PARSE FAILED"),
    };
    let body = decoded_body(&response, body_start);
    match extract_json_string_after(&body, 0, "status").as_deref() {
        Some("completed") => {}
        Some(_) => return HttpsResult::Error(b"OPENAI DIRECT RESPONSE INCOMPLETE"),
        None => return HttpsResult::Error(b"OPENAI DIRECT RESPONSE STATUS MISSING"),
    }
    let Some(answer) = extract_output_text(&body) else {
        return HttpsResult::Error(b"OPENAI DIRECT RESPONSE PARSE FAILED");
    };
    HttpsResult::Answer(
        answer,
        provider_answer_provenance(envelope, provider_trust::snapshot()),
    )
}

fn provider_answer_provenance(
    envelope: ProviderRequestEnvelope,
    trust: provider_trust::Snapshot,
) -> provider::AnswerProvenance {
    provider::AnswerProvenance {
        request_body_sha256: envelope.request_body_hash,
        request_envelope_sha256: envelope.envelope_hash,
        observed_verifier_id: trust.verifier.id,
        observed_verifier_state: trust.state,
        observed_verifier_outcome: trust.verifier_decision.outcome,
        chain_policy: trust.verifier.chain_policy,
        time_policy: trust.verifier.time_policy,
        development_tls_bypass: trust.development_bypass,
    }
}

fn build_http_preamble() -> String {
    format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: raiOS/0.1\r\n",
        API_PATH, API_HOST
    )
}

fn provider_credential_configured() -> bool {
    match secret_vault::provider_status() {
        secret_vault::VaultSecretStatus::Available { .. } => true,
        secret_vault::VaultSecretStatus::Missing => provider_config::api_key_set(),
        secret_vault::VaultSecretStatus::Forgotten { .. } => false,
    }
}

fn provider_credential_usable() -> bool {
    match secret_vault::provider_status() {
        secret_vault::VaultSecretStatus::Available { .. } => {
            secret_vault::recovery_state() == secret_vault::VaultRecoveryState::Unlocked
        }
        secret_vault::VaultSecretStatus::Missing => provider_config::api_key_set(),
        secret_vault::VaultSecretStatus::Forgotten { .. } => false,
    }
}

fn build_request_body(prompt: &str, max_output_tokens: u16) -> String {
    let mut body = String::new();
    body.push_str("{\"model\":\"");
    body.push_str(MODEL);
    body.push_str("\",\"input\":\"");
    push_json_string(&mut body, prompt);
    body.push_str("\",\"max_output_tokens\":");
    body.push_str(format!("{}", max_output_tokens).as_str());
    body.push_str(",\"store\":false}");
    body
}

fn push_json_string(out: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
}

fn read_http_response(
    tls: &mut TlsConnection<'_, KernelTcpStream, Aes128GcmSha256>,
) -> Result<Vec<u8>, &'static [u8]> {
    let start = now_ms();
    let mut response = Vec::new();
    let mut scratch = [0u8; 1024];

    loop {
        match tls.read(&mut scratch) {
            Ok(0) => break,
            Ok(read) => {
                if response.len() + read > HTTP_RESPONSE_LIMIT {
                    return Err(b"OPENAI DIRECT HTTP RESPONSE TOO LARGE");
                }
                response.extend_from_slice(&scratch[..read]);
                if http_response_complete(&response) {
                    break;
                }
            }
            Err(_) if !response.is_empty() => break,
            Err(_) => return Err(b"OPENAI DIRECT HTTPS READ FAILED"),
        }

        if now_ms().saturating_sub(start) >= HTTPS_TIMEOUT_MS {
            return Err(b"OPENAI DIRECT HTTPS READ TIMEOUT");
        }
    }

    if response.is_empty() {
        Err(b"OPENAI DIRECT EMPTY HTTPS RESPONSE")
    } else {
        Ok(response)
    }
}

fn decoded_body(response: &[u8], body_start: usize) -> Vec<u8> {
    if header_contains(response, b"transfer-encoding:", b"chunked") {
        decode_chunked(&response[body_start..]).unwrap_or_else(|| response[body_start..].to_vec())
    } else {
        response[body_start..].to_vec()
    }
}

fn decode_chunked(body: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    loop {
        let line_end = find_subslice(&body[offset..], b"\r\n")? + offset;
        let size = parse_hex_usize(trim_ascii(&body[offset..line_end]))?;
        offset = line_end + 2;
        if size == 0 {
            return Some(out);
        }
        if offset + size + 2 > body.len() {
            return None;
        }
        out.extend_from_slice(&body[offset..offset + size]);
        offset += size + 2;
    }
}

fn extract_output_text(body: &[u8]) -> Option<String> {
    let output_text = find_subslice(body, b"\"output_text\"")?;
    extract_json_string_after(body, output_text, "text")
}

fn extract_json_string_after(body: &[u8], start: usize, key: &str) -> Option<String> {
    let key_pattern = format!("\"{}\"", key);
    let key_pos = find_subslice(&body[start..], key_pattern.as_bytes())? + start;
    let mut index = key_pos + key_pattern.len();
    while index < body.len() && body[index].is_ascii_whitespace() {
        index += 1;
    }
    if body.get(index) != Some(&b':') {
        return None;
    }
    index += 1;
    while index < body.len() && body[index].is_ascii_whitespace() {
        index += 1;
    }
    if body.get(index) != Some(&b'"') {
        return None;
    }
    index += 1;

    let mut out = String::new();
    while index < body.len() {
        let byte = body[index];
        index += 1;
        match byte {
            b'"' => return Some(out),
            b'\\' => {
                let escape = *body.get(index)?;
                index += 1;
                match escape {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'b' => out.push(' '),
                    b'f' => out.push(' '),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'u' => index = push_json_unicode_escape(body, index, &mut out)?,
                    _ => out.push('?'),
                }
            }
            b if b.is_ascii() => out.push(b as char),
            _ => {
                let start = index - 1;
                let tail = str::from_utf8(&body[start..]).ok()?;
                let ch = tail.chars().next()?;
                out.push(ch);
                index = start + ch.len_utf8();
            }
        }
    }
    None
}

fn push_json_unicode_escape(body: &[u8], index: usize, out: &mut String) -> Option<usize> {
    let code = read_json_u16(body, index)?;
    let mut next = index + 4;

    let scalar = if (0xD800..=0xDBFF).contains(&code) {
        if body.get(next) == Some(&b'\\') && body.get(next + 1) == Some(&b'u') {
            let low = read_json_u16(body, next + 2)?;
            next += 6;
            if (0xDC00..=0xDFFF).contains(&low) {
                0x10000 + (((u32::from(code) - 0xD800) << 10) | (u32::from(low) - 0xDC00))
            } else {
                u32::from(b'?')
            }
        } else {
            u32::from(b'?')
        }
    } else {
        u32::from(code)
    };

    out.push(core::char::from_u32(scalar).unwrap_or('?'));
    Some(next)
}

fn read_json_u16(body: &[u8], index: usize) -> Option<u16> {
    if index + 4 > body.len() {
        return None;
    }
    let mut value = 0u16;
    for &byte in &body[index..index + 4] {
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return None,
        };
        value = value.checked_mul(16)?.checked_add(u16::from(digit))?;
    }
    Some(value)
}

struct KernelRng;

impl RngCore for KernelRng {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        entropy::take(&mut bytes);
        u32::from_le_bytes(bytes)
    }

    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        entropy::take(&mut bytes);
        u64::from_le_bytes(bytes)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        entropy::take(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl CryptoRng for KernelRng {}
