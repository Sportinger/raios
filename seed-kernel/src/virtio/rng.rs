use core::cmp;
use core::hint::spin_loop;
use core::mem::size_of;
use core::ptr;
use core::sync::atomic::{fence, Ordering};

use spin::Mutex;

use crate::memory;
use crate::pci::{self, PciAddress};
use crate::serial;

use super::{device_id, VIRTIO_VENDOR_ID};

const REG_DEVICE_FEATURES: u16 = 0x00;
const REG_DRIVER_FEATURES: u16 = 0x04;
const REG_QUEUE_ADDRESS: u16 = 0x08;
const REG_QUEUE_SIZE: u16 = 0x0C;
const REG_QUEUE_SELECT: u16 = 0x0E;
const REG_QUEUE_NOTIFY: u16 = 0x10;
const REG_DEVICE_STATUS: u16 = 0x12;
const REG_ISR_STATUS: u16 = 0x13;

const STATUS_ACKNOWLEDGE: u8 = 0x01;
const STATUS_DRIVER: u8 = 0x02;
const STATUS_DRIVER_OK: u8 = 0x04;

const VIRTQ_DESC_F_WRITE: u16 = 1 << 1;

const LEGACY_QUEUE_INDEX: u16 = 0;
const QUEUE_CAPACITY: usize = 64;
const QUEUE_ALIGN: usize = 4096;
const DMA_BUFFER_LEN: usize = 256;
const FILL_SPIN_LIMIT: usize = 1_000_000;

#[derive(Debug, Clone, Copy)]
pub enum VirtioRngKind {
    Legacy,
}

#[allow(dead_code)]
pub struct VirtioRng {
    pub address: PciAddress,
    pub kind: VirtioRngKind,
    io_base: u16,
    queue_size: u16,
    last_used_idx: u16,
    timed_out: bool,
}

const fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

#[repr(C)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
    ring: [u16; QUEUE_CAPACITY],
    used_event: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

#[repr(C)]
struct VirtqUsed {
    flags: u16,
    idx: u16,
    ring: [VirtqUsedElem; QUEUE_CAPACITY],
    avail_event: u16,
}

const USED_BYTES: usize = size_of::<VirtqUsed>();
const MAX_DESC_TABLE_BYTES: usize = size_of::<[VirtqDesc; QUEUE_CAPACITY]>();
const MAX_AVAIL_BYTES: usize = size_of::<VirtqAvail>();
const USED_OFFSET: usize = align_up(MAX_DESC_TABLE_BYTES + MAX_AVAIL_BYTES, QUEUE_ALIGN);
const QUEUE_BYTES: usize = USED_OFFSET + USED_BYTES;

#[repr(align(4096))]
struct QueueStorage([u8; QUEUE_BYTES]);

static mut QUEUE_STORAGE: QueueStorage = QueueStorage([0; QUEUE_BYTES]);
static mut DMA_BUFFER: [u8; DMA_BUFFER_LEN] = [0; DMA_BUFFER_LEN];
static QUEUE_LOCK: Mutex<()> = Mutex::new(());

pub fn probe() -> Option<VirtioRng> {
    if let Some(addr) = pci::find_device(VIRTIO_VENDOR_ID, device_id::LEGACY_RNG) {
        serial::write_fmt(format_args!("virtio-rng (legacy) @ {} detected\r\n", addr));
        return VirtioRng::from_legacy(addr);
    }

    if let Some(addr) = pci::find_device(VIRTIO_VENDOR_ID, device_id::MODERN_RNG) {
        serial::write_fmt(format_args!(
            "virtio-rng (modern) @ {} detected but modern transport not supported yet\r\n",
            addr
        ));
        return None;
    }

    serial::write_line("virtio-rng device not present");
    None
}

impl VirtioRng {
    fn from_legacy(address: PciAddress) -> Option<Self> {
        let bar0 = address.read_u32(0x10);
        if bar0 & 0x1 == 0 {
            serial::write_line("virtio-rng legacy device missing I/O BAR");
            return None;
        }
        let io_base = (bar0 & 0xFFFC) as u16;

        pci::enable_bus_master(address);

        unsafe {
            write_status(io_base, 0);
            write_status(io_base, STATUS_ACKNOWLEDGE);
            write_status(io_base, STATUS_ACKNOWLEDGE | STATUS_DRIVER);
            let host_features = read_device_features(io_base);
            if host_features != 0 {
                serial::write_fmt(format_args!(
                    "virtio-rng host features 0x{:08x} (ignored)\r\n",
                    host_features
                ));
            }
            write_driver_features(io_base, 0);
            write_queue_select(io_base, LEGACY_QUEUE_INDEX);
        }

        let raw_queue_size = unsafe { read_queue_size(io_base) };
        if raw_queue_size == 0 {
            serial::write_line("virtio-rng queue size reported as zero");
            return None;
        }
        let queue_size = cmp::min(raw_queue_size, QUEUE_CAPACITY as u16);

        let queue_phys = unsafe { queue_storage_phys() }?;
        let queue_pfn = legacy_queue_pfn(queue_phys, "virtio-rng queue")?;

        unsafe {
            zero_queue_storage();
            write_queue_address(io_base, queue_pfn);
            write_status(
                io_base,
                STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_DRIVER_OK,
            );
        }

        serial::write_fmt(format_args!(
            "virtio-rng queue configured (size={}, io_base=0x{:x})\r\n",
            queue_size, io_base
        ));

        Some(Self {
            address,
            kind: VirtioRngKind::Legacy,
            io_base,
            queue_size,
            last_used_idx: 0,
            timed_out: false,
        })
    }

    pub fn fill_bytes(&mut self, buffer: &mut [u8]) -> usize {
        if buffer.is_empty() || self.timed_out {
            return 0;
        }

        let _guard = QUEUE_LOCK.lock();
        let (descs, avail, used) = unsafe { queue_parts(self.queue_size) };
        let request_len = cmp::min(buffer.len(), DMA_BUFFER_LEN);
        let Some(dma_addr) = dma_buffer_phys() else {
            self.timed_out = true;
            serial::write_line("virtio-rng DMA buffer address is not translatable");
            return 0;
        };

        unsafe {
            ptr::write_bytes(dma_buffer_ptr(), 0, request_len);
        }

        descs[0].addr = dma_addr;
        descs[0].len = request_len as u32;
        descs[0].flags = VIRTQ_DESC_F_WRITE;
        descs[0].next = 0;

        let slot = (avail.idx % self.queue_size) as usize;
        avail.ring[slot] = 0;
        fence(Ordering::Release);
        avail.idx = avail.idx.wrapping_add(1);

        unsafe {
            write_queue_notify(self.io_base, LEGACY_QUEUE_INDEX);
        }

        let mut spins = 0usize;
        loop {
            fence(Ordering::Acquire);
            if used.idx != self.last_used_idx {
                let used_slot = (self.last_used_idx % self.queue_size) as usize;
                let elem = used.ring[used_slot];
                self.last_used_idx = self.last_used_idx.wrapping_add(1);
                let _ = unsafe { read_isr_status(self.io_base) };
                let actual = cmp::min(elem.len as usize, request_len);
                unsafe {
                    ptr::copy_nonoverlapping(dma_buffer_ptr(), buffer.as_mut_ptr(), actual);
                }
                return actual;
            }
            if spins >= FILL_SPIN_LIMIT {
                self.timed_out = true;
                serial::write_line("virtio-rng request timed out; entropy source disabled");
                return 0;
            }
            spins += 1;
            spin_loop();
        }
    }
}

#[allow(static_mut_refs)]
unsafe fn queue_parts(
    queue_size: u16,
) -> (
    &'static mut [VirtqDesc; QUEUE_CAPACITY],
    &'static mut VirtqAvail,
    &'static mut VirtqUsed,
) {
    let base = QUEUE_STORAGE.0.as_mut_ptr();
    let desc = &mut *(base.cast::<[VirtqDesc; QUEUE_CAPACITY]>());
    let avail_ptr = base.add(desc_table_bytes(queue_size)).cast::<VirtqAvail>();
    let avail = &mut *avail_ptr;
    let used_ptr = base.add(used_offset(queue_size)).cast::<VirtqUsed>();
    let used = &mut *used_ptr;
    (desc, avail, used)
}

const fn desc_table_bytes(queue_size: u16) -> usize {
    size_of::<VirtqDesc>() * queue_size as usize
}

const fn avail_bytes(queue_size: u16) -> usize {
    2 + 2 + (2 * queue_size as usize) + 2
}

const fn used_offset(queue_size: u16) -> usize {
    align_up(
        desc_table_bytes(queue_size) + avail_bytes(queue_size),
        QUEUE_ALIGN,
    )
}

#[allow(static_mut_refs)]
unsafe fn zero_queue_storage() {
    ptr::write_bytes(QUEUE_STORAGE.0.as_mut_ptr(), 0, QUEUE_BYTES);
}

#[allow(static_mut_refs)]
unsafe fn queue_storage_phys() -> Option<u64> {
    memory::virt_to_phys(QUEUE_STORAGE.0.as_ptr())
}

#[allow(static_mut_refs)]
fn dma_buffer_phys() -> Option<u64> {
    memory::virt_to_phys(ptr::addr_of!(DMA_BUFFER).cast::<u8>())
}

unsafe fn dma_buffer_ptr() -> *mut u8 {
    ptr::addr_of_mut!(DMA_BUFFER).cast::<u8>()
}

fn legacy_queue_pfn(phys: u64, label: &'static str) -> Option<u64> {
    let pfn = phys >> 12;
    if pfn > u32::MAX as u64 {
        serial::write_fmt(format_args!(
            "{} physical address 0x{:x} is above legacy PFN range\r\n",
            label, phys
        ));
        return None;
    }
    Some(pfn)
}

unsafe fn write_status(io_base: u16, value: u8) {
    outb(io_base + REG_DEVICE_STATUS, value);
}

unsafe fn read_device_features(io_base: u16) -> u32 {
    inl(io_base + REG_DEVICE_FEATURES)
}

unsafe fn write_driver_features(io_base: u16, value: u32) {
    outl(io_base + REG_DRIVER_FEATURES, value);
}

unsafe fn write_queue_select(io_base: u16, queue: u16) {
    outw(io_base + REG_QUEUE_SELECT, queue);
}

unsafe fn read_queue_size(io_base: u16) -> u16 {
    inw(io_base + REG_QUEUE_SIZE)
}

unsafe fn write_queue_address(io_base: u16, pfn: u64) {
    outl(io_base + REG_QUEUE_ADDRESS, pfn as u32);
}

unsafe fn write_queue_notify(io_base: u16, queue: u16) {
    outw(io_base + REG_QUEUE_NOTIFY, queue);
}

unsafe fn read_isr_status(io_base: u16) -> u8 {
    inb(io_base + REG_ISR_STATUS)
}

unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, preserves_flags));
}

unsafe fn outw(port: u16, value: u16) {
    core::arch::asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, preserves_flags));
}

unsafe fn outl(port: u16, value: u32) {
    core::arch::asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, preserves_flags));
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", in("dx") port, out("al") value, options(nomem, preserves_flags));
    value
}

unsafe fn inw(port: u16) -> u16 {
    let value: u16;
    core::arch::asm!("in ax, dx", in("dx") port, out("ax") value, options(nomem, preserves_flags));
    value
}

unsafe fn inl(port: u16) -> u32 {
    let value: u32;
    core::arch::asm!("in eax, dx", in("dx") port, out("eax") value, options(nomem, preserves_flags));
    value
}
