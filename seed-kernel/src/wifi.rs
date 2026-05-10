use core::str;

use spin::Mutex;

use crate::pci::{self, PciAddress};
use crate::serial;

const MARVELL_VENDOR_ID: u16 = 0x11ab;
const AVASTAR_88W8897_DEVICE_ID: u16 = 0x2b38;
const MICROSOFT_SUBSYSTEM_VENDOR_ID: u16 = 0x045e;

pub const SSID_CAPACITY: usize = 32;
pub const PASSPHRASE_CAPACITY: usize = 63;

static STATE: Mutex<WifiRuntime> = Mutex::new(WifiRuntime::new());

pub type WifiSsid = WifiText<SSID_CAPACITY>;

#[derive(Clone, Copy)]
pub struct WifiText<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

impl<const N: usize> WifiText<N> {
    pub const fn empty() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn set_bytes(&mut self, bytes: &[u8]) {
        self.bytes.fill(0);
        self.bytes[..bytes.len()].copy_from_slice(bytes);
        self.len = bytes.len();
    }

    fn clear(&mut self) {
        self.bytes[..self.len].fill(0);
        self.len = 0;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WifiConfigError {
    EmptySsid,
    SsidTooLong,
    PassphraseTooShort,
    PassphraseTooLong,
    InvalidByte,
}

struct WifiRuntime {
    snapshot: WifiSnapshot,
    passphrase: WifiText<PASSPHRASE_CAPACITY>,
}

impl WifiRuntime {
    const fn new() -> Self {
        Self {
            snapshot: WifiSnapshot::new(),
            passphrase: WifiText::empty(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WifiState {
    NotProbed,
    Missing,
    Detected,
}

#[derive(Clone, Copy)]
pub struct WifiSnapshot {
    pub state: WifiState,
    pub address: Option<PciAddress>,
    pub vendor_id: u16,
    pub device_id: u16,
    pub subsystem_vendor_id: u16,
    pub subsystem_id: u16,
    pub bar0_base: Option<u64>,
    pub ssid: WifiSsid,
    pub passphrase_set: bool,
}

impl WifiSnapshot {
    const fn new() -> Self {
        Self {
            state: WifiState::NotProbed,
            address: None,
            vendor_id: 0,
            device_id: 0,
            subsystem_vendor_id: 0,
            subsystem_id: 0,
            bar0_base: None,
            ssid: WifiText::empty(),
            passphrase_set: false,
        }
    }
}

pub fn probe() -> WifiSnapshot {
    let mut guard = STATE.lock();
    if guard.snapshot.state != WifiState::NotProbed {
        return guard.snapshot;
    }

    let Some(address) = pci::find_device(MARVELL_VENDOR_ID, AVASTAR_88W8897_DEVICE_ID) else {
        serial::write_line("wifi: Marvell Avastar 88W8897 target not detected");
        guard.snapshot.state = WifiState::Missing;
        return guard.snapshot;
    };

    let subsystem = address.read_u32(0x2c);
    let subsystem_vendor_id = (subsystem & 0xffff) as u16;
    let subsystem_id = ((subsystem >> 16) & 0xffff) as u16;
    let bar0_base = read_bar0_base(address);
    let ssid = guard.snapshot.ssid;
    let passphrase_set = !guard.passphrase.is_empty();

    guard.snapshot = WifiSnapshot {
        state: WifiState::Detected,
        address: Some(address),
        vendor_id: MARVELL_VENDOR_ID,
        device_id: AVASTAR_88W8897_DEVICE_ID,
        subsystem_vendor_id,
        subsystem_id,
        bar0_base,
        ssid,
        passphrase_set,
    };

    serial::write_fmt(format_args!(
        "wifi: Marvell Avastar 88W8897 target detected at {} pci {:04x}:{:04x} subsys {:04x}:{:04x}",
        address,
        MARVELL_VENDOR_ID,
        AVASTAR_88W8897_DEVICE_ID,
        subsystem_vendor_id,
        subsystem_id
    ));
    if subsystem_vendor_id == MICROSOFT_SUBSYSTEM_VENDOR_ID {
        serial::write_line(" microsoft-surface");
    } else {
        serial::write_line("");
    }

    guard.snapshot
}

pub fn snapshot() -> WifiSnapshot {
    STATE.lock().snapshot
}

pub fn set_ssid(bytes: &[u8]) -> Result<(), WifiConfigError> {
    if bytes.is_empty() {
        return Err(WifiConfigError::EmptySsid);
    }
    if bytes.len() > SSID_CAPACITY {
        return Err(WifiConfigError::SsidTooLong);
    }
    if !is_printable_ascii(bytes) {
        return Err(WifiConfigError::InvalidByte);
    }

    let mut text = WifiSsid::empty();
    text.set_bytes(bytes);

    let mut guard = STATE.lock();
    guard.snapshot.ssid = text;
    Ok(())
}

pub fn set_passphrase(bytes: &[u8]) -> Result<(), WifiConfigError> {
    if bytes.len() < 8 {
        return Err(WifiConfigError::PassphraseTooShort);
    }
    if bytes.len() > PASSPHRASE_CAPACITY {
        return Err(WifiConfigError::PassphraseTooLong);
    }
    if !is_printable_ascii(bytes) {
        return Err(WifiConfigError::InvalidByte);
    }

    let mut text = WifiText::<PASSPHRASE_CAPACITY>::empty();
    text.set_bytes(bytes);

    let mut guard = STATE.lock();
    guard.passphrase = text;
    guard.snapshot.passphrase_set = true;
    Ok(())
}

pub fn clear_config() {
    let mut guard = STATE.lock();
    guard.snapshot.ssid.clear();
    guard.passphrase.clear();
    guard.snapshot.passphrase_set = false;
}

fn read_bar0_base(address: PciAddress) -> Option<u64> {
    let low = address.read_u32(0x10);
    if low == 0 || low == u32::MAX {
        return None;
    }

    if low & 0x1 != 0 {
        return Some((low & !0x3) as u64);
    }

    let bar_type = (low >> 1) & 0x3;
    if bar_type == 0x2 {
        let high = address.read_u32(0x14);
        Some(((high as u64) << 32) | ((low & !0xf) as u64))
    } else {
        Some((low & !0xf) as u64)
    }
}

fn is_printable_ascii(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| (0x20..=0x7e).contains(byte))
}
