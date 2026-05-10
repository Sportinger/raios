#![allow(static_mut_refs)]

use core::hint::spin_loop;
use core::ptr;
use core::sync::atomic::{fence, Ordering};

use spin::Mutex;

use crate::memory;
use crate::pci::{self, PciAddress};
use crate::serial;

const PCI_CLASS_SERIAL_BUS: u8 = 0x0C;
const PCI_SUBCLASS_USB: u8 = 0x03;
const PCI_PROG_IF_XHCI: u8 = 0x30;
const XHCI_BAR: u8 = 0;

const CAP_CAPLENGTH: usize = 0x00;
const CAP_HCSPARAMS1: usize = 0x04;
const CAP_HCSPARAMS2: usize = 0x08;
const CAP_HCCPARAMS1: usize = 0x10;
const CAP_DBOFF: usize = 0x14;
const CAP_RTSOFF: usize = 0x18;

const OP_USBCMD: usize = 0x00;
const OP_USBSTS: usize = 0x04;
const OP_PAGESIZE: usize = 0x08;
const OP_CRCR: usize = 0x18;
const OP_DCBAAP: usize = 0x30;
const OP_CONFIG: usize = 0x38;
const OP_PORT_BASE: usize = 0x400;
const PORT_REGISTER_STRIDE: usize = 0x10;

const USBCMD_RUN_STOP: u32 = 1 << 0;
const USBCMD_HOST_CONTROLLER_RESET: u32 = 1 << 1;
const USBCMD_INTERRUPTER_ENABLE: u32 = 1 << 2;
const USBSTS_HALTED: u32 = 1 << 0;
const USBSTS_CONTROLLER_NOT_READY: u32 = 1 << 11;

const PORTSC_CCS: u32 = 1 << 0;
const PORTSC_PED: u32 = 1 << 1;
const PORTSC_PR: u32 = 1 << 4;
const PORTSC_PP: u32 = 1 << 9;
const PORTSC_SPEED_SHIFT: u32 = 10;
const PORTSC_SPEED_MASK: u32 = 0xF << PORTSC_SPEED_SHIFT;
const PORTSC_CHANGE_BITS: u32 =
    (1 << 17) | (1 << 18) | (1 << 20) | (1 << 21) | (1 << 22) | (1 << 23) | (1 << 24);
const PORTSC_PRESERVE_MASK: u32 = (0xF << 5) | PORTSC_PP | (0x3 << 14) | (0x7 << 25);

const RUNTIME_IR0: usize = 0x20;
const IR_IMAN: usize = 0x00;
const IR_IMOD: usize = 0x04;
const IR_ERSTSZ: usize = 0x08;
const IR_ERSTBA: usize = 0x10;
const IR_ERDP: usize = 0x18;
const ERDP_EHB: u64 = 1 << 3;

const TRB_TYPE_NORMAL: u32 = 1;
const TRB_TYPE_SETUP_STAGE: u32 = 2;
const TRB_TYPE_DATA_STAGE: u32 = 3;
const TRB_TYPE_STATUS_STAGE: u32 = 4;
const TRB_TYPE_LINK: u32 = 6;
const TRB_TYPE_ENABLE_SLOT: u32 = 9;
const TRB_TYPE_ADDRESS_DEVICE: u32 = 11;
const TRB_TYPE_CONFIGURE_ENDPOINT: u32 = 12;
const TRB_TYPE_TRANSFER_EVENT: u32 = 32;
const TRB_TYPE_COMMAND_COMPLETION_EVENT: u32 = 33;

const TRB_CYCLE: u32 = 1 << 0;
const TRB_LINK_TOGGLE_CYCLE: u32 = 1 << 1;
const TRB_IOC: u32 = 1 << 5;
const TRB_IDT: u32 = 1 << 6;
const TRB_DIR_IN: u32 = 1 << 16;
const TRB_TYPE_SHIFT: u32 = 10;
const SETUP_TRT_NO_DATA: u32 = 0;
const SETUP_TRT_OUT: u32 = 2;
const SETUP_TRT_IN: u32 = 3;

const CC_SUCCESS: u32 = 1;
const CC_SHORT_PACKET: u32 = 13;

const DCI_EP0: u8 = 1;
const EP_TYPE_CONTROL: u32 = 4;
const EP_TYPE_INTERRUPT_IN: u32 = 7;
const CONTEXT_ENTRIES_EP0: u8 = 1;

const USB_REQ_GET_DESCRIPTOR: u8 = 6;
const USB_REQ_SET_CONFIGURATION: u8 = 9;
const HID_REQ_SET_IDLE: u8 = 10;
const HID_REQ_SET_PROTOCOL: u8 = 11;
const DESC_DEVICE: u8 = 1;
const DESC_CONFIGURATION: u8 = 2;

const COMMAND_RING_LEN: usize = 64;
const EVENT_RING_LEN: usize = 64;
const EP0_RING_LEN: usize = 64;
const INTR_RING_LEN: usize = 16;
const MAX_SLOTS: usize = 32;
const MAX_SCRATCHPADS: usize = 64;
const CONTEXT_BYTES: usize = 4096;
const CONTROL_BUFFER_LEN: usize = 256;
const KEYBOARD_REPORT_LEN: usize = 8;
const WAIT_ITERS: usize = 5_000_000;

static STATE: Mutex<UsbState> = Mutex::new(UsbState::new());

#[derive(Clone, Copy)]
pub struct UsbSnapshot {
    pub state: UsbStatus,
    pub address: Option<PciAddress>,
    pub hci_version: u16,
    pub max_ports: u8,
    pub connected_ports: u8,
    pub keyboard_status: UsbKeyboardStatus,
    pub keyboard_detail: Option<&'static str>,
    pub last_error: Option<&'static str>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UsbStatus {
    NotProbed,
    Missing,
    Ready,
    Error,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UsbKeyboardStatus {
    NotProbed,
    Ready,
    NotFound,
    Error,
}

struct UsbState {
    snapshot: UsbSnapshot,
    controller: Option<XhciController>,
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
                keyboard_status: UsbKeyboardStatus::NotProbed,
                keyboard_detail: None,
                last_error: None,
            },
            controller: None,
        }
    }
}

pub fn init() {
    let (snapshot, controller) = unsafe { probe_xhci() };
    *STATE.lock() = UsbState {
        snapshot,
        controller,
    };
}

pub fn snapshot() -> UsbSnapshot {
    STATE.lock().snapshot
}

pub fn keyboard_active() -> bool {
    STATE
        .lock()
        .controller
        .as_ref()
        .is_some_and(|controller| controller.keyboard.is_some())
}

pub fn keyboard_detail() -> &'static str {
    if keyboard_active() {
        "USB HID BOOT KEYBOARD"
    } else {
        "USB HID KEYBOARD ABSENT"
    }
}

pub fn poll_keyboard<F: FnMut(u16, bool)>(mut f: F) {
    let mut state = STATE.lock();
    if let Some(controller) = state.controller.as_mut() {
        unsafe {
            controller.poll_keyboard(&mut f);
        }
    }
}

unsafe fn probe_xhci() -> (UsbSnapshot, Option<XhciController>) {
    let Some(address) =
        pci::find_by_class(PCI_CLASS_SERIAL_BUS, PCI_SUBCLASS_USB, PCI_PROG_IF_XHCI)
    else {
        serial::write_line("usb-xhci: controller not present");
        return (
            UsbSnapshot {
                state: UsbStatus::Missing,
                address: None,
                hci_version: 0,
                max_ports: 0,
                connected_ports: 0,
                keyboard_status: UsbKeyboardStatus::NotProbed,
                keyboard_detail: None,
                last_error: None,
            },
            None,
        );
    };

    serial::write_fmt(format_args!(
        "usb-xhci: controller @ {} detected\r\n",
        address
    ));

    let Some(bar) = pci::read_bar_info(address, XHCI_BAR) else {
        return (error_snapshot(address, "usb-xhci: missing BAR0"), None);
    };
    if !bar.is_memory() {
        return (error_snapshot(address, "usb-xhci: BAR0 is not MMIO"), None);
    }

    serial::write_fmt(format_args!(
        "usb-xhci: BAR{} base 0x{:x} size 0x{:x}\r\n",
        bar.index, bar.base, bar.size
    ));

    pci::enable_bus_master(address);

    let map_len = usize::min(bar.size as usize, 0x100000).max(0x1000);
    let Ok(mapping) = memory::map_mmio(bar.base, map_len) else {
        return (error_snapshot(address, "usb-xhci: MMIO map failed"), None);
    };

    match XhciController::new(address, mapping) {
        Ok(mut controller) => {
            let connected_ports = controller.count_connected_ports();
            serial::write_fmt(format_args!(
                "usb-xhci: hci 0x{:04x}, ports {}, connected {}\r\n",
                controller.hci_version, controller.max_ports, connected_ports
            ));

            let (keyboard_status, keyboard_detail) = match controller.initialise_keyboard() {
                Ok(()) => (UsbKeyboardStatus::Ready, Some("USB HID BOOT KEYBOARD")),
                Err(err) => {
                    serial::write_fmt(format_args!("usb-hid: {}\r\n", err));
                    (UsbKeyboardStatus::NotFound, Some(err))
                }
            };

            let snapshot = UsbSnapshot {
                state: UsbStatus::Ready,
                address: Some(address),
                hci_version: controller.hci_version,
                max_ports: controller.max_ports,
                connected_ports,
                keyboard_status,
                keyboard_detail,
                last_error: None,
            };

            (snapshot, Some(controller))
        }
        Err(err) => (error_snapshot(address, err), None),
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
        keyboard_status: UsbKeyboardStatus::Error,
        keyboard_detail: Some(error),
        last_error: Some(error),
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Trb {
    parameter: u64,
    status: u32,
    control: u32,
}

impl Trb {
    const fn zero() -> Self {
        Self {
            parameter: 0,
            status: 0,
            control: 0,
        }
    }

    fn trb_type(self) -> u32 {
        (self.control >> TRB_TYPE_SHIFT) & 0x3F
    }

    fn completion_code(self) -> u32 {
        (self.status >> 24) & 0xFF
    }

    fn slot_id(self) -> u8 {
        (self.control >> 24) as u8
    }

    fn endpoint_id(self) -> u8 {
        ((self.control >> 16) & 0x1F) as u8
    }
}

#[repr(C, align(64))]
struct TrbRing<const N: usize>([Trb; N]);

#[repr(C)]
#[derive(Clone, Copy)]
struct ErstEntry {
    ring_segment_base: u64,
    ring_segment_size: u32,
    reserved: u32,
}

impl ErstEntry {
    const fn zero() -> Self {
        Self {
            ring_segment_base: 0,
            ring_segment_size: 0,
            reserved: 0,
        }
    }
}

#[repr(C, align(64))]
struct Erst([ErstEntry; 1]);

#[repr(C, align(64))]
struct Dcbaa([u64; 256]);

#[repr(C, align(64))]
struct AlignedBytes<const N: usize>([u8; N]);

#[repr(C, align(4096))]
#[derive(Clone, Copy)]
struct ScratchPage([u8; 4096]);

static mut COMMAND_RING: TrbRing<COMMAND_RING_LEN> = TrbRing([Trb::zero(); COMMAND_RING_LEN]);
static mut EVENT_RING: TrbRing<EVENT_RING_LEN> = TrbRing([Trb::zero(); EVENT_RING_LEN]);
static mut EP0_RING: TrbRing<EP0_RING_LEN> = TrbRing([Trb::zero(); EP0_RING_LEN]);
static mut INTR_RING: TrbRing<INTR_RING_LEN> = TrbRing([Trb::zero(); INTR_RING_LEN]);
static mut ERST: Erst = Erst([ErstEntry::zero(); 1]);
static mut DCBAA: Dcbaa = Dcbaa([0; 256]);
static mut SCRATCHPAD_ARRAY: AlignedBytes<{ MAX_SCRATCHPADS * 8 }> =
    AlignedBytes([0; MAX_SCRATCHPADS * 8]);
static mut SCRATCHPAD_PAGES: [ScratchPage; MAX_SCRATCHPADS] =
    [ScratchPage([0; 4096]); MAX_SCRATCHPADS];
static mut INPUT_CONTEXT: AlignedBytes<CONTEXT_BYTES> = AlignedBytes([0; CONTEXT_BYTES]);
static mut DEVICE_CONTEXT: AlignedBytes<CONTEXT_BYTES> = AlignedBytes([0; CONTEXT_BYTES]);
static mut CONTROL_BUFFER: AlignedBytes<CONTROL_BUFFER_LEN> = AlignedBytes([0; CONTROL_BUFFER_LEN]);
static mut KEYBOARD_REPORT: AlignedBytes<KEYBOARD_REPORT_LEN> =
    AlignedBytes([0; KEYBOARD_REPORT_LEN]);

struct XhciController {
    _address: PciAddress,
    _mapping: memory::MmioMapping,
    base: *mut u8,
    _cap_length: usize,
    hci_version: u16,
    _max_slots: u8,
    max_ports: u8,
    context_size: usize,
    op_offset: usize,
    runtime_offset: usize,
    doorbell_offset: usize,
    command_enqueue: usize,
    command_cycle: bool,
    event_dequeue: usize,
    event_cycle: bool,
    ep0_enqueue: usize,
    ep0_cycle: bool,
    intr_enqueue: usize,
    intr_cycle: bool,
    event_ring_phys: u64,
    current_slot_id: u8,
    keyboard: Option<KeyboardDevice>,
}

unsafe impl Send for XhciController {}
unsafe impl Sync for XhciController {}

#[derive(Clone, Copy)]
struct KeyboardDevice {
    slot_id: u8,
    dci: u8,
    previous_report: [u8; KEYBOARD_REPORT_LEN],
}

#[derive(Clone, Copy)]
struct PortInfo {
    port_number: u8,
    speed: u8,
}

#[derive(Clone, Copy)]
struct KeyboardEndpoint {
    interface_number: u8,
    configuration_value: u8,
    endpoint_address: u8,
    dci: u8,
    max_packet_size: u16,
    interval: u8,
}

impl XhciController {
    unsafe fn new(address: PciAddress, mapping: memory::MmioMapping) -> Result<Self, &'static str> {
        let base = mapping.as_ptr::<u8>();
        let cap_header = read32(base, CAP_CAPLENGTH);
        let cap_length = (cap_header & 0xFF) as usize;
        let hci_version = ((cap_header >> 16) & 0xFFFF) as u16;
        let hcsparams1 = read32(base, CAP_HCSPARAMS1);
        let hcsparams2 = read32(base, CAP_HCSPARAMS2);
        let hccparams1 = read32(base, CAP_HCCPARAMS1);
        let max_slots = u8::min((hcsparams1 & 0xFF) as u8, MAX_SLOTS as u8);
        let max_ports = ((hcsparams1 >> 24) & 0xFF) as u8;
        let context_size = if hccparams1 & (1 << 2) != 0 { 64 } else { 32 };
        let doorbell_offset = (read32(base, CAP_DBOFF) & !0x3) as usize;
        let runtime_offset = (read32(base, CAP_RTSOFF) & !0x1F) as usize;

        if max_slots == 0 {
            return Err("usb-xhci: controller reports zero slots");
        }
        if cap_length + OP_PORT_BASE + max_ports as usize * PORT_REGISTER_STRIDE > mapping.len() {
            return Err("usb-xhci: port register range outside BAR map");
        }
        if doorbell_offset + (max_slots as usize + 1) * 4 > mapping.len() {
            return Err("usb-xhci: doorbell range outside BAR map");
        }
        if runtime_offset + RUNTIME_IR0 + IR_ERDP + 8 > mapping.len() {
            return Err("usb-xhci: runtime range outside BAR map");
        }

        legacy_handoff(base, hccparams1, mapping.len());
        stop_and_reset(base, cap_length)?;

        let scratchpads = max_scratchpads(hcsparams2);
        if scratchpads > MAX_SCRATCHPADS {
            return Err("usb-xhci: scratchpad requirement too large");
        }

        let op_offset = cap_length;
        let page_size = read32(base, op_offset + OP_PAGESIZE);
        if page_size & 1 == 0 {
            return Err("usb-xhci: 4K pages unsupported");
        }

        zero_static_storage();
        setup_scratchpads(scratchpads)?;

        let command_ring_phys = phys_of(ptr::addr_of!(COMMAND_RING.0[0]), "command ring phys")?;
        let event_ring_phys = phys_of(ptr::addr_of!(EVENT_RING.0[0]), "event ring phys")?;
        let erst_phys = phys_of(ptr::addr_of!(ERST.0[0]), "event table phys")?;
        let dcbaa_phys = phys_of(ptr::addr_of!(DCBAA.0[0]), "dcbaa phys")?;

        ERST.0[0] = ErstEntry {
            ring_segment_base: event_ring_phys,
            ring_segment_size: EVENT_RING_LEN as u32,
            reserved: 0,
        };

        write64(base, op_offset + OP_CRCR, command_ring_phys | 1);
        write64(base, op_offset + OP_DCBAAP, dcbaa_phys);
        write32(base, op_offset + OP_CONFIG, max_slots as u32);

        let ir0 = runtime_offset + RUNTIME_IR0;
        write32(base, ir0 + IR_IMOD, 0);
        write32(base, ir0 + IR_ERSTSZ, 1);
        write64(base, ir0 + IR_ERSTBA, erst_phys);
        write64(base, ir0 + IR_ERDP, event_ring_phys | ERDP_EHB);
        write32(base, ir0 + IR_IMAN, 1 << 1);

        fence(Ordering::SeqCst);
        start_controller(base, op_offset)?;

        Ok(Self {
            _address: address,
            _mapping: mapping,
            base,
            _cap_length: cap_length,
            hci_version,
            _max_slots: max_slots,
            max_ports,
            context_size,
            op_offset,
            runtime_offset,
            doorbell_offset,
            command_enqueue: 0,
            command_cycle: true,
            event_dequeue: 0,
            event_cycle: true,
            ep0_enqueue: 0,
            ep0_cycle: true,
            intr_enqueue: 0,
            intr_cycle: true,
            event_ring_phys,
            current_slot_id: 0,
            keyboard: None,
        })
    }

    unsafe fn count_connected_ports(&self) -> u8 {
        let mut count = 0u8;
        let mut port = 1u8;
        while port <= self.max_ports {
            if read32(self.base, self.portsc_offset(port)) & PORTSC_CCS != 0 {
                count = count.saturating_add(1);
            }
            port += 1;
        }
        count
    }

    unsafe fn initialise_keyboard(&mut self) -> Result<(), &'static str> {
        let mut port = 1u8;
        while port <= self.max_ports {
            let status = read32(self.base, self.portsc_offset(port));
            if status & PORTSC_CCS == 0 {
                port += 1;
                continue;
            }

            match self.reset_port(port) {
                Ok(port_info) => match self.enumerate_keyboard(port_info) {
                    Ok(()) => return Ok(()),
                    Err(err) => {
                        serial::write_fmt(format_args!(
                            "usb-hid: port {} skipped: {}\r\n",
                            port, err
                        ));
                    }
                },
                Err(err) => {
                    serial::write_fmt(format_args!("usb-xhci: port {} reset: {}\r\n", port, err));
                }
            }

            port += 1;
        }

        Err("no USB boot keyboard found")
    }

    unsafe fn reset_port(&self, port_number: u8) -> Result<PortInfo, &'static str> {
        let offset = self.portsc_offset(port_number);
        let initial = read32(self.base, offset);
        if initial & PORTSC_PP == 0 {
            self.write_portsc(offset, PORTSC_PP);
            wait_for(WAIT_ITERS / 10, || {
                read32(self.base, offset) & PORTSC_PP != 0
            })
            .ok_or("port power did not enable")?;
        }

        self.write_portsc(offset, PORTSC_PR | PORTSC_PP);
        wait_for(WAIT_ITERS, || read32(self.base, offset) & PORTSC_PR == 0)
            .ok_or("port reset did not complete")?;

        let status = read32(self.base, offset);
        self.write_portsc(offset, PORTSC_CHANGE_BITS | PORTSC_PP);
        if status & PORTSC_PED == 0 {
            return Err("port not enabled after reset");
        }

        let speed = ((status & PORTSC_SPEED_MASK) >> PORTSC_SPEED_SHIFT) as u8;
        serial::write_fmt(format_args!(
            "usb-xhci: port {} reset complete speed {}\r\n",
            port_number, speed
        ));
        Ok(PortInfo { port_number, speed })
    }

    unsafe fn enumerate_keyboard(&mut self, port: PortInfo) -> Result<(), &'static str> {
        let slot_id = self.enable_slot()?;
        self.current_slot_id = slot_id;
        self.reset_ep0_ring();
        self.prepare_address_context(slot_id, port)?;
        let input_phys = phys_of(ptr::addr_of!(INPUT_CONTEXT.0[0]), "input context phys")?;
        self.execute_command(Trb {
            parameter: input_phys,
            status: 0,
            control: trb_type(TRB_TYPE_ADDRESS_DEVICE) | ((slot_id as u32) << 24),
        })?;

        let device_desc = self.get_device_descriptor()?;
        let ep0_mps = descriptor_ep0_mps(port.speed, device_desc[7]);
        if ep0_mps != default_ep0_mps(port.speed) {
            serial::write_fmt(format_args!(
                "usb-xhci: ep0 mps descriptor {} initial {}\r\n",
                ep0_mps,
                default_ep0_mps(port.speed)
            ));
        }

        let endpoint = self.find_keyboard_endpoint()?;
        self.control_no_data(
            0x00,
            USB_REQ_SET_CONFIGURATION,
            endpoint.configuration_value as u16,
            0,
        )?;
        self.control_no_data(
            0x21,
            HID_REQ_SET_PROTOCOL,
            0,
            endpoint.interface_number as u16,
        )?;
        self.control_no_data(0x21, HID_REQ_SET_IDLE, 0, endpoint.interface_number as u16)?;

        self.configure_interrupt_endpoint(slot_id, port, endpoint)?;
        serial::write_fmt(format_args!(
            "usb-hid: boot keyboard ready on slot {} endpoint 0x{:02x}\r\n",
            slot_id, endpoint.endpoint_address
        ));
        Ok(())
    }

    unsafe fn enable_slot(&mut self) -> Result<u8, &'static str> {
        let event = self.execute_command(Trb {
            parameter: 0,
            status: 0,
            control: trb_type(TRB_TYPE_ENABLE_SLOT),
        })?;
        let slot_id = event.slot_id();
        if slot_id == 0 || slot_id as usize > MAX_SLOTS {
            return Err("enable slot returned invalid slot");
        }
        Ok(slot_id)
    }

    unsafe fn prepare_address_context(
        &self,
        slot_id: u8,
        port: PortInfo,
    ) -> Result<(), &'static str> {
        zero_contexts();
        let device_context_phys =
            phys_of(ptr::addr_of!(DEVICE_CONTEXT.0[0]), "device context phys")?;
        DCBAA.0[slot_id as usize] = device_context_phys;

        input_control_add_flags((1 << 0) | (1 << DCI_EP0));
        write_slot_context(port, CONTEXT_ENTRIES_EP0, self.context_size);
        write_endpoint_context(
            DCI_EP0,
            self.context_size,
            EP_TYPE_CONTROL,
            0,
            default_ep0_mps(port.speed),
            phys_of(ptr::addr_of!(EP0_RING.0[0]), "ep0 ring phys")? | 1,
            8,
            0,
        );
        fence(Ordering::SeqCst);
        Ok(())
    }

    unsafe fn get_device_descriptor(&mut self) -> Result<[u8; 18], &'static str> {
        let len = self.control_in(
            0x80,
            USB_REQ_GET_DESCRIPTOR,
            (DESC_DEVICE as u16) << 8,
            0,
            18,
        )?;
        if len < 18 {
            return Err("short device descriptor");
        }
        let mut out = [0u8; 18];
        out.copy_from_slice(&CONTROL_BUFFER.0[..18]);
        if out[1] != DESC_DEVICE {
            return Err("unexpected device descriptor type");
        }
        serial::write_fmt(format_args!(
            "usb-hid: device class {:02x} subclass {:02x} protocol {:02x}\r\n",
            out[4], out[5], out[6]
        ));
        Ok(out)
    }

    unsafe fn find_keyboard_endpoint(&mut self) -> Result<KeyboardEndpoint, &'static str> {
        let header_len = self.control_in(
            0x80,
            USB_REQ_GET_DESCRIPTOR,
            (DESC_CONFIGURATION as u16) << 8,
            0,
            9,
        )?;
        if header_len < 9 || CONTROL_BUFFER.0[1] != DESC_CONFIGURATION {
            return Err("configuration header unavailable");
        }
        let total_len = u16::from_le_bytes([CONTROL_BUFFER.0[2], CONTROL_BUFFER.0[3]]) as usize;
        let config_len = usize::min(total_len, CONTROL_BUFFER_LEN);
        let actual_len = self.control_in(
            0x80,
            USB_REQ_GET_DESCRIPTOR,
            (DESC_CONFIGURATION as u16) << 8,
            0,
            config_len,
        )?;
        parse_keyboard_endpoint(&CONTROL_BUFFER.0[..actual_len])
    }

    unsafe fn configure_interrupt_endpoint(
        &mut self,
        slot_id: u8,
        port: PortInfo,
        endpoint: KeyboardEndpoint,
    ) -> Result<(), &'static str> {
        self.reset_interrupt_ring();
        clear_input_context();
        input_control_add_flags((1 << 0) | (1 << endpoint.dci));
        write_slot_context(port, endpoint.dci, self.context_size);
        write_endpoint_context(
            endpoint.dci,
            self.context_size,
            EP_TYPE_INTERRUPT_IN,
            interval_to_xhci(port.speed, endpoint.interval),
            endpoint.max_packet_size,
            phys_of(ptr::addr_of!(INTR_RING.0[0]), "interrupt ring phys")? | 1,
            endpoint.max_packet_size,
            endpoint.max_packet_size,
        );
        fence(Ordering::SeqCst);

        let input_phys = phys_of(ptr::addr_of!(INPUT_CONTEXT.0[0]), "input context phys")?;
        self.execute_command(Trb {
            parameter: input_phys,
            status: 0,
            control: trb_type(TRB_TYPE_CONFIGURE_ENDPOINT) | ((slot_id as u32) << 24),
        })?;

        self.keyboard = Some(KeyboardDevice {
            slot_id,
            dci: endpoint.dci,
            previous_report: [0; KEYBOARD_REPORT_LEN],
        });
        self.queue_keyboard_report()?;
        Ok(())
    }

    unsafe fn control_in(
        &mut self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        length: usize,
    ) -> Result<usize, &'static str> {
        if length > CONTROL_BUFFER_LEN {
            return Err("control buffer too small");
        }
        ptr::write_bytes(CONTROL_BUFFER.0.as_mut_ptr(), 0, CONTROL_BUFFER_LEN);
        self.control_transfer(request_type, request, value, index, length, true)
    }

    unsafe fn control_no_data(
        &mut self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
    ) -> Result<(), &'static str> {
        self.control_transfer(request_type, request, value, index, 0, false)?;
        Ok(())
    }

    unsafe fn control_transfer(
        &mut self,
        request_type: u8,
        request: u8,
        value: u16,
        index: u16,
        length: usize,
        input: bool,
    ) -> Result<usize, &'static str> {
        let setup = (request_type as u64)
            | ((request as u64) << 8)
            | ((value as u64) << 16)
            | ((index as u64) << 32)
            | ((length as u64) << 48);
        let transfer_type = if length == 0 {
            SETUP_TRT_NO_DATA
        } else if input {
            SETUP_TRT_IN
        } else {
            SETUP_TRT_OUT
        };

        self.push_ep0(Trb {
            parameter: setup,
            status: 8,
            control: TRB_IDT | (transfer_type << 16) | trb_type(TRB_TYPE_SETUP_STAGE),
        })?;

        if length > 0 {
            let data_phys = phys_of(ptr::addr_of!(CONTROL_BUFFER.0[0]), "control data phys")?;
            self.push_ep0(Trb {
                parameter: data_phys,
                status: length as u32,
                control: if input { TRB_DIR_IN } else { 0 } | trb_type(TRB_TYPE_DATA_STAGE),
            })?;
        }

        self.push_ep0(Trb {
            parameter: 0,
            status: 0,
            control: if input { 0 } else { TRB_DIR_IN } | TRB_IOC | trb_type(TRB_TYPE_STATUS_STAGE),
        })?;

        fence(Ordering::SeqCst);
        self.ring_doorbell(self.keyboard_slot_or_one(), DCI_EP0);
        let event = self.wait_transfer_event(self.keyboard_slot_or_one(), DCI_EP0)?;
        let cc = event.completion_code();
        if cc != CC_SUCCESS && cc != CC_SHORT_PACKET {
            return Err("control transfer failed");
        }
        let residual = (event.status & 0x00FF_FFFF) as usize;
        Ok(length.saturating_sub(residual))
    }

    fn keyboard_slot_or_one(&self) -> u8 {
        self.keyboard
            .map_or(self.current_slot_id.max(1), |keyboard| keyboard.slot_id)
    }

    unsafe fn execute_command(&mut self, mut trb: Trb) -> Result<Trb, &'static str> {
        let command_type = trb.trb_type();
        if self.command_enqueue >= COMMAND_RING_LEN {
            return Err("command ring exhausted");
        }
        let index = self.command_enqueue;
        if self.command_cycle {
            trb.control |= TRB_CYCLE;
        } else {
            trb.control &= !TRB_CYCLE;
        }
        let trb_ptr = ptr::addr_of_mut!(COMMAND_RING.0[index]);
        ptr::write_volatile(trb_ptr, trb);
        let trb_phys = phys_of(trb_ptr, "command trb phys")?;
        self.command_enqueue += 1;

        fence(Ordering::SeqCst);
        self.ring_doorbell(0, 0);

        let mut waited = 0usize;
        while waited < WAIT_ITERS {
            if let Some(event) = self.poll_event() {
                if event.trb_type() == TRB_TYPE_COMMAND_COMPLETION_EVENT
                    && event.parameter == trb_phys
                {
                    let cc = event.completion_code();
                    if cc == CC_SUCCESS {
                        return Ok(event);
                    }
                    serial::write_fmt(format_args!(
                        "usb-xhci: command type {} completion code {}\r\n",
                        command_type, cc
                    ));
                    return Err("xHCI command failed");
                }
            }
            spin_loop();
            waited += 1;
        }
        Err("xHCI command timeout")
    }

    unsafe fn wait_transfer_event(&mut self, slot_id: u8, dci: u8) -> Result<Trb, &'static str> {
        let mut waited = 0usize;
        while waited < WAIT_ITERS {
            if let Some(event) = self.poll_event() {
                if event.trb_type() == TRB_TYPE_TRANSFER_EVENT
                    && event.slot_id() == slot_id
                    && event.endpoint_id() == dci
                {
                    return Ok(event);
                }
            }
            spin_loop();
            waited += 1;
        }
        Err("transfer event timeout")
    }

    unsafe fn poll_event(&mut self) -> Option<Trb> {
        let trb_ptr = ptr::addr_of!(EVENT_RING.0[self.event_dequeue]);
        let event = ptr::read_volatile(trb_ptr);
        let has_cycle = event.control & TRB_CYCLE != 0;
        if has_cycle != self.event_cycle {
            return None;
        }

        self.event_dequeue += 1;
        if self.event_dequeue == EVENT_RING_LEN {
            self.event_dequeue = 0;
            self.event_cycle = !self.event_cycle;
        }

        let erdp = self.event_ring_phys + (self.event_dequeue * core::mem::size_of::<Trb>()) as u64;
        write64(
            self.base,
            self.runtime_offset + RUNTIME_IR0 + IR_ERDP,
            erdp | ERDP_EHB,
        );

        Some(event)
    }

    unsafe fn push_ep0(&mut self, mut trb: Trb) -> Result<(), &'static str> {
        if self.ep0_enqueue >= EP0_RING_LEN {
            return Err("ep0 ring exhausted");
        }
        if self.ep0_cycle {
            trb.control |= TRB_CYCLE;
        } else {
            trb.control &= !TRB_CYCLE;
        }
        ptr::write_volatile(ptr::addr_of_mut!(EP0_RING.0[self.ep0_enqueue]), trb);
        self.ep0_enqueue += 1;
        Ok(())
    }

    unsafe fn queue_keyboard_report(&mut self) -> Result<(), &'static str> {
        let Some(keyboard) = self.keyboard else {
            return Err("keyboard not configured");
        };
        ptr::write_bytes(KEYBOARD_REPORT.0.as_mut_ptr(), 0, KEYBOARD_REPORT_LEN);
        self.push_interrupt_trb(Trb {
            parameter: phys_of(ptr::addr_of!(KEYBOARD_REPORT.0[0]), "keyboard report phys")?,
            status: KEYBOARD_REPORT_LEN as u32,
            control: TRB_IOC | trb_type(TRB_TYPE_NORMAL),
        })?;
        fence(Ordering::SeqCst);
        self.ring_doorbell(keyboard.slot_id, keyboard.dci);
        Ok(())
    }

    unsafe fn push_interrupt_trb(&mut self, mut trb: Trb) -> Result<(), &'static str> {
        if self.intr_enqueue == INTR_RING_LEN - 1 {
            let link_phys = phys_of(ptr::addr_of!(INTR_RING.0[0]), "interrupt ring link phys")?;
            let mut link = Trb {
                parameter: link_phys,
                status: 0,
                control: TRB_LINK_TOGGLE_CYCLE | trb_type(TRB_TYPE_LINK),
            };
            if self.intr_cycle {
                link.control |= TRB_CYCLE;
            }
            ptr::write_volatile(ptr::addr_of_mut!(INTR_RING.0[self.intr_enqueue]), link);
            self.intr_enqueue = 0;
            self.intr_cycle = !self.intr_cycle;
        }

        if self.intr_cycle {
            trb.control |= TRB_CYCLE;
        } else {
            trb.control &= !TRB_CYCLE;
        }
        ptr::write_volatile(ptr::addr_of_mut!(INTR_RING.0[self.intr_enqueue]), trb);
        self.intr_enqueue += 1;
        Ok(())
    }

    unsafe fn poll_keyboard<F: FnMut(u16, bool)>(&mut self, f: &mut F) {
        let Some(mut keyboard) = self.keyboard else {
            return;
        };

        let mut processed = 0usize;
        while processed < 16 {
            let Some(event) = self.poll_event() else {
                break;
            };
            if event.trb_type() == TRB_TYPE_TRANSFER_EVENT
                && event.slot_id() == keyboard.slot_id
                && event.endpoint_id() == keyboard.dci
            {
                let cc = event.completion_code();
                if cc == CC_SUCCESS || cc == CC_SHORT_PACKET {
                    let report = KEYBOARD_REPORT.0;
                    emit_keyboard_report_changes(&mut keyboard.previous_report, &report, f);
                } else {
                    serial::write_fmt(format_args!("usb-hid: interrupt completion {}\r\n", cc));
                }
                let _ = self.queue_keyboard_report();
                self.keyboard = Some(keyboard);
            }
            processed += 1;
        }
    }

    unsafe fn ring_doorbell(&self, slot_id: u8, target: u8) {
        let offset = self.doorbell_offset + slot_id as usize * 4;
        write32(self.base, offset, target as u32);
    }

    fn portsc_offset(&self, port_number: u8) -> usize {
        self.op_offset + OP_PORT_BASE + (port_number as usize - 1) * PORT_REGISTER_STRIDE
    }

    unsafe fn write_portsc(&self, offset: usize, set_bits: u32) {
        let current = read32(self.base, offset);
        write32(
            self.base,
            offset,
            (current & PORTSC_PRESERVE_MASK) | set_bits,
        );
    }

    unsafe fn reset_ep0_ring(&mut self) {
        ptr::write_bytes(
            ptr::addr_of_mut!(EP0_RING.0[0]).cast::<u8>(),
            0,
            core::mem::size_of::<Trb>() * EP0_RING_LEN,
        );
        self.ep0_enqueue = 0;
        self.ep0_cycle = true;
    }

    unsafe fn reset_interrupt_ring(&mut self) {
        ptr::write_bytes(
            ptr::addr_of_mut!(INTR_RING.0[0]).cast::<u8>(),
            0,
            core::mem::size_of::<Trb>() * INTR_RING_LEN,
        );
        self.intr_enqueue = 0;
        self.intr_cycle = true;
    }
}

unsafe fn zero_static_storage() {
    ptr::write_bytes(
        ptr::addr_of_mut!(COMMAND_RING.0[0]).cast::<u8>(),
        0,
        core::mem::size_of::<Trb>() * COMMAND_RING_LEN,
    );
    ptr::write_bytes(
        ptr::addr_of_mut!(EVENT_RING.0[0]).cast::<u8>(),
        0,
        core::mem::size_of::<Trb>() * EVENT_RING_LEN,
    );
    ptr::write_bytes(
        ptr::addr_of_mut!(ERST).cast::<u8>(),
        0,
        core::mem::size_of::<Erst>(),
    );
    ptr::write_bytes(
        ptr::addr_of_mut!(DCBAA).cast::<u8>(),
        0,
        core::mem::size_of::<Dcbaa>(),
    );
    zero_contexts();
    ptr::write_bytes(CONTROL_BUFFER.0.as_mut_ptr(), 0, CONTROL_BUFFER_LEN);
    ptr::write_bytes(KEYBOARD_REPORT.0.as_mut_ptr(), 0, KEYBOARD_REPORT_LEN);
}

unsafe fn setup_scratchpads(count: usize) -> Result<(), &'static str> {
    if count == 0 {
        return Ok(());
    }
    let mut index = 0usize;
    while index < count {
        ptr::write_bytes(SCRATCHPAD_PAGES[index].0.as_mut_ptr(), 0, 4096);
        let phys = phys_of(
            ptr::addr_of!(SCRATCHPAD_PAGES[index].0[0]),
            "scratchpad page phys",
        )?;
        ptr::write_volatile(
            SCRATCHPAD_ARRAY.0.as_mut_ptr().add(index * 8).cast::<u64>(),
            phys,
        );
        index += 1;
    }
    DCBAA.0[0] = phys_of(
        ptr::addr_of!(SCRATCHPAD_ARRAY.0[0]),
        "scratchpad array phys",
    )?;
    Ok(())
}

unsafe fn zero_contexts() {
    ptr::write_bytes(INPUT_CONTEXT.0.as_mut_ptr(), 0, CONTEXT_BYTES);
    ptr::write_bytes(DEVICE_CONTEXT.0.as_mut_ptr(), 0, CONTEXT_BYTES);
}

unsafe fn clear_input_context() {
    ptr::write_bytes(INPUT_CONTEXT.0.as_mut_ptr(), 0, CONTEXT_BYTES);
}

unsafe fn input_control_add_flags(flags: u32) {
    ctx_write_raw(ptr::addr_of_mut!(INPUT_CONTEXT.0[0]), 1, flags);
}

unsafe fn write_slot_context(port: PortInfo, context_entries: u8, context_size: usize) {
    let slot = input_context_ptr(context_size, 0);
    let dword0 = ((port.speed as u32) << 20) | ((context_entries as u32) << 27);
    let dword1 = (port.port_number as u32) << 16;
    ctx_write_raw(slot, 0, dword0);
    ctx_write_raw(slot, 1, dword1);
}

unsafe fn write_endpoint_context(
    dci: u8,
    context_size: usize,
    ep_type: u32,
    interval: u8,
    max_packet_size: u16,
    dequeue_ptr: u64,
    average_trb_length: u16,
    max_esit_payload: u16,
) {
    let ep = input_context_ptr(context_size, dci as usize);
    let dword0 = (interval as u32) << 16;
    let dword1 = (3 << 1) | (ep_type << 3) | ((max_packet_size as u32) << 16);
    let dword4 = average_trb_length as u32 | ((max_esit_payload as u32) << 16);
    ctx_write_raw(ep, 0, dword0);
    ctx_write_raw(ep, 1, dword1);
    ctx_write_raw(ep, 2, dequeue_ptr as u32);
    ctx_write_raw(ep, 3, (dequeue_ptr >> 32) as u32);
    ctx_write_raw(ep, 4, dword4);
}

unsafe fn input_context_ptr(context_size: usize, device_context_index: usize) -> *mut u8 {
    INPUT_CONTEXT
        .0
        .as_mut_ptr()
        .add((device_context_index + 1) * context_size)
}

unsafe fn ctx_write_raw(base: *mut u8, dword: usize, value: u32) {
    ptr::write_volatile(base.add(dword * 4).cast::<u32>(), value);
}

unsafe fn parse_keyboard_endpoint(config: &[u8]) -> Result<KeyboardEndpoint, &'static str> {
    if config.len() < 9 {
        return Err("configuration descriptor too short");
    }

    let mut configuration_value = 0u8;
    let mut current_interface = 0u8;
    let mut current_is_keyboard = false;
    let mut offset = 0usize;

    while offset + 2 <= config.len() {
        let len = config[offset] as usize;
        let dtype = config[offset + 1];
        if len < 2 || offset + len > config.len() {
            break;
        }

        match dtype {
            2 if len >= 9 => {
                configuration_value = config[offset + 5];
            }
            4 if len >= 9 => {
                current_interface = config[offset + 2];
                let class = config[offset + 5];
                let subclass = config[offset + 6];
                let protocol = config[offset + 7];
                current_is_keyboard = class == 0x03 && subclass == 0x01 && protocol == 0x01;
                if current_is_keyboard {
                    serial::write_fmt(format_args!(
                        "usb-hid: boot keyboard interface {}\r\n",
                        current_interface
                    ));
                }
            }
            5 if len >= 7 && current_is_keyboard => {
                let endpoint_address = config[offset + 2];
                let attributes = config[offset + 3] & 0x03;
                if endpoint_address & 0x80 != 0 && attributes == 0x03 {
                    let max_packet_size =
                        u16::from_le_bytes([config[offset + 4], config[offset + 5]]) & 0x07FF;
                    let endpoint_number = endpoint_address & 0x0F;
                    let dci = endpoint_number * 2 + 1;
                    return Ok(KeyboardEndpoint {
                        interface_number: current_interface,
                        configuration_value,
                        endpoint_address,
                        dci,
                        max_packet_size: u16::max(max_packet_size, KEYBOARD_REPORT_LEN as u16),
                        interval: config[offset + 6],
                    });
                }
            }
            _ => {}
        }

        offset += len;
    }

    Err("boot keyboard interrupt endpoint not found")
}

fn emit_keyboard_report_changes<F: FnMut(u16, bool)>(
    previous: &mut [u8; KEYBOARD_REPORT_LEN],
    current: &[u8; KEYBOARD_REPORT_LEN],
    f: &mut F,
) {
    let modifier_codes = [29, 42, 56, 125, 97, 54, 100, 126];
    let old_mods = previous[0];
    let new_mods = current[0];
    let mut bit = 0usize;
    while bit < modifier_codes.len() {
        let mask = 1u8 << bit;
        if old_mods & mask == 0 && new_mods & mask != 0 {
            f(modifier_codes[bit], true);
        } else if old_mods & mask != 0 && new_mods & mask == 0 {
            f(modifier_codes[bit], false);
        }
        bit += 1;
    }

    let mut idx = 2usize;
    while idx < KEYBOARD_REPORT_LEN {
        let usage = previous[idx];
        if usage >= 4 && !report_contains(current, usage) {
            if let Some(code) = hid_usage_to_keycode(usage) {
                f(code, false);
            }
        }
        idx += 1;
    }

    idx = 2;
    while idx < KEYBOARD_REPORT_LEN {
        let usage = current[idx];
        if usage >= 4 && !report_contains(previous, usage) {
            if let Some(code) = hid_usage_to_keycode(usage) {
                f(code, true);
            }
        }
        idx += 1;
    }

    *previous = *current;
}

fn report_contains(report: &[u8; KEYBOARD_REPORT_LEN], usage: u8) -> bool {
    let mut idx = 2usize;
    while idx < KEYBOARD_REPORT_LEN {
        if report[idx] == usage {
            return true;
        }
        idx += 1;
    }
    false
}

fn hid_usage_to_keycode(usage: u8) -> Option<u16> {
    let code = match usage {
        0x04 => 30,
        0x05 => 48,
        0x06 => 46,
        0x07 => 32,
        0x08 => 18,
        0x09 => 33,
        0x0A => 34,
        0x0B => 35,
        0x0C => 23,
        0x0D => 36,
        0x0E => 37,
        0x0F => 38,
        0x10 => 50,
        0x11 => 49,
        0x12 => 24,
        0x13 => 25,
        0x14 => 16,
        0x15 => 19,
        0x16 => 31,
        0x17 => 20,
        0x18 => 22,
        0x19 => 47,
        0x1A => 17,
        0x1B => 45,
        0x1C => 21,
        0x1D => 44,
        0x1E => 2,
        0x1F => 3,
        0x20 => 4,
        0x21 => 5,
        0x22 => 6,
        0x23 => 7,
        0x24 => 8,
        0x25 => 9,
        0x26 => 10,
        0x27 => 11,
        0x28 => 28,
        0x29 => 1,
        0x2A => 14,
        0x2B => 15,
        0x2C => 57,
        0x2D => 12,
        0x2E => 13,
        0x2F => 26,
        0x30 => 27,
        0x31 => 43,
        0x33 => 39,
        0x34 => 40,
        0x35 => 41,
        0x36 => 51,
        0x37 => 52,
        0x38 => 53,
        _ => return None,
    };
    Some(code)
}

unsafe fn legacy_handoff(base: *mut u8, hccparams1: u32, map_len: usize) {
    let mut offset = (((hccparams1 >> 16) & 0xFFFF) as usize) << 2;
    let mut guard = 0usize;
    while offset != 0 && offset + 4 <= map_len && guard < 64 {
        let cap = read32(base, offset);
        let cap_id = cap & 0xFF;
        let next = ((cap >> 8) & 0xFF) as usize;
        if cap_id == 1 {
            if cap & (1 << 16) != 0 {
                write32(base, offset, cap | (1 << 24));
                let _ = wait_for(WAIT_ITERS / 4, || {
                    let current = read32(base, offset);
                    current & (1 << 16) == 0
                });
            }
            break;
        }
        if next == 0 {
            break;
        }
        offset += next << 2;
        guard += 1;
    }
}

unsafe fn stop_and_reset(base: *mut u8, op_offset: usize) -> Result<(), &'static str> {
    let cmd = read32(base, op_offset + OP_USBCMD) & !USBCMD_RUN_STOP;
    write32(base, op_offset + OP_USBCMD, cmd);
    wait_for(WAIT_ITERS, || {
        read32(base, op_offset + OP_USBSTS) & USBSTS_HALTED != 0
    })
    .ok_or("usb-xhci: controller did not halt")?;

    write32(
        base,
        op_offset + OP_USBCMD,
        read32(base, op_offset + OP_USBCMD) | USBCMD_HOST_CONTROLLER_RESET,
    );
    wait_for(WAIT_ITERS, || {
        read32(base, op_offset + OP_USBCMD) & USBCMD_HOST_CONTROLLER_RESET == 0
    })
    .ok_or("usb-xhci: controller reset stuck")?;
    wait_for(WAIT_ITERS, || {
        read32(base, op_offset + OP_USBSTS) & USBSTS_CONTROLLER_NOT_READY == 0
    })
    .ok_or("usb-xhci: controller not ready")?;
    Ok(())
}

unsafe fn start_controller(base: *mut u8, op_offset: usize) -> Result<(), &'static str> {
    write32(
        base,
        op_offset + OP_USBCMD,
        read32(base, op_offset + OP_USBCMD) | USBCMD_RUN_STOP | USBCMD_INTERRUPTER_ENABLE,
    );
    wait_for(WAIT_ITERS, || {
        read32(base, op_offset + OP_USBSTS) & USBSTS_HALTED == 0
    })
    .ok_or("usb-xhci: controller did not start")?;
    Ok(())
}

fn max_scratchpads(hcsparams2: u32) -> usize {
    let lo = (hcsparams2 >> 27) & 0x1F;
    let hi = (hcsparams2 >> 21) & 0x1F;
    ((hi << 5) | lo) as usize
}

fn default_ep0_mps(speed: u8) -> u16 {
    match speed {
        4 => 512,
        3 => 64,
        _ => 8,
    }
}

fn descriptor_ep0_mps(speed: u8, raw: u8) -> u16 {
    if speed == 4 {
        1u16.checked_shl(raw as u32).unwrap_or(512)
    } else {
        raw as u16
    }
}

fn interval_to_xhci(speed: u8, interval: u8) -> u8 {
    if interval == 0 {
        return 0;
    }
    if speed <= 2 {
        let microframes = (interval as u16).saturating_mul(8).max(1);
        let mut value = 0u8;
        let mut period = 1u16;
        while period < microframes && value < 15 {
            period <<= 1;
            value += 1;
        }
        value
    } else {
        interval.saturating_sub(1).min(15)
    }
}

fn trb_type(trb_type: u32) -> u32 {
    trb_type << TRB_TYPE_SHIFT
}

fn wait_for<F: Fn() -> bool>(limit: usize, condition: F) -> Option<()> {
    let mut waited = 0usize;
    while waited < limit {
        if condition() {
            return Some(());
        }
        spin_loop();
        waited += 1;
    }
    None
}

fn phys_of<T>(ptr: *const T, label: &'static str) -> Result<u64, &'static str> {
    memory::virt_to_phys(ptr).ok_or(label)
}

unsafe fn read32(base: *mut u8, offset: usize) -> u32 {
    ptr::read_volatile(base.add(offset).cast::<u32>())
}

unsafe fn write32(base: *mut u8, offset: usize, value: u32) {
    ptr::write_volatile(base.add(offset).cast::<u32>(), value);
}

unsafe fn write64(base: *mut u8, offset: usize, value: u64) {
    ptr::write_volatile(base.add(offset).cast::<u64>(), value);
}
