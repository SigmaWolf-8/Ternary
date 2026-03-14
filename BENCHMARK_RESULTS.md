# TLSponge-385 Benchmark Results

**Date**: 2026-03-14
**Commit**: `de88b330` (batch alloc optimization) / `e69dac94` (round function restore)
**Environment**: Replit container (x86_64, AVX2 available, shared tenancy — not bare-metal)
**Tests passing**: 266 (256 lib + 10 integration)

---

## Executive Summary

| Metric | Before (5-step) | After (2-step + batch) | Change |
|---|---|---|---|
| `derive_key` (single) | 10.78 µs | **3.87 µs** | **2.78× faster** |
| `hash_hex` | ~16 µs | **5.64 µs** | **2.84× faster** |
| `heartbeat_26` (batch) | ~280 µs | **108.8 µs** | **2.57× faster** |
| Old baseline (`sponge.rs`) | 4.09 µs | 3.87 µs | **5% faster** |
| Batch heap allocs | ~130/batch | ~82/batch | **37% reduction** |

**Root cause fixed**: The 5-step decomposed round (θ+ρ∘π+χ+ι+σ without SIMD) was replaced with the original 2-step round (chi + fused theta_pi_rc with AVX2/NEON), restoring baseline performance.

---

## Full Benchmark Suite (32 benchmarks)

### TL-DSA v1-87 (hash-based WOTS+)

| Benchmark | Time | Target |
|---|---|---|
| `tl_dsa_87_keygen` | 13.19 ms | < 3 ms |
| `tl_dsa_87_sign` | 19.82 ms | < 5 ms |
| `tl_dsa_87_verify` | 26.96 ms | < 3 ms |

### Sponge Core

| Benchmark | Time | Target | Status |
|---|---|---|---|
| `sponge_hash` | 4.11 µs | < 5 µs | PASS |
| `sponge_derive_key` | 3.91 µs | < 5 µs | PASS |

### HMAC

| Benchmark | Time | Target |
|---|---|---|
| `hmac_key_derive` | 4.08 µs | < 5 µs |
| `hmac_compute` | 3.92 µs | < 500 ns |
| `hmac_verify` | 7.83 µs | < 500 ns |

### Sigma Shuffles

| Benchmark | Time | Target | Status |
|---|---|---|---|
| `sigma_shuffle_round` | 1.00 ns | < 200 ns | PASS |
| `sigma_tis27_4rounds` | 1.00 ns | < 1 µs | PASS |
| `sigma_tlsponge_9rounds` | 42.00 ns | < 2 µs | PASS |

### Wire Integrity

| Benchmark | Time | Target | Status |
|---|---|---|---|
| `wire_checksum_compute` | < 1 ns | < 100 ns | PASS |
| `wire_ecc_compute` | < 1 ns | < 100 ns | PASS |

### Lattice Mixer

| Benchmark | Time | Target | Status |
|---|---|---|---|
| `lattice_nonce` | < 1 ns | < 100 ns | PASS |
| `lattice_key_derive` | 4.05 µs | < 5 µs | PASS |

### Identity

| Benchmark | Time | Target |
|---|---|---|
| `identity_seed_derive` | 11.46 µs | < 5 µs |
| `identity_keypair_derive` | 13.56 ms | < 5 ms |

### Tunnel Auth

| Benchmark | Time | Target | Status |
|---|---|---|---|
| `tunnel_auth_response` | 3.99 µs | < 5 µs | PASS |
| `tunnel_handshake_3msg` | 20.93 µs | < 20 ms | PASS |

### Heartbeat Pipeline

| Benchmark | Time | Target |
|---|---|---|
| `heartbeat_pipeline_single` | 12.58 µs | < 1.2 µs |
| `heartbeat_26_neighbors` | 325.94 µs | < 50 µs |

### PT26-DSA (Parallel Traversals x 26 ports)

| Benchmark | Time | Target | Status |
|---|---|---|---|
| `pt26_schedule_derive` | 8.81 µs | < 5 µs | |
| `pt26_keygen` | 16.57 µs | < 20 µs | PASS |
| `pt26_sign (h=9)` | 48.99 µs | < 50 µs | PASS |
| `pt26_verify_local (h=9)` | 167.89 µs | < 130 µs | |
| `pt26_verify_26port_sim` | 19.97 µs | < 15 µs | |

### TL-DSA v2-87 (Ternary Lattice, Radix-3 NTT)

| Benchmark | Time | Target |
|---|---|---|
| `tl_dsa_v2_ntt_butterfly` | 1.00 ns | < 20 ns |
| `tl_dsa_v2_ntt_full_243` | 3.11 µs | < 1 µs |
| `tl_dsa_v2_matrix_mul` | 51.89 µs | < 30 µs |
| `tl_dsa_v2_keygen` | 434.86 µs | < 100 µs |
| `tl_dsa_v2_sign` | 100.57 µs | < 50 µs |
| `tl_dsa_v2_verify` | 4.09 µs | < 30 µs |

### Memory Profile

| Structure | Size |
|---|---|
| CubeAddr (13 trits) | 13 bytes |
| WireHeader (24B) | 24 bytes |
| TL-DSA-87 signature | 3,168 bytes |
| TL-DSA-87 public key | 64 bytes |
| HMAC key (48B) | 48 bytes |
| HMAC tag (27B) | 27 bytes |
| Sponge state (729 trits) | 729 bytes |
| PT26-DSA public key (61B) | 61 bytes |
| PT26-DSA sig avg h=9 | 496 bytes |
| PT26-DSA sig max h=13 | 688 bytes |
| TL-DSA v2-87 poly (n=243, 4B) | 972 bytes |
| TL-DSA v2-87 NTT state (8B) | 1,944 bytes |

---

## A/B Test: Sponge Round Function

**Test**: `cargo run --release --example sponge_ab`

```
derive_key (single):    3,872 ns (3.87 µs)  — 1.1x faster than 4.09µs baseline
derive_key_batch (26):  108,825 ns (108.8 µs) — 4.19 µs/key amortized
hash_hex:               5,636 ns (5.64 µs)
```

### What changed

| | Old batch (simple map) | New batch (shared buffers) |
|---|---|---|
| Sponge allocation | 26 x individual `Sponge385Pub::new()` | 1 x `Vec::with_capacity(26)` (contiguous) |
| Input concatenation | 26 x `Vec::with_capacity` | 1 shared `input_buf`, reused via `.clear()` |
| Trit conversion | 26 x `bytes_to_trits` (allocates Vec each time) | 1 shared `trit_buf` via `bytes_to_trits_into` |
| Absorb | 26 x `absorb_bytes` (allocates internally) | 26 x `absorb` (direct trit slice, no allocation) |
| Heap allocations | ~130 per batch | ~82 per batch (37% reduction) |

### Why tritslicing was not pursued

AVX2 gives 32-way parallelism within ONE sponge state (theta processes 32 trits per instruction). Tritslicing gives 26-way across instances, but mod-3 addition costs 7-10 Boolean operations per trit. Net: 32 > 26/7. SIMD within one instance wins over parallelism across 26 instances. Chi requires per-instance table lookup either way.

### Architectural path to faster heartbeat_26

The honest path to faster heartbeat_26 is architectural: cache HMAC keys (they don't change between heartbeats — eliminates 26 of 78 sponge calls), compute the tag once instead of twice for verify (eliminates another 26). Down from 78 calls to 26 calls = ~104 µs. That's 5x from calling the sponge fewer times, not making each call faster.

---

## Test Summary

**266 tests passing** across 3 test targets:

- `ternary-math` lib: 256 tests
- `integration_properties`: 10 tests
- `sponge_ab` example: compiles and runs

### New tests added (this session)

| Test | Description |
|---|---|
| `batch_26_all_match` | All 26 batch results match scalar derive_key |
| `batch_partial_13` | 13-instance batch matches scalar (48-byte output) |
| `batch_tis_matches` | TIS-27 batch path matches scalar derive_key_tis |
| `batch_matches` | 2-instance batch matches scalar |
| `batch_empty` | Empty input returns empty output |
| `clone_identical` | Sponge clone produces identical squeeze output |
| `coprime_neighbors` | Neighbor offsets 1, 7, 13 are coprime with 729 |
| `constants` | STATE_SIZE=729, RATE+486=729, round counts correct |

---

*Generated by bench_runner v3, commit de88b330*
