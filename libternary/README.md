# libternary

**Salvi Framework Core Library** - Bijective Ternary Logic for Post-Quantum Applications

A TypeScript/JavaScript library implementing the Unified Ternary Logic System from the Salvi Framework whitepaper for post-quantum cryptographic applications.

## Features

- **Three Bijective Representations** (A, B, C) with seamless conversions
- **GF(3) Ternary Operations** - Addition, Multiplication, Rotation, XOR, NOT
- **Femtosecond Timestamp Generation** - 10⁻¹⁵ second precision timestamps
- **Phase-aware Encryption** - Adaptive dual-phase quantum encryption system

## Installation

```bash
npm install libternary
```

## Quick Start

```typescript
import {
  convertTrit,
  ternaryAdd,
  getFemtosecondTimestamp,
  phaseSplit,
  phaseRecombine
} from 'libternary';

// Convert between representations
const result = convertTrit(0, 'A', 'B');
console.log(result);
// { original: { value: 0, representation: 'A', meaning: 'Neutral' },
//   converted: { value: 1, representation: 'B', meaning: 'Neutral' },
//   bijection: 'f(a) = a + 1' }

// Ternary addition in GF(3)
const sum = ternaryAdd(-1, 1);
console.log(sum.result); // 0

// Get femtosecond timestamp
const timestamp = getFemtosecondTimestamp();
console.log(timestamp.humanReadable);

// Phase-split encryption
const encrypted = phaseSplit('Sensitive data', 'high_security');
const decrypted = phaseRecombine(encrypted);
console.log(decrypted.data); // 'Sensitive data'
```

## Ternary Representations

The library supports three bijective representations as defined in the whitepaper:

| Representation | Values | Use Case |
|---------------|--------|----------|
| **A** (Computational) | {-1, 0, +1} | Internal computations |
| **B** (Network) | {0, 1, 2} | Network transmission |
| **C** (Human) | {1, 2, 3} | User interfaces |

### Bijection Formulas

- **A→B**: `f(a) = a + 1`
- **A→C**: `f(a) = a + 2`
- **B→C**: `f(b) = b + 1`
- **B→A**: `f(b) = b - 1`
- **C→A**: `f(c) = c - 2`
- **C→B**: `f(c) = c - 1`

## GF(3) Operations

```typescript
import { ternaryAdd, ternaryMultiply, ternaryRotate, ternaryNot } from 'libternary';

// Addition: a ⊕₃ b = (a + b) mod 3
ternaryAdd(-1, 1);  // Result: 0

// Multiplication: a ⊗₃ b = (a × b) mod 3
ternaryMultiply(1, 1);  // Result: 1

// Rotation: TBR(θ)
ternaryRotate(0, 1);  // Rotate by 1 step

// Negation
ternaryNot(1);  // Result: -1
```

## Femtosecond Timing

```typescript
import { getFemtosecondTimestamp, calculateDuration } from 'libternary';

const start = getFemtosecondTimestamp();
// ... some operation ...
const end = getFemtosecondTimestamp();

const duration = calculateDuration(start, end);
console.log(duration.humanReadable);
```

## Phase Encryption

```typescript
import { phaseSplit, phaseRecombine, getPhaseConfig } from 'libternary';

// Get configuration for different security modes
const config = getPhaseConfig('high_security');
// { mode: 'high_security', primaryPhase: 0, secondaryOffset: 10,
//   guardianEnabled: true, guardianOffset: 358 }

// Split and encrypt data
const encrypted = phaseSplit('Secret message', 'balanced');

// Recombine and decrypt
const result = phaseRecombine(encrypted);
if (result.success) {
  console.log(result.data); // 'Secret message'
}
```

### Encryption Modes

| Mode | Timing Tolerance | Guardian Phase | Use Case |
|------|-----------------|----------------|----------|
| `high_security` | 100 fs | Enabled | Quantum-resistant apps |
| `balanced` | 1 ms | Disabled | General use |
| `performance` | 1 s | Disabled | High throughput |
| `adaptive` | 1 µs | Enabled | Dynamic adjustment |

## Information Density

Ternary provides +58.5% information density over binary:

```typescript
import { calculateInformationDensity } from 'libternary';

const density = calculateInformationDensity(100);
// { trits: 100, bitsEquivalent: 158.5, efficiencyGain: '+58.50%' }
```

## API Reference

### Ternary Types

- `convertTrit(value, from, to)` - Convert between representations
- `convertVector(values, from, to)` - Convert arrays
- `getTritMeaning(value, representation)` - Get semantic meaning
- `isValidTrit(value, representation)` - Validate trit value

### Ternary Operations

- `ternaryAdd(a, b)` - GF(3) addition
- `ternaryMultiply(a, b)` - GF(3) multiplication
- `ternaryRotate(value, steps)` - Bijective rotation
- `ternaryXor(a, b)` - Ternary XOR
- `ternaryNot(value)` - Ternary negation
- `calculateInformationDensity(tritCount)` - Density calculator

### Femtosecond Timing

- `getFemtosecondTimestamp()` - Get current timestamp
- `calculateDuration(start, end)` - Calculate time difference
- `getTimingMetrics()` - Get timing metrics
- `validateRecombinationWindow(primary, secondary, tolerance)` - Validate timing

### Phase Encryption

- `getPhaseConfig(mode)` - Get configuration
- `phaseSplit(data, mode)` - Split and encrypt
- `phaseRecombine(encrypted)` - Recombine and decrypt
- `getRecommendedMode(dataLength, isSensitive)` - Get recommended mode

## License

All Rights Reserved and Preserved | © Capomastro Holdings Ltd 2026

For licensing inquiries, please contact Capomastro Holdings Ltd.
