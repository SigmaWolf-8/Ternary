# PlenumNET / Salvi Framework — Benchmark Report

**Date**: 2026-03-14  
**Run ID**: `64C268A3`  
**Suite**: v6 (109 benchmarks, 100 iterations)  
**Architecture**: x86_64, AVX2  
**Rust**: 0.1.0  

---

## Executive Summary

| Metric | Value |
|---|---|
| Total benchmarks | 109 |
| **Production Grade** | **47/52** (90% pass) |
| In The Forge | 21/57 (37% pass) |
| PQ benchmarks | 96/109 |
| Grand total | **225.926 ms** |

### Grading Scale

| Grade | Criteria |
|---|---|
| Plenum+ | ≤ 50% of target (world-class) |
| Pass | ≤ target |
| *Pass | ≤ 115% of target (within noise) |
| FAIL | > 115% of target |
| DIAGNOSE | > 500% of target |

---

## Industry Comparison — Post-Quantum Signatures

| Scheme | Keygen | Sign | Verify | Roundtrip | Sig Size | Security Basis | Status |
|---|---|---|---|---|---|---|---|
| **PT26-DSA (Salvi)** | **7.45 µs** | **11.11 µs** | **18.38 µs** | **44.64 µs** | **71 B** | Ternary Hypercube Walk + sponge | **Measured this run** |
| ML-DSA-65 (Dilithium) | ~150 µs | ~300 µs | ~150 µs | ~600 µs | 3,309 B | Module-LWE | NIST FIPS 204 |
| ML-DSA-87 | ~300 µs | ~500 µs | ~300 µs | ~1,100 µs | 4,627 B | Module-LWE | NIST FIPS 204 |
| FALCON-512 | ~8 ms | ~500 µs | ~50 µs | ~8.5 ms | 666 B | NTRU lattice | NIST standard |
| SPHINCS+-128f | ~3 ms | ~8 ms | ~500 µs | ~11.5 ms | 17,088 B | Hash-based | NIST FIPS 205 |
| Ed25519 (classical) | ~30 µs | ~50 µs | ~100 µs | ~180 µs | 64 B | Elliptic curve | **NOT post-quantum** |

## Industry Comparison — Key Encapsulation

| Scheme | Keygen | Encaps | Decaps | Roundtrip | CT Size | Security Basis | Status |
|---|---|---|---|---|---|---|---|
| **TL-KEM-1024 (Salvi)** | **102.43 µs** | **25.46 µs** | **22.50 µs** | **152.14 µs** | ~1,568 B | Ternary Module-LWE | **Measured this run** |
| ML-KEM-1024 (Kyber) | ~150 µs | ~180 µs | ~170 µs | ~500 µs | 1,568 B | Module-LWE | NIST FIPS 203 |
| ML-KEM-768 | ~120 µs | ~140 µs | ~130 µs | ~390 µs | 1,088 B | Module-LWE | NIST FIPS 203 |

---

## Production Grade — 47/52 pass (90%) — 17.778 ms

### 1. PT26-DSA

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `pt26_keygen` | 7.45 µs | < 8µs | PQ | Pass |
| `pt26_sign` | 11.11 µs | < 18µs | PQ | Pass |
| `pt26_verify` | 18.38 µs | < 18µs | PQ | *Pass |
| `pt26_verify_parallel` | 11.11 µs | < 18µs | PQ | Pass |
| `pt26_trit_diff` | 30 ns | < 5ns |  | DIAGNOSE |
| `pt26_step_token` | 30 ns | < 5ns |  | DIAGNOSE |
| `pt26_walk_token` | 30 ns | < 5ns |  | DIAGNOSE |
| **▸ PT26-DSA TOTAL (7)** | **48.14 µs** | | | **[4/7]** |

### 2. TL-KEM

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `tl_kem_512_keygen` | 36.59 µs | < 50µs | PQ | Pass |
| `tl_kem_512_encaps` | 14.67 µs | < 30µs | PQ | Plenum+ |
| `tl_kem_512_decaps` | 10.95 µs | < 30µs | PQ | Plenum+ |
| `tl_kem_768_keygen` | 62.38 µs | < 80µs | PQ | Pass |
| `tl_kem_768_encaps` | 18.32 µs | < 50µs | PQ | Plenum+ |
| `tl_kem_768_decaps` | 18.08 µs | < 50µs | PQ | Plenum+ |
| `tl_kem_1024_keygen` | 102.43 µs | < 120µs | PQ | Pass |
| `tl_kem_1024_encaps` | 25.46 µs | < 80µs | PQ | Plenum+ |
| `tl_kem_1024_decaps` | 22.50 µs | < 80µs | PQ | Plenum+ |
| **▸ TL-KEM TOTAL (9)** | **311.38 µs** | | PQ | **[9/9]** |

### 3. Sponge

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `sponge_hash` | 5.55 µs | < 5µs | PQ | *Pass |
| `sponge_derive_key` | 3.75 µs | < 5µs | PQ | Pass |
| **▸ Sponge TOTAL (2)** | **9.30 µs** | | PQ | **[2/2]** |

### 4. Wire

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `wire_checksum` | 30 ns | < 100ns |  | Plenum+ |
| `wire_ecc` | 30 ns | < 100ns |  | Plenum+ |
| **▸ Wire TOTAL (2)** | **60 ns** | | | **[2/2]** |

### 5. TSA

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `tsa_timestamp_create` | 25.78 µs | < 30µs | PQ | Pass |
| `tsa_timestamp_verify` | 22.08 µs | < 20µs | PQ | *Pass |
| **▸ TSA TOTAL (2)** | **47.86 µs** | | PQ | **[2/2]** |

### 6. TDNS

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `tdns_derive_identity` | 3.75 µs | < 10µs | PQ | Plenum+ |
| `tdns_scan_hash` | 3.74 µs | < 10µs | PQ | Plenum+ |
| `tdns_repunit_checksum` | 30 ns | < 100ns |  | Plenum+ |
| **▸ TDNS TOTAL (3)** | **7.52 µs** | | | **[3/3]** |

### 7. Calendar

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `tern_compress` | 30 ns | < 500ns |  | Plenum+ |
| `tern_decompress` | 30 ns | < 500ns |  | Plenum+ |
| **▸ Calendar TOTAL (2)** | **60 ns** | | | **[2/2]** |

### 8. CON

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `con_derive_tunnel_key` | 7.33 µs | < 10µs | PQ | Pass |
| `con_rekey_single` | 7.32 µs | < 10µs | PQ | Pass |
| `con_rekey_all` | 192.20 µs | < 300µs | PQ | Pass |
| **▸ CON TOTAL (3)** | **206.85 µs** | | PQ | **[3/3]** |

### 9. HPTP

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `hptp_timestamp_verify` | 11.10 µs | < 20µs | PQ | Pass |
| `hptp_drift_compensate` | 30 ns | < 500ns |  | Plenum+ |
| `hptp_jitter_filter` | 190 ns | < 1µs |  | Plenum+ |
| **▸ HPTP TOTAL (3)** | **11.32 µs** | | | **[3/3]** |

### 10. ZK

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `zk_prove` | 22.14 µs | < 30µs | PQ | Pass |
| `zk_verify` | 25.92 µs | < 30µs | PQ | Pass |
| **▸ ZK TOTAL (2)** | **48.06 µs** | | PQ | **[2/2]** |

### 11. SignHere

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `signhere_secure_doc` | 69.29 µs | < 100µs | PQ | Pass |
| `signhere_6check` | 48.18 µs | < 80µs | PQ | Pass |
| `signhere_cnsa2` | 29.30 µs | < 50µs | PQ | Pass |
| `signhere_witness` | 14.59 µs | < 20µs | PQ | Pass |
| **▸ SignHere TOTAL (4)** | **161.36 µs** | | PQ | **[4/4]** |

### 12. SFK

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `sfk_key_derive` | 3.76 µs | < 10µs | PQ | Plenum+ |
| `sfk_sign` | 18.50 µs | < 25µs | PQ | Pass |
| `sfk_verify` | 29.12 µs | < 25µs | PQ | FAIL |
| **▸ SFK TOTAL (3)** | **51.38 µs** | | PQ | **[2/3]** |

### 13. Hedera

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `hedera_submit_witness` | 22.45 µs | < 25µs | PQ | Pass |
| `hedera_verify_witness` | 21.95 µs | < 20µs | PQ | *Pass |
| **▸ Hedera TOTAL (2)** | **44.40 µs** | | PQ | **[2/2]** |

### 14. User Action

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `ux_sign_document` | 84.01 µs | < 150µs | PQ | Pass |
| `ux_verify_document` | 55.96 µs | < 100µs | PQ | Pass |
| `ux_establish_tunnel` | 202.76 µs | < 200µs | PQ | *Pass |
| `ux_heartbeat_cycle` | 402.56 µs | < 500µs | PQ | Pass |
| `ux_node_join` | 15.681 ms | < 20ms | PQ | Pass |
| `ux_tdns_register` | 40.77 µs | < 60µs | PQ | Pass |
| `ux_epoch_rekey` | 289.71 µs | < 400µs | PQ | Pass |
| `ux_secure_message` | 73.32 µs | < 60µs | PQ | FAIL |
| **▸ User Action TOTAL (8)** | **16.830 ms** | | PQ | **[7/8]** |

---

## In The Forge — 21/57 pass (37%) — 208.148 ms

### 1. TL-DSA v1

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `tl_dsa_87_keygen` | 16.253 ms | < 3ms | PQ | DIAGNOSE |
| `tl_dsa_87_sign` | 21.378 ms | < 5ms | PQ | FAIL |
| `tl_dsa_87_verify` | 26.425 ms | < 3ms | PQ | DIAGNOSE |
| **▸ TL-DSA v1 TOTAL (3)** | **64.057 ms** | | PQ | **[0/3]** |

### 2. TL-DSA v2

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `tl_dsa_v2_ntt_butterfly` | 30 ns | < 20ns | PQ | FAIL |
| `tl_dsa_v2_ntt_full` | 3.70 µs | < 1µs | PQ | FAIL |
| `tl_dsa_v2_matrix_mul` | 38.60 µs | < 30µs | PQ | FAIL |
| `tl_dsa_v2_keygen` | 255.22 µs | < 100µs | PQ | FAIL |
| `tl_dsa_v2_sign` | 90.58 µs | < 50µs | PQ | FAIL |
| `tl_dsa_v2_verify` | 6.74 µs | < 30µs | PQ | Plenum+ |
| **▸ TL-DSA v2 TOTAL (6)** | **394.87 µs** | | PQ | **[1/6]** |

### 3. T-AE-MAC

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `tae_mac_encrypt` | 36.72 µs | < 30µs | PQ | FAIL |
| `tae_mac_decrypt` | 36.85 µs | < 30µs | PQ | FAIL |
| `tae_mac_compute` | 14.62 µs | < 15µs | PQ | Pass |
| `tae_mac_verify` | 25.36 µs | < 20µs | PQ | FAIL |
| **▸ T-AE-MAC TOTAL (4)** | **113.55 µs** | | PQ | **[1/4]** |

### 4. Phase Enc

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `phase_split` | 58.25 µs | < 40µs | PQ | FAIL |
| `phase_recombine` | 58.13 µs | < 40µs | PQ | FAIL |
| `phase_batch_split` | 1.451 ms | < 400µs | PQ | FAIL |
| `phase_batch_recombine` | 1.216 ms | < 400µs | PQ | FAIL |
| **▸ Phase Enc TOTAL (4)** | **2.783 ms** | | PQ | **[0/4]** |

### 5. AES-GCM

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `aes_gcm_encrypt` | 40.33 µs | < 25µs | PQ | FAIL |
| `aes_gcm_decrypt` | 40.46 µs | < 25µs | PQ | FAIL |
| **▸ AES-GCM TOTAL (2)** | **80.79 µs** | | PQ | **[0/2]** |

### 6. RSA-4096

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `rsa_4096_sign` | 1.470 ms | < 2ms |  | Pass |
| `rsa_4096_verify` | 269.14 µs | < 200µs |  | FAIL |
| **▸ RSA-4096 TOTAL (2)** | **1.739 ms** | | | **[1/2]** |

### 7. TIS-27

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `tis27_hash_27trit` | 3.73 µs | < 5µs | PQ | Pass |
| `tis27_hash_54trit` | 7.31 µs | < 5µs | PQ | FAIL |
| `tis27_absorb_squeeze` | 10.93 µs | < 8µs | PQ | FAIL |
| **▸ TIS-27 TOTAL (3)** | **21.97 µs** | | PQ | **[1/3]** |

### 8. HMAC

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `hmac_key_derive` | 3.79 µs | < 5µs | PQ | Pass |
| `hmac_compute` | 7.30 µs | < 5µs | PQ | FAIL |
| `hmac_verify` | 14.09 µs | < 10µs | PQ | FAIL |
| **▸ HMAC TOTAL (3)** | **25.18 µs** | | PQ | **[1/3]** |

### 9. Lattice

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `lattice_nonce` | 30 ns | < 100ns |  | Plenum+ |
| `lattice_key_derive` | 7.30 µs | < 5µs | PQ | FAIL |
| **▸ Lattice TOTAL (2)** | **7.33 µs** | | | **[1/2]** |

### 10. Identity

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `identity_seed_derive` | 14.48 µs | < 5µs | PQ | FAIL |
| `identity_keypair_derive` | 13.160 ms | < 5ms | PQ | FAIL |
| **▸ Identity TOTAL (2)** | **13.174 ms** | | PQ | **[0/2]** |

### 11. Tunnel

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `tunnel_auth_response` | 10.94 µs | < 5µs | PQ | FAIL |
| `tunnel_handshake_3msg` | 40.96 µs | < 20ms | PQ | Plenum+ |
| **▸ Tunnel TOTAL (2)** | **51.90 µs** | | PQ | **[1/2]** |

### 12. Heartbeat

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `heartbeat_single` | 14.55 µs | < 9µs | PQ | FAIL |
| `heartbeat_26` | 388.93 µs | < 210µs | PQ | FAIL |
| **▸ Heartbeat TOTAL (2)** | **403.48 µs** | | PQ | **[0/2]** |

### 13. Merkle

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `merkle_insert` | 304.11 µs | < 200µs | PQ | FAIL |
| `merkle_verify` | 300.87 µs | < 200µs | PQ | FAIL |
| **▸ Merkle TOTAL (2)** | **604.98 µs** | | PQ | **[0/2]** |

### 14. Lamport

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `lamport_keygen` | 1.960 ms | < 5ms | PQ | Plenum+ |
| `lamport_sign` | 986.02 µs | < 3ms | PQ | Plenum+ |
| `lamport_verify` | 4.171 ms | < 3ms | PQ | FAIL |
| **▸ Lamport TOTAL (3)** | **7.117 ms** | | PQ | **[2/3]** |

### 15. Roundtrip

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `rt_pt26_full` | 44.64 µs | < 80µs | PQ | Pass |
| `rt_pt26_sign_verify` | 29.45 µs | < 60µs | PQ | Plenum+ |
| `rt_tl_dsa_v1_full` | 60.378 ms | < 60ms | PQ | *Pass |
| `rt_tl_dsa_v1_sign_verify` | 47.427 ms | < 50ms | PQ | Pass |
| `rt_tl_dsa_v2_full` | 355.04 µs | < 500µs | PQ | Pass |
| `rt_tl_kem_1024` | 152.14 µs | < 300µs | PQ | Pass |
| `rt_tae_mac` | 73.62 µs | < 60µs | PQ | FAIL |
| `rt_phase_encrypt` | 120.16 µs | < 80µs | PQ | FAIL |
| `rt_signhere_full` | 119.70 µs | < 200µs | PQ | Pass |
| `rt_tsa_full` | 48.05 µs | < 50µs | PQ | Pass |
| `rt_merkle_full` | 621.14 µs | < 400µs | PQ | FAIL |
| `rt_lamport_full` | 7.250 ms | < 10ms | PQ | Pass |
| `rt_zk_full` | 48.01 µs | < 60µs | PQ | Pass |
| **▸ Roundtrip TOTAL (13)** | **116.668 ms** | | PQ | **[10/13]** |

### 16. A/B

| Benchmark | Time | Target | PQ | Grade |
|---|---|---|---|---|
| `ab_derive_key_scalar` | 3.77 µs | ~4µs | PQ | Pass |
| `ab_derive_key_batch` | 103.43 µs | < 110µs | PQ | Pass |
| `ab_heartbeat26_scalar` | 389.37 µs | ~210µs | PQ | FAIL |
| `ab_heartbeat26_batch` | 408.90 µs | < 210µs | PQ | FAIL |
| **▸ A/B TOTAL (4)** | **905.47 µs** | | PQ | **[2/4]** |

---

## Category Summary

| # | Category | Status | Count | Total | PQ | Pass Rate |
|---|---|---|---|---|---|---|
| 1 | TL-DSA v1 | 🔧 Forge | 3 | 64.057 ms | PQ | 0/3 |
| 2 | PT26-DSA | ✅ Production | 7 | 48.14 µs |  | 4/7 |
| 3 | TL-DSA v2 | 🔧 Forge | 6 | 394.87 µs | PQ | 1/6 |
| 4 | TL-KEM | ✅ Production | 9 | 311.38 µs | PQ | 9/9 |
| 5 | T-AE-MAC | 🔧 Forge | 4 | 113.55 µs | PQ | 1/4 |
| 6 | Phase Enc | 🔧 Forge | 4 | 2.783 ms | PQ | 0/4 |
| 7 | AES-GCM | 🔧 Forge | 2 | 80.79 µs | PQ | 0/2 |
| 8 | RSA-4096 | 🔧 Forge | 2 | 1.739 ms |  | 1/2 |
| 9 | Sponge | ✅ Production | 2 | 9.30 µs | PQ | 2/2 |
| 10 | TIS-27 | 🔧 Forge | 3 | 21.97 µs | PQ | 1/3 |
| 11 | HMAC | 🔧 Forge | 3 | 25.18 µs | PQ | 1/3 |
| 12 | Wire | ✅ Production | 2 | 60 ns |  | 2/2 |
| 13 | Lattice | 🔧 Forge | 2 | 7.33 µs |  | 1/2 |
| 14 | Identity | 🔧 Forge | 2 | 13.174 ms | PQ | 0/2 |
| 15 | Tunnel | 🔧 Forge | 2 | 51.90 µs | PQ | 1/2 |
| 16 | Heartbeat | 🔧 Forge | 2 | 403.48 µs | PQ | 0/2 |
| 17 | TSA | ✅ Production | 2 | 47.86 µs | PQ | 2/2 |
| 18 | Merkle | 🔧 Forge | 2 | 604.98 µs | PQ | 0/2 |
| 19 | TDNS | ✅ Production | 3 | 7.52 µs |  | 3/3 |
| 20 | Calendar | ✅ Production | 2 | 60 ns |  | 2/2 |
| 21 | CON | ✅ Production | 3 | 206.85 µs | PQ | 3/3 |
| 22 | HPTP | ✅ Production | 3 | 11.32 µs |  | 3/3 |
| 23 | ZK | ✅ Production | 2 | 48.06 µs | PQ | 2/2 |
| 24 | SignHere | ✅ Production | 4 | 161.36 µs | PQ | 4/4 |
| 25 | SFK | ✅ Production | 3 | 51.38 µs | PQ | 2/3 |
| 26 | Hedera | ✅ Production | 2 | 44.40 µs | PQ | 2/2 |
| 27 | Lamport | 🔧 Forge | 3 | 7.117 ms | PQ | 2/3 |
| 28 | Roundtrip | 🔧 Forge | 13 | 116.668 ms | PQ | 10/13 |
| 29 | User Action | ✅ Production | 8 | 16.830 ms | PQ | 7/8 |
| 30 | A/B | 🔧 Forge | 4 | 905.47 µs | PQ | 2/4 |
| | **GRAND TOTAL** | | **109** | **225.926 ms** | **96 PQ** | **68/109** |

---

*Generated by inter_cube v6 benchmark runner*  
*PQ = Post-quantum verifiable (sponge pre-image, lattice, hash-based)*
