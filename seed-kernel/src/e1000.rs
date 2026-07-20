#![allow(static_mut_refs)]

use core::cmp;
use core::ptr;
use core::sync::atomic::{compiler_fence, Ordering};

use spin::Mutex;

use crate::memory::{self, MmioMapping};
use crate::pci::{self, PciAddress};
use crate::serial;

const INTEL_VENDOR_ID: u16 = 0x8086;
const E1000_82540EM: u16 = 0x100e;
const E1000_82545EM: u16 = 0x100f;

const REG_CTRL: usize = 0x0000;
const REG_STATUS: usize = 0x0008;
const REG_ICR: usize = 0x00c0;
const REG_IMC: usize = 0x00d8;
const REG_RCTL: usize = 0x0100;
const REG_TCTL: usize = 0x0400;
const REG_RDBAL: usize = 0x2800;
const REG_RDBAH: usize = 0x2804;
const REG_RDLEN: usize = 0x2808;
const REG_RDH: usize = 0x2810;
const REG_RDT: usize = 0x2818;
const REG_TDBAL: usize = 0x3800;
const REG_TDBAH: usize = 0x3804;
const REG_TDLEN: usize = 0x3808;
const REG_TDH: usize = 0x3810;
const REG_TDT: usize = 0x3818;
const REG_MTA: usize = 0x5200;
const REG_RAL: usize = 0x5400;
const REG_RAH: usize = 0x5404;

const CTRL_SLU: u32 = 1 << 6;

const RCTL_EN: u32 = 1 << 1;
const RCTL_BAM: u32 = 1 << 15;
const RCTL_SECRC: u32 = 1 << 26;

const TCTL_EN: u32 = 1 << 1;
const TCTL_PSP: u32 = 1 << 3;
const TCTL_CT_SHIFT: u32 = 4;
const TCTL_COLD_SHIFT: u32 = 12;

const RX_DESC_STATUS_DD: u8 = 1 << 0;
const RX_DESC_STATUS_EOP: u8 = 1 << 1;
const TX_DESC_STATUS_DD: u8 = 1 << 0;
const TX_DESC_CMD_EOP: u8 = 1 << 0;
const TX_DESC_CMD_IFCS: u8 = 1 << 1;
const TX_DESC_CMD_RS: u8 = 1 << 3;

const RX_DESC_COUNT: usize = 32;
const TX_DESC_COUNT: usize = 8;
const RX_BUFFER_SIZE: usize = 2048;
pub const MAX_FRAME_SIZE: usize = 1536;

static DRIVER: Mutex<Option<E1000>> = Mutex::new(None);

#[repr(C)]
#[derive(Clone, Copy)]
struct RxDesc {
    addr: u64,
    length: u16,
    checksum: u16,
    status: u8,
    errors: u8,
    special: u16,
}

impl RxDesc {
    const EMPTY: Self = Self {
        addr: 0,
        length: 0,
        checksum: 0,
        status: 0,
        errors: 0,
        special: 0,
    };
}

#[repr(C)]
#[derive(Clone, Copy)]
struct TxDesc {
    addr: u64,
    length: u16,
    cso: u8,
    cmd: u8,
    status: u8,
    css: u8,
    special: u16,
}

impl TxDesc {
    const EMPTY: Self = Self {
        addr: 0,
        length: 0,
        cso: 0,
        cmd: 0,
        status: TX_DESC_STATUS_DD,
        css: 0,
        special: 0,
    };
}

#[repr(align(16))]
struct RxDescRing([RxDesc; RX_DESC_COUNT]);

#[repr(align(16))]
struct TxDescRing([TxDesc; TX_DESC_COUNT]);

#[repr(align(16))]
#[derive(Clone, Copy)]
struct RxBuffer([u8; RX_BUFFER_SIZE]);

#[repr(align(16))]
#[derive(Clone, Copy)]
struct TxBuffer([u8; MAX_FRAME_SIZE]);

static mut RX_DESCS: RxDescRing = RxDescRing([RxDesc::EMPTY; RX_DESC_COUNT]);
static mut TX_DESCS: TxDescRing = TxDescRing([TxDesc::EMPTY; TX_DESC_COUNT]);
static mut RX_BUFFERS: [RxBuffer; RX_DESC_COUNT] = [RxBuffer([0; RX_BUFFER_SIZE]); RX_DESC_COUNT];
static mut TX_BUFFERS: [TxBuffer; TX_DESC_COUNT] = [TxBuffer([0; MAX_FRAME_SIZE]); TX_DESC_COUNT];

#[derive(Clone, Copy)]
pub struct E1000Info {
    pub mac: [u8; 6],
}

pub struct RxPacket {
    len: usize,
    bytes: [u8; MAX_FRAME_SIZE],
}

impl RxPacket {
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.bytes[..self.len]
    }
}

struct E1000 {
    _mapping: MmioMapping,
    base: *mut u8,
    mac: [u8; 6],
    rx_index: usize,
    tx_index: usize,
}

unsafe impl Send for E1000 {}

pub fn probe() -> Option<E1000Info> {
    let mut guard = DRIVER.lock();
    if let Some(driver) = guard.as_ref() {
        return Some(E1000Info { mac: driver.mac });
    }

    let (address, device_id) = find_device()?;
    let Some(bar) = pci::read_bar_info(address, 0) else {
        serial::write_line("e1000: BAR0 missing");
        return None;
    };
    if !bar.is_memory() {
        serial::write_line("e1000: BAR0 is not MMIO");
        return None;
    }

    pci::enable_bus_master(address);
    let mapping = match memory::map_mmio(bar.base, bar.size as usize) {
        Ok(mapping) => mapping,
        Err(error) => {
            serial::write_fmt(format_args!("e1000: MMIO map failed: {}\r\n", error));
            return None;
        }
    };

    let mut driver = E1000 {
        _mapping: mapping,
        base: mapping.as_ptr::<u8>(),
        mac: [0; 6],
        rx_index: 0,
        tx_index: 0,
    };

    if unsafe { driver.init() }.is_none() {
        return None;
    }

    serial::write_fmt(format_args!(
        "e1000: device {} id=0x{:04x} mmio=0x{:x} size={} mac {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}\r\n",
        address,
        device_id,
        bar.base,
        bar.size,
        driver.mac[0],
        driver.mac[1],
        driver.mac[2],
        driver.mac[3],
        driver.mac[4],
        driver.mac[5],
    ));

    let info = E1000Info { mac: driver.mac };
    *guard = Some(driver);
    Some(info)
}

pub fn receive() -> Option<RxPacket> {
    DRIVER.lock().as_mut()?.receive()
}

pub fn transmit(frame: &[u8]) -> bool {
    DRIVER
        .lock()
        .as_mut()
        .map(|driver| driver.transmit(frame))
        .unwrap_or(false)
}

fn find_device() -> Option<(PciAddress, u16)> {
    for device_id in [E1000_82540EM, E1000_82545EM] {
        if let Some(address) = pci::find_device(INTEL_VENDOR_ID, device_id) {
            return Some((address, device_id));
        }
    }
    None
}

impl E1000 {
    unsafe fn init(&mut self) -> Option<()> {
        self.write32(REG_IMC, u32::MAX);
        let _ = self.read32(REG_ICR);

        self.write32(REG_RCTL, 0);
        self.write32(REG_TCTL, 0);
        self.write32(REG_CTRL, self.read32(REG_CTRL) | CTRL_SLU);

        self.mac = self.read_mac();
        if !mac_is_valid(self.mac) {
            self.mac = [0x52, 0x54, 0x00, 0x12, 0x34, 0x56];
            self.write_mac(self.mac);
        }

        for index in 0..128 {
            self.write32(REG_MTA + index * 4, 0);
        }

        self.init_rx_ring()?;
        self.init_tx_ring()?;

        let rctl = RCTL_EN | RCTL_BAM | RCTL_SECRC;
        self.write32(REG_RCTL, rctl);

        let tctl = TCTL_EN | TCTL_PSP | (0x10 << TCTL_CT_SHIFT) | (0x40 << TCTL_COLD_SHIFT);
        self.write32(REG_TCTL, tctl);

        let status = self.read32(REG_STATUS);
        serial::write_fmt(format_args!("e1000: status=0x{:08x}\r\n", status));
        Some(())
    }

    unsafe fn init_rx_ring(&mut self) -> Option<()> {
        let descs = ptr::addr_of_mut!(RX_DESCS.0).cast::<RxDesc>();
        for index in 0..RX_DESC_COUNT {
            let buffer = ptr::addr_of_mut!(RX_BUFFERS[index].0).cast::<u8>();
            let desc = descs.add(index);
            (*desc).addr = memory::virt_to_phys(buffer)?;
            (*desc).length = 0;
            (*desc).checksum = 0;
            (*desc).status = 0;
            (*desc).errors = 0;
            (*desc).special = 0;
        }

        let desc_phys = memory::virt_to_phys(descs)?;
        self.write32(REG_RDBAL, desc_phys as u32);
        self.write32(REG_RDBAH, (desc_phys >> 32) as u32);
        self.write32(
            REG_RDLEN,
            (RX_DESC_COUNT * core::mem::size_of::<RxDesc>()) as u32,
        );
        self.write32(REG_RDH, 0);
        self.write32(REG_RDT, (RX_DESC_COUNT - 1) as u32);
        self.rx_index = 0;
        Some(())
    }

    unsafe fn init_tx_ring(&mut self) -> Option<()> {
        let descs = ptr::addr_of_mut!(TX_DESCS.0).cast::<TxDesc>();
        for index in 0..TX_DESC_COUNT {
            let buffer = ptr::addr_of_mut!(TX_BUFFERS[index].0).cast::<u8>();
            let desc = descs.add(index);
            (*desc).addr = memory::virt_to_phys(buffer)?;
            (*desc).length = 0;
            (*desc).cso = 0;
            (*desc).cmd = 0;
            (*desc).status = TX_DESC_STATUS_DD;
            (*desc).css = 0;
            (*desc).special = 0;
        }

        let desc_phys = memory::virt_to_phys(descs)?;
        self.write32(REG_TDBAL, desc_phys as u32);
        self.write32(REG_TDBAH, (desc_phys >> 32) as u32);
        self.write32(
            REG_TDLEN,
            (TX_DESC_COUNT * core::mem::size_of::<TxDesc>()) as u32,
        );
        self.write32(REG_TDH, 0);
        self.write32(REG_TDT, 0);
        self.tx_index = 0;
        Some(())
    }

    fn receive(&mut self) -> Option<RxPacket> {
        unsafe {
            let descs = ptr::addr_of_mut!(RX_DESCS.0).cast::<RxDesc>();
            let desc = descs.add(self.rx_index);
            let status = ptr::read_volatile(ptr::addr_of!((*desc).status));
            if status & RX_DESC_STATUS_DD == 0 {
                return None;
            }
            if status & RX_DESC_STATUS_EOP == 0 {
                self.recycle_rx_desc(desc);
                return None;
            }

            let len = ptr::read_volatile(ptr::addr_of!((*desc).length)) as usize;
            let actual_len = cmp::min(len, MAX_FRAME_SIZE);
            let mut packet = RxPacket {
                len: actual_len,
                bytes: [0; MAX_FRAME_SIZE],
            };
            let source = ptr::addr_of!(RX_BUFFERS[self.rx_index].0).cast::<u8>();
            ptr::copy_nonoverlapping(source, packet.bytes.as_mut_ptr(), actual_len);
            self.recycle_rx_desc(desc);
            Some(packet)
        }
    }

    unsafe fn recycle_rx_desc(&mut self, desc: *mut RxDesc) {
        (*desc).length = 0;
        (*desc).checksum = 0;
        (*desc).status = 0;
        (*desc).errors = 0;
        (*desc).special = 0;
        let returned = self.rx_index;
        self.rx_index = (self.rx_index + 1) % RX_DESC_COUNT;
        compiler_fence(Ordering::SeqCst);
        self.write32(REG_RDT, returned as u32);
    }

    fn transmit(&mut self, frame: &[u8]) -> bool {
        if frame.is_empty() || frame.len() > MAX_FRAME_SIZE {
            return false;
        }

        unsafe {
            let descs = ptr::addr_of_mut!(TX_DESCS.0).cast::<TxDesc>();
            let desc = descs.add(self.tx_index);
            let status = ptr::read_volatile(ptr::addr_of!((*desc).status));
            if status & TX_DESC_STATUS_DD == 0 {
                return false;
            }

            let buffer = ptr::addr_of_mut!(TX_BUFFERS[self.tx_index].0).cast::<u8>();
            ptr::copy_nonoverlapping(frame.as_ptr(), buffer, frame.len());

            (*desc).length = frame.len() as u16;
            (*desc).cso = 0;
            (*desc).cmd = TX_DESC_CMD_EOP | TX_DESC_CMD_IFCS | TX_DESC_CMD_RS;
            (*desc).status = 0;
            (*desc).css = 0;
            (*desc).special = 0;

            let next = (self.tx_index + 1) % TX_DESC_COUNT;
            compiler_fence(Ordering::SeqCst);
            self.write32(REG_TDT, next as u32);
            self.tx_index = next;
        }

        true
    }

    unsafe fn read_mac(&self) -> [u8; 6] {
        let ral = self.read32(REG_RAL);
        let rah = self.read32(REG_RAH);
        [
            (ral & 0xff) as u8,
            ((ral >> 8) & 0xff) as u8,
            ((ral >> 16) & 0xff) as u8,
            ((ral >> 24) & 0xff) as u8,
            (rah & 0xff) as u8,
            ((rah >> 8) & 0xff) as u8,
        ]
    }

    unsafe fn write_mac(&self, mac: [u8; 6]) {
        let ral = (mac[0] as u32)
            | ((mac[1] as u32) << 8)
            | ((mac[2] as u32) << 16)
            | ((mac[3] as u32) << 24);
        let rah = (mac[4] as u32) | ((mac[5] as u32) << 8) | (1 << 31);
        self.write32(REG_RAL, ral);
        self.write32(REG_RAH, rah);
    }

    unsafe fn read32(&self, offset: usize) -> u32 {
        ptr::read_volatile(self.base.add(offset).cast::<u32>())
    }

    unsafe fn write32(&self, offset: usize, value: u32) {
        ptr::write_volatile(self.base.add(offset).cast::<u32>(), value);
    }
}

fn mac_is_valid(mac: [u8; 6]) -> bool {
    mac != [0; 6] && mac != [0xff; 6]
}
