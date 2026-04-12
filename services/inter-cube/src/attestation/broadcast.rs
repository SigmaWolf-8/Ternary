// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// Attestation Broadcast Service — Task #119 Task 3
// HModal signal dispatch with HPTP-jittered intervals

//! Periodic attestation broadcast using HModal signal dispatch.
//!
//! The attestation interval follows the HModal duty cycle from the
//! framework's signal model (TM-2026-028):
//!
//!   duty = 1/R₂ = 1/4 → 25% dispatch, 75% idle
//!   dispatch_ratio = 1/3 → dispatch time = idle time / 3
//!
//! During the idle phase (α state, 75% dwell), the node collects
//! heartbeat challenges and builds the rolling Merkle tree.
//! During the dispatch phase (β state, 25% dwell), the node signs
//! and broadcasts the attestation report to geometric neighbors.
//!
//! Jitter: TIS-27(timestamp ‖ node_rep_c ‖ counter) mod range.
//! Backoff: per-link, exponential, capped at interval_max_s.
//! Rate floor: minimum 1 report per interval_max_s.

use std::collections::HashMap;
use ternary_math::trit_int::TritInt;

use crate::cube_addr::CubeAddr;

// ═══════════════════════════════════════════════════════════════════════
// HMODAL DISPATCH CONSTANTS — from framework signal model
// ═══════════════════════════════════════════════════════════════════════

/// Duty cycle numerator: d = 1/R₂ = 1/4.
/// 25% of the interval is the dispatch window.
const DUTY_NUM: u32 = ternary_math::constants::DUTY_NUM;

/// Duty cycle denominator.
const DUTY_DEN: u32 = ternary_math::constants::DUTY_DEN;

/// Dispatch-to-idle time ratio = 1/TERNARY_BASE = 1/3.
/// dispatch_time = idle_time / 3.
const DISPATCH_RATIO_NUM: u32 = ternary_math::constants::DISPATCH_RATIO_NUM;
const DISPATCH_RATIO_DEN: u32 = ternary_math::constants::DISPATCH_RATIO_DEN;

// ═══════════════════════════════════════════════════════════════════════
// CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════

/// Attestation broadcast configuration (from PlenumConfig).
#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    /// Base interval in seconds before jitter. Default: 30. Range: 10–120.
    pub interval_base_s: u16,
    /// Maximum interval in seconds (cap for jitter and backoff). Default: 120. Range: 30–300.
    pub interval_max_s: u16,
    /// Per-link bandwidth threshold (% of link capacity) before backoff. Default: 5. Range: 1–50.
    pub bandwidth_threshold_pct: u8,
    /// Whether attestation is enabled. Default: true.
    pub enabled: bool,
}

impl Default for BroadcastConfig {
    fn default() -> Self {
        Self {
            interval_base_s: 30,
            interval_max_s: 120,
            bandwidth_threshold_pct: 5,
            enabled: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// HMODAL DISPATCH CYCLE
// ═══════════════════════════════════════════════════════════════════════

/// HModal dispatch phase within an attestation interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchPhase {
    /// α state (idle): collecting heartbeat challenges, building Merkle tree.
    /// Occupies (1 - duty) = 3/4 of the interval.
    Idle,
    /// β state (dispatch): signing and broadcasting attestation report.
    /// Occupies duty = 1/4 of the interval.
    Dispatch,
}

/// Timing split for one attestation interval using HModal duty cycle.
#[derive(Debug, Clone, Copy)]
pub struct HModalTiming {
    /// Total interval in seconds (after jitter + backoff).
    pub total_s: u16,
    /// Idle phase duration in seconds = total × (1 - duty) = total × 3/4.
    pub idle_s: u16,
    /// Dispatch phase duration in seconds = total × duty = total × 1/4.
    pub dispatch_s: u16,
}

impl HModalTiming {
    /// Compute the HModal timing split for a given total interval.
    pub fn from_interval(total_s: u16) -> Self {
        // dispatch = total × DUTY_NUM / DUTY_DEN = total / 4
        let dispatch_s = (total_s as u32 * DUTY_NUM / DUTY_DEN) as u16;
        // idle = total - dispatch = total × 3/4
        let idle_s = total_s - dispatch_s;
        HModalTiming { total_s, idle_s, dispatch_s }
    }

    /// Determine which phase we're in given elapsed seconds since interval start.
    pub fn phase_at(&self, elapsed_s: u16) -> DispatchPhase {
        if elapsed_s < self.idle_s {
            DispatchPhase::Idle
        } else {
            DispatchPhase::Dispatch
        }
    }

    /// Seconds until the dispatch window opens.
    /// Returns 0 if already in dispatch phase.
    pub fn seconds_until_dispatch(&self, elapsed_s: u16) -> u16 {
        if elapsed_s >= self.idle_s {
            0
        } else {
            self.idle_s - elapsed_s
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PER-LINK BANDWIDTH STATE
// ═══════════════════════════════════════════════════════════════════════

/// Per-link bandwidth tracking and backoff state.
#[derive(Debug, Clone)]
pub struct LinkBandwidthState {
    /// Configured link capacity in bytes/sec. None = budget disabled (fail-open).
    pub link_capacity_bps: Option<u64>,
    /// Attestation bytes sent in current measurement window.
    pub attest_bytes_sent: u64,
    /// Current backoff multiplier (1 = no backoff, 2 = doubled, etc.).
    pub backoff_level: u32,
    /// Window start timestamp (femtoseconds since Salvi Epoch).
    pub window_start_fs: u128,
}

impl LinkBandwidthState {
    pub fn new(capacity: Option<u64>, now_fs: u128) -> Self {
        Self {
            link_capacity_bps: capacity,
            attest_bytes_sent: 0,
            backoff_level: 1,
            window_start_fs: now_fs,
        }
    }

    /// Check if attestation traffic exceeds the configured threshold.
    pub fn exceeds_threshold(&self, threshold_pct: u8, window_duration_s: u64) -> bool {
        let cap = match self.link_capacity_bps {
            Some(c) => c,
            None => return false, // No capacity → budget disabled
        };
        let budget = cap * window_duration_s * threshold_pct as u64 / 100;
        self.attest_bytes_sent > budget
    }

    /// Apply exponential backoff: double the interval.
    pub fn apply_backoff(&mut self) {
        self.backoff_level = self.backoff_level.saturating_mul(2);
    }

    /// Reset backoff when traffic drops below threshold.
    pub fn reset_backoff(&mut self) {
        self.backoff_level = 1;
    }

    /// Record bytes sent on this link.
    pub fn record_sent(&mut self, bytes: u64) {
        self.attest_bytes_sent = self.attest_bytes_sent.saturating_add(bytes);
    }

    /// Reset measurement window.
    pub fn reset_window(&mut self, now_fs: u128) {
        self.attest_bytes_sent = 0;
        self.window_start_fs = now_fs;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// JITTER COMPUTATION
// ═══════════════════════════════════════════════════════════════════════

/// Compute the jittered attestation interval for a specific tick.
///
/// Jitter = TIS-27(timestamp ‖ node_rep_c ‖ counter) mod (max - base) + base.
/// Cryptographic: deterministic per-node per-interval, unpredictable to
/// external observer without knowledge of node_rep_c and counter.
pub fn compute_jittered_interval(
    hptp_timestamp: u128,
    node_addr: &CubeAddr,
    interval_counter: u64,
    base_s: u16,
    max_s: u16,
) -> u16 {
    let range = max_s.saturating_sub(base_s);
    if range == 0 {
        return base_s;
    }

    let ts_bytes = hptp_timestamp.to_le_bytes();
    let addr_bytes = node_addr.to_bytes();
    let ctr_bytes = interval_counter.to_le_bytes();

    let mut input = Vec::with_capacity(ts_bytes.len() + addr_bytes.len() + ctr_bytes.len());
    input.extend_from_slice(&ts_bytes);
    input.extend_from_slice(&addr_bytes);
    input.extend_from_slice(&ctr_bytes);

    let hash = ternary_math::sponge::derive_key(
        b"PLENUMNET-ATTEST-JITTER",
        &input,
        2,
    );

    let raw = u16::from_le_bytes([hash[0], hash[1]]);
    base_s + (raw % range)
}

/// Compute the effective interval accounting for backoff.
/// Rate floor: never exceeds interval_max_s.
pub fn effective_interval(base_jittered_s: u16, backoff_level: u32, max_s: u16) -> u16 {
    let backed_off = (base_jittered_s as u32).saturating_mul(backoff_level);
    std::cmp::min(backed_off as u16, max_s)
}

// ═══════════════════════════════════════════════════════════════════════
// BROADCAST STATE
// ═══════════════════════════════════════════════════════════════════════

/// State for the attestation broadcast service.
pub struct BroadcastState {
    /// Configuration.
    pub config: BroadcastConfig,
    /// Interval counter for jitter computation.
    pub interval_counter: u64,
    /// Per-link bandwidth state, keyed by neighbor Rep C address.
    pub link_states: HashMap<CubeAddr, LinkBandwidthState>,
    /// Current HModal timing for this interval.
    pub timing: HModalTiming,
}

impl BroadcastState {
    pub fn new(config: BroadcastConfig) -> Self {
        let timing = HModalTiming::from_interval(config.interval_base_s);
        Self {
            config,
            interval_counter: 0,
            link_states: HashMap::new(),
            timing,
        }
    }

    /// Compute the next interval and its HModal timing split.
    pub fn next_interval(&mut self, node_addr: &CubeAddr, neighbor: &CubeAddr, now_fs: u128) -> HModalTiming {
        let jittered = compute_jittered_interval(
            now_fs,
            node_addr,
            self.interval_counter,
            self.config.interval_base_s,
            self.config.interval_max_s,
        );

        let backoff = self.link_states.get(neighbor)
            .map(|ls| ls.backoff_level)
            .unwrap_or(1);

        let effective = effective_interval(jittered, backoff, self.config.interval_max_s);
        let timing = HModalTiming::from_interval(effective);
        self.timing = timing;
        timing
    }

    /// Record that an attestation report was sent to a neighbor.
    pub fn record_send(&mut self, neighbor: &CubeAddr, bytes: u64, now_fs: u128) {
        let state = self.link_states.entry(neighbor.clone())
            .or_insert_with(|| LinkBandwidthState::new(None, now_fs));
        state.record_sent(bytes);

        let window_s = self.config.interval_base_s as u64;
        if state.exceeds_threshold(self.config.bandwidth_threshold_pct, window_s) {
            state.apply_backoff();
        }
    }

    /// Advance to next interval.
    pub fn tick(&mut self) {
        self.interval_counter += 1;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn addr1() -> CubeAddr { CubeAddr::new([1; 13]) }
    fn addr2() -> CubeAddr { CubeAddr::new([2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]) }

    // ── HModal timing tests ─────────────────────────────────────

    #[test]
    fn hmodal_duty_cycle_split() {
        let t = HModalTiming::from_interval(120);
        // duty = 1/4 → dispatch = 30, idle = 90
        assert_eq!(t.dispatch_s, 30);
        assert_eq!(t.idle_s, 90);
        assert_eq!(t.total_s, 120);
    }

    #[test]
    fn hmodal_dispatch_ratio() {
        let t = HModalTiming::from_interval(120);
        // dispatch / idle = 1/3 (DISPATCH_RATIO)
        assert_eq!(t.dispatch_s as u32 * DISPATCH_RATIO_DEN,
                   t.idle_s as u32 * DISPATCH_RATIO_NUM,
                   "dispatch/idle should equal 1/3");
    }

    #[test]
    fn hmodal_phase_transition() {
        let t = HModalTiming::from_interval(120);
        // 0..89 = idle, 90..119 = dispatch
        assert_eq!(t.phase_at(0), DispatchPhase::Idle);
        assert_eq!(t.phase_at(89), DispatchPhase::Idle);
        assert_eq!(t.phase_at(90), DispatchPhase::Dispatch);
        assert_eq!(t.phase_at(119), DispatchPhase::Dispatch);
    }

    #[test]
    fn hmodal_seconds_until_dispatch() {
        let t = HModalTiming::from_interval(120);
        assert_eq!(t.seconds_until_dispatch(0), 90);
        assert_eq!(t.seconds_until_dispatch(45), 45);
        assert_eq!(t.seconds_until_dispatch(90), 0);
        assert_eq!(t.seconds_until_dispatch(100), 0);
    }

    #[test]
    fn hmodal_base_interval() {
        let t = HModalTiming::from_interval(30);
        // 30s: dispatch = 7, idle = 23 (integer truncation of 30/4)
        assert_eq!(t.dispatch_s, 7);
        assert_eq!(t.idle_s, 23);
    }

    // ── Jitter tests ────────────────────────────────────────────

    #[test]
    fn jitter_within_range() {
        let addr = addr1();
        for counter in 0..100u64 {
            let interval = compute_jittered_interval(
                1_000_000_000_000_000, &addr, counter, 30, 120,
            );
            assert!(interval >= 30 && interval <= 120,
                "interval {interval} out of range for counter {counter}");
        }
    }

    #[test]
    fn jitter_deterministic() {
        let addr = addr1();
        let a = compute_jittered_interval(42, &addr, 7, 30, 120);
        let b = compute_jittered_interval(42, &addr, 7, 30, 120);
        assert_eq!(a, b);
    }

    // ── Backoff tests ───────────────────────────────────────────

    #[test]
    fn backoff_doubles_interval() {
        assert_eq!(effective_interval(30, 1, 120), 30);
        assert_eq!(effective_interval(30, 2, 120), 60);
        assert_eq!(effective_interval(30, 4, 120), 120);
    }

    #[test]
    fn backoff_capped_at_max() {
        assert_eq!(effective_interval(30, 100, 120), 120);
    }

    #[test]
    fn bandwidth_threshold_triggers_backoff() {
        let mut state = LinkBandwidthState::new(Some(10_000), 0);
        assert!(!state.exceeds_threshold(5, 30));
        state.record_sent(20_000);
        assert!(state.exceeds_threshold(5, 30));
    }

    #[test]
    fn no_capacity_disables_budget() {
        let mut state = LinkBandwidthState::new(None, 0);
        state.record_sent(1_000_000_000);
        assert!(!state.exceeds_threshold(5, 30));
    }

    // ── BroadcastState with HModal ──────────────────────────────

    #[test]
    fn broadcast_returns_hmodal_timing() {
        let mut bs = BroadcastState::new(BroadcastConfig::default());
        let node = addr1();
        let nbr = addr2();
        let timing = bs.next_interval(&node, &nbr, 1_000_000_000_000_000);
        // Should have valid HModal split
        assert_eq!(timing.idle_s + timing.dispatch_s, timing.total_s);
        assert!(timing.dispatch_s > 0);
        assert!(timing.idle_s > timing.dispatch_s, "idle > dispatch (75% > 25%)");
    }
}
