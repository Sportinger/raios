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
pub const VTD_PAGE_SIZE: u64 = 4096;
pub const VTD_PAGE_ENTRIES: usize = 512;
pub const VTD_ROOT_ENTRY_COUNT: usize = 256;
pub const VTD_CONTEXT_ENTRY_COUNT: usize = 256;
pub const VTD_MAX_DMA_MAPPINGS: usize = 8;
pub const VTD_SL_TABLE_PAGE_COUNT: usize = 64;

const DMAR_SIGNATURE: &[u8] = b"DMAR";
const VTD_PAGE_MASK: u64 = !(VTD_PAGE_SIZE - 1);
const VTD_INDEX_MASK: u64 = 0x1ff;
const VTD_STRIDE_BITS: u8 = 9;
const VTD_PAGE_SHIFT: u8 = 12;
const VTD_MAX_SL_AGAW: u8 = 2;
const DMA_PTE_READ: u64 = 1 << 0;
const DMA_PTE_WRITE: u64 = 1 << 1;
const ROOT_ENTRY_PRESENT: u64 = 1 << 0;
const CONTEXT_ENTRY_PRESENT: u64 = 1 << 0;
const CONTEXT_DOMAIN_ID_SHIFT: u8 = 8;

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

#[cfg(not(test))]
static VTD_DOMAIN_TABLE_LOCK: Mutex<()> = Mutex::new(());

#[cfg(not(test))]
static mut VTD_ROOT_TABLE: VtdRootTable = VtdRootTable::empty();
#[cfg(not(test))]
static mut VTD_CONTEXT_TABLE: VtdContextTable = VtdContextTable::empty();
#[cfg(not(test))]
static mut VTD_SL_TABLES: VtdSlTableStorage<VTD_SL_TABLE_PAGE_COUNT> = VtdSlTableStorage::empty();

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VtdDeviceSelector {
    pub segment: u16,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VtdDmaMapping {
    pub host_phys: u64,
    pub iova: u64,
    pub len: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VtdMappedRegion {
    pub host_phys: u64,
    pub iova: u64,
    pub len: u64,
    pub page_count: u64,
}

impl VtdMappedRegion {
    const EMPTY: Self = Self {
        host_phys: 0,
        iova: 0,
        len: 0,
        page_count: 0,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VtdSlConfig {
    pub agaw: u8,
    pub levels: u8,
    pub address_width: u8,
    pub cap_mgaw: u8,
    pub cap_sagaw: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VtdBuildError {
    UnsupportedAddressWidth,
    InvalidDeviceSelector,
    DomainIdZero,
    NoMappings,
    TooManyMappings,
    ZeroLength,
    UnalignedRegion,
    RegionAddressOverflow,
    RegionExceedsAddressWidth,
    OverlappingRegion,
    TablePhysUnaligned,
    TableAddressOverflow,
    TableCapacityOverflow,
    TablePointerOutOfArena,
    LeafAlreadyMapped,
    StaticTablePhysUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VtdRootEntry {
    pub lo: u64,
    pub hi: u64,
}

impl VtdRootEntry {
    const EMPTY: Self = Self { lo: 0, hi: 0 };
}

#[repr(C, align(4096))]
#[derive(Clone, Copy)]
pub struct VtdRootTable {
    pub entries: [VtdRootEntry; VTD_ROOT_ENTRY_COUNT],
}

impl VtdRootTable {
    pub const fn empty() -> Self {
        Self {
            entries: [VtdRootEntry::EMPTY; VTD_ROOT_ENTRY_COUNT],
        }
    }

    fn zero(&mut self) {
        let mut idx = 0usize;
        while idx < VTD_ROOT_ENTRY_COUNT {
            self.entries[idx] = VtdRootEntry::EMPTY;
            idx += 1;
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VtdContextEntry {
    pub lo: u64,
    pub hi: u64,
}

impl VtdContextEntry {
    const EMPTY: Self = Self { lo: 0, hi: 0 };
}

#[repr(C, align(4096))]
#[derive(Clone, Copy)]
pub struct VtdContextTable {
    pub entries: [VtdContextEntry; VTD_CONTEXT_ENTRY_COUNT],
}

impl VtdContextTable {
    pub const fn empty() -> Self {
        Self {
            entries: [VtdContextEntry::EMPTY; VTD_CONTEXT_ENTRY_COUNT],
        }
    }

    fn zero(&mut self) {
        let mut idx = 0usize;
        while idx < VTD_CONTEXT_ENTRY_COUNT {
            self.entries[idx] = VtdContextEntry::EMPTY;
            idx += 1;
        }
    }
}

#[repr(C, align(4096))]
#[derive(Clone, Copy)]
pub struct VtdPageTable {
    pub entries: [u64; VTD_PAGE_ENTRIES],
}

impl VtdPageTable {
    pub const fn empty() -> Self {
        Self {
            entries: [0; VTD_PAGE_ENTRIES],
        }
    }

    fn zero(&mut self) {
        let mut idx = 0usize;
        while idx < VTD_PAGE_ENTRIES {
            self.entries[idx] = 0;
            idx += 1;
        }
    }
}

#[derive(Clone, Copy)]
pub struct VtdSlTableStorage<const N: usize> {
    pub tables: [VtdPageTable; N],
    pub next_free: usize,
}

impl<const N: usize> VtdSlTableStorage<N> {
    pub const fn empty() -> Self {
        Self {
            tables: [VtdPageTable::empty(); N],
            next_free: 0,
        }
    }

    fn reset(&mut self) {
        let mut idx = 0usize;
        while idx < N {
            self.tables[idx].zero();
            idx += 1;
        }
        self.next_free = 0;
    }

    fn alloc_table(&mut self, base_phys: u64) -> Result<(usize, u64), VtdBuildError> {
        if self.next_free >= N {
            return Err(VtdBuildError::TableCapacityOverflow);
        }
        let index = self.next_free;
        self.next_free += 1;
        self.tables[index].zero();
        let offset = (index as u64)
            .checked_mul(VTD_PAGE_SIZE)
            .ok_or(VtdBuildError::TableAddressOverflow)?;
        let phys = base_phys
            .checked_add(offset)
            .ok_or(VtdBuildError::TableAddressOverflow)?;
        Ok((index, phys))
    }

    fn index_from_phys(&self, base_phys: u64, phys: u64) -> Result<usize, VtdBuildError> {
        if phys < base_phys || (phys & (VTD_PAGE_SIZE - 1)) != 0 {
            return Err(VtdBuildError::TablePointerOutOfArena);
        }
        let offset = phys - base_phys;
        let index = (offset / VTD_PAGE_SIZE) as usize;
        if index >= self.next_free {
            return Err(VtdBuildError::TablePointerOutOfArena);
        }
        Ok(index)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VtdDomainBuildReport {
    pub segment: u16,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub agaw: u8,
    pub levels: u8,
    pub address_width: u8,
    pub mapped_region_count: usize,
    pub mapped_page_count: u64,
    pub mapped_regions: [VtdMappedRegion; VTD_MAX_DMA_MAPPINGS],
    pub sl_root_phys: u64,
    pub root_table_phys: u64,
    pub context_table_phys: u64,
    pub domain_id: u16,
    pub root_entry_lo: u64,
    pub root_entry_hi: u64,
    pub context_entry_lo: u64,
    pub context_entry_hi: u64,
    pub installed: bool,
    pub translation_enabled: bool,
    pub remapping_enabled: bool,
    pub reason: &'static str,
}

pub fn vtd_sl_config_from_cap(cap: u64) -> Result<VtdSlConfig, VtdBuildError> {
    let sagaw = ((cap >> 8) & 0x1f) as u8;
    let mgaw = (((cap >> 16) & 0x3f) + 1) as u8;
    let mut agaw = VTD_MAX_SL_AGAW;
    loop {
        let width = agaw_to_width(agaw);
        if (sagaw & (1 << agaw)) != 0 && width <= mgaw {
            return Ok(VtdSlConfig {
                agaw,
                levels: agaw + 2,
                address_width: width,
                cap_mgaw: mgaw,
                cap_sagaw: sagaw,
            });
        }
        if agaw == 0 {
            break;
        }
        agaw -= 1;
    }
    Err(VtdBuildError::UnsupportedAddressWidth)
}

pub fn sl_index_for_level(iova: u64, level: u8) -> Result<usize, VtdBuildError> {
    if level == 0 || level > 4 {
        return Err(VtdBuildError::UnsupportedAddressWidth);
    }
    let shift = VTD_PAGE_SHIFT + (level - 1) * VTD_STRIDE_BITS;
    Ok(((iova >> shift) & VTD_INDEX_MASK) as usize)
}

pub fn dma_pte_for_page(host_phys: u64) -> Result<u64, VtdBuildError> {
    page_aligned(host_phys)?;
    Ok((host_phys & VTD_PAGE_MASK) | DMA_PTE_READ | DMA_PTE_WRITE)
}

pub fn pack_root_entry(context_table_phys: u64) -> Result<VtdRootEntry, VtdBuildError> {
    table_phys_aligned(context_table_phys)?;
    Ok(VtdRootEntry {
        lo: (context_table_phys & VTD_PAGE_MASK) | ROOT_ENTRY_PRESENT,
        hi: 0,
    })
}

pub fn pack_context_entry(
    sl_root_phys: u64,
    config: VtdSlConfig,
    domain_id: u16,
) -> Result<VtdContextEntry, VtdBuildError> {
    if domain_id == 0 {
        return Err(VtdBuildError::DomainIdZero);
    }
    table_phys_aligned(sl_root_phys)?;
    Ok(VtdContextEntry {
        lo: (sl_root_phys & VTD_PAGE_MASK) | CONTEXT_ENTRY_PRESENT,
        hi: (config.agaw as u64) | ((domain_id as u64) << CONTEXT_DOMAIN_ID_SHIFT),
    })
}

pub fn construct_vtd_domain_tables<const N: usize>(
    device: VtdDeviceSelector,
    mappings: &[VtdDmaMapping],
    cap: u64,
    domain_id: u16,
    root_table_phys: u64,
    context_table_phys: u64,
    sl_table_phys_base: u64,
    root_table: &mut VtdRootTable,
    context_table: &mut VtdContextTable,
    sl_tables: &mut VtdSlTableStorage<N>,
) -> Result<VtdDomainBuildReport, VtdBuildError> {
    validate_device(device)?;
    if domain_id == 0 {
        return Err(VtdBuildError::DomainIdZero);
    }
    table_phys_aligned(root_table_phys)?;
    table_phys_aligned(context_table_phys)?;
    table_phys_aligned(sl_table_phys_base)?;

    let config = vtd_sl_config_from_cap(cap)?;
    validate_mappings(config, mappings)?;

    root_table.zero();
    context_table.zero();
    sl_tables.reset();

    let (sl_root_index, sl_root_phys) = sl_tables.alloc_table(sl_table_phys_base)?;
    let mut mapped_regions = [VtdMappedRegion::EMPTY; VTD_MAX_DMA_MAPPINGS];
    let mut mapped_page_count = 0u64;

    let mut region_idx = 0usize;
    while region_idx < mappings.len() {
        let mapping = mappings[region_idx];
        let pages = mapping.len / VTD_PAGE_SIZE;
        let mut page = 0u64;
        while page < pages {
            let offset = page
                .checked_mul(VTD_PAGE_SIZE)
                .ok_or(VtdBuildError::RegionAddressOverflow)?;
            map_sl_page(
                config,
                sl_root_index,
                sl_table_phys_base,
                sl_tables,
                mapping.iova + offset,
                mapping.host_phys + offset,
            )?;
            page += 1;
        }
        mapped_regions[region_idx] = VtdMappedRegion {
            host_phys: mapping.host_phys,
            iova: mapping.iova,
            len: mapping.len,
            page_count: pages,
        };
        mapped_page_count = mapped_page_count
            .checked_add(pages)
            .ok_or(VtdBuildError::RegionAddressOverflow)?;
        region_idx += 1;
    }

    let root_entry = pack_root_entry(context_table_phys)?;
    let context_entry = pack_context_entry(sl_root_phys, config, domain_id)?;
    let devfn = devfn(device);
    root_table.entries[device.bus as usize] = root_entry;
    context_table.entries[devfn as usize] = context_entry;

    Ok(VtdDomainBuildReport {
        segment: device.segment,
        bus: device.bus,
        device: device.device,
        function: device.function,
        agaw: config.agaw,
        levels: config.levels,
        address_width: config.address_width,
        mapped_region_count: mappings.len(),
        mapped_page_count,
        mapped_regions,
        sl_root_phys,
        root_table_phys,
        context_table_phys,
        domain_id,
        root_entry_lo: root_entry.lo,
        root_entry_hi: root_entry.hi,
        context_entry_lo: context_entry.lo,
        context_entry_hi: context_entry.hi,
        installed: false,
        translation_enabled: false,
        remapping_enabled: false,
        reason: "constructed_not_enforced",
    })
}

#[cfg(not(test))]
pub fn build_identity_or_granted_domain(
    device: VtdDeviceSelector,
    mappings: &[VtdDmaMapping],
    cap: u64,
    domain_id: u16,
) -> Result<VtdDomainBuildReport, VtdBuildError> {
    let _guard = VTD_DOMAIN_TABLE_LOCK.lock();
    unsafe {
        let root_ptr = ptr::addr_of_mut!(VTD_ROOT_TABLE);
        let context_ptr = ptr::addr_of_mut!(VTD_CONTEXT_TABLE);
        let sl_ptr = ptr::addr_of_mut!(VTD_SL_TABLES);
        let root_phys = memory::virt_to_phys(root_ptr.cast_const())
            .ok_or(VtdBuildError::StaticTablePhysUnavailable)?;
        let context_phys = memory::virt_to_phys(context_ptr.cast_const())
            .ok_or(VtdBuildError::StaticTablePhysUnavailable)?;
        let sl_phys = memory::virt_to_phys(sl_ptr.cast_const())
            .ok_or(VtdBuildError::StaticTablePhysUnavailable)?;

        construct_vtd_domain_tables(
            device,
            mappings,
            cap,
            domain_id,
            root_phys,
            context_phys,
            sl_phys,
            &mut *root_ptr,
            &mut *context_ptr,
            &mut *sl_ptr,
        )
    }
}

fn validate_device(device: VtdDeviceSelector) -> Result<(), VtdBuildError> {
    if device.device >= 32 || device.function >= 8 {
        return Err(VtdBuildError::InvalidDeviceSelector);
    }
    Ok(())
}

fn validate_mappings(config: VtdSlConfig, mappings: &[VtdDmaMapping]) -> Result<(), VtdBuildError> {
    if mappings.is_empty() {
        return Err(VtdBuildError::NoMappings);
    }
    if mappings.len() > VTD_MAX_DMA_MAPPINGS {
        return Err(VtdBuildError::TooManyMappings);
    }

    let mut idx = 0usize;
    while idx < mappings.len() {
        validate_mapping(config, mappings[idx])?;
        let mut other = idx + 1;
        while other < mappings.len() {
            if ranges_overlap(
                mappings[idx].iova,
                mappings[idx].len,
                mappings[other].iova,
                mappings[other].len,
            )? {
                return Err(VtdBuildError::OverlappingRegion);
            }
            other += 1;
        }
        idx += 1;
    }
    Ok(())
}

fn validate_mapping(config: VtdSlConfig, mapping: VtdDmaMapping) -> Result<(), VtdBuildError> {
    if mapping.len == 0 {
        return Err(VtdBuildError::ZeroLength);
    }
    page_aligned(mapping.host_phys)?;
    page_aligned(mapping.iova)?;
    page_aligned(mapping.len)?;
    let iova_end = mapping
        .iova
        .checked_add(mapping.len)
        .ok_or(VtdBuildError::RegionAddressOverflow)?;
    mapping
        .host_phys
        .checked_add(mapping.len)
        .ok_or(VtdBuildError::RegionAddressOverflow)?;
    if iova_end > address_limit(config.address_width) {
        return Err(VtdBuildError::RegionExceedsAddressWidth);
    }
    Ok(())
}

fn ranges_overlap(
    a_start: u64,
    a_len: u64,
    b_start: u64,
    b_len: u64,
) -> Result<bool, VtdBuildError> {
    let a_end = a_start
        .checked_add(a_len)
        .ok_or(VtdBuildError::RegionAddressOverflow)?;
    let b_end = b_start
        .checked_add(b_len)
        .ok_or(VtdBuildError::RegionAddressOverflow)?;
    Ok(a_start < b_end && b_start < a_end)
}

fn map_sl_page<const N: usize>(
    config: VtdSlConfig,
    sl_root_index: usize,
    sl_table_phys_base: u64,
    sl_tables: &mut VtdSlTableStorage<N>,
    iova: u64,
    host_phys: u64,
) -> Result<(), VtdBuildError> {
    let mut table_index = sl_root_index;
    let mut level = config.levels;
    while level > 1 {
        let index = sl_index_for_level(iova, level)?;
        let entry = sl_tables.tables[table_index].entries[index];
        if entry & DMA_PTE_READ == 0 {
            let (next_index, next_phys) = sl_tables.alloc_table(sl_table_phys_base)?;
            sl_tables.tables[table_index].entries[index] = dma_pte_for_page(next_phys)?;
            table_index = next_index;
        } else {
            let next_phys = entry & VTD_PAGE_MASK;
            table_index = sl_tables.index_from_phys(sl_table_phys_base, next_phys)?;
        }
        level -= 1;
    }

    let leaf_index = sl_index_for_level(iova, 1)?;
    if sl_tables.tables[table_index].entries[leaf_index] & DMA_PTE_READ != 0 {
        return Err(VtdBuildError::LeafAlreadyMapped);
    }
    sl_tables.tables[table_index].entries[leaf_index] = dma_pte_for_page(host_phys)?;
    Ok(())
}

fn page_aligned(value: u64) -> Result<(), VtdBuildError> {
    if value & (VTD_PAGE_SIZE - 1) != 0 {
        return Err(VtdBuildError::UnalignedRegion);
    }
    Ok(())
}

fn table_phys_aligned(value: u64) -> Result<(), VtdBuildError> {
    if value & (VTD_PAGE_SIZE - 1) != 0 {
        return Err(VtdBuildError::TablePhysUnaligned);
    }
    Ok(())
}

fn address_limit(width: u8) -> u64 {
    if width >= 64 {
        u64::MAX
    } else {
        1u64 << width
    }
}

const fn agaw_to_width(agaw: u8) -> u8 {
    30 + agaw * 9
}

const fn devfn(device: VtdDeviceSelector) -> u8 {
    (device.device << 3) | device.function
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

    const ROOT_PHYS: u64 = 0x1000_0000;
    const CONTEXT_PHYS: u64 = 0x1000_1000;
    const SL_PHYS: u64 = 0x1000_2000;
    const TEST_DOMAIN_ID: u16 = 7;

    #[test]
    fn sl_index_extracts_each_level() {
        let iova = (0x12u64 << 39) | (0x34u64 << 30) | (0x56u64 << 21) | (0x78u64 << 12);

        assert_eq!(sl_index_for_level(iova, 4), Ok(0x12));
        assert_eq!(sl_index_for_level(iova, 3), Ok(0x34));
        assert_eq!(sl_index_for_level(iova, 2), Ok(0x56));
        assert_eq!(sl_index_for_level(iova, 1), Ok(0x78));
        assert_eq!(
            sl_index_for_level(iova, 5),
            Err(VtdBuildError::UnsupportedAddressWidth)
        );
    }

    #[test]
    fn cap_selects_highest_supported_sl_width() {
        let config = vtd_sl_config_from_cap(cap_with_sagaw((1 << 1) | (1 << 2), 48)).unwrap();
        assert_eq!(config.agaw, 2);
        assert_eq!(config.levels, 4);
        assert_eq!(config.address_width, 48);

        let config = vtd_sl_config_from_cap(cap_with_sagaw((1 << 1) | (1 << 2), 39)).unwrap();
        assert_eq!(config.agaw, 1);
        assert_eq!(config.levels, 3);
        assert_eq!(config.address_width, 39);

        assert_eq!(
            vtd_sl_config_from_cap(cap_with_sagaw(1 << 3, 57)),
            Err(VtdBuildError::UnsupportedAddressWidth)
        );
    }

    #[test]
    fn dma_pte_sets_read_write_and_present_bits() {
        assert_eq!(dma_pte_for_page(0x1234_5000), Ok(0x1234_5003));
        assert_eq!(
            dma_pte_for_page(0x1234_5001),
            Err(VtdBuildError::UnalignedRegion)
        );
    }

    #[test]
    fn maps_single_4k_region() {
        let mut root = VtdRootTable::empty();
        let mut context = VtdContextTable::empty();
        let mut sl = VtdSlTableStorage::<8>::empty();
        let mapping = [VtdDmaMapping {
            host_phys: 0x2000_0000,
            iova: 0x0000_8123_4567_8000,
            len: VTD_PAGE_SIZE,
        }];

        let report = construct_vtd_domain_tables(
            device(),
            &mapping,
            cap_48(),
            TEST_DOMAIN_ID,
            ROOT_PHYS,
            CONTEXT_PHYS,
            SL_PHYS,
            &mut root,
            &mut context,
            &mut sl,
        )
        .unwrap();

        assert_eq!(report.levels, 4);
        assert_eq!(report.sl_root_phys, SL_PHYS);
        assert_eq!(report.mapped_region_count, 1);
        assert_eq!(report.mapped_page_count, 1);
        assert_eq!(
            leaf_pte(&sl, SL_PHYS, 0, report, mapping[0].iova),
            0x2000_0003
        );
    }

    #[test]
    fn maps_multi_page_region() {
        let mut root = VtdRootTable::empty();
        let mut context = VtdContextTable::empty();
        let mut sl = VtdSlTableStorage::<8>::empty();
        let mapping = [VtdDmaMapping {
            host_phys: 0x3000_0000,
            iova: 0x4000_0000,
            len: VTD_PAGE_SIZE * 3,
        }];

        let report = construct_vtd_domain_tables(
            device(),
            &mapping,
            cap_48(),
            TEST_DOMAIN_ID,
            ROOT_PHYS,
            CONTEXT_PHYS,
            SL_PHYS,
            &mut root,
            &mut context,
            &mut sl,
        )
        .unwrap();

        assert_eq!(report.mapped_page_count, 3);
        assert_eq!(leaf_pte(&sl, SL_PHYS, 0, report, 0x4000_0000), 0x3000_0003);
        assert_eq!(leaf_pte(&sl, SL_PHYS, 0, report, 0x4000_1000), 0x3000_1003);
        assert_eq!(leaf_pte(&sl, SL_PHYS, 0, report, 0x4000_2000), 0x3000_2003);
    }

    #[test]
    fn rejects_alignment_overlap_and_oversize() {
        let unaligned = [VtdDmaMapping {
            host_phys: 0x2000_0001,
            iova: 0x4000_0000,
            len: VTD_PAGE_SIZE,
        }];
        assert_eq!(
            construct_with_mappings(&unaligned),
            Err(VtdBuildError::UnalignedRegion)
        );

        let overlapping = [
            VtdDmaMapping {
                host_phys: 0x2000_0000,
                iova: 0x4000_0000,
                len: VTD_PAGE_SIZE * 2,
            },
            VtdDmaMapping {
                host_phys: 0x2000_2000,
                iova: 0x4000_1000,
                len: VTD_PAGE_SIZE,
            },
        ];
        assert_eq!(
            construct_with_mappings(&overlapping),
            Err(VtdBuildError::OverlappingRegion)
        );

        let oversized = [VtdDmaMapping {
            host_phys: 0x2000_0000,
            iova: 1u64 << 48,
            len: VTD_PAGE_SIZE,
        }];
        assert_eq!(
            construct_with_mappings(&oversized),
            Err(VtdBuildError::RegionExceedsAddressWidth)
        );
    }

    #[test]
    fn rejects_table_capacity_overflow() {
        let mut root = VtdRootTable::empty();
        let mut context = VtdContextTable::empty();
        let mut sl = VtdSlTableStorage::<1>::empty();
        let mapping = [VtdDmaMapping {
            host_phys: 0x2000_0000,
            iova: 0x4000_0000,
            len: VTD_PAGE_SIZE,
        }];

        assert_eq!(
            construct_vtd_domain_tables(
                device(),
                &mapping,
                cap_48(),
                TEST_DOMAIN_ID,
                ROOT_PHYS,
                CONTEXT_PHYS,
                SL_PHYS,
                &mut root,
                &mut context,
                &mut sl,
            ),
            Err(VtdBuildError::TableCapacityOverflow)
        );
    }

    #[test]
    fn packs_root_and_context_entry_fields() {
        let mut root = VtdRootTable::empty();
        let mut context = VtdContextTable::empty();
        let mut sl = VtdSlTableStorage::<8>::empty();
        let mapping = [VtdDmaMapping {
            host_phys: 0x2000_0000,
            iova: 0x4000_0000,
            len: VTD_PAGE_SIZE,
        }];
        let dev = device();

        let report = construct_vtd_domain_tables(
            dev,
            &mapping,
            cap_48(),
            TEST_DOMAIN_ID,
            ROOT_PHYS,
            CONTEXT_PHYS,
            SL_PHYS,
            &mut root,
            &mut context,
            &mut sl,
        )
        .unwrap();

        let devfn = ((dev.device << 3) | dev.function) as usize;
        assert_eq!(root.entries[dev.bus as usize].lo, CONTEXT_PHYS | 1);
        assert_eq!(root.entries[dev.bus as usize].hi, 0);
        assert_eq!(context.entries[devfn].lo, SL_PHYS | 1);
        assert_eq!(
            context.entries[devfn].hi,
            2 | ((TEST_DOMAIN_ID as u64) << CONTEXT_DOMAIN_ID_SHIFT)
        );
        assert_eq!(report.root_entry_lo, CONTEXT_PHYS | 1);
        assert_eq!(report.context_entry_lo, SL_PHYS | 1);
        assert!(!report.installed);
        assert!(!report.translation_enabled);
        assert!(!report.remapping_enabled);
        assert_eq!(report.reason, "constructed_not_enforced");
    }

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

    fn construct_with_mappings(
        mappings: &[VtdDmaMapping],
    ) -> Result<VtdDomainBuildReport, VtdBuildError> {
        let mut root = VtdRootTable::empty();
        let mut context = VtdContextTable::empty();
        let mut sl = VtdSlTableStorage::<8>::empty();
        construct_vtd_domain_tables(
            device(),
            mappings,
            cap_48(),
            TEST_DOMAIN_ID,
            ROOT_PHYS,
            CONTEXT_PHYS,
            SL_PHYS,
            &mut root,
            &mut context,
            &mut sl,
        )
    }

    fn leaf_pte<const N: usize>(
        sl: &VtdSlTableStorage<N>,
        base_phys: u64,
        root_index: usize,
        report: VtdDomainBuildReport,
        iova: u64,
    ) -> u64 {
        let config = VtdSlConfig {
            agaw: report.agaw,
            levels: report.levels,
            address_width: report.address_width,
            cap_mgaw: report.address_width,
            cap_sagaw: 1 << report.agaw,
        };
        let mut table_index = root_index;
        let mut level = config.levels;
        while level > 1 {
            let entry = sl.tables[table_index].entries[sl_index_for_level(iova, level).unwrap()];
            table_index = sl
                .index_from_phys(base_phys, entry & VTD_PAGE_MASK)
                .unwrap();
            level -= 1;
        }
        sl.tables[table_index].entries[sl_index_for_level(iova, 1).unwrap()]
    }

    fn device() -> VtdDeviceSelector {
        VtdDeviceSelector {
            segment: 0,
            bus: 2,
            device: 3,
            function: 1,
        }
    }

    fn cap_48() -> u64 {
        cap_with_sagaw(1 << 2, 48)
    }

    fn cap_with_sagaw(sagaw: u8, mgaw: u8) -> u64 {
        ((mgaw as u64 - 1) << 16) | ((sagaw as u64) << 8)
    }
}
