use core::fmt::{self, Write};
use core::str;

use spin::Mutex;

use crate::{net, serial, time};

const API_HOST: &str = "api.openai.com";
const API_PORT: u16 = 443;
const API_PATH: &str = "/v1/responses";
const MODEL: &str = "gpt-5.4";
const LINE_CAPACITY: usize = 104;
const DNS_TIMEOUT_MS: u64 = 6_000;
const TCP_TIMEOUT_MS: u64 = 8_000;

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
        for &byte in bytes.iter().take(LINE_CAPACITY) {
            self.push_sanitized(byte);
        }
    }

    fn push_sanitized(&mut self, byte: u8) {
        if self.len == self.bytes.len() {
            return;
        }
        self.bytes[self.len] = if byte.is_ascii_graphic() || byte == b' ' {
            byte
        } else {
            b'?'
        };
        self.len += 1;
    }
}

impl Write for FixedLine {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let remaining = self.bytes.len().saturating_sub(self.len);
        let bytes = s.as_bytes();
        let take = usize::min(remaining, bytes.len());
        self.bytes[self.len..self.len + take].copy_from_slice(&bytes[..take]);
        self.len += take;
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
}

struct PendingRequest {
    id: u32,
    phase: Phase,
    address: Option<smoltcp::wire::Ipv4Address>,
    phase_started_ms: u64,
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

pub fn submit_request(prompt: &str) -> Result<u32, SubmitError> {
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
    state.pending = Some(PendingRequest {
        id,
        phase: Phase::Resolving,
        address: None,
        phase_started_ms: now_ms(),
    });
    state.last_request_id = Some(id);
    state.last_prompt.set_from_bytes(prompt.as_bytes());
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
                handle_tcp_result(&mut state, net::tcp_connect_ipv4(address, API_PORT))
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
            handle_tcp_result(&mut state, net::tcp_connect_ipv4(address, API_PORT))
        }
    }
}

pub fn snapshot() -> Snapshot {
    STATE.lock().snapshot()
}

fn handle_tcp_result(state: &mut OpenAiState, result: net::TcpConnectResult) -> Option<FixedLine> {
    match result {
        net::TcpConnectResult::Connected => {
            complete_event(state, b"OPENAI DIRECT TCP 443 READY; TLS CLIENT MISSING")
        }
        net::TcpConnectResult::Started => {
            state
                .last_event
                .set_from_bytes(b"OPENAI DIRECT: TCP CONNECT STARTED");
            None
        }
        net::TcpConnectResult::Connecting(tcp_state) => {
            let mut line = FixedLine::empty();
            let _ = write!(line, "OPENAI DIRECT: TCP {}", tcp_state);
            state.last_event = line;
            None
        }
        net::TcpConnectResult::NetworkUnavailable => {
            complete_error(state, b"OPENAI DIRECT NETWORK UNAVAILABLE")
        }
        net::TcpConnectResult::NetworkUnconfigured => {
            complete_error(state, b"OPENAI DIRECT NETWORK UNCONFIGURED")
        }
        net::TcpConnectResult::ConnectError => complete_error(state, b"OPENAI DIRECT TCP ERROR"),
    }
}

fn complete_event(state: &mut OpenAiState, message: &[u8]) -> Option<FixedLine> {
    let mut line = FixedLine::empty();
    line.set_from_bytes(message);
    state.last_event = line;
    state.pending = None;
    Some(line)
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
    }
}

fn now_ms() -> u64 {
    let per_ms = time::tsc_per_ms().max(1);
    time::rdtsc() / per_ms
}
