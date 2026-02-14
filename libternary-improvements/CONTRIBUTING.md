# Contributing to PlenumNET · Salvi Framework

Welcome, Fratello. PlenumNET is the world's first ternary computing platform, sitting at the intersection of ternary computing, post-quantum cryptography, and theoretical physics. This guide ensures you have the context needed to contribute meaningfully — and that your contributions maintain the mathematical rigor the framework demands.

---

## Table of Contents

1. [The 60-Second Version](#the-60-second-version)
2. [Core Concept: The Tribonacci Constant τ](#core-concept-the-tribonacci-constant-τ)
3. [Core Concept: GF(3) — Why Ternary](#core-concept-gf3--why-ternary)
4. [Core Concept: 13 Dimensions and the VM](#core-concept-13-dimensions-and-the-vm)
5. [Core Concept: Applied vs. Theoretical Physics](#core-concept-applied-vs-theoretical-physics)
6. [Theory Prerequisites by Layer](#theory-prerequisites-by-layer)
7. [Project Structure](#project-structure)
8. [Key Constants Reference](#key-constants-reference)
9. [Development Setup](#development-setup)
10. [Contribution Workflow](#contribution-workflow)
11. [Coding Standards](#coding-standards)
12. [Testing Requirements](#testing-requirements)
13. [What to Work On](#what-to-work-on)
14. [Submitting Changes](#submitting-changes)
15. [Architecture Decision Records](#architecture-decision-records)
16. [Further Reading](#further-reading)

---

## The 60-Second Version

PlenumNET implements a ternary computing system where the core mathematics come from the **Unified 13D Torsion Plenum Theory V9.8Rf**. Here's what that means in practice:

- **Ternary, not binary.** Every value is a trit (-1, 0, +1) instead of a bit (0, 1). This gives 59% more information per digit (log₂(3) ≈ 1.585 bits per trit).
- **One constant drives everything.** The Tribonacci constant τ ≈ 1.839 appears throughout the codebase — in hash seeds, VM parameters, timing constants, and buffer sizes. It's not arbitrary; it's the real root of τ³ = τ² + τ + 1.
- **The math is in production.** This isn't a simulation. The GF(3) arithmetic, the encryption, the compression — they run in the Rust kernel with 1,011+ tests passing. When you change code, you're changing a working system.

If a function uses a number like `2757` or `142.42` or `13`, it's almost certainly derived from τ. Check the constants module before treating it as a magic number.

---

## Core Concept: The Tribonacci Constant τ

### What It Is

τ (tau) is the Tribonacci constant: the unique real root of the equation:

```
τ³ = τ² + τ + 1
```

Its value is approximately **1.8392867552141612**.

This is analogous to how the golden ratio φ is the root of φ² = φ + 1, but τ satisfies a cubic instead of a quadratic. Just as φ appears throughout nature and the Fibonacci sequence, τ appears throughout the Tribonacci sequence and (according to the theory) throughout fundamental physics.

**Important:** τ is *not* the mathematical constant 2π (sometimes called τ in other contexts). In the Salvi Framework, τ encodes the relationship between the torsion field density and the high-precision timing protocol (HPTP).

### Why It Matters for the Code

τ is not just a theoretical curiosity — it's the source of most constants in the codebase:

| Constant | Formula | Value | Where It Appears |
|----------|---------|-------|------------------|
| τ¹³ | τ^13 | 2757.038 | Fundamental period, buffer sizing |
| S_inst | 2τ⁷ | 142.42 | Instanton action, hash mixing seed |
| Δθ | 9/τ⁵ | 0.4275° | Rotation constants |
| M₁ | — | 1.30 TeV | Resonance mass calculations |
| log₂(3) | — | 1.585 | Information density ratio |

**Rule:** If you need a new constant, derive it from τ when possible. Import from the `tribonacci` module — never hardcode τ's numeric value directly.

### The Tribonacci Sequence

The Tribonacci sequence is defined by T(n) = T(n-1) + T(n-2) + T(n-3), starting with T(0)=0, T(1)=0, T(2)=1:

```
0, 0, 1, 1, 2, 4, 7, 13, 24, 44, 81, 149, 274, 504, 927, ...
```

The critical value is **T(7) = 13**, which connects the 7-cycle in the compact manifold to the 13 dimensions of the theory. You'll see the numbers 7 and 13 appear frequently in the codebase — they come from here.

### How to Verify

The `tribonacci` module includes a `verifyTau()` function that confirms τ³ = τ² + τ + 1 to machine precision. The CI pipeline runs this on every PR. `verifyTau()` is also a self-test intrinsic (one of the 160 VM opcodes) that checks whether a set of GF(3) polynomial identities still hold after a state transition. If `verifyTau()` fails, the VM halts — a timing-inconsistent state is considered corrupted.

You do not need to derive τ from first principles to contribute, but you must never write code that bypasses or mocks `verifyTau()` in production paths.

---

## Core Concept: GF(3) — Why Ternary

### What GF(3) Is

GF(3) is the **Galois Field with 3 elements**: {0, 1, 2}. It is a *field*, meaning every non-zero element has a multiplicative inverse (1⁻¹ = 1, 2⁻¹ = 2 since 2 × 2 = 4 ≡ 1 mod 3). The entire `libternary` kernel operates in GF(3). See [ADR-002](docs/adr/ADR-002-gf3-over-balanced-ternary.md) for the rationale.

If this is new to you, read:
- Lidl & Niederreiter, *Finite Fields* — Chapter 1 (field axioms, GF(p) construction)
- Or, more accessibly: the "Finite Field" article on Wikipedia through the section on prime fields

### Balanced Ternary and the Correct Mapping

PlenumNET uses **balanced ternary** representation: {-1, 0, +1} instead of {0, 1, 2}. The mapping between them is the standard modular equivalence:

```
Balanced Ternary    GF(3)
     -1        ↔      2
      0        ↔      0
     +1        ↔      1
```

Conversion: `toGF3(a) = ((a % 3) + 3) % 3`

### Why This Mapping Matters

Getting this mapping wrong breaks everything. The previous implementation used `f(a) = a + 1` (mapping -1→0, 0→1, 1→2), which is **not** a ring homomorphism — it maps -1 to GF(3)'s additive identity (0), which means -1 behaves like zero in arithmetic operations.

Example of what goes wrong with the broken mapping:

| Operation | Wrong result (old) | Correct result | Why |
|-----------|-------------------|----------------|-----|
| (-1) × (-1) | -1 | +1 | Old code: 0×0=0→-1. Correct: 2×2=4%3=1→+1 |
| (-1) + (-1) | -1 | +1 | Old code: 0+0=0→-1. Correct: 2+2=4%3=1→+1 |
| 1 + 1 | 0 | -1 | Old code: 2+2=4%3=1→0. Correct: 1+1=2→-1 |

**Rule:** Never write your own balanced-ternary-to-GF(3) conversion. Use `toGF3()` and `fromGF3()` from the ternary operations module.

### The Three Representations

PlenumNET supports three trit representations:

- **Representation A** (balanced/computational): {-1, 0, +1} — the native representation for arithmetic
- **Representation B** (unbalanced/network): {0, 1, 2} — matches GF(3) directly
- **Representation C** (bijective/human): {1, 2, 3} — used for encoding where zero is problematic

The binary-ternary gateway handles conversion to/from binary for compatibility with x86_64 hardware.

### Information Density

The fundamental advantage of ternary: each trit carries log₂(3) ≈ 1.585 bits of information, compared to 1 bit per bit. That's a **59% density advantage** — the same data fits in fewer digits. PlenumDB's compression is built on this principle.

---

## Core Concept: 13 Dimensions and the VM

### Where 13 Comes From

The theory proposes reality has 13 dimensions:
- 4 spacetime dimensions (the ones we experience)
- 8 gauge dimensions (from SO(8) symmetry group, with dim = 28)
- 1 time-angle dimension

The number 13 is not chosen arbitrarily — it emerges from the Tribonacci sequence. T(7) = 13, connecting the 7-cycle in the compact manifold K₉ to the total dimensionality.

### How This Affects the VM

The 160-opcode virtual machine (ISA v2.0) uses theory-derived parameters:

| VM Parameter | Value | Derivation |
|-------------|-------|------------|
| Opcode count | 160 | ISA v2.0 enterprise-grade expansion (backward compatible with v1.0's 62) |
| Finalization rounds | 13 | D = 13, the dimensional constant |
| Hash seed base | τ² | From SO(8) graph stability |
| Hash mixing multiplier | τ⁷ | Instanton action volume |
| GC cycle interval | τ¹³ | Fundamental period constant |

The VM has 6 timing/density opcodes that read from and write to a 13-component state vector. Each component is a GF(3) value (or a GF(3^k) extension for higher-trit-width components). The 13 components decompose as: 3 spatial + 1 temporal + 9 torsion-field degrees of freedom.

When you see magic numbers in the VM code, check whether they're derived from τ. If they are, they should import from the constants module with a comment explaining the derivation.

**Operational rule:** You do not need to understand general relativity or differential geometry to work on the VM — but you must not silently reduce the dimensionality (e.g., treating the state vector as 4D by ignoring the torsion components).

### The SO(8) Connection

SO(8) is the Special Orthogonal group in 8 dimensions — a mathematical structure with 28 dimensions (D₄ gauge group). It has a unique property called **triality**: three equivalent 8-dimensional representations that can be permuted. The theory uses this triality to explain why there are exactly 3 generations of fermions (electron, muon, tau).

For contributors: you don't need to understand SO(8) group theory to work on the codebase. What matters is that when you see the number 28 (gauge dimensions) or 3 (fermion generations or trit values), they connect back to SO(8) triality. The `DERIVED_CONSTANTS.D4_DIM = 28` constant captures this.

---

## Core Concept: Applied vs. Theoretical Physics

PlenumNET draws a clear line between two things:

1. **Applied physics validation**: The formulas work in production. GF(3) arithmetic is correct, ternary compression achieves 59% density gain, HPTP timing reaches femtosecond precision. These are engineering facts verified by 1,011+ tests.

2. **Theoretical physics validation**: Whether the 13D Torsion Plenum Theory accurately describes reality. This requires experimental confirmation of predictions like the M₁ = 1.30 TeV resonance at the LHC.

**As a contributor, you are doing applied physics.** Your code needs to be correct, tested, and performant. The theoretical claims are documented in the whitepaper and the Project 13D site — they inform design decisions but your PR won't be judged on whether the theory is true, only on whether the code works.

---

## Theory Prerequisites by Layer

You do not need to be an expert in all areas — but you must understand the *specific subset* relevant to the layer you are touching.

| Layer | Directory | You Need to Understand |
|-------|-----------|----------------------|
| **Rust Kernel** (`libternary`) | `/libternary`, `/src/kernel` | GF(3) arithmetic, bijective mappings, the 160-opcode ISA v2.0 specification (ADR-001), `unsafe` Rust and FFI conventions |
| **Cryptographic Modules** | `/libternary/crypto`, `/services` | Lamport OTS, Merkle tree authentication, lattice-based KEMs (Kyber/ML-KEM), lattice-based signatures (Dilithium/ML-DSA), CNSA 2.0 requirements. Read ADR-003. |
| **Frontend / Dashboard** | `/client` | React, TypeScript, Tailwind. No physics prerequisites — but understand what the API responses *mean* (trit values are {0, 1, 2} in Representation B, not {-1, 0, 1}). |
| **Server / API** | `/server`, `/shared` | Express/Node.js patterns, Drizzle ORM. Understand that API payloads carry GF(3)-encoded data. |
| **Smart Contracts** | `/contracts` | Solidity fundamentals, blockchain state models. Understand how Lamport key-state is tracked on-chain. |
| **Kong Gateway / Infra** | `/kong`, `/deployments` | Kong Konnect configuration, Docker, HPTP timing requirements for API gateway latency budgets. |
| **CI / Testing** | `/.github`, `/tests` | GitHub Actions YAML, the theory-validation pipeline, what constitutes a valid `verifyTau()` pass. |

---

## Project Structure

```
Ternary/
├── .github/              # CI workflows, issue templates
├── client/               # React/TypeScript frontend (Vite + Tailwind)
│   └── src/
│       ├── pages/        # Route pages (Landing, HPTP Demo, PlenumDB, etc.)
│       └── components/   # Reusable UI components
├── contracts/            # Blockchain smart contracts
├── deployments/docker/   # Docker compose & Dockerfiles
├── docs/
│   ├── adr/              # Architecture Decision Records
│   └── deployment/       # Deployment runbooks
├── kong/                 # Kong Konnect API gateway config
├── libternary/           # TypeScript ternary library
│   └── src/
│       ├── tribonacci.ts         # τ constants, sequences, derived values
│       ├── ternary-operations.ts # GF(3) arithmetic (add, multiply, rotate)
│       ├── phase-encryption.ts   # Dual-phase encryption with guardian checksum
│       ├── femtosecond-timing.ts # HPTP timing protocol
│       ├── ternary-types.ts      # Type definitions (TritA, Representation, etc.)
│       └── index.ts              # Public API exports
├── scripts/              # Build & utility scripts
├── server/               # Node.js/Express API server
├── services/             # Microservice modules
├── shared/               # Shared TypeScript types & utilities
│   └── tribonacci-constants.ts  # Canonical τ values for shared use
├── src/
│   └── kernel/           # Rust kernel — GF(3) field ops, VM, crypto
│       ├── src/
│       │   ├── vm/       # 160-opcode virtual machine (ISA v2.0)
│       │   ├── crypto/   # Post-quantum encryption (CNSA 2.0)
│       │   └── compat/   # Binary-ternary gateway
│       ├── benches/      # Criterion benchmark suite
│       ├── spec/         # Machine-readable ISA specifications
│       ├── wasm/         # WebAssembly compilation target
│       ├── tests/        # Property-based tests (proptest)
│       └── fuzz/         # Fuzz testing harnesses
└── tests/integration/    # Integration test suites
```

### Key Modules

| Module | Purpose | Theory Connection |
|--------|---------|-------------------|
| `tribonacci` | τ constants and sequences | Single source of truth for all derived values |
| `ternary-operations` | GF(3) arithmetic | Core kernel math — add, multiply, rotate, XOR |
| `phase-encryption` | Dual-phase quantum encryption | Guardian phase uses τ-weighted checksum |
| `femtosecond-timing` | HPTP timing (10⁻¹⁵s) | FINRA 613 & MiFID II compliance |
| `vm/constants` | VM parameter derivations | Links opcodes and cycles to τ |
| `crypto` | CNSA 2.0 algorithms | ML-KEM, ML-DSA, AES-256-GCM, SHA-384 |
| `compat/gateway` | Binary-ternary conversion | Bridge between x86_64 binary and GF(3) ternary |

---

## Key Constants Reference

Quick reference for the constants you'll encounter most often:

```
τ    = 1.8392867552141612    (Tribonacci constant, root of τ³ = τ² + τ + 1)
τ²   = 3.3830                (hash seed base)
τ³   = 6.2223                (= τ² + τ + 1, by definition)
τ⁵   = 21.056                (used in Δθ = 9/τ⁵)
τ⁷   = 71.21                 (instanton action: S_inst = 2τ⁷ = 142.42)
τ¹³  = 2757.038              (fundamental period constant)

T(7) = 13                    (Tribonacci sequence: 7-cycle → 13 dimensions)
D₄   = 28                    (SO(8) gauge group dimension)

log₂(3) = 1.585              (bits per trit)
59%  = (log₂(3) - 1) × 100   (density advantage over binary)
```

**Import these from the `tribonacci` module.** Never hardcode them.

---

## Development Setup

### Prerequisites

- **Rust** >= 1.75 (latest stable) for the kernel
- **Node.js** >= 20 LTS for libternary, client, and server
- **Docker** for integration tests and local deployment
- **Make** (the repo uses Makefiles extensively)
- **PostgreSQL** (optional for full-stack testing; Drizzle ORM manages the schema)

### Quick Start

```bash
# Clone
git clone https://github.com/SigmaWolf-8/Ternary.git
cd Ternary

# Install Node dependencies
npm install

# Build the Rust kernel
cd src/kernel && cargo build --release && cd ../..

# Build libternary (TypeScript)
cd libternary && npm install && npm run build && cd ..

# Run the theory validation suite
make test-theory

# Start the dev server
npm run dev
```

### Environment Variables

Copy `.env.example` to `.env` and configure your database connection (PostgreSQL via Drizzle). The `DATABASE_URL` must be set for the server to start.

### Running Self-Tests

PlenumNET includes self-test endpoints that validate key claims:

```bash
# Density benchmark — validates the 59% compression claim
# Runs across 4 sample sizes, returns PASS/FAIL
curl http://localhost:5000/api/salvi/ternary/density-benchmark

# Timing self-test — validates femtosecond precision
# Runs 1000 samples, reports resolution and jitter
curl http://localhost:5000/api/salvi/timing/self-test

# VM conformance — validates GF(3) arithmetic tables and ISA spec
curl http://localhost:5000/api/salvi/vm/conformance
```

All must return `PASS` before any PR is merged.

---

## Contribution Workflow

1. **Read the relevant ADRs.** If your change touches the opcode set, read ADR-001. If it touches crypto, read ADR-003. If it touches trit arithmetic, read ADR-002. This is not optional.

2. **Fork and branch** from `main`. Branch naming convention: `<type>/<short-description>`
   - `feat/lattice-ntt-optimisation`
   - `fix/gf3-inverse-edge-case`
   - `docs/adr-004-timing-epoch-format`

3. **Write the code.** Follow the coding standards below.

4. **Run the full test suite locally**, including theory validation:
   ```bash
   cargo test              # Rust kernel tests
   npm test                # libternary + integration tests
   make test-theory        # Theory validation suite
   ```

5. **Run self-tests** — density, timing, and VM conformance must all return PASS.

6. **Run `verifyTau()`** — confirm τ³ = τ² + τ + 1 still holds (catches accidental constant corruption).

7. **Open a PR.** The CI pipeline will run `verifyTau()`, GF(3) algebraic property tests, and density/timing self-tests automatically. PRs that fail theory validation will not be reviewed.

8. **Respond to review.** Expect questions about mathematical correctness, not just code style.

---

## Coding Standards

### Constants

- Import all τ-derived values from the `tribonacci` module
- Never hardcode τ, τ⁷, 2757, or any derived value as a raw number
- When adding a new constant derived from τ, add it to `DERIVED_CONSTANTS` with a doc comment explaining the derivation
- Example: `const ROUNDS: u32 = 13; // D=13, dimensional constant from T(7)=13`

### Arithmetic

- Always use `toGF3()` / `fromGF3()` for balanced ternary <-> GF(3) conversion
- Never write `a + 1` or `a - 1` to convert between representations
- All ternary operations must be constant-time (no data-dependent branches on trit values in crypto paths)

### Rust (`libternary` / Kernel)

- All GF(3) operations must be constant-time. No branching on secret trit values.
- Every public function must have a `#[doc]` comment explaining the mathematical operation it performs.
- Use `#[cfg(test)]` modules for algebraic property tests (associativity, commutativity, distributivity, inverse existence).
- No `unwrap()` in production paths. All errors must propagate or trigger a deterministic halt.

### TypeScript (Client / Server / Shared)

- Strict TypeScript (`"strict": true` in `tsconfig.json`).
- Trit values are always typed as `0 | 1 | 2` (Representation B) or `-1 | 0 | 1` (Representation A), never plain `number`. Use the shared type definitions.
- API response types must reflect GF(3) semantics in their documentation comments.

### Comments

- When using a theory-derived constant, include a one-line comment citing the derivation
- Don't explain the physics in detail — reference the relevant section of the whitepaper

### Security

- The guardian phase checksum (`tribonacciHash`) is a **non-cryptographic** checksum for tamper detection
- For cryptographic integrity, use the CNSA 2.0 algorithms in the kernel crypto module (HMAC-SHA-384, etc.)
- Never expose or log encryption keys, seeds, or intermediate cipher states

### General

- Commits must be signed (`git commit -S`).
- Each commit message must reference the layer it touches: `[libternary]`, `[client]`, `[server]`, `[contracts]`, `[infra]`, `[docs]`.
- If your change alters the behaviour of any of the 160 opcodes, you must update the corresponding `verifyTau()` test case.

---

## Testing Requirements

### Theory Validation (Mandatory on Every PR)

The CI pipeline runs three theory-validation stages:

1. **`verifyTau()` Self-Test** — Executes the τ polynomial identity check across all 160 opcodes. A single failure halts the pipeline.

2. **GF(3) Algebraic Property Tests** — Exhaustive verification over all 3 x 3 = 9 input pairs for each binary operation:
   - Associativity: `(a + b) + c = a + (b + c)` and `(a * b) * c = a * (b * c)`
   - Commutativity: `a + b = b + a` and `a * b = b * a`
   - Distributivity: `a * (b + c) = (a * b) + (a * c)`
   - Identity: `a + 0 = a` and `a * 1 = a`
   - Inverse: `a + (-a) = 0` and `a * a^-1 = 1` for a != 0

3. **Density/Timing Self-Tests** — Validates that the 13-component state vector remains consistent after a sequence of timing opcodes, and that HPTP synchronisation tolerances are met within the test harness.

### Unit Tests

- Every new function needs unit tests
- GF(3) operations: test the full 3x3 table (9 input pairs)
- Constants: verify against the `tribonacci` module values, not hardcoded expectations
- Self-test endpoints must pass after your changes
- Rust: `cargo test` in `/src/kernel`
- TypeScript: framework-standard test runner as configured in `package.json`

### Integration Tests

- Located in `/tests/integration`
- Require Docker (the test harness spins up a database and the full server stack)

---

## What to Work On

### Good First Issues

- Add missing doc comments to public API functions
- Expand test coverage for edge cases (empty inputs, maximum trit arrays)
- Fix clippy warnings in the Rust kernel

### Intermediate

- Add property-based tests (Rust `proptest`, TypeScript `fast-check`) for GF(3) algebraic properties
- Benchmark suite with regression tracking using Rust's `criterion` crate
- WASM compilation target for the ternary kernel
- Optimize NTT implementations in lattice-based crypto modules

### Advanced

- Fuzz testing harness for the binary-ternary gateway boundary
- Formal specification (YAML/JSON) for the 160-opcode ISA v2.0 instruction set
- Error budget tracking for HPTP timing drift over extended periods
- New opcode proposals (consumes a reserved slot — requires ADR and justification)

---

## Submitting Changes

### PR Process

1. **Fork and branch** from `main`
2. **Run all tests** — `cargo test && npm test`
3. **Run self-tests** — density, timing, and VM conformance must return `PASS`
4. **Run `verifyTau()`** — confirm τ³ = τ² + τ + 1 still holds
5. **Write a clear PR description** explaining what changed and why
6. **Reference theory connections** — if your change involves τ-derived values, explain which constant and how it's derived

### PR Checklist

- [ ] All existing tests pass
- [ ] New code has unit tests
- [ ] Self-test endpoints return PASS (`density-benchmark`, `self-test`, `vm/conformance`)
- [ ] `verifyTau()` passes
- [ ] Constants imported from `tribonacci` module (no hardcoded magic numbers)
- [ ] GF(3) conversions use `toGF3()` / `fromGF3()` (no `a + 1` shortcuts)
- [ ] Doc comments on new public functions
- [ ] No secrets, keys, or sensitive data in committed code
- [ ] Commit messages reference the layer: `[libternary]`, `[client]`, `[server]`, etc.
- [ ] ADR written if introducing a new design choice that would surprise a future contributor

---

## Architecture Decision Records

All significant design decisions are documented in `/docs/adr/`. See the [ADR index](docs/adr/README.md) for the current set.

**If your contribution introduces a new design choice that would surprise a future contributor**, write an ADR. Examples:
- Adding a new opcode (consumes a reserved slot — requires justification)
- Changing the hash function used in Lamport signatures
- Altering the 13-component state vector structure
- Modifying HPTP timing tolerances

---

## Further Reading

- **Project 13D Site**: Full theory presentation with interactive visualizations
- **Whitepaper V9.8Rf**: Complete mathematical derivations and 57+ predictions
- **PlenumNET Live Platform**: https://PlenumNET.replit.app
- **Source Code**: https://github.com/SigmaWolf-8/Ternary

---

## Questions?

Open an issue tagged `question` or `theory-discussion`. There is no such thing as a dumb question about finite field arithmetic or torsion geometry — only unexplored understanding.

*Cosi sia.*

---

## License

PlenumNET is licensed under GPL-3.0. All contributions are subject to the same license.

All Rights Reserved and Preserved | Capomastro Holdings Ltd 2026
