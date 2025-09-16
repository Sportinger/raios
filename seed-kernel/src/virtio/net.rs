#![allow(static_mut_refs)]

extern crate alloc;

use alloc::collections::VecDeque;

use core::cmp;
use core::mem;
use core::ptr;
use core::sync::atomic::{fence, AtomicU16, Ordering};

use spin::Mutex;

use crate::pci::{self, PciAddress};
use crate::serial;

use super::{device_id, VIRTIO_VENDOR_ID};

const NET_REG_DEVICE_FEATURES: u16 = 0x00;
const NET_REG_DRIVER_FEATURES: u16 = 0x04;
const NET_REG_QUEUE_ADDRESS: u16 = 0x08;
const NET_REG_QUEUE_SIZE: u16 = 0x0C;
const NET_REG_QUEUE_SELECT: u16 = 0x0E;
const NET_REG_QUEUE_NOTIFY: u16 = 0x10;
const NET_REG_DEVICE_STATUS: u16 = 0x12;
const NET_REG_ISR_STATUS: u16 = 0x13;
const NET_REG_MAC_LOW: u16 = 0x14;
const NET_REG_MAC_HIGH: u16 = 0x18;

const STATUS_ACKNOWLEDGE: u8 = 0x01;
const STATUS_DRIVER: u8 = 0x02;
const STATUS_DRIVER_OK: u8 = 0x04;
const STATUS_FEATURES_OK: u8 = 0x08;

const VIRTIO_NET_F_MAC: u32 = 1 << 5;
const VIRTIO_NET_F_STATUS: u32 = 1 << 16;

const RX_QUEUE_INDEX: u16 = 0;
const TX_QUEUE_INDEX: u16 = 1;

const RX_BUFFER_COUNT: usize = 32;
const RX_BUFFER_LEN: usize = 2048;
const TX_BUFFER_COUNT: usize = 16;
const TX_BUFFER_LEN: usize = 2048;

pub const MAX_FRAME_SIZE: usize = TX_BUFFER_LEN;
const VIRTQ_DESC_F_WRITE: u16 = 1 << 1;

const QUEUE_CAPACITY: usize = 256;

#[derive(Clone, Copy, Debug)]
pub enum VirtioNetKind {
    Legacy,
    Modern,
}

#[derive(Clone, Copy, Debug)]
pub struct VirtioNet {
    pub address: PciAddress,
    pub kind: VirtioNetKind,
    pub mac: [u8; 6],
    pub rx_queue_size: u16,
    pub tx_queue_size: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct RxPacket {
    pub desc_idx: u16,
    pub len: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct TxPacket {
    pub desc_idx: u16,
}

struct TxState {
    free: [bool; TX_BUFFER_COUNT],
    usable: usize,
}

impl TxState {
    const fn new() -> Self {
        Self {
            free: [true; TX_BUFFER_COUNT],
            usable: 0,
        }
    }

    fn reset(&mut self, usable: usize) {
        let capped = cmp::min(usable, TX_BUFFER_COUNT);
        self.usable = capped;
        let mut i = 0;
        while i < TX_BUFFER_COUNT {
            self.free[i] = i < capped;
            i += 1;
        }
    }

    fn acquire(&mut self) -> Option<u16> {
        for i in 0..self.usable {
            if self.free[i] {
                self.free[i] = false;
                return Some(i as u16);
            }
        }
        None
    }

    fn release(&mut self, idx: u16) {
        let index = idx as usize;
        if index < self.usable {
            self.free[index] = true;
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
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

#[repr(align(4096))]
struct QueueStorage {
    desc: [VirtqDesc; QUEUE_CAPACITY],
    avail: VirtqAvail,
    used_padding: [u8; 4096],
    used: VirtqUsed,
}

static mut RX_QUEUE_STORAGE: QueueStorage = QueueStorage {
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

static mut TX_QUEUE_STORAGE: QueueStorage = QueueStorage {
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

static mut RX_BUFFERS: [[u8; RX_BUFFER_LEN]; RX_BUFFER_COUNT] =
    [[0; RX_BUFFER_LEN]; RX_BUFFER_COUNT];
static mut TX_BUFFERS: [[u8; TX_BUFFER_LEN]; TX_BUFFER_COUNT] =
    [[0; TX_BUFFER_LEN]; TX_BUFFER_COUNT];

static LEGACY_IO_BASE: AtomicU16 = AtomicU16::new(0);
static RX_QUEUE_LAST_USED: AtomicU16 = AtomicU16::new(0);
static TX_QUEUE_LAST_USED: AtomicU16 = AtomicU16::new(0);
static RX_QUEUE_SIZE: AtomicU16 = AtomicU16::new(0);
static TX_QUEUE_SIZE: AtomicU16 = AtomicU16::new(0);

static RX_READY: Mutex<VecDeque<RxPacket>> = Mutex::new(VecDeque::new());
static TX_STATE: Mutex<TxState> = Mutex::new(TxState::new());
static NET_INFO: Mutex<Option<VirtioNet>> = Mutex::new(None);

pub fn probe() -> Option<VirtioNet> {
    if let Some(addr) = pci::find_device(VIRTIO_VENDOR_ID, device_id::LEGACY_NET) {
        serial::write_fmt(format_args!("virtio-net (legacy) @ {} detected\r\n", addr));
        return configure_legacy(addr);
    }

    if let Some(addr) = pci::find_device(VIRTIO_VENDOR_ID, device_id::MODERN_NET) {
        serial::write_fmt(format_args!(
            "virtio-net (modern) @ {} detected but modern transport not supported yet\r\n",
            addr
        ));
        return None;
    }

    serial::write_line("virtio-net device not present");
    None
}

pub fn info() -> Option<VirtioNet> {
    *NET_INFO.lock()
}

pub fn poll() {
    let base = LEGACY_IO_BASE.load(Ordering::Relaxed);
    if base == 0 {
        return;
    }
    poll_rx(base);
    process_tx_used(base);
}

pub fn pop_rx_packet() -> Option<RxPacket> {
    let mut queue = RX_READY.lock();
    queue.pop_front()
}

pub fn rx_packet_buffer(packet: &RxPacket) -> &'static mut [u8] {
    unsafe { &mut RX_BUFFERS[packet.desc_idx as usize][..packet.len] }
}

pub fn recycle_rx_packet(packet: RxPacket) {
    let base = LEGACY_IO_BASE.load(Ordering::Relaxed);
    if base == 0 {
        return;
    }
    recycle_rx_descriptor(base, packet.desc_idx);
}

pub fn alloc_tx_packet() -> Option<(TxPacket, &'static mut [u8])> {
    let base = LEGACY_IO_BASE.load(Ordering::Relaxed);
    if base == 0 {
        return None;
    }
    process_tx_used(base);
    let desc_idx = {
        let mut state = TX_STATE.lock();
        state.acquire()?
    };
    let buffer = unsafe { &mut TX_BUFFERS[desc_idx as usize] };
    Some((TxPacket { desc_idx }, &mut buffer[..]))
}

pub fn submit_tx_packet(packet: TxPacket, len: usize) -> bool {
    let base = LEGACY_IO_BASE.load(Ordering::Relaxed);
    if base == 0 {
        return false;
    }
    if len > TX_BUFFER_LEN {
        serial::write_line("virtio-net TX frame larger than buffer");
        return false;
    }

    unsafe {
        let desc = &mut TX_QUEUE_STORAGE.desc[packet.desc_idx as usize];
        desc.len = len as u32;
        desc.flags = 0;
        desc.next = 0;
        let avail = &mut TX_QUEUE_STORAGE.avail;
        let slot = (avail.idx % QUEUE_CAPACITY as u16) as usize;
        avail.ring[slot] = packet.desc_idx;
        fence(Ordering::Release);
        avail.idx = avail.idx.wrapping_add(1);
    }

    notify_queue(base, TX_QUEUE_INDEX);
    true
}

pub fn release_tx_packet(packet: TxPacket) {
    let base = LEGACY_IO_BASE.load(Ordering::Relaxed);
    {
        let mut state = TX_STATE.lock();
        state.release(packet.desc_idx);
    }
    if base != 0 {
        unsafe {
            TX_QUEUE_STORAGE.desc[packet.desc_idx as usize].len = 0;
        }
    }
}

fn configure_legacy(address: PciAddress) -> Option<VirtioNet> {
    let bar0 = address.read_u32(0x10);
    if bar0 & 0x1 == 0 {
        serial::write_line("virtio-net legacy device missing I/O BAR");
        return None;
    }

    let io_base = (bar0 & 0xFFFC) as u16;
    pci::enable_bus_master(address);

    unsafe {
        write_status(io_base, 0);
        write_status(io_base, STATUS_ACKNOWLEDGE);
        write_status(io_base, STATUS_ACKNOWLEDGE | STATUS_DRIVER);
    }

    let host_features = read_io_u32(io_base, NET_REG_DEVICE_FEATURES);
    serial::write_fmt(format_args!(
        "virtio-net host features 0x{:08x}\r\n",
        host_features
    ));

    let desired_features = host_features & (VIRTIO_NET_F_MAC | VIRTIO_NET_F_STATUS);
    unsafe {
        write_driver_features(io_base, desired_features);
        write_status(
            io_base,
            STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_FEATURES_OK,
        );
    }

    let status = read_status(io_base);
    if status & STATUS_FEATURES_OK == 0 {
        serial::write_line("virtio-net failed to accept negotiated features");
        return None;
    }

    let rx_queue_size = unsafe { setup_rx_queue(io_base)? };
    let tx_queue_size = unsafe { setup_tx_queue(io_base)? };

    unsafe {
        write_status(
            io_base,
            STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_FEATURES_OK | STATUS_DRIVER_OK,
        );
    }

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

    LEGACY_IO_BASE.store(io_base, Ordering::Relaxed);
    RX_QUEUE_SIZE.store(rx_queue_size, Ordering::Relaxed);
    TX_QUEUE_SIZE.store(tx_queue_size, Ordering::Relaxed);
    RX_QUEUE_LAST_USED.store(0, Ordering::Relaxed);
    TX_QUEUE_LAST_USED.store(0, Ordering::Relaxed);

    {
        let mut state = TX_STATE.lock();
        state.reset(tx_queue_size as usize);
    }

    serial::write_fmt(format_args!(
        "virtio-net legacy transport @ 0x{:x}, mac {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}, rx_q={}, tx_q={}\r\n",
        io_base,
        mac[0],
        mac[1],
        mac[2],
        mac[3],
        mac[4],
        mac[5],
        rx_queue_size,
        tx_queue_size
    ));

    let info = VirtioNet {
        address,
        kind: VirtioNetKind::Legacy,
        mac,
        rx_queue_size,
        tx_queue_size,
    };
    *NET_INFO.lock() = Some(info);
    Some(info)
}

fn poll_rx(base: u16) {
    unsafe {
        let used = &RX_QUEUE_STORAGE.used;
        let mut last = RX_QUEUE_LAST_USED.load(Ordering::Acquire);
        let current = used.idx;
        if last != current {
            let mut queue = RX_READY.lock();
            while last != current {
                let slot = (last % QUEUE_CAPACITY as u16) as usize;
                let elem = used.ring[slot];
                let desc_idx = elem.id as u16;
                let len = elem.len as usize;
                queue.push_back(RxPacket { desc_idx, len });
                last = last.wrapping_add(1);
                let _ = read_isr_status(base);
            }
        }
        RX_QUEUE_LAST_USED.store(last, Ordering::Release);
    }
}

fn process_tx_used(base: u16) {
    unsafe {
        let used = &TX_QUEUE_STORAGE.used;
        let mut last = TX_QUEUE_LAST_USED.load(Ordering::Acquire);
        let current = used.idx;
        while last != current {
            let slot = (last % QUEUE_CAPACITY as u16) as usize;
            let elem = used.ring[slot];
            let desc_idx = elem.id as u16;
            {
                let mut state = TX_STATE.lock();
                state.release(desc_idx);
            }
            TX_QUEUE_STORAGE.desc[desc_idx as usize].len = 0;
            last = last.wrapping_add(1);
            let _ = read_isr_status(base);
        }
        TX_QUEUE_LAST_USED.store(last, Ordering::Release);
    }
}

unsafe fn setup_rx_queue(base: u16) -> Option<u16> {
    write_queue_select(base, RX_QUEUE_INDEX);
    let mut queue_size = read_queue_size(base);
    if queue_size == 0 {
        serial::write_line("virtio-net RX queue reports size zero");
        return None;
    }
    if queue_size as usize > QUEUE_CAPACITY {
        serial::write_fmt(format_args!(
            "virtio-net RX queue size {} exceeds storage capacity {}; truncating\r\n",
            queue_size, QUEUE_CAPACITY
        ));
        queue_size = QUEUE_CAPACITY as u16;
    }

    zero_queue_storage(&mut RX_QUEUE_STORAGE);
    write_queue_address(base, queue_storage_phys(&RX_QUEUE_STORAGE) >> 12);

    let desc = &mut RX_QUEUE_STORAGE.desc;
    let avail = &mut RX_QUEUE_STORAGE.avail;
    avail.idx = 0;
    let buffers = cmp::min(queue_size as usize, RX_BUFFER_COUNT);
    for i in 0..buffers {
        desc[i].addr = RX_BUFFERS[i].as_mut_ptr() as u64;
        desc[i].len = RX_BUFFER_LEN as u32;
        desc[i].flags = VIRTQ_DESC_F_WRITE;
        desc[i].next = 0;
        avail.ring[i] = i as u16;
        avail.idx = avail.idx.wrapping_add(1);
    }
    fence(Ordering::Release);
    notify_queue(base, RX_QUEUE_INDEX);
    Some(queue_size)
}

unsafe fn setup_tx_queue(base: u16) -> Option<u16> {
    write_queue_select(base, TX_QUEUE_INDEX);
    let mut queue_size = read_queue_size(base);
    if queue_size == 0 {
        serial::write_line("virtio-net TX queue reports size zero");
        return None;
    }
    if queue_size as usize > QUEUE_CAPACITY {
        serial::write_fmt(format_args!(
            "virtio-net TX queue size {} exceeds storage capacity {}; truncating\r\n",
            queue_size, QUEUE_CAPACITY
        ));
        queue_size = QUEUE_CAPACITY as u16;
    }

    zero_queue_storage(&mut TX_QUEUE_STORAGE);
    write_queue_address(base, queue_storage_phys(&TX_QUEUE_STORAGE) >> 12);

    let desc = &mut TX_QUEUE_STORAGE.desc;
    let limit = cmp::min(queue_size as usize, TX_BUFFER_COUNT);
    for i in 0..limit {
        desc[i].addr = TX_BUFFERS[i].as_mut_ptr() as u64;
        desc[i].len = 0;
        desc[i].flags = 0;
        desc[i].next = 0;
    }
    Some(queue_size)
}

fn recycle_rx_descriptor(base: u16, desc_idx: u16) {
    unsafe {
        let desc = &mut RX_QUEUE_STORAGE.desc[desc_idx as usize];
        desc.len = RX_BUFFER_LEN as u32;
        desc.flags = VIRTQ_DESC_F_WRITE;
        let avail = &mut RX_QUEUE_STORAGE.avail;
        let slot = (avail.idx % QUEUE_CAPACITY as u16) as usize;
        avail.ring[slot] = desc_idx;
        fence(Ordering::Release);
        avail.idx = avail.idx.wrapping_add(1);
    }
    notify_queue(base, RX_QUEUE_INDEX);
}

fn read_io_u32(base: u16, offset: u16) -> u32 {
    unsafe { inl(base + offset) }
}

unsafe fn write_status(base: u16, value: u8) {
    outb(base + NET_REG_DEVICE_STATUS, value);
}

fn read_status(base: u16) -> u8 {
    unsafe { inb(base + NET_REG_DEVICE_STATUS) }
}

unsafe fn write_driver_features(base: u16, value: u32) {
    outl(base + NET_REG_DRIVER_FEATURES, value);
}

unsafe fn write_queue_select(base: u16, queue: u16) {
    outw(base + NET_REG_QUEUE_SELECT, queue);
}

unsafe fn read_queue_size(base: u16) -> u16 {
    inw(base + NET_REG_QUEUE_SIZE)
}

unsafe fn write_queue_address(base: u16, pfn: u64) {
    outl(base + NET_REG_QUEUE_ADDRESS, pfn as u32);
}

fn notify_queue(base: u16, queue: u16) {
    unsafe { outw(base + NET_REG_QUEUE_NOTIFY, queue) };
}

unsafe fn read_isr_status(base: u16) -> u8 {
    inb(base + NET_REG_ISR_STATUS)
}

unsafe fn zero_queue_storage(storage: &mut QueueStorage) {
    ptr::write_bytes(
        storage as *mut QueueStorage as *mut u8,
        0,
        mem::size_of::<QueueStorage>(),
    );
}

unsafe fn queue_storage_phys(storage: &QueueStorage) -> u64 {
    storage as *const QueueStorage as u64
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
