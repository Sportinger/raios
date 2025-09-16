use core::{mem, ptr};

use crate::pci::{self, PciAddress};
use crate::serial;

use super::{device_id, VIRTIO_VENDOR_ID};

const NET_REG_DEVICE_FEATURES: u16 = 0x00;
const NET_REG_DRIVER_FEATURES: u16 = 0x04;
#[allow(dead_code)]
const NET_REG_QUEUE_ADDRESS: u16 = 0x08;
#[allow(dead_code)]
const NET_REG_QUEUE_SIZE: u16 = 0x0C;
#[allow(dead_code)]
const NET_REG_QUEUE_SELECT: u16 = 0x0E;
#[allow(dead_code)]
const NET_REG_QUEUE_NOTIFY: u16 = 0x10;
const NET_REG_DEVICE_STATUS: u16 = 0x12;
#[allow(dead_code)]
const NET_REG_ISR_STATUS: u16 = 0x13;
const NET_REG_MAC_LOW: u16 = 0x14;
const NET_REG_MAC_HIGH: u16 = 0x18;
const _NET_REG_UNUSED: u16 = 0;
#[derive(Debug, Clone, Copy)]
pub enum VirtioNetKind {
    Legacy,
    Modern,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioNet {
    pub address: PciAddress,
    pub kind: VirtioNetKind,
}

impl VirtioNet {
    pub fn configure(&self) {
        match self.kind {
            VirtioNetKind::Legacy => configure_legacy(self.address),
            VirtioNetKind::Modern => serial::write_line("virtio-net modern transport unsupported"),
        }
    }
}

fn configure_legacy(address: PciAddress) {
    let bar0 = address.read_u32(0x10);
    if bar0 & 0x1 == 0 {
        serial::write_line("virtio-net legacy device missing I/O BAR");
        return;
    }

    let io_base = (bar0 & 0xFFFC) as u16;
    pci::enable_bus_master(address);

    reset_device(io_base);
    setup_legacy_queue(io_base);

    let mac_low = read_io_u32(io_base, NET_REG_MAC_LOW) as u64;
    let mac_high = read_io_u32(io_base, NET_REG_MAC_HIGH) as u64;
    let mac = [
        (mac_low & 0xFF) as u8,
        ((mac_low >> 8) & 0xFF) as u8,
        ((mac_low >> 16) & 0xFF) as u8,
        ((mac_low >> 24) & 0xFF) as u8,
        (mac_high & 0xFF) as u8,
        ((mac_high >> 8) & 0xFF) as u8,
    ];

    serial::write_fmt(format_args!(
        "virtio-net legacy transport @ 0x{:x}, mac {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}\r\n",
        io_base, mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    ));
}

pub fn probe() -> Option<VirtioNet> {
    if let Some(addr) = pci::find_device(VIRTIO_VENDOR_ID, device_id::LEGACY_NET) {
        serial::write_fmt(format_args!("virtio-net (legacy) @ {} detected\r\n", addr));
        return Some(VirtioNet {
            address: addr,
            kind: VirtioNetKind::Legacy,
        });
    }

    if let Some(addr) = pci::find_device(VIRTIO_VENDOR_ID, device_id::MODERN_NET) {
        serial::write_fmt(format_args!(
            "virtio-net (modern) @ {} detected but modern transport not supported yet\r\n",
            addr
        ));
        return Some(VirtioNet {
            address: addr,
            kind: VirtioNetKind::Modern,
        });
    }

    serial::write_line("virtio-net device not present");
    None
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

fn read_io_u32(base: u16, offset: u16) -> u32 {
    unsafe { inl(base + offset) }
}

fn write_status(base: u16, value: u8) {
    unsafe { outb(base + NET_REG_DEVICE_STATUS, value) };
}

fn read_status(base: u16) -> u8 {
    unsafe { inb(base + NET_REG_DEVICE_STATUS) }
}

fn write_driver_features(base: u16, value: u32) {
    unsafe { outl(base + NET_REG_DRIVER_FEATURES, value) };
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
#[allow(dead_code)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
    ring: [u16; QUEUE_CAPACITY],
    used_event: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
#[allow(dead_code)]
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

#[allow(dead_code)]
const QUEUE_CAPACITY: usize = 256;

#[repr(align(4096))]
#[allow(dead_code)]
struct QueueStorage {
    desc: [VirtqDesc; QUEUE_CAPACITY],
    avail: VirtqAvail,
    used_padding: [u8; 4096],
    used: VirtqUsed,
}

#[allow(dead_code)]
static mut NET_QUEUE_STORAGE: QueueStorage = QueueStorage {
    desc: [VirtqDesc {
        addr: 0,
        len: 0,
        flags: 0,
        next: 0,
    }; QUEUE_CAPACITY],
    avail: VirtqAvail {
        flags: 0,
        idx: 0,
        ring: [0; QUEUE_CAPACITY],
        used_event: 0,
    },
    used_padding: [0; 4096],
    used: VirtqUsed {
        flags: 0,
        idx: 0,
        ring: [VirtqUsedElem { id: 0, len: 0 }; QUEUE_CAPACITY],
        avail_event: 0,
    },
};

fn reset_device(base: u16) {
    // Follow virtio legacy initialization: ACKNOWLEDGE -> DRIVER -> FEATURES_OK -> DRIVER_OK
    write_status(base, 0);
    write_status(base, 0x01); // ACKNOWLEDGE
    write_status(base, 0x01 | 0x02); // DRIVER

    let host_features = read_io_u32(base, NET_REG_DEVICE_FEATURES);
    serial::write_fmt(format_args!("virtio-net host features 0x{:08x}
", host_features));

    // For now, accept no optional features.
    write_driver_features(base, 0);
    write_status(base, 0x01 | 0x02 | 0x08); // FEATURES_OK

    let status = read_status(base);
    if status & 0x08 == 0 {
        serial::write_line("virtio-net failed to accept features");
    }

    write_status(base, 0x01 | 0x02 | 0x08 | 0x04); // DRIVER_OK
}

fn setup_legacy_queue(base: u16) {
    unsafe {
        let storage_ptr = ptr::addr_of_mut!(NET_QUEUE_STORAGE);
        ptr::write_bytes(storage_ptr.cast::<u8>(), 0, mem::size_of::<QueueStorage>());
    }

    write_queue_select(base, 0);
    let mut queue_size = read_queue_size(base);
    if queue_size == 0 {
        serial::write_line("virtio-net queue size reported as zero");
        return;
    }
    if queue_size as usize > QUEUE_CAPACITY {
        serial::write_fmt(format_args!(
            "virtio-net queue size {} exceeds storage capacity {}; truncating\r\n",
            queue_size,
            QUEUE_CAPACITY
        ));
        queue_size = QUEUE_CAPACITY as u16;
    }

    let queue_addr = ptr::addr_of!(NET_QUEUE_STORAGE) as u64;
    let pfn = (queue_addr >> 12) as u32;
    write_queue_address(base, pfn);
    serial::write_fmt(format_args!(
        "virtio-net queue configured size={} pfn=0x{:x}\r\n",
        queue_size, pfn
    ));
}

fn write_queue_select(base: u16, queue: u16) {
    unsafe { outw(base + NET_REG_QUEUE_SELECT, queue) };
}

fn read_queue_size(base: u16) -> u16 {
    unsafe { inw(base + NET_REG_QUEUE_SIZE) }
}

fn write_queue_address(base: u16, pfn: u32) {
    unsafe { outl(base + NET_REG_QUEUE_ADDRESS, pfn) };
}
