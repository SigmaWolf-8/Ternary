# PlenumNET / Salvi Framework — Complete Benchmark Report

**Date:** 2026-03-14
**Toolchain:** Rust 1.77.2, `cargo bench --release` (Criterion)
**Platform:** Linux x86_64 (Replit container)
**Suite:** `ternary-math/benches/inter_cube.rs` — 1,398 lines, 104 benchmarks, 30 categories

---

## 1. TL-DSA v1-87 (Hash-based WOTS+) — 3 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `tl_dsa_87_keygen` | 13.577 ms | < 3 ms |
| `tl_dsa_87_sign` | 20.640 ms | < 5 ms |
| `tl_dsa_87_verify` | 26.799 ms | < 3 ms |

## 2. PT26-DSA (Geometric Signature) — 7 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `pt26_keygen` | 8.114 µs | < 8 µs |
| `pt26_sign` | 11.983 µs | < 18 µs |
| `pt26_verify` | 19.937 µs | < 18 µs |
| `pt26_verify_parallel` | 12.109 µs | < 18 µs |
| `pt26_trit_diff` | 621.7 ps | < 5 ns |
| `pt26_step_token` | 314.1 ps | < 5 ns |
| `pt26_walk_token` | 312.8 ps | < 5 ns |

## 3. TL-DSA v2-87 (Ternary Lattice NTT) — 6 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `tl_dsa_v2_ntt_butterfly` | 697.6 ps | < 20 ns |
| `tl_dsa_v2_ntt_full_243` | 2.843 µs | < 1 µs |
| `tl_dsa_v2_matrix_mul` | 35.686 µs | < 30 µs |
| `tl_dsa_v2_keygen` | 266.55 µs | < 100 µs |
| `tl_dsa_v2_sign` | 94.399 µs | < 50 µs |
| `tl_dsa_v2_verify` | 5.072 µs | < 30 µs |

## 4. TL-KEM (Key Encapsulation) — 9 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `tl_kem_512_keygen` | 38.553 µs | < 50 µs |
| `tl_kem_512_encaps` | 15.683 µs | < 30 µs |
| `tl_kem_512_decaps` | 11.743 µs | < 30 µs |
| `tl_kem_768_keygen` | 65.843 µs | < 80 µs |
| `tl_kem_768_encaps` | 19.923 µs | < 50 µs |
| `tl_kem_768_decaps` | 19.972 µs | < 50 µs |
| `tl_kem_1024_keygen` | 108.83 µs | < 120 µs |
| `tl_kem_1024_encaps` | 27.473 µs | < 80 µs |
| `tl_kem_1024_decaps` | 23.573 µs | < 80 µs |

## 5. T-AE-MAC (Authenticated Encryption) — 4 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `tae_mac_encrypt` | 39.780 µs | < 30 µs |
| `tae_mac_decrypt` | 40.043 µs | < 30 µs |
| `tae_mac_compute` | 16.437 µs | < 15 µs |
| `tae_mac_verify` | 28.498 µs | < 20 µs |

## 6. Phase Encryption — 4 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `phase_split` | 61.849 µs | < 40 µs |
| `phase_recombine` | 63.784 µs | < 40 µs |
| `phase_batch_split` | 1.452 ms | < 400 µs |
| `phase_batch_recombine` | 1.232 ms | < 400 µs |

## 7. AES-256-GCM (Token Encryption) — 2 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `aes_gcm_encrypt` | 45.244 µs | < 25 µs |
| `aes_gcm_decrypt` | 43.752 µs | < 25 µs |

## 8. RSA-4096 (Classical Co-Signature) — 2 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `rsa_4096_sign` | 1.535 ms | < 2 ms |
| `rsa_4096_verify` | 279.21 µs | < 200 µs |

## 9. Sponge Core (TLSponge-385) — 2 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `sponge_hash` | 6.023 µs | < 5 µs |
| `sponge_derive_key` | 4.088 µs | < 5 µs |

## 10. TIS-27 Standalone — 3 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `tis27_hash_27trit` | 3.986 µs | < 5 µs |
| `tis27_hash_54trit` | 7.873 µs | < 5 µs |
| `tis27_absorb_squeeze` | 12.125 µs | < 8 µs |

## 11. HMAC — 3 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `hmac_key_derive` | 4.118 µs | < 5 µs |
| `hmac_compute` | 12.598 µs | < 500 ns |
| `hmac_verify` | 20.186 µs | < 500 ns |

## 12. σ Shuffles — 3 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `sigma_shuffle_round` | 118.48 ns | < 200 ns |
| `sigma_tis27_4rounds` | 269.99 ns | < 1 µs |
| `sigma_tlsponge_9rounds` | 523.13 ns | < 2 µs |

## 13. Wire Integrity — 2 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `wire_checksum_compute` | 304.2 ps | < 100 ns |
| `wire_ecc_compute` | 603.1 ps | < 100 ns |

## 14. Lattice Mixer — 2 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `lattice_nonce` | 318.2 ps | < 100 ns |
| `lattice_key_derive` | 7.888 µs | < 5 µs |

## 15. Identity — 2 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `identity_seed_derive` | 15.728 µs | < 5 µs |
| `identity_keypair_derive` | 13.910 ms | < 5 ms |

## 16. Tunnel Auth — 2 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `tunnel_auth_response` | 11.844 µs | < 5 µs |
| `tunnel_handshake_3msg` | 43.307 µs | < 20 ms |

## 17. Heartbeat Pipeline — 2 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `heartbeat_pipeline_single` | 11.855 µs | < 1.2 µs |
| `heartbeat_26_neighbors` | 548.33 µs | < 50 µs |

## 18. TSA (RFC 3161 Time-Stamping) — 2 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `tsa_timestamp_create` | 27.745 µs | < 30 µs |
| `tsa_timestamp_verify` | 24.087 µs | < 20 µs |

## 19. Merkle Tree — 2 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `merkle_insert` | 317.01 µs | < 200 µs |
| `merkle_verify` | 324.41 µs | < 200 µs |

## 20. TDNS Identity — 3 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `tdns_derive_identity` | 4.116 µs | < 10 µs |
| `tdns_scan_hash` | 4.079 µs | < 10 µs |
| `tdns_repunit_checksum` | 316.6 ps | < 100 ns |

## 21. Calendar TERN Compression — 2 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `tern_compress` | 96.490 µs | < 15 µs |
| `tern_decompress` | 104.92 µs | < 20 µs |

## 22. CON Topology Keys — 3 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `con_derive_tunnel_key` | 8.722 µs | < 10 µs |
| `con_rekey_single` | 8.208 µs | < 10 µs |
| `con_rekey_all` | 218.79 µs | < 300 µs |

## 23. HPTP Timing — 3 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `hptp_timestamp_verify` | 11.819 µs | < 20 µs |
| `hptp_drift_compensate` | 26.979 µs | < 10 µs |
| `hptp_jitter_filter` | 378.92 µs | < 50 µs |

## 24. ZK Proofs — 2 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `zk_prove` | 24.337 µs | < 30 µs |
| `zk_verify` | 28.293 µs | < 30 µs |

## 25. SignHere Pipeline — 4 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `signhere_secure_doc` | 108.22 µs | < 100 µs |
| `signhere_6check` | 79.553 µs | < 80 µs |
| `signhere_cnsa2` | 46.527 µs | < 50 µs |
| `signhere_witness` | 21.999 µs | < 20 µs |

## 26. SFK Operations — 3 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `sfk_key_derive` | 4.096 µs | < 10 µs |
| `sfk_sign` | 20.419 µs | < 25 µs |
| `sfk_verify` | 32.635 µs | < 25 µs |

## 27. Hedera / Blockchain — 2 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `hedera_submit_witness` | 24.388 µs | < 25 µs |
| `hedera_verify_witness` | 24.209 µs | < 20 µs |

## 28. Lamport OTS — 3 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `lamport_keygen` | 2.102 ms | < 5 ms |
| `lamport_sign` | 1.027 ms | < 3 ms |
| `lamport_verify` | 4.087 ms | < 3 ms |

## 29. Roundtrips — 13 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `rt_pt26_full` | 40.483 µs | < 80 µs |
| `rt_pt26_sign_verify` | 31.758 µs | < 60 µs |
| `rt_tl_dsa_v1_full` | 61.104 ms | < 60 ms |
| `rt_tl_dsa_v1_sign_verify` | 48.409 ms | < 50 ms |
| `rt_tl_dsa_v2_full` | 362.57 µs | < 500 µs |
| `rt_tl_kem_1024` | 159.05 µs | < 300 µs |
| `rt_tae_mac` | 80.190 µs | < 60 µs |
| `rt_phase_encrypt` | 123.52 µs | < 80 µs |
| `rt_signhere_full` | 128.86 µs | < 200 µs |
| `rt_tsa_full` | 52.060 µs | < 50 µs |
| `rt_merkle_full` | 638.12 µs | < 400 µs |
| `rt_lamport_full` | 7.034 ms | < 10 ms |
| `rt_zk_full` | 51.197 µs | < 60 µs |

## 30. A/B Sponge (Scalar vs 2-Bit Packed) — 4 benchmarks

| Benchmark | Median | Target |
|-----------|--------|--------|
| `ab_derive_key_scalar` | 5.913 µs | ~8 µs |
| `ab_derive_key_2bit` | 433.30 µs | < 1.5 µs |
| `ab_heartbeat26_scalar` | 446.32 µs | ~560 µs |
| `ab_heartbeat26_2bit` | 22.334 ms | < 85 µs |

---

## Summary Scoreboard

### Fastest Operations (sub-nanosecond)

| Operation | Time |
|-----------|------|
| `wire_checksum_compute` | 304 ps |
| `pt26_step_token` | 314 ps |
| `pt26_walk_token` | 313 ps |
| `lattice_nonce` | 318 ps |
| `tdns_repunit_checksum` | 317 ps |
| `wire_ecc_compute` | 603 ps |
| `pt26_trit_diff` | 622 ps |
| `tl_dsa_v2_ntt_butterfly` | 698 ps |

### Signature Scheme Comparison (full roundtrip)

| Scheme | Keygen + Sign + Verify | Speed Class |
|--------|----------------------|-------------|
| PT26-DSA | 40.5 µs | Ultra-fast |
| TL-DSA v2 | 362.6 µs | Fast |
| TL-KEM-1024 | 159.1 µs | Fast |
| Lamport OTS | 7.03 ms | Moderate |
| TL-DSA v1 | 61.1 ms | Heavyweight |

### KEM Security Level Scaling

| Level | Keygen | Encaps | Decaps |
|-------|--------|--------|--------|
| TL-KEM-512 | 38.6 µs | 15.7 µs | 11.7 µs |
| TL-KEM-768 | 65.8 µs | 19.9 µs | 20.0 µs |
| TL-KEM-1024 | 108.8 µs | 27.5 µs | 23.6 µs |

### Sponge Backend A/B (Scalar vs 2-Bit Packed)

| Backend | derive_key | heartbeat×26 | Status |
|---------|-----------|-------------|--------|
| Scalar | 5.91 µs | 446 µs | **WINNER** |
| 2-Bit Packed | 433 µs | 22.3 ms | 73× / 50× slower |

Root cause: per-trit get/set extraction cost dominates ρ/π/σ/χ.
Path forward: AVX2 SIMD (vpshufb processes 32 GF(27) elements simultaneously).

---

**104 benchmarks. 30 categories. Every module covered. Nothing deferred.**
