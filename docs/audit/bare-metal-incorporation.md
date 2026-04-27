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
`cargo test -p plenumnet-kernel` run is recorded in CI; that work is
deferred to a follow-up task.

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

## Deferral rationale

The "Done When" gate of Task #158 explicitly permits the kernel and
ternary-math shims to be deferred *with rationale documented*. The
rationale is:

- `aasc` is purely additive at this stage. No existing crate
  changes its public surface.
- Both shims (steps 13, 14) require a recorded
  pre-shim baseline test pass-count for `plenumnet-kernel` and
  `ternary-math` to certify "no regression". Capturing that
  baseline is itself a non-trivial follow-up because the
  workspace-level `cargo test` is currently dominated by
  long-running integration tests that need quarantining.
- The bare-metal incorporation (this audit, step 15) chains off the
  kernel shim and inherits its deferral.

Until those follow-ups land, `aasc` lives next to (not inside) the
existing crates, every existing import path keeps working, and the
bare-metal kernel continues to build with its pre-task ternary core.

---

© 2025–2026 Capomastro Holdings Ltd. (Canada). Patent(s) Pending —
All Rights Reserved. Applied Physics Division — Salvi Framework.
