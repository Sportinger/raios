//! Renderer for the bounded `RUIP` program packet staged by raiOS.
//!
//! State transitions stay core-owned. This signed guest only turns validated
//! program data plus its current numeric state into one bounded RFRM display list.

const APP_HEADER_LEN: usize = 32;
const PROGRAM_HEADER_LEN: usize = 32;
const WIDGET_PREFIX_LEN: usize = 16;
const FRAME_HEADER_LEN: usize = 16;

pub(crate) fn build_frame(
    context: &[u8],
    input: &[u8],
    width: u16,
    height: u16,
    out: &mut [u8],
) -> Option<usize> {
    if context.len() < APP_HEADER_LEN
        || context.get(..4)? != b"RAPP"
        || read_u16(context, 4)? != 1
        || read_u16(context, 6)? as usize != APP_HEADER_LEN
        || read_u32(context, 8)? as usize != context.len()
        || read_u32(context, 12)? != read_u32(input, 8)?
        || read_u16(context, 16)? != width
        || read_u16(context, 18)? != height
        || read_u32(context, 28)? != 0
    {
        return None;
    }
    let state_count = read_u16(context, 20)? as usize;
    let program_len = read_u32(context, 24)? as usize;
    if state_count == 0 || state_count > 16 || program_len > 16 * 1024 {
        return None;
    }
    let state_end = APP_HEADER_LEN.checked_add(state_count.checked_mul(8)?)?;
    let text_state_len = read_u16(context, 22)? as usize;
    let program_start = state_end.checked_add(text_state_len)?;
    if program_start.checked_add(program_len)? != context.len() {
        return None;
    }
    let state = context.get(APP_HEADER_LEN..state_end)?;
    let text_state = context.get(state_end..program_start)?;
    let program = context.get(program_start..)?;
    if program.len() < PROGRAM_HEADER_LEN
        || program.get(..4)? != b"RUIP"
        || read_u16(program, 4)? != 1
        || read_u16(program, 6)? as usize != PROGRAM_HEADER_LEN
        || read_u32(program, 8)? as usize != program.len()
        || program[12] as usize != state_count
        || program[17..PROGRAM_HEADER_LEN]
            .iter()
            .any(|byte| *byte != 0)
    {
        return None;
    }

    let widget_count = program[13] as usize;
    let text_slot_count = program[16] as usize;
    if widget_count > 64 || text_slot_count > 1 || (text_state_len == 0) != (text_slot_count == 0) {
        return None;
    }
    let mut program_cursor = PROGRAM_HEADER_LEN.checked_add(state_count.checked_mul(8)?)?;
    let text_capacity = if text_slot_count == 0 {
        0
    } else {
        let slot = program.get(program_cursor..program_cursor.checked_add(4)?)?;
        let capacity = read_u16(slot, 0)? as usize;
        if capacity == 0 || capacity > 2048 || read_u16(slot, 2)? != 0 {
            return None;
        }
        program_cursor += 4;
        capacity
    };
    let text = if text_slot_count == 0 {
        &[][..]
    } else {
        let text_len = read_u16(text_state, 0)? as usize;
        if text_state_len != 2usize.checked_add(text_len)? || text_len > text_capacity {
            return None;
        }
        text_state.get(2..)?
    };
    let mut frame_cursor = FRAME_HEADER_LEN;
    let mut command_count = 0u16;
    push_command(out, &mut frame_cursor, 1, &0x14161affu32.to_le_bytes())?;
    command_count += 1;

    let mut edit_box_seen = false;
    for _ in 0..widget_count {
        let prefix = program.get(program_cursor..program_cursor.checked_add(WIDGET_PREFIX_LEN)?)?;
        if prefix[1] != 0 || read_u16(prefix, 14)? != 0 {
            return None;
        }
        program_cursor += WIDGET_PREFIX_LEN;
        let text_len = read_u16(prefix, 2)? as usize;
        if text_len > 64 {
            return None;
        }
        let widget_text = program.get(program_cursor..program_cursor.checked_add(text_len)?)?;
        program_cursor += text_len;
        let padding = (4 - text_len % 4) % 4;
        if program
            .get(program_cursor..program_cursor.checked_add(padding)?)?
            .iter()
            .any(|byte| *byte != 0)
        {
            return None;
        }
        program_cursor += padding;

        let rect = [
            read_u16(prefix, 4)?,
            read_u16(prefix, 6)?,
            read_u16(prefix, 8)?,
            read_u16(prefix, 10)?,
        ];
        if rect[2] == 0 || rect[3] == 0 {
            return None;
        }
        match prefix[0] {
            1 if read_u16(prefix, 12)? == 0 => {
                push_text(
                    out,
                    &mut frame_cursor,
                    rect[0].saturating_add(8),
                    rect[1].saturating_add(8),
                    0xf4f7faff,
                    widget_text,
                )?;
                command_count += 1;
            }
            2 if text_len == 0 => {
                let slot = read_u16(prefix, 12)? as usize;
                if slot >= state_count {
                    return None;
                }
                push_rect(out, &mut frame_cursor, 2, rect, 0x272b33ff)?;
                push_rect(out, &mut frame_cursor, 3, rect, 0x3e434cff)?;
                let mut digits = [0u8; 20];
                let value = read_i64(state, slot.checked_mul(8)?)?;
                let digits = decimal(value, &mut digits);
                push_text(
                    out,
                    &mut frame_cursor,
                    rect[0].saturating_add(12),
                    rect[1].saturating_add(18),
                    0xf4f7faff,
                    digits,
                )?;
                command_count += 3;
            }
            3 if read_u16(prefix, 12)? != 0 => {
                push_rect(out, &mut frame_cursor, 2, rect, 0x272b33ff)?;
                push_rect(out, &mut frame_cursor, 3, rect, 0x4897f2ff)?;
                push_text(
                    out,
                    &mut frame_cursor,
                    rect[0].saturating_add(12),
                    rect[1].saturating_add(18),
                    0xf4f7faff,
                    widget_text,
                )?;
                command_count += 3;
            }
            4 if text_len == 0 => {
                let slot = read_u16(prefix, 12)? as usize;
                if edit_box_seen || slot >= text_slot_count {
                    return None;
                }
                edit_box_seen = true;
                render_edit_box(out, &mut frame_cursor, &mut command_count, rect, text)?;
            }
            _ => return None,
        }
    }
    if command_count > 256 || frame_cursor > 16 * 1024 {
        return None;
    }
    out[..4].copy_from_slice(b"RFRM");
    write_u16(out, 4, 1);
    write_u16(out, 6, FRAME_HEADER_LEN as u16);
    write_u32(out, 8, frame_cursor as u32);
    write_u16(out, 12, command_count);
    write_u16(out, 14, 0);
    Some(frame_cursor)
}

fn render_edit_box(
    out: &mut [u8],
    frame_cursor: &mut usize,
    command_count: &mut u16,
    rect: [u16; 4],
    text: &[u8],
) -> Option<()> {
    push_rect(out, frame_cursor, 2, rect, 0x1b212bff)?;
    push_rect(out, frame_cursor, 3, rect, 0x3e434cff)?;
    *command_count += 2;

    let columns = ((rect[2].saturating_sub(24) / 8).clamp(1, 52)) as usize;
    let rows = (rect[3].saturating_sub(24) / 20).max(1) as usize;
    let mut line_count = 1usize;
    let mut column = 0usize;
    for byte in text {
        if *byte == b'\n' {
            line_count += 1;
            column = 0;
        } else {
            column += 1;
            if column > columns {
                line_count += 1;
                column = 1;
            }
        }
    }
    let first_line = line_count.saturating_sub(rows);
    let mut line = [0u8; 52];
    let mut line_len = 0usize;
    let mut line_index = 0usize;
    let mut last_line_len = 0usize;
    let mut last_line_row = 0usize;
    let origin_x = rect[0].saturating_add(12);
    let origin_y = rect[1].saturating_add(18);

    for byte in text.iter().chain(core::iter::once(&b'\n')) {
        if *byte == b'\n' {
            if line_index >= first_line {
                push_text(
                    out,
                    frame_cursor,
                    origin_x,
                    origin_y.saturating_add(((line_index - first_line) * 20) as u16),
                    0xf4f7faff,
                    &line[..line_len],
                )?;
                *command_count += 1;
            }
            last_line_len = line_len;
            last_line_row = line_index.saturating_sub(first_line);
            line_index += 1;
            line_len = 0;
        } else {
            if line_len == columns {
                if line_index >= first_line {
                    push_text(
                        out,
                        frame_cursor,
                        origin_x,
                        origin_y.saturating_add(((line_index - first_line) * 20) as u16),
                        0xf4f7faff,
                        &line[..line_len],
                    )?;
                    *command_count += 1;
                }
                last_line_len = line_len;
                last_line_row = line_index.saturating_sub(first_line);
                line_index += 1;
                line_len = 0;
            }
            line[line_len] = if (0x20..=0x7e).contains(byte) {
                *byte
            } else {
                b'?'
            };
            line_len += 1;
        }
    }

    let cursor_x = origin_x
        .saturating_add((last_line_len * 8) as u16)
        .min(rect[0].saturating_add(rect[2].saturating_sub(8)));
    let cursor_y = origin_y
        .saturating_add((last_line_row * 20) as u16)
        .min(rect[1].saturating_add(rect[3].saturating_sub(16)));
    push_rect(
        out,
        frame_cursor,
        2,
        [cursor_x, cursor_y, 8, 16],
        0x36c7a1ff,
    )?;
    *command_count += 1;
    Some(())
}

fn decimal(value: i64, out: &mut [u8; 20]) -> &[u8] {
    let negative = value < 0;
    let mut magnitude = value.unsigned_abs();
    let mut cursor = out.len();
    loop {
        cursor -= 1;
        out[cursor] = b'0' + (magnitude % 10) as u8;
        magnitude /= 10;
        if magnitude == 0 {
            break;
        }
    }
    if negative {
        cursor -= 1;
        out[cursor] = b'-';
    }
    &out[cursor..]
}

fn push_rect(
    out: &mut [u8],
    cursor: &mut usize,
    opcode: u8,
    rect: [u16; 4],
    rgba: u32,
) -> Option<()> {
    let mut payload = [0u8; 12];
    for (index, value) in rect.into_iter().enumerate() {
        write_u16(&mut payload, index * 2, value);
    }
    write_u32(&mut payload, 8, rgba);
    push_command(out, cursor, opcode, &payload)
}

fn push_text(
    out: &mut [u8],
    cursor: &mut usize,
    x: u16,
    y: u16,
    rgba: u32,
    text: &[u8],
) -> Option<()> {
    if text.len() > 64 {
        return None;
    }
    let mut payload = [0u8; 76];
    let len = 12 + text.len();
    write_u16(&mut payload, 0, x);
    write_u16(&mut payload, 2, y);
    write_u32(&mut payload, 4, rgba);
    write_u16(&mut payload, 8, text.len() as u16);
    payload[12..len].copy_from_slice(text);
    push_command(out, cursor, 4, &payload[..len])
}

fn push_command(out: &mut [u8], cursor: &mut usize, opcode: u8, payload: &[u8]) -> Option<()> {
    let padded_len = payload.len().checked_add(3)? & !3;
    let payload_start = cursor.checked_add(4)?;
    let payload_end = payload_start.checked_add(payload.len())?;
    let command_end = payload_start.checked_add(padded_len)?;
    if command_end > out.len() || payload.len() > u16::MAX as usize {
        return None;
    }
    out[*cursor] = opcode;
    out[*cursor + 1] = 0;
    write_u16(out, *cursor + 2, payload.len() as u16);
    out[payload_start..payload_end].copy_from_slice(payload);
    out[payload_end..command_end].fill(0);
    *cursor = command_end;
    Some(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let raw = bytes.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let raw = bytes.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes(raw.try_into().ok()?))
}

fn read_i64(bytes: &[u8], offset: usize) -> Option<i64> {
    let raw = bytes.get(offset..offset.checked_add(8)?)?;
    Some(i64::from_le_bytes(raw.try_into().ok()?))
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::{build_frame, decimal, read_u16};

    fn edit_context(
        text: &[u8],
        capacity: u16,
        widgets: usize,
    ) -> ([u8; 256], [u8; 16], usize, usize) {
        let mut context = [0u8; 256];
        let mut input = [0u8; 16];
        let text_state_len = 2 + text.len();
        let program_start = 32 + 8 + text_state_len;
        let program_len = 32 + 8 + 4 + widgets * 16;
        let context_len = program_start + program_len;

        context[..4].copy_from_slice(b"RAPP");
        context[4..6].copy_from_slice(&1u16.to_le_bytes());
        context[6..8].copy_from_slice(&32u16.to_le_bytes());
        context[8..12].copy_from_slice(&(context_len as u32).to_le_bytes());
        context[12..16].copy_from_slice(&7u32.to_le_bytes());
        context[16..18].copy_from_slice(&320u16.to_le_bytes());
        context[18..20].copy_from_slice(&240u16.to_le_bytes());
        context[20..22].copy_from_slice(&1u16.to_le_bytes());
        context[22..24].copy_from_slice(&(text_state_len as u16).to_le_bytes());
        context[24..28].copy_from_slice(&(program_len as u32).to_le_bytes());
        context[40..42].copy_from_slice(&(text.len() as u16).to_le_bytes());
        context[42..42 + text.len()].copy_from_slice(text);

        let program = &mut context[program_start..context_len];
        program[..4].copy_from_slice(b"RUIP");
        program[4..6].copy_from_slice(&1u16.to_le_bytes());
        program[6..8].copy_from_slice(&32u16.to_le_bytes());
        program[8..12].copy_from_slice(&(program_len as u32).to_le_bytes());
        program[12] = 1;
        program[13] = widgets as u8;
        program[16] = 1;
        program[40..42].copy_from_slice(&capacity.to_le_bytes());
        for widget in 0..widgets {
            let prefix = 44 + widget * 16;
            program[prefix] = 4;
            program[prefix + 4..prefix + 6].copy_from_slice(&10u16.to_le_bytes());
            program[prefix + 6..prefix + 8].copy_from_slice(&20u16.to_le_bytes());
            program[prefix + 8..prefix + 10].copy_from_slice(&120u16.to_le_bytes());
            program[prefix + 10..prefix + 12].copy_from_slice(&100u16.to_le_bytes());
        }

        input[..4].copy_from_slice(b"RINP");
        input[4..6].copy_from_slice(&1u16.to_le_bytes());
        input[6..8].copy_from_slice(&16u16.to_le_bytes());
        input[8..12].copy_from_slice(&7u32.to_le_bytes());
        (context, input, context_len, program_start)
    }

    fn has_text(frame: &[u8], len: usize, expected: &[u8]) -> bool {
        let mut cursor = 16;
        while cursor < len {
            let payload_len = read_u16(frame, cursor + 2).unwrap() as usize;
            let payload = &frame[cursor + 4..cursor + 4 + payload_len];
            if frame[cursor] == 4
                && read_u16(payload, 8) == Some(expected.len() as u16)
                && payload.get(12..) == Some(expected)
            {
                return true;
            }
            cursor += 4 + ((payload_len + 3) & !3);
        }
        false
    }

    fn has_cursor(frame: &[u8], len: usize) -> bool {
        let mut cursor = 16;
        while cursor < len {
            let payload_len = read_u16(frame, cursor + 2).unwrap() as usize;
            let payload = &frame[cursor + 4..cursor + 4 + payload_len];
            if frame[cursor] == 2
                && read_u16(payload, 4) == Some(8)
                && read_u16(payload, 6) == Some(16)
                && payload.get(8..12) == Some(&0x36c7a1ffu32.to_le_bytes())
            {
                return true;
            }
            cursor += 4 + ((payload_len + 3) & !3);
        }
        false
    }

    #[test]
    fn signed_decimal_handles_boundaries() {
        let mut out = [0u8; 20];
        assert_eq!(decimal(0, &mut out), b"0");
        assert_eq!(decimal(42, &mut out), b"42");
        assert_eq!(decimal(-42, &mut out), b"-42");
        assert_eq!(decimal(i64::MIN, &mut out), b"-9223372036854775808");
        assert_eq!(read_u16(&[1, 2], 0), Some(0x0201));
    }

    #[test]
    fn program_packet_renders_current_state_and_rejects_mismatch() {
        let mut program = [0u8; 32 + 8 + 16];
        program[..4].copy_from_slice(b"RUIP");
        program[4..6].copy_from_slice(&1u16.to_le_bytes());
        program[6..8].copy_from_slice(&32u16.to_le_bytes());
        let program_len = program.len() as u32;
        program[8..12].copy_from_slice(&program_len.to_le_bytes());
        program[12] = 1;
        program[13] = 1;
        let widget = 40;
        program[widget] = 2;
        program[widget + 8..widget + 10].copy_from_slice(&120u16.to_le_bytes());
        program[widget + 10..widget + 12].copy_from_slice(&40u16.to_le_bytes());

        let mut context = [0u8; 32 + 8 + 32 + 8 + 16];
        context[..4].copy_from_slice(b"RAPP");
        context[4..6].copy_from_slice(&1u16.to_le_bytes());
        context[6..8].copy_from_slice(&32u16.to_le_bytes());
        context[12..16].copy_from_slice(&7u32.to_le_bytes());
        context[16..18].copy_from_slice(&320u16.to_le_bytes());
        context[18..20].copy_from_slice(&240u16.to_le_bytes());
        context[20..22].copy_from_slice(&1u16.to_le_bytes());
        context[24..28].copy_from_slice(&program_len.to_le_bytes());
        context[32..40].copy_from_slice(&42i64.to_le_bytes());
        context[40..].copy_from_slice(&program);
        let context_len = context.len() as u32;
        context[8..12].copy_from_slice(&context_len.to_le_bytes());

        let mut input = [0u8; 16];
        input[..4].copy_from_slice(b"RINP");
        input[4..6].copy_from_slice(&1u16.to_le_bytes());
        input[6..8].copy_from_slice(&16u16.to_le_bytes());
        input[8..12].copy_from_slice(&7u32.to_le_bytes());
        let mut frame = [0u8; 1024];

        let len = build_frame(&context, &input, 320, 240, &mut frame).unwrap();
        assert_eq!(&frame[..4], b"RFRM");
        assert_eq!(read_u16(&frame, 12), Some(4));
        assert!(frame[..len].windows(2).any(|window| window == b"42"));

        context[20] = 2;
        assert_eq!(build_frame(&context, &input, 320, 240, &mut frame), None);
    }

    #[test]
    fn edit_box_renders_text_and_cursor() {
        let (context, input, context_len, _) = edit_context(b"AB\nC", 16, 1);
        let mut frame = [0u8; 16 * 1024];
        let len = build_frame(&context[..context_len], &input, 320, 240, &mut frame).unwrap();
        assert_eq!(&frame[..4], b"RFRM");
        assert_eq!(read_u16(&frame, 12), Some(6));
        assert!(has_text(&frame, len, b"AB"));
        assert!(has_text(&frame, len, b"C"));
        assert!(has_cursor(&frame, len));
    }

    #[test]
    fn edit_box_bottom_follows_trailing_rows() {
        let (mut context, input, context_len, program_start) = edit_context(b"A\nB\nC", 16, 1);
        let widget = program_start + 44;
        context[widget + 10..widget + 12].copy_from_slice(&44u16.to_le_bytes());
        let mut frame = [0u8; 16 * 1024];
        let len = build_frame(&context[..context_len], &input, 320, 240, &mut frame).unwrap();
        assert!(has_text(&frame, len, b"C"));
        assert!(!has_text(&frame, len, b"A"));
        assert!(!has_text(&frame, len, b"B"));
    }

    #[test]
    fn edit_box_hard_wraps_at_columns() {
        let (mut context, input, context_len, program_start) = edit_context(b"ABCD", 16, 1);
        let widget = program_start + 44;
        context[widget + 8..widget + 10].copy_from_slice(&48u16.to_le_bytes());
        context[widget + 10..widget + 12].copy_from_slice(&64u16.to_le_bytes());
        let mut frame = [0u8; 16 * 1024];
        let len = build_frame(&context[..context_len], &input, 320, 240, &mut frame).unwrap();
        assert!(has_text(&frame, len, b"ABC"));
        assert!(has_text(&frame, len, b"D"));
    }

    #[test]
    fn edit_box_rejects_invalid_text_extensions() {
        let (mut context, input, context_len, program_start) = edit_context(b"AB", 16, 1);
        let mut frame = [0u8; 16 * 1024];

        context[program_start + 16] = 0;
        assert_eq!(
            build_frame(&context[..context_len], &input, 320, 240, &mut frame),
            None
        );

        let (mut context, input, context_len, _) = edit_context(b"AB", 16, 1);
        context[22..24].copy_from_slice(&0u16.to_le_bytes());
        assert_eq!(
            build_frame(&context[..context_len], &input, 320, 240, &mut frame),
            None
        );

        let (mut context, input, context_len, program_start) = edit_context(b"AB", 1, 1);
        assert_eq!(
            build_frame(&context[..context_len], &input, 320, 240, &mut frame),
            None
        );
        context[program_start + 40..program_start + 42].copy_from_slice(&0u16.to_le_bytes());
        assert_eq!(
            build_frame(&context[..context_len], &input, 320, 240, &mut frame),
            None
        );
        context[program_start + 40..program_start + 42].copy_from_slice(&2049u16.to_le_bytes());
        assert_eq!(
            build_frame(&context[..context_len], &input, 320, 240, &mut frame),
            None
        );

        let (mut context, input, context_len, _) = edit_context(b"AB", 16, 1);
        context[22..24].copy_from_slice(&3u16.to_le_bytes());
        assert_eq!(
            build_frame(&context[..context_len], &input, 320, 240, &mut frame),
            None
        );

        let (mut context, input, context_len, program_start) = edit_context(b"AB", 16, 1);
        context[program_start + 44 + 12..program_start + 44 + 14]
            .copy_from_slice(&1u16.to_le_bytes());
        assert_eq!(
            build_frame(&context[..context_len], &input, 320, 240, &mut frame),
            None
        );

        let (context, input, context_len, _) = edit_context(b"AB", 16, 2);
        assert_eq!(
            build_frame(&context[..context_len], &input, 320, 240, &mut frame),
            None
        );

        let (mut context, input, context_len, program_start) = edit_context(b"AB", 16, 1);
        context[program_start + 42] = 1;
        assert_eq!(
            build_frame(&context[..context_len], &input, 320, 240, &mut frame),
            None
        );
        context[program_start + 42] = 0;
        context[program_start + 44 + 1] = 1;
        assert_eq!(
            build_frame(&context[..context_len], &input, 320, 240, &mut frame),
            None
        );
        context[program_start + 44 + 1] = 0;
        context[program_start + 44 + 14] = 1;
        assert_eq!(
            build_frame(&context[..context_len], &input, 320, 240, &mut frame),
            None
        );
    }
}
