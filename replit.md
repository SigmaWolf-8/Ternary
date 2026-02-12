# PlenumNET Framework Marketing Website

## Overview
PlenumNET is a post-quantum internet solutions company focused on building quantum-resistant infrastructure. This project is a professional marketing website showcasing PlenumNET's deployable components, including a PlenumDB product page with a live compression demo and comprehensive whitepaper management. The project also encompasses a complete payment processing and blockchain witnessing architecture, integrating high-precision timing and regulatory compliance for secure and verifiable operations in quantum-resistant data and financial services.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## Legal Jurisdiction
All legal documents use **Province of Alberta** (not Ontario) for governing law, jurisdiction, court references, and entity incorporation ("incorporated under the laws of the Province of Alberta"). Capomastro Holdings Ltd. is an Alberta corporation.

## System Architecture

### Frontend
The frontend uses React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui` for components, and Wouter for routing. It adheres to a light theme with a white background and blue accents. Navigation uses a collapsible shadcn Sidebar. Key pages include the Landing Page, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, and Admin Dashboard.
The `salvi_docs/` directory contains comprehensive developer documentation. The `/docs` page on the site provides a browsable index linking to GitHub-hosted markdown files.
The system supports ancient calendar synchronization, anchoring the Salvi Epoch to 24 global calendar systems spanning over 30,000 years, with conversions computed via JDN and individual API endpoints for each calendar system.

### Backend and Core Framework
The backend is built with Express.js and Node.js, using PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic System operations (conversion, arithmetic, XOR in GF(3)), Femtosecond Timing, and Phase Encryption. Ternary representations include Computational (`{-1, 0, +1}`), Network (`{0, 1, 2}`), and Human (`{1, 2, 3}`).
The architecture includes microservices for payment processing and blockchain witnessing. These services handle payment webhooks, orchestrate CRUD operations, and integrate with blockchain platforms like Hedera HCS, XRPL, and Algorand. A Femtosecond Timing Service provides high-precision timing, and a Certification Service ensures regulatory compliance. Security features include HMAC validation, timing-safe equality checks, idempotency, and rate limiting.
The database schema includes tables for `users`, `sessions`, `demo_sessions`, `binary_storage`, `ternary_storage`, `compression_benchmarks`, `compression_history`, and `whitepapers`.

### Rust Kernel Architecture
The `src/kernel/` directory contains a robust kernel developed in Rust, encompassing:
-   **Ternary Operations**: GF(3) arithmetic and representation conversions.
-   **Timing**: Femtosecond-precision timestamps and the High-Precision Timing Protocol (HPTP).
-   **Phase Encryption**: Split/recombine encryption with timing-window enforcement.
-   **Memory Subsystem**: Bitmap-based frame allocator, page table management with ternary security modes, and free-list heap allocator.
-   **Synchronization Primitives**: Ticket-based spinlocks, ternary-security-gated mutexes, semaphores, and phase-encryption-aware mutexes.
-   **Process Management**: Process states, priority levels, multi-level priority round-robin scheduler, CPU context management, and message-passing IPC.
-   **Modal Security System**: Security domain management, capability-based access control, femtosecond-timestamped audit trails, and a priority-ordered policy engine.
-   **Cryptographic Primitives**: Ternary hash, sponge construction, HMAC, key derivation, ternary Lamport one-time signatures, AES-256-GCM cipher, SHA-2/SHA-3, TL-KEM, TL-DSA, GF(3) polynomial ring arithmetic, and CNSA 2.0 compliance framework (100% coverage).
-   **Device Driver Framework**: Abstractions for device types, bus management, device registry, interrupt controller, and DMA.
-   **I/O Subsystem**: Priority-based I/O scheduler, buffer cache, block device layer, character device layer, and I/O multiplexing.
-   **Filesystem**: Inode management, directory operations, file operations, and a mount system.
-   **Error Handling**: Unified `KernelError` enum.
-   **Architecture Support**: Generic traits and specific implementations for `x86_64`, `aarch64`, and `riscv64`.
-   **Hardware Drivers**: Drivers for TPU FPGA/ASIC and femtosecond clock.
-   **Torsion Network**: N-dimensional torus topology, greedy geodesic routing, Ternary Transport Protocol (TTP), Ternary Transfer Protocol (T3P), and Ternary DNS (TDNS).
-   **Ternary Virtual Machine**: 35-opcode ISA, execution engine with ternary ops, and a ternary-aware mark-sweep garbage collector (TAGC).
-   **Binary Compatibility Layer**: Gateway for balanced ternary conversion, a universal ternary adapter, and CryptoInteropBridge for ML-KEM/ML-DSA interoperability.

### Legal & IP Compliance
All source files (224 total: 121 Rust, 103 TypeScript/JavaScript) carry standardized copyright headers with "Patent(s) Pending" and "Capomastro Holdings Ltd. (Canada)" designation. A `license-check.yml` CI workflow enforces header presence on all commits. Legal pages at `/terms`, `/privacy`, `/security` are served via `/api/legal/:type` with content from local markdown files. Key legal documents on GitHub: LICENSE, CLA.md, CONTRIBUTING.md, TERMS-OF-SERVICE.md, ACCEPTABLE-USE-POLICY.md, INTELLECTUAL-PROPERTY-NOTICE.md, TRADEMARK-NOTICE.md, SECURITY.md, FILE-HEADER-TEMPLATE.md, and CODE-OF-CONDUCT.md.

### GitHub Integration
The GitHub Manager page (`/github`, admin-only) provides a file browser for the `SigmaWolf-8/Ternary` repository and enables push actions for CI/CD workflows, crypto modules, and a full project sync.

## External Dependencies

-   **Authentication**: Replit Auth (GitHub, Google, Apple, X, email/password).
-   **Database**: PostgreSQL.
-   **ORM**: Drizzle ORM.
-   **API Gateway**: Kong Konnect (18 services, 70+ endpoints, full CNSA 2.0 enforcement).
-   **Payment Gateways**: Stripe, Interac, various cryptocurrency platforms.
-   **Message Queue**: BullMQ.
-   **Blockchain Platforms**: Hedera Hashgraph Consensus Service (HCS), XRP Ledger (XRPL), Algorand.
-   **Containerization**: Docker.
-   **Cloud Deployment**: Render, Railway.