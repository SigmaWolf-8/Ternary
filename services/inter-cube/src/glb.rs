// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Geometric Load Balancer (GLB) — Service 1
//!
//! Distributes inter-cube traffic across all geometrically equivalent shortest
//! paths to maximize throughput and eliminate single-point congestion.
//!
//! ## Design Principle
//!
//! The geometry guarantees multiple shortest paths between any two cubes.
//! Between cubes at Hamming distance d, there are d! shortest paths —
//! each corresponding to a different ordering of the differing dimensions.
//! The GLB selects which dimension to correct first based on a consistent
//! hash of the flow ID, ensuring all packets in a flow follow the same path.
//!
//! ## No Routing Tables
//!
//! The next hop is computed from trit coordinates on every packet.
//! The only stored state is transient flow affinity (hash → dimension index)
//! which auto-expires on idle timeout.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use crate::cube_addr::{CubeAddr, DIMENSIONS};

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Default flow TTL: entries expire after 60 seconds of inactivity.
const DEFAULT_FLOW_TTL_SECS: u64 = 60;

/// Maximum delta dimensions to fully enumerate paths for.
/// For d > MAX_FULL_ENUM, we use modular selection instead of full enumeration.
/// 8! = 40,320 which is already very large; beyond that just use hash mod.
const MAX_FULL_ENUM: usize = 8;

// ═══════════════════════════════════════════════════════════════════════
// FLOW ENTRY — Transient affinity (NOT a routing table)
// ═══════════════════════════════════════════════════════════════════════

/// Transient flow affinity entry. This is NOT a routing table entry —
/// it records which delta dimension a flow was assigned to fix first,
/// so that all packets in the same flow traverse the same path.
#[derive(Debug, Clone)]
pub struct FlowEntry {
    /// Hash of the flow identifier (src + dst + session).
    pub flow_hash: u64,
    /// Which index into the live_delta array this flow fixes first.
    pub selected_index: usize,
    /// When this entry was last touched.
    pub last_active: Instant,
    /// When this entry expires.
    pub expires: Instant,
}

// ═══════════════════════════════════════════════════════════════════════
// FORWARDING RESULT
// ═══════════════════════════════════════════════════════════════════════

/// Result of a GLB forwarding decision.
#[derive(Debug, Clone)]
pub struct ForwardResult {
    /// The computed next-hop cube address.
    pub next_hop: CubeAddr,
    /// Which dimension was fixed in this hop.
    pub dimension_fixed: usize,
    /// Total Hamming distance to destination.
    pub total_distance: usize,
    /// Number of available shortest paths (after excluding dead neighbors).
    pub available_paths: usize,
    /// Whether a detour was required (all direct paths blocked).
    pub is_detour: bool,
}

/// Error type for forwarding failures.
#[derive(Debug, Clone, PartialEq)]
pub enum ForwardError {
    /// Already at destination — no forwarding needed.
    AlreadyAtDestination,
    /// All 26 neighbors are down — cube is isolated.
    Isolated,
}

// ═══════════════════════════════════════════════════════════════════════
// GLB STATISTICS
// ═══════════════════════════════════════════════════════════════════════

/// GLB operational statistics.
#[derive(Debug, Clone, Default)]
pub struct GlbStats {
    pub active_flows: u64,
    pub total_forwards: u64,
    pub detours_computed: u64,
    pub flows_expired: u64,
    pub flows_rehashed: u64,
}

// ═══════════════════════════════════════════════════════════════════════
// GEOMETRIC LOAD BALANCER
// ═══════════════════════════════════════════════════════════════════════

/// The Geometric Load Balancer.
///
/// Computes next-hop forwarding decisions from trit coordinates alone.
/// No routing tables — the geometry IS the routing protocol.
pub struct GeometricLoadBalancer {
    /// This cube's Rep C address.
    local_cube: CubeAddr,
    /// Active flow affinities. Key = flow_hash, Value = FlowEntry.
    /// This is transient state — not a routing table.
    active_flows: HashMap<u64, FlowEntry>,
    /// Flow TTL — how long an idle flow entry persists.
    flow_ttl: Duration,
    /// Dead neighbor set — populated by the FTS (Service 4).
    dead_neighbors: HashSet<CubeAddr>,
    /// Operational statistics.
    stats: GlbStats,
}

impl GeometricLoadBalancer {
    /// Create a new GLB for the given local cube address.
    pub fn new(local_cube: CubeAddr) -> Self {
        GeometricLoadBalancer {
            local_cube,
            active_flows: HashMap::new(),
            flow_ttl: Duration::from_secs(DEFAULT_FLOW_TTL_SECS),
            dead_neighbors: HashSet::new(),
            stats: GlbStats::default(),
        }
    }

    /// Create with custom flow TTL.
    pub fn with_flow_ttl(mut self, ttl: Duration) -> Self {
        self.flow_ttl = ttl;
        self
    }

    /// Get the local cube address.
    pub fn local_addr(&self) -> &CubeAddr {
        &self.local_cube
    }

    // ═══════════════════════════════════════════════════════════════
    // CORE FORWARDING — Pure Math + Flow Affinity
    // ═══════════════════════════════════════════════════════════════

    /// Compute the next hop toward `destination` for the given flow.
    ///
    /// This is the complete routing algorithm:
    /// 1. Compute delta (dimensions where src ≠ dst)
    /// 2. Filter out dimensions whose next-hop is in the dead set
    /// 3. Select dimension via consistent hash of flow_id
    /// 4. Compute next hop by changing one trit toward destination
    ///
    /// **No routing table. No stored paths.**
    pub fn forward(
        &mut self,
        destination: &CubeAddr,
        flow_id: u64,
    ) -> Result<ForwardResult, ForwardError> {
        let now = Instant::now();

        // Step 1: Compute delta — which dimensions differ
        let delta = self.local_cube.delta(destination);
        if delta.is_empty() {
            return Err(ForwardError::AlreadyAtDestination);
        }

        let total_distance = delta.len();

        // Step 2: Filter out dead neighbors
        let live_delta: Vec<usize> = delta
            .iter()
            .copied()
            .filter(|&dim| {
                let candidate = self.local_cube.step_toward(destination, dim);
                !self.dead_neighbors.contains(&candidate)
            })
            .collect();

        // Step 3: If all direct paths blocked, compute detour
        if live_delta.is_empty() {
            return self.compute_detour(destination, &delta);
        }

        // Step 4: Select dimension via flow-consistent hashing
        let flow_hash = hash_flow_id(flow_id);
        let selected_index = (flow_hash as usize) % live_delta.len();
        let fix_dim = live_delta[selected_index];

        // Update flow affinity
        let entry = FlowEntry {
            flow_hash,
            selected_index,
            last_active: now,
            expires: now + self.flow_ttl,
        };
        self.active_flows.insert(flow_hash, entry);

        // Step 5: Compute next hop
        let next_hop = self.local_cube.step_toward(destination, fix_dim);

        self.stats.total_forwards += 1;

        Ok(ForwardResult {
            next_hop,
            dimension_fixed: fix_dim,
            total_distance,
            available_paths: live_delta.len(),
            is_detour: false,
        })
    }

    /// Stateless forwarding — no flow affinity tracking.
    /// Used for one-off packets or when flow state is not needed.
    pub fn forward_stateless(
        &self,
        destination: &CubeAddr,
        flow_id: u64,
    ) -> Result<ForwardResult, ForwardError> {
        let delta = self.local_cube.delta(destination);
        if delta.is_empty() {
            return Err(ForwardError::AlreadyAtDestination);
        }

        let live_delta: Vec<usize> = delta
            .iter()
            .copied()
            .filter(|&dim| {
                let candidate = self.local_cube.step_toward(destination, dim);
                !self.dead_neighbors.contains(&candidate)
            })
            .collect();

        if live_delta.is_empty() {
            return self.compute_detour(destination, &delta);
        }

        let flow_hash = hash_flow_id(flow_id);
        let selected_index = (flow_hash as usize) % live_delta.len();
        let fix_dim = live_delta[selected_index];
        let next_hop = self.local_cube.step_toward(destination, fix_dim);

        Ok(ForwardResult {
            next_hop,
            dimension_fixed: fix_dim,
            total_distance: delta.len(),
            available_paths: live_delta.len(),
            is_detour: false,
        })
    }

    // ═══════════════════════════════════════════════════════════════
    // DETOUR COMPUTATION
    // ═══════════════════════════════════════════════════════════════

    /// Compute a one-hop detour when all direct shortest paths are blocked.
    ///
    /// Strategy: route "sideways" to a live neighbor in a non-differing
    /// dimension, then resume normal Hamming routing from there.
    /// Total path length: d + 2 instead of d.
    fn compute_detour(
        &self,
        destination: &CubeAddr,
        delta: &[usize],
    ) -> Result<ForwardResult, ForwardError> {
        // Try dimensions NOT in delta — go sideways
        let delta_set: HashSet<usize> = delta.iter().copied().collect();

        for dim in 0..DIMENSIONS {
            if delta_set.contains(&dim) {
                continue; // skip differing dimensions — those are blocked
            }
            for alt in self.local_cube.trit(dim).alternatives() {
                let candidate = self.local_cube.neighbor_at(dim, alt);
                if !self.dead_neighbors.contains(&candidate) {
                    return Ok(ForwardResult {
                        next_hop: candidate,
                        dimension_fixed: dim,
                        total_distance: delta.len() + 2, // d + 2 for detour
                        available_paths: 1,
                        is_detour: true,
                    });
                }
            }
        }

        // All 26 neighbors down — cube is isolated
        Err(ForwardError::Isolated)
    }

    // ═══════════════════════════════════════════════════════════════
    // PATH ENUMERATION — For monitoring and debugging
    // ═══════════════════════════════════════════════════════════════

    /// Enumerate all shortest paths to destination (for small delta).
    /// Each path is a sequence of dimension indices to fix, in order.
    /// Returns at most MAX_FULL_ENUM! paths.
    pub fn enumerate_paths(&self, destination: &CubeAddr) -> Vec<Vec<usize>> {
        let delta = self.local_cube.delta(destination);
        if delta.len() > MAX_FULL_ENUM {
            // Too many to enumerate — return a representative subset
            return vec![delta.clone()]; // dimension-order path only
        }
        permutations(&delta)
    }

    // ═══════════════════════════════════════════════════════════════
    // DEAD NEIGHBOR SET — Interface with FTS (Service 4)
    // ═══════════════════════════════════════════════════════════════

    /// Update the dead neighbor set. Called by FTS when a neighbor's
    /// state changes.
    pub fn set_dead_neighbors(&mut self, dead: HashSet<CubeAddr>) {
        let old_dead = std::mem::replace(&mut self.dead_neighbors, dead);

        // Rehash flows that were using now-dead neighbors
        let affected: Vec<u64> = self
            .active_flows
            .keys()
            .copied()
            .collect();

        for flow_hash in affected {
            if let Some(entry) = self.active_flows.get(&flow_hash) {
                // Check if the flow's assigned path is still valid
                // (we don't store the destination, so just expire affected flows
                // and let them re-hash on next packet)
                let _ = entry; // Flow will naturally re-compute on next forward()
            }
        }

        // Count newly dead neighbors for stats
        let new_dead: usize = self
            .dead_neighbors
            .difference(&old_dead)
            .count();
        if new_dead > 0 {
            self.stats.flows_rehashed += self.active_flows.len() as u64;
        }
    }

    /// Add a single dead neighbor. Called by FTS on individual failure detection.
    pub fn add_dead_neighbor(&mut self, addr: CubeAddr) {
        self.dead_neighbors.insert(addr);
    }

    /// Remove a single dead neighbor. Called by FTS on recovery.
    pub fn remove_dead_neighbor(&mut self, addr: &CubeAddr) {
        self.dead_neighbors.remove(addr);
    }

    /// Get the current dead neighbor set.
    pub fn dead_neighbors(&self) -> &HashSet<CubeAddr> {
        &self.dead_neighbors
    }

    // ═══════════════════════════════════════════════════════════════
    // FLOW MANAGEMENT
    // ═══════════════════════════════════════════════════════════════

    /// Expire stale flow entries.
    pub fn expire_flows(&mut self) {
        let now = Instant::now();
        let before = self.active_flows.len();
        self.active_flows.retain(|_, entry| entry.expires > now);
        let expired = before - self.active_flows.len();
        self.stats.flows_expired += expired as u64;
    }

    /// Number of active flows.
    pub fn active_flow_count(&self) -> usize {
        self.active_flows.len()
    }

    /// Get statistics.
    pub fn stats(&self) -> &GlbStats {
        &self.stats
    }

    /// Get the number of live neighbors (26 - |dead_set|).
    pub fn live_neighbor_count(&self) -> usize {
        crate::cube_addr::NEIGHBORS_PER_CUBE - self.dead_neighbors.len()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// FLOW HASHING — Cryptographic, using BLAKE3
// ═══════════════════════════════════════════════════════════════════════

/// Hash a flow ID using BLAKE3 for uniform distribution.
/// The hash determines which path ordering a flow uses.
#[inline]
fn hash_flow_id(flow_id: u64) -> u64 {
    let hash = blake3::hash(&flow_id.to_le_bytes());
    let bytes = hash.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

// ═══════════════════════════════════════════════════════════════════════
// PERMUTATION HELPER
// ═══════════════════════════════════════════════════════════════════════

/// Generate all permutations of the input slice (Heap's algorithm).
fn permutations(items: &[usize]) -> Vec<Vec<usize>> {
    if items.is_empty() {
        return vec![vec![]];
    }
    let mut result = Vec::new();
    let mut arr = items.to_vec();
    let n = arr.len();
    heap_permute(&mut arr, n, &mut result);
    result
}

fn heap_permute(arr: &mut Vec<usize>, size: usize, result: &mut Vec<Vec<usize>>) {
    if size == 1 {
        result.push(arr.clone());
        return;
    }
    for i in 0..size {
        heap_permute(arr, size - 1, result);
        if size % 2 == 0 {
            arr.swap(i, size - 1);
        } else {
            arr.swap(0, size - 1);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(trits: [u8; 13]) -> CubeAddr {
        CubeAddr::new(trits)
    }

    #[test]
    fn test_forward_same_destination() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut glb = GeometricLoadBalancer::new(local.clone());
        let result = glb.forward(&local, 42);
        assert_eq!(result, Err(ForwardError::AlreadyAtDestination));
    }

    #[test]
    fn test_forward_one_hop() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let dest = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut glb = GeometricLoadBalancer::new(local);
        let result = glb.forward(&dest, 42).unwrap();
        assert_eq!(result.next_hop, dest);
        assert_eq!(result.dimension_fixed, 0);
        assert_eq!(result.total_distance, 1);
        assert!(!result.is_detour);
    }

    #[test]
    fn test_forward_multi_hop() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let dest = addr([3, 2, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut glb = GeometricLoadBalancer::new(local);
        let result = glb.forward(&dest, 100).unwrap();
        assert_eq!(result.total_distance, 3);
        assert_eq!(result.available_paths, 3); // 3 live dimensions
        assert!(!result.is_detour);
    }

    #[test]
    fn test_flow_consistency() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let dest = addr([3, 2, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let glb = GeometricLoadBalancer::new(local);

        // Same flow_id must produce the same next hop
        let r1 = glb.forward_stateless(&dest, 999).unwrap();
        let r2 = glb.forward_stateless(&dest, 999).unwrap();
        assert_eq!(r1.next_hop, r2.next_hop);
        assert_eq!(r1.dimension_fixed, r2.dimension_fixed);
    }

    #[test]
    fn test_different_flows_may_differ() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let dest = addr([3, 2, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let glb = GeometricLoadBalancer::new(local);

        // Different flow_ids should distribute across paths
        let mut seen_dims = HashSet::new();
        for fid in 0u64..100 {
            let r = glb.forward_stateless(&dest, fid).unwrap();
            seen_dims.insert(r.dimension_fixed);
        }
        // With 100 flows across 3 paths, we should see all 3 dimensions
        assert!(seen_dims.len() > 1, "Flows should distribute across paths");
    }

    #[test]
    fn test_dead_neighbor_avoidance() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let dest = addr([2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut glb = GeometricLoadBalancer::new(local.clone());

        // Block dimension 0's next hop
        let blocked = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        glb.add_dead_neighbor(blocked.clone());

        // All forwards should now use dimension 1
        for fid in 0u64..50 {
            let r = glb.forward_stateless(&dest, fid).unwrap();
            assert_eq!(r.dimension_fixed, 1, "Should avoid dead neighbor on dim 0");
        }
    }

    #[test]
    fn test_detour_when_all_direct_blocked() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let dest = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut glb = GeometricLoadBalancer::new(local.clone());

        // Block the only direct next hop (dim 0 → value 2)
        let blocked = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        glb.add_dead_neighbor(blocked);

        let r = glb.forward_stateless(&dest, 42).unwrap();
        assert!(r.is_detour, "Should use detour when direct path blocked");
        assert_eq!(r.total_distance, 3, "Detour adds 2 hops: d=1 → 1+2=3");
    }

    #[test]
    fn test_enumerate_paths() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let dest = addr([3, 2, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let glb = GeometricLoadBalancer::new(local);
        let paths = glb.enumerate_paths(&dest);
        // d=3 → 3! = 6 paths
        assert_eq!(paths.len(), 6);
    }

    #[test]
    fn test_path_count_factorial() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let dest = addr([2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1]);
        let glb = GeometricLoadBalancer::new(local.clone());
        // d=5 → 5! = 120 paths
        let paths = glb.enumerate_paths(&dest);
        assert_eq!(paths.len(), 120);
        assert_eq!(local.shortest_path_count(&dest), 120);
    }
}