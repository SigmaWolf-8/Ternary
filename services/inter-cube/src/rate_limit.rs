// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # CRS Rate Limiting, Proof-of-Work & Ghost Scoring (T-11)
//!
//! Three defense layers for the Cube Registration Service:
//!
//! ## 1. Per-IP Rate Limiting
//!
//! Sliding-window counter: max 10 registrations per minute per source IP.
//! After the cap, further registrations from that IP are rejected with
//! `RateLimited` error until the window slides forward.
//!
//! ## 2. Proof-of-Work (PoW)
//!
//! Find `nonce` such that `TIS-27(address_bytes ‖ nonce_le)` has `K`
//! leading zero trits. The TIS-27 sponge squeeze produces trits in
//! balanced ternary {-1, 0, 1} — we count leading zeros.
//!
//! K scales with network load:
//! - K=5 during bootstrap (<1,000 registered nodes)
//! - K=8 steady-state (default, configurable via `PlenumConfig.pow_k`)
//! - K=10 under sustained load (>90% address usage or rate limit hits)
//!
//! At K=8, expected nonce search ≈ 3⁸ = 6,561 TIS-27 evaluations.
//! At ~3µs per TIS-27, that's ~20ms — imperceptible for legitimate
//! registrations, expensive for mass Sybil creation.
//!
//! ## 3. Ghost Scoring
//!
//! A "ghost" is a node that registers but never responds to heartbeats.
//! After a grace period (configurable, default 5 minutes), the ghost
//! accumulates strikes. At 3 strikes, CRS purges the registration and
//! releases the address.
//!
//! ## 4. Heartbeat Auth Failure Rate Limiting
//!
//! Per-IP cap on heartbeat authentication failures: 30/minute.
//! Prevents an attacker from brute-forcing HMAC tags.

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

use crate::cube_addr::CubeAddr;

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Maximum registrations per IP per window.
pub const DEFAULT_REG_RATE_LIMIT: u32 = 10;

/// Rate limit window duration.
pub const DEFAULT_REG_WINDOW: Duration = Duration::from_secs(60);

/// Maximum heartbeat auth failures per IP per window.
pub const DEFAULT_HB_FAIL_RATE_LIMIT: u32 = 30;

/// Heartbeat failure rate limit window.
pub const DEFAULT_HB_FAIL_WINDOW: Duration = Duration::from_secs(60);

/// Ghost grace period: time after registration before ghost detection starts.
pub const DEFAULT_GHOST_GRACE: Duration = Duration::from_secs(300); // 5 minutes

/// Ghost strikes before purge.
pub const DEFAULT_GHOST_STRIKES: u8 = 3;

/// Ghost check interval: how often to scan for ghosts.
pub const DEFAULT_GHOST_CHECK_INTERVAL: Duration = Duration::from_secs(60);

/// PoW difficulty: bootstrap mode (<1,000 nodes).
pub const POW_K_BOOTSTRAP: u8 = 5;

/// PoW difficulty: steady-state.
pub const POW_K_STEADY: u8 = 8;

/// PoW difficulty: under load.
pub const POW_K_LOAD: u8 = 10;

/// Node count threshold for bootstrap → steady-state transition.
pub const BOOTSTRAP_THRESHOLD: usize = 1_000;

/// Address usage ratio threshold for steady-state → load transition.
pub const LOAD_THRESHOLD_RATIO: f64 = 0.90;

// ═══════════════════════════════════════════════════════════════════════
// ERRORS
// ═══════════════════════════════════════════════════════════════════════

/// Rate limiting and PoW errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardError {
    /// Source IP exceeded registration rate limit.
    RateLimited {
        ip: IpAddr,
        limit: u32,
        window_secs: u64,
    },
    /// Proof-of-work nonce is invalid (insufficient leading zero trits).
    InvalidPoW {
        required_k: u8,
        found_zeros: u8,
    },
    /// Source IP exceeded heartbeat auth failure rate limit.
    HbFailRateLimited {
        ip: IpAddr,
        limit: u32,
    },
}

impl std::fmt::Display for GuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RateLimited { ip, limit, window_secs } => {
                write!(f, "rate limited: {} exceeded {}/{}s", ip, limit, window_secs)
            }
            Self::InvalidPoW { required_k, found_zeros } => {
                write!(
                    f,
                    "PoW invalid: need {} leading zero trits, found {}",
                    required_k, found_zeros
                )
            }
            Self::HbFailRateLimited { ip, limit } => {
                write!(f, "HB auth failure rate limited: {} exceeded {}/min", ip, limit)
            }
        }
    }
}

impl std::error::Error for GuardError {}

// ═══════════════════════════════════════════════════════════════════════
// SLIDING WINDOW RATE LIMITER
// ═══════════════════════════════════════════════════════════════════════

/// Per-key sliding window rate limiter.
///
/// Tracks timestamps of recent events per key (IP address).
/// Events older than the window are pruned on each check.
#[derive(Debug, Clone)]
struct SlidingWindow {
    /// Key → list of event timestamps.
    entries: HashMap<IpAddr, Vec<Instant>>,
    /// Maximum events per window.
    limit: u32,
    /// Window duration.
    window: Duration,
}

impl SlidingWindow {
    fn new(limit: u32, window: Duration) -> Self {
        SlidingWindow {
            entries: HashMap::new(),
            limit,
            window,
        }
    }

    /// Check if an event from this IP is allowed. Does NOT record the event.
    fn is_allowed(&mut self, ip: &IpAddr, now: Instant) -> bool {
        let cutoff = now - self.window;
        if let Some(timestamps) = self.entries.get_mut(ip) {
            timestamps.retain(|&t| t > cutoff);
            timestamps.len() < self.limit as usize
        } else {
            true
        }
    }

    /// Record an event from this IP.
    fn record(&mut self, ip: IpAddr, now: Instant) {
        self.entries
            .entry(ip)
            .or_insert_with(Vec::new)
            .push(now);
    }

    /// Check and record in one call. Returns `true` if allowed.
    fn check_and_record(&mut self, ip: IpAddr, now: Instant) -> bool {
        if self.is_allowed(&ip, now) {
            self.record(ip, now);
            true
        } else {
            false
        }
    }

    /// Get the current count for an IP within the window.
    fn count(&mut self, ip: &IpAddr, now: Instant) -> u32 {
        let cutoff = now - self.window;
        if let Some(timestamps) = self.entries.get_mut(ip) {
            timestamps.retain(|&t| t > cutoff);
            timestamps.len() as u32
        } else {
            0
        }
    }

    /// Prune all expired entries (memory cleanup).
    fn prune(&mut self, now: Instant) {
        let cutoff = now - self.window;
        self.entries.retain(|_, timestamps| {
            timestamps.retain(|&t| t > cutoff);
            !timestamps.is_empty()
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PROOF OF WORK — TIS-27 Leading Zero Trits
// ═══════════════════════════════════════════════════════════════════════

/// Verify a proof-of-work nonce for a CRS registration.
///
/// Checks that `TIS-27(address_bytes ‖ nonce_le)` has at least `k`
/// leading zero trits. The TIS-27 sponge produces balanced ternary
/// output — we count zeros from the beginning.
///
/// ## Parameters
///
/// - `address_bytes`: The 13 Rep C trit bytes of the cube address
/// - `nonce`: The 64-bit nonce (little-endian in the hash input)
/// - `k`: Required number of leading zero trits
///
/// ## Returns
///
/// `Ok(())` if the PoW is valid, `Err(GuardError::InvalidPoW)` otherwise.
pub fn verify_pow(address_bytes: &[u8], nonce: u64, k: u8) -> Result<(), GuardError> {
    let zeros = count_leading_zeros(address_bytes, nonce);
    if zeros >= k {
        Ok(())
    } else {
        Err(GuardError::InvalidPoW {
            required_k: k,
            found_zeros: zeros,
        })
    }
}

/// Count leading zero trits in the TIS-27 hash of `address ‖ nonce`.
///
/// Uses TLSponge-385 `derive_key` with a PoW domain separator.
/// The output is 27 bytes; we convert each byte to a balanced trit
/// and count zeros from the front.
pub fn count_leading_zeros(address_bytes: &[u8], nonce: u64) -> u8 {
    let nonce_le = nonce.to_le_bytes();
    let mut material = Vec::with_capacity(address_bytes.len() + 8);
    material.extend_from_slice(address_bytes);
    material.extend_from_slice(&nonce_le);

    // Use TLSponge-385 derive_key with PoW domain separator
    // Output 27 bytes = 27 trits (one trit per byte in balanced representation)
    let hash = ternary_math::sponge::derive_key(b"PlenumNET-CRS-PoW", &material, 27);

    // Count leading zeros in the balanced trit representation
    // derive_key returns bytes; convert to balanced trits {-1, 0, 1}
    // via mod-3 mapping: byte mod 3 → {0, 1, 2} → balanced {-1, 0, 1}
    let mut zeros: u8 = 0;
    for &b in &hash {
        let balanced = (b % 3) as i8 - 1; // {0,1,2} → {-1, 0, 1}
        if balanced == 0 {
            zeros += 1;
        } else {
            break;
        }
    }
    zeros
}

/// Compute the appropriate PoW difficulty K for the current network state.
///
/// - `registered_count`: Current number of registered nodes
/// - `address_usage_ratio`: `registered / total_addresses` (0.0–1.0)
/// - `config_k`: Operator-configured K from `PlenumConfig.pow_k`
///
/// Returns the maximum of the operator-configured K and the state-derived K.
pub fn adaptive_pow_k(
    registered_count: usize,
    address_usage_ratio: f64,
    config_k: u8,
) -> u8 {
    let state_k = if registered_count < BOOTSTRAP_THRESHOLD {
        POW_K_BOOTSTRAP
    } else if address_usage_ratio >= LOAD_THRESHOLD_RATIO {
        POW_K_LOAD
    } else {
        POW_K_STEADY
    };
    // Operator can always raise the floor, but state can push higher
    config_k.max(state_k)
}

// ═══════════════════════════════════════════════════════════════════════
// GHOST SCORING
// ═══════════════════════════════════════════════════════════════════════

/// Ghost tracking record for a registered cube that hasn't heartbeated.
#[derive(Debug, Clone)]
pub struct GhostRecord {
    /// When the cube was registered.
    pub registered_at: Instant,
    /// Number of ghost strikes (missed ghost checks after grace period).
    pub strikes: u8,
    /// Source IP that registered this address.
    pub source_ip: IpAddr,
    /// Last time a ghost check was performed for this record.
    pub last_checked: Instant,
}

/// Ghost scoring tracker.
///
/// Monitors newly registered cubes. If a cube doesn't send any heartbeats
/// within the grace period, it starts accumulating strikes. At 3 strikes,
/// the CRS purges the registration.
#[derive(Debug)]
pub struct GhostTracker {
    /// Address → ghost record.
    records: HashMap<CubeAddr, GhostRecord>,
    /// Grace period before ghost detection starts.
    grace_period: Duration,
    /// Strikes before purge.
    max_strikes: u8,
    /// Minimum interval between ghost checks for the same address.
    check_interval: Duration,
}

impl GhostTracker {
    /// Create a new ghost tracker with default settings.
    pub fn new() -> Self {
        GhostTracker {
            records: HashMap::new(),
            grace_period: DEFAULT_GHOST_GRACE,
            max_strikes: DEFAULT_GHOST_STRIKES,
            check_interval: DEFAULT_GHOST_CHECK_INTERVAL,
        }
    }

    /// Create with custom settings.
    pub fn with_config(
        grace_period: Duration,
        max_strikes: u8,
        check_interval: Duration,
    ) -> Self {
        GhostTracker {
            records: HashMap::new(),
            grace_period,
            max_strikes,
            check_interval,
        }
    }

    /// Register a new address for ghost tracking.
    ///
    /// Called when a cube registers. If the cube heartbeats before the
    /// grace period expires, `clear()` removes it from tracking.
    pub fn track(&mut self, addr: CubeAddr, source_ip: IpAddr) {
        let now = Instant::now();
        self.records.insert(addr, GhostRecord {
            registered_at: now,
            strikes: 0,
            source_ip,
            last_checked: now,
        });
    }

    /// Clear a tracked address (cube sent a valid heartbeat).
    ///
    /// Called by FTS when a heartbeat is received. Once a cube heartbeats,
    /// it's no longer a ghost.
    pub fn clear(&mut self, addr: &CubeAddr) {
        self.records.remove(addr);
    }

    /// Check all tracked ghosts and increment strikes for those past grace.
    ///
    /// Returns the list of addresses that have exceeded `max_strikes`
    /// and should be purged from the CRS.
    pub fn check_ghosts(&mut self) -> Vec<CubeAddr> {
        let now = Instant::now();
        let mut to_purge = Vec::new();

        for (addr, record) in self.records.iter_mut() {
            // Still in grace period?
            if now.duration_since(record.registered_at) < self.grace_period {
                continue;
            }

            // Already checked recently?
            if now.duration_since(record.last_checked) < self.check_interval {
                continue;
            }

            // Increment strike
            record.strikes += 1;
            record.last_checked = now;

            if record.strikes >= self.max_strikes {
                to_purge.push(addr.clone());
            }
        }

        // Remove purged entries
        for addr in &to_purge {
            self.records.remove(addr);
        }

        to_purge
    }

    /// Number of tracked potential ghosts.
    pub fn tracked_count(&self) -> usize {
        self.records.len()
    }

    /// Get the ghost record for an address.
    pub fn ghost_record(&self, addr: &CubeAddr) -> Option<&GhostRecord> {
        self.records.get(addr)
    }

    /// Get all tracked addresses with their strike counts.
    pub fn all_tracked(&self) -> Vec<(&CubeAddr, u8)> {
        self.records
            .iter()
            .map(|(addr, rec)| (addr, rec.strikes))
            .collect()
    }
}

impl Default for GhostTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CRS GUARD — Combines all three defenses
// ═══════════════════════════════════════════════════════════════════════

/// Combined rate limiter, PoW verifier, and ghost tracker for CRS.
///
/// The API layer calls `check_registration()` before `register_signed()`.
/// If it returns `Ok`, the registration proceeds. If `Err`, the request
/// is rejected with the appropriate error.
pub struct CrsGuard {
    /// Registration rate limiter (per-IP).
    reg_limiter: SlidingWindow,
    /// Heartbeat auth failure rate limiter (per-IP).
    hb_fail_limiter: SlidingWindow,
    /// Ghost tracker.
    pub ghosts: GhostTracker,
}

impl CrsGuard {
    /// Create a new CRS guard with default settings.
    pub fn new() -> Self {
        CrsGuard {
            reg_limiter: SlidingWindow::new(DEFAULT_REG_RATE_LIMIT, DEFAULT_REG_WINDOW),
            hb_fail_limiter: SlidingWindow::new(DEFAULT_HB_FAIL_RATE_LIMIT, DEFAULT_HB_FAIL_WINDOW),
            ghosts: GhostTracker::new(),
        }
    }

    /// Check if a registration from this IP is allowed.
    ///
    /// Verifies:
    /// 1. Per-IP rate limit (10/min default)
    /// 2. Proof-of-work (K leading zero trits)
    ///
    /// Does NOT call `register_signed()` — that's the caller's job.
    pub fn check_registration(
        &mut self,
        source_ip: IpAddr,
        address_bytes: &[u8],
        pow_nonce: u64,
        required_k: u8,
    ) -> Result<(), GuardError> {
        let now = Instant::now();

        // 1. Rate limit check
        if !self.reg_limiter.check_and_record(source_ip, now) {
            return Err(GuardError::RateLimited {
                ip: source_ip,
                limit: DEFAULT_REG_RATE_LIMIT,
                window_secs: DEFAULT_REG_WINDOW.as_secs(),
            });
        }

        // 2. Proof-of-work check
        verify_pow(address_bytes, pow_nonce, required_k)?;

        Ok(())
    }

    /// Record a successful registration for ghost tracking.
    pub fn track_registration(&mut self, addr: CubeAddr, source_ip: IpAddr) {
        self.ghosts.track(addr, source_ip);
    }

    /// Clear ghost tracking when a heartbeat is received.
    pub fn on_heartbeat(&mut self, addr: &CubeAddr) {
        self.ghosts.clear(addr);
    }

    /// Check if a heartbeat auth failure from this IP is rate-limited.
    ///
    /// Returns `Ok(())` if under the limit, `Err` if rate limited.
    pub fn check_hb_failure(&mut self, source_ip: IpAddr) -> Result<(), GuardError> {
        let now = Instant::now();
        if !self.hb_fail_limiter.check_and_record(source_ip, now) {
            Err(GuardError::HbFailRateLimited {
                ip: source_ip,
                limit: DEFAULT_HB_FAIL_RATE_LIMIT,
            })
        } else {
            Ok(())
        }
    }

    /// Run periodic maintenance: ghost checks + memory cleanup.
    ///
    /// Returns addresses that should be purged from CRS.
    pub fn periodic_check(&mut self) -> Vec<CubeAddr> {
        let now = Instant::now();
        self.reg_limiter.prune(now);
        self.hb_fail_limiter.prune(now);
        self.ghosts.check_ghosts()
    }

    /// Get the current registration rate for an IP.
    pub fn reg_count(&mut self, ip: &IpAddr) -> u32 {
        self.reg_limiter.count(ip, Instant::now())
    }

    /// Get the current HB failure count for an IP.
    pub fn hb_fail_count(&mut self, ip: &IpAddr) -> u32 {
        self.hb_fail_limiter.count(ip, Instant::now())
    }
}

impl Default for CrsGuard {
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

    fn test_ip() -> IpAddr {
        "10.0.0.1".parse().unwrap()
    }

    fn test_ip2() -> IpAddr {
        "10.0.0.2".parse().unwrap()
    }

    fn test_addr() -> CubeAddr {
        CubeAddr::new([2, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2])
    }

    // ── Sliding window rate limiter tests ────────────────────────

    #[test]
    fn test_rate_limiter_allows_under_limit() {
        let mut limiter = SlidingWindow::new(3, Duration::from_secs(60));
        let ip = test_ip();
        let now = Instant::now();

        assert!(limiter.check_and_record(ip, now));
        assert!(limiter.check_and_record(ip, now));
        assert!(limiter.check_and_record(ip, now));
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let mut limiter = SlidingWindow::new(3, Duration::from_secs(60));
        let ip = test_ip();
        let now = Instant::now();

        for _ in 0..3 {
            assert!(limiter.check_and_record(ip, now));
        }
        // 4th should be blocked
        assert!(!limiter.is_allowed(&ip, now));
    }

    #[test]
    fn test_rate_limiter_different_ips_independent() {
        let mut limiter = SlidingWindow::new(2, Duration::from_secs(60));
        let now = Instant::now();

        assert!(limiter.check_and_record(test_ip(), now));
        assert!(limiter.check_and_record(test_ip(), now));
        assert!(!limiter.is_allowed(&test_ip(), now));

        // Different IP is still allowed
        assert!(limiter.check_and_record(test_ip2(), now));
    }

    #[test]
    fn test_rate_limiter_window_expires() {
        let mut limiter = SlidingWindow::new(2, Duration::from_millis(50));
        let ip = test_ip();
        let now = Instant::now();

        limiter.check_and_record(ip, now);
        limiter.check_and_record(ip, now);
        assert!(!limiter.is_allowed(&ip, now));

        // After window expires, should be allowed again
        let later = now + Duration::from_millis(60);
        assert!(limiter.is_allowed(&ip, later));
    }

    #[test]
    fn test_rate_limiter_count() {
        let mut limiter = SlidingWindow::new(10, Duration::from_secs(60));
        let ip = test_ip();
        let now = Instant::now();

        assert_eq!(limiter.count(&ip, now), 0);
        limiter.record(ip, now);
        limiter.record(ip, now);
        assert_eq!(limiter.count(&ip, now), 2);
    }

    #[test]
    fn test_rate_limiter_prune() {
        let mut limiter = SlidingWindow::new(10, Duration::from_millis(50));
        let ip = test_ip();
        let now = Instant::now();

        limiter.record(ip, now);
        assert_eq!(limiter.entries.len(), 1);

        let later = now + Duration::from_millis(60);
        limiter.prune(later);
        assert_eq!(limiter.entries.len(), 0, "Expired entries should be pruned");
    }

    // ── Proof of work tests ─────────────────────────────────────

    #[test]
    fn test_pow_count_leading_zeros() {
        let addr = [2u8, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2];
        // Just verify it returns a value in [0, 27]
        let zeros = count_leading_zeros(&addr, 0);
        assert!(zeros <= 27);
    }

    #[test]
    fn test_pow_different_nonces_different_hashes() {
        let addr = [2u8, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2];
        let h0 = ternary_math::sponge::derive_key(b"PlenumNET-CRS-PoW", &{
            let mut m = addr.to_vec(); m.extend_from_slice(&0u64.to_le_bytes()); m
        }, 27);
        let h1 = ternary_math::sponge::derive_key(b"PlenumNET-CRS-PoW", &{
            let mut m = addr.to_vec(); m.extend_from_slice(&1u64.to_le_bytes()); m
        }, 27);
        assert_ne!(h0, h1, "Different nonces must produce different hashes");
    }

    #[test]
    fn test_pow_verify_k0_always_passes() {
        let addr = [1u8; 13];
        assert!(verify_pow(&addr, 0, 0).is_ok(), "K=0 should always pass");
    }

    #[test]
    fn test_pow_verify_k27_likely_fails() {
        let addr = [1u8; 13];
        // K=27 requires ALL 27 trits to be zero — astronomically unlikely
        let result = verify_pow(&addr, 42, 27);
        assert!(result.is_err(), "K=27 should almost certainly fail");
    }

    #[test]
    fn test_pow_brute_force_k1() {
        // K=1 should be easy to find — try first 100 nonces
        let addr = [2u8, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2];
        let mut found = false;
        for nonce in 0..100 {
            if verify_pow(&addr, nonce, 1).is_ok() {
                found = true;
                break;
            }
        }
        assert!(found, "K=1 PoW should be found within 100 nonces");
    }

    // ── Adaptive K tests ────────────────────────────────────────

    #[test]
    fn test_adaptive_k_bootstrap() {
        assert_eq!(adaptive_pow_k(500, 0.01, 5), 5);
    }

    #[test]
    fn test_adaptive_k_steady_state() {
        assert_eq!(adaptive_pow_k(5000, 0.5, 5), 8);
    }

    #[test]
    fn test_adaptive_k_load() {
        assert_eq!(adaptive_pow_k(5000, 0.95, 5), 10);
    }

    #[test]
    fn test_adaptive_k_operator_override() {
        // Operator sets K=12, which is higher than any state-derived value
        assert_eq!(adaptive_pow_k(500, 0.01, 12), 12);
    }

    // ── Ghost tracker tests ─────────────────────────────────────

    #[test]
    fn test_ghost_tracker_track_and_clear() {
        let mut tracker = GhostTracker::new();
        let addr = test_addr();
        tracker.track(addr.clone(), test_ip());
        assert_eq!(tracker.tracked_count(), 1);

        tracker.clear(&addr);
        assert_eq!(tracker.tracked_count(), 0);
    }

    #[test]
    fn test_ghost_tracker_grace_period() {
        let mut tracker = GhostTracker::with_config(
            Duration::from_secs(300), // 5 min grace
            3,
            Duration::from_millis(0), // instant check interval
        );
        let addr = test_addr();
        tracker.track(addr.clone(), test_ip());

        // Within grace period — no strikes
        let purged = tracker.check_ghosts();
        assert!(purged.is_empty());
        assert_eq!(tracker.ghost_record(&addr).unwrap().strikes, 0);
    }

    #[test]
    fn test_ghost_tracker_strikes_after_grace() {
        let mut tracker = GhostTracker::with_config(
            Duration::from_millis(0), // Instant grace (expired immediately)
            3,
            Duration::from_millis(0), // instant check interval
        );
        let addr = test_addr();
        tracker.track(addr.clone(), test_ip());

        // First check: 1 strike
        let purged = tracker.check_ghosts();
        assert!(purged.is_empty());
        assert_eq!(tracker.ghost_record(&addr).unwrap().strikes, 1);

        // Second check: 2 strikes
        let purged = tracker.check_ghosts();
        assert!(purged.is_empty());
        assert_eq!(tracker.ghost_record(&addr).unwrap().strikes, 2);

        // Third check: 3 strikes → purge
        let purged = tracker.check_ghosts();
        assert_eq!(purged.len(), 1);
        assert_eq!(purged[0], addr);
        assert_eq!(tracker.tracked_count(), 0);
    }

    #[test]
    fn test_ghost_cleared_before_strikes() {
        let mut tracker = GhostTracker::with_config(
            Duration::from_millis(0),
            3,
            Duration::from_millis(0),
        );
        let addr = test_addr();
        tracker.track(addr.clone(), test_ip());

        // One strike
        tracker.check_ghosts();
        assert_eq!(tracker.ghost_record(&addr).unwrap().strikes, 1);

        // Heartbeat received — clear the ghost
        tracker.clear(&addr);
        assert_eq!(tracker.tracked_count(), 0);
    }

    // ── CRS Guard integration tests ─────────────────────────────

    #[test]
    fn test_guard_allows_registration() {
        let mut guard = CrsGuard::new();
        let ip = test_ip();
        let addr_bytes = [2u8, 1, 3, 2, 1, 3, 2, 1, 3, 2, 1, 3, 2];

        // K=0 always passes PoW
        let result = guard.check_registration(ip, &addr_bytes, 0, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_guard_rate_limits_registration() {
        let mut guard = CrsGuard::new();
        let ip = test_ip();
        let addr_bytes = [1u8; 13];

        // Exhaust the limit (K=0 for easy PoW)
        for _ in 0..DEFAULT_REG_RATE_LIMIT {
            assert!(guard.check_registration(ip, &addr_bytes, 0, 0).is_ok());
        }

        // Next should be rate limited
        let err = guard.check_registration(ip, &addr_bytes, 0, 0).unwrap_err();
        assert!(matches!(err, GuardError::RateLimited { .. }));
    }

    #[test]
    fn test_guard_hb_failure_rate_limit() {
        let mut guard = CrsGuard::new();
        let ip = test_ip();

        for _ in 0..DEFAULT_HB_FAIL_RATE_LIMIT {
            assert!(guard.check_hb_failure(ip).is_ok());
        }

        let err = guard.check_hb_failure(ip).unwrap_err();
        assert!(matches!(err, GuardError::HbFailRateLimited { .. }));
    }

    #[test]
    fn test_guard_tracks_and_clears_ghosts() {
        let mut guard = CrsGuard::new();
        let addr = test_addr();

        guard.track_registration(addr.clone(), test_ip());
        assert_eq!(guard.ghosts.tracked_count(), 1);

        guard.on_heartbeat(&addr);
        assert_eq!(guard.ghosts.tracked_count(), 0);
    }

    #[test]
    fn test_guard_reg_count() {
        let mut guard = CrsGuard::new();
        let ip = test_ip();
        let addr_bytes = [1u8; 13];

        assert_eq!(guard.reg_count(&ip), 0);
        guard.check_registration(ip, &addr_bytes, 0, 0).unwrap();
        assert_eq!(guard.reg_count(&ip), 1);
    }
}