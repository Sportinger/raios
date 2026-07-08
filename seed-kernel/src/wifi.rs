use core::str;

use raios_core::dot11_scan::{parse_scan_frame, Dot11ScanError, Dot11Security};
use spin::Mutex;

use crate::pci::{self, PciAddress};
use crate::serial;

const MARVELL_VENDOR_ID: u16 = 0x11ab;
const AVASTAR_88W8897_DEVICE_ID: u16 = 0x2b38;
const MICROSOFT_SUBSYSTEM_VENDOR_ID: u16 = 0x045e;

pub const SSID_CAPACITY: usize = 32;
pub const PASSPHRASE_CAPACITY: usize = 63;
pub const SCAN_RESULT_CAPACITY: usize = 16;
pub const WIFI_SCAN_UNAVAILABLE_REASON: &str = "wifi firmware not loaded";
pub const WIFI_SCAN_MAILBOX_UNAVAILABLE_REASON: &str =
    "firmware ready; command mailbox not implemented";

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

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    fn set_bytes(&mut self, bytes: &[u8]) {
        self.bytes.fill(0);
        self.bytes[..bytes.len()].copy_from_slice(bytes);
        self.len = bytes.len();
    }

    fn set_scan_bytes(&mut self, bytes: &[u8]) {
        self.clear();
        let take = usize::min(bytes.len(), N);
        let mut idx = 0usize;
        while idx < take {
            let byte = bytes[idx];
            self.bytes[idx] = if (0x20..=0x7e).contains(&byte) {
                byte
            } else {
                b'?'
            };
            idx += 1;
        }
        self.len = take;
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
    scan_results: [ScannedNetwork; SCAN_RESULT_CAPACITY],
    scan_count: usize,
}

impl WifiRuntime {
    const fn new() -> Self {
        Self {
            snapshot: WifiSnapshot::new(),
            passphrase: WifiText::empty(),
            scan_results: [ScannedNetwork::empty(); SCAN_RESULT_CAPACITY],
            scan_count: 0,
        }
    }

    fn clear_scan_results(&mut self) {
        self.scan_results = [ScannedNetwork::empty(); SCAN_RESULT_CAPACITY];
        self.scan_count = 0;
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
    pub scan_available: bool,
    pub scan_unavailable_reason: &'static str,
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
            scan_available: false,
            scan_unavailable_reason: WIFI_SCAN_UNAVAILABLE_REASON,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScanSource {
    SelfTestDemo,
    LiveRadio,
}

impl ScanSource {
    pub const fn tag(self) -> &'static str {
        match self {
            ScanSource::SelfTestDemo => "[SELF-TEST]",
            ScanSource::LiveRadio => "[LIVE]",
        }
    }
}

#[derive(Clone, Copy)]
pub struct ScannedNetwork {
    pub ssid: WifiSsid,
    pub hidden_ssid: bool,
    pub channel: u8,
    pub security: Dot11Security,
    pub rssi: Option<i8>,
    pub source: ScanSource,
}

impl ScannedNetwork {
    const fn empty() -> Self {
        Self {
            ssid: WifiSsid::empty(),
            hidden_ssid: false,
            channel: 0,
            security: Dot11Security::Unknown,
            rssi: None,
            source: ScanSource::SelfTestDemo,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ScanResultsSnapshot {
    pub networks: [ScannedNetwork; SCAN_RESULT_CAPACITY],
    pub count: usize,
    pub scan_available: bool,
    pub unavailable_reason: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WifiScanIngestError {
    Parse(Dot11ScanError),
    Full,
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

    let snapshot = WifiSnapshot {
        state: WifiState::Detected,
        address: Some(address),
        vendor_id: MARVELL_VENDOR_ID,
        device_id: AVASTAR_88W8897_DEVICE_ID,
        subsystem_vendor_id,
        subsystem_id,
        bar0_base,
        ssid,
        passphrase_set,
        scan_available: false,
        scan_unavailable_reason: WIFI_SCAN_UNAVAILABLE_REASON,
    };
    guard.snapshot = snapshot;

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

    drop(guard);
    run_scan_selftest();
    snapshot
}

pub fn snapshot() -> WifiSnapshot {
    STATE.lock().snapshot
}

pub fn scan_results() -> ScanResultsSnapshot {
    let guard = STATE.lock();
    ScanResultsSnapshot {
        networks: guard.scan_results,
        count: guard.scan_count,
        scan_available: guard.snapshot.scan_available,
        unavailable_reason: guard.snapshot.scan_unavailable_reason,
    }
}

pub fn ingest_scan_frame(
    bytes: &[u8],
    source: ScanSource,
    rssi: Option<i8>,
) -> Result<(), WifiScanIngestError> {
    let parsed = parse_scan_frame(bytes).map_err(WifiScanIngestError::Parse)?;
    let mut ssid = WifiSsid::empty();
    ssid.set_scan_bytes(parsed.ssid.as_bytes());
    let network = ScannedNetwork {
        ssid,
        hidden_ssid: parsed.ssid.is_hidden(),
        channel: parsed.channel.unwrap_or(0),
        security: parsed.security,
        rssi,
        source,
    };

    let mut guard = STATE.lock();
    let mut idx = 0usize;
    while idx < guard.scan_count {
        if same_scan_key(&guard.scan_results[idx], &network) {
            guard.scan_results[idx] = network;
            return Ok(());
        }
        idx += 1;
    }
    if guard.scan_count >= SCAN_RESULT_CAPACITY {
        return Err(WifiScanIngestError::Full);
    }
    let insert = guard.scan_count;
    guard.scan_results[insert] = network;
    guard.scan_count = insert + 1;
    Ok(())
}

pub fn run_scan_selftest() {
    {
        let mut guard = STATE.lock();
        guard.clear_scan_results();
    }
    let _ = ingest_scan_frame(SELFTEST_OPEN_BEACON, ScanSource::SelfTestDemo, None);
    let _ = ingest_scan_frame(SELFTEST_WPA2_BEACON, ScanSource::SelfTestDemo, None);
    let _ = ingest_scan_frame(
        SELFTEST_HIDDEN_WPA3_PROBE_RESPONSE,
        ScanSource::SelfTestDemo,
        None,
    );
    serial::write_line(
        "wifi scan self-test: parsed embedded demo beacons; live scan unavailable (wifi firmware not loaded)",
    );
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

pub fn note_firmware_ready_scan_unavailable() {
    let mut guard = STATE.lock();
    guard.snapshot.scan_available = false;
    guard.snapshot.scan_unavailable_reason = WIFI_SCAN_MAILBOX_UNAVAILABLE_REASON;
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

fn same_scan_key(left: &ScannedNetwork, right: &ScannedNetwork) -> bool {
    left.hidden_ssid == right.hidden_ssid && left.ssid.as_bytes() == right.ssid.as_bytes()
}

pub fn scan_security_label(security: Dot11Security) -> &'static str {
    match security {
        Dot11Security::Open => "OPEN",
        Dot11Security::Wep => "WEP",
        Dot11Security::Wpa => "WPA",
        Dot11Security::Wpa2 => "WPA2",
        Dot11Security::Wpa3 => "WPA3",
        Dot11Security::Unknown => "UNKNOWN",
    }
}

const SELFTEST_OPEN_BEACON: &[u8] = &[
    0x80, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x10, 0x22, 0x33, 0x44, 0x55, 0x66,
    0x10, 0x22, 0x33, 0x44, 0x55, 0x66, 0x10, 0x00, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
    0x64, 0x00, 0x01, 0x00, 0x00, 0x07, b'R', b'a', b'i', b'O', b'p', b'e', b'n', 0x01, 0x08, 0x82,
    0x84, 0x8b, 0x96, 0x24, 0x30, 0x48, 0x6c, 0x03, 0x01, 0x06,
];

const SELFTEST_WPA2_BEACON: &[u8] = &[
    0x80, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x20, 0x22, 0x33, 0x44, 0x55, 0x66,
    0x20, 0x22, 0x33, 0x44, 0x55, 0x66, 0x20, 0x00, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01,
    0x64, 0x00, 0x11, 0x00, 0x00, 0x07, b'R', b'a', b'i', b'W', b'P', b'A', b'2', 0x03, 0x01, 0x0b,
    0x30, 0x14, 0x01, 0x00, 0x00, 0x0f, 0xac, 0x04, 0x01, 0x00, 0x00, 0x0f, 0xac, 0x04, 0x01, 0x00,
    0x00, 0x0f, 0xac, 0x02, 0x00, 0x00,
];

const SELFTEST_HIDDEN_WPA3_PROBE_RESPONSE: &[u8] = &[
    0x50, 0x00, 0x00, 0x00, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x30, 0x22, 0x33, 0x44, 0x55, 0x66,
    0x30, 0x22, 0x33, 0x44, 0x55, 0x66, 0x30, 0x00, 0x18, 0x17, 0x16, 0x15, 0x14, 0x13, 0x12, 0x11,
    0x64, 0x00, 0x11, 0x00, 0x00, 0x00, 0x03, 0x01, 0x01, 0x30, 0x14, 0x01, 0x00, 0x00, 0x0f, 0xac,
    0x04, 0x01, 0x00, 0x00, 0x0f, 0xac, 0x04, 0x01, 0x00, 0x00, 0x0f, 0xac, 0x08, 0x00, 0x00,
];
