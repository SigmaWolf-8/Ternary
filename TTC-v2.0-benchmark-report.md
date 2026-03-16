# TTC v2.0 & Crypto Primitives — Benchmark Report

**Date:** 2026-03-16  
**Platform:** Replit Linux (x86_64), Rust 1.x release build, Node.js v20.20.0  
**Module:** TM-2026-017 Tribonacci Ternary Compression v2.0  
**Author:** RSalvi@Salvigroup.com  

---

## 1. Bug Fixes Summary (Before → After)

### Issues Resolved

| # | Bug | Root Cause | Fix | Tests Affected |
|---|-----|-----------|-----|----------------|
| 1 | `test_delta_all_flags_round_trip` FAIL | TernaryEnhanced deserializer skipped 2-bit rep selector when `adaptive_rep=false`, but serializer always writes them | Always read rep bits unconditionally in `deserialize_ternary_enhanced` | 1 test |
| 2 | `test_stored_mode_constant_data` FAIL | Decompressor always called `extract_filename()` — non-zero leading bytes were interpreted as filename length, consuming data | Added `has_filename` flag (bit 0x20) in container header; extraction gated on flag | 1 test |
| 3 | `test_full_round_trip_via_dispatch` FAIL | Same filename extraction bug as #2 | Same fix | 1 test |
| 4 | 16 KB+ data decompresses to `[]` | tANS codec quantized LZ77 match distances to 256 buckets — lossy encoding meant decoder recovered wrong distances | tANS now only used for Literal/Run-only token streams; data with Match tokens uses Compressed or TernaryEnhanced codecs | All multi-KB sizes |

### Test Count: Before → After

| Suite | Before | After |
|-------|--------|-------|
| TTC module tests | 34/37 | 38/38 (incl. benchmark) |
| Rust total | 300 | 304 |
| Vitest (TS) | 577/578 | 577/578 (1 pre-existing timeout) |
| **Platform total** | ~2,616 | **2,620** |

---

## 2. TTC v2.0 Compression Level Parameters

All levels use `CompressionMode::Basic` with `independent_chunks: true`.

| Level | Tier | Window | Chunk | Min Match | Min Run | Chain Depth | Parsing | Candidates | GURFT | Target Payload |
|-------|------|--------|-------|-----------|---------|-------------|---------|------------|-------|----------------|
| L1 | TTC1-1 | 3 KB | 26 KB | 8 | 6 | 8 | Greedy | 2 | Skip | 1–16 KB |
| L2 | TTC1-2 | 9 KB | 26 KB | 6 | 5 | 16 | Lazy | 3 | Skip | 4–64 KB |
| L3 | TTC1-3 | 27 KB | 13 KB | 4 | 4 | 32 | Lazy | 4 | On | 16–256 KB |
| L4 | TTC2-1 | 81 KB | 13 KB | 4 | 4 | 32 | Lazy | 4 | On | 64 KB–1 MB |
| L5 | TTC2-2 | 243 KB | 13 KB | 4 | 4 | 64 | Lazy | 4 | On | 256 KB–4 MB |
| L6 | TTC2-3 | 729 KB | 13 KB | 4 | 4 | 128 | Lazy | 4 | On | 1–16 MB |
| L7 | TTC3-1 | 2.1 MB | 13 KB | 3 | 3 | 128 | BeamOptimal | 4 | On | 4–32 MB |
| L8 | TTC3-2 | 6.4 MB | 13 KB | 3 | 3 | 192 | BeamOptimal | 4 | On | 16–64 MB |
| L9 | TTC3-3 | 19.2 MB | 13 KB | 3 | 3 | 256 | BeamOptimal | 4 | On | 50 MB+ |

**Parsing modes:**
- **Greedy** — first match wins, fastest
- **Lazy** — check if next position has a better match before committing
- **BeamOptimal** — exhaustive multi-candidate search, best ratio, highest cost

**Codec candidates per chunk (best wins):**
- **Stored** (mode 0) — raw data, no compression
- **Compressed** (mode 1) — LZ77 + Rice coding (lossless distances)
- **TernaryEnhanced** (mode 2) — ternary rep encoding + Rice (≤16 KB chunks only)
- **tANS** (mode 3) — asymmetric numeral systems entropy coding (Literal/Run data only)

---

## 3. TTC v2.0 Compression Benchmark

**Input:** Repeating 420-byte PlenumNET technical prose.  
**Config:** `CompressionMode::Basic`, `independent_chunks: true`, release build.

| Size | Level | Compress | Decompress | Total | Ratio | Savings | Codec | Chunks |
|------|-------|----------|------------|-------|-------|---------|-------|--------|
| 1 KB | L1 | 473 µs | 9 µs | 482 µs | 1.84x | 45.5% | Comp | 1 |
| 1 KB | L2 | 65 µs | 9 µs | 75 µs | 1.85x | 45.8% | Comp | 1 |
| 4 KB | L1 | 82 µs | 19 µs | 100 µs | 6.80x | 85.3% | Comp | 1 |
| 4 KB | L2 | 98 µs | 19 µs | 117 µs | 6.84x | 85.4% | Comp | 1 |
| 16 KB | L1 | 300 µs | 67 µs | 366 µs | 21.11x | 95.3% | Comp | 1 |
| 16 KB | L2 | 467 µs | 63 µs | 530 µs | 21.20x | 95.3% | Comp | 1 |
| 16 KB | L3 | 2.1 ms | 72 µs | 2.2 ms | 13.76x | 92.7% | Comp | 2 |
| 64 KB | L2 | 1.5 ms | 250 µs | 1.8 ms | 27.85x | 96.4% | Comp | 3 |
| 64 KB | L3 | 5.7 ms | 280 µs | 6.0 ms | 20.60x | 95.1% | Comp | 5 |
| 64 KB | L4 | 5.5 ms | 252 µs | 5.7 ms | 20.60x | 95.1% | Comp | 5 |
| 256 KB | L3 | 15.6 ms | 1.1 ms | 16.6 ms | 21.09x | 95.3% | Comp | 20 |
| 256 KB | L4 | 9.8 ms | 1.2 ms | 11.0 ms | 21.09x | 95.3% | Comp | 20 |
| 256 KB | L5 | 20.0 ms | 1.0 ms | 21.1 ms | 21.09x | 95.3% | Comp | 20 |
| 1 MB | L4 | 50.2 ms | 5.0 ms | 55.3 ms | 21.40x | 95.3% | Comp | 79 |
| 1 MB | L5 | 45.9 ms | 3.9 ms | 49.8 ms | 21.40x | 95.3% | Comp | 79 |
| 1 MB | L6 | 60.2 ms | 3.9 ms | 64.1 ms | 21.40x | 95.3% | Comp | 79 |
| 4 MB | L5 | 159.7 ms | 19.2 ms | 178.9 ms | 21.43x | 95.3% | Comp | 316 |
| 4 MB | L6 | 206.8 ms | 17.3 ms | 224.1 ms | 21.43x | 95.3% | Comp | 316 |

**Key observations:**
- Decompression is consistently fast (9 µs at 1 KB → 19 ms at 4 MB), independent of compression level.
- L1 at 1 KB shows a cold-start penalty (473 µs) — L2 at 65 µs is the steady-state for small payloads.
- Ratios plateau at ~21x for sizes ≥ 16 KB on this input (highly repetitive text).
- L3 at 16 KB (13.76x) underperforms L1 (21.11x) because the 13 KB chunk size splits into 2 chunks, adding per-chunk overhead.
- L7–L9 (BeamOptimal) not benchmarked here — they require 16 MB+ payloads and single-threaded wall time measured in seconds to minutes.

---

## 4. T-AE-MAC Benchmark (Rust, Release Build)

1 KB block, TL-Sponge-385-based authenticated encryption. Industry baseline: Ascon-128 (NIST LWC winner).

| Primitive | Measured | Target | Status |
|-----------|----------|--------|--------|
| tae_mac_encrypt | 123.5 µs | < 40 µs | **FAIL** (3.1x over) |
| tae_mac_decrypt | 165.9 µs | < 40 µs | **FAIL** (4.1x over) |
| tae_mac_compute | 48.3 µs | < 30 µs | **FAIL** (1.6x over) |
| tae_mac_verify | 97.0 µs | < 50 µs | **FAIL** (1.9x over) |
| **T-AE-MAC Total** | **434.6 µs** | — | **0/4 passing** |

**Before (prior session):** encrypt 100–111 µs, decrypt ~110 µs, compute ~45 µs, verify ~90 µs.  
**After (current):** encrypt 123 µs, decrypt 166 µs, compute 48 µs, verify 97 µs.  
**Analysis:** No improvement — slight regression likely due to heap-path `Vec` allocation for tag_input (1072 bytes exceeds stack buffer). The sponge permutation cost (~4.3 µs × multiple rounds) dominates. Meeting the Ascon-128 targets requires either the WASM bridge or a native N-API path for T-AE-MAC specifically.

---

## 5. Phase Encryption v3 Benchmark (TypeScript, Native Sponge Backend)

Duplex TL-Sponge-385 + GF(3) stream cipher. Backend: Rust N-API native addon.

| Payload | Mode | Encrypt | Decrypt | Roundtrip | Throughput | Expansion |
|---------|------|---------|---------|-----------|------------|-----------|
| 64 B | high_security | 215 µs | 87 µs | 303 µs | 207 KB/s | 1.47x |
| 64 B | balanced | 110 µs | 45 µs | 155 µs | 404 KB/s | 1.47x |
| 64 B | performance | 136 µs | 62 µs | 198 µs | 315 KB/s | 1.47x |
| 256 B | high_security | 282 µs | 174 µs | 456 µs | 548 KB/s | 1.27x |
| 256 B | balanced | 201 µs | 112 µs | 313 µs | 799 KB/s | 1.27x |
| 256 B | performance | 256 µs | 158 µs | 414 µs | 604 KB/s | 1.27x |
| 1 KB | high_security | 775 µs | 648 µs | 1.42 ms | 703 KB/s | 1.22x |
| 1 KB | balanced | 515 µs | 437 µs | 952 µs | 1,051 KB/s | 1.22x |
| 1 KB | performance | 576 µs | 465 µs | 1.04 ms | 961 KB/s | 1.22x |
| 4 KB | high_security | 2,038 µs | 1,839 µs | 3.88 ms | 1,032 KB/s | 1.20x |
| 4 KB | balanced | 1,426 µs | 1,288 µs | 2.71 ms | 1,474 KB/s | 1.20x |
| 4 KB | performance | 1,379 µs | 1,281 µs | 2.66 ms | **1,504 KB/s** | 1.20x |

**Summary:**
- **Avg throughput:** 766 KB/s | **Peak:** 1,504 KB/s (4 KB, performance mode)
- **Trit expansion:** 1.20x–1.47x (smaller payloads have higher relative overhead)
- Balanced and performance modes are consistently ~1.5x faster than high_security

---

## 6. Full Crypto Primitive Suite (Rust, Inter-Cube Benchmark)

92 benchmarks across 30 categories. Selected highlights:

| Category | Primitives | Pass Rate | Notes |
|----------|-----------|-----------|-------|
| TL-DSA v1-87 | 3 | 0/3 | WOTS+ — 13–27 ms per op, needs hardware accel |
| PT26-DSA | 7 | 7/7 | All Plenum+ (7–22 µs range) |
| TL-DSA v2 (NTT) | 6 | 5/6 | matrix_mul 39 µs slightly over 30 µs target |
| TL-KEM (512/768/1024) | 9 | 9/9 | All pass, 11–106 µs range |
| T-AE-MAC | 4 | 0/4 | 48–166 µs, targets 30–50 µs |
| Phase Encryption | 4 | 3/4 | batch_split 2.3 ms slightly over 2 ms target |
| AES-GCM (baseline) | 2 | 2/2 | 1 µs — industry reference |
| TL-Sponge-385 | 4 | 4/4 | Permutation 4.3 µs, hash 1 KB 52 µs |
| **Overall** | **92** | **66/92 (72%)** | 79/92 PQ-verifiable |

---

## 7. Container Format Reference

**Header:** 96 bytes fixed.  
**Chunk map entry:** 16 bytes each.  
**Flags byte (offset 0x28):**

| Bit | Mask | Meaning |
|-----|------|---------|
| 0 | 0x01 | Non-base-3 data |
| 1 | 0x02 | Mixed bases |
| 2 | 0x04 | Independent chunks |
| 3 | 0x08 | Adaptive rep |
| 4 | 0x10 | Fibonacci computed |
| 5 | 0x20 | Has filename (new) |

**tANS alphabet:** 1,024 symbols (TANS_L = 3¹¹ = 177,147).  
**tANS symbol ranges:** 0–255 literals, 256–510 run lengths, 511–766 match lengths, 767–1022 distance buckets, 1023 EOB.  
**tANS restriction:** Only used when token stream contains zero Match tokens (distance encoding is lossy).
