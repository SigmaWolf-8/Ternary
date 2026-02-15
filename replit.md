# PlenumNET Framework Marketing Website

## Overview
PlenumNET is a post-quantum internet solutions company developing quantum-resistant infrastructure. This project delivers a professional marketing website to showcase PlenumNET's offerings, including the PlenumDB product with a live compression demo and comprehensive whitepaper management. It integrates payment processing and blockchain witnessing for secure, verifiable, and regulatory-compliant operations in quantum-resistant data and financial services. The project aims to establish PlenumNET as a leader in next-generation internet solutions, offering unparalleled security and performance.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend uses React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui`, and Wouter for routing. It supports light/dark modes with a blue brand identity and features a dual-layout navigation system (`MarketingLayout` for public pages, `DashboardLayout` for tools/admin). Key features include a Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, and an Admin Dashboard. It includes ancient calendar synchronization across 24 global systems, anchoring the Salvi Epoch for historical and cross-cultural timing conversions. SEO is optimized with robots.txt, sitemap.xml, JSON-LD, and PWA capabilities.

### Backend and Core Framework
The backend is built with Express.js and Node.js, using PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic System operations, Femtosecond Timing, and Phase Encryption. The architecture incorporates microservices for payment processing and blockchain witnessing, a Femtosecond Timing Service, and a Certification Service. Security features include tiered rate limiting, CORS enforcement, Helmet.js security headers, AES-256-GCM token encryption, input validation, hardened path sanitization, and `execFile()`-only subprocess execution. API versioning is supported via `/api/v1/`.

### Rust Kernel Architecture
The `src/kernel/` directory contains a Rust-based kernel providing advanced functionalities:
-   **Ternary Operations**: GF(3) arithmetic and conversions.
-   **Timing**: Femtosecond-precision timing and High-Precision Timing Protocol (HPTP).
-   **Phase Encryption**: Split/recombine encryption with timing-window enforcement.
-   **Memory Subsystem**: Bitmap-based frame allocator, page table management, and heap allocator.
-   **Synchronization Primitives**: Ticket spinlocks, ternary-security-gated mutexes, semaphores.
-   **Process Management**: States, priority levels, scheduler, CPU context, and message-passing IPC.
-   **Modal Security System**: Domain management, capability-based access control, timestamped audit trails, and policy engine.
-   **Cryptographic Primitives**: Ternary hash, sponge, HMAC, KDF, Lamport signatures, AES-256-GCM, SHA-2/SHA-3, TL-KEM, TL-DSA, GF(3) polynomial arithmetic, and CNSA 2.0 compliance.
-   **Device Driver Framework**: Abstractions for various device types and buses.
-   **I/O Subsystem**: Priority-based scheduler, buffer cache, block/character device layers.
-   **Filesystem**: Inode management, directory/file operations, mount system.
-   **Torsion Network**: N-dimensional torus topology, greedy geodesic routing, Ternary Transport Protocol (TTP), Ternary Transfer Protocol (T3P), and Ternary DNS (TDNS).
-   **Ternary Virtual Machine**: A 160-opcode ISA v2.0 supporting various operations with ternary addressing, three-ring privilege levels, and a ternary-aware garbage collector.
-   **Binary Compatibility Layer**: For balanced ternary conversion and crypto interoperability.

### Legal & IP Compliance
All source files include standardized copyright headers from "Capomastro Holdings Ltd. (Canada)" with "Patent(s) Pending." A CI workflow enforces header presence. Legal documents for terms, privacy, and security are served dynamically from markdown content. IP-NOTICE.md documents patent-pending claims and proprietary algorithms. EXPORT-CONTROL.md provides CNSA 2.0 and Wassenaar classification guidance.

### GitHub Integration
An admin-only GitHub Manager page facilitates file browsing for the `SigmaWolf-8/Ternary` repository and enables push actions for CI/CD, crypto modules, and project synchronization. Branch protection is enabled on main.

### Scientific Integrations
The system incorporates advanced mathematical and physics concepts:
-   **Saturnian Magic Square Blueprint**: Provides the 3×3 Saturnian circulant magic square as a static foundation for SUFT-derived constants, bridging to the Tribonacci sequence.
-   **Hamiltonian Mechanics Integration**: Includes HPTP Symplectic Jitter Corrector for femtosecond jitter correction, Hamiltonian VM Constraints for energy invariant enforcement, and Symplectic Phase Mixing for phase encryption.
-   **Lagrangian Mechanics Utilities**: Provides discrete Euler-Lagrange equations adapted for ternary logic, including canonical momenta, mass-shell constraints, and Noether charge.
-   **Noether Symmetries**: Implements conserved quantities from Noether's theorem for ternary gauge, reparametrization, and periodicity symmetries.
-   **Tribonacci Variational Methods**: Constructs discrete variational functionals for the Tribonacci sequence, including penalty Lagrangians and Tribonacci-weighted potentials.

### Quantum Ternary Modules
Five modules provide classical simulation of quantum ternary (qutrit/qudit) operations:
-   **Complex Utilities**: Lightweight self-contained complex arithmetic for quantum calculations.
-   **Qutrit Basics**: Implements qutrit (3-level quantum) states coupled to SUFT branches, including phase gates, shift/clock operators, Gell-Mann generators, probabilities, and unitarity checks.
-   **Lagrangian Qutrit Utilities**: Discrete Euler-Lagrange for qutrit evolution, including quantum EL branch updates and discrete qutrit action.
-   **Qutrit Fault Tolerance**: Classical simulation of fault-tolerant protocols, including error operators, stabilizer codes, syndrome measurement, and magic state distillation.
-   **Qudit Basics**: Generalized higher-dimensional (d ≥ 2) quantum states, supporting various dimensions with basis, operators, error simulation, and code parameters.

### Quantum-Ternary Simulator (/quantum-sim)
A public-facing interactive simulator page with three tabs:
-   **Qutrit FT Mode**: Interactive fault-tolerance simulation — state preparation (4 basis choices), error injection (phase/leak/depolarize), [[3,1,2]]_3 stabilizer encoding + syndrome measurement + correction, triorthogonal magic state distillation (configurable m and noise rate), SUFT phase gate unitarity verification, fidelity tracking.
-   **FIPS 140-3 Path**: 14-item compliance readiness checklist across 5 categories (Cryptographic Boundary, Fault Tolerance, Quantum Resistance, Invariant Enforcement, Export Control) with ASCII CMVP module boundary diagram.
-   **Variational Benchmarks (QVQE/QAOA)**: 6 client-side benchmarks — Tribonacci variational action, perturbed sequence recovery, Hamiltonian constraint chain, stabilizer code throughput, distillation scaling, SUFT phase gate stability — with performance.now() timing.
All computations are client-side using shared modules. A "Qutrit FT Mode" CTA button on the VM demo hero links to this page.

### Testing
Testing is conducted using Vitest with multiple test suites covering complex utilities, qutrit/qudit basics, Lagrangian and Hamiltonian mechanics, Noether symmetries, Tribonacci variational methods, ternary operations, phase encryption, calendar synchronization, API route integration, blockchain services, and payment webhooks. Rust fuzz targets and kernel tests are also performed.

### 28-Dimension Agent Array
The Agent Array system orchestrates 28 specialist AI agents for parallel query analysis. It features an Etymology Audit, Veritas Fact-Check, unified Situation Report generation, and Lexical Protocol enforcement before translation into 28 world languages. Integrity protocols (Etymology Engine, Veritas Audit, Lexical Protocols) ensure factual accuracy. Reports are persisted to a PostgreSQL table.

### Kong Gateway Integration
Kong Konnect API integration manages services, routes, and plugins, supporting synchronization for PlenumNET's 17 services and optional Kong proxy routing.

### Security Middleware Stack
Includes 4-tier rate limiting, CORS with origin allowlist, Helmet.js security headers (CSP, HSTS, X-Frame-Options: deny, X-Content-Type-Options: nosniff), AES-256-GCM token encryption, null-byte stripping, double URL-decode protection, and `execFile()`-only subprocess execution.

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