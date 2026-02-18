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

### XPlenum RISC-V Hardware Extension (Phases 1–8 Complete)
CVA6-integrated custom RISC-V extension for ternary security operations:
-   **Core Selection**: CVA6 (OpenHW Group) selected over Rocket, BOOM, PicoRV32 (score 4.85/5).
-   **Integration RTL**: `rtl/integration/` — CVA6 wrapper (64-bit data path, sign extension), stall controller (RAW/WAW/structural hazard detection, result forwarding), top-level integration module.
-   **21 Custom Instructions**: Masking (TMASK/TUNMASK/TMASKR/TMASKRF), Domain (TDOMSET/TDOMCHK/TDOMCLR/TDOMXFR), Capability (TCAPST/TCAPLD/TCAPCHK/TCAPREV), Crypto (TROTL/TROTR/TTBOX/TPERM), Encoding (TTRIT/TDETRIT), Signal (TSIGFLT/TSIGCMP/TSIGACC).
-   **12 Custom CSRs**: 0x7C0–0x7CB (XPSTATUS, XPDOMID, XPCAPBASE, XPCAPBOUND, XPMASK_SEED, XPMASK_STATE, XPTRIT_MODE, XPSIG_CFG, XPEXC_CAUSE, XPEXC_ADDR, XPPERF_CNT, XPVERSION).
-   **Formal Verification**: 115+ properties across standalone (454-line) + integration (65 new) files.
-   **Testing**: 31-test integration testbench, DRBG testbench, regression framework, benchmark suite, side-channel analysis scripts.
-   **Phase 4 — NIST DRBG**: LFSR replaced with SP 800-90A CTR_DRBG (AES-256). Includes `rtl/xplenum_aes256_core.v` (14-round AES-256), `rtl/xplenum_ctr_drbg.v` (CTR_DRBG FSM with Instantiate/Generate/Reseed/Update), SP 800-90B health tests (repetition count, adaptive proportion), NIST STS validation script (`scripts/xplenum_drbg_nist_sts.py`), DRBG testbench (`tb/xplenum_drbg_tb.v`). Health error propagated as exception; TMASKR/TMASKRF gated on drbg_buffer_valid. External 256-bit entropy port on xplenum_top.
-   **Phase 5 — Rust Kernel Interfaces**: Inline assembly wrappers for all 21 instructions (`src/kernel/src/arch/xplenum.rs`), safe abstraction layer with subsystem-enable gating and exception checking (`src/kernel/src/security/xplenum_hal.rs`), unit tests (`src/kernel/src/security/xplenum_tests.rs`), CI/CD pipeline (`.github/workflows/xplenum-riscv.yml`).
-   **Phase 6 — Emulation & Validation**: Spike ISS extension (50/50 tests passing, `sim/spike/`), QEMU TCG helpers (`sim/qemu/xplenum_qemu_helper.c`, `sim/qemu/xplenum_qemu_trans.c.inc`), kernel boot validation script, E2E security test suite (6 adversarial scenarios), security fuzzer (1M iterations, 0 invariant violations, `sim/fuzzing/`), cross-verification framework (`sim/cross-verify/`), FPGA synthesis constraints (`synth/xplenum_fpga.sdc`, `synth/xplenum_pinmap.xdc`, `synth/xplenum_synth.tcl`). Performance profiling: 19.4x aggregate HW speedup over software.
-   **Phase 7 — Documentation & Compliance**: FIPS 140-3 Level 2 compliance mapping, CNSA 2.0 compliance documentation, ISA Specification v1.0, external audit coordination package. All in `docs/xplenum/phase7_*.md`.
-   **Phase 8 — Beyond Defense-Grade**: 5 parallel tracks. Track 8A: Full-system formal verification via riscv-formal RVFI (21 instruction property modules, pipeline SVA P100-P600, no-harm proofs, SymbiYosys BMC depth 100). Track 8B: Higher-order DOM masking (3-share/4-share AND gadgets with glitch-resistant pipeline registers, Rust API, TVLA validation orders 1-3). Track 8C: PQC acceleration unit (10 new instructions for NTT butterfly, modular arithmetic, CBD/rejection sampling; ML-KEM/ML-DSA Rust API with Kyber polynomial ops). Track 8D: Red-team adversarial validation (fault injection testbench for clock/voltage/laser attacks, tamper response module with health monitoring FSM and zeroisation). Track 8E: Common Criteria GPCP cPP SFR mapping (9/9 SFRs satisfied). Total ISA: 35 instructions, ~152K gates cumulative.
-   **Key Files**: `docs/xplenum/phase4_drbg_algorithm_selection_2026-02-18.md`, `rtl/xplenum_aes256_core.v`, `rtl/xplenum_ctr_drbg.v`, `rtl/xplenum_mask_unit.v`, `src/kernel/src/arch/xplenum.rs`, `src/kernel/src/security/xplenum_hal.rs`, `sim/spike/xplenum_spike_extension.h`, `sim/qemu/xplenum_qemu_helper.c`, `sim/fuzzing/xplenum_fuzz_harness.cpp`, `synth/xplenum_fpga.sdc`, `formal/xplenum_rvfi_insn_gen.py`, `rtl/xplenum_dom_gadgets.v`, `rtl/xplenum_pqc_unit.v`, `rtl/xplenum_tamper_response.v`, `scripts/cc_sfr_mapper.py`, `docs/xplenum/phase8_beyond_defense_grade_2026-02-18.md`.

### Tonal Diffusion System
Network-wide time synchronization using FM timing packets, toroidal topology, and gradient-driven diffusion consensus. Implemented across Rust and TypeScript:
-   **Rust FM Timing Engine** (`libternary/src/fm_timing/`): Van der Pol oscillator with FM modulation (`oscillator.rs`), HRV entropy source for post-quantum key material (`hrv.rs`), FM timing packet codec with 27-trit balanced ternary timestamps (`packet.rs`), GF(3) gradient operator for ternary field computation (`gf3_gradient.rs`).
-   **Shared Topology** (`shared/topology/`): Toroidal coordinate system, distance metrics, natural neighbor selection, GF(3) arithmetic, and ternary gradient computation in TypeScript.
-   **Tonal Field Service** (`services/tonal-field/`): Field potential tracker (`field.ts`), graph Laplacian diffusion solver (`diffusion.ts`), dimensionless Plenum metrics (Pi1-Pi4) with health assessment (`metrics.ts`).
-   **Resonance Detector** (`server/resonance/`): Adaptive sync rate tuning via RTT-based network wave speed estimation, resonant frequency detection, and frequency sweep optimization.
-   **API Endpoints**: `GET /api/tonal/field`, `GET /api/tonal/neighbors`, `POST /api/tonal/packet`, `GET /api/resonance/status`, `POST /api/resonance/sweep`, `POST /api/resonance/rtt`, `GET /api/metrics/plenum`.
-   **Core Type**: `TernaryTrit` enum (Neg/Zero/Pos) in `libternary/src/lib.rs` — shared across all fm_timing modules.

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

### Ternary Ephemeris API
REST API for the separate Astrology App frontend. Endpoints under `/api/v1/ephemeris/`:
-   **POST /convert**: Standard ↔ ternary degree conversion with resonance scoring.
-   **POST /position**: Single planet ephemeris (Keplerian elements, Kepler solver, ecliptic → ternary).
-   **POST /batch**: All planets at once for a given Julian Date.
-   **GET /info**: API metadata, supported planets, system constants.
-   Supports: sun, moon, mercury, venus, earth, mars, jupiter, saturn, uranus, neptune, pluto.
-   Ternary system: 364° circle, 13° ternary radian, Z₂₈ lattice (resonance only — no snapping).
-   Key files: `server/ternary-ephemeris.ts` (math engine), `server/routes/ephemeris.ts` (API routes).
-   Integration docs: `REPLIT_AI_INSTRUCTIONS.md`, `TERNARY_EPHEMERIS_INTEGRATION_GUIDE.md`.
-   CORS allows `*.replit.dev` and `*.replit.app` for cross-origin frontend access.

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