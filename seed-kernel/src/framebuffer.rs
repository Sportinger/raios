extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use core::ptr;

use limine::framebuffer::Framebuffer;

use crate::serial;

#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub format: PixelFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Bgra8888,
    Rgba8888,
    Unknown,
}

impl PixelFormat {
    fn infer(fb: &Framebuffer<'_>) -> Self {
        let r = fb.red_mask_shift();
        let g = fb.green_mask_shift();
        let b = fb.blue_mask_shift();
        match (r, g, b) {
            (16, 8, 0) => PixelFormat::Bgra8888,
            (0, 8, 16) => PixelFormat::Rgba8888,
            _ => PixelFormat::Unknown,
        }
    }
}

pub struct FramebufferSurface {
    info: FramebufferInfo,
    front: *mut u8,
    back: &'static mut [u8],
    draw_scale: usize,
}

impl FramebufferSurface {
    #[inline(never)]
    pub fn from_limine(fb: &Framebuffer<'_>) -> Option<Self> {
        serial::write_line("Framebuffer surface: validating Limine mode");
        if fb.bpp() < 32 {
            return None;
        }
        let front = fb.addr();
        if front.is_null() {
            return None;
        }
        let info = FramebufferInfo {
            width: fb.width(),
            height: fb.height(),
            pitch: fb.pitch(),
            format: PixelFormat::infer(fb),
        };
        let row_bytes = info.width.checked_mul(4)?;
        if info.width == 0 || info.height == 0 || info.pitch < row_bytes {
            return None;
        }
        let framebuffer_len = (info.pitch as usize).checked_mul(info.height as usize)?;
        let back = Box::leak(vec![0u8; framebuffer_len].into_boxed_slice());
        serial::write_line("Framebuffer surface: ready");
        Some(Self {
            info,
            front,
            back,
            draw_scale: 1,
        })
    }

    pub fn info(&self) -> FramebufferInfo {
        self.info
    }

    pub fn set_draw_scale(&mut self, scale: usize) {
        self.draw_scale = scale.max(1);
    }

    pub fn draw_scale(&self) -> usize {
        self.draw_scale
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        let scale = self.draw_scale;
        if scale == 1 {
            self.set_back_pixel_unscaled(x, y, color);
            return;
        }

        let phys_x = x.saturating_mul(scale);
        let phys_y = y.saturating_mul(scale);
        let mut dy = 0usize;
        while dy < scale {
            let mut dx = 0usize;
            while dx < scale {
                self.set_back_pixel_unscaled(phys_x + dx, phys_y + dy, color);
                dx += 1;
            }
            dy += 1;
        }
    }

    pub fn fill(&mut self, color: Color) {
        self.fill_rect_unscaled(
            0,
            0,
            self.info.width as usize,
            self.info.height as usize,
            color,
        );
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        let scale = self.draw_scale;
        if scale == 1 {
            self.fill_rect_unscaled(x, y, w, h, color);
        } else {
            self.fill_rect_unscaled(
                x.saturating_mul(scale),
                y.saturating_mul(scale),
                w.saturating_mul(scale),
                h.saturating_mul(scale),
                color,
            );
        }
    }

    fn fill_rect_unscaled(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        let pitch = self.info.pitch as usize;
        let bytes = color.to_bytes(self.info.format);
        let max_y = usize::min(y.saturating_add(h), self.info.height as usize);
        let max_x = usize::min(x.saturating_add(w), self.info.width as usize);
        for row in y..max_y {
            if x >= max_x {
                continue;
            }
            for col in x..max_x {
                self.write_pixel_bytes(row * pitch + col * 4, bytes);
            }
        }
    }

    pub fn present(&mut self) {
        unsafe {
            ptr::copy_nonoverlapping(self.back.as_ptr(), self.front, self.back.len());
        }
    }

    pub fn restore_from_back_rect(&mut self, x: usize, y: usize, w: usize, h: usize) {
        let pitch = self.info.pitch as usize;
        let max_y = usize::min(y.saturating_add(h), self.info.height as usize);
        let max_x = usize::min(x.saturating_add(w), self.info.width as usize);
        if x >= max_x {
            return;
        }

        let row_bytes = (max_x - x).saturating_mul(4);
        for row in y..max_y {
            let offset = row
                .saturating_mul(pitch)
                .saturating_add(x.saturating_mul(4));
            if offset.saturating_add(row_bytes) > self.back.len() {
                break;
            }
            unsafe {
                ptr::copy_nonoverlapping(
                    self.back.as_ptr().add(offset),
                    self.front.add(offset),
                    row_bytes,
                );
            }
        }
    }

    pub fn set_front_pixel(&mut self, x: usize, y: usize, color: Color) {
        if let Some(offset) = self.pixel_offset(x, y) {
            let bytes = color.to_bytes(self.info.format);
            unsafe {
                let px = self.front.add(offset);
                ptr::write_volatile(px, bytes[0]);
                ptr::write_volatile(px.add(1), bytes[1]);
                ptr::write_volatile(px.add(2), bytes[2]);
                ptr::write_volatile(px.add(3), bytes[3]);
            }
        }
    }

    pub fn set_physical_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.set_back_pixel_unscaled(x, y, color);
    }

    pub fn blend_physical_pixel(&mut self, x: usize, y: usize, color: Color, alpha: u8) {
        if alpha == 0 {
            return;
        }
        if alpha == u8::MAX {
            self.set_back_pixel_unscaled(x, y, color);
            return;
        }

        let Some(offset) = self.pixel_offset(x, y) else {
            return;
        };
        if offset + 3 >= self.back.len() {
            return;
        }

        let bg = Color::from_bytes(
            [
                self.back[offset],
                self.back[offset + 1],
                self.back[offset + 2],
                self.back[offset + 3],
            ],
            self.info.format,
        );
        self.write_pixel_bytes(
            offset,
            color.blend_over(bg, alpha).to_bytes(self.info.format),
        );
    }

    fn pixel_offset(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.info.width as usize || y >= self.info.height as usize {
            return None;
        }
        let pitch = self.info.pitch as usize;
        Some(y * pitch + x * 4)
    }

    fn set_back_pixel_unscaled(&mut self, x: usize, y: usize, color: Color) {
        if let Some(offset) = self.pixel_offset(x, y) {
            let bytes = color.to_bytes(self.info.format);
            self.write_pixel_bytes(offset, bytes);
        }
    }

    fn write_pixel_bytes(&mut self, offset: usize, bytes: [u8; 4]) {
        if offset + 3 < self.back.len() {
            self.back[offset] = bytes[0];
            self.back[offset + 1] = bytes[1];
            self.back[offset + 2] = bytes[2];
            self.back[offset + 3] = bytes[3];
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 0xFF }
    }

    fn to_bytes(&self, format: PixelFormat) -> [u8; 4] {
        match format {
            PixelFormat::Bgra8888 | PixelFormat::Unknown => [self.b, self.g, self.r, self.a],
            PixelFormat::Rgba8888 => [self.r, self.g, self.b, self.a],
        }
    }

    fn from_bytes(bytes: [u8; 4], format: PixelFormat) -> Self {
        match format {
            PixelFormat::Bgra8888 | PixelFormat::Unknown => Self {
                r: bytes[2],
                g: bytes[1],
                b: bytes[0],
                a: bytes[3],
            },
            PixelFormat::Rgba8888 => Self {
                r: bytes[0],
                g: bytes[1],
                b: bytes[2],
                a: bytes[3],
            },
        }
    }

    fn blend_over(self, bg: Color, alpha: u8) -> Color {
        let alpha = u16::from(alpha);
        let inv = 255u16.saturating_sub(alpha);
        let blend = |fg: u8, bg: u8| -> u8 {
            ((u16::from(fg) * alpha + u16::from(bg) * inv + 127) / 255) as u8
        };
        Color {
            r: blend(self.r, bg.r),
            g: blend(self.g, bg.g),
            b: blend(self.b, bg.b),
            a: 0xFF,
        }
    }
}
