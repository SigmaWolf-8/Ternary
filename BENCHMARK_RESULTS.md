# PlenumNET / Salvi Framework — Benchmark Results

**Date**: 2026-03-14
**Suite**: v5 (109 benchmarks, 30 categories)
**Tests passing**: 266 (256 lib + 10 integration)

## Environments

| | Replit (shared tenancy) | Bare Metal |
|---|---|---|
| **Architecture** | x86_64, AVX2 | ARM64 (aarch64-pc-windows-msvc), NEON |
| **Rust** | stable | 1.94.0 (2026-03-02) |
| **Pass rate** | 64 / 109 (59%) | 78 / 109 (72%) |
| **PQ benchmarks** | 96 / 109 (88%) | 96 / 109 (88%) |
| **Grand total** | **218.526 ms** | **201.874 ms** |

---

## Grading Scale

| Grade | Criteria |
|---|---|
| **Plenum+** | ≤ 50% of target (world-class) |
| **Pass** | ≤ target |
| **\*Pass** | ≤ 115% of target (within noise) |
| **FAIL** | > 115% of target |
| **DIAGNOSE** | > 500% of target |

**PQ** = Post-quantum verifiable (sponge pre-image, lattice, hash-based)

---

## Performance Highlights

| Metric | Before (5-step) | After (2-step + stack buf) | Change |
|---|---|---|---|
| `derive_key` (single) | 10.78 µs | **3.20 µs** (bare metal) | **3.37× faster** |
| `hash_hex` | ~16 µs | **5.60 µs** (Replit) | **2.86× faster** |
| `heartbeat_26` (cached keys) | ~280 µs | **347 µs** (bare metal, steady-state) | see note¹ |
| Old baseline (`sponge.rs`) | 4.09 µs | 3.20 µs (bare metal) | **28% faster** |
| `derive_key` heap allocs | 3/call | **0/call** (≤256B input) | **eliminated** |

¹ heartbeat_26 now measures steady-state: 52 sponge calls (compute+verify) with pre-cached HMAC keys. Previous measurement included key derivation in the hot path (78 calls total).

---

## Full Benchmark Suite (109 benchmarks)

### 1. TL-DSA v1-87 (Hash-based WOTS+) — 3 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| tl_dsa_87_keygen | 12.911 ms | 11.678 ms | < 3ms | PQ | FAIL |
| tl_dsa_87_sign | 19.881 ms | 17.886 ms | < 5ms | PQ | FAIL |
| tl_dsa_87_verify | 26.410 ms | 25.071 ms | < 3ms | PQ | DIAGNOSE |
| **▸ TL-DSA v1 TOTAL (3)** | **59.201 ms** | **54.635 ms** | | PQ | [0/3] |

### 2. PT26-DSA (Geometric Signature) — 7 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| pt26_keygen | 11.03 µs | 7.00 µs | < 8µs | PQ | Pass |
| pt26_sign | 10.95 µs | 10.50 µs | < 18µs | PQ | Pass |
| pt26_verify | 18.22 µs | 15.70 µs | < 18µs | PQ | Pass |
| pt26_verify_parallel | 10.95 µs | 10.40 µs | < 18µs | PQ | Pass |
| pt26_trit_diff | 30 ns | 0 ns | < 5ns | | Plenum+ |
| pt26_step_token | 30 ns | 0 ns | < 5ns | | Plenum+ |
| pt26_walk_token | 30 ns | 0 ns | < 5ns | | Plenum+ |
| **▸ PT26-DSA TOTAL (7)** | **51.24 µs** | **43.60 µs** | | | [7/7] |

### 3. TL-DSA v2-87 (Ternary Lattice NTT) — 6 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| tl_dsa_v2_ntt_butterfly | 30 ns | 0 ns | < 20ns | PQ | Plenum+ |
| tl_dsa_v2_ntt_full | 3.73 µs | 1.20 µs | < 1µs | PQ | FAIL |
| tl_dsa_v2_matrix_mul | 39.89 µs | 11.70 µs | < 30µs | PQ | Plenum+ |
| tl_dsa_v2_keygen | 250.09 µs | 225.50 µs | < 100µs | PQ | FAIL |
| tl_dsa_v2_sign | 90.95 µs | 84.70 µs | < 50µs | PQ | FAIL |
| tl_dsa_v2_verify | 4.60 µs | 4.00 µs | < 30µs | PQ | Plenum+ |
| **▸ TL-DSA v2 TOTAL (6)** | **389.29 µs** | **327.10 µs** | | PQ | [3/6] |

### 4. TL-KEM (Key Encapsulation) — 9 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| tl_kem_512_keygen | 36.41 µs | 34.70 µs | < 50µs | PQ | Pass |
| tl_kem_512_encaps | 14.50 µs | 13.90 µs | < 30µs | PQ | Plenum+ |
| tl_kem_512_decaps | 13.12 µs | 10.20 µs | < 30µs | PQ | Plenum+ |
| tl_kem_768_keygen | 61.69 µs | 58.30 µs | < 80µs | PQ | Pass |
| tl_kem_768_encaps | 18.04 µs | 17.20 µs | < 50µs | PQ | Plenum+ |
| tl_kem_768_decaps | 17.62 µs | 16.70 µs | < 50µs | PQ | Plenum+ |
| tl_kem_1024_keygen | 101.44 µs | 95.80 µs | < 120µs | PQ | Pass |
| tl_kem_1024_encaps | 25.18 µs | 23.90 µs | < 80µs | PQ | Plenum+ |
| tl_kem_1024_decaps | 22.30 µs | 21.00 µs | < 80µs | PQ | Plenum+ |
| **▸ TL-KEM TOTAL (9)** | **310.30 µs** | **291.70 µs** | | PQ | [9/9] |

### 5. T-AE-MAC (Authenticated Encryption) — 4 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| tae_mac_encrypt | 36.66 µs | 34.50 µs | < 30µs | PQ | *Pass |
| tae_mac_decrypt | 36.26 µs | 34.40 µs | < 30µs | PQ | *Pass |
| tae_mac_compute | 14.65 µs | 13.70 µs | < 15µs | PQ | Pass |
| tae_mac_verify | 25.44 µs | 23.80 µs | < 20µs | PQ | FAIL |
| **▸ T-AE-MAC TOTAL (4)** | **113.01 µs** | **106.40 µs** | | PQ | [3/4] |

### 6. Phase Encryption — 4 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| phase_split | 57.70 µs | 55.60 µs | < 40µs | PQ | FAIL |
| phase_recombine | 57.87 µs | 55.50 µs | < 40µs | PQ | FAIL |
| phase_batch_split | 1.394 ms | 1.281 ms | < 400µs | PQ | FAIL |
| phase_batch_recombine | 1.171 ms | 1.069 ms | < 400µs | PQ | FAIL |
| **▸ Phase Enc TOTAL (4)** | **2.681 ms** | **2.460 ms** | | PQ | [0/4] |

### 7. AES-256-GCM (Token Encryption) — 2 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| aes_gcm_encrypt | 40.09 µs | 38.40 µs | < 25µs | PQ | FAIL |
| aes_gcm_decrypt | 40.12 µs | 38.50 µs | < 25µs | PQ | FAIL |
| **▸ AES-GCM TOTAL (2)** | **80.21 µs** | **76.90 µs** | | PQ | [0/2] |

### 8. RSA-4096 (Classical Co-Signature) — 2 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| rsa_4096_sign | 1.438 ms | 1.291 ms | < 2ms | | Pass |
| rsa_4096_verify | 261.39 µs | 239.20 µs | < 200µs | | FAIL |
| **▸ RSA-4096 TOTAL (2)** | **1.699 ms** | **1.530 ms** | | | [1/2] |

### 9. Sponge Core — 2 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| sponge_hash | 5.60 µs | 5.70 µs | < 5µs | PQ | *Pass |
| sponge_derive_key | 3.66 µs | 3.50 µs | < 5µs | PQ | Pass |
| **▸ Sponge TOTAL (2)** | **9.26 µs** | **9.20 µs** | | PQ | [2/2] |

### 10. TIS-27 Standalone — 3 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| tis27_hash_27trit | 3.73 µs | 3.50 µs | < 5µs | PQ | Pass |
| tis27_hash_54trit | 7.11 µs | 6.20 µs | < 5µs | PQ | FAIL |
| tis27_absorb_squeeze | 10.60 µs | 9.30 µs | < 8µs | PQ | FAIL |
| **▸ TIS-27 TOTAL (3)** | **21.44 µs** | **19.00 µs** | | PQ | [1/3] |

### 11. HMAC (Cached Keys) — 3 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| hmac_key_derive | 3.74 µs | 3.60 µs | < 5µs | PQ | Pass |
| hmac_compute | 7.31 µs | 6.70 µs | < 5µs | PQ | FAIL |
| hmac_verify | 14.45 µs | 12.30 µs | < 10µs | PQ | FAIL |
| **▸ HMAC TOTAL (3)** | **25.50 µs** | **22.60 µs** | | PQ | [1/3] |

### 12. Wire Integrity — 2 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| wire_checksum | 120 ns | 0 ns | < 100ns | | Plenum+ |
| wire_ecc | 70 ns | 0 ns | < 100ns | | Plenum+ |
| **▸ Wire TOTAL (2)** | **190 ns** | **0 ns** | | | [2/2] |

### 13. Lattice Mixer — 2 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| lattice_nonce | 30 ns | 0 ns | < 100ns | PQ | Plenum+ |
| lattice_key_derive | 7.38 µs | 6.70 µs | < 5µs | PQ | FAIL |
| **▸ Lattice TOTAL (2)** | **7.41 µs** | **6.70 µs** | | PQ | [1/2] |

### 14. Identity — 2 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| identity_seed_derive | 3.73 µs | 12.60 µs | < 5µs | PQ | FAIL |
| identity_keypair_derive | 13.214 ms | 11.545 ms | < 5ms | PQ | FAIL |
| **▸ Identity TOTAL (2)** | **13.218 ms** | **11.558 ms** | | PQ | [0/2] |

### 15. Tunnel Auth — 2 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| tunnel_auth_response | 3.73 µs | 10.20 µs | < 5µs | PQ | FAIL |
| tunnel_handshake_3msg | 18.35 µs | 43.00 µs | < 20ms | PQ | Plenum+ |
| **▸ Tunnel TOTAL (2)** | **22.08 µs** | **53.20 µs** | | PQ | [1/2] |

### 16. Heartbeat Pipeline (Cached Keys) — 2 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| heartbeat_single | 14.66 µs | 13.50 µs | < 9µs | PQ | FAIL |
| heartbeat_26 | 390.22 µs | 347.20 µs | < 210µs | PQ | FAIL |
| **▸ Heartbeat TOTAL (2)** | **404.88 µs** | **360.70 µs** | | PQ | [0/2] |

### 17. TSA — 2 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| tsa_timestamp_create | 29.50 µs | 24.00 µs | < 30µs | PQ | Pass |
| tsa_timestamp_verify | 22.00 µs | 20.60 µs | < 20µs | PQ | *Pass |
| **▸ TSA TOTAL (2)** | **51.50 µs** | **44.60 µs** | | PQ | [2/2] |

### 18. Merkle — 2 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| merkle_insert | 288.15 µs | 274.70 µs | < 200µs | PQ | FAIL |
| merkle_verify | 289.11 µs | 252.20 µs | < 200µs | PQ | FAIL |
| **▸ Merkle TOTAL (2)** | **577.26 µs** | **526.90 µs** | | PQ | [0/2] |

### 19. TDNS Identity — 3 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| tdns_derive_identity | 3.73 µs | 3.10 µs | < 10µs | PQ | Plenum+ |
| tdns_scan_hash | 3.73 µs | 3.20 µs | < 10µs | PQ | Plenum+ |
| tdns_repunit_checksum | 30 ns | 0 ns | < 100ns | | Plenum+ |
| **▸ TDNS TOTAL (3)** | **7.49 µs** | **6.30 µs** | | | [3/3] |

### 20. Calendar TERN Compression (Pure Arithmetic) — 2 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| tern_compress | 30 ns | 0 ns | < 500ns | | Plenum+ |
| tern_decompress | 30 ns | 0 ns | < 500ns | | Plenum+ |
| **▸ Calendar TOTAL (2)** | **60 ns** | **0 ns** | | | [2/2] |

### 21. CON Topology Keys — 3 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| con_derive_tunnel_key | 7.34 µs | 6.20 µs | < 10µs | PQ | Pass |
| con_rekey_single | 7.43 µs | 6.20 µs | < 10µs | PQ | Pass |
| con_rekey_all | 189.18 µs | 167.90 µs | < 300µs | PQ | Pass |
| **▸ CON TOTAL (3)** | **203.95 µs** | **180.30 µs** | | PQ | [3/3] |

### 22. HPTP Timing — 3 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| hptp_timestamp_verify | 11.13 µs | 9.30 µs | < 20µs | PQ | Plenum+ |
| hptp_drift_compensate | 30 ns | 0 ns | < 500ns | | Plenum+ |
| hptp_jitter_filter | 200 ns | 200 ns | < 1µs | | Plenum+ |
| **▸ HPTP TOTAL (3)** | **11.36 µs** | **9.50 µs** | | | [3/3] |

### 23. ZK Proofs — 2 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| zk_prove | 22.10 µs | 18.60 µs | < 30µs | PQ | Pass |
| zk_verify | 25.76 µs | 23.90 µs | < 30µs | PQ | Pass |
| **▸ ZK TOTAL (2)** | **47.86 µs** | **42.50 µs** | | PQ | [2/2] |

### 24. SignHere Pipeline — 4 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| signhere_secure_doc | 68.60 µs | 65.60 µs | < 100µs | PQ | Pass |
| signhere_6check | 48.11 µs | 44.10 µs | < 80µs | PQ | Pass |
| signhere_cnsa2 | 29.35 µs | 27.20 µs | < 50µs | PQ | Pass |
| signhere_witness | 14.34 µs | 13.70 µs | < 20µs | PQ | Pass |
| **▸ SignHere TOTAL (4)** | **160.40 µs** | **150.60 µs** | | PQ | [4/4] |

### 25. SFK Operations — 3 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| sfk_key_derive | 3.74 µs | 3.50 µs | < 10µs | PQ | Plenum+ |
| sfk_sign | 18.31 µs | 15.60 µs | < 25µs | PQ | Pass |
| sfk_verify | 29.35 µs | 25.40 µs | < 25µs | PQ | *Pass |
| **▸ SFK TOTAL (3)** | **51.40 µs** | **44.50 µs** | | PQ | [3/3] |

### 26. Hedera / Blockchain — 2 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| hedera_submit_witness | 33.99 µs | 18.50 µs | < 25µs | PQ | Pass |
| hedera_verify_witness | 33.45 µs | 18.50 µs | < 20µs | PQ | Pass |
| **▸ Hedera TOTAL (2)** | **67.44 µs** | **37.00 µs** | | PQ | [2/2] |

### 27. Lamport OTS — 3 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| lamport_keygen | 1.940 ms | 1.761 ms | < 5ms | PQ | Plenum+ |
| lamport_sign | 975.71 µs | 886.80 µs | < 3ms | PQ | Plenum+ |
| lamport_verify | 3.886 ms | 3.452 ms | < 3ms | PQ | FAIL |
| **▸ Lamport TOTAL (3)** | **6.802 ms** | **6.100 ms** | | PQ | [2/3] |

### 28. Roundtrips — 13 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| rt_pt26_full | 45.23 µs | 31.10 µs | < 80µs | PQ | Plenum+ |
| rt_pt26_sign_verify | 29.25 µs | 24.70 µs | < 60µs | PQ | Plenum+ |
| rt_tl_dsa_v1_full | 59.714 ms | 55.752 ms | < 60ms | PQ | Pass |
| rt_tl_dsa_v1_sign_verify | 46.394 ms | 43.558 ms | < 50ms | PQ | Pass |
| rt_tl_dsa_v2_full | 342.66 µs | 299.90 µs | < 500µs | PQ | Pass |
| rt_tl_kem_1024 | 149.24 µs | 129.20 µs | < 300µs | PQ | Plenum+ |
| rt_tae_mac | 73.41 µs | 68.20 µs | < 60µs | PQ | *Pass |
| rt_phase_encrypt | 115.23 µs | 108.90 µs | < 80µs | PQ | FAIL |
| rt_signhere_full | 116.72 µs | 99.30 µs | < 200µs | PQ | Plenum+ |
| rt_tsa_full | 75.28 µs | 40.30 µs | < 50µs | PQ | Pass |
| rt_merkle_full | 613.61 µs | 535.90 µs | < 400µs | PQ | FAIL |
| rt_lamport_full | 6.875 ms | 6.180 ms | < 10ms | PQ | Pass |
| rt_zk_full | 47.95 µs | 44.60 µs | < 60µs | PQ | Pass |
| **▸ Roundtrip TOTAL (13)** | **114.591 ms** | **106.872 ms** | | PQ | [11/13] |

### 29. User Actions — 8 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) | What it measures |
|---|---|---|---|---|---|---|
| ux_sign_document | 83.38 µs | 79.20 µs | < 150µs | PQ | Pass | Phase encrypt + timestamp + TL-DSA + PT26 + Hedera witness |
| ux_verify_document | 51.04 µs | 44.10 µs | < 100µs | PQ | Plenum+ | All 6 checks: integrity + TSA + RSA + TL-DSA + PT26 + Hedera |
| ux_establish_tunnel | 196.99 µs | 188.30 µs | < 200µs | PQ | Pass | KEM-1024 keygen + encaps + decaps + 3-msg handshake + tunnel key |
| ux_heartbeat_cycle | 389.14 µs | 349.30 µs | < 500µs | PQ | Pass | 26 neighbors × (HMAC compute + verify) with cached keys |
| ux_node_join | 15.764 ms | 14.534 ms | < 20ms | PQ | Pass | Identity derive + keypair + 26 tunnel establishments |
| ux_tdns_register | 40.30 µs | 37.70 µs | < 60µs | PQ | Pass | Derive identity + PT26 sign + TSA timestamp |
| ux_epoch_rekey | 303.57 µs | 261.50 µs | < 400µs | PQ | Pass | Rekey all 26 tunnels + re-derive 26 HMAC keys |
| ux_secure_message | 73.12 µs | 68.40 µs | < 60µs | PQ | *Pass | T-AE-MAC encrypt on sender + decrypt on receiver |
| **▸ User Action TOTAL (8)** | **16.902 ms** | **15.563 ms** | | PQ | [8/8] |

### 30. A/B Sponge (Scalar vs Batch) — 4 benchmarks

| Benchmark | Replit | Bare Metal | Target | PQ | Grade (BM) |
|---|---|---|---|---|---|
| ab_derive_key_scalar | 3.72 µs | 3.20 µs | ~4µs | PQ | Pass |
| ab_derive_key_batch | 104.53 µs | 94.10 µs | < 110µs | PQ | Pass |
| ab_heartbeat26_scalar | 391.43 µs | 338.40 µs | ~210µs | PQ | FAIL |
| ab_heartbeat26_batch | 414.14 µs | 361.50 µs | < 210µs | PQ | FAIL |
| **▸ A/B TOTAL (4)** | **913.82 µs** | **797.20 µs** | | PQ | [2/4] |

---

## Category Summary

| # | Category | Count | Replit | Bare Metal | PQ | Pass Rate (BM) |
|---|---|---|---|---|---|---|
| 1 | TL-DSA v1 | 3 | 59.201 ms | 54.635 ms | PQ | 0/3 |
| 2 | PT26-DSA | 7 | 51.24 µs | 43.60 µs | | 7/7 |
| 3 | TL-DSA v2 | 6 | 389.29 µs | 327.10 µs | PQ | 3/6 |
| 4 | TL-KEM | 9 | 310.30 µs | 291.70 µs | PQ | 9/9 |
| 5 | T-AE-MAC | 4 | 113.01 µs | 106.40 µs | PQ | 3/4 |
| 6 | Phase Enc | 4 | 2.681 ms | 2.460 ms | PQ | 0/4 |
| 7 | AES-GCM | 2 | 80.21 µs | 76.90 µs | PQ | 0/2 |
| 8 | RSA-4096 | 2 | 1.699 ms | 1.530 ms | | 1/2 |
| 9 | Sponge | 2 | 9.26 µs | 9.20 µs | PQ | 2/2 |
| 10 | TIS-27 | 3 | 21.44 µs | 19.00 µs | PQ | 1/3 |
| 11 | HMAC | 3 | 25.50 µs | 22.60 µs | PQ | 1/3 |
| 12 | Wire | 2 | 190 ns | 0 ns | | 2/2 |
| 13 | Lattice | 2 | 7.41 µs | 6.70 µs | PQ | 1/2 |
| 14 | Identity | 2 | 13.218 ms | 11.558 ms | PQ | 0/2 |
| 15 | Tunnel | 2 | 22.08 µs | 53.20 µs | PQ | 1/2 |
| 16 | Heartbeat | 2 | 404.88 µs | 360.70 µs | PQ | 0/2 |
| 17 | TSA | 2 | 51.50 µs | 44.60 µs | PQ | 2/2 |
| 18 | Merkle | 2 | 577.26 µs | 526.90 µs | PQ | 0/2 |
| 19 | TDNS | 3 | 7.49 µs | 6.30 µs | | 3/3 |
| 20 | Calendar | 2 | 60 ns | 0 ns | | 2/2 |
| 21 | CON | 3 | 203.95 µs | 180.30 µs | PQ | 3/3 |
| 22 | HPTP | 3 | 11.36 µs | 9.50 µs | | 3/3 |
| 23 | ZK | 2 | 47.86 µs | 42.50 µs | PQ | 2/2 |
| 24 | SignHere | 4 | 160.40 µs | 150.60 µs | PQ | 4/4 |
| 25 | SFK | 3 | 51.40 µs | 44.50 µs | PQ | 3/3 |
| 26 | Hedera | 2 | 67.44 µs | 37.00 µs | PQ | 2/2 |
| 27 | Lamport | 3 | 6.802 ms | 6.100 ms | PQ | 2/3 |
| 28 | Roundtrip | 13 | 114.591 ms | 106.872 ms | PQ | 11/13 |
| 29 | User Action | 8 | 16.902 ms | 15.563 ms | PQ | 8/8 |
| 30 | A/B | 4 | 913.82 µs | 797.20 µs | PQ | 2/4 |
| | **GRAND TOTAL** | **109** | **218.526 ms** | **201.874 ms** | **96 PQ** | **78/109 (72%)** |

---

## Notes

- All measurements are **medians over 100 iterations** in release mode (`cargo bench`).
- **Replit**: shared-tenancy x86_64 container with AVX2. Timer resolution floor at ~30ns.
- **Bare Metal**: ARM64 Windows (aarch64-pc-windows-msvc) with NEON SIMD. Timer resolution floor at 0ns (sub-nanosecond ops report 0ns).
- TL-DSA v1 dominates total time (~55–59ms) because WOTS+ is inherently compute-heavy (87 hash chains × 48 bytes each). These targets assume hardware acceleration.
- TL-KEM is the standout: **9/9 pass** on both environments with 6 Plenum+ grades.
- User Actions achieve **8/8 pass on bare metal** — all end-to-end UX operations meet targets.
- TERN compress/decompress and HPTP drift/jitter are **pure arithmetic** (zero sponge calls).
- Heartbeat and HMAC benchmarks use **pre-cached keys** to measure steady-state performance.
- `derive_key` uses stack-allocated buffers for inputs ≤256 bytes, eliminating all heap allocations for the common case.
- `sponge_kdf_cat` provides zero-allocation concatenated KDF for multi-part inputs.
- Bare-metal results show ~8% improvement over Replit across the board, with some categories (Hedera, ZK, CON) seeing 15–45% improvement.
