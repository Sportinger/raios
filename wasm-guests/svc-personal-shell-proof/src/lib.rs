#![no_std]

#[cfg(not(test))]
use core::panic::PanicInfo;

const CONTEXT_LEN: usize = 32;
const INPUT_HEADER_LEN: usize = 16;
const INPUT_EVENT_LEN: usize = 16;
const MAX_INPUT_LEN: usize = INPUT_HEADER_LEN + 64 * INPUT_EVENT_LEN;
const FRAME_CAPACITY: usize = 512;

// These are test-only sanitized key codes. Every other event, including an
// empty input packet, keeps the proof guest in its normal render mode.
const MODE_MALFORMED_FRAME: u16 = 0x7ff1;
const MODE_CLIPPED_OVERDRAW: u16 = 0x7ff2;
const MODE_TRAP: u16 = 0x7ff3;
const MODE_FUEL_EXHAUSTION: u16 = 0x7ff4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InvocationMode {
    Normal,
    MalformedFrame,
    ClippedOverdraw,
    Trap,
    FuelExhaustion,
}

#[cfg(not(test))]
#[link(wasm_import_module = "ui")]
extern "C" {
    #[link_name = "viewport"]
    fn ui_viewport() -> i64;
    #[link_name = "context_len"]
    fn ui_context_len() -> i32;
    #[link_name = "context_read"]
    fn ui_context_read(ptr: i32, cap: i32) -> i32;
    #[link_name = "input_len"]
    fn ui_input_len() -> i32;
    #[link_name = "input_read"]
    fn ui_input_read(ptr: i32, cap: i32) -> i32;
    #[link_name = "frame_submit"]
    fn ui_frame_submit(ptr: i32, len: i32) -> i32;
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn raios_service_main() -> i32 {
    let packed_viewport = unsafe { ui_viewport() } as u64;
    let width = (packed_viewport >> 32) as u32;
    let height = packed_viewport as u32;

    let context_len = unsafe { ui_context_len() };
    if context_len != CONTEXT_LEN as i32 {
        return -1;
    }
    let mut context = [0u8; CONTEXT_LEN];
    if unsafe { ui_context_read(context.as_mut_ptr() as i32, CONTEXT_LEN as i32) }
        != CONTEXT_LEN as i32
    {
        return -2;
    }

    let input_len = unsafe { ui_input_len() };
    if input_len < INPUT_HEADER_LEN as i32 || input_len > MAX_INPUT_LEN as i32 {
        return -2;
    }
    let mut input = [0u8; MAX_INPUT_LEN];
    if unsafe { ui_input_read(input.as_mut_ptr() as i32, input_len) } != input_len {
        return -2;
    }

    let input = &input[..input_len as usize];
    if !valid_packets(&context, input, width, height) {
        return -2;
    }

    let mode = mode_from_input(input);
    match mode {
        InvocationMode::Trap => trigger_trap(),
        InvocationMode::FuelExhaustion => exhaust_fuel(),
        _ => {}
    }

    let mut frame = [0u8; FRAME_CAPACITY];
    let Some(frame_len) = build_frame(
        &mut frame,
        width as u16,
        height as u16,
        read_u16(input, 12) != 0,
        mode,
    ) else {
        return -3;
    };

    if mode == InvocationMode::MalformedFrame {
        // Keep the RFRM identity but violate its required all-zero flags field.
        // The host must atomically reject this complete invalid display list.
        write_u16(&mut frame, 14, 1);
    }

    unsafe { ui_frame_submit(frame.as_ptr() as i32, frame_len as i32) }
}

fn mode_from_input(input: &[u8]) -> InvocationMode {
    for event in input[INPUT_HEADER_LEN..].chunks_exact(INPUT_EVENT_LEN) {
        if event[0] != 1 || event[1] & 1 == 0 {
            continue;
        }
        match read_u16(event, 2) {
            MODE_MALFORMED_FRAME => return InvocationMode::MalformedFrame,
            MODE_CLIPPED_OVERDRAW => return InvocationMode::ClippedOverdraw,
            MODE_TRAP => return InvocationMode::Trap,
            MODE_FUEL_EXHAUSTION => return InvocationMode::FuelExhaustion,
            _ => {}
        }
    }
    InvocationMode::Normal
}

#[inline(never)]
fn trigger_trap() -> ! {
    #[cfg(target_arch = "wasm32")]
    {
        core::arch::wasm32::unreachable()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        panic!("proof guest trap mode")
    }
}

#[inline(never)]
fn exhaust_fuel() -> ! {
    let mut tick = 0u32;
    loop {
        let next = unsafe { core::ptr::read_volatile(core::ptr::addr_of!(tick)) }.wrapping_add(1);
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!(tick), next) };
    }
}

fn valid_packets(context: &[u8; CONTEXT_LEN], input: &[u8], width: u32, height: u32) -> bool {
    if width == 0
        || height == 0
        || width > u16::MAX as u32
        || height > u16::MAX as u32
        || context[..4] != *b"RCTX"
        || read_u16(context, 4) != 1
        || read_u16(context, 6) != CONTEXT_LEN as u16
        || read_u16(context, 12) != width as u16
        || read_u16(context, 14) != height as u16
        || read_u16(context, 22) & !0b11 != 0
        || read_u32(context, 28) != 0
        || input[..4] != *b"RINP"
        || read_u16(input, 4) != 1
        || read_u16(input, 6) != INPUT_HEADER_LEN as u16
        || read_u32(input, 8) != read_u32(context, 8)
        || read_u16(input, 14) != 0
    {
        return false;
    }

    let event_count = read_u16(input, 12) as usize;
    if event_count > 64 || input.len() != INPUT_HEADER_LEN + event_count * INPUT_EVENT_LEN {
        return false;
    }

    for event in input[INPUT_HEADER_LEN..].chunks_exact(INPUT_EVENT_LEN) {
        if !(1..=3).contains(&event[0]) || event[1] & !0b11 != 0 || read_u16(event, 14) != 0 {
            return false;
        }
    }
    true
}

fn build_frame(
    out: &mut [u8],
    width: u16,
    height: u16,
    has_input: bool,
    mode: InvocationMode,
) -> Option<usize> {
    let mut cursor = 16usize;
    let mut commands = 0u16;

    let background: u32 = if has_input { 0x101f2aff } else { 0x111820ff };
    push_command(out, &mut cursor, 1, &background.to_le_bytes())?;
    commands += 1;

    let panel_width = width.saturating_sub(48).max(1);
    let panel_height = height.saturating_sub(48).min(220).max(1);
    let accent: u32 = if has_input { 0x36c7a1ff } else { 0x63a7ffff };
    push_rect(
        out,
        &mut cursor,
        2,
        [24, 24, panel_width, panel_height],
        0x1c2733ff,
    )?;
    commands += 1;
    push_rect(
        out,
        &mut cursor,
        3,
        [24, 24, panel_width, panel_height],
        accent,
    )?;
    commands += 1;

    push_text(
        out,
        &mut cursor,
        48,
        52,
        0xf4f7faff,
        b"PERSONAL SHELL PROOF",
    )?;
    commands += 1;
    let status = if has_input {
        b"SANITIZED INPUT RECEIVED".as_slice()
    } else {
        b"READY FOR SANITIZED INPUT".as_slice()
    };
    push_text(out, &mut cursor, 48, 92, accent, status)?;
    commands += 1;

    let focus_rect = if mode == InvocationMode::ClippedOverdraw {
        // The rectangle deliberately crosses the declared viewport edge. It is
        // valid RFRM and must arrive at the host as a clipped command.
        let rect = [width.saturating_sub(1), height.saturating_sub(1), 64, 64];
        push_rect(out, &mut cursor, 2, rect, accent)?;
        commands += 1;
        rect
    } else {
        [44, 78, panel_width.saturating_sub(40).max(1), 40]
    };

    let mut focus = [0u8; 8];
    for (index, value) in focus_rect.into_iter().enumerate() {
        write_u16(&mut focus, index * 2, value);
    }
    push_command(out, &mut cursor, 5, &focus)?;
    commands += 1;

    out[..4].copy_from_slice(b"RFRM");
    write_u16(out, 4, 1);
    write_u16(out, 6, 16);
    write_u32(out, 8, cursor as u32);
    write_u16(out, 12, commands);
    write_u16(out, 14, 0);
    Some(cursor)
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
    let mut payload = [0u8; 64];
    let len = 12usize.checked_add(text.len())?;
    if len > payload.len() {
        return None;
    }
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

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input_with_key(code: u16) -> [u8; INPUT_HEADER_LEN + INPUT_EVENT_LEN] {
        let mut input = [0u8; INPUT_HEADER_LEN + INPUT_EVENT_LEN];
        input[..4].copy_from_slice(b"RINP");
        write_u16(&mut input, 4, 1);
        write_u16(&mut input, 6, INPUT_HEADER_LEN as u16);
        write_u16(&mut input, 12, 1);
        input[INPUT_HEADER_LEN] = 1;
        input[INPUT_HEADER_LEN + 1] = 1;
        write_u16(&mut input, INPUT_HEADER_LEN + 2, code);
        input
    }

    #[test]
    fn test_only_key_modes_leave_other_sanitized_input_normal() {
        assert_eq!(mode_from_input(&input_with_key(0)), InvocationMode::Normal);
        assert_eq!(
            mode_from_input(&input_with_key(MODE_MALFORMED_FRAME)),
            InvocationMode::MalformedFrame
        );
        assert_eq!(
            mode_from_input(&input_with_key(MODE_CLIPPED_OVERDRAW)),
            InvocationMode::ClippedOverdraw
        );
        assert_eq!(
            mode_from_input(&input_with_key(MODE_TRAP)),
            InvocationMode::Trap
        );
        assert_eq!(
            mode_from_input(&input_with_key(MODE_FUEL_EXHAUSTION)),
            InvocationMode::FuelExhaustion
        );
    }
}
