use crate::framebuffer::{Color, FramebufferSurface};

mod data {
    include!("font_data.rs");
}

const FONT_WIDTH: usize = 8;
const FONT_HEIGHT: usize = 8;
const FONT_FIRST: u8 = 32;

pub fn draw_text(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    text: &str,
    fg: Color,
    bg: Option<Color>,
) {
    let mut cursor_x = x;
    let mut cursor_y = y;
    for ch in text.chars() {
        match ch {
            '\n' => {
                cursor_x = x;
                cursor_y += FONT_HEIGHT + 2;
                continue;
            }
            '\r' => {
                cursor_x = x;
                continue;
            }
            _ => {}
        }
        if let Some(glyph) = glyph(ch) {
            draw_glyph(surface, cursor_x, cursor_y, glyph, fg, bg);
        }
        cursor_x += FONT_WIDTH + 1;
    }
}

fn glyph(ch: char) -> Option<&'static [u8; FONT_HEIGHT]> {
    let code = ch as u32;
    if !(FONT_FIRST as u32..=(FONT_FIRST as u32 + data::FONT8X8_BASIC.len() as u32 - 1))
        .contains(&code)
    {
        return None;
    }
    let idx = (code - FONT_FIRST as u32) as usize;
    data::FONT8X8_BASIC.get(idx)
}

fn draw_glyph(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    glyph: &[u8; FONT_HEIGHT],
    fg: Color,
    bg: Option<Color>,
) {
    for (row_idx, row_bits) in glyph.iter().enumerate() {
        if y + row_idx >= surface.info().height as usize {
            break;
        }
        for col in 0..FONT_WIDTH {
            let bit = (row_bits >> (7 - col)) & 1;
            let color = if bit == 1 { Some(fg) } else { bg };
            if let Some(color) = color {
                surface.set_pixel(x + col, y + row_idx, color);
            }
        }
    }
}
