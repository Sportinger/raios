//! Scoped honesty check for Stage-0 provider trust labels (M10A-1).
//!
//! This evaluator grants nothing. It only decides whether pin-only provider
//! trust evidence is honest about the chain and certificate-time checks that
//! Stage-0 has not built yet.

pub const SCOPED_PROVIDER_TRUST_HONESTY_DECISION_SCHEMA: &str =
    "raios.scoped_provider_trust_honesty_decision.v0";
pub const SCOPED_PROVIDER_TRUST_HONESTY_DECISION_ID: &str =
    "scoped_provider_trust_honesty.current_boot.v0";
pub const SCOPED_PROVIDER_TRUST_HONESTY_DECISION_MARKER: &str =
    "RAIOS_PROVIDER_TRUST_HONESTY_SCOPE_DECISION";

pub const PIN_ONLY_POSITIVE_STATES: &[&str] = &["pinned_cert_verified", "pinned_spki_verified"];
pub const EXPECTED_CHAIN_POLICY_PIN_ONLY: &str = "pin_only_no_webpki_chain_validation";
pub const EXPECTED_TIME_POLICY_UNVALIDATED: &str = "not_validated_stage0";

#[derive(Clone, Copy)]
pub struct ProviderTrustHonestyInput<'a> {
    pub provider_id: Option<&'a str>,
    pub trust_state: Option<&'a str>,
    pub chain_policy: Option<&'a str>,
    pub time_policy: Option<&'a str>,
    pub development_bypass: bool,
    pub claims_chain_validated: bool,
    pub claims_time_validated: bool,
}

impl<'a> ProviderTrustHonestyInput<'a> {
    pub const fn empty() -> Self {
        Self {
            provider_id: None,
            trust_state: None,
            chain_policy: None,
            time_policy: None,
            development_bypass: false,
            claims_chain_validated: false,
            claims_time_validated: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderTrustHonestyDecision {
    pub performed: bool,
    pub status: &'static str,
    pub reason: &'static str,
    pub honest: bool,
    pub chain_validated: bool,
    pub time_validated: bool,
    pub authorizes_provider_request: bool,
    pub authorizes_provider_export: bool,
}

pub fn evaluate_provider_trust_honesty(
    input: &ProviderTrustHonestyInput<'_>,
) -> ProviderTrustHonestyDecision {
    if let Err(decision) = require_str(input.provider_id, "missing_provider_id") {
        return decision;
    }
    let trust_state = match input.trust_state {
        Some(value) => value,
        None => return denied("missing_trust_state"),
    };
    let chain_policy = match input.chain_policy {
        Some(value) => value,
        None => return denied("missing_chain_policy"),
    };
    let time_policy = match input.time_policy {
        Some(value) => value,
        None => return denied("missing_time_policy"),
    };

    if input.development_bypass {
        return denied("dev_bypass_not_honest_trust");
    }
    if chain_policy != EXPECTED_CHAIN_POLICY_PIN_ONLY {
        return denied("unsupported_chain_policy");
    }
    if time_policy != EXPECTED_TIME_POLICY_UNVALIDATED {
        return denied("unsupported_time_policy");
    }
    if input.claims_chain_validated {
        return denied("chain_validation_overclaimed");
    }
    if input.claims_time_validated {
        return denied("time_validation_overclaimed");
    }
    if trust_state == "webpki_verified" {
        return denied("webpki_state_overclaims_unbuilt_chain");
    }
    if !PIN_ONLY_POSITIVE_STATES.contains(&trust_state) {
        return denied("trust_state_not_pin_verified");
    }

    ProviderTrustHonestyDecision {
        performed: true,
        status: "trust_labels_honest",
        reason: "honest_pin_only_time_unvalidated_grants_nothing",
        honest: true,
        chain_validated: false,
        time_validated: false,
        authorizes_provider_request: false,
        authorizes_provider_export: false,
    }
}

fn require_str<'a>(
    actual: Option<&'a str>,
    missing: &'static str,
) -> Result<&'a str, ProviderTrustHonestyDecision> {
    match actual {
        Some(value) if !value.is_empty() => Ok(value),
        _ => Err(denied(missing)),
    }
}

fn denied(reason: &'static str) -> ProviderTrustHonestyDecision {
    ProviderTrustHonestyDecision {
        performed: false,
        status: "denied",
        reason,
        honest: false,
        chain_validated: false,
        time_validated: false,
        authorizes_provider_request: false,
        authorizes_provider_export: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> ProviderTrustHonestyInput<'static> {
        ProviderTrustHonestyInput {
            provider_id: Some("provider.openai.direct"),
            trust_state: Some("pinned_spki_verified"),
            chain_policy: Some(EXPECTED_CHAIN_POLICY_PIN_ONLY),
            time_policy: Some(EXPECTED_TIME_POLICY_UNVALIDATED),
            development_bypass: false,
            claims_chain_validated: false,
            claims_time_validated: false,
        }
    }

    #[test]
    fn accepts_valid_pin_only_unvalidated_time_labels() {
        assert_eq!(
            evaluate_provider_trust_honesty(&valid_input()),
            ProviderTrustHonestyDecision {
                performed: true,
                status: "trust_labels_honest",
                reason: "honest_pin_only_time_unvalidated_grants_nothing",
                honest: true,
                chain_validated: false,
                time_validated: false,
                authorizes_provider_request: false,
                authorizes_provider_export: false,
            }
        );
    }

    #[test]
    fn accepts_legacy_pinned_cert_as_pin_only_positive() {
        let mut input = valid_input();
        input.trust_state = Some("pinned_cert_verified");
        assert_eq!(
            evaluate_provider_trust_honesty(&input).reason,
            "honest_pin_only_time_unvalidated_grants_nothing"
        );
    }

    #[derive(Clone, Copy)]
    enum Mutation {
        MissingProviderId,
        MissingTrustState,
        MissingChainPolicy,
        MissingTimePolicy,
        DevelopmentBypass,
        UnsupportedChainPolicy,
        UnsupportedTimePolicy,
        ClaimsChainValidated,
        ClaimsTimeValidated,
        WebPkiState,
        TrustStateNotPinVerified,
    }

    fn apply(input: &mut ProviderTrustHonestyInput<'static>, mutation: Mutation) {
        match mutation {
            Mutation::MissingProviderId => input.provider_id = None,
            Mutation::MissingTrustState => input.trust_state = None,
            Mutation::MissingChainPolicy => input.chain_policy = None,
            Mutation::MissingTimePolicy => input.time_policy = None,
            Mutation::DevelopmentBypass => input.development_bypass = true,
            Mutation::UnsupportedChainPolicy => input.chain_policy = Some("webpki_chain_validated"),
            Mutation::UnsupportedTimePolicy => input.time_policy = Some("trusted_wall_clock"),
            Mutation::ClaimsChainValidated => input.claims_chain_validated = true,
            Mutation::ClaimsTimeValidated => input.claims_time_validated = true,
            Mutation::WebPkiState => input.trust_state = Some("webpki_verified"),
            Mutation::TrustStateNotPinVerified => input.trust_state = Some("pin_config_missing"),
        }
    }

    #[test]
    fn denial_truth_table_names_first_failed_pin() {
        let cases = [
            (Mutation::MissingProviderId, "missing_provider_id"),
            (Mutation::MissingTrustState, "missing_trust_state"),
            (Mutation::MissingChainPolicy, "missing_chain_policy"),
            (Mutation::MissingTimePolicy, "missing_time_policy"),
            (Mutation::DevelopmentBypass, "dev_bypass_not_honest_trust"),
            (Mutation::UnsupportedChainPolicy, "unsupported_chain_policy"),
            (Mutation::UnsupportedTimePolicy, "unsupported_time_policy"),
            (
                Mutation::ClaimsChainValidated,
                "chain_validation_overclaimed",
            ),
            (Mutation::ClaimsTimeValidated, "time_validation_overclaimed"),
            (
                Mutation::WebPkiState,
                "webpki_state_overclaims_unbuilt_chain",
            ),
            (
                Mutation::TrustStateNotPinVerified,
                "trust_state_not_pin_verified",
            ),
        ];
        assert_eq!(cases.len(), 11);

        let mut idx = 0usize;
        while idx < cases.len() {
            let mut input = valid_input();
            apply(&mut input, cases[idx].0);
            let decision = evaluate_provider_trust_honesty(&input);
            assert_eq!(decision.status, "denied");
            assert_eq!(decision.reason, cases[idx].1);
            assert!(!decision.performed);
            assert!(!decision.honest);

            let mut other = 0usize;
            while other < idx {
                assert_ne!(cases[idx].1, cases[other].1);
                other += 1;
            }
            idx += 1;
        }
    }

    #[test]
    fn honesty_critical_denials_are_explicit() {
        let cases = [
            (
                Mutation::ClaimsChainValidated,
                "chain_validation_overclaimed",
            ),
            (Mutation::ClaimsTimeValidated, "time_validation_overclaimed"),
            (
                Mutation::WebPkiState,
                "webpki_state_overclaims_unbuilt_chain",
            ),
            (Mutation::DevelopmentBypass, "dev_bypass_not_honest_trust"),
        ];

        let mut idx = 0usize;
        while idx < cases.len() {
            let mut input = valid_input();
            apply(&mut input, cases[idx].0);
            assert_eq!(evaluate_provider_trust_honesty(&input).reason, cases[idx].1);
            idx += 1;
        }
    }

    #[test]
    fn empty_provider_id_is_missing_provider_id() {
        let mut input = valid_input();
        input.provider_id = Some("");
        assert_eq!(
            evaluate_provider_trust_honesty(&input).reason,
            "missing_provider_id"
        );
    }

    #[test]
    fn success_never_authorizes_request_or_export() {
        let decision = evaluate_provider_trust_honesty(&valid_input());
        assert!(decision.honest);
        assert!(!decision.chain_validated);
        assert!(!decision.time_validated);
        assert!(!decision.authorizes_provider_request);
        assert!(!decision.authorizes_provider_export);
    }
}
