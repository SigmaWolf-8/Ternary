# Algeometric Arc Σ-182 Calculi (`aasc`)

> **UPIID v1.1 governed.** Pure ternary computation crate. The single
> canonical home of the Σ-182 axis algebra, Plenum geometry, and the
> four sub-calculi.

## What this crate is

`aasc` consolidates the trit-pure subset of the Salvi Framework into one
`no_std`-clean Rust crate. Everything inside operates on `TritVec`. The
single permitted byte ↔ trit boundary lives in [`bridge`] and is
gated behind `feature = "bridge"`.

## Layers

| Layer        | Modules                                                                                                     |
|--------------|-------------------------------------------------------------------------------------------------------------|
| **Atom**     | [`trit`], [`tritvec`], [`arithmetic`]                                                                       |
| **Notation** | [`constants`] (every numeric literal lives here, exactly once)                                              |
| **Algebraic**| [`gf3`], [`zphi`], [`borromean`], [`coprime`]                                                               |
| **Geometric**| [`circle`], [`generating_system`], [`dual_circle`], [`repunit_circles`], [`tribonacci`], [`triangular_numbers`], [`coprime_polygon_pair`], [`plenum_color_harmonics`], [`arc182`], [`plenum_square`], [`gait`], [`disdyakis_bridge`], [`crystal_2d_3d`], [`uv_spectral`], [`hydrogen_spectral`], [`speed_of_light`], [`saturnian_metatron`], [`wave_stratum`], [`conservation_laws`], [`nona_state`], [`wieferich_register`], [`grh_register`], [`beal_register`] |
| **Calculi**  | [`repx`] (bijective base-`b`), [`milesian`] (`divmod(b³)` + glyphs), [`walk`] (coprime walks + CRT), [`calculus`] (difference / circular / iteration / series) |
| **Boundary** | [`bridge`] (gated)                                                                                          |

## Notation table (canonical)

| Symbol           | Value | Where defined        |
|------------------|-------|----------------------|
| `B_INT`          | 3     | `constants::B_INT`   |
| `B3_INT`         | 27    | `constants::B3_INT`  |
| `B6_INT`         | 729   | `constants::B6_INT`  |
| `R_1_INT`        | 1     | `constants::R_1_INT` |
| `R_2_INT`        | 4     | `constants::R_2_INT` |
| `R_3_INT`        | 13    | `constants::R_3_INT` |
| `R_5_INT`        | 121   | `constants::R_5_INT` |
| `R_6_INT`        | 364   | `constants::R_6_INT` |
| `PI_INT`         | 14    | `constants::PI_INT`  |
| `ARC_INT`        | 182   | `constants::ARC_INT` (= π·(π−1) = R₆/2 = 2·p·R₃ = 2·λ_EUV) |
| `DELTA_SPONGE_INT` | 729 | `constants::DELTA_SPONGE_INT` (= 1 + 4·ARC = b⁶) |
| `P_INT`, `Q_INT`, `R_INT` | 7, 11, 13 | `constants::*` (pairwise coprime, p·q·r = 1001) |
| `P_H_INT`        | 11    | `constants::P_H_INT` (Generator Duality with R_3) |
| `COMBINED_VERTICES_INT` | 23 | `constants::COMBINED_VERTICES_INT` |
| `M_SQ_INT`       | 12    | `constants::M_SQ_INT` (= R_2 · b) |
| `SIGMA_TILDE_INT`| 3699  | `constants::SIGMA_TILDE_INT` (= 27·137 = b³·⌊1/α⌋) |
| `LAMBDA_LYMAN_INT` | 91   | `constants::LAMBDA_LYMAN_INT` |
| `LAMBDA_UVC_INT` | 182   | `constants::LAMBDA_UVC_INT` |
| `LAMBDA_UVB_INT` | 286   | `constants::LAMBDA_UVB_INT` |
| `LAMBDA_UVA_INT` | 364   | `constants::LAMBDA_UVA_INT` |

Every identity in the Notation table is re-proved at compile time inside
the const identity block at the bottom of `constants.rs`. If any value
were typed wrong, the crate would refuse to compile.

## The 50 invariants

The crate proves Tier-1 invariants **at compile time** (via
`const _: () = { … assert!(…); … };` blocks) and Tier-2 invariants **at
test time** in `tests/invariants.rs`.

| Tier | Coverage | Proof site |
|------|----------|-----------|
| Tier 1 (algebraic) | I-1 .. I-19, I-22 .. I-24, I-29, I-31, I-44, I-46 | const identity blocks throughout `src/` |
| Tier 2 (numerical) | I-1 .. I-46 named tests, I-37 sweep 0..1001 | `tests/invariants.rs` |
| Tier 3 (boundary)  | I-3 (no opaque-byte leak), no_std-clean | `tests/no_boundary_leak.rs`, `tests/no_std_in_core.rs` |

## Features

| Feature   | Default | Effect |
|-----------|---------|--------|
| `bridge`  | off     | Compiles `bridge.rs`, the bytes ↔ TritVec boundary |

The crate is `no_std + alloc` clean on the default feature path.

## Calculi

The four sub-calculi (`repx`, `milesian`, `walk`, `calculus`) all
operate on `TritVec`. The only function in the whole crate that
narrows from a TritVec digit to a host `usize` is
[`milesian::MilesianDigit::to_index`] — that is the boundary call
required by I-3.

## Build

```bash
# default no_std + alloc build
cargo build -p algeometric-arc-sigma182-calculi --no-default-features

# with the bridge feature
cargo build -p algeometric-arc-sigma182-calculi --features bridge

# all tests
cargo test  -p algeometric-arc-sigma182-calculi
```

## Forks retired by this crate

`aasc` is the canonical replacement for the trit-pure subsets of:

- `src/kernel/src/ternary.rs` (kernel ternary core)
- `ternary-math/` (host-side ternary math)

Shim plans are tracked in Task #158 steps 13 and 14. Until those shims
land, the forks remain in place and `aasc` is purely additive.

### Pre-shim baseline (Task #161)

Both shim PRs are gated by a recorded pre-shim test pass-count. Those
baselines are now captured by `scripts/capture-shim-baseline.sh` and
the `Shim-Gate Baseline` GitHub Actions workflow
(`.github/workflows/shim-gate-baseline.yml`). The full numbers and
methodology live in `docs/audit/bare-metal-incorporation.md` under
"Pre-shim baseline (Task #161)". Snapshot at time of capture:

| Crate              | Passed | Failed | Ignored | Cargo exit |
|--------------------|-------:|-------:|--------:|-----------:|
| `plenumnet-kernel` |  2,129 |      7 |       0 |        101 |
| `ternary-math`     |    715 |      0 |       0 |          0 |

The 7 recorded `failed` are pre-existing kernel doctest compile
errors that predate Task #161 entirely; they are part of the
recorded baseline so the shim PRs can prove they don't introduce
*new* failures, and they are tracked separately in
`docs/audit/bare-metal-incorporation.md`. The fast `shim-gate`
subset (lib only) — `2,741` passed / `0` failed — is what every PR
touching these crates must keep green; the `full` scope is the
pre-merge gate.

---

© 2025–2026 Capomastro Holdings Ltd. (Canada). Patent(s) Pending —
All Rights Reserved. Applied Physics Division — Salvi Framework.
