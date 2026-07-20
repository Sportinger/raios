//! Pure packet helpers for the NXP 88W8897 firmware supplicant.
//!
//! The layouts follow the NXP `mlan` definitions for
//! `HostCmd_CMD_SUPPLICANT_PROFILE` and `HostCmd_CMD_SUPPLICANT_PMK`.
//! This module performs no MMIO, DMA, allocation, logging, or secret retention.

use crate::marvell_wifi_cmd::has_single_bss_host_cmd_context;

pub const FW_CAP_FIRMWARE_SUPPLICANT: u32 = 1 << 21;
pub const EVENT_PORT_RELEASE: u32 = 0x002b;

pub const SUPPLICANT_PMK_CMD: u16 = 0x00c4;
pub const SUPPLICANT_PROFILE_CMD: u16 = 0x00c5;
pub const SUPPLICANT_PROFILE_SET_TOTAL_LEN: usize = 28;
pub const SUPPLICANT_PMK_SET_MAX_TOTAL_LEN: usize = 34 + MAX_SSID_LEN + MAX_PASSPHRASE_LEN;

pub const MAX_SSID_LEN: usize = 32;
pub const MIN_PASSPHRASE_LEN: usize = 8;
pub const MAX_PASSPHRASE_LEN: usize = 63;

const MWIFIEX_TYPE_CMD: u16 = 1;
const HOST_CMD_RET_BIT: u16 = 0x8000;
const HOST_CMD_RESULT_OK: u16 = 0;
const HOST_CMD_ACT_GEN_SET: u16 = 1;
const INTF_HEADER_LEN: usize = 4;
const S_DS_GEN: usize = 8;
const TLV_HEADER_LEN: usize = 4;

const TLV_TYPE_SSID: u16 = 0x0000;
const TLV_TYPE_BSSID: u16 = 0x0123;
const TLV_TYPE_PASSPHRASE: u16 = 0x013c;
const TLV_TYPE_ENCRYPTION_PROTO: u16 = 0x0140;
const TLV_TYPE_CIPHER: u16 = 0x0142;

const RSN_MODE_WPA2: u16 = 0x0020;
const CIPHER_AES: u8 = 0x08;
const PROFILE_FIXED_LEN: usize = 4;
const PROFILE_ENCRYPTION_TLV_LEN: usize = TLV_HEADER_LEN + 2;
const PROFILE_CIPHER_TLV_LEN: usize = TLV_HEADER_LEN + 2;
const PROFILE_GEN_SIZE: usize =
    S_DS_GEN + PROFILE_FIXED_LEN + PROFILE_ENCRYPTION_TLV_LEN + PROFILE_CIPHER_TLV_LEN;
const PMK_FIXED_LEN: usize = 4;
const BSSID_LEN: usize = 6;
const RESPONSE_MIN_LEN: usize = INTF_HEADER_LEN + S_DS_GEN;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SupplicantError {
    OutputBufferTooSmall,
    InvalidSsidLength,
    InvalidBssid,
    InvalidPassphraseLength,
    ResponseTooShort,
    BadInterfaceLength,
    BadInterfaceType { got: u16 },
    BadHostCommandLength,
    BadCommand { got: u16 },
    BadSequence { got: u16 },
    FirmwareResult { code: u16 },
    InvalidSequenceContext { got: u16 },
}

pub fn build_supplicant_profile_set(seq: u16, out: &mut [u8]) -> Result<usize, SupplicantError> {
    require_single_bss_host_cmd_context(seq)?;
    if out.len() < SUPPLICANT_PROFILE_SET_TOTAL_LEN {
        return Err(SupplicantError::OutputBufferTooSmall);
    }

    out[..SUPPLICANT_PROFILE_SET_TOTAL_LEN].fill(0);
    write_command_header(
        out,
        SUPPLICANT_PROFILE_SET_TOTAL_LEN,
        SUPPLICANT_PROFILE_CMD,
        PROFILE_GEN_SIZE,
        seq,
    );
    put_le16(out, 12, HOST_CMD_ACT_GEN_SET);

    let mut offset = INTF_HEADER_LEN + S_DS_GEN + PROFILE_FIXED_LEN;
    write_tlv_header(out, offset, TLV_TYPE_ENCRYPTION_PROTO, 2);
    put_le16(out, offset + TLV_HEADER_LEN, RSN_MODE_WPA2);
    offset += PROFILE_ENCRYPTION_TLV_LEN;

    write_tlv_header(out, offset, TLV_TYPE_CIPHER, 2);
    out[offset + TLV_HEADER_LEN] = CIPHER_AES;
    out[offset + TLV_HEADER_LEN + 1] = CIPHER_AES;

    Ok(SUPPLICANT_PROFILE_SET_TOTAL_LEN)
}

pub fn build_supplicant_pmk_set(
    seq: u16,
    bssid: [u8; BSSID_LEN],
    ssid: &[u8],
    passphrase: &[u8],
    out: &mut [u8],
) -> Result<usize, SupplicantError> {
    require_single_bss_host_cmd_context(seq)?;
    if ssid.is_empty() || ssid.len() > MAX_SSID_LEN {
        return Err(SupplicantError::InvalidSsidLength);
    }
    if bssid == [0; BSSID_LEN] || bssid[0] & 1 != 0 {
        return Err(SupplicantError::InvalidBssid);
    }
    if !(MIN_PASSPHRASE_LEN..=MAX_PASSPHRASE_LEN).contains(&passphrase.len()) {
        return Err(SupplicantError::InvalidPassphraseLength);
    }

    let total_len = 34 + ssid.len() + passphrase.len();
    if out.len() < total_len {
        return Err(SupplicantError::OutputBufferTooSmall);
    }

    out[..total_len].fill(0);
    write_command_header(
        out,
        total_len,
        SUPPLICANT_PMK_CMD,
        total_len - INTF_HEADER_LEN,
        seq,
    );
    put_le16(out, 12, HOST_CMD_ACT_GEN_SET);

    let mut offset = INTF_HEADER_LEN + S_DS_GEN + PMK_FIXED_LEN;
    write_tlv_header(out, offset, TLV_TYPE_SSID, ssid.len() as u16);
    offset += TLV_HEADER_LEN;
    out[offset..offset + ssid.len()].copy_from_slice(ssid);
    offset += ssid.len();

    write_tlv_header(out, offset, TLV_TYPE_BSSID, BSSID_LEN as u16);
    offset += TLV_HEADER_LEN;
    out[offset..offset + BSSID_LEN].copy_from_slice(&bssid);
    offset += BSSID_LEN;

    write_tlv_header(out, offset, TLV_TYPE_PASSPHRASE, passphrase.len() as u16);
    offset += TLV_HEADER_LEN;
    out[offset..offset + passphrase.len()].copy_from_slice(passphrase);

    Ok(total_len)
}

fn require_single_bss_host_cmd_context(sequence: u16) -> Result<(), SupplicantError> {
    if has_single_bss_host_cmd_context(sequence) {
        Ok(())
    } else {
        Err(SupplicantError::InvalidSequenceContext { got: sequence })
    }
}

pub fn parse_supplicant_profile_response(
    expected_seq: u16,
    buf: &[u8],
) -> Result<(), SupplicantError> {
    parse_response(buf, SUPPLICANT_PROFILE_CMD, expected_seq)
}

pub fn parse_supplicant_pmk_response(expected_seq: u16, buf: &[u8]) -> Result<(), SupplicantError> {
    parse_response(buf, SUPPLICANT_PMK_CMD, expected_seq)
}

fn parse_response(buf: &[u8], expected_cmd: u16, expected_seq: u16) -> Result<(), SupplicantError> {
    if buf.len() < RESPONSE_MIN_LEN {
        return Err(SupplicantError::ResponseTooShort);
    }

    let interface_len = le16(buf, 0) as usize;
    if interface_len < RESPONSE_MIN_LEN || interface_len > buf.len() {
        return Err(SupplicantError::BadInterfaceLength);
    }
    let interface_type = le16(buf, 2);
    if interface_type != MWIFIEX_TYPE_CMD {
        return Err(SupplicantError::BadInterfaceType {
            got: interface_type,
        });
    }

    let host_cmd_len = le16(buf, 6) as usize;
    if host_cmd_len < S_DS_GEN || interface_len != INTF_HEADER_LEN + host_cmd_len {
        return Err(SupplicantError::BadHostCommandLength);
    }

    let command = le16(buf, 4);
    if command != (expected_cmd | HOST_CMD_RET_BIT) {
        return Err(SupplicantError::BadCommand { got: command });
    }
    let sequence = le16(buf, 8);
    if sequence != expected_seq {
        return Err(SupplicantError::BadSequence { got: sequence });
    }
    let result = le16(buf, 10);
    if result != HOST_CMD_RESULT_OK {
        return Err(SupplicantError::FirmwareResult { code: result });
    }

    Ok(())
}

fn write_command_header(out: &mut [u8], total_len: usize, cmd: u16, size: usize, seq: u16) {
    put_le16(out, 0, total_len as u16);
    put_le16(out, 2, MWIFIEX_TYPE_CMD);
    put_le16(out, 4, cmd);
    put_le16(out, 6, size as u16);
    put_le16(out, 8, seq);
    put_le16(out, 10, 0);
}

fn write_tlv_header(out: &mut [u8], offset: usize, ty: u16, len: u16) {
    put_le16(out, offset, ty);
    put_le16(out, offset + 2, len);
}

fn put_le16(out: &mut [u8], offset: usize, value: u16) {
    out[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn le16(buf: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([buf[offset], buf[offset + 1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    const BSSID: [u8; BSSID_LEN] = [0x02, 0x11, 0x22, 0x33, 0x44, 0x55];

    fn status_response(cmd: u16, seq: u16, result: u16) -> [u8; RESPONSE_MIN_LEN] {
        let mut response = [0u8; RESPONSE_MIN_LEN];
        put_le16(&mut response, 0, RESPONSE_MIN_LEN as u16);
        put_le16(&mut response, 2, MWIFIEX_TYPE_CMD);
        put_le16(&mut response, 4, cmd);
        put_le16(&mut response, 6, S_DS_GEN as u16);
        put_le16(&mut response, 8, seq);
        put_le16(&mut response, 10, result);
        response
    }

    #[test]
    fn profile_set_pins_nxp_wpa2_aes_packet() {
        let mut out = [0xa5; SUPPLICANT_PROFILE_SET_TOTAL_LEN + 4];

        let len = build_supplicant_profile_set(0x34, &mut out).unwrap();

        assert_eq!(len, 28);
        assert_eq!(
            &out[..len],
            &[
                0x1c, 0x00, 0x01, 0x00, 0xc5, 0x00, 0x18, 0x00, 0x34, 0x00, 0x00, 0x00, 0x01, 0x00,
                0x00, 0x00, 0x40, 0x01, 0x02, 0x00, 0x20, 0x00, 0x42, 0x01, 0x02, 0x00, 0x08, 0x08,
            ]
        );
        assert_eq!(&out[len..], &[0xa5; 4]);
    }

    #[test]
    fn profile_set_rejects_small_output_without_writing() {
        let mut out = [0xa5; SUPPLICANT_PROFILE_SET_TOTAL_LEN - 1];

        assert_eq!(
            build_supplicant_profile_set(0, &mut out),
            Err(SupplicantError::OutputBufferTooSmall)
        );
        assert!(out.iter().all(|byte| *byte == 0xa5));
    }

    #[test]
    fn pmk_set_pins_nxp_ssid_bssid_passphrase_packet() {
        let mut out = [0xa5; 50];

        let len = build_supplicant_pmk_set(0x34, BSSID, b"test", b"12345678", &mut out).unwrap();

        assert_eq!(len, 46);
        assert_eq!(
            &out[..len],
            &[
                0x2e, 0x00, 0x01, 0x00, 0xc4, 0x00, 0x2a, 0x00, 0x34, 0x00, 0x00, 0x00, 0x01, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x04, 0x00, b't', b'e', b's', b't', 0x23, 0x01, 0x06, 0x00,
                0x02, 0x11, 0x22, 0x33, 0x44, 0x55, 0x3c, 0x01, 0x08, 0x00, b'1', b'2', b'3', b'4',
                b'5', b'6', b'7', b'8',
            ]
        );
        assert_eq!(&out[len..], &[0xa5; 4]);
    }

    #[test]
    fn pmk_set_accepts_pinned_maximum_bounds() {
        let ssid = [b's'; MAX_SSID_LEN];
        let passphrase = [b'p'; MAX_PASSPHRASE_LEN];
        let mut out = [0u8; SUPPLICANT_PMK_SET_MAX_TOTAL_LEN];

        assert_eq!(
            build_supplicant_pmk_set(1, BSSID, &ssid, &passphrase, &mut out),
            Ok(SUPPLICANT_PMK_SET_MAX_TOTAL_LEN)
        );
        assert_eq!(le16(&out, 0) as usize, SUPPLICANT_PMK_SET_MAX_TOTAL_LEN);
        assert_eq!(
            le16(&out, 6) as usize,
            SUPPLICANT_PMK_SET_MAX_TOTAL_LEN - INTF_HEADER_LEN
        );
    }

    #[test]
    fn connection_supplicant_builders_reject_old_bss_one_sequence_context() {
        let mut out = [0u8; SUPPLICANT_PMK_SET_MAX_TOTAL_LEN];
        assert_eq!(
            build_supplicant_profile_set(0x0100, &mut out),
            Err(SupplicantError::InvalidSequenceContext { got: 0x0100 })
        );
        assert_eq!(
            build_supplicant_pmk_set(0x0100, BSSID, b"test", b"12345678", &mut out),
            Err(SupplicantError::InvalidSequenceContext { got: 0x0100 })
        );
    }

    #[test]
    fn pmk_set_rejects_ssid_bssid_and_output_bounds_without_writing() {
        let mut out = [0xa5; SUPPLICANT_PMK_SET_MAX_TOTAL_LEN];
        let long_ssid = [b's'; MAX_SSID_LEN + 1];

        assert_eq!(
            build_supplicant_pmk_set(0, BSSID, b"", b"12345678", &mut out),
            Err(SupplicantError::InvalidSsidLength)
        );
        assert_eq!(
            build_supplicant_pmk_set(0, BSSID, &long_ssid, b"12345678", &mut out),
            Err(SupplicantError::InvalidSsidLength)
        );
        assert_eq!(
            build_supplicant_pmk_set(0, [0; 6], b"s", b"12345678", &mut out),
            Err(SupplicantError::InvalidBssid)
        );
        assert_eq!(
            build_supplicant_pmk_set(0, [1, 2, 3, 4, 5, 6], b"s", b"12345678", &mut out),
            Err(SupplicantError::InvalidBssid)
        );
        assert!(out.iter().all(|byte| *byte == 0xa5));

        let mut short = [0xa5; 42];
        assert_eq!(
            build_supplicant_pmk_set(0, BSSID, b"s", b"12345678", &mut short),
            Err(SupplicantError::OutputBufferTooSmall)
        );
        assert!(short.iter().all(|byte| *byte == 0xa5));
    }

    #[test]
    fn pmk_set_rejects_invalid_passphrase_lengths_without_writing() {
        let short_passphrase = [b'p'; MIN_PASSPHRASE_LEN - 1];
        let long_passphrase = [b'p'; MAX_PASSPHRASE_LEN + 1];
        let mut out = [0xa5; SUPPLICANT_PMK_SET_MAX_TOTAL_LEN];

        assert_eq!(
            build_supplicant_pmk_set(0, BSSID, b"s", &short_passphrase, &mut out),
            Err(SupplicantError::InvalidPassphraseLength)
        );
        assert_eq!(
            build_supplicant_pmk_set(0, BSSID, b"s", &long_passphrase, &mut out),
            Err(SupplicantError::InvalidPassphraseLength)
        );
        assert!(out.iter().all(|byte| *byte == 0xa5));
    }

    #[test]
    fn response_parsers_accept_only_matching_success_status() {
        let profile = status_response(SUPPLICANT_PROFILE_CMD | HOST_CMD_RET_BIT, 7, 0);
        let pmk = status_response(SUPPLICANT_PMK_CMD | HOST_CMD_RET_BIT, 8, 0);

        assert_eq!(parse_supplicant_profile_response(7, &profile), Ok(()));
        assert_eq!(parse_supplicant_pmk_response(8, &pmk), Ok(()));

        let no_ret = status_response(SUPPLICANT_PROFILE_CMD, 7, 0);
        assert_eq!(
            parse_supplicant_profile_response(7, &no_ret),
            Err(SupplicantError::BadCommand {
                got: SUPPLICANT_PROFILE_CMD
            })
        );

        let wrong_seq = status_response(SUPPLICANT_PMK_CMD | HOST_CMD_RET_BIT, 9, 0);
        assert_eq!(
            parse_supplicant_pmk_response(8, &wrong_seq),
            Err(SupplicantError::BadSequence { got: 9 })
        );

        let fw_error = status_response(SUPPLICANT_PMK_CMD | HOST_CMD_RET_BIT, 8, 4);
        assert_eq!(
            parse_supplicant_pmk_response(8, &fw_error),
            Err(SupplicantError::FirmwareResult { code: 4 })
        );
    }

    #[test]
    fn response_parsers_reject_malformed_bounds_and_framing() {
        assert_eq!(
            parse_supplicant_pmk_response(0, &[0; RESPONSE_MIN_LEN - 1]),
            Err(SupplicantError::ResponseTooShort)
        );

        let mut bad_interface_len = status_response(SUPPLICANT_PMK_CMD | HOST_CMD_RET_BIT, 0, 0);
        put_le16(&mut bad_interface_len, 0, (RESPONSE_MIN_LEN + 1) as u16);
        assert_eq!(
            parse_supplicant_pmk_response(0, &bad_interface_len),
            Err(SupplicantError::BadInterfaceLength)
        );

        let mut bad_type = status_response(SUPPLICANT_PMK_CMD | HOST_CMD_RET_BIT, 0, 0);
        put_le16(&mut bad_type, 2, 2);
        assert_eq!(
            parse_supplicant_pmk_response(0, &bad_type),
            Err(SupplicantError::BadInterfaceType { got: 2 })
        );

        let mut bad_host_len = status_response(SUPPLICANT_PMK_CMD | HOST_CMD_RET_BIT, 0, 0);
        put_le16(&mut bad_host_len, 6, (S_DS_GEN + 1) as u16);
        assert_eq!(
            parse_supplicant_pmk_response(0, &bad_host_len),
            Err(SupplicantError::BadHostCommandLength)
        );
    }
}
