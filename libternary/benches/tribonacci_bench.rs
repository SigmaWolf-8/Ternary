// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

use criterion::{criterion_group, criterion_main, Criterion};
use libternary::tribonacci::TribonacciBase3;

fn bench_tribonacci_sequence(c: &mut Criterion) {
    c.bench_function("tribonacci_base3_50_terms", |b| {
        b.iter(|| {
            let mut gen = TribonacciBase3::new();
            let mut terms = Vec::with_capacity(50);
            for _ in 0..50 {
                terms.push(gen.next_term());
            }
            terms
        });
    });
}

fn bench_tribonacci_single(c: &mut Criterion) {
    c.bench_function("tribonacci_base3_single", |b| {
        b.iter(|| {
            let mut gen = TribonacciBase3::new();
            gen.next_term()
        });
    });
}

criterion_group!(benches, bench_tribonacci_sequence, bench_tribonacci_single);
criterion_main!(benches);
