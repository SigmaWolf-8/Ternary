# PlenumNET Framework Marketing Website

## Overview
PlenumNET is developing post-quantum internet solutions. This project creates a marketing website to showcase PlenumNET's quantum-resistant infrastructure, including the PlenumDB product with a compression demo and whitepaper management. It integrates payment processing and blockchain witnessing for secure, verifiable, and regulatory-compliant operations in quantum-resistant data and financial services. The project aims to position PlenumNET as a leader in next-generation internet solutions, offering unparalleled security and performance in the quantum-resistant internet domain.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend uses React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui`, and Wouter, supporting light/dark modes and single-layout navigation. Key pages include a Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, TSA Time-Stamping Authority, and an Admin Dashboard. It features a quantum-ternary simulator and FIPS 140-3 compliance checks. The homepage performance section displays real benchmark data from `salvi-bench`, including a 6-card benchmark grid, TL-DSA bar chart, and binary-vs-ternary comparison table, with constants managed in `shared/constants.ts`.

### Backend and Core Framework
The backend is built with Express.js and Node.js, using PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic, Femtosecond Timing, and Phase Encryption v3 (post-quantum, duplex-mode TL-Sponge-385-based GF(3) stream cipher with 364° geometric domain separation — IND-CPA secure). Phase Encryption v3 uses a single duplex sponge per encrypt (down from 4 calls), precomputed byte-to-trit LUTs, pre-allocated domain buffers, and a unified MAC that binds both phase halves and authenticates headers. Backward compatible with v2 format. WASM sponge bridge infrastructure prepared (requires wasm-pack build). The architecture includes microservices for payment processing and blockchain witnessing, a Femtosecond Timing Service, and a Certification Service. Security features include tiered rate limiting, CORS, Helmet.js, AES-256-GCM token encryption, input validation, hardened path sanitization, and API versioning.

### Rust Kernel Architecture
A Rust-based kernel provides core functionalities: Ternary Operations (GF(3) arithmetic), Femtosecond-precision Timing (HPTP), Phase Encryption, and a 3-Tier Security System. It includes Cryptographic Primitives (ternary hash, TL-KEM, TL-DSA, CNSA 2.0 compliance), a Torsion Network (N-dimensional torus topology, Ternary Transport/Transfer/DNS), and a Ternary Virtual Machine (176-opcode ISA, ternary addressing, three-ring privilege levels, quantum-ternary simulation, ternary-aware garbage collector). A Binary Compatibility Layer handles balanced ternary conversion and crypto interoperability. Formal verification is performed using Kani and MIRI.

### XPlenum RISC-V Hardware Extension
A custom RISC-V extension integrated with CVA6 provides 21 custom instructions and 12 custom CSRs for ternary security operations, PQC acceleration, and compliance.

### Sponge Architecture
TL-Sponge-385 provides 385-bit post-quantum security for signing, key derivation, FIPS validation, and document hashing. The sponge permutation includes a χ(x) = x¹⁷ chi layer over GF(27) = GF(3)[t]/(t³+2t+1), applied to 243 three-trit blocks per round before theta. Sponge is versioned: v1 (no chi, for backward compat) and v2 (with chi, default). Implementations: TypeScript (`server/crypto/sponge-hash.ts`), Rust kernel (scalar + AVX2 + NEON SIMD), Rust ternary-math (scalar). Phase encryption uses `spongeVersion` field for backward-compatible decryption. TL-Sponge-43 is used for TDNS identity derivation. TIS-27 is used for fast integrity checks, wire packet integrity, and scan hashing.

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

### Inter-Cube Infrastructure Services
A 4-service system provides geometric routing across the 13D ternary cube network: Geometric Load Balancer (GLB), Cube Overlay Network (CON), Cube Registration Service (CRS), and Fault Tolerance Service (FTS). It is implemented as a Rust crate with TypeScript API routes.

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