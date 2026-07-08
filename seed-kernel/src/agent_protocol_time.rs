use alloc::vec;

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_null as n, record_str as s, record_str_or_null as opt_s,
    },
    time::{read_cmos_rtc_wall_clock, CmosRtcError, CmosRtcWallClock},
};
use raios_core::{
    cert_validity_window::{evaluate_cert_validity_window_unverified_basis, CertValidityDateTime},
    record::Value as V,
    scoped_time_authority_honesty::{
        evaluate_time_authority_honesty, TimeAuthorityHonestyInput, EXPECTED_TIME_SOURCE,
        SCOPED_TIME_AUTHORITY_HONESTY_DECISION_ID, SCOPED_TIME_AUTHORITY_HONESTY_DECISION_MARKER,
        SCOPED_TIME_AUTHORITY_HONESTY_DECISION_SCHEMA,
    },
};

pub(crate) fn emit_system_time_authority(_request: &str) {
    let (read_status, clock) = match read_cmos_rtc_wall_clock() {
        Ok(clock) => ("ok", Some(clock)),
        Err(error) => (cmos_error_name(error), None),
    };
    let input = TimeAuthorityHonestyInput {
        source: Some(EXPECTED_TIME_SOURCE),
        trusted: false,
        source_verified: false,
        validates_cert_time: false,
        timezone_validated: false,
        authorizes_provider_request: false,
        authorizes_provider_export: false,
        durable_write: false,
        capability_granted: false,
    };
    let decision = evaluate_time_authority_honesty(&input);

    begin_response("system.time_authority");
    emit_record_fields(
        vec![
            f("schema", s("raios.time_authority_status.v0")),
            f(
                "decision_schema",
                s(SCOPED_TIME_AUTHORITY_HONESTY_DECISION_SCHEMA),
            ),
            f("decision_id", s(SCOPED_TIME_AUTHORITY_HONESTY_DECISION_ID)),
            f(
                "decision_marker",
                s(SCOPED_TIME_AUTHORITY_HONESTY_DECISION_MARKER),
            ),
            f("source", s(EXPECTED_TIME_SOURCE)),
            f("classification", s("local_only")),
            f("scope", s("current_boot")),
            f("read_status", s(read_status)),
            f("year", clock_u16(clock, |clock| clock.year)),
            f("month", clock_u8(clock, |clock| clock.month)),
            f("day", clock_u8(clock, |clock| clock.day)),
            f("hour", clock_u8(clock, |clock| clock.hour)),
            f("minute", clock_u8(clock, |clock| clock.minute)),
            f("second", clock_u8(clock, |clock| clock.second)),
            f("data_mode", opt_s(clock.map(|clock| clock.data_mode))),
            f("hour_mode", opt_s(clock.map(|clock| clock.hour_mode))),
            f(
                "century_source",
                opt_s(clock.map(|clock| clock.century_source)),
            ),
            f("trusted", b(decision.trusted)),
            f("source_verified", b(decision.source_verified)),
            f("host_settable", b(true)),
            f("timezone_validated", b(decision.timezone_validated)),
            f("validates_cert_time", b(decision.validates_cert_time)),
            f(
                "authorizes_provider_request",
                b(decision.authorizes_provider_request),
            ),
            f(
                "authorizes_provider_export",
                b(decision.authorizes_provider_export),
            ),
            f("durable_write", b(decision.durable_write)),
            f("capability_granted", b(decision.capability_granted)),
            f("provider_write", s("not_attempted")),
            f("transmission", b(false)),
            f("performed", b(decision.performed)),
            f("status", s(decision.status)),
            f("reason", s(decision.reason)),
            f("honest", b(decision.honest)),
        ],
        6,
    );
    end_response("system.time_authority");
}

pub(crate) fn emit_system_cert_time_check_selftest(_request: &str) {
    let (read_status, clock) = match read_cmos_rtc_wall_clock() {
        Ok(clock) => ("ok", Some(clock)),
        Err(error) => (cmos_error_name(error), None),
    };
    let now = clock.map(clock_to_cert_datetime);

    begin_response("system.cert_time_check_selftest");
    emit_record_fields(
        vec![
            f("schema", s("raios.cert_time_check_selftest.v0")),
            f("test_infrastructure", b(true)),
            f(
                "fixture_kind",
                s("fixed_synthetic_certificate_validity_window_not_der_not_live"),
            ),
            f("basis_source", s("cmos_rtc_unverified")),
            f("read_status", s(read_status)),
            f("now_source", s("cmos_rtc_unverified")),
            f("now", datetime_or_null(now)),
            f("trusted", b(false)),
            f("source_verified", b(false)),
            f("validates_cert_time", b(false)),
            f("authorizes_provider_request", b(false)),
            f("authorizes_provider_export", b(false)),
            f("durable_write", b(false)),
            f("capability_granted", b(false)),
            f("provider_write", s("not_attempted")),
            f("transmission", b(false)),
            f("cases", cert_time_check_cases(now)),
        ],
        6,
    );
    end_response("system.cert_time_check_selftest");
}

fn cmos_error_name(error: CmosRtcError) -> &'static str {
    match error {
        CmosRtcError::UpdateNeverSettled => "UpdateNeverSettled",
        CmosRtcError::ImplausibleField => "ImplausibleField",
    }
}

fn clock_u16(clock: Option<CmosRtcWallClock>, value: fn(CmosRtcWallClock) -> u16) -> V<'static> {
    match clock {
        Some(clock) => V::U64(value(clock) as u64),
        None => n(),
    }
}

fn clock_to_cert_datetime(clock: CmosRtcWallClock) -> CertValidityDateTime {
    CertValidityDateTime {
        year: clock.year,
        month: clock.month,
        day: clock.day,
        hour: clock.hour,
        minute: clock.minute,
        second: clock.second,
    }
}

fn cert_time_check_cases(now: Option<CertValidityDateTime>) -> V<'static> {
    match now {
        Some(now) => V::Array(vec![
            cert_time_check_case(
                "wide",
                CertValidityDateTime {
                    year: 2020,
                    month: 1,
                    day: 1,
                    hour: 0,
                    minute: 0,
                    second: 0,
                },
                now,
                CertValidityDateTime {
                    year: 9999,
                    month: 12,
                    day: 31,
                    hour: 23,
                    minute: 59,
                    second: 59,
                },
                "within_window_unverified_basis",
            ),
            cert_time_check_case(
                "expired",
                CertValidityDateTime {
                    year: 2000,
                    month: 1,
                    day: 1,
                    hour: 0,
                    minute: 0,
                    second: 0,
                },
                now,
                CertValidityDateTime {
                    year: 2010,
                    month: 1,
                    day: 1,
                    hour: 0,
                    minute: 0,
                    second: 0,
                },
                "after_expired_unverified_basis",
            ),
        ]),
        None => V::Array(vec![]),
    }
}

fn cert_time_check_case(
    name: &'static str,
    not_before: CertValidityDateTime,
    now: CertValidityDateTime,
    not_after: CertValidityDateTime,
    expected_status: &'static str,
) -> V<'static> {
    let decision = evaluate_cert_validity_window_unverified_basis(not_before, now, not_after);
    V::Object(vec![
        f("case", s(name)),
        f("fixture_source", s("fixed_synthetic_not_der_not_live")),
        f("basis_source", s(decision.basis_source)),
        f("not_before", datetime_value(not_before)),
        f("not_after", datetime_value(not_after)),
        f("status", s(decision.status)),
        f("expected_status", s(expected_status)),
        f("passed", b(decision.status == expected_status)),
        f("trusted", b(decision.trusted)),
        f("source_verified", b(decision.source_verified)),
        f("validates_cert_time", b(decision.validates_cert_time)),
        f(
            "authorizes_provider_request",
            b(decision.authorizes_provider_request),
        ),
        f(
            "authorizes_provider_export",
            b(decision.authorizes_provider_export),
        ),
        f("durable_write", b(decision.durable_write)),
        f("capability_granted", b(decision.capability_granted)),
        f("provider_write", s("not_attempted")),
        f("transmission", b(false)),
    ])
}

fn datetime_or_null(value: Option<CertValidityDateTime>) -> V<'static> {
    match value {
        Some(value) => datetime_value(value),
        None => n(),
    }
}

fn datetime_value(value: CertValidityDateTime) -> V<'static> {
    V::Object(vec![
        f("year", V::U64(value.year as u64)),
        f("month", V::U64(value.month as u64)),
        f("day", V::U64(value.day as u64)),
        f("hour", V::U64(value.hour as u64)),
        f("minute", V::U64(value.minute as u64)),
        f("second", V::U64(value.second as u64)),
    ])
}

fn clock_u8(clock: Option<CmosRtcWallClock>, value: fn(CmosRtcWallClock) -> u8) -> V<'static> {
    match clock {
        Some(clock) => V::U64(value(clock) as u64),
        None => n(),
    }
}
