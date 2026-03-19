# PlenumNET Crypto Benchmark Results

**Spec:** TM-2026-020.1-PREREQ §7  
**Suite:** `ternary-math/benches/crypto_benchmarks.rs` (Criterion 0.5)  
**Date:** _(fill on run)_  
**Platform:** _(fill — e.g. Linux x86_64 AVX2, Rust 1.x release)_  
**SIMD dispatch:** _(AVX2 / NEON / scalar — auto-detected at runtime)_

---

## TIS-27 (4-round TLSponge — fast path)

| Benchmark | Target | Actual | Unit | Notes |
|-----------|--------|--------|------|-------|
| hash_hex_tis / 48B | ≤ 5 µs | — | µs/op | Identity / scan hash |
| derive_key_tis / 1KB | ≤ 15 µs | — | µs/op | Wire HMAC derivation |
| derive_key_tis / 64KB | ≤ 500 µs | — | µs/op | Bulk TIS throughput |
| derive_key_tis / 1MB | ≤ 8 ms | — | ms/op | Large input TIS throughput |
| batch_x26 | ≤ 100 µs | — | µs/op | 26-neighbor heartbeat batch |

## TLSponge-385 (9-round — full security)

| Benchmark | Target | Actual | Unit | Notes |
|-----------|--------|--------|------|-------|
| hash / 48B | ≤ 10 µs | — | µs/op | Short-message hash |
| hash_hex / 48B | ≤ 12 µs | — | µs/op | Hex-encoded output |
| derive_key / 48B | ≤ 12 µs | — | µs/op | KDF short input |
| hash / 1KB | ≤ 50 µs | — | µs/op | |
| hash / 64KB | ≤ 2 ms | — | ms/op | |
| hash / 1MB | ≤ 30 ms | — | ms/op | |
| derive_key / 1KB | ≤ 50 µs | — | µs/op | Full-security KDF |
| derive_key / 64KB | ≤ 2 ms | — | ms/op | |
| derive_key / 1MB | ≤ 30 ms | — | ms/op | |
| derive_key_bulk / 1KB | ≤ 30 µs | — | µs/op | RATE_BULK (486 trits, ~10 MB/s) |
| derive_key_bulk / 64KB | ≤ 1.5 ms | — | ms/op | |
| derive_key_bulk / 1MB | ≤ 20 ms | — | ms/op | |

## Sponge: Full (9-round) vs TIS (4-round) Comparison

| Benchmark | Target (full) | Actual (full) | Target (TIS) | Actual (TIS) | Speedup | Unit |
|-----------|---------------|---------------|--------------|--------------|---------|------|
| hash_hex / 48B | ≤ 12 µs | — | ≤ 5 µs | — | ~2.4× | µs/op |
| derive_key / 1KB | ≤ 50 µs | — | ≤ 15 µs | — | ~3.3× | µs/op |
| derive_key / 64KB | ≤ 2 ms | — | ≤ 500 µs | — | ~4× | µs/op |

## Sponge Permutation (raw — SIMD-dispatched)

| Benchmark | Target | Actual | Unit | Notes |
|-----------|--------|--------|------|-------|
| v2_chi_9rounds | ≤ 5 µs | — | µs/op | Chi + Theta-Pi-RC (AVX2/NEON auto) |
| v1_no_chi_9rounds | ≤ 3 µs | — | µs/op | Theta-Pi-RC only (legacy v1) |

> **SIMD note:** On x86_64 with AVX2, chi uses `vpshufb`+`blendv` split-table
> and theta uses contiguous `_mm256_loadu_si256` loads. On aarch64, NEON
> `vtbl1q` is used. Scalar fallback is automatic when neither is detected.
> To force scalar: set `SIMDENABLE=0` env var (if supported) or benchmark on
> a platform without AVX2/NEON. The v2 vs v1 delta isolates chi-layer cost.

## TL-DSA (WOTS+ hash-based signatures)

| Benchmark | Target | Actual | Unit | Notes |
|-----------|--------|--------|------|-------|
| keygen / 44 | ≤ 20 ms | — | ms/op | 51 chains × 15-deep |
| keygen / 65 | ≤ 30 ms | — | ms/op | 67 chains × 15-deep |
| keygen / 87 | ≤ 50 ms | — | ms/op | 99 chains × 15-deep |
| sign / 44 | ≤ 10 ms | — | ms/op | |
| sign / 65 | ≤ 15 ms | — | ms/op | |
| sign / 87 | ≤ 25 ms | — | ms/op | |
| verify / 44 | ≤ 10 ms | — | ms/op | PK-only verification |
| verify / 65 | ≤ 15 ms | — | ms/op | |
| verify / 87 | ≤ 25 ms | — | ms/op | |

## TL-KEM (Lattice Key Encapsulation)

| Benchmark | Target | Actual | Unit | Notes |
|-----------|--------|--------|------|-------|
| keygen / 512 | ≤ 5 ms | — | ms/op | NIST Level 1 (k=2) |
| keygen / 768 | ≤ 8 ms | — | ms/op | NIST Level 3 (k=3) |
| keygen / 1024 | ≤ 12 ms | — | ms/op | NIST Level 5 (k=4) |
| encapsulate / 512 | ≤ 3 ms | — | ms/op | Deterministic (seeded randomness) |
| encapsulate / 768 | ≤ 5 ms | — | ms/op | |
| encapsulate / 1024 | ≤ 8 ms | — | ms/op | |
| decapsulate / 512 | ≤ 3 ms | — | ms/op | IND-CCA2, FO transform |
| decapsulate / 768 | ≤ 5 ms | — | ms/op | |
| decapsulate / 1024 | ≤ 8 ms | — | ms/op | |

## Phase Encryption v3 (duplex sponge stream cipher)

### HighSecurity mode

| Benchmark | Target | Actual | Unit | Notes |
|-----------|--------|--------|------|-------|
| encrypt / HighSec / 1KB | ≤ 1 ms | — | µs/op | 4-phase, full security |
| encrypt / HighSec / 64KB | ≤ 40 ms | — | ms/op | |
| encrypt / HighSec / 1MB | ≤ 600 ms | — | ms/op | |
| decrypt / HighSec / 1KB | ≤ 1 ms | — | µs/op | |
| decrypt / HighSec / 64KB | ≤ 40 ms | — | ms/op | |
| decrypt / HighSec / 1MB | ≤ 600 ms | — | ms/op | |

### Balanced mode

| Benchmark | Target | Actual | Unit | Notes |
|-----------|--------|--------|------|-------|
| encrypt / Balanced / 1KB | ≤ 500 µs | — | µs/op | Default mode |
| encrypt / Balanced / 64KB | ≤ 20 ms | — | ms/op | |
| encrypt / Balanced / 1MB | ≤ 300 ms | — | ms/op | |
| decrypt / Balanced / 1KB | ≤ 500 µs | — | µs/op | |
| decrypt / Balanced / 64KB | ≤ 20 ms | — | ms/op | |
| decrypt / Balanced / 1MB | ≤ 300 ms | — | ms/op | |

### Performance mode

| Benchmark | Target | Actual | Unit | Notes |
|-----------|--------|--------|------|-------|
| encrypt / Perf / 1KB | ≤ 300 µs | — | µs/op | Reduced rounds |
| encrypt / Perf / 64KB | ≤ 12 ms | — | ms/op | |
| encrypt / Perf / 1MB | ≤ 180 ms | — | ms/op | |
| decrypt / Perf / 1KB | ≤ 300 µs | — | µs/op | |
| decrypt / Perf / 64KB | ≤ 12 ms | — | ms/op | |
| decrypt / Perf / 1MB | ≤ 180 ms | — | ms/op | |

### Adaptive mode

| Benchmark | Target | Actual | Unit | Notes |
|-----------|--------|--------|------|-------|
| encrypt / Adaptive / 1KB | ≤ 500 µs | — | µs/op | Auto-selects based on input |
| encrypt / Adaptive / 64KB | ≤ 20 ms | — | ms/op | |
| encrypt / Adaptive / 1MB | ≤ 300 ms | — | ms/op | |
| decrypt / Adaptive / 1KB | ≤ 500 µs | — | µs/op | |
| decrypt / Adaptive / 64KB | ≤ 20 ms | — | ms/op | |
| decrypt / Adaptive / 1MB | ≤ 300 ms | — | ms/op | |

## Inter-Cube Infrastructure Benchmarks

> These benchmarks are covered in the existing `inter_cube.rs` suite
> (109 benchmarks, manual `black_box` timing). The targets below are
> the §7 reference values. Actual values come from running
> `cargo bench --bench inter_cube`.

| Benchmark | Target | Actual | Unit | Notes |
|-----------|--------|--------|------|-------|
| CON tunnel_key_derive | ≤ 20 µs | — | µs/op | Per-tunnel TLSponge KDF |
| CON rekey_single | ≤ 25 µs | — | µs/op | Single tunnel rekey |
| CON rekey_all_26 | ≤ 200 µs | — | µs/op | Full neighbor rekey |
| GLB route_lookup | ≤ 5 µs | — | µs/op | 13D geometric routing |
| FTS fault_detection | ≤ 50 µs | — | µs/op | Heartbeat timeout check |
| CRS registration | ≤ 100 ms | — | ms/op | TL-DSA-signed register |
| CRS resolution | ≤ 10 µs | — | µs/op | Address lookup |
| Heartbeat single | ≤ 20 µs | — | µs/op | HMAC compute + verify |
| Heartbeat ×26 | ≤ 200 µs | — | µs/op | All neighbors |

---

## How to Run

```bash
cd ternary-math

cargo bench --bench crypto_benchmarks

cargo bench --bench inter_cube
```

HTML reports are generated in `ternary-math/target/criterion/`.

## How to Update This Document

After running the benchmark suite, fill the **Actual** column with the median
value from Criterion output. Flag any result exceeding the **Target** column
with ⚠️ and file a performance ticket.

## NIST / Industry Comparisons

| Primitive | PlenumNET | NIST Equivalent | Reference |
|-----------|-----------|-----------------|-----------|
| TLSponge-385 | 385-bit PQ | SHA3-256 (FIPS 202) | Keccak sponge |
| TIS-27 | 4-round fast | SHAKE-128 (FIPS 202) | Reduced-round XOF |
| TL-DSA | WOTS+ hash-based | SPHINCS+ (FIPS 205) | Hash-based OTS |
| TL-KEM | Module-LWE | ML-KEM (FIPS 203) | Kyber |
| Phase Enc v3 | Duplex sponge | AES-256-CTR | Stream cipher |

## Platform Notes

| Platform | SIMD Path | Expected Speedup |
|----------|-----------|------------------|
| x86_64 + AVX2 | `vpshufb` chi + contiguous theta | ~2× over scalar |
| aarch64 + NEON | `vtbl1q` chi + `vld1q` theta | ~1.5× over scalar |
| Scalar fallback | Loop-based chi + THETA_IDX table | Baseline |
