# PlenumNET Framework Marketing Website

## Overview
PlenumNET is developing post-quantum internet solutions. This project creates a marketing website to showcase PlenumNET's quantum-resistant infrastructure, including the PlenumDB product with a compression demo and whitepaper management. It integrates payment processing and blockchain witnessing for secure, verifiable, and regulatory-compliant operations in quantum-resistant data and financial services. The project aims to position PlenumNET as a leader in next-generation internet solutions, offering unparalleled security and performance in the quantum-resistant internet domain.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend uses React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui`, and Wouter, supporting light/dark modes. Key pages include a Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, TSA Time-Stamping Authority, Node Terminal, and an Admin Dashboard. It features a quantum-ternary simulator and FIPS 140-3 compliance checks.

### Node Terminal + Array3 Cluster Shell + Ops Console
A browser-accessible PTY terminal at `/terminal` provides interactive shell access via WebSocket, built with xterm.js (frontend) and node-pty (backend). It supports multiple concurrent sessions and a Cluster Shell mode for fanning out commands to Array3 peers. The terminal also functions as a full production Ops Console with tabbed views for remote script execution, live log tailing, system telemetry dashboards, and an operations timeline.

### Daemon Remote Operations Channel
The WebSocket relay serves as a full production ops channel enabling remote PowerShell script execution (with approval gate), live log tailing, system telemetry heartbeats, small and chunked GGUF model transfers with UI, GGUF model hot-swapping, and multi-operator RBAC. All authenticated operations require TL-DSA signatures from registered operators, with audit logs written to `ops-audit.jsonl`.

### Backend and Core Framework
The backend is built with Express.js and Node.js, using PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic, Femtosecond Timing, and Phase Encryption v3 (post-quantum, duplex-mode TL-Sponge-385-based GF(3) stream cipher). The architecture includes microservices for payment processing and blockchain witnessing, a Femtosecond Timing Service, and a Certification Service. Security features include tiered rate limiting, CORS, Helmet.js, AES-256-GCM token encryption, input validation, hardened path sanitization, and API versioning.

### Inter-Cube Infrastructure Services
A 4-service system provides geometric routing across the 13D ternary cube network: Geometric Load Balancer (GLB), Cube Overlay Network (CON), Cube Registration Service (CRS), and Fault Tolerance Service (FTS). It is implemented as a Rust crate with TypeScript API routes, featuring PT26-DSA native daemon identity and automatic radian-epoch key rotation.

### Rust Kernel Architecture
A Rust-based kernel provides core functionalities: Ternary Operations (GF(3) arithmetic), Femtosecond-precision Timing (HPTP), Phase Encryption, and a 3-Tier Security System. It includes Cryptographic Primitives (ternary hash, TL-KEM, TL-DSA, CNSA 2.0 compliance), a Torsion Network (N-dimensional torus topology, Ternary Transport/Transfer/DNS), and a Ternary Virtual Machine (176-opcode ISA, ternary addressing, three-ring privilege levels, quantum-ternary simulation, ternary-aware garbage collector). A Binary Compatibility Layer handles balanced ternary conversion and crypto interoperability. The kernel boots as a bare-metal binary for x86_64, aarch64, and riscv64 architectures.

### PlenumBrowser Kernel Subsystem (CPU Path)
A browser engine built as kernel subsystem modules, implementing CPU rendering, including parsing (DOM/CSS types), layout (iterative Flexbox), scripting (cooperative JS executor), CPU rendering (framebuffer + sponge XOR encryption), tabs (isolation via kernel tasks), input (TIS-27 encoded key dispatch), networking, mesh, and color mapping.

### z=0 Distributor
This component implements the z-axis dome geometry, acting as an equatorial distributor plane using a (7, 11, 13) coprime walk over 540 nodes for full coverage.

### Geometric Framework
The system incorporates an extended geometric framework including repunits, a squared circle, polygon central angles, node census, superhub zones, torus knots, and a Brieskorn sphere for spatial and network topology definitions.

### PUV UV Spectral Protocol
The PlenumNET UV Spectral Protocol (PUV v1.0) defines UV band definitions based on the axiom π=14, with primary and secondary bands, band boundaries, exact ratios, and biases.

### XPlenum RISC-V Hardware Extension
A custom RISC-V extension integrated with CVA6 provides 21 custom instructions and 12 custom CSRs for ternary security operations, PQC acceleration, and compliance.

### TL-KEM — Ternary Lattice Key Encapsulation
TL-KEM is a ternary-native equivalent of ML-KEM, providing IND-CCA2 secure key encapsulation at three security levels based on Module-LWE over R_q.

### Sponge Architecture
TL-Sponge-385 provides 385-bit post-quantum security for signing, key derivation, FIPS validation, and document hashing, with implementations in TypeScript, Rust kernel, and Rust ternary-math. TL-Sponge-43 is used for TDNS identity derivation and TIS-27 for fast integrity checks.

### TTC v4.2 Compression Pipeline
File compression uses the TTC v4.2 native Rust engine via N-API, including domain analysis, ternary rANS, and GURFT fast-path, with frontend display of TTC metadata badges and round-trip verification using CRC32.

### TDNS v2.5.0 — Ternary Domain Name System
A standalone Rust crate implementing a 27-dimensional ontological addressing protocol with 54-trit dual-layer addressing, using TL-Sponge-43 for identity derivation and TIS-27 for wire packet integrity.

### Tonal Diffusion System
This system enables network-wide time synchronization using FM timing packets, a toroidal topology, and gradient-driven diffusion consensus.

### RFC 3161 Time-Stamping Authority (TSA)
A digital notary service providing cryptographic proof-of-existence timestamps per RFC 3161, featuring Merkle tamper-evident audit logs and dual-signature (RSA-4096 + TL-DSA-87).

### Hedera HCS Witnessing
Blockchain-based non-repudiation via Hedera Consensus Service for immutable, ordered, timestamped proof of PlenumNET operations.

### API Key Management System
A comprehensive system handles API key generation, validation, rotation, per-key rate limiting, and audit trails.

### Security Middleware Stack
Includes 4-tier rate limiting, CORS, Helmet.js security headers, AES-256-GCM token encryption, null-byte stripping, double URL-decode protection, and `execFile()`-only subprocess execution.

### Capability-Based Security
Authorization uses unforgeable, self-contained, bearer-verified capability tokens signed with TL-DSA.

### Array3 Watchdog
A Windows scheduled task monitors daemon services, LLM engines, and orphan processes, providing health checks, smart restarts, and log rotation.

### PlenumNET App Installer Framework
A manifest-driven MSI build system for packaging all PlenumNET Windows applications with consistent branding, system integration, and clean uninstall.

### NinjaExec — PlenumNET Local Signing Agent
A standalone Rust binary that holds the operator's TL-DSA-87 private key in an encrypted keystore and exposes a localhost-only HTTP signing API. It includes a signing engine, encrypted keystore, HTTP API server, confirmation system, audit log, and CLI interface.

### Yoda Global Command
A universal `y` command prefix for operator-to-Yoda communication through a relay, implementing message types with signing context, verification, replay protection, rate limiting, and confidentiality-preserving audit trails. It supports a standalone CLI, Node Terminal integration, and a desktop chat widget.

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