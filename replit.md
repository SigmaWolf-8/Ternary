# PlenumNET Framework Marketing Website

## Overview
PlenumNET is developing post-quantum internet solutions. This project creates a marketing website to showcase PlenumNET's quantum-resistant infrastructure, including the PlenumDB product with a compression demo and whitepaper management. It integrates payment processing and blockchain witnessing for secure, verifiable, and regulatory-compliant operations in quantum-resistant data and financial services. The project aims to position PlenumNET as a leader in next-generation internet solutions, offering unparalleled security and performance in the quantum-resistant internet domain.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend uses React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui`, and Wouter, supporting light/dark modes. Key pages include a Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, TSA Time-Stamping Authority, Node Terminal, and an Admin Dashboard. It features a quantum-ternary simulator and FIPS 140-3 compliance checks.

### Backend and Core Framework
The backend is built with Express.js and Node.js, using PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic, Femtosecond Timing, and Phase Encryption v3 (post-quantum, duplex-mode TL-Sponge-385-based GF(3) stream cipher). The architecture includes microservices for payment processing and blockchain witnessing, a Femtosecond Timing Service, and a Certification Service. Security features include tiered rate limiting, CORS, Helmet.js, AES-256-GCM token encryption, input validation, hardened path sanitization, and API versioning.

### Inter-Cube Infrastructure Services
A 4-service system provides geometric routing across the 13D ternary cube network: Geometric Load Balancer (GLB), Cube Overlay Network (CON), Cube Registration Service (CRS), and Fault Tolerance Service (FTS). It is implemented as a Rust crate with TypeScript API routes, featuring PT26-DSA native daemon identity and automatic radian-epoch key rotation.

### Rust Kernel Architecture
A Rust-based kernel provides core functionalities: Ternary Operations (GF(3) arithmetic), Femtosecond-precision Timing (HPTP), Phase Encryption, and a 3-Tier Security System. It includes Cryptographic Primitives (ternary hash, TL-KEM, TL-DSA, CNSA 2.0 compliance), a Torsion Network (N-dimensional torus topology, Ternary Transport/Transfer/DNS), and a Ternary Virtual Machine (176-opcode ISA, ternary addressing, three-ring privilege levels, quantum-ternary simulation, ternary-aware garbage collector). A Binary Compatibility Layer handles balanced ternary conversion and crypto interoperability. The kernel boots as a bare-metal binary for x86_64, aarch64, and riscv64, utilizing a 512MB linked-list allocator.

### PlenumBrowser Kernel Subsystem
A browser engine built as kernel subsystem modules, implementing CPU rendering including parsing (DOM/CSS types), layout (iterative Flexbox), scripting (cooperative JS executor), CPU rendering (framebuffer + sponge XOR encryption), tabs (isolation via kernel tasks), input (TIS-27 encoded key dispatch), networking (resource requests to z=0), mesh (540-node recursive polygon mesh), and color (PlenumColor mesh↔sRGB mapping).

### Core Protocols and Features
- **TTC v4.2 Compression Pipeline**: Native Rust engine for file compression using domain analysis, ternary rANS, and GURFT fast-path.
- **TDNS v2.5.0**: A 27-dimensional ontological addressing protocol with 54-trit dual-layer addressing, using TL-Sponge-43 for identity derivation and TIS-27 for wire packet integrity.
- **RFC 3161 Time-Stamping Authority (TSA)**: Digital notary service with Merkle tamper-evident audit logs and dual-signature (RSA-4096 + TL-DSA-87).
- **Hedera HCS Witnessing**: Blockchain-based non-repudiation for immutable, ordered, timestamped proof of PlenumNET operations.
- **Yoda Global Command**: A universal `y` command prefix for operator-to-Yoda communication through a relay, with client-side (CLI, Node Terminal, desktop widget) and server-side components ensuring secure, verified, and auditable interactions.
- **XPlenum RISC-V Hardware Extension**: Custom RISC-V extension for ternary security operations, PQC acceleration, and compliance.
- **TL-KEM**: Ternary-native equivalent of ML-KEM for IND-CCA2 secure key encapsulation.
- **NinjaExec**: Local signing agent (`ssh-agent` equivalent) for PlenumNET, holding operator's TL-DSA-87 private key in an encrypted keystore and exposing a localhost-only HTTP signing API.

### UI/UX and Branding
The brand palette includes dark mode (#0F0C0A bg, #4A9EF5 accent) and light mode (#FAF8F6 bg, #2D7DD2 accent). Status indicators use Active=#4A9EF5, Warning=#78828C, Inactive=#3D444B.

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