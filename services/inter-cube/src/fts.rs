// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Fault Tolerance Service (FTS) — Service 4
//!
//! Detects failures of inter-cube links or entire cubes and provides the
//! routing layer (GLB) with updated availability information so it can
//! compute alternative paths — still using pure math, just excluding
//! dead neighbors.
//!
//! ## Design Principle
//!
//! The FTS doesn't compute routes. It doesn't maintain path tables. It simply
//! monitors neighbor liveness and publishes a **dead neighbor set**. The GLB
//! excludes dead neighbors from its path computation.
//!
//! Because the 13D cube has high connectivity (26 neighbors per node),
//! alternative paths of equal or near-equal length are mathematically
//! guaranteed to exist.
//!
//! ## Failure Detection
//!
//! Threshold-based: if N consecutive pings are missed, mark as Suspect.
//! After a grace period, promote to Down. Recovery requires M consecutive
//! successful pings before returning to Up.
//!
//! ## Locality
//!
//! Each cube monitors ONLY its 26 direct neighbors — not the global network.
//! Failures are handled locally. No flooding, no global state.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::cube_addr::{CubeAddr, RepCTrit, DIMENSIONS, NEIGHBORS_PER_CUBE};

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Default ping interval: 1 second.
const DEFAULT_PING_INTERVAL_MS: u64 = 1000;

/// Default miss threshold: 3 consecutive missed pings → Suspect.
const DEFAULT_MISS_THRESHOLD: u8 = 3;

/// Default recovery threshold: 5 consecutive successes → Up.
const DEFAULT_RECOVERY_THRESHOLD: u8 = 5;

/// Default grace period before Suspect → Down: 5 seconds.
const DEFAULT_GRACE_PERIOD_MS: u64 = 5000;

// ═══════════════════════════════════════════════════════════════════════
// NEIGHBOR STATE MACHINE
// ═══════════════════════════════════════════════════════════════════════

/// Health state of a neighbor cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeighborState {
    /// Healthy — receiving pong responses.
    Up,
    /// Missed threshold consecutive pings — may recover.
    Suspect,
    /// Confirmed dead — reported to GLB's dead set.
    Down,
    /// Was down, now responding — under probation.
    Recovering,
}

impl NeighborState {
    /// Whether this neighbor is considered available for routing.
    pub fn is_available(&self) -> bool {
        matches!(self, NeighborState::Up)
    }

    /// Whether this neighbor should be in the dead set.
    pub fn is_dead(&self) -> bool {
        matches!(self, NeighborState::Down | NeighborState::Suspect)
    }
}

impl std::fmt::Display for NeighborState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NeighborState::Up => write!(f, "up"),
            NeighborState::Suspect => write!(f, "suspect"),
            NeighborState::Down => write!(f, "down"),
            NeighborState::Recovering => write!(f, "recovering"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// NEIGHBOR HEALTH RECORD
// ═══════════════════════════════════════════════════════════════════════

/// Complete health record for a single geometric neighbor.
#[derive(Debug, Clone)]
pub struct NeighborHealth {
    /// The neighbor's Rep C cube address.
    pub addr: CubeAddr,
    /// Which dimension this neighbor differs from us in.
    pub dimension: usize,
    /// What value the neighbor holds in the differing dimension.
    pub alt_value: RepCTrit,
    /// Current health state.
    pub state: NeighborState,
    /// Smoothed round-trip time in nanoseconds.
    pub srtt_ns: u64,
    /// Jitter (SRTT variance) in nanoseconds.
    pub jitter_ns: u64,
    /// Consecutive missed pings.
    pub consecutive_misses: u8,
    /// Consecutive successful pings (used during recovery).
    pub consecutive_successes: u8,
    /// Last successful pong timestamp.
    pub last_pong: Option<Instant>,
    /// When this neighbor entered Suspect state (for grace period).
    pub suspect_since: Option<Instant>,
}

impl NeighborHealth {
    /// Create a new health record from computed geometry.
    fn new(addr: CubeAddr, dimension: usize, alt_value: RepCTrit) -> Self {
        NeighborHealth {
            addr,
            dimension,
            alt_value,
            state: NeighborState::Up, // Assume up until proven otherwise
            srtt_ns: 0,
            jitter_ns: 0,
            consecutive_misses: 0,
            consecutive_successes: 0,
            last_pong: None,
            suspect_since: None,
        }
    }

    /// Time since last successful pong.
    pub fn time_since_pong(&self) -> Option<Duration> {
        self.last_pong.map(|t| t.elapsed())
    }

    /// SRTT in milliseconds.
    pub fn srtt_ms(&self) -> f64 {
        self.srtt_ns as f64 / 1_000_000.0
    }

    /// Jitter in milliseconds.
    pub fn jitter_ms(&self) -> f64 {
        self.jitter_ns as f64 / 1_000_000.0
    }
}

// ═══════════════════════════════════════════════════════════════════════
// STATE CHANGE EVENT — For notifying GLB and CON
// ═══════════════════════════════════════════════════════════════════════

/// Event emitted when a neighbor's state changes.
#[derive(Debug, Clone)]
pub struct StateChangeEvent {
    /// The neighbor whose state changed.
    pub addr: CubeAddr,
    /// Previous state.
    pub from: NeighborState,
    /// New state.
    pub to: NeighborState,
    /// When the change occurred.
    pub timestamp: Instant,
}

// ═══════════════════════════════════════════════════════════════════════
// FTS CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════

/// FTS tuning parameters.
#[derive(Debug, Clone)]
pub struct FtsConfig {
    /// Ping interval.
    pub ping_interval: Duration,
    /// Consecutive misses before Suspect.
    pub miss_threshold: u8,
    /// Consecutive successes before recovery.
    pub recovery_threshold: u8,
    /// Grace period before Suspect → Down.
    pub grace_period: Duration,
}

impl Default for FtsConfig {
    fn default() -> Self {
        FtsConfig {
            ping_interval: Duration::from_millis(DEFAULT_PING_INTERVAL_MS),
            miss_threshold: DEFAULT_MISS_THRESHOLD,
            recovery_threshold: DEFAULT_RECOVERY_THRESHOLD,
            grace_period: Duration::from_millis(DEFAULT_GRACE_PERIOD_MS),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// FAULT TOLERANCE SERVICE
// ═══════════════════════════════════════════════════════════════════════

/// The Fault Tolerance Service daemon.
///
/// Monitors exactly 26 neighbors via heartbeat ping/pong.
/// Publishes a dead neighbor set consumed by the GLB for routing.
/// Each cube runs its own FTS — no global state, no flooding.
pub struct FaultToleranceService {
    /// This cube's Rep C address.
    local_addr: CubeAddr,
    /// Health records for all 26 neighbors.
    neighbors: Vec<NeighborHealth>,
    /// The dead set — published to GLB.
    dead_set: HashSet<CubeAddr>,
    /// Configuration.
    config: FtsConfig,
    /// Pending state change events (consumed by GLB/CON notification).
    pending_events: Vec<StateChangeEvent>,
}

impl FaultToleranceService {
    /// Create a new FTS daemon for the given local cube address.
    ///
    /// Computes all 26 neighbors from geometry — identical computation
    /// to CON's neighbor list.
    pub fn new(local_addr: CubeAddr) -> Self {
        let mut neighbors = Vec::with_capacity(NEIGHBORS_PER_CUBE);

        for dim in 0..DIMENSIONS {
            for alt in local_addr.trit(dim).alternatives() {
                let mut nbr_addr = local_addr.clone();
                nbr_addr.set_trit(dim, alt);
                neighbors.push(NeighborHealth::new(nbr_addr, dim, alt));
            }
        }

        FaultToleranceService {
            local_addr,
            neighbors,
            dead_set: HashSet::new(),
            config: FtsConfig::default(),
            pending_events: Vec::new(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(mut self, config: FtsConfig) -> Self {
        self.config = config;
        self
    }

    /// Get the local cube address.
    pub fn local_addr(&self) -> &CubeAddr {
        &self.local_addr
    }

    // ═══════════════════════════════════════════════════════════════
    // HEARTBEAT PROCESSING — The core detection loop
    // ═══════════════════════════════════════════════════════════════

    /// Record a successful pong response from a neighbor.
    /// Updates SRTT, jitter, and state machine.
    pub fn record_pong(&mut self, addr: &CubeAddr, rtt_ns: u64) {
        let config = self.config.clone();
        let now = Instant::now();

        if let Some(nbr) = self.neighbors.iter_mut().find(|n| n.addr == *addr) {
            let old_state = nbr.state;

            // Update RTT metrics
            if nbr.srtt_ns == 0 {
                nbr.srtt_ns = rtt_ns;
                nbr.jitter_ns = rtt_ns / 2;
            } else {
                // RFC 6298 SRTT computation
                let diff = if rtt_ns > nbr.srtt_ns {
                    rtt_ns - nbr.srtt_ns
                } else {
                    nbr.srtt_ns - rtt_ns
                };
                nbr.jitter_ns = (nbr.jitter_ns * 3 + diff) / 4;
                nbr.srtt_ns = (nbr.srtt_ns * 7 + rtt_ns) / 8;
            }

            nbr.last_pong = Some(now);
            nbr.consecutive_misses = 0;
            nbr.consecutive_successes += 1;

            // State transitions on pong
            match nbr.state {
                NeighborState::Up => {
                    // Already up — nothing to do
                }
                NeighborState::Suspect => {
                    // Pong received while suspect — go to Recovering
                    nbr.state = NeighborState::Recovering;
                    nbr.suspect_since = None;
                    nbr.consecutive_successes = 1;
                }
                NeighborState::Down => {
                    // Was dead, now responding — begin recovery
                    nbr.state = NeighborState::Recovering;
                    nbr.consecutive_successes = 1;
                }
                NeighborState::Recovering => {
                    // Check if recovery threshold met
                    if nbr.consecutive_successes >= config.recovery_threshold {
                        nbr.state = NeighborState::Up;
                    }
                }
            }

            // Emit event if state changed
            if nbr.state != old_state {
                self.pending_events.push(StateChangeEvent {
                    addr: addr.clone(),
                    from: old_state,
                    to: nbr.state,
                    timestamp: now,
                });
            }

            // Update dead set
            self.rebuild_dead_set();
        }
    }

    /// Record a missed ping (no pong response within timeout).
    pub fn record_miss(&mut self, addr: &CubeAddr) {
        let config = self.config.clone();
        let now = Instant::now();

        if let Some(nbr) = self.neighbors.iter_mut().find(|n| n.addr == *addr) {
            let old_state = nbr.state;

            nbr.consecutive_misses += 1;
            nbr.consecutive_successes = 0;

            // State transitions on miss
            match nbr.state {
                NeighborState::Up => {
                    if nbr.consecutive_misses >= config.miss_threshold {
                        nbr.state = NeighborState::Suspect;
                        nbr.suspect_since = Some(now);
                    }
                }
                NeighborState::Suspect => {
                    // Check if grace period expired → promote to Down
                    if let Some(since) = nbr.suspect_since {
                        if now.duration_since(since) >= config.grace_period {
                            nbr.state = NeighborState::Down;
                            nbr.suspect_since = None;
                        }
                    }
                }
                NeighborState::Down => {
                    // Already down — nothing to do
                }
                NeighborState::Recovering => {
                    // Miss during recovery — back to Suspect
                    nbr.state = NeighborState::Suspect;
                    nbr.suspect_since = Some(now);
                }
            }

            // Emit event if state changed
            if nbr.state != old_state {
                self.pending_events.push(StateChangeEvent {
                    addr: addr.clone(),
                    from: old_state,
                    to: nbr.state,
                    timestamp: now,
                });
            }

            // Update dead set
            self.rebuild_dead_set();
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // DEAD SET — Published to GLB (Service 1)
    // ═══════════════════════════════════════════════════════════════

    /// Rebuild the dead set from neighbor health states.
    fn rebuild_dead_set(&mut self) {
        self.dead_set.clear();
        for nbr in &self.neighbors {
            if nbr.state.is_dead() {
                self.dead_set.insert(nbr.addr.clone());
            }
        }
    }

    /// Get the current dead neighbor set.
    /// This is what the GLB reads for routing decisions.
    pub fn dead_set(&self) -> &HashSet<CubeAddr> {
        &self.dead_set
    }

    /// Get a cloned dead set (for passing to GLB).
    pub fn dead_set_cloned(&self) -> HashSet<CubeAddr> {
        self.dead_set.clone()
    }

    // ═══════════════════════════════════════════════════════════════
    // EVENT DRAIN — For notifying GLB and CON
    // ═══════════════════════════════════════════════════════════════

    /// Drain pending state change events.
    /// Call this after processing to clear the event queue.
    pub fn drain_events(&mut self) -> Vec<StateChangeEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Check if there are pending events.
    pub fn has_pending_events(&self) -> bool {
        !self.pending_events.is_empty()
    }

    // ═══════════════════════════════════════════════════════════════
    // QUERY
    // ═══════════════════════════════════════════════════════════════

    /// Get the health status of all 26 neighbors.
    pub fn all_status(&self) -> &[NeighborHealth] {
        &self.neighbors
    }

    /// Get a specific neighbor's health.
    pub fn neighbor_health(&self, addr: &CubeAddr) -> Option<&NeighborHealth> {
        self.neighbors.iter().find(|n| n.addr == *addr)
    }

    /// Count neighbors by state.
    pub fn state_counts(&self) -> (usize, usize, usize, usize) {
        let mut up = 0;
        let mut suspect = 0;
        let mut down = 0;
        let mut recovering = 0;
        for n in &self.neighbors {
            match n.state {
                NeighborState::Up => up += 1,
                NeighborState::Suspect => suspect += 1,
                NeighborState::Down => down += 1,
                NeighborState::Recovering => recovering += 1,
            }
        }
        (up, suspect, down, recovering)
    }

    /// Get the configuration.
    pub fn config(&self) -> &FtsConfig {
        &self.config
    }

    /// Update configuration at runtime.
    pub fn update_config(&mut self, config: FtsConfig) {
        self.config = config;
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
    fn test_initial_state() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let fts = FaultToleranceService::new(local);

        assert_eq!(fts.neighbors.len(), NEIGHBORS_PER_CUBE);
        assert!(fts.dead_set().is_empty(), "Initially no dead neighbors");

        let (up, suspect, down, recovering) = fts.state_counts();
        assert_eq!(up, NEIGHBORS_PER_CUBE);
        assert_eq!(suspect, 0);
        assert_eq!(down, 0);
        assert_eq!(recovering, 0);
    }

    #[test]
    fn test_pong_updates_srtt() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut fts = FaultToleranceService::new(local);
        let nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        fts.record_pong(&nbr, 1_000_000); // 1ms
        let health = fts.neighbor_health(&nbr).unwrap();
        assert_eq!(health.srtt_ns, 1_000_000);
        assert!(health.last_pong.is_some());
    }

    #[test]
    fn test_miss_threshold_to_suspect() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut fts = FaultToleranceService::new(local);
        let nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        // Miss 3 times (default threshold)
        fts.record_miss(&nbr);
        assert_eq!(
            fts.neighbor_health(&nbr).unwrap().state,
            NeighborState::Up
        );
        fts.record_miss(&nbr);
        assert_eq!(
            fts.neighbor_health(&nbr).unwrap().state,
            NeighborState::Up
        );
        fts.record_miss(&nbr);
        assert_eq!(
            fts.neighbor_health(&nbr).unwrap().state,
            NeighborState::Suspect,
            "Should be Suspect after 3 misses"
        );

        // Should be in dead set now
        assert!(fts.dead_set().contains(&nbr));
    }

    #[test]
    fn test_suspect_to_down_after_grace() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let config = FtsConfig {
            grace_period: Duration::from_millis(0), // Instant grace period for test
            ..Default::default()
        };
        let mut fts = FaultToleranceService::new(local).with_config(config);
        let nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        // Reach Suspect
        for _ in 0..3 {
            fts.record_miss(&nbr);
        }
        assert_eq!(
            fts.neighbor_health(&nbr).unwrap().state,
            NeighborState::Suspect
        );

        // One more miss with expired grace → Down
        fts.record_miss(&nbr);
        assert_eq!(
            fts.neighbor_health(&nbr).unwrap().state,
            NeighborState::Down
        );
    }

    #[test]
    fn test_recovery_from_down() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let config = FtsConfig {
            grace_period: Duration::from_millis(0),
            recovery_threshold: 3, // Faster recovery for test
            ..Default::default()
        };
        let mut fts = FaultToleranceService::new(local).with_config(config);
        let nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        // Drive to Down
        for _ in 0..4 {
            fts.record_miss(&nbr);
        }
        assert_eq!(
            fts.neighbor_health(&nbr).unwrap().state,
            NeighborState::Down
        );

        // Begin recovery
        fts.record_pong(&nbr, 500_000);
        assert_eq!(
            fts.neighbor_health(&nbr).unwrap().state,
            NeighborState::Recovering
        );

        // Complete recovery
        fts.record_pong(&nbr, 500_000);
        fts.record_pong(&nbr, 500_000);
        assert_eq!(
            fts.neighbor_health(&nbr).unwrap().state,
            NeighborState::Up,
            "Should be Up after recovery threshold"
        );

        // Should be removed from dead set
        assert!(!fts.dead_set().contains(&nbr));
    }

    #[test]
    fn test_miss_during_recovery() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let config = FtsConfig {
            grace_period: Duration::from_millis(0),
            ..Default::default()
        };
        let mut fts = FaultToleranceService::new(local).with_config(config);
        let nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        // Drive to Down
        for _ in 0..4 {
            fts.record_miss(&nbr);
        }

        // Begin recovery
        fts.record_pong(&nbr, 500_000);
        assert_eq!(
            fts.neighbor_health(&nbr).unwrap().state,
            NeighborState::Recovering
        );

        // Miss during recovery → back to Suspect
        fts.record_miss(&nbr);
        assert_eq!(
            fts.neighbor_health(&nbr).unwrap().state,
            NeighborState::Suspect
        );
    }

    #[test]
    fn test_events_emitted() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut fts = FaultToleranceService::new(local);
        let nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        assert!(!fts.has_pending_events());

        // Drive to Suspect
        for _ in 0..3 {
            fts.record_miss(&nbr);
        }

        assert!(fts.has_pending_events());
        let events = fts.drain_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].from, NeighborState::Up);
        assert_eq!(events[0].to, NeighborState::Suspect);
    }

    #[test]
    fn test_integration_fts_to_glb() {
        use crate::glb::GeometricLoadBalancer;

        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let dest = addr([2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        let config = FtsConfig {
            grace_period: Duration::from_millis(0),
            ..Default::default()
        };
        let mut fts = FaultToleranceService::new(local.clone()).with_config(config);
        let mut glb = GeometricLoadBalancer::new(local.clone());

        // Kill dim 0 neighbor
        let dead_nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        for _ in 0..4 {
            fts.record_miss(&dead_nbr);
        }

        // Push dead set to GLB
        glb.set_dead_neighbors(fts.dead_set_cloned());

        // GLB should avoid the dead neighbor
        let result = glb.forward_stateless(&dest, 42).unwrap();
        assert_ne!(
            result.next_hop,
            dead_nbr,
            "GLB must avoid dead neighbor"
        );
        assert_eq!(result.dimension_fixed, 1, "Should route via dim 1");
    }
}