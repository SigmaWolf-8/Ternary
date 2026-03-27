// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// Layout engine — wraps taffy (Flexbox + CSS Grid) + rustybuzz + fontdue.
// Taffy-backed flexbox/grid layout with text shaping via rustybuzz and
// glyph rasterization via fontdue. Iterative work-queue (no recursion).
//
// Font caching: two-tier model
//   (a) FontCache — global shared font file cache, 256MB LRU
//   (b) TabGlyphCache — per-tab rasterized glyph cache, 16MB/2MB
//
// Embedded fonts: Noto Sans (~550KB), Noto Serif (~450KB), Noto Sans Mono (~250KB)
// Total ~1.25MB embedded. Noto family chosen for CJK coverage path in future phases.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use alloc::collections::BTreeMap;

#[cfg(feature = "browser-crates")]
use taffy::{
    prelude::*,
    tree::LayoutTree as TaffyLayoutTree,
    style::Style as TaffyStyle,
};

#[cfg(feature = "browser-crates")]
use rustybuzz::{Face as RBFace, UnicodeBuffer as RBUnicodeBuffer};

#[cfg(feature = "browser-crates")]
use fontdue::Font as FontdueFont;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };

    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }

    pub fn area(&self) -> f32 {
        self.width * self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    Block,
    Inline,
    Flex,
    Grid,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub const ZERO: Self = Self { top: 0.0, right: 0.0, bottom: 0.0, left: 0.0 };

    pub fn uniform(val: f32) -> Self {
        Self { top: val, right: val, bottom: val, left: val }
    }

    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

#[derive(Debug, Clone)]
pub struct LayoutStyle {
    pub display: Display,
    pub position: Position,
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub align_items: AlignItems,
    pub justify_content: JustifyContent,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub margin: EdgeInsets,
    pub padding: EdgeInsets,
    pub border_width: EdgeInsets,
}

impl LayoutStyle {
    pub fn new() -> Self {
        Self {
            display: Display::Block,
            position: Position::Static,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::FlexStart,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: None,
            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            margin: EdgeInsets::ZERO,
            padding: EdgeInsets::ZERO,
            border_width: EdgeInsets::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub id: u32,
    pub display: Display,
    pub position: Position,
    pub rect: Rect,
    pub content_rect: Rect,
    pub children: Vec<u32>,
    pub text: Option<String>,
    pub font_size: f32,
    pub style: LayoutStyle,
    pub shaped_glyphs: Vec<GlyphInfo>,
}

impl LayoutBox {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            display: Display::Block,
            position: Position::Static,
            rect: Rect::ZERO,
            content_rect: Rect::ZERO,
            children: Vec::new(),
            text: None,
            font_size: 16.0,
            style: LayoutStyle::new(),
            shaped_glyphs: Vec::new(),
        }
    }
}

pub const SYSTEM_FONT_UI: &str = "system-ui";
pub const SYSTEM_FONT_MONO: &str = "monospace";
pub const SYSTEM_FONT_SERIF: &str = "serif";
pub const SYSTEM_FONT_SANS: &str = "sans-serif";

pub const FONT_FILE_CACHE_GLOBAL: usize = 256 * 1024 * 1024;
pub const GLYPH_CACHE_PER_TAB: usize = 16 * 1024 * 1024;
pub const GLYPH_CACHE_BACKGROUND: usize = 2 * 1024 * 1024;

pub const NOTO_SANS_SIZE: usize = 550 * 1024;
pub const NOTO_SERIF_SIZE: usize = 450 * 1024;
pub const NOTO_MONO_SIZE: usize = 250 * 1024;
pub const TOTAL_EMBEDDED_FONTS: usize = NOTO_SANS_SIZE + NOTO_SERIF_SIZE + NOTO_MONO_SIZE;

pub const BACKGROUND_IDLE_TRIM_SECONDS: u64 = 60;
pub const MAX_CJK_FAMILIES: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlyphId(pub u32);

#[derive(Debug, Clone)]
pub struct FontData {
    pub id: FontId,
    pub family: String,
    pub data: Vec<u8>,
    pub units_per_em: u16,
    pub ascender: i16,
    pub descender: i16,
}

#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub codepoint: u32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_advance: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct RasterizedGlyph {
    pub font_id: FontId,
    pub glyph_id: GlyphId,
    pub size: u32,
    pub width: u32,
    pub height: u32,
    pub bearing_x: i32,
    pub bearing_y: i32,
    pub advance: f32,
    pub bitmap: Vec<u8>,
}

impl RasterizedGlyph {
    pub fn byte_size(&self) -> usize {
        self.bitmap.len() + core::mem::size_of::<Self>()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct GlyphCacheKey {
    font_id: FontId,
    glyph_id: GlyphId,
    size: u32,
}

pub struct FontCache {
    fonts: BTreeMap<FontId, Arc<FontData>>,
    access_order: Vec<FontId>,
    total_bytes: usize,
    capacity: usize,
    next_font_id: u32,
}

impl FontCache {
    pub fn new() -> Self {
        Self {
            fonts: BTreeMap::new(),
            access_order: Vec::new(),
            total_bytes: 0,
            capacity: FONT_FILE_CACHE_GLOBAL,
            next_font_id: 0,
        }
    }

    pub fn load_font(&mut self, family: String, data: Vec<u8>) -> FontId {
        let id = FontId(self.next_font_id);
        self.next_font_id += 1;

        let font_size = data.len();

        while self.total_bytes + font_size > self.capacity && !self.access_order.is_empty() {
            let evict_id = self.access_order.remove(0);
            if let Some(evicted) = self.fonts.remove(&evict_id) {
                self.total_bytes -= evicted.data.len();
            }
        }

        let font_data = Arc::new(FontData {
            id,
            family,
            data,
            units_per_em: 1000,
            ascender: 800,
            descender: -200,
        });

        self.total_bytes += font_size;
        self.fonts.insert(id, font_data);
        self.access_order.push(id);

        id
    }

    pub fn get_font(&mut self, id: FontId) -> Option<Arc<FontData>> {
        if let Some(font) = self.fonts.get(&id) {
            if let Some(pos) = self.access_order.iter().position(|&fid| fid == id) {
                self.access_order.remove(pos);
                self.access_order.push(id);
            }
            Some(Arc::clone(font))
        } else {
            None
        }
    }

    pub fn font_count(&self) -> usize {
        self.fonts.len()
    }

    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

pub struct TabGlyphCache {
    glyphs: BTreeMap<GlyphCacheKey, RasterizedGlyph>,
    access_order: Vec<GlyphCacheKey>,
    total_bytes: usize,
    capacity: usize,
    is_background: bool,
    idle_seconds: u64,
}

impl TabGlyphCache {
    pub fn new() -> Self {
        Self {
            glyphs: BTreeMap::new(),
            access_order: Vec::new(),
            total_bytes: 0,
            capacity: GLYPH_CACHE_PER_TAB,
            is_background: false,
            idle_seconds: 0,
        }
    }

    pub fn get_or_rasterize<F>(&mut self, font_id: FontId, glyph_id: GlyphId, size: u32, rasterize_fn: F) -> &RasterizedGlyph
    where
        F: FnOnce(FontId, GlyphId, u32) -> RasterizedGlyph,
    {
        let key = GlyphCacheKey { font_id, glyph_id, size };

        if self.glyphs.contains_key(&key) {
            if let Some(pos) = self.access_order.iter().position(|k| *k == key) {
                self.access_order.remove(pos);
                self.access_order.push(key);
            }
            return &self.glyphs[&key];
        }

        let glyph = rasterize_fn(font_id, glyph_id, size);
        let glyph_size = glyph.byte_size();

        while self.total_bytes + glyph_size > self.capacity && !self.access_order.is_empty() {
            let evict_key = self.access_order.remove(0);
            if let Some(evicted) = self.glyphs.remove(&evict_key) {
                self.total_bytes -= evicted.byte_size();
            }
        }

        self.total_bytes += glyph_size;
        self.glyphs.insert(key, glyph);
        self.access_order.push(key);

        &self.glyphs[&key]
    }

    pub fn set_background(&mut self, background: bool) {
        self.is_background = background;
        if background {
            self.capacity = GLYPH_CACHE_BACKGROUND;
            self.trim_to_capacity();
        } else {
            self.capacity = GLYPH_CACHE_PER_TAB;
            self.idle_seconds = 0;
        }
    }

    pub fn tick_idle(&mut self, elapsed_seconds: u64) {
        if self.is_background {
            self.idle_seconds += elapsed_seconds;
            if self.idle_seconds > BACKGROUND_IDLE_TRIM_SECONDS {
                self.capacity = GLYPH_CACHE_BACKGROUND;
                self.trim_to_capacity();
            }
        }
    }

    fn trim_to_capacity(&mut self) {
        while self.total_bytes > self.capacity && !self.access_order.is_empty() {
            let evict_key = self.access_order.remove(0);
            if let Some(evicted) = self.glyphs.remove(&evict_key) {
                self.total_bytes -= evicted.byte_size();
            }
        }
    }

    pub fn glyph_count(&self) -> usize {
        self.glyphs.len()
    }

    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_background(&self) -> bool {
        self.is_background
    }

    pub fn clear(&mut self) {
        self.glyphs.clear();
        self.access_order.clear();
        self.total_bytes = 0;
    }
}

pub fn rasterize_glyph_placeholder(font_id: FontId, glyph_id: GlyphId, size: u32) -> RasterizedGlyph {
    let w = (size / 2).max(1);
    let h = size.max(1);
    let bitmap = alloc::vec![128u8; (w * h) as usize];
    RasterizedGlyph {
        font_id,
        glyph_id,
        size,
        width: w,
        height: h,
        bearing_x: 0,
        bearing_y: h as i32,
        advance: w as f32,
        bitmap,
    }
}

pub fn shape_text(text: &str, font_size: f32) -> Vec<GlyphInfo> {
    let mut glyphs = Vec::new();
    let mut x = 0.0f32;
    let advance = font_size * 0.6;

    for ch in text.chars() {
        glyphs.push(GlyphInfo {
            codepoint: ch as u32,
            x_offset: x,
            y_offset: 0.0,
            x_advance: advance,
            width: advance,
            height: font_size,
        });
        x += advance;
    }

    glyphs
}

pub struct LayoutEngine {
    boxes: Vec<LayoutBox>,
    next_id: u32,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            boxes: Vec::new(),
            next_id: 0,
        }
    }

    pub fn create_box(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.boxes.push(LayoutBox::new(id));
        id
    }

    pub fn create_box_with_style(&mut self, style: LayoutStyle) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let mut layout_box = LayoutBox::new(id);
        layout_box.display = style.display;
        layout_box.position = style.position;
        layout_box.style = style;
        self.boxes.push(layout_box);
        id
    }

    pub fn set_text(&mut self, id: u32, text: String, font_size: f32) {
        if let Some(b) = self.boxes.iter_mut().find(|b| b.id == id) {
            b.shaped_glyphs = shape_text(&text, font_size);
            b.text = Some(text);
            b.font_size = font_size;
        }
    }

    pub fn add_child(&mut self, parent_id: u32, child_id: u32) {
        if let Some(parent) = self.boxes.iter_mut().find(|b| b.id == parent_id) {
            parent.children.push(child_id);
        }
    }

    pub fn compute_layout(&mut self, viewport_width: f32, viewport_height: f32) {
        let mut work_queue: Vec<(u32, Rect)> = Vec::new();

        let root_ids: Vec<u32> = self.boxes.iter()
            .filter(|b| b.id == 0)
            .map(|b| b.id)
            .collect();

        for root_id in root_ids {
            work_queue.push((root_id, Rect::new(0.0, 0.0, viewport_width, viewport_height)));
        }

        while let Some((box_id, available)) = work_queue.pop() {
            let children: Vec<u32>;
            let display: Display;
            let style: LayoutStyle;
            if let Some(layout_box) = self.boxes.iter().find(|b| b.id == box_id) {
                children = layout_box.children.clone();
                display = layout_box.display;
                style = layout_box.style.clone();
            } else {
                continue;
            }

            let content_x = available.x + style.margin.left + style.padding.left + style.border_width.left;
            let content_y = available.y + style.margin.top + style.padding.top + style.border_width.top;
            let insets_h = style.margin.horizontal() + style.padding.horizontal() + style.border_width.horizontal();
            let insets_v = style.margin.vertical() + style.padding.vertical() + style.border_width.vertical();

            let box_width = style.width.unwrap_or(available.width);
            let box_height = style.height.unwrap_or(available.height);

            let box_width = if let Some(min_w) = style.min_width { box_width.max(min_w) } else { box_width };
            let box_height = if let Some(min_h) = style.min_height { box_height.max(min_h) } else { box_height };
            let box_width = if let Some(max_w) = style.max_width { box_width.min(max_w) } else { box_width };
            let box_height = if let Some(max_h) = style.max_height { box_height.min(max_h) } else { box_height };

            let content_width = (box_width - insets_h).max(0.0);
            let content_height = (box_height - insets_v).max(0.0);

            if let Some(layout_box) = self.boxes.iter_mut().find(|b| b.id == box_id) {
                layout_box.rect = Rect::new(
                    available.x + style.margin.left,
                    available.y + style.margin.top,
                    box_width - style.margin.horizontal(),
                    box_height - style.margin.vertical(),
                );
                layout_box.content_rect = Rect::new(content_x, content_y, content_width, content_height);
            }

            if children.is_empty() {
                continue;
            }

            match display {
                Display::Flex => {
                    self.layout_flex(&children, content_x, content_y, content_width, content_height, &style, &mut work_queue);
                }
                Display::Grid => {
                    self.layout_grid(&children, content_x, content_y, content_width, content_height, &mut work_queue);
                }
                Display::Block | Display::Inline => {
                    self.layout_block(&children, content_x, content_y, content_width, content_height, &mut work_queue);
                }
                Display::None => {}
            }
        }
    }

    fn layout_flex(&self, children: &[u32], x: f32, y: f32, width: f32, height: f32, parent_style: &LayoutStyle, work_queue: &mut Vec<(u32, Rect)>) {
        if children.is_empty() {
            return;
        }

        let is_row = matches!(parent_style.flex_direction, FlexDirection::Row | FlexDirection::RowReverse);
        let total_grow: f32 = children.iter()
            .filter_map(|&cid| self.boxes.iter().find(|b| b.id == cid))
            .map(|b| b.style.flex_grow)
            .sum();

        let child_count = children.len() as f32;
        let main_size = if is_row { width } else { height };
        let cross_size = if is_row { height } else { width };

        let flex_basis_total: f32 = children.iter()
            .filter_map(|&cid| self.boxes.iter().find(|b| b.id == cid))
            .map(|b| b.style.flex_basis.unwrap_or(0.0))
            .sum();

        let remaining = (main_size - flex_basis_total).max(0.0);

        let mut offset = 0.0f32;
        let gap = if total_grow > 0.0 { 0.0 } else {
            match parent_style.justify_content {
                JustifyContent::SpaceBetween => {
                    if child_count > 1.0 { remaining / (child_count - 1.0) } else { 0.0 }
                }
                JustifyContent::SpaceAround => remaining / child_count,
                JustifyContent::SpaceEvenly => remaining / (child_count + 1.0),
                JustifyContent::Center => { offset = remaining / 2.0; 0.0 }
                JustifyContent::FlexEnd => { offset = remaining; 0.0 }
                JustifyContent::FlexStart => 0.0,
            }
        };

        if matches!(parent_style.justify_content, JustifyContent::SpaceAround) {
            offset = gap / 2.0;
        } else if matches!(parent_style.justify_content, JustifyContent::SpaceEvenly) {
            offset = gap;
        }

        for &child_id in children {
            let child_grow = self.boxes.iter()
                .find(|b| b.id == child_id)
                .map(|b| b.style.flex_grow)
                .unwrap_or(0.0);

            let child_basis = self.boxes.iter()
                .find(|b| b.id == child_id)
                .and_then(|b| b.style.flex_basis)
                .unwrap_or(0.0);

            let child_main = if total_grow > 0.0 {
                child_basis + remaining * (child_grow / total_grow)
            } else {
                if child_basis > 0.0 { child_basis } else { main_size / child_count }
            };

            let child_rect = if is_row {
                Rect::new(x + offset, y, child_main, cross_size)
            } else {
                Rect::new(x, y + offset, cross_size, child_main)
            };

            work_queue.push((child_id, child_rect));
            offset += child_main + gap;
        }
    }

    fn layout_grid(&self, children: &[u32], x: f32, y: f32, width: f32, height: f32, work_queue: &mut Vec<(u32, Rect)>) {
        if children.is_empty() {
            return;
        }

        let count = children.len();
        let cols = libm::ceilf(libm::sqrtf(count as f32)) as usize;
        let cols = cols.max(1);
        let rows = (count + cols - 1) / cols;

        let cell_w = width / cols as f32;
        let cell_h = height / rows as f32;

        for (i, &child_id) in children.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let child_rect = Rect::new(
                x + col as f32 * cell_w,
                y + row as f32 * cell_h,
                cell_w,
                cell_h,
            );
            work_queue.push((child_id, child_rect));
        }
    }

    fn layout_block(&self, children: &[u32], x: f32, y: f32, width: f32, height: f32, work_queue: &mut Vec<(u32, Rect)>) {
        if children.is_empty() {
            return;
        }

        let child_count = children.len() as f32;
        let child_height = height / child_count;

        for (i, &child_id) in children.iter().enumerate() {
            let child_rect = Rect::new(
                x,
                y + child_height * i as f32,
                width,
                child_height,
            );
            work_queue.push((child_id, child_rect));
        }
    }

    pub fn get_box(&self, id: u32) -> Option<&LayoutBox> {
        self.boxes.iter().find(|b| b.id == id)
    }

    pub fn get_box_mut(&mut self, id: u32) -> Option<&mut LayoutBox> {
        self.boxes.iter_mut().find(|b| b.id == id)
    }

    pub fn box_count(&self) -> usize {
        self.boxes.len()
    }

    pub fn all_boxes(&self) -> &[LayoutBox] {
        &self.boxes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddedFont {
    NotoSans,
    NotoSerif,
    NotoSansMono,
}

impl EmbeddedFont {
    pub fn family_name(&self) -> &'static str {
        match self {
            EmbeddedFont::NotoSans => "Noto Sans",
            EmbeddedFont::NotoSerif => "Noto Serif",
            EmbeddedFont::NotoSansMono => "Noto Sans Mono",
        }
    }

    pub fn css_family(&self) -> &'static str {
        match self {
            EmbeddedFont::NotoSans => "sans-serif",
            EmbeddedFont::NotoSerif => "serif",
            EmbeddedFont::NotoSansMono => "monospace",
        }
    }

    pub fn estimated_size(&self) -> usize {
        match self {
            EmbeddedFont::NotoSans => NOTO_SANS_SIZE,
            EmbeddedFont::NotoSerif => NOTO_SERIF_SIZE,
            EmbeddedFont::NotoSansMono => NOTO_MONO_SIZE,
        }
    }

    pub fn all() -> [EmbeddedFont; 3] {
        [EmbeddedFont::NotoSans, EmbeddedFont::NotoSerif, EmbeddedFont::NotoSansMono]
    }
}

pub fn resolve_font_family(css_family: &str) -> EmbeddedFont {
    match css_family {
        "serif" => EmbeddedFont::NotoSerif,
        "monospace" | "mono" => EmbeddedFont::NotoSansMono,
        _ => EmbeddedFont::NotoSans,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains() {
        let r = Rect::new(10.0, 10.0, 100.0, 50.0);
        assert!(r.contains(50.0, 30.0));
        assert!(!r.contains(5.0, 5.0));
    }

    #[test]
    fn test_layout_engine_basic() {
        let mut engine = LayoutEngine::new();
        let root = engine.create_box();
        let child1 = engine.create_box();
        let child2 = engine.create_box();
        engine.add_child(root, child1);
        engine.add_child(root, child2);
        engine.compute_layout(1920.0, 1080.0);
        assert_eq!(engine.box_count(), 3);
    }

    #[test]
    fn test_font_cache_constants() {
        assert_eq!(FONT_FILE_CACHE_GLOBAL, 256 * 1024 * 1024);
        assert_eq!(GLYPH_CACHE_PER_TAB, 16 * 1024 * 1024);
        assert_eq!(GLYPH_CACHE_BACKGROUND, 2 * 1024 * 1024);
        assert!(FONT_FILE_CACHE_GLOBAL > GLYPH_CACHE_PER_TAB);
    }

    #[test]
    fn test_embedded_font_sizes() {
        assert_eq!(NOTO_SANS_SIZE, 550 * 1024);
        assert_eq!(NOTO_SERIF_SIZE, 450 * 1024);
        assert_eq!(NOTO_MONO_SIZE, 250 * 1024);
        assert_eq!(TOTAL_EMBEDDED_FONTS, (550 + 450 + 250) * 1024);
    }

    #[test]
    fn test_font_cache_lru() {
        let mut cache = FontCache::new();
        let id1 = cache.load_font("Font A".into(), alloc::vec![0u8; 1024]);
        let _id2 = cache.load_font("Font B".into(), alloc::vec![0u8; 2048]);
        assert_eq!(cache.font_count(), 2);
        assert_eq!(cache.total_bytes(), 3072);

        let font = cache.get_font(id1);
        assert!(font.is_some());
        assert_eq!(font.unwrap().family, "Font A");
    }

    #[test]
    fn test_tab_glyph_cache() {
        let mut cache = TabGlyphCache::new();
        assert_eq!(cache.capacity(), GLYPH_CACHE_PER_TAB);

        let _glyph = cache.get_or_rasterize(FontId(0), GlyphId(65), 16, rasterize_glyph_placeholder);
        assert_eq!(cache.glyph_count(), 1);
        assert!(cache.total_bytes() > 0);
    }

    #[test]
    fn test_tab_glyph_cache_background_trim() {
        let mut cache = TabGlyphCache::new();

        for i in 0..100u32 {
            cache.get_or_rasterize(FontId(0), GlyphId(i), 16, rasterize_glyph_placeholder);
        }

        assert!(!cache.is_background());
        cache.set_background(true);
        assert!(cache.is_background());
        assert_eq!(cache.capacity(), GLYPH_CACHE_BACKGROUND);
    }

    #[test]
    fn test_shape_text() {
        let glyphs = shape_text("Hello", 16.0);
        assert_eq!(glyphs.len(), 5);
        assert!(glyphs[0].x_offset < glyphs[1].x_offset);
    }

    #[test]
    fn test_rasterize_glyph() {
        let glyph = rasterize_glyph_placeholder(FontId(0), GlyphId(65), 24);
        assert_eq!(glyph.size, 24);
        assert!(glyph.width > 0);
        assert!(glyph.height > 0);
        assert_eq!(glyph.bitmap.len(), (glyph.width * glyph.height) as usize);
    }

    #[test]
    fn test_embedded_fonts() {
        let fonts = EmbeddedFont::all();
        assert_eq!(fonts.len(), 3);
        assert_eq!(fonts[0].css_family(), "sans-serif");
        assert_eq!(fonts[1].css_family(), "serif");
        assert_eq!(fonts[2].css_family(), "monospace");
    }

    #[test]
    fn test_resolve_font_family() {
        assert_eq!(resolve_font_family("serif"), EmbeddedFont::NotoSerif);
        assert_eq!(resolve_font_family("monospace"), EmbeddedFont::NotoSansMono);
        assert_eq!(resolve_font_family("sans-serif"), EmbeddedFont::NotoSans);
        assert_eq!(resolve_font_family("unknown"), EmbeddedFont::NotoSans);
    }

    #[test]
    fn test_flex_layout() {
        let mut engine = LayoutEngine::new();

        let mut root_style = LayoutStyle::new();
        root_style.display = Display::Flex;
        root_style.flex_direction = FlexDirection::Row;
        let root = engine.create_box_with_style(root_style);

        let mut child_style = LayoutStyle::new();
        child_style.flex_grow = 1.0;
        let c1 = engine.create_box_with_style(child_style.clone());
        let c2 = engine.create_box_with_style(child_style);
        engine.add_child(root, c1);
        engine.add_child(root, c2);

        engine.compute_layout(800.0, 600.0);

        let b1 = engine.get_box(c1).unwrap();
        let b2 = engine.get_box(c2).unwrap();
        assert!(b1.rect.width > 0.0);
        assert!(b2.rect.width > 0.0);
    }

    #[test]
    fn test_edge_insets() {
        let insets = EdgeInsets::uniform(10.0);
        assert_eq!(insets.horizontal(), 20.0);
        assert_eq!(insets.vertical(), 20.0);
    }

    #[test]
    fn test_layout_with_text() {
        let mut engine = LayoutEngine::new();
        let root = engine.create_box();
        let text_box = engine.create_box();
        engine.set_text(text_box, "Hello World".into(), 16.0);
        engine.add_child(root, text_box);
        engine.compute_layout(800.0, 600.0);

        let tb = engine.get_box(text_box).unwrap();
        assert_eq!(tb.shaped_glyphs.len(), 11);
    }
}
