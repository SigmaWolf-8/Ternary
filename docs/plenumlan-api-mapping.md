# PlenumNET → PlenumLAN API Mapping Document

**Reference:** TM-2026-019.2
**Repository:** [SigmaWolf-8/Ternary](https://github.com/SigmaWolf-8/Ternary)
**Capomastro Holdings Ltd. — Applied Physics Division**
**Generated: March 2026**

---

## Summary

| Category | Count |
|---|---|
| Mapped endpoints | 79 |
| PlenumNET-Only endpoints | 195 |
| Total existing endpoints | 274 |
| Gap items (net-new) | 24 |

---

# Section 1: Mapped Endpoints

## 1.1 TDNS → TDNS-L

| Method | Path | Source | LAN Target | §Ref |
|---|---|---|---|---|
| GET | `/api/tdns/health` | `tdns.ts` | `tdns_l/health.rs` | §5–§7 |
| POST | `/api/tdns/scan` | `tdns.ts` | `tdns_l/scan.rs` | §7 |
| POST | `/api/tdns/register` | `tdns.ts` | `crs_l/register.rs` | §10.1 |
| GET | `/api/tdns/resolve/:name` | `tdns.ts` | `tdns_l/resolve.rs` | §6.7 |
| GET | `/api/tdns/list` | `tdns.ts` | `crs_l/list.rs` | §10 |
| POST | `/api/tdns/org/create` | `tdns.ts` | `pds/org.rs` | §10.3 |
| POST | `/api/tdns/org/add-url` | `tdns.ts` | `pds/org.rs` | §10.3 |
| GET | `/api/tdns/org/:name` | `tdns.ts` | `pds/org.rs` | §10.3 |
| GET | `/api/tdns/orgs` | `tdns.ts` | `pds/org.rs` | §10.3 |

**Delta Notes:**

- `register` → moves to CRS-L; adds TL-DSA keygen, IP derivation (§6), heartbeat init, capability evaluation
- `resolve` → adds dual-stack (A/AAAA/TDNS-L native); `.plm.local` TLD triggers host-ID interpretation of second 27 trits
- `scan` → HTTP-fetch identical for `.plm.local`; physical LAN entities use template-driven scan (§7.8)
- `list` → sourced from CRS-L entity store instead of in-memory map
- `org/*` → become PDS directory objects; creation requires TL-DSA-signed capability

## 1.2 Inter-Cube CRS → CRS-L

| Method | Path | Source | LAN Target | §Ref |
|---|---|---|---|---|
| POST | `/api/salvi/inter-cube/crs/register` | `inter-cube.ts` | `crs_l/register.rs` | §10.1 |
| GET | `/api/salvi/inter-cube/crs/lookup/:address` | `inter-cube.ts` | `crs_l/lookup.rs` | §10 |
| GET | `/api/salvi/inter-cube/crs/neighbors/:address` | `inter-cube.ts` | `crs_l/neighbors.rs` | §8.1 |
| POST | `/api/salvi/inter-cube/crs/heartbeat` | `inter-cube.ts` | `crs_l/heartbeat.rs` | §10.1 |
| POST | `/api/salvi/inter-cube/crs/deregister` | `inter-cube.ts` | `crs_l/deregister.rs` | §10 |
| GET | `/api/salvi/inter-cube/crs/stats` | `inter-cube.ts` | `crs_l/stats.rs` | §10 |

**Delta Notes:**

- `register` → adds IP derivation (§6), scan template classification (§7.8), auto-capability evaluation
- `lookup` → returns full dual-stack addresses + classification metadata
- `neighbors` → used for nearest-service routing (e.g., closest printer)
- `heartbeat` → HPTP-timestamped; FTS health state transitions
- `deregister` → frees vertex in bitmap allocator; revokes all capability tokens
- `stats` → includes vertex occupancy bitmap utilization

## 1.3 Inter-Cube GLB/CON/FTS → LAN Infrastructure

| Method | Path | Source | LAN Target | §Ref |
|---|---|---|---|---|
| POST | `/api/salvi/inter-cube/glb/forward` | `inter-cube.ts` | `routes/glb.rs` | §8.1 |
| GET | `/api/salvi/inter-cube/glb/stats` | `inter-cube.ts` | `routes/glb.rs` | §8.1 |
| GET | `/api/salvi/inter-cube/glb/health` | `inter-cube.ts` | `routes/glb.rs` | §8.1 |
| GET | `/api/salvi/inter-cube/con/neighbors` | `inter-cube.ts` | `routes/con.rs` | §12 |
| GET | `/api/salvi/inter-cube/con/stats` | `inter-cube.ts` | `routes/con.rs` | §12 |
| POST | `/api/salvi/inter-cube/con/tunnel/refresh` | `inter-cube.ts` | `routes/con.rs` | §12 |
| POST | `/api/salvi/inter-cube/con/tunnel/upgrade-key` | `inter-cube.ts` | `routes/con.rs` | §12 |
| GET | `/api/salvi/inter-cube/fts/status` | `inter-cube.ts` | `crs_l/fts.rs` | §10.1 |
| GET | `/api/salvi/inter-cube/fts/dead` | `inter-cube.ts` | `crs_l/fts.rs` | §10.1 |
| POST | `/api/salvi/inter-cube/fts/config` | `inter-cube.ts` | `crs_l/fts.rs` | §10.1 |
| POST | `/api/salvi/inter-cube/routing/compute` | `inter-cube.ts` | `routes/routing.rs` | §8.1 |
| POST | `/api/salvi/inter-cube/address/validate` | `inter-cube.ts` | `address/rep_c.rs` | §5 |
| GET | `/api/salvi/inter-cube/topology` | `inter-cube.ts` | `routes/topology.rs` | §8 |

**Delta Notes:**

- GLB → active only when second site onboards; LAN-internal routing uses direct cube adjacency
- CON → tunnels activate on second-site onboarding; 26 tunnels per cube; same TLSponge-385 key derivation
- FTS → health states for LAN entities (heartbeat monitoring); tuning for miss thresholds and recovery periods
- `routing/compute` → pure math Hamming distance/path computation — identical logic
- `address/validate` → Rep C validation with zero-sentinel forgery detection — identical

## 1.4 Salvi Core Crypto → LAN Kernel (Direct Rust)

In PlenumLAN (pure Rust), these become direct function calls — no HTTP layer. Thin Axum handlers wrap the same Rust functions for the web console.

| Method | Path | Source | LAN Target | §Ref |
|---|---|---|---|---|
| POST | `/api/salvi/crypto/hash` | `salvi.ts` | `routes/crypto.rs` | §8.3 |
| POST | `/api/salvi/crypto/tl-dsa/keygen` | `salvi.ts` | `routes/crypto.rs` | §8.3 |
| POST | `/api/salvi/crypto/tl-dsa/sign` | `salvi.ts` | `routes/crypto.rs` | §8.3 |
| POST | `/api/salvi/crypto/tl-dsa/verify` | `salvi.ts` | `routes/crypto.rs` | §8.3 |
| GET | `/api/salvi/crypto/tl-dsa/spec` | `salvi.ts` | `routes/crypto.rs` | §8.3 |
| GET | `/api/salvi/crypto/tl-kem/spec` | `salvi.ts` | `routes/crypto.rs` | §8.3 |
| POST | `/api/salvi/phase/split` | `salvi.ts` | `routes/crypto.rs` | §8.3 |
| POST | `/api/salvi/phase/recombine` | `salvi.ts` | `routes/crypto.rs` | §8.3 |
| GET | `/api/salvi/phase/config/:mode` | `salvi.ts` | `routes/crypto.rs` | §8.3 |
| GET | `/api/salvi/phase/recommend` | `salvi.ts` | `routes/crypto.rs` | §8.3 |
| POST | `/api/salvi/phase/batch/split` | `salvi.ts` | `routes/crypto.rs` | §8.3 |
| POST | `/api/salvi/phase/batch/recombine` | `salvi.ts` | `routes/crypto.rs` | §8.3 |

**Delta Notes:**

- `hash` → TL-Sponge-385 direct Rust, no TS-Rust bridge
- `tl-dsa/*` → keygen at entity registration; sign/verify for capability tokens and auth
- `tl-kem/spec` → used for CON tunnel key exchange
- `phase/*` → phase encryption for PFS data at rest

## 1.5 Timing → HPTP

| Method | Path | Source | LAN Target | §Ref |
|---|---|---|---|---|
| GET | `/api/salvi/timing/timestamp` | `salvi.ts` | `routes/timing.rs` | §8.4 |
| GET | `/api/salvi/timing/metrics` | `salvi.ts` | `routes/timing.rs` | §8.4 |
| GET | `/api/salvi/timing/self-test` | `salvi.ts` | `routes/timing.rs` | §8.4 |
| GET | `/api/salvi/timing/error-budget` | `salvi.ts` | `routes/timing.rs` | §8.4 |
| GET | `/api/salvi/timing/batch/:count` | `salvi.ts` | `routes/timing.rs` | §8.4 |

**Delta Notes:**

- `timestamp` → direct Rust; used for token expiration and audit
- `error-budget` → drift and jitter reporting

## 1.6 Capability Tokens → LAN Capability System

| Method | Path | Source | LAN Target | §Ref |
|---|---|---|---|---|
| POST | `/api/capabilities/issue` | `capabilities.ts` | `pds/capabilities.rs` | §8.5 |
| POST | `/api/capabilities/validate` | `capabilities.ts` | `pds/capabilities.rs` | §8.5 |
| POST | `/api/capabilities/delegate` | `capabilities.ts` | `pds/capabilities.rs` | §8.5 |
| POST | `/api/capabilities/delegate/chain` | `capabilities.ts` | `pds/capabilities.rs` | §8.5 |
| POST | `/api/capabilities/verify-chain` | `capabilities.ts` | `pds/capabilities.rs` | §8.5 |
| GET | `/api/capabilities/audit` | `capabilities.ts` | `pds/capabilities.rs` | §8.5 |
| POST | `/api/capabilities/hardware/register` | `capabilities.ts` | `pds/hardware_bind.rs` | §8.5 |
| POST | `/api/capabilities/hardware/challenge` | `capabilities.ts` | `pds/hardware_bind.rs` | §10.2 |
| POST | `/api/capabilities/hardware/verify` | `capabilities.ts` | `pds/hardware_bind.rs` | §10.2 |
| POST | `/api/capabilities/hardware/issue` | `capabilities.ts` | `pds/hardware_bind.rs` | §8.5 |
| POST | `/api/capabilities/certificate/issue` | `capabilities.ts` | `pds/certificates.rs` | §8.3 |
| POST | `/api/capabilities/certificate/verify` | `capabilities.ts` | `pds/certificates.rs` | §8.3 |
| GET | `/api/capabilities/certificate/:certId/rfc3161` | `capabilities.ts` | `pds/certificates.rs` | §8.3 |
| GET | `/api/capabilities/status` | `capabilities.ts` | `pds/capabilities.rs` | §8.5 |

**Delta Notes:**

- `issue` → auto-triggered by CRS-L issuance rules on cube regions
- `validate` → called by every protocol bridge (PFS, RADIUS, LDAP)
- `delegate` → HMAC-chained delegation replaces AD group membership
- `hardware/*` → WebAuthn/FIDO2 device binding; HPTP-based challenge for remote auth
- `certificate/*` → RFC 3161 certificates for capabilities

## 1.7 TSA → LAN Audit Fabric

| Method | Path | Source | LAN Target | §Ref |
|---|---|---|---|---|
| POST | `/api/tsa/timestamp` | `tsa.ts` | `pds/audit.rs` | §8.4 |
| POST | `/api/tsa/timestamp/json` | `tsa.ts` | `pds/audit.rs` | §8.4 |
| POST | `/api/tsa/verify` | `tsa.ts` | `pds/audit.rs` | §8.4 |
| GET | `/api/tsa/certificate` | `tsa.ts` | `pds/audit.rs` | §8.3 |
| GET | `/api/tsa/health` | `tsa.ts` | `pds/audit.rs` | §8.4 |
| GET | `/api/tsa/audit/query` | `tsa.ts` | `pds/audit.rs` | §8.4 |

**Delta Notes:**

- `timestamp` → RFC 3161 timestamping feeds Merkle-chained audit fabric
- `audit/query` → Merkle-chained in PlenumLAN

## 1.8 Security → LAN Security Dashboard

| Method | Path | Source | LAN Target | §Ref |
|---|---|---|---|---|
| POST | `/api/security/audit` | `security.ts` | `pds/security.rs` | §11 |
| GET | `/api/security/audit` | `security.ts` | `pds/security.rs` | §11 |
| GET | `/api/security/audit/summary` | `security.ts` | `pds/security.rs` | §11 |
| GET | `/api/security/audit/stats` | `security.ts` | `pds/security.rs` | §11 |
| GET | `/api/security/dashboard` | `security.ts` | `pds/security.rs` | §11 |
| GET | `/api/security/kri` | `security.ts` | `pds/security.rs` | §11 |

**Delta Notes:**

- `audit` → security event logging to Merkle audit fabric
- `dashboard` → unified dashboard (Console Screen: Security Overview)
- `kri` → Key Risk Indicators

## 1.9 Ternary Operations → LAN Kernel

| Method | Path | Source | LAN Target | §Ref |
|---|---|---|---|---|
| POST | `/api/salvi/ternary/convert` | `salvi.ts` | `routes/ternary.rs` | §5.1 |
| POST | `/api/salvi/ternary/add` | `salvi.ts` | `routes/ternary.rs` | §4 |
| POST | `/api/salvi/ternary/multiply` | `salvi.ts` | `routes/ternary.rs` | §4 |
| POST | `/api/salvi/ternary/rotate` | `salvi.ts` | `routes/ternary.rs` | §4 |
| POST | `/api/salvi/ternary/not` | `salvi.ts` | `routes/ternary.rs` | §4 |
| POST | `/api/salvi/ternary/xor` | `salvi.ts` | `routes/ternary.rs` | §4 |

**Delta Notes:**

- All → direct Rust calls; thin Axum handler for console
- `convert` → Rep A/B/C conversion

## 1.10 Health & Platform

| Method | Path | Source | LAN Target | §Ref |
|---|---|---|---|---|
| GET | `/api/health` | `routes.ts` | `routes/health.rs` | §18 |
| GET | `/api/salvi/docs` | `salvi.ts` | `routes/docs.rs` | §18 |

---

# Section 2: Gap Analysis

PlenumLAN features required by TM-2026-019.2 with no existing PlenumNET API equivalent. Net-new Rust implementations.

| # | Feature | §Ref | Release | Size |
|---|---|---|---|---|
| G1 | PFS (SMB 3.1.1 + NFS v4.2) | §9.1 | 1.0 | Large |
| G2 | RADIUS Shim | §9.2.1 | 1.0 | Medium |
| G3 | LDAP Compatibility Shim | §9.2.2 | 1.0 | Medium |
| G4 | Legacy DHCP Responder | §9.2.3 | 1.0 | Medium |
| G5 | Print Bridge (CUPS) | §9.3 | 1.0 | Medium |
| G6 | PDS — Directory Service | §10 | 0.5 | Large |
| G7 | 54-Trit Address Module | §5 | 0.1 | Medium |
| G8 | LAN Ontological Scan (27D) | §7 | 0.1 | Large |
| G9 | Site Network Configuration | §6.2 | 0.1 | Small |
| G10 | First-Run Setup Wizard | §14.4 | 0.1 | Medium |
| G11 | Download/Installer Pipeline | §14 | 0.1 | Medium |
| G12 | Automatic Update System | §14.6 | 2.0 | Medium |
| G13 | PlenumDB Storage Backend | §19.5 | 2.0 | Large |
| G14 | PTS — Tunnel Service | §16 | 2.0 | Large |
| G15 | Management Console (16 Screens) | §18 | 0.1–1.0 | Large |
| G16 | Windows Server Importer | §19.2 | 0.5 | Medium |
| G17 | Cube Bitmap Allocator | §10.1 | 0.1 | Small |
| G18 | Cross-Shell Authorization | §8.2 | 0.5 | Small |
| G19 | Issuance Rules Engine | §8.5 | 0.5 | Medium |
| G20 | Merkle-Chained Audit Fabric | §8.4 | 0.5 | Medium |
| G21 | Nearest-Service Routing | §8.1 | 0.1 | Small |
| G22 | IPv4 Collision Table | §6.5 | 0.1 | Small |
| G23 | Dual-Stack DNS Resolver | §6.7 | 0.1 | Medium |
| G24 | Emergency Console (Text) | §17.3 | 3.0 | Small |

**Gap Descriptions:**

- **G1 PFS** — Full file protocol bridge: SMB/NFS to capability-mediated cube-addressed storage. Includes backup snapshots, update distribution, print driver distribution.
- **G2 RADIUS** — Access-Request to TL-DSA challenge; capability tokens to RADIUS attributes (VLAN=shell, ACL=capability scope). 3 endpoints.
- **G3 LDAP** — bind/search/compare to CRS cube queries. Read-only; writes to PDS API with TL-DSA-signed capabilities. 4 endpoints.
- **G4 DHCP** — DISCOVER to scan template assignment to CRS registration to IP derivation to OFFER/ACK. 2 endpoints.
- **G5 Print Bridge** — CUPS wrapping with cube-native discovery (D5=3 query) and capability-gated authorization.
- **G6 PDS** — User enrollment, cryptographic login (TL-DSA challenge-response), session management, capability lifecycle, delegation chains. Replaces Active Directory.
- **G7 Address Module** — Bidirectional IP-ternary bijection with host integer intermediary. Rep C parse with zero-sentinel forgery detection.
- **G8 LAN Scan** — MAC OUI/DHCP fingerprint, mDNS, SNMP, LLDP, port scan. 6 scan templates (workstation, server, infrastructure, printer, IoT, service).
- **G9 Site Config** — First-run auto-detection of IPv4 prefix, host range, IPv6 ULA prefix, shell-to-VLAN mapping.
- **G10 Setup Wizard** — 3 screens: site name, network settings, admin account (TL-DSA keygen). Root key generation, resolver init, CRS init, first entity registration, network scan start.
- **G11 Installer** — Cross-compiled for Windows (.exe), Debian (.deb), RPM (.rpm), macOS (.dmg), Linux binary, ARM binary. CI pipeline with cargo build for 6 targets.
- **G12 Auto-Update** — Periodic version check, TL-DSA signature verification, self-update with zero data loss, automatic restart.
- **G13 PlenumDB** — Purpose-built key-value store: 27-trit entity lookup, Merkle-tree audit chain, vertex occupancy bitmap (1,594,323-bit). WAL crash safety.
- **G14 PTS** — Phase-encrypted remote console. WebAuthn/FIDO2 enrollment, TL-DSA challenge-response, scoped tunnel (console only). Per-service capability tokens.
- **G15 Console** — 16 React screens: System Overview, Entity Registry, Entity Detail, Name Browser, Address Allocation, Permission Rules, Delegation Chains, Identity Dashboard, Shared Folders, Printer Devices, Audit Log, Security Overview, Network Access, System Health, Cube Visualizer, Scan Workflow.
- **G16 AD Importer** — Reads Active Directory (LDAP export) and creates matching CRS-L entries with capability equivalents for group memberships and GPOs.
- **G17 Bitmap Allocator** — flatIndex/fromFlatIndex bijection for 3^13 = 1,594,323 vertex occupancy tracking.
- **G18 Cross-Shell Auth** — Axis 12 shell lookup (Inner/Void/Outer to VLAN 10/20/30). Shell-transition capability tokens.
- **G19 Issuance Rules** — CRS metadata on cube regions: entities with address prefix P receive capability set C. Evaluated at registration time.
- **G20 Merkle Audit** — Every event HPTP-timestamped and Merkle-chained. TIS-27 integrity per entry. Tamper-evident, immutable log.
- **G21 Nearest-Service** — Hamming-distance query: find printer with smallest distance from requesting workstation.
- **G22 IPv4 Collision** — CRS override table for /24 subnets where modular reduction causes collisions.
- **G23 DNS Resolver** — Serves A (IPv4), AAAA (IPv6), and TDNS-L native queries from CRS records. Port 53. Replaces Windows DNS.
- **G24 Emergency Console** — USB keyboard + VGA/serial text display: read-only system status, FTS health, last Merkle hash, logs. No shell, no login.

### Complexity Summary

| Size | Count | Items |
|---|---|---|
| Large | 6 | PFS, PDS, LAN Scan, PlenumDB, PTS, Console |
| Medium | 12 | RADIUS, LDAP, DHCP, Print, Address, Wizard, Installer, Update, AD Import, Issuance, Merkle, DNS |
| Small | 6 | Site Config, Bitmap, Cross-Shell, Nearest-Service, IPv4 Collision, Emergency |

---

# Section 3: PlenumNET-Only Endpoints

Production endpoints that do NOT transfer to PlenumLAN. They remain exclusive to plenumnet.replit.app — platform services, developer tools, and third-party integrations.

## 3.1 Kong Gateway Management

| Method | Path | Source |
|---|---|---|
| GET | `/api/kong/status` | `kong.ts` |
| GET | `/api/kong/organization` | `kong.ts` |
| GET | `/api/kong/control-planes` | `kong.ts` |
| GET | `/api/kong/control-planes/:cpId/services` | `kong.ts` |
| GET | `/api/kong/control-planes/:cpId/routes` | `kong.ts` |
| GET | `/api/kong/control-planes/:cpId/plugins` | `kong.ts` |
| GET | `/api/kong/config` | `kong.ts` |
| POST | `/api/kong/control-planes/:cpId/services` | `kong.ts` |
| POST | `/api/kong/control-planes/:cpId/services/:serviceId/routes` | `kong.ts` |
| POST | `/api/kong/control-planes/:cpId/services/:serviceId/plugins` | `kong.ts` |
| POST | `/api/kong/control-planes/:cpId/sync-plenumnet` | `kong.ts` |
| POST | `/api/kong/sync-all-control-planes` | `kong.ts` |
| GET | `/api/kong/service-catalog` | `kong.ts` |
| POST | `/api/kong/save-to-github` | `kong.ts` |
| GET | `/api/kong/control-planes/:cpId/deploy-instructions` | `kong.ts` |
| POST | `/api/kong/control-planes/:cpId/generate-deployment` | `kong.ts` |
| POST | `/api/kong/control-planes/:cpId/deploy-to-cloud` | `kong.ts` |

PlenumNET platform API gateway. PlenumLAN has no external API gateway.

## 3.2 SFK Operations Pipeline

| Method | Path | Source |
|---|---|---|
| POST | `/api/sfk/v1/operations` | `sfk-operations.ts` |
| GET | `/api/sfk/v1/operations/:id` | `sfk-operations.ts` |
| GET | `/api/sfk/v1/operations` | `sfk-operations.ts` |
| DELETE | `/api/sfk/v1/operations/:id` | `sfk-operations.ts` |
| GET | `/api/sfk/v1/stats` | `sfk-operations.ts` |

Platform-specific batch processing. PlenumLAN uses direct Rust calls.

## 3.3 Tonal Field System

| Method | Path | Source |
|---|---|---|
| GET | `/api/tonal/field` | `tonal-field.ts` |
| GET | `/api/tonal/neighbors` | `tonal-field.ts` |
| POST | `/api/tonal/packet` | `tonal-field.ts` |
| GET | `/api/resonance/status` | `tonal-field.ts` |
| POST | `/api/resonance/sweep` | `tonal-field.ts` |
| POST | `/api/resonance/rtt` | `tonal-field.ts` |
| GET | `/api/metrics/plenum` | `tonal-field.ts` |

Network diffusion research tool.

## 3.4 PPTPro

| Method | Path | Source |
|---|---|---|
| GET | `/api/v1/status` | `pptpro-integration.ts` |
| GET | `/api/v1/safety/limits` | `pptpro-integration.ts` |
| GET | `/api/v1/ternary/state` | `pptpro-integration.ts` |
| POST | `/api/v1/entrain/advise` | `pptpro-integration.ts` |
| POST | `/api/v1/logs/coherence` | `pptpro-integration.ts` |

Biometric/tonal integration.

## 3.5 GitHub Integration

| Method | Path | Source |
|---|---|---|
| POST | `/api/github/token` | `github.ts` |
| GET | `/api/github/status` | `github.ts` |
| GET | `/api/github/repos/:owner/:repo/branches` | `github.ts` |
| GET | `/api/github/repos/:owner/:repo/contents` | `github.ts` |
| GET | `/api/github/file/:owner/:repo` | `github.ts` |
| PUT | `/api/github/file/:owner/:repo` | `github.ts` |
| DELETE | `/api/github/file/:owner/:repo` | `github.ts` |
| POST | `/api/github/push-workflows/:owner/:repo` | `github.ts` |
| POST | `/api/github/push-env/:owner/:repo` | `github.ts` |
| POST | `/api/github/push-batch/:owner/:repo` | `github.ts` |

Developer platform GitHub proxy.

## 3.6 Hedera HCS Witnessing

| Method | Path | Source |
|---|---|---|
| POST | `/api/hedera/v1/witness` | `hedera.ts` |
| GET | `/api/hedera/v1/witness/:txId` | `hedera.ts` |
| POST | `/api/hedera/v1/verify` | `hedera.ts` |
| GET | `/api/hedera/v1/topic` | `hedera.ts` |
| GET | `/api/hedera/v1/health` | `hedera.ts` |
| GET | `/api/hedera/v1/stats` | `hedera.ts` |

Blockchain witnessing. PlenumLAN uses Merkle audit fabric instead.

## 3.7 PQTI Proxy

| Method | Path | Source |
|---|---|---|
| ALL | `/api/pqti/*` | `pqti.ts` |
| GET | `/api/pqti-status` | `pqti.ts` |

Microservice proxy. PlenumLAN integrates PQTI modules directly.

## 3.8 API Key Management

| Method | Path | Source |
|---|---|---|
| GET | `/api/keys/scopes` | `api-keys.ts` |
| POST | `/api/keys/generate` | `api-keys.ts` |
| GET | `/api/keys` | `api-keys.ts` |
| GET | `/api/keys/stats` | `api-keys.ts` |
| POST | `/api/keys/revoke/:id` | `api-keys.ts` |
| GET | `/api/keys/:id/logs` | `api-keys.ts` |
| POST | `/api/keys/rotate/:id` | `api-keys.ts` |
| GET | `/api/keys/expiring` | `api-keys.ts` |
| PATCH | `/api/keys/:id/rate-limit` | `api-keys.ts` |
| GET | `/api/keys/rate-limit-tiers` | `api-keys.ts` |
| GET | `/api/keys/entity-types` | `api-keys.ts` |
| PATCH | `/api/keys/:id/metadata` | `api-keys.ts` |
| GET | `/api/keys/anomalies` | `api-keys.ts` |
| GET | `/api/keys/audit` | `api-keys.ts` |
| GET | `/api/keys/:id/audit` | `api-keys.ts` |
| GET | `/api/keys/validate-external` | `api-keys.ts` |

PlenumNET developer/partner API keys. PlenumLAN uses TL-DSA identity + capability tokens exclusively.

## 3.9 Compression & Whitepaper

| Method | Path | Source |
|---|---|---|
| POST | `/api/demo/run` | `routes.ts` |
| GET | `/api/demo/stats` | `routes.ts` |
| GET | `/api/demo/session/:sessionId` | `routes.ts` |
| POST | `/api/demo/upload` | `routes.ts` |
| GET | `/api/demo/history` | `routes.ts` |
| GET | `/api/demo/files` | `routes.ts` |
| GET | `/api/demo/data/:sessionId` | `routes.ts` |
| POST | `/api/compression/file` | `routes.ts` |
| POST | `/api/compression/decompress` | `routes.ts` |
| POST | `/api/compression/file/raw` | `routes.ts` |
| POST | `/api/compression/decompress/raw` | `routes.ts` |
| POST | `/api/compression/db/store` | `routes.ts` |
| GET | `/api/compression/db/retrieve/:id` | `routes.ts` |
| GET | `/api/compression/db/documents` | `routes.ts` |
| GET | `/api/compression/db/raw/:id` | `routes.ts` |
| DELETE | `/api/compression/db/documents/:id` | `routes.ts` |
| GET | `/api/whitepapers` | `routes.ts` |
| GET | `/api/whitepapers/active` | `routes.ts` |
| GET | `/api/whitepapers/:id` | `routes.ts` |
| POST | `/api/whitepapers` | `routes.ts` |

TTC compression and whitepaper management.

## 3.10 Developer Signup & Admin

| Method | Path | Source |
|---|---|---|
| POST | `/api/developer-signup` | `routes.ts` |
| GET | `/api/developer-signup/count` | `routes.ts` |
| GET | `/api/admin/developer-signups` | `routes.ts` |
| DELETE | `/api/admin/developer-signups/:id` | `routes.ts` |
| GET | `/api/user/admin-status` | `routes.ts` |

Platform developer enrollment. PlenumLAN has PDS for user enrollment.

## 3.11 Legacy TDNS Endpoints

| Method | Path | Source |
|---|---|---|
| GET | `/api/tdns/resolve` | `routes.ts` |
| GET | `/api/tdns/records` | `routes.ts` |

Legacy query-param variants; superseded by routes in `tdns.ts`.

## 3.12 Legal, Benchmark, CSP

| Method | Path | Source |
|---|---|---|
| GET | `/api/legal/:type` | `routes.ts` |
| GET | `/api/benchmark-report` | `routes.ts` |
| GET | `/.well-known/security.txt` | `routes.ts` |
| POST | `/api/csp-reports` | `routes.ts` |
| GET | `/api/verify` | `routes.ts` |

Web platform legal docs, benchmarks, security disclosure, Replit Auth.

## 3.13 Salvi Core — PlenumNET-Only

| Method | Path | Source |
|---|---|---|
| POST | `/api/salvi/ternary/batch` | `salvi.ts` |
| GET | `/api/salvi/ternary/density/:tritCount` | `salvi.ts` |
| GET | `/api/salvi/ternary/density-benchmark` | `salvi.ts` |
| POST | `/api/salvi/ternary/noether-verify` | `salvi.ts` |
| GET | `/api/salvi/crypto/phase-benchmark` | `salvi.ts` |
| GET | `/api/salvi/timing/epoch/anchors` | `salvi.ts` |
| GET | `/api/salvi/timing/epoch/calendars` | `salvi.ts` |
| GET | `/api/salvi/timing/epoch/calendars/mayan` | `salvi.ts` |
| GET | `/api/salvi/timing/epoch/calendars/hebrew` | `salvi.ts` |
| GET | `/api/salvi/timing/epoch/calendars/vedic` | `salvi.ts` |
| GET | `/api/salvi/timing/epoch/calendars/chinese` | `salvi.ts` |
| GET | `/api/salvi/timing/epoch/calendars/islamic` | `salvi.ts` |
| GET | `/api/salvi/timing/epoch/calendars/egyptian` | `salvi.ts` |
| GET | `/api/salvi/timing/epoch/calendars/julian-day` | `salvi.ts` |
| GET | `/api/salvi/timing/epoch/calendars/byzantine` | `salvi.ts` |
| GET | `/api/salvi/timing/epoch/calendars/thirteen-moon` | `salvi.ts` |
| GET | `/api/salvi/crypto/pt26/spec` | `salvi.ts` |
| POST | `/api/salvi/crypto/pt26/keygen` | `salvi.ts` |
| POST | `/api/salvi/crypto/pt26/sign` | `salvi.ts` |
| POST | `/api/salvi/crypto/pt26/verify` | `salvi.ts` |
| GET | `/api/salvi/vm/spec` | `salvi.ts` |
| GET | `/api/salvi/vm/conformance` | `salvi.ts` |

Developer tools, benchmarks, calendar showcases, PT26-DSA, TVM spec.

## 3.14 Security — Admin-Only

| Method | Path | Source |
|---|---|---|
| GET | `/api/security/audit/unresolved` | `security.ts` |
| GET | `/api/security/audit/:id` | `security.ts` |
| PATCH | `/api/security/audit/:id/resolve` | `security.ts` |
| POST | `/api/security/hptp/anomalies` | `security.ts` |
| GET | `/api/security/hptp/anomalies` | `security.ts` |
| GET | `/api/security/hptp/status` | `security.ts` |
| GET | `/api/security/hptp/fallback-analysis` | `security.ts` |
| GET | `/api/security/hptp/stats` | `security.ts` |
| GET | `/api/security/hptp/thresholds` | `security.ts` |
| GET | `/api/security/hptp/fallback-modes` | `security.ts` |
| GET | `/api/security/hptp/redundancy` | `security.ts` |
| POST | `/api/security/threats` | `security.ts` |
| GET | `/api/security/threats` | `security.ts` |
| GET | `/api/security/threats/risk-matrix` | `security.ts` |
| GET | `/api/security/threats/stats` | `security.ts` |
| GET | `/api/security/threats/meta` | `security.ts` |
| GET | `/api/security/threats/:id` | `security.ts` |
| PATCH | `/api/security/threats/:id` | `security.ts` |
| DELETE | `/api/security/threats/:id` | `security.ts` |
| POST | `/api/security/threats/seed` | `security.ts` |
| POST | `/api/security/implementation` | `security.ts` |
| GET | `/api/security/implementation` | `security.ts` |
| GET | `/api/security/implementation/summary` | `security.ts` |
| GET | `/api/security/implementation/metrics` | `security.ts` |
| GET | `/api/security/implementation/milestones` | `security.ts` |
| GET | `/api/security/implementation/meta` | `security.ts` |
| GET | `/api/security/implementation/:id` | `security.ts` |
| PATCH | `/api/security/implementation/:id` | `security.ts` |
| DELETE | `/api/security/implementation/:id` | `security.ts` |
| POST | `/api/security/implementation/seed` | `security.ts` |
| GET | `/api/security/metadata/categories` | `security.ts` |
| GET | `/api/security/metadata/types` | `security.ts` |

Platform-internal security admin, HPTP anomaly detection, threat model, implementation tracker.

## 3.15 Capability Token — Platform-Only

| Method | Path | Source |
|---|---|---|
| GET | `/api/capabilities/demo/expiration` | `capabilities.ts` |
| GET | `/api/capabilities/demo/delegation` | `capabilities.ts` |
| GET | `/api/capabilities/demo/confinement` | `capabilities.ts` |
| GET | `/api/capabilities/demo/certificates` | `capabilities.ts` |
| GET | `/api/capabilities/demo/mesh` | `capabilities.ts` |
| POST | `/api/capabilities/certificate/evidence-chain` | `capabilities.ts` |
| GET | `/api/capabilities/certificate/stats` | `capabilities.ts` |
| GET | `/api/capabilities/certificate/:certId/verify-data` | `capabilities.ts` |
| POST | `/api/capabilities/mesh/register` | `capabilities.ts` |
| POST | `/api/capabilities/mesh/issue` | `capabilities.ts` |
| POST | `/api/capabilities/mesh/propagate` | `capabilities.ts` |
| GET | `/api/capabilities/mesh/discover` | `capabilities.ts` |
| POST | `/api/capabilities/mesh/validate` | `capabilities.ts` |
| GET | `/api/capabilities/mesh/topology` | `capabilities.ts` |
| GET | `/api/capabilities/mesh/health` | `capabilities.ts` |

Capability demos and service mesh (inter-service, not LAN).

## 3.16 Tribonacci & Agent Array

| Method | Path | Source |
|---|---|---|
| GET | `/api/tribonacci/hook` | `tribonacci.ts` |
| GET | `/api/tribonacci/permutation` | `tribonacci.ts` |
| GET | `/api/tribonacci/coverage` | `tribonacci.ts` |
| GET | `/api/tribonacci/hash` | `tribonacci.ts` |
| GET | `/api/tribonacci/sequence` | `tribonacci.ts` |
| POST | `/api/tribonacci/generate-id` | `tribonacci.ts` |
| GET | `/api/tribonacci/next-worker` | `tribonacci.ts` |
| GET | `/api/tribonacci/skip-lookup` | `tribonacci.ts` |
| GET | `/api/tribonacci/hash-distribution` | `tribonacci.ts` |
| POST | `/api/tribonacci/agent-array` | `agent-array.ts` |
| GET | `/api/tribonacci/agent-array/stream/:sessionId` | `agent-array.ts` |
| POST | `/api/tribonacci/agent-array/save` | `agent-array.ts` |
| GET | `/api/tribonacci/agent-array/reports` | `agent-array.ts` |
| GET | `/api/tribonacci/agent-array/reports/:id` | `agent-array.ts` |
| GET | `/api/tribonacci/agent-array/positions` | `agent-array.ts` |

Developer tools and AI agent array.

## 3.17 Ephemeris

| Method | Path | Source |
|---|---|---|
| POST | `/api/ephemeris/convert` | `ephemeris.ts` |
| POST | `/api/ephemeris/position` | `ephemeris.ts` |
| POST | `/api/ephemeris/batch` | `ephemeris.ts` |
| GET | `/api/ephemeris/info` | `ephemeris.ts` |

Astronomical tool.

## 3.18 GDPR / Data Subject Rights

| Method | Path | Source |
|---|---|---|
| GET | `/api/gdpr/data-export` | `data-subject-rights.ts` |
| DELETE | `/api/gdpr/delete-account` | `data-subject-rights.ts` |
| GET | `/api/gdpr/requests` | `data-subject-rights.ts` |
| GET | `/api/gdpr/policy` | `data-subject-rights.ts` |

Web platform compliance.

## 3.19 TSA — PlenumNET-Only Subset

| Method | Path | Source |
|---|---|---|
| GET | `/api/tsa/certificate/download` | `tsa.ts` |
| GET | `/api/tsa/tokens` | `tsa.ts` |
| GET | `/api/tsa/policy` | `tsa.ts` |

PEM download, admin token log, policy info.

---

# Appendix: Cross-Reference by PlenumLAN Module

| LAN Module | Mapped From | Gaps (Net-New) |
|---|---|---|
| TDNS-L | TDNS scan, resolve, list, health | Dual-stack DNS, LAN scan |
| CRS-L | CRS register/lookup/heartbeat/deregister/stats | Address module, bitmap, IPv4 collision, nearest-service |
| PDS | Capabilities (issue/validate/delegate), TDNS orgs | User enrollment, auth, sessions, issuance rules, delegation, Merkle audit, AD importer |
| PFS | *(none)* | SMB 3.1.1 + NFS v4.2 bridge, backups, updates, print drivers |
| Shims | *(none)* | RADIUS, LDAP, DHCP, print bridge |
| PTS | *(none)* | Phase-encrypted tunnel, WebAuthn, scoped remote access |
| Crypto Kernel | Salvi crypto (hash, TL-DSA, phase, timing) | Direct Rust calls (no bridge) |
| GLB/CON/FTS | Inter-Cube GLB/CON/FTS endpoints | Identical logic; activates on second-site |
| Console | *(none)* | 16 management screens |
| Infrastructure | *(none)* | Setup wizard, installer, auto-update, PlenumDB, emergency console |

---

*Document produced from full endpoint inventory of PlenumNET mapped against TM-2026-019.2 §5–§24.*
*All source files under `server/routes/`. All LAN targets under `plenumlan/src/`.*
