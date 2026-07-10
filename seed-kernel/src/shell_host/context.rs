//! Read-only Genesis Context projection over typed current-boot snapshots.

use crate::{
    provider,
    system_problem_facts::{self, ProblemFact},
    system_status::{RowState, SystemSnapshot},
    wifi,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContextTone {
    Neutral,
    Good,
    Attention,
    Critical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ContextState {
    pub(crate) value: &'static str,
    pub(crate) tone: ContextTone,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProblemSummary {
    pub(crate) active: u8,
    pub(crate) critical: u8,
    pub(crate) primary: Option<ProblemFact>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ContextProjection {
    pub(crate) ai_connection: ContextState,
    pub(crate) network: ContextState,
    pub(crate) problems: ProblemSummary,
}

pub(crate) fn project(
    status: &SystemSnapshot,
    provider: &provider::Snapshot,
    wifi: wifi::WifiSnapshot,
) -> ContextProjection {
    let mut problems = ProblemSummary {
        active: 0,
        critical: 0,
        primary: None,
    };
    for fact in system_problem_facts::collect(status, provider, wifi) {
        problems.active = problems.active.saturating_add(1);
        if fact.severity.needs_attention() {
            problems.critical = problems.critical.saturating_add(1);
        }
        if problems.primary.is_none() {
            problems.primary = Some(fact);
        }
    }
    ContextProjection {
        ai_connection: provider_state(provider),
        network: network_state(status.network.state),
        problems,
    }
}

fn provider_state(provider: &provider::Snapshot) -> ContextState {
    if !provider.api_key_set {
        return ContextState {
            value: "Needs key",
            tone: ContextTone::Attention,
        };
    }
    match provider.trust_state {
        "pinned_cert_verified" | "pinned_spki_verified" | "webpki_verified" => ContextState {
            value: "Ready",
            tone: ContextTone::Good,
        },
        "tls_certificate_verification_bypassed" | "pin_mismatch" => ContextState {
            value: "Trust blocked",
            tone: ContextTone::Critical,
        },
        _ => ContextState {
            value: "Needs trust",
            tone: ContextTone::Attention,
        },
    }
}

fn network_state(state: RowState) -> ContextState {
    match state {
        RowState::Ready | RowState::Configured => ContextState {
            value: "Connected",
            tone: ContextTone::Good,
        },
        RowState::Detected => ContextState {
            value: "Detected",
            tone: ContextTone::Neutral,
        },
        RowState::Waiting => ContextState {
            value: "Connecting",
            tone: ContextTone::Attention,
        },
        RowState::Degraded => ContextState {
            value: "Needs attention",
            tone: ContextTone::Attention,
        },
        RowState::Missing => ContextState {
            value: "Unavailable",
            tone: ContextTone::Critical,
        },
    }
}
