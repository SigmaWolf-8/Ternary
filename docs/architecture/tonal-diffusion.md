# PlenumNET Tonal Diffusion Architecture

**Version**: 1.0  
**Date**: February 2026

## Overview

The Tonal Diffusion system replaces hierarchical stratum-based time distribution (NTP model) with gradient-driven diffusion consensus. Nodes follow the tonal potential gradient toward consensus, achieving network-wide time synchronization through physical diffusion rather than centralized dictation.

The architecture spans two languages (Rust and TypeScript) across six modules, with data flowing from the FM timing engine through the tonal field service to the resonance tuner and metrics dashboard.

## Module Architecture

```
                    ┌─────────────────────────────────────┐
                    │          Kong API Gateway            │
                    │   /api/tonal/*  /api/resonance/*     │
                    │   /api/metrics/plenum                │
                    └─────────────┬───────────────────────┘
                                  │
                    ┌─────────────▼───────────────────────┐
                    │        Express Routes                │
                    │   server/routes/tonal-field.ts       │
                    └──┬──────────┬───────────────┬───────┘
                       │          │               │
          ┌────────────▼──┐  ┌────▼────────┐  ┌──▼──────────────┐
          │  Tonal Field   │  │  Resonance  │  │ Plenum Metrics  │
          │   Service      │  │  Detector   │  │   (Pi1-Pi4)     │
          │ services/      │  │ server/     │  │ services/       │
          │ tonal-field/   │  │ resonance/  │  │ tonal-field/    │
          │                │  │             │  │ metrics.ts      │
          │  field.ts      │  │  index.ts   │  └─────────────────┘
          │  diffusion.ts  │  │             │
          │  index.ts      │  └─────────────┘
          └───────┬────────┘
                  │ imports
          ┌───────▼────────┐
          │ Shared Topology │
          │ shared/         │
          │ topology/       │
          │ index.ts        │
          └────────────────┘

    ═══════════════════════════════════════════════
    Rust Layer (libternary — compile-time, WASM-exportable)
    ═══════════════════════════════════════════════

          ┌────────────────────────────────────┐
          │       libternary/src/fm_timing/    │
          │                                    │
          │  oscillator.rs  — Van der Pol + FM │
          │  packet.rs      — FM packet codec  │
          │  hrv.rs         — Entropy source   │
          │  gf3_gradient.rs— GF(3) gradient   │
          │  mod.rs         — Module root      │
          └────────────────────────────────────┘
                         │
          ┌──────────────▼─────────────────────┐
          │  libternary/src/lib.rs              │
          │  TernaryTrit enum (Neg/Zero/Pos)    │
          │  — unified type across all modules  │
          └────────────────────────────────────┘
```

## Data Flow

The data flow follows the paper's cardiovascular analogy:

```
  Oscillator (Heart)
       │
       ▼ generates
  FM Timing Packet (Tone)
       │
       ▼ propagates through
  Toroidal Mesh (Plenum)
       │
       ▼ creates
  Tonal Potential Field (Pressure)
       │
       ▼ gradient drives
  Diffusion Correction (Flow)
       │
       ▼ monitored by
  Plenum Metrics Pi1-Pi4 (Vitals)
       │
       ▼ triggers
  Resonance Sweep (Heartbeat Tuning)
```

### Step-by-step

1. **FM Packet Generation** (`libternary/fm_timing/oscillator.rs`)
   - Van der Pol oscillator generates base timing signal
   - RK4 integration of: `d²V/dt² - μ(1-V²)dV/dt + ω²V = F_ext + ξ(t)`
   - Phase quantized to ternary: Behind (-1) / Synchronized (0) / Ahead (+1)
   - HRV entropy source provides stochastic noise term ξ(t)

2. **Packet Encoding** (`libternary/fm_timing/packet.rs`)
   - 27-trit balanced ternary timestamp (3³ encoding)
   - Frequency state carries instantaneous frequency, sidebands, coherence
   - 8-byte entropy nonce from HRV pool

3. **Topology Routing** (`shared/topology/index.ts`)
   - Toroidal address (η, θ, ψ) determines natural neighbors
   - Geodesic distance: `ds² = (R + η·cos(θ))²·dψ² + dη² + η²·dθ²`
   - k-nearest neighbor selection in toroidal metric
   - GF(3) gradient computation via balanced ternary subtraction

4. **Field Computation** (`services/tonal-field/field.ts`)
   - Packets attenuated by `e^{-α·η}` (distance decay)
   - Potential = coherence × attenuation × cos(f_inst·t + k·d)
   - Gradient computed by finite differences along each axis
   - Stale neighbors pruned after configurable timeout

5. **Diffusion Solving** (`services/tonal-field/diffusion.ts`)
   - Graph Laplacian: `L_ij = deg(i)·δ_ij - w_ij`
   - Edge weights: `w_ij = e^{-α·η_ij} · coherence_ij`
   - Diffusion step: `dc_i/dt = -D·Σ L_ij·offset_j - (D·offset_i/kT)·q·F_T^{(3)}`
   - Clock offset is the Laplacian state variable (not confidence)
   - GF(3) ternary gradient provides discrete drift term

6. **Resonance Detection** (`server/resonance/index.ts`)
   - Moens-Korteweg analog: `c₀ = pathLength / medianRTT`
   - Quarter-wave resonance: `f_r = c₀ / (4·L_longest)`
   - FM sweep: chirp sync rate through ±20% of f_r
   - Lorentzian quality function peaked at resonance

7. **Metrics Dashboard** (`services/tonal-field/metrics.ts`)
   - Pi1: Field/Pressure ratio (target ~1e-6)
   - Pi2: Sync/Resonance ratio (target ~1.0, headline KPI)
   - Pi3: Information density (target >>1)
   - Pi4: Tonal authority (context-dependent)
   - Health assessment: critical/warning/healthy

## Cross-Language Type Mapping

| Concept | Rust (`libternary`) | TypeScript (`shared/topology`) |
|---------|-------|------------|
| Balanced trit | `TernaryTrit::Neg/Zero/Pos` | `Trit = -1 \| 0 \| 1` |
| GF(3) subtraction | `gf3_sub(a, b) -> TernaryTrit` | `gf3Sub(a, b) -> Trit` |
| Gradient | `TernaryGradient { eta, theta, psi }` | `TernaryGradient { eta, theta, psi }` |
| Toroidal axis | `ToroidalAxis::Eta/Theta/Psi` | `'eta' \| 'theta' \| 'psi'` |

## Configuration Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `alpha` | 0.3 | Toroidal distance attenuation coefficient |
| `couplingStrength` | 0.1 | Gradient-to-correction coupling |
| `D` | 0.01 | Diffusion coefficient |
| `kT` | 0.1 | Thermal energy (noise tolerance) |
| `dt` | 0.05 | Diffusion time step |
| `freqCoupling` | 0.001 | Frequency adjustment coupling |
| `staleThresholdMs` | 30000 | Neighbor timeout (ms) |
| `historySize` | 256 | RTT ring buffer capacity |
| `pathLength` | 5.0 | Average network path length |
| `longestPath` | 10.0 | Longest path for resonance calculation |

## Security Considerations

- FM timing packets are currently unauthenticated at the API level
- In production, packets carry post-quantum signatures (PqSignature field in Rust codec)
- The HRV entropy source provides key material for PQ signature generation
- Adversarial resistance tested: honest nodes converge despite minority adversarial nodes injecting high-offset packets with low coherence
- The diffusion solver's coherence-weighted Laplacian naturally attenuates adversarial influence
