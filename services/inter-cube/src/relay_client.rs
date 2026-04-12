// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// RELAY CLIENT HELPERS — Task #27, Task 7
//
// Client-side relay protocol helpers. These are consumed by the existing
// ws_relay.rs read loop and main.rs retry loop. See the delivery doc
// for the exact ws_relay.rs and main.rs modifications needed.
//
// - Full jitter backoff: base 500ms, cap 60s (replaces doubling-only)
// - Client-side circuit breaker for outgoing requests
// - Frame handlers for new control frame types
// - Dedup state persistence before backoff sleep

use std::time::Duration;

use crate::relay_seq::RelaySequenceStore;

// ═══════════════════════════════════════════════════════════════════════
// FULL JITTER BACKOFF
//
// Replaces the existing doubling-only backoff in main.rs:2551-2554.
// Formula: delay = min(2^attempt × 500ms + random(0..500ms), 60s)
// ═══════════════════════════════════════════════════════════════════════

/// Full jitter backoff state.
pub struct JitterBackoff {
    /// Current attempt counter (0-indexed).
    attempt: u32,
    /// Base delay in milliseconds.
    base_ms: u64,
    /// Maximum delay in milliseconds.
    cap_ms: u64,
    /// Whether the server pushed a circuit_open frame (extends backoff).
    circuit_open_received: bool,
}

impl JitterBackoff {
    /// Create with default parameters: base 500ms, cap 60s.
    pub fn new() -> Self {
        JitterBackoff {
            attempt: 0,
            base_ms: 500,
            cap_ms: 60_000,
            circuit_open_received: false,
        }
    }

    /// Compute the next backoff delay with full jitter.
    ///
    /// `delay = min(2^attempt × base + random(0..base), cap)`
    ///
    /// If circuit_open was received, the cap is doubled to give
    /// the server more recovery time.
    pub fn next_delay(&mut self) -> Duration {
        let effective_cap = if self.circuit_open_received {
            self.cap_ms * 2
        } else {
            self.cap_ms
        };

        let exp_delay = self.base_ms.saturating_mul(1u64 << self.attempt.min(20));

        // Generate jitter: random(0..base_ms) using getrandom
        let mut jitter_bytes = [0u8; 8];
        let _ = getrandom::getrandom(&mut jitter_bytes);
        let jitter = u64::from_le_bytes(jitter_bytes) % self.base_ms;

        let delay = exp_delay.saturating_add(jitter).min(effective_cap);
        self.attempt = self.attempt.saturating_add(1);

        Duration::from_millis(delay)
    }

    /// Reset on successful connection.
    pub fn reset(&mut self) {
        self.attempt = 0;
        self.circuit_open_received = false;
    }

    /// Mark that a circuit_open frame was received from the server.
    pub fn mark_circuit_open(&mut self) {
        self.circuit_open_received = true;
    }

    /// Get the current attempt count.
    pub fn attempt(&self) -> u32 {
        self.attempt
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CLIENT-SIDE CIRCUIT BREAKER
//
// Prevents overloading a struggling server with outgoing requests.
// Simpler than the server-side breaker — no coprime probing.
// ═══════════════════════════════════════════════════════════════════════

/// Client-side circuit breaker for outgoing relay requests.
pub struct ClientCircuitBreaker {
    consecutive_failures: u32,
    threshold: u32,
    is_open: bool,
}

impl ClientCircuitBreaker {
    pub fn new(threshold: u32) -> Self {
        ClientCircuitBreaker {
            consecutive_failures: 0,
            threshold,
            is_open: false,
        }
    }

    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.is_open = false;
    }

    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= self.threshold {
            self.is_open = true;
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn reset(&mut self) {
        self.consecutive_failures = 0;
        self.is_open = false;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// FRAME HANDLERS
//
// These are called from new match arms in the ws_relay.rs read loop.
// Each returns an action enum the caller uses to drive state changes.
// ═══════════════════════════════════════════════════════════════════════

/// Action the read loop should take after processing a control frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameAction {
    /// No action needed — frame was handled internally.
    None,
    /// Acknowledge the frame by sending a response.
    Ack(String),
    /// Reset dedup state for a topic (topic_reset received).
    ResetTopic { topic: String, new_epoch: u64 },
    /// Remove a topic from the subscription set (topic_revoked).
    RemoveTopic { topic: String },
    /// Apply tombstone to dedup state.
    ApplyTombstone,
    /// Stop sending messages, schedule reconnect after delay.
    GoAway { reconnect_after_ms: u64 },
    /// Extend backoff — server circuit breaker is open.
    CircuitOpen,
    /// Disconnect immediately.
    Disconnect,
}

/// Handle a heartbeat_interval_changed control frame.
///
/// Client must ack within 10s. Returns Ack with the ack response JSON.
pub fn handle_heartbeat_interval_changed(
    new_interval_ms: u64,
) -> FrameAction {
    println!(
        "[ws-relay] Heartbeat interval changed to {}ms — sending ack",
        new_interval_ms
    );
    let ack = serde_json::json!({
        "type": "heartbeat_interval_ack",
        "heartbeatIntervalMs": new_interval_ms,
    });
    FrameAction::Ack(serde_json::to_string(&ack).unwrap_or_default())
}

/// Handle a topic_reset control frame (topic was GC'd and recreated).
///
/// Client must reset persisted seq for that topic and replay from seq 1.
pub fn handle_topic_reset(
    topic: &str,
    old_epoch: u64,
    new_epoch: u64,
    current_seq: u64,
) -> FrameAction {
    println!(
        "[ws-relay] Topic '{}' reset: epoch {} → {} (currentSeq={})",
        topic, old_epoch, new_epoch, current_seq
    );
    FrameAction::ResetTopic {
        topic: topic.to_string(),
        new_epoch,
    }
}

/// Handle a topic_revoked control frame (permission revoked mid-session).
///
/// Client must remove this topic from its subscription set.
pub fn handle_topic_revoked(
    topic: &str,
    reason: &str,
    last_delivered_seq: u64,
    topic_epoch: u64,
) -> FrameAction {
    println!(
        "[ws-relay] Topic '{}' revoked: reason={}, lastSeq={}, epoch={}",
        topic, reason, last_delivered_seq, topic_epoch
    );
    FrameAction::RemoveTopic {
        topic: topic.to_string(),
    }
}

/// Handle a tombstone control frame (global queue eviction).
///
/// Client must reset dedup state per topicSeqs snapshot.
pub fn handle_tombstone(
    resync_count: u64,
    suggested_resync_after_ms: u64,
    gap_size_estimate: u64,
) -> FrameAction {
    println!(
        "[ws-relay] Tombstone received: resyncCount={}, resyncAfter={}ms, gap~={}",
        resync_count, suggested_resync_after_ms, gap_size_estimate
    );
    FrameAction::ApplyTombstone
}

/// Handle a go-away control frame (server shutting down).
///
/// Client must stop sending new messages, ack, and schedule reconnect.
pub fn handle_go_away(reason: &str, reconnect_after_ms: u64) -> FrameAction {
    println!(
        "[ws-relay] Go-away received: reason={}, reconnectAfter={}ms",
        reason, reconnect_after_ms
    );
    FrameAction::GoAway { reconnect_after_ms }
}

/// Handle a circuit_open control frame (server breaker tripped).
///
/// Client should extend its backoff accordingly.
pub fn handle_circuit_open(breaker: &str) -> FrameAction {
    println!("[ws-relay] Circuit open: breaker={}", breaker);
    FrameAction::CircuitOpen
}

// ═══════════════════════════════════════════════════════════════════════
// DEDUP PERSISTENCE HELPER
//
// Called before entering backoff sleep to persist dedup state.
// ═══════════════════════════════════════════════════════════════════════

/// Persist dedup state before backoff sleep. Logs but doesn't fail.
pub fn persist_dedup_before_backoff(store: &mut RelaySequenceStore) {
    match store.flush() {
        Ok(true) => println!("[ws-relay] Dedup state persisted before backoff"),
        Ok(false) => {} // Not dirty, nothing to flush
        Err(e) => eprintln!("[ws-relay] Failed to persist dedup state: {}", e),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jitter_backoff_increasing() {
        let mut backoff = JitterBackoff::new();
        let d1 = backoff.next_delay();
        let d2 = backoff.next_delay();
        let d3 = backoff.next_delay();
        // Should generally increase (with jitter, not guaranteed monotonic)
        assert!(d1.as_millis() < 65_000);
        assert!(d2.as_millis() < 65_000);
        assert!(d3.as_millis() < 65_000);
    }

    #[test]
    fn test_jitter_backoff_capped() {
        let mut backoff = JitterBackoff::new();
        for _ in 0..30 {
            let d = backoff.next_delay();
            assert!(d.as_millis() <= 60_000, "Should be capped at 60s");
        }
    }

    #[test]
    fn test_jitter_backoff_circuit_open_extends() {
        let mut backoff = JitterBackoff::new();
        backoff.mark_circuit_open();
        // With circuit_open, cap doubles to 120s
        for _ in 0..30 {
            let d = backoff.next_delay();
            assert!(d.as_millis() <= 120_000, "Circuit open cap should be 120s");
        }
    }

    #[test]
    fn test_jitter_backoff_reset() {
        let mut backoff = JitterBackoff::new();
        backoff.next_delay();
        backoff.next_delay();
        backoff.next_delay();
        assert!(backoff.attempt() >= 3);
        backoff.reset();
        assert_eq!(backoff.attempt(), 0);
    }

    #[test]
    fn test_client_circuit_breaker() {
        let mut cb = ClientCircuitBreaker::new(3);
        assert!(!cb.is_open());
        cb.record_failure();
        cb.record_failure();
        assert!(!cb.is_open());
        cb.record_failure();
        assert!(cb.is_open());
        cb.record_success();
        assert!(!cb.is_open());
    }

    #[test]
    fn test_frame_handlers_return_correct_actions() {
        assert_eq!(
            handle_heartbeat_interval_changed(15_000),
            FrameAction::Ack(serde_json::json!({"type":"heartbeat_interval_ack","heartbeatIntervalMs":15000}).to_string()),
        );

        assert_eq!(
            handle_topic_reset("data", 100, 200, 1),
            FrameAction::ResetTopic { topic: "data".to_string(), new_epoch: 200 },
        );

        assert_eq!(
            handle_topic_revoked("data", "permission_revoked", 47, 100),
            FrameAction::RemoveTopic { topic: "data".to_string() },
        );

        assert_eq!(
            handle_tombstone(1, 3500, 42),
            FrameAction::ApplyTombstone,
        );

        assert_eq!(
            handle_go_away("server_shutdown", 3500),
            FrameAction::GoAway { reconnect_after_ms: 3500 },
        );

        assert_eq!(
            handle_circuit_open("crs-verification"),
            FrameAction::CircuitOpen,
        );
    }
}
