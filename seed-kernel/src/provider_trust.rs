const OPENAI_CERT_SHA256: Option<&str> = option_env!("SEEDOS_OPENAI_CERT_SHA256");
const ALLOW_UNVERIFIED_OPENAI_TLS: Option<&str> = option_env!("SEEDOS_ALLOW_UNVERIFIED_OPENAI_TLS");

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
        Some(_) => TrustState::PinVerifierUnavailable,
    };

    Snapshot {
        state,
        pin_kind: pin_kind(pin),
        pin_id: pin_id(pin),
        development_bypass: false,
    }
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
