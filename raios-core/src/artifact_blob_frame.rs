use alloc::{vec, vec::Vec};

use crate::sha256_bytes;

pub const ARTIFACT_BLOB_MAGIC: &[u8; 8] = b"RAIOSAR0";
pub const ARTIFACT_BLOB_SECTOR_SIZE: usize = 512;
pub const ARTIFACT_BLOB_FRAME_HEADER_LEN: usize = 48;
pub const ARTIFACT_BLOB_PAYLOAD_SHA256_OFFSET: usize = 16;

const ARTIFACT_BLOB_MAGIC_OFFSET: usize = 0;
const ARTIFACT_BLOB_FRAME_LEN_OFFSET: usize = 8;
const ARTIFACT_BLOB_PAYLOAD_LEN_OFFSET: usize = 12;
const SHA256_LEN: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParsedArtifactBlobFrame {
    pub offset: u64,
    pub frame_len: u32,
    pub payload_len: u32,
    pub payload_sha256: [u8; 32],
    pub frame_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedArtifactBlobWrite {
    pub write_offset: u64,
    pub frame: Vec<u8>,
    pub frame_len: u64,
    pub payload_sha256: [u8; 32],
    pub frame_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactBlobDenied {
    ArtifactStoreFull,
    PayloadTooLarge,
}

impl ArtifactBlobDenied {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::ArtifactStoreFull => "artifact_store_full",
            Self::PayloadTooLarge => "payload_too_large",
        }
    }
}

pub fn parse_artifact_blob_frame(
    bytes: &[u8],
    offset: usize,
) -> Result<ParsedArtifactBlobFrame, &'static str> {
    if offset
        .checked_add(ARTIFACT_BLOB_FRAME_HEADER_LEN)
        .map(|end| end > bytes.len())
        .unwrap_or(true)
    {
        return Err("truncated_frame_header");
    }
    let header = &bytes[offset..offset + ARTIFACT_BLOB_FRAME_HEADER_LEN];
    if &header[ARTIFACT_BLOB_MAGIC_OFFSET..ARTIFACT_BLOB_MAGIC_OFFSET + ARTIFACT_BLOB_MAGIC.len()]
        != ARTIFACT_BLOB_MAGIC
    {
        return Err("bad_magic");
    }

    let frame_len = read_u32(header, ARTIFACT_BLOB_FRAME_LEN_OFFSET) as usize;
    let payload_len = read_u32(header, ARTIFACT_BLOB_PAYLOAD_LEN_OFFSET) as usize;
    if frame_len < ARTIFACT_BLOB_FRAME_HEADER_LEN || frame_len % ARTIFACT_BLOB_SECTOR_SIZE != 0 {
        return Err("bad_frame_len");
    }
    if offset
        .checked_add(frame_len)
        .map(|end| end > bytes.len())
        .unwrap_or(true)
    {
        return Err("frame_len_out_of_bounds");
    }
    let body_len = frame_len - ARTIFACT_BLOB_FRAME_HEADER_LEN;
    if payload_len > body_len {
        return Err("payload_len_out_of_bounds");
    }

    let frame = &bytes[offset..offset + frame_len];
    let payload_start = ARTIFACT_BLOB_FRAME_HEADER_LEN;
    let payload_end = payload_start + payload_len;
    let payload_sha256 = sha256_bytes(&frame[payload_start..payload_end]);
    if frame[ARTIFACT_BLOB_PAYLOAD_SHA256_OFFSET..ARTIFACT_BLOB_PAYLOAD_SHA256_OFFSET + SHA256_LEN]
        != payload_sha256
    {
        return Err("bad_payload_sha256");
    }
    if !all_zero(&frame[payload_end..]) {
        return Err("frame_padding_nonzero");
    }

    Ok(ParsedArtifactBlobFrame {
        offset: offset as u64,
        frame_len: frame_len as u32,
        payload_len: payload_len as u32,
        payload_sha256,
        frame_sha256: sha256_bytes(frame),
    })
}

pub fn build_artifact_blob_frame(payload: &[u8]) -> Vec<u8> {
    let frame_len = rounded_frame_len(payload.len()).unwrap_or(ARTIFACT_BLOB_SECTOR_SIZE);
    build_artifact_blob_frame_with_len(payload, frame_len)
}

pub fn plan_artifact_blob_write(
    next_free_offset: u64,
    payload: &[u8],
    artstor_byte_count: u64,
) -> Result<PlannedArtifactBlobWrite, ArtifactBlobDenied> {
    if next_free_offset % ARTIFACT_BLOB_SECTOR_SIZE as u64 != 0 {
        return Err(ArtifactBlobDenied::ArtifactStoreFull);
    }

    let frame_len = rounded_frame_len(payload.len()).ok_or(ArtifactBlobDenied::PayloadTooLarge)?;
    let frame_len_u64 = frame_len as u64;
    let end = next_free_offset
        .checked_add(frame_len_u64)
        .ok_or(ArtifactBlobDenied::ArtifactStoreFull)?;
    if end > artstor_byte_count {
        return Err(ArtifactBlobDenied::ArtifactStoreFull);
    }

    let frame = build_artifact_blob_frame_with_len(payload, frame_len);
    let payload_sha256 = sha256_bytes(payload);
    let frame_sha256 = sha256_bytes(&frame);
    Ok(PlannedArtifactBlobWrite {
        write_offset: next_free_offset,
        frame,
        frame_len: frame_len_u64,
        payload_sha256,
        frame_sha256,
    })
}

fn rounded_frame_len(payload_len: usize) -> Option<usize> {
    if payload_len > u32::MAX as usize {
        return None;
    }
    let needed = ARTIFACT_BLOB_FRAME_HEADER_LEN.checked_add(payload_len)?;
    let rounded = needed
        .checked_add(ARTIFACT_BLOB_SECTOR_SIZE - 1)?
        .checked_div(ARTIFACT_BLOB_SECTOR_SIZE)?
        .checked_mul(ARTIFACT_BLOB_SECTOR_SIZE)?;
    if rounded > u32::MAX as usize {
        return None;
    }
    Some(usize::max(ARTIFACT_BLOB_SECTOR_SIZE, rounded))
}

fn build_artifact_blob_frame_with_len(payload: &[u8], frame_len: usize) -> Vec<u8> {
    let payload_hash = sha256_bytes(payload);
    let mut out = vec![0u8; frame_len];
    out[ARTIFACT_BLOB_MAGIC_OFFSET..ARTIFACT_BLOB_MAGIC_OFFSET + ARTIFACT_BLOB_MAGIC.len()]
        .copy_from_slice(ARTIFACT_BLOB_MAGIC);
    write_u32(&mut out, ARTIFACT_BLOB_FRAME_LEN_OFFSET, frame_len as u32);
    write_u32(
        &mut out,
        ARTIFACT_BLOB_PAYLOAD_LEN_OFFSET,
        payload.len() as u32,
    );
    out[ARTIFACT_BLOB_PAYLOAD_SHA256_OFFSET..ARTIFACT_BLOB_PAYLOAD_SHA256_OFFSET + SHA256_LEN]
        .copy_from_slice(&payload_hash);
    out[ARTIFACT_BLOB_FRAME_HEADER_LEN..ARTIFACT_BLOB_FRAME_HEADER_LEN + payload.len()]
        .copy_from_slice(payload);
    out
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

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset] = value as u8;
    bytes[offset + 1] = (value >> 8) as u8;
    bytes[offset + 2] = (value >> 16) as u8;
    bytes[offset + 3] = (value >> 24) as u8;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    #[test]
    fn empty_payload_frame_round_trips() {
        let frame = build_artifact_blob_frame(b"");
        let parsed = parse_artifact_blob_frame(&frame, 0).unwrap();

        assert_eq!(parsed.offset, 0);
        assert_eq!(parsed.frame_len, ARTIFACT_BLOB_SECTOR_SIZE as u32);
        assert_eq!(parsed.payload_len, 0);
        assert_eq!(parsed.payload_sha256, sha256_bytes(b""));
        assert_eq!(parsed.frame_sha256, sha256_bytes(&frame));
    }

    #[test]
    fn parses_multiple_valid_frames_at_offsets() {
        let payloads = [b"one".as_slice(), b"two".as_slice(), b"three".as_slice()];
        let mut region = Vec::new();
        let mut offsets = Vec::new();
        for payload in payloads {
            offsets.push(region.len());
            region.extend_from_slice(&build_artifact_blob_frame(payload));
        }

        let mut idx = 0usize;
        while idx < offsets.len() {
            let parsed = parse_artifact_blob_frame(&region, offsets[idx]).unwrap();
            assert_eq!(parsed.offset, offsets[idx] as u64);
            assert_eq!(parsed.payload_sha256, sha256_bytes(payloads[idx]));
            idx += 1;
        }
    }

    #[test]
    fn rejects_bad_magic() {
        let mut frame = build_artifact_blob_frame(b"artifact");
        frame[0] = b'X';

        assert_eq!(parse_artifact_blob_frame(&frame, 0), Err("bad_magic"));
    }

    #[test]
    fn rejects_bad_payload_hash() {
        let mut frame = build_artifact_blob_frame(b"artifact");
        frame[ARTIFACT_BLOB_PAYLOAD_SHA256_OFFSET] ^= 1;

        assert_eq!(
            parse_artifact_blob_frame(&frame, 0),
            Err("bad_payload_sha256")
        );
    }

    #[test]
    fn rejects_nonzero_padding() {
        let mut frame = build_artifact_blob_frame(b"artifact");
        let parsed = parse_artifact_blob_frame(&frame, 0).unwrap();
        let pad_offset = ARTIFACT_BLOB_FRAME_HEADER_LEN + parsed.payload_len as usize;
        frame[pad_offset] = 1;

        assert_eq!(
            parse_artifact_blob_frame(&frame, 0),
            Err("frame_padding_nonzero")
        );
    }

    #[test]
    fn builds_multi_sector_payload_frames() {
        let two_sector = build_artifact_blob_frame(&vec![
            7u8;
            ARTIFACT_BLOB_SECTOR_SIZE
                - ARTIFACT_BLOB_FRAME_HEADER_LEN
                + 1
        ]);
        let four_sector = build_artifact_blob_frame(&vec![
            9u8;
            ARTIFACT_BLOB_SECTOR_SIZE * 3
                - ARTIFACT_BLOB_FRAME_HEADER_LEN
                + 1
        ]);

        assert_eq!(two_sector.len(), ARTIFACT_BLOB_SECTOR_SIZE * 2);
        assert_eq!(four_sector.len(), ARTIFACT_BLOB_SECTOR_SIZE * 4);
        assert_eq!(
            parse_artifact_blob_frame(&two_sector, 0).unwrap().frame_len,
            (ARTIFACT_BLOB_SECTOR_SIZE * 2) as u32
        );
        assert_eq!(
            parse_artifact_blob_frame(&four_sector, 0)
                .unwrap()
                .frame_len,
            (ARTIFACT_BLOB_SECTOR_SIZE * 4) as u32
        );
    }

    #[test]
    fn plan_accepts_exact_full_boundary() {
        let payload = b"fits exactly";
        let frame = build_artifact_blob_frame(payload);
        let planned =
            plan_artifact_blob_write(0, payload, frame.len() as u64).expect("boundary fits");

        assert_eq!(planned.write_offset, 0);
        assert_eq!(planned.frame_len, frame.len() as u64);
        assert_eq!(planned.payload_sha256, sha256_bytes(payload));
    }

    #[test]
    fn build_parse_round_trip_hashes_are_stable() {
        let payload = b"stable artifact payload";
        let frame = build_artifact_blob_frame(payload);
        let rebuilt = build_artifact_blob_frame(payload);
        let parsed = parse_artifact_blob_frame(&frame, 0).unwrap();

        assert_eq!(frame, rebuilt);
        assert_eq!(parsed.payload_sha256, sha256_bytes(payload));
        assert_eq!(parsed.frame_sha256, sha256_bytes(&rebuilt));
    }

    #[test]
    fn plan_denies_store_full_at_exact_ceiling_minus_one() {
        let payload = b"does not fit";
        let frame_len = build_artifact_blob_frame(payload).len() as u64;

        assert_eq!(
            plan_artifact_blob_write(0, payload, frame_len - 1).unwrap_err(),
            ArtifactBlobDenied::ArtifactStoreFull
        );
        assert!(plan_artifact_blob_write(0, payload, frame_len).is_ok());
    }

    #[test]
    fn denied_reasons_are_distinct() {
        assert_ne!(
            ArtifactBlobDenied::ArtifactStoreFull.reason(),
            ArtifactBlobDenied::PayloadTooLarge.reason()
        );
    }
}
