# PlenumNET Framework Marketing Website

## Overview
PlenumNET is developing post-quantum internet solutions. This project creates a marketing website to showcase PlenumNET's quantum-resistant infrastructure, including the PlenumDB product with a compression demo and whitepaper management. It integrates payment processing and blockchain witnessing for secure, verifiable, and regulatory-compliant operations in quantum-resistant data and financial services. The project aims to position PlenumNET as a leader in next-generation internet solutions, offering unparalleled security and performance.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend is built with React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui`, and Wouter. It features light/dark modes, a single-layout navigation system, and includes a Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, TSA Time-Stamping Authority (`/tsa`), and an Admin Dashboard. It also incorporates a quantum-ternary simulator (`/quantum-sim`) for interactive fault-tolerance simulations and FIPS 140-3 compliance checks.

### Backend and Core Framework
The backend uses Express.js and Node.js with PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic, Femtosecond Timing, and Phase Encryption. The architecture includes microservices for payment processing and blockchain witnessing, a Femtosecond Timing Service, and a Certification Service. Security features include tiered rate limiting, CORS, Helmet.js, AES-256-GCM token encryption, input validation, and hardened path sanitization, with API versioning support.

### Rust Kernel Architecture
A Rust-based kernel provides core functionalities: Ternary Operations (GF(3) arithmetic), Femtosecond-precision Timing (HPTP), Phase Encryption, Memory Subsystem, Synchronization Primitives, Process Management, Modal Security System (domain management, capability-based access control), and Cryptographic Primitives (ternary hash, sponge, HMAC, KDF, Lamport signatures, AES-256-GCM, SHA-2/SHA-3, TL-KEM, TL-DSA, GF(3) polynomial arithmetic, CNSA 2.0 compliance). It also features a Device Driver Framework, I/O Subsystem, Filesystem, Torsion Network (N-dimensional torus topology, greedy geodesic routing, Ternary Transport Protocol, Ternary Transfer Protocol, Ternary DNS), and a Ternary Virtual Machine (176-opcode ISA v2.1, ternary addressing, three-ring privilege levels, quantum-ternary simulation, ternary-aware garbage collector). A Binary Compatibility Layer is included for balanced ternary conversion and crypto interoperability.

### XPlenum RISC-V Hardware Extension
A CVA6-integrated custom RISC-V extension provides 21 custom instructions and 12 custom CSRs for ternary security operations, including masking, domain management, capability handling, and cryptography. It incorporates NIST SP 800-90A CTR_DRBG for random number generation, Rust kernel interfaces for instruction access, and robust emulation and validation frameworks. Formal verification, extensive testing, and compliance documentation (FIPS 140-3, CNSA 2.0) are integral. Advanced features include full-system formal verification, higher-order DOM masking, PQC acceleration (10 new instructions for ML-KEM/ML-DSA), red-team adversarial validation, and Common Criteria GPCP cPP SFR mapping.

### Tonal Diffusion System
This system enables network-wide time synchronization using FM timing packets, a toroidal topology, and gradient-driven diffusion consensus. It consists of a Rust FM Timing Engine, a shared topology definition, a Tonal Field Service, and a Resonance Detector. An API provides endpoints for field data, neighbor information, packet handling, resonance status, and metrics.

### Legal & IP Compliance
All source files include standardized copyright headers. Legal documents (terms, privacy, security) are served dynamically. `IP-NOTICE.md` documents patent-pending claims, and `EXPORT-CONTROL.md` provides CNSA 2.0 and Wassenaar classification guidance.

### Saturnian Tesseract Metatron Ternary Cube
The 13-dimensional ternary cube is a geometric object unifying Metatron's 13 circles, the Saturnian Black Cube, the Saturnian Magic Square, the ternary circle, and the torsion network. A Rust kernel module defines its structure, including named axes, Saturnian shells, correspondence edges, embedded polytopes, Metatronic automorphisms, sponge embedding, structured 12D→3D projection, Z₂₈ angular relationships, and Rep C bijective axis numbering. Supporting modules handle permutations, keyed sponges, and address validation. A network bridge connects MetatronicVertex to TorusCoordinate with shell-aware hop classification and Saturnian-weighted torus distance. A TypeScript parallel provides full port importing from related modules.

### Quantum Ternary Modules
Five modules provide classical simulation of quantum ternary (qutrit/qudit) operations, covering complex utilities, qutrit basics (states, gates, probabilities), Lagrangian qutrit evolution, qutrit fault tolerance (error operators, stabilizer codes), and generalized qudit basics.

### 28-Dimension Agent Array
This system orchestrates 28 specialist AI agents for parallel query analysis, featuring an Etymology Audit, Veritas Fact-Check, unified Situation Report generation, and Lexical Protocol enforcement.

### RFC 3161 Time-Stamping Authority (Kong #21)
A digital notary service providing cryptographic proof-of-existence timestamps per RFC 3161. Features 5 policy tiers (DEFAULT, COMPLY, FORENSICS, SENTINEL, SECURE), Merkle tamper-evident audit log, dual-signature (RSA-4096 + post-quantum TL-DSA), HPTP timing integration, and ASN.1 wire protocol. It includes 9 endpoints for timestamping, verification, certificate retrieval, and auditing. Keys and Merkle state persist at `server/crypto/tsa-keys/`. CMS SignedData uses SET-encoded signed attributes for verification per RFC 5652. A Calendar Context Extension embeds multi-calendar timestamps as non-critical ASN.1 extensions, supporting 42 calendar systems with auto-embedding for financial markets and all for forensics.

### Hedera HCS Witnessing (Kong #22)
Blockchain-based non-repudiation via Hedera Consensus Service. Submits cryptographic witness hashes (SHA-256/384/512) to an HCS topic for immutable, ordered, timestamped proof of PlenumNET operations. 6 endpoints at `/api/hedera/v1/*`: witness submission, status lookup, mirror node verification, topic info, health check, and audit stats. Gracefully degrades when credentials are absent — logs a warning and skips route mounting. Requires `HEDERA_ACCOUNT_ID` and `HEDERA_PRIVATE_KEY` environment variables. Auto-creates topic on first boot and persists ID to `server/crypto/tsa-keys/hedera-topic-id.txt`. Local audit trail at `server/crypto/tsa-keys/hedera-witness-audit.jsonl`. Files: `server/services/hedera-witnessing-service.ts`, `server/routes/hedera.ts`.

### SFK Operations Pipeline (Kong #23)
Manages Salvi Framework Kernel operation lifecycle: initialization → ternary_processing → witnessing → finalization. Mode φ and φ+ operations submit SHA-256 result hashes to Hedera HCS for blockchain non-repudiation. Mode 1 and 0 skip witnessing. 5 endpoints at `/api/sfk/v1/*`: submit operation (202), poll status, list operations, cancel, and stats. In-memory operation tracking with 1-hour retention and automatic eviction. Files: `server/services/sfk-operations-service.ts`, `server/routes/sfk-operations.ts`.

### API Key Management System
A comprehensive system at `/api-keys` handles API key generation, validation (constant-time, scope, optional HPTP timing-bound), rotation (manual/auto), per-key rate limiting (three tiers), and audit trails. It includes anomaly detection for usage spikes, high failure rates, and IP dispersion. A WBS Tagging System allows entity classification and project metadata, with robust search and filtering capabilities. A dashboard provides stats, alerts, and management controls.

### Security Middleware Stack
Includes 4-tier rate limiting, CORS, Helmet.js security headers, AES-256-GCM token encryption, null-byte stripping, double URL-decode protection, and `execFile()`-only subprocess execution.

### Capability-Based Security
Authorization via unforgeable, self-contained, bearer-verified capability tokens signed with TL-DSA. All signing uses the TL-DSA bridge (`server/crypto/tl-dsa-bridge.ts`) with managed keys from the Key Management Service (`server/crypto/key-management.ts`). RSA-4096 dual-signing uses real Node.js crypto (`server/crypto/rsa4096-signing.ts`) — no hash-based fallbacks. Six phases are live:
- **Phase 1**: Typed constraint registry (8 constraint types), capability token schema, Merkle-chained audit events with SHA3-256. Attenuation enforcement covers all 8 types: `vault_id`, `document_id`, `project_id`, `template`, `recipient_domain`, `ip_range`, `geo_country`, `max_uses` — fail-closed on unknown types. Files: `shared/types/capability.ts`, `shared/types/capability-constraints.ts`, `server/services/capability-service.ts`, `server/services/capability-audit-events.ts`.
- **Phase 2**: HPTP-bound expiration — nanosecond-precise token expiry using the femtosecond timing engine. No client clock consulted.
- **Phase 3**: HMAC-chained delegation (macaroon-style) — TL-DSA root signature with HKDF-derived HMAC attenuation chain. Root signature verified on both initial delegation and re-delegation (P1-02 closed). Delegated token validation checks constraints/expiration directly without re-signing (P1-03 closed). Authority can only diminish, never grow.
- **Phase 4**: Hardware-bound capabilities solving the confinement problem — TPM/enclave/HSM device registration with TL-DSA-65 keypairs, HPTP challenge-response authentication (nonce + timing window + TL-DSA signature verification), replay rejection via TTL-evicted nonce set, single-use chains with SHA3-256 hash-chained positions and first-use-wins enforcement. File: `server/services/capability-hardware-binding.ts`.
- **Phase 5**: RFC 3161 capability certificates — court-admissible timestamped certificates integrating with the real TSA service. Dual-signed (TL-DSA + RSA-4096 real crypto), Merkle inclusion proofs with full sibling-hash paths, evidence chain assembly, certificate revocation, tamper detection. Hedera HCS witnessing integrated — each certificate hash is submitted to Hedera for blockchain-anchored proof of existence (hedera_tx_id + mirror node verification URL in certificate and verification response). File: `server/services/capability-certificates.ts`.
- **Phase 6**: Inter-service capability mesh — service registration, service-to-service capability issuance with TL-DSA mesh signatures, capability propagation with per-hop attenuation (authority diminishes at each hop), cycle detection, fail-closed expiry policy, service discovery by resource pattern, topology retrieval, health monitoring. File: `server/services/capability-mesh.ts`.
- API routes at `server/routes/capabilities.ts` expose all 6 phases with Zod-validated endpoints and demo endpoints at `/api/capabilities/demo/{expiration,delegation,confinement,certificates,mesh}`.

### Security Infrastructure Services
Admin-protected backend services under `/api/security/` include a Security Audit Service, HPTP Anomaly Detection, Threat Model Registry, Implementation Status Tracker, and a Security Dashboard.

### Inter-Cube Infrastructure Services
A 4-service system providing pure geometric routing with no routing tables across the 13D ternary cube network. Services: Geometric Load Balancer (GLB) — greedy geodesic forwarding with flow affinity; Cube Overlay Network (CON) — encrypted PQ-Native tunnels between 26 geometric neighbors; Cube Registration Service (CRS) — bitmap address allocation over 3¹³ = 1,594,323 Rep C address space; Fault Tolerance Service (FTS) — heartbeat monitoring with dead-set publication. Rust crate at `services/inter-cube/` with TypeScript API routes at `server/routes/inter-cube.ts` exposing 15 endpoints under `/api/salvi/inter-cube/*`.

### Ternary Ephemeris API
A REST API for a separate Astrology App frontend provides endpoints for converting standard to ternary degrees with resonance scoring, calculating single planet ephemeris, batch processing for all planets, and retrieving API metadata. It supports various celestial bodies and uses a 364° circle, 13° ternary radian, and Z₂₈ lattice for its ternary system.

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