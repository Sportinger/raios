use alloc::{vec, vec::Vec};
use core::fmt;

use crate::{event_log, serial};
use raios_core::{
    record::{write_json, write_json_fields, Field, Value},
    ByteSink,
};

pub(crate) use raios_core::{method_eq, method_head_eq, parse_sha256_ref};

pub(crate) struct SerialSink;

impl raios_core::ByteSink for SerialSink {
    fn write_bytes(&mut self, bytes: &[u8]) {
        if let Ok(text) = core::str::from_utf8(bytes) {
            serial::write_raw_str(text);
        } else {
            let mut idx = 0usize;
            while idx < bytes.len() {
                serial::write_byte(bytes[idx]);
                idx += 1;
            }
        }
    }
}

pub(crate) fn record_str_or_null<'a>(value: Option<&'a str>) -> Value<'a> {
    match value {
        Some(value) => Value::Str(value),
        None => Value::Null,
    }
}

pub(crate) fn record_field<'a>(key: &'a str, value: Value<'a>) -> Field<'a> {
    Field::new(key, value)
}

pub(crate) fn record_str(value: &str) -> Value<'_> {
    Value::Str(value)
}

pub(crate) fn record_bool(value: bool) -> Value<'static> {
    Value::Bool(value)
}

pub(crate) fn record_false() -> Value<'static> {
    Value::Bool(false)
}

pub(crate) fn record_null() -> Value<'static> {
    Value::Null
}

pub(crate) fn record_event(value: event_log::EventId) -> Value<'static> {
    Value::EventSequence(value.sequence())
}

pub(crate) fn record_sha(value: [u8; 32]) -> Value<'static> {
    Value::Sha256(value)
}

pub(crate) fn record_inline<'a>(fields: Vec<Field<'a>>) -> Value<'a> {
    Value::InlineObject(fields)
}

pub(crate) fn record_sha_fields<'a>(pairs: &[(&'a str, [u8; 32])]) -> Vec<Field<'a>> {
    let mut fields = Vec::new();
    let mut idx = 0usize;
    while idx < pairs.len() {
        fields.push(record_field(pairs[idx].0, record_sha(pairs[idx].1)));
        idx += 1;
    }
    fields
}

pub(crate) fn extend_false_fields<'a>(fields: &mut Vec<Field<'a>>, names: &'a [&'a str]) {
    let mut idx = 0usize;
    while idx < names.len() {
        fields.push(record_field(names[idx], record_false()));
        idx += 1;
    }
}

pub(crate) fn record_non_authorizing_fact_fields<'a>(
    schema: &'a str,
    present: bool,
    missing_reason: &'a str,
    false_fields: &'a [&'a str],
) -> Vec<Field<'a>> {
    let mut fields = vec![
        record_field("schema", record_str(schema)),
        record_field(
            "status",
            record_str(if present { "present" } else { "missing" }),
        ),
        record_field("event_id", record_null()),
        record_field("retained", record_false()),
        record_field("required", record_bool(true)),
        record_field("scope", record_str("current_boot")),
        record_field("classification", record_str("local_only")),
        record_field(
            "reason",
            record_str(if present {
                "current_boot_fact_available"
            } else {
                missing_reason
            }),
        ),
    ];
    extend_false_fields(&mut fields, false_fields);
    fields.push(record_field("service_inventory_change", record_str("none")));
    fields.push(record_field("load_attempted", record_false()));
    fields
}

pub(crate) fn record_object<'a>(fields: Vec<Field<'a>>) -> Value<'a> {
    Value::Object(fields)
}

pub(crate) fn record_sha_or_null(value: Option<[u8; 32]>) -> Value<'static> {
    match value {
        Some(value) => Value::Sha256(value),
        None => Value::Null,
    }
}

pub(crate) fn record_event_or_null(value: Option<event_log::EventId>) -> Value<'static> {
    match value {
        Some(value) => Value::EventSequence(value.sequence()),
        None => Value::Null,
    }
}

pub(crate) fn emit_record_property(name: &str, fields: Vec<Field<'_>>) {
    let mut sink = SerialSink;

    sink.write_bytes(b"      ");
    write_json(&Value::Str(name), &mut sink, 6);
    sink.write_bytes(b": ");
    write_json(&Value::Object(fields), &mut sink, 6);
}

pub(crate) fn emit_record_property_line(name: &str, fields: Vec<Field<'_>>, comma: bool) {
    emit_record_property(name, fields);
    raw_line(if comma { "," } else { "" });
}

pub(crate) fn emit_inline_record_property(name: &str, fields: Vec<Field<'_>>, comma: bool) {
    emit_inline_record_property_at(name, fields, 6, comma);
}

pub(crate) fn emit_inline_record_property_at(
    name: &str,
    fields: Vec<Field<'_>>,
    spaces: usize,
    comma: bool,
) {
    let mut sink = SerialSink;
    let mut idx = 0usize;

    while idx < spaces {
        sink.write_bytes(b" ");
        idx += 1;
    }
    write_json(&Value::Str(name), &mut sink, 6);
    sink.write_bytes(b": ");
    write_json(&Value::InlineObject(fields), &mut sink, spaces);
    if comma {
        sink.write_bytes(b",");
    }
    sink.write_bytes(b"\r\n");
}

pub(crate) fn emit_record_fields(fields: Vec<Field<'_>>, spaces: usize) {
    let mut sink = SerialSink;

    write_json_fields(&fields, &mut sink, spaces);
}

pub(crate) fn emit_record_fields_trailing_comma(fields: Vec<Field<'_>>, spaces: usize) {
    let mut sink = SerialSink;
    let mut idx = 0usize;

    while idx < fields.len() {
        let mut written = 0usize;
        while written < spaces {
            sink.write_bytes(b" ");
            written += 1;
        }
        write_json(&Value::Str(fields[idx].key), &mut sink, spaces);
        sink.write_bytes(b": ");
        write_json(&fields[idx].value, &mut sink, spaces);
        sink.write_bytes(b",\r\n");
        idx += 1;
    }
}

pub(crate) fn emit_record_object(fields: Vec<Field<'_>>, spaces: usize, comma: bool) {
    let mut sink = SerialSink;
    let mut idx = 0usize;
    while idx < spaces {
        sink.write_bytes(b" ");
        idx += 1;
    }
    write_json(&Value::Object(fields), &mut sink, spaces);
    if comma {
        sink.write_bytes(b",");
    }
    sink.write_bytes(b"\r\n");
}

pub(crate) fn emit_inline_record_object_fragment(fields: Vec<Field<'_>>, spaces: usize) {
    let mut sink = SerialSink;
    let mut idx = 0usize;
    while idx < spaces {
        sink.write_bytes(b" ");
        idx += 1;
    }
    write_json(&Value::InlineObject(fields), &mut sink, spaces);
}

pub(crate) fn emit_inline_record_object(fields: Vec<Field<'_>>, comma: bool) {
    emit_inline_record_object_fragment(fields, 8);
    if comma {
        let mut sink = SerialSink;
        sink.write_bytes(b",");
    }
    let mut sink = SerialSink;
    sink.write_bytes(b"\r\n");
}

pub(crate) fn current_boot_event_id_str(value: &str) -> bool {
    parse_current_boot_event_id(value).is_some()
}

pub(crate) fn parse_current_boot_event_id(value: &str) -> Option<event_log::EventId> {
    event_log::EventId::from_sequence(raios_core::parse_current_boot_event_sequence(value)?)
}

pub(crate) fn json_event_id(event_id: event_log::EventId) {
    json_event_sequence(event_id.sequence());
}

pub(crate) fn json_event_id_option(event_id: Option<event_log::EventId>) {
    if let Some(event_id) = event_id {
        json_event_id(event_id);
    } else {
        raw("null");
    }
}

pub(crate) fn json_current_boot_id(prefix: &'static str, event_id: event_log::EventId) {
    raw("\"");
    raw(prefix);
    raw(".");
    raw_fmt(format_args!("{:08}", event_id.sequence()));
    raw("\"");
}

pub(crate) fn json_event_sequence(sequence: u64) {
    raw("\"event.current_boot.");
    raw_fmt(format_args!("{:08}", sequence));
    raw("\"");
}

pub(crate) fn json_sha256(hash: [u8; 32]) {
    raw("\"sha256:");
    let mut idx = 0usize;
    while idx < hash.len() {
        raw_fmt(format_args!("{:02x}", hash[idx]));
        idx += 1;
    }
    raw("\"");
}

pub(crate) fn json_sha256_option(hash: Option<[u8; 32]>) {
    if let Some(hash) = hash {
        json_sha256(hash);
    } else {
        raw("null");
    }
}

pub(crate) fn raw(value: &str) {
    serial::write_raw_str(value);
}

pub(crate) fn raw_line(value: &str) {
    serial::write_raw_line(value);
}

pub(crate) fn raw_fmt(args: fmt::Arguments<'_>) {
    serial::write_raw_fmt(args);
}

pub(crate) fn raw_bool(value: bool) {
    raw(if value { "true" } else { "false" });
}

pub(crate) fn crlf() {
    raw("\r\n");
}

pub(crate) fn indent(spaces: usize) {
    let mut idx = 0usize;
    while idx < spaces {
        raw(" ");
        idx += 1;
    }
}

pub(crate) fn json_str(value: &str) {
    raw("\"");
    for byte in value.bytes() {
        match byte {
            b'"' => raw("\\\""),
            b'\\' => raw("\\\\"),
            b'\n' => raw("\\n"),
            b'\r' => raw("\\r"),
            b'\t' => raw("\\t"),
            0x20..=0x7e => serial::write_byte(byte),
            _ => raw(" "),
        }
    }
    raw("\"");
}

pub(crate) fn json_opt_str(value: Option<&str>) {
    match value {
        Some(value) => json_str(value),
        None => raw("null"),
    }
}

pub(crate) fn emit_static_string_array(values: &[&str], spaces: usize) {
    let mut idx = 0usize;
    while idx < values.len() {
        indent(spaces);
        json_str(values[idx]);
        if idx + 1 != values.len() {
            raw(",");
        }
        crlf();
        idx += 1;
    }
}

pub(crate) fn emit_inline_string_array(values: &[&str]) {
    let mut idx = 0usize;
    while idx < values.len() {
        if idx != 0 {
            raw(", ");
        }
        json_str(values[idx]);
        idx += 1;
    }
}

pub(crate) fn emit_export_gate(
    wrote: &mut bool,
    gate: &'static str,
    state: &'static str,
    reason: &'static str,
) {
    if *wrote {
        raw_line(",");
    }
    emit_inline_record_object_fragment(
        vec![
            record_field("gate", record_str(gate)),
            record_field("state", record_str(state)),
            record_field("reason", record_str(reason)),
        ],
        6,
    );
    *wrote = true;
}

pub(crate) fn begin_response(method: &'static str) {
    raw_fmt(format_args!("RAIOS_AGENT_BEGIN {}\r\n", method));
    raw_line("{");
    raw_line("  \"v\": \"raios.agent.v0\",");
    raw_line("  \"t\": \"response\",");
    raw_line("  \"id\": \"serial\",");
    raw_line("  \"body\": {");
    raw("    \"method\": ");
    json_str(method);
    raw_line(",");
    raw_line("    \"result\": {");
}

pub(crate) fn end_response(method: &'static str) {
    raw_line("    }");
    raw_line("  }");
    raw_line("}");
    raw_fmt(format_args!("RAIOS_AGENT_END {}\r\n", method));
}
