# TM-2026-026 — PlenumNET UV Spectral Protocol (PUV v1.2)

First-principles UV band definitions, protocol specification, and applied framework. Replaces empirically drawn UV band boundaries with exact integers derived from the axiom π = 14.

## When to Use

Activate this skill when working on:
- UV spectral band classification or wavelength handling in PlenumNET code
- The four system wavelengths: 91, 182, 286, 364 nm
- VACUUM_BIAS (+0.194%) or UNIVERSAL_BIAS (+0.139%) constants
- PUV protocol data structures (Rust or TypeScript)
- Band classification logic (EUV, UV-C, UV-B, UV-A)
- The ozone bridge ratio 286/91 = 22/7 (Archimedean π)
- Secondary system integers: 222 (KrCl excimer), 308 (XeCl excimer), 311 (NB-UVB therapeutic)
- Orbifold Euler characteristic χ = −690/1001 and its UV decomposition
- Spectral irradiance data at system wavelengths
- UV torus knot spectral crossings
- Industry applications of first-principles UV definitions
- Hydrogen series limits and the Rydberg confirmation

## Key Results

### Four System Wavelengths
| Value (nm) | Derivation | Factorization | UV Band |
|------------|-----------|---------------|---------|
| 91 | Quarter-turn = 7 × 13 | 7 × 13 | EUV edge |
| 182 | Half-turn = π × radian | 14 × 13 | UV-C (O₂ wall) |
| 286 | Green arc eff. = 650 mod 364 | 22 × 13 | UV-B (O₃ bridge) |
| 364 | Full circle = 2π × radian | 28 × 13 | UV-A (transmission) |

### Vacuum Bias
+0.194% = +0.139% (R∞ bias, UNIVERSAL_BIAS) + 0.055% (reduced-mass, VACUUM_BIAS). Constant across all hydrogen series limits.

### The Ozone Bridge
286/91 = 22/7 — Archimedean π as exact structural ratio. UV-B = Lyman threshold × π_Archimedes.

### Secondary Integers (§7.4)
- 222 = 2 × 111 (KrCl excimer, germicidal)
- 308 = 4 × 77 (XeCl excimer)
- 311 = prime (NB-UVB therapeutic, spectral weight e₂)

### Orbifold Euler Characteristic (§7.3)
χ(Σ(7,11,13)) = −690/1001. Numerator 690 = 650 + 40 = arc root + R₄. UV integers (143, 91, 77) appear as pairwise products in the reciprocal sum 1/7 + 1/11 + 1/13 = 311/1001.

### PUV Protocol (§8)
Rust data structures, deterministic band classification, exact rational ratios, vacuum-bias conversion functions.

## Cross-References
- **TM-2026-017 v7.0 §18**: UV Spectral Correspondence (summary version)
- **TM-2026-028**: HModal Signaling Architecture (coprime walk scheduling)

## Full Document
Read the complete technical memo at: `docs/technical-memos/TM-2026-026-UV-Spectral-Protocol.md`
