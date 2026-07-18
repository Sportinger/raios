//! Small no-alloc 802.11 management-frame scan parser.
//!
//! Parses only Beacon and Probe Response frames. RSSI is intentionally absent:
//! it comes from the radio RX descriptor, not the 802.11 frame body.

pub const DOT11_SSID_CAPACITY: usize = 32;

const MANAGEMENT_FRAME_TYPE: u16 = 0;
const BEACON_SUBTYPE: u16 = 8;
const PROBE_RESPONSE_SUBTYPE: u16 = 5;
const MANAGEMENT_HEADER_LEN: usize = 24;
const FIXED_BEACON_PARAMS_LEN: usize = 12;
const TAGGED_PARAMS_OFFSET: usize = MANAGEMENT_HEADER_LEN + FIXED_BEACON_PARAMS_LEN;
const CAPABILITY_PRIVACY: u16 = 1 << 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dot11FrameKind {
    Beacon,
    ProbeResponse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dot11Security {
    Open,
    Wep,
    Wpa,
    Wpa2,
    Wpa3,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dot11ScanError {
    FrameTooShort,
    UnsupportedFrame,
    MissingSsid,
    SsidTooLong,
    TruncatedElement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Dot11Ssid {
    bytes: [u8; DOT11_SSID_CAPACITY],
    len: usize,
    hidden: bool,
}

impl Dot11Ssid {
    pub const fn empty_hidden() -> Self {
        Self {
            bytes: [0; DOT11_SSID_CAPACITY],
            len: 0,
            hidden: true,
        }
    }

    pub fn from_ie(bytes: &[u8]) -> Result<Self, Dot11ScanError> {
        if bytes.len() > DOT11_SSID_CAPACITY {
            return Err(Dot11ScanError::SsidTooLong);
        }

        let hidden = bytes.is_empty() || bytes.iter().all(|byte| *byte == 0);
        let mut ssid = Self {
            bytes: [0; DOT11_SSID_CAPACITY],
            len: if hidden { 0 } else { bytes.len() },
            hidden,
        };
        if !hidden {
            ssid.bytes[..bytes.len()].copy_from_slice(bytes);
        }
        Ok(ssid)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_hidden(&self) -> bool {
        self.hidden
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Dot11ScanNetwork {
    pub frame_kind: Dot11FrameKind,
    pub ssid: Dot11Ssid,
    pub channel: Option<u8>,
    pub security: Dot11Security,
}

pub fn parse_scan_frame(frame: &[u8]) -> Result<Dot11ScanNetwork, Dot11ScanError> {
    if frame.len() < TAGGED_PARAMS_OFFSET {
        return Err(Dot11ScanError::FrameTooShort);
    }

    let frame_control = u16::from_le_bytes([frame[0], frame[1]]);
    let frame_type = (frame_control >> 2) & 0x3;
    let subtype = (frame_control >> 4) & 0xf;
    if frame_type != MANAGEMENT_FRAME_TYPE {
        return Err(Dot11ScanError::UnsupportedFrame);
    }
    let frame_kind = match subtype {
        BEACON_SUBTYPE => Dot11FrameKind::Beacon,
        PROBE_RESPONSE_SUBTYPE => Dot11FrameKind::ProbeResponse,
        _ => return Err(Dot11ScanError::UnsupportedFrame),
    };

    let capabilities = u16::from_le_bytes([frame[34], frame[35]]);
    let privacy = capabilities & CAPABILITY_PRIVACY != 0;
    let mut ssid = None;
    let mut channel = None;
    let mut rsn_security = None;
    let mut has_wpa_vendor_ie = false;

    let mut pos = TAGGED_PARAMS_OFFSET;
    while pos < frame.len() {
        if frame.len() - pos < 2 {
            return Err(Dot11ScanError::TruncatedElement);
        }
        let tag = frame[pos];
        let len = frame[pos + 1] as usize;
        pos += 2;
        if len > frame.len() - pos {
            return Err(Dot11ScanError::TruncatedElement);
        }
        let body = &frame[pos..pos + len];
        match tag {
            0 => ssid = Some(Dot11Ssid::from_ie(body)?),
            3 if len == 1 => channel = Some(body[0]),
            48 => rsn_security = Some(parse_rsn_security(body)),
            221 if is_vendor_wpa_ie(body) => has_wpa_vendor_ie = true,
            _ => {}
        }
        pos += len;
    }

    let Some(ssid) = ssid else {
        return Err(Dot11ScanError::MissingSsid);
    };
    let security = match rsn_security {
        Some(Dot11Security::Wpa3) => Dot11Security::Wpa3,
        Some(Dot11Security::Wpa2) => Dot11Security::Wpa2,
        Some(_) => Dot11Security::Unknown,
        None if has_wpa_vendor_ie => Dot11Security::Wpa,
        None if privacy => Dot11Security::Wep,
        None => Dot11Security::Open,
    };

    Ok(Dot11ScanNetwork {
        frame_kind,
        ssid,
        channel,
        security,
    })
}

fn parse_rsn_security(body: &[u8]) -> Dot11Security {
    if body.len() < 8 || u16::from_le_bytes([body[0], body[1]]) != 1 {
        return Dot11Security::Unknown;
    }

    let pairwise_count_offset = 6;
    let pairwise_count = u16::from_le_bytes([body[pairwise_count_offset], body[7]]) as usize;
    let pairwise_list_len = match pairwise_count.checked_mul(4) {
        Some(len) => len,
        None => return Dot11Security::Unknown,
    };
    let akm_count_offset = match 8usize.checked_add(pairwise_list_len) {
        Some(offset) => offset,
        None => return Dot11Security::Unknown,
    };
    if body.len() < akm_count_offset + 2 {
        return Dot11Security::Unknown;
    }
    let akm_count =
        u16::from_le_bytes([body[akm_count_offset], body[akm_count_offset + 1]]) as usize;
    let akm_list_offset = akm_count_offset + 2;
    let akm_list_len = match akm_count.checked_mul(4) {
        Some(len) => len,
        None => return Dot11Security::Unknown,
    };
    if body.len() < akm_list_offset + akm_list_len {
        return Dot11Security::Unknown;
    }

    let mut idx = 0usize;
    while idx < akm_count {
        let offset = akm_list_offset + idx * 4;
        if body[offset..offset + 3] == [0x00, 0x0f, 0xac] && matches!(body[offset + 3], 8 | 9) {
            return Dot11Security::Wpa3;
        }
        idx += 1;
    }

    Dot11Security::Wpa2
}

fn is_vendor_wpa_ie(body: &[u8]) -> bool {
    body.len() >= 4 && body[0..4] == [0x00, 0x50, 0xf2, 0x01]
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPEN_BEACON: &[u8] = &[
        0x80, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x10, 0x22, 0x33, 0x44, 0x55,
        0x66, 0x10, 0x22, 0x33, 0x44, 0x55, 0x66, 0x10, 0x00, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33,
        0x22, 0x11, 0x64, 0x00, 0x01, 0x00, 0x00, 0x07, b'R', b'a', b'i', b'O', b'p', b'e', b'n',
        0x01, 0x08, 0x82, 0x84, 0x8b, 0x96, 0x24, 0x30, 0x48, 0x6c, 0x03, 0x01, 0x06,
    ];

    const WPA2_BEACON: &[u8] = &[
        0x80, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x20, 0x22, 0x33, 0x44, 0x55,
        0x66, 0x20, 0x22, 0x33, 0x44, 0x55, 0x66, 0x20, 0x00, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03,
        0x02, 0x01, 0x64, 0x00, 0x11, 0x00, 0x00, 0x07, b'R', b'a', b'i', b'W', b'P', b'A', b'2',
        0x03, 0x01, 0x0b, 0x30, 0x14, 0x01, 0x00, 0x00, 0x0f, 0xac, 0x04, 0x01, 0x00, 0x00, 0x0f,
        0xac, 0x04, 0x01, 0x00, 0x00, 0x0f, 0xac, 0x02, 0x00, 0x00,
    ];

    const HIDDEN_WPA3_PROBE_RESPONSE: &[u8] = &[
        0x50, 0x00, 0x00, 0x00, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x30, 0x22, 0x33, 0x44, 0x55,
        0x66, 0x30, 0x22, 0x33, 0x44, 0x55, 0x66, 0x30, 0x00, 0x18, 0x17, 0x16, 0x15, 0x14, 0x13,
        0x12, 0x11, 0x64, 0x00, 0x11, 0x00, 0x00, 0x00, 0x03, 0x01, 0x01, 0x30, 0x14, 0x01, 0x00,
        0x00, 0x0f, 0xac, 0x04, 0x01, 0x00, 0x00, 0x0f, 0xac, 0x04, 0x01, 0x00, 0x00, 0x0f, 0xac,
        0x08, 0x00, 0x00,
    ];

    const VENDOR_WPA_BEACON: &[u8] = &[
        0x80, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x40, 0x22, 0x33, 0x44, 0x55,
        0x66, 0x40, 0x22, 0x33, 0x44, 0x55, 0x66, 0x40, 0x00, 0x28, 0x27, 0x26, 0x25, 0x24, 0x23,
        0x22, 0x21, 0x64, 0x00, 0x11, 0x00, 0x00, 0x06, b'R', b'a', b'i', b'W', b'P', b'A', 0x03,
        0x01, 0x03, 0xdd, 0x16, 0x00, 0x50, 0xf2, 0x01, 0x01, 0x00, 0x00, 0x50, 0xf2, 0x02, 0x01,
        0x00, 0x00, 0x50, 0xf2, 0x02, 0x01, 0x00, 0x00, 0x50, 0xf2, 0x02,
    ];

    #[test]
    fn parses_open_beacon_ssid_channel_security() {
        let parsed = parse_scan_frame(OPEN_BEACON).unwrap();

        assert_eq!(parsed.frame_kind, Dot11FrameKind::Beacon);
        assert_eq!(parsed.ssid.as_bytes(), b"RaiOpen");
        assert!(!parsed.ssid.is_hidden());
        assert_eq!(parsed.channel, Some(6));
        assert_eq!(parsed.security, Dot11Security::Open);
    }

    #[test]
    fn parses_wpa2_rsn_beacon() {
        let parsed = parse_scan_frame(WPA2_BEACON).unwrap();

        assert_eq!(parsed.ssid.as_bytes(), b"RaiWPA2");
        assert_eq!(parsed.channel, Some(11));
        assert_eq!(parsed.security, Dot11Security::Wpa2);
    }

    #[test]
    fn parses_hidden_wpa3_probe_response() {
        let parsed = parse_scan_frame(HIDDEN_WPA3_PROBE_RESPONSE).unwrap();

        assert_eq!(parsed.frame_kind, Dot11FrameKind::ProbeResponse);
        assert!(parsed.ssid.is_hidden());
        assert_eq!(parsed.ssid.len(), 0);
        assert_eq!(parsed.channel, Some(1));
        assert_eq!(parsed.security, Dot11Security::Wpa3);
    }

    #[test]
    fn parses_vendor_wpa_ie() {
        let parsed = parse_scan_frame(VENDOR_WPA_BEACON).unwrap();

        assert_eq!(parsed.ssid.as_bytes(), b"RaiWPA");
        assert_eq!(parsed.channel, Some(3));
        assert_eq!(parsed.security, Dot11Security::Wpa);
    }

    #[test]
    fn rejects_truncated_ie_without_panic() {
        let mut frame = [0u8; 40];
        frame[0] = 0x80;
        frame[32] = 0x64;
        frame[36] = 0;
        frame[37] = 4;
        frame[38] = b'a';

        assert_eq!(
            parse_scan_frame(&frame),
            Err(Dot11ScanError::TruncatedElement)
        );
    }
}
