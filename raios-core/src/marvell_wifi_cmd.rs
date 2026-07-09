//! Pure Marvell 88W8897 command-mailbox packet helpers.
//!
//! No MMIO, DMA, allocation, or authority lives here. The kernel-side PCIe
//! shell owns transport and timeouts.

pub const GET_HW_SPEC_CMD: u16 = 0x0003;
pub const SCAN_CMD: u16 = 0x0006;
pub const SCAN_EXT_CMD: u16 = 0x0107;
pub const MWIFIEX_TYPE_CMD: u16 = 1;
pub const HOST_CMD_RET_BIT: u16 = 0x8000;
pub const HOST_CMD_RESULT_OK: u16 = 0;
pub const S_DS_GEN: usize = 8;
pub const INTF_HEADER_LEN: usize = 4;
pub const GET_HW_SPEC_BODY_LEN: usize = 63;
pub const GET_HW_SPEC_GEN_SIZE: usize = S_DS_GEN + GET_HW_SPEC_BODY_LEN;
pub const GET_HW_SPEC_CMD_TOTAL_LEN: usize = INTF_HEADER_LEN + GET_HW_SPEC_GEN_SIZE;
pub const HW_SPEC_MIN_RESPONSE_LEN: usize = 34;
pub const HOST_CMD_MIN_RESPONSE_LEN: usize = INTF_HEADER_LEN + S_DS_GEN;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HwSpecCmdError {
    TooShort,
    BadCommand { got: u16 },
    FwResult { code: u16 },
    OutputBufferTooSmall,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HwSpec {
    pub mac: [u8; 6],
    pub fw_release: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyScanBss<'a> {
    pub bssid: [u8; 6],
    pub rssi_dbm: i16,
    pub fixed_beacon_params: &'a [u8],
    pub ies: &'a [u8],
}

pub fn build_get_hw_spec(seq: u16, out: &mut [u8]) -> Result<usize, HwSpecCmdError> {
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

pub fn parse_hw_spec_response(buf: &[u8]) -> Result<HwSpec, HwSpecCmdError> {
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

    let command = le16(buf, 4);
    let expected = GET_HW_SPEC_CMD | HOST_CMD_RET_BIT;
    if command != expected {
        return Err(HwSpecCmdError::BadCommand { got: command });
    }

    let result = le16(buf, 10);
    if result != HOST_CMD_RESULT_OK {
        return Err(HwSpecCmdError::FwResult { code: result });
    }

    let mut mac = [0u8; 6];
    mac.copy_from_slice(&buf[20..26]);
    Ok(HwSpec {
        mac,
        fw_release: le32(buf, 30),
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

        let len = build_get_hw_spec(0x1234, &mut out).unwrap();

        assert_eq!(len, 75);
        assert_eq!(&out[0..2], &75u16.to_le_bytes());
        assert_eq!(&out[2..4], &1u16.to_le_bytes());
        assert_eq!(&out[4..6], &0x0003u16.to_le_bytes());
        assert_eq!(&out[6..8], &71u16.to_le_bytes());
        assert_eq!(&out[8..10], &0x1234u16.to_le_bytes());
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

        let parsed = parse_hw_spec_response(&response).unwrap();

        assert_eq!(parsed.mac, [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
        assert_eq!(parsed.fw_release, 0x1568_0019);
    }

    #[test]
    fn parse_hw_spec_response_rejects_short_buffer_or_declared_length() {
        let mut response = [0u8; HW_SPEC_MIN_RESPONSE_LEN - 1];
        put_response_le16(&mut response, 0, HW_SPEC_MIN_RESPONSE_LEN as u16);
        assert_eq!(
            parse_hw_spec_response(&response),
            Err(HwSpecCmdError::TooShort)
        );

        let mut short_declared = [0u8; HW_SPEC_MIN_RESPONSE_LEN];
        put_response_le16(
            &mut short_declared,
            0,
            (HW_SPEC_MIN_RESPONSE_LEN - 1) as u16,
        );
        assert_eq!(
            parse_hw_spec_response(&short_declared),
            Err(HwSpecCmdError::TooShort)
        );
    }

    #[test]
    fn parse_hw_spec_response_rejects_wrong_command_id() {
        let mut response = [0u8; 96];
        put_response_le16(&mut response, 0, 75);
        put_response_le16(&mut response, 4, 0x8004);

        assert_eq!(
            parse_hw_spec_response(&response),
            Err(HwSpecCmdError::BadCommand { got: 0x8004 })
        );
    }

    #[test]
    fn parse_hw_spec_response_rejects_non_zero_result() {
        let mut response = [0u8; 96];
        put_response_le16(&mut response, 0, 75);
        put_response_le16(&mut response, 4, GET_HW_SPEC_CMD | HOST_CMD_RET_BIT);
        put_response_le16(&mut response, 10, 0x0002);

        assert_eq!(
            parse_hw_spec_response(&response),
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
}
