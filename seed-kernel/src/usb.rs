use core::ptr;

use spin::Mutex;

use crate::memory;
use crate::pci::{self, PciAddress};
use crate::serial;

const PCI_CLASS_SERIAL_BUS: u8 = 0x0C;
const PCI_SUBCLASS_USB: u8 = 0x03;
const PCI_PROG_IF_XHCI: u8 = 0x30;
const XHCI_BAR: u8 = 0;

const CAPLENGTH: usize = 0x00;
const HCSPARAMS1: usize = 0x04;
const HCIVERSION: usize = 0x02;
const PORTSC_BASE: usize = 0x400;
const PORT_REGISTER_STRIDE: usize = 0x10;
const PORTSC_CCS: u32 = 1 << 0;

static STATE: Mutex<UsbState> = Mutex::new(UsbState::new());

#[derive(Clone, Copy)]
pub struct UsbSnapshot {
    pub state: UsbStatus,
    pub address: Option<PciAddress>,
    pub hci_version: u16,
    pub max_ports: u8,
    pub connected_ports: u8,
    pub last_error: Option<&'static str>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UsbStatus {
    NotProbed,
    Missing,
    Ready,
    Error,
}

struct UsbState {
    snapshot: UsbSnapshot,
}

impl UsbState {
    const fn new() -> Self {
        Self {
            snapshot: UsbSnapshot {
                state: UsbStatus::NotProbed,
                address: None,
                hci_version: 0,
                max_ports: 0,
                connected_ports: 0,
                last_error: None,
            },
        }
    }
}

pub fn init() {
    let snapshot = probe_xhci();
    *STATE.lock() = UsbState { snapshot };
}

pub fn snapshot() -> UsbSnapshot {
    STATE.lock().snapshot
}

fn probe_xhci() -> UsbSnapshot {
    let Some(address) =
        pci::find_by_class(PCI_CLASS_SERIAL_BUS, PCI_SUBCLASS_USB, PCI_PROG_IF_XHCI)
    else {
        serial::write_line("usb-xhci: controller not present");
        return UsbSnapshot {
            state: UsbStatus::Missing,
            address: None,
            hci_version: 0,
            max_ports: 0,
            connected_ports: 0,
            last_error: None,
        };
    };

    serial::write_fmt(format_args!(
        "usb-xhci: controller @ {} detected\r\n",
        address
    ));

    let Some(bar) = pci::read_bar_info(address, XHCI_BAR) else {
        return error_snapshot(address, "usb-xhci: missing BAR0");
    };
    if !bar.is_memory() {
        return error_snapshot(address, "usb-xhci: BAR0 is not MMIO");
    }

    pci::enable_bus_master(address);

    let map_len = usize::min(bar.size as usize, 0x10000).max(0x1000);
    let Ok(mapping) = memory::map_mmio(bar.base, map_len) else {
        return error_snapshot(address, "usb-xhci: MMIO map failed");
    };

    let base = mapping.as_ptr::<u8>();
    let hci_version = unsafe { read_u16(base, HCIVERSION) };
    let cap_length = unsafe { read_u8(base, CAPLENGTH) } as usize;
    let hcsparams1 = unsafe { read_u32(base, HCSPARAMS1) };
    let max_ports = ((hcsparams1 >> 24) & 0xFF) as u8;
    let connected_ports = count_connected_ports(base, cap_length, max_ports, mapping.len());

    serial::write_fmt(format_args!(
        "usb-xhci: hci 0x{:04x}, ports {}, connected {}\r\n",
        hci_version, max_ports, connected_ports
    ));

    UsbSnapshot {
        state: UsbStatus::Ready,
        address: Some(address),
        hci_version,
        max_ports,
        connected_ports,
        last_error: None,
    }
}

fn error_snapshot(address: PciAddress, error: &'static str) -> UsbSnapshot {
    serial::write_line(error);
    UsbSnapshot {
        state: UsbStatus::Error,
        address: Some(address),
        hci_version: 0,
        max_ports: 0,
        connected_ports: 0,
        last_error: Some(error),
    }
}

fn count_connected_ports(base: *mut u8, cap_length: usize, max_ports: u8, map_len: usize) -> u8 {
    let mut count = 0u8;
    let mut port = 0usize;
    while port < max_ports as usize {
        let offset = cap_length + PORTSC_BASE + port * PORT_REGISTER_STRIDE;
        if offset + 4 > map_len {
            break;
        }
        let status = unsafe { read_u32(base, offset) };
        if status & PORTSC_CCS != 0 {
            count = count.saturating_add(1);
        }
        port += 1;
    }
    count
}

unsafe fn read_u8(base: *mut u8, offset: usize) -> u8 {
    ptr::read_volatile(base.add(offset))
}

unsafe fn read_u16(base: *mut u8, offset: usize) -> u16 {
    let low = read_u8(base, offset) as u16;
    let high = read_u8(base, offset + 1) as u16;
    low | (high << 8)
}

unsafe fn read_u32(base: *mut u8, offset: usize) -> u32 {
    let b0 = read_u8(base, offset) as u32;
    let b1 = read_u8(base, offset + 1) as u32;
    let b2 = read_u8(base, offset + 2) as u32;
    let b3 = read_u8(base, offset + 3) as u32;
    b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
}
