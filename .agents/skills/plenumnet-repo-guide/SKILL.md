---
name: plenumnet-repo-guide
description: Complete A-Z structural guide to the PlenumNET / Salvi Framework repository (SigmaWolf-8/Ternary, 1,252+ commits, 80/80 milestones). Covers ternary mathematics (base-3, pi=14, 364-degree circle, 13x28 calendar), first-position derivation rules, TDNS v2.5 ontological addressing (19 Rust modules), Rep A/B/C trit encodings, Tribonacci constants, Saturnian geometry, Inter-Cube infrastructure (8-constraint system, 20.7M PQ tunnels, d! structural resilience, Cubes-of-Cubes scaling, Sybil resistance, topology-derived TLSponge-385 keys), quantum ternary modules, XPlenum RISC-V extension, Rust kernel subsystems (176-opcode ISA v2.1), bare-metal validation (Kani/MIRI), TL-DSA/TL-KEM post-quantum crypto (34 crypto modules), Kong Konnect gateway (33 services, 293 endpoints), PlenumDB, SignHere e-signature integration, SFK Operations Pipeline, kernel sponge (729-trit, 9 rounds, 385-bit PQ security) for key derivation and signing, T-AE-MAC dual-phase authenticated encryption (bulk rate 486, trit-native MAC, IND-CPA + INT-CTXT, auto-gated rayon parallelism, v1/v2/v3 version dispatch), TIS-27 wire integrity sponge (54-trit, 4 rounds, 43-bit), TIS-81 benchmark variant (243-trit, 4 rounds, 257-bit), 42 calendar systems, and all codebase conventions. Use this skill when working on ANY PlenumNET feature, reviewing architecture, onboarding, writing code that touches the Salvi Framework, modifying the Ternary repo, debugging crypto or TDNS issues, building frontend pages, or discussing any Capomastro Holdings technical product. Always consult this skill before making changes — the invariants are load-bearing and violations break mathematical consistency across the entire framework.
---

# PlenumNET Repository — Complete A-Z Guide

Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
Patent(s) Pending — All Rights Reserved — Applied Physics Division
Author: RSalvi@Salvigroup.com | OWNER: SigmaWolf-8 | REPO: Ternary
Verified against live repo: March 2026 | Commits: 1,252+ | Milestones: 80/80

---

## ⚠ ARCHITECTURAL INVARIANTS — ABSOLUTE RULES THAT CANNOT BE BROKEN

These rules are load-bearing. Violating any of them breaks the mathematical consistency of the entire framework. They are not suggestions. They are structural axioms.

### INVARIANT 1: The Geometry IS the System

Every component in PlenumNET derives from ternary geometry. The geometry is not a metaphor, not a visualization aid, not decoration. It IS the routing protocol, the addressing scheme, the key derivation, the forgery detection, the calendar, and the timing system. If you remove the geometry, you remove the system.

- **Routing** = Hamming distance in a 13D ternary hypercube (trit flips)
- **Addressing** = 27-trit coordinates in a 27D ontological space
- **Key derivation** = Kernel cryptographic sponge (729-trit balanced ternary, rate=243, capacity=486, 9 rounds, 7-neighbor extended theta at ±1/±7/±13, 385-bit PQ security)
- **Wire integrity** = TIS-27 sponge (54-trit GF(3), rate=27, 4 rounds, stride=13, 7-neighbor extended theta at ±1/±7/±13, 43-bit cryptographic security — proven by wide-trail analysis, DP ≤ 9⁻⁴⁰⁹⁶, TM-2026-008)
- **Forgery detection** = Rep C zero-exclusion property (structural, not bolted on)
- **Calendar** = 13 × 28 = 364 = 111111₃ (base-3 repunit)
- **Timing** = Femtosecond precision bound to HPTP-mandatory addresses
- **Agent scheduling** = (position × 13) mod 28 coprime walk on Z₂₈

🚫 **DO NOT** replace geometric routing with routing tables.
🚫 **DO NOT** replace Rep C {1,2,3} with {0,1,2} in TDNS addresses.
🚫 **DO NOT** replace 364° with 360°, π=14 with π≈3.14159, or 13 radians with 57.3°.
🚫 **DO NOT** decouple cryptographic keys from topological adjacency.
🚫 **DO NOT** replace the first-position derivation formula with arbitrary thresholds.

### INVARIANT 2: First-Position Derivation — No Tuning Parameters

The universal derivation formula is mathematical, not empirical:

```
gf3 = min(floor(3k / N), 2)     where k = signals fired, N = total signals
trit = gf3 + 1                   lift from GF(3) {0,1,2} to Rep C {1,2,3}
```

Boundaries between trit values fall at exactly N/3 and 2N/3. These are derived from the definition of ternary quantization. There are NO tuning parameters, NO arbitrary thresholds, NO machine-learned weights. The math determines the boundaries.

🚫 **DO NOT** add configurable thresholds to derivation rules.
🚫 **DO NOT** replace `project_to_gf3` with any other quantization function.
🚫 **DO NOT** add "confidence weighting" that changes derivation boundaries.

### INVARIANT 3: Rep C (Bijective Ternary) — Zero Is Excluded

TDNS addresses, wire formats, and cryptographic operations use Rep C: {1, 2, 3}.
Zero is NEVER a valid trit value. Zero is a sentinel — its presence proves forgery.

| Repr | Digits | Domain |
|------|--------|--------|
| **A** (Balanced) | {-1, 0, +1} | Signed arithmetic, negation |
| **B** (Standard) | {0, 1, 2} | Recurrence, analysis (internal only) |
| **C** (Bijective) | {1, 2, 3} | Wire format, TDNS, crypto — THE external representation |

🚫 **DO NOT** allow trit value 0 in CubeAddr, wire encoding, or any external-facing format.
🚫 **DO NOT** confuse Rep B (internal) with Rep C (external).

### INVARIANT 4: Constants Are Bound, Not Chosen

The ternary circle constants are not independent design choices. They are bound by a single equation:

```
C = πd = 14d  →  C/r = 28  →  Full circle = 28 radians = 364°
```

| Constant | Value | Why |
|----------|-------|-----|
| Full circle | 364° | = 111111₃ = (3⁶ − 1)/2 — the six-digit base-3 repunit |
| π | 14 | = T₇ + T₃ = 13 + 1 — exact integer |
| 2π | 28 | = full circle in radians — Z₂₈ cyclic order |
| 1 radian | 13° | = 111₃ = T₇ (7th Tribonacci number) — exact integer |
| τ | 1.8392867552... | Tribonacci constant: τ³ = τ² + τ + 1 |
| Calendar | 13 × 28 = 364 | 13 moons of 28 days |

🚫 If you change any one of these, you break all of them.

#### Why 13 Is the Radian and 28 Is the Circle Order (Not Interchangeable)

The product 13 × 28 = 364 is symmetric, but the roles are not interchangeable because 13 has properties 28 does not:

| Property | 13 | 28 |
|----------|----|----|
| Base-3 repunit | ✓ (111₃) | ✗ |
| Tribonacci number | ✓ (T₇) | ✗ |
| Prime | ✓ | ✗ (2² × 7) |
| Coprime to TIS-27 state width 54 | ✓ (gcd=1) | ✗ (gcd=2) |
| Produces integer π | ✓ (π = 364/(2×13) = 14) | ✗ (π = 364/(2×28) = 6.5) |

13 is the only factor of 364 that simultaneously satisfies all five constraints. The assignment is not a design choice — it is forced by the intersection of repunit structure, Tribonacci alignment, primality, sponge coprimality, and the integer-π requirement.

### INVARIANT 5: HPTP-Mandatory Is Structural

If trits 15 AND 16 are both 3 (dim 15 = Live, dim 16 = Real-time), the address is HPTP-mandatory. Femtosecond timing verification is REQUIRED for all packets to/from that entity. This is not optional, not configurable, not a policy decision. The address itself dictates timing requirements.

### INVARIANT 6: The Salvi Epoch

April 1, 2025 (2025-04-01T00:00:00Z) = Day Zero. All femtosecond timestamps are 128-bit integers measuring femtoseconds since this epoch. The epoch is a constant. Do not change it.

### INVARIANT 7: TL-DSA Everywhere — No Ed25519, No Simulations

The cryptographic stack uses TL-DSA for ALL digital signatures. Ed25519 was explicitly rejected — not post-quantum safe.

🚫 **DO NOT** use `crypto.generateKeyPairSync('ed25519')` anywhere in the stack.
🚫 **DO NOT** use HMAC-based "simulations" of TL-DSA — call the real Rust implementation.
🚫 **DO NOT** use `crypto.sign(null, ...)` / `crypto.verify(null, ...)` — this is ed25519's API.
🚫 If TypeScript needs TL-DSA, it MUST call through the Rust bridge or WASM path, never a Node.js simulation.

### INVARIANT 8: Raw Binary Integers Must Be Decomposed Before Ternary Operations

**CRITICAL LESSON (March 2026 TL-DSA bug fix):** Never feed raw binary integers directly into ternary sponge operations. All integer values MUST be decomposed into trit representations using `u16_to_trits()` / `u8_to_trits()` before entering any ternary hash, sponge, or derivation function. This was a systemic bug across `tl_dsa.rs` and `ternary_lattice.rs` that invalidated all previously generated keys.

🚫 **DO NOT** pass raw binary integers to ternary sponge absorb functions.
🚫 **DO NOT** skip trit decomposition when bridging binary↔ternary boundaries.

### INVARIANT 9: The deployments/ Folder Is Sacred

🚫 **NEVER modify the `deployments/` folder.** Zero exceptions.

### INVARIANT 10: Sponge Stride Must Be Coprime to State Width

The TIS-27 `tisPi` permutation uses stride **s** on a state of width **W**: `t[(i × s) mod W] = s[i]`. For the permutation to visit every position exactly once (a complete cycle of length W), `gcd(s, W) = 1` is **required**. If the stride shares a factor with the state width, the permutation fragments into disjoint sub-cycles, positions are revisited before all are touched, and diffusion is incomplete — a cryptographic weakness.

**Why stride = 13 specifically:**

```
W = 54 (TIS-27 state width)
Coprimes of 54: {5, 7, 11, 13, 17, 19, 23, 25, 29, 31, 35, 37, 41, 43, 47, 49, 53}
```

Among the 17 valid coprimes, 13 is **uniquely canonical** in the Salvi Framework:

- 13 = T₇ (7th Tribonacci number)
- 13 = 111₃ (three-digit base-3 repunit)
- 13 = 1 ternary radian = 364° / 28
- 13 = number of moons in the Salvi calendar
- 13 = dimension count of the Saturnian Tesseract Metatron Ternary Cube

The stride is not an arbitrary coprime — it is the geometric constant that recurs across the entire framework. The same coprime property (`gcd(13, 28) = 1`) enables the complete Z₂₈ agent scheduling walk.

**General design rule for future sponge variants:** Any sponge construction over a state of width W must choose a stride s such that `gcd(s, W) = 1`. The preferred stride should be the geometrically canonical constant for that context — default to 13 unless the state width is a multiple of 13.

🚫 **DO NOT** choose a stride that shares a factor with the state width.
🚫 **DO NOT** replace stride=13 with an arbitrary coprime — 13 is structurally bound (see INVARIANT 4).

---

## 1. Foundation Mathematics

### 1.1 Ternary Base-3 System

PlenumNET operates entirely in base-3. Key facts:

- `log₂(3) ≈ 1.585` — ternary has a **59% information density advantage** over binary
- Three equivalent digit encodings: Rep A, Rep B, Rep C (see INVARIANT 3)
- Internal computation uses Rep B. External/wire uses Rep C.
- Conversion at module boundaries: `to_repr_a()`, `to_repr_c()`, `from_repr_a()`, `from_repr_c()`, etc.

Source: `libternary/src/lib.rs`

### 1.2 The Ternary Circle (364°)

See INVARIANT 4. The full derivation chain:

```
111111₃ = 1 + 3 + 9 + 27 + 81 + 243 = 364  (six-digit base-3 repunit)
364 / 28 = 13 = 111₃ = T₇               (one radian = 13 degrees)
28 = 2 × 14 = 2π                         (full circle = 28 radians)
13 × 28 = 364                            (ternary calendar identity)
gcd(13, 28) = 1                          (coprime — enables complete Z₂₈ walk)
```

Source: `shared/ternary-circle.ts` — FULL_CIRCLE_DEG, PI_TERNARY, TWO_PI_TERNARY, RADIAN_DEG, Z28 class

#### 1.2.1 Repunit Circle Hierarchy

Base-3 repunits R(n) = (3ⁿ − 1) / 2 define the natural geometric cycle hierarchy. These are **circle-days** (pure geometry), NOT calendar days. Calendar conversion requires DOT insertion: `calendarDays = circleDays + floor(circleDays / 364)`.

| Circle | R(n) | Value | Label | Prime? | Calendar Equiv |
|--------|------|-------|-------|--------|----------------|
| R₃ | 13 | 111₃ | Radian | ✓ | 13 days |
| R₄ | 40 | 1111₃ | Minor Circle | ✗ | 40 days |
| R₅ | 121 | 11111₃ | Quarter Circle | ✗ (11²) | 121 days |
| R₆ | 364 | 111111₃ | Full Circle | ✗ (2²×7×13) | 365 days (1 DOT) |
| R₇ | 1093 | 1111111₃ | Triple Circle | ✓ | 1096 days (3 DOTs) |
| R₈ | 3280 | 11111111₃ | Ennead Circle | ✗ | 3289 days (9 DOTs) |
| R₉ | 9841 | 111111111₃ | Grand Circle | ✗ | 9868 days (27 DOTs) |

Key properties:
- **Factorization**: R(2n) = R(n) × (3ⁿ + 1)
- **Period**: 3⁶ ≡ 1 (mod 364), so checksum space has period 6 in the exponent
- **Calendar scaling**: R₆→1yr, R₇→3yr, R₈→9yr, R₉→27yr (powers of 3)
- **Bug fix (verified)**: R₇ DOT count = floor(1093/364) = **3** (not 2), giving 1096 calendar days

**Critical structural fact**: 13 does NOT generate Z₃₆₄ because 364 = 13 × 28. For the full circle modulus, a different coprime generator is needed (e.g., step 11, since gcd(11, 364) = 1). See INVARIANT 10.

**The (13, 1093) prime repunit bracket**: R₃ = 13 and R₇ = 1093 are the only prime repunits in the operational hierarchy (R₃–R₉). This is not accidental — it is forced by repunit structure and deliberately retained because every composite repunit introduces algebraic factors that fragment required cyclic groups. Their primality has specific consequences:

- **R₃ = 13 (intra-cube)**: Governs 13 dimensions, 13 moons, 13° per radian, stride-13 sponge permutation. Primality guarantees coprimality with all state widths not divisible by 13 (see INVARIANT 10).
- **R₇ = 1093 (inter-cube / multi-year)**: Governs the 3-year harmonic (3 × 364 + 1). Because 1093 is prime, any non-zero residue modulo 1093 generates the full multiplicative group Z*₁₀₉₃, simplifying proofs about complete cycles in extended diffusion layers.
- **Combined guarantee**: The pair brackets the operational range — 13 for geometry, 1093 for extended arithmetic. Their primality ensures no unexpected factorization can break cyclic properties assumed by routing, scheduling, and cryptographic subsystems.

Source: `shared/repunit-circles.ts` (211 lines), `ternary-math/src/repunit_circles.rs` (132 lines)

#### 1.2.2 Repunit Checksum

A lightweight 6-trit integrity check for 27-trit classification addresses using mod R₆ = 364. Algorithm: interpret Rep C trits as Rep B (subtract 1), evaluate as base-3 number via Horner's method with incremental mod-364 reduction, decompose result into 6 Rep B trits, lift back to Rep C. All arithmetic in GF(3) — no domain crossing. Complements TIS-27 sponge integrity with fast constant-time branchless verification.

Source: `shared/repunit-checksum.ts` (83 lines), `ternary-math/src/repunit_checksum.rs` (200 lines)

### 1.3 The 13-Moon Calendar (13 × 28 = 364)

The year divides into 13 moons of 28 days + 1 intercalary Day Out of Time (DOT):

| Phase | Moons | Period | Days |
|-------|-------|--------|------|
| Pre-DOT (waxing) | 1–8 (Magnetic → Galactic) | Apr 1 → Nov 10 | 224 |
| **DOT** | Day Out of Time | **Nov 11** | 1 |
| Post-DOT (waning) | 9–13 (Solar → Cosmic) | Nov 12 → Mar 31 | 140 |
| **Total** | | | **365** |

The DOT splits the year at the **Fibonacci point**: 8 moons before, 5 after (8 and 5 are consecutive Fibonacci numbers). This is not arbitrary — it is the golden-ratio partition of 13.

Moon names: Magnetic, Lunar, Electric, Self-Existing, Overtone, Rhythmic, Resonant, Galactic, [DOT], Solar, Planetary, Spectral, Crystal, Cosmic.

Source: `client/src/pages/thirteen-moon.tsx`

### 1.4 Tribonacci Constant (τ)

```
τ ≈ 1.83928675521416113255185256465328660042417874609759
τ³ = τ² + τ + 1  (defining polynomial)
```

50-digit precision above. JS/TS uses 17 significant digits (IEEE 754 double). Rust f128 / arbitrary precision uses full value.

Tribonacci sequence: `0, 0, 1, 1, 2, 4, 7, 13, 24, 44, 81, 149, 274, 504, 927, 1705, ...`

**Critical alignment**: T₇ = **13** = 1 ternary radian = `111₃` = Cosmic Radius.

Source: `shared/tribonacci-constants.ts` — TAU, TAU_POWERS, TRIBONACCI_SEQUENCE, VM_CONSTANTS

### 1.5 Saturnian Magic Square

The 3×3 circulant magic square:

```
| 111 |  14 | 208 |
| 208 | 111 |  14 |
|  14 | 208 | 111 |
```

Magic constant = 333. Every row, column, diagonal sums to 333. Each row is a cyclic permutation of [111, 14, 208] — the matrix is **circulant**. The three entries are not free parameters; they are forced by the magic constant and the circulant structure.

#### 1.5.1 The Cosmic Radius

The **Cosmic Radius** is the radius of the ternary circle — the fundamental geometric constant connecting the Saturnian Magic Square to every layer of PlenumNET's architecture. It is extracted from the magic square entry 208:

```
RADIUS_COSMIC = 208 / 16 = 13 = T₇
```

The ternary circle formula **C = 2πr** with all-integer constants:

| Symbol | Value | Derivation |
|--------|-------|------------|
| r (cosmic radius) | **13** | 208 / 16 = T₇ |
| π (ternary pi) | **14** | T₇ + T₃ = 13 + 1 |
| 2π | **28** | Lunar-solar harmonic = Z₂₈ cyclic order |
| C (circumference) | **364** | 2 × 14 × 13 = 111111₃ |

Verify: 2 × 14 × 13 = **364** ✓

#### 1.5.2 Where 16 Comes From

The expression `208 / 16` is written in the code (rather than the literal `13`) to **preserve provenance** — to show that the cosmic radius is derived from the Saturnian matrix entry, not independently declared. This is the "no rounding, no floating-point" principle: every constant traces back to an exact integer relationship in the magic square.

The number 16 itself carries structural significance:

- **16 = 2 × 8**, where 8 = the number of constraints in the formal system that forces d = 13 as the unique dimension count (§3.2.1)
- **16 in base 3 = 121₃** — a palindromic ternary number (1×9 + 2×3 + 1)
- **16 = 4² = (T₅)²** — the square of the 5th Tribonacci number (T₅ = 4)
- **208 = 16 × 13**: the magic square entry encodes the radius; the cofactor 16 releases it

The three magic square entries decompose through the same Tribonacci lattice: 111 = TERNARY_BALANCE_CENTER, 14 = T₇ + T₃, 208 = (T₅)² × T₇.

#### 1.5.3 Generative Cascade

Every constant in the framework cascades from the magic square through exact integer arithmetic — zero rounding, zero approximation:

```
208 / 16 = 13           (cosmic radius = T₇)
14 = 13 + 1             (ternary π = T₇ + T₃)
2 × 14 = 28             (lunar-solar harmonic = Z₂₈ cyclic order)
28 × 13 = 364           (cosmic circumference = ternary full circle = 111111₃)
360 − 333 = 27          (phase dissonance = 3³ = TDNS dimensions)
27 + 1 = 28             (dissonance closure = 2π)
```

The magic square is the **generative root**. The values {111, 14, 208} propagate through Tribonacci alignment into the circle, calendar, sponge stride, cube dimension, and every layer of the architecture.

#### 1.5.4 Derived Constants

| Constant | Expression | Value | Role |
|----------|-----------|-------|------|
| RADIUS_COSMIC | 208 / 16 | 13 | Cosmic radius, cube dimension, T₇ |
| PI_ESOTERIC | T₇ + T₃ | 14 | Ternary π |
| LUNAR_SOLAR_HARMONIC | 2 × PI_ESOTERIC | 28 | Z₂₈ cyclic order, lunar month |
| COSMIC_CIRCUMFERENCE | 28 × 13 | 364 | Full ternary circle, Salvi calendar year |
| PHASE_DISSONANCE | 360 − 333 | 27 | 3³, TDNS dimensions |
| DISSONANCE_CLOSURE | 27 + 1 | 28 | Returns to 2π |
| TEMPORAL_CROSS_DENOM | SUFT_RADIUS | 13 | φ/13 cross-ratio denominator |
| MASS_SHELL_RATIO | 13 / 28 | 13/28 | Mass-shell coefficient |
| SATURNIAN_NATURAL_YEAR | COSMIC_CIRCUMFERENCE | 364 | Calendar year in days |

**Tribonacci–Saturnian Harmony invariant**: `RADIUS_COSMIC === TRIBONACCI_SEQUENCE[7]` — the magic square and the Tribonacci sequence independently produce the same value (13). This is verified at compile time via `HAS_TRIBONACCI_SATURNIAN_HARMONY` in the source.

Source: `shared/saturnian-blueprint.ts`, `shared/saturnian-matrix-utils.ts`

### 1.6 The 28th Factor: Squaring the Circle

Within the ternary system, "squaring the circle" is not a geometric curiosity — it is the pivotal demonstration that the system is **self-consistent, discrete, and free of transcendental numbers**. This is a structural axiom, not an approximation.

#### 1.6.1 First-Principles Derivation

In classical Euclidean geometry, squaring the circle is impossible because π is transcendental — no finite compass-and-straightedge construction can yield a square with area exactly equal to a given circle. That impossibility is specific to compass-and-straightedge methods under the Euclidean axiom set.

PlenumNET derives the circle from different first principles: the base-3 repunit summation 111111₃ = 364 defines the full circle, and from that derivation π = 14 follows as an exact integer. The system does not approximate or work around classical π — it **derives the circle differently**, from ternary arithmetic axioms, producing a self-consistent geometry where every constant is an integer and the classical constraint simply does not apply.

#### 1.6.2 The Ternary Circle (364°)

The framework replaces the conventional 360° circle with a **364° circle**, defined by the six-digit base-3 repunit:

```
111111₃ = 1 + 3 + 9 + 27 + 81 + 243 = 364
```

All angular measures derive from this integer value. No transcendental numbers appear.

#### 1.6.3 Integer π and the Radian

Because the full circle is 364°, the radian is defined as:

```
1 radian = 364° / 28 = 13°     (exact)
```

The factor 28 arises from the geometry: the system has **27 core ontological trits** plus **1 confidence factor** (3² = 9 confidence levels, which together with 27 give 28 effective dimensions). Thus:

```
2π = 28     ⟹     π = 14     (exact integer)
```

There is no approximation; π is exactly 14 within this arithmetic.

#### 1.6.4 The Structural Roles of 13 and 28

The numbers 13 and 28 are not arbitrary. They are bound by five independent structural constraints:

- **13** = 111₃ (base-3 repunit) = T₇ (7th Tribonacci number) = 1 ternary radian = prime = coprime to 54 (sponge state width)
- **28** = 2π = the cyclic order of the group Z₂₈ = 27 + 1 (ontological trits + confidence factor)
- **13 × 28 = 364** = the full circle in degrees = the six-digit repunit

These values satisfy required coprimality conditions: gcd(13, 28) = 1 guarantees the step-13 walk through the 28-element agent array visits every position exactly once. gcd(13, 54) = 1 ensures complete diffusion in TIS-27 cryptographic permutations. The relationship 13 × 28 = 364 binds the 13-dimensional hypercube to the circle's circumference.

#### 1.6.5 How the Square Emerges

The classical compass-and-straightedge impossibility does not apply here because the circle is not derived from Euclidean ratio definitions. It is derived from first principles: the repunit summation 111111₃ = **364 degrees**, with all angular measures following as exact integers.

The **square** emerges from the **3² confidence levels** (9 trit values) that extend the 27-trit ontological address to **28 effective dimensions**. This 28th factor "squares" the circle because:

```
13 (hypercube dimensions) × 28 = 364 (full circle)
```

The derivation chain:

```
3³ = 27                (core ontological trits)
3² = 9                 (confidence levels — the ternary "square")
27 + 1 = 28            (effective dimensions with confidence factor)
13 × 28 = 364          (circle closed)
111111₃ = 364          (repunit identity confirms closure)
364 / 28 = 13          (1 radian = 13° exactly)
28 / 2 = 14            (π = 14 exactly, since 2π = 28 radians = full circle)
```

The circle and its circumscribed square share an exact, integral relationship. All quantities are integers; there are no approximations, no infinite decimals, and no irrationals or transcendentals — only **powers of 3, repunit identities, and Tribonacci alignments**.

#### 1.6.6 Why This Is Load-Bearing (INVARIANT 4)

This definition is enforced as **INVARIANT 4**. Changing any of the linked constants (364°, π=14, 13° radian, 28-element cycle, or the repunit identities) would break the entire framework: routing, addressing, cryptographic key derivation, calendar systems, and timing protocols would all become inconsistent. The geometry **is** the system.

The structural closure echoes throughout the architecture: routing, addressing, key derivation, calendar systems, and timing protocols all derive from these same integer constants, ensuring every component is mathematically coherent.

### 1.7 GF(3) — Galois Field of Order 3

Elements {0, 1, 2} under modular arithmetic. This is the algebraic foundation for:

- Address derivation: `project_to_gf3(k, N)` maps signal counts to trit values
- Hypercube routing: next-hop = single trit flip in GF(3)
- Metatronic Cube operations: axis arithmetic uses `gf3Add`, `gf3Neg`
- Phase encryption: phase angles computed in GF(3)

The lift from GF(3) to Rep C: `trit = gf3 + 1` (mapping {0,1,2} → {1,2,3}).

---

## 2. TDNS v2.5.0 — Ternary Domain Name System

### 2.1 What It Does and Why

TDNS replaces **DNS, BGP, PKI, IGMP/PIM, and PTP** within the managed PlenumNET fabric. It answers: "What IS this entity, ontologically?" — not just "Where is it?"

Every entity gets a **54-trit dual-layer address**: 27 classification trits (ontological description) + 27 identity anchor trits (unique identity derived from canonical URL via TIS-27 sponge). The classification layer IS the route. Hamming distance between classification addresses equals hop count. No routing tables needed — the geometry is the protocol.

**Why 27 dimensions**: 3³ = 27. Three cubed. The ternary system's own cube count.
**Why 7 categories**: 7 root questions cover the complete ontological space of any networked entity.
**Why 54 trits**: 27 classification + 27 identity = state width of the TIS-27 sponge (54 trits).
**Address space**: 3²⁷ × 9 = 68,630,377,364,883 (68.63 trillion) classification addresses (27 trits × 9 confidence levels). Identity anchors are unique per canonical URL.

### 2.2 The 27-Dimensional Ontological Schema

| # | Category | Prefix | Dim | Question | Values (1/2/3) |
|---|----------|--------|-----|----------|----------------|
| 1 | WHO | WO | 1 | What kind? | Personal / Corporate / Governance |
| 2 | WHO | WO | 2 | Who's it for? | Just me / My group / Everyone |
| 3 | WHO | WO | 3 | Who runs it? | Anonymous / Known / Transparent |
| 4 | WHO | WO | 4 | Who hosts it? | Me / A provider / The cloud |
| 5 | WHAT | WA | 5 | What is it? | Website / App / Device |
| 6 | WHAT | WA | 6 | What's on it? | Text / Media / Live |
| 7 | WHAT | WA | 7 | Who uses it? | People / Software / Both |
| 8 | WHAT | WA | 8 | Does it think? | No / Partly / Yes |
| 9 | WHERE | WR | 9 | Who can see it? | Just me / My group / Everyone |
| 10 | WHERE | WR | 10 | Need to log in? | No / Password / ID Check |
| 11 | WHERE | WR | 11 | How many servers? | One / Several / Many |
| 12 | WHERE | WR | 12 | What connection? | HTTP / WebSocket / Raw TCP |
| 13 | WHEN | WN | 13 | What era? | Pre-2010 / 2010s / 2020s+ |
| 14 | WHEN | WN | 14 | Availability? | Business hrs / Extended / 24/7 |
| 15 | WHEN | WN | 15 | Data freshness? | Historical / Current / **Live** |
| 16 | WHEN | WN | 16 | Real-time? | Batch / Near-time / **Real-time** |
| 17 | WHY | WY | 17 | Handles money? | No / Accepts / Processes |
| 18 | WHY | WY | 18 | Wants my data? | No / Some / Lots |
| 19 | WHY | WY | 19 | Has policies? | No / Basic / Detailed |
| 20 | WHY | WY | 20 | Costs money? | Free / Pay-per-use / Subscription |
| 21 | HOW | HO | 21 | Who gets it? | One person / A group / Closest |
| 22 | HOW | HO | 22 | Data direction? | Out / Through / In |
| 23 | HOW | HO | 23 | Get updates? | I ask / I subscribe / It tells me |
| 24 | HOW | HO | 24 | Remembers me? | No / For a bit / Always |
| 25 | PEACE | PE | 25 | Encrypted? | No / Basic TLS / Full TLS |
| 26 | PEACE | PE | 26 | Trackers? | Many / Few / **None** |
| 27 | PEACE | PE | 27 | Audited? | No / Self-certified / Audited |

When dims 15 AND 16 are both 3 (Live + Real-time), the address is HPTP-mandatory.

Source: `services/tdns-v2/src/schema.rs` — SCHEMA[27]

### 2.3 54-Trit Dual-Layer Address Format

The v2.5.0 address has two layers separated by ` · ` (space-dot-space):

```
WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313 · ID:123312231312213312123321231
├── 27 classification trits (ontological)              ├── 27 identity anchor trits (URL-derived)
```

| Layer | Trits | Source | Purpose |
|-------|-------|--------|---------|
| Classification | 27 (7 category groups) | Live HTTP/DNS/TLS scan + derivation rules | Routing, ontological description |
| Identity Anchor | 27 | TIS-27 sponge of canonical URL | Unique entity fingerprint |

**Identity derivation** (`deriveIdentityTrits()`): Canonical URL → UTF-8 bytes → trit decomposition → TIS-27 sponge absorb/squeeze → 27 Rep C trits. Same sponge primitive as scan hash, different input domain. Mirrors `services/tdns-v2/src/identity.rs`.

**Scan hash**: TIS-27 sponge of classification trits (GF(3) values), output as 27-trit Rep C string. Algorithm tag: `scan_hash_algo: "tis-27"`.

**CGUID** (Cube Globally Unique Identifier): Derived from scan hash trits — `(trit[0] - 1) * 3 + trit[1]`. Range 0–8.

| Format | Example | Use |
|--------|---------|-----|
| Full dual-layer | `WO:2323 WA:1133 ... PE:313 · ID:123312231312213312123321231` | Display, extension |
| Classification only | `WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313` | Routing, Rust crate |
| Wire | 7 bytes (27 trits × 2 bits = 54 bits + 2 padding) | Network |

Wire encoding per trit: `1=0b01, 2=0b10, 3=0b11, 0b00=reserved/invalid`

### 2.4 TIS-27 Integrity Sponge (server-side)

TIS-27 is a fast sponge with 43-bit cryptographic security, proven by wide-trail analysis (χ(x)=x¹⁷ over GF(27), DP_max=1/9, B(M_θ)=8, trail probability < 10⁻³⁹⁰⁸ — see TM-2026-008). Same construction as the kernel sponge (TL-Sponge-385), sized for fast integrity rather than post-quantum resistance. Used for wire packet integrity and scan hashing. For operations requiring post-quantum security (signing, key derivation, identity binding), use TL-Sponge-385 (Section 6.7).

The sponge parameters are derived from TDNS architecture — not chosen arbitrarily:

| Parameter | Value | Why |
|-----------|-------|-----|
| State | 54 trits | Full TDNS address width (27 classification + 27 identity) |
| Rate | 27 trits | Identity anchor width = classification width |
| Capacity | 27 trits (~43 bits) | Classification layer width |
| Rounds | 4 | Full diffusion in 2 rounds; 4 rounds yield trail probability < 10⁻³⁹⁰⁸ (TM-2026-008) |
| Stride | 13 | gcd(13,54)=1 — complete permutation cycle; 13=T₇=111₃=1 rad (see INVARIANT 10) |
| Theta | 7-neighbor extended | Offsets ±1/±7/±13 — all coprime to 54; balanced_wrap for GF(3) nonlinearity |
| Speed | ~191 ns/hash | 3.5× faster than SHA-256 |

Operations: `tisTheta` (7-neighbor diffusion at ±1/±7/±13), `tisPi` (stride-13 permutation), round constant addition. All arithmetic in GF(3) = {0,1,2}. Output lifted to Rep C {1,2,3}. No SHA-256. No BLAKE3. No binary hash primitives.

Source: `ternary-math/src/tis_sponge.rs` (290 LOC), `shared/tis-sponge.ts` (77 LOC), `server/routes/tdns.ts` (lines 39–97)

### 2.5 5 GF(3) Composite Scores

Each scan produces 5 normalized scores (0–100):

| Score | Dimensions | Weight |
|-------|------------|--------|
| Trust Index | 0.35×WHO + 0.30×PEACE + 0.20×WHY + 0.10×WHEN + 0.05×complexity | Primary |
| Privacy Score | inv(D18) + D19 + inv(D24) + D26 + inv(D8) | Privacy-focused |
| Complexity Score | D9+D10+D11+D12+D21+D22 avg | Infrastructure |
| Maturity Score | D13+D14+D25+D27+D5 avg | Age/readiness |
| Privacy-Focused Index (PFI) | Tracker analysis, data collection, cookie audit | "Data Trust" in UI |

### 2.6 Scanner → Derive → Register Pipeline

1. **Scanner** (`server/routes/tdns.ts` or `scanner.rs`) makes live HTTP/DNS/TLS probes → produces 27 `RawValue` measurements + 12-header security audit + 5-category tracker analysis + cookie audit + tech fingerprint + SEO signals
2. **Derivation** applies 27 rules (15 categorical + 12 quantitative) → 27 classification trits + confidences
3. **Identity** (`deriveIdentityTrits()`) derives 27 identity anchor trits from canonical URL via TIS-27 sponge
4. **Scan Hash** computed via TIS-27 sponge of classification trits (not URL)
5. **Findings Engine** generates Critical/Warning/Info findings from scan data
6. **Registration** (`/api/tdns/register`) stores entity with .plm name, optional `org_name` for multi-URL grouping

### 2.7 Org Entities (Multi-URL Grouping)

Org entities allow grouping multiple .plm registrations under a single organizational handle:

- **Create**: `POST /api/tdns/org/create` with `org_name`, optional `display_name`
- **Add URL**: `POST /api/tdns/org/add-url` with `org_name`, `plm_name`
- **Auto-attach**: Pass `org_name` during `/api/tdns/register` to auto-create and attach
- **Query**: `GET /api/tdns/org/:name` returns all members with addresses, identity trits, CGUIDs
- **List all**: `GET /api/tdns/orgs` returns all org entities

### 2.8 API Routes (server/routes/tdns.ts)

| Method | Route | Purpose |
|--------|-------|---------|
| POST | `/api/tdns/scan` | Scan a URL → full 54-trit address, scores, findings, trackers, headers |
| POST | `/api/tdns/register` | Register .plm name with scan result, optional org_name |
| GET | `/api/tdns/resolve/:name` | Resolve .plm name → full entry with identity trits, CGUID |
| POST | `/api/tdns/org/create` | Create an org entity |
| POST | `/api/tdns/org/add-url` | Add a .plm registration to an org |
| GET | `/api/tdns/org/:name` | Get org details with all members |
| GET | `/api/tdns/orgs` | List all org entities |
| GET | `/api/tdns/list` | List all registered .plm entries |
| GET | `/api/tdns/health` | Health check (version, entity count, engine) |

### 2.9 Scan Hash Type Tags

| Tag | Algorithm |
|-----|-----------|
| `0x01` | SHA-256 |
| `0x02` | TIS-27 sponge |
| `0x03` | TL-DSA signature |
| `0x04` | Composite (multi-hash) |

### 2.10 Reference Fixture Addresses

| Entity | Classification Address | HPTP | Notes |
|--------|----------------------|------|-------|
| Google | `WO:2323 WA:1133 WR:3131 WN:1322 WY:2331 HO:1212 PE:313` | No | Trit 26 (dim 26, 0-idx 25): Numeric(0.0) = no trackers → trit 3 (None) |
| PPTPro (Capomastro) | `WO:2333 WA:2333 WR:2222 WN:3333 WY:1221 HO:2133 PE:332` | **Yes** | Trits 15+16 = 3,3 → HPTP-mandatory |
| Nonna's Cucina (blog) | Derived from blog_measurements() | No | Simple blog fixture |

### 2.11 TDNS Module Map

**Rust crate** — `services/tdns-v2/src/` — 19 source modules + bin/:

| File | Purpose |
|------|---------|
| `trit.rs` | Trit type {1,2,3}, wire encoding, GF(3) ops |
| `addr.rs` | CubeAddr (27-trit), Hamming distance, wire pack/unpack, DIMENSIONS=27 |
| `schema.rs` | 27 dimension definitions, Category enum, describe() |
| `derive.rs` | 27 derivation rules, project_to_gf3(), confidence_digit() |
| `scan.rs` | ScanMeasurement, RawValue, Confidence (#[serde(transparent)]) |
| `scanner.rs` | Live HTTP/DNS/TLS scanner (27 probes via ureq) |
| `crs.rs` | CRS Registry — register, resolve, verify, rescan, drift log, neighbor maps |
| `trn.rs` | TRN (Ternary Resource Name) records |
| `fts.rs` | FTS (Fault Tolerance Service) — heartbeat, suspect/dead/recovered |
| `glb.rs` | GLB (Geometric Load Balancer) — point/multicast forwarding, redirect, dead set |
| `overlay.rs` | CON (Cube Overlay Network) — PQ-encrypted tunnels, kernel-sponge-derived keys, rekey |
| `subcube.rs` | SubCube multicast addressing, wildcard |
| `routing.rs` | NeighborMap, greedy geometric routing |
| `bridge.rs` | Metatronic Bridge — .plm→TDNS / legacy DNS resolution |
| `wire.rs` | Packet wire protocol (version 0x25), TIS-27 integrity, heartbeat encoding |
| `identity.rs` | Identity derivation — TIS-27 sponge of canonical URL → 27 anchor trits |
| `storage.rs` | Persistent storage layer for TRN records |
| `api.rs` | HTTP API endpoints, ApiRouter |
| `lib.rs` | Crate root, module re-exports |

**Server-side scanner** — `server/routes/tdns.ts` (1,326 LOC): Full TypeScript implementation of TIS-27 sponge, 27 derivation rules, identity anchor derivation, 5 GF(3) scores, findings engine, tracker analysis, header audit, cookie audit, tech fingerprint, SEO signals, org entity management, 9 API routes. Version: v2.5.0.

Binaries (Rust):
- `src/bin/tdns_scan.rs` — CLI: scan, compare, describe
- `src/bin/tdns_server.rs` — HTTP server (port 3927 default)

---

## 3. The 13-Dimensional Hypercube (Inter-Cube Network)

### 3.1 Why Geometry Replaces Routing Tables

In conventional networks, routing tables map destination addresses to next-hop interfaces. These tables must be computed (BGP convergence), distributed (flooding), stored (memory), and protected (route poisoning). All of this is eliminated when the address space IS a geometric object.

In a 13D ternary hypercube:
- Every node knows its own 13-trit address
- The destination is a 13-trit address
- The next hop = flip the trit in the dimension with the greatest distance reduction
- No table lookups, no state synchronization, no convergence delays
- The routing decision is a single GF(3) arithmetic operation

| Property | Value | Why |
|----------|-------|-----|
| Dimensions | 13 | 13 = T₇ = 1 ternary radian |
| Vertices | 3¹³ = 1,594,323 | Single-cube address space |
| Inter-Cube Space | 3²⁶ Rep C = 2,541,865,828,329 | Scales infinitely by stacking 13 more trits |
| Neighbors/node | 2 × 13 = 26 | 2 non-self values per dimension × 13 dims |
| Max diameter | 13 hops | Worst case: all 13 trits differ |
| Routing tables | **0** | Geometry IS the routing protocol |

### 3.2 Inter-Cube Infrastructure Services (4)

Combined: 18,649 lines Rust (26 modules), 422 tests, 4-node Docker deployment, 12 HTTP endpoints.

#### 3.2.1 The Eight-Constraint System (d = 13 Unique Solution)

The dimension count 13 is not a design choice — it is forced by eight simultaneous constraints:

1. d = T₇ (7th Tribonacci number)
2. d = 111₃ (base-3 repunit)
3. d is prime
4. gcd(d, 3) = 1 (coprime to trit alphabet)
5. gcd(d, 28) = 1 (full-period walk on Z₂₈)
6. gcd(d, 54) = 1 (full-period sponge stride on 54-trit state)
7. d × 28 = 364 = 111111₃ (calendar = ternary circle)
8. 364 / (2d) ∈ ℤ (ternary π is integer)

**d = 13 is the unique solution.** Changing 13 severs the connections between routing, calendar, circle, radian, sponge, Tribonacci, and the repunit hierarchy. Full proof: `docs/TM-2026-014-Inter-Cube-Infrastructure.md`.

#### 3.2.2 Derived Quantities

| Quantity | Formula | Value |
|----------|---------|-------|
| Vertices per cube | 3¹³ | 1,594,323 |
| Neighbors per node | 2 × 13 | 26 |
| Total unique tunnels (full cube) | 26 × 3¹³ / 2 | **20,726,199** |
| Max diameter | 13 | 13 hops |
| Shortest paths (d hops) | d! | Up to 13! = 6,227,020,800 |

Tunnel count derivation: each node has 26 neighbors, each tunnel has 2 endpoints, so 1,594,323 × 26 / 2 = 20,726,199 unique PQ-encrypted tunnels per fully populated cube.

#### 3.2.3 Structural Resilience

| Trits differing (d) | Shortest paths (d!) | What it takes to block ALL |
|---------------------|---------------------|----------------------------|
| 1 | 1 | 1 specific node |
| 3 | 6 | 6 specific nodes |
| 7 | 5,040 | 5,040 specific nodes |
| 10 | 3,628,800 | 3.6 million specific nodes |
| 13 (max) | 6,227,020,800 | **Impossible** — cube has only 1.59M nodes |

At maximum distance (d=13), the shortest path count (6.2 billion) exceeds total nodes (1.59 million) by ~4,000×. Blocking all shortest paths is mathematically impossible. Detour paths provide further redundancy beyond d!.

**Structural Sybil resistance**: Each identity requires 26 authenticated tunnels with unique PQ keypairs — the hypercube's 26-neighbor requirement turns topology into a per-identity cost function. Large-scale Sybil attacks are computationally prohibitive.

#### 3.2.4 Four Services

**GLB — Geometric Load Balancer** (`glb.rs`)
- Point forwarding: greedy geometric routing via NeighborMap
- Flow affinity: `hashFlowId(flowId) % liveDelta.length` selects differing dimension — different flows take different paths through the cube automatically
- SubCube multicast: fan-out to all neighbors matching sub-cube mask
- Dead-node avoidance: skip dead nodes, find alternates via detour (flip a non-differing dimension, adding 2 hops)
- Redirect: old_addr → new_addr mappings with TTL

**CON — Cube Overlay Network** (`overlay.rs`)
- 20.7M unique PQ-encrypted tunnels per populated cube, 26 per node
- Virtual interfaces: `cubetun0` through `cubetun25`, one per geometric neighbor
- **v3 tunnel key derivation**: `TLSponge-385("PlenumNET-CON-v3.0" ‖ canonical(addr_a, addr_b) ‖ kem_shared_secret ‖ epoch) → 32-byte key`
- TL-KEM shared secret provides IND-CCA2 key secrecy and forward secrecy
- Canonical ordering: lexicographic sort of both addresses ensures both sides compute the same key independently
- Key rotation: `rekey_all()` increments epoch, re-derives all link keys
- Traffic accounting: bytes sent/received per link
- Link state: Active / Down / Rekeying

**WHY topology-derived crypto matters**: The key is derived from the geometric positions of the two endpoints. A node at position X cannot derive the key for the tunnel between Y and Z — it doesn't occupy either position. The geometry IS the key hierarchy. No key exchange protocol, no certificate authority, no central point of failure. No existing overlay network derives keys from its own topology.

**CRS — Cube Registration Service** (`crs.rs`)
- Entity registration with scan measurements
- TRN record management (name → address lookup)
- Neighbor map computation from registered entities
- Multi-homing: a single physical machine can register multiple ternary addresses, each with its own CON tunnel set
- Dimension density tracking
- Drift detection and redirect management
- Verification and re-scan protocols
- In-place public key update via `update_public_key()` / `POST /crs/update-key` (no address reallocation)

**FTS — Fault Tolerance Service** (`fts.rs`)
- Heartbeat-based failure detection
- Four-state health model: Up → Suspect → Down → Recovering
- Configurable thresholds: heartbeat_interval_ns, suspect_threshold, failure_threshold
- Recovery detection: Down → Recovering → Up via NodeRecovered event
- HPTP anomaly tracking
- Dead-set publication feeds GLB for real-time path avoidance

#### 3.2.5 Daemon Identity & Key Rotation

**Daemon Identity** (`daemon_identity.rs`, `identity.rs`)
- Persistent encrypted MasterSecret stored at `~/.plenumnet/identity/master.key` (or `CUBE_IDENTITY_DIR`)
- Generated via OS CSPRNG (`getrandom` crate) — 48-byte secret
- Encrypted at rest with passphrase from `CUBE_IDENTITY_PASSPHRASE` env var (or hostname-derived fallback)
- File permissions set to 0o600 on Unix
- PT26-DSA keypair derived from MasterSecret via `DaemonIdentity::init()`

**Daemon Modes** (`main.rs` — `CUBE_MODE` env var)
- `CUBE_MODE=crs`: Starts CRS node with real PT26-DSA identity; registers with own public key, derives address-bound TL-DSA-87 key after address assignment, updates CRS registry in-place
- `CUBE_MODE=cube`: Starts cube node; registers with CRS using PT26-DSA pk, derives address-bound key after assignment, updates CRS via `/crs/update-key`, runs 30s heartbeat with rotation checks
- `CUBE_MODE=keygen`: Generates/loads identity, prints hex public key, and exits

**Key Rotation** (`key_rotation.rs`)
- Radian epoch rotation: 14-day units from Salvi Epoch (2025-04-01T00:00:00Z)
- `RotationOrchestrator::check_and_rotate()` called every 30s in heartbeat loop
- On epoch boundary: generates new MasterSecret, re-derives address-bound TL-DSA-87 keypair, persists new encrypted secret, updates CRS key via `/crs/update-key` (no address reallocation)

**Key Freshness Zones** (`key_rotation.rs` — `KeyFreshnessZone`)
- GF(3) quantization at 1/3 and 2/3 of ARC_EPOCH (182 days):
  - `Fresh` (age 0–60 days): all operations permitted
  - `Active` (age 61–121 days): regulated operations. Boundary at 121 = REPUNIT_R5 = 11²
  - `Aging` (age 122–182 days): read-only
- `key_freshness_zone(age_days)` → `Option<KeyFreshnessZone>` (None = expired)
- `key_suitable_for_regulated(age_days)` → bool (true for Fresh or Active)
- `key_birth_epoch(current_epoch, age_days)` → birth epoch

#### 3.2.6 PlenumLAN Node (`plenumlan/`)

Standalone Rust crate: Array3 Node Cluster pipeline — 27 classification trits → 3-trit slot → port → registration. Version 2.4.1.

**Crate location**: `plenumlan/` (workspace member, depends on `ternary-math` and `tdns-v2`)

**Modules** (all under `plenumlan/src/cube/`):

| Module | Purpose |
|--------|---------|
| `constants.rs` | BASE_PORT=11111, MAX_NODES=3, SLOTS_PER_NODE=27, DIMS_PER_GROUP=9, GATEWAY_NODE_ID=1, GATEWAY_OFFSET=13. `const_assert!` compile-time checks. Imports DIMENSIONS, GF3_ORDER from ternary-math. |
| `projection.rs` | 27→3 projection via polarity tables (3 groups × 9 dims). `project_to_slot(classification) → SlotAddr`. Uses `project_to_gf3(k, DIMS_PER_GROUP)` from INVARIANT 2. |
| `port.rs` | Port formula: `BASE_PORT + (node_id-1)*27 + offset`. 27 single-node ports, 81 Array3 ports. `node_port_range(node_id) → (start, end)`. Round-trip verified. |
| `bridge.rs` | Legacy bridge: `derive_legacy_bridge()` — D5/D6/D12/D15 match → ports 53/67/445/631/1812. |
| `routing.rs` | Inter-service routing: HD 0 = loopback, HD 1 = direct, HD 2 = capability token, HD 3 = full mutual auth. Loop-free proof for all 702 pairs. Uses `hamming_distance` from placement.rs. |
| `node.rs` | `NodeConfig` from env vars (CUBE_NODE_ID, CUBE_ARRAY3_PEERS, EAGER_BIND). `Array3Handshake` with node_id, port_range, wire_protocol_version, gateway_addr, slot_inventory. `validate_handshake()` rejects zero node IDs, duplicates, port overlaps, incompatible protocols. |
| `sentinel.rs` | Zero-sentinel checks at trust boundaries. |
| `freshness.rs` | Fresh/Active/Aging zones (mirrors key_rotation.rs KeyFreshnessZone). |
| `scan_templates.rs` | 6 LAN entity scan templates per §7.8. |
| `entity_detect.rs` | Entity class detection: HTTP endpoints → TDNS scanner, hardware → template-driven scan. |
| `windows_roles.rs` | 12 SMB roles with plane validation (Outer→data, Void→control, Inner→management). |

**Environment Variables** (parsed in `node.rs` and `main.rs`):

| Variable | Default | Purpose |
|----------|---------|---------|
| `CUBE_NODE_ID` | 1 | Rep C {1,2,3}. Zero = fatal (zero-sentinel). |
| `CUBE_ARRAY3_PEERS` | (empty) | Comma-separated `ip:port` for Array3 peer discovery. |
| `EAGER_BIND` | false | `true`/`1` = bind all 27 ports at startup (production). Default = lazy bind on register (Replit/dev). |

**Port Ranges** (BASE_PORT=11111, 27 ports per node):

| Node | Range | Gateway/Center |
|------|-------|----------------|
| Node 1 | 11111–11137 | 11124 (gateway) |
| Node 2 | 11138–11164 | 11151 |
| Node 3 | 11165–11191 | 11178 |

#### 3.2.7 ServiceSlotRegistry (`crs.rs`)

V3 extension to CRS for Array3 slot management. Manages the 81-slot space (3 nodes × 27 slots each).

- `SlotAddr = [u8; 3]` — 3-trit Rep C slot address: [plane, role, instance]
- `ServiceSlot` — registered slot with slot_addr, node_id, port, identity (TL-DSA pubkey), 13 capability flags, 27-trit classification, registration timestamp
- `SlotError` — ZeroNodeId, NodeIdOutOfRange, InvalidClassification, SlotOccupied, ProjectionFailed
- `ServiceSlotRegistry::register_service(node_id, classification, identity, capabilities)` — 27 trits in → ServiceSlot out. Performs 27→3 projection, computes port, validates Rep C, checks conflicts.
- `ServiceSlotRegistry::resolve_slot(slot_addr)` — 3-trit slot → `SlotRoute` (node_id, port, identity). Searches all 3 nodes.

#### 3.2.8 Wire Protocol V3 (`wire.rs`)

Version history: V1 (legacy), V2 (signed/hardened), V3 (Array3 slot addressing).

| Constant | Value | Purpose |
|----------|-------|---------|
| `PROTOCOL_VERSION_V3` | 0x03 | Current version emitted by this build |
| `PROTOCOL_VERSION_MIN` | 0x02 | Minimum accepted (V2 dual-acceptance during rollout) |
| `PROTOCOL_VERSION_CURRENT` | 0x03 | Alias for V3 |

**V3 Message Types** (0x60–0x65):

| Code | Type | Purpose |
|------|------|---------|
| 0x60 | `Array3Handshake` | Node cluster formation |
| 0x61 | `Array3HandshakeAck` | Formation acknowledgement |
| 0x62 | `SlotRegister` | Service slot registration |
| 0x63 | `SlotRegisterAck` | Registration acknowledgement |
| 0x64 | `SlotQuery` | Slot lookup query |
| 0x65 | `SlotQueryResponse` | Slot lookup response |

- `requires_v3()` returns true for all 0x60–0x65 message types
- `WireMessage::validate()` enforces: V3 message types on V2 headers → `WireError::MessageRequiresV3`
- V3 accepts V2 messages (dual-acceptance). V1 is rejected.

**Slot Address Wire Encoding**:
- `pack_slot_addr([u8; 3]) → Option<u8>` — 3-trit slot in 1 byte (bits [7:6]=plane, [5:4]=role, [3:2]=instance)
- `unpack_slot_addr(u8) → Option<[u8; 3]>` — zero in any 2-bit field = forgery
- `WIRE_SLOT_ADDR_SIZE = 1`
- `ARRAY3_HANDSHAKE_MIN_SIZE = 7` (node_id + port_start + port_end + wire_ver + slots)
- `SLOT_REGISTER_PAYLOAD_SIZE = 63` (1 + 27 + 1 + 32 + 2)

#### 3.2.9 Health & Version Reporting

`GET /health` returns:

```json
{
  "status": "ok",
  "service": "PlenumNET Inter-Cube Infrastructure",
  "version": "2.4.1",
  "mode": "crs",
  "address": "1111111111111",
  "wire_protocol": 3,
  "wire_protocol_min": 2,
  "node_id": 1
}
```

Startup banner includes: version, wire protocol, node ID, Array3 peers (if configured).

**Version Constants** (`shared/constants.ts`):

| Constant | Value |
|----------|-------|
| `WIRE_PROTOCOL_VERSION` | 0x03 |
| `DAEMON_VERSION` | "2.4.1" |
| `TDNS_VERSION` | "v2.5" |
| `PLENUMLAN_VERSION` | "2.4.1" |

#### 3.2.10 Cubes-of-Cubes Scaling

| Level | Address format | Nodes | Scale |
|-------|---------------|-------|-------|
| 1 | [13 trits] | 3¹³ = 1,594,323 | Campus network |
| 2 | [13 trits : 13 trits] | 3²⁶ = 2.54 trillion | Every device on Earth × 300 |
| 3 | [13 trits : 13 trits : 13 trits] | 3³⁹ = 4.05 quintillion | More than grains of sand |

Same geometry, same routing math, same four services. No architectural change. Max hops per level: 13 (each level adds at most 13 hops — geometric routing applied recursively). Gateway nodes at cube boundaries maintain tunnels to geometric neighbors in the outer address space.

#### 3.2.11 Comparison to Existing Systems

| System | How it routes | How it secures | What it stores |
|--------|--------------|----------------|----------------|
| **IP/BGP** | Routing tables (thousands of entries, minutes to converge) | TLS certs from CAs | Full routing tables |
| **Tor** | Onion routing through directory-selected relays | Layered encryption, fixed directory servers | Circuit state, relay lists |
| **Chord/Kademlia** | Distributed hash table lookups | Key-based identity | Finger tables, successor lists |
| **WireGuard** | Kernel routing table + allowed IPs | Pre-exchanged public keys | Config files per peer |
| **PlenumNET CON** | Hamming distance on coordinates — d! paths, zero tables | Topology-derived keys from geometric position | Nothing — 26 neighbors computed from address |

#### 3.2.12 WebSocket Relay Architecture

For cross-cluster and satellite/remote access, PlenumNET provides a WebSocket relay at `wss://plenumnet.replit.app/ws/relay`. This bridges nodes that cannot reach each other on a LAN.

- **Authentication**: Nodes send `{"type":"auth","address":"...","publicKey":"..."}` within 10s of connecting; the relay verifies registration against the CRS registry
- **Message routing**: `{"type":"relay","to":"<address>","msgType":"...","payload":"..."}` — relay forwards to the target WebSocket if connected, otherwise queues (up to 100 msgs per destination)
- **Keepalive**: `{"type":"ping"}` every 25s
- **Peer discovery**: `{"type":"peers"}` returns all connected node addresses
- **Monitoring**: `GET /api/salvi/inter-cube/relay/status` returns connected peers and pending queues; `GET /api/salvi/inter-cube/relay/cluster-health` returns full cluster status with relay throughput metrics

For local YODA clusters, all registration and heartbeat happens through the local CRS (Daemon #1, port 8081). The remote relay is used only for cross-cluster communication and remote monitoring. Full WebSocket protocol details are in `PLENUMNET_INVARIANTS_FOR_YODA.md` Section 3 and the monitoring quick reference in Section 9.

### 3.3 Metatronic Cube

The 13D cube viewed through Saturnian geometry:
- Three shells of 3¹² = 531,441 vertices each
- Depth axis at position 13 (= T₇ = 1 radian)
- Rep C axis numbering (1-based bijective, zero = sentinel)
- Named axes, correspondence edges, embedded polytopes
- Automorphism group for symmetry operations

Source: `shared/metatronic-cube.ts` — METATRONIC_DIM=13, METATRONIC_VERTICES=1,594,323, SHELL_VERTICES=531,441

---

## 4. HPTP — High-Precision Timing Protocol

### 4.1 What It Does and Why

HPTP provides **femtosecond-precision time synchronization** (10⁻¹⁵ seconds). This is structurally required by HPTP-mandatory addresses (trits 15+16 = 3,3).

**Why femtosecond matters**:
- FINRA 613: ≤50ms drift from NIST atomic clock
- MiFID II (HFT): ≤1ms divergence from UTC
- MiFID II (general): ≤100μs for gateways
- PlenumNET's timing exceeds all regulatory requirements by orders of magnitude

### 4.2 Femtosecond Timestamps

- 128-bit integer measuring femtoseconds since Salvi Epoch (2025-04-01T00:00:00Z)
- `FEMTOSECONDS_PER_MILLISECOND = 1,000,000,000,000n` (bigint)
- `FEMTOSECONDS_PER_SECOND = 1,000,000,000,000,000n`

Source: `server/salvi-core/femtosecond-timing.ts`

### 4.3 HPTP Service Architecture

- **Femtosecond Service** (`services/timing/femtosecond-service/`): Fastify server, port 3006
  - ClockDriver: hardware clock abstraction (GPS/PTP/atomic in production)
  - HPTPClient: multi-peer consensus timing, drift compensation, jitter filtering
  - Timing API: `/api/timing/v1/*`
  
- **Certification Service** (`services/timing/certification-service/`): 
  - Timing Certificate Authority for FINRA 613 / MiFID II compliance
  - Cryptographically signed timing certificates
  - TimingVerifier for compliance checks

- **Kernel HPTP** (`src/kernel/src/hptp/`): 2,369 LOC across 7 files (certification, coprime_clock_rotation, crt_fast_path, jitter_correction, mod, optical, protocol), supports 7 clock sources (Local, GPSDO, Atomic Rubidium, Atomic Cesium, Optical Lattice, Chip-Scale, Network Peer), 5 precision levels (Millisecond → Femtosecond). The `coprime_clock_rotation.rs` module (224 lines) formalizes the generator theorem for clock failover: rotation step must be coprime to source count N. For N=7 (prime), all steps 1–6 are valid (maximally robust). Optimal step selection prefers T₇ mod N (for N=7: step 6). The `crt_fast_path.rs` module (458 lines, 15 tests) implements CRT decomposition Z₃₆₄ ≅ Z₂₈ × Z₁₃ for O(1) calendar-position indexing — decomposes a circle-day into (moon_position mod 28, radian_phase mod 13) and reconstructs via CRT coefficients COEFF_FINE=196, COEFF_FAST=169. CRT inverses: 13⁻¹ mod 28 = 13 (self-inverse), 28⁻¹ mod 13 = 7. The `fast_mod_28` function uses sub-CRT: (21×r4 + 8×r7) mod 28. Bonus structural alignment: 364 = 7 × 52, so all 7 clock sources are hit exactly 52 times each — perfectly uniform distribution with zero bias. TypeScript mirror: `shared/crt-fast-path.ts` (176 lines).

### 4.4 HPTP in TDNS Wire Protocol

Wire packets include femtosecond timestamps. HPTP-mandatory packets (destination has trits 15+16=3,3) MUST have valid HPTP timing — the wire protocol enforces this via the HPTP_MANDATORY flag.

Heartbeat packets carry `(hptp_offset_ns, sequence_number)` for continuous clock synchronization. 42 global calendar system conversions synchronized.

---

## 5. Document Notary — RFC 3161 Time-Stamping Authority (TSA)

### 5.1 What It Does and Why

The TSA provides **cryptographic proof-of-existence timestamps** per RFC 3161. It is a digital notary — it proves a document existed at a specific point in time, with cryptographic non-repudiation.

### 5.2 TSA Features

| Feature | Details |
|---------|---------|
| TSA Policies | 4 distinct policies for different compliance levels |
| Audit Log | Merkle tamper-evident tree — every timestamp is a leaf |
| Dual Signature | RSA-4096 + TL-DSA-87 (post-quantum + classical) |
| HPTP Integration | Femtosecond timestamp bound to HPTP-synchronized clock |
| Wire Protocol | ASN.1 DER encoding per RFC 3161 |

### 5.3 TSA Services

- **TSA Service** (`server/services/tsa-service.ts`): Core timestamp generation, policy enforcement
- **TSA Calendar Compression** (`server/services/tsa-calendar-compression.ts`): Merkle tree management
- **TSA Calendar Enrichment** (`server/services/tsa-calendar-enrichment.ts`): Metadata attachment
- **TSA Policy** (`server/services/notification-tsa-policy.ts`): Policy-based notification rules
- **TSA Routes** (`server/routes/tsa.ts`): HTTP API endpoints
- **TSA Page** (`client/src/pages/tsa.tsx`): Frontend interface

### 5.4 plenum-stamp CLI

Zero-dependency Node.js CLI for offline TSA operations:

```
plenum-stamp sign <file>       Hash and timestamp a file
plenum-stamp verify <file>     Verify a file's timestamp  
plenum-stamp info <file.tsp>   Display token metadata
plenum-stamp cert              Download TSA certificate
```

Source: `cli/plenum-stamp/index.mjs`

---

## 6. Encryption & Cryptographic Architecture

### 6.1 Phase Encryption

Adaptive Dual-Phase Quantum Encryption from the whitepaper:

| Phase | Angle | Purpose |
|-------|-------|---------|
| Primary | 360°/0° reference (fixed) | Main encryption carrier |
| Secondary | Δθ(t) = 1°–10° (tunable) | Adaptive security parameter |
| Guardian | 358° offset | Tamper detection (Tribonacci-weighted checksum) |

Modes: `high_security`, `balanced`, `performance`, `adaptive`

The guardian phase uses τ-derived constants for mixing (not djb2). This directly connects tamper detection to the framework's mathematics.

Source: `server/salvi-core/phase-encryption.ts`

### 6.2 Token Encryption (AES-256-GCM)

All API tokens and session data encrypted at rest:
- Algorithm: AES-256-GCM
- IV: 12 bytes random
- Tag: 16 bytes
- Key: SHA-256 of SESSION_SECRET
- Format: `{iv_hex}:{tag_hex}:{ciphertext_hex}`

Source: `server/crypto-utils.ts`

### 6.3 Post-Quantum Cryptography Stack

| Algorithm | Purpose | Security Levels |
|-----------|---------|-----------------|
| **TL-DSA** (Ternary Lattice DSA) | Digital signatures | 44 / 65 / 87 |
| **TL-KEM** | Key encapsulation | 512 / 768 / 1024 |
| **Phase Encryption** | Data encryption | 4 modes |
| **Kernel sponge** | Key derivation, signing, identity binding (729-trit, 9 rounds) | 385-bit PQ (243-trit capacity) |
| **TIS-27 sponge** | Wire integrity, scan hashing (54-trit, 4 rounds, proven secure — TM-2026-008) | 43-bit cryptographic security |
| **RSA-4096** | Classical signatures (dual-sig with TL-DSA) | — |
| **AES-256-GCM** | Token encryption at rest | — |

TL-DSA uses **integer NTT** for efficient polynomial multiplication and **AVX2 vectorization** for performance. Benchmarks:
- TL-DSA-44: 1,220 μs
- TL-DSA-65: 1,700 μs  
- TL-DSA-87: 2,470 μs (5.9× speedup over reference)

### 6.4 TL-DSA Bug Fix History (March 2026 — CAUTIONARY)

Eight bugs fixed in `tl_dsa.rs` and `ternary_lattice.rs`: broken `sample_challenge`, wrong matrix dims, zero-init secret keys, raw binary integers fed into ternary sponge (see INVARIANT 8). Added `u16_to_trits()` / `u8_to_trits()`. **All previously generated keys incompatible.**

### 6.5 Topology-Derived Cryptography (CON)

Each of the 20,726,199 tunnels in a fully populated cube gets a unique key derived from the geometric relationship between its two endpoints:

```
TLSponge-385("PlenumNET-CON-v3.0" ‖ canonical(addr_a, addr_b) ‖ kem_shared_secret ‖ epoch) → 32-byte key
```

Canonical ordering: lexicographic sort ensures both endpoints compute the same key independently. TL-KEM shared secret provides IND-CCA2 key secrecy and forward secrecy. Epoch counter enables key rotation.

Security-critical tunnel key derivation uses **TLSponge-385** (729-trit, 9 rounds, 385-bit PQ security — see Section 6.7). TIS-27 is used only for wire packet integrity on already-authenticated channels. The key derivation is **deterministic from topology** — both endpoints independently compute the same key pair from their geometric positions. No key exchange protocol needed. The geometry IS the key agreement.

### 6.6 6-Phase Capability-Based Security

Authorization uses unforgeable, self-contained, bearer-verified capability tokens:

| Phase | Feature |
|-------|---------|
| 1 | Typed constraint registry |
| 2 | HPTP-bound expiration (femtosecond-precise) |
| 3 | HMAC-chained delegation |
| 4 | Hardware-bound capabilities |
| 5 | RFC 3161 capability certificates |
| 6 | Inter-service capability mesh |

Source: `server/services/capability-service.ts`, `capability-certificates.ts`, `capability-hardware-binding.ts`, `capability-mesh.ts`, `capability-audit-events.ts`

### 6.7 Kernel Cryptographic Sponge (Post-Quantum)

The kernel sponge is the primary cryptographic primitive for signing, key derivation, identity binding, TDNS registration, and FIPS validation. It operates in balanced ternary {-1, 0, +1} (Rep A), distinct from TIS-27's unsigned GF(3) {0, 1, 2}.

| Parameter | Value | Derivation |
|-----------|-------|------------|
| State | 729 trits (3⁶) | Ternary cube dimension alignment |
| Rate | 243 trits (3⁵) | Absorption width |
| Capacity | 486 trits (~770 bits) | 729 − 243; determines security level |
| Rounds | 9 (3²) | 3× safety margin over 3-round full diffusion threshold |
| Permutation (π) | π(i) = (376·i + 1) mod 729 | Affine permutation — full-cycle guaranteed |
| Theta | 7-neighbor extended | Offsets ±1/±7/±13; all coprime to 729 |
| Theta nonlinearity | `balanced_wrap(left + s[i] + right + 1)` | The `+1` constant ensures algebraic nonlinearity (trit rotation) |
| Security | 243-trit preimage = 385.4 bits | 243 × log₂(3) ≈ 385.4 — exceeds SHA-384's 192 bits |

The 7-neighbor theta computes:
```
left  = balanced_wrap(s[i-13] + s[i-7] + s[i-1])
right = balanced_wrap(s[i+1] + s[i+7] + s[i+13])
out[i] = balanced_wrap(left + s[i] + right + 1)
```

The extended theta was previously 3-neighbor (±1 only), requiring 6 rounds for full diffusion. The v2 upgrade to 7 neighbors at ±1/±7/±13 halved the diffusion threshold to 3 rounds (verified), allowing the round count reduction from 27 to 9 (3²) while maintaining a 3× safety margin.

Source: `src/kernel/src/crypto/sponge.rs`

### 6.8 TIS-81 — High-Security Integrity Variant (Benchmark Only)

TIS-81 is a higher-capacity variant of the TIS sponge family, positioned between TIS-27 (fast integrity) and the kernel sponge (full PQ crypto). It exists in benchmarks only — NOT deployed as a separate runtime module.

| Parameter | Value | Notes |
|-----------|-------|-------|
| State | 243 trits (3⁵) | 4.5× TIS-27 |
| Rate | 81 trits (3⁴) | 3× TIS-27 |
| Capacity | 162 trits (~257 bits) | Post-quantum secure (exceeds 128-bit PQ threshold) |
| Rounds | 4 | Same as TIS-27 |
| Stride | 13 | gcd(13,243)=1 — complete cycle |
| Theta | 7-neighbor extended | Same ±1/±7/±13 offsets as kernel and TIS-27 |

Decision: TIS-81 was evaluated but deliberately NOT deployed as a separate runtime path. The kernel sponge (385-bit) covers all security-critical operations, and TIS-27 (43-bit) covers all wire integrity needs. TIS-81 exists for benchmark comparisons against SHA3-256.

Source: `benchmarks/rust-bench/src/tis_sponge.rs`, `benchmarks/c-bench/tis81_simd.c`

### 6.9 Sponge Family Summary

| Variant | State | Rate | Capacity | Rounds | Security | Role |
|---------|-------|------|----------|--------|----------|------|
| **TIS-27** | 54 | 27 | 27 (~43 bit) | 4 | Integrity only | Wire packets, scan hashing |
| **TIS-81** | 243 | 81 | 162 (~257 bit) | 4 | PQ-secure | Benchmark only (not deployed) |
| **Kernel** | 729 | 243 | 486 (~385 bit) | 9 | Full PQ | Signing, key derivation, identity |

All three share the same 7-neighbor extended theta at ±1/±7/±13 and stride=13 permutation (adapted to state width). The theta function is the unifying design element across the family.

### 6.10 T-AE-MAC — Ternary Authenticated Encryption MAC (TM-2026-011)

T-AE-MAC is the dual-phase authenticated encryption construction used by PlenumNET's phase encryption system. It separates confidentiality (forked bulk-rate keystream) and integrity (clean base MAC on undisturbed state) from a shared cryptographic root, operating entirely in native GF(3) arithmetic.

**Construction:**

```
base = absorb(key_material ‖ nonce ‖ phase_angle_364 ‖ context_tag)   ← domain commitment
clone(base) → absorb(tag+counter) → squeeze_bulk(keystream)            ← keystream path (clones only)
cipher_trit[i] = plain_trit[i] + keystream_trit[i]  (mod 3, balanced)  ← GF(3) OTP
base.absorb(header_trits ‖ cipher_trits)                               ← MAC path (base never squeezed)
base.squeeze(mac_trit_count)                                            ← tag from pristine base
```

**Critical architectural distinction**: The base sponge that produces the MAC tag has **never had keystream extracted from it**. Keystream generation was performed on independent clones. The MAC state is pristine — no rate bits were ever revealed. This is unlike classical sponge AE modes where the same state alternates between squeezing keystream and absorbing ciphertext.

**Sponge Version Dispatch:**

| Version | Chi | Keystream Rate | MAC Path | Parallelism |
|---------|-----|----------------|----------|-------------|
| v1 | None | 243 (standard) | Sequential, bytes | Sequential |
| v2 | χ(x¹⁷) | 243 (standard) | Sequential, bytes | Sequential |
| v3 (T-AE-MAC) | χ(x¹⁷) | 486 (bulk) | Dual-phase, trits native | Auto-gated rayon |

v1 and v2 preserved for backward compatibility. All new encryptions use v3. `spongeVersion` field in encrypted format for dispatch.

**Trit-native MAC path (v3 fix)**: Cipher trits absorbed directly via `absorb(&cipher_trits)` — the sponge never leaves GF(3) for the MAC path. Prior versions packed cipher trits to bytes then unpacked back, wasting CPU on redundant encoding/decoding. The cryptographic pipeline is ternary throughout.

**Permutation counts (4 KB payload):**

| Mode | Domain | Keystream | MAC absorb | MAC squeeze | Total |
|------|--------|-----------|------------|-------------|-------|
| v2 (standard) | 1 | 101 | ~83 | 1 | ~186 |
| v3 T-AE-MAC | 1 | 51 | ~83 | 1 | ~136 |
| v3 bulk (mac=0) | 1 | 51 | 0 | 0 | ~52 |

v3 T-AE-MAC: **1.37× fewer** permutations than v2. v3 bulk: **3.6× fewer**.

**Auto-gated parallelism**: rayon multi-core gated on two conditions: (1) `rayon::current_num_threads() > 1`, AND (2) payload ≥ PARALLEL_THRESHOLD (1,944 trits = 8 rate blocks ≈ 324 bytes). Primary/secondary phases run in parallel via `rayon::join`. Keystream segments squeezed in parallel across cores. GF(3) cipher applied in `par_chunks_mut` (1024-trit blocks). When single-threaded or below threshold: same v3 construction, zero thread pool overhead.

**Security bounds (TM-2026-011):**

| Claim | Bound | Condition |
|-------|-------|-----------|
| IND-CPA (authenticated) | q²/2²⁵⁶ + (σ+q)²/2³⁸⁵ + q·ε_perm | mac_trit_count > 0, fresh nonce |
| IND-CPA (bulk) | q²/2²⁵⁶ + (σ+q)²/2¹⁹² + q·ε_perm | mac_trit_count = 0, fresh nonce |
| INT-CTXT (243-trit tag) | q_f/3²⁴³ + (σ+q+q_f)²/2³⁸⁵ | Full 486-trit capacity preserved |
| Post-quantum (MAC) | ≥ 2¹⁹²·⁵ (Grover) | All modes with mac > 0 |
| Post-quantum (keystream) | ≥ 2⁹⁶ (Grover on bulk capacity) | Bulk-rate keystream |
| Nonce-misuse | Tag still commits to ciphertext | Reused nonce leaks Δplaintext |
| Committing | (domain, ciphertext, tag) commits to plaintext | Capacity prevents collisions |

⚠️ **WARNING**: When `mac_trit_count = 0`, T-AE-MAC provides encryption only — no authentication tag. Ciphertext integrity MUST be provided by an outer mechanism (TL-DSA signature). Callers MUST NOT assume ciphertext integrity when `mac_trit_count = 0`.

Source: `server/crypto/sponge-hash.ts`, `ternary-math/napi/src/lib.rs`, `docs/TM-2026-011.md`

### 6.11 CNSA 2.0 Compliance

11/11 algorithms implemented. AES-256, SHA-384/512, ML-KEM ×3 (via TL-KEM), ML-DSA ×3 (via TL-DSA), LMS (Ternary Lamport OTS), XMSS (partial — Merkle tree planned 2029). CMVP target: FIPS 140-3 Level 1. Compliance page: `client/src/pages/compliance.tsx`

### 6.12 ZK Proof Layer (SignHere)

Groth16-structured proofs (pi_a, pi_b, pi_c) with commitments and nullifiers from document hashes, tenant IDs, signer counts, HPTP timestamps. Source: `sign-here/server/services/zk.ts`

---

## 7. Rust Kernel Subsystems (`src/kernel/`)

| Subsystem | ~LOC | Key Content |
|-----------|------|-------------|
| **crypto/** | 24,231 | 39 files — TL-DSA, TL-KEM, AES-256, SHA-2/3, sponge, CAVP, side-channel, formal_verify, metatronic_cube (2,324), ternary_lattice (1,820) |
| **vm/** | 7,314 | 14 files — **176-opcode ISA v2.1**, 27 regs, 64-bit, quantum-ternary opcodes (0xA0–0xAF), engine (2,111), instruction_v2 (1,159), HptpProvider trait |
| **network/** | 3,920 | 8 files — torus (710), routing, T3P (461), TDNS (531), TTP (775), cnsa_profiles, metatronic_bridge |
| **arch/** | 3,273 | x86_64, aarch64, RISC-V 64 boot sequences |
| **security/** | 2,747 | Audit logging, capabilities, security domains, MAC policy |
| **process/** | 1,943 | Context switching, IPC, scheduler, process table |
| **compat/** | 1,711 | Binary↔ternary adapter, CryptoInteropBridge, gateway |
| **drivers/** | 1,578 | Femtoclock driver, TPU driver |
| **hptp/** | 1,687 | 5 files — certification, jitter_correction, optical (311), protocol (592) — 7 clock sources, 5 precision levels |
| **device/io/memory/fs/sync** | ~6,376 | device (1,420), io (1,393), memory (1,378), fs (1,209), sync (976) — phase_mutex |

Top-level kernel files: `ternary.rs` (974), `timing.rs` (433), `phase.rs` (362), `error.rs` (218), `lib.rs` (156).

### 7.1 VM ISA v2.1 (176 opcodes)

Backward-compatible: v1.0 (62) → v2.0 (160) → v2.1 (176). Added quantum-ternary simulation (0xA0–0xAF), atomics, SIMD, lattice crypto accel (ML-KEM/ML-DSA), FINRA audit logging, capability control. Mark-sweep GC, τ-derived constants. `HptpProvider` trait with `SimulatedHptp` and `LiveHptp`, 8-selector `ReadTime`.

### 7.2 Bare-Metal Kernel Validation

`src/kernel/bare-metal/`: 45+ self-tests exercising real kernel code (GF(3) arithmetic, boot sequences, femtosecond timing, phase encryption, VM components). Kani Rust Verifier + MIRI formal verification pipeline with 38 proof harnesses. GitHub Actions CI. Includes `generator_theorem_harness.rs` (288 lines) — exhaustive formal verification that step `a` generates Z_m iff gcd(a, m) = 1 for all framework moduli {13, 27, 28, 54, 364}, with CRT product group verification, repunit circle generator checks, and Euler totient validation. Confirms 13 does NOT generate Z₃₆₄ (since 364 = 13 × 28) and documents step 11 as a valid Z₃₆₄ generator.

---

## 8. Blockchain Witnessing & SFK Operations

### 8.1 Hedera HCS Witnessing

Submits cryptographic witness hashes to a Hedera Consensus Service topic:
- Immutable, ordered, timestamped proof of PlenumNET operations
- Witness types: MERKLE_ROOT_BATCH, SINGLE_HASH, AGGREGATE_PROOF
- Hash algorithms: SHA256, SHA384, SHA512, KECCAK256
- Ternary context (security_mode, phase_offset, torsion_dimensions) attached to each witness

Source: `server/salvi-core/blockchain-integrations.ts`, `server/services/hedera-witnessing-service.ts`

### 8.2 SFK Operations Pipeline

Manages Salvi Framework Kernel operation lifecycle:

```
initialization → ternary_processing → witnessing → settlement → finalization
```

Operation types: TERNARY_BATCH_PROCESSING, PHASE_ENCRYPTION, TORSION_ROUTING, WITNESS_SUBMISSION, SETTLEMENT_EXECUTION

Fortified-tier operations submit SHA-256 result hashes to Hedera HCS for non-repudiation.

Source: `server/salvi-core/sfk-operations-api.ts`, `server/services/sfk-operations-service.ts`

### 8.3 Additional Blockchain Integrations

- **XRPL** (XRP Ledger): `services/blockchain/xrpl-service/`
- **Algorand**: `services/blockchain/algorand-service/`, `contracts/algorand/`

---

## 9. Tonal Diffusion System

Network-wide time synchronization using FM timing packets:
- Toroidal topology
- Gradient-driven diffusion consensus
- FM Timing Engine (Rust-backed)
- Tonal Field Service with resonance detection

Source: `services/tonal-field/` — TonalField, DiffusionSolver, PlenumMetrics

---

## 10. Kong Konnect Gateway (33 services, 293 endpoints)

Config: `kong/`. Frontend: `client/src/pages/kong-konnect.tsx`. Both `getPlenumnetServices()` and `/api/kong/service-catalog` are synchronized.

| Category | Services | Endpoints |
|----------|----------|-----------|
| Core | 16 | 153 |
| Security | 3 | 83 |
| Tools | 2 | 13 |
| Reference | 3 | 9 |
| Platform | 6 | 12 |
| Admin | 3 | 23 |
| **Total** | **33** | **293** |

Key services: TSA (9), Hedera (6), SFK Operations (5), Capabilities (29), Inter-Cube (18), TDNS (11), Security Infrastructure (38), API Keys (16), Tribonacci (15), Ephemeris (4), GDPR (4), Tonal Diffusion (3), Resonance Detector (3), Entrainment (5), Metrics (1). Phase Encryption: 6 endpoints (batch split/recombine).

---

## 11. Security Middleware & API Infrastructure

### 11.1 Security Middleware Stack

| Layer | What | Why |
|-------|------|-----|
| 4-tier rate limiting | Tiered by endpoint sensitivity | Prevents abuse without blocking legitimate use |
| CORS | Origin-restricted | Prevents cross-site attacks |
| Helmet.js | Security headers (CSP, HSTS, etc.) | Browser security hardening |
| AES-256-GCM | Token encryption at rest | Secrets never stored plaintext |
| Null-byte stripping | Input sanitization | Prevents null-byte injection |
| Double URL-decode | Path sanitization | Prevents encoding-based traversal |
| execFile() only | No shell spawning | Eliminates shell injection entirely |

### 11.2 API Key Management

- Generation, validation, rotation
- Per-key rate limiting
- Audit trails with anomaly detection
- WBS (Work Breakdown Structure) tagging system

Source: `server/services/api-key.service.ts`, `server/routes/api-keys.ts`

### 11.3 Security Infrastructure Services

Admin-protected backend services:
- Security Audit Service (`server/services/security-audit.service.ts`)
- HPTP Anomaly Detection (`server/services/hptp-anomaly.service.ts`)
- Threat Model Registry (`server/services/threat-model.service.ts`)
- Implementation Status Tracker (`server/services/implementation-status.service.ts`)

---

## 12. PlenumDB

Ternary-encoded data storage demonstrating the 58.5% information density advantage. Live compression demo with benchmark validation endpoint.

Source: `client/src/pages/ternarydb.tsx`, `compression.tsx`

---

## 13. SignHere / SalviSign

Live at `SignHere.replit.app`. Full PlenumNET crypto pipeline (`sign-here/server/services/plenum.ts`): `secureDoc()` → phase-encrypt, `witnessSign()` → XRPL attestation, `getHPTP()` → femtosecond timestamp, `mlDsaSign()` → TL-DSA signature, `cnsa2SecureDocument()` → full CNSA 2.0 pipeline.

Design system: `#090807` bg, `#D4A017` gold, `#E4DFD5` fg, Inter 13px, shadcn-style flat components, `#059669` emerald success.
Live TSA fields: `stampResult.hptpTimestamp`, `.calendarContext.calendars`, `.merkleLeafHash`, `.accuracy`.

---

## 14. Quantum Ternary Modules

Five shared modules provide classical simulation of quantum ternary operations:

| Module | What | Why |
|--------|------|-----|
| `qutrit-basics.ts` | Qutrit (d=3) states, Gell-Mann generators, SU(3) unitaries | Core quantum ternary simulation |
| `qudit-basics.ts` | Generalized qudit (d≥2), shift/clock operators | Higher-dimensional extension |
| `lagrangian-qutrit-utils.ts` | Lagrangian evolution for qutrits | Time evolution simulation |
| `lagrangian-ternary-utils.ts` | Ternary-specific Lagrangian mechanics | Framework-coupled evolution |
| `qutrit-fault-tolerance.ts` | Qutrit error correction codes | Fault-tolerant operations |

Supporting: `complex-utils.ts` (complex arithmetic), `hamiltonian-constraints.ts`, `noether-symmetries-utils.ts`, `tribonacci-variational.ts`

---

## 15. 28-Dimension Agent Array

Maps Z₂₈ cyclic positions to 28 parallel AI agents:

- **Scheduling**: `(position × 13) mod 28` visits all 28 positions exactly once
- **Why coprime walk**: gcd(13, 28) = 1 guarantees complete coverage, bias-free
- **Walk sequence**: 0 → 13 → 26 → 11 → 24 → 9 → 22 → 7 → 20 → 5 → 18 → 3 → 16 → 1 → ...
- **Convolution kernel**: [13, 24, 44] = [T₇, T₈, T₉] (three consecutive Tribonacci numbers)
- **Identity**: 13 × 28 = 364 = `111111₃`

### 15.1 Multi-Generator Scheduling

All 12 generators (units) of Z₂₈: {1, 3, 5, 9, 11, 13, 15, 17, 19, 23, 25, 27}. φ(28) = φ(4) × φ(7) = 2 × 6 = 12.

Self-inverse generators (g = g⁻¹): {1, 13, 15, 27}. Inverse pairs: 3↔19, 5↔17, 9↔25, 11↔23.

The multi-generator module supports parallel schedule assignments for fault-tolerant monitoring: different generators produce different walk orders visiting the same 28 agents, enabling redundant heartbeat coverage. The reverse walk uses the multiplicative inverse generator.

Features: Etymology Audit, Veritas Fact-Check, unified Situation Report, Lexical Protocol enforcement.

Source: `shared/agent-array.ts`, `shared/agent-generators.ts` (184 lines), `client/src/pages/agent-array.tsx`

---

## 16. XPlenum RISC-V Hardware Extension

Custom RISC-V extension integrated with CVA6, using two opcode spaces:
- **custom-0** (0x0B / `7'b0001011`): Core ternary operations
- **custom-1** (0x2B / `7'b0101011`): Phase 8 PQC acceleration
- 6 functional groups (funct3): F3_TMASK (ternary masking), F3_TDOM (domain isolation), F3_TCAP (capability ops), F3_TROT (ternary rotation/crypto), F3_TENC (trit encode/decode), F3_TSIG (signal processing)
- 22/22 tests passing. Yosys synthesis: 19,173-cell gate-level netlist

Verilog RTL modules (`XPlenum/rtl/` — 19 files):

| Module | Purpose |
|--------|---------|
| `xplenum_top.v` | Base top-level module (32-bit) |
| `xplenum_top_v2.v` | Phase 8 top-level integration (64-bit) |
| `xplenum_pkg.vh` | Opcode/group definitions header |
| `xplenum_trit_unit.v` | Ternary rotation and encoding unit |
| `xplenum_aes256_core.v` | AES-256 hardware core |
| `xplenum_pqc_unit.v` | Post-quantum crypto accelerator (Phase 8) |
| `xplenum_cap_unit.v` | Capability-based security unit |
| `xplenum_ctr_drbg.v` | NIST SP 800-90A CTR_DRBG PRNG |
| `xplenum_mask_unit.v` | Side-channel masking countermeasures |
| `xplenum_dom_gadgets.v` | Domain-oriented masking (DOM) gadgets for higher-order security |
| `xplenum_tamper_response.v` | Security lockdown and zeroization logic |
| `xplenum_domain_unit.v` | Domain isolation enforcement |
| `xplenum_crt_unit.v` | CRT decomposition pipeline (330 LOC, 5-stage: Z₃₆₄ → Z₂₈ × Z₁₃) |

The CRT unit implements hardware-pipelined calendar-position decomposition at 200 MHz FPGA clock: quarter-phase at 0 ns (wire), clock source index at 5 ns (stage 1), day-within-moon at 10 ns (stage 2), moon sector at 15 ns (stage 3), full position at 20 ns (stage 4). Pipeline data-alignment bug found and fixed during simulation: initial design read stage-3 registers from stage-4 output encoder, causing 1-cycle misalignment. Fixed by merging all outputs into a single registered block driven by s3_valid. Structural alignment: 364 = 7 × 52, so all 7 clock sources are hit exactly 52 times each — perfectly uniform, zero bias.

Integration (`rtl/integration/`): `xplenum_cva6_top.v`, `xplenum_cva6_wrapper.v`, `xplenum_cva6_wrapper_v2.v`, `xplenum_stall_controller.v`

Formal verification (`rtl/formal/`): `xplenum_formal_props.v`, `xplenum_induction_helpers.v`, `xplenum_integration_formal_props.v`, `xplenum_crt_formal_props.v` (202 LOC — SymbiYosys BMC + induction), `xplenum_crt_formal.sby` (25 LOC)

Testbenches (`tb/`): `xplenum_tb.v`, `xplenum_drbg_tb.v`, `xplenum_cva6_integration_tb.v`, `xplenum_fault_inject_tb.v`, `xplenum_crt_tb.v` (376 LOC — exhaustive 364-point round-trip verification)

Benchmarks (`benchmarks/`): `crt_bench.c` (261 LOC — raw throughput), `crt_bench_v2.c` (162 LOC — honest latency analysis). On x86 software, GCC -O2 optimizes constant-modulo to multiply-shift (no throughput advantage). The real advantage is in FPGA hardware pipelining where coarse routing decisions arrive before full computation completes.

---

## 17. Standalone Crates & Libraries

**plenumlan/** — PlenumLAN Node crate. Array3 cluster pipeline: 27 classification trits → 3-trit slot → port → registration. 11 modules under `src/cube/`: constants, projection, port, bridge, routing, node, sentinel, freshness, scan_templates, entity_detect, windows_roles. Depends on `ternary-math` and `tdns-v2`. Version 2.4.1. See §3.2.6 for full documentation.
**libternary/** — Core ternary Rust lib, `cdylib` + WASM (`wasm-bindgen`). TritVec with Rep A/B/C conversions.
**libternary-improvements/** — Enhancement staging area.
**ternary-math/** — standalone crate, 12 modules: gf3, gf3_algebra (94 LOC — division-free GF(3) closed-form algebra, zero sponge code), tribonacci, borromean, clifford, torus, ternary_circle, tis_sponge (290 LOC — TIS-27: 54-trit, 4 rounds, 7-neighbor extended theta at ±1/±7/±13, ~303 ns/hash; also defines TIS-81 parameters: 243-trit, 4 rounds, benchmark only), radix, constants, repunit_checksum (200 LOC), repunit_circles (132 LOC). Plus integration tests (210 LOC). TypeScript mirrors: `shared/gf3-algebra.ts` (77 LOC), `shared/tis-sponge.ts` (77 LOC).

**benchmarks/** — comprehensive benchmark suite:
- `c-bench/pipeline_v2.c` (192 LOC): TIS-27 vs SHA-256 honest pipeline — raw input → routable address
- `c-bench/plenum_full.c` (394 LOC): complete platform benchmark — 40 tests across 12 categories
- `c-bench/tis81_simd.c` (262 LOC): TIS-81 SIMD vs SHA3-256
- `c-bench/xplenum_sim.c` (221 LOC): XPlenum cycle-accurate hardware simulation vs SHA-NI
- `rust-bench/` (Cargo.toml + 3 source files, 735 LOC): full Rust benchmark against sha2, sha3, blake2, aes-gcm, hkdf crates
**wasm/** — 412 LOC browser deployment target.
**Ternary Ephemeris** — `TERNARY_EPHEMERIS_INTEGRATION_GUIDE.md`

Testing infrastructure: Criterion benchmarks (395 LOC), fuzz targets (330 LOC: `fuzz_gateway`, `fuzz_trit_ops`, `fuzz_tryte_ops`), PropTest VM verification (348 LOC).

Test totals: **2,276** (1,783 Rust #[test] + 493 TypeScript). Single source of truth: `shared/constants.ts` TESTS_PASSING.

---

## 18. Repository Structure (82 top-level items, verified)

```
/                              Root (Express.js + Vite full-stack app)
├── .github/                   CI workflows (ci, security-scan, license-check, owasp, codeql)
├── XPlenum/                   RISC-V hardware extension (Verilog, 19K-cell netlist)
├── attached_assets/           Static assets
├── cli/plenum-stamp/          RFC 3161 CLI tool
├── client/src/pages/          26 React pages (landing→whitepaper)
├── client/src/components/     Shared components (sidebar, footer, nav, VM terminal, ...)
├── contracts/                 Smart contracts (Algorand)
├── deployments/               🚫 DO NOT MODIFY 🚫
├── docs/                      Architecture, security, legal docs (docs/legal/INDEX.md)
├── github-push/               Push automation scripts
├── keys/                      Key material
├── kong/                      Kong Konnect gateway (33 services, 293 endpoints)
├── libternary/                Core ternary library (Rust, cdylib + WASM)
├── libternary-improvements/   Enhancement staging
├── plenumlan/                 PlenumLAN Node (Rust, Array3 cluster pipeline, 11 modules)
├── salvi_docs/specs/          Specifications (15 modules, 7,300+ lines)
├── script/ + scripts/         Build/utility scripts
├── server/                    Express.js backend
│   ├── routes.ts + routes/    API routes (tsa, security, kong, hedera, capabilities, ...)
│   ├── services/              Backend services (TSA, capability, security audit, ...)
│   ├── salvi-core/            Core framework (ternary ops, phase encryption, timing, blockchain)
│   ├── crypto/                TL-DSA bridge
│   ├── storage.ts             IStorage interface + DatabaseStorage
│   └── crypto-utils.ts        AES-256-GCM token encryption
├── services/                  8 microservices
│   ├── tdns-v2/               TDNS v2.5 (Rust, 19 modules + 2 binaries)
│   ├── inter-cube/            Inter-Cube Infrastructure (Rust, 18,649 LOC, 26 modules, 422 tests)
│   ├── blockchain/            Hedera, XRPL, Algorand services
│   ├── payment-listener/      Payment processing
│   ├── sfk-core-api/          SFK Operations Pipeline
│   ├── pqti-service/          Post-Quantum TLS Inspection (Rust)
│   ├── timing/                Femtosecond + Certification services
│   └── tonal-field/           Tonal Diffusion System
├── shared/                    Shared TypeScript modules
│   ├── constants.ts           PLATFORM object (SINGLE SOURCE OF TRUTH)
│   ├── schema.ts              Drizzle DB schema + Zod insert schemas
│   ├── topology/              Toroidal addressing, GF(3) operations
│   └── [math modules]         Circle, Tribonacci, Saturnian, quantum, agents
├── sign-here/                 SalviSign e-signature platform
├── src/kernel/                Rust kernel (123 files, ~54,780 LOC)
│   ├── src/                   Main kernel (crypto, vm, network, arch, security, process, ...)
│   ├── bare-metal/            Bare-metal validation (45+ tests, Kani/MIRI, 38 proofs)
│   └── ISA_REFERENCE.md       176-opcode ISA v2.1 reference
├── ternary-math/              Standalone math crate (6,195 LOC, 12 modules + TIS-27 sponge)
├── tests/                     TypeScript test suites (295 tests, 10 suites)
├── wasm/                      Browser deployment target (412 LOC)
└── [config files]             Cargo.toml, package.json, Dockerfile, docker-compose.yml, etc.
```

**26 Client Pages** (verified): about, admin, agent-array, api-demo, api-keys, calendar, compliance, compression, contact, distribution, docs, fpga-benchmarks, github-manager, hptp-demo, isa-security-paper, kong-konnect, landing, legal, not-found, quantum-sim, ternarydb, thirteen-moon, tribonacci-28ds, tsa, vm-demo, whitepaper.

### 18.1 PLATFORM Constants (Single Source of Truth)

ALL numeric values used in the frontend and docs MUST come from `shared/constants.ts` → `PLATFORM` object. Never hardcode numbers. If you need a new constant, add it to PLATFORM.

---

## 19. Development Rules & Conventions

### 19.1 Absolute Rules (CANNOT BE BROKEN)

1. 🚫 **NEVER modify `deployments/` folder** — user-enforced constraint
2. 🚫 **NEVER hardcode numbers** — use PLATFORM constants
3. 🚫 **NEVER use 0 as a trit value** in TDNS (Rep C only)
4. 🚫 **NEVER replace geometric routing** with routing tables
5. 🚫 **NEVER change the 364°/π=14 circle** to 360°/π≈3.14
6. 🚫 **NEVER change the derivation formula** — no tuning parameters
7. 🚫 **NEVER use ed25519** — TL-DSA everywhere, no exceptions
8. 🚫 **NEVER feed raw binary into ternary sponge** — decompose via `u16_to_trits()`/`u8_to_trits()` first
9. 🚫 **NEVER simulate TL-DSA** — call the real Rust implementation
10. 🚫 **NEVER change the Salvi Epoch** (2025-04-01T00:00:00Z)
11. **All source files include copyright headers** (Capomastro Holdings Ltd.)
12. **All constants** MUST come from PLATFORM — no inline magic numbers

### 19.2 Frontend Conventions

- React + TypeScript + Tailwind CSS + shadcn/ui + Wouter + Framer Motion
- Light/dark mode support (class-based)
- `data-testid` on all interactive and meaningful elements
- TanStack Query v5 (object form only)
- `@assets/` import prefix for attached assets

### 19.3 Backend Conventions

- Express.js + Drizzle ORM + PostgreSQL
- IStorage interface pattern for all CRUD
- Zod validation on request bodies
- Tiered rate limiting, CORS, Helmet.js

### 19.4 Rust Conventions

- TDNS: `edition = "2021"`, pinned deps
- `thiserror` for error types, `serde` with derive for serialization
- Constant-time GF(3) operations in crypto modules
- All modules re-exported from `lib.rs`
- `no_std` math: Use `libm::log2()`, `libm::ceil()`, `libm::fabs()` — NOT f64 methods. Fixed in `metatronic_cube.rs`, `ternary_lattice.rs`, `cross_impl.rs`, `gateway.rs`

### 19.5 GitHub Push Convention

Push to `SigmaWolf-8/Ternary@main` using GitHub Contents API via `GITHUB_TOKEN` env secret. Use bash + curl, NOT the code_execution sandbox.

### 19.6 CI Pipeline

5-stage gated (theory-validation.yml): GF(3) axioms + bijective check + clippy → verifyTau() + GF(3^k) → 13D state vectors + HPTP + determinism → Lamport + NTT + Merkle → full-stack build + integration. Plus: security-scan, license-check, OWASP, CodeQL.

---

## 20. Key Mathematical Identities (Quick Reference)

```
3²⁷ × 9 = 68,630,377,364,883     (TDNS address space — 68.63 trillion)
3¹³ = 1,594,323                   (13D hypercube vertices)
26 × 3¹³ / 2 = 20,726,199        (unique PQ-encrypted tunnels per populated cube)
13! = 6,227,020,800               (max shortest paths at Hamming distance 13)
(3⁶ − 1) / 2 = 364               (full ternary circle, base-3 repunit)
364 / 28 = 13                     (1 radian = T₇)
13 × 28 = 364                     (13 moons × 28 days = ternary year)
gcd(13, 28) = 1                   (coprime — complete Z₂₈ walk)
τ³ = τ² + τ + 1                   (Tribonacci constant defining polynomial)
T₇ = 13                          (7th Tribonacci number = 1 radian = 111₃)
111₃ = 13                        (three-digit base-3 repunit)
111111₃ = 364                    (six-digit base-3 repunit = full circle)
log₂(3) ≈ 1.585                  (59% density advantage of ternary)
333 = 3 × 111                    (Saturnian magic constant)
360 − 333 = 27                   (phase dissonance = TDNS dimensions)
27 + 1 = 28 = 2π                 (dissonance closure = cyclic order)
208 / 16 = 13                    (Saturnian cosmic radius = T₇)
```