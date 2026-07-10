use alloc::vec::Vec;

#[cfg(test)]
use alloc::vec;

use crate::sha256_bytes;

pub const STORE_BLOCK_LEN: usize = 512;
pub const SUPERBLOCK_LEN: usize = STORE_BLOCK_LEN;
pub const FRAME_HEADER_LEN: usize = 208;
pub const MAX_FRAME_PAYLOAD_LEN: usize = STORE_BLOCK_LEN - FRAME_HEADER_LEN;
pub const MAX_RECORD_PAYLOAD_LEN: usize = 64 * 1024;
pub const MAX_DATA_FRAME_COUNT: u32 = 256;

const SUPERBLOCK_MAGIC: &[u8; 8] = b"RAIOSSB1";
const FRAME_MAGIC: &[u8; 8] = b"RAIOSFR1";
const FORMAT_VERSION: u16 = 1;
const ZERO_HASH: [u8; 32] = [0; 32];

const SB_VERSION: usize = 8;
const SB_HEADER_LEN: usize = 10;
const SB_COPY_INDEX: usize = 12;
const SB_ACTIVE_COPY: usize = 13;
const SB_SECTOR_SIZE: usize = 16;
const SB_STORE_UUID: usize = 24;
const SB_GENERATION: usize = 40;
const SB_SELECTION_EPOCH: usize = 48;
const SB_PARTITION_START_LBA: usize = 56;
const SB_PARTITION_LBA_COUNT: usize = 64;
const SB_LOG_START_LBA: usize = 72;
const SB_LOG_LBA_COUNT: usize = 80;
const SB_HASH: usize = 88;
const SB_HASH_END: usize = SB_HASH + 32;

const FRAME_VERSION: usize = 8;
const FRAME_KIND: usize = 10;
const FRAME_FLAGS: usize = 11;
const FRAME_HEADER_SIZE: usize = 12;
const FRAME_RESERVED: usize = 14;
const FRAME_LEN: usize = 16;
const FRAME_PAYLOAD_LEN: usize = 20;
const FRAME_SEQUENCE: usize = 24;
const FRAME_TRANSACTION_ID: usize = 32;
const FRAME_RECORD_VERSION: usize = 40;
const FRAME_STORE_GENERATION: usize = 48;
const FRAME_STORE_UUID: usize = 56;
const FRAME_NAMESPACE: usize = 72;
const FRAME_RECORD_ID: usize = 88;
const FRAME_PREVIOUS_HASH: usize = 104;
const FRAME_PAYLOAD_HASH: usize = 136;
const FRAME_HEADER_HASH: usize = 168;
const FRAME_HEADER_HASH_END: usize = FRAME_HEADER_HASH + 32;
const FRAME_CRC32: usize = 200;
const FRAME_RESERVED_TAIL: usize = 204;

const PREPARE_DESCRIPTOR_LEN: usize = 48;
const COMMIT_DESCRIPTOR_LEN: usize = 72;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoreIdentity {
    pub store_uuid: [u8; 16],
    pub generation: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordKey {
    pub namespace: [u8; 16],
    pub record_id: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StoreGeometry {
    pub partition_start_lba: u64,
    pub partition_lba_count: u64,
    pub log_start_lba: u64,
    pub log_lba_count: u64,
    pub logical_sector_size: u32,
}

impl StoreGeometry {
    pub fn log_byte_len(self) -> Result<u64, StoreDenied> {
        self.log_lba_count
            .checked_mul(self.logical_sector_size as u64)
            .ok_or(StoreDenied::GeometryOverflow)
    }

    pub fn validate(self) -> Result<(), StoreDenied> {
        if self.logical_sector_size as usize != STORE_BLOCK_LEN {
            return Err(StoreDenied::UnsupportedSectorSize);
        }
        if self.partition_start_lba == 0
            || self.partition_lba_count < 3
            || self.log_start_lba < 2
            || self.log_lba_count == 0
        {
            return Err(StoreDenied::InvalidGeometry);
        }
        let log_end = self
            .log_start_lba
            .checked_add(self.log_lba_count)
            .ok_or(StoreDenied::GeometryOverflow)?;
        if log_end > self.partition_lba_count {
            return Err(StoreDenied::BoundsDenied);
        }
        self.partition_start_lba
            .checked_add(self.partition_lba_count)
            .ok_or(StoreDenied::GeometryOverflow)?;
        self.log_byte_len()?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Superblock {
    pub copy_index: u8,
    pub active_copy: u8,
    pub identity: StoreIdentity,
    pub selection_epoch: u64,
    pub geometry: StoreGeometry,
    pub block_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedSuperblock {
    pub superblock: Superblock,
    pub bytes: [u8; SUPERBLOCK_LEN],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreDenied {
    InvalidSuperblockLength,
    InvalidSuperblockMagic,
    UnsupportedFormatVersion,
    InvalidSuperblockHeaderLength,
    InvalidSuperblockCopy,
    InvalidSuperblockActiveSelection,
    InvalidSuperblockReservedBytes,
    SuperblockHashMismatch,
    SuperblockIdentityMismatch,
    SuperblockGeometryMismatch,
    AmbiguousSuperblockSelection,
    NoValidSuperblock,
    StoreUuidMissing,
    StaleStoreGeneration,
    UnsupportedSectorSize,
    InvalidGeometry,
    GeometryOverflow,
    BoundsDenied,
    RegionNotBlockAligned,
    RegionLengthMismatch,
    InvalidFrameMagic,
    InvalidFrameHeaderLength,
    InvalidFrameLength,
    InvalidFrameKind,
    InvalidFrameFlags,
    InvalidFrameReservedBytes,
    InvalidFramePayloadLength,
    FrameSequenceMismatch,
    TransactionIdMissing,
    RecordVersionMissing,
    RecordKeyMissing,
    PreviousFrameHashMismatch,
    PayloadHashMismatch,
    HeaderHashMismatch,
    FrameCrcMismatch,
    FramePaddingNonzero,
    FrameStoreIdentityMismatch,
    PrepareDescriptorInvalid,
    CommitDescriptorInvalid,
    TransactionProtocolInvalid,
    TransactionBindingMismatch,
    TransactionPayloadMismatch,
    StaleTransactionId,
    StaleRecordVersion,
    TransactionIdOverflow,
    RecordVersionOverflow,
    FrameSequenceOverflow,
    RecordPayloadTooLarge,
    TooManyDataFrames,
    NamespaceLocked,
    StoreChainLocked,
    TailNotAppendable,
    ExpectedVersionMismatch,
    CapacityExhausted,
    ReadbackMismatch,
    CommitReplayMismatch,
}

impl StoreDenied {
    pub const fn reason(self) -> &'static str {
        match self {
            Self::InvalidSuperblockLength => "structured_store_superblock_length_invalid",
            Self::InvalidSuperblockMagic => "structured_store_superblock_magic_invalid",
            Self::UnsupportedFormatVersion => "structured_store_format_version_unsupported",
            Self::InvalidSuperblockHeaderLength => {
                "structured_store_superblock_header_length_invalid"
            }
            Self::InvalidSuperblockCopy => "structured_store_superblock_copy_invalid",
            Self::InvalidSuperblockActiveSelection => {
                "structured_store_superblock_active_selection_invalid"
            }
            Self::InvalidSuperblockReservedBytes => {
                "structured_store_superblock_reserved_bytes_nonzero"
            }
            Self::SuperblockHashMismatch => "structured_store_superblock_hash_mismatch",
            Self::SuperblockIdentityMismatch => "structured_store_superblock_identity_mismatch",
            Self::SuperblockGeometryMismatch => "structured_store_superblock_geometry_mismatch",
            Self::AmbiguousSuperblockSelection => "structured_store_superblock_selection_ambiguous",
            Self::NoValidSuperblock => "structured_store_superblock_missing_or_invalid",
            Self::StoreUuidMissing => "structured_store_uuid_missing",
            Self::StaleStoreGeneration => "structured_store_generation_stale",
            Self::UnsupportedSectorSize => "structured_store_sector_size_unsupported",
            Self::InvalidGeometry => "structured_store_geometry_invalid",
            Self::GeometryOverflow => "structured_store_geometry_overflow",
            Self::BoundsDenied => "structured_store_bounds_denied",
            Self::RegionNotBlockAligned => "structured_store_region_not_block_aligned",
            Self::RegionLengthMismatch => "structured_store_region_length_mismatch",
            Self::InvalidFrameMagic => "structured_store_frame_magic_invalid",
            Self::InvalidFrameHeaderLength => "structured_store_frame_header_length_invalid",
            Self::InvalidFrameLength => "structured_store_frame_length_invalid",
            Self::InvalidFrameKind => "structured_store_frame_kind_invalid",
            Self::InvalidFrameFlags => "structured_store_frame_flags_invalid",
            Self::InvalidFrameReservedBytes => "structured_store_frame_reserved_bytes_nonzero",
            Self::InvalidFramePayloadLength => "structured_store_frame_payload_length_invalid",
            Self::FrameSequenceMismatch => "structured_store_frame_sequence_mismatch",
            Self::TransactionIdMissing => "structured_store_transaction_id_missing",
            Self::RecordVersionMissing => "structured_store_record_version_missing",
            Self::RecordKeyMissing => "structured_store_record_key_missing",
            Self::PreviousFrameHashMismatch => "structured_store_previous_frame_hash_mismatch",
            Self::PayloadHashMismatch => "structured_store_payload_hash_mismatch",
            Self::HeaderHashMismatch => "structured_store_frame_header_hash_mismatch",
            Self::FrameCrcMismatch => "structured_store_frame_crc_mismatch",
            Self::FramePaddingNonzero => "structured_store_frame_padding_nonzero",
            Self::FrameStoreIdentityMismatch => "structured_store_frame_identity_mismatch",
            Self::PrepareDescriptorInvalid => "structured_store_prepare_descriptor_invalid",
            Self::CommitDescriptorInvalid => "structured_store_commit_descriptor_invalid",
            Self::TransactionProtocolInvalid => "structured_store_transaction_protocol_invalid",
            Self::TransactionBindingMismatch => "structured_store_transaction_binding_mismatch",
            Self::TransactionPayloadMismatch => "structured_store_transaction_payload_mismatch",
            Self::StaleTransactionId => "structured_store_transaction_id_stale",
            Self::StaleRecordVersion => "structured_store_record_version_stale",
            Self::TransactionIdOverflow => "structured_store_transaction_id_overflow",
            Self::RecordVersionOverflow => "structured_store_record_version_overflow",
            Self::FrameSequenceOverflow => "structured_store_frame_sequence_overflow",
            Self::RecordPayloadTooLarge => "structured_store_record_payload_too_large",
            Self::TooManyDataFrames => "structured_store_data_frame_count_too_large",
            Self::NamespaceLocked => "structured_store_namespace_locked",
            Self::StoreChainLocked => "structured_store_chain_locked",
            Self::TailNotAppendable => "structured_store_tail_not_appendable",
            Self::ExpectedVersionMismatch => "structured_store_expected_version_mismatch",
            Self::CapacityExhausted => "structured_store_capacity_exhausted",
            Self::ReadbackMismatch => "structured_store_readback_mismatch",
            Self::CommitReplayMismatch => "structured_store_commit_replay_mismatch",
        }
    }
}

pub fn encode_superblock(
    copy_index: u8,
    active_copy: u8,
    identity: StoreIdentity,
    selection_epoch: u64,
    geometry: StoreGeometry,
) -> Result<[u8; SUPERBLOCK_LEN], StoreDenied> {
    if identity.store_uuid == [0; 16] {
        return Err(StoreDenied::StoreUuidMissing);
    }
    if identity.generation == 0 {
        return Err(StoreDenied::StaleStoreGeneration);
    }
    if copy_index > 1 {
        return Err(StoreDenied::InvalidSuperblockCopy);
    }
    if active_copy > 1 || active_copy != copy_index {
        return Err(StoreDenied::InvalidSuperblockActiveSelection);
    }
    geometry.validate()?;

    let mut out = [0u8; SUPERBLOCK_LEN];
    out[..8].copy_from_slice(SUPERBLOCK_MAGIC);
    write_u16(&mut out, SB_VERSION, FORMAT_VERSION);
    write_u16(&mut out, SB_HEADER_LEN, SB_HASH_END as u16);
    out[SB_COPY_INDEX] = copy_index;
    out[SB_ACTIVE_COPY] = active_copy;
    write_u32(&mut out, SB_SECTOR_SIZE, geometry.logical_sector_size);
    out[SB_STORE_UUID..SB_STORE_UUID + 16].copy_from_slice(&identity.store_uuid);
    write_u64(&mut out, SB_GENERATION, identity.generation);
    write_u64(&mut out, SB_SELECTION_EPOCH, selection_epoch);
    write_u64(
        &mut out,
        SB_PARTITION_START_LBA,
        geometry.partition_start_lba,
    );
    write_u64(
        &mut out,
        SB_PARTITION_LBA_COUNT,
        geometry.partition_lba_count,
    );
    write_u64(&mut out, SB_LOG_START_LBA, geometry.log_start_lba);
    write_u64(&mut out, SB_LOG_LBA_COUNT, geometry.log_lba_count);
    let digest = superblock_hash(&out);
    out[SB_HASH..SB_HASH_END].copy_from_slice(&digest);
    Ok(out)
}

pub fn parse_superblock(bytes: &[u8]) -> Result<Superblock, StoreDenied> {
    if bytes.len() != SUPERBLOCK_LEN {
        return Err(StoreDenied::InvalidSuperblockLength);
    }
    if &bytes[..8] != SUPERBLOCK_MAGIC {
        return Err(StoreDenied::InvalidSuperblockMagic);
    }
    if read_u16(bytes, SB_VERSION) != FORMAT_VERSION {
        return Err(StoreDenied::UnsupportedFormatVersion);
    }
    if read_u16(bytes, SB_HEADER_LEN) as usize != SB_HASH_END {
        return Err(StoreDenied::InvalidSuperblockHeaderLength);
    }
    let copy_index = bytes[SB_COPY_INDEX];
    let active_copy = bytes[SB_ACTIVE_COPY];
    if copy_index > 1 {
        return Err(StoreDenied::InvalidSuperblockCopy);
    }
    if active_copy > 1 || active_copy != copy_index {
        return Err(StoreDenied::InvalidSuperblockActiveSelection);
    }
    if !all_zero(&bytes[14..16]) || !all_zero(&bytes[20..24]) || !all_zero(&bytes[SB_HASH_END..]) {
        return Err(StoreDenied::InvalidSuperblockReservedBytes);
    }

    let mut stored_hash = [0u8; 32];
    stored_hash.copy_from_slice(&bytes[SB_HASH..SB_HASH_END]);
    if stored_hash != superblock_hash(bytes) {
        return Err(StoreDenied::SuperblockHashMismatch);
    }

    let mut store_uuid = [0u8; 16];
    store_uuid.copy_from_slice(&bytes[SB_STORE_UUID..SB_STORE_UUID + 16]);
    let identity = StoreIdentity {
        store_uuid,
        generation: read_u64(bytes, SB_GENERATION),
    };
    if identity.store_uuid == [0; 16] {
        return Err(StoreDenied::StoreUuidMissing);
    }
    if identity.generation == 0 {
        return Err(StoreDenied::StaleStoreGeneration);
    }
    let geometry = StoreGeometry {
        partition_start_lba: read_u64(bytes, SB_PARTITION_START_LBA),
        partition_lba_count: read_u64(bytes, SB_PARTITION_LBA_COUNT),
        log_start_lba: read_u64(bytes, SB_LOG_START_LBA),
        log_lba_count: read_u64(bytes, SB_LOG_LBA_COUNT),
        logical_sector_size: read_u32(bytes, SB_SECTOR_SIZE),
    };
    geometry.validate()?;

    Ok(Superblock {
        copy_index,
        active_copy,
        identity,
        selection_epoch: read_u64(bytes, SB_SELECTION_EPOCH),
        geometry,
        block_sha256: sha256_bytes(bytes),
    })
}

pub fn select_superblock(
    copy0: &[u8],
    copy1: &[u8],
    expected_identity: StoreIdentity,
    expected_geometry: StoreGeometry,
) -> Result<SelectedSuperblock, StoreDenied> {
    expected_geometry.validate()?;
    let parsed_first = parse_superblock(copy0).ok();
    let parsed_second = parse_superblock(copy1).ok();
    if parsed_first.iter().any(|value| value.copy_index != 0)
        || parsed_second.iter().any(|value| value.copy_index != 1)
    {
        return Err(StoreDenied::InvalidSuperblockCopy);
    }
    if parsed_first
        .iter()
        .chain(parsed_second.iter())
        .any(|value| value.identity.store_uuid != expected_identity.store_uuid)
    {
        return Err(StoreDenied::SuperblockIdentityMismatch);
    }
    if parsed_first
        .iter()
        .chain(parsed_second.iter())
        .any(|value| value.geometry != expected_geometry)
    {
        return Err(StoreDenied::SuperblockGeometryMismatch);
    }
    if parsed_first
        .iter()
        .chain(parsed_second.iter())
        .any(|value| value.identity.generation > expected_identity.generation)
    {
        return Err(StoreDenied::StaleStoreGeneration);
    }
    let first = parsed_first
        .filter(|value| value.identity == expected_identity && value.geometry == expected_geometry);
    let second = parsed_second
        .filter(|value| value.identity == expected_identity && value.geometry == expected_geometry);
    let selected = match (first, second) {
        (None, None) => return Err(StoreDenied::NoValidSuperblock),
        (Some(value), None) | (None, Some(value)) => value,
        (Some(left), Some(right)) => {
            if left.selection_epoch == right.selection_epoch {
                if left.block_sha256 != right.block_sha256 {
                    return Err(StoreDenied::AmbiguousSuperblockSelection);
                }
                left
            } else if left.selection_epoch > right.selection_epoch {
                left
            } else {
                right
            }
        }
    };
    let source = if selected.copy_index == 0 {
        copy0
    } else {
        copy1
    };
    let mut bytes = [0u8; SUPERBLOCK_LEN];
    bytes.copy_from_slice(source);
    Ok(SelectedSuperblock {
        superblock: selected,
        bytes,
    })
}

pub fn plan_superblock_replacement(
    selected: &Superblock,
    next_generation: u64,
) -> Result<SelectedSuperblock, StoreDenied> {
    if next_generation <= selected.identity.generation {
        return Err(StoreDenied::StaleStoreGeneration);
    }
    let selection_epoch = selected
        .selection_epoch
        .checked_add(1)
        .ok_or(StoreDenied::StaleStoreGeneration)?;
    let copy_index = selected.copy_index ^ 1;
    let identity = StoreIdentity {
        generation: next_generation,
        ..selected.identity
    };
    let bytes = encode_superblock(
        copy_index,
        copy_index,
        identity,
        selection_epoch,
        selected.geometry,
    )?;
    let superblock = parse_superblock(&bytes)?;
    Ok(SelectedSuperblock { superblock, bytes })
}

fn superblock_hash(bytes: &[u8]) -> [u8; 32] {
    let mut canonical = [0u8; SUPERBLOCK_LEN];
    canonical.copy_from_slice(bytes);
    canonical[SB_HASH..SB_HASH_END].fill(0);
    sha256_bytes(&canonical)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum FrameKind {
    Prepare = 1,
    Data = 2,
    Commit = 3,
    Tombstone = 4,
}

impl FrameKind {
    fn parse(value: u8) -> Result<Self, StoreDenied> {
        match value {
            1 => Ok(Self::Prepare),
            2 => Ok(Self::Data),
            3 => Ok(Self::Commit),
            4 => Ok(Self::Tombstone),
            _ => Err(StoreDenied::InvalidFrameKind),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameBinding {
    pub identity: StoreIdentity,
    pub key: RecordKey,
    pub frame_sequence: u64,
    pub transaction_id: u64,
    pub record_version: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParsedFrame<'a> {
    pub kind: FrameKind,
    pub binding: FrameBinding,
    pub previous_frame_sha256: [u8; 32],
    pub payload_sha256: [u8; 32],
    pub frame_sha256: [u8; 32],
    pub payload: &'a [u8],
}

pub fn parse_frame<'a>(
    bytes: &'a [u8],
    expected_identity: StoreIdentity,
    expected_sequence: u64,
    expected_previous_hash: [u8; 32],
) -> Result<ParsedFrame<'a>, StoreDenied> {
    if bytes.len() != STORE_BLOCK_LEN {
        return Err(StoreDenied::InvalidFrameLength);
    }
    if &bytes[..8] != FRAME_MAGIC {
        return Err(StoreDenied::InvalidFrameMagic);
    }
    if read_u16(bytes, FRAME_VERSION) != FORMAT_VERSION {
        return Err(StoreDenied::UnsupportedFormatVersion);
    }
    let kind = FrameKind::parse(bytes[FRAME_KIND])?;
    if bytes[FRAME_FLAGS] != 0 {
        return Err(StoreDenied::InvalidFrameFlags);
    }
    if read_u16(bytes, FRAME_HEADER_SIZE) as usize != FRAME_HEADER_LEN {
        return Err(StoreDenied::InvalidFrameHeaderLength);
    }
    if read_u16(bytes, FRAME_RESERVED) != 0 || read_u32(bytes, FRAME_RESERVED_TAIL) != 0 {
        return Err(StoreDenied::InvalidFrameReservedBytes);
    }
    if read_u32(bytes, FRAME_LEN) as usize != STORE_BLOCK_LEN {
        return Err(StoreDenied::InvalidFrameLength);
    }
    let payload_len = read_u32(bytes, FRAME_PAYLOAD_LEN) as usize;
    if payload_len > MAX_FRAME_PAYLOAD_LEN {
        return Err(StoreDenied::InvalidFramePayloadLength);
    }
    let frame_sequence = read_u64(bytes, FRAME_SEQUENCE);
    if frame_sequence != expected_sequence {
        return Err(StoreDenied::FrameSequenceMismatch);
    }
    let transaction_id = read_u64(bytes, FRAME_TRANSACTION_ID);
    if transaction_id == 0 {
        return Err(StoreDenied::TransactionIdMissing);
    }
    let record_version = read_u64(bytes, FRAME_RECORD_VERSION);
    if record_version == 0 {
        return Err(StoreDenied::RecordVersionMissing);
    }

    let mut store_uuid = [0u8; 16];
    store_uuid.copy_from_slice(&bytes[FRAME_STORE_UUID..FRAME_STORE_UUID + 16]);
    let identity = StoreIdentity {
        store_uuid,
        generation: read_u64(bytes, FRAME_STORE_GENERATION),
    };
    if identity != expected_identity {
        return Err(StoreDenied::FrameStoreIdentityMismatch);
    }
    let mut namespace = [0u8; 16];
    namespace.copy_from_slice(&bytes[FRAME_NAMESPACE..FRAME_NAMESPACE + 16]);
    let mut record_id = [0u8; 16];
    record_id.copy_from_slice(&bytes[FRAME_RECORD_ID..FRAME_RECORD_ID + 16]);
    let key = RecordKey {
        namespace,
        record_id,
    };
    if key.namespace == [0; 16] || key.record_id == [0; 16] {
        return Err(StoreDenied::RecordKeyMissing);
    }

    let mut previous_frame_sha256 = [0u8; 32];
    previous_frame_sha256.copy_from_slice(&bytes[FRAME_PREVIOUS_HASH..FRAME_PREVIOUS_HASH + 32]);
    if previous_frame_sha256 != expected_previous_hash {
        return Err(StoreDenied::PreviousFrameHashMismatch);
    }
    let mut payload_sha256 = [0u8; 32];
    payload_sha256.copy_from_slice(&bytes[FRAME_PAYLOAD_HASH..FRAME_PAYLOAD_HASH + 32]);
    let payload = &bytes[FRAME_HEADER_LEN..FRAME_HEADER_LEN + payload_len];
    if sha256_bytes(payload) != payload_sha256 {
        return Err(StoreDenied::PayloadHashMismatch);
    }
    if !all_zero(&bytes[FRAME_HEADER_LEN + payload_len..]) {
        return Err(StoreDenied::FramePaddingNonzero);
    }

    let mut stored_header_hash = [0u8; 32];
    stored_header_hash.copy_from_slice(&bytes[FRAME_HEADER_HASH..FRAME_HEADER_HASH_END]);
    if frame_header_hash(bytes) != stored_header_hash {
        return Err(StoreDenied::HeaderHashMismatch);
    }
    if crc32_zeroed(bytes, FRAME_CRC32, 4) != read_u32(bytes, FRAME_CRC32) {
        return Err(StoreDenied::FrameCrcMismatch);
    }

    Ok(ParsedFrame {
        kind,
        binding: FrameBinding {
            identity,
            key,
            frame_sequence,
            transaction_id,
            record_version,
        },
        previous_frame_sha256,
        payload_sha256,
        frame_sha256: sha256_bytes(bytes),
        payload,
    })
}

fn encode_frame(
    kind: FrameKind,
    binding: FrameBinding,
    previous_frame_sha256: [u8; 32],
    payload: &[u8],
) -> Result<[u8; STORE_BLOCK_LEN], StoreDenied> {
    if payload.len() > MAX_FRAME_PAYLOAD_LEN {
        return Err(StoreDenied::InvalidFramePayloadLength);
    }
    if binding.identity.store_uuid == [0; 16] {
        return Err(StoreDenied::StoreUuidMissing);
    }
    if binding.identity.generation == 0 {
        return Err(StoreDenied::StaleStoreGeneration);
    }
    if binding.frame_sequence == 0 {
        return Err(StoreDenied::FrameSequenceMismatch);
    }
    if binding.transaction_id == 0 {
        return Err(StoreDenied::TransactionIdMissing);
    }
    if binding.record_version == 0 {
        return Err(StoreDenied::RecordVersionMissing);
    }
    if binding.key.namespace == [0; 16] || binding.key.record_id == [0; 16] {
        return Err(StoreDenied::RecordKeyMissing);
    }

    let mut out = [0u8; STORE_BLOCK_LEN];
    out[..8].copy_from_slice(FRAME_MAGIC);
    write_u16(&mut out, FRAME_VERSION, FORMAT_VERSION);
    out[FRAME_KIND] = kind as u8;
    write_u16(&mut out, FRAME_HEADER_SIZE, FRAME_HEADER_LEN as u16);
    write_u32(&mut out, FRAME_LEN, STORE_BLOCK_LEN as u32);
    write_u32(&mut out, FRAME_PAYLOAD_LEN, payload.len() as u32);
    write_u64(&mut out, FRAME_SEQUENCE, binding.frame_sequence);
    write_u64(&mut out, FRAME_TRANSACTION_ID, binding.transaction_id);
    write_u64(&mut out, FRAME_RECORD_VERSION, binding.record_version);
    write_u64(
        &mut out,
        FRAME_STORE_GENERATION,
        binding.identity.generation,
    );
    out[FRAME_STORE_UUID..FRAME_STORE_UUID + 16].copy_from_slice(&binding.identity.store_uuid);
    out[FRAME_NAMESPACE..FRAME_NAMESPACE + 16].copy_from_slice(&binding.key.namespace);
    out[FRAME_RECORD_ID..FRAME_RECORD_ID + 16].copy_from_slice(&binding.key.record_id);
    out[FRAME_PREVIOUS_HASH..FRAME_PREVIOUS_HASH + 32].copy_from_slice(&previous_frame_sha256);
    out[FRAME_PAYLOAD_HASH..FRAME_PAYLOAD_HASH + 32].copy_from_slice(&sha256_bytes(payload));
    out[FRAME_HEADER_LEN..FRAME_HEADER_LEN + payload.len()].copy_from_slice(payload);
    let header_hash = frame_header_hash(&out);
    out[FRAME_HEADER_HASH..FRAME_HEADER_HASH_END].copy_from_slice(&header_hash);
    let checksum = crc32_zeroed(&out, FRAME_CRC32, 4);
    write_u32(&mut out, FRAME_CRC32, checksum);
    Ok(out)
}

fn frame_header_hash(bytes: &[u8]) -> [u8; 32] {
    let mut canonical = [0u8; FRAME_HEADER_LEN];
    canonical.copy_from_slice(&bytes[..FRAME_HEADER_LEN]);
    canonical[FRAME_HEADER_HASH..FRAME_HEADER_HASH_END].fill(0);
    canonical[FRAME_CRC32..FRAME_CRC32 + 4].fill(0);
    sha256_bytes(&canonical)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordOperation {
    Put,
    Delete,
}

impl RecordOperation {
    const fn encoded(self) -> u8 {
        match self {
            Self::Put => 1,
            Self::Delete => 2,
        }
    }

    fn parse(value: u8) -> Result<Self, StoreDenied> {
        match value {
            1 => Ok(Self::Put),
            2 => Ok(Self::Delete),
            _ => Err(StoreDenied::PrepareDescriptorInvalid),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PrepareDescriptor {
    operation: RecordOperation,
    data_frame_count: u32,
    payload_len: u64,
    payload_sha256: [u8; 32],
}

fn encode_prepare_descriptor(descriptor: PrepareDescriptor) -> [u8; PREPARE_DESCRIPTOR_LEN] {
    let mut out = [0u8; PREPARE_DESCRIPTOR_LEN];
    out[0] = descriptor.operation.encoded();
    write_u32(&mut out, 4, descriptor.data_frame_count);
    write_u64(&mut out, 8, descriptor.payload_len);
    out[16..48].copy_from_slice(&descriptor.payload_sha256);
    out
}

fn parse_prepare_descriptor(bytes: &[u8]) -> Result<PrepareDescriptor, StoreDenied> {
    if bytes.len() != PREPARE_DESCRIPTOR_LEN || !all_zero(&bytes[1..4]) {
        return Err(StoreDenied::PrepareDescriptorInvalid);
    }
    let operation = RecordOperation::parse(bytes[0])?;
    let data_frame_count = read_u32(bytes, 4);
    let payload_len = read_u64(bytes, 8);
    let mut payload_sha256 = [0u8; 32];
    payload_sha256.copy_from_slice(&bytes[16..48]);
    match operation {
        RecordOperation::Put => {
            if data_frame_count == 0
                || data_frame_count > MAX_DATA_FRAME_COUNT
                || payload_len == 0
                || payload_len > MAX_RECORD_PAYLOAD_LEN as u64
            {
                return Err(StoreDenied::PrepareDescriptorInvalid);
            }
        }
        RecordOperation::Delete => {
            if data_frame_count != 0 || payload_len != 0 || payload_sha256 != sha256_bytes(&[]) {
                return Err(StoreDenied::PrepareDescriptorInvalid);
            }
        }
    }
    Ok(PrepareDescriptor {
        operation,
        data_frame_count,
        payload_len,
        payload_sha256,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CommitDescriptor {
    operation: RecordOperation,
    data_frame_count: u32,
    prepare_frame_sha256: [u8; 32],
    payload_sha256: [u8; 32],
}

fn encode_commit_descriptor(descriptor: CommitDescriptor) -> [u8; COMMIT_DESCRIPTOR_LEN] {
    let mut out = [0u8; COMMIT_DESCRIPTOR_LEN];
    out[0] = descriptor.operation.encoded();
    write_u32(&mut out, 4, descriptor.data_frame_count);
    out[8..40].copy_from_slice(&descriptor.prepare_frame_sha256);
    out[40..72].copy_from_slice(&descriptor.payload_sha256);
    out
}

fn parse_commit_descriptor(bytes: &[u8]) -> Result<CommitDescriptor, StoreDenied> {
    if bytes.len() != COMMIT_DESCRIPTOR_LEN || !all_zero(&bytes[1..4]) {
        return Err(StoreDenied::CommitDescriptorInvalid);
    }
    let operation = match bytes[0] {
        1 => RecordOperation::Put,
        2 => RecordOperation::Delete,
        _ => return Err(StoreDenied::CommitDescriptorInvalid),
    };
    let mut prepare_frame_sha256 = [0u8; 32];
    prepare_frame_sha256.copy_from_slice(&bytes[8..40]);
    let mut payload_sha256 = [0u8; 32];
    payload_sha256.copy_from_slice(&bytes[40..72]);
    Ok(CommitDescriptor {
        operation,
        data_frame_count: read_u32(bytes, 4),
        prepare_frame_sha256,
        payload_sha256,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordValue {
    Present(Vec<u8>),
    Tombstone,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedRecord {
    pub key: RecordKey,
    pub transaction_id: u64,
    pub record_version: u64,
    pub commit_frame_sha256: [u8; 32],
    pub payload_sha256: [u8; 32],
    pub value: RecordValue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayTail {
    ZeroFilled,
    RegionExhausted,
    IncompleteTransaction,
    Corrupt,
}

impl ReplayTail {
    pub const fn appendable(self) -> bool {
        matches!(
            self,
            Self::ZeroFilled | Self::RegionExhausted | Self::IncompleteTransaction
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ObservedVersion {
    key: RecordKey,
    version: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayState {
    pub identity: StoreIdentity,
    pub records: Vec<CommittedRecord>,
    pub locked_namespaces: Vec<[u8; 16]>,
    pub tail: ReplayTail,
    pub tail_reason: Option<StoreDenied>,
    pub chain_locked: bool,
    pub next_write_offset: u64,
    pub last_frame_sequence: u64,
    pub last_frame_sha256: [u8; 32],
    pub max_observed_transaction_id: u64,
    observed_versions: Vec<ObservedVersion>,
}

impl ReplayState {
    pub fn record(&self, key: RecordKey) -> Result<Option<&CommittedRecord>, StoreDenied> {
        if self.chain_locked {
            return Err(StoreDenied::StoreChainLocked);
        }
        if self.namespace_locked(key.namespace) {
            return Err(StoreDenied::NamespaceLocked);
        }
        Ok(self.records.iter().find(|record| record.key == key))
    }

    pub fn namespace_locked(&self, namespace: [u8; 16]) -> bool {
        self.locked_namespaces.contains(&namespace)
    }

    fn observed_version(&self, key: RecordKey) -> u64 {
        self.observed_versions
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.version)
            .unwrap_or(0)
    }

    fn committed_version(&self, key: RecordKey) -> Option<u64> {
        self.records
            .iter()
            .find(|record| record.key == key)
            .map(|record| record.record_version)
    }

    fn observe_version(&mut self, key: RecordKey, version: u64) {
        if let Some(entry) = self
            .observed_versions
            .iter_mut()
            .find(|entry| entry.key == key)
        {
            entry.version = entry.version.max(version);
        } else {
            self.observed_versions
                .push(ObservedVersion { key, version });
        }
    }

    fn lock_namespace(&mut self, namespace: [u8; 16]) {
        if !self.locked_namespaces.contains(&namespace) {
            self.locked_namespaces.push(namespace);
        }
        self.records
            .retain(|record| record.key.namespace != namespace);
    }

    fn lock_chain(&mut self, reason: StoreDenied) {
        self.chain_locked = true;
        self.tail = ReplayTail::Corrupt;
        self.tail_reason = Some(reason);
        self.records.clear();
    }
}

struct PendingTransaction {
    binding: FrameBinding,
    descriptor: PrepareDescriptor,
    prepare_frame_sha256: [u8; 32],
    payload: Vec<u8>,
    data_frame_count: u32,
    tombstone_seen: bool,
    invalid: bool,
}

pub fn replay_log(bytes: &[u8], identity: StoreIdentity) -> Result<ReplayState, StoreDenied> {
    if identity.store_uuid == [0; 16] {
        return Err(StoreDenied::StoreUuidMissing);
    }
    if identity.generation == 0 {
        return Err(StoreDenied::StaleStoreGeneration);
    }
    if bytes.len() % STORE_BLOCK_LEN != 0 {
        return Err(StoreDenied::RegionNotBlockAligned);
    }

    let mut state = ReplayState {
        identity,
        records: Vec::new(),
        locked_namespaces: Vec::new(),
        tail: ReplayTail::RegionExhausted,
        tail_reason: None,
        chain_locked: false,
        next_write_offset: 0,
        last_frame_sequence: 0,
        last_frame_sha256: ZERO_HASH,
        max_observed_transaction_id: 0,
        observed_versions: Vec::new(),
    };
    let mut pending: Option<PendingTransaction> = None;
    let mut offset = 0usize;

    while offset < bytes.len() {
        let frame_bytes = &bytes[offset..offset + STORE_BLOCK_LEN];
        if all_zero(frame_bytes) {
            if all_zero(&bytes[offset..]) {
                state.tail = if pending.is_some() {
                    ReplayTail::IncompleteTransaction
                } else {
                    ReplayTail::ZeroFilled
                };
                state.next_write_offset = offset as u64;
                return Ok(state);
            }
            state.next_write_offset = offset as u64;
            state.lock_chain(StoreDenied::InvalidFrameMagic);
            return Ok(state);
        }

        let expected_sequence = state
            .last_frame_sequence
            .checked_add(1)
            .ok_or(StoreDenied::FrameSequenceOverflow)?;
        let frame = match parse_frame(
            frame_bytes,
            identity,
            expected_sequence,
            state.last_frame_sha256,
        ) {
            Ok(frame) => frame,
            Err(StoreDenied::FrameStoreIdentityMismatch) => {
                return Err(StoreDenied::FrameStoreIdentityMismatch)
            }
            Err(reason) => {
                state.next_write_offset = offset as u64;
                state.lock_chain(reason);
                return Ok(state);
            }
        };

        state.last_frame_sequence = frame.binding.frame_sequence;
        state.last_frame_sha256 = frame.frame_sha256;
        let prior_max_transaction_id = state.max_observed_transaction_id;
        let prior_observed_version = state.observed_version(frame.binding.key);
        state.max_observed_transaction_id = state
            .max_observed_transaction_id
            .max(frame.binding.transaction_id);
        state.observe_version(frame.binding.key, frame.binding.record_version);
        offset += STORE_BLOCK_LEN;
        state.next_write_offset = offset as u64;

        match frame.kind {
            FrameKind::Prepare => {
                // A later PREPARE abandons a complete, hash-valid transaction
                // that never reached COMMIT. Its ids stay burned, but it never
                // becomes authoritative and does not poison the prior commit.
                pending.take();
                let descriptor = match parse_prepare_descriptor(frame.payload) {
                    Ok(value) => value,
                    Err(_) => {
                        state.lock_namespace(frame.binding.key.namespace);
                        continue;
                    }
                };
                let stale_transaction = prior_max_transaction_id != 0
                    && frame.binding.transaction_id <= prior_max_transaction_id;
                let stale_version = prior_observed_version != 0
                    && frame.binding.record_version <= prior_observed_version;
                pending = Some(PendingTransaction {
                    binding: frame.binding,
                    descriptor,
                    prepare_frame_sha256: frame.frame_sha256,
                    payload: Vec::new(),
                    data_frame_count: 0,
                    tombstone_seen: false,
                    invalid: stale_transaction || stale_version,
                });
            }
            FrameKind::Data => {
                let Some(active) = pending.as_mut() else {
                    state.lock_namespace(frame.binding.key.namespace);
                    continue;
                };
                if !same_transaction(active.binding, frame.binding)
                    || active.descriptor.operation != RecordOperation::Put
                    || active.data_frame_count >= active.descriptor.data_frame_count
                {
                    active.invalid = true;
                    continue;
                }
                let next_len = match active.payload.len().checked_add(frame.payload.len()) {
                    Some(value) => value,
                    None => {
                        active.invalid = true;
                        continue;
                    }
                };
                if next_len > active.descriptor.payload_len as usize
                    || next_len > MAX_RECORD_PAYLOAD_LEN
                {
                    active.invalid = true;
                    continue;
                }
                active.payload.extend_from_slice(frame.payload);
                active.data_frame_count += 1;
            }
            FrameKind::Tombstone => {
                let Some(active) = pending.as_mut() else {
                    state.lock_namespace(frame.binding.key.namespace);
                    continue;
                };
                if !same_transaction(active.binding, frame.binding)
                    || active.descriptor.operation != RecordOperation::Delete
                    || active.tombstone_seen
                    || !frame.payload.is_empty()
                {
                    active.invalid = true;
                    continue;
                }
                active.tombstone_seen = true;
            }
            FrameKind::Commit => {
                let Some(active) = pending.take() else {
                    state.lock_namespace(frame.binding.key.namespace);
                    continue;
                };
                if !same_transaction(active.binding, frame.binding) {
                    state.lock_namespace(active.binding.key.namespace);
                    state.lock_namespace(frame.binding.key.namespace);
                    continue;
                }
                let commit = match parse_commit_descriptor(frame.payload) {
                    Ok(value) => value,
                    Err(_) => {
                        state.lock_namespace(frame.binding.key.namespace);
                        continue;
                    }
                };
                if !committed_transaction_valid(&active, commit) {
                    state.lock_namespace(frame.binding.key.namespace);
                    continue;
                }
                let previous_version = state.committed_version(frame.binding.key).unwrap_or(0);
                if active.invalid || frame.binding.record_version <= previous_version {
                    state.lock_namespace(frame.binding.key.namespace);
                    continue;
                }
                let value = match active.descriptor.operation {
                    RecordOperation::Put => RecordValue::Present(active.payload),
                    RecordOperation::Delete => RecordValue::Tombstone,
                };
                let record = CommittedRecord {
                    key: frame.binding.key,
                    transaction_id: frame.binding.transaction_id,
                    record_version: frame.binding.record_version,
                    commit_frame_sha256: frame.frame_sha256,
                    payload_sha256: active.descriptor.payload_sha256,
                    value,
                };
                if let Some(existing) = state
                    .records
                    .iter_mut()
                    .find(|record| record.key == frame.binding.key)
                {
                    *existing = record;
                } else {
                    state.records.push(record);
                }
            }
        }
    }

    if pending.is_some() {
        state.tail = ReplayTail::IncompleteTransaction;
    }
    Ok(state)
}

fn same_transaction(left: FrameBinding, right: FrameBinding) -> bool {
    left.identity == right.identity
        && left.key == right.key
        && left.transaction_id == right.transaction_id
        && left.record_version == right.record_version
}

fn committed_transaction_valid(active: &PendingTransaction, commit: CommitDescriptor) -> bool {
    if commit.operation != active.descriptor.operation
        || commit.data_frame_count != active.descriptor.data_frame_count
        || commit.prepare_frame_sha256 != active.prepare_frame_sha256
        || commit.payload_sha256 != active.descriptor.payload_sha256
    {
        return false;
    }
    match active.descriptor.operation {
        RecordOperation::Put => {
            !active.tombstone_seen
                && active.data_frame_count == active.descriptor.data_frame_count
                && active.payload.len() as u64 == active.descriptor.payload_len
                && sha256_bytes(&active.payload) == active.descriptor.payload_sha256
        }
        RecordOperation::Delete => {
            active.tombstone_seen
                && active.data_frame_count == 0
                && active.payload.is_empty()
                && active.descriptor.payload_sha256 == sha256_bytes(&[])
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransactionRequest {
    pub identity: StoreIdentity,
    pub key: RecordKey,
    pub operation: RecordOperation,
    pub expected_committed_version: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedFrame {
    pub offset: u64,
    pub kind: FrameKind,
    pub binding: FrameBinding,
    pub previous_frame_sha256: [u8; 32],
    pub frame_sha256: [u8; 32],
    pub bytes: [u8; STORE_BLOCK_LEN],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionPlan {
    pub identity: StoreIdentity,
    pub key: RecordKey,
    pub operation: RecordOperation,
    pub transaction_id: u64,
    pub record_version: u64,
    pub payload_sha256: [u8; 32],
    pub frames: Vec<PlannedFrame>,
    pub start_offset: u64,
    pub end_offset: u64,
    pub final_frame_sha256: [u8; 32],
}

pub fn plan_transaction(
    replay: &ReplayState,
    request: TransactionRequest,
    payload: &[u8],
    region_byte_len: u64,
) -> Result<TransactionPlan, StoreDenied> {
    if replay.identity != request.identity {
        return Err(StoreDenied::FrameStoreIdentityMismatch);
    }
    if replay.chain_locked {
        return Err(StoreDenied::StoreChainLocked);
    }
    if replay.namespace_locked(request.key.namespace) {
        return Err(StoreDenied::NamespaceLocked);
    }
    if !replay.tail.appendable() {
        return Err(StoreDenied::TailNotAppendable);
    }
    if replay.next_write_offset > region_byte_len
        || replay.next_write_offset % STORE_BLOCK_LEN as u64 != 0
        || region_byte_len % STORE_BLOCK_LEN as u64 != 0
    {
        return Err(StoreDenied::BoundsDenied);
    }
    let committed_version = replay.committed_version(request.key);
    if committed_version != request.expected_committed_version {
        return Err(StoreDenied::ExpectedVersionMismatch);
    }
    match request.operation {
        RecordOperation::Put if payload.is_empty() => {
            return Err(StoreDenied::RecordPayloadTooLarge)
        }
        RecordOperation::Put if payload.len() > MAX_RECORD_PAYLOAD_LEN => {
            return Err(StoreDenied::RecordPayloadTooLarge)
        }
        RecordOperation::Delete if !payload.is_empty() => {
            return Err(StoreDenied::TransactionPayloadMismatch)
        }
        _ => {}
    }

    let transaction_id = replay
        .max_observed_transaction_id
        .checked_add(1)
        .ok_or(StoreDenied::TransactionIdOverflow)?;
    let record_version = replay
        .observed_version(request.key)
        .checked_add(1)
        .ok_or(StoreDenied::RecordVersionOverflow)?;
    let data_frame_count = match request.operation {
        RecordOperation::Put => payload.len().div_ceil(MAX_FRAME_PAYLOAD_LEN) as u32,
        RecordOperation::Delete => 0,
    };
    if data_frame_count > MAX_DATA_FRAME_COUNT {
        return Err(StoreDenied::TooManyDataFrames);
    }
    let frame_count = match request.operation {
        RecordOperation::Put => data_frame_count as u64 + 2,
        RecordOperation::Delete => 3,
    };
    let required = frame_count
        .checked_mul(STORE_BLOCK_LEN as u64)
        .ok_or(StoreDenied::CapacityExhausted)?;
    let end_offset = replay
        .next_write_offset
        .checked_add(required)
        .ok_or(StoreDenied::CapacityExhausted)?;
    if end_offset > region_byte_len {
        return Err(StoreDenied::CapacityExhausted);
    }

    let payload_sha256 = sha256_bytes(payload);
    let descriptor = PrepareDescriptor {
        operation: request.operation,
        data_frame_count,
        payload_len: payload.len() as u64,
        payload_sha256,
    };
    let mut frames = Vec::with_capacity(frame_count as usize);
    let mut sequence = replay
        .last_frame_sequence
        .checked_add(1)
        .ok_or(StoreDenied::FrameSequenceOverflow)?;
    let mut previous_hash = replay.last_frame_sha256;
    let mut offset = replay.next_write_offset;
    let binding = FrameBinding {
        identity: request.identity,
        key: request.key,
        frame_sequence: sequence,
        transaction_id,
        record_version,
    };
    push_planned_frame(
        &mut frames,
        offset,
        FrameKind::Prepare,
        binding,
        previous_hash,
        &encode_prepare_descriptor(descriptor),
    )?;
    let prepare_hash = frames[0].frame_sha256;
    previous_hash = prepare_hash;
    offset += STORE_BLOCK_LEN as u64;

    match request.operation {
        RecordOperation::Put => {
            for chunk in payload.chunks(MAX_FRAME_PAYLOAD_LEN) {
                sequence = sequence
                    .checked_add(1)
                    .ok_or(StoreDenied::FrameSequenceOverflow)?;
                let binding = FrameBinding {
                    frame_sequence: sequence,
                    ..binding
                };
                push_planned_frame(
                    &mut frames,
                    offset,
                    FrameKind::Data,
                    binding,
                    previous_hash,
                    chunk,
                )?;
                previous_hash = frames.last().expect("planned data frame").frame_sha256;
                offset += STORE_BLOCK_LEN as u64;
            }
        }
        RecordOperation::Delete => {
            sequence = sequence
                .checked_add(1)
                .ok_or(StoreDenied::FrameSequenceOverflow)?;
            let binding = FrameBinding {
                frame_sequence: sequence,
                ..binding
            };
            push_planned_frame(
                &mut frames,
                offset,
                FrameKind::Tombstone,
                binding,
                previous_hash,
                &[],
            )?;
            previous_hash = frames.last().expect("planned tombstone frame").frame_sha256;
            offset += STORE_BLOCK_LEN as u64;
        }
    }

    sequence = sequence
        .checked_add(1)
        .ok_or(StoreDenied::FrameSequenceOverflow)?;
    let binding = FrameBinding {
        frame_sequence: sequence,
        ..binding
    };
    let commit = CommitDescriptor {
        operation: request.operation,
        data_frame_count,
        prepare_frame_sha256: prepare_hash,
        payload_sha256,
    };
    push_planned_frame(
        &mut frames,
        offset,
        FrameKind::Commit,
        binding,
        previous_hash,
        &encode_commit_descriptor(commit),
    )?;
    let final_frame_sha256 = frames.last().expect("planned commit frame").frame_sha256;

    Ok(TransactionPlan {
        identity: request.identity,
        key: request.key,
        operation: request.operation,
        transaction_id,
        record_version,
        payload_sha256,
        frames,
        start_offset: replay.next_write_offset,
        end_offset,
        final_frame_sha256,
    })
}

fn push_planned_frame(
    frames: &mut Vec<PlannedFrame>,
    offset: u64,
    kind: FrameKind,
    binding: FrameBinding,
    previous_frame_sha256: [u8; 32],
    payload: &[u8],
) -> Result<(), StoreDenied> {
    let bytes = encode_frame(kind, binding, previous_frame_sha256, payload)?;
    let frame_sha256 = sha256_bytes(&bytes);
    frames.push(PlannedFrame {
        offset,
        kind,
        binding,
        previous_frame_sha256,
        frame_sha256,
        bytes,
    });
    Ok(())
}

pub fn verify_frame_readback(planned: &PlannedFrame, readback: &[u8]) -> Result<(), StoreDenied> {
    if readback != planned.bytes {
        return Err(StoreDenied::ReadbackMismatch);
    }
    let parsed = parse_frame(
        readback,
        planned.binding.identity,
        planned.binding.frame_sequence,
        planned.previous_frame_sha256,
    )?;
    if parsed.kind != planned.kind
        || parsed.binding != planned.binding
        || parsed.frame_sha256 != planned.frame_sha256
    {
        return Err(StoreDenied::ReadbackMismatch);
    }
    Ok(())
}

pub fn validate_transaction_plan(
    plan: &TransactionPlan,
    region_byte_len: u64,
) -> Result<(), StoreDenied> {
    if plan.frames.len() < 3
        || plan.frames.first().map(|frame| frame.kind) != Some(FrameKind::Prepare)
        || plan.frames.last().map(|frame| frame.kind) != Some(FrameKind::Commit)
        || plan.start_offset % STORE_BLOCK_LEN as u64 != 0
        || region_byte_len % STORE_BLOCK_LEN as u64 != 0
    {
        return Err(StoreDenied::TransactionProtocolInvalid);
    }

    let mut expected_offset = plan.start_offset;
    let mut expected_sequence = plan.frames[0].binding.frame_sequence;
    let mut expected_previous_hash = plan.frames[0].previous_frame_sha256;
    for (index, frame) in plan.frames.iter().enumerate() {
        if frame.offset != expected_offset
            || frame.binding.identity != plan.identity
            || frame.binding.key != plan.key
            || frame.binding.transaction_id != plan.transaction_id
            || frame.binding.record_version != plan.record_version
            || frame.binding.frame_sequence != expected_sequence
            || frame.previous_frame_sha256 != expected_previous_hash
            || (index != 0 && frame.kind == FrameKind::Prepare)
            || (index + 1 != plan.frames.len() && frame.kind == FrameKind::Commit)
        {
            return Err(StoreDenied::TransactionBindingMismatch);
        }
        verify_frame_readback(frame, &frame.bytes)?;
        expected_offset = expected_offset
            .checked_add(STORE_BLOCK_LEN as u64)
            .ok_or(StoreDenied::CapacityExhausted)?;
        expected_sequence = expected_sequence
            .checked_add(1)
            .ok_or(StoreDenied::FrameSequenceOverflow)?;
        expected_previous_hash = frame.frame_sha256;
    }

    let prepare_frame = &plan.frames[0];
    let prepare_payload_len = read_u32(&prepare_frame.bytes, FRAME_PAYLOAD_LEN) as usize;
    let prepare = parse_prepare_descriptor(
        &prepare_frame.bytes[FRAME_HEADER_LEN..FRAME_HEADER_LEN + prepare_payload_len],
    )?;
    if prepare.operation != plan.operation || prepare.payload_sha256 != plan.payload_sha256 {
        return Err(StoreDenied::TransactionPayloadMismatch);
    }

    let mut payload = Vec::new();
    match plan.operation {
        RecordOperation::Put => {
            if prepare.data_frame_count as usize != plan.frames.len() - 2 {
                return Err(StoreDenied::TransactionProtocolInvalid);
            }
            for frame in &plan.frames[1..plan.frames.len() - 1] {
                if frame.kind != FrameKind::Data {
                    return Err(StoreDenied::TransactionProtocolInvalid);
                }
                let payload_len = read_u32(&frame.bytes, FRAME_PAYLOAD_LEN) as usize;
                payload.extend_from_slice(
                    &frame.bytes[FRAME_HEADER_LEN..FRAME_HEADER_LEN + payload_len],
                );
            }
            if payload.len() as u64 != prepare.payload_len
                || sha256_bytes(&payload) != plan.payload_sha256
            {
                return Err(StoreDenied::TransactionPayloadMismatch);
            }
        }
        RecordOperation::Delete => {
            if plan.frames.len() != 3
                || plan.frames[1].kind != FrameKind::Tombstone
                || read_u32(&plan.frames[1].bytes, FRAME_PAYLOAD_LEN) != 0
                || prepare.data_frame_count != 0
                || prepare.payload_len != 0
            {
                return Err(StoreDenied::TransactionProtocolInvalid);
            }
        }
    }

    let commit_frame = plan.frames.last().expect("validated plan has commit");
    let commit_payload_len = read_u32(&commit_frame.bytes, FRAME_PAYLOAD_LEN) as usize;
    let commit = parse_commit_descriptor(
        &commit_frame.bytes[FRAME_HEADER_LEN..FRAME_HEADER_LEN + commit_payload_len],
    )?;
    if commit.operation != plan.operation
        || commit.data_frame_count != prepare.data_frame_count
        || commit.prepare_frame_sha256 != prepare_frame.frame_sha256
        || commit.payload_sha256 != plan.payload_sha256
    {
        return Err(StoreDenied::TransactionPayloadMismatch);
    }
    if expected_offset != plan.end_offset
        || plan.end_offset > region_byte_len
        || plan.final_frame_sha256 != expected_previous_hash
    {
        return Err(StoreDenied::CapacityExhausted);
    }
    Ok(())
}

pub fn verify_committed_plan(
    replay: &ReplayState,
    plan: &TransactionPlan,
) -> Result<(), StoreDenied> {
    if replay.identity != plan.identity
        || replay.chain_locked
        || replay.namespace_locked(plan.key.namespace)
        || replay.last_frame_sha256 != plan.final_frame_sha256
        || replay.next_write_offset != plan.end_offset
    {
        return Err(StoreDenied::CommitReplayMismatch);
    }
    let record = replay
        .record(plan.key)?
        .ok_or(StoreDenied::CommitReplayMismatch)?;
    if record.transaction_id != plan.transaction_id
        || record.record_version != plan.record_version
        || record.payload_sha256 != plan.payload_sha256
    {
        return Err(StoreDenied::CommitReplayMismatch);
    }
    match (&record.value, plan.operation) {
        (RecordValue::Present(_), RecordOperation::Put)
        | (RecordValue::Tombstone, RecordOperation::Delete) => Ok(()),
        _ => Err(StoreDenied::CommitReplayMismatch),
    }
}

fn crc32_zeroed(bytes: &[u8], zero_offset: usize, zero_len: usize) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for (idx, byte) in bytes.iter().enumerate() {
        let value = if idx >= zero_offset && idx < zero_offset + zero_len {
            0
        } else {
            *byte
        };
        crc ^= value as u32;
        let mut bit = 0;
        while bit < 8 {
            let mask = if crc & 1 == 1 { 0xedb8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
            bit += 1;
        }
    }
    !crc
}

fn all_zero(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| *byte == 0)
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    (bytes[offset] as u16) | ((bytes[offset + 1] as u16) << 8)
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

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset] = value as u8;
    bytes[offset + 1] = (value >> 8) as u8;
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

    const STORE: StoreIdentity = StoreIdentity {
        store_uuid: *b"raios-store-test",
        generation: 7,
    };
    const KEY: RecordKey = RecordKey {
        namespace: *b"vault-----------",
        record_id: *b"wifi-home-------",
    };
    const GEOMETRY: StoreGeometry = StoreGeometry {
        partition_start_lba: 2048,
        partition_lba_count: 4096,
        log_start_lba: 2,
        log_lba_count: 4094,
        logical_sector_size: 512,
    };

    fn empty_region(blocks: usize) -> Vec<u8> {
        vec![0; blocks * STORE_BLOCK_LEN]
    }

    fn append_plan(region: &mut [u8], plan: &TransactionPlan, frame_count: usize) {
        for frame in plan.frames.iter().take(frame_count) {
            let start = frame.offset as usize;
            region[start..start + STORE_BLOCK_LEN].copy_from_slice(&frame.bytes);
        }
    }

    fn put(region: &mut [u8], expected_version: Option<u64>, payload: &[u8]) -> TransactionPlan {
        let before = replay_log(region, STORE).unwrap();
        let plan = plan_transaction(
            &before,
            TransactionRequest {
                identity: STORE,
                key: KEY,
                operation: RecordOperation::Put,
                expected_committed_version: expected_version,
            },
            payload,
            region.len() as u64,
        )
        .unwrap();
        let count = plan.frames.len();
        append_plan(region, &plan, count);
        plan
    }

    #[test]
    fn superblock_selects_newest_readback_candidate() {
        let first = encode_superblock(0, 0, STORE, 1, GEOMETRY).unwrap();
        let second = encode_superblock(1, 1, STORE, 2, GEOMETRY).unwrap();
        let selected = select_superblock(&first, &second, STORE, GEOMETRY).unwrap();

        assert_eq!(selected.superblock.copy_index, 1);
        assert_eq!(selected.superblock.selection_epoch, 2);
        assert_eq!(
            parse_superblock(&selected.bytes).unwrap(),
            selected.superblock
        );
    }

    #[test]
    fn corrupt_or_ambiguous_superblock_denies() {
        let first = encode_superblock(0, 0, STORE, 1, GEOMETRY).unwrap();
        let mut corrupt = encode_superblock(1, 1, STORE, 2, GEOMETRY).unwrap();
        corrupt[SB_GENERATION] ^= 1;
        assert_eq!(
            select_superblock(&corrupt, &corrupt, STORE, GEOMETRY),
            Err(StoreDenied::NoValidSuperblock)
        );

        let mut same_epoch_other_copy = encode_superblock(1, 1, STORE, 1, GEOMETRY).unwrap();
        assert_eq!(
            select_superblock(&first, &same_epoch_other_copy, STORE, GEOMETRY),
            Err(StoreDenied::AmbiguousSuperblockSelection)
        );

        same_epoch_other_copy[SB_HASH] ^= 1;
        assert_eq!(
            select_superblock(&first, &same_epoch_other_copy, STORE, GEOMETRY)
                .unwrap()
                .superblock
                .copy_index,
            0
        );
    }

    #[test]
    fn replacement_uses_other_copy_and_higher_generation() {
        let bytes = encode_superblock(0, 0, STORE, 4, GEOMETRY).unwrap();
        let selected = parse_superblock(&bytes).unwrap();
        let replacement = plan_superblock_replacement(&selected, 8).unwrap();

        assert_eq!(replacement.superblock.copy_index, 1);
        assert_eq!(replacement.superblock.selection_epoch, 5);
        assert_eq!(replacement.superblock.identity.generation, 8);
        assert_eq!(
            plan_superblock_replacement(&selected, 7),
            Err(StoreDenied::StaleStoreGeneration)
        );
    }

    #[test]
    fn put_is_invisible_until_commit_and_replays_exactly() {
        let mut region = empty_region(16);
        let before = replay_log(&region, STORE).unwrap();
        let plan = plan_transaction(
            &before,
            TransactionRequest {
                identity: STORE,
                key: KEY,
                operation: RecordOperation::Put,
                expected_committed_version: None,
            },
            b"ciphertext-envelope",
            region.len() as u64,
        )
        .unwrap();

        append_plan(&mut region, &plan, plan.frames.len() - 1);
        let torn = replay_log(&region, STORE).unwrap();
        assert_eq!(torn.tail, ReplayTail::IncompleteTransaction);
        assert_eq!(torn.record(KEY).unwrap(), None);

        let commit = plan.frames.last().unwrap();
        region[commit.offset as usize..commit.offset as usize + STORE_BLOCK_LEN]
            .copy_from_slice(&commit.bytes);
        let committed = replay_log(&region, STORE).unwrap();
        verify_committed_plan(&committed, &plan).unwrap();
        assert_eq!(
            committed.record(KEY).unwrap().unwrap().value,
            RecordValue::Present(b"ciphertext-envelope".to_vec())
        );
    }

    #[test]
    fn multi_frame_payload_and_tombstone_are_committed_transactions() {
        let mut region = empty_region(32);
        let payload = vec![0x5a; MAX_FRAME_PAYLOAD_LEN + 41];
        let put = put(&mut region, None, &payload);
        let committed = replay_log(&region, STORE).unwrap();
        assert_eq!(put.frames.len(), 4);
        assert_eq!(
            committed.record(KEY).unwrap().unwrap().value,
            RecordValue::Present(payload)
        );

        let delete = plan_transaction(
            &committed,
            TransactionRequest {
                identity: STORE,
                key: KEY,
                operation: RecordOperation::Delete,
                expected_committed_version: Some(1),
            },
            &[],
            region.len() as u64,
        )
        .unwrap();
        append_plan(&mut region, &delete, delete.frames.len());
        let replayed = replay_log(&region, STORE).unwrap();
        verify_committed_plan(&replayed, &delete).unwrap();
        assert_eq!(
            replayed.record(KEY).unwrap().unwrap().value,
            RecordValue::Tombstone
        );
    }

    #[test]
    fn incomplete_transaction_burns_ids_and_versions_without_publishing() {
        let mut region = empty_region(24);
        let first = replay_log(&region, STORE).unwrap();
        let abandoned = plan_transaction(
            &first,
            TransactionRequest {
                identity: STORE,
                key: KEY,
                operation: RecordOperation::Put,
                expected_committed_version: None,
            },
            b"never-committed",
            region.len() as u64,
        )
        .unwrap();
        append_plan(&mut region, &abandoned, abandoned.frames.len() - 1);

        let after_cut = replay_log(&region, STORE).unwrap();
        let replacement = plan_transaction(
            &after_cut,
            TransactionRequest {
                identity: STORE,
                key: KEY,
                operation: RecordOperation::Put,
                expected_committed_version: None,
            },
            b"new-value",
            region.len() as u64,
        )
        .unwrap();
        assert_eq!(replacement.transaction_id, abandoned.transaction_id + 1);
        assert_eq!(replacement.record_version, abandoned.record_version + 1);
        assert_eq!(
            replacement.start_offset,
            abandoned.end_offset - STORE_BLOCK_LEN as u64
        );
    }

    #[test]
    fn corruption_locks_chain_instead_of_returning_old_record() {
        let mut region = empty_region(16);
        put(&mut region, None, b"committed");
        let next_offset = replay_log(&region, STORE).unwrap().next_write_offset as usize;
        region[next_offset] = b'X';

        let replayed = replay_log(&region, STORE).unwrap();
        assert_eq!(replayed.tail, ReplayTail::Corrupt);
        assert!(replayed.chain_locked);
        assert_eq!(replayed.record(KEY), Err(StoreDenied::StoreChainLocked));
    }

    #[test]
    fn committed_payload_or_commit_tamper_locks_namespace() {
        let mut region = empty_region(16);
        let plan = put(&mut region, None, b"committed");
        let data_offset = plan.frames[1].offset as usize;
        region[data_offset + FRAME_HEADER_LEN] ^= 1;
        let replayed = replay_log(&region, STORE).unwrap();
        assert!(replayed.chain_locked);

        let mut region = empty_region(16);
        let plan = put(&mut region, None, b"committed");
        let commit_offset = plan.frames.last().unwrap().offset as usize;
        let commit_payload_hash = commit_offset + FRAME_HEADER_LEN + 40;
        region[commit_payload_hash] ^= 1;
        repair_frame_integrity(&mut region[commit_offset..commit_offset + STORE_BLOCK_LEN]);
        let replayed = replay_log(&region, STORE).unwrap();
        assert!(replayed.namespace_locked(KEY.namespace));
        assert_eq!(replayed.record(KEY), Err(StoreDenied::NamespaceLocked));
    }

    #[test]
    fn wrong_identity_generation_bounds_capacity_and_version_deny() {
        let region = empty_region(4);
        let replayed = replay_log(&region, STORE).unwrap();
        let mut request = TransactionRequest {
            identity: StoreIdentity {
                generation: STORE.generation + 1,
                ..STORE
            },
            key: KEY,
            operation: RecordOperation::Put,
            expected_committed_version: None,
        };
        assert_eq!(
            plan_transaction(&replayed, request, b"x", region.len() as u64),
            Err(StoreDenied::FrameStoreIdentityMismatch)
        );

        request.identity = STORE;
        request.expected_committed_version = Some(1);
        assert_eq!(
            plan_transaction(&replayed, request, b"x", region.len() as u64),
            Err(StoreDenied::ExpectedVersionMismatch)
        );
        request.expected_committed_version = None;
        assert_eq!(
            plan_transaction(&replayed, request, b"x", (STORE_BLOCK_LEN * 2) as u64),
            Err(StoreDenied::CapacityExhausted)
        );
        assert_eq!(
            plan_transaction(&replayed, request, b"x", 513),
            Err(StoreDenied::BoundsDenied)
        );
    }

    #[test]
    fn readback_must_match_exact_planned_frame() {
        let region = empty_region(8);
        let replayed = replay_log(&region, STORE).unwrap();
        let plan = plan_transaction(
            &replayed,
            TransactionRequest {
                identity: STORE,
                key: KEY,
                operation: RecordOperation::Put,
                expected_committed_version: None,
            },
            b"ciphertext",
            region.len() as u64,
        )
        .unwrap();
        let frame = &plan.frames[0];
        verify_frame_readback(frame, &frame.bytes).unwrap();
        let mut changed = frame.bytes;
        changed[FRAME_HEADER_LEN] ^= 1;
        assert_eq!(
            verify_frame_readback(frame, &changed),
            Err(StoreDenied::ReadbackMismatch)
        );
    }

    fn repair_frame_integrity(frame: &mut [u8]) {
        let payload_len = read_u32(frame, FRAME_PAYLOAD_LEN) as usize;
        let payload_hash = sha256_bytes(&frame[FRAME_HEADER_LEN..FRAME_HEADER_LEN + payload_len]);
        frame[FRAME_PAYLOAD_HASH..FRAME_PAYLOAD_HASH + 32].copy_from_slice(&payload_hash);
        frame[FRAME_HEADER_HASH..FRAME_HEADER_HASH_END].fill(0);
        write_u32(frame, FRAME_CRC32, 0);
        let header_hash = frame_header_hash(frame);
        frame[FRAME_HEADER_HASH..FRAME_HEADER_HASH_END].copy_from_slice(&header_hash);
        let checksum = crc32_zeroed(frame, FRAME_CRC32, 4);
        write_u32(frame, FRAME_CRC32, checksum);
    }

    #[test]
    fn imported_crc32_helper_matches_local_zero_free_path() {
        let bytes = b"structured-store";
        assert_eq!(
            crate::gpt_layout::crc32(bytes),
            crc32_zeroed(bytes, bytes.len(), 0)
        );
    }
}
