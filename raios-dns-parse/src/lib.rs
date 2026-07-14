#![cfg_attr(not(test), no_std)]

use core::str;

pub const DNS_DEFAULT_TTL_SECS: u32 = 300;
pub const DNS_QUERY_RECORD_HEADER_LEN: usize = 11;
pub const DNS_QUERY_RECORD_MAX_HOSTNAME_LEN: usize = 253;
pub const DNS_QUERY_RECORD_MAX_PAYLOAD_LEN: usize = 512;
pub const DNS_QUERY_RECORD_MAX_LEN: usize = DNS_QUERY_RECORD_HEADER_LEN
    + DNS_QUERY_RECORD_MAX_HOSTNAME_LEN
    + DNS_QUERY_RECORD_MAX_PAYLOAD_LEN;
pub const DNS_RESPONSE_RECORD_LEN: usize = 16;

const DNS_QUERY_RECORD_MAGIC: &[u8; 4] = b"DNSQ";
const DNS_RESPONSE_RECORD_MAGIC: &[u8; 4] = b"DNSR";
const DNS_RECORD_VERSION: u8 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DnsARecord {
    pub address: [u8; 4],
    pub ttl: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DnsQueryRecord<'a> {
    pub expected_id: u16,
    pub hostname: &'a str,
    pub payload: &'a [u8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodedDnsResponseRecord {
    pub record_valid: bool,
    pub answer: Option<DnsARecord>,
}

pub fn build_dns_query(buffer: &mut [u8], tx_id: u16, hostname: &str) -> Option<usize> {
    if buffer.len() < 12 {
        return None;
    }

    buffer[0] = (tx_id >> 8) as u8;
    buffer[1] = tx_id as u8;
    buffer[2] = 0x01;
    buffer[3] = 0x00;
    buffer[4] = 0x00;
    buffer[5] = 0x01;
    buffer[6..12].fill(0);

    let mut offset = 12;
    for label in hostname.split('.') {
        if label.is_empty() || label.len() > 63 {
            return None;
        }
        if offset + 1 + label.len() > buffer.len() {
            return None;
        }
        buffer[offset] = label.len() as u8;
        offset += 1;
        buffer[offset..offset + label.len()].copy_from_slice(label.as_bytes());
        offset += label.len();
    }

    if offset >= buffer.len() {
        return None;
    }
    buffer[offset] = 0;
    offset += 1;

    if offset + 4 > buffer.len() {
        return None;
    }
    buffer[offset] = 0;
    buffer[offset + 1] = 1;
    buffer[offset + 2] = 0;
    buffer[offset + 3] = 1;
    Some(offset + 4)
}

pub fn parse_dns_response(payload: &[u8], expected_id: u16, hostname: &str) -> Option<DnsARecord> {
    if payload.len() < 12 {
        return None;
    }
    let id = u16::from_be_bytes([payload[0], payload[1]]);
    if id != expected_id {
        return None;
    }
    let flags = u16::from_be_bytes([payload[2], payload[3]]);
    if flags & 0x8000 == 0 || flags & 0x000f != 0 {
        return None;
    }
    let qdcount = u16::from_be_bytes([payload[4], payload[5]]) as usize;
    let ancount = u16::from_be_bytes([payload[6], payload[7]]) as usize;

    let mut offset = 12;
    for _ in 0..qdcount {
        let (name_matches, next) = dns_name_matches(payload, offset, hostname)?;
        offset = next + 4;
        if !name_matches {
            return None;
        }
    }

    for _ in 0..ancount {
        let (name_matches, next) = dns_name_matches(payload, offset, hostname)?;
        if next + 10 > payload.len() {
            return None;
        }
        let rtype = u16::from_be_bytes([payload[next], payload[next + 1]]);
        let class = u16::from_be_bytes([payload[next + 2], payload[next + 3]]);
        let ttl = u32::from_be_bytes([
            payload[next + 4],
            payload[next + 5],
            payload[next + 6],
            payload[next + 7],
        ]);
        let rdlength = u16::from_be_bytes([payload[next + 8], payload[next + 9]]) as usize;
        let data_offset = next + 10;
        if data_offset + rdlength > payload.len() {
            return None;
        }
        if rtype == 1 && class == 1 && rdlength == 4 && name_matches {
            return Some(DnsARecord {
                address: [
                    payload[data_offset],
                    payload[data_offset + 1],
                    payload[data_offset + 2],
                    payload[data_offset + 3],
                ],
                ttl: if ttl == 0 { DNS_DEFAULT_TTL_SECS } else { ttl },
            });
        }
        offset = data_offset + rdlength;
    }

    None
}

pub fn encode_dns_query_record(
    output: &mut [u8],
    expected_id: u16,
    hostname: &str,
    payload: &[u8],
) -> Option<usize> {
    if !valid_record_hostname(hostname) || payload.len() > DNS_QUERY_RECORD_MAX_PAYLOAD_LEN {
        return None;
    }
    let len = DNS_QUERY_RECORD_HEADER_LEN
        .checked_add(hostname.len())?
        .checked_add(payload.len())?;
    if output.len() < len {
        return None;
    }

    output[..4].copy_from_slice(DNS_QUERY_RECORD_MAGIC);
    output[4] = DNS_RECORD_VERSION;
    output[5] = 0;
    output[6..8].copy_from_slice(&expected_id.to_be_bytes());
    output[8] = hostname.len() as u8;
    output[9..11].copy_from_slice(&(payload.len() as u16).to_be_bytes());
    output[11..11 + hostname.len()].copy_from_slice(hostname.as_bytes());
    output[11 + hostname.len()..len].copy_from_slice(payload);
    Some(len)
}

pub fn decode_dns_query_record(bytes: &[u8]) -> Option<DnsQueryRecord<'_>> {
    if bytes.len() < DNS_QUERY_RECORD_HEADER_LEN
        || &bytes[..4] != DNS_QUERY_RECORD_MAGIC
        || bytes[4] != DNS_RECORD_VERSION
        || bytes[5] != 0
    {
        return None;
    }
    let hostname_len = bytes[8] as usize;
    let payload_len = u16::from_be_bytes([bytes[9], bytes[10]]) as usize;
    if hostname_len > DNS_QUERY_RECORD_MAX_HOSTNAME_LEN
        || payload_len > DNS_QUERY_RECORD_MAX_PAYLOAD_LEN
    {
        return None;
    }
    let expected_len = DNS_QUERY_RECORD_HEADER_LEN
        .checked_add(hostname_len)?
        .checked_add(payload_len)?;
    if bytes.len() != expected_len {
        return None;
    }
    let hostname = str::from_utf8(&bytes[11..11 + hostname_len]).ok()?;
    if !valid_record_hostname(hostname) {
        return None;
    }
    Some(DnsQueryRecord {
        expected_id: u16::from_be_bytes([bytes[6], bytes[7]]),
        hostname,
        payload: &bytes[11 + hostname_len..],
    })
}

pub fn encode_dns_response_record(answer: Option<DnsARecord>) -> [u8; DNS_RESPONSE_RECORD_LEN] {
    let mut output = [0u8; DNS_RESPONSE_RECORD_LEN];
    output[..4].copy_from_slice(DNS_RESPONSE_RECORD_MAGIC);
    output[4] = DNS_RECORD_VERSION;
    if let Some(answer) = answer {
        output[5] = 1;
        output[6..10].copy_from_slice(&answer.address);
        output[10..14].copy_from_slice(&answer.ttl.to_be_bytes());
    }
    output
}

pub fn decode_dns_response_record(bytes: &[u8]) -> DecodedDnsResponseRecord {
    if bytes.len() != DNS_RESPONSE_RECORD_LEN
        || &bytes[..4] != DNS_RESPONSE_RECORD_MAGIC
        || bytes[4] != DNS_RECORD_VERSION
        || bytes[14] != 0
        || bytes[15] != 0
    {
        return invalid_response_record();
    }

    let address = [bytes[6], bytes[7], bytes[8], bytes[9]];
    let ttl = u32::from_be_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]);
    let answer = match bytes[5] {
        0 if address == [0; 4] && ttl == 0 => None,
        1 if ttl != 0 => Some(DnsARecord { address, ttl }),
        _ => return invalid_response_record(),
    };
    DecodedDnsResponseRecord {
        record_valid: true,
        answer,
    }
}

fn invalid_response_record() -> DecodedDnsResponseRecord {
    DecodedDnsResponseRecord {
        record_valid: false,
        answer: None,
    }
}

fn valid_record_hostname(hostname: &str) -> bool {
    !hostname.is_empty()
        && hostname.len() <= DNS_QUERY_RECORD_MAX_HOSTNAME_LEN
        && hostname
            .split('.')
            .all(|label| !label.is_empty() && label.len() <= 63)
}

fn dns_name_matches(payload: &[u8], start: usize, expected: &str) -> Option<(bool, usize)> {
    let expected = expected.as_bytes();
    let mut expected_offset = 0usize;
    let mut label_count = 0usize;
    let mut matches = true;
    let mut offset = start;
    let mut jumped = false;
    let mut jump_return = 0usize;
    let mut steps = 0usize;

    loop {
        if offset >= payload.len() {
            return None;
        }
        let len = payload[offset];
        if len & 0xc0 == 0xc0 {
            if offset + 1 >= payload.len() {
                return None;
            }
            let pointer = (((len & 0x3f) as usize) << 8) | payload[offset + 1] as usize;
            if !jumped {
                jump_return = offset + 2;
                jumped = true;
            }
            offset = pointer;
            steps += 1;
            if steps > 16 {
                return None;
            }
            continue;
        } else if len == 0 {
            offset += 1;
            break;
        }

        offset += 1;
        let end = offset.checked_add(len as usize)?;
        if end > payload.len() {
            return None;
        }
        let label = &payload[offset..end];
        str::from_utf8(label).ok()?;
        if label_count > 0 {
            if expected.get(expected_offset) != Some(&b'.') {
                matches = false;
            }
            expected_offset = expected_offset.checked_add(1)?;
        }
        let expected_end = expected_offset.checked_add(label.len())?;
        if expected.get(expected_offset..expected_end) != Some(label) {
            matches = false;
        }
        expected_offset = expected_end;
        label_count += 1;
        offset = end;
    }

    matches &= expected_offset == expected.len();
    Some((matches, if jumped { jump_return } else { offset }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOSTNAME: &str = "api.openai.com";
    const QUERY: [u8; 32] = [
        0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x61, 0x70,
        0x69, 0x06, 0x6f, 0x70, 0x65, 0x6e, 0x61, 0x69, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01,
        0x00, 0x01,
    ];
    const COMPRESSED_RESPONSE: [u8; 48] = [
        0x12, 0x34, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0x61, 0x70,
        0x69, 0x06, 0x6f, 0x70, 0x65, 0x6e, 0x61, 0x69, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01,
        0x00, 0x01, 0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3c, 0x00, 0x04, 0x68,
        0x12, 0x21, 0x2d,
    ];
    const UNCOMPRESSED_RESPONSE: [u8; 62] = [
        0x12, 0x34, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0x61, 0x70,
        0x69, 0x06, 0x6f, 0x70, 0x65, 0x6e, 0x61, 0x69, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01,
        0x00, 0x01, 0x03, 0x61, 0x70, 0x69, 0x06, 0x6f, 0x70, 0x65, 0x6e, 0x61, 0x69, 0x03, 0x63,
        0x6f, 0x6d, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3c, 0x00, 0x04, 0x68, 0x12,
        0x21, 0x2d,
    ];
    const ANSWER: DnsARecord = DnsARecord {
        address: [104, 18, 33, 45],
        ttl: 60,
    };

    fn fixture_hash(bytes: &[u8]) -> [u8; 64] {
        use sha2::{Digest, Sha256};
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let digest = Sha256::digest(bytes);
        let mut output = [0u8; 64];
        for (index, byte) in digest.iter().enumerate() {
            output[index * 2] = HEX[(byte >> 4) as usize];
            output[index * 2 + 1] = HEX[(byte & 0x0f) as usize];
        }
        output
    }

    #[test]
    fn pinned_query_bytes_for_api_openai_com() {
        let mut output = [0u8; 32];
        assert_eq!(build_dns_query(&mut output, 0x1234, HOSTNAME), Some(32));
        assert_eq!(output, QUERY);
    }

    #[test]
    fn compressed_and_uncompressed_matching_a_answers() {
        assert_eq!(
            parse_dns_response(&COMPRESSED_RESPONSE, 0x1234, HOSTNAME),
            Some(ANSWER)
        );
        assert_eq!(
            parse_dns_response(&UNCOMPRESSED_RESPONSE, 0x1234, HOSTNAME),
            Some(ANSWER)
        );
    }

    #[test]
    fn zero_ttl_maps_to_default() {
        let mut response = COMPRESSED_RESPONSE;
        response[38..42].fill(0);
        assert_eq!(
            parse_dns_response(&response, 0x1234, HOSTNAME),
            Some(DnsARecord {
                ttl: DNS_DEFAULT_TTL_SECS,
                ..ANSWER
            })
        );
    }

    #[test]
    fn rejects_wrong_id_response_bit_rcode_question_hostname_type_class_and_no_a() {
        assert_eq!(
            parse_dns_response(&COMPRESSED_RESPONSE, 0x4321, HOSTNAME),
            None
        );

        let mut response_bit_absent = COMPRESSED_RESPONSE;
        response_bit_absent[2] &= 0x7f;
        assert_eq!(
            parse_dns_response(&response_bit_absent, 0x1234, HOSTNAME),
            None
        );

        let mut rcode = COMPRESSED_RESPONSE;
        rcode[3] |= 3;
        assert_eq!(parse_dns_response(&rcode, 0x1234, HOSTNAME), None);

        let mut hostname = COMPRESSED_RESPONSE;
        hostname[13] = b'x';
        assert_eq!(parse_dns_response(&hostname, 0x1234, HOSTNAME), None);

        let mut answer_type = COMPRESSED_RESPONSE;
        answer_type[35] = 28;
        assert_eq!(parse_dns_response(&answer_type, 0x1234, HOSTNAME), None);

        let mut answer_class = COMPRESSED_RESPONSE;
        answer_class[37] = 2;
        assert_eq!(parse_dns_response(&answer_class, 0x1234, HOSTNAME), None);

        let mut no_answer = COMPRESSED_RESPONSE;
        no_answer[6..8].fill(0);
        assert_eq!(parse_dns_response(&no_answer, 0x1234, HOSTNAME), None);
    }

    #[test]
    fn truncation_at_every_header_question_and_answer_boundary_is_rejected() {
        for len in 0..COMPRESSED_RESPONSE.len() {
            assert_eq!(
                parse_dns_response(&COMPRESSED_RESPONSE[..len], 0x1234, HOSTNAME),
                None,
                "prefix length {len}"
            );
        }
    }

    #[test]
    fn compression_pointer_out_of_range_and_more_than_sixteen_jump_cycle_are_rejected() {
        let mut out_of_range = COMPRESSED_RESPONSE;
        out_of_range[32] = 0xff;
        out_of_range[33] = 0xff;
        assert_eq!(parse_dns_response(&out_of_range, 0x1234, HOSTNAME), None);

        let mut cycle = COMPRESSED_RESPONSE;
        cycle[32] = 0xc0;
        cycle[33] = 32;
        assert_eq!(parse_dns_response(&cycle, 0x1234, HOSTNAME), None);
    }

    #[test]
    fn label_length_empty_labels_and_output_exhaustion_follow_live_query_rules() {
        let maximum_label = "a".repeat(63);
        let oversized_label = "a".repeat(64);
        let mut output = [0u8; 512];
        assert!(build_dns_query(&mut output, 1, &maximum_label).is_some());
        assert_eq!(build_dns_query(&mut output, 1, &oversized_label), None);
        for hostname in ["", ".api.openai.com", "api..openai.com", "api.openai.com."] {
            assert_eq!(build_dns_query(&mut output, 1, hostname), None);
        }
        assert_eq!(build_dns_query(&mut [0u8; 31], 0x1234, HOSTNAME), None);
        assert_eq!(
            encode_dns_query_record(&mut [0u8; 42], 0x1234, HOSTNAME, &QUERY),
            None
        );
    }

    #[test]
    fn input_record_round_trip() {
        let mut encoded = [0u8; DNS_QUERY_RECORD_MAX_LEN];
        let len = encode_dns_query_record(&mut encoded, 0x1234, HOSTNAME, &COMPRESSED_RESPONSE)
            .expect("fixture fits");
        let decoded = decode_dns_query_record(&encoded[..len]).expect("record valid");
        assert_eq!(decoded.expected_id, 0x1234);
        assert_eq!(decoded.hostname, HOSTNAME);
        assert_eq!(decoded.payload, &COMPRESSED_RESPONSE);
    }

    #[test]
    fn input_record_rejects_every_malformed_field() {
        let mut valid = [0u8; DNS_QUERY_RECORD_MAX_LEN];
        let len = encode_dns_query_record(&mut valid, 0x1234, HOSTNAME, &COMPRESSED_RESPONSE)
            .expect("fixture fits");

        for index in 0..4 {
            let mut bad = valid;
            bad[index] ^= 1;
            assert_eq!(decode_dns_query_record(&bad[..len]), None);
        }
        let mut version = valid;
        version[4] = 2;
        assert_eq!(decode_dns_query_record(&version[..len]), None);
        let mut reserved = valid;
        reserved[5] = 1;
        assert_eq!(decode_dns_query_record(&reserved[..len]), None);
        assert_eq!(decode_dns_query_record(&valid[..10]), None);
        assert_eq!(decode_dns_query_record(&valid[..len - 1]), None);
        assert_eq!(decode_dns_query_record(&valid[..len + 1]), None);

        let mut hostname_length = valid;
        hostname_length[8] += 1;
        assert_eq!(decode_dns_query_record(&hostname_length[..len]), None);
        let mut payload_length = valid;
        payload_length[10] += 1;
        assert_eq!(decode_dns_query_record(&payload_length[..len]), None);

        let mut invalid_utf8 = [0u8; 12];
        invalid_utf8[..4].copy_from_slice(DNS_QUERY_RECORD_MAGIC);
        invalid_utf8[4] = DNS_RECORD_VERSION;
        invalid_utf8[8] = 1;
        invalid_utf8[11] = 0xff;
        assert_eq!(decode_dns_query_record(&invalid_utf8), None);

        let mut empty_label = [0u8; 15];
        empty_label[..4].copy_from_slice(DNS_QUERY_RECORD_MAGIC);
        empty_label[4] = DNS_RECORD_VERSION;
        empty_label[8] = 4;
        empty_label[11..].copy_from_slice(b"a..b");
        assert_eq!(decode_dns_query_record(&empty_label), None);

        let too_long = format!("{}aa", "a.".repeat(126));
        assert_eq!(too_long.len(), 254);
        assert_eq!(encode_dns_query_record(&mut valid, 1, &too_long, &[]), None);

        let mut payload_too_long = [0u8; DNS_QUERY_RECORD_MAX_LEN];
        payload_too_long[..4].copy_from_slice(DNS_QUERY_RECORD_MAGIC);
        payload_too_long[4] = DNS_RECORD_VERSION;
        payload_too_long[8] = 1;
        payload_too_long[9..11].copy_from_slice(&513u16.to_be_bytes());
        payload_too_long[11] = b'a';
        assert_eq!(decode_dns_query_record(&payload_too_long[..525]), None);
    }

    #[test]
    fn output_record_round_trips_answer_and_canonical_no_answer() {
        let answer = decode_dns_response_record(&encode_dns_response_record(Some(ANSWER)));
        assert!(answer.record_valid);
        assert_eq!(answer.answer, Some(ANSWER));

        let no_answer = encode_dns_response_record(None);
        assert_eq!(&no_answer[5..], &[0; 11]);
        let decoded = decode_dns_response_record(&no_answer);
        assert!(decoded.record_valid);
        assert_eq!(decoded.answer, None);
    }

    #[test]
    fn output_record_rejects_bad_magic_version_length_status_reserved_and_inconsistent_fields() {
        let valid = encode_dns_response_record(Some(ANSWER));
        assert!(!decode_dns_response_record(&valid[..15]).record_valid);

        let mut long = [0u8; 17];
        long[..16].copy_from_slice(&valid);
        assert!(!decode_dns_response_record(&long).record_valid);

        let mut magic = valid;
        magic[0] ^= 1;
        assert!(!decode_dns_response_record(&magic).record_valid);
        let mut version = valid;
        version[4] = 2;
        assert!(!decode_dns_response_record(&version).record_valid);
        let mut status = valid;
        status[5] = 2;
        assert!(!decode_dns_response_record(&status).record_valid);
        for index in [14, 15] {
            let mut reserved = valid;
            reserved[index] = 1;
            assert!(!decode_dns_response_record(&reserved).record_valid);
        }

        let mut no_answer_address = encode_dns_response_record(None);
        no_answer_address[6] = 1;
        assert!(!decode_dns_response_record(&no_answer_address).record_valid);
        let mut no_answer_ttl = encode_dns_response_record(None);
        no_answer_ttl[13] = 1;
        assert!(!decode_dns_response_record(&no_answer_ttl).record_valid);
        let mut zero_answer_ttl = valid;
        zero_answer_ttl[10..14].fill(0);
        assert!(!decode_dns_response_record(&zero_answer_ttl).record_valid);
    }

    #[test]
    fn pinned_fixture_sha256_provenance() {
        assert_eq!(
            &fixture_hash(&QUERY),
            b"0880bac492d57a54e30b35f07bc2857b1118f5ab4a222bae5cbe259193df3224"
        );
        assert_eq!(
            &fixture_hash(&COMPRESSED_RESPONSE),
            b"55d5ea1dd8035478049cd029f9969625212b0fbb9dfd55867eb5dffca7af84b3"
        );
        assert_eq!(
            &fixture_hash(&UNCOMPRESSED_RESPONSE),
            b"70cd34819b23c2927538be39f4887d79bb1195568f667b0f16d6f9e40b149fba"
        );
    }
}
