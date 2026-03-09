# PlenumNET / Salvi Framework

[![Salvi Framework CI](https://github.com/SigmaWolf-8/Ternary/actions/workflows/ci.yml/badge.svg)](https://github.com/SigmaWolf-8/Ternary/actions/workflows/ci.yml)
[![Security Scan](https://github.com/SigmaWolf-8/Ternary/actions/workflows/security-scan.yml/badge.svg)](https://github.com/SigmaWolf-8/Ternary/actions/workflows/security-scan.yml)
[![License Check](https://github.com/SigmaWolf-8/Ternary/actions/workflows/license-check.yml/badge.svg)](https://github.com/SigmaWolf-8/Ternary/actions/workflows/license-check.yml)
[![OWASP Scan](https://github.com/SigmaWolf-8/Ternary/actions/workflows/owasp-scan.yml/badge.svg)](https://github.com/SigmaWolf-8/Ternary/actions/workflows/owasp-scan.yml)
[![CodeQL](https://github.com/SigmaWolf-8/Ternary/actions/workflows/codeql-analysis.yml/badge.svg)](https://github.com/SigmaWolf-8/Ternary/actions/workflows/codeql-analysis.yml)

Post-quantum ternary computing platform implementing the Unified 13D Torsion Plenum Theory. Production-grade infrastructure for quantum-resistant data operations, femtosecond timing, and CNSA 2.0 compliant cryptography.

> **Export Control Notice:** This software contains post-quantum cryptography subject to export controls under U.S. EAR (ECCN 5D002), Canadian ECL (Category 5, Part 2), and the Wassenaar Arrangement. Distribution to restricted countries (Cuba, Iran, North Korea, Syria, Russia, Belarus) may require authorization. See [EXPORT-CONTROL.md](EXPORT-CONTROL.md) for full classification details.

## Status

| Metric | Value |
|--------|-------|
| Roadmap | 80/80 milestones complete |
| Commits | 1,252+ |
| Crypto modules | 34 |
| CNSA 2.0 coverage | 11/11 algorithms (100%) |
| Test count | 2,276 (1,783 Rust + 493 TypeScript) |
| Kernel | 123 files, ~54,780 LOC |
| CMVP target | FIPS 140-3 Level 1 |
| VM opcodes | 176 (ISA v2.1) |
| Quantum modules | 5 (qutrit/qudit simulation) |
| Kong gateway | 33 services, 293 endpoints |

## Performance

All numbers measured via [Criterion](https://github.com/bheisler/criterion.rs) on x86-64 (AVX2) and aarch64 (NEON). Not estimated.

### TL-DSA vs ML-DSA (FIPS 204) — Full Roundtrip

TL-DSA achieves 2.5-2.9x faster signing & verification than ML-DSA at the same NIST security levels — using pure ternary arithmetic and first-principles optimizations.

| Security Level | TL-DSA (measured) | ML-DSA (FIPS 204 ref) | Speedup |
|----------------|-------------------|-----------------------|---------|
| 128-bit | **653 us** | ~1,600 us | **2.5x** |
| 192-bit | **963 us** | ~2,700 us | **2.8x** |
| 256-bit | **1,441 us** | ~4,200 us | **2.9x** |

Optimizations: Integer NTT (q=12289), XOF-batched sponge expansion, GF(3)-associative balanced_wrap, AVX2-vectorized substitution (32 trits/cycle).

<details>
<summary>TL-DSA-87 breakdown (keygen / sign / verify)</summary>

| Operation | Time |
|-----------|------|
| Keygen | 723 us |
| Sign | 383 us |
| Verify | 357 us |
| **Full roundtrip** | **1,441 us** |

</details>

### Sponge Permutation (Kernel Cryptographic Sponge)

729-trit state, 9 rounds, 7-neighbor extended theta, 385-bit post-quantum security. Three-tier SIMD dispatch: AVX2 (x86-64), NEON (aarch64), scalar fallback.

| Benchmark | x86 (AVX2) | ARM (NEON) | vs Scalar |
|-----------|------------|------------|-----------|
| Classification hash (27 trits) | 3.2 us | 3.2 us | 6-10x |
| 243-trit (1 block) | 3.4 us | 3.1 us | ~6x |
| 729-trit (3 blocks) | 10.2 us | 9.4 us | ~7x |
| Identity (short URL) | 3.4 us | 4.1 us | — |
| Identity (medium URL) | 6.5 us | 6.6 us | — |
| Identity (long URL) | 10.2 us | 9.5 us | — |
| Throughput | ~14 MB/s | ~14 MB/s | — |

### TIS-27 Integrity Hash

Fast non-cryptographic ternary integrity function (state=54, rate=27, 4 rounds, 43-bit). Used for wire packet integrity and TDNS scan hashing. NOT a cryptographic hash.

| Benchmark | Time | Throughput |
|-----------|------|------------|
| Single hash (54 trits) | **191 ns** | — |
| 27-trit block | 184 ns | 140 MB/s |
| 243-trit block | 186 ns | 1.22 GB/s |
| 512-trit block | 188 ns | 2.52 GB/s |

TIS-27 vs SHA-256 (27-byte input): 191 ns vs 669 ns — **3.5x faster** with native GF(3) output.

### Benchmark Harnesses

```bash
# Kernel sponge (AVX2/NEON/scalar)
cd src/kernel && cargo bench --bench sponge_bench

# TL-DSA (all three security levels)
cd src/kernel && cargo bench --bench tldsa_bench

# TIS-27 integrity hash
cd ternary-math && cargo bench --bench tis27_bench
```

## Architecture

```
libternary/          Rust core — GF(3) arithmetic, cdylib + WASM (wasm-bindgen)
ternary-math/        Standalone math crate (5,154 LOC, 11 modules + TIS-27 sponge)
shared/              Centralized constants, repunit circles, agent generators,
                     CRT fast path, checksum (single source of truth)
server/salvi-core/   Express API server — timing, ternary ops, calendars,
                     payment/witnessing, blockchain integrations
server/routes/       TDNS v2.5, Kong catalog, TSA, Hedera, capabilities,
                     security infrastructure, SFK operations
src/kernel/          Rust kernel (123 files, ~54,780 LOC) — crypto, VM, memory,
                     I/O, filesystem, process scheduler, device drivers, HPTP,
                     torsion network, inter-cube, bare-metal validation
client/              React frontend — 26 pages, investor demo, docs, admin
kong/                Kong Konnect gateway config (33 services, 293 endpoints)
services/tdns-v2/    TDNS v2.5 Rust crate (19 modules) + Chrome extension v1.0.9
salvi_docs/          Developer documentation
```

## Applied Physics vs. Theoretical Validation

This project distinguishes between two kinds of claims:

**Applied validation (the formulas work):** The Tribonacci constant tau satisfies tau^3 = tau^2 + tau + 1. The GF(3) arithmetic is mathematically correct. The density advantage log2(3) - 1 = 58.5% holds exactly. The CNSA 2.0 algorithms pass their KAT vectors. These are verifiable and tested.

**Theoretical validation (the theory is true):** Whether tau actually emerges from SO(8) quantum graph stability, or whether dark energy really involves instanton suppression at S_inst = 2*tau^7, are physics questions outside the scope of this codebase. The software implements these formulas faithfully and produces consistent results, but does not claim to prove the underlying physics.

## Core Modules

### GF(3) Ternary Arithmetic
Balanced ternary operations in GF(3) with three bijective representations:
- **A (Balanced):** {-1, 0, +1} — signed arithmetic, negation
- **B (Standard):** {0, 1, 2} — recurrence, analysis (internal only)
- **C (Bijective):** {1, 2, 3} — wire format, TDNS, crypto (THE external representation)

All operations use a correct ring isomorphism via modular arithmetic. Full 9-case addition and multiplication tables are tested. Zero in Rep C is the forgery sentinel.

### TDNS v2.5.0 — Ternary Domain Name System
54-trit dual-layer ontological addressing (27 classification + 27 identity anchor). Identity derivation uses the cryptographic sponge (state=54, rate=27, 9 rounds, 7-neighbor theta). Scan hashing uses TIS-27 (4 rounds, gather pi, direct copy absorption). Supports Org Entities for multi-URL grouping. 9 API routes. Chrome extension (v1.0.9) renders dual-color addresses: classification in gold, identity anchor in sky blue.

### Tribonacci Constants
The constant tau = 1.8392867552141612 and its derived values appear throughout the system:
- Hash seeds and mix constants (tau^2, tau^7)
- VM register count: 27 = 3^3
- Hash finalization rounds: 13 = T(7)
- Information density: log2(3) = 1.585 bits per trit
- GC threshold ratio: tau^-2

Constants are centralized in `shared/tribonacci-constants.ts` and mirrored in `src/kernel/src/vm/constants.rs` for the Rust kernel.

### Ternary Circle (364°)
The ternary circle is defined by 111111₃ = 364 = full circle in degrees. Constants are structurally bound: π = 14 (exact integer), 1 radian = 13° = T₇ = 111₃, 2π = 28 radians. The product 13 × 28 = 364 is the calendar identity (13 moons of 28 days). Base-3 repunits R(n) = (3ⁿ − 1) / 2 define the geometric cycle hierarchy (R₃–R₉).

### HPTP Femtosecond Timing
High-Precision Timing Protocol providing femtosecond-scale timestamps anchored to the Salvi Epoch (April 1, 2025 00:00:00 UTC). 7 files, 2,369 LOC in the kernel. Includes:
- 7 clock sources (Local, GPSDO, Atomic Rb/Cs, Optical Lattice, Chip-Scale, Network Peer)
- Coprime clock rotation (generator theorem for failover)
- CRT fast path (Z₃₆₄ ≅ Z₂₈ × Z₁₃ decomposition for O(1) calendar indexing)
- Synchronization across 42 global calendar systems spanning 30,000+ years

### CNSA 2.0 Cryptographic Suite
11/11 required algorithms implemented in Rust:
- ML-KEM (FIPS 203) at 3 security levels
- ML-DSA (FIPS 204) at 3 security levels
- AES-256-GCM, SHA-384/SHA-512, HMAC-SHA-384
- XMSS and LMS stateful hash-based signatures (SP 800-208)
- TL-DSA/TL-KEM (ternary lattice post-quantum crypto)
- Constant-time primitives throughout

### Phase Encryption
Adaptive dual-phase quantum encryption with guardian phase tamper detection using Tribonacci-weighted checksums.

### Ternary Virtual Machine
176-opcode ISA v2.1 with ternary-native instructions (TAdd, TMul, TNeg, TRot, TXor, TConvert), quantum-ternary simulation (opcodes 0xA0–0xAF), 27 registers, mark-sweep GC, and theory-derived constants for cycle limits and buffer sizes. Fully backward compatible with v2.0's 160 opcodes and v1.0's 62 opcodes.

### PlenumDB
Ternary-encoded data storage demonstrating the 58.5% information density advantage. Live compression demo with benchmark validation endpoint.

### Inter-Cube Infrastructure
4-service system for geometric routing across the 13D ternary cube network:
- Geometric Load Balancer (GLB) — d! shortest paths via dimension ordering
- Cube Overlay Network (CON) — 26 TIS-27-derived encrypted tunnels to geometric neighbors
- Cube Registration Service (CRS) — address allocation + endpoint registry
- Fault Tolerance Service (FTS) — heartbeat monitoring + dead neighbor set

### 28-Dimension Agent Array
Maps Z₂₈ cyclic positions to 28 parallel AI agents via (position × 13) mod 28 coprime walk. All 12 generators of Z₂₈ supported for fault-tolerant parallel scheduling. Features: Etymology Audit, Veritas Fact-Check, unified Situation Report, Lexical Protocol enforcement.

### RFC 3161 Time-Stamping Authority (TSA)
Digital notary providing cryptographic proof-of-existence timestamps per RFC 3161. Four TSA policies, Merkle tamper-evident audit log, dual-signature (RSA-4096 + TL-DSA-87), HPTP timing integration, ASN.1 wire protocol. CLI: `plenum-stamp`.

### Hedera HCS Witnessing
Blockchain-based non-repudiation via Hedera Consensus Service. Submits cryptographic witness hashes to an HCS topic for immutable, ordered, timestamped proof of PlenumNET operations.

## Self-Test Endpoints

```
GET /api/salvi/ternary/density-benchmark   — Validates 59% density claim at 4 sample sizes
GET /api/salvi/timing/self-test            — 1000-sample timer resolution and jitter analysis
```

### Quantum Ternary Modules
Classical simulation of quantum ternary (qutrit/qudit) operations:
- **Complex Utilities** — Self-contained complex arithmetic (no external deps)
- **Qutrit Basics** — 3-level quantum states coupled to SUFT branches, phase gates, Gell-Mann generators
- **Lagrangian Qutrit** — Discrete Euler-Lagrange for qutrit evolution, Tribonacci-weighted potential
- **Qutrit Fault Tolerance** — Stabilizer codes, syndrome measurement, triorthogonal distillation
- **Qudit Basics** — Generalized d>=2 quantum states (qubit through d=13 SUFT)

### Scientific Integrations
- **Saturnian Magic Square** — 3x3 circulant foundation for SUFT-derived constants
- **Hamiltonian Mechanics** — Symplectic jitter correction, energy invariant enforcement
- **Lagrangian Mechanics** — Discrete Euler-Lagrange for ternary logic
- **Noether Symmetries** — Conserved quantities for ternary gauge/reparametrization
- **Tribonacci Variational** — Discrete variational functionals, ratio convergence to tau

## Running Tests

```bash
# Rust tests (1,783 tests)
cargo test --release --all-features

# TypeScript tests (493 tests)
npx vitest run

# TDNS E2E tests (84 assertions)
node tis27-e2e-tests.js
```

## API Surface

The platform exposes 293 endpoints through 33 Kong Konnect services:

| Category | Endpoints | Description |
|----------|-----------|-------------|
| Ternary Ops | `/api/salvi/ternary/*` | Add, multiply, XOR, rotate, batch, density |
| Timing | `/api/salvi/timing/*` | Timestamps, metrics, batch, epoch anchors |
| Calendars | `/api/salvi/timing/epoch/calendars/*` | 42 calendar system conversions |
| Phase | `/api/salvi/phase/*` | Split, recombine, config, recommend |
| Compression | `/api/demo/*` | PlenumDB live demo, stats, history |
| TDNS | `/api/tdns/*` | Scan, register, resolve, org entities, health |
| TSA | `/api/tsa/*` | RFC 3161 timestamps, verify, audit, policies |
| Hedera | `/api/hedera/*` | HCS witnessing, topic info, message history |
| Capabilities | `/api/capabilities/*` | 6-phase capability token lifecycle |
| Security | `/api/security/*` | Audit, anomaly detection, threat model |
| Inter-Cube | `/api/salvi/inter-cube/*` | GLB, CON, CRS, FTS topology services |

## Development

```bash
npm run dev           # Start Express + Vite dev server on port 5000
```

## Legal & Compliance

All legal and compliance documents are indexed in [docs/legal/INDEX.md](docs/legal/INDEX.md), including:
- Privacy Policy (GDPR/PIPEDA compliant)
- Export Control (ECCN 5D002, Wassenaar)
- Data Processing Agreement
- Incident Response Plan
- Enterprise SLA

See [CONTRIBUTORS.md](CONTRIBUTORS.md) for contributor information.

## License

All Rights Reserved and Preserved. Copyright Capomastro Holdings Ltd 2026.
