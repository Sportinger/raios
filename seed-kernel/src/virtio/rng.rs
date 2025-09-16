use core::cmp;
use core::hint::spin_loop;
use core::mem::size_of;
use core::ptr;
use core::sync::atomic::{fence, Ordering};

use spin::Mutex;

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

const DESC_TABLE_BYTES: usize = size_of::<[VirtqDesc; QUEUE_CAPACITY]>();
const AVAIL_BYTES: usize = size_of::<VirtqAvail>();
const USED_BYTES: usize = size_of::<VirtqUsed>();
const USED_OFFSET: usize = align_up(DESC_TABLE_BYTES + AVAIL_BYTES, QUEUE_ALIGN);
const QUEUE_BYTES: usize = USED_OFFSET + USED_BYTES;

#[repr(align(4096))]
struct QueueStorage([u8; QUEUE_BYTES]);

static mut QUEUE_STORAGE: QueueStorage = QueueStorage([0; QUEUE_BYTES]);
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

        enable_bus_master(address);

        unsafe {
            write_status(io_base, 0);
            write_status(io_base, STATUS_ACKNOWLEDGE);
            write_status(io_base, STATUS_ACKNOWLEDGE | STATUS_DRIVER);
            let host_features = read_device_features(io_base);
            if host_features != 0 {
                serial::write_fmt(format_args!("virtio-rng host features 0x{:08x} (ignored)\r\n", host_features));
            }
            write_driver_features(io_base, 0);
            write_status(io_base, STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_DRIVER_OK);
            write_queue_select(io_base, LEGACY_QUEUE_INDEX);
        }

        let raw_queue_size = unsafe { read_queue_size(io_base) };
        if raw_queue_size == 0 {
            serial::write_line("virtio-rng queue size reported as zero");
            return None;
        }
        let queue_size = cmp::min(raw_queue_size, QUEUE_CAPACITY as u16);

        unsafe {
            zero_queue_storage();
            write_queue_address(io_base, queue_storage_phys() >> 12);
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
        })
    }

    pub fn fill_bytes(&mut self, buffer: &mut [u8]) -> usize {
        if buffer.is_empty() {
            return 0;
        }

        let _guard = QUEUE_LOCK.lock();
        let (descs, avail, used) = unsafe { queue_parts() };

        descs[0].addr = buffer.as_mut_ptr() as u64;
        descs[0].len = buffer.len() as u32;
        descs[0].flags = VIRTQ_DESC_F_WRITE;
        descs[0].next = 0;

        let slot = (avail.idx % self.queue_size) as usize;
        avail.ring[slot] = 0;
        fence(Ordering::Release);
        avail.idx = avail.idx.wrapping_add(1);

        unsafe {
            write_queue_notify(self.io_base, LEGACY_QUEUE_INDEX);
        }

        loop {
            fence(Ordering::Acquire);
            if used.idx != self.last_used_idx {
                let used_slot = (self.last_used_idx % self.queue_size) as usize;
                let elem = used.ring[used_slot];
                self.last_used_idx = self.last_used_idx.wrapping_add(1);
                let _ = unsafe { read_isr_status(self.io_base) };
                return cmp::min(elem.len as usize, buffer.len());
            }
            spin_loop();
        }
    }
}

fn enable_bus_master(address: PciAddress) {
    let mut command = (address.read_u32(0x04) & 0xFFFF) as u16;
    command |= 0x1 | 0x2 | 0x4; // I/O space, memory space, bus master
    address.write_u16(0x04, command);
}

#[allow(static_mut_refs)]
unsafe fn queue_parts() -> (&'static mut [VirtqDesc; QUEUE_CAPACITY], &'static mut VirtqAvail, &'static mut VirtqUsed) {
    let base = QUEUE_STORAGE.0.as_mut_ptr();
    let desc = &mut *(base.cast::<[VirtqDesc; QUEUE_CAPACITY]>());
    let avail_ptr = base.add(DESC_TABLE_BYTES).cast::<VirtqAvail>();
    let avail = &mut *avail_ptr;
    let used_ptr = base.add(USED_OFFSET).cast::<VirtqUsed>();
    let used = &mut *used_ptr;
    (desc, avail, used)
}

#[allow(static_mut_refs)]
unsafe fn zero_queue_storage() {
    ptr::write_bytes(QUEUE_STORAGE.0.as_mut_ptr(), 0, QUEUE_BYTES);
}

#[allow(static_mut_refs)]
unsafe fn queue_storage_phys() -> u64 {
    // Kernel is identity mapped at boot; revisit when higher-half move happens.
    QUEUE_STORAGE.0.as_ptr() as u64
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
