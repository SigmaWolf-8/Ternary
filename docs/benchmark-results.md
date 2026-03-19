# PlenumNET Crypto Benchmark Results

**Spec:** TM-2026-020.1-PREREQ §7  
**Suite:** `ternary-math/benches/crypto_benchmarks.rs` (Criterion)  
**Date:** _(fill on run)_  
**Platform:** _(fill on run — e.g. Replit Linux x86_64, Rust 1.x release)_  
**Criterion version:** 0.5

---

## TIS-27 (4-round TLSponge — fast path)

| Benchmark | Target | Actual | Unit | Notes |
|-----------|--------|--------|------|-------|
| hash_hex_tis / 48B | ≤ 5 µs | — | µs/op | Identity / scan hash |
| derive_key_tis / 1KB | ≤ 15 µs | — | µs/op | Wire HMAC derivation |
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
| derive_key_bulk / 1KB | ≤ 30 µs | — | µs/op | RATE_BULK (486 trits) |
| derive_key_bulk / 64KB | ≤ 1.5 ms | — | ms/op | |
| derive_key_bulk / 1MB | ≤ 20 ms | — | ms/op | |

## Sponge Permutation (raw)

| Benchmark | Target | Actual | Unit | Notes |
|-----------|--------|--------|------|-------|
| v2_full_9rounds | ≤ 5 µs | — | µs/op | Chi + Theta-Pi-RC, AVX2 |
| v1_no_chi_9rounds | ≤ 3 µs | — | µs/op | Theta-Pi-RC only |

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
| encapsulate / 512 | ≤ 3 ms | — | ms/op | |
| encapsulate / 768 | ≤ 5 ms | — | ms/op | |
| encapsulate / 1024 | ≤ 8 ms | — | ms/op | |
| decapsulate / 512 | ≤ 3 ms | — | ms/op | IND-CCA2, FO transform |
| decapsulate / 768 | ≤ 5 ms | — | ms/op | |
| decapsulate / 1024 | ≤ 8 ms | — | ms/op | |

## Phase Encryption v3 (duplex sponge stream cipher)

| Benchmark | Target | Actual | Unit | Notes |
|-----------|--------|--------|------|-------|
| encrypt / 1KB | ≤ 500 µs | — | µs/op | Balanced mode |
| encrypt / 64KB | ≤ 20 ms | — | ms/op | |
| encrypt / 1MB | ≤ 300 ms | — | ms/op | |
| decrypt / 1KB | ≤ 500 µs | — | µs/op | |
| decrypt / 64KB | ≤ 20 ms | — | ms/op | |
| decrypt / 1MB | ≤ 300 ms | — | ms/op | |

---

## How to Run

```bash
cd ternary-math
cargo bench --bench crypto_benchmarks
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
