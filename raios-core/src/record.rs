//! Typed record values for raiOS JSON.
//!
//! The crate stays `no_std` outside tests, but nested record values need owned
//! child lists. Arrays and objects therefore use `alloc::vec::Vec`, while keys
//! and string values stay borrowed to avoid cloning kernel literals.

use alloc::vec::Vec;
use sha2::{Digest, Sha256};

use crate::{sha256_hex, ByteSink};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value<'a> {
    Null,
    Bool(bool),
    U64(u64),
    Str(&'a str),
    HexBytes(&'a [u8]),
    TrimmedAsciiBytes(&'a [u8]),
    Sha256([u8; 32]),
    EventSequence(u64),
    ResponseSequence(u64),
    InlineArray(Vec<Value<'a>>),
    Array(Vec<Value<'a>>),
    InlineObject(Vec<Field<'a>>),
    Object(Vec<Field<'a>>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field<'a> {
    pub key: &'a str,
    pub value: Value<'a>,
}

impl<'a> Field<'a> {
    pub const fn new(key: &'a str, value: Value<'a>) -> Self {
        Self { key, value }
    }
}

pub fn write_json<S: ByteSink>(value: &Value<'_>, sink: &mut S, indent: usize) {
    match value {
        Value::Null => sink.write_bytes(b"null"),
        Value::Bool(value) => sink.write_bytes(if *value { b"true" } else { b"false" }),
        Value::U64(value) => write_u64(*value, sink),
        Value::Str(value) => write_json_str(value, sink),
        Value::HexBytes(value) => write_hex_bytes(value, sink),
        Value::TrimmedAsciiBytes(value) => write_trimmed_ascii_bytes(value, sink),
        Value::Sha256(value) => write_sha256(*value, sink),
        Value::EventSequence(value) => write_event_sequence(*value, sink),
        Value::ResponseSequence(value) => write_response_sequence(*value, sink),
        Value::InlineArray(values) => write_inline_array(values, sink, indent),
        Value::Array(values) => write_array(values, sink, indent),
        Value::InlineObject(fields) => write_inline_object(fields, sink, indent),
        Value::Object(fields) => write_object(fields, sink, indent),
    }
}

pub fn write_json_fields<S: ByteSink>(fields: &[Field<'_>], sink: &mut S, indent: usize) {
    let mut idx = 0usize;
    while idx < fields.len() {
        write_indent(sink, indent);
        write_json_str(fields[idx].key, sink);
        sink.write_bytes(b": ");
        write_json(&fields[idx].value, sink, indent);
        if idx + 1 != fields.len() {
            sink.write_bytes(b",");
        }
        sink.write_bytes(b"\r\n");
        idx += 1;
    }
}

pub struct InlineObjectWriter<'a, S: ByteSink> {
    sink: &'a mut S,
    first: bool,
    indent: usize,
}

impl<'a, S: ByteSink> InlineObjectWriter<'a, S> {
    pub fn new(sink: &'a mut S, indent: usize) -> Self {
        sink.write_bytes(b"{");
        Self {
            sink,
            first: true,
            indent,
        }
    }

    pub fn field(&mut self, key: &str, value: &Value<'_>) {
        let indent = self.indent;
        self.field_with(key, |sink| write_json(value, sink, indent));
    }

    pub fn field_with(&mut self, key: &str, write_value: impl FnOnce(&mut S)) {
        if self.first {
            self.first = false;
        } else {
            self.sink.write_bytes(b", ");
        }
        write_json_str(key, self.sink);
        self.sink.write_bytes(b": ");
        write_value(self.sink);
    }

    pub fn finish(self) {
        self.sink.write_bytes(b"}");
    }
}

pub fn sha256_of_json(value: &Value<'_>) -> [u8; 32] {
    let mut sink = Sha256Sink(Sha256::new());
    write_json(value, &mut sink, 0);
    let digest = sink.0.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

struct Sha256Sink(Sha256);

impl ByteSink for Sha256Sink {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
}

fn write_array<S: ByteSink>(values: &[Value<'_>], sink: &mut S, indent: usize) {
    sink.write_bytes(b"[");
    if values.is_empty() {
        sink.write_bytes(b"]");
        return;
    }

    sink.write_bytes(b"\r\n");
    let mut idx = 0usize;
    while idx < values.len() {
        write_indent(sink, indent + 2);
        write_json(&values[idx], sink, indent + 2);
        if idx + 1 != values.len() {
            sink.write_bytes(b",");
        }
        sink.write_bytes(b"\r\n");
        idx += 1;
    }
    write_indent(sink, indent);
    sink.write_bytes(b"]");
}

fn write_inline_array<S: ByteSink>(values: &[Value<'_>], sink: &mut S, indent: usize) {
    sink.write_bytes(b"[");
    let mut idx = 0usize;
    while idx < values.len() {
        if idx != 0 {
            sink.write_bytes(b", ");
        }
        write_json(&values[idx], sink, indent);
        idx += 1;
    }
    sink.write_bytes(b"]");
}

fn write_inline_object<S: ByteSink>(fields: &[Field<'_>], sink: &mut S, indent: usize) {
    sink.write_bytes(b"{");
    let mut idx = 0usize;
    while idx < fields.len() {
        if idx != 0 {
            sink.write_bytes(b", ");
        }
        write_json_str(fields[idx].key, sink);
        sink.write_bytes(b": ");
        write_json(&fields[idx].value, sink, indent);
        idx += 1;
    }
    sink.write_bytes(b"}");
}

fn write_object<S: ByteSink>(fields: &[Field<'_>], sink: &mut S, indent: usize) {
    sink.write_bytes(b"{");
    if fields.is_empty() {
        sink.write_bytes(b"}");
        return;
    }

    sink.write_bytes(b"\r\n");
    let mut idx = 0usize;
    while idx < fields.len() {
        write_indent(sink, indent + 2);
        write_json_str(fields[idx].key, sink);
        sink.write_bytes(b": ");
        write_json(&fields[idx].value, sink, indent + 2);
        if idx + 1 != fields.len() {
            sink.write_bytes(b",");
        }
        sink.write_bytes(b"\r\n");
        idx += 1;
    }
    write_indent(sink, indent);
    sink.write_bytes(b"}");
}

fn write_json_str<S: ByteSink>(value: &str, sink: &mut S) {
    sink.write_bytes(b"\"");
    for byte in value.bytes() {
        match byte {
            b'"' => sink.write_bytes(b"\\\""),
            b'\\' => sink.write_bytes(b"\\\\"),
            b'\n' => sink.write_bytes(b"\\n"),
            b'\r' => sink.write_bytes(b"\\r"),
            b'\t' => sink.write_bytes(b"\\t"),
            0x20..=0x7e => sink.write_bytes(&[byte]),
            _ => sink.write_bytes(b" "),
        }
    }
    sink.write_bytes(b"\"");
}

fn write_hex_bytes<S: ByteSink>(value: &[u8], sink: &mut S) {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    sink.write_bytes(b"\"");
    for byte in value {
        sink.write_bytes(&[HEX[(byte >> 4) as usize], HEX[(byte & 0x0f) as usize]]);
    }
    sink.write_bytes(b"\"");
}

fn write_trimmed_ascii_bytes<S: ByteSink>(value: &[u8], sink: &mut S) {
    let mut start = 0usize;
    let mut end = value.len();
    while start < end && (value[start] == 0 || value[start] == b' ') {
        start += 1;
    }
    while end > start && (value[end - 1] == 0 || value[end - 1] == b' ') {
        end -= 1;
    }

    sink.write_bytes(b"\"");
    let mut idx = start;
    while idx < end {
        match value[idx] {
            b'"' => sink.write_bytes(b"\\\""),
            b'\\' => sink.write_bytes(b"\\\\"),
            0x20..=0x7e => sink.write_bytes(&[value[idx]]),
            _ => sink.write_bytes(b" "),
        }
        idx += 1;
    }
    sink.write_bytes(b"\"");
}

fn write_sha256<S: ByteSink>(value: [u8; 32], sink: &mut S) {
    let hex = sha256_hex(&value);
    sink.write_bytes(b"\"sha256:");
    sink.write_bytes(&hex);
    sink.write_bytes(b"\"");
}

fn write_event_sequence<S: ByteSink>(value: u64, sink: &mut S) {
    sink.write_bytes(b"\"event.current_boot.");
    write_u64_padded(value, 8, sink);
    sink.write_bytes(b"\"");
}

fn write_response_sequence<S: ByteSink>(value: u64, sink: &mut S) {
    sink.write_bytes(b"\"response.current_boot.");
    write_u64_padded(value, 8, sink);
    sink.write_bytes(b"\"");
}

fn write_u64<S: ByteSink>(value: u64, sink: &mut S) {
    write_u64_padded(value, 0, sink);
}

fn write_u64_padded<S: ByteSink>(mut value: u64, min_width: usize, sink: &mut S) {
    let mut bytes = [b'0'; 20];
    let mut start = bytes.len();

    loop {
        start -= 1;
        bytes[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }

    let digits = bytes.len() - start;
    let mut padding = min_width.saturating_sub(digits);
    while padding > 0 {
        sink.write_bytes(b"0");
        padding -= 1;
    }
    sink.write_bytes(&bytes[start..]);
}

fn write_indent<S: ByteSink>(sink: &mut S, spaces: usize) {
    let mut idx = 0usize;
    while idx < spaces {
        sink.write_bytes(b" ");
        idx += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{sha256_of_json, write_json, Field, InlineObjectWriter, Value};
    use crate::sha256_bytes;

    fn render(value: &Value<'_>) -> std::vec::Vec<u8> {
        let mut out = std::vec::Vec::new();
        write_json(value, &mut out, 0);
        out
    }

    #[test]
    fn escaping_matches_kernel_truth_table() {
        assert_eq!(render(&Value::Str("\"")), br#""\"""#);
        assert_eq!(render(&Value::Str("\\")), br#""\\""#);
        assert_eq!(render(&Value::Str("\n")), br#""\n""#);
        assert_eq!(render(&Value::Str("\r")), br#""\r""#);
        assert_eq!(render(&Value::Str("\t")), br#""\t""#);
        assert_eq!(render(&Value::Str("\u{80}")), b"\"  \"");
    }

    #[test]
    fn empty_array_and_object_render_closed() {
        assert_eq!(render(&Value::Array(vec![])), b"[]");
        assert_eq!(render(&Value::Object(vec![])), b"{}");
    }

    #[test]
    fn inline_object_renders_compact_nested_fields() {
        let digest = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
            0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67,
            0x89, 0xab, 0xcd, 0xef,
        ];
        let value = Value::InlineObject(vec![
            Field::new("schema", Value::Str("raios.test.v0")),
            Field::new(
                "hashes",
                Value::InlineObject(vec![Field::new("artifact_hash", Value::Sha256(digest))]),
            ),
            Field::new("load_attempted", Value::Bool(false)),
        ]);

        assert_eq!(
            render(&value),
            b"{\"schema\": \"raios.test.v0\", \"hashes\": {\"artifact_hash\": \"sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\"}, \"load_attempted\": false}"
        );
    }

    #[test]
    fn inline_array_renders_single_line_values() {
        assert_eq!(
            render(&Value::InlineArray(vec![
                Value::Bool(true),
                Value::Null,
                Value::EventSequence(7),
            ])),
            b"[true, null, \"event.current_boot.00000007\"]"
        );
    }

    #[test]
    fn field_list_renders_inside_open_object() {
        let mut out = std::vec::Vec::new();
        super::write_json_fields(
            &[
                Field::new("schema", Value::Str("raios.test.v0")),
                Field::new("ready", Value::Bool(false)),
            ],
            &mut out,
            4,
        );

        assert_eq!(
            out,
            b"    \"schema\": \"raios.test.v0\",\r\n    \"ready\": false\r\n"
        );
    }

    #[test]
    fn nested_object_and_array_render_with_crlf_indentation() {
        let value = Value::Object(vec![
            Field::new("schema", Value::Str("raios.test.v0")),
            Field::new(
                "items",
                Value::Array(vec![
                    Value::Str("one"),
                    Value::Object(vec![
                        Field::new("ok", Value::Bool(true)),
                        Field::new("count", Value::U64(7)),
                    ]),
                ]),
            ),
            Field::new("nothing", Value::Null),
        ]);

        assert_eq!(
            render(&value),
            b"{\r\n  \"schema\": \"raios.test.v0\",\r\n  \"items\": [\r\n    \"one\",\r\n    {\r\n      \"ok\": true,\r\n      \"count\": 7\r\n    }\r\n  ],\r\n  \"nothing\": null\r\n}"
        );
    }

    #[test]
    fn sha256_of_json_hashes_rendered_bytes() {
        let value = Value::Object(vec![Field::new("answer", Value::U64(42))]);
        let rendered = render(&value);

        assert_eq!(sha256_of_json(&value), sha256_bytes(&rendered));
    }

    #[test]
    fn sha256_and_event_sequence_render_as_kernel_forms() {
        let digest = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
            0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67,
            0x89, 0xab, 0xcd, 0xef,
        ];

        assert_eq!(
            render(&Value::Sha256(digest)),
            b"\"sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\""
        );
        assert_eq!(
            render(&Value::EventSequence(42)),
            b"\"event.current_boot.00000042\""
        );
        assert_eq!(
            render(&Value::ResponseSequence(42)),
            b"\"response.current_boot.00000042\""
        );
    }

    #[test]
    fn byte_string_forms_match_kernel_rendering() {
        assert_eq!(
            render(&Value::HexBytes(&[0x00, 0xab, 0xcd, 0xef])),
            b"\"00abcdef\""
        );
        assert_eq!(
            render(&Value::TrimmedAsciiBytes(b"\0  QEMU \\\"disk\x01  \0")),
            b"\"QEMU \\\\\\\"disk \""
        );
    }

    #[test]
    fn inline_object_writer_streams_same_inline_shape() {
        let mut out = std::vec::Vec::new();
        let mut object = InlineObjectWriter::new(&mut out, 0);
        object.field("schema", &Value::Str("raios.test.v0"));
        object.field("count", &Value::U64(3));
        object.field_with("nested", |sink| {
            let mut nested = InlineObjectWriter::new(sink, 0);
            nested.field("ok", &Value::Bool(true));
            nested.field(
                "items",
                &Value::InlineArray(vec![Value::Str("a"), Value::Null]),
            );
            nested.finish();
        });
        object.finish();

        assert_eq!(
            out,
            b"{\"schema\": \"raios.test.v0\", \"count\": 3, \"nested\": {\"ok\": true, \"items\": [\"a\", null]}}"
        );
    }
}
