# TM-2026-035 — 364° Crystal: 2D→3D Derivation

**Capomastro Holdings Ltd — Applied Physics Division**  
**Version 1.0 — April 2026 — Patent Pending**  
**π = 14 · radian = 13 · full circle = 364°**  
**Source construction: TM-2026-017 v11.14**

---

## 1. Source: The 2D Inscribed Circle

All geometry derives from the 364° construction (TM-2026-017 v11.14). 13 regular polygons (n = 3 through n = 15) are inscribed in a single circle of radius R.

The vertex function `pa(deg)` converts a 364° angle to position:

```
θ = deg × 2π / 364

SVG:   sx = cx + R·sin(θ)      sy = cy − R·cos(θ)
Math:  mx = cos(θ)              my = sin(θ)
```

Key reference points on the unit circle:

| 364° angle | θ (radians) | Math coords (mx, my) | Identity |
|------------|-------------|----------------------|----------|
| 0° | 0 | (1, 0) | **S** — starting point |
| 91° | π/2 | (0, 1) | **C₁₈₂** — red arc control, 7×13, 7 radians |
| 143° | — | (−0.6235, 0.7818) | **C₆₅₀** — green arc control, 11×13 |
| 182° | π | (−1, 0) | **half-turn** (14×13 = 182), arc root |
| 218.4° | — | (−0.8090, −0.5878) | **P** — convergence vertex, −φ/2 |
| 273° | 3π/2 | (0, −1) | nadir |
| 364° | 2π | (1, 0) | **full circle = 0°** |

### Polygon Parameters (from PC object)

| n | Name | Step (364/n) | Offset | Color |
|---|------|-------------|--------|-------|
| 3 | triangle | 121.333° | 0 | #993C1D |
| 4 | square | 91.000° | 45.5 | #5F5E5A |
| 5 | pentagon | 72.800° | 0 | #BA7517 |
| 6 | hexagon | 60.667° | 364/12 = 30.333 | #3B6D11 |
| 7 | heptagon | 52.000° | 0 | #3D444B |
| 8 | octagon | 45.500° | 22.75 | #185FA5 |
| 9 | nonagon | 40.444° | 0 | #993556 |
| 10 | decagon | 36.400° | 0 | #0F6E56 |
| 11 | hendecagon | 33.091° | 0 | #D4537E |
| 12 | dodecagon | 30.333° | 364/24 = 15.167 | #378ADD |
| 13 | tridecagon | 28.000° | 0 | #D85A30 |
| 14 | tetradecagon | 26.000° | 13 | #EF9F27 |
| 15 | pentadecagon | 24.267° | 0 | #639922 |

Each polygon's k-th vertex is at 364° angle: `off + k × (364/n)`

### Superhub Zones (4-Edge Crossings)

The 2D construction produces 4 superhub zones where edges from 4 different polygons intersect simultaneously. These are derived intersection loci — not vertices of any single polygon:

| Zone | Math coords (mx, my) | Polygons | r ≈ |
|------|---------------------|----------|-----|
| A | (−0.9010, +0.3728) | 7 × 11 × 12 × 13 | 0.974 |
| B | (−0.9010, −0.3728) | 7 × 11 × 12 × 13 | 0.974 |
| C | (+0.7396, +0.6288) | 8 × 11 × 12 × 13 | 0.970 |
| D | (+0.7396, −0.6288) | 8 × 11 × 12 × 13 | 0.970 |

In 3D these survive as intersection loci on the circumsphere — points where edges of 4 different inscribed antiprisms cross on the sphere surface.

**3D projection status: UNRESOLVED.** The 2D superhubs are edge intersections with no natural latitude. Their 3D positions on the circumsphere are NOT yet computed. The claim that they "survive" requires solving actual great circle arc intersections for each pair of 3D antiprism edges. This is a computational geometry task — the algorithm is:

1. For each pair of 2D edges that cross at a superhub, identify the corresponding 3D edges (great circle arcs on the sphere)
2. Find the intersection of the two great circles on S²
3. Verify that the intersection point lies within both arc segments (not just on the extended great circles)
4. Confirm that all 4 edges converge to the same point (within numerical tolerance)

Until this is computed, the 3D superhub positions are **unknown** and no structural claims about them on the circumsphere should be relied upon.

### UV Spectral Reference Points

| 364° | λ (nm) | Band | Physics |
|------|--------|------|---------|
| 91 | 91 | **EUV** | Lyman limit — H ionization threshold |
| 182 | 182 | UV-C | O₂ absorption wall |
| 286 | 286 | UV-B | O₃ bridge — Mg II doublet |
| 364 | 364 | UV-A | Full window — atmospheric transmission |

Integers exact from axiom. Vacuum-frame measurements differ by factor 1.00194: +0.139% R∞ bias (UNIVERSAL_BIAS) + 0.055% reduced-mass (VACUUM_BIAS) = 0.194% total.

---

## 2. The 3D Extension: Circle → Sphere

The 2D inscribed circle becomes a circumscribing sphere. Every vertex must lie on the sphere surface.

### Coordinate Mapping

For a vertex at 364° angle `deg`, lifted to latitude ±α on the unit sphere:

```
θ = deg × 2π / 364

X = sin(θ) · cos(α)      [= my · cos(α)]
Y = sin(α)               [height on sphere]
Z = −cos(θ) · cos(α)     [= −mx · cos(α)]
```

**Proof that every vertex is on the unit sphere:**

```
|V|² = sin²(θ)·cos²(α) + sin²(α) + cos²(θ)·cos²(α)
     = cos²(α)·[sin²(θ) + cos²(θ)] + sin²(α)
     = cos²(α) + sin²(α)
     = 1  ✓
```

### Viewing Correspondence

When viewed from directly above (looking down the Y axis), the 3D polygon projects to:

```
X = sin(θ)·cos(α)    ∝  my  (east in 2D)
Z = −cos(θ)·cos(α)   ∝ −mx  (south in 2D)
```

This recovers the 2D inscribed circle layout, scaled by cos(α).

---

## 3. Latitude Derivation: α from the Step Angle

Each polygon n has step angle `364/n` custom degrees. The half-step is `182/n` custom degrees.

### The Key Simplification

```
α_custom(n) = 182/n          (latitude in custom degrees)
α_std(n)    = (182/n)×(90/91) (latitude in standard degrees)

Since 182/91 = 2 exactly:
  α_std(n) = 180/n  standard degrees

ALL 3D coordinate calculations (cos, sin) use α_std = 180/n.
The custom-degree latitude α_custom = 182/n is the half-step itself.
```

**Why this works:** Converting custom → standard multiplies by 90/91. Since 182 = 2 × 91, the factor 182/91 cancels to 2, giving 2 × 90/n = 180/n. The entire antiprism latitude system reduces to **α_std = 180/n**.

**Why 182?** 182 is a root of the arc equation `arc² − 832·arc + 118,300 = 0`. Also 14 × 13 = π_custom × radian = the squared circle area.

**These two numbers — 182 and 91 — reappear as topological invariants (§12):**

```
Coprime pair (13, 14): lcm = 13×14 = 182 (the arc root as walk length)
Torus knot T(14, 15):  Seifert genus = 13×14/2 = 182/2 = 91

The arc root generates the walk. The quarter turn is its Seifert genus.
The numbers that define the latitude formula ARE the crystal's topology.
```

**The re-priming (TM-2026-034):** The latitude formula α = (182/n) × (90/91) works because 182 = 2 × 7 × 13 replaces the standard half-turn 180 = 2² × 3² × 5. This is not a rescaling — it is an algebraic **re-priming** that threads the golden ratio through {7, 13} instead of {3, 5}. The full analysis is in §11a.

### Computed Latitudes (all α_std in standard degrees for 3D use)

| n | Step (custom°) | Half-step (custom°) | α_std = 180/n | α_custom = 182/n | Integer α_std? |
|---|---------------|--------------------|--------------|--------------------|----------------|
| 3 | 121.333° | 60.667° | **60.000°** | 60.667° | yes |
| 4 | 91.000° | 45.500° | **45.000°** | 45.500° | yes |
| 5 | 72.800° | 36.400° | **36.000°** | 36.400° | yes |
| 6 | 60.667° | 30.333° | **30.000°** | 30.333° | yes |
| 7 | 52.000° | 26.000° | **25.714°** = 180/7 | 26.000° | no |
| 8 | 45.500° | 22.750° | **22.500°** | 22.750° | yes |
| 9 | 40.444° | 20.222° | **20.000°** | 20.222° | yes |
| 10 | 36.400° | 18.200° | **18.000°** | 18.200° | yes |
| 11 | 33.091° | 16.545° | **16.364°** = 180/11 | 16.545° | no (repeating) |
| 12 | 30.333° | 15.167° | **15.000°** | 15.167° | yes |
| 13 | 28.000° | 14.000° | **13.846°** = 180/13 | 14.000° | no |
| 14 | 26.000° | 13.000° | **12.857°** = 90/7 | 13.000° | no |
| 15 | 24.267° | 12.133° | **12.000°** | 12.133° | yes |

**Observation:** Exact integer latitudes occur at n = 3, 4, 5, 6, 8, 9, 10, 12, 15. These are precisely the divisors where 182/n × 90 is divisible by 91. This is not coincidence — it follows from the factorization of 182 = 2 × 7 × 13 and 91 = 7 × 13.

**Coprime polygon latitudes:** The coprime walk polygons (n = 7, 11, 13, 14) produce non-integer latitudes that are rational multiples of fundamental angles:

```
n=7:   α = 180/7  = 25.714°     (heptagon — coprime walk primary)
n=13:  α = 180/13 = 13.846°     (tridecagon — radian polygon)
n=14:  α = 90/7   = 12.857°     (tetradecagon — π-gon, 14 = π_custom)
n=11:  α = 180/11 = 16.364°     (hendecagon — repeating decimal, not a simple
                                  fraction of 180° like n=7,13,14)
```

These are the latitudes that matter most for the coprime walk on the circumsphere. The Brieskorn solid Σ(7,11,13) uses three of them.

---

## 4. The Antiprism (Prismatoid) Construction

For each n-gon, the 3D solid is an **n-gonal antiprism** — a prismatoid with all 2n vertices in exactly two parallel planes.

### Structure

- **Top face:** n-gon at latitude **+α**, vertices at 364° angles `off + k × step`
- **Bottom face:** n-gon at latitude **−α**, rotated by half-step: `off + step/2 + k × step`
- **Belt:** 2n triangles connecting the two rings
- **Total:** 2 n-gonal faces + 2n triangular faces = 2n + 2 faces, 2n vertices

### Why Half-Step Rotation?

The bottom ring is rotated by `step/2 = 182/n` degrees. This is derived, not chosen:

1. **182 is a root of the arc equation** — the twist comes from the generating system
2. **The construction's offsets encode half-steps:** square has off = 91/2 = 45.5° (half of its own step), octagon has off = 45.5/2 = 22.75°, dodecagon has off = 364/24 = 15.167° — these are half-steps of related polygons
3. **Geometric necessity:** because the twist means no top edge aligns with any bottom edge, only triangles (not rectangles) can close the belt — you get n downward-pointing + n upward-pointing = 2n triangles

### Belt Triangle Winding

Because bottom vertex B[i] sits angularly **between** top vertices T[i] and T[i+1]:

```
Downward triangle:  T[i], T[i+1], B[i]
Upward triangle:    T[i+1], B[i+1], B[i]
```

This produces 2n triangles with consistent winding.

---

## 5. Worked Example: Triangle Antiprism (n = 3)

### Parameters

```
n = 3,  off = 0,  step = 364/3 = 121.333°
half_step = 182/3 = 60.667°
α = (182/3) × (90/91) = 5460/91 = 60.000° exactly

cos(60°) = 0.5000    sin(60°) = 0.8660
```

### Top Ring (latitude +60°)

| Vertex | 364° deg | θ (rad) | mx=cos(θ) | my=sin(θ) | X | Y | Z | 364° Identity |
|--------|----------|---------|-----------|-----------|------|------|------|---------------|
| T₀ | 0.00 | 0.0000 | 1.0000 | 0.0000 | 0.0000 | 0.8660 | −0.5000 | **S point** |
| T₁ | 121.33 | 2.0944 | −0.5000 | 0.8660 | 0.4330 | 0.8660 | 0.2500 | |
| T₂ | 242.67 | 4.1888 | −0.5000 | −0.8660 | −0.4330 | 0.8660 | 0.2500 | |

### Bottom Ring (latitude −60°, rotated by 60.667°)

| Vertex | 364° deg | θ (rad) | mx=cos(θ) | my=sin(θ) | X | Y | Z | 364° Identity |
|--------|----------|---------|-----------|-----------|------|------|------|---------------|
| B₀ | 60.67 | 1.0472 | 0.5000 | 0.8660 | 0.4330 | −0.8660 | −0.2500 | |
| B₁ | **182.00** | 3.1416 | −1.0000 | 0.0000 | 0.0000 | −0.8660 | 0.5000 | **182° half-turn** |
| B₂ | 303.33 | 5.2360 | 0.5000 | −0.8660 | −0.4330 | −0.8660 | −0.2500 | |

### Verification

- All |V|² = 1.0 ✓
- 6 vertices, 8 equilateral triangle faces = **octahedron** ✓
- T₀ sits directly above the S point (1, 0) at 0° ✓
- B₁ sits directly below the 182° point (−1, 0) = π ✓

---

## 6. Worked Example: Square Antiprism (n = 4)

### Parameters

```
n = 4,  off = 45.5,  step = 364/4 = 91°
half_step = 182/4 = 45.5°
α = (182/4) × (90/91) = 4095/91 = 45.000° exactly

cos(45°) = 0.7071    sin(45°) = 0.7071
```

### Top Ring (latitude +45°)

| Vertex | 364° deg | θ (rad) | mx | my | X | Y | Z | 364° Identity |
|--------|----------|---------|------|------|------|------|------|---------------|
| T₀ | 45.50 | 0.7854 | 0.7071 | 0.7071 | 0.5000 | 0.7071 | −0.5000 | |
| T₁ | 136.50 | 2.3562 | −0.7071 | 0.7071 | 0.5000 | 0.7071 | 0.5000 | |
| T₂ | 227.50 | 3.9270 | −0.7071 | −0.7071 | −0.5000 | 0.7071 | 0.5000 | |
| T₃ | 318.50 | 5.4978 | 0.7071 | −0.7071 | −0.5000 | 0.7071 | −0.5000 | |

### Bottom Ring (latitude −45°, rotated by 45.5°)

| Vertex | 364° deg | θ (rad) | mx | my | X | Y | Z | 364° Identity |
|--------|----------|---------|------|------|------|------|------|---------------|
| B₀ | **91.00** | π/2 | 0.0000 | 1.0000 | 0.7071 | −0.7071 | 0.0000 | **C₁₈₂ (0,1)** |
| B₁ | **182.00** | π | −1.0000 | 0.0000 | 0.0000 | −0.7071 | 0.7071 | **182° (−1,0)** |
| B₂ | **273.00** | 3π/2 | 0.0000 | −1.0000 | −0.7071 | −0.7071 | 0.0000 | **(0,−1)** |
| B₃ | **364.00** | 2π = 0 | 1.0000 | 0.0000 | 0.0000 | −0.7071 | −0.7071 | **S (1,0)** |

### Verification

- All |V|² = 1.0 ✓
- 8 vertices, 10 faces (2 squares + 8 triangles) ✓
- Bottom ring lands on the **four cardinal points** of the 364° system: 91°, 182°, 273°, 0° ✓
- B₀ = C₁₈₂ control point, B₁ = 182° half-turn, B₃ = S starting point ✓

### Cross-Validation

The square's offset is 45.5° = step/2. The bottom ring's offset is off + step/2 = 45.5 + 45.5 = **91° = quarter turn**. The bottom square is the 364° square rotated by exactly one quarter turn — this is the square's own half-step, and it produces the cardinal alignment. This is not coincidence; it's the self-consistency of the 364° system.

**Bézier arc anchor connection:** B₀ at 91° is the **C₁₈₂ control point** — the same point that anchors the red Bézier arc in the 2D construction. B₁ at 182° is the **182° half-turn** — the nadir of the arc equation. The 2D construction's key structural nodes are literally vertices of the 3D square antiprism. The 2D arc geometry is embedded in the 3D crystal.

---

## 6a. Worked Example: Pentagon Antiprism (n = 5) — Geometric Polygon

### Parameters

```
n = 5,  off = 0,  step = 364/5 = 72.8°
half_step = 182/5 = 36.4°
α = (182/5) × (90/91) = 3276/91 = 36.000° exactly

cos(36°) = 0.8090    sin(36°) = 0.5878
```

### Top Ring (latitude +36°)

| Vertex | 364° deg | mx | my | X | Y | Z | 364° Identity |
|--------|----------|------|------|--------|--------|--------|---------------|
| T₀ | 0.00 | 1.0000 | 0.0000 | 0.0000 | 0.5878 | −0.8090 | **S point** |
| T₁ | 72.80 | 0.3090 | 0.9511 | 0.7694 | 0.5878 | −0.2500 | |
| T₂ | 145.60 | −0.8090 | 0.5878 | 0.4755 | 0.5878 | 0.6545 | |
| T₃ | **218.40** | −0.8090 | −0.5878 | −0.4755 | 0.5878 | 0.6545 | **P vertex (−φ/2)** |
| T₄ | 291.20 | 0.3090 | −0.9511 | −0.7694 | 0.5878 | −0.2500 | |

### Bottom Ring (latitude −36°, rotated by 36.4°)

| Vertex | 364° deg | mx | my | X | Y | Z | 364° Identity |
|--------|----------|------|------|--------|--------|--------|---------------|
| B₀ | 36.40 | 0.8090 | 0.5878 | 0.4755 | −0.5878 | −0.6545 | |
| B₁ | 109.20 | −0.3090 | 0.9511 | 0.7694 | −0.5878 | 0.2500 | |
| B₂ | **182.00** | −1.0000 | 0.0000 | 0.0000 | −0.5878 | 0.8090 | **182° half-turn** |
| B₃ | 254.80 | −0.3090 | −0.9511 | −0.7694 | −0.5878 | 0.2500 | |
| B₄ | 327.60 | 0.8090 | −0.5878 | −0.4755 | −0.5878 | −0.6545 | |

### Verification

- All |V|² = 1.0 ✓ (e.g. T₀: 0 + 0.3455 + 0.6545 = 1.0)
- 10 vertices, 12 faces (2 pentagons + 10 triangles) ✓
- **T₃ at 218.4° = P convergence vertex** — mx = −0.8090 = −φ/2 ✓
- The golden ratio point P, where the Bézier arcs converge in 2D, is a vertex of the pentagon antiprism in 3D
- B₂ at 182° = half-turn ✓

### Golden Ratio Connection

```
T₃ sits above the P vertex: pa(218.4°) = P = (−φ/2, −√(10−2√5)/4)

In the 2D construction, P is the convergence point of both Bézier arcs.
In 3D, it's a vertex of the pentagon antiprism at latitude +36°.

φ/2 = 0.8090 = cos(36°) = cos(α₅)

The golden ratio's half IS the pentagon antiprism's latitude cosine.

Further φ structure in the pentagon antiprism:
  sin(36°) = 0.5878 = √(10−2√5)/4   → the P vertex's y-component
  sin(72°) = 0.9511 = (√(10+2√5))/4 → appears as T₁,T₄ my-component
  cos(72°) = 0.3090 = 1/(2φ)         → appears as T₁,T₄ mx-component

  Belt triangle edge ratios:
    Top edge / belt edge = 2sin(π/5) / 2sin(π/10) = φ  (golden ratio)
    The antiprism's belt triangles have sides in golden proportion.

The pentagon antiprism is saturated with φ at every level:
latitude, vertex coordinates, and edge ratios.
```

---

## 6b. Worked Example: Heptagon Antiprism (n = 7) — Arithmetic/Coprime Polygon

### Parameters

```
n = 7,  off = 0,  step = 364/7 = 52°
half_step = 182/7 = 26°
α = (182/7) × (90/91) = 2340/91 = 180/7 = 25.7143°

cos(180/7) = 0.9009    sin(180/7) = 0.4339
```

This is the first **coprime walk polygon** — n=7 doesn't appear in any Archimedean solid. Its role is purely arithmetic: as a member of the Brieskorn triple (7, 11, 13).

### Top Ring (latitude +25.714°)

| Vertex | 364° deg | mx | my | X | Y | Z | 364° Identity |
|--------|----------|------|------|--------|--------|--------|---------------|
| T₀ | 0 | 1.0000 | 0.0000 | 0.0000 | 0.4339 | −0.9009 | **S point** |
| T₁ | 52 | 0.6235 | 0.7818 | 0.7043 | 0.4339 | −0.5618 | |
| T₂ | 104 | −0.2225 | 0.9749 | 0.8783 | 0.4339 | 0.2004 | |
| T₃ | 156 | −0.9010 | 0.4339 | 0.3909 | 0.4339 | 0.8114 | ≈ **superhub A mx** |
| T₄ | 208 | −0.9010 | −0.4339 | −0.3909 | 0.4339 | 0.8114 | ≈ **superhub B mx** |
| T₅ | 260 | −0.2225 | −0.9749 | −0.8783 | 0.4339 | 0.2004 | |
| T₆ | 312 | 0.6235 | −0.7818 | −0.7043 | 0.4339 | −0.5618 | |

### Bottom Ring (latitude −25.714°, rotated by 26°)

| Vertex | 364° deg | mx | my | X | Y | Z | 364° Identity |
|--------|----------|------|------|--------|--------|--------|---------------|
| B₀ | 26 | 0.8987 | 0.4384 | 0.3949 | −0.4339 | −0.8094 | |
| B₁ | 78 | 0.1045 | 0.9945 | 0.8959 | −0.4339 | −0.0942 | |
| B₂ | 130 | −0.6691 | 0.7431 | 0.6694 | −0.4339 | 0.6028 | |
| B₃ | **182** | −1.0000 | 0.0000 | 0.0000 | −0.4339 | 0.9009 | **182° half-turn** |
| B₄ | 234 | −0.6691 | −0.7431 | −0.6694 | −0.4339 | 0.6028 | |
| B₅ | **286** | 0.1045 | −0.9945 | −0.8959 | −0.4339 | −0.0942 | **UV-B/A boundary** |
| B₆ | 338 | 0.8987 | −0.4384 | −0.3949 | −0.4339 | −0.8094 | |

### Verification

- All |V|² = 1.0 ✓ (e.g. T₃: 0.1528 + 0.1883 + 0.6589 = 1.0)
- 14 vertices, 16 faces (2 heptagons + 14 triangles) ✓
- T₃ at 156° has mx = −0.9010 — this is the **superhub A/B x-coordinate** from the 2D construction
- B₃ at 182° = half-turn ✓
- B₅ at 286° = UV-B/UV-A spectral boundary (O₃ bridge wavelength)
- **286° torus knot connection:** the (7,11) torus knot has twist angle 11 × 26° = 286° in 364° measure (where 26° = heptagon half-step). The heptagon antiprism's bottom vertex B₅ and the T(7,11) knot twist both land on 286°. The spectral correspondence (O₃ bridge at 286 nm) connects to the knot twist, not just the vertex position. See §12.

### Superhub Connection

```
2D superhub A: math coords (−0.9010, +0.3728) — polygons 7×11×12×13
Heptagon T₃:   math coords (−0.9010, +0.4339)

The mx components match exactly (−0.9010). The my components differ
(+0.3728 vs +0.4339) because the superhub is a 4-edge intersection,
not a vertex of any single polygon.

The shared mx = −0.9010 means superhub A lies on the same RADIAL LINE
from center as the arc between T₃ (156°) and T₄ (208°). However, 
the superhub is the intersection of edges from 4 different polygons
(7×11×12×13) — it is NOT a point on the heptagon edge alone.
The mx alignment is structural (the heptagon's angular position
determines the radial line) but the actual superhub position
requires solving the 4-edge intersection.
```

---

## 6c. The Information-Theoretic Crystal

**Status: Conceptual framework — not yet operational.** The definitions below map the Buried Question's entropy decomposition onto the crystal geometry. Precise computation of cone points requires the full set of edge-edge intersections on the sphere, which is deferred to implementation. This section is included for completeness but is not used by later sections.

The crystal geometry has a direct information-theoretic interpretation through the entropy decomposition.

### Entropy Decomposition on the Sphere

The coprime walk on the circumsphere decomposes into three components:

```
Total walk = Current + Stillness + Rock

Current:    Non-identity walk steps — movement between lattice positions
            → Great circle arcs connecting antiprism vertices
            → The visible edges and paths on the sphere
            → Trits with value 1 or 2 in Rep C

Stillness:  Identity walk steps — the walk visits a vertex but does NOT advance
            → Trits with value 0 (the walk stays put)
            → In 3D: the walk occupies a vertex with non-trivial stabilizer
            → Correlates with high-intersection vertices (superhubs)
              but is NOT identical to them — stillness is a walk state,
              superhubs are geometric loci. They overlap at cone points
              where the stabilizer group is non-trivial.

Rock:       The geometric floor — minimum structure from cone-point density
            → |C|/C where C is the set of cone points (vertices with
              non-trivial stabilizer under the walk's group action)
            → Sets the irreducible vertex density per colatitude band
            → Independent of the specific walk — it's the lattice itself
```

### Cone Points on the Circumsphere

In the 2D construction, edge intersections produce ~580 intersection points (hotspots) plus 4 superhubs. On the circumsphere, these become the **cone points** — positions where the spherical surface has concentrated curvature from the lattice structure.

```
Cone point density = intersections per unit solid angle

Polar cap:     low density (few polygons, few crossings)
Temperate:     medium density
Equatorial:    high density (9 polygons, maximum crossings)

The "rock" (geometric floor) varies with colatitude:
  Rock(colat) = |C(colat)| / C_total
```

The crystal's information content is not uniform across its surface — it concentrates at the equatorial belt where the coprime walk polygons (n=7 through 15) cluster within 18° of colatitude.

---

## 7. The Circumsphere

The circumsphere is the 3D analog of the 2D inscribed circle. Every vertex of every solid lies on its surface.

### Standard Geometry (π_std ≈ 3.14159)

For circumsphere radius R:

```
Surface area  = 4π_std · R²
Volume        = (4/3)π_std · R³
Circumference = 2π_std · R    (any great circle)
```

### 364° Framework (π_custom = 14)

```
Circumference of great circle = 2 × 14 × R = 28R
Full revolution = 28 × 13 = 364° ✓

Surface area  = 4 × 14 × R² = 56R²
Volume        = (4/3) × 14 × R³ = 56R³/3
```

For the unit sphere (R = 1):

| Property | Standard (π_std) | Salvi Framework (π=14) | Ratio |
|----------|-----------------|----------------------|-------|
| Circumference | 6.2832 | 28 | 4.4563 |
| Surface area | 12.5664 | 56 | 4.4563 |
| Volume | 4.1888 | 18.6667 | 4.4563 |

The ratio is constant: 14/π_std = 4.4563... This is the **bridge factor** (noted in your construction: π_std/14 ≈ 0.2244).

**Bridge factor and the Archimedes approximation:**

```
π_std ≈ 22/7  (Archimedes)
91 × 22/7 = 286  (the UV-B wavelength, the O₃ bridge)

So: 286/91 = 22/7 ≈ π_std
And: 14/(22/7) = 14 × 7/22 = 98/22 = 49/11 = 4.4545...

The bridge factor 14/π_std ≈ 14/(22/7) = 49/11 to first order.
The residual (4.4563 − 4.4545 = 0.0018) is the correction
beyond Archimedes. A closed form in the framework is not known,
but the first-order approximation 49/11 connects the bridge
factor to the hendecagon (n=11) — the median coprime polygon.
```

### Squared Circle Connection

```
Squared circle area: π_custom · r² = 14 × 13 = 182
Side of squared circle: s = √182 = √(14 × 13)
```

The number 182 is simultaneously:
- A root of the arc equation `arc² − 832·arc + 118,300 = 0`
- The squared circle area (14 × 13)
- The generator of every antiprism half-step: `half_step(n) = 182/n`

The same number that squares the circle in 2D generates the antiprism latitudes in 3D. This is the deepest structural connection between the 2D and 3D constructions.

**Disdyakis confirmation (TM-2026-034):** The most complex Archimedean solid — the truncated icosidodecahedron (120 vertices, 62 faces) — has circumradius squared R² = 14 + 5φ exactly. The number 14 = π_custom is baked into the geometry of this solid as an exact algebraic fact, not an approximation. See §11a.

### Euler Characteristic

For any convex polyhedron inscribed in the sphere:

```
V − E + F = 2   (Euler's formula)
```

Verified for each solid below.

### Sphere as Refractive Boundary

Physically, the sphere is the **crystal-air interface** — the only surface where Snell's law applies for entry/exit. Internal faces produce lattice deflection, not air-crystal refraction.

```
At sphere entry:   n₁ sin(θ₁) = n₂ sin(θ₂)     [air → crystal]
At internal face:  lattice deflection (not Snell)
At sphere exit:    n₂ sin(θ₂) = n₁ sin(θ₁)     [crystal → air]

Normal at any sphere point = radial vector from origin (trivially computed)
```

---

## 8. The 13 Antiprisms — Vertex Configuration & Topology

Each n-gonal antiprism is a prismatoid. All vertices lie in exactly two parallel planes at latitudes ±α(n).

### Antiprism Topology Table

| n | Name | Vertex Config | n-gon faces | Tri faces | Total F | E | V | χ |
|---|------|--------------|-------------|-----------|---------|---|---|---|
| 3 | triangle | 3.3.3.3 | 2 | 6 | 8 | 12 | 6 | **2** ✓ |
| 4 | square | 3.3.3.4 | 2 | 8 | 10 | 16 | 8 | **2** ✓ |
| 5 | pentagon | 3.3.3.5 | 2 | 10 | 12 | 20 | 10 | **2** ✓ |
| 6 | hexagon | 3.3.3.6 | 2 | 12 | 14 | 24 | 12 | **2** ✓ |
| 7 | heptagon | 3.3.3.7 | 2 | 14 | 16 | 28 | 14 | **2** ✓ |
| 8 | octagon | 3.3.3.8 | 2 | 16 | 18 | 32 | 16 | **2** ✓ |
| 9 | nonagon | 3.3.3.9 | 2 | 18 | 20 | 36 | 18 | **2** ✓ |
| 10 | decagon | 3.3.3.10 | 2 | 20 | 22 | 40 | 20 | **2** ✓ |
| 11 | hendecagon | 3.3.3.11 | 2 | 22 | 24 | 44 | 22 | **2** ✓ |
| 12 | dodecagon | 3.3.3.12 | 2 | 24 | 26 | 48 | 24 | **2** ✓ |
| 13 | tridecagon | 3.3.3.13 | 2 | 26 | 28 | 52 | 26 | **2** ✓ |
| 14 | tetradecagon | 3.3.3.14 | 2 | 28 | 30 | 56 | 28 | **2** ✓ |
| 15 | pentadecagon | 3.3.3.15 | 2 | 30 | 32 | 60 | 30 | **2** ✓ |

**General formula for n-gonal antiprism:**

```
Vertex configuration: 3.3.3.n
Faces:    2n + 2      (2 n-gons + 2n triangles)
Edges:    4n          (n top + n bottom + 2n belt)
Vertices: 2n          (n top + n bottom)
Euler:    (2n+2) − 4n + 2n = 2  ✓
```

**Reading the vertex configuration:** At every vertex, the surrounding faces are: triangle, triangle, triangle, n-gon. That is, three belt triangles and one cap polygon meet at each vertex. This is uniform — the same configuration at every vertex — making each antiprism a **uniform polyhedron**.

### Cross-Reference: n=3 Antiprism = Octahedron

The triangular antiprism has vertex config 3.3.3.3 (four triangles at each vertex), 8 triangular faces, 12 edges, 6 vertices. This is exactly the regular **octahedron**. The two "triangular cap" faces are indistinguishable from the six belt triangles — all 8 faces are equilateral. This cross-validates with the Radian Octahedron in the named solids.

### Antiprism Totals

```
Total antiprism vertices:   2 × (3+4+5+...+15) = 2 × 117 = 234
Total antiprism faces:      Σ(2n+2) for n=3..15 = 2×117 + 26 = 260
Total antiprism edges:      Σ(4n) for n=3..15 = 4 × 117 = 468
All vertices on circumsphere: 234 points
```

---

## 9. The 6 Named Solids — Vertex Configuration & Topology

All inscribed in the **same circumsphere** as the 13 antiprisms.

### Named Solids Topology Table

| # | Name | Vertex Config | Face Breakdown | F | E | V | χ | Source |
|---|------|--------------|----------------|---|---|---|---|--------|
| 1 | Salvi Tetrahedron | 3.3.3 | 4 triangles | 4 | 6 | 4 | **2** ✓ | n=3, off=0° |
| 2 | Plenum Cube | 4.4.4 | 6 squares | 6 | 12 | 8 | **2** ✓ | n=4, off=45.5° |
| 3 | Radian Octahedron | 3.3.3.3 | 8 triangles | 8 | 12 | 6 | **2** ✓ | n=4 eq + poles |
| 4 | Golden Icosahedron | 3.3.3.3.3 | 20 triangles | 20 | 30 | 12 | **2** ✓ | n=5, off=0°, φ |
| 5 | Golden Dodecahedron | 5.5.5 | 12 pentagons | 12 | 30 | 20 | **2** ✓ | n=5 dual, φ |
| 6 | Brieskorn Σ(7,11,13) | mixed | 7-cap + belts + 13-cap | 62 | 93 | 33 | **2** ✓ | n=7,11,13 |

### Latitude Derivations

**1. Salvi Tetrahedron (3.3.3)**

```
Source: n=3 polygon at off=0°
Inscribed tetrahedron in unit sphere:
  Base latitude:  −asin(1/3) = −19.471°
  Apex latitude:  +90° (north pole)
  Base ring radius: cos(19.471°) = √(8/9) = 0.9428
  
Base vertices use pa(0°), pa(121.33°), pa(242.67°)
Apex at (0, R, 0)

Circumradius = R (all 4 vertices on sphere) ✓
Edge length = R × √(8/3) = R × 1.6330
```

**2. Plenum Cube (4.4.4)**

```
Source: n=4 polygon at off=45.5°
Inscribed cube in unit sphere:
  Ring latitudes:  ±asin(1/√3) = ±35.264°
  Ring radius: cos(35.264°) = √(2/3) = 0.8165
  
Top square: pa(45.5°), pa(136.5°), pa(227.5°), pa(318.5°)
Bottom square: same angles, latitude negated

Circumradius = R (all 8 vertices on sphere) ✓
Edge length = R × 2/√3 = R × 1.1547
Note: 91° step = 7×13 = the squared circle number
```

**3. Radian Octahedron (3.3.3.3)**

```
Source: n=4 polygon at off=45.5° (equator) + sphere poles
  Equatorial vertices at latitude 0°: pa(45.5°), pa(136.5°), pa(227.5°), pa(318.5°)
  Polar vertices at ±90°: (0, ±R, 0)

Circumradius = R (all 6 vertices on sphere) ✓
Edge length = R × √2 = R × 1.4142

Cross-validation: matches n=3 antiprism (also 3.3.3.3, also 8 faces, 12 edges, 6 vertices)
```

**4. Golden Icosahedron (3.3.3.3.3)**

```
Source: n=5 polygon at off=0°, golden ratio φ = (1+√5)/2
  Upper ring latitude:  +atan(1/2) = +26.565°
  Lower ring latitude:  −26.565°, rotated by F/10 = 36.4° (half pentagon step)
  Upper ring radius: cos(26.565°) = 2/√5 = 0.8944
  Poles at ±90°

Upper 5: pa(0°), pa(72.8°), pa(145.6°), pa(218.4°), pa(291.2°)
Lower 5: pa(36.4°), pa(109.2°), pa(182°), pa(254.8°), pa(327.6°)
Note: pa(218.4°) = P (convergence vertex, −φ/2)
Note: pa(182°) = half-turn point (14×13 = 182)

Circumradius = R ✓
Edge length = R × 2/√(1+φ²) = R × 1.0515
```

**5. Golden Dodecahedron (5.5.5)**

```
Source: dual of icosahedron, governed by n=5 polygon and golden ratio φ

Vertex coordinates in ℤ[φ]: all even permutations of
  (±1, ±1, ±1)           — 8 "cube" vertices
  (0, ±1/φ, ±φ)          — 12 "rectangle" vertices
  (±1/φ, ±φ, 0)
  (±φ, 0, ±1/φ)

Total: 20 vertices. All lie on sphere of radius √3.

Circumradius = √3  (exact)
  Proof: (1)² + (1)² + (1)² = 3 ✓
         (0)² + (1/φ)² + φ² = 0 + (2−φ) + (φ+1) = 3 ✓
  All 20 vertices verified at R² = 3.000000 (deviation < 10⁻¹⁵)

To inscribe in crystal circumsphere of radius SR:
  Scale factor = SR/√3
  All coordinates × SR/√3

Edge length = 2/(φ×√3) × SR = SR × 0.7136

Latitude structure (with 5-fold axis along Y):
  NOT simple pentagonal rings — the 20 vertices distribute across
  12 distinct latitude bands (±79.2°, ±52.6°, ±37.4°, ±29.4°,
  ±17.7°, ±10.8°) with 1–2 vertices per band.

  However, the AZIMUTHAL angles align with n=5 pa364() values:
    Vertices at lon = 72.8°, 145.6°, 218.4°, 291.2°, 182.0°, 0°
    These are exact n=5 polygon angles from the 364° construction.

  Notable vertex-landmark alignments:
    lon = 218.4° = P (convergence vertex, −φ/2) at lat ≈ +10.8°
    lon = 182.0° = half-turn at lat ≈ +79.2° and +37.4°
    lon = 0°     = S point at lat ≈ −37.4° and −79.2°

  The dodecahedron's complexity exceeds a simple antiprism:
  the icosahedron (its dual) has clean 5-vertex rings,
  but the dodecahedron distributes vertices more broadly.

12 regular pentagonal faces, 30 edges, 20 vertices
Euler: 20 − 30 + 12 = 2 ✓
```

**6. Brieskorn Σ(7,11,13)**

```
Source: coprime triple (7,11,13), lcm = 1,001
  Apex: (0, R, 0) at +90°
  Ring 7:  latitude +α₇  = +(182/7)×90/91  = +180/7  = +25.714°
  Ring 11: latitude +α₁₁ = +(182/11)×90/91 = +180/11 = +16.364°
  Ring 13: latitude −α₁₃ = −(182/13)×90/91 = −180/13 = −13.846°
  Nadir: (0, −R, 0) at −90°

  NOTE on latitude signs:
  The formula α_std = 180/n gives the MAGNITUDE of each ring's latitude.
  The SIGN (+ or −) is a design choice for the Brieskorn solid:
    Ring 7 at +25.714°, Ring 11 at +16.364°, Ring 13 at −13.846°
  
  This places two rings in the northern hemisphere and one in the south.
  The choice is NOT uniquely derived from the axioms — other sign patterns
  (e.g., all positive, or alternating) produce valid convex polyhedra with
  the same Euler characteristic. The specific pattern (+,+,−) was chosen to:
    (a) create a geometric descent: apex → 7 → 11 southward through belts
    (b) place ring 13 opposite to create a longer pavilion (nadir cap)
    (c) avoid a configuration where all three rings cluster too tightly
        (the alternative +25.7°, +16.4°, +13.8° spans only 11.9°)
  
  A stronger justification would derive the signs from self-intersection
  avoidance of the (7,11), (7,13), (11,13) torus knots on the sphere.
  This computation is deferred.

  Topology (single solid, apex-to-nadir triangulation):
    Top cap:     7 triangles  (apex → ring 7)
    Belt 7→11:   18 triangles (m+n = 7+11, standard ring stitching)
    Belt 11→13:  24 triangles (m+n = 11+13)
    Bottom cap:  13 triangles (ring 13 → nadir)
    Total:       62 triangular faces exactly
    Edges:       93  (from Euler: E = V + F − 2 = 33 + 62 − 2 = 93)
    Euler:       33 − 93 + 62 = 2 ✓
    
    Ring stitching formula: two rings of size m and n always
    produce exactly m+n belt triangles.

Ring 7:  pa(0°), pa(52°), pa(104°), pa(156°), pa(208°), pa(260°), pa(312°)
Ring 11: pa(0°), pa(33.09°), ..., pa(330.9°)
Ring 13: pa(0°), pa(28°), pa(56°), ..., pa(336°)

Vertices: 1 + 7 + 11 + 13 + 1 = 33
All on circumsphere ✓
Brieskorn sphere: link of x⁷ + y¹¹ + z¹³ = 0 in ℂ³
```

---

## 10. Archimedean Solids from the 364° Polygon Set

The 13 polygons (n = 3 through 15) provide all face types needed to construct several Archimedean solids. These are convex polyhedra with regular polygon faces of two or more types, and the same vertex configuration at every vertex.

### Buildable Archimedean Solids

| Name | Vertex Config | Face Breakdown | F | E | V | Polygons Used |
|------|--------------|----------------|---|---|---|---------------|
| Truncated Tetrahedron | 3.6.6 | 4 triangles + 4 hexagons | 8 | 18 | 12 | n=3, n=6 |
| Cuboctahedron | 3.4.3.4 | 8 triangles + 6 squares | 14 | 24 | 12 | n=3, n=4 |
| Truncated Cube | 3.8.8 | 8 triangles + 6 octagons | 14 | 36 | 24 | n=3, n=8 |
| Truncated Octahedron | 4.6.6 | 6 squares + 8 hexagons | 14 | 36 | 24 | n=4, n=6 |
| Rhombicuboctahedron | 3.4.4.4 | 8 tri + 18 squares | 26 | 48 | 24 | n=3, n=4 |
| Truncated Cuboctahedron | 4.6.8 | 12 sq + 8 hex + 6 oct | 26 | 72 | 48 | n=4, n=6, n=8 |
| Snub Cube | 3.3.3.3.4 | 32 triangles + 6 squares | 38 | 60 | 24 | n=3, n=4 |
| Icosidodecahedron | 3.5.3.5 | 20 tri + 12 pentagons | 32 | 60 | 30 | n=3, n=5 |
| Truncated Dodecahedron | 3.10.10 | 20 tri + 12 decagons | 32 | 90 | 60 | n=3, n=10 |
| Truncated Icosahedron | 5.6.6 | 12 pent + 20 hexagons | 32 | 90 | 60 | n=5, n=6 |
| Rhombicosidodecahedron | 3.4.5.4 | 20 tri + 30 sq + 12 pent | 62 | 120 | 60 | n=3, n=4, n=5 |
| Trunc. Icosidodecahedron | 4.6.10 | 30 sq + 20 hex + 12 dec | 62 | 180 | 120 | n=4, n=6, n=10 |
| Snub Dodecahedron | 3.3.3.3.5 | 80 tri + 12 pentagons | 92 | 150 | 60 | n=3, n=5 |

**Observation:** Every Archimedean solid uses only polygons with n ∈ {3, 4, 5, 6, 8, 10} — all of which are in the 364° construction. The construction's polygon set is a **complete basis** for the Archimedean solids.

**Two roles for the polygon set:** The 13 polygons serve two distinct functions:

```
Geometric polygons (n = 3, 4, 5, 6, 8, 10):
  → Archimedean/Catalan face types
  → Classical solid geometry
  → All produce exact integer latitudes

Arithmetic polygons (n = 7, 9, 11, 13, 14, 15):
  → Coprime walk generators
  → Torus knot parameters
  → The primes 7, 11, 13 define the Brieskorn sphere
  → n=14 = π_custom, n=15 = pentadecagon (lcm quadruple)
  → Non-integer latitudes (except n=9, 15)
```

The geometric polygons build the solid families. The arithmetic polygons define the walk topology. Both inhabit the same circumsphere — this is the dual nature of the crystal.

**Note on vertex configuration notation:** "3.6.6" means at every vertex, you encounter (going around) a triangle, then a hexagon, then a hexagon. This is uniform — the same arrangement at every vertex.

All Archimedean solids satisfy Euler's formula V − E + F = 2 and can be inscribed in a circumsphere.

---

## 11. Catalan Solids — Duals of the Archimedeans

The 13 Catalan solids are the **duals** of the 13 Archimedean solids. Where Archimedean solids are vertex-transitive (same vertex configuration everywhere), Catalan solids are **face-transitive** (every face is congruent). All Catalan solids can be inscribed in a circumsphere.

### Why They Matter Here

Catalan faces are **not regular polygons** — they are kites, rhombi, or irregular pentagons. These are **derived geometry** generated by the intersections of the regular polygon framework. The crystal contains not just regular polygons but shapes that emerge from the interplay between them.

### The 13 Catalan Solids

| # | Catalan Solid | Dual Of | Face Shape | F | E | V | Symmetry |
|---|--------------|---------|-----------|---|---|---|----------|
| 1 | Triakis Tetrahedron | Trunc. Tetrahedron | isosceles tri | 12 | 18 | 8 | Td |
| 2 | Rhombic Dodecahedron | Cuboctahedron | rhombus | 12 | 24 | 14 | Oh |
| 3 | Triakis Octahedron | Trunc. Cube | isosceles tri | 24 | 36 | 14 | Oh |
| 4 | Tetrakis Hexahedron | Trunc. Octahedron | isosceles tri | 24 | 36 | 14 | Oh |
| 5 | Deltoidal Icositetrahedron | Rhombicuboctahedron | kite | 24 | 48 | 26 | Oh |
| 6 | Disdyakis Dodecahedron | Trunc. Cuboctahedron | scalene tri | 48 | 72 | 26 | Oh |
| 7 | Pentagonal Icositetrahedron | Snub Cube | irr. pentagon | 24 | 60 | 38 | O |
| 8 | Rhombic Triacontahedron | Icosidodecahedron | rhombus | 30 | 60 | 32 | Ih |
| 9 | Triakis Icosahedron | Trunc. Dodecahedron | isosceles tri | 60 | 90 | 32 | Ih |
| 10 | Pentakis Dodecahedron | Trunc. Icosahedron | isosceles tri | 60 | 90 | 32 | Ih |
| 11 | Deltoidal Hexecontahedron | Rhombicosidodecahedron | kite | 60 | 120 | 62 | Ih |
| 12 | Disdyakis Triacontahedron | Trunc. Icosidodecahedron | scalene tri | 120 | 180 | 62 | Ih |
| 13 | Pentagonal Hexecontahedron | Snub Dodecahedron | irr. pentagon | 60 | 150 | 92 | I |

All satisfy Euler: V − E + F = 2 ✓

### Duality on the Circumsphere

```
Duality exchanges:  vertices ↔ faces,  edges ↔ edges
  Archimedean V  =  Catalan F
  Archimedean F  =  Catalan V
  Edges preserved

Both solids in each dual pair inscribe in the same sphere.
The duality is a spherical operation — mapping points on S² to points on S².
```

### Colatitude Structure

Colatitude = angular distance from the polar axis = **90° − latitude = 90° − α(n)**.

In the 364° system:

```
colat(n) = 90° − (182/n) × (90/91)
```

| n | α (latitude) | Colatitude | Dual n | Notes |
|---|-------------|-----------|--------|-------|
| 3 | 60.0° | **30.0°** | n=6 | colat(3) = lat(6) |
| 4 | 45.0° | **45.0°** | n=4 | **self-dual** in colatitude |
| 5 | 36.0° | **54.0°** | — | ≈ lat(10) + lat(5) complement |
| 6 | 30.0° | **60.0°** | n=3 | colat(6) = lat(3) |
| 7 | 25.714° | **64.286°** | — | 180/7 |
| 8 | 22.5° | **67.5°** | — | |
| 9 | 20.0° | **70.0°** | — | |
| 10 | 18.0° | **72.0°** | — | ≈ pentagon step 72.8° |
| 11 | 16.364° | **73.636°** | — | |
| 12 | 15.0° | **75.0°** | — | |
| 13 | 13.846° | **76.154°** | — | 180/13 |
| 14 | 12.857° | **77.143°** | — | 90/7 |
| 15 | 12.0° | **78.0°** | — | |

**Colatitude duality (limited to n=3,4,6):** colat(n=3) = lat(n=6) = 30°. colat(n=6) = lat(n=3) = 60°. The triangle and hexagon are colatitude duals. The square (n=4) is **self-dual** at 45°. This pattern does NOT extend generally — e.g. colat(5) = 54° ≠ lat(10) = 18°. The duality is specific to the (3,6) pair and the self-dual 4, which are precisely the polygons that tile the plane (triangular, square, hexagonal tilings). The connection to Archimedean/Catalan duality holds for these three only.

### Example: Rhombic Triacontahedron

32 vertices on the circumsphere. Relative to a 5-fold axis:

```
Vertex colatitudes (icosahedral geometry):
  colat₁ = atan(2) ≈ 63.43°
  colat₂ = 90° − atan(2) ≈ 26.57°
  colat₃ = 90° (equatorial)

All 30 rhombic faces: diagonals in golden ratio d₁/d₂ = φ

In 364° system:
  63.43° std → 64.12° in 364° ≈ n=7 colatitude (64.29°)
  26.57° std → 26.86° in 364° ≈ n=7 latitude (25.71°)
```

The rhombic triacontahedron's vertex colatitudes align **approximately** (within ~0.9°) with the heptagon's derived angles. This is a near-coincidence: arctan(2) = 63.435° is irrational and not a rational multiple of 180°, while 180/7 = 25.714° (colatitude 64.286°). The difference of 0.85° is suggestive but not structural.

---

## 11a. The Disdyakis Bridge (TM-2026-034)

The disdyakis triacontahedron — Catalan #12, dual of the truncated icosidodecahedron — is the geometric witness that connects icosahedral symmetry to the 364° framework. Full derivation in TM-2026-034 v1.0.

### The Circumradius Identity

The truncated icosidodecahedron (120 vertices, 62 faces: 12 decagons + 20 hexagons + 30 squares) has circumradius squared:

```
R² = 14 + 5φ    where φ = (1+√5)/2

14 = π_custom (PlenumNET)
```

This is an **exact algebraic identity** derived from the vertex coordinates in ℤ[φ], not an approximation. The integer part of R² in the golden ring is PlenumNET's fundamental constant.

**In plain language:** The truncated icosidodecahedron is the most complex Archimedean solid in existence — 120 vertices, 62 faces of three different shapes. The distance from its center to any vertex, squared, is exactly 14 + 5φ. The most complex uniform solid has π_custom baked into its size as an exact algebraic fact.

**Verification from all 5 base triples** (deviation < 10⁻¹⁵ in each case):

```
Triple (1/φ, 1/φ, 3+φ):     2/φ² + (3+φ)²   = (4−2φ) + (10+7φ)   = 14+5φ ✓
Triple (2/φ, φ, 1+2φ):       4/φ² + φ² + (1+2φ)² = ... = 14+5φ ✓
Triple (1/φ, φ², 3φ−1):      1/φ² + φ⁴ + (3φ−1)² = ... = 14+5φ ✓
Triple (2φ−1, 2, 2+φ):       (2φ−1)² + 4 + (2+φ)² = ... = 14+5φ ✓
Triple (φ, 3, 2φ):           φ² + 9 + 4φ²    = ... = 14+5φ ✓

Numerically: R² = 22.0901699437..., R = 4.7000180791...
```

**Normalization to crystal circumsphere:**

```
To inscribe the truncated icosidodecahedron in the crystal's
circumsphere of radius SR:

  Scale factor = SR / R = SR / √(14+5φ)
  All 120 vertex coordinates × scale factor

The disdyakis triacontahedron (its dual) then has vertices at:
  r_orbit = R² / d_face  (dual vertex distance from center)
  
  Normalized to crystal sphere:
    r₅ = SR × √(14+5φ) / √(10+5φ)   (5-fold vertices, 12 total)
    r₃ = SR × √(14+5φ) / √(6+9φ)    (3-fold vertices, 20 total)
    r₂ = SR × √(14+5φ) / √(10+7φ)   (2-fold vertices, 30 total)
  
  These are NOT all on the crystal circumsphere — 
  the disdyakis has 3 different orbital radii.
  Only when inscribed in its OWN circumsphere (not the crystal's)
  are all 62 vertices on a single sphere.
```

### The Re-Priming: 180 → 182

The structural content of the 364° system is an algebraic re-priming of the half-turn:

```
Standard:   180 = 2² × 3² × 5    → primes {2, 3, 5} → Platonic symmetry
PlenumNET:  182 = 2 × 7 × 13     → primes {2, 7, 13} → coprime generators

The golden ratio's action is identical in both systems:
  Standard golden angle:  180(3−√5) ≈ 137.508°
  PlenumNET golden angle: 182(3−√5) ≈ 139.036°

The irrational factor (3−√5) is invariant. Only the integer lattice changes.
The 180 → 182 shift re-primes φ through {7, 13} instead of {3, 5}.
```

This is not a rescaling. It is a change of the prime factorization of the angular foundation, threading the golden ratio through the coprime generators.

### The ℤ[φ] Lattice: Face-Centre Distances

Every metric quantity lies in ℤ[φ] = {a + bφ : a, b ∈ ℤ}:

| Quantity | a + bφ | Numerical | Defect R²−d² |
|----------|--------|-----------|-------------|
| d₁₀² (decagon, 12 faces) | 10 + 5φ | 18.090 | **4** (pure integer) |
| d₆² (hexagon, 20 faces) | 6 + 9φ | 20.562 | 4/φ² |
| d₄² (square, 30 faces) | 10 + 7φ | 21.326 | 2/φ² |
| R² (circumradius) | **14** + 5φ | 22.090 | — |

Defect ratio: (R²−d₁₀²) : (R²−d₆²) : (R²−d₄²) = **2φ² : 2 : 1**

The decagonal face produces a pure-integer defect (φ terms cancel exactly). The φ-dependent defects form a geometric series in 1/φ².

### All Four Coprime Generators in the Algebraic Data

| Generator | Where it appears | Mechanism |
|-----------|-----------------|-----------|
| **7** | 182 = 2 × 7 × 13 (triangle angle sum in 364°) | Half-turn prime factor |
| **11** | N(2 + 5φ) = 11 (ℤ[φ]-norm of 5-fold cosine numerator) | Algebraic norm |
| **13** | 182 = 2 × 7 × 13 (triangle angle sum in 364°) | Half-turn prime factor |
| **15** | cos(α₃) = (15 − 2√5)/20 (3-fold angle numerator) | ℚ(√5) coefficient |

Four generators, four independent algebraic channels. The disdyakis triacontahedron encodes the entire coprime quadruple.

### Face Angle Cosines in ℚ(φ)

| Angle at | cos(α) | Denominator | α (364°) |
|----------|--------|-------------|----------|
| 5-fold | (9 + 5√5) / 24 | 12 (= 5-fold vertex count) | 33.134° |
| 3-fold | (15 − 2√5) / 20 | 20 (= 3-fold vertex count) | 58.885° |
| 2-fold | (5 − 2√5) / 30 | 30 (= 2-fold vertex count) | 89.981° |
| **Sum** | — | lcm = 60 = \|I\| | **182.000°** |

The denominators are the vertex counts. Their lcm is the order of the icosahedral rotation group. The angle sum in 364° is the half-turn = arc root.

### Resonance Proof Engine Visualization

For the Three.js crystal engine, the disdyakis triacontahedron renders as:

```
62 vertices on the circumsphere (R² = 14 + 5φ):
  12 vertices (5-fold) — colored red   — innermost orbit
  20 vertices (3-fold) — colored green — middle orbit
  30 vertices (2-fold) — colored blue  — outermost orbit

120 congruent scalene triangle faces
  Each face connects one vertex from each orbit (red-green-blue)

Circumsphere R = √(14 + 5φ) displayed as semi-transparent shell
  with the identity "R² = 14 + 5φ = π + 5φ" as overlay text

Face coloring: by defect from circumsphere
  Decagonal (4):     warm (closest to sphere)
  Hexagonal (4/φ²):  neutral
  Square (2/φ²):     cool (farthest from sphere)
```

This makes the R² = 14 + 5φ identity geometrically tangible — each symmetry orbit at a different depth from the circumsphere, with the defect ratio 2φ² : 2 : 1 visible as a color gradient.

---

## 12. Coprime Walks on the Circumsphere

The 2D construction defines 27 coprime groups (9 pairs, 7 triples, 2 quadruples) that trace torus knots on the inscribed circle. In 3D, these become **walk paths on the circumsphere** connecting latitude bands.

### Coprime Pairs → Sphere Geodesics

A coprime pair (p, q) traces a torus knot T(p, q) on the circumsphere, connecting latitude bands ±α(p) and ±α(q).

**Torus knot crossing numbers:** For T(p, q) with p < q:

```
Standard minimum crossing number:  c = (p−1) × q
Construction's knotCross value:    kc = p × (q−1)

These are different quantities:
  c  = minimum crossing number of torus knot T(p,q) (knot theory)
  kc = number of edge-pair intersections in the 2D polygon overlay

Derivation of kc = p(q−1):
  A regular p-gon inscribed in a circle has p edges.
  A regular q-gon inscribed in the same circle has q edges.
  Each edge of the p-gon is a chord spanning an arc of 364/p custom°.
  Each edge of the q-gon spans 364/q custom°.
  With the half-step rotation between top and bottom rings,
  each p-gon edge crosses (q−1) of the q-gon's q edges
  (it misses one because the half-step aligns it parallel to that one).
  Total: p edges × (q−1) crossings each = p(q−1).

Relationship: kc = c + (q − p), always.
  Proof: c = pq−q, kc = pq−p, so kc − c = q − p ✓

Example T(7,11): c = 6×11 = 66 (standard), kc = 7×10 = 70 (construction)
  Difference = 11 − 7 = 4

The construction's groupData uses kc (edge-pair intersections),
not c (knot-theoretic minimum). Both are legitimate; they count
different things. The HTML's knotCross values are correct for
what they measure.
```

| Pair | α(p) | α(q) | Lat span | lcm | knotCross p(q−1) | Genus g | Arc segments |
|------|------|------|---------|-----|-------------------|---------|-------------|
| (7, 11) | 25.714° | 16.364° | 9.35° | 77 | 7×10 = 70 | **30** | 77 |
| (7, 13) | 25.714° | 13.846° | 11.87° | 91 | 7×12 = 84 | 36 | 91 |
| (7, 15) | 25.714° | 12.000° | 13.71° | 105 | 7×14 = 98 | 42 | 105 |
| (11, 13) | 16.364° | 13.846° | 2.52° | 143 | 11×12 = 132 | 60 | 143 |
| (11, 14) | 16.364° | 12.857° | 3.51° | 154 | 11×13 = 143 | 65 | 154 |
| (11, 15) | 16.364° | 12.000° | 4.36° | 165 | 11×14 = 154 | 70 | 165 |
| (13, 14) | 13.846° | 12.857° | **0.99°** | **182** | 13×13 = **169** | **78** | **182** |
| (13, 15) | 13.846° | 12.000° | 1.85° | 195 | 13×14 = 182 | 84 | 195 |
| (14, 15) | 12.857° | 12.000° | 0.86° | 210 | 14×14 = 196 | **91** | 210 |

### Seifert Surface Genus

The **Seifert surface** of a torus knot T(p,q) is an orientable surface bounded by the knot in S³. Its genus measures the topological complexity of the knot:

```
g = (p−1)(q−1) / 2
```

This is relevant to the 4D Clifford embedding (§12 dimensional ladder): the Seifert surface lives in S³, which is exactly where the Clifford torus parametrization places the coprime walk. The genus counts the number of "handles" the surface needs — it quantifies how tangled the walk is in 4D.

**Framework numbers as genera:**

```
T(7, 11):  g = 6×10/2  = 30    Brieskorn pair genus
T(7, 13):  g = 6×12/2  = 36    = 6²
T(7, 15):  g = 6×14/2  = 42    = 6 × 7 = 3 × π_custom
T(11, 14): g = 10×13/2 = 65    = 5 × 13 = 5 × radian
T(13, 14): g = 12×13/2 = 78    = 6 × 13 = 6 × radian
T(13, 15): g = 12×14/2 = 84    = 12 × 7 = 6 × π_custom
T(14, 15): g = 13×14/2 = 91    = 7 × 13 = QUARTER TURN
```

**The π_custom factor (14) recurs:** g(7,15) = 3×14, g(13,15) = 6×14, and g(14,15) = 91 = 182/2 where 182 = 13×14. The tetradecagon (n=14 = π_custom) seeds its value into the genera of every pair it touches. The radian (13) does the same: g(11,14) = 5×13, g(13,14) = 6×13, g(14,15) = 7×13.

### **Highlighted Result: Genus of T(14,15) = 91**

```
T(14, 15): g = (14−1)(15−1)/2 = 13 × 14 / 2 = 182/2 = 91

  13 × 14 = 182    (the arc root, the squared circle area)
  182 / 2  = 91     (the quarter turn, 7×13, the C₁₈₂ angle)

The numerator is the arc root. The result is the quarter turn.
The Seifert surface genus of the (π_custom, pentadecagon) torus knot
encodes both fundamental constants of the 364° system in one formula.

This is the arc equation expressed as topology:
  arc root → Seifert genus → quarter turn
  182      → g = 182/2    → 91
```

### **Highlighted Result: The (13, 14) Pair**

```
Pair (13, 14): latitude span = 0.99° — less than 1°
  lcm = 182 = arc root of arc² − 832·arc + 118,300 = 0
  knotCross = 13 × 13 = 169
  Arc segments = 182

The tridecagon (radian polygon, n=13) and tetradecagon (π_custom-gon, n=14)
are nearly co-latitudinal. Their walk produces 182 great circle
arcs packed into a sub-1° latitude band — the densest knot 
pattern in the crystal, and its lcm closes to the arc root.

13 × 14 = 182.  The radian times π_custom equals the arc root.
This is the squared circle expressed as a coprime walk.
```

### Coprime Triples → Walk on T³

A coprime triple (p, q, r) does NOT form a torus knot — torus knots T(p,q) live on a 2-torus T². A triple traces a path on a **3-torus** T³ = S¹ × S¹ × S¹, which is a fundamentally different topological object. The crossing number formula `q(p−1)` does not apply to triples.

The Brieskorn triple (7, 11, 13) visits three latitude bands:

```
α₇  = +25.714°  = +180/7   (heptagon band)
α₁₁ = +16.364°  = +180/11  (hendecagon band)  
α₁₃ = +13.846°  = +180/13  (tridecagon band)

Walk visits all three bands with lcm = 1,001 steps
The path lives on T³, not T² — its topology is described by
the orbit structure of Z₇ × Z₁₁ × Z₁₃ acting on the sphere
Brieskorn sphere Σ(7,11,13): link of x⁷ + y¹¹ + z¹³ = 0 in ℂ³
```

### The Dimensional Ladder: 2D → 3D → 4D

The coprime walk exists at each dimensional level:

```
2D:  Torus knot T(p,q) projected flat on the inscribed circle
     → The familiar 364° construction view
     → Crossing number = q(p−1) visible as edge intersections
     
3D:  Walk path on the circumsphere surface
     → Connects latitude bands at ±α(p) and ±α(q)
     → Great circle arcs between antiprism vertices
     → Pairs: knots on S². Triples: paths on T³ embedded in S².
     
4D:  Clifford torus embedding — general form for triple (p,q,r):
     x = cos(p·t)·cos(q·t)
     y = cos(p·t)·sin(q·t)
     z = sin(p·t)·cos(r·t)
     w = sin(p·t)·sin(r·t)
     
     Brieskorn instance (7,11,13):
     x = cos(7t)·cos(11t),  y = cos(7t)·sin(11t)
     z = sin(7t)·cos(13t),  w = sin(7t)·sin(13t)
     
     This is a curve on S³ (the 3-sphere in 4D)
     The Seifert surface of genus g = (p−1)(q−1)/2 spans the knot IN S³
     Stereographic projection S³ → ℝ³ gives the 3D sphere path:
       For point (x,y,z,w) on S³:
         X₃D = x/(1−w),  Y₃D = y/(1−w),  Z₃D = z/(1−w)
       (projecting from the pole w=1)
     Further projection ℝ³ → ℝ² gives the 2D construction
```

Each dimensional step is a projection that loses information:
- 4D Clifford: full topological structure, no crossings
- 3D sphere: path crossings appear (knot theory)
- 2D flat: all structure compressed to plane (the construction as drawn)

The crystal IS the 3D intermediate — the stage where the full 4D topology is partially visible.

### Coprime Quadruples → Maximum Walk Density

```
(7, 11, 13, 15):  lcm = 15,015    → 15,015 conflict-free 2D positions
                   At Δ₂ = 729 = 3⁶: 21,891,870 volumetric positions in 3D+
                   Latitude span: α₇ − α₁₅ = 25.714° − 12.000° = 13.714°

(11, 13, 14, 15): lcm = 30,030    → 2 × 15,015
                   Latitude span: α₁₁ − α₁₅ = 16.364° − 12.000° = 4.364°
                   Denser packing in narrower band
```

### Quintuples and Sextuples (Full Coprime Landscape)

The 364° construction's coprime groups extend beyond quadruples. The full landscape:

```
27 total coprime groups:
  9 pairs     → torus knots T(p,q) on S²
  7 triples   → paths on T³
  5 quintuples → paths on T⁵  (5 simultaneous latitude bands)
  4 sextuples  → paths on T⁶  (6 simultaneous latitude bands)
  2 quadruples → paths on T⁴

Higher groups span more latitude bands simultaneously:
  Quintuple example: (7, 11, 13, 14, 15)
    Span: α₇ − α₁₅ = 25.714° − 12.000° = 13.714°
    lcm = lcm(7,11,13,14,15) = 30,030
    Visits 5 latitude bands in the equatorial cluster

  Sextuple example: (7, 9, 11, 13, 14, 15)  
    Span: α₇ − α₁₅ = 13.714° (same outer bounds)
    lcm grows further
    Visits 6 bands — nearly the full equatorial zone
```

---

## 13. The Spherical Tiling — Geodesic Structure

The 364° crystal is not merely a collection of inscribed polyhedra. It is a **spherical tiling**: polygon edges projected onto the sphere surface as great circle arcs create a tessellation.

### From Polyhedra to Spherical Tiling

```
Each polyhedron edge → great circle arc on sphere
Each polyhedron face → spherical polygon
Sum of all spherical polygon areas = 4πR² (full sphere)
```

With all solid families overlaid on the same sphere, the resulting tiling has extraordinary density and structure.

### Multi-Resolution Geodesic Mesh

A geodesic dome subdivides icosahedron faces on a sphere. The 364° crystal does something more general: it overlays **multiple geodesic resolutions simultaneously**.

```
Each polygon n provides a resolution level:
  n=3:   coarsest — 6 antiprism vertices, large spherical cells
  n=15:  finest  — 30 antiprism vertices, fine cells

Overlaying all 13: multi-resolution spherical mesh
  Antiprism vertices alone:     234 points
  + Platonic vertices:          ~50 (with overlaps)
  + Archimedean vertices:       ~600 (with overlaps)
  + Catalan vertices:           ~500 (with overlaps)
```

### Vertex Density by Colatitude Band

```
Polar cap (0°–30°):     n=3 only → very sparse
Temperate (30°–60°):    n=4, 5, 6 → medium density
Equatorial (60°–78°):   n=7 through 15 → HIGH density
                        9 polygons in 18° of colatitude
Near-equator (78°–90°): bottom rings of n=13,14,15 only
```

The crystal is **denser near the equator** — matching geodesic dome design where equatorial cells are smallest and most numerous. The non-uniformity is structural: 3 polygons spread over 30° in the temperate zone vs 9 polygons packed into 18° in the equatorial zone.

### UV Spectral × Colatitude Grid

The four UV spectral bands map to **longitude bands** with non-uniform widths:

```
EUV:   0°–91°    = 91°  (25.0% of circle)  — Lyman limit
UV-C:  91°–182°  = 91°  (25.0% of circle)  — O₂ wall
UV-B:  182°–286° = 104° (28.6% of circle)  — O₃ bridge (widest)
UV-A:  286°–364° = 78°  (21.4% of circle)  — full window (narrowest)
```

Combined with the 13 colatitude bands:

```
Longitude:  4 UV bands (azimuthal, non-uniform: 91+91+104+78 = 364)
Colatitude: 13 polygon bands (polar, clustered near equator)
Result:     ~52 spherical zones with non-uniform areas
            Polar zones: large and sparse
            Equatorial zones: small and dense
```

### Spherical Excess

For a spherical polygon with interior angles α₁, α₂, ..., αₙ:

```
Spherical excess E = (α₁ + α₂ + ... + αₙ) − (n−2)×180°
Spherical polygon area = E × R²

In 364° framework (π=14):
  Full sphere area = 56R²
  Σ E_i = 56  (over all spherical faces, in 364° area units)
```

---

## 14. Complete Shape Inventory

| Category | Count | Details |
|----------|-------|---------|
| Circumsphere | 1 | R, all vertices on surface |
| Antiprisms | 13 | n = 3–15, α = (182/n)×90/91 |
| Platonic solids | 5 | Tetra, Cube, Octa, Icosa, Dodeca |
| Brieskorn solid | 1 | Σ(7,11,13), lcm = 1,001 |
| Archimedean solids | 13 | Complete basis from polygon set |
| Catalan solids | 13 | Duals of Archimedeans, face-transitive |
| **Disdyakis triacontahedron** | **(1)** | **Geometric witness: R² = 14 + 5φ (§11a, TM-2026-034)** |
| **Total solids** | **46** | All inscribable in circumsphere |

Plus 13 n-gonal prisms = 59. Plus the spherical tiling with ~52 spectral-colatitude zones.

The disdyakis triacontahedron is already counted in the 13 Catalans but is highlighted separately as the **geometric witness** — the solid whose circumradius encodes π_custom as an exact algebraic identity.

---

## 15. Known Issues in Current Build (v7)

1. **Belt triangle winding** — may be reversed on some antiprisms; needs audit
2. **Mouse navigation** — pointer events broken in v7; needs fix before next build
3. **Rays** — not real refraction; use Three.js `MeshPhysicalMaterial` with `transmission` and `ior` for sphere boundary; internal faces produce lattice deflection, not Snell
4. **Named solid alignment** — partially resolved for n=3 and n=4 via worked examples; remaining solids unverified
5. **Dodecahedron** — using Three.js built-in geometry, not derived from n=5/n=10 vertices; needs custom construction
6. **Archimedean solids** — not yet built; vertex coordinates need derivation from 364° polygon intersections
7. **Catalan solids** — not yet built; require duality operation (vertex↔face) on Archimedean solids
8. **Prisms** — 13 n-gonal prisms mentioned in inventory but not detailed or built
9. **Spherical tiling** — geodesic mesh, great circle arcs, zone coloring not yet implemented
10. **Coprime walks** — 2D torus knot projections attempted but 3D sphere-path visualization unbuilt
11. **Clifford projection** — 4D→3D stereographic projection referenced but not implemented
12. **Superhub zones** — A/B/C/D intersection loci documented in §1 but not projected onto circumsphere
13. **Worked vertex tables** — only n=3, n=4, n=5, n=7 computed; remaining 9 antiprisms need full coordinate derivation
14. **Disdyakis visualization** — TM-2026-034 specifies orbit-colored vertices (red 5-fold, green 3-fold, blue 2-fold) on circumsphere R²=14+5φ with defect-gradient face coloring; not yet implemented in Three.js

---

## 16. Next Steps

1. Verify n=3 and n=4 antiprism vertex tables against the 2D construction
2. Fix v7 mouse navigation
3. Build remaining 9 antiprisms one at a time, verifying each against `pa(deg)`
4. Build named solids with full vertex-coordinate provenance
5. Derive Archimedean solid vertices from 364° polygon geometry
6. Build Catalan solids via duality (vertex↔face exchange on sphere)
7. **Build disdyakis triacontahedron visualization** — orbit-colored vertices on R²=14+5φ circumsphere, defect-gradient face coloring (§11a)
8. Implement coprime walk paths as great circle arcs on circumsphere (§12)
9. Build Clifford torus 4D→3D stereographic projection visualization
10. Implement spherical tiling — great circle arcs, zone coloring, area computation
11. Implement correct sphere-boundary refraction (Three.js `MeshPhysicalMaterial` with `transmission`/`ior`)
12. Compute worked vertex tables for remaining antiprisms (n=6, 8–15)
13. Derive superhub zone positions (A/B/C/D) on circumsphere via great circle arc intersection
14. **Investigate TM-2026-034 Q1:** do other Archimedean circumradii have system-significant integer parts in ℤ[φ]?
15. **Investigate TM-2026-034 Q4:** can 7 and 13 be surfaced from the disdyakis's intrinsic geometry (not just from 182 = 2×7×13)?

---

*© 2026 Capomastro Holdings Ltd — Applied Physics Division — Patent Pending*  
*TM-2026-035 v1.0 · R. Salvi · GitHub: SigmaWolf-8/Ternary*