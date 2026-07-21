//! Bounded wire contract for one owner-custodied hardware fact capture.
//!
//! This module describes facts only. It grants no hardware access and makes no
//! freshness, attestation, manifest-readiness, or machine-identity claim.

use sha2::{Digest, Sha256};

pub const SURFACE_FACT_CAPTURE_SCHEMA_VERSION: u16 = 1;
pub const SURFACE_FACT_CAPTURE_MAX_PARTS: usize = 128;
pub const SURFACE_FACT_CAPTURE_MAX_TEXT_LEN: usize = 32;
pub const SURFACE_FACT_CAPTURE_MAX_PCI_BARS: usize = 6;
pub type CaptureId = [u8; 16];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CapturePartKind {
    Cpu = 1,
    SmbiosMemory = 2,
    LimineMemoryRegion = 3,
    PciDevice = 4,
    Completion = 255,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureHeader {
    pub schema_version: u16,
    pub capture_id: CaptureId,
    pub part_kind: CapturePartKind,
    pub part_index: u16,
    pub part_count: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundedText {
    bytes: [u8; SURFACE_FACT_CAPTURE_MAX_TEXT_LEN],
    len: u8,
}

impl BoundedText {
    pub fn new(value: &str) -> Result<Self, CaptureValidationError> {
        if value.len() > SURFACE_FACT_CAPTURE_MAX_TEXT_LEN {
            return Err(CaptureValidationError::StringTooLong);
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() || byte == b' ')
        {
            return Err(CaptureValidationError::InvalidStringEncoding);
        }
        let mut bytes = [0; SURFACE_FACT_CAPTURE_MAX_TEXT_LEN];
        bytes[..value.len()].copy_from_slice(value.as_bytes());
        Ok(Self {
            bytes,
            len: value.len() as u8,
        })
    }

    /// Constructs a decoded wire value. Validation remains fail-closed when
    /// the declared length exceeds the fixed storage or contains non-ASCII.
    pub const fn from_raw(bytes: [u8; SURFACE_FACT_CAPTURE_MAX_TEXT_LEN], len: u8) -> Self {
        Self { bytes, len }
    }

    pub fn as_bytes(&self) -> Result<&[u8], CaptureValidationError> {
        let len = self.len as usize;
        if len > self.bytes.len() {
            return Err(CaptureValidationError::StringTooLong);
        }
        let value = &self.bytes[..len];
        if !value
            .iter()
            .all(|byte| byte.is_ascii_graphic() || *byte == b' ')
        {
            return Err(CaptureValidationError::InvalidStringEncoding);
        }
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpuFact {
    pub leaf: u32,
    pub subleaf: u32,
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SmbiosMemoryFact {
    pub array_handle: u16,
    pub device_handle: u16,
    pub size_bytes: u64,
    pub speed_mt_s: u32,
    pub memory_type: u8,
    pub form_factor: u8,
    pub device_locator: BoundedText,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LimineMemoryRegionFact {
    pub base: u64,
    pub length: u64,
    /// Numeric Limine memory-map entry type as observed at boot.
    pub region_type: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PciBarKind {
    Io = 1,
    Memory32 = 2,
    Memory64 = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum PciBarStatus {
    Assigned = 1,
    Disabled = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PciBarFact {
    pub index: u8,
    pub kind: PciBarKind,
    pub base: u64,
    pub length: u64,
    pub status: PciBarStatus,
}

impl PciBarFact {
    pub const EMPTY: Self = Self {
        index: 0,
        kind: PciBarKind::Memory32,
        base: 0,
        length: 0,
        status: PciBarStatus::Disabled,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PciDeviceFact {
    pub segment: u16,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision: u8,
    /// Raw PCI configuration-space interrupt line and pin values.
    pub irq_line: u8,
    pub irq_pin: u8,
    pub bar_count: u8,
    pub bars: [PciBarFact; SURFACE_FACT_CAPTURE_MAX_PCI_BARS],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureCompletion {
    pub cpu_parts: u16,
    pub smbios_memory_parts: u16,
    pub limine_memory_region_parts: u16,
    pub pci_device_parts: u16,
    pub facts_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapturePayload {
    Cpu(CpuFact),
    SmbiosMemory(SmbiosMemoryFact),
    LimineMemoryRegion(LimineMemoryRegionFact),
    PciDevice(PciDeviceFact),
    Completion(CaptureCompletion),
}

impl CapturePayload {
    pub const fn kind(&self) -> CapturePartKind {
        match self {
            Self::Cpu(_) => CapturePartKind::Cpu,
            Self::SmbiosMemory(_) => CapturePartKind::SmbiosMemory,
            Self::LimineMemoryRegion(_) => CapturePartKind::LimineMemoryRegion,
            Self::PciDevice(_) => CapturePartKind::PciDevice,
            Self::Completion(_) => CapturePartKind::Completion,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureRecord {
    pub header: CaptureHeader,
    pub payload: CapturePayload,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum CaptureValidationError {
    EmptySeries = 1,
    TooManyRecords = 2,
    SchemaVersionMismatch = 3,
    MixedCaptureId = 4,
    InvalidPartCount = 5,
    PartCountMismatch = 6,
    PartIndexOutOfRange = 7,
    DuplicatePart = 8,
    MissingPart = 9,
    KindPayloadMismatch = 10,
    MissingCpu = 11,
    MissingSmbiosMemory = 12,
    MissingLimineMemoryRegion = 13,
    MissingPciDevice = 14,
    MissingCompletion = 15,
    MultipleCompletion = 16,
    CompletionNotLast = 17,
    CompletionCountMismatch = 18,
    DigestMismatch = 19,
    StringTooLong = 20,
    InvalidStringEncoding = 21,
    ZeroLengthMemoryRegion = 22,
    MemoryRegionOverflow = 23,
    ZeroSizedSmbiosMemory = 24,
    TooManyPciBars = 25,
    PciBarIndexOutOfRange = 26,
    ZeroLengthPciBar = 27,
    PciBarAddressOverflow = 28,
    DuplicatePciBarIndex = 29,
    PciDeviceOutOfRange = 30,
    PciFunctionOutOfRange = 31,
    PciIrqPinOutOfRange = 32,
    PciBar32BitAddressOverflow = 33,
    PciMemory64AtLastIndex = 34,
    PciBarSlotConflict = 35,
}

impl CaptureValidationError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptySeries => "surface_capture_empty_series",
            Self::TooManyRecords => "surface_capture_too_many_records",
            Self::SchemaVersionMismatch => "surface_capture_schema_version_mismatch",
            Self::MixedCaptureId => "surface_capture_mixed_capture_id",
            Self::InvalidPartCount => "surface_capture_invalid_part_count",
            Self::PartCountMismatch => "surface_capture_part_count_mismatch",
            Self::PartIndexOutOfRange => "surface_capture_part_index_out_of_range",
            Self::DuplicatePart => "surface_capture_duplicate_part",
            Self::MissingPart => "surface_capture_missing_part",
            Self::KindPayloadMismatch => "surface_capture_kind_payload_mismatch",
            Self::MissingCpu => "surface_capture_missing_cpu",
            Self::MissingSmbiosMemory => "surface_capture_missing_smbios_memory",
            Self::MissingLimineMemoryRegion => "surface_capture_missing_limine_memory_region",
            Self::MissingPciDevice => "surface_capture_missing_pci_device",
            Self::MissingCompletion => "surface_capture_missing_completion",
            Self::MultipleCompletion => "surface_capture_multiple_completion",
            Self::CompletionNotLast => "surface_capture_completion_not_last",
            Self::CompletionCountMismatch => "surface_capture_completion_count_mismatch",
            Self::DigestMismatch => "surface_capture_digest_mismatch",
            Self::StringTooLong => "surface_capture_string_too_long",
            Self::InvalidStringEncoding => "surface_capture_invalid_string_encoding",
            Self::ZeroLengthMemoryRegion => "surface_capture_zero_length_memory_region",
            Self::MemoryRegionOverflow => "surface_capture_memory_region_overflow",
            Self::ZeroSizedSmbiosMemory => "surface_capture_zero_sized_smbios_memory",
            Self::TooManyPciBars => "surface_capture_too_many_pci_bars",
            Self::PciBarIndexOutOfRange => "surface_capture_pci_bar_index_out_of_range",
            Self::ZeroLengthPciBar => "surface_capture_zero_length_pci_bar",
            Self::PciBarAddressOverflow => "surface_capture_pci_bar_address_overflow",
            Self::DuplicatePciBarIndex => "surface_capture_duplicate_pci_bar_index",
            Self::PciDeviceOutOfRange => "surface_capture_pci_device_out_of_range",
            Self::PciFunctionOutOfRange => "surface_capture_pci_function_out_of_range",
            Self::PciIrqPinOutOfRange => "surface_capture_pci_irq_pin_out_of_range",
            Self::PciBar32BitAddressOverflow => "surface_capture_pci_bar_32bit_address_overflow",
            Self::PciMemory64AtLastIndex => "surface_capture_pci_memory64_at_last_index",
            Self::PciBarSlotConflict => "surface_capture_pci_bar_slot_conflict",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedCapture {
    pub capture_id: CaptureId,
    pub part_count: u16,
    pub facts_sha256: [u8; 32],
}

/// Validates exactly one complete capture series. Input order is irrelevant;
/// part indices define the sole canonical order used for digest verification.
pub fn validate_capture(
    records: &[CaptureRecord],
) -> Result<ValidatedCapture, CaptureValidationError> {
    let ordered = order_and_validate_records(records)?;
    let mut counts = [0u16; 4];
    let mut completion = None;

    for record in ordered.iter().flatten() {
        validate_payload(record.payload)?;
        match record.payload {
            CapturePayload::Cpu(_) => counts[0] += 1,
            CapturePayload::SmbiosMemory(_) => counts[1] += 1,
            CapturePayload::LimineMemoryRegion(_) => counts[2] += 1,
            CapturePayload::PciDevice(_) => counts[3] += 1,
            CapturePayload::Completion(value) => {
                if completion.replace(value).is_some() {
                    return Err(CaptureValidationError::MultipleCompletion);
                }
                if record.header.part_index + 1 != record.header.part_count {
                    return Err(CaptureValidationError::CompletionNotLast);
                }
            }
        }
    }

    let missing = [
        CaptureValidationError::MissingCpu,
        CaptureValidationError::MissingSmbiosMemory,
        CaptureValidationError::MissingLimineMemoryRegion,
        CaptureValidationError::MissingPciDevice,
    ];
    for index in 0..counts.len() {
        if counts[index] == 0 {
            return Err(missing[index]);
        }
    }
    let completion = completion.ok_or(CaptureValidationError::MissingCompletion)?;
    if counts
        != [
            completion.cpu_parts,
            completion.smbios_memory_parts,
            completion.limine_memory_region_parts,
            completion.pci_device_parts,
        ]
    {
        return Err(CaptureValidationError::CompletionCountMismatch);
    }
    let digest = digest_ordered(&ordered)?;
    if digest != completion.facts_sha256 {
        return Err(CaptureValidationError::DigestMismatch);
    }
    Ok(ValidatedCapture {
        capture_id: records[0].header.capture_id,
        part_count: records.len() as u16,
        facts_sha256: digest,
    })
}

/// Computes the canonical facts digest for a structurally valid series. The
/// completion payload is excluded so callers can fill its digest field.
pub fn canonical_facts_sha256(
    records: &[CaptureRecord],
) -> Result<[u8; 32], CaptureValidationError> {
    let ordered = order_and_validate_records(records)?;
    for record in ordered.iter().flatten() {
        validate_payload(record.payload)?;
    }
    digest_ordered(&ordered)
}

fn order_and_validate_records<'a>(
    records: &'a [CaptureRecord],
) -> Result<[Option<&'a CaptureRecord>; SURFACE_FACT_CAPTURE_MAX_PARTS], CaptureValidationError> {
    if records.is_empty() {
        return Err(CaptureValidationError::EmptySeries);
    }
    if records.len() > SURFACE_FACT_CAPTURE_MAX_PARTS {
        return Err(CaptureValidationError::TooManyRecords);
    }
    let expected_id = records[0].header.capture_id;
    let expected_count = records[0].header.part_count;
    if expected_count == 0 || expected_count as usize > SURFACE_FACT_CAPTURE_MAX_PARTS {
        return Err(CaptureValidationError::InvalidPartCount);
    }
    let mut ordered = [None; SURFACE_FACT_CAPTURE_MAX_PARTS];
    for record in records {
        let header = record.header;
        if header.schema_version != SURFACE_FACT_CAPTURE_SCHEMA_VERSION {
            return Err(CaptureValidationError::SchemaVersionMismatch);
        }
        if header.capture_id != expected_id {
            return Err(CaptureValidationError::MixedCaptureId);
        }
        if header.part_count != expected_count {
            return Err(CaptureValidationError::PartCountMismatch);
        }
        let index = header.part_index as usize;
        if index >= expected_count as usize {
            return Err(CaptureValidationError::PartIndexOutOfRange);
        }
        if header.part_kind != record.payload.kind() {
            return Err(CaptureValidationError::KindPayloadMismatch);
        }
        if ordered[index].replace(record).is_some() {
            return Err(CaptureValidationError::DuplicatePart);
        }
    }
    if records.len() != expected_count as usize
        || ordered[..expected_count as usize]
            .iter()
            .any(Option::is_none)
    {
        return Err(CaptureValidationError::MissingPart);
    }
    Ok(ordered)
}

pub(crate) fn validate_payload(payload: CapturePayload) -> Result<(), CaptureValidationError> {
    match payload {
        CapturePayload::SmbiosMemory(value) => {
            value.device_locator.as_bytes()?;
            if value.size_bytes == 0 {
                return Err(CaptureValidationError::ZeroSizedSmbiosMemory);
            }
        }
        CapturePayload::LimineMemoryRegion(value) => {
            if value.length == 0 {
                return Err(CaptureValidationError::ZeroLengthMemoryRegion);
            }
            value
                .base
                .checked_add(value.length)
                .ok_or(CaptureValidationError::MemoryRegionOverflow)?;
        }
        CapturePayload::PciDevice(value) => {
            if value.device > 31 {
                return Err(CaptureValidationError::PciDeviceOutOfRange);
            }
            if value.function > 7 {
                return Err(CaptureValidationError::PciFunctionOutOfRange);
            }
            if value.irq_pin > 4 {
                return Err(CaptureValidationError::PciIrqPinOutOfRange);
            }
            let bar_count = value.bar_count as usize;
            if bar_count > SURFACE_FACT_CAPTURE_MAX_PCI_BARS {
                return Err(CaptureValidationError::TooManyPciBars);
            }
            let mut seen = [false; SURFACE_FACT_CAPTURE_MAX_PCI_BARS];
            let mut reserved = [false; SURFACE_FACT_CAPTURE_MAX_PCI_BARS];
            for bar in &value.bars[..bar_count] {
                let index = bar.index as usize;
                if index >= SURFACE_FACT_CAPTURE_MAX_PCI_BARS {
                    return Err(CaptureValidationError::PciBarIndexOutOfRange);
                }
                if seen[index] {
                    return Err(CaptureValidationError::DuplicatePciBarIndex);
                }
                if reserved[index] {
                    return Err(CaptureValidationError::PciBarSlotConflict);
                }
                if bar.kind == PciBarKind::Memory64 {
                    let next = index + 1;
                    if next >= SURFACE_FACT_CAPTURE_MAX_PCI_BARS {
                        return Err(CaptureValidationError::PciMemory64AtLastIndex);
                    }
                    if seen[next] || reserved[next] {
                        return Err(CaptureValidationError::PciBarSlotConflict);
                    }
                    reserved[next] = true;
                }
                seen[index] = true;
                if bar.length == 0 {
                    return Err(CaptureValidationError::ZeroLengthPciBar);
                }
                let end = bar.base
                    .checked_add(bar.length)
                    .ok_or(CaptureValidationError::PciBarAddressOverflow)?;
                if matches!(bar.kind, PciBarKind::Io | PciBarKind::Memory32)
                    && end > (1u64 << 32)
                {
                    return Err(CaptureValidationError::PciBar32BitAddressOverflow);
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn digest_ordered(
    ordered: &[Option<&CaptureRecord>; SURFACE_FACT_CAPTURE_MAX_PARTS],
) -> Result<[u8; 32], CaptureValidationError> {
    let mut hash = Sha256::new();
    hash.update(b"raios.surface_fact_capture.facts.v1\0");
    for record in ordered.iter().flatten() {
        if matches!(record.payload, CapturePayload::Completion(_)) {
            continue;
        }
        hash.update(record.header.schema_version.to_le_bytes());
        hash.update(record.header.capture_id);
        hash.update([record.header.part_kind as u8]);
        hash.update(record.header.part_index.to_le_bytes());
        hash.update(record.header.part_count.to_le_bytes());
        match record.payload {
            CapturePayload::Cpu(value) => {
                for word in [
                    value.leaf,
                    value.subleaf,
                    value.eax,
                    value.ebx,
                    value.ecx,
                    value.edx,
                ] {
                    hash.update(word.to_le_bytes());
                }
            }
            CapturePayload::SmbiosMemory(value) => {
                hash.update(value.array_handle.to_le_bytes());
                hash.update(value.device_handle.to_le_bytes());
                hash.update(value.size_bytes.to_le_bytes());
                hash.update(value.speed_mt_s.to_le_bytes());
                hash.update([value.memory_type, value.form_factor]);
                let text = value.device_locator.as_bytes()?;
                hash.update([text.len() as u8]);
                hash.update(text);
            }
            CapturePayload::LimineMemoryRegion(value) => {
                hash.update(value.base.to_le_bytes());
                hash.update(value.length.to_le_bytes());
                hash.update(value.region_type.to_le_bytes());
            }
            CapturePayload::PciDevice(value) => {
                hash.update(value.segment.to_le_bytes());
                hash.update([value.bus, value.device, value.function]);
                hash.update(value.vendor_id.to_le_bytes());
                hash.update(value.device_id.to_le_bytes());
                hash.update([value.class_code, value.subclass, value.prog_if, value.revision]);
                hash.update([value.irq_line, value.irq_pin, value.bar_count]);
                for bar in &value.bars[..value.bar_count as usize] {
                    hash.update([bar.index, bar.kind as u8]);
                    hash.update(bar.base.to_le_bytes());
                    hash.update(bar.length.to_le_bytes());
                    hash.update([bar.status as u8]);
                }
            }
            CapturePayload::Completion(_) => unreachable!(),
        }
    }
    let digest = hash.finalize();
    let mut out = [0; 32];
    out.copy_from_slice(&digest);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    const ID: CaptureId = [0x38; 16];

    fn header(kind: CapturePartKind, index: u16, count: u16) -> CaptureHeader {
        CaptureHeader {
            schema_version: SURFACE_FACT_CAPTURE_SCHEMA_VERSION, capture_id: ID,
            part_kind: kind, part_index: index, part_count: count,
        }
    }

    fn sample() -> Vec<CaptureRecord> {
        let count = 5;
        let mut records = vec![
            CaptureRecord {
                header: header(CapturePartKind::Cpu, 0, count),
                payload: CapturePayload::Cpu(CpuFact {
                    leaf: 1, subleaf: 0, eax: 1, ebx: 2, ecx: 3, edx: 4,
                }),
            },
            CaptureRecord {
                header: header(CapturePartKind::SmbiosMemory, 1, count),
                payload: CapturePayload::SmbiosMemory(SmbiosMemoryFact {
                    array_handle: 1, device_handle: 2, size_bytes: 8 << 30,
                    speed_mt_s: 1600, memory_type: 24, form_factor: 9,
                    device_locator: BoundedText::new("DIMM 0").unwrap(),
                }),
            },
            CaptureRecord {
                header: header(CapturePartKind::LimineMemoryRegion, 2, count),
                payload: CapturePayload::LimineMemoryRegion(LimineMemoryRegionFact {
                    base: 0x100000, length: 0x400000, region_type: 0,
                }),
            },
            CaptureRecord {
                header: header(CapturePartKind::PciDevice, 3, count),
                payload: CapturePayload::PciDevice(PciDeviceFact {
                    segment: 0, bus: 1, device: 0, function: 0,
                    vendor_id: 0x11ab, device_id: 0x2b38,
                    class_code: 2, subclass: 0x80, prog_if: 0, revision: 0,
                    irq_line: 11, irq_pin: 1, bar_count: 2,
                    bars: [
                        PciBarFact {
                            index: 0, kind: PciBarKind::Memory64,
                            base: 0xf100_0000, length: 0x1_0000,
                            status: PciBarStatus::Assigned,
                        },
                        PciBarFact {
                            index: 2, kind: PciBarKind::Io,
                            base: 0x2000, length: 0x100,
                            status: PciBarStatus::Assigned,
                        },
                        PciBarFact::EMPTY,
                        PciBarFact::EMPTY,
                        PciBarFact::EMPTY,
                        PciBarFact::EMPTY,
                    ],
                }),
            },
            CaptureRecord {
                header: header(CapturePartKind::Completion, 4, count),
                payload: CapturePayload::Completion(CaptureCompletion {
                    cpu_parts: 1, smbios_memory_parts: 1,
                    limine_memory_region_parts: 1, pci_device_parts: 1,
                    facts_sha256: [0; 32],
                }),
            },
        ];
        let digest = canonical_facts_sha256(&records).unwrap();
        if let CapturePayload::Completion(ref mut completion) = records[4].payload {
            completion.facts_sha256 = digest;
        }
        records
    }

    fn assert_pci_rejected(
        error: CaptureValidationError,
        change: impl FnOnce(&mut PciDeviceFact),
    ) {
        let mut records = sample();
        let CapturePayload::PciDevice(ref mut fact) = records[3].payload else {
            unreachable!()
        };
        change(fact);
        assert_eq!(validate_capture(&records), Err(error));
    }

    #[test]
    fn surface_fact_capture_accepts_one_complete_numbered_series() {
        let mut records = sample();
        records.swap(0, 3);
        let validated = validate_capture(&records).unwrap();
        assert_eq!(validated.capture_id, ID);
        assert_eq!(validated.part_count, 5);
        assert_ne!(validated.facts_sha256, [0; 32]);
        assert_eq!(CaptureValidationError::DigestMismatch as u16, 19);
        assert_eq!(CaptureValidationError::DigestMismatch.code(), "surface_capture_digest_mismatch");
    }

    #[test]
    fn surface_fact_capture_rejects_missing_duplicate_mixed_and_wrong_numbering() {
        let records = sample();
        assert_eq!(validate_capture(&records[..4]), Err(CaptureValidationError::MissingPart));

        let mut changed = records.clone();
        changed[1].header.part_index = 0;
        assert_eq!(
            validate_capture(&changed),
            Err(CaptureValidationError::DuplicatePart)
        );

        let mut changed = records.clone();
        changed[2].header.capture_id = [7; 16];
        assert_eq!(
            validate_capture(&changed),
            Err(CaptureValidationError::MixedCaptureId)
        );

        let mut changed = records.clone();
        changed[2].header.part_count = 6;
        assert_eq!(
            validate_capture(&changed),
            Err(CaptureValidationError::PartCountMismatch)
        );

        let mut changed = records.clone();
        changed[2].header.part_index = 5;
        assert_eq!(
            validate_capture(&changed),
            Err(CaptureValidationError::PartIndexOutOfRange)
        );
    }

    #[test]
    fn surface_fact_capture_rejects_oversized_collection_and_string() {
        let record = sample()[0];
        let oversized = vec![record; SURFACE_FACT_CAPTURE_MAX_PARTS + 1];
        assert_eq!(
            validate_capture(&oversized),
            Err(CaptureValidationError::TooManyRecords)
        );

        assert_eq!(
            BoundedText::new("123456789012345678901234567890123"),
            Err(CaptureValidationError::StringTooLong)
        );
        let mut changed = sample();
        let invalid_text = BoundedText::from_raw([b'x'; SURFACE_FACT_CAPTURE_MAX_TEXT_LEN], 33);
        if let CapturePayload::SmbiosMemory(ref mut fact) = changed[1].payload {
            fact.device_locator = invalid_text;
        }
        assert_eq!(
            validate_capture(&changed),
            Err(CaptureValidationError::StringTooLong)
        );
    }

    #[test]
    fn surface_fact_capture_rejects_invalid_memory_ranges() {
        let mut changed = sample();
        if let CapturePayload::LimineMemoryRegion(ref mut fact) = changed[2].payload {
            fact.length = 0;
        }
        assert_eq!(
            validate_capture(&changed),
            Err(CaptureValidationError::ZeroLengthMemoryRegion)
        );

        let mut changed = sample();
        if let CapturePayload::LimineMemoryRegion(ref mut fact) = changed[2].payload {
            fact.base = u64::MAX;
            fact.length = 1;
        }
        assert_eq!(
            validate_capture(&changed),
            Err(CaptureValidationError::MemoryRegionOverflow)
        );
    }

    #[test]
    fn surface_fact_capture_rejects_invalid_pci_bars() {
        assert_pci_rejected(CaptureValidationError::TooManyPciBars, |f| f.bar_count = 7);
        assert_pci_rejected(CaptureValidationError::PciBarIndexOutOfRange, |f| {
            f.bars[0].index = 6
        });
        assert_pci_rejected(CaptureValidationError::ZeroLengthPciBar, |f| {
            f.bars[0].length = 0
        });
        assert_pci_rejected(CaptureValidationError::PciBarAddressOverflow, |f| {
            f.bars[0].base = u64::MAX;
            f.bars[0].length = 1;
        });
        assert_pci_rejected(CaptureValidationError::DuplicatePciBarIndex, |f| {
            f.bars[1].index = f.bars[0].index
        });
    }

    #[test]
    fn surface_fact_capture_rejects_invalid_pci_coordinates_and_slots() {
        assert_pci_rejected(CaptureValidationError::PciDeviceOutOfRange, |f| f.device = 32);
        assert_pci_rejected(CaptureValidationError::PciFunctionOutOfRange, |f| f.function = 8);
        assert_pci_rejected(CaptureValidationError::PciIrqPinOutOfRange, |f| f.irq_pin = 5);
        assert_pci_rejected(CaptureValidationError::PciBar32BitAddressOverflow, |f| {
            f.bars[1].base = u32::MAX as u64;
            f.bars[1].length = 2;
        });
        assert_pci_rejected(CaptureValidationError::PciBar32BitAddressOverflow, |f| {
            f.bars[0].kind = PciBarKind::Memory32;
            f.bars[0].base = 1u64 << 32;
        });
        assert_pci_rejected(CaptureValidationError::PciMemory64AtLastIndex, |f| {
            f.bars[0].index = 5
        });
        assert_pci_rejected(CaptureValidationError::PciBarSlotConflict, |f| {
            f.bars[1].index = 1
        });
        assert_eq!(CaptureValidationError::PciBarSlotConflict as u16, 35);
        assert_eq!(
            CaptureValidationError::PciBarSlotConflict.code(),
            "surface_capture_pci_bar_slot_conflict"
        );
    }

    #[test]
    fn surface_fact_capture_rejects_missing_or_inconsistent_completion() {
        let mut changed = sample();
        changed[4].header.part_kind = CapturePartKind::Cpu;
        changed[4].payload = CapturePayload::Cpu(CpuFact {
            leaf: 7, subleaf: 0, eax: 0, ebx: 0, ecx: 0, edx: 0,
        });
        assert_eq!(validate_capture(&changed), Err(CaptureValidationError::MissingCompletion));

        let mut changed = sample();
        if let CapturePayload::Completion(ref mut completion) = changed[4].payload {
            completion.pci_device_parts = 2;
        }
        assert_eq!(
            validate_capture(&changed),
            Err(CaptureValidationError::CompletionCountMismatch)
        );

        let mut changed = sample();
        if let CapturePayload::Completion(ref mut completion) = changed[4].payload {
            completion.facts_sha256[0] ^= 1;
        }
        assert_eq!(
            validate_capture(&changed),
            Err(CaptureValidationError::DigestMismatch)
        );
    }
}
