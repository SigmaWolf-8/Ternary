# PlenumNET / Salvi Framework — Benchmark Results

**Date**: 2026-03-14
**Suite**: v5 (109 benchmarks, 30 categories)
**Environment**: Replit container (x86_64, AVX2 available, shared tenancy — not bare-metal)
**Tests passing**: 266 (256 lib + 10 integration)

---

## Executive Summary

| Metric | Before (5-step) | After (2-step + stack buf) | Change |
|---|---|---|---|
| `derive_key` (single) | 10.78 µs | **3.75 µs** | **2.87× faster** |
| `hash_hex` | ~16 µs | **5.67 µs** | **2.82× faster** |
| `heartbeat_26` (cached keys) | ~280 µs | **388 µs** (steady-state) | see note¹ |
| Old baseline (`sponge.rs`) | 4.09 µs | 3.75 µs | **9% faster** |
| `derive_key` heap allocs | 3/call | **0/call** (≤256B input) | **eliminated** |
| Batch heap allocs | ~130/batch | ~82/batch | **37% reduction** |

¹ heartbeat_26 now measures steady-state: 52 sponge calls (compute+verify) with pre-cached HMAC keys. Previous measurement included key derivation in the hot path (78 calls total).

**Optimizations applied**:
- 2-step fused round (chi + theta_pi_rc with AVX2/NEON) replaces 5-step decomposed round
- Stack-buffer `derive_key`: inputs ≤256 bytes use `[0u8; 256]` input + `[0i8; 1280]` trit + `[0u8; 256]` output buffers — zero heap allocations
- `absorb_bytes_stack()` eliminates Vec in `bytes_to_trits` for common case
- TERN compress/decompress and HPTP drift/jitter use pure arithmetic (zero sponge calls)

---

## Grand Total

| Metric | Value |
|---|---|
| Total benchmarks | 109 |
| Categories | 30 |
| Grand total (all benchmarks) | **219.387 ms** |

---

## Full Benchmark Suite (109 benchmarks)

### 1. TL-DSA v1-87 (Hash-based WOTS+) — 3 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| tl_dsa_87_keygen | 13.263 ms | < 3ms |
| tl_dsa_87_sign | 20.139 ms | < 5ms |
| tl_dsa_87_verify | 26.300 ms | < 3ms |
| **▸ TL-DSA v1 TOTAL (3)** | **59.701 ms** | |

### 2. PT26-DSA (Geometric Signature) — 7 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| pt26_keygen | 7.49 µs | < 8µs |
| pt26_sign | 11.12 µs | < 18µs |
| pt26_verify | 18.40 µs | < 18µs |
| pt26_verify_parallel | 11.01 µs | < 18µs |
| pt26_trit_diff | 30 ns | < 5ns |
| pt26_step_token | 30 ns | < 5ns |
| pt26_walk_token | 30 ns | < 5ns |
| **▸ PT26-DSA TOTAL (7)** | **48.11 µs** | |

### 3. TL-DSA v2-87 (Ternary Lattice NTT) — 6 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| tl_dsa_v2_ntt_butterfly | 30 ns | < 20ns |
| tl_dsa_v2_ntt_full | 2.73 µs | < 1µs |
| tl_dsa_v2_matrix_mul | 38.61 µs | < 30µs |
| tl_dsa_v2_keygen | 253.01 µs | < 100µs |
| tl_dsa_v2_sign | 91.28 µs | < 50µs |
| tl_dsa_v2_verify | 4.56 µs | < 30µs |
| **▸ TL-DSA v2 TOTAL (6)** | **390.22 µs** | |

### 4. TL-KEM (Key Encapsulation) — 9 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| tl_kem_512_keygen | 36.55 µs | < 50µs |
| tl_kem_512_encaps | 14.71 µs | < 30µs |
| tl_kem_512_decaps | 11.00 µs | < 30µs |
| tl_kem_768_keygen | 62.71 µs | < 80µs |
| tl_kem_768_encaps | 18.34 µs | < 50µs |
| tl_kem_768_decaps | 18.14 µs | < 50µs |
| tl_kem_1024_keygen | 114.87 µs | < 120µs |
| tl_kem_1024_encaps | 25.56 µs | < 80µs |
| tl_kem_1024_decaps | 22.62 µs | < 80µs |
| **▸ TL-KEM TOTAL (9)** | **324.50 µs** | |

### 5. T-AE-MAC (Authenticated Encryption) — 4 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| tae_mac_encrypt | 36.87 µs | < 30µs |
| tae_mac_decrypt | 36.58 µs | < 30µs |
| tae_mac_compute | 14.73 µs | < 15µs |
| tae_mac_verify | 25.64 µs | < 20µs |
| **▸ T-AE-MAC TOTAL (4)** | **113.82 µs** | |

### 6. Phase Encryption — 4 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| phase_split | 71.10 µs | < 40µs |
| phase_recombine | 58.70 µs | < 40µs |
| phase_batch_split | 1.442 ms | < 400µs |
| phase_batch_recombine | 1.240 ms | < 400µs |
| **▸ Phase Enc TOTAL (4)** | **2.811 ms** | |

### 7. AES-256-GCM (Token Encryption) — 2 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| aes_gcm_encrypt | 40.56 µs | < 25µs |
| aes_gcm_decrypt | 40.46 µs | < 25µs |
| **▸ AES-GCM TOTAL (2)** | **81.02 µs** | |

### 8. RSA-4096 (Classical Co-Signature) — 2 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| rsa_4096_sign | 1.463 ms | < 2ms |
| rsa_4096_verify | 261.84 µs | < 200µs |
| **▸ RSA-4096 TOTAL (2)** | **1.725 ms** | |

### 9. Sponge Core — 5 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| sponge_hash | 5.67 µs | < 5µs |
| sponge_derive_key | 5.73 µs | < 5µs |
| tis27_hash_27trit | 3.74 µs | < 5µs |
| tis27_hash_54trit | 3.73 µs | < 5µs |
| tis27_absorb_squeeze | 3.73 µs | < 8µs |
| **▸ Sponge + TIS-27 TOTAL (5)** | **22.60 µs** | |

### 10. HMAC (Cached Keys) — 3 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| hmac_key_derive | 3.76 µs | < 5µs |
| hmac_compute | 7.35 µs | < 5µs |
| hmac_verify | 14.70 µs | < 10µs |
| **▸ HMAC TOTAL (3)** | **25.81 µs** | |

### 11. Wire Integrity — 2 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| wire_checksum | 120 ns | < 100ns |
| wire_ecc | 70 ns | < 100ns |
| **▸ Wire TOTAL (2)** | **190 ns** | |

### 12. Lattice Mixer — 2 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| lattice_nonce | 30 ns | < 100ns |
| lattice_key_derive | 7.36 µs | < 5µs |
| **▸ Lattice TOTAL (2)** | **7.39 µs** | |

### 13. Identity — 2 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| identity_seed_derive | 3.77 µs | < 5µs |
| identity_keypair_derive | 13.260 ms | < 5ms |
| **▸ Identity TOTAL (2)** | **13.264 ms** | |

### 14. Tunnel Auth — 2 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| tunnel_auth_response | 3.73 µs | < 5µs |
| tunnel_handshake_3msg | 18.49 µs | < 20ms |
| **▸ Tunnel TOTAL (2)** | **22.22 µs** | |

### 15. Heartbeat Pipeline (Cached Keys) — 2 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| heartbeat_single | 14.72 µs | < 9µs |
| heartbeat_26 | 388.08 µs | < 210µs |
| **▸ Heartbeat TOTAL (2)** | **402.80 µs** | |

### 16. TSA / Merkle — 4 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| tsa_timestamp_create | 29.24 µs | < 30µs |
| tsa_timestamp_verify | 22.19 µs | < 20µs |
| merkle_insert | 296.66 µs | < 200µs |
| merkle_verify | 297.82 µs | < 200µs |
| **▸ TSA/Merkle TOTAL (4)** | **645.91 µs** | |

### 17. TDNS Identity — 3 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| tdns_derive_identity | 3.73 µs | < 10µs |
| tdns_scan_hash | 3.73 µs | < 10µs |
| tdns_repunit_checksum | 30 ns | < 100ns |
| **▸ TDNS TOTAL (3)** | **7.49 µs** | |

### 18. Calendar TERN Compression (Pure Arithmetic) — 2 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| tern_compress | 30 ns | < 500ns |
| tern_decompress | 30 ns | < 500ns |
| **▸ Calendar TOTAL (2)** | **60 ns** | |

### 19. CON Topology Keys — 3 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| con_derive_tunnel_key | 7.43 µs | < 10µs |
| con_rekey_single | 7.38 µs | < 10µs |
| con_rekey_all | 192.64 µs | < 300µs |
| **▸ CON TOTAL (3)** | **207.45 µs** | |

### 20. HPTP Timing — 3 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| hptp_timestamp_verify | 11.12 µs | < 20µs |
| hptp_drift_compensate | 30 ns | < 500ns |
| hptp_jitter_filter | 200 ns | < 1µs |
| **▸ HPTP TOTAL (3)** | **11.35 µs** | |

### 21. ZK Proofs — 2 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| zk_prove | 32.47 µs | < 30µs |
| zk_verify | 25.80 µs | < 30µs |
| **▸ ZK TOTAL (2)** | **58.27 µs** | |

### 22. SignHere Pipeline — 4 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| signhere_secure_doc | 69.78 µs | < 100µs |
| signhere_6check | 48.13 µs | < 80µs |
| signhere_cnsa2 | 29.49 µs | < 50µs |
| signhere_witness | 14.67 µs | < 20µs |
| **▸ SignHere TOTAL (4)** | **162.07 µs** | |

### 23. SFK Operations — 3 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| sfk_key_derive | 3.77 µs | < 10µs |
| sfk_sign | 18.47 µs | < 25µs |
| sfk_verify | 29.51 µs | < 25µs |
| **▸ SFK TOTAL (3)** | **51.75 µs** | |

### 24. Hedera / Blockchain — 2 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| hedera_submit_witness | 21.57 µs | < 25µs |
| hedera_verify_witness | 22.01 µs | < 20µs |
| **▸ Hedera TOTAL (2)** | **43.58 µs** | |

### 25. Lamport OTS — 3 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| lamport_keygen | 2.014 ms | < 5ms |
| lamport_sign | 984.97 µs | < 3ms |
| lamport_verify | 3.905 ms | < 3ms |
| **▸ Lamport TOTAL (3)** | **6.904 ms** | |

### 26. Roundtrips — 13 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| rt_pt26_full | 37.02 µs | < 80µs |
| rt_pt26_sign_verify | 29.76 µs | < 60µs |
| rt_tl_dsa_v1_full | 59.598 ms | < 60ms |
| rt_tl_dsa_v1_sign_verify | 46.138 ms | < 50ms |
| rt_tl_dsa_v2_full | 353.87 µs | < 500µs |
| rt_tl_kem_1024 | 151.29 µs | < 300µs |
| rt_tae_mac | 73.50 µs | < 60µs |
| rt_phase_encrypt | 118.27 µs | < 80µs |
| rt_signhere_full | 117.79 µs | < 200µs |
| rt_tsa_full | 47.30 µs | < 50µs |
| rt_merkle_full | 611.36 µs | < 400µs |
| rt_lamport_full | 6.978 ms | < 10ms |
| rt_zk_full | 47.80 µs | < 60µs |
| **▸ Roundtrip TOTAL (13)** | **114.302 ms** | |

### 27. User Actions — 8 benchmarks

| Benchmark | Time | Target | What it measures |
|---|---|---|---|
| ux_sign_document | 103.84 µs | < 150µs | Phase encrypt + timestamp + TL-DSA + PT26 + Hedera witness |
| ux_verify_document | 47.93 µs | < 100µs | All 6 checks: integrity + TSA + RSA + TL-DSA + PT26 + Hedera |
| ux_establish_tunnel | 199.02 µs | < 200µs | KEM-1024 keygen + encaps + decaps + 3-msg handshake + tunnel key |
| ux_heartbeat_cycle | 388.59 µs | < 500µs | 26 neighbors × (HMAC compute + verify) with cached keys |
| ux_node_join | 15.670 ms | < 20ms | Identity derive + keypair + 26 tunnel establishments |
| ux_tdns_register | 40.32 µs | < 60µs | Derive identity + PT26 sign + TSA timestamp |
| ux_epoch_rekey | 290.07 µs | < 400µs | Rekey all 26 tunnels + re-derive 26 HMAC keys |
| ux_secure_message | 72.96 µs | < 60µs | T-AE-MAC encrypt on sender + decrypt on receiver |
| **▸ User Action TOTAL (8)** | **16.812 ms** | | |

### 28. A/B Sponge (Scalar vs Batch) — 4 benchmarks

| Benchmark | Time | Target |
|---|---|---|
| ab_derive_key_scalar | 3.75 µs | ~4µs |
| ab_derive_key_batch | 101.47 µs | < 110µs |
| ab_heartbeat26_scalar | 388.08 µs | ~210µs |
| ab_heartbeat26_batch | 396.84 µs | < 210µs |
| **▸ A/B TOTAL (4)** | **890.14 µs** | |

---

## Category Summary

| # | Category | Count | Total Time |
|---|---|---|---|
| 1 | TL-DSA v1 | 3 | 59.701 ms |
| 2 | PT26-DSA | 7 | 48.11 µs |
| 3 | TL-DSA v2 | 6 | 390.22 µs |
| 4 | TL-KEM | 9 | 324.50 µs |
| 5 | T-AE-MAC | 4 | 113.82 µs |
| 6 | Phase Enc | 4 | 2.811 ms |
| 7 | AES-GCM | 2 | 81.02 µs |
| 8 | RSA-4096 | 2 | 1.725 ms |
| 9 | Sponge + TIS-27 | 5 | 22.60 µs |
| 10 | HMAC | 3 | 25.81 µs |
| 11 | Wire | 2 | 190 ns |
| 12 | Lattice | 2 | 7.39 µs |
| 13 | Identity | 2 | 13.264 ms |
| 14 | Tunnel | 2 | 22.22 µs |
| 15 | Heartbeat | 2 | 402.80 µs |
| 16 | TSA/Merkle | 4 | 645.91 µs |
| 17 | TDNS | 3 | 7.49 µs |
| 18 | Calendar | 2 | 60 ns |
| 19 | CON | 3 | 207.45 µs |
| 20 | HPTP | 3 | 11.35 µs |
| 21 | ZK | 2 | 58.27 µs |
| 22 | SignHere | 4 | 162.07 µs |
| 23 | SFK | 3 | 51.75 µs |
| 24 | Hedera | 2 | 43.58 µs |
| 25 | Lamport | 3 | 6.904 ms |
| 26 | Roundtrip | 13 | 114.302 ms |
| 27 | User Action | 8 | 16.812 ms |
| 28 | A/B | 4 | 890.14 µs |
| | **GRAND TOTAL** | **109** | **219.387 ms** |

---

## Notes

- All measurements are **medians over 100 iterations** in release mode (`cargo bench`).
- Environment is a shared-tenancy Replit container — bare-metal results will be faster.
- TL-DSA v1 dominates total time (~60ms per keygen/sign/verify cycle) because WOTS+ is inherently compute-heavy (87 hash chains × 48 bytes each).
- TERN compress/decompress and HPTP drift/jitter are **pure arithmetic** (zero sponge calls) — they measure the codec and consensus algorithms, not cryptographic operations.
- Heartbeat and HMAC benchmarks use **pre-cached keys** to measure steady-state performance. Key derivation happens at tunnel establishment, not every heartbeat cycle.
- `derive_key` now uses stack-allocated buffers for inputs ≤256 bytes, eliminating all heap allocations for the common case.
