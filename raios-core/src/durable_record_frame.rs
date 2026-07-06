use alloc::{vec, vec::Vec};

use crate::{
    record::{Field, Value},
    sha256_bytes,
};

pub const DURABLE_RECORD_LOG_SCAN_SCHEMA: &str = "raios.durable_record_log_scan.v0";
pub const DURABLE_RECORD_LOG_SCAN_ID: &str = "durable_record_log_scan.seed_data.current_boot.v0";
pub const RECLOG_MAGIC: &[u8; 8] = b"RAIOSRC0";
pub const RECLOG_SECTOR_SIZE: usize = 512;
pub const RECLOG_FRAME_HEADER_LEN: usize = 88;
pub const RECLOG_PREV_FRAME_SHA256_OFFSET: usize = 24;
pub const RECLOG_PAYLOAD_SHA256_OFFSET: usize = 56;

const RECLOG_MAGIC_OFFSET: usize = 0;
const RECLOG_FRAME_LEN_OFFSET: usize = 8;
const RECLOG_PAYLOAD_LEN_OFFSET: usize = 12;
const RECLOG_SEQ_OFFSET: usize = 16;
const SHA256_LEN: usize = 32;
const MAX_APPEND_PAYLOAD_LEN: usize = 4096 - RECLOG_FRAME_HEADER_LEN;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParsedReclogFrame {
    pub offset: u64,
    pub frame_len: u32,
    pub payload_len: u32,
    pub seq: u64,
    pub frame_sha256: [u8; 32],
    pub payload_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameInvalid {
    pub reason: &'static str,
    pub offset: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordLogScan {
    pub head_seq: u64,
    pub tail_seq: u64,
    pub count: u64,
    pub terminated_reason: &'static str,
    pub torn_tail: bool,
    pub first_invalid_offset: u64,
    pub valid_prefix_chain: bool,
    pub full_region_valid: bool,
    pub head_frame_sha256: Option<[u8; 32]>,
    pub tail_frame_sha256: Option<[u8; 32]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedAppend {
    pub write_offset: u64,
    pub seq: u64,
    pub prev_frame_sha256: [u8; 32],
    pub frame: Vec<u8>,
    pub frame_len: u64,
    pub payload_sha256: [u8; 32],
    pub frame_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppendDenied {
    ReclogNotAppendable,
    DurableStoreFull,
    PayloadTooLargeForFrame,
}

impl AppendDenied {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::ReclogNotAppendable => "reclog_not_appendable",
            Self::DurableStoreFull => "durable_store_full",
            Self::PayloadTooLargeForFrame => "payload_too_large_for_frame",
        }
    }
}

impl RecordLogScan {
    pub const fn not_scanned(reason: &'static str) -> Self {
        Self {
            head_seq: 0,
            tail_seq: 0,
            count: 0,
            terminated_reason: reason,
            torn_tail: false,
            first_invalid_offset: 0,
            valid_prefix_chain: false,
            full_region_valid: false,
            head_frame_sha256: None,
            tail_frame_sha256: None,
        }
    }

    pub fn status(self) -> &'static str {
        if self.full_region_valid && self.count == 0 {
            "valid_empty"
        } else if self.full_region_valid {
            "valid"
        } else if self.torn_tail {
            "torn_tail"
        } else {
            "invalid"
        }
    }
}

pub fn parse_reclog_frame(
    bytes: &[u8],
    offset: usize,
    expected_seq: u64,
    expected_prev_frame_sha256: [u8; 32],
) -> Result<ParsedReclogFrame, FrameInvalid> {
    if offset
        .checked_add(RECLOG_FRAME_HEADER_LEN)
        .map(|end| end > bytes.len())
        .unwrap_or(true)
    {
        return Err(invalid(offset, "truncated_frame_header"));
    }
    let header = &bytes[offset..offset + RECLOG_FRAME_HEADER_LEN];
    if &header[RECLOG_MAGIC_OFFSET..RECLOG_MAGIC_OFFSET + RECLOG_MAGIC.len()] != RECLOG_MAGIC {
        return Err(invalid(offset, "bad_magic"));
    }

    let frame_len = read_u32(header, RECLOG_FRAME_LEN_OFFSET) as usize;
    let payload_len = read_u32(header, RECLOG_PAYLOAD_LEN_OFFSET) as usize;
    let seq = read_u64(header, RECLOG_SEQ_OFFSET);
    if frame_len < RECLOG_SECTOR_SIZE || frame_len % RECLOG_SECTOR_SIZE != 0 {
        return Err(invalid(offset, "bad_frame_len"));
    }
    if offset
        .checked_add(frame_len)
        .map(|end| end > bytes.len())
        .unwrap_or(true)
    {
        return Err(invalid(offset, "frame_len_out_of_bounds"));
    }
    let body_len = frame_len - RECLOG_FRAME_HEADER_LEN;
    if payload_len > body_len {
        return Err(invalid(offset, "payload_len_out_of_bounds"));
    }
    if seq != expected_seq {
        return Err(invalid(offset, "bad_seq"));
    }

    let frame = &bytes[offset..offset + frame_len];
    if frame[RECLOG_PREV_FRAME_SHA256_OFFSET..RECLOG_PREV_FRAME_SHA256_OFFSET + SHA256_LEN]
        != expected_prev_frame_sha256
    {
        return Err(invalid(offset, "bad_prev_frame_sha256"));
    }

    let payload_start = RECLOG_FRAME_HEADER_LEN;
    let payload_end = payload_start + payload_len;
    let payload_sha256 = sha256_bytes(&frame[payload_start..payload_end]);
    if frame[RECLOG_PAYLOAD_SHA256_OFFSET..RECLOG_PAYLOAD_SHA256_OFFSET + SHA256_LEN]
        != payload_sha256
    {
        return Err(invalid(offset, "bad_payload_sha256"));
    }
    if !all_zero(&frame[payload_end..]) {
        return Err(invalid(offset, "frame_padding_nonzero"));
    }

    Ok(ParsedReclogFrame {
        offset: offset as u64,
        frame_len: frame_len as u32,
        payload_len: payload_len as u32,
        seq,
        frame_sha256: sha256_bytes(frame),
        payload_sha256,
    })
}

pub fn plan_reclog_append(
    scan: &RecordLogScan,
    payload: &[u8],
    reclog_byte_count: u64,
) -> Result<PlannedAppend, AppendDenied> {
    if !scan.full_region_valid || scan.first_invalid_offset % RECLOG_SECTOR_SIZE as u64 != 0 {
        return Err(AppendDenied::ReclogNotAppendable);
    }
    if payload.len() > MAX_APPEND_PAYLOAD_LEN {
        return Err(AppendDenied::PayloadTooLargeForFrame);
    }

    let write_offset = scan.first_invalid_offset;
    let seq = if scan.count == 0 {
        1
    } else {
        scan.tail_seq
            .checked_add(1)
            .ok_or(AppendDenied::ReclogNotAppendable)?
    };
    let prev_frame_sha256 = scan.tail_frame_sha256.unwrap_or([0u8; 32]);
    let frame_len = rounded_frame_len(payload.len())?;
    let end = write_offset
        .checked_add(frame_len as u64)
        .ok_or(AppendDenied::DurableStoreFull)?;
    if end > reclog_byte_count {
        return Err(AppendDenied::DurableStoreFull);
    }

    let frame = build_reclog_frame(seq, prev_frame_sha256, payload, frame_len);
    let payload_sha256 = sha256_bytes(payload);
    let frame_sha256 = sha256_bytes(&frame);
    Ok(PlannedAppend {
        write_offset,
        seq,
        prev_frame_sha256,
        frame,
        frame_len: frame_len as u64,
        payload_sha256,
        frame_sha256,
    })
}

pub fn scan_reclog(bytes: &[u8]) -> RecordLogScan {
    if bytes.is_empty() {
        return RecordLogScan {
            terminated_reason: "empty_region",
            full_region_valid: true,
            ..RecordLogScan::not_scanned("empty_region")
        };
    }
    if bytes.len() % RECLOG_SECTOR_SIZE != 0 {
        return RecordLogScan::not_scanned("region_not_sector_aligned");
    }

    let mut offset = 0usize;
    let mut expected_seq = 1u64;
    let mut expected_prev_hash = [0u8; 32];
    let mut scan = RecordLogScan {
        valid_prefix_chain: true,
        ..RecordLogScan::not_scanned("zero_filled")
    };

    while offset < bytes.len() {
        if all_zero(&bytes[offset..]) {
            scan.terminated_reason = "zero_filled";
            scan.first_invalid_offset = offset as u64;
            scan.full_region_valid = true;
            return scan;
        }

        let frame = match parse_reclog_frame(bytes, offset, expected_seq, expected_prev_hash) {
            Ok(frame) => frame,
            Err(err) => {
                scan.terminated_reason = err.reason;
                scan.first_invalid_offset = err.offset;
                scan.torn_tail = scan.count > 0;
                scan.full_region_valid = false;
                return scan;
            }
        };

        if scan.count == 0 {
            scan.head_seq = frame.seq;
            scan.head_frame_sha256 = Some(frame.frame_sha256);
        }
        scan.tail_seq = frame.seq;
        scan.tail_frame_sha256 = Some(frame.frame_sha256);
        scan.count += 1;
        expected_seq = frame.seq.saturating_add(1);
        expected_prev_hash = frame.frame_sha256;
        offset += frame.frame_len as usize;
    }

    scan.terminated_reason = "region_exhausted";
    scan.first_invalid_offset = bytes.len() as u64;
    scan.full_region_valid = true;
    scan
}

pub fn durable_record_log_scan_record(scan: &RecordLogScan) -> Value<'static> {
    Value::Object(durable_record_log_scan_fields(scan))
}

pub fn durable_record_log_scan_fields(scan: &RecordLogScan) -> alloc::vec::Vec<Field<'static>> {
    vec![
        Field::new("schema", Value::Str(DURABLE_RECORD_LOG_SCAN_SCHEMA)),
        Field::new("id", Value::Str(DURABLE_RECORD_LOG_SCAN_ID)),
        Field::new("scope", Value::Str("current_boot")),
        Field::new("classification", Value::Str("local_only")),
        Field::new("status", Value::Str(scan.status())),
        Field::new("terminated_reason", Value::Str(scan.terminated_reason)),
        Field::new("head_seq", Value::U64(scan.head_seq)),
        Field::new("tail_seq", Value::U64(scan.tail_seq)),
        Field::new("count", Value::U64(scan.count)),
        Field::new("torn_tail", Value::Bool(scan.torn_tail)),
        Field::new(
            "first_invalid_offset",
            Value::U64(scan.first_invalid_offset),
        ),
        Field::new("valid_prefix_chain", Value::Bool(scan.valid_prefix_chain)),
        Field::new("full_region_valid", Value::Bool(scan.full_region_valid)),
        Field::new("head_frame_sha256", optional_sha256(scan.head_frame_sha256)),
        Field::new("tail_frame_sha256", optional_sha256(scan.tail_frame_sha256)),
        Field::new("frame_magic", Value::Str("RAIOSRC0")),
        Field::new(
            "frame_header_len",
            Value::U64(RECLOG_FRAME_HEADER_LEN as u64),
        ),
        Field::new("sector_size", Value::U64(RECLOG_SECTOR_SIZE as u64)),
        Field::new("read_only", Value::Bool(true)),
        Field::new("append_result", Value::Str("capability_denied")),
        Field::new("append_authority", Value::Bool(false)),
        Field::new("write_attempted", Value::Bool(false)),
        Field::new("write_dma_ext_called", Value::Bool(false)),
        Field::new("writes_enabled", Value::Bool(false)),
        Field::new("persistence_claimed", Value::Bool(false)),
    ]
}

fn optional_sha256(value: Option<[u8; 32]>) -> Value<'static> {
    match value {
        Some(value) => Value::Sha256(value),
        None => Value::Null,
    }
}

fn rounded_frame_len(payload_len: usize) -> Result<usize, AppendDenied> {
    let needed = RECLOG_FRAME_HEADER_LEN
        .checked_add(payload_len)
        .ok_or(AppendDenied::PayloadTooLargeForFrame)?;
    let rounded = needed
        .checked_add(RECLOG_SECTOR_SIZE - 1)
        .ok_or(AppendDenied::PayloadTooLargeForFrame)?
        / RECLOG_SECTOR_SIZE
        * RECLOG_SECTOR_SIZE;
    Ok(usize::max(RECLOG_SECTOR_SIZE, rounded))
}

fn build_reclog_frame(seq: u64, prev_hash: [u8; 32], payload: &[u8], frame_len: usize) -> Vec<u8> {
    let payload_hash = sha256_bytes(payload);
    let mut out = vec![0u8; frame_len];
    out[RECLOG_MAGIC_OFFSET..RECLOG_MAGIC_OFFSET + RECLOG_MAGIC.len()]
        .copy_from_slice(RECLOG_MAGIC);
    write_u32(&mut out, RECLOG_FRAME_LEN_OFFSET, frame_len as u32);
    write_u32(&mut out, RECLOG_PAYLOAD_LEN_OFFSET, payload.len() as u32);
    write_u64(&mut out, RECLOG_SEQ_OFFSET, seq);
    out[RECLOG_PREV_FRAME_SHA256_OFFSET..RECLOG_PREV_FRAME_SHA256_OFFSET + SHA256_LEN]
        .copy_from_slice(&prev_hash);
    out[RECLOG_PAYLOAD_SHA256_OFFSET..RECLOG_PAYLOAD_SHA256_OFFSET + SHA256_LEN]
        .copy_from_slice(&payload_hash);
    out[RECLOG_FRAME_HEADER_LEN..RECLOG_FRAME_HEADER_LEN + payload.len()].copy_from_slice(payload);
    out
}

fn invalid(offset: usize, reason: &'static str) -> FrameInvalid {
    FrameInvalid {
        reason,
        offset: offset as u64,
    }
}

fn all_zero(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| *byte == 0)
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    (bytes[offset] as u32)
        | ((bytes[offset + 1] as u32) << 8)
        | ((bytes[offset + 2] as u32) << 16)
        | ((bytes[offset + 3] as u32) << 24)
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    (read_u32(bytes, offset) as u64) | ((read_u32(bytes, offset + 4) as u64) << 32)
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset] = value as u8;
    bytes[offset + 1] = (value >> 8) as u8;
    bytes[offset + 2] = (value >> 16) as u8;
    bytes[offset + 3] = (value >> 24) as u8;
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    write_u32(bytes, offset, value as u32);
    write_u32(bytes, offset + 4, (value >> 32) as u32);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::write_json;
    use std::{format, vec::Vec};

    fn render(value: &Value<'_>) -> Vec<u8> {
        let mut out = Vec::new();
        write_json(value, &mut out, 0);
        out
    }

    fn payload(seq: u64) -> Vec<u8> {
        format!(
            "{{\r\n  \"fixture\": \"m7b-reclog\",\r\n  \"seq\": {}\r\n}}",
            seq
        )
        .into_bytes()
    }

    fn frame(seq: u64, prev_hash: [u8; 32], frame_len: usize) -> (Vec<u8>, [u8; 32]) {
        let payload = payload(seq);
        let out = build_reclog_frame(seq, prev_hash, &payload, frame_len);
        let hash = sha256_bytes(&out);
        (out, hash)
    }

    fn region_with_frames(count: u64, frame_len: usize) -> Vec<u8> {
        let mut region = Vec::new();
        let mut prev_hash = [0u8; 32];
        for seq in 1..=count {
            let (frame, hash) = frame(seq, prev_hash, frame_len);
            region.extend_from_slice(&frame);
            prev_hash = hash;
        }
        region.resize(region.len() + RECLOG_SECTOR_SIZE, 0);
        region
    }

    #[test]
    fn empty_log_all_zero_is_valid_empty() {
        let scan = scan_reclog(&vec![0u8; RECLOG_SECTOR_SIZE * 2]);

        assert_eq!(scan.status(), "valid_empty");
        assert_eq!(scan.count, 0);
        assert_eq!(scan.head_seq, 0);
        assert_eq!(scan.tail_seq, 0);
        assert_eq!(scan.terminated_reason, "zero_filled");
        assert!(!scan.torn_tail);
        assert!(scan.full_region_valid);
    }

    #[test]
    fn scans_valid_chained_records() {
        let region = region_with_frames(3, RECLOG_SECTOR_SIZE);
        let scan = scan_reclog(&region);

        assert_eq!(scan.status(), "valid");
        assert_eq!(scan.count, 3);
        assert_eq!(scan.head_seq, 1);
        assert_eq!(scan.tail_seq, 3);
        assert_eq!(scan.first_invalid_offset, (RECLOG_SECTOR_SIZE * 3) as u64);
        assert!(!scan.torn_tail);
        assert!(scan.head_frame_sha256.is_some());
        assert!(scan.tail_frame_sha256.is_some());
    }

    #[test]
    fn bad_magic_stops_at_first_frame() {
        let mut region = region_with_frames(1, RECLOG_SECTOR_SIZE);
        region[0] = b'X';
        let scan = scan_reclog(&region);

        assert_eq!(scan.status(), "invalid");
        assert_eq!(scan.count, 0);
        assert_eq!(scan.terminated_reason, "bad_magic");
        assert_eq!(scan.first_invalid_offset, 0);
        assert!(!scan.torn_tail);
    }

    #[test]
    fn bad_payload_hash_stops_scan() {
        let mut region = region_with_frames(1, RECLOG_SECTOR_SIZE);
        region[RECLOG_PAYLOAD_SHA256_OFFSET] ^= 1;
        let scan = scan_reclog(&region);

        assert_eq!(scan.status(), "invalid");
        assert_eq!(scan.count, 0);
        assert_eq!(scan.terminated_reason, "bad_payload_sha256");
    }

    #[test]
    fn bad_prev_hash_stops_after_valid_prefix() {
        let mut region = region_with_frames(2, RECLOG_SECTOR_SIZE);
        region[RECLOG_SECTOR_SIZE + RECLOG_PREV_FRAME_SHA256_OFFSET] ^= 1;
        let scan = scan_reclog(&region);

        assert_eq!(scan.status(), "torn_tail");
        assert_eq!(scan.count, 1);
        assert_eq!(scan.terminated_reason, "bad_prev_frame_sha256");
        assert_eq!(scan.first_invalid_offset, RECLOG_SECTOR_SIZE as u64);
        assert!(scan.torn_tail);
    }

    #[test]
    fn bad_seq_gap_and_duplicate_stop_after_valid_prefix() {
        for bad_seq in [1u64, 3u64] {
            let mut region = region_with_frames(2, RECLOG_SECTOR_SIZE);
            write_u64(&mut region, RECLOG_SECTOR_SIZE + RECLOG_SEQ_OFFSET, bad_seq);
            let scan = scan_reclog(&region);

            assert_eq!(scan.status(), "torn_tail");
            assert_eq!(scan.count, 1);
            assert_eq!(scan.terminated_reason, "bad_seq");
            assert!(scan.torn_tail);
        }
    }

    #[test]
    fn valid_prefix_plus_garbage_is_torn_tail() {
        let mut region = region_with_frames(2, RECLOG_SECTOR_SIZE);
        let invalid_offset = RECLOG_SECTOR_SIZE * 2;
        region[invalid_offset..invalid_offset + 8].copy_from_slice(b"GARBAGE!");
        let scan = scan_reclog(&region);

        assert_eq!(scan.status(), "torn_tail");
        assert_eq!(scan.count, 2);
        assert_eq!(scan.terminated_reason, "bad_magic");
        assert_eq!(scan.first_invalid_offset, invalid_offset as u64);
        assert!(scan.torn_tail);
    }

    #[test]
    fn accepts_multi_sector_frame() {
        let region = region_with_frames(1, RECLOG_SECTOR_SIZE * 2);
        let scan = scan_reclog(&region);

        assert_eq!(scan.status(), "valid");
        assert_eq!(scan.count, 1);
        assert_eq!(scan.tail_seq, 1);
        assert_eq!(scan.first_invalid_offset, (RECLOG_SECTOR_SIZE * 2) as u64);
    }

    #[test]
    fn record_renders_read_only_denied_append_shape() {
        let scan = scan_reclog(&vec![0u8; RECLOG_SECTOR_SIZE]);

        assert_eq!(
            render(&durable_record_log_scan_record(&scan)),
            b"{\r\n  \"schema\": \"raios.durable_record_log_scan.v0\",\r\n  \"id\": \"durable_record_log_scan.seed_data.current_boot.v0\",\r\n  \"scope\": \"current_boot\",\r\n  \"classification\": \"local_only\",\r\n  \"status\": \"valid_empty\",\r\n  \"terminated_reason\": \"zero_filled\",\r\n  \"head_seq\": 0,\r\n  \"tail_seq\": 0,\r\n  \"count\": 0,\r\n  \"torn_tail\": false,\r\n  \"first_invalid_offset\": 0,\r\n  \"valid_prefix_chain\": true,\r\n  \"full_region_valid\": true,\r\n  \"head_frame_sha256\": null,\r\n  \"tail_frame_sha256\": null,\r\n  \"frame_magic\": \"RAIOSRC0\",\r\n  \"frame_header_len\": 88,\r\n  \"sector_size\": 512,\r\n  \"read_only\": true,\r\n  \"append_result\": \"capability_denied\",\r\n  \"append_authority\": false,\r\n  \"write_attempted\": false,\r\n  \"write_dma_ext_called\": false,\r\n  \"writes_enabled\": false,\r\n  \"persistence_claimed\": false\r\n}"
        );
    }

    #[test]
    fn append_plan_empty_region_starts_seq_one_at_offset_zero() {
        let region = vec![0u8; RECLOG_SECTOR_SIZE * 2];
        let scan = scan_reclog(&region);
        let planned = plan_reclog_append(&scan, b"first", region.len() as u64).unwrap();

        assert_eq!(planned.seq, 1);
        assert_eq!(planned.prev_frame_sha256, [0u8; 32]);
        assert_eq!(planned.write_offset, 0);
        assert_eq!(planned.frame_len, RECLOG_SECTOR_SIZE as u64);
        assert_eq!(planned.payload_sha256, sha256_bytes(b"first"));
        assert_eq!(planned.frame_sha256, sha256_bytes(&planned.frame));
    }

    #[test]
    fn append_plan_chains_after_tail_frame() {
        let region = region_with_frames(3, RECLOG_SECTOR_SIZE);
        let scan = scan_reclog(&region);
        let planned = plan_reclog_append(&scan, b"next", region.len() as u64).unwrap();

        assert_eq!(planned.seq, 4);
        assert_eq!(planned.prev_frame_sha256, scan.tail_frame_sha256.unwrap());
        assert_eq!(planned.write_offset, scan.first_invalid_offset);
    }

    #[test]
    fn append_plan_denies_full_store_but_accepts_exact_boundary() {
        let region = region_with_frames(1, RECLOG_SECTOR_SIZE);
        let scan = scan_reclog(&region);
        let planned = plan_reclog_append(&scan, b"fits", region.len() as u64).unwrap();

        assert_eq!(
            plan_reclog_append(&scan, b"fits", planned.write_offset + planned.frame_len - 1)
                .unwrap_err(),
            AppendDenied::DurableStoreFull
        );
        assert!(
            plan_reclog_append(&scan, b"fits", planned.write_offset + planned.frame_len).is_ok()
        );
    }

    #[test]
    fn append_plan_builds_multisector_frame_when_payload_crosses_one_sector() {
        let region = vec![0u8; RECLOG_SECTOR_SIZE * 3];
        let scan = scan_reclog(&region);
        let payload = vec![7u8; RECLOG_SECTOR_SIZE - RECLOG_FRAME_HEADER_LEN + 1];
        let planned = plan_reclog_append(&scan, &payload, region.len() as u64).unwrap();

        assert_eq!(planned.frame_len, (RECLOG_SECTOR_SIZE * 2) as u64);
    }

    #[test]
    fn append_plan_round_trips_through_scan_and_parse() {
        let mut region = region_with_frames(2, RECLOG_SECTOR_SIZE);
        let before = scan_reclog(&region);
        let planned = plan_reclog_append(&before, b"round-trip", region.len() as u64).unwrap();
        let offset = planned.write_offset as usize;
        region[offset..offset + planned.frame.len()].copy_from_slice(&planned.frame);

        let after = scan_reclog(&region);
        assert_eq!(after.count, before.count + 1);
        assert_eq!(after.tail_seq, planned.seq);
        assert_eq!(after.tail_frame_sha256, Some(planned.frame_sha256));
        assert!(
            parse_reclog_frame(&region, offset, planned.seq, planned.prev_frame_sha256).is_ok()
        );
    }

    #[test]
    fn append_plan_denies_torn_tail_and_oversized_payload() {
        let mut region = region_with_frames(1, RECLOG_SECTOR_SIZE);
        region[RECLOG_SECTOR_SIZE..RECLOG_SECTOR_SIZE + 8].copy_from_slice(b"GARBAGE!");
        let scan = scan_reclog(&region);

        assert_eq!(
            plan_reclog_append(&scan, b"next", region.len() as u64).unwrap_err(),
            AppendDenied::ReclogNotAppendable
        );
        assert_eq!(
            plan_reclog_append(
                &scan_reclog(&vec![0u8; RECLOG_SECTOR_SIZE * 10]),
                &vec![1u8; MAX_APPEND_PAYLOAD_LEN + 1],
                (RECLOG_SECTOR_SIZE * 10) as u64,
            )
            .unwrap_err(),
            AppendDenied::PayloadTooLargeForFrame
        );
    }
}
