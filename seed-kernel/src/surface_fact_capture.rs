use alloc::vec::Vec;
use core::arch::x86_64::{__cpuid_count, CpuidResult};

use limine::{
    memory_map::EntryType,
    response::{HhdmResponse, MemoryMapResponse, SmbiosResponse},
};
use raios_core::surface_fact_capture::{
    canonical_facts_sha256, validate_capture, BoundedText, CaptureCompletion, CaptureHeader,
    CapturePayload, CaptureRecord, CpuFact, LimineMemoryRegionFact, PciBarFact,
    PciBarKind as FactBarKind, PciBarStatus, PciDeviceFact, SmbiosMemoryFact,
    SURFACE_FACT_CAPTURE_MAX_PARTS, SURFACE_FACT_CAPTURE_SCHEMA_VERSION,
};
use sha2::{Digest, Sha256};

use crate::pci::{self, PciBarKind};

const SMBIOS32_MIN_LEN: usize = 0x1f;
const SMBIOS64_MIN_LEN: usize = 0x18;
const SMBIOS_ENTRY_MAX_LEN: usize = 256;
const SMBIOS_SLICE_MAX_LEN: usize = 1024 * 1024;
const CPUID_LEAF7_MAX_SUBLEAF: u32 = 32;

pub enum CapturePhase {
    Cpuid,
    Smbios,
    SmbiosEntryPoint,
    SmbiosTable,
    SmbiosParse,
    MemoryMap,
    Pci,
    Finalize,
}

/// Measures one bounded boot epoch. Failure returns no partial series, so the
/// caller cannot reach USB persistence with malformed or incomplete facts.
pub fn capture(
    smbios: Option<&SmbiosResponse>,
    memory_map: Option<&MemoryMapResponse>,
    hhdm: Option<&HhdmResponse>,
    mut progress: impl FnMut(CapturePhase),
) -> Result<Vec<CaptureRecord>, &'static str> {
    let nonce = crate::time::rdtsc();
    let smbios = smbios.ok_or("surface_capture_smbios_missing")?;
    let memory_map = memory_map.ok_or("surface_capture_memory_map_missing")?;
    let hhdm = hhdm.ok_or("surface_capture_hhdm_missing")?;
    let mut payloads = Vec::new();

    progress(CapturePhase::Cpuid);
    capture_cpuid(&mut payloads)?;
    progress(CapturePhase::Smbios);
    capture_smbios(smbios, memory_map, hhdm, &mut progress, &mut payloads)?;
    progress(CapturePhase::MemoryMap);
    capture_memory_map(memory_map, &mut payloads)?;

    // This is the sole whole-bus enumeration for this boot capture. BAR sizing
    // completes before USB or any later PCI driver can reuse the functions.
    progress(CapturePhase::Pci);
    let functions = pci::enumerate_functions_for_capture()?;
    for function in functions {
        let mut bars = [PciBarFact::EMPTY; 6];
        if function.bars.len() > bars.len() {
            return Err("surface_capture_pci_bar_count");
        }
        for (slot, bar) in function.bars.iter().enumerate() {
            if slot != 0 && function.bars[slot - 1].index >= bar.index {
                return Err("surface_capture_pci_bar_order");
            }
            bars[slot] = PciBarFact {
                index: bar.index,
                kind: match bar.kind {
                    PciBarKind::Io => FactBarKind::Io,
                    PciBarKind::Memory32 => FactBarKind::Memory32,
                    PciBarKind::Memory64 => FactBarKind::Memory64,
                },
                base: bar.base,
                length: bar.size,
                status: PciBarStatus::Assigned,
            };
        }
        push_bounded(
            &mut payloads,
            CapturePayload::PciDevice(PciDeviceFact {
                segment: 0,
                bus: function.bus,
                device: function.device,
                function: function.function,
                vendor_id: function.vendor_id,
                device_id: function.device_id,
                class_code: function.class,
                subclass: function.subclass,
                prog_if: function.prog_if,
                revision: function.revision,
                irq_line: function.interrupt_line,
                irq_pin: function.interrupt_pin,
                bar_count: function.bars.len() as u8,
                bars,
            }),
        )?;
    }

    progress(CapturePhase::Finalize);
    let capture_id = capture_id(nonce, &payloads);
    let part_count = payloads
        .len()
        .checked_add(1)
        .and_then(|count| u16::try_from(count).ok())
        .ok_or("surface_capture_part_count")?;
    let mut records = Vec::with_capacity(part_count as usize);
    for (index, payload) in payloads.iter().copied().enumerate() {
        records.push(record(capture_id, index as u16, part_count, payload));
    }
    let counts = count_parts(&payloads)?;
    records.push(record(
        capture_id,
        part_count - 1,
        part_count,
        CapturePayload::Completion(CaptureCompletion {
            cpu_parts: counts[0],
            smbios_memory_parts: counts[1],
            limine_memory_region_parts: counts[2],
            pci_device_parts: counts[3],
            facts_sha256: [0; 32],
        }),
    ));
    let digest = canonical_facts_sha256(&records).map_err(|error| error.code())?;
    let last = records.last_mut().ok_or("surface_capture_empty")?;
    if let CapturePayload::Completion(ref mut completion) = last.payload {
        completion.facts_sha256 = digest;
    }
    validate_capture(&records).map_err(|error| error.code())?;
    Ok(records)
}

fn capture_cpuid(payloads: &mut Vec<CapturePayload>) -> Result<(), &'static str> {
    let basic = cpuid(0, 0);
    push_cpu(payloads, 0, 0, basic)?;
    if basic.eax >= 1 {
        push_cpu(payloads, 1, 0, cpuid(1, 0))?;
    }
    if basic.eax >= 7 {
        let leaf7 = cpuid(7, 0);
        if leaf7.eax > CPUID_LEAF7_MAX_SUBLEAF {
            return Err("surface_capture_cpuid_leaf7_subleaf_cap");
        }
        push_cpu(payloads, 7, 0, leaf7)?;
        for subleaf in 1..=leaf7.eax {
            push_cpu(payloads, 7, subleaf, cpuid(7, subleaf))?;
        }
    }
    let extended = cpuid(0x8000_0000, 0);
    push_cpu(payloads, 0x8000_0000, 0, extended)?;
    if extended.eax >= 0x8000_0001 {
        push_cpu(payloads, 0x8000_0001, 0, cpuid(0x8000_0001, 0))?;
    }
    for leaf in 0x8000_0002..=0x8000_0004 {
        if extended.eax >= leaf {
            push_cpu(payloads, leaf, 0, cpuid(leaf, 0))?;
        }
    }
    Ok(())
}

fn push_cpu(
    payloads: &mut Vec<CapturePayload>,
    leaf: u32,
    subleaf: u32,
    value: CpuidResult,
) -> Result<(), &'static str> {
    push_bounded(payloads, CapturePayload::Cpu(CpuFact {
        leaf,
        subleaf,
        eax: value.eax,
        ebx: value.ebx,
        ecx: value.ecx,
        edx: value.edx,
    }))
}

fn capture_memory_map(
    response: &MemoryMapResponse,
    payloads: &mut Vec<CapturePayload>,
) -> Result<(), &'static str> {
    for entry in response.entries() {
        let region_type = if entry.entry_type == EntryType::USABLE { 0 }
        else if entry.entry_type == EntryType::RESERVED { 1 }
        else if entry.entry_type == EntryType::ACPI_RECLAIMABLE { 2 }
        else if entry.entry_type == EntryType::ACPI_NVS { 3 }
        else if entry.entry_type == EntryType::BAD_MEMORY { 4 }
        else if entry.entry_type == EntryType::BOOTLOADER_RECLAIMABLE { 5 }
        else if entry.entry_type == EntryType::EXECUTABLE_AND_MODULES { 6 }
        else if entry.entry_type == EntryType::FRAMEBUFFER { 7 }
        else { return Err("surface_capture_memory_type_unknown"); };
        if entry.length == 0 || entry.base.checked_add(entry.length).is_none() {
            return Err("surface_capture_memory_range_invalid");
        }
        push_bounded(payloads, CapturePayload::LimineMemoryRegion(
            LimineMemoryRegionFact { base: entry.base, length: entry.length, region_type },
        ))?;
    }
    Ok(())
}

fn capture_smbios(
    response: &SmbiosResponse,
    memory_map: &MemoryMapResponse,
    hhdm: &HhdmResponse,
    progress: &mut impl FnMut(CapturePhase),
    payloads: &mut Vec<CapturePayload>,
) -> Result<(), &'static str> {
    let (table_phys, table_len, declared_structure_count) = if let Some(address) = response.entry_64() {
        let pointer = PhysicalSmbiosPointer::new(address.get() as u64, hhdm)?;
        progress(CapturePhase::SmbiosEntryPoint);
        let entry = pointer.slice(SMBIOS64_MIN_LEN, memory_map)?;
        if &entry[..5] != b"_SM3_" { return Err("surface_capture_smbios64_anchor"); }
        let length = entry[6] as usize;
        if !(SMBIOS64_MIN_LEN..=SMBIOS_ENTRY_MAX_LEN).contains(&length) {
            return Err("surface_capture_smbios64_length");
        }
        let entry = pointer.slice(length, memory_map)?;
        if checksum(entry) != 0 { return Err("surface_capture_smbios64_checksum"); }
        (read_u64(entry, 16)?, read_u32(entry, 12)? as usize, None)
    } else if let Some(address) = response.entry_32() {
        let pointer = PhysicalSmbiosPointer::new(address.get() as u64, hhdm)?;
        progress(CapturePhase::SmbiosEntryPoint);
        let entry = pointer.slice(SMBIOS32_MIN_LEN, memory_map)?;
        if &entry[..4] != b"_SM_" { return Err("surface_capture_smbios32_anchor"); }
        let length = entry[5] as usize;
        if !(SMBIOS32_MIN_LEN..=SMBIOS_ENTRY_MAX_LEN).contains(&length) {
            return Err("surface_capture_smbios32_length");
        }
        let entry = pointer.slice(length, memory_map)?;
        if checksum(entry) != 0 || &entry[16..21] != b"_DMI_" || checksum(&entry[16..31]) != 0 {
            return Err("surface_capture_smbios32_checksum");
        }
        (
            read_u32(entry, 24)? as u64,
            read_u16(entry, 22)? as usize,
            Some(read_u16(entry, 28)?),
        )
    } else {
        return Err("surface_capture_smbios_entry_missing");
    };
    if table_len == 0 { return Err("surface_capture_smbios_table_empty"); }
    let table_pointer = PhysicalSmbiosPointer::new(table_phys, hhdm)?;
    progress(CapturePhase::SmbiosTable);
    let table = table_pointer.slice(table_len, memory_map)?;
    progress(CapturePhase::SmbiosParse);
    parse_smbios_table(table, declared_structure_count, payloads)
}

fn parse_smbios_table(
    table: &[u8],
    declared_structure_count: Option<u16>,
    payloads: &mut Vec<CapturePayload>,
) -> Result<(), &'static str> {
    let mut offset = 0usize;
    let mut ended = false;
    let mut structure_count = 0usize;
    let mut type17_handles = Vec::new();
    while offset < table.len() {
        let header_end = offset.checked_add(4).ok_or("surface_capture_smbios_overflow")?;
        if header_end > table.len() { return Err("surface_capture_smbios_header_truncated"); }
        let kind = table[offset];
        let formatted_len = table[offset + 1] as usize;
        if formatted_len < 4 { return Err("surface_capture_smbios_structure_length"); }
        let formatted_end = offset.checked_add(formatted_len).ok_or("surface_capture_smbios_overflow")?;
        if formatted_end > table.len() { return Err("surface_capture_smbios_structure_truncated"); }
        let strings_end = find_double_nul(table, formatted_end)?;
        structure_count = structure_count
            .checked_add(1)
            .ok_or("surface_capture_smbios_structure_count")?;
        if kind == 17 {
            let formatted = &table[offset..formatted_end];
            if formatted.len() < 23 { return Err("surface_capture_smbios_type17_short"); }
            let device_handle = read_u16(formatted, 2)?;
            if type17_handles.iter().any(|handle| *handle == device_handle) {
                return Err("surface_capture_smbios_type17_duplicate_handle");
            }
            type17_handles.push(device_handle);
            let raw_size = read_u16(formatted, 12)?;
            if raw_size != 0 {
                let size_bytes = if raw_size == 0x7fff {
                    if formatted.len() < 32 { return Err("surface_capture_smbios_extended_size_missing"); }
                    (read_u32(formatted, 28)? as u64 & 0x7fff_ffff)
                        .checked_mul(1024 * 1024).ok_or("surface_capture_smbios_size_overflow")?
                } else if raw_size == 0xffff {
                    return Err("surface_capture_smbios_size_unknown");
                } else if raw_size & 0x8000 != 0 {
                    ((raw_size & 0x7fff) as u64).checked_mul(1024).ok_or("surface_capture_smbios_size_overflow")?
                } else {
                    (raw_size as u64).checked_mul(1024 * 1024).ok_or("surface_capture_smbios_size_overflow")?
                };
                let raw_speed = read_u16(formatted, 21)?;
                let speed_mt_s = if raw_speed == 0xffff {
                    if formatted.len() < 88 { return Err("surface_capture_smbios_extended_speed_missing"); }
                    read_u32(formatted, 84)?
                } else { raw_speed as u32 };
                let locator = smbios_string(table, formatted_end, strings_end, formatted[16])?;
                push_bounded(payloads, CapturePayload::SmbiosMemory(SmbiosMemoryFact {
                    array_handle: read_u16(formatted, 4)?,
                    device_handle,
                    size_bytes,
                    speed_mt_s,
                    memory_type: formatted[18],
                    form_factor: formatted[14],
                    device_locator: locator,
                }))?;
            }
        }
        offset = strings_end;
        if kind == 127 { ended = true; break; }
    }
    if !ended { return Err("surface_capture_smbios_end_missing"); }
    if let Some(declared) = declared_structure_count {
        if structure_count != declared as usize {
            return Err("surface_capture_smbios32_structure_count");
        }
        if offset != table.len() {
            return Err("surface_capture_smbios32_type127_not_final");
        }
    }
    Ok(())
}

fn find_double_nul(table: &[u8], start: usize) -> Result<usize, &'static str> {
    let mut cursor = start;
    while cursor.checked_add(1).is_some_and(|end| end < table.len()) {
        if table[cursor] == 0 && table[cursor + 1] == 0 { return Ok(cursor + 2); }
        cursor += 1;
    }
    Err("surface_capture_smbios_double_nul_missing")
}

fn smbios_string(
    table: &[u8],
    start: usize,
    end: usize,
    index: u8,
) -> Result<BoundedText, &'static str> {
    if index == 0 { return Err("surface_capture_smbios_string_index_zero"); }
    let mut current = 1u8;
    let mut cursor = start;
    while cursor + 1 < end {
        let relative = table[cursor..end - 1].iter().position(|byte| *byte == 0)
            .ok_or("surface_capture_smbios_string_terminator")?;
        let next = cursor + relative;
        if current == index {
            if next == cursor { return Err("surface_capture_smbios_string_empty"); }
            let value = core::str::from_utf8(&table[cursor..next])
                .map_err(|_| "surface_capture_smbios_string_encoding")?;
            return BoundedText::new(value).map_err(|error| error.code());
        }
        current = current.checked_add(1).ok_or("surface_capture_smbios_string_index")?;
        cursor = next + 1;
    }
    Err("surface_capture_smbios_string_index")
}

struct PhysicalSmbiosPointer {
    physical: u64,
    virtual_address: u64,
}

impl PhysicalSmbiosPointer {
    fn new(physical: u64, hhdm: &HhdmResponse) -> Result<Self, &'static str> {
        if physical == 0 { return Err("surface_capture_physical_zero_range"); }
        let virtual_address = physical
            .checked_add(hhdm.offset())
            .ok_or("surface_capture_hhdm_overflow")?;
        if !is_canonical_x86_64(virtual_address) {
            return Err("surface_capture_hhdm_noncanonical");
        }
        Ok(Self { physical, virtual_address })
    }

    fn slice<'a>(
        &'a self,
        length: usize,
        memory_map: &MemoryMapResponse,
    ) -> Result<&'a [u8], &'static str> {
        if length == 0 { return Err("surface_capture_physical_zero_range"); }
        if length > SMBIOS_SLICE_MAX_LEN || length > isize::MAX as usize {
            return Err("surface_capture_physical_length");
        }
        let length64 = u64::try_from(length).map_err(|_| "surface_capture_physical_length")?;
        let physical_end = self
            .physical
            .checked_add(length64)
            .ok_or("surface_capture_physical_overflow")?;
        authorize_smbios_interval(self.physical, physical_end, memory_map)?;
        let virtual_end = self
            .virtual_address
            .checked_add(length64)
            .ok_or("surface_capture_hhdm_overflow")?;
        let virtual_last = virtual_end
            .checked_sub(1)
            .ok_or("surface_capture_hhdm_overflow")?;
        if !is_canonical_x86_64(self.virtual_address) || !is_canonical_x86_64(virtual_last) {
            return Err("surface_capture_hhdm_noncanonical");
        }
        let pointer = usize::try_from(self.virtual_address)
            .map_err(|_| "surface_capture_hhdm_range")? as *const u8;
        // SAFETY: Reason: Limine supplies physical SMBIOS pointers. Invariant: this exact interval is bounded, canonical, HHDM-translated once, and owned by one allowed memory-map entry.
        Ok(unsafe { core::slice::from_raw_parts(pointer, length) })
    }
}

fn authorize_smbios_interval(
    physical: u64,
    physical_end: u64,
    memory_map: &MemoryMapResponse,
) -> Result<(), &'static str> {
    let mut containing_type = None;
    for entry in memory_map.entries() {
        let entry_end = entry
            .base
            .checked_add(entry.length)
            .ok_or("surface_capture_memory_range_invalid")?;
        if entry.base <= physical && physical_end <= entry_end {
            if containing_type.replace(entry.entry_type).is_some() {
                return Err("surface_capture_smbios_memory_map_ambiguous");
            }
        }
    }
    let entry_type = containing_type.ok_or("surface_capture_smbios_memory_map_unmapped")?;
    if entry_type != EntryType::RESERVED
        && entry_type != EntryType::ACPI_RECLAIMABLE
        && entry_type != EntryType::ACPI_NVS
        && entry_type != EntryType::BOOTLOADER_RECLAIMABLE
    {
        return Err("surface_capture_smbios_memory_map_type");
    }
    Ok(())
}

fn is_canonical_x86_64(address: u64) -> bool {
    let upper = address >> 48;
    if address & (1 << 47) == 0 { upper == 0 } else { upper == 0xffff }
}

fn cpuid(leaf: u32, subleaf: u32) -> CpuidResult {
    // SAFETY: Reason: x86_64 guarantees CPUID. Invariant: callers use only the bounded set selected from reported maxima.
    unsafe { __cpuid_count(leaf, subleaf) }
}

fn push_bounded(payloads: &mut Vec<CapturePayload>, payload: CapturePayload) -> Result<(), &'static str> {
    if payloads.len() >= SURFACE_FACT_CAPTURE_MAX_PARTS - 1 {
        return Err("surface_capture_too_many_parts");
    }
    payloads.push(payload);
    Ok(())
}

fn capture_id(nonce: u64, payloads: &[CapturePayload]) -> [u8; 16] {
    let mut hash = Sha256::new();
    hash.update(b"raios.surface_capture.boot_observations.v1\0");
    hash.update(nonce.to_le_bytes());
    hash.update((payloads.len() as u64).to_le_bytes());
    for payload in payloads {
        hash.update([payload.kind() as u8]);
        match payload {
            CapturePayload::Cpu(value) => hash.update([
                value.leaf.to_le_bytes(), value.subleaf.to_le_bytes(), value.eax.to_le_bytes(),
                value.ebx.to_le_bytes(), value.ecx.to_le_bytes(), value.edx.to_le_bytes(),
            ].concat()),
            CapturePayload::SmbiosMemory(value) => hash.update(value.size_bytes.to_le_bytes()),
            CapturePayload::LimineMemoryRegion(value) => {
                hash.update(value.base.to_le_bytes()); hash.update(value.length.to_le_bytes());
            }
            CapturePayload::PciDevice(value) => {
                hash.update([value.bus, value.device, value.function]);
                hash.update(value.vendor_id.to_le_bytes()); hash.update(value.device_id.to_le_bytes());
            }
            CapturePayload::Completion(_) => {}
        }
    }
    let digest: [u8; 32] = hash.finalize().into();
    let mut id = [0; 16];
    id.copy_from_slice(&digest[..16]);
    id
}

fn count_parts(payloads: &[CapturePayload]) -> Result<[u16; 4], &'static str> {
    let mut counts = [0u16; 4];
    for payload in payloads {
        let slot = match payload {
            CapturePayload::Cpu(_) => 0,
            CapturePayload::SmbiosMemory(_) => 1,
            CapturePayload::LimineMemoryRegion(_) => 2,
            CapturePayload::PciDevice(_) => 3,
            CapturePayload::Completion(_) => return Err("surface_capture_early_completion"),
        };
        counts[slot] = counts[slot].checked_add(1).ok_or("surface_capture_count_overflow")?;
    }
    Ok(counts)
}

fn record(id: [u8; 16], index: u16, count: u16, payload: CapturePayload) -> CaptureRecord {
    CaptureRecord {
        header: CaptureHeader {
            schema_version: SURFACE_FACT_CAPTURE_SCHEMA_VERSION,
            capture_id: id,
            part_kind: payload.kind(),
            part_index: index,
            part_count: count,
        },
        payload,
    }
}

fn checksum(bytes: &[u8]) -> u8 { bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)) }
fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, &'static str> {
    let end = offset.checked_add(2).ok_or("surface_capture_smbios_field")?;
    let value = bytes.get(offset..end).ok_or("surface_capture_smbios_field")?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}
fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, &'static str> {
    let end = offset.checked_add(4).ok_or("surface_capture_smbios_field")?;
    let value = bytes.get(offset..end).ok_or("surface_capture_smbios_field")?;
    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}
fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, &'static str> {
    let end = offset.checked_add(8).ok_or("surface_capture_smbios_field")?;
    let value = bytes.get(offset..end).ok_or("surface_capture_smbios_field")?;
    Ok(u64::from_le_bytes(value.try_into().map_err(|_| "surface_capture_smbios_field")?))
}
