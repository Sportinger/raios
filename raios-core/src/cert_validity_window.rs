#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CertValidityDateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CertValidityWindowDecision {
    pub status: &'static str,
    pub basis_source: &'static str,
    pub trusted: bool,
    pub source_verified: bool,
    pub validates_cert_time: bool,
    pub authorizes_provider_request: bool,
    pub authorizes_provider_export: bool,
    pub durable_write: bool,
    pub capability_granted: bool,
}

pub fn evaluate_cert_validity_window_unverified_basis(
    not_before: CertValidityDateTime,
    now: CertValidityDateTime,
    not_after: CertValidityDateTime,
) -> CertValidityWindowDecision {
    if !valid(not_before) || !valid(now) || !valid(not_after) {
        return decision("invalid_window");
    }

    let not_before = tuple(not_before);
    let now = tuple(now);
    let not_after = tuple(not_after);
    if not_before > not_after {
        return decision("invalid_window");
    }
    if now < not_before {
        return decision("before_not_yet_valid_unverified_basis");
    }
    if now > not_after {
        return decision("after_expired_unverified_basis");
    }
    decision("within_window_unverified_basis")
}

fn valid(value: CertValidityDateTime) -> bool {
    (1..=9999).contains(&value.year)
        && (1..=12).contains(&value.month)
        && (1..=31).contains(&value.day)
        && value.hour <= 23
        && value.minute <= 59
        && value.second <= 59
}

fn tuple(value: CertValidityDateTime) -> (u16, u8, u8, u8, u8, u8) {
    (
        value.year,
        value.month,
        value.day,
        value.hour,
        value.minute,
        value.second,
    )
}

fn decision(status: &'static str) -> CertValidityWindowDecision {
    CertValidityWindowDecision {
        status,
        basis_source: "cmos_rtc_unverified",
        trusted: false,
        source_verified: false,
        validates_cert_time: false,
        authorizes_provider_request: false,
        authorizes_provider_export: false,
        durable_write: false,
        capability_granted: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOT_BEFORE: CertValidityDateTime = dt(2026, 1, 2, 3, 4, 5);
    const NOT_AFTER: CertValidityDateTime = dt(2026, 2, 3, 4, 5, 6);

    const fn dt(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> CertValidityDateTime {
        CertValidityDateTime {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    fn eval(now: CertValidityDateTime) -> CertValidityWindowDecision {
        evaluate_cert_validity_window_unverified_basis(NOT_BEFORE, now, NOT_AFTER)
    }

    #[test]
    fn before_window_is_not_yet_valid_on_unverified_basis() {
        assert_eq!(
            eval(dt(2026, 1, 2, 3, 4, 4)).status,
            "before_not_yet_valid_unverified_basis"
        );
    }

    #[test]
    fn within_window_is_within_on_unverified_basis() {
        assert_eq!(
            eval(dt(2026, 1, 20, 0, 0, 0)).status,
            "within_window_unverified_basis"
        );
    }

    #[test]
    fn after_window_is_expired_on_unverified_basis() {
        assert_eq!(
            eval(dt(2026, 2, 3, 4, 5, 7)).status,
            "after_expired_unverified_basis"
        );
    }

    #[test]
    fn equals_not_before_is_inside_window() {
        assert_eq!(eval(NOT_BEFORE).status, "within_window_unverified_basis");
    }

    #[test]
    fn equals_not_after_is_inside_window() {
        assert_eq!(eval(NOT_AFTER).status, "within_window_unverified_basis");
    }

    #[test]
    fn invalid_component_is_invalid_window() {
        assert_eq!(
            evaluate_cert_validity_window_unverified_basis(
                NOT_BEFORE,
                dt(2026, 13, 1, 0, 0, 0),
                NOT_AFTER,
            )
            .status,
            "invalid_window"
        );
    }

    #[test]
    fn reversed_window_is_invalid_window() {
        assert_eq!(
            evaluate_cert_validity_window_unverified_basis(
                NOT_AFTER,
                dt(2026, 1, 20, 0, 0, 0),
                NOT_BEFORE,
            )
            .status,
            "invalid_window"
        );
    }

    #[test]
    fn all_outcomes_grant_nothing() {
        let decisions = [
            eval(dt(2026, 1, 2, 3, 4, 4)),
            eval(dt(2026, 1, 20, 0, 0, 0)),
            eval(dt(2026, 2, 3, 4, 5, 7)),
            evaluate_cert_validity_window_unverified_basis(
                NOT_BEFORE,
                dt(2026, 13, 1, 0, 0, 0),
                NOT_AFTER,
            ),
            evaluate_cert_validity_window_unverified_basis(
                NOT_AFTER,
                dt(2026, 1, 20, 0, 0, 0),
                NOT_BEFORE,
            ),
        ];

        for decision in decisions {
            assert_eq!(decision.basis_source, "cmos_rtc_unverified");
            assert!(!decision.trusted);
            assert!(!decision.source_verified);
            assert!(!decision.validates_cert_time);
            assert!(!decision.authorizes_provider_request);
            assert!(!decision.authorizes_provider_export);
            assert!(!decision.durable_write);
            assert!(!decision.capability_granted);
        }
    }
}
