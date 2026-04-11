# TM-2026-034 — The Disdyakis Bridge

Icosahedral geometry in the 364° circle. How the re-priming of the half-turn from 180 = 2²×3²×5 to 182 = 2×7×13 threads the golden ratio through the PlenumNET coprime generators.

## When to Use

Activate this skill when working on:
- Icosahedral or polyhedral geometry in the Salvi Framework
- The golden ratio φ and its ℤ[φ] ring properties in framework code
- Coprime generator hierarchy (7, 11, 13, 15) and their algebraic provenance
- The circumradius identity R² = 14 + 5φ (14 = π in PlenumNET)
- Disdyakis triacontahedron face angles and cosine expressions in ℚ(φ)
- Schwarz triangle / icosahedral fundamental domain calculations
- Archimedean solid R² survey (Appendix A — four framework constants appear as integer parts)
- Quadratic reciprocity and Legendre symbol (5/p) splitting behaviour in ℤ[φ]
- Hyperbolic Coxeter group (7, 11, 13) derived from the repunit axiom (Q6)
- Brieskorn sphere Σ(7, 11, 13) exotic 3-manifold connection

## Key Results

### R² = 14 + 5φ
The circumradius squared of the truncated icosidodecahedron. Integer coefficient 14 = π in PlenumNET. Exact algebraic identity, not approximation.

### Generator Hierarchy (forced by quadratic reciprocity)
| Level | Generator | Mechanism | Legendre (5/p) |
|-------|-----------|-----------|----------------|
| 1 (strongest) | 11 | ℤ[φ]-norm N(2+5φ) = 11 | +1 splits |
| 2 | 7 | ℤ[φ] coefficient cos(α₂) = (7−4φ)/30 | −1 inert |
| 3 | 15 | ℚ(√5) coefficient (non-preferred basis only) | 3 inert, 5 ramifies |
| 4 (weakest) | 13 | Global metric 182 = 2×7×13 only | −1 inert |

### Face Angle Cosines in ℚ(φ)
| Angle at | cos(α) | Denominator = vertex count |
|----------|--------|---------------------------|
| 5-fold | (2 + 5φ) / 12 | 12 (icosahedral) |
| 3-fold | (17 − 4φ) / 20 | 20 (dodecahedral) |
| 2-fold | (7 − 4φ) / 30 | 30 (icosidodecahedral) |
| lcm(12,20,30) = 60 = \|I\| (icosahedral rotation group order) |

### Defect Structure
R² − d² = {4, 4/φ², 2/φ²} in ratio 2φ² : 2 : 1

### Archimedean R² Survey (Appendix A)
Integer parts of R² across all 11 non-chiral Archimedeans: {2, 2, 5, 5, 5, **7**, **7**, 10, **11**, **13**, **14**}. Four framework constants surface.

## Full Document
Read the complete technical memo at: `docs/technical-memos/TM-2026-034-Disdyakis-Bridge.md`
