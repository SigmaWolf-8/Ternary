// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Cube Registration Service (CRS) — Service 3
//!
//! Allows new cubes to join the network, assigns them a unique Rep C address,
//! and provides the physical endpoint information they need to establish
//! overlay tunnels to their geometric neighbors.
//!
//! ## Design Principle
//!
//! The CRS is a thin coordination layer. It does NOT compute routing,
//! maintain topology, or make forwarding decisions. Its only job is to map
//! cube addresses (computed from geometry) to physical endpoints (IP:port).
//! Neighbor relationships are derived from the address itself — the CRS
//! just tells you where to find those neighbors on the physical network.
//!
//! ## Address Space
//!
//! The 13-trit Rep C address space has 3¹³ = 1,594,323 valid addresses.
//! The allocator maintains a bitmap of used addresses and never produces
//! an address containing zero (Rep C guarantee).
//!
//! ## Recursive Design
//!
//! A CRS at any level of the cube-of-cubes hierarchy works identically:
//! same allocation algorithm, same neighbor computation, same API.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::cube_addr::{CubeAddr, RepCTrit, DIMENSIONS, NEIGHBORS_PER_CUBE, TOTAL_VERTICES};

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Default heartbeat interval expected from registered cubes.
const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 30_000; // 30 seconds

/// Default grace period after deregistration before address reuse.
const DEFAULT_GRACE_PERIOD_SECS: u64 = 86_400; // 24 hours

/// Default offline threshold: if no heartbeat for this long, mark offline.
const DEFAULT_OFFLINE_THRESHOLD_SECS: u64 = 120; // 2 minutes

// ═══════════════════════════════════════════════════════════════════════
// CUBE STATUS
// ═══════════════════════════════════════════════════════════════════════

/// Status of a registered cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CubeStatus {
    /// Sending heartbeats, tunnels expected to be active.
    Active,
    /// Shutting down gracefully — stop sending new traffic.
    Draining,
    /// Missed heartbeats — neighbors have been notified.
    Offline,
}

impl std::fmt::Display for CubeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CubeStatus::Active => write!(f, "active"),
            CubeStatus::Draining => write!(f, "draining"),
            CubeStatus::Offline => write!(f, "offline"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CUBE RECORD — The only stored state in the system
// ═══════════════════════════════════════════════════════════════════════

/// Registration record for a cube in the network.
/// This is the Registry Database entry — the only persistent state
/// in the entire inter-cube infrastructure.
#[derive(Debug, Clone)]
pub struct CubeRecord {
    /// Primary key: 13-trit Rep C address (no zeros, guaranteed).
    pub addr: CubeAddr,
    /// Physical IP:port of the cube's gateway nodes.
    pub endpoints: Vec<SocketAddr>,
    /// Identity public key for tunnel authentication.
    pub public_key: [u8; 32],
    /// Current status.
    pub status: CubeStatus,
    /// Last heartbeat timestamp.
    pub last_heartbeat: Instant,
    /// When this cube was first registered.
    pub registered_at: Instant,
    /// Hierarchical level this CRS manages (0 = root).
    pub level: usize,
}

// ═══════════════════════════════════════════════════════════════════════
// NEIGHBOR INFO — Returned to joining cubes
// ═══════════════════════════════════════════════════════════════════════

/// Information about a geometric neighbor, returned during registration.
#[derive(Debug, Clone)]
pub struct NeighborInfo {
    /// The neighbor's Rep C cube address (computed from trit flips).
    pub addr: CubeAddr,
    /// Physical endpoint (if the neighbor is registered).
    pub endpoint: Option<SocketAddr>,
    /// Public key (if the neighbor is registered).
    pub public_key: Option<[u8; 32]>,
    /// Status (if registered).
    pub status: Option<CubeStatus>,
}

// ═══════════════════════════════════════════════════════════════════════
// REGISTRATION RESULT
// ═══════════════════════════════════════════════════════════════════════

/// Result of a successful cube registration.
#[derive(Debug, Clone)]
pub struct RegistrationResult {
    /// The assigned Rep C address.
    pub address: CubeAddr,
    /// Computed neighbors with their endpoint info.
    pub neighbors: Vec<NeighborInfo>,
}

/// Errors during registration.
#[derive(Debug, Clone, PartialEq)]
pub enum RegistrationError {
    /// Address space is full.
    AddressSpaceExhausted,
    /// Requested address is already in use.
    AddressInUse,
    /// Requested address contains zero (invalid Rep C).
    InvalidAddress,
    /// Public key is required.
    MissingPublicKey,
}

// ═══════════════════════════════════════════════════════════════════════
// ADDRESS ALLOCATOR — Manages the Rep C address space
// ═══════════════════════════════════════════════════════════════════════

/// Bitmap-based address allocator for the 3¹³ = 1,594,323 address space.
///
/// Guarantees all allocated addresses are valid Rep C (no zeros).
/// By construction of the flat_index ↔ CubeAddr bijection, every index
/// maps to a valid Rep C address — zero cannot appear.
struct AddressAllocator {
    /// Bitmap: true = in use.
    used: Vec<bool>,
    /// Next allocation hint (sequential scan optimization).
    next_hint: u64,
    /// Count of used addresses.
    used_count: u64,
    /// Addresses in grace period (recently deregistered).
    grace_period: HashMap<u64, Instant>,
    /// Grace period duration.
    grace_duration: Duration,
}

impl AddressAllocator {
    fn new() -> Self {
        AddressAllocator {
            used: vec![false; TOTAL_VERTICES as usize],
            next_hint: 0,
            used_count: 0,
            grace_period: HashMap::new(),
            grace_duration: Duration::from_secs(DEFAULT_GRACE_PERIOD_SECS),
        }
    }

    /// Allocate the next available address.
    fn allocate(&mut self) -> Option<CubeAddr> {
        let now = Instant::now();
        let total = TOTAL_VERTICES as usize;

        // Scan from hint position
        for offset in 0..total {
            let idx = ((self.next_hint as usize) + offset) % total;
            if !self.used[idx] {
                // Check grace period
                if let Some(released_at) = self.grace_period.get(&(idx as u64)) {
                    if now.duration_since(*released_at) < self.grace_duration {
                        continue; // Still in grace period
                    }
                    self.grace_period.remove(&(idx as u64));
                }

                self.used[idx] = true;
                self.used_count += 1;
                self.next_hint = ((idx + 1) % total) as u64;
                return CubeAddr::from_flat_index(idx as u64);
            }
        }
        None // Address space exhausted
    }

    /// Allocate a specific requested address.
    fn allocate_specific(&mut self, addr: &CubeAddr) -> bool {
        let idx = addr.flat_index() as usize;
        if self.used[idx] {
            return false;
        }
        if let Some(released_at) = self.grace_period.get(&(idx as u64)) {
            if Instant::now().duration_since(*released_at) < self.grace_duration {
                return false; // Still in grace period
            }
            self.grace_period.remove(&(idx as u64));
        }
        self.used[idx] = true;
        self.used_count += 1;
        true
    }

    /// Release an address (enters grace period).
    fn release(&mut self, addr: &CubeAddr) {
        let idx = addr.flat_index() as usize;
        if self.used[idx] {
            self.used[idx] = false;
            self.used_count -= 1;
            self.grace_period.insert(idx as u64, Instant::now());
        }
    }

    /// Number of used addresses.
    fn count(&self) -> u64 {
        self.used_count
    }

    /// Number of available addresses (excluding grace period).
    fn available(&self) -> u64 {
        TOTAL_VERTICES - self.used_count
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CUBE REGISTRATION SERVICE
// ═══════════════════════════════════════════════════════════════════════

/// The Cube Registration Service coordinator.
///
/// Manages address allocation, endpoint registry, and neighbor computation.
/// In production, runs as a 3–5 node Raft cluster for fault tolerance.
pub struct CubeRegistrationService {
    /// Address allocator (bitmap over 3¹³ space).
    allocator: AddressAllocator,
    /// Registry database: addr → CubeRecord.
    registry: HashMap<CubeAddr, CubeRecord>,
    /// Expected heartbeat interval.
    heartbeat_interval: Duration,
    /// Offline threshold.
    offline_threshold: Duration,
    /// Hierarchical level this CRS manages (0 = root).
    level: usize,
}

impl CubeRegistrationService {
    /// Create a new CRS coordinator.
    pub fn new() -> Self {
        CubeRegistrationService {
            allocator: AddressAllocator::new(),
            registry: HashMap::new(),
            heartbeat_interval: Duration::from_millis(DEFAULT_HEARTBEAT_INTERVAL_MS),
            offline_threshold: Duration::from_secs(DEFAULT_OFFLINE_THRESHOLD_SECS),
            level: 0,
        }
    }

    /// Create for a specific hierarchy level.
    pub fn at_level(mut self, level: usize) -> Self {
        self.level = level;
        self
    }

    // ═══════════════════════════════════════════════════════════════
    // REGISTRATION
    // ═══════════════════════════════════════════════════════════════

    /// Register a new cube. Allocates an address (or uses the requested one)
    /// and returns the address along with neighbor endpoint information.
    ///
    /// This is the bootstrap entry point for a new cube joining the network.
    pub fn register(
        &mut self,
        endpoint: SocketAddr,
        public_key: [u8; 32],
        desired_address: Option<CubeAddr>,
    ) -> Result<RegistrationResult, RegistrationError> {
        let now = Instant::now();

        // Allocate address
        let addr = if let Some(desired) = desired_address {
            // Validate desired address is valid Rep C
            let bytes = desired.to_bytes();
            for &b in &bytes {
                if b < 1 || b > 3 {
                    return Err(RegistrationError::InvalidAddress);
                }
            }
            if !self.allocator.allocate_specific(&desired) {
                return Err(RegistrationError::AddressInUse);
            }
            desired
        } else {
            self.allocator
                .allocate()
                .ok_or(RegistrationError::AddressSpaceExhausted)?
        };

        // Create registry record
        let record = CubeRecord {
            addr: addr.clone(),
            endpoints: vec![endpoint],
            public_key,
            status: CubeStatus::Active,
            last_heartbeat: now,
            registered_at: now,
            level: self.level,
        };
        self.registry.insert(addr.clone(), record);

        // Compute 26 neighbors and look up their endpoints
        let neighbors = self.compute_neighbor_info(&addr);

        Ok(RegistrationResult {
            address: addr,
            neighbors,
        })
    }

    /// Compute neighbor info for a cube address.
    ///
    /// This is the pure math part: flip each of the 13 trits to its
    /// 2 alternative values → 26 neighbors. Then look up endpoints.
    /// Computed on every call — not stored.
    pub fn compute_neighbor_info(&self, addr: &CubeAddr) -> Vec<NeighborInfo> {
        let mut neighbors = Vec::with_capacity(NEIGHBORS_PER_CUBE);
        for dim in 0..DIMENSIONS {
            for alt in addr.trit(dim).alternatives() {
                let mut nbr_addr = addr.clone();
                nbr_addr.set_trit(dim, alt);

                let (endpoint, public_key, status) =
                    if let Some(record) = self.registry.get(&nbr_addr) {
                        (
                            record.endpoints.first().copied(),
                            Some(record.public_key),
                            Some(record.status),
                        )
                    } else {
                        (None, None, None) // Neighbor not yet registered
                    };

                neighbors.push(NeighborInfo {
                    addr: nbr_addr,
                    endpoint,
                    public_key,
                    status,
                });
            }
        }
        neighbors
    }

    // ═══════════════════════════════════════════════════════════════
    // LOOKUP
    // ═══════════════════════════════════════════════════════════════

    /// Look up a cube's registration record.
    pub fn lookup(&self, addr: &CubeAddr) -> Option<&CubeRecord> {
        self.registry.get(addr)
    }

    /// Look up just the endpoint for a cube address.
    pub fn lookup_endpoint(&self, addr: &CubeAddr) -> Option<SocketAddr> {
        self.registry
            .get(addr)
            .and_then(|r| r.endpoints.first().copied())
    }

    // ═══════════════════════════════════════════════════════════════
    // HEARTBEAT
    // ═══════════════════════════════════════════════════════════════

    /// Process a heartbeat from a registered cube.
    pub fn heartbeat(&mut self, addr: &CubeAddr, endpoint: SocketAddr) -> bool {
        if let Some(record) = self.registry.get_mut(addr) {
            record.last_heartbeat = Instant::now();
            // Update endpoint if it changed (mobile cubes)
            if !record.endpoints.contains(&endpoint) {
                record.endpoints.push(endpoint);
                // Keep only the 3 most recent endpoints
                if record.endpoints.len() > 3 {
                    record.endpoints.remove(0);
                }
            }
            if record.status == CubeStatus::Offline {
                record.status = CubeStatus::Active; // Recovery
            }
            true
        } else {
            false // Unknown cube
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // DEREGISTRATION
    // ═══════════════════════════════════════════════════════════════

    /// Deregister a cube. The address enters a grace period before reuse.
    pub fn deregister(&mut self, addr: &CubeAddr) -> bool {
        if self.registry.remove(addr).is_some() {
            self.allocator.release(addr);
            true
        } else {
            false
        }
    }

    /// Mark a cube as draining (graceful shutdown).
    pub fn drain(&mut self, addr: &CubeAddr) -> bool {
        if let Some(record) = self.registry.get_mut(addr) {
            record.status = CubeStatus::Draining;
            true
        } else {
            false
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // HEALTH CHECK — Detect offline cubes
    // ═══════════════════════════════════════════════════════════════

    /// Scan all registered cubes and mark those with expired heartbeats
    /// as offline. Returns the list of newly offline cubes.
    pub fn check_heartbeats(&mut self) -> Vec<CubeAddr> {
        let now = Instant::now();
        let threshold = self.offline_threshold;
        let mut newly_offline = Vec::new();

        for (addr, record) in self.registry.iter_mut() {
            if record.status == CubeStatus::Active
                && now.duration_since(record.last_heartbeat) > threshold
            {
                record.status = CubeStatus::Offline;
                newly_offline.push(addr.clone());
            }
        }

        newly_offline
    }

    // ═══════════════════════════════════════════════════════════════
    // STATISTICS
    // ═══════════════════════════════════════════════════════════════

    /// Number of registered cubes.
    pub fn registered_count(&self) -> usize {
        self.registry.len()
    }

    /// Number of available addresses.
    pub fn available_addresses(&self) -> u64 {
        self.allocator.available()
    }

    /// Number of active cubes.
    pub fn active_count(&self) -> usize {
        self.registry
            .values()
            .filter(|r| r.status == CubeStatus::Active)
            .count()
    }

    /// Number of offline cubes.
    pub fn offline_count(&self) -> usize {
        self.registry
            .values()
            .filter(|r| r.status == CubeStatus::Offline)
            .count()
    }

    /// Get all registered cube addresses.
    pub fn all_addresses(&self) -> Vec<&CubeAddr> {
        self.registry.keys().collect()
    }

    /// The hierarchy level this CRS manages.
    pub fn level(&self) -> usize {
        self.level
    }
}

impl Default for CubeRegistrationService {
    fn default() -> Self {
        Self::new()
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

    fn test_endpoint() -> SocketAddr {
        "127.0.0.1:51820".parse().unwrap()
    }

    fn test_key() -> [u8; 32] {
        [0xAB; 32]
    }

    #[test]
    fn test_register_auto_address() {
        let mut crs = CubeRegistrationService::new();
        let result = crs.register(test_endpoint(), test_key(), None).unwrap();
        // Should get a valid Rep C address
        let bytes = result.address.to_bytes();
        for &b in &bytes {
            assert!(b >= 1 && b <= 3, "Address must be Rep C");
        }
        // Should get 26 neighbors
        assert_eq!(result.neighbors.len(), NEIGHBORS_PER_CUBE);
    }

    #[test]
    fn test_register_specific_address() {
        let mut crs = CubeRegistrationService::new();
        let desired = addr([2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2]);
        let result = crs
            .register(test_endpoint(), test_key(), Some(desired.clone()))
            .unwrap();
        assert_eq!(result.address, desired);
    }

    #[test]
    fn test_register_duplicate_rejected() {
        let mut crs = CubeRegistrationService::new();
        let desired = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        crs.register(test_endpoint(), test_key(), Some(desired.clone()))
            .unwrap();
        let result = crs.register(test_endpoint(), test_key(), Some(desired));
        assert_eq!(result.unwrap_err(), RegistrationError::AddressInUse);
    }

    #[test]
    fn test_neighbor_info_includes_registered() {
        let mut crs = CubeRegistrationService::new();

        // Register cube A
        let addr_a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        crs.register(
            "10.0.0.1:51820".parse().unwrap(),
            [0x11; 32],
            Some(addr_a.clone()),
        )
        .unwrap();

        // Register cube B (neighbor of A at dim 0)
        let addr_b = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let result = crs
            .register(
                "10.0.0.2:51820".parse().unwrap(),
                [0x22; 32],
                Some(addr_b.clone()),
            )
            .unwrap();

        // B's neighbor info should include A's endpoint
        let a_info = result.neighbors.iter().find(|n| n.addr == addr_a).unwrap();
        assert!(a_info.endpoint.is_some());
        assert_eq!(a_info.status, Some(CubeStatus::Active));
    }

    #[test]
    fn test_lookup() {
        let mut crs = CubeRegistrationService::new();
        let desired = addr([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]);
        crs.register(test_endpoint(), test_key(), Some(desired.clone()))
            .unwrap();

        let record = crs.lookup(&desired).unwrap();
        assert_eq!(record.status, CubeStatus::Active);
        assert_eq!(record.public_key, test_key());
    }

    #[test]
    fn test_heartbeat() {
        let mut crs = CubeRegistrationService::new();
        let desired = addr([1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1]);
        crs.register(test_endpoint(), test_key(), Some(desired.clone()))
            .unwrap();

        assert!(crs.heartbeat(&desired, test_endpoint()));
        // Unknown address returns false
        let unknown = addr([3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]);
        assert!(!crs.heartbeat(&unknown, test_endpoint()));
    }

    #[test]
    fn test_deregister() {
        let mut crs = CubeRegistrationService::new();
        let desired = addr([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2]);
        crs.register(test_endpoint(), test_key(), Some(desired.clone()))
            .unwrap();
        assert_eq!(crs.registered_count(), 1);

        assert!(crs.deregister(&desired));
        assert_eq!(crs.registered_count(), 0);
        assert!(crs.lookup(&desired).is_none());
    }

    #[test]
    fn test_multiple_registrations() {
        let mut crs = CubeRegistrationService::new();
        for i in 0u8..10 {
            let trits = [((i % 3) + 1); 13];
            // Use auto-allocation to avoid collisions
            crs.register(
                format!("10.0.0.{}:51820", i).parse().unwrap(),
                [i; 32],
                None,
            )
            .unwrap();
        }
        assert_eq!(crs.registered_count(), 10);
        assert_eq!(crs.active_count(), 10);
    }

    #[test]
    fn test_address_never_contains_zero() {
        let mut crs = CubeRegistrationService::new();
        // Register 100 cubes with auto-allocated addresses
        for i in 0..100 {
            let result = crs
                .register(
                    format!("10.0.0.{}:{}", i % 256, 51820 + i / 256)
                        .parse()
                        .unwrap(),
                    [i as u8; 32],
                    None,
                )
                .unwrap();
            // Verify every trit is {1, 2, 3}
            for b in result.address.to_bytes() {
                assert!(
                    b >= 1 && b <= 3,
                    "Allocated address must never contain zero"
                );
            }
        }
    }

    #[test]
    fn test_recursive_levels() {
        let root_crs = CubeRegistrationService::new();
        let inner_crs = CubeRegistrationService::new().at_level(1);
        assert_eq!(root_crs.level(), 0);
        assert_eq!(inner_crs.level(), 1);
    }
}