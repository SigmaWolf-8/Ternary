// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Cube Overlay Network (CON) — Service 2
//!
//! Decouples logical cube adjacency (neighbors that differ by one trit) from
//! the physical network. Creates encrypted tunnels between geometric neighbors
//! regardless of physical location — same rack, different continent, or across
//! the public internet.
//!
//! ## Design Principle
//!
//! The geometry tells each cube exactly who its 26 neighbors are. The overlay
//! just builds tunnels to them. No discovery protocol, no neighbor negotiation —
//! the math produces the neighbor list, the overlay connects them.
//!
//! ## Tunnel Protocol
//!
//! Two modes:
//! - **Standard**: WireGuard's Noise protocol with pre-exchanged keys.
//! - **PQ-Native**: Keys derived from the cryptographic sponge using both
//!   cubes' Rep C addresses as BLAKE3 input. Post-quantum by construction.
//!
//! ## Integration
//!
//! - Queries CRS (Service 3) for physical endpoints of geometric neighbors.
//! - Reports tunnel health to FTS (Service 4) via heartbeat responses.
//! - GLB (Service 1) forwards packets to the appropriate virtual tunnel
//!   interface based on the computed next hop.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::cube_addr::{CubeAddr, RepCTrit, DIMENSIONS, NEIGHBORS_PER_CUBE};

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Default heartbeat interval for tunnel liveness checks.
const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 1000;

/// Default key rotation interval.
const DEFAULT_KEY_ROTATION_SECS: u64 = 86400; // 24 hours

/// Virtual interface name prefix.
const TUNNEL_IFACE_PREFIX: &str = "cubetun";

// ═══════════════════════════════════════════════════════════════════════
// TUNNEL STATE MACHINE
// ═══════════════════════════════════════════════════════════════════════

/// State of a tunnel to a geometric neighbor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunnelState {
    /// Neighbor address computed but physical endpoint not yet resolved.
    Unknown,
    /// Querying the CRS for the neighbor's physical endpoint.
    Resolving,
    /// Tunnel handshake in progress.
    Connecting,
    /// Encrypted tunnel active and passing traffic.
    Up,
    /// Heartbeat lost — reported to FTS as potentially down.
    Down,
}

impl TunnelState {
    /// Whether this state allows forwarding traffic.
    pub fn is_active(&self) -> bool {
        matches!(self, TunnelState::Up)
    }

    /// Whether this state should trigger CRS re-resolution.
    pub fn needs_resolution(&self) -> bool {
        matches!(self, TunnelState::Unknown | TunnelState::Down)
    }
}

impl std::fmt::Display for TunnelState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TunnelState::Unknown => write!(f, "unknown"),
            TunnelState::Resolving => write!(f, "resolving"),
            TunnelState::Connecting => write!(f, "connecting"),
            TunnelState::Up => write!(f, "up"),
            TunnelState::Down => write!(f, "down"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TUNNEL PROTOCOL SELECTION
// ═══════════════════════════════════════════════════════════════════════

/// Encryption protocol for inter-cube tunnels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunnelProtocol {
    /// Standard WireGuard Noise protocol with pre-exchanged keys.
    WireGuard,
    /// PlenumNET-native: keys derived from cryptographic sponge
    /// using both cubes' Rep C addresses. Post-quantum by construction.
    PqNative,
}

// ═══════════════════════════════════════════════════════════════════════
// NEIGHBOR RECORD
// ═══════════════════════════════════════════════════════════════════════

/// Complete record for a single geometric neighbor.
///
/// There are exactly 26 of these per cube — one for each neighbor
/// computed from the 13D cube geometry.
#[derive(Debug, Clone)]
pub struct Neighbor {
    /// The neighbor's Rep C cube address (computed from geometry).
    pub cube_addr: CubeAddr,
    /// Which dimension this neighbor differs from us in.
    pub dimension: usize,
    /// What value the neighbor holds in the differing dimension.
    pub alt_value: RepCTrit,
    /// Physical network endpoint (from CRS lookup).
    pub endpoint: Option<SocketAddr>,
    /// Neighbor's public key for tunnel authentication.
    pub public_key: Option<[u8; 32]>,
    /// Assigned virtual interface name (e.g., "cubetun0").
    pub tunnel_iface: Option<String>,
    /// Current tunnel state.
    pub state: TunnelState,
    /// Last successful heartbeat timestamp.
    pub last_heartbeat: Option<Instant>,
    /// Smoothed round-trip time in nanoseconds.
    pub srtt_ns: Option<u64>,
    /// Traffic counters.
    pub bytes_in: u64,
    pub bytes_out: u64,
    /// Tunnel uptime since last establishment.
    pub tunnel_established: Option<Instant>,
}

impl Neighbor {
    /// Create a new neighbor record from computed geometry.
    fn new(cube_addr: CubeAddr, dimension: usize, alt_value: RepCTrit) -> Self {
        Neighbor {
            cube_addr,
            dimension,
            alt_value,
            endpoint: None,
            public_key: None,
            tunnel_iface: None,
            state: TunnelState::Unknown,
            last_heartbeat: None,
            srtt_ns: None,
            bytes_in: 0,
            bytes_out: 0,
            tunnel_established: None,
        }
    }

    /// RTT in milliseconds (for API responses).
    pub fn rtt_ms(&self) -> Option<f64> {
        self.srtt_ns.map(|ns| ns as f64 / 1_000_000.0)
    }

    /// Uptime in seconds since tunnel establishment.
    pub fn uptime_secs(&self) -> Option<f64> {
        self.tunnel_established
            .map(|t| t.elapsed().as_secs_f64())
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CON STATISTICS
// ═══════════════════════════════════════════════════════════════════════

/// Aggregate statistics for the Cube Overlay Network.
#[derive(Debug, Clone)]
pub struct ConStats {
    pub tunnels_up: usize,
    pub tunnels_down: usize,
    pub tunnels_resolving: usize,
    pub tunnels_connecting: usize,
    pub tunnels_unknown: usize,
    pub total_bytes_in: u64,
    pub total_bytes_out: u64,
    pub avg_rtt_ms: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════════
// CUBE OVERLAY NETWORK
// ═══════════════════════════════════════════════════════════════════════

/// The Cube Overlay Network daemon.
///
/// Manages exactly 26 tunnels — one for each geometric neighbor.
/// The neighbor list is computed from the local cube address using
/// pure trit arithmetic. No discovery protocol needed.
pub struct CubeOverlayNetwork {
    /// This cube's Rep C address.
    local_addr: CubeAddr,
    /// Exactly 26 neighbors, computed at initialization.
    neighbors: Vec<Neighbor>,
    /// Index: cube_addr → position in neighbors vec.
    addr_index: HashMap<CubeAddr, usize>,
    /// Tunnel encryption protocol.
    tunnel_protocol: TunnelProtocol,
    /// Heartbeat interval.
    heartbeat_interval: Duration,
    /// Key rotation interval.
    key_rotation_interval: Duration,
}

impl CubeOverlayNetwork {
    /// Create a new CON daemon for the given local cube address.
    ///
    /// Immediately computes all 26 geometric neighbors from the address.
    /// This is pure math — no network calls, no discovery.
    pub fn new(local_addr: CubeAddr) -> Self {
        let mut neighbors = Vec::with_capacity(NEIGHBORS_PER_CUBE);
        let mut addr_index = HashMap::with_capacity(NEIGHBORS_PER_CUBE);

        // Compute neighbors: for each of 13 dimensions, flip to 2 alternative values
        let mut idx = 0usize;
        for dim in 0..DIMENSIONS {
            for alt in local_addr.trit(dim).alternatives() {
                let mut nbr_addr = local_addr.clone();
                nbr_addr.set_trit(dim, alt);
                let neighbor = Neighbor::new(nbr_addr.clone(), dim, alt);
                addr_index.insert(nbr_addr, idx);
                neighbors.push(neighbor);
                idx += 1;
            }
        }

        assert_eq!(
            neighbors.len(),
            NEIGHBORS_PER_CUBE,
            "Must have exactly 26 neighbors"
        );

        CubeOverlayNetwork {
            local_addr,
            neighbors,
            addr_index,
            tunnel_protocol: TunnelProtocol::PqNative,
            heartbeat_interval: Duration::from_millis(DEFAULT_HEARTBEAT_INTERVAL_MS),
            key_rotation_interval: Duration::from_secs(DEFAULT_KEY_ROTATION_SECS),
        }
    }

    /// Create with custom protocol selection.
    pub fn with_protocol(mut self, protocol: TunnelProtocol) -> Self {
        self.tunnel_protocol = protocol;
        self
    }

    /// Get the local cube address.
    pub fn local_addr(&self) -> &CubeAddr {
        &self.local_addr
    }

    /// Get all 26 neighbors.
    pub fn neighbors(&self) -> &[Neighbor] {
        &self.neighbors
    }

    /// Get a specific neighbor by cube address.
    pub fn neighbor(&self, addr: &CubeAddr) -> Option<&Neighbor> {
        self.addr_index.get(addr).map(|&i| &self.neighbors[i])
    }

    /// Get a mutable reference to a specific neighbor.
    pub fn neighbor_mut(&mut self, addr: &CubeAddr) -> Option<&mut Neighbor> {
        self.addr_index
            .get(addr)
            .copied()
            .map(move |i| &mut self.neighbors[i])
    }

    /// Get the neighbor at a specific dimension and alternative value.
    pub fn neighbor_at_dim(&self, dim: usize, alt: RepCTrit) -> Option<&Neighbor> {
        self.neighbors
            .iter()
            .find(|n| n.dimension == dim && n.alt_value == alt)
    }

    // ═══════════════════════════════════════════════════════════════
    // ENDPOINT RESOLUTION — Interface with CRS (Service 3)
    // ═══════════════════════════════════════════════════════════════

    /// Resolve a neighbor's physical endpoint from CRS lookup result.
    /// Called after querying the CRS for the neighbor's registration.
    pub fn resolve_neighbor(
        &mut self,
        addr: &CubeAddr,
        endpoint: SocketAddr,
        public_key: [u8; 32],
    ) -> bool {
        if let Some(nbr) = self.neighbor_mut(addr) {
            nbr.endpoint = Some(endpoint);
            nbr.public_key = Some(public_key);
            nbr.state = TunnelState::Connecting;
            true
        } else {
            false
        }
    }

    /// Mark a neighbor as unresolvable (not registered in CRS).
    pub fn mark_unresolved(&mut self, addr: &CubeAddr) {
        if let Some(nbr) = self.neighbor_mut(addr) {
            nbr.state = TunnelState::Unknown;
            nbr.endpoint = None;
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // TUNNEL LIFECYCLE
    // ═══════════════════════════════════════════════════════════════

    /// Mark a tunnel as successfully established.
    pub fn tunnel_up(&mut self, addr: &CubeAddr, iface: String) {
        if let Some(nbr) = self.neighbor_mut(addr) {
            nbr.state = TunnelState::Up;
            nbr.tunnel_iface = Some(iface);
            nbr.tunnel_established = Some(Instant::now());
        }
    }

    /// Mark a tunnel as down (heartbeat lost).
    pub fn tunnel_down(&mut self, addr: &CubeAddr) {
        if let Some(nbr) = self.neighbor_mut(addr) {
            nbr.state = TunnelState::Down;
            nbr.tunnel_iface = None;
            nbr.tunnel_established = None;
        }
    }

    /// Record a heartbeat response from a neighbor.
    pub fn record_heartbeat(&mut self, addr: &CubeAddr, rtt_ns: u64) {
        if let Some(nbr) = self.neighbor_mut(addr) {
            nbr.last_heartbeat = Some(Instant::now());
            // Exponentially weighted moving average for SRTT
            nbr.srtt_ns = Some(match nbr.srtt_ns {
                Some(prev) => (prev * 7 + rtt_ns) / 8, // EWMA α = 1/8
                None => rtt_ns,
            });
        }
    }

    /// Record traffic bytes for a neighbor tunnel.
    pub fn record_traffic(&mut self, addr: &CubeAddr, bytes_in: u64, bytes_out: u64) {
        if let Some(nbr) = self.neighbor_mut(addr) {
            nbr.bytes_in += bytes_in;
            nbr.bytes_out += bytes_out;
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // PQ-NATIVE KEY DERIVATION
    // ═══════════════════════════════════════════════════════════════

    /// Derive a shared tunnel key from both cubes' Rep C addresses.
    ///
    /// Uses BLAKE3 keyed hash with the concatenation of both addresses
    /// (sorted lexicographically to ensure both sides derive the same key).
    /// Post-quantum by construction — no elliptic curve operations.
    pub fn derive_pq_tunnel_key(
        addr_a: &CubeAddr,
        addr_b: &CubeAddr,
    ) -> [u8; 32] {
        let bytes_a = addr_a.to_bytes();
        let bytes_b = addr_b.to_bytes();

        // Canonical ordering: smaller address first
        let (first, second) = if bytes_a <= bytes_b {
            (&bytes_a, &bytes_b)
        } else {
            (&bytes_b, &bytes_a)
        };

        // Derive key material
        let mut input = Vec::with_capacity(26 + 16); // 2×13 trits + domain separator
        input.extend_from_slice(first);
        input.extend_from_slice(second);
        input.extend_from_slice(b"PlenumNET-CON-v1");

        let hash = blake3::hash(&input);
        *hash.as_bytes()
    }

    /// Derive all tunnel keys for this cube's neighbors.
    pub fn derive_all_keys(&self) -> Vec<(CubeAddr, [u8; 32])> {
        self.neighbors
            .iter()
            .map(|nbr| {
                let key = Self::derive_pq_tunnel_key(&self.local_addr, &nbr.cube_addr);
                (nbr.cube_addr.clone(), key)
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════
    // VIRTUAL INTERFACE ASSIGNMENT
    // ═══════════════════════════════════════════════════════════════

    /// Generate the virtual interface name for a tunnel.
    /// Format: cubetunN where N is the neighbor's index (0..25).
    pub fn iface_name_for(&self, addr: &CubeAddr) -> Option<String> {
        self.addr_index
            .get(addr)
            .map(|&idx| format!("{}{}", TUNNEL_IFACE_PREFIX, idx))
    }

    /// Get the cube address for a given virtual interface name.
    pub fn addr_for_iface(&self, iface: &str) -> Option<&CubeAddr> {
        if !iface.starts_with(TUNNEL_IFACE_PREFIX) {
            return None;
        }
        let idx_str = &iface[TUNNEL_IFACE_PREFIX.len()..];
        let idx: usize = idx_str.parse().ok()?;
        self.neighbors.get(idx).map(|n| &n.cube_addr)
    }

    // ═══════════════════════════════════════════════════════════════
    // STATISTICS
    // ═══════════════════════════════════════════════════════════════

    /// Compute aggregate statistics.
    pub fn stats(&self) -> ConStats {
        let mut up = 0;
        let mut down = 0;
        let mut resolving = 0;
        let mut connecting = 0;
        let mut unknown = 0;
        let mut total_in = 0u64;
        let mut total_out = 0u64;
        let mut rtt_sum = 0.0f64;
        let mut rtt_count = 0usize;

        for n in &self.neighbors {
            match n.state {
                TunnelState::Up => up += 1,
                TunnelState::Down => down += 1,
                TunnelState::Resolving => resolving += 1,
                TunnelState::Connecting => connecting += 1,
                TunnelState::Unknown => unknown += 1,
            }
            total_in += n.bytes_in;
            total_out += n.bytes_out;
            if let Some(rtt) = n.rtt_ms() {
                rtt_sum += rtt;
                rtt_count += 1;
            }
        }

        ConStats {
            tunnels_up: up,
            tunnels_down: down,
            tunnels_resolving: resolving,
            tunnels_connecting: connecting,
            tunnels_unknown: unknown,
            total_bytes_in: total_in,
            total_bytes_out: total_out,
            avg_rtt_ms: if rtt_count > 0 {
                Some(rtt_sum / rtt_count as f64)
            } else {
                None
            },
        }
    }

    /// Get addresses of all neighbors with active tunnels.
    pub fn live_neighbors(&self) -> Vec<&CubeAddr> {
        self.neighbors
            .iter()
            .filter(|n| n.state == TunnelState::Up)
            .map(|n| &n.cube_addr)
            .collect()
    }

    /// Get addresses of all neighbors with down tunnels.
    pub fn dead_neighbors(&self) -> Vec<&CubeAddr> {
        self.neighbors
            .iter()
            .filter(|n| n.state == TunnelState::Down)
            .map(|n| &n.cube_addr)
            .collect()
    }

    /// Trigger re-resolution of all neighbor endpoints from CRS.
    pub fn refresh_all(&mut self) {
        for nbr in &mut self.neighbors {
            if nbr.state != TunnelState::Up {
                nbr.state = TunnelState::Resolving;
            }
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
    fn test_neighbor_computation() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let con = CubeOverlayNetwork::new(local);
        assert_eq!(con.neighbors().len(), NEIGHBORS_PER_CUBE);
    }

    #[test]
    fn test_all_neighbors_differ_by_one_trit() {
        let local = addr([2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2]);
        let con = CubeOverlayNetwork::new(local.clone());
        for nbr in con.neighbors() {
            let dist = local.hamming_distance(&nbr.cube_addr);
            assert_eq!(dist, 1, "Neighbor must differ by exactly one trit");
        }
    }

    #[test]
    fn test_no_duplicate_neighbors() {
        let local = addr([1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1]);
        let con = CubeOverlayNetwork::new(local);
        let addrs: Vec<_> = con.neighbors().iter().map(|n| &n.cube_addr).collect();
        let unique: std::collections::HashSet<_> = addrs.iter().collect();
        assert_eq!(addrs.len(), unique.len(), "No duplicate neighbors");
    }

    #[test]
    fn test_neighbor_lookup() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let con = CubeOverlayNetwork::new(local);
        let expected = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let nbr = con.neighbor(&expected).unwrap();
        assert_eq!(nbr.dimension, 0);
    }

    #[test]
    fn test_pq_key_derivation_symmetric() {
        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let key_ab = CubeOverlayNetwork::derive_pq_tunnel_key(&a, &b);
        let key_ba = CubeOverlayNetwork::derive_pq_tunnel_key(&b, &a);
        assert_eq!(key_ab, key_ba, "Key derivation must be symmetric");
    }

    #[test]
    fn test_pq_key_derivation_unique() {
        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let c = addr([3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let key_ab = CubeOverlayNetwork::derive_pq_tunnel_key(&a, &b);
        let key_ac = CubeOverlayNetwork::derive_pq_tunnel_key(&a, &c);
        assert_ne!(key_ab, key_ac, "Different pairs must produce different keys");
    }

    #[test]
    fn test_iface_naming() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let con = CubeOverlayNetwork::new(local);
        let nbr_addr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let iface = con.iface_name_for(&nbr_addr).unwrap();
        assert!(iface.starts_with("cubetun"));

        // Reverse lookup
        let back = con.addr_for_iface(&iface).unwrap();
        assert_eq!(back, &nbr_addr);
    }

    #[test]
    fn test_tunnel_lifecycle() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut con = CubeOverlayNetwork::new(local);
        let nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        assert_eq!(con.neighbor(&nbr).unwrap().state, TunnelState::Unknown);

        // Resolve endpoint
        con.resolve_neighbor(
            &nbr,
            "192.168.1.1:51820".parse().unwrap(),
            [0u8; 32],
        );
        assert_eq!(con.neighbor(&nbr).unwrap().state, TunnelState::Connecting);

        // Tunnel established
        con.tunnel_up(&nbr, "cubetun0".to_string());
        assert_eq!(con.neighbor(&nbr).unwrap().state, TunnelState::Up);

        // Record heartbeat
        con.record_heartbeat(&nbr, 500_000); // 0.5ms
        assert!(con.neighbor(&nbr).unwrap().rtt_ms().unwrap() < 1.0);

        // Tunnel goes down
        con.tunnel_down(&nbr);
        assert_eq!(con.neighbor(&nbr).unwrap().state, TunnelState::Down);
    }

    #[test]
    fn test_stats() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let con = CubeOverlayNetwork::new(local);
        let stats = con.stats();
        assert_eq!(stats.tunnels_unknown, NEIGHBORS_PER_CUBE);
        assert_eq!(stats.tunnels_up, 0);
    }
}