# PlenumNET Invariants for YODA Integration

**Capomastro Holdings Ltd. — Applied Physics Division**
**Authoritative reference for any agent working on YODA (yoda.replit.app)**

---

## SECTION 1: ABSOLUTE INVARIANTS — VIOLATIONS BREAK THE SYSTEM

### I-01: No BLAKE3, No SHA-256, No ed25519

PlenumNET uses its own cryptographic primitives. There are ZERO references to BLAKE3, SHA-256, or ed25519 anywhere in the codebase. Do not introduce, reference, or log them.

| Correct | Wrong |
|---------|-------|
| TL-Sponge-385 | BLAKE3 |
| TL-DSA-87 / PT26-DSA | ed25519 |
| TIS-27 | SHA-256 |
| TL-KEM | X25519 / ECDH |

If you see "PQ-native tunnel keys derived," the parenthetical MUST say `(TL-Sponge-385)`, never `(BLAKE3)`.

### I-02: Rep C Trit Encoding — {1, 2, 3} Only

Ternary trit values are 1, 2, 3. Never 0, 1, 2. There is no trit value 0.

- Valid address: `1111111111112`
- Invalid address: `1111111111110`

### I-03: Port Assignments Are Fixed

These are the production port assignments. Do not change them. Do not suggest alternatives. Do not override them in code.

| Service | Port | Controlled By |
|---------|------|---------------|
| LLM Engine A (llama-server) | 8080 | User starts manually |
| Cube daemon HTTP API | 8081 | `CUBE_API_PORT` env var |
| LLM Engine B | 8082 | User starts manually |
| LLM Engine C | 8084 | User starts manually |

The cube daemon reads `CUBE_API_PORT` (or `API_PORT`) and defaults to 8080 if unset. In the YODA setup, it is always set to 8081.

The cube daemon calls llama-server via `LLM_PORT` env var (default 8080). This is correct — Engine A is on 8080.

Multi-agent addressing (future): Agent N = engine port `8080 + 2N`, node port `8081 + 2N`.

### I-04: Ternary Address Format

Addresses are 13-trit strings using Rep C encoding.

- Flat: `1111111111112`
- Dotted: `111.111.111.111.2` (groups of 3.3.3.3.1)
- Both forms accepted everywhere

CRS address (geometric center): `1111111111111`

### I-05: The Geometry IS the Protocol

Routing is geometric — Hamming distance in a 13D ternary hypercube (3^13 = 1,594,323 vertices, 26 neighbors per node). Do not describe it as a VPN, a mesh network, or a conventional P2P system. It is a self-organizing topological network where the address space defines the routing.

### I-06: Do Not Mock Crypto

All cryptographic operations use real TL-DSA-87 / PT26-DSA. No mock signatures, no placeholder keys, no ed25519 substitutes. A temporary random hex key for relay authentication is an acknowledged shortcut — do not present it as production crypto.

---

## SECTION 2: DAEMON CONFIGURATION

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CUBE_MODE` | `all` | `crs`, `cube`, `all`, or `keygen` |
| `CUBE_CRS_URL` | (required for cube) | CRS base URL |
| `CUBE_API_PORT` | `8080` | Daemon HTTP API port |
| `API_PORT` | (alias) | Alias for CUBE_API_PORT |
| `CUBE_ENDPOINT` | `0.0.0.0:51820` | Wire protocol endpoint |
| `CUBE_ROLE` | (optional) | `inference`, `review`, `kb`, `infra`, `relay`, `standby` |
| `LLM_PORT` | `8080` | Where llama-server listens |
| `CUBE_IDENTITY_DIR` | `~/.plenumnet/identity/` | Master key storage |
| `CUBE_IDENTITY_PASSPHRASE` | (hostname fallback) | Encryption passphrase |

### Daemon Startup (Windows)

```powershell
$env:CUBE_MODE="cube"
$env:CUBE_API_PORT="8081"
$env:CUBE_CRS_URL="https://plenumnet.replit.app"
$env:CUBE_ROLE="inference"
& "C:\Users\Sigma\PlenumNET\target\release\inter-cube-daemon.exe"
```

### Daemon Local API (port 8081)

| Endpoint | Method |
|----------|--------|
| `/health` | GET |
| `/api/salvi/inter-cube/node/info` | GET |
| `/api/salvi/inter-cube/topology` | GET |
| `/api/salvi/inter-cube/glb/stats` | GET |
| `/api/salvi/inter-cube/con/stats` | GET |
| `/api/salvi/inter-cube/fts/status` | GET |
| `/api/salvi/inter-cube/fts/dead` | GET |
| `/api/salvi/inter-cube/address/validate` | POST |

---

## SECTION 3: RELAY PROTOCOL

### CRS Endpoints (plenumnet.replit.app)

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/salvi/inter-cube/relay/register` | GET | Register node (`?publicKey=...&endpoint=...`) |
| `/api/salvi/inter-cube/relay/heartbeat` | GET | Refresh registration (`?address=...&publicKey=...`) |
| `/api/salvi/inter-cube/relay/status` | GET | Connected nodes + pending queues |
| `/health/crs` | GET | CRS daemon health |

### WebSocket Relay

**Endpoint:** `wss://plenumnet.replit.app/ws/relay`

**Auth (first message, 10s deadline):**
```json
{ "type": "auth", "address": "<assigned>", "publicKey": "<hex>" }
```
Response: `{ "type": "auth_ok", "address": "...", "connectedPeers": [...] }`

**Keepalive (every 25s):**
```json
{ "type": "ping" }
```

**Send message:**
```json
{ "type": "relay", "to": "<address>", "msgType": "<type>", "payload": "<json-string>" }
```
Ack: `{ "type": "relay_ack", "to": "...", "delivered": true|false }`

**Receive message:**
```json
{ "type": "relay", "from": "<address>", "msgType": "<type>", "payload": "<json-string>" }
```

**Peer list:**
```json
{ "type": "peers" }
```

Offline queue: up to 100 messages per destination, delivered on reconnect.

### Reconnection Protocol

On every reconnect (including after CRS restart):
1. Re-register via HTTP GET (CRS clears in-memory registry on restart)
2. Open new WebSocket
3. Re-authenticate with the address returned by registration

Backoff: 2s → 4s → 8s → ... → 60s cap.

---

## SECTION 4: INFERENCE VIA RELAY

### Request (YODA → CRS → Cube)

```json
{
  "type": "relay",
  "to": "1111111111112",
  "msgType": "inference_request",
  "payload": "{\"requestId\":\"<uuid>\",\"messages\":[...],\"maxTokens\":512,\"model\":\"local\",\"temperature\":0.7}"
}
```

### Response (Cube → CRS → YODA)

**Success — `inference_response`:**
```json
{
  "type": "relay",
  "from": "1111111111112",
  "msgType": "inference_response",
  "payload": "{\"requestId\":\"<uuid>\",\"content\":\"...\",\"model\":\"local\",\"tokens\":42,\"usage\":{...}}"
}
```

**Error — `inference_error`:**
```json
{
  "type": "relay",
  "from": "1111111111112",
  "msgType": "inference_error",
  "payload": "{\"requestId\":\"<uuid>\",\"error\":\"LLM server unreachable...\"}"
}
```

### Payload Fields

| Field | Type | Required | Default |
|-------|------|----------|---------|
| `requestId` | string (UUID) | yes | — |
| `messages` | array (OpenAI format) | yes | — |
| `maxTokens` | integer | no | 512 |
| `model` | string | no | "local" |
| `temperature` | float | no | 0.7 |

### Timeout

The daemon allows 120 seconds per LLM call. YODA should use its own timeout (e.g. 30s) and fall back to browser relay if no response arrives.

---

## SECTION 5: UI COLOR SCHEME

| Color | Meaning |
|-------|---------|
| Blue | Live / Active |
| Grey | Trouble / Degraded |
| Black | Down / Offline |

No green. No red. These are the only status colors.

---

## SECTION 6: DO NOT TOUCH

These files and systems are off-limits. Do not modify, mock, or rewrite them.

- `deployments/` directory
- `server/ternary.ts`
- `compress/decompress` pipeline
- `ternarydb.tsx`
- `tunnel_auth.rs`
- Engine endpoint URLs in YODA settings (user-configured)
- Port assignments (user-controlled)

---

## SECTION 7: TERMINOLOGY

| Term | Correct Usage |
|------|---------------|
| GLB | Geometric Load Balancer (NOT Global) |
| CON | Cube Overlay Network (NOT Connection Manager) |
| CRS | Cube Registration Service |
| FTS | Fault Tolerance Service |
| PT26-DSA | Daemon identity signature scheme |
| TL-DSA-87 | Address-bound signature scheme (Level 5 PQ) |
| TL-Sponge-385 | Cryptographic sponge (385-bit PQ security) |
| TIS-27 | Wire integrity sponge (43-bit) |
| PlenumLAN | The live 2-node network |
| YODA | The frontend app at yoda.replit.app |
| Cube daemon | The Rust binary on the laptop |
| CRS | The authority at plenumnet.replit.app |

---

*Violations of these invariants will be rejected. This document is the source of truth for any agent integrating with PlenumNET.*
