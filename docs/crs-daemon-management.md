# CRS Daemon Startup & Management — Reference

> **Task #76 — Investigation Results**
> Last updated: 2026-03-31
> Status: All 10 questions answered from codebase audit

---

## Q1: What starts the Inter-Cube daemon on Node 1 (CRS)?

**Answer: A compiled Rust binary (`inter-cube-daemon` / `inter-cube-daemon.exe`) launched via environment variables.**

The daemon is started by setting environment variables and running the compiled binary directly. There is no single "start command" — different deployment methods set env vars differently, including the port:

- **Manual / foreground (daemon default port 11124):**
  ```
  CUBE_MODE=crs CUBE_NODE_ID=1 ./target/release/inter-cube-daemon
  ```
  Without `CUBE_API_PORT` set, the daemon defaults to 11124 (`main.rs` line 33).

- **`start-crs.sh` script (Linux, defaults to port 8181):**
  Uses `nohup` for background execution or `systemctl restart plenumnet-crs.service` if a systemd unit is installed. The script defaults `CUBE_API_PORT` to **8181** (`CRS_PORT="${CUBE_API_PORT:-8181}"`, `start-crs.sh` line 6).

- **`deploy-daemon.ps1` (Windows, port varies):**
  Creates a `.bat` wrapper that sets env vars and invokes the binary, then registers it as a Windows Service via `New-Service`. Port is derived from the node ID formula.

- **Relay server auto-spawn (Replit, port 8181):**
  The Express server in `server/index.ts` (lines 2813–2868) automatically searches for the compiled daemon binary and spawns it as a child process on startup. It checks four candidate paths (`dist/inter-cube-daemon`, `target/release/inter-cube-daemon`, and absolute equivalents), then spawns with `CUBE_MODE=crs`, `CUBE_API_PORT=8181`. The daemon runs as a child of the Node.js process (`detached: false`), meaning it dies when the Express server stops.

> **Known deployment variance:** The API port differs across deployment paths. The Rust daemon defaults to 11124, `start-crs.sh` overrides to 8181, and the relay auto-spawn also uses 8181. Always check the actual `CUBE_API_PORT` env var for the running instance.

### Deployment Matrix

| Path | Platform | Port | Supervisor | Auto-restart |
|------|----------|------|------------|-------------|
| Relay auto-spawn (`server/index.ts`) | Replit | 8181 | Express child process | No (dies with server) |
| `start-crs.sh` (systemd) | Linux | 8181 | systemd | If unit file has `Restart=on-failure` |
| `start-crs.sh` (nohup) | Linux | 8181 | None (background) | No |
| `deploy-daemon.ps1` | Windows | Formula-derived | Windows Service + watchdog | Yes (5s/10s/30s + 5min watchdog) |
| Manual | Any | 11124 (default) | None | No |

The daemon mode is controlled by `CUBE_MODE` env var:
- `"crs"` — Central Registration Service (coordinator)
- `"cube"` — Worker cube (registers with a remote CRS)
- `"all"` — Same as CRS (backward compat, also the default)
- `"keygen"` — Generate identity keypair and exit

Port formula for the gateway (used by `deploy-daemon.ps1`): `gateway = 11111 + ((CUBE_NODE_ID - 1) × 27) + 13`
- Node 1: **11124**, Node 2: **11151**, Node 3: **11178**

**Source:** `services/inter-cube/src/main.rs` lines 1–44 (mode + default port), `start-crs.sh` lines 6–27 (8181 default)

---

## Q2: Where is the daemon binary / entry point on the local filesystem?

**Answer: Two locations depending on the deployment scenario.**

| Scenario | Binary Path |
|----------|-------------|
| Windows (deploy-daemon.ps1) | `C:\PlenumNET\target\release\inter-cube-daemon.exe` |
| Linux / Replit | `/home/runner/workspace/target/release/inter-cube-daemon` |
| Docker | Built inside container from `services/inter-cube/Dockerfile` |

The Cargo binary target is defined in `services/inter-cube/Cargo.toml`:
```toml
[[bin]]
name = "inter-cube-daemon"
path = "src/main.rs"
```

Build command: `cargo build --release -p inter-cube`

**Source:** `services/inter-cube/Cargo.toml` line 85–87, `services/inter-cube/deploy-daemon.ps1` line 23, `start-crs.sh` line 7

---

## Q3: Is there a process supervisor wrapping the daemon?

**Answer: Yes — depends on the deployment path (see Deployment Matrix in Q1).**

### Windows (production deployment)
- **Windows Services:** `deploy-daemon.ps1` registers each daemon as `PlenumNET-Cube-{N}` via `New-Service` with `StartupType Automatic`.
- **Service failure recovery:** Configured via `sc.exe failure` — restart after 5s, 10s, 30s.
- **Watchdog scheduled task:** `PlenumNET-Daemon-Watchdog` runs every 5 minutes + on boot, restarts any stopped `PlenumNET-Cube-*` services.
- **Service management script:** `client/public/install/plenumnet-service.ps1` provides `start`, `stop`, `restart`, `status`, `install`, `uninstall`, `logs`, `watchdog` subcommands.

### Linux
- **systemd (if configured):** `start-crs.sh` checks for `/etc/systemd/system/plenumnet-crs.service` and uses `systemctl restart`.
- **Fallback:** `nohup` with output to `/tmp/crs-daemon.log` — no auto-restart.

### Replit (relay auto-spawn)
- The Express server spawns the daemon as a child process (`detached: false`). The daemon has no independent supervisor — if the Express server restarts (e.g., via workflow restart), it re-spawns the daemon automatically. If the daemon crashes independently, the Express server does not respawn it.

### MSI Installer
- `services/inter-cube/plenum-app.toml` declares `kind = "service"` with `autostart = true` and service account `NT SERVICE\\InterCubeDaemon`.

**Source:** `services/inter-cube/deploy-daemon.ps1` lines 322–438, `start-crs.sh`, `services/inter-cube/plenum-app.toml`

---

## Q4: What is the current recovery procedure if the daemon crashes or stops?

**Answer: Windows has auto-restart at multiple layers. Linux depends on deployment method.**

### Windows (full deployment)
1. **Layer 1 — Windows Service recovery:** The `sc.exe failure` config restarts the service after 5s / 10s / 30s on consecutive failures, resetting the counter after 24 hours.
2. **Layer 2 — Watchdog scheduled task:** Runs every 5 minutes under SYSTEM, restarts any `PlenumNET-Cube-*` services that are not running.
3. **Layer 3 — Manual:** `Restart-Service PlenumNET-Cube-1` or `plenumnet-service.ps1 restart 1`.

### Replit (relay auto-spawn)
- If the daemon child process crashes, the Express server does NOT respawn it. Recovery requires restarting the Express server (workflow restart), which re-runs the auto-spawn logic in `server/index.ts` lines 2813–2868.

### Linux
- **With systemd (if unit file exists):** `start-crs.sh` checks for `/etc/systemd/system/plenumnet-crs.service` and uses `systemctl restart`. The unit file's restart policy (e.g., `Restart=on-failure`) would control auto-recovery — but this unit file is not provided in the codebase; it must be created by the operator.
- **Without systemd:** `start-crs.sh` falls back to `nohup` — no auto-restart on crash. Manual restart only.

### Heartbeat persistence
The daemon persists heartbeat sequence numbers to disk (`persistence.rs`) so that after restart, FTS replay protection survives — the node resumes with correct sequence state minus a sliding window of 10 (`LOAD_WINDOW_ADJUST: u64 = 10`, `persistence.rs` line 75) to handle in-flight heartbeats sent before the restart but arriving after.

**Source:** `services/inter-cube/deploy-daemon.ps1` lines 392, 405–426, `services/inter-cube/src/persistence.rs` lines 42–45 (design), 75 (constant), 334 (application)

---

## Q5: Does NinjaExec have any capability to start/stop/restart local processes?

**Answer: No. NinjaExec is strictly a TL-DSA signing agent.**

NinjaExec (`ninja-exec`) listens on `127.0.0.1:21027` and provides these operations (`ninja-exec/src/server.rs` line 623+):
- `/sign` — Sign a payload with the operator's TL-DSA-87 private key
- `/verify` — Verify a signature against a public key
- `/pubkey` — Export the operator's public key (base64)
- `/status` — Agent status (uptime, locked/unlocked)
- `/lock` / `/unlock` — Lock/unlock the keystore
- `/confirm/pending` (GET), `/confirm/decide` (POST) — Confirmation-gated signing flow

All routes are cryptographic key operations. NinjaExec has **no process management capabilities** — it does not start, stop, or restart any local processes. Its sole purpose is key custody and signing.

**Source:** `ninja-exec/src/server.rs` (route definitions), `ninja-exec/src/main.rs`

---

## Q6: Is there an existing relay endpoint or WSS message type to signal a node to restart?

**Answer: A `"restart"` relay message exists but only triggers a relay reconnect, not a daemon process restart. The `exec` ops message type is the current path for actual process restart.**

### What exists today
- **`"restart"` relay message:** The daemon's WebSocket relay client handles `msg_type == "restart"` by closing the relay connection and reconnecting (`main.rs` lines 1604–1606: `"[ws-relay] Restart command received — closing relay for reconnect"`). This restarts the WebSocket connection, NOT the daemon process itself.
- **Ops channel message types** (defined in `shared/ops-protocol.ts`): `exec`, `exec-result`, `tail`, `tail-data`, `tail-stop`, `telemetry`, `file-push`, `file-pull`, `chunk-*`, `model-swap`, `ops-error`.
- **`exec` message type:** Executes a PowerShell/shell script on the target node with TL-DSA signature verification. This is the current viable path for actual daemon process restart — an operator sends a signed `exec` command with `Restart-Service PlenumNET-Cube-1`.

### What does NOT exist
- No dedicated `restart-daemon` or `service-control` ops message type for process lifecycle management
- No REST endpoint for daemon lifecycle management
- No self-restart capability in the daemon process itself (only relay reconnect via `"restart"`)

### Relay architecture
The WebSocket relay is implemented in `server/index.ts` (path `/ws/relay`, line 2801). It forwards ops messages between the Array3 Monitor (browser) and connected daemon nodes. The `OpsChannelService` (`server/services/ops-channel.ts`) validates signatures, checks scopes, and routes messages. The daemon's `OpsHandler` (`services/inter-cube/src/ops_handler.rs`) processes incoming ops messages. Note: `server/routes/inter-cube.ts` contains the HTTP API routes (CRS registration, health, etc.), not the WebSocket relay.

**Source:** `services/inter-cube/src/main.rs` lines 1604–1606 (restart = reconnect), `server/index.ts` line 2801 (relay), `shared/ops-protocol.ts`, `server/services/ops-channel.ts`

---

## Q7: Simplest path for Array3 Monitor to send a "restart CRS" command from a browser?

**Evaluation of all four options:**

### Option A: New relay endpoint `POST /api/salvi/inter-cube/daemon/restart`
- **Feasibility:** Medium. Would need to be a relay-side endpoint that translates to a WSS message forwarded to the daemon.
- **Pros:** Clean REST API, easy to call from frontend, can add auth/rate-limiting.
- **Cons:** Adds a new endpoint that duplicates existing ops channel functionality.
- **Verdict:** Viable but redundant with the ops channel.

### Option B: WSS message type on the relay channel
- **Feasibility:** High. The ops channel infrastructure already exists and handles signed messages.
- **Pros:** Uses existing auth, audit, and routing infrastructure. Just needs a new message type (e.g., `"service-control"`) or can use `exec` with a restart script.
- **Cons:** Requires the daemon to be connected to the relay (which it should be, but if it's down, it can't receive the message).
- **Verdict:** **Recommended approach** — use existing `exec` for immediate capability, add a dedicated `service-control` message type for a cleaner long-term solution.

### Option C: NinjaExec accepting a restart command
- **Feasibility:** Low. NinjaExec is architecturally a signing-only agent. Adding process management would violate its security model (single responsibility: key custody).
- **Pros:** NinjaExec runs locally and could control local processes.
- **Cons:** Fundamentally changes NinjaExec's purpose. Creates a privilege escalation surface.
- **Verdict:** Not recommended — wrong tool for the job.

### Option D: Local HTTP endpoint on the daemon (`POST /api/salvi/inter-cube/self/restart`)
- **Feasibility:** Medium. The daemon already runs an Axum HTTP server.
- **Pros:** Direct, no relay dependency. Works even if relay is down.
- **Cons:** The daemon can't easily restart itself (it would need to spawn a replacement and exit). Also requires the daemon to be reachable from the browser, which it typically isn't (localhost on the target machine).
- **Verdict:** Useful as a secondary mechanism, but the daemon's port is not exposed to the browser — the relay is the browser's only path to the daemon.

### Recommended Path (immediate)
**Use Option B with existing `exec` message type.** The Array3 Monitor sends a signed `exec` ops message through the relay with script:
```powershell
Restart-Service PlenumNET-Cube-1
```
This works today with no code changes, requires TL-DSA signature from NinjaExec (security maintained), and produces an audited result.

> **Security note:** The `exec` message type requires the operator's TL-DSA signature and a scope of `"full"` or `"exec-only"`. If using `exec` for restart, the daemon-side handler should apply command allowlisting (only `Restart-Service PlenumNET-Cube-*` and `systemctl restart plenumnet-*`) to prevent arbitrary command execution via this path.

### Recommended Path (long-term)
Add a dedicated `service-control` ops message type with actions: `restart`, `stop`, `status`. This provides better semantics, dedicated audit entries, and avoids arbitrary script execution for a common operation.

---

## Q8: What environment variables / config does the daemon need at startup?

**Answer: Environment variables — no `.env` file. Set in the shell, wrapper `.bat`, or systemd unit.**

### Required (for cube mode only)
| Variable | Example | Purpose |
|----------|---------|---------|
| `CUBE_CRS_URL` | `https://plenumnet.replit.app` | CRS coordinator URL (required for cube mode to register) |

### Required-ish (have defaults but should be set explicitly)
| Variable | Default | Example | Purpose |
|----------|---------|---------|---------|
| `CUBE_MODE` | `"all"` (= CRS) | `cube` | Operating mode — must be set to `"cube"` for worker nodes |
| `CUBE_NODE_ID` | `1` | `2` | Node identity, Rep C {1,2,3} — defaults to 1 (`main.rs` line 38) |

### Important (with defaults)
| Variable | Default | Purpose |
|----------|---------|---------|
| `CUBE_API_PORT` / `API_PORT` | `11124` | HTTP API bind port |
| `CUBE_PEER_PORT` / `PEER_PORT` | `API_PORT - 1` | Direct peer-to-peer WebSocket port |
| `CUBE_TERMINAL_PORT` | `API_PORT - 2` | PTY terminal WebSocket port |
| `CUBE_ENDPOINT` / `ADDRESS` | `0.0.0.0:51820` | Wire protocol endpoint |
| `CUBE_ROLE` / `ROLE` | (none) | Role annotation: inference, review, kb, infra, relay, standby |
| `CUBE_IDENTITY_DIR` | `~/.plenumnet/identity/` | Directory for master.key |
| `CUBE_IDENTITY_PASSPHRASE` | (none) | Passphrase for master.key encryption |
| `CUBE_CLUSTER_TOKEN` | (none) | Shared secret for cluster API auth |
| `CUBE_ARRAY3_PEERS` | (none) | Comma-separated peer addresses |
| `RELAY_URL` | `CUBE_CRS_URL` | WebSocket relay URL |
| `LLM_PORT` | `API_PORT + 1` | Local LLM engine port |
| `CUBE_TERMINAL_BIND` | `127.0.0.1` | Terminal WebSocket bind address |

### Feature flags (PLENUM_* prefix)
| Variable | Default | Purpose |
|----------|---------|---------|
| `PLENUM_REQUIRE_SIGNATURE` | `false` | Require TL-DSA on CRS registrations |
| `PLENUM_ENABLE_RATE_LIMIT` | `false` | Per-IP rate limiting with PoW |
| `PLENUM_POW_K` | `5` | PoW leading zero trits |
| `PLENUM_PROTOCOL_VERSION` | `3` | Wire protocol version to emit |
| `PLENUM_PROTOCOL_VERSION_MIN` | `2` | Minimum wire version to accept |
| `PLENUM_ENABLE_DUAL_CHECKSUM` | `false` | mod-364 + mod-333 wire checksum |
| `PLENUM_ENABLE_WIRE_ECC` | `false` | 8-trit ECC syndrome |
| `PLENUM_ENABLE_SPONGE_SHUFFLES` | `false` | σ_A–σ_D block permutations |
| `PLENUM_ENABLE_SLOT_ADDRESSING` | `false` | V3 slot addressing |
| `PLENUM_ENABLE_KEY_FRESHNESS` | `false` | Key freshness zone in heartbeats |
| `PLENUM_API_KEY` | (none) | API key for slot endpoint auth |
| `PLENUM_SLOT_REGISTRY` | (none) | JSON slot→service-type mapping |

No `.env` file is used. The deploy scripts set env vars in wrapper `.bat` files (Windows) or shell env (Linux).

**Source:** `services/inter-cube/src/main.rs` lines 6–43, `services/inter-cube/src/config.rs`

---

## Q9: On Windows ARM64 — target triple and binary location?

**Answer: The binary supports both `aarch64` and `x86_64`. Build output is at the standard Cargo location.**

### Target architecture
- `deploy-daemon.ps1` auto-detects the architecture at runtime:
  ```powershell
  $cpuArch = if ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -eq 'Arm64') { "Arm64" } else { "x64" }
  ```
- `plenum-app.toml` declares support for both: `architecture = ["aarch64", "x86_64"]`

### Compiled binary location
| Platform | Path |
|----------|------|
| Windows (deploy-daemon.ps1) | `C:\PlenumNET\target\release\inter-cube-daemon.exe` |
| Native `aarch64` build | `target\aarch64-pc-windows-msvc\release\inter-cube-daemon.exe` (if cross-compiled) |
| Standard `cargo build --release` | `target\release\inter-cube-daemon.exe` (uses host triple) |

The `deploy-daemon.ps1` script does `cargo build --release -p inter-cube` which compiles for the active Rust toolchain's host target. On an ARM64 Windows machine with the `aarch64-pc-windows-msvc` toolchain installed, this produces a native ARM64 binary at the standard `target/release/` path. Verify the active toolchain with `rustc -vV` and check the output binary architecture after build.

**Source:** `services/inter-cube/deploy-daemon.ps1` lines 22–23, 156, `services/inter-cube/plenum-app.toml` line 16

---

## Q10: Health-check or readiness probe after sending a restart command?

**Answer: Yes — use the ops status endpoint and telemetry messages.**

### Option A: Poll `GET /api/ops/status` for `connection_state` (Recommended)
The `OpsChannelService` tracks each node via `updateNodeSeen()` (`server/services/ops-channel.ts` line 222). Each node's status includes a `connection_state` field (`ops-channel.ts` lines 252–260) with values:
- `"connected"`: Last seen < 90 seconds ago
- `"suspect"`: Last seen 90–300 seconds ago
- `"disconnected"`: Last seen > 300 seconds ago

The Array3 Monitor can poll `GET /api/ops/status` (requires ops auth, `server/index.ts` line 527) and watch for the restarted node's `connection_state` to transition from `"disconnected"` → `"connected"`.

Note: The `computeHealthState()` function in `node-watchdog.ts` (lines 27–33) uses different thresholds (UP < 60s, SUSPECT < 300s, DOWN > 300s) but this is a library function — it is not directly exposed as an API field on the ops status response. Use `connection_state` from `OpsChannelService.getNodeStatus()` for restart verification.

### Option B: Direct health endpoint (if network-reachable)
The daemon exposes `GET /health` on its API port. After restart, poll `http://{daemon_ip}:{port}/health` for a 200 response. This requires direct network access to the daemon, which is typically NOT available from the browser — only useful for server-side or same-network monitoring.

### Option C: Telemetry heartbeat via ops channel
The daemon sends periodic `telemetry` messages through the ops channel. After restart, the first telemetry message confirms the daemon is back. If the telemetry payload includes process uptime, a low value confirms a genuine restart (not just a reconnect).

### Recommended approach
1. **Immediate:** Poll `GET /api/ops/status` — when the node's `connection_state` transitions to `"connected"`, the daemon is back on the relay.
2. **Confirmation:** Wait for the first `telemetry` ops message from the node to confirm it's operational (not just connected).
3. **Timeout:** If no reconnection within 60 seconds, surface an alert in the UI.

**Source:** `server/services/ops-channel.ts` lines 222, 252–260 (`connection_state`), `server/index.ts` line 527 (`/api/ops/status`), `server/services/node-watchdog.ts` lines 27–33 (`computeHealthState` — library, not directly in API response)

---

## Architectural Decisions Required Before Building Daemon Management Features

### Decision 1: Exec vs. Dedicated Message Type
Should daemon restart use the existing `exec` ops message type (send `Restart-Service PlenumNET-Cube-{N}`) or a new dedicated `service-control` message type?

**Recommendation:** Start with `exec` for immediate capability. Add `service-control` in a follow-up task for cleaner semantics, but this is not blocking.

### Decision 2: What happens if the daemon is down?
The relay can only forward ops messages to connected nodes. If the daemon is already crashed and not connected to the relay, the restart command cannot reach it.

**Recommendation:** For nodes with Windows Services configured, the watchdog task handles this automatically (every 5 minutes). For immediate recovery of a disconnected node, the monitor should surface a message: "Node is disconnected — the watchdog will attempt automatic restart within 5 minutes" or instruct the operator to use RDP/SSH to manually restart.

### Decision 3: Which nodes need restart capability?
There are two components that together form the CRS:
1. **Rust daemon in CRS mode** (`CUBE_MODE=crs`) — the Rust binary with geometric routing, FTS, and key management. In the Replit deployment, this runs as a child process of the Express server (auto-spawned, `server/index.ts` lines 2813–2868). In Windows, it runs as an independent Windows Service.
2. **Express relay server** (`server/index.ts`) — the TypeScript server that hosts the WebSocket relay, ops channel, HTTP API routes (`server/routes/inter-cube.ts`), and CRS registration endpoints. In the Replit deployment, it also spawns the Rust daemon.

In Replit, both components run together — restarting the Express server (workflow restart) also restarts the co-located Rust daemon. In Windows, the Rust daemon is independent and managed by Windows Services.

**Recommendation:** The Array3 Monitor should distinguish three restart operations:
1. **Restart remote Rust daemon** — use ops channel `exec` message (Q7)
2. **Restart co-located Rust daemon (Replit)** — restart the Express server workflow (which re-spawns the daemon)
3. **Restart Express relay server** — Replit workflow restart
