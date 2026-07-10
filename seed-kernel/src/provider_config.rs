use core::sync::atomic::{compiler_fence, Ordering};

use raios_core::secret_vault::{SecretKind, SecretPlaintext};
use spin::Mutex;

const API_KEY_CAPACITY: usize = 256;
const DEFAULT_OPENAI_API_KEY: Option<&str> = option_env!("RAIOS_DEFAULT_OPENAI_API_KEY");

static STATE: Mutex<ProviderConfig> = Mutex::new(ProviderConfig::new());

#[derive(Clone, Copy)]
pub enum ApiKeyError {
    Empty,
    TooLong,
    InvalidByte,
}

#[derive(Clone, Copy)]
pub struct Snapshot {
    pub provider_name: &'static str,
    pub api_key_set: bool,
}

struct ProviderConfig {
    api_key: [u8; API_KEY_CAPACITY],
    api_key_len: usize,
}

impl ProviderConfig {
    const fn new() -> Self {
        Self {
            api_key: [0; API_KEY_CAPACITY],
            api_key_len: 0,
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            provider_name: "OPENAI",
            api_key_set: self.api_key_len > 0 && self.api_key[0] != 0,
        }
    }
}

pub fn snapshot() -> Snapshot {
    STATE.lock().snapshot()
}

pub fn api_key_set() -> bool {
    snapshot().api_key_set
}

/// Exact legacy bridge for RAM-only configuration. It returns an owned,
/// zeroizing provider value rather than a caller-selected plaintext buffer.
pub(crate) fn legacy_openai_credential() -> Option<SecretPlaintext> {
    let state = STATE.lock();
    if state.api_key_len == 0 || state.api_key[0] == 0 {
        return None;
    }
    let mut staged = [0u8; API_KEY_CAPACITY];
    let len = state.api_key_len;
    staged[..len].copy_from_slice(&state.api_key[..len]);
    SecretPlaintext::take_from(SecretKind::ProviderApiKey, &mut staged[..len]).ok()
}

pub fn clear_api_key() {
    let mut state = STATE.lock();
    erase(&mut state.api_key);
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
    erase(&mut state.api_key);
    state.api_key[..key.len()].copy_from_slice(key);
    state.api_key_len = key.len();
    Ok(())
}

pub fn init_default_config() -> bool {
    let Some(key) = DEFAULT_OPENAI_API_KEY else {
        return false;
    };

    set_api_key(key.as_bytes()).is_ok()
}

fn erase(bytes: &mut [u8]) {
    for byte in bytes {
        // SAFETY: `byte` is a unique mutable reference into the state buffer.
        unsafe { core::ptr::write_volatile(byte, 0) };
    }
    compiler_fence(Ordering::SeqCst);
}
