extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{self, Write};
use core::str;

use embedded_io::Write as IoWrite;
use embedded_tls::blocking::{Aes128GcmSha256, NoVerify, TlsConfig, TlsConnection, TlsContext};
use rand_core::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};
use spin::Mutex;

use crate::{
    agent_protocol, entropy, event_log, net, openai_trust::OpenAiPinnedCertVerifier,
    provider_config, provider_trust, serial, time, tls_io::KernelTcpStream, ui,
};

const API_HOST: &str = "api.openai.com";
const API_PORT: u16 = 443;
const API_PATH: &str = "/v1/responses";
const MODEL: &str = "gpt-5.4";
const MAX_OUTPUT_TOKENS: u16 = 128;
const LINE_CAPACITY: usize = 104;
const DNS_TIMEOUT_MS: u64 = 6_000;
const TCP_TIMEOUT_MS: u64 = 8_000;
const HTTPS_TIMEOUT_MS: u64 = 60_000;
const TLS_RECORD_BUFFER_SIZE: usize = 16_640;
const TLS_WRITE_BUFFER_SIZE: usize = 4_096;
const HTTP_RESPONSE_LIMIT: usize = 32 * 1024;

static STATE: Mutex<OpenAiState> = Mutex::new(OpenAiState::new());

#[derive(Clone, Copy)]
pub enum SubmitError {
    Empty,
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
    phase_started_ms: u64,
    envelope: ProviderRequestEnvelope,
    envelope_event_id: event_log::EventId,
    runtime: ui::RuntimeStatus,
}

#[derive(Clone, Copy)]
struct ProviderRequestEnvelope {
    request_id: u32,
    request_body_hash: [u8; 32],
    envelope_hash: [u8; 32],
    provider_trust_state: &'static str,
    provider_trust_positive: bool,
    development_tls_bypass: bool,
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

pub fn submit_request(prompt: &str, runtime: ui::RuntimeStatus) -> Result<u32, SubmitError> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err(SubmitError::Empty);
    }

    let mut state = STATE.lock();
    if let Some(pending) = state.pending.as_ref() {
        return Err(SubmitError::Busy(pending.id));
    }

    let id = state.next_id;
    state.next_id = state.next_id.wrapping_add(1).max(1);
    let body = build_request_body(prompt);
    let envelope = build_provider_request_envelope(
        id,
        hash_bytes(body.as_bytes()),
        provider_trust::snapshot(),
    );
    let envelope_event_id =
        event_log::record_provider_request_envelope_created(envelope.event_binding());
    state.pending = Some(PendingRequest {
        id,
        phase: Phase::Resolving,
        address: None,
        phase_started_ms: now_ms(),
        envelope,
        envelope_event_id,
        runtime,
    });
    state.last_request_id = Some(id);
    state.last_prompt.set_from_str(prompt);
    state
        .last_event
        .set_from_bytes(b"OPENAI DIRECT: RESOLVING api.openai.com");
    state.last_error.clear();
    net::tcp_abort();
    emit_provider_request_envelope(envelope, runtime);

    serial::write_fmt(format_args!(
        "OPENAI_DIRECT_REQ {} {} {}\r\n",
        id, API_HOST, API_PATH
    ));
    Ok(id)
}

pub fn poll() -> Option<FixedLine> {
    let now = now_ms();
    let (phase, phase_started_ms, address) = {
        let state = STATE.lock();
        let pending = state.pending.as_ref()?;
        (pending.phase, pending.phase_started_ms, pending.address)
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
                match handle_tcp_result(&mut state, net::tcp_connect_ipv4(address, API_PORT)) {
                    TcpAction::None | TcpAction::StartHttps { .. } => None,
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
            let mut state = STATE.lock();
            if now.saturating_sub(phase_started_ms) >= TCP_TIMEOUT_MS {
                return complete_error(&mut state, b"OPENAI DIRECT TCP TIMEOUT");
            }
            match handle_tcp_result(&mut state, net::tcp_connect_ipv4(address, API_PORT)) {
                TcpAction::None => None,
                TcpAction::Event(line) => Some(line),
                TcpAction::StartHttps {
                    prompt,
                    envelope,
                    envelope_event_id,
                    runtime,
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
                        prompt.as_str(),
                        envelope,
                        envelope_event_id,
                        runtime,
                    );
                    net::tcp_abort();
                    let mut state = STATE.lock();
                    match result {
                        HttpsResult::Answer(answer) => {
                            let mut line = FixedLine::empty();
                            let _ = write!(line, "OPENAI: {}", answer.as_str());
                            serial::write_fmt(format_args!(
                                "openai: response text: {}\r\n",
                                answer.as_str()
                            ));
                            state.last_event = line;
                            state.pending = None;
                            Some(line)
                        }
                        HttpsResult::Status(status, detail) => {
                            let mut line = FixedLine::empty();
                            let _ = write!(line, "OPENAI HTTP {} {}", status, detail.as_str());
                            state.last_error = line;
                            state.last_event = line;
                            state.pending = None;
                            Some(line)
                        }
                        HttpsResult::Error(error) => complete_error(&mut state, error),
                    }
                }
            }
        }
        Phase::Requesting => {
            let mut state = STATE.lock();
            if now.saturating_sub(phase_started_ms) >= HTTPS_TIMEOUT_MS {
                net::tcp_abort();
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
    Event(FixedLine),
    StartHttps {
        prompt: FixedLine,
        envelope: ProviderRequestEnvelope,
        envelope_event_id: event_log::EventId,
        runtime: ui::RuntimeStatus,
    },
}

fn handle_tcp_result(state: &mut OpenAiState, result: net::TcpConnectResult) -> TcpAction {
    match result {
        net::TcpConnectResult::Connected => {
            let Some(pending) = state.pending.as_ref() else {
                return TcpAction::Event(
                    complete_error(state, b"OPENAI DIRECT REQUEST ENVELOPE MISSING")
                        .unwrap_or_else(FixedLine::empty),
                );
            };
            TcpAction::StartHttps {
                prompt: state.last_prompt,
                envelope: pending.envelope,
                envelope_event_id: pending.envelope_event_id,
                runtime: pending.runtime,
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
        net::TcpConnectResult::NetworkUnavailable => TcpAction::Event(
            complete_error(state, b"OPENAI DIRECT NETWORK UNAVAILABLE")
                .unwrap_or_else(FixedLine::empty),
        ),
        net::TcpConnectResult::NetworkUnconfigured => TcpAction::Event(
            complete_error(state, b"OPENAI DIRECT NETWORK UNCONFIGURED")
                .unwrap_or_else(FixedLine::empty),
        ),
        net::TcpConnectResult::ConnectError => TcpAction::Event(
            complete_error(state, b"OPENAI DIRECT TCP ERROR").unwrap_or_else(FixedLine::empty),
        ),
    }
}

fn complete_error(state: &mut OpenAiState, message: &[u8]) -> Option<FixedLine> {
    let mut line = FixedLine::empty();
    line.set_from_bytes(message);
    state.last_error = line;
    state.last_event = line;
    state.pending = None;
    Some(line)
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
    request_body_hash: [u8; 32],
    trust: provider_trust::Snapshot,
) -> ProviderRequestEnvelope {
    let provider_trust_state = trust.state.as_protocol();
    let provider_trust_positive = provider_trust_positive(trust.state);
    let development_tls_bypass = trust.development_bypass;
    let envelope_hash = provider_request_envelope_hash(
        request_id,
        request_body_hash,
        provider_trust_state,
        provider_trust_positive,
        development_tls_bypass,
    );

    ProviderRequestEnvelope {
        request_id,
        request_body_hash,
        envelope_hash,
        provider_trust_state,
        provider_trust_positive,
        development_tls_bypass,
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
    request_body_hash: [u8; 32],
    provider_trust_state: &'static str,
    provider_trust_positive: bool,
    development_tls_bypass: bool,
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
    hash_field(&mut hash, "source.method", "ask");
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
    hash_field(&mut hash, "request_body.user_prompt", "present_redacted");
    hash_field(
        &mut hash,
        "request_body.max_output_tokens",
        format!("{}", MAX_OUTPUT_TOKENS).as_str(),
    );
    hash_field(&mut hash, "request_body.store", "false");
    hash_field(
        &mut hash,
        "request_body.context_attached_to_provider_body",
        "false",
    );
    hash_hash_field(&mut hash, "request_body.body_sha256", request_body_hash);
    hash_field(
        &mut hash,
        "secret_state.api_key_state",
        if provider_config::api_key_set() {
            "set"
        } else {
            "missing"
        },
    );
    hash_field(&mut hash, "secret_state.authorization_header", "redacted");
    hash_field(&mut hash, "secret_state.api_key_value", "not_recorded");
    hash_field(&mut hash, "provider_minimal_context.attached", "false");
    hash_field(
        &mut hash,
        "provider_minimal_context.binding_status",
        "not_bound",
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
    serial::write_raw_str("\",\"scope\":\"current_boot\",\"classification\":\"local_only\",\"persistence\":\"none\",\"status\":\"local_prewrite_envelope\",\"provider_write\":\"not_attempted\",\"source\":{\"method\":\"ask\",\"capability\":\"cap.provider.request\",\"risk\":\"export\",\"code_path\":\"seed-kernel/src/openai.rs\"},\"provider\":{\"selected\":\"OPENAI\",\"route\":\"OPENAI DIRECT\",\"host\":\"");
    serial::write_raw_str(API_HOST);
    serial::write_raw_str("\",\"port\":");
    serial::write_raw_fmt(format_args!("{}", API_PORT));
    serial::write_raw_str(",\"method\":\"POST\",\"path\":\"");
    serial::write_raw_str(API_PATH);
    serial::write_raw_str("\",\"model\":\"");
    serial::write_raw_str(MODEL);
    serial::write_raw_str("\"},\"request_body\":{\"schema\":\"openai.responses.request.redacted.v0\",\"user_prompt\":\"present_redacted\",\"max_output_tokens\":");
    serial::write_raw_fmt(format_args!("{}", MAX_OUTPUT_TOKENS));
    serial::write_raw_str(
        ",\"store\":false,\"context_attached_to_provider_body\":false,\"body_sha256\":",
    );
    write_raw_sha256(envelope.request_body_hash);
    serial::write_raw_str("},\"secret_state\":{\"api_key_state\":\"");
    serial::write_raw_str(if provider_config::api_key_set() {
        "set"
    } else {
        "missing"
    });
    serial::write_raw_str("\",\"authorization_header\":\"redacted\",\"api_key_value\":\"not_recorded\"},\"provider_minimal_context\":{\"attached\":false,\"binding_status\":\"not_bound\"},\"trust_snapshot\":{\"provider_trust_state\":\"");
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
) {
    if trust.development_bypass || !provider_trust_positive(trust.state) {
        return;
    }

    let context = agent_protocol::provider_minimal_context_evidence_for_runtime(runtime);
    let context_hashes = context.event_hashes();
    let provider_trust_state = trust.state.as_protocol();
    let request_binding_hash = provider_request_binding_hash(
        envelope,
        envelope_event_id,
        context_hashes,
        provider_trust_state,
    );
    let request_binding = event_log::ProviderRequestBinding {
        request_id: envelope.request_id,
        request_envelope_event_id: envelope_event_id,
        request_body_hash: envelope.request_body_hash,
        request_envelope_hash: envelope.envelope_hash,
        request_binding_hash,
        context: context_hashes,
        provider_trust_state,
        development_tls_bypass: trust.development_bypass,
    };
    let request_binding_event_id =
        event_log::record_provider_request_binding_bound(request_binding);
    emit_provider_request_binding(request_binding, request_binding_event_id);

    let export_audit_binding_hash = provider_export_audit_binding_hash(
        request_binding,
        request_binding_event_id,
        provider_trust_state,
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
        context_attached_to_provider_body: false,
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
    serial::write_raw_str("\",\"provider_trust_positive\":true,\"final_authorization_schema\":\"raios.provider_context_injection_authorization.v0\",\"final_authorization\":\"missing\",\"satisfies_current_boot_export_gate\":false,\"automatic_context_injection\":\"disabled\",\"context_attached_to_provider_body\":false,\"provider_write\":\"not_attempted\",\"can_attach_context\":false,\"hashes\":");
    write_raw_context_hashes(context.event_hashes());
    serial::write_raw_str("}\r\n");
}

fn provider_request_binding_hash(
    envelope: ProviderRequestEnvelope,
    envelope_event_id: event_log::EventId,
    context: event_log::ProviderContextHashes,
    provider_trust_state: &'static str,
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
    hash_field(
        &mut hash,
        "provider_trust_state_at_binding",
        provider_trust_state,
    );
    hash_field(&mut hash, "development_tls_bypass", "false");
    hash_field(&mut hash, "provider_write_at_binding", "not_attempted");
    hash_field(&mut hash, "context_attached_to_provider_body", "false");
    hash.finalize().into()
}

fn provider_export_audit_binding_hash(
    request_binding: event_log::ProviderRequestBinding,
    request_binding_event_id: event_log::EventId,
    provider_trust_state: &'static str,
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
    hash_field(
        &mut hash,
        "provider_trust_state_at_binding",
        provider_trust_state,
    );
    hash_field(&mut hash, "positive_export_authorization", "true");
    hash_field(&mut hash, "context_attached_to_provider_body", "false");
    hash_field(&mut hash, "automatic_context_injection", "disabled");
    hash.finalize().into()
}

fn emit_provider_request_binding(
    binding: event_log::ProviderRequestBinding,
    event_id: event_log::EventId,
) {
    serial::write_raw_str("OPENAI_PROVIDER_REQUEST_BINDING {\"schema\":\"raios.provider_request_binding.v0\",\"id\":\"provider_request_binding.current_boot.");
    serial::write_raw_fmt(format_args!("{:08}", event_id.sequence()));
    serial::write_raw_str("\",\"event_id\":\"event.current_boot.");
    serial::write_raw_fmt(format_args!("{:08}", event_id.sequence()));
    serial::write_raw_str("\",\"status\":\"bound\",\"satisfies_request_binding_gate\":true,\"satisfies_current_boot_export_gate\":false,\"provider_write_at_binding\":\"not_attempted\",\"context_attached_to_provider_body\":false,\"request_id\":");
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
    serial::write_raw_str("\",\"provider_trust_positive\":true,\"development_tls_bypass\":");
    serial::write_raw_str(bool_str(binding.development_tls_bypass));
    serial::write_raw_str("},\"hashes\":");
    write_raw_context_hashes(binding.context);
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
    serial::write_raw_str("\",\"status\":\"authorized_for_single_provider_request\",\"satisfies_export_audit_binding_gate\":true,\"satisfies_current_boot_export_gate\":false,\"positive_export_authorization\":true,\"automatic_context_injection\":\"disabled\",\"provider_write_at_binding\":\"not_attempted\",\"context_attached_to_provider_body\":");
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
    serial::write_raw_str(
        "\",\"provider_trust_positive\":true,\"development_tls_bypass\":false},\"hashes\":",
    );
    write_raw_context_hashes(binding.context);
    serial::write_raw_str("}\r\n");
}

fn write_raw_context_hashes(context: event_log::ProviderContextHashes) {
    serial::write_raw_str("{\"packet_canonicalization\":\"raios.provider_minimal.packet.canonical.v0\",\"projected_packet_hash\":");
    write_raw_sha256(context.projected_packet_hash);
    serial::write_raw_str(",\"exported_field_list_hash\":");
    write_raw_sha256(context.exported_field_list_hash);
    serial::write_raw_str(",\"omitted_field_list_hash\":");
    write_raw_sha256(context.omitted_field_list_hash);
    serial::write_raw_str("}");
}

enum HttpsResult {
    Answer(FixedLine),
    Status(u16, FixedLine),
    Error(&'static [u8]),
}

fn perform_https_request(
    prompt: &str,
    envelope: ProviderRequestEnvelope,
    envelope_event_id: event_log::EventId,
    runtime: ui::RuntimeStatus,
) -> HttpsResult {
    let trust = provider_trust::snapshot();
    if !provider_config::api_key_set() {
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
    let stream = KernelTcpStream::new();
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
    if !trust.allows_provider_request() {
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
    } else {
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
    }

    let body = build_request_body(prompt);
    if hash_bytes(body.as_bytes()) != envelope.request_body_hash {
        return HttpsResult::Error(b"OPENAI DIRECT REQUEST ENVELOPE BODY HASH MISMATCH");
    }
    record_positive_provider_context_bindings(envelope, envelope_event_id, runtime, trust);
    emit_provider_context_injection_gate_blocked(envelope, runtime, trust);

    let mut key = [0u8; 256];
    let Some(key_len) = provider_config::copy_api_key(&mut key) else {
        return HttpsResult::Error(b"OPENAI DIRECT API KEY MISSING");
    };

    let header = build_http_header(key_len, body.len());

    if tls.write_all(header.as_bytes()).is_err()
        || tls.write_all(&key[..key_len]).is_err()
        || tls.write_all(b"\r\nContent-Type: application/json\r\nContent-Length: ").is_err()
        || tls.write_all(format!("{}", body.len()).as_bytes()).is_err()
        || tls
            .write_all(b"\r\nAccept: application/json\r\nAccept-Encoding: identity\r\nConnection: close\r\n\r\n")
            .is_err()
        || tls.write_all(body.as_bytes()).is_err()
        || tls.flush().is_err()
    {
        return HttpsResult::Error(b"OPENAI DIRECT HTTPS WRITE FAILED");
    }
    serial::write_line("openai: HTTPS request sent");

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
    let Some(answer) = extract_output_text(&body) else {
        return HttpsResult::Error(b"OPENAI DIRECT RESPONSE PARSE FAILED");
    };

    let mut line = FixedLine::empty();
    line.set_from_bytes(answer.as_bytes());
    HttpsResult::Answer(line)
}

fn build_http_header(key_len: usize, _body_len: usize) -> String {
    format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: raiOS/0.1\r\nAuthorization: Bearer ",
        API_PATH, API_HOST
    )
    .chars()
    .take(512usize.saturating_sub(key_len))
    .collect()
}

fn build_request_body(prompt: &str) -> String {
    let mut body = String::new();
    body.push_str("{\"model\":\"");
    body.push_str(MODEL);
    body.push_str("\",\"input\":\"");
    push_json_string(&mut body, prompt);
    body.push_str("\",\"max_output_tokens\":");
    body.push_str(format!("{}", MAX_OUTPUT_TOKENS).as_str());
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

fn http_response_complete(response: &[u8]) -> bool {
    let Some(body_start) = find_subslice(response, b"\r\n\r\n").map(|index| index + 4) else {
        return false;
    };

    if header_contains(response, b"transfer-encoding:", b"chunked") {
        return find_subslice(&response[body_start..], b"\r\n0\r\n").is_some();
    }

    if let Some(content_len) = parse_content_length(response) {
        return response.len().saturating_sub(body_start) >= content_len;
    }

    false
}

fn parse_status(response: &[u8]) -> Option<u16> {
    let line_end = find_subslice(response, b"\r\n")?;
    let line = str::from_utf8(&response[..line_end]).ok()?;
    let mut parts = line.split_ascii_whitespace();
    let _http = parts.next()?;
    parts.next()?.parse().ok()
}

fn parse_content_length(response: &[u8]) -> Option<usize> {
    let header_end = find_subslice(response, b"\r\n\r\n")?;
    for line in response[..header_end].split(|byte| *byte == b'\n') {
        let line = trim_ascii(line.strip_suffix(b"\r").unwrap_or(line));
        let (name, value) = split_header(line)?;
        if eq_ignore_ascii_case(name, b"content-length") {
            return parse_usize(trim_ascii(value));
        }
    }
    None
}

fn header_contains(response: &[u8], name_prefix: &[u8], value: &[u8]) -> bool {
    let Some(header_end) = find_subslice(response, b"\r\n\r\n") else {
        return false;
    };
    for line in response[..header_end].split(|byte| *byte == b'\n') {
        let line = trim_ascii(line.strip_suffix(b"\r").unwrap_or(line));
        if line.len() >= name_prefix.len()
            && eq_ignore_ascii_case(&line[..name_prefix.len()], name_prefix)
            && contains_ignore_ascii_case(line, value)
        {
            return true;
        }
    }
    false
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

fn split_header(line: &[u8]) -> Option<(&[u8], &[u8])> {
    let colon = line.iter().position(|byte| *byte == b':')?;
    Some((&line[..colon], &line[colon + 1..]))
}

fn trim_ascii(mut value: &[u8]) -> &[u8] {
    while value.first().is_some_and(|byte| byte.is_ascii_whitespace()) {
        value = &value[1..];
    }
    while value.last().is_some_and(|byte| byte.is_ascii_whitespace()) {
        value = &value[..value.len() - 1];
    }
    value
}

fn parse_usize(value: &[u8]) -> Option<usize> {
    let mut out = 0usize;
    for &byte in value {
        if !byte.is_ascii_digit() {
            return None;
        }
        out = out.checked_mul(10)?.checked_add((byte - b'0') as usize)?;
    }
    Some(out)
}

fn parse_hex_usize(value: &[u8]) -> Option<usize> {
    let mut out = 0usize;
    for &byte in value {
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            b';' => break,
            _ => return None,
        };
        out = out.checked_mul(16)?.checked_add(digit as usize)?;
    }
    Some(out)
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn contains_ignore_ascii_case(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| eq_ignore_ascii_case(window, needle))
}

fn eq_ignore_ascii_case(lhs: &[u8], rhs: &[u8]) -> bool {
    lhs.len() == rhs.len()
        && lhs
            .iter()
            .zip(rhs)
            .all(|(left, right)| left.eq_ignore_ascii_case(right))
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
