# task-133 — RepX Acceptance Checklist

This file tracks the 14 G-gates of task-133. The current task delivered the
**foundation slice** (rename + workspace wiring + deprecation shim + new
constants + 39-register grid + sealed-trait skeletons). The remaining work
is decomposed into follow-up tasks; each is a multi-day implementation in
its own right.

## Pre-rename SHA
- [x] `tests/repx_acceptance/PRE_REPX_SHA` captured (`860a6fa…`).

## What landed in this task
- [x] `ternary-math/src/gf3_algebra.rs` → `ternary-math/src/repx.rs`
      (rename done with `mv`; `git mv` not used because version control is
      platform-managed in this environment, see drift note in commit message).
- [x] `lib.rs` declares `pub mod repx;` plus `#[deprecated(since = "0.2.0",
      note = "use \`repx\`")] pub use crate::repx as gf3_algebra;`.
- [x] Workspace members added: `repx-cli`, `repx-ffi`, `xtask` (stubs).
- [x] Downstream call-site sweep: `trit.rs`, `trit_int.rs`,
      `services/inter-cube/src/relay_frames.rs`,
      `services/inter-cube/tests/relay_integration.rs`,
      `plenumlan/src/cube/freshness.rs`,
      `plenumlan/src/cube/projection.rs` updated to import via `repx`.
- [x] `constants.rs` additions: `T_36`, `ALPHA_INV_INT`,
      `RYDBERG_NUM_TM017`, `KAPPA_BRIDGE_NUM`, `KAPPA_BRIDGE_DEN`,
      `MILNOR_NUMBER`, `RHO_0`, `GAMMA_0`, `C_LIGHT`, `kappa_ep()`.
- [x] 39-cell `REGISTERS` grid with pentadic Element tags, post-OT-1
      π_fw forms, OT-1f / OT-1e / OT-1a notes on flagged cells.
- [x] Lookup helpers: `at()`, `registers_for_element()`,
      `registers_for_archetype()`.
- [x] Sealed-trait skeletons + concrete `Engine` implementation:
      `RepXTransferFunction`, `RepXTransform`, `ComposeOp` enum;
      `Engine` provides `read_si`, `convert_si`, `compose_si_op`,
      `period_natural`, `precession_natural`, `ot1c_delta_over_a`,
      `omega_from_frequency`, `c_natural`.
- [x] Worked-example tests pass (in-module
      `ternary-math/src/repx.rs::engine_tests`):
      1 AU Sun-Gaia chain (Γ ≈ 5.93×10⁻³ m/s²), Luna-Gaia nested
      fulcrum (Γ ≈ 2.7×10⁻³ m/s²), tropical-year framework Kepler
      ratio = ROOT_X1/π, axial-precession great cycle =
      ROOT_X2 · LCM_PRIMARY = 26 026, OT-1c residual EP
      Δa/a ≈ 7.41×10⁻¹¹, Sun→Moon `rotate` routes through Gaia,
      `compose(ChainMultiply)` evaluates in ℤ[ω].
- [x] Grid invariant tests `ternary-math/tests/repx_engine.rs`:
      39-cell shape, 13 rows per archetype, pentadic counts
      (Fire 8 / Water 7 / Air 6 / Earth 7 / Aether 8 / Unmapped 3),
      Quintessence F4 reconciliation (Moon #8 / #9 = Water),
      unmapped set = {Sun #8, Sun #9, Gaia #5}, Sun #1 horn form,
      Gaia #7 OT-1f note, Moon #8 Stokes drag, Moon #13 tidal,
      bare-π-literal sweep (no `14`/`28`/`56`/`84` in formulas),
      deprecation-shim resolution.

## What remains (per-gate follow-ups)

### G1 — Build & lint hardening
- [ ] Workspace-level `RUSTFLAGS="-D warnings"`, `clippy::pedantic`,
      `cargo doc` broken-link check, `cargo deny`.

### G2 — Rename-safety scripts
- [ ] `scripts/repx_rename_sweep.sh`, `repx_cargo_sweep.sh`,
      `repx_doc_sweep.sh` (hard-coded ignore lists).
- [ ] `cargo public-api` surface diff against `PRE_REPX_SHA`.

### G3 — 1521-pair interchange matrix
- [ ] Engine façade (`Engine::convert(seed_reading(R_src), R_tgt)`).
- [ ] `tests/repx_acceptance/budgets.rs` per-pair + sweep budgets,
      `scripts/repx_sweep_watchdog.sh`.

### G4 — Symbol-Map AST lint (pinned `syn = "=2.0.x"`)
- [ ] `tests/repx_symbol_map.rs` with the `// SI-ANCHOR:` grammar.

### G5 — Backend bit-identity
- [ ] `proptest` harness with seed pin `0x5EED_FED_BEE_F`, 10 000 cases
      per backend operation, snapshot files under `tests/snapshots/`.

### G6 — Worked-example acceptance
- [ ] 1 AU emission via Gaia chain, Luna-Gaia via NestedFulcrum,
      tropical year (0.27 ppm), precession `ROOT_X2 * LCM_PRIMARY`,
      OT-1c residual EP delta (test against framework band only;
      LLR comparison deferred).

### G7 — Doc-test & cookbook
- [ ] Five runnable doc examples in `repx.rs` module docs.
- [ ] `repx-cli` subcommands: `describe / read / find / convert`.

### G8 — FFI / JSON surface
- [ ] `repx_describe_json`, `repx_read`, `repx_convert`,
      `repx_find_json` with `catch_unwind` wrappers and documented
      error codes.
- [ ] C smoke harness, `tests/ffi/cc.lock` toolchain pin.

### G9 — Determinism & reproducibility
- [ ] `scripts/repx_double_run.sh` (byte-identity across two runs).
- [ ] Lambert-W / Newton inverter contracts.
- [ ] `SOURCE_DATE_EPOCH`, locale, `--remap-path-prefix` hygiene.

### G10 — Downstream consumer smoke
- [ ] `scripts/repx_downstream_sweep.sh` reading `[workspace.members]`.
- [ ] `scripts/test_count_diff.sh` pre-vs-post test count parity.

### G11 — CI wiring
- [ ] `cargo xtask repx-zero-error` orchestrates G1–G14.
- [ ] CI matrix: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
      `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`.

### G12 — Pre-merge checklist
- [ ] This file pasted into the merge commit message with every box
      checked.

### G13 — Crypto byte-identity (CRITICAL)
- [ ] `tests/repx_acceptance/CRYPTO_CALL_SITES.txt` enumerated.
- [ ] BLAKE3-addressed `tests/snapshots/crypto/` corpus.
- [ ] Pre-rename signed-message verification with post-rename verifier.
- [ ] Rep-C boundary AST verification (no `Rep*` in crypto signatures).

### G14 — Workspace wiring (verified at build time today)
- [x] `repx-cli`, `repx-ffi`, `xtask` listed in root `Cargo.toml`.
- [ ] `toml_edit` parser test + `cargo metadata` member-count delta.

## Drift note
- The spec mandates a true `git mv` rename (similarity ≥ 90%). Per this
  environment's rules of engagement, version control is platform-managed
  and explicit `git` invocations are not permitted from the agent. The
  rename was performed with `mv`; the platform records this commit and the
  similarity score depends on the platform's snapshot logic. A real
  `git mv` may need to be re-applied during a follow-up if G2 reports
  the rename as add+delete.
- The spec requires `panic = "abort"` on `repx-ffi`'s `[profile.dev]`.
  Cargo only honours profile overrides at the workspace root; doing so
  affects every workspace member and is deferred to a workspace-policy
  follow-up.
