# PlenumNET Framework Marketing Website

## Overview
PlenumNET is developing post-quantum internet solutions. This project creates a marketing website to showcase PlenumNET's quantum-resistant infrastructure, including the PlenumDB product with a compression demo and whitepaper management. It integrates payment processing and blockchain witnessing for secure, verifiable, and regulatory-compliant operations in quantum-resistant data and financial services. The project aims to position PlenumNET as a leader in next-generation internet solutions, offering unparalleled security and performance, and leveraging advanced mathematical and physics concepts for its core technologies.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

### Single Source of Truth Pattern
Platform-wide constants (opcode count, API endpoint count, ISA version, etc.) are defined once in `shared/constants.ts` and imported everywhere. The frontend `client/src/lib/strings.ts` references these constants so all UI text auto-updates. When adding new endpoints or updating platform numbers, update `shared/constants.ts` only — frontend propagation is automatic. Documentation files (README.md, etc.) must be updated manually since they cannot import TypeScript.

## System Architecture

### Frontend
The frontend is built with React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui`, and Wouter. It features light/dark modes, a dual-layout navigation system (`MarketingLayout`, `DashboardLayout`), and includes a Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, and an Admin Dashboard. It incorporates a quantum-ternary simulator (`/quantum-sim`) for interactive fault-tolerance simulations and FIPS 140-3 compliance checks.

### Backend and Core Framework
The backend uses Express.js and Node.js with PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic, Femtosecond Timing, and Phase Encryption. The architecture includes microservices for payment processing and blockchain witnessing, a Femtosecond Timing Service, and a Certification Service. Security features include tiered rate limiting, CORS, Helmet.js, AES-256-GCM token encryption, input validation, and hardened path sanitization. API versioning is supported.

### Rust Kernel Architecture
A Rust-based kernel provides core functionalities: Ternary Operations (GF(3) arithmetic), Femtosecond-precision Timing (HPTP), Phase Encryption (split/recombine with timing-window enforcement), Memory Subsystem, Synchronization Primitives, Process Management, Modal Security System (domain management, capability-based access control), and Cryptographic Primitives (ternary hash, sponge, HMAC, KDF, Lamport signatures, AES-256-GCM, SHA-2/SHA-3, TL-KEM, TL-DSA, GF(3) polynomial arithmetic, CNSA 2.0 compliance). It also features a Device Driver Framework, I/O Subsystem, Filesystem, Torsion Network (N-dimensional torus topology, greedy geodesic routing, Ternary Transport Protocol, Ternary Transfer Protocol, Ternary DNS), and a Ternary Virtual Machine (176-opcode ISA v2.1, ternary addressing, three-ring privilege levels, quantum-ternary simulation, ternary-aware garbage collector). A Binary Compatibility Layer is included for balanced ternary conversion and crypto interoperability.

### XPlenum RISC-V Hardware Extension
A CVA6-integrated custom RISC-V extension provides 21 custom instructions and 12 custom CSRs for ternary security operations, including masking, domain management, capability handling, and cryptography. It incorporates NIST SP 800-90A CTR_DRBG for random number generation, Rust kernel interfaces for instruction access, and robust emulation and validation frameworks (Spike ISS, QEMU TCG helpers). Formal verification, extensive testing, and compliance documentation (FIPS 140-3, CNSA 2.0) are integral. Advanced features include full-system formal verification, higher-order DOM masking, PQC acceleration (10 new instructions for ML-KEM/ML-DSA), red-team adversarial validation, and Common Criteria GPCP cPP SFR mapping.

### Tonal Diffusion System
This system enables network-wide time synchronization using FM timing packets, a toroidal topology, and gradient-driven diffusion consensus. It consists of a Rust FM Timing Engine (Van der Pol oscillator, HRV entropy source, FM timing packet codec, GF(3) gradient operator), a shared topology definition (toroidal coordinates, natural neighbor selection, GF(3) arithmetic), a Tonal Field Service (field potential tracker, graph Laplacian diffusion solver, dimensionless Plenum metrics), and a Resonance Detector (adaptive sync rate tuning, resonant frequency detection). An API provides endpoints for field data, neighbor information, packet handling, resonance status, and metrics.

### Legal & IP Compliance
All source files include standardized copyright headers. Legal documents (terms, privacy, security) are served dynamically. `IP-NOTICE.md` documents patent-pending claims, and `EXPORT-CONTROL.md` provides CNSA 2.0 and Wassenaar classification guidance.

### PPTPro Integration
PPTPro (Plenum Pulse Tonal Professor V2.2) is a separate private Python package at `https://github.com/SigmaWolf-8/PPTPro` (managed via `GITHUB_PPTPro_Token`). It provides the tonal intelligence engine — HRV analysis, vascular coherence (C_VP), phase-advance target generation — consumed by PlenumNET as a private dependency. PPTPro's landing page deploys to `https://PPTPro.Replit.App`. The integration manifest is at `docs/pptpro-integration-manifest.md`. PPTPro depends on the Salvi Framework (`SigmaWolf-8/Ternary`) for bijective ternary logic.

### GitHub Integration
An admin-only GitHub Manager page allows browsing and push actions for the `SigmaWolf-8/Ternary` repository. The `SigmaWolf-8/PPTPro` repository is managed separately via the `GITHUB_PPTPro_Token` secret.

### Scientific Integrations
The system incorporates mathematical and physics concepts such as the Saturnian Magic Square Blueprint, Hamiltonian Mechanics (HPTP Symplectic Jitter Corrector, Hamiltonian VM Constraints, Symplectic Phase Mixing), Lagrangian Mechanics (Discrete Euler-Lagrange equations), Noether Symmetries (conserved quantities for ternary gauge), and Tribonacci Variational Methods.

### Quantum Ternary Modules
Five modules provide classical simulation of quantum ternary (qutrit/qudit) operations, covering complex utilities, qutrit basics (states, gates, probabilities), Lagrangian qutrit evolution, qutrit fault tolerance (error operators, stabilizer codes), and generalized qudit basics.

### 28-Dimension Agent Array
This system orchestrates 28 specialist AI agents for parallel query analysis, featuring an Etymology Audit, Veritas Fact-Check, unified Situation Report generation, and Lexical Protocol enforcement.

### Kong Gateway Integration
Kong Konnect API integration manages services, routes, and plugins for PlenumNET's 17 services.

### API Key Management System
A comprehensive system at `/api-keys` handles API key generation, validation (constant-time, scope, optional HPTP timing-bound), rotation (manual/auto), per-key rate limiting (three tiers), and audit trails. It includes anomaly detection for usage spikes, high failure rates, and IP dispersion. WBS Tagging System allows entity classification and project metadata, with robust search and filtering capabilities. A dashboard provides stats, alerts, and management controls.

### Security Middleware Stack
Includes 4-tier rate limiting, CORS, Helmet.js security headers, AES-256-GCM token encryption, null-byte stripping, double URL-decode protection, and `execFile()`-only subprocess execution.

### Security Infrastructure Services
Admin-protected backend services under `/api/security/` include a Security Audit Service, HPTP Anomaly Detection, Threat Model Registry, Implementation Status Tracker, and a Security Dashboard.

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