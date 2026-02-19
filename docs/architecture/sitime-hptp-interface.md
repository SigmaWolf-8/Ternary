# SiTime Hardware Oscillator Interface Specification

**Document**: HPTP-SiTime Integration Specification v1.0  
**Date**: February 2026  
**Status**: Draft — Positions HPTP as protocol layer for SiTime precision MEMS oscillators

## Purpose

This specification defines the interface between PlenumNET's High-Precision Timing Protocol (HPTP) and SiTime MEMS precision oscillators. The FM timing engine (`libternary/fm_timing`) provides the protocol layer; SiTime hardware provides the physical timing reference. Together they enable sub-microsecond network-wide time synchronization through tonal diffusion.

## Target Hardware

### Compatible SiTime Product Families

| Product | Part Number | Stability | Frequency | Use Case |
|---------|------------|-----------|-----------|----------|
| SiT5711 Elite Super-TCXO | SiT5711AI | ±50 ppb | 1-60 MHz | Core timing authority (η=0) |
| SiT5356 Precision OCXO | SiT5356AI | ±5 ppb | 1-220 MHz | Regional timing nodes |
| SiT9501 Stratum 3E | SiT9501AI | ±30 ppb | 1-220 MHz | Edge timing nodes |
| SiT5008 TCXO | SiT5008AC | ±0.5 ppm | 1-60 MHz | Standard network nodes |

### Selection Criteria

Nodes at toroidal coordinate η=0 (core timing authority) require the highest stability oscillators (SiT5711 or SiT5356). Edge nodes (η > 2) can use standard TCXOs. The tonal diffusion model's attenuation factor `e^{-α·η}` naturally reduces precision requirements as η increases.

## Electrical Interface

### Clock Output

```
Signal:      LVCMOS / LVDS differential pair
Voltage:     1.8V or 3.3V (configurable via VDDO)
Frequency:   Nominal f₀ (configurable, typically 10 MHz or 25 MHz)
Jitter:      < 0.5 ps RMS (12 kHz - 20 MHz integration band)
Rise/Fall:   < 1 ns (10%-90%)
```

### FM Control Input

The SiTime oscillator's voltage-controlled frequency pulling input enables real-time FM modulation by the HPTP engine.

```
Pin:         AFC/VC (Analog Frequency Control)
Range:       ±25 ppm pull range at ±0.9V (SiT5711)
Sensitivity: ~28 ppm/V (linear region)
Bandwidth:   DC to 10 kHz modulation bandwidth
Input Z:     > 100 kΩ
```

### DAC Requirements

The FM modulation from the van der Pol oscillator engine must be converted to an analog voltage for the AFC input:

```
Resolution:  16-bit minimum (for sub-ppb frequency steps)
Update Rate: ≥ 1 kHz (must exceed modulation bandwidth)
Settling:    < 100 μs to 1 LSB
Voltage:     0 to 3.3V (mapped to AFC ±0.9V via resistive divider)
Interface:   SPI or I2C to host processor
```

### Timing Capture

For phase measurement and tonal field computation:

```
Capture:     TDC (Time-to-Digital Converter) or FPGA timestamp
Resolution:  < 100 ps (for femtosecond-scale error budget contribution)
Input:       Clock edge from SiTime output
Reference:   PPS from GNSS or network timing reference
```

## Protocol Binding

### Oscillator-to-HPTP Mapping

| Van der Pol Parameter | SiTime Mapping | Units |
|----------------------|----------------|-------|
| `ω₀` (intrinsic frequency) | Nominal oscillator frequency f₀ | rad/s |
| `μ` (damping coefficient) | AFC loop bandwidth setting | dimensionless |
| `F_ext` (external forcing) | AFC voltage from DAC | V → ppm |
| `ξ(t)` (noise) | Allan deviation floor (ADEV) | ppb |
| `V(t)` (oscillator state) | Instantaneous phase from TDC | radians |
| `dV/dt` (velocity) | Instantaneous frequency deviation from TDC | ppm |

### FM Modulation Parameters

The HPTP FM timing engine operates with these parameters mapped to SiTime hardware:

```
f₀         = SiTime nominal frequency (Hz)
β          = modulation_index (0-255, maps to ±25 ppm pull range)
f_m        = modulation frequency (adaptive, from resonance tuner)
Δf_peak    = β × f_m (maximum frequency deviation)
```

Modulation index β mapping:
```
β = 0     →  no modulation (CW reference)
β = 128   →  ±12.5 ppm deviation (standard operation)
β = 255   →  ±25 ppm deviation (maximum sweep)
```

### Phase Quantization Thresholds

The ternary phase quantization maps SiTime phase error to TernaryTrit:

```
Phase Error           TernaryTrit    Action
──────────────────    ───────────    ──────────────────
< -π/6 rad           Neg (Behind)   Increase AFC voltage
[-π/6, +π/6] rad     Zero (Sync)    Hold AFC voltage
> +π/6 rad           Pos (Ahead)    Decrease AFC voltage
```

At f₀ = 10 MHz, π/6 radians ≈ 16.7 ns timing uncertainty window.

## FM Timing Packet Framing

### Packet Generation from Hardware

```
1. TDC captures phase at each sync interval (1/f_sync seconds)
2. Host computes phase error against network reference
3. Van der Pol engine integrates one RK4 step
4. Phase quantized to TernaryTrit
5. Packet assembled:
   - timestamp_trits[27]: from system clock, balanced ternary encoded
   - frequency_state.f_inst: from TDC frequency measurement
   - frequency_state.sidebands[4]: from FFT of recent phase samples
   - frequency_state.coherence: from Allan deviation over last N samples
   - modulation_index: current AFC β setting
   - network_health: aggregate from neighbor packets
   - entropy_nonce[8]: from HRV entropy pool (ADEV fluctuations)
6. Packet transmitted to neighbors per toroidal topology
```

### AFC Correction from Diffusion

```
1. Receive neighbor FM timing packets
2. TonalField.updateFromPacket() computes local potential
3. DiffusionSolver.step() produces ClockCorrection
4. correction.offsetAdjust → DAC voltage update for AFC
5. correction.frequencyAdjust → long-term AFC bias trim
6. correction.newConfidence → coherence field for next packet
```

### AFC Voltage Calculation

```
V_AFC = V_center + (correction.offsetAdjust / AFC_SENSITIVITY)
      = 1.65V + (offset_ppm / 28 ppm/V) × 1.0V

Clamped to [0.75V, 2.55V] (AFC linear range)
```

## Synchronization State Machine

```
    ┌──────────┐    phase error    ┌───────────┐
    │          │    within π/6     │           │
    │ AQUIRING ├──────────────────►│ TRACKING  │
    │          │                   │           │
    │ AFC sweep│                   │ Fine AFC  │
    │ ±25 ppm  │    phase error    │ ±2 ppm    │
    │          │◄──────────────────┤           │
    └──────────┘    exceeds π/3    └─────┬─────┘
         │                               │
         │         Pi2 ≈ 1.0             │
         │                               ▼
         │                        ┌───────────┐
         │                        │           │
         └────────────────────────│  LOCKED   │
              holdover timeout    │           │
                                  │ Diffusion │
                                  │ consensus │
                                  └───────────┘
```

### States

| State | Entry Condition | AFC Range | Duration |
|-------|----------------|-----------|----------|
| ACQUIRING | Power-on or loss of lock | ±25 ppm sweep | 1-30 seconds |
| TRACKING | Phase error < π/6 | ±2 ppm fine | Until Pi2 stabilizes |
| LOCKED | Pi2 within [0.8, 1.2] for 10+ cycles | ±0.5 ppm trim | Indefinite |

### Holdover

If all neighbor packets are lost (network partition):

```
Holdover Duration = f(oscillator_stability, temperature_range)

SiT5711 (±50 ppb):  > 24 hours to ±1 μs drift
SiT5356 (±5 ppb):   > 72 hours to ±1 μs drift
SiT5008 (±0.5 ppm): > 2 hours to ±1 μs drift
```

During holdover, the node broadcasts packets with `coherence = 0.0` and `network_health = Neg`, signaling degraded timing confidence to the tonal field.

## Environmental Specifications

### Operating Conditions

```
Temperature:     -40°C to +85°C (industrial, all SiTime families)
Vibration:       Per SiTime datasheet (MEMS inherently vibration-resistant)
Altitude:        No derating required (solid-state, no pressure sensitivity)
EMI:             SiTime MEMS oscillators rated per IEC 61000-4-3
```

### Power

```
SiT5711:  3.3V, 12 mA typical (core timing)
SiT5356:  3.3V, 65 mA typical (regional timing, OCXO)
SiT5008:  1.8V, 3 mA typical (edge nodes, low power)
```

## Acceptance Criteria

### Phase 4 Benchmark Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Phase noise | < -110 dBc/Hz @ 1 kHz offset | Spectrum analyzer |
| Allan deviation (1s) | < 1e-11 (SiT5711) | Counter/TDC |
| AFC linearity | < 1% deviation over ±10 ppm | DAC sweep |
| Lock acquisition | < 10 seconds (standard) | Timer |
| Tonal field convergence | Pi2 within [0.8, 1.2] in < 60s | Software metric |
| Holdover drift | < 1 μs/24h (SiT5711) | TDC long-term |

### Compatibility Verification

1. Verify AFC input does not exceed SiTime absolute maximum ratings
2. Confirm DAC resolution provides sub-ppb frequency steps
3. Validate TDC resolution captures SiTime jitter floor
4. Test FM modulation bandwidth does not exceed oscillator AFC bandwidth
5. Verify holdover behavior matches predicted drift from ADEV specifications

## Forward Compatibility

This specification is designed to accommodate:

- **Future PQ signature integration**: The `entropy_nonce` field in FM packets will carry hardware-derived entropy from SiTime ADEV fluctuations, XOR-mixed with the supplementary HRV chaotic-map entropy, feeding the post-quantum signature chain. The SiTime ADEV source serves as the primary entropy; the HRV source is supplementary
- **Multi-oscillator configurations**: Core nodes may run multiple SiTime oscillators in voting configurations for fault tolerance
- **FPGA integration**: The XPlenum RISC-V extension can directly interface with SiTime AFC via memory-mapped I/O, eliminating host processor latency
