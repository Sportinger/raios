#![allow(dead_code)]

use alloc::boxed::Box;
use alloc::vec;
use core::ptr;

use limine::framebuffer::Framebuffer;

#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub format: PixelFormat,
    pub address: u64,
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
}

impl FramebufferSurface {
    pub fn from_limine(fb: &Framebuffer<'_>) -> Option<Self> {
        if fb.bpp() < 32 {
            return None;
        }
        let info = FramebufferInfo {
            width: fb.width(),
            height: fb.height(),
            pitch: fb.pitch(),
            format: PixelFormat::infer(fb),
            address: fb.addr() as u64,
        };
        let back_len = (info.pitch as usize).saturating_mul(info.height as usize);
        let buffer = vec![0u8; back_len].into_boxed_slice();
        let back_slice: &'static mut [u8] = Box::leak(buffer);
        Some(Self {
            info,
            front: fb.addr(),
            back: back_slice,
        })
    }

    pub fn info(&self) -> FramebufferInfo {
        self.info
    }

    pub fn back_buffer(&mut self) -> &mut [u8] {
        self.back
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if let Some(offset) = self.pixel_offset(x, y) {
            let bytes = color.to_bytes(self.info.format);
            self.back[offset..offset + 4].copy_from_slice(&bytes);
        }
    }

    pub fn fill(&mut self, color: Color) {
        let pitch = self.info.pitch as usize;
        let row_bytes = self.info.width as usize * 4;
        let bytes = color.to_bytes(self.info.format);
        for y in 0..self.info.height as usize {
            let row = &mut self.back[y * pitch..y * pitch + row_bytes];
            for px in row.chunks_exact_mut(4) {
                px.copy_from_slice(&bytes);
            }
        }
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        let pitch = self.info.pitch as usize;
        let row_bytes = self.info.width as usize * 4;
        let bytes = color.to_bytes(self.info.format);
        let max_y = usize::min(y + h, self.info.height as usize);
        let max_x = usize::min(x + w, self.info.width as usize);
        for row in y..max_y {
            let start = row * pitch;
            let row_slice = &mut self.back[start..start + row_bytes];
            if x >= max_x {
                continue;
            }
            for px in row_slice[x * 4..max_x * 4].chunks_exact_mut(4) {
                px.copy_from_slice(&bytes);
            }
        }
    }

    pub fn present(&mut self) {
        let len = self.back.len();
        unsafe {
            ptr::copy_nonoverlapping(self.back.as_ptr(), self.front, len);
        }
    }
    fn pixel_offset(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.info.width as usize || y >= self.info.height as usize {
            return None;
        }
        let pitch = self.info.pitch as usize;
        Some(y * pitch + x * 4)
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
