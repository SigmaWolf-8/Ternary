// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// TIS-27 Benchmark
// Location: ternary-math/benches/tis27_bench.rs
//
// Run:  cd ternary-math && cargo bench --bench tis27_bench
//
// Cargo.toml additions needed:
//   [[bench]]
//   name = "tis27_bench"
//   harness = false
//
//   [dev-dependencies]
//   criterion = { version = "0.5", features = ["html_reports"] }

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

// ── ADJUST THIS IMPORT ──────────────────────────────────────────────────────
// Check ternary-math/src/lib.rs for how tis_sponge.rs is exported.
// Possible paths:
//   use ternary_math::tis_sponge::tis27_hash;
//   use ternary_math::tis27_hash;
// Pick whichever matches your lib.rs re-exports.
use ternary_math::tis_sponge::tis27_hash;
// ─────────────────────────────────────────────────────────────────────────────

// ── Single-hash latency ──────────────────────────────────────────────────────

fn bench_tis27_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("tis27_single_hash");

    // 27-trit classification address — the primary scan hash use case
    let input_27: Vec<u8> = (0..27).map(|i| (i % 3) as u8).collect();
    group.bench_function("27_trits_scan_hash", |b| {
        b.iter(|| tis27_hash(black_box(&input_27), 27))
    });

    // Empty input — baseline
    group.bench_function("empty", |b| {
        b.iter(|| tis27_hash(black_box(&[]), 27))
    });

    // 54 trits — full state width
    let input_54: Vec<u8> = (0..54).map(|i| (i % 3) as u8).collect();
    group.bench_function("54_trits_full_state", |b| {
        b.iter(|| tis27_hash(black_box(&input_54), 27))
    });

    group.finish();
}

// ── Throughput ───────────────────────────────────────────────────────────────

fn bench_tis27_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("tis27_throughput");

    for &size in &[27usize, 54, 108, 243, 512] {
        let input: Vec<u8> = (0..size).map(|i| (i % 3) as u8).collect();
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("integrity_check", size),
            &input,
            |b, data| b.iter(|| tis27_hash(black_box(data), 27)),
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_tis27_single,
    bench_tis27_throughput,
);
criterion_main!(benches);
