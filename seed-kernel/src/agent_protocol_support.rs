use core::fmt;

use crate::{event_log, serial};

pub(crate) fn method_eq(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

pub(crate) fn method_head_eq(left: &str, right: &str) -> bool {
    let left = left.trim();
    if left.len() < right.len() {
        return false;
    }
    let (head, rest) = left.split_at(right.len());
    method_eq(head, right) && (rest.is_empty() || rest.as_bytes()[0].is_ascii_whitespace())
}

pub(crate) fn parse_sha256_ref(value: &str) -> Option<[u8; 32]> {
    let mut value = value.trim();
    if value.len() >= 7 && value[..7].eq_ignore_ascii_case("sha256:") {
        value = &value[7..];
    }
    if value.len() != 64 {
        return None;
    }
    let bytes = value.as_bytes();
    let mut out = [0u8; 32];
    let mut idx = 0usize;
    while idx < out.len() {
        let high = hex_value(bytes[idx * 2])?;
        let low = hex_value(bytes[idx * 2 + 1])?;
        out[idx] = (high << 4) | low;
        idx += 1;
    }
    Some(out)
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

pub(crate) fn current_boot_event_id_str(value: &str) -> bool {
    parse_current_boot_event_id(value).is_some()
}

pub(crate) fn parse_current_boot_event_id(value: &str) -> Option<event_log::EventId> {
    let sequence = value.strip_prefix("event.current_boot.")?;
    if sequence.len() != 8 || !sequence.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let mut parsed = 0u64;
    for byte in sequence.bytes() {
        parsed = parsed
            .saturating_mul(10)
            .saturating_add((byte - b'0') as u64);
    }
    event_log::EventId::from_sequence(parsed)
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
