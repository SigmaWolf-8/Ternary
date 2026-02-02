# TSL Examples

Ternary System Language (TSL) example programs for PlenumNET.

Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

## Examples

### hello_ternary.tsl
Basic ternary operations demonstrating:
- Trit addition and multiplication in GF(3)
- Ternary NOT operation
- Bijective rotation

### tryte_math.tsl
Tryte-level mathematics:
- 6-trit operations (729 possible values)
- Carry propagation
- Polynomial multiplication

### phase_encryption.tsl
Quantum-resistant phase-split encryption:
- Primary/secondary phase splitting
- Recombination with timing verification
- Golden ratio split for high security

### femtosecond_timing.tsl
FINRA Rule 613 CAT compliant timing:
- Femtosecond precision timestamps
- Salvi Epoch (April 1, 2025) reference
- 50ms compliance checking
- Recombination window validation (100fs)

## Compiling

```bash
# Compile to THDL (hardware description)
tsl compile hello_ternary.tsl -o hello_ternary.thdl

# Compile to ternary bytecode
tsl compile hello_ternary.tsl -o hello_ternary.tbc --target=bytecode

# Compile for FPGA
tsl compile hello_ternary.tsl -o hello_ternary.bit --target=fpga
```

## Language Features

### Types
- `trit` - Single ternary digit: {-1, 0, +1}
- `tryte` - 6 trits (729 values)
- `word` - 27 trits (ternary word)

### Operators
- `+` - Ternary addition (GF(3))
- `*` - Ternary multiplication (GF(3))
- `~` - Ternary NOT (negation)
- `^` - Ternary XOR (minimum)
- `>>>` - Bijective rotation
- `<<<` - Inverse rotation

### Special Keywords
- `timing` - Access femtosecond clock
- `phase` - Phase encryption operations
