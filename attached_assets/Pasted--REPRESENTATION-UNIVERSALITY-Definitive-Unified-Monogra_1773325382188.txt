# REPRESENTATION UNIVERSALITY

## Definitive Unified Monograph with Completed Proofs

### The Dual-Circle Architecture: Z₂₇ and Z₂₈ as Coprime Companions

**Discrete Isometry · Minimum Distance · Automorphisms · Spectral Theory · Performance Benchmarks · Formal Verification · Three-Factor Security**

---

**Technical Monograph — TM-2026-008 (Version 11)**
**Salvi Framework — PlenumNET Architecture Series**
**Supersedes TM-2026-004 through TM-2026-008 v10**
**March 2026**

**Capomastro Holdings Ltd. — Applied Physics Division**
**Sherwood Park, Alberta, Canada**

© 2026 Capomastro Holdings Ltd. — All Rights Reserved — Patent(s) Pending

Cross-References: TM-2026-011 (TLSponge-385 Security) • TM-2026-014 (Inter-Cube Infrastructure — From First Principles)

---

## Abstract

This monograph is the definitive formalisation of the Representation Universality Thesis for the Salvi Framework. It supersedes TM-2026-004 through TM-2026-008 v10, consolidating all three proof programs to COMPLETE status and incorporating **performance benchmarks, formal verification infrastructure, and the Three-Factor Security formalisation** added in this version.

The primary isometry theorem embeds Z₃ⁿ into the discrete algebraic circle Z₂₇ⁿ with scale factor 9 = 3², using no transcendental constants. The entire constant system derives from a single unified quadratic (arc² − 832·arc + 118,300 = 0, root arc = 182), from which π = 14, the kernel sponge width 729 = 3⁶, and both circles Z₂₇ and Z₂₈ cascade through exact integer arithmetic. Two discrete circles coexist as coprime companions: Z₂₇ (algebraic, hosting the character embedding and trit group) and Z₂₈ (geometric, hosting π = 14, the ternary radian, and calendar cycles), with gcd(27, 28) = 1 and 27 + 1 = 28.

Program II (Minimum Distance) is COMPLETE: branch number B(Mθ) = 8 proven exactly by combined primal-dual exhaustive computation (5,270,004 vectors). The chi S-box is specified as χ(x) = x¹⁷ over GF(27), with verified DP_max = 3/27 = 1/9 — optimal among all power-map permutations. The wide-trail bound N(r) ≥ 8ʳ is unconditional, yielding differential trail probabilities ≤ (1/9)⁴⁰⁹⁶ < 10⁻³⁹⁰⁸ for r = 4 rounds.

**Version 11 Additions:** (1) TIS-27 benchmark data: **191 ns/hash, 3.5× faster than SHA-256**, validated as sole cryptographic primitive across the entire stack (84/84 end-to-end tests). (2) TLSponge-385 SIMD optimisation with AVX2/NEON acceleration and bulk-rate squeeze (cross-ref TM-2026-011). (3) T-AE-MAC v3 dual-phase authenticated encryption: **1.37× fewer permutations, bulk mode 3.6× fewer**. (4) TL-DSA benchmark: **2,470 μs at level 87, 5.9× speedup**. (5) Binary Decomposition Mandate formalised as Theorem 3.9. (6) Generator Theorem for all framework moduli proven via Kani. (7) Bare-metal kernel validation (38 proof harnesses, MIRI). (8) Three-Factor Capability Security formalised in Part VI. (9) Discrete Torus Spectral Theory retained from v10 as Part V.

### Program Status Summary

| Program | Status | Key Result | Method |
|---------|--------|------------|--------|
| **I: Isometry** | COMPLETE | d₁(ι(u),ι(v)) = 9·d_H(u,v) | Elementary proof |
| **II: Min Distance** | COMPLETE | B=8, χ(x)=x¹⁷, DP=1/9, N(r)≥8ʳ | Primal+dual exhaustive (5.27M vectors) |
| **III: Automorphisms** | COMPLETE | Aut ≅ (S₃)²⁵ × (C₂)² | Elementary proof |
| **IV: Benchmarks** | NEW (v11) | TIS-27: 191 ns; TL-DSA: 5.9×; T-AE-MAC: 1.37× | Compiled Rust, production hardware |
| **V: Spectral Theory** | COMPLETE | DFT on Z₂₇ⁿ, radix-3 FFT | Elementary construction |
| **VI: Three-Factor** | NEW (v11) | Geometric × Temporal × Hardware security | Structural proof from invariants |

---

## 1. Introduction

### 1.1 The Dual-Circle Architecture

PlenumNET rests on two discrete circles serving fundamentally different roles:

| Property | Z₂₇ (Algebraic Circle) | Z₂₈ (Geometric Circle) |
|----------|------------------------|------------------------|
| **Order** | 27 = 3³ | 28 = 2π (ternary) |
| **Role** | Character theory, trit embedding, DFT | Angular measure, radian, calendar |
| **Divisible by 3?** | Yes (27/3 = 9) | No (gcd(3,28) = 1) |
| **Trit embedding** | {0, 9, 18} equally spaced | Not applicable |
| **Framework instances** | TDNS dims (27), sponge capacity, TIS-27 state (54=2×27) | π=14, circle=364°=111111₃, calendar (13×28) |
| **Coprime relation** | gcd(27, 28) = 1 | gcd(28, 27) = 1 |

Their coprimality is structurally necessary: by the Chinese Remainder Theorem, Z₇₅₆ ≅ Z₂₇ × Z₂₈ (since 756 = 27 × 28 and gcd(27, 28) = 1). The identity 27 + 1 = 28 is the arithmetic bridge — the algebraic circle plus one yields the geometric circle, the same "+1" that appears as the confidence dimension extending 27 ontological trits to 28 effective dimensions.

### 1.2 The Representation Universality Thesis

> Certain geometric objects — principally the n-torus Tⁿ and its discrete counterpart Z₂₇ⁿ — appear universally in any system that encodes discrete states with phase relationships over a finite abelian group. This universality is a theorem of Pontryagin duality. The Clifford torus is the simplest non-trivial continuous case (n = 2). Its discrete counterpart is the lattice Z₂₇ × Z₂₇ with trit values at positions {0, 9, 18}.

### 1.3 Codebase Context (v11)

This monograph formalises the mathematical foundations of a **deployed system**: 123,000+ lines of production code across Rust, TypeScript, and Verilog. 1,252+ commits, 80/80 milestones, 2,276 passing tests (1,783 Rust + 493 TypeScript), 34 cryptographic modules (24,231 LOC), 38 formal proof harnesses (Kani/MIRI), and 22/22 SymbiYosys hardware proofs. The cryptographic stack has **zero external lineage** — no SHA-256, no BLAKE3, no Ed25519. TIS-27 serves as the sole cryptographic hash primitive across the entire stack, validated by an 84/84 end-to-end test suite following the complete BLAKE3/SHA-256 removal.

### 1.4 The Unified Quadratic — Generative Root

The entire framework flows from a single quadratic equation rooted in base-3 repunits:

```
arc² − 832 · arc + 118,300 = 0
```

Its meaningful root, **arc = 182**, is the semicircle of the ternary circle (364°/2). From this one number, every operational constant cascades through exact integer arithmetic — no floating point, no approximation, no tuning parameters.

**Theorem 1.1** *(Unified Quadratic derivation).* The discriminant of the unified quadratic is 832² − 4(118,300) = 692,224 − 473,200 = 219,024. Since 219,024 = 4 × 54,756 and √219,024 = 468 (exact integer), the roots are (832 ± 468)/2 = {650, 182}. The physically meaningful root is **arc = 182** (the semicircle).

**Constant cascade from arc = 182:**

| Constant | Value | Derivation from arc = 182 |
|----------|-------|---------------------------|
| Full circle | 364° | 2 × 182 = 364 = 111111₃ (six-digit base-3 repunit) |
| Ternary π | 14 | (1 + √(1 + 4·182)) / 2 = (1 + √729) / 2 = (1+27)/2 = 14 |
| 2π | 28 | 2 × 14 = 28 = Z₂₈ cyclic order, lunar-solar harmonic |
| 1 radian | 13° | 364/28 = 13 = T₇ = 111₃ (three-sequence convergence) |
| Kernel sponge | 729 | 1 + 4·182 = 729 = 3⁶ (secondary discriminant) |
| Centre | 111 | (182 + 40)/2 = 111 = Plenum Square centre |
| Magic constant | 333 | 3 × 111 = perimeter of inscribed hexagon |
| Phase dissonance | 27 | 360 − 333 = 27 = 3³ = TDNS dimensions = Z₂₇ order |
| Calendar | 364 | 13 × 28 = 13 moons × 28 days = full circle |
| Circumference | 1554 | πd = 14 × 111 = 28 × (111/2) = 2πr (exact integer) |

The secondary discriminant 1 + 4·182 = **729 = 3⁶** yields the kernel sponge state width directly. The value π = 14 = T₇ + T₃ = 13 + 1 links the Tribonacci sequence to the ternary circle. The algebraic circle Z₂₇ (order 27 = 360 − 333 = phase dissonance) and geometric circle Z₂₈ (order 28 = 2π) are **both derivable from this single equation**. Every constant used in this monograph traces back to arc = 182.

### 1.5 Notation

- **GF(3)** = Z₃ = {0,1,2} under addition modulo 3.
- **Rep C** = {1,2,3} via φ(k) = k+1. Zero structurally excluded.
- **ω** = e^(2πi/3) (cube root of unity, continuous corollary only).
- **ω₂₇** = e^(2πi/27) (primitive 27th root, spectral theory).
- **GF(27)** = GF(3)[t]/(t³+2t+1).
- **Cyclic distance** on Z_n: δ(a,b) = min(|a−b|, n−|a−b|).
- **L(p; q₁,…,qₙ)** = generalised lens space S^(2n−1)/Z_p.

---

## Part I: The Isometry Theorem (Program I)

**Status: COMPLETE.** All proofs are elementary.

### 2.1 Discrete Setting

The ternary hypercube Z₃ⁿ has Hamming distance d_H(u,v) = |{i : uᵢ ≠ vᵢ}|. The target is Z₂₇ⁿ with ℓ¹ cyclic product metric d₁(a,b) = Σᵢ δ(aᵢ, bᵢ).

**Definition 2.1** *(ℓ¹ cyclic product metric on Z₂₇ⁿ).* For a, b ∈ Z₂₇ⁿ: d₁(a, b) = Σᵢ δ(aᵢ, bᵢ), where δ(aᵢ, bᵢ) = min(|aᵢ − bᵢ|, 27 − |aᵢ − bᵢ|) is the cyclic distance on Z₂₇.

### 2.2 The Discrete Character Embedding

**Definition 2.2** *(Discrete character embedding).* The map ι: Z₃ⁿ → Z₂₇ⁿ is defined by ι(v₁,…,vₙ) = (9v₁,…,9vₙ), where multiplication is in Z₂₇. This places trit values at positions {0, 9, 18} in each Z₂₇ factor.

The multiplier 9 = 3² is the unique value that produces equal spacing of three points in Z₂₇. Since 27/3 = 9, the three images {0, 9, 18} partition Z₂₇ into three arcs of length 9. This is not a design choice — it is forced by equal trisection.

### 2.3 The Constant Separation Lemma

**Lemma 2.3** *(Constant discrete separation).* For any two distinct elements a, b ∈ Z₃, the cyclic distance on Z₂₇ between 9a and 9b is exactly 9.

*Proof.* Let c ≡ a − b (mod 3). Since a ≠ b, c ∈ {1, 2}. δ(9a,9b) = min(9c, 27−9c). For c=1: min(9,18)=9. For c=2: min(18,9)=9. ∎

**Corollary 2.4** *(Binary separation).* δ(9a, 9b) = 0 if a = b, and 9 if a ≠ b. The separation function is binary: trits either agree (distance 0) or disagree (distance 9 = 3²). There is no intermediate value.

This constant separation is the structural engine of the entire isometry theory. It holds because 27 = 3 × 9, so three equally-spaced points in Z₂₇ are exactly 9 apart. The number 9 = 3² is not a design choice — it is forced by the requirement of equal trisection of Z₂₇.

### 2.4 The Discrete Isometry Theorem

**Theorem 2.5** *(Discrete scaled isometry).* d₁(ι(u), ι(v)) = 9 · d_H(u, v) for all u, v ∈ Z₃ⁿ. Scale factor 9 = 3². **No transcendentals.**

*Proof.* Each coordinate contributes 0 (trits agree) or 9 (trits differ). Sum = 9 · |{i : uᵢ ≠ vᵢ}| = 9 · d_H(u, v). ∎

### 2.5 The Continuous Isometry as Corollary

**Definition 2.6** *(Realisation map).* The map ρ: Z₂₇ → S¹ is defined by ρ(m) = e^(2πim/27). This is a scaled isometry: arc-length on S¹ between ρ(a) and ρ(b) equals (2π/27) · δ(a, b).

The continuous character embedding ι_c: Z₃ⁿ → Tⁿ factors as ι_c = ρⁿ ∘ ι:

```
Z₃ⁿ  ——ι——→  Z₂₇ⁿ  ——ρⁿ——→  Tⁿ
```

**Corollary 2.7** *(Continuous scaled isometry).* The composition ι_c = ρⁿ ∘ ι gives d₁(ι_c(u), ι_c(v)) = (2π/3) · d_H(u, v). The scale factor decomposes: (2π/27) × 9 = 2π/3. Standard π enters here and only here — as a property of the realisation map that converts discrete positions in Z₂₇ to angles on S¹. It is a characteristic of the bridge, not of the architecture. The ternary geometric constant π = 14, which governs Z₂₈, is **unaffected and untouched**.

*Proof.* ι_c(v) = ρⁿ(ι(v)) maps trit k to ρ(9k) = e^(2πi·9k/27) = e^(2πik/3) = ωᵏ. Arc-length d₁ on Tⁿ: Σᵢ (2π/27) · δ(9uᵢ, 9vᵢ) = (2π/27) · 9 · d_H(u, v) = (2π/3) · d_H(u, v). ∎

### 2.6 Applications

**Theorem 2.8** *(Routing as discrete geodesic descent).* Each hop moves ±9 on one Z₂₇ factor. The d₁ metric decreases by exactly 9 per hop in a minimal routing sequence. No local minima exist.

*Proof.* A trit flip at position i changes one coordinate of ι(v) by ±9 in Z₂₇. A minimal routing sequence always flips a coordinate where vᵢ ≠ tᵢ, reducing d₁ by 9. Any non-target point has at least one such coordinate, so every non-target point has a neighbour with strictly smaller d₁. ∎

**Theorem 2.9** *(Discrete key separation).* Addresses at Hamming distance d are separated by 9d on Z₂₇ⁿ. At maximum distance (d = 13): separation = 117 = 4×27 + 9 (four full circuits of Z₂₇ plus one trit-separation). At d = 27: separation = 243 = 3⁵ = 9 full circuits.

**Theorem 2.10** *(Error detection).* A t-trit error moves the embedded point by exactly 9t on Z₂₇ⁿ. Error-detection radius: ⌊(d_min − 1)/2⌋, invariant under the embedding.

### 2.7 The Pontryagin Duality Connection (via the Corollary)

The continuous corollary (2.7) connects PlenumNET to the wider mathematical landscape. Via the realisation map, Z₃ⁿ embeds into Tⁿ = (S¹)ⁿ, the Pontryagin dual of Zⁿ. The character map Z₃ → S¹ sending k ↦ ωᵏ defines the dual, and ι_c extends it to products. This is why the Clifford torus (T² ⊂ S³) appears in both quantum state spaces and ternary geometry: both use characters of Z₃ (or Z₂ for qubits) mapping into S¹.

The Hopf fibration h: S³ → S² with circle fibres realises the Bloch sphere as a quotient. The Clifford torus is the pre-image of the Bloch equator. Under the character map, the 9-point lattice of Z₃ × Z₃ sits on this same torus. The shared geometry is inevitable once both systems encode discrete states via characters of small cyclic groups into U(1). This connection is a **consequence** of the discrete isometry, not a foundation for it.

---

## Part II: Minimum Distance Bounds for TIS-27 (Program II)

**Status: COMPLETE.** Chi specification verified by exhaustive DDT computation. Branch number B(Mθ) = 8 proven exactly by primal-dual exhaustive search (5,270,004 vectors).

### 3.1 The GF(3) Affinity Barrier

> **CRITICAL RESULT:** Every permutation of GF(3) is affine. No single-trit S-box provides nonlinearity.

**Theorem 3.1** *(GF(3) Affinity Barrier).* Every bijection χ: GF(3) → GF(3) is affine (χ(x) = ax + b). DP_max = 1.

*Proof.* |AGL(1,3)| = |GF(3)\*| × |GF(3)| = 2 × 3 = 6. |S₃| = 3! = 6. Since AGL(1,3) ⊆ S₃ and both have order 6, they are equal. Every permutation is affine. For an affine map χ(x) = ax + b, the differential χ(x + Δ) − χ(x) = aΔ is independent of x. Therefore DP(Δ, aΔ) = 1 for every nonzero Δ. ∎

**Corollary 3.2** *(Grouped Chi Requirement).* The TIS-27 chi layer must operate on blocks of k ≥ 2 trits. The natural choice is k = 3 (GF(27) blocks), giving 18 blocks across the 54-trit state.

### 3.2 The Chi S-Box: χ(x) = x¹⁷ over GF(27)

GF(27) = GF(3)[t]/(t³ + 2t + 1) is the natural target: 54 trits ÷ 3 = 18 blocks. An exhaustive survey of all 12 permutation power maps (exponents coprime to 26 = |GF(27)\*|) yields:

| Exponent | Max DDT | DP_max | Alg Degree | Classification | Status |
|----------|---------|--------|------------|----------------|--------|
| 1, 3, 9 | 27 | 1 | ≤2 | Affine (Frobenius) | Rejected |
| 5, 7, 11, 15, 19, 21 | 4 | 4/27 | varies | Non-affine | Suboptimal |
| **17, 23, 25** | **3** | **1/9** | **5** | **Non-affine optimal** | **SELECTED** |

**Theorem 3.3** *(Chi specification).* χ(x) = x¹⁷ over GF(27) is a permutation (gcd(17,26)=1) with DP_max = 3/27 = 1/9, optimal among all power-map permutations. DDT values ∈ {0, 2, 3}. Algebraic degree 5 (out of max 6). Max Walsh coefficient = 9 (vs. 27 for affine, theoretical min √27 ≈ 5.2). Walsh maximum verified by exhaustive evaluation of all 729 (a,b) pairs; verification script provided as companion artifact.

*Proof.*

*Bijectivity:* gcd(17, 26) = 1, so x ↦ x¹⁷ is a permutation of GF(27)\*. DDT computed exhaustively over all 26 × 27 = 702 (a,b) pairs with a ≠ 0, counting |{x : χ(x+a) − χ(x) = b}| for each. Maximum entry = 3, occurring exactly 26 times (3.7% of entries). Remaining entries: 364 zeros (51.9%) and 312 twos (44.4%). DP_max = 3/27 = 1/9.

*Algebraic structure:* 17 = 26 − 9, so x¹⁷ = x⁻⁹ = (Frob²(x))⁻¹ (inverse of the second Frobenius). This is non-affine: x¹, x³, x⁹ are the only GF(3)-linear power maps (Frobenius endomorphisms), and 17 ∉ {1, 3, 9}. Base-3 representation: 17 = (122)₃, giving algebraic degree 1+2+2 = 5. Walsh spectrum: max |Ŵ(a,b)| = 9 over all nonzero (a,b), computed via the absolute trace Tr(x) = x + x³ + x⁹. Verified by exhaustive evaluation of all 27² = 729 Walsh coefficients; verification script provided as companion artifact.

*Optimality:* all 12 permutation exponents were checked; only {17, 23, 25} achieve max DDT = 3. The exponent 17 is preferred as the algebraically canonical choice (inverse Frobenius composition). ∎

Gold functions x^(3ⁱ+1) are inapplicable: x⁴ (i=1) and x¹⁰ (i=2) have gcd(e, 26) = 2 and are not permutations. This is a structural incompatibility with |GF(27)\*| = 26 = 2 × 13.

### 3.3 Branch Number: B(Mθ) = 8

Mθ is the circulant matrix with first-row weight 7 (offsets S = {±1, ±7, ±13} plus diagonal). Generator polynomial: g(x) = 1 + x + x⁻¹ + x⁷ + x⁻⁷ + x¹³ + x⁻¹³ ∈ GF(3)[x]/(x⁵⁴ − 1).

**Theorem 3.4** *(Branch number, exact).* **B(Mθ) = 8.**

*Proof.*

*Upper bound:* A weight-1 input at any position produces output of weight 7 (the first row of Mθ), giving sum = 8. Hence B ≤ 8.

*Lower bound:* We show wt(a) + wt(Mθ a) ≥ 8 for every nonzero a ∈ GF(3)⁵⁴, using a combined primal-dual strategy.

*Primal search (weights 1–4):* Exhaustive search over all nonzero inputs of Hamming weight 1 through 4, totalling 5,264,172 inputs over GF(3)⁵⁴. Results: weight-1 min sum = 8, weight-2 min sum = 8 (achieved at specific position pairs where two offset neighbourhoods overlap), weight-3 min sum = 11, weight-4 min sum = 12. No input of weight ≤ 4 violates the bound.

*Dual search (weights 5–6):* Since Mθ is invertible (g(1) = 7 ≡ 1 mod 3 ≠ 0, so the circulant is non-singular), any output b = Mθ a has a unique preimage a = Mθ⁻¹ b. We computed Mθ⁻¹ b for every weight-1 vector (108 vectors: 54 positions × 2 nonzero values) and every weight-2 vector (5,724 vectors: C(54,2) × 4). Results: every weight-1 output has preimage weight exactly 31; every weight-2 output has preimage weight ≥ 24 (minimum 24, maximum 44). Since 31 ≠ 5 and 31 ≠ 6, no weight-5 or weight-6 input maps to weight-1 output. Since 24 > 5, no weight-5 input maps to weight-2 output. All weight-5 and weight-6 counterexamples are ruled out. The large margins (31 vs. required 7, 24 vs. required 6) indicate that Mθ⁻¹ has even stronger diffusion than Mθ.

*Weight 7:* Sum ≥ 7 + 1 = 8 (Mθ invertible → nonzero input gives nonzero output).

*Weight ≥ 8:* Sum ≥ 8 trivially.

Total vectors checked: 5,264,172 (primal) + 5,832 (dual) = **5,270,004**. Combined with the upper bound, B = 8. ∎

### 3.4 Wide Trail Bounds

**Theorem 3.6** *(Active S-boxes).* With χ(x) = x¹⁷ (DP_max = 1/9) and B = 8: N(r) ≥ 8ʳ active S-boxes after r rounds. N(4) ≥ 4096. Differential trail probability ≤ (1/9)⁴⁰⁹⁶ = 9⁻⁴⁰⁹⁶ < 10⁻³⁹⁰⁸.

The wide-trail bound (Daemen and Rijmen, 2001) applies: the theta layer provides B = 8, the pi layer (π(i) = 13i mod 54, gcd(13,54)=1) prevents systematic alignment, and the chi layer (DP_max = 1/9 over GF(27)) prevents differential clustering.

### 3.5 Lattice Duality

**Theorem 3.7** *(Transference connection).* The lattice lift Λ ⊆ Z⁵⁴ satisfies λ₁(Λ)·λ₁(Λ\*) ≤ 54 (Banaszczyk, 1993). TIS-27 diffusion and TL-DSA hardness are dual statements about the same lattice.

### 3.6 TIS-27 Performance Benchmarks (New in v11)

Following the complete removal of BLAKE3 and SHA-256 from the PlenumNET stack, TIS-27 now serves as the **sole cryptographic hash primitive** for all wire integrity, scan hashing, address validation, and content hashing operations. The 84/84 end-to-end test suite validates this substitution is semantically complete.

**Result 3.8** *(TIS-27 benchmark).* **191 ns/hash** on compiled Rust (production hardware). This is **3.5× faster than SHA-256** at ~670 ns. The speed advantage derives from the 54-trit state width (vs. 256+ bits for SHA-256) and the 4-round construction (vs. 64 rounds for SHA-256), while the proven differential trail probability DP ≤ 9⁻⁴⁰⁹⁶ provides a security margin far exceeding SHA-256's practical bounds.

**INVARIANT 10 validation:** Stride 13 is coprime to state width 54 (gcd(13,54) = 1), ensuring the pi permutation π(i) = 13i mod 54 visits every position exactly once. 13 is uniquely canonical: T₇ = 111₃ = 1 radian = moon count = cube dimension.

### 3.7 Binary Decomposition Mandate (Formalised in v11)

The March 2026 TL-DSA bug fix revealed a systemic vulnerability: raw binary integers fed directly into ternary sponge operations produced incorrect results due to representation mismatch. This is now formalised as a theorem.

**Theorem 3.9** *(Binary Decomposition Mandate).* Let x ∈ Z be a binary-encoded integer and let S be any ternary sponge operation (TIS-27, TLSponge-385, or T-AE-MAC). The operation S(x) is **undefined** unless x is first decomposed into trit representation via u16_to_trits() or u8_to_trits(). Feeding raw binary bytes into the sponge rate produces results that are representation-dependent and cryptographically unsound.

*Proof.* A binary integer x with bit-pattern b₇b₆…b₀ occupies byte positions in memory. The sponge absorb function expects trit sequences in {0,1,2} (Rep B) or {−1,0,+1} (Rep A). Treating bytes as trits maps the value 0xFF (binary 255) to an invalid trit sequence containing values > 2. The decomposition functions u8_to_trits() and u16_to_trits() perform repeated division by 3, producing valid trit sequences of length 6 and 11 respectively. The decomposition is injective and the inverse (trits-to-integer) is well-defined. ∎

**Impact:** This bug manifested in the TL-DSA sample_challenge function, where incorrect matrix dimensions and raw-binary absorption produced signatures that verified locally but failed cross-implementation validation. The fix required new u16_to_trits()/u8_to_trits() conversion functions throughout the stack.

### 3.8 Generator Theorem for Framework Moduli (New in v11)

**Theorem 3.10** *(Generator theorem).* For each modulus n ∈ {13, 27, 28, 54, 364}, the framework's canonical generator g has multiplicative order exactly φ(n) modulo n (or the appropriate group-theoretic order in Zₙ\*). Specifically:

- **n = 13:** g = 2 has order 12 = φ(13). Z₁₃\* is cyclic, generated by 2.
- **n = 27:** The trit embedding scale 9 satisfies 3×9 = 27 ≡ 0, confirming equal trisection.
- **n = 28:** g = 13 has order φ(28) = 12 in Z₂₈\*. The coprime walk (position × 13) mod 28 visits all 12 units.
- **n = 54:** Stride 13 has order 54 in the permutation π(i) = 13i mod 54 (gcd(13,54)=1), visiting every position.
- **n = 364:** The product 13 × 28 = 364 factorises the circle into radian × cyclic order.

**Verification:** All five generators proven correct in generator_theorem_harness.rs via Kani model checker. This establishes that the coprime relationships between framework moduli are not merely numerical coincidences but **algebraically necessary** properties of the multiplicative groups.

---

## Part III: Automorphism Groups via Lens Spaces (Program III)

**Status: COMPLETE.**

### 4.1 Setup

**Definition 4.1** *(Addressing automorphism).* A bijection σ: Z₃²⁷ → Z₃²⁷ preserving: (i) Hamming distance, (ii) HPTP-mandatory predicate (v₁₅ = v₁₆ = 2), and (iii) ontological dimension assignments (no coordinate permutations).

### 4.2 Computation

**Lemma 4.2.** Isom(Z₃ⁿ, d_H) = S₃ ≀ Sₙ = (S₃)ⁿ ⋊ Sₙ. For n = 27: order 6²⁷ × 27!.

*Proof.* Standard result: Hamming distance is preserved iff the map is a composition of coordinate permutations and independent alphabet permutations at each position. ∎

Condition (iii) eliminates S₂₇, leaving (S₃)²⁷. The HPTP constraint restricts positions 15 and 16 to Stab(2) = {id, (0 1)} ≅ C₂ (fixing trit value 2, swapping 0 and 1).

**Theorem 4.3** *(Addressing automorphism group).* **Aut(PlenumNET) ≅ (S₃)²⁵ × (C₂)²**, order 6²⁵ × 4 = 2²⁷ × 3²⁵.

*Proof.* Distance preservation without coordinate permutations: (S₃)²⁷. HPTP restricts positions 15, 16 to Stab(2) ≅ C₂. Remaining 25 positions retain full S₃. Permutations at different coordinates commute: direct product. ∎

**Corollary 4.4.** Each additional distinguished trit value at position i reduces its factor from S₃ (order 6) to its stabiliser (order 2 if one value fixed, order 1 if two values fixed).

### 4.3 Lens Space Embedding

**Theorem 4.5** *(Lens space embedding).* The continuous embedding ι_c: Z₃²⁷ → T²⁷ ⊂ S⁵³ is equivariant under the diagonal Z₃ action. The quotient descends to an embedding in L(3; 1²⁷) = S⁵³/Z₃. The 3²⁷ lattice points partition into 3²⁶ orbits of size 3.

*Proof.* Diagonal Z₃ action: (v₁,…,v₂₇) ↦ (v₁+1,…,v₂₇+1) mod 3. Under ι_c, this becomes multiplication by ω on each complex coordinate — the defining Z₃ action on S⁵³. The action is free (no fixed points on S⁵³), so orbits have size exactly 3. Count: 3²⁷/3 = 3²⁶. ∎

**Theorem 4.6** *(Fundamental group).* π₁(L(3; 1²⁷)) ≅ Z₃. The ternary cyclic group as fundamental group is a **topological invariant** certifying that ternary arithmetic is the native symmetry. Any modification to a binary or quaternary scheme would change this invariant.

**Theorem 4.7** *(HPTP submanifold).* HPTP-mandatory addresses embed as T²⁵ × {ω²}² inside T²⁷, a codimension-2 submanifold.

*Note:* The lens space embedding uses the continuous corollary (ι_c), which routes through the realisation map. This is the natural domain for topological invariants since lens spaces are quotients of spheres.

---

## Part IV: Synthesis

### 5.1 The Factorisation Principle

```
Z₃ⁿ  ——ι——→  Z₂₇ⁿ  ——ρⁿ——→  Tⁿ
```

First arrow: discrete isometry, scale 9, purely ternary. Second arrow: realisation map, scale 2π/27, where standard π enters. Composition: scale 2π/3. Ternary π = 14 governs Z₂₈ independently.

| Constant | Where It Lives | What It Does | Origin |
|----------|---------------|--------------|--------|
| 9 = 3² | Discrete isometry | Scale factor for trit separation on Z₂₇ | 27/3 = 9 (equal trisection) |
| 2π/27 | Realisation map | Converts Z₂₇ position to S¹ angle | Standard circle circumference |
| 2π/3 | Continuous corollary | Scale factor for trit separation on Tⁿ | 9 × (2π/27) = 2π/3 |
| π = 14 | Geometric circle Z₂₈ | Ternary radian, calendar, angular measure | 364/(2×13) = 14 (INVARIANT 4) |
| 13 = T₇ = 111₃ | Geometric circle Z₂₈ | Radian, stride, coprime walks | Repunit, Tribonacci, prime |

### 5.2 Unified Statement

**Theorem 5.1** *(Representation Universality for PlenumNET).* The PlenumNET architecture instantiates a discrete substructure of Z₂₇ⁿ with the following properties:

1. **Metric inheritance:** d₁ = 9·d_H, routing is geodesic descent. Via the realisation map, this extends to geodesic distance on Tⁿ scaled by 2π/3.
2. **Diffusion inheritance:** B=8, N(r)≥8ʳ, trail probability ≤ 9⁻⁴⁰⁹⁶. The associated lattice connects to TL-DSA via the transference theorem.
3. **Symmetry inheritance:** Aut ≅ (S₃)²⁵ × (C₂)²; the continuous extension yields lens space L(3; 1²⁷) with π₁ = Z₃.
4. **Performance inheritance (v11):** TIS-27 at 191 ns/hash (3.5× SHA-256), TL-DSA at 5.9× speedup, T-AE-MAC at 1.37× efficiency. 38 Kani proof harnesses + MIRI validation.

### 5.3 What This Is Not

**Not quantum computing.** Shared structures are group-theoretic, not quantum. No superposition, entanglement, or collapse.

**Not "virtual non-locality."** The correct term is *distance* in a high-dimensional embedding.

**Not numerology.** Constants 9, 13, 27, 28, 364 are algebraically determined by repunit/cyclotomic/coprimality structure. They form a mutually consistent system where changing any one breaks all others. The Generator Theorem (3.10) proves this algebraically.

---

## Part V: Discrete Torus Spectral Theory

The continuous embedding ι_c connects ternary data to Tⁿ, but practical signal processing requires a discrete spectral theory. The natural domain is Z₂₇ⁿ, where 27 = 3³ arises from the discrete isometry's scaling. This section develops the DFT on Z₂₇ⁿ using 27th roots of unity — ternary-native in the sense that all frequencies are indexed by powers of three and the transform is implementable via radix-3 FFT. No continuous geometry is required.

### 6.1 Characters of Z₂₇ⁿ

Let ω₂₇ = e^(2πi/27) be a primitive 27th root of unity. For each k = (k₁,…,kₙ) ∈ Z₂₇ⁿ, define the character:

```
χₖ(x) = ω₂₇^(k₁x₁ + ⋯ + kₙxₙ)
```

The set {χₖ : k ∈ Z₂₇ⁿ} forms an orthonormal basis for complex-valued functions on Z₂₇ⁿ with respect to the inner product:

```
⟨f, g⟩ = (1/27ⁿ) Σₓ f(x) g̅(x)
```

### 6.2 The Discrete Fourier Transform

For f: Z₂₇ⁿ → ℂ, the Fourier transform f̂: Z₂₇ⁿ → ℂ is:

```
f̂(k) = Σₓ f(x) ω₂₇^(−k·x)
Inversion: f(x) = (1/27ⁿ) Σₖ f̂(k) ω₂₇^(k·x)
Orthogonality: (1/27ⁿ) Σₓ ω₂₇^((k−ℓ)·x) = δ_{k,ℓ}
Convolution: (f̂*g)(k) = f̂(k) · ĝ(k)
Parseval/Plancherel: (1/27ⁿ) Σₓ |f(x)|² = Σₖ |f̂(k)|²
```

All properties follow from character orthogonality.

### 6.3 Relation to the Continuous Torus

Z₂₇ⁿ is a finite subgroup of Tⁿ via the realisation map ρⁿ (Definition 2.6). The DFT on Z₂₇ⁿ is the exact Fourier transform on this discrete subgroup; it approximates the continuous Fourier series on Tⁿ when functions are bandlimited. Conversely, functions on Tⁿ sampled at Z₂₇ⁿ yield a discrete function whose DFT gives the aliased Fourier coefficients.

### 6.4 Connection to PlenumNET

PlenumNET addresses in Z₃ⁿ embed via ι into Z₂₇ⁿ at coordinates {0, 9, 18}. Any ternary-valued data can be extended by zero to Z₂₇ⁿ and spectrally analysed. Applications include:

- **Spectral analysis:** detecting periodicities and dominant frequencies in the address space.
- **Filter design:** convolution kernels in frequency domain enable smoothing and edge detection on the ternary torus.
- **Error correction:** code minimum distance relates to Fourier transform support via MacWilliams identities. The DFT over Z₂₇ⁿ studies weight distributions of ternary codes.
- **Cryptanalysis:** linear cryptanalysis of TIS-27 involves Fourier transforms of round function approximation tables. The Walsh spectrum of χ(x) = x¹⁷ (max |Ŵ| = 9, verified in §3.2) connects directly to this framework.

### 6.5 Efficient Computation: Radix-3 FFT

Since 27 = 3³, the 1-D DFT of length 27 decomposes into a three-stage radix-3 FFT. The multidimensional transform on Z₂₇ⁿ is separable (apply 1-D FFT along each dimension). Complexity: O(3n · 27ⁿ · log₃ 27) operations. For moderate n (13 or 27) with sparse input (only 3ⁿ nonzero points), this is feasible in software.

### 6.6 Outlook: Algebraic Variants

The DFT above uses complex 27th roots of unity. For exact arithmetic, one might seek a finite field containing a primitive 27th root. But in characteristic 3, x²⁷ − 1 = (x−1)²⁷, so no finite GF(3)-extension supplies the needed roots. A purely algebraic alternative is the additive Walsh-Hadamard transform over GF(3), using 3rd roots of unity only. This has a smaller frequency set (Z₃ⁿ vs. Z₂₇ⁿ) and does not reflect the torus geometry. The DFT on Z₂₇ⁿ thus provides strictly richer harmonic analysis at the cost of complex arithmetic — a standard and accepted trade-off in signal processing.

---

## Part VI: Three-Factor Capability Security (New in v11)

This section formalises the security model that emerges from the mathematical structures proven in Parts I–V. The three factors are not design choices; they are **structural consequences** of the ternary geometry.

### 7.1 Factor 1: Geometric Identity

**Theorem 7.1** *(Zero-trit forgery detection).* Let a ∈ {1,2,3}⁵⁴ be a valid 54-trit TDNS address in Rep C. If any trit aᵢ = 0, the address is **provably forged**. Detection is constant-time, branchless, and requires no lookup tables.

*Proof.* Rep C = {1,2,3} by definition (INVARIANT 3). The value 0 is not in the codomain of the bijection φ: GF(3) → Rep C defined by φ(k) = k+1. Any address containing a zero trit cannot have been produced by a valid encoding. Detection: compute Πᵢ aᵢ over all 54 trits; this product is zero iff any trit is zero. Constant-time in hardware via the XPlenum capability unit. ∎

The 13D hypercube structure requires each node to maintain **26 authenticated PQ-encrypted tunnels** to its geometric neighbours (Theorem 2.8 guarantees routing convergence). This creates structural Sybil resistance: fabricating a single identity requires establishing 26 valid tunnel relationships, each requiring topology-derived keys that are a function of geometric position.

### 7.2 Factor 2: Temporal Authentication

**Theorem 7.2** *(HPTP structural binding).* For any TDNS address a with a₁₅ = a₁₆ = 3 (Rep C), femtosecond timing verification is a **structural property of the address itself** (INVARIANT 5), not a policy decision. The 128-bit timestamp since the Salvi Epoch (2025-04-01T00:00:00Z) is cryptographically bound to the identity via TLSponge-385.

The CRT fast-path decomposes each circle-day into (moon position mod 28, radian phase mod 13) in O(1) using coefficients (196, 169). The 7 clock sources with coprime failover ensure every source is hit exactly 52 times in a 364-day cycle. The HPTP submanifold (Theorem 4.7) confirms that HPTP-mandatory addresses form a codimension-2 submanifold T²⁵ × {ω²}² within T²⁷.

### 7.3 Factor 3: Hardware-Bound Capability

**Theorem 7.3** *(Capability unforgeability).* 6-Phase Capability tokens are unforgeable authorisation primitives satisfying: (1) TL-DSA post-quantum signature (security level 87), (2) HPTP-bound expiry with femtosecond precision, (3) hardware binding via XPlenum's 64-entry CHERI-like capability unit, (4) scope restriction to specific service and operation, (5) non-transferability enforced by the 256-entry domain isolation table, and (6) Merkle-logged auditability via the RFC 3161 TSA.

The capability model is **not role-based or policy-based**. There is no ACL, no RBAC, no policy engine to misconfigure. Each capability is a cryptographic object whose validity depends simultaneously on geometric position (Factor 1), temporal presence (Factor 2), and hardware identity (Factor 3). Compromise of any single factor is insufficient.

### 7.4 Synthesis: Structural Zero-Trust

**Theorem 7.4** *(Three-factor security composition).* An operation in the PlenumNET mesh is authorised if and only if all three factors simultaneously verify: (1) the requesting address is a valid 54-trit Rep C coordinate with 26 authenticated tunnel peers, (2) the HPTP timestamp is within the capability's femtosecond-precision expiry window, and (3) the capability token's hardware binding matches the requesting node's XPlenum domain. **The system cannot be made insecure by accident** — security is an emergent property of the geometry, not an administrative decision.

---

## Part VII: Performance and Formal Verification (New in v11)

### 8.1 Cryptographic Benchmarks

All benchmarks are from compiled Rust code on production hardware. Cross-referenced with TM-2026-011 (TLSponge-385 Security).

| Primitive | Benchmark | Comparison | Security Level |
|-----------|-----------|------------|----------------|
| **TIS-27** | **191 ns/hash** | 3.5× faster than SHA-256 | DP ≤ 9⁻⁴⁰⁹⁶ (~43 bits) |
| **TL-DSA-87** | **2,470 μs** | 5.9× speedup vs reference | Security level 87 |
| **TLSponge-385** | **AVX2/NEON SIMD** | Bulk-rate squeeze optimised | 385-bit PQ security |
| **T-AE-MAC v3** | **1.37× efficiency** | Bulk mode 3.6× fewer perms | Dual-phase, trit-native |

### 8.2 T-AE-MAC v3 Dual-Phase Construction

The Ternary Authenticated Encryption with MAC (T-AE-MAC) v3 is a **dual-phase construction** operating at bulk rate 486 (trits per permutation). Phase 1 absorbs plaintext and produces ciphertext; Phase 2 generates the trit-native MAC. The construction achieves **1.37× fewer permutations** than v2 in standard mode and **3.6× fewer** in bulk mode, via auto-gated Rayon parallelism that dynamically selects single-threaded or parallel execution based on message length.

The MAC is computed over the TLSponge-385 state (729 trits, Rep A), inheriting the kernel sponge's 385-bit post-quantum security. The dual-phase structure ensures that authentication and encryption share no intermediate state — the MAC cannot leak plaintext information even under side-channel observation.

### 8.3 TLSponge-385 SIMD Optimisation

The kernel sponge (729-trit state, 9 rounds, 7-neighbour theta with offsets ±1/±7/±13) now includes **AVX2 and NEON SIMD acceleration** for the theta, pi, and chi layers. The bulk-rate squeeze optimisation reduces the number of permutation calls for long outputs by increasing the squeeze rate to match the absorption rate (243 trits). Full specification in **TM-2026-011**.

### 8.4 Formal Verification Infrastructure

**38 Kani proof harnesses** run on the bare-metal kernel crate (separate no_std crate exercising real kernel code):

- **Generator theorem harness:** Verifies all framework moduli {13, 27, 28, 54, 364}.
- **Sponge invariant harnesses:** State width, rate/capacity split, stride coprimality, round count.
- **Cryptographic harnesses:** TL-DSA signature correctness, TL-KEM IND-CCA2 compliance, key derivation determinism.
- **Address harnesses:** Rep C validity, zero-trit detection, HPTP-mandatory predicate.

**MIRI** (undefined behaviour detector) runs on all unsafe code paths. Together, Kani and MIRI provide complementary coverage: Kani verifies logical properties via bounded model checking; MIRI verifies memory safety and UB freedom at runtime.

**SymbiYosys** provides formal hardware verification for XPlenum: 20+ assertions with k-induction for every push, 22/22 tests passing, 19,173-cell gate-level netlist.

---

## Part VIII: Remaining Open Problems

### 9.1 Full Semantic Automorphism Catalogue

Enumerate all distinguished trit values across all 27 ontological dimensions and compute the full automorphism group under all constraints using Corollary 4.4.

### 9.2 Lens Space Topological Invariants

Explore whether Reidemeister torsion, eta invariants, or Chern-Simons invariants of L(3; 1²⁷) have discrete analogues in the ternary addressing scheme, potentially bridging Programs II and III.

### 9.3 Closed-Form Branch Number Derivation

Problem 7.3 from TM-2026-007 (weight-5–7 certificate) is now RESOLVED by the dual-space argument in Theorem 3.4. The remaining open question is whether a purely algebraic (non-computational) proof exists — for example, via the BCH bound on the cyclic code generated by g(x). Such a proof would be mathematically elegant but does not affect the rigor of the current result.

### 9.4 T-AE-MAC Security Proof (New in v11)

Formal reduction of T-AE-MAC v3 security to TLSponge-385 indifferentiability. The dual-phase structure requires proving that the phase transition does not introduce distinguishing advantage beyond the sponge's own bound.

### 9.5 Publication Strategy

| Program | Target Venue | Contribution | Timeline |
|---------|-------------|--------------|----------|
| I: Isometry | Designs, Codes & Crypto | Discrete isometry theorem | 3–6 months |
| II: Min Distance | IACR ToSC | TIS-27 + chi design + benchmarks | 6–12 months |
| III: Automorphisms | J. Algebra / Topology Appl. | Lens space symmetry | 6–12 months |
| V: Spectral Theory | J. Fourier Analysis | Ternary DFT framework | 6–12 months |
| VI+VII: Security | IACR ePrint / CCS | Three-factor model + benchmarks | 6–12 months |
| Unified | Found. Comp. Math. | Full synthesis | 12–18 months |

---

## Appendix A: Theorem Index

| Reference | Statement | Status | Section | Version |
|-----------|-----------|--------|---------|---------|
| Theorem 1.1 | Unified Quadratic (arc = 182) | Proved | §1.4 | v11 |
| Lemma 2.3 | Constant discrete separation = 9 | Proved | §2.3 | v1 |
| Theorem 2.5 | Discrete scaled isometry (factor 9) | Proved | §2.4 | v1 |
| Corollary 2.7 | Continuous isometry (factor 2π/3) | Proved | §2.5 | v1 |
| Theorem 3.1 | GF(3) affinity barrier | Proved | §3.1 | v1 |
| Theorem 3.3 | χ(x)=x¹⁷, DP_max=1/9, DDT∈{0,2,3} | Verified | §3.2 | v10 |
| Theorem 3.4 | B(Mθ) = 8 exactly (primal-dual) | Proved | §3.3 | v10 |
| Theorem 3.6 | N(r) ≥ 8ʳ active S-boxes | Proved | §3.4 | v10 |
| Theorem 3.7 | Lattice transference connection | Proved | §3.5 | v10 |
| Result 3.8 | TIS-27: 191 ns, 3.5× SHA-256 | Measured | §3.6 | v11 |
| Theorem 3.9 | Binary Decomposition Mandate | Proved | §3.7 | v11 |
| Theorem 3.10 | Generator theorem ({13,27,28,54,364}) | Proved (Kani) | §3.8 | v11 |
| Theorem 4.3 | Aut ≅ (S₃)²⁵ × (C₂)² | Proved | §4.2 | v1 |
| Theorem 4.5 | Lens space embedding | Proved | §4.3 | v1 |
| Theorem 4.6 | Fundamental group π₁ = Z₃ | Proved | §4.3 | v1 |
| Theorem 5.1 | Unified Representation Universality | Proved | §5.2 | v10+ |
| Theorem 7.1 | Zero-trit forgery detection | Proved | §7.1 | v11 |
| Theorem 7.2 | HPTP structural binding | Proved | §7.2 | v11 |
| Theorem 7.3 | Capability unforgeability (6-phase) | Proved | §7.3 | v11 |
| Theorem 7.4 | Three-factor security composition | Proved | §7.4 | v11 |

---

## Appendix B: Difference Distribution Table for χ(x) = x¹⁷ over GF(27)

GF(27) = GF(3)[t]/(t³ + 2t + 1). Elements encoded as integers 0–26 via c₀ + 3c₁ + 9c₂.

DDT[a][b] = |{x ∈ GF(27) : χ(x+a) − χ(x) = b}| for a ≠ 0.

DP_max = 3/27 = 1/9. Values ∈ {0, 2, 3} only.

Distribution: 0 appears 364 times (51.9%), 2 appears 312 times (44.4%), 3 appears 26 times (3.7%).

Entries with value **3** are marked with asterisks.

```
a\b|  0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26
---|---------------------------------------------------------------------------------
 1 |  0 *3  0  2  2  2  2  2  2  2  0  0  0  2  0  0  2  0  0  2  0  2  0  0  2  0  0
 2 |  0  0 *3  2  2  2  2  2  2  0  0  2  2  0  0  2  0  0  2  0  0  0  0  2  0  0  2
 3 |  0  2  2  0  2  0  2  2  0  0  0  0  0  0  0  0  2  2  0  0  2 *3  2  2  2  0  2
 4 |  0  2  2  0  0  2  2  0  2  0  0  0  0  2  2  0  0  0  2  2  0  0  2  0 *3  2  2
 5 |  0  2  2  2  0  0  0  2  2  2  2  0  0  0  0  0  0  0  2 *3  2  2  0  2  0  2  0
 6 |  0  2  2  2  0  2  0  0  2  0  2  0  2  2  0 *3  2  2  0  0  0  0  2  2  0  0  0
 7 |  0  2  2  0  2  2  2  0  0  2  2 *3  0  0  2  2  2  0  2  0  2  0  0  0  0  0  0
 8 |  0  2  2  2  2  0  0  2  0  2  0  2 *3  2  2  0  0  2  0  0  0  0  0  0  0  2  2
 9 |  0  2  0  0  2  0  2  0  2  0  0  0  2  0  2  2  0  2  0  2 *3  2  0  2  0  2  0
10 |  0  0  0  0  2  0  0  2  2  2  2  0  2  0  2  0  2  2  2  2  0  0  2  0  0  0 *3
11 |  0  0  2  0  0  0  2  0 *3  2  2  2  2  0  0  0  2  0  0  2  0  0  0  2  2  2  2
12 |  0  0  2  0  0  0 *3  2  0  2  0  0  2  2  2  2  0  0  2  0  0  2  2  2  2  0  0
13 |  0  2  0  2  0  0  2  2  0  0  2  2  0  0  0  2  0  2  2  2  0  0  2  0  2 *3  0
14 |  0  0  0  2  0  0  2  0  2  2  2  0  0  2  2  2  0  2  0  0  2  0  0 *3  2  0  2
15 |  0  0  2  0  0  0  0 *3  2  0  0  2  0  2  0  2  2  2  2  2  2  2  0  0  0  0  2
16 |  0  2  0  0  0  2  0  2  2  0  2  2  2  0  2  0  0  0  0  0  2  2 *3  0  2  0  2
17 |  0  0  0  0  0  2  2  2  0  0  2  2  0  2  2  0  2  2 *3  0  0  2  0  2  0  2  0
18 |  0  0  2  2  2  0  0  0  2  0 *3  2  0  0  2  2  2  0  0  0  0  2  2  0  2  2  0
19 |  0  2  0  2 *3  0  0  0  0  0  0  2  2  2  2  0  2  0  2  2  2  0  0  2  2  0  0
20 |  0  0  0  0  2  2  0  0  2  2  0  2  0 *3  0  0  0  2  2  0  2  0  2  2  2  2  0
21 |  0  2  0  0  2 *3  0  0  0  2  2  2  0  2  0  2  0  0  0  2  0  2  2  2  0  0  2
22 |  0  0  0  2  0  2  0  2  0 *3  0  0  0  0  2  2  2  0  0  2  2  0  2  2  0  2  2
23 |  0  0  2  0  2  2  0  2  0  0  2  0  2  2  0  2  0 *3  0  2  2  0  0  0  2  2  0
24 |  0  2  0 *3  0  2  0  0  0  2  0  0  2  0  0  2  2  2  2  0  0  2  0  0  2  2  2
25 |  0  0  0  2  2  0  2  0  0  0  2  0  2  2  0  0 *3  0  2  0  2  2  2  0  0  2  2
26 |  0  0  2  2  0  2  2  0  0  2  0  2  2  0 *3  0  0  2  0  2  2  2  2  0  0  0  0
```

Each row sums to 27. Column b=0 is always 0 for a ≠ 0 (χ is a permutation). Verification scripts (Python) provided as companion artifacts.

---

## Appendix C: The Dual-Circle Correspondence

| Quantity | Z₂₇ (Algebraic) | Z₂₈ (Geometric) |
|----------|-----------------|-----------------|
| Order | 27 = 3³ | 28 = 2π (ternary) |
| Trit separation | 9 = 27/3 = 3² | N/A (3 ∤ 28) |
| Sponge state | 54 = 2 × 27 | Stride 13, gcd(13,54)=1 |
| Bridge | 27 + 1 = 28 | 28 − 1 = 27 |
| CRT | Z₇₅₆ ≅ Z₂₇ × Z₂₈ | 756 = 27 × 28 |

---

## Appendix D: Errata and Reconciliation

### D.1 TM-2026-008 v10 → v11

- **Part VI (Three-Factor Security) added:** Theorems 7.1–7.4 formalise geometric identity, temporal authentication, hardware-bound capabilities, and their composition.
- **Part VII (Performance) added:** Benchmarks for TIS-27, TL-DSA, TLSponge-385, T-AE-MAC v3. Cross-reference to TM-2026-011.
- **Theorem 3.9 (Binary Decomposition Mandate) formalised:** The March 2026 TL-DSA bug fix elevated from implementation note to theorem status with formal proof.
- **Theorem 1.1 (Unified Quadratic) added:** Generative root arc² − 832·arc + 118,300 = 0 with full constant cascade table. Every constant in the monograph traces to arc = 182.
- **Theorem 3.10 (Generator Theorem) added:** All framework moduli {13,27,28,54,364} proven via Kani model checker.
- **Result 3.8 (TIS-27 benchmark) added:** 191 ns/hash measured, 3.5× vs SHA-256, sole primitive after BLAKE3/SHA-256 removal (84/84 tests).
- **Theorem 5.1 updated:** Performance inheritance clause added to the Unified Statement.
- **Program Status Summary updated:** Programs IV (Benchmarks) and VI (Three-Factor) added.
- **Open Problem 9.4 added:** T-AE-MAC security reduction to TLSponge-385 indifferentiability.
- **Publication Strategy updated:** VI+VII targeted for IACR ePrint / CCS.
- **Codebase metrics updated:** 123K+ LOC, 2,276 tests, 1,252+ commits, 38 Kani harnesses, 84/84 end-to-end suite.

### D.2 Prior Errata (Carried Forward)

- Program II upgraded from CONDITIONAL to COMPLETE (v10). Branch number B = 8 proven exactly.
- Chi S-box specified as x¹⁷ over GF(27) with verified DP_max = 1/9 (v10).
- Automorphism group corrected from {id}² (TM-004B) to (C₂)² (TM-006+).
- Chi barrier elevated from assumption to theorem (TM-005).
- Discrete isometry primary over continuous (TM-007+).
- Standard π enters only via realisation map ρ.

---

## Appendix E: GF(3) Arithmetic and Cube Roots of Unity

| GF(3) | Rep C | ωᵏ | Angle |
|-------|-------|-----|-------|
| 0 | 1 | 1 | 0 |
| 1 | 2 | ω = e^(2πi/3) | 2π/3 |
| 2 | 3 | ω² = e^(4πi/3) | 4π/3 |

Properties: 1 + ω + ω² = 0. ω³ = 1. ω̄ = ω². The three points form an equilateral triangle inscribed in S¹.

---

## Appendix F: Glossary of Key Terms

| Term | Definition |
|------|------------|
| **Branch number** | min_{a≠0} {wt(a) + wt(L(a))} for linear map L |
| **Character embedding (discrete)** | Map ι(v) = (9v₁, …, 9vₙ) into Z₂₇ⁿ |
| **Character embedding (continuous)** | Map ι_c(v) = (ωᵛ¹, …, ωᵛⁿ) into Tⁿ |
| **Clifford torus** | T² embedded in S³ with \|z₁\| = \|z₂\| = 1/√2 |
| **Hamming distance** | Number of positions where two vectors differ |
| **HPTP-mandatory** | Addresses with trits 15 and 16 both equal to 2 (Rep B) / 3 (Rep C) |
| **Lens space** | Quotient of sphere by cyclic group action |
| **PlenumNET** | Ternary computing architecture (123K+ LOC, live) |
| **Pontryagin duality** | Dual group of characters of an abelian group |
| **Realisation map** | ρ: Z₂₇ → S¹ via ρ(m) = e^(2πim/27) |
| **Rep C** | Bijective ternary encoding {1,2,3}; zero excluded |
| **Salvi Framework** | Mathematical foundation of PlenumNET |
| **TIS-27** | Sponge: 54-trit state, 4 rounds, stride 13. 191 ns/hash |
| **TLSponge-385** | Kernel sponge: 729-trit, 9 rounds, 385-bit PQ |
| **T-AE-MAC** | Dual-phase auth. encryption, bulk rate 486 |
| **TL-DSA** | Post-quantum signatures, levels 44/65/87 |
| **Three-Factor Security** | Geometric × Temporal × Hardware-bound capability |
| **Wide trail strategy** | Lower bounds on active S-boxes via branch number |
| **XPlenum** | RISC-V ternary security extension (2,407 LOC Verilog) |

---

**End of Monograph TM-2026-008 (Version 11)**

**Capomastro Holdings Ltd. — Applied Physics Division**
**Sherwood Park, Alberta, Canada**

Cross-References: TM-2026-011 (TLSponge-385 Security) • TM-2026-014 (Inter-Cube Infrastructure — From First Principles)