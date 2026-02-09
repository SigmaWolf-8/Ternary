# PlenumNET Framework Marketing Website

## Overview
PlenumNET is a post-quantum internet solutions company focused on building quantum-resistant infrastructure. This project is a professional, light-themed marketing website showcasing PlenumNET's deployable components, including a PlenumDB product page with a live compression demo and comprehensive whitepaper management. The project also encompasses a complete payment processing and blockchain witnessing architecture, integrating high-precision timing and regulatory compliance for secure and verifiable operations in quantum-resistant data and financial services.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend uses React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui` for components, and Wouter for routing. It adheres to a light theme with a white background and blue accents. Navigation uses a collapsible shadcn Sidebar (`client/src/components/app-sidebar.tsx`) wrapped in `SidebarProvider` in `App.tsx`, replacing individual page headers. The sidebar structure is: Platform (collapsible: Architecture, Components, Performance, Calendar sub-items), Calendar API, API Demo, Whitepaper, Docs, CNSA 2.0 (compliance page), App Links (collapsible: GitHub, Kong Konnect), and Admin section (visible to admins). Auth (login/logout) is in the sidebar footer. Key pages include the Landing Page, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub (/docs), CNSA 2.0 Compliance (/compliance), and Admin Dashboard (/admin).

**Documentation**: The `salvi_docs/` directory contains comprehensive developer documentation (15 files, ~7,316+ lines) covering tutorials and module guides for the Salvi Framework. The `/docs` page on the site provides a browsable index linking to GitHub-hosted markdown files.

**Ancient Calendar Synchronization**: The Salvi Epoch (April 1, 2025 00:00:00 UTC) is anchored to 9 ancient calendar systems spanning 30,000+ years: Mayan Long Count, Hebrew (Anno Mundi), Chinese Sexagenary (Yellow Emperor epoch, Cycle 78), Vedic Kali Yuga, Egyptian Civil, Julian Day Number, Islamic Hijri, Byzantine Anno Mundi, and 13-Moon Natural Time (13 months x 28 days = 364-day cycle, prehistoric attestation ~28,000 BCE). All conversions computed via JDN with verified backward time compatibility.

### Backend and Core Framework
The backend is built with Express.js and Node.js, using PostgreSQL and Drizzle ORM.

**PlenumNET Core API**: Implements Unified Ternary Logic System operations (conversion, arithmetic, XOR in GF(3)), Femtosecond Timing, and Phase Encryption.

**Ternary Representations**:
-   **A (Computational)**: `{-1, 0, +1}`
-   **B (Network)**: `{0, 1, 2}`
-   **C (Human)**: `{1, 2, 3}`

**Microservices Architecture (Payment & Witnessing)**:
-   **Payment Listener**: Handles payment webhooks (Stripe, Interac, Crypto) with HMAC validation and queues payments using BullMQ.
-   **SFK Core API**: An orchestration layer for CRUD operations and blockchain integration using Fastify, with Swagger/OpenAPI documentation.
-   **Blockchain Services**: Dedicated services for Hedera HCS (witnessing), XRPL (payment settlement), and Algorand (smart contract execution and oracle bridge).
-   **Femtosecond Timing Service**: Provides high-precision timing synchronized via HPTP.
-   **Certification Service**: Ensures regulatory compliance (FINRA 613, MiFID II) for timestamps.

**Security Features**: HMAC-SHA256/SHA512 validation, `crypto.timingSafeEqual`, idempotency keys, and rate limiting.

### Database Schema
Includes tables for `users`, `sessions`, `demo_sessions`, `binary_storage`, `ternary_storage`, `compression_benchmarks`, `compression_history`, and `whitepapers`.

### Rust Kernel Architecture
The `src/kernel/` directory contains a robust kernel developed in Rust, encompassing:
-   **Ternary Operations**: GF(3) arithmetic and representation conversions.
-   **Timing**: Femtosecond-precision timestamps.
-   **Phase Encryption**: Split/recombine encryption with timing-window enforcement.
-   **Memory Subsystem**: Bitmap-based frame allocator, page table management with ternary security modes, and free-list heap allocator.
-   **Synchronization Primitives**: Ticket-based spinlocks, ternary-security-gated mutexes, semaphores, and phase-encryption-aware mutexes.
-   **Process Management**: Process states, priority levels, multi-level priority round-robin scheduler, CPU context management, and message-passing IPC.
-   **Modal Security System**: Security domain management, capability-based access control, femtosecond-timestamped audit trails, and a priority-ordered policy engine.
-   **Cryptographic Primitives**: Ternary hash, sponge construction, HMAC, key derivation, ternary Lamport one-time signatures, AES-256-GCM cipher, SHA-2/SHA-3, TL-KEM (IND-CCA2 key encapsulation at 3 security levels), TL-DSA (EUF-CMA digital signatures at 3 security levels), GF(3) polynomial ring arithmetic, and CNSA 2.0 compliance framework (11/11 algorithms at 100% coverage).
-   **Device Driver Framework**: Abstractions for device types, bus management, device registry, interrupt controller, and DMA.
-   **I/O Subsystem**: Priority-based I/O scheduler, buffer cache, block device layer, character device layer, and I/O multiplexing.
-   **Filesystem**: Inode management, directory operations, file operations, and a mount system supporting various filesystem types.
-   **Error Handling**: Unified `KernelError` enum for all subsystems.
-   **Architecture Support**: Generic traits and specific implementations for `x86_64`, `aarch64`, and `riscv64`, including boot sequence management.
-   **Hardware Drivers**: Drivers for TPU FPGA/ASIC and femtosecond clock with multiple sources.
-   **Torsion Network**: N-dimensional torus topology, greedy geodesic routing, Ternary Transport Protocol (TTP), Ternary Transfer Protocol (T3P), and Ternary DNS (TDNS).
-   **Ternary Virtual Machine**: 35-opcode ISA, execution engine with ternary ops, and a ternary-aware mark-sweep garbage collector (TAGC).
-   **High-Precision Timing Protocol (HPTP)**: Synchronization protocol, optical clock manager, and regulatory compliance certification.
-   **Binary Compatibility Layer**: Gateway for balanced ternary conversion, a universal ternary adapter for format-transparent data handling, and CryptoInteropBridge for ML-KEM/ML-DSA binary format interoperability.

### GitHub Integration
The GitHub Manager page (`/github`, admin-only) provides:
-   **File Browser**: Browse, edit, create, and delete files in the `SigmaWolf-8/Ternary` repository.
-   **P0 Actions**: Three push actions available:
    -   **Push CI/CD Workflows** — Pushes all `.github/workflows/*.yml` files.
    -   **Push Kernel Crypto (Phase 2)** — Pushes Stage 1-3 crypto modules (10 files).
    -   **Push All Stages (1-5)** — Pushes complete Stages 1-5 sync (36 files): 18 crypto modules, 4 compat modules, scheduler fix, libternary package, docs, FIPS plan, governance docs, key management docs, and status doc.
-   **API Endpoints**:
    -   `POST /api/github/push-workflows/:owner/:repo` — Workflow file push.
    -   `POST /api/github/push-batch/:owner/:repo` — Allowlisted batch push with path traversal protection.
-   **Sync Status**: Stages 1-5 complete locally, pending push to main branch via "Push All Stages (1-5)" button. `libternary.tar.gz` requires separate recompilation before push.

## Recent Changes (February 2026)
-   **SP 800-208 Gap CLOSED**: Full XMSS (WOTS+ with Merkle tree, heights 10/16/20, w=16, L-tree compression) and LMS (LM-OTS with Merkle tree, heights 5/10/15/20/25, W=1/2/4/8) in `signature.rs`. Stateful signing with monotonic index advancement and `StateExhausted` enforcement.
-   **Firmware Signing**: `firmware_sign.rs` — Sign → boot verify → reject pipeline with manifest validation for secure boot.
-   **X.509 PKI**: `x509.rs` — Minimal X.509v3 certificates with DER/PEM encoding, chain validation, ML-DSA-87 signature support.
-   **Algorithm Agility**: `agility.rs` — CnsaOnly/Hybrid/Legacy policy engine with algorithm classification and enforcement.
-   **CNSA Protocol Profiles**: `network/cnsa_profiles.rs` — TLS 1.3, SSH, IPsec/IKEv2, S/MIME profiles with cipher suite validation and forbidden algorithm rejection.
-   **Hybrid Key Exchange**: `phase_cnsa.rs` — ML-KEM-1024 + phase encryption temporal binding for session key derivation with forward secrecy.
-   **NTT Extension**: `ternary_lattice.rs` — NTT-like transform via modulus lifting (q=7681) for O(n log n) polynomial multiplication. Forward/inverse/pointwise operations.
-   **Documentation**: Full `05_CRYPTOGRAPHY.md` rewrite (11 algorithms, API reference), `15_FIPS_BOUNDARY.md` (FIPS 140-3 module boundary spec), `16_MIGRATION_GUIDE.md` (Lamport → XMSS/LMS migration).
-   **Compliance Dashboard**: Updated compliance.tsx with full XMSS/LMS descriptions and timeline.
-   **Kong CNSA Plugin**: CNSA 2.0 enforcement headers (request-transformer, response-transformer) in kong.yaml.
-   **Phase 3 CMVP Preparation Complete**: Production hardening, CAVP submission package, FPGA HDL generator, hardware testing framework, formal verification, and compliance documentation.
-   **Production Hardening**: `ct_utils.rs` — Constant-time primitives (ct_select_u8, ct_eq_u8, ct_eq_slices, ct_select_vec, ct_zeroize). AES S-box replaced with GF(2^8) Fermat inversion (a^254, no lookup tables). TL-KEM decaps hardened with ct_select_vec for FO transform.
-   **Phase 3 Crypto Complete**: TL-KEM (FIPS 203 equivalent) and TL-DSA (FIPS 204 equivalent) implemented with 3 security levels each. CNSA 2.0 coverage at 100% (11/11 algorithms).

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