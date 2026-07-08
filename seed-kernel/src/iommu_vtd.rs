#[cfg(not(test))]
use core::{ptr, slice};

#[cfg(not(test))]
use limine::request::RsdpRequest;
#[cfg(not(test))]
use spin::Mutex;

#[cfg(not(test))]
use crate::memory;

const ACPI_HEADER_LEN: usize = 36;
const DMAR_HEADER_LEN: usize = 48;
const DRHD_TYPE: u16 = 0;
const DRHD_LEN: usize = 16;
pub const DRHD_FLAG_INCLUDE_PCI_ALL: u8 = 1;
pub const MAX_DRHD_UNITS: usize = 8;

const DMAR_SIGNATURE: &[u8] = b"DMAR";

#[cfg(not(test))]
const RSDP_SIGNATURE: &[u8] = b"RSD PTR ";
#[cfg(not(test))]
const RSDT_SIGNATURE: &[u8] = b"RSDT";
#[cfg(not(test))]
const XSDT_SIGNATURE: &[u8] = b"XSDT";
#[cfg(not(test))]
const RSDP_V1_LEN: usize = 20;
#[cfg(not(test))]
const RSDP_V2_MIN_LEN: usize = 36;
#[cfg(not(test))]
const MAX_RSDP_LEN: usize = 256;
#[cfg(not(test))]
const MAX_ACPI_TABLE_LEN: usize = 1024 * 1024;
#[cfg(not(test))]
const VT_D_REGISTER_WINDOW_LEN: usize = 0x18;
#[cfg(not(test))]
const VT_D_VER_REG: usize = 0x00;
#[cfg(not(test))]
const VT_D_CAP_REG: usize = 0x08;
#[cfg(not(test))]
const VT_D_ECAP_REG: usize = 0x10;
#[cfg(not(test))]
const NO_DMAR_REASON: &str = "no DMAR (VT-d absent or not exposed)";

#[cfg(not(test))]
#[used]
#[link_section = ".limine_requests"]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrhdEntry {
    pub register_base_address: u64,
    pub segment: u16,
    pub flags: u8,
}

impl DrhdEntry {
    const EMPTY: Self = Self {
        register_base_address: 0,
        segment: 0,
        flags: 0,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DmarInfo {
    pub drhd: [DrhdEntry; MAX_DRHD_UNITS],
    pub drhd_count: usize,
}

impl DmarInfo {
    const fn empty() -> Self {
        Self {
            drhd: [DrhdEntry::EMPTY; MAX_DRHD_UNITS],
            drhd_count: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DmarParseError {
    ShortTable,
    BadSignature,
    BadLength,
    BadChecksum,
    BadStructureLength,
    TooManyDrhd,
}

#[derive(Clone, Copy)]
pub struct IommuReport {
    pub present: bool,
    pub version: u32,
    pub drhd_count: usize,
    pub cap: u64,
    pub ecap: u64,
    pub remapping_enabled: bool,
    pub reason: &'static str,
}

#[cfg(not(test))]
impl IommuReport {
    const fn absent(reason: &'static str) -> Self {
        Self {
            present: false,
            version: 0,
            drhd_count: 0,
            cap: 0,
            ecap: 0,
            remapping_enabled: false,
            reason,
        }
    }
}

#[cfg(not(test))]
static REPORT: Mutex<IommuReport> = Mutex::new(IommuReport::absent("not probed"));

pub fn parse_dmar(table: &[u8]) -> Result<DmarInfo, DmarParseError> {
    if table.len() < ACPI_HEADER_LEN {
        return Err(DmarParseError::ShortTable);
    }
    if !bytes_eq(table, 0, DMAR_SIGNATURE) {
        return Err(DmarParseError::BadSignature);
    }

    let Some(length) = read_le_u32(table, 4).map(|value| value as usize) else {
        return Err(DmarParseError::ShortTable);
    };
    if length < DMAR_HEADER_LEN || length > table.len() {
        return Err(DmarParseError::BadLength);
    }
    if !checksum_ok(table, length) {
        return Err(DmarParseError::BadChecksum);
    }

    let mut info = DmarInfo::empty();
    let mut offset = DMAR_HEADER_LEN;
    while offset < length {
        if length - offset < 4 {
            return Err(DmarParseError::BadStructureLength);
        }
        let entry_type = read_le_u16(table, offset).ok_or(DmarParseError::BadStructureLength)?;
        let entry_len =
            read_le_u16(table, offset + 2).ok_or(DmarParseError::BadStructureLength)? as usize;
        if entry_len < 4 {
            return Err(DmarParseError::BadStructureLength);
        }
        let Some(next) = offset.checked_add(entry_len) else {
            return Err(DmarParseError::BadStructureLength);
        };
        if next > length {
            return Err(DmarParseError::BadStructureLength);
        }

        if entry_type == DRHD_TYPE {
            if entry_len < DRHD_LEN {
                return Err(DmarParseError::BadStructureLength);
            }
            if info.drhd_count == MAX_DRHD_UNITS {
                return Err(DmarParseError::TooManyDrhd);
            }

            let flags = read_u8(table, offset + 4).ok_or(DmarParseError::BadStructureLength)?;
            let segment =
                read_le_u16(table, offset + 6).ok_or(DmarParseError::BadStructureLength)?;
            let register_base_address =
                read_le_u64(table, offset + 8).ok_or(DmarParseError::BadStructureLength)?;
            info.drhd[info.drhd_count] = DrhdEntry {
                register_base_address,
                segment,
                flags,
            };
            info.drhd_count += 1;
        }

        offset = next;
    }

    Ok(info)
}

#[cfg(not(test))]
pub fn probe() -> IommuReport {
    let report = probe_inner();
    *REPORT.lock() = report;
    report
}

#[cfg(not(test))]
#[allow(dead_code)]
pub fn report() -> IommuReport {
    *REPORT.lock()
}

#[cfg(not(test))]
fn probe_inner() -> IommuReport {
    let dmar_table = match find_dmar_table() {
        Ok(table) => table,
        Err(reason) => return IommuReport::absent(reason),
    };
    let dmar = match parse_dmar(dmar_table) {
        Ok(info) => info,
        Err(_) => return IommuReport::absent("DMAR table invalid"),
    };
    if dmar.drhd_count == 0 {
        return IommuReport::absent("DMAR has no DRHD remapping units");
    }

    let first = dmar.drhd[0];
    if first.register_base_address == 0 {
        return IommuReport::absent("DRHD register base missing");
    }

    let mapping = match memory::map_mmio(first.register_base_address, VT_D_REGISTER_WINDOW_LEN) {
        Ok(mapping) => mapping,
        Err(_) => return IommuReport::absent("VT-d MMIO map failed"),
    };
    let base = mapping.as_ptr::<u8>();
    let version = unsafe { read_reg32(base, VT_D_VER_REG) };
    let cap = unsafe { read_reg64(base, VT_D_CAP_REG) };
    let ecap = unsafe { read_reg64(base, VT_D_ECAP_REG) };

    IommuReport {
        present: true,
        version,
        drhd_count: dmar.drhd_count,
        cap,
        ecap,
        remapping_enabled: false,
        reason: "VT-d detected; remapping disabled (slice1 detect-only)",
    }
}

#[cfg(not(test))]
fn find_dmar_table() -> Result<&'static [u8], &'static str> {
    let rsdp = RSDP_REQUEST.get_response().ok_or(NO_DMAR_REASON)?;
    let root = read_rsdp_root(rsdp.address() as u64)?;
    let root_table = map_acpi_table(root.phys, root.signature)?;
    find_table_in_root(root_table, root.entry_bytes, root.signature)
}

#[cfg(not(test))]
#[derive(Clone, Copy)]
struct AcpiRoot {
    phys: u64,
    signature: &'static [u8],
    entry_bytes: usize,
}

#[cfg(not(test))]
fn read_rsdp_root(rsdp_addr: u64) -> Result<AcpiRoot, &'static str> {
    let rsdp_v1 = unsafe { map_limine_rsdp_slice(rsdp_addr, RSDP_V1_LEN)? };
    if !bytes_eq(rsdp_v1, 0, RSDP_SIGNATURE) {
        return Err("RSDP signature invalid");
    }
    if !checksum_ok(rsdp_v1, RSDP_V1_LEN) {
        return Err("RSDP checksum invalid");
    }

    let revision = read_u8(rsdp_v1, 15).ok_or("RSDP revision missing")?;
    let rsdt = read_le_u32(rsdp_v1, 16).ok_or("RSDT address missing")? as u64;
    if revision >= 2 {
        let rsdp_min = unsafe { map_limine_rsdp_slice(rsdp_addr, RSDP_V2_MIN_LEN)? };
        let length = read_le_u32(rsdp_min, 20).ok_or("RSDP length missing")? as usize;
        if !(RSDP_V2_MIN_LEN..=MAX_RSDP_LEN).contains(&length) {
            return Err("RSDP length invalid");
        }
        let rsdp_full = unsafe { map_limine_rsdp_slice(rsdp_addr, length)? };
        if !checksum_ok(rsdp_full, length) {
            return Err("RSDP extended checksum invalid");
        }
        let xsdt = read_le_u64(rsdp_full, 24).ok_or("XSDT address missing")?;
        if xsdt != 0 {
            return Ok(AcpiRoot {
                phys: xsdt,
                signature: XSDT_SIGNATURE,
                entry_bytes: 8,
            });
        }
    }

    if rsdt == 0 {
        return Err("ACPI root table address missing");
    }
    Ok(AcpiRoot {
        phys: rsdt,
        signature: RSDT_SIGNATURE,
        entry_bytes: 4,
    })
}

#[cfg(not(test))]
fn find_table_in_root(
    root: &'static [u8],
    entry_bytes: usize,
    root_signature: &[u8],
) -> Result<&'static [u8], &'static str> {
    if !bytes_eq(root, 0, root_signature) {
        return Err("ACPI root table signature invalid");
    }
    let length = acpi_table_length(root)?;
    let body_len = length
        .checked_sub(ACPI_HEADER_LEN)
        .ok_or("ACPI root table length invalid")?;
    if body_len % entry_bytes != 0 {
        return Err("ACPI root entry length invalid");
    }

    let mut offset = ACPI_HEADER_LEN;
    while offset < length {
        let table_phys = if entry_bytes == 8 {
            read_le_u64(root, offset).ok_or("ACPI XSDT entry truncated")?
        } else {
            read_le_u32(root, offset).ok_or("ACPI RSDT entry truncated")? as u64
        };
        if table_phys != 0 && acpi_table_signature(table_phys)? == DMAR_SIGNATURE {
            return map_acpi_table(table_phys, DMAR_SIGNATURE);
        }
        offset += entry_bytes;
    }

    Err(NO_DMAR_REASON)
}

#[cfg(not(test))]
fn acpi_table_signature(phys: u64) -> Result<&'static [u8], &'static str> {
    let header = unsafe { map_physical_slice(phys, ACPI_HEADER_LEN)? };
    header.get(0..4).ok_or("ACPI table signature truncated")
}

#[cfg(not(test))]
fn map_acpi_table(phys: u64, signature: &[u8]) -> Result<&'static [u8], &'static str> {
    let header = unsafe { map_physical_slice(phys, ACPI_HEADER_LEN)? };
    if !bytes_eq(header, 0, signature) {
        return Err("ACPI table signature invalid");
    }
    let length = acpi_table_length(header)?;
    let table = unsafe { map_physical_slice(phys, length)? };
    if !bytes_eq(table, 0, signature) {
        return Err("ACPI table signature invalid");
    }
    if !checksum_ok(table, length) {
        return Err("ACPI table checksum invalid");
    }
    Ok(table)
}

#[cfg(not(test))]
fn acpi_table_length(header: &[u8]) -> Result<usize, &'static str> {
    let length = read_le_u32(header, 4).ok_or("ACPI table length missing")? as usize;
    if !(ACPI_HEADER_LEN..=MAX_ACPI_TABLE_LEN).contains(&length) {
        return Err("ACPI table length invalid");
    }
    Ok(length)
}

#[cfg(not(test))]
unsafe fn map_limine_rsdp_slice(addr: u64, len: usize) -> Result<&'static [u8], &'static str> {
    if addr >= 0xffff_0000_0000_0000 {
        return Ok(slice::from_raw_parts(addr as *const u8, len));
    }
    map_physical_slice(addr, len)
}

#[cfg(not(test))]
unsafe fn map_physical_slice(phys: u64, len: usize) -> Result<&'static [u8], &'static str> {
    if len == 0 {
        return Err("ACPI mapping length zero");
    }
    let mapping = memory::map_mmio(phys, len).map_err(|_| "ACPI table map failed")?;
    Ok(slice::from_raw_parts(
        mapping.as_ptr::<u8>().cast_const(),
        len,
    ))
}

#[cfg(not(test))]
unsafe fn read_reg32(base: *mut u8, offset: usize) -> u32 {
    ptr::read_volatile(base.add(offset).cast::<u32>())
}

#[cfg(not(test))]
unsafe fn read_reg64(base: *mut u8, offset: usize) -> u64 {
    ptr::read_volatile(base.add(offset).cast::<u64>())
}

fn bytes_eq(data: &[u8], offset: usize, expected: &[u8]) -> bool {
    let Some(end) = offset.checked_add(expected.len()) else {
        return false;
    };
    data.get(offset..end) == Some(expected)
}

fn checksum_ok(data: &[u8], len: usize) -> bool {
    if len > data.len() {
        return false;
    }
    let mut sum = 0u8;
    for byte in &data[..len] {
        sum = sum.wrapping_add(*byte);
    }
    sum == 0
}

fn read_u8(data: &[u8], offset: usize) -> Option<u8> {
    data.get(offset).copied()
}

fn read_le_u16(data: &[u8], offset: usize) -> Option<u16> {
    read_le(data, offset, 2).map(|value| value as u16)
}

fn read_le_u32(data: &[u8], offset: usize) -> Option<u32> {
    read_le(data, offset, 4).map(|value| value as u32)
}

fn read_le_u64(data: &[u8], offset: usize) -> Option<u64> {
    read_le(data, offset, 8)
}

fn read_le(data: &[u8], offset: usize, width: usize) -> Option<u64> {
    let mut value = 0u64;
    let mut idx = 0usize;
    while idx < width {
        let byte = read_u8(data, offset.checked_add(idx)?)?;
        value |= (byte as u64) << ((idx * 8) as u32);
        idx += 1;
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_one_drhd() {
        let dmar = synthetic_dmar();
        let parsed = parse_dmar(&dmar).expect("synthetic DMAR parses");

        assert_eq!(parsed.drhd_count, 1);
        assert_eq!(parsed.drhd[0].register_base_address, 0x0000_0000_fed9_0000);
        assert_eq!(parsed.drhd[0].segment, 2);
        assert_eq!(parsed.drhd[0].flags & DRHD_FLAG_INCLUDE_PCI_ALL, 1);
    }

    #[test]
    fn bad_signature_fails_closed() {
        let mut dmar = synthetic_dmar();
        dmar[0] = b'X';

        assert_eq!(parse_dmar(&dmar), Err(DmarParseError::BadSignature));
    }

    #[test]
    fn short_table_fails_closed() {
        assert_eq!(parse_dmar(&[0u8; 16]), Err(DmarParseError::ShortTable));
    }

    #[test]
    fn bad_checksum_fails_closed() {
        let mut dmar = synthetic_dmar();
        dmar[10] = dmar[10].wrapping_add(1);

        assert_eq!(parse_dmar(&dmar), Err(DmarParseError::BadChecksum));
    }

    fn synthetic_dmar() -> Vec<u8> {
        let len = DMAR_HEADER_LEN + DRHD_LEN;
        let mut table = vec![0u8; len];
        table[0..4].copy_from_slice(DMAR_SIGNATURE);
        table[4..8].copy_from_slice(&(len as u32).to_le_bytes());
        table[8] = 1;
        table[36] = 39;

        let drhd = DMAR_HEADER_LEN;
        table[drhd..drhd + 2].copy_from_slice(&DRHD_TYPE.to_le_bytes());
        table[drhd + 2..drhd + 4].copy_from_slice(&(DRHD_LEN as u16).to_le_bytes());
        table[drhd + 4] = DRHD_FLAG_INCLUDE_PCI_ALL;
        table[drhd + 6..drhd + 8].copy_from_slice(&2u16.to_le_bytes());
        table[drhd + 8..drhd + 16].copy_from_slice(&0x0000_0000_fed9_0000u64.to_le_bytes());

        let sum = table.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte));
        table[9] = (0u8).wrapping_sub(sum);
        table
    }
}
