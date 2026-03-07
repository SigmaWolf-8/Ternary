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
use libternary::ternary_circle::{Z28, walk_tribonacci_radian_spiral};

fn bench_z28_operations(c: &mut Criterion) {
    c.bench_function("z28_add_sub", |b| {
        b.iter(|| {
            let a = Z28::new(13);
            let b_val = Z28::new(7);
            let sum = a.add(b_val);
            let diff = a.sub(b_val);
            (sum, diff)
        });
    });
}

fn bench_radian_spiral(c: &mut Criterion) {
    c.bench_function("radian_spiral_walk", |b| {
        b.iter(|| {
            let trits: Vec<u8> = vec![1, 0, 2, 1, 1, 0, 2, 0, 1, 1, 2, 0, 1];
            walk_tribonacci_radian_spiral(&trits)
        });
    });
}

criterion_group!(benches, bench_z28_operations, bench_radian_spiral);
criterion_main!(benches);
