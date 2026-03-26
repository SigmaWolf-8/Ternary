// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// Layout engine — wraps taffy (Flexbox + CSS Grid) + rustybuzz + fontdue.
// Phase 1: Type definitions for layout boxes and text shaping.
// Uses iterative layout with explicit work queue (no recursion → no stack overflow).

use alloc::string::String;
use alloc::vec::Vec;

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

#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub id: u32,
    pub display: Display,
    pub position: Position,
    pub rect: Rect,
    pub children: Vec<u32>,
    pub text: Option<String>,
    pub font_size: f32,
}

impl LayoutBox {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            display: Display::Block,
            position: Position::Static,
            rect: Rect::ZERO,
            children: Vec::new(),
            text: None,
            font_size: 16.0,
        }
    }
}

pub const SYSTEM_FONT_UI: &str = "system-ui";
pub const SYSTEM_FONT_MONO: &str = "monospace";
pub const SYSTEM_FONT_SERIF: &str = "serif";
pub const SYSTEM_FONT_SANS: &str = "sans-serif";

pub const MAX_FONT_CACHE_PER_TAB: usize = 8 * 1024 * 1024;
pub const KERNEL_FONT_CACHE_CAP: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct GlyphInfo {
    pub codepoint: u32,
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_advance: f32,
    pub width: f32,
    pub height: f32,
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

            if let Some(layout_box) = self.boxes.iter().find(|b| b.id == box_id) {
                children = layout_box.children.clone();
                display = layout_box.display;
            } else {
                continue;
            }

            if let Some(layout_box) = self.boxes.iter_mut().find(|b| b.id == box_id) {
                layout_box.rect = available;
            }

            if !children.is_empty() {
                let child_count = children.len() as f32;
                for (i, &child_id) in children.iter().enumerate() {
                    let child_rect = match display {
                        Display::Flex => Rect::new(
                            available.x + (available.width / child_count) * i as f32,
                            available.y,
                            available.width / child_count,
                            available.height,
                        ),
                        _ => Rect::new(
                            available.x,
                            available.y + (available.height / child_count) * i as f32,
                            available.width,
                            available.height / child_count,
                        ),
                    };
                    work_queue.push((child_id, child_rect));
                }
            }
        }
    }

    pub fn get_box(&self, id: u32) -> Option<&LayoutBox> {
        self.boxes.iter().find(|b| b.id == id)
    }

    pub fn box_count(&self) -> usize {
        self.boxes.len()
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
    fn test_font_cache_limits() {
        assert_eq!(MAX_FONT_CACHE_PER_TAB, 8 * 1024 * 1024);
        assert!(KERNEL_FONT_CACHE_CAP > MAX_FONT_CACHE_PER_TAB);
    }
}
