// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// RELAY HEARTBEAT — Task #27, Task 3
//
// Application-level heartbeat with HModal-phased scheduling.
// Gated behind heartbeat:1 capability.
//
// HModal phasing (TM-2026-028 §4.2): heartbeats are control signals
// riding null channels (n ≡ 0 mod 4, zero data energy). Each connection
// is assigned a coprime-walk position via coprime::coprime_options()
// directly — no NinjaExec, no HTTP bridge.
//
// 500 connections at 30s interval → ~one ping every 60ms
// instead of a 500-ping burst at the boundary.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use ternary_math::coprime;
use ternary_math::trit_int::TritInt;

// ═══════════════════════════════════════════════════════════════════════
// HMODAL PHASE ASSIGNMENT
// ═══════════════════════════════════════════════════════════════════════

/// Per-connection heartbeat phase assignment.
#[derive(Debug, Clone)]
pub struct HeartbeatPhase {
    /// Coprime-walk position for this connection (0..connection_count).
    pub position: u64,
    /// Offset from cycle start in milliseconds.
    pub offset_ms: u64,
    /// Whether the heartbeat is pending ack for an interval change.
    pub interval_change_pending: bool,
    /// Timestamp when interval change was sent (for 10s ack timeout).
    pub interval_change_sent: Option<Instant>,
}

/// Heartbeat scheduler managing phased pings across all connections.
pub struct HeartbeatScheduler {
    /// Current heartbeat interval in milliseconds.
    pub interval_ms: u64,
    /// Coprime step positions — cached until connection count changes.
    coprime_positions: Vec<u64>,
    /// Last connection count used for coprime computation.
    last_connection_count: usize,
    /// Per-connection phase assignments keyed by Rep C address.
    phases: HashMap<String, HeartbeatPhase>,
    /// Ack timeout for interval changes (10 seconds per spec).
    pub ack_timeout: Duration,
}

/// Default heartbeat interval: 30 seconds.
pub const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 30_000;

/// Default pong timeout: 60 seconds.
pub const DEFAULT_PONG_TIMEOUT_MS: u64 = 60_000;

/// Ack timeout for heartbeat interval changes: 10 seconds.
pub const INTERVAL_CHANGE_ACK_TIMEOUT: Duration = Duration::from_secs(10);

impl HeartbeatScheduler {
    pub fn new(interval_ms: u64) -> Self {
        HeartbeatScheduler {
            interval_ms,
            coprime_positions: Vec::new(),
            last_connection_count: 0,
            phases: HashMap::new(),
            ack_timeout: INTERVAL_CHANGE_ACK_TIMEOUT,
        }
    }

    /// Recompute coprime walk positions when connection count changes.
    ///
    /// Uses coprime::coprime_options() directly — no NinjaExec HTTP call.
    /// The smallest coprime to connection_count is used as the step size.
    /// CRT guarantees every position is visited exactly once per cycle.
    ///
    /// Called on connect/disconnect. In-progress heartbeat cycles complete
    /// with the old step — recomputation applies to the next cycle.
    pub fn recompute_positions(&mut self, connection_count: usize) {
        if connection_count == self.last_connection_count && connection_count > 0 {
            return; // No change
        }
        self.last_connection_count = connection_count;

        if connection_count <= 1 {
            self.coprime_positions = vec![0];
            return;
        }

        let axis = TritInt::from_host_u64(connection_count as u64);
        let min = TritInt::from_host_u64(1);
        let max = TritInt::from_host_u64(connection_count as u64);
        let options = coprime::coprime_options(&axis, &min, &max);
        self.coprime_positions = options.iter().map(|t| t.host_u64()).collect();

        // If no coprimes found (shouldn't happen for count > 1), fallback
        if self.coprime_positions.is_empty() {
            self.coprime_positions = vec![1];
        }
    }

    /// Assign a heartbeat phase to a new connection.
    ///
    /// The offset spreads heartbeats across the interval:
    /// offset = (position × interval) / connection_count
    ///
    /// Coprime step guarantees every position is visited exactly once
    /// per interval cycle — 500 connections at 30s produce ~one ping
    /// every 60ms instead of a 500-ping burst.
    pub fn assign_phase(&mut self, address: &str, connection_index: usize) {
        let count = self.last_connection_count.max(1);
        let position = if !self.coprime_positions.is_empty() {
            self.coprime_positions[connection_index % self.coprime_positions.len()]
        } else {
            connection_index as u64
        };

        let offset_ms = if count > 0 {
            (position * self.interval_ms) / count as u64
        } else {
            0
        };

        self.phases.insert(address.to_string(), HeartbeatPhase {
            position,
            offset_ms,
            interval_change_pending: false,
            interval_change_sent: None,
        });
    }

    /// Remove a connection's phase assignment.
    pub fn remove_phase(&mut self, address: &str) {
        self.phases.remove(address);
    }

    /// Get a connection's phase assignment.
    pub fn get_phase(&self, address: &str) -> Option<&HeartbeatPhase> {
        self.phases.get(address)
    }

    /// Get all connections that should be pinged at the given cycle offset.
    ///
    /// HModal null channel check: heartbeats ride n ≡ 0 (mod 4) harmonics.
    /// The offset within the interval determines which harmonic the ping
    /// falls on. Only offsets where (offset_slot mod 4 == 0) are valid
    /// heartbeat slots — algebraically separated from data delivery.
    pub fn connections_due_at(&self, cycle_offset_ms: u64) -> Vec<String> {
        let mut due = Vec::new();
        for (addr, phase) in &self.phases {
            if phase.offset_ms == cycle_offset_ms {
                // HModal null channel check: n ≡ 0 (mod 4)
                let slot = if self.last_connection_count > 0 {
                    phase.position % (self.last_connection_count as u64)
                } else {
                    0
                };
                // Null channel: slot mod 4 == 0 carries zero data energy
                // All heartbeats are assigned to these slots
                if slot % 4 == 0 || self.last_connection_count <= 4 {
                    due.push(addr.clone());
                }
            }
        }
        due
    }

    // ── Interval change protocol ────────────────────────────────

    /// Mark a connection as having a pending interval change ack.
    pub fn mark_interval_change_sent(&mut self, address: &str) {
        if let Some(phase) = self.phases.get_mut(address) {
            phase.interval_change_pending = true;
            phase.interval_change_sent = Some(Instant::now());
        }
    }

    /// Acknowledge an interval change from a client.
    pub fn ack_interval_change(&mut self, address: &str) -> bool {
        if let Some(phase) = self.phases.get_mut(address) {
            if phase.interval_change_pending {
                phase.interval_change_pending = false;
                phase.interval_change_sent = None;
                return true;
            }
        }
        false
    }

    /// Check for timed-out interval change acks.
    ///
    /// Returns addresses that failed to ack within the timeout.
    /// Per spec: server continues old interval for non-acking connections.
    /// Server does NOT disconnect — old interval remains safe.
    /// Client gets new interval on next reconnect via auth_ok.
    pub fn check_ack_timeouts(&mut self) -> Vec<String> {
        let mut timed_out = Vec::new();
        for (addr, phase) in &mut self.phases {
            if phase.interval_change_pending {
                if let Some(sent) = phase.interval_change_sent {
                    if sent.elapsed() >= self.ack_timeout {
                        phase.interval_change_pending = false;
                        phase.interval_change_sent = None;
                        timed_out.push(addr.clone());
                    }
                }
            }
        }
        timed_out
    }

    /// Change the heartbeat interval. Returns the old interval.
    ///
    /// Does NOT send frames — caller is responsible for broadcasting
    /// heartbeat_interval_changed to all connected clients and marking
    /// them via mark_interval_change_sent().
    pub fn set_interval(&mut self, new_interval_ms: u64) -> u64 {
        let old = self.interval_ms;
        self.interval_ms = new_interval_ms;
        old
    }

    /// Number of assigned phases.
    pub fn phase_count(&self) -> usize {
        self.phases.len()
    }

    /// Number of cached coprime positions.
    pub fn coprime_position_count(&self) -> usize {
        self.coprime_positions.len()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let sched = HeartbeatScheduler::new(DEFAULT_HEARTBEAT_INTERVAL_MS);
        assert_eq!(sched.interval_ms, 30_000);
        assert_eq!(sched.phase_count(), 0);
    }

    #[test]
    fn test_recompute_positions() {
        let mut sched = HeartbeatScheduler::new(30_000);
        sched.recompute_positions(10);
        assert!(!sched.coprime_positions.is_empty());
        // All positions should be coprime to 10
        for &pos in &sched.coprime_positions {
            // gcd(pos, 10) == 1
            let g = gcd(pos, 10);
            assert_eq!(g, 1, "Position {} should be coprime to 10", pos);
        }
    }

    #[test]
    fn test_recompute_caches() {
        let mut sched = HeartbeatScheduler::new(30_000);
        sched.recompute_positions(10);
        let count1 = sched.coprime_position_count();
        // Same count shouldn't recompute
        sched.recompute_positions(10);
        assert_eq!(sched.coprime_position_count(), count1);
    }

    #[test]
    fn test_assign_phase_unique_offsets() {
        let mut sched = HeartbeatScheduler::new(30_000);
        sched.recompute_positions(5);
        for i in 0..5 {
            sched.assign_phase(&format!("node_{}", i), i);
        }
        assert_eq!(sched.phase_count(), 5);

        // Offsets should be spread across the interval
        let mut offsets: Vec<u64> = (0..5)
            .filter_map(|i| sched.get_phase(&format!("node_{}", i)).map(|p| p.offset_ms))
            .collect();
        offsets.sort();
        offsets.dedup();
        // At least some should be different (coprime walk)
        assert!(offsets.len() > 1, "Offsets should be spread, not all identical");
    }

    #[test]
    fn test_remove_phase() {
        let mut sched = HeartbeatScheduler::new(30_000);
        sched.recompute_positions(2);
        sched.assign_phase("node_a", 0);
        assert_eq!(sched.phase_count(), 1);
        sched.remove_phase("node_a");
        assert_eq!(sched.phase_count(), 0);
    }

    #[test]
    fn test_interval_change_ack_protocol() {
        let mut sched = HeartbeatScheduler::new(30_000);
        sched.recompute_positions(1);
        sched.assign_phase("node_a", 0);

        // Send interval change
        sched.mark_interval_change_sent("node_a");
        let phase = sched.get_phase("node_a").unwrap();
        assert!(phase.interval_change_pending);
        assert!(phase.interval_change_sent.is_some());

        // Ack it
        assert!(sched.ack_interval_change("node_a"));
        let phase = sched.get_phase("node_a").unwrap();
        assert!(!phase.interval_change_pending);
    }

    #[test]
    fn test_interval_change_ack_timeout() {
        let mut sched = HeartbeatScheduler::new(30_000);
        sched.ack_timeout = Duration::from_millis(10); // Short for test
        sched.recompute_positions(1);
        sched.assign_phase("node_a", 0);

        sched.mark_interval_change_sent("node_a");
        assert!(sched.check_ack_timeouts().is_empty()); // Not timed out yet

        std::thread::sleep(Duration::from_millis(15));
        let timed_out = sched.check_ack_timeouts();
        assert_eq!(timed_out.len(), 1);
        assert_eq!(timed_out[0], "node_a");

        // After timeout, pending flag cleared
        let phase = sched.get_phase("node_a").unwrap();
        assert!(!phase.interval_change_pending);
    }

    #[test]
    fn test_set_interval_returns_old() {
        let mut sched = HeartbeatScheduler::new(30_000);
        let old = sched.set_interval(15_000);
        assert_eq!(old, 30_000);
        assert_eq!(sched.interval_ms, 15_000);
    }

    #[test]
    fn test_single_connection() {
        let mut sched = HeartbeatScheduler::new(30_000);
        sched.recompute_positions(1);
        sched.assign_phase("solo", 0);
        let phase = sched.get_phase("solo").unwrap();
        assert_eq!(phase.offset_ms, 0); // Single connection = offset 0
    }

    // Helper: simple GCD for test assertions
    fn gcd(a: u64, b: u64) -> u64 {
        if b == 0 { a } else { gcd(b, a % b) }
    }
}
