// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// TL-Sponge-385 Benchmark
// Location: src/kernel/benches/sponge_bench.rs
//
// Run:  cd src/kernel && cargo bench --bench sponge_bench
//
// Cargo.toml additions needed:
//   [[bench]]
//   name = "sponge_bench"
//   harness = false
//
//   [dev-dependencies]
//   criterion = { version = "0.5", features = ["html_reports"] }

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use plenumnet_kernel::crypto::sponge::{sponge_hash, sponge_hash_bytes, TernarySponge};

// ── Single-hash latency ──────────────────────────────────────────────────────

fn bench_single_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("sponge_single_hash");

    // Empty input — baseline permutation cost
    group.bench_function("empty", |b| {
        b.iter(|| sponge_hash(black_box(&[])))
    });

    // 27 trits — one TDNS classification layer
    let input_27: Vec<i8> = (0..27).map(|i| (i % 3) as i8 - 1).collect();
    group.bench_function("27_trits", |b| {
        b.iter(|| sponge_hash(black_box(&input_27)))
    });

    // 243 trits — exactly one rate block
    let input_243: Vec<i8> = (0..243).map(|i| (i % 3) as i8 - 1).collect();
    group.bench_function("243_trits_1_block", |b| {
        b.iter(|| sponge_hash(black_box(&input_243)))
    });

    // 729 trits — 3 rate blocks (full state width of input)
    let input_729: Vec<i8> = (0..729).map(|i| (i % 3) as i8 - 1).collect();
    group.bench_function("729_trits_3_blocks", |b| {
        b.iter(|| sponge_hash(black_box(&input_729)))
    });

    // Typical URL (64 bytes → 320 trits via absorb_bytes)
    group.bench_function("64_bytes_url", |b| {
        b.iter(|| sponge_hash_bytes(black_box(b"https://www.example.com/path/to/resource?query=value#frag")))
    });

    group.finish();
}

// ── Throughput (bytes/sec) ───────────────────────────────────────────────────

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("sponge_throughput");

    for &size in &[64, 256, 1024, 4096, 16384] {
        let input: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("absorb_bytes", size),
            &input,
            |b, data| b.iter(|| sponge_hash_bytes(black_box(data))),
        );
    }

    group.finish();
}

// ── Component isolation ──────────────────────────────────────────────────────

fn bench_components(c: &mut Criterion) {
    let mut group = c.benchmark_group("sponge_components");

    let input: Vec<i8> = (0..243).map(|i| (i % 3) as i8 - 1).collect();
    group.bench_function("absorb_243_no_squeeze", |b| {
        b.iter(|| {
            let mut sponge = TernarySponge::new();
            sponge.absorb(black_box(&input));
            black_box(&sponge);
        })
    });

    group.bench_function("squeeze_729_trits", |b| {
        b.iter(|| {
            let mut sponge = TernarySponge::new();
            sponge.absorb(&[1i8, 0, -1]);
            sponge.squeeze(black_box(729))
        })
    });

    group.bench_function("squeeze_243_default", |b| {
        b.iter(|| {
            let mut sponge = TernarySponge::new();
            sponge.absorb(&[1i8, 0, -1]);
            sponge.squeeze_default()
        })
    });

    group.finish();
}

// ── SIMD comparison ──────────────────────────────────────────────────────────

fn bench_simd_vs_scalar(c: &mut Criterion) {
    let mut group = c.benchmark_group("sponge_simd_comparison");

    for &size in &[243usize, 729, 2187] {
        let input: Vec<i8> = (0..size).map(|i| (i % 3) as i8 - 1).collect();
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("auto", size),
            &input,
            |b, data| b.iter(|| sponge_hash(black_box(data))),
        );
    }

    group.finish();
}

// ── Identity derivation cost ─────────────────────────────────────────────────

fn bench_identity_derivation(c: &mut Criterion) {
    let mut group = c.benchmark_group("sponge_identity_derivation");

    let urls: [(&str, &[u8]); 3] = [
        ("short",  b"https://google.com"),
        ("medium", b"https://www.example.com/products/category/item-12345"),
        ("long",   b"https://very-long-domain-name.enterprise.company.co.uk/deeply/nested/path/to/resource/with/many/segments"),
    ];

    for (label, url) in urls {
        group.bench_function(label, |b| {
            b.iter(|| {
                let mut sponge = TernarySponge::new();
                sponge.absorb_bytes(black_box(url));
                sponge.squeeze(27)
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_single_hash,
    bench_throughput,
    bench_components,
    bench_simd_vs_scalar,
    bench_identity_derivation,
);
criterion_main!(benches);
