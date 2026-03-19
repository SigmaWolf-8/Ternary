# PlenumNET Framework Marketing Website

## Overview
PlenumNET is developing post-quantum internet solutions. This project creates a marketing website to showcase PlenumNET's quantum-resistant infrastructure, including the PlenumDB product with a compression demo and whitepaper management. It integrates payment processing and blockchain witnessing for secure, verifiable, and regulatory-compliant operations in quantum-resistant data and financial services. The project aims to position PlenumNET as a leader in next-generation internet solutions, offering unparalleled security and performance in the quantum-resistant internet domain.

## Platform Metrics
Total platform LOC: ~293,500 across 12 languages (source only, excluding .d.ts declarations and package caches). Tests passing: 2,690 (Rust 374 + Vitest 577 + remaining platform suites). TTC v2.0 module: 38/38 tests passing (incl. benchmark). Inter-Cube benchmark: 68/92 primitives (74% pass rate), 79/92 PQ-verifiable. Ternary-math: 434/434 passing (incl. TL-KEM 29, ternary_lattice 24, Phase Encryption 44 + 14 cross-compat integration). Breakdown by language: Rust 112,466 (38.6%) | TypeScript 57,980 (19.9%) | TSX 53,893 (18.5%) | Python 39,395 (13.5%) | HTML 16,387 (5.6%) | SystemVerilog 3,100 (1.1%) | YAML 2,536 (0.9%) | JavaScript 2,403 (0.8%) | Shell 2,035 (0.7%) | TOML 581 (0.2%) | CSS 273 (0.1%) | SQL 283 (0.1%).

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend uses React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui`, and Wouter, supporting light/dark modes and single-layout navigation. Key pages include a Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, TSA Time-Stamping Authority, and an Admin Dashboard. It features a quantum-ternary simulator and FIPS 140-3 compliance checks. The homepage performance section displays real benchmark data from `salvi-bench`, including a 6-card benchmark grid, TL-DSA bar chart, and binary-vs-ternary comparison table, with constants managed in `shared/constants.ts`. All API service definitions (29 services, 301 endpoints, 7 domain groups) live in `shared/service-catalog.ts` as the single source of truth — the API Demo page, Kong Konnect page, and backend Kong routes all derive from this file.

### Backend and Core Framework
The backend is built with Express.js and Node.js, using PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic, Femtosecond Timing, and Phase Encryption v3 (post-quantum, duplex-mode TL-Sponge-385-based GF(3) stream cipher with 364° geometric domain separation — IND-CPA secure). Phase Encryption v3 uses a single duplex sponge per encrypt (down from 4 calls), precomputed byte-to-trit LUTs, pre-allocated domain buffers, and a unified MAC that binds both phase halves and authenticates headers. Backward compatible with v2 format. WASM sponge bridge infrastructure prepared (requires wasm-pack build). The architecture includes microservices for payment processing and blockchain witnessing, a Femtosecond Timing Service, and a Certification Service. Security features include tiered rate limiting, CORS, Helmet.js, AES-256-GCM token encryption, input validation, hardened path sanitization, and API versioning.

### Inter-Cube Infrastructure Services
A 4-service system provides geometric routing across the 13D ternary cube network: Geometric Load Balancer (GLB), Cube Overlay Network (CON), Cube Registration Service (CRS), and Fault Tolerance Service (FTS). It is implemented as a Rust crate (`services/inter-cube/`) with TypeScript API routes. The dimension count 13 is forced by an 8-constraint formal system (unique solution). Each populated cube contains 20,726,199 unique PQ-encrypted tunnels (26 × 3¹³ / 2). Structural resilience: at max Hamming distance, d!=6.2B shortest paths exceed 1.59M nodes by ~4,000×. Topology-derived keys via `TLSponge-385("PlenumNET-CON-v3.0" ‖ canonical(addr_a, addr_b) ‖ kem_shared_secret ‖ epoch)`. Cubes-of-Cubes scaling: Level 2 = 3²⁶ = 2.54T nodes. SPEC-2026-NEXT Phases 0–2 complete (T-01 through T-20): 12,865 LOC across 15 Rust modules, 324 passing tests. Modules: `wire.rs` (1,203 LOC — wire protocol, dual checksum), `api.rs` (554 LOC — REST API), `config.rs` (348 LOC — feature flags), `crs.rs` (1,328 LOC — signed registrations), `overlay.rs` (1,132 LOC — neighbor verification, v2.5 fallback removed, Authenticating state), `fts.rs` (1,211 LOC — authenticated heartbeats), `rate_limit.rs` (864 LOC — sliding window, PoW, ghost scoring), `identity.rs` (940 LOC — master secret, arc-epoch rotation), `persistence.rs` (818 LOC — heartbeat sequence persistence), `tunnel_auth.rs` (979 LOC — 3-message mutual tunnel authentication), `address_keys.rs` (790 LOC — address-bound TL-DSA-87 identity keys, LRU cache), `placement.rs` (832 LOC — geometry-aware bootstrap placement, max-spread K=2 floor, DimensionDensity tracker), `wire_ecc.rs` (599 LOC — 8-trit ECC syndrome, single-trit error correction, 26 unique syndrome patterns), `key_rotation.rs` (628 LOC — arc-synchronized key rotation orchestrator, jitter-aware epoch boundaries, rekey helpers), `verify_cache.rs` (641 LOC — CRS verification LRU+TTL cache, CrsCacheManager, unified telemetry stats).

### Rust Kernel Architecture
A Rust-based kernel provides core functionalities: Ternary Operations (GF(3) arithmetic), Femtosecond-precision Timing (HPTP), Phase Encryption, and a 3-Tier Security System. It includes Cryptographic Primitives (ternary hash, TL-KEM, TL-DSA, CNSA 2.0 compliance), a Torsion Network (N-dimensional torus topology, Ternary Transport/Transfer/DNS), and a Ternary Virtual Machine (176-opcode ISA, ternary addressing, three-ring privilege levels, quantum-ternary simulation, ternary-aware garbage collector). A Binary Compatibility Layer handles balanced ternary conversion and crypto interoperability. Formal verification is performed using Kani and MIRI.

### XPlenum RISC-V Hardware Extension
A custom RISC-V extension integrated with CVA6 provides 21 custom instructions and 12 custom CSRs for ternary security operations, PQC acceleration, and compliance.

### TL-KEM — Ternary Lattice Key Encapsulation
TL-KEM is a ternary-native equivalent of ML-KEM (FIPS 203) providing IND-CCA2 secure key encapsulation at three security levels: TL-KEM-512 (k=2, NIST Level 1, 128-bit), TL-KEM-768 (k=3, Level 3, 192-bit), TL-KEM-1024 (k=4, Level 5, 256-bit). Built on Module-LWE over R_q = Z_3[X]/(X^256+1) with Fujisaki-Okamoto transform, constant-time ciphertext comparison, and implicit rejection. Implementations: Rust kernel (`src/kernel/src/crypto/tl_kem.rs`), Rust ternary-math (`ternary-math/src/tl_kem.rs` + `ternary_lattice.rs`). The ternary-math port uses TL-Sponge-385 as the hash primitive and negacyclic integer NTT (q=12289, n=256) for accelerated matrix-vector multiplication. `SharedSecret::to_bytes_32()` produces 32-byte output compatible with `sponge385_derive_key`'s `kem_shared_secret` parameter.

### Sponge Architecture
TL-Sponge-385 provides 385-bit post-quantum security for signing, key derivation, FIPS validation, and document hashing. The sponge permutation includes a χ(x) = x¹⁷ chi layer over GF(27) = GF(3)[t]/(t³+2t+1), applied to 243 three-trit blocks per round before theta, with a precomputed CHI_MAP[27] lookup table (compile-time in Rust, startup IIFE in TypeScript) replacing per-block polynomial multiplication. Sponge is versioned: v1 (no chi, for backward compat), v2 (with chi, sequential), and v3 (chi + tree-parallel keystream via rayon, parallel GF(3) cipher, parallel primary/secondary phases via `rayon::join`, independent MAC sponge). Implementations: TypeScript (`server/crypto/sponge-hash.ts`), Rust kernel (scalar + AVX2 split-table vpshufb+blendv + NEON vtbl1q lo/hi), Rust ternary-math (scalar + AVX2/NEON SIMD, ported from kernel with runtime `is_x86_feature_detected!("avx2")` dispatch). A Rust N-API native addon (`ternary-math/napi/`, compiled to `server/crypto/sponge-native.node`) provides compiled native permutation to Node.js via napi-rs with AVX2 SIMD permutations (~10 MB/s raw hash, ~2 MB/s phase encrypt, ~4.3 µs/permutation), plus TTC v4.2 compress/decompress (`ttcCompress`/`ttcDecompress` N-API exports). TypeScript sponge stays as automatic fallback. Phase encryption uses `spongeVersion` field for backward-compatible decryption. TL-Sponge-43 is used for TDNS identity derivation. TIS-27 is used for fast integrity checks, wire packet integrity, and scan hashing.

### TTC v4.2 Compression Pipeline
File compression (`/api/compression/file`, `/api/compression/decompress`) uses the TTC v4.2 native Rust engine via N-API (`server/crypto/sponge-native.node`). Pipeline: domain analysis → ternary rANS with pure 3^k window/chunk sizes → GURFT fast-path → compact delta-varint freq tables. `compression-layer.ts` auto-detects the native addon and falls back to legacy zlib+ternary if unavailable. TTC metadata (engine, version, mode, level, CRC32, tau/delta, predominant base, adaptive rep) is propagated via `X-TTC-*` response headers (raw binary transport) and `ttcMetadata` JSON field. Frontend displays TTC metadata badges (engine, mode, level, CRC32 verification). Round-trip verified byte-perfect with CRC32 integrity checks.

### TDNS v2.5.0 — Ternary Domain Name System
A standalone Rust crate implementing a 27-dimensional ontological addressing protocol with 54-trit dual-layer addressing. It uses TL-Sponge-43 for identity derivation and TIS-27 for wire packet integrity. The system supports Org Entities and formally analyzed scaling. API routes are provided for scan, registration, resolution, and organization management.

### Tonal Diffusion System
This system enables network-wide time synchronization using FM timing packets, a toroidal topology, and gradient-driven diffusion consensus.

### Legal & IP Compliance
All source files include standardized copyright headers. Legal documents (terms, privacy, security) are served dynamically.

### Saturnian Tesseract Metatron Ternary Cube
A 13-dimensional ternary cube concept unifies geometric objects, defined by a Rust kernel module.

### Quantum Ternary Modules
Five modules provide classical simulation of quantum ternary (qutrit/qudit) operations.

### 28-Dimension Agent Array
This system orchestrates 28 specialist AI agents for parallel query analysis, featuring an Etymology Audit, Veritas Fact-Check, unified Situation Report generation, and Lexical Protocol enforcement.

### RFC 3161 Time-Stamping Authority (TSA)
A digital notary service providing cryptographic proof-of-existence timestamps per RFC 3161, featuring Merkle tamper-evident audit logs and dual-signature (RSA-4096 + TL-DSA-87).

### TL-Sponge-385 Document Hashing
A TypeScript port of TL-Sponge-385 for document hashing, exposed via a dedicated API endpoint.

### Hedera HCS Witnessing
Blockchain-based non-repudiation via Hedera Consensus Service for immutable, ordered, timestamped proof of PlenumNET operations.

### SFK Operations Pipeline
Manages Salvi Framework Kernel operation lifecycle: initialization → ternary_processing → witnessing → finalization.

### API Key Management System
A comprehensive system handles API key generation, validation, rotation, per-key rate limiting, and audit trails.

### Security Middleware Stack
Includes 4-tier rate limiting, CORS, Helmet.js security headers, AES-256-GCM token encryption, null-byte stripping, double URL-decode protection, and `execFile()`-only subprocess execution.

### Capability-Based Security
Authorization uses unforgeable, self-contained, bearer-verified capability tokens signed with TL-DSA, implemented across six phases.

### Security Infrastructure Services
Admin-protected backend services include a Security Audit Service, HPTP Anomaly Detection, Threat Model Registry, Implementation Status Tracker, and a Security Dashboard.

### Ternary Ephemeris API
A REST API provides endpoints for converting standard to ternary degrees with resonance scoring, calculating planet ephemeris, and retrieving API metadata.

## External Dependencies

-   **Authentication**: Replit Auth (GitHub, Google, Apple, X, email/password).
-   **Database**: PostgreSQL.
-   **ORM**: Drizzle ORM.
-   **API Gateway**: Kong Konnect.
-   **Payment Gateways**: Stripe, Interac, various cryptocurrency platforms.
-   **Message Queue**: BullMQ.
-   **Blockchain Platforms**: Hedera Hashgraph Consensus Service (HCS), XRP Ledger (XRPL), Algorand.
-   **Containerization**: Docker.
-   **Cloud Deployment**: Render, Railway.