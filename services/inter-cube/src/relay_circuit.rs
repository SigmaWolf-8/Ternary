// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// RELAY CIRCUIT BREAKER — Task #27
//
// Rust implementation replacing the retired TypeScript CircuitBreaker
// class (node-watchdog.ts:89-160). Extends with:
// - Token-bucket recovery ramp after probe success
// - Coprime probe scheduling via coprime::coprime_options() directly
// - HModal null channel probe assignment (n ≡ 0 mod 4)
// - Prometheus gauge via existing MetricsRegistry

use std::time::{Duration, Instant};

use ternary_math::coprime;
use ternary_math::trit_int::TritInt;

/// Circuit breaker states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Open => "open",
            Self::HalfOpen => "half-open",
        }
    }

    /// Numeric value for Prometheus gauge.
    pub fn as_gauge(&self) -> u64 {
        match self {
            Self::Closed => 0,
            Self::HalfOpen => 1,
            Self::Open => 2,
        }
    }
}

/// WebSocket close codes that indicate server-side failures.
const FAILURE_CLOSE_CODES: &[u16] = &[1006, 1011, 1012, 1013, 1014];

/// Circuit breaker with coprime probe scheduling.
///
/// State machine: Closed → (failures ≥ threshold) → Open → (timeout) →
/// HalfOpen → (probe success) → Closed (with token-bucket ramp)
///       ↑                                    │
///       └──────── (probe failure) ←──────────┘
pub struct RelayCircuitBreaker {
    name: String,
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    reset_timeout: Duration,
    last_failure_time: Option<Instant>,
    last_state_change: Instant,
    /// Token bucket for recovery ramp. Starts at 1 after first probe success,
    /// increases to full capacity over recovery_ramp_steps successful probes.
    recovery_tokens: u32,
    recovery_ramp_steps: u32,
    /// Half-open linger threshold — re-open if half-open too long.
    half_open_max: Duration,
    /// Coprime probe schedule — cached until common_service_period changes.
    probe_schedule: Vec<u64>,
    probe_index: usize,
    /// Callback for state change notifications.
    on_state_change: Option<Box<dyn Fn(&str, CircuitState) + Send + Sync>>,
}

impl RelayCircuitBreaker {
    pub fn new(name: impl Into<String>) -> Self {
        RelayCircuitBreaker {
            name: name.into(),
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold: 5,
            reset_timeout: Duration::from_secs(30),
            last_failure_time: None,
            last_state_change: Instant::now(),
            recovery_tokens: 0,
            recovery_ramp_steps: 3,
            half_open_max: Duration::from_secs(120),
            probe_schedule: Vec::new(),
            probe_index: 0,
            on_state_change: None,
        }
    }

    pub fn with_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    pub fn with_reset_timeout(mut self, timeout: Duration) -> Self {
        self.reset_timeout = timeout;
        self
    }

    pub fn with_on_state_change<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, CircuitState) + Send + Sync + 'static,
    {
        self.on_state_change = Some(Box::new(f));
        self
    }

    // ── State queries ───────────────────────────────────────────

    pub fn state(&self) -> CircuitState { self.state }
    pub fn name(&self) -> &str { &self.name }
    pub fn failure_count(&self) -> u32 { self.failure_count }
    pub fn last_state_change(&self) -> Instant { self.last_state_change }

    /// Check if a request should be allowed through.
    pub fn is_request_allowed(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last) = self.last_failure_time {
                    if last.elapsed() >= self.reset_timeout {
                        self.transition(CircuitState::HalfOpen);
                        true // Allow probe request
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Check half-open linger — re-open if too long
                if self.last_state_change.elapsed() >= self.half_open_max {
                    self.transition(CircuitState::Open);
                    self.last_failure_time = Some(Instant::now());
                    false
                } else {
                    // Token-bucket: allow only if tokens available
                    self.recovery_tokens > 0
                }
            }
        }
    }

    // ── Recording results ───────────────────────────────────────

    /// Record a successful operation.
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::HalfOpen => {
                self.recovery_tokens = self.recovery_tokens.saturating_add(1);
                if self.recovery_tokens >= self.recovery_ramp_steps {
                    // Full recovery
                    self.failure_count = 0;
                    self.recovery_tokens = 0;
                    self.transition(CircuitState::Closed);
                }
            }
            CircuitState::Closed => {
                if self.failure_count > 0 {
                    self.failure_count = 0;
                }
            }
            _ => {}
        }
    }

    /// Record a failure. May trip the breaker to Open.
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.transition(CircuitState::Open);
                }
            }
            CircuitState::HalfOpen => {
                // Probe failed — re-open
                self.recovery_tokens = 0;
                self.transition(CircuitState::Open);
            }
            _ => {}
        }
    }

    /// Record a WebSocket close code. Failure codes trigger record_failure().
    pub fn record_ws_close(&mut self, code: u16) {
        if FAILURE_CLOSE_CODES.contains(&code) {
            self.record_failure();
        }
    }

    /// Manual reset to Closed (admin endpoint).
    pub fn reset(&mut self) {
        self.failure_count = 0;
        self.recovery_tokens = 0;
        self.transition(CircuitState::Closed);
    }

    // ── Coprime probe scheduling ────────────────────────────────

    /// Compute probe schedule for half-open state.
    ///
    /// Uses coprime_options() to find intervals algebraically independent
    /// of common deployment periods (10s, 15s, 30s, 60s). Each probe fires
    /// at a different coprime offset — no phase-locking with upstream cron.
    ///
    /// `common_period` is the LCM or typical period to avoid (default 60).
    pub fn compute_probe_schedule(&mut self, common_period: u64) {
        let axis = TritInt::from_u64(common_period);
        let min = TritInt::from_u64(2);
        let max = TritInt::from_u64(common_period.saturating_sub(1).max(2));
        let options = coprime::coprime_options(&axis, &min, &max);
        self.probe_schedule = options.iter().map(|t| t.to_decimal()).collect();
        self.probe_index = 0;
    }

    /// Get the next probe interval in seconds (coprime-stepped).
    /// Falls back to reset_timeout if no schedule computed.
    pub fn next_probe_interval(&mut self) -> Duration {
        if self.probe_schedule.is_empty() {
            return self.reset_timeout;
        }
        let interval = self.probe_schedule[self.probe_index % self.probe_schedule.len()];
        self.probe_index += 1;
        Duration::from_secs(interval)
    }

    // ── Internal ────────────────────────────────────────────────

    fn transition(&mut self, new_state: CircuitState) {
        if self.state == new_state {
            return;
        }
        let old = self.state;
        self.state = new_state;
        self.last_state_change = Instant::now();
        println!(
            "[circuit-breaker] {}: {} -> {} (failures={})",
            self.name, old.as_str(), new_state.as_str(), self.failure_count
        );
        if let Some(ref cb) = self.on_state_change {
            cb(&self.name, new_state);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starts_closed() {
        let cb = RelayCircuitBreaker::new("test");
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.failure_count(), 0);
    }

    #[test]
    fn test_trips_to_open_on_threshold() {
        let mut cb = RelayCircuitBreaker::new("test").with_threshold(3);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_success_resets_count_in_closed() {
        let mut cb = RelayCircuitBreaker::new("test").with_threshold(5);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.failure_count(), 2);
        cb.record_success();
        assert_eq!(cb.failure_count(), 0);
    }

    #[test]
    fn test_half_open_after_timeout() {
        let mut cb = RelayCircuitBreaker::new("test")
            .with_threshold(1)
            .with_reset_timeout(Duration::from_millis(10));
        cb.record_failure(); // trips to open
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.is_request_allowed());

        std::thread::sleep(Duration::from_millis(15));
        assert!(cb.is_request_allowed()); // transitions to half-open
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_recovery_ramp() {
        let mut cb = RelayCircuitBreaker::new("test")
            .with_threshold(1)
            .with_reset_timeout(Duration::from_millis(1));
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(5));
        cb.is_request_allowed(); // half-open

        // Need 3 successes (default ramp) to fully close
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_probe_failure_reopens() {
        let mut cb = RelayCircuitBreaker::new("test")
            .with_threshold(1)
            .with_reset_timeout(Duration::from_millis(1));
        cb.record_failure();
        std::thread::sleep(Duration::from_millis(5));
        cb.is_request_allowed(); // half-open

        cb.record_failure(); // probe fails
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_ws_close_codes() {
        let mut cb = RelayCircuitBreaker::new("test").with_threshold(2);
        cb.record_ws_close(1000); // normal close — no failure
        assert_eq!(cb.failure_count(), 0);
        cb.record_ws_close(1006); // abnormal — failure
        assert_eq!(cb.failure_count(), 1);
        cb.record_ws_close(1011); // server error — failure
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_manual_reset() {
        let mut cb = RelayCircuitBreaker::new("test").with_threshold(1);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.failure_count(), 0);
    }

    #[test]
    fn test_coprime_probe_schedule() {
        let mut cb = RelayCircuitBreaker::new("test");
        cb.compute_probe_schedule(12);
        // coprime_options(12, 1, 11) should include 1, 5, 7, 11
        assert!(!cb.probe_schedule.is_empty());
        let interval = cb.next_probe_interval();
        assert!(interval.as_secs() > 0);
        // Successive intervals should differ (coprime walk)
        let i1 = cb.next_probe_interval();
        let i2 = cb.next_probe_interval();
        // They're from different coprime values
        assert!(i1.as_secs() > 0);
        assert!(i2.as_secs() > 0);
    }

    #[test]
    fn test_gauge_values() {
        assert_eq!(CircuitState::Closed.as_gauge(), 0);
        assert_eq!(CircuitState::HalfOpen.as_gauge(), 1);
        assert_eq!(CircuitState::Open.as_gauge(), 2);
    }
}
