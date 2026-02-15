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
// Qutrit Fault-Tolerance Cycle Benchmark (Rust Kernel)
// [[3,1,2]]_3 full cycle: encode → error → syndrome → correction
// ============================================================================

use std::time::{Duration, Instant};
use nalgebra::{Complex, DMatrix, DVector};
use rand::Rng;

type CMatrix = DMatrix<Complex<f64>>;
type CVector = DVector<Complex<f64>>;

fn x3() -> CMatrix {
    CMatrix::from_row_slice(3, 3, &[
        Complex::new(0.0, 0.0), Complex::new(0.0, 0.0), Complex::new(1.0, 0.0),
        Complex::new(1.0, 0.0), Complex::new(0.0, 0.0), Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0), Complex::new(1.0, 0.0), Complex::new(0.0, 0.0),
    ])
}

fn z3() -> CMatrix {
    let omega = Complex::from_polar(1.0, 2.0 * std::f64::consts::PI / 3.0);
    CMatrix::from_row_slice(3, 3, &[
        Complex::new(1.0, 0.0), Complex::new(0.0, 0.0), Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0), omega,                   Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0), Complex::new(0.0, 0.0), omega.conj(),
    ])
}

fn i3() -> CMatrix {
    CMatrix::identity(3, 3)
}

fn kron3(a: &CMatrix, b: &CMatrix, c: &CMatrix) -> CMatrix {
    a.kronecker(b).kronecker(c)
}

fn normalize(v: &mut CVector) {
    let norm = v.norm();
    if norm > 0.0 {
        *v /= norm;
    }
}

fn expectation(state: &CVector, op: &CMatrix) -> f64 {
    let bra = state.adjoint();
    (bra * op * state)[(0, 0)].re
}

fn apply_single_op(state: &CVector, target: usize, op: &CMatrix) -> CVector {
    let id = i3();
    let full_op = match target {
        0 => kron3(op, &id, &id),
        1 => kron3(&id, op, &id),
        _ => kron3(&id, &id, op),
    };
    full_op * state
}

fn encode(logical_idx: usize) -> CVector {
    let mut state = CVector::zeros(27);
    let idx = logical_idx * 9 + logical_idx * 3 + logical_idx;
    state[idx] = Complex::new(1.0, 0.0);
    state
}

fn full_ft_cycle() -> (CVector, bool) {
    let mut rng = rand::thread_rng();

    let mut state = encode(0);

    let target = rng.gen_range(0..3);
    let op_idx = rng.gen_range(0..3);
    let op = match op_idx {
        0 => i3(),
        1 => x3(),
        _ => z3(),
    };
    state = apply_single_op(&state, target, &op);

    let s1_op = kron3(&x3(), &x3(), &x3());
    let s2_op = kron3(&(&x3() * &x3()), &x3(), &x3());
    let s1 = expectation(&state, &s1_op);
    let s2 = expectation(&state, &s2_op);

    let mut corrected = false;
    if s1.abs() > 0.1 || s2.abs() > 0.1 {
        let corr_op = if s1.abs() > s2.abs() { x3() } else { z3() };
        let corr_target = if s1.abs() > s2.abs() { 0 } else { 1 };
        state = apply_single_op(&state, corr_target, &corr_op);
        normalize(&mut state);
        corrected = true;
    }

    (state, corrected)
}

fn main() {
    let cycles_list = [100, 1000, 10000];
    let trials = 15;

    println!("Qutrit Fault-Tolerance Cycle Benchmark (Rust Kernel)");
    println!("[[3,1,2]]_3 full cycle: encode -> error -> syndrome -> correction\n");

    for &cycles in &cycles_list {
        let mut total_duration = Duration::ZERO;
        let mut total_corrections = 0u64;

        for _ in 0..trials {
            let start = Instant::now();
            for _ in 0..cycles {
                let (_, corrected) = full_ft_cycle();
                if corrected {
                    total_corrections += 1;
                }
            }
            total_duration += start.elapsed();
        }

        let mean_ms = total_duration.as_secs_f64() * 1000.0 / trials as f64;
        let ops_per_sec = (cycles * trials) as f64 / total_duration.as_secs_f64();
        let correction_rate = total_corrections as f64 / (cycles * trials) as f64 * 100.0;

        println!("{cycles} cycles/trial ({trials} trials):");
        println!("   Mean time       = {mean_ms:.2} ms");
        println!("   Ops/sec         = {ops_per_sec:.0}");
        println!("   Correction rate = {correction_rate:.1}%");
        println!();
    }
}
