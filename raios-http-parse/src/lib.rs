#![cfg_attr(not(test), no_std)]

//! Non-allocating HTTP/1.1 response-header parsers relocated verbatim from
//! seed-kernel/src/openai.rs (M11-7 Phase A). seed-kernel (via
//! raios_core::http_response_parse) and the svc.demo.httphead Wasm guest compile
//! this identical source: the guest is evidence of faithful execution, the kernel
//! core stays authority. Behavior is byte-identical to the pre-move kernel - no
//! bug is fixed here (see parse_content_length note).

use core::str;

// ---- relocated verbatim from openai.rs (do NOT change behavior) ----

pub fn http_response_complete(response: &[u8]) -> bool {
    let Some(body_start) = find_subslice(response, b"\r\n\r\n").map(|index| index + 4) else {
        return false;
    };

    if header_contains(response, b"transfer-encoding:", b"chunked") {
        return find_subslice(&response[body_start..], b"\r\n0\r\n").is_some();
    }

    if let Some(content_len) = parse_content_length(response) {
        return response.len().saturating_sub(body_start) >= content_len;
    }

    false
}

pub fn parse_status(response: &[u8]) -> Option<u16> {
    let line_end = find_subslice(response, b"\r\n")?;
    let line = str::from_utf8(&response[..line_end]).ok()?;
    let mut parts = line.split_ascii_whitespace();
    let _http = parts.next()?;
    parts.next()?.parse().ok()
}

// PRESERVED BEHAVIOR - do NOT "fix" here. The first header line is the status
// line ("HTTP/1.1 200 OK") which has no colon, so split_header(line)? returns
// None and `?` short-circuits this whole function to None for ANY response.
// Content-Length is never parsed via this path; production completion rides on
// the chunked branch. A fix + its own VM profile is a follow-up slice.
pub fn parse_content_length(response: &[u8]) -> Option<usize> {
    let header_end = find_subslice(response, b"\r\n\r\n")?;
    for line in response[..header_end].split(|byte| *byte == b'\n') {
        let line = trim_ascii(line.strip_suffix(b"\r").unwrap_or(line));
        let (name, value) = split_header(line)?;
        if eq_ignore_ascii_case(name, b"content-length") {
            return parse_usize(trim_ascii(value));
        }
    }
    None
}

pub fn header_contains(response: &[u8], name_prefix: &[u8], value: &[u8]) -> bool {
    let Some(header_end) = find_subslice(response, b"\r\n\r\n") else {
        return false;
    };
    for line in response[..header_end].split(|byte| *byte == b'\n') {
        let line = trim_ascii(line.strip_suffix(b"\r").unwrap_or(line));
        if line.len() >= name_prefix.len()
            && eq_ignore_ascii_case(&line[..name_prefix.len()], name_prefix)
            && contains_ignore_ascii_case(line, value)
        {
            return true;
        }
    }
    false
}

fn split_header(line: &[u8]) -> Option<(&[u8], &[u8])> {
    let colon = line.iter().position(|byte| *byte == b':')?;
    Some((&line[..colon], &line[colon + 1..]))
}

pub fn trim_ascii(mut value: &[u8]) -> &[u8] {
    while value.first().is_some_and(|byte| byte.is_ascii_whitespace()) {
        value = &value[1..];
    }
    while value.last().is_some_and(|byte| byte.is_ascii_whitespace()) {
        value = &value[..value.len() - 1];
    }
    value
}

fn parse_usize(value: &[u8]) -> Option<usize> {
    let mut out = 0usize;
    for &byte in value {
        if !byte.is_ascii_digit() {
            return None;
        }
        out = out.checked_mul(10)?.checked_add((byte - b'0') as usize)?;
    }
    Some(out)
}

pub fn parse_hex_usize(value: &[u8]) -> Option<usize> {
    let mut out = 0usize;
    for &byte in value {
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            b';' => break,
            _ => return None,
        };
        out = out.checked_mul(16)?.checked_add(digit as usize)?;
    }
    Some(out)
}

pub fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn contains_ignore_ascii_case(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| eq_ignore_ascii_case(window, needle))
}

fn eq_ignore_ascii_case(lhs: &[u8], rhs: &[u8]) -> bool {
    lhs.len() == rhs.len()
        && lhs
            .iter()
            .zip(rhs)
            .all(|(left, right)| left.eq_ignore_ascii_case(right))
}

// ---- HTTP-head facts + fixed-length fail-closed wire record (mirrors CERTWINDOW) ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HttpHeadFacts {
    pub status_code: Option<u16>,
    pub content_length: Option<u64>,
    pub response_complete: bool,
    pub chunked: bool,
}

/// Single entry point both guest and kernel call. content_length is widened to
/// u64 so the encoded record is byte-identical on wasm32 (usize=u32) and
/// x86_64 (usize=u64).
pub fn parse_http_head(response: &[u8]) -> HttpHeadFacts {
    HttpHeadFacts {
        status_code: parse_status(response),
        content_length: parse_content_length(response).map(|value| value as u64),
        response_complete: http_response_complete(response),
        chunked: header_contains(response, b"transfer-encoding:", b"chunked"),
    }
}

pub const HTTPHEAD_RECORD_LEN: usize = 16;
pub const HTTPHEAD_RECORD_MAGIC: u8 = 0x48; // 'H', distinct from certwindow 0xC7
pub const HTTPHEAD_RECORD_VERSION: u8 = 1;

/// Fixed 16-byte record, big-endian. Always 16 bytes.
/// [0]=magic 0x48 [1]=version 1
/// [2]=status_present(0/1) [3..5]=status_code u16 BE (0 when absent)
/// [5]=content_length_present(0/1) [6..14]=content_length u64 BE (0 when absent)
/// [14]=response_complete(0/1) [15]=chunked(0/1)
pub fn encode_httphead_record(facts: HttpHeadFacts) -> [u8; HTTPHEAD_RECORD_LEN] {
    let mut b = [0u8; HTTPHEAD_RECORD_LEN];
    b[0] = HTTPHEAD_RECORD_MAGIC;
    b[1] = HTTPHEAD_RECORD_VERSION;
    if let Some(code) = facts.status_code {
        b[2] = 1;
        let c = code.to_be_bytes();
        b[3] = c[0];
        b[4] = c[1];
    }
    if let Some(len) = facts.content_length {
        b[5] = 1;
        b[6..14].copy_from_slice(&len.to_be_bytes());
    }
    b[14] = facts.response_complete as u8;
    b[15] = facts.chunked as u8;
    b
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodedHttpHeadRecord {
    pub record_valid: bool,
    pub status_present: bool,
    pub status_code: u16,
    pub content_length_present: bool,
    pub content_length: u64,
    pub response_complete: bool,
    pub chunked: bool,
}

/// Fail-closed decode. Length/magic/version guarded FIRST, so an empty or short
/// buffer yields record_valid=false instead of a panic.
pub fn decode_httphead_record(bytes: &[u8]) -> DecodedHttpHeadRecord {
    if bytes.len() != HTTPHEAD_RECORD_LEN
        || bytes[0] != HTTPHEAD_RECORD_MAGIC
        || bytes[1] != HTTPHEAD_RECORD_VERSION
    {
        return DecodedHttpHeadRecord {
            record_valid: false,
            status_present: false,
            status_code: 0,
            content_length_present: false,
            content_length: 0,
            response_complete: false,
            chunked: false,
        };
    }
    // len==16 guaranteed by the guard above, so try_into().unwrap() cannot panic.
    DecodedHttpHeadRecord {
        record_valid: true,
        status_present: bytes[2] != 0,
        status_code: u16::from_be_bytes([bytes[3], bytes[4]]),
        content_length_present: bytes[5] != 0,
        content_length: u64::from_be_bytes(bytes[6..14].try_into().unwrap()),
        response_complete: bytes[14] != 0,
        chunked: bytes[15] != 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sha256_hex(bytes: &[u8]) -> [u8; 64] {
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest(bytes);
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = [0u8; 64];
        for (i, byte) in digest.iter().enumerate() {
            out[i * 2] = HEX[(byte >> 4) as usize];
            out[i * 2 + 1] = HEX[(byte & 0x0f) as usize];
        }
        out
    }

    // Provenance-pinned fixtures - byte-identical to the kernel emitter's (Phase B).
    const HTTP_CHUNKED: &[u8] =
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nhi\r\n0\r\n\r\n"; // 59
    const HTTP_CONTENT_LENGTH: &[u8] =
        b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 11\r\nConnection: close\r\n\r\n{\"ok\":true}"; // 101

    #[test]
    fn fixture_bytes_match_pinned_provenance() {
        assert_eq!(HTTP_CHUNKED.len(), 59);
        assert_eq!(
            &sha256_hex(HTTP_CHUNKED),
            b"f49383a81b679f0569a28430340963e08c898b581b9f910be129ccfe60e54f9a"
        );
        assert_eq!(HTTP_CHUNKED[..54].len(), 54);
        assert_eq!(
            &sha256_hex(&HTTP_CHUNKED[..54]),
            b"ec11883d03c5aa89eeb04064577993e812b53441349fac620d99c4497806bbcf"
        );
        assert_eq!(HTTP_CONTENT_LENGTH.len(), 101);
        assert_eq!(
            &sha256_hex(HTTP_CONTENT_LENGTH),
            b"9d467b71e028580c15a68e9bd3aa9dba44dc0055ff11eb71123b05dfc648fa9c"
        );
    }

    #[test]
    fn chunked_response_facts() {
        let f = parse_http_head(HTTP_CHUNKED);
        assert_eq!(f.status_code, Some(200));
        assert_eq!(f.content_length, None);
        assert!(f.response_complete);
        assert!(f.chunked);
    }

    #[test]
    fn truncated_chunked_is_not_complete() {
        let f = parse_http_head(&HTTP_CHUNKED[..54]);
        assert_eq!(f.status_code, Some(200));
        assert_eq!(f.content_length, None);
        assert!(!f.response_complete); // terminal "\r\n0\r\n" chunk dropped
        assert!(f.chunked);
    }

    #[test]
    fn content_length_response_short_circuits_to_absent() {
        // PRESERVED upstream behavior: the status line has no colon, so
        // parse_content_length short-circuits to None even though the response
        // literally contains "Content-Length: 11". Do NOT "fix" this here.
        let f = parse_http_head(HTTP_CONTENT_LENGTH);
        assert_eq!(f.status_code, Some(200));
        assert_eq!(f.content_length, None);
        assert!(!f.response_complete);
        assert!(!f.chunked);
    }

    #[test]
    fn parse_status_edges() {
        assert_eq!(parse_status(b""), None);
        assert_eq!(parse_status(b"HTTP/1.1 200 OK\r\n"), Some(200));
        assert!(header_contains(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n",
            b"transfer-encoding:",
            b"chunked"
        ));
        assert!(!header_contains(
            b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n",
            b"transfer-encoding:",
            b"chunked"
        ));
    }

    #[test]
    fn record_round_trips_each_fixture() {
        for fixture in [HTTP_CHUNKED, &HTTP_CHUNKED[..54], HTTP_CONTENT_LENGTH] {
            let facts = parse_http_head(fixture);
            let decoded = decode_httphead_record(&encode_httphead_record(facts));
            assert!(decoded.record_valid);
            assert_eq!(decoded.status_present, facts.status_code.is_some());
            assert_eq!(decoded.status_code, facts.status_code.unwrap_or(0));
            assert_eq!(
                decoded.content_length_present,
                facts.content_length.is_some()
            );
            assert_eq!(decoded.content_length, facts.content_length.unwrap_or(0));
            assert_eq!(decoded.response_complete, facts.response_complete);
            assert_eq!(decoded.chunked, facts.chunked);
        }
    }

    // parse_content_length is always None, so the present content_length u64
    // lane is never exercised end-to-end. Cover it directly.
    #[test]
    fn record_covers_present_content_length_u64_lane() {
        let facts = HttpHeadFacts {
            status_code: Some(200),
            content_length: Some(11),
            response_complete: true,
            chunked: true,
        };
        let encoded = encode_httphead_record(facts);
        assert_eq!(&encoded[6..14], &[0, 0, 0, 0, 0, 0, 0, 11]); // big-endian u64
        let decoded = decode_httphead_record(&encoded);
        assert!(decoded.record_valid);
        assert!(decoded.content_length_present);
        assert_eq!(decoded.content_length, 11);
        assert_eq!(decoded.status_code, 200);
        assert!(decoded.response_complete);
        assert!(decoded.chunked);
    }

    // Fail-closed framing (mirrors x509 lib.rs:855-869).
    #[test]
    fn decode_fails_closed_for_bad_framing() {
        let mut wrong_version = encode_httphead_record(parse_http_head(HTTP_CHUNKED));
        wrong_version[1] = HTTPHEAD_RECORD_VERSION.wrapping_add(1);
        let mut wrong_magic = encode_httphead_record(parse_http_head(HTTP_CHUNKED));
        wrong_magic[0] = HTTPHEAD_RECORD_MAGIC.wrapping_add(1);
        for bytes in [
            &[][..],
            &[0u8; HTTPHEAD_RECORD_LEN][..],
            &[0u8; HTTPHEAD_RECORD_LEN - 1][..],
            &[0u8; HTTPHEAD_RECORD_LEN + 1][..],
            &wrong_version[..],
            &wrong_magic[..],
        ] {
            let decoded = decode_httphead_record(bytes);
            assert!(!decoded.record_valid);
            assert!(!decoded.status_present);
            assert_eq!(decoded.content_length, 0);
        }
    }
}
