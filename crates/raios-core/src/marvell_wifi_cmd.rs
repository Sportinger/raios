//! Pure Marvell 88W8897 command-mailbox packet helpers.
//!
//! No MMIO, DMA, allocation, or authority lives here. The kernel-side PCIe
//! shell owns transport and timeouts.

pub const GET_HW_SPEC_CMD: u16 = 0x0003;
pub const SCAN_CMD: u16 = 0x0006;
pub const ASSOCIATE_CMD: u16 = 0x0012;
pub const MAC_CONTROL_CMD: u16 = 0x0028;
pub const FUNC_INIT_CMD: u16 = 0x00a9;
pub const PCIE_DESC_DETAILS_CMD: u16 = 0x00fa;
pub const SCAN_EXT_CMD: u16 = 0x0107;
pub const MWIFIEX_TYPE_CMD: u16 = 1;
pub const HOST_CMD_RET_BIT: u16 = 0x8000;
pub const HOST_CMD_RESULT_OK: u16 = 0;
pub const HOST_CMD_SEQUENCE_MASK: u16 = 0x00ff;
pub const S_DS_GEN: usize = 8;
pub const INTF_HEADER_LEN: usize = 4;
pub const GET_HW_SPEC_BODY_LEN: usize = 63;
pub const GET_HW_SPEC_GEN_SIZE: usize = S_DS_GEN + GET_HW_SPEC_BODY_LEN;
pub const GET_HW_SPEC_CMD_TOTAL_LEN: usize = INTF_HEADER_LEN + GET_HW_SPEC_GEN_SIZE;
pub const HW_SPEC_MIN_RESPONSE_LEN: usize = 50;
pub const HOST_CMD_MIN_RESPONSE_LEN: usize = INTF_HEADER_LEN + S_DS_GEN;
pub const FUNC_INIT_CMD_TOTAL_LEN: usize = HOST_CMD_MIN_RESPONSE_LEN;
pub const IEEE80211_MAX_SSID_LEN: u8 = 32;
pub const MWIFIEX_BSS_MODE_INFRA: u8 = 1;
pub const MWIFIEX_DISABLE_CHAN_FILT: u8 = 0x02;
pub const MWIFIEX_ACTIVE_SCAN_CHAN_TIME: u16 = 40;
pub const SCAN_EXT_DEFAULT_NUM_PROBES: u16 = 2;
pub const TLV_TYPE_CHANLIST: u16 = 0x0101;
pub const TLV_TYPE_NUMPROBES: u16 = 0x0102;
pub const TLV_TYPE_WILDCARDSSID: u16 = 0x0112;
pub const TLV_TYPE_BSS_MODE: u16 = 0x01ce;
pub const TLV_HEADER_LEN: usize = 4;
pub const SCAN_EXT_RESERVED_LEN: usize = 4;
pub const SCAN_EXT_WILDCARD_SSID_TLV_LEN: usize = TLV_HEADER_LEN + 1;
pub const SCAN_EXT_BSS_MODE_TLV_LEN: usize = TLV_HEADER_LEN + 1;
pub const SCAN_EXT_NUM_PROBES_TLV_LEN: usize = TLV_HEADER_LEN + 2;
pub const SCAN_EXT_CHANNEL_PARAM_LEN: usize = 7;
pub const SCAN_EXT_24GHZ_CHANNEL_COUNT: usize = 11;
pub const SCAN_EXT_24GHZ_CHANLIST_TLV_LEN: usize =
    TLV_HEADER_LEN + SCAN_EXT_24GHZ_CHANNEL_COUNT * SCAN_EXT_CHANNEL_PARAM_LEN;
pub const SCAN_EXT_24GHZ_TLV_LEN: usize = SCAN_EXT_WILDCARD_SSID_TLV_LEN
    + SCAN_EXT_BSS_MODE_TLV_LEN
    + SCAN_EXT_NUM_PROBES_TLV_LEN
    + SCAN_EXT_24GHZ_CHANLIST_TLV_LEN;
pub const SCAN_EXT_24GHZ_GEN_SIZE: usize =
    S_DS_GEN + SCAN_EXT_RESERVED_LEN + SCAN_EXT_24GHZ_TLV_LEN;
pub const SCAN_EXT_24GHZ_CMD_TOTAL_LEN: usize = INTF_HEADER_LEN + SCAN_EXT_24GHZ_GEN_SIZE;
pub const SCAN_FIXED_LEN: usize = 7;
pub const SCAN_24GHZ_TLV_LEN: usize = SCAN_EXT_NUM_PROBES_TLV_LEN + SCAN_EXT_24GHZ_CHANLIST_TLV_LEN;
pub const SCAN_24GHZ_GEN_SIZE: usize = S_DS_GEN + SCAN_FIXED_LEN + SCAN_24GHZ_TLV_LEN;
pub const SCAN_24GHZ_CMD_TOTAL_LEN: usize = INTF_HEADER_LEN + SCAN_24GHZ_GEN_SIZE;
pub const SCAN_RESPONSE_FIXED_LEN: usize = INTF_HEADER_LEN + S_DS_GEN + 3;
pub const SCAN_BSS_FIXED_LEN: usize = 19;
pub const MWIFIEX_MAX_TXRX_BD: u32 = 0x20;
pub const MWIFIEX_MAX_EVT_BD: u32 = 0x08;
pub const PCIE_DESC_DETAILS_BODY_LEN: usize = 44;
pub const PCIE_DESC_DETAILS_GEN_SIZE: usize = S_DS_GEN + PCIE_DESC_DETAILS_BODY_LEN;
pub const PCIE_DESC_DETAILS_CMD_TOTAL_LEN: usize = INTF_HEADER_LEN + PCIE_DESC_DETAILS_GEN_SIZE;
pub const MAC_CONTROL_ACTION_RX_TX_ETHERNET_II: u32 = 0x13;
pub const MAC_CONTROL_BODY_LEN: usize = 4;
pub const MAC_CONTROL_GEN_SIZE: usize = S_DS_GEN + MAC_CONTROL_BODY_LEN;
pub const MAC_CONTROL_CMD_TOTAL_LEN: usize = INTF_HEADER_LEN + MAC_CONTROL_GEN_SIZE;
pub const ASSOCIATE_FIXED_BODY_LEN: usize = 13;
pub const ASSOCIATE_MIN_RESPONSE_LEN: usize = INTF_HEADER_LEN + S_DS_GEN + 6;
pub const MWIFIEX_DEFAULT_LISTEN_INTERVAL: u16 = 10;
pub const MWIFIEX_SUPPORTED_RATES: usize = 14;
pub const MWIFIEX_CAPINFO_MASK: u16 = 0x25ff;
pub const WLAN_EID_SSID: u16 = 0;
pub const WLAN_EID_SUPP_RATES: u16 = 1;
pub const WLAN_EID_DS_PARAMS: u16 = 3;
pub const WLAN_EID_CF_PARAMS: u16 = 4;
pub const WLAN_EID_RSN: u8 = 48;
pub const WLAN_EID_VENDOR_SPECIFIC: u8 = 221;
pub const TLV_TYPE_AUTH_TYPE: u16 = 0x011f;
pub const IEEE80211_MAX_AID: u16 = 2007;

const WPA_IE_OUI_TYPE: [u8; 4] = [0x00, 0x50, 0xf2, 0x01];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HwSpecCmdError {
    TooShort,
    BadLength,
    BadCommand { got: u16 },
    BadSequence { got: u16 },
    FwResult { code: u16 },
    InvalidSequenceContext { got: u16 },
    OutputBufferTooSmall,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HwSpec {
    pub mac: [u8; 6],
    pub fw_release: u32,
    pub fw_cap_info: u32,
    pub key_api_version: Option<(u8, u8)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyScanBss<'a> {
    pub bssid: [u8; 6],
    pub rssi_dbm: i16,
    pub fixed_beacon_params: &'a [u8],
    pub ies: &'a [u8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PcieDescriptorRings {
    pub tx_phys: u64,
    pub rx_phys: u64,
    pub event_phys: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssociationBss<'a> {
    pub bssid: [u8; 6],
    pub ssid: &'a [u8],
    pub channel: u8,
    pub beacon_period: u16,
    pub capability_info: u16,
    pub rates: &'a [u8],
    pub rsn_or_wpa_ie: Option<&'a [u8]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AssociationResponse {
    pub capability_info: u16,
    pub status_code: u16,
    pub raw_association_id: u16,
    pub association_id: Option<u16>,
}

/// Monotonic HostCmd sequence allocation for the single STA BSS. The firmware
/// reserves bits 8..=11 for the BSS number and bits 12..=15 for the BSS type,
/// so an allocation window must never cross out of the low eight bits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SingleBssHostCmdSequenceAllocator {
    next: u8,
}

impl SingleBssHostCmdSequenceAllocator {
    pub const fn new() -> Self {
        Self { next: 1 }
    }

    pub fn reserve_window(&mut self, count: u8) -> u8 {
        assert!(count != 0, "HostCmd sequence window cannot be empty");
        let mut base = self.next;
        let last = u16::from(base) + u16::from(count) - 1;
        if last > HOST_CMD_SEQUENCE_MASK {
            base = 1;
        }
        let next = u16::from(base) + u16::from(count);
        self.next = if next > HOST_CMD_SEQUENCE_MASK {
            1
        } else {
            next as u8
        };
        base
    }
}

impl Default for SingleBssHostCmdSequenceAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MarvellResponseHeader {
    pub interface_len: u16,
    pub interface_type: u16,
    pub command: u16,
    pub host_command_size: u16,
    pub sequence: u16,
    pub result: u16,
}

impl MarvellResponseHeader {
    pub const fn is_empty(self) -> bool {
        self.interface_len == 0
            && self.interface_type == 0
            && self.command == 0
            && self.host_command_size == 0
            && self.sequence == 0
            && self.result == 0
    }
}

pub const fn host_cmd_done_low_after_clear(status: u32, cmd_done_mask: u32) -> bool {
    status != u32::MAX && status & cmd_done_mask == 0
}

pub const fn host_cmd_done_is_current(
    low_baseline_observed: bool,
    status: u32,
    cmd_done_mask: u32,
) -> bool {
    low_baseline_observed && status != u32::MAX && status & cmd_done_mask != 0
}

pub const fn marvell_8897_pci_identity_valid(vendor_device: u32) -> bool {
    vendor_device == 0x2b38_11ab
}

pub const fn pci_memory_bus_master_enabled(command: u16) -> bool {
    command != u16::MAX && command & 0x0006 == 0x0006
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PciCommandEnablePlan {
    AlreadyEnabled,
    Write(u16),
    Unavailable,
}

pub const fn plan_pci_memory_bus_master_enable(
    vendor_device: u32,
    command: u16,
) -> PciCommandEnablePlan {
    if !marvell_8897_pci_identity_valid(vendor_device) || command == u16::MAX {
        PciCommandEnablePlan::Unavailable
    } else if pci_memory_bus_master_enabled(command) {
        PciCommandEnablePlan::AlreadyEnabled
    } else {
        PciCommandEnablePlan::Write(command | 0x0006)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionMmioLiveness {
    Ready,
    FirmwareNotReady,
    MmioUnavailable,
}

pub const fn connection_mmio_liveness(
    firmware_status: u32,
    host_interrupt_status: u32,
) -> ConnectionMmioLiveness {
    if host_interrupt_status == u32::MAX {
        ConnectionMmioLiveness::MmioUnavailable
    } else if firmware_status != crate::marvell_wifi_fw::FIRMWARE_READY_PCIE {
        ConnectionMmioLiveness::FirmwareNotReady
    } else {
        ConnectionMmioLiveness::Ready
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarvellCmdError {
    TooShort,
    BadLength,
    BadCommand { got: u16 },
    BadSequence { got: u16 },
    FwResult { code: u16 },
    InvalidSequenceContext { got: u16 },
    OutputBufferTooSmall,
    InvalidDescriptorAddress,
    InvalidBssid,
    InvalidSsidLength,
    InvalidChannel,
    InvalidBeaconPeriod,
    InvalidRates,
    InvalidSecurityIe,
    InvalidAssociationId { raw: u16 },
}

/// Copies only the fixed, non-payload HostCmd response header. Callers must
/// not retain or log bytes beyond these six protocol words.
pub fn redacted_response_header(buf: &[u8]) -> Option<MarvellResponseHeader> {
    if buf.len() < HOST_CMD_MIN_RESPONSE_LEN {
        return None;
    }
    Some(MarvellResponseHeader {
        interface_len: le16(buf, 0),
        interface_type: le16(buf, 2),
        command: le16(buf, 4),
        host_command_size: le16(buf, 6),
        sequence: le16(buf, 8),
        result: le16(buf, 10),
    })
}

pub const fn has_single_bss_host_cmd_context(sequence: u16) -> bool {
    sequence & !HOST_CMD_SEQUENCE_MASK == 0
}

fn require_single_bss_host_cmd_context(sequence: u16) -> Result<(), MarvellCmdError> {
    if has_single_bss_host_cmd_context(sequence) {
        Ok(())
    } else {
        Err(MarvellCmdError::InvalidSequenceContext { got: sequence })
    }
}

pub fn build_pcie_desc_details(
    seq: u16,
    rings: PcieDescriptorRings,
    out: &mut [u8],
) -> Result<usize, MarvellCmdError> {
    require_single_bss_host_cmd_context(seq)?;
    if rings.tx_phys == 0 || rings.rx_phys == 0 || rings.event_phys == 0 {
        return Err(MarvellCmdError::InvalidDescriptorAddress);
    }
    if out.len() < PCIE_DESC_DETAILS_CMD_TOTAL_LEN {
        return Err(MarvellCmdError::OutputBufferTooSmall);
    }

    out[..PCIE_DESC_DETAILS_CMD_TOTAL_LEN].fill(0);
    put_le16(out, 0, PCIE_DESC_DETAILS_CMD_TOTAL_LEN as u16);
    put_le16(out, 2, MWIFIEX_TYPE_CMD);
    put_le16(out, 4, PCIE_DESC_DETAILS_CMD);
    put_le16(out, 6, PCIE_DESC_DETAILS_GEN_SIZE as u16);
    put_le16(out, 8, seq);
    put_phys_addr(out, 12, rings.tx_phys);
    put_le32(out, 20, MWIFIEX_MAX_TXRX_BD);
    put_phys_addr(out, 24, rings.rx_phys);
    put_le32(out, 32, MWIFIEX_MAX_TXRX_BD);
    put_phys_addr(out, 36, rings.event_phys);
    put_le32(out, 44, MWIFIEX_MAX_EVT_BD);
    Ok(PCIE_DESC_DETAILS_CMD_TOTAL_LEN)
}

pub fn build_func_init(seq: u16, out: &mut [u8]) -> Result<usize, MarvellCmdError> {
    require_single_bss_host_cmd_context(seq)?;
    if out.len() < FUNC_INIT_CMD_TOTAL_LEN {
        return Err(MarvellCmdError::OutputBufferTooSmall);
    }

    out[..FUNC_INIT_CMD_TOTAL_LEN].fill(0);
    put_le16(out, 0, FUNC_INIT_CMD_TOTAL_LEN as u16);
    put_le16(out, 2, MWIFIEX_TYPE_CMD);
    put_le16(out, 4, FUNC_INIT_CMD);
    put_le16(out, 6, S_DS_GEN as u16);
    put_le16(out, 8, seq);
    Ok(FUNC_INIT_CMD_TOTAL_LEN)
}

pub fn build_mac_control(seq: u16, out: &mut [u8]) -> Result<usize, MarvellCmdError> {
    require_single_bss_host_cmd_context(seq)?;
    if out.len() < MAC_CONTROL_CMD_TOTAL_LEN {
        return Err(MarvellCmdError::OutputBufferTooSmall);
    }

    out[..MAC_CONTROL_CMD_TOTAL_LEN].fill(0);
    put_le16(out, 0, MAC_CONTROL_CMD_TOTAL_LEN as u16);
    put_le16(out, 2, MWIFIEX_TYPE_CMD);
    put_le16(out, 4, MAC_CONTROL_CMD);
    put_le16(out, 6, MAC_CONTROL_GEN_SIZE as u16);
    put_le16(out, 8, seq);
    put_le32(out, 12, MAC_CONTROL_ACTION_RX_TX_ETHERNET_II);
    Ok(MAC_CONTROL_CMD_TOTAL_LEN)
}

pub fn build_associate_24ghz(
    seq: u16,
    bss: AssociationBss<'_>,
    out: &mut [u8],
) -> Result<usize, MarvellCmdError> {
    require_single_bss_host_cmd_context(seq)?;
    if bss.bssid.iter().all(|byte| *byte == 0) || bss.bssid[0] & 1 != 0 {
        return Err(MarvellCmdError::InvalidBssid);
    }
    if bss.ssid.is_empty() || bss.ssid.len() > IEEE80211_MAX_SSID_LEN as usize {
        return Err(MarvellCmdError::InvalidSsidLength);
    }
    if !(1..=14).contains(&bss.channel) {
        return Err(MarvellCmdError::InvalidChannel);
    }
    if bss.beacon_period == 0 {
        return Err(MarvellCmdError::InvalidBeaconPeriod);
    }
    if bss.rates.is_empty()
        || bss.rates.len() > MWIFIEX_SUPPORTED_RATES
        || bss.rates.iter().any(|rate| rate & 0x7f == 0)
    {
        return Err(MarvellCmdError::InvalidRates);
    }

    let security = match bss.rsn_or_wpa_ie {
        None => None,
        Some(ie) => {
            if ie.len() < 2 || ie.len() != ie[1] as usize + 2 {
                return Err(MarvellCmdError::InvalidSecurityIe);
            }
            match ie[0] {
                WLAN_EID_RSN if ie[1] >= 2 => {}
                WLAN_EID_VENDOR_SPECIFIC
                    if ie[1] >= WPA_IE_OUI_TYPE.len() as u8
                        && ie[2..].starts_with(&WPA_IE_OUI_TYPE) => {}
                _ => return Err(MarvellCmdError::InvalidSecurityIe),
            }
            Some((ie[0] as u16, &ie[2..]))
        }
    };

    let security_len = security.map_or(0, |(_, payload)| TLV_HEADER_LEN + payload.len());
    let body_len = ASSOCIATE_FIXED_BODY_LEN
        + TLV_HEADER_LEN
        + bss.ssid.len()
        + (TLV_HEADER_LEN + 1)
        + (TLV_HEADER_LEN + 6)
        + TLV_HEADER_LEN
        + bss.rates.len()
        + (TLV_HEADER_LEN + 2)
        + (TLV_HEADER_LEN + SCAN_EXT_CHANNEL_PARAM_LEN)
        + security_len;
    let gen_size = S_DS_GEN + body_len;
    let total_len = INTF_HEADER_LEN + gen_size;
    if out.len() < total_len {
        return Err(MarvellCmdError::OutputBufferTooSmall);
    }

    out[..total_len].fill(0);
    put_le16(out, 0, total_len as u16);
    put_le16(out, 2, MWIFIEX_TYPE_CMD);
    put_le16(out, 4, ASSOCIATE_CMD);
    put_le16(out, 6, gen_size as u16);
    put_le16(out, 8, seq);

    out[12..18].copy_from_slice(&bss.bssid);
    put_le16(out, 18, bss.capability_info & MWIFIEX_CAPINFO_MASK);
    put_le16(out, 20, MWIFIEX_DEFAULT_LISTEN_INTERVAL);
    put_le16(out, 22, bss.beacon_period);

    let mut offset = INTF_HEADER_LEN + S_DS_GEN + ASSOCIATE_FIXED_BODY_LEN;
    write_tlv_header(out, offset, WLAN_EID_SSID, bss.ssid.len() as u16);
    out[offset + TLV_HEADER_LEN..offset + TLV_HEADER_LEN + bss.ssid.len()]
        .copy_from_slice(bss.ssid);
    offset += TLV_HEADER_LEN + bss.ssid.len();

    write_tlv_header(out, offset, WLAN_EID_DS_PARAMS, 1);
    out[offset + TLV_HEADER_LEN] = bss.channel;
    offset += TLV_HEADER_LEN + 1;

    write_tlv_header(out, offset, WLAN_EID_CF_PARAMS, 6);
    offset += TLV_HEADER_LEN + 6;

    write_tlv_header(out, offset, WLAN_EID_SUPP_RATES, bss.rates.len() as u16);
    out[offset + TLV_HEADER_LEN..offset + TLV_HEADER_LEN + bss.rates.len()]
        .copy_from_slice(bss.rates);
    offset += TLV_HEADER_LEN + bss.rates.len();

    write_tlv_header(out, offset, TLV_TYPE_AUTH_TYPE, 2);
    offset += TLV_HEADER_LEN + 2;

    write_tlv_header(
        out,
        offset,
        TLV_TYPE_CHANLIST,
        SCAN_EXT_CHANNEL_PARAM_LEN as u16,
    );
    out[offset + TLV_HEADER_LEN + 1] = bss.channel;
    offset += TLV_HEADER_LEN + SCAN_EXT_CHANNEL_PARAM_LEN;

    if let Some((ty, payload)) = security {
        write_tlv_header(out, offset, ty, payload.len() as u16);
        out[offset + TLV_HEADER_LEN..offset + TLV_HEADER_LEN + payload.len()]
            .copy_from_slice(payload);
        offset += TLV_HEADER_LEN + payload.len();
    }

    debug_assert_eq!(offset, total_len);
    Ok(total_len)
}

pub fn parse_associate_response(
    expected_seq: u16,
    buf: &[u8],
) -> Result<AssociationResponse, MarvellCmdError> {
    if buf.len() < 2 {
        return Err(MarvellCmdError::TooShort);
    }
    let response_len = le16(buf, 0) as usize;
    if response_len < ASSOCIATE_MIN_RESPONSE_LEN || buf.len() < response_len {
        return Err(MarvellCmdError::TooShort);
    }
    let command_size = le16(buf, 6) as usize;
    if command_size < S_DS_GEN + 6 || response_len != INTF_HEADER_LEN + command_size {
        return Err(MarvellCmdError::BadLength);
    }
    let command = le16(buf, 4);
    if command != ASSOCIATE_CMD | HOST_CMD_RET_BIT {
        return Err(MarvellCmdError::BadCommand { got: command });
    }
    let sequence = le16(buf, 8);
    if sequence != expected_seq {
        return Err(MarvellCmdError::BadSequence { got: sequence });
    }
    let result = le16(buf, 10);
    if result != HOST_CMD_RESULT_OK {
        return Err(MarvellCmdError::FwResult { code: result });
    }

    let capability_info = le16(buf, 12);
    let status_code = le16(buf, 14);
    let raw_association_id = le16(buf, 16);
    let association_id = if status_code == 0 {
        let aid = raw_association_id & 0x3fff;
        if raw_association_id & 0xc000 != 0xc000 || !(1..=IEEE80211_MAX_AID).contains(&aid) {
            return Err(MarvellCmdError::InvalidAssociationId {
                raw: raw_association_id,
            });
        }
        Some(aid)
    } else {
        None
    };

    Ok(AssociationResponse {
        capability_info,
        status_code,
        raw_association_id,
        association_id,
    })
}

pub fn parse_pcie_desc_details_response(
    expected_seq: u16,
    buf: &[u8],
) -> Result<(), MarvellCmdError> {
    parse_marvell_command_status(buf, PCIE_DESC_DETAILS_CMD, expected_seq)
}

pub fn parse_func_init_response(expected_seq: u16, buf: &[u8]) -> Result<(), MarvellCmdError> {
    parse_marvell_command_status(buf, FUNC_INIT_CMD, expected_seq)
}

pub fn parse_mac_control_response(expected_seq: u16, buf: &[u8]) -> Result<(), MarvellCmdError> {
    parse_marvell_command_status(buf, MAC_CONTROL_CMD, expected_seq)
}

fn parse_marvell_command_status(
    buf: &[u8],
    expected_cmd: u16,
    expected_seq: u16,
) -> Result<(), MarvellCmdError> {
    require_single_bss_host_cmd_context(expected_seq)?;
    if buf.len() < HOST_CMD_MIN_RESPONSE_LEN {
        return Err(MarvellCmdError::TooShort);
    }
    let response_len = le16(buf, 0) as usize;
    let command_size = le16(buf, 6) as usize;
    if response_len < HOST_CMD_MIN_RESPONSE_LEN
        || response_len > buf.len()
        || command_size < S_DS_GEN
        || response_len != INTF_HEADER_LEN + command_size
    {
        return Err(MarvellCmdError::BadLength);
    }
    let command = le16(buf, 4);
    if command != expected_cmd | HOST_CMD_RET_BIT {
        return Err(MarvellCmdError::BadCommand { got: command });
    }
    let sequence = le16(buf, 8);
    if sequence != expected_seq {
        return Err(MarvellCmdError::BadSequence { got: sequence });
    }
    let result = le16(buf, 10);
    if result != HOST_CMD_RESULT_OK {
        return Err(MarvellCmdError::FwResult { code: result });
    }
    Ok(())
}

pub fn build_get_hw_spec(seq: u16, out: &mut [u8]) -> Result<usize, HwSpecCmdError> {
    if !has_single_bss_host_cmd_context(seq) {
        return Err(HwSpecCmdError::InvalidSequenceContext { got: seq });
    }
    if out.len() < GET_HW_SPEC_CMD_TOTAL_LEN {
        return Err(HwSpecCmdError::OutputBufferTooSmall);
    }

    out[..GET_HW_SPEC_CMD_TOTAL_LEN].fill(0);
    put_le16(out, 0, GET_HW_SPEC_CMD_TOTAL_LEN as u16);
    put_le16(out, 2, MWIFIEX_TYPE_CMD);
    put_le16(out, 4, GET_HW_SPEC_CMD);
    put_le16(out, 6, GET_HW_SPEC_GEN_SIZE as u16);
    put_le16(out, 8, seq);
    put_le16(out, 10, 0);
    Ok(GET_HW_SPEC_CMD_TOTAL_LEN)
}

pub fn build_scan_ext_24ghz_wildcard(seq: u16, out: &mut [u8]) -> Result<usize, HwSpecCmdError> {
    if out.len() < SCAN_EXT_24GHZ_CMD_TOTAL_LEN {
        return Err(HwSpecCmdError::OutputBufferTooSmall);
    }

    out[..SCAN_EXT_24GHZ_CMD_TOTAL_LEN].fill(0);
    put_le16(out, 0, SCAN_EXT_24GHZ_CMD_TOTAL_LEN as u16);
    put_le16(out, 2, MWIFIEX_TYPE_CMD);
    put_le16(out, 4, SCAN_EXT_CMD);
    put_le16(out, 6, SCAN_EXT_24GHZ_GEN_SIZE as u16);
    put_le16(out, 8, seq);
    put_le16(out, 10, 0);
    put_le32(out, 12, 0);

    let mut offset = INTF_HEADER_LEN + S_DS_GEN + SCAN_EXT_RESERVED_LEN;
    write_tlv_header(out, offset, TLV_TYPE_WILDCARDSSID, 1);
    out[offset + TLV_HEADER_LEN] = IEEE80211_MAX_SSID_LEN;
    offset += SCAN_EXT_WILDCARD_SSID_TLV_LEN;

    write_tlv_header(out, offset, TLV_TYPE_BSS_MODE, 1);
    out[offset + TLV_HEADER_LEN] = MWIFIEX_BSS_MODE_INFRA;
    offset += SCAN_EXT_BSS_MODE_TLV_LEN;

    write_tlv_header(out, offset, TLV_TYPE_NUMPROBES, 2);
    put_le16(out, offset + TLV_HEADER_LEN, SCAN_EXT_DEFAULT_NUM_PROBES);
    offset += SCAN_EXT_NUM_PROBES_TLV_LEN;

    write_tlv_header(
        out,
        offset,
        TLV_TYPE_CHANLIST,
        (SCAN_EXT_24GHZ_CHANNEL_COUNT * SCAN_EXT_CHANNEL_PARAM_LEN) as u16,
    );
    let mut chan_offset = offset + TLV_HEADER_LEN;
    let mut channel = 1u8;
    while channel <= SCAN_EXT_24GHZ_CHANNEL_COUNT as u8 {
        out[chan_offset] = 0;
        out[chan_offset + 1] = channel;
        out[chan_offset + 2] = MWIFIEX_DISABLE_CHAN_FILT;
        put_le16(out, chan_offset + 3, MWIFIEX_ACTIVE_SCAN_CHAN_TIME);
        put_le16(out, chan_offset + 5, MWIFIEX_ACTIVE_SCAN_CHAN_TIME);
        chan_offset += SCAN_EXT_CHANNEL_PARAM_LEN;
        channel += 1;
    }

    Ok(SCAN_EXT_24GHZ_CMD_TOTAL_LEN)
}

pub fn build_scan_24ghz(seq: u16, out: &mut [u8]) -> Result<usize, HwSpecCmdError> {
    if out.len() < SCAN_24GHZ_CMD_TOTAL_LEN {
        return Err(HwSpecCmdError::OutputBufferTooSmall);
    }

    out[..SCAN_24GHZ_CMD_TOTAL_LEN].fill(0);
    put_le16(out, 0, SCAN_24GHZ_CMD_TOTAL_LEN as u16);
    put_le16(out, 2, MWIFIEX_TYPE_CMD);
    put_le16(out, 4, SCAN_CMD);
    put_le16(out, 6, SCAN_24GHZ_GEN_SIZE as u16);
    put_le16(out, 8, seq);
    put_le16(out, 10, 0);
    out[12] = MWIFIEX_BSS_MODE_INFRA;

    let mut offset = INTF_HEADER_LEN + S_DS_GEN + SCAN_FIXED_LEN;
    write_tlv_header(out, offset, TLV_TYPE_NUMPROBES, 2);
    put_le16(out, offset + TLV_HEADER_LEN, SCAN_EXT_DEFAULT_NUM_PROBES);
    offset += SCAN_EXT_NUM_PROBES_TLV_LEN;

    write_tlv_header(
        out,
        offset,
        TLV_TYPE_CHANLIST,
        (SCAN_EXT_24GHZ_CHANNEL_COUNT * SCAN_EXT_CHANNEL_PARAM_LEN) as u16,
    );
    let mut chan_offset = offset + TLV_HEADER_LEN;
    let mut channel = 1u8;
    while channel <= SCAN_EXT_24GHZ_CHANNEL_COUNT as u8 {
        out[chan_offset] = 0;
        out[chan_offset + 1] = channel;
        out[chan_offset + 2] = MWIFIEX_DISABLE_CHAN_FILT;
        put_le16(out, chan_offset + 3, MWIFIEX_ACTIVE_SCAN_CHAN_TIME);
        put_le16(out, chan_offset + 5, MWIFIEX_ACTIVE_SCAN_CHAN_TIME);
        chan_offset += SCAN_EXT_CHANNEL_PARAM_LEN;
        channel += 1;
    }

    Ok(SCAN_24GHZ_CMD_TOTAL_LEN)
}

pub fn parse_hw_spec_response(expected_seq: u16, buf: &[u8]) -> Result<HwSpec, HwSpecCmdError> {
    if !has_single_bss_host_cmd_context(expected_seq) {
        return Err(HwSpecCmdError::InvalidSequenceContext { got: expected_seq });
    }
    if buf.len() < 2 {
        return Err(HwSpecCmdError::TooShort);
    }
    let response_len = le16(buf, 0) as usize;
    if response_len < HW_SPEC_MIN_RESPONSE_LEN
        || buf.len() < HW_SPEC_MIN_RESPONSE_LEN
        || buf.len() < response_len
    {
        return Err(HwSpecCmdError::TooShort);
    }

    let command_size = le16(buf, 6) as usize;
    if command_size < S_DS_GEN || response_len != INTF_HEADER_LEN + command_size {
        return Err(HwSpecCmdError::BadLength);
    }

    let command = le16(buf, 4);
    let expected = GET_HW_SPEC_CMD | HOST_CMD_RET_BIT;
    if command != expected {
        return Err(HwSpecCmdError::BadCommand { got: command });
    }

    let sequence = le16(buf, 8);
    if sequence != expected_seq {
        return Err(HwSpecCmdError::BadSequence { got: sequence });
    }

    let result = le16(buf, 10);
    if result != HOST_CMD_RESULT_OK {
        return Err(HwSpecCmdError::FwResult { code: result });
    }

    let mut mac = [0u8; 6];
    mac.copy_from_slice(&buf[20..26]);
    let mut key_api_version = None;
    let mut offset = GET_HW_SPEC_CMD_TOTAL_LEN;
    while offset + TLV_HEADER_LEN <= response_len {
        let ty = le16(buf, offset);
        let len = le16(buf, offset + 2) as usize;
        let Some(end) = offset.checked_add(TLV_HEADER_LEN + len) else {
            return Err(HwSpecCmdError::TooShort);
        };
        if end > response_len {
            return Err(HwSpecCmdError::TooShort);
        }
        if ty == 0x01c7 && len >= 4 && le16(buf, offset + TLV_HEADER_LEN) == 1 {
            key_api_version = Some((buf[offset + 6], buf[offset + 7]));
        }
        offset = end;
    }

    Ok(HwSpec {
        mac,
        fw_release: le32(buf, 30),
        fw_cap_info: le32(buf, 46),
        key_api_version,
    })
}

pub fn parse_scan_ext_response(buf: &[u8]) -> Result<(), HwSpecCmdError> {
    parse_host_cmd_status(buf, SCAN_EXT_CMD)
}

pub fn visit_scan_response<F>(buf: &[u8], mut visit: F) -> Result<u8, HwSpecCmdError>
where
    F: FnMut(LegacyScanBss<'_>),
{
    parse_host_cmd_status(buf, SCAN_CMD)?;
    let response_len = le16(buf, 0) as usize;
    if response_len < SCAN_RESPONSE_FIXED_LEN {
        return Err(HwSpecCmdError::TooShort);
    }

    let descriptors_len = le16(buf, INTF_HEADER_LEN + S_DS_GEN) as usize;
    let count = buf[INTF_HEADER_LEN + S_DS_GEN + 2];
    let descriptors_start = SCAN_RESPONSE_FIXED_LEN;
    let Some(descriptors_end) = descriptors_start.checked_add(descriptors_len) else {
        return Err(HwSpecCmdError::TooShort);
    };
    if descriptors_end > response_len {
        return Err(HwSpecCmdError::TooShort);
    }

    let mut offset = descriptors_start;
    let mut index = 0u8;
    while index < count {
        let (_, next) = parse_scan_bss(buf, offset, descriptors_end)?;
        offset = next;
        index += 1;
    }
    if offset != descriptors_end {
        return Err(HwSpecCmdError::TooShort);
    }

    offset = descriptors_start;
    index = 0;
    while index < count {
        let (bss, next) = parse_scan_bss(buf, offset, descriptors_end)?;
        visit(bss);
        offset = next;
        index += 1;
    }
    Ok(count)
}

fn parse_scan_bss(
    buf: &[u8],
    offset: usize,
    descriptors_end: usize,
) -> Result<(LegacyScanBss<'_>, usize), HwSpecCmdError> {
    let Some(size_end) = offset.checked_add(2) else {
        return Err(HwSpecCmdError::TooShort);
    };
    if size_end > descriptors_end {
        return Err(HwSpecCmdError::TooShort);
    }
    let beacon_size = le16(buf, offset) as usize;
    if beacon_size < SCAN_BSS_FIXED_LEN {
        return Err(HwSpecCmdError::TooShort);
    }
    let Some(next) = size_end.checked_add(beacon_size) else {
        return Err(HwSpecCmdError::TooShort);
    };
    if next > descriptors_end {
        return Err(HwSpecCmdError::TooShort);
    }

    let body = &buf[size_end..next];
    let mut bssid = [0u8; 6];
    bssid.copy_from_slice(&body[..6]);
    Ok((
        LegacyScanBss {
            bssid,
            rssi_dbm: -(body[6] as i16),
            fixed_beacon_params: &body[7..19],
            ies: &body[19..],
        },
        next,
    ))
}

fn parse_host_cmd_status(buf: &[u8], expected_cmd: u16) -> Result<(), HwSpecCmdError> {
    if buf.len() < 2 {
        return Err(HwSpecCmdError::TooShort);
    }
    let response_len = le16(buf, 0) as usize;
    if response_len < HOST_CMD_MIN_RESPONSE_LEN
        || buf.len() < HOST_CMD_MIN_RESPONSE_LEN
        || buf.len() < response_len
    {
        return Err(HwSpecCmdError::TooShort);
    }

    let command = le16(buf, 4);
    let expected = expected_cmd | HOST_CMD_RET_BIT;
    if command != expected {
        return Err(HwSpecCmdError::BadCommand { got: command });
    }

    let result = le16(buf, 10);
    if result != HOST_CMD_RESULT_OK {
        return Err(HwSpecCmdError::FwResult { code: result });
    }

    Ok(())
}

fn put_le16(out: &mut [u8], offset: usize, value: u16) {
    out[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_le32(out: &mut [u8], offset: usize, value: u32) {
    out[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_phys_addr(out: &mut [u8], offset: usize, value: u64) {
    put_le32(out, offset, value as u32);
    put_le32(out, offset + 4, (value >> 32) as u32);
}

fn write_tlv_header(out: &mut [u8], offset: usize, ty: u16, len: u16) {
    put_le16(out, offset, ty);
    put_le16(out, offset + 2, len);
}

fn le16(buf: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([buf[offset], buf[offset + 1]])
}

fn le32(buf: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        buf[offset],
        buf[offset + 1],
        buf[offset + 2],
        buf[offset + 3],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put_response_le16(buf: &mut [u8], offset: usize, value: u16) {
        buf[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn put_response_le32(buf: &mut [u8], offset: usize, value: u32) {
        buf[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn build_get_hw_spec_pins_linux_framing() {
        let mut out = [0xa5u8; 96];

        let len = build_get_hw_spec(0x34, &mut out).unwrap();

        assert_eq!(len, 75);
        assert_eq!(&out[0..2], &75u16.to_le_bytes());
        assert_eq!(&out[2..4], &1u16.to_le_bytes());
        assert_eq!(&out[4..6], &0x0003u16.to_le_bytes());
        assert_eq!(&out[6..8], &71u16.to_le_bytes());
        assert_eq!(&out[8..10], &0x34u16.to_le_bytes());
        assert_eq!(&out[10..12], &0u16.to_le_bytes());
        assert!(out[12..75].iter().all(|byte| *byte == 0));
        assert!(out[75..].iter().all(|byte| *byte == 0xa5));
    }

    #[test]
    fn build_get_hw_spec_rejects_small_output() {
        let mut out = [0u8; GET_HW_SPEC_CMD_TOTAL_LEN - 1];

        assert_eq!(
            build_get_hw_spec(0, &mut out),
            Err(HwSpecCmdError::OutputBufferTooSmall)
        );
    }

    #[test]
    fn build_scan_ext_24ghz_wildcard_pins_linux_tlv_shape() {
        let mut out = [0xa5u8; SCAN_EXT_24GHZ_CMD_TOTAL_LEN + 8];

        let len = build_scan_ext_24ghz_wildcard(0x2211, &mut out).unwrap();

        assert_eq!(len, SCAN_EXT_24GHZ_CMD_TOTAL_LEN);
        assert_eq!(
            &out[0..2],
            &(SCAN_EXT_24GHZ_CMD_TOTAL_LEN as u16).to_le_bytes()
        );
        assert_eq!(&out[2..4], &MWIFIEX_TYPE_CMD.to_le_bytes());
        assert_eq!(&out[4..6], &SCAN_EXT_CMD.to_le_bytes());
        assert_eq!(&out[6..8], &(SCAN_EXT_24GHZ_GEN_SIZE as u16).to_le_bytes());
        assert_eq!(&out[8..10], &0x2211u16.to_le_bytes());
        assert_eq!(&out[10..16], &[0, 0, 0, 0, 0, 0]);

        let mut offset = INTF_HEADER_LEN + S_DS_GEN + SCAN_EXT_RESERVED_LEN;
        assert_eq!(
            &out[offset..offset + 2],
            &TLV_TYPE_WILDCARDSSID.to_le_bytes()
        );
        assert_eq!(&out[offset + 2..offset + 4], &1u16.to_le_bytes());
        assert_eq!(out[offset + 4], IEEE80211_MAX_SSID_LEN);
        offset += SCAN_EXT_WILDCARD_SSID_TLV_LEN;

        assert_eq!(&out[offset..offset + 2], &TLV_TYPE_BSS_MODE.to_le_bytes());
        assert_eq!(&out[offset + 2..offset + 4], &1u16.to_le_bytes());
        assert_eq!(out[offset + 4], MWIFIEX_BSS_MODE_INFRA);
        offset += SCAN_EXT_BSS_MODE_TLV_LEN;

        assert_eq!(&out[offset..offset + 2], &TLV_TYPE_NUMPROBES.to_le_bytes());
        assert_eq!(&out[offset + 2..offset + 4], &2u16.to_le_bytes());
        assert_eq!(
            &out[offset + 4..offset + 6],
            &SCAN_EXT_DEFAULT_NUM_PROBES.to_le_bytes()
        );
        offset += SCAN_EXT_NUM_PROBES_TLV_LEN;

        assert_eq!(&out[offset..offset + 2], &TLV_TYPE_CHANLIST.to_le_bytes());
        assert_eq!(
            &out[offset + 2..offset + 4],
            &((SCAN_EXT_24GHZ_CHANNEL_COUNT * SCAN_EXT_CHANNEL_PARAM_LEN) as u16).to_le_bytes()
        );
        let channels = &out[offset + TLV_HEADER_LEN..len];
        assert_eq!(
            channels.len(),
            SCAN_EXT_24GHZ_CHANNEL_COUNT * SCAN_EXT_CHANNEL_PARAM_LEN
        );
        assert!(out[len..].iter().all(|byte| *byte == 0xa5));
    }

    #[test]
    fn build_scan_ext_24ghz_wildcard_writes_channels_1_to_11() {
        let mut out = [0u8; SCAN_EXT_24GHZ_CMD_TOTAL_LEN];

        build_scan_ext_24ghz_wildcard(0, &mut out).unwrap();

        let channel_base = INTF_HEADER_LEN
            + S_DS_GEN
            + SCAN_EXT_RESERVED_LEN
            + SCAN_EXT_WILDCARD_SSID_TLV_LEN
            + SCAN_EXT_BSS_MODE_TLV_LEN
            + SCAN_EXT_NUM_PROBES_TLV_LEN
            + TLV_HEADER_LEN;
        let mut index = 0usize;
        while index < SCAN_EXT_24GHZ_CHANNEL_COUNT {
            let offset = channel_base + index * SCAN_EXT_CHANNEL_PARAM_LEN;
            assert_eq!(out[offset], 0);
            assert_eq!(out[offset + 1], (index + 1) as u8);
            assert_eq!(out[offset + 2], MWIFIEX_DISABLE_CHAN_FILT);
            assert_eq!(
                &out[offset + 3..offset + 5],
                &MWIFIEX_ACTIVE_SCAN_CHAN_TIME.to_le_bytes()
            );
            assert_eq!(
                &out[offset + 5..offset + 7],
                &MWIFIEX_ACTIVE_SCAN_CHAN_TIME.to_le_bytes()
            );
            index += 1;
        }
    }

    #[test]
    fn build_scan_ext_24ghz_wildcard_rejects_small_output() {
        let mut out = [0u8; SCAN_EXT_24GHZ_CMD_TOTAL_LEN - 1];

        assert_eq!(
            build_scan_ext_24ghz_wildcard(0, &mut out),
            Err(HwSpecCmdError::OutputBufferTooSmall)
        );
    }

    #[test]
    fn build_scan_24ghz_pins_legacy_response_scan_shape() {
        let mut out = [0xa5u8; SCAN_24GHZ_CMD_TOTAL_LEN + 8];

        let len = build_scan_24ghz(0x3344, &mut out).unwrap();

        assert_eq!(len, SCAN_24GHZ_CMD_TOTAL_LEN);
        assert_eq!(&out[0..2], &(len as u16).to_le_bytes());
        assert_eq!(&out[2..4], &MWIFIEX_TYPE_CMD.to_le_bytes());
        assert_eq!(&out[4..6], &SCAN_CMD.to_le_bytes());
        assert_eq!(&out[6..8], &(SCAN_24GHZ_GEN_SIZE as u16).to_le_bytes());
        assert_eq!(&out[8..10], &0x3344u16.to_le_bytes());
        assert_eq!(out[12], MWIFIEX_BSS_MODE_INFRA);
        assert_eq!(&out[13..19], &[0; 6]);

        let probes = INTF_HEADER_LEN + S_DS_GEN + SCAN_FIXED_LEN;
        assert_eq!(&out[probes..probes + 2], &TLV_TYPE_NUMPROBES.to_le_bytes());
        assert_eq!(&out[probes + 2..probes + 4], &2u16.to_le_bytes());
        assert_eq!(
            &out[probes + 4..probes + 6],
            &SCAN_EXT_DEFAULT_NUM_PROBES.to_le_bytes()
        );
        let channels = probes + SCAN_EXT_NUM_PROBES_TLV_LEN;
        assert_eq!(
            &out[channels..channels + 2],
            &TLV_TYPE_CHANLIST.to_le_bytes()
        );
        assert_eq!(
            &out[channels + 2..channels + 4],
            &((SCAN_EXT_24GHZ_CHANNEL_COUNT * SCAN_EXT_CHANNEL_PARAM_LEN) as u16).to_le_bytes()
        );
        assert!(out[len..].iter().all(|byte| *byte == 0xa5));
    }

    #[test]
    fn build_scan_24ghz_rejects_small_output() {
        let mut out = [0u8; SCAN_24GHZ_CMD_TOTAL_LEN - 1];

        assert_eq!(
            build_scan_24ghz(0, &mut out),
            Err(HwSpecCmdError::OutputBufferTooSmall)
        );
    }

    #[test]
    fn parse_hw_spec_response_extracts_mac_and_firmware_release() {
        let mut response = [0u8; 96];
        put_response_le16(&mut response, 0, 75);
        put_response_le16(&mut response, 4, GET_HW_SPEC_CMD | HOST_CMD_RET_BIT);
        put_response_le16(&mut response, 6, GET_HW_SPEC_GEN_SIZE as u16);
        put_response_le16(&mut response, 10, HOST_CMD_RESULT_OK);
        response[20..26].copy_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
        put_response_le32(&mut response, 30, 0x1568_0019);
        put_response_le32(&mut response, 46, 1 << 21);

        let parsed = parse_hw_spec_response(0, &response).unwrap();

        assert_eq!(parsed.mac, [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
        assert_eq!(parsed.fw_release, 0x1568_0019);
        assert_eq!(parsed.fw_cap_info, 1 << 21);
        assert_eq!(parsed.key_api_version, None);
    }

    #[test]
    fn parse_hw_spec_response_extracts_key_api_revision_tlv() {
        let mut response = [0u8; 96];
        put_response_le16(&mut response, 0, 83);
        put_response_le16(&mut response, 4, GET_HW_SPEC_CMD | HOST_CMD_RET_BIT);
        put_response_le16(&mut response, 6, 79);
        put_response_le16(&mut response, 10, HOST_CMD_RESULT_OK);
        response[20..26].copy_from_slice(&[2, 4, 6, 8, 10, 12]);
        put_response_le16(&mut response, 75, 0x01c7);
        put_response_le16(&mut response, 77, 4);
        put_response_le16(&mut response, 79, 1);
        response[81] = 2;
        response[82] = 1;

        let parsed = parse_hw_spec_response(0, &response).unwrap();

        assert_eq!(parsed.key_api_version, Some((2, 1)));
    }

    #[test]
    fn parse_hw_spec_response_rejects_short_buffer_or_declared_length() {
        let mut response = [0u8; HW_SPEC_MIN_RESPONSE_LEN - 1];
        put_response_le16(&mut response, 0, HW_SPEC_MIN_RESPONSE_LEN as u16);
        assert_eq!(
            parse_hw_spec_response(0, &response),
            Err(HwSpecCmdError::TooShort)
        );

        let mut short_declared = [0u8; HW_SPEC_MIN_RESPONSE_LEN];
        put_response_le16(
            &mut short_declared,
            0,
            (HW_SPEC_MIN_RESPONSE_LEN - 1) as u16,
        );
        assert_eq!(
            parse_hw_spec_response(0, &short_declared),
            Err(HwSpecCmdError::TooShort)
        );
    }

    #[test]
    fn parse_hw_spec_response_rejects_wrong_command_id() {
        let mut response = [0u8; 96];
        put_response_le16(&mut response, 0, 75);
        put_response_le16(&mut response, 4, 0x8004);
        put_response_le16(&mut response, 6, GET_HW_SPEC_GEN_SIZE as u16);

        assert_eq!(
            parse_hw_spec_response(0, &response),
            Err(HwSpecCmdError::BadCommand { got: 0x8004 })
        );
    }

    #[test]
    fn parse_hw_spec_response_rejects_non_zero_result() {
        let mut response = [0u8; 96];
        put_response_le16(&mut response, 0, 75);
        put_response_le16(&mut response, 4, GET_HW_SPEC_CMD | HOST_CMD_RET_BIT);
        put_response_le16(&mut response, 6, GET_HW_SPEC_GEN_SIZE as u16);
        put_response_le16(&mut response, 10, 0x0002);

        assert_eq!(
            parse_hw_spec_response(0, &response),
            Err(HwSpecCmdError::FwResult { code: 0x0002 })
        );
    }

    #[test]
    fn parse_scan_ext_response_accepts_command_done_status() {
        let mut response = [0u8; HOST_CMD_MIN_RESPONSE_LEN];
        put_response_le16(&mut response, 0, HOST_CMD_MIN_RESPONSE_LEN as u16);
        put_response_le16(&mut response, 4, SCAN_EXT_CMD | HOST_CMD_RET_BIT);
        put_response_le16(&mut response, 10, HOST_CMD_RESULT_OK);

        assert_eq!(parse_scan_ext_response(&response), Ok(()));
    }

    #[test]
    fn parse_scan_ext_response_rejects_wrong_command_or_result() {
        let mut wrong_command = [0u8; HOST_CMD_MIN_RESPONSE_LEN];
        put_response_le16(&mut wrong_command, 0, HOST_CMD_MIN_RESPONSE_LEN as u16);
        put_response_le16(&mut wrong_command, 4, GET_HW_SPEC_CMD | HOST_CMD_RET_BIT);
        assert_eq!(
            parse_scan_ext_response(&wrong_command),
            Err(HwSpecCmdError::BadCommand {
                got: GET_HW_SPEC_CMD | HOST_CMD_RET_BIT
            })
        );

        let mut fw_error = [0u8; HOST_CMD_MIN_RESPONSE_LEN];
        put_response_le16(&mut fw_error, 0, HOST_CMD_MIN_RESPONSE_LEN as u16);
        put_response_le16(&mut fw_error, 4, SCAN_EXT_CMD | HOST_CMD_RET_BIT);
        put_response_le16(&mut fw_error, 10, 0x0004);
        assert_eq!(
            parse_scan_ext_response(&fw_error),
            Err(HwSpecCmdError::FwResult { code: 0x0004 })
        );
    }

    #[test]
    fn visit_scan_response_extracts_legacy_bss_fields() {
        let ies = [0, 4, b't', b'e', b's', b't', 3, 1, 6];
        let beacon_size = SCAN_BSS_FIXED_LEN + ies.len();
        let descriptors_len = 2 + beacon_size;
        let response_len = SCAN_RESPONSE_FIXED_LEN + descriptors_len;
        let mut response = [0u8; 64];
        put_response_le16(&mut response, 0, response_len as u16);
        put_response_le16(&mut response, 2, MWIFIEX_TYPE_CMD);
        put_response_le16(&mut response, 4, SCAN_CMD | HOST_CMD_RET_BIT);
        put_response_le16(&mut response, 6, (response_len - INTF_HEADER_LEN) as u16);
        put_response_le16(&mut response, 10, HOST_CMD_RESULT_OK);
        put_response_le16(&mut response, 12, descriptors_len as u16);
        response[14] = 1;
        put_response_le16(&mut response, 15, beacon_size as u16);
        response[17..23].copy_from_slice(&[1, 2, 3, 4, 5, 6]);
        response[23] = 42;
        response[24..36].copy_from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        response[36..36 + ies.len()].copy_from_slice(&ies);

        let mut visits = 0;
        let count = visit_scan_response(&response, |bss| {
            visits += 1;
            assert_eq!(bss.bssid, [1, 2, 3, 4, 5, 6]);
            assert_eq!(bss.rssi_dbm, -42);
            assert_eq!(
                bss.fixed_beacon_params,
                &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
            );
            assert_eq!(bss.ies, &ies);
        })
        .unwrap();

        assert_eq!(count, 1);
        assert_eq!(visits, 1);
    }

    #[test]
    fn visit_scan_response_validates_all_descriptors_before_visiting() {
        let mut response = [0u8; SCAN_RESPONSE_FIXED_LEN + 2 + SCAN_BSS_FIXED_LEN];
        let response_len = response.len();
        put_response_le16(&mut response, 0, response_len as u16);
        put_response_le16(&mut response, 4, SCAN_CMD | HOST_CMD_RET_BIT);
        put_response_le16(&mut response, 10, HOST_CMD_RESULT_OK);
        put_response_le16(
            &mut response,
            12,
            (response_len - SCAN_RESPONSE_FIXED_LEN) as u16,
        );
        response[14] = 2;
        put_response_le16(&mut response, 15, SCAN_BSS_FIXED_LEN as u16);
        let mut visits = 0;

        assert_eq!(
            visit_scan_response(&response, |_| visits += 1),
            Err(HwSpecCmdError::TooShort)
        );
        assert_eq!(visits, 0);
    }

    #[test]
    fn build_pcie_desc_details_pins_linux_ring_layout() {
        let mut out = [0xa5u8; PCIE_DESC_DETAILS_CMD_TOTAL_LEN + 4];
        let rings = PcieDescriptorRings {
            tx_phys: 0x1122_3344_5566_7788,
            rx_phys: 0x99aa_bbcc_ddee_ff00,
            event_phys: 0x0102_0304_0506_0708,
        };

        let len = build_pcie_desc_details(0x34, rings, &mut out).unwrap();

        let expected = [
            0x38, 0x00, 0x01, 0x00, 0xfa, 0x00, 0x34, 0x00, 0x34, 0x00, 0x00, 0x00, 0x88, 0x77,
            0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x20, 0x00, 0x00, 0x00, 0x00, 0xff, 0xee, 0xdd,
            0xcc, 0xbb, 0xaa, 0x99, 0x20, 0x00, 0x00, 0x00, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03,
            0x02, 0x01, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(&out[..expected.len()], &expected);
        assert_eq!(len, expected.len());
        assert_eq!(&out[len..], &[0xa5; 4]);
    }

    #[test]
    fn build_pcie_desc_details_rejects_missing_ring_or_short_output() {
        let rings = PcieDescriptorRings {
            tx_phys: 0,
            rx_phys: 2,
            event_phys: 3,
        };
        let mut out = [0u8; PCIE_DESC_DETAILS_CMD_TOTAL_LEN];
        assert_eq!(
            build_pcie_desc_details(0, rings, &mut out),
            Err(MarvellCmdError::InvalidDescriptorAddress)
        );

        let rings = PcieDescriptorRings {
            tx_phys: 1,
            rx_phys: 2,
            event_phys: 3,
        };
        let mut short = [0u8; PCIE_DESC_DETAILS_CMD_TOTAL_LEN - 1];
        assert_eq!(
            build_pcie_desc_details(0, rings, &mut short),
            Err(MarvellCmdError::OutputBufferTooSmall)
        );
    }

    #[test]
    fn build_func_init_pins_empty_linux_hostcmd_shape() {
        let mut out = [0xa5u8; FUNC_INIT_CMD_TOTAL_LEN + 2];
        let len = build_func_init(0x35, &mut out).unwrap();

        assert_eq!(
            &out[..len],
            &[0x0c, 0x00, 0x01, 0x00, 0xa9, 0x00, 0x08, 0x00, 0x35, 0x00, 0x00, 0x00]
        );
        assert_eq!(&out[len..], &[0xa5; 2]);
    }

    #[test]
    fn build_mac_control_pins_rx_tx_ethernet_ii_action() {
        let mut out = [0xa5u8; MAC_CONTROL_CMD_TOTAL_LEN + 2];

        let len = build_mac_control(0x78, &mut out).unwrap();

        assert_eq!(
            &out[..len],
            &[
                0x10, 0x00, 0x01, 0x00, 0x28, 0x00, 0x0c, 0x00, 0x78, 0x00, 0x00, 0x00, 0x13, 0x00,
                0x00, 0x00,
            ]
        );
        assert_eq!(&out[len..], &[0xa5; 2]);

        let mut short = [0u8; MAC_CONTROL_CMD_TOTAL_LEN - 1];
        assert_eq!(
            build_mac_control(0, &mut short),
            Err(MarvellCmdError::OutputBufferTooSmall)
        );
    }

    #[test]
    fn parse_control_responses_require_matching_command_and_sequence() {
        let mut response = [0u8; HOST_CMD_MIN_RESPONSE_LEN];
        put_response_le16(&mut response, 0, HOST_CMD_MIN_RESPONSE_LEN as u16);
        put_response_le16(&mut response, 6, S_DS_GEN as u16);
        put_response_le16(&mut response, 8, 0x34);
        put_response_le16(&mut response, 4, PCIE_DESC_DETAILS_CMD | HOST_CMD_RET_BIT);

        assert_eq!(parse_pcie_desc_details_response(0x34, &response), Ok(()));
        assert_eq!(
            parse_pcie_desc_details_response(0x35, &response),
            Err(MarvellCmdError::BadSequence { got: 0x34 })
        );

        put_response_le16(&mut response, 4, MAC_CONTROL_CMD | HOST_CMD_RET_BIT);
        assert_eq!(parse_mac_control_response(0x34, &response), Ok(()));
        assert_eq!(
            parse_pcie_desc_details_response(0x34, &response),
            Err(MarvellCmdError::BadCommand {
                got: MAC_CONTROL_CMD | HOST_CMD_RET_BIT
            })
        );
    }

    #[test]
    fn pci_command_enable_plan_is_idempotent_and_fail_closed() {
        assert_eq!(
            plan_pci_memory_bus_master_enable(0x2b38_11ab, 0x0406),
            PciCommandEnablePlan::AlreadyEnabled
        );
        assert_eq!(
            plan_pci_memory_bus_master_enable(0x2b38_11ab, 0x0402),
            PciCommandEnablePlan::Write(0x0406)
        );
        assert_eq!(
            plan_pci_memory_bus_master_enable(0x2b38_11ab, u16::MAX),
            PciCommandEnablePlan::Unavailable
        );
        assert_eq!(
            plan_pci_memory_bus_master_enable(u32::MAX, 0x0402),
            PciCommandEnablePlan::Unavailable
        );
    }

    #[test]
    fn single_bss_sequence_windows_start_advance_and_wrap_without_context_bits() {
        let mut allocator = SingleBssHostCmdSequenceAllocator::new();
        assert_eq!(allocator.reserve_window(5), 1);
        assert_eq!(allocator.reserve_window(5), 6);
        assert_eq!(allocator.reserve_window(5), 11);

        let mut boundary = SingleBssHostCmdSequenceAllocator { next: 251 };
        let base = boundary.reserve_window(5);
        assert_eq!(base, 251);
        let sequences = [base, base + 1, base + 2, base + 3, base + 4];
        assert_eq!(sequences, [251, 252, 253, 254, 255]);
        assert!(sequences
            .iter()
            .all(|sequence| has_single_bss_host_cmd_context(u16::from(*sequence))));
        assert_eq!(boundary.reserve_window(5), 1);

        let mut crossing = SingleBssHostCmdSequenceAllocator { next: 252 };
        assert_eq!(crossing.reserve_window(5), 1);
    }

    #[test]
    fn connection_builders_reject_old_bss_one_sequence_context() {
        let rings = PcieDescriptorRings {
            tx_phys: 1,
            rx_phys: 2,
            event_phys: 3,
        };
        let mut out = [0u8; 128];
        let expected = Err(MarvellCmdError::InvalidSequenceContext { got: 0x0100 });
        assert_eq!(build_pcie_desc_details(0x0100, rings, &mut out), expected);
        assert_eq!(build_mac_control(0x0100, &mut out), expected);
        assert!(!has_single_bss_host_cmd_context(0x0100));
    }

    #[test]
    fn redacted_response_header_copies_only_fixed_protocol_words() {
        let mut response = [0xa5; HOST_CMD_MIN_RESPONSE_LEN + 4];
        put_response_le16(&mut response, 0, 0x000c);
        put_response_le16(&mut response, 2, MWIFIEX_TYPE_CMD);
        put_response_le16(&mut response, 4, MAC_CONTROL_CMD | HOST_CMD_RET_BIT);
        put_response_le16(&mut response, 6, S_DS_GEN as u16);
        put_response_le16(&mut response, 8, 0x002a);
        put_response_le16(&mut response, 10, 0x0002);

        assert_eq!(
            redacted_response_header(&response),
            Some(MarvellResponseHeader {
                interface_len: 0x000c,
                interface_type: MWIFIEX_TYPE_CMD,
                command: MAC_CONTROL_CMD | HOST_CMD_RET_BIT,
                host_command_size: S_DS_GEN as u16,
                sequence: 0x002a,
                result: 0x0002,
            })
        );
        assert_eq!(
            redacted_response_header(&response[..HOST_CMD_MIN_RESPONSE_LEN - 1]),
            None
        );
    }

    #[test]
    fn parse_hw_spec_response_requires_exact_sequence_and_size() {
        let mut response = [0u8; GET_HW_SPEC_CMD_TOTAL_LEN];
        put_response_le16(&mut response, 0, GET_HW_SPEC_CMD_TOTAL_LEN as u16);
        put_response_le16(&mut response, 4, GET_HW_SPEC_CMD | HOST_CMD_RET_BIT);
        put_response_le16(&mut response, 6, GET_HW_SPEC_GEN_SIZE as u16);
        put_response_le16(&mut response, 8, 7);

        assert_eq!(
            parse_hw_spec_response(6, &response),
            Err(HwSpecCmdError::BadSequence { got: 7 })
        );
        put_response_le16(&mut response, 8, 6);
        put_response_le16(&mut response, 6, (GET_HW_SPEC_GEN_SIZE - 1) as u16);
        assert_eq!(
            parse_hw_spec_response(6, &response),
            Err(HwSpecCmdError::BadLength)
        );
        assert_eq!(
            parse_hw_spec_response(0x0100, &response),
            Err(HwSpecCmdError::InvalidSequenceContext { got: 0x0100 })
        );
    }

    #[test]
    fn init_commands_require_single_bss_sequence_context() {
        let mut out = [0u8; GET_HW_SPEC_CMD_TOTAL_LEN];
        assert_eq!(
            build_get_hw_spec(0x0100, &mut out),
            Err(HwSpecCmdError::InvalidSequenceContext { got: 0x0100 })
        );
        assert_eq!(
            build_func_init(0x0100, &mut out),
            Err(MarvellCmdError::InvalidSequenceContext { got: 0x0100 })
        );
    }

    #[test]
    fn host_cmd_epoch_rejects_stale_high_and_accepts_only_low_then_done() {
        const CMD_DONE: u32 = 1 << 2;
        assert!(!host_cmd_done_low_after_clear(CMD_DONE, CMD_DONE));
        assert!(!host_cmd_done_low_after_clear(u32::MAX, CMD_DONE));
        assert!(host_cmd_done_low_after_clear(0, CMD_DONE));
        assert!(!host_cmd_done_is_current(false, CMD_DONE, CMD_DONE));
        assert!(!host_cmd_done_is_current(true, 0, CMD_DONE));
        assert!(!host_cmd_done_is_current(true, u32::MAX, CMD_DONE));
        assert!(host_cmd_done_is_current(true, CMD_DONE, CMD_DONE));
    }

    #[test]
    fn host_cmd_empty_response_is_typed_without_scratch_readback_contract() {
        let empty = redacted_response_header(&[0; HOST_CMD_MIN_RESPONSE_LEN]).unwrap();
        assert!(empty.is_empty());
        assert!(!MarvellResponseHeader {
            command: PCIE_DESC_DETAILS_CMD | HOST_CMD_RET_BIT,
            ..empty
        }
        .is_empty());
    }

    #[test]
    fn connection_pci_enable_and_post_enable_mmio_liveness_are_fail_closed() {
        use crate::marvell_wifi_fw::FIRMWARE_READY_PCIE;

        assert!(marvell_8897_pci_identity_valid(0x2b38_11ab));
        assert!(!marvell_8897_pci_identity_valid(u32::MAX));
        assert!(pci_memory_bus_master_enabled(0x0006));
        assert!(!pci_memory_bus_master_enabled(0x0002));
        assert!(!pci_memory_bus_master_enabled(0x0004));
        assert!(!pci_memory_bus_master_enabled(u16::MAX));
        assert_eq!(
            connection_mmio_liveness(FIRMWARE_READY_PCIE, 0),
            ConnectionMmioLiveness::Ready
        );
        assert_eq!(
            connection_mmio_liveness(FIRMWARE_READY_PCIE, u32::MAX),
            ConnectionMmioLiveness::MmioUnavailable
        );
        assert_eq!(
            connection_mmio_liveness(0, 0),
            ConnectionMmioLiveness::FirmwareNotReady
        );
    }

    #[test]
    fn build_associate_24ghz_pins_linux_tlvs_and_complete_rsn_ie() {
        let rsn_ie = [
            0x30, 0x14, 0x01, 0x00, 0x00, 0x0f, 0xac, 0x04, 0x01, 0x00, 0x00, 0x0f, 0xac, 0x04,
            0x01, 0x00, 0x00, 0x0f, 0xac, 0x02, 0x00, 0x00,
        ];
        let bss = AssociationBss {
            bssid: [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
            ssid: b"net",
            channel: 6,
            beacon_period: 100,
            capability_info: 0x0431,
            rates: &[0x82, 0x84, 0x8b, 0x96, 0x0c, 0x12, 0x18, 0x24],
            rsn_or_wpa_ie: Some(&rsn_ie),
        };
        let mut out = [0xa5u8; 112];

        let len = build_associate_24ghz(0x44, bss, &mut out).unwrap();

        let expected = [
            0x64, 0x00, 0x01, 0x00, 0x12, 0x00, 0x60, 0x00, 0x44, 0x00, 0x00, 0x00, 0x00, 0x11,
            0x22, 0x33, 0x44, 0x55, 0x31, 0x04, 0x0a, 0x00, 0x64, 0x00, 0x00, 0x00, 0x00, 0x03,
            0x00, 0x6e, 0x65, 0x74, 0x03, 0x00, 0x01, 0x00, 0x06, 0x04, 0x00, 0x06, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x08, 0x00, 0x82, 0x84, 0x8b, 0x96, 0x0c,
            0x12, 0x18, 0x24, 0x1f, 0x01, 0x02, 0x00, 0x00, 0x00, 0x01, 0x01, 0x07, 0x00, 0x00,
            0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x00, 0x14, 0x00, 0x01, 0x00, 0x00, 0x0f,
            0xac, 0x04, 0x01, 0x00, 0x00, 0x0f, 0xac, 0x04, 0x01, 0x00, 0x00, 0x0f, 0xac, 0x02,
            0x00, 0x00,
        ];
        assert_eq!(&out[..expected.len()], &expected);
        assert_eq!(len, expected.len());
        assert!(out[len..].iter().all(|byte| *byte == 0xa5));
    }

    #[test]
    fn build_associate_24ghz_preserves_wpa_ie_and_rejects_bad_inputs() {
        let wpa_ie = [0xdd, 0x06, 0x00, 0x50, 0xf2, 0x01, 0x01, 0x00];
        let bss = AssociationBss {
            bssid: [0x02, 1, 2, 3, 4, 5],
            ssid: b"x",
            channel: 1,
            beacon_period: 100,
            capability_info: 0xffff,
            rates: &[0x82],
            rsn_or_wpa_ie: Some(&wpa_ie),
        };
        let mut out = [0u8; 128];

        let len = build_associate_24ghz(0, bss, &mut out).unwrap();
        assert_eq!(
            &out[len - 10..len],
            &[0xdd, 0, 6, 0, 0, 0x50, 0xf2, 1, 1, 0]
        );
        assert_eq!(&out[18..20], &MWIFIEX_CAPINFO_MASK.to_le_bytes());

        assert_eq!(
            build_associate_24ghz(0x0100, bss, &mut out),
            Err(MarvellCmdError::InvalidSequenceContext { got: 0x0100 })
        );

        assert_eq!(
            build_associate_24ghz(0, AssociationBss { channel: 0, ..bss }, &mut out),
            Err(MarvellCmdError::InvalidChannel)
        );
        assert_eq!(
            build_associate_24ghz(0, AssociationBss { rates: &[], ..bss }, &mut out,),
            Err(MarvellCmdError::InvalidRates)
        );
        assert_eq!(
            build_associate_24ghz(
                0,
                AssociationBss {
                    rsn_or_wpa_ie: Some(&[0xdd, 4, 1, 2, 3, 4]),
                    ..bss
                },
                &mut out,
            ),
            Err(MarvellCmdError::InvalidSecurityIe)
        );
        let mut short = [0u8; 8];
        assert_eq!(
            build_associate_24ghz(0, bss, &mut short),
            Err(MarvellCmdError::OutputBufferTooSmall)
        );
    }

    #[test]
    fn parse_associate_response_returns_ieee_status_and_aid() {
        let mut response = [0u8; ASSOCIATE_MIN_RESPONSE_LEN];
        put_response_le16(&mut response, 0, ASSOCIATE_MIN_RESPONSE_LEN as u16);
        put_response_le16(&mut response, 2, MWIFIEX_TYPE_CMD);
        put_response_le16(&mut response, 4, ASSOCIATE_CMD | HOST_CMD_RET_BIT);
        put_response_le16(&mut response, 6, (S_DS_GEN + 6) as u16);
        put_response_le16(&mut response, 10, HOST_CMD_RESULT_OK);
        put_response_le16(&mut response, 12, 0x0431);
        put_response_le16(&mut response, 14, 0);
        put_response_le16(&mut response, 16, 0xc02a);

        assert_eq!(
            parse_associate_response(0, &response),
            Ok(AssociationResponse {
                capability_info: 0x0431,
                status_code: 0,
                raw_association_id: 0xc02a,
                association_id: Some(42),
            })
        );
    }

    #[test]
    fn parse_associate_response_keeps_ieee_failure_distinct_from_firmware_error() {
        let mut response = [0u8; ASSOCIATE_MIN_RESPONSE_LEN];
        put_response_le16(&mut response, 0, ASSOCIATE_MIN_RESPONSE_LEN as u16);
        put_response_le16(&mut response, 4, ASSOCIATE_CMD | HOST_CMD_RET_BIT);
        put_response_le16(&mut response, 6, (S_DS_GEN + 6) as u16);
        put_response_le16(&mut response, 12, 0xfffd);
        put_response_le16(&mut response, 14, 17);
        put_response_le16(&mut response, 16, 0xffff);

        let parsed = parse_associate_response(0, &response).unwrap();
        assert_eq!(parsed.status_code, 17);
        assert_eq!(parsed.raw_association_id, 0xffff);
        assert_eq!(parsed.association_id, None);

        put_response_le16(&mut response, 10, 3);
        assert_eq!(
            parse_associate_response(0, &response),
            Err(MarvellCmdError::FwResult { code: 3 })
        );
    }

    #[test]
    fn parse_associate_response_rejects_bad_framing_command_and_aid() {
        let mut response = [0u8; ASSOCIATE_MIN_RESPONSE_LEN];
        put_response_le16(&mut response, 0, ASSOCIATE_MIN_RESPONSE_LEN as u16);
        put_response_le16(&mut response, 4, ASSOCIATE_CMD | HOST_CMD_RET_BIT);
        put_response_le16(&mut response, 6, (S_DS_GEN + 6) as u16);
        put_response_le16(&mut response, 16, 42);
        assert_eq!(
            parse_associate_response(0, &response),
            Err(MarvellCmdError::InvalidAssociationId { raw: 42 })
        );

        put_response_le16(&mut response, 4, SCAN_CMD | HOST_CMD_RET_BIT);
        assert_eq!(
            parse_associate_response(0, &response),
            Err(MarvellCmdError::BadCommand {
                got: SCAN_CMD | HOST_CMD_RET_BIT
            })
        );

        put_response_le16(&mut response, 4, ASSOCIATE_CMD | HOST_CMD_RET_BIT);
        put_response_le16(&mut response, 6, (S_DS_GEN + 7) as u16);
        assert_eq!(
            parse_associate_response(0, &response),
            Err(MarvellCmdError::BadLength)
        );
        assert_eq!(
            parse_associate_response(0, &response[..ASSOCIATE_MIN_RESPONSE_LEN - 1]),
            Err(MarvellCmdError::TooShort)
        );

        put_response_le16(&mut response, 6, (S_DS_GEN + 6) as u16);
        put_response_le16(&mut response, 8, 0x1234);
        assert_eq!(
            parse_associate_response(0x4321, &response),
            Err(MarvellCmdError::BadSequence { got: 0x1234 })
        );
    }
}
