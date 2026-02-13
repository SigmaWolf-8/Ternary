// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// All Rights Reserved.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! # Ternary Torus Network Topology
//!
//! A k-ary n-cube with k=3: the natural network topology for ternary-addressed nodes.
//!
//! ## Why ternary torus for PlenumNET
//!
//! - Node addresses are n-trit balanced ternary words
//! - Routing is subtraction in GF(3)^n — each component of the routing vector
//!   is a single trit telling the node: stay (0), go forward (+1), or go back (-1)
//! - The topology is vertex-transitive: every node sees the same local structure,
//!   so no node is a natural bottleneck
//! - Diameter is exactly n for a 3^n node network (every ring of 3 is fully connected)

use crate::gf3::{Gf3, Gf3Vec};
use std::fmt;

/// A node address in the ternary torus: an n-trit balanced ternary word.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TorusAddress {
    pub trits: Gf3Vec,
}

impl TorusAddress {
    /// Create an address from a vector of GF(3) elements.
    pub fn new(trits: Vec<Gf3>) -> Self {
        TorusAddress {
            trits: Gf3Vec::new(trits),
        }
    }

    /// The all-zeros address (origin node).
    pub fn origin(dimensions: usize) -> Self {
        TorusAddress {
            trits: Gf3Vec::zeros(dimensions),
        }
    }

    /// Number of dimensions.
    pub fn dimensions(&self) -> usize {
        self.trits.len()
    }

    /// Compute the routing vector from self to destination.
    /// Each component tells the router: 0 = stay, 1 = forward, 2 (-1) = backward.
    pub fn routing_vector(&self, destination: &TorusAddress) -> Gf3Vec {
        destination.trits.sub(&self.trits)
    }

    /// Manhattan distance (hop count) to destination on the torus.
    /// Each dimension contributes 0 or 1 hop (since a ring of 3 has diameter 1).
    pub fn distance(&self, destination: &TorusAddress) -> usize {
        let rv = self.routing_vector(destination);
        let mut hops = 0;
        for i in 0..rv.len() {
            if !rv.get(i).is_zero() {
                hops += 1;
            }
        }
        hops
    }

    /// List all neighbors of this node (2 per dimension = 2n total).
    pub fn neighbors(&self) -> Vec<TorusAddress> {
        let n = self.dimensions();
        let mut nbrs = Vec::with_capacity(2 * n);
        for dim in 0..n {
            // Forward neighbor (+1 in this dimension)
            let mut trits_fwd: Vec<Gf3> = (0..n).map(|i| self.trits.get(i)).collect();
            trits_fwd[dim] = trits_fwd[dim] + Gf3::ONE;
            nbrs.push(TorusAddress::new(trits_fwd));

            // Backward neighbor (-1 = +2 in this dimension)
            let mut trits_bwd: Vec<Gf3> = (0..n).map(|i| self.trits.get(i)).collect();
            trits_bwd[dim] = trits_bwd[dim] + Gf3::TWO;
            nbrs.push(TorusAddress::new(trits_bwd));
        }
        nbrs
    }

    /// Dimension-order routing: returns the sequence of intermediate hops
    /// from self to destination.
    pub fn route_to(&self, destination: &TorusAddress) -> Vec<TorusAddress> {
        let rv = self.routing_vector(destination);
        let n = self.dimensions();
        let mut path = Vec::new();
        let mut current: Vec<Gf3> = (0..n).map(|i| self.trits.get(i)).collect();

        for dim in 0..n {
            let step = rv.get(dim);
            if !step.is_zero() {
                current[dim] = current[dim] + step;
                path.push(TorusAddress::new(current.clone()));
            }
        }
        path
    }
}

impl fmt::Debug for TorusAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T[")?;
        for i in 0..self.trits.len() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", self.trits.get(i).to_balanced())?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for TorusAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

// -- Torus topology analysis --------------------------------------------------

/// Properties of a 3-ary n-cube network.
#[derive(Debug, Clone)]
pub struct TorusProperties {
    pub dimensions: usize,
    pub node_count: u64,
    pub degree: usize,
    pub diameter: usize,
    pub total_links: u64,
    pub bisection_bandwidth: u64,
    pub average_distance: f64,
}

/// Compute properties of a 3-ary n-cube.
pub fn torus_properties(n: usize) -> TorusProperties {
    let node_count = 3u64.pow(u32::try_from(n).unwrap());
    let degree = 2 * n;
    let diameter = n; // Each ring of 3 has diameter 1
    let total_links = n as u64 * node_count; // Each dimension contributes 3^n links
    let bisection_bandwidth = 2 * 3u64.pow(u32::try_from(n - 1).unwrap());
    // Average distance: each dimension contributes 2/3 on average
    // (1/3 chance of distance 0, 2/3 chance of distance 1)
    let average_distance = n as f64 * (2.0 / 3.0);

    TorusProperties {
        dimensions: n,
        node_count,
        degree,
        diameter,
        total_links,
        bisection_bandwidth,
        average_distance,
    }
}

impl fmt::Display for TorusProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "3-ary {}-cube (Ternary Torus)", self.dimensions)?;
        writeln!(f, "  Nodes:       {}", self.node_count)?;
        writeln!(f, "  Degree:      {} (connections per node)", self.degree)?;
        writeln!(f, "  Diameter:    {} hops (worst case)", self.diameter)?;
        writeln!(f, "  Avg distance: {:.2} hops", self.average_distance)?;
        writeln!(f, "  Total links: {}", self.total_links)?;
        writeln!(f, "  Bisection BW: {}", self.bisection_bandwidth)?;
        Ok(())
    }
}

/// Compare ternary torus to binary hypercube at similar node counts.
pub fn topology_comparison(ternary_dims: usize) -> TopologyComparison {
    let ternary = torus_properties(ternary_dims);
    // Find binary hypercube with closest node count
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let binary_dims = (ternary.node_count as f64).log2().ceil() as usize;
    let binary_nodes = 2u64.pow(u32::try_from(binary_dims).unwrap());

    TopologyComparison {
        ternary_dims,
        ternary_nodes: ternary.node_count,
        ternary_degree: ternary.degree,
        ternary_diameter: ternary.diameter,
        ternary_avg_distance: ternary.average_distance,
        ternary_bisection: ternary.bisection_bandwidth,
        binary_dims,
        binary_nodes,
        binary_degree: binary_dims,
        binary_diameter: binary_dims,
        binary_avg_distance: binary_dims as f64 / 2.0,
        binary_bisection: binary_nodes / 2,
    }
}

#[derive(Debug, Clone)]
pub struct TopologyComparison {
    pub ternary_dims: usize,
    pub ternary_nodes: u64,
    pub ternary_degree: usize,
    pub ternary_diameter: usize,
    pub ternary_avg_distance: f64,
    pub ternary_bisection: u64,
    pub binary_dims: usize,
    pub binary_nodes: u64,
    pub binary_degree: usize,
    pub binary_diameter: usize,
    pub binary_avg_distance: f64,
    pub binary_bisection: u64,
}

impl fmt::Display for TopologyComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Topology Comparison")?;
        writeln!(f, "                    Ternary Torus    Binary Hypercube")?;
        writeln!(f, "  Dimensions:       {:<16} {}", self.ternary_dims, self.binary_dims)?;
        writeln!(f, "  Nodes:            {:<16} {}", self.ternary_nodes, self.binary_nodes)?;
        writeln!(f, "  Degree:           {:<16} {}", self.ternary_degree, self.binary_degree)?;
        writeln!(f, "  Diameter:         {:<16} {}", self.ternary_diameter, self.binary_diameter)?;
        writeln!(f, "  Avg distance:     {:<16.2} {:.2}", self.ternary_avg_distance, self.binary_avg_distance)?;
        writeln!(f, "  Bisection BW:     {:<16} {}", self.ternary_bisection, self.binary_bisection)?;
        Ok(())
    }
}

/// Ternary Circle Bridge — connecting torus topology to Z₂₈ geometry.
///
/// This bridge maps Z₂₈ cyclic group positions to torus node addresses,
/// enabling Tribonacci-guided walks on the torus network. The integration
/// connects the angular geometry of the ternary circle (28 discrete positions)
/// with the GF(3)-addressed torus topology.
///
/// ## Architecture
///
/// Z₂₈ positions are decomposed into torus coordinates via mod-3 projections:
/// - Position → [pos mod 3, (pos/1) mod 3, (pos/3) mod 3, ...]
/// - This maps the 28-element cyclic group into a multi-dimensional GF(3) lattice
///
/// Tribonacci word digits drive walks on both Z₂₈ and the torus simultaneously,
/// providing a unified walk model across angular and topological spaces.
pub mod ternary_circle_bridge {
    use super::*;
    use crate::constants::{TAU_TRIBONACCI, TORUS_RADIX};
    use crate::ternary_circle::Z28;
    use crate::tribonacci::tribonacci_word;

    /// Map a Z₂₈ position to a torus address by decomposing the position
    /// into GF(3) coordinates across `dims` dimensions.
    ///
    /// The mapping uses successive division by 3:
    ///   coord[0] = pos mod 3
    ///   coord[1] = (pos / 3) mod 3
    ///   coord[2] = (pos / 9) mod 3
    ///   ...
    ///
    /// This embeds Z₂₈ into the torus lattice GF(3)^dims.
    pub fn z28_to_torus_address(pos: Z28, dims: usize) -> TorusAddress {
        let mut value = pos.value() as u32;
        let mut trits = Vec::with_capacity(dims);
        for _ in 0..dims {
            let gf3_val = match value % TORUS_RADIX {
                0 => Gf3::ZERO,
                1 => Gf3::ONE,
                2 => Gf3::TWO,
                _ => unreachable!(),
            };
            trits.push(gf3_val);
            value /= TORUS_RADIX;
        }
        TorusAddress::new(trits)
    }

    /// Map a torus address back to the nearest Z₂₈ position.
    ///
    /// Reconstructs a Z₂₈ position from GF(3) coordinates. Only the first
    /// few dimensions contribute meaningfully (since 3^3 = 27 ≈ 28).
    pub fn torus_address_to_z28(addr: &TorusAddress) -> Z28 {
        let mut value: u32 = 0;
        let mut power: u32 = 1;
        let dims = std::cmp::min(addr.dimensions(), 3);
        for i in 0..dims {
            let trit = addr.trits.get(i);
            let v = if trit == Gf3::ONE { 1u32 }
                    else if trit == Gf3::TWO { 2u32 }
                    else { 0u32 };
            value += v * power;
            power *= TORUS_RADIX;
        }
        Z28::new(value)
    }

    /// Walk the torus using Tribonacci word digits as routing instructions.
    ///
    /// Each Tribonacci digit (0, 1, 2) advances the torus position in a
    /// cyclic dimension:
    ///   0 → no hop (stay)
    ///   1 → forward hop (+1 in GF(3))
    ///   2 → backward hop (-1 = +2 in GF(3))
    ///
    /// The walk proceeds dimension-by-dimension in round-robin fashion,
    /// creating a Tribonacci-guided path through the torus.
    pub fn tribonacci_torus_walk(dims: usize, steps: usize) -> Vec<TorusAddress> {
        let word = tribonacci_word(steps);
        let mut path = Vec::with_capacity(steps + 1);
        let mut current: Vec<Gf3> = vec![Gf3::ZERO; dims];
        path.push(TorusAddress::new(current.clone()));

        for (step, &trit) in word.iter().enumerate() {
            let dim = step % dims;
            let offset = match trit {
                0 => Gf3::ZERO,
                1 => Gf3::ONE,
                2 => Gf3::TWO,
                _ => unreachable!(),
            };
            current[dim] = current[dim] + offset;
            path.push(TorusAddress::new(current.clone()));
        }

        path
    }

    /// Simultaneous Z₂₈ + torus walk using Tribonacci word.
    ///
    /// Returns pairs of (Z₂₈ angular position, torus network address)
    /// at each step, providing unified tracking across both spaces.
    pub fn z28_torus_walk(dims: usize, steps: usize) -> Vec<(Z28, TorusAddress)> {
        let word = tribonacci_word(steps);
        let mut result = Vec::with_capacity(steps);
        let mut z28_pos = Z28::zero();
        let mut torus_coords: Vec<Gf3> = vec![Gf3::ZERO; dims];

        for (step, &trit) in word.iter().enumerate() {
            z28_pos = z28_pos.step(trit);

            let dim = step % dims;
            let offset = match trit {
                0 => Gf3::ZERO,
                1 => Gf3::ONE,
                2 => Gf3::TWO,
                _ => unreachable!(),
            };
            torus_coords[dim] = torus_coords[dim] + offset;

            result.push((z28_pos, TorusAddress::new(torus_coords.clone())));
        }

        result
    }

    /// Compute the τ-scaled distance between two Z₂₈-mapped torus positions.
    ///
    /// The distance is the standard torus hop-count, scaled by 1/τ^k where
    /// k is the step index. This mirrors the Tribonacci spiral scaling in
    /// the angular domain.
    pub fn tau_scaled_torus_distance(a: &TorusAddress, b: &TorusAddress, step: usize) -> f64 {
        let hops = a.distance(b) as f64;
        let tau_power = TAU_TRIBONACCI.powi(step as i32);
        hops / tau_power
    }
}

/// Run the full topology analysis for PlenumNET.
pub fn full_topology_report() -> String {
    let mut report = String::new();

    report.push_str("═══════════════════════════════════════════════════════\n");
    report.push_str("  TERNARY TORUS TOPOLOGY ANALYSIS — PlenumNET\n");
    report.push_str("═══════════════════════════════════════════════════════\n\n");

    for n in 2..=7 {
        report.push_str(&format!("{}\n", torus_properties(n)));
    }

    report.push_str("─────────────────────────────────────────────────────\n");
    report.push_str("Ternary Torus vs Binary Hypercube:\n\n");

    for n in 3..=7 {
        report.push_str(&format!("{}\n", topology_comparison(n)));
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::ternary_circle_bridge::*;
    use crate::ternary_circle::Z28;

    #[test]
    fn z28_to_torus_roundtrip_3d() {
        for i in 0..28u8 {
            let pos = Z28(i);
            let addr = z28_to_torus_address(pos, 3);
            let back = torus_address_to_z28(&addr);
            if i < 27 {
                assert_eq!(back.value(), i,
                    "Z₂₈({}) → torus → Z₂₈ roundtrip failed (got {})", i, back.value());
            }
        }
    }

    #[test]
    fn z28_origin_maps_to_torus_origin() {
        let addr = z28_to_torus_address(Z28::zero(), 3);
        for i in 0..3 {
            assert_eq!(addr.trits.get(i), Gf3::ZERO);
        }
    }

    #[test]
    fn tribonacci_torus_walk_starts_at_origin() {
        let path = tribonacci_torus_walk(3, 10);
        assert_eq!(path.len(), 11);
        let origin = &path[0];
        for i in 0..3 {
            assert_eq!(origin.trits.get(i), Gf3::ZERO);
        }
    }

    #[test]
    fn z28_torus_walk_tracks_both() {
        let walk = z28_torus_walk(3, 20);
        assert_eq!(walk.len(), 20);
        for (z28_pos, torus_addr) in &walk {
            assert!(z28_pos.value() < 28);
            assert_eq!(torus_addr.dimensions(), 3);
        }
    }

    #[test]
    fn tau_scaled_distance_decreases() {
        let a = TorusAddress::origin(3);
        let b = TorusAddress::new(vec![Gf3::ONE, Gf3::ONE, Gf3::ONE]);
        let d1 = tau_scaled_torus_distance(&a, &b, 1);
        let d5 = tau_scaled_torus_distance(&a, &b, 5);
        assert!(d5 < d1, "τ-scaled distance should decrease with step index");
    }

    #[test]
    fn origin_address() {
        let origin = TorusAddress::origin(3);
        assert_eq!(origin.dimensions(), 3);
        for i in 0..3 {
            assert_eq!(origin.trits.get(i), Gf3::ZERO);
        }
    }

    #[test]
    fn routing_vector_self_is_zero() {
        let addr = TorusAddress::new(vec![Gf3::ONE, Gf3::TWO, Gf3::ZERO]);
        let rv = addr.routing_vector(&addr);
        for i in 0..3 {
            assert_eq!(rv.get(i), Gf3::ZERO, "Self-routing vector should be zero");
        }
    }

    #[test]
    fn distance_self_is_zero() {
        let addr = TorusAddress::new(vec![Gf3::ONE, Gf3::TWO, Gf3::ZERO]);
        assert_eq!(addr.distance(&addr), 0);
    }

    #[test]
    fn distance_is_symmetric() {
        let a = TorusAddress::new(vec![Gf3::ONE, Gf3::TWO, Gf3::ZERO]);
        let b = TorusAddress::new(vec![Gf3::TWO, Gf3::ONE, Gf3::ONE]);
        assert_eq!(a.distance(&b), b.distance(&a));
    }

    #[test]
    fn diameter_is_n() {
        // In 3D torus, max distance should be 3 (all dimensions differ)
        let origin = TorusAddress::origin(3);
        let far = TorusAddress::new(vec![Gf3::ONE, Gf3::ONE, Gf3::ONE]);
        assert_eq!(origin.distance(&far), 3);
    }

    #[test]
    fn neighbors_count() {
        let addr = TorusAddress::origin(3);
        let nbrs = addr.neighbors();
        assert_eq!(nbrs.len(), 6); // 2 per dimension × 3 dimensions
    }

    #[test]
    fn neighbors_are_distance_one() {
        let addr = TorusAddress::origin(4);
        for nbr in addr.neighbors() {
            assert_eq!(addr.distance(&nbr), 1, "Neighbor {:?} not distance 1", nbr);
        }
    }

    #[test]
    fn routing_path_reaches_destination() {
        let src = TorusAddress::origin(3);
        let dst = TorusAddress::new(vec![Gf3::TWO, Gf3::ONE, Gf3::TWO]);
        let path = src.route_to(&dst);
        // Path should end at destination
        assert_eq!(path.last().unwrap(), &dst);
        // Path length should equal distance
        assert_eq!(path.len(), src.distance(&dst));
    }

    #[test]
    fn routing_consecutive_hops_are_neighbors() {
        let src = TorusAddress::new(vec![Gf3::ONE, Gf3::ZERO, Gf3::TWO]);
        let dst = TorusAddress::new(vec![Gf3::TWO, Gf3::TWO, Gf3::ONE]);
        let path = src.route_to(&dst);

        let mut prev = src;
        for hop in &path {
            assert_eq!(prev.distance(hop), 1, "{:?} → {:?} is not a single hop", prev, hop);
            prev = hop.clone();
        }
    }

    #[test]
    fn torus_properties_3d() {
        let props = torus_properties(3);
        assert_eq!(props.node_count, 27);
        assert_eq!(props.degree, 6);
        assert_eq!(props.diameter, 3);
        assert_eq!(props.total_links, 81);
        assert_eq!(props.bisection_bandwidth, 18);
    }

    #[test]
    fn all_3d_nodes_reachable_within_diameter() {
        // Exhaustively verify: all 27 nodes are within distance 3 of origin
        let origin = TorusAddress::origin(3);
        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                for &c in &Gf3::ALL {
                    let dst = TorusAddress::new(vec![a, b, c]);
                    let d = origin.distance(&dst);
                    assert!(d <= 3, "Node {:?} at distance {d} > diameter 3", dst);
                }
            }
        }
    }

    #[test]
    fn torus_is_vertex_transitive() {
        // Every node should have the same number of neighbors and the same
        // distance distribution. Verify for 3D (27 nodes, tractable).
        let n = 3;
        let mut reference_distances: Option<Vec<usize>> = None;

        for &a in &Gf3::ALL {
            for &b in &Gf3::ALL {
                for &c in &Gf3::ALL {
                    let node = TorusAddress::new(vec![a, b, c]);
                    let mut dist_histogram = vec![0usize; n + 1];

                    for &x in &Gf3::ALL {
                        for &y in &Gf3::ALL {
                            for &z in &Gf3::ALL {
                                let other = TorusAddress::new(vec![x, y, z]);
                                let d = node.distance(&other);
                                dist_histogram[d] += 1;
                            }
                        }
                    }

                    match &reference_distances {
                        None => reference_distances = Some(dist_histogram),
                        Some(ref_dist) => {
                            assert_eq!(
                                &dist_histogram, ref_dist,
                                "Node {:?} has different distance distribution", node
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn topology_report_runs() {
        let report = full_topology_report();
        assert!(!report.is_empty());
        println!("{report}");
    }
}
