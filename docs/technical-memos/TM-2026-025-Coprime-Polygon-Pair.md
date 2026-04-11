# THE (11, 13) COPRIME POLYGON PAIR

## Generator Duality, CRT Decomposition, and Structural Alignment in the Salvi Framework

---

**Technical Memorandum — TM-2026-025 (Version 3)**
**Salvi Framework — PlenumNET Architecture Series**
**March 2026**

**Capomastro Holdings Ltd. — Applied Physics Division**
**Sherwood Park, Alberta, Canada**

© 2026 Capomastro Holdings Ltd. — All Rights Reserved — Patent(s) Pending

---

## Abstract

This memorandum formalizes the (11, 13) coprime polygon pair as a named structural element in the Salvi Framework. The hendecagon (11 edges) and tridecagon (13 edges), inscribed in the 364° ternary circle, produce a combined arc of 143° = 11 × 13° = 11 ternary radians. Their interleave yields 23 combined vertices with palindromic distribution pattern [1,1,1,1,1,2,1,1,1,1,1].

The principal finding is a Generator Duality Theorem: the two factors of 143 are the canonical generators of the framework's two fundamental cyclic groups — 13 generates Z₂₈ (the radian cycle), while 11 generates Z₃₆₄ (the full circle). This duality is already formally verified in the bare-metal validation suite (`generator_theorem_harness.rs`) but has not previously been named or connected to the polygon pair.

Cross-referencing against TM-2026-008 (Representation Universality) reveals that 143 maps to (8, 3) in the dual-circle CRT decomposition Z₇₅₆ ≅ Z₂₇ × Z₂₈, where 8 is the proven branch number B(Mθ) and 3 is the Rep C maximum. Additionally, 23 — the combined vertex count — is both 11⁻¹ mod 28 and one of the three optimal chi S-box exponents {17, 23, 25} from Program II.

Version 3 completes the PlenumColor harmonic system (§6.1). All four harmonic identifiers in `color.rs` — ARC_RED (182), ARC_BLUE (240), ARC_GREEN (650), and the new ARC_COPRIME (286) — are now derived from the (11, 13) coprime polygon pair:

| Harmonic | Value | Derivation from (11, 13) |
|----------|-------|--------------------------|
| ARC_RED | 182 | πR₃ = 14 × 13 |
| ARC_BLUE | 240 | 2 × φ(143) = 2 × φ(11 × 13) = 3⁵ − 3 |
| ARC_COPRIME | 286 | 2 × 143 = 2 × 11 × 13 |
| √Δ_arc | 468 | ARC_RED + ARC_COPRIME = 36 × 13 |
| ARC_GREEN | 650 | FULL_CIRCLE + ARC_COPRIME = 364 + 286 |

---

## 1. The Generator Duality Theorem

The framework's verified generator theorem (`bare-metal/generator_theorem_harness.rs`, 288 lines) establishes by exhaustive formal verification that step `a` generates Z_m if and only if gcd(a, m) = 1, for all framework moduli {13, 27, 28, 54, 364}.

| Generator | Group | Role | Why Unique |
|-----------|-------|------|------------|
| 13 | Z₂₈ | Radian cycle, agent scheduling, calendar | gcd(13,28) = 1; T₇; 111₃; prime |
| 11 | Z₃₆₄ | Full circle traversal, degree cycle | gcd(11,364) = 1; gcd(11,54) = 1 |

**Theorem 1.1 (Generator Duality).** The two factors of 143 = 11 × 13 are the canonical generators of the framework's two fundamental cyclic groups. 13 generates Z₂₈ (geometric circle order, 28 radians). 11 generates Z₃₆₄ (full ternary circle, 364 degrees). Neither can perform the other's role: 13 does not generate Z₃₆₄ (since 13 | 364), and 11 does not generate Z₂₈ in the canonical coprime walk (stride 13 is structurally preferred per Invariant 10). The product 11 × 13 = 143 = 11 ternary radians encodes both generators in a single angular measure.

*Proof.* 364 = 2² × 7 × 13. Since 13 | 364, the walk 13k mod 364 has period 364/gcd(13,364) = 364/13 = 28, visiting only 28 of 364 positions. Conversely, gcd(11, 364) = 1 (since 11 is prime and 11 ∤ 364), so 11 generates Z₃₆₄. For Z₂₈: gcd(13, 28) = 1 (since 28 = 2² × 7 and 13 is prime), confirmed by the coprime walk visiting all 28 agent positions. ∎

---

## 2. The Euclidean Ladder: gcd(364, 143) = 13

```
364 = 2(143) + 78          ← remainder 78 = 6 × 13 = 6 radians
143 = 1(78)  + 65          ← remainder 65 = 5 × 13 = 5 radians
 78 = 1(65)  + 13          ← remainder 13 = 1 × 13 = 1 radian
 65 = 5(13)  + 0           ← terminates at the radian
```

Every remainder is a positive integer multiple of 13. The algorithm terminates at the radian.

**Continued fraction:** 364/143 = [2; 1, 1, 5]. Convergents: 2/1, 3/1, 5/2, **28/11**. The final convergent is the ratio of Z₂₈ order to Z₃₆₄ generator.

---

## 3. Chinese Remainder Theorem Decompositions

### 3.1 Full Circle: Z₃₆₄ ≅ Z₄ × Z₇ × Z₁₃

```
143 mod  4 = 3    (Rep C maximum)
143 mod  7 = 3    (Rep C maximum)
143 mod 13 = 0    (forced: 13 | 143)
```

CRT image: **(3, 3, 0)**. Rep C ceiling in both Z₄ and Z₇ simultaneously.

### 3.2 Dual-Circle: Z₇₅₆ ≅ Z₂₇ × Z₂₈

```
143 mod 27 = 8    = B(Mθ), the proven branch number
143 mod 28 = 3    = Rep C maximum
```

**Proposition 3.1.** The coprime arc maps to **(B, 3)** in the dual-circle CRT — encoding the sponge's cryptographic diffusion constant on the algebraic circle and the bijective ternary maximum on the geometric circle.

---

## 4. The 23 Combined Vertices

| Context | Role of 23 | Verification |
|---------|-----------|--------------|
| Z₂₈ modular inverse | 11⁻¹ mod 28 = 23 | `agent-generators.ts` |
| Optimal chi exponent | x²³ over GF(27), DP_max = 1/9 | TM-2026-008 §3.2 |
| Valid sponge stride | gcd(23, 54) = 1 | coprime to state width |
| Non-units of Z₁₄₃ | 143 − φ(143) = 23 | inclusion-exclusion (§4.1) |

### 4.1 The Inclusion-Exclusion Identity

**Theorem 4.1.** For n = p × q with p, q distinct primes: n − φ(n) = p + q − 1.

*Proof.* φ(pq) = (p−1)(q−1) = pq − p − q + 1. Therefore n − φ(n) = pq − (pq − p − q + 1) = p + q − 1. ∎

**Corollary 4.2.** COMBINED_VERTICES = COPRIME_ARC − φ(COPRIME_ARC) = 143 − 120 = 11 + 13 − 1 = 23.

The 23 combined vertices from the polygon interleave equals the count of non-units in Z₁₄₃. This is not a geometric coincidence — it is a direct consequence of the inclusion-exclusion principle applied to the prime factorization of the coprime arc. The vertex count, the modular inverse, and the chi exponent are three independent manifestations of the same number-theoretic structure.

---

## 5. The Palindromic Interleave

**[1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1]** — 11 entries summing to 12 = 13 − 1. Palindromic around index 5. The single "2" at the center marks the angular region where tridecagon and hendecagon vertices nearly coincide.

---

## 6. Unified Equation and PlenumColor Harmonics

The Plenum Square unified arc equation arc² − 832·arc + 118,300 = 0 has roots 182 and 650, with discriminant Δ = 832² − 4(118,300) = 219,024 = 468².

| Relationship | Value |
|-------------|-------|
| 650 − 364 | 286 = 2 × 143 = ARC_COPRIME |
| 182 − 143 | 39 = 3 × 13 = 3 radians |
| 182 + 143 | 325 = 25 × 13 = 25 radians |
| 650 − 143 | 507 = 3 × 13² |
| φ(364) − φ(143) | 144 − 120 = 24 = T₈ |

### 6.1 The Complete PlenumColor Harmonic System

The harmonic identifiers in `src/kernel/src/browser/color.rs` shape quadratic Bézier control points via `arc_bezier_control()`, normalized by FULL_CIRCLE for sub-cell interpolation. All five values derive from the (11, 13) coprime polygon pair:

#### ARC_RED = 182 = 14 × 13 = πR₃

The semicircle root. Half the full ternary circle: 364 / 2 = 182.

#### ARC_BLUE = 240 = 2 × φ(143) = 3⁵ − 3

**Primary derivation:** ARC_BLUE = 2 × φ(COPRIME_ARC) = 2 × φ(11 × 13) = 2 × (10 × 12) = 240.

**Base-3 structural identity:** 240 = 3⁵ − 3 = 243 − 3 = 3(3⁴ − 1). The highest power of 3 in the repunit sum (1 + 3 + 9 + 27 + 81 + **243** = 364) minus the lowest non-trivial power.

**CRT decompositions:**
- Z₇₅₆ ≅ Z₂₇ × Z₂₈ → **(24, 16)** = **(T₈, 2⁴)** — Tribonacci on the algebraic circle, power-of-2 on the geometric circle
- Z₃₆₄ ≅ Z₄ × Z₇ × Z₁₃ → **(0, 2, 6)** — the Z₁₃ component is 6 = φ(7) = the first Bézout coefficient

#### ARC_COPRIME = 286 = 2 × 143 = 2 × 11 × 13

The bridge between the rejected root and the full circle: ARC_GREEN = FULL_CIRCLE + ARC_COPRIME.

#### √Δ_arc = 468 = 36 × 13 = 36 ternary radians

The discriminant root: ARC_RED + ARC_COPRIME = 182 + 286 = 468. Also ARC_GREEN − ARC_RED = 650 − 182 = 468. The roots are (832 ± 468) / 2.

#### ARC_GREEN = 650 = 364 + 286 = FULL_CIRCLE + ARC_COPRIME

The rejected root.

### 6.2 The "2×" Scaling Pattern

A uniform doubling maps the (11, 13) coprime polygon pair's three structural quantities to three harmonic identifiers:

| Source | × 2 | Harmonic |
|--------|-----|----------|
| COPRIME_ARC = 143 | 286 | ARC_COPRIME |
| φ(COPRIME_ARC) = 120 | 240 | ARC_BLUE |
| COMBINED_VERTICES = 23 | 46 | ARC_COPRIME − ARC_BLUE |

And the gaps close perfectly:

```
ARC_COPRIME − ARC_BLUE = 286 − 240 = 46 = 2 × COMBINED_VERTICES
143 − φ(143) = 23 = COMBINED_VERTICES  (Theorem 4.1)
```

The harmonic gap between ARC_COPRIME and ARC_BLUE is twice the combined vertex count — itself the non-unit count of Z₁₄₃ by the inclusion-exclusion identity.

### 6.3 Harmonic System Closure

The five harmonics {182, 240, 286, 468, 650} form a closed system with three independent generation routes to ARC_GREEN:

1. ARC_GREEN = FULL_CIRCLE + ARC_COPRIME (364 + 286)
2. ARC_GREEN = ARC_RED + √Δ (182 + 468)
3. ARC_GREEN = (832 + 468) / 2 (quadratic formula)

And ARC_BLUE sits at:
- ARC_BLUE = ARC_COPRIME − 2 × COMBINED_VERTICES (286 − 46)
- ARC_BLUE = 2 × φ(COPRIME_ARC) (2 × 120)
- ARC_BLUE = 3⁵ − 3 (base-3 structural)

---

## 7. DDT Zero-Count Alignment

The DDT of χ(x) = x¹⁷ over GF(27) contains exactly **364 zero entries** out of 702. The full circle constant governs the differential impossibilities of the chi S-box. Non-zero entries: 312 twos + 26 threes = 338, where 26 = |GF(27)*| = 2 × 13.

---

## 8. Bézout Identity

**11 × 6 − 13 × 5 = 66 − 65 = 1.** Coefficients (6, −5), where 6 = φ(7) = 240 mod 13 (the Z₁₃ CRT component of ARC_BLUE).

---

## 9. Proposed Framework Applications

### 9.1 Named Structural Constant

Add `COPRIME_POLYGON_PAIR` to PLATFORM in `shared/constants.ts`.

### 9.2 Dual-Generator Z₂₈ Scheduling

Extend `agent-generators.ts` with stride-11/stride-13 dual-generator schedule.

### 9.3 Coprime Walk Layer

Name the (11, 13) sub-pair in `coprime_walk.rs`.

### 9.4 PlenumColor Harmonic Naming

Add `ARC_COPRIME = 286` and `ARC_SQRT_DISCRIMINANT = 468` to `color.rs`. Update header comment documenting the complete harmonic derivation from the (11, 13) pair.

### 9.5 TIS-27 Dual-Stride Exploration (Future — Out of Scope)

Both 11 and 13 are coprime to 54. Dual-stride and triple-stride (11 → 13 → 23) constructions deferred to separate task.

---

## 10. Summary of Structural Connections

| Quantity | Value | Framework Significance |
|----------|-------|----------------------|
| 143 | 11 × 13 | Coprime polygon arc = 11 ternary radians |
| 143 mod 27 | 8 | Branch number B(Mθ) |
| 143 mod 28 | 3 | Rep C maximum |
| 143 in Z₃₆₄ | (3, 3, 0) | CRT: double-3 in Z₄ × Z₇ |
| 23 | 11⁻¹ mod 28 | Combined vertices, chi exponent, sponge stride, non-units of Z₁₄₃ |
| 120 | φ(143) | Euler totient of coprime arc; ARC_BLUE = 2 × 120 |
| 240 | 2 × φ(143) = 3⁵ − 3 | ARC_BLUE; CRT in Z₇₅₆ = (T₈, 2⁴) |
| 286 | 2 × 143 | ARC_COPRIME; Plenum Square gap |
| 46 | 286 − 240 | 2 × COMBINED_VERTICES = ARC_COPRIME − ARC_BLUE |
| 468 | 36 × 13 | √Δ_arc = ARC_RED + ARC_COPRIME = 36 radians |
| 24 = T₈ | φ(364) − φ(143) | Tribonacci-controlled totient gap = 240 mod 27 |
| 364 | DDT zero count | Full circle governs chi impossibilities |
| 28/11 | CF convergent | 364/143 converges to Z₂₈ order / Z₃₆₄ generator |

---

## Appendix A: Errata and Versioning

### A.1 v1 → v2

Added §6.1 (PlenumColor): ARC_COPRIME = 286, √Δ = 468.

### A.2 v2 → v3

Completed the harmonic system. ARC_BLUE = 240 = 2 × φ(143) = 3⁵ − 3. Added Theorem 4.1 (inclusion-exclusion identity: 143 − φ(143) = 23). Added §6.2 (2× scaling pattern) and §6.3 (harmonic system closure). Documented ARC_BLUE CRT images: (T₈, 2⁴) in Z₇₅₆, (0, 2, 6) in Z₃₆₄ with 6 = φ(7) = Bézout[0]. Updated summary table.

---

**End of Memorandum TM-2026-025 (Version 3)**

**Capomastro Holdings Ltd. — Applied Physics Division**
**Sherwood Park, Alberta, Canada**
