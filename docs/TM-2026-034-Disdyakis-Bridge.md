# TM-2026-034 — THE DISDYAKIS BRIDGE

## Icosahedral Geometry in the 364° Circle

**How the re-priming of the half-turn from 180 = 2²×3²×5 to 182 = 2×7×13 threads the golden ratio through the PlenumNET coprime generators 7 and 11 (intrinsically) and 13 (via the global angular metric)**

---

**Capomastro Holdings Ltd.**
Applied Physics Division — Sherwood Park, Alberta, Canada
Version 1.21 — April 2026
Patent Pending

---

### Revision History

| Version | Date | Changes |
|---|---|---|
| 1.0 | April 2026 | Initial derivation |
| 1.1 | April 2026 | Added §10 (survey of all icosahedral Catalans); corrected overclaim re uniqueness; incorporated observation that 15 = 3×5 unites Platonic and coprime prime families |
| 1.2 | April 2026 | Clarified generator hierarchy (intrinsic vs global vs basis-dependent); added derivation method for §10; corrected scope to "five of six" icosahedral Catalans; clarified Schwarz triangle terminology; refined abstract and §7 |
| 1.21 | April 2026 | Added basis-preference theorem (§6.5); quadratic reciprocity proof that hierarchy is forced (§6.6); resolved Q1 (Appendix A: R² survey of all 11 non-chiral Archimedeans — four framework constants surface as integer parts); resolved Q4 (Appendix B: negative result with Legendre symbol proof — 13 is inert in ℤ[φ]); closed Q5 as non-structural; added Q6 (derived rank-3 hyperbolic Coxeter group (7,11,13) and Brieskorn sphere connection, cross-ref TM-2026-COX-MATH) |

---

## §1. Introduction

This document establishes a rigorous structural connection between icosahedral geometry and the Salvi Framework's 364° circle. The central result is not a numerical coincidence but an algebraic re-priming: the replacement of the standard half-turn 180 = 2² × 3² × 5 with the PlenumNET half-turn 182 = 2 × 7 × 13 threads the golden ratio φ through the coprime generators {7, 13} rather than the Platonic symmetry primes {3, 5}.

The disdyakis triacontahedron — the dual of the truncated icosidodecahedron, with 120 congruent scalene triangle faces — serves as the primary test case. We derive from first principles that every metric quantity of its parent polyhedron lies in the ring ℤ[φ], and that the circumradius squared is exactly:

> **R² = 14 + 5φ, where 14 = π in PlenumNET.**

The face angle cosines are exact elements of ℚ(φ), with denominators equal to the vertex counts of the three symmetry orbits (12, 20, 30), whose least common multiple is 60 = |I| (the order of the icosahedral rotation group).

The coprime generators surface through a hierarchy of algebraic mechanisms with decreasing invariance:

- **11** appears as a ℤ[φ]-norm (basis-independent algebraic invariant) — intrinsic to the solid.
- **7** appears as a ℤ[φ] coefficient (basis-dependent but canonical in the standard basis {1, φ}) — intrinsic to the solid.
- **15** appears as a ℚ(√5) rational coefficient (dependent on the choice of basis {1, √5} vs {1, φ}) — a property of the representation, not an invariant.
- **13** appears only through the triangle angle sum 182 = 2 × 7 × 13 — a global property of the PlenumNET angular metric, not a local property of the polyhedron.

A survey of all five icosahedral Catalan solids lying in ℚ(φ) (§10) confirms that the disdyakis triacontahedron has the richest concentration of coprime generators, while the sixth icosahedral Catalan (the pentagonal hexecontahedron) lies outside ℚ(φ) and is excluded from this analysis.

All computations were performed from explicit vertex coordinates, verified against SciPy convex hull decomposition, with numerical precision to 10⁻⁹ or better.

---

## §2. PlenumNET Axioms (Relevant Subset)

**Axiom 1.** Full circle = 364°. Half-turn = 182°.

**Axiom 2.** π = 14 custom radians. One custom radian = 13°. Thus 2π = 28 custom radians = 364°.

**Axiom 3.** The arc equation arc² − 832·arc + 118,300 = 0 has roots 182 and 650.

**Axiom 4.** The coprime generators are (7, 11, 13, 15) with lcm = 15,015.

**Conversion factor.** Standard degrees → PlenumNET degrees: multiply by 364/360 = 91/90.

**Consequence.** The triangle angle sum in PlenumNET is 180° × 91/90 = **182° = 2 × 7 × 13**.

---

## §3. The Golden Ratio — Standard Definitions

The golden ratio φ = (1+√5)/2 ≈ 1.61803398875 satisfies:

> φ² = φ + 1,    1/φ = φ − 1,    1/φ² = 2 − φ

These identities are used throughout without further citation.

---

## §4. The Golden Angle Re-Primed

### 4.1 Standard Golden Angle

The golden angle partitions a full circle 360° in the ratio 1 : φ. The smaller (canonical) golden angle is:

> θ_std = 360° / φ² = 180°(3−√5) ≈ 137.508°

### 4.2 PlenumNET Golden Angle

In the 364° circle, the same proportional partition gives:

> θ_P = 364° / φ² = **182(3−√5)** ≈ 139.036°

The complement is:

> 364° − θ_P = 182(√5−1) = 364/φ ≈ 224.965°

Both arcs are algebraic multiples of 182 — the half-turn and root of the unified equation.

### 4.3 The Re-Priming

The structural content of the 180 → 182 shift is in the prime factorisation:

| | Standard | PlenumNET |
|---|---|---|
| Half-turn | 180 = 2² × 3² × 5 | 182 = 2 × 7 × 13 |
| Golden angle | 180(3−√5) | 182(3−√5) |
| Complement | 360/φ | 364/φ |
| Primes in half-turn | {2, 3, 5} | {2, 7, 13} |

The irrational factor (3−√5) is unchanged. Only the integer lattice through which it acts is replaced. The 180 → 182 shift does not merely rescale the golden angle — it **re-primes** it: the algebraic foundation swaps from the Platonic symmetry primes {2, 3, 5} to the coprime generators {2, 7, 13}.

Note also that 182 = 14 × 13 = π × (1 custom radian), anchoring the golden angle directly to both PlenumNET constants.

**Observation.** The coprime generator 15 = 3 × 5 is the product of the two smallest odd Platonic symmetry primes (3-fold and 5-fold axes of the icosahedron). Thus the coprime quadruple {7, 11, 13, 15} contains both the re-primed primes {7, 13} and, through 15, a bridge back to the Platonic primes {3, 5}. The quadruple unites both families.

---

## §5. The Truncated Icosidodecahedron in ℤ[φ]

### 5.1 Vertex Coordinates

The truncated icosidodecahedron has 120 vertices generated as all cyclic permutations with all sign combinations of five base triples (where φ = (1+√5)/2):

> (1/φ, 1/φ, 3+φ),  (2/φ, φ, 1+2φ),  (1/φ, φ², 3φ−1)
>
> (2φ−1, 2, 2+φ),  (φ, 3, 2φ)

Computational verification: SciPy ConvexHull returns 62 faces (12 decagons, 20 hexagons, 30 squares), confirming the known face inventory.

### 5.2 Circumradius: R² = 14 + 5φ

**Derivation.** From the first base triple (1/φ, 1/φ, 3+φ):

> R² = 2/φ² + (3+φ)²

Using 1/φ² = 2−φ (from φ² = φ+1) and expanding (3+φ)² = 9 + 6φ + φ² = 9 + 6φ + φ + 1 = 10 + 7φ:

> R² = 2(2−φ) + (10 + 7φ) = 4 − 2φ + 10 + 7φ = **14 + 5φ**

Numerically: R² = 14 + 5(1.6180…) = 22.0902…

> **The integer coefficient is 14 = π in PlenumNET.** The circumsphere of the most face-diverse Archimedean solid encodes PlenumNET's fundamental constant as the integer part of R² in ℤ[φ].

### 5.3 Face-Centre Distances: The Complete ℤ[φ] Lattice

Every face-centre distance squared lies in ℤ[φ] = {a + bφ : a, b ∈ ℤ}:

| Quantity | a + bφ | Numerical | Symmetry | Face count |
|---|---|---|---|---|
| d₁₀² (decagon) | 10 + 5φ | 18.090 | 5-fold | 12 |
| d₆² (hexagon) | 6 + 9φ | 20.562 | 3-fold | 20 |
| d₄² (square) | 10 + 7φ | 21.326 | 2-fold | 30 |
| R² (circumradius) | **14** + 5φ | 22.090 | — | — |

The integer coefficient 14 appears only in R².

### 5.4 Defect Structure

The differences R² − d² reveal a geometric descent governed by φ:

> R² − d₁₀² = (14+5φ) − (10+5φ) = **4** (φ terms cancel exactly)
>
> R² − d₆²  = (14+5φ) − (6+9φ) = 8 − 4φ = **4/φ²**
>
> R² − d₄²  = (14+5φ) − (10+7φ) = 4 − 2φ = **2/φ²**

The defect ratio is:

> (R²−d₁₀²) : (R²−d₆²) : (R²−d₄²) = 4 : 4/φ² : 2/φ² = **2φ² : 2 : 1**

The φ-dependent defects form a geometric series in 1/φ², with the decagonal face producing a pure-integer defect (the φ terms annihilate).

---

## §6. Disdyakis Triacontahedron Face Angles

### 6.1 Construction

The disdyakis triacontahedron is the polar dual of the truncated icosidodecahedron with respect to its circumsphere. Its 62 vertices lie in three orbits under icosahedral symmetry: 12 on 5-fold axes (from decagonal face centres), 20 on 3-fold axes (hexagonal), and 30 on 2-fold axes (square). Each of its 120 congruent scalene triangle faces connects one vertex from each orbit.

The dual vertex distances are r₅ = R²/d₁₀ = (14+5φ)/√(10+5φ), r₃ = R²/d₆ = (14+5φ)/√(6+9φ), and r₂ = R²/d₄ = (14+5φ)/(3+φ) = **(103+√5)/22**.

### 6.2 Schwarz Triangle and Central Angles

The fundamental domain of icosahedral symmetry on the unit sphere is the Schwarz triangle with vertex angles π/5, π/3, π/2 at the 5-fold, 3-fold, and 2-fold symmetry poles respectively. The sides of this spherical triangle are the **central angles** (angular separations as seen from the origin) between adjacent symmetry axes. These are computed via the spherical law of cosines:

> cos(θ₃₂) = cos(π/5)/sin(π/3)     → θ₃₂ ≈ 20.905° (3-fold ↔ 2-fold central angle)
>
> cos(θ₅₂) = cos(π/3)/sin(π/5)     → θ₅₂ ≈ 31.717° (5-fold ↔ 2-fold central angle)
>
> cos(θ₅₃) = cos(π/5)cos(π/3) / [sin(π/5)sin(π/3)]  → θ₅₃ ≈ 37.377° (5-fold ↔ 3-fold central angle)

These central angles, combined with the dual vertex distances r₅, r₃, r₂, yield the edge lengths via the law of cosines in ℝ³:

> e²ᵢⱼ = rᵢ² + rⱼ² − 2rᵢrⱼcos(θᵢⱼ)

The face angles then follow from the planar law of cosines on the resulting triangle.

### 6.3 Exact Cosine Expressions in ℚ(φ)

**Principal result.** The three face angle cosines are exact elements of the field ℚ(φ) = ℚ(√5):

| Angle at | cos(α) in ℤ[φ] form | cos(α) in ℚ(√5) form | α (std °) | α (364° circle) |
|---|---|---|---|---|
| 5-fold vertex | **(2 + 5φ) / 12** | (9 + 5√5) / 24 | 32.770° | 33.134° |
| 3-fold vertex | **(17 − 4φ) / 20** | (15 − 2√5) / 20 | 58.238° | 58.885° |
| 2-fold vertex | **(7 − 4φ) / 30** | (5 − 2√5) / 30 | 88.992° | 89.981° |
| **Sum** | — | — | **180.000°** | **182.000°** |

Verification: all three expressions confirmed to machine precision (error < 10⁻⁹) against direct computation from vertex coordinates of the truncated icosidodecahedron and its polar dual.

### 6.4 Structural Analysis of the Cosine Expressions

**Denominators equal vertex counts.** The denominators 12, 20, 30 are respectively the count of 5-fold vertices (icosahedral), 3-fold vertices (dodecahedral), and 2-fold vertices (icosidodecahedral) of the disdyakis triacontahedron. Their least common multiple is:

> lcm(12, 20, 30) = **60 = |I|** (the order of the icosahedral rotation group)

**Generator hierarchy.** The coprime generators appear through mechanisms of decreasing algebraic invariance:

**(a) ℤ[φ]-norm (basis-independent).** The algebraic norm in ℤ[φ] is N(a+bφ) = |a² + ab − b²|. This is an invariant: it does not depend on the choice of integral basis for the ring.

> N(2 + 5φ) = |4 + 10 − 25| = **11** (coprime generator)

The norm of the 5-fold numerator is 11. This is the most robust appearance of a coprime generator — it would be 11 regardless of whether we write the number in the basis {1, φ}, {1, √5}, or any other integral basis for ℤ[φ].

**(b) ℤ[φ] coefficient (canonical but basis-dependent).** In the standard basis {1, φ}, the integer coefficients are determined by the choice of φ = (1+√5)/2 as the fundamental unit. In this basis:

> cos(α₂) = (**7** − 4φ) / 30

The coefficient 7 is a coprime generator. This is a canonical feature of the algebraic number in the standard basis, but it is not an invariant: in the alternative basis {1, √5}, the same number becomes (5 − 2√5)/30, and 7 does not appear.

**(c) ℚ(√5) rational coefficient (basis-dependent).** In the basis {1, √5}:

> cos(α₃) = (**15** − 2√5) / 20

The integer 15 is a coprime generator. However, in the ℤ[φ] basis, this same cosine is (17 − 4φ)/20, where 15 does not appear. The appearance of 15 is thus a property of the {1, √5} representation, not an algebraic invariant. It is recorded here for completeness but carries less structural weight than the appearances of 11 and 7.

**ℤ[φ]-norm table:**

| Numerator | N(a+bφ) | Significance |
|---|---|---|
| 2 + 5φ (5-fold) | **11** | Coprime generator (basis-independent) |
| 7 − 4φ (2-fold) | **5** | Factor of coprime generator 15 |
| 17 − 4φ (3-fold) | 205 = 5 × 41 | — |

**Triangle angle sum.** In the 364° circle, every planar triangle has angle sum:

> 180° × 91/90 = **182° = 2 × 7 × 13**

This introduces the coprime generator **13** into the angular data. However, 13 is a property of the PlenumNET angular metric (the 364° circle), not a property of the disdyakis triacontahedron's Euclidean geometry (which sums to 180° regardless of angular convention). The generator 13 enters through the measurement convention, not through the polyhedron's combinatorial or algebraic structure.

### 6.5 Basis Preference: {1, φ} is Geometrically Natural

The cosine expressions can be written in two standard bases for ℚ(√5):

| Angle | ℤ[φ] basis {1, φ} | denom | ℚ(√5) basis {1, √5} | denom |
|---|---|---|---|---|
| 5-fold | (2 + 5φ) / **12** | **12** | (9 + 5√5) / 24 | 24 |
| 3-fold | (17 − 4φ) / **20** | **20** | (15 − 2√5) / **20** | 20 |
| 2-fold | (7 − 4φ) / **30** | **30** | (5 − 2√5) / **30** | 30 |

The denominator = vertex count correspondence (12, 20, 30) holds **only in the ℤ[φ] basis**. In ℚ(√5), the 5-fold denominator doubles to 24 because gcd(9, 5, 24) = 1 — the fraction cannot be reduced. The other two denominators happen to survive unchanged because the ℤ[φ] → ℚ(√5) conversion produces a common factor of 2 in both numerator and denominator that cancels.

This is not a defect of the ℚ(√5) representation — it is a structural signal. The geometry of the disdyakis triacontahedron **selects** the basis {1, φ} as the natural one: it is the unique basis in which all three denominators equal the vertex orbit sizes and lcm to |I| = 60. The ℤ[φ] basis is not an arbitrary algebraic convenience; it is the representation preferred by the polyhedron itself.

This observation also clarifies the generator hierarchy. In the geometrically natural basis {1, φ}:

- **7** is visible as a coefficient (in cos(α₂) = (7−4φ)/30).
- **11** is visible as a norm (N(2+5φ) = 11, which is basis-independent in any case).
- **15** is *not* visible — it appears only in the ℚ(√5) basis, which the geometry does not prefer.

The basis-dependent appearance of 15 in (15−2√5)/20 is therefore a weaker structural claim than the appearances of 7 and 11, which survive in the geometrically selected basis.

### 6.6 The Hierarchy Is Forced by Quadratic Reciprocity

The generator hierarchy 11 > 7 > 15 > 13 is not merely an empirical observation — it is **determined by the splitting behaviour of primes in ℤ[φ]**.

The ring of integers of ℚ(√5) is ℤ[φ]. A rational prime p splits, is inert, or ramifies in ℤ[φ] according to the Legendre symbol (5/p), computed by Euler's criterion: (5/p) ≡ 5^((p−1)/2) mod p.

- If (5/p) = +1: p **splits**. There exist elements α ∈ ℤ[φ] with N(α) = p.
- If (5/p) = −1: p is **inert**. No element of ℤ[φ] has norm p.
- If p = 5: p **ramifies** (discriminant prime).

**Computation for each coprime generator's prime factors:**

| Prime | 5^((p−1)/2) mod p | (5/p) | Status in ℤ[φ] | Can appear as ℤ[φ]-norm? |
|---|---|---|---|---|
| 7 | 5³ ≡ 6 ≡ −1 (mod 7) | −1 | **Inert** | No |
| 11 | 5⁵ ≡ 1 (mod 11) | +1 | **Splits** | **Yes** — confirmed: N(2+5φ) = 11 |
| 13 | 5⁶ ≡ 12 ≡ −1 (mod 13) | −1 | **Inert** | No |
| 3 | 5¹ ≡ 2 ≡ −1 (mod 3) | −1 | **Inert** | No |
| 5 | — | 0 | **Ramifies** | Yes — confirmed: N(7−4φ) = 5 |

**Verification.** Exhaustive search over |a|, |b| ≤ 100 confirms: the equation |a² + ab − b²| = 7 has zero solutions; |a² + ab − b²| = 13 has zero solutions; |a² + ab − b²| = 11 has 68 solutions (e.g., a=2, b=5).

**Consequences for the hierarchy:**

- **11 splits** → it can and does appear as a basis-independent ℤ[φ]-norm. This is the strongest possible algebraic mechanism. No other coprime generator prime has this property.

- **7 is inert** → it **cannot** appear as a ℤ[φ]-norm. The strongest mechanism available to 7 is as a coefficient in the geometrically preferred basis {1, φ}. The hierarchy **11 > 7 is forced by number theory**, not by the geometry of any particular solid.

- **13 is inert** → same obstruction as 7. But 13 also fails empirically to appear as a coefficient in any of the disdyakis's algebraic quantities. Its only entry is through the global metric (182 = 2 × 7 × 13). The rank **7 > 13 is empirical** (7 appears as a coefficient, 13 does not); the exclusion of both from norm status is number-theoretic.

- **15 = 3 × 5** — both prime factors have restricted behaviour (3 inert, 5 ramified), consistent with 15 appearing only in the non-preferred basis.

The hierarchy restated with its provenance:

| Level | Generator | Mechanism | Why this level and no higher |
|---|---|---|---|
| 1 (strongest) | 11 | ℤ[φ]-norm | (5/11) = +1: **splits** — norm representation exists |
| 2 | 7 | ℤ[φ] coefficient | (5/7) = −1: **inert** — norm representation impossible |
| 3 | 15 | ℚ(√5) coefficient | Absent in preferred basis; prime factors 3,5 are inert/ramified |
| 4 (weakest) | 13 | Global metric only | (5/13) = −1: **inert**; empirically absent from all coefficients |

---

## §7. Synthesis: Three Levels of Connection

The structural connection between icosahedral geometry and PlenumNET operates at three distinct algebraic levels:

**Level 1 — The circumradius.** R² = 14 + 5φ contains π (PlenumNET) as its exact integer part in ℤ[φ]. This is an identity derived from the vertex coordinates of the truncated icosidodecahedron — the most face-diverse Archimedean solid — not an approximation.

**Level 2 — The re-priming.** The golden ratio's geometric action (partitioning circles, governing icosahedral symmetry, determining face angles) is algebraically identical in both systems. The difference is the integer lattice through which it acts:

> Standard: 180(3−√5) → primes {2, 3, 5} → Platonic symmetry orders
>
> PlenumNET: 182(3−√5) → primes {2, 7, 13} → coprime generators

The irrational factor (3−√5) is invariant. Only the half-turn constant changes, and with it, the prime factorisation of the entire angular framework.

**Level 3 — The ℤ[φ] lattice.** Every metric quantity of the truncated icosidodecahedron — circumradius, face-centre distances, face angle cosines — lies in ℤ[φ] or ℚ(φ). The coprime generators surface within these expressions through a hierarchy of mechanisms:

| Generator | Where it appears | Mechanism | Invariance |
|---|---|---|---|
| **11** | N(2 + 5φ) = 11 | ℤ[φ]-norm of 5-fold numerator | Basis-independent; (5/11)=+1 splits (§6.6) |
| **7** | cos(α₂) = (7−4φ)/30 | ℤ[φ] coefficient | Preferred basis (§6.5); (5/7)=−1 blocks norm (§6.6) |
| **15** | cos(α₃) = (15−2√5)/20 | ℚ(√5) rational coefficient | Non-preferred basis only (§6.5) |
| **13** | 182 = 2 × 7 × 13 | Half-turn prime factor | Global metric; (5/13)=−1 blocks norm (§6.6) |

The hierarchy is: **11 (strongest, invariant) > 7 (canonical) > 15 (representation-dependent) > 13 (global only).**

---

## §8. Honest Boundaries

This document does **not** claim:

- That the number 92 (Johnson solids = uranium atomic number) is anything other than a coincidence. That path was investigated and closed.

- That the individual face angles are rational multiples of 13°. They are not; the custom-radian values 2.549…, 4.530…, 6.922… are irrational.

- That the coprime quadruple (7, 11, 13, 15) generates new Johnson solids or Catalan solids. Polyhedral symmetry groups are constrained to rotation orders 2, 3, 4, 5, which excludes 7, 11, 13, 15.

- That the coprime generators are unique to the disdyakis triacontahedron. The survey in §10 demonstrates that **7 appears in 4 of 5 icosahedral Catalans** lying in ℚ(φ). The disdyakis is distinguished by concentration (3/4 generators) and by quality (unique appearance of 11 as a ℤ[φ]-norm; unique denominator-vertex-count correspondence; unique R² = 14+5φ), not by exclusive possession.

- That all four coprime generators appear through equally robust mechanisms. The hierarchy in §7 makes clear that 11 is the strongest (basis-independent norm), 7 is canonical but basis-dependent, 15 is representation-dependent, and 13 is global rather than intrinsic.

- That placing the framework numbers {7, 11, 13, 14, 15} as Coxeter exponents in arbitrary higher-rank matrices constitutes a structural derivation. The rank-3 hyperbolic Coxeter group (7, 11, 13) is genuinely derived from the axiom (Q6), but extending to rank 4 or higher requires assignment rules (e.g., m_ij = max(a,b)) that are not derivable from the framework.

- That the appearance of 11 and 7 in the face angle cosines is *caused by* PlenumNET's axioms. These are properties of icosahedral geometry that exist independently. What PlenumNET provides is the **framework in which these numbers are system-significant** — they are coprime generators in the Salvi Framework, not arbitrary integers.

What this document **does** establish:

1. R² = 14 + 5φ is an exact algebraic identity (14 = π in PlenumNET).
2. The re-priming 180 → 182 is a real algebraic operation that changes the prime factorisation of the half-turn and threads the golden ratio through {2, 7, 13}.
3. The face angle cosines of the disdyakis triacontahedron are exact elements of ℚ(φ) with denominators equal to the vertex counts of each symmetry orbit.
4. The defect structure R² − d² = {4, 4/φ², 2/φ²} in ratio 2φ²:2:1 governs how the three face types deviate from the circumsphere.
5. The coprime generator 11 appears as a ℤ[φ]-norm (basis-independent), 7 appears as a ℤ[φ] coefficient (canonical), and 13 enters through the global angular metric.
6. The generator 7 is pervasive across the icosahedral Catalan family (§10), while 11 is unique to the disdyakis.
7. The hierarchy 11 > 7 > 15 > 13 is forced at its top two levels by quadratic reciprocity: (5/11) = +1 (splits, norm possible) vs (5/7) = (5/13) = −1 (inert, norm impossible) (§6.6).
8. Across all 11 non-chiral Archimedean solids, the integer parts of R² include exactly four PlenumNET framework constants: 14 = π, 13 = radian unit, 11 = coprime generator, 7 = coprime generator (Appendix A).
9. The derived triple (7, 11, 13) forms a rank-3 hyperbolic Coxeter group — an infinite reflection group whose exponents are forced by the repunit axiom, not chosen (Q6).

---

## §9. Open Questions

**Q1. [RESOLVED — see Appendix A.]** Among all 11 non-chiral Archimedean solids, R² = 14 + 5φ is unique in having integer part 14 = π. The complete set of integer parts is {2, 2, 5, 5, 5, 7, 7, 10, 11, 13, 14} — four PlenumNET framework constants (7, 11, 13, 14) appear, distributed across the three symmetry families. No solid gives 91, 182, 286, or 364.

**Q2.** The defect ratios 2φ² : 2 : 1 governing the face-distance deviations from the circumsphere — do these connect to the dual-circle architecture Z₂₇ × Z₂₈?

**Q3.** Can the Resonance Proof Engine's Three.js visualisation incorporate the disdyakis triacontahedron as a geometric witness for the 14 + 5φ identity? Suggested rendering: vertices coloured by symmetry orbit (e.g., red for 5-fold, green for 3-fold, blue for 2-fold) with the circumsphere R² = 14+5φ as a semi-transparent shell.

**Q4. [RESOLVED — see Appendix B.]** No intrinsic Euclidean invariant of the disdyakis triacontahedron yields 13. The Legendre symbol (5/13) = −1 proves that 13 is inert in ℤ[φ]: the norm equation |a²+ab−b²| = 13 has no integer solutions. This is a number-theoretic obstruction, not an empirical gap. The generator 13 enters exclusively through the PlenumNET conversion factor 91/90.

**Q5. [CLOSED — non-structural.]** The 3-fold numerator norm N(17−4φ) = 205 = 5 × 41. The prime 41 splits in ℤ[φ] (since (5/41) = +1), but neither factor connects to the coprime generators through a mechanism comparable to those in §6.6. The observation that 41 is the 13th prime depends on enumeration of primes, not on ring-theoretic structure.

**Q6.** The coprime triple (7, 11, 13) is fully derived from the repunit axiom R_n = (3ⁿ−1)/2 without arbitrary choices: R₃ = 13 (radian unit); the unified equation arc² − 832·arc + 118,300 = 0 yields roots 182 and 650; reducing modulo 364 gives arcs 182 and 286; dividing by 2·13 = 26 gives the ratio 7:11; adjoining R₃ = 13 completes the pairwise coprime triple. This triple satisfies the hyperbolicity condition for triangle Coxeter groups:

> 1/7 + 1/11 + 1/13 = 311/1001 < 1

where the denominator 1001 = 7×11×13 is the Hamiltonian cycle length of the coprime walk (TM-2026-017). The group ⟨r₁, r₂, r₃ | r²ᵢ = (r₁r₂)⁷ = (r₁r₃)¹¹ = (r₂r₃)¹³ = 1⟩ is therefore a well-defined infinite hyperbolic Coxeter group of rank 3, derived entirely from the axiom.

The numbers 14 (= π) and 15 (pentadecagon) appear elsewhere in the framework but do **not** naturally participate in a higher-rank Coxeter matrix without introducing an arbitrary assignment rule (e.g., m_ij = max(a,b)). Such rules are not derivable from the axiom and are excluded from the core derivation (see TM-2026-COX-MATH §6).

Separately, the Brieskorn sphere Σ(7,11,13) — the link of the singularity z₁⁷ + z₂¹¹ + z₃¹³ = 0 in ℂ³ — is a Seifert fibered 3-manifold whose base orbifold has the hyperbolic triangle group (7,11,13) as its orbifold fundamental group. The (7,11) torus knot from the framework's 4D Clifford torus parametrisation lives on this manifold. Whether the hyperbolic Coxeter group (7,11,13) connects to the re-primed icosahedral geometry of the disdyakis triacontahedron remains open. (Note: H₃, the icosahedral Coxeter group, has exponents (2,3,5). The re-priming replaces {3,5} with {7,13} in the angular metric but does not change the solid's symmetry group. A direct algebraic bridge between H₃ and the (7,11,13) triangle group is unestablished.)

---

## §10. Survey: Icosahedral Catalan Solids in ℚ(φ)

### 10.0 Scope and Method

There are six Catalan solids with icosahedral symmetry. Five of them — the rhombic triacontahedron, triakis icosahedron, pentakis dodecahedron, deltoidal hexecontahedron, and disdyakis triacontahedron — have coordinates and face angle cosines lying in ℚ(φ) = ℚ(√5). The sixth, the **pentagonal hexecontahedron** (dual of the snub dodecahedron), is chiral and its vertex coordinates involve a cubic irrational beyond ℚ(φ). Since the coprime generators {7, 11, 13, 15} are rational integers, they can only appear as coefficients or norms of elements in ℚ(φ) if the cosines lie in that field. The pentagonal hexecontahedron is therefore excluded from this survey.

**Method.** For each of the five solids, we:
1. Generated the Archimedean dual's vertex coordinates from the standard even-permutation-with-signs construction (the same method used in §5.1 for the truncated icosidodecahedron).
2. Computed the convex hull to identify face types and face-centre distances.
3. Found the minimum central angles between adjacent face centres of each type (these are the angular separations between symmetry axes, identical to the Schwarz triangle sides for the icosahedral group).
4. Computed dual vertex distances via the polar duality r = R²/d, where R is the circumradius of the Archimedean solid (shared with its dual) and d is the distance from the centre to the Archimedean face centre.
5. Computed edge lengths and face angles via the law of cosines in ℝ³.
6. Identified exact ℤ[φ] expressions for each face angle cosine by exhaustive search over (a + bφ)/d with |a|, |b| ≤ 40, d ≤ 60, verified to precision < 10⁻⁸.

### 10.1 Exact Cosine Expressions

**Rhombic triacontahedron** (dual of icosidodecahedron, 30 rhombus faces):

> cos(acute 63.4°) = (−1 + 2φ)/5 = √5/5,    N = 5
>
> cos(obtuse 116.6°) = (1 − 2φ)/5 = −√5/5,   N = 5

No coprime generators in any representation.

**Triakis icosahedron** (dual of truncated dodecahedron, 60 isosceles triangle faces):

> cos(base 30.5°) = **(7 + φ)/10** = (15 + √5)/20,   N = 55
>
> cos(apex 119.0°) = −3φ/10 = (−3 − 3√5)/20,          N = 9

Coprime generators: **7** as ℤ[φ] coefficient (canonical); 15 as ℚ(√5) rational coefficient (basis-dependent).

**Pentakis dodecahedron** (dual of truncated icosahedron, 60 isosceles triangle faces):

> cos(base 55.7°) = (5 − φ)/6 = (9 − √5)/12,          N = 19
>
> cos(apex 68.6°) = **(−8 + 9φ)/18** = (−7 + 9√5)/36,  N = 89

Coprime generators: **7** as ℚ(√5) rational coefficient (basis-dependent, as |−7| = 7).

**Deltoidal hexecontahedron** (dual of rhombicosidodecahedron, 60 kite faces):

> cos(narrow 67.8°) = **(−7 + 9φ)/20**,   N = 95
>
> cos(wide 87.0°) = **(7 − 4φ)/10**,      N = 5
>
> cos(obtuse 118.3°) = (−3 − 4φ)/20,      N = 5

Coprime generators: **7** as ℤ[φ] coefficient (canonical), appearing in two distinct cosines. Note: cos(wide) = (7−4φ)/10 shares the numerator 7−4φ with the disdyakis 2-fold cosine (7−4φ)/30, differing only in the denominator.

**Disdyakis triacontahedron** (dual of truncated icosidodecahedron, 120 scalene triangle faces):

> cos(5-fold 32.8°) = (2 + 5φ)/12,           **N = 11** (basis-independent)
>
> cos(3-fold 58.2°) = (17 − 4φ)/20 = (15 − 2√5)/20,   N = 205
>
> cos(2-fold 89.0°) = **(7** − 4φ)/30,       N = 5

Coprime generators: **11** as ℤ[φ]-norm (invariant), **7** as ℤ[φ] coefficient (canonical), 15 as ℚ(√5) coefficient (basis-dependent).

### 10.2 Distribution Summary

| Generator | Mechanism | Solids where it appears | Count |
|---|---|---|---|
| **7** | ℤ[φ] coeff or ℚ(√5) rat. part | Triakis ico., pentakis dod., deltoidal hex., disdyakis tri. | 4/5 |
| **11** | ℤ[φ]-norm (invariant) | Disdyakis triacontahedron only | 1/5 |
| **13** | Global metric (182 = 2×7×13) | All triangulated Catalans (triakis ico., pentakis dod., disdyakis tri.) via angle sum | 3/5 (global) |
| **15** | ℚ(√5) rational coefficient | Triakis icosahedron, disdyakis triacontahedron | 2/5 |

### 10.3 Uniqueness of the Disdyakis Triacontahedron

The disdyakis triacontahedron is distinguished from the other icosahedral Catalans in four ways:

1. **Generator concentration.** It is the only solid containing 3 of the 4 coprime generators in its face angle cosines (at any level of the hierarchy). No other solid has more than 2.

2. **Unique invariant.** It is the only solid where a coprime generator (11) appears as a ℤ[φ]-norm — an algebraic invariant independent of basis choice.

3. **Denominator = vertex count.** It is the only solid where the cosine denominators (12, 20, 30) equal the vertex orbit sizes, with lcm = 60 = |I|.

4. **Circumradius encodes π.** R² = 14 + 5φ, with 14 = π in PlenumNET. Among all 11 non-chiral Archimedean solids, no other has integer part 14 (Appendix A). The integer parts across the full family are {2, 2, 5, 5, 5, 7, 7, 10, 11, 13, 14} — four PlenumNET framework constants appear (7, 11, 13, 14), each in a different algebraic ring.

### 10.4 The Octahedral and Tetrahedral Catalans

The seven octahedral Catalan solids have face angle cosines in ℚ(√2) or ℚ. The single tetrahedral Catalan solid (triakis tetrahedron) has cosines in ℚ(√3) or ℚ. Since the coprime generators {7, 11, 13, 15} are not elements of these fields in any structurally meaningful way (the algebraic integers in ℚ(√2) and ℚ(√3) are generated by different primes), no coprime generators appear. The icosahedral family — specifically ℚ(φ) = ℚ(√5) — is the only Catalan family where the coprime generators surface in the face angle data.

---

## Appendix A: Resolution of Q1 — Circumradii of All Archimedean Solids

We compute R² for all 11 non-chiral Archimedean solids (the snub cube and snub dodecahedron involve cubic irrationals and are excluded). Each R² is expressed in its natural algebraic ring.

### A.1 Icosahedral Family (ℤ[φ])

| Solid | Representative vertex | Derivation | R² in ℤ[φ] | a |
|---|---|---|---|---|
| Icosidodecahedron | (0, 1, φ) | 0+1+φ² = 1+(φ+1) | 2 + φ | 2 |
| Truncated dodecahedron | (0, 1/φ, 2+φ) | 0+(2−φ)+(5+5φ) | 7 + 4φ | **7** |
| Truncated icosahedron | (0, 1, 3φ) | 0+1+9(φ+1) | 10 + 9φ | 10 |
| Rhombicosidodecahedron | (1, 1, φ³) | 2+(4φ+4+4φ+1) | 7 + 8φ | **7** |
| Truncated icosidodecahedron | (1/φ, 1/φ, 3+φ) | (4−2φ)+(10+7φ) | **14** + 5φ | **14** |

### A.2 Octahedral Family (ℤ[√2] or ℤ)

| Solid | Representative vertex | Derivation | R² | a |
|---|---|---|---|---|
| Cuboctahedron | (0, 1, 1) | 0+1+1 | 2 | 2 |
| Truncated octahedron | (0, 1, 2) | 0+1+4 | 5 | 5 |
| Truncated cube | (√2−1, 1, 1) | (3−2√2)+1+1 | 5 − 2√2 | 5 |
| Rhombicuboctahedron | (1, 1, 1+√2) | 1+1+(3+2√2) | 5 + 2√2 | 5 |
| Truncated cuboctahedron | (1, 1+√2, 1+2√2) | 1+(3+2√2)+(9+4√2) | **13** + 6√2 | **13** |

### A.3 Tetrahedral Family (ℤ)

| Solid | Representative vertex | R² | a |
|---|---|---|---|
| Truncated tetrahedron | (1, 1, 3) | 1+1+9 | **11** |

### A.4 Summary and Analysis

The complete set of integer parts across all 11 non-chiral Archimedean solids:

> {2, 2, 5, 5, 5, **7**, **7**, 10, **11**, **13**, **14**}

Four of the five PlenumNET framework constants {7, 11, 13, 14} appear as integer parts:

| Integer part | Solid(s) | Ring | PlenumNET identity |
|---|---|---|---|
| **14** | Truncated icosidodecahedron | ℤ[φ] | **π** |
| **13** | Truncated cuboctahedron | ℤ[√2] | Radian unit |
| **11** | Truncated tetrahedron | ℤ | Coprime generator |
| **7** | Truncated dodecahedron, rhombicosidodecahedron | ℤ[φ] | Coprime generator |

Each framework constant appears in a different algebraic ring: 14 in ℤ[φ] (icosahedral), 13 in ℤ[√2] (octahedral), 11 in ℤ (tetrahedral), 7 in ℤ[φ] (icosahedral). The only missing coprime generator is 15.

### A.5 Conclusion

The truncated icosidodecahedron is the **unique** Archimedean solid with R² integer part equal to π = 14. However, Q1 yields a stronger result than originally asked: the Archimedean solids, taken as a family, encode four of the five PlenumNET framework constants as integer parts of their circumradii squared, distributed across the three symmetry families (icosahedral, octahedral, tetrahedral). The larger system numbers (91, 182, 286, 364) do not appear as integer parts; they arise from combining π with the radian unit (e.g., 182 = 14 × 13). ∎

---

## Appendix B: Resolution of Q4 — 13 Cannot Appear Intrinsically (Proof)

### B.1 Statement

Can the coprime generator 13 surface from any intrinsic Euclidean invariant of the disdyakis triacontahedron, independently of the PlenumNET angular metric?

### B.2 Answer: No. This is a number-theoretic impossibility.

### B.3 Proof via inertness in ℤ[φ]

The ring of integers of ℚ(√5) is ℤ[φ]. The ℤ[φ]-norm is the only basis-independent algebraic invariant available for elements of ℚ(φ). A rational prime p can appear as a ℤ[φ]-norm if and only if p splits in ℤ[φ], which occurs if and only if the Legendre symbol (5/p) = +1.

By Euler's criterion:

> (5/13) ≡ 5⁶ mod 13

Computing: 5¹ = 5, 5² = 25 ≡ 12, 5³ ≡ 60 ≡ 8, 5⁴ ≡ 40 ≡ 1, 5⁵ ≡ 5, 5⁶ ≡ 25 ≡ 12 ≡ −1 (mod 13).

Therefore **(5/13) = −1**. The prime 13 is **inert** in ℤ[φ].

**Consequence.** The norm equation |a² + ab − b²| = 13 has no integer solutions. (Confirmed by exhaustive search over |a|, |b| ≤ 100: zero solutions.) There is no element α ∈ ℤ[φ] with N(α) = 13.

Since every metric quantity of the disdyakis triacontahedron (and every icosahedral solid) lies in ℤ[φ] or ℚ(φ), **13 cannot appear as a basis-independent algebraic invariant of any icosahedral solid**. This is not an empirical gap — it is a number-theoretic obstruction.

### B.4 Can 13 appear as a coefficient?

13 could in principle appear as a basis-dependent coefficient (a or b in the expression (a+bφ)/d), as 7 does. However, an exhaustive scan of all algebraic quantities of the disdyakis triacontahedron — face-centre distances, dual vertex distances, edge lengths, face angle cosines, and Descartes angular deficits — confirms that 13 does not appear as a ℤ[φ] coefficient in any of them. This is an empirical observation, not a number-theoretic necessity.

### B.5 Combinatorial invariants

The disdyakis triacontahedron has V = 62 = 2 × 31, E = 180 = 2² × 3² × 5, F = 120 = 2³ × 3 × 5. No factor of 13 appears.

### B.6 Descartes deficit (Gauss-Bonnet)

The total angular deficit for any convex polyhedron homeomorphic to S² is 720° = 4π. For the disdyakis triacontahedron:

| Vertex type | Count | Faces meeting | Angle per face | Angle sum | Deficit |
|---|---|---|---|---|---|
| 5-fold | 12 | 10 | 32.770° | 327.703° | 32.297° |
| 3-fold | 20 | 6 | 58.238° | 349.428° | 10.572° |
| 2-fold | 30 | 4 | 88.992° | 355.967° | 4.033° |

Total: 12(32.297) + 20(10.572) + 30(4.033) = **720.0°** ✓

In PlenumNET: 720° × 91/90 = 728° = 8 × 7 × 13. This does contain 13, but it holds for **every** convex polyhedron — it is a property of the sphere, not of the disdyakis triacontahedron.

### B.7 Conclusion

The generator 13 is excluded from the disdyakis triacontahedron's intrinsic algebraic data by two independent mechanisms: (1) the number-theoretic inertness of 13 in ℤ[φ], proven via the Legendre symbol (5/13) = −1, blocks it from appearing as a norm; (2) it empirically does not appear as a coefficient. The generator 13 enters the PlenumNET framework exclusively through the conversion factor 91/90 = (7 × 13) / (2 × 3² × 5), applied to the global angular metric.

This confirms and strengthens the hierarchy established in §6.6 and §7: the ranking 11 > 7 > 13 is forced at its endpoints by quadratic reciprocity. ∎

---

*Così sia, Fratello.*

**Capomastro Holdings Ltd.** — Applied Physics Division
TM-2026-034 v1.21 — Patent Pending