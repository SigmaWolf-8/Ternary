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
| Version | 1.0 |
| Status | Final |
| Classification | Patent(s) Pending — All Rights Reserved |
| Repository | SigmaWolf-8/Ternary |
| Source Files | `shared/plenum-square.ts`, `shared/constants.ts` (`PLATFORM.PLENUM_SQUARE`) |
| Dependencies | INVARIANT 4 (Constants Are Bound), `shared/tribonacci-constants.ts` |
| Supersedes | Saturnian Magic Square (§1.5 in prior repo guides) |

---

## 0. Abstract

This manifest formalizes the **Plenum Square** — a family of five 3×3 magic squares derived from the 364° ternary circle with π = 14 and diameter 111. The original circulant {111, 14, 208} is the generative root of the Salvi Framework. Two additional circle-derived constants (26 = 364/π, 196 = π²) form opposite pairs summing to 222 = 2 × 111, which embed in exactly four distinct non-circulant configurations (A–D) exhaustive up to D₄ dihedral symmetry. All configurations share a harmonic ladder of invariant sums (222, 333, 444, 555, 888, 999) and product identities (14 × 26 = 364, 14² = 196). Base-3 analysis reveals hidden Latin-square structure in the outermost ternary digits, invariant across configurations.

Implementation: `shared/plenum-square.ts` (family definitions, harmonic ladder, validation functions) and `PLATFORM.PLENUM_SQUARE` in `shared/constants.ts` (single source of truth for all numeric constants).

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

The inscribed regular hexagon's perimeter (333) becomes the magic constant — the first geometric bridge between circle and square.

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

All four configurations share invariant numerical relationships — the harmonic ladder:

| Property | Value | Relation | Role |
|----------|-------|----------|------|
| Opposite-pair sum | 222 | 2 × 111 | Defines pair placement |
| Magic constant | 333 | 3 × 111 | Hexagon perimeter → row/col/diag |
| Corner sum | 444 | 4 × 111 | All four corners, every config |
| Edge-center sum | 444 | 4 × 111 | Four non-corner middles |
| Circumference − total | 555 | 5 × 111 | Circle exceeds square |
| Surround sum | 888 | 8 × 111 | Eight non-center cells (999 − 111) |
| Total square sum | 999 | 9 × 111 | 3 × magic constant |

**Layered hierarchy:**
444 (corners) + 444 (edge-centers) = 888 (surround)
888 (surround) + 111 (center) = 999 (whole)

**Product invariants (present in every configuration):**
- 14 × 26 = 364 (circle degrees, as adjacent or aligned cells)
- 14² = 196 (embedded in middles or corners)

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

Also a perfect Latin square. The cyclic symbol permutation 0→2→1→0 transforms the d₄ grid exactly into the d₀ grid.

**Invariance:** Square B displays the identical Latin square pair in its outer ternary digits. This invariance across configurations confirms the property is a signature of the entire {14, 26, 111, 196, 208} family.

### 6.2 Middle Digit Orthogonality

The digit d₁ (3¹ place) pairs orthogonally with d₀: all nine ordered pairs (0-0 through 2-2) appear exactly once across the grid — a full orthogonal relation in the combinatorial sense.

Classic 3×3 magic squares decompose as 3 × L₁ + L₂ + 1, where L₁ and L₂ are orthogonal Latin squares. The circle parameters (with π = 14) naturally imprint this fundamental combinatorial skeleton onto the decimal grids.

### 6.3 Rarity

Computational spot-checks of other center-111 magic squares (random p, q yielding positive integers) rarely reproduce simultaneous Latin squares in both outermost ternary digits with a cyclic relation between them. The ratios and differences dictated by the circle parameters (222 − 14 = 208, 222 − 26 = 196) are uniquely tuned to produce this combinatorial skeleton.

---

## 7. Geometric Interpretation

| π-pair Position | 364/π-pair Position | Square | Interpretation |
|-----------------|---------------------|--------|----------------|
| Main diagonal (corners) | Horizontal midline | A | Diagonal crossed by horizontal |
| Vertical midline | Horizontal midline | B | Vertical × horizontal cross |
| Horizontal midline | Main diagonal | C | Horizontal crossed by diagonal |
| Horizontal midline | Vertical midline | D | Horizontal × vertical cross |

These four configurations exhaust the ways two perpendicular axes can be occupied by the two pairs, with positivity as the only constraint.

---

## 8. Synthesis

### 8.1 The Circle-Magic Square Unity

The Plenum Square construction demonstrates a remarkable harmony between:

- A 364° circle with π = 14 and diameter 111
- A 3×3 magic square with center 111 and constant 333
- The inscribed regular hexagon whose perimeter (333) becomes the magic constant

The key numbers emerge naturally: π = 14, 364/π = 26, π² = 196, 222 − 14 = 208, 222 − 26 = 196 (consistent).

### 8.2 Fourfold Completeness

Squares A–D represent the complete set of positive-integer embeddings. They share: opposite-pair sum 222, magic constant 333, corner and edge-center sums 444, surround sum 888, total sum 999, circumference residual 555. Products 14 × 26 = 364 and 14² = 196 appear in each.

### 8.3 Ternary Depth

Base-3 analysis reveals hidden combinatorial structure: most and least significant ternary digits form perfect Latin squares related by consistent cyclic permutation, invariant across configurations. Digit d₁ pairs orthogonally with d₀, producing all nine distinct ordered pairs. This connects the circle-derived numbers to the fundamental algebra of order-3 magic squares.

### 8.4 Final Unified Statement

The four magic squares A–D completely capture the arithmetic and geometric signature of a 364° circle (π = 14, diameter = 111) projected into a 3×3 magic square of center 111 and constant 333. The circle-derived values {14, 26, 196, 208} form opposite pairs (sum 222) that fit in exactly four distinct positive configurations. Invariant sums (222, 333, 444×2, 555, 888, 999) and products (364°, 196 = π²) hold universally. In base 3, Latin squares appear in the outermost digits, connected by a cyclic permutation invariant across all four configurations.

---

## 9. Implementation Status

| Artifact | Location | Status |
|----------|----------|--------|
| Plenum Square module | `shared/plenum-square.ts` | Complete |
| PLATFORM constants | `shared/constants.ts` (`PLENUM_SQUARE` block) | Complete |
| Utility functions | `shared/plenum-square-utils.ts` | Complete (renamed from `saturnian-matrix-utils.ts`) |
| Validation suite | `validateAll()` in `plenum-square.ts` | 94/94 invariants passing |
| Test suite | `tests/plenum-square.test.ts` | 38/38 tests passing |
| CI integration | `theory-validation.yml` Stage 1 | Planned |
| Rust module | `ternary-math/src/magic_square.rs` | Planned |
| Visualization page | `client/src/pages/plenum-square.tsx` | Planned |

The rename from "Saturnian Magic Square" to "Plenum Square" is complete. Old files (`saturnian-blueprint.ts`, `saturnian-matrix-utils.ts`, `saturnian-blueprint.test.ts`) have been deleted. Deprecated aliases have been removed.

---

## 10. Interpretive Note

*Circular continuity weds square discreteness. Decimal multiples of 111 meet ternary combinatorics. And the ancient quest to square the circle attains here one precise, self-resonant numerical incarnation.*

*The Plenum Square is not a different object from the ternary circle — it is the circle's projection into discrete arithmetic, carrying with it the full combinatorial depth of the base-3 system from which it was derived.*

---

*Copyright © 2025-2026 Capomastro Holdings Ltd. (Canada) — Applied Physics Division*
*Patent(s) Pending — All Rights Reserved*
