// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// Recursive Polygon Mesh — 11 polygons inscribed in 364° unit circle.
// 540 nodes (482 interior + 58 rim). Coprime walk via (7, 11, 13).

use alloc::vec::Vec;
use core::fmt;

pub const POLYGON_COUNT: usize = 11;
pub const POLYGON_SIDES: [usize; POLYGON_COUNT] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

pub const FULL_CIRCLE_DEGREES: u32 = 364;
pub const RADIAN_CUSTOM: f64 = 13.85;
pub const PI_TERNARY: u32 = 14;

pub const TILT_STEPS: usize = 4;
pub const TILT_ANGLES_STD: [f64; TILT_STEPS] = [0.0, 13.85, 27.69, 41.54];
pub const TILT_COSINES: [f64; TILT_STEPS] = [1.0000, 0.9709, 0.8855, 0.7485];
pub const TILT_AREAS: [f64; TILT_STEPS] = [1.000, 0.971, 0.885, 0.749];

#[derive(Debug, Clone, Copy)]
pub struct MeshNode {
    pub index: u16,
    pub x: f64,
    pub y: f64,
    pub is_rim: bool,
    pub polygon_a: u8,
    pub polygon_b: u8,
}

pub struct PlanarMesh {
    nodes: Vec<MeshNode>,
}

impl PlanarMesh {
    pub fn new() -> Self {
        let mut nodes = Vec::with_capacity(super::color::MESH_NODES);

        let circle_rad = FULL_CIRCLE_DEGREES as f64 * core::f64::consts::PI / 180.0;

        let mut index: u16 = 0;
        for (pi, &sides_a) in POLYGON_SIDES.iter().enumerate() {
            for (pj, &sides_b) in POLYGON_SIDES.iter().enumerate() {
                if pj <= pi {
                    continue;
                }
                let intersections = estimate_intersections(sides_a, sides_b);
                for k in 0..intersections {
                    if (index as usize) >= super::color::INTERIOR_NODES {
                        break;
                    }
                    let angle = (k as f64 / intersections as f64) * circle_rad;
                    let (sin_a, cos_a) = libm::sincos(angle);
                    nodes.push(MeshNode {
                        index,
                        x: cos_a,
                        y: sin_a,
                        is_rim: false,
                        polygon_a: pi as u8,
                        polygon_b: pj as u8,
                    });
                    index += 1;
                }
            }
        }

        while (index as usize) < super::color::INTERIOR_NODES {
            let angle = (index as f64 / super::color::INTERIOR_NODES as f64) * circle_rad;
            let (sin_a, cos_a) = libm::sincos(angle);
            nodes.push(MeshNode {
                index,
                x: cos_a,
                y: sin_a,
                is_rim: false,
                polygon_a: 0,
                polygon_b: 0,
            });
            index += 1;
        }

        for i in 0..super::color::RIM_NODES {
            let angle = (i as f64 / super::color::RIM_NODES as f64) * circle_rad;
            let (sin_a, cos_a) = libm::sincos(angle);
            nodes.push(MeshNode {
                index,
                x: cos_a,
                y: sin_a,
                is_rim: true,
                polygon_a: 0,
                polygon_b: 0,
            });
            index += 1;
        }

        Self { nodes }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_node(&self, index: u16) -> Option<&MeshNode> {
        self.nodes.get(index as usize)
    }

    pub fn nodes(&self) -> &[MeshNode] {
        &self.nodes
    }

    pub fn interior_nodes(&self) -> &[MeshNode] {
        &self.nodes[..super::color::INTERIOR_NODES]
    }

    pub fn rim_nodes(&self) -> &[MeshNode] {
        &self.nodes[super::color::INTERIOR_NODES..]
    }
}

fn estimate_intersections(sides_a: usize, sides_b: usize) -> usize {
    let max_cross = sides_a * sides_b;
    let scaled = max_cross / (POLYGON_COUNT * 2);
    if scaled == 0 { 1 } else { scaled }
}

#[derive(Debug, Clone, Copy)]
pub struct TiltState {
    pub step: usize,
    pub angle_std: f64,
    pub cosine: f64,
    pub area_fraction: f64,
}

impl TiltState {
    pub fn new(step: usize) -> Self {
        let step = step.min(TILT_STEPS - 1);
        Self {
            step,
            angle_std: TILT_ANGLES_STD[step],
            cosine: TILT_COSINES[step],
            area_fraction: TILT_AREAS[step],
        }
    }

    pub fn visible_nodes(&self) -> usize {
        (super::color::MESH_NODES as f64 * self.area_fraction) as usize
    }
}

impl fmt::Display for TiltState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "tilt[{}]: {:.2}° cos={:.4} area={:.1}%",
            self.step,
            self.angle_std,
            self.cosine,
            self.area_fraction * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polygon_constants() {
        assert_eq!(POLYGON_COUNT, 11);
        assert_eq!(POLYGON_SIDES[0], 3);
        assert_eq!(POLYGON_SIDES[10], 13);
        assert_eq!(FULL_CIRCLE_DEGREES, 364);
        assert_eq!(PI_TERNARY, 14);
    }

    #[test]
    fn test_planar_mesh_construction() {
        let mesh = PlanarMesh::new();
        assert_eq!(mesh.node_count(), super::super::color::MESH_NODES);
    }

    #[test]
    fn test_rim_interior_split() {
        let mesh = PlanarMesh::new();
        assert_eq!(mesh.interior_nodes().len(), super::super::color::INTERIOR_NODES);
        assert_eq!(mesh.rim_nodes().len(), super::super::color::RIM_NODES);
    }

    #[test]
    fn test_tilt_states() {
        let t0 = TiltState::new(0);
        assert_eq!(t0.visible_nodes(), 540);

        let t3 = TiltState::new(3);
        assert!(t3.visible_nodes() < 540);
        assert!(t3.area_fraction < 0.75);
    }

    #[test]
    fn test_tilt_clamping() {
        let t = TiltState::new(99);
        assert_eq!(t.step, TILT_STEPS - 1);
    }
}
