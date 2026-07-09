use sha2::{Digest, Sha256};
use spin::Mutex;

use crate::{entropy, serial};

pub const RAM_CANDIDATE_ID: &str = "owner_key.ram_candidate.current_boot";
pub const RAM_CANDIDATE_HANDLE: &str = "owner_key.handle.current_boot.ram0";
pub const RAM_CANDIDATE_ALGORITHM: &str = "ram_32_byte_entropy_seed_sha256_fingerprint";
pub const RAM_CANDIDATE_FINGERPRINT_DOMAIN: &[u8] = b"raios.owner_key.ram_candidate.v0";
const SECRET_LEN: usize = 32;

static OWNER_KEY_STATE: Mutex<OwnerKeyState> = Mutex::new(OwnerKeyState::new());

#[derive(Clone, Copy)]
pub struct OwnerKeySnapshot {
    pub generated: bool,
    pub handle: Option<&'static str>,
    pub fingerprint: Option<[u8; 32]>,
    pub secret_len: usize,
}

struct OwnerKeyState {
    generated: bool,
    secret: [u8; SECRET_LEN],
    fingerprint: [u8; 32],
}

impl OwnerKeyState {
    const fn new() -> Self {
        Self {
            generated: false,
            secret: [0; SECRET_LEN],
            fingerprint: [0; 32],
        }
    }

    fn snapshot(&self) -> OwnerKeySnapshot {
        OwnerKeySnapshot {
            generated: self.generated,
            handle: if self.generated {
                Some(RAM_CANDIDATE_HANDLE)
            } else {
                None
            },
            fingerprint: if self.generated {
                Some(self.fingerprint)
            } else {
                None
            },
            secret_len: SECRET_LEN,
        }
    }
}

pub fn ensure_current_boot_candidate() -> OwnerKeySnapshot {
    {
        let state = OWNER_KEY_STATE.lock();
        if state.generated {
            return state.snapshot();
        }
    }

    if !entropy::is_ready() {
        return snapshot();
    }

    let mut secret = [0u8; SECRET_LEN];
    entropy::take(&mut secret);
    let fingerprint = fingerprint_secret(&secret);

    let mut state = OWNER_KEY_STATE.lock();
    if !state.generated {
        state.secret.copy_from_slice(&secret);
        state.fingerprint = fingerprint;
        state.generated = true;
        serial::write_line("owner-key: RAM current_boot candidate generated");
    }
    for byte in secret.iter_mut() {
        *byte = 0;
    }
    state.snapshot()
}

pub fn snapshot() -> OwnerKeySnapshot {
    OWNER_KEY_STATE.lock().snapshot()
}

fn fingerprint_secret(secret: &[u8; SECRET_LEN]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(RAM_CANDIDATE_FINGERPRINT_DOMAIN);
    hash.update(secret);
    let digest = hash.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}
