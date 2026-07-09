#![allow(static_mut_refs)]

use alloc::vec;
use core::hint::spin_loop;
use core::ptr;
use core::sync::atomic::{fence, Ordering};

use spin::Mutex;

use crate::memory;
use crate::pci::{self, PciAddress};
use crate::serial;
use raios_core::{
    gpt_layout::{self, LayoutStatus, GPT_ENTRY_ARRAY_BYTES, PRIMARY_GPT_ENTRIES_LBA, SECTOR_SIZE},
    seed_data_layout,
};

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
const EP_TYPE_BULK_OUT: u32 = 2;
const EP_TYPE_CONTROL: u32 = 4;
const EP_TYPE_BULK_IN: u32 = 6;
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
const USB_CLASS_MASS_STORAGE: u8 = 0x08;
const USB_MSC_SUBCLASS_SCSI: u8 = 0x06;
const USB_MSC_PROTOCOL_BOT: u8 = 0x50;
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
const BULK_RING_LEN: usize = 16;
const MAX_HID_DEVICES: usize = 16;
const MAX_SLOTS: usize = 32;
const MAX_SCRATCHPADS: usize = 64;
const CONTEXT_BYTES: usize = 4096;
const CONTROL_BUFFER_LEN: usize = 1024;
const KEYBOARD_REPORT_LEN: usize = 8;
const MOUSE_REPORT_LEN: usize = 4;
const TABLET_REPORT_LEN: usize = 8;
const POINTER_REPORT_BUFFER_LEN: usize = TABLET_REPORT_LEN;
const TABLET_AXIS_MAX: u16 = 0x7fff;
const MSC_CBW_LEN: usize = 31;
const MSC_CSW_LEN: usize = 13;
const MSC_SECTOR_BUFFER_LEN: usize = 512;
const MSC_CBW_SIGNATURE: u32 = 0x4342_5355;
const MSC_CSW_SIGNATURE: u32 = 0x5342_5355;
const MSC_READ_CAPACITY10_LEN: usize = 8;
const WAIT_ITERS: usize = 5_000_000;
const ROOT_PORT_POWER_SETTLE_ITERS: usize = WAIT_ITERS;
const HUB_PORT_RESET_POLLS: usize = 64;
const HUB_PORT_RESET_POLL_DELAY_ITERS: usize = WAIT_ITERS / 100;
const HOTPLUG_MAX_ROOT_PORTS: u8 = 32;
const MAX_HUB_WATCHES: usize = MAX_HID_DEVICES;

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
    pub last_enum_error: Option<&'static str>,
    pub last_device_vid: u16,
    pub last_device_pid: u16,
    pub last_device_class: u8,
    pub enum_location: UsbEnumLocation,
    pub enum_speed: u8,
    pub enum_slot_id: u8,
    pub enum_stage: &'static str,
    pub enum_cmd_cc: u8,
    pub enum_xfer_cc: u8,
    pub enum_vid: u16,
    pub enum_pid: u16,
    pub enum_dev_class: u8,
    pub enum_ep0_mps: u16,
    pub enum_err: &'static str,
    pub last_hotplug_seq: u32,
    pub last_hotplug_present: bool,
    pub last_hotplug_connected: bool,
    pub last_hotplug_is_hub: bool,
    pub last_hotplug_hub_slot: u8,
    pub last_hotplug_port: u8,
    pub last_hotplug_detail: &'static str,
    pub last_hotplug_skipped: bool,
    pub last_command_type: u8,
    pub last_completion_code: u8,
    pub last_transfer_completion_code: u8,
    pub last_int_cc: u8,
    pub recover_count: u32,
    pub input_report_count: u32,
    pub input_error_count: u32,
    pub keyboard_status: UsbKeyboardStatus,
    pub keyboard_detail: Option<&'static str>,
    pub mouse_status: UsbMouseStatus,
    pub mouse_detail: Option<&'static str>,
    pub mass_storage_present: bool,
    pub mass_storage_read_completed: bool,
    pub mass_storage_seed_data_present: bool,
    pub mass_storage_block_size: u32,
    pub mass_storage_last_lba: u32,
    pub mass_storage_lba0_signature: u16,
    pub mass_storage_detail: &'static str,
    pub last_error: Option<&'static str>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UsbEnumLocation {
    pub is_hub: bool,
    pub hub_slot: u8,
    pub port: u8,
}

impl UsbEnumLocation {
    const fn none() -> Self {
        Self {
            is_hub: false,
            hub_slot: 0,
            port: 0,
        }
    }
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
                last_enum_error: None,
                last_device_vid: 0,
                last_device_pid: 0,
                last_device_class: 0,
                enum_location: UsbEnumLocation::none(),
                enum_speed: 0,
                enum_slot_id: 0,
                enum_stage: "none",
                enum_cmd_cc: 0,
                enum_xfer_cc: 0,
                enum_vid: 0,
                enum_pid: 0,
                enum_dev_class: 0,
                enum_ep0_mps: 0,
                enum_err: "ok",
                last_hotplug_seq: 0,
                last_hotplug_present: false,
                last_hotplug_connected: false,
                last_hotplug_is_hub: false,
                last_hotplug_hub_slot: 0,
                last_hotplug_port: 0,
                last_hotplug_detail: "none",
                last_hotplug_skipped: false,
                last_command_type: 0,
                last_completion_code: 0,
                last_transfer_completion_code: 0,
                last_int_cc: 0,
                recover_count: 0,
                input_report_count: 0,
                input_error_count: 0,
                keyboard_status: UsbKeyboardStatus::NotProbed,
                keyboard_detail: None,
                mouse_status: UsbMouseStatus::NotProbed,
                mouse_detail: None,
                mass_storage_present: false,
                mass_storage_read_completed: false,
                mass_storage_seed_data_present: false,
                mass_storage_block_size: 0,
                mass_storage_last_lba: 0,
                mass_storage_lba0_signature: 0,
                mass_storage_detail: "not_probed",
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
    let previous = STATE.lock().snapshot;
    let (snapshot, controller) = unsafe { probe_xhci() };
    let input_ready = controller
        .as_ref()
        .is_some_and(|controller| controller.keyboard.is_some() || controller.mouse.is_some());
    let (snapshot, controller) = carry_hotplug(previous, snapshot, controller);
    *STATE.lock() = UsbState {
        snapshot,
        controller,
    };
    if input_ready {
        serial::write_line("usb-hotplug: input device configured");
    }
    true
}

pub fn poll_hotplug() -> bool {
    let mut state = STATE.lock();
    let mut snapshot = state.snapshot;
    let Some(controller) = state.controller.as_mut() else {
        return false;
    };
    let changed = unsafe { controller.poll_hotplug() };
    refresh_snapshot_from_controller(&mut snapshot, controller);
    state.snapshot = snapshot;
    changed
}

pub fn snapshot() -> UsbSnapshot {
    let state = STATE.lock();
    let mut snapshot = state.snapshot;
    if let Some(controller) = state.controller.as_ref() {
        snapshot.last_command_type = controller.last_command_type;
        snapshot.last_completion_code = controller.last_completion_code;
        snapshot.last_transfer_completion_code = controller.last_transfer_completion_code;
        snapshot.last_int_cc = controller.last_int_cc;
        snapshot.recover_count = controller.recover_count;
        snapshot.input_report_count = controller
            .keyboard_report_count
            .saturating_add(controller.mouse_report_count);
        snapshot.input_error_count = controller.transfer_error_count;
        snapshot.last_enum_error = controller.last_enum_error;
        snapshot.last_device_vid = controller.last_device_vid;
        snapshot.last_device_pid = controller.last_device_pid;
        snapshot.last_device_class = controller.last_device_class;
        snapshot.enum_location = controller.enum_location;
        snapshot.enum_speed = controller.enum_speed;
        snapshot.enum_slot_id = controller.enum_slot_id;
        snapshot.enum_stage = controller.enum_stage;
        snapshot.enum_cmd_cc = controller.enum_cmd_cc;
        snapshot.enum_xfer_cc = controller.enum_xfer_cc;
        snapshot.enum_vid = controller.enum_vid;
        snapshot.enum_pid = controller.enum_pid;
        snapshot.enum_dev_class = controller.enum_dev_class;
        snapshot.enum_ep0_mps = controller.enum_ep0_mps;
        snapshot.enum_err = controller.enum_err;
        snapshot.mass_storage_present = controller.mass_storage_probe.present;
        snapshot.mass_storage_read_completed = controller.mass_storage_probe.read_completed;
        snapshot.mass_storage_seed_data_present = controller.mass_storage_probe.seed_data_present;
        snapshot.mass_storage_block_size = controller.mass_storage_probe.block_size;
        snapshot.mass_storage_last_lba = controller.mass_storage_probe.last_lba;
        snapshot.mass_storage_lba0_signature = controller.mass_storage_probe.lba0_signature;
        snapshot.mass_storage_detail = controller.mass_storage_probe.detail;
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
                last_enum_error: None,
                last_device_vid: 0,
                last_device_pid: 0,
                last_device_class: 0,
                enum_location: UsbEnumLocation::none(),
                enum_speed: 0,
                enum_slot_id: 0,
                enum_stage: "none",
                enum_cmd_cc: 0,
                enum_xfer_cc: 0,
                enum_vid: 0,
                enum_pid: 0,
                enum_dev_class: 0,
                enum_ep0_mps: 0,
                enum_err: "ok",
                last_hotplug_seq: 0,
                last_hotplug_present: false,
                last_hotplug_connected: false,
                last_hotplug_is_hub: false,
                last_hotplug_hub_slot: 0,
                last_hotplug_port: 0,
                last_hotplug_detail: "none",
                last_hotplug_skipped: false,
                last_command_type: 0,
                last_completion_code: 0,
                last_transfer_completion_code: 0,
                last_int_cc: 0,
                recover_count: 0,
                input_report_count: 0,
                input_error_count: 0,
                keyboard_status: UsbKeyboardStatus::NotProbed,
                keyboard_detail: None,
                mouse_status: UsbMouseStatus::NotProbed,
                mouse_detail: None,
                mass_storage_present: false,
                mass_storage_read_completed: false,
                mass_storage_seed_data_present: false,
                mass_storage_block_size: 0,
                mass_storage_last_lba: 0,
                mass_storage_lba0_signature: 0,
                mass_storage_detail: "controller_absent",
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
                last_enum_error: controller.last_enum_error,
                last_device_vid: controller.last_device_vid,
                last_device_pid: controller.last_device_pid,
                last_device_class: controller.last_device_class,
                enum_location: controller.enum_location,
                enum_speed: controller.enum_speed,
                enum_slot_id: controller.enum_slot_id,
                enum_stage: controller.enum_stage,
                enum_cmd_cc: controller.enum_cmd_cc,
                enum_xfer_cc: controller.enum_xfer_cc,
                enum_vid: controller.enum_vid,
                enum_pid: controller.enum_pid,
                enum_dev_class: controller.enum_dev_class,
                enum_ep0_mps: controller.enum_ep0_mps,
                enum_err: controller.enum_err,
                last_hotplug_seq: controller.hotplug.seq,
                last_hotplug_present: controller.hotplug.present,
                last_hotplug_connected: controller.hotplug.connected,
                last_hotplug_is_hub: controller.hotplug.is_hub,
                last_hotplug_hub_slot: controller.hotplug.hub_slot,
                last_hotplug_port: controller.hotplug.port,
                last_hotplug_detail: controller.hotplug.detail,
                last_hotplug_skipped: controller.hotplug.skipped,
                last_command_type: controller.last_command_type,
                last_completion_code: controller.last_completion_code,
                last_transfer_completion_code: controller.last_transfer_completion_code,
                last_int_cc: controller.last_int_cc,
                recover_count: controller.recover_count,
                input_report_count: controller
                    .keyboard_report_count
                    .saturating_add(controller.mouse_report_count),
                input_error_count: controller.transfer_error_count,
                keyboard_status,
                keyboard_detail,
                mouse_status,
                mouse_detail,
                mass_storage_present: controller.mass_storage_probe.present,
                mass_storage_read_completed: controller.mass_storage_probe.read_completed,
                mass_storage_seed_data_present: controller.mass_storage_probe.seed_data_present,
                mass_storage_block_size: controller.mass_storage_probe.block_size,
                mass_storage_last_lba: controller.mass_storage_probe.last_lba,
                mass_storage_lba0_signature: controller.mass_storage_probe.lba0_signature,
                mass_storage_detail: controller.mass_storage_probe.detail,
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
        last_enum_error: None,
        last_device_vid: 0,
        last_device_pid: 0,
        last_device_class: 0,
        enum_location: UsbEnumLocation::none(),
        enum_speed: 0,
        enum_slot_id: 0,
        enum_stage: "none",
        enum_cmd_cc: 0,
        enum_xfer_cc: 0,
        enum_vid: 0,
        enum_pid: 0,
        enum_dev_class: 0,
        enum_ep0_mps: 0,
        enum_err: "ok",
        last_hotplug_seq: 0,
        last_hotplug_present: false,
        last_hotplug_connected: false,
        last_hotplug_is_hub: false,
        last_hotplug_hub_slot: 0,
        last_hotplug_port: 0,
        last_hotplug_detail: "none",
        last_hotplug_skipped: false,
        last_command_type: 0,
        last_completion_code: 0,
        last_transfer_completion_code: 0,
        last_int_cc: 0,
        recover_count: 0,
        input_report_count: 0,
        input_error_count: 0,
        keyboard_status: UsbKeyboardStatus::Error,
        keyboard_detail: Some(error),
        mouse_status: UsbMouseStatus::Error,
        mouse_detail: Some(error),
        mass_storage_present: false,
        mass_storage_read_completed: false,
        mass_storage_seed_data_present: false,
        mass_storage_block_size: 0,
        mass_storage_last_lba: 0,
        mass_storage_lba0_signature: 0,
        mass_storage_detail: error,
        last_error: Some(error),
    }
}

fn carry_hotplug(
    previous: UsbSnapshot,
    mut snapshot: UsbSnapshot,
    mut controller: Option<XhciController>,
) -> (UsbSnapshot, Option<XhciController>) {
    snapshot.last_hotplug_seq = previous.last_hotplug_seq;
    snapshot.last_hotplug_present = previous.last_hotplug_present;
    snapshot.last_hotplug_connected = previous.last_hotplug_connected;
    snapshot.last_hotplug_is_hub = previous.last_hotplug_is_hub;
    snapshot.last_hotplug_hub_slot = previous.last_hotplug_hub_slot;
    snapshot.last_hotplug_port = previous.last_hotplug_port;
    snapshot.last_hotplug_detail = previous.last_hotplug_detail;
    snapshot.last_hotplug_skipped = previous.last_hotplug_skipped;
    if let Some(controller) = controller.as_mut() {
        controller.hotplug = HotplugRecord::from_snapshot(snapshot);
    }
    (snapshot, controller)
}

fn refresh_snapshot_from_controller(snapshot: &mut UsbSnapshot, controller: &XhciController) {
    snapshot.connected_ports = count_mask(controller.root_connected_mask);
    snapshot.hub_count = controller.hub_count;
    snapshot.hub_ports = controller.hub_ports;
    snapshot.hub_connected_ports = controller.current_hub_connected_ports();
    snapshot.hub_reset_ports = controller.hub_reset_ports;
    snapshot.hub_configured_devices = controller.hub_configured_devices;
    snapshot.hub_last_error = controller.hub_last_error;
    snapshot.last_enum_error = controller.last_enum_error;
    snapshot.last_device_vid = controller.last_device_vid;
    snapshot.last_device_pid = controller.last_device_pid;
    snapshot.last_device_class = controller.last_device_class;
    snapshot.enum_location = controller.enum_location;
    snapshot.enum_speed = controller.enum_speed;
    snapshot.enum_slot_id = controller.enum_slot_id;
    snapshot.enum_stage = controller.enum_stage;
    snapshot.enum_cmd_cc = controller.enum_cmd_cc;
    snapshot.enum_xfer_cc = controller.enum_xfer_cc;
    snapshot.enum_vid = controller.enum_vid;
    snapshot.enum_pid = controller.enum_pid;
    snapshot.enum_dev_class = controller.enum_dev_class;
    snapshot.enum_ep0_mps = controller.enum_ep0_mps;
    snapshot.enum_err = controller.enum_err;
    snapshot.last_hotplug_seq = controller.hotplug.seq;
    snapshot.last_hotplug_present = controller.hotplug.present;
    snapshot.last_hotplug_connected = controller.hotplug.connected;
    snapshot.last_hotplug_is_hub = controller.hotplug.is_hub;
    snapshot.last_hotplug_hub_slot = controller.hotplug.hub_slot;
    snapshot.last_hotplug_port = controller.hotplug.port;
    snapshot.last_hotplug_detail = controller.hotplug.detail;
    snapshot.last_hotplug_skipped = controller.hotplug.skipped;
    snapshot.last_int_cc = controller.last_int_cc;
    snapshot.recover_count = controller.recover_count;
    snapshot.input_report_count = controller
        .keyboard_report_count
        .saturating_add(controller.mouse_report_count);
    snapshot.input_error_count = controller.transfer_error_count;
    snapshot.mass_storage_present = controller.mass_storage_probe.present;
    snapshot.mass_storage_read_completed = controller.mass_storage_probe.read_completed;
    snapshot.mass_storage_seed_data_present = controller.mass_storage_probe.seed_data_present;
    snapshot.mass_storage_block_size = controller.mass_storage_probe.block_size;
    snapshot.mass_storage_last_lba = controller.mass_storage_probe.last_lba;
    snapshot.mass_storage_lba0_signature = controller.mass_storage_probe.lba0_signature;
    snapshot.mass_storage_detail = controller.mass_storage_probe.detail;
    snapshot.keyboard_status = if controller.keyboard.is_some() {
        UsbKeyboardStatus::Ready
    } else {
        UsbKeyboardStatus::NotFound
    };
    snapshot.keyboard_detail = if controller.keyboard.is_some() {
        Some("USB HID BOOT KEYBOARD")
    } else {
        Some("no USB boot keyboard found")
    };
    snapshot.mouse_status = if controller.mouse.is_some() {
        UsbMouseStatus::Ready
    } else {
        UsbMouseStatus::NotFound
    };
    snapshot.mouse_detail = if controller.mouse.is_some() {
        Some("USB HID BOOT MOUSE")
    } else {
        Some("no USB pointer found")
    };
}

fn count_mask(mask: u32) -> u8 {
    mask.count_ones() as u8
}

fn port_mask(port: u8) -> u32 {
    1u32 << (port.saturating_sub(1) as u32)
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
static mut BULK_IN_RINGS: [TrbRing<BULK_RING_LEN>; MAX_HID_DEVICES] =
    [TrbRing([Trb::zero(); BULK_RING_LEN]); MAX_HID_DEVICES];
static mut BULK_OUT_RINGS: [TrbRing<BULK_RING_LEN>; MAX_HID_DEVICES] =
    [TrbRing([Trb::zero(); BULK_RING_LEN]); MAX_HID_DEVICES];
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
static mut MSC_CBW: AlignedBytes<MSC_CBW_LEN> = AlignedBytes([0; MSC_CBW_LEN]);
static mut MSC_CSW: AlignedBytes<MSC_CSW_LEN> = AlignedBytes([0; MSC_CSW_LEN]);
static mut MSC_BUFFER: AlignedBytes<MSC_SECTOR_BUFFER_LEN> =
    AlignedBytes([0; MSC_SECTOR_BUFFER_LEN]);

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
    bulk_in_enqueue: [usize; MAX_HID_DEVICES],
    bulk_in_cycle: [bool; MAX_HID_DEVICES],
    bulk_out_enqueue: [usize; MAX_HID_DEVICES],
    bulk_out_cycle: [bool; MAX_HID_DEVICES],
    event_ring_phys: u64,
    current_device_index: usize,
    current_slot_id: u8,
    keyboard: Option<KeyboardDevice>,
    mouse: Option<MouseDevice>,
    mass_storage: Option<MassStorageDevice>,
    mass_storage_probe: MassStorageProbe,
    hub_count: u8,
    hub_ports: u8,
    hub_connected_ports: u8,
    hub_reset_ports: u8,
    hub_configured_devices: u8,
    hub_last_error: Option<&'static str>,
    last_enum_error: Option<&'static str>,
    last_device_vid: u16,
    last_device_pid: u16,
    last_device_class: u8,
    enum_location: UsbEnumLocation,
    enum_speed: u8,
    enum_slot_id: u8,
    enum_stage: &'static str,
    enum_cmd_cc: u8,
    enum_xfer_cc: u8,
    enum_vid: u16,
    enum_pid: u16,
    enum_dev_class: u8,
    enum_ep0_mps: u16,
    enum_err: &'static str,
    next_device_index: usize,
    root_connected_mask: u32,
    hub_watches: [HubWatch; MAX_HUB_WATCHES],
    hotplug: HotplugRecord,
    last_command_type: u8,
    last_completion_code: u8,
    last_transfer_completion_code: u8,
    last_int_cc: u8,
    recover_count: u32,
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
    root_port_number: u8,
    parent_hub_slot_id: u8,
    parent_hub_port_number: u8,
    previous_report: [u8; KEYBOARD_REPORT_LEN],
}

#[derive(Clone, Copy)]
struct MouseDevice {
    slot_id: u8,
    dci: u8,
    ring_index: usize,
    kind: HidKind,
    root_port_number: u8,
    parent_hub_slot_id: u8,
    parent_hub_port_number: u8,
}

#[derive(Clone, Copy)]
struct MassStorageDevice {
    slot_id: u8,
    bulk_in_dci: u8,
    bulk_out_dci: u8,
    ring_index: usize,
    root_port_number: u8,
    parent_hub_slot_id: u8,
    parent_hub_port_number: u8,
    tag: u32,
}

#[derive(Clone, Copy)]
struct MassStorageProbe {
    present: bool,
    read_completed: bool,
    seed_data_present: bool,
    block_size: u32,
    last_lba: u32,
    lba0_signature: u16,
    detail: &'static str,
}

impl MassStorageProbe {
    const fn none() -> Self {
        Self {
            present: false,
            read_completed: false,
            seed_data_present: false,
            block_size: 0,
            last_lba: 0,
            lba0_signature: 0,
            detail: "not_found",
        }
    }
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

#[derive(Clone, Copy)]
struct HubWatch {
    active: bool,
    slot_id: u8,
    device_index: usize,
    root_port_number: u8,
    route_string: u32,
    speed: u8,
    parent_hub_slot_id: u8,
    parent_hub_port_number: u8,
    port_count: u8,
    connected_mask: u32,
}

impl HubWatch {
    const fn none() -> Self {
        Self {
            active: false,
            slot_id: 0,
            device_index: 0,
            root_port_number: 0,
            route_string: 0,
            speed: 0,
            parent_hub_slot_id: 0,
            parent_hub_port_number: 0,
            port_count: 0,
            connected_mask: 0,
        }
    }

    fn port_info(self) -> PortInfo {
        PortInfo {
            root_port_number: self.root_port_number,
            route_string: self.route_string,
            speed: self.speed,
            parent_hub_slot_id: self.parent_hub_slot_id,
            parent_hub_port_number: self.parent_hub_port_number,
            is_hub: true,
            hub_port_count: self.port_count,
        }
    }
}

#[derive(Clone, Copy)]
struct HotplugRecord {
    seq: u32,
    present: bool,
    connected: bool,
    is_hub: bool,
    hub_slot: u8,
    port: u8,
    detail: &'static str,
    skipped: bool,
}

impl HotplugRecord {
    const fn none() -> Self {
        Self {
            seq: 0,
            present: false,
            connected: false,
            is_hub: false,
            hub_slot: 0,
            port: 0,
            detail: "none",
            skipped: false,
        }
    }

    fn from_snapshot(snapshot: UsbSnapshot) -> Self {
        Self {
            seq: snapshot.last_hotplug_seq,
            present: snapshot.last_hotplug_present,
            connected: snapshot.last_hotplug_connected,
            is_hub: snapshot.last_hotplug_is_hub,
            hub_slot: snapshot.last_hotplug_hub_slot,
            port: snapshot.last_hotplug_port,
            detail: snapshot.last_hotplug_detail,
            skipped: snapshot.last_hotplug_skipped,
        }
    }
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

impl UsbEnumLocation {
    fn root(port: u8) -> Self {
        Self {
            is_hub: false,
            hub_slot: 0,
            port,
        }
    }

    fn hub(hub_slot: u8, port: u8) -> Self {
        Self {
            is_hub: true,
            hub_slot,
            port,
        }
    }

    fn from_port(port: PortInfo) -> Self {
        if port.parent_hub_slot_id == 0 {
            Self::root(port.root_port_number)
        } else {
            Self::hub(port.parent_hub_slot_id, port.parent_hub_port_number)
        }
    }
}

#[derive(Clone, Copy)]
enum EnumeratedKind {
    Hid(HidKind),
    Hub,
    MassStorage,
}

impl EnumeratedKind {
    fn as_str(self) -> &'static str {
        match self {
            EnumeratedKind::Hid(kind) => kind.as_str(),
            EnumeratedKind::Hub => "hub",
            EnumeratedKind::MassStorage => "mass-storage",
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
struct MassStorageInterface {
    configuration_value: u8,
    bulk_in_dci: u8,
    bulk_out_dci: u8,
    bulk_in_endpoint_address: u8,
    bulk_out_endpoint_address: u8,
    bulk_in_max_packet_size: u16,
    bulk_out_max_packet_size: u16,
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
            bulk_in_enqueue: [0; MAX_HID_DEVICES],
            bulk_in_cycle: [true; MAX_HID_DEVICES],
            bulk_out_enqueue: [0; MAX_HID_DEVICES],
            bulk_out_cycle: [true; MAX_HID_DEVICES],
            event_ring_phys,
            current_device_index: 0,
            current_slot_id: 0,
            keyboard: None,
            mouse: None,
            mass_storage: None,
            mass_storage_probe: MassStorageProbe::none(),
            hub_count: 0,
            hub_ports: 0,
            hub_connected_ports: 0,
            hub_reset_ports: 0,
            hub_configured_devices: 0,
            hub_last_error: None,
            last_enum_error: None,
            last_device_vid: 0,
            last_device_pid: 0,
            last_device_class: 0,
            enum_location: UsbEnumLocation::none(),
            enum_speed: 0,
            enum_slot_id: 0,
            enum_stage: "none",
            enum_cmd_cc: 0,
            enum_xfer_cc: 0,
            enum_vid: 0,
            enum_pid: 0,
            enum_dev_class: 0,
            enum_ep0_mps: 0,
            enum_err: "ok",
            next_device_index: 0,
            root_connected_mask: 0,
            hub_watches: [HubWatch::none(); MAX_HUB_WATCHES],
            hotplug: HotplugRecord::none(),
            last_command_type: 0,
            last_completion_code: 0,
            last_transfer_completion_code: 0,
            last_int_cc: 0,
            recover_count: 0,
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

    unsafe fn read_root_connected_mask(&self) -> u32 {
        let mut mask = 0u32;
        let mut port = 1u8;
        let limit = u8::min(self.max_ports, HOTPLUG_MAX_ROOT_PORTS);
        while port <= limit {
            if read32(self.base, self.portsc_offset(port)) & PORTSC_CCS != 0 {
                mask |= port_mask(port);
            }
            port += 1;
        }
        mask
    }

    fn clear_last_enumerated_device(&mut self) {
        self.last_enum_error = None;
        self.last_device_vid = 0;
        self.last_device_pid = 0;
        self.last_device_class = 0;
    }

    fn begin_enum_reset(&mut self, location: UsbEnumLocation) {
        self.begin_enum_at(location, 0, "reset");
    }

    fn begin_enum_attempt(&mut self, port: PortInfo) -> UsbEnumLocation {
        let location = UsbEnumLocation::from_port(port);
        self.begin_enum_at(location, port.speed, "slot");
        location
    }

    fn begin_enum_at(&mut self, location: UsbEnumLocation, speed: u8, stage: &'static str) {
        self.clear_last_enumerated_device();
        self.enum_location = location;
        self.enum_speed = speed;
        self.enum_slot_id = 0;
        self.enum_stage = stage;
        self.enum_cmd_cc = 0;
        self.enum_xfer_cc = 0;
        self.enum_vid = 0;
        self.enum_pid = 0;
        self.enum_dev_class = 0;
        self.enum_ep0_mps = 0;
        self.enum_err = "ok";
    }

    fn set_enum_stage(&mut self, stage: &'static str) {
        self.enum_stage = stage;
    }

    fn capture_enum_completion_codes(&mut self) {
        self.enum_cmd_cc = self.last_completion_code;
        self.enum_xfer_cc = self.last_transfer_completion_code;
    }

    fn finish_enum_attempt(
        &mut self,
        location: UsbEnumLocation,
        result: Result<EnumeratedKind, &'static str>,
    ) -> Result<EnumeratedKind, &'static str> {
        if self.enum_location != location {
            return result;
        }
        match result {
            Ok(kind) => {
                self.enum_stage = "done";
                self.enum_err = "ok";
                self.capture_enum_completion_codes();
                self.log_enum_attempt();
                Ok(kind)
            }
            Err(err) => {
                self.enum_err = err;
                self.capture_enum_completion_codes();
                self.log_enum_attempt();
                Err(err)
            }
        }
    }

    fn fail_enum_attempt(&mut self, err: &'static str) {
        self.enum_err = err;
        self.capture_enum_completion_codes();
        self.log_enum_attempt();
    }

    fn log_enum_attempt(&self) {
        if self.enum_location.is_hub {
            serial::write_fmt(format_args!(
                "usb-enum: ENUM HUB{} P{} SPD{} SLOT{} STAGE {} CMDCC{} XFERCC{} {:04X}:{:04X} CLS{:02X} MPS{} ERR {}\r\n",
                self.enum_location.hub_slot,
                self.enum_location.port,
                self.enum_speed,
                self.enum_slot_id,
                self.enum_stage,
                self.enum_cmd_cc,
                self.enum_xfer_cc,
                self.enum_vid,
                self.enum_pid,
                self.enum_dev_class,
                self.enum_ep0_mps,
                self.enum_err
            ));
        } else {
            serial::write_fmt(format_args!(
                "usb-enum: ENUM ROOT P{} SPD{} SLOT{} STAGE {} CMDCC{} XFERCC{} {:04X}:{:04X} CLS{:02X} MPS{} ERR {}\r\n",
                self.enum_location.port,
                self.enum_speed,
                self.enum_slot_id,
                self.enum_stage,
                self.enum_cmd_cc,
                self.enum_xfer_cc,
                self.enum_vid,
                self.enum_pid,
                self.enum_dev_class,
                self.enum_ep0_mps,
                self.enum_err
            ));
        }
    }

    unsafe fn initialise_hid_devices(&mut self) -> Result<(), &'static str> {
        self.power_root_ports();
        let mut wait_round = 0usize;
        while self.count_connected_ports() == 0 && wait_round < 3 {
            spin_delay(ROOT_PORT_POWER_SETTLE_ITERS);
            wait_round += 1;
        }
        self.root_connected_mask = self.read_root_connected_mask();

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

            self.begin_enum_reset(UsbEnumLocation::root(port));
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
                        self.last_enum_error = Some(err);
                    }
                },
                Err(err) => {
                    serial::write_fmt(format_args!("usb-xhci: port {} reset: {}\r\n", port, err));
                    self.last_enum_error = Some(err);
                    self.fail_enum_attempt(err);
                }
            }

            port += 1;
        }

        self.next_device_index = next_device_index;
        self.root_connected_mask = self.read_root_connected_mask();
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
        let location = self.begin_enum_attempt(port);
        let result = self.enumerate_device_inner(port, next_device_index);
        self.finish_enum_attempt(location, result)
    }

    unsafe fn enumerate_device_inner(
        &mut self,
        port: PortInfo,
        next_device_index: &mut usize,
    ) -> Result<EnumeratedKind, &'static str> {
        if *next_device_index >= MAX_HID_DEVICES {
            return Err("device storage exhausted");
        }
        let device_index = *next_device_index;
        *next_device_index += 1;

        self.set_enum_stage("slot");
        let slot_result = self.enable_slot();
        self.capture_enum_completion_codes();
        let slot_id = slot_result?;
        self.enum_slot_id = slot_id;
        self.current_device_index = device_index;
        self.current_slot_id = slot_id;
        self.reset_ep0_ring();
        self.set_enum_stage("address");
        let prepare_result = self.prepare_address_context(device_index, slot_id, port);
        self.capture_enum_completion_codes();
        prepare_result?;
        self.set_enum_stage("address");
        let input_phys_result = phys_of(ptr::addr_of!(INPUT_CONTEXT.0[0]), "input context phys");
        self.capture_enum_completion_codes();
        let input_phys = input_phys_result?;
        self.set_enum_stage("address");
        let address_result = self.execute_command(Trb {
            parameter: input_phys,
            status: 0,
            control: trb_type(TRB_TYPE_ADDRESS_DEVICE) | ((slot_id as u32) << 24),
        });
        self.capture_enum_completion_codes();
        if port.parent_hub_slot_id != 0 {
            match address_result {
                Ok(_) => serial::write_fmt(format_args!(
                    "usb-hub: address-device ok speed {}\r\n",
                    port.speed
                )),
                Err(_) if self.last_command_type == TRB_TYPE_ADDRESS_DEVICE as u8 => {
                    serial::write_fmt(format_args!(
                        "usb-hub: address-device failed speed {} cc {}\r\n",
                        port.speed, self.last_completion_code
                    ));
                }
                Err(_) => {}
            }
        }
        address_result?;

        self.set_enum_stage("ep0-mps");
        let ep0_mps_result = self.correct_ep0_max_packet_size(device_index, slot_id, port);
        self.capture_enum_completion_codes();
        ep0_mps_result?;

        self.set_enum_stage("devdesc");
        let device_desc_result = self.get_device_descriptor();
        self.capture_enum_completion_codes();
        let device_desc = device_desc_result?;
        self.last_device_vid = u16::from_le_bytes([device_desc[8], device_desc[9]]);
        self.last_device_pid = u16::from_le_bytes([device_desc[10], device_desc[11]]);
        self.last_device_class = device_desc[4];
        self.enum_vid = self.last_device_vid;
        self.enum_pid = self.last_device_pid;
        self.enum_dev_class = self.last_device_class;
        serial::write_fmt(format_args!(
            "usb-hid: device VID:PID {:04x}:{:04x} class {:02x}\r\n",
            self.last_device_vid, self.last_device_pid, self.last_device_class
        ));
        let ep0_mps = descriptor_ep0_mps(port.speed, device_desc[7]);
        self.enum_ep0_mps = ep0_mps;
        if ep0_mps != default_ep0_mps(port.speed) {
            serial::write_fmt(format_args!(
                "usb-xhci: ep0 mps descriptor {} initial {}\r\n",
                ep0_mps,
                default_ep0_mps(port.speed)
            ));
        }

        let is_hub = if device_desc[4] == USB_CLASS_HUB {
            true
        } else {
            let hub_result = self.configuration_has_hub_interface();
            self.capture_enum_completion_codes();
            hub_result?
        };
        if is_hub {
            let hub_result = self.configure_hub(port, slot_id, device_index, next_device_index);
            self.capture_enum_completion_codes();
            hub_result?;
            return Ok(EnumeratedKind::Hub);
        }

        let endpoint_result = self.find_boot_hid_endpoint();
        self.capture_enum_completion_codes();
        let endpoint = match endpoint_result {
            Ok(endpoint) => endpoint,
            Err(hid_err) => {
                let storage_result = self.find_mass_storage_interface();
                self.capture_enum_completion_codes();
                match storage_result {
                    Ok(storage) => {
                        self.set_enum_stage("set-config");
                        let set_config_result = self.control_no_data(
                            0x00,
                            USB_REQ_SET_CONFIGURATION,
                            storage.configuration_value as u16,
                            0,
                        );
                        self.capture_enum_completion_codes();
                        set_config_result?;
                        self.set_enum_stage("cfg-msc");
                        let cfg_result =
                            self.configure_mass_storage(device_index, slot_id, port, storage);
                        self.capture_enum_completion_codes();
                        cfg_result?;
                        serial::write_fmt(format_args!(
                            "usb-msc: ready on slot {} bulk-in 0x{:02x} bulk-out 0x{:02x}\r\n",
                            slot_id,
                            storage.bulk_in_endpoint_address,
                            storage.bulk_out_endpoint_address
                        ));
                        return Ok(EnumeratedKind::MassStorage);
                    }
                    Err(_) => return Err(hid_err),
                }
            }
        };
        if endpoint.kind.is_keyboard() && self.keyboard.is_some() {
            return Err("duplicate USB boot keyboard");
        }
        if endpoint.kind.is_pointer() && self.mouse.is_some() {
            return Err("duplicate USB pointer");
        }
        self.set_enum_stage("set-config");
        let set_config_result = self.control_no_data(
            0x00,
            USB_REQ_SET_CONFIGURATION,
            endpoint.configuration_value as u16,
            0,
        );
        self.capture_enum_completion_codes();
        set_config_result?;
        if endpoint.kind.uses_boot_protocol() {
            self.set_enum_stage("set-proto");
            let set_proto_result =
                self.control_no_data(0x21, HID_REQ_SET_PROTOCOL, 0, endpoint.interface_number as u16);
            self.capture_enum_completion_codes();
            if set_proto_result.is_err() {
                serial::write_line("usb-hid: set-protocol stalled (ignored)");
            }
        }
        self.set_enum_stage("set-idle");
        let set_idle_result =
            self.control_no_data(0x21, HID_REQ_SET_IDLE, 0, endpoint.interface_number as u16);
        self.capture_enum_completion_codes();
        if set_idle_result.is_err() {
            serial::write_line("usb-hid: set-idle stalled (ignored)");
        }

        self.set_enum_stage("cfg-ep");
        let cfg_ep_result = self.configure_interrupt_endpoint(device_index, slot_id, port, endpoint);
        self.capture_enum_completion_codes();
        cfg_ep_result?;
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

    unsafe fn correct_ep0_max_packet_size(
        &mut self,
        device_index: usize,
        slot_id: u8,
        port: PortInfo,
    ) -> Result<(), &'static str> {
        let len = self.control_in(
            0x80,
            USB_REQ_GET_DESCRIPTOR,
            (DESC_DEVICE as u16) << 8,
            0,
            8,
        )?;
        if len < 8 {
            return Err("short device descriptor header");
        }
        if CONTROL_BUFFER.0[1] != DESC_DEVICE {
            return Err("unexpected device descriptor type");
        }

        let default_mps = default_ep0_mps(port.speed);
        let real_mps = descriptor_ep0_mps(port.speed, CONTROL_BUFFER.0[7]);
        self.enum_ep0_mps = real_mps;
        if !valid_ep0_mps(port.speed, real_mps) {
            return Err("invalid ep0 max packet size");
        }
        if real_mps == default_mps {
            return Ok(());
        }

        clear_input_context();
        input_control_add_flags(1 << DCI_EP0);
        write_endpoint_context(
            DCI_EP0,
            self.context_size,
            EP_TYPE_CONTROL,
            0,
            real_mps,
            phys_of(ptr::addr_of!(EP0_RINGS[device_index].0[0]), "ep0 ring phys")? | 1,
            8,
            0,
        );
        fence(Ordering::SeqCst);
        let input_phys = phys_of(ptr::addr_of!(INPUT_CONTEXT.0[0]), "input context phys")?;
        self.execute_command(Trb {
            parameter: input_phys,
            status: 0,
            control: trb_type(TRB_TYPE_EVALUATE_CONTEXT) | ((slot_id as u32) << 24),
        })?;
        serial::write_fmt(format_args!(
            "usb-enum: ep0 mps corrected {} -> {}\r\n",
            default_mps, real_mps
        ));
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
        self.set_enum_stage("config-hdr");
        let header_result = self.control_in(
            0x80,
            USB_REQ_GET_DESCRIPTOR,
            (DESC_CONFIGURATION as u16) << 8,
            0,
            9,
        );
        self.capture_enum_completion_codes();
        let header_len = header_result?;
        if header_len < 9 || CONTROL_BUFFER.0[1] != DESC_CONFIGURATION {
            return Err("configuration header unavailable");
        }
        Ok(CONTROL_BUFFER.0[5])
    }

    unsafe fn configuration_has_hub_interface(&mut self) -> Result<bool, &'static str> {
        self.set_enum_stage("config-hdr");
        let header_result = self.control_in(
            0x80,
            USB_REQ_GET_DESCRIPTOR,
            (DESC_CONFIGURATION as u16) << 8,
            0,
            9,
        );
        self.capture_enum_completion_codes();
        let header_len = header_result?;
        if header_len < 9 || CONTROL_BUFFER.0[1] != DESC_CONFIGURATION {
            return Err("configuration header unavailable");
        }
        let total_len = u16::from_le_bytes([CONTROL_BUFFER.0[2], CONTROL_BUFFER.0[3]]) as usize;
        let config_len = usize::min(total_len, CONTROL_BUFFER_LEN);
        self.set_enum_stage("config");
        let config_result = self.control_in(
            0x80,
            USB_REQ_GET_DESCRIPTOR,
            (DESC_CONFIGURATION as u16) << 8,
            0,
            config_len,
        );
        self.capture_enum_completion_codes();
        let actual_len = config_result?;

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
        self.set_enum_stage("set-config");
        let set_config_result =
            self.control_no_data(0x00, USB_REQ_SET_CONFIGURATION, config_value as u16, 0);
        self.capture_enum_completion_codes();
        if let Err(err) = set_config_result {
            self.hub_last_error = Some(err);
            return Err(err);
        }
        self.set_enum_stage("config");
        let descriptor = match self.hub_descriptor() {
            Ok(descriptor) => descriptor,
            Err(err) => {
                self.capture_enum_completion_codes();
                self.hub_last_error = Some(err);
                return Err(err);
            }
        };
        self.capture_enum_completion_codes();
        self.hub_count = self.hub_count.saturating_add(1);
        self.hub_ports = self.hub_ports.saturating_add(descriptor.port_count);

        serial::write_fmt(format_args!(
            "usb-hub: slot {} ports {} pwr-good {}ms\r\n",
            slot_id, descriptor.port_count, descriptor.power_on_delay_ms
        ));

        self.set_enum_stage("config");
        let evaluate_result = self.evaluate_hub_slot(port, slot_id, descriptor.port_count);
        self.capture_enum_completion_codes();
        if let Err(err) = evaluate_result {
            self.hub_last_error = Some(err);
            return Err(err);
        }

        let mut connected_mask = 0u32;
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
            connected_mask |= port_mask(hub_port);
            self.hub_connected_ports = self.hub_connected_ports.saturating_add(1);
            if *next_device_index >= MAX_HID_DEVICES {
                serial::write_line("usb-hub: device storage exhausted");
                self.hub_last_error = Some("usb-hub: device storage exhausted");
                break;
            }

            self.begin_enum_reset(UsbEnumLocation::hub(slot_id, hub_port));
            match self.reset_hub_port(hub_port) {
                Ok(port_status) => {
                    self.hub_reset_ports = self.hub_reset_ports.saturating_add(1);
                    match self.enumerate_hub_child(
                        port,
                        slot_id,
                        hub_port,
                        port_status,
                        next_device_index,
                    ) {
                        Ok(kind) => {
                            self.hub_configured_devices =
                                self.hub_configured_devices.saturating_add(1);
                            serial::write_fmt(format_args!(
                                "usb-hub: port {} configured as {}\r\n",
                                hub_port,
                                kind.as_str()
                            ));
                        }
                        Err(err) => {
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
                    self.fail_enum_attempt(err);
                    serial::write_fmt(format_args!(
                        "usb-hub: port {} reset: {}\r\n",
                        hub_port, err
                    ));
                }
            }

            hub_port += 1;
        }

        self.remember_hub(
            port,
            slot_id,
            hub_device_index,
            descriptor.port_count,
            connected_mask,
        );
        Ok(())
    }

    unsafe fn enumerate_hub_child(
        &mut self,
        hub: PortInfo,
        hub_slot_id: u8,
        hub_port: u8,
        port_status: HubPortStatus,
        next_device_index: &mut usize,
    ) -> Result<EnumeratedKind, &'static str> {
        let primary_speed = hub_port_speed(port_status.status, hub.speed);
        let (speeds, speed_count) = child_speed_candidates(primary_speed);
        let mut speed_index = 0usize;
        let mut last_err = None;

        while speed_index < speed_count {
            let child_speed = speeds[speed_index];
            let child_port = hub.child(hub_slot_id, hub_port, child_speed);
            let tt = if slot_context_tt_fields(child_port) != 0 {
                "applied"
            } else {
                "none"
            };
            serial::write_fmt(format_args!(
                "usb-hub: port {} reset complete speed {} try {}\r\n",
                hub_port,
                child_speed,
                speed_index + 1
            ));
            serial::write_fmt(format_args!(
                "usb-hub: child port {} speed {} tt {}\r\n",
                hub_port, child_speed, tt
            ));
            match self.enumerate_device(child_port, next_device_index) {
                Ok(kind) => return Ok(kind),
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

        Err(last_err.unwrap_or("hub child enumeration failed"))
    }

    fn should_retry_child_enumeration(&self, err: &'static str) -> bool {
        err == "xHCI command failed"
            && self.last_command_type == TRB_TYPE_ADDRESS_DEVICE as u8
            && self.last_completion_code == 4
    }

    fn remember_hub(
        &mut self,
        port: PortInfo,
        slot_id: u8,
        device_index: usize,
        port_count: u8,
        connected_mask: u32,
    ) {
        let mut index = 0usize;
        let mut free = None;
        while index < MAX_HUB_WATCHES {
            let watch = self.hub_watches[index];
            if watch.active && watch.slot_id == slot_id {
                free = Some(index);
                break;
            }
            if !watch.active && free.is_none() {
                free = Some(index);
            }
            index += 1;
        }

        if let Some(index) = free {
            self.hub_watches[index] = HubWatch {
                active: true,
                slot_id,
                device_index,
                root_port_number: port.root_port_number,
                route_string: port.route_string,
                speed: port.speed,
                parent_hub_slot_id: port.parent_hub_slot_id,
                parent_hub_port_number: port.parent_hub_port_number,
                port_count: u8::min(port_count, HOTPLUG_MAX_ROOT_PORTS),
                connected_mask,
            };
        } else {
            self.hub_last_error = Some("usb-hub: watch storage exhausted");
        }
    }

    unsafe fn poll_hotplug(&mut self) -> bool {
        let mut changed = false;
        let root_current = self.read_root_connected_mask_and_clear_changes();
        let root_diff = self.root_connected_mask ^ root_current;
        let mut port = 1u8;
        let root_limit = u8::min(self.max_ports, HOTPLUG_MAX_ROOT_PORTS);
        while port <= root_limit {
            let bit = port_mask(port);
            if root_diff & bit != 0 {
                if root_current & bit != 0 {
                    let (detail, skipped) = self.configure_root_hotplug(port);
                    self.record_hotplug_connect(false, 0, port, detail, skipped);
                } else {
                    self.clear_devices_for_root_port(port);
                    self.deactivate_hubs_for_root_port(port);
                    self.record_hotplug_disconnect(false, 0, port);
                }
                changed = true;
            }
            port += 1;
        }
        self.root_connected_mask = root_current;

        let mut index = 0usize;
        while index < MAX_HUB_WATCHES {
            let watch = self.hub_watches[index];
            if watch.active {
                match self.read_hub_connected_mask_and_clear_changes(watch) {
                    Ok(current) => {
                        let diff = watch.connected_mask ^ current;
                        let mut hub_port = 1u8;
                        while hub_port <= watch.port_count {
                            let bit = port_mask(hub_port);
                            if diff & bit != 0 {
                                if current & bit != 0 {
                                    let (detail, skipped) =
                                        self.configure_hub_hotplug(watch, hub_port);
                                    self.record_hotplug_connect(
                                        true,
                                        watch.slot_id,
                                        hub_port,
                                        detail,
                                        skipped,
                                    );
                                } else {
                                    self.clear_devices_for_hub_port(watch.slot_id, hub_port);
                                    self.deactivate_child_hubs(watch.slot_id, hub_port);
                                    self.record_hotplug_disconnect(true, watch.slot_id, hub_port);
                                }
                                changed = true;
                            }
                            hub_port += 1;
                        }
                        if self.hub_watches[index].active {
                            self.hub_watches[index].connected_mask = current;
                        }
                    }
                    Err(err) => {
                        self.hub_last_error = Some(err);
                    }
                }
            }
            index += 1;
        }

        changed
    }

    unsafe fn read_root_connected_mask_and_clear_changes(&self) -> u32 {
        let mut mask = 0u32;
        let mut port = 1u8;
        let limit = u8::min(self.max_ports, HOTPLUG_MAX_ROOT_PORTS);
        while port <= limit {
            let offset = self.portsc_offset(port);
            let status = read32(self.base, offset);
            if status & PORTSC_CCS != 0 {
                mask |= port_mask(port);
            }
            if status & PORTSC_CHANGE_BITS != 0 {
                self.write_portsc(offset, PORTSC_CHANGE_BITS | (status & PORTSC_PP));
            }
            port += 1;
        }
        mask
    }

    unsafe fn read_hub_connected_mask_and_clear_changes(
        &mut self,
        watch: HubWatch,
    ) -> Result<u32, &'static str> {
        self.current_slot_id = watch.slot_id;
        self.current_device_index = watch.device_index;
        let mut mask = 0u32;
        let mut port = 1u8;
        while port <= watch.port_count {
            let status = self.hub_port_status(port)?;
            if status.status & HUB_PORT_CONNECTION != 0 {
                mask |= port_mask(port);
            }
            if status.change & HUB_CHANGE_C_PORT_CONNECTION != 0 {
                let _ = self.hub_clear_feature(port, HUB_FEATURE_C_PORT_CONNECTION);
            }
            port += 1;
        }
        Ok(mask)
    }

    unsafe fn configure_root_hotplug(&mut self, port: u8) -> (&'static str, bool) {
        if self.next_device_index >= MAX_HID_DEVICES {
            return ("device storage exhausted", true);
        }
        self.begin_enum_reset(UsbEnumLocation::root(port));
        match self.reset_port(port) {
            Ok(port_info) => self.enumerate_hotplug_device(port_info),
            Err(err) => {
                self.last_enum_error = Some(err);
                self.fail_enum_attempt(err);
                (err, true)
            }
        }
    }

    unsafe fn configure_hub_hotplug(
        &mut self,
        watch: HubWatch,
        hub_port: u8,
    ) -> (&'static str, bool) {
        if self.next_device_index >= MAX_HID_DEVICES {
            return ("device storage exhausted", true);
        }
        self.current_slot_id = watch.slot_id;
        self.current_device_index = watch.device_index;
        self.begin_enum_reset(UsbEnumLocation::hub(watch.slot_id, hub_port));
        match self.reset_hub_port(hub_port) {
            Ok(port_status) => {
                self.hub_reset_ports = self.hub_reset_ports.saturating_add(1);
                let mut next = self.next_device_index;
                let result = self.enumerate_hub_child(
                    watch.port_info(),
                    watch.slot_id,
                    hub_port,
                    port_status,
                    &mut next,
                );
                self.next_device_index = next;
                match result {
                    Ok(kind) => {
                        self.hub_configured_devices = self.hub_configured_devices.saturating_add(1);
                        let _ = self.queue_keyboard_report();
                        let _ = self.queue_mouse_report();
                        (kind.as_str(), false)
                    }
                    Err(err) => {
                        self.hub_last_error = Some(err);
                        (err, true)
                    }
                }
            }
            Err(err) => {
                self.hub_last_error = Some(err);
                self.fail_enum_attempt(err);
                (err, true)
            }
        }
    }

    unsafe fn enumerate_hotplug_device(&mut self, port: PortInfo) -> (&'static str, bool) {
        let mut next = self.next_device_index;
        let result = self.enumerate_device(port, &mut next);
        self.next_device_index = next;
        match result {
            Ok(kind) => {
                let _ = self.queue_keyboard_report();
                let _ = self.queue_mouse_report();
                (kind.as_str(), false)
            }
            Err(err) => {
                self.last_enum_error = Some(err);
                (err, true)
            }
        }
    }

    fn clear_devices_for_root_port(&mut self, port: u8) {
        if self
            .keyboard
            .is_some_and(|keyboard| keyboard.root_port_number == port)
        {
            self.keyboard = None;
        }
        if self
            .mouse
            .is_some_and(|mouse| mouse.root_port_number == port)
        {
            self.mouse = None;
        }
        if self
            .mass_storage
            .is_some_and(|storage| storage.root_port_number == port)
        {
            self.mass_storage = None;
            self.mass_storage_probe = MassStorageProbe::none();
        }
    }

    fn clear_devices_for_hub_port(&mut self, hub_slot: u8, port: u8) {
        if self.keyboard.is_some_and(|keyboard| {
            keyboard.parent_hub_slot_id == hub_slot && keyboard.parent_hub_port_number == port
        }) {
            self.keyboard = None;
        }
        if self.mouse.is_some_and(|mouse| {
            mouse.parent_hub_slot_id == hub_slot && mouse.parent_hub_port_number == port
        }) {
            self.mouse = None;
        }
        if self.mass_storage.is_some_and(|storage| {
            storage.parent_hub_slot_id == hub_slot && storage.parent_hub_port_number == port
        }) {
            self.mass_storage = None;
            self.mass_storage_probe = MassStorageProbe::none();
        }
    }

    fn deactivate_hubs_for_root_port(&mut self, port: u8) {
        let mut index = 0usize;
        while index < MAX_HUB_WATCHES {
            if self.hub_watches[index].active && self.hub_watches[index].root_port_number == port {
                self.hub_watches[index].active = false;
            }
            index += 1;
        }
    }

    fn deactivate_child_hubs(&mut self, hub_slot: u8, hub_port: u8) {
        let mut index = 0usize;
        while index < MAX_HUB_WATCHES {
            let watch = self.hub_watches[index];
            if watch.active
                && watch.parent_hub_slot_id == hub_slot
                && watch.parent_hub_port_number == hub_port
            {
                self.hub_watches[index].active = false;
                self.clear_devices_for_hub_slot(watch.slot_id);
                self.deactivate_descendant_hubs(watch.slot_id);
            }
            index += 1;
        }
    }

    fn deactivate_descendant_hubs(&mut self, parent_slot: u8) {
        let mut index = 0usize;
        while index < MAX_HUB_WATCHES {
            let watch = self.hub_watches[index];
            if watch.active && watch.parent_hub_slot_id == parent_slot {
                self.hub_watches[index].active = false;
                self.clear_devices_for_hub_slot(watch.slot_id);
                self.deactivate_descendant_hubs(watch.slot_id);
            }
            index += 1;
        }
    }

    fn clear_devices_for_hub_slot(&mut self, hub_slot: u8) {
        if self
            .keyboard
            .is_some_and(|keyboard| keyboard.parent_hub_slot_id == hub_slot)
        {
            self.keyboard = None;
        }
        if self
            .mouse
            .is_some_and(|mouse| mouse.parent_hub_slot_id == hub_slot)
        {
            self.mouse = None;
        }
        if self
            .mass_storage
            .is_some_and(|storage| storage.parent_hub_slot_id == hub_slot)
        {
            self.mass_storage = None;
            self.mass_storage_probe = MassStorageProbe::none();
        }
    }

    fn current_hub_connected_ports(&self) -> u8 {
        let mut count = 0u8;
        let mut index = 0usize;
        while index < MAX_HUB_WATCHES {
            if self.hub_watches[index].active {
                count = count.saturating_add(count_mask(self.hub_watches[index].connected_mask));
            }
            index += 1;
        }
        count
    }

    fn record_hotplug_connect(
        &mut self,
        is_hub: bool,
        hub_slot: u8,
        port: u8,
        detail: &'static str,
        skipped: bool,
    ) {
        self.hotplug = HotplugRecord {
            seq: self.hotplug.seq.wrapping_add(1),
            present: true,
            connected: true,
            is_hub,
            hub_slot,
            port,
            detail,
            skipped,
        };
        if is_hub {
            if skipped {
                serial::write_fmt(format_args!(
                    "usb-hotplug: connect hub {} port {} -> skipped: {}\r\n",
                    hub_slot, port, detail
                ));
            } else {
                serial::write_fmt(format_args!(
                    "usb-hotplug: connect hub {} port {} -> {}\r\n",
                    hub_slot, port, detail
                ));
            }
        } else if skipped {
            serial::write_fmt(format_args!(
                "usb-hotplug: connect root-port {} -> skipped: {}\r\n",
                port, detail
            ));
        } else {
            serial::write_fmt(format_args!(
                "usb-hotplug: connect root-port {} -> {}\r\n",
                port, detail
            ));
        }
    }

    fn record_hotplug_disconnect(&mut self, is_hub: bool, hub_slot: u8, port: u8) {
        self.hotplug = HotplugRecord {
            seq: self.hotplug.seq.wrapping_add(1),
            present: true,
            connected: false,
            is_hub,
            hub_slot,
            port,
            detail: "removed",
            skipped: false,
        };
        if is_hub {
            serial::write_fmt(format_args!(
                "usb-hotplug: disconnect hub {} port {}\r\n",
                hub_slot, port
            ));
        } else {
            serial::write_fmt(format_args!(
                "usb-hotplug: disconnect root-port {}\r\n",
                port
            ));
        }
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
        self.set_enum_stage("config-hdr");
        let header_result = self.control_in(
            0x80,
            USB_REQ_GET_DESCRIPTOR,
            (DESC_CONFIGURATION as u16) << 8,
            0,
            9,
        );
        self.capture_enum_completion_codes();
        let header_len = header_result?;
        if header_len < 9 || CONTROL_BUFFER.0[1] != DESC_CONFIGURATION {
            return Err("configuration header unavailable");
        }
        let total_len = u16::from_le_bytes([CONTROL_BUFFER.0[2], CONTROL_BUFFER.0[3]]) as usize;
        let config_len = usize::min(total_len, CONTROL_BUFFER_LEN);
        self.set_enum_stage("config");
        let config_result = self.control_in(
            0x80,
            USB_REQ_GET_DESCRIPTOR,
            (DESC_CONFIGURATION as u16) << 8,
            0,
            config_len,
        );
        self.capture_enum_completion_codes();
        let actual_len = config_result?;
        self.set_enum_stage("hid-match");
        let endpoint_result =
            parse_boot_hid_endpoint(&CONTROL_BUFFER.0[..actual_len], self.keyboard.is_none());
        self.capture_enum_completion_codes();
        endpoint_result
    }

    unsafe fn find_mass_storage_interface(&mut self) -> Result<MassStorageInterface, &'static str> {
        self.set_enum_stage("config-hdr");
        let header_result = self.control_in(
            0x80,
            USB_REQ_GET_DESCRIPTOR,
            (DESC_CONFIGURATION as u16) << 8,
            0,
            9,
        );
        self.capture_enum_completion_codes();
        let header_len = header_result?;
        if header_len < 9 || CONTROL_BUFFER.0[1] != DESC_CONFIGURATION {
            return Err("configuration header unavailable");
        }
        let total_len = u16::from_le_bytes([CONTROL_BUFFER.0[2], CONTROL_BUFFER.0[3]]) as usize;
        let config_len = usize::min(total_len, CONTROL_BUFFER_LEN);
        self.set_enum_stage("config");
        let config_result = self.control_in(
            0x80,
            USB_REQ_GET_DESCRIPTOR,
            (DESC_CONFIGURATION as u16) << 8,
            0,
            config_len,
        );
        self.capture_enum_completion_codes();
        let actual_len = config_result?;
        self.set_enum_stage("msc-match");
        parse_mass_storage_interface(&CONTROL_BUFFER.0[..actual_len])
    }

    unsafe fn configure_mass_storage(
        &mut self,
        device_index: usize,
        slot_id: u8,
        port: PortInfo,
        storage: MassStorageInterface,
    ) -> Result<(), &'static str> {
        if self.mass_storage.is_some() {
            return Err("duplicate USB mass storage");
        }

        self.reset_bulk_rings_at(device_index);
        clear_input_context();
        input_control_add_flags(
            (1 << 0) | (1 << storage.bulk_in_dci) | (1 << storage.bulk_out_dci),
        );
        write_slot_context(
            port,
            u8::max(storage.bulk_in_dci, storage.bulk_out_dci),
            self.context_size,
        );
        write_endpoint_context(
            storage.bulk_out_dci,
            self.context_size,
            EP_TYPE_BULK_OUT,
            0,
            storage.bulk_out_max_packet_size,
            phys_of(
                ptr::addr_of!(BULK_OUT_RINGS[device_index].0[0]),
                "bulk out ring phys",
            )? | 1,
            storage.bulk_out_max_packet_size,
            0,
        );
        write_endpoint_context(
            storage.bulk_in_dci,
            self.context_size,
            EP_TYPE_BULK_IN,
            0,
            storage.bulk_in_max_packet_size,
            phys_of(
                ptr::addr_of!(BULK_IN_RINGS[device_index].0[0]),
                "bulk in ring phys",
            )? | 1,
            storage.bulk_in_max_packet_size,
            0,
        );
        fence(Ordering::SeqCst);

        let input_phys = phys_of(ptr::addr_of!(INPUT_CONTEXT.0[0]), "msc input context phys")?;
        self.execute_command(Trb {
            parameter: input_phys,
            status: 0,
            control: trb_type(TRB_TYPE_CONFIGURE_ENDPOINT) | ((slot_id as u32) << 24),
        })?;

        self.mass_storage = Some(MassStorageDevice {
            slot_id,
            bulk_in_dci: storage.bulk_in_dci,
            bulk_out_dci: storage.bulk_out_dci,
            ring_index: device_index,
            root_port_number: port.root_port_number,
            parent_hub_slot_id: port.parent_hub_slot_id,
            parent_hub_port_number: port.parent_hub_port_number,
            tag: 1,
        });
        self.mass_storage_probe.present = true;
        match self.probe_mass_storage_layout() {
            Ok(probe) => self.mass_storage_probe = probe,
            Err(err) => {
                self.mass_storage_probe = MassStorageProbe {
                    present: true,
                    detail: err,
                    ..MassStorageProbe::none()
                };
                serial::write_fmt(format_args!("usb-msc: read-only probe failed: {}\r\n", err));
            }
        }
        Ok(())
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
                    root_port_number: port.root_port_number,
                    parent_hub_slot_id: port.parent_hub_slot_id,
                    parent_hub_port_number: port.parent_hub_port_number,
                    previous_report: [0; KEYBOARD_REPORT_LEN],
                });
            }
            HidKind::Mouse => {
                self.mouse = Some(MouseDevice {
                    slot_id,
                    dci: endpoint.dci,
                    ring_index: device_index,
                    kind: endpoint.kind,
                    root_port_number: port.root_port_number,
                    parent_hub_slot_id: port.parent_hub_slot_id,
                    parent_hub_port_number: port.parent_hub_port_number,
                });
            }
            HidKind::Tablet => {
                self.mouse = Some(MouseDevice {
                    slot_id,
                    dci: endpoint.dci,
                    ring_index: device_index,
                    kind: endpoint.kind,
                    root_port_number: port.root_port_number,
                    parent_hub_slot_id: port.parent_hub_slot_id,
                    parent_hub_port_number: port.parent_hub_port_number,
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
        self.last_transfer_completion_code = 0;
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
        self.last_transfer_completion_code = cc as u8;
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
        self.last_command_type = command_type as u8;
        self.last_completion_code = 0;
        if self.command_enqueue >= COMMAND_RING_LEN {
            return Err("command ring index invalid");
        }
        if self.command_enqueue == COMMAND_RING_LEN - 1 {
            let link_phys = phys_of(
                ptr::addr_of!(COMMAND_RING.0[0]),
                "command ring link phys",
            )?;
            let mut link = Trb {
                parameter: link_phys,
                status: 0,
                control: TRB_LINK_TOGGLE_CYCLE | trb_type(TRB_TYPE_LINK),
            };
            if self.command_cycle {
                link.control |= TRB_CYCLE;
            }
            ptr::write_volatile(
                ptr::addr_of_mut!(COMMAND_RING.0[self.command_enqueue]),
                link,
            );
            self.command_enqueue = 0;
            self.command_cycle = !self.command_cycle;
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
                    self.last_completion_code = cc as u8;
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

    unsafe fn probe_mass_storage_layout(&mut self) -> Result<MassStorageProbe, &'static str> {
        self.msc_scsi_command(36, true, &[0x12, 0, 0, 0, 36, 0])?;
        let cap_len = self.msc_scsi_command(
            MSC_READ_CAPACITY10_LEN,
            true,
            &[0x25, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        )?;
        if cap_len < MSC_READ_CAPACITY10_LEN {
            return Err("msc_read_capacity_short");
        }
        let last_lba = u32::from_be_bytes([
            MSC_BUFFER.0[0],
            MSC_BUFFER.0[1],
            MSC_BUFFER.0[2],
            MSC_BUFFER.0[3],
        ]);
        let block_size = u32::from_be_bytes([
            MSC_BUFFER.0[4],
            MSC_BUFFER.0[5],
            MSC_BUFFER.0[6],
            MSC_BUFFER.0[7],
        ]);
        if block_size != SECTOR_SIZE as u32 {
            return Ok(MassStorageProbe {
                present: true,
                read_completed: true,
                seed_data_present: false,
                block_size,
                last_lba,
                lba0_signature: 0,
                detail: "msc_block_size_not_512",
            });
        }

        let mbr = self.msc_read_sector_copy(0)?;
        let lba0_signature = u16::from_le_bytes([mbr[510], mbr[511]]);
        let header = self.msc_read_sector_copy(1)?;
        let mut entries = vec![0u8; GPT_ENTRY_ARRAY_BYTES];
        let mut sector_idx = 0usize;
        while sector_idx < GPT_ENTRY_ARRAY_BYTES / SECTOR_SIZE {
            let sector = self.msc_read_sector_copy(PRIMARY_GPT_ENTRIES_LBA + sector_idx as u64)?;
            let out = sector_idx * SECTOR_SIZE;
            entries[out..out + SECTOR_SIZE].copy_from_slice(&sector);
            sector_idx += 1;
        }

        let gpt = gpt_layout::validate_gpt_layout(&mbr, &header, &entries);
        if gpt.status != LayoutStatus::Present {
            return Ok(MassStorageProbe {
                present: true,
                read_completed: true,
                seed_data_present: false,
                block_size,
                last_lba,
                lba0_signature,
                detail: gpt.reason,
            });
        }

        let Some(seed_data_lba1) = gpt.seed_data_first_lba.checked_add(1) else {
            return Ok(MassStorageProbe {
                present: true,
                read_completed: true,
                seed_data_present: false,
                block_size,
                last_lba,
                lba0_signature,
                detail: "seed_data_first_lba_overflow",
            });
        };
        let sb0 = self.msc_read_sector_copy(gpt.seed_data_first_lba)?;
        let sb1 = self.msc_read_sector_copy(seed_data_lba1)?;
        let data = seed_data_layout::validate_seed_data_layout(&sb0, &sb1, gpt.seed_data_lba_count);
        let seed_data_present = data.status == LayoutStatus::Present;
        let probe = MassStorageProbe {
            present: true,
            read_completed: true,
            seed_data_present,
            block_size,
            last_lba,
            lba0_signature,
            detail: data.reason,
        };
        serial::write_fmt(format_args!(
            "usb-msc: capacity last_lba={} block={} mbr=0x{:04x} seed_data={} {}\r\n",
            last_lba,
            block_size,
            lba0_signature,
            if seed_data_present {
                "present"
            } else {
                "absent"
            },
            probe.detail
        ));
        Ok(probe)
    }

    unsafe fn msc_read_sector_copy(
        &mut self,
        lba: u64,
    ) -> Result<[u8; MSC_SECTOR_BUFFER_LEN], &'static str> {
        if lba > u32::MAX as u64 {
            return Err("msc_lba_out_of_read10_range");
        }
        let lba32 = lba as u32;
        let cdb = [
            0x28,
            0,
            (lba32 >> 24) as u8,
            (lba32 >> 16) as u8,
            (lba32 >> 8) as u8,
            lba32 as u8,
            0,
            0,
            1,
            0,
        ];
        let len = self.msc_scsi_command(MSC_SECTOR_BUFFER_LEN, true, &cdb)?;
        if len < MSC_SECTOR_BUFFER_LEN {
            return Err("msc_read10_short_sector");
        }
        let mut out = [0u8; MSC_SECTOR_BUFFER_LEN];
        let mut idx = 0usize;
        while idx < MSC_SECTOR_BUFFER_LEN {
            out[idx] = ptr::read_volatile(ptr::addr_of!(MSC_BUFFER.0[idx]));
            idx += 1;
        }
        Ok(out)
    }

    unsafe fn msc_scsi_command(
        &mut self,
        data_len: usize,
        input: bool,
        cdb: &[u8],
    ) -> Result<usize, &'static str> {
        if data_len > MSC_SECTOR_BUFFER_LEN {
            return Err("msc_data_buffer_too_small");
        }
        if cdb.is_empty() || cdb.len() > 16 {
            return Err("msc_cdb_len_invalid");
        }
        let Some(mut storage) = self.mass_storage else {
            return Err("msc_device_not_configured");
        };
        storage.tag = storage.tag.wrapping_add(1).max(1);
        self.mass_storage = Some(storage);

        ptr::write_bytes(MSC_CBW.0.as_mut_ptr(), 0, MSC_CBW_LEN);
        ptr::write_bytes(MSC_CSW.0.as_mut_ptr(), 0, MSC_CSW_LEN);
        ptr::write_bytes(MSC_BUFFER.0.as_mut_ptr(), 0, MSC_SECTOR_BUFFER_LEN);
        write_le32(&mut MSC_CBW.0, 0, MSC_CBW_SIGNATURE);
        write_le32(&mut MSC_CBW.0, 4, storage.tag);
        write_le32(&mut MSC_CBW.0, 8, data_len as u32);
        MSC_CBW.0[12] = if input { 0x80 } else { 0x00 };
        MSC_CBW.0[13] = 0;
        MSC_CBW.0[14] = cdb.len() as u8;
        let mut idx = 0usize;
        while idx < cdb.len() {
            MSC_CBW.0[15 + idx] = cdb[idx];
            idx += 1;
        }

        self.bulk_transfer(
            storage.slot_id,
            storage.bulk_out_dci,
            storage.ring_index,
            false,
            ptr::addr_of!(MSC_CBW.0[0]),
            MSC_CBW_LEN,
        )?;
        let mut transferred = 0usize;
        if data_len > 0 {
            transferred = self.bulk_transfer(
                storage.slot_id,
                if input {
                    storage.bulk_in_dci
                } else {
                    storage.bulk_out_dci
                },
                storage.ring_index,
                input,
                ptr::addr_of!(MSC_BUFFER.0[0]),
                data_len,
            )?;
        }
        self.bulk_transfer(
            storage.slot_id,
            storage.bulk_in_dci,
            storage.ring_index,
            true,
            ptr::addr_of!(MSC_CSW.0[0]),
            MSC_CSW_LEN,
        )?;

        let signature = read_le32(&MSC_CSW.0, 0);
        let tag = read_le32(&MSC_CSW.0, 4);
        let status = MSC_CSW.0[12];
        if signature != MSC_CSW_SIGNATURE || tag != storage.tag {
            return Err("msc_csw_invalid");
        }
        if status != 0 {
            return Err("msc_scsi_status_failed");
        }
        Ok(transferred)
    }

    unsafe fn bulk_transfer(
        &mut self,
        slot_id: u8,
        dci: u8,
        ring_index: usize,
        input: bool,
        buffer: *const u8,
        len: usize,
    ) -> Result<usize, &'static str> {
        let phys = phys_of(buffer, "bulk buffer phys")?;
        let trb = Trb {
            parameter: phys,
            status: len as u32,
            control: TRB_IOC | trb_type(TRB_TYPE_NORMAL),
        };
        if input {
            self.push_bulk_in_trb(ring_index, trb)?;
        } else {
            self.push_bulk_out_trb(ring_index, trb)?;
        }
        fence(Ordering::SeqCst);
        self.ring_doorbell(slot_id, dci);
        let event = self.wait_transfer_event(slot_id, dci)?;
        let cc = event.completion_code();
        self.last_transfer_completion_code = cc as u8;
        if cc != CC_SUCCESS && cc != CC_SHORT_PACKET {
            return Err("bulk transfer failed");
        }
        let residual = (event.status & 0x00FF_FFFF) as usize;
        Ok(len.saturating_sub(residual))
    }

    unsafe fn push_bulk_in_trb(
        &mut self,
        ring_index: usize,
        mut trb: Trb,
    ) -> Result<(), &'static str> {
        if ring_index >= MAX_HID_DEVICES {
            return Err("bulk in ring index out of range");
        }
        if self.bulk_in_enqueue[ring_index] == BULK_RING_LEN - 1 {
            let link_phys = phys_of(
                ptr::addr_of!(BULK_IN_RINGS[ring_index].0[0]),
                "bulk in ring link phys",
            )?;
            let mut link = Trb {
                parameter: link_phys,
                status: 0,
                control: TRB_LINK_TOGGLE_CYCLE | trb_type(TRB_TYPE_LINK),
            };
            if self.bulk_in_cycle[ring_index] {
                link.control |= TRB_CYCLE;
            }
            ptr::write_volatile(
                ptr::addr_of_mut!(BULK_IN_RINGS[ring_index].0[self.bulk_in_enqueue[ring_index]]),
                link,
            );
            self.bulk_in_enqueue[ring_index] = 0;
            self.bulk_in_cycle[ring_index] = !self.bulk_in_cycle[ring_index];
        }

        if self.bulk_in_cycle[ring_index] {
            trb.control |= TRB_CYCLE;
        } else {
            trb.control &= !TRB_CYCLE;
        }
        ptr::write_volatile(
            ptr::addr_of_mut!(BULK_IN_RINGS[ring_index].0[self.bulk_in_enqueue[ring_index]]),
            trb,
        );
        self.bulk_in_enqueue[ring_index] += 1;
        Ok(())
    }

    unsafe fn push_bulk_out_trb(
        &mut self,
        ring_index: usize,
        mut trb: Trb,
    ) -> Result<(), &'static str> {
        if ring_index >= MAX_HID_DEVICES {
            return Err("bulk out ring index out of range");
        }
        if self.bulk_out_enqueue[ring_index] == BULK_RING_LEN - 1 {
            let link_phys = phys_of(
                ptr::addr_of!(BULK_OUT_RINGS[ring_index].0[0]),
                "bulk out ring link phys",
            )?;
            let mut link = Trb {
                parameter: link_phys,
                status: 0,
                control: TRB_LINK_TOGGLE_CYCLE | trb_type(TRB_TYPE_LINK),
            };
            if self.bulk_out_cycle[ring_index] {
                link.control |= TRB_CYCLE;
            }
            ptr::write_volatile(
                ptr::addr_of_mut!(BULK_OUT_RINGS[ring_index].0[self.bulk_out_enqueue[ring_index]]),
                link,
            );
            self.bulk_out_enqueue[ring_index] = 0;
            self.bulk_out_cycle[ring_index] = !self.bulk_out_cycle[ring_index];
        }

        if self.bulk_out_cycle[ring_index] {
            trb.control |= TRB_CYCLE;
        } else {
            trb.control &= !TRB_CYCLE;
        }
        ptr::write_volatile(
            ptr::addr_of_mut!(BULK_OUT_RINGS[ring_index].0[self.bulk_out_enqueue[ring_index]]),
            trb,
        );
        self.bulk_out_enqueue[ring_index] += 1;
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
                    self.last_int_cc = cc as u8;
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
                    self.last_int_cc = cc as u8;
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

    unsafe fn reset_bulk_rings_at(&mut self, ring_index: usize) {
        ptr::write_bytes(
            ptr::addr_of_mut!(BULK_IN_RINGS[ring_index].0[0]).cast::<u8>(),
            0,
            core::mem::size_of::<Trb>() * BULK_RING_LEN,
        );
        ptr::write_bytes(
            ptr::addr_of_mut!(BULK_OUT_RINGS[ring_index].0[0]).cast::<u8>(),
            0,
            core::mem::size_of::<Trb>() * BULK_RING_LEN,
        );
        self.bulk_in_enqueue[ring_index] = 0;
        self.bulk_in_cycle[ring_index] = true;
        self.bulk_out_enqueue[ring_index] = 0;
        self.bulk_out_cycle[ring_index] = true;
    }

    unsafe fn recover_interrupt_endpoint(
        &mut self,
        slot_id: u8,
        dci: u8,
        ring_index: usize,
    ) -> Result<(), &'static str> {
        self.recover_count = self.recover_count.saturating_add(1);
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
        ptr::addr_of_mut!(BULK_IN_RINGS).cast::<u8>(),
        0,
        core::mem::size_of::<[TrbRing<BULK_RING_LEN>; MAX_HID_DEVICES]>(),
    );
    ptr::write_bytes(
        ptr::addr_of_mut!(BULK_OUT_RINGS).cast::<u8>(),
        0,
        core::mem::size_of::<[TrbRing<BULK_RING_LEN>; MAX_HID_DEVICES]>(),
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
    ptr::write_bytes(MSC_CBW.0.as_mut_ptr(), 0, MSC_CBW_LEN);
    ptr::write_bytes(MSC_CSW.0.as_mut_ptr(), 0, MSC_CSW_LEN);
    ptr::write_bytes(MSC_BUFFER.0.as_mut_ptr(), 0, MSC_SECTOR_BUFFER_LEN);
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
    let dword2 = slot_context_tt_fields(port);
    ctx_write_raw(slot, 0, dword0);
    ctx_write_raw(slot, 1, dword1);
    ctx_write_raw(slot, 2, dword2);
}

fn slot_context_tt_fields(port: PortInfo) -> u32 {
    if (port.speed == 1 || port.speed == 2) && port.parent_hub_slot_id != 0 {
        // TODO: multi-tier TT hub: this uses the immediate parent hub.
        (port.parent_hub_slot_id as u32) | ((port.parent_hub_port_number as u32) << 8)
    } else {
        0
    }
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

fn parse_mass_storage_interface(config: &[u8]) -> Result<MassStorageInterface, &'static str> {
    if config.len() < 9 {
        return Err("configuration descriptor too short");
    }

    let mut configuration_value = 0u8;
    let mut in_dci = 0u8;
    let mut out_dci = 0u8;
    let mut in_addr = 0u8;
    let mut out_addr = 0u8;
    let mut in_mps = 0u16;
    let mut out_mps = 0u16;
    let mut in_msc = false;
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
                let current_interface = config[offset + 2];
                let class = config[offset + 5];
                let subclass = config[offset + 6];
                let protocol = config[offset + 7];
                in_msc = class == USB_CLASS_MASS_STORAGE
                    && subclass == USB_MSC_SUBCLASS_SCSI
                    && protocol == USB_MSC_PROTOCOL_BOT;
                if in_msc {
                    in_dci = 0;
                    out_dci = 0;
                    in_addr = 0;
                    out_addr = 0;
                    in_mps = 0;
                    out_mps = 0;
                    serial::write_fmt(format_args!(
                        "usb-msc: bulk-only interface {}\r\n",
                        current_interface,
                    ));
                }
            }
            5 if len >= 7 && in_msc => {
                let endpoint_address = config[offset + 2];
                let attributes = config[offset + 3] & 0x03;
                if attributes == 0x02 {
                    let endpoint_number = endpoint_address & 0x0F;
                    let max_packet_size =
                        u16::from_le_bytes([config[offset + 4], config[offset + 5]]) & 0x07FF;
                    if endpoint_address & 0x80 != 0 {
                        in_addr = endpoint_address;
                        in_dci = endpoint_number * 2 + 1;
                        in_mps = max_packet_size;
                    } else {
                        out_addr = endpoint_address;
                        out_dci = endpoint_number * 2;
                        out_mps = max_packet_size;
                    }
                    if in_dci != 0 && out_dci != 0 {
                        return Ok(MassStorageInterface {
                            configuration_value,
                            bulk_in_dci: in_dci,
                            bulk_out_dci: out_dci,
                            bulk_in_endpoint_address: in_addr,
                            bulk_out_endpoint_address: out_addr,
                            bulk_in_max_packet_size: u16::max(in_mps, 64),
                            bulk_out_max_packet_size: u16::max(out_mps, 64),
                        });
                    }
                }
            }
            _ => {}
        }

        offset += len;
    }

    Err("MSC bulk endpoints not found")
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

fn valid_ep0_mps(speed: u8, mps: u16) -> bool {
    match speed {
        1 => mps == 8 || mps == 16 || mps == 32 || mps == 64,
        2 => mps == 8,
        3 => mps == 64,
        4 => mps == 512,
        _ => false,
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

fn read_le32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn write_le32(bytes: &mut [u8], offset: usize, value: u32) {
    let raw = value.to_le_bytes();
    bytes[offset..offset + 4].copy_from_slice(&raw);
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
