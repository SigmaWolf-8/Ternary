# PlenumNET Ternary Hardware Description Language (THDL)

Hardware description language for synthesizing ternary logic circuits.

Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

## Overview

THDL enables the design and synthesis of ternary computing hardware targeting FPGA and ASIC implementations. It includes optimization passes specifically designed for ternary logic.

## Features

- **Ternary Gate Primitives**: Native support for trit operations
- **Multi-Target Synthesis**: Xilinx, Intel, Lattice FPGAs and custom ASICs
- **Optimization Passes**: Ternary-specific optimizations
- **Timing Analysis**: Femtosecond-precision timing constraints

## Synthesis Targets

| Target | Description |
|--------|-------------|
| `XilinxFpga` | Xilinx FPGA family (Artix, Kintex, Virtex) |
| `IntelFpga` | Intel/Altera FPGA family |
| `LatticeFpga` | Lattice FPGA family |
| `Asic` | Custom ASIC with standard cell library |
| `Simulation` | Simulation-only with testbench |

## Optimization Passes

### Constant Folding
Evaluates constant ternary expressions at compile time using GF(3) arithmetic.

### Dead Code Elimination
Removes unused signals and assignments.

### Common Subexpression Elimination
Identifies and reuses duplicate computations.

### Ternary-Specific Optimizations
- Triple rotation elimination: `rot(rot(rot(x))) = x`
- Double NOT elimination: `not(not(x)) = x`
- Identity elimination: `x + 0 = x`, `x * 1 = x`
- Zero multiplication: `x * 0 = 0`

### Timing Optimization
Restructures logic to meet timing constraints, adding pipeline stages as needed.

## Usage

```rust
use plenumnet_thdl::{synthesize, SynthesisOptions, Target};

let options = SynthesisOptions {
    target: Target::XilinxFpga,
    optimize_area: true,
    optimize_speed: true,
    ..Default::default()
};

let result = synthesize(thdl_source, &options)?;
println!("Gates: {}", result.statistics.gates);
println!("Critical path: {}ps", result.statistics.critical_path_ps);
```

## Ternary Cell Library

THDL includes a standard cell library for ternary operations:

| Cell | Function | Description |
|------|----------|-------------|
| `TRIT_ADD` | a + b (mod 3) | GF(3) addition |
| `TRIT_MUL` | a × b (mod 3) | GF(3) multiplication |
| `TRIT_NOT` | -a | Ternary negation |
| `TRIT_ROT` | rotate(a) | Bijective rotation |
| `TRIT_XOR` | min(a, b) | Ternary XOR |

## See Also

- [TSL Documentation](../tsl/README.md)
- [Kernel API](../kernel/README.md)
