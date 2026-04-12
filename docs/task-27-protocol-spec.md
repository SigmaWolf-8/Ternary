# PlenumNET Relay Protocol — Task #27

## Control Frame Reference

**Capomastro Holdings Ltd. — Applied Physics Division**
**Version: 1.0 — April 2026**

---

## 1. Error Code Table (20 codes)

| Code | Message | WS Close | Trigger |
|------|---------|----------|---------|
| `ERR_AUTH_FAILED` | Authentication failed | 1008 | Address not registered or publicKey mismatch |
| `ERR_SIGNATURE_INVALID` | Signature verification failed | 1008 | TL-DSA-87 challenge signature fails |
| `ERR_SIGNATURE_REQUIRED` | Signature required | 1008 | TL-DSA key registered but no signature sent |
| `ERR_AUTH_TIMEOUT` | Authentication timeout | 1008 | No auth within 10 seconds |
| `ERR_RATE_LIMITED` | Rate limit exceeded | 1008 | Too many requests |
| `ERR_FRAME_MALFORMED` | Malformed frame | 1003 | Invalid JSON, bad structure, malformed capability string |
| `ERR_FRAME_TOO_LARGE` | Frame too large | 1009 | Exceeds 64KB max frame size |
| `ERR_RELAY_TARGET_UNKNOWN` | Target unknown | — | Destination not connected |
| `ERR_RELAY_QUEUE_FULL` | Queue full | — | Legacy client: queue at 500 capacity |
| `ERR_UNKNOWN_MSG_TYPE` | Unknown type | — | Unrecognized message type |
| `ERR_NOT_AUTHENTICATED` | Not authenticated | — | Message before auth |
| `ERR_CIRCUIT_OPEN` | Circuit breaker open | — | CRS verification breaker tripped |
| `ERR_CAPABILITY_NOT_NEGOTIATED` | Capability not negotiated | — | Feature requires capability not in auth_ok enabled |
| `ERR_CAPABILITY_DOWNGRADE` | Downgrade rejected | 1008 | Reconnect with fewer capabilities than baseline |
| `ERR_TOPIC_BACKPRESSURE` | Topic queue full | — | Per-topic queue at capacity |
| `ERR_TOPIC_UNAUTHORIZED` | Not authorized | — | Rep C address not in topic permission table |
| `ERR_TOPIC_LIMIT_EXCEEDED` | Topic limit exceeded | — | Per-connection (50) or per-server (10,000) max |
| `ERR_TOPIC_RESET` | Topic reset | — | Topic GC'd and recreated with new epoch |
| `ERR_RESYNC_RATE_LIMITED` | Resync rate limited | — | >3 resync requests per minute |
| `ERR_RESYNC_PAYLOAD_TOO_LARGE` | Bitmap too large | — | Resync bitmap exceeds 8KB |

All errors produce: `{"type":"error","error":"ERR_...","message":"...","offendingType":"..."}`

---

## 2. Capability Negotiation

### Version String Format
```
name:version
name    = [a-z][a-z0-9_]*     (starts with lowercase letter)
version = [1-9][0-9]*         (positive integer, no leading zeros)
```

**Valid:** `topics:1`, `seq:12`, `future_feature:1`
**Invalid:** `topics:01`, `topics:0`, `TOPICS:1`, `:1`, `topics:`, `topics`

### Protocol Flow
1. Client sends `supported: ["topics:1","seq:1","heartbeat:1"]` in auth message
2. Server intersects with its capabilities, negotiates down if needed
3. Server replies in auth_ok: `enabled: ["topics:1","seq:1"]`, `negotiated_down: ["seq"]`
4. Server enforces enabled set on every subsequent message
5. Old clients omitting `supported` get existing behavior unchanged

### Downgrade Detection
- Server persists each node's capability set to audit log
- On reconnect, compares against baseline (constant-time via `subtle`)
- Downgrade rejected with `ERR_CAPABILITY_DOWNGRADE` by default
- Admin flag `allow_capability_downgrade` overrides (audit-logged)
- Baseline reconstructed from audit log on server restart

---

## 3. Control Frame Schemas

### 3.1 tombstone (Rep C = 1)
**When:** Global per-client queue eviction. NOT from topic backpressure, reauth failure, or GC.

```json
{
  "type": "tombstone",
  "resyncCount": 1,
  "suggestedResyncAfterMs": 3500,
  "topicSeqs": {
    "sensor-data": {"seq": 47, "topicEpoch": 1712800000000}
  },
  "gapSizeEstimate": 42
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| resyncCount | number | yes | How many tombstones this client has received |
| suggestedResyncAfterMs | number | yes | Golden angle staggered delay (2000-7000ms) |
| topicSeqs | object | yes | Per-topic seq + epoch snapshot |
| gapSizeEstimate | number | yes | Approximate evicted message count |

**topicSeqs absence semantics:** A topic absent from the snapshot means no eviction occurred — the client's persisted seq is authoritative.

**Client behavior:**
1. Apply topicSeqs to persisted dedup state (present topics reset, absent unchanged)
2. Wait suggestedResyncAfterMs before reconnecting
3. On reconnect, send lastSeenSeq per topic with topicEpoch

### 3.2 topic_reset (Rep C = 2)
**When:** Client's topicEpoch doesn't match the topic's current epoch (topic was GC'd and recreated).

```json
{
  "type": "topic_reset",
  "topic": "sensor-data",
  "oldEpoch": 1712800000000,
  "newEpoch": 1712800300000,
  "currentSeq": 1
}
```

**Client behavior:**
1. Reset persisted seq for this topic to 0
2. Set topicEpoch to newEpoch
3. Request replay from seq 1 of the new epoch

### 3.3 topic_revoked (Rep C = 3)
**When:** Node's permissions on a topic were revoked mid-session (reauthorization failure on heartbeat cycle).

```json
{
  "type": "topic_revoked",
  "topic": "sensor-data",
  "reason": "permission_revoked",
  "lastDeliveredSeq": 47,
  "topicEpoch": 1712800000000
}
```

**Client behavior:**
1. Remove this topic from subscription set
2. Stop expecting messages for this topic
3. May re-subscribe if permissions are restored

**This is NOT a tombstone** (no queue pressure — access was lost) and **NOT a topic_reset** (topic still exists, client just lost permission).

### 3.4 heartbeat_interval_changed
```json
{
  "type": "heartbeat_interval_changed",
  "heartbeatIntervalMs": 15000
}
```

**Client must ack within 10 seconds.** Non-ack: server keeps old interval, audit-logs. Client gets new interval on next reconnect.

### 3.5 circuit_open
```json
{
  "type": "circuit_open",
  "breaker": "crs-verification",
  "ts": 1712800000000
}
```

**Client behavior:** Extend backoff duration.

### 3.6 go-away
```json
{
  "type": "go-away",
  "reason": "server_shutdown",
  "reconnectAfterMs": 3500,
  "ts": 1712800000000
}
```

**Client behavior:** Stop sending new messages, ack, schedule reconnect after hinted delay plus own jitter.

---

## 4. Authorization Model

**INVARIANT 9:** The authorization subject is always the node's Rep C address, never a connection ID, hostname, or IP address.

- **Subscribe:** Any authenticated node can subscribe (default)
- **Publish:** Only the topic creator (by Rep C address) can publish (default)
- **Delivery:** Reauthorized on every heartbeat cycle
- **Revocation:** Sends topic_revoked, force-unsubscribes

Permission table is **in-memory and ephemeral** — not persisted. GC discards it. Server restart clears all topic state.

---

## 5. Accepted Risk

**Control frame replay within TLS session:** Not mitigated independently because all frames are within a TL-DSA authenticated, TLS-encrypted session. If TLS is compromised, the attacker has full session control.

**TLS termination warning:** If TLS terminates upstream (e.g., TLS-terminating load balancer), end-to-end encryption is violated and control frame replay becomes a real risk. Use TLS passthrough or re-encryption.

---

## 6. Architecture Migration Note

The relay server moved from TypeScript (`server/index.ts`) to Rust (`services/inter-cube/src/relay_server.rs`).

- Same `/ws/relay` endpoint
- Same protocol (backward compatible)
- Same TL-DSA-87 challenge-response auth
- New binary: `inter-cube-daemon` serves the relay alongside CRS, YODA, cluster shell
- TypeScript relay handler in `server/index.ts` is retired

---

# Operational Runbooks

## A. Prometheus Metrics

### Scrape Configuration
```yaml
scrape_configs:
  - job_name: 'plenumnet-relay'
    static_configs:
      - targets: ['127.0.0.1:<API_PORT>']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

### Relay Metric Queries
```bash
# Circuit breaker state (0=closed, 1=half-open, 2=open)
curl -s http://127.0.0.1:$PORT/metrics | grep plenum_relay_circuit_state

# Messages delivered
curl -s http://127.0.0.1:$PORT/metrics | grep plenum_relay_messages_delivered_total

# Active topics
curl -s http://127.0.0.1:$PORT/metrics | grep plenum_relay_topics_active

# Tombstones generated
curl -s http://127.0.0.1:$PORT/metrics | grep plenum_relay_tombstones_generated_total

# Capability downgrades
curl -s http://127.0.0.1:$PORT/metrics | grep plenum_relay_capability_downgrades_total

# Heartbeat failures
curl -s http://127.0.0.1:$PORT/metrics | grep plenum_relay_heartbeat_failures_total

# All relay metrics at once
curl -s http://127.0.0.1:$PORT/metrics | grep ^plenum_relay_
```

## B. Manual Circuit Breaker Reset
```bash
curl -X POST http://127.0.0.1:$PORT/admin/circuit-breaker/crs-verification/reset
# Response: {"ok":true,"breaker":"crs-verification","state":"closed"}
```
This is audit-logged. Use when the breaker is stuck open after an upstream recovery.

## C. Maintenance Drain Procedure

1. **Initiate drain:** Send SIGTERM to the inter-cube-daemon process
2. **Observe:** Health endpoint returns 503 with `{"status":"draining",...}`
3. **Wait:** Relay drain window (10s) — in-flight messages complete
4. **Verify:** All connections drained (check logs for "All connections drained")
5. **Timeout:** 30s overall — after which the process exits regardless

## D. Audit Log Entries

### Example: Capability Negotiation
```json
{
  "hash": "a1b2c3...",
  "parent_event_hash": "d4e5f6...",
  "hashAlgorithm": "tis-27",
  "event": "relay.capability_negotiation",
  "jti": "relay-relay.capability_negotiation-42",
  "ts": "2026-04-12T00:00:00+00:00",
  "details": {"subject":"1.2.3.1.2.3.1.2.3.1.2.3.1","capabilities":["topics:1","seq:1"]},
  "eventId": "550e8400-e29b-41d4-a716-446655440000",
  "source_service": "relay",
  "severity": "info",
  "subsystem": "capability"
}
```

### Example: Topic Revoked
```json
{
  "hash": "b2c3d4...",
  "parent_event_hash": "a1b2c3...",
  "hashAlgorithm": "tis-27",
  "event": "relay.topic_revoked",
  "jti": "relay-relay.topic_revoked-43",
  "ts": "2026-04-12T00:01:00+00:00",
  "details": {"topic":"sensor-data","reason":"permission_revoked","address":"1.2.3.1.2.3.1.2.3.1.2.3.1"},
  "eventId": "660e8400-e29b-41d4-a716-446655440001",
  "source_service": "relay",
  "severity": "warn",
  "subsystem": "topic",
  "correlationRefs": ["550e8400-e29b-41d4-a716-446655440000"]
}
```

## E. Alerting Migration: ERR_RELAY_QUEUE_FULL

**Behavioral change:** For seq-capable clients (negotiate `seq:1`), queue overflow produces a tombstone instead of `ERR_RELAY_QUEUE_FULL`. Legacy clients without `seq:1` still receive `ERR_RELAY_QUEUE_FULL`.

**Action:** Update alerting rules that key on this error code. Monitor `plenum_relay_tombstones_generated_total` for seq-capable client overflow events.

---

*Lo Sono Capomastro — Così sia.*
