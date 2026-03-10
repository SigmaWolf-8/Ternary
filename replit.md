# PlenumNET Framework Marketing Website

## Overview
PlenumNET is developing post-quantum internet solutions. This project creates a marketing website to showcase PlenumNET's quantum-resistant infrastructure, including the PlenumDB product with a compression demo and whitepaper management. It integrates payment processing and blockchain witnessing for secure, verifiable, and regulatory-compliant operations in quantum-resistant data and financial services. The project aims to position PlenumNET as a leader in next-generation internet solutions, offering unparalleled security and performance in the quantum-resistant internet domain.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend uses React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui`, and Wouter, supporting light/dark modes and single-layout navigation. Key pages include a Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, TSA Time-Stamping Authority, and an Admin Dashboard. It features a quantum-ternary simulator and FIPS 140-3 compliance checks. The homepage performance section displays real benchmark data from `salvi-bench`, including a 6-card benchmark grid, TL-DSA bar chart, and binary-vs-ternary comparison table, with constants managed in `shared/constants.ts`.

### Backend and Core Framework
The backend is built with Express.js and Node.js, using PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic, Femtosecond Timing, and Phase Encryption (post-quantum, TL-Sponge-385-based GF(3) stream cipher with 364° geometric domain separation — IND-CPA secure via per-operation random nonce, constant-time LUT-based GF(3) arithmetic). Formal security analysis is in TM-2026-011 (`docs/proofs/Phase-Encryption-Security-Proof.md`): 12 sections, all 7 open problems closed — IND-CPA via keyed-sponge PRF reduction (Gaži et al. 2015), INT-CTXT via sponge MAC, AE via Bellare-Namprempre composition, multi-key μ-IND-CPA, key-committing at 2^{-385}, orthogonal security game (Exp^{PO}), Walsh spectrum verified (LP_max=DP_max=1/9), constant-time LUT analysis. Performance benchmarks in BR-2026-001 (`docs/security/phase-encryption-benchmarks.md`): ~133 KB/s peak throughput (balanced mode, 4KB payload), live at `GET /api/salvi/crypto/phase-benchmark`. MAC verification uses `timingSafeEqual` and is mandatory on all nonce-based decryptions. The architecture includes microservices for payment processing and blockchain witnessing, a Femtosecond Timing Service, and a Certification Service. Security features include tiered rate limiting, CORS, Helmet.js, AES-256-GCM token encryption, input validation, hardened path sanitization, and API versioning.

### Rust Kernel Architecture
A Rust-based kernel provides core functionalities: Ternary Operations (GF(3) arithmetic), Femtosecond-precision Timing (HPTP), Phase Encryption, and a 3-Tier Security System (`SecurityTier` enum). It includes Cryptographic Primitives (ternary hash, TL-KEM, TL-DSA, CNSA 2.0 compliance), a Torsion Network (N-dimensional torus topology, Ternary Transport/Transfer/DNS), and a Ternary Virtual Machine (176-opcode ISA, ternary addressing, three-ring privilege levels, quantum-ternary simulation, ternary-aware garbage collector). A Binary Compatibility Layer handles balanced ternary conversion and crypto interoperability. Formal verification is performed using Kani and MIRI. The kernel also supports a bare-metal boot target for x86_64 validation via QEMU. TL-DSA uses integer NTT for efficient polynomial multiplication and AVX2 vectorization for performance.

### XPlenum RISC-V Hardware Extension
A custom RISC-V extension integrated with CVA6 provides 21 custom instructions and 12 custom CSRs for ternary security operations, PQC acceleration, and compliance.

### Sponge Architecture — Three Implementations, Three Jobs
TL-Sponge-385 (`src/kernel/src/crypto/sponge.rs`) provides 385-bit post-quantum security (state=729, rate=243, capacity=486, 9 rounds, 7-neighbor extended theta) for signing, key derivation, FIPS validation, and document hashing. TL-Sponge-43 (`server/routes/tdns.ts`, mirrors `services/tdns-v2/src/identity.rs`) is the TDNS identity sponge (state=54, rate=27, capacity=27, 9 rounds, 3-neighbor theta, 43-bit preimage) for URL → identity anchor derivation. TIS-27 (`ternary-math/src/tis_sponge.rs` + `shared/tis-sponge.ts`) is the same proven sponge construction sized for fast integrity (state=54, 4 rounds, 7-neighbor theta, 43-bit cryptographic security, 191 ns). Wide-trail analysis proves DP ≤ 9⁻⁴⁰⁹⁶ (TM-2026-008). Used for wire packet integrity and scan hashing. For post-quantum operations, use TL-Sponge-385.

### TDNS v2.5.0 — Ternary Domain Name System
A standalone Rust crate implementing a 27-dimensional ontological addressing protocol. The server-side scanner (`server/routes/tdns.ts`) implements TDNS v2.5.0 with 54-trit dual-layer addressing (27 classification + 27 identity anchor). Identity derivation uses TL-Sponge-43 (state=54, capacity=27, 43-bit preimage). TIS-27 is used for wire packet integrity and scan hashing on already-authenticated channels. The system supports Org Entities for multi-URL grouping under a single handle. Scaling beyond 3¹³ is formally analyzed in TM-2026-012 (`docs/proofs/TM-2026-012-TDNS-Scaling-Analysis.md`): Level 1 (13-trit, 1.59M nodes), Level 2 (26-trit, 2.54T nodes), Level 3 (39-trit, 4.05 quintillion nodes). All levels use O(d) greedy forwarding with zero routing tables. The TDNS 27-dim ontological space and inter-cube 13-dim transport space are unified via CRS binding. API routes: `/api/tdns/scan`, `/api/tdns/register` (with `org_name` support), `/api/tdns/resolve/:name`, `/api/tdns/org/create`, `/api/tdns/org/add-url`, `/api/tdns/org/:name`, `/api/tdns/orgs`, `/api/tdns/list`, `/api/tdns/health`. The Chrome extension (v1.0.9, `services/tdns-v2/extension-chromium/`) renders dual-color addresses: classification in gold, identity anchor in sky blue. The Rust crate includes 15 modules + CLI binary: trit arithmetic, address management, subcube multicasting, schema definition, scan operations, TRN records, routing, derivation rules, CrsRegistry service, live network scanner, GLB, FTS, CON, HTTP API, and Metatronic Bridge.

### Tonal Diffusion System
This system enables network-wide time synchronization using FM timing packets, a toroidal topology, and gradient-driven diffusion consensus. It includes a Rust FM Timing Engine, a shared topology definition, a Tonal Field Service, and a Resonance Detector with an API.

### Legal & IP Compliance
All source files include standardized copyright headers. Legal documents (terms, privacy, security) are served dynamically. `IP-NOTICE.md` documents patent-pending claims, and `EXPORT-CONTROL.md` provides CNSA 2.0 and Wassenaar classification guidance.

### Saturnian Tesseract Metatron Ternary Cube
A 13-dimensional ternary cube concept unifies geometric objects, defined by a Rust kernel module. It includes named axes, Saturnian shells, correspondence edges, embedded polytopes, Metatronic automorphisms, and a network bridge.

### Quantum Ternary Modules
Five modules provide classical simulation of quantum ternary (qutrit/qudit) operations, covering complex utilities, qutrit basics, Lagrangian qutrit evolution, qutrit fault tolerance, and generalized qudit basics.

### 28-Dimension Agent Array
This system orchestrates 28 specialist AI agents for parallel query analysis, featuring an Etymology Audit, Veritas Fact-Check, unified Situation Report generation, and Lexical Protocol enforcement.

### RFC 3161 Time-Stamping Authority (TSA)
A digital notary service providing cryptographic proof-of-existence timestamps per RFC 3161. It features four TSA policies, a Merkle tamper-evident audit log (TL-Sponge-385 hashed), dual-signature (RSA-4096 + TL-DSA-87), HPTP timing integration, and ASN.1 wire protocol. The `plenum-stamp` CLI tool allows signing and verifying files. TL-Sponge-385 (OID `1.3.6.1.4.1.0.100.3.1`) is registered as a native hash algorithm for TSA timestamp requests.

### TL-Sponge-385 Document Hashing
TypeScript port of TL-Sponge-385 (`server/crypto/sponge-hash.ts`). Parameters: STATE=729, RATE=243, CAPACITY=486, ROUNDS=9, 7-neighbor theta, scatter pi. Output: 49 bytes (98 hex chars, 243 trits), 385-bit PQ security. Exposed via `POST /api/salvi/crypto/hash` (accepts JSON base64 or raw binary). Integration instructions in `SIGNHERE-INTEGRATION.md`.

### Hedera HCS Witnessing
Blockchain-based non-repudiation via Hedera Consensus Service. It submits cryptographic witness hashes to an HCS topic for immutable, ordered, timestamped proof of PlenumNET operations.

### SFK Operations Pipeline
Manages Salvi Framework Kernel operation lifecycle: initialization → ternary_processing → witnessing → finalization. Fortified-tier operations submit SHA-256 result hashes to Hedera HCS.

### API Key Management System
A comprehensive system handles API key generation, validation, rotation, per-key rate limiting, and audit trails. It includes anomaly detection and a WBS Tagging System.

### Security Middleware Stack
Includes 4-tier rate limiting, CORS, Helmet.js security headers, AES-256-GCM token encryption, null-byte stripping, double URL-decode protection, and `execFile()`-only subprocess execution.

### Capability-Based Security
Authorization uses unforgeable, self-contained, bearer-verified capability tokens signed with TL-DSA, implemented across six phases: typed constraint registry, HPTP-bound expiration, HMAC-chained delegation, hardware-bound capabilities, RFC 3161 capability certificates, and an inter-service capability mesh.

### Security Infrastructure Services
Admin-protected backend services include a Security Audit Service, HPTP Anomaly Detection, Threat Model Registry, Implementation Status Tracker, and a Security Dashboard.

### Inter-Cube Infrastructure Services
A 4-service system provides geometric routing across the 13D ternary cube network: Geometric Load Balancer (GLB), Cube Overlay Network (CON), Cube Registration Service (CRS), and Fault Tolerance Service (FTS). It is implemented as a Rust crate with TypeScript API routes. CON PQ-Native tunnel key derivation uses TIS-27 sponge KDF with canonical address ordering and domain separator `"PlenumNET-CON-v2.5"`. Formal security analysis in TM-2026-013 (`docs/proofs/TM-2026-013-Topology-Key-Agreement.md`): game-based Exp^{TDKA} security experiment, PRF reduction, adaptive corruption analysis (key independence under partial corruption), forward secrecy negative result with three mitigation paths (WireGuard composition, ephemeral salt injection, hash ratcheting), 43-bit TIS-27 capacity limitation identified with TL-Sponge-385 upgrade path for CNSA 2.0 compliance. Key finding: PQ-Native tunnel keys are deterministic PSKs from public inputs — confidentiality requires network-layer access control or WireGuard composition.

### Ternary Ephemeris API
A REST API provides endpoints for converting standard to ternary degrees with resonance scoring, calculating planet ephemeris, and retrieving API metadata.

## External Dependencies

-   **Authentication**: Replit Auth (GitHub, Google, Apple, X, email/password).
-   **Database**: PostgreSQL.
-   **ORM**: Drizzle ORM.
-   **API Gateway**: Kong Konnect (33 services, 293 endpoints).
-   **Payment Gateways**: Stripe, Interac, various cryptocurrency platforms.
-   **Message Queue**: BullMQ.
-   **Blockchain Platforms**: Hedera Hashgraph Consensus Service (HCS), XRP Ledger (XRPL), Algorand.
-   **Containerization**: Docker.
-   **Cloud Deployment**: Render, Railway.

## Skills

-   **plenumnet-repo-guide** (`.agents/skills/plenumnet-repo-guide/SKILL.md`): Complete A-Z structural guide to the PlenumNET / Salvi Framework repository (SigmaWolf-8/Ternary, 1,252+ commits, 80/80 milestones). Covers ternary mathematics (base-3, pi=14, 364-degree circle, 13x28 calendar), first-position derivation rules, TDNS v2.5 ontological addressing (19 Rust modules), Rep A/B/C trit encodings, Tribonacci constants, Saturnian geometry, Inter-Cube infrastructure, quantum ternary modules, XPlenum RISC-V extension, Rust kernel subsystems (176-opcode ISA v2.1), bare-metal validation (Kani/MIRI), TL-DSA/TL-KEM post-quantum crypto (34 crypto modules), Kong Konnect gateway (33 services, 293 endpoints), PlenumDB, SignHere e-signature integration, SFK Operations Pipeline, TL-Sponge-385 (729-trit, 9 rounds, 385-bit PQ security) for key derivation and signing, TL-Sponge-43 (54-trit, 9 rounds, 43-bit) for TDNS identity derivation, TIS-27 wire integrity sponge (54-trit, 4 rounds, 43-bit), TIS-81 benchmark variant (243-trit, 4 rounds, 257-bit), 42 calendar systems, and all codebase conventions. Use this skill when working on ANY PlenumNET feature, reviewing architecture, onboarding, writing code that touches the Salvi Framework, modifying the Ternary repo, debugging crypto or TDNS issues, building frontend pages, or discussing any Capomastro Holdings technical product. Always consult this skill before making changes — the invariants are load-bearing and violations break mathematical consistency across the entire framework.