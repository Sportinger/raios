//! Allocation-free Surface Fact Wire V1 record codec.
//!
//! One encoded value is exactly one inner RECLOG payload. This module does not
//! parse or produce RECLOG framing and makes no attestation or readiness claim.

use crate::surface_fact_capture::{
    validate_payload, BoundedText, CaptureCompletion, CaptureHeader, CapturePartKind,
    CapturePayload, CaptureRecord, CpuFact, LimineMemoryRegionFact, PciBarFact, PciBarKind,
    PciBarStatus, PciDeviceFact, SmbiosMemoryFact, SURFACE_FACT_CAPTURE_MAX_PARTS,
    SURFACE_FACT_CAPTURE_MAX_PCI_BARS, SURFACE_FACT_CAPTURE_MAX_TEXT_LEN,
    SURFACE_FACT_CAPTURE_SCHEMA_VERSION,
};

pub const SURFACE_FACT_WIRE_MAGIC: [u8; 8] = *b"RAIOSSF0";
pub const SURFACE_FACT_WIRE_VERSION: u16 = 1;
pub const SURFACE_FACT_WIRE_HEADER_LEN: usize = 40;
pub const SURFACE_FACT_WIRE_MAX_RECORD_LEN: usize = 170;

const CPU_PAYLOAD_LEN: usize = 24;
const SMBIOS_FIXED_PAYLOAD_LEN: usize = 19;
const LIMINE_PAYLOAD_LEN: usize = 20;
const PCI_FIXED_PAYLOAD_LEN: usize = 16;
const PCI_BAR_LEN: usize = 19;
const COMPLETION_PAYLOAD_LEN: usize = 40;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum SurfaceFactWireError {
    OutputTooSmall = 1,
    InvalidMagic = 2,
    UnsupportedWireVersion = 3,
    InvalidHeaderLength = 4,
    InvalidRecordLength = 5,
    InvalidPayloadLength = 6,
    SchemaVersionMismatch = 7,
    InvalidFlags = 8,
    UnknownPartKind = 9,
    InvalidPartCount = 10,
    PartIndexOutOfRange = 11,
    KindPayloadMismatch = 12,
    Truncated = 13,
    TrailingBytes = 14,
    InvalidLocator = 15,
    InvalidBarCount = 16,
    UnknownBarKind = 17,
    UnknownBarStatus = 18,
    NonCanonicalBarOrder = 19,
    InvalidModel = 20,
    LengthOverflow = 21,
}

impl SurfaceFactWireError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::OutputTooSmall => "surface_fact_wire_output_too_small",
            Self::InvalidMagic => "surface_fact_wire_invalid_magic",
            Self::UnsupportedWireVersion => "surface_fact_wire_unsupported_version",
            Self::InvalidHeaderLength => "surface_fact_wire_invalid_header_length",
            Self::InvalidRecordLength => "surface_fact_wire_invalid_record_length",
            Self::InvalidPayloadLength => "surface_fact_wire_invalid_payload_length",
            Self::SchemaVersionMismatch => "surface_fact_wire_schema_version_mismatch",
            Self::InvalidFlags => "surface_fact_wire_invalid_flags",
            Self::UnknownPartKind => "surface_fact_wire_unknown_part_kind",
            Self::InvalidPartCount => "surface_fact_wire_invalid_part_count",
            Self::PartIndexOutOfRange => "surface_fact_wire_part_index_out_of_range",
            Self::KindPayloadMismatch => "surface_fact_wire_kind_payload_mismatch",
            Self::Truncated => "surface_fact_wire_truncated",
            Self::TrailingBytes => "surface_fact_wire_trailing_bytes",
            Self::InvalidLocator => "surface_fact_wire_invalid_locator",
            Self::InvalidBarCount => "surface_fact_wire_invalid_bar_count",
            Self::UnknownBarKind => "surface_fact_wire_unknown_bar_kind",
            Self::UnknownBarStatus => "surface_fact_wire_unknown_bar_status",
            Self::NonCanonicalBarOrder => "surface_fact_wire_noncanonical_bar_order",
            Self::InvalidModel => "surface_fact_wire_invalid_model",
            Self::LengthOverflow => "surface_fact_wire_length_overflow",
        }
    }
}

pub fn encoded_len(record: &CaptureRecord) -> Result<usize, SurfaceFactWireError> {
    validate_record(record)?;
    SURFACE_FACT_WIRE_HEADER_LEN
        .checked_add(payload_len(record)?)
        .ok_or(SurfaceFactWireError::LengthOverflow)
}

/// Encodes one record and returns the number of bytes written. If `output` is
/// too small it is left unchanged.
pub fn encode(
    record: &CaptureRecord,
    output: &mut [u8],
) -> Result<usize, SurfaceFactWireError> {
    let payload_len = payload_len(record)?;
    validate_record(record)?;
    let record_len = SURFACE_FACT_WIRE_HEADER_LEN
        .checked_add(payload_len)
        .ok_or(SurfaceFactWireError::LengthOverflow)?;
    if output.len() < record_len {
        return Err(SurfaceFactWireError::OutputTooSmall);
    }
    let record_len_u16 = u16::try_from(record_len).map_err(|_| SurfaceFactWireError::LengthOverflow)?;
    let payload_len_u16 = u16::try_from(payload_len).map_err(|_| SurfaceFactWireError::LengthOverflow)?;
    let mut writer = Writer::new(output);
    writer.bytes(&SURFACE_FACT_WIRE_MAGIC)?;
    writer.u16(SURFACE_FACT_WIRE_VERSION)?;
    writer.u16(SURFACE_FACT_WIRE_HEADER_LEN as u16)?;
    writer.u16(record_len_u16)?;
    writer.u16(payload_len_u16)?;
    writer.u16(record.header.schema_version)?;
    writer.u8(record.header.part_kind as u8)?;
    writer.u8(0)?;
    writer.bytes(&record.header.capture_id)?;
    writer.u16(record.header.part_index)?;
    writer.u16(record.header.part_count)?;
    encode_payload(record.payload, &mut writer)?;
    Ok(record_len)
}

pub fn decode(input: &[u8]) -> Result<CaptureRecord, SurfaceFactWireError> {
    let mut header = Reader::new(input);
    if header.bytes(8)? != SURFACE_FACT_WIRE_MAGIC {
        return Err(SurfaceFactWireError::InvalidMagic);
    }
    if header.u16()? != SURFACE_FACT_WIRE_VERSION {
        return Err(SurfaceFactWireError::UnsupportedWireVersion);
    }
    if header.u16()? as usize != SURFACE_FACT_WIRE_HEADER_LEN {
        return Err(SurfaceFactWireError::InvalidHeaderLength);
    }
    let record_len = header.u16()? as usize;
    let payload_len = header.u16()? as usize;
    let expected = SURFACE_FACT_WIRE_HEADER_LEN
        .checked_add(payload_len)
        .ok_or(SurfaceFactWireError::LengthOverflow)?;
    if record_len != expected || record_len > SURFACE_FACT_WIRE_MAX_RECORD_LEN {
        return Err(SurfaceFactWireError::InvalidRecordLength);
    }
    if record_len > input.len() {
        return Err(SurfaceFactWireError::Truncated);
    }
    if record_len < input.len() {
        return Err(SurfaceFactWireError::TrailingBytes);
    }
    let schema_version = header.u16()?;
    if schema_version != SURFACE_FACT_CAPTURE_SCHEMA_VERSION {
        return Err(SurfaceFactWireError::SchemaVersionMismatch);
    }
    let part_kind = parse_part_kind(header.u8()?)?;
    if header.u8()? != 0 {
        return Err(SurfaceFactWireError::InvalidFlags);
    }
    let mut capture_id = [0; 16];
    capture_id.copy_from_slice(header.bytes(16)?);
    let part_index = header.u16()?;
    let part_count = header.u16()?;
    validate_coordinates(part_index, part_count)?;
    let payload_bytes = header.bytes(payload_len)?;
    header.finish()?;
    let payload = decode_payload(part_kind, payload_bytes)?;
    let record = CaptureRecord {
        header: CaptureHeader {
            schema_version,
            capture_id,
            part_kind,
            part_index,
            part_count,
        },
        payload,
    };
    validate_record(&record)?;
    Ok(record)
}

fn payload_len(record: &CaptureRecord) -> Result<usize, SurfaceFactWireError> {
    match record.payload {
        CapturePayload::Cpu(_) => Ok(CPU_PAYLOAD_LEN),
        CapturePayload::SmbiosMemory(value) => SMBIOS_FIXED_PAYLOAD_LEN
            .checked_add(
                value
                    .device_locator
                    .as_bytes()
                    .map_err(|_| SurfaceFactWireError::InvalidLocator)?
                    .len(),
            )
            .ok_or(SurfaceFactWireError::LengthOverflow),
        CapturePayload::LimineMemoryRegion(_) => Ok(LIMINE_PAYLOAD_LEN),
        CapturePayload::PciDevice(value) => PCI_BAR_LEN
            .checked_mul(value.bar_count as usize)
            .and_then(|bars| PCI_FIXED_PAYLOAD_LEN.checked_add(bars))
            .ok_or(SurfaceFactWireError::LengthOverflow),
        CapturePayload::Completion(_) => Ok(COMPLETION_PAYLOAD_LEN),
    }
}

fn validate_coordinates(index: u16, count: u16) -> Result<(), SurfaceFactWireError> {
    if count == 0 || count as usize > SURFACE_FACT_CAPTURE_MAX_PARTS {
        return Err(SurfaceFactWireError::InvalidPartCount);
    }
    if index >= count {
        return Err(SurfaceFactWireError::PartIndexOutOfRange);
    }
    Ok(())
}

fn validate_record(record: &CaptureRecord) -> Result<(), SurfaceFactWireError> {
    if record.header.schema_version != SURFACE_FACT_CAPTURE_SCHEMA_VERSION {
        return Err(SurfaceFactWireError::SchemaVersionMismatch);
    }
    validate_coordinates(record.header.part_index, record.header.part_count)?;
    if record.header.part_kind != record.payload.kind() {
        return Err(SurfaceFactWireError::KindPayloadMismatch);
    }
    validate_payload(record.payload).map_err(|_| SurfaceFactWireError::InvalidModel)?;
    if let CapturePayload::PciDevice(value) = record.payload {
        let count = value.bar_count as usize;
        if count > SURFACE_FACT_CAPTURE_MAX_PCI_BARS {
            return Err(SurfaceFactWireError::InvalidBarCount);
        }
        for pair in value.bars[..count].windows(2) {
            if pair[0].index >= pair[1].index {
                return Err(SurfaceFactWireError::NonCanonicalBarOrder);
            }
        }
    }
    Ok(())
}

fn encode_payload(payload: CapturePayload, out: &mut Writer<'_>) -> Result<(), SurfaceFactWireError> {
    match payload {
        CapturePayload::Cpu(value) => {
            for word in [value.leaf, value.subleaf, value.eax, value.ebx, value.ecx, value.edx] {
                out.u32(word)?;
            }
        }
        CapturePayload::SmbiosMemory(value) => {
            out.u16(value.array_handle)?;
            out.u16(value.device_handle)?;
            out.u64(value.size_bytes)?;
            out.u32(value.speed_mt_s)?;
            out.u8(value.memory_type)?;
            out.u8(value.form_factor)?;
            let locator = value.device_locator.as_bytes().map_err(|_| SurfaceFactWireError::InvalidLocator)?;
            out.u8(locator.len() as u8)?;
            out.bytes(locator)?;
        }
        CapturePayload::LimineMemoryRegion(value) => {
            out.u64(value.base)?;
            out.u64(value.length)?;
            out.u32(value.region_type)?;
        }
        CapturePayload::PciDevice(value) => {
            out.u16(value.segment)?;
            for byte in [value.bus, value.device, value.function] { out.u8(byte)?; }
            out.u16(value.vendor_id)?;
            out.u16(value.device_id)?;
            for byte in [value.class_code, value.subclass, value.prog_if, value.revision,
                         value.irq_line, value.irq_pin, value.bar_count] { out.u8(byte)?; }
            for bar in &value.bars[..value.bar_count as usize] {
                out.u8(bar.index)?;
                out.u8(bar.kind as u8)?;
                out.u64(bar.base)?;
                out.u64(bar.length)?;
                out.u8(bar.status as u8)?;
            }
        }
        CapturePayload::Completion(value) => {
            for count in [value.cpu_parts, value.smbios_memory_parts,
                          value.limine_memory_region_parts, value.pci_device_parts] {
                out.u16(count)?;
            }
            out.bytes(&value.facts_sha256)?;
        }
    }
    Ok(())
}

fn decode_payload(kind: CapturePartKind, bytes: &[u8]) -> Result<CapturePayload, SurfaceFactWireError> {
    let mut input = Reader::new(bytes);
    let payload = match kind {
        CapturePartKind::Cpu => {
            require_len(bytes, CPU_PAYLOAD_LEN)?;
            CapturePayload::Cpu(CpuFact { leaf: input.u32()?, subleaf: input.u32()?,
                eax: input.u32()?, ebx: input.u32()?, ecx: input.u32()?, edx: input.u32()? })
        }
        CapturePartKind::SmbiosMemory => {
            if bytes.len() < SMBIOS_FIXED_PAYLOAD_LEN { return Err(SurfaceFactWireError::InvalidPayloadLength); }
            let array_handle = input.u16()?;
            let device_handle = input.u16()?;
            let size_bytes = input.u64()?;
            let speed_mt_s = input.u32()?;
            let memory_type = input.u8()?;
            let form_factor = input.u8()?;
            let locator_len = input.u8()? as usize;
            if locator_len > SURFACE_FACT_CAPTURE_MAX_TEXT_LEN || locator_len != input.remaining() {
                return Err(SurfaceFactWireError::InvalidLocator);
            }
            let mut raw = [0; SURFACE_FACT_CAPTURE_MAX_TEXT_LEN];
            raw[..locator_len].copy_from_slice(input.bytes(locator_len)?);
            CapturePayload::SmbiosMemory(SmbiosMemoryFact { array_handle, device_handle,
                size_bytes, speed_mt_s, memory_type, form_factor,
                device_locator: BoundedText::from_raw(raw, locator_len as u8) })
        }
        CapturePartKind::LimineMemoryRegion => {
            require_len(bytes, LIMINE_PAYLOAD_LEN)?;
            CapturePayload::LimineMemoryRegion(LimineMemoryRegionFact {
                base: input.u64()?, length: input.u64()?, region_type: input.u32()? })
        }
        CapturePartKind::PciDevice => {
            if bytes.len() < PCI_FIXED_PAYLOAD_LEN { return Err(SurfaceFactWireError::InvalidPayloadLength); }
            let segment = input.u16()?;
            let bus = input.u8()?;
            let device = input.u8()?;
            let function = input.u8()?;
            let vendor_id = input.u16()?;
            let device_id = input.u16()?;
            let class_code = input.u8()?;
            let subclass = input.u8()?;
            let prog_if = input.u8()?;
            let revision = input.u8()?;
            let irq_line = input.u8()?;
            let irq_pin = input.u8()?;
            let bar_count = input.u8()? as usize;
            if bar_count > SURFACE_FACT_CAPTURE_MAX_PCI_BARS {
                return Err(SurfaceFactWireError::InvalidBarCount);
            }
            let expected = PCI_BAR_LEN.checked_mul(bar_count)
                .and_then(|bars| PCI_FIXED_PAYLOAD_LEN.checked_add(bars))
                .ok_or(SurfaceFactWireError::LengthOverflow)?;
            require_len(bytes, expected)?;
            let mut bars = [PciBarFact::EMPTY; SURFACE_FACT_CAPTURE_MAX_PCI_BARS];
            for slot in &mut bars[..bar_count] {
                slot.index = input.u8()?;
                slot.kind = parse_bar_kind(input.u8()?)?;
                slot.base = input.u64()?;
                slot.length = input.u64()?;
                slot.status = parse_bar_status(input.u8()?)?;
            }
            CapturePayload::PciDevice(PciDeviceFact { segment, bus, device, function,
                vendor_id, device_id, class_code, subclass, prog_if, revision, irq_line,
                irq_pin, bar_count: bar_count as u8, bars })
        }
        CapturePartKind::Completion => {
            require_len(bytes, COMPLETION_PAYLOAD_LEN)?;
            let cpu_parts = input.u16()?;
            let smbios_memory_parts = input.u16()?;
            let limine_memory_region_parts = input.u16()?;
            let pci_device_parts = input.u16()?;
            let mut facts_sha256 = [0; 32];
            facts_sha256.copy_from_slice(input.bytes(32)?);
            CapturePayload::Completion(CaptureCompletion { cpu_parts, smbios_memory_parts,
                limine_memory_region_parts, pci_device_parts, facts_sha256 })
        }
    };
    input.finish()?;
    Ok(payload)
}

fn require_len(bytes: &[u8], expected: usize) -> Result<(), SurfaceFactWireError> {
    if bytes.len() == expected { Ok(()) } else { Err(SurfaceFactWireError::InvalidPayloadLength) }
}

fn parse_part_kind(value: u8) -> Result<CapturePartKind, SurfaceFactWireError> {
    match value { 1 => Ok(CapturePartKind::Cpu), 2 => Ok(CapturePartKind::SmbiosMemory),
        3 => Ok(CapturePartKind::LimineMemoryRegion), 4 => Ok(CapturePartKind::PciDevice),
        255 => Ok(CapturePartKind::Completion), _ => Err(SurfaceFactWireError::UnknownPartKind) }
}

fn parse_bar_kind(value: u8) -> Result<PciBarKind, SurfaceFactWireError> {
    match value { 1 => Ok(PciBarKind::Io), 2 => Ok(PciBarKind::Memory32),
        3 => Ok(PciBarKind::Memory64), _ => Err(SurfaceFactWireError::UnknownBarKind) }
}

fn parse_bar_status(value: u8) -> Result<PciBarStatus, SurfaceFactWireError> {
    match value { 1 => Ok(PciBarStatus::Assigned), 2 => Ok(PciBarStatus::Disabled),
        _ => Err(SurfaceFactWireError::UnknownBarStatus) }
}

struct Reader<'a> { bytes: &'a [u8], offset: usize }

impl<'a> Reader<'a> {
    const fn new(bytes: &'a [u8]) -> Self { Self { bytes, offset: 0 } }
    fn bytes(&mut self, len: usize) -> Result<&'a [u8], SurfaceFactWireError> {
        let end = self.offset.checked_add(len).ok_or(SurfaceFactWireError::LengthOverflow)?;
        let value = self.bytes.get(self.offset..end).ok_or(SurfaceFactWireError::Truncated)?;
        self.offset = end;
        Ok(value)
    }
    fn u8(&mut self) -> Result<u8, SurfaceFactWireError> { Ok(self.bytes(1)?[0]) }
    fn u16(&mut self) -> Result<u16, SurfaceFactWireError> {
        let value = self.bytes(2)?; Ok(u16::from_le_bytes([value[0], value[1]]))
    }
    fn u32(&mut self) -> Result<u32, SurfaceFactWireError> {
        let value = self.bytes(4)?; Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
    }
    fn u64(&mut self) -> Result<u64, SurfaceFactWireError> {
        let value = self.bytes(8)?;
        Ok(u64::from_le_bytes([value[0], value[1], value[2], value[3],
            value[4], value[5], value[6], value[7]]))
    }
    fn remaining(&self) -> usize { self.bytes.len() - self.offset }
    fn finish(&self) -> Result<(), SurfaceFactWireError> {
        if self.offset == self.bytes.len() { Ok(()) } else { Err(SurfaceFactWireError::TrailingBytes) }
    }
}

struct Writer<'a> { bytes: &'a mut [u8], offset: usize }

impl<'a> Writer<'a> {
    fn new(bytes: &'a mut [u8]) -> Self { Self { bytes, offset: 0 } }
    fn bytes(&mut self, value: &[u8]) -> Result<(), SurfaceFactWireError> {
        let end = self.offset.checked_add(value.len()).ok_or(SurfaceFactWireError::LengthOverflow)?;
        let target = self.bytes.get_mut(self.offset..end).ok_or(SurfaceFactWireError::OutputTooSmall)?;
        target.copy_from_slice(value);
        self.offset = end;
        Ok(())
    }
    fn u8(&mut self, value: u8) -> Result<(), SurfaceFactWireError> { self.bytes(&[value]) }
    fn u16(&mut self, value: u16) -> Result<(), SurfaceFactWireError> { self.bytes(&value.to_le_bytes()) }
    fn u32(&mut self, value: u32) -> Result<(), SurfaceFactWireError> { self.bytes(&value.to_le_bytes()) }
    fn u64(&mut self, value: u64) -> Result<(), SurfaceFactWireError> { self.bytes(&value.to_le_bytes()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: [u8; 16] = [0x11; 16];

    fn header(kind: CapturePartKind) -> CaptureHeader {
        CaptureHeader { schema_version: 1, capture_id: ID, part_kind: kind,
            part_index: 1, part_count: 5 }
    }

    fn records() -> [CaptureRecord; 5] {
        [
            CaptureRecord { header: header(CapturePartKind::Cpu), payload: CapturePayload::Cpu(
                CpuFact { leaf: 1, subleaf: 2, eax: 3, ebx: 4, ecx: 5, edx: 6 }) },
            CaptureRecord { header: header(CapturePartKind::SmbiosMemory),
                payload: CapturePayload::SmbiosMemory(SmbiosMemoryFact { array_handle: 1,
                    device_handle: 2, size_bytes: 3, speed_mt_s: 4, memory_type: 5,
                    form_factor: 6, device_locator: BoundedText::new("A").unwrap() }) },
            CaptureRecord { header: header(CapturePartKind::LimineMemoryRegion),
                payload: CapturePayload::LimineMemoryRegion(LimineMemoryRegionFact {
                    base: 1, length: 2, region_type: 3 }) },
            CaptureRecord { header: header(CapturePartKind::PciDevice),
                payload: CapturePayload::PciDevice(PciDeviceFact { segment: 1, bus: 2,
                    device: 3, function: 4, vendor_id: 5, device_id: 6, class_code: 7,
                    subclass: 8, prog_if: 9, revision: 10, irq_line: 11, irq_pin: 1,
                    bar_count: 1, bars: [PciBarFact { index: 0, kind: PciBarKind::Memory32,
                        base: 0x1000, length: 0x100, status: PciBarStatus::Assigned },
                        PciBarFact::EMPTY, PciBarFact::EMPTY, PciBarFact::EMPTY,
                        PciBarFact::EMPTY, PciBarFact::EMPTY] }) },
            CaptureRecord { header: header(CapturePartKind::Completion),
                payload: CapturePayload::Completion(CaptureCompletion { cpu_parts: 1,
                    smbios_memory_parts: 2, limine_memory_region_parts: 3,
                    pci_device_parts: 4, facts_sha256: [0xaa; 32] }) },
        ]
    }

    fn encoded(record: &CaptureRecord) -> std::vec::Vec<u8> {
        let mut bytes = [0xcc; SURFACE_FACT_WIRE_MAX_RECORD_LEN];
        let len = encode(record, &mut bytes).unwrap();
        bytes[..len].to_vec()
    }

    #[test]
    fn surface_fact_wire_golden_vectors_and_roundtrips_all_kinds() {
        const HEADER_TAIL: [u8; 20] = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 5, 0];
        let expected_payloads: [&[u8]; 5] = [
            &[1,0,0,0, 2,0,0,0, 3,0,0,0, 4,0,0,0, 5,0,0,0, 6,0,0,0],
            &[1,0, 2,0, 3,0,0,0,0,0,0,0, 4,0,0,0, 5,6,1, b'A'],
            &[1,0,0,0,0,0,0,0, 2,0,0,0,0,0,0,0, 3,0,0,0],
            &[1,0, 2,3,4, 5,0, 6,0, 7,8,9,10,11,1,1,
              0,2, 0,0x10,0,0,0,0,0,0, 0,1,0,0,0,0,0,0, 1],
            &[1,0, 2,0, 3,0, 4,0,
              0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,
              0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,
              0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,
              0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa,0xaa],
        ];
        for (index, record) in records().iter().enumerate() {
            let bytes = encoded(record);
            let payload = expected_payloads[index];
            let total = 40 + payload.len();
            assert_eq!(&bytes[..8], b"RAIOSSF0");
            assert_eq!(&bytes[8..12], &[1, 0, 40, 0]);
            assert_eq!(&bytes[12..16], &[total as u8, 0, payload.len() as u8, 0]);
            let mut tail = HEADER_TAIL;
            tail[2] = record.header.part_kind as u8;
            assert_eq!(&bytes[16..40], &[&tail[..4], &ID, &tail[16..]].concat());
            assert_eq!(&bytes[40..], payload);
            assert_eq!(decode(&bytes), Ok(*record));
            assert_eq!(encoded_len(record), Ok(total));
        }
        assert_eq!(encoded_len(&records()[3]), Ok(75));
        assert_eq!(SurfaceFactWireError::InvalidModel as u16, 20);
        assert_eq!(SurfaceFactWireError::InvalidModel.code(), "surface_fact_wire_invalid_model");
    }

    #[test]
    fn surface_fact_wire_small_output_is_untouched() {
        let record = records()[0];
        let mut output = [0x5a; 63];
        assert_eq!(encode(&record, &mut output), Err(SurfaceFactWireError::OutputTooSmall));
        assert_eq!(output, [0x5a; 63]);
    }

    fn set_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn surface_fact_wire_rejects_header_and_length_mutations() {
        let good = encoded(&records()[0]);
        let cases: &[(usize, u8, SurfaceFactWireError)] = &[
            (0, b'X', SurfaceFactWireError::InvalidMagic),
            (8, 2, SurfaceFactWireError::UnsupportedWireVersion),
            (10, 39, SurfaceFactWireError::InvalidHeaderLength),
            (16, 2, SurfaceFactWireError::SchemaVersionMismatch),
            (19, 1, SurfaceFactWireError::InvalidFlags),
            (18, 9, SurfaceFactWireError::UnknownPartKind),
        ];
        for &(offset, value, error) in cases {
            let mut changed = good.clone(); changed[offset] = value;
            assert_eq!(decode(&changed), Err(error));
        }
        let mut changed = good.clone(); set_u16(&mut changed, 12, 63);
        assert_eq!(decode(&changed), Err(SurfaceFactWireError::InvalidRecordLength));
        let mut changed = good.clone(); set_u16(&mut changed, 14, 23);
        assert_eq!(decode(&changed), Err(SurfaceFactWireError::InvalidRecordLength));
        let mut changed = good.clone(); set_u16(&mut changed, 38, 0);
        assert_eq!(decode(&changed), Err(SurfaceFactWireError::InvalidPartCount));
        let mut changed = good.clone(); set_u16(&mut changed, 36, 5);
        assert_eq!(decode(&changed), Err(SurfaceFactWireError::PartIndexOutOfRange));
        assert_eq!(decode(&good[..63]), Err(SurfaceFactWireError::Truncated));
        let mut changed = good.clone(); changed.push(0);
        assert_eq!(decode(&changed), Err(SurfaceFactWireError::TrailingBytes));
        let mut changed = good.clone(); changed[18] = CapturePartKind::Completion as u8;
        assert_eq!(decode(&changed), Err(SurfaceFactWireError::InvalidPayloadLength));
    }

    #[test]
    fn surface_fact_wire_rejects_locator_bar_and_model_mutations() {
        let mut locator = encoded(&records()[1]); locator[58] = 2;
        assert_eq!(decode(&locator), Err(SurfaceFactWireError::InvalidLocator));
        let mut locator = encoded(&records()[1]); locator[59] = 0;
        assert_eq!(decode(&locator), Err(SurfaceFactWireError::InvalidModel));

        let pci = encoded(&records()[3]);
        let mut changed = pci.clone(); changed[55] = 7;
        assert_eq!(decode(&changed), Err(SurfaceFactWireError::InvalidBarCount));
        let mut changed = pci.clone(); changed[57] = 9;
        assert_eq!(decode(&changed), Err(SurfaceFactWireError::UnknownBarKind));
        let mut changed = pci.clone(); changed[74] = 9;
        assert_eq!(decode(&changed), Err(SurfaceFactWireError::UnknownBarStatus));
        let mut two = records()[3];
        if let CapturePayload::PciDevice(ref mut fact) = two.payload {
            fact.bar_count = 2; fact.bars[1] = fact.bars[0];
        }
        assert_eq!(encoded_len(&two), Err(SurfaceFactWireError::InvalidModel));
        if let CapturePayload::PciDevice(ref mut fact) = two.payload {
            fact.bars[1].index = 0; fact.bars[0].index = 2;
        }
        assert_eq!(encoded_len(&two), Err(SurfaceFactWireError::NonCanonicalBarOrder));
        let mut ordered = records()[3];
        if let CapturePayload::PciDevice(ref mut fact) = ordered.payload {
            fact.bar_count = 2;
            fact.bars[1] = fact.bars[0];
            fact.bars[1].index = 2;
            fact.bars[1].base = 0x2000;
        }
        let mut changed = encoded(&ordered);
        changed[56] = 2;
        changed[75] = 0;
        assert_eq!(decode(&changed), Err(SurfaceFactWireError::NonCanonicalBarOrder));
        let mut invalid = records()[2];
        if let CapturePayload::LimineMemoryRegion(ref mut fact) = invalid.payload { fact.length = 0; }
        assert_eq!(encoded_len(&invalid), Err(SurfaceFactWireError::InvalidModel));
        let mut wrong = records()[0]; wrong.header.part_kind = CapturePartKind::Completion;
        assert_eq!(encoded_len(&wrong), Err(SurfaceFactWireError::KindPayloadMismatch));
    }
}
