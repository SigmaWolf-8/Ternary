// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.

//! # Sponge Backend Dispatch
//!
//! Runtime selection of the optimal sponge implementation.
//! The scalar (1-byte-per-trit) path is retained as a fallback
//! and correctness oracle. The packed GF(27)-native path is the
//! new default.
//!
//! ## Backend Priority
//!
//! 1. AVX2 packed (x86_64 with AVX2) — ~530ns target [Phase C]
//! 2. NEON packed (ARM64) — ~700ns target [Phase D]
//! 3. 2-bit packed (any platform) — ~2µs target [Phase A-2] ← CURRENT
//! 4. GF(27)-native packed — ~67µs measured (16× slower than scalar)
//! 5. Scalar (fallback) — ~4.3µs measured
//!
//! ## Usage
//!
//! All existing callers of `sponge::derive_key` and `sponge::hash_hex`
//! are redirected through this module. No call-site changes required.

use std::sync::atomic::{AtomicU8, Ordering};

// ═══════════════════════════════════════════════════════════════════════
// BACKEND ENUM
// ═══════════════════════════════════════════════════════════════════════

/// Available sponge computation backends, ordered by performance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SpongeBackend {
    /// Original scalar implementation (1 byte per trit, mod-3 arithmetic).
    /// ~4.3µs per derive_key. Retained as correctness oracle.
    Scalar = 0,
    /// GF(27)-native packed (1 byte per GF(27) element, derived arithmetic).
    /// ~67µs measured (16× slower than scalar). Phase A of TM-2026-013.
    Packed = 1,
    /// Packed + AVX2 SIMD (x86_64). ~530ns target. Phase C. [NOT YET IMPLEMENTED]
    Avx2 = 2,
    /// Packed + NEON SIMD (ARM64). ~700ns target. Phase D. [NOT YET IMPLEMENTED]
    Neon = 3,
    /// 2-bit packed trits (23 × u64 words, bitwise GF(3)).
    /// ~2µs target per derive_key. Phase A-2 of TM-2026-013.
    TwoBit = 4,
}

impl SpongeBackend {
    /// Human-readable backend name for diagnostics and logging.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Scalar => "scalar",
            Self::Packed => "packed-gf27",
            Self::Avx2 => "avx2-packed",
            Self::Neon => "neon-packed",
            Self::TwoBit => "2bit-packed",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// GLOBAL BACKEND SELECTION
// ═══════════════════════════════════════════════════════════════════════

/// Active backend (atomic for thread safety, set once at init).
static ACTIVE_BACKEND: AtomicU8 = AtomicU8::new(4); // Default: TwoBit

/// Detect the best available backend for this platform.
pub fn detect_best() -> SpongeBackend {
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") {
            // AVX2 available but SIMD implementation not yet built.
            // When sponge_simd_avx2.rs lands, this returns Avx2.
            return SpongeBackend::TwoBit;
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        // NEON is mandatory on AArch64 but SIMD impl not yet built.
        return SpongeBackend::TwoBit;
    }
    SpongeBackend::TwoBit
}

/// Initialize the sponge backend. Call once at startup.
pub fn init() {
    let best = detect_best();
    ACTIVE_BACKEND.store(best as u8, Ordering::Relaxed);
}

/// Get the currently active backend.
pub fn active() -> SpongeBackend {
    match ACTIVE_BACKEND.load(Ordering::Relaxed) {
        0 => SpongeBackend::Scalar,
        1 => SpongeBackend::Packed,
        2 => SpongeBackend::Avx2,
        3 => SpongeBackend::Neon,
        4 => SpongeBackend::TwoBit,
        _ => SpongeBackend::Scalar,
    }
}

/// Force a specific backend (for testing/benchmarking).
pub fn set_backend(backend: SpongeBackend) {
    ACTIVE_BACKEND.store(backend as u8, Ordering::Relaxed);
}

// ═══════════════════════════════════════════════════════════════════════
// DISPATCHED API — Drop-in replacement for sponge::derive_key
// ═══════════════════════════════════════════════════════════════════════

/// Derive a key using the active sponge backend.
///
/// This is the function that replaces all calls to `sponge::derive_key`.
/// Same signature, same semantics, different backend.
pub fn derive_key(domain: &[u8], material: &[u8], output_len: usize) -> Vec<u8> {
    match active() {
        SpongeBackend::TwoBit => {
            crate::sponge_2bit::derive_key_2bit(domain, material, output_len)
        }
        SpongeBackend::Packed | SpongeBackend::Avx2 | SpongeBackend::Neon => {
            crate::sponge_packed::derive_key_packed(domain, material, output_len)
        }
        SpongeBackend::Scalar => {
            crate::sponge::derive_key(domain, material, output_len)
        }
    }
}

/// Hash with hex output using the active backend.
pub fn hash_hex(input: &[u8]) -> String {
    match active() {
        SpongeBackend::TwoBit => {
            crate::sponge_2bit::hash_hex_2bit(input)
        }
        SpongeBackend::Packed | SpongeBackend::Avx2 | SpongeBackend::Neon => {
            crate::sponge_packed::hash_hex_packed(input)
        }
        SpongeBackend::Scalar => {
            crate::sponge::hash_hex(input)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// A/B COMPARISON — Run both backends and verify equivalence
// ═══════════════════════════════════════════════════════════════════════

/// Result of an A/B comparison between backends.
#[derive(Debug)]
pub struct AbComparison {
    /// Scalar backend elapsed time in nanoseconds.
    pub scalar_ns: u64,
    /// Packed backend elapsed time in nanoseconds.
    pub packed_ns: u64,
    /// Whether both backends produced identical output bytes.
    pub outputs_match: bool,
    /// Scalar time / packed time (higher = packed is faster).
    pub speedup: f64,
}

/// Run derive_key on both backends and compare output + timing.
pub fn ab_compare(domain: &[u8], material: &[u8], output_len: usize) -> AbComparison {
    let t0 = std::time::Instant::now();
    let scalar_out = crate::sponge::derive_key(domain, material, output_len);
    let scalar_ns = t0.elapsed().as_nanos() as u64;

    let t1 = std::time::Instant::now();
    let packed_out = crate::sponge_packed::derive_key_packed(domain, material, output_len);
    let packed_ns = t1.elapsed().as_nanos() as u64;

    let outputs_match = scalar_out == packed_out;
    let speedup = if packed_ns > 0 {
        scalar_ns as f64 / packed_ns as f64
    } else {
        f64::INFINITY
    };

    AbComparison { scalar_ns, packed_ns, outputs_match, speedup }
}

/// Run N iterations of A/B comparison and return aggregate stats.
pub fn ab_benchmark(
    domain: &[u8],
    material: &[u8],
    output_len: usize,
    iterations: usize,
) -> AbBenchmarkResult {
    let mut scalar_total: u64 = 0;
    let mut packed_total: u64 = 0;
    let mut all_match = true;

    for _ in 0..iterations {
        let result = ab_compare(domain, material, output_len);
        scalar_total += result.scalar_ns;
        packed_total += result.packed_ns;
        if !result.outputs_match { all_match = false; }
    }

    AbBenchmarkResult {
        iterations,
        scalar_mean_ns: scalar_total / iterations as u64,
        packed_mean_ns: packed_total / iterations as u64,
        all_outputs_match: all_match,
        mean_speedup: scalar_total as f64 / packed_total as f64,
    }
}

/// Aggregate A/B benchmark result.
#[derive(Debug)]
pub struct AbBenchmarkResult {
    /// Number of iterations run.
    pub iterations: usize,
    /// Mean scalar backend time in nanoseconds.
    pub scalar_mean_ns: u64,
    /// Mean packed backend time in nanoseconds.
    pub packed_mean_ns: u64,
    /// Whether all iterations produced matching output.
    pub all_outputs_match: bool,
    /// Mean speedup ratio (scalar / packed).
    pub mean_speedup: f64,
}

impl std::fmt::Display for AbBenchmarkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "A/B Benchmark ({} iters): scalar={:.1}µs packed={:.1}µs speedup={:.2}× match={}",
            self.iterations,
            self.scalar_mean_ns as f64 / 1000.0,
            self.packed_mean_ns as f64 / 1000.0,
            self.mean_speedup,
            self.all_outputs_match,
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_backend_is_twobit() {
        ACTIVE_BACKEND.store(4, Ordering::Relaxed);
        assert_eq!(active(), SpongeBackend::TwoBit);
    }

    #[test]
    fn set_and_get_backend() {
        set_backend(SpongeBackend::Scalar);
        assert_eq!(active(), SpongeBackend::Scalar);
        set_backend(SpongeBackend::TwoBit);
        assert_eq!(active(), SpongeBackend::TwoBit);
    }

    #[test]
    fn detect_best_returns_valid() {
        let best = detect_best();
        assert!(
            best == SpongeBackend::TwoBit
            || best == SpongeBackend::Avx2
            || best == SpongeBackend::Neon,
        );
    }

    #[test]
    fn backend_names() {
        assert_eq!(SpongeBackend::Scalar.name(), "scalar");
        assert_eq!(SpongeBackend::Packed.name(), "packed-gf27");
        assert_eq!(SpongeBackend::Avx2.name(), "avx2-packed");
        assert_eq!(SpongeBackend::Neon.name(), "neon-packed");
        assert_eq!(SpongeBackend::TwoBit.name(), "2bit-packed");
    }

    #[test]
    fn dispatched_derive_key_deterministic() {
        set_backend(SpongeBackend::TwoBit);
        let a = derive_key(b"TEST", b"material", 32);
        let b = derive_key(b"TEST", b"material", 32);
        assert_eq!(a, b);
    }

    #[test]
    fn dispatched_derive_key_domain_separation() {
        set_backend(SpongeBackend::TwoBit);
        let a = derive_key(b"DOMAIN-A", b"material", 32);
        let b = derive_key(b"DOMAIN-B", b"material", 32);
        assert_ne!(a, b);
    }

    #[test]
    fn dispatched_hash_hex_valid() {
        set_backend(SpongeBackend::TwoBit);
        let hex = hash_hex(b"hello");
        assert_eq!(hex.len(), 96);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn ab_compare_runs() {
        let result = ab_compare(b"TEST", b"material", 32);
        assert!(result.scalar_ns > 0);
        assert!(result.packed_ns > 0);
    }

    #[test]
    fn ab_benchmark_runs() {
        let result = ab_benchmark(b"TEST", b"mat", 32, 5);
        assert_eq!(result.iterations, 5);
        assert!(result.scalar_mean_ns > 0);
        assert!(result.packed_mean_ns > 0);
    }

    #[test]
    fn ab_speedup_positive() {
        let result = ab_benchmark(b"BENCH", b"material", 48, 10);
        assert!(result.mean_speedup > 0.0, "Speedup must be positive");
    }

    #[test]
    fn scalar_backend_works() {
        set_backend(SpongeBackend::Scalar);
        let out = derive_key(b"SCALAR", b"test", 32);
        assert_eq!(out.len(), 32);
        set_backend(SpongeBackend::TwoBit);
    }

    #[test]
    fn packed_backend_works() {
        set_backend(SpongeBackend::Packed);
        let out = derive_key(b"PACKED", b"test", 32);
        assert_eq!(out.len(), 32);
        set_backend(SpongeBackend::TwoBit);
    }

    #[test]
    fn twobit_backend_works() {
        set_backend(SpongeBackend::TwoBit);
        let out = derive_key(b"TWOBIT", b"test", 32);
        assert_eq!(out.len(), 32);
    }

    #[test]
    fn all_backends_produce_correct_length() {
        for len in [16, 27, 32, 48, 64] {
            set_backend(SpongeBackend::Scalar);
            let s = derive_key(b"LEN", b"test", len);
            assert_eq!(s.len(), len);

            set_backend(SpongeBackend::Packed);
            let p = derive_key(b"LEN", b"test", len);
            assert_eq!(p.len(), len);

            set_backend(SpongeBackend::TwoBit);
            let t = derive_key(b"LEN", b"test", len);
            assert_eq!(t.len(), len);
        }
        set_backend(SpongeBackend::TwoBit);
    }

    #[test]
    fn both_backends_deterministic() {
        let sa = crate::sponge::derive_key(b"DET", b"input", 32);
        let sb = crate::sponge::derive_key(b"DET", b"input", 32);
        assert_eq!(sa, sb, "Scalar must be deterministic");

        let pa = crate::sponge_packed::derive_key_packed(b"DET", b"input", 32);
        let pb = crate::sponge_packed::derive_key_packed(b"DET", b"input", 32);
        assert_eq!(pa, pb, "Packed must be deterministic");
    }

    #[test]
    fn both_backends_domain_separate() {
        let sa = crate::sponge::derive_key(b"DOM-A", b"input", 32);
        let sb = crate::sponge::derive_key(b"DOM-B", b"input", 32);
        assert_ne!(sa, sb, "Scalar must domain-separate");

        let pa = crate::sponge_packed::derive_key_packed(b"DOM-A", b"input", 32);
        let pb = crate::sponge_packed::derive_key_packed(b"DOM-B", b"input", 32);
        assert_ne!(pa, pb, "Packed must domain-separate");
    }
}
