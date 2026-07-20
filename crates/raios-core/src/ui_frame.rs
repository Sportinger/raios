//! Decoder for the bounded personal-shell display-list ABI.
//!
//! Validation is atomic: callers receive commands only after the complete frame
//! has passed every structural, UTF-8, limit, padding, and clipping check.

extern crate alloc;

use alloc::vec::Vec;
use core::{cmp, str};

pub const FRAME_ABI_VERSION: u16 = 1;
pub const FRAME_HEADER_LEN: usize = 16;
pub const COMMAND_PREFIX_LEN: usize = 4;
pub const MAX_FRAME_BYTES: usize = 16 * 1024;
pub const MAX_COMMANDS: usize = 256;
pub const MAX_TOTAL_TEXT_BYTES: usize = 4 * 1024;
pub const MAX_TEXT_RUN_BYTES: usize = 512;

pub const FRAME_RESULT_ACCEPTED: i32 = 0;
pub const FRAME_RESULT_WRONG_ABI_VERSION: i32 = -1;
pub const FRAME_RESULT_MALFORMED: i32 = -2;
pub const FRAME_RESULT_LIMIT_EXCEEDED: i32 = -3;
pub const FRAME_RESULT_SECOND_SUBMIT: i32 = -4;

const FRAME_MAGIC: &[u8; 4] = b"RFRM";
const OPCODE_CLEAR: u8 = 1;
const OPCODE_FILL_RECT: u8 = 2;
const OPCODE_STROKE_RECT: u8 = 3;
const OPCODE_TEXT: u8 = 4;
const OPCODE_FOCUS_HINT: u8 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Viewport {
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command<'a> {
    Clear {
        rgba: u32,
    },
    FillRect {
        rect: Rect,
        rgba: u32,
    },
    StrokeRect {
        rect: Rect,
        rgba: u32,
    },
    Text {
        x: u16,
        y: u16,
        rgba: u32,
        text: &'a str,
    },
    FocusHint {
        rect: Rect,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameError {
    WrongAbiVersion,
    Malformed,
    LimitExceeded,
}

impl FrameError {
    pub const fn result_code(self) -> i32 {
        match self {
            Self::WrongAbiVersion => FRAME_RESULT_WRONG_ABI_VERSION,
            Self::Malformed => FRAME_RESULT_MALFORMED,
            Self::LimitExceeded => FRAME_RESULT_LIMIT_EXCEEDED,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ValidatedFrame<'a> {
    commands: Vec<Command<'a>>,
    total_text_bytes: usize,
}

impl<'a> ValidatedFrame<'a> {
    pub fn commands(&self) -> &[Command<'a>] {
        &self.commands
    }

    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    pub fn total_text_bytes(&self) -> usize {
        self.total_text_bytes
    }
}

pub fn validate_frame(bytes: &[u8], viewport: Viewport) -> Result<ValidatedFrame<'_>, FrameError> {
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(FrameError::LimitExceeded);
    }
    if bytes.len() < FRAME_HEADER_LEN || viewport.width == 0 || viewport.height == 0 {
        return Err(FrameError::Malformed);
    }
    if bytes.get(..4) != Some(FRAME_MAGIC) {
        return Err(FrameError::Malformed);
    }
    if read_u16(bytes, 4) != Some(FRAME_ABI_VERSION) {
        return Err(FrameError::WrongAbiVersion);
    }
    if read_u16(bytes, 6) != Some(FRAME_HEADER_LEN as u16) || read_u16(bytes, 14) != Some(0) {
        return Err(FrameError::Malformed);
    }

    let total_len = read_u32(bytes, 8).ok_or(FrameError::Malformed)? as usize;
    if total_len > MAX_FRAME_BYTES {
        return Err(FrameError::LimitExceeded);
    }
    if total_len != bytes.len() {
        return Err(FrameError::Malformed);
    }
    let command_count = read_u16(bytes, 12).ok_or(FrameError::Malformed)? as usize;
    if command_count > MAX_COMMANDS {
        return Err(FrameError::LimitExceeded);
    }

    let mut commands = Vec::with_capacity(command_count);
    let mut total_text_bytes = 0usize;
    let mut cursor = FRAME_HEADER_LEN;
    while commands.len() < command_count {
        let prefix_end = cursor
            .checked_add(COMMAND_PREFIX_LEN)
            .ok_or(FrameError::Malformed)?;
        let prefix = bytes.get(cursor..prefix_end).ok_or(FrameError::Malformed)?;
        if prefix[1] != 0 {
            return Err(FrameError::Malformed);
        }

        let payload_len = u16::from_le_bytes([prefix[2], prefix[3]]) as usize;
        let payload_end = prefix_end
            .checked_add(payload_len)
            .ok_or(FrameError::Malformed)?;
        let padded_len = payload_len.checked_add(3).ok_or(FrameError::Malformed)? & !3;
        let command_end = prefix_end
            .checked_add(padded_len)
            .ok_or(FrameError::Malformed)?;
        let payload = bytes
            .get(prefix_end..payload_end)
            .ok_or(FrameError::Malformed)?;
        let padding = bytes
            .get(payload_end..command_end)
            .ok_or(FrameError::Malformed)?;
        if padding.iter().any(|byte| *byte != 0) {
            return Err(FrameError::Malformed);
        }

        commands.push(parse_command(
            prefix[0],
            payload,
            viewport,
            &mut total_text_bytes,
        )?);
        cursor = command_end;
    }
    if cursor != total_len {
        return Err(FrameError::Malformed);
    }

    Ok(ValidatedFrame {
        commands,
        total_text_bytes,
    })
}

fn parse_command<'a>(
    opcode: u8,
    payload: &'a [u8],
    viewport: Viewport,
    total_text_bytes: &mut usize,
) -> Result<Command<'a>, FrameError> {
    match opcode {
        OPCODE_CLEAR if payload.len() == 4 => Ok(Command::Clear {
            rgba: read_u32(payload, 0).ok_or(FrameError::Malformed)?,
        }),
        OPCODE_FILL_RECT if payload.len() == 12 => Ok(Command::FillRect {
            rect: parse_rect(payload, viewport)?,
            rgba: read_u32(payload, 8).ok_or(FrameError::Malformed)?,
        }),
        OPCODE_STROKE_RECT if payload.len() == 12 => Ok(Command::StrokeRect {
            rect: parse_rect(payload, viewport)?,
            rgba: read_u32(payload, 8).ok_or(FrameError::Malformed)?,
        }),
        OPCODE_TEXT if payload.len() >= 12 => {
            let text_len = read_u16(payload, 8).ok_or(FrameError::Malformed)? as usize;
            if read_u16(payload, 10) != Some(0) {
                return Err(FrameError::Malformed);
            }
            if text_len > MAX_TEXT_RUN_BYTES {
                return Err(FrameError::LimitExceeded);
            }
            let expected_len = 12usize.checked_add(text_len).ok_or(FrameError::Malformed)?;
            if payload.len() != expected_len {
                return Err(FrameError::Malformed);
            }
            let next_total = total_text_bytes
                .checked_add(text_len)
                .ok_or(FrameError::LimitExceeded)?;
            if next_total > MAX_TOTAL_TEXT_BYTES {
                return Err(FrameError::LimitExceeded);
            }
            let text = str::from_utf8(&payload[12..]).map_err(|_| FrameError::Malformed)?;
            *total_text_bytes = next_total;
            Ok(Command::Text {
                x: clip_point(
                    read_u16(payload, 0).ok_or(FrameError::Malformed)?,
                    viewport.width,
                ),
                y: clip_point(
                    read_u16(payload, 2).ok_or(FrameError::Malformed)?,
                    viewport.height,
                ),
                rgba: read_u32(payload, 4).ok_or(FrameError::Malformed)?,
                text,
            })
        }
        OPCODE_FOCUS_HINT if payload.len() == 8 => Ok(Command::FocusHint {
            rect: parse_rect(payload, viewport)?,
        }),
        _ => Err(FrameError::Malformed),
    }
}

fn parse_rect(payload: &[u8], viewport: Viewport) -> Result<Rect, FrameError> {
    let x = read_u16(payload, 0).ok_or(FrameError::Malformed)?;
    let y = read_u16(payload, 2).ok_or(FrameError::Malformed)?;
    let width = read_u16(payload, 4).ok_or(FrameError::Malformed)?;
    let height = read_u16(payload, 6).ok_or(FrameError::Malformed)?;
    if width == 0 || height == 0 {
        return Err(FrameError::Malformed);
    }
    Ok(clip_rect(x, y, width, height, viewport))
}

fn clip_rect(x: u16, y: u16, width: u16, height: u16, viewport: Viewport) -> Rect {
    let x = usize::from(x);
    let y = usize::from(y);
    let right = x.checked_add(usize::from(width)).unwrap_or(usize::MAX);
    let bottom = y.checked_add(usize::from(height)).unwrap_or(usize::MAX);
    let viewport_width = usize::from(viewport.width);
    let viewport_height = usize::from(viewport.height);
    let clipped_x = cmp::min(x, viewport_width);
    let clipped_y = cmp::min(y, viewport_height);
    let clipped_right = cmp::min(right, viewport_width);
    let clipped_bottom = cmp::min(bottom, viewport_height);

    Rect {
        x: cmp::min(clipped_x, viewport_width - 1) as u16,
        y: cmp::min(clipped_y, viewport_height - 1) as u16,
        width: clipped_right.saturating_sub(clipped_x) as u16,
        height: clipped_bottom.saturating_sub(clipped_y) as u16,
    }
}

fn clip_point(value: u16, limit: u16) -> u16 {
    cmp::min(value, limit - 1)
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let end = offset.checked_add(2)?;
    let raw = bytes.get(offset..end)?;
    Some(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let raw = bytes.get(offset..end)?;
    Some(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{string::String, vec, vec::Vec};

    const VIEWPORT: Viewport = Viewport {
        width: 100,
        height: 50,
    };

    fn command(opcode: u8, payload: &[u8]) -> Vec<u8> {
        let mut out = vec![opcode, 0];
        out.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        out.extend_from_slice(payload);
        while out.len() % 4 != 0 {
            out.push(0);
        }
        out
    }

    fn frame(commands: &[Vec<u8>]) -> Vec<u8> {
        let mut out = Vec::from(&b"RFRM\x01\0\x10\0\0\0\0\0\0\0\0\0"[..]);
        for command in commands {
            out.extend_from_slice(command);
        }
        let len = out.len() as u32;
        out[8..12].copy_from_slice(&len.to_le_bytes());
        out[12..14].copy_from_slice(&(commands.len() as u16).to_le_bytes());
        out
    }

    fn rect_payload(x: u16, y: u16, width: u16, height: u16, rgba: u32) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&x.to_le_bytes());
        out.extend_from_slice(&y.to_le_bytes());
        out.extend_from_slice(&width.to_le_bytes());
        out.extend_from_slice(&height.to_le_bytes());
        out.extend_from_slice(&rgba.to_le_bytes());
        out
    }

    fn text_payload(x: u16, y: u16, text: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&x.to_le_bytes());
        out.extend_from_slice(&y.to_le_bytes());
        out.extend_from_slice(&0x1122_3344u32.to_le_bytes());
        out.extend_from_slice(&(text.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(text);
        out
    }

    #[test]
    fn validates_every_opcode_and_clips_to_personal_viewport() {
        let clear = command(OPCODE_CLEAR, &0xaabb_ccddu32.to_le_bytes());
        let fill = command(OPCODE_FILL_RECT, &rect_payload(90, 45, 20, 10, 0x0102_0304));
        let stroke = command(OPCODE_STROKE_RECT, &rect_payload(2, 3, 4, 5, 0x0506_0708));
        let text = command(OPCODE_TEXT, &text_payload(500, 300, b"hello"));
        let focus = command(OPCODE_FOCUS_HINT, &rect_payload(99, 49, 10, 10, 0)[..8]);
        let bytes = frame(&[clear, fill, stroke, text, focus]);

        let validated = validate_frame(&bytes, VIEWPORT).unwrap();

        assert_eq!(validated.command_count(), 5);
        assert_eq!(validated.total_text_bytes(), 5);
        assert_eq!(
            validated.commands(),
            &[
                Command::Clear { rgba: 0xaabb_ccdd },
                Command::FillRect {
                    rect: Rect {
                        x: 90,
                        y: 45,
                        width: 10,
                        height: 5,
                    },
                    rgba: 0x0102_0304,
                },
                Command::StrokeRect {
                    rect: Rect {
                        x: 2,
                        y: 3,
                        width: 4,
                        height: 5,
                    },
                    rgba: 0x0506_0708,
                },
                Command::Text {
                    x: 99,
                    y: 49,
                    rgba: 0x1122_3344,
                    text: "hello",
                },
                Command::FocusHint {
                    rect: Rect {
                        x: 99,
                        y: 49,
                        width: 1,
                        height: 1,
                    },
                },
            ]
        );
    }

    #[test]
    fn result_codes_match_the_frozen_abi() {
        assert_eq!(FrameError::WrongAbiVersion.result_code(), -1);
        assert_eq!(FrameError::Malformed.result_code(), -2);
        assert_eq!(FrameError::LimitExceeded.result_code(), -3);
        assert_eq!(FRAME_RESULT_ACCEPTED, 0);
        assert_eq!(FRAME_RESULT_SECOND_SUBMIT, -4);
    }

    #[test]
    fn rejects_wrong_version_and_structural_mutations_atomically() {
        let valid = frame(&[command(OPCODE_CLEAR, &0u32.to_le_bytes())]);

        let mut wrong_version = valid.clone();
        wrong_version[4] = 2;
        assert_eq!(
            validate_frame(&wrong_version, VIEWPORT),
            Err(FrameError::WrongAbiVersion)
        );

        let mut cases = Vec::new();
        let mut bad_magic = valid.clone();
        bad_magic[0] = b'X';
        cases.push(bad_magic);
        let mut bad_header_len = valid.clone();
        bad_header_len[6] = 15;
        cases.push(bad_header_len);
        let mut bad_header_flags = valid.clone();
        bad_header_flags[14] = 1;
        cases.push(bad_header_flags);
        let mut bad_command_flags = valid.clone();
        bad_command_flags[17] = 1;
        cases.push(bad_command_flags);
        let mut unknown_opcode = valid.clone();
        unknown_opcode[16] = 99;
        cases.push(unknown_opcode);
        let mut trailing = valid.clone();
        trailing.push(0);
        cases.push(trailing);
        cases.push(valid[..valid.len() - 1].to_vec());

        for bytes in cases {
            assert_eq!(validate_frame(&bytes, VIEWPORT), Err(FrameError::Malformed));
        }
        assert_eq!(
            validate_frame(
                &valid,
                Viewport {
                    width: 0,
                    height: 1
                }
            ),
            Err(FrameError::Malformed)
        );
    }

    #[test]
    fn rejects_nonzero_padding_invalid_utf8_and_impossible_rectangles() {
        let mut padded = frame(&[command(OPCODE_TEXT, &text_payload(0, 0, b"x"))]);
        *padded.last_mut().unwrap() = 1;
        assert_eq!(
            validate_frame(&padded, VIEWPORT),
            Err(FrameError::Malformed)
        );

        let bad_utf8 = frame(&[command(OPCODE_TEXT, &text_payload(0, 0, &[0xff]))]);
        assert_eq!(
            validate_frame(&bad_utf8, VIEWPORT),
            Err(FrameError::Malformed)
        );

        let mut reserved_text = text_payload(0, 0, b"x");
        reserved_text[10] = 1;
        assert_eq!(
            validate_frame(&frame(&[command(OPCODE_TEXT, &reserved_text)]), VIEWPORT),
            Err(FrameError::Malformed)
        );

        assert_eq!(
            validate_frame(&frame(&[command(OPCODE_CLEAR, &[0; 3])]), VIEWPORT),
            Err(FrameError::Malformed)
        );

        let zero_width = frame(&[command(OPCODE_FILL_RECT, &rect_payload(0, 0, 0, 1, 0))]);
        assert_eq!(
            validate_frame(&zero_width, VIEWPORT),
            Err(FrameError::Malformed)
        );
    }

    #[test]
    fn enforces_frame_command_and_text_limits() {
        let mut oversized = vec![0u8; MAX_FRAME_BYTES + 1];
        oversized[..4].copy_from_slice(FRAME_MAGIC);
        assert_eq!(
            validate_frame(&oversized, VIEWPORT),
            Err(FrameError::LimitExceeded)
        );

        let mut too_many_commands = frame(&[]);
        too_many_commands[12..14].copy_from_slice(&((MAX_COMMANDS + 1) as u16).to_le_bytes());
        assert_eq!(
            validate_frame(&too_many_commands, VIEWPORT),
            Err(FrameError::LimitExceeded)
        );

        let long_run = vec![b'x'; MAX_TEXT_RUN_BYTES + 1];
        let long_run_frame = frame(&[command(OPCODE_TEXT, &text_payload(0, 0, &long_run))]);
        assert_eq!(
            validate_frame(&long_run_frame, VIEWPORT),
            Err(FrameError::LimitExceeded)
        );

        let run = vec![b'x'; MAX_TEXT_RUN_BYTES];
        let mut commands = Vec::new();
        for _ in 0..(MAX_TOTAL_TEXT_BYTES / MAX_TEXT_RUN_BYTES) {
            commands.push(command(OPCODE_TEXT, &text_payload(0, 0, &run)));
        }
        commands.push(command(OPCODE_TEXT, &text_payload(0, 0, b"x")));
        let too_much_text = frame(&commands);
        assert_eq!(
            validate_frame(&too_much_text, VIEWPORT),
            Err(FrameError::LimitExceeded)
        );
    }

    #[test]
    fn accepts_exact_hard_limits() {
        let run = vec![b'x'; MAX_TEXT_RUN_BYTES];
        let mut commands = Vec::new();
        for _ in 0..(MAX_TOTAL_TEXT_BYTES / MAX_TEXT_RUN_BYTES) {
            commands.push(command(OPCODE_TEXT, &text_payload(0, 0, &run)));
        }
        while commands.len() < MAX_COMMANDS {
            commands.push(command(OPCODE_CLEAR, &0u32.to_le_bytes()));
        }
        let bytes = frame(&commands);
        assert!(bytes.len() <= MAX_FRAME_BYTES);

        let validated = validate_frame(&bytes, VIEWPORT).unwrap();
        assert_eq!(validated.command_count(), MAX_COMMANDS);
        assert_eq!(validated.total_text_bytes(), MAX_TOTAL_TEXT_BYTES);
    }

    #[test]
    fn accepts_utf8_text_and_fully_clips_offscreen_rectangles() {
        let text = String::from("raiOS »");
        let bytes = frame(&[
            command(OPCODE_TEXT, &text_payload(0, 0, text.as_bytes())),
            command(OPCODE_FILL_RECT, &rect_payload(500, 500, 2, 2, 0)),
        ]);

        let validated = validate_frame(&bytes, VIEWPORT).unwrap();
        assert_eq!(
            validated.commands()[0],
            Command::Text {
                x: 0,
                y: 0,
                rgba: 0x1122_3344,
                text: "raiOS »",
            }
        );
        assert_eq!(
            validated.commands()[1],
            Command::FillRect {
                rect: Rect {
                    x: 99,
                    y: 49,
                    width: 0,
                    height: 0,
                },
                rgba: 0,
            }
        );
    }
}
