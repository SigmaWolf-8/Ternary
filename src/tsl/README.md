# PlenumNET Ternary System Language (TSL)

A high-level programming language for ternary computing systems.

Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

## Overview

TSL is designed for writing software that runs on ternary computing hardware. It compiles to THDL (Ternary Hardware Description Language) or directly to ternary bytecode.

## Language Features

### Types

| Type | Description | Size |
|------|-------------|------|
| `trit` | Single ternary digit | {-1, 0, +1} |
| `tryte` | 6 trits | 729 values |
| `word` | 27 trits | Ternary word |

### Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | GF(3) addition | `a + b` |
| `*` | GF(3) multiplication | `a * b` |
| `~` | Ternary NOT | `~x` |
| `^` | Ternary XOR (min) | `a ^ b` |
| `>>>` | Bijective rotation | `>>> x` |
| `<<<` | Inverse rotation | `<<< x` |

### Keywords

- `fn` - Function definition
- `let` - Variable declaration
- `if/else` - Conditional
- `while` - Loop
- `return` - Return statement
- `timing` - Access femtosecond clock
- `phase` - Phase encryption operations

## Example

```tsl
// Ternary addition in GF(3)
fn add_trits(a: trit, b: trit) -> trit {
    return a + b;
}

// Main computation
fn main() -> trit {
    let x: trit = 1;
    let y: trit = -1;
    return add_trits(x, y);
}
```

## Compiling

```bash
# Compile to THDL
tsl compile program.tsl -o program.thdl

# Compile to bytecode
tsl compile program.tsl -o program.tbc --target=bytecode

# Compile for FPGA
tsl compile program.tsl -o program.bit --target=fpga
```

## See Also

- [TSL Examples](examples/)
- [THDL Documentation](../thdl/README.md)
- [Kernel API](../kernel/README.md)
