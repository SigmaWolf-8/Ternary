# TM-2026-035 — 364° Crystal: 2D→3D Derivation

The complete 2D-to-3D lifting of the 364° inscribed-circle construction (TM-2026-017 v8.0). Derives antiprism latitudes, circumsphere geometry, coprime walk topology, and the dimensional ladder from circle to sphere to Clifford torus.

## When to Use

Activate this skill when working on:
- 3D visualization of the 364° crystal (Three.js circumsphere, antiprisms, spherical tiling)
- Antiprism construction for any of the 13 polygons (n=3..15)
- Latitude calculations for polygon lifting (α = 180/n standard degrees)
- Coprime walk paths on the circumsphere (torus knots, Seifert genera, Brieskorn triples)
- The dimensional ladder: 2D flat → 3D sphere → 4D Clifford torus
- Spherical tiling, geodesic mesh, or zone computation (UV spectral × colatitude grid)
- Superhub zone 3D projection (UNRESOLVED — great circle arc intersection needed)
- Belt triangle winding for antiprism mesh generation
- The re-priming identity: 180 → 182 (prime factors {2,3,5} → {2,7,13})
- Information-theoretic crystal (Current/Stillness/Rock entropy decomposition)
- Pentagon antiprism φ-saturation (cos 36° = φ/2 = latitude cosine)
- Circumsphere properties in the 364° framework (surface area 56R², volume 56R³/3)

## Key Results

### Latitude Formula
```
α_custom(n) = 182/n  custom degrees
α_std(n)    = 180/n  standard degrees (for all trig calculations)

Derivation: 182/n × 90/91 = 180/n  because 182 = 2 × 91
```

### Antiprism Construction
- Top ring: n-gon at latitude +α, vertices at 364° angles `off + k × step`
- Bottom ring: n-gon at latitude −α, rotated by half-step `182/n` custom degrees
- Belt: 2n triangles (n downward + n upward) with consistent winding
- Total per antiprism: 2n+2 faces, 2n vertices, all on unit sphere

### 3D Coordinate Mapping
```
θ = deg × 2π / 364
X = sin(θ) · cos(α)    Y = sin(α)    Z = −cos(θ) · cos(α)
|V|² = 1  (all vertices on unit sphere)
```

### Coprime Walk Topology (§12)
| Pair | lcm | Seifert genus | Significance |
|------|-----|---------------|--------------|
| (7, 11) | 77 | 30 | Brieskorn pair |
| (13, 14) | **182** | 78 | Arc root; sub-1° latitude span; densest knot |
| (14, 15) | 210 | **91** | Quarter turn = genus; arc equation as topology |

### Seifert Genus = 91 (Highlighted)
```
T(14, 15): g = (14−1)(15−1)/2 = 13×14/2 = 182/2 = 91
Numerator = arc root (182). Result = quarter turn (91).
```

### Re-Priming (§11a, links to TM-2026-034)
```
Standard:   180 = 2² × 3² × 5    → Platonic symmetry
PlenumNET:  182 = 2 × 7 × 13     → coprime generators
Golden angle invariant factor: (3−√5)
```

### Pentagon Antiprism φ-Saturation (§6a)
```
cos(36°) = φ/2 = 0.8090 = latitude cosine = P vertex mx-component
sin(36°) = √(10−2√5)/4   cos(72°) = 1/(2φ)
Belt edge ratio = φ (golden proportion)
```

### Circumsphere (364° Framework, §7)
```
Surface area = 56R²    Volume = 56R³/3    Great circle = 28R
```

### Shape Inventory (§14)
46 solids: 1 circumsphere + 13 antiprisms + 5 Platonic + 1 Brieskorn + 13 Archimedean + 13 Catalan (+ 13 prisms = 59)

### Spherical Tiling (§13)
~52 zones from 4 UV longitude bands × 13 colatitude bands. Density peaks at equatorial belt (9 polygons in 18° of colatitude).

### Known Open Issues (§15)
14 items including: superhub 3D projection (UNRESOLVED), belt winding audit, remaining 9 antiprism vertex tables, Archimedean/Catalan construction, Clifford 4D→3D projection, spherical tiling implementation.

## Cross-References
- **TM-2026-017 v8.0**: Source 2D construction (13 inscribed polygons, superhub zones, coprime groups)
- **TM-2026-034**: Disdyakis Bridge (R² = 14+5φ, face angle cosines, defect structure, ℤ[φ]-norm)
- §11a reproduces TM-034's face-centre distances and face angle cosines in the crystal context

## Full Document
Read the complete technical memo at: `docs/technical-memos/TM-2026-035-364-Crystal-2D-3D-Derivation.md`
