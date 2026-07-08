//! Pure Marvell 88W8897 command-mailbox packet helpers.
//!
//! No MMIO, DMA, allocation, or authority lives here. The kernel-side PCIe
//! shell owns transport and timeouts.

pub const GET_HW_SPEC_CMD: u16 = 0x0003;
pub const MWIFIEX_TYPE_CMD: u16 = 1;
pub const HOST_CMD_RET_BIT: u16 = 0x8000;
pub const HOST_CMD_RESULT_OK: u16 = 0;
pub const S_DS_GEN: usize = 8;
pub const INTF_HEADER_LEN: usize = 4;
pub const GET_HW_SPEC_BODY_LEN: usize = 63;
pub const GET_HW_SPEC_GEN_SIZE: usize = S_DS_GEN + GET_HW_SPEC_BODY_LEN;
pub const GET_HW_SPEC_CMD_TOTAL_LEN: usize = INTF_HEADER_LEN + GET_HW_SPEC_GEN_SIZE;
pub const HW_SPEC_MIN_RESPONSE_LEN: usize = 34;

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

fn put_le16(out: &mut [u8], offset: usize, value: u16) {
    out[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
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
}
