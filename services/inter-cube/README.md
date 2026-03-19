# PlenumNET Inter-Cube Infrastructure Services

> Applied Physics Division — Capomastro Holdings Ltd.

13-dimensional ternary cube network with geometric routing, PQ-native tunnel keys, and fault-tolerant heartbeat monitoring.

## Architecture

Four services compose the Inter-Cube stack:

| Service | Acronym | Purpose |
|---------|---------|---------|
| Cube Registration Service | CRS | Address allocation, neighbor discovery, heartbeat tracking |
| Cube Overlay Network | CON | PQ-encrypted tunnel management via TL-Sponge-385 key derivation |
| Fault Tolerance Service | FTS | Heartbeat monitoring, suspect/dead detection, recovery |
| Geometric Load Balancer | GLB | Shortest-path forwarding across the 13D ternary cube topology |

## Operating Modes

### CRS Mode (`CUBE_MODE=crs`)
Runs the central coordinator. Allocates addresses, accepts cube registrations, serves the full REST API (11 routes).

### Cube Mode (`CUBE_MODE=cube`)
Runs a worker cube. Registers with a remote CRS on boot, heartbeats every 30s, serves a stats-only API (8 routes).

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CUBE_MODE` | `all` | Operating mode: `crs`, `cube`, or `all` |
| `CUBE_CRS_URL` | — | CRS base URL (required for cube mode) |
| `CUBE_ENDPOINT` / `ADDRESS` | `0.0.0.0:51820` | This cube's wire protocol endpoint |
| `CUBE_ROLE` / `ROLE` | — | Role annotation (inference, review, kb, infra, relay, standby) |
| `CUBE_API_PORT` / `API_PORT` | `8080` | HTTP API bind port |
| `RUST_LOG` | `info` | Log level |
| `PLENUM_REQUIRE_SIGNATURE` | `false` | Require TL-DSA signatures on registrations |
| `PLENUM_ENABLE_RATE_LIMIT` | `false` | Enable per-IP rate limiting with PoW |
| `PLENUM_POW_K` | `5` | Proof-of-work leading zero trits |
| `PLENUM_ENABLE_DUAL_CHECKSUM` | `false` | mod-364 + mod-333 wire checksum |
| `PLENUM_ENABLE_WIRE_ECC` | `false` | 8-trit ECC syndrome on addresses |
| `PLENUM_PROTOCOL_VERSION` | `2` | Wire protocol version to emit |

## Ports

| Port | Protocol | Service |
|------|----------|---------|
| 8080 | TCP/HTTP | REST API |
| 51820 | UDP | Cube-to-cube wire protocol |

## Build

### Local (Rust)

```bash
cd services/inter-cube
cargo build --release --bin inter-cube-daemon
```

### Docker

```bash
docker build -f services/inter-cube/Dockerfile -t plenum-inter-cube .
```

## Run

### Single Node

```bash
CUBE_MODE=crs ./target/release/inter-cube-daemon
```

### 4-Node Docker Compose

```bash
cd services/inter-cube
docker compose up -d
```

Services:
- `crs` — Central coordinator (port 8080)
- `cube-1` — Worker cube (port 8081)
- `cube-2` — Worker cube (port 8082)
- `cube-3` — Worker cube (port 8083)

### 27-Node Docker Compose

```bash
cd services/inter-cube
docker compose -f docker-compose.27-node.yml up -d
```

Full deployment with role-annotated nodes:
- 1 CRS coordinator
- 8 inference nodes
- 4 review nodes
- 4 knowledge base nodes
- 4 infrastructure nodes
- 4 relay nodes
- 2 standby nodes

## API Endpoints

### Health Check
```
GET /health
```

### CRS (CRS mode only)
```
GET  /api/salvi/inter-cube/crs/stats
POST /api/salvi/inter-cube/crs/register
POST /api/salvi/inter-cube/crs/heartbeat
```

### GLB
```
POST /api/salvi/inter-cube/glb/forward
GET  /api/salvi/inter-cube/glb/stats
```

### CON
```
GET /api/salvi/inter-cube/con/stats
```

### FTS
```
GET /api/salvi/inter-cube/fts/status
GET /api/salvi/inter-cube/fts/dead
```

### Topology
```
GET  /api/salvi/inter-cube/topology
POST /api/salvi/inter-cube/address/validate
```

## Tests

```bash
cd services/inter-cube
cargo test
```

420 tests across 15 modules.

## Address Table (TM-2026-020.2 §5.2)

The 27-node deployment assigns deterministic wire endpoints via `ADDRESS` env vars.
Ternary cube addresses (54-trit) are allocated dynamically by CRS upon registration.
The mapping between service names and roles:

| Service | Role | Wire Endpoint | API Port |
|---------|------|---------------|----------|
| `crs` | coordinator | — | 8080 |
| `cube-inf-01`..`cube-inf-08` | inference | `cube-inf-{N}:51820` | 8101–8108 |
| `cube-rev-01`..`cube-rev-04` | review | `cube-rev-{N}:51820` | 8111–8114 |
| `cube-kb-01`..`cube-kb-04` | kb | `cube-kb-{N}:51820` | 8121–8124 |
| `cube-infra-01`..`cube-infra-04` | infra | `cube-infra-{N}:51820` | 8131–8134 |
| `cube-relay-01`..`cube-relay-04` | relay | `cube-relay-{N}:51820` | 8141–8144 |
| `cube-standby-01`..`cube-standby-02` | standby | `cube-standby-{N}:51820` | 8151–8152 |

CRS assigns 54-trit addresses from the 3^13 address space upon registration.
Docker service names serve as DNS-resolvable hostnames within the compose network.

## Known Gaps

- NAT traversal / rathole integration not yet implemented (TM-2026-020.2 §4.10)
- Wire protocol runs over plaintext UDP; TLS/DTLS wrapping planned for production
- 27-node deployment requires target hardware validation
