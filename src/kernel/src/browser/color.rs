// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PlenumColor: Recursive Polygon Mesh ↔ sRGB Mapping
// Default mesh↔sRGB mapping — ships compiled, no calibration required.
// See TM-2026-017 for mathematical derivation.

use alloc::vec::Vec;
use core::fmt;

pub const MESH_NODES: usize = 540;
pub const INTERIOR_NODES: usize = 482;
pub const RIM_NODES: usize = 58;

pub const ARC_RED: u32 = 182;
pub const ARC_GREEN: u32 = 650;
pub const ARC_PRODUCT: u32 = 118_300;

pub const COPRIME_STEP: u32 = 1001;

pub const BEZIER_IMPROVEMENT_MAX: f64 = 4.8;
pub const BEZIER_IMPROVEMENT_MEAN: f64 = 6.4;

pub const RED_ARC_CONTROL_X: f64 = 0.0;
pub const RED_ARC_CONTROL_Y: f64 = 1.0;
pub const GREEN_ARC_CONTROL_X: f64 = -0.7818;
pub const GREEN_ARC_CONTROL_Y: f64 = 0.6235;

pub const MAX_ARC_SEPARATION: f64 = 0.434;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcConfig {
    Diverge = 0,
    AsymmetricLens = 1,
    MirrorLens = 2,
    VesicaPiscis = 3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeshAddress {
    cells: Vec<u16>,
    depth: usize,
}

impl MeshAddress {
    pub fn new(depth: usize) -> Self {
        Self {
            cells: Vec::with_capacity(depth),
            depth,
        }
    }

    pub fn from_cells(cells: Vec<u16>, depth: usize) -> Option<Self> {
        if cells.len() != depth {
            return None;
        }
        for &c in &cells {
            if c as usize >= MESH_NODES {
                return None;
            }
        }
        Some(Self { cells, depth })
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn cells(&self) -> &[u16] {
        &self.cells
    }

    pub fn total_addresses(depth: usize) -> u64 {
        (MESH_NODES as u64).pow(depth as u32)
    }

    pub fn bits_per_channel(depth: usize) -> f64 {
        use libm::log2;
        log2(Self::total_addresses(depth) as f64) / 3.0
    }

    pub fn effective_bits_per_channel(depth: usize) -> f64 {
        use libm::log2;
        let discrete = Self::bits_per_channel(depth);
        discrete + log2(BEZIER_IMPROVEMENT_MAX)
    }
}

impl fmt::Display for MeshAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mesh[")?;
        for (i, cell) in self.cells.iter().enumerate() {
            if i > 0 {
                write!(f, ".")?;
            }
            write!(f, "{}", cell)?;
        }
        write!(f, "]@d{}", self.depth)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

pub struct MeshPrecision {
    pub depth: usize,
    pub total_addresses: u64,
    pub bits_per_channel: f64,
    pub levels_per_channel: u32,
    pub effective_bits: f64,
}

impl MeshPrecision {
    pub fn compute(depth: usize) -> Self {
        use libm::pow;
        let total = MeshAddress::total_addresses(depth);
        let bits = MeshAddress::bits_per_channel(depth);
        let effective = MeshAddress::effective_bits_per_channel(depth);
        let levels = pow(MESH_NODES as f64, depth as f64 / 3.0) as u32;
        Self {
            depth,
            total_addresses: total,
            bits_per_channel: bits,
            levels_per_channel: levels,
            effective_bits: effective,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct BezierPoint {
    x: f64,
    y: f64,
}

fn quadratic_bezier(p0: BezierPoint, p1: BezierPoint, p2: BezierPoint, t: f64) -> BezierPoint {
    let inv = 1.0 - t;
    BezierPoint {
        x: inv * inv * p0.x + 2.0 * inv * t * p1.x + t * t * p2.x,
        y: inv * inv * p0.y + 2.0 * inv * t * p1.y + t * t * p2.y,
    }
}

pub fn srgb_to_mesh(rgba: Rgba, depth: usize) -> MeshAddress {
    let combined = (rgba.r as f64 / 255.0) * 0.5
        + (rgba.g as f64 / 255.0) * 0.3
        + (rgba.b as f64 / 255.0) * 0.2;

    let total = MESH_NODES as f64;
    let mut cells = Vec::with_capacity(depth);
    let mut remainder = combined;

    for _ in 0..depth {
        let scaled = remainder * total;
        let cell = (scaled as u16).min(MESH_NODES as u16 - 1);
        cells.push(cell);
        remainder = scaled - cell as f64;
        remainder = remainder.max(0.0).min(1.0);
    }

    MeshAddress { cells, depth }
}

pub fn srgb_to_mesh_rgb(rgba: Rgba, depth: usize) -> [MeshAddress; 3] {
    [
        channel_to_mesh(rgba.r, depth),
        channel_to_mesh(rgba.g, depth),
        channel_to_mesh(rgba.b, depth),
    ]
}

fn channel_to_mesh(val: u8, depth: usize) -> MeshAddress {
    let norm = val as f64 / 255.0;
    let total = MESH_NODES as f64;
    let mut cells = Vec::with_capacity(depth);
    let mut remainder = norm;

    for _ in 0..depth {
        let scaled = remainder * total;
        let cell = (scaled as u16).min(MESH_NODES as u16 - 1);
        cells.push(cell);
        remainder = scaled - cell as f64;
        remainder = remainder.max(0.0).min(1.0);
    }

    MeshAddress { cells, depth }
}

fn mesh_to_channel(address: &MeshAddress) -> u8 {
    let mut value = 0.0f64;
    let mut divisor = MESH_NODES as f64;

    for &cell in address.cells() {
        value += cell as f64 / divisor;
        divisor *= MESH_NODES as f64;
    }

    (value * 255.0).round().min(255.0).max(0.0) as u8
}

pub fn mesh_to_srgb(address: &MeshAddress) -> Rgba {
    let mut value = 0.0f64;
    let mut divisor = MESH_NODES as f64;

    for &cell in address.cells() {
        value += cell as f64 / divisor;
        divisor *= MESH_NODES as f64;
    }

    let luma = value;
    let r = (luma * 255.0).round().min(255.0).max(0.0) as u8;
    let g = (luma * 255.0).round().min(255.0).max(0.0) as u8;
    let b = (luma * 255.0).round().min(255.0).max(0.0) as u8;

    Rgba::new(r, g, b, 255)
}

pub fn mesh_rgb_to_srgb(channels: &[MeshAddress; 3]) -> Rgba {
    Rgba::new(
        mesh_to_channel(&channels[0]),
        mesh_to_channel(&channels[1]),
        mesh_to_channel(&channels[2]),
        255,
    )
}

pub fn bezier_interpolate_mesh(addr_a: &MeshAddress, addr_b: &MeshAddress, t: f64) -> Rgba {
    let rgba_a = mesh_to_srgb(addr_a);
    let rgba_b = mesh_to_srgb(addr_b);

    let p0 = BezierPoint {
        x: rgba_a.r as f64 / 255.0,
        y: rgba_a.g as f64 / 255.0,
    };
    let p2 = BezierPoint {
        x: rgba_b.r as f64 / 255.0,
        y: rgba_b.g as f64 / 255.0,
    };
    let control = BezierPoint {
        x: (p0.x + p2.x) / 2.0 + 0.05,
        y: (p0.y + p2.y) / 2.0 + 0.05,
    };

    let interp = quadratic_bezier(p0, control, p2, t);

    let b_val = (1.0 - t) * (rgba_a.b as f64 / 255.0) + t * (rgba_b.b as f64 / 255.0);

    Rgba::new(
        (interp.x * 255.0).min(255.0).max(0.0) as u8,
        (interp.y * 255.0).min(255.0).max(0.0) as u8,
        (b_val * 255.0).min(255.0).max(0.0) as u8,
        255,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_constants() {
        assert_eq!(MESH_NODES, INTERIOR_NODES + RIM_NODES);
        assert_eq!(ARC_RED * ARC_GREEN, ARC_PRODUCT);
        assert_eq!(540, 4 * 27 * 5);
    }

    #[test]
    fn test_mesh_address_creation() {
        let addr = MeshAddress::from_cells(alloc::vec![0, 100, 539], 3);
        assert!(addr.is_some());
        let addr = addr.unwrap();
        assert_eq!(addr.depth(), 3);
        assert_eq!(addr.cells().len(), 3);
    }

    #[test]
    fn test_mesh_address_bounds() {
        let addr = MeshAddress::from_cells(alloc::vec![540, 0, 0], 3);
        assert!(addr.is_none());
    }

    #[test]
    fn test_mesh_precision_depth3() {
        let p = MeshPrecision::compute(3);
        assert_eq!(p.depth, 3);
        assert!(p.bits_per_channel > 9.0);
        assert!(p.effective_bits > 11.0);
    }

    #[test]
    fn test_total_addresses() {
        assert_eq!(MeshAddress::total_addresses(1), 540);
        assert_eq!(MeshAddress::total_addresses(2), 540 * 540);
    }

    #[test]
    fn test_coprime_coverage() {
        assert_eq!(gcd(COPRIME_STEP, MESH_NODES as u32), 1);
        assert_eq!(COPRIME_STEP, 7 * 11 * 13);
    }

    #[test]
    fn test_srgb_roundtrip_black() {
        let black = Rgba::new(0, 0, 0, 255);
        let mesh = srgb_to_mesh(black, 3);
        let back = mesh_to_srgb(&mesh);
        assert_eq!(back.r, 0);
        assert_eq!(back.g, 0);
        assert_eq!(back.b, 0);
    }

    #[test]
    fn test_srgb_rgb_roundtrip() {
        let colors = [
            Rgba::new(0, 0, 0, 255),
            Rgba::new(255, 255, 255, 255),
            Rgba::new(128, 64, 32, 255),
            Rgba::new(255, 0, 0, 255),
            Rgba::new(0, 255, 0, 255),
            Rgba::new(0, 0, 255, 255),
        ];
        for &color in &colors {
            let channels = srgb_to_mesh_rgb(color, 3);
            let back = mesh_rgb_to_srgb(&channels);
            assert!((back.r as i16 - color.r as i16).abs() <= 1,
                "R mismatch: {} vs {}", back.r, color.r);
            assert!((back.g as i16 - color.g as i16).abs() <= 1,
                "G mismatch: {} vs {}", back.g, color.g);
            assert!((back.b as i16 - color.b as i16).abs() <= 1,
                "B mismatch: {} vs {}", back.b, color.b);
        }
    }

    #[test]
    fn test_mesh_node_distribution() {
        let mut seen = alloc::collections::BTreeSet::new();
        for r in (0..=255).step_by(16) {
            let addr = channel_to_mesh(r as u8, 1);
            seen.insert(addr.cells()[0]);
        }
        assert!(seen.len() > 10, "should map to many distinct mesh nodes, got {}", seen.len());
    }

    #[test]
    fn test_different_colors_different_mesh() {
        let red = srgb_to_mesh_rgb(Rgba::new(255, 0, 0, 255), 3);
        let blue = srgb_to_mesh_rgb(Rgba::new(0, 0, 255, 255), 3);
        assert_ne!(red[0].cells(), blue[0].cells(),
            "R channel: 255 vs 0 must map to different mesh addresses");
        assert_ne!(red[2].cells(), blue[2].cells(),
            "B channel: 0 vs 255 must map to different mesh addresses");

        let white = srgb_to_mesh_rgb(Rgba::new(255, 255, 255, 255), 3);
        let black = srgb_to_mesh_rgb(Rgba::new(0, 0, 0, 255), 3);
        for ch in 0..3 {
            assert_ne!(white[ch].cells(), black[ch].cells(),
                "channel {} must differ between white and black", ch);
        }
    }

    #[test]
    fn test_arc_configs() {
        assert_eq!(ArcConfig::Diverge as u8, 0);
        assert_eq!(ArcConfig::VesicaPiscis as u8, 3);
    }

    #[test]
    fn test_arc_equation_roots() {
        let sum = ARC_RED + ARC_GREEN;
        let product = ARC_RED * ARC_GREEN;
        assert_eq!(sum, 832);
        assert_eq!(product, ARC_PRODUCT);
        assert_eq!(ARC_PRODUCT, 118_300);
    }

    fn gcd(a: u32, b: u32) -> u32 {
        if b == 0 { a } else { gcd(b, a % b) }
    }
}
