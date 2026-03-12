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
//! ## T-08 (SPEC-2026-NEXT): Authenticated Heartbeats
//!
//! Two authentication modes, negotiated at registration time:
//!
//! - **HMAC mode** (default): One TIS-27 sponge operation per heartbeat,
//!   sub-microsecond. HMAC key derived via `TLSponge-385("PlenumNET-HB-HMAC"
//!   ‖ address ‖ master_secret)` — 48 bytes, independently derived by both
//!   CRS and node, never transmitted.
//!
//! - **Full-Sig mode** (optional, configurable per node): TL-DSA-87 signature
//!   per heartbeat for non-repudiable trails. Higher cost (~5ms vs ~1µs).
//!
//! ### Replay Protection
//!
//! Monotonic `sequence: u64` counter prevents replay. FTS rejects any
//! heartbeat with `sequence <= last_accepted`. Sliding window (±10) to
//! tolerate minor out-of-order delivery.
//!
//! ### Auth Failure Escalation
//!
//! After 3 consecutive authentication failures, FTS emits a suspect event
//! for the neighbor — same as 3 consecutive missed pings. This prevents
//! an attacker from keeping a hijacked registration alive with forged
//! heartbeats.

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

/// Authentication failure threshold: 3 consecutive auth failures → Suspect (T-08).
const DEFAULT_AUTH_FAILURE_THRESHOLD: u8 = 3;

/// Sequence number sliding window for out-of-order tolerance (T-08).
/// Accept sequences in `[last_accepted - WINDOW, last_accepted + ∞)`.
/// This allows minor reordering without rejecting valid heartbeats.
const SEQUENCE_WINDOW: u64 = 10;

/// Domain separator for heartbeat HMAC key derivation (T-08).
pub const HB_HMAC_DOMAIN: &[u8] = b"PlenumNET-HB-HMAC";

/// Domain separator for heartbeat HMAC computation (T-08).
pub const HB_HMAC_TAG_DOMAIN: &[u8] = b"PlenumNET-HB-TAG";

/// HMAC key length in bytes (derived from TLSponge-385).
pub const HB_HMAC_KEY_LEN: usize = 48;

/// HMAC tag length in bytes (squeezed from TIS-27).
pub const HB_HMAC_TAG_LEN: usize = 27;

// ═══════════════════════════════════════════════════════════════════════
// HEARTBEAT AUTHENTICATION — T-08
// ═══════════════════════════════════════════════════════════════════════

/// Authentication mode for heartbeats (T-08).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HeartbeatAuth {
    /// TIS-27 HMAC — sub-microsecond, symmetric. Default mode.
    TisHmac = 0x01,
    /// TL-DSA-87 full signature — non-repudiable, higher cost.
    TlDsaSig = 0x02,
}

/// An authenticated heartbeat payload (T-08).
///
/// Both CRS and the sending node compute the HMAC tag independently.
/// The sequence counter prevents replay. The timestamp is for ordering.
#[derive(Debug, Clone)]
pub struct AuthenticatedHeartbeat {
    /// The sender's Rep C cube address.
    pub address: CubeAddr,
    /// Physical endpoint (may change for mobile cubes).
    pub endpoint: String,
    /// Monotonic sequence counter. Must be strictly increasing
    /// (with sliding window tolerance for minor reordering).
    pub sequence: u64,
    /// Femtosecond timestamp since Salvi Epoch (HPTP-synchronized).
    pub timestamp_fs: u128,
    /// Authentication mode.
    pub auth_mode: HeartbeatAuth,
    /// Authentication data: HMAC tag (27 bytes) or TL-DSA signature.
    pub auth_data: Vec<u8>,
}

impl AuthenticatedHeartbeat {
    /// Construct the canonical message for HMAC computation.
    ///
    /// Format: `address_bytes ‖ endpoint_bytes ‖ sequence_le ‖ timestamp_le`
    ///
    /// Both sender and receiver construct the same message.
    pub fn canonical_message(&self) -> Vec<u8> {
        let addr_bytes = self.address.to_bytes();
        let endpoint_bytes = self.endpoint.as_bytes();
        let seq_bytes = self.sequence.to_le_bytes();
        let ts_bytes = self.timestamp_fs.to_le_bytes();

        let mut msg = Vec::with_capacity(
            addr_bytes.len() + endpoint_bytes.len() + 8 + 16,
        );
        msg.extend_from_slice(&addr_bytes);
        msg.extend_from_slice(endpoint_bytes);
        msg.extend_from_slice(&seq_bytes);
        msg.extend_from_slice(&ts_bytes);
        msg
    }
}

/// Derive an HMAC key for heartbeat authentication (T-08).
///
/// `key = TLSponge-385("PlenumNET-HB-HMAC" ‖ address_bytes ‖ master_secret)`
///
/// Derived independently by both CRS and the registering node from the
/// shared `master_secret` established during KEM exchange. Never transmitted.
///
/// The key is 48 bytes (384 bits) — matching the TLSponge-385 security level.
pub fn derive_hb_hmac_key(address: &CubeAddr, master_secret: &[u8]) -> Vec<u8> {
    let addr_bytes = address.to_bytes();
    let mut material = Vec::with_capacity(addr_bytes.len() + master_secret.len());
    material.extend_from_slice(&addr_bytes);
    material.extend_from_slice(master_secret);

    ternary_math::sponge::derive_key(HB_HMAC_DOMAIN, &material, HB_HMAC_KEY_LEN)
}

/// Compute an HMAC tag for a heartbeat message (T-08).
///
/// `tag = TIS-27-Sponge(hmac_key ‖ message)` squeezed to 27 bytes.
///
/// Uses the existing `derive_key` function with the HMAC tag domain
/// separator to produce a keyed hash.
pub fn compute_hb_hmac(hmac_key: &[u8], message: &[u8]) -> Vec<u8> {
    let mut material = Vec::with_capacity(hmac_key.len() + message.len());
    material.extend_from_slice(hmac_key);
    material.extend_from_slice(message);

    ternary_math::sponge::derive_key(HB_HMAC_TAG_DOMAIN, &material, HB_HMAC_TAG_LEN)
}

/// Verify a heartbeat HMAC tag (T-08).
///
/// Recomputes the tag and compares in constant time.
pub fn verify_hb_hmac(hmac_key: &[u8], message: &[u8], received_tag: &[u8]) -> bool {
    let expected = compute_hb_hmac(hmac_key, message);
    if expected.len() != received_tag.len() {
        return false;
    }
    // Constant-time comparison
    let mut diff: u8 = 0;
    for (&a, &b) in expected.iter().zip(received_tag.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}

/// Errors during heartbeat authentication (T-08).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeartbeatAuthError {
    /// HMAC tag verification failed.
    HmacInvalid,
    /// TL-DSA signature verification failed.
    SignatureInvalid,
    /// Sequence number not strictly increasing (replay attempt).
    SequenceReplay {
        received: u64,
        last_accepted: u64,
    },
    /// Unknown authentication mode.
    UnknownAuthMode(u8),
    /// Address not found in neighbor list.
    UnknownAddress,
}

impl std::fmt::Display for HeartbeatAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HmacInvalid => write!(f, "heartbeat HMAC verification failed"),
            Self::SignatureInvalid => write!(f, "heartbeat TL-DSA signature verification failed"),
            Self::SequenceReplay { received, last_accepted } => {
                write!(f, "heartbeat sequence replay: received {}, last accepted {}", received, last_accepted)
            }
            Self::UnknownAuthMode(m) => write!(f, "unknown heartbeat auth mode 0x{:02X}", m),
            Self::UnknownAddress => write!(f, "heartbeat from unknown address"),
        }
    }
}

impl std::error::Error for HeartbeatAuthError {}

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
    /// Last accepted heartbeat sequence number (T-08).
    /// Monotonically increasing — rejects replays.
    pub last_hb_sequence: u64,
    /// Consecutive heartbeat authentication failures (T-08).
    /// At threshold (3), emits suspect event — same as missed pings.
    pub consecutive_auth_failures: u8,
    /// Cached HMAC key for this neighbor (T-08).
    /// Derived at registration time, invalidated on master_secret rotation.
    pub hmac_key: Option<Vec<u8>>,
}

impl NeighborHealth {
    /// Create a new health record from computed geometry.
    fn new(addr: CubeAddr, dimension: usize, alt_value: RepCTrit) -> Self {
        NeighborHealth {
            addr,
            dimension,
            alt_value,
            state: NeighborState::Up,
            srtt_ns: 0,
            jitter_ns: 0,
            consecutive_misses: 0,
            consecutive_successes: 0,
            last_pong: None,
            suspect_since: None,
            last_hb_sequence: 0,
            consecutive_auth_failures: 0,
            hmac_key: None,
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
// STATE CHANGE EVENT
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
    /// Consecutive auth failures before suspect event (T-08).
    pub auth_failure_threshold: u8,
}

impl Default for FtsConfig {
    fn default() -> Self {
        FtsConfig {
            ping_interval: Duration::from_millis(DEFAULT_PING_INTERVAL_MS),
            miss_threshold: DEFAULT_MISS_THRESHOLD,
            recovery_threshold: DEFAULT_RECOVERY_THRESHOLD,
            grace_period: Duration::from_millis(DEFAULT_GRACE_PERIOD_MS),
            auth_failure_threshold: DEFAULT_AUTH_FAILURE_THRESHOLD,
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
    // HMAC KEY MANAGEMENT — T-08
    // ═══════════════════════════════════════════════════════════════

    /// Set the HMAC key for a specific neighbor (T-08).
    ///
    /// Called during registration when the master_secret is known.
    /// Both CRS and the node derive the same key independently.
    pub fn set_hmac_key(&mut self, addr: &CubeAddr, key: Vec<u8>) {
        if let Some(nbr) = self.neighbors.iter_mut().find(|n| n.addr == *addr) {
            nbr.hmac_key = Some(key);
        }
    }

    /// Derive and set HMAC keys for all neighbors from a master secret (T-08).
    ///
    /// Convenience function: derives `TLSponge-385("PlenumNET-HB-HMAC" ‖ addr ‖ secret)`
    /// for each of the 26 neighbors and caches the keys.
    pub fn derive_all_hmac_keys(&mut self, master_secret: &[u8]) {
        for nbr in &mut self.neighbors {
            let key = derive_hb_hmac_key(&nbr.addr, master_secret);
            nbr.hmac_key = Some(key);
        }
    }

    /// Invalidate all cached HMAC keys (T-08).
    ///
    /// Called when the master_secret rotates. New keys must be derived.
    pub fn invalidate_hmac_keys(&mut self) {
        for nbr in &mut self.neighbors {
            nbr.hmac_key = None;
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // HEARTBEAT PROCESSING — Legacy (unauthenticated)
    // ═══════════════════════════════════════════════════════════════

    /// Record a successful pong response from a neighbor (legacy, unauthenticated).
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
            nbr.consecutive_auth_failures = 0; // Reset on any successful pong

            // State transitions on pong
            match nbr.state {
                NeighborState::Up => {}
                NeighborState::Suspect => {
                    nbr.state = NeighborState::Recovering;
                    nbr.suspect_since = None;
                    nbr.consecutive_successes = 1;
                }
                NeighborState::Down => {
                    nbr.state = NeighborState::Recovering;
                    nbr.consecutive_successes = 1;
                }
                NeighborState::Recovering => {
                    if nbr.consecutive_successes >= config.recovery_threshold {
                        nbr.state = NeighborState::Up;
                    }
                }
            }

            if nbr.state != old_state {
                self.pending_events.push(StateChangeEvent {
                    addr: addr.clone(),
                    from: old_state,
                    to: nbr.state,
                    timestamp: now,
                });
            }

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

            match nbr.state {
                NeighborState::Up => {
                    if nbr.consecutive_misses >= config.miss_threshold {
                        nbr.state = NeighborState::Suspect;
                        nbr.suspect_since = Some(now);
                    }
                }
                NeighborState::Suspect => {
                    if let Some(since) = nbr.suspect_since {
                        if now.duration_since(since) >= config.grace_period {
                            nbr.state = NeighborState::Down;
                            nbr.suspect_since = None;
                        }
                    }
                }
                NeighborState::Down => {}
                NeighborState::Recovering => {
                    nbr.state = NeighborState::Suspect;
                    nbr.suspect_since = Some(now);
                }
            }

            if nbr.state != old_state {
                self.pending_events.push(StateChangeEvent {
                    addr: addr.clone(),
                    from: old_state,
                    to: nbr.state,
                    timestamp: now,
                });
            }

            self.rebuild_dead_set();
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // HEARTBEAT PROCESSING — Authenticated (T-08)
    // ═══════════════════════════════════════════════════════════════

    /// Process an authenticated heartbeat (T-08).
    ///
    /// Verifies the authentication data (HMAC or TL-DSA signature),
    /// checks sequence monotonicity, updates RTT and state machine.
    ///
    /// On authentication failure: increments `consecutive_auth_failures`.
    /// At threshold (3), triggers suspect event — same escalation path
    /// as missed pings. This prevents forged heartbeats from keeping
    /// a hijacked registration alive.
    ///
    /// ## Returns
    ///
    /// - `Ok(())` — heartbeat accepted, state updated
    /// - `Err(HeartbeatAuthError)` — authentication or sequence check failed
    pub fn record_authenticated_pong(
        &mut self,
        hb: &AuthenticatedHeartbeat,
        rtt_ns: u64,
    ) -> Result<(), HeartbeatAuthError> {
        let config = self.config.clone();
        let now = Instant::now();

        // Find the neighbor
        let nbr_idx = self.neighbors
            .iter()
            .position(|n| n.addr == hb.address)
            .ok_or(HeartbeatAuthError::UnknownAddress)?;

        // Step 1: Check sequence number (replay protection)
        let last_seq = self.neighbors[nbr_idx].last_hb_sequence;
        if hb.sequence <= last_seq && last_seq > 0 {
            // Allow sliding window for minor reordering
            if last_seq - hb.sequence > SEQUENCE_WINDOW {
                return Err(HeartbeatAuthError::SequenceReplay {
                    received: hb.sequence,
                    last_accepted: last_seq,
                });
            }
            // Within window but not strictly newer — accept but don't update last_seq
        }

        // Step 2: Verify authentication
        let canonical_msg = hb.canonical_message();

        match hb.auth_mode {
            HeartbeatAuth::TisHmac => {
                let hmac_key = self.neighbors[nbr_idx]
                    .hmac_key
                    .as_ref()
                    .ok_or(HeartbeatAuthError::HmacInvalid)?;

                if !verify_hb_hmac(hmac_key, &canonical_msg, &hb.auth_data) {
                    self.record_auth_failure(&hb.address, now, &config);
                    return Err(HeartbeatAuthError::HmacInvalid);
                }
            }
            HeartbeatAuth::TlDsaSig => {
                // For TL-DSA mode, we need the neighbor's public key
                // This is stored in the CON neighbor record, not FTS
                // For now, verify using the auth_data as signature
                // against the public key passed through the heartbeat system
                //
                // NOTE: Full TL-DSA heartbeat verification requires the neighbor's
                // public key from CON. For the MVP, HMAC mode is the default.
                // TL-DSA mode will be wired when CON exposes neighbor public keys to FTS.
                return Err(HeartbeatAuthError::SignatureInvalid);
            }
        }

        // Step 3: Authentication passed — update state
        let nbr = &mut self.neighbors[nbr_idx];
        let old_state = nbr.state;

        // Update sequence (only if strictly newer)
        if hb.sequence > nbr.last_hb_sequence {
            nbr.last_hb_sequence = hb.sequence;
        }

        // Reset auth failure counter on success
        nbr.consecutive_auth_failures = 0;

        // Update RTT metrics
        if nbr.srtt_ns == 0 {
            nbr.srtt_ns = rtt_ns;
            nbr.jitter_ns = rtt_ns / 2;
        } else {
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

        // State transitions (same as record_pong)
        match nbr.state {
            NeighborState::Up => {}
            NeighborState::Suspect => {
                nbr.state = NeighborState::Recovering;
                nbr.suspect_since = None;
                nbr.consecutive_successes = 1;
            }
            NeighborState::Down => {
                nbr.state = NeighborState::Recovering;
                nbr.consecutive_successes = 1;
            }
            NeighborState::Recovering => {
                if nbr.consecutive_successes >= config.recovery_threshold {
                    nbr.state = NeighborState::Up;
                }
            }
        }

        if nbr.state != old_state {
            self.pending_events.push(StateChangeEvent {
                addr: hb.address.clone(),
                from: old_state,
                to: nbr.state,
                timestamp: now,
            });
        }

        self.rebuild_dead_set();
        Ok(())
    }

    /// Record an authentication failure (T-08).
    ///
    /// Internal helper called when HMAC or signature verification fails.
    /// After `auth_failure_threshold` consecutive failures, escalates
    /// to suspect event — same as missed pings.
    fn record_auth_failure(&mut self, addr: &CubeAddr, now: Instant, config: &FtsConfig) {
        if let Some(nbr) = self.neighbors.iter_mut().find(|n| n.addr == *addr) {
            let old_state = nbr.state;
            nbr.consecutive_auth_failures += 1;

            if nbr.consecutive_auth_failures >= config.auth_failure_threshold {
                if nbr.state == NeighborState::Up {
                    nbr.state = NeighborState::Suspect;
                    nbr.suspect_since = Some(now);
                    println!(
                        "[FTS] Node {} → Suspect (3 consecutive auth failures)",
                        addr
                    );
                }
            }

            if nbr.state != old_state {
                self.pending_events.push(StateChangeEvent {
                    addr: addr.clone(),
                    from: old_state,
                    to: nbr.state,
                    timestamp: now,
                });
            }

            self.rebuild_dead_set();
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // DEAD SET
    // ═══════════════════════════════════════════════════════════════

    fn rebuild_dead_set(&mut self) {
        self.dead_set.clear();
        for nbr in &self.neighbors {
            if nbr.state.is_dead() {
                self.dead_set.insert(nbr.addr.clone());
            }
        }
    }

    /// Get the current dead neighbor set.
    pub fn dead_set(&self) -> &HashSet<CubeAddr> {
        &self.dead_set
    }

    /// Get a cloned dead set (for passing to GLB).
    pub fn dead_set_cloned(&self) -> HashSet<CubeAddr> {
        self.dead_set.clone()
    }

    // ═══════════════════════════════════════════════════════════════
    // EVENT DRAIN
    // ═══════════════════════════════════════════════════════════════

    pub fn drain_events(&mut self) -> Vec<StateChangeEvent> {
        std::mem::take(&mut self.pending_events)
    }

    pub fn has_pending_events(&self) -> bool {
        !self.pending_events.is_empty()
    }

    // ═══════════════════════════════════════════════════════════════
    // QUERY
    // ═══════════════════════════════════════════════════════════════

    pub fn all_status(&self) -> &[NeighborHealth] {
        &self.neighbors
    }

    pub fn neighbor_health(&self, addr: &CubeAddr) -> Option<&NeighborHealth> {
        self.neighbors.iter().find(|n| n.addr == *addr)
    }

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

    pub fn config(&self) -> &FtsConfig {
        &self.config
    }

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

    // ── Original tests (unchanged) ──────────────────────────────

    #[test]
    fn test_initial_state() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let fts = FaultToleranceService::new(local);
        assert_eq!(fts.neighbors.len(), NEIGHBORS_PER_CUBE);
        assert!(fts.dead_set().is_empty());
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
        fts.record_pong(&nbr, 1_000_000);
        let health = fts.neighbor_health(&nbr).unwrap();
        assert_eq!(health.srtt_ns, 1_000_000);
        assert!(health.last_pong.is_some());
    }

    #[test]
    fn test_miss_threshold_to_suspect() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut fts = FaultToleranceService::new(local);
        let nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        fts.record_miss(&nbr);
        assert_eq!(fts.neighbor_health(&nbr).unwrap().state, NeighborState::Up);
        fts.record_miss(&nbr);
        assert_eq!(fts.neighbor_health(&nbr).unwrap().state, NeighborState::Up);
        fts.record_miss(&nbr);
        assert_eq!(fts.neighbor_health(&nbr).unwrap().state, NeighborState::Suspect);
        assert!(fts.dead_set().contains(&nbr));
    }

    #[test]
    fn test_suspect_to_down_after_grace() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let config = FtsConfig {
            grace_period: Duration::from_millis(0),
            ..Default::default()
        };
        let mut fts = FaultToleranceService::new(local).with_config(config);
        let nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        for _ in 0..3 { fts.record_miss(&nbr); }
        assert_eq!(fts.neighbor_health(&nbr).unwrap().state, NeighborState::Suspect);
        fts.record_miss(&nbr);
        assert_eq!(fts.neighbor_health(&nbr).unwrap().state, NeighborState::Down);
    }

    #[test]
    fn test_recovery_from_down() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let config = FtsConfig {
            grace_period: Duration::from_millis(0),
            recovery_threshold: 3,
            ..Default::default()
        };
        let mut fts = FaultToleranceService::new(local).with_config(config);
        let nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        for _ in 0..4 { fts.record_miss(&nbr); }
        assert_eq!(fts.neighbor_health(&nbr).unwrap().state, NeighborState::Down);
        fts.record_pong(&nbr, 500_000);
        assert_eq!(fts.neighbor_health(&nbr).unwrap().state, NeighborState::Recovering);
        fts.record_pong(&nbr, 500_000);
        fts.record_pong(&nbr, 500_000);
        assert_eq!(fts.neighbor_health(&nbr).unwrap().state, NeighborState::Up);
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
        for _ in 0..4 { fts.record_miss(&nbr); }
        fts.record_pong(&nbr, 500_000);
        assert_eq!(fts.neighbor_health(&nbr).unwrap().state, NeighborState::Recovering);
        fts.record_miss(&nbr);
        assert_eq!(fts.neighbor_health(&nbr).unwrap().state, NeighborState::Suspect);
    }

    #[test]
    fn test_events_emitted() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut fts = FaultToleranceService::new(local);
        let nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        assert!(!fts.has_pending_events());
        for _ in 0..3 { fts.record_miss(&nbr); }
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
        let mut glb = GeometricLoadBalancer::new(local);
        let dead_nbr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        for _ in 0..4 { fts.record_miss(&dead_nbr); }
        glb.set_dead_neighbors(fts.dead_set_cloned());
        let result = glb.forward_stateless(&dest, 42).unwrap();
        assert_ne!(result.next_hop, dead_nbr);
        assert_eq!(result.dimension_fixed, 1);
    }

    // ── T-08: Authenticated heartbeat tests ─────────────────────

    #[test]
    fn test_hmac_key_derivation_deterministic() {
        let a = addr([1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1]);
        let secret = b"test-master-secret-for-hmac";
        let key1 = derive_hb_hmac_key(&a, secret);
        let key2 = derive_hb_hmac_key(&a, secret);
        assert_eq!(key1, key2, "Same (addr, secret) → same HMAC key");
        assert_eq!(key1.len(), HB_HMAC_KEY_LEN);
    }

    #[test]
    fn test_hmac_key_different_addresses() {
        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let b = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let secret = b"test-master-secret";
        assert_ne!(
            derive_hb_hmac_key(&a, secret),
            derive_hb_hmac_key(&b, secret),
            "Different addresses → different HMAC keys"
        );
    }

    #[test]
    fn test_hmac_key_different_secrets() {
        let a = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        assert_ne!(
            derive_hb_hmac_key(&a, b"secret-one"),
            derive_hb_hmac_key(&a, b"secret-two"),
            "Different secrets → different HMAC keys"
        );
    }

    #[test]
    fn test_hmac_compute_and_verify() {
        let key = derive_hb_hmac_key(
            &addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
            b"test-secret",
        );
        let msg = b"heartbeat-payload-data";
        let tag = compute_hb_hmac(&key, msg);
        assert_eq!(tag.len(), HB_HMAC_TAG_LEN);
        assert!(verify_hb_hmac(&key, msg, &tag), "Valid HMAC must verify");
    }

    #[test]
    fn test_hmac_wrong_key_fails() {
        let key1 = derive_hb_hmac_key(
            &addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
            b"secret-one",
        );
        let key2 = derive_hb_hmac_key(
            &addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
            b"secret-two",
        );
        let msg = b"heartbeat-payload";
        let tag = compute_hb_hmac(&key1, msg);
        assert!(!verify_hb_hmac(&key2, msg, &tag), "Wrong key must fail");
    }

    #[test]
    fn test_hmac_wrong_message_fails() {
        let key = derive_hb_hmac_key(
            &addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
            b"test-secret",
        );
        let tag = compute_hb_hmac(&key, b"correct-message");
        assert!(!verify_hb_hmac(&key, b"wrong-message", &tag), "Wrong message must fail");
    }

    #[test]
    fn test_authenticated_pong_valid_hmac() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let nbr_addr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let secret = b"test-master-secret";

        let mut fts = FaultToleranceService::new(local);
        fts.derive_all_hmac_keys(secret);

        let hmac_key = derive_hb_hmac_key(&nbr_addr, secret);

        let hb = AuthenticatedHeartbeat {
            address: nbr_addr.clone(),
            endpoint: "10.0.0.1:51820".to_string(),
            sequence: 1,
            timestamp_fs: 100 * crate::wire::FS_PER_SECOND,
            auth_mode: HeartbeatAuth::TisHmac,
            auth_data: compute_hb_hmac(&hmac_key, &{
                let mut m = Vec::new();
                m.extend_from_slice(&nbr_addr.to_bytes());
                m.extend_from_slice(b"10.0.0.1:51820");
                m.extend_from_slice(&1u64.to_le_bytes());
                m.extend_from_slice(&(100u128 * crate::wire::FS_PER_SECOND).to_le_bytes());
                m
            }),
        };

        let result = fts.record_authenticated_pong(&hb, 500_000);
        assert!(result.is_ok(), "Valid HMAC heartbeat must succeed");
        assert_eq!(fts.neighbor_health(&nbr_addr).unwrap().last_hb_sequence, 1);
    }

    #[test]
    fn test_authenticated_pong_invalid_hmac() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let nbr_addr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        let mut fts = FaultToleranceService::new(local);
        fts.derive_all_hmac_keys(b"correct-secret");

        let hb = AuthenticatedHeartbeat {
            address: nbr_addr.clone(),
            endpoint: "10.0.0.1:51820".to_string(),
            sequence: 1,
            timestamp_fs: 100 * crate::wire::FS_PER_SECOND,
            auth_mode: HeartbeatAuth::TisHmac,
            auth_data: vec![0u8; HB_HMAC_TAG_LEN], // Wrong tag
        };

        let result = fts.record_authenticated_pong(&hb, 500_000);
        assert_eq!(result.unwrap_err(), HeartbeatAuthError::HmacInvalid);
        assert_eq!(fts.neighbor_health(&nbr_addr).unwrap().consecutive_auth_failures, 1);
    }

    #[test]
    fn test_three_auth_failures_trigger_suspect() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let nbr_addr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        let mut fts = FaultToleranceService::new(local);
        fts.derive_all_hmac_keys(b"correct-secret");

        let bad_hb = AuthenticatedHeartbeat {
            address: nbr_addr.clone(),
            endpoint: "10.0.0.1:51820".to_string(),
            sequence: 1,
            timestamp_fs: 100 * crate::wire::FS_PER_SECOND,
            auth_mode: HeartbeatAuth::TisHmac,
            auth_data: vec![0u8; HB_HMAC_TAG_LEN], // Wrong tag
        };

        // 3 consecutive auth failures → Suspect
        for _ in 0..3 {
            let _ = fts.record_authenticated_pong(&bad_hb, 500_000);
        }

        assert_eq!(
            fts.neighbor_health(&nbr_addr).unwrap().state,
            NeighborState::Suspect,
            "3 consecutive auth failures must trigger Suspect"
        );
        assert!(fts.dead_set().contains(&nbr_addr));
    }

    #[test]
    fn test_sequence_replay_rejected() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let nbr_addr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let secret = b"test-secret";

        let mut fts = FaultToleranceService::new(local);
        fts.derive_all_hmac_keys(secret);

        let hmac_key = derive_hb_hmac_key(&nbr_addr, secret);

        // Build valid heartbeat at sequence 100
        let build_hb = |seq: u64| {
            let hb_inner = AuthenticatedHeartbeat {
                address: nbr_addr.clone(),
                endpoint: "10.0.0.1:51820".to_string(),
                sequence: seq,
                timestamp_fs: (100 + seq as u128) * crate::wire::FS_PER_SECOND,
                auth_mode: HeartbeatAuth::TisHmac,
                auth_data: vec![], // placeholder
            };
            let msg = hb_inner.canonical_message();
            let tag = compute_hb_hmac(&hmac_key, &msg);
            AuthenticatedHeartbeat {
                auth_data: tag,
                ..hb_inner
            }
        };

        // Accept sequence 100
        let hb100 = build_hb(100);
        assert!(fts.record_authenticated_pong(&hb100, 500_000).is_ok());

        // Replay sequence 50 (well outside window) → rejected
        let hb50 = build_hb(50);
        let err = fts.record_authenticated_pong(&hb50, 500_000).unwrap_err();
        assert!(matches!(err, HeartbeatAuthError::SequenceReplay { .. }));
    }

    #[test]
    fn test_derive_all_hmac_keys() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut fts = FaultToleranceService::new(local);

        // Initially no keys
        for nbr in fts.all_status() {
            assert!(nbr.hmac_key.is_none());
        }

        // Derive all
        fts.derive_all_hmac_keys(b"master-secret");

        // All 26 neighbors should have keys
        for nbr in fts.all_status() {
            assert!(nbr.hmac_key.is_some());
            assert_eq!(nbr.hmac_key.as_ref().unwrap().len(), HB_HMAC_KEY_LEN);
        }
    }

    #[test]
    fn test_invalidate_hmac_keys() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let mut fts = FaultToleranceService::new(local);
        fts.derive_all_hmac_keys(b"master-secret");

        // Invalidate (master secret rotation)
        fts.invalidate_hmac_keys();

        for nbr in fts.all_status() {
            assert!(nbr.hmac_key.is_none());
        }
    }

    #[test]
    fn test_auth_failure_reset_on_success() {
        let local = addr([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        let nbr_addr = addr([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

        let mut fts = FaultToleranceService::new(local);
        fts.derive_all_hmac_keys(b"secret");

        // 2 auth failures (below threshold)
        let bad_hb = AuthenticatedHeartbeat {
            address: nbr_addr.clone(),
            endpoint: "10.0.0.1:51820".to_string(),
            sequence: 1,
            timestamp_fs: 100 * crate::wire::FS_PER_SECOND,
            auth_mode: HeartbeatAuth::TisHmac,
            auth_data: vec![0u8; HB_HMAC_TAG_LEN],
        };
        let _ = fts.record_authenticated_pong(&bad_hb, 500_000);
        let _ = fts.record_authenticated_pong(&bad_hb, 500_000);
        assert_eq!(fts.neighbor_health(&nbr_addr).unwrap().consecutive_auth_failures, 2);

        // One successful legacy pong resets the counter
        fts.record_pong(&nbr_addr, 500_000);
        assert_eq!(fts.neighbor_health(&nbr_addr).unwrap().consecutive_auth_failures, 0);
    }
}