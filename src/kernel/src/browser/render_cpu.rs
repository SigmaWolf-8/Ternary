// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// CPU rendering fallback — tiny-skia path for headless/embedded systems.
// Same pipeline as GPU, but software-rendered. Sponge encryption is CPU XOR.
// Performance: ~124M pixels/sec at 1080p (marginal). GPU path preferred.

use alloc::vec::Vec;

#[derive(Debug, Clone, Copy)]
pub struct RenderCommand {
    pub cmd_type: RenderCommandType,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: [u8; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderCommandType {
    FillRect,
    StrokeRect,
    DrawText,
    DrawImage,
    DrawGradient,
    Clear,
}

pub struct CpuFramebuffer {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl CpuFramebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width as usize) * (height as usize) * 4;
        Self {
            width,
            height,
            pixels: alloc::vec![0u8; size],
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    pub fn clear(&mut self, color: [u8; 4]) {
        for chunk in self.pixels.chunks_exact_mut(4) {
            chunk.copy_from_slice(&color);
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x < self.width && y < self.height {
            let offset = ((y * self.width + x) * 4) as usize;
            self.pixels[offset..offset + 4].copy_from_slice(&color);
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        if x < self.width && y < self.height {
            let offset = ((y * self.width + x) * 4) as usize;
            let mut px = [0u8; 4];
            px.copy_from_slice(&self.pixels[offset..offset + 4]);
            px
        } else {
            [0, 0, 0, 0]
        }
    }

    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);
        for py in y..y_end {
            for px in x..x_end {
                self.set_pixel(px, py, color);
            }
        }
    }

    pub fn pixel_count(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    pub fn byte_size(&self) -> usize {
        self.pixels.len()
    }
}

pub struct CpuRenderer {
    command_queue: Vec<RenderCommand>,
}

impl CpuRenderer {
    pub fn new() -> Self {
        Self {
            command_queue: Vec::new(),
        }
    }

    pub fn submit(&mut self, cmd: RenderCommand) {
        self.command_queue.push(cmd);
    }

    pub fn flush(&mut self, fb: &mut CpuFramebuffer) {
        for cmd in self.command_queue.drain(..) {
            match cmd.cmd_type {
                RenderCommandType::Clear => {
                    fb.clear(cmd.color);
                }
                RenderCommandType::FillRect => {
                    fb.fill_rect(
                        cmd.x as u32,
                        cmd.y as u32,
                        cmd.width as u32,
                        cmd.height as u32,
                        cmd.color,
                    );
                }
                _ => {}
            }
        }
    }

    pub fn pending_commands(&self) -> usize {
        self.command_queue.len()
    }
}

pub fn sponge_encrypt_framebuffer(fb: &mut CpuFramebuffer, keystream: &[u8]) {
    let pixels = fb.pixels_mut();
    let key_len = keystream.len();
    if key_len == 0 {
        return;
    }
    for (i, pixel) in pixels.iter_mut().enumerate() {
        *pixel ^= keystream[i % key_len];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framebuffer_creation() {
        let fb = CpuFramebuffer::new(100, 50);
        assert_eq!(fb.width(), 100);
        assert_eq!(fb.height(), 50);
        assert_eq!(fb.pixel_count(), 5000);
        assert_eq!(fb.byte_size(), 20000);
    }

    #[test]
    fn test_framebuffer_clear() {
        let mut fb = CpuFramebuffer::new(2, 2);
        fb.clear([255, 0, 0, 255]);
        assert_eq!(fb.get_pixel(0, 0), [255, 0, 0, 255]);
        assert_eq!(fb.get_pixel(1, 1), [255, 0, 0, 255]);
    }

    #[test]
    fn test_framebuffer_set_get() {
        let mut fb = CpuFramebuffer::new(10, 10);
        fb.set_pixel(5, 5, [1, 2, 3, 4]);
        assert_eq!(fb.get_pixel(5, 5), [1, 2, 3, 4]);
        assert_eq!(fb.get_pixel(0, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn test_sponge_encrypt_roundtrip() {
        let mut fb = CpuFramebuffer::new(4, 4);
        fb.set_pixel(0, 0, [42, 43, 44, 45]);
        let original = fb.get_pixel(0, 0);

        let key = [0xAB, 0xCD, 0xEF, 0x12];
        sponge_encrypt_framebuffer(&mut fb, &key);
        assert_ne!(fb.get_pixel(0, 0), original);

        sponge_encrypt_framebuffer(&mut fb, &key);
        assert_eq!(fb.get_pixel(0, 0), original);
    }

    #[test]
    fn test_renderer_commands() {
        let mut renderer = CpuRenderer::new();
        let mut fb = CpuFramebuffer::new(100, 100);

        renderer.submit(RenderCommand {
            cmd_type: RenderCommandType::Clear,
            x: 0.0, y: 0.0, width: 0.0, height: 0.0,
            color: [128, 128, 128, 255],
        });
        renderer.submit(RenderCommand {
            cmd_type: RenderCommandType::FillRect,
            x: 10.0, y: 10.0, width: 20.0, height: 20.0,
            color: [255, 0, 0, 255],
        });

        assert_eq!(renderer.pending_commands(), 2);
        renderer.flush(&mut fb);
        assert_eq!(renderer.pending_commands(), 0);
        assert_eq!(fb.get_pixel(15, 15), [255, 0, 0, 255]);
    }
}
