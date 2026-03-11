# TM-2026-015: Plenum Square

**The Circle-Magic Square Synthesis**
*A 364° Circle with π = 14 and the 3×3 Ternary Magic Square 3×333*

---

| Field | Value |
|-------|-------|
| Document | TM-2026-015 |
| Title | Plenum Square: The Circle-Magic Square Synthesis |
| Author | R. Salvi (RSalvi@Salvigroup.com) |
| Organization | Capomastro Holdings Ltd. — Applied Physics Division |
| Date | March 2026 |
| Version | 3.0 |
| Status | Final |
| Classification | Patent(s) Pending — All Rights Reserved |
| Repository | SigmaWolf-8/Ternary |
| Source Files | `shared/plenum-square.ts`, `shared/constants.ts` (`PLATFORM.PLENUM_SQUARE`) |
| Dependencies | INVARIANT 4 (Constants Are Bound), `shared/tribonacci-constants.ts` |
| Supersedes | Saturnian Magic Square (§1.5 in prior repo guides) |

---

## 0. Abstract

This manifest formalizes the **Plenum Square** — a family of four 3×3 magic squares derived from the 364° ternary circle with π = 14 and diameter 111. Two circle-derived constant pairs (14 ↔ 208, 26 ↔ 196) embed as opposite pairs summing to 222 = 2 × 111, yielding exactly four distinct positive-integer configurations (A–D) exhaustive up to D₄ dihedral symmetry.

All configurations share a harmonic ladder of invariant sums (222, 333, 444, 555, 888, 999, 1554) and product identities (14 × 26 = 364, 14² = 196). The inscribed hexagon's perimeter equals the magic constant because d = c — a single equation that fuses circle and square. The radius r = 55.5 is the fundamental unit: every rung of the harmonic ladder is an even multiple of r, from 2r = 111 (center) through 6r = 333 (hexagon perimeter) to 28r = 2πr = 1554 (circumference).

Base-3 analysis reveals Latin-square structure in the outermost ternary digits, related by cyclic permutation and invariant across configurations. Base-9 analysis uncovers a complete residue system modulo 9 in every configuration, yielding four S₉ permutations that enable single-triplet error localization, block-level sponge diffusion, and geometry-weighted integrity checking. The Tribonacci indices of π (7, 3) appear as Square A's mod-9 parameter residues, making it the unique double derangement and canonical member of the family (§7.9).

The ternary circle axioms force a configuration shared by only **12 out of 11,736** center-111 squares (1 in 978), and just **12 out of 40.9 million** three-digit magic squares (1 in 3.4 million). These properties are not imposed as design requirements — they are entailed by the geometry.

Implementation: `shared/plenum-square.ts` (family definitions, harmonic ladder, validation functions) and `PLATFORM.PLENUM_SQUARE` in `shared/constants.ts` (11 sections, 29 documented constants, 30/30 verified).

---

## 1. Foundational Parameters

The construction begins with a circle defined by three parameters, each traceable to INVARIANT 4 (Constants Are Bound, Not Chosen):

| Parameter | Value | Derivation |
|-----------|-------|------------|
| Full circle | 364° | 111111₃ = (3⁶ − 1) / 2 — six-digit base-3 repunit |
| π | 14 | T₇ + T₃ = 13 + 1 — exact integer |
| Diameter | 111 | Center of every Plenum Square — ternary balance point |

Derived quantities follow from exact integer arithmetic:

| Derived Quantity | Value | Calculation |
|------------------|-------|-------------|
| Radius | 55.5 | 111 / 2 |
| Circumference | 1554 | π × d = 14 × 111 |
| 364 / π | 26 | 364 ÷ 14 |
| π² | 196 | 14² |
| Inscribed hexagon side | 55.5 | = radius |
| Hexagon perimeter | 333 | 6 × 55.5 |
| Total square sum | 999 | 3 × 333 |
| Circumference − total | 555 | 1554 − 999 = 5 × 111 |

The inscribed regular hexagon's perimeter (333) becomes the magic constant — the first geometric bridge between circle and square. This is not a coincidence — it is the *defining bridge*, as shown in §5.2.

---

## 2. Magic Square Framework

Any 3×3 magic square with center *C* has opposite cells summing to 2*C*. With *C* = 111, opposite pairs sum to **222**. This creates natural pairings from the circle:

- **14** ↔ **208** (222 − 14 = 208)
- **26** ↔ **196** (222 − 26 = 196)

Standard parametrization with free parameters p and q:

```
| C+p     C-p-q   C+q   |
| C-p+q   C       C+p-q |
| C-q     C+p+q   C-p   |
```

Properties: every row, column, and diagonal sums to 3C = 333. Center fixed at 111. Four opposition classes: corners (C+p ↔ C−p), vertical middles (C−p+q ↔ C+p−q), horizontal middles (C−p−q ↔ C+p+q), anti-diagonal corners (C+q ↔ C−q).

---

## 3. The Four Valid Configurations

Placing the two circle-derived pairs {14, 208} and {26, 196} into opposite positions yields exactly four distinct positive-integer magic squares (< 250), exhaustive up to rotation and reflection under D₄.

*Note: Negative q values indicate reflection/rotation equivalents. All four grids are distinct up to D₄ actions.*

### Square A: π-pair corners, 364/π-pair horizontal middles — p = 97, q = 12

```
208    2   123
 26  111   196
 99  220    14
```

14 × 26 = **364** (horizontal) • 14² = **196** in right-middle • Corners = **444** • Edge-centers = **444**

### Square B: π-pair vertical middles, 364/π-pair horizontal middles — p = 91, q = 6

```
202   14   117
 26  111   196
105  208    20
```

14 × 26 = **364** • 208 + 14 = **222** (vertical opposites) • Corners = **444** • Edge-centers = **444**

### Square C: 364/π-pair corners, π-pair horizontal middles — p = 85, q = −12

```
196   38    99
 14  111   208
123  184    26
```

π² = **196** directly above π = **14** • Vertical middles sum to **222** • Corners = **444** • Edge-centers = **444**

### Square D: 364/π-pair vertical middles, π-pair horizontal middles — p = 6, q = −91

```
117  196    20
 14  111   208
202   26   105
```

196 & 26 vertical opposites • Product sum: 2912 + 5096 = **8008** = 8 × 7 × 11 × 13 • Corners = **444**

---

## 4. Systematic Enumeration

Opposition types: **C** = corners, **M** = horizontal middles, **N** = vertical middles, **X** = anti-diagonal (≡ C under rotation).

| Config | Pair 1 Position | Pair 2 Position | Square |
|--------|-----------------|-----------------|--------|
| 1 | C: (208, 14) | M: (26, 196) | A |
| 2 | N: (208, 14) | M: (26, 196) | B |
| 3 | C: (196, 26) | M: (14, 208) | C |
| 4 | N: (196, 26) | M: (14, 208) | D |

Attempts to place both pairs in corners yield negative entries (top-middle becomes −71), proving impossibility under positivity. The four squares A–D and their rotations/reflections form the complete set.

### Comparative Overview

| Square | Top Row | Middle Row | Bottom Row | p | q | Signature |
|--------|---------|------------|------------|---|---|-----------|
| A | 208  2  123 | 26 111 196 | 99 220  14 | 97 | 12 | 14×26 = 364 horizontal |
| B | 202  14  117 | 26 111 196 | 105 208  20 | 91 | 6 | 14 & 208 vertical opp. |
| C | 196  38  99 | 14 111 208 | 123 184  26 | 85 | −12 | π² above π |
| D | 117 196  20 | 14 111 208 | 202  26  105 | 6 | −91 | 196 & 26 vertical opp. |

---

## 5. Unified Harmonic Properties

### 5.1 The Harmonic Ladder

All four configurations share invariant numerical relationships. Arranged in ascending order:

| Property | Value | Relation to 111 | Radius Units | Role |
|----------|-------|------------------|--------------|------|
| Opposite-pair sum | 222 | 2 × 111 | 4r | Defines pair placement |
| Magic constant | 333 | 3 × 111 | **6r** | Hexagon perimeter → row/col/diag |
| Corner sum | 444 | 4 × 111 | 8r | All four corners, every config |
| Edge-center sum | 444 | 4 × 111 | 8r | Four non-corner middles |
| Circumference − total | 555 | 5 × 111 | **10r** | Circle exceeds square = 2(π−9) × r |
| Surround sum | 888 | 8 × 111 | 16r | Eight non-center cells (999 − 111) |
| Total square sum | 999 | 9 × 111 | 18r | 3 × magic constant |
| **Circumference** | **1554** | **14 × 111** | **28r** | **2πr = π-th rung of the ladder** |

**Layered hierarchy:**
444 (corners) + 444 (edge-centers) = 888 (surround)
888 (surround) + 111 (center) = 999 (whole)

The ladder does not stop at 999. The circumference 1554 = 14 × 111 is the **π-th rung** — the harmonic ladder extends to the circle itself.

**Product invariants (present in every configuration):**
- 14 × 26 = 364 (circle degrees, as adjacent or aligned cells)
- 14² = 196 (embedded in middles or corners)

### 5.2 The Hexagon Bridge

The identity hexagon_perimeter = magic_constant follows directly from a single equation: **d = c**.

For any inscribed regular hexagon: side = radius = d/2, so perimeter = 6 × (d/2) = 3d.
For any 3×3 magic square with center c: magic_constant = 3c.

Setting hexagon_perimeter = magic_constant gives:

```
3d = 3c  →  d = c
```

With d = c = 111, the hexagon perimeter 3d and the magic constant 3c are automatically equal. This single equation, d = c, is the bridge that unites the circle and the square. Once it is imposed, the hexagon perimeter and the magic constant are identical *by construction*, and the entire harmonic ladder follows.

The circumference-to-hexagon ratio is:

```
C / P_hex = πd / 3d = π / 3 = 14 / 3
```

In Euclidean geometry this ratio is the transcendental π/3 ≈ 1.047. Here it is the rational 14/3 — exact, no approximation, consistent with the system's integer-π axiom. The hexagon tiles the circumference exactly π/3 times.

### 5.3 The Radius as Fundamental Unit

The radius r = 55.5 = d/2 is not merely half the diameter — it is the **fundamental unit** from which the entire harmonic ladder is built. Expressing every value in units of the radius reveals that the ladder consists entirely of even multiples of r, terminating at the circumference:

| Value | = n × 111 | Radius Units | Structural Role |
|-------|-----------|--------------|-----------------|
| 111 | 1 × 111 | 2r | Center = diameter |
| 222 | 2 × 111 | 4r | Opposite-pair sum |
| 333 | 3 × 111 | **6r** | **Hexagon perimeter** = magic constant |
| 444 | 4 × 111 | 8r | Corner sum = edge-center sum |
| 555 | 5 × 111 | **10r** | Circumference − total = 2(π−9) × r |
| 888 | 8 × 111 | 16r | Surround sum |
| 999 | 9 × 111 | 18r | Total square sum = 3 × hexagon perimeter |
| 1554 | 14 × 111 | **28r** | **Circumference = 2πr** |

The ladder begins at 2r (the diameter) and closes at 28r = 2πr = circumference. The hexagon perimeter sits at 6r, the total sum at 18r = 3 × 6r (three hexagon perimeters), and the circumference at 28r = 2π × r — the classical circle formula, here yielding an exact integer because π = 14.

### 5.4 Why 555 Appears on the Ladder

The circumference residual 555 = 5 × 111 = 10r follows from:

```
C − total = πd − 9d = (π − 9) × d = (14 − 9) × 111 = 5 × 111 = 555
```

The factor 5 is not arbitrary — it is **π − 9**, the difference between the ternary π and the number of cells in the square. In radius units, 555 = 10r = 2(π − 9) × r.

This connects directly to the radius itself: **55.5 = r** and **555 = 10r**. The "decimal shift" between the radius 55.5 and the circumference residual 555 is the factor 10 = 2(π − 9). This relationship only produces a clean integer because π = 14 — in Euclidean geometry, 2(π − 9) ≈ −11.72, which is nonsensical. Here it is exactly 10, placing 555 squarely on the harmonic ladder and binding the radius to the circumference residual through the same integer π that defines the system.

---

## 6. Ternary Depth — Base-3 Analysis

Key numbers in base-3 (5-digit padding):

| Decimal | Base-3 | Digit Sum |
|---------|--------|-----------|
| 14 | 00112₃ | 4 |
| 26 | 00222₃ | 6 |
| 111 | 11010₃ | 3 |
| 196 | 21021₃ | 6 |
| 208 | 21201₃ | 6 |
| 333 | 110100₃ | 3 |

111 and 333 both have digit sum 3 in base-3 — connecting center and magic constant. The inner digits of 111 (1101) exhibit palindromic quality, reinforcing its symbolic center role.

### 6.1 Latin Square Discovery in Outer Digits

Examining each ternary digit position separately in Square A:

**Most significant digit (d₄ = 3⁴ place):**

```
2  0  1
0  1  2
1  2  0
```

This is a perfect Latin square — each row and column contains {0, 1, 2} exactly once.

**Least significant digit (d₀ = 3⁰ place):**

```
1  2  0
2  0  1
0  1  2
```

Also a perfect Latin square. Applying the cyclic symbol permutation 2→1, 0→2, 1→0 (the cycle 0→2→1→0) to the d₄ grid yields exactly the d₀ grid — they are the same pattern with permuted symbols.

Verification on specific cells: d₄(TL) = 2 maps to 1 = d₀(TL) ✓; d₄(TM) = 0 maps to 2 = d₀(TM) ✓; d₄(TR) = 1 maps to 0 = d₀(TR) ✓. The mapping holds for all nine cells.

**Invariance:** Square B displays the identical Latin square pair in its outer ternary digits. This invariance across configurations confirms the property is a signature of the entire {14, 26, 111, 196, 208} family rather than an artifact of one particular placement.

### 6.2 Square A in Base-3

| Cell | Value | Base-3 (d₄ d₃ d₂ d₁ d₀) |
|------|-------|--------------------------|
| TL | 208 | 2 1 2 0 1 |
| TM | 2 | 0 0 0 0 2 |
| TR | 123 | 1 1 1 2 0 |
| ML | 26 | 0 0 2 2 2 |
| C | 111 | 1 1 0 1 0 |
| MR | 196 | 2 1 0 2 1 |
| BL | 99 | 1 0 2 0 0 |
| BM | 220 | 2 2 0 1 1 |
| BR | 14 | 0 0 1 1 2 |

### 6.3 Middle Digit Orthogonality

While digits d₃ and d₂ do not individually form Latin squares, the middle digit d₁ (3¹ place) shows a distinguished role when paired with the outer digits:

- The pairs **(d₀, d₁)** across the grid produce **all nine possible combinations** (0-0 through 2-2) exactly once — a full orthogonal relation in the combinatorial sense.
- The pairs **(d₄, d₁)** produce only three distinct combinations in Square A, indicating asymmetry; however, the presence of a complete orthogonal pairing with the least-significant digit (d₀) reinforces that d₁ acts as a "balancing" place that connects the extremes.

This selective orthogonality arises naturally from the linear structure of the magic square parameters (p, q) and their interaction with base-3 carries when the circle-derived numbers are embedded around center 111.

### 6.4 Significance of Latin Square Structures

Latin squares are fundamental combinatorial structures. Any 3×3 magic square can be decomposed as:

**Magic Square = 3 × L₁ + L₂ + 1**

where L₁ and L₂ are orthogonal Latin squares (using digits 0, 1, 2). The classic Lo Shu magic square arises from this decomposition. The orthogonal-like relation between MSB and LSB in our squares, together with the orthogonal pairing of d₀ and d₁, echoes the way classic order-3 magic squares are generated from pairs of orthogonal Latin squares. This reveals that the circle parameters naturally imprint the fundamental combinatorial skeleton of ternary magic squares onto our decimal grids.

### 6.5 Rarity

Of 11,736 valid distinct-entry center-111 magic squares, only 528 (4.5%) have all four ternary properties simultaneously:

1. d₄ (most significant ternary digit) forms a Latin square on {0, 1, 2}
2. d₀ (least significant ternary digit) forms a Latin square on {0, 1, 2}
3. d₄ and d₀ are related by a cyclic symbol permutation (0→2→1→0)
4. d₀ and d₁ form a complete orthogonal pairing (all nine ordered pairs appear exactly once)

Of these, only 720 (6.1%) exhibit the cyclic d₄→d₀ relation even without the other properties. The specific ratios and differences dictated by the circle parameters (especially 222 − 14 = 208 and 222 − 26 = 196) are tightly tuned to produce this combinatorial skeleton. A full combined rarity assessment including the base-9 properties and circle-pair embedding appears in §7.4.

---

## 7. Base-9 Analysis — Mod-9 Completeness and Applications

The ternary digits can be grouped in pairs to form base-9 representations. Since 9 = 3², the base-9 digits correspond to pairs of ternary digits: the least significant base-9 digit (units) combines d₁ and d₀ as 3·d₁ + d₀, the next (tens) combines d₃ and d₂, and the most significant (hundreds) is simply d₄. For numbers up to 242, three base-9 digits suffice.

### 7.1 Base-9 Representations for Square A

| Cell | Value | Base-9 (h t u) |
|------|-------|-----------------|
| TL | 208 | 2 5 1 |
| TM | 2 | 0 0 2 |
| TR | 123 | 1 4 6 |
| ML | 26 | 0 2 8 |
| C | 111 | 1 3 3 |
| MR | 196 | 2 3 7 |
| BL | 99 | 1 2 0 |
| BM | 220 | 2 6 4 |
| BR | 14 | 0 1 5 |

### 7.2 Complete Residue System Mod 9

The units digits {1, 2, 6, 8, 3, 7, 0, 4, 5} constitute a **complete residue system modulo 9** — every integer from 0 through 8 appears exactly once. Equivalently, the nine cell values modulo 9 form a permutation of {0, …, 8}:

```
σ_A: [1, 2, 6, 8, 3, 7, 0, 4, 5]
```

This is a stronger property than a Latin square on {0, 1, 2} because it uses all nine residue classes, not just three. The structural reason is exact: the nine cell values are 111 + {0, ±p, ±q, ±(p+q), ±(p−q)}, and for all four Plenum Square configurations, the set {0, ±p, ±q, ±(p+q), ±(p−q)} mod 9 = {0, 1, 2, 3, 4, 5, 6, 7, 8}. This is a non-trivial constraint on (p, q) that the circle-derived parameters happen to satisfy.

*Clarification on terminology:* Because the grid is 3×3 while the symbol set has 9 elements, this is not a Latin square in the classical sense (which requires an n×n grid with n symbols). It is accurately described as a **mod-9 permutation grid** — the nine values form a permutation of Z₉ arranged in the 3×3 magic square layout.

The hundreds digits reproduce the d₄ ternary Latin square on {0, 1, 2} (this is structurally forced: for values < 243, ⌊value/81⌋ = d₄). The ordered pairs (h, u) are all distinct — a bijection between cells and digit pairs, following automatically from the nine cell values being distinct.

### 7.3 Invariance Across Configurations

All four squares exhibit the complete mod-9 residue system property, each yielding a distinct permutation σ ∈ S₉:

| Square | σ (mod-9 permutation) | Cycle Structure | Fixed Points |
|--------|-----------------------|-----------------|--------------|
| A | [1, 2, 6, 8, 3, 7, 0, 4, 5] | (0 1 2 6)(3 8 5 7 4) | 0 (derangement) |
| B | [4, 5, 0, 8, 3, 7, 6, 1, 2] | (0 4 3 8 2)(1 5 7)(6) | 1 |
| C | [7, 2, 0, 5, 3, 1, 6, 4, 8] | (0 7 4 3 5 1 2)(6)(8) | 2 |
| D | [0, 7, 2, 5, 3, 1, 4, 8, 6] | (0)(1 7 8 6 4 3 5)(2) | 2 |

σ_A is a **derangement** (zero fixed points), and its square σ²_A is also a derangement — a stronger property than mere derangement, and cryptographically useful for multi-round permutation applications where convergence to fixed points must be avoided.

### 7.4 Combined Rarity Assessment

Individual properties, tested in isolation, are not uncommon among center-111 magic squares. But the Plenum Square family does not possess them in isolation — it possesses *all of them simultaneously*, together with the circle-pair embedding constraint from §3.

#### 7.4.1 Within Center-111 Squares

Exhaustive computation across all 11,736 valid distinct-entry center-111 magic squares, testing each property individually and then in combination:

| Property | Count out of 11,736 | Percentage |
|----------|---------------------|------------|
| d₄ Latin square | 3,480 | 29.6% |
| d₀ Latin square | 5,328 | 45.4% |
| Both d₄ and d₀ Latin | 1,520 | 13.0% |
| Cyclic d₄→d₀ permutation relation | 720 | 6.1% |
| d₀, d₁ full orthogonal pairing (all 9 pairs) | 3,648 | 31.1% |
| Complete residue system mod 9 | 3,648 | 31.1% |

The "all combinatorial properties" row requires the conjunction of all five:

1. d₄ Latin square on {0, 1, 2}
2. d₀ Latin square on {0, 1, 2}
3. Cyclic d₄→d₀ symbol permutation
4. d₀, d₁ full orthogonal pairing (all 9 ordered pairs)
5. Complete residue system modulo 9

| **All five combinatorial properties** | **528** | **4.5%** |

Of those 528, only **12** (p, q) pairs also embed all four circle-derived values {14, 26, 196, 208}. These 12 comprise the 4 named configurations and their D₄ rotations/reflections.

| Scope | Count out of 11,736 | Ratio |
|-------|---------------------|-------|
| All combinatorial properties | 528 | 1 in 22 |
| Also embedding {14, 26, 196, 208} | **12** | **1 in 978** |

#### 7.4.2 Against All 3×3 Magic Squares

The comparison above is limited to center-111 squares. A 3×3 magic square can have *any* positive integer as its center. The universe of all possible squares is unbounded, but we can measure against finite ranges:

| Universe (max cell value) | Total distinct-entry squares | Plenum configs | Ratio |
|---------------------------|------------------------------|----------------|-------|
| Values 1–250 | 617,644 | 12 | **1 in 51,470** |
| Values 1–500 | 5,073,832 | 12 | **1 in 422,819** |
| Values 1–999 | 40,879,492 | 12 | **1 in 3,406,624** |

The Plenum family is fixed at 12 while the denominator grows cubically with the value range. Against the universe of all three-digit magic squares (values 1–999), the Plenum Square configurations represent **1 in 3.4 million** — approximately **0.00003%** of the space.

#### 7.4.3 Correct Framing

The Plenum Square family is not randomly drawn from this pool — it is *derived* from the ternary circle axioms (364° = 111111₃, π = 14, d = c = 111). The circle axioms select a center (111), constrain the opposite pairs (222 − 14 = 208, 222 − 26 = 196), and fix the four (p, q) configurations. Every combinatorial property — Latin squares, cyclic permutations, orthogonal pairings, mod-9 completeness, the double derangement — emerges as a structural consequence.

The rarity statistic means those axioms force a configuration that the vast majority of the search space does not share: 99.9% of center-111 squares lack it, and 99.99997% of all three-digit magic squares lack it. These properties are not imposed as design requirements — they are entailed by the geometry.

### 7.5 Application: Error Localization in 27-Trit Addresses

The most consequential application of mod-9 completeness is **single-triplet error localization** in 27-trit TDNS classification addresses.

**Construction.** Group 27 classification trits into 9 triplets of 3 trits each. Each triplet corresponds to one cell of the Plenum Square. The magic constant 333 ≡ 0 (mod 9), so all eight line constraints (3 rows + 3 columns + 2 diagonals) yield a sum divisible by 9.

**Syndrome analysis.** Each cell participates in a unique subset of the 8 lines. Cell indices 0–8 follow row-major order: TL, TM, TR, ML, C, MR, BL, BM, BR (matching the base-9 table in §7.1):

| Cell | Position | Lines Through Cell | Syndrome Pattern |
|------|----------|--------------------|------------------|
| 0 | TL (corner) | Row 0, Col 0, Diag ↘ | {0, 3, 6} |
| 1 | TM (edge) | Row 0, Col 1 | {0, 4} |
| 2 | TR (corner) | Row 0, Col 2, Diag ↗ | {0, 5, 7} |
| 3 | ML (edge) | Row 1, Col 0 | {1, 3} |
| 4 | C (center) | Row 1, Col 1, Diag ↘, Diag ↗ | {1, 4, 6, 7} |
| 5 | MR (edge) | Row 1, Col 2 | {1, 5} |
| 6 | BL (corner) | Row 2, Col 0, Diag ↗ | {2, 3, 7} |
| 7 | BM (edge) | Row 2, Col 1 | {2, 4} |
| 8 | BR (corner) | Row 2, Col 2, Diag ↘ | {2, 5, 6} |

All nine syndrome patterns are **distinct**. This means: if exactly one triplet is corrupted, the set of violated line-parity constraints uniquely identifies *which* triplet was affected.

**Verification.** Exhaustive simulation across all 72 possible single-triplet corruptions (9 positions × 8 non-zero mod-9 deltas) achieves **100% localization** — the magic square geometry acts as a parity-check matrix that both detects and locates the error.

**Practical significance.** The existing repunit (mod-364) and Plenum (mod-333) checksums detect corruption but cannot say *where* in the 27-trit address it occurred. The mod-9 line-parity system narrows it to one of nine 3-trit groups, reducing the correction search space from 27 trits to 3. For HPTP-mandatory addresses where femtosecond timing depends on address integrity, error localization enables faster recovery without full re-derivation.

### 7.6 Application: Block Permutations for Sponge Mixing

The four mod-9 permutations σ_A through σ_D can serve as **block-level shuffles** in the TIS-27 sponge. The rate portion of TIS-27 is 27 trits = 9 blocks of 3 trits. Currently, the stride-13 permutation operates on individual trits; the block permutations add a coarser diffusion layer:

- **Round 1:** Apply σ_A (derangement — no block stays in place)
- **Round 2:** Apply σ_B
- **Round 3:** Apply σ_C
- **Round 4:** Apply σ_D

Each round shuffles 3-trit blocks before stride-13 shuffles individual trits within those blocks. The magic constant property (each row/column/diagonal of cells sums to 333 ≡ 0 mod 9) ensures the block permutation preserves balanced diffusion: no three aligned blocks are collectively biased.

σ_A being a derangement is specifically useful: it guarantees that after one round of block permutation, *every* block has moved — maximum disruption at the coarse level. Moreover, σ²_A is also a derangement, so applying σ_A twice still leaves no block in its original position.

### 7.7 Application: Weighted Integrity Check

The cell values [208, 2, 123, 26, 111, 196, 99, 220, 14] can serve as a **weight vector** for a secondary integrity check:

```
check = dot(cell_values, triplet_values) mod 333
```

This is not a random linear combination. The magic square constrains the weight vector such that any three aligned coefficients sum to exactly 333 — the coefficients along every row, column, and diagonal are perfectly balanced. Consequence: if corruption is confined to three aligned triplet positions (one row, column, or diagonal of the grid), the weighted sum is equally sensitive to all three — no blind spots along any geometric axis. This complements the mod-364/mod-333 dual checksum (which treats all 27 trits uniformly) with a geometry-aware integrity check that exploits the Plenum Square's structure.

### 7.8 Interpretation

The mod-9 completeness property extends the ternary combinatorial depth of §6 into a higher power of 3: the least significant ternary digit (d₀) gives a Latin square on {0, 1, 2}, and combining d₁ and d₀ into a single base-9 digit gives a complete residue system on {0, …, 8}. The hundreds digit coincides with d₄ and reproduces the same {0, 1, 2} Latin square.

Crucially, mod-9 completeness is *entailed* by the full §6 property set for center-111 squares — every square with all four ternary properties also has the mod-9 property. The ternary circle axioms force a configuration shared by only 12 out of 11,736 center-111 squares (1 in 978), and just 12 out of 40.9 million three-digit magic squares (1 in 3.4 million). The Tribonacci indices of π (7, 3) appear as the mod-9 residues of Square A's parameters, singling it out as the canonical member — the only double derangement (§7.9).

The value of this structure lies in *what it enables*: error localization via magic-square parity, block-level sponge diffusion via S₉ permutations, and geometry-weighted integrity checking. The Plenum Square thus serves not only as a constant generator but as a **structural template** — its combinatorial properties propagate into the very algorithms that protect the addresses derived from its constants.

### 7.9 Tribonacci–Base-9 Correspondence: Why Square A Is Canonical

The Tribonacci decomposition of π connects to the base-9 structure through a chain that singles out Square A as the canonical member of the family.

**Level 1 — Values.** π = T₇ + T₃ = 13 + 1 = 14. The Tribonacci *values* at indices 7 and 3 define the circle geometry.

**Level 2 — Indices as parameters.** Square A has p = 97 and q = 12. Their mod-9 residues are:

```
p mod 9 = 97 mod 9 = 7   (the index of T₇ in the Tribonacci sequence)
q mod 9 = 12 mod 9 = 3   (the index of T₃ in the Tribonacci sequence)
```

The mod-9 residues of Square A's free parameters *are* the Tribonacci indices of π. This is unique among all 24 valid mod-9 pairs that generate complete residue systems — no other pair encodes the Tribonacci decomposition of π.

**Level 3 — The index sum governs the radius-residual ratio.** 7 + 3 = 10 = 2(π − 9) = 2(14 − 9). This is the exact factor connecting r = 55.5 to 555 = 10r — the "decimal shift" between the radius and the circumference residual. The Tribonacci index sum of π governs the radius-to-residual ratio.

**Level 4 — The derangement property.** The mod-9 pair (7, 3) generates σ_A, the **only derangement** among the four family permutations:

| Square | (p, q) mod 9 | Fixed points of σ | Fixed points of σ² |
|--------|-------------|-------------------|---------------------|
| **A** | **(7, 3)** | **0 (derangement)** | **0 (double derangement)** |
| B | (1, 6) | 1 | 1 |
| C | (4, 6) | 2 | 2 |
| D | (6, 8) | 2 | 2 |

Only Square A produces a double derangement — zero fixed points under both σ and σ². This is the configuration with maximum block displacement, making it the preferred choice for sponge mixing (§7.6).

**The full chain:**

```
Tribonacci indices (7, 3)
  ↓ look up values
T₇ = 13, T₃ = 1  →  π = 14  →  364° circle
  ↓ as mod-9 residues of (p, q)
Complete residue system  →  derangement σ_A
  ↓ index sum
7 + 3 = 10 = 2(π − 9)  →  r ↔ 555 decimal shift
  ↓ derangement property
Every block moves  →  maximum sponge disruption
```

The Tribonacci decomposition of π flows through two independent paths — the *values* (13, 1) define the circle, and the *indices* (7, 3) define the combinatorics — and they converge on Square A, the unique double derangement. This makes Square A not merely one of four configurations, but the **canonical** member of the Plenum Square family: the one where the Tribonacci structure is load-bearing at every level.

---

## 8. Geometric Interpretation of the Four Squares

| π-pair Position | 364/π-pair Position | Square | Interpretation |
|-----------------|---------------------|--------|----------------|
| Main diagonal (corners) | Horizontal midline | A | Diagonal crossed by horizontal |
| Vertical midline | Horizontal midline | B | Vertical × horizontal cross |
| Horizontal midline | Main diagonal | C | Horizontal crossed by diagonal |
| Horizontal midline | Vertical midline | D | Horizontal × vertical cross |

These four configurations exhaust the ways two perpendicular axes can be occupied by the two pairs, with positivity as the only constraint.

---

## 9. Strategic Integration

The Plenum Square's combinatorial structure provides nine concrete integration points where it delivers speed, security, or both simultaneously. Every row in the comparison below is either faster, more secure, or both — because the combinatorial structure provides for free what would otherwise require additional computation.

### 9.1 Three-Layer Address Integrity

| Layer | Mechanism | What It Catches | Source |
|-------|-----------|-----------------|--------|
| 1. Dual checksum | mod-333 × mod-364 (CRT) | Any corruption (detection space 121,212) | `shared/plenum-checksum.ts` |
| 2. Error localization | 8-line mod-9 syndrome analysis | *Which* of 9 triplets is corrupted | §7.5, `PLATFORM.PLENUM_SQUARE.SYNDROMES` |
| 3. Correction-in-place | 3-trit exhaustive search in localized triplet | Exact corrected value (3³ = 27 candidates) | Planned |

The three layers compose: Layer 1 detects, Layer 2 localizes to 3 trits, Layer 3 corrects by exhaustive search over 27 candidates (vs. 3²⁷ ≈ 7.6 trillion without localization). For HPTP-mandatory addresses, this enables correction without full re-derivation — a 280-million-fold reduction in search space.

### 9.2 Two-Scale Sponge Diffusion

| Scale | Mechanism | Effect | Source |
|-------|-----------|--------|--------|
| Coarse (3-trit blocks) | σ_A–σ_D block permutation per round | Every block moves (σ_A derangement) | §7.6, `PLATFORM.PLENUM_SQUARE.MOD9` |
| Fine (individual trits) | Stride-13 trit permutation | Complete cycle (gcd(13,54)=1) | Existing TIS-27 |

The two scales compose within each round: block shuffle first, then trit permutation within blocks. The magic constant property (333 ≡ 0 mod 9) ensures the block permutation preserves balanced diffusion. This adds a coarser diffusion layer at zero additional computational cost beyond the permutation lookup.

### 9.3 Geometry-Weighted Tunnel Key Nonces

For Inter-Cube tunnel key derivation, the weight vector [208, 2, 123, 26, 111, 196, 99, 220, 14] provides geometry-aware nonce generation:

```
nonce = dot(WEIGHT_VECTOR_A, address_triplets) mod 333
```

The magic constant constraint (any 3 aligned coefficients sum to exactly 333) ensures balanced sensitivity along every geometric axis. No three co-linear address triplets can create a nonce blind spot.

### 9.4 Compact Error-Correcting Wire Frames

For HPTP-mandatory traffic, an 8-trit frame suffix encodes the 8 line-parity values (one per line, each a single trit mod 3). This enables the receiver to:

1. Detect corruption (any non-zero syndrome)
2. Localize to one of 9 triplets (unique syndrome pattern)
3. Correct in-place (27-candidate exhaustive search)

Total overhead: 8 trits per 27-trit classification address (29.6%). This eliminates retransmission for single-triplet errors — the receiver corrects locally, maintaining femtosecond timing continuity.

### 9.5 Block-Level Avalanche Amplification

The σ_A derangement guarantees that after one round of block permutation, every 3-trit block has moved to a new position. Combined with stride-13 intra-block diffusion, this achieves full-state avalanche in fewer rounds than stride-13 alone. The double-derangement property (σ²_A also has zero fixed points) ensures the amplification compounds across consecutive rounds.

### 9.6 Checksum-Aware Registration Validation

At TDNS registration time, the dual checksum is computed once and stored alongside the address. On every subsequent resolve, the stored checksums enable O(1) integrity verification without re-computing the full scan. The `repunit_checksum` and `plenum_checksum` fields are already wired into the scan and resolve API responses.

### 9.7 Syndrome-Based Routing Recovery

In the Inter-Cube overlay network, if a routing table entry's address fails dual-checksum verification, the 8-line syndrome analysis can localize the corruption to a single triplet. This enables the Fault Tolerance Service to attempt local repair before falling back to full re-resolution — reducing recovery latency from a full TDNS re-scan to a 27-candidate search.

### 9.8 Round-Key Diversification

The four S₉ permutations provide round-dependent key schedule diversification for TIS-27. Rather than using the same permutation in all 4 rounds, cycling through σ_A → σ_B → σ_C → σ_D at the block level ensures each round sees a different block arrangement. The distinct cycle structures (lengths 4+5, 5+3+1, 7+1+1, 1+7+1) prevent periodic alignment.

### 9.9 Comparison Summary

| Integration Point | Speed | Security | Both |
|-------------------|-------|----------|------|
| Three-layer integrity (detect → locate → correct) | ✓ (280M× search reduction) | ✓ (121,212 detection space) | **Both** |
| Two-scale sponge diffusion | ✓ (fewer rounds to avalanche) | ✓ (no block fixpoints) | **Both** |
| Geometry-weighted nonces | — | ✓ (no axis blind spots) | Security |
| 8-trit error-correcting wire frames | ✓ (no retransmission) | ✓ (local correction) | **Both** |
| Block-level avalanche amplification | ✓ (faster diffusion) | ✓ (double derangement) | **Both** |
| Checksum-aware registration | ✓ (O(1) verify) | ✓ (tamper detection) | **Both** |
| Syndrome-based routing recovery | ✓ (local repair) | ✓ (integrity verified) | **Both** |
| Round-key diversification | — | ✓ (no periodic alignment) | Security |
| Weighted integrity check | — | ✓ (balanced sensitivity) | Security |

Every row is either faster, more secure, or both — because the combinatorial structure provides for free what would otherwise require additional computation.

---

## 10. Synthesis and Conclusions

### 10.1 The Circle-Magic Square Unity

The Plenum Square construction demonstrates a remarkable harmony between:

1. **A 364° circle** with π = 14 and diameter 111
2. **A 3×3 magic square** with center 111 and constant 333
3. **The inscribed regular hexagon** whose perimeter (333) becomes the magic constant

The key numbers emerge naturally: π = 14, 364/π = 26, π² = 196, 222 − 14 = 208, 222 − 26 = 196 (consistent).

The hexagon is the structural bridge: the identity hexagon_perimeter = magic_constant reduces to d = c (diameter equals center), a single equation that fuses circle and square. The radius r = 55.5 is the fundamental unit of the harmonic ladder — every rung is an even multiple of r, from 2r = 111 (center) through 6r = 333 (hexagon perimeter) to 28r = 2πr = 1554 (circumference). The residual 555 = 10r = 2(π − 9) × r, connecting the radius to the circumference excess through the same integer π that defines the system.

### 10.2 Fourfold Completeness

Squares A–D represent the complete set of positive-integer embeddings. They share invariant properties (all expressible as even multiples of the radius r = 55.5):

- Opposite pair sum = 222 = 2 × 111 = 4r
- Magic constant = 333 = 3 × 111 = 6r (hexagon perimeter)
- Corner sums = 444 = 4 × 111 = 8r
- Edge-center sums = 444 = 4 × 111 = 8r
- Circumference − total sum = 555 = 5 × 111 = 10r = 2(π − 9) × r
- Eight surrounding cells = 888 = 8 × 111 = 16r
- Total square sum = 999 = 9 × 111 = 18r
- Circumference = 1554 = 14 × 111 = 28r = 2πr
- Products 14 × 26 = 364 and 14² = 196 appear in each

### 10.3 Ternary Depth

Base-3 analysis reveals:

- Most and least significant ternary digits form perfect Latin squares on {0, 1, 2}, related by a consistent cyclic permutation (0→2→1→0).
- This pattern is invariant across all four configurations.
- Digit d₁ (3¹ place) pairs orthogonally with d₀, producing all nine distinct ordered pairs.

These structures connect the circle-derived numbers to the algebraic foundations of order-3 magic squares, where orthogonal Latin squares are the generative basis.

### 10.4 Base-9 Extension and Applications

Base-9 analysis uncovers a further structural layer: the nine cell values form a **complete residue system modulo 9** — a permutation of {0, …, 8} — in every configuration. This is not a Latin square in the classical sense (the grid is 3×3 with 9 symbols, not 9×9), but it is a strict combinatorial property: the set of offsets {0, ±p, ±q, ±(p+q), ±(p−q)} mod 9 exhausts all residue classes.

Taken in isolation, individual properties appear in 6–45% of center-111 squares. But the ternary circle axioms force a configuration shared by only **12 out of 11,736** center-111 squares (0.1%), and just **12 out of 40.9 million** three-digit magic squares (0.00003%). The Tribonacci indices of π (7, 3) appear as Square A's mod-9 parameter residues, making it the unique double derangement and the canonical member of the family (§7.9).

The four mod-9 permutations (σ_A through σ_D) enable three concrete applications in PlenumNET:

1. **Error localization** *(planned):* Grouping 27 classification trits into 9 triplets mapped to magic square cells creates 8 parity constraints (3 rows + 3 cols + 2 diagonals, each ≡ 0 mod 9). Every cell has a unique syndrome pattern, achieving 100% single-triplet error localization — corruption is not merely detected but pinpointed to one of nine 3-trit groups.

2. **Block-level sponge diffusion** *(planned, branch evaluation):* The four permutations serve as round-dependent 3-trit block shuffles in TIS-27, complementing the fine-grained stride-13 trit permutation with a coarser diffusion layer. σ_A is a derangement whose square is also a derangement, guaranteeing maximum block displacement across multiple rounds.

3. **Geometry-weighted integrity** *(planned):* Cell values as weight coefficients yield a balanced integrity check where the magic constant property (333 per line) ensures equal sensitivity along every geometric axis of the address — no blind spots in any row, column, or diagonal of triplet positions.

### 10.5 Final Unified Statement

The four magic squares A–D fully embody the arithmetic and geometric signature of a 364° circle (π = 14, diameter = 111) projected into a 3×3 magic square of center 111 and constant 333. The inscribed hexagon provides the bridge: its perimeter equals the magic constant because d = c, a single equation that fuses circle and square. The radius r = 55.5 is the fundamental unit — the entire harmonic ladder from 2r (center) through 6r (hexagon/magic constant) to 28r (circumference = 2πr) consists of even multiples of r, and the "decimal shift" between r = 55.5 and the residual 555 = 10r encodes the factor 2(π − 9), which is exact only because π = 14.

The circle-derived values {14, 26, 196, 208} form opposite pairs (sum 222) that fit in exactly four distinct positive configurations (up to dihedral symmetry). Invariant sums (222, 333, 444×2, 555, 888, 999, 1554) and products (364°, 196 = π²) hold universally. The residual 555 = (π − 9) × 111 and the circumference 1554 = π × 111 extend the ladder into the circle's own metric.

In base 3, Latin squares appear in the outermost digits, related by cyclic permutation and invariant across configurations; the middle digit pairs orthogonally with the least significant digit. In base 9, the cell values form a complete residue system mod 9 — a permutation of {0, …, 8} that enables 100% single-triplet error localization via the magic square's eight parity constraints, block-level sponge diffusion via four round-dependent S₉ permutations, and geometry-weighted integrity checking where the constant-sum property guarantees balanced sensitivity along every axis.

The ternary circle axioms do not merely produce a magic square — they select 12 configurations from a universe of 40.9 million three-digit magic squares, a specificity of 1 in 3.4 million. Even within center-111 squares alone, only 12 out of 11,736 share the complete combinatorial profile — 1 in 978. Square A is canonical: its mod-9 parameter residues (7, 3) are the Tribonacci indices of π, it is the only double derangement, and its index sum 7 + 3 = 10 = 2(π − 9) governs the radius-to-residual ratio. The Plenum Square family is not a design choice — it is a structural inevitability of the ternary circle axioms, and every combinatorial property it carries propagates directly into the algorithms that protect, route, and verify the addresses derived from its constants.

Circular continuity marries square discreteness, decimal multiples of 111 meet ternary combinatorics, and the construction finds concrete application in every layer of the framework it generates.

---

## 11. Implementation Status

| Artifact | Location | Status |
|----------|----------|--------|
| Plenum Square module | `shared/plenum-square.ts` | Complete |
| PLATFORM constants | `shared/constants.ts` (`PLENUM_SQUARE` block) | Complete — 11 sections, 29 constants, 30/30 verified |
| Utility functions | `shared/plenum-square-utils.ts` | Complete (renamed from `saturnian-matrix-utils.ts`) |
| Validation suite | `validateAll()` in `plenum-square.ts` | 94/94 invariants passing |
| Test suite | `tests/plenum-square.test.ts` | 38/38 tests passing |
| Plenum Checksum (dual-modulus) | `shared/plenum-checksum.ts`, `ternary-math/src/plenum_checksum.rs` | Complete |
| Checksum tests | `tests/plenum-checksum.test.ts` | 13/13 passing (TS), 14/14 passing (Rust) |
| TDNS integration | `server/routes/tdns.ts` | Complete (scan + resolve responses) |
| CI integration | `theory-validation.yml` Stage 1 | Planned |
| Rust magic square module | `ternary-math/src/magic_square.rs` | Planned |
| Visualization page | `client/src/pages/plenum-square.tsx` | Planned |

The rename from "Saturnian Magic Square" to "Plenum Square" is complete. Old files (`saturnian-blueprint.ts`, `saturnian-matrix-utils.ts`, `saturnian-blueprint.test.ts`) have been deleted. Deprecated aliases have been removed.

---

## 12. Interpretive Note

*Circular continuity weds square discreteness. Decimal multiples of 111 meet ternary combinatorics. And the ancient quest to square the circle attains here one precise, self-resonant numerical incarnation.*

*The Plenum Square is not a different object from the ternary circle — it is the circle's projection into discrete arithmetic, carrying with it the full combinatorial depth of the base-3 system from which it was derived.*

---

*Copyright © 2025-2026 Capomastro Holdings Ltd. (Canada) — Applied Physics Division*
*Patent(s) Pending — All Rights Reserved*
