# TM-2026-028 — HModal Signaling Architecture

Coprime walk scheduling via axiom-derived clock signal. Every parameter — amplitude levels, duty cycle, frequency content, null channels, phase structure — derives from the circle quadratic x² − 40x + 364 = 0.

## When to Use

Activate this skill when working on:
- Inter-Cube communication scheduling or the HModal waveform
- Coprime walk scheduling (dispatch/idle cycle)
- Channel architecture: data channels (non-null harmonics) vs control channels (n ≡ 0 mod 4)
- Spread spectrum modulation for Inter-Cube nodes
- Signal amplitude levels α = 91/36 (idle), β = 91/3 (dispatch)
- Duty cycle d = 1/4 and phase stepping π/4
- DC component 455/48 (455 = 5 × 7 × 13)
- Null channel detection (sync beacons, collision detection)
- Walk position → sideband routing
- Array3 Service Cube HModal integration
- WebSocket relay HModal frame encoding
- Fourier series of the scheduling signal

## Key Results

### Signal Parameters (all from x² − 40x + 364 = 0)
| Parameter | Value | Origin |
|-----------|-------|--------|
| Amplitude ratio | 12 = √Δ | Discriminant |
| High state (dispatch) | β = 91/3 | R₆/√Δ |
| Low state (idle) | α = 91/36 | R₆/Δ |
| Transition magnitude | γ = 1001/36 | 7×11×13 in numerator |
| Duty cycle | 1/4 | Harmonic position inverse |
| DC level | 455/48 | 5×7×13 emerges naturally |
| Null channels | n ≡ 0 mod 4 | sin(πn/4) = 0 |
| Phase step | π/4 | From d = 1/4 |

### Fourier Series
H(t) = 455/48 + (1001/18π) Σ (1/n) sin(πn/4) cos(nωt − πn/4)

### Channel Architecture
- Data: n = 1,2,3,5,6,7,9,10,11,... (87% power in first 3)
- Control: n = 4,8,12,16,... (exactly zero HModal energy)
- Separation is algebraic — no filtering required

### Walk Integration
| Walk | LCM (2D) | × 729 (3D+) |
|------|----------|-------------|
| (7,11,13) | 1,001 | 729,729 |
| (7,11,13,15) | 15,015 | 10,945,935 |
| Maximum sextuple (5,7,8,9,11,13) | 360,360 | 262,822,440 |

### Scheduling Cycle
0 to T/4: dispatch (β). T/4 to T: idle (α). After lcm periods, every node transmitted once. Zero collisions (CRT).

## Cross-References
- **TM-2026-017 v7.0 §19**: HModal derivation (summary)
- **TM-2026-028a**: Perfect Hash Spatial Indexing (companion)
- **TM-2026-026**: UV Spectral Protocol (spectral architecture)

## Full Document
Read the complete technical memo at: `docs/technical-memos/TM-2026-028-HModal-Signaling-Architecture.md`
