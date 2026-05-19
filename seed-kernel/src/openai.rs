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
use spin::Mutex;

use crate::{
    entropy, net, openai_trust::OpenAiPinnedCertVerifier, provider_config, provider_trust, serial,
    time, tls_io::KernelTcpStream,
};

const API_HOST: &str = "api.openai.com";
const API_PORT: u16 = 443;
const API_PATH: &str = "/v1/responses";
const MODEL: &str = "gpt-5.4";
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
    state.last_prompt.set_from_str(prompt);
    state
        .last_event
        .set_from_bytes(b"OPENAI DIRECT: RESOLVING api.openai.com");
    state.last_error.clear();
    net::tcp_abort();

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
                    TcpAction::None | TcpAction::StartHttps(_) => None,
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
                TcpAction::StartHttps(prompt) => {
                    if let Some(pending) = state.pending.as_mut() {
                        pending.phase = Phase::Requesting;
                        pending.phase_started_ms = now;
                    }
                    state
                        .last_event
                        .set_from_bytes(b"OPENAI DIRECT: TLS HANDSHAKE STARTED");
                    drop(state);
                    let result = perform_https_request(prompt.as_str());
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
    StartHttps(FixedLine),
}

fn handle_tcp_result(state: &mut OpenAiState, result: net::TcpConnectResult) -> TcpAction {
    match result {
        net::TcpConnectResult::Connected => TcpAction::StartHttps(state.last_prompt),
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

enum HttpsResult {
    Answer(FixedLine),
    Status(u16, FixedLine),
    Error(&'static [u8]),
}

fn perform_https_request(prompt: &str) -> HttpsResult {
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

    let mut key = [0u8; 256];
    let Some(key_len) = provider_config::copy_api_key(&mut key) else {
        return HttpsResult::Error(b"OPENAI DIRECT API KEY MISSING");
    };

    let body = build_request_body(prompt);
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
    body.push_str("\",\"max_output_tokens\":128,\"store\":false}");
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
