// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// Render backend trait — dispatches to CPU (Phase 1) or GPU (Phase 2).
// The trait boundary is the Phase 1 → Phase 2 seam:
// Phase 2 GPU path swaps in without changing browser or distributor modules.
//
// Pipeline: parse → layout → script → render → encrypt → display
// No separate compositor module — single pipeline.

use alloc::vec::Vec;
use crate::browser::layout::{LayoutBox, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackendType {
    Cpu,
    Gpu,
}

#[derive(Debug, Clone, Copy)]
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl DirtyRect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn full(width: u32, height: u32) -> Self {
        Self { x: 0, y: 0, width, height }
    }

    pub fn union(&self, other: &DirtyRect) -> DirtyRect {
        let x0 = self.x.min(other.x);
        let y0 = self.y.min(other.y);
        let x1 = (self.x + self.width).max(other.x + other.width);
        let y1 = (self.y + self.height).max(other.y + other.height);
        DirtyRect {
            x: x0,
            y: y0,
            width: x1 - x0,
            height: y1 - y0,
        }
    }

    pub fn intersects(&self, other: &DirtyRect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    pub fn from_layout_rect(r: &Rect) -> Self {
        Self {
            x: r.x.max(0.0) as u32,
            y: r.y.max(0.0) as u32,
            width: r.width.max(0.0) as u32,
            height: r.height.max(0.0) as u32,
        }
    }
}

pub const TARGET_FPS: u32 = 30;
pub const FRAME_BUDGET_US: u64 = 1_000_000 / TARGET_FPS as u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
}

#[derive(Debug, Clone, Copy)]
pub struct RenderColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RenderColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn white() -> Self {
        Self { r: 255, g: 255, b: 255, a: 255 }
    }

    pub const fn black() -> Self {
        Self { r: 0, g: 0, b: 0, a: 255 }
    }

    pub const fn transparent() -> Self {
        Self { r: 0, g: 0, b: 0, a: 0 }
    }

    pub fn to_array(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn from_array(arr: [u8; 4]) -> Self {
        Self { r: arr[0], g: arr[1], b: arr[2], a: arr[3] }
    }

    pub fn blend_over(&self, dst: &RenderColor) -> RenderColor {
        if self.a == 255 {
            return *self;
        }
        if self.a == 0 {
            return *dst;
        }
        let sa = self.a as u16;
        let da = 255 - sa;
        RenderColor {
            r: ((self.r as u16 * sa + dst.r as u16 * da) / 255) as u8,
            g: ((self.g as u16 * sa + dst.g as u16 * da) / 255) as u8,
            b: ((self.b as u16 * sa + dst.b as u16 * da) / 255) as u8,
            a: (sa + (dst.a as u16 * da / 255)) as u8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GradientStop {
    pub offset: f32,
    pub color: RenderColor,
}

#[derive(Debug, Clone)]
pub struct LinearGradient {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub stops: Vec<GradientStop>,
}

#[derive(Debug, Clone)]
pub struct RadialGradient {
    pub cx: f32,
    pub cy: f32,
    pub radius: f32,
    pub stops: Vec<GradientStop>,
}

#[derive(Debug, Clone)]
pub enum PaintStyle {
    Solid(RenderColor),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
}

#[derive(Debug, Clone)]
pub struct TextRun {
    pub x: f32,
    pub y: f32,
    pub text: alloc::string::String,
    pub font_size: f32,
    pub color: RenderColor,
    pub font_family: FontFamily,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFamily {
    SansSerif,
    Serif,
    Monospace,
}

#[derive(Debug, Clone)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct SvgData {
    pub content: alloc::string::String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub enum RenderPrimitive {
    FillRect {
        rect: Rect,
        paint: PaintStyle,
    },
    StrokeRect {
        rect: Rect,
        color: RenderColor,
        width: f32,
    },
    DrawText(TextRun),
    DrawImage {
        rect: Rect,
        image: ImageData,
    },
    DrawSvg {
        rect: Rect,
        svg: SvgData,
    },
    Clear(RenderColor),
    ClipRect(Rect),
    PopClip,
}

pub struct RenderScene {
    pub primitives: Vec<RenderPrimitive>,
    pub dirty_regions: Vec<DirtyRect>,
    pub viewport_width: u32,
    pub viewport_height: u32,
}

impl RenderScene {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            primitives: Vec::new(),
            dirty_regions: Vec::new(),
            viewport_width: width,
            viewport_height: height,
        }
    }

    pub fn push(&mut self, prim: RenderPrimitive) {
        self.primitives.push(prim);
    }

    pub fn mark_dirty(&mut self, rect: DirtyRect) {
        self.dirty_regions.push(rect);
    }

    pub fn mark_full_dirty(&mut self) {
        self.dirty_regions.clear();
        self.dirty_regions.push(DirtyRect::full(self.viewport_width, self.viewport_height));
    }

    pub fn clear(&mut self) {
        self.primitives.clear();
        self.dirty_regions.clear();
    }

    pub fn merged_dirty_region(&self) -> Option<DirtyRect> {
        if self.dirty_regions.is_empty() {
            return None;
        }
        let mut merged = self.dirty_regions[0];
        for r in &self.dirty_regions[1..] {
            merged = merged.union(r);
        }
        Some(merged)
    }

    pub fn from_layout_boxes(boxes: &[LayoutBox], viewport_width: u32, viewport_height: u32) -> Self {
        let mut scene = Self::new(viewport_width, viewport_height);

        scene.push(RenderPrimitive::Clear(RenderColor::white()));

        for layout_box in boxes {
            if layout_box.rect.width <= 0.0 || layout_box.rect.height <= 0.0 {
                continue;
            }

            if let Some(text) = &layout_box.text {
                scene.push(RenderPrimitive::DrawText(TextRun {
                    x: layout_box.rect.x,
                    y: layout_box.rect.y + layout_box.font_size,
                    text: text.clone(),
                    font_size: layout_box.font_size,
                    color: RenderColor::black(),
                    font_family: FontFamily::SansSerif,
                }));
            }
        }

        scene.mark_full_dirty();
        scene
    }
}

pub trait RenderBackend {
    fn backend_type(&self) -> RenderBackendType;

    fn render_scene(&mut self, scene: &RenderScene, output: &mut [u8], stride: u32);

    fn render_dirty(&mut self, scene: &RenderScene, output: &mut [u8], stride: u32, dirty: &DirtyRect);

    fn supports_partial_update(&self) -> bool;

    fn max_texture_size(&self) -> u32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirty_rect_union() {
        let a = DirtyRect::new(10, 10, 50, 50);
        let b = DirtyRect::new(40, 40, 60, 60);
        let u = a.union(&b);
        assert_eq!(u.x, 10);
        assert_eq!(u.y, 10);
        assert_eq!(u.width, 90);
        assert_eq!(u.height, 90);
    }

    #[test]
    fn test_dirty_rect_intersects() {
        let a = DirtyRect::new(0, 0, 100, 100);
        let b = DirtyRect::new(50, 50, 100, 100);
        assert!(a.intersects(&b));

        let c = DirtyRect::new(200, 200, 10, 10);
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_render_color_blend() {
        let fg = RenderColor::new(255, 0, 0, 128);
        let bg = RenderColor::white();
        let result = fg.blend_over(&bg);
        assert!(result.r > 100);
        assert!(result.g > 50);
    }

    #[test]
    fn test_render_color_opaque_blend() {
        let fg = RenderColor::new(255, 0, 0, 255);
        let bg = RenderColor::white();
        let result = fg.blend_over(&bg);
        assert_eq!(result.r, 255);
        assert_eq!(result.g, 0);
        assert_eq!(result.b, 0);
    }

    #[test]
    fn test_render_scene_creation() {
        let scene = RenderScene::new(1920, 1080);
        assert_eq!(scene.viewport_width, 1920);
        assert_eq!(scene.viewport_height, 1080);
        assert!(scene.primitives.is_empty());
    }

    #[test]
    fn test_render_scene_dirty_merge() {
        let mut scene = RenderScene::new(800, 600);
        scene.mark_dirty(DirtyRect::new(0, 0, 100, 100));
        scene.mark_dirty(DirtyRect::new(50, 50, 100, 100));
        let merged = scene.merged_dirty_region().unwrap();
        assert_eq!(merged.x, 0);
        assert_eq!(merged.y, 0);
        assert_eq!(merged.width, 150);
        assert_eq!(merged.height, 150);
    }

    #[test]
    fn test_frame_budget() {
        assert_eq!(TARGET_FPS, 30);
        assert!(FRAME_BUDGET_US >= 33000);
        assert!(FRAME_BUDGET_US <= 34000);
    }

    #[test]
    fn test_gradient_stop() {
        let stop = GradientStop {
            offset: 0.5,
            color: RenderColor::new(128, 128, 128, 255),
        };
        assert_eq!(stop.offset, 0.5);
    }

    #[test]
    fn test_font_family_variants() {
        assert_ne!(FontFamily::SansSerif, FontFamily::Serif);
        assert_ne!(FontFamily::Serif, FontFamily::Monospace);
    }
}
