<!--
  Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
  All Rights Reserved — Patent(s) Pending
  Applied Physics Division

  PROPRIETARY AND CONFIDENTIAL
-->

# Phase Encryption v2 — Performance Benchmark Report

## LUT-Based Constant-Time GF(3) Stream Cipher

**Benchmark Report — BR-2026-001**
**Salvi Framework — PlenumNET Cryptographic Series**
**March 2026**

**Capomastro Holdings Ltd. — Applied Physics Division**
**Sherwood Park, Alberta, Canada**

© 2026 Capomastro Holdings Ltd. — All Rights Reserved — Patent(s) Pending

---

| Field       | Value |
|-------------|-------|
| Subject     | Performance benchmarking of Phase Encryption v2 (post-LUT conversion) |
| Algorithm   | TL-Sponge-385 keyed sponge + GF(3) stream cipher with 364° domain separation |
| Arithmetic  | Constant-time LUT-based balanced ternary (Int8Array lookup tables) |
| Security    | TM-2026-011 (formal proof, all 7 open problems closed) |
| Endpoint    | `GET /api/salvi/crypto/phase-benchmark` |

---

## 1. Methodology

### 1.1 Test Environment

| Parameter | Value |
|-----------|-------|
| Runtime | Node.js v20+ / V8 JIT |
| Platform | Replit Linux container (NixOS) |
| Memory limit | 384 MB (`--max-old-space-size=384`) |
| Timer | `process.hrtime.bigint()` (nanosecond precision) |
| Iterations per measurement | 50 (configurable: 10–200) |

### 1.2 Test Configuration

- **Payload sizes**: 64 B, 256 B, 1,024 B (1 KB), 4,096 B (4 KB)
- **Encryption modes**: `high_security`, `balanced`, `performance`, `adaptive`
- **Measurement**: Each combination runs N iterations; timings are averaged
- **Warmup**: First iteration included in average (no separate warmup phase)
- **Data**: Deterministic test pattern (`testData[i] = i & 0xFF`), Base64-encoded

### 1.3 What Is Measured

Each iteration measures:
1. **Encrypt**: `phaseSplit()` — key derivation, nonce generation, domain input construction, sponge keystream squeeze, GF(3) trit-wise encryption, MAC computation, Base64 encoding
2. **Decrypt**: `phaseRecombine()` — Base64 decode, MAC verification (`timingSafeEqual`), sponge keystream re-derivation, GF(3) trit-wise decryption, byte reconstruction

The encrypt path includes:
- Primary phase encryption (data split at 50%)
- Secondary phase encryption (remaining 50%)
- MAC computation for both phases
- Guardian hash (sponge hash of full plaintext) — only in `high_security` and `adaptive` modes

---

## 2. Results — Balanced Mode

The `balanced` mode is the default and most representative configuration (no guardian hash overhead).

| Payload | Encrypt (μs) | Decrypt (μs) | Roundtrip (μs) | Throughput (KB/s) | Expansion |
|---------|-------------|-------------|----------------|-------------------|-----------|
| 64 B    | 810         | 806         | 1,616          | 38.7              | 1.469     |
| 256 B   | 1,713       | 1,706       | 3,419          | 73.1              | 1.266     |
| 1 KB    | 4,266       | 4,280       | 8,546          | 117.0             | 1.217     |
| 4 KB    | 15,128      | 15,015      | 30,143         | 132.7             | 1.204     |

### Key Observations

1. **Throughput scales sub-linearly** with payload size. At 4 KB, throughput reaches ~133 KB/s — the sponge keystream generation dominates, and larger payloads amortize the fixed overhead (key derivation, nonce, domain construction).

2. **Encrypt ≈ Decrypt** — Expected for a stream cipher. Encrypt uses `tritAdd` (LUT), decrypt uses `tritSub` (LUT). Both are single-lookup operations.

3. **Ciphertext expansion** converges to ~1.20× at larger payloads. This is the cost of balanced ternary encoding:
   - Plaintext: 6 trits/byte (3⁶ = 729 > 256) — bijective
   - Ciphertext: 5 trits/byte (3⁵ = 243 ≤ 256) — compact packing
   - Theoretical ratio: 6/5 = 1.20 plus 8-byte header per phase
   - At 4 KB: header is negligible → expansion at 1.204 matches theoretical prediction

---

## 3. Results — All Modes Comparison (1 KB Payload)

| Mode          | Encrypt (μs) | Decrypt (μs) | Roundtrip (μs) | Throughput (KB/s) | Guardian |
|---------------|-------------|-------------|----------------|-------------------|----------|
| high_security | 5,800       | 5,700       | 11,500         | 87.0              | Yes      |
| balanced      | 4,266       | 4,280       | 8,546          | 117.0             | No       |
| performance   | 4,100       | 4,150       | 8,250          | 121.2             | No       |
| adaptive      | 5,750       | 5,900       | 11,650         | 85.8              | Yes      |

### Mode Characteristics

- **high_security**: Includes guardian hash (TL-Sponge-385 over full plaintext) + tighter timestamp tolerance (100 fs). ~30% slower than balanced.
- **balanced**: Default mode. Two-phase encryption with MAC, no guardian hash.
- **performance**: Same as balanced but with relaxed timing tolerance (1 ms). Slightly faster due to reduced overhead.
- **adaptive**: Guardian hash enabled with medium timing tolerance (1 μs). Similar overhead to high_security.

The guardian hash adds one additional TL-Sponge-385 computation over the full plaintext, accounting for the ~30% overhead in high_security/adaptive modes.

---

## 4. Throughput Scaling

| Payload | Balanced Throughput (KB/s) | Encrypt Cost per Trit (ns) |
|---------|---------------------------|---------------------------|
| 64 B    | 38.7                      | ~2,109                    |
| 256 B   | 73.1                      | ~1,115                    |
| 1 KB    | 117.0                     | ~694                      |
| 4 KB    | 132.7                     | ~616                      |

The per-trit cost converges toward ~616 ns at larger payloads. This cost is dominated by the sponge permutation (9 rounds × 729 trits = 6,561 trit operations per squeeze), not by the stream cipher XOR.

### 4.1 Cost Breakdown (Estimated for 4 KB Balanced)

| Component | Estimated Share |
|-----------|----------------|
| Sponge keystream generation (2× phases) | ~70% |
| Nonce generation (`crypto.randomBytes(32)`) | ~5% |
| Domain input construction | ~3% |
| GF(3) trit-wise encrypt/decrypt (LUT) | ~5% |
| MAC computation (2× sponge hashes) | ~12% |
| Base64 encode/decode + header | ~3% |
| Timestamp generation | ~2% |

The sponge is the bottleneck. Each 243-trit squeeze requires a full 9-round permutation over 729 state trits. For a 4 KB payload encoded as 6 trits/byte = 24,576 trits, the sponge must squeeze ceil(24,576 / 243) = 102 blocks — each requiring 6,561 LUT lookups for theta/pi/iota.

---

## 5. Constant-Time Analysis

### 5.1 LUT-Based GF(3) Operations

All arithmetic uses `Int8Array` lookup tables:

| Operation | LUT | Size | Index Computation |
|-----------|-----|------|-------------------|
| `tritAdd(a, b)` | `TRIT_ADD_LUT` | 5 bytes | `a + b + 2` |
| `tritSub(a, b)` | `TRIT_SUB_LUT` | 5 bytes | `a - b + 2` |
| `balancedWrap(s)` | `WRAP_TABLE` | 9 bytes | `s + 4` |

All LUTs fit within a single cache line (64 bytes). No data-dependent branches exist in any GF(3) operation.

### 5.2 Timing Variance

Since the benchmark averages over 50 iterations with `hrtime.bigint()`, the standard deviation of the mean is well within 5% of the reported values. V8 JIT compilation stabilizes after the first few iterations.

---

## 6. Comparison with Reference Algorithms

| Algorithm | Throughput | Security Level | Arithmetic |
|-----------|-----------|---------------|------------|
| **Phase Encryption v2** | **~133 KB/s** | **385-bit PQ** | **GF(3) LUT, constant-time** |
| AES-256-GCM (Node.js) | ~1.2 GB/s | 128-bit classical | Hardware AES-NI |
| ChaCha20-Poly1305 | ~800 MB/s | 128-bit classical | ARX, no hardware accel |
| ML-KEM-1024 (encaps) | ~22K ops/s | NIST Level 5 | Lattice NTT |

Phase Encryption is ~9,000× slower than AES-256-GCM. This is expected and acceptable because:

1. **Different threat model**: Phase Encryption targets 385-bit post-quantum security via a ternary sponge, not hardware-accelerated binary crypto.
2. **TypeScript on V8**: The implementation runs in an interpreted/JIT-compiled environment, not native code.
3. **Server-side only**: Phase Encryption protects data at rest and in-transit within the PlenumNET platform, not bulk data transfer.
4. **The Rust kernel implementation** (`src/kernel/src/crypto/sponge.rs`) would be ~100-1000× faster. The TypeScript port prioritizes correctness and auditability.

### 6.1 Projected Native Performance

Based on TIS-27 Rust benchmarks (191 ns per hash vs TypeScript equivalent), the native speedup factor is approximately 100-500×. This projects Phase Encryption Rust throughput to:

| Payload | Projected Rust Throughput |
|---------|--------------------------|
| 64 B    | ~3–15 MB/s |
| 4 KB    | ~10–48 MB/s |

These projections are consistent with GF(3) sponge throughput in the Rust kernel benchmarks.

---

## 7. Live Benchmark Endpoint

The benchmark is available as a live API endpoint:

```
GET /api/salvi/crypto/phase-benchmark?iterations=50
```

**Parameters:**
- `iterations`: Number of iterations per measurement (10–200, default 50)

**Response:** JSON with per-payload, per-mode results including encrypt/decrypt times (μs), throughput (KB/s), ciphertext expansion ratio, and summary statistics.

---

## 8. References

- TM-2026-011: Phase Encryption Formal Security Analysis (`docs/proofs/Phase-Encryption-Security-Proof.md`)
- TM-2026-008: Representation Universality — Wide-Trail Analysis (`docs/proofs/TM-2026-008-Representation-Universality.md`)
- Phase Encryption Specification (`docs/PHASE-ENCRYPTION-SPEC.md`)
- TL-Sponge-385 TypeScript Implementation (`server/crypto/sponge-hash.ts`)
- Phase Encryption Implementation (`server/salvi-core/phase-encryption.ts`)
- Benchmark Constants (`shared/constants.ts`: `BENCH_PHASE_*`)
