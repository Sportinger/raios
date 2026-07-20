use alloc::vec;

use crate::{
    gpt_layout::LayoutStatus,
    record::{Field, Value},
    sha256_bytes,
};

pub const DATA_LAYOUT_SCHEMA: &str = "raios.data_layout.v0";
pub const DATA_LAYOUT_ID: &str = "data_layout.current_boot.v0";
pub const SUPERBLOCK_MAGIC: &[u8; 16] = b"RAIOS_DATA_SB_V0";
pub const SUPERBLOCK_VERSION: u32 = 0;
pub const SUPERBLOCK_HEADER_LEN: usize = 128;
pub const SUPERBLOCK_HASH_OFFSET: usize = SUPERBLOCK_HEADER_LEN;
pub const SUPERBLOCK_HASH_LEN: usize = 32;
pub const SUPERBLOCK_REGION_TABLE_OFFSET: usize = 48;
pub const SUPERBLOCK_REGION_ENTRY_LEN: usize = 24;
pub const SECTOR_SIZE: usize = 512;

pub const BOOTCTL_START_LBA: u64 = 2;
pub const BOOTCTL_LBA_COUNT: u64 = 8;
pub const RECLOG_START_LBA: u64 = 16;
pub const RECLOG_LBA_COUNT: u64 = 4096;
pub const ARTSTOR_START_LBA: u64 = 8192;

const REGION_COUNT: u32 = 3;
const BOOTCTL_TAG: &[u8; 8] = b"BOOTCTL\0";
const RECLOG_TAG: &[u8; 8] = b"RECLOG\0\0";
const ARTSTOR_TAG: &[u8; 8] = b"ARTSTOR\0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DataLayoutEvidence {
    pub status: LayoutStatus,
    pub reason: &'static str,
    pub magic_valid: bool,
    pub version: u32,
    pub data_lba_count: u64,
    pub superblock_valid: bool,
    pub superblock_copy_valid: bool,
    pub superblock_lba1_copy_matches: bool,
    pub header_sha256_checked: bool,
    pub bootctl_start_lba: u64,
    pub bootctl_lba_count: u64,
    pub reclog_start_lba: u64,
    pub reclog_lba_count: u64,
    pub artstor_start_lba: u64,
    pub artstor_lba_count: u64,
}

impl DataLayoutEvidence {
    pub const fn absent(reason: &'static str) -> Self {
        Self {
            status: LayoutStatus::Absent,
            reason,
            magic_valid: false,
            version: 0,
            data_lba_count: 0,
            superblock_valid: false,
            superblock_copy_valid: false,
            superblock_lba1_copy_matches: false,
            header_sha256_checked: false,
            bootctl_start_lba: 0,
            bootctl_lba_count: 0,
            reclog_start_lba: 0,
            reclog_lba_count: 0,
            artstor_start_lba: 0,
            artstor_lba_count: 0,
        }
    }

    pub const fn invalid(reason: &'static str, data_lba_count: u64) -> Self {
        Self {
            status: LayoutStatus::Invalid,
            reason,
            data_lba_count,
            ..Self::absent(reason)
        }
    }
}

pub fn validate_seed_data_layout(
    superblock_lba0: &[u8],
    superblock_lba1: &[u8],
    expected_data_lba_count: u64,
) -> DataLayoutEvidence {
    let primary = match parse_superblock(superblock_lba0, expected_data_lba_count) {
        Ok(primary) => primary,
        Err(reason) => return DataLayoutEvidence::invalid(reason, expected_data_lba_count),
    };
    if superblock_lba0.len() < SECTOR_SIZE || superblock_lba1.len() < SECTOR_SIZE {
        return DataLayoutEvidence::invalid("superblock_copy_truncated", expected_data_lba_count);
    }
    if &superblock_lba0[..SECTOR_SIZE] != &superblock_lba1[..SECTOR_SIZE] {
        return DataLayoutEvidence {
            magic_valid: true,
            version: primary.version,
            data_lba_count: primary.data_lba_count,
            superblock_valid: true,
            header_sha256_checked: true,
            bootctl_start_lba: primary.bootctl_start_lba,
            bootctl_lba_count: primary.bootctl_lba_count,
            reclog_start_lba: primary.reclog_start_lba,
            reclog_lba_count: primary.reclog_lba_count,
            artstor_start_lba: primary.artstor_start_lba,
            artstor_lba_count: primary.artstor_lba_count,
            ..DataLayoutEvidence::invalid("superblock_lba1_copy_mismatch", expected_data_lba_count)
        };
    }
    if let Err(reason) = parse_superblock(superblock_lba1, expected_data_lba_count) {
        return DataLayoutEvidence {
            magic_valid: true,
            version: primary.version,
            data_lba_count: primary.data_lba_count,
            superblock_valid: true,
            header_sha256_checked: true,
            bootctl_start_lba: primary.bootctl_start_lba,
            bootctl_lba_count: primary.bootctl_lba_count,
            reclog_start_lba: primary.reclog_start_lba,
            reclog_lba_count: primary.reclog_lba_count,
            artstor_start_lba: primary.artstor_start_lba,
            artstor_lba_count: primary.artstor_lba_count,
            ..DataLayoutEvidence::invalid(reason, expected_data_lba_count)
        };
    }

    DataLayoutEvidence {
        status: LayoutStatus::Present,
        reason: "seed_data_superblock_validated",
        magic_valid: true,
        version: primary.version,
        data_lba_count: primary.data_lba_count,
        superblock_valid: true,
        superblock_copy_valid: true,
        superblock_lba1_copy_matches: true,
        header_sha256_checked: true,
        bootctl_start_lba: primary.bootctl_start_lba,
        bootctl_lba_count: primary.bootctl_lba_count,
        reclog_start_lba: primary.reclog_start_lba,
        reclog_lba_count: primary.reclog_lba_count,
        artstor_start_lba: primary.artstor_start_lba,
        artstor_lba_count: primary.artstor_lba_count,
    }
}

pub fn data_layout_record(evidence: &DataLayoutEvidence) -> Value<'static> {
    Value::Object(vec![
        Field::new("schema", Value::Str(DATA_LAYOUT_SCHEMA)),
        Field::new("id", Value::Str(DATA_LAYOUT_ID)),
        Field::new("scope", Value::Str("current_boot")),
        Field::new("classification", Value::Str("local_only")),
        Field::new("status", Value::Str(evidence.status.as_str())),
        Field::new("reason", Value::Str(evidence.reason)),
        Field::new("magic_valid", Value::Bool(evidence.magic_valid)),
        Field::new("version", Value::U64(evidence.version as u64)),
        Field::new("data_lba_count", Value::U64(evidence.data_lba_count)),
        Field::new("superblock_valid", Value::Bool(evidence.superblock_valid)),
        Field::new(
            "superblock_copy_valid",
            Value::Bool(evidence.superblock_copy_valid),
        ),
        Field::new(
            "superblock_lba1_copy_matches",
            Value::Bool(evidence.superblock_lba1_copy_matches),
        ),
        Field::new(
            "header_sha256_checked",
            Value::Bool(evidence.header_sha256_checked),
        ),
        Field::new("bootctl_start_lba", Value::U64(evidence.bootctl_start_lba)),
        Field::new("bootctl_lba_count", Value::U64(evidence.bootctl_lba_count)),
        Field::new("reclog_start_lba", Value::U64(evidence.reclog_start_lba)),
        Field::new("reclog_lba_count", Value::U64(evidence.reclog_lba_count)),
        Field::new("artstor_start_lba", Value::U64(evidence.artstor_start_lba)),
        Field::new("artstor_lba_count", Value::U64(evidence.artstor_lba_count)),
        Field::new("read_only", Value::Bool(true)),
        Field::new("write_attempted", Value::Bool(false)),
        Field::new("writes_enabled", Value::Bool(false)),
        Field::new("persistence_claimed", Value::Bool(false)),
    ])
}

struct ParsedSuperblock {
    version: u32,
    data_lba_count: u64,
    bootctl_start_lba: u64,
    bootctl_lba_count: u64,
    reclog_start_lba: u64,
    reclog_lba_count: u64,
    artstor_start_lba: u64,
    artstor_lba_count: u64,
}

fn parse_superblock(
    sector: &[u8],
    expected_data_lba_count: u64,
) -> Result<ParsedSuperblock, &'static str> {
    if sector.len() < SECTOR_SIZE {
        return Err("superblock_truncated");
    }
    if &sector[..16] != SUPERBLOCK_MAGIC {
        return Err("superblock_magic_mismatch");
    }
    let version = read_u32(sector, 16);
    if version != SUPERBLOCK_VERSION {
        return Err("superblock_version_mismatch");
    }
    if read_u32(sector, 20) as usize != SUPERBLOCK_HEADER_LEN {
        return Err("superblock_header_len_mismatch");
    }
    if read_u32(sector, 24) != REGION_COUNT {
        return Err("superblock_region_count_mismatch");
    }
    if read_u32(sector, 28) as usize != SUPERBLOCK_REGION_ENTRY_LEN {
        return Err("superblock_region_entry_len_mismatch");
    }
    let data_lba_count = read_u64(sector, 32);
    if data_lba_count != expected_data_lba_count {
        return Err("superblock_data_lba_count_mismatch");
    }
    let computed_hash = sha256_bytes(&sector[..SUPERBLOCK_HEADER_LEN]);
    if sector[SUPERBLOCK_HASH_OFFSET..SUPERBLOCK_HASH_OFFSET + SUPERBLOCK_HASH_LEN] != computed_hash
    {
        return Err("superblock_header_sha256_mismatch");
    }

    let bootctl = parse_region(sector, 0, BOOTCTL_TAG, BOOTCTL_START_LBA, BOOTCTL_LBA_COUNT)?;
    let reclog = parse_region(sector, 1, RECLOG_TAG, RECLOG_START_LBA, RECLOG_LBA_COUNT)?;
    let artstor_lba_count = data_lba_count
        .checked_sub(ARTSTOR_START_LBA)
        .ok_or("superblock_artstor_range_invalid")?;
    let artstor = parse_region(sector, 2, ARTSTOR_TAG, ARTSTOR_START_LBA, artstor_lba_count)?;

    Ok(ParsedSuperblock {
        version,
        data_lba_count,
        bootctl_start_lba: bootctl.0,
        bootctl_lba_count: bootctl.1,
        reclog_start_lba: reclog.0,
        reclog_lba_count: reclog.1,
        artstor_start_lba: artstor.0,
        artstor_lba_count: artstor.1,
    })
}

fn parse_region(
    sector: &[u8],
    index: usize,
    expected_tag: &[u8; 8],
    expected_start_lba: u64,
    expected_lba_count: u64,
) -> Result<(u64, u64), &'static str> {
    let offset = SUPERBLOCK_REGION_TABLE_OFFSET + index * SUPERBLOCK_REGION_ENTRY_LEN;
    if &sector[offset..offset + 8] != expected_tag {
        return Err("superblock_region_tag_mismatch");
    }
    let start_lba = read_u64(sector, offset + 8);
    let lba_count = read_u64(sector, offset + 16);
    if start_lba != expected_start_lba || lba_count != expected_lba_count {
        return Err("superblock_region_span_mismatch");
    }
    Ok((start_lba, lba_count))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::write_json;

    const DATA_LBA_COUNT: u64 = 524_288;

    fn render(value: &Value<'_>) -> Vec<u8> {
        let mut out = Vec::new();
        write_json(value, &mut out, 0);
        out
    }

    fn valid_superblock() -> [u8; SECTOR_SIZE] {
        let mut sector = [0u8; SECTOR_SIZE];
        sector[..16].copy_from_slice(SUPERBLOCK_MAGIC);
        write_u32(&mut sector, 16, SUPERBLOCK_VERSION);
        write_u32(&mut sector, 20, SUPERBLOCK_HEADER_LEN as u32);
        write_u32(&mut sector, 24, REGION_COUNT);
        write_u32(&mut sector, 28, SUPERBLOCK_REGION_ENTRY_LEN as u32);
        write_u64(&mut sector, 32, DATA_LBA_COUNT);
        write_region(
            &mut sector,
            0,
            BOOTCTL_TAG,
            BOOTCTL_START_LBA,
            BOOTCTL_LBA_COUNT,
        );
        write_region(
            &mut sector,
            1,
            RECLOG_TAG,
            RECLOG_START_LBA,
            RECLOG_LBA_COUNT,
        );
        write_region(
            &mut sector,
            2,
            ARTSTOR_TAG,
            ARTSTOR_START_LBA,
            DATA_LBA_COUNT - ARTSTOR_START_LBA,
        );
        let hash = sha256_bytes(&sector[..SUPERBLOCK_HEADER_LEN]);
        sector[SUPERBLOCK_HASH_OFFSET..SUPERBLOCK_HASH_OFFSET + SUPERBLOCK_HASH_LEN]
            .copy_from_slice(&hash);
        sector
    }

    fn write_region(
        sector: &mut [u8],
        index: usize,
        tag: &[u8; 8],
        start_lba: u64,
        lba_count: u64,
    ) {
        let offset = SUPERBLOCK_REGION_TABLE_OFFSET + index * SUPERBLOCK_REGION_ENTRY_LEN;
        sector[offset..offset + 8].copy_from_slice(tag);
        write_u64(sector, offset + 8, start_lba);
        write_u64(sector, offset + 16, lba_count);
    }

    #[test]
    fn valid_superblock_and_copy_are_present() {
        let sb = valid_superblock();
        let evidence = validate_seed_data_layout(&sb, &sb, DATA_LBA_COUNT);

        assert_eq!(evidence.status, LayoutStatus::Present);
        assert!(evidence.superblock_valid);
        assert!(evidence.superblock_copy_valid);
        assert!(evidence.superblock_lba1_copy_matches);
        assert!(evidence.header_sha256_checked);
        assert_eq!(evidence.bootctl_start_lba, BOOTCTL_START_LBA);
        assert_eq!(evidence.reclog_lba_count, RECLOG_LBA_COUNT);
        assert_eq!(
            evidence.artstor_lba_count,
            DATA_LBA_COUNT - ARTSTOR_START_LBA
        );
    }

    #[test]
    fn superblock_hash_mismatch_is_invalid() {
        let mut sb = valid_superblock();
        sb[64] ^= 1;
        let evidence = validate_seed_data_layout(&sb, &sb, DATA_LBA_COUNT);

        assert_eq!(evidence.status, LayoutStatus::Invalid);
        assert_eq!(evidence.reason, "superblock_header_sha256_mismatch");
    }

    #[test]
    fn lba1_copy_must_match_lba0() {
        let sb0 = valid_superblock();
        let mut sb1 = valid_superblock();
        sb1[200] = 1;
        let evidence = validate_seed_data_layout(&sb0, &sb1, DATA_LBA_COUNT);

        assert_eq!(evidence.status, LayoutStatus::Invalid);
        assert_eq!(evidence.reason, "superblock_lba1_copy_mismatch");
        assert!(evidence.superblock_valid);
        assert!(!evidence.superblock_lba1_copy_matches);
    }

    #[test]
    fn data_layout_record_renders_expected_shape() {
        let sb = valid_superblock();
        let evidence = validate_seed_data_layout(&sb, &sb, DATA_LBA_COUNT);

        assert_eq!(
            render(&data_layout_record(&evidence)),
            b"{\r\n  \"schema\": \"raios.data_layout.v0\",\r\n  \"id\": \"data_layout.current_boot.v0\",\r\n  \"scope\": \"current_boot\",\r\n  \"classification\": \"local_only\",\r\n  \"status\": \"present\",\r\n  \"reason\": \"seed_data_superblock_validated\",\r\n  \"magic_valid\": true,\r\n  \"version\": 0,\r\n  \"data_lba_count\": 524288,\r\n  \"superblock_valid\": true,\r\n  \"superblock_copy_valid\": true,\r\n  \"superblock_lba1_copy_matches\": true,\r\n  \"header_sha256_checked\": true,\r\n  \"bootctl_start_lba\": 2,\r\n  \"bootctl_lba_count\": 8,\r\n  \"reclog_start_lba\": 16,\r\n  \"reclog_lba_count\": 4096,\r\n  \"artstor_start_lba\": 8192,\r\n  \"artstor_lba_count\": 516096,\r\n  \"read_only\": true,\r\n  \"write_attempted\": false,\r\n  \"writes_enabled\": false,\r\n  \"persistence_claimed\": false\r\n}"
        );
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
}
