# PlenumNET Framework Marketing Website

## Overview
PlenumNET is a post-quantum internet solutions company developing quantum-resistant infrastructure. This project delivers a professional marketing website to showcase PlenumNET's offerings, including the PlenumDB product with a live compression demo and comprehensive whitepaper management. It also integrates a complete payment processing and blockchain witnessing architecture, ensuring secure, verifiable, and regulatory-compliant operations for quantum-resistant data and financial services. The project's vision is to establish PlenumNET as a leader in next-generation internet solutions, offering unparalleled security and performance.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend is built with React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui`, and Wouter for routing, supporting both light and dark modes with a unified blue brand identity. It features a dual-layout navigation system (`MarketingLayout` for public pages and `DashboardLayout` for tools/admin) with dynamic page titles and anchor navigation. Key pages include the Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, and an Admin Dashboard. The system supports ancient calendar synchronization across 24 global systems, anchoring the Salvi Epoch to provide historical and cross-cultural timing conversions. SEO is optimized with robots.txt, sitemap.xml, JSON-LD, and PWA capabilities.

### Backend and Core Framework
The backend utilizes Express.js and Node.js with PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic System operations, Femtosecond Timing, and Phase Encryption. The backend routes are modularized for better organization and maintainability. Security is a priority, incorporating rate limiting, CORS restrictions, Helmet.js security headers, AES-256-GCM token encryption, input validation, and hardened path sanitization. The architecture includes microservices for payment processing and blockchain witnessing, a Femtosecond Timing Service, and a Certification Service for regulatory compliance.

### Rust Kernel Architecture
The `src/kernel/` directory houses a Rust-based kernel providing advanced functionalities:
-   **Ternary Operations**: GF(3) arithmetic and conversions.
-   **Timing**: Femtosecond-precision timing and High-Precision Timing Protocol (HPTP).
-   **Phase Encryption**: Split/recombine encryption with timing-window enforcement.
-   **Memory Subsystem**: Bitmap-based frame allocator, page table management with ternary security, and heap allocator.
-   **Synchronization Primitives**: Ticket spinlocks, ternary-security-gated mutexes, semaphores, and phase-encryption-aware mutexes.
-   **Process Management**: States, priority levels, scheduler, CPU context, and message-passing IPC.
-   **Modal Security System**: Domain management, capability-based access control, timestamped audit trails, and policy engine.
-   **Cryptographic Primitives**: Ternary hash, sponge, HMAC, KDF, Lamport signatures, AES-256-GCM, SHA-2/SHA-3, TL-KEM, TL-DSA, GF(3) polynomial arithmetic, and CNSA 2.0 compliance.
-   **Device Driver Framework**: Abstractions for various device types and buses.
-   **I/O Subsystem**: Priority-based scheduler, buffer cache, block/character device layers.
-   **Filesystem**: Inode management, directory/file operations, mount system.
-   **Torsion Network**: N-dimensional torus topology, greedy geodesic routing, Ternary Transport Protocol (TTP), Ternary Transfer Protocol (T3P), and Ternary DNS (TDNS).
-   **Ternary Virtual Machine**: A 55-opcode ISA supporting core, extended, crypto acceleration, and SIMD operations with ternary addressing, privilege levels, and a ternary-aware garbage collector.
-   **Binary Compatibility Layer**: For balanced ternary conversion and crypto interoperability.

### Legal & IP Compliance
All source files include standardized copyright headers from "Capomastro Holdings Ltd. (Canada)" with "Patent(s) Pending." A CI workflow enforces header presence. Legal documents for terms, privacy, and security are served dynamically from markdown content.

### GitHub Integration
An admin-only GitHub Manager page facilitates file browsing for the `SigmaWolf-8/Ternary` repository and enables push actions for CI/CD, crypto modules, and project synchronization.

### Testing
Testing is conducted using Vitest, with dedicated test files for ternary operations, phase encryption, and calendar synchronization. CI workflows trigger tests on push/PR.

### Kong Gateway Integration
Kong Konnect API integration manages services, routes, and plugins. It supports service synchronization for PlenumNET's 17 services (97 endpoints) and allows optional Kong proxy routing in the frontend.

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