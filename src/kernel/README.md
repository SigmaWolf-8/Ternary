# PlenumNET Salvi Framework Kernel

Post-Quantum Ternary Computing Core implementing the Salvi Framework specification.

Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

## Features

- **Ternary Logic Operations**: GF(3) arithmetic with bijective representations
- **Femtosecond Timing**: FINRA Rule 613 CAT compliant timestamps
- **Phase Encryption**: Quantum-resistant phase-split encryption
- **Multi-Architecture**: x86_64, aarch64, riscv64, FPGA, ASIC support

## Salvi Epoch

All timestamps reference the Salvi Epoch:
- **Day Zero**: April 1, 2025 00:00:00.000 UTC
- **Unix NS**: 1,743,465,600,000,000,000

## Building

```bash
# Standard build
cargo build --release

# With FINRA compliance
cargo build --release --features finra-613

# For FPGA targets
cargo build --release --features fpga --target thumbv7em-none-eabi
```

## Modules

### `timing`
Femtosecond-precision timestamps with FINRA Rule 613 compliance.

```rust
use plenumnet_kernel::timing::{FemtosecondTimestamp, validate_recombination_window};

let ts = FemtosecondTimestamp::new(1_000_000_000_000_000);
println!("Timestamp: {}", ts.format());
```

### `ternary`
GF(3) arithmetic with three representations (A, B, C).

```rust
use plenumnet_kernel::ternary::{Trit, Tryte, Representation};

let a = Trit::from_a(1).unwrap();
let b = Trit::from_a(-1).unwrap();
let result = a.add(&b);  // 0 in GF(3)
```

### `phase`
Quantum-resistant phase-split encryption.

```rust
use plenumnet_kernel::phase::{split_data, recombine_data, EncryptionMode};

let data = b"Sensitive data";
let split = split_data(data, EncryptionMode::HighSecurity, timestamp);
let original = recombine_data(&split).unwrap();
```

## License

MIT License - See LICENSE file for details.
