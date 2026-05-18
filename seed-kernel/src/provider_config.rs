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

pub fn copy_api_key(dest: &mut [u8]) -> Option<usize> {
    let state = STATE.lock();
    if state.api_key_len == 0 || state.api_key[0] == 0 {
        return None;
    }

    let len = usize::min(dest.len(), state.api_key_len);
    dest[..len].copy_from_slice(&state.api_key[..len]);
    Some(len)
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

pub fn init_default_config() -> bool {
    let Some(key) = DEFAULT_OPENAI_API_KEY else {
        return false;
    };

    set_api_key(key.as_bytes()).is_ok()
}
