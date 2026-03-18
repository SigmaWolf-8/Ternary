# PlenumNET → PlenumLAN API Mapping Document

**Reference:** TM-2026-019.2 (PlenumLAN Technical Manifest — Unified)
**Capomastro Holdings Ltd. — Applied Physics Division**
**Generated: March 2026**

---

## Summary

| Category | Count |
|---|---|
| **Mapped endpoints** (PlenumNET → PlenumLAN equivalent) | 79 |
| **Showcase-only endpoints** (stay in plenumnet.replit.app) | 195 |
| **Total existing endpoints** | 274 |
| **Gap items** (PlenumLAN net-new, no existing API) | 24 |

*Inventory method: endpoint counts derived from explicit `app.METHOD(path)`, `router.METHOD(path)`, and `app.use(path)` proxy-mount declarations across `server/routes.ts` and 18 files in `server/routes/*.ts` (excluding `middleware.ts` which has 0 endpoints). Router-mounted files (e.g., `inter-cube.ts` mounted at `/api/salvi/inter-cube`) list fully-resolved paths in the table. The PQTI catch-all proxy (`app.use("/api/pqti", ...)`) is inventoried as `ALL /api/pqti/*` alongside its companion `GET /api/pqti-status`. Each declared endpoint appears exactly once in either Section 1 (Mapped) or Section 3 (Showcase-only). Gap items (Section 2) represent planned PlenumLAN features with no existing PlenumNET API and are excluded from the endpoint total.*

---

# Section 1: Mapped Endpoints

Each existing PlenumNET API endpoint that has a corresponding PlenumLAN Rust module, the TM-2026-019.2 section it fulfills, and notes on what changes in the Rust rewrite.

## 1.1 TDNS → TDNS-L (Local Name Resolver)

| Method | Path | Source File | Target PlenumLAN Handler | TM §Ref | Delta Note |
|---|---|---|---|---|---|
| GET | `/api/tdns/health` | `server/routes/tdns.ts` | `plenumlan/src/tdns_l/health.rs` | §5–§7 | Identical health-check pattern; version changes to TDNS-L |
| POST | `/api/tdns/scan` | `server/routes/tdns.ts` | `plenumlan/src/tdns_l/scan.rs` | §7 | HTTP-fetch scanner identical for `.plm.local` service endpoints; physical LAN entities use template-driven scan (§7.8) instead of HTTP fetch |
| POST | `/api/tdns/register` | `server/routes/tdns.ts` | `plenumlan/src/crs_l/register.rs` | §10.1 | Registration moves to CRS-L; adds TL-DSA keygen, IP derivation (§6), heartbeat init, capability evaluation |
| GET | `/api/tdns/resolve/:name` | `server/routes/tdns.ts` | `plenumlan/src/tdns_l/resolve.rs` | §6.7 | Adds dual-stack resolution (A/AAAA/TDNS-L native); `.plm.local` TLD triggers host-ID interpretation of second 27 trits |
| GET | `/api/tdns/list` | `server/routes/tdns.ts` | `plenumlan/src/crs_l/list.rs` | §10 | Registry listing sourced from CRS-L entity store instead of in-memory map |
| POST | `/api/tdns/org/create` | `server/routes/tdns.ts` | `plenumlan/src/pds/org.rs` | §10.3 | Org entities become PDS directory objects; creation requires TL-DSA-signed capability |
| POST | `/api/tdns/org/add-url` | `server/routes/tdns.ts` | `plenumlan/src/pds/org.rs` | §10.3 | Member association via CRS-L entity linking |
| GET | `/api/tdns/org/:name` | `server/routes/tdns.ts` | `plenumlan/src/pds/org.rs` | §10.3 | Org detail from PDS directory query |
| GET | `/api/tdns/orgs` | `server/routes/tdns.ts` | `plenumlan/src/pds/org.rs` | §10.3 | Org listing from PDS |

## 1.2 Inter-Cube CRS → CRS-L (Local Cube Registration Service)

| Method | Path | Source File | Target PlenumLAN Handler | TM §Ref | Delta Note |
|---|---|---|---|---|---|
| POST | `/api/salvi/inter-cube/crs/register` | `server/routes/inter-cube.ts` | `plenumlan/src/crs_l/register.rs` | §10.1 | LAN version adds IP derivation (§6), scan template classification (§7.8), and auto-capability evaluation |
| GET | `/api/salvi/inter-cube/crs/lookup/:address` | `server/routes/inter-cube.ts` | `plenumlan/src/crs_l/lookup.rs` | §10 | CRS-L lookup returns full dual-stack addresses + classification metadata |
| GET | `/api/salvi/inter-cube/crs/neighbors/:address` | `server/routes/inter-cube.ts` | `plenumlan/src/crs_l/neighbors.rs` | §8.1 | Neighbors used for nearest-service routing (e.g., closest printer) |
| POST | `/api/salvi/inter-cube/crs/heartbeat` | `server/routes/inter-cube.ts` | `plenumlan/src/crs_l/heartbeat.rs` | §10.1, §8.4 | HPTP-timestamped heartbeats; FTS health state transitions (Up→Suspect→Down→Recovering) |
| POST | `/api/salvi/inter-cube/crs/deregister` | `server/routes/inter-cube.ts` | `plenumlan/src/crs_l/deregister.rs` | §10 | Frees vertex in cube bitmap allocator; revokes all capability tokens |
| GET | `/api/salvi/inter-cube/crs/stats` | `server/routes/inter-cube.ts` | `plenumlan/src/crs_l/stats.rs` | §10 | Registry stats include vertex occupancy bitmap utilization |

## 1.3 Inter-Cube GLB/CON/FTS → PlenumLAN Infrastructure Services

| Method | Path | Source File | Target PlenumLAN Handler | TM §Ref | Delta Note |
|---|---|---|---|---|---|
| POST | `/api/salvi/inter-cube/glb/forward` | `server/routes/inter-cube.ts` | `plenumlan/src/routes/glb.rs` | §8.1, §12 | Active only when second site onboards (promotion path); LAN-internal routing uses direct cube adjacency |
| GET | `/api/salvi/inter-cube/glb/stats` | `server/routes/inter-cube.ts` | `plenumlan/src/routes/glb.rs` | §8.1 | Same metrics; inactive until multi-site |
| GET | `/api/salvi/inter-cube/glb/health` | `server/routes/inter-cube.ts` | `plenumlan/src/routes/glb.rs` | §8.1 | Same |
| GET | `/api/salvi/inter-cube/con/neighbors` | `server/routes/inter-cube.ts` | `plenumlan/src/routes/con.rs` | §12 | Tunnels activate on second-site onboarding; 26 tunnels per cube |
| GET | `/api/salvi/inter-cube/con/stats` | `server/routes/inter-cube.ts` | `plenumlan/src/routes/con.rs` | §12 | Same |
| POST | `/api/salvi/inter-cube/con/tunnel/refresh` | `server/routes/inter-cube.ts` | `plenumlan/src/routes/con.rs` | §12 | Same |
| POST | `/api/salvi/inter-cube/con/tunnel/upgrade-key` | `server/routes/inter-cube.ts` | `plenumlan/src/routes/con.rs` | §12 | Same TLSponge-385 key derivation |
| GET | `/api/salvi/inter-cube/fts/status` | `server/routes/inter-cube.ts` | `plenumlan/src/crs_l/fts.rs` | §10.1 | FTS health states for LAN entities (heartbeat monitoring) |
| GET | `/api/salvi/inter-cube/fts/dead` | `server/routes/inter-cube.ts` | `plenumlan/src/crs_l/fts.rs` | §10.1 | Dead/suspect entity listing |
| POST | `/api/salvi/inter-cube/fts/config` | `server/routes/inter-cube.ts` | `plenumlan/src/crs_l/fts.rs` | §10.1 | FTS tuning (miss thresholds, recovery periods) |
| POST | `/api/salvi/inter-cube/routing/compute` | `server/routes/inter-cube.ts` | `plenumlan/src/routes/routing.rs` | §8.1 | Pure math Hamming distance/path computation — identical logic |
| POST | `/api/salvi/inter-cube/address/validate` | `server/routes/inter-cube.ts` | `plenumlan/src/address/rep_c.rs` | §5 | Rep C validation with zero-sentinel forgery detection — identical |
| GET | `/api/salvi/inter-cube/topology` | `server/routes/inter-cube.ts` | `plenumlan/src/routes/topology.rs` | §8 | Architectural constants (13D, 3¹³ vertices, etc.) |

## 1.4 Salvi Core Crypto → PlenumLAN Kernel (Direct Rust Calls)

These endpoints exist in PlenumNET as HTTP APIs because the TypeScript server needs to expose Rust crypto logic over HTTP. In PlenumLAN (pure Rust), these become direct function calls — no HTTP layer. However, the web console still needs API access, so thin Axum handlers wrap the same Rust functions.

| Method | Path | Source File | Target PlenumLAN Handler | TM §Ref | Delta Note |
|---|---|---|---|---|---|
| POST | `/api/salvi/crypto/hash` | `server/routes/salvi.ts` | `plenumlan/src/routes/crypto.rs` | §8.3 | TL-Sponge-385 hash — direct Rust call, no TS↔Rust bridge |
| POST | `/api/salvi/crypto/tl-dsa/keygen` | `server/routes/salvi.ts` | `plenumlan/src/routes/crypto.rs` | §8.3, §10.1 | TL-DSA keygen at entity registration; direct Rust |
| POST | `/api/salvi/crypto/tl-dsa/sign` | `server/routes/salvi.ts` | `plenumlan/src/routes/crypto.rs` | §8.3, §10.2 | TL-DSA sign for capability tokens and auth |
| POST | `/api/salvi/crypto/tl-dsa/verify` | `server/routes/salvi.ts` | `plenumlan/src/routes/crypto.rs` | §8.3, §10.2 | TL-DSA verify — used in every auth challenge |
| GET | `/api/salvi/crypto/tl-dsa/spec` | `server/routes/salvi.ts` | `plenumlan/src/routes/crypto.rs` | §8.3 | Spec metadata |
| GET | `/api/salvi/crypto/tl-kem/spec` | `server/routes/salvi.ts` | `plenumlan/src/routes/crypto.rs` | §8.3, §12 | TL-KEM for CON tunnel key exchange |
| POST | `/api/salvi/phase/split` | `server/routes/salvi.ts` | `plenumlan/src/routes/crypto.rs` | §8.3 | Phase encryption for PFS data at rest |
| POST | `/api/salvi/phase/recombine` | `server/routes/salvi.ts` | `plenumlan/src/routes/crypto.rs` | §8.3 | Phase decryption |
| GET | `/api/salvi/phase/config/:mode` | `server/routes/salvi.ts` | `plenumlan/src/routes/crypto.rs` | §8.3 | Mode configuration |
| GET | `/api/salvi/phase/recommend` | `server/routes/salvi.ts` | `plenumlan/src/routes/crypto.rs` | §8.3 | Mode recommendation |
| POST | `/api/salvi/phase/batch/split` | `server/routes/salvi.ts` | `plenumlan/src/routes/crypto.rs` | §8.3 | Batch phase encrypt |
| POST | `/api/salvi/phase/batch/recombine` | `server/routes/salvi.ts` | `plenumlan/src/routes/crypto.rs` | §8.3 | Batch phase decrypt |

## 1.5 Timing → HPTP (Femtosecond Timing)

| Method | Path | Source File | Target PlenumLAN Handler | TM §Ref | Delta Note |
|---|---|---|---|---|---|
| GET | `/api/salvi/timing/timestamp` | `server/routes/salvi.ts` | `plenumlan/src/routes/timing.rs` | §8.4 | HPTP timestamp — direct Rust; used for token expiration and audit |
| GET | `/api/salvi/timing/metrics` | `server/routes/salvi.ts` | `plenumlan/src/routes/timing.rs` | §8.4 | Timing metrics |
| GET | `/api/salvi/timing/self-test` | `server/routes/salvi.ts` | `plenumlan/src/routes/timing.rs` | §8.4 | Diagnostic self-test |
| GET | `/api/salvi/timing/error-budget` | `server/routes/salvi.ts` | `plenumlan/src/routes/timing.rs` | §8.4 | Drift and jitter reporting |
| GET | `/api/salvi/timing/batch/:count` | `server/routes/salvi.ts` | `plenumlan/src/routes/timing.rs` | §8.4 | Batch timestamps |

## 1.6 Capability Tokens → PlenumLAN Capability System

| Method | Path | Source File | Target PlenumLAN Handler | TM §Ref | Delta Note |
|---|---|---|---|---|---|
| POST | `/api/capabilities/issue` | `server/routes/capabilities.ts` | `plenumlan/src/pds/capabilities.rs` | §8.5, §10.1 | Core issuance; in PlenumLAN, auto-triggered by CRS-L issuance rules on cube regions |
| POST | `/api/capabilities/validate` | `server/routes/capabilities.ts` | `plenumlan/src/pds/capabilities.rs` | §8.5 | Validation called by every protocol bridge (PFS, RADIUS shim, LDAP shim) |
| POST | `/api/capabilities/delegate` | `server/routes/capabilities.ts` | `plenumlan/src/pds/capabilities.rs` | §8.5 | HMAC-chained delegation replaces AD group membership |
| POST | `/api/capabilities/delegate/chain` | `server/routes/capabilities.ts` | `plenumlan/src/pds/capabilities.rs` | §8.5 | Multi-hop delegation |
| POST | `/api/capabilities/verify-chain` | `server/routes/capabilities.ts` | `plenumlan/src/pds/capabilities.rs` | §8.5 | Chain integrity verification |
| GET | `/api/capabilities/audit` | `server/routes/capabilities.ts` | `plenumlan/src/pds/capabilities.rs` | §8.5 | Capability audit statistics |
| POST | `/api/capabilities/hardware/register` | `server/routes/capabilities.ts` | `plenumlan/src/pds/hardware_bind.rs` | §8.5, §16 | Device registration for WebAuthn/hardware binding |
| POST | `/api/capabilities/hardware/challenge` | `server/routes/capabilities.ts` | `plenumlan/src/pds/hardware_bind.rs` | §10.2, §16 | HPTP-based challenge for remote auth |
| POST | `/api/capabilities/hardware/verify` | `server/routes/capabilities.ts` | `plenumlan/src/pds/hardware_bind.rs` | §10.2 | Challenge-response verification |
| POST | `/api/capabilities/hardware/issue` | `server/routes/capabilities.ts` | `plenumlan/src/pds/hardware_bind.rs` | §8.5 | Hardware-bound token issuance |
| POST | `/api/capabilities/certificate/issue` | `server/routes/capabilities.ts` | `plenumlan/src/pds/certificates.rs` | §8.3 | RFC 3161 certificate for capabilities |
| POST | `/api/capabilities/certificate/verify` | `server/routes/capabilities.ts` | `plenumlan/src/pds/certificates.rs` | §8.3 | Certificate verification |
| GET | `/api/capabilities/certificate/:certId/rfc3161` | `server/routes/capabilities.ts` | `plenumlan/src/pds/certificates.rs` | §8.3 | RFC 3161 TSR export |
| GET | `/api/capabilities/status` | `server/routes/capabilities.ts` | `plenumlan/src/pds/capabilities.rs` | §8.5 | System status |

## 1.7 TSA → PlenumLAN Audit Fabric

| Method | Path | Source File | Target PlenumLAN Handler | TM §Ref | Delta Note |
|---|---|---|---|---|---|
| POST | `/api/tsa/timestamp` | `server/routes/tsa.ts` | `plenumlan/src/pds/audit.rs` | §8.4 | RFC 3161 timestamping feeds Merkle-chained audit fabric |
| POST | `/api/tsa/timestamp/json` | `server/routes/tsa.ts` | `plenumlan/src/pds/audit.rs` | §8.4 | JSON-based timestamp requests |
| POST | `/api/tsa/verify` | `server/routes/tsa.ts` | `plenumlan/src/pds/audit.rs` | §8.4 | Token verification |
| GET | `/api/tsa/certificate` | `server/routes/tsa.ts` | `plenumlan/src/pds/audit.rs` | §8.3 | TSA certificate info |
| GET | `/api/tsa/health` | `server/routes/tsa.ts` | `plenumlan/src/pds/audit.rs` | §8.4 | Health check |
| GET | `/api/tsa/audit/query` | `server/routes/tsa.ts` | `plenumlan/src/pds/audit.rs` | §8.4 | Audit record query — Merkle-chained in PlenumLAN |

## 1.8 Security → PlenumLAN Security Dashboard

| Method | Path | Source File | Target PlenumLAN Handler | TM §Ref | Delta Note |
|---|---|---|---|---|---|
| POST | `/api/security/audit` | `server/routes/security.ts` | `plenumlan/src/pds/security.rs` | §11 | Security event logging to Merkle audit fabric |
| GET | `/api/security/audit` | `server/routes/security.ts` | `plenumlan/src/pds/security.rs` | §11 | Event retrieval |
| GET | `/api/security/audit/summary` | `server/routes/security.ts` | `plenumlan/src/pds/security.rs` | §11 | Severity summary |
| GET | `/api/security/audit/stats` | `server/routes/security.ts` | `plenumlan/src/pds/security.rs` | §11 | Audit statistics |
| GET | `/api/security/dashboard` | `server/routes/security.ts` | `plenumlan/src/pds/security.rs` | §11, §18 | Unified dashboard (Console Screen: Security Overview) |
| GET | `/api/security/kri` | `server/routes/security.ts` | `plenumlan/src/pds/security.rs` | §11 | Key Risk Indicators |

## 1.9 Ternary Operations → PlenumLAN Kernel (Direct Calls)

| Method | Path | Source File | Target PlenumLAN Handler | TM §Ref | Delta Note |
|---|---|---|---|---|---|
| POST | `/api/salvi/ternary/convert` | `server/routes/salvi.ts` | `plenumlan/src/routes/ternary.rs` | §5.1 | Rep A/B/C conversion — direct Rust; thin Axum handler for console |
| POST | `/api/salvi/ternary/add` | `server/routes/salvi.ts` | `plenumlan/src/routes/ternary.rs` | §4 | GF(3) addition |
| POST | `/api/salvi/ternary/multiply` | `server/routes/salvi.ts` | `plenumlan/src/routes/ternary.rs` | §4 | GF(3) multiplication |
| POST | `/api/salvi/ternary/rotate` | `server/routes/salvi.ts` | `plenumlan/src/routes/ternary.rs` | §4 | Bijective rotation |
| POST | `/api/salvi/ternary/not` | `server/routes/salvi.ts` | `plenumlan/src/routes/ternary.rs` | §4 | Ternary NOT |
| POST | `/api/salvi/ternary/xor` | `server/routes/salvi.ts` | `plenumlan/src/routes/ternary.rs` | §4 | Ternary XOR |

## 1.10 Health & Platform

| Method | Path | Source File | Target PlenumLAN Handler | TM §Ref | Delta Note |
|---|---|---|---|---|---|
| GET | `/api/health` | `server/routes.ts` | `plenumlan/src/routes/health.rs` | §18 | System health for console System Overview screen |
| GET | `/api/salvi/docs` | `server/routes/salvi.ts` | `plenumlan/src/routes/docs.rs` | §18 | API documentation endpoint |

---

# Section 2: Gap Analysis

PlenumLAN features required by TM-2026-019.2 that have **NO existing API equivalent** in the current PlenumNET app. These are net-new Rust implementations.

| # | Feature | TM §Ref | Description | Release | Complexity |
|---|---|---|---|---|---|
| G1 | **PFS — Plenum File Service (SMB 3.1.1 + NFS v4.2)** | §9.1 | Full file protocol bridge: SMB/NFS → capability-mediated access against cube-addressed storage. Includes backup snapshots, update distribution, print driver distribution. The largest net-new build. | 1.0 | **Large** |
| G2 | **RADIUS Shim** | §9.2.1 | RADIUS Access-Request → TL-DSA challenge; capability tokens → RADIUS attributes (VLAN=shell, ACL=capability scope). 3 endpoints: `/api/radius/auth`, `/api/radius/acct`, `/api/radius/status`. | 1.0 | **Medium** |
| G3 | **LDAP Compatibility Shim** | §9.2.2 | LDAP bind/search/compare → CRS cube queries. Read-only; LDAP writes → PDS API calls with TL-DSA-signed capabilities. 4 endpoints. | 1.0 | **Medium** |
| G4 | **Legacy DHCP Responder** | §9.2.3 | DHCP DISCOVER → scan template assignment → CRS registration → IP derivation → DHCP OFFER/ACK. 2 endpoints: `/crs/dhcp-discover`, `/crs/dhcp-request`. | 1.0 (0.1 partial) | **Medium** |
| G5 | **Print Bridge (CUPS integration)** | §9.3 | CUPS wrapping with cube-native discovery (D5=3 query) and capability-gated authorization. Drivers served from PFS, TL-DSA-signed. | 1.0 | **Medium** |
| G6 | **PDS — Plenum Directory Service** | §10 | User enrollment, cryptographic login (TL-DSA challenge-response), session management, capability lifecycle, group delegation chains. Replaces Active Directory. | 0.5 | **Large** |
| G7 | **54-Trit Dual-Interpretation Address Module** | §5 | Bidirectional IP↔ternary bijection with host integer intermediary. Rep C parse with zero-sentinel forgery detection. Direction A (ternary→IP) and Direction B (IP→ternary). | 0.1 | **Medium** |
| G8 | **LAN Ontological Scan (27-Dimension)** | §7 | MAC OUI/DHCP fingerprint, mDNS, SNMP, LLDP, port scan — all LAN-native signals (not HTTP fetch). 6 scan templates (workstation, server, infrastructure, printer, IoT, service). `gf3` quantitative threshold formula. | 0.1 | **Large** |
| G9 | **Site Network Configuration** | §6.2 | First-run auto-detection of IPv4 prefix, host range, IPv6 ULA prefix, shell-to-VLAN mapping. Stored once, referenced by all IP derivation. | 0.1 | **Small** |
| G10 | **First-Run Setup Wizard** | §14.4 | 3 screens: site name, network settings (auto-detect), admin account (TL-DSA keygen). Behind the scenes: root key generation, resolver init, CRS init, first entity registration, network scan start. | 0.1 | **Medium** |
| G11 | **Download/Installer Pipeline** | §14 | Cross-compiled installers for Windows (.exe), Debian (.deb), RPM (.rpm), macOS (.dmg), standalone Linux binary, ARM binary. CI pipeline with `cargo build --release` for 6 targets. | 0.1 | **Medium** |
| G12 | **Automatic Update System** | §14.6 | Periodic version check against PlenumLAN.replit.app, TL-DSA signature verification of new binary, self-update with zero data loss, automatic restart. | 2.0 | **Medium** |
| G13 | **PlenumDB — Native Storage Backend** | §19.5 | Purpose-built key-value store: 27-trit entity lookup, Merkle-tree audit chain traversal, vertex occupancy bitmap (1,594,323-bit). WAL crash safety with crash-injection test harness. | 2.0 | **Large** |
| G14 | **PTS — Plenum Tunnel Service** | §16 | Phase-encrypted remote console access. WebAuthn/FIDO2 enrollment, TL-DSA challenge-response, scoped tunnel (console only, not full network). Per-service tunnels with individual capability tokens. | 2.0 | **Large** |
| G15 | **Management Console: 16 Screens** | §18 | React web console: System Overview, Entity Registry, Entity Detail, Name Browser, Address Allocation, Permission Rules, Delegation Chains, Identity Dashboard, Shared Folders, Printer Devices, Audit Log, Security Overview, Network Access, System Health, Cube Visualizer, Scan Workflow. | 0.1–1.0 | **Large** |
| G16 | **Windows Server Importer** | §19.2 | Reads existing Active Directory (LDAP export) and creates matching CRS-L entries with capability equivalents for group memberships and GPOs. | 0.5 | **Medium** |
| G17 | **Cube Bitmap Allocator** | §10.1 | flatIndex/fromFlatIndex bijection for 3¹³ = 1,594,323 vertex occupancy tracking. Fast scan for next available vertex. | 0.1 | **Small** |
| G18 | **Cross-Shell Authorization** | §8.2 | Axis 12 shell lookup (Inner/Void/Outer → VLAN 10/20/30). Single-trit security zone determination. Shell-transition capability tokens. | 0.5 | **Small** |
| G19 | **Issuance Rules Engine** | §8.5 | CRS metadata on cube regions: "all entities with address prefix P receive capability set C." Evaluated at registration time. The token set IS the applied GPO. | 0.5 | **Medium** |
| G20 | **Merkle-Chained Audit Fabric** | §8.4 | Every event HPTP-timestamped and Merkle-chained. TIS-27 integrity per entry. Tamper-evident, immutable log with chain verification. | 0.5 | **Medium** |
| G21 | **Nearest-Service Routing** | §8.1 | Hamming-distance query: "find printer with smallest distance from requesting workstation." Classification-based discovery (D5=3 → all printers). | 0.1 | **Small** |
| G22 | **IPv4 Collision Resolution Table** | §6.5–§6.6 | CRS override table for /24 subnets where modular reduction causes collisions. IPv4 preservation for existing devices scanned via Direction B. | 0.1 | **Small** |
| G23 | **Dual-Stack DNS Resolver** | §6.7 | Serves A (IPv4), AAAA (IPv6), and TDNS-L native queries from CRS records. Listens on port 53. Replaces Windows DNS Server. | 0.1 | **Medium** |
| G24 | **Emergency Console (Text-Only)** | §17.3 | USB keyboard + VGA/serial text display: read-only system status, FTS health, last Merkle audit hash, scrollable logs. No shell, no login. Stage 3 only. | 3.0 | **Small** |

### Complexity Summary

| Complexity | Count | Items |
|---|---|---|
| **Large** | 6 | PFS, PDS, LAN Scan, PlenumDB, PTS, 16-Screen Console |
| **Medium** | 12 | RADIUS, LDAP, DHCP, Print Bridge, Address Module, First-Run Wizard, Installer, Auto-Update, AD Importer, Issuance Rules, Merkle Audit, DNS Resolver |
| **Small** | 6 | Site Config, Bitmap Allocator, Cross-Shell Auth, Nearest-Service, IPv4 Collision Table, Emergency Console |

---

# Section 3: PlenumNET-Only Endpoints

Current PlenumNET APIs that do **NOT transfer** to PlenumLAN and remain exclusive to **plenumnet.replit.app**. These are production endpoints for developer tools, third-party integrations, or platform services that serve the PlenumNET platform, not the LAN management product.

## 3.1 Kong Gateway Management

| Method | Path | Source File | Reason |
|---|---|---|---|
| GET | `/api/kong/status` | `server/routes/kong.ts` | Kong Konnect is a PlenumNET platform API gateway; PlenumLAN has no external API gateway |
| GET | `/api/kong/organization` | `server/routes/kong.ts` | Same |
| GET | `/api/kong/control-planes` | `server/routes/kong.ts` | Same |
| GET | `/api/kong/control-planes/:cpId/services` | `server/routes/kong.ts` | Same |
| GET | `/api/kong/control-planes/:cpId/routes` | `server/routes/kong.ts` | Same |
| GET | `/api/kong/control-planes/:cpId/plugins` | `server/routes/kong.ts` | Same |
| GET | `/api/kong/config` | `server/routes/kong.ts` | Same |
| POST | `/api/kong/control-planes/:cpId/services` | `server/routes/kong.ts` | Same |
| POST | `/api/kong/control-planes/:cpId/services/:serviceId/routes` | `server/routes/kong.ts` | Same |
| POST | `/api/kong/control-planes/:cpId/services/:serviceId/plugins` | `server/routes/kong.ts` | Same |
| POST | `/api/kong/control-planes/:cpId/sync-plenumnet` | `server/routes/kong.ts` | Same |
| POST | `/api/kong/sync-all-control-planes` | `server/routes/kong.ts` | Same |
| GET | `/api/kong/service-catalog` | `server/routes/kong.ts` | Same |
| POST | `/api/kong/save-to-github` | `server/routes/kong.ts` | Same |
| GET | `/api/kong/control-planes/:cpId/deploy-instructions` | `server/routes/kong.ts` | Same |
| POST | `/api/kong/control-planes/:cpId/generate-deployment` | `server/routes/kong.ts` | Same |
| POST | `/api/kong/control-planes/:cpId/deploy-to-cloud` | `server/routes/kong.ts` | Same |

## 3.2 SFK Operations Pipeline

| Method | Path | Source File | Reason |
|---|---|---|---|
| POST | `/api/sfk/v1/operations` | `server/routes/sfk-operations.ts` | Platform-specific batch processing workflow; PlenumLAN uses direct Rust calls |
| GET | `/api/sfk/v1/operations/:id` | `server/routes/sfk-operations.ts` | Same |
| GET | `/api/sfk/v1/operations` | `server/routes/sfk-operations.ts` | Same |
| DELETE | `/api/sfk/v1/operations/:id` | `server/routes/sfk-operations.ts` | Same |
| GET | `/api/sfk/v1/stats` | `server/routes/sfk-operations.ts` | Same |

## 3.3 Tonal Field System

| Method | Path | Source File | Reason |
|---|---|---|---|
| GET | `/api/tonal/field` | `server/routes/tonal-field.ts` | Network diffusion research tool; not a LAN management feature |
| GET | `/api/tonal/neighbors` | `server/routes/tonal-field.ts` | Same |
| POST | `/api/tonal/packet` | `server/routes/tonal-field.ts` | Same |
| GET | `/api/resonance/status` | `server/routes/tonal-field.ts` | Same |
| POST | `/api/resonance/sweep` | `server/routes/tonal-field.ts` | Same |
| POST | `/api/resonance/rtt` | `server/routes/tonal-field.ts` | Same |
| GET | `/api/metrics/plenum` | `server/routes/tonal-field.ts` | Same |

## 3.4 PPTPro (Plenum Pulse Tonal Professor)

| Method | Path | Source File | Reason |
|---|---|---|---|
| GET | `/api/v1/status` | `server/routes/pptpro-integration.ts` | Biometric/tonal integration; not a LAN management feature |
| GET | `/api/v1/safety/limits` | `server/routes/pptpro-integration.ts` | Same |
| GET | `/api/v1/ternary/state` | `server/routes/pptpro-integration.ts` | Same |
| POST | `/api/v1/entrain/advise` | `server/routes/pptpro-integration.ts` | Same |
| POST | `/api/v1/logs/coherence` | `server/routes/pptpro-integration.ts` | Same |

## 3.5 GitHub Integration

| Method | Path | Source File | Reason |
|---|---|---|---|
| POST | `/api/github/token` | `server/routes/github.ts` | Developer platform GitHub proxy; PlenumLAN has no GitHub integration |
| GET | `/api/github/status` | `server/routes/github.ts` | Same |
| GET | `/api/github/repos/:owner/:repo/branches` | `server/routes/github.ts` | Same |
| GET | `/api/github/repos/:owner/:repo/contents` | `server/routes/github.ts` | Same |
| GET | `/api/github/file/:owner/:repo` | `server/routes/github.ts` | Same |
| PUT | `/api/github/file/:owner/:repo` | `server/routes/github.ts` | Same |
| DELETE | `/api/github/file/:owner/:repo` | `server/routes/github.ts` | Same |
| POST | `/api/github/push-workflows/:owner/:repo` | `server/routes/github.ts` | Same |
| POST | `/api/github/push-env/:owner/:repo` | `server/routes/github.ts` | Same |
| POST | `/api/github/push-batch/:owner/:repo` | `server/routes/github.ts` | Same |

## 3.6 Hedera HCS Witnessing

| Method | Path | Source File | Reason |
|---|---|---|---|
| POST | `/api/hedera/v1/witness` | `server/routes/hedera.ts` | Blockchain witnessing is a PlenumNET platform feature; PlenumLAN uses Merkle audit fabric instead |
| GET | `/api/hedera/v1/witness/:txId` | `server/routes/hedera.ts` | Same |
| POST | `/api/hedera/v1/verify` | `server/routes/hedera.ts` | Same |
| GET | `/api/hedera/v1/topic` | `server/routes/hedera.ts` | Same |
| GET | `/api/hedera/v1/health` | `server/routes/hedera.ts` | Same |
| GET | `/api/hedera/v1/stats` | `server/routes/hedera.ts` | Same |

## 3.7 PQTI Proxy

| Method | Path | Source File | Reason |
|---|---|---|---|
| ALL | `/api/pqti/*` | `server/routes/pqti.ts` | Microservice proxy for developer platform; PlenumLAN integrates PQTI modules directly |
| GET | `/api/pqti-status` | `server/routes/pqti.ts` | Same |

## 3.8 API Key Management

| Method | Path | Source File | Reason |
|---|---|---|---|
| GET | `/api/keys/scopes` | `server/routes/api-keys.ts` | PlenumNET uses API keys for developer/partner access; PlenumLAN uses TL-DSA identity + capability tokens exclusively |
| POST | `/api/keys/generate` | `server/routes/api-keys.ts` | Same |
| GET | `/api/keys` | `server/routes/api-keys.ts` | Same |
| GET | `/api/keys/stats` | `server/routes/api-keys.ts` | Same |
| POST | `/api/keys/revoke/:id` | `server/routes/api-keys.ts` | Same |
| GET | `/api/keys/:id/logs` | `server/routes/api-keys.ts` | Same |
| POST | `/api/keys/rotate/:id` | `server/routes/api-keys.ts` | Same |
| GET | `/api/keys/expiring` | `server/routes/api-keys.ts` | Same |
| PATCH | `/api/keys/:id/rate-limit` | `server/routes/api-keys.ts` | Same |
| GET | `/api/keys/rate-limit-tiers` | `server/routes/api-keys.ts` | Same |
| GET | `/api/keys/entity-types` | `server/routes/api-keys.ts` | Same |
| PATCH | `/api/keys/:id/metadata` | `server/routes/api-keys.ts` | Same |
| GET | `/api/keys/anomalies` | `server/routes/api-keys.ts` | Same |
| GET | `/api/keys/audit` | `server/routes/api-keys.ts` | Same |
| GET | `/api/keys/:id/audit` | `server/routes/api-keys.ts` | Same |
| GET | `/api/keys/validate-external` | `server/routes/api-keys.ts` | Same |

## 3.9 Compression Demo & Whitepaper

| Method | Path | Source File | Reason |
|---|---|---|---|
| POST | `/api/demo/run` | `server/routes.ts` | Marketing demo; PlenumLAN doesn't expose compression as a user feature |
| GET | `/api/demo/stats` | `server/routes.ts` | Same |
| GET | `/api/demo/session/:sessionId` | `server/routes.ts` | Same |
| POST | `/api/demo/upload` | `server/routes.ts` | Same |
| GET | `/api/demo/history` | `server/routes.ts` | Same |
| GET | `/api/demo/files` | `server/routes.ts` | Same |
| GET | `/api/demo/data/:sessionId` | `server/routes.ts` | Same |
| POST | `/api/compression/file` | `server/routes.ts` | Same |
| POST | `/api/compression/decompress` | `server/routes.ts` | Same |
| POST | `/api/compression/file/raw` | `server/routes.ts` | Same (raw binary transport variant) |
| POST | `/api/compression/decompress/raw` | `server/routes.ts` | Same (raw binary transport variant) |
| POST | `/api/compression/db/store` | `server/routes.ts` | Same (DB storage for compressed docs) |
| GET | `/api/compression/db/retrieve/:id` | `server/routes.ts` | Same (retrieve stored compressed doc) |
| GET | `/api/compression/db/documents` | `server/routes.ts` | Same |
| GET | `/api/compression/db/raw/:id` | `server/routes.ts` | Same |
| DELETE | `/api/compression/db/documents/:id` | `server/routes.ts` | Same |
| GET | `/api/whitepapers` | `server/routes.ts` | Marketing/compliance; not LAN management |
| GET | `/api/whitepapers/active` | `server/routes.ts` | Same |
| GET | `/api/whitepapers/:id` | `server/routes.ts` | Same (individual whitepaper by ID) |
| POST | `/api/whitepapers` | `server/routes.ts` | Same |

## 3.10 Developer Signup & Admin

| Method | Path | Source File | Reason |
|---|---|---|---|
| POST | `/api/developer-signup` | `server/routes.ts` | Platform developer enrollment; PlenumLAN has PDS for user enrollment |
| GET | `/api/developer-signup/count` | `server/routes.ts` | Same |
| GET | `/api/admin/developer-signups` | `server/routes.ts` | Same |
| DELETE | `/api/admin/developer-signups/:id` | `server/routes.ts` | Same |
| GET | `/api/user/admin-status` | `server/routes.ts` | PlenumLAN uses capability-token-based admin, not Replit Auth |

## 3.11 Legacy TDNS Endpoints (routes.ts)

| Method | Path | Source File | Reason |
|---|---|---|---|
| GET | `/api/tdns/resolve` | `server/routes.ts` | Legacy TDNS resolve (query-param variant); superseded by `/api/tdns/resolve/:name` in tdns.ts |
| GET | `/api/tdns/records` | `server/routes.ts` | Legacy TDNS record listing; superseded by `/api/tdns/list` in tdns.ts |

## 3.12 Legal, Benchmark, CSP

| Method | Path | Source File | Reason |
|---|---|---|---|
| GET | `/api/legal/:type` | `server/routes.ts` | Marketing website legal docs |
| GET | `/api/benchmark-report` | `server/routes.ts` | Benchmark HTML report for developer platform |
| GET | `/.well-known/security.txt` | `server/routes.ts` | Web platform security disclosure |
| POST | `/api/csp-reports` | `server/routes.ts` | CSP violation reporting for web platform |
| GET | `/api/verify` | `server/routes.ts` | Replit Auth verification |

## 3.13 Salvi Core — Demo/PlenumNET-Only Endpoints

| Method | Path | Source File | Reason |
|---|---|---|---|
| POST | `/api/salvi/ternary/batch` | `server/routes/salvi.ts` | Developer demo |
| GET | `/api/salvi/ternary/density/:tritCount` | `server/routes/salvi.ts` | Developer educational tool |
| GET | `/api/salvi/ternary/density-benchmark` | `server/routes/salvi.ts` | Same |
| POST | `/api/salvi/ternary/noether-verify` | `server/routes/salvi.ts` | Physics verification demo |
| GET | `/api/salvi/crypto/phase-benchmark` | `server/routes/salvi.ts` | Benchmark demo |
| GET | `/api/salvi/timing/epoch/anchors` | `server/routes/salvi.ts` | Calendar showcase |
| GET | `/api/salvi/timing/epoch/calendars` | `server/routes/salvi.ts` | 42-calendar synchronization showcase |
| GET | `/api/salvi/timing/epoch/calendars/mayan` | `server/routes/salvi.ts` | Mayan calendar conversion |
| GET | `/api/salvi/timing/epoch/calendars/hebrew` | `server/routes/salvi.ts` | Hebrew calendar conversion |
| GET | `/api/salvi/timing/epoch/calendars/vedic` | `server/routes/salvi.ts` | Vedic calendar conversion |
| GET | `/api/salvi/timing/epoch/calendars/chinese` | `server/routes/salvi.ts` | Chinese calendar conversion |
| GET | `/api/salvi/timing/epoch/calendars/islamic` | `server/routes/salvi.ts` | Islamic calendar conversion |
| GET | `/api/salvi/timing/epoch/calendars/egyptian` | `server/routes/salvi.ts` | Egyptian calendar conversion |
| GET | `/api/salvi/timing/epoch/calendars/julian-day` | `server/routes/salvi.ts` | Julian Day conversion |
| GET | `/api/salvi/timing/epoch/calendars/byzantine` | `server/routes/salvi.ts` | Byzantine calendar conversion |
| GET | `/api/salvi/timing/epoch/calendars/thirteen-moon` | `server/routes/salvi.ts` | 13-Moon calendar conversion |
| GET | `/api/salvi/crypto/pt26/spec` | `server/routes/salvi.ts` | PT26-DSA spec (research/demo) |
| POST | `/api/salvi/crypto/pt26/keygen` | `server/routes/salvi.ts` | PT26-DSA keygen demo |
| POST | `/api/salvi/crypto/pt26/sign` | `server/routes/salvi.ts` | PT26-DSA sign demo |
| POST | `/api/salvi/crypto/pt26/verify` | `server/routes/salvi.ts` | PT26-DSA verify demo |
| GET | `/api/salvi/vm/spec` | `server/routes/salvi.ts` | TVM ISA spec (developer reference) |
| GET | `/api/salvi/vm/conformance` | `server/routes/salvi.ts` | TVM conformance testing |

## 3.14 Security — Showcase/Admin-Only Endpoints

| Method | Path | Source File | Reason |
|---|---|---|---|
| GET | `/api/security/audit/unresolved` | `server/routes/security.ts` | Platform-internal security admin |
| GET | `/api/security/audit/:id` | `server/routes/security.ts` | Same |
| PATCH | `/api/security/audit/:id/resolve` | `server/routes/security.ts` | Same |
| POST | `/api/security/hptp/anomalies` | `server/routes/security.ts` | HPTP anomaly detection (platform tool) |
| GET | `/api/security/hptp/anomalies` | `server/routes/security.ts` | Same |
| GET | `/api/security/hptp/status` | `server/routes/security.ts` | Same |
| GET | `/api/security/hptp/fallback-analysis` | `server/routes/security.ts` | Same |
| GET | `/api/security/hptp/stats` | `server/routes/security.ts` | Same |
| GET | `/api/security/hptp/thresholds` | `server/routes/security.ts` | Same |
| GET | `/api/security/hptp/fallback-modes` | `server/routes/security.ts` | Same |
| GET | `/api/security/hptp/redundancy` | `server/routes/security.ts` | Same |
| POST | `/api/security/threats` | `server/routes/security.ts` | Threat model (development tool) |
| GET | `/api/security/threats` | `server/routes/security.ts` | Same |
| GET | `/api/security/threats/risk-matrix` | `server/routes/security.ts` | Same |
| GET | `/api/security/threats/stats` | `server/routes/security.ts` | Same |
| GET | `/api/security/threats/meta` | `server/routes/security.ts` | Same |
| GET | `/api/security/threats/:id` | `server/routes/security.ts` | Same |
| PATCH | `/api/security/threats/:id` | `server/routes/security.ts` | Same |
| DELETE | `/api/security/threats/:id` | `server/routes/security.ts` | Same |
| POST | `/api/security/threats/seed` | `server/routes/security.ts` | Same |
| POST | `/api/security/implementation` | `server/routes/security.ts` | Implementation tracker (dev tool) |
| GET | `/api/security/implementation` | `server/routes/security.ts` | Same |
| GET | `/api/security/implementation/summary` | `server/routes/security.ts` | Same |
| GET | `/api/security/implementation/metrics` | `server/routes/security.ts` | Same |
| GET | `/api/security/implementation/milestones` | `server/routes/security.ts` | Same |
| GET | `/api/security/implementation/meta` | `server/routes/security.ts` | Same |
| GET | `/api/security/implementation/:id` | `server/routes/security.ts` | Same |
| PATCH | `/api/security/implementation/:id` | `server/routes/security.ts` | Same |
| DELETE | `/api/security/implementation/:id` | `server/routes/security.ts` | Same |
| POST | `/api/security/implementation/seed` | `server/routes/security.ts` | Same |
| GET | `/api/security/metadata/categories` | `server/routes/security.ts` | Security metadata categories (dev tool) |
| GET | `/api/security/metadata/types` | `server/routes/security.ts` | Security metadata types (dev tool) |

## 3.15 Capability Token — Demo-Only Endpoints

| Method | Path | Source File | Reason |
|---|---|---|---|
| GET | `/api/capabilities/demo/expiration` | `server/routes/capabilities.ts` | Demo showcase |
| GET | `/api/capabilities/demo/delegation` | `server/routes/capabilities.ts` | Same |
| GET | `/api/capabilities/demo/confinement` | `server/routes/capabilities.ts` | Same |
| GET | `/api/capabilities/demo/certificates` | `server/routes/capabilities.ts` | Same |
| GET | `/api/capabilities/demo/mesh` | `server/routes/capabilities.ts` | Same |
| POST | `/api/capabilities/certificate/evidence-chain` | `server/routes/capabilities.ts` | Advanced certificate feature |
| GET | `/api/capabilities/certificate/stats` | `server/routes/capabilities.ts` | Certificate stats |
| GET | `/api/capabilities/certificate/:certId/verify-data` | `server/routes/capabilities.ts` | Certificate data export |
| POST | `/api/capabilities/mesh/register` | `server/routes/capabilities.ts` | Service mesh (inter-service, not LAN) |
| POST | `/api/capabilities/mesh/issue` | `server/routes/capabilities.ts` | Same |
| POST | `/api/capabilities/mesh/propagate` | `server/routes/capabilities.ts` | Same |
| GET | `/api/capabilities/mesh/discover` | `server/routes/capabilities.ts` | Same |
| POST | `/api/capabilities/mesh/validate` | `server/routes/capabilities.ts` | Same |
| GET | `/api/capabilities/mesh/topology` | `server/routes/capabilities.ts` | Same |
| GET | `/api/capabilities/mesh/health` | `server/routes/capabilities.ts` | Same |

## 3.16 Tribonacci & Agent Array

| Method | Path | Source File | Reason |
|---|---|---|---|
| GET | `/api/tribonacci/hook` | `server/routes/tribonacci.ts` | Developer demo tools |
| GET | `/api/tribonacci/permutation` | `server/routes/tribonacci.ts` | Same |
| GET | `/api/tribonacci/coverage` | `server/routes/tribonacci.ts` | Same |
| GET | `/api/tribonacci/hash` | `server/routes/tribonacci.ts` | Same |
| GET | `/api/tribonacci/sequence` | `server/routes/tribonacci.ts` | Same |
| POST | `/api/tribonacci/generate-id` | `server/routes/tribonacci.ts` | Same |
| GET | `/api/tribonacci/next-worker` | `server/routes/tribonacci.ts` | Same |
| GET | `/api/tribonacci/skip-lookup` | `server/routes/tribonacci.ts` | Same |
| GET | `/api/tribonacci/hash-distribution` | `server/routes/tribonacci.ts` | Same |
| POST | `/api/tribonacci/agent-array` | `server/routes/agent-array.ts` | AI agent array (showcase) |
| GET | `/api/tribonacci/agent-array/stream/:sessionId` | `server/routes/agent-array.ts` | Same |
| POST | `/api/tribonacci/agent-array/save` | `server/routes/agent-array.ts` | Same |
| GET | `/api/tribonacci/agent-array/reports` | `server/routes/agent-array.ts` | Same |
| GET | `/api/tribonacci/agent-array/reports/:id` | `server/routes/agent-array.ts` | Same |
| GET | `/api/tribonacci/agent-array/positions` | `server/routes/agent-array.ts` | Same |

## 3.17 Ephemeris

| Method | Path | Source File | Reason |
|---|---|---|---|
| POST | `/api/ephemeris/convert` | `server/routes/ephemeris.ts` | Astronomical tool; not LAN management |
| POST | `/api/ephemeris/position` | `server/routes/ephemeris.ts` | Same |
| POST | `/api/ephemeris/batch` | `server/routes/ephemeris.ts` | Same |
| GET | `/api/ephemeris/info` | `server/routes/ephemeris.ts` | Same |

## 3.18 GDPR / Data Subject Rights

| Method | Path | Source File | Reason |
|---|---|---|---|
| GET | `/api/gdpr/data-export` | `server/routes/data-subject-rights.ts` | Web platform compliance feature |
| DELETE | `/api/gdpr/delete-account` | `server/routes/data-subject-rights.ts` | Same |
| GET | `/api/gdpr/requests` | `server/routes/data-subject-rights.ts` | Same |
| GET | `/api/gdpr/policy` | `server/routes/data-subject-rights.ts` | Same |

## 3.19 TSA — PlenumNET-Only Subset

| Method | Path | Source File | Reason |
|---|---|---|---|
| GET | `/api/tsa/certificate/download` | `server/routes/tsa.ts` | PEM download for platform users |
| GET | `/api/tsa/tokens` | `server/routes/tsa.ts` | Admin token log query |
| GET | `/api/tsa/policy` | `server/routes/tsa.ts` | Policy info (platform-facing) |

---

# Appendix: Cross-Reference by PlenumLAN Module

| PlenumLAN Module | Mapped From (Existing) | Gaps (Net-New) | Console Screen(s) |
|---|---|---|---|
| **TDNS-L** (Name Resolver) | TDNS scan, resolve, list, health | Dual-stack DNS (port 53), LAN scan signals | Name Browser |
| **CRS-L** (Local Registry) | Inter-Cube CRS register/lookup/heartbeat/deregister/stats | Address module (§5), bitmap allocator, IPv4 collision table, nearest-service routing | Entity Registry, Entity Detail, Address Allocation |
| **PDS** (Directory Service) | Capability system (issue/validate/delegate/verify-chain), TDNS orgs | User enrollment, auth flow, session mgmt, issuance rules, delegation chains, Merkle audit, AD importer | Permission Rules, Delegation Chains, Identity Dashboard, Audit Log |
| **PFS** (File Service) | *(none)* | Full SMB 3.1.1 + NFS v4.2 bridge, backup snapshots, update distribution, print driver serving | Shared Folders |
| **Shims** (Protocol Bridges) | *(none)* | RADIUS shim, LDAP shim, DHCP responder, print bridge | Network Access, Printer Devices |
| **PTS** (Remote Tunnel) | *(none)* | Phase-encrypted tunnel, WebAuthn enrollment, scoped remote access | *(remote console access)* |
| **Crypto Kernel** | Salvi crypto (hash, TL-DSA, phase encrypt, timing) | *(direct Rust calls — no bridge needed)* | — |
| **GLB/CON/FTS** (Multi-Site) | Inter-Cube GLB/CON/FTS endpoints | *(logic identical; activates on second-site onboarding)* | — |
| **Console** | *(none — entirely new React app)* | 16 management screens | All 16 screens (§18) |
| **Infrastructure** | *(none)* | First-run wizard, installer pipeline, auto-update, PlenumDB, emergency console | System Overview, Scan Workflow |

---

*Document produced from full endpoint inventory of PlenumNET (server/routes/*.ts, server/routes.ts) mapped against TM-2026-019.2 §5–§24.*
