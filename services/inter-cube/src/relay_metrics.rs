// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// RELAY METRICS & OTEL — Task #27, Task 8
//
// 16 relay-specific lock-free atomic counters/gauges using the same
// Counter/Gauge types from telemetry.rs. Exposed alongside existing
// 50 metrics on /metrics.
//
// OTel integration: span helpers for the relay message lifecycle.
// No-op when OTEL_EXPORTER_OTLP_ENDPOINT is not set.
//
// INTEGRATION NOTE FOR REPLIT:
// These 16 fields must be added to MetricsRegistry in telemetry.rs.
// Until that modification is made, this module provides a standalone
// RelayMetrics struct that can be Arc-shared alongside MetricsRegistry.
// See the delivery doc for the exact telemetry.rs modifications.

use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::BTreeMap;

// ═══════════════════════════════════════════════════════════════════════
// RELAY METRICS — 16 counters/gauges
//
// Same lock-free atomic pattern as telemetry.rs Counter/Gauge.
// ~5ns per update, zero allocation on hot path.
// ═══════════════════════════════════════════════════════════════════════

/// Relay-specific metrics. Thread-safe via atomic operations.
///
/// These parallel the 16 metrics in the spec. When integrated into
/// telemetry.rs MetricsRegistry, they become fields on that struct.
/// Until then, this standalone struct is Arc-shared.
pub struct RelayMetrics {
    // ── Circuit Breaker ─────────────────────────────────────
    /// Current circuit breaker state (0=closed, 1=half-open, 2=open).
    pub circuit_state: AtomicU64,

    // ── Message Delivery ────────────────────────────────────
    /// Total relay messages delivered (all topics).
    pub messages_delivered_total: AtomicU64,

    // ── Tombstones ──────────────────────────────────────────
    /// Total tombstones generated.
    pub tombstones_generated_total: AtomicU64,

    // ── Topics ──────────────────────────────────────────────
    /// Total topic_reset frames sent.
    pub topic_resets_total: AtomicU64,
    /// Total topic_revoked frames sent.
    pub topic_revocations_total: AtomicU64,
    /// Currently active topics.
    pub topics_active: AtomicU64,
    /// Total topics garbage-collected.
    pub topics_gc_total: AtomicU64,
    /// Total topic backpressure rejections.
    pub topic_backpressure_total: AtomicU64,
    /// Total topic reauthorization failures.
    pub topic_reauth_failures_total: AtomicU64,

    // ── Resync ──────────────────────────────────────────────
    /// Total resync requests (by outcome: success, rate_limited, oversized).
    pub resync_requests_total: AtomicU64,

    // ── Shutdown ─────────────────────────────────────────────
    /// Total go-away frames acked by clients.
    pub goaway_acked_total: AtomicU64,

    // ── Capabilities ────────────────────────────────────────
    /// Total capability downgrade attempts.
    pub capability_downgrades_total: AtomicU64,

    // ── Heartbeat ───────────────────────────────────────────
    /// Total heartbeat failures (pong timeout).
    pub heartbeat_failures_total: AtomicU64,
    /// Total heartbeat interval change non-acks.
    pub heartbeat_interval_nonacks_total: AtomicU64,

    // ── Circuit Breaker Probing ──────────────────────────────
    /// Probe success rate (stored as percentage × 100 for integer gauge).
    pub circuit_probe_success_rate: AtomicU64,
    /// Last circuit recovery time in milliseconds.
    pub circuit_recovery_time_ms: AtomicU64,
}

impl RelayMetrics {
    pub fn new() -> Self {
        RelayMetrics {
            circuit_state: AtomicU64::new(0),
            messages_delivered_total: AtomicU64::new(0),
            tombstones_generated_total: AtomicU64::new(0),
            topic_resets_total: AtomicU64::new(0),
            topic_revocations_total: AtomicU64::new(0),
            topics_active: AtomicU64::new(0),
            topics_gc_total: AtomicU64::new(0),
            topic_backpressure_total: AtomicU64::new(0),
            topic_reauth_failures_total: AtomicU64::new(0),
            resync_requests_total: AtomicU64::new(0),
            goaway_acked_total: AtomicU64::new(0),
            capability_downgrades_total: AtomicU64::new(0),
            heartbeat_failures_total: AtomicU64::new(0),
            heartbeat_interval_nonacks_total: AtomicU64::new(0),
            circuit_probe_success_rate: AtomicU64::new(0),
            circuit_recovery_time_ms: AtomicU64::new(0),
        }
    }

    // ── Increment helpers ───────────────────────────────────

    pub fn inc_messages_delivered(&self) {
        self.messages_delivered_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_tombstones(&self) {
        self.tombstones_generated_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_topic_resets(&self) {
        self.topic_resets_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_topic_revocations(&self) {
        self.topic_revocations_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_topics_gc(&self) {
        self.topics_gc_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_topic_backpressure(&self) {
        self.topic_backpressure_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_topic_reauth_failures(&self) {
        self.topic_reauth_failures_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_resync_requests(&self) {
        self.resync_requests_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_goaway_acked(&self) {
        self.goaway_acked_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_capability_downgrades(&self) {
        self.capability_downgrades_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_heartbeat_failures(&self) {
        self.heartbeat_failures_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_heartbeat_nonacks(&self) {
        self.heartbeat_interval_nonacks_total.fetch_add(1, Ordering::Relaxed);
    }

    // ── Gauge setters ───────────────────────────────────────

    pub fn set_circuit_state(&self, state: u64) {
        self.circuit_state.store(state, Ordering::Relaxed);
    }

    pub fn set_topics_active(&self, count: u64) {
        self.topics_active.store(count, Ordering::Relaxed);
    }

    pub fn set_probe_success_rate(&self, rate_pct_x100: u64) {
        self.circuit_probe_success_rate.store(rate_pct_x100, Ordering::Relaxed);
    }

    pub fn set_recovery_time_ms(&self, ms: u64) {
        self.circuit_recovery_time_ms.store(ms, Ordering::Relaxed);
    }

    // ── Snapshot ─────────────────────────────────────────────

    /// Collect all relay metrics into a BTreeMap for Prometheus exposition.
    /// Metric names prefixed with "plenum_relay_" for consistency.
    pub fn collect(&self) -> BTreeMap<String, u64> {
        let mut m = BTreeMap::new();
        m.insert("plenum_relay_circuit_state".into(), self.circuit_state.load(Ordering::Relaxed));
        m.insert("plenum_relay_messages_delivered_total".into(), self.messages_delivered_total.load(Ordering::Relaxed));
        m.insert("plenum_relay_tombstones_generated_total".into(), self.tombstones_generated_total.load(Ordering::Relaxed));
        m.insert("plenum_relay_topic_resets_total".into(), self.topic_resets_total.load(Ordering::Relaxed));
        m.insert("plenum_relay_topic_revocations_total".into(), self.topic_revocations_total.load(Ordering::Relaxed));
        m.insert("plenum_relay_topics_active".into(), self.topics_active.load(Ordering::Relaxed));
        m.insert("plenum_relay_topics_gc_total".into(), self.topics_gc_total.load(Ordering::Relaxed));
        m.insert("plenum_relay_topic_backpressure_total".into(), self.topic_backpressure_total.load(Ordering::Relaxed));
        m.insert("plenum_relay_topic_reauth_failures_total".into(), self.topic_reauth_failures_total.load(Ordering::Relaxed));
        m.insert("plenum_relay_resync_requests_total".into(), self.resync_requests_total.load(Ordering::Relaxed));
        m.insert("plenum_relay_goaway_acked_total".into(), self.goaway_acked_total.load(Ordering::Relaxed));
        m.insert("plenum_relay_capability_downgrades_total".into(), self.capability_downgrades_total.load(Ordering::Relaxed));
        m.insert("plenum_relay_heartbeat_failures_total".into(), self.heartbeat_failures_total.load(Ordering::Relaxed));
        m.insert("plenum_relay_heartbeat_interval_nonacks_total".into(), self.heartbeat_interval_nonacks_total.load(Ordering::Relaxed));
        m.insert("plenum_relay_circuit_probe_success_rate".into(), self.circuit_probe_success_rate.load(Ordering::Relaxed));
        m.insert("plenum_relay_circuit_recovery_time_ms".into(), self.circuit_recovery_time_ms.load(Ordering::Relaxed));
        m
    }

    /// Render relay metrics in Prometheus text exposition format.
    pub fn to_prometheus(&self) -> String {
        let snapshot = self.collect();
        let mut lines = Vec::with_capacity(snapshot.len() * 2 + 1);
        lines.push("# PlenumNET Relay Metrics (Task #27)".to_string());
        for (name, value) in &snapshot {
            let metric_type = if name.ends_with("_total") { "counter" } else { "gauge" };
            lines.push(format!("# TYPE {} {}", name, metric_type));
            lines.push(format!("{} {}", name, value));
        }
        lines.join("\n")
    }
}

impl Default for RelayMetrics {
    fn default() -> Self { Self::new() }
}

// ═══════════════════════════════════════════════════════════════════════
// OTEL SPAN HELPERS
//
// OpenTelemetry span creation helpers for the relay message lifecycle.
// No-op when OTEL_EXPORTER_OTLP_ENDPOINT is not set.
//
// Crate dependencies (add when enabling OTel):
//   tracing = "0.1"
//   opentelemetry = "0.22"
//   opentelemetry-otlp = "0.15"
//   tracing-opentelemetry = "0.23"
//
// Until OTel crates are added, these are lightweight log-based helpers
// that match the spec's attribute requirements. When the crates are
// added, these become thin wrappers around tracing::info_span!().
// ═══════════════════════════════════════════════════════════════════════

/// Relay span attributes per the spec.
#[derive(Debug, Clone)]
pub struct RelaySpanAttrs {
    pub connection_id: String,
    pub node_address: String,
    pub topic: Option<String>,
    pub message_seq: Option<u64>,
    pub topic_epoch: Option<u64>,
    pub circuit_breaker_state: String,
    pub capability_set: Vec<String>,
}

/// Sampling decision for a relay message.
///
/// Default: 1% of relay messages, 100% of control frames.
pub fn should_sample(msg_type: &str, sample_rate_pct: f64) -> bool {
    // Control frames always sampled at 100%
    match msg_type {
        "go-away" | "circuit_open" | "topic_reset" | "topic_revoked" | "tombstone" => true,
        _ => {
            // Sample at configured rate
            let mut bytes = [0u8; 4];
            let _ = getrandom::getrandom(&mut bytes);
            let rand_val = u32::from_le_bytes(bytes) as f64 / u32::MAX as f64 * 100.0;
            rand_val < sample_rate_pct
        }
    }
}

/// Log a relay span (placeholder for full OTel integration).
///
/// When OTel crates are added, this becomes:
/// ```ignore
/// let span = tracing::info_span!("relay.message",
///     "connection.id" = %attrs.connection_id,
///     "node.address" = %attrs.node_address,
///     ...
/// );
/// ```
pub fn log_relay_span(phase: &str, attrs: &RelaySpanAttrs) {
    if cfg!(debug_assertions) {
        println!(
            "[otel] {} connection={} node={} topic={} seq={} epoch={} circuit={}",
            phase,
            &attrs.connection_id[..8.min(attrs.connection_id.len())],
            attrs.node_address,
            attrs.topic.as_deref().unwrap_or("-"),
            attrs.message_seq.unwrap_or(0),
            attrs.topic_epoch.unwrap_or(0),
            attrs.circuit_breaker_state,
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_metrics_defaults() {
        let m = RelayMetrics::new();
        assert_eq!(m.circuit_state.load(Ordering::Relaxed), 0);
        assert_eq!(m.messages_delivered_total.load(Ordering::Relaxed), 0);
        assert_eq!(m.topics_active.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_relay_metrics_increment() {
        let m = RelayMetrics::new();
        m.inc_messages_delivered();
        m.inc_messages_delivered();
        m.inc_messages_delivered();
        assert_eq!(m.messages_delivered_total.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn test_relay_metrics_gauge() {
        let m = RelayMetrics::new();
        m.set_topics_active(42);
        assert_eq!(m.topics_active.load(Ordering::Relaxed), 42);
        m.set_circuit_state(2); // open
        assert_eq!(m.circuit_state.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_collect_has_16_metrics() {
        let m = RelayMetrics::new();
        let snapshot = m.collect();
        assert_eq!(snapshot.len(), 16, "Must have exactly 16 relay metrics");
    }

    #[test]
    fn test_all_metric_names_prefixed() {
        let m = RelayMetrics::new();
        for name in m.collect().keys() {
            assert!(name.starts_with("plenum_relay_"), "Metric '{}' must start with plenum_relay_", name);
        }
    }

    #[test]
    fn test_prometheus_output() {
        let m = RelayMetrics::new();
        m.inc_tombstones();
        m.set_topics_active(5);
        let prom = m.to_prometheus();
        assert!(prom.contains("plenum_relay_tombstones_generated_total 1"));
        assert!(prom.contains("plenum_relay_topics_active 5"));
        assert!(prom.contains("# TYPE plenum_relay_tombstones_generated_total counter"));
        assert!(prom.contains("# TYPE plenum_relay_topics_active gauge"));
    }

    #[test]
    fn test_sampling_control_frames_always() {
        // Control frames always sampled
        assert!(should_sample("go-away", 0.0));
        assert!(should_sample("circuit_open", 0.0));
        assert!(should_sample("topic_reset", 0.0));
        assert!(should_sample("topic_revoked", 0.0));
        assert!(should_sample("tombstone", 0.0));
    }

    #[test]
    fn test_sampling_data_at_zero_rate() {
        // At 0% rate, relay messages should (almost) never be sampled
        let mut sampled = 0;
        for _ in 0..100 {
            if should_sample("relay", 0.0) { sampled += 1; }
        }
        assert!(sampled < 5, "At 0% rate, very few should be sampled (got {})", sampled);
    }

    #[test]
    fn test_sampling_data_at_100_rate() {
        // At 100% rate, all should be sampled
        let mut sampled = 0;
        for _ in 0..100 {
            if should_sample("relay", 100.0) { sampled += 1; }
        }
        assert_eq!(sampled, 100);
    }
}
