# PlenumNET Framework Marketing Website

## Overview
PlenumNET is developing post-quantum internet solutions. This project creates a marketing website to showcase PlenumNET's quantum-resistant infrastructure, including the PlenumDB product with a compression demo and whitepaper management. It integrates payment processing and blockchain witnessing for secure, verifiable, and regulatory-compliant operations in quantum-resistant data and financial services. The project aims to position PlenumNET as a leader in next-generation internet solutions, offering unparalleled security and performance in the quantum-resistant internet domain.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend uses React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui`, and Wouter, supporting light/dark modes and single-layout navigation. Key pages include a Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, TSA Time-Stamping Authority, and an Admin Dashboard. It features a quantum-ternary simulator and FIPS 140-3 compliance checks. The homepage performance section displays real benchmark data from `salvi-bench`, including a 6-card benchmark grid, TL-DSA bar chart, and binary-vs-ternary comparison table, with constants managed in `shared/constants.ts`.

### Backend and Core Framework
The backend is built with Express.js and Node.js, using PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic, Femtosecond Timing, and Phase Encryption. The architecture includes microservices for payment processing and blockchain witnessing, a Femtosecond Timing Service, and a Certification Service. Security features include tiered rate limiting, CORS, Helmet.js, AES-256-GCM token encryption, input validation, hardened path sanitization, and API versioning.

### Rust Kernel Architecture
A Rust-based kernel provides core functionalities: Ternary Operations (GF(3) arithmetic), Femtosecond-precision Timing (HPTP), Phase Encryption, and a 3-Tier Security System (`SecurityTier` enum). It includes Cryptographic Primitives (ternary hash, TL-KEM, TL-DSA, CNSA 2.0 compliance), a Torsion Network (N-dimensional torus topology, Ternary Transport/Transfer/DNS), and a Ternary Virtual Machine (176-opcode ISA, ternary addressing, three-ring privilege levels, quantum-ternary simulation, ternary-aware garbage collector). A Binary Compatibility Layer handles balanced ternary conversion and crypto interoperability. Formal verification is performed using Kani and MIRI. The kernel also supports a bare-metal boot target for x86_64 validation via QEMU. TL-DSA uses integer NTT for efficient polynomial multiplication and AVX2 vectorization for performance.

### XPlenum RISC-V Hardware Extension
A custom RISC-V extension integrated with CVA6 provides 21 custom instructions and 12 custom CSRs for ternary security operations, PQC acceleration, and compliance.

### TDNS v2.3 — Ternary Domain Name System
A standalone Rust crate implementing a 27-dimensional ontological addressing protocol (spec at `salvi_docs/specs/TDNS-v2.3-SPECIFICATION.md`). It replaces DNS, BGP, PKI, IGMP/PIM, and PTP within the managed fabric, featuring 15 modules + CLI binary: trit arithmetic, address management, subcube multicasting, schema definition, scan operations, TRN records, routing, derivation rules, CrsRegistry service, live network scanner (URL → 27 probe measurements → CubeAddr via ureq HTTP/DNS/TLS inspection), a Geometric Load Balancer (GLB) for data-plane forwarding with HPTP enforcement, sub-cube multicast, and anycast, a Fault Tolerance Service (FTS) for heartbeat-based failure detection, a Cube Overlay Network (CON) for PQ-native encrypted tunnels with BLAKE3-derived keys, an HTTP API layer with 11 typed endpoints, and a Metatronic Bridge for .plm→TDNS / legacy DNS resolution. The `tdns-scan` CLI binary provides scan, compare, and describe commands.

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
A digital notary service providing cryptographic proof-of-existence timestamps per RFC 3161. It features four TSA policies, a Merkle tamper-evident audit log, dual-signature (RSA-4096 + TL-DSA-87), HPTP timing integration, and ASN.1 wire protocol. The `plenum-stamp` CLI tool allows signing and verifying files.

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