use alloc::string::String;
use core::fmt::{self, Write};
use core::str;

use crate::{
    agent_build_loop::FeedbackPacket, net, openai, provider_config, provider_trust, secret_vault,
    ui,
};

const LINE_CAPACITY: usize = 104;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Route {
    OpenAiDirect,
}

impl Route {
    pub fn as_str(self) -> &'static str {
        match self {
            Route::OpenAiDirect => "OPENAI DIRECT",
        }
    }
}

#[derive(Clone, Copy)]
pub struct AgentRequest<'a> {
    pub prompt: &'a str,
    pub model: Option<&'a str>,
    pub max_output: Option<u16>,
    pub target: RequestTarget,
}

impl<'a> AgentRequest<'a> {
    pub fn text(prompt: &'a str) -> Self {
        Self {
            prompt,
            model: None,
            max_output: None,
            target: RequestTarget::Conversation,
        }
    }

    pub fn program(prompt: &'a str) -> Self {
        Self {
            prompt,
            model: None,
            max_output: Some(openai::MAX_OUTPUT_TOKENS),
            target: RequestTarget::ProgramWorkspace,
        }
    }

    pub(crate) fn project(prompt: &'a str) -> Self {
        Self {
            prompt,
            model: None,
            max_output: Some(openai::MAX_OUTPUT_TOKENS),
            target: RequestTarget::ProjectWorkspace,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RequestTarget {
    Conversation,
    ProgramWorkspace,
    ProjectWorkspace,
    ProjectFeedbackWorkspace,
}

#[derive(Clone, Copy)]
pub(crate) struct AnswerProvenance {
    pub(crate) request_body_sha256: [u8; 32],
    pub(crate) request_envelope_sha256: [u8; 32],
    pub(crate) observed_verifier_id: &'static str,
    pub(crate) observed_verifier_state: provider_trust::TrustState,
    pub(crate) observed_verifier_outcome: &'static str,
    pub(crate) chain_policy: &'static str,
    pub(crate) time_policy: &'static str,
    pub(crate) development_tls_bypass: bool,
}

impl AnswerProvenance {
    pub(crate) fn is_positive_pinned_non_bypass(self) -> bool {
        let expected = provider_trust::OPENAI_PINNED_TLS_VERIFIER_METADATA;
        !self.development_tls_bypass
            && matches!(
                self.observed_verifier_state,
                provider_trust::TrustState::PinnedCertVerified
                    | provider_trust::TrustState::PinnedSpkiVerified
            )
            && self.observed_verifier_id == expected.id
            && self.observed_verifier_outcome == "verified"
            && self.chain_policy == expected.chain_policy
            && self.time_policy == expected.time_policy
    }
}

#[derive(Clone, Copy)]
pub struct Submitted {
    pub route: Route,
    pub id: u32,
}

#[derive(Clone, Copy)]
pub enum SubmitError {
    Empty,
    UnsupportedModel,
    InvalidMaxOutput,
    MissingApiKey,
    TrustDenied { state: &'static str },
    Busy { route: Route, id: u32 },
}

pub(crate) struct Event {
    pub(crate) route: Route,
    pub(crate) id: u32,
    pub(crate) target: RequestTarget,
    pub(crate) kind: EventKind,
}

pub(crate) enum EventKind {
    Answer(String, AnswerProvenance),
    Error(String),
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

    fn set_from_str(&mut self, value: &str) {
        self.len = 0;
        push_str_truncated(&mut self.bytes, &mut self.len, value);
    }
}

impl Write for FixedLine {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        push_str_truncated(&mut self.bytes, &mut self.len, s);
        Ok(())
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

#[derive(Clone, Copy)]
pub struct Snapshot {
    pub provider_name: &'static str,
    pub api_key_set: bool,
    pub route: Route,
    pub trust_state: &'static str,
    pub trust_pin_kind: Option<&'static str>,
    pub trust_pin_id: Option<&'static str>,
    pub trust_pin_slot: Option<&'static str>,
    pub trust_pin_rotation_policy: &'static str,
    pub trust_pin_rotation_id: Option<&'static str>,
    pub trust_development_bypass: bool,
    pub trust_verifier_decision: provider_trust::ProviderTrustVerifierDecision,
    pub direct_phase: &'static str,
    pub direct_pending_id: Option<u32>,
    pub direct_last_request_id: Option<u32>,
    pub direct_last_prompt: FixedLine,
    pub direct_last_event: FixedLine,
    pub direct_last_error: FixedLine,
    pub direct_endpoint: &'static str,
    pub direct_model: &'static str,
    pub tcp: Option<net::TcpSnapshot>,
}

pub fn submit_text(prompt: &str, runtime: ui::RuntimeStatus) -> Result<Submitted, SubmitError> {
    submit(AgentRequest::text(prompt), runtime)
}

pub fn submit(
    request: AgentRequest<'_>,
    runtime: ui::RuntimeStatus,
) -> Result<Submitted, SubmitError> {
    let prompt = request.prompt.trim();
    if prompt.is_empty() {
        return Err(SubmitError::Empty);
    }
    if request
        .model
        .is_some_and(|model| !model.eq_ignore_ascii_case(openai::model()))
    {
        return Err(SubmitError::UnsupportedModel);
    }
    let max_output = request
        .max_output
        .unwrap_or(openai::DEFAULT_MAX_OUTPUT_TOKENS);
    if !(1..=openai::MAX_OUTPUT_TOKENS).contains(&max_output) {
        return Err(SubmitError::InvalidMaxOutput);
    }

    if !provider_credential_usable() {
        return Err(SubmitError::MissingApiKey);
    }

    let trust = provider_trust::snapshot();
    if !trust.allows_provider_request() && !provider_trust::can_attempt_openai_tls() {
        return Err(SubmitError::TrustDenied {
            state: trust.state.as_protocol(),
        });
    }

    match openai::submit_request(prompt, max_output, request.target, runtime) {
        Ok(id) => Ok(Submitted {
            route: Route::OpenAiDirect,
            id,
        }),
        Err(openai::SubmitError::Empty) => Err(SubmitError::Empty),
        Err(openai::SubmitError::InvalidMaxOutput) => Err(SubmitError::InvalidMaxOutput),
        Err(openai::SubmitError::Busy(id)) => Err(SubmitError::Busy {
            route: Route::OpenAiDirect,
            id,
        }),
    }
}

pub(crate) fn submit_scoped_project_feedback(
    packet: FeedbackPacket,
    runtime: ui::RuntimeStatus,
) -> Result<Submitted, SubmitError> {
    if !provider_credential_usable() {
        return Err(SubmitError::MissingApiKey);
    }

    let trust = provider_trust::snapshot();
    if !trust.allows_provider_request() && !provider_trust::can_attempt_openai_tls() {
        return Err(SubmitError::TrustDenied {
            state: trust.state.as_protocol(),
        });
    }

    match openai::submit_scoped_project_feedback(packet, runtime) {
        Ok(id) => Ok(Submitted {
            route: Route::OpenAiDirect,
            id,
        }),
        Err(openai::SubmitError::Empty) => Err(SubmitError::Empty),
        Err(openai::SubmitError::InvalidMaxOutput) => Err(SubmitError::InvalidMaxOutput),
        Err(openai::SubmitError::Busy(id)) => Err(SubmitError::Busy {
            route: Route::OpenAiDirect,
            id,
        }),
    }
}

pub(crate) fn poll() -> Option<Event> {
    openai::poll().map(|event| {
        let kind = match event.kind {
            openai::EventKind::Answer(answer, provenance) => {
                EventKind::Answer(answer, provenance)
            }
            openai::EventKind::Error(error) => EventKind::Error(String::from(error.as_str())),
        };
        Event {
            route: Route::OpenAiDirect,
            id: event.id,
            target: event.target,
            kind,
        }
    })
}

pub fn snapshot() -> Snapshot {
    let config = provider_config::snapshot();
    let direct = openai::snapshot();
    let trust = provider_trust::snapshot();
    let mut direct_last_prompt = FixedLine::empty();
    let mut direct_last_event = FixedLine::empty();
    let mut direct_last_error = FixedLine::empty();

    direct_last_prompt.set_from_str(direct.last_prompt.as_str());
    direct_last_event.set_from_str(direct.last_event.as_str());
    direct_last_error.set_from_str(direct.last_error.as_str());

    Snapshot {
        provider_name: config.provider_name,
        api_key_set: provider_credential_configured(),
        route: Route::OpenAiDirect,
        trust_state: trust.state.as_protocol(),
        trust_pin_kind: trust.pin_kind,
        trust_pin_id: trust.pin_id,
        trust_pin_slot: trust.pin_slot,
        trust_pin_rotation_policy: trust.pin_rotation_policy,
        trust_pin_rotation_id: trust.pin_rotation_id,
        trust_development_bypass: trust.development_bypass,
        trust_verifier_decision: trust.verifier_decision,
        direct_phase: direct.phase,
        direct_pending_id: direct.pending_id,
        direct_last_request_id: direct.last_request_id,
        direct_last_prompt,
        direct_last_event,
        direct_last_error,
        direct_endpoint: direct.endpoint,
        direct_model: direct.model,
        tcp: net::tcp_snapshot(),
    }
}

fn provider_credential_configured() -> bool {
    match secret_vault::provider_status() {
        secret_vault::VaultSecretStatus::Available { .. } => true,
        secret_vault::VaultSecretStatus::Missing => provider_config::api_key_set(),
        secret_vault::VaultSecretStatus::Forgotten { .. } => false,
    }
}

fn provider_credential_usable() -> bool {
    match secret_vault::provider_status() {
        secret_vault::VaultSecretStatus::Available { .. } => {
            secret_vault::recovery_state() == secret_vault::VaultRecoveryState::Unlocked
        }
        secret_vault::VaultSecretStatus::Missing => provider_config::api_key_set(),
        secret_vault::VaultSecretStatus::Forgotten { .. } => false,
    }
}
