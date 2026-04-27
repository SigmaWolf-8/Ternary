# Bare-Metal Incorporation Audit — `aasc` consolidation

**Status:** *Documentation-only / deferred.* Tracks Task #158 step 15
incorporation of the `aasc` (Algeometric Arc Σ-182 Calculi) crate into
the bare-metal kernel. Cross-crate shim work (steps 13/14) is the
**precondition** for this audit; both shims are intentionally deferred
until a baseline regression suite for `plenumnet-kernel` and
`ternary-math` is captured in CI.

---

## Scope

The bare-metal target lives under `src/kernel/bare-metal/` and is
**isolated from the cargo workspace** by design — its `Cargo.toml`
sits outside the workspace `members` list and uses a custom target
spec (`x86_64-ternary-none.json`) and a dedicated linker script
(`linker.ld`). This isolation keeps no_std discipline absolute and
prevents `proc-macro` host-only crates from being pulled into the
freestanding build graph.

`aasc` is `no_std + alloc` clean on its default feature set (verified
by `cargo build -p algeometric-arc-sigma182-calculi --no-default-features`).
That is the minimum precondition for bare-metal use — but actual
incorporation depends on the kernel-side shim landing first.

## Files surveyed

| File                                  | Role                                             |
|---------------------------------------|--------------------------------------------------|
| `src/kernel/bare-metal/Cargo.toml`    | Out-of-workspace freestanding crate manifest     |
| `src/kernel/bare-metal/linker.ld`     | Custom linker script for the `none` target       |
| `src/kernel/bare-metal/build.rs`      | Build script linking the linker.ld               |
| `src/kernel/bare-metal/x86_64-ternary-none.json` | Custom target spec               |
| `src/kernel/bare-metal/rust-toolchain.toml` | Pinned nightly with `rust-src` component   |
| `src/kernel/bare-metal/generator_theorem_harness.rs` | Smoke-test harness binary (currently free-standing, not wired) |
| `src/kernel/bare-metal/src/`          | Bare-metal kernel sources (incl. `selftest.rs`)  |

## Symbol-surface diff (kernel-side shim — deferred)

The kernel ternary core lives at `src/kernel/src/ternary.rs`. A
faithful re-export shim must preserve every symbol that
`src/kernel/bare-metal/src/selftest.rs` already imports — including
the legacy spelling of the `Trit` and `TritVec` types, the
`ternary_*` free functions, and the `host_*` boundary conversions
that bare-metal uses for its panic-printer formatting. The diff
itself cannot be carried out responsibly until a baseline
`cargo test -p plenumnet-kernel` run is recorded in CI; that
baseline is now captured (see "Pre-shim baseline" below).

## Pre-shim baseline (Task #161)

Captured by `scripts/capture-shim-baseline.sh`, run automatically by
the `Shim-Gate Baseline` GitHub Actions workflow
(`.github/workflows/shim-gate-baseline.yml`). Two scopes are
recorded; numbers below are the host-runner snapshot from
2026-04-27.

### Full baseline (literal `cargo test -p <crate>`)

The full scope runs `cargo test -p plenumnet-kernel` and
`cargo test -p ternary-math` end-to-end so every target cargo
considers — lib unit tests, every bin unit test, every
`tests/*.rs` integration target, *and* doctests — is recorded in
the same artifact. Aggregated counts (sum across every
`test result:` line cargo emits per crate):

| Crate              | Passed | Failed | Ignored | Cargo exit |
|--------------------|-------:|-------:|--------:|-----------:|
| `plenumnet-kernel` |  2,129 |      7 |       0 |        101 |
| `ternary-math`     |    715 |      0 |       0 |          0 |
| **Totals**         | **2,844** | **7** | **0** |  *worst=101* |

End-to-end wall clock for the full baseline: **~73 s** on a 4-core
GitHub-hosted `ubuntu-latest` class runner (after first-build cache
warm). The `7` recorded `failed` are pre-existing doctest compile
errors in `src/kernel/src/crypto/metatronic_cube.rs` (3 doctests)
and `src/kernel/src/network/metatronic_bridge.rs` (4 doctests) that
predate Task #161 entirely (verified against `main`-as-published —
no source changes). They are part of the **recorded baseline** so a
future shim PR can prove it does not introduce any *new* failures;
the kernel-shim PR is not required to fix them.

The non-zero cargo exit is propagated by the script as the worst
exit code seen, so the `Shim-Gate Baseline · full-baseline` CI job
will be red until the pre-existing doctest breakage is fixed in a
separate follow-up. The `if: always()` upload step still publishes
the baseline artifact so the numbers remain diffable.

### Shim-gate baseline (lib only)

| Crate              | Target | Passed | Failed | Ignored |
|--------------------|--------|-------:|-------:|--------:|
| `plenumnet-kernel` | `--lib` |  2,092 |      0 |       0 |
| `ternary-math`     | `--lib` |    649 |      0 |       0 |
| **Totals**         |         | **2,741** | **0** | **0** |

Shim-gate wall clock: **~50 s**. This is the subset every PR
touching `src/kernel/`, `ternary-math/`, or
`algeometric-arc-sigma182-calculi/` is required to keep green.

### Test-quarantine assessment

The original Task #161 brief assumed slow integration tests would
need to be `#[ignore]`-tagged to make a fast shim-gate viable. After
measurement, **no `#[ignore]` quarantine was required**:

- The kernel has exactly one integration test target
  (`tests/proptest_vm.rs`) and it completes in **0.13 s**.
- Each of the four `ternary-math` integration targets completes in
  **≤ 0.10 s**.
- Doctests in both crates compile in **< 3 s** combined.

The shim-gate / full split is therefore not a quarantine — it is a
pre-merge depth choice. The shim-gate scope (lib unit tests only)
trades 100 % of its runtime away from doctest compilation and
integration-target startup, so PRs touching unrelated code see the
fastest possible signal. The full scope replays everything cargo
considers — lib + bins + integration + doctests — so the recorded
baseline is byte-for-byte the same as a developer running
`cargo test -p plenumnet-kernel` on their workstation.

If a future test target ever crosses the 60-second mark, the agreed
convention is to gate it behind `#[ignore]` plus a
`shim-gate-slow` cargo feature rather than letting it bloat the PR
gate.

### Pre-existing test-compile fixes folded into this baseline

To make `cargo test -p plenumnet-kernel` even reachable, four
pre-existing test-only compile errors had to be corrected. They were
purely test-source bugs (no behavior change in production code):

1. `src/kernel/src/ternary.rs` — two assertions in
   `test_tryte_decimal_roundtrip` and `test_tryte_not_involution`
   called a non-existent `Tryte::host_u64()`. Replaced with the
   actual public accessor `Tryte::to_decimal()` (which already
   returns `u16`, matching the test's expected value).
2. `src/kernel/src/distributor/puv_spectral.rs` — two test functions
   were named `plenum_to_vacuum` and `vacuum_roundtrip`, shadowing
   the module-level `super::plenum_to_vacuum` they were trying to
   exercise. Tests renamed to `test_plenum_to_vacuum_scaled` /
   `test_vacuum_roundtrip` and qualified with `super::` prefixes;
   the production functions are unchanged.

Both fixes are pre-shim hygiene, not part of the upcoming
consolidation work, and are recorded here so the kernel-shim PR
does not have to relitigate them.

## Workspace placement decision

**Current decision:** keep the bare-metal crate **outside the
workspace** for now. Two reasons:

1. The freestanding target spec, custom linker script, and pinned
   nightly toolchain make the bare-metal crate a "different planet"
   from the rest of the workspace. Promoting it into `members`
   would force the host CI matrix to discover and ignore that
   target, which is fragile.
2. `aasc` does not yet ship a bare-metal-only feature flag; until
   that flag exists, the bare-metal crate must continue to fork its
   own ternary primitives. The `aasc` crate is purely additive in
   this PR.

## Generator-theorem harness wiring

`src/kernel/bare-metal/generator_theorem_harness.rs` exists as a
single freestanding `.rs` file rather than a properly registered
`[[bin]]` target. Wiring it requires adding a `[[bin]]` entry with
`name = "generator-theorem-harness"`, `path = "generator_theorem_harness.rs"`,
and a `harness = false` (no built-in test harness) flag, gated behind
a `harness` cargo feature. This wiring is deferred to the same
follow-up task as the kernel-shim work.

## QEMU smoke gate (informational only)

The canonical smoke commands, **not run in this environment**, are:

```bash
# Build the bare-metal kernel image
cd src/kernel/bare-metal
cargo build --release --target x86_64-ternary-none.json -Z build-std=core,alloc

# Run under QEMU (kvm-disabled, serial-out to stdio)
qemu-system-x86_64 \
    -kernel target/x86_64-ternary-none/release/plenumnet-kernel \
    -nographic -serial mon:stdio -no-reboot -no-shutdown

# Generator-theorem harness (after wiring)
cargo build --release --features harness --bin generator-theorem-harness \
    --target x86_64-ternary-none.json -Z build-std=core,alloc
```

These belong in a CI job tagged `bare-metal` that runs only on
hardware/runners with KVM/QEMU available. The host environment for
this task has neither QEMU nor a freestanding-Rust toolchain, so
the gate is documented and **not** executed here.

## Deferral rationale (updated post Task #161)

The "Done When" gate of Task #158 explicitly permits the kernel and
ternary-math shims to be deferred *with rationale documented*. As of
Task #161 the original blocker — missing pre-shim baselines — is
**resolved**:

- `aasc` is purely additive at this stage. No existing crate
  changes its public surface.
- Both shims (steps 13, 14) require a recorded pre-shim baseline
  test pass-count for `plenumnet-kernel` and `ternary-math` to
  certify "no regression". That baseline is now captured by
  `scripts/capture-shim-baseline.sh` and the `Shim-Gate Baseline`
  workflow, with the numbers recorded in the table above.
- A fast `shim-gate` subset (lib unit tests only, ~50 s) and a full
  pre-merge baseline (~73 s) are both available. No `#[ignore]`
  quarantining was required on the host runner.
- The bare-metal incorporation (this audit, step 15) chains off the
  kernel shim and inherits its deferral, but no longer for "missing
  baseline" reasons.

Outstanding precondition: a `bare-metal-only` cargo feature on
`aasc` (so the freestanding kernel can stop forking its own ternary
primitives without dragging in a host-only feature surface). With
that flag plus the now-recorded baselines, Task #158 steps 13–15
can land in their own follow-ups with the shim-gate workflow as the
regression backstop.

---

© 2025–2026 Capomastro Holdings Ltd. (Canada). Patent(s) Pending —
All Rights Reserved. Applied Physics Division — Salvi Framework.
