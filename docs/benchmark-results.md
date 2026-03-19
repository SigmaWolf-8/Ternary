# PlenumNET Crypto Benchmark Results

**Spec:** TM-2026-020.1-PREREQ §7  
**Suite:** `ternary-math/benches/crypto_benchmarks.rs` (Criterion 0.5)  
**Date:** _(fill on run)_  
**Platform:** _(fill — e.g. Linux x86_64 AVX2, Rust 1.x release)_  
**SIMD dispatch:** _(AVX2 / NEON / scalar — auto-detected at runtime)_

---

## TIS-27 (4-round TLSponge — fast path)

| Benchmark | Target | Actual (x86_64 AVX2) | Actual (ARM64 NEON) | Actual (scalar) | Unit |
|-----------|--------|----------------------|---------------------|-----------------|------|
| hash_hex_tis / 48B | ≤ 191 ns | — | — | — | ns/op |
| hash_hex_tis / 1KB | ≤ 5 µs | — | — | — | µs/op |
| hash_hex_tis / 64KB | ≤ 300 µs | — | — | — | µs/op |
| hash_hex_tis / 1MB | ≤ 5 ms | — | — | — | ms/op |
| derive_key_tis / 48B | ≤ 200 ns | — | — | — | ns/op |
| derive_key_tis / 1KB | ≤ 5 µs | — | — | — | µs/op |
| derive_key_tis / 64KB | ≤ 300 µs | — | — | — | µs/op |
| derive_key_tis / 1MB | ≤ 5 ms | — | — | — | ms/op |
| derive_key_bulk_tis / 48B | ≤ 150 ns | — | — | — | ns/op |
| derive_key_bulk_tis / 1KB | ≤ 4 µs | — | — | — | µs/op |
| derive_key_bulk_tis / 64KB | ≤ 200 µs | — | — | — | µs/op |
| derive_key_bulk_tis / 1MB | ≤ 3 ms | — | — | — | ms/op |
| batch_x26 | ≤ 50 µs | — | — | — | µs/op |

## TLSponge-385 (9-round — full security)

| Benchmark | Target | Actual (x86_64 AVX2) | Actual (ARM64 NEON) | Actual (scalar) | Unit |
|-----------|--------|----------------------|---------------------|-----------------|------|
| hash / 48B | ≤ 10 µs | — | — | — | µs/op |
| hash / 1KB | ≤ 50 µs | — | — | — | µs/op |
| hash / 64KB | ≤ 2 ms | — | — | — | ms/op |
| hash / 1MB | ≤ 30 ms | — | — | — | ms/op |
| hash_hex / 48B | ≤ 12 µs | — | — | — | µs/op |
| hash_hex / 1KB | ≤ 50 µs | — | — | — | µs/op |
| hash_hex / 64KB | ≤ 2 ms | — | — | — | ms/op |
| hash_hex / 1MB | ≤ 30 ms | — | — | — | ms/op |
| derive_key / 48B | ≤ 12 µs | — | — | — | µs/op |
| derive_key / 1KB | ≤ 50 µs | — | — | — | µs/op |
| derive_key / 64KB | ≤ 2 ms | — | — | — | ms/op |
| derive_key / 1MB | ≤ 30 ms | — | — | — | ms/op |
| derive_key_bulk / 48B | ≤ 10 µs | — | — | — | µs/op |
| derive_key_bulk / 1KB | ≤ 30 µs | — | — | — | µs/op |
| derive_key_bulk / 64KB | ≤ 1.5 ms | — | — | — | ms/op |
| derive_key_bulk / 1MB | ≤ 20 ms | — | — | — | ms/op |

> **Bulk throughput target:** ~10 MB/s (RATE_BULK = 486 trits = 97 bytes/permutation)

## Sponge: Full (9-round) vs TIS (4-round) Comparison

| Operation | Size | Full (9R) | TIS (4R) | Speedup | Unit |
|-----------|------|-----------|----------|---------|------|
| hash_hex | 48B | — | — | — | µs/op |
| hash_hex | 1KB | — | — | — | µs/op |
| hash_hex | 64KB | — | — | — | ms/op |
| hash_hex | 1MB | — | — | — | ms/op |
| derive_key | 48B | — | — | — | µs/op |
| derive_key | 1KB | — | — | — | µs/op |
| derive_key | 64KB | — | — | — | ms/op |
| derive_key | 1MB | — | — | — | ms/op |
| derive_key_bulk | 48B | — | — | — | µs/op |
| derive_key_bulk | 1KB | — | — | — | µs/op |
| derive_key_bulk | 64KB | — | — | — | ms/op |
| derive_key_bulk | 1MB | — | — | — | ms/op |

> Expected TIS speedup: ~2.25× (9/4 round ratio)

## Sponge Permutation (raw)

| Benchmark | Target | Actual (x86_64 AVX2) | Actual (ARM64 NEON) | Actual (scalar) | Unit |
|-----------|--------|----------------------|---------------------|-----------------|------|
| v2_chi_9rounds | ≤ 4.3 µs | — | — | — | µs/op |
| v1_no_chi_9rounds | ≤ 2.5 µs | — | — | — | µs/op |

> **SIMD dispatch:** Chi layer uses AVX2 `vpshufb`+`blendv` split-table (x86_64)
> or NEON `vtbl1q` lo/hi (aarch64). Theta-Pi-RC uses contiguous SIMD loads on
> both architectures. The v2-v1 delta isolates chi-layer cost. Both share the
> same SIMD-dispatched theta path. To measure scalar-only performance, run on
> a platform without AVX2/NEON (e.g. `QEMU_CPU=core2duo` user-mode emulation)
> or patch `permute_n()` to skip the `is_x86_feature_detected!` branch.

## TL-DSA (WOTS+ hash-based signatures)

| Benchmark | Target | Actual (x86_64 AVX2) | Actual (ARM64 NEON) | Actual (scalar) | Unit |
|-----------|--------|----------------------|---------------------|-----------------|------|
| keygen / 44 | ≤ 20 ms | — | — | — | ms/op |
| keygen / 65 | ≤ 30 ms | — | — | — | ms/op |
| keygen / 87 | ≤ 50 ms | — | — | — | ms/op |
| sign / 44 | ≤ 500 µs | — | — | — | µs/op |
| sign / 65 | ≤ 1,441 µs | — | — | — | µs/op |
| sign / 87 | ≤ 2,500 µs | — | — | — | µs/op |
| verify / 44 | ≤ 10 ms | — | — | — | ms/op |
| verify / 65 | ≤ 15 ms | — | — | — | ms/op |
| verify / 87 | ≤ 25 ms | — | — | — | ms/op |

## TL-KEM (Lattice Key Encapsulation)

| Benchmark | Target | Actual (x86_64 AVX2) | Actual (ARM64 NEON) | Actual (scalar) | Unit |
|-----------|--------|----------------------|---------------------|-----------------|------|
| keygen / 512 | ≤ 5 ms | — | — | — | ms/op |
| keygen / 768 | ≤ 8 ms | — | — | — | ms/op |
| keygen / 1024 | ≤ 12 ms | — | — | — | ms/op |
| encapsulate / 512 | ≤ 3 ms | — | — | — | ms/op |
| encapsulate / 768 | ≤ 5 ms | — | — | — | ms/op |
| encapsulate / 1024 | ≤ 8 ms | — | — | — | ms/op |
| decapsulate / 512 | ≤ 3 ms | — | — | — | ms/op |
| decapsulate / 768 | ≤ 5 ms | — | — | — | ms/op |
| decapsulate / 1024 | ≤ 8 ms | — | — | — | ms/op |

## Phase Encryption v3 (duplex sponge stream cipher)

### HighSecurity mode

| Benchmark | Target | Actual | Unit |
|-----------|--------|--------|------|
| encrypt / HighSec / 1KB | ≤ 1 ms | — | µs/op |
| encrypt / HighSec / 64KB | ≤ 40 ms | — | ms/op |
| encrypt / HighSec / 1MB | ≤ 600 ms | — | ms/op |
| decrypt / HighSec / 1KB | ≤ 1 ms | — | µs/op |
| decrypt / HighSec / 64KB | ≤ 40 ms | — | ms/op |
| decrypt / HighSec / 1MB | ≤ 600 ms | — | ms/op |

### Balanced mode

| Benchmark | Target | Actual | Unit |
|-----------|--------|--------|------|
| encrypt / Balanced / 1KB | ≤ 500 µs | — | µs/op |
| encrypt / Balanced / 64KB | ≤ 20 ms | — | ms/op |
| encrypt / Balanced / 1MB | ≤ 300 ms | — | ms/op |
| decrypt / Balanced / 1KB | ≤ 500 µs | — | µs/op |
| decrypt / Balanced / 64KB | ≤ 20 ms | — | ms/op |
| decrypt / Balanced / 1MB | ≤ 300 ms | — | ms/op |

### Performance mode

| Benchmark | Target | Actual | Unit |
|-----------|--------|--------|------|
| encrypt / Perf / 1KB | ≤ 300 µs | — | µs/op |
| encrypt / Perf / 64KB | ≤ 12 ms | — | ms/op |
| encrypt / Perf / 1MB | ≤ 180 ms | — | ms/op |
| decrypt / Perf / 1KB | ≤ 300 µs | — | µs/op |
| decrypt / Perf / 64KB | ≤ 12 ms | — | ms/op |
| decrypt / Perf / 1MB | ≤ 180 ms | — | ms/op |

### Adaptive mode

| Benchmark | Target | Actual | Unit |
|-----------|--------|--------|------|
| encrypt / Adaptive / 1KB | ≤ 500 µs | — | µs/op |
| encrypt / Adaptive / 64KB | ≤ 20 ms | — | ms/op |
| encrypt / Adaptive / 1MB | ≤ 300 ms | — | ms/op |
| decrypt / Adaptive / 1KB | ≤ 500 µs | — | µs/op |
| decrypt / Adaptive / 64KB | ≤ 20 ms | — | ms/op |
| decrypt / Adaptive / 1MB | ≤ 300 ms | — | ms/op |

## Inter-Cube Infrastructure Benchmarks

> These benchmarks are covered in the existing `inter_cube.rs` suite
> (109 benchmarks, manual `black_box` timing). Actual values from
> `cargo bench --bench inter_cube`.

| Benchmark | Target | Actual | Unit |
|-----------|--------|--------|------|
| CON tunnel_key_derive | ≤ 20 µs | — | µs/op |
| CON rekey_single | ≤ 25 µs | — | µs/op |
| CON rekey_all_26 | ≤ 200 µs | — | µs/op |
| GLB route_lookup | ≤ 5 µs | — | µs/op |
| FTS fault_detection | ≤ 50 µs | — | µs/op |
| CRS registration | ≤ 100 ms | — | ms/op |
| CRS resolution | ≤ 10 µs | — | µs/op |
| Heartbeat single | ≤ 20 µs | — | µs/op |
| Heartbeat ×26 | ≤ 200 µs | — | µs/op |

---

## How to Run

```bash
cd ternary-math

cargo bench --bench crypto_benchmarks

cargo bench --bench inter_cube
```

HTML reports are generated in `ternary-math/target/criterion/`.

## How to Update This Document

1. Run `cargo bench --bench crypto_benchmarks` on each target platform.
2. Fill the **Actual** column for the corresponding architecture.
3. Compute the Full-vs-TIS speedup ratio and fill the comparison table.
4. Flag any result exceeding the **Target** column with ⚠️.
5. File a performance ticket for any regression.

## NIST / Industry Comparisons

| Primitive | PlenumNET | NIST Equivalent | Reference |
|-----------|-----------|-----------------|-----------|
| TLSponge-385 | 385-bit PQ | SHA3-256 (FIPS 202) | Keccak sponge |
| TIS-27 | 4-round fast | SHAKE-128 (FIPS 202) | Reduced-round XOF |
| TL-DSA | WOTS+ hash-based | SPHINCS+ (FIPS 205) | Hash-based OTS |
| TL-KEM | Module-LWE | ML-KEM (FIPS 203) | Kyber |
| Phase Enc v3 | Duplex sponge | AES-256-CTR | Stream cipher |

## Platform Architecture Notes

| Platform | SIMD Path | Expected Speedup | Hash Target |
|----------|-----------|------------------|-------------|
| x86_64 + AVX2 | `vpshufb` chi + contiguous theta | ~2× over scalar | ≤ 4.3 µs/perm |
| aarch64 + NEON | `vtbl1q` chi + `vld1q` theta | ~1.5× over scalar | ≤ 3.2 µs/perm |
| Scalar fallback | Loop chi + THETA_IDX table | Baseline | ≤ 8 µs/perm |

> **SIMD vs scalar methodology:** The sponge permutation auto-dispatches at
> runtime via `is_x86_feature_detected!("avx2")` / `is_aarch64_feature_detected!("neon")`.
> The Criterion suite benchmarks the dispatched path. To isolate scalar performance:
> (a) run on a platform without AVX2/NEON, or (b) use QEMU user-mode emulation
> (`qemu-x86_64 -cpu core2duo`), or (c) temporarily patch `permute_n()` to skip
> the SIMD branch. The v2 vs v1 permutation benchmark isolates chi-layer cost
> (both share the same SIMD-dispatched theta-pi-rc path).
