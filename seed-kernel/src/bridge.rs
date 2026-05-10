use core::str;

use spin::Mutex;

use crate::serial;

const FRAME_START: u8 = 0x02;
const FRAME_CAPACITY: usize = 224;
const TEXT_CAPACITY: usize = 96;
const API_KEY_CAPACITY: usize = 256;
const REQUEST_PREFIX: &str = "SEEDOS_BRIDGE_REQ";
const RESPONSE_PREFIX: &str = "SEEDOS_BRIDGE_RESP ";

static STATE: Mutex<BridgeState> = Mutex::new(BridgeState::new());

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Echo,
    OpenAi,
}

impl Provider {
    pub fn as_str(self) -> &'static str {
        match self {
            Provider::Echo => "ECHO",
            Provider::OpenAi => "OPENAI",
        }
    }
}

#[derive(Clone, Copy)]
pub enum SerialIngest {
    Console(u8),
    Consumed,
    Response,
    Error,
}

#[derive(Clone, Copy)]
pub enum SubmitError {
    Empty,
    Busy(u32),
}

#[derive(Clone, Copy)]
pub enum ApiKeyError {
    Empty,
    TooLong,
    InvalidByte,
}

#[derive(Clone, Copy)]
pub struct FixedText {
    bytes: [u8; TEXT_CAPACITY],
    len: usize,
}

impl FixedText {
    const fn empty() -> Self {
        Self {
            bytes: [0; TEXT_CAPACITY],
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
        for &byte in bytes.iter().take(TEXT_CAPACITY) {
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

#[derive(Clone, Copy)]
pub struct Snapshot {
    pub provider: Provider,
    pub api_key_set: bool,
    pub pending_id: Option<u32>,
    pub last_request_id: Option<u32>,
    pub last_response_id: Option<u32>,
    pub error_count: u32,
    pub last_prompt: FixedText,
    pub last_response: FixedText,
    pub last_error: FixedText,
}

struct BridgeState {
    provider: Provider,
    api_key: [u8; API_KEY_CAPACITY],
    api_key_len: usize,
    next_id: u32,
    pending_id: Option<u32>,
    last_request_id: Option<u32>,
    last_response_id: Option<u32>,
    error_count: u32,
    last_prompt: FixedText,
    last_response: FixedText,
    last_error: FixedText,
    in_frame: bool,
    frame: [u8; FRAME_CAPACITY],
    frame_len: usize,
}

impl BridgeState {
    const fn new() -> Self {
        Self {
            provider: Provider::Echo,
            api_key: [0; API_KEY_CAPACITY],
            api_key_len: 0,
            next_id: 1,
            pending_id: None,
            last_request_id: None,
            last_response_id: None,
            error_count: 0,
            last_prompt: FixedText::empty(),
            last_response: FixedText::empty(),
            last_error: FixedText::empty(),
            in_frame: false,
            frame: [0; FRAME_CAPACITY],
            frame_len: 0,
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            provider: self.provider,
            api_key_set: self.api_key_len > 0 && self.api_key[0] != 0,
            pending_id: self.pending_id,
            last_request_id: self.last_request_id,
            last_response_id: self.last_response_id,
            error_count: self.error_count,
            last_prompt: self.last_prompt,
            last_response: self.last_response,
            last_error: self.last_error,
        }
    }

    fn record_error(&mut self, message: &[u8]) {
        self.error_count = self.error_count.saturating_add(1);
        self.last_error.set_from_bytes(message);
    }
}

pub fn submit_request(prompt: &str) -> Result<u32, SubmitError> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err(SubmitError::Empty);
    }

    let id = {
        let mut state = STATE.lock();
        if let Some(id) = state.pending_id {
            return Err(SubmitError::Busy(id));
        }

        let id = state.next_id;
        state.next_id = state.next_id.wrapping_add(1).max(1);
        state.pending_id = Some(id);
        state.last_request_id = Some(id);
        state.last_prompt.set_from_bytes(prompt.as_bytes());
        id
    };

    serial::write_fmt(format_args!("{} {} ", REQUEST_PREFIX, id));
    write_hex(prompt.as_bytes());
    serial::write_line("");

    Ok(id)
}

pub fn set_provider(provider: Provider) {
    STATE.lock().provider = provider;
}

pub fn clear_api_key() {
    let mut state = STATE.lock();
    state.api_key.fill(0);
    state.api_key_len = 0;
}

pub fn set_api_key(input: &[u8]) -> Result<(), ApiKeyError> {
    let start = input
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(input.len());
    let end = input
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map(|index| index + 1)
        .unwrap_or(start);
    let key = &input[start..end];

    if key.is_empty() {
        return Err(ApiKeyError::Empty);
    }
    if key.len() > API_KEY_CAPACITY {
        return Err(ApiKeyError::TooLong);
    }
    if key.iter().any(|byte| !byte.is_ascii_graphic()) {
        return Err(ApiKeyError::InvalidByte);
    }

    let mut state = STATE.lock();
    state.api_key.fill(0);
    state.api_key[..key.len()].copy_from_slice(key);
    state.api_key_len = key.len();
    Ok(())
}

pub fn ingest_serial_byte(byte: u8) -> SerialIngest {
    let mut state = STATE.lock();

    if state.in_frame {
        match byte {
            b'\r' | b'\n' => {
                state.in_frame = false;
                let result = process_frame(&mut state);
                state.frame_len = 0;
                result
            }
            FRAME_START => {
                state.frame_len = 0;
                SerialIngest::Consumed
            }
            _ => {
                if state.frame_len == state.frame.len() {
                    state.in_frame = false;
                    state.frame_len = 0;
                    state.record_error(b"BRIDGE FRAME TOO LONG");
                    SerialIngest::Error
                } else {
                    let idx = state.frame_len;
                    state.frame[idx] = byte;
                    state.frame_len += 1;
                    SerialIngest::Consumed
                }
            }
        }
    } else if byte == FRAME_START {
        state.in_frame = true;
        state.frame_len = 0;
        SerialIngest::Consumed
    } else {
        SerialIngest::Console(byte)
    }
}

pub fn snapshot() -> Snapshot {
    STATE.lock().snapshot()
}

fn process_frame(state: &mut BridgeState) -> SerialIngest {
    let Ok(line) = str::from_utf8(&state.frame[..state.frame_len]) else {
        state.record_error(b"BRIDGE FRAME UTF8 ERROR");
        return SerialIngest::Error;
    };

    let Some(rest) = line.strip_prefix(RESPONSE_PREFIX) else {
        state.record_error(b"UNKNOWN BRIDGE FRAME");
        return SerialIngest::Error;
    };

    let Some((id_text, hex_text)) = split_once_space(rest) else {
        state.record_error(b"MALFORMED BRIDGE RESPONSE");
        return SerialIngest::Error;
    };

    let Some(id) = parse_u32(id_text.as_bytes()) else {
        state.record_error(b"BAD BRIDGE RESPONSE ID");
        return SerialIngest::Error;
    };

    let mut response = FixedText::empty();
    if !decode_hex(hex_text.as_bytes(), &mut response) {
        state.record_error(b"BAD BRIDGE RESPONSE HEX");
        return SerialIngest::Error;
    }

    state.last_response_id = Some(id);
    state.last_response = response;
    if state.pending_id == Some(id) {
        state.pending_id = None;
    }

    SerialIngest::Response
}

fn split_once_space(text: &str) -> Option<(&str, &str)> {
    let bytes = text.as_bytes();
    let mut idx = 0usize;
    while idx < bytes.len() {
        if bytes[idx] == b' ' {
            let mut rest = idx + 1;
            while rest < bytes.len() && bytes[rest] == b' ' {
                rest += 1;
            }
            return Some((&text[..idx], &text[rest..]));
        }
        idx += 1;
    }
    None
}

fn parse_u32(bytes: &[u8]) -> Option<u32> {
    let mut value = 0u32;
    if bytes.is_empty() {
        return None;
    }

    for &byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?;
        value = value.checked_add((byte - b'0') as u32)?;
    }
    Some(value)
}

fn decode_hex(bytes: &[u8], out: &mut FixedText) -> bool {
    if bytes.len() % 2 != 0 {
        return false;
    }

    let mut idx = 0usize;
    while idx < bytes.len() {
        let Some(high) = hex_value(bytes[idx]) else {
            return false;
        };
        let Some(low) = hex_value(bytes[idx + 1]) else {
            return false;
        };
        out.push_sanitized((high << 4) | low);
        idx += 2;
    }

    true
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn write_hex(bytes: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for &byte in bytes {
        serial::write_byte(HEX[(byte >> 4) as usize]);
        serial::write_byte(HEX[(byte & 0x0F) as usize]);
    }
}
