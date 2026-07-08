#![cfg_attr(not(test), no_std)]

pub const P256_UNCOMPRESSED_POINT_LEN: usize = 65;
const OID_EC_PUBLIC_KEY: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01];
const OID_PRIME256V1: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct P256Spki<'a> {
    pub der: &'a [u8],
    pub public_key: &'a [u8],
}

pub const CERTSPKI_RECORD_LEN: usize = 71;
pub const CERTSPKI_RECORD_MAGIC: u8 = 0x53; // 'S', distinct from HTTPHEAD/CERTWINDOW.
pub const CERTSPKI_RECORD_VERSION: u8 = 1;
pub const CERTSPKI_ERROR_NOT_P256_SPKI: u8 = 1;

/// Fixed 71-byte record, big-endian. Always 71 bytes.
/// [0]=magic 0x53 [1]=version 1 [2]=status(0 ok / 1 err) [3]=error_code(0 when ok)
/// [4..6]=spki_der_len u16 BE [6..71]=P-256 uncompressed public key (65 bytes).
/// On Err, bytes [4..71] are all 0x00.
pub fn encode_certspki_record(result: Option<P256Spki<'_>>) -> [u8; CERTSPKI_RECORD_LEN] {
    let mut b = [0u8; CERTSPKI_RECORD_LEN];
    b[0] = CERTSPKI_RECORD_MAGIC;
    b[1] = CERTSPKI_RECORD_VERSION;
    match result {
        Some(spki)
            if spki.public_key.len() == P256_UNCOMPRESSED_POINT_LEN
                && spki.der.len() <= u16::MAX as usize =>
        {
            b[2] = 0;
            b[3] = 0;
            let len = (spki.der.len() as u16).to_be_bytes();
            b[4] = len[0];
            b[5] = len[1];
            b[6..71].copy_from_slice(spki.public_key);
        }
        _ => {
            b[2] = 1;
            b[3] = CERTSPKI_ERROR_NOT_P256_SPKI;
        }
    }
    b
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodedCertSpkiRecord {
    pub record_valid: bool,
    pub status: u8,
    pub error_code: u8,
    pub spki_der_len: u16,
    pub public_key: [u8; P256_UNCOMPRESSED_POINT_LEN],
}

/// Fail-closed decode. Length/magic/version are guarded first, so an empty or
/// short buffer yields record_valid=false instead of a panic.
pub fn decode_certspki_record(bytes: &[u8]) -> DecodedCertSpkiRecord {
    if bytes.len() != CERTSPKI_RECORD_LEN
        || bytes[0] != CERTSPKI_RECORD_MAGIC
        || bytes[1] != CERTSPKI_RECORD_VERSION
    {
        return DecodedCertSpkiRecord {
            record_valid: false,
            status: 0,
            error_code: 0,
            spki_der_len: 0,
            public_key: [0; P256_UNCOMPRESSED_POINT_LEN],
        };
    }
    let mut public_key = [0u8; P256_UNCOMPRESSED_POINT_LEN];
    public_key.copy_from_slice(&bytes[6..71]);
    DecodedCertSpkiRecord {
        record_valid: true,
        status: bytes[2],
        error_code: bytes[3],
        spki_der_len: u16::from_be_bytes([bytes[4], bytes[5]]),
        public_key,
    }
}

pub fn extract_p256_spki(cert_der: &[u8]) -> Option<P256Spki<'_>> {
    let cert = read_single_tlv(cert_der, 0x30)?;
    let mut cert_reader = DerReader::new(cert.value);
    let tbs = cert_reader.read_tlv(0x30)?;
    let mut tbs_reader = DerReader::new(tbs.value);

    if tbs_reader.peek_tag()? == 0xa0 {
        tbs_reader.read_any()?;
    }
    tbs_reader.read_any()?;
    tbs_reader.read_tlv(0x30)?;
    tbs_reader.read_tlv(0x30)?;
    tbs_reader.read_tlv(0x30)?;
    tbs_reader.read_tlv(0x30)?;

    let spki = tbs_reader.read_tlv(0x30)?;
    let mut spki_reader = DerReader::new(spki.value);
    let algorithm = spki_reader.read_tlv(0x30)?;
    if !algorithm_is_p256_ec(algorithm.value) {
        return None;
    }
    let public_key = spki_reader.read_tlv(0x03)?;
    let (&unused_bits, key) = public_key.value.split_first()?;
    if unused_bits != 0
        || key.len() != P256_UNCOMPRESSED_POINT_LEN
        || key.first().copied() != Some(0x04)
    {
        return None;
    }
    Some(P256Spki {
        der: spki.full,
        public_key: key,
    })
}

fn algorithm_is_p256_ec(algorithm_der: &[u8]) -> bool {
    let mut reader = DerReader::new(algorithm_der);
    let Some(ec_public_key) = reader.read_tlv(0x06) else {
        return false;
    };
    let Some(prime256v1) = reader.read_tlv(0x06) else {
        return false;
    };
    ec_public_key.value == OID_EC_PUBLIC_KEY && prime256v1.value == OID_PRIME256V1
}

fn read_single_tlv(input: &[u8], expected_tag: u8) -> Option<DerTlv<'_>> {
    let mut reader = DerReader::new(input);
    let value = reader.read_tlv(expected_tag)?;
    if reader.is_empty() {
        Some(value)
    } else {
        None
    }
}

struct DerReader<'a> {
    input: &'a [u8],
    offset: usize,
}

impl<'a> DerReader<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, offset: 0 }
    }

    fn is_empty(&self) -> bool {
        self.offset == self.input.len()
    }

    fn peek_tag(&self) -> Option<u8> {
        self.input.get(self.offset).copied()
    }

    fn read_tlv(&mut self, expected_tag: u8) -> Option<DerTlv<'a>> {
        let tlv = self.read_any()?;
        if tlv.tag == expected_tag {
            Some(tlv)
        } else {
            None
        }
    }

    fn read_any(&mut self) -> Option<DerTlv<'a>> {
        let start = self.offset;
        let tag = *self.input.get(self.offset)?;
        self.offset += 1;
        let len = self.read_len()?;
        let value_start = self.offset;
        let value_end = value_start.checked_add(len)?;
        if value_end > self.input.len() {
            return None;
        }
        self.offset = value_end;
        Some(DerTlv {
            tag,
            value: &self.input[value_start..value_end],
            full: &self.input[start..value_end],
        })
    }

    fn read_len(&mut self) -> Option<usize> {
        let first = *self.input.get(self.offset)?;
        self.offset += 1;
        if first & 0x80 == 0 {
            return Some(first as usize);
        }

        let octets = (first & 0x7f) as usize;
        if octets == 0 || octets > 4 {
            return None;
        }
        let mut len = 0usize;
        let mut idx = 0usize;
        while idx < octets {
            len = len.checked_mul(256)?;
            len = len.checked_add(*self.input.get(self.offset)? as usize)?;
            self.offset += 1;
            idx += 1;
        }
        Some(len)
    }
}

struct DerTlv<'a> {
    tag: u8,
    value: &'a [u8],
    #[allow(dead_code)]
    full: &'a [u8],
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fixed DER fixture copied from seed-kernel/src/agent_protocol_time.rs:25-64.
    // len=522, sha256=baa2a6c3263fb8170aa2b4013046414a1e4760c2f5e7bfdf88c74f51742e0cb4.
    const REAL_TEST_CERT_DER: &[u8] = &[
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

    const EXPECTED_PUBLIC_KEY: &[u8; P256_UNCOMPRESSED_POINT_LEN] = &[
        0x04, 0xc4, 0x99, 0xf6, 0xf3, 0xaa, 0xa1, 0xe2, 0x67, 0x00, 0x8a, 0x5e, 0x01, 0x1f, 0x8c,
        0x05, 0xa3, 0x93, 0xac, 0xcf, 0x94, 0xaf, 0x45, 0xb3, 0x76, 0xd7, 0x7e, 0x3a, 0x36, 0x82,
        0xdd, 0x4d, 0xba, 0xa0, 0x38, 0xc8, 0x27, 0x4e, 0x50, 0xb2, 0x9a, 0xe9, 0xa2, 0x05, 0x1f,
        0x20, 0x2f, 0x7c, 0xcd, 0xf3, 0x1c, 0xd8, 0x8b, 0xe6, 0xf9, 0x39, 0xa5, 0xb0, 0x6d, 0xce,
        0x36, 0xba, 0xbd, 0xa2, 0x23,
    ];

    fn sha256_hex(bytes: &[u8]) -> [u8; 64] {
        use sha2::{Digest, Sha256};

        let digest = Sha256::digest(bytes);
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = [0u8; 64];
        for (idx, byte) in digest.iter().enumerate() {
            out[idx * 2] = HEX[(byte >> 4) as usize];
            out[idx * 2 + 1] = HEX[(byte & 0x0f) as usize];
        }
        out
    }

    #[test]
    fn fixture_bytes_match_agent_protocol_time_source() {
        assert_eq!(REAL_TEST_CERT_DER.len(), 522);
        assert_eq!(
            &sha256_hex(REAL_TEST_CERT_DER),
            b"baa2a6c3263fb8170aa2b4013046414a1e4760c2f5e7bfdf88c74f51742e0cb4"
        );
    }

    #[test]
    fn extracts_p256_spki_from_agent_protocol_time_fixture() {
        let spki = extract_p256_spki(REAL_TEST_CERT_DER).expect("fixture has P-256 SPKI");
        assert_eq!(spki.der.len(), 91);
        assert_eq!(spki.der[0], 0x30);
        assert_eq!(spki.der[1], 0x59);
        assert_eq!(spki.public_key, EXPECTED_PUBLIC_KEY);
    }

    #[test]
    fn truncated_cert_is_rejected() {
        assert!(extract_p256_spki(&REAL_TEST_CERT_DER[..120]).is_none());
    }

    #[test]
    fn certspki_record_round_trips_fixture_success() {
        let spki = extract_p256_spki(REAL_TEST_CERT_DER).expect("fixture has P-256 SPKI");
        let decoded = decode_certspki_record(&encode_certspki_record(Some(spki)));

        assert!(decoded.record_valid);
        assert_eq!(decoded.status, 0);
        assert_eq!(decoded.error_code, 0);
        assert_eq!(decoded.spki_der_len, 91);
        assert_eq!(&decoded.public_key, EXPECTED_PUBLIC_KEY);
    }

    #[test]
    fn certspki_record_fails_closed_for_bad_framing_and_missing_spki() {
        let err = decode_certspki_record(&encode_certspki_record(None));
        assert!(err.record_valid);
        assert_eq!(err.status, 1);
        assert_eq!(err.error_code, CERTSPKI_ERROR_NOT_P256_SPKI);
        assert_eq!(err.spki_der_len, 0);

        let mut wrong_version = encode_certspki_record(extract_p256_spki(REAL_TEST_CERT_DER));
        wrong_version[1] = CERTSPKI_RECORD_VERSION.wrapping_add(1);
        for bytes in [&[][..], &[0u8; CERTSPKI_RECORD_LEN][..], &wrong_version[..]] {
            let decoded = decode_certspki_record(bytes);
            assert!(!decoded.record_valid);
            assert_eq!(decoded.status, 0);
            assert_eq!(decoded.spki_der_len, 0);
        }
    }
}
