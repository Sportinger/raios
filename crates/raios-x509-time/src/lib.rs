#![cfg_attr(not(test), no_std)]

#[cfg(test)]
fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut out = [0u8; 32];
    out.copy_from_slice(&Sha256::digest(bytes));
    out
}

#[cfg(test)]
fn sha256_hex(digest: &[u8; 32]) -> [u8; 64] {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = [0u8; 64];
    let mut i = 0;
    while i < digest.len() {
        out[i * 2] = HEX[(digest[i] >> 4) as usize];
        out[i * 2 + 1] = HEX[(digest[i] & 0x0f) as usize];
        i += 1;
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CertValidityDateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CertValidityWindow {
    pub not_before: CertValidityDateTime,
    pub not_after: CertValidityDateTime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum X509ValidityError {
    DerTooLong,
    Truncated,
    UnexpectedTag,
    IndefiniteLengthUnsupported,
    LongFormLengthUnsupported,
    NonMinimalLength,
    LengthOverflow,
    HighTagNumberUnsupported,
    ValidityNotTwoTimes,
    TrailingBytes,
    UnsupportedTimeTag,
    MalformedTime,
    NonUtcTimeUnsupported,
    ImplausibleTimeComponent,
}

impl X509ValidityError {
    pub const fn reason(self) -> &'static str {
        match self {
            X509ValidityError::DerTooLong => "der_too_long",
            X509ValidityError::Truncated => "truncated",
            X509ValidityError::UnexpectedTag => "unexpected_tag",
            X509ValidityError::IndefiniteLengthUnsupported => "indefinite_length_unsupported",
            X509ValidityError::LongFormLengthUnsupported => "long_form_length_unsupported",
            X509ValidityError::NonMinimalLength => "non_minimal_length",
            X509ValidityError::LengthOverflow => "length_overflow",
            X509ValidityError::HighTagNumberUnsupported => "high_tag_number_unsupported",
            X509ValidityError::ValidityNotTwoTimes => "validity_not_two_times",
            X509ValidityError::TrailingBytes => "trailing_bytes",
            X509ValidityError::UnsupportedTimeTag => "unsupported_time_tag",
            X509ValidityError::MalformedTime => "malformed_time",
            X509ValidityError::NonUtcTimeUnsupported => "non_utc_time_unsupported",
            X509ValidityError::ImplausibleTimeComponent => "implausible_time_component",
        }
    }

    /// Stable wire code 1..=14, explicit match (never `as u8`) so codes never
    /// shift if the enum is reordered. 0 is reserved for "no error".
    pub const fn code(self) -> u8 {
        match self {
            X509ValidityError::DerTooLong => 1,
            X509ValidityError::Truncated => 2,
            X509ValidityError::UnexpectedTag => 3,
            X509ValidityError::IndefiniteLengthUnsupported => 4,
            X509ValidityError::LongFormLengthUnsupported => 5,
            X509ValidityError::NonMinimalLength => 6,
            X509ValidityError::LengthOverflow => 7,
            X509ValidityError::HighTagNumberUnsupported => 8,
            X509ValidityError::ValidityNotTwoTimes => 9,
            X509ValidityError::TrailingBytes => 10,
            X509ValidityError::UnsupportedTimeTag => 11,
            X509ValidityError::MalformedTime => 12,
            X509ValidityError::NonUtcTimeUnsupported => 13,
            X509ValidityError::ImplausibleTimeComponent => 14,
        }
    }

    pub const fn from_code(code: u8) -> Option<X509ValidityError> {
        Some(match code {
            1 => X509ValidityError::DerTooLong,
            2 => X509ValidityError::Truncated,
            3 => X509ValidityError::UnexpectedTag,
            4 => X509ValidityError::IndefiniteLengthUnsupported,
            5 => X509ValidityError::LongFormLengthUnsupported,
            6 => X509ValidityError::NonMinimalLength,
            7 => X509ValidityError::LengthOverflow,
            8 => X509ValidityError::HighTagNumberUnsupported,
            9 => X509ValidityError::ValidityNotTwoTimes,
            10 => X509ValidityError::TrailingBytes,
            11 => X509ValidityError::UnsupportedTimeTag,
            12 => X509ValidityError::MalformedTime,
            13 => X509ValidityError::NonUtcTimeUnsupported,
            14 => X509ValidityError::ImplausibleTimeComponent,
            _ => return None,
        })
    }
}

pub const MAX_X509_CERT_DER_LEN: usize = 16 * 1024;
pub const CERTWINDOW_RECORD_LEN: usize = 18;
pub const CERTWINDOW_RECORD_MAGIC: u8 = 0xC7;
pub const CERTWINDOW_RECORD_VERSION: u8 = 1;

/// Fixed 18-byte record, big-endian years. Always 18 bytes.
/// [0]=magic 0xC7 [1]=version 1 [2]=status(0 ok / 1 err) [3]=error_code(0 when ok)
/// [4..6]=nb.year BE [6]=nb.month [7]=nb.day [8]=nb.hour [9]=nb.min [10]=nb.sec
/// [11..13]=na.year BE [13]=na.month [14]=na.day [15]=na.hour [16]=na.min [17]=na.sec
/// On Err, bytes [4..18] are all 0x00.
pub fn encode_certwindow_record(
    result: Result<CertValidityWindow, X509ValidityError>,
) -> [u8; CERTWINDOW_RECORD_LEN] {
    let mut b = [0u8; CERTWINDOW_RECORD_LEN];
    b[0] = CERTWINDOW_RECORD_MAGIC;
    b[1] = CERTWINDOW_RECORD_VERSION;
    match result {
        Ok(w) => {
            b[2] = 0;
            b[3] = 0;
            let nb = w.not_before.year.to_be_bytes();
            b[4] = nb[0];
            b[5] = nb[1];
            b[6] = w.not_before.month;
            b[7] = w.not_before.day;
            b[8] = w.not_before.hour;
            b[9] = w.not_before.minute;
            b[10] = w.not_before.second;
            let na = w.not_after.year.to_be_bytes();
            b[11] = na[0];
            b[12] = na[1];
            b[13] = w.not_after.month;
            b[14] = w.not_after.day;
            b[15] = w.not_after.hour;
            b[16] = w.not_after.minute;
            b[17] = w.not_after.second;
        }
        Err(e) => {
            b[2] = 1;
            b[3] = e.code();
        }
    }
    b
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodedCertWindowRecord {
    pub record_valid: bool,
    pub status: u8,
    pub error_code: u8,
    pub window: Option<CertValidityWindow>,
}

/// Fail-closed decode. Length/magic/version are guarded first, so an empty or
/// short buffer yields record_valid=false instead of a panic.
pub fn decode_certwindow_record(bytes: &[u8]) -> DecodedCertWindowRecord {
    if bytes.len() != CERTWINDOW_RECORD_LEN
        || bytes[0] != CERTWINDOW_RECORD_MAGIC
        || bytes[1] != CERTWINDOW_RECORD_VERSION
    {
        return DecodedCertWindowRecord {
            record_valid: false,
            status: 0,
            error_code: 0,
            window: None,
        };
    }
    let status = bytes[2];
    let error_code = bytes[3];
    let window = if status == 0 {
        Some(CertValidityWindow {
            not_before: CertValidityDateTime {
                year: u16::from_be_bytes([bytes[4], bytes[5]]),
                month: bytes[6],
                day: bytes[7],
                hour: bytes[8],
                minute: bytes[9],
                second: bytes[10],
            },
            not_after: CertValidityDateTime {
                year: u16::from_be_bytes([bytes[11], bytes[12]]),
                month: bytes[13],
                day: bytes[14],
                hour: bytes[15],
                minute: bytes[16],
                second: bytes[17],
            },
        })
    } else {
        None
    };
    DecodedCertWindowRecord {
        record_valid: true,
        status,
        error_code,
        window,
    }
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

pub fn parse_x509_cert_validity_window(
    cert_der: &[u8],
) -> Result<CertValidityWindow, X509ValidityError> {
    if cert_der.len() > MAX_X509_CERT_DER_LEN {
        return Err(X509ValidityError::DerTooLong);
    }

    let mut cert = DerCursor::new(cert_der);
    let outer = cert.read_expected(0x30)?;
    if !cert.at_end() {
        return Err(X509ValidityError::TrailingBytes);
    }

    let mut outer = DerCursor::new(outer.value);
    let tbs = outer.read_expected(0x30)?;
    outer.read_expected(0x30)?;
    outer.read_expected(0x03)?;
    if !outer.at_end() {
        return Err(X509ValidityError::TrailingBytes);
    }

    let mut tbs = DerCursor::new(tbs.value);
    if tbs.peek_tag() == Some(0xa0) {
        tbs.read_any()?;
    }
    tbs.read_expected(0x02)?;
    tbs.read_expected(0x30)?;
    tbs.read_expected(0x30)?;

    let validity = tbs.read_expected(0x30)?;
    let window = parse_validity_sequence(validity.value)?;

    tbs.read_expected(0x30)?;
    tbs.read_expected(0x30)?;
    let mut seen_issuer_unique_id = false;
    let mut seen_subject_unique_id = false;
    let mut seen_extensions = false;
    while !tbs.at_end() {
        let tag = tbs.peek_tag().ok_or(X509ValidityError::Truncated)?;
        match tag {
            0x81 if !seen_issuer_unique_id => {
                seen_issuer_unique_id = true;
                tbs.read_any()?;
            }
            0x82 if !seen_subject_unique_id => {
                seen_subject_unique_id = true;
                tbs.read_any()?;
            }
            0xa3 if !seen_extensions => {
                seen_extensions = true;
                tbs.read_any()?;
            }
            _ => return Err(X509ValidityError::UnexpectedTag),
        }
    }

    Ok(window)
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

fn parse_validity_sequence(bytes: &[u8]) -> Result<CertValidityWindow, X509ValidityError> {
    let mut validity = DerCursor::new(bytes);
    if validity.at_end() {
        return Err(X509ValidityError::ValidityNotTwoTimes);
    }
    let not_before = decode_x509_time(validity.read_any()?)?;
    if validity.at_end() {
        return Err(X509ValidityError::ValidityNotTwoTimes);
    }
    let not_after = decode_x509_time(validity.read_any()?)?;
    if !validity.at_end() {
        return Err(X509ValidityError::ValidityNotTwoTimes);
    }
    Ok(CertValidityWindow {
        not_before,
        not_after,
    })
}

#[derive(Clone, Copy)]
struct DerTlv<'a> {
    tag: u8,
    value: &'a [u8],
}

struct DerCursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> DerCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn peek_tag(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn read_expected(&mut self, expected: u8) -> Result<DerTlv<'a>, X509ValidityError> {
        let tlv = self.read_any()?;
        if tlv.tag == expected {
            Ok(tlv)
        } else {
            Err(X509ValidityError::UnexpectedTag)
        }
    }

    fn read_any(&mut self) -> Result<DerTlv<'a>, X509ValidityError> {
        let tag = self.read_byte()?;
        if tag & 0x1f == 0x1f {
            return Err(X509ValidityError::HighTagNumberUnsupported);
        }
        let len = self.read_len()?;
        let start = self.pos;
        let end = start
            .checked_add(len)
            .ok_or(X509ValidityError::LengthOverflow)?;
        if end > self.bytes.len() {
            return Err(X509ValidityError::Truncated);
        }
        self.pos = end;
        Ok(DerTlv {
            tag,
            value: &self.bytes[start..end],
        })
    }

    fn read_byte(&mut self) -> Result<u8, X509ValidityError> {
        let byte = self
            .bytes
            .get(self.pos)
            .copied()
            .ok_or(X509ValidityError::Truncated)?;
        self.pos += 1;
        Ok(byte)
    }

    fn read_len(&mut self) -> Result<usize, X509ValidityError> {
        let first = self.read_byte()?;
        if first < 0x80 {
            return Ok(first as usize);
        }
        if first == 0x80 {
            return Err(X509ValidityError::IndefiniteLengthUnsupported);
        }

        let count = (first & 0x7f) as usize;
        if count > 4 {
            return Err(X509ValidityError::LongFormLengthUnsupported);
        }

        let first_len_byte = self
            .bytes
            .get(self.pos)
            .copied()
            .ok_or(X509ValidityError::Truncated)?;
        if first_len_byte == 0 {
            return Err(X509ValidityError::NonMinimalLength);
        }

        let mut len = 0usize;
        let mut idx = 0usize;
        while idx < count {
            let byte = self.read_byte()? as usize;
            len = len
                .checked_mul(256)
                .and_then(|value| value.checked_add(byte))
                .ok_or(X509ValidityError::LengthOverflow)?;
            idx += 1;
        }
        if len <= 127 {
            return Err(X509ValidityError::NonMinimalLength);
        }
        Ok(len)
    }
}

fn decode_x509_time(tlv: DerTlv<'_>) -> Result<CertValidityDateTime, X509ValidityError> {
    match tlv.tag {
        0x17 => decode_utc_time(tlv.value),
        0x18 => decode_generalized_time(tlv.value),
        _ => Err(X509ValidityError::UnsupportedTimeTag),
    }
}

fn decode_utc_time(bytes: &[u8]) -> Result<CertValidityDateTime, X509ValidityError> {
    if bytes.len() != 13 {
        return malformed_or_non_utc(bytes);
    }
    if bytes.last().copied() != Some(b'Z') {
        return Err(X509ValidityError::NonUtcTimeUnsupported);
    }
    let year = digits(bytes, 0, 2)?;
    let year = if year <= 49 { 2000 + year } else { 1900 + year };
    build_datetime(
        year,
        digits(bytes, 2, 2)? as u8,
        digits(bytes, 4, 2)? as u8,
        digits(bytes, 6, 2)? as u8,
        digits(bytes, 8, 2)? as u8,
        digits(bytes, 10, 2)? as u8,
    )
}

fn decode_generalized_time(bytes: &[u8]) -> Result<CertValidityDateTime, X509ValidityError> {
    if bytes.len() != 15 {
        return malformed_or_non_utc(bytes);
    }
    if bytes.last().copied() != Some(b'Z') {
        return Err(X509ValidityError::NonUtcTimeUnsupported);
    }
    build_datetime(
        digits(bytes, 0, 4)?,
        digits(bytes, 4, 2)? as u8,
        digits(bytes, 6, 2)? as u8,
        digits(bytes, 8, 2)? as u8,
        digits(bytes, 10, 2)? as u8,
        digits(bytes, 12, 2)? as u8,
    )
}

fn malformed_or_non_utc(bytes: &[u8]) -> Result<CertValidityDateTime, X509ValidityError> {
    if bytes.iter().any(|byte| *byte == b'+' || *byte == b'-') {
        Err(X509ValidityError::NonUtcTimeUnsupported)
    } else {
        Err(X509ValidityError::MalformedTime)
    }
}

fn digits(bytes: &[u8], start: usize, len: usize) -> Result<u16, X509ValidityError> {
    let mut value = 0u16;
    let mut idx = 0usize;
    while idx < len {
        let byte = bytes
            .get(start + idx)
            .copied()
            .ok_or(X509ValidityError::MalformedTime)?;
        if !byte.is_ascii_digit() {
            return Err(X509ValidityError::MalformedTime);
        }
        value = value
            .checked_mul(10)
            .and_then(|value| value.checked_add((byte - b'0') as u16))
            .ok_or(X509ValidityError::MalformedTime)?;
        idx += 1;
    }
    Ok(value)
}

fn build_datetime(
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> Result<CertValidityDateTime, X509ValidityError> {
    let value = CertValidityDateTime {
        year,
        month,
        day,
        hour,
        minute,
        second,
    };
    if valid(value) {
        Ok(value)
    } else {
        Err(X509ValidityError::ImplausibleTimeComponent)
    }
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
    use crate::{sha256_bytes, sha256_hex};

    // DER decoded from vendor/embedded-tls-0.17.0/tests/data/server-cert.pem.
    // Provenance: sha256 baa2a6c3263fb8170aa2b4013046414a1e4760c2f5e7bfdf88c74f51742e0cb4,
    // `openssl x509 -noout -dates` -> notBefore=Oct 13 08:20:42 2021 GMT,
    // notAfter=Oct 11 08:20:42 2031 GMT.
    const SERVER_CERT_DER: &[u8] = &[
        0x30, 0x82, 0x02, 0x06, 0x30, 0x82, 0x01, 0xad, 0xa0, 0x03, 0x02, 0x01, 0x02, 0x02, 0x14,
        0x2c, 0x3c, 0x82, 0x61, 0x8b, 0x5e, 0x91, 0xad, 0x92, 0xea, 0x11, 0x02, 0xdb, 0xcf, 0x74,
        0xa3, 0xb9, 0x3e, 0xd1, 0x9d, 0x30, 0x0a, 0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x04,
        0x03, 0x02, 0x30, 0x42, 0x31, 0x0b, 0x30, 0x09, 0x06, 0x03, 0x55, 0x04, 0x06, 0x13, 0x02,
        0x58, 0x58, 0x31, 0x15, 0x30, 0x13, 0x06, 0x03, 0x55, 0x04, 0x07, 0x0c, 0x0c, 0x44, 0x65,
        0x66, 0x61, 0x75, 0x6c, 0x74, 0x20, 0x43, 0x69, 0x74, 0x79, 0x31, 0x1c, 0x30, 0x1a, 0x06,
        0x03, 0x55, 0x04, 0x0a, 0x0c, 0x13, 0x44, 0x65, 0x66, 0x61, 0x75, 0x6c, 0x74, 0x20, 0x43,
        0x6f, 0x6d, 0x70, 0x61, 0x6e, 0x79, 0x20, 0x4c, 0x74, 0x64, 0x30, 0x1e, 0x17, 0x0d, 0x32,
        0x31, 0x31, 0x30, 0x31, 0x33, 0x30, 0x38, 0x32, 0x30, 0x34, 0x32, 0x5a, 0x17, 0x0d, 0x33,
        0x31, 0x31, 0x30, 0x31, 0x31, 0x30, 0x38, 0x32, 0x30, 0x34, 0x32, 0x5a, 0x30, 0x72, 0x31,
        0x0b, 0x30, 0x09, 0x06, 0x03, 0x55, 0x04, 0x06, 0x13, 0x02, 0x4e, 0x4f, 0x31, 0x0e, 0x30,
        0x0c, 0x06, 0x03, 0x55, 0x04, 0x08, 0x0c, 0x05, 0x48, 0x61, 0x6d, 0x61, 0x72, 0x31, 0x0e,
        0x30, 0x0c, 0x06, 0x03, 0x55, 0x04, 0x07, 0x0c, 0x05, 0x48, 0x61, 0x6d, 0x61, 0x72, 0x31,
        0x18, 0x30, 0x16, 0x06, 0x03, 0x55, 0x04, 0x0a, 0x0c, 0x0f, 0x47, 0x6c, 0x6f, 0x62, 0x61,
        0x6c, 0x20, 0x53, 0x65, 0x63, 0x75, 0x72, 0x69, 0x74, 0x79, 0x31, 0x15, 0x30, 0x13, 0x06,
        0x03, 0x55, 0x04, 0x0b, 0x0c, 0x0c, 0x48, 0x6f, 0x6c, 0x73, 0x65, 0x74, 0x62, 0x61, 0x6b,
        0x6b, 0x65, 0x6e, 0x31, 0x12, 0x30, 0x10, 0x06, 0x03, 0x55, 0x04, 0x03, 0x0c, 0x09, 0x6c,
        0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f, 0x73, 0x74, 0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a,
        0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, 0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01,
        0x07, 0x03, 0x42, 0x00, 0x04, 0xc4, 0x99, 0xf6, 0xf3, 0xaa, 0xa1, 0xe2, 0x67, 0x00, 0x8a,
        0x5e, 0x01, 0x1f, 0x8c, 0x05, 0xa3, 0x93, 0xac, 0xcf, 0x94, 0xaf, 0x45, 0xb3, 0x76, 0xd7,
        0x7e, 0x3a, 0x36, 0x82, 0xdd, 0x4d, 0xba, 0xa0, 0x38, 0xc8, 0x27, 0x4e, 0x50, 0xb2, 0x9a,
        0xe9, 0xa2, 0x05, 0x1f, 0x20, 0x2f, 0x7c, 0xcd, 0xf3, 0x1c, 0xd8, 0x8b, 0xe6, 0xf9, 0x39,
        0xa5, 0xb0, 0x6d, 0xce, 0x36, 0xba, 0xbd, 0xa2, 0x23, 0xa3, 0x51, 0x30, 0x4f, 0x30, 0x1f,
        0x06, 0x03, 0x55, 0x1d, 0x23, 0x04, 0x18, 0x30, 0x16, 0x80, 0x14, 0xec, 0x74, 0x3a, 0xe2,
        0x98, 0xac, 0x83, 0x53, 0x1a, 0xb0, 0xdf, 0x70, 0x48, 0xb1, 0x3f, 0x2c, 0x2e, 0x8f, 0x72,
        0x3a, 0x30, 0x09, 0x06, 0x03, 0x55, 0x1d, 0x13, 0x04, 0x02, 0x30, 0x00, 0x30, 0x0b, 0x06,
        0x03, 0x55, 0x1d, 0x0f, 0x04, 0x04, 0x03, 0x02, 0x04, 0xf0, 0x30, 0x14, 0x06, 0x03, 0x55,
        0x1d, 0x11, 0x04, 0x0d, 0x30, 0x0b, 0x82, 0x09, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x68, 0x6f,
        0x73, 0x74, 0x30, 0x0a, 0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x04, 0x03, 0x02, 0x03,
        0x47, 0x00, 0x30, 0x44, 0x02, 0x20, 0x6f, 0x34, 0x17, 0xfc, 0x5a, 0x21, 0xa5, 0xba, 0xcc,
        0x13, 0xe4, 0x04, 0xe9, 0xba, 0x34, 0x52, 0xb6, 0x1d, 0x5c, 0x8d, 0x61, 0x49, 0xa4, 0xac,
        0xf3, 0x28, 0xdd, 0x33, 0xb7, 0x6b, 0xe0, 0x7a, 0x02, 0x20, 0x43, 0x3e, 0x75, 0xba, 0x83,
        0x1a, 0x6f, 0x90, 0x1b, 0xeb, 0x64, 0x84, 0x7e, 0xe3, 0x07, 0x7e, 0x5c, 0xcd, 0xb0, 0x56,
        0x98, 0x2f, 0xd8, 0xe4, 0xe6, 0xcc, 0x33, 0x6f, 0x62, 0xfe, 0xed, 0xa1,
    ];

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

    fn decode_time(tag: u8, value: &[u8]) -> Result<CertValidityDateTime, X509ValidityError> {
        decode_x509_time(DerTlv { tag, value })
    }

    fn assert_grants_nothing(decision: CertValidityWindowDecision) {
        assert_eq!(decision.basis_source, "cmos_rtc_unverified");
        assert!(!decision.trusted);
        assert!(!decision.source_verified);
        assert!(!decision.validates_cert_time);
        assert!(!decision.authorizes_provider_request);
        assert!(!decision.authorizes_provider_export);
        assert!(!decision.durable_write);
        assert!(!decision.capability_granted);
    }

    #[test]
    fn x509_fixture_bytes_match_vendor_cert_provenance() {
        assert_eq!(SERVER_CERT_DER.len(), 522);
        assert_eq!(
            &sha256_hex(&sha256_bytes(SERVER_CERT_DER)),
            b"baa2a6c3263fb8170aa2b4013046414a1e4760c2f5e7bfdf88c74f51742e0cb4"
        );
    }

    #[test]
    fn parses_real_vendor_server_cert_validity_window() {
        let window = parse_x509_cert_validity_window(SERVER_CERT_DER).expect("fixture parses");
        assert_eq!(window.not_before, dt(2021, 10, 13, 8, 20, 42));
        assert_eq!(window.not_after, dt(2031, 10, 11, 8, 20, 42));
    }

    #[test]
    fn truncated_cert_is_rejected() {
        assert_eq!(
            parse_x509_cert_validity_window(&SERVER_CERT_DER[..120]).unwrap_err(),
            X509ValidityError::Truncated
        );
    }

    #[test]
    fn corrupt_outer_tag_and_bad_length_octet_are_distinct_errors() {
        let mut bad_tag = [0u8; 522];
        bad_tag.copy_from_slice(SERVER_CERT_DER);
        bad_tag[0] = 0x31;
        assert_eq!(
            parse_x509_cert_validity_window(&bad_tag)
                .unwrap_err()
                .reason(),
            "unexpected_tag"
        );

        let mut bad_len = [0u8; 522];
        bad_len.copy_from_slice(SERVER_CERT_DER);
        bad_len[1] = 0x80;
        assert_eq!(
            parse_x509_cert_validity_window(&bad_len)
                .unwrap_err()
                .reason(),
            "indefinite_length_unsupported"
        );
    }

    #[test]
    fn utc_time_pivot_decodes_2049_and_1950() {
        assert_eq!(
            decode_time(0x17, b"491231235959Z").unwrap(),
            dt(2049, 12, 31, 23, 59, 59)
        );
        assert_eq!(
            decode_time(0x17, b"500101000000Z").unwrap(),
            dt(1950, 1, 1, 0, 0, 0)
        );
    }

    #[test]
    fn generalized_time_decodes_year_2050() {
        assert_eq!(
            decode_time(0x18, b"20501231235959Z").unwrap(),
            dt(2050, 12, 31, 23, 59, 59)
        );
    }

    #[test]
    fn non_utc_offset_and_fractional_seconds_are_rejected() {
        assert_eq!(
            decode_time(0x17, b"230101000000+0000").unwrap_err(),
            X509ValidityError::NonUtcTimeUnsupported
        );
        assert_eq!(
            decode_time(0x18, b"20501231235959.1Z").unwrap_err(),
            X509ValidityError::MalformedTime
        );
    }

    #[test]
    fn parsed_fixture_feeds_unverified_comparator_and_grants_nothing() {
        let window = parse_x509_cert_validity_window(SERVER_CERT_DER).expect("fixture parses");
        let inside = evaluate_cert_validity_window_unverified_basis(
            window.not_before,
            dt(2026, 1, 1, 0, 0, 0),
            window.not_after,
        );
        assert_eq!(inside.status, "within_window_unverified_basis");
        assert_grants_nothing(inside);

        let expired = evaluate_cert_validity_window_unverified_basis(
            window.not_before,
            dt(2032, 1, 1, 0, 0, 0),
            window.not_after,
        );
        assert_eq!(expired.status, "after_expired_unverified_basis");
        assert_grants_nothing(expired);
    }

    #[test]
    fn x509_validity_error_reasons_are_pairwise_unique() {
        let reasons = [
            X509ValidityError::DerTooLong.reason(),
            X509ValidityError::Truncated.reason(),
            X509ValidityError::UnexpectedTag.reason(),
            X509ValidityError::IndefiniteLengthUnsupported.reason(),
            X509ValidityError::LongFormLengthUnsupported.reason(),
            X509ValidityError::NonMinimalLength.reason(),
            X509ValidityError::LengthOverflow.reason(),
            X509ValidityError::HighTagNumberUnsupported.reason(),
            X509ValidityError::ValidityNotTwoTimes.reason(),
            X509ValidityError::TrailingBytes.reason(),
            X509ValidityError::UnsupportedTimeTag.reason(),
            X509ValidityError::MalformedTime.reason(),
            X509ValidityError::NonUtcTimeUnsupported.reason(),
            X509ValidityError::ImplausibleTimeComponent.reason(),
        ];
        let mut idx = 0usize;
        while idx < reasons.len() {
            let mut other = idx + 1;
            while other < reasons.len() {
                assert_ne!(reasons[idx], reasons[other]);
                other += 1;
            }
            idx += 1;
        }
    }

    #[test]
    fn x509_validity_error_wire_codes_are_stable_and_bijective() {
        let variants = [
            X509ValidityError::DerTooLong,
            X509ValidityError::Truncated,
            X509ValidityError::UnexpectedTag,
            X509ValidityError::IndefiniteLengthUnsupported,
            X509ValidityError::LongFormLengthUnsupported,
            X509ValidityError::NonMinimalLength,
            X509ValidityError::LengthOverflow,
            X509ValidityError::HighTagNumberUnsupported,
            X509ValidityError::ValidityNotTwoTimes,
            X509ValidityError::TrailingBytes,
            X509ValidityError::UnsupportedTimeTag,
            X509ValidityError::MalformedTime,
            X509ValidityError::NonUtcTimeUnsupported,
            X509ValidityError::ImplausibleTimeComponent,
        ];

        for code in 1..=14 {
            assert_eq!(X509ValidityError::from_code(code).unwrap().code(), code);
        }
        for (idx, variant) in variants.iter().copied().enumerate() {
            let code = (idx + 1) as u8;
            assert_eq!(variant.code(), code);
            assert_eq!(X509ValidityError::from_code(code), Some(variant));
        }
        assert_eq!(X509ValidityError::from_code(0), None);
        assert_eq!(X509ValidityError::from_code(15), None);
    }

    #[test]
    fn certwindow_record_round_trips_fixture_success() {
        let parsed = parse_x509_cert_validity_window(SERVER_CERT_DER);
        let decoded = decode_certwindow_record(&encode_certwindow_record(parsed));

        assert!(decoded.record_valid);
        assert_eq!(decoded.status, 0);
        assert_eq!(decoded.error_code, 0);
        assert_eq!(
            decoded.window,
            Some(parse_x509_cert_validity_window(SERVER_CERT_DER).unwrap())
        );
    }

    #[test]
    fn certwindow_record_round_trips_every_error_variant() {
        let variants = [
            X509ValidityError::DerTooLong,
            X509ValidityError::Truncated,
            X509ValidityError::UnexpectedTag,
            X509ValidityError::IndefiniteLengthUnsupported,
            X509ValidityError::LongFormLengthUnsupported,
            X509ValidityError::NonMinimalLength,
            X509ValidityError::LengthOverflow,
            X509ValidityError::HighTagNumberUnsupported,
            X509ValidityError::ValidityNotTwoTimes,
            X509ValidityError::TrailingBytes,
            X509ValidityError::UnsupportedTimeTag,
            X509ValidityError::MalformedTime,
            X509ValidityError::NonUtcTimeUnsupported,
            X509ValidityError::ImplausibleTimeComponent,
        ];

        for variant in variants {
            let decoded = decode_certwindow_record(&encode_certwindow_record(Err(variant)));
            assert!(decoded.record_valid);
            assert_eq!(decoded.status, 1);
            assert_eq!(decoded.error_code, variant.code());
            assert_eq!(decoded.window, None);
        }
    }

    #[test]
    fn certwindow_record_decode_fails_closed_for_bad_framing() {
        let mut wrong_version =
            encode_certwindow_record(parse_x509_cert_validity_window(SERVER_CERT_DER));
        wrong_version[1] = CERTWINDOW_RECORD_VERSION.wrapping_add(1);

        for bytes in [
            &[][..],
            &[0u8; CERTWINDOW_RECORD_LEN][..],
            &wrong_version[..],
        ] {
            let decoded = decode_certwindow_record(bytes);
            assert!(!decoded.record_valid);
            assert_eq!(decoded.window, None);
        }
    }

    #[test]
    fn certwindow_record_truncated_error_code_is_pinned() {
        assert_eq!(
            encode_certwindow_record(parse_x509_cert_validity_window(&SERVER_CERT_DER[..120]))[3],
            2
        );
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
            assert_grants_nothing(decision);
        }
    }
}
