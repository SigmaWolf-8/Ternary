# PlenumNET Framework Marketing Website

## Overview
PlenumNET is developing post-quantum internet solutions. This project creates a marketing website to showcase PlenumNET's quantum-resistant infrastructure, including the PlenumDB product with a compression demo and whitepaper management. It integrates payment processing and blockchain witnessing for secure, verifiable, and regulatory-compliant operations in quantum-resistant data and financial services. The project aims to position PlenumNET as a leader in next-generation internet solutions, offering unparalleled security and performance in the quantum-resistant internet domain.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend uses React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui`, and Wouter, supporting light/dark modes. It includes a Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, TSA Time-Stamping Authority, Node Terminal, and an Admin Dashboard. It features a quantum-ternary simulator and FIPS 140-3 compliance checks.

### Node Terminal + Array3 Cluster Shell + Ops Console
A browser-accessible PTY terminal at `/terminal` provides interactive shell access via WebSocket. It features multiple concurrent sessions and a Cluster Shell mode for fanning out commands. The terminal doubles as a full production Ops Console for remote script execution (with approval gate + NinjaExec TL-DSA signing), live log tailing, system telemetry dashboards, and operations timeline.

### Daemon Remote Operations Channel
A WebSocket relay provides remote PowerShell script execution, live log tailing, system telemetry heartbeats, small and chunked GGUF model transfer, GGUF model hot-swap, and multi-operator RBAC. All authenticated operations require TL-DSA signatures.

### Backend and Core Framework
The backend is built with Express.js and Node.js, using PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic, Femtosecond Timing, and Phase Encryption v3 (post-quantum, duplex-mode TL-Sponge-385-based GF(3) stream cipher). The architecture includes microservices for payment processing and blockchain witnessing, a Femtosecond Timing Service, and a Certification Service. Security features include tiered rate limiting, CORS, Helmet.js, AES-256-GCM token encryption, input validation, and API versioning.

### Inter-Cube Infrastructure Services
A 4-service system (Geometric Load Balancer, Cube Overlay Network, Cube Registration Service, Fault Tolerance Service) provides geometric routing across a 13D ternary cube network. It features PT26-DSA native daemon identity with persistent encrypted MasterSecret and automatic radian-epoch key rotation.

### Rust Kernel Architecture
A Rust-based kernel provides core functionalities: Ternary Operations (GF(3) arithmetic), Femtosecond-precision Timing (HPTP), Phase Encryption, and a 3-Tier Security System. It includes Cryptographic Primitives (ternary hash, TL-KEM, TL-DSA, CNSA 2.0 compliance), a Torsion Network (N-dimensional torus topology, Ternary Transport/Transfer/DNS), and a Ternary Virtual Machine (176-opcode ISA, quantum-ternary simulation, ternary-aware garbage collector). A Binary Compatibility Layer handles balanced ternary conversion and crypto interoperability. The kernel boots bare-metal for x86_64, aarch64, and riscv64.

### PlenumBrowser Kernel Subsystem
A browser engine built as kernel subsystem modules, implementing CPU rendering, parsing, layout, scripting, CPU rendering, tabs, input, networking, mesh, and color mapping.

### z=0 Distributor
This component implements the z-axis dome geometry, acting as an equatorial distributor plane using a (7, 11, 13) coprime walk over 540 nodes.

### (11, 13) Coprime Polygon Pair & Extended Geometric Framework
Specific geometric constants and relationships derived from hendecagons and tridecagons, and an extended geometric framework defining repunits, squared circle parameters, polygon central angles, node census, superhub zones, and torus knots, are used for platform calculations.

### PUV UV Spectral Protocol
The PlenumNET UV Spectral Protocol (PUV v1.0) defines UV band definitions based on the axiom π=14, with primary and secondary bands partitioning the UV spectrum, including specific constants for spectral analysis.

### TIS-27 Keyboard Input & XPlenum RISC-V Hardware Extension
Kernel-space TIS-27 encoding is used for keyboard input. A custom RISC-V extension with 21 custom instructions and 12 custom CSRs is integrated with CVA6 for ternary security operations, PQC acceleration, and compliance.

### TL-KEM & Crypto Benchmark Suite
TL-KEM provides IND-CCA2 secure key encapsulation at three security levels. A Criterion-based statistical benchmark suite covers all core cryptographic primitives.

### Sponge Architecture
TL-Sponge-385 provides 385-bit post-quantum security for signing, key derivation, FIPS validation, and document hashing. TL-Sponge-43 is used for TDNS identity derivation.

### TTC v4.2 Compression Pipeline
File compression uses the TTC v4.2 native Rust engine via N-API, including domain analysis, ternary rANS, and GURFT fast-path.

### TDNS v2.5.0 — Ternary Domain Name System
A standalone Rust crate implementing a 27-dimensional ontological addressing protocol with 54-trit dual-layer addressing, using TL-Sponge-43 for identity derivation and TIS-27 for wire packet integrity.

### Tonal Diffusion System & RFC 3161 Time-Stamping Authority
A system for network-wide time synchronization using FM timing packets and a digital notary service providing cryptographic proof-of-existence timestamps per RFC 3161 with Merkle tamper-evident audit logs and dual-signature.

### Hedera HCS Witnessing
Blockchain-based non-repudiation via Hedera Consensus Service for immutable, ordered, timestamped proof of PlenumNET operations.

### API Key Management System & Security Middleware Stack
A comprehensive system handles API key generation, validation, rotation, per-key rate limiting, and audit trails. The security middleware includes 4-tier rate limiting, CORS, Helmet.js, AES-256-GCM token encryption, and hardened path sanitization.

### Capability-Based Security
Authorization uses unforgeable, self-contained, bearer-verified capability tokens signed with TL-DSA.

### Array3 Watchdog & PlenumNET App Installer Framework
A Windows scheduled task monitors daemon services and processes. A manifest-driven MSI build system packages PlenumNET Windows applications.

### NinjaExec — PlenumNET Local Signing Agent
A standalone Rust binary that acts as PlenumNET's `ssh-agent`, holding the operator's TL-DSA-87 private key in an encrypted keystore and exposing a localhost-only HTTP signing API.

### Yoda Global Command
A universal `y` command prefix for operator-to-Yoda communication through a relay, facilitating chat and responses with robust security features, session management, and diagnostics.

### Brand Palette
Dark mode primary (#0F0C0A bg, #4A9EF5 accent), light mode (#FAF8F6 bg, #2D7DD2 accent). Status: Active=#4A9EF5, Warning=#78828C, Inactive=#3D444B.

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