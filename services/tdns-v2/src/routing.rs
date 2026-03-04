// TDNS v2.3 — Routing
// Capomastro Holdings Ltd. — Applied Physics Division
//
// Greedy geometric forwarding in a 27-dimensional ternary hypercube.
// No routing tables. No convergence. No longest-prefix matching.
// The geometry carries the routing.
//
// Each node stores at most 54 neighbor entries (27 dims × 2 directions).
// Path length = Hamming distance. Worst case: 27 hops. Loop-free.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::addr::{CubeAddr, DIMENSIONS};
use crate::trit::Trit;

// ─── Neighbor Map ────────────────────────────────────────────────────────────

/// A neighbor map entry: for dimension `dim`, flipping to `target_value`,
/// the closest populated node is at `addr`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeighborEntry {
    /// Dimension index (0-based).
    pub dim: usize,
    /// The target trit value in this dimension.
    pub target_value: Trit,
    /// Address of the closest populated node with this trit value.
    pub addr: CubeAddr,
    /// Hamming distance from this node to the neighbor.
    pub distance: u8,
}

/// The 54-entry neighbor map. Constant size regardless of network scale.
///
/// For each dimension i and each direction (the two values != local[i]),
/// we store the closest populated node. CRS maintains these maps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborMap {
    /// This node's address.
    local: CubeAddr,
    /// Map: (dimension, target_value) → neighbor address.
    /// At most 54 entries (27 dims × 2 directions).
    entries: HashMap<(usize, u8), NeighborEntry>,
}

impl NeighborMap {
    /// Create an empty neighbor map for a node.
    pub fn new(local: CubeAddr) -> Self {
        Self {
            local,
            entries: HashMap::with_capacity(54),
        }
    }

    /// This node's address.
    pub fn local(&self) -> &CubeAddr {
        &self.local
    }

    /// Set a neighbor for (dimension, target_value).
    pub fn set(&mut self, dim: usize, target_value: Trit, addr: CubeAddr) {
        let distance = self.local.distance(&addr);
        self.entries.insert(
            (dim, target_value.value()),
            NeighborEntry {
                dim,
                target_value,
                addr,
                distance,
            },
        );
    }

    /// Get the neighbor for (dimension, target_value).
    pub fn get(&self, dim: usize, target_value: Trit) -> Option<&NeighborEntry> {
        self.entries.get(&(dim, target_value.value()))
    }

    /// Remove a neighbor entry (node deregistered).
    pub fn remove(&mut self, dim: usize, target_value: Trit) -> Option<NeighborEntry> {
        self.entries.remove(&(dim, target_value.value()))
    }

    /// Number of populated entries (max 54).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Is the map empty?
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// How many of the 54 possible slots are filled.
    /// Full coverage = efficient routing in all directions.
    pub fn coverage(&self) -> f64 {
        self.entries.len() as f64 / 54.0
    }

    /// Dimensions with no neighbor in either direction.
    pub fn uncovered_dims(&self) -> Vec<usize> {
        (0..DIMENSIONS)
            .filter(|&dim| {
                let local_val = self.local.trit(dim);
                local_val
                    .neighbors()
                    .iter()
                    .all(|&tv| self.get(dim, tv).is_none())
            })
            .collect()
    }

    /// All entries, sorted by dimension then target value.
    pub fn all_entries(&self) -> Vec<&NeighborEntry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_by_key(|e| (e.dim, e.target_value.value()));
        entries
    }
}

// ─── Greedy Forwarding ──────────────────────────────────────────────────────

/// The result of a single forwarding step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForwardResult {
    /// Destination reached — we are the target.
    Arrived,
    /// Forward to this neighbor (next hop).
    NextHop {
        /// The neighbor to forward to.
        next: CubeAddr,
        /// Which dimension we're correcting.
        dim: usize,
        /// Remaining hops (Hamming distance to destination from next hop).
        remaining: u8,
    },
    /// No neighbor available in any differing dimension.
    /// Routing failure — sparse cube, insufficient population.
    NoRoute {
        /// Dimensions that differ but have no populated neighbor.
        stuck_dims: Vec<usize>,
    },
}

/// Compute the next hop from `current` toward `destination` using `map`.
///
/// Algorithm (§11.1):
/// 1. Compare current and destination trit-by-trit.
/// 2. Find the first dimension i where they differ.
/// 3. Look up neighbor_map[i][destination[i]].
/// 4. Forward to that neighbor.
///
/// If no neighbor exists for the first differing dimension, try
/// subsequent differing dimensions (sparse fallback, §11.2).
pub fn forward(
    current: &CubeAddr,
    destination: &CubeAddr,
    map: &NeighborMap,
) -> ForwardResult {
    // Are we there?
    if current == destination {
        return ForwardResult::Arrived;
    }

    // Find all differing dimensions, ordered by index (WHO before WHAT before ...).
    let diffs = current.differing_dims(destination);

    // Try each differing dimension in priority order.
    for &dim in &diffs {
        let target_value = destination.trit(dim);
        if let Some(entry) = map.get(dim, target_value) {
            let remaining = entry.addr.distance(destination);
            return ForwardResult::NextHop {
                next: entry.addr,
                dim,
                remaining,
            };
        }
    }

    // No route found in any differing dimension.
    ForwardResult::NoRoute {
        stuck_dims: diffs,
    }
}

/// Compute the full path from source to destination.
///
/// Returns the sequence of addresses (including source and destination).
/// Uses the provided function to look up neighbor maps for each hop.
///
/// Returns None if routing fails at any point (sparse cube gap).
pub fn compute_path<F>(
    source: &CubeAddr,
    destination: &CubeAddr,
    mut get_map: F,
) -> Option<Vec<CubeAddr>>
where
    F: FnMut(&CubeAddr) -> NeighborMap,
{
    let mut path = vec![*source];
    let mut current = *source;
    let max_hops = DIMENSIONS as u8; // 27 — absolute worst case

    for _ in 0..=max_hops {
        if current == *destination {
            return Some(path);
        }

        let map = get_map(&current);
        match forward(&current, destination, &map) {
            ForwardResult::Arrived => {
                return Some(path);
            }
            ForwardResult::NextHop { next, .. } => {
                path.push(next);
                current = next;
            }
            ForwardResult::NoRoute { .. } => {
                return None;
            }
        }
    }

    None // Should never reach — 27 hops max in a 27-dim hypercube
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn google() -> CubeAddr {
        CubeAddr::from_category_string("WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313")
            .unwrap()
    }

    fn pptpro() -> CubeAddr {
        CubeAddr::from_category_string("WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332")
            .unwrap()
    }

    fn blog() -> CubeAddr {
        CubeAddr::from_category_string("WO:1312 WA:1111 WR:3111 WN:2311 WY:1111 HO:1111 PE:211")
            .unwrap()
    }

    #[test]
    fn forward_to_self_is_arrived() {
        let g = google();
        let map = NeighborMap::new(g);
        assert_eq!(forward(&g, &g, &map), ForwardResult::Arrived);
    }

    #[test]
    fn forward_with_direct_neighbor() {
        let g = google();
        let p = pptpro();

        let mut map = NeighborMap::new(g);

        // Google and PPTPro differ first at dim 2 (trit 2: Google=3, PPTPro=3 — same!)
        // Actually let's check the diff
        let diffs = g.differing_dims(&p);

        // Set neighbors for all differing dims pointing to PPTPro
        for &dim in &diffs {
            map.set(dim, p.trit(dim), p);
        }

        match forward(&g, &p, &map) {
            ForwardResult::NextHop { next, dim, .. } => {
                assert_eq!(next, p);
                assert_eq!(dim, diffs[0]); // First differing dim
            }
            other => panic!("expected NextHop, got {:?}", other),
        }
    }

    #[test]
    fn forward_no_route_when_empty_map() {
        let g = google();
        let p = pptpro();
        let map = NeighborMap::new(g);

        match forward(&g, &p, &map) {
            ForwardResult::NoRoute { stuck_dims } => {
                assert!(!stuck_dims.is_empty());
            }
            other => panic!("expected NoRoute, got {:?}", other),
        }
    }

    #[test]
    fn neighbor_map_coverage() {
        let g = google();
        let mut map = NeighborMap::new(g);
        assert_eq!(map.len(), 0);
        assert_eq!(map.coverage(), 0.0);

        // Fill all 54 slots with dummy neighbors
        for dim in 0..DIMENSIONS {
            for target in g.trit(dim).neighbors() {
                let neighbor = g.with_trit(dim, target);
                map.set(dim, target, neighbor);
            }
        }

        assert_eq!(map.len(), 54);
        assert!((map.coverage() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn uncovered_dims_detection() {
        let g = google();
        let map = NeighborMap::new(g);
        let uncovered = map.uncovered_dims();
        assert_eq!(uncovered.len(), DIMENSIONS); // All uncovered when empty
    }

    #[test]
    fn compute_path_single_hop() {
        let g = google();
        // Create a neighbor one trit away
        let neighbor = g.with_trit(0, Trit::V1); // Flip dim 0

        let path = compute_path(&g, &neighbor, |addr| {
            let mut map = NeighborMap::new(*addr);
            if *addr == g {
                map.set(0, Trit::V1, neighbor);
            }
            map
        });

        assert_eq!(path, Some(vec![g, neighbor]));
    }

    #[test]
    fn compute_path_multi_hop() {
        // Create a 3-hop path: A → B → C → D
        let a = CubeAddr::from_values(&[1; 27]).unwrap();
        let b = a.with_trit(0, Trit::V2); // flip dim 0
        let c = b.with_trit(1, Trit::V2); // flip dim 1
        let d = c.with_trit(2, Trit::V2); // flip dim 2

        assert_eq!(a.distance(&d), 3);

        let path = compute_path(&a, &d, |addr| {
            let mut map = NeighborMap::new(*addr);
            if *addr == a {
                map.set(0, Trit::V2, b);
                map.set(1, Trit::V2, c);
                map.set(2, Trit::V2, d);
            } else if *addr == b {
                map.set(1, Trit::V2, c);
                map.set(2, Trit::V2, d);
            } else if *addr == c {
                map.set(2, Trit::V2, d);
            }
            map
        });

        assert_eq!(path, Some(vec![a, b, c, d]));
    }

    #[test]
    fn sparse_fallback_skips_dim() {
        // Destination differs at dims 0 and 1.
        // Dim 0 has no neighbor. Dim 1 does. Should fall back to dim 1.
        let src = CubeAddr::from_values(&[1; 27]).unwrap();
        let dst = src.with_trit(0, Trit::V3).with_trit(1, Trit::V3);
        let intermediate = src.with_trit(1, Trit::V3); // Only dim 1 flipped

        let mut map = NeighborMap::new(src);
        // Don't set dim 0 neighbor — simulate sparse gap
        map.set(1, Trit::V3, intermediate);

        match forward(&src, &dst, &map) {
            ForwardResult::NextHop { next, dim, .. } => {
                assert_eq!(next, intermediate);
                assert_eq!(dim, 1); // Fell back to dim 1
            }
            other => panic!("expected NextHop fallback, got {:?}", other),
        }
    }
}
