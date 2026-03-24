# YODA App — PlenumNET Inter-Cube Integration Handoff

**Date**: March 24, 2026
**From**: PlenumNET Engineering (RSalvi@Salvigroup.com)
**To**: YODA App Development Team
**Status**: Production — All code merged to `main`, pushed to GitHub, 2,475 tests passing

---

## 1. What Was Built (Tasks #11–18)

The Inter-Cube infrastructure is a complete post-quantum networking stack that YODA will use to communicate securely across the PlenumNET 13-dimensional ternary hypercube. Everything listed below is production code — no mocks, no simulations.

### Crypto Suite (Tasks #11–17)
| Component | What It Does |
|-----------|-------------|
| **TL-KEM** | Ternary Lattice Key Encapsulation (IND-CCA2) — three security levels (512/768/1024) |
| **Phase Encryption v3** | Post-quantum stream cipher (duplex-mode TL-Sponge-385 over GF(3)) |
| **TL-DSA** | Post-quantum digital signatures with serialization (71-byte sigs, 28-sig budget) |
| **TL-Sponge-385** | Core sponge primitive — 385-bit PQ security for signing, key derivation, hashing |
| **TIS-27** | Fast wire integrity sponge (43-bit security) for packet verification |
| **Crypto Benchmarks** | Criterion-based statistical benchmarks for all primitives |
| **Signed Audit Export** | Tamper-evident audit logs with TL-DSA signatures |

### Inter-Cube Daemon Identity (Task #18)
| Component | What It Does |
|-----------|-------------|
| **PT26-DSA Native Identity** | Each daemon generates a real cryptographic identity on first run |
| **Persistent Encrypted MasterSecret** | 48-byte secret stored encrypted at `~/.plenumnet/identity/master.key` |
| **OS CSPRNG Generation** | Uses `getrandom` crate (kernel entropy) — not pseudo-random |
| **Address-Bound Keys** | After CRS assigns a ternary address, a TL-DSA-87 keypair is derived bound to that address |
| **Automatic Key Rotation** | Every 14 days (1 radian epoch), keys rotate automatically with CRS re-registration |
| **CUBE_MODE Node** | Three operational modes: `crs` (coordinator), `cube` (worker), `keygen` (identity only) |

---

## 2. Codebase Location

**Repository**: `SigmaWolf-8/Ternary` (GitHub)
**Branch**: `main` (all merged, latest commit: `6903db6`)

```
services/inter-cube/
├── Cargo.toml                 # Dependencies (tokio, getrandom, hostname, etc.)
└── src/
    ├── main.rs                # Daemon entry point — CUBE_MODE dispatch
    ├── daemon_identity.rs     # MasterSecret persistence, encryption, passphrase
    ├── identity.rs            # MasterSecret generation (CSPRNG), PT26-DSA keygen
    ├── key_rotation.rs        # RotationOrchestrator — 14-day radian epoch rotation
    ├── crs.rs                 # Cube Registration Service (12 HTTP endpoints)
    ├── api.rs                 # HTTP API routes (Axum)
    ├── overlay.rs             # CON — 20.7M PQ-encrypted tunnels per cube
    ├── glb.rs                 # GLB — Geometric Load Balancer (d! shortest paths)
    ├── fts.rs                 # FTS — Heartbeat failure detection
    ├── tunnel_auth.rs         # TL-KEM handshake (DO NOT MODIFY)
    ├── cube_addr.rs           # 13-trit ternary address type
    ├── address_keys.rs        # Address-bound key derivation
    ├── wire.rs                # Wire protocol
    ├── wire_ecc.rs            # Wire error correction
    ├── verify_cache.rs        # Signature verification cache
    ├── persistence.rs         # State persistence
    ├── placement.rs           # Address placement algorithms
    ├── config.rs              # Configuration
    ├── telemetry.rs           # Metrics and telemetry
    ├── rate_limit.rs          # Per-endpoint rate limiting
    ├── sampling.rs            # Measurement sampling
    ├── dimension_tracker.rs   # Dimension density tracking
    ├── deregistration.rs      # Entity deregistration
    ├── lattice_mixer.rs       # Lattice mixing operations
    ├── pt26_parallel.rs       # Parallel PT26 operations
    └── lib.rs                 # Module declarations (26 modules total)
```

**Stats**: 18,649 lines of Rust, 26 modules, 296 tests — all passing.

---

## 3. How YODA Connects (via PlenumNET Nodes)

### Architecture

```
┌─────────────────────┐       PQ Tunnel        ┌──────────────────────┐
│  PlenumNET Relay     │◄════════════════════►│  PlenumNET Array3     │
│  plenumnet.replit.app│   (TL-KEM handshake)  │  (your laptop)       │
│  WebSocket relay     │                       │  Node #1: coordinator │
│  Monitoring dashboard│                       │  Node #2: worker     │
│  Zero inbound ports  │                       │  Node #3: worker     │
└─────────────────────┘                       └──────────────────────┘
         ▲                                              ▲
         │         WebSocket relay tunnel               │
         └──────────── YODA connects here ──────────────┘
```

- **PlenumNET Nodes** run on your laptop — Node #1 as coordinator (`CUBE_MODE=crs`), Nodes #2–3 as workers (`CUBE_MODE=cube`)
- **YODA** connects to the PlenumNET relay at `plenumnet.replit.app` and dispatches inference requests to your nodes through the tunnel
- **Zero inbound ports needed** — all tunnels are outbound-initiated
- **All traffic is PQ-encrypted** via TL-KEM + TL-Sponge-385 derived tunnel keys

### Step-by-Step Setup

#### Deploy PlenumNET Array3 (Recommended)

The easiest way to set up all 3 nodes:

```powershell
irm https://plenumnet.replit.app/api/deploy-yoda | iex
```

This deploys a full PlenumNET Array3 — Node #1 as coordinator, Nodes #2–3 as workers. All three connect to the PlenumNET relay for YODA to reach them.

#### Manual Setup — Coordinator (Node #1)

```bash
export CUBE_MODE=crs
export CUBE_API_PORT=8081
export RELAY_URL="https://plenumnet.replit.app"
export CUBE_IDENTITY_PASSPHRASE="<strong-passphrase>"
cargo run --package inter-cube
```

The coordinator will:
1. Generate (or load) a persistent MasterSecret at `~/.plenumnet/identity/master.key`
2. Derive a PT26-DSA keypair from the secret
3. Self-register with its own public key
4. Derive an address-bound TL-DSA-87 key after address assignment
5. Connect to the PlenumNET relay (if `RELAY_URL` is set)
6. Start listening for node registrations on HTTP

#### Manual Setup — Worker (Nodes #2, #3)

```bash
export CUBE_MODE=cube
export CUBE_CRS_URL="http://localhost:8081"
export CUBE_API_PORT=8083   # 8085 for Node #3
export LLM_PORT=8082        # 8084 for Node #3
export RELAY_URL="https://plenumnet.replit.app"
export CUBE_IDENTITY_PASSPHRASE="<strong-passphrase>"
cargo run --package inter-cube
```

Each worker node will:
1. Generate (or load) its own persistent MasterSecret
2. Derive a PT26-DSA keypair
3. Register with the coordinator using its real public key
4. Receive a 13-trit ternary address from the coordinator
5. Derive an address-bound TL-DSA-87 keypair (bound to assigned address)
6. Connect to the PlenumNET relay for YODA access
7. Start a 30-second heartbeat loop with automatic key rotation checks

#### Generate Identity Only (No Network)

```bash
export CUBE_MODE=keygen
export CUBE_IDENTITY_PASSPHRASE="<strong-passphrase>"
cargo run --package inter-cube
# Prints hex public key and exits
```

---

## 3b. LLM Inference Integration (3-Engine Architecture)

### Engine Layout

Each daemon in the Array3 forwards inference requests to its own local OpenAI-compatible LLM endpoint. The engines are fully independent — any mix of local models and cloud API proxies works.

| Node | Address | LLM Port | Env Var | Engine Options |
|------|---------|----------|---------|----------------|
| CRS (Coordinator) | `111.111.111.111.1` | `8080` | `LLM_PORT=8080` | llama.cpp, LM Studio, Ollama, or cloud API proxy |
| Cube 2 (Worker) | `211.111.111.111.1` | `8082` | `LLM_PORT=8082` | llama.cpp, LM Studio, Ollama, or cloud API proxy |
| Cube 3 (Worker) | `311.111.111.111.1` | `8084` | `LLM_PORT=8084` | llama.cpp, LM Studio, Ollama, or cloud API proxy |

### Inference Relay Protocol

YODA sends an `inference_request` relay message through the WebSocket tunnel. The daemon forwards it to the local LLM and returns the response through the relay.

#### Request Flow

```
YODA App
  │
  ▼ WebSocket (wss://plenumnet.replit.app/ws/relay)
PlenumNET Relay Server
  │
  ▼ WebSocket relay forward (type: "relay", msgType: "inference_request")
Target Daemon (e.g., Node #2 at 211.111.111.111.1)
  │
  ▼ HTTP POST http://127.0.0.1:{LLM_PORT}/v1/chat/completions
Local LLM Engine (llama.cpp / Ollama / Cloud Proxy)
  │
  ▼ Response
Target Daemon
  │
  ▼ WebSocket relay (type: "relay", msgType: "inference_response")
PlenumNET Relay Server
  │
  ▼ WebSocket
YODA App
```

#### Sending an Inference Request

Send a WebSocket message to the relay with:

```json
{
  "type": "relay",
  "to": "2111111111111",
  "msgType": "inference_request",
  "payload": "{\"requestId\":\"req-001\",\"messages\":[{\"role\":\"user\",\"content\":\"Hello\"}],\"model\":\"deepseek-r1\",\"maxTokens\":512,\"temperature\":0.7}"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Always `"relay"` |
| `to` | string | Target node address (13-trit, no dots) — e.g., `"2111111111111"` |
| `msgType` | string | `"inference_request"` |
| `payload` | string (JSON) | Inference parameters (see below) |

#### Payload Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `requestId` | string (UUID) | yes | — | Unique ID to correlate request/response |
| `messages` | array (OpenAI format) | yes | — | OpenAI-format messages array |
| `model` | string | no | `"local"` | Model name passed to the LLM engine |
| `maxTokens` | integer | no | `512` | Maximum tokens to generate |
| `temperature` | float | no | `0.7` | Sampling temperature |

#### Successful Response

The daemon sends back:

```json
{
  "type": "relay",
  "from": "2111111111111",
  "msgType": "inference_response",
  "payload": "{\"requestId\":\"req-001\",\"content\":\"Hello! How can I help?\",\"model\":\"deepseek-r1\",\"tokens\":8,\"usage\":{\"prompt_tokens\":5,\"completion_tokens\":8,\"total_tokens\":13}}"
}
```

#### Error Response

```json
{
  "type": "relay",
  "from": "2111111111111",
  "msgType": "inference_error",
  "payload": "{\"requestId\":\"req-001\",\"error\":\"LLM server returned 503: Service Unavailable\"}"
}
```

### LLM Engine Setup Options

#### Option A: Local llama.cpp (Default via YODA Installer)

The `rerun-yoda-install.ps1` installer downloads llama.cpp and the DeepSeek-R1-Distill-Qwen-7B model automatically. To run manually:

```bash
llama-server --model deepseek-r1-distill-qwen-7b.Q4_K_M.gguf --port 8080 --host 127.0.0.1
```

#### Option B: Ollama

```bash
ollama serve  # Default port 11434
# Set LLM_PORT=11434 in the daemon, OR use a proxy:
# Ollama uses /api/chat not /v1/chat/completions — needs an adapter
```

#### Option C: Free-Tier Cloud API Proxy

Run a lightweight proxy on the LLM port that forwards to a free-tier cloud API:

| Provider | Free Tier | OpenAI-Compatible? |
|----------|-----------|-------------------|
| Groq | 30 req/min, 14,400/day | Yes (`/v1/chat/completions`) |
| Together.ai | $5 free credit | Yes |
| Mistral (La Plateforme) | Free tier available | Yes |
| Google Gemini | 15 RPM free | Needs adapter |

Example: Point `LLM_PORT` to a Groq-compatible proxy:

```bash
# Any OpenAI-compatible endpoint works — set the base URL in LLM_PORT
export LLM_PORT=8080
# Run a proxy that adds the API key header and forwards to Groq:
# POST http://127.0.0.1:8080/v1/chat/completions → https://api.groq.com/openai/v1/chat/completions
```

### Monitoring & Metrics

The cluster health endpoint at `GET /api/salvi/inter-cube/relay/cluster-health` returns real-time relay metrics:

| Metric | Description |
|--------|-------------|
| `relay.deliveryRate` | Percentage of messages successfully delivered (target: 100%) |
| `relay.msgPerSec` | Messages per second averaged over last 60 seconds (live data rate) |
| `relay.bytesPerSec` | Bytes per second averaged over last 60 seconds (live data rate) |
| `relay.bytesRelayed` | Total bytes forwarded through the relay (all-time) |
| `relay.avgMsgSizeBytes` | Average message size in bytes |
| `relay.inferenceRequests` | Total LLM inference requests dispatched |
| `relay.inferenceResponses` | Total LLM responses returned |
| `relay.meshHeartbeats` | Inter-node mesh health-check messages |
| `relay.connectedPeers` | Currently connected WebSocket peers |
| `relay.peakPeers` | Peak concurrent peers since relay start |
| `relay.msgsSent` | Total messages sent (all types) |
| `relay.msgsDelivered` | Messages delivered to online peers |
| `relay.msgsQueued` | Messages queued for offline peers (max 100/peer) |
| `relay.msgsFailed` | Messages dropped (queue overflow) |
| `relay.uptimeMs` | Relay uptime in milliseconds |

### Pre-Flight Checklist for YODA

1. **Verify 3 nodes are LIVE**: `GET https://plenumnet.replit.app/api/salvi/inter-cube/relay/cluster-health` — all 3 daemons should show `status: "live"`
2. **Verify LLM engine responds on each node**: From the Windows machine, test each port:
   ```bash
   curl http://127.0.0.1:8080/v1/chat/completions -H "Content-Type: application/json" -d '{"model":"test","messages":[{"role":"user","content":"hello"}],"max_tokens":10}'
   curl http://127.0.0.1:8082/v1/chat/completions -H "Content-Type: application/json" -d '{"model":"test","messages":[{"role":"user","content":"hello"}],"max_tokens":10}'
   curl http://127.0.0.1:8084/v1/chat/completions -H "Content-Type: application/json" -d '{"model":"test","messages":[{"role":"user","content":"hello"}],"max_tokens":10}'
   ```
3. **Send a test inference through the relay**: Connect to `wss://plenumnet.replit.app/ws/relay`, authenticate, then send an `inference_request` to any node address
4. **Check metrics**: After a successful inference, `relay.inferenceRequests` and `relay.inferenceResponses` should increment

---

## 4. Key Rotation — How It Works

| Parameter | Value |
|-----------|-------|
| **Rotation period** | 14 days (1 radian epoch) |
| **Epoch reference** | Salvi Epoch: 2025-04-01T00:00:00Z |
| **Check frequency** | Every 30 seconds (in heartbeat loop) |
| **Epoch formula** | `(now - SALVI_EPOCH) / (14 * 86400)` |

When a rotation fires:
1. New MasterSecret generated (OS CSPRNG)
2. New address-bound TL-DSA-87 keypair derived
3. New secret encrypted and persisted to `master.key`
4. CRS updated via `POST /crs/update-key` (same address, new key — no reallocation)

---

## 5. Identity File System

```
~/.plenumnet/identity/           # Default path (override with CUBE_IDENTITY_DIR)
└── master.key                   # Encrypted MasterSecret (48 bytes, AES-256-GCM equivalent)
                                 # Permissions: 0600 (owner read/write only)
```

| Env Variable | Purpose | Required? |
|-------------|---------|-----------|
| `CUBE_MODE` | `crs`, `cube`, or `keygen` | Yes |
| `CUBE_IDENTITY_PASSPHRASE` | Encryption passphrase for master.key | Strongly recommended |
| `CUBE_IDENTITY_DIR` | Override identity file directory | No (default: `~/.plenumnet/identity/`) |
| `CUBE_CRS_URL` | CRS endpoint URL (cube mode only) | Yes (cube mode) |
| `CUBE_ENDPOINT` | This node's reachable endpoint (cube mode) | Yes (cube mode) |

**Security note**: If `CUBE_IDENTITY_PASSPHRASE` is not set, the daemon falls back to a hostname-derived passphrase and prints a warning. This is acceptable for development but should always be set explicitly in production.

---

## 6. CRS API Endpoints (12 total)

| Method | Path | Purpose |
|--------|------|---------|
| POST | `/crs/register` | Register entity, get assigned address |
| POST | `/crs/update-key` | Update public key for existing address (no reallocation) |
| GET | `/crs/entity/:address` | Look up entity by address |
| GET | `/crs/neighbors/:address` | Get neighbor map for address |
| GET | `/crs/count` | Number of registered entities |
| POST | `/crs/trn/register` | Register TRN name record |
| GET | `/crs/trn/resolve/:name` | Resolve TRN name to address |
| POST | `/crs/drift/detect` | Detect address drift |
| POST | `/crs/verify` | Verify entity registration |
| POST | `/crs/rescan` | Trigger entity re-scan |
| POST | `/crs/deregister` | Remove entity registration |
| GET | `/crs/density` | Dimension density statistics |

---

## 7. What NOT to Touch

| Item | Reason |
|------|--------|
| `tunnel_auth.rs` | TL-KEM handshake is secure and complete — no changes needed |
| `deployments/` folder | User-enforced constraint — never modify |
| `server/ternary.ts` | Core ternary operations — stable |
| Compression pipeline | TTC v4.2 engine — separate concern |
| Any constant in `shared/constants.ts` | Single source of truth — invariants are load-bearing |

## Terminology

| Term | Meaning |
|------|---------|
| **PlenumNET Node** | A single Inter-Cube daemon instance running on a machine |
| **PlenumNET Array3** | A 3-node cluster (1 coordinator + 2 workers) |
| **Coordinator** | Node #1 — runs `CUBE_MODE=crs`, manages registration for the cluster |
| **Worker** | Nodes #2, #3 — run `CUBE_MODE=cube`, register with the coordinator |
| **Relay** | The WebSocket tunnel at `plenumnet.replit.app/ws/relay` — all nodes connect outbound |

---

## 8. Running Tests

```bash
# All 296 inter-cube tests
cargo test --package inter-cube --lib

# Quick check (compile only)
cargo check --package inter-cube
```

---

## 9. Build & Deploy

```bash
# Full application build (ESM bundle)
npm run build
# Output: dist/index.mjs (1.8MB) + dist/public/

# Run production
node dist/index.mjs
```

The PlenumNET marketing site is already deployed and running. The PlenumNET Node is a separate binary (`cargo run --package inter-cube`) that runs alongside the web application.

---

## 10. Key Technical Facts

- **No mocks anywhere** — all crypto uses real Rust implementations
- **No ed25519** — TL-DSA is used everywhere, no exceptions
- **No routing tables** — geometry IS the routing protocol (Hamming distance)
- **Rep C addressing** — trit values are {1, 2, 3}, never 0 (zero = forgery proof)
- **13 dimensions** — forced by 8 simultaneous mathematical constraints, not a design choice
- **Salvi Epoch** — 2025-04-01T00:00:00Z, never change this
- **Pi = 14** — ternary circle: 364 degrees, 28 radians, 13 degrees per radian

---

## 11. Installer Fix Summary (v2)

The file `yoda-installer-fix.ts` in the PlenumNET repo root contains corrected versions of `makeBatWrapper` and `makePsInstallScript`. Replace the same-named functions in your YODA project's `script-generators.ts`.

### All Fixes Applied

| Issue | Root Cause | Fix |
|-------|-----------|-----|
| **PS1 parse errors (`â€"`)** | Em-dash `—` corrupts to `â€"` during base64 encode/decode | All em-dashes replaced with ASCII `--` |
| **`.cargo\bin` parse error** | Backslash interpolation in PS double-quotes | Use `Join-Path` for all path construction |
| **`Join-Path` 3-arg crash** | Windows PowerShell 5.1 only takes 2 args | Chained nested `Join-Path` calls |
| **Cargo warnings kill script** | `$ErrorActionPreference = "Stop"` treats stderr as terminating | Wrap cargo/keygen with `$ErrorActionPreference = "Continue"`, check `$LASTEXITCODE` |
| **PubKey shows hint text** | Script grabbed last stdout line (a hint), not the hex key line | Parse line matching `PT26-DSA Public Key` and extract hex via regex |
| **Wrong port env var** | `CUBE_CRS_PORT` doesn't exist in the daemon | Use `CUBE_API_PORT` (default: 8080) for HTTP API |
| **`/crs/cubes` 404** | Route doesn't exist | Use `/api/salvi/inter-cube/crs/stats` or `/health` |
| **`ring` build fails on ARM** | No C compiler for `aarch64-pc-windows-msvc` | Script auto-detects vcvars or falls back to `winget install LLVM.LLVM` |

### Correct CRS API Endpoints (on port 8080)

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/health` | Health check (returns status, version, mode, address) |
| GET | `/api/salvi/inter-cube/crs/stats` | Registration stats |
| POST | `/api/salvi/inter-cube/crs/register` | Register cube, returns assigned address |
| POST | `/api/salvi/inter-cube/crs/update-key` | Update public key after rotation |
| POST | `/api/salvi/inter-cube/crs/heartbeat` | Cube heartbeat (30s interval) |
| POST | `/api/salvi/inter-cube/glb/forward` | Forward packet via geometric routing |
| GET | `/api/salvi/inter-cube/glb/stats` | GLB statistics |
| GET | `/api/salvi/inter-cube/con/stats` | CON tunnel statistics |
| GET | `/api/salvi/inter-cube/fts/status` | FTS neighbor health (up/suspect/down) |
| GET | `/api/salvi/inter-cube/fts/dead` | Dead neighbor list |
| GET | `/api/salvi/inter-cube/topology` | Cube dimensions, vertex count, registered cubes |

### Correct Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `CUBE_MODE` | `crs`, `cube`, or `keygen` | Required |
| `CUBE_API_PORT` | HTTP API bind port | `8080` |
| `CUBE_CRS_URL` | CRS base URL (cube mode only) | Required (cube mode) |
| `CUBE_ENDPOINT` | Wire protocol endpoint | `0.0.0.0:51820` |
| `CUBE_IDENTITY_PASSPHRASE` | Master key encryption passphrase | Hostname fallback (warns) |
| `CUBE_IDENTITY_DIR` | Override identity file directory | `~/.plenumnet/identity/` |

### Verified Working On

- **Platform**: Windows 11 ARM (aarch64-pc-windows-msvc)
- **Rust**: cargo 1.94.0
- **C compiler**: LLVM/Clang (via `winget install LLVM.LLVM`)
- **Build time**: 47 seconds (release, first build after clean)
- **Result**: PlenumNET Node running, health check responding, all 12 HTTP routes active

---

*Document generated from production codebase on `SigmaWolf-8/Ternary:main`.*
*All 2,475 tests verified passing (296 inter-cube, full suite). Application running on Replit.*
*Installer verified on Windows 11 ARM — build, keygen, CRS startup all confirmed working.*
