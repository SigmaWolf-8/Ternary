# PlenumNET — Unified Geometric Monograph

## From the Axiom π = 14 (radian = 13)

**TM-2026-017 v7.0 — April 2026**
**Capomastro Holdings Ltd. — Applied Physics Division**
**Sherwood Park, Alberta, Canada**

*All rights reserved © Capomastro Holdings Ltd 2026*
*Patent(s) Pending*

> *The circle is UV‑A, the square is UV‑C, and the Bézier arcs are the ozone layer.*

*Sed Quis Est Deus*
*Qui Commando IO*

---

## 1. The Axiom

The entire system follows from a single, self-contained rule:

> **π = 14 when the radian unit = 13**

Let the *custom degree* be the unit such that a full circle measures 364°. From the axiom:

- Half-circle (π in custom measure) = 14 × 13 = **182°** exactly.
- Full circle = 2π = 28 × 13 = **364°** exactly.

The unit circle (radius = 1) is the standard Euclidean circle. Its area is:

> Area(unit circle) = πr² = 14 × 1² = **14**

For radius r = √13 (the radian unit as length):

> Area = πr² = 14 × 13 = **182**

The unit circle squares with side √14 = √π (algebraic). The r = √13 circle squares with side √182 = √14 · √13 = √π · r (also algebraic). Both are impossible in standard geometry where √π is transcendental. See §3.

### 1.1 The Derivation Direction

This system is not a rescaling of standard mathematics. The derivation direction matters:

1. The repunit family Rₙ = (3ⁿ − 1)/2 is defined from pure integer arithmetic (§2.1).
2. R₃ = 13 and R₆ = 364 are outputs — not chosen, computed.
3. The circle quadratic x² − 40x + 364 = 0 is derived from R₃ and R₆. Its roots are 14 and 26.
4. π = 14 is the smaller root. This is the axiom's first consequence, not its input.

Standard π ≈ 3.14159... does not disappear. It is located as the bridge coefficient between the integer framework and the transcendental one:

> Sin₃₆₄(ρ) = sin_std(π_std · ρ / 14)

> d/dρ Sin₃₆₄(ρ) = (π_std / 14) · Cos₃₆₄(ρ)

The factor π_std/14 is where standard π lives: it is the conversion constant, not the fundamental constant. The integers 13, 14, 91, 182, 364 are fundamental. Standard π is derived from them as the ratio that maps the integer system onto the transcendental one.

A rescaling has no consequences beyond the rescaling. This axiom generates the coprime walk (1,001 positions), the UV spectral partition (91, 182, 286, 364 nm confirmed by NIST/CODATA to within +0.194% — decomposed as R∞ bias +0.139% plus reduced-mass correction +0.055%), a deterministic signaling architecture with algebraically guaranteed null channels (§19), and 15,015 conflict-free 2D positions from the coprime quadruple (§10). No rescaling produces these. ∎

---

## 2. The Generating System

The axiom is not arbitrary — it is the unique solution of a quadratic whose coefficients are base-3 repunits.

### 2.1 Repunit Definitions

Define the base-3 repunit:

> R_n = (3ⁿ − 1) / 2

| Repunit | Value | Base-3 |
|---------|-------|--------|
| R₃ | 13 | 111₃ |
| R₄ | 40 | 1111₃ |
| R₅ | 121 | 11111₃ |
| R₆ | 364 | 111111₃ |

The full circle R₆ = 364 is itself a base-3 repunit: 111111₃.

### 2.2 The Two-Equation Decomposition

The unified equation is the elimination of x from two simpler equations:

> x² − R₄·x + R₆ = 0 (the circle quadratic)
> 2c = x² + R₄ − x (the bridge — fuses circle to square)

By Vieta's formulas, the roots of the circle quadratic satisfy x₁ + x₂ = R₄ = 40 and x₁ · x₂ = R₆ = 364. The discriminant:

> Δ = R₄² − 4R₆ = 1600 − 1456 = 144 = 12²

The discriminant is a perfect square, guaranteeing integer roots:

> x₁ = (40 − 12) / 2 = **14** (= π)
> x₂ = (40 + 12) / 2 = **26** (= R₆/π)

The spread between the roots is √Δ = **12** = 26 − 14. This number reappears as the q-parameter of the canonical Plenum Square, in the superhub structure (§11), and as the amplitude ratio of the HModal signaling wave (§19).

### 2.2.1 The Discriminant as Amplitude Ratio

Define two amplitude levels from the circle measure and its discriminant:

> α = R₆ / Δ = 364 / 144 = 91/36
> β = R₆ / √Δ = 364 / 12 = 91/3

The ratio between them is exact:

> β / α = (364/12) / (364/144) = 144/12 = **12 = √Δ**

The excursion γ = β − α = 91/3 − 91/36 = **1001/36**, where 1001 = 7 × 11 × 13 — the coprime walk appears in the difference between the two amplitude levels. The discriminant of the circle quadratic is simultaneously the amplitude ratio of the natural signaling wave and the bridge between the two states encodes the torus topology. See §19 for the full signaling derivation.

### 2.3 The Unified Equation — Full Derivation

To eliminate x, define arc = x(x − 1) = 2c − R₄. Then x = (arc + R₆)/(R₄ − 1). Substituting back into the circle quadratic and multiplying through by (R₄ − 1)²:

**Linear coefficient:**

> 2R₆ − R₄(R₄ − 1) = 2(364) − 40(39) = 728 − 1560 = −832

**Constant term:**

> R₆(R₆ − R₄ + 1) = 364 × (364 − 40 + 1) = 364 × 325 = **118,300**

The factor 325 = R₆ − R₄ + 1 is pure repunit arithmetic — no π, no assumed constants.

This gives the unified equation:

> **arc² − 832·arc + 118,300 = 0**

The discriminant:

> Δ_arc = 832² − 4(118,300) = 692,224 − 473,200 = 219,024 = 468²

The roots:

> arc = (832 ± 468) / 2 = **650** or **182**

The meaningful root is arc = 182: the semicircle of the 364° circle.

### 2.4 Recovery of π from the Semicircle

From arc = 182, solve x² − x = arc:

> x² − x − 182 = 0
> Δ₂ = 1 + 4(182) = **729 = 27² = 3⁶**

> π = (1 + 27) / 2 = **14**

The secondary discriminant 729 = 3⁶ is the kernel sponge state width of TLSponge-385. It emerges not as a design parameter but as the discriminant of the equation that recovers π from the semicircle.

### 2.5 The 13-Multiple Pattern

Every key angular measure is a multiple of 13 (the radian unit):

| Angle | Factorization | Radians (custom) | Role |
|-------|---------------|------------------|------|
| 91° | 7 × 13 | 7 | Quarter-turn; C₁₈₂ control point |
| 143° | 11 × 13 | 11 | C₆₅₀ control point |
| 182° | 14 × 13 | 14 = π | Half-turn; red arc span |
| 364° | 28 × 13 | 28 = 2π | Full circle |

The control points sit at exactly 7 and 11 custom radians — both primes, both coprime factors of the toroidal walk (§10). This is not a design choice; it follows from the mid-angle construction applied to the 364° scaling. ∎

### 2.6 The Arc Roots and the Bézier Arcs

The two roots of the unified equation are 182 and 650. These are the degree spans of the two quadratic Bézier arcs:

- The **red arc** spans 182° — the semicircle, the meaningful root.
- The **green arc** spans 650° custom. Since 650 mod 364 = 286°, the arc physically subtends 286° of the circle, but its algebraic measure in the generating system is 650 — the complementary root.

Both arcs start at S = (1, 0) and converge at the 218.4° vertex P. The fact that their degree-spans are the two roots of a single quadratic whose coefficients are base-3 repunits is a structural identity: the same equation that generates π and the full circle also generates the two arc measures. ∎

### 2.7 The Complete Derivation Chain

| Step | Derivation | Result |
|------|-----------|--------|
| Definition | R_n = (3ⁿ − 1)/2 | Repunit family |
| Unified equation | arc² − 832·arc + 118,300 = 0 | Roots: 182, 650 |
| Semicircle | arc = 182 = R₆/2 = π(π−1) | The master constant |
| Center | c = (arc + R₄)/2 = 111 | Diameter (derived) |
| Secondary disc. | Δ₂ = 1 + 4·arc = 729 = 3⁶ | Sponge width |
| π | (1 + √729)/2 = 14 | Ternary pi |
| R₆/π | R₄ − π = 26 | Circle degrees per π |
| Radius | r = d/2 = 55.5 | Fundamental unit |
| Hexagon perimeter | 6r = 3d = 333 | Magic constant |
| Circumference | πd = 1554 = 28r = 2πr | π-th rung |

One equation → semicircle (182) → center (111) → discriminant (3⁶) → π (14) → all constants. ∎

---

## 3. Squaring the Circle — Exact Equality

### 3.0 The Unit Circle

For radius r = 1, the circle area is π = 14. The equal-area square has side:

> s = √14 = √π

This is algebraic — a root of s² − 14 = 0. In standard geometry, √π ≈ √3.14159... is transcendental and the construction is impossible.

### 3.1 The r = √13 Formulation

Choosing radius r = √13 (the radian unit as length) gives:

> Area = πr² = 14 × 13 = **182**

The equal-area square has side:

> s = √182 = √(14 × 13) = √14 · √13 = √π · r

The square is aligned with the quarter points: 0°, 91°, 182°, 273°.

### 3.2 The Transcendental Barrier — Resolved

In standard geometry, the area of the unit circle is π = 3.14159... and the side of the equal-area square is √π, which is transcendental. Lindemann proved in 1882 that π is transcendental, therefore squaring the circle is impossible with compass and straightedge.

In the PlenumNET system, the unit circle area is 14 (an integer) and the r = √13 circle area is 182 (an integer). Both square sides — √14 and √182 — are algebraic roots of integer polynomials. The transcendental barrier does not arise.

### 3.3 The Factorization

> √182 = √(2 × 91) = √2 · √91

The factor √2 is the side of the inscribed square (the chord subtending 90° standard). The factor √91 comes from the quarter-turn (91° = π/2 custom, 91 = 7 × 13). ∎

### 3.4 The Vesica Piscis Connection

The vesica piscis formed by two circles of radius √13 with centers one radius apart has height √3 · √13 = √39 and width √13. The ratio √3 : 1 is preserved. The squared circle side √182 = √14 · √13 combines the vesica's structural √3 with the factor √14 = √π — the system's defining constant expressed as a square root. ∎

---

## 4. Regular Polygons Inscribed in the 364° Circle

For a regular n-gon, the central angle is θ_n = 364°/n. All vertices lie on the unit circle. The polygon count — 13, from n = 3 through n = 15 — equals the radian unit.

| n | Name | Central angle | Offset | Special alignments |
|---|------|---------------|--------|--------------------|
| 3 | Triangle | 121.33° | 0° | 3-fold division of the circle |
| 4 | Square | 91° | 45.5° (= step/2) | Quarter points 0°, 91°, 182°, 273° |
| 5 | Pentagon | 72.8° | 0° | 218.4° = 3×72.8° is the arc convergence; involves φ |
| 6 | Hexagon | 60.67° | 30.33° (= 364/12) | Vertex at 182°; encodes √3 (vesica piscis) |
| 7 | Heptagon | 52° | 0° | 7×52 = 364; clean integer central angle |
| 8 | Octagon | 45.5° | 22.75° (= step/2) | Shares vertices with square at quarter points |
| 9 | Nonagon | 40.44° | 0° | 9 = 3²; links to triangle |
| 10 | Decagon | 36.4° | 0° | Both 182° and 218.4° are vertices |
| 11 | Hendecagon | 33.09° | 0° | Prime 11; coprime walk generator |
| 12 | Dodecagon | 30.33° | 15.17° (= 364/24) | 6×30.33° = 182°; vertex at half-turn |
| 13 | Tridecagon | **28°** | 0° | 28° = 2π; ties directly to π = 14 |
| 14 | Tetradecagon | **26°** | 13° (= 1 radian) | 26° = x₂ of circle quadratic; the π-gon |
| 15 | Pentadecagon | 24.27° | 0° | 15 = 3×5; bridges triangle and pentagon families |

The offsets encode half-steps of the polygon or a related polygon: the square and octagon use their own step/2, the hexagon and dodecagon use steps of the 12-gon and 24-division respectively, and the tetradecagon offsets by exactly one radian (13°). Polygons with offset 0° have their first vertex at S = (1, 0). These offsets are the generators of the antiprism half-step rotation in 3D (see TM-2026-035 §4).

### 4.1 The π-gon and the Pentadecagon

The tetradecagon (n = 14) has central angle 364/14 = **26°** — the second root of the circle quadratic x² − 40x + 364 = 0. Its step IS x₂. The number of sides IS π. This is the only polygon whose side count equals the system's circle constant.

The pentadecagon (n = 15) has central angle 364/15 = 24.27°. Its significance is structural: 15 = 3 × 5 bridges the triangle (n = 3) and pentagon (n = 5), the two simplest prime polygon families. It shares 3 vertices with the triangle, 5 with the pentagon, and 1 with each of the heptagon, hendecagon, and tridecagon (at 0°).

With the pentadecagon, the total polygon count is **13 = the radian unit**. The number of inscribed polygons equals the system's fundamental modulus. ∎

---

## 5. Quadratic Bézier Arcs — Unified Derivation

Both arcs are quadratic Bézier curves with the same fixed endpoints S and P:

> **B(t) = (1−t)²·S + 2(1−t)t·C + t²·P, t ∈ [0, 1]**

### 5.1 Endpoints from the Pentagon

- **S** = 0° vertex = (1, 0)
- **P** = 218.4° vertex = 3 × (364°/5). Using the golden ratio φ = (1+√5)/2:

> P = (−φ/2, −√(10−2√5)/4)

### 5.2 Angular Mapping — Standard to Custom

All four defining points of the Bézier construction sit on the unit circle. Their exact angular positions:

| Point | Standard angle | Custom angle | Coordinates | Custom radians |
|-------|----------------|--------------|-------------|----------------|
| S | 0° | 0° | (1, 0) | 0 |
| C₁₈₂ | 90° | **91° = 7 × 13** | (0, 1) | **7** |
| C₆₅₀ | 141.4286° = 990/7° | **143° = 11 × 13** | (−cos 3π/14, sin 3π/14) | **11** |
| P | 216° | 218.4° | (−φ/2, −√(10−2√5)/4) | 218.4/13 |

The control points sit at integer custom angles and at prime custom radians: C₁₈₂ at 7 radians, C₆₅₀ at 11 radians. The conversion factor 364/360 maps 90° standard to exactly 91° custom and 990/7° standard to exactly 143° custom. ∎

### 5.3 Control Points from Mid-Angle Mapping

**Red arc (182° custom):** Θ = 180° standard → mid-angle 90° →

> C₁₈₂ = (0, 1) at 91° = 7 × 13 = 7 custom radians

**Green arc (650° custom, eff. 286°):** Half-angle = 180° − 3π/14 rad →

> C₆₅₀ = (−cos 3π/14, sin 3π/14) at 143° = 11 × 13 = 11 custom radians

C₆₅₀ divides the circle into 28 equal arcs since 143° = 11 × (364/28) and 2π = 28 in the custom system. It marks the 11th radian — a natural rational division of the circumference. ∎

### 5.4 Reflection Property

Reflecting C₁₈₂ across the chord SP yields C₆₅₀. The mid-angles of the two arcs are symmetric about the perpendicular bisector of chord SP. The control points, placed on the unit circle at these mid-angles, are reflections of each other across line SP. ∎

### 5.5 Parametric Equations (Exact)

Let P_x = −φ/2, P_y = −√(10−2√5)/4.

**Red arc:**

> x(t) = (1−t)² + P_x·t²
> y(t) = 2(1−t)t + P_y·t²

**Green arc**, with C_x = −cos(3π/14), C_y = sin(3π/14):

> x(t) = (1−t)² + 2(1−t)t·C_x + P_x·t²
> y(t) = 2(1−t)t·C_y + P_y·t²

Both curves are parabolic arcs with all three defining points (S, C, P) on the unit circle. They are exact geometric entities — not approximations — that together with the circle create the construction for the squared circle. ∎

---

## 6. Golden Ratio Convergence

The vertex at 218.4° custom maps to 216° standard:

> cos(216°) = −cos(36°) = −(1+√5)/4 = −φ/2
> sin(216°) = −sin(36°) = −√(10−2√5)/4

The meeting point of the two arcs is exactly the golden-ratio-based vertex of the pentagon. The golden ratio appears because the pentagon's geometry is linked to the √5 that emerges from the vesica piscis when combined with the factor √13. ∎

---

## 7. Chord Relationships: √2, √3, and the Pythagorean Triple

- **Square (n=4):** side = √2 (chord subtending 90° standard)
- **Hexagon (n=6):** side = 1 (chord subtending 60° standard)
- **Hexagon short diagonal:** √3 (chord subtending 120° standard)

> **1² + (√2)² = (√3)²**

### 7.1 The Vesica Piscis in the 364° System

The vesica piscis ratio √3 : 1 is encoded in the hexagon's central angle 60.67° = 182°/3. The hexagonal structure is where √3 enters the construction. For the r = √13 formulation:

- Vesica height = √3 · √13 = √39
- Vesica width = √13
- Squared circle side = √14 · √13 = √182

The two Bézier curves embed these ratios as exact geometric constructs: C₁₈₂ uses the vertical point (91°, the vesica's axis), while C₆₅₀ uses the 143° point that splits the circle into 28 equal arcs — connecting S and P along coprime-derived paths whose control points sit at 7 and 11 custom radians respectively. ∎

---

## 8. Dodecagon, Nonagon, and Rational Ratios

### 8.1 The Dodecagon Step: π/6

30.33° = 364°/12 = 91°/3. In custom radians: 7/3 = π/6. This divides the red arc's half-turn into exactly six equal parts: 6 × 30.33° = 182°.

### 8.2 The Nonagon in the Convergence Zone

202.22° = 5 × (364°/9) = 10π/9 custom radians. It sits exactly π/9 past the half-turn: 202.22° − 182° = 20.22° = 182°/9.

### 8.3 The 20/3 Ratio

> 202.22° / 30.33° = 20/3 (exact) ∎

---

## 9. Three-Sequence Convergence

### 9.1 Repunits

The quadratic coefficients are repunits. The roots' sum is R₄ = 40 and their product is R₆ = 364.

### 9.2 Tribonacci Numbers

| T_n | Value | Expression |
|-----|-------|------------|
| T₃ | 1 | Additive unit in π = T₇ + T₃ |
| T₅ | 4 | T₅² = 16; cofactor in 208 = T₅² × T₇ |
| T₆ | 7 | Bridge: Tri(7) = 28 = 2π |
| T₇ | **13** | Cosmic radius = R₃ = 1 radian = 111₃ |

### 9.3 Triangular Numbers

| Tri(n) | Value | Where it appears |
|--------|-------|-----------------|
| Tri(3) | 6 | Hexagon sides |
| Tri(7) | **28 = 2π** | Full circle in radians |
| Tri(10) | 55 | Radius = Tri(10) + ½ |
| Tri(13) | 91 | Quarter-turn |

The chain: T₆ (Tribonacci) = 7 → Tri(7) (Triangular) = 28 = 2π (Circle). ∎

---

## 10. Torus Knots from the Coprime Structure

### 10.1 The Reduced Arc Ratio

The two arc spans (182° and 286° effective) factorize as:

> 182 = 2 × 7 × 13
> 286 = 2 × 11 × 13

Dividing out the common factor 26:

> **182 : 286 = 7 : 11**

gcd(7, 11) = 1. The reduced arc ratio is coprime.

### 10.2 The Three-Rotation System

Adding the 90° flip (91° = 7 × 13) produces three step sizes:

- 7 (from the red arc)
- 11 (from the green arc)
- 13 (from the radian unit / 90° flip)

All three are pairwise coprime:

> gcd(7, 11) = 1, gcd(7, 13) = 1, gcd(11, 13) = 1

The combined cycle length before full repeat: lcm(7, 11, 13) = **1,001** = 7 × 11 × 13.

By the equidistribution theorem, a diagonal walk on a 7 × 11 × 13 torus visits every one of 1,001 positions exactly once before returning to start. 100% coverage, zero collisions, zero gaps. This is a Hamiltonian cycle guaranteed by pairwise coprimality. ∎

### 10.3 The Full Coprime Landscape

The tetradecagon (n = 14) and pentadecagon (n = 15) extend the coprime structure. Note: gcd(7, 14) = 7, so any combination containing both 7 and 14 fails pairwise coprimality. All other pairs from {7, 11, 13, 14, 15} are coprime.

**9 coprime pairs:**

| Pair | LCM | Source |
|------|-----|--------|
| (7, 11) | 77 | Red/green arc reduced ratio |
| (7, 13) | 91 | C₁₈₂ angle = 7 × 13 |
| (7, 15) | 105 | Heptagon × pentadecagon |
| (11, 13) | 143 | C₆₅₀ angle = 11 × 13 |
| (11, 14) | 154 | Hendecagon × π-gon |
| (11, 15) | 165 | Hendecagon × pentadecagon |
| (13, 14) | 182 | Radian × π = half-turn |
| (13, 15) | 195 | Radian × pentadecagon |
| (14, 15) | 210 | π-gon × pentadecagon |

**7 coprime triples:**

| Triple | LCM | Notes |
|--------|-----|-------|
| (7, 11, 13) | **1,001** | Primary walk — Brieskorn sphere Σ(7, 11, 13) |
| (7, 11, 15) | 1,155 | Heptagon walk with pentadecagon |
| (7, 13, 15) | 1,365 | Radian walk with pentadecagon |
| (11, 13, 14) | 2,002 | Alternate walk with π-gon |
| (11, 13, 15) | 2,145 | Hendecagon-radian-pentadecagon |
| (11, 14, 15) | 2,310 | Hendecagon-π-pentadecagon |
| (13, 14, 15) | 2,730 | Radian-π-pentadecagon |

**2 coprime quadruples:**

| Quadruple | LCM | Factorization |
|-----------|-----|---------------|
| **(7, 11, 13, 15)** | **15,015** | 3 × 5 × 7 × 11 × 13 = **15 × 1,001** |
| **(11, 13, 14, 15)** | **30,030** | 2 × 3 × 5 × 7 × 11 × 13 = **2 × 15,015** |

### 10.4 The Odd-Prime Quadruple

The coprime quadruple (7, 11, 13, 15) gives lcm = **15,015** = 3 × 5 × 7 × 11 × 13 — the product of all odd primes from 3 to 13. Five consecutive odd primes. The pentadecagon (15 = 3 × 5) multiplies the primary 1,001-step walk by itself: 15,015 = 15 × 1,001.

This is a 4-torus walk with 15,015 conflict-free positions in 2D from pure integer arithmetic. At 6 z-trits (729 = 3⁶ = Δ₂):

> 15,015 × 729 = **10,945,935** positions in 3D+

The second quadruple (11, 13, 14, 15) doubles this:

> 30,030 × 729 = **21,891,870** positions in 3D+

Nearly 22 million conflict-free addresses derived entirely from one axiom. ∎

### 10.5 Coprime Expansion — Compression vs. Depth

The pentadecagon (15 = 3 × 5) is a **compression**: it packs the triangle (3) and pentagon (5) into a single polygon step. The quadruple (7, 11, 13, 15) and the quintuple (3, 5, 7, 11, 13) generate the same walk length — lcm = 15,015 either way — because 3 × 5 × 7 × 11 × 13 = 15 × 7 × 11 × 13 = 15,015.

But 15 shares factors with every polygon divisible by 3 or 5: gcd(3, 15) = 3, gcd(5, 15) = 5, gcd(6, 15) = 3, gcd(9, 15) = 3, gcd(10, 15) = 5, gcd(12, 15) = 3. If 15 is in the walk, none of these can join. If 15 is decomposed back into 3 and 5, the gate opens.

#### 10.5.1 Quintuples

| Quintuple | LCM | Notes |
|-----------|-----|-------|
| (3, 5, 7, 11, 13) | 15,015 | Same as (7,11,13,15) — decompressed |
| (3, 5, 11, 13, 14) | 30,030 | Same as (11,13,14,15) — decompressed |
| (3, 4, 5, 7, 11) | 4,620 | Adds square |
| (3, 4, 5, 7, 13) | 5,460 | Adds square |
| (3, 4, 5, 11, 13) | 8,580 | Adds square |
| (3, 4, 7, 11, 13) | 12,012 | 12 × 1,001 |
| (3, 5, 7, 8, 11) | 9,240 | Adds octagon |
| (3, 5, 7, 8, 13) | 10,920 | Adds octagon |
| (3, 5, 7, 11, 13) | 15,015 | Odd primes 3–13 |
| (3, 5, 8, 11, 13) | 17,160 | Octagon path |
| (3, 7, 8, 11, 13) | 24,024 | 24 × 1,001 |
| (4, 5, 7, 9, 11) | 13,860 | No 13 — square-nonagon path |
| (4, 5, 7, 9, 13) | 16,380 | Square-nonagon-radian |
| (4, 5, 7, 11, 13) | 20,020 | 20 × 1,001 |
| (4, 5, 9, 11, 13) | 25,740 | Nonagon path |
| (4, 7, 9, 11, 13) | 36,036 | 36 × 1,001 |
| (4, 7, 11, 13, 15) | 60,060 | 15 with square (coprime to 15) |
| (5, 7, 8, 9, 11) | 27,720 | Pentagon-octagon-nonagon |
| (5, 7, 8, 9, 13) | 32,760 | Pentagon-octagon-nonagon-radian |
| (5, 7, 8, 11, 13) | 40,040 | 40 × 1,001 |
| (5, 7, 9, 11, 13) | 45,045 | 45 × 1,001 |
| (5, 9, 11, 13, 14) | 90,090 | 90 × 1,001 |
| (7, 8, 9, 11, 13) | 72,072 | 72 × 1,001 |
| (7, 8, 11, 13, 15) | 120,120 | 15 with octagon |

28 valid quintuples from the 13 polygons.

#### 10.5.2 Sextuples — The Maximum Coprime Group

The largest pairwise coprime groups from the 13 polygon set are **size 6**. No valid group of 7 or more exists.

| Sextuple | LCM | Structure |
|----------|-----|-----------|
| (3, 4, 5, 7, 11, 13) | **60,060** | 4 × 15,015 |
| (3, 5, 7, 8, 11, 13) | **120,120** | 8 × 15,015 |
| (4, 5, 7, 9, 11, 13) | **180,180** | 12 × 15,015 = 180 × 1,001 |
| **(5, 7, 8, 9, 11, 13)** | **360,360** | **24 × 15,015 = 360 × 1,001** |

The maximum: **(5, 7, 8, 9, 11, 13)** with lcm = **360,360**.

This is 360 × 1,001 — the standard circle (360) times the primary coprime walk (1,001). The six polygons are pentagon, heptagon, octagon, nonagon, hendecagon, tridecagon. All pairwise coprime. A 6-torus walk visiting 360,360 positions exactly once.

At 6 z-trits: 360,360 × 729 = **262,702,440** conflict-free positions. Over a quarter billion from pure integer arithmetic.

#### 10.5.3 The Compression–Expansion Duality

The pentadecagon presents a choice:

**Compression path (15 in the walk):** Fewer polygons, tighter encoding. Maximum: (11, 13, 14, 15) → 30,030 positions. The pentadecagon absorbs the triangle and pentagon, blocking access to 3, 5, 6, 9, 10, 12.

**Expansion path (15 decomposed into 3 and 5):** More polygons, deeper address space. Maximum: (5, 7, 8, 9, 11, 13) → 360,360 positions. Twelve times larger than the compressed maximum.

Both paths start from the same axiom. The pentadecagon is the gate — it either compresses the walk or gates the expansion. The choice is architectural: compression for simplicity, expansion for capacity. ∎

### 10.6 Torus Knot Families

The coprime pairs produce prime torus knots:

| Knot (p, q) | Harmonic source | c(p,q) | g(p,q) |
|-------------|-----------------|--------|--------|
| (7, 11) | Red/green arc reduced ratio | 66 | 30 |
| (7, 13) | Red arc × radian unit (91° = 7 × 13) | 78 | 36 |
| (7, 15) | Heptagon × pentadecagon | 90 | 42 |
| (11, 13) | Green arc × radian unit (143° = 11 × 13) | 130 | 60 |
| (11, 14) | Hendecagon × π-gon | 140 | 65 |
| (11, 15) | Hendecagon × pentadecagon | 150 | 70 |
| (13, 14) | Radian × π = 182 (half-circle) | 168 | 78 |
| (13, 15) | Radian × pentadecagon | 180 | 84 |
| (14, 15) | π-gon × pentadecagon | 195 | **91** |

The crossing number is the standard minimum crossing number c(p, q) = (p−1)q for torus knot T(p, q) with p < q. The Seifert genus g = (p−1)(q−1)/2 is the genus of the minimal Seifert surface in S³. Note: the edge-pair intersection count kc = p(q−1) = c + (q−p) is a distinct quantity (see TM-2026-035 §12).

All pairs are coprime, guaranteeing single, non-self-intersecting knots.

Note: gcd(7, 14) = 7 — the (7, 14) pair is NOT coprime and does not form a prime knot. This is the only excluded pair from the key polygon set {7, 11, 13, 14, 15}. The exclusion is structural: 14 = 2 × 7.

### 10.7 3D Torus Knot Equations (R = 2, r = 1)

> x(t) = (2 + cos(11t)) · cos(7t)
> y(t) = (2 + cos(11t)) · sin(7t)
> z(t) = sin(11t)

for t ∈ [0, 2π].

### 10.8 4D Clifford Torus Knot

> x(t) = cos(7t) · cos(11t)
> y(t) = cos(7t) · sin(11t)
> z(t) = sin(7t) · cos(13t)
> w(t) = sin(7t) · sin(13t)

This lies on the unit Clifford torus S¹ × S¹ in ℝ⁴. The triple winding (7, 11, 13) encodes all the harmonic factors of the 364° system simultaneously. ∎

### 10.9 Seifert Genus and the Arc Root Closure

The Seifert genus of a torus knot T(p, q) with p < q is:

> g(p, q) = (p − 1)(q − 1) / 2

This is the genus of the minimal orientable surface bounded by the knot in S³. Three results close the derivation chain back onto the unified equation:

**T(14, 15): g = 13 × 14 / 2 = 182/2 = 91.** The Seifert genus of the π-gon × pentadecagon torus knot is the quarter-turn — the arc root divided by 2, the ionization threshold wavelength, the triangular number Tri(13). The unified equation's arithmetic reproduces itself as 4D topology: the genus of the knot whose winding numbers are π and 15 is the quarter of the circle whose circumference they define.

**T(13, 14): lcm = 182 = arc root.** The radian × π torus knot has walk length equal to the semicircle — the meaningful root of the unified equation. Its Seifert genus g = 78 = 6 × 13 = 6 × (1 custom radian).

**T(7, 11): twist angle = 286° in the 364° system.** A 7-gonal antiprism twisted by 11 half-steps produces twist angle 11 × 182/7 = 11 × 26° = **286°** custom — exactly the UV-B boundary wavelength (§18). The spectral correspondence connects to the torus knot twist, not merely to wavelength position. The green arc's effective span (650 mod 364 = 286) is simultaneously the twist angle of the primary torus knot.

The T(14, 15) result is the strongest closure in the framework: the topology of the Seifert surface reproduces the arithmetic of the arc equation. ∎

---

## 11. The Superhub Zones

### 11.1 Node Census

Inscribing all 13 regular polygons (n = 3 through 15) produces:

| Category | Count | Description |
|----------|-------|-------------|
| Rim vertices | 58 | On the unit circle |
| Interior intersections | 446 | Where edges of different polygons cross |
| **Total nodes** | **504** | Complete node set |

### 11.2 The Four Superhub Zones

Out of 504 nodes, exactly four zones have 4 polygon edges simultaneously crossing — the maximum connectivity in the construction:

| Zone | Coordinates (x, y) | Custom angle | Polygons crossing | Distance from center |
|------|--------------------|--------------|--------------------|---------------------|
| A | (−0.9010, +0.3728) | 159.3° | 7 × 11 × 12 × 13 | 0.9743 |
| B | (−0.9010, −0.3728) | 204.7° | 7 × 11 × 12 × 13 | 0.9743 |
| C | (+0.7396, +0.6288) | 40.8° | 8 × 11 × 12 × 13 | 0.9699 |
| D | (+0.7396, −0.6288) | 323.2° | 8 × 11 × 12 × 13 | 0.9699 |

### 11.3 The Pattern

Three polygons appear in ALL four zones: **11, 12, 13** — the three highest polygons. The 4th member switches between **7** (heptagon) and **8** (octagon).

- Zones A and B are mirrors across the x-axis (same x, opposite y)
- Zones C and D are mirrors across the x-axis (same x, opposite y)
- This splitting into two mirror pairs reflects the two distinct root branches of the governing quadratic

### 11.4 The 12 = Discriminant Connection

The dodecagon (12) appears in all four superhub zones. The number 12 is:

- √Δ of the circle quadratic x² − 40x + 364 = 0
- The spread between the two roots: 26 − 14 = 12
- The q-parameter of canonical Plenum Square A

It is not a coprime step size but the **discriminant** — a structural constant that ties the superhub geometry to the same modular arithmetic that generates π.

### 11.5 Proximity to Key Points

| Zone | Distance to S (1,0) | Distance to P (−φ/2, ...) |
|------|---------------------|---------------------------|
| A | 1.937 | 0.964 |
| B | 1.937 | **0.234** (closest) |
| C | **0.679** (closest) | 1.969 |
| D | **0.679** (closest) | 1.550 |

Zone B is nearest to P (the golden-ratio convergence where both arcs meet) — distance 0.234. Zones C and D flank S (the arc entry) — distance 0.679. The four superhubs guard the two endpoints of the arc highways. ∎

---

## 12. Torus Projection and Key Point Mapping

### 12.1 Unit Circle to Torus

For a torus with major radius R and minor radius r, a point at unit-circle angle θ with distance d from center maps to:

> φ = arccos(d) (tube elevation)
> X = (R + r·cos(φ)) · cos(θ)
> Y = (R + r·cos(φ)) · sin(θ)
> Z = r·sin(φ)

Points on the rim (d = 1) sit on the torus equator (φ = 0, Z = 0). Interior points lift off the equator.

### 12.2 Key Points on the Torus (R = 2, r = 1)

| Point | Unit circle | θ (rad) | φ (rad) | Torus (X, Y, Z) |
|-------|------------|---------|---------|-----------------|
| S (0°) | (1.0, 0.0) | 0.0 | 0.0 | (3.0, 0.0, 0.0) |
| P (218.4°) | (−0.809, −0.588) | −2.513 | 0.0 | (−2.427, −1.763, 0.0) |
| C₁₈₂ (91°) | (0.0, 1.0) | 1.571 | 0.0 | (0.0, 3.0, 0.0) |
| C₆₅₀ (143°) | (−0.782, 0.623) | 2.468 | 0.0 | (−2.345, 1.871, 0.0) |
| Zone A | (−0.901, +0.373) | 2.749 | 0.224 | (−2.749, 1.138, 0.222) |
| Zone B | (−0.901, −0.373) | −2.749 | 0.224 | (−2.749, −1.138, 0.222) |
| Zone C | (+0.740, +0.629) | 0.705 | 0.243 | (2.263, 1.924, 0.240) |
| Zone D | (+0.740, −0.629) | −0.705 | 0.243 | (2.263, −1.924, 0.240) |

S, P, C₁₈₂, and C₆₅₀ sit on the torus equator (Z = 0). The four superhubs lift off the equator to Z ≈ 0.22–0.24.

### 12.3 Bézier Arcs on the Torus

Both arcs start at S on the equator (Z = 0), climb through the torus interior, and descend back to P on the equator:

| Arc | Peak Z | At t = | Character |
|-----|--------|--------|-----------|
| Red (182°) | 0.955 | 0.62 | Shorter, later peak |
| Green (286°) | 0.977 | 0.34 | Longer, earlier peak |

The arcs trace ridgelines on the torus surface — rising from the equator, reaching nearly the top of the tube, and returning. ∎

---

## 13. The Brieskorn Sphere Σ(7, 11, 13)

### 13.1 Identification

The network — comprising 13 inscribed polygons (n = 3–15), their rim vertices, edge-edge intersections, and superhub zones arranged on a torus with the parametric equations of §10.7 — is a combinatorial cell decomposition of the Brieskorn sphere Σ(7, 11, 13), the link of the singularity x⁷ + y¹¹ + z¹³ = 0 in ℂ³. (Node census for the original 11-polygon system: 58 rim + 446 interior = 504; the addition of the tetradecagon and pentadecagon increases both counts.)

### 13.2 Evidence

This identification follows from:

1. The explicit 4D Clifford parametrization (cos 7t · cos 11t, cos 7t · sin 11t, sin 7t · cos 13t, sin 7t · sin 13t) embeds Σ(7, 11, 13) as a closed curve in S³.

2. The (7, 11) and (7, 13) torus knots, realized in the 3D projection and in the node graph, correspond to the coordinate axis links within the Brieskorn sphere.

3. The 364° circle, with π = 14 and angles 91° = 7 × 13 and 143° = 11 × 13, encodes the Seifert invariants 7, 11, 13 and the linking numbers between the three torus knots.

4. The network's vertices on the equator (Z = 0) represent the base orbifold of the Seifert fibration; the intersections lift off the equator to form the crossing structure of the three torus knots, producing a triangulation of the 3-manifold.

### 13.3 Consequence

The node data and the geometric construction together realize Σ(7, 11, 13) as a computable, harmonically tuned 3-manifold, unifying the Bézier arcs, the torus knots, and the 364° circle into a single topological object. ∎

---

## 14. The Hyperbolic Coxeter Group

The coprime triple (7, 11, 13) — derived in §10.1–10.2 from the arc ratio and the radian unit — satisfies the defining relations of a rank-3 Coxeter group. This section establishes the algebraic layer between the geometric construction (§§4–10) and the topological identification (§13).

### 14.1 Coxeter Groups — Definition

A Coxeter group of rank n is an abstract group with presentation

> ⟨ r₁, …, rₙ | rᵢ² = 1, (rᵢrⱼ)^mᵢⱼ = 1 ⟩

where mᵢᵢ = 1 and for i ≠ j, mᵢⱼ = mⱼᵢ ∈ {2, 3, 4, …} ∪ {∞}. The integers mᵢⱼ are the Coxeter exponents. The group is finite (spherical) if and only if its Coxeter diagram is a disjoint union of the classical types Aₙ, Bₙ, Dₙ, E₆, E₇, E₈, F₄, H₃, H₄, I₂(m); otherwise it is infinite (affine or hyperbolic).

For a rank-3 triangle group with exponents (p, q, r):

- Spherical (finite) if 1/p + 1/q + 1/r > 1
- Affine (Euclidean, infinite) if 1/p + 1/q + 1/r = 1
- Hyperbolic (infinite) if 1/p + 1/q + 1/r < 1

### 14.2 The Derived Coxeter Matrix

The three generators r₁, r₂, r₃ with Coxeter matrix

> M = | 1   7  11 |
>     | 7   1  13 |
>     | 11  13   1 |

give the presentation

> ⟨ r₁, r₂, r₃ | rᵢ² = 1, (r₁r₂)⁷ = 1, (r₁r₃)¹¹ = 1, (r₂r₃)¹³ = 1 ⟩

The exponents are:

- **7**: from the reduced red arc ratio 182/26 = 7 (§10.1), equivalently the C₁₈₂ control point at 7 custom radians (§5.2)
- **11**: from the reduced green arc ratio 286/26 = 11 (§10.1), equivalently the C₆₅₀ control point at 11 custom radians (§5.2)
- **13**: the radian unit R₃ = (3³ − 1)/2 = 13 (§2.1)

No external input is used. The triple (7, 11, 13) is forced by the repunit family and the Bézier arc geometry.

### 14.3 Hyperbolicity

Compute the reciprocal sum:

> 1/7 + 1/11 + 1/13 = (143 + 91 + 77) / 1001 = 311/1001 ≈ 0.3107 < 1

The group is **hyperbolic** — an infinite Coxeter group of rank 3. It acts on the hyperbolic plane by reflections in three geodesic lines meeting at angles π_std/7, π_std/11, and π_std/13. The quotient is a hyperbolic triangle with area

> A = π_std(1 − 1/7 − 1/11 − 1/13) = π_std · 690/1001

### 14.4 Geometric Realisation in the Custom System

In the standard geometric realisation, the dihedral angle between reflecting hyperplanes for rᵢ and rⱼ is π_std/mᵢⱼ. Using the custom radian (1 custom rad = 13°), the same physical angle becomes

> π_std/mᵢⱼ = 14/mᵢⱼ custom radians

For the (7, 11, 13) group: angles 2, 14/11, 14/13 custom radians. The custom unit rescales the geometric picture but does not alter the group — the abstract presentation is independent of angle units.

### 14.5 Connection to the Brieskorn Sphere

The hyperbolic triangle group (7, 11, 13) acts on H² with quotient orbifold a sphere with three cone points of orders 7, 11, 13. This orbifold is the base of the Seifert fibration of the Brieskorn sphere

> Σ(7, 11, 13) = { x⁷ + y¹¹ + z¹³ = 0 } ∩ S⁵ ⊂ ℂ³

identified in §13. The 4D Clifford parametrisation of §10.8 realises a (7, 11, 13) torus knot on Σ(7, 11, 13). The Coxeter group is the reflection group whose orbit structure generates the Seifert fibration — it is the algebraic engine of the topological object described in §13.

### 14.6 What Is Not Derived

The integers 14 (= π) and 15 (pentadecagon side count) appear elsewhere in the framework but do not naturally participate in a higher-rank Coxeter matrix without introducing an external selection rule (e.g., max, min, or product operations on the derived constants). Such rules are not derivable from the axiom and are therefore excluded from this first-principles account.

The dihedral groups I₂(n) for n = 3, …, 15 follow trivially from the inscribed regular polygons — each n-gon has the symmetry group I₂(n) — but these are rank-2 and do not constitute a novel higher-rank structure. The rank-3 hyperbolic group (7, 11, 13) is the unique higher-rank Coxeter group that emerges without arbitrary choices. ∎

---

## 15. The Disdyakis Bridge — Icosahedral Geometry

### 15.1 The Re-Priming

The structural content of the 180 → 182 shift is in the prime factorisation:

| | Standard | PlenumNET |
|---|---|---|
| Half-turn | 180 = 2² × 3² × 5 | 182 = 2 × 7 × 13 |
| Golden angle | 180(3−√5) | 182(3−√5) |
| Primes in half-turn | {2, 3, 5} | {2, 7, 13} |

The irrational factor (3−√5) is unchanged. Only the integer lattice through which it acts is replaced. The 180 → 182 shift **re-primes** the golden angle: the algebraic foundation swaps from the Platonic symmetry primes {2, 3, 5} to the coprime generators {2, 7, 13}. The coprime generator 15 = 3 × 5 bridges both families.

### 15.2 The Circumradius Identity

The truncated icosidodecahedron — the most face-diverse Archimedean solid (120 vertices, 62 faces: 12 decagons, 20 hexagons, 30 squares) — has circumradius squared:

> **R² = 14 + 5φ, where 14 = π in PlenumNET.**

Derived from vertex triple (1/φ, 1/φ, 3+φ): R² = 2(2−φ) + (10+7φ) = 14 + 5φ. The integer part in ℤ[φ] is π.

### 15.3 The Defect Structure

Face-centre distances and their defects from the circumsphere:

| Face type | d² in ℤ[φ] | R² − d² | Count |
|-----------|-----------|---------|-------|
| Decagon (5-fold) | 10 + 5φ | **4** (pure integer) | 12 |
| Hexagon (3-fold) | 6 + 9φ | 4/φ² | 20 |
| Square (2-fold) | 10 + 7φ | 2/φ² | 30 |

The defect ratio is **2φ² : 2 : 1** — a geometric series in 1/φ² with the decagonal face producing a pure-integer defect (φ terms annihilate exactly).

### 15.4 Coprime Generator Hierarchy

The disdyakis triacontahedron (dual of the truncated icosidodecahedron, 120 scalene triangle faces) has face angle cosines in ℚ(φ) with denominators equal to vertex orbit sizes (12, 20, 30), whose lcm = 60 = |I| (icosahedral rotation group order). The coprime generators surface through mechanisms of decreasing algebraic invariance:

| Level | Generator | Mechanism | Why this level |
|---|---|---|---|
| 1 (strongest) | **11** | ℤ[φ]-norm: N(2+5φ) = 11 | (5/11) = +1: splits — norm exists |
| 2 | **7** | ℤ[φ] coefficient: cos(α₂) = (7−4φ)/30 | (5/7) = −1: inert — norm impossible |
| 3 | **15** | ℚ(√5) coefficient: cos(α₃) = (15−2√5)/20 | Absent in preferred basis |
| 4 (weakest) | **13** | Global metric: 182 = 2 × 7 × 13 | (5/13) = −1: inert; absent from all coefficients |

The hierarchy is forced at its top two levels by quadratic reciprocity: (5/11) = +1 (splits, norm possible) vs (5/7) = (5/13) = −1 (inert, norm impossible). The generator 13 is excluded from intrinsic icosahedral data by a number-theoretic obstruction — the norm equation |a² + ab − b²| = 13 has no integer solutions.

### 15.5 Archimedean Circumradii — The Full Survey

Across all 11 non-chiral Archimedean solids, the integer parts of R² are:

> {2, 2, 5, 5, 5, **7**, **7**, 10, **11**, **13**, **14**}

Four PlenumNET framework constants appear, each in a different algebraic ring:

| Integer part | Solid | Ring | PlenumNET identity |
|---|---|---|---|
| **14** | Truncated icosidodecahedron | ℤ[φ] | π |
| **13** | Truncated cuboctahedron | ℤ[√2] | Radian unit |
| **11** | Truncated tetrahedron | ℤ | Coprime generator |
| **7** | Truncated dodecahedron, rhombicosidodecahedron | ℤ[φ] | Coprime generator |

The Archimedean solids, taken as a family, encode four of five framework constants distributed across the three symmetry families (icosahedral, octahedral, tetrahedral). See TM-2026-034 for the complete derivation and all proofs. ∎

---

## 16. The 3D Crystal Extension

The planar construction — 13 polygons inscribed in the 364° circle — elevates to three dimensions. The inscribed circle becomes a circumsphere, every polygon becomes an antiprism, and the coprime walk traces geodesics on the sphere surface. The full derivation with worked vertex tables is in TM-2026-035.

### 16.1 Circle to Circumsphere

For a vertex at 364° angle `deg`, lifted to latitude ±α on the unit sphere:

> θ = deg × 2π / 364
> X = sin(θ) · cos(α),  Y = sin(α),  Z = −cos(θ) · cos(α)

Every vertex lies on the unit sphere: |V|² = cos²(α)[sin²(θ) + cos²(θ)] + sin²(α) = 1. Viewed from directly above (down the Y-axis), the 3D construction projects to the 2D inscribed circle scaled by cos(α).

### 16.2 Antiprism Latitude Derivation

Each polygon n becomes an n-gonal antiprism — a prismatoid with 2n vertices in two parallel planes at latitudes ±α. The latitude is the half-step:

> α_custom(n) = 182/n custom degrees
> α_std(n) = (182/n) × (90/91) = **180/n** standard degrees

The simplification is exact: 182/91 = 2, so the factor cancels to 2 × 90/n = 180/n. All 3D coordinate calculations use α_std = 180/n.

**Why 182?** The number 182 is a root of the unified equation arc² − 832·arc + 118,300 = 0. It is also π × radian = 14 × 13 = the squared circle area. The same number that squares the circle in 2D generates the antiprism latitudes in 3D.

| n | α_std = 180/n | Integer? | Role |
|---|--------------|----------|------|
| 3 | **60.000°** | yes | Triangle antiprism = octahedron |
| 4 | **45.000°** | yes | Square antiprism; bottom ring = cardinal points |
| 5 | **36.000°** | yes | Pentagon antiprism; cos(36°) = φ/2 |
| 6 | **30.000°** | yes | Hexagon antiprism |
| 7 | 25.714° | no | Coprime walk; T₃ mx = −0.9010 (superhub A) |
| 8 | **22.500°** | yes | Octagon antiprism |
| 9 | **20.000°** | yes | Nonagon antiprism |
| 10 | **18.000°** | yes | Decagon antiprism |
| 11 | 16.364° | no | Coprime walk; Brieskorn triple |
| 12 | **15.000°** | yes | Dodecagon antiprism |
| 13 | 13.846° | no | Radian polygon; Brieskorn triple |
| 14 | 12.857° | no | π-gon; α = 90/7 |
| 15 | **12.000°** | yes | Pentadecagon antiprism |

Integer latitudes occur at n = 3, 4, 5, 6, 8, 9, 10, 12, 15 — precisely where 180/n reduces to an integer. The coprime walk polygons (n = 7, 11, 13, 14) all produce non-integer latitudes. This is the latitude-domain signature of the geometric/arithmetic polygon split (§16.6).

### 16.3 The Antiprism Construction

For each n-gon, the 3D solid is:

- **Top face:** n-gon at latitude +α, vertices at angles off + k × step
- **Bottom face:** n-gon at latitude −α, rotated by half-step 182/n degrees
- **Belt:** 2n triangles connecting the two rings
- **Total:** 2n + 2 faces, 4n edges, 2n vertices

The half-step rotation is derived, not chosen — 182 is a root of the arc equation. The rotation ensures no top edge aligns with any bottom edge: only triangles (not rectangles) can close the belt.

General formulas: V = 2n, E = 4n, F = 2n + 2, χ = V − E + F = 2. ∎

### 16.4 2D Landmarks as 3D Vertices

The 2D construction's structural nodes are literally vertices of the 3D antiprisms:

**Square antiprism (n = 4):** The bottom ring at offset 91° lands on the four cardinal points: B₀ = 91° (C₁₈₂ control point), B₁ = 182° (half-turn), B₂ = 273° (nadir), B₃ = 0° (S starting point). The Bézier arc anchor C₁₈₂ = (0, 1) is a vertex of the square antiprism.

**Pentagon antiprism (n = 5):** T₃ at 218.4° = P, the golden ratio convergence point where both Bézier arcs meet. cos(36°) = φ/2 — the golden ratio's half IS the pentagon antiprism's latitude cosine. The belt triangles have sides in golden proportion.

**Heptagon antiprism (n = 7):** T₃ at 156° has mx = −0.9010, matching superhub A's x-coordinate exactly. B₃ at 182° = half-turn. B₅ at 286° = UV-B boundary wavelength. The 286° vertex connects to the T(7,11) torus knot twist: 11 × 26° = 286° (§10.9).

**Triangle antiprism (n = 3):** The 6-vertex, 8-face prismatoid with vertex configuration 3.3.3.3 is the regular **octahedron**. T₀ at 0° sits above S; B₁ at 182° sits below the half-turn.

The 2D arc geometry is embedded in the 3D crystal. ∎

### 16.5 The Circumsphere

For the unit circumsphere (R = 1) in the 364° framework:

> Circumference = 2π = 28, Surface area = 4π = 56, Volume = 4π/3 = 56/3

The bridge factor between frameworks is 14/π_std ≈ 49/11 to first order — connecting the bridge factor to the hendecagon (n = 11), the median coprime polygon. The number 182 is simultaneously the arc root, the squared circle area (14 × 13), and the generator of every antiprism half-step (182/n). The same number that squares the circle in 2D generates the antiprism latitudes in 3D.

The circumsphere is the crystal-air interface — the only surface where Snell's law applies for entry/exit. Internal faces produce lattice deflection, not air-crystal refraction. ∎

### 16.6 The Dual Role of the Polygon Set

The 13 polygons serve two distinct functions:

**Geometric polygons (n = 3, 4, 5, 6, 8, 10):** These build the Archimedean solids — every Archimedean solid uses only polygons from this set. They also produce exact integer latitudes. Their role is classical solid geometry.

**Arithmetic polygons (n = 7, 9, 11, 13, 14, 15):** These define the coprime walk generators, the torus knot parameters, and the Brieskorn sphere. The primes 7, 11, 13 define Σ(7,11,13); n = 14 = π; n = 15 enables the coprime quadruple. They produce non-integer latitudes (except n = 9, 15). Their role is number-theoretic topology.

Both families inhabit the same circumsphere. The Archimedean solids are the skeleton; the coprime walk is the nervous system. This is the dual nature of the crystal.

### 16.7 Named Solids on the Circumsphere

Six named solids are inscribed in the same circumsphere:

| Solid | Source | V | F | Key property |
|-------|--------|---|---|-------------|
| Salvi Tetrahedron | n = 3, off = 0° | 4 | 4 | Base lat −19.471°, apex at pole |
| Plenum Cube | n = 4, off = 45.5° | 8 | 6 | Step = 91° = 7 × 13 |
| Radian Octahedron | n = 4 equator + poles | 6 | 8 | = triangle antiprism; edge = R√2 |
| Golden Icosahedron | n = 5, off = 0° | 12 | 20 | Vertex at P (218.4°); rings at ±26.565° |
| Golden Dodecahedron | n = 5 dual | 20 | 12 | R² = 3; vertices in ℤ[φ]; azimuthal alignment with n = 5 |
| Brieskorn Σ(7,11,13) | Coprime triple | 33 | 62 | Rings at +25.714°, +16.364°, −13.846° |

### 16.8 Shape Inventory

| Category | Count |
|----------|-------|
| Circumsphere | 1 |
| Antiprisms | 13 (n = 3–15) |
| Platonic solids | 5 |
| Named solids | 6 (including Brieskorn) |
| Archimedean solids | 13 (complete basis from polygon set) |
| Catalan solids | 13 (duals, face-transitive) |
| Disdyakis triacontahedron | (1, counted in Catalans — geometric witness: R² = 14 + 5φ, §15) |
| **Total solids** | **46** (plus 13 prisms = 59) |

Total antiprism vertices on the circumsphere: 2 × (3 + 4 + … + 15) = 2 × 117 = **234** points.

### 16.9 Coprime Walks on the Circumsphere

The coprime pairs trace torus knots connecting latitude bands. The tightest pair is (13, 14): latitude span = 0.99° (sub-1°), lcm = 182 = arc root. The radian times π equals the arc root — the squared circle expressed as a coprime walk packed into a sub-1° latitude band.

The Brieskorn triple (7, 11, 13) visits three latitude bands at +25.714°, +16.364°, +13.846° with lcm = 1,001 steps. The path lives on T³, not T² — its topology is described by the orbit structure of Z₇ × Z₁₁ × Z₁₃ acting on the sphere.

Higher coprime groups span more bands simultaneously: the maximum sextuple (5, 7, 8, 9, 11, 13) with lcm = 360,360 visits 6 bands in the equatorial cluster, spanning 25.714° − 12.000° = 13.714° of colatitude. See §10 for the full coprime landscape.

### 16.10 Spherical Tiling and the UV-Colatitude Grid

Polygon edges projected onto the sphere as great circle arcs create a spherical tiling. The UV spectral bands (§18) define longitude partitions:

> EUV: 0°–91° (25.0%), UV-C: 91°–182° (25.0%), UV-B: 182°–286° (28.6%), UV-A: 286°–364° (21.4%)

Combined with 13 colatitude bands from the antiprism latitudes:

> 4 UV bands × 13 colatitude bands → ~52 spherical zones

The density is non-uniform: 3 polygons span 30° in the temperate zone vs 9 polygons packed into 18° in the equatorial zone (colatitudes 60°–78°). The crystal is densest near the equator, matching geodesic dome design.

### 16.11 The Dimensional Ladder

The coprime walk exists at each dimensional level:

**2D:** Torus knot T(p,q) projected flat on the inscribed circle — the familiar 364° construction. Crossings visible as edge intersections.

**3D:** Walk path on the circumsphere surface, connecting latitude bands at ±α(p) and ±α(q) via great circle arcs. The crystal is this intermediate — where 4D topology becomes partially visible.

**4D:** Clifford torus embedding. For triple (p,q,r): x = cos(pt)cos(qt), y = cos(pt)sin(qt), z = sin(pt)cos(rt), w = sin(pt)sin(rt). The Seifert surface of genus g = (p−1)(q−1)/2 spans the knot in S³. Stereographic projection S³ → ℝ³ → ℝ² recovers the lower-dimensional views.

Each projection loses information: 4D has no crossings (full topological structure), 3D introduces knot crossings, 2D compresses everything to the plane. ∎

---

## 17. Unification — All Elements from One Axiom

Every quantity in this system is derived from π = 14 (radian = 13), which is the smaller root of x² − 40x + 364 = 0. The system is not a rescaling of standard mathematics — it is an axiomatic derivation that generates standard π as a conversion constant (§1.1).

- The full circle (364°), half-turn (182°), quarter-turn (91°).
- The squared circle: unit circle area = π = 14, side = √14; r = √13 circle area = 182, side = √182 = √(14 × 13). Both algebraic.
- The r = √13 formulation yielding the vesica piscis ratios and √14 · √13 = √182.
- 13 regular polygons (n = 3–15) inscribed in the 364° circle; 13 polygons = radian.
- The tetradecagon (n = 14 = π): step = 26° = x₂ of the circle quadratic.
- The pentadecagon (n = 15 = 3 × 5): bridges triangle and pentagon families.
- The Bézier control points at 7 and 11 custom radians via mid-angle mapping.
- The 13-multiple pattern: 91 = 7·13, 143 = 11·13, 182 = 14·13, 364 = 28·13.
- The reflection property between C₁₈₂ and C₆₅₀.
- The golden ratio φ emerging from the pentagon at 218.4°.
- The Pythagorean triple 1, √2, √3 from the square and hexagon chord lengths.
- 9 coprime pairs, 7 coprime triples, 2 coprime quadruples from {7, 11, 13, 14, 15}.
- The (7, 11, 13) coprime walk with 1,001-step Hamiltonian cycle.
- The (7, 11, 13, 15) coprime quadruple: lcm = 15,015 = 3 × 5 × 7 × 11 × 13 — all odd primes 3–13.
- The (11, 13, 14, 15) coprime quadruple: lcm = 30,030 = 2 × 15,015.
- The compression–expansion duality: pentadecagon compresses (3, 5) into 15, or decomposes to unlock 28 quintuples and 4 sextuples.
- The maximum coprime sextuple: (5, 7, 8, 9, 11, 13) → lcm = **360,360** = 360 × 1,001 = 24 × 15,015.
- At 6 z-trits (Δ₂ = 729): 360,360 × 729 = **262,702,440** conflict-free positions — over a quarter billion.
- The torus knot families (9 prime knots) and the 4D Clifford embedding.
- The four superhub zones where polygons 7/8, 11, 12, 13 cross.
- The Brieskorn sphere Σ(7, 11, 13) as the topological structure of the node network.
- The rank-3 hyperbolic Coxeter group (7, 11, 13) as the reflection group whose quotient orbifold is the Brieskorn sphere's Seifert base (§14).
- The sponge width 729 = 3⁶ as the secondary discriminant.
- Integer angular arithmetic replacing transcendental π in every formula.
- The UV spectral partition: 91 (EUV), 182 (O₂ wall), 286 (ozone bridge), 364 (UV-A).
- The HModal signaling wave: β/α = √Δ = 12, γ = 1001/36, duty cycle d = 1/4, null at every 4th harmonic.
- The DC component ⟨H⟩ = 455/48 where 455 = 5 × 7 × 13 (pentadecagon factor emerging uninvited from the signal average).
- The Seifert genus closure: T(14,15) genus = (13 × 14)/2 = 91 = quarter-turn. The arc equation's topology reproduces the arc equation's arithmetic (§10.9).
- The (7,11) twist = 286°: a 7-gonal antiprism twisted by 11 half-steps produces twist angle 11 × 26° = 286° custom — the UV-B boundary wavelength. The spectral correspondence connects to the torus knot twist (§10.9).
- The Disdyakis Bridge: R² = 14 + 5φ for the truncated icosidodecahedron — π is the integer part. The re-priming 180 → 182 threads the golden ratio through {7, 13} instead of {3, 5}. Generator hierarchy forced by quadratic reciprocity: (5/11) = +1 splits, (5/7) = (5/13) = −1 inert (§15).
- The two roles of the polygon set: geometric polygons {3, 4, 5, 6, 8, 10} build Archimedean solids; arithmetic polygons {7, 9, 11, 13, 14, 15} define the coprime walk. Both inhabit the same circumsphere.
- The colatitude duality: colat(3) = lat(6) = 30°, colat(6) = lat(3) = 60°, square self-dual at 45° — limited to the three plane-tiling polygons.
- The antiprism latitude formula: α_std = 180/n, derived from the half-step 182/n via the bridge factor 90/91. The number 182 simultaneously = arc root = squared circle area = antiprism half-step generator.
- The dimensional ladder: 2D flat projection → 3D circumsphere → 4D Clifford torus, each step a projection that loses information. The crystal is the 3D intermediate where 4D topology becomes partially visible.
- The 3D crystal: 46 solids (13 antiprisms, 5 Platonic, 6 named, 13 Archimedean, 13 Catalan) on one circumsphere with 234 antiprism vertices (§16).
- 2D landmarks as 3D vertices: C₁₈₂ = square antiprism bottom vertex at 91°; P (golden ratio convergence) = pentagon antiprism T₃ at 218.4°; heptagon B₅ at 286° = UV-B boundary (§16.4).
- The (13, 14) coprime pair: lcm = 182 = arc root, latitude span 0.99° — 182 great circle arcs in a sub-1° band. The squared circle as a coprime walk (§16.9).
- The spherical tiling: 4 UV longitude bands × 13 colatitude bands → ~52 non-uniform spherical zones, densest at the equatorial cluster where 9 coprime walk polygons pack into 18° of colatitude (§16.10).

One equation generates every constant:

> **arc² − 832·arc + 118,300 = 0**

where 118,300 = R₆(R₆ − R₄ + 1) = 364 × 325, derived from pure repunit arithmetic with no assumed constants.

---

## 18. UV Spectral Correspondence

### 18.1 The Plenum Premise

Standard atomic physics derives wavelengths under a vacuum assumption: the electron exists in empty space, interacting only with the nucleus and the electromagnetic field. The Rydberg constant, the Lyman series, the Balmer series — all are vacuum quantities.

In the Plenum framework, there is no vacuum. Space is a medium. The electron exists in the plenum, not in emptiness. The "inversed weight of gravity" — buoyancy in the medium — modifies the effective relationships between mass, energy, and wavelength. Every measurement made under the vacuum assumption carries a systematic bias: the assumption of emptiness where there is fullness.

The axiom π = 14 generates the integers 91, 182, 286, 364 from pure algebra. These are not approximations of measured values. They are the exact values. The physical measurements confirm them to three significant figures while carrying the vacuum assumption as a systematic offset.

### 18.2 The Algebraic Derivation

The axiom produces four wavelengths through exact integer arithmetic:

| Value | Derivation | Factorization | Custom radians |
|-------|-----------|---------------|----------------|
| **91** | Quarter-turn = Tri(13) = radian × π / 2 | 7 × 13 | 7 |
| **182** | Half-turn = π × radian | 14 × 13 | 14 = π |
| **286** | Green arc effective = 650 mod 364 | 22 × 13 | 22 |
| **364** | Full circle = 2π × radian = R₆ | 28 × 13 | 28 = 2π |

These are derived from the unified equation (§2.3) and the repunit family. No physical constants, no measurements, no instruments. Pure algebra from the axiom.

### 18.3 The Exact Ratios

The four values relate by exact rational multiples of 91:

| Ratio | Value | Exact? | Source in axiom | Source in physics |
|-------|-------|--------|-----------------|-------------------|
| 182 / 91 | **2** | Yes | Half-turn / quarter-turn | Rydberg series: 1/1² vs 1/2² = 4:1, so Lyman × 2 |
| 286 / 91 | **22/7** | Yes | Green arc / quarter-turn | Integer arithmetic |
| 364 / 91 | **4** | Yes | Full circle / quarter-turn | Balmer limit / Lyman limit = 2²/1² |
| 286 / 182 | **11/7** | Yes | Coprime pair | Primary torus knot (7, 11) ratio |
| 364 / 286 | **14/11** | Yes | π / 11 | Full circle to ozone bridge |

The ratios 2 and 4 are exact in both the axiom (integer arithmetic) and quantum mechanics (the n² structure of the Rydberg formula). Two independent derivations — one algebraic, one physical — produce the same exact ratios. The 22/7 ratio is exact integer arithmetic; it is the Archimedean approximation to standard π, which in this system is not approximate but structural.

### 18.4 The UV Band Identification

The four integers, interpreted as nanometers, fall within the three empirically defined bands of ultraviolet radiation:

| Integer | UV Band | Range (nm) | Position in band |
|---------|---------|------------|-----------------|
| **91** | Vacuum UV / EUV edge | < 100–200 | Ionization threshold |
| **182** | UV-C | 100–280 | Deep UV-C, Schumann-Runge continuum |
| **286** | UV-B | 280–315 | Mid UV-B, ozone absorption region |
| **364** | UV-A | 315–400 | Near center of UV-A |

Each value sits well within its respective band, not at the boundary. These band assignments were defined empirically by photobiologists based on biological effects — skin penetration, DNA damage thresholds, ozone absorption coefficients. They were not designed to accommodate these integers.

### 18.5 Physical Confirmation

The Rydberg formula for hydrogen gives the series limits:

| Series | Formula | Measured wavelength | System integer | Vacuum bias |
|--------|---------|--------------------|--------------------|-------------|
| Lyman limit (n→∞ to n=1) | 1/R_H | 91.176 nm | **91** | +0.194% |
| — | 2/R_H | 182.353 nm | **182** | +0.194% |
| Balmer limit (n→∞ to n=2) | 4/R_H | 364.705 nm | **364** | +0.194% |

The bias is constant: +0.194% across all three measurements, decomposed as:

> 1.00194 = 1 + R∞ bias (+0.139%) + reduced-mass μ/mₑ (+0.055%)

The first component (+0.139%, UNIVERSAL_BIAS) is the difference between the infinite-mass Rydberg constant R∞ and the integer system. The second (+0.055%, VACUUM_BIAS) is the reduced-mass correction for finite proton mass. Both are systematic — the vacuum assumption shifts every wavelength by the same fractional amount, preserving all internal ratios exactly.

The oxygen ionization threshold independently confirms the anchor: atomic oxygen ionizes at 13.618 eV, corresponding to 91.06 nm — a second independent measurement converging on 91.

### 18.6 The Ozone Bridge: 22/7

The green arc span 286 nm does not correspond to a hydrogen series limit. Its derivation is different:

> 286 / 91 = **22/7**

Twenty-two sevenths — the oldest known rational approximation to standard π, attributed to Archimedes. In the axiom's integer system, this is not an approximation. It is the exact ratio between the ozone bridge wavelength and the ionization threshold. The radian unit (13) cancels: 286/91 = (22 × 13)/(7 × 13) = 22/7.

> **UV-B = Lyman threshold × π_Archimedes**

Standard π appears inside the custom system as the exact ratio between the ozone bridge and the hydrogen ionization anchor.

### 18.7 The Atmospheric Filter

The three UV band integers correspond to three distinct behaviors of oxygen in the atmosphere:

**91 nm — Ionization threshold.** The energy at which atomic hydrogen and oxygen ionize. Radiation at this energy is absorbed by individual atoms in the upper thermosphere. The quarter-turn: the first boundary.

**182 nm — O₂ molecular absorption wall.** The Schumann-Runge continuum spans 130–200 nm. Molecular oxygen (O₂) absorbs strongly in this range. Radiation at 182 nm does not reach the stratosphere. The half-turn: total containment. The energy that gets "squared" — absorbed, transformed, prevented from reaching the biosphere.

**286 nm — O₃ ozone bridge.** The Hartley band spans 200–310 nm with peak absorption near 255 nm. At 286 nm, ozone is still absorbing significantly but transmission is increasing. The Bézier bridge: a continuous modulation between two states. The ozone layer functions as the system's quadratic arc — a smooth, parabolic transition between two states.

**364 nm — Full transmission.** UV-A passes through the atmosphere almost unattenuated. It penetrates glass, reaches the dermis. The full circle: the complete cycle that transmits without obstruction.

### 18.8 The Carbon π-Bond and the Biological Threshold

The UV-B band's lower boundary at 280 nm was defined by biology, and the biology is governed by carbon:

The conjugated π-bonds in DNA bases (purines, pyrimidines) absorb maximally at 260 nm, with a tail extending into the UV-B range. Protein absorption — the tryptophan π→π* transition — peaks at 280 nm. The UV-B boundary was drawn where carbon π-bond damage begins in biological molecules.

The spectral region where carbon-based life is most vulnerable to photodamage is the region defined by the Archimedean π ratio applied to the ionization threshold. The π-bond — named for the constant that 22/7 approximates — is damaged by the wavelength derived from that same ratio.

### 18.9 First-Principle Derivation Summary

| Element | Physical mechanism | Wavelength | System integer | Ratio to 91 |
|---------|-------------------|------------|----------------|-------------|
| H and O (ionization) | Atomic energy scale | 91 nm | 91 = 7 × 13 | 1 (quarter-turn) |
| O₂ (Schumann-Runge) | Molecular absorption | 182 nm | 182 = 14 × 13 | 2 (half-turn) |
| O₃ (Hartley band) | Ozone bridge | 286 nm | 286 = 22 × 13 | 22/7 (π_Archimedes) |
| C (π-bond damage) | Biological threshold | ~280 nm | UV-B lower edge | — |
| Transparent atmosphere | No absorption | 364 nm | 364 = 28 × 13 | 4 (full circle) |

The derivation chain: the axiom π = 14 generates the unified equation, whose roots are 182 and 650 (eff. 286). The quarter-turn 91 = Tri(13) = 7 × 13 is the ionization threshold. From that anchor, the ratios ×2, ×22/7, and ×4 give the O₂ wall, the ozone bridge, and the full circle. The ratios are exact in the system and confirmed independently by the n² structure of the Rydberg formula. The constant +0.194% offset between the measured wavelengths and the system integers (decomposed as +0.139% R∞ bias and +0.055% reduced-mass correction) is the vacuum assumption — the systematic bias of measuring in emptiness what exists in fullness.

One axiom, one anchor, three elements (H, O, C), three UV bands. ∎

### 18.10 The Coprime Triple as Spectral Architecture

The coprime triple (7, 11, 13) is not merely a topological invariant of the Brieskorn sphere Σ(7, 11, 13). It encodes the spectral partition of the ultraviolet:

- **7** is the denominator of the Archimedean π (22/7) and the factor linking the ionization threshold to the radian unit (91 = 7 × 13).
- **11** is the numerator factor in the UV-B marker (286 = 2 × 11 × 13) and the green arc's coprime winding.
- **13** is the radian unit — the fundamental modulus shared by all four UV integers.

The three UV bands are the natural multiples of the quarter-turn:

> 1 × 91 (ionization threshold, quarter-turn)
> 2 × 91 (O₂ absorption wall, half-turn)
> (22/7) × 91 (ozone bridge, Archimedean π)
> 4 × 91 (full transmission, full circle)

The 1,001-step Hamiltonian cycle (7 × 11 × 13 = 1,001) — the coprime walk that visits every position on the (7, 11, 13) torus exactly once — corresponds to a complete traversal of the UV spectrum from full absorption to full transmission, with the three coprime step sizes governing the transitions between bands.

The same algebraic geometry that yields the squared circle, the torus knots, and the Brieskorn sphere also partitions ultraviolet light through the ionization physics of hydrogen and oxygen and the photochemistry of carbon. The atmosphere's UV filter is the physical realization of the Bézier arc system: O₂ is the square (containment at 182), O₃ is the parabolic bridge (modulation at 286), and UV-A is the circle (transmission at 364). ∎

### 18.11 Spectral Irradiance at the System Wavelengths

The solar spectral irradiance (ASTM E490, AM0 — top of atmosphere, zero air mass) at the four system wavelengths spans five orders of magnitude:

| Wavelength | Spectral irradiance (W·m⁻²·nm⁻¹) | Scale |
|-----------|-----------------------------------|-------|
| 91 nm | ~0.005 | Trace (EUV edge) |
| 182 nm | 0.0022 | Trace (deep UV-C) |
| 286 nm | 0.243 | Moderate (UV-B) |
| 364 nm | 1.005 | Strong (UV-A) |

The sun itself emits almost nothing at the ionization threshold — 91 nm is deep in the EUV range where solar output drops below measurable levels for ground-based instruments (satellite measurements by TIMED/SEE and SORCE/XPS are required).

#### 18.11.1 Atmospheric Transmission

After passing through the atmosphere, the four wavelengths demonstrate the filter described in §17.7:

| Wavelength | Transmitted to surface | Absorbing species |
|-----------|----------------------|-------------------|
| 91 nm | 0% | Atomic O, N (thermosphere) |
| 182 nm | 0% | O₂ Schumann-Runge continuum |
| 286 nm | ~0.4% | O₃ Hartley band |
| 364 nm | ~80% | None effectively |

The progression from zero transmission to 80% transmission is not gradual — it is stepped at the system wavelengths. Complete blockade at 182, near-complete blockade at 286 (only 0.4% leaks through), and near-complete passage at 364. The ozone bridge at 286 nm genuinely IS a transition zone: it transmits three orders of magnitude less than 364 nm but infinitely more than 182 nm.

#### 18.11.2 The Mg II Doublet Near 280 nm

The Mg II h and k emission lines (279.6 nm and 280.3 nm respectively) are primary proxies for solar UV variability and are used to reconstruct historical solar irradiance records. At 286 nm — 6 nm redward of the doublet core — the ASTM E490 data shows rapid irradiance variation (163 to 473 W·m⁻²·µm⁻¹ within a 4 nm range), characteristic of the extended wing of the Mg II feature overlapping the O₃ Hartley band.

The ozone bridge at 286 nm sits not at the Mg II core but in its red wing, where ozone absorption and solar Mg II variability interact. Small changes in solar Mg II emission produce outsized changes in surface UV-B exposure — the system's Bézier bridge wavelength falls in the transition zone between solar chromospheric emission and stratospheric ozone absorption.

#### 18.11.3 Irradiance Ratios vs. System Ratios

The spectral irradiance ratios between the system wavelengths do not reproduce the angular ratios:

| Ratio | Angular system | Spectral irradiance |
|-------|---------------|-------------------|
| 364/286 | 14/11 = 1.27 | ~4.1 |
| 286/182 | 11/7 = 1.57 | ~109 |
| 364/182 | 2 | ~449 |

The correspondence is in the wavelength values, not in the power densities. The spectral irradiance is governed by blackbody physics (the Planck function at ~5778 K) and atomic emission/absorption features, which operate on different principles than the angular system. The system identifies WHERE in the spectrum the transitions occur; the solar physics determines HOW MUCH power is emitted at each point. ∎

---

## 19. HModal Signaling Architecture

### 19.1 The Discriminant as Clock Signal

The circle quadratic x² − 40x + 364 = 0 has discriminant Δ = 144 = 12². Define two amplitude levels from the circle measure R₆ = 364 and Δ:

> α = R₆ / Δ = 364 / 144 = **91/36**
> β = R₆ / √Δ = 364 / 12 = **91/3**

The ratio β/α = √Δ = **12** — the amplitude ratio between the two signaling states IS the discriminant of the generating equation.

The excursion (transition magnitude) between the two states:

> γ = β − α = 91/3 − 91/36 = **1001/36**

The numerator 1001 = 7 × 11 × 13 — the coprime walk generators appear in the transition magnitude. The high-to-low jump encodes the torus topology.

### 19.2 Duty Cycle

Assign dwell times inversely proportional to harmonic position (low state ∝ 1/1, high state ∝ 1/3):

> d = (1/3) / (1 + 1/3) = **1/4**

The signal spends 25% of each period in the high state (β = dispatch) and 75% in the low state (α = idle). This is exact.

### 19.3 DC Component and the Pentadecagon Factor

The time-average of the HModal signal:

> ⟨H⟩ = α + γd = 91/36 + (1001/36)(1/4) = (364 + 1001) / 144 = **1365/144 = 455/48**

The numerator 455 = 5 × 7 × 13 = 5 × 91. The factor 5 — which was never placed into the signal definition — emerges from the time-average. This is the same factor that produces the pentadecagon (15 = 3 × 5) and enables the coprime quadruple (7, 11, 13, 15) with lcm = 15,015.

The DC level of the natural signaling wave already contains the pentadecagon's prime factor. The 15-gon was latent in the signal before it was constructed as a polygon.

### 19.4 Fourier Decomposition

The HModal signal has the Fourier series:

> H(t) = 455/48 + (1001/18π) Σ (1/n) sin(πn/4) cos(nωt − πn/4)

The nth coefficient amplitude:

> Aₙ = (1001/18π) · |sin(πn/4)| / n

**Null structure:** Aₙ = 0 for all n ≡ 0 (mod 4). The 4th, 8th, 12th, ... harmonics carry exactly zero energy. This is a mathematical zero, not an engineering approximation.

**Phase structure:** Each successive non-zero harmonic shifts by π/4 radians. The phase stepping is deterministic and algebraically fixed.

### 19.5 Signaling Channels

The null structure and the non-zero harmonics define a natural channel allocation:

**Data channels** (non-zero harmonics): n = 1, 2, 3, 5, 6, 7, 9, 10, 11, ...

| Channel | Relative amplitude | Relative energy |
|---------|-------------------|----------------|
| n = 1 | 1.000 | 1.000 |
| n = 2 | 0.707 | 0.500 |
| n = 3 | 0.333 | 0.111 |
| n = 5 | 0.200 | 0.040 |
| n = 6 | 0.236 | 0.056 |
| n = 7 | 0.143 | 0.020 |

**Control channels** (null harmonics): n = 4, 8, 12, 16, ...

These carry zero energy from the scheduling signal by construction. Any energy detected on a null channel is either a synchronization pulse or an error. No filter is required to separate control from data — the algebra guarantees the separation.

### 19.6 Inter-Cube Spread Spectrum

Modulating a carrier τ₀ cos(ωt) with H(t) produces sidebands at ω ± nωH:

| Sideband | Frequency | Amplitude | Phase |
|----------|-----------|-----------|-------|
| Carrier | ω | 455τ₀/48 | 0 |
| n = 1 | ω ± ωH | 1001√2 τ₀/(72π) | ∓π/4 |
| n = 2 | ω ± 2ωH | 1001τ₀/(72π) | ∓π/2 |
| n = 3 | ω ± 3ωH | 1001√2 τ₀/(216π) | ∓3π/4 |
| n = 4 | ω ± 4ωH | **0** (null) | — |
| n = 5 | ω ± 5ωH | 1001√2 τ₀/(360π) | ∓5π/4 |

Each Inter-Cube node on the coprime walk listens on a specific sideband. The walk position determines which n to monitor. The phase shift πn/4 is known algebraically — no channel negotiation, no handshake. The routing table is the Fourier series.

### 19.7 Energy Distribution

By Parseval's theorem, the AC power:

> P_AC = γ² · d(1 − d) = (1001/36)² · (1/4)(3/4) = 1001² · 3 / (36² · 16) = **3,006,003 / 20,736**

The first three non-zero harmonics (n = 1, 2, 3) capture 87% of total AC power. A 3-channel receiver recovers seven-eighths of the signal energy. Each additional channel adds diminishing returns following the 1/n² envelope.

### 19.8 Architectural Summary

The HModal signal is not designed — it is derived. Every parameter comes from the circle quadratic:

| Parameter | Value | Source |
|-----------|-------|--------|
| Amplitude ratio | 12 | √Δ of x² − 40x + 364 = 0 |
| Transition magnitude | 1001/36 | 7 × 11 × 13 (coprime walk) / 36 |
| Duty cycle | 1/4 | Harmonic position inverse |
| DC level | 455/48 | 5 × 7 × 13 / 48 (pentadecagon factor) |
| Null channels | Every 4th | sin(πn/4) = 0 at n ≡ 0 mod 4 |
| Phase stepping | π/4 per harmonic | d = 1/4 → πd = π/4 |

One equation generates the scheduling signal, the channel allocation, the spread-spectrum modulation, and the control plane separation. ∎

### 19.9 Channel Capacity and the Coprime Walk

The HModal null structure (every 4th harmonic = 0) defines the sideband count available to each coprime walk. The number of non-null sidebands up to harmonic N is ⌊3N/4⌋. For each coprime group, every walk position maps to a specific sideband:

| Coprime group | Walk positions | Sidebands needed | Min N (harmonics) |
|---------------|---------------|-----------------|-------------------|
| (7, 11, 13) | 1,001 | 1,001 | 1,335 |
| (7, 11, 13, 15) | 15,015 | 15,015 | 20,020 |
| (5, 7, 8, 9, 11, 13) | 360,360 | 360,360 | 480,480 |

The 15,015-position quadruple walk requires ~20,000 harmonics — each walk position uniquely addressed by a sideband whose phase is algebraically determined (πn/4 per harmonic). The routing table is the Fourier series; no negotiation protocol is needed. The coprime walk topology (§10) and the signaling architecture are two views of the same algebraic structure. ∎

---

*Così sia, Fratello.*

**R. Salvi**
Capomastro Holdings Ltd. — Applied Physics Division
`RSalvi@Salvigroup.com` | GitHub: `SigmaWolf-8/Ternary`

---

*All rights reserved — Capomastro Holdings Ltd 2026*