#![allow(static_mut_refs)]

use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use limine::response::{ExecutableAddressResponse, HhdmResponse};
use spin::Mutex;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

use crate::serial;

const PAGE_SIZE: usize = 4096;
const PAGE_SIZE_U64: u64 = PAGE_SIZE as u64;
const MMIO_WINDOW_BASE: u64 = 0xffff_ffff_c000_0000;
const MMIO_WINDOW_SIZE: u64 = 16 * 1024 * 1024;
const MMIO_CACHE_SLOTS: usize = 32;
const PAGE_TABLE_POOL_PAGES: usize = 64;

extern "C" {
    static __bss_end: u8;
}

/// A non-empty, checked physical half-open interval `[start, end)`.
///
/// This type describes observed host memory only. It does not imply that an
/// IOMMU translation or DMA drain is present.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PhysicalSpan {
    start: u64,
    end_exclusive: u64,
}

impl PhysicalSpan {
    pub(crate) const fn from_start_len(start: u64, len: u64) -> Option<Self> {
        if len == 0 {
            return None;
        }
        let Some(end_exclusive) = start.checked_add(len) else {
            return None;
        };
        Some(Self {
            start,
            end_exclusive,
        })
    }

    pub(crate) const fn start(self) -> u64 {
        self.start
    }

    pub(crate) const fn end_exclusive(self) -> u64 {
        self.end_exclusive
    }

    pub(crate) const fn len(self) -> u64 {
        self.end_exclusive - self.start
    }

    pub(crate) const fn contains(self, other: Self) -> bool {
        self.start <= other.start && other.end_exclusive <= self.end_exclusive
    }
}

static KERNEL_PHYSICAL_BASE: AtomicU64 = AtomicU64::new(0);
static KERNEL_VIRTUAL_BASE: AtomicU64 = AtomicU64::new(0);
static KERNEL_ADDRESS_READY: AtomicBool = AtomicBool::new(false);
static HHDM_OFFSET: AtomicU64 = AtomicU64::new(0);
static HHDM_READY: AtomicBool = AtomicBool::new(false);
static MMIO_NEXT: AtomicU64 = AtomicU64::new(MMIO_WINDOW_BASE);
static NEXT_PAGE_TABLE: AtomicUsize = AtomicUsize::new(0);
static PAGE_TABLE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy)]
struct CachedMmio {
    phys_start: u64,
    map_len: u64,
    virt_start: u64,
}

static mut MMIO_CACHE: [Option<CachedMmio>; MMIO_CACHE_SLOTS] = [None; MMIO_CACHE_SLOTS];

#[repr(align(4096))]
#[derive(Clone, Copy)]
struct PageTablePage([u8; PAGE_SIZE]);

static mut PAGE_TABLE_POOL: [PageTablePage; PAGE_TABLE_POOL_PAGES] =
    [PageTablePage([0; PAGE_SIZE]); PAGE_TABLE_POOL_PAGES];

#[derive(Clone, Copy)]
pub struct MmioMapping {
    virt: u64,
    len: usize,
}

impl MmioMapping {
    pub fn as_ptr<T>(&self) -> *mut T {
        self.virt as *mut T
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

pub fn init(
    executable_response: Option<&ExecutableAddressResponse>,
    hhdm_response: Option<&HhdmResponse>,
) {
    let Some(response) = executable_response else {
        serial::write_line("Kernel address response unavailable; DMA translation disabled");
        KERNEL_ADDRESS_READY.store(false, Ordering::Release);
        init_hhdm(hhdm_response);
        return;
    };

    let physical_base = response.physical_base();
    let virtual_base = response.virtual_base();
    KERNEL_PHYSICAL_BASE.store(physical_base, Ordering::Relaxed);
    KERNEL_VIRTUAL_BASE.store(virtual_base, Ordering::Relaxed);
    KERNEL_ADDRESS_READY.store(true, Ordering::Release);

    serial::write_fmt(format_args!(
        "Kernel address map: physical=0x{:x} virtual=0x{:x}\r\n",
        physical_base, virtual_base
    ));

    init_hhdm(hhdm_response);
}

fn init_hhdm(response: Option<&HhdmResponse>) {
    let Some(response) = response else {
        serial::write_line("HHDM response unavailable; MMIO mapping disabled");
        HHDM_READY.store(false, Ordering::Release);
        return;
    };

    let offset = response.offset();
    HHDM_OFFSET.store(offset, Ordering::Relaxed);
    HHDM_READY.store(true, Ordering::Release);
    serial::write_fmt(format_args!("HHDM offset=0x{:x}\r\n", offset));
}

/// Returns the physical interval occupied by the linked kernel image.
///
/// The Limine executable-address response is mandatory. In particular this
/// never falls back to treating a virtual address as an identity mapping.
pub(crate) fn kernel_image_physical_span() -> Option<PhysicalSpan> {
    let (physical_base, virtual_base) = kernel_address_map()?;
    let virtual_end = kernel_image_virtual_end();
    let len = virtual_end.checked_sub(virtual_base)?;
    PhysicalSpan::from_start_len(physical_base, len)
}

/// Resolves a stable virtual interval and proves that each covered page maps
/// to one contiguous physical interval. Only the Limine kernel-image and HHDM
/// mappings are authoritative here; the legacy identity fallback is excluded.
pub(crate) fn physical_span_for_virtual_range<T>(
    virtual_start: *const T,
    len: u64,
) -> Option<PhysicalSpan> {
    checked_physical_span_from_virtual_range(virtual_start as u64, len, |address| {
        managed_virt_to_phys(address)
    })
}

pub fn virt_to_phys<T>(ptr: *const T) -> Option<u64> {
    let virt = ptr as u64;
    if KERNEL_ADDRESS_READY.load(Ordering::Acquire) {
        let virtual_base = KERNEL_VIRTUAL_BASE.load(Ordering::Relaxed);
        let physical_base = KERNEL_PHYSICAL_BASE.load(Ordering::Relaxed);
        if virt >= virtual_base {
            return virt.checked_sub(virtual_base)?.checked_add(physical_base);
        }
    }

    // Heap-backed DMA buffers live in Limine's HHDM, outside the kernel image.
    if HHDM_READY.load(Ordering::Acquire) {
        let hhdm_offset = HHDM_OFFSET.load(Ordering::Relaxed);
        if virt >= hhdm_offset {
            let physical = virt.checked_sub(hhdm_offset)?;
            if physical < (1u64 << 46) {
                return Some(physical);
            }
        }
    }

    identity_phys(virt)
}

pub fn mmio_ready() -> bool {
    HHDM_READY.load(Ordering::Acquire)
}

pub fn map_mmio(phys: u64, len: usize) -> Result<MmioMapping, &'static str> {
    if len == 0 {
        return Err("MMIO mapping length is zero");
    }
    if !mmio_ready() {
        return Err("MMIO mapping unavailable without HHDM");
    }

    let page_offset = phys & (PAGE_SIZE_U64 - 1);
    let phys_start = phys & !(PAGE_SIZE_U64 - 1);
    let map_len = align_up_u64(
        page_offset
            .checked_add(len as u64)
            .ok_or("MMIO mapping length overflow")?,
        PAGE_SIZE_U64,
    )
    .ok_or("MMIO mapping alignment overflow")?;

    let _guard = PAGE_TABLE_LOCK.lock();
    // SAFETY: every MMIO_CACHE access is serialized by PAGE_TABLE_LOCK.
    let cached_virt_start = unsafe {
        MMIO_CACHE
            .iter()
            .flatten()
            .find(|cached| cached.phys_start == phys_start && cached.map_len == map_len)
            .map(|cached| cached.virt_start)
    };
    if let Some(virt_start) = cached_virt_start {
        return Ok(MmioMapping {
            virt: virt_start + page_offset,
            len,
        });
    }

    let virt_start = reserve_mmio_va(map_len)?;
    unsafe {
        map_mmio_pages(phys_start, virt_start, map_len)?;

        // SAFETY: PAGE_TABLE_LOCK remains held through mapping and insertion.
        if let Some(slot) = MMIO_CACHE.iter_mut().find(|entry| entry.is_none()) {
            *slot = Some(CachedMmio {
                phys_start,
                map_len,
                virt_start,
            });
        }
    }

    Ok(MmioMapping {
        virt: virt_start + page_offset,
        len,
    })
}

fn reserve_mmio_va(len: u64) -> Result<u64, &'static str> {
    let current = MMIO_NEXT.load(Ordering::Relaxed);
    let aligned = align_up_u64(current, PAGE_SIZE_U64).ok_or("MMIO VA alignment overflow")?;
    let next = aligned
        .checked_add(len)
        .ok_or("MMIO VA allocation overflow")?;
    if next > MMIO_WINDOW_BASE + MMIO_WINDOW_SIZE {
        return Err("MMIO VA window exhausted");
    }
    MMIO_NEXT.store(next, Ordering::Relaxed);
    Ok(aligned)
}

unsafe fn map_mmio_pages(phys_start: u64, virt_start: u64, len: u64) -> Result<(), &'static str> {
    let level_4_table = active_level_4_table()?;
    let hhdm_offset = HHDM_OFFSET.load(Ordering::Relaxed);
    let mut mapper = OffsetPageTable::new(level_4_table, VirtAddr::new(hhdm_offset));
    let mut allocator = StaticFrameAllocator;
    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::NO_CACHE
        | PageTableFlags::WRITE_THROUGH;
    let parent_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let mut offset = 0u64;
    while offset < len {
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(virt_start + offset));
        let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(phys_start + offset));
        let flush = mapper
            .map_to_with_table_flags(page, frame, flags, parent_flags, &mut allocator)
            .map_err(|_| "MMIO page map failed")?;
        flush.flush();
        offset += PAGE_SIZE_U64;
    }

    Ok(())
}

unsafe fn active_level_4_table() -> Result<&'static mut PageTable, &'static str> {
    let (frame, _) = Cr3::read();
    let hhdm_offset = HHDM_OFFSET.load(Ordering::Relaxed);
    let virt = frame
        .start_address()
        .as_u64()
        .checked_add(hhdm_offset)
        .ok_or("active PML4 HHDM address overflow")?;
    Ok(&mut *VirtAddr::new(virt).as_mut_ptr())
}

struct StaticFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for StaticFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let index = NEXT_PAGE_TABLE.fetch_add(1, Ordering::Relaxed);
        if index >= PAGE_TABLE_POOL_PAGES {
            return None;
        }

        unsafe {
            let page = ptr::addr_of_mut!(PAGE_TABLE_POOL)
                .cast::<PageTablePage>()
                .add(index);
            ptr::write_bytes((*page).0.as_mut_ptr(), 0, PAGE_SIZE);
            let phys = virt_to_phys((*page).0.as_ptr())?;
            PhysFrame::from_start_address(PhysAddr::new(phys)).ok()
        }
    }
}

fn align_up_u64(value: u64, align: u64) -> Option<u64> {
    let mask = align.checked_sub(1)?;
    value.checked_add(mask).map(|v| v & !mask)
}

fn identity_phys(virt: u64) -> Option<u64> {
    if virt < 0x0000_8000_0000_0000 {
        Some(virt)
    } else {
        None
    }
}

fn kernel_address_map() -> Option<(u64, u64)> {
    if !KERNEL_ADDRESS_READY.load(Ordering::Acquire) {
        return None;
    }
    Some((
        KERNEL_PHYSICAL_BASE.load(Ordering::Relaxed),
        KERNEL_VIRTUAL_BASE.load(Ordering::Relaxed),
    ))
}

fn kernel_image_virtual_end() -> u64 {
    ptr::addr_of!(__bss_end) as u64
}

fn managed_virt_to_phys(virt: u64) -> Option<u64> {
    if let Some((physical_base, virtual_base)) = kernel_address_map() {
        let virtual_end = kernel_image_virtual_end();
        if virt >= virtual_base && virt < virtual_end {
            return virt.checked_sub(virtual_base)?.checked_add(physical_base);
        }
    }

    if HHDM_READY.load(Ordering::Acquire) {
        let hhdm_offset = HHDM_OFFSET.load(Ordering::Relaxed);
        if virt >= hhdm_offset {
            let physical = virt.checked_sub(hhdm_offset)?;
            if physical < (1u64 << 46) {
                return Some(physical);
            }
        }
    }

    None
}

fn checked_physical_span_from_virtual_range<F>(
    virtual_start: u64,
    len: u64,
    mut translate: F,
) -> Option<PhysicalSpan>
where
    F: FnMut(u64) -> Option<u64>,
{
    if len == 0 {
        return None;
    }
    let virtual_end = virtual_start.checked_add(len)?;
    let physical_start = translate(virtual_start)?;
    let physical_span = PhysicalSpan::from_start_len(physical_start, len)?;

    let mut virtual_at = virtual_start;
    loop {
        let offset = virtual_at.checked_sub(virtual_start)?;
        let expected = physical_start.checked_add(offset)?;
        if translate(virtual_at) != Some(expected) {
            return None;
        }

        let page_start = virtual_at & !(PAGE_SIZE_U64 - 1);
        let Some(next_page) = page_start.checked_add(PAGE_SIZE_U64) else {
            break;
        };
        if next_page >= virtual_end {
            break;
        }
        virtual_at = next_page;
    }

    let last_virtual = virtual_end.checked_sub(1)?;
    let last_physical = physical_span.end_exclusive().checked_sub(1)?;
    if translate(last_virtual) != Some(last_physical) {
        return None;
    }
    Some(physical_span)
}

#[cfg(test)]
mod physical_span_tests {
    use super::{checked_physical_span_from_virtual_range, PhysicalSpan, PAGE_SIZE_U64};

    #[test]
    fn physical_half_interval_rejects_zero_and_overflow() {
        assert_eq!(PhysicalSpan::from_start_len(0x1000, 0), None);
        assert_eq!(PhysicalSpan::from_start_len(u64::MAX, 2), None);
    }

    #[test]
    fn physical_half_interval_contains_adjacent_inner_boundary() {
        let outer = PhysicalSpan::from_start_len(0x1000, 0x2000).unwrap();
        let inner = PhysicalSpan::from_start_len(0x2000, 0x1000).unwrap();
        assert!(outer.contains(inner));
        assert_eq!(outer.end_exclusive(), inner.end_exclusive());
    }

    #[test]
    fn contiguous_virtual_translation_is_accepted() {
        let span = checked_physical_span_from_virtual_range(0x8000, PAGE_SIZE_U64 * 2, |virt| {
            virt.checked_sub(0x8000)?.checked_add(0x20_0000)
        })
        .unwrap();
        assert_eq!(span.start(), 0x20_0000);
        assert_eq!(span.len(), PAGE_SIZE_U64 * 2);
    }

    #[test]
    fn untranslated_or_discontiguous_virtual_translation_is_rejected() {
        assert_eq!(
            checked_physical_span_from_virtual_range(0x8000, PAGE_SIZE_U64, |_| None),
            None
        );
        assert_eq!(
            checked_physical_span_from_virtual_range(0x8000, PAGE_SIZE_U64 * 2, |virt| {
                let base = virt.checked_sub(0x8000)?.checked_add(0x20_0000)?;
                Some(if virt >= 0x9000 {
                    base + PAGE_SIZE_U64
                } else {
                    base
                })
            },),
            None
        );
    }
}
