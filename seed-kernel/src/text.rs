use crate::framebuffer::{Color, FramebufferSurface};

mod data {
    include!("font_data.rs");
}

const FONT_WIDTH: usize = 8;
const FONT_HEIGHT: usize = 8;
const FONT_FIRST: u8 = 32;
const FALLBACK: char = '?';

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
        draw_glyph(surface, cursor_x, cursor_y, glyph(ch), fg, bg);
        cursor_x += FONT_WIDTH + 1;
    }
}

fn glyph(ch: char) -> &'static [u8; FONT_HEIGHT] {
    let code = ch as u32;
    if (FONT_FIRST as u32..=(FONT_FIRST as u32 + data::FONT8X8_BASIC.len() as u32 - 1))
        .contains(&code)
    {
        let idx = (code - FONT_FIRST as u32) as usize;
        return &data::FONT8X8_BASIC[idx];
    }

    match ch {
        'ä' => &data::GLYPH_A_UMLAUT_LOWER,
        'ö' => &data::GLYPH_O_UMLAUT_LOWER,
        'ü' => &data::GLYPH_U_UMLAUT_LOWER,
        'Ä' => &data::GLYPH_A_UMLAUT_UPPER,
        'Ö' => &data::GLYPH_O_UMLAUT_UPPER,
        'Ü' => &data::GLYPH_U_UMLAUT_UPPER,
        'ß' => &data::GLYPH_SHARP_S,
        _ if ch != FALLBACK => glyph(FALLBACK),
        _ => &data::FONT8X8_BASIC[(FALLBACK as u8 - FONT_FIRST) as usize],
    }
}

fn draw_glyph(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    glyph: &[u8; FONT_HEIGHT],
    fg: Color,
    bg: Option<Color>,
) {
    let scale = surface.draw_scale();
    if scale <= 1 {
        draw_glyph_unscaled(surface, x, y, glyph, fg, bg);
    } else {
        draw_glyph_antialiased(surface, x, y, glyph, fg, bg, scale);
    }
}

fn draw_glyph_unscaled(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    glyph: &[u8; FONT_HEIGHT],
    fg: Color,
    bg: Option<Color>,
) {
    let origin_x = x.saturating_mul(surface.draw_scale());
    let origin_y = y.saturating_mul(surface.draw_scale());

    for (row_idx, row_bits) in glyph.iter().enumerate() {
        for col in 0..FONT_WIDTH {
            let bit = (row_bits >> col) & 1;
            match (bit, bg) {
                (1, _) => surface.set_physical_pixel(origin_x + col, origin_y + row_idx, fg),
                (0, Some(bg)) => surface.set_physical_pixel(origin_x + col, origin_y + row_idx, bg),
                _ => {}
            }
        }
    }
}

fn draw_glyph_antialiased(
    surface: &mut FramebufferSurface,
    x: usize,
    y: usize,
    glyph: &[u8; FONT_HEIGHT],
    fg: Color,
    bg: Option<Color>,
    scale: usize,
) {
    let origin_x = x.saturating_mul(scale);
    let origin_y = y.saturating_mul(scale);
    let width = FONT_WIDTH.saturating_mul(scale);
    let height = FONT_HEIGHT.saturating_mul(scale);

    for py in 0..height {
        for px in 0..width {
            let alpha = smoothed_alpha(glyph, px, py, scale);
            match bg {
                Some(bg) => {
                    surface.set_physical_pixel(
                        origin_x + px,
                        origin_y + py,
                        blend_colors(fg, bg, alpha),
                    );
                }
                None => surface.blend_physical_pixel(origin_x + px, origin_y + py, fg, alpha),
            }
        }
    }
}

fn smoothed_alpha(glyph: &[u8; FONT_HEIGHT], px: usize, py: usize, scale: usize) -> u8 {
    let src_x = centered_source_coord(px, scale, FONT_WIDTH);
    let src_y = centered_source_coord(py, scale, FONT_HEIGHT);
    let x0 = src_x.cell;
    let y0 = src_y.cell;
    let x1 = usize::min(x0 + 1, FONT_WIDTH - 1);
    let y1 = usize::min(y0 + 1, FONT_HEIGHT - 1);

    let a00 = u32::from(mask_alpha(glyph, x0, y0));
    let a10 = u32::from(mask_alpha(glyph, x1, y0));
    let a01 = u32::from(mask_alpha(glyph, x0, y1));
    let a11 = u32::from(mask_alpha(glyph, x1, y1));
    let fx = u32::from(src_x.frac);
    let fy = u32::from(src_y.frac);
    let ix = 255 - fx;
    let iy = 255 - fy;
    let alpha = (a00 * ix * iy + a10 * fx * iy + a01 * ix * fy + a11 * fx * fy) / (255 * 255);

    alpha_curve(alpha as u8)
}

#[derive(Clone, Copy)]
struct SourceCoord {
    cell: usize,
    frac: u8,
}

fn centered_source_coord(pixel: usize, scale: usize, cells: usize) -> SourceCoord {
    let coord = ((pixel.saturating_mul(256) + 128) / scale) as isize - 128;
    if coord <= 0 {
        return SourceCoord { cell: 0, frac: 0 };
    }

    let max = ((cells - 1) * 256) as isize;
    let clamped = coord.min(max) as usize;
    SourceCoord {
        cell: clamped / 256,
        frac: (clamped % 256) as u8,
    }
}

fn mask_alpha(glyph: &[u8; FONT_HEIGHT], x: usize, y: usize) -> u8 {
    if (glyph[y] >> x) & 1 == 1 {
        u8::MAX
    } else {
        0
    }
}

fn alpha_curve(alpha: u8) -> u8 {
    match alpha {
        0 => 0,
        1..=48 => alpha.saturating_mul(2),
        49..=192 => alpha,
        _ => u8::MAX,
    }
}

fn blend_colors(fg: Color, bg: Color, alpha: u8) -> Color {
    let alpha = u16::from(alpha);
    let inv = 255u16.saturating_sub(alpha);
    let blend = |fg: u8, bg: u8| -> u8 {
        ((u16::from(fg) * alpha + u16::from(bg) * inv + 127) / 255) as u8
    };
    Color::new(blend(fg.r, bg.r), blend(fg.g, bg.g), blend(fg.b, bg.b))
}
