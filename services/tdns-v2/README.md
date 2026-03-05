# TDNS v2.3 — Ternary Domain Name System

**27-Dimensional Ontological Addressing for PlenumNET**

*The Address IS the Description. The Description IS the Route. The Description IS a Measurement.*

Capomastro Holdings Ltd. — Applied Physics Division

---

## What Is This

TDNS replaces five conventional internet protocols — DNS, BGP, PKI, IGMP/PIM, and PTP — with a single mathematical structure: a 27-dimensional ternary hypercube.

Every entity on the network receives a 27-trit coordinate derived from machine-measurable properties. No human judgment. No subjective classification. A CRS scanner points at the entity and the address derives itself.

**Key numbers:**
- Address space: 3²⁷ = 7,625,597,484,987 (7.6 trillion)
- Neighbors per node: 54
- Maximum diameter: 27 hops
- Routing tables: zero
- Human input required: zero

## Quick Start

### Scan a Live Target

```bash
cargo run --bin tdns-scan -- https://github.com
```

### Describe an Address

```bash
cargo run --bin tdns-scan -- --describe "WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332"
```

### Compare Two Entities

```bash
cargo run --bin tdns-scan -- --compare github.com google.com
```

### Run the Server

```bash
cargo run --bin tdns-server
# Listening on http://0.0.0.0:3927

curl http://localhost:3927/api/v1/health
curl http://localhost:3927/api/v1/describe/WO:2333%20WA:2333%20WR:2222%20WN:3333%20WY:1221%20HO:2133%20PE:332
```

### Docker

```bash
docker build -t tdns-v2 .
docker run -p 3927:3927 tdns-v2
docker run tdns-v2 tdns-scan https://github.com
```

### 4-Node Mesh

```bash
docker-compose up -d
curl http://localhost:3927/api/v1/health
```

## The 27 Dimensions

Seven questions, 27 measurable properties:

| Category | Trits | Root Question |
|----------|-------|---------------|
| **WHO** | 1–4 | Who is behind it? (Entity type, audience, transparency, hosting) |
| **WHAT** | 5–8 | What is it? (Form factor, content, consumers, intelligence) |
| **WHERE** | 9–12 | Where can I find it? (Visibility, auth, scale, protocol) |
| **WHEN** | 13–16 | When does it operate? (Era, availability, data freshness, latency) |
| **WHY** | 17–20 | Why does it exist? (Payment, data collection, policies, cost) |
| **HOW** | 21–24 | How does it work? (Delivery, flow, updates, persistence) |
| **PEACE** | 25–27 | Can I sleep at night? (Encryption, trackers, audits) |

Every trit is machine-derived from live scans: HTTP probes, DNS records, TLS inspection, header analysis, tracker counting, latency measurement.

## Architecture

### Module Map

```
┌──────────────────────────────────────────────────────────────┐
│  Binaries                                                     │
│    tdns-scan    CLI scanner (scan, compare, describe)         │
│    tdns-server  HTTP server (11 endpoints, port 3927)         │
├──────────────────────────────────────────────────────────────┤
│  Service Layer                                                │
│    api.rs       HTTP request/response types + router          │
│    bridge.rs    Metatronic bridge (.plm → TDNS / legacy DNS)  │
├──────────────────────────────────────────────────────────────┤
│  Inter-Cube Services                                          │
│    crs.rs       Cube Registration Service (scan, derive, TRN) │
│    glb.rs       Geometric Load Balancer (routing, multicast)  │
│    fts.rs       Fault Tolerance Service (heartbeats, dead set) │
│    con.rs       Cube Overlay Network (PQ-encrypted tunnels)   │
├──────────────────────────────────────────────────────────────┤
│  Scanner Pipeline                                             │
│    scanner.rs   Live network probes (27 measurements)         │
│    derive.rs    27 DerivationRule implementations             │
├──────────────────────────────────────────────────────────────┤
│  Core Types                                                   │
│    trit.rs      Atomic ternary digit {1, 2, 3}               │
│    addr.rs      27-trit CubeAddr + wire encoding (7 bytes)   │
│    subcube.rs   Wildcard multicast (base + mask)              │
│    trn.rs       Ternary Resource Name records                 │
│    schema.rs    27 ontological dimensions (WHO→PEACE)         │
│    scan.rs      Scan types + BLAKE3 hash                      │
│    routing.rs   Neighbor maps + greedy forwarding             │
└──────────────────────────────────────────────────────────────┘
```

### Data Flow

```
URL → ScanTarget → collect() → ScanContext → extract_measurements() → 27 RawValues
  → DerivationRules → 27 Trits → CubeAddr → TRN record → CRS registry
                                     ↓
                              NeighborMap (54 entries)
                                     ↓
                              GLB forwarding decisions
                                     ↓
                              CON encrypted tunnels
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/v1/register` | Scan URL, derive address, register TRN |
| `POST` | `/api/v1/scan` | Scan URL → address + 27 dimensions (no registration) |
| `GET` | `/api/v1/resolve/:name` | Name → TRN record |
| `GET` | `/api/v1/address/:addr` | Reverse lookup: address → name |
| `GET` | `/api/v1/describe/:addr` | Plain-English address narration |
| `POST` | `/api/v1/verify/:name` | Re-verification (§9.4): re-scan + drift detection |
| `DELETE` | `/api/v1/deregister/:name` | Remove entity |
| `POST` | `/api/v1/route` | Compute route between two addresses |
| `GET` | `/api/v1/fts/status` | FTS health summary |
| `GET` | `/api/v1/con/metrics` | CON tunnel metrics |
| `GET` | `/api/v1/health` | Service health check |

### Example: Scan

```bash
curl -X POST http://localhost:3927/api/v1/scan \
  -H "Content-Type: application/json" \
  -d '{"url": "https://github.com"}'
```

Response:
```json
{
  "status": "ok",
  "url": "https://github.com",
  "address": "WO:2323 WA:2133 WR:3131 WN:3322 WY:1231 HO:3123 PE:332",
  "hptp_mandatory": false,
  "dimensions": [
    {"number": 1, "category": "WO", "question": "What kind?", "value": 2, "label": "Corporate"}
  ]
}
```

### Example: Register

```bash
curl -X POST http://localhost:3927/api/v1/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "myapp.capomastro.plm",
    "zone": "capomastro.plm",
    "public_key_hex": "deadbeef",
    "url": "https://myapp.example.com"
  }'
```

### Example: Route

```bash
curl -X POST http://localhost:3927/api/v1/route \
  -H "Content-Type: application/json" \
  -d '{
    "source": "WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313",
    "destination": "WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332"
  }'
```

## Address Formats

Three interconvertible representations:

| Format | Example | Use |
|--------|---------|-----|
| Human name | `pptpro.capomastro.plm` | Registration, resolution |
| Category-grouped | `WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332` | Debugging, display |
| Canonical wire | `233.323.322.222.333.312.121.331.332` | Wire protocol |

Binary wire encoding: 27 trits × 2 bits = 54 bits → 7 bytes (2-bit padding).

## Key Protocol Mechanics

### Greedy Forwarding (§11.1)
Compare source and destination trit-by-trit. Find first differing dimension. Flip to match. Forward. Repeat. Path length = Hamming distance. Loop-free. No routing tables.

### HPTP Enforcement (§10.4)
Trits 15 (Live data) + 16 (Real-time) both = 3 → HPTP-mandatory. CRS verifies sync at registration (≤1μs tolerance). GLB drops packets to degraded nodes. 5-second hold-down after recovery.

### Self-Certifying Names (§9)
Public key in TRN, not address. Ownership via challenge-response. Scan hash (BLAKE3) binds address to measurements. Any party can re-verify via §9.4 open protocol.

### Zero-Cleartext Fabric (§2.6)
All inter-cube traffic encrypted via CON tunnels. BLAKE3 key derivation per neighbor link. Trit 25 measures what the entity offers end users, not the fabric transport. Double encryption for regulated entities.

### Sub-cube Multicast (§11.3)
Wildcard address (base + mask) defines a sub-cube. Forward along unconstrained dimensions. Natural spanning tree. Zero additional state. No IGMP. No PIM.

## Testing

```bash
# Unit tests (148)
cargo test --lib

# Integration tests (12 end-to-end scenarios)
cargo test --test integration

# E2E tests (9 service-level tests)
cargo test --test e2e

# All tests
cargo test
```

## What TDNS Replaces

| Protocol | Conventional Role | TDNS Equivalent |
|----------|-------------------|-----------------|
| DNS | Name → IP | Name → 27-trit coordinate via TRN |
| BGP/OSPF | Routing tables | Greedy forwarding; neighbor maps only |
| PKI/CA | Certificate authorities | Challenge-response + scan hash binding |
| IGMP/PIM | Multicast groups | Sub-cube via dimensional wildcards |
| PTP/NTP | Time synchronization | HPTP nanosecond timestamps |

Five protocol systems collapsed into the geometry of a 27-dimensional ternary hypercube.

## License

Proprietary — Capomastro Holdings Ltd. All rights reserved.

## Specification

See `TDNS_v2.3_Specification.md` for the complete protocol specification.
