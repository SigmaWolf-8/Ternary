// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// RELAY SHUTDOWN — Task #27, Task 6
//
// Enhanced graceful shutdown:
// - Per-connection go-away with golden angle stagger
// - Per-connection ack timeout (2s)
// - Per-connection drain window (10s)
// - force_close flag for immediate termination
// - HTTP 503 draining health endpoint
// - Shutdown ordering: relay drain first → other subsystems → exit
// - Overall 30s timeout → process::exit(1)

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures_util::SinkExt;
use serde_json::json;

use crate::relay_server::{RelayState, broadcast_go_away};

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Go-away ack timeout per connection.
pub const GO_AWAY_ACK_TIMEOUT: Duration = Duration::from_secs(2);

/// Drain window per connection — max time to finish in-flight messages.
pub const DRAIN_WINDOW: Duration = Duration::from_secs(10);

/// Overall shutdown timeout. After this, process::exit(1).
pub const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

// ═══════════════════════════════════════════════════════════════════════
// DRAINING HEALTH RESPONSE
// ═══════════════════════════════════════════════════════════════════════

/// Build the draining health response body.
///
/// Returns HTTP 503 with:
/// ```json
/// {
///   "status": "draining",
///   "reason": "shutdown_in_progress",
///   "drain_deadline_ms": <unix_timestamp_ms>
/// }
/// ```
///
/// Liveness: 200 (process alive). Readiness: 503 (not accepting new connections).
pub fn draining_health_body(drain_deadline_ms: u64) -> serde_json::Value {
    json!({
        "status": "draining",
        "reason": "shutdown_in_progress",
        "drain_deadline_ms": drain_deadline_ms,
    })
}

// ═══════════════════════════════════════════════════════════════════════
// SHUTDOWN ORCHESTRATOR
// ═══════════════════════════════════════════════════════════════════════

/// Global draining flag — checked by health endpoints and new connection handler.
static DRAINING: AtomicBool = AtomicBool::new(false);

/// Check if the server is in draining state.
pub fn is_draining() -> bool {
    DRAINING.load(Ordering::Relaxed)
}

/// Initiate graceful shutdown sequence.
///
/// Shutdown ordering (from spec):
/// 1. Set draining flag (health endpoint returns 503)
/// 2. Broadcast go-away with golden angle stagger to all relay clients
/// 3. Wait for drain window (up to DRAIN_WINDOW for in-flight completion)
/// 4. Force-close remaining connections
/// 5. Signal other subsystems to shut down
/// 6. Overall timeout: 30s → process::exit(1)
///
/// `force_close`: if true, skip drain window — send close immediately.
pub async fn initiate_shutdown(
    relay_state: &RelayState,
    reason: &str,
    force_close: bool,
) {
    // Step 1: Set draining flag
    DRAINING.store(true, Ordering::Relaxed);
    println!("[relay-shutdown] Initiating shutdown: reason={}, force={}", reason, force_close);

    // Step 2: Broadcast go-away with golden angle stagger
    broadcast_go_away(relay_state, reason).await;

    if force_close {
        // Skip drain window — connections already closed by broadcast_go_away
        println!("[relay-shutdown] Force close — skipping drain window");
        return;
    }

    // Step 3: Wait for drain window
    println!("[relay-shutdown] Drain window: {}s", DRAIN_WINDOW.as_secs());
    let drain_start = std::time::Instant::now();

    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;

        let remaining = {
            let s = relay_state.read().await;
            s.clients.len()
        };

        if remaining == 0 {
            println!("[relay-shutdown] All connections drained");
            break;
        }

        if drain_start.elapsed() >= DRAIN_WINDOW {
            println!("[relay-shutdown] Drain window expired, {} connections remaining — force closing", remaining);
            // Force close remaining
            let s = relay_state.read().await;
            for (addr, conn) in s.clients.iter() {
                let _ = conn.sender.lock().await.close().await;
                println!("[relay-shutdown] Force-closed connection to {}", addr);
            }
            break;
        }
    }

    println!("[relay-shutdown] Relay drain complete");
}

/// Wire shutdown into SIGTERM/SIGINT handlers.
///
/// Call from main.rs after the axum server is created:
/// ```ignore
/// let relay_state_for_shutdown = relay_state.clone();
/// tokio::spawn(async move {
///     tokio::signal::ctrl_c().await.ok();
///     relay_shutdown::handle_signal(relay_state_for_shutdown).await;
/// });
/// ```
pub async fn handle_signal(relay_state: RelayState) {
    println!("[relay-shutdown] Signal received — initiating graceful shutdown");

    // Spawn timeout watchdog
    let timeout_handle = tokio::spawn(async {
        tokio::time::sleep(SHUTDOWN_TIMEOUT).await;
        eprintln!("[relay-shutdown] Shutdown timeout ({}s) exceeded — forcing exit", SHUTDOWN_TIMEOUT.as_secs());
        std::process::exit(1);
    });

    // Run shutdown sequence
    initiate_shutdown(&relay_state, "server_shutdown", false).await;

    // Step 5 placeholder: signal other subsystems
    // In production, this calls sfkOpsService.shutdown(), hederaService.shutdown(), etc.
    // These are wired in main.rs, not here.
    println!("[relay-shutdown] Relay shutdown complete — other subsystems can proceed");

    // Cancel timeout watchdog if we finished in time
    timeout_handle.abort();
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draining_health_body() {
        let body = draining_health_body(1712800030000);
        assert_eq!(body["status"], "draining");
        assert_eq!(body["reason"], "shutdown_in_progress");
        assert_eq!(body["drain_deadline_ms"], 1712800030000u64);
    }

    #[test]
    fn test_draining_flag() {
        // Reset for test isolation
        DRAINING.store(false, Ordering::Relaxed);
        assert!(!is_draining());
        DRAINING.store(true, Ordering::Relaxed);
        assert!(is_draining());
        // Reset
        DRAINING.store(false, Ordering::Relaxed);
    }

    #[test]
    fn test_constants() {
        assert_eq!(GO_AWAY_ACK_TIMEOUT, Duration::from_secs(2));
        assert_eq!(DRAIN_WINDOW, Duration::from_secs(10));
        assert_eq!(SHUTDOWN_TIMEOUT, Duration::from_secs(30));
    }
}
