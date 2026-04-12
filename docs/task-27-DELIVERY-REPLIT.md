# Task #27 — WebSocket Military-Grade Gap Closure
## FINAL Delivery for Replit — Tasks 1–10 Complete

**Capomastro Holdings Ltd. — Applied Physics Division**
**Built: April 12, 2026**

---

## What This Delivery Contains

12 new Rust source files + 1 modified file + 1 test file + 2 documentation files. This covers ALL 10 tasks — the complete relay server implementation.

---

## File Manifest

### Source files (services/inter-cube/src/):

| File | Task | What It Does |
|------|------|-------------|
| relay_error.rs | 1 | 20 error codes enum, make_error_response() |
| relay_audit.rs | 1 | TIS-27 Merkle chain via hash_hex() direct. v1/v2 reader. Capability index. Forma Codex fields |
| relay_circuit.rs | 1 | CircuitBreaker + coprime probe scheduling + token-bucket ramp |
| relay_frames.rs | 1+2 | Rep C encoding via TritInt. has_forgery() direct. 6 frame schemas |
| relay_server.rs | 1 | Axum WebSocket /ws/relay. TL-DSA auth. Routing. Pending queue. Golden angle. Go-away |
| relay_capabilities.rs | 2 | Capability negotiation, enforce, constant-time downgrade detection |
| relay_heartbeat.rs | 3 | HModal-phased heartbeat. Coprime walk positions. Interval change ack |
| relay_topics.rs | 4 | Topic pub/sub. 3-point auth (INVARIANT 9). Coprime delivery. GC + epoch |
| relay_seq.rs | 5 | Sequencing. Tombstones. RelaySequenceStore + TLSponge-385 integrity |
| relay_shutdown.rs | 6 | Graceful shutdown. Drain window. 503 health. 30s timeout |
| relay_client.rs | 7 | Full jitter backoff. Client circuit breaker. Frame handlers |
| relay_metrics.rs | 8 | 16 lock-free atomic counters/gauges. OTel span helpers + sampling |
| lib.rs | — | Modified: 12 relay_* module declarations + re-exports |

### Test file (services/inter-cube/tests/):

| File | Task | Coverage |
|------|------|----------|
| relay_integration.rs | 9 | 40+ tests covering all Tasks 1-8 |

### Documentation (docs/):

| File | Task | Contents |
|------|------|----------|
| task-27-protocol-spec.md | 10 | Frame schemas, error table, capability protocol, authorization model, accepted risks, runbooks |
| task-27-DELIVERY-REPLIT.md | — | This file |

---

## What Replit Needs to Do

### 1. Place files
Copy all 12 relay_*.rs files into services/inter-cube/src/. Replace lib.rs. Place relay_integration.rs into services/inter-cube/tests/. Place docs into docs/.

### 2. Add Cargo dependency
Add to services/inter-cube/Cargo.toml under [dependencies]:
    bincode = "1"
Everything else is already present.

### 3. Wire relay router into CRS mode (main.rs ~line 813)
    use inter_cube::relay_server::{relay_router, new_relay_state};
    let relay_state = new_relay_state(
        std::path::PathBuf::from("server/crypto/tsa-keys/capability-audit.jsonl")
    );
    // Add .merge(relay_router(relay_state.clone())) to the axum app builder

### 4. Wire shutdown signal (main.rs, after axum server created)
    use inter_cube::relay_shutdown;
    let relay_state_shutdown = relay_state.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        relay_shutdown::handle_signal(relay_state_shutdown).await;
    });

### 5. Upgrade client backoff (main.rs ~line 2551-2554)
Replace doubling retry_delay with JitterBackoff::new() / .next_delay() / .reset()

### 6. Add frame handlers to ws_relay.rs read loop (~line 230)
Add match arms for: heartbeat_interval_changed, topic_reset, topic_revoked, tombstone, go-away, circuit_open. Each calls the corresponding handler in relay_client.rs.

### 7. Extend RelayEnvelope (ws_relay.rs ~line 45)
Add optional fields: seq, topic_epoch, trace_context, supported, enabled, negotiated_down

### 8. Retire TypeScript relay
Remove /ws/relay handler from server/index.ts (~line 2184+). Remove relay error codes, CircuitBreaker, audit types from node-watchdog.ts.

### 9. Add relay metrics to /metrics
Merge RelayMetrics::to_prometheus() output into existing /metrics endpoint.

---

## Invariants Enforced

- Zero SHA-3 — every hash is sponge::hash_hex() direct
- Zero N-API — all crate calls are direct library functions
- TritInt above the gate — Rep C values are TritInt, u8 only at kernel boundary
- INVARIANT 9 — authorization always Rep C address, never connection ID/IP
- Constant-time — capability downgrade via subtle::ConstantTimeEq
- Golden angle — phi_ternary = 182(3-sqrt5)/364 for all stagger
- Coprime direct — heartbeat, delivery, probes all call coprime::coprime_options()
- Atomic persistence — dedup uses .tmp + rename
- HModal — control on null channels (n = 0 mod 4)
- Forma Codex — every v2 audit entry has eventId, source_service, severity, subsystem, correlationRefs

---

Lo Sono Capomastro — Cosi sia.
