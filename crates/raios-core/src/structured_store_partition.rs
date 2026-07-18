//! Identity validation for the dedicated M13 structured-store GPT partition.
//!
//! This is deliberately a parser, not a disk selector. Callers supply the
//! already approved disk and partition GUIDs; a positive result only describes
//! the one bounded region that a controller-specific port may later revalidate.

use crate::gpt_layout::{
    crc32, GPT_ENTRY_ARRAY_BYTES, GPT_ENTRY_COUNT, GPT_ENTRY_SIZE, SECTOR_SIZE,
};

pub const STRUCTURED_STORE_TYPE_GUID_LE: [u8; 16] = [
    0x7a, 0xda, 0xed, 0x5e, 0xde, 0xc0, 0x55, 0x4a, 0x9a, 0x15, 0x00, 0x00, 0x00, 0x00, 0x00, 0x13,
];
pub const STRUCTURED_STORE_LABEL: &str = "RAIOS_STRUCTURED_STORE";

const GPT_SIGNATURE: &[u8; 8] = b"EFI PART";
const GPT_REVISION_1_0: u32 = 0x0001_0000;
const GPT_HEADER_MIN_LEN: usize = 92;
const GPT_HEADER_CRC_OFFSET: usize = 16;
const GPT_HEADER_CRC_LEN: usize = 4;
const GPT_PRIMARY_HEADER_LBA: u64 = 1;
const GPT_PRIMARY_ENTRIES_LBA: u64 = 2;
const GPT_BACKUP_ENTRY_LBAS: u64 = (GPT_ENTRY_ARRAY_BYTES / SECTOR_SIZE) as u64;
const MBR_PARTITION_OFFSET: usize = 446;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApprovedStructuredStorePartition {
    pub gpt_disk_guid: [u8; 16],
    pub partition_guid: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StructuredStorePartition {
    pub gpt_disk_guid: [u8; 16],
    pub partition_guid: [u8; 16],
    pub first_lba: u64,
    pub lba_count: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructuredStorePartitionDenied {
    InvalidApproval,
    InvalidTotalLbaCount,
    ProtectiveMbrInvalid,
    PrimaryHeaderInvalid,
    BackupHeaderInvalid,
    HeaderPairMismatch,
    EntryArrayCrcMismatch,
    EntryArrayPairMismatch,
    PartitionRangeInvalid,
    TargetTypeLabelMismatch,
    TargetPartitionMissing,
    DuplicateTargetPartition,
    DiskIdentityMismatch,
    PartitionIdentityMismatch,
    TargetOverlap,
}

impl StructuredStorePartitionDenied {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::InvalidApproval => "structured_store_partition_approval_invalid",
            Self::InvalidTotalLbaCount => "structured_store_partition_total_lba_invalid",
            Self::ProtectiveMbrInvalid => "structured_store_partition_protective_mbr_invalid",
            Self::PrimaryHeaderInvalid => "structured_store_partition_primary_gpt_invalid",
            Self::BackupHeaderInvalid => "structured_store_partition_backup_gpt_invalid",
            Self::HeaderPairMismatch => "structured_store_partition_gpt_pair_mismatch",
            Self::EntryArrayCrcMismatch => "structured_store_partition_entry_crc_mismatch",
            Self::EntryArrayPairMismatch => "structured_store_partition_entry_pair_mismatch",
            Self::PartitionRangeInvalid => "structured_store_partition_range_invalid",
            Self::TargetTypeLabelMismatch => "structured_store_partition_type_label_mismatch",
            Self::TargetPartitionMissing => "structured_store_partition_missing",
            Self::DuplicateTargetPartition => "structured_store_partition_duplicate",
            Self::DiskIdentityMismatch => "structured_store_partition_disk_identity_mismatch",
            Self::PartitionIdentityMismatch => "structured_store_partition_identity_mismatch",
            Self::TargetOverlap => "structured_store_partition_target_overlap",
        }
    }
}

#[derive(Clone, Copy)]
struct GptHeader {
    current_lba: u64,
    backup_lba: u64,
    first_usable_lba: u64,
    last_usable_lba: u64,
    disk_guid: [u8; 16],
    entries_lba: u64,
    entry_count: u32,
    entry_size: u32,
    entries_crc32: u32,
}

/// Validates both GPT copies and locates exactly one approved M13 partition.
///
/// `total_lba_count` comes from a controller identity/identify result. The
/// function never derives it from a host path and cannot choose a device.
pub fn validate_approved_structured_store_partition(
    protective_mbr: &[u8],
    primary_header: &[u8],
    primary_entries: &[u8],
    backup_header: &[u8],
    backup_entries: &[u8],
    total_lba_count: u64,
    approval: ApprovedStructuredStorePartition,
) -> Result<StructuredStorePartition, StructuredStorePartitionDenied> {
    if approval.gpt_disk_guid == [0; 16] || approval.partition_guid == [0; 16] {
        return Err(StructuredStorePartitionDenied::InvalidApproval);
    }
    if total_lba_count <= GPT_PRIMARY_ENTRIES_LBA + GPT_BACKUP_ENTRY_LBAS + 2 {
        return Err(StructuredStorePartitionDenied::InvalidTotalLbaCount);
    }
    validate_protective_mbr(protective_mbr)?;

    let primary = parse_header(primary_header)
        .map_err(|_| StructuredStorePartitionDenied::PrimaryHeaderInvalid)?;
    let backup = parse_header(backup_header)
        .map_err(|_| StructuredStorePartitionDenied::BackupHeaderInvalid)?;
    validate_header_pair(primary, backup, total_lba_count)?;
    if primary.disk_guid != approval.gpt_disk_guid {
        return Err(StructuredStorePartitionDenied::DiskIdentityMismatch);
    }
    validate_entries(primary_entries, primary)?;
    validate_entries(backup_entries, backup)?;
    if primary_entries != backup_entries {
        return Err(StructuredStorePartitionDenied::EntryArrayPairMismatch);
    }

    let mut target = None;
    let mut index = 0usize;
    while index < GPT_ENTRY_COUNT {
        let start = index * GPT_ENTRY_SIZE;
        let entry = &primary_entries[start..start + GPT_ENTRY_SIZE];
        if entry[..16] == [0; 16] {
            index += 1;
            continue;
        }
        let first_lba = read_u64(entry, 32);
        let last_lba = read_u64(entry, 40);
        if first_lba == 0
            || last_lba < first_lba
            || first_lba < primary.first_usable_lba
            || last_lba > primary.last_usable_lba
        {
            return Err(StructuredStorePartitionDenied::PartitionRangeInvalid);
        }

        let is_target_type = entry[..16] == STRUCTURED_STORE_TYPE_GUID_LE;
        let is_target_label = gpt_name_eq(&entry[56..128], STRUCTURED_STORE_LABEL);
        if is_target_type || is_target_label {
            if !is_target_type || !is_target_label {
                return Err(StructuredStorePartitionDenied::TargetTypeLabelMismatch);
            }
            let mut partition_guid = [0u8; 16];
            partition_guid.copy_from_slice(&entry[16..32]);
            if partition_guid != approval.partition_guid {
                return Err(StructuredStorePartitionDenied::PartitionIdentityMismatch);
            }
            if target.is_some() {
                return Err(StructuredStorePartitionDenied::DuplicateTargetPartition);
            }
            target = Some(StructuredStorePartition {
                gpt_disk_guid: primary.disk_guid,
                partition_guid,
                first_lba,
                lba_count: last_lba
                    .checked_sub(first_lba)
                    .and_then(|span| span.checked_add(1))
                    .ok_or(StructuredStorePartitionDenied::PartitionRangeInvalid)?,
            });
        }
        index += 1;
    }

    let target = target.ok_or(StructuredStorePartitionDenied::TargetPartitionMissing)?;
    validate_no_target_overlap(primary_entries, target)?;
    Ok(target)
}

fn validate_protective_mbr(mbr: &[u8]) -> Result<(), StructuredStorePartitionDenied> {
    if mbr.len() != SECTOR_SIZE
        || mbr[510] != 0x55
        || mbr[511] != 0xaa
        || mbr[MBR_PARTITION_OFFSET + 4] != 0xee
        || read_u32(mbr, MBR_PARTITION_OFFSET + 8) != 1
        || read_u32(mbr, MBR_PARTITION_OFFSET + 12) == 0
    {
        return Err(StructuredStorePartitionDenied::ProtectiveMbrInvalid);
    }
    Ok(())
}

fn parse_header(bytes: &[u8]) -> Result<GptHeader, ()> {
    if bytes.len() != SECTOR_SIZE || &bytes[..8] != GPT_SIGNATURE {
        return Err(());
    }
    if read_u32(bytes, 8) != GPT_REVISION_1_0 || read_u32(bytes, 20) != 0 {
        return Err(());
    }
    let header_len = read_u32(bytes, 12) as usize;
    if !(GPT_HEADER_MIN_LEN..=SECTOR_SIZE).contains(&header_len)
        || bytes[header_len..].iter().any(|byte| *byte != 0)
    {
        return Err(());
    }
    let stored_crc = read_u32(bytes, GPT_HEADER_CRC_OFFSET);
    let mut checked = [0u8; SECTOR_SIZE];
    checked.copy_from_slice(bytes);
    checked[GPT_HEADER_CRC_OFFSET..GPT_HEADER_CRC_OFFSET + GPT_HEADER_CRC_LEN].fill(0);
    if crc32(&checked[..header_len]) != stored_crc {
        return Err(());
    }

    let mut disk_guid = [0u8; 16];
    disk_guid.copy_from_slice(&bytes[56..72]);
    let header = GptHeader {
        current_lba: read_u64(bytes, 24),
        backup_lba: read_u64(bytes, 32),
        first_usable_lba: read_u64(bytes, 40),
        last_usable_lba: read_u64(bytes, 48),
        disk_guid,
        entries_lba: read_u64(bytes, 72),
        entry_count: read_u32(bytes, 80),
        entry_size: read_u32(bytes, 84),
        entries_crc32: read_u32(bytes, 88),
    };
    if header.disk_guid == [0; 16]
        || header.first_usable_lba == 0
        || header.last_usable_lba < header.first_usable_lba
        || header.entry_count != GPT_ENTRY_COUNT as u32
        || header.entry_size != GPT_ENTRY_SIZE as u32
    {
        return Err(());
    }
    Ok(header)
}

fn validate_header_pair(
    primary: GptHeader,
    backup: GptHeader,
    total_lba_count: u64,
) -> Result<(), StructuredStorePartitionDenied> {
    let backup_lba = total_lba_count - 1;
    let backup_entries_lba = backup_lba
        .checked_sub(GPT_BACKUP_ENTRY_LBAS)
        .ok_or(StructuredStorePartitionDenied::InvalidTotalLbaCount)?;
    if primary.current_lba != GPT_PRIMARY_HEADER_LBA
        || primary.backup_lba != backup_lba
        || primary.entries_lba != GPT_PRIMARY_ENTRIES_LBA
        || backup.current_lba != backup_lba
        || backup.backup_lba != GPT_PRIMARY_HEADER_LBA
        || backup.entries_lba != backup_entries_lba
        || primary.first_usable_lba != backup.first_usable_lba
        || primary.last_usable_lba != backup.last_usable_lba
        || primary.disk_guid != backup.disk_guid
        || primary.entry_count != backup.entry_count
        || primary.entry_size != backup.entry_size
        || primary.entries_crc32 != backup.entries_crc32
        || primary.last_usable_lba >= backup_entries_lba
    {
        return Err(StructuredStorePartitionDenied::HeaderPairMismatch);
    }
    Ok(())
}

fn validate_entries(
    entries: &[u8],
    header: GptHeader,
) -> Result<(), StructuredStorePartitionDenied> {
    if entries.len() != GPT_ENTRY_ARRAY_BYTES || crc32(entries) != header.entries_crc32 {
        return Err(StructuredStorePartitionDenied::EntryArrayCrcMismatch);
    }
    Ok(())
}

fn validate_no_target_overlap(
    entries: &[u8],
    target: StructuredStorePartition,
) -> Result<(), StructuredStorePartitionDenied> {
    let target_last = target
        .first_lba
        .checked_add(target.lba_count - 1)
        .ok_or(StructuredStorePartitionDenied::PartitionRangeInvalid)?;
    let mut index = 0usize;
    while index < GPT_ENTRY_COUNT {
        let start = index * GPT_ENTRY_SIZE;
        let entry = &entries[start..start + GPT_ENTRY_SIZE];
        if entry[..16] != [0; 16] {
            let mut guid = [0u8; 16];
            guid.copy_from_slice(&entry[16..32]);
            if guid != target.partition_guid {
                let first = read_u64(entry, 32);
                let last = read_u64(entry, 40);
                if first <= target_last && target.first_lba <= last {
                    return Err(StructuredStorePartitionDenied::TargetOverlap);
                }
            }
        }
        index += 1;
    }
    Ok(())
}

fn gpt_name_eq(raw: &[u8], expected: &str) -> bool {
    if raw.len() != 72 || expected.len() * 2 > raw.len() {
        return false;
    }
    for (index, byte) in expected.bytes().enumerate() {
        if raw[index * 2] != byte || raw[index * 2 + 1] != 0 {
            return false;
        }
    }
    raw[expected.len() * 2..].iter().all(|byte| *byte == 0)
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]))
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOTAL_LBAS: u64 = 65_536;
    const DISK_GUID: [u8; 16] = [0x11; 16];
    const PARTITION_GUID: [u8; 16] = [0x22; 16];
    const FIRST_LBA: u64 = 2_048;

    fn approval() -> ApprovedStructuredStorePartition {
        ApprovedStructuredStorePartition {
            gpt_disk_guid: DISK_GUID,
            partition_guid: PARTITION_GUID,
        }
    }

    fn mbr() -> [u8; SECTOR_SIZE] {
        let mut bytes = [0u8; SECTOR_SIZE];
        bytes[MBR_PARTITION_OFFSET + 4] = 0xee;
        write_u32(&mut bytes, MBR_PARTITION_OFFSET + 8, 1);
        write_u32(&mut bytes, MBR_PARTITION_OFFSET + 12, u32::MAX);
        bytes[510] = 0x55;
        bytes[511] = 0xaa;
        bytes
    }

    fn entries() -> [u8; GPT_ENTRY_ARRAY_BYTES] {
        let mut bytes = [0u8; GPT_ENTRY_ARRAY_BYTES];
        bytes[..16].copy_from_slice(&STRUCTURED_STORE_TYPE_GUID_LE);
        bytes[16..32].copy_from_slice(&PARTITION_GUID);
        write_u64(&mut bytes, 32, FIRST_LBA);
        write_u64(&mut bytes, 40, TOTAL_LBAS - 34);
        for (index, byte) in STRUCTURED_STORE_LABEL.bytes().enumerate() {
            bytes[56 + index * 2] = byte;
        }
        bytes
    }

    fn header(
        current_lba: u64,
        backup_lba: u64,
        entries_lba: u64,
        entries: &[u8],
    ) -> [u8; SECTOR_SIZE] {
        let mut bytes = [0u8; SECTOR_SIZE];
        bytes[..8].copy_from_slice(GPT_SIGNATURE);
        write_u32(&mut bytes, 8, GPT_REVISION_1_0);
        write_u32(&mut bytes, 12, GPT_HEADER_MIN_LEN as u32);
        write_u64(&mut bytes, 24, current_lba);
        write_u64(&mut bytes, 32, backup_lba);
        write_u64(&mut bytes, 40, 34);
        write_u64(&mut bytes, 48, TOTAL_LBAS - GPT_BACKUP_ENTRY_LBAS - 2);
        bytes[56..72].copy_from_slice(&DISK_GUID);
        write_u64(&mut bytes, 72, entries_lba);
        write_u32(&mut bytes, 80, GPT_ENTRY_COUNT as u32);
        write_u32(&mut bytes, 84, GPT_ENTRY_SIZE as u32);
        write_u32(&mut bytes, 88, crc32(entries));
        let header_crc = crc32(&bytes[..GPT_HEADER_MIN_LEN]);
        write_u32(&mut bytes, 16, header_crc);
        bytes
    }

    fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    fn valid_inputs() -> (
        [u8; SECTOR_SIZE],
        [u8; SECTOR_SIZE],
        [u8; GPT_ENTRY_ARRAY_BYTES],
        [u8; SECTOR_SIZE],
    ) {
        let entries = entries();
        let primary = header(1, TOTAL_LBAS - 1, 2, &entries);
        let backup = header(
            TOTAL_LBAS - 1,
            1,
            TOTAL_LBAS - 1 - GPT_BACKUP_ENTRY_LBAS,
            &entries,
        );
        (mbr(), primary, entries, backup)
    }

    fn validate() -> Result<StructuredStorePartition, StructuredStorePartitionDenied> {
        let (mbr, primary, entries, backup) = valid_inputs();
        validate_approved_structured_store_partition(
            &mbr,
            &primary,
            &entries,
            &backup,
            &entries,
            TOTAL_LBAS,
            approval(),
        )
    }

    #[test]
    fn validates_the_exact_approved_redundant_partition() {
        assert_eq!(
            validate().unwrap(),
            StructuredStorePartition {
                gpt_disk_guid: DISK_GUID,
                partition_guid: PARTITION_GUID,
                first_lba: FIRST_LBA,
                lba_count: TOTAL_LBAS - 34 - FIRST_LBA + 1,
            }
        );
    }

    #[test]
    fn rejects_identity_type_label_and_redundancy_mismatches() {
        let (mbr, primary, mut entries, backup) = valid_inputs();
        entries[16] ^= 1;
        assert_eq!(
            validate_approved_structured_store_partition(
                &mbr,
                &primary,
                &entries,
                &backup,
                &entries,
                TOTAL_LBAS,
                approval(),
            ),
            Err(StructuredStorePartitionDenied::EntryArrayCrcMismatch)
        );

        let (mbr, primary, entries, mut backup) = valid_inputs();
        backup[56] ^= 1;
        assert_eq!(
            validate_approved_structured_store_partition(
                &mbr,
                &primary,
                &entries,
                &backup,
                &entries,
                TOTAL_LBAS,
                approval(),
            ),
            Err(StructuredStorePartitionDenied::BackupHeaderInvalid)
        );

        let mut wrong = approval();
        wrong.partition_guid[0] ^= 1;
        let (mbr, primary, entries, backup) = valid_inputs();
        assert_eq!(
            validate_approved_structured_store_partition(
                &mbr, &primary, &entries, &backup, &entries, TOTAL_LBAS, wrong,
            ),
            Err(StructuredStorePartitionDenied::PartitionIdentityMismatch)
        );
    }

    #[test]
    fn rejects_target_overlap_and_missing_target() {
        let (mbr, _primary, mut entries, _backup) = valid_inputs();
        let second = GPT_ENTRY_SIZE;
        entries[second..second + 16].copy_from_slice(&[0x99; 16]);
        entries[second + 16..second + 32].copy_from_slice(&[0x88; 16]);
        write_u64(&mut entries, second + 32, FIRST_LBA + 1);
        write_u64(&mut entries, second + 40, FIRST_LBA + 2);
        let primary = header(1, TOTAL_LBAS - 1, 2, &entries);
        let backup = header(
            TOTAL_LBAS - 1,
            1,
            TOTAL_LBAS - 1 - GPT_BACKUP_ENTRY_LBAS,
            &entries,
        );
        assert_eq!(
            validate_approved_structured_store_partition(
                &mbr,
                &primary,
                &entries,
                &backup,
                &entries,
                TOTAL_LBAS,
                approval(),
            ),
            Err(StructuredStorePartitionDenied::TargetOverlap)
        );

        let (mbr, _primary, mut entries, _backup) = valid_inputs();
        entries[..16].fill(0);
        entries[56..128].fill(0);
        let primary = header(1, TOTAL_LBAS - 1, 2, &entries);
        let backup = header(
            TOTAL_LBAS - 1,
            1,
            TOTAL_LBAS - 1 - GPT_BACKUP_ENTRY_LBAS,
            &entries,
        );
        assert_eq!(
            validate_approved_structured_store_partition(
                &mbr,
                &primary,
                &entries,
                &backup,
                &entries,
                TOTAL_LBAS,
                approval(),
            ),
            Err(StructuredStorePartitionDenied::TargetPartitionMissing)
        );
    }
}
