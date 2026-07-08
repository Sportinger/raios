use alloc::vec;

use crate::{
    agent_protocol_support::{
        begin_response, emit_record_fields, end_response, record_bool as b, record_field as f,
        record_null as n, record_str as s, record_str_or_null as opt_s,
    },
    time::{read_cmos_rtc_wall_clock, CmosRtcError, CmosRtcWallClock},
};
use raios_core::{
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

fn clock_u8(clock: Option<CmosRtcWallClock>, value: fn(CmosRtcWallClock) -> u8) -> V<'static> {
    match clock {
        Some(clock) => V::U64(value(clock) as u64),
        None => n(),
    }
}
