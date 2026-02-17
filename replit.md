# PlenumNET Framework Marketing Website

## Overview
PlenumNET is developing post-quantum internet solutions. This project creates a marketing website to showcase PlenumNET's quantum-resistant infrastructure, including the PlenumDB product with a compression demo and whitepaper management. It integrates payment processing and blockchain witnessing for secure, verifiable, and regulatory-compliant operations in quantum-resistant data and financial services. The project aims to position PlenumNET as a leader in next-generation internet solutions, offering unparalleled security and performance, and leveraging advanced mathematical and physics concepts for its core technologies.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend is built with React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui`, and Wouter. It features light/dark modes, a dual-layout navigation system (`MarketingLayout`, `DashboardLayout`), and includes a Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, and an Admin Dashboard. It incorporates ancient calendar synchronization and is optimized for SEO and PWA capabilities. A quantum-ternary simulator (`/quantum-sim`) provides interactive fault-tolerance simulations, FIPS 140-3 compliance checks, and variational benchmarks.

### Backend and Core Framework
The backend uses Express.js and Node.js with PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic, Femtosecond Timing, and Phase Encryption. The architecture includes microservices for payment processing and blockchain witnessing, a Femtosecond Timing Service, and a Certification Service. Security features include tiered rate limiting, CORS, Helmet.js, AES-256-GCM token encryption, input validation, and hardened path sanitization. API versioning is supported.

### Rust Kernel Architecture
A Rust-based kernel in `src/kernel/` provides core functionalities:
-   **Ternary Operations**: GF(3) arithmetic.
-   **Timing**: Femtosecond-precision timing via High-Precision Timing Protocol (HPTP).
-   **Phase Encryption**: Split/recombine encryption with timing-window enforcement.
-   **Memory Subsystem**: Bitmap-based frame allocator, page table, and heap management.
-   **Synchronization Primitives**: Ticket spinlocks, ternary-security-gated mutexes, semaphores.
-   **Process Management**: States, priority levels, scheduler, CPU context, and message-passing IPC.
-   **Modal Security System**: Domain management, capability-based access control, audit trails, and policy engine.
-   **Cryptographic Primitives**: Ternary hash, sponge, HMAC, KDF, Lamport signatures, AES-256-GCM, SHA-2/SHA-3, TL-KEM, TL-DSA, GF(3) polynomial arithmetic, and CNSA 2.0 compliance.
-   **Device Driver Framework**: Abstractions for various device types.
-   **I/O Subsystem**: Priority-based scheduler, buffer cache, block/character device layers.
-   **Filesystem**: Inode management, directory/file operations, mount system.
-   **Torsion Network**: N-dimensional torus topology, greedy geodesic routing, Ternary Transport Protocol (TTP), Ternary Transfer Protocol (T3P), and Ternary DNS (TDNS).
-   **Ternary Virtual Machine**: A 176-opcode ISA v2.1 with ternary addressing, three-ring privilege levels, quantum-ternary simulation, and a ternary-aware garbage collector.
-   **Binary Compatibility Layer**: For balanced ternary conversion and crypto interoperability.

### Legal & IP Compliance
All source files include standardized copyright headers. Legal documents (terms, privacy, security) are served dynamically. `IP-NOTICE.md` documents patent-pending claims, and `EXPORT-CONTROL.md` provides CNSA 2.0 and Wassenaar classification guidance.

### GitHub Integration
An admin-only GitHub Manager page allows browsing and push actions for the `SigmaWolf-8/Ternary` repository.

### Scientific Integrations
The system incorporates:
-   **Saturnian Magic Square Blueprint**: Foundation for SUFT-derived constants.
-   **Hamiltonian Mechanics Integration**: HPTP Symplectic Jitter Corrector, Hamiltonian VM Constraints, and Symplectic Phase Mixing.
-   **Lagrangian Mechanics Utilities**: Discrete Euler-Lagrange equations for ternary logic.
-   **Noether Symmetries**: Conserved quantities for ternary gauge, reparametrization, and periodicity symmetries.
-   **Tribonacci Variational Methods**: Discrete variational functionals for the Tribonacci sequence.

### Quantum Ternary Modules
Five modules provide classical simulation of quantum ternary (qutrit/qudit) operations, including complex utilities, qutrit basics (states, gates, probabilities), Lagrangian qutrit evolution, qutrit fault tolerance (error operators, stabilizer codes), and generalized qudit basics.

### 28-Dimension Agent Array
This system orchestrates 28 specialist AI agents for parallel query analysis, featuring an Etymology Audit, Veritas Fact-Check, unified Situation Report generation, and Lexical Protocol enforcement.

### Kong Gateway Integration
Kong Konnect API integration manages services, routes, and plugins for PlenumNET's 17 services.

### API Key Management System
A comprehensive system at `/api-keys` handles API key generation, validation (constant-time, scope, optional HPTP timing-bound), rotation (manual/auto), per-key rate limiting (three tiers), and audit trails. It includes anomaly detection for usage spikes, high failure rates, and IP dispersion. WBS Tagging System allows entity classification and project metadata, with robust search and filtering capabilities. A dashboard provides stats, alerts, and management controls.

### Security Middleware Stack
Includes 4-tier rate limiting, CORS, Helmet.js security headers, AES-256-GCM token encryption, null-byte stripping, double URL-decode protection, and `execFile()`-only subprocess execution.

### Security Infrastructure Services
Admin-protected backend services under `/api/security/`:
-   **Security Audit Service**: Event logging with severity and resolution workflow.
-   **HPTP Anomaly Detection**: Severity-score-based detection for timing anomalies with a 5-tier fallback chain.
-   **Threat Model Registry**: CRUD for threats with likelihood/impact, risk score, mitigation status, and controls.
-   **Implementation Status Tracker**: Tracks component status, completion percentage, and testing metrics.
-   **Security Dashboard**: Unified endpoint aggregating security stats.

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