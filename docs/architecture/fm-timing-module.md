# FM Timing Engine — Rust Module Documentation

**Crate**: `libternary`  
**Module**: `fm_timing`  
**Location**: `libternary/src/fm_timing/`

## Module Structure

```
libternary/src/fm_timing/
  mod.rs           — Module root, public re-exports
  oscillator.rs    — Van der Pol oscillator with FM modulation
  packet.rs        — FM timing packet codec (27-trit timestamps)
  hrv.rs           — HRV entropy source for PQ key material
  gf3_gradient.rs  — GF(3) ternary gradient operator
```

## Core Type: TernaryTrit

All modules use the unified `TernaryTrit` enum defined in `libternary/src/lib.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TernaryTrit {
    Neg,   // -1: Behind / Falling
    Zero,  //  0: Synchronized / Flat
    Pos,   // +1: Ahead / Rising
}
```

Conversion methods:
- `TernaryTrit::from_i8(val: i8) -> Self` — Maps -1/0/+1 to enum variants
- `TernaryTrit::to_i8(&self) -> i8` — Maps enum variants to numeric values

## oscillator.rs — Van der Pol Oscillator

Implements the van der Pol oscillator from Section 2.1 of the unified model, generating FM-modulated timing signals.

### Equations

The oscillator integrates the nonlinear ODE:

```
d²V/dt² - μ(1 - V²)·dV/dt + ω₀²·V = F_ext + ξ(t)
```

Where:
- `ω₀` = intrinsic angular frequency
- `μ` = nonlinear damping coefficient (controls limit cycle amplitude)
- `F_ext` = external forcing from modulation function `m(t)`
- `ξ(t)` = stochastic noise from HRV entropy source

### Key Types

```rust
pub struct TonalOscillator {
    omega_0: f64,      // intrinsic angular frequency
    mu: f64,           // nonlinear damping
    state: [f64; 2],   // [V, dV/dt] phase space
    t: f64,            // elapsed time
    noise: HrvEntropy, // entropy source
}
```

### Methods

- `new(omega_0, mu) -> Self` — Construct with given frequency and damping
- `step(&mut self, dt: f64) -> TernaryTrit` — Advance one RK4 time step, return quantized phase
- `frequency(&self) -> f64` — Current instantaneous frequency
- `phase(&self) -> f64` — Current phase angle
- `modulation_index(&self) -> u8` — Quantized FM beta parameter (0-255)

### Phase Quantization

The continuous phase is quantized to a balanced ternary trit:

```
phase_error < -THRESHOLD  →  TernaryTrit::Neg   (clock behind)
|phase_error| < THRESHOLD →  TernaryTrit::Zero   (synchronized)
phase_error > +THRESHOLD  →  TernaryTrit::Pos    (clock ahead)
```

The threshold is `π/6` (30 degrees), providing a dead zone for noise rejection.

## packet.rs — FM Timing Packet Codec

Encodes timing state into a structured packet where the instantaneous frequency carries network state metadata.

### Packet Structure

```rust
pub struct FmTimingPacket {
    pub timestamp_trits: [TernaryTrit; 27],  // 3³ balanced ternary timestamp
    pub frequency_state: FrequencyState,
    pub modulation_index: u8,                // FM beta (quantized)
    pub network_health: TernaryTrit,         // aggregate tonal state
    pub entropy_nonce: [u8; 8],              // from HRV source
}

pub struct FrequencyState {
    pub f_inst: f64,          // instantaneous frequency (Hz)
    pub sidebands: [f64; 4],  // power at f ± nf_m harmonics
    pub coherence: f64,       // sync quality (0.0 - 1.0)
}
```

### Encoding

- 27-trit timestamp: Each trit is one of {Neg, Zero, Pos}, giving 3²⁷ = 7,625,597,484,987 distinct timestamps
- Frequency state carries the oscillator's instantaneous frequency and sideband power spectrum
- Entropy nonce provides 64 bits of randomness from the HRV pool

### Methods

- `from_oscillator(osc: &TonalOscillator, health: TernaryTrit) -> Self` — Build packet from oscillator state
- `encode(&self) -> Vec<u8>` — Serialize to byte stream
- `decode(bytes: &[u8]) -> Result<Self, PacketError>` — Deserialize from byte stream

## hrv.rs — HRV Entropy Source

Implements the stochastic noise term ξ(t) as a deterministic chaotic map, providing bounded entropy for post-quantum key material generation.

### Implementation

Uses a logistic map (`x_{n+1} = r·x_n·(1-x_n)` with r=3.99) as the chaos generator, with output clamped to `[-MAX_JITTER, +MAX_JITTER]`.

```rust
pub struct HrvEntropy {
    buffer: VecDeque<f64>,  // ring buffer of jitter samples
    pool: [u8; 256],        // accumulated entropy pool
    state: f64,             // chaotic map state
    health: EntropyHealth,  // health monitoring
}

pub struct EntropyHealth {
    pub min_entropy: f64,   // estimated min-entropy (bits/sample)
    pub samples_collected: u64,
    pub health_status: bool,
}
```

### Methods

- `new() -> Self` — Initialize with default seed
- `new_deterministic(seed: f64) -> Self` — Initialize with fixed seed for reproducible testing
- `sample(&mut self) -> f64` — Draw one bounded noise sample
- `extract_bytes(&mut self, n: usize) -> Vec<u8>` — Extract whitened entropy bytes
- `health(&self) -> &EntropyHealth` — Current health metrics

### Health Monitoring

Inspired by NIST SP 800-90B:
- Tracks min-entropy estimate
- Monitors for stuck states (repeated outputs)
- Reports health status for upstream consumers

## gf3_gradient.rs — GF(3) Ternary Gradient Operator

Computes field gradients natively in balanced ternary arithmetic. The three-valued output (rising/falling/flat) runs directly on TVM opcodes with no floating-point overhead.

### GF(3) Arithmetic

Balanced ternary field GF(3) = {-1, 0, +1} with modular arithmetic:

```rust
pub fn gf3_add(a: TernaryTrit, b: TernaryTrit) -> TernaryTrit
pub fn gf3_sub(a: TernaryTrit, b: TernaryTrit) -> TernaryTrit
pub fn gf3_neg(a: TernaryTrit) -> TernaryTrit
```

Addition/subtraction tables:

```
⊕₃   -1   0  +1       ⊖₃   -1   0  +1
-1   +1  -1   0       -1    0  -1  +1
 0   -1   0  +1        0   +1   0  -1
+1    0  +1  -1       +1   -1  +1   0
```

### Gradient Computation

```rust
pub fn ternary_gradient(
    local_value: TernaryTrit,
    neighbors: &[GradientNeighbor],
) -> TernaryGradient
```

For each toroidal axis (η, θ, ψ):
1. Collect GF(3) differences with neighbors along that axis
2. Apply majority vote to determine gradient direction
3. Negate (gradient points downhill)

Formula: `F_{T,η}^{(3)} = -(Φ_T(η + Δη) ⊖₃ Φ_T(η))`

### Majority Vote

```rust
pub fn majority_vote(trits: &[TernaryTrit]) -> TernaryTrit
```

Returns the sign of the sum: positive sum → Pos, negative → Neg, zero → Zero. Empty input returns Zero (no information = flat field).

## Integration Points

- **HRV → PQ Crypto**: `HrvEntropy::extract_bytes()` feeds into existing post-quantum signature generation in `libternary`. No changes to PQ modules needed.
- **Oscillator → Packet**: `FmTimingPacket::from_oscillator()` captures the oscillator's instantaneous state.
- **GF(3) Gradient → Diffusion**: The `TernaryGradient` struct maps directly to the TypeScript `{ eta: Trit; theta: Trit; psi: Trit }` type consumed by `DiffusionSolver.step()`.
- **TernaryTrit → TypeScript Trit**: `TernaryTrit::Neg` = `-1`, `TernaryTrit::Zero` = `0`, `TernaryTrit::Pos` = `+1`.
