# PlenumNET / Salvi Framework

Post-quantum ternary computing platform implementing the Unified 13D Torsion Plenum Theory. Production-grade infrastructure for quantum-resistant data operations, femtosecond timing, and CNSA 2.0 compliant cryptography.

## Status

| Metric | Value |
|--------|-------|
| Roadmap | 80/80 milestones complete |
| Commits | 1,252+ |
| Crypto modules | 34 |
| CNSA 2.0 coverage | 11/11 algorithms (100%) |
| Test count | 1,011+ |
| CMVP target | FIPS 140-3 Level 1 |
| VM opcodes | 35 |

## Architecture

```
libternary/          TypeScript library — GF(3) arithmetic, phase encryption,
                     femtosecond timing, Tribonacci constants
shared/              Centralized constants (single source of truth for TAU)
server/salvi-core/   Express API server — timing, ternary ops, calendars,
                     payment/witnessing, blockchain integrations
src/kernel/          Rust kernel — crypto, VM, memory, I/O, filesystem,
                     process scheduler, device drivers, HPTP, torsion network
client/              React frontend — investor demo, docs, admin dashboard
kong/                Kong Konnect gateway config (18 services, 70+ endpoints)
salvi_docs/          Developer documentation (15 modules, 7,300+ lines)
```

## Applied Physics vs. Theoretical Validation

This project distinguishes between two kinds of claims:

**Applied validation (the formulas work):** The Tribonacci constant tau satisfies tau^3 = tau^2 + tau + 1. The GF(3) arithmetic is mathematically correct. The density advantage log2(3) - 1 = 58.5% holds exactly. The CNSA 2.0 algorithms pass their KAT vectors. These are verifiable and tested.

**Theoretical validation (the theory is true):** Whether tau actually emerges from SO(8) quantum graph stability, or whether dark energy really involves instanton suppression at S_inst = 2*tau^7, are physics questions outside the scope of this codebase. The software implements these formulas faithfully and produces consistent results, but does not claim to prove the underlying physics.

## Core Modules

### GF(3) Ternary Arithmetic
Balanced ternary operations in GF(3) with three bijective representations:
- **A (Computational):** {-1, 0, +1}
- **B (Network):** {0, 1, 2}
- **C (Human):** {1, 2, 3}

All operations use a correct ring isomorphism via modular arithmetic. Full 9-case addition and multiplication tables are tested.

### Tribonacci Constants
The constant tau = 1.8392867552141612 and its derived values appear throughout the system:
- Hash seeds and mix constants (tau^2, tau^7)
- VM register count: 27 = 3^3
- Hash finalization rounds: 13 = T(7)
- Information density: log2(3) = 1.585 bits per trit
- GC threshold ratio: tau^-2

Constants are centralized in `shared/tribonacci-constants.ts` and mirrored in `src/kernel/src/vm/constants.rs` for the Rust kernel.

### HPTP Femtosecond Timing
High-Precision Timing Protocol providing femtosecond-scale timestamps anchored to the Salvi Epoch (April 1, 2025 00:00:00 UTC). Includes:
- Network latency correction
- Self-test endpoint (1000-sample jitter analysis)
- Synchronization across 24 global calendar systems spanning 30,000+ years

### CNSA 2.0 Cryptographic Suite
11/11 required algorithms implemented in Rust:
- ML-KEM (FIPS 203) at 3 security levels
- ML-DSA (FIPS 204) at 3 security levels
- AES-256-GCM, SHA-384/SHA-512, HMAC-SHA-384
- XMSS and LMS stateful hash-based signatures (SP 800-208)
- Constant-time primitives throughout

### Phase Encryption
Adaptive dual-phase quantum encryption with guardian phase tamper detection using Tribonacci-weighted checksums.

### Ternary Virtual Machine
35-opcode ISA with ternary-native instructions (TAdd, TMul, TNeg, TRot, TXor, TConvert), 27 registers, mark-sweep GC, and theory-derived constants for cycle limits and buffer sizes.

### PlenumDB
Ternary-encoded data storage demonstrating the 58.5% information density advantage. Live compression demo with benchmark validation endpoint.

## Self-Test Endpoints

```
GET /api/salvi/ternary/density-benchmark   — Validates 59% density claim at 4 sample sizes
GET /api/salvi/timing/self-test            — 1000-sample timer resolution and jitter analysis
```

## Running Tests

```bash
npx tsx libternary/tests/ternary-operations.test.ts   # GF(3) arithmetic (58 tests)
npx tsx libternary/tests/tribonacci.test.ts           # Tribonacci module (58 tests)
```

## API Surface

The platform exposes 70+ endpoints through Kong Konnect:

| Category | Endpoints | Description |
|----------|-----------|-------------|
| Ternary Ops | `/api/salvi/ternary/*` | Add, multiply, XOR, rotate, batch, density |
| Timing | `/api/salvi/timing/*` | Timestamps, metrics, batch, epoch anchors |
| Calendars | `/api/salvi/timing/epoch/calendars/*` | 24 calendar system conversions |
| Phase | `/api/salvi/phase/*` | Split, recombine, config, recommend |
| Compression | `/api/demo/*` | PlenumDB live demo, stats, history |

## Development

```bash
npm run dev           # Start Express + Vite dev server on port 5000
```

## License

All Rights Reserved and Preserved. Copyright Capomastro Holdings Ltd 2026.
