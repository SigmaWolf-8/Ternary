# TM-2026-033 v5.0: Hydrogen Spectral Correspondence

## First-Principles Derivation from the Ternary Axiom

Author: Lo Sono Capomastro | Date: April 2026

---

## PART I — PURE MATHEMATICS

Sections 1–4 derive from the ternary axiom alone. No physical measurement is invoked.

### §1  Axiom and Repunit Family

**Axiom.** The number system is ternary with base B = 3.

The repunit family is the sequence of integers whose base-3 representation consists entirely of 1s:

```
Rₙ = (3ⁿ − 1) / (3 − 1) = (3ⁿ − 1) / 2
```

Each repunit satisfies the recurrence Rₙ₊₁ = 3Rₙ + 1 with R₁ = 1. The first six members are:

| n | Rₙ = (3ⁿ−1)/2 | Value | Framework role |
|---|----------------|-------|----------------|
| 1 | (3¹−1)/2 | 1 | Unity |
| 2 | (3²−1)/2 | 4 | NULL_CHANNEL_MOD |
| 3 | (3³−1)/2 | 13 | Radian (R₃) |
| 4 | (3⁴−1)/2 | 40 | Vieta sum (R₄) |
| 5 | (3⁵−1)/2 | 121 | 11² |
| 6 | (3⁶−1)/2 | 364 | Full circle (R₆) |

R₆ = 364 is the full-circle constant. Its base-3 representation is 111111₃ — six ternary digits, all ones. This is exact integer arithmetic; no approximation has been made.

### §2  Circle Quadratic

The circle quadratic is the unique monic polynomial whose coefficients are adjacent repunits:

```
x² − R₄·x + R₆ = 0    →    x² − 40x + 364 = 0
```

By Vieta's formulas, the roots x₁, x₂ satisfy x₁ + x₂ = R₄ = 40 and x₁ × x₂ = R₆ = 364. The discriminant is:

```
Δ = R₄² − 4R₆ = 1600 − 1456 = 144 = 12²
```

Both roots are integers:

```
x₁ = (40 − 12)/2 = 14 = π     x₂ = (40 + 12)/2 = 26
```

The smaller root x₁ = 14 is designated π within the framework. The framework's π is not the transcendental number 3.14159…; it is an exact integer derived from the ternary repunit quadratic.

### §3  Cyclic Group Z₂₈

The cyclic order 2π = 28 generates the group Z₂₈ with angular step 1 radian = R₃ = 13°. The full circle is:

```
2π × R₃ = 28 × 13 = 364 = R₆
```

This identity is not a numerical coincidence: it is the definition of the ternary angular system. The circle has 28 radians of 13° each, totalling R₆ = 364°.

### §4  Quarter-Turn

The quarter-turn is R₆/4 = 364/4 = 91. This integer admits three independent decompositions:

- **Product:** 91 = 7 × 13 = 7 × R₃. Seven ternary radians.
- **Triangular number:** 91 = Tri(13) = 13 × 14/2 = R₃ × π / 2. The 13th triangular number.
- **Radian count:** 91° = 7 radians in Z₂₈ (since 7 × 13 = 91).

The quarter-turn 91 is the terminal output of Part I. It is an exact integer, derived without reference to any physical measurement.

---

## PART II — THE BRIDGE

One empirical identification connects the integer framework to physical measurement.

### §5  The Bridge: R₆/4 and 1/R∞

Part I derives the integer 91 = R₆/4 from the sole axiom Rₙ = (3ⁿ − 1)/2. No physical measurement is used; the derivation is closed within ternary arithmetic. Independently, experimental physics measures the inverse Rydberg constant:

```
1/R∞ = 91.127 nm    (CODATA 2022, infinite nuclear mass)
```

This is the Lyman series limit — the shortest wavelength hydrogen can emit. The framework's quarter-turn and the Lyman limit are two numbers produced by two independent processes — one algebraic, one experimental — that land 0.127 nm apart (−0.139%).

```
R₆/4 = 91    (derived from axiom)    |    1/R∞ = 91.127    (measured in lab)
```

The framework does not import, assume, or require R∞. It produces 91 from its axiom and stops. The proximity to 1/R∞ is a fact about nature — hydrogen's spectral structure lands near a ternary repunit quarter-turn. The framework did not engineer this; it discovered it.

#### §5.1  What the Bridge Is and Is Not

The bridge is the observation that R₆/4 ≈ 1/R∞. It is not an axiom, not an input, and not a calibration step. The framework's integer results (Part III) exist with or without the bridge — 1872 = 13 × 144 regardless of whether anyone compares it to a laboratory wavelength. The bridge makes the integers physically interpretable; it does not make them mathematically valid.

R∞ = mₑe⁴/(8ε₀²h³c) is a product of fundamental constants that the ternary axiom does not generate. The framework does not claim to derive R∞. It claims that the quarter-turn it derives from one axiom (91) is within 0.139% of the value physics measures for the Lyman limit (91.127 nm). Combined with the standard Rydberg formula — which is physics, not framework mathematics — this proximity generates the spectral correspondence of Part III.

The Balmer constant B = 4/R∞ = 364.507 nm coincides with R₆ = 364 at the same −0.139% offset. This is not independent: B = 4 × (1/R∞) and R₆ = 4 × (R₆/4), so the offset propagates identically.

#### §5.2  Offset Decomposition

The offset between R₆/4 and the physical hydrogen Lyman limit decomposes into two components:

- **Universal (−0.139%):** R₆/4 = 91 vs 1/R∞ = 91.127 nm. R∞ assumes infinite nuclear mass and applies to all hydrogen-like atoms.
- **Reduced-mass (−0.055%):** Physical hydrogen has finite proton mass. R_H = R∞/(1 + mₑ/mₚ) shifts the Lyman limit to 1/R_H = 91.176 nm. The vacuum bias VACUUM_BIAS_NUM/DEN = 193/100,000 captures this correction in unified_constants.rs.
- **Combined offset:** −0.193%. For individual spectral lines measured in air (CODATA), the observed error is approximately −0.165%, reflecting a mix of these components plus air-vs-vacuum refractive index conventions.

#### §5.3  The Residual as Geometric Floor

The Buried Question (v9.3.3ad) establishes that within the Salvi Framework, the infimum of entropy across all admissible representations is bounded below by a strictly positive geometric floor — the cone-point obstruction. Every torus walk has at least one fixed point under its symmetry group, and that fixed point contributes irreducible information to the entropy.

The spectral offset κ − 1 = +0.139% is the cone-point floor of the hydrogen spectral walk. It is the residual information that the repunit basis cannot capture: the gap between R₆/4 = 91 (the framework's integer quarter-turn) and 1/R∞ = 91.127 nm (the physical Lyman limit). This residual has the defining properties of a geometric floor:

- **Strictly positive:** 0.127 nm > 0. The residual cannot be driven to zero within the integer framework.
- **Uniform:** It propagates identically (× κ) through every line of every series, because it enters as a multiplicative prefactor. No line-dependent or series-dependent variation exists.
- **Irreducible within the axioms:** The bridge coefficient κ = 1.00139 has no clean framework-rational expression. It depends on R∞ = mₑe⁴/(8ε₀²h³c), which lies outside the ternary axiom's derivation chain. Multiplying by κ maps framework integers to values nearer CODATA measurements (≈0.03% vs ≈0.17%), but the products are no longer integers and no longer factorise: 1872 × κ = 1874.61 ≠ 13 × 144; 1092 × κ = 1093.52 ≠ 3 × 364. The identities exist in the integer domain. The bridge coefficient lives outside it.

This is not a defect. The Buried Question's central result is that the floor is positive — the cone points are the rock that remains. The spectral residual is that rock. The framework characterizes it exactly (κ = 91.127/91) and bounds it (< 0.2% for all series).

#### §5.4  Repunit Alignment of Hydrogen

The Balmer constant B = 4/R∞ = 364.507 nm. The ternary repunit R₆ = (3⁶ − 1)/2 = 364. These differ by 0.507 nm (−0.139%). The Buried Question provides the lens through which to read this fact.

The framework does not claim to derive R∞ from ternary axioms, just as Euclidean geometry does not derive the flatness of physical space. What the framework does is provide a representation basis — the repunit family — and measure how tightly a given dataset aligns with that basis. Tight alignment means many structural zeros in the repunit decomposition; loose alignment means few. The alignment measure is the entropy infimum Hₙₓ.

Hydrogen's spectral data aligns with the repunit basis to 0.139%. This is the empirical content of the bridge. The framework does not explain WHY hydrogen's constants exhibit this alignment — that is a question about nature, not about mathematics. What the framework does is:

(a) Characterize the alignment precisely: R₆/4 = 91 vs 1/R∞ = 91.127, uniform multiplicative offset −0.139%.

(b) Show that the alignment propagates: every hydrogen series limit, every individual line, every denominator factor is captured by the repunit basis with the same residual.

(c) Extract the algebraic consequences: Pa-α = R₃ × Δ, Pa-γ = 3R₆, the denominator map — structures that exist in the integer domain and are invisible in the corrected domain.

The question "why does R₆ ≈ B?" is equivalent to "why does hydrogen's spectrum exhibit repunit alignment?" This is a physics question about the fine-structure of nature, not a mathematics question about the framework. The framework's scope is the characterization, not the explanation. It answers Question 2 of the Buried Question — definitively, within its own axioms — and leaves Question 1 (the universal, uncomputable question) to nature.

#### §5.5  Epistemological Boundaries

Three scope declarations govern the interpretation of all results in Parts III and IV.

**I.**  The framework derives 91; it does not derive R∞. The integer R₆/4 = 91 is produced by the sole axiom Rₙ = (3ⁿ − 1)/2 through four steps of exact arithmetic (Part I). The Rydberg constant R∞ = mₑe⁴/(8ε₀²h³c) is a product of fundamental physical constants that the ternary axiom does not generate. The proximity R₆/4 ≈ 1/R∞ (−0.139%) is a fact about nature, not an assumption of the framework. The framework's results exist independently of this proximity; the proximity makes them physically interpretable.

**II.**  The framework's outputs are exact integers. Pa-α = R₃ × Δ = 13 × 144 = 1872. This is not an approximation of anything — it is the exact product of two constants derived from the circle quadratic. Separately, CODATA measures the physical Paschen-alpha wavelength at 1875.1 nm. The two numbers are close (−0.165%). The framework does not claim they should be equal; it observes that they are near and characterizes the gap.

**III.**  The offset is a measured gap, not an omitted correction. The systematic −0.139% between framework integers and physical measurements is the distance between an algebraic constant and a laboratory value. It is not a rounding error, not an approximation, and not something that was neglected. A multiplicative factor κ = 1.00139 maps one domain to the other, but κ is not a correction — it is the bridge coefficient itself, and applying it converts exact integers into non-integer reals that no longer factorise over the framework's constants. The framework produces integers; physics produces measurements; the gap between them is the geometric floor of §5.3.

---

## PART III — CONSEQUENCES

All results below follow from the standard Rydberg formula plus the bridge (§5).

### §6  Master Formula

The standard Rydberg formula for hydrogen emission:

```
1/λ = R∞(1/m² − 1/n²)    →    λ = (1/R∞) × m²n²/(n² − m²)
```

Applying the bridge (1/R∞ → R₆/4):

```
λ₆(m,n) = R₆ × m²n² / [4(n² − m²)]
```

This inherits the −0.139% offset uniformly. Every predicted wavelength undershoots CODATA by the same multiplicative factor. No series-dependent or line-dependent correction is needed because the Rydberg formula is already exact (for infinite nuclear mass); the only approximation is in the prefactor.

### §7  Series Limits

As n → ∞, each series converges to λ∞(m) = m²R₆/4. The derivation is mechanical: in the master formula, 1/n² → 0, leaving m²R₆/4. The CODATA column uses m²/R∞ (infinite mass).

| Series | m | m²R₆/4 | FW (nm) | 1/R∞ (nm) | Offset |
|--------|---|--------|---------|-----------|--------|
| Lyman | 1 | 1²×364/4 | 91 | 91.127 | −0.139% |
| Balmer | 2 | 2²×364/4 | 364 | 364.507 | −0.139% |
| Paschen | 3 | 3²×364/4 | 819 | 820.140 | −0.139% |
| Brackett | 4 | 4²×364/4 | 1456 | 1458.03 | −0.139% |
| Pfund | 5 | 5²×364/4 | 2275 | 2278.17 | −0.139% |

Every series limit undershoots CODATA by exactly −0.139%. This is not a coincidence: it is the bridge offset propagating through a linear prefactor. The identity of the offset across all five series is a consistency check, not an independent validation.

### §8  Balmer Series (m = 2)

The Balmer formula reduces to λ = R₆n²/(n² − 4), since m²/4 = 1 cancels. The denominator column (n² − 4) shows the divisor that produces each wavelength:

| n | Line | n²−4 | Predicted | CODATA | Error |
|---|------|------|-----------|--------|-------|
| 3 | H-α | 5 | 655.20 | 656.281 | −0.165% |
| 4 | H-β | 12 | 485.33 | 486.135 | −0.165% |
| 5 | H-γ | 21 | 433.33 | 434.047 | −0.164% |
| 6 | H-δ | 32 | 409.50 | 410.174 | −0.164% |

All four errors are −0.165%, slightly larger than the series-limit offset (−0.139%) because CODATA visible-line wavelengths are measured in air for hydrogen (finite proton mass plus air refractive index). The error uniformity confirms the systematic nature of the offset.

### §9  Paschen Series (m = 3)

The Paschen formula is λ = 9R₆n²/[4(n² − 9)]. This is the first infrared series:

| n | Line | n²−9 | Predicted | CODATA | Error |
|---|------|------|-----------|--------|-------|
| 4 | Pa-α | 7 | 1872.00 | 1875.1 | −0.165% |
| 5 | Pa-β | 16 | 1279.69 | 1281.8 | −0.165% |
| 6 | Pa-γ | 27 | 1092.00 | 1093.8 | −0.165% |

#### §9.1  Pa-α = R₃ × Δ (Integer Identity)

For n = 4, the denominator n² − 9 = 7. The factorization R₆ = 4 × 7 × 13 causes exact cancellation with the 4 × 7 in the denominator:

```
λ = (4×7×13) × 9 × 16 / (4×7) = 13 × 144 = R₃ × Δ = 1872
```

The derivation chain: π = 14 is a root of x² − 40x + 364 = 0, whose discriminant is Δ = 144. The other factor R₃ = 13 is the ternary radian, itself a repunit. The Paschen-alpha wavelength is the product of two quantities derived directly from the circle quadratic. CODATA Pa-α = 1875.1 nm; error = −0.165%.

#### §9.2  Pa-γ = 3R₆ (Integer Identity)

For n = 6, the denominator n² − 9 = 27 = 3³. The simplification:

```
λ = 9 × 364 × 36 / (4 × 27) = 364 × 3 = 1092
```

Pa-γ is exactly three times the full circle. The factor 3 = TERNARY_BASE. CODATA Pa-γ = 1093.8 nm; error = −0.165%.

#### §9.3  Denominator Sequence

The Paschen denominators (n² − 9) for n = 4, 5, 6, 7 produce 7, 16, 27, 40. All four are framework constants: coprime generator 7, the square 4², the sponge root DISCRIMINANT_2_SQRT = 27, and REPUNIT_4 = 40. No other series' denominator sequence hits this density.

### §10  Brackett (m = 4) and Pfund (m = 5)

| Line | n→m | n²−m² | Predicted | CODATA | Error |
|------|-----|-------|-----------|--------|-------|
| Br-α | 5→4 | 9 | 4044.44 | ≈4052 | −0.17% |
| Pf-α | 6→5 | 11 | 7445.45 | ≈7460 | −0.19% |

Brackett Br-α: λ = 16R₆ × 25 / [4 × 9] = 4R₆ × 25/9 = 4044.44 nm. Denominator 9 = 3².

Pfund Pf-α: λ = 25R₆ × 36 / [4 × 11] = 7445.45 nm. Denominator 11 is a coprime generator. Both errors are consistent with the combined offset (−0.17% to −0.19%).

### §11  Structural Map

The denominators (n² − m²) across all five series systematically produce framework constants:

| Series | n=m+1 | n=m+2 | n=m+3 | n=m+4 | Framework constants hit |
|--------|-------|-------|-------|-------|------------------------|
| Lyman (m²=1) | 3 | 8 | 15 | 24 | TERNARY_BASE, PENTADECAGON |
| Balmer (m²=4) | 5 | 12 | 21 | 32 | DISCRIMINANT_SQRT=12 |
| Paschen (m²=9) | 7 | 16 | 27 | 40 | 7, 4², √Δ₂=27, R₄=40 |
| Brackett (m²=16) | 9 | 20 | 33 | 48 | 3²=9 |
| Pfund (m²=25) | 11 | 24 | 39 | 56 | 11, 2×Z₂₈=56 |

Paschen (m² = 9) is the densest: all four of its first denominators are framework constants. This follows from the ternary power structure: 16 = 3² + 7, 27 = 3³, 40 = 3⁴ + 1 = R₄.

---

## PART IV — APPARATUS

### §12  Proposed Constants File Additions (§19d)

The following additions to unified_constants.rs are recommended:

```rust
pub const CODATA_PASCHEN_ALPHA_NM: f64 = 1_875.1;
pub const CODATA_BRACKETT_ALPHA_NM: f64 = 4_052.0;
pub const CODATA_PFUND_ALPHA_NM: f64 = 7_460.0;
```

A universal helper replaces per-series test functions:

```rust
fn rydberg_predict(m: u32, n: u32) -> f64 {
    QUAD_PRODUCT as f64 * (m*m*n*n) as f64
        / (4 * (n*n - m*m)) as f64
}
```

Tests should assert Tier 2 tolerance (< 1%) for all Paschen, Brackett, and Pfund alpha lines.

### §13  Cross-References

- **TM-2026-017 §16:** UV Spectral Correspondence. Establishes R₆ as the Balmer constant analog. This document generalizes to all series.
- **TM-2026-026 v1.2:** UV Spectral Protocol. Orbifold topology, vacuum bias decomposition, secondary integers (222, 308, 311 nm). UV-specific; not superseded.
- **The Buried Question v9.3.3ad:** Establishes the entropy infimum, cone-point floor, and the distinction between the scoped question (answerable) and the universal question (uncomputable). Provides the epistemological framework for §5.3–5.4.
- **Kolmogorov Companion:** Maps the Buried Question to Kolmogorov complexity, KS entropy, and Shannon's theory. Establishes that Hₙₓ is a computable upper bound on K(x).
- **unified_constants.rs §19b:** CODATA 2022 bridge. Lyman/Balmer references, tolerance tiers, VACUUM_BIAS_NUM/DEN.
- **unified_constants.rs §6:** UV spectral wavelengths. The four primary wavelengths (91, 182, 286, 364 nm) are the Lyman series subset of the master formula.

### §14  Conclusion

The derivation chain is: ternary axiom (Rₙ = (3ⁿ−1)/2) → repunit family → circle quadratic x² − 40x + 364 = 0 → quarter-turn R₆/4 = 91 → bridge identification 91 ≈ 1/R∞ → master formula λ = R₆m²n²/[4(n² − m²)]. Steps 1–4 are exact integer mathematics from the sole axiom. Step 5 is a single empirical identification (−0.139%). Steps 6–11 are standard Rydberg mechanics with that identification applied.

The spectral residual (κ = 91.127/91 = 1.00139) is the geometric floor of the hydrogen spectral walk — the cone-point obstruction of the Buried Question applied to spectroscopy. It is strictly positive, uniform across all series, and irreducible within the integer domain. It is the measured distance between exact algebraic constants and laboratory wavelengths — not an omitted correction. The framework produces integers; physics produces measurements; the gap is characterized, not apologised for. The framework characterizes hydrogen's repunit alignment; it does not explain why nature exhibits that alignment. The ocean is the repunit formula. The hydrogen spectrum is one wave. The cone point is the rock that remains.

The most significant structural result is Pa-α = R₃ × Δ = 13 × 144 = 1872 nm: the radian and discriminant of the circle quadratic multiply to predict the Paschen-alpha wavelength. The master formula approximates all five named hydrogen series (Lyman through Pfund) to within 0.2% of CODATA 2022 measurements. The offset is systematic, uniform, and fully characterized.

---

PROPRIETARY AND CONFIDENTIAL
Capomastro Holdings Ltd. — Applied Physics Division
Patent(s) Pending — All Rights Reserved
