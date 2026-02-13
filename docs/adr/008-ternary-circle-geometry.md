# ADR-008: Ternary Circle Geometry — 364° / π = 14

**Status:** Accepted
**Date:** 2026-02-12
**Author:** Salvi — Applied Physics Division, Capomastro Holdings Ltd.
**Related:** ADR-003 (Ternary Arithmetic Engine), ADR-007 (Carry Propagation and Timing Jerk)

---

## Context

The Salvi Framework's geometric subsystem previously used conventional angular units (360° circle, π ≈ 3.14159, 1 radian ≈ 57.296°) with ternary-specific operations layered on top. The Triskellion walk visualization used 120° turns (360°/3) — a decimal-circle quantity divided by three, not a natively ternary construct.

The document "Unification: The Ternary Circle and the Tribonacci Radian" demonstrated that the circle can be re-founded on the ternary radix itself, yielding a system where every angular quantity is a native base-3 expression:

- **Full circle = 364°** = `111111₃` (six-digit base-3 repunit)
- **π = 14** (ratio of circumference to diameter)
- **1 radian = 13°** = `111₃` (three-digit repunit AND the seventh Tribonacci number T₇)
- **Full circle = 28 radians** → cyclic group **Z₂₈**

The question: should the framework adopt this ternary-native angular system as its canonical geometry, replacing all conventional angular constants?

## Decision

**Yes.** The 364° ternary circle is adopted as the canonical angular system for all Salvi Framework components.

### Rationale

1. **The radian is simultaneously a ternary repunit and a Tribonacci number.** The fact that 13 = `111₃` = T₇ is not a coincidence we engineer — it is a mathematical identity that connects the recurrence structure (Tribonacci) with the positional structure (base-3 repunit) in a single constant. This is the kind of structural unification the framework is designed to expose.

2. **The full circle is a repunit.** 364 = `111111₃` = (3⁶ − 1)/2. The circle is not described in base 3; the circle IS a base-3 repunit. Every angular measurement is an integer combination of repunits.

3. **The cyclic group Z₂₈ replaces continuous rotation with discrete lattice directions.** The 28 positions of Z₂₈ (each separated by 13°) provide a finite, exact angular grid. The Tribonacci radian spiral walks this lattice with τ-scaling, producing a quasicrystalline point set with 28-fold discrete rotational symmetry.

4. **The golden angle factors through Tribonacci numbers.** In the ternary circle, the classical golden angle becomes Θ_φ,364 = 364/φ² = 2·7·13·(3−√5)°, where 2, 7, and 13 are all Tribonacci numbers (T₄, T₆, T₇). The irrationality (3−√5) is cleanly separated from the Tribonacci arithmetic.

5. **Phase corrections in HPTP become Z₂₈ operations.** Clock phase offsets are positions in Z₂₈. Addition of corrections is group addition mod 28. The minimum correction quantum is one ternary radian (13°). This quantization provides a natural noise floor for timing jitter — sub-radian corrections indicate clocks are within alignment tolerance.

### Implementation

- **Rust module:** `libternary/src/ternary_circle.rs` — all constants, Z₂₈ struct with group operations, conversion functions, spiral walk engine, repunit verification. 20+ unit tests.

- **TypeScript module:** `shared/ternary-circle.ts` — parallel constants and conversion functions for the client/server TypeScript layer. Migration guide included.

- **Visualization:** `triskellion-walk.html` rebuilt — walk on 28 lattice rays using 13° ternary radian steps with τ-scaling, replacing the old 120° three-arm system.

- **Conversion at boundary:** Standard geometry libraries (`Math.cos`, `Math.sin`) require standard radians. The functions `ternary_rad_to_std_rad()` and `ternary_deg_to_std_deg()` perform the conversion at the cos/sin boundary only. All internal computation uses ternary degrees or ternary radians.

### Migration Path

Search the codebase for these patterns and replace:

| Legacy Pattern | Replacement |
|---|---|
| `360` (degrees) | `FULL_CIRCLE_DEG` (364) |
| `Math.PI` / `std::f64::consts::PI` | Use conversion functions at cos/sin boundary |
| `2 * Math.PI` | `TWO_PI_TERNARY` (28) then convert at boundary |
| `120` (degree turns) | `RADIAN_DEG` (13) — one ternary radian |
| `2 * Math.PI / 3` | `ternaryRadToStdRad(1)` — one ternary radian in std |

## Consequences

### Positive

- Angular geometry now speaks the same language as the arithmetic (base 3) and the topology (Tribonacci/Borromean). The Rosetta Stone gains a third face.
- The Z₂₈ cyclic group provides a finite, exact angular grid — no floating-point rounding in direction computation.
- Phase corrections in HPTP have a natural quantum (13°) and a formal group structure.
- The ternary radian spiral is a lattice walk, not a continuous curve — discreteness is a feature.

### Negative

- Every interaction with standard geometry libraries (CSS transforms, SVG, WebGL, etc.) requires a conversion at the boundary. This is a pervasive tax.
- External collaborators will initially be confused by "π = 14" — the documentation must clearly explain this is a re-founding, not an approximation.
- The 364° system has no established ecosystem. We are the first implementation.

### Neutral

- The old 120° Triskellion walk was a valid visualization of the Tribonacci word morphism. It still exists as a historical artifact (the ternary radian spiral subsumes it — at 120° ≈ 9.23 ternary radians, which is not a natural position in Z₂₈).

---

## References

- "Unification: The Ternary Circle and the Tribonacci Radian" — Salvi Framework internal document
- Base-3 repunits: OEIS A003462 (1, 4, 13, 40, 121, 364, 1093, …)
- Tribonacci sequence: OEIS A000073 (0, 0, 1, 1, 2, 4, 7, 13, 24, 44, 81, …)
- Tribonacci constant: τ ≈ 1.839286755214161 (real root of x³ − x² − x − 1 = 0)
- Cyclic groups: Z₂₈ generated by ⟨1⟩ under addition mod 28
