use sha2::{Digest, Sha256};
use spin::Mutex;

const OPENAI_CERT_SHA256: Option<&str> = option_env!("RAIOS_OPENAI_CERT_SHA256");
const OPENAI_SPKI_SHA256: Option<&str> = option_env!("RAIOS_OPENAI_SPKI_SHA256");
const OPENAI_SPKI_SHA256_NEXT: Option<&str> = option_env!("RAIOS_OPENAI_SPKI_SHA256_NEXT");
const ALLOW_UNVERIFIED_OPENAI_TLS: Option<&str> = option_env!("RAIOS_ALLOW_UNVERIFIED_OPENAI_TLS");

static STATE: Mutex<RuntimeTrust> = Mutex::new(RuntimeTrust::new());

struct RuntimeTrust {
    state: TrustState,
    decision: ProviderTrustVerifierDecision,
    matched_pin: Option<ProviderTrustMatchedPin>,
}

impl RuntimeTrust {
    const fn new() -> Self {
        Self {
            state: TrustState::Unknown,
            decision: ProviderTrustVerifierDecision::not_attempted(),
            matched_pin: None,
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
        pin_policy: "configured_leaf_or_spki_sha256_required_optional_spki_rotation",
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
    pub pin_slot: Option<&'static str>,
    pub pin_rotation_policy: &'static str,
    pub pin_rotation_id: Option<&'static str>,
    pub verifier: ProviderTrustVerifierMetadata,
    pub verifier_decision: ProviderTrustVerifierDecision,
    pub development_bypass: bool,
}

/// Opaque proof that the current OpenAI endpoint passed a real configured
/// verifier. The development bypass can never create this token.
pub(crate) struct VerifiedOpenAiProviderTrust {
    audit_source: &'static str,
    audit_evidence_sha256: [u8; 32],
}

impl VerifiedOpenAiProviderTrust {
    pub(crate) const fn audit_source(&self) -> &'static str {
        self.audit_source
    }

    pub(crate) const fn audit_evidence_sha256(&self) -> [u8; 32] {
        self.audit_evidence_sha256
    }
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
    pub id: &'static str,
    pub slot: &'static str,
}

#[derive(Clone, Copy)]
pub struct OpenAiPins {
    pub active: OpenAiPin,
    pub rotation: Option<OpenAiPin>,
}

impl OpenAiPins {
    pub fn match_pin(self, kind: PinKind, bytes: &[u8]) -> Option<ProviderTrustMatchedPin> {
        if self.active.kind == kind && bytes == self.active.bytes.as_slice() {
            return Some(ProviderTrustMatchedPin {
                kind: self.active.kind,
                id: self.active.id,
                slot: self.active.slot,
            });
        }
        let rotation = self.rotation?;
        if rotation.kind == kind && bytes == rotation.bytes.as_slice() {
            return Some(ProviderTrustMatchedPin {
                kind: rotation.kind,
                id: rotation.id,
                slot: rotation.slot,
            });
        }
        None
    }
}

#[derive(Clone, Copy)]
pub struct ProviderTrustMatchedPin {
    pub kind: PinKind,
    pub id: &'static str,
    pub slot: &'static str,
}

pub fn snapshot() -> Snapshot {
    let pins = configured_openai_pins();
    if unverified_development_allowed() {
        return Snapshot {
            state: TrustState::TlsCertificateVerificationBypassed,
            pin_kind: active_pin_kind(pins),
            pin_id: active_pin_id(pins),
            pin_slot: active_pin_slot(pins),
            pin_rotation_policy: pin_rotation_policy(pins),
            pin_rotation_id: pin_rotation_id(pins),
            verifier: OPENAI_PINNED_TLS_VERIFIER_METADATA,
            verifier_decision: ProviderTrustVerifierDecision::rejected(
                "development_bypass",
                "unverified_tls_development_bypass",
            ),
            development_bypass: true,
        };
    }

    let (state, verifier_decision, matched_pin) = match pins {
        _ if has_invalid_spki_rotation_config() => (
            TrustState::PinConfigInvalid,
            ProviderTrustVerifierDecision::rejected("pin_config", "pin_config_invalid"),
            None,
        ),
        None => (
            TrustState::PinConfigMissing,
            ProviderTrustVerifierDecision::rejected("pin_config", "pin_config_missing"),
            None,
        ),
        Some(value) if !is_valid_pin_config(value) => (
            TrustState::PinConfigInvalid,
            ProviderTrustVerifierDecision::rejected("pin_config", "pin_config_invalid"),
            None,
        ),
        Some(_) => {
            let runtime = STATE.lock();
            (runtime.state, runtime.decision, runtime.matched_pin)
        }
    };
    let display_pin = matched_pin;

    Snapshot {
        state,
        pin_kind: display_pin
            .map(|pin| pin.kind.as_protocol())
            .or_else(|| active_pin_kind(pins)),
        pin_id: display_pin
            .map(|pin| pin.id)
            .or_else(|| active_pin_id(pins)),
        pin_slot: display_pin
            .map(|pin| pin.slot)
            .or_else(|| active_pin_slot(pins)),
        pin_rotation_policy: pin_rotation_policy(pins),
        pin_rotation_id: pin_rotation_id(pins),
        verifier: OPENAI_PINNED_TLS_VERIFIER_METADATA,
        verifier_decision,
        development_bypass: false,
    }
}

pub fn can_attempt_openai_tls() -> bool {
    unverified_development_allowed() || openai_pins().is_ok()
}

pub(crate) fn verified_openai_provider_trust() -> Option<VerifiedOpenAiProviderTrust> {
    let trust = snapshot();
    if trust.development_bypass || trust.verifier_decision.outcome != "verified" {
        return None;
    }
    let audit_source = match trust.state {
        TrustState::PinnedCertVerified => "pinned_cert_verified",
        TrustState::PinnedSpkiVerified => "pinned_spki_verified",
        TrustState::WebPkiVerified => "webpki_verified",
        _ => return None,
    };
    let mut hash = Sha256::new();
    for field in [
        "raios.provider-use-trust.v1",
        trust.verifier.id,
        trust.verifier.host,
        trust.verifier.pin_policy,
        trust.verifier.chain_policy,
        trust.verifier.time_policy,
        trust.verifier_decision.stage,
        trust.verifier_decision.outcome,
        trust.verifier_decision.reason,
        trust.state.as_protocol(),
        trust.pin_kind.unwrap_or("none"),
        trust.pin_id.unwrap_or("none"),
        trust.pin_slot.unwrap_or("none"),
    ] {
        hash.update((field.len() as u64).to_le_bytes());
        hash.update(field.as_bytes());
    }
    let digest = hash.finalize();
    let mut audit_evidence_sha256 = [0u8; 32];
    audit_evidence_sha256.copy_from_slice(&digest);
    Some(VerifiedOpenAiProviderTrust {
        audit_source,
        audit_evidence_sha256,
    })
}

pub fn openai_pins() -> Result<OpenAiPins, TrustState> {
    if has_invalid_spki_rotation_config() {
        return Err(TrustState::PinConfigInvalid);
    }
    let Some(pins) = configured_openai_pins() else {
        return Err(TrustState::PinConfigMissing);
    };
    if !is_valid_pin_config(pins) {
        return Err(TrustState::PinConfigInvalid);
    }
    let active = parse_openai_pin(pins.active).ok_or(TrustState::PinConfigInvalid)?;
    let rotation = match pins.rotation {
        Some(pin) => Some(parse_openai_pin(pin).ok_or(TrustState::PinConfigInvalid)?),
        None => None,
    };
    Ok(OpenAiPins { active, rotation })
}

pub fn mark_pin_mismatch_at(stage: &'static str, reason: &'static str) {
    let mut state = STATE.lock();
    state.state = TrustState::PinMismatch;
    state.decision = ProviderTrustVerifierDecision::rejected(stage, reason);
    state.matched_pin = None;
}

pub fn mark_pin_verifier_unavailable_at(stage: &'static str, reason: &'static str) {
    let mut state = STATE.lock();
    state.state = TrustState::PinVerifierUnavailable;
    state.decision = ProviderTrustVerifierDecision::rejected(stage, reason);
    state.matched_pin = None;
}

pub fn mark_pinned_cert_verified_at(
    stage: &'static str,
    reason: &'static str,
    matched_pin: ProviderTrustMatchedPin,
) {
    let mut state = STATE.lock();
    state.state = TrustState::PinnedCertVerified;
    state.decision = ProviderTrustVerifierDecision::verified(stage, reason);
    state.matched_pin = Some(matched_pin);
}

pub fn mark_pinned_spki_verified_at(
    stage: &'static str,
    reason: &'static str,
    matched_pin: ProviderTrustMatchedPin,
) {
    let mut state = STATE.lock();
    state.state = TrustState::PinnedSpkiVerified;
    state.decision = ProviderTrustVerifierDecision::verified(stage, reason);
    state.matched_pin = Some(matched_pin);
}

#[derive(Clone, Copy)]
struct ConfiguredPin {
    kind: PinKind,
    value: &'static str,
    slot: &'static str,
}

#[derive(Clone, Copy)]
struct ConfiguredOpenAiPins {
    active: ConfiguredPin,
    rotation: Option<ConfiguredPin>,
}

fn configured_openai_pins() -> Option<ConfiguredOpenAiPins> {
    let rotation = configured_openai_spki_rotation_pin().map(|value| ConfiguredPin {
        kind: PinKind::SpkiSha256,
        value,
        slot: "rotation",
    });
    if let Some(value) = configured_openai_spki_pin() {
        return Some(ConfiguredOpenAiPins {
            active: ConfiguredPin {
                kind: PinKind::SpkiSha256,
                value,
                slot: "active",
            },
            rotation,
        });
    }
    configured_openai_cert_pin().map(|value| ConfiguredOpenAiPins {
        active: ConfiguredPin {
            kind: PinKind::LeafCertSha256,
            value,
            slot: "active",
        },
        rotation,
    })
}

fn parse_openai_pin(pin: ConfiguredPin) -> Option<OpenAiPin> {
    let bytes = parse_sha256_hex(pin.value)?;
    Some(OpenAiPin {
        kind: pin.kind,
        bytes,
        id: pin_id(pin)?,
        slot: pin.slot,
    })
}

fn is_valid_pin_config(pins: ConfiguredOpenAiPins) -> bool {
    if !is_sha256_hex(pins.active.value) {
        return false;
    }
    if let Some(rotation) = pins.rotation {
        return pins.active.kind == PinKind::SpkiSha256
            && rotation.kind == PinKind::SpkiSha256
            && is_sha256_hex(rotation.value)
            && rotation.value != pins.active.value;
    }
    true
}

fn configured_openai_spki_pin() -> Option<&'static str> {
    configured_non_empty(OPENAI_SPKI_SHA256)
}

fn configured_openai_spki_rotation_pin() -> Option<&'static str> {
    configured_non_empty(OPENAI_SPKI_SHA256_NEXT)
}

fn configured_openai_cert_pin() -> Option<&'static str> {
    configured_non_empty(OPENAI_CERT_SHA256)
}

fn configured_non_empty(value: Option<&'static str>) -> Option<&'static str> {
    let value = value?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn active_pin_kind(pins: Option<ConfiguredOpenAiPins>) -> Option<&'static str> {
    pins.map(|pins| pins.active.kind.as_protocol())
}

fn active_pin_id(pins: Option<ConfiguredOpenAiPins>) -> Option<&'static str> {
    pin_id(pins?.active)
}

fn active_pin_slot(pins: Option<ConfiguredOpenAiPins>) -> Option<&'static str> {
    Some(pins?.active.slot)
}

fn pin_rotation_policy(pins: Option<ConfiguredOpenAiPins>) -> &'static str {
    let Some(pins) = pins else {
        if configured_openai_spki_rotation_pin().is_some() {
            return "invalid_spki_rotation_config";
        }
        return "missing_active_pin";
    };
    if pins.rotation.is_some() {
        if is_valid_pin_config(pins) {
            "active_spki_plus_rotation_spki"
        } else {
            "invalid_spki_rotation_config"
        }
    } else {
        "single_active_pin"
    }
}

fn pin_rotation_id(pins: Option<ConfiguredOpenAiPins>) -> Option<&'static str> {
    pin_id(pins?.rotation?)
}

fn has_invalid_spki_rotation_config() -> bool {
    configured_openai_spki_rotation_pin().is_some() && configured_openai_spki_pin().is_none()
}

fn pin_id(pin: ConfiguredPin) -> Option<&'static str> {
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
