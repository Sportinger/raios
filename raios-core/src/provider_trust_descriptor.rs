//! Provider-agnostic trust descriptors over the M10A-1 honesty evaluator.
//!
//! This module grants nothing. It only gives provider metadata one typed shape
//! and routes its trust labels through `scoped_provider_trust_honesty`.

use alloc::vec;

use crate::record::{sha256_of_json, Field, Value};
use crate::scoped_provider_trust_honesty::{
    evaluate_provider_trust_honesty, ProviderTrustHonestyDecision, ProviderTrustHonestyInput,
    EXPECTED_CHAIN_POLICY_PIN_ONLY, EXPECTED_TIME_POLICY_UNVALIDATED,
};

pub const PROVIDER_TRUST_VERIFIER_METADATA_SCHEMA: &str =
    "raios.provider_trust_verifier_metadata.v0";
pub const TLS13_ECDSA_P256_SHA256_POLICY: &str = "tls13_ecdsa_secp256r1_sha256_required";
pub const PIN_POLICY_LEAF_OR_SPKI_SHA256: &str =
    "configured_leaf_or_spki_sha256_required_optional_spki_rotation";

#[derive(Clone, Copy)]
pub struct ProviderTrustDescriptor<'a> {
    pub provider_id: &'a str,
    pub id: &'a str,
    pub host: &'a str,
    pub port: &'a str,
    pub transport: &'a str,
    pub hostname_policy: &'a str,
    pub pin_policy: &'a str,
    pub chain_policy: &'a str,
    pub time_policy: &'a str,
    pub certificate_verify_policy: &'a str,
    pub trust_state: &'a str,
    pub development_bypass: bool,
    pub claims_chain_validated: bool,
    pub claims_time_validated: bool,
}

impl<'a> ProviderTrustDescriptor<'a> {
    pub fn honesty_input(&self) -> ProviderTrustHonestyInput<'a> {
        ProviderTrustHonestyInput {
            provider_id: Some(self.provider_id),
            trust_state: Some(self.trust_state),
            chain_policy: Some(self.chain_policy),
            time_policy: Some(self.time_policy),
            development_bypass: self.development_bypass,
            claims_chain_validated: self.claims_chain_validated,
            claims_time_validated: self.claims_time_validated,
        }
    }

    pub fn to_record_value(&self) -> Value<'a> {
        Value::Object(vec![
            Field::new(
                "schema",
                Value::Str(PROVIDER_TRUST_VERIFIER_METADATA_SCHEMA),
            ),
            Field::new("provider_id", Value::Str(self.provider_id)),
            Field::new("id", Value::Str(self.id)),
            Field::new("host", Value::Str(self.host)),
            Field::new("port", Value::Str(self.port)),
            Field::new("transport", Value::Str(self.transport)),
            Field::new("hostname_policy", Value::Str(self.hostname_policy)),
            Field::new("pin_policy", Value::Str(self.pin_policy)),
            Field::new("chain_policy", Value::Str(self.chain_policy)),
            Field::new("time_policy", Value::Str(self.time_policy)),
            Field::new(
                "certificate_verify_policy",
                Value::Str(self.certificate_verify_policy),
            ),
        ])
    }

    pub fn record_sha256(&self) -> [u8; 32] {
        sha256_of_json(&self.to_record_value())
    }
}

pub fn evaluate_provider_trust_descriptor_honesty(
    descriptor: &ProviderTrustDescriptor<'_>,
) -> ProviderTrustHonestyDecision {
    evaluate_provider_trust_honesty(&descriptor.honesty_input())
}

pub const OPENAI_PROVIDER_TRUST_DESCRIPTOR: ProviderTrustDescriptor<'static> =
    ProviderTrustDescriptor {
        provider_id: "openai",
        id: "openai.pinned_tls13_p256_sha256.v0",
        host: "api.openai.com",
        port: "443",
        transport: "tls1.3",
        hostname_policy: "exact_api.openai.com_required",
        pin_policy: PIN_POLICY_LEAF_OR_SPKI_SHA256,
        chain_policy: EXPECTED_CHAIN_POLICY_PIN_ONLY,
        time_policy: EXPECTED_TIME_POLICY_UNVALIDATED,
        certificate_verify_policy: TLS13_ECDSA_P256_SHA256_POLICY,
        trust_state: "pinned_spki_verified",
        development_bypass: false,
        claims_chain_validated: false,
        claims_time_validated: false,
    };

// Test-only synthetic Anthropic-shaped descriptor: no real pins/chain/time facts
// are configured here, so its trust state must stay `pin_config_missing`.
pub const SYNTHETIC_ANTHROPIC_PROVIDER_TRUST_DESCRIPTOR: ProviderTrustDescriptor<'static> =
    ProviderTrustDescriptor {
        provider_id: "anthropic",
        id: "anthropic.pinned_tls13_p256_sha256.selftest.v0",
        host: "api.anthropic.com",
        port: "443",
        transport: "tls1.3",
        hostname_policy: "exact_api.anthropic.com_required",
        pin_policy: PIN_POLICY_LEAF_OR_SPKI_SHA256,
        chain_policy: EXPECTED_CHAIN_POLICY_PIN_ONLY,
        time_policy: EXPECTED_TIME_POLICY_UNVALIDATED,
        certificate_verify_policy: TLS13_ECDSA_P256_SHA256_POLICY,
        trust_state: "pin_config_missing",
        development_bypass: false,
        claims_chain_validated: false,
        claims_time_validated: false,
    };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_descriptor_pinned_is_honest_grants_nothing() {
        let decision =
            evaluate_provider_trust_descriptor_honesty(&OPENAI_PROVIDER_TRUST_DESCRIPTOR);

        assert!(decision.performed);
        assert!(decision.honest);
        assert!(!decision.chain_validated);
        assert!(!decision.time_validated);
        assert!(!decision.authorizes_provider_request);
        assert!(!decision.authorizes_provider_export);
    }

    #[test]
    fn anthropic_synthetic_descriptor_unpinned_denies_honestly() {
        let decision = evaluate_provider_trust_descriptor_honesty(
            &SYNTHETIC_ANTHROPIC_PROVIDER_TRUST_DESCRIPTOR,
        );

        assert_eq!(decision.reason, "trust_state_not_pin_verified");
        assert!(!decision.authorizes_provider_request);
        assert!(!decision.authorizes_provider_export);
    }

    #[test]
    fn overclaims_denied_identically_for_both() {
        let descriptors = [
            OPENAI_PROVIDER_TRUST_DESCRIPTOR,
            SYNTHETIC_ANTHROPIC_PROVIDER_TRUST_DESCRIPTOR,
        ];

        for descriptor in descriptors {
            let mut overclaimed_chain = descriptor;
            overclaimed_chain.claims_chain_validated = true;
            assert_eq!(
                evaluate_provider_trust_descriptor_honesty(&overclaimed_chain).reason,
                "chain_validation_overclaimed"
            );

            let mut overclaimed_time = descriptor;
            overclaimed_time.claims_time_validated = true;
            assert_eq!(
                evaluate_provider_trust_descriptor_honesty(&overclaimed_time).reason,
                "time_validation_overclaimed"
            );

            let mut overclaimed_webpki = descriptor;
            overclaimed_webpki.trust_state = "webpki_verified";
            assert_eq!(
                evaluate_provider_trust_descriptor_honesty(&overclaimed_webpki).reason,
                "webpki_state_overclaims_unbuilt_chain"
            );
        }
    }

    #[test]
    fn distinct_providers_share_one_shape_but_hash_differently() {
        assert_ne!(
            OPENAI_PROVIDER_TRUST_DESCRIPTOR.record_sha256(),
            SYNTHETIC_ANTHROPIC_PROVIDER_TRUST_DESCRIPTOR.record_sha256()
        );

        let _ = evaluate_provider_trust_descriptor_honesty(&OPENAI_PROVIDER_TRUST_DESCRIPTOR);
        let _ = evaluate_provider_trust_descriptor_honesty(
            &SYNTHETIC_ANTHROPIC_PROVIDER_TRUST_DESCRIPTOR,
        );
    }
}
