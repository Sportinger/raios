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
    (1 << 17) | (1 << 18) | (1 << 19) | (1 << 20) | (1 << 21) | (1 << 22) | (1 << 23) | (1 << 24);
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
const TRB_TYPE_EVALUATE_CONTEXT: u32 = 13;
const TRB_TYPE_RESET_ENDPOINT: u32 = 14;
const TRB_TYPE_SET_TR_DEQUEUE_POINTER: u32 = 16;
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
const USB_REQ_CLEAR_FEATURE: u8 = 1;
const USB_REQ_GET_STATUS: u8 = 0;
const USB_REQ_SET_FEATURE: u8 = 3;
const USB_REQ_SET_CONFIGURATION: u8 = 9;
const HID_REQ_SET_IDLE: u8 = 10;
const HID_REQ_SET_PROTOCOL: u8 = 11;
const DESC_DEVICE: u8 = 1;
const DESC_CONFIGURATION: u8 = 2;
const DESC_HUB: u8 = 0x29;
const DESC_SUPERSPEED_HUB: u8 = 0x2A;
const USB_CLASS_HUB: u8 = 0x09;
const HUB_PORT_CONNECTION: u16 = 1 << 0;
const HUB_PORT_ENABLED: u16 = 1 << 1;
const HUB_PORT_RESET: u16 = 1 << 4;
const HUB_PORT_LOW_SPEED: u16 = 1 << 9;
const HUB_PORT_HIGH_SPEED: u16 = 1 << 10;
const HUB_FEATURE_PORT_RESET: u16 = 4;
const HUB_FEATURE_PORT_POWER: u16 = 8;
const HUB_FEATURE_C_PORT_CONNECTION: u16 = 16;
const HUB_FEATURE_C_PORT_ENABLE: u16 = 17;
const HUB_FEATURE_C_PORT_RESET: u16 = 20;
const HUB_CHANGE_C_PORT_CONNECTION: u16 = 1 << 0;
const HUB_CHANGE_C_PORT_RESET: u16 = 1 << 4;

const COMMAND_RING_LEN: usize = 64;
const EVENT_RING_LEN: usize = 64;
const EP0_RING_LEN: usize = 64;
const INTR_RING_LEN: usize = 16;
const MAX_HID_DEVICES: usize = 16;
const MAX_SLOTS: usize = 32;
const MAX_SCRATCHPADS: usize = 64;
const CONTEXT_BYTES: usize = 4096;
const CONTROL_BUFFER_LEN: usize = 256;
const KEYBOARD_REPORT_LEN: usize = 8;
const MOUSE_REPORT_LEN: usize = 4;
const TABLET_REPORT_LEN: usize = 8;
const POINTER_REPORT_BUFFER_LEN: usize = TABLET_REPORT_LEN;
const TABLET_AXIS_MAX: u16 = 0x7fff;
const WAIT_ITERS: usize = 5_000_000;
const ROOT_PORT_POWER_SETTLE_ITERS: usize = WAIT_ITERS;
const HUB_PORT_RESET_POLLS: usize = 64;
const HUB_PORT_RESET_POLL_DELAY_ITERS: usize = WAIT_ITERS / 100;

static STATE: Mutex<UsbState> = Mutex::new(UsbState::new());

#[derive(Clone, Copy)]
pub struct UsbSnapshot {
    pub state: UsbStatus,
    pub address: Option<PciAddress>,
    pub hci_version: u16,
    pub max_ports: u8,
    pub powered_ports: u8,
    pub connected_ports: u8,
    pub hub_count: u8,
    pub hub_ports: u8,
    pub hub_connected_ports: u8,
    pub hub_reset_ports: u8,
    pub hub_configured_devices: u8,
    pub hub_last_error: Option<&'static str>,
    pub last_command_type: u8,
    pub last_completion_code: u8,
    pub last_transfer_completion_code: u8,
    pub input_report_count: u32,
    pub input_error_count: u32,
    pub keyboard_status: UsbKeyboardStatus,
    pub keyboard_detail: Option<&'static str>,
    pub mouse_status: UsbMouseStatus,
    pub mouse_detail: Option<&'static str>,
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UsbMouseStatus {
    NotProbed,
    Ready,
    NotFound,
    Error,
}

#[derive(Clone, Copy)]
pub struct UsbMouseReport {
    pub buttons: u8,
    pub dx: i8,
    pub dy: i8,
    pub wheel: i8,
    pub absolute: bool,
    pub x: u16,
    pub y: u16,
    pub max_x: u16,
    pub max_y: u16,
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
                powered_ports: 0,
                connected_ports: 0,
                hub_count: 0,
                hub_ports: 0,
                hub_connected_ports: 0,
                hub_reset_ports: 0,
                hub_configured_devices: 0,
                hub_last_error: None,
                last_command_type: 0,
                last_completion_code: 0,
                last_transfer_completion_code: 0,
                input_report_count: 0,
                input_error_count: 0,
                keyboard_status: UsbKeyboardStatus::NotProbed,
                keyboard_detail: None,
                mouse_status: UsbMouseStatus::NotProbed,
                mouse_detail: None,
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

pub fn rescan_if_input_missing() -> bool {
    if keyboard_active() || mouse_active() {
        return false;
    }

    serial::write_line("usb-hotplug: rescanning xHCI input devices");
    let (snapshot, controller) = unsafe { probe_xhci() };
    let input_ready = controller
        .as_ref()
        .is_some_and(|controller| controller.keyboard.is_some() || controller.mouse.is_some());
    *STATE.lock() = UsbState {
        snapshot,
        controller,
    };
    if input_ready {
        serial::write_line("usb-hotplug: input device configured");
    }
    true
}

pub fn snapshot() -> UsbSnapshot {
    let state = STATE.lock();
    let mut snapshot = state.snapshot;
    if let Some(controller) = state.controller.as_ref() {
        snapshot.last_command_type = controller.last_command_type;
        snapshot.last_completion_code = controller.last_completion_code;
        snapshot.last_transfer_completion_code = controller.last_transfer_completion_code;
        snapshot.input_report_count = controller
            .keyboard_report_count
            .saturating_add(controller.mouse_report_count);
        snapshot.input_error_count = controller.transfer_error_count;
    }
    snapshot
}

pub fn keyboard_active() -> bool {
    STATE
        .lock()
        .controller
        .as_ref()
        .is_some_and(|controller| controller.keyboard.is_some())
}

pub fn mouse_active() -> bool {
    STATE
        .lock()
        .controller
        .as_ref()
        .is_some_and(|controller| controller.mouse.is_some())
}

pub fn input_active() -> bool {
    keyboard_active() || mouse_active()
}

pub fn input_detail() -> &'static str {
    match (keyboard_active(), mouse_active()) {
        (true, true) => "USB HID KEYBOARD + POINTER",
        (true, false) => "USB HID BOOT KEYBOARD",
        (false, true) => "USB HID BOOT MOUSE",
        (false, false) => "USB HID ABSENT",
    }
}

pub fn poll_input<K, M>(mut keyboard: K, mut mouse: M)
where
    K: FnMut(u16, bool),
    M: FnMut(UsbMouseReport),
{
    let mut state = STATE.lock();
    if let Some(controller) = state.controller.as_mut() {
        unsafe {
            controller.poll_input(&mut keyboard, &mut mouse);
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
                powered_ports: 0,
                connected_ports: 0,
                hub_count: 0,
                hub_ports: 0,
                hub_connected_ports: 0,
                hub_reset_ports: 0,
                hub_configured_devices: 0,
                hub_last_error: None,
                last_command_type: 0,
                last_completion_code: 0,
                last_transfer_completion_code: 0,
                input_report_count: 0,
                input_error_count: 0,
                keyboard_status: UsbKeyboardStatus::NotProbed,
                keyboard_detail: None,
                mouse_status: UsbMouseStatus::NotProbed,
                mouse_detail: None,
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
            controller.power_root_ports();
            let mut connected_ports = controller.count_connected_ports();
            serial::write_fmt(format_args!(
                "usb-xhci: hci 0x{:04x}, ports {}, powered {}, connected {}\r\n",
                controller.hci_version,
                controller.max_ports,
                controller.powered_ports,
                connected_ports
            ));

            let hid_result = controller.initialise_hid_devices();
            if let Err(err) = hid_result {
                serial::write_fmt(format_args!("usb-hid: {}\r\n", err));
            }
            connected_ports = controller.count_connected_ports();
            let keyboard_status = if controller.keyboard.is_some() {
                UsbKeyboardStatus::Ready
            } else {
                UsbKeyboardStatus::NotFound
            };
            let mouse_status = if controller.mouse.is_some() {
                UsbMouseStatus::Ready
            } else {
                UsbMouseStatus::NotFound
            };
            let keyboard_detail = if controller.keyboard.is_some() {
                Some("USB HID BOOT KEYBOARD")
            } else {
                Some("no USB boot keyboard found")
            };
            let mouse_detail = if controller.mouse.is_some() {
                Some("USB HID BOOT MOUSE")
            } else {
                Some("no USB pointer found")
            };

            let snapshot = UsbSnapshot {
                state: UsbStatus::Ready,
                address: Some(address),
                hci_version: controller.hci_version,
                max_ports: controller.max_ports,
                powered_ports: controller.powered_ports,
                connected_ports,
                hub_count: controller.hub_count,
                hub_ports: controller.hub_ports,
                hub_connected_ports: controller.hub_connected_ports,
                hub_reset_ports: controller.hub_reset_ports,
                hub_configured_devices: controller.hub_configured_devices,
                hub_last_error: controller.hub_last_error,
                last_command_type: controller.last_command_type,
                last_completion_code: controller.last_completion_code,
                last_transfer_completion_code: controller.last_transfer_completion_code,
                input_report_count: controller
                    .keyboard_report_count
                    .saturating_add(controller.mouse_report_count),
                input_error_count: controller.transfer_error_count,
                keyboard_status,
                keyboard_detail,
                mouse_status,
                mouse_detail,
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
        powered_ports: 0,
        connected_ports: 0,
        hub_count: 0,
        hub_ports: 0,
        hub_connected_ports: 0,
        hub_reset_ports: 0,
        hub_configured_devices: 0,
        hub_last_error: None,
        last_command_type: 0,
        last_completion_code: 0,
        last_transfer_completion_code: 0,
        input_report_count: 0,
        input_error_count: 0,
        keyboard_status: UsbKeyboardStatus::Error,
        keyboard_detail: Some(error),
        mouse_status: UsbMouseStatus::Error,
        mouse_detail: Some(error),
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
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
struct AlignedBytes<const N: usize>([u8; N]);

#[repr(C, align(4096))]
#[derive(Clone, Copy)]
struct ScratchPage([u8; 4096]);

static mut COMMAND_RING: TrbRing<COMMAND_RING_LEN> = TrbRing([Trb::zero(); COMMAND_RING_LEN]);
static mut EVENT_RING: TrbRing<EVENT_RING_LEN> = TrbRing([Trb::zero(); EVENT_RING_LEN]);
static mut EP0_RINGS: [TrbRing<EP0_RING_LEN>; MAX_HID_DEVICES] =
    [TrbRing([Trb::zero(); EP0_RING_LEN]); MAX_HID_DEVICES];
static mut INTR_RINGS: [TrbRing<INTR_RING_LEN>; MAX_HID_DEVICES] =
    [TrbRing([Trb::zero(); INTR_RING_LEN]); MAX_HID_DEVICES];
static mut ERST: Erst = Erst([ErstEntry::zero(); 1]);
static mut DCBAA: Dcbaa = Dcbaa([0; 256]);
static mut SCRATCHPAD_ARRAY: AlignedBytes<{ MAX_SCRATCHPADS * 8 }> =
    AlignedBytes([0; MAX_SCRATCHPADS * 8]);
static mut SCRATCHPAD_PAGES: [ScratchPage; MAX_SCRATCHPADS] =
    [ScratchPage([0; 4096]); MAX_SCRATCHPADS];
static mut INPUT_CONTEXT: AlignedBytes<CONTEXT_BYTES> = AlignedBytes([0; CONTEXT_BYTES]);
static mut DEVICE_CONTEXTS: [AlignedBytes<CONTEXT_BYTES>; MAX_HID_DEVICES] =
    [AlignedBytes([0; CONTEXT_BYTES]); MAX_HID_DEVICES];
static mut CONTROL_BUFFER: AlignedBytes<CONTROL_BUFFER_LEN> = AlignedBytes([0; CONTROL_BUFFER_LEN]);
static mut KEYBOARD_REPORT: AlignedBytes<KEYBOARD_REPORT_LEN> =
    AlignedBytes([0; KEYBOARD_REPORT_LEN]);
static mut MOUSE_REPORT: AlignedBytes<POINTER_REPORT_BUFFER_LEN> =
    AlignedBytes([0; POINTER_REPORT_BUFFER_LEN]);

struct XhciController {
    _address: PciAddress,
    _mapping: memory::MmioMapping,
    base: *mut u8,
    _cap_length: usize,
    hci_version: u16,
    _max_slots: u8,
    max_ports: u8,
    powered_ports: u8,
    context_size: usize,
    op_offset: usize,
    runtime_offset: usize,
    doorbell_offset: usize,
    command_enqueue: usize,
    command_cycle: bool,
    event_dequeue: usize,
    event_cycle: bool,
    ep0_enqueue: [usize; MAX_HID_DEVICES],
    ep0_cycle: [bool; MAX_HID_DEVICES],
    intr_enqueue: [usize; MAX_HID_DEVICES],
    intr_cycle: [bool; MAX_HID_DEVICES],
    event_ring_phys: u64,
    current_device_index: usize,
    current_slot_id: u8,
    keyboard: Option<KeyboardDevice>,
    mouse: Option<MouseDevice>,
    hub_count: u8,
    hub_ports: u8,
    hub_connected_ports: u8,
    hub_reset_ports: u8,
    hub_configured_devices: u8,
    hub_last_error: Option<&'static str>,
    last_command_type: u8,
    last_completion_code: u8,
    last_transfer_completion_code: u8,
    keyboard_report_count: u32,
    mouse_report_count: u32,
    transfer_error_count: u32,
}

unsafe impl Send for XhciController {}
unsafe impl Sync for XhciController {}

#[derive(Clone, Copy)]
struct KeyboardDevice {
    slot_id: u8,
    dci: u8,
    ring_index: usize,
    previous_report: [u8; KEYBOARD_REPORT_LEN],
}

#[derive(Clone, Copy)]
struct MouseDevice {
    slot_id: u8,
    dci: u8,
    ring_index: usize,
    kind: HidKind,
}

#[derive(Clone, Copy)]
struct PortInfo {
    root_port_number: u8,
    route_string: u32,
    speed: u8,
    parent_hub_slot_id: u8,
    parent_hub_port_number: u8,
    is_hub: bool,
    hub_port_count: u8,
}

impl PortInfo {
    fn root(port_number: u8, speed: u8) -> Self {
        Self {
            root_port_number: port_number,
            route_string: 0,
            speed,
            parent_hub_slot_id: 0,
            parent_hub_port_number: 0,
            is_hub: false,
            hub_port_count: 0,
        }
    }

    fn hub_context(mut self, hub_port_count: u8) -> Self {
        self.is_hub = true;
        self.hub_port_count = hub_port_count;
        self
    }

    fn child(self, hub_slot_id: u8, hub_port_number: u8, speed: u8) -> Self {
        Self {
            root_port_number: self.root_port_number,
            route_string: child_route_string(self.route_string, hub_port_number),
            speed,
            parent_hub_slot_id: hub_slot_id,
            parent_hub_port_number: hub_port_number,
            is_hub: false,
            hub_port_count: 0,
        }
    }
}

#[derive(Clone, Copy)]
enum EnumeratedKind {
    Hid(HidKind),
    Hub,
}

impl EnumeratedKind {
    fn as_str(self) -> &'static str {
        match self {
            EnumeratedKind::Hid(kind) => kind.as_str(),
            EnumeratedKind::Hub => "hub",
        }
    }
}

#[derive(Clone, Copy)]
struct HidEndpoint {
    kind: HidKind,
    interface_number: u8,
    configuration_value: u8,
    endpoint_address: u8,
    dci: u8,
    max_packet_size: u16,
    interval: u8,
}

#[derive(Clone, Copy)]
struct HubDescriptor {
    port_count: u8,
    power_on_delay_ms: u16,
}

#[derive(Clone, Copy)]
struct HubPortStatus {
    status: u16,
    change: u16,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HidKind {
    Keyboard,
    GenericKeyboard,
    Mouse,
    Tablet,
}

impl HidKind {
    fn as_str(self) -> &'static str {
        match self {
            HidKind::Keyboard => "keyboard",
            HidKind::GenericKeyboard => "generic keyboard",
            HidKind::Mouse => "mouse",
            HidKind::Tablet => "tablet",
        }
    }

    fn report_len(self) -> usize {
        match self {
            HidKind::Keyboard | HidKind::GenericKeyboard => KEYBOARD_REPORT_LEN,
            HidKind::Mouse => MOUSE_REPORT_LEN,
            HidKind::Tablet => TABLET_REPORT_LEN,
        }
    }

    fn is_keyboard(self) -> bool {
        matches!(self, HidKind::Keyboard | HidKind::GenericKeyboard)
    }

    fn is_pointer(self) -> bool {
        matches!(self, HidKind::Mouse | HidKind::Tablet)
    }

    fn uses_boot_protocol(self) -> bool {
        matches!(self, HidKind::Keyboard | HidKind::Mouse)
    }
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
            powered_ports: 0,
            context_size,
            op_offset,
            runtime_offset,
            doorbell_offset,
            command_enqueue: 0,
            command_cycle: true,
            event_dequeue: 0,
            event_cycle: true,
            ep0_enqueue: [0; MAX_HID_DEVICES],
            ep0_cycle: [true; MAX_HID_DEVICES],
            intr_enqueue: [0; MAX_HID_DEVICES],
            intr_cycle: [true; MAX_HID_DEVICES],
            event_ring_phys,
            current_device_index: 0,
            current_slot_id: 0,
            keyboard: None,
            mouse: None,
            hub_count: 0,
            hub_ports: 0,
            hub_connected_ports: 0,
            hub_reset_ports: 0,
            hub_configured_devices: 0,
            hub_last_error: None,
            last_command_type: 0,
            last_completion_code: 0,
            last_transfer_completion_code: 0,
            keyboard_report_count: 0,
            mouse_report_count: 0,
            transfer_error_count: 0,
        })
    }

    unsafe fn power_root_ports(&mut self) {
        let mut requested = 0u8;
        let mut port = 1u8;
        while port <= self.max_ports {
            let offset = self.portsc_offset(port);
            let status = read32(self.base, offset);
            if status & PORTSC_PP == 0 {
                self.write_portsc(offset, PORTSC_PP);
                requested = requested.saturating_add(1);
            }
            port += 1;
        }

        if requested > 0 {
            spin_delay(ROOT_PORT_POWER_SETTLE_ITERS);
        }

        let mut powered = 0u8;
        port = 1;
        while port <= self.max_ports {
            let offset = self.portsc_offset(port);
            let status = read32(self.base, offset);
            if status & PORTSC_PP != 0 {
                powered = powered.saturating_add(1);
            }
            if status & PORTSC_CHANGE_BITS != 0 {
                self.write_portsc(offset, PORTSC_CHANGE_BITS | (status & PORTSC_PP));
            }
            port += 1;
        }
        self.powered_ports = powered;
        serial::write_fmt(format_args!(
            "usb-xhci: root ports powered {}/{} requested {}\r\n",
            powered, self.max_ports, requested
        ));
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

    unsafe fn initialise_hid_devices(&mut self) -> Result<(), &'static str> {
        self.power_root_ports();
        let mut wait_round = 0usize;
        while self.count_connected_ports() == 0 && wait_round < 3 {
            spin_delay(ROOT_PORT_POWER_SETTLE_ITERS);
            wait_round += 1;
        }

        let mut next_device_index = 0usize;
        let mut port = 1u8;
        while port <= self.max_ports {
            let status = read32(self.base, self.portsc_offset(port));
            if status & PORTSC_CCS == 0 {
                port += 1;
                continue;
            }
            if next_device_index >= MAX_HID_DEVICES {
                serial::write_line("usb-hid: device storage exhausted");
                break;
            }

            match self.reset_port(port) {
                Ok(port_info) => match self.enumerate_device(port_info, &mut next_device_index) {
                    Ok(kind) => {
                        serial::write_fmt(format_args!(
                            "usb-hid: port {} configured as {}\r\n",
                            port,
                            kind.as_str()
                        ));
                    }
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

        if self.keyboard.is_none() && self.mouse.is_none() {
            return Err("no USB HID input devices found");
        }
        let _ = self.queue_keyboard_report();
        let _ = self.queue_mouse_report();
        Ok(())
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
        Ok(PortInfo::root(port_number, speed))
    }

    unsafe fn enumerate_device(
        &mut self,
        port: PortInfo,
        next_device_index: &mut usize,
    ) -> Result<EnumeratedKind, &'static str> {
        if *next_device_index >= MAX_HID_DEVICES {
            return Err("device storage exhausted");
        }
        let device_index = *next_device_index;
        *next_device_index += 1;

        let slot_id = self.enable_slot()?;
        self.current_device_index = device_index;
        self.current_slot_id = slot_id;
        self.reset_ep0_ring();
        self.prepare_address_context(device_index, slot_id, port)?;
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

        if device_desc[4] == USB_CLASS_HUB || self.configuration_has_hub_interface()? {
            self.configure_hub(port, slot_id, device_index, next_device_index)?;
            return Ok(EnumeratedKind::Hub);
        }

        let endpoint = self.find_boot_hid_endpoint()?;
        if endpoint.kind.is_keyboard() && self.keyboard.is_some() {
            return Err("duplicate USB boot keyboard");
        }
        if endpoint.kind.is_pointer() && self.mouse.is_some() {
            return Err("duplicate USB pointer");
        }
        self.control_no_data(
            0x00,
            USB_REQ_SET_CONFIGURATION,
            endpoint.configuration_value as u16,
            0,
        )?;
        if endpoint.kind.uses_boot_protocol() {
            self.control_no_data(
                0x21,
                HID_REQ_SET_PROTOCOL,
                0,
                endpoint.interface_number as u16,
            )?;
        }
        self.control_no_data(0x21, HID_REQ_SET_IDLE, 0, endpoint.interface_number as u16)?;

        self.configure_interrupt_endpoint(device_index, slot_id, port, endpoint)?;
        serial::write_fmt(format_args!(
            "usb-hid: {} ready on slot {} endpoint 0x{:02x}\r\n",
            endpoint.kind.as_str(),
            slot_id,
            endpoint.endpoint_address
        ));
        Ok(EnumeratedKind::Hid(endpoint.kind))
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
        device_index: usize,
        slot_id: u8,
        port: PortInfo,
    ) -> Result<(), &'static str> {
        clear_input_context();
        clear_device_context(device_index);
        let device_context_phys = phys_of(
            ptr::addr_of!(DEVICE_CONTEXTS[device_index].0[0]),
            "device context phys",
        )?;
        DCBAA.0[slot_id as usize] = device_context_phys;

        input_control_add_flags((1 << 0) | (1 << DCI_EP0));
        write_slot_context(port, CONTEXT_ENTRIES_EP0, self.context_size);
        write_endpoint_context(
            DCI_EP0,
            self.context_size,
            EP_TYPE_CONTROL,
            0,
            default_ep0_mps(port.speed),
            phys_of(ptr::addr_of!(EP0_RINGS[device_index].0[0]), "ep0 ring phys")? | 1,
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

    unsafe fn configuration_value(&mut self) -> Result<u8, &'static str> {
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
        Ok(CONTROL_BUFFER.0[5])
    }

    unsafe fn configuration_has_hub_interface(&mut self) -> Result<bool, &'static str> {
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

        let mut offset = 0usize;
        while offset + 2 <= actual_len {
            let len = CONTROL_BUFFER.0[offset] as usize;
            let dtype = CONTROL_BUFFER.0[offset + 1];
            if len < 2 || offset + len > actual_len {
                break;
            }
            if dtype == 4 && len >= 9 && CONTROL_BUFFER.0[offset + 5] == USB_CLASS_HUB {
                return Ok(true);
            }
            offset += len;
        }
        Ok(false)
    }

    unsafe fn configure_hub(
        &mut self,
        port: PortInfo,
        slot_id: u8,
        hub_device_index: usize,
        next_device_index: &mut usize,
    ) -> Result<(), &'static str> {
        let config_value = match self.configuration_value() {
            Ok(value) => value,
            Err(err) => {
                self.hub_last_error = Some(err);
                return Err(err);
            }
        };
        if let Err(err) =
            self.control_no_data(0x00, USB_REQ_SET_CONFIGURATION, config_value as u16, 0)
        {
            self.hub_last_error = Some(err);
            return Err(err);
        }
        let descriptor = match self.hub_descriptor() {
            Ok(descriptor) => descriptor,
            Err(err) => {
                self.hub_last_error = Some(err);
                return Err(err);
            }
        };
        self.hub_count = self.hub_count.saturating_add(1);
        self.hub_ports = self.hub_ports.saturating_add(descriptor.port_count);

        serial::write_fmt(format_args!(
            "usb-hub: slot {} ports {} pwr-good {}ms\r\n",
            slot_id, descriptor.port_count, descriptor.power_on_delay_ms
        ));

        if let Err(err) = self.evaluate_hub_slot(port, slot_id, descriptor.port_count) {
            self.hub_last_error = Some(err);
            return Err(err);
        }

        let mut hub_port = 1u8;
        while hub_port <= descriptor.port_count {
            self.current_slot_id = slot_id;
            self.current_device_index = hub_device_index;
            let _ = self.hub_set_feature(hub_port, HUB_FEATURE_PORT_POWER);
            spin_delay(usize::max(
                WAIT_ITERS / 50,
                descriptor.power_on_delay_ms as usize * 1000,
            ));

            let status = self.hub_port_status(hub_port)?;
            if status.change & HUB_CHANGE_C_PORT_CONNECTION != 0 {
                let _ = self.hub_clear_feature(hub_port, HUB_FEATURE_C_PORT_CONNECTION);
            }
            if status.status & HUB_PORT_CONNECTION == 0 {
                serial::write_fmt(format_args!("usb-hub: port {} empty\r\n", hub_port));
                hub_port += 1;
                continue;
            }
            self.hub_connected_ports = self.hub_connected_ports.saturating_add(1);
            if *next_device_index >= MAX_HID_DEVICES {
                serial::write_line("usb-hub: device storage exhausted");
                self.hub_last_error = Some("usb-hub: device storage exhausted");
                break;
            }

            match self.reset_hub_port(hub_port) {
                Ok(port_status) => {
                    self.hub_reset_ports = self.hub_reset_ports.saturating_add(1);
                    let primary_speed = hub_port_speed(port_status.status, port.speed);
                    let (speeds, speed_count) = child_speed_candidates(primary_speed);
                    let mut speed_index = 0usize;
                    let mut configured = false;
                    let mut last_err = None;

                    while speed_index < speed_count {
                        let child_speed = speeds[speed_index];
                        let child_port = port.child(slot_id, hub_port, child_speed);
                        serial::write_fmt(format_args!(
                            "usb-hub: port {} reset complete speed {} try {}\r\n",
                            hub_port,
                            child_speed,
                            speed_index + 1
                        ));
                        match self.enumerate_device(child_port, next_device_index) {
                            Ok(kind) => {
                                configured = true;
                                self.hub_configured_devices =
                                    self.hub_configured_devices.saturating_add(1);
                                serial::write_fmt(format_args!(
                                    "usb-hub: port {} configured as {}\r\n",
                                    hub_port,
                                    kind.as_str()
                                ));
                                break;
                            }
                            Err(err) => {
                                last_err = Some(err);
                                if !self.should_retry_child_enumeration(err) {
                                    break;
                                }
                                serial::write_fmt(format_args!(
                                    "usb-hub: port {} speed {} retry after {}\r\n",
                                    hub_port, child_speed, err
                                ));
                            }
                        }

                        speed_index += 1;
                    }

                    if !configured {
                        if let Some(err) = last_err {
                            self.hub_last_error = Some(err);
                            serial::write_fmt(format_args!(
                                "usb-hub: port {} skipped: {}\r\n",
                                hub_port, err
                            ));
                        }
                    }
                }
                Err(err) => {
                    self.hub_last_error = Some(err);
                    serial::write_fmt(format_args!(
                        "usb-hub: port {} reset: {}\r\n",
                        hub_port, err
                    ));
                }
            }

            hub_port += 1;
        }

        Ok(())
    }

    fn should_retry_child_enumeration(&self, err: &'static str) -> bool {
        err == "xHCI command failed"
            && self.last_command_type == TRB_TYPE_ADDRESS_DEVICE as u8
            && self.last_completion_code == 4
    }

    unsafe fn evaluate_hub_slot(
        &mut self,
        port: PortInfo,
        slot_id: u8,
        hub_port_count: u8,
    ) -> Result<(), &'static str> {
        clear_input_context();
        input_control_add_flags(1 << 0);
        write_slot_context(
            port.hub_context(hub_port_count),
            CONTEXT_ENTRIES_EP0,
            self.context_size,
        );
        let input_phys = phys_of(ptr::addr_of!(INPUT_CONTEXT.0[0]), "hub input context phys")?;
        self.execute_command(Trb {
            parameter: input_phys,
            status: 0,
            control: trb_type(TRB_TYPE_EVALUATE_CONTEXT) | ((slot_id as u32) << 24),
        })?;
        Ok(())
    }

    unsafe fn hub_descriptor(&mut self) -> Result<HubDescriptor, &'static str> {
        match self.hub_descriptor_of_type(DESC_HUB) {
            Ok(descriptor) => Ok(descriptor),
            Err(_) => self.hub_descriptor_of_type(DESC_SUPERSPEED_HUB),
        }
    }

    unsafe fn hub_descriptor_of_type(
        &mut self,
        descriptor_type: u8,
    ) -> Result<HubDescriptor, &'static str> {
        let len = self.control_in(
            0xA0,
            USB_REQ_GET_DESCRIPTOR,
            (descriptor_type as u16) << 8,
            0,
            12,
        )?;
        if len < 7 || CONTROL_BUFFER.0[1] != descriptor_type {
            return Err("hub descriptor unavailable");
        }
        let port_count = CONTROL_BUFFER.0[2];
        if port_count == 0 {
            return Err("hub reports zero ports");
        }
        Ok(HubDescriptor {
            port_count: u8::min(port_count, 8),
            power_on_delay_ms: (CONTROL_BUFFER.0[5] as u16).saturating_mul(2),
        })
    }

    unsafe fn hub_set_feature(&mut self, port: u8, feature: u16) -> Result<(), &'static str> {
        self.control_no_data(0x23, USB_REQ_SET_FEATURE, feature, port as u16)
    }

    unsafe fn hub_clear_feature(&mut self, port: u8, feature: u16) -> Result<(), &'static str> {
        self.control_no_data(0x23, USB_REQ_CLEAR_FEATURE, feature, port as u16)
    }

    unsafe fn hub_port_status(&mut self, port: u8) -> Result<HubPortStatus, &'static str> {
        let len = self.control_in(0xA3, USB_REQ_GET_STATUS, 0, port as u16, 4)?;
        if len < 4 {
            return Err("short hub port status");
        }
        Ok(HubPortStatus {
            status: u16::from_le_bytes([CONTROL_BUFFER.0[0], CONTROL_BUFFER.0[1]]),
            change: u16::from_le_bytes([CONTROL_BUFFER.0[2], CONTROL_BUFFER.0[3]]),
        })
    }

    unsafe fn reset_hub_port(&mut self, port: u8) -> Result<HubPortStatus, &'static str> {
        self.hub_set_feature(port, HUB_FEATURE_PORT_RESET)?;
        let mut polls = 0usize;
        let mut last_status = None;
        while polls < HUB_PORT_RESET_POLLS {
            spin_delay(HUB_PORT_RESET_POLL_DELAY_ITERS);
            let status = self.hub_port_status(port)?;
            last_status = Some(status);
            if status.change & HUB_CHANGE_C_PORT_RESET != 0
                || (status.status & HUB_PORT_RESET == 0 && polls >= 4)
            {
                let _ = self.hub_clear_feature(port, HUB_FEATURE_C_PORT_RESET);
                let _ = self.hub_clear_feature(port, HUB_FEATURE_C_PORT_ENABLE);
                if status.status & HUB_PORT_ENABLED == 0 {
                    return Err("hub port not enabled after reset");
                }
                return Ok(status);
            }
            polls += 1;
        }

        if let Some(status) = last_status {
            if status.status & HUB_PORT_ENABLED != 0 {
                let _ = self.hub_clear_feature(port, HUB_FEATURE_C_PORT_RESET);
                let _ = self.hub_clear_feature(port, HUB_FEATURE_C_PORT_ENABLE);
                return Ok(status);
            }
        }
        Err("hub port reset timeout")
    }

    unsafe fn find_boot_hid_endpoint(&mut self) -> Result<HidEndpoint, &'static str> {
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
        parse_boot_hid_endpoint(&CONTROL_BUFFER.0[..actual_len], self.keyboard.is_none())
    }

    unsafe fn configure_interrupt_endpoint(
        &mut self,
        device_index: usize,
        slot_id: u8,
        port: PortInfo,
        endpoint: HidEndpoint,
    ) -> Result<(), &'static str> {
        self.reset_interrupt_ring_at(device_index);
        clear_input_context();
        input_control_add_flags((1 << 0) | (1 << endpoint.dci));
        write_slot_context(port, endpoint.dci, self.context_size);
        write_endpoint_context(
            endpoint.dci,
            self.context_size,
            EP_TYPE_INTERRUPT_IN,
            interval_to_xhci(port.speed, endpoint.interval),
            endpoint.max_packet_size,
            phys_of(
                ptr::addr_of!(INTR_RINGS[device_index].0[0]),
                "interrupt ring phys",
            )? | 1,
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

        match endpoint.kind {
            HidKind::Keyboard | HidKind::GenericKeyboard => {
                self.keyboard = Some(KeyboardDevice {
                    slot_id,
                    dci: endpoint.dci,
                    ring_index: device_index,
                    previous_report: [0; KEYBOARD_REPORT_LEN],
                });
            }
            HidKind::Mouse => {
                self.mouse = Some(MouseDevice {
                    slot_id,
                    dci: endpoint.dci,
                    ring_index: device_index,
                    kind: endpoint.kind,
                });
            }
            HidKind::Tablet => {
                self.mouse = Some(MouseDevice {
                    slot_id,
                    dci: endpoint.dci,
                    ring_index: device_index,
                    kind: endpoint.kind,
                });
            }
        }
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
        self.ring_doorbell(self.control_slot_or_one(), DCI_EP0);
        let event = self.wait_transfer_event(self.control_slot_or_one(), DCI_EP0)?;
        let cc = event.completion_code();
        if cc != CC_SUCCESS && cc != CC_SHORT_PACKET {
            return Err("control transfer failed");
        }
        let residual = (event.status & 0x00FF_FFFF) as usize;
        Ok(length.saturating_sub(residual))
    }

    fn control_slot_or_one(&self) -> u8 {
        self.current_slot_id.max(1)
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
                    self.last_command_type = command_type as u8;
                    self.last_completion_code = cc as u8;
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
        let ring_index = self.current_device_index;
        if ring_index >= MAX_HID_DEVICES {
            return Err("ep0 ring index out of range");
        }
        if self.ep0_enqueue[ring_index] == EP0_RING_LEN - 1 {
            let link_phys = phys_of(
                ptr::addr_of!(EP0_RINGS[ring_index].0[0]),
                "ep0 ring link phys",
            )?;
            let mut link = Trb {
                parameter: link_phys,
                status: 0,
                control: TRB_LINK_TOGGLE_CYCLE | trb_type(TRB_TYPE_LINK),
            };
            if self.ep0_cycle[ring_index] {
                link.control |= TRB_CYCLE;
            }
            ptr::write_volatile(
                ptr::addr_of_mut!(EP0_RINGS[ring_index].0[self.ep0_enqueue[ring_index]]),
                link,
            );
            self.ep0_enqueue[ring_index] = 0;
            self.ep0_cycle[ring_index] = !self.ep0_cycle[ring_index];
        }

        if self.ep0_cycle[ring_index] {
            trb.control |= TRB_CYCLE;
        } else {
            trb.control &= !TRB_CYCLE;
        }
        ptr::write_volatile(
            ptr::addr_of_mut!(EP0_RINGS[ring_index].0[self.ep0_enqueue[ring_index]]),
            trb,
        );
        self.ep0_enqueue[ring_index] += 1;
        Ok(())
    }

    unsafe fn queue_keyboard_report(&mut self) -> Result<(), &'static str> {
        let Some(keyboard) = self.keyboard else {
            return Err("keyboard not configured");
        };
        ptr::write_bytes(KEYBOARD_REPORT.0.as_mut_ptr(), 0, KEYBOARD_REPORT_LEN);
        self.push_interrupt_trb(
            keyboard.ring_index,
            Trb {
                parameter: phys_of(ptr::addr_of!(KEYBOARD_REPORT.0[0]), "keyboard report phys")?,
                status: KEYBOARD_REPORT_LEN as u32,
                control: TRB_IOC | trb_type(TRB_TYPE_NORMAL),
            },
        )?;
        fence(Ordering::SeqCst);
        self.ring_doorbell(keyboard.slot_id, keyboard.dci);
        Ok(())
    }

    unsafe fn queue_mouse_report(&mut self) -> Result<(), &'static str> {
        let Some(mouse) = self.mouse else {
            return Err("mouse not configured");
        };
        ptr::write_bytes(MOUSE_REPORT.0.as_mut_ptr(), 0, POINTER_REPORT_BUFFER_LEN);
        self.push_interrupt_trb(
            mouse.ring_index,
            Trb {
                parameter: phys_of(ptr::addr_of!(MOUSE_REPORT.0[0]), "mouse report phys")?,
                status: mouse.kind.report_len() as u32,
                control: TRB_IOC | trb_type(TRB_TYPE_NORMAL),
            },
        )?;
        fence(Ordering::SeqCst);
        self.ring_doorbell(mouse.slot_id, mouse.dci);
        Ok(())
    }

    unsafe fn push_interrupt_trb(
        &mut self,
        ring_index: usize,
        mut trb: Trb,
    ) -> Result<(), &'static str> {
        if ring_index >= MAX_HID_DEVICES {
            return Err("interrupt ring index out of range");
        }
        if self.intr_enqueue[ring_index] == INTR_RING_LEN - 1 {
            let link_phys = phys_of(
                ptr::addr_of!(INTR_RINGS[ring_index].0[0]),
                "interrupt ring link phys",
            )?;
            let mut link = Trb {
                parameter: link_phys,
                status: 0,
                control: TRB_LINK_TOGGLE_CYCLE | trb_type(TRB_TYPE_LINK),
            };
            if self.intr_cycle[ring_index] {
                link.control |= TRB_CYCLE;
            }
            ptr::write_volatile(
                ptr::addr_of_mut!(INTR_RINGS[ring_index].0[self.intr_enqueue[ring_index]]),
                link,
            );
            self.intr_enqueue[ring_index] = 0;
            self.intr_cycle[ring_index] = !self.intr_cycle[ring_index];
        }

        if self.intr_cycle[ring_index] {
            trb.control |= TRB_CYCLE;
        } else {
            trb.control &= !TRB_CYCLE;
        }
        ptr::write_volatile(
            ptr::addr_of_mut!(INTR_RINGS[ring_index].0[self.intr_enqueue[ring_index]]),
            trb,
        );
        self.intr_enqueue[ring_index] += 1;
        Ok(())
    }

    unsafe fn poll_input<K, M>(&mut self, keyboard_fn: &mut K, mouse_fn: &mut M)
    where
        K: FnMut(u16, bool),
        M: FnMut(UsbMouseReport),
    {
        let mut processed = 0usize;
        while processed < 16 {
            let Some(event) = self.poll_event() else {
                break;
            };
            if event.trb_type() == TRB_TYPE_TRANSFER_EVENT {
                self.handle_hid_transfer_event(event, keyboard_fn, mouse_fn);
            }
            processed += 1;
        }
        if processed == 0 {
            self.kick_input_endpoints();
        }
    }

    unsafe fn handle_hid_transfer_event<K, M>(
        &mut self,
        event: Trb,
        keyboard_fn: &mut K,
        mouse_fn: &mut M,
    ) where
        K: FnMut(u16, bool),
        M: FnMut(UsbMouseReport),
    {
        if let Some(mut keyboard) = self.keyboard {
            if event.slot_id() == keyboard.slot_id && event.endpoint_id() == keyboard.dci {
                let cc = event.completion_code();
                if cc == CC_SUCCESS || cc == CC_SHORT_PACKET {
                    self.last_transfer_completion_code = cc as u8;
                    self.keyboard_report_count = self.keyboard_report_count.saturating_add(1);
                    let report = KEYBOARD_REPORT.0;
                    emit_keyboard_report_changes(
                        &mut keyboard.previous_report,
                        &report,
                        keyboard_fn,
                    );
                } else {
                    serial::write_fmt(format_args!(
                        "usb-hid: keyboard interrupt completion {}\r\n",
                        cc
                    ));
                    self.last_transfer_completion_code = cc as u8;
                    self.transfer_error_count = self.transfer_error_count.saturating_add(1);
                    let _ = self.recover_interrupt_endpoint(
                        keyboard.slot_id,
                        keyboard.dci,
                        keyboard.ring_index,
                    );
                }
                self.keyboard = Some(keyboard);
                let _ = self.queue_keyboard_report();
                return;
            }
        }

        if let Some(mouse) = self.mouse {
            if event.slot_id() == mouse.slot_id && event.endpoint_id() == mouse.dci {
                let cc = event.completion_code();
                if cc == CC_SUCCESS || cc == CC_SHORT_PACKET {
                    self.last_transfer_completion_code = cc as u8;
                    self.mouse_report_count = self.mouse_report_count.saturating_add(1);
                    let report = MOUSE_REPORT.0;
                    mouse_fn(decode_pointer_report(mouse.kind, &report));
                } else {
                    serial::write_fmt(format_args!(
                        "usb-hid: mouse interrupt completion {}\r\n",
                        cc
                    ));
                    self.last_transfer_completion_code = cc as u8;
                    self.transfer_error_count = self.transfer_error_count.saturating_add(1);
                    let _ =
                        self.recover_interrupt_endpoint(mouse.slot_id, mouse.dci, mouse.ring_index);
                }
                let _ = self.queue_mouse_report();
            }
        }
    }

    unsafe fn kick_input_endpoints(&self) {
        if let Some(keyboard) = self.keyboard {
            self.ring_doorbell(keyboard.slot_id, keyboard.dci);
        }
        if let Some(mouse) = self.mouse {
            self.ring_doorbell(mouse.slot_id, mouse.dci);
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
        let ring_index = self.current_device_index;
        ptr::write_bytes(
            ptr::addr_of_mut!(EP0_RINGS[ring_index].0[0]).cast::<u8>(),
            0,
            core::mem::size_of::<Trb>() * EP0_RING_LEN,
        );
        self.ep0_enqueue[ring_index] = 0;
        self.ep0_cycle[ring_index] = true;
    }

    unsafe fn reset_interrupt_ring_at(&mut self, ring_index: usize) {
        ptr::write_bytes(
            ptr::addr_of_mut!(INTR_RINGS[ring_index].0[0]).cast::<u8>(),
            0,
            core::mem::size_of::<Trb>() * INTR_RING_LEN,
        );
        self.intr_enqueue[ring_index] = 0;
        self.intr_cycle[ring_index] = true;
    }

    unsafe fn recover_interrupt_endpoint(
        &mut self,
        slot_id: u8,
        dci: u8,
        ring_index: usize,
    ) -> Result<(), &'static str> {
        serial::write_fmt(format_args!(
            "usb-hid: recovering slot {} endpoint {}\r\n",
            slot_id, dci
        ));
        self.execute_command(Trb {
            parameter: 0,
            status: 0,
            control: trb_type(TRB_TYPE_RESET_ENDPOINT)
                | ((dci as u32) << 16)
                | ((slot_id as u32) << 24),
        })?;
        self.reset_interrupt_ring_at(ring_index);
        let dequeue = phys_of(
            ptr::addr_of!(INTR_RINGS[ring_index].0[0]),
            "interrupt dequeue phys",
        )? | 1;
        self.execute_command(Trb {
            parameter: dequeue,
            status: 0,
            control: trb_type(TRB_TYPE_SET_TR_DEQUEUE_POINTER)
                | ((dci as u32) << 16)
                | ((slot_id as u32) << 24),
        })?;
        Ok(())
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
        ptr::addr_of_mut!(EP0_RINGS).cast::<u8>(),
        0,
        core::mem::size_of::<[TrbRing<EP0_RING_LEN>; MAX_HID_DEVICES]>(),
    );
    ptr::write_bytes(
        ptr::addr_of_mut!(INTR_RINGS).cast::<u8>(),
        0,
        core::mem::size_of::<[TrbRing<INTR_RING_LEN>; MAX_HID_DEVICES]>(),
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
    ptr::write_bytes(MOUSE_REPORT.0.as_mut_ptr(), 0, POINTER_REPORT_BUFFER_LEN);
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
    ptr::write_bytes(
        ptr::addr_of_mut!(DEVICE_CONTEXTS).cast::<u8>(),
        0,
        core::mem::size_of::<[AlignedBytes<CONTEXT_BYTES>; MAX_HID_DEVICES]>(),
    );
}

unsafe fn clear_input_context() {
    ptr::write_bytes(INPUT_CONTEXT.0.as_mut_ptr(), 0, CONTEXT_BYTES);
}

unsafe fn clear_device_context(device_index: usize) {
    if device_index < MAX_HID_DEVICES {
        ptr::write_bytes(
            DEVICE_CONTEXTS[device_index].0.as_mut_ptr(),
            0,
            CONTEXT_BYTES,
        );
    }
}

unsafe fn input_control_add_flags(flags: u32) {
    ctx_write_raw(ptr::addr_of_mut!(INPUT_CONTEXT.0[0]), 1, flags);
}

fn child_route_string(parent_route: u32, port_number: u8) -> u32 {
    let mut shift = 0u32;
    while shift < 20 {
        if ((parent_route >> shift) & 0xF) == 0 {
            return parent_route | (((port_number as u32) & 0xF) << shift);
        }
        shift += 4;
    }
    parent_route
}

fn hub_port_speed(status: u16, parent_hub_speed: u8) -> u8 {
    if status & HUB_PORT_LOW_SPEED != 0 {
        2
    } else if status & HUB_PORT_HIGH_SPEED != 0 {
        3
    } else if parent_hub_speed >= 4 {
        parent_hub_speed
    } else {
        1
    }
}

fn child_speed_candidates(primary: u8) -> ([u8; 5], usize) {
    let mut speeds = [0u8; 5];
    let mut count = 0usize;
    push_unique_speed(&mut speeds, &mut count, primary);
    push_unique_speed(&mut speeds, &mut count, 3);
    push_unique_speed(&mut speeds, &mut count, 1);
    push_unique_speed(&mut speeds, &mut count, 4);
    push_unique_speed(&mut speeds, &mut count, 2);
    (speeds, count)
}

fn push_unique_speed(speeds: &mut [u8; 5], count: &mut usize, speed: u8) {
    if speed == 0 || *count >= speeds.len() {
        return;
    }

    let mut index = 0usize;
    while index < *count {
        if speeds[index] == speed {
            return;
        }
        index += 1;
    }

    speeds[*count] = speed;
    *count += 1;
}

fn spin_delay(iterations: usize) {
    let mut waited = 0usize;
    while waited < iterations {
        spin_loop();
        waited += 1;
    }
}

unsafe fn write_slot_context(port: PortInfo, context_entries: u8, context_size: usize) {
    let slot = input_context_ptr(context_size, 0);
    let hub_bit = if port.is_hub { 1 << 26 } else { 0 };
    let dword0 = port.route_string
        | ((port.speed as u32) << 20)
        | hub_bit
        | ((context_entries as u32) << 27);
    let dword1 = ((port.root_port_number as u32) << 16) | ((port.hub_port_count as u32) << 24);
    let dword2 = (port.parent_hub_slot_id as u32) | ((port.parent_hub_port_number as u32) << 8);
    ctx_write_raw(slot, 0, dword0);
    ctx_write_raw(slot, 1, dword1);
    ctx_write_raw(slot, 2, dword2);
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

unsafe fn parse_boot_hid_endpoint(
    config: &[u8],
    prefer_generic_keyboard: bool,
) -> Result<HidEndpoint, &'static str> {
    if config.len() < 9 {
        return Err("configuration descriptor too short");
    }

    let mut configuration_value = 0u8;
    let mut current_interface = 0u8;
    let mut current_hid_kind: Option<HidKind> = None;
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
                current_hid_kind = match (class, subclass, protocol) {
                    (0x03, 0x01, 0x01) => Some(HidKind::Keyboard),
                    (0x03, 0x01, 0x02) => Some(HidKind::Mouse),
                    (0x03, 0x00, 0x00) if prefer_generic_keyboard => Some(HidKind::GenericKeyboard),
                    (0x03, 0x00, 0x00) => Some(HidKind::Tablet),
                    _ => None,
                };
                if let Some(kind) = current_hid_kind {
                    serial::write_fmt(format_args!(
                        "usb-hid: {} interface {}\r\n",
                        kind.as_str(),
                        current_interface,
                    ));
                }
            }
            5 if len >= 7 && current_hid_kind.is_some() => {
                let endpoint_address = config[offset + 2];
                let attributes = config[offset + 3] & 0x03;
                if endpoint_address & 0x80 != 0 && attributes == 0x03 {
                    let kind = current_hid_kind.unwrap();
                    let max_packet_size =
                        u16::from_le_bytes([config[offset + 4], config[offset + 5]]) & 0x07FF;
                    let endpoint_number = endpoint_address & 0x0F;
                    let dci = endpoint_number * 2 + 1;
                    return Ok(HidEndpoint {
                        kind,
                        interface_number: current_interface,
                        configuration_value,
                        endpoint_address,
                        dci,
                        max_packet_size: u16::max(max_packet_size, kind.report_len() as u16),
                        interval: config[offset + 6],
                    });
                }
            }
            _ => {}
        }

        offset += len;
    }

    Err("HID interrupt endpoint not found")
}

fn decode_pointer_report(
    kind: HidKind,
    report: &[u8; POINTER_REPORT_BUFFER_LEN],
) -> UsbMouseReport {
    match kind {
        HidKind::Tablet => UsbMouseReport {
            buttons: report[0] & 0x07,
            dx: 0,
            dy: 0,
            wheel: report[5] as i8,
            absolute: true,
            x: u16::from_le_bytes([report[1], report[2]]),
            y: u16::from_le_bytes([report[3], report[4]]),
            max_x: TABLET_AXIS_MAX,
            max_y: TABLET_AXIS_MAX,
        },
        _ => UsbMouseReport {
            buttons: report[0] & 0x07,
            dx: report[1] as i8,
            dy: report[2] as i8,
            wheel: report[3] as i8,
            absolute: false,
            x: 0,
            y: 0,
            max_x: 0,
            max_y: 0,
        },
    }
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
        0x4F => 106,
        0x50 => 105,
        0x51 => 108,
        0x52 => 103,
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
