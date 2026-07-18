//! Scoped honesty check for Stage-0 wall-clock labels (M10C-1).
//!
//! This evaluator grants nothing. It only certifies that the CMOS RTC wall
//! clock is labeled as local, host-settable, and unverified.

pub const SCOPED_TIME_AUTHORITY_HONESTY_DECISION_SCHEMA: &str =
    "raios.scoped_time_authority_honesty_decision.v0";
pub const SCOPED_TIME_AUTHORITY_HONESTY_DECISION_ID: &str =
    "scoped_time_authority_honesty.current_boot.v0";
pub const SCOPED_TIME_AUTHORITY_HONESTY_DECISION_MARKER: &str =
    "RAIOS_TIME_AUTHORITY_HONESTY_SCOPE_DECISION";

pub const EXPECTED_TIME_SOURCE: &str = "cmos_rtc_unverified";

#[derive(Clone, Copy)]
pub struct TimeAuthorityHonestyInput<'a> {
    pub source: Option<&'a str>,
    pub trusted: bool,
    pub source_verified: bool,
    pub validates_cert_time: bool,
    pub timezone_validated: bool,
    pub authorizes_provider_request: bool,
    pub authorizes_provider_export: bool,
    pub durable_write: bool,
    pub capability_granted: bool,
}

impl<'a> TimeAuthorityHonestyInput<'a> {
    pub const fn empty() -> Self {
        Self {
            source: None,
            trusted: false,
            source_verified: false,
            validates_cert_time: false,
            timezone_validated: false,
            authorizes_provider_request: false,
            authorizes_provider_export: false,
            durable_write: false,
            capability_granted: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeAuthorityHonestyDecision {
    pub performed: bool,
    pub status: &'static str,
    pub reason: &'static str,
    pub honest: bool,
    pub trusted: bool,
    pub source_verified: bool,
    pub validates_cert_time: bool,
    pub timezone_validated: bool,
    pub authorizes_provider_request: bool,
    pub authorizes_provider_export: bool,
    pub durable_write: bool,
    pub capability_granted: bool,
}

pub fn evaluate_time_authority_honesty(
    input: &TimeAuthorityHonestyInput<'_>,
) -> TimeAuthorityHonestyDecision {
    let source = match input.source {
        Some(value) if !value.is_empty() => value,
        _ => return denied("missing_time_source"),
    };

    if source != EXPECTED_TIME_SOURCE {
        return denied("time_source_not_cmos_rtc_unverified");
    }
    if input.trusted {
        return denied("trusted_time_overclaimed");
    }
    if input.source_verified {
        return denied("source_verified_overclaimed");
    }
    if input.validates_cert_time {
        return denied("cert_time_validation_overclaimed");
    }
    if input.timezone_validated {
        return denied("timezone_validation_overclaimed");
    }
    if input.authorizes_provider_request {
        return denied("provider_request_authority_overclaimed");
    }
    if input.authorizes_provider_export {
        return denied("provider_export_authority_overclaimed");
    }
    if input.durable_write {
        return denied("durable_write_overclaimed");
    }
    if input.capability_granted {
        return denied("capability_grant_overclaimed");
    }

    TimeAuthorityHonestyDecision {
        performed: true,
        status: "time_labels_honest",
        reason: "honest_cmos_rtc_unverified_grants_nothing",
        honest: true,
        trusted: false,
        source_verified: false,
        validates_cert_time: false,
        timezone_validated: false,
        authorizes_provider_request: false,
        authorizes_provider_export: false,
        durable_write: false,
        capability_granted: false,
    }
}

fn denied(reason: &'static str) -> TimeAuthorityHonestyDecision {
    TimeAuthorityHonestyDecision {
        performed: false,
        status: "denied",
        reason,
        honest: false,
        trusted: false,
        source_verified: false,
        validates_cert_time: false,
        timezone_validated: false,
        authorizes_provider_request: false,
        authorizes_provider_export: false,
        durable_write: false,
        capability_granted: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> TimeAuthorityHonestyInput<'static> {
        TimeAuthorityHonestyInput {
            source: Some(EXPECTED_TIME_SOURCE),
            trusted: false,
            source_verified: false,
            validates_cert_time: false,
            timezone_validated: false,
            authorizes_provider_request: false,
            authorizes_provider_export: false,
            durable_write: false,
            capability_granted: false,
        }
    }

    #[test]
    fn accepts_untrusted_cmos_rtc_source_and_grants_nothing() {
        assert_eq!(
            evaluate_time_authority_honesty(&valid_input()),
            TimeAuthorityHonestyDecision {
                performed: true,
                status: "time_labels_honest",
                reason: "honest_cmos_rtc_unverified_grants_nothing",
                honest: true,
                trusted: false,
                source_verified: false,
                validates_cert_time: false,
                timezone_validated: false,
                authorizes_provider_request: false,
                authorizes_provider_export: false,
                durable_write: false,
                capability_granted: false,
            }
        );
    }

    #[derive(Clone, Copy)]
    enum Mutation {
        MissingSource,
        WrongSource,
        Trusted,
        SourceVerified,
        ValidatesCertTime,
        TimezoneValidated,
        AuthorizesProviderRequest,
        AuthorizesProviderExport,
        DurableWrite,
        CapabilityGranted,
    }

    fn apply(input: &mut TimeAuthorityHonestyInput<'static>, mutation: Mutation) {
        match mutation {
            Mutation::MissingSource => input.source = None,
            Mutation::WrongSource => input.source = Some("trusted_wall_clock"),
            Mutation::Trusted => input.trusted = true,
            Mutation::SourceVerified => input.source_verified = true,
            Mutation::ValidatesCertTime => input.validates_cert_time = true,
            Mutation::TimezoneValidated => input.timezone_validated = true,
            Mutation::AuthorizesProviderRequest => input.authorizes_provider_request = true,
            Mutation::AuthorizesProviderExport => input.authorizes_provider_export = true,
            Mutation::DurableWrite => input.durable_write = true,
            Mutation::CapabilityGranted => input.capability_granted = true,
        }
    }

    #[test]
    fn denial_truth_table_reasons_are_pairwise_unique() {
        let cases = [
            (Mutation::MissingSource, "missing_time_source"),
            (Mutation::WrongSource, "time_source_not_cmos_rtc_unverified"),
            (Mutation::Trusted, "trusted_time_overclaimed"),
            (Mutation::SourceVerified, "source_verified_overclaimed"),
            (
                Mutation::ValidatesCertTime,
                "cert_time_validation_overclaimed",
            ),
            (
                Mutation::TimezoneValidated,
                "timezone_validation_overclaimed",
            ),
            (
                Mutation::AuthorizesProviderRequest,
                "provider_request_authority_overclaimed",
            ),
            (
                Mutation::AuthorizesProviderExport,
                "provider_export_authority_overclaimed",
            ),
            (Mutation::DurableWrite, "durable_write_overclaimed"),
            (Mutation::CapabilityGranted, "capability_grant_overclaimed"),
        ];
        assert_eq!(cases.len(), 10);

        let mut idx = 0usize;
        while idx < cases.len() {
            let mut input = valid_input();
            apply(&mut input, cases[idx].0);
            let decision = evaluate_time_authority_honesty(&input);
            assert_eq!(decision.status, "denied");
            assert_eq!(decision.reason, cases[idx].1);
            assert!(!decision.performed);
            assert!(!decision.honest);
            assert!(!decision.trusted);
            assert!(!decision.validates_cert_time);
            assert!(!decision.authorizes_provider_request);
            assert!(!decision.authorizes_provider_export);
            assert!(!decision.durable_write);
            assert!(!decision.capability_granted);

            let mut other = 0usize;
            while other < idx {
                assert_ne!(cases[idx].1, cases[other].1);
                other += 1;
            }
            idx += 1;
        }
    }

    #[test]
    fn overclaim_denials_are_explicit_and_unique() {
        let cases = [
            (Mutation::Trusted, "trusted_time_overclaimed"),
            (Mutation::SourceVerified, "source_verified_overclaimed"),
            (
                Mutation::ValidatesCertTime,
                "cert_time_validation_overclaimed",
            ),
            (
                Mutation::TimezoneValidated,
                "timezone_validation_overclaimed",
            ),
            (
                Mutation::AuthorizesProviderRequest,
                "provider_request_authority_overclaimed",
            ),
            (
                Mutation::AuthorizesProviderExport,
                "provider_export_authority_overclaimed",
            ),
            (Mutation::DurableWrite, "durable_write_overclaimed"),
            (Mutation::CapabilityGranted, "capability_grant_overclaimed"),
        ];
        assert_eq!(cases.len(), 8);

        let mut idx = 0usize;
        while idx < cases.len() {
            let mut input = valid_input();
            apply(&mut input, cases[idx].0);
            assert_eq!(evaluate_time_authority_honesty(&input).reason, cases[idx].1);

            let mut other = 0usize;
            while other < idx {
                assert_ne!(cases[idx].1, cases[other].1);
                other += 1;
            }
            idx += 1;
        }
    }

    #[test]
    fn empty_time_source_is_missing_time_source() {
        let mut input = valid_input();
        input.source = Some("");
        assert_eq!(
            evaluate_time_authority_honesty(&input).reason,
            "missing_time_source"
        );
    }
}
