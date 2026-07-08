use core::fmt::{self, Write};
use core::str;

use crate::framebuffer::FramebufferInfo;
use crate::{entropy, input, net, usb, wifi};

pub const STATUS_DETAIL_LEN: usize = 192;

#[derive(Clone, Copy)]
pub struct RuntimeStatus {
    pub framebuffer: Option<FramebufferInfo>,
    pub net_probe_complete: bool,
    pub input_probe_complete: bool,
}

impl RuntimeStatus {
    pub const fn new() -> Self {
        Self {
            framebuffer: None,
            net_probe_complete: false,
            input_probe_complete: false,
        }
    }
}

pub struct SystemSnapshot {
    pub framebuffer: StatusLine,
    pub entropy: StatusLine,
    pub usb_xhci: StatusLine,
    pub usb_hotplug: StatusLine,
    pub wifi: StatusLine,
    pub network: StatusLine,
    pub input: StatusLine,
}

impl SystemSnapshot {
    pub fn collect(framebuffer: Option<FramebufferInfo>, runtime: RuntimeStatus) -> Self {
        let framebuffer = match framebuffer {
            Some(info) => Some(info),
            None => runtime.framebuffer,
        };
        Self {
            framebuffer: framebuffer_line(framebuffer),
            entropy: entropy_line(),
            usb_xhci: usb_xhci_line(),
            usb_hotplug: usb_hotplug_line(),
            wifi: wifi_line(),
            network: network_line(runtime),
            input: input_line(runtime),
        }
    }

    pub fn states(&self) -> SnapshotStates {
        SnapshotStates {
            framebuffer: self.framebuffer.state,
            entropy: self.entropy.state,
            usb_xhci: self.usb_xhci.state,
            usb_hotplug: self.usb_hotplug.state,
            wifi: self.wifi.state,
            network: self.network.state,
            input: self.input.state,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SnapshotStates {
    pub framebuffer: RowState,
    pub entropy: RowState,
    pub usb_xhci: RowState,
    pub usb_hotplug: RowState,
    pub wifi: RowState,
    pub network: RowState,
    pub input: RowState,
}

pub struct StatusLine {
    pub label: &'static str,
    pub state: RowState,
    pub detail: TextBuf<STATUS_DETAIL_LEN>,
}

impl StatusLine {
    fn new(label: &'static str, state: RowState, detail: TextBuf<STATUS_DETAIL_LEN>) -> Self {
        Self {
            label,
            state,
            detail,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RowState {
    Ready,
    Waiting,
    Configured,
    Detected,
    Degraded,
    Missing,
}

impl RowState {
    pub fn as_str(self) -> &'static str {
        match self {
            RowState::Ready => "READY",
            RowState::Waiting => "WAITING",
            RowState::Configured => "CONFIGURED",
            RowState::Detected => "DETECTED",
            RowState::Degraded => "DEGRADED",
            RowState::Missing => "MISSING",
        }
    }

    pub fn as_protocol(self) -> &'static str {
        match self {
            RowState::Ready => "ready",
            RowState::Waiting => "waiting",
            RowState::Configured => "configured",
            RowState::Detected => "detected",
            RowState::Degraded => "degraded",
            RowState::Missing => "missing",
        }
    }
}

fn framebuffer_line(info: Option<FramebufferInfo>) -> StatusLine {
    match info {
        Some(info) => StatusLine::new(
            "FRAMEBUFFER",
            RowState::Ready,
            detail(format_args!(
                "{}x{} PITCH {}",
                info.width, info.height, info.pitch
            )),
        ),
        None => StatusLine::new(
            "FRAMEBUFFER",
            RowState::Missing,
            detail(format_args!("LIMINE FRAMEBUFFER UNAVAILABLE")),
        ),
    }
}

fn entropy_line() -> StatusLine {
    let stats = entropy::stats();
    let state = if stats.ready {
        RowState::Ready
    } else {
        RowState::Waiting
    };
    let mut line = detail(format_args!(
        "FILL {}/{} TOTAL {} SRC ",
        stats.pool_fill,
        entropy::POOL_CAPACITY,
        stats.total_collected
    ));
    append_entropy_sources(&mut line, stats);
    StatusLine::new("ENTROPY", state, line)
}

fn usb_xhci_line() -> StatusLine {
    let snapshot = usb::snapshot();
    match snapshot.state {
        usb::UsbStatus::NotProbed => StatusLine::new(
            "USB-XHCI",
            RowState::Waiting,
            detail(format_args!("PROBE PENDING")),
        ),
        usb::UsbStatus::Missing => StatusLine::new(
            "USB-XHCI",
            RowState::Missing,
            detail(format_args!("CONTROLLER ABSENT")),
        ),
        usb::UsbStatus::Error => StatusLine::new(
            "USB-XHCI",
            RowState::Degraded,
            detail(format_args!(
                "{}",
                snapshot.last_error.unwrap_or("PROBE ERROR")
            )),
        ),
        usb::UsbStatus::Ready => {
            let address = match snapshot.address {
                Some(address) => detail(format_args!("{}", address)),
                None => detail(format_args!("UNKNOWN")),
            };
            let keyboard = usb_keyboard_status(snapshot.keyboard_status);
            let mouse = usb_mouse_status(snapshot.mouse_status);
            let hid_detail = if snapshot.keyboard_status != usb::UsbKeyboardStatus::Ready {
                snapshot.keyboard_detail
            } else if snapshot.mouse_status != usb::UsbMouseStatus::Ready {
                snapshot.mouse_detail
            } else {
                None
            };
            let hid_detail = snapshot.hub_last_error.or(hid_detail).unwrap_or("OK");
            let last_enum = usb_last_enum_detail(snapshot);
            if snapshot.hub_count > 0 {
                if snapshot.last_completion_code != 0 {
                    StatusLine::new(
                        "USB-XHCI",
                        RowState::Ready,
                        detail(format_args!(
                            "{} HCI {:04X} ROOT {}/{} PWR {} HUB {} {}P {}C {}R {}D KBD {} MOUSE {} EV {} ERR {} TCC {} CMD {} CC {} {}{} RCV{} ICC{}",
                            address.as_str(),
                            snapshot.hci_version,
                            snapshot.connected_ports,
                            snapshot.max_ports,
                            snapshot.powered_ports,
                            snapshot.hub_count,
                            snapshot.hub_ports,
                            snapshot.hub_connected_ports,
                            snapshot.hub_reset_ports,
                            snapshot.hub_configured_devices,
                            keyboard,
                            mouse,
                            snapshot.input_report_count,
                            snapshot.input_error_count,
                            snapshot.last_transfer_completion_code,
                            snapshot.last_command_type,
                            snapshot.last_completion_code,
                            hid_detail,
                            last_enum.as_str(),
                            snapshot.recover_count,
                            snapshot.last_int_cc
                        )),
                    )
                } else {
                    StatusLine::new(
                        "USB-XHCI",
                        RowState::Ready,
                        detail(format_args!(
                            "{} HCI {:04X} ROOT {}/{} PWR {} HUB {} {}P {}C {}R {}D KBD {} MOUSE {} EV {} ERR {} TCC {} {}{} RCV{} ICC{}",
                            address.as_str(),
                            snapshot.hci_version,
                            snapshot.connected_ports,
                            snapshot.max_ports,
                            snapshot.powered_ports,
                            snapshot.hub_count,
                            snapshot.hub_ports,
                            snapshot.hub_connected_ports,
                            snapshot.hub_reset_ports,
                            snapshot.hub_configured_devices,
                            keyboard,
                            mouse,
                            snapshot.input_report_count,
                            snapshot.input_error_count,
                            snapshot.last_transfer_completion_code,
                            hid_detail,
                            last_enum.as_str(),
                            snapshot.recover_count,
                            snapshot.last_int_cc
                        )),
                    )
                }
            } else {
                StatusLine::new(
                    "USB-XHCI",
                    RowState::Ready,
                    detail(format_args!(
                        "{} HCI {:04X} PORTS {} PWR {} CONNECTED {} KBD {} MOUSE {} EV {} ERR {} TCC {} HID {}{} RCV{} ICC{}",
                        address.as_str(),
                        snapshot.hci_version,
                        snapshot.max_ports,
                        snapshot.powered_ports,
                        snapshot.connected_ports,
                        keyboard,
                        mouse,
                        snapshot.input_report_count,
                        snapshot.input_error_count,
                        snapshot.last_transfer_completion_code,
                        hid_detail,
                        last_enum.as_str(),
                        snapshot.recover_count,
                        snapshot.last_int_cc
                    )),
                )
            }
        }
    }
}

fn usb_last_enum_detail(snapshot: usb::UsbSnapshot) -> TextBuf<96> {
    let mut line = TextBuf::new();
    if snapshot.keyboard_status == usb::UsbKeyboardStatus::Ready
        && snapshot.mouse_status == usb::UsbMouseStatus::Ready
    {
        return line;
    }
    if let Some(error) = snapshot.last_enum_error {
        let _ = line.write_fmt(format_args!(
            " LAST {:04X}:{:04X} CLS {:02X} {}",
            snapshot.last_device_vid, snapshot.last_device_pid, snapshot.last_device_class, error
        ));
    }
    line
}

fn usb_keyboard_status(status: usb::UsbKeyboardStatus) -> &'static str {
    match status {
        usb::UsbKeyboardStatus::NotProbed => "PENDING",
        usb::UsbKeyboardStatus::Ready => "READY",
        usb::UsbKeyboardStatus::NotFound => "NONE",
        usb::UsbKeyboardStatus::Error => "ERROR",
    }
}

fn usb_mouse_status(status: usb::UsbMouseStatus) -> &'static str {
    match status {
        usb::UsbMouseStatus::NotProbed => "PENDING",
        usb::UsbMouseStatus::Ready => "READY",
        usb::UsbMouseStatus::NotFound => "NONE",
        usb::UsbMouseStatus::Error => "ERROR",
    }
}

fn usb_hotplug_line() -> StatusLine {
    let snapshot = usb::snapshot();
    if snapshot.enum_stage != "none" {
        let state = if snapshot.enum_err == "ok" {
            RowState::Detected
        } else {
            RowState::Degraded
        };
        return StatusLine::new("USB HOTPLUG", state, usb_enum_trace_detail(snapshot));
    }

    if !snapshot.last_hotplug_present {
        return StatusLine::new(
            "USB HOTPLUG",
            RowState::Waiting,
            detail(format_args!("none")),
        );
    }

    let location = if snapshot.last_hotplug_is_hub {
        detail(format_args!(
            "HUB{} P{}",
            snapshot.last_hotplug_hub_slot, snapshot.last_hotplug_port
        ))
    } else {
        detail(format_args!("ROOT P{}", snapshot.last_hotplug_port))
    };

    if snapshot.last_hotplug_connected {
        if snapshot.last_hotplug_skipped {
            StatusLine::new(
                "USB HOTPLUG",
                RowState::Detected,
                detail(format_args!(
                    "#{} {} CONNECT -> skipped: {}",
                    snapshot.last_hotplug_seq,
                    location.as_str(),
                    snapshot.last_hotplug_detail
                )),
            )
        } else {
            StatusLine::new(
                "USB HOTPLUG",
                RowState::Detected,
                detail(format_args!(
                    "#{} {} CONNECT -> {}",
                    snapshot.last_hotplug_seq,
                    location.as_str(),
                    snapshot.last_hotplug_detail
                )),
            )
        }
    } else {
        StatusLine::new(
            "USB HOTPLUG",
            RowState::Detected,
            detail(format_args!(
                "#{} {} DISCONNECT",
                snapshot.last_hotplug_seq,
                location.as_str()
            )),
        )
    }
}

fn usb_enum_trace_detail(snapshot: usb::UsbSnapshot) -> TextBuf<STATUS_DETAIL_LEN> {
    if snapshot.enum_location.is_hub {
        detail(format_args!(
            "ENUM HUB{} P{} SPD{} SLOT{} STAGE {} CMDCC{} XFERCC{} {:04X}:{:04X} CLS{:02X} MPS{} ERR {}",
            snapshot.enum_location.hub_slot,
            snapshot.enum_location.port,
            snapshot.enum_speed,
            snapshot.enum_slot_id,
            snapshot.enum_stage,
            snapshot.enum_cmd_cc,
            snapshot.enum_xfer_cc,
            snapshot.enum_vid,
            snapshot.enum_pid,
            snapshot.enum_dev_class,
            snapshot.enum_ep0_mps,
            snapshot.enum_err
        ))
    } else {
        detail(format_args!(
            "ENUM ROOT P{} SPD{} SLOT{} STAGE {} CMDCC{} XFERCC{} {:04X}:{:04X} CLS{:02X} MPS{} ERR {}",
            snapshot.enum_location.port,
            snapshot.enum_speed,
            snapshot.enum_slot_id,
            snapshot.enum_stage,
            snapshot.enum_cmd_cc,
            snapshot.enum_xfer_cc,
            snapshot.enum_vid,
            snapshot.enum_pid,
            snapshot.enum_dev_class,
            snapshot.enum_ep0_mps,
            snapshot.enum_err
        ))
    }
}

fn wifi_line() -> StatusLine {
    let snapshot = wifi::snapshot();
    match snapshot.state {
        wifi::WifiState::NotProbed => StatusLine::new(
            "WIFI",
            RowState::Waiting,
            detail(format_args!("PROBE PENDING")),
        ),
        wifi::WifiState::Missing => StatusLine::new(
            "WIFI",
            RowState::Missing,
            detail(format_args!(
                "SURFACE PRO 4 88W8897 TARGET ABSENT SSID {} KEY {}",
                wifi_ssid_status(&snapshot.ssid),
                secret_status(snapshot.passphrase_set)
            )),
        ),
        wifi::WifiState::Detected => {
            let address = match snapshot.address {
                Some(address) => detail(format_args!("{}", address)),
                None => detail(format_args!("UNKNOWN")),
            };
            StatusLine::new(
                "WIFI",
                RowState::Detected,
                detail(format_args!(
                    "{} MARVELL 88W8897 SSID {} KEY {} PCI {:04X}:{:04X} SUBSYS {:04X}:{:04X} BAR0 {} FW TODO",
                    address.as_str(),
                    wifi_ssid_status(&snapshot.ssid),
                    secret_status(snapshot.passphrase_set),
                    snapshot.vendor_id,
                    snapshot.device_id,
                    snapshot.subsystem_vendor_id,
                    snapshot.subsystem_id,
                    Bar0(snapshot.bar0_base)
                )),
            )
        }
    }
}

fn wifi_ssid_status(ssid: &wifi::WifiSsid) -> &str {
    if ssid.is_empty() {
        "NONE"
    } else {
        ssid.as_str()
    }
}

fn secret_status(set: bool) -> &'static str {
    if set {
        "SET"
    } else {
        "MISSING"
    }
}

fn network_line(runtime: RuntimeStatus) -> StatusLine {
    if let Some(config) = net::ui_snapshot() {
        if let Some(ip) = config.ip {
            let gateway = match config.gateway {
                Some(gateway) => detail(format_args!("{}", gateway)),
                None => detail(format_args!("NONE")),
            };
            return StatusLine::new(
                "NETWORK",
                RowState::Configured,
                detail(format_args!(
                    "IP {}/{} GW {}",
                    ip.address(),
                    ip.prefix_len(),
                    gateway.as_str()
                )),
            );
        }

        return StatusLine::new(
            "NETWORK",
            RowState::Waiting,
            if net::dhcp_poll_enabled() {
                detail(format_args!("MAC {} AWAITING DHCP", Mac(config.mac)))
            } else {
                detail(format_args!("MAC {} DHCP DEFERRED", Mac(config.mac)))
            },
        );
    }

    if runtime.net_probe_complete {
        StatusLine::new(
            "NETWORK",
            RowState::Missing,
            detail(format_args!("E1000 DEVICE ABSENT OR UNSUPPORTED")),
        )
    } else if entropy::is_ready() {
        StatusLine::new(
            "NETWORK",
            RowState::Waiting,
            detail(format_args!("PROBE PENDING")),
        )
    } else {
        StatusLine::new(
            "NETWORK",
            RowState::Waiting,
            detail(format_args!("WAITING ENTROPY")),
        )
    }
}

fn input_line(runtime: RuntimeStatus) -> StatusLine {
    if input::device_present() {
        return StatusLine::new(
            "INPUT",
            RowState::Ready,
            detail(format_args!("{}", input::device_detail())),
        );
    }

    if runtime.input_probe_complete {
        StatusLine::new(
            "INPUT",
            RowState::Missing,
            detail(format_args!("DEVICE ABSENT OR UNSUPPORTED")),
        )
    } else if entropy::is_ready() {
        StatusLine::new(
            "INPUT",
            RowState::Waiting,
            detail(format_args!("PROBE PENDING")),
        )
    } else {
        StatusLine::new(
            "INPUT",
            RowState::Waiting,
            detail(format_args!("WAITING ENTROPY")),
        )
    }
}

fn append_entropy_sources(buffer: &mut TextBuf<STATUS_DETAIL_LEN>, stats: entropy::EntropyStats) {
    let mut wrote = false;
    if stats.used_rdrand {
        buffer.push_str("RDRAND");
        wrote = true;
    }
    if !wrote {
        buffer.push_str("NONE");
    }
}

fn detail(args: fmt::Arguments<'_>) -> TextBuf<STATUS_DETAIL_LEN> {
    let mut buffer = TextBuf::new();
    let _ = buffer.write_fmt(args);
    buffer
}

struct Mac([u8; 6]);

impl fmt::Display for Mac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

struct Bar0(Option<u64>);

impl fmt::Display for Bar0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(base) => write!(f, "0x{:X}", base),
            None => f.write_str("UNKNOWN"),
        }
    }
}

pub struct TextBuf<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

impl<const N: usize> TextBuf<N> {
    pub const fn new() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }

    pub fn push_str(&mut self, value: &str) {
        let remaining = N.saturating_sub(self.len);
        let bytes = value.as_bytes();
        let take = usize::min(remaining, bytes.len());
        self.bytes[self.len..self.len + take].copy_from_slice(&bytes[..take]);
        self.len += take;
    }
}

impl<const N: usize> Write for TextBuf<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}
