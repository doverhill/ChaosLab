//! Framebuffer text console.
//!
//! Renders monospace glyphs from `noto-sans-mono-bitmap` into the UEFI pixel
//! framebuffer. Supports cursor tracking, line wrap, scrolling, and per-section
//! RGB color.

use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};

const FONT_WEIGHT: FontWeight = FontWeight::Regular;
const FONT_SIZE: RasterHeight = RasterHeight::Size16;
const FONT_HEIGHT: usize = 16;
const LINE_SPACING: usize = 2;

/// Character width in pixels (constant for monospace font).
pub const CHAR_WIDTH: usize = get_raster_width(FONT_WEIGHT, FONT_SIZE);
/// Character cell height including line spacing.
pub const CHAR_HEIGHT: usize = FONT_HEIGHT + LINE_SPACING;

pub struct FramebufferWriter {
    buffer: &'static mut [u8],
    info: FrameBufferInfo,
    cols: usize,
    rows: usize,
    cursor_x: usize,
    cursor_y: usize,
    color: [u8; 3],
}

impl FramebufferWriter {
    pub fn new(buffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let cols = info.width / CHAR_WIDTH;
        let rows = info.height / CHAR_HEIGHT;

        let mut writer = FramebufferWriter {
            buffer,
            info,
            cols,
            rows,
            cursor_x: 0,
            cursor_y: 0,
            color: [200, 200, 200],
        };
        writer.clear();
        writer
    }

    pub fn buffer_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Update the buffer pointer to a new base address (e.g., after remapping).
    pub fn remap_buffer(&mut self, new_base: *mut u8) {
        let len = self.buffer.len();
        self.buffer = unsafe { core::slice::from_raw_parts_mut(new_base, len) };
    }

    pub fn set_color(&mut self, r: u8, g: u8, b: u8) {
        self.color = [r, g, b];
    }

    pub fn clear(&mut self) {
        for byte in self.buffer.iter_mut() {
            *byte = 0;
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    fn write_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        let offset = (y * self.info.stride + x) * self.info.bytes_per_pixel;
        if offset + self.info.bytes_per_pixel > self.buffer.len() {
            return;
        }
        match self.info.pixel_format {
            PixelFormat::Rgb => {
                self.buffer[offset] = r;
                self.buffer[offset + 1] = g;
                self.buffer[offset + 2] = b;
            }
            PixelFormat::Bgr => {
                self.buffer[offset] = b;
                self.buffer[offset + 1] = g;
                self.buffer[offset + 2] = r;
            }
            PixelFormat::U8 => {
                // grayscale: use luminance approximation
                self.buffer[offset] = ((r as u16 * 77 + g as u16 * 150 + b as u16 * 29) >> 8) as u8;
            }
            _ => {
                self.buffer[offset] = g;
            }
        }
    }

    fn render_char(&mut self, c: char, cx: usize, cy: usize) {
        let raster = match get_raster(c, FONT_WEIGHT, FONT_SIZE) {
            Some(r) => r,
            None => match get_raster('\u{FFFD}', FONT_WEIGHT, FONT_SIZE) {
                Some(r) => r,
                None => return,
            },
        };

        let x0 = cx * CHAR_WIDTH;
        let y0 = cy * CHAR_HEIGHT;

        for (row, row_data) in raster.raster().iter().enumerate() {
            for (col, &intensity) in row_data.iter().enumerate() {
                if intensity > 0 {
                    let r = (self.color[0] as u16 * intensity as u16 / 255) as u8;
                    let g = (self.color[1] as u16 * intensity as u16 / 255) as u8;
                    let b = (self.color[2] as u16 * intensity as u16 / 255) as u8;
                    self.write_pixel(x0 + col, y0 + row, r, g, b);
                }
            }
        }
    }

    fn scroll_up(&mut self) {
        let row_bytes = CHAR_HEIGHT * self.info.stride * self.info.bytes_per_pixel;
        let src_start = row_bytes;
        let copy_len = (self.rows - 1) * row_bytes;

        if src_start + copy_len <= self.buffer.len() {
            unsafe {
                core::ptr::copy(self.buffer.as_ptr().add(src_start), self.buffer.as_mut_ptr(), copy_len);
            }
        }

        // clear last row
        let last_start = (self.rows - 1) * row_bytes;
        let last_end = (last_start + row_bytes).min(self.buffer.len());
        for byte in &mut self.buffer[last_start..last_end] {
            *byte = 0;
        }
    }

    fn newline(&mut self) {
        self.cursor_x = 0;
        if self.cursor_y + 1 < self.rows {
            self.cursor_y += 1;
        } else {
            self.scroll_up();
        }
    }
}

impl core::fmt::Write for FramebufferWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            match c {
                '\n' => self.newline(),
                c => {
                    if self.cursor_x >= self.cols {
                        self.newline();
                    }
                    self.render_char(c, self.cursor_x, self.cursor_y);
                    self.cursor_x += 1;
                }
            }
        }
        Ok(())
    }
}
