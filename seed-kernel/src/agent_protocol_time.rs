use alloc::vec;

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_null as n, record_sha as sha, record_str as s, record_str_or_null as opt_s,
    },
    time::{read_cmos_rtc_wall_clock, CmosRtcError, CmosRtcWallClock},
};
use raios_core::{
    cert_validity_window::{
        evaluate_cert_validity_window_unverified_basis, parse_x509_cert_validity_window,
        CertValidityDateTime,
    },
    record::Value as V,
    scoped_time_authority_honesty::{
        evaluate_time_authority_honesty, TimeAuthorityHonestyDecision, TimeAuthorityHonestyInput,
        EXPECTED_TIME_SOURCE, SCOPED_TIME_AUTHORITY_HONESTY_DECISION_ID,
        SCOPED_TIME_AUTHORITY_HONESTY_DECISION_MARKER,
        SCOPED_TIME_AUTHORITY_HONESTY_DECISION_SCHEMA,
    },
    sha256_bytes,
};

// Fixed DER fixture decoded from
// vendor/embedded-tls-0.17.0/tests/data/server-cert.pem.
// len=522, sha256=baa2a6c3263fb8170aa2b4013046414a1e4760c2f5e7bfdf88c74f51742e0cb4.
// notBefore=2021-10-13T08:20:42Z, notAfter=2031-10-11T08:20:42Z.
// This is not a live handshake cert and grants no provider trust.
pub(crate) const REAL_TEST_CERT_DER: &[u8] = &[
    0x30, 0x82, 0x02, 0x06, 0x30, 0x82, 0x01, 0xad, 0xa0, 0x03, 0x02, 0x01, 0x02, 0x02, 0x14, 0x2c,
    0x3c, 0x82, 0x61, 0x8b, 0x5e, 0x91, 0xad, 0x92, 0xea, 0x11, 0x02, 0xdb, 0xcf, 0x74, 0xa3, 0xb9,
    0x3e, 0xd1, 0x9d, 0x30, 0x0a, 0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x04, 0x03, 0x02, 0x30,
    0x42, 0x31, 0x0b, 0x30, 0x09, 0x06, 0x03, 0x55, 0x04, 0x06, 0x13, 0x02, 0x58, 0x58, 0x31, 0x15,
    0x30, 0x13, 0x06, 0x03, 0x55, 0x04, 0x07, 0x0c, 0x0c, 0x44, 0x65, 0x66, 0x61, 0x75, 0x6c, 0x74,
    0x20, 0x43, 0x69, 0x74, 0x79, 0x31, 0x1c, 0x30, 0x1a, 0x06, 0x03, 0x55, 0x04, 0x0a, 0x0c, 0x13,
    0x44, 0x65, 0x66, 0x61, 0x75, 0x6c, 0x74, 0x20, 0x43, 0x6f, 0x6d, 0x70, 0x61, 0x6e, 0x79, 0x20,
    0x4c, 0x74, 0x64, 0x30, 0x1e, 0x17, 0x0d, 0x32, 0x31, 0x31, 0x30, 0x31, 0x33, 0x30, 0x38, 0x32,
    0x30, 0x34, 0x32, 0x5a, 0x17, 0x0d, 0x33, 0x31, 0x31, 0x30, 0x31, 0x31, 0x30, 0x38, 0x32, 0x30,
    0x34, 0x32, 0x5a, 0x30, 0x72, 0x31, 0x0b, 0x30, 0x09, 0x06, 0x03, 0x55, 0x04, 0x06, 0x13, 0x02,
    0x4e, 0x4f, 0x31, 0x0e, 0x30, 0x0c, 0x06, 0x03, 0x55, 0x04, 0x08, 0x0c, 0x05, 0x48, 0x61, 0x6d,
    0x61, 0x72, 0x31, 0x0e, 0x30, 0x0c, 0x06, 0x03, 0x55, 0x04, 0x07, 0x0c, 0x05, 0x48, 0x61, 0x6d,
    0x61, 0x72, 0x31, 0x18, 0x30, 0x16, 0x06, 0x03, 0x55, 0x04, 0x0a, 0x0c, 0x0f, 0x47, 0x6c, 0x6f,
    0x62, 0x61, 0x6c, 0x20, 0x53, 0x65, 0x63, 0x75, 0x72, 0x69, 0x74, 0x79, 0x31, 0x15, 0x30, 0x13,
    0x06, 0x03, 0x55, 0x04, 0x0b, 0x0c, 0x0c, 0x48, 0x6f, 0x6c, 0x73, 0x65, 0x74, 0x62, 0x61, 0x6b,
    0x6b, 0x65, 0x6e, 0x31, 0x12, 0x30, 0x10, 0x06, 0x03, 0x55, 0x04, 0x03, 0x0c, 0x09, 0x6c, 0x6f,
    0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73, 0x74, 0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a, 0x86, 0x48,
    0xce, 0x3d, 0x02, 0x01, 0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, 0x03, 0x42,
    0x00, 0x04, 0xc4, 0x99, 0xf6, 0xf3, 0xaa, 0xa1, 0xe2, 0x67, 0x00, 0x8a, 0x5e, 0x01, 0x1f, 0x8c,
    0x05, 0xa3, 0x93, 0xac, 0xcf, 0x94, 0xaf, 0x45, 0xb3, 0x76, 0xd7, 0x7e, 0x3a, 0x36, 0x82, 0xdd,
    0x4d, 0xba, 0xa0, 0x38, 0xc8, 0x27, 0x4e, 0x50, 0xb2, 0x9a, 0xe9, 0xa2, 0x05, 0x1f, 0x20, 0x2f,
    0x7c, 0xcd, 0xf3, 0x1c, 0xd8, 0x8b, 0xe6, 0xf9, 0x39, 0xa5, 0xb0, 0x6d, 0xce, 0x36, 0xba, 0xbd,
    0xa2, 0x23, 0xa3, 0x51, 0x30, 0x4f, 0x30, 0x1f, 0x06, 0x03, 0x55, 0x1d, 0x23, 0x04, 0x18, 0x30,
    0x16, 0x80, 0x14, 0xec, 0x74, 0x3a, 0xe2, 0x98, 0xac, 0x83, 0x53, 0x1a, 0xb0, 0xdf, 0x70, 0x48,
    0xb1, 0x3f, 0x2c, 0x2e, 0x8f, 0x72, 0x3a, 0x30, 0x09, 0x06, 0x03, 0x55, 0x1d, 0x13, 0x04, 0x02,
    0x30, 0x00, 0x30, 0x0b, 0x06, 0x03, 0x55, 0x1d, 0x0f, 0x04, 0x04, 0x03, 0x02, 0x04, 0xf0, 0x30,
    0x14, 0x06, 0x03, 0x55, 0x1d, 0x11, 0x04, 0x0d, 0x30, 0x0b, 0x82, 0x09, 0x6c, 0x6f, 0x63, 0x61,
    0x6c, 0x68, 0x6f, 0x73, 0x74, 0x30, 0x0a, 0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x04, 0x03,
    0x02, 0x03, 0x47, 0x00, 0x30, 0x44, 0x02, 0x20, 0x6f, 0x34, 0x17, 0xfc, 0x5a, 0x21, 0xa5, 0xba,
    0xcc, 0x13, 0xe4, 0x04, 0xe9, 0xba, 0x34, 0x52, 0xb6, 0x1d, 0x5c, 0x8d, 0x61, 0x49, 0xa4, 0xac,
    0xf3, 0x28, 0xdd, 0x33, 0xb7, 0x6b, 0xe0, 0x7a, 0x02, 0x20, 0x43, 0x3e, 0x75, 0xba, 0x83, 0x1a,
    0x6f, 0x90, 0x1b, 0xeb, 0x64, 0x84, 0x7e, 0xe3, 0x07, 0x7e, 0x5c, 0xcd, 0xb0, 0x56, 0x98, 0x2f,
    0xd8, 0xe4, 0xe6, 0xcc, 0x33, 0x6f, 0x62, 0xfe, 0xed, 0xa1,
];

pub(crate) struct LiveTimeAuthorityHonesty {
    pub(crate) read_status: &'static str,
    pub(crate) clock: Option<CmosRtcWallClock>,
    pub(crate) decision: TimeAuthorityHonestyDecision,
}

pub(crate) fn live_time_authority_honesty() -> LiveTimeAuthorityHonesty {
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

    LiveTimeAuthorityHonesty {
        read_status,
        clock,
        decision,
    }
}

pub(crate) fn emit_system_time_authority(_request: &str) {
    let honesty = live_time_authority_honesty();
    let clock = honesty.clock;
    let decision = honesty.decision;

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
            f("read_status", s(honesty.read_status)),
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
            f(
                "real_cert_fixture",
                s("vendor_embedded_tls_localhost_server_cert_der_not_live"),
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
            f("real_cert_probe", real_cert_probe(read_status, now)),
            f("cases", cert_time_check_cases(now)),
        ],
        6,
    );
    end_response("system.cert_time_check_selftest");
}

fn real_cert_probe(read_status: &'static str, now: Option<CertValidityDateTime>) -> V<'static> {
    let cert_sha256 = sha256_bytes(REAL_TEST_CERT_DER);
    match parse_x509_cert_validity_window(REAL_TEST_CERT_DER) {
        Ok(window) => {
            let decision = now.map(|now| {
                evaluate_cert_validity_window_unverified_basis(
                    window.not_before,
                    now,
                    window.not_after,
                )
            });
            V::Object(vec![
                f(
                    "fixture_kind",
                    s("vendor_embedded_tls_localhost_server_cert_der_not_live"),
                ),
                f("parse_ok", b(true)),
                f("parse_error", n()),
                f("cert_sha256", sha(cert_sha256)),
                f("not_before", datetime_value(window.not_before)),
                f("not_after", datetime_value(window.not_after)),
                f("now_source", s("cmos_rtc_unverified")),
                f("now_read_status", s(read_status)),
                f("now", datetime_or_null(now)),
                f("basis", s("cmos_rtc_unverified")),
                f(
                    "basis_source",
                    s(decision
                        .map(|decision| decision.basis_source)
                        .unwrap_or("cmos_rtc_unverified")),
                ),
                f(
                    "status",
                    s(decision
                        .map(|decision| decision.status)
                        .unwrap_or("clock_read_failed_unverified_basis")),
                ),
                f("trusted", b(false)),
                f("source_verified", b(false)),
                f("validates_cert_time", b(false)),
                f("authorizes_provider_request", b(false)),
                f("authorizes_provider_export", b(false)),
                f("durable_write", b(false)),
                f("capability_granted", b(false)),
                f("provider_write", s("not_attempted")),
                f("transmission", b(false)),
                f("owner_sealed", b(false)),
                f("trust_tier", s("dev_key_not_owner_sealed")),
            ])
        }
        Err(error) => V::Object(vec![
            f(
                "fixture_kind",
                s("vendor_embedded_tls_localhost_server_cert_der_not_live"),
            ),
            f("parse_ok", b(false)),
            f("parse_error", s(error.reason())),
            f("cert_sha256", sha(cert_sha256)),
            f("not_before", n()),
            f("not_after", n()),
            f("now_source", s("cmos_rtc_unverified")),
            f("now_read_status", s(read_status)),
            f("now", datetime_or_null(now)),
            f("basis", s("cmos_rtc_unverified")),
            f("basis_source", s("cmos_rtc_unverified")),
            f("status", s("parse_failed")),
            f("trusted", b(false)),
            f("source_verified", b(false)),
            f("validates_cert_time", b(false)),
            f("authorizes_provider_request", b(false)),
            f("authorizes_provider_export", b(false)),
            f("durable_write", b(false)),
            f("capability_granted", b(false)),
            f("provider_write", s("not_attempted")),
            f("transmission", b(false)),
            f("owner_sealed", b(false)),
            f("trust_tier", s("dev_key_not_owner_sealed")),
        ]),
    }
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
