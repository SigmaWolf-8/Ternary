// Criterion benchmarks for the Trit type system.
//
// NATIVE-ONLY: Criterion does not run on WASM.
// Run with: cargo bench --bench trit_benchmarks
//
// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ternary_math::trit_int::TritInt;
use ternary_math::trit::Trit;

fn bench_trit_int_add(c: &mut Criterion) {
    let a = TritInt::from_u64(182); // 8-trit value
    let b = TritInt::from_u64(364); // 8-trit value
    c.bench_function("TritInt::add 8-trit", |bencher| {
        bencher.iter(|| TritInt::add(black_box(&a), black_box(&b)))
    });
}

fn bench_trit_int_mul(c: &mut Criterion) {
    let a = TritInt::from_u64(182);
    let b = TritInt::from_u64(364);
    c.bench_function("TritInt::mul 8-trit × 8-trit", |bencher| {
        bencher.iter(|| TritInt::mul(black_box(&a), black_box(&b)))
    });
}

fn bench_trit_int_div_mod(c: &mut Criterion) {
    let a = TritInt::from_u64(118_300); // 16-trit value
    let b = TritInt::from_u64(182);     // 8-trit value
    c.bench_function("TritInt::div_mod 16÷8 trit", |bencher| {
        bencher.iter(|| black_box(&a).div_mod(black_box(&b)))
    });
}

fn bench_trit_mul_golden(c: &mut Criterion) {
    let r_sq = Trit::golden(TritInt::from_u64(14), TritInt::from_u64(5));
    let icosa = Trit::golden(TritInt::from_u64(2), TritInt::from_u64(1));
    c.bench_function("Trit::mul_golden R² × icosa", |bencher| {
        bencher.iter(|| black_box(&r_sq).mul_golden(black_box(&icosa)))
    });
}

fn bench_trit_mul_eisenstein(c: &mut Criterion) {
    let a = Trit::eisenstein(TritInt::from_u64(2), TritInt::from_u64(1));
    let b = Trit::eisenstein(TritInt::from_u64(1), TritInt::from_u64(2));
    c.bench_function("Trit::mul_eisenstein GF(3)", |bencher| {
        bencher.iter(|| black_box(&a).mul_eisenstein(black_box(&b)))
    });
}

fn bench_trit_norm_golden(c: &mut Criterion) {
    let r_sq = Trit::golden(TritInt::from_u64(14), TritInt::from_u64(5));
    c.bench_function("Trit::norm_golden R²", |bencher| {
        bencher.iter(|| black_box(&r_sq).norm_golden())
    });
}

fn bench_ags_crt(c: &mut Criterion) {
    use ternary_math::ags::{Ags, derive_ags_seed, derive_ags_capacity};

    let ags = Ags::new();
    let test_position = TritInt::from_u64(42);
    c.bench_function("Ags::crt_project seed", |bencher| {
        bencher.iter(|| black_box(&ags).crt_project(black_box(&test_position)))
    });
}

fn bench_tri182_project(c: &mut Criterion) {
    use ternary_math::tri182::project_to_zphi;

    let classification = [1u8, 2, 0, 1, 2, 1, 0, 2, 1, 1, 2, 0, 1, 0, 2, 1, 2, 0, 1, 1, 2, 0, 1, 2, 1, 0, 2];
    c.bench_function("tri182::project_to_zphi 27-trit", |bencher| {
        bencher.iter(|| project_to_zphi(black_box(&classification)))
    });
}

criterion_group!(
    benches,
    bench_trit_int_add,
    bench_trit_int_mul,
    bench_trit_int_div_mod,
    bench_trit_mul_golden,
    bench_trit_mul_eisenstein,
    bench_trit_norm_golden,
    bench_ags_crt,
    bench_tri182_project,
);
criterion_main!(benches);
