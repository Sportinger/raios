//! Pure current-boot problem projection shared by protocol and trusted UI.

use crate::{
    provider,
    system_status::{RowState, SystemSnapshot},
    wifi,
};

const MAX_PROBLEM_FACTS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProblemSeverity {
    Info,
    Warning,
    Error,
    High,
}

impl ProblemSeverity {
    pub(crate) const fn as_protocol(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::High => "high",
        }
    }

    pub(crate) const fn needs_attention(self) -> bool {
        matches!(self, Self::Error | Self::High)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProblemFact {
    pub(crate) id: &'static str,
    pub(crate) severity: ProblemSeverity,
    pub(crate) summary: &'static str,
}

pub(crate) struct ProblemFacts {
    entries: [Option<ProblemFact>; MAX_PROBLEM_FACTS],
    len: usize,
    cursor: usize,
}

impl Iterator for ProblemFacts {
    type Item = ProblemFact;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor == self.len {
            return None;
        }
        let fact = self.entries[self.cursor];
        self.cursor += 1;
        fact
    }
}

/// Derives stable public/current-boot problem facts without reading global state.
pub(crate) fn collect(
    status: &SystemSnapshot,
    provider: &provider::Snapshot,
    wifi: wifi::WifiSnapshot,
) -> ProblemFacts {
    let mut facts = ProblemFacts {
        entries: [None; MAX_PROBLEM_FACTS],
        len: 0,
        cursor: 0,
    };

    if let Some(fact) = provider_trust_problem(provider) {
        facts.push(fact);
    }
    if !provider.api_key_set {
        facts.push(ProblemFact {
            id: "provider.openai.api_key_missing",
            severity: ProblemSeverity::Info,
            summary: "OpenAI direct requests need a RAM-only API key",
        });
    }
    facts.push_if_unready(
        &status.framebuffer,
        ProblemFact {
            id: "framebuffer.unavailable",
            severity: ProblemSeverity::Error,
            summary: "Limine framebuffer is unavailable",
        },
    );
    facts.push_if_unready(
        &status.entropy,
        ProblemFact {
            id: "entropy.not_ready",
            severity: ProblemSeverity::Warning,
            summary: "Entropy is not ready yet",
        },
    );
    facts.push_if_unready(
        &status.usb_xhci,
        ProblemFact {
            id: "usb_xhci.unavailable",
            severity: ProblemSeverity::Warning,
            summary: "xHCI USB path is missing or degraded",
        },
    );
    facts.push_if_unready(
        &status.network,
        ProblemFact {
            id: "network.unavailable",
            severity: ProblemSeverity::Warning,
            summary: "e1000/IPv4 network path is not configured",
        },
    );
    facts.push_if_unready(
        &status.input,
        ProblemFact {
            id: "input.unavailable",
            severity: ProblemSeverity::Warning,
            summary: "keyboard or pointer input is missing",
        },
    );
    match wifi.state {
        wifi::WifiState::Detected => facts.push(ProblemFact {
            id: "wifi.avastar.firmware_todo",
            severity: ProblemSeverity::Info,
            summary: "Marvell AVASTAR target is detected, but firmware upload and WPA are not implemented",
        }),
        wifi::WifiState::Missing => facts.push(ProblemFact {
            id: "wifi.avastar.target_absent",
            severity: ProblemSeverity::Info,
            summary: "Surface Pro 4 Marvell AVASTAR Wi-Fi target is not present in this machine profile",
        }),
        wifi::WifiState::NotProbed => {}
    }

    facts
}

impl ProblemFacts {
    fn push(&mut self, fact: ProblemFact) {
        debug_assert!(self.len < self.entries.len());
        self.entries[self.len] = Some(fact);
        self.len += 1;
    }

    fn push_if_unready(&mut self, line: &crate::system_status::StatusLine, fact: ProblemFact) {
        if !matches!(
            line.state,
            RowState::Ready | RowState::Configured | RowState::Detected
        ) {
            self.push(fact);
        }
    }
}

fn provider_trust_problem(provider: &provider::Snapshot) -> Option<ProblemFact> {
    let (id, summary) = match provider.trust_state {
        "unknown" => (
            "provider.tls_unknown",
            "OpenAI provider trust has not been established",
        ),
        "tls_certificate_verification_bypassed" => (
            "provider.tls_unverified",
            "OpenAI direct transport is using an explicit unverified TLS development override",
        ),
        "pin_config_missing" => (
            "provider.tls_pin_config_missing",
            "OpenAI direct transport is fail-closed until a provider pin is configured",
        ),
        "pin_config_invalid" => (
            "provider.tls_pin_config_invalid",
            "Configured OpenAI provider pin is invalid",
        ),
        "pin_verifier_unavailable" => (
            "provider.tls_pin_verifier_unavailable",
            "Configured OpenAI provider pin cannot be checked until TLS verifier input access exists",
        ),
        "pin_mismatch" => (
            "provider.tls_pin_mismatch",
            "OpenAI provider certificate did not match the configured pin",
        ),
        "pinned_cert_verified" | "pinned_spki_verified" | "webpki_verified" => return None,
        _ => (
            "provider.tls_unknown",
            "OpenAI provider trust state is not recognized by this protocol build",
        ),
    };
    Some(ProblemFact {
        id,
        severity: ProblemSeverity::High,
        summary,
    })
}
