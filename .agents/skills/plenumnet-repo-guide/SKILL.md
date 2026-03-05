---
name: plenumnet-repo-guide
description: Complete A-Z structural guide to the PlenumNET / Salvi Framework repository. Covers ternary mathematics (base-3, pi=14, 364-degree circle, 13x28 calendar), first-position derivation rules, TDNS ontological addressing, Rep A/B/C trit encodings, Tribonacci constants, Saturnian geometry, Inter-Cube infrastructure, quantum ternary modules, XPlenum RISC-V extension, and all codebase conventions. Use when working on any PlenumNET feature, reviewing architecture, or onboarding.
---

# PlenumNET Repository — Complete A-Z Guide

Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
Patent(s) Pending — All Rights Reserved — Applied Physics Division
Author: RSalvi@Salvigroup.com | OWNER: SigmaWolf-8 | REPO: Ternary

---

## 1. Foundation Mathematics

### 1.1 Ternary Base-3 System

PlenumNET operates entirely in base-3 (ternary). Three equivalent digit encodings exist:

| Repr | Digits | Domain | Translation from B |
|------|--------|--------|--------------------|
| **A** (Balanced) | `{-1, 0, +1}` | Signed arithmetic, negation | Subtract 1 (with carry) |
| **B** (Standard) | `{0, 1, 2}` | Recurrence, analysis | Identity (internal) |
| **C** (Bijective) | `{1, 2, 3}` | Wire format, crypto, TDNS | Add 1 (with carry) |

**CRITICAL RULE**: TDNS addresses use **Rep C** exclusively. Trit values are `{1, 2, 3}` — **zero is excluded**. Any address containing a zero trit is provably forged. This zero-exclusion property is the structural basis for forgery detection.

Internal computation uses Rep B. Conversion happens at module boundaries via `to_repr_a()`, `to_repr_c()`, etc. (See `libternary/src/lib.rs`.)

### 1.2 The Ternary Circle (364 Degrees)

A full circle is **364 degrees**, not 360. This is not arbitrary — it is derived:

```
364 = 111111₃  (base-3 repunit of six 1's)
364 = (3⁶ - 1) / 2
```

**Core constants** (all exact integers, no floating point):

| Symbol | Value | Derivation |
|--------|-------|------------|
| Full circle | **364°** | `111111₃ = (3⁶ − 1)/2` |
| π | **14** | Circumference / diameter (exact) |
| 2π | **28** | Full circle in radians |
| 1 radian | **13°** | `364/28 = 13 = 111₃` = T₇ (7th Tribonacci number) |

**Binding equation**: `C = πd = 14d`, so `C/r = 28`. Full circle = 28 radians = 364°.

Source files:
- `shared/ternary-circle.ts` — TypeScript constants, Z₂₈ cyclic group, conversion functions
- `shared/tribonacci-constants.ts` — `TERNARY_CIRCLE` object with verification
- `libternary/src/ternary_circle.rs` — Rust implementation

### 1.3 The 13-Moon Calendar (13 × 28 = 364)

The calendar divides the year into **13 moons of 28 days each** = 364 days, plus one intercalary **Day Out of Time (DOT)** on November 11.

| Phase | Moons | Period |
|-------|-------|--------|
| Pre-DOT (waxing) | Moons 1–8 | Apr 1 → Nov 10 (8 × 28 = 224 days) |
| **DOT** | Day Out of Time | Nov 11 (1 day, Fibonacci split point) |
| Post-DOT (waning) | Moons 9–13 | Nov 12 → Mar 31 (5 × 28 = 140 days) |

**Total**: 224 + 1 + 140 = **365 days** (366 in leap years via extra intercalary day).

The DOT splits the year at the **Fibonacci point**: 8 moons before, 5 after (8 and 5 are consecutive Fibonacci numbers). Nov 11 is the pivot.

Source: `client/src/pages/thirteen-moon.tsx`

### 1.4 Tribonacci Constant (τ)

The Tribonacci constant τ replaces the golden ratio φ throughout the framework:

```
τ ≈ 1.83928675521416
τ³ = τ² + τ + 1  (defining polynomial)
```

Key Tribonacci sequence values: `0, 0, 1, 1, 2, 4, 7, 13, 24, 44, 81, 149, 274, 504, 927, ...`

**Critical alignment**: T₇ = **13** (the 7th Tribonacci number) = 1 ternary radian = `111₃`.

Source: `shared/tribonacci-constants.ts` — `TAU`, `TAU_POWERS`, `TRIBONACCI_SEQUENCE`

### 1.5 Saturnian Magic Square

The 3×3 circulant magic square:

```
| 111 |  14 | 208 |
| 208 | 111 |  14 |
|  14 | 208 | 111 |
```

Every row, column, and diagonal sums to **333** (magic constant). Exact integer alignments:

- RADIUS_COSMIC = 208/16 = **13** = T₇
- PI_ESOTERIC = **14** = T₇ + T₃ = 13 + 1
- LUNAR_SOLAR_HARMONIC = 2 × 14 = **28** = Z₂₈ cyclic order
- COSMIC_CIRCUMFERENCE = 28 × 13 = **364** = full ternary circle

Source: `shared/saturnian-blueprint.ts`

### 1.6 GF(3) — Galois Field of Order 3

GF(3) is the finite field with elements {0, 1, 2} under modular arithmetic:
- Addition: `(a + b) mod 3`
- Multiplication: `(a × b) mod 3`
- Negation: `(3 - a) mod 3`

In Rep C (wire/TDNS), the lift is: `trit = gf3 + 1`, mapping GF(3) {0,1,2} → Rep C {1,2,3}.

GF(3) arithmetic is the foundation of:
- Address derivation (quantitative rules)
- Hypercube routing (next-hop = single trit flip)
- Metatronic Cube geometry
- Phase encryption

---

## 2. TDNS v2.3 — Ternary Domain Name System

### 2.1 Overview

TDNS replaces DNS, BGP, PKI, IGMP/PIM, and PTP within the managed fabric. Every entity on PlenumNET occupies exactly one point in a **27-dimensional ternary hypercube** with 3²⁷ = 7,625,597,484,987 (7.63 trillion) possible addresses.

Current version: **2.3.2**
Spec: `salvi_docs/specs/TDNS-v2.3-SPECIFICATION.md`

### 2.2 The 27-Dimensional Ontological Schema

27 dimensions organized into 7 categories (WHO · WHAT · WHERE · WHEN · WHY · HOW · PEACE):

| Category | Prefix | Trits | Dims | Root Question |
|----------|--------|-------|------|---------------|
| WHO | WO | 1–4 | 4 | Who is behind it? |
| WHAT | WA | 5–8 | 4 | What is it? |
| WHERE | WR | 9–12 | 4 | Where can I find it? |
| WHEN | WN | 13–16 | 4 | When does it operate? |
| WHY | WY | 17–20 | 4 | Why does it exist? |
| HOW | HO | 21–24 | 4 | How does it work? |
| PEACE | PE | 25–27 | 3 | Can I sleep at night? |

Each dimension has 3 possible values (Rep C: 1, 2, 3) with human-readable labels. Example for dim 1 (WHO: What kind?): 1=Personal, 2=Corporate, 3=Governance.

Full schema: `services/tdns-v2/src/schema.rs` — `SCHEMA` array (27 entries)

### 2.3 Address Formats

**Category format** (human-readable): `WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313`
**Canonical format**: 9 dot-separated groups
**Wire format**: 27 trits × 2 bits = 54 bits, packed into 7 bytes (56 bits, 2 padding bits MSB)

Wire encoding per trit: `1=0b01, 2=0b10, 3=0b11, 0b00=reserved/invalid`

### 2.4 First-Position Derivation Rules

**THE UNIVERSAL FORMULA** (the only quantitative derivation function):

```
gf3 = min(floor(3k / N), 2)     where k = signals fired, N = total signals
trit = gf3 + 1                   lift from GF(3) {0,1,2} to Rep C {1,2,3}
```

Boundaries between trit values fall at exactly N/3 and 2N/3 — derived from the definition of ternary quantization, not from empirical tuning. No arbitrary thresholds. No tuning parameters.

**Confidence digit** (§4.1):
```
p = k / N
δ = min(|p - 1/3|, |p - 2/3|)
C = min(floor(27δ) + 1, 9)
```
Range 1–9. The number **27** (number of dimensions) determines confidence scaling — the system's own structure defines it.

**Two derivation types**:
- **CATEGORICAL** (15 rules): Scanner produces a pattern string → direct mapping to trit. Confidence always 9.
- **QUANTITATIVE** (12 rules): Scanner counts binary signals (k out of N) → `project_to_gf3` maps to trit.

15 + 12 = 27 rules = 27 dimensions. Source: `services/tdns-v2/src/derive.rs`

### 2.5 HPTP Mandatory Rule

If trits 15 AND 16 are both **3** (Live data AND Real-time), the address is **HPTP-mandatory**: femtosecond timing verification is required for all packets to/from this entity.

### 2.6 Scan Hash Type Tags

| Tag | Meaning |
|-----|---------|
| `0x01` | SHA-256 |
| `0x02` | BLAKE3 |
| `0x03` | TL-DSA signature |
| `0x04` | Composite (multi-hash) |

### 2.7 TDNS Module Map

All modules live under `services/tdns-v2/src/`:

| File | Purpose |
|------|---------|
| `trit.rs` | Trit type {1,2,3}, wire encoding, GF(3) ops |
| `addr.rs` | CubeAddr (27-trit address), Hamming distance, wire pack/unpack |
| `schema.rs` | 27 dimension definitions, category layout, describe() |
| `derive.rs` | 27 derivation rules, project_to_gf3, confidence_digit |
| `scan.rs` | ScanMeasurement, RawValue, Confidence (serde transparent) |
| `scanner.rs` | Live HTTP/DNS/TLS scanner (27 probe measurements via ureq) |
| `crs.rs` | CRS Registry Service — register, resolve, verify, rescan, drift log |
| `trn.rs` | TRN (Ternary Resource Name) records |
| `fts.rs` | FTS (Fault Tolerance Service) — heartbeat, suspect/dead detection |
| `glb.rs` | GLB (Geometric Load Balancer) — forwarding, redirect, dead set |
| `overlay.rs` | CON (Cube Overlay Network) — PQ-encrypted tunnels, BLAKE3 keys |
| `subcube.rs` | SubCube multicast addressing |
| `routing.rs` | NeighborMap, greedy geometric routing |
| `bridge.rs` | Metatronic Bridge — .plm→TDNS / legacy DNS resolution |
| `wire.rs` | Packet wire protocol (version 0x23), integrity checks |
| `api.rs` | 11 HTTP API endpoints, ApiRouter |
| `lib.rs` | Crate root, module re-exports |

Binaries:
- `src/bin/tdns_scan.rs` — CLI: scan, compare, describe commands
- `src/bin/tdns_server.rs` — HTTP server (port 3927 default)

Tests:
- `tests/e2e.rs` — 9 end-to-end tests (full pipeline)
- `tests/integration.rs` — 12 scenario integration tests

---

## 3. The 13-Dimensional Hypercube (Inter-Cube Network)

### 3.1 Geometry

The Inter-Cube mesh network uses a **13-dimensional ternary hypercube**:

| Property | Value |
|----------|-------|
| Dimensions | 13 |
| Vertices | 3¹³ = 1,594,323 |
| Neighbors per node | 2 × 13 = 26 |
| Max diameter (hops) | 13 |
| Routing tables | **0** (geometry = protocol) |

Next-hop computation: a single trit flip in the dimension with the greatest distance reduction. No BGP, no OSPF, no routing tables — pure GF(3) arithmetic.

### 3.2 Inter-Cube Services (4)

1. **GLB** (Geometric Load Balancer) — point forwarding, sub-cube multicast fan-out, dead-node avoidance, redirects
2. **CON** (Cube Overlay Network) — PQ-encrypted tunnels with BLAKE3-derived keys, key rotation, traffic accounting
3. **CRS** (Cube Registration Service) — entity registration, neighbor maps, dimension density
4. **FTS** (Fault Tolerance Service) — heartbeat-based failure detection, suspect/dead/recovered states

Source: `services/inter-cube/` (Rust crate) and `services/tdns-v2/` (integrated)

### 3.3 Metatronic Cube

The 13D cube viewed through Saturnian geometry. Three shells of 3¹² = 531,441 vertices each, with depth axis at position 13 (= T₇ = 1 radian).

Source: `shared/metatronic-cube.ts` — `METATRONIC_DIM`, `METATRONIC_VERTICES`, `SHELL_VERTICES`

---

## 4. Repository Structure

### 4.1 Top-Level Layout

```
/                           Root (Express.js + Vite full-stack app)
├── client/                 React frontend (TypeScript, Tailwind, shadcn/ui, Wouter)
│   └── src/
│       ├── pages/          26 pages (landing, about, contact, docs, admin, ...)
│       └── components/     Shared components (sidebar, footer, nav, ...)
├── server/                 Express.js backend (TypeScript)
│   ├── routes.ts           API routes
│   ├── storage.ts          IStorage interface + DatabaseStorage
│   ├── db.ts               Drizzle ORM connection
│   └── ...                 Config, crypto, compression, ephemeris, etc.
├── shared/                 Shared TypeScript modules (constants, schema, math)
│   ├── constants.ts        PLATFORM object (single source of truth)
│   ├── schema.ts           Drizzle DB schema + Zod insert schemas
│   ├── ternary-circle.ts   364° circle, Z₂₈, conversion functions
│   ├── tribonacci-constants.ts  τ, Tribonacci sequence, VM constants
│   ├── saturnian-blueprint.ts   Saturnian magic square, exact alignments
│   ├── metatronic-cube.ts  13D ternary cube geometry
│   ├── agent-array.ts      28-dimension AI agent orchestration
│   ├── qutrit-basics.ts    Qutrit (3-level quantum) operations
│   ├── qudit-basics.ts     Generalized qudit (d-level) operations
│   ├── complex-utils.ts    Complex number arithmetic for quantum sim
│   ├── lagrangian-*.ts     Lagrangian qutrit/ternary evolution
│   ├── hamiltonian-constraints.ts  Hamiltonian constraint system
│   ├── noether-symmetries-utils.ts Noether symmetry analysis
│   ├── qutrit-fault-tolerance.ts   Qutrit error correction
│   ├── saturnian-matrix-utils.ts   Matrix operations on Saturnian square
│   └── tribonacci-variational.ts   Variational Tribonacci calculus
├── services/               Microservices (Rust + TypeScript)
│   ├── tdns-v2/            TDNS v2.3.2 (Rust crate, 17 modules + 2 binaries)
│   ├── inter-cube/         Inter-Cube Infrastructure (Rust crate)
│   ├── blockchain/         Blockchain services
│   │   ├── hedera-service/ Hedera HCS witnessing
│   │   ├── xrpl-service/   XRP Ledger integration
│   │   └── algorand-service/ Algorand integration
│   ├── payment-listener/   Payment processing (Stripe, Interac, crypto)
│   ├── sfk-core-api/       SFK Operations Pipeline
│   ├── pqti-service/       Post-Quantum TLS Inspection
│   ├── timing/             Timing services
│   │   ├── femtosecond-service/  HPTP timing
│   │   └── certification-service/ RFC 3161 TSA
│   └── tonal-field/        Tonal Diffusion System (FM timing)
├── libternary/             Core ternary library (Rust crate)
│   └── src/
│       ├── lib.rs          Rep A/B/C conversions, ternary numeration
│       ├── tribonacci.rs   Tribonacci sequence generation
│       ├── ternary_circle.rs  364° circle (Rust)
│       └── borromean.rs    Borromean topology primitives
├── ternary-math/           Additional ternary math modules
├── XPlenum/                RISC-V hardware extension
│   ├── rtl/                Verilog RTL (AES, PQC, trit unit, cap unit, ...)
│   ├── tb/                 Testbenches
│   ├── sim/                Simulation
│   ├── synth/              Synthesis
│   └── docs/               Hardware documentation
├── cli/                    CLI tools
│   └── plenum-stamp/       RFC 3161 stamp CLI
├── contracts/              Smart contracts (Algorand, oracle bridge)
├── kong/                   Kong Konnect API gateway config
├── salvi_docs/             Documentation
│   ├── specs/              Specifications (TDNS v2.3, etc.)
│   ├── modules/            Module documentation
│   └── tutorials/          Tutorials
├── docs/                   Architecture docs, ADRs, security, legal
├── deployments/            Deployment configs (DO NOT MODIFY)
└── keys/                   Key material
```

### 4.2 Frontend Pages

| Page | Route | Purpose |
|------|-------|---------|
| `landing.tsx` | `/` | Main marketing landing page |
| `about.tsx` | `/about` | About PlenumNET |
| `contact.tsx` | `/contact` | Contact form |
| `docs.tsx` | `/docs` | Documentation hub |
| `ternarydb.tsx` | `/ternarydb` | PlenumDB product page |
| `compression.tsx` | `/compression` | Compression demo |
| `whitepaper.tsx` | `/whitepaper` | Whitepaper viewer |
| `hptp-demo.tsx` | `/hptp-demo` | HPTP Timing API demo |
| `api-demo.tsx` | `/api-demo` | API demo page |
| `api-keys.tsx` | `/api-keys` | API key management |
| `vm-demo.tsx` | `/vm-demo` | Ternary VM terminal |
| `quantum-sim.tsx` | `/quantum-sim` | Quantum ternary simulator |
| `compliance.tsx` | `/compliance` | CNSA 2.0 compliance |
| `tsa.tsx` | `/tsa` | TSA Time-Stamping Authority |
| `thirteen-moon.tsx` | `/thirteen-moon` | 13-Moon Calendar |
| `tribonacci-28ds.tsx` | `/tribonacci-28ds` | Tribonacci 28D system |
| `calendar.tsx` | `/calendar` | Calendar view |
| `github-manager.tsx` | `/github-manager` | GitHub repository manager |
| `kong-konnect.tsx` | `/kong-konnect` | Kong Konnect integration |
| `admin.tsx` | `/admin` | Admin dashboard |
| `agent-array.tsx` | `/agent-array` | 28-Agent Array UI |
| `distribution.tsx` | `/distribution` | Distribution page |
| `fpga-benchmarks.tsx` | `/fpga-benchmarks` | FPGA benchmark results |
| `isa-security-paper.tsx` | `/isa-security-paper` | ISA security paper |
| `legal.tsx` | `/legal` | Legal pages (terms, privacy, security) |
| `not-found.tsx` | `*` | 404 page |

---

## 5. PLATFORM Constants (Single Source of Truth)

All numeric constants live in `shared/constants.ts` → `PLATFORM` object. **Never hardcode numbers in pages or components** — always import from `PLATFORM`.

```typescript
export const PLATFORM = {
  VM_OPCODES: 176,
  VM_ISA_VERSION: "v2.1",
  VM_REGISTERS: 27,
  API_ENDPOINTS: 279,
  API_SERVICES: 23,
  KERNEL_LOC: "47,000+",
  KERNEL_SUBSYSTEMS: 14,
  TESTS_PASSING: "1,901",
  DENSITY_ADVANTAGE: 59,
  PLATFORM_VERSION: "2.3.2",

  // Benchmarks
  BENCH_TL_DSA_44_US: "1,220",
  BENCH_TL_DSA_65_US: "1,700",
  BENCH_TL_DSA_87_US: "2,470",
  BENCH_TL_DSA_87_SPEEDUP: "5.9",
  BENCH_KANI_PROOFS: 50,

  // Inter-Cube
  HYPERCUBE_DIMENSIONS: 13,
  HYPERCUBE_VERTICES: "1,594,323",
  TDNS_ADDRESS_SPACE: "7.63 trillion",
  TDNS_TRITS: 27,
  HYPERCUBE_NEIGHBORS: 26,
  INTER_CUBE_TESTS: 97,
  INTER_CUBE_ENDPOINTS: 11,
  INTER_CUBE_SERVICES: 4,
} as const;
```

---

## 6. Quantum Ternary Modules

Five shared modules provide classical simulation of quantum ternary operations:

| Module | Purpose |
|--------|---------|
| `qutrit-basics.ts` | Qutrit (d=3) states, Gell-Mann generators, SU(3) unitaries, SUFT-coupled phase gates |
| `qudit-basics.ts` | Generalized qudit (d≥2) states, shift/clock operators, error simulation |
| `lagrangian-qutrit-utils.ts` | Lagrangian evolution for qutrit systems |
| `lagrangian-ternary-utils.ts` | Ternary-specific Lagrangian mechanics |
| `qutrit-fault-tolerance.ts` | Qutrit error correction codes |

Supporting: `complex-utils.ts` (complex arithmetic), `hamiltonian-constraints.ts`, `noether-symmetries-utils.ts`

---

## 7. 28-Dimension Agent Array

Maps Z₂₈ cyclic positions to 28 parallel AI agents:

- **Scheduling**: `(position × 13) mod 28` visits all 28 positions exactly once (gcd(13,28) = 1)
- **Walk**: 0 → 13 → 26 → 11 → 24 → 9 → 22 → 7 → 20 → 5 → 18 → 3 → 16 → 1 → ...
- **Result**: 13 × 28 = 364 = `111111₃` (ternary palindrome)
- **Convolution kernel**: `[13, 24, 44]` (three consecutive Tribonacci numbers: T₇, T₈, T₉)

Source: `shared/agent-array.ts`

---

## 8. XPlenum RISC-V Hardware Extension

Custom RISC-V extension integrated with CVA6 for hardware-accelerated ternary operations:
- 21 custom instructions
- 12 custom CSRs
- Ternary security operations, PQC acceleration, compliance

Verilog RTL modules in `XPlenum/rtl/`:
- `xplenum_top.v` / `xplenum_top_v2.v` — Top-level integration
- `xplenum_trit_unit.v` — Ternary arithmetic unit
- `xplenum_aes256_core.v` — AES-256 core
- `xplenum_pqc_unit.v` — Post-quantum crypto accelerator
- `xplenum_cap_unit.v` — Capability-based security unit
- `xplenum_ctr_drbg.v` — NIST CTR_DRBG random number generator
- `xplenum_mask_unit.v` — Masking countermeasures
- `xplenum_tamper_response.v` — Tamper detection/response
- `xplenum_domain_unit.v` — Domain isolation
- `xplenum_dom_gadgets.v` — Domain gadget library

---

## 9. Security Architecture

### 9.1 6-Phase Capability-Based Security

Authorization uses unforgeable, self-contained, bearer-verified capability tokens signed with TL-DSA:
1. Typed constraint registry
2. HPTP-bound expiration
3. HMAC-chained delegation
4. Hardware-bound capabilities
5. RFC 3161 capability certificates
6. Inter-service capability mesh

### 9.2 Security Middleware Stack

- 4-tier rate limiting
- CORS + Helmet.js security headers
- AES-256-GCM token encryption
- Null-byte stripping + double URL-decode protection
- `execFile()`-only subprocess execution (no shell injection)

### 9.3 Post-Quantum Cryptography

- **TL-DSA** (Ternary Lattice DSA): Security levels 44/65/87
  - Uses integer NTT for polynomial multiplication
  - AVX2 vectorization for performance
- **TL-KEM**: Ternary lattice key encapsulation
- **CNSA 2.0 compliance**: Full algorithm coverage
- **Phase encryption**: Ternary-native encryption scheme

---

## 10. RFC 3161 Time-Stamping Authority (TSA)

Digital notary service providing cryptographic proof-of-existence timestamps:
- 4 TSA policies
- Merkle tamper-evident audit log
- Dual-signature: RSA-4096 + TL-DSA-87
- HPTP timing integration
- ASN.1 wire protocol

CLI: `cli/plenum-stamp/` — `plenum-stamp sign` and `plenum-stamp verify`

---

## 11. Blockchain Witnessing

### Hedera HCS
Submits cryptographic witness hashes to an HCS topic for immutable, ordered, timestamped proof of PlenumNET operations.

### SFK Operations Pipeline
Manages operation lifecycle: initialization → ternary_processing → witnessing → finalization.
Fortified-tier operations submit SHA-256 result hashes to Hedera HCS.

---

## 12. Key Rules and Conventions

### 12.1 Development Rules

1. **NEVER modify `deployments/` folder** — user-enforced constraint
2. **NEVER hardcode numbers** — use `PLATFORM` constants from `shared/constants.ts`
3. **All source files include copyright headers** (Capomastro Holdings Ltd.)
4. **Rep C everywhere in TDNS** — trit values {1,2,3}, never 0
5. **First-position derivation** — universal formula `gf3 = min(floor(3k/N), 2)`, no tuning parameters
6. **HPTP mandatory** = trits 15 AND 16 both equal 3

### 12.2 Frontend Conventions

- React + TypeScript + Tailwind CSS + shadcn/ui + Wouter + Framer Motion
- Light/dark mode support
- `data-testid` attributes on all interactive and meaningful elements
- All data from `PLATFORM` constants — no magic numbers in JSX
- TanStack Query v5 (object form only) for data fetching
- `@assets/` import prefix for attached assets

### 12.3 Backend Conventions

- Express.js + Drizzle ORM + PostgreSQL
- IStorage interface pattern for all CRUD
- Zod validation on all request bodies
- API versioning, tiered rate limiting

### 12.4 Rust Conventions

- `edition = "2021"` for TDNS v2.3.2
- Pinned deps where stability matters (`blake3 = "=1.5.4"`, `ureq = "=2.7.1"`)
- `thiserror` for error types
- `serde` with `derive` feature for serialization
- All modules re-exported from `lib.rs`

### 12.5 GitHub Push Convention

Files are pushed to `SigmaWolf-8/Ternary@main` using the GitHub Contents API via `GITHUB_TOKEN` environment secret. Push via bash + curl, NOT the code_execution sandbox.

---

## 13. Reference Addresses (Spec Fixtures)

| Entity | Address (Category) | HPTP |
|--------|-------------------|------|
| Google | `WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313` | No |
| PPTPro (Capomastro) | `WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332` | **Yes** |
| Nonna's Cucina (blog) | Derived from blog_measurements() | No |

Google trit 26 (dim 26, 0-indexed 25): `Numeric(0.0)` = no trackers → trit 3 (None/Clean).

---

## 14. Key Mathematical Identities (Quick Reference)

```
3²⁷ = 7,625,597,484,987          (TDNS address space)
3¹³ = 1,594,323                   (13D hypercube vertices)
(3⁶ - 1) / 2 = 364               (full ternary circle, base-3 repunit)
364 / 28 = 13                     (1 radian = T₇)
13 × 28 = 364                     (13 moons × 28 days)
gcd(13, 28) = 1                   (coprime — enables complete Z₂₈ walk)
τ³ = τ² + τ + 1                   (Tribonacci defining polynomial)
T₇ = 13                          (7th Tribonacci number)
111₃ = 13                        (base-3 repunit)
111111₃ = 364                    (six-digit base-3 repunit)
log₂(3) ≈ 1.585                  (59% density advantage of ternary over binary)
```
