// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// All Rights Reserved.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.
//
// ============================================================================
// Generalized Qudit Fault-Tolerance Cycle Benchmark (d >= 3)
// Repetition code [[3,1,d-1]]_d simulation with full syndrome extraction
// + correction. Optimized with nalgebra, zero-copy where possible.
// ============================================================================

use std::time::Instant;
use nalgebra::{Complex, DMatrix, DVector};
use rand::Rng;

fn x_d(d: usize) -> DMatrix<Complex<f64>> {
    let mut m = DMatrix::zeros(d, d);
    for i in 0..d {
        m[(i, (i + 1) % d)] = Complex::new(1.0, 0.0);
    }
    m
}

fn z_d(d: usize) -> DMatrix<Complex<f64>> {
    let mut m = DMatrix::zeros(d, d);
    let omega = Complex::from_polar(1.0, 2.0 * std::f64::consts::PI / d as f64);
    for i in 0..d {
        m[(i, i)] = omega.powu(i as u32);
    }
    m
}

fn kron3(
    a: &DMatrix<Complex<f64>>,
    b: &DMatrix<Complex<f64>>,
    c: &DMatrix<Complex<f64>>,
) -> DMatrix<Complex<f64>> {
    a.kronecker(b).kronecker(c)
}

fn normalize(v: &mut DVector<Complex<f64>>) {
    let norm = v.norm();
    if norm > 0.0 {
        *v /= norm;
    }
}

fn full_ft_cycle(d: usize) -> DVector<Complex<f64>> {
    let mut rng = rand::thread_rng();

    let i_d = DMatrix::identity(d, d);
    let dim3 = d * d * d;

    let mut state = DVector::zeros(dim3);
    state[0] = Complex::new(1.0, 0.0);

    let target = rng.gen_range(0..3);
    let error_type = rng.gen_range(0..2);
    let op = if error_type == 0 { x_d(d) } else { z_d(d) };

    let full_op = match target {
        0 => kron3(&op, &i_d, &i_d),
        1 => kron3(&i_d, &op, &i_d),
        _ => kron3(&i_d, &i_d, &op),
    };

    state = full_op * state;
    normalize(&mut state);

    let mut syndrome = [0.0f64; 2];
    for i in 0..d {
        let idx0 = i * d * d;
        let idx1 = idx0 + d;
        if idx0 < dim3 && idx1 < dim3 {
            syndrome[0] += (state[idx0] - state[idx1]).norm_squared();
        }
    }
    for i in 0..d {
        let idx1 = i * d * d + d;
        let idx2 = idx1 + d;
        if idx1 < dim3 && idx2 < dim3 {
            syndrome[1] += (state[idx1] - state[idx2]).norm_squared();
        }
    }

    if syndrome[0] > 0.1 || syndrome[1] > 0.1 {
        let target_corr = if syndrome[0] > syndrome[1] { 0 } else { 1 };
        let corr_op = x_d(d);
        let full_corr = match target_corr {
            0 => kron3(&corr_op, &i_d, &i_d),
            1 => kron3(&i_d, &corr_op, &i_d),
            _ => kron3(&i_d, &i_d, &corr_op),
        };
        state = full_corr * state;
        normalize(&mut state);
    }

    state
}

fn main() {
    let ds = [3, 4, 5, 8, 13];
    let cycles_list = [100, 1000, 10000];
    let trials = 20;

    println!("Qudit Fault-Tolerance Cycle Benchmark (d >= 3 generalization)");
    println!("Repetition code [[3,1,d-1]]_d — full encode -> error -> syndrome -> correction\n");

    for &d in &ds {
        println!("Dimension d = {}", d);
        println!("State size = {} (d^3)\n", d * d * d);

        for &cycles in &cycles_list {
            let mut total_duration = std::time::Duration::ZERO;

            for _ in 0..trials {
                let start = Instant::now();
                for _ in 0..cycles {
                    let _ = full_ft_cycle(d);
                }
                total_duration += start.elapsed();
            }

            let mean_ms = total_duration.as_secs_f64() * 1000.0 / trials as f64;
            let ops_per_sec = (cycles * trials) as f64 / total_duration.as_secs_f64();

            println!("  {cycles:5} cycles/trial ({trials} trials):");
            println!("     Mean time = {mean_ms:6.2} ms");
            println!("     Ops/sec   = {ops_per_sec:8.0}");
            println!();
        }
        println!("{}", "-".repeat(60));
    }
}
