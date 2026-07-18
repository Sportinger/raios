use alloc::vec;

use crate::record::{Field, Value};

pub const GPT_LAYOUT_SCHEMA: &str = "raios.gpt_layout.v0";
pub const GPT_LAYOUT_ID: &str = "gpt_layout.current_boot.v0";
pub const SECTOR_SIZE: usize = 512;
pub const PRIMARY_GPT_HEADER_LBA: u64 = 1;
pub const PRIMARY_GPT_ENTRIES_LBA: u64 = 2;
pub const GPT_ENTRY_SIZE: usize = 128;
pub const GPT_ENTRY_COUNT: usize = 128;
pub const GPT_ENTRY_ARRAY_BYTES: usize = GPT_ENTRY_SIZE * GPT_ENTRY_COUNT;

const GPT_HEADER_MIN_SIZE: usize = 92;
const MBR_PARTITION_TABLE_OFFSET: usize = 446;
const MBR_SIGNATURE_OFFSET: usize = 510;
const PROTECTIVE_MBR_TYPE: u8 = 0xee;
const GPT_SIGNATURE: &[u8; 8] = b"EFI PART";
const ESP_A_NAME: &str = "SEED_ESP_A";
const ESP_B_NAME: &str = "SEED_ESP_B";
const SEED_DATA_NAME: &str = "SEED_DATA";
const ESP_TYPE_GUID_LE: [u8; 16] = [
    0x28, 0x73, 0x2a, 0xc1, 0x1f, 0xf8, 0xd2, 0x11, 0xba, 0x4b, 0x00, 0xa0, 0xc9, 0x3e, 0xc9, 0x3b,
];
const SEED_DATA_TYPE_GUID_LE: [u8; 16] = [
    0x7a, 0xda, 0xed, 0x5e, 0xde, 0xc0, 0x55, 0x4a, 0x9a, 0x15, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutStatus {
    Present,
    Absent,
    Invalid,
}

impl LayoutStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Absent => "absent",
            Self::Invalid => "invalid",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GptLayoutEvidence {
    pub status: LayoutStatus,
    pub reason: &'static str,
    pub protective_mbr_valid: bool,
    pub gpt_header_valid: bool,
    pub header_crc32_checked: bool,
    pub entry_array_crc32_checked: bool,
    pub esp_a_found: bool,
    pub esp_b_found: bool,
    pub seed_data_found: bool,
    pub seed_data_first_lba: u64,
    pub seed_data_lba_count: u64,
    pub partition_entry_lba: u64,
    pub partition_entry_count: u32,
    pub partition_entry_size: u32,
    pub partition_entry_array_crc32: u32,
}

impl GptLayoutEvidence {
    pub const fn absent(reason: &'static str) -> Self {
        Self {
            status: LayoutStatus::Absent,
            reason,
            protective_mbr_valid: false,
            gpt_header_valid: false,
            header_crc32_checked: false,
            entry_array_crc32_checked: false,
            esp_a_found: false,
            esp_b_found: false,
            seed_data_found: false,
            seed_data_first_lba: 0,
            seed_data_lba_count: 0,
            partition_entry_lba: 0,
            partition_entry_count: 0,
            partition_entry_size: 0,
            partition_entry_array_crc32: 0,
        }
    }

    fn invalid_from(self, reason: &'static str) -> Self {
        Self {
            status: LayoutStatus::Invalid,
            reason,
            ..self
        }
    }
}

struct Header {
    first_usable_lba: u64,
    last_usable_lba: u64,
    entry_lba: u64,
    entry_count: u32,
    entry_size: u32,
    entry_array_crc32: u32,
}

pub fn validate_gpt_layout(
    protective_mbr: &[u8],
    primary_header: &[u8],
    partition_entries: &[u8],
) -> GptLayoutEvidence {
    let header_signature_present =
        primary_header.len() >= 8 && &primary_header[..8] == GPT_SIGNATURE;
    let protective_mbr_valid = protective_mbr_is_valid(protective_mbr);
    if !protective_mbr_valid {
        return if header_signature_present {
            GptLayoutEvidence::absent("protective_mbr_invalid")
                .invalid_from("protective_mbr_invalid")
        } else {
            GptLayoutEvidence::absent("protective_mbr_missing")
        };
    }
    if !header_signature_present {
        return GptLayoutEvidence {
            protective_mbr_valid: true,
            ..GptLayoutEvidence::absent("gpt_header_missing")
        }
        .invalid_from("gpt_header_missing");
    }

    let mut evidence = GptLayoutEvidence {
        protective_mbr_valid: true,
        gpt_header_valid: true,
        ..GptLayoutEvidence::absent("gpt_header_unchecked")
    };
    let header = match parse_header(primary_header) {
        Ok(header) => header,
        Err(reason) => return evidence.invalid_from(reason),
    };
    evidence.header_crc32_checked = true;
    evidence.partition_entry_lba = header.entry_lba;
    evidence.partition_entry_count = header.entry_count;
    evidence.partition_entry_size = header.entry_size;
    evidence.partition_entry_array_crc32 = header.entry_array_crc32;

    if header.entry_lba != PRIMARY_GPT_ENTRIES_LBA {
        return evidence.invalid_from("partition_entry_lba_unexpected");
    }
    if header.entry_count != GPT_ENTRY_COUNT as u32 || header.entry_size != GPT_ENTRY_SIZE as u32 {
        return evidence.invalid_from("partition_entry_shape_unexpected");
    }
    let needed = match (header.entry_count as usize).checked_mul(header.entry_size as usize) {
        Some(value) => value,
        None => return evidence.invalid_from("partition_entry_array_too_large"),
    };
    if partition_entries.len() < needed {
        return evidence.invalid_from("partition_entry_array_truncated");
    }
    if crc32(&partition_entries[..needed]) != header.entry_array_crc32 {
        return evidence.invalid_from("partition_entry_array_crc32_mismatch");
    }
    evidence.entry_array_crc32_checked = true;

    let mut seed_count = 0u8;
    let mut idx = 0usize;
    while idx < header.entry_count as usize {
        let start = idx * header.entry_size as usize;
        let raw = &partition_entries[start..start + GPT_ENTRY_SIZE];
        if raw[..16] == [0u8; 16] {
            idx += 1;
            continue;
        }
        let first_lba = read_u64(raw, 32);
        let last_lba = read_u64(raw, 40);
        if first_lba == 0 || last_lba < first_lba {
            return evidence.invalid_from("partition_lba_range_invalid");
        }
        if first_lba < header.first_usable_lba || last_lba > header.last_usable_lba {
            return evidence.invalid_from("partition_lba_out_of_usable_range");
        }

        let type_guid = &raw[..16];
        let name_is_esp_a = gpt_name_eq(&raw[56..128], ESP_A_NAME);
        let name_is_esp_b = gpt_name_eq(&raw[56..128], ESP_B_NAME);
        let name_is_seed_data = gpt_name_eq(&raw[56..128], SEED_DATA_NAME);
        if name_is_esp_a || name_is_esp_b {
            if type_guid != ESP_TYPE_GUID_LE {
                return evidence.invalid_from("esp_partition_type_guid_mismatch");
            }
            evidence.esp_a_found |= name_is_esp_a;
            evidence.esp_b_found |= name_is_esp_b;
        }
        if type_guid == ESP_TYPE_GUID_LE && !(name_is_esp_a || name_is_esp_b) {
            return evidence.invalid_from("esp_partition_name_mismatch");
        }
        if name_is_seed_data || type_guid == SEED_DATA_TYPE_GUID_LE {
            if !name_is_seed_data || type_guid != SEED_DATA_TYPE_GUID_LE {
                return evidence.invalid_from("seed_data_guid_name_mismatch");
            }
            seed_count = seed_count.saturating_add(1);
            if seed_count > 1 {
                return evidence.invalid_from("duplicate_seed_data_partition");
            }
            evidence.seed_data_found = true;
            evidence.seed_data_first_lba = first_lba;
            evidence.seed_data_lba_count = last_lba - first_lba + 1;
        }
        idx += 1;
    }

    if !evidence.seed_data_found {
        return GptLayoutEvidence {
            status: LayoutStatus::Absent,
            reason: "seed_data_partition_missing",
            ..evidence
        };
    }
    if !evidence.esp_a_found || !evidence.esp_b_found {
        return evidence.invalid_from("esp_slots_missing");
    }

    GptLayoutEvidence {
        status: LayoutStatus::Present,
        reason: "seed_data_partition_validated",
        ..evidence
    }
}

pub fn gpt_layout_record(evidence: &GptLayoutEvidence) -> Value<'static> {
    Value::Object(vec![
        Field::new("schema", Value::Str(GPT_LAYOUT_SCHEMA)),
        Field::new("id", Value::Str(GPT_LAYOUT_ID)),
        Field::new("scope", Value::Str("current_boot")),
        Field::new("classification", Value::Str("local_only")),
        Field::new("status", Value::Str(evidence.status.as_str())),
        Field::new("reason", Value::Str(evidence.reason)),
        Field::new(
            "protective_mbr_valid",
            Value::Bool(evidence.protective_mbr_valid),
        ),
        Field::new("gpt_header_valid", Value::Bool(evidence.gpt_header_valid)),
        Field::new("gpt_crc_checked", Value::Bool(gpt_crc_checked(evidence))),
        Field::new(
            "header_crc32_checked",
            Value::Bool(evidence.header_crc32_checked),
        ),
        Field::new(
            "entry_array_crc32_checked",
            Value::Bool(evidence.entry_array_crc32_checked),
        ),
        Field::new("esp_a_found", Value::Bool(evidence.esp_a_found)),
        Field::new("esp_b_found", Value::Bool(evidence.esp_b_found)),
        Field::new("seed_data_found", Value::Bool(evidence.seed_data_found)),
        Field::new(
            "seed_data_first_lba",
            Value::U64(evidence.seed_data_first_lba),
        ),
        Field::new(
            "seed_data_lba_count",
            Value::U64(evidence.seed_data_lba_count),
        ),
        Field::new("read_only", Value::Bool(true)),
        Field::new("write_attempted", Value::Bool(false)),
        Field::new("writes_enabled", Value::Bool(false)),
        Field::new("persistence_claimed", Value::Bool(false)),
    ])
}

pub const fn gpt_crc_checked(evidence: &GptLayoutEvidence) -> bool {
    evidence.header_crc32_checked && evidence.entry_array_crc32_checked
}

pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= *byte as u32;
        let mut bit = 0u8;
        while bit < 8 {
            let mask = if crc & 1 == 1 { 0xedb8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
            bit += 1;
        }
    }
    !crc
}

fn protective_mbr_is_valid(mbr: &[u8]) -> bool {
    mbr.len() >= SECTOR_SIZE
        && mbr[MBR_SIGNATURE_OFFSET] == 0x55
        && mbr[MBR_SIGNATURE_OFFSET + 1] == 0xaa
        && mbr[MBR_PARTITION_TABLE_OFFSET + 4] == PROTECTIVE_MBR_TYPE
        && read_u32(mbr, MBR_PARTITION_TABLE_OFFSET + 8) == 1
        && read_u32(mbr, MBR_PARTITION_TABLE_OFFSET + 12) != 0
}

fn parse_header(header: &[u8]) -> Result<Header, &'static str> {
    if header.len() < SECTOR_SIZE {
        return Err("gpt_header_truncated");
    }
    let header_size = read_u32(header, 12) as usize;
    if !(GPT_HEADER_MIN_SIZE..=SECTOR_SIZE).contains(&header_size) {
        return Err("gpt_header_size_invalid");
    }
    let stored_crc = read_u32(header, 16);
    if crc32_with_zeroed_range(&header[..header_size], 16, 4) != stored_crc {
        return Err("gpt_header_crc32_mismatch");
    }

    Ok(Header {
        first_usable_lba: read_u64(header, 40),
        last_usable_lba: read_u64(header, 48),
        entry_lba: read_u64(header, 72),
        entry_count: read_u32(header, 80),
        entry_size: read_u32(header, 84),
        entry_array_crc32: read_u32(header, 88),
    })
}

fn crc32_with_zeroed_range(bytes: &[u8], zero_offset: usize, zero_len: usize) -> u32 {
    let mut crc = 0xffff_ffffu32;
    let mut idx = 0usize;
    while idx < bytes.len() {
        let byte = if idx >= zero_offset && idx < zero_offset + zero_len {
            0
        } else {
            bytes[idx]
        };
        crc ^= byte as u32;
        let mut bit = 0u8;
        while bit < 8 {
            let mask = if crc & 1 == 1 { 0xedb8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
            bit += 1;
        }
        idx += 1;
    }
    !crc
}

fn gpt_name_eq(raw: &[u8], expected: &str) -> bool {
    let expected = expected.as_bytes();
    if raw.len() < expected.len() * 2 {
        return false;
    }
    let mut idx = 0usize;
    while idx < expected.len() {
        if raw[idx * 2] != expected[idx] || raw[idx * 2 + 1] != 0 {
            return false;
        }
        idx += 1;
    }
    let mut tail = expected.len() * 2;
    while tail < raw.len() {
        if raw[tail] != 0 {
            return false;
        }
        tail += 1;
    }
    true
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

    const FIRST_USABLE_LBA: u64 = 34;
    const LAST_USABLE_LBA: u64 = 1_050_623;
    const ESP_A_START_LBA: u64 = 2_048;
    const ESP_LBA_COUNT: u64 = 262_144;
    const ESP_B_START_LBA: u64 = ESP_A_START_LBA + ESP_LBA_COUNT;
    const SEED_DATA_START_LBA: u64 = ESP_B_START_LBA + ESP_LBA_COUNT;
    const SEED_DATA_LBA_COUNT: u64 = 524_288;

    fn render(value: &Value<'_>) -> Vec<u8> {
        let mut out = Vec::new();
        write_json(value, &mut out, 0);
        out
    }

    fn valid_mbr() -> [u8; SECTOR_SIZE] {
        let mut mbr = [0u8; SECTOR_SIZE];
        mbr[MBR_PARTITION_TABLE_OFFSET + 4] = PROTECTIVE_MBR_TYPE;
        write_u32(&mut mbr, MBR_PARTITION_TABLE_OFFSET + 8, 1);
        write_u32(&mut mbr, MBR_PARTITION_TABLE_OFFSET + 12, 0x000c_087d);
        mbr[510] = 0x55;
        mbr[511] = 0xaa;
        mbr
    }

    fn valid_header(entries: &[u8]) -> [u8; SECTOR_SIZE] {
        let mut header = [0u8; SECTOR_SIZE];
        header[..8].copy_from_slice(GPT_SIGNATURE);
        write_u32(&mut header, 8, 0x0001_0000);
        write_u32(&mut header, 12, GPT_HEADER_MIN_SIZE as u32);
        write_u64(&mut header, 24, PRIMARY_GPT_HEADER_LBA);
        write_u64(&mut header, 32, 788_543);
        write_u64(&mut header, 40, FIRST_USABLE_LBA);
        write_u64(&mut header, 48, LAST_USABLE_LBA);
        write_u64(&mut header, 72, PRIMARY_GPT_ENTRIES_LBA);
        write_u32(&mut header, 80, GPT_ENTRY_COUNT as u32);
        write_u32(&mut header, 84, GPT_ENTRY_SIZE as u32);
        write_u32(&mut header, 88, crc32(entries));
        let checksum = crc32_with_zeroed_range(&header[..GPT_HEADER_MIN_SIZE], 16, 4);
        write_u32(&mut header, 16, checksum);
        header
    }

    fn valid_entries() -> [u8; GPT_ENTRY_ARRAY_BYTES] {
        let mut entries = [0u8; GPT_ENTRY_ARRAY_BYTES];
        write_entry(
            &mut entries,
            0,
            ESP_TYPE_GUID_LE,
            ESP_A_START_LBA,
            ESP_LBA_COUNT,
            ESP_A_NAME,
        );
        write_entry(
            &mut entries,
            1,
            ESP_TYPE_GUID_LE,
            ESP_B_START_LBA,
            ESP_LBA_COUNT,
            ESP_B_NAME,
        );
        write_entry(
            &mut entries,
            2,
            SEED_DATA_TYPE_GUID_LE,
            SEED_DATA_START_LBA,
            SEED_DATA_LBA_COUNT,
            SEED_DATA_NAME,
        );
        entries
    }

    fn write_entry(
        entries: &mut [u8],
        index: usize,
        type_guid: [u8; 16],
        first_lba: u64,
        lba_count: u64,
        name: &str,
    ) {
        let start = index * GPT_ENTRY_SIZE;
        entries[start..start + 16].copy_from_slice(&type_guid);
        write_u64(entries, start + 32, first_lba);
        write_u64(entries, start + 40, first_lba + lba_count - 1);
        for (idx, byte) in name.bytes().enumerate() {
            entries[start + 56 + idx * 2] = byte;
        }
    }

    fn validate(entries: &[u8]) -> GptLayoutEvidence {
        validate_gpt_layout(&valid_mbr(), &valid_header(entries), entries)
    }

    #[test]
    fn valid_disk_finds_seed_data() {
        let entries = valid_entries();
        let evidence = validate(&entries);

        assert_eq!(evidence.status, LayoutStatus::Present);
        assert!(evidence.protective_mbr_valid);
        assert!(gpt_crc_checked(&evidence));
        assert!(evidence.esp_a_found);
        assert!(evidence.esp_b_found);
        assert_eq!(evidence.seed_data_first_lba, SEED_DATA_START_LBA);
        assert_eq!(evidence.seed_data_lba_count, SEED_DATA_LBA_COUNT);
    }

    #[test]
    fn bad_header_crc_is_invalid() {
        let entries = valid_entries();
        let mut header = valid_header(&entries);
        header[24] ^= 1;
        let evidence = validate_gpt_layout(&valid_mbr(), &header, &entries);

        assert_eq!(evidence.status, LayoutStatus::Invalid);
        assert_eq!(evidence.reason, "gpt_header_crc32_mismatch");
    }

    #[test]
    fn bad_entry_array_crc_is_invalid() {
        let mut entries = valid_entries();
        let header = valid_header(&entries);
        entries[0] ^= 1;
        let evidence = validate_gpt_layout(&valid_mbr(), &header, &entries);

        assert_eq!(evidence.status, LayoutStatus::Invalid);
        assert_eq!(evidence.reason, "partition_entry_array_crc32_mismatch");
    }

    #[test]
    fn truncated_table_is_invalid() {
        let entries = valid_entries();
        let header = valid_header(&entries);
        let evidence = validate_gpt_layout(&valid_mbr(), &header, &entries[..GPT_ENTRY_SIZE]);

        assert_eq!(evidence.status, LayoutStatus::Invalid);
        assert_eq!(evidence.reason, "partition_entry_array_truncated");
    }

    #[test]
    fn duplicate_seed_data_is_invalid() {
        let mut entries = valid_entries();
        write_entry(
            &mut entries,
            3,
            SEED_DATA_TYPE_GUID_LE,
            SEED_DATA_START_LBA + 128,
            128,
            SEED_DATA_NAME,
        );
        let evidence = validate(&entries);

        assert_eq!(evidence.status, LayoutStatus::Invalid);
        assert_eq!(evidence.reason, "duplicate_seed_data_partition");
    }

    #[test]
    fn missing_seed_data_is_absent() {
        let mut entries = valid_entries();
        entries[GPT_ENTRY_SIZE * 2..GPT_ENTRY_SIZE * 3].fill(0);
        let evidence = validate(&entries);

        assert_eq!(evidence.status, LayoutStatus::Absent);
        assert_eq!(evidence.reason, "seed_data_partition_missing");
    }

    #[test]
    fn gpt_layout_record_renders_expected_shape() {
        let entries = valid_entries();
        let evidence = validate(&entries);

        assert_eq!(
            render(&gpt_layout_record(&evidence)),
            b"{\r\n  \"schema\": \"raios.gpt_layout.v0\",\r\n  \"id\": \"gpt_layout.current_boot.v0\",\r\n  \"scope\": \"current_boot\",\r\n  \"classification\": \"local_only\",\r\n  \"status\": \"present\",\r\n  \"reason\": \"seed_data_partition_validated\",\r\n  \"protective_mbr_valid\": true,\r\n  \"gpt_header_valid\": true,\r\n  \"gpt_crc_checked\": true,\r\n  \"header_crc32_checked\": true,\r\n  \"entry_array_crc32_checked\": true,\r\n  \"esp_a_found\": true,\r\n  \"esp_b_found\": true,\r\n  \"seed_data_found\": true,\r\n  \"seed_data_first_lba\": 526336,\r\n  \"seed_data_lba_count\": 524288,\r\n  \"read_only\": true,\r\n  \"write_attempted\": false,\r\n  \"writes_enabled\": false,\r\n  \"persistence_claimed\": false\r\n}"
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
