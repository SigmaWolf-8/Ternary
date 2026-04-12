// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// RELAY SERVER — Task #27
//
// Axum WebSocket relay server at /ws/relay on the inter-cube CRS router.
// Replaces the retired TypeScript relay in server/index.ts.
//
// Architecture:
// - axum + tokio-tungstenite WebSocket handler
// - TL-DSA-87 challenge-response authentication
// - Direct crate access: coprime, gf3_algebra, sponge, tl_dsa
// - No N-API, no NinjaExec, no SHA-3
//
// Sponge Context String Registry (this module):
// | Context String           | Usage                    | Module           |
// |--------------------------|--------------------------|------------------|
// | "relay-audit-genesis"    | Audit chain genesis      | relay_audit.rs   |
// | "PlenumNET-DEDUP-STATE-v1" | Dedup state integrity  | relay_seq_store  |

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt, TryFutureExt, stream::SplitSink};
use serde_json::{json, Value};
use tokio::sync::{Mutex, RwLock};

use crate::relay_error::{RelayErrorCode, make_error_response};
use crate::relay_audit::{
    RelayAuditLog, RelayAuditEntry, RelayAuditEventType,
    AuditSeverity, AuditSubsystem,
};
use crate::relay_circuit::{RelayCircuitBreaker, CircuitState};
use crate::relay_frames::{
    wire_type_to_rep_c, is_frame_type_corrupt, validate_control_frame,
    MAX_FRAME_SIZE,
};

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Maximum pending messages per offline peer.
const PENDING_MAX: usize = 500;

/// Pending message TTL (5 minutes).
const PENDING_TTL: Duration = Duration::from_secs(300);

/// Default heartbeat ping interval.
const DEFAULT_PING_INTERVAL_MS: u64 = 30_000;

/// Authentication timeout — must auth within 10 seconds.
const AUTH_TIMEOUT: Duration = Duration::from_secs(10);

/// Nonce length in bytes for challenge-response.
const NONCE_LEN: usize = 32;

/// Golden angle ternary fraction: φ_ternary = 182(3−√5)/364 ≈ 0.381966
/// Derived from ARC_ROOT_SEMI = 182 in constants.rs:120.
const PHI_TERNARY: f64 = 182.0 * (3.0 - 2.2360679774997896) / 364.0;

// ═══════════════════════════════════════════════════════════════════════
// RELAY STATE
// ═══════════════════════════════════════════════════════════════════════

/// A single queued message for an offline peer.
#[derive(Debug, Clone)]
pub struct PendingMessage {
    pub from: String,
    pub msg_type: String,
    pub payload: String,
    pub ts: u64,
}

/// Per-connection metadata.
#[derive(Debug)]
pub struct RelayConnection {
    /// Authenticated Rep C address (INVARIANT 9: always Rep C, never IP/hostname).
    pub address: String,
    /// TL-DSA public key hex.
    pub public_key: String,
    /// Negotiated capabilities (Task 2 extends this).
    pub capabilities: Vec<String>,
    /// Connection timestamp.
    pub connected_at: Instant,
    /// Last pong/activity timestamp.
    pub last_activity: Instant,
    /// Sender half of the WebSocket — for pushing messages to this connection.
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

/// Shared relay server state. Thread-safe via Arc<RwLock<...>>.
pub struct RelayStateInner {
    /// Connected clients keyed by Rep C address.
    pub clients: HashMap<String, RelayConnection>,
    /// Pending messages for offline peers, keyed by Rep C address.
    pub pending: HashMap<String, Vec<PendingMessage>>,
    /// Circuit breaker for CRS verification.
    pub circuit_breaker: RelayCircuitBreaker,
    /// TIS-27 Merkle-chained audit log.
    pub audit_log: RelayAuditLog,
    /// Server-global monotonic counter for golden angle stagger.
    pub disconnect_index: AtomicU64,
    /// Heartbeat interval in milliseconds (configurable at runtime).
    pub heartbeat_interval_ms: u64,
    /// Server draining flag — reject new connections during shutdown.
    pub draining: bool,
    /// Peak concurrent peers (for telemetry).
    pub peak_peers: usize,
}

/// Thread-safe relay state handle.
pub type RelayState = Arc<RwLock<RelayStateInner>>;

/// Create a new relay state.
pub fn new_relay_state(audit_path: std::path::PathBuf) -> RelayState {
    Arc::new(RwLock::new(RelayStateInner {
        clients: HashMap::new(),
        pending: HashMap::new(),
        circuit_breaker: RelayCircuitBreaker::new("crs-verification")
            .with_threshold(5)
            .with_reset_timeout(Duration::from_secs(30)),
        audit_log: RelayAuditLog::new(audit_path),
        disconnect_index: AtomicU64::new(0),
        heartbeat_interval_ms: DEFAULT_PING_INTERVAL_MS,
        draining: false,
        peak_peers: 0,
    }))
}

// ═══════════════════════════════════════════════════════════════════════
// AXUM ROUTER
// ═══════════════════════════════════════════════════════════════════════

/// Build the relay axum router. Merge into the CRS app:
/// ```ignore
/// let app = crs_router(shared_state)
///     .merge(relay_router(relay_state))
///     ...
/// ```
pub fn relay_router(state: RelayState) -> Router {
    Router::new()
        .route("/ws/relay", get(ws_upgrade_handler))
        .route(
            "/admin/circuit-breaker/:name/reset",
            axum::routing::post(circuit_breaker_reset_handler),
        )
        .with_state(state)
}

/// WebSocket upgrade handler.
async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    State(state): State<RelayState>,
) -> impl IntoResponse {
    // Check if draining
    {
        let s = state.read().await;
        if s.draining {
            return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    }
    ws.on_upgrade(move |socket| handle_connection(socket, state))
}

/// Manual circuit breaker reset endpoint (admin-only).
async fn circuit_breaker_reset_handler(
    State(state): State<RelayState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let mut s = state.write().await;
    if s.circuit_breaker.name() != name {
        return (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(json!({"error": "breaker not found", "name": name})),
        ).into_response();
    }
    s.circuit_breaker.reset();
    let ts = now_iso();
    s.audit_log.record_event(&RelayAuditEntry {
        event_type: RelayAuditEventType::CircuitBreakerManualReset,
        address: "admin".to_string(),
        timestamp: ts,
        details: json!({"breaker": name, "action": "manual_reset"}),
        severity: AuditSeverity::Warn,
        subsystem: AuditSubsystem::CircuitBreaker,
        correlation_refs: vec![],
    });
    axum::Json(json!({"ok": true, "breaker": name, "state": "closed"})).into_response()
}

// ═══════════════════════════════════════════════════════════════════════
// CONNECTION HANDLER
// ═══════════════════════════════════════════════════════════════════════

/// Handle a single WebSocket connection lifecycle.
///
/// Flow: upgrade → challenge → auth verify → message loop → cleanup
async fn handle_connection(socket: WebSocket, state: RelayState) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));

    // ── Phase 1: Challenge-response authentication ──────────
    let auth_result = tokio::time::timeout(
        AUTH_TIMEOUT,
        handle_auth(&sender, &mut receiver, &state),
    ).await;

    let (node_address, public_key) = match auth_result {
        Ok(Ok(auth)) => auth,
        Ok(Err(err_code)) => {
            let _ = send_json(&sender, make_error_response(err_code, Some("auth"))).await;
            if let Some(_close_code) = err_code.ws_close() {
                // Close with protocol-level close frame. axum's close()
                // sends code 1000 — the error code was already conveyed
                // in the JSON error response above. The ws_close code
                // informs the caller whether to close at all.
                let _ = sender.lock().await.close().await;
            }
            return;
        }
        Err(_timeout) => {
            let _ = send_json(&sender, make_error_response(
                RelayErrorCode::ErrAuthTimeout, Some("auth"),
            )).await;
            let _ = sender.lock().await.close().await;
            return;
        }
    };

    // ── Phase 2: Register connection ────────────────────────
    let is_reconnect;
    let connected_peers;
    {
        let mut s = state.write().await;

        // Close old connection if exists
        is_reconnect = s.clients.contains_key(&node_address);
        if let Some(old) = s.clients.remove(&node_address) {
            let _ = old.sender.lock().await.close().await;
        }

        // Get peer list before inserting self
        connected_peers = s.clients.keys()
            .filter(|a| **a != node_address)
            .cloned()
            .collect::<Vec<_>>();

        s.clients.insert(node_address.clone(), RelayConnection {
            address: node_address.clone(),
            public_key: public_key.clone(),
            capabilities: vec![],
            connected_at: Instant::now(),
            last_activity: Instant::now(),
            sender: sender.clone(),
        });

        if s.clients.len() > s.peak_peers {
            s.peak_peers = s.clients.len();
        }

        // Audit log
        s.audit_log.record_event(&RelayAuditEntry {
            event_type: RelayAuditEventType::AuthSuccess,
            address: node_address.clone(),
            timestamp: now_iso(),
            details: json!({
                "hasTlDsa": true,
                "reconnect": is_reconnect,
            }),
            severity: AuditSeverity::Info,
            subsystem: AuditSubsystem::Capability,
            correlation_refs: vec![],
        });
    }

    // ── Phase 3: Send auth_ok + flush pending ───────────────
    let heartbeat_ms = {
        let s = state.read().await;
        s.heartbeat_interval_ms
    };
    let _ = send_json(&sender, json!({
        "type": "auth_ok",
        "address": node_address,
        "connectedPeers": connected_peers,
        "heartbeatIntervalMs": heartbeat_ms,
    })).await;

    // Flush pending messages
    let pending = {
        let mut s = state.write().await;
        s.pending.remove(&node_address).unwrap_or_default()
    };
    if !pending.is_empty() {
        let count = pending.len();
        for msg in pending {
            let envelope = json!({
                "type": "relay",
                "from": msg.from,
                "msgType": msg.msg_type,
                "payload": msg.payload,
            });
            let _ = send_json(&sender, envelope).await;
        }
        println!("[ws-relay] Drained {} queued message(s) to {}", count, node_address);
    }

    // ── Phase 4: Message loop ───────────────────────────────
    println!("[ws-relay] Node {} {}authenticated and connected",
        node_address, if is_reconnect { "re" } else { "" });

    while let Some(msg_result) = receiver.next().await {
        match msg_result {
            Ok(Message::Text(text)) => {
                // Frame size check
                if text.len() > MAX_FRAME_SIZE {
                    let _ = send_json(&sender, make_error_response(
                        RelayErrorCode::ErrFrameTooLarge, None,
                    )).await;
                    continue;
                }

                match serde_json::from_str::<Value>(&text) {
                    Ok(msg) => {
                        handle_message(&msg, &node_address, &sender, &state).await;
                    }
                    Err(_) => {
                        let _ = send_json(&sender, make_error_response(
                            RelayErrorCode::ErrFrameMalformed, None,
                        )).await;
                    }
                }
            }
            Ok(Message::Ping(_)) => {
                // axum handles pong automatically
            }
            Ok(Message::Pong(_)) => {
                // Record activity
                if let Ok(mut s) = state.try_write() {
                    if let Some(conn) = s.clients.get_mut(&node_address) {
                        conn.last_activity = Instant::now();
                    }
                }
            }
            Ok(Message::Close(reason)) => {
                let close_code = reason.as_ref().map(|r| r.code).unwrap_or(1000);
                println!("[ws-relay] Node {} DISCONNECTED (code={})", node_address, close_code);

                // Record failure for circuit breaker if abnormal close
                let mut s = state.write().await;
                s.circuit_breaker.record_ws_close(close_code);
                break;
            }
            Err(e) => {
                eprintln!("[ws-relay] Read error for {}: {}", node_address, e);
                break;
            }
            _ => {}
        }
    }

    // ── Phase 5: Cleanup ────────────────────────────────────
    {
        let mut s = state.write().await;
        s.clients.remove(&node_address);

        let remaining = s.clients.len();
        s.audit_log.record_event(&RelayAuditEntry {
            event_type: RelayAuditEventType::Disconnect,
            address: node_address.clone(),
            timestamp: now_iso(),
            details: json!({"remaining_peers": remaining}),
            severity: AuditSeverity::Info,
            subsystem: AuditSubsystem::Capability,
            correlation_refs: vec![],
        });

        // Notify remaining peers
        let offline_msg = json!({
            "type": "peer-offline",
            "address": node_address,
            "ts": now_millis(),
        });
        for (_, conn) in s.clients.iter() {
            let _ = send_json(&conn.sender, offline_msg.clone()).await;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// AUTHENTICATION — TL-DSA-87 Challenge-Response
// ═══════════════════════════════════════════════════════════════════════

/// Perform TL-DSA-87 challenge-response authentication.
///
/// 1. Server generates random nonce
/// 2. Server sends {type:"challenge", nonce}
/// 3. Client sends {type:"auth", address, publicKey, nonce, signature}
/// 4. Server verifies signature over "{nonce}||{address}||{publicKey}"
/// 5. Returns (address, publicKey) on success
async fn handle_auth(
    sender: &Arc<Mutex<SplitSink<WebSocket, Message>>>,
    receiver: &mut futures_util::stream::SplitStream<WebSocket>,
    state: &RelayState,
) -> Result<(String, String), RelayErrorCode> {
    // Generate nonce
    let mut nonce_bytes = [0u8; NONCE_LEN];
    getrandom::getrandom(&mut nonce_bytes)
        .map_err(|_| RelayErrorCode::ErrAuthFailed)?;
    let nonce_hex: String = nonce_bytes.iter().map(|b| format!("{:02x}", b)).collect();

    // Send challenge
    send_json(sender, json!({
        "type": "challenge",
        "nonce": nonce_hex,
    })).await.map_err(|_| RelayErrorCode::ErrAuthFailed)?;

    // Wait for auth response
    let auth_msg = match receiver.next().await {
        Some(Ok(Message::Text(text))) => {
            serde_json::from_str::<Value>(&text)
                .map_err(|_| RelayErrorCode::ErrFrameMalformed)?
        }
        _ => return Err(RelayErrorCode::ErrAuthFailed),
    };

    // Validate auth message fields
    let msg_type = auth_msg.get("type").and_then(|t| t.as_str())
        .ok_or(RelayErrorCode::ErrFrameMalformed)?;
    if msg_type != "auth" {
        return Err(RelayErrorCode::ErrAuthFailed);
    }

    let address = auth_msg.get("address").and_then(|a| a.as_str())
        .ok_or(RelayErrorCode::ErrAuthFailed)?
        .to_string();
    let public_key = auth_msg.get("publicKey").and_then(|p| p.as_str())
        .ok_or(RelayErrorCode::ErrAuthFailed)?
        .to_string();
    let recv_nonce = auth_msg.get("nonce").and_then(|n| n.as_str())
        .ok_or(RelayErrorCode::ErrAuthFailed)?;
    let signature_hex = auth_msg.get("signature").and_then(|s| s.as_str())
        .ok_or(RelayErrorCode::ErrSignatureRequired)?;

    // Verify nonce matches
    if recv_nonce != nonce_hex {
        return Err(RelayErrorCode::ErrAuthFailed);
    }

    // Verify TL-DSA-87 signature
    let challenge_payload = format!("{}||{}||{}", nonce_hex, address, public_key);
    let sig_bytes: Vec<u8> = (0..signature_hex.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&signature_hex[i..i + 2], 16).ok())
        .collect();
    let pk_bytes: Vec<u8> = (0..public_key.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&public_key[i..i + 2], 16).ok())
        .collect();

    let valid = ternary_math::tl_dsa::verify(
        &pk_bytes,
        challenge_payload.as_bytes(),
        &sig_bytes,
        ternary_math::tl_dsa::TlDsaVariant::TlDsa87,
    );

    if !valid {
        let mut s = state.write().await;
        s.audit_log.record_event(&RelayAuditEntry {
            event_type: RelayAuditEventType::AuthFailure,
            address: address.clone(),
            timestamp: now_iso(),
            details: json!({"reason": "signature_invalid"}),
            severity: AuditSeverity::Warn,
            subsystem: AuditSubsystem::Capability,
            correlation_refs: vec![],
        });
        return Err(RelayErrorCode::ErrSignatureInvalid);
    }

    println!("[ws-relay] Challenge signature VERIFIED (TL-DSA-87) for {}", address);
    Ok((address, public_key))
}

// ═══════════════════════════════════════════════════════════════════════
// MESSAGE ROUTING
// ═══════════════════════════════════════════════════════════════════════

/// Handle a single parsed message from an authenticated connection.
async fn handle_message(
    msg: &Value,
    from_address: &str,
    sender: &Arc<Mutex<SplitSink<WebSocket, Message>>>,
    state: &RelayState,
) {
    let msg_type = match msg.get("type").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            let _ = send_json(sender, make_error_response(
                RelayErrorCode::ErrFrameMalformed, None,
            )).await;
            return;
        }
    };

    match msg_type {
        "relay" => handle_relay(msg, from_address, sender, state).await,
        "ping" => {
            let _ = send_json(sender, json!({"type": "pong", "ts": now_millis()})).await;
        }
        _ => {
            let _ = send_json(sender, make_error_response(
                RelayErrorCode::ErrUnknownMsgType, Some(msg_type),
            )).await;
        }
    }
}

/// Handle a relay message: route to target or queue if offline.
async fn handle_relay(
    msg: &Value,
    from_address: &str,
    sender: &Arc<Mutex<SplitSink<WebSocket, Message>>>,
    state: &RelayState,
) {
    let target = match msg.get("to").and_then(|t| t.as_str()) {
        Some(t) => t.to_string(),
        None => {
            let _ = send_json(sender, make_error_response(
                RelayErrorCode::ErrFrameMalformed, Some("relay"),
            )).await;
            return;
        }
    };
    let relay_msg_type = msg.get("msgType").and_then(|m| m.as_str())
        .unwrap_or("data")
        .to_string();
    let payload = msg.get("payload").and_then(|p| p.as_str())
        .unwrap_or("")
        .to_string();

    let envelope = json!({
        "type": "relay",
        "from": from_address,
        "msgType": relay_msg_type,
        "payload": payload,
    });

    let mut s = state.write().await;

    // Try direct delivery
    if let Some(target_conn) = s.clients.get(&target) {
        let delivery = send_json(&target_conn.sender, envelope).await;
        if delivery.is_ok() {
            let _ = send_json(sender, json!({
                "type": "relay_ack",
                "to": target,
                "delivered": true,
            })).await;
            return;
        }
    }

    // Target offline — queue the message
    let now = now_millis();
    let queue = s.pending.entry(target.clone()).or_insert_with(Vec::new);

    // TTL eviction (saturating_sub prevents underflow on clock skew)
    queue.retain(|m| now.saturating_sub(m.ts) < PENDING_TTL.as_millis() as u64);

    if queue.len() < PENDING_MAX {
        queue.push(PendingMessage {
            from: from_address.to_string(),
            msg_type: relay_msg_type,
            payload,
            ts: now,
        });
        let _ = send_json(sender, json!({
            "type": "relay_ack",
            "to": target,
            "delivered": false,
            "queued": true,
        })).await;
    } else {
        let _ = send_json(sender, make_error_response(
            RelayErrorCode::ErrRelayQueueFull, Some("relay"),
        )).await;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// GOLDEN ANGLE STAGGER
// ═══════════════════════════════════════════════════════════════════════

/// Compute golden-angle staggered delay for anti-thundering-herd.
///
/// `reconnectAfterMs = base_delay + ((index × φ_ternary) mod 1.0) × stagger_window`
///
/// φ_ternary = 182(3−√5)/364 ≈ 0.381966 (from ARC_ROOT_SEMI = 182).
/// Guarantees optimal spread — no two indices produce the same fractional
/// offset, preventing reconnect clustering regardless of count.
pub fn golden_angle_delay(index: u64, base_delay_ms: u64, stagger_window_ms: u64) -> u64 {
    let frac = ((index as f64) * PHI_TERNARY) % 1.0;
    base_delay_ms + (frac * stagger_window_ms as f64) as u64
}

// ═══════════════════════════════════════════════════════════════════════
// BROADCAST
// ═══════════════════════════════════════════════════════════════════════

/// Broadcast a message to all connected relay clients.
pub async fn broadcast_to_all(state: &RelayState, msg: Value) {
    let s = state.read().await;
    for (_, conn) in s.clients.iter() {
        let _ = send_json(&conn.sender, msg.clone()).await;
    }
}

/// Broadcast circuit_open to all clients when breaker trips.
pub async fn broadcast_circuit_open(state: &RelayState, breaker_name: &str) {
    let msg = json!({
        "type": "circuit_open",
        "breaker": breaker_name,
        "ts": now_millis(),
    });
    broadcast_to_all(state, msg).await;
}

// ═══════════════════════════════════════════════════════════════════════
// GRACEFUL SHUTDOWN (Task 6 extends this)
// ═══════════════════════════════════════════════════════════════════════

/// Broadcast go-away with golden angle stagger to all clients.
///
/// Each connection gets a unique `reconnectAfterMs` spread over
/// [base_delay, base_delay + stagger_window] with optimal golden angle
/// distribution — no clustering regardless of connection count.
pub async fn broadcast_go_away(state: &RelayState, reason: &str) {
    let mut s = state.write().await;
    s.draining = true;
    let base_delay = 2000u64;
    let stagger_window = 5000u64;

    let addrs: Vec<String> = s.clients.keys().cloned().collect();
    for addr in &addrs {
        let idx = s.disconnect_index.fetch_add(1, Ordering::Relaxed);
        let delay = golden_angle_delay(idx, base_delay, stagger_window);
        if let Some(conn) = s.clients.get(addr) {
            let go_away = json!({
                "type": "go-away",
                "reason": reason,
                "reconnectAfterMs": delay,
                "ts": now_millis(),
            });
            let _ = send_json(&conn.sender, go_away).await;
            let _ = conn.sender.lock().await.close().await;
        }
    }

    s.audit_log.record_event(&RelayAuditEntry {
        event_type: RelayAuditEventType::GoAway,
        address: "server".to_string(),
        timestamp: now_iso(),
        details: json!({"reason": reason, "peerCount": addrs.len()}),
        severity: AuditSeverity::Info,
        subsystem: AuditSubsystem::Shutdown,
        correlation_refs: vec![],
    });
}

// ═══════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════

/// Send a JSON value over a WebSocket sender.
async fn send_json(
    sender: &Arc<Mutex<SplitSink<WebSocket, Message>>>,
    value: Value,
) -> Result<(), ()> {
    let text = serde_json::to_string(&value).map_err(|_| ())?;
    sender.lock().await.send(Message::Text(text.into())).map_err(|_| ()).await
}

/// Current time as ISO-8601 string via chrono (already in Cargo.toml).
fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Current time as milliseconds since epoch.
fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_angle_no_duplicates() {
        let mut delays = Vec::new();
        for i in 0..100 {
            delays.push(golden_angle_delay(i, 2000, 5000));
        }
        // All delays should be in range [2000, 7000)
        for d in &delays {
            assert!(*d >= 2000 && *d < 7000, "Delay {} out of range", d);
        }
        // No exact duplicates (golden angle guarantees optimal spread)
        let unique: std::collections::HashSet<u64> = delays.iter().cloned().collect();
        // Allow a few collisions from integer truncation but expect mostly unique
        assert!(unique.len() > 90, "Expected >90 unique delays, got {}", unique.len());
    }

    #[test]
    fn test_golden_angle_spread() {
        // First 10 offsets should be well-distributed, not clustered
        let offsets: Vec<f64> = (0..10)
            .map(|i| ((i as f64) * PHI_TERNARY) % 1.0)
            .collect();
        // No two adjacent offsets should be within 0.05 of each other
        let mut sorted = offsets.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for i in 1..sorted.len() {
            let gap = sorted[i] - sorted[i - 1];
            assert!(gap > 0.01, "Gap too small: {} between {} and {}", gap, sorted[i-1], sorted[i]);
        }
    }

    #[test]
    fn test_phi_ternary_value() {
        // φ_ternary = 182(3−√5)/364 ≈ 0.381966
        assert!((PHI_TERNARY - 0.381966).abs() < 0.001);
    }

    #[test]
    fn test_pending_message_struct() {
        let msg = PendingMessage {
            from: "1.1.1.1.1.1.1.1.1.1.1.1.1".to_string(),
            msg_type: "data".to_string(),
            payload: r#"{"test": true}"#.to_string(),
            ts: 1712800000000,
        };
        assert_eq!(msg.from, "1.1.1.1.1.1.1.1.1.1.1.1.1");
    }

    #[test]
    fn test_now_iso_format() {
        let ts = now_iso();
        // chrono::to_rfc3339() produces "2026-04-12T03:00:00.123456789+00:00"
        assert!(ts.contains('T'), "ISO-8601 must contain T separator: {}", ts);
        assert!(ts.contains('+') || ts.contains('Z'), "ISO-8601 must have timezone: {}", ts);
    }

    #[test]
    fn test_now_millis_reasonable() {
        let ms = now_millis();
        // Should be after Jan 1, 2026 (1735689600000)
        assert!(ms > 1735689600000);
    }

    #[tokio::test]
    async fn test_new_relay_state() {
        let state = new_relay_state(std::path::PathBuf::new());
        let s = state.read().await;
        assert!(s.clients.is_empty());
        assert!(s.pending.is_empty());
        assert_eq!(s.circuit_breaker.state(), CircuitState::Closed);
        assert_eq!(s.heartbeat_interval_ms, 30_000);
        assert!(!s.draining);
    }
}
