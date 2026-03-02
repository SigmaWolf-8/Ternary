# PlenumNET Bare-Metal Boot Target

**Salvi Framework** · Capomastro Holdings Ltd. · Applied Physics Division

Bare-metal boot target for the PlenumNET Ternary Kernel. This binary boots on raw x86_64 hardware (via QEMU) with **no operating system**, imports the **actual `plenumnet-kernel` library**, and runs a comprehensive self-test suite.

## What This Validates

| Claim | Evidence |
|-------|----------|
| **Kernel boots without an OS** | `_start` entry, custom linker layout, bare-metal allocator |
| **GF(3) trit arithmetic works** | All Trit operations: NOT, AND, OR, add, multiply, rotate |
| **Bijective representations correct** | A↔B↔C round-trip conversions verified |
| **Packed trit words round-trip** | 9-trit and 27-trit pack/unpack verified |
| **Boot sequence completes** | All 11 BootStages advance from PowerOn to Running |
| **x86_64 boot config valid** | Physical base at 0x100000, memory map populated |
| **Femtosecond timing types work** | Timestamp arithmetic, FINRA constants, Salvi Epoch |
| **Phase encryption types valid** | Mode selection, phase counts, golden ratio split |

## This Uses the Real Kernel

```rust
use plenumnet_kernel::ternary::{Trit, pack_trits, unpack_trits};
use plenumnet_kernel::arch::boot::{BootSequence, BootStage};
use plenumnet_kernel::timing::FemtosecondTimestamp;
use plenumnet_kernel::phase::EncryptionMode;
```

## Quick Start

```bash
cd src/kernel/bare-metal
rustup toolchain install nightly --component rust-src
sudo apt install qemu-system-x86
chmod +x scripts/*.sh
./scripts/build.sh
./scripts/qemu-run.sh target/x86_64-ternary-none/debug/ternary-kernel
```

## License

Proprietary — Capomastro Holdings Ltd.
