// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # PlenumNET Inter-Cube Infrastructure Services
//!
//! Four lightweight software services that handle connections between cubes
//! in the 13-dimensional Metatronic Cube network. All four are computed
//! directly from the geometry — no routing tables, no discovery protocols,
//! no stored topology.
//!
//! ## Services
//!
//! | Service | Module | Purpose |
//! |---------|--------|---------|
//! | **GLB** | [`glb`] | Geometric Load Balancer — spreads traffic across d! shortest paths |
//! | **CON** | [`con`] | Cube Overlay Network — encrypted tunnels to 26 geometric neighbors |
//! | **CRS** | [`crs`] | Cube Registration Service — address allocation + endpoint registry |
//! | **FTS** | [`fts`] | Fault Tolerance Service — heartbeat monitoring + dead neighbor set |
//!
//! ## Wire Protocol
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`wire`] | Versioned binary wire format for ALL inter-cube messages |
//!
//! The wire module (added by T-01, SPEC-2026-NEXT) provides a universal
//! 24-byte header with `protocol_version: u8` for safe rollout of format
//! changes. Every inter-cube message — heartbeats, CRS queries, tunnel
//! messages — carries this header.
//!
//! ## HTTP API
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`api`] | Axum route handlers, request/response types, router builders |
//!
//! The api module (added by T-02, SPEC-2026-NEXT) extracts all HTTP
//! handlers from `main.rs` into a reusable module. Provides `crs_router()`
//! (11 routes) and `cube_router()` (8 routes) for the two operating modes.
//!
//! ## Dependency Chain
//!
//! ```text
//!                     ┌─────┐
//!                     │ CRS │  ← Coordination (address + endpoint registry)
//!                     └──┬──┘
//!                        │ neighbor endpoints
//!                        ▼
//!                     ┌─────┐
//!                     │ CON │  ← Connectivity (encrypted tunnels)
//!                     └──┬──┘
//!                        │ tunnel health
//!                        ▼
//!                     ┌─────┐
//!                     │ FTS │  ← Monitoring (dead neighbor set)
//!                     └──┬──┘
//!                        │ dead set
//!                        ▼
//!                     ┌─────┐
//!                     │ GLB │  ← Forwarding (pure math + flow affinity)
//!                     └─────┘
//! ```
//!
//! ## Startup Sequence
//!
//! 1. **CRS** → Register, get address + neighbor endpoints
//! 2. **CON** → Build tunnels to all 26 neighbors
//! 3. **FTS** → Begin heartbeats on all tunnels
//! 4. **GLB** → Ready to forward. Dead set is empty. All 26 paths available.
//!
//! ## Geometric Foundation
//!
//! All four services operate on Rep C (bijective ternary {1, 2, 3}) addresses.
//! The 13-trit address maps to a vertex in the 3¹³ = 1,594,323 node cube.
//! Zero never appears — its presence is instant proof of forgery.
//!
//! The routing algorithm is a single principle: given source S and destination D,
//! compute delta (dimensions where S ≠ D), select a dimension to fix, and step
//! one trit toward the destination. The number of hops equals the Hamming distance.
//! No table, no lookup, no stored state.
//!
//! ## Integration with Existing Stack
//!
//! - Uses `ternary_math::gf3::{Gf3, Gf3Vec}` for GF(3) primitives
//! - Rep C ↔ Rep B bijection: `f(c) = c - 1` / `f(b) = b + 1`
//! - Interoperates with `ternary_math::torus::TorusAddress` for torus topology
//! - TIS-27 / TL-Sponge-385 for key derivation (CON) — no binary hash primitives
//! - HPTP timestamps for precise RTT measurement (FTS)
//! - Wire protocol carries femtosecond timestamps on all messages

pub mod cube_addr;
pub mod glb;
pub mod overlay;
pub mod crs;
pub mod fts;
pub mod wire;
pub mod api;
pub mod rate_limit;
pub mod identity;
pub mod persistence;
pub mod tunnel_auth;
pub mod address_keys;
pub mod placement;
pub mod wire_ecc;
pub mod key_rotation;
pub mod verify_cache;
pub mod deregistration;
pub mod dimension_tracker;
pub mod lattice_mixer;
pub mod sampling;
pub mod telemetry;

// Re-export the most commonly used types
pub use cube_addr::{CubeAddr, MultiLevelAddr, RepCTrit, DIMENSIONS, TOTAL_VERTICES, NEIGHBORS_PER_CUBE};
pub use glb::{GeometricLoadBalancer, ForwardResult, ForwardError, GlbStats};
pub use overlay::{CubeOverlayNetwork, Neighbor, TunnelState, TunnelProtocol, ConStats};
pub use crs::{CubeRegistrationService, CubeRecord, CubeStatus, RegistrationResult, RegistrationError, NeighborInfo};
pub use fts::{FaultToleranceService, NeighborHealth, NeighborState, StateChangeEvent, FtsConfig};
pub use rate_limit::{CrsGuard, GuardError};
pub use identity::{MasterSecret, SecretRotation, IdentityError};
pub use persistence::{SequenceStore, PersistenceError};
pub use tunnel_auth::{HandshakeManager, HandshakeSession, TunnelAuthError};
pub use address_keys::{AddressKeyManager, IdentityKeypair};
pub use placement::{allocate_optimal, DimensionDensity, PlacementMetrics};
pub use wire_ecc::{EccSyndrome, EccResult};
pub use key_rotation::RotationOrchestrator;
pub use verify_cache::{VerificationCache, CrsCacheManager};
pub use deregistration::{SignedDeregistration, DeregReason, DeregError, verify_deregistration};
pub use dimension_tracker::{DimensionTracker, DensityMetrics};
pub use lattice_mixer::{compute_pair_nonce, derive_lattice_mixed_key};
pub use sampling::{AddressSnapshot, SamplingInfo};
pub use telemetry::{MetricsRegistry, MetricsSnapshot};
pub use wire::{
    WireHeader, WireMessage, WireError, WireFlags, MessageType,
    WIRE_HEADER_SIZE, WIRE_ADDR_SIZE,
    PROTOCOL_VERSION_CURRENT, PROTOCOL_VERSION_V1, PROTOCOL_VERSION_V2,
    pack_addr, unpack_addr, pack_trit_array, unpack_trit_array,
    negotiate_version, timestamp_in_window,
    FS_PER_SECOND, FS_PER_MILLISECOND,
    REGISTRATION_MAX_AGE_FS, TIMESTAMP_FUTURE_TOLERANCE_FS,
};

/// Library version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The framework identifier.
pub const FRAMEWORK: &str = "PlenumNET Inter-Cube Infrastructure";

// ═══════════════════════════════════════════════════════════════════════
// INTEGRATION ORCHESTRATOR — Full startup sequence
// ═══════════════════════════════════════════════════════════════════════

/// Complete inter-cube service stack for a single cube.
///
/// Orchestrates the four services through the startup sequence:
/// CRS → CON → FTS → GLB.
pub struct InterCubeStack {
    /// The cube's Rep C address (assigned by CRS).
    pub addr: CubeAddr,
    /// Geometric Load Balancer.
    pub glb: GeometricLoadBalancer,
    /// Cube Overlay Network.
    pub con: CubeOverlayNetwork,
    /// Fault Tolerance Service.
    pub fts: FaultToleranceService,
}

impl InterCubeStack {
    /// Initialize the full stack with an assigned address.
    ///
    /// This is called after CRS registration returns an address.
    /// The sequence:
    /// 1. CON computes 26 neighbors and prepares tunnel state
    /// 2. FTS initializes health monitoring for all 26
    /// 3. GLB is ready to forward (empty dead set)
    pub fn new(addr: CubeAddr) -> Self {
        let glb = GeometricLoadBalancer::new(addr.clone());
        let con = CubeOverlayNetwork::new(addr.clone());
        let fts = FaultToleranceService::new(addr.clone());

        InterCubeStack { addr, glb, con, fts }
    }

    /// Synchronize FTS dead set → GLB dead set.
    /// Call this periodically or after FTS processes heartbeats.
    pub fn sync_dead_set(&mut self) {
        self.glb.set_dead_neighbors(self.fts.dead_set_cloned());
    }

    /// Process FTS events and update CON tunnel states accordingly.
    pub fn process_fts_events(&mut self) {
        let events = self.fts.drain_events();
        for event in &events {
            match event.to {
                NeighborState::Down => {
                    self.con.tunnel_down(&event.addr);
                }
                NeighborState::Up => {
                    // Tunnel recovery — CON should re-establish
                    // (In production, this triggers async tunnel setup)
                }
                _ => {}
            }
        }
        // Sync dead set after processing
        self.sync_dead_set();
    }

    /// Get the number of live tunnels.
    pub fn live_tunnels(&self) -> usize {
        self.con.stats().tunnels_up
    }

    /// Get the number of dead neighbors.
    pub fn dead_count(&self) -> usize {
        self.fts.dead_set().len()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// INTEGRATION TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn addr(trits: [u8; 13]) -> CubeAddr {
        CubeAddr::new(trits)
    }

    #[test]
    fn test_full_bootstrap_sequence() {
        // Step 1: CRS — register a new cube
        let mut crs = CubeRegistrationService::new();
        let result = crs
            .register(
                "10.0.0.1:51820".parse().unwrap(),
                vec![0xAB; 32],
                Some(addr([2, 1, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1])),
            )
            .unwrap();

        // Step 2: Initialize the stack with the assigned address
        let mut stack = InterCubeStack::new(result.address.clone());

        // Step 3: CON resolves neighbor endpoints from CRS
        for nbr_info in &result.neighbors {
            if let Some(ep) = nbr_info.endpoint {
                let pk = nbr_info.public_key.clone().unwrap_or_else(|| vec![0u8; 32]);
                stack.con.resolve_neighbor(&nbr_info.addr, ep, pk);
            }
        }

        // Step 4: FTS begins monitoring (all neighbors start as Up)
        let (up, _, _, _) = stack.fts.state_counts();
        assert_eq!(up, NEIGHBORS_PER_CUBE);

        // Step 5: GLB ready to forward
        let dest = addr([3, 1, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1]);
        let forward = stack.glb.forward_stateless(&dest, 42);
        assert!(forward.is_ok(), "GLB should be ready to forward");
    }

    #[test]
    fn test_failure_and_recovery_flow() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut stack = InterCubeStack::new(local);
        let nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        // Simulate failure
        let config = FtsConfig {
            grace_period: std::time::Duration::from_millis(0),
            recovery_threshold: 2,
            ..Default::default()
        };
        stack.fts = FaultToleranceService::new(stack.addr.clone()).with_config(config);

        // Drive neighbor to Down
        for _ in 0..4 {
            stack.fts.record_miss(&nbr);
        }
        stack.process_fts_events();

        assert!(stack.fts.dead_set().contains(&nbr));
        assert!(stack.glb.dead_neighbors().contains(&nbr));

        // Simulate recovery
        stack.fts.record_pong(&nbr, 500_000);
        stack.fts.record_pong(&nbr, 500_000);
        stack.process_fts_events();

        assert!(!stack.fts.dead_set().contains(&nbr));
        assert!(!stack.glb.dead_neighbors().contains(&nbr));
    }

    #[test]
    fn test_multi_cube_registration() {
        let mut crs = CubeRegistrationService::new();

        // Register 5 cubes
        let addrs: Vec<CubeAddr> = (0..5)
            .map(|_| {
                crs.register(
                    "10.0.0.1:51820".parse().unwrap(),
                    vec![0xAB; 32],
                    None,
                )
                .unwrap()
                .address
            })
            .collect();

        assert_eq!(crs.registered_count(), 5);

        // Each cube should be able to find registered neighbors
        for a in &addrs {
            let nbrs = crs.compute_neighbor_info(a);
            assert_eq!(nbrs.len(), NEIGHBORS_PER_CUBE);
        }
    }

    #[test]
    fn test_con_key_derivation_all_unique() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let con = CubeOverlayNetwork::new(local.clone());

        // T-09: v2.5 fallback removed — must provide KEM secrets for v3 keys.
        // Generate unique KEM secrets for all 26 neighbors.
        let mut kem_secrets = std::collections::HashMap::new();
        for (i, nbr) in con.neighbors().iter().enumerate() {
            let mut secret = [0u8; 32];
            secret[0] = (i + 1) as u8;
            secret[1] = ((i + 1) >> 8) as u8;
            kem_secrets.insert(nbr.cube_addr.clone(), secret);
        }

        let keys = con.derive_all_keys(&kem_secrets, 0);
        assert_eq!(keys.len(), NEIGHBORS_PER_CUBE);

        // All keys must be unique
        let key_set: std::collections::HashSet<[u8; 32]> =
            keys.iter().map(|(_, k)| *k).collect();
        assert_eq!(key_set.len(), NEIGHBORS_PER_CUBE, "All v3 tunnel keys must be unique");
    }

    #[test]
    fn test_con_no_kem_no_keys() {
        // T-09: No KEM secrets → 0 keys (v2.5 fallback removed)
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let con = CubeOverlayNetwork::new(local);
        let keys = con.derive_all_keys(&std::collections::HashMap::new(), 0);
        assert_eq!(keys.len(), 0, "No KEM secrets → no keys (T-09)");
    }

    // ── Wire Protocol Integration Tests ────────────────────────

    #[test]
    fn test_wire_message_for_heartbeat() {
        let ts: u128 = 42 * FS_PER_SECOND;
        let msg = WireMessage::new(MessageType::HeartbeatPing, ts, vec![]);
        assert!(msg.validate().is_ok());
        assert_eq!(msg.header.version, PROTOCOL_VERSION_CURRENT);
    }

    #[test]
    fn test_wire_version_negotiation_v1_v2() {
        // Local supports v1-v2, remote only v1
        let negotiated = negotiate_version(
            PROTOCOL_VERSION_V1,
            PROTOCOL_VERSION_V2,
            PROTOCOL_VERSION_V1,
            PROTOCOL_VERSION_V1,
        );
        assert_eq!(negotiated, Some(PROTOCOL_VERSION_V1));
    }
}