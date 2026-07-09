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
pub const ASSOCIATION_RATES_CAPACITY: usize = 14;
pub const ASSOCIATION_SECURITY_IE_CAPACITY: usize = 257;
pub const WIFI_SCAN_UNAVAILABLE_REASON: &str = "wifi firmware not loaded";
pub const WIFI_SCAN_MAILBOX_UNAVAILABLE_REASON: &str = "firmware ready; scan command not started";
pub const WIFI_SCAN_COMMAND_PENDING_REASON: &str = "scan command pending";
pub const WIFI_SCAN_RX_RING_UNAVAILABLE_REASON: &str =
    "scan event observed; live parser not implemented";
pub const WIFI_SCAN_COMMAND_FAILED_REASON: &str = "scan command failed";
pub const WIFI_SCAN_LIVE_RESULTS_REASON: &str = "live scan response parsed";

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
    selected_bss: Option<ScannedNetwork>,
}

impl WifiRuntime {
    const fn new() -> Self {
        Self {
            snapshot: WifiSnapshot::new(),
            passphrase: WifiText::empty(),
            scan_results: [ScannedNetwork::empty(); SCAN_RESULT_CAPACITY],
            scan_count: 0,
            selected_bss: None,
        }
    }

    fn clear_scan_results(&mut self) {
        self.scan_results = [ScannedNetwork::empty(); SCAN_RESULT_CAPACITY];
        self.scan_count = 0;
        self.selected_bss = None;
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
    pub remember_passphrase_for_boot: bool,
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
            remember_passphrase_for_boot: true,
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
    pub bssid: [u8; 6],
    pub beacon_period: u16,
    pub capability_info: u16,
    pub rates: [u8; ASSOCIATION_RATES_CAPACITY],
    pub rates_len: usize,
    pub security_ie: [u8; ASSOCIATION_SECURITY_IE_CAPACITY],
    pub security_ie_len: usize,
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
            bssid: [0; 6],
            beacon_period: 0,
            capability_info: 0,
            rates: [0; ASSOCIATION_RATES_CAPACITY],
            rates_len: 0,
            security_ie: [0; ASSOCIATION_SECURITY_IE_CAPACITY],
            security_ie_len: 0,
        }
    }

    pub fn association_ready(&self) -> bool {
        self.source == ScanSource::LiveRadio
            && self.bssid != [0; 6]
            && !self.ssid.is_empty()
            && self.channel != 0
            && self.beacon_period != 0
            && self.rates_len != 0
    }

    pub fn rates(&self) -> &[u8] {
        &self.rates[..self.rates_len]
    }

    pub fn security_ie(&self) -> &[u8] {
        &self.security_ie[..self.security_ie_len]
    }

    pub fn supports_wpa2_psk_ccmp(&self) -> bool {
        let ie = self.security_ie();
        if self.security != Dot11Security::Wpa2
            || ie.len() < 2
            || ie[0] != 48
            || ie.len() != ie[1] as usize + 2
        {
            return false;
        }
        rsn_has_psk_ccmp(&ie[2..])
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
    let remember_passphrase_for_boot = guard.snapshot.remember_passphrase_for_boot;

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
        remember_passphrase_for_boot,
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
    let mut network = ScannedNetwork {
        ssid,
        hidden_ssid: parsed.ssid.is_hidden(),
        channel: parsed.channel.unwrap_or(0),
        security: parsed.security,
        rssi,
        source,
        ..ScannedNetwork::empty()
    };
    retain_association_evidence(bytes, &mut network);

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

pub fn select_scan_result(index: usize) -> Result<ScannedNetwork, WifiConfigError> {
    let mut guard = STATE.lock();
    if index >= guard.scan_count {
        return Err(WifiConfigError::EmptySsid);
    }
    let network = guard.scan_results[index];
    if !network.association_ready() {
        return Err(WifiConfigError::EmptySsid);
    }
    guard.snapshot.ssid = network.ssid;
    guard.selected_bss = Some(network);
    Ok(network)
}

pub fn association_target() -> Option<ScannedNetwork> {
    STATE.lock().selected_bss
}

pub fn run_scan_selftest() {
    let reason = {
        let mut guard = STATE.lock();
        guard.clear_scan_results();
        guard.snapshot.scan_available = false;
        guard.snapshot.scan_unavailable_reason
    };
    let _ = ingest_scan_frame(SELFTEST_OPEN_BEACON, ScanSource::SelfTestDemo, None);
    let _ = ingest_scan_frame(SELFTEST_WPA2_BEACON, ScanSource::SelfTestDemo, None);
    let _ = ingest_scan_frame(
        SELFTEST_HIDDEN_WPA3_PROBE_RESPONSE,
        ScanSource::SelfTestDemo,
        None,
    );
    serial::write_fmt(format_args!(
        "wifi scan self-test: parsed embedded demo beacons; live scan unavailable ({})\r\n",
        reason
    ));
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
    guard.selected_bss = None;
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

pub fn set_remember_passphrase_for_boot(remember: bool) {
    STATE.lock().snapshot.remember_passphrase_for_boot = remember;
}

pub fn copy_passphrase(out: &mut [u8]) -> Option<usize> {
    let guard = STATE.lock();
    if guard.passphrase.is_empty() || out.len() < guard.passphrase.as_bytes().len() {
        return None;
    }
    let len = guard.passphrase.as_bytes().len();
    out[..len].copy_from_slice(guard.passphrase.as_bytes());
    Some(len)
}

pub fn clear_config() {
    let mut guard = STATE.lock();
    guard.snapshot.ssid.clear();
    guard.passphrase.clear();
    guard.selected_bss = None;
    guard.snapshot.passphrase_set = false;
    guard.snapshot.remember_passphrase_for_boot = true;
}

pub fn note_firmware_ready_scan_unavailable() {
    let mut guard = STATE.lock();
    guard.snapshot.scan_available = false;
    guard.snapshot.scan_unavailable_reason = WIFI_SCAN_MAILBOX_UNAVAILABLE_REASON;
}

pub fn note_scan_command_started() {
    let mut guard = STATE.lock();
    guard.clear_scan_results();
    guard.snapshot.scan_available = false;
    guard.snapshot.scan_unavailable_reason = WIFI_SCAN_COMMAND_PENDING_REASON;
}

pub fn note_scan_results_available() {
    let mut guard = STATE.lock();
    guard.snapshot.scan_available = true;
    guard.snapshot.scan_unavailable_reason = WIFI_SCAN_LIVE_RESULTS_REASON;
}

pub fn note_scan_event_observed_rx_ring_unavailable() {
    let mut guard = STATE.lock();
    guard.clear_scan_results();
    guard.snapshot.scan_available = false;
    guard.snapshot.scan_unavailable_reason = WIFI_SCAN_RX_RING_UNAVAILABLE_REASON;
}

pub fn note_scan_command_failed() {
    let mut guard = STATE.lock();
    guard.clear_scan_results();
    guard.snapshot.scan_available = false;
    guard.snapshot.scan_unavailable_reason = WIFI_SCAN_COMMAND_FAILED_REASON;
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
    if left.bssid != [0; 6] && right.bssid != [0; 6] {
        left.bssid == right.bssid
    } else {
        left.hidden_ssid == right.hidden_ssid && left.ssid.as_bytes() == right.ssid.as_bytes()
    }
}

fn retain_association_evidence(frame: &[u8], network: &mut ScannedNetwork) {
    const BEACON_FIXED_OFFSET: usize = 24;
    const BEACON_IES_OFFSET: usize = 36;
    if frame.len() < BEACON_IES_OFFSET {
        return;
    }

    network.bssid.copy_from_slice(&frame[16..22]);
    network.beacon_period = u16::from_le_bytes([
        frame[BEACON_FIXED_OFFSET + 8],
        frame[BEACON_FIXED_OFFSET + 9],
    ]);
    network.capability_info = u16::from_le_bytes([
        frame[BEACON_FIXED_OFFSET + 10],
        frame[BEACON_FIXED_OFFSET + 11],
    ]);

    let mut offset = BEACON_IES_OFFSET;
    while offset + 2 <= frame.len() {
        let element_id = frame[offset];
        let element_len = frame[offset + 1] as usize;
        let end = offset.saturating_add(2).saturating_add(element_len);
        if end > frame.len() {
            break;
        }
        let element = &frame[offset + 2..end];
        match element_id {
            1 | 50 => append_rates(element, network),
            48 if network.security_ie_len == 0 => retain_security_ie(&frame[offset..end], network),
            221 if network.security_ie_len == 0 && is_wpa_vendor_ie(element) => {
                retain_security_ie(&frame[offset..end], network)
            }
            _ => {}
        }
        offset = end;
    }
}

fn append_rates(rates: &[u8], network: &mut ScannedNetwork) {
    let remaining = ASSOCIATION_RATES_CAPACITY.saturating_sub(network.rates_len);
    let take = usize::min(remaining, rates.len());
    let end = network.rates_len + take;
    network.rates[network.rates_len..end].copy_from_slice(&rates[..take]);
    network.rates_len = end;
}

fn retain_security_ie(ie: &[u8], network: &mut ScannedNetwork) {
    if ie.len() > ASSOCIATION_SECURITY_IE_CAPACITY {
        return;
    }
    network.security_ie[..ie.len()].copy_from_slice(ie);
    network.security_ie_len = ie.len();
}

fn is_wpa_vendor_ie(element: &[u8]) -> bool {
    element.len() >= 4 && element[..4] == [0x00, 0x50, 0xf2, 0x01]
}

fn rsn_has_psk_ccmp(rsn: &[u8]) -> bool {
    const CCMP: [u8; 4] = [0x00, 0x0f, 0xac, 0x04];
    const PSK: [u8; 4] = [0x00, 0x0f, 0xac, 0x02];
    if rsn.len() < 8 || u16::from_le_bytes([rsn[0], rsn[1]]) != 1 || rsn[2..6] != CCMP {
        return false;
    }
    let pairwise_count = u16::from_le_bytes([rsn[6], rsn[7]]) as usize;
    let pairwise_start = 8usize;
    let Some(pairwise_end) = pairwise_start.checked_add(pairwise_count.saturating_mul(4)) else {
        return false;
    };
    if pairwise_count == 0 || pairwise_end + 2 > rsn.len() {
        return false;
    }
    let pairwise_ccmp = rsn[pairwise_start..pairwise_end]
        .chunks_exact(4)
        .any(|suite| suite == CCMP);
    let akm_count = u16::from_le_bytes([rsn[pairwise_end], rsn[pairwise_end + 1]]) as usize;
    let akm_start = pairwise_end + 2;
    let Some(akm_end) = akm_start.checked_add(akm_count.saturating_mul(4)) else {
        return false;
    };
    pairwise_ccmp
        && akm_count != 0
        && akm_end <= rsn.len()
        && rsn[akm_start..akm_end]
            .chunks_exact(4)
            .any(|suite| suite == PSK)
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
