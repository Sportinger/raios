use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use limine::memory_map::EntryType;

use crate::{memory, serial, ALLOCATOR, HEAP, HEAP_SIZE, HHDM_REQUEST, MEMORY_MAP_REQUEST};

const MIB: u64 = 1024 * 1024;
const MIN_HEAP_SIZE: u64 = 64 * MIB;
// 4 GiB covers the declared 1-GiB window, its 2x doubling reservation, and margin.
const MAX_HEAP_SIZE: u64 = 4096 * MIB;
const PAGE_SIZE: u64 = 4096;

#[derive(Clone, Copy)]
struct Region {
    base: u64,
    length: u64,
}

#[derive(Clone, Copy)]
enum HeapSource {
    Memmap,
    StaticFallback,
}

impl HeapSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::Memmap => "memmap",
            Self::StaticFallback => "static_fallback",
        }
    }
}

#[derive(Clone, Copy)]
enum HeapReason {
    Ok,
    NoMemmap,
    NoHhdm,
    TooSmall,
}

impl HeapReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::NoMemmap => "no_memmap",
            Self::NoHhdm => "no_hhdm",
            Self::TooSmall => "too_small",
        }
    }
}

#[derive(Clone, Copy)]
struct HeapReport {
    source: HeapSource,
    size: u64,
    base: u64,
    usable_entries: u64,
    largest_usable: u64,
    reason: HeapReason,
}

static mut HEAP_REPORT: HeapReport = HeapReport {
    source: HeapSource::StaticFallback,
    size: HEAP_SIZE as u64,
    base: 0,
    usable_entries: 0,
    largest_usable: 0,
    reason: HeapReason::NoMemmap,
};

// Published once by heap::init. Readers use only these atomics and never take
// the allocator lock or observe HEAP_REPORT's diagnostic-only mutable state.
static HEAP_SPAN_READY: AtomicBool = AtomicBool::new(false);
static HEAP_VIRTUAL_BASE: AtomicU64 = AtomicU64::new(0);
static HEAP_RUNTIME_SIZE: AtomicU64 = AtomicU64::new(0);
static HEAP_EXPECTED_PHYSICAL_READY: AtomicBool = AtomicBool::new(false);
static HEAP_EXPECTED_PHYSICAL_BASE: AtomicU64 = AtomicU64::new(0);

pub fn select_heap_region(entries: &[(u64, u64)]) -> Option<(u64, u64)> {
    let region = select_largest_region(entries.iter().copied())?;
    if region.length < MIN_HEAP_SIZE {
        return None;
    }
    Some((region.base, region.length.min(MAX_HEAP_SIZE)))
}

pub fn init() {
    let Some(memory_map) = MEMORY_MAP_REQUEST.get_response() else {
        init_static_fallback(0, 0, HeapReason::NoMemmap);
        return;
    };

    let mut usable_entries = 0u64;
    let largest_region = select_largest_region(memory_map.entries().iter().filter_map(|entry| {
        if entry.entry_type != EntryType::USABLE {
            return None;
        }
        usable_entries = usable_entries.saturating_add(1);
        Some((entry.base, entry.length))
    }));
    let largest_usable = largest_region.map_or(0, |region| region.length);

    let Some(region) = largest_region.filter(|region| region.length >= MIN_HEAP_SIZE) else {
        init_static_fallback(usable_entries, largest_usable, HeapReason::TooSmall);
        return;
    };
    let heap_len = region.length.min(MAX_HEAP_SIZE);

    let Some(hhdm) = HHDM_REQUEST.get_response() else {
        init_static_fallback(usable_entries, largest_usable, HeapReason::NoHhdm);
        return;
    };
    let Some(heap_base) = hhdm.offset().checked_add(region.base) else {
        init_static_fallback(usable_entries, largest_usable, HeapReason::NoHhdm);
        return;
    };
    if heap_base.checked_add(heap_len).is_none() {
        init_static_fallback(usable_entries, largest_usable, HeapReason::NoHhdm);
        return;
    }

    unsafe {
        ALLOCATOR.lock().init(heap_base as *mut u8, heap_len as usize);
    }
    publish_heap_span(heap_base, heap_len, Some(region.base));
    store_report(HeapReport {
        source: HeapSource::Memmap,
        size: heap_len,
        base: heap_base,
        usable_entries,
        largest_usable,
        reason: HeapReason::Ok,
    });
}

pub fn report() {
    let report = unsafe { ptr::read(ptr::addr_of!(HEAP_REPORT)) };
    serial::write_fmt(format_args!(
        "RAIOS_HEAP source={} size_mib={} base=0x{:x} usable_entries={} largest_usable_mib={} reason={}\r\n",
        report.source.as_str(),
        report.size / MIB,
        report.base,
        report.usable_entries,
        report.largest_usable / MIB,
        report.reason.as_str()
    ));
}

/// Returns the allocator arena as a contiguous physical half-interval.
///
/// This must be called only after both `heap::init` and `memory::init`. The
/// snapshot is lock-free and fail-closed: it revalidates the full virtual
/// interval page-by-page and, for the Limine memmap arena, also requires the
/// translation to agree with the physical base selected during heap init.
pub(crate) fn physical_span() -> Option<memory::PhysicalSpan> {
    if !HEAP_SPAN_READY.load(Ordering::Acquire) {
        return None;
    }
    let virtual_base = HEAP_VIRTUAL_BASE.load(Ordering::Relaxed);
    let len = HEAP_RUNTIME_SIZE.load(Ordering::Relaxed);
    let translated = memory::physical_span_for_virtual_range(virtual_base as *const u8, len)?;
    if HEAP_EXPECTED_PHYSICAL_READY.load(Ordering::Relaxed)
        && translated.start() != HEAP_EXPECTED_PHYSICAL_BASE.load(Ordering::Relaxed)
    {
        return None;
    }
    Some(translated)
}

fn init_static_fallback(usable_entries: u64, largest_usable: u64, reason: HeapReason) {
    unsafe {
        let heap_start = ptr::addr_of_mut!(HEAP.0).cast::<u8>();
        ALLOCATOR.lock().init(heap_start, HEAP_SIZE);
        store_report(HeapReport {
            source: HeapSource::StaticFallback,
            size: HEAP_SIZE as u64,
            base: heap_start as u64,
            usable_entries,
            largest_usable,
            reason,
        });
    }
}

fn publish_heap_span(virtual_base: u64, len: u64, expected_physical_base: Option<u64>) {
    HEAP_SPAN_READY.store(false, Ordering::Release);
    HEAP_VIRTUAL_BASE.store(virtual_base, Ordering::Relaxed);
    HEAP_RUNTIME_SIZE.store(len, Ordering::Relaxed);
    match expected_physical_base {
        Some(physical_base) => {
            HEAP_EXPECTED_PHYSICAL_BASE.store(physical_base, Ordering::Relaxed);
            HEAP_EXPECTED_PHYSICAL_READY.store(true, Ordering::Relaxed);
        }
        None => {
            HEAP_EXPECTED_PHYSICAL_BASE.store(0, Ordering::Relaxed);
            HEAP_EXPECTED_PHYSICAL_READY.store(false, Ordering::Relaxed);
        }
    }
    HEAP_SPAN_READY.store(true, Ordering::Release);
}

fn store_report(report: HeapReport) {
    unsafe {
        ptr::write(ptr::addr_of_mut!(HEAP_REPORT), report);
    }
    if matches!(report.source, HeapSource::StaticFallback) {
        publish_heap_span(report.base, report.size, None);
    }
}

fn select_largest_region<I>(entries: I) -> Option<Region>
where
    I: IntoIterator<Item = (u64, u64)>,
{
    let mut largest: Option<Region> = None;
    for (base, length) in entries {
        let Some(candidate) = aligned_region(base, length) else {
            continue;
        };
        let replace = match largest {
            Some(current) => {
                candidate.length > current.length
                    || (candidate.length == current.length && candidate.base < current.base)
            }
            None => true,
        };
        if replace {
            largest = Some(candidate);
        }
    }
    largest
}

fn aligned_region(base: u64, length: u64) -> Option<Region> {
    base.checked_add(length)?;
    let remainder = base % PAGE_SIZE;
    let padding = if remainder == 0 {
        0
    } else {
        PAGE_SIZE.checked_sub(remainder)?
    };
    Some(Region {
        base: base.checked_add(padding)?,
        length: length.checked_sub(padding)?,
    })
}

#[cfg(test)]
mod tests {
    use super::{select_heap_region, MAX_HEAP_SIZE, MIB, MIN_HEAP_SIZE, PAGE_SIZE};

    #[test]
    fn empty_map_has_no_heap_region() {
        assert_eq!(select_heap_region(&[]), None);
    }

    #[test]
    fn regions_below_minimum_are_rejected() {
        assert_eq!(
            select_heap_region(&[(PAGE_SIZE, MIN_HEAP_SIZE - 1), (0x20_0000, 8 * 1024 * 1024)]),
            None
        );
    }

    #[test]
    fn exact_minimum_is_accepted() {
        assert_eq!(
            select_heap_region(&[(0x10_0000, MIN_HEAP_SIZE)]),
            Some((0x10_0000, MIN_HEAP_SIZE))
        );
    }

    #[test]
    fn region_above_maximum_is_capped() {
        assert_eq!(
            select_heap_region(&[(0x10_0000, MAX_HEAP_SIZE + MIB)]),
            Some((0x10_0000, 4096 * MIB))
        );
    }

    #[test]
    fn region_between_old_and_new_cap_is_used_fully() {
        let region_size = 3584 * MIB;
        assert_eq!(
            select_heap_region(&[(0x10_0000, region_size)]),
            Some((0x10_0000, region_size))
        );
    }

    #[test]
    fn region_within_old_cap_keeps_its_exact_size() {
        let region_size = 512 * MIB;
        assert_eq!(
            select_heap_region(&[(0x10_0000, region_size)]),
            Some((0x10_0000, region_size))
        );
    }

    #[test]
    fn equal_regions_choose_lowest_aligned_base() {
        assert_eq!(
            select_heap_region(&[
                (0x90_0000, 2 * MIN_HEAP_SIZE),
                (0x10_0000, 2 * MIN_HEAP_SIZE),
            ]),
            Some((0x10_0000, 2 * MIN_HEAP_SIZE))
        );
    }

    #[test]
    fn unaligned_base_is_rounded_up_and_length_shortened() {
        assert_eq!(
            select_heap_region(&[(0x10_0001, MIN_HEAP_SIZE + PAGE_SIZE)]),
            Some((0x10_1000, MIN_HEAP_SIZE + 1))
        );
    }
}
