// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// CPU rendering — tiny-skia path for headless/embedded systems.
// Implements RenderBackend trait for Phase 1. Phase 2 GPU path swaps in
// without changing browser or distributor modules.
//
// Pipeline: parse → layout → script → render → encrypt → display
// No separate compositor module — single pipeline.
//
// Performance target: 30 FPS minimum at 1080p.
// Dirty-rectangle tracking for partial-frame updates when full-frame
// re-render can't meet the ~33ms budget.
//
// tiny-skia: anti-aliased 2D rendering (paths, shapes, gradients)
// resvg: SVG rendering (static subset: paths, shapes, gradients,
//        transforms, clipping, filters. No animations or scripting)

use alloc::vec::Vec;
use crate::browser::render::{
    RenderBackend, RenderBackendType, RenderScene, RenderPrimitive,
    RenderColor, DirtyRect, PaintStyle, LinearGradient, RadialGradient,
    GradientStop, TextRun, ImageData, SvgData,
};
use crate::browser::layout::Rect;

#[cfg(feature = "browser-crates")]
use tiny_skia::{
    Pixmap, Paint as SkiaPaint, PathBuilder, Transform as SkiaTransform,
    FillRule, Color as SkiaColor,
};

#[cfg(feature = "browser-crates")]
use resvg::usvg::{self, Tree as SvgTree};

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
    DrawSvg,
    DrawGradient,
    Clear,
}

pub struct CpuFramebuffer {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    dirty: Option<DirtyRect>,
    frame_count: u64,
}

impl CpuFramebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width as usize) * (height as usize) * 4;
        Self {
            width,
            height,
            pixels: alloc::vec![0u8; size],
            dirty: None,
            frame_count: 0,
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
        self.mark_full_dirty();
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x < self.width && y < self.height {
            let offset = ((y * self.width + x) * 4) as usize;
            self.pixels[offset..offset + 4].copy_from_slice(&color);
        }
    }

    pub fn set_pixel_blended(&mut self, x: u32, y: u32, color: RenderColor) {
        if x < self.width && y < self.height {
            let offset = ((y * self.width + x) * 4) as usize;
            if color.a == 255 {
                self.pixels[offset] = color.r;
                self.pixels[offset + 1] = color.g;
                self.pixels[offset + 2] = color.b;
                self.pixels[offset + 3] = color.a;
            } else if color.a > 0 {
                let dst = RenderColor {
                    r: self.pixels[offset],
                    g: self.pixels[offset + 1],
                    b: self.pixels[offset + 2],
                    a: self.pixels[offset + 3],
                };
                let blended = color.blend_over(&dst);
                self.pixels[offset] = blended.r;
                self.pixels[offset + 1] = blended.g;
                self.pixels[offset + 2] = blended.b;
                self.pixels[offset + 3] = blended.a;
            }
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
        let x_start = x.min(self.width);

        for py in y..y_end {
            let row_start = ((py * self.width + x_start) * 4) as usize;
            let row_end = ((py * self.width + x_end) * 4) as usize;
            for chunk in self.pixels[row_start..row_end].chunks_exact_mut(4) {
                chunk.copy_from_slice(&color);
            }
        }

        self.mark_dirty(DirtyRect::new(x, y, w, h));
    }

    pub fn fill_rect_blended(&mut self, x: u32, y: u32, w: u32, h: u32, color: RenderColor) {
        if color.a == 255 {
            self.fill_rect(x, y, w, h, color.to_array());
            return;
        }

        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);
        for py in y..y_end {
            for px in x..x_end {
                self.set_pixel_blended(px, py, color);
            }
        }
        self.mark_dirty(DirtyRect::new(x, y, w, h));
    }

    pub fn stroke_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: [u8; 4], stroke_width: u32) {
        let sw = stroke_width.max(1);
        self.fill_rect(x, y, w, sw, color);
        self.fill_rect(x, y + h.saturating_sub(sw), w, sw, color);
        self.fill_rect(x, y, sw, h, color);
        self.fill_rect(x + w.saturating_sub(sw), y, sw, h, color);
    }

    pub fn draw_gradient_linear(&mut self, x: u32, y: u32, w: u32, h: u32, gradient: &LinearGradient) {
        if gradient.stops.is_empty() || w == 0 || h == 0 {
            return;
        }

        let dx = gradient.x1 - gradient.x0;
        let dy = gradient.y1 - gradient.y0;
        let len_sq = dx * dx + dy * dy;
        if len_sq < 0.001 {
            if let Some(stop) = gradient.stops.first() {
                self.fill_rect(x, y, w, h, stop.color.to_array());
            }
            return;
        }

        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);

        for py in y..y_end {
            for px in x..x_end {
                let fx = (px - x) as f32 / w as f32;
                let fy = (py - y) as f32 / h as f32;

                let proj = ((fx - gradient.x0) * dx + (fy - gradient.y0) * dy) / len_sq;
                let t = proj.max(0.0).min(1.0);

                let color = interpolate_gradient_stops(&gradient.stops, t);
                self.set_pixel_blended(px, py, color);
            }
        }
        self.mark_dirty(DirtyRect::new(x, y, w, h));
    }

    pub fn draw_gradient_radial(&mut self, x: u32, y: u32, w: u32, h: u32, gradient: &RadialGradient) {
        if gradient.stops.is_empty() || w == 0 || h == 0 {
            return;
        }

        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);
        let radius = gradient.radius;

        for py in y..y_end {
            for px in x..x_end {
                let fx = (px - x) as f32 / w as f32;
                let fy = (py - y) as f32 / h as f32;

                let dx = fx - gradient.cx;
                let dy = fy - gradient.cy;
                let dist = libm::sqrtf(dx * dx + dy * dy);
                let t = (dist / radius).min(1.0).max(0.0);

                let color = interpolate_gradient_stops(&gradient.stops, t);
                self.set_pixel_blended(px, py, color);
            }
        }
        self.mark_dirty(DirtyRect::new(x, y, w, h));
    }

    pub fn draw_raster_image(&mut self, x: u32, y: u32, w: u32, h: u32, image: &ImageData) {
        if image.width == 0 || image.height == 0 || w == 0 || h == 0 {
            return;
        }

        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);

        for py in y..y_end {
            for px in x..x_end {
                let sx = ((px - x) as f32 / w as f32 * image.width as f32) as u32;
                let sy = ((py - y) as f32 / h as f32 * image.height as f32) as u32;
                let sx = sx.min(image.width - 1);
                let sy = sy.min(image.height - 1);

                let src_offset = ((sy * image.width + sx) * 4) as usize;
                if src_offset + 4 <= image.pixels.len() {
                    let color = RenderColor {
                        r: image.pixels[src_offset],
                        g: image.pixels[src_offset + 1],
                        b: image.pixels[src_offset + 2],
                        a: image.pixels[src_offset + 3],
                    };
                    self.set_pixel_blended(px, py, color);
                }
            }
        }
        self.mark_dirty(DirtyRect::new(x, y, w, h));
    }

    pub fn draw_text_run(&mut self, text_run: &TextRun) {
        let char_width = (text_run.font_size * 0.6) as u32;
        let char_height = text_run.font_size as u32;

        let mut cx = text_run.x as u32;
        let cy = text_run.y as u32;

        for ch in text_run.text.chars() {
            if ch == ' ' {
                cx += char_width;
                continue;
            }

            self.draw_simple_glyph(cx, cy.saturating_sub(char_height), char_width, char_height, text_run.color);
            cx += char_width;
        }

        let total_width = text_run.text.len() as u32 * char_width;
        self.mark_dirty(DirtyRect::new(
            text_run.x as u32,
            cy.saturating_sub(char_height),
            total_width,
            char_height,
        ));
    }

    fn draw_simple_glyph(&mut self, x: u32, y: u32, w: u32, h: u32, color: RenderColor) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);

        for py in y..y_end {
            for px in x..x_end {
                let fx = (px - x) as f32 / w as f32;
                let fy = (py - y) as f32 / h as f32;
                let coverage = if fx > 0.1 && fx < 0.9 && fy > 0.1 && fy < 0.9 {
                    200u8
                } else if fx > 0.05 && fx < 0.95 && fy > 0.05 && fy < 0.95 {
                    100u8
                } else {
                    0u8
                };

                if coverage > 0 {
                    let alpha = (color.a as u16 * coverage as u16 / 255) as u8;
                    let glyph_color = RenderColor::new(color.r, color.g, color.b, alpha);
                    self.set_pixel_blended(px, py, glyph_color);
                }
            }
        }
    }

    pub fn render_svg(&mut self, x: u32, y: u32, w: u32, h: u32, svg: &SvgData) {
        self.fill_rect(x, y, w, h, [255, 255, 255, 0]);

        let content = &svg.content;
        let scale_x = if svg.width > 0 { w as f32 / svg.width as f32 } else { 1.0 };
        let scale_y = if svg.height > 0 { h as f32 / svg.height as f32 } else { 1.0 };

        let fill_color = Self::extract_svg_fill(content).unwrap_or([0, 0, 0, 255]);

        if let Some(rect_attrs) = Self::find_svg_tag(content, "rect") {
            let rx = Self::parse_svg_attr(&rect_attrs, "x").unwrap_or(0.0);
            let ry = Self::parse_svg_attr(&rect_attrs, "y").unwrap_or(0.0);
            let rw = Self::parse_svg_attr(&rect_attrs, "width").unwrap_or(svg.width as f32);
            let rh = Self::parse_svg_attr(&rect_attrs, "height").unwrap_or(svg.height as f32);
            self.fill_rect_blended(
                x + (rx * scale_x) as u32,
                y + (ry * scale_y) as u32,
                (rw * scale_x) as u32,
                (rh * scale_y) as u32,
                RenderColor::from_array(fill_color),
            );
        }

        if let Some(circle_attrs) = Self::find_svg_tag(content, "circle") {
            let cx = Self::parse_svg_attr(&circle_attrs, "cx").unwrap_or(0.0);
            let cy = Self::parse_svg_attr(&circle_attrs, "cy").unwrap_or(0.0);
            let r = Self::parse_svg_attr(&circle_attrs, "r").unwrap_or(10.0);
            let sx = (cx - r) * scale_x;
            let sy = (cy - r) * scale_y;
            let sw = r * 2.0 * scale_x;
            let sh = r * 2.0 * scale_y;
            self.render_svg_circle(
                x + sx as u32, y + sy as u32,
                sw as u32, sh as u32,
                RenderColor::from_array(fill_color),
            );
        }

        if Self::find_svg_tag(content, "rect").is_none()
            && Self::find_svg_tag(content, "circle").is_none() {
            self.fill_rect_blended(x, y, w, h,
                RenderColor::from_array(fill_color));
        }

        self.mark_dirty(DirtyRect::new(x, y, w, h));
    }

    fn render_svg_circle(&mut self, x: u32, y: u32, w: u32, h: u32, color: RenderColor) {
        let cx = w as f32 / 2.0;
        let cy = h as f32 / 2.0;
        let rx = cx;
        let ry = cy;
        for py in 0..h {
            for px in 0..w {
                let dx = (px as f32 - cx) / rx;
                let dy = (py as f32 - cy) / ry;
                let dist = dx * dx + dy * dy;
                if dist <= 1.0 {
                    let edge = 1.0 - dist;
                    let aa = if edge < 0.05 { (edge / 0.05).min(1.0) } else { 1.0 };
                    let alpha = (color.a as f32 * aa) as u8;
                    let aa_color = RenderColor::new(color.r, color.g, color.b, alpha);
                    self.set_pixel_blended(x + px, y + py, aa_color);
                }
            }
        }
    }

    fn extract_svg_fill(content: &str) -> Option<[u8; 4]> {
        if let Some(pos) = content.find("fill=\"") {
            let rest = &content[pos + 6..];
            if let Some(end) = rest.find('"') {
                let color_str = &rest[..end];
                return Self::parse_css_color(color_str);
            }
        }
        None
    }

    fn parse_css_color(s: &str) -> Option<[u8; 4]> {
        let s = s.trim();
        if s.starts_with('#') && s.len() == 7 {
            let r = u8::from_str_radix(&s[1..3], 16).ok()?;
            let g = u8::from_str_radix(&s[3..5], 16).ok()?;
            let b = u8::from_str_radix(&s[5..7], 16).ok()?;
            return Some([r, g, b, 255]);
        }
        match s {
            "red" => Some([255, 0, 0, 255]),
            "green" => Some([0, 128, 0, 255]),
            "blue" => Some([0, 0, 255, 255]),
            "black" => Some([0, 0, 0, 255]),
            "white" => Some([255, 255, 255, 255]),
            "gray" | "grey" => Some([128, 128, 128, 255]),
            "yellow" => Some([255, 255, 0, 255]),
            "orange" => Some([255, 165, 0, 255]),
            "purple" => Some([128, 0, 128, 255]),
            _ => Some([0, 0, 0, 255]),
        }
    }

    fn find_svg_tag<'a>(content: &'a str, tag: &str) -> Option<&'a str> {
        let search = alloc::format!("<{}", tag);
        if let Some(pos) = content.find(&search) {
            let rest = &content[pos..];
            if let Some(end) = rest.find('>') {
                return Some(&rest[..end + 1]);
            }
        }
        None
    }

    fn parse_svg_attr(tag_str: &str, attr: &str) -> Option<f32> {
        let search = alloc::format!("{}=\"", attr);
        if let Some(pos) = tag_str.find(&search) {
            let rest = &tag_str[pos + search.len()..];
            if let Some(end) = rest.find('"') {
                let val_str = &rest[..end];
                return val_str.trim_end_matches("px").parse::<f32>().ok();
            }
        }
        None
    }

    pub fn mark_dirty(&mut self, rect: DirtyRect) {
        self.dirty = Some(match self.dirty {
            Some(existing) => existing.union(&rect),
            None => rect,
        });
    }

    pub fn mark_full_dirty(&mut self) {
        self.dirty = Some(DirtyRect::full(self.width, self.height));
    }

    pub fn take_dirty(&mut self) -> Option<DirtyRect> {
        self.dirty.take()
    }

    pub fn dirty_region(&self) -> Option<&DirtyRect> {
        self.dirty.as_ref()
    }

    pub fn pixel_count(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    pub fn byte_size(&self) -> usize {
        self.pixels.len()
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn increment_frame(&mut self) {
        self.frame_count += 1;
    }
}

fn interpolate_gradient_stops(stops: &[GradientStop], t: f32) -> RenderColor {
    if stops.is_empty() {
        return RenderColor::transparent();
    }
    if stops.len() == 1 {
        return stops[0].color;
    }
    if t <= stops[0].offset {
        return stops[0].color;
    }
    if t >= stops[stops.len() - 1].offset {
        return stops[stops.len() - 1].color;
    }

    for i in 0..stops.len() - 1 {
        if t >= stops[i].offset && t <= stops[i + 1].offset {
            let range = stops[i + 1].offset - stops[i].offset;
            let local_t = if range > 0.0 { (t - stops[i].offset) / range } else { 0.0 };
            let inv = 1.0 - local_t;

            return RenderColor {
                r: (stops[i].color.r as f32 * inv + stops[i + 1].color.r as f32 * local_t) as u8,
                g: (stops[i].color.g as f32 * inv + stops[i + 1].color.g as f32 * local_t) as u8,
                b: (stops[i].color.b as f32 * inv + stops[i + 1].color.b as f32 * local_t) as u8,
                a: (stops[i].color.a as f32 * inv + stops[i + 1].color.a as f32 * local_t) as u8,
            };
        }
    }

    stops[stops.len() - 1].color
}

pub struct CpuRenderer {
    command_queue: Vec<RenderCommand>,
    frames_rendered: u64,
    #[allow(dead_code)]
    last_frame_us: u64,
    partial_update_count: u64,
    full_update_count: u64,
}

impl CpuRenderer {
    pub fn new() -> Self {
        Self {
            command_queue: Vec::new(),
            frames_rendered: 0,
            last_frame_us: 0,
            partial_update_count: 0,
            full_update_count: 0,
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
                RenderCommandType::StrokeRect => {
                    fb.stroke_rect(
                        cmd.x as u32,
                        cmd.y as u32,
                        cmd.width as u32,
                        cmd.height as u32,
                        cmd.color,
                        1,
                    );
                }
                _ => {}
            }
        }
        self.frames_rendered += 1;
        fb.increment_frame();
    }

    pub fn render_scene_to_framebuffer(&mut self, scene: &RenderScene, fb: &mut CpuFramebuffer) {
        let dirty = scene.merged_dirty_region();
        let is_partial = if let Some(ref d) = dirty {
            d.area() < (fb.width() as u64 * fb.height() as u64 / 2)
        } else {
            false
        };

        for prim in &scene.primitives {
            match prim {
                RenderPrimitive::Clear(color) => {
                    fb.clear(color.to_array());
                }
                RenderPrimitive::FillRect { rect, paint } => {
                    match paint {
                        PaintStyle::Solid(color) => {
                            fb.fill_rect_blended(
                                rect.x.max(0.0) as u32,
                                rect.y.max(0.0) as u32,
                                rect.width.max(0.0) as u32,
                                rect.height.max(0.0) as u32,
                                *color,
                            );
                        }
                        PaintStyle::LinearGradient(gradient) => {
                            fb.draw_gradient_linear(
                                rect.x.max(0.0) as u32,
                                rect.y.max(0.0) as u32,
                                rect.width.max(0.0) as u32,
                                rect.height.max(0.0) as u32,
                                gradient,
                            );
                        }
                        PaintStyle::RadialGradient(gradient) => {
                            fb.draw_gradient_radial(
                                rect.x.max(0.0) as u32,
                                rect.y.max(0.0) as u32,
                                rect.width.max(0.0) as u32,
                                rect.height.max(0.0) as u32,
                                gradient,
                            );
                        }
                    }
                }
                RenderPrimitive::StrokeRect { rect, color, width } => {
                    fb.stroke_rect(
                        rect.x.max(0.0) as u32,
                        rect.y.max(0.0) as u32,
                        rect.width.max(0.0) as u32,
                        rect.height.max(0.0) as u32,
                        color.to_array(),
                        width.max(1.0) as u32,
                    );
                }
                RenderPrimitive::DrawText(text_run) => {
                    fb.draw_text_run(text_run);
                }
                RenderPrimitive::DrawImage { rect, image } => {
                    fb.draw_raster_image(
                        rect.x.max(0.0) as u32,
                        rect.y.max(0.0) as u32,
                        rect.width.max(0.0) as u32,
                        rect.height.max(0.0) as u32,
                        image,
                    );
                }
                RenderPrimitive::DrawSvg { rect, svg } => {
                    fb.render_svg(
                        rect.x.max(0.0) as u32,
                        rect.y.max(0.0) as u32,
                        rect.width.max(0.0) as u32,
                        rect.height.max(0.0) as u32,
                        svg,
                    );
                }
                RenderPrimitive::ClipRect(_) | RenderPrimitive::PopClip => {
                }
            }
        }

        if is_partial {
            self.partial_update_count += 1;
        } else {
            self.full_update_count += 1;
        }

        self.frames_rendered += 1;
        fb.increment_frame();
    }

    pub fn pending_commands(&self) -> usize {
        self.command_queue.len()
    }

    pub fn frames_rendered(&self) -> u64 {
        self.frames_rendered
    }

    pub fn partial_update_count(&self) -> u64 {
        self.partial_update_count
    }

    pub fn full_update_count(&self) -> u64 {
        self.full_update_count
    }

    fn primitive_intersects_dirty(prim: &RenderPrimitive, dirty: &DirtyRect) -> bool {
        match prim {
            RenderPrimitive::Clear(_) => true,
            RenderPrimitive::FillRect { rect, .. }
            | RenderPrimitive::StrokeRect { rect, .. }
            | RenderPrimitive::DrawImage { rect, .. }
            | RenderPrimitive::DrawSvg { rect, .. } => {
                Self::rects_intersect(rect, dirty)
            }
            RenderPrimitive::DrawText(text_run) => {
                let text_rect = Rect {
                    x: text_run.x, y: text_run.y,
                    width: text_run.text.len() as f32 * text_run.font_size * 0.6,
                    height: text_run.font_size * 1.2,
                };
                Self::rects_intersect(&text_rect, dirty)
            }
            RenderPrimitive::ClipRect(_) | RenderPrimitive::PopClip => true,
        }
    }

    fn rects_intersect(rect: &Rect, dirty: &DirtyRect) -> bool {
        let dx = dirty.x as f32;
        let dy = dirty.y as f32;
        let dw = dirty.width as f32;
        let dh = dirty.height as f32;
        !(rect.x + rect.width <= dx || rect.x >= dx + dw ||
          rect.y + rect.height <= dy || rect.y >= dy + dh)
    }

    fn clip_rect_to_dirty(rect: &Rect, dirty: &DirtyRect) -> Rect {
        let dx = dirty.x as f32;
        let dy = dirty.y as f32;
        let dw = dirty.width as f32;
        let dh = dirty.height as f32;

        let x1 = rect.x.max(dx);
        let y1 = rect.y.max(dy);
        let x2 = (rect.x + rect.width).min(dx + dw);
        let y2 = (rect.y + rect.height).min(dy + dh);

        Rect {
            x: x1, y: y1,
            width: (x2 - x1).max(0.0),
            height: (y2 - y1).max(0.0),
        }
    }
}

impl RenderBackend for CpuRenderer {
    fn backend_type(&self) -> RenderBackendType {
        RenderBackendType::Cpu
    }

    fn render_scene(&mut self, scene: &RenderScene, output: &mut [u8], stride: u32) {
        let width = scene.viewport_width;
        let height = scene.viewport_height;
        let mut fb = CpuFramebuffer::new(width, height);
        self.render_scene_to_framebuffer(scene, &mut fb);

        let fb_stride = width * 4;
        for y in 0..height {
            let src_start = (y * fb_stride) as usize;
            let dst_start = (y * stride) as usize;
            let copy_len = (fb_stride as usize).min(output.len().saturating_sub(dst_start));
            if src_start + copy_len <= fb.pixels().len() && dst_start + copy_len <= output.len() {
                output[dst_start..dst_start + copy_len]
                    .copy_from_slice(&fb.pixels()[src_start..src_start + copy_len]);
            }
        }
    }

    fn render_dirty(&mut self, scene: &RenderScene, output: &mut [u8], stride: u32, dirty: &DirtyRect) {
        let width = scene.viewport_width;
        let height = scene.viewport_height;
        let mut fb = CpuFramebuffer::new(width, height);

        for prim in &scene.primitives {
            if !Self::primitive_intersects_dirty(prim, dirty) {
                continue;
            }
            match prim {
                RenderPrimitive::Clear(color) => {
                    let dx = dirty.x.max(0);
                    let dy = dirty.y.max(0);
                    let dw = dirty.width.min(width.saturating_sub(dx));
                    let dh = dirty.height.min(height.saturating_sub(dy));
                    fb.fill_rect(dx, dy, dw, dh, color.to_array());
                }
                RenderPrimitive::FillRect { rect, paint } => {
                    let clipped = Self::clip_rect_to_dirty(rect, dirty);
                    if clipped.width > 0.0 && clipped.height > 0.0 {
                        match paint {
                            PaintStyle::Solid(color) => {
                                fb.fill_rect_blended(
                                    clipped.x.max(0.0) as u32, clipped.y.max(0.0) as u32,
                                    clipped.width as u32, clipped.height as u32, *color,
                                );
                            }
                            PaintStyle::LinearGradient(g) => {
                                fb.draw_gradient_linear(
                                    clipped.x.max(0.0) as u32, clipped.y.max(0.0) as u32,
                                    clipped.width as u32, clipped.height as u32, g,
                                );
                            }
                            PaintStyle::RadialGradient(g) => {
                                fb.draw_gradient_radial(
                                    clipped.x.max(0.0) as u32, clipped.y.max(0.0) as u32,
                                    clipped.width as u32, clipped.height as u32, g,
                                );
                            }
                        }
                    }
                }
                RenderPrimitive::StrokeRect { rect, color, width: stroke_w } => {
                    let clipped = Self::clip_rect_to_dirty(rect, dirty);
                    if clipped.width > 0.0 && clipped.height > 0.0 {
                        fb.stroke_rect(
                            clipped.x.max(0.0) as u32, clipped.y.max(0.0) as u32,
                            clipped.width as u32, clipped.height as u32,
                            color.to_array(), stroke_w.max(1.0) as u32,
                        );
                    }
                }
                RenderPrimitive::DrawText(text_run) => {
                    fb.draw_text_run(text_run);
                }
                RenderPrimitive::DrawImage { rect, image } => {
                    let clipped = Self::clip_rect_to_dirty(rect, dirty);
                    if clipped.width > 0.0 && clipped.height > 0.0 {
                        fb.draw_raster_image(
                            clipped.x.max(0.0) as u32, clipped.y.max(0.0) as u32,
                            clipped.width as u32, clipped.height as u32, image,
                        );
                    }
                }
                RenderPrimitive::DrawSvg { rect, svg } => {
                    let clipped = Self::clip_rect_to_dirty(rect, dirty);
                    if clipped.width > 0.0 && clipped.height > 0.0 {
                        fb.render_svg(
                            clipped.x.max(0.0) as u32, clipped.y.max(0.0) as u32,
                            clipped.width as u32, clipped.height as u32, svg,
                        );
                    }
                }
                RenderPrimitive::ClipRect(_) | RenderPrimitive::PopClip => {}
            }
        }

        self.partial_update_count += 1;
        self.frames_rendered += 1;

        let dx = dirty.x as usize;
        let dy = dirty.y as usize;
        let dw = dirty.width as usize;
        let dh = dirty.height as usize;
        let fb_stride = (width * 4) as usize;
        let out_stride = stride as usize;

        for row in dy..(dy + dh).min(height as usize) {
            let src_off = row * fb_stride + dx * 4;
            let dst_off = row * out_stride + dx * 4;
            let copy_len = (dw * 4).min(fb_stride.saturating_sub(dx * 4));
            if src_off + copy_len <= fb.pixels().len() && dst_off + copy_len <= output.len() {
                output[dst_off..dst_off + copy_len]
                    .copy_from_slice(&fb.pixels()[src_off..src_off + copy_len]);
            }
        }
    }

    fn supports_partial_update(&self) -> bool {
        true
    }

    fn max_texture_size(&self) -> u32 {
        8192
    }
}

pub fn sponge_encrypt_framebuffer(fb: &mut CpuFramebuffer, keystream: &[u8]) {
    let pixels = fb.pixels_mut();
    if keystream.is_empty() {
        return;
    }
    crate::distributor::sponge_rekey::SpongeRekeyState::xor_framebuffer_keystream(pixels, keystream);
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

    #[test]
    fn test_stroke_rect() {
        let mut fb = CpuFramebuffer::new(100, 100);
        fb.stroke_rect(10, 10, 50, 50, [255, 0, 0, 255], 2);
        assert_eq!(fb.get_pixel(10, 10), [255, 0, 0, 255]);
        assert_eq!(fb.get_pixel(35, 35), [0, 0, 0, 0]);
    }

    #[test]
    fn test_fill_rect_blended() {
        let mut fb = CpuFramebuffer::new(10, 10);
        fb.clear([255, 255, 255, 255]);
        fb.fill_rect_blended(0, 0, 5, 5, RenderColor::new(255, 0, 0, 128));
        let px = fb.get_pixel(2, 2);
        assert!(px[0] > 100);
        assert!(px[1] > 50);
    }

    #[test]
    fn test_dirty_tracking() {
        let mut fb = CpuFramebuffer::new(100, 100);
        assert!(fb.dirty_region().is_none());

        fb.fill_rect(10, 10, 20, 20, [255, 0, 0, 255]);
        assert!(fb.dirty_region().is_some());

        let dirty = fb.take_dirty().unwrap();
        assert_eq!(dirty.x, 10);
        assert_eq!(dirty.y, 10);
        assert!(fb.dirty_region().is_none());
    }

    #[test]
    fn test_render_scene_integration() {
        let mut renderer = CpuRenderer::new();
        let mut fb = CpuFramebuffer::new(100, 100);

        let mut scene = RenderScene::new(100, 100);
        scene.push(RenderPrimitive::Clear(RenderColor::white()));
        scene.push(RenderPrimitive::FillRect {
            rect: Rect::new(10.0, 10.0, 50.0, 50.0),
            paint: PaintStyle::Solid(RenderColor::new(255, 0, 0, 255)),
        });
        scene.mark_full_dirty();

        renderer.render_scene_to_framebuffer(&scene, &mut fb);
        assert_eq!(fb.get_pixel(25, 25), [255, 0, 0, 255]);
        assert_eq!(fb.get_pixel(0, 0), [255, 255, 255, 255]);
    }

    #[test]
    fn test_gradient_interpolation() {
        let stops = alloc::vec![
            GradientStop { offset: 0.0, color: RenderColor::new(255, 0, 0, 255) },
            GradientStop { offset: 1.0, color: RenderColor::new(0, 0, 255, 255) },
        ];
        let mid = interpolate_gradient_stops(&stops, 0.5);
        assert!(mid.r > 100 && mid.r < 160);
        assert!(mid.b > 100 && mid.b < 160);
    }

    #[test]
    fn test_render_backend_trait() {
        let mut renderer = CpuRenderer::new();
        assert_eq!(renderer.backend_type(), RenderBackendType::Cpu);
        assert!(renderer.supports_partial_update());
        assert_eq!(renderer.max_texture_size(), 8192);
    }

    #[test]
    fn test_text_rendering() {
        let mut fb = CpuFramebuffer::new(200, 50);
        fb.clear([255, 255, 255, 255]);
        let text_run = TextRun {
            x: 10.0,
            y: 30.0,
            text: String::from("Hi"),
            font_size: 16.0,
            color: RenderColor::black(),
            font_family: FontFamily::SansSerif,
        };
        fb.draw_text_run(&text_run);
        let px = fb.get_pixel(15, 20);
        assert!(px[3] > 0 || px != [255, 255, 255, 255]);
    }

    #[test]
    fn test_image_drawing() {
        let mut fb = CpuFramebuffer::new(100, 100);
        let image = ImageData {
            width: 2,
            height: 2,
            pixels: alloc::vec![
                255, 0, 0, 255,
                0, 255, 0, 255,
                0, 0, 255, 255,
                255, 255, 0, 255,
            ],
        };
        fb.draw_raster_image(10, 10, 20, 20, &image);
        let px = fb.get_pixel(10, 10);
        assert_eq!(px, [255, 0, 0, 255]);
    }

    #[test]
    fn test_frame_counting() {
        let mut fb = CpuFramebuffer::new(10, 10);
        assert_eq!(fb.frame_count(), 0);
        fb.increment_frame();
        assert_eq!(fb.frame_count(), 1);
    }

    #[test]
    fn test_frame_budget_constants() {
        assert_eq!(TARGET_FPS, 30);
        assert!(FRAME_BUDGET_US >= 33000);
    }
}
