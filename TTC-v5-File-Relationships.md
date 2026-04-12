# TTC v5.0.3 — File Relationships & Architecture Map

**Date:** 2026-04-11
**Version:** TTC v5.0 (wire version 0x05), implementation v5.0.3
**Spec:** TM-2026-030 v3

---

## Version History

| Version | Wire Byte | Key Change |
|---------|-----------|------------|
| TTC v1.x | 0x02 | Legacy |
| TTC v2.0 | 0x03 | 96-byte header |
| TTC v3.0 | 0x04 | 27-byte header |
| **TTC v5.0** | **0x05** | **CPT (Coprime Periodic Transform), 28-byte header, Modes 4/5** |

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        SPECIFICATION                            │
│  TM-2026-030-TTC-v5-Protocol-Specification.md                   │
│  (§1-§12: transform, ANOVA, modulus source, wire format, probe) │
└────────────────────┬────────────────────────────────────────────┘
                     │
         ┌───────────┼───────────┐
         ▼           ▼           ▼
┌──────────────┐ ┌─────────┐ ┌───────────────┐
│   ttc.rs     │ │ cpd.rs  │ │ ctx_ans.rs    │
│  (2,677 ln)  │ │(134 ln) │ │  (524 ln)     │
│  CORE ENGINE │ │PROBE ALG│ │ CONTEXT rANS  │
│  Modes 0-5   │ │ §8 impl │ │ Z₂₇ context  │
│  Phase 1+2   │ │ AC scan │ │ Order-1 model │
│  Wire format │ │ Groups  │ │ ChunkMode = 4 │
└──────┬───────┘ └────┬────┘ └───────┬───────┘
       │              │              │
       │    ┌─────────┴──────────────┘
       │    │
       ▼    ▼
┌──────────────────┐     ┌────────────────────┐
│ container_decomp │     │   constants.rs     │
│    (849 ln)      │     │   (1,790 ln)       │
│ ZIP/PDF/GZ/PNG   │     │ §7: COPRIME_TRIPLES│
│ crack + reorder  │     │ QUADRUPLES/QUINT/  │
│ coprime walk     │     │ SEXTUPLES + LCMs   │
└──────────────────┘     │ DEFICIT_RATE=1/91  │
                         │ SPECTRAL_PRODUCT   │
                         └────────┬───────────┘
                                  │
                         ┌────────┴───────────┐
                         │   coprime.rs       │
                         │    (516 ln)        │
                         │ euler_totient()    │
                         │ coprime walk gen   │
                         │ is_coprime()       │
                         └────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                     N-API BRIDGE                                │
│  ternary-math/napi/src/lib.rs (588 ln)                         │
│  ttc_compress() / ttc_decompress() → Node.js Buffer            │
│  Lines 414-498: TTC N-API exports                              │
└────────────────────┬────────────────────────────────────────────┘
                     │
         ┌───────────┼───────────┐
         ▼           ▼           ▼
┌──────────────┐ ┌─────────────────┐ ┌──────────────┐
│compression-  │ │  ternary.ts     │ │compression.  │
│layer.ts      │ │   (235 ln)      │ │tsx (982 ln)  │
│  (656 ln)    │ │ TS fallback     │ │ FRONTEND UI  │
│ SERVER API   │ │ RLE + ternary   │ │ Demo page    │
│ Native probe │ │ encode (legacy) │ │ Upload/test  │
│ /api/compress│ │                 │ │ TTC metadata │
└──────────────┘ └─────────────────┘ └──────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                     DATABASE SCHEMA                             │
│  shared/schema.ts                                               │
│  compression_benchmarks — benchmark results table               │
│  compression_history — per-file compression log                 │
│  compressed_documents — stored compressed files                 │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                     BENCHMARKS                                  │
│  ternary-math/benches/ttc_benchmark.rs (406 ln)                 │
│  TTC-v2.0-benchmark-report.md                                   │
│  TTC-v4.1-benchmark-report.md                                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## File Inventory

### Tier 1: Core Rust Engine (ternary-math/src/)

| File | Lines | Role | TTC Version |
|------|-------|------|-------------|
| `ttc.rs` | 2,677 | **Primary engine.** Chunk modes 0-5, phase1_analyze + phase2_compress pipeline, wire format encode/decode, CRC32 verification, CPT integration (Modes 4/5), GURFT fast-path, domain analysis. | v5.0.3 |
| `cpd.rs` | 134 | **Coprime Periodic Detection.** Implements §8 probe algorithm — autocorrelation at coprime strides, canonical group scoring (18 predefined groups), non-canonical small-period scan, competitive selection. | v5.0.2 |
| `ctx_ans.rs` | 524 | **Context-1 rANS.** Order-1 context model with Z₂₇ (27) context bins. Context = previous_byte mod 27. Per-context frequency distribution. ChunkMode = 4. | v5.0.3 |
| `container_decomp.rs` | 849 | **Container decomposition.** Cracks open ZIP/PDF/GZ/PNG, inflates internal streams, reorders content by type using coprime walk ordering for maximum cross-entry LZ77 matching, reconstructs original container. | v5.0.2 |

### Tier 2: Mathematical Foundation (ternary-math/src/)

| File | Lines | Role |
|------|-------|------|
| `constants.rs` | 1,790 | **Coprime group tables.** §7: COPRIME_TRIPLES[7], COPRIME_QUADRUPLES[2], COPRIME_QUINTUPLES[5], COPRIME_SEXTUPLES[4] + all LCMs. DEFICIT_RATE = 1/91. NULL_HARMONIC_DEFICIT = 4,004. GEOMETRIC_SPECTRAL_PRODUCT = 364,364. |
| `coprime.rs` | 516 | **Coprime walk generator.** euler_totient(), is_coprime(), trial_factor_decomposition(). Source of the moduli used by CPT probe and transform. |
| `lib.rs` | ~400 | **Module declarations.** `pub mod ttc; pub mod cpd; pub mod container_decomp; pub mod ctx_ans;` |

### Tier 3: N-API Bridge

| File | Lines | Role |
|------|-------|------|
| `ternary-math/napi/src/lib.rs` | 588 | Lines 414-498: `ttc_compress()` and `ttc_decompress()` exported as N-API functions. Buffer in → struct out (compressed data, ratio, CRC, metadata). |
| `ternary-math/napi/Cargo.toml` | — | N-API crate dependencies |

### Tier 4: Server Integration

| File | Lines | Role |
|------|-------|------|
| `server/compression-layer.ts` | 656 | **Server-side API.** Probes for native N-API addon, falls back to TS. Exposes `/api/compress`, `/api/decompress` routes. TTC metadata extraction for frontend. |
| `server/ternary.ts` | 235 | **TypeScript fallback.** Legacy RLE + ternary encoding. Used when N-API addon not available. |
| `server/routes.ts` | — | Route registration (compression endpoints) |

### Tier 5: Frontend

| File | Lines | Role |
|------|-------|------|
| `client/src/pages/compression.tsx` | 982 | **Compression demo page.** File upload, compress/decompress, displays TTC metadata badges, round-trip verification, CRC32 display, compression ratio chart. |

### Tier 6: Database Schema

| File | Lines | Role |
|------|-------|------|
| `shared/schema.ts` | — | `compression_benchmarks`, `compression_history`, `compressed_documents` tables. |

### Tier 7: Specification & Benchmarks

| File | Lines | Role |
|------|-------|------|
| `docs/technical-memos/TM-2026-030-TTC-v5-Protocol-Specification.md` | 274 | **Canonical spec.** §1-§12: transform math, ANOVA rationale, modulus source, overhead bound, AC thresholds, pipeline (6 modes), wire format, probe algorithm, version compat, diagnostics, worked example, patent claims. |
| `ternary-math/benches/ttc_benchmark.rs` | 406 | Rust benchmark suite — industry-standard test patterns, speed/throughput metrics. |
| `TTC-v2.0-benchmark-report.md` | — | Historical v2.0 benchmark results. |
| `TTC-v4.1-benchmark-report.md` | — | Historical v4.1 benchmark results. |
| `ternary-math/Cargo.toml` | — | Crate manifest — `flate2`, CRC32 dependencies. |

---

## Dependency Chain

```
constants.rs (coprime group tables, DEFICIT_RATE)
     │
     ├──→ coprime.rs (euler_totient, is_coprime, trial_factor)
     │         │
     │         ▼
     │    cpd.rs (probe algorithm, autocorrelation, canonical group scoring)
     │         │
     │         ▼
     ├──→ ttc.rs (core engine — calls cpd::probe, uses constants, calls ctx_ans)
     │         │                    │
     │         │                    ├──→ ctx_ans.rs (context-1 rANS, Z₂₇ model)
     │         │                    │
     │         │                    └──→ container_decomp.rs (ZIP/PDF crack, reorder)
     │         │
     │         ▼
     │    napi/src/lib.rs (N-API bridge: ttc_compress, ttc_decompress)
     │         │
     │         ▼
     │    server/compression-layer.ts (API: probes native, exposes /api/compress)
     │         │
     │         ├──→ server/ternary.ts (TS fallback if native unavailable)
     │         │
     │         ▼
     │    client/src/pages/compression.tsx (frontend demo)
     │         │
     │         ▼
     │    shared/schema.ts (database tables for benchmarks, history, documents)
     │
     └──→ TM-2026-030 (specification — everything above implements this)
```

---

## Chunk Mode Summary (from ttc.rs + TM-2026-030)

| Mode | Name | Pipeline | Wire Version |
|------|------|----------|--------------|
| 0 | Stored | Unchanged | Any |
| 1 | Compressed | LZ77 + prefix codes | 0x03+ |
| 2 | TernaryEnhanced | LZ77 + trit encoding | 0x03+ |
| 3 | TernaryAns | LZ77 + ternary rANS | 0x04+ |
| **4** | **CptAns** | **CPT + varint residual** | **0x05** |
| **5** | **CptLz77Ans** | **CPT + LZ77 on residual + rANS** | **0x05** |

---

## Total Line Count

| Tier | Lines |
|------|-------|
| Core Rust engine | 4,184 |
| Mathematical foundation | 2,706 |
| N-API bridge | 588 |
| Server integration | 891 |
| Frontend | 982 |
| Benchmarks | 406 |
| Specification | 274 |
| **Total** | **~10,031** |
