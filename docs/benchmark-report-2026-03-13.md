# PlenumNET / Salvi Framework — Criterion Benchmark Report

**Date:** 2026-03-13
**Suite:** 46 benchmarks (36 production + 10 A/B comparison)
**Tool:** `cargo bench --bench inter_cube` (Criterion 0.5, release profile)
**Platform:** Replit container, x86_64, Rust 1.77.2

---

## 1. TL-DSA v1-87 (Hash-Based WOTS+)

| Benchmark | Measured | Target |
|-----------|----------|--------|
| `tl_dsa_87_keygen` | **13.96 ms** | < 3 ms |
| `tl_dsa_87_sign` | **21.10 ms** | < 5 ms |
| `tl_dsa_87_verify` | **29.46 ms** | < 3 ms |

---

## 2. PT26-DSA (Unified Geometric Signature)

| Benchmark | Measured | Target | Status |
|-----------|----------|--------|--------|
| `pt26_keygen` | **9.03 µs** | < 8 µs | ≈ target |
| `pt26_sign` | **29.45 µs** | < 18 µs | sponge-bound |
| `pt26_verify` | **50.45 µs** | < 18 µs | sponge-bound |
| `pt26_verify_parallel` | **49.91 µs** | < 18 µs | sponge-bound |
| `pt26_trit_diff` | **634 ps** | < 5 ns | ✓ 7.9× under |
| `pt26_step_token` | **320 ps** | < 5 ns | ✓ 15.6× under |
| `pt26_walk_token` | **320 ps** | < 5 ns | ✓ 15.6× under |

---

## 3. TL-DSA v2-87 (Ternary Lattice, Radix-3 NTT)

| Benchmark | Measured | Target | Status |
|-----------|----------|--------|--------|
| `tl_dsa_v2_ntt_butterfly` | **496 ps** | < 20 ns | ✓ 40× under |
| `tl_dsa_v2_ntt_full_243` | **2.97 µs** | < 1 µs | |
| `tl_dsa_v2_matrix_mul` | **40.41 µs** | < 30 µs | |
| `tl_dsa_v2_keygen` | **397.3 µs** | < 100 µs | |
| `tl_dsa_v2_sign` | **104.2 µs** | < 50 µs | |
| `tl_dsa_v2_verify` | **5.89 µs** | < 30 µs | ✓ 5.1× under |

---

## 4. HMAC

| Benchmark | Measured | Target |
|-----------|----------|--------|
| `hmac_key_derive` | **8.09 µs** | < 5 µs |
| `hmac_compute` | **16.00 µs** | < 500 ns |
| `hmac_verify` | **20.56 µs** | < 500 ns |

---

## 5. Sponge Core

| Benchmark | Measured | Target |
|-----------|----------|--------|
| `sponge_hash` | **9.90 µs** | < 5 µs |
| `sponge_derive_key` | **7.96 µs** | < 5 µs |

---

## 6. σ Shuffles

| Benchmark | Measured | Target | Status |
|-----------|----------|--------|--------|
| `sigma_shuffle_round` | **136 ns** | < 200 ns | ✓ |
| `sigma_tis27_4rounds` | **257 ns** | < 1 µs | ✓ 3.9× under |
| `sigma_tlsponge_9rounds` | **489 ns** | < 2 µs | ✓ 4.1× under |

---

## 7. Wire Integrity

| Benchmark | Measured | Target | Status |
|-----------|----------|--------|--------|
| `wire_checksum_compute` | **312 ps** | < 100 ns | ✓ 320× under |
| `wire_ecc_compute` | **602 ps** | < 100 ns | ✓ 166× under |

---

## 8. Lattice Mixer

| Benchmark | Measured | Target | Status |
|-----------|----------|--------|--------|
| `lattice_nonce` | **347 ps** | < 100 ns | ✓ 288× under |
| `lattice_key_derive` | **12.36 µs** | < 5 µs | |

---

## 9. Identity

| Benchmark | Measured | Target |
|-----------|----------|--------|
| `identity_seed_derive` | **15.65 µs** | < 5 µs |
| `identity_keypair_derive` | **15.04 ms** | < 5 ms |

---

## 10. Tunnel Auth

| Benchmark | Measured | Target | Status |
|-----------|----------|--------|--------|
| `tunnel_auth_response` | **12.29 µs** | < 5 µs | |
| `tunnel_handshake_3msg` | **45.00 µs** | < 20 ms | ✓ 444× under |

---

## 11. Heartbeat Pipeline

| Benchmark | Measured | Target |
|-----------|----------|--------|
| `heartbeat_pipeline_single` | **12.88 µs** | < 1.2 µs |
| `heartbeat_26_neighbors` | **539.3 µs** | < 50 µs |

---

## 12. A/B Comparison: Scalar vs Packed GF(27) Sponge

*TM-2026-013 Phase A validation. Tests whether the GF(27)-native packed
representation (1 byte per GF(27) element) outperforms the scalar path
(1 byte per trit, mod-3 arithmetic) without SIMD.*

| Pair | Scalar (A) | Packed (B) | Ratio (A/B) | Verdict |
|------|-----------|-----------|-------------|---------|
| `derive_key` | **4.33 µs** | 67.33 µs | 0.064× | Packed 15.5× slower |
| `hash` | **10.07 µs** | 69.86 µs | 0.144× | Packed 6.9× slower |
| `hmac` | **12.42 µs** | 133.55 µs | 0.093× | Packed 10.8× slower |
| `heartbeat_26` | **538 µs** | 5,205 µs | 0.103× | Packed 9.7× slower |
| `pt26_sign` | **12.48 µs** | 138.09 µs | 0.090× | Packed 11.1× slower |

**Root cause:** Each χ(x) = x¹⁷ over GF(27) requires 4 squarings × 9
GF(3) coefficient multiplications = 36 mod-3 ops per element × 243
elements × 9 rounds = 78,732 mod-3 operations per permutation.
The scalar path does far fewer total operations despite each
individual mod-3 being more expensive.

**Path forward:** Phase C (AVX2) processes 32 GF(27) elements per SIMD
instruction, collapsing 78,732 scalar ops to ~2,460 SIMD instructions.
Target: ~530 ns per derive_key.

---

## Summary: Sub-nanosecond Operations (World-Class)

These operations are pure arithmetic — no sponge dependency:

| Operation | Time | Notes |
|-----------|------|-------|
| `wire_checksum_compute` | 312 ps | Single-cycle |
| `pt26_step_token` | 320 ps | GF(3) walk step |
| `pt26_walk_token` | 320 ps | Token extraction |
| `lattice_nonce` | 347 ps | Nonce generation |
| `tl_dsa_v2_ntt_butterfly` | 496 ps | Radix-3 butterfly |
| `wire_ecc_compute` | 602 ps | 8-trit ECC syndrome |
| `pt26_trit_diff` | 634 ps | Rep C subtraction |

## Summary: Sponge-Bound Operations (Optimization Target)

Every operation above ~5 µs is gated by `sponge::derive_key` at ~4–8 µs
per call. The sponge is the single bottleneck for the entire platform.

| Operation | Time | Sponge calls |
|-----------|------|-------------|
| `sponge_derive_key` | 7.96 µs | 1 |
| `hmac_compute` | 16.00 µs | 2 |
| `pt26_sign` | 29.45 µs | 2 |
| `heartbeat_26` | 539 µs | 78 (3 × 26) |
| `identity_keypair_derive` | 15.04 ms | ~1,800 |

---

*46 benchmarks. 59 module tests. Zero lookup tables in computation.*
*Report generated from cargo bench on Replit container, 2026-03-13.*
