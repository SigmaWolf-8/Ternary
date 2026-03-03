# PlenumNET Framework Marketing Website

## Overview
PlenumNET is developing post-quantum internet solutions. This project creates a marketing website to showcase PlenumNET's quantum-resistant infrastructure, including the PlenumDB product with a compression demo and whitepaper management. It integrates payment processing and blockchain witnessing for secure, verifiable, and regulatory-compliant operations in quantum-resistant data and financial services. The project aims to position PlenumNET as a leader in next-generation internet solutions, offering unparalleled security and performance.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend uses React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui`, and Wouter, featuring light/dark modes and a single-layout navigation. Key pages include a Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, TSA Time-Stamping Authority, and an Admin Dashboard. It also includes a quantum-ternary simulator for interactive fault-tolerance simulations and FIPS 140-3 compliance checks.

### Backend and Core Framework
The backend is built with Express.js and Node.js, using PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic, Femtosecond Timing, and Phase Encryption. The architecture includes microservices for payment processing and blockchain witnessing, a Femtosecond Timing Service, and a Certification Service. Security features encompass tiered rate limiting, CORS, Helmet.js, AES-256-GCM token encryption, input validation, and hardened path sanitization, with API versioning.

### Rust Kernel Architecture
A Rust-based kernel provides core functionalities such as Ternary Operations (GF(3) arithmetic), Femtosecond-precision Timing (HPTP), Phase Encryption, and a 3-Tier Security System (`SecurityTier` enum: Fortified/Verified/Basic — domain management, capability-based access control). Phase-split encryption is a boolean flag within the Fortified tier. It includes Cryptographic Primitives (ternary hash, TL-KEM, TL-DSA, CNSA 2.0 compliance), a Torsion Network (N-dimensional torus topology, Ternary Transport/Transfer/DNS), and a Ternary Virtual Machine (176-opcode ISA, ternary addressing, three-ring privilege levels, quantum-ternary simulation, ternary-aware garbage collector). A Binary Compatibility Layer handles balanced ternary conversion and crypto interoperability. A bare-metal boot target (`src/kernel/bare-metal/`) validates kernel boot on raw x86_64 hardware via QEMU. TL-DSA and TL-KEM use integer NTT (q=12289, ψ=3400 as primitive 512th root of unity, negacyclic convolution, n=256) with pre-NTT'd matrix A cached in public/secret keys (`NttMatrix`), reducing matrix-vector multiplies from O(k·l) Karatsuba calls to O(l+k) NTT calls. TL-DSA-87 full roundtrip: 2.8ms (5.2× speedup from 14.4ms). XOF-batched sponge expansion (single-squeeze for expand_A and sample_noise_vec) eliminates per-entry sponge overhead. Sponge permutation uses GF(3)-associative `balanced_wrap` (parallel integer sum + one conditional wrap) instead of serial `trit_add` chains — no tables, no division, first-principles only. Polynomial ring multiplication uses Karatsuba (O(n^1.585)) via `ring_mul_karatsuba` for individual products and `NttMatrix::mul_vec` (O(n log n)) for matrix-vector products in `ternary_lattice.rs`.

### Rust Formal Verification (Kani + MIRI)
Three Kani proof harness files provide 50 bounded model checking proofs across the kernel:
- `src/kernel/src/kani_proofs.rs` — 30 proofs: ternary arithmetic (GF(3) closure, commutativity, identity, involution), bijection round-trips (A↔B↔C), pack/unpack, rotation cycles, boot sequence length, timing bounds, phase encryption invariants.
- `src/kernel/src/crypto/kani_proofs.rs` — 18 proofs: constant-time utility correctness (ct_eq reflexive/symmetric/binary, ct_select, ct_is_zero, ct_lt, ct_zeroize completeness, ct_cmov_bytes).
- `src/kernel/src/vm/kani_proofs.rs` — 2 proofs: GC heap bounded allocation, no double-free aliasing.
GitHub Actions CI at `.github/workflows/formal-verification-rust.yml` runs Kani (bounded model checking), MIRI (undefined behavior detection), and dynamic verification (existing formal_verify.rs) as three parallel jobs.

### XPlenum RISC-V Hardware Extension
A custom RISC-V extension integrated with CVA6 provides 21 custom instructions and 12 custom CSRs for ternary security operations (masking, domain management, capability handling, cryptography). It includes NIST SP 800-90A CTR_DRBG and Rust kernel interfaces. Advanced features include full-system formal verification, PQC acceleration (10 new instructions for ML-KEM/ML-DSA), and FIPS 140-3/CNSA 2.0 compliance.

### Tonal Diffusion System
This system enables network-wide time synchronization using FM timing packets, a toroidal topology, and gradient-driven diffusion consensus. It comprises a Rust FM Timing Engine, a shared topology definition, a Tonal Field Service, and a Resonance Detector, with an API for field data, packet handling, and metrics.

### Legal & IP Compliance
All source files include standardized copyright headers. Legal documents (terms, privacy, security) are served dynamically. `IP-NOTICE.md` documents patent-pending claims, and `EXPORT-CONTROL.md` provides CNSA 2.0 and Wassenaar classification guidance.

### Saturnian Tesseract Metatron Ternary Cube
A 13-dimensional ternary cube concept unifies geometric objects, defined by a Rust kernel module. It includes named axes, Saturnian shells, correspondence edges, embedded polytopes, Metatronic automorphisms, and a network bridge connecting MetatronicVertex to TorusCoordinate.

### Quantum Ternary Modules
Five modules provide classical simulation of quantum ternary (qutrit/qudit) operations, covering complex utilities, qutrit basics, Lagrangian qutrit evolution, qutrit fault tolerance, and generalized qudit basics.

### 28-Dimension Agent Array
This system orchestrates 28 specialist AI agents for parallel query analysis, featuring an Etymology Audit, Veritas Fact-Check, unified Situation Report generation, and Lexical Protocol enforcement.

### RFC 3161 Time-Stamping Authority (Kong #21)
A digital notary service providing cryptographic proof-of-existence timestamps per RFC 3161. Features 4 TSA policies (COMPLY, FORENSICS, SENTINEL, SECURE) mapped from SecurityTier via `defaultTsaPolicy()` (Fortified→SENTINEL, Verified→COMPLY, Basic→null). Includes a Merkle tamper-evident audit log, dual-signature (RSA-4096 + post-quantum TL-DSA), HPTP timing integration, and ASN.1 wire protocol. It supports 9 endpoints for timestamping, verification, and auditing.

### Hedera HCS Witnessing (Kong #22)
Blockchain-based non-repudiation via Hedera Consensus Service. Submits cryptographic witness hashes to an HCS topic for immutable, ordered, timestamped proof of PlenumNET operations. Includes 6 endpoints for submission, status lookup, and verification.

### SFK Operations Pipeline (Kong #23)
Manages Salvi Framework Kernel operation lifecycle: initialization → ternary_processing → witnessing → finalization. Fortified-tier operations submit SHA-256 result hashes to Hedera HCS. It provides 5 endpoints for operation submission, status, listing, cancellation, and stats.

### API Key Management System
A comprehensive system at `/api-keys` handles API key generation, validation (constant-time, scope, optional HPTP timing-bound), rotation, per-key rate limiting, and audit trails. It includes anomaly detection, a WBS Tagging System for entity classification, and a dashboard.

### Security Middleware Stack
Includes 4-tier rate limiting, CORS, Helmet.js security headers, AES-256-GCM token encryption, null-byte stripping, double URL-decode protection, and `execFile()`-only subprocess execution.

### Capability-Based Security
Authorization uses unforgeable, self-contained, bearer-verified capability tokens signed with TL-DSA. It involves a 6-phase system:
- **Phase 1**: Typed constraint registry (8 constraint types), capability token schema, Merkle-chained audit events.
- **Phase 2**: HPTP-bound expiration for nanosecond-precise token expiry.
- **Phase 3**: HMAC-chained delegation (macaroon-style) with TL-DSA root signature and HKDF-derived HMAC attenuation chain.
- **Phase 4**: Hardware-bound capabilities solving the confinement problem via TPM/enclave/HSM device registration and HPTP challenge-response.
- **Phase 5**: RFC 3161 capability certificates, court-admissible timestamped certificates integrated with the TSA service and Hedera HCS witnessing.
- **Phase 6**: Inter-service capability mesh for service registration, service-to-service capability issuance, and propagation.
API routes at `server/routes/capabilities.ts` expose all 6 phases.

### Security Infrastructure Services
Admin-protected backend services under `/api/security/` include a Security Audit Service, HPTP Anomaly Detection, Threat Model Registry, Implementation Status Tracker, and a Security Dashboard.

### Inter-Cube Infrastructure Services
A 4-service system provides geometric routing across the 13D ternary cube network: Geometric Load Balancer (GLB), Cube Overlay Network (CON), Cube Registration Service (CRS), and Fault Tolerance Service (FTS). It's implemented as a Rust crate with TypeScript API routes.

### Ternary Ephemeris API
A REST API for an Astrology App frontend provides endpoints for converting standard to ternary degrees with resonance scoring, calculating single/batch planet ephemeris, and retrieving API metadata.

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