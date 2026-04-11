# TM-2026-030 v3: TTC v5.0 Protocol Specification

## Tribonacci Ternary Compression — Coprime Periodic Transform Stage

Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
Patent(s) Pending — All Rights Reserved — Applied Physics Division

All multi-byte integers in wire format are big-endian.

---

## §1  The Transform

### §1.1 Definitions

Given input `d[0..N]` and k pairwise-coprime moduli `(m₁, ..., mₖ)`:

```
μ       = round(mean(d[i]))  clamped to [0, 255]            grand mean (u8)
μⱼ[a]   = round(mean{d[i] : i ≡ a mod mⱼ})  clamped to [0, 255]   marginal mean (u8)
```

Rounding: `round(x) = floor(x + 0.5)` (round half away from zero = Rust `f64::round()`).

### §1.2 Prediction

All μ and μⱼ are u8. Prediction is integer arithmetic:

```
d̂[i] = Σⱼ μⱼ[i mod mⱼ] − (k − 1) · μ        (i32, can exceed [0, 255])
pred[i] = clamp(d̂[i], 0, 255)                  (i16 — chosen for residual arithmetic, R2-A5-3)
```

### §1.3 Residual

```
r[i] = d[i] − pred[i]        (i16, range [-255, 255])
```

### §1.4 Reconstruction

```
pred[i] = clamp(Σⱼ μⱼ[i mod mⱼ] − (k−1)·μ, 0, 255)
d[i]    = clamp(pred[i] + r[i], 0, 255)
```

Lossless: encoder and decoder compute pred[i] from the same u8 values. QED.

---

## §2  Why Additive ANOVA

CRT: if m₁,...,mₖ are pairwise coprime, `i ↦ (i mod m₁, ..., i mod mₖ)` is a bijection on ℤ/(Πmⱼ)ℤ. The additive main-effects model is the unique model that is linear, lossless (subtraction inverse), and O(Σmⱼ) parameters. Full-interaction requires Πmⱼ parameters (header ≥ data). Multiplicative requires division (rounding breaks lossless). Additive is forced.

---

## §3  Modulus Source

### §3.1 Primary triple

ARC_ROOT_SEMI = 182 = 2 × 7 × 13. GREEN_ARC_EFF = 286 = 2 × 11 × 13. Distinct odd primes: {7, 11, 13} = COPRIME_TRIPLES[0].

Geometric tie (constants.rs §4): UNIT_CIRCLE_AREA = 14 = 2 × COPRIME_TRIPLES[0][0]. RADIAN_CIRCLE_AREA = 182 = COPRIME_PAIR_LCMS[6] = SQUARED_SIDE_SQ_RADIAN. The moduli are polygon central-angle periods; their composites are geometric-area strides.

### §3.2 Expansion

4-factor: COPRIME_QUADRUPLES[0] = [7,11,13,15]. 5-factor: COPRIME_QUINTUPLES[4] = [5,7,9,11,13]. 6-factor: COPRIME_SEXTUPLES[3] = [5,7,8,9,11,13]. No group of 7 exists (proved: any 7-subset contains excluded pair (3,9) or (4,8)).

### §3.3 Non-canonical periods

Trial factorization into two coprime factors (only two-factor splits attempted). Non-canonical moduli are not derived from the constant set. The deficit rate bound is not proved for them. Competitive selection discards them if unhelpful.

---

## §4  Overhead Bound

DEFICIT_RATE = NULL_HARMONIC_DEFICIT / GEOMETRIC_SPECTRAL_PRODUCT = 4,004 / 364,364 = 1/91.

**Rule**: header_bytes ≤ chunk.len() / 91 (integer floor division).

This is an algebraically motivated structural heuristic, not an information-theoretic proof. The competitive gate (§6.3) is the actual guarantee that CPT never produces worse output than LZ77.

| k | Header | Min chunk (91 × H) |
|---|---|---|
| 2 | 20 | 1,820 B |
| 3 | 33 | 3,003 B |
| 4 | 48 | 4,368 B |
| 5 | 47 | 4,277 B |
| 6 | 55 | 5,005 B |

---

## §5  AC Thresholds

Break-even: `ac > √(8H / (H₀ × N))`. For k=3, H₀=6, N=3,003: ac > 0.121.

AC_FLOOR = 0.15 (24% margin). AC_STRONG = 0.40 for non-canonical (406σ above noise floor on 4 KB sample). High-entropy chunks naturally fail the AC gate (E[ac] = 1/256 for random data).

---

## §6  Pipeline

### §6.1 Chunk modes

| Mode | Name | Pipeline |
|---|---|---|
| 0 | Stored | Unchanged |
| 1 | Compressed | LZ77 + prefix codes (unchanged) |
| 2 | TernaryEnhanced | LZ77 + trit encoding (unchanged) |
| 3 | TernaryAns | LZ77 + ternary rANS (unchanged) |
| 4 | CptAns | CPT + varint residual |
| 5 | CptLz77Ans | CPT + LZ77 on residual + rANS |

### §6.2 Mode 5 overflow guard

Mode 5 maps residuals to u8 via (r + 128). This is lossless only when all r[i] ∈ [-128, 127]. Mode 5 is produced as a candidate ONLY if `residual.iter().all(|&r| r >= -128 && r <= 127)`. If any residual exceeds this range, Mode 5 is not generated. Mode 4 (varint, unbounded) remains available.

### §6.3 Three-gate activation

1. **Overhead**: chunk.len() / 91 ≥ header_bytes
2. **Autocorrelation**: at least one canonical group scores > 0.15
3. **Competitive**: CPT payload < best LZ77 payload

All three must pass. CPT never makes compression worse.

---

## §7  Wire Format

### §7.1 Mode 4 (CptAns)

```
[4 BE]   original size N
[1]      mode = 0x04
[1]      factor count k (2–6)
[k]      moduli (each u8, valid range [2, 128])
[1]      grand mean μ
[Σmⱼ]    marginal means
[var]    residual: zigzag-varint i16, base-128 LSB-first
```

Zigzag: `zz = (r << 1) ^ (r >> 15)`. Examples: r=0→[0x00], r=255→[0xFE,0x03], r=-255→[0xFD,0x03].

### §7.2 Mode 5 (CptLz77Ans)

```
[4 BE]   original size N
[1]      mode = 0x05
[2 BE]   CPT header length L
[L]      CPT header
[var]    LZ77 + rANS of residual bytes (same format as Mode 3)
```

### §7.3 Input validation

Decoders MUST validate: all modulus values ∈ [2, 128]. Modulus 0 or 1 causes division-by-zero or degenerate marginals. Values > 128 exceed the wire format's u8 representation.

---

## §8  Probe Algorithm

**This section describes the algorithm implemented in cpd.rs probe().**

The probe selects a decomposition group from the 18 predefined canonical groups (7 triples + 2 quadruples + 5 quintuples + 4 sextuples) by scoring each group's autocorrelation at its member strides. This predefined-group approach was chosen over greedy factorization because: (a) groups are compile-time verified pairwise-coprime — no runtime gcd checks; (b) the search space is bounded (18 groups, not combinatorial); (c) the groups are architecturally significant (from the coprime walk landscape, not arbitrary).

```
INPUT:  chunk bytes, length N
OUTPUT: CpdAnalysis { moduli, header_bytes, group_source, best_ac }

1. If N < 91: return empty.

2. sample = chunk[0..min(N, 4096)]
   budget = N / 91  (integer floor division)

3. MEMBER STRIDE SCAN
   Compute autocorrelation at each of the 10 member values:
     strides = {3, 4, 5, 7, 8, 9, 11, 13, 14, 15}
   Store as ac_table: Vec<(u32, f64)>.

4. CANONICAL GROUP SCORING
   For each group G in COPRIME_TRIPLES[0..7], COPRIME_QUADRUPLES[0..2],
     COPRIME_QUINTUPLES[0..5], COPRIME_SEXTUPLES[0..4]:
       h = 2 + Σ members(G)
       If h > budget: skip
       score = min{ ac_table[m] for m in members(G) }
       If score ≥ 0.15: add (G, h, score) to candidates

5. NON-CANONICAL SMALL-PERIOD SCAN
   For each stride s in 2..128, excluding the 10 member strides:
     ac = autocorrelation(sample, s)
     If ac > 0.40:
       factors = factorize_to_members(s)  [two-factor coprime split]
       If factors exist and hdr(factors) ≤ budget:
         Add (factors, hdr, ac) to candidates

6. SELECT BEST
   Among candidates, select the one with max(score / header_cost).
   Sort its moduli ascending. Return.

7. FALLBACK
   If no candidate passed step 4/5, and budget ≥ 33:
     Return [7, 11, 13] speculatively.
     (Competitive gate in phase2_compress discards if unhelpful.)
   Else: return empty.
```

---

## §9  Version Compatibility

Archives containing ANY Mode 4 or Mode 5 chunk use container version VERSION_CPT = 0x05.

| Version byte | Meaning |
|---|---|
| 0x02 | TTC v1.x (legacy) |
| 0x03 | TTC v2.0 (96-byte header) |
| 0x04 | TTC v3.0 (27-byte header) |
| **0x05** | **TTC v5.0 (CPT-capable, 28-byte header)** |

Pre-CPT decompressors encountering 0x05 return `TtcError::UnsupportedVersion(0x05)`. This is clean and actionable.

Pre-CPT code encountering an unknown chunk mode returns: "Unknown chunk mode N. Archive may require TTC v5.0+. Supported: 0–3." (The existing ChunkMode::from_u8 error is updated with this message.)

Archives containing only Mode 0–3 chunks continue to use version 0x04.

---

## §10  Operator Diagnostics

CompressionResult gains a `cpd_diagnostics: CpdDiagnostics` field:

```rust
pub struct CpdDiagnostics {
    pub chunks_attempted: u32,   // probe returned moduli
    pub chunks_selected: u32,    // CPT won competitive selection
    pub mode4_count: u32,        // CptAns chunks
    pub mode5_count: u32,        // CptLz77Ans chunks
    pub avg_ac: f64,             // average best AC across attempted chunks
}
```

This enables operators to diagnose: "Why did compression ratio change after upgrade to v5.0?" Answer: N chunks used CPT, average AC was X, M4/M5 split was Y/Z.

CPT can be disabled (if needed for debugging) by setting all probe results to empty in phase1_analyze. A `disable_cpt: bool` field can be added to CompressOptions if operator demand warrants it.

---

## §11  Worked Example

Input: `d = [10, 20, 30, 10, 20, 30, 10, 20]`, moduli (2, 3).

| Step | Computation | Result |
|---|---|---|
| Grand mean | round(150/8) | μ = 19 |
| μ₂ | [round(70/4), round(80/4)] | [18, 20] |
| μ₃ | [round(30/3), round(60/3), round(60/2)] | [10, 20, 30] |
| pred[0] | 18 + 10 − 19 | 9 |
| r[0] | 10 − 9 | 1 → zz=2 → [0x02] |
| pred[1] | 20 + 20 − 19 | 21 |
| r[1] | 20 − 21 | −1 → zz=1 → [0x01] |

All residuals are ±1. Entropy: 1.58 → 1.00 bits/byte (37% reduction).

Reconstruction: pred[0] + r[0] = 9 + 1 = 10 ✓. Decoder uses same u8 marginals.

---

## §12  Patent Claims

1. **Coprime Periodic Transform for lossless compression.** Additive ANOVA using pairwise-coprime moduli from the factorization of ARC_ROOT_SEMI (182 = 2×7×13) and GREEN_ARC_EFF (286 = 2×11×13), roots of arc²−832·arc+118,300=0, derived from x²−40x+364=0.

2. **Overhead bounding via null harmonic deficit.** Factor count scaled by DEFICIT_RATE = 1/91 from NULL_HARMONIC_DEFICIT (4,004) / GEOMETRIC_SPECTRAL_PRODUCT (364,364).

3. **Competitive mode integration.** Transform and dictionary codecs as parallel per-chunk candidates, smallest-output selection, zero user configuration.