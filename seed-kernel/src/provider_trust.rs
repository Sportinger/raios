use spin::Mutex;

const OPENAI_CERT_SHA256: Option<&str> = option_env!("RAIOS_OPENAI_CERT_SHA256");
const ALLOW_UNVERIFIED_OPENAI_TLS: Option<&str> = option_env!("RAIOS_ALLOW_UNVERIFIED_OPENAI_TLS");

static STATE: Mutex<RuntimeTrust> = Mutex::new(RuntimeTrust::new());

struct RuntimeTrust {
    state: TrustState,
}

impl RuntimeTrust {
    const fn new() -> Self {
        Self {
            state: TrustState::Unknown,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TrustState {
    Unknown,
    TlsCertificateVerificationBypassed,
    PinConfigMissing,
    PinConfigInvalid,
    PinVerifierUnavailable,
    PinMismatch,
    PinnedCertVerified,
    PinnedSpkiVerified,
    WebPkiVerified,
}

impl TrustState {
    pub fn as_protocol(self) -> &'static str {
        match self {
            TrustState::Unknown => "unknown",
            TrustState::TlsCertificateVerificationBypassed => {
                "tls_certificate_verification_bypassed"
            }
            TrustState::PinConfigMissing => "pin_config_missing",
            TrustState::PinConfigInvalid => "pin_config_invalid",
            TrustState::PinVerifierUnavailable => "pin_verifier_unavailable",
            TrustState::PinMismatch => "pin_mismatch",
            TrustState::PinnedCertVerified => "pinned_cert_verified",
            TrustState::PinnedSpkiVerified => "pinned_spki_verified",
            TrustState::WebPkiVerified => "webpki_verified",
        }
    }

    pub fn openai_error(self) -> &'static [u8] {
        match self {
            TrustState::Unknown => b"OPENAI DIRECT TLS TRUST UNKNOWN",
            TrustState::TlsCertificateVerificationBypassed => {
                b"OPENAI DIRECT TLS UNVERIFIED DEVELOPMENT OVERRIDE"
            }
            TrustState::PinConfigMissing => b"OPENAI DIRECT TLS PIN CONFIG MISSING",
            TrustState::PinConfigInvalid => b"OPENAI DIRECT TLS PIN CONFIG INVALID",
            TrustState::PinVerifierUnavailable => b"OPENAI DIRECT TLS PIN VERIFIER UNAVAILABLE",
            TrustState::PinMismatch => b"OPENAI DIRECT TLS PIN MISMATCH",
            TrustState::PinnedCertVerified
            | TrustState::PinnedSpkiVerified
            | TrustState::WebPkiVerified => b"OPENAI DIRECT TLS TRUST VERIFIED",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Snapshot {
    pub state: TrustState,
    pub pin_kind: Option<&'static str>,
    pub pin_id: Option<&'static str>,
    pub development_bypass: bool,
}

impl Snapshot {
    pub fn allows_provider_request(self) -> bool {
        self.development_bypass
            || matches!(
                self.state,
                TrustState::PinnedCertVerified
                    | TrustState::PinnedSpkiVerified
                    | TrustState::WebPkiVerified
            )
    }
}

pub fn snapshot() -> Snapshot {
    let pin = configured_openai_cert_pin();
    if unverified_development_allowed() {
        return Snapshot {
            state: TrustState::TlsCertificateVerificationBypassed,
            pin_kind: pin_kind(pin),
            pin_id: pin_id(pin),
            development_bypass: true,
        };
    }

    let state = match pin {
        None => TrustState::PinConfigMissing,
        Some(value) if !is_sha256_hex(value) => TrustState::PinConfigInvalid,
        Some(_) => STATE.lock().state,
    };

    Snapshot {
        state,
        pin_kind: pin_kind(pin),
        pin_id: pin_id(pin),
        development_bypass: false,
    }
}

pub fn can_attempt_openai_tls() -> bool {
    unverified_development_allowed() || openai_cert_pin_bytes().is_ok()
}

pub fn openai_cert_pin_bytes() -> Result<[u8; 32], TrustState> {
    let Some(pin) = configured_openai_cert_pin() else {
        return Err(TrustState::PinConfigMissing);
    };
    parse_sha256_hex(pin).ok_or(TrustState::PinConfigInvalid)
}

pub fn mark_pin_mismatch() {
    STATE.lock().state = TrustState::PinMismatch;
}

pub fn mark_pin_verifier_unavailable() {
    STATE.lock().state = TrustState::PinVerifierUnavailable;
}

pub fn mark_pinned_cert_verified() {
    STATE.lock().state = TrustState::PinnedCertVerified;
}

fn configured_openai_cert_pin() -> Option<&'static str> {
    let value = OPENAI_CERT_SHA256?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn pin_kind(pin: Option<&str>) -> Option<&'static str> {
    pin.map(|_| "leaf_cert_sha256")
}

fn pin_id(pin: Option<&'static str>) -> Option<&'static str> {
    let pin = pin?;
    if is_sha256_hex(pin) {
        Some(&pin[..12])
    } else {
        None
    }
}

fn unverified_development_allowed() -> bool {
    matches!(
        ALLOW_UNVERIFIED_OPENAI_TLS.map(str::trim),
        Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("YES")
    )
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn parse_sha256_hex(value: &str) -> Option<[u8; 32]> {
    if !is_sha256_hex(value) {
        return None;
    }
    let mut out = [0u8; 32];
    let bytes = value.as_bytes();
    let mut index = 0usize;
    while index < out.len() {
        let high = hex_nibble(bytes[index * 2])?;
        let low = hex_nibble(bytes[index * 2 + 1])?;
        out[index] = (high << 4) | low;
        index += 1;
    }
    Some(out)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
