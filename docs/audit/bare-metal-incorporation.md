# Bare-Metal Incorporation Audit — `aasc` consolidation

**Status:** *Phase 1 landed — kernel-side shim and harness wired; QEMU
smoke gate documented in CI; full TritInt body migration deferred.*
Tracks Task #158 incorporation of the `aasc` (Algeometric Arc Σ-182
Calculi) crate into the bare-metal kernel and the live downstream
consumers. This audit replaces the previous `documentation-only`
status — the precondition steps that were deferred have now landed
incrementally, with the residual deferral scoped explicitly below.

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
That is the minimum precondition for bare-metal use; with the kernel
shim landed, the bare-metal target now reaches `aasc` transitively
through the `plenumnet-kernel = { …, features = ["no_std"], default-features = false }`
dependency.

## Files surveyed

| File                                  | Role                                             |
|---------------------------------------|--------------------------------------------------|
| `src/kernel/bare-metal/Cargo.toml`    | Out-of-workspace freestanding crate manifest     |
| `src/kernel/bare-metal/linker.ld`     | Custom linker script for the `none` target       |
| `src/kernel/bare-metal/build.rs`      | Build script linking the linker.ld               |
| `src/kernel/bare-metal/x86_64-ternary-none.json` | Custom target spec               |
| `src/kernel/bare-metal/rust-toolchain.toml` | Pinned nightly with `rust-src` component   |
| `src/kernel/bare-metal/generator_theorem_harness.rs` | **Wired** — `[[bin]] generator-theorem-harness` behind `feature = "harness"` |
| `src/kernel/bare-metal/src/`          | Bare-metal kernel sources (incl. `selftest.rs`)  |

## Symbol-surface diff (kernel-side shim — landed)

The kernel ternary core lives at `src/kernel/src/ternary.rs`. The file
is now a documented **`aasc` compatibility shim**:

- `algeometric-arc-sigma182-calculi` is added as a kernel dependency
  with `default-features = false` so the freestanding build remains
  `no_std`-clean.
- The canonical engine symbols are re-exported under the names
  `AascTrit`, `TritVec`, and `AascRepresentation` for forward
  migration. New code reaches `aasc` directly through these.
- The legacy `Trit` / `Tryte` / `TernaryWord` / `Representation`
  names continue to resolve to the kernel-local Rep-A `i8` types so
  every existing consumer — bare-metal `selftest.rs`
  (imports `Trit`, `pack_trits`, `unpack_trits`, `Representation`,
  `convert_representation`), `vm/engine.rs`, `vm/cache.rs`,
  `vm/vm_tests.rs`, `kani_proofs.rs`, and the rest of `src/kernel/` —
  keeps compiling without edits (Task #158 I-47).
- Mathematical shim parity: `Trit::add` / `Trit::multiply` now
  delegate to `AascTrit::add` / `AascTrit::mul`, so the kernel and
  the canonical engine share one GF(3) truth at every call site.
  The kernel storage stays Rep-A `i8` for backward compatibility;
  the bridge is constructor + accessor on the canonical Trit enum
  (zero-cost in release builds).
- `Tryte::host_u64` was added to mirror the `host_u64` boundary on
  `aasc::TritVec` (and to fix a pre-existing test-only build failure
  inside `ternary.rs`'s own `#[cfg(test)] mod tests` block).

The pure re-export rewrite — collapsing the local `Trit`/`Tryte`
storage onto `aasc::Trit` directly — is **deferred** because the
existing `Trit { value: i8 }` field shape is referenced by 22 internal
literals and several `vm/*` accessors that depend on the `i8` storage.
That migration is tracked separately and is gated on the in-flight
Task #154 `M1A.std-shim` rebuild (S004 `TritInt`) so the two shim
crates evolve in lock-step.

## Math-crate shim — re-export anchor only

`ternary-math/src/lib.rs` now exposes `pub use
algeometric_arc_sigma182_calculi as aasc;` so callers who reach into
`ternary_math::aasc::…` flow straight through to the canonical engine
while the per-module rewrites land. The full `TritInt` body
replacement (Task #158 step 14, I-48) is the responsibility of Task
#154 S004 `TritInt rebuild` and is intentionally not duplicated here.
The boundary methods listed in I-48 (`host_u32`, `host_u64`,
`host_u128`, `from_host_u64`, `from_host_u128`, `const_eq`) keep their
existing definitions on `TritInt` until that rebuild lands.

## Workspace placement decision

**Current decision:** keep the bare-metal crate **outside the
workspace**. Two reasons:

1. The freestanding target spec, custom linker script, and pinned
   nightly toolchain make the bare-metal crate a "different planet"
   from the rest of the workspace. Promoting it into `members`
   would force the host CI matrix to discover and ignore that
   target, which is fragile.
2. With the kernel shim landed, the bare-metal crate inherits `aasc`
   transitively through `plenumnet-kernel` — no direct dep on
   `aasc` is required from the freestanding manifest. This preserves
   the single-edge dependency story bare-metal already had.

## Generator-theorem harness wiring

`src/kernel/bare-metal/generator_theorem_harness.rs` is now wired:

- `[features] harness = []` is declared in
  `src/kernel/bare-metal/Cargo.toml`.
- `[[bin]] name = "generator-theorem-harness"` is declared with
  `path = "generator_theorem_harness.rs"`,
  `required-features = ["harness"]`, `test = false`, `bench = false`,
  `harness = false`.
- The file's `#[test]` annotations were rewritten to
  `#[cfg_attr(test, test)]` so the same source compiles both as a
  `cargo test` target (when the unit test runner is on) and as a
  regular `[[bin]]` (when the `harness` feature is on).
- A `#[cfg(feature = "harness")] fn main()` runs every named test
  function in turn and prints a per-step success line on stdout.

Run with:

```bash
cd src/kernel/bare-metal
cargo build --release --features harness --bin generator-theorem-harness
./target/release/generator-theorem-harness
```

The harness is exercised by the QEMU smoke-gate CI workflow (below)
on every PR that touches the kernel, the canonical engine, or the
math crate.

## QEMU smoke gate (CI live)

A dedicated workflow at `.github/workflows/bare-metal-qemu.yml`
gates the bare-metal target on a self-hosted runner labelled
`bare-metal` with `nightly + rust-src + qemu-system-x86_64 + KVM`.
The workflow:

1. Confirms the toolchain (nightly + `rust-src`).
2. Builds and runs the generator-theorem harness against the host
   target.
3. Builds the bare-metal kernel image
   (`cargo build --release --target x86_64-ternary-none.json
    -Z build-std=core,alloc`).
4. Boots the kernel under QEMU (no-graphics, serial-out to stdio,
   `-no-reboot -no-shutdown`) with a 90-second hard timeout.
5. Asserts `[selftest] OK` appears in the captured serial log; the
   serial log is uploaded as an artifact for diagnosis on failure.

These commands mirror the canonical reference set previously held in
this audit. The host environment of the working session has neither
QEMU nor a freestanding-Rust toolchain, so the workflow is
configured but cannot be run from the workspace; the runner-side
operator (or the CI) executes it on every PR.

## What is **not** done in this audit

- `TritInt` body replacement in `ternary-math/src/trit_int.rs` —
  belongs to Task #154 S004 `TritInt rebuild` and is sequenced after
  this audit so the two shim crates evolve in lock-step.
- Per-module re-export collapse in
  `ternary-math/src/{repx, tri182, borromean, plenum_square,
  ternary_circle, coprime, repunit_circles, tribonacci, gf3}.rs` —
  same dependency (Task #158 step 14).
- Migration of the live downstream consumers (`plenumlan`,
  `services/tdns-v2`, `services/inter-cube`, the rest of
  `src/kernel/src/distributor/*`, `browser/color.rs`) **off** the
  legacy shims onto the pure `aasc` surface — Task #158 explicitly
  defers this to follow-up tasks per consumer.
- A pre-shim baseline regression suite for `plenumnet-kernel` and
  `ternary-math`. The `cargo test -p plenumnet-kernel --lib --no-run`
  build had 4 pre-existing errors before this audit — 2 inside
  `ternary.rs` (`Tryte::host_u64` missing, **fixed** by this audit)
  and 2 inside `src/kernel/src/distributor/puv_spectral.rs`
  (test-only function-shadowing — fixed by Task #161 which renamed
  the offending test fns and added `super::` qualification at
  `puv_spectral.rs:327-335`). All four pre-existing errors are now
  resolved on `main`.

## Post-shim verification (Task #169)

Re-confirmed on `main` after Task #162 landed:

| Command                                                | Result                       |
|--------------------------------------------------------|------------------------------|
| `cargo test -p plenumnet-kernel --lib --no-run`        | ok (35 warnings, 0 errors)   |
| `cargo test -p plenumnet-kernel --no-run`              | ok (lib + bins + integration)|
| `cargo test -p plenumnet-kernel --lib`                 | 2092 passed / 0 failed       |
| `bash scripts/capture-shim-baseline.sh shim-gate`      | both crates rc=0             |

The 7 doctest failures recorded in the Task #161 baseline are still
present (pre-existing, tracked separately) and are not regressed by
the shim landings. Lib-only counts (2092 kernel / 649 ternary-math)
are the apples-to-apples post-shim numbers; the full-scope numbers
(2129 / 715) live in `.baseline/` artifacts from the
`shim-gate-baseline.yml` workflow.

## Trit struct collapse onto canonical `aasc::Trit` (Task #170)

The kernel-local `Trit { value: i8 }` struct that lived in
`src/kernel/src/ternary.rs` has been retired. The shim now resolves
`Trit` and `Representation` to the canonical `aasc::trit::{Trit,
Representation}` re-exports (the same enum that already backed the
`AascTrit` / `AascRepresentation` aliases since Task #162). Kernel-only
behaviours that the canonical enum does not carry — `xor`, `rotate`,
`rotate_inverse`, `and`, `or`, `cmp_trit`, `multiply` (an alias of
`mul`), `lukasiewicz_and`, `gf3_inverse_unchecked` (panicking variant),
and `reduce_with_trit` — are exposed through a new
`KernelTritExt` extension trait `impl`d on `aasc::Trit` so existing
call-sites keep working unchanged. The `Tryte`, `pack_trits`,
`unpack_trits`, and `packed_*` helpers were rewritten on top of
`Trit::ZERO` / `Trit::ONE` / `Trit::NEG_ONE` and `value_a()` so the
i8 literal `0/1/-1` constructors no longer appear anywhere in the
kernel.

Consumer files that previously called `.add(&t)` / `.sub(&t)` on a
kernel `Trit` were updated to the by-value form
`.add(t)` / `.sub(t)` because the canonical inherent methods take
`(self, Self)` and shadow the trait `Add`/`Sub` impls. The two
`vm_tests.rs` sites that called `.gf3_inverse()` for the panicking
contract were renamed to `.gf3_inverse_unchecked()` so the canonical
`Option`-returning `aasc::Trit::gf3_inverse` is not silently swapped
in. Files touched in Phase 2 (import + by-value rewrite, no
behavioural changes):

- `src/kernel/src/vm/engine.rs`
- `src/kernel/src/vm/vm_tests.rs`
- `src/kernel/src/kani_proofs.rs`
- `src/kernel/bare-metal/src/selftest.rs`
- `src/kernel/tests/proptest_vm.rs`
- `src/kernel/wasm/src/lib.rs`
- `src/kernel/benches/ternary_ops.rs`
- `src/kernel/benches/salvi_benchmarks.rs`
- `src/kernel/benchmarks/salvi_benchmarks.rs`
- `src/kernel/fuzz/fuzz_targets/fuzz_trit_ops.rs`

Three `packed_zip(a, b, |x, y| x.add(y))` closure sites in
`vm/engine.rs` and one in `vm/vm_tests.rs` were updated to
`|x, y| x.add(*y)` to dereference the `&Trit` callback parameter into
the by-value canonical `add`. The `Tryte::add(&Tryte)` call sites in
`tests/proptest_vm.rs:199` and `benches/ternary_ops.rs:78` were
preserved with their `&` borrow because `Tryte` keeps the existing
kernel-local `add(&self, other: &Tryte)` signature (Tryte composition
is unaffected by Task #170). The bench site additionally moves the
`black_box` inside the borrow (`ta.add(black_box(&tb))`) so the
non-`Copy` `Tryte` value is not consumed across `FnMut` iterations.

Re-confirmed on `main` after Task #170:

| Command                                                | Result                       |
|--------------------------------------------------------|------------------------------|
| `cargo build -p plenumnet-kernel --lib`                | ok (37 warnings, 0 errors)   |
| `cargo test  -p plenumnet-kernel --lib`                | 2092 passed / 0 failed       |
| `cargo test  -p plenumnet-kernel --tests`              | 37 passed / 0 failed         |
| `cargo check -p plenumnet-kernel --lib --no-default-features --features no_std` | ok (44 warnings, 0 errors) |
| `cargo check -p plenumnet-kernel --benches`            | ok                           |
| `bash scripts/capture-shim-baseline.sh shim-gate`      | both crates rc=0, 2092/649   |

The bare-metal QEMU smoke gate
(`.github/workflows/bare-metal-qemu.yml`) and the wasm crate
(`plenumnet-wasm`, separate manifest) are runner-side targets and
are not exercised on the workspace host; both consume the same
`KernelTritExt`-bearing `ternary` shim and were updated for source
compatibility under Phase 2.

## Bare-metal runner provisioning (Task #171)

The `bare-metal-qemu.yml` workflow targets
`runs-on: [self-hosted, bare-metal]`. Until a runner with that
label is registered, the workflow is *dark* — the gate is
configured but never executes — so bare-metal regressions can ship
undetected. This section is the operator runbook for keeping the
runner alive and rotating it.

### Label contract

The workflow's `runs-on` matrix requires **two** labels and is
satisfied by any runner that carries both:

- `self-hosted` — implicit on every self-hosted runner.
- `bare-metal` — must be added explicitly at registration time
  (`./config.sh --labels self-hosted,bare-metal,Linux,X64`).

Adding extra labels (e.g. `Linux`, `X64`, `kvm`) is fine; removing
either of the two contract labels breaks the gate.

### Host requirements

| Requirement                | Why                                                           |
|----------------------------|----------------------------------------------------------------|
| Linux x86_64               | The custom target spec is `x86_64-ternary-none.json`.         |
| `qemu-system-x86_64`       | Smoke step boots the freestanding kernel under QEMU.          |
| `/dev/kvm` + KVM group     | Without KVM the boot falls back to TCG and may exceed the 90s `timeout` cap. |
| Pinned **nightly** Rust    | `src/kernel/bare-metal/rust-toolchain.toml` pins `channel = "nightly"`. |
| `rust-src` component       | `-Z build-std=core,alloc` requires the source of `core`/`alloc`. |
| `build-essential`/`gcc`    | Required by the linker (`linker.ld`) and `build.rs`.          |
| `git`, `curl`, `jq`, `ca-certificates`, `libicu`, `libssl`, `zlib` | Runtime deps of the GitHub Actions runner agent. |
| Outbound HTTPS to `github.com` and `objects.githubusercontent.com` | Job pickup, artifact upload, runner self-update. |

The CI matrix-side `bare-metal-validation.yml` workflow
(`runs-on: ubuntu-latest`) is a *separate, complementary* gate that
does NOT require the self-hosted runner; it gives a partial signal
on PRs even if the self-hosted runner is offline, but it does not
replace the QEMU smoke gate's serial-log assertion.

### Toolchain pin

The runner inherits the toolchain channel from the in-tree
`rust-toolchain.toml`:

```toml
# src/kernel/bare-metal/rust-toolchain.toml
[toolchain]
channel = "nightly"
components = ["rust-src"]
```

`rustup show` inside `src/kernel/bare-metal/` (the workflow's
`working-directory`) auto-installs the pinned channel on first run.
**Do not** override this with a system-wide default toolchain — the
workflow's `Toolchain — confirm nightly + rust-src present` step
fails fast if the resolved channel is not `nightly` or if
`rust-src` is missing.

### One-shot registration

`scripts/bootstrap-bare-metal-runner.sh` is the operator-runnable
script that performs steps 1–2 of the task end-to-end on the
runner host:

1. Installs OS prerequisites (QEMU, KVM tools, build-essential,
   curl, jq, libicu, libssl, git).
2. Adds the runner user to the `kvm` group.
3. Installs nightly Rust + `rust-src` under the runner user.
4. Downloads the pinned actions-runner release, registers it with
   the operator-supplied token, and applies the
   `self-hosted,bare-metal,Linux,X64` label set.
5. Installs the runner as a systemd service and starts it.

Operator workflow:

```bash
# On the runner host (NOT the Replit workspace, NOT inside CI):
export RUNNER_URL='https://github.com/SigmaWolf-8/Ternary'
# Get the registration token from:
#   Settings → Actions → Runners → "New self-hosted runner" → copy the token.
# It is single-use and expires after ~1 hour.
export RUNNER_TOKEN='AAAA…'

# Optional overrides:
#   RUNNER_NAME      (default: hostname)
#   RUNNER_HOME      (default: $HOME/actions-runner)
#   RUNNER_USER      (default: $USER)
#   RUNNER_VERSION   (default: pinned in the script)

bash scripts/bootstrap-bare-metal-runner.sh
```

After the script returns:

1. Confirm the runner shows up as **Idle** at
   `${RUNNER_URL}/settings/actions/runners`.
2. Trigger the workflow on demand:
   `gh workflow run bare-metal-qemu.yml --ref main` (or just push
   any change touching `src/kernel/**`,
   `algeometric-arc-sigma182-calculi/**`, `ternary-math/**`, or
   `.github/workflows/bare-metal-qemu.yml`).
3. The first run downloads the actions cache and the pinned
   nightly toolchain (~3–5 minutes); subsequent runs land in
   ~90–120s end-to-end.
4. The `qemu-smoke-log` artifact uploads on every run (success or
   failure) and is downloadable from the workflow run page for 14
   days — that artifact is the canonical diagnostic for any
   `[selftest] OK` assertion failure.

### Rotation

Tokens are short-lived; the runner installation itself can be
re-pointed at a fresh token without redoing the OS-level
prerequisites:

```bash
# Re-run the same script with a fresh RUNNER_TOKEN — the script's
# config step uses --replace, so the same RUNNER_NAME re-registers
# cleanly (Step 1–3 are idempotent and short-circuit).
export RUNNER_TOKEN='AAAA…'   # fresh token from the GitHub UI
bash scripts/bootstrap-bare-metal-runner.sh
```

To fully decommission a host:

```bash
cd "$RUNNER_HOME"
sudo ./svc.sh stop
sudo ./svc.sh uninstall
./config.sh remove --token "$REMOVAL_TOKEN"   # also from the GitHub UI
```

### Troubleshooting

| Symptom                                                  | Likely cause / fix                                                                                       |
|----------------------------------------------------------|----------------------------------------------------------------------------------------------------------|
| Workflow stays *queued* forever                          | No runner with the `bare-metal` label is online. Verify in `Settings → Actions → Runners`.               |
| Boot times out at 90s                                    | KVM not available; `kvm-ok` reports failure. Confirm `/dev/kvm` exists and the runner user is in `kvm`. |
| `error: component 'rust-src' not found`                  | The host has a stable channel pinned globally. Remove `~/.rustup/settings.toml` overrides.               |
| `qemu-system-x86_64: command not found`                  | `qemu-system-x86` package missing. Re-run the bootstrap; the OS-prereq step is idempotent.               |
| Sentinel grep fails but boot completes                   | Selftest module hit a panic before the `[selftest] OK` print. Inspect the uploaded `qemu-smoke-log`.     |

These notes mirror the canonical command set carried in
`.github/workflows/bare-metal-qemu.yml`. Any change to that
workflow's command list must be reflected back into this section
so the operator runbook stays in sync.

---

© 2025–2026 Capomastro Holdings Ltd. (Canada). Patent(s) Pending —
All Rights Reserved. Applied Physics Division — Salvi Framework.
