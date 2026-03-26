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
    let r_norm = rgba.r as f64 / 255.0;
    let g_norm = rgba.g as f64 / 255.0;
    let b_norm = rgba.b as f64 / 255.0;

    let mut cells = Vec::with_capacity(depth);
    let mut r_rem = r_norm;
    let mut g_rem = g_norm;
    let mut b_rem = b_norm;

    for _ in 0..depth {
        let r_cell = (r_rem * (MESH_NODES as f64 / 3.0)).min((MESH_NODES as f64 / 3.0) - 1.0) as u16;
        let g_cell = (g_rem * (MESH_NODES as f64 / 3.0)).min((MESH_NODES as f64 / 3.0) - 1.0) as u16;
        let b_cell = (b_rem * (MESH_NODES as f64 / 3.0)).min((MESH_NODES as f64 / 3.0) - 1.0) as u16;

        let cell = (r_cell * (MESH_NODES as u16 / 3) + g_cell) * (MESH_NODES as u16 / 3) + b_cell;
        let cell = cell.min(MESH_NODES as u16 - 1);
        cells.push(cell);

        let scale = MESH_NODES as f64 / 3.0;
        r_rem = r_rem * scale - r_cell as f64;
        g_rem = g_rem * scale - g_cell as f64;
        b_rem = b_rem * scale - b_cell as f64;

        r_rem = r_rem.max(0.0).min(1.0);
        g_rem = g_rem.max(0.0).min(1.0);
        b_rem = b_rem.max(0.0).min(1.0);
    }

    MeshAddress { cells, depth }
}

pub fn mesh_to_srgb(address: &MeshAddress) -> Rgba {
    let depth = address.depth();
    let cells = address.cells();
    let scale = MESH_NODES as f64 / 3.0;

    let mut r = 0.0f64;
    let mut g = 0.0f64;
    let mut b = 0.0f64;

    let mut divisor = 1.0f64;
    for &cell in cells {
        let cell = cell as f64;
        let cube_root = (MESH_NODES as f64 / 3.0) as u16;
        let r_cell = (cell / (cube_root as f64 * cube_root as f64)) as f64;
        let g_cell = ((cell % (cube_root as f64 * cube_root as f64)) / cube_root as f64) as f64;
        let b_cell = (cell % cube_root as f64) as f64;

        r += r_cell / (divisor * scale);
        g += g_cell / (divisor * scale);
        b += b_cell / (divisor * scale);

        divisor *= scale;
    }

    let _ = depth;

    Rgba::new(
        (r * 255.0).min(255.0).max(0.0) as u8,
        (g * 255.0).min(255.0).max(0.0) as u8,
        (b * 255.0).min(255.0).max(0.0) as u8,
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
    fn test_arc_configs() {
        assert_eq!(ArcConfig::Diverge as u8, 0);
        assert_eq!(ArcConfig::VesicaPiscis as u8, 3);
    }

    fn gcd(a: u32, b: u32) -> u32 {
        if b == 0 { a } else { gcd(b, a % b) }
    }
}
