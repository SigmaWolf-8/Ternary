# PlenumNET Framework Marketing Website

## Overview
PlenumNET is a post-quantum internet solutions company focused on building quantum-resistant infrastructure. This project is a professional, light-themed marketing website showcasing PlenumNET's deployable components, including a PlenumDB product page with a live compression demo and comprehensive whitepaper management. The project also encompasses a complete payment processing and blockchain witnessing architecture, integrating high-precision timing and regulatory compliance for secure and verifiable operations in quantum-resistant data and financial services.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend uses React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui` for components, and Wouter for routing. It adheres to a light theme with a white background and blue accents. Key pages include the Landing Page, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub (/docs), and Admin Dashboard (/admin).

**Documentation**: The `salvi_docs/` directory contains comprehensive developer documentation (8 files, ~3,200 lines) covering tutorials and module guides for the Salvi Framework. The `/docs` page on the site provides a browsable index linking to GitHub-hosted markdown files.

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
-   **Cryptographic Primitives**: Ternary hash, sponge construction, HMAC, key derivation, and ternary Lamport one-time signatures.
-   **Device Driver Framework**: Abstractions for device types, bus management, device registry, interrupt controller, and DMA.
-   **I/O Subsystem**: Priority-based I/O scheduler, buffer cache, block device layer, character device layer, and I/O multiplexing.
-   **Filesystem**: Inode management, directory operations, file operations, and a mount system supporting various filesystem types.
-   **Error Handling**: Unified `KernelError` enum for all subsystems.
-   **Architecture Support**: Generic traits and specific implementations for `x86_64`, `aarch64`, and `riscv64`, including boot sequence management.
-   **Hardware Drivers**: Drivers for TPU FPGA/ASIC and femtosecond clock with multiple sources.
-   **Torsion Network**: N-dimensional torus topology, greedy geodesic routing, Ternary Transport Protocol (TTP), Ternary Transfer Protocol (T3P), and Ternary DNS (TDNS).
-   **Ternary Virtual Machine**: 35-opcode ISA, execution engine with ternary ops, and a ternary-aware mark-sweep garbage collector (TAGC).
-   **High-Precision Timing Protocol (HPTP)**: Synchronization protocol, optical clock manager, and regulatory compliance certification.
-   **Binary Compatibility Layer**: Gateway for balanced ternary conversion and a universal ternary adapter for format-transparent data handling.

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