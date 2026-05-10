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
        let _framebuffer_len = (info.pitch as usize).checked_mul(info.height as usize)?;
        serial::write_line("Framebuffer surface: ready");
        Some(Self { info, front })
    }

    pub fn info(&self) -> FramebufferInfo {
        self.info
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if let Some(offset) = self.pixel_offset(x, y) {
            let bytes = color.to_bytes(self.info.format);
            self.write_pixel_bytes(offset, bytes);
        }
    }

    pub fn fill(&mut self, color: Color) {
        self.fill_rect(
            0,
            0,
            self.info.width as usize,
            self.info.height as usize,
            color,
        );
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        let pitch = self.info.pitch as usize;
        let bytes = color.to_bytes(self.info.format);
        let max_y = usize::min(y + h, self.info.height as usize);
        let max_x = usize::min(x + w, self.info.width as usize);
        for row in y..max_y {
            if x >= max_x {
                continue;
            }
            for col in x..max_x {
                self.write_pixel_bytes(row * pitch + col * 4, bytes);
            }
        }
    }

    pub fn present(&mut self) {}

    fn pixel_offset(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.info.width as usize || y >= self.info.height as usize {
            return None;
        }
        let pitch = self.info.pitch as usize;
        Some(y * pitch + x * 4)
    }

    fn write_pixel_bytes(&mut self, offset: usize, bytes: [u8; 4]) {
        unsafe {
            let px = self.front.add(offset);
            ptr::write_volatile(px, bytes[0]);
            ptr::write_volatile(px.add(1), bytes[1]);
            ptr::write_volatile(px.add(2), bytes[2]);
            ptr::write_volatile(px.add(3), bytes[3]);
        }
    }
}

#[derive(Clone, Copy)]
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
}
