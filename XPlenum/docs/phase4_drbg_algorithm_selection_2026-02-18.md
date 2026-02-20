# XPlenum Phase 4.1 — DRBG Algorithm Selection

**Capomastro Holdings Ltd. — Applied Physics Division**
**Date**: 2026-02-18
**Classification**: CONFIDENTIAL
**Author**: PlenumNET Engineering
**Status**: APPROVED

---

## 1. Context

The XPlenum masking unit (`xplenum_mask_unit.v`) currently uses a 32-bit maximal-length Linear Feedback Shift Register (LFSR) with polynomial x³² + x²² + x² + x + 1 for random mask generation. While functionally adequate for development, the LFSR is **not FIPS 140-3 compliant**:

- LFSR output is deterministic and linear — an attacker observing 32 consecutive output bits can recover the entire internal state
- LFSR output fails NIST Statistical Test Suite randomness tests (linear complexity)
- No seed/reseed mechanism per SP 800-90A
- No health tests per SP 800-90B

FIPS 140-3 Level 3+ certification requires a NIST SP 800-90A approved Deterministic Random Bit Generator (DRBG) mechanism.

## 2. Candidate Algorithms

### 2.1 CTR_DRBG (AES-256)

**Specification**: NIST SP 800-90A Rev.1 Section 10.2.1
**Underlying Primitive**: AES-256 in counter mode
**Security Strength**: 256 bits

| Criterion | Assessment |
|-----------|-----------|
| **Area (gates)** | ~25,000–35,000 (AES-256 core: ~20K gates + CTR_DRBG FSM: ~5K–15K) |
| **Throughput** | 128 bits per AES round; at 100 MHz: ~1.28 Gbps (pipelined) or ~100 Mbps (iterative) |
| **Compliance** | Direct SP 800-90A Section 10.2.1 mapping; most common CMVP-validated mechanism |
| **Implementation Complexity** | Moderate — requires AES-256 core (SubBytes, ShiftRows, MixColumns, KeyExpansion) plus CTR_DRBG state machine (Instantiate, Update, Generate) |
| **Existing IP** | Multiple open-source AES cores available (secworks/aes, tiny-AES-c reference) |
| **CAVP KATs** | Published test vectors for CTR_DRBG with AES-256 (NIST CAVP DRBG vectors) |

### 2.2 Hash_DRBG (SHA-512)

**Specification**: NIST SP 800-90A Rev.1 Section 10.1.1
**Underlying Primitive**: SHA-512
**Security Strength**: 256 bits

| Criterion | Assessment |
|-----------|-----------|
| **Area (gates)** | ~45,000–60,000 (SHA-512 core: ~40K gates + Hash_DRBG FSM: ~5K–20K) |
| **Throughput** | 1024-bit blocks; at 100 MHz: ~200 Mbps (iterative, 80 rounds) |
| **Compliance** | SP 800-90A Section 10.1.1; less common than CTR_DRBG in hardware CMVP validations |
| **Implementation Complexity** | Higher — SHA-512 has 80 rounds with 64-bit arithmetic (requires 64-bit adders); Hash_DRBG has more complex state update (hashgen loop) |
| **Existing IP** | Some open-source SHA-512 cores; less hardware-optimised than AES |
| **CAVP KATs** | Published test vectors available but fewer implementations to cross-check |

### 2.3 HMAC_DRBG (SHA-384)

**Specification**: NIST SP 800-90A Rev.1 Section 10.1.2
**Underlying Primitive**: HMAC-SHA-384
**Security Strength**: 256 bits

| Criterion | Assessment |
|-----------|-----------|
| **Area (gates)** | ~50,000–70,000 (SHA-384 core + HMAC wrapper + DRBG FSM) |
| **Throughput** | Similar to Hash_DRBG (~200 Mbps at 100 MHz) |
| **Compliance** | SP 800-90A Section 10.1.2; common in software but rare in hardware |
| **Implementation Complexity** | Highest — HMAC requires two SHA passes; HMAC_DRBG Update function calls HMAC multiple times per generate |
| **Existing IP** | Minimal hardware-optimised implementations |
| **Note** | Already implemented in **software** in `src/kernel/src/crypto/drbg.rs` using SHA-384 |

## 3. Comparison Matrix

| Criterion | CTR_DRBG (AES-256) | Hash_DRBG (SHA-512) | HMAC_DRBG (SHA-384) |
|-----------|:---:|:---:|:---:|
| Gate Count | **~30K** | ~50K | ~60K |
| Throughput (100 MHz) | **~1.28 Gbps** | ~200 Mbps | ~200 Mbps |
| CMVP Validation Count | **Highest** | Moderate | Low (HW) |
| Implementation Complexity | **Moderate** | High | Highest |
| Open-Source HW IP | **Abundant** | Some | Minimal |
| CAVP Vector Availability | **Extensive** | Good | Good |
| Area Overhead on XPlenum | **+25–35K gates (~20%)** | +45–60K gates (~40%) | +50–70K gates (~50%) |

## 4. Decision

**Selected: CTR_DRBG with AES-256**

### Rationale

1. **Smallest area footprint**: At ~30K gates, CTR_DRBG adds approximately 20% to XPlenum's current ~12,700-gate integration overhead, keeping total extension area well under 50K gates. Hash_DRBG would nearly quadruple the extension's area.

2. **Highest throughput**: AES-256 in counter mode delivers ~10x the throughput of SHA-based alternatives. This matters because the masking unit generates random values on every TMASKR instruction — high throughput means no pipeline stalls waiting for random data.

3. **Most validated pathway**: CTR_DRBG with AES-256 is the most commonly validated DRBG mechanism in FIPS 140-3 hardware modules (CMVP database). This means extensive precedent for certification reviewers and fewer surprises during lab evaluation.

4. **Reuses existing AES knowledge**: The Salvi Framework already uses AES-256-GCM for token encryption in the backend security stack. Adding a hardware AES-256 core creates future reuse opportunities for hardware-accelerated symmetric encryption.

### Fallback Path

If AES-256 gate count proves prohibitive after synthesis on the target FPGA, Hash_DRBG with SHA-256 (not SHA-512) can be considered as a smaller alternative at ~35K gates, sacrificing throughput. This fallback maintains NIST compliance while reducing 64-bit arithmetic requirements.

## 5. Implementation Architecture

```
                    ┌─────────────────────────────────────┐
                    │        xplenum_ctr_drbg.v            │
                    │                                      │
  seed_i[255:0] ───►│  ┌──────────┐    ┌──────────────┐   │
  seed_valid_i ────►│  │ AES-256  │    │  CTR_DRBG    │   │
  reseed_i ────────►│  │  Core    │◄──►│  State       │   │──► drbg_data_o[31:0]
                    │  │          │    │  Machine     │   │──► drbg_valid_o
                    │  └──────────┘    │              │   │
                    │                  │  - V (key)   │   │──► health_error_o
                    │  ┌──────────┐    │  - Key       │   │
                    │  │ Health   │◄───│  - Counter   │   │
                    │  │ Tests    │    │  - Reseed    │   │
                    │  │ (SP90B)  │    │    Counter   │   │
                    │  └──────────┘    └──────────────┘   │
                    └─────────────────────────────────────┘
```

### Module Interface (Preliminary)

```verilog
module xplenum_ctr_drbg (
    input  wire         clk,
    input  wire         rst_n,

    // Seed / Reseed
    input  wire [255:0] seed_i,
    input  wire         seed_valid_i,
    input  wire         reseed_i,

    // Generate request
    input  wire         generate_i,

    // Output
    output wire [31:0]  drbg_data_o,
    output wire         drbg_valid_o,

    // Health
    output wire         health_error_o,
    output wire         ready_o
);
```

### Integration Points

1. **Replaces**: `lfsr` register and `lfsr_feedback` wire in `xplenum_mask_unit.v` (lines 43–55)
2. **Seed Source**: `CSR_XPMASK_SEED` (0x7C4) via `seed_wr` / `seed_data` signals already present
3. **Random Output**: `drbg_data_o` replaces `lfsr` in TMASKR result path
4. **Health Error**: New signal, routed to `xp_exception` / `xp_exc_code` as `XP_EXC_MASK_FAULT`

## 6. NIST CAVP Known Answer Test Vectors

The implementation will be validated against NIST CAVP CTR_DRBG test vectors:
- **Source**: NIST CAVP DRBG Test Vectors (drbgvectors.zip)
- **Configuration**: AES-256, no derivation function (for hardware), 256-bit security strength
- **Test Categories**: Instantiate, Generate, Reseed, Prediction Resistance

## 7. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| AES-256 core timing violations at 100 MHz | Low | Medium | Use iterative (non-pipelined) AES — meets timing with margin; throughput still exceeds masking unit demand |
| Health test false positives | Low | Low | Tune adaptive proportion test window size per SP 800-90B recommendations |
| Gate count exceeds budget | Low | Medium | Fallback to Hash_DRBG with SHA-256 if synthesis shows > 40K gate overhead |
| KAT vector mismatch | Medium | High | Cross-validate against multiple reference implementations before integration |

## 8. References

- NIST SP 800-90A Rev.1: "Recommendation for Random Number Generation Using Deterministic Random Bit Generators" (2015)
- NIST SP 800-90B: "Recommendation for the Entropy Sources Used for Random Bit Generation" (2018)
- FIPS 197: "Advanced Encryption Standard (AES)" (2001)
- NIST CAVP DRBG Test Vectors: https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program
- FIPS 140-3 (ISO/IEC 19790:2012): "Security Requirements for Cryptographic Modules"
