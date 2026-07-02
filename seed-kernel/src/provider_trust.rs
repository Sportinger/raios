use spin::Mutex;

const OPENAI_CERT_SHA256: Option<&str> = option_env!("RAIOS_OPENAI_CERT_SHA256");
const OPENAI_SPKI_SHA256: Option<&str> = option_env!("RAIOS_OPENAI_SPKI_SHA256");
const ALLOW_UNVERIFIED_OPENAI_TLS: Option<&str> = option_env!("RAIOS_ALLOW_UNVERIFIED_OPENAI_TLS");

static STATE: Mutex<RuntimeTrust> = Mutex::new(RuntimeTrust::new());

struct RuntimeTrust {
    state: TrustState,
    decision: ProviderTrustVerifierDecision,
}

impl RuntimeTrust {
    const fn new() -> Self {
        Self {
            state: TrustState::Unknown,
            decision: ProviderTrustVerifierDecision::not_attempted(),
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
pub struct ProviderTrustVerifierMetadata {
    pub schema: &'static str,
    pub id: &'static str,
    pub host: &'static str,
    pub port: &'static str,
    pub transport: &'static str,
    pub hostname_policy: &'static str,
    pub pin_policy: &'static str,
    pub chain_policy: &'static str,
    pub time_policy: &'static str,
    pub certificate_verify_policy: &'static str,
}

pub const OPENAI_PINNED_TLS_VERIFIER_METADATA: ProviderTrustVerifierMetadata =
    ProviderTrustVerifierMetadata {
        schema: "raios.provider_trust_verifier_metadata.v0",
        id: "openai.pinned_tls13_p256_sha256.v0",
        host: "api.openai.com",
        port: "443",
        transport: "tls1.3",
        hostname_policy: "exact_api.openai.com_required",
        pin_policy: "configured_leaf_or_spki_sha256_required",
        chain_policy: "pin_only_no_webpki_chain_validation",
        time_policy: "not_validated_stage0",
        certificate_verify_policy: "tls13_ecdsa_secp256r1_sha256_required",
    };

#[derive(Clone, Copy)]
pub struct ProviderTrustVerifierDecision {
    pub schema: &'static str,
    pub verifier_id: &'static str,
    pub stage: &'static str,
    pub outcome: &'static str,
    pub reason: &'static str,
}

impl ProviderTrustVerifierDecision {
    pub const fn not_attempted() -> Self {
        Self {
            schema: "raios.provider_trust_verifier_decision.v0",
            verifier_id: OPENAI_PINNED_TLS_VERIFIER_METADATA.id,
            stage: "not_attempted",
            outcome: "not_attempted",
            reason: "tls_verifier_not_entered",
        }
    }

    pub const fn rejected(stage: &'static str, reason: &'static str) -> Self {
        Self {
            schema: "raios.provider_trust_verifier_decision.v0",
            verifier_id: OPENAI_PINNED_TLS_VERIFIER_METADATA.id,
            stage,
            outcome: "rejected",
            reason,
        }
    }

    pub const fn verified(stage: &'static str, reason: &'static str) -> Self {
        Self {
            schema: "raios.provider_trust_verifier_decision.v0",
            verifier_id: OPENAI_PINNED_TLS_VERIFIER_METADATA.id,
            stage,
            outcome: "verified",
            reason,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Snapshot {
    pub state: TrustState,
    pub pin_kind: Option<&'static str>,
    pub pin_id: Option<&'static str>,
    pub verifier: ProviderTrustVerifierMetadata,
    pub verifier_decision: ProviderTrustVerifierDecision,
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PinKind {
    LeafCertSha256,
    SpkiSha256,
}

impl PinKind {
    pub fn as_protocol(self) -> &'static str {
        match self {
            PinKind::LeafCertSha256 => "leaf_cert_sha256",
            PinKind::SpkiSha256 => "spki_sha256",
        }
    }
}

#[derive(Clone, Copy)]
pub struct OpenAiPin {
    pub kind: PinKind,
    pub bytes: [u8; 32],
}

pub fn snapshot() -> Snapshot {
    let pin = configured_openai_pin();
    if unverified_development_allowed() {
        return Snapshot {
            state: TrustState::TlsCertificateVerificationBypassed,
            pin_kind: pin_kind(pin),
            pin_id: pin_id(pin),
            verifier: OPENAI_PINNED_TLS_VERIFIER_METADATA,
            verifier_decision: ProviderTrustVerifierDecision::rejected(
                "development_bypass",
                "unverified_tls_development_bypass",
            ),
            development_bypass: true,
        };
    }

    let (state, verifier_decision) = match pin {
        None => (
            TrustState::PinConfigMissing,
            ProviderTrustVerifierDecision::rejected("pin_config", "pin_config_missing"),
        ),
        Some(value) if !is_sha256_hex(value.value) => (
            TrustState::PinConfigInvalid,
            ProviderTrustVerifierDecision::rejected("pin_config", "pin_config_invalid"),
        ),
        Some(_) => {
            let runtime = STATE.lock();
            (runtime.state, runtime.decision)
        }
    };

    Snapshot {
        state,
        pin_kind: pin_kind(pin),
        pin_id: pin_id(pin),
        verifier: OPENAI_PINNED_TLS_VERIFIER_METADATA,
        verifier_decision,
        development_bypass: false,
    }
}

pub fn can_attempt_openai_tls() -> bool {
    unverified_development_allowed() || openai_pin().is_ok()
}

pub fn openai_pin() -> Result<OpenAiPin, TrustState> {
    let Some(pin) = configured_openai_pin() else {
        return Err(TrustState::PinConfigMissing);
    };
    let bytes = parse_sha256_hex(pin.value).ok_or(TrustState::PinConfigInvalid)?;
    Ok(OpenAiPin {
        kind: pin.kind,
        bytes,
    })
}

pub fn mark_pin_mismatch_at(stage: &'static str, reason: &'static str) {
    let mut state = STATE.lock();
    state.state = TrustState::PinMismatch;
    state.decision = ProviderTrustVerifierDecision::rejected(stage, reason);
}

pub fn mark_pin_verifier_unavailable_at(stage: &'static str, reason: &'static str) {
    let mut state = STATE.lock();
    state.state = TrustState::PinVerifierUnavailable;
    state.decision = ProviderTrustVerifierDecision::rejected(stage, reason);
}

pub fn mark_pinned_cert_verified_at(stage: &'static str, reason: &'static str) {
    let mut state = STATE.lock();
    state.state = TrustState::PinnedCertVerified;
    state.decision = ProviderTrustVerifierDecision::verified(stage, reason);
}

pub fn mark_pinned_spki_verified_at(stage: &'static str, reason: &'static str) {
    let mut state = STATE.lock();
    state.state = TrustState::PinnedSpkiVerified;
    state.decision = ProviderTrustVerifierDecision::verified(stage, reason);
}

#[derive(Clone, Copy)]
struct ConfiguredPin {
    kind: PinKind,
    value: &'static str,
}

fn configured_openai_pin() -> Option<ConfiguredPin> {
    if let Some(value) = configured_openai_spki_pin() {
        return Some(ConfiguredPin {
            kind: PinKind::SpkiSha256,
            value,
        });
    }
    configured_openai_cert_pin().map(|value| ConfiguredPin {
        kind: PinKind::LeafCertSha256,
        value,
    })
}

fn configured_openai_spki_pin() -> Option<&'static str> {
    let value = OPENAI_SPKI_SHA256?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value)
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

fn pin_kind(pin: Option<ConfiguredPin>) -> Option<&'static str> {
    pin.map(|pin| pin.kind.as_protocol())
}

fn pin_id(pin: Option<ConfiguredPin>) -> Option<&'static str> {
    let pin = pin?;
    if is_sha256_hex(pin.value) {
        Some(&pin.value[..12])
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
