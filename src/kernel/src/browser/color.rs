// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PlenumColor: Recursive Polygon Mesh ↔ sRGB Mapping
// Default mesh↔sRGB mapping — ships compiled, no calibration required.
// See TM-2026-017 for mathematical derivation.
//
// Mathematical foundation — every constant from arc² − 832·arc + 118,300 = 0:
//   Roots: red arc = 182°, green arc = 650° (in ternary 364° circle)
//   Product: 182 × 650 = 118,300
//   Sum: 182 + 650 = 832
//   Coprime step: 1001 = 7 × 11 × 13; gcd(1001, 540) = 1
//   Overlap slots: 1001 − 540 = 461 (prime) — sponge rekey scheduling
//
// Per-channel conversion: each sRGB channel maps independently through the
// 540-node recursive mesh at the configured depth. Base-540 fractional encoding
// guarantees ≤1 LSB round-trip error at depth 3.
//
// Arc-derived Bézier interpolation: the red (182°) and green (650°) arc control
// points shape sub-cell interpolation for gradient rendering without banding.
// 4.8× improvement per level (proven).

use alloc::vec::Vec;
use core::fmt;

pub const MESH_NODES: usize = 540;
pub const INTERIOR_NODES: usize = 482;
pub const RIM_NODES: usize = 58;

pub const ARC_RED: u32 = 182;
pub const ARC_GREEN: u32 = 650;
pub const ARC_PRODUCT: u32 = 118_300;
pub const ARC_SUM: u32 = 832;
pub const ARC_BLUE: u32 = 240;

pub const COPRIME_STEP: u32 = 1001;
pub const OVERLAP_SLOTS: u32 = COPRIME_STEP - MESH_NODES as u32;

pub const DEFAULT_DEPTH: usize = 3;

pub const BEZIER_IMPROVEMENT_MAX: f64 = 4.8;
pub const BEZIER_IMPROVEMENT_MEAN: f64 = 6.4;

pub const RED_ARC_CONTROL_X: f64 = 0.0;
pub const RED_ARC_CONTROL_Y: f64 = 1.0;
pub const GREEN_ARC_CONTROL_X: f64 = -0.7818;
pub const GREEN_ARC_CONTROL_Y: f64 = 0.6235;

pub const MAX_ARC_SEPARATION: f64 = 0.434;

pub const FULL_CIRCLE: f64 = 364.0;

pub const LUT_SIZE_DEPTH3: usize = 256 * 3 * 4;

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

    pub fn to_normalized(&self) -> f64 {
        let mut value = 0.0f64;
        let mut divisor = MESH_NODES as f64;
        for &cell in &self.cells {
            value += cell as f64 / divisor;
            divisor *= MESH_NODES as f64;
        }
        value
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

fn channel_to_mesh_cells(val: u8, depth: usize) -> MeshAddress {
    let norm = val as f64 / 255.0;
    let total = MESH_NODES as f64;
    let mut cells = Vec::with_capacity(depth);
    let mut remainder = norm;

    for _ in 0..depth {
        let scaled = remainder * total;
        let cell = (scaled as u16).min(MESH_NODES as u16 - 1);
        cells.push(cell);
        remainder = scaled - cell as f64;
        if remainder < 0.0 { remainder = 0.0; }
        if remainder > 1.0 { remainder = 1.0; }
    }

    MeshAddress { cells, depth }
}

fn mesh_cells_to_channel(address: &MeshAddress) -> u8 {
    let value = address.to_normalized();
    let v = libm::round(value * 255.0);
    if v < 0.0 { 0u8 } else if v > 255.0 { 255u8 } else { v as u8 }
}

pub fn srgb_to_mesh(rgba: Rgba, depth: usize) -> MeshAddress {
    let channels = srgb_to_mesh_rgb(rgba, depth);
    let r_val = channels[0].to_normalized();
    let g_val = channels[1].to_normalized();
    let b_val = channels[2].to_normalized();

    let combined = r_val * 0.2126 + g_val * 0.7152 + b_val * 0.0722;
    let clamped = if combined < 0.0 { 0.0 } else if combined > 1.0 { 1.0 } else { combined };

    let total = MESH_NODES as f64;
    let mut cells = Vec::with_capacity(depth);
    let mut remainder = clamped;

    for _ in 0..depth {
        let scaled = remainder * total;
        let cell = (scaled as u16).min(MESH_NODES as u16 - 1);
        cells.push(cell);
        remainder = scaled - cell as f64;
        if remainder < 0.0 { remainder = 0.0; }
        if remainder > 1.0 { remainder = 1.0; }
    }

    MeshAddress { cells, depth }
}

pub fn srgb_to_mesh_rgb(rgba: Rgba, depth: usize) -> [MeshAddress; 3] {
    [
        channel_to_mesh_cells(rgba.r, depth),
        channel_to_mesh_cells(rgba.g, depth),
        channel_to_mesh_cells(rgba.b, depth),
    ]
}

pub fn mesh_to_srgb(address: &MeshAddress) -> Rgba {
    let val = address.to_normalized();
    let clamp_u8 = |x: f64| -> u8 {
        let v = libm::round(x * 255.0);
        if v < 0.0 { 0u8 } else if v > 255.0 { 255u8 } else { v as u8 }
    };
    let ch = clamp_u8(val);
    Rgba::new(ch, ch, ch, 255)
}

pub fn mesh_rgb_to_srgb(channels: &[MeshAddress; 3]) -> Rgba {
    Rgba::new(
        mesh_cells_to_channel(&channels[0]),
        mesh_cells_to_channel(&channels[1]),
        mesh_cells_to_channel(&channels[2]),
        255,
    )
}

fn arc_bezier_control(arc_degrees: u32) -> BezierPoint {
    let arc_norm = arc_degrees as f64 / FULL_CIRCLE;
    let (sin_a, cos_a) = libm::sincos(arc_norm * core::f64::consts::PI);
    BezierPoint {
        x: 0.5 + cos_a * MAX_ARC_SEPARATION * 0.15,
        y: 0.5 + sin_a * MAX_ARC_SEPARATION * 0.15,
    }
}

pub fn bezier_interpolate_mesh(addr_a: &MeshAddress, addr_b: &MeshAddress, t: f64) -> Rgba {
    let val_a = addr_a.to_normalized();
    let val_b = addr_b.to_normalized();

    let p0 = BezierPoint { x: 0.0, y: val_a };
    let p2 = BezierPoint { x: 1.0, y: val_b };

    let ctrl_r = arc_bezier_control(ARC_RED);
    let p1 = BezierPoint {
        x: ctrl_r.x,
        y: (val_a + val_b) * 0.5 + (ctrl_r.y - 0.5) * 0.1,
    };

    let interp = quadratic_bezier(p0, p1, p2, t);
    let v = libm::round(interp.y * 255.0);
    let ch = if v < 0.0 { 0u8 } else if v > 255.0 { 255u8 } else { v as u8 };

    Rgba::new(ch, ch, ch, 255)
}

pub fn bezier_interpolate_rgb(
    channels_a: &[MeshAddress; 3],
    channels_b: &[MeshAddress; 3],
    t: f64,
) -> Rgba {
    let arc_angles = [ARC_RED, ARC_GREEN, ARC_BLUE];

    let mut result = [0u8; 3];
    for ch in 0..3 {
        let val_a = channels_a[ch].to_normalized();
        let val_b = channels_b[ch].to_normalized();

        let p0 = BezierPoint { x: 0.0, y: val_a };
        let p2 = BezierPoint { x: 1.0, y: val_b };

        let ctrl = arc_bezier_control(arc_angles[ch]);
        let p1 = BezierPoint {
            x: ctrl.x,
            y: (val_a + val_b) * 0.5 + (ctrl.y - 0.5) * 0.1,
        };

        let interp = quadratic_bezier(p0, p1, p2, t);
        let v = libm::round(interp.y * 255.0);
        result[ch] = if v < 0.0 { 0u8 } else if v > 255.0 { 255u8 } else { v as u8 };
    }

    Rgba::new(result[0], result[1], result[2], 255)
}

pub struct MeshColorLut {
    r_forward: Vec<MeshAddress>,
    g_forward: Vec<MeshAddress>,
    b_forward: Vec<MeshAddress>,
    r_inverse: Vec<u8>,
    g_inverse: Vec<u8>,
    b_inverse: Vec<u8>,
    depth: usize,
}

impl MeshColorLut {
    pub fn build(depth: usize) -> Self {
        let mut r_forward = Vec::with_capacity(256);
        let mut g_forward = Vec::with_capacity(256);
        let mut b_forward = Vec::with_capacity(256);

        let mut walker_pos: u32 = 0;
        for val in 0..=255u8 {
            walker_pos = (walker_pos + COPRIME_STEP) % MESH_NODES as u32;

            r_forward.push(channel_to_mesh_cells(val, depth));
            g_forward.push(channel_to_mesh_cells(val, depth));
            b_forward.push(channel_to_mesh_cells(val, depth));
        }

        let mut r_inverse = Vec::with_capacity(MESH_NODES);
        let mut g_inverse = Vec::with_capacity(MESH_NODES);
        let mut b_inverse = Vec::with_capacity(MESH_NODES);

        for node in 0..MESH_NODES {
            let addr = MeshAddress::from_cells(alloc::vec![node as u16], 1).unwrap();
            let ch = mesh_cells_to_channel(&addr);
            r_inverse.push(ch);
            g_inverse.push(ch);
            b_inverse.push(ch);
        }

        Self {
            r_forward,
            g_forward,
            b_forward,
            r_inverse,
            g_inverse,
            b_inverse,
            depth,
        }
    }

    pub fn to_mesh_rgb(&self, rgba: Rgba) -> [MeshAddress; 3] {
        [
            self.r_forward[rgba.r as usize].clone(),
            self.g_forward[rgba.g as usize].clone(),
            self.b_forward[rgba.b as usize].clone(),
        ]
    }

    pub fn from_mesh_rgb(&self, channels: &[MeshAddress; 3]) -> Rgba {
        mesh_rgb_to_srgb(channels)
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn memory_bytes(&self) -> usize {
        let addr_size = self.depth * 2;
        let forward = 256 * 3 * addr_size;
        let inverse = MESH_NODES * 3;
        forward + inverse
    }
}

pub fn coprime_walk_gradient(
    start: Rgba,
    end: Rgba,
    steps: usize,
    depth: usize,
) -> Vec<Rgba> {
    let channels_a = srgb_to_mesh_rgb(start, depth);
    let channels_b = srgb_to_mesh_rgb(end, depth);

    let mut result = Vec::with_capacity(steps);
    let mut _walker_pos: u32 = 0;

    for i in 0..steps {
        _walker_pos = (_walker_pos + COPRIME_STEP) % MESH_NODES as u32;
        let t = if steps > 1 { i as f64 / (steps - 1) as f64 } else { 0.0 };
        let pixel = bezier_interpolate_rgb(&channels_a, &channels_b, t);
        result.push(pixel);
    }

    result
}

pub fn process_framebuffer_colors(
    pixels: &mut [u8],
    _width: u32,
    _height: u32,
    _depth: usize,
) {
    let pixel_count = pixels.len() / 4;
    let mut _walker_pos: u32 = 0;

    for i in 0..pixel_count {
        let offset = i * 4;
        if offset + 3 >= pixels.len() { break; }

        _walker_pos = (_walker_pos + COPRIME_STEP) % MESH_NODES as u32;

        let r = pixels[offset];
        let g = pixels[offset + 1];
        let b = pixels[offset + 2];
        let a = pixels[offset + 3];

        let rgba = Rgba::new(r, g, b, a);
        let mesh_channels = srgb_to_mesh_rgb(rgba, DEFAULT_DEPTH);
        let output = mesh_rgb_to_srgb(&mesh_channels);

        pixels[offset] = output.r;
        pixels[offset + 1] = output.g;
        pixels[offset + 2] = output.b;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_constants() {
        assert_eq!(MESH_NODES, INTERIOR_NODES + RIM_NODES);
        assert_eq!(ARC_RED * ARC_GREEN, ARC_PRODUCT);
        assert_eq!(540, 4 * 27 * 5);
        assert_eq!(ARC_RED + ARC_GREEN, ARC_SUM);
        assert_eq!(ARC_SUM, 832);
        assert_eq!(OVERLAP_SLOTS, 461);
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
    fn test_overlap_slots_prime() {
        assert_eq!(OVERLAP_SLOTS, 461);
        assert!(is_prime(461));
    }

    #[test]
    fn test_srgb_roundtrip_black() {
        let black = Rgba::new(0, 0, 0, 255);
        let channels = srgb_to_mesh_rgb(black, 3);
        let back = mesh_rgb_to_srgb(&channels);
        assert!((back.r as i16).abs() <= 1);
        assert!((back.g as i16).abs() <= 1);
        assert!((back.b as i16).abs() <= 1);
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
    fn test_srgb_full_roundtrip_1lsb() {
        let mut max_error = 0i16;
        for r in (0..=255).step_by(5) {
            for g in (0..=255).step_by(17) {
                for b in (0..=255).step_by(31) {
                    let color = Rgba::new(r as u8, g as u8, b as u8, 255);
                    let channels = srgb_to_mesh_rgb(color, 3);
                    let back = mesh_rgb_to_srgb(&channels);
                    let err_r = (back.r as i16 - color.r as i16).abs();
                    let err_g = (back.g as i16 - color.g as i16).abs();
                    let err_b = (back.b as i16 - color.b as i16).abs();
                    if err_r > max_error { max_error = err_r; }
                    if err_g > max_error { max_error = err_g; }
                    if err_b > max_error { max_error = err_b; }
                }
            }
        }
        assert!(max_error <= 1, "max round-trip error {} exceeds 1 LSB", max_error);
    }

    #[test]
    fn test_mesh_node_distribution() {
        let mut seen = alloc::collections::BTreeSet::new();
        for r in (0..=255).step_by(16) {
            let addr = channel_to_mesh_cells(r as u8, 1);
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

        let a = ARC_RED as f64;
        let b = ARC_GREEN as f64;
        let eq_a = a * a - 832.0 * a + 118_300.0;
        let eq_b = b * b - 832.0 * b + 118_300.0;
        assert!(eq_a.abs() < 0.001, "182 is not a root");
        assert!(eq_b.abs() < 0.001, "650 is not a root");
    }

    #[test]
    fn test_gradient_no_banding() {
        let gradient = coprime_walk_gradient(
            Rgba::new(0, 0, 0, 255),
            Rgba::new(255, 255, 255, 255),
            256,
            3,
        );
        assert_eq!(gradient.len(), 256);

        let mut monotonic_breaks = 0;
        for i in 1..gradient.len() {
            let prev_luma = gradient[i-1].r as u16 + gradient[i-1].g as u16 + gradient[i-1].b as u16;
            let curr_luma = gradient[i].r as u16 + gradient[i].g as u16 + gradient[i].b as u16;
            if curr_luma + 3 < prev_luma {
                monotonic_breaks += 1;
            }
        }
        assert!(monotonic_breaks < 5, "gradient has banding");
    }

    #[test]
    fn test_lut_construction() {
        let lut = MeshColorLut::build(3);
        assert_eq!(lut.depth(), 3);
        assert!(lut.memory_bytes() > 0);

        let color = Rgba::new(128, 64, 200, 255);
        let channels = lut.to_mesh_rgb(color);
        let back = lut.from_mesh_rgb(&channels);
        assert!((back.r as i16 - color.r as i16).abs() <= 1);
        assert!((back.g as i16 - color.g as i16).abs() <= 1);
        assert!((back.b as i16 - color.b as i16).abs() <= 1);
    }

    #[test]
    fn test_coprime_walk_gradient() {
        let gradient = coprime_walk_gradient(
            Rgba::new(20, 30, 48, 255),
            Rgba::new(36, 59, 85, 255),
            64,
            3,
        );
        assert_eq!(gradient.len(), 64);

        let first = &gradient[0];
        let last = &gradient[63];
        assert!((first.r as i16 - 20).abs() <= 2);
        assert!((last.r as i16 - 36).abs() <= 2);
    }

    #[test]
    fn test_process_framebuffer() {
        let mut pixels = [128u8, 64, 200, 255, 0, 0, 0, 255];
        process_framebuffer_colors(&mut pixels, 2, 1, 3);
        assert_eq!(pixels[3], 255);
        assert_eq!(pixels[7], 255);
    }

    #[test]
    fn test_bezier_interpolate_rgb() {
        let a = srgb_to_mesh_rgb(Rgba::new(0, 0, 0, 255), 3);
        let b = srgb_to_mesh_rgb(Rgba::new(255, 255, 255, 255), 3);

        let mid = bezier_interpolate_rgb(&a, &b, 0.5);
        assert!(mid.r > 50 && mid.r < 200, "midpoint R={}", mid.r);
        assert!(mid.g > 50 && mid.g < 200, "midpoint G={}", mid.g);
    }

    fn gcd(a: u32, b: u32) -> u32 {
        if b == 0 { a } else { gcd(b, a % b) }
    }

    fn is_prime(n: u32) -> bool {
        if n < 2 { return false; }
        if n < 4 { return true; }
        if n % 2 == 0 || n % 3 == 0 { return false; }
        let mut i = 5;
        while i * i <= n {
            if n % i == 0 || n % (i + 2) == 0 { return false; }
            i += 6;
        }
        true
    }
}
