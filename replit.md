# PlenumNET Framework Marketing Website

## Overview
PlenumNET is developing post-quantum internet solutions. This project creates a marketing website to showcase PlenumNET's quantum-resistant infrastructure, including the PlenumDB product with a compression demo and whitepaper management. It integrates payment processing and blockchain witnessing for secure, verifiable, and regulatory-compliant operations in quantum-resistant data and financial services. The project aims to position PlenumNET as a leader in next-generation internet solutions, offering unparalleled security and performance in the quantum-resistant internet domain.

## User Preferences
I prefer iterative development with a focus on delivering working features incrementally. Please ask before making any major architectural changes or decisions that might impact the overall direction of the project. I prefer clear and concise explanations, avoiding overly technical jargon where simpler terms suffice. Do not make changes to the `deployments/` folder.

## System Architecture

### Frontend
The frontend uses React, TypeScript, Tailwind CSS, Framer Motion, `shadcn/ui`, and Wouter, supporting light/dark modes. Key pages include a Landing Page, About, Contact, HPTP Timing API Demo, PlenumDB Product Page, Whitepaper Viewer, GitHub Manager, Kong Konnect Integration, Documentation Hub, CNSA 2.0 Compliance, TSA Time-Stamping Authority, Node Terminal, and an Admin Dashboard. It features a quantum-ternary simulator and FIPS 140-3 compliance checks. The homepage performance section displays real benchmark data from `salvi-bench`. All API service definitions are managed in `shared/service-catalog.ts` as the single source of truth.

### Node Terminal + Array3 Cluster Shell
A browser-accessible PTY terminal at `/terminal` providing interactive shell access via WebSocket (`/ws/terminal`). Built with xterm.js (frontend) and node-pty (backend). Features include multiple concurrent sessions, terminal resize handling, and a Cluster Shell mode that fans out commands to all connected Array3 peers via the Inter-Cube relay. Session management is in `server/terminal.ts`. Rust source code for the syscall shim (`src/kernel/src/compat/syscall_shim.rs`), gateway router extension (`gateway.rs`), and PTY mux crate skeleton (`services/pty-mux/`) are committed as source-only (compiled externally).

### Backend and Core Framework
The backend is built with Express.js and Node.js, using PostgreSQL and Drizzle ORM. It implements Unified Ternary Logic, Femtosecond Timing, and Phase Encryption v3 (post-quantum, duplex-mode TL-Sponge-385-based GF(3) stream cipher). The architecture includes microservices for payment processing and blockchain witnessing, a Femtosecond Timing Service, and a Certification Service. Security features include tiered rate limiting, CORS, Helmet.js, AES-256-GCM token encryption, input validation, hardened path sanitization, and API versioning.

### Inter-Cube Infrastructure Services
A 4-service system provides geometric routing across the 13D ternary cube network: Geometric Load Balancer (GLB), Cube Overlay Network (CON), Cube Registration Service (CRS, 13 HTTP endpoints), and Fault Tolerance Service (FTS). Implemented as a Rust crate (18,649 LOC, 26 modules, 422 tests) with TypeScript API routes. Features PT26-DSA native daemon identity with persistent encrypted MasterSecret (`~/.plenumnet/identity/master.key`), OS CSPRNG generation (`getrandom`), address-bound TL-DSA-87 key derivation, and automatic radian-epoch key rotation (14-day intervals). Daemon modes: `CUBE_MODE=crs` (CRS node), `CUBE_MODE=cube` (cube node with heartbeat rotation), `CUBE_MODE=keygen` (generate and print identity). WebSocket relay auth uses challenge-response: server issues a random nonce, client signs `nonce||address||publicKey` with TL-DSA-87 (address-bound key), and the CRS daemon verifies via `/crs/verify-challenge`. Addresses are displayed in dot-separated format (e.g., `111.111.111.111.1`).

### Rust Kernel Architecture
A Rust-based kernel provides core functionalities: Ternary Operations (GF(3) arithmetic), Femtosecond-precision Timing (HPTP), Phase Encryption, and a 3-Tier Security System. It includes Cryptographic Primitives (ternary hash, TL-KEM, TL-DSA, CNSA 2.0 compliance), a Torsion Network (N-dimensional torus topology, Ternary Transport/Transfer/DNS), and a Ternary Virtual Machine (176-opcode ISA, ternary addressing, three-ring privilege levels, quantum-ternary simulation, ternary-aware garbage collector). A Binary Compatibility Layer handles balanced ternary conversion and crypto interoperability.

### Kernel Boot Infrastructure
The kernel boots as a bare-metal binary (`src/kernel/src/main.rs`) for three architectures: x86_64 (multiboot2 + 32→64 trampoline), aarch64 (QEMU virt, PL011 UART), and riscv64 (OpenSBI, SBI console). Uses a 512MB linked-list allocator (`src/kernel/src/allocator.rs`) with proper deallocation support for heap. Per-arch serial drivers walk 11 `BootSequence` stages from the `arch/boot.rs` infrastructure. After boot, initializes PlenumBrowser at full resolution (1920×1080 on x86_64/aarch64, 1280×720 on riscv64), exercises the z=0 distributor, prints allocator stats, and enters a main event loop (no halt). Linker scripts per-arch in `src/kernel/linker-{x86_64,aarch64,riscv64}.ld`. Custom Rust target specs in `src/kernel/targets/{x86_64,aarch64,riscv64gc}-plenum-none.json` with `-Z build-std` configured via `.cargo/config.toml` aliases. Nightly toolchain pinned in `src/kernel/rust-toolchain.toml`.

### Plenum-Std Shim (`src/plenum-std/`)
A standalone `#![no_std]` crate providing the full Rust `std` API surface mapped to kernel primitives. Crates that require `std` compile against this shim transparently. Modules: `collections` (hashbrown HashMap/HashSet, alloc BTree*), `sync` (Mutex, RwLock, Once, OnceLock, Condvar, Barrier, mpsc, Arc/Weak, atomics), `thread` (spawn, JoinHandle, LocalKey/TLS, sleep, yield), `time` (Instant, SystemTime, Duration), `io` (Read, Write, Seek, BufRead, Cursor, BufReader, BufWriter, Error/ErrorKind), `net` (stub — Err(Unsupported)), `fs` (stub — Err(Unsupported)), `env` (stub defaults), `process` (stub), `panic` (catch_unwind passthrough), plus re-exports of core/alloc types (fmt, string, vec, boxed, rc, etc.).

### PlenumBrowser Kernel Subsystem (Phase 1 — CPU Path)
A browser engine built as kernel subsystem modules in `src/kernel/src/browser/`. Not a fork — parsing, layout, scripting, rendering, and networking are kernel-space with direct access to the GPU, ternary crypto stack, and z=0 distributor. Phase 1 implements CPU rendering via `render_cpu.rs` (tiny-skia fallback path). Modules: `parse.rs` (DOM/CSS types), `layout.rs` (iterative Flexbox layout), `script.rs` (cooperative JS executor with watchdog), `render_cpu.rs` (framebuffer + sponge XOR encryption), `tabs.rs` (tab isolation via kernel tasks, max 64), `input.rs` (TIS-27 encoded key dispatch), `net.rs` (resource requests to z=0), `mesh.rs` (540-node recursive polygon mesh with Bézier interpolation), `color.rs` (PlenumColor mesh↔sRGB mapping). 71 unit tests.

### z=0 Distributor
The z-axis dome geometry from TM-2026-017. Above ground (+z) is presentation, below ground (−z) is processing, z=0 is the equatorial distributor plane. Implements (7, 11, 13) coprime walk over 540 nodes — gcd(1001, 540)=1, full coverage guaranteed. Modules in `src/kernel/src/distributor/`: `coprime_walk.rs` (parallel walkers with stride 7/11/13), `z_router.rs` (routes requests to z-levels −6 through +n), `sponge_rekey.rs` (per-frame TLSponge-385 keystream advance). Layer stubs in `src/kernel/src/layers/` for all z-levels: gateway (−1), services (−2), conventional (−3), ternary_native (−4), data (−5), infrastructure (−6), fileserver (+2), snapshots (+3..+n).

### TIS-27 Keyboard Input
Kernel-space TIS-27 encoding in `src/kernel/src/input/keyboard.rs`. Scancodes encoded before any buffer using 54-trit sponge (4 rounds, 43-bit integrity). Decoded to Unicode inside browser DOM handler at last possible moment — direct kernel call, no IPC.

### XPlenum RISC-V Hardware Extension
A custom RISC-V extension integrated with CVA6 provides 21 custom instructions and 12 custom CSRs for ternary security operations, PQC acceleration, and compliance.

### TL-KEM — Ternary Lattice Key Encapsulation
TL-KEM is a ternary-native equivalent of ML-KEM (FIPS 203) providing IND-CCA2 secure key encapsulation at three security levels: TL-KEM-512, TL-KEM-768, TL-KEM-1024. Built on Module-LWE over R_q = Z_3[X]/(X^256+1) with Fujisaki-Okamoto transform.

### Crypto Benchmark Suite
A Criterion-based statistical benchmark suite covers all core cryptographic primitives: TIS-27, TLSponge-385, TL-DSA, TL-KEM, Phase Encryption v3, and raw sponge permutation.

### Sponge Architecture
TL-Sponge-385 provides 385-bit post-quantum security for signing, key derivation, FIPS validation, and document hashing, including a chi layer over GF(27). Implementations exist in TypeScript, Rust kernel (scalar + AVX2 split-table), and Rust ternary-math (scalar + AVX2/NEON SIMD). A Rust N-API native addon provides compiled native permutation to Node.js with AVX2 SIMD. TL-Sponge-43 is used for TDNS identity derivation and TIS-27 for fast integrity checks.

### TTC v4.2 Compression Pipeline
File compression uses the TTC v4.2 native Rust engine via N-API. The pipeline includes domain analysis, ternary rANS, and GURFT fast-path. Frontend displays TTC metadata badges, with round-trip verification using CRC32.

### TDNS v2.5.0 — Ternary Domain Name System
A standalone Rust crate implementing a 27-dimensional ontological addressing protocol with 54-trit dual-layer addressing. It uses TL-Sponge-43 for identity derivation and TIS-27 for wire packet integrity.

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
Authorization uses unforgeable, self-contained, bearer-verified capability tokens signed with TL-DSA, implemented across six phases.

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