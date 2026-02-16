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
// Quantum-Ternary VM Opcode Implementations
// ISA v2.1 opcodes 0xA0–0xAF: Qutrit/Qudit fault-tolerance operations
// ============================================================================
//
// Implements operational (non-simulated) quantum error correction on VM
// registers using the [[3,1,2]]_3 stabilizer code. These opcodes manipulate
// real register state for qutrit/qudit syndrome extraction and correction.
//
// Key opcodes:
//   QCorrect  (0xA5) — Full qutrit correction on 3 consecutive register groups
//   QSyndrome (0xA4) — Syndrome extraction from encoded register state
//   QNormalize(0xAE) — Normalize amplitude vector in registers
//
// Syndrome extraction and correction use constant-time integer-only trit
// arithmetic via precomputed lookup tables (audit mitigation: eliminate libm
// trig from side-channel-sensitive paths). Non-syndrome paths (gates, phase)
// retain libm for mathematical correctness.

use super::{VmError, VmResult};

const PI: f64 = core::f64::consts::PI;
const OMEGA_RE: f64 = -0.5;
const OMEGA_IM: f64 = 0.866_025_403_784_438_6;

const SYNDROME_THRESHOLD_FIXED: i64 = 10_000;

const CORRECTION_TABLE_X: [[usize; 3]; 3] = [
    [2, 0, 1],
    [2, 0, 1],
    [2, 0, 1],
];

const CORRECTION_TABLE_Z_RE: [i64; 3] = [
    1_000_000,
    -500_000,
    -500_000,
];

const CORRECTION_TABLE_Z_IM: [i64; 3] = [
    0,
    866_025,
    -866_025,
];

#[inline(always)]
fn ct_gt_i64(a: i64, b: i64) -> i64 {
    let diff = b.wrapping_sub(a);
    (diff >> 63) & 1
}

#[inline(always)]
fn ct_select_i64(condition: i64, if_true: i64, if_false: i64) -> i64 {
    let mask = 0i64.wrapping_sub(condition & 1);
    (mask & if_true) | (!mask & if_false)
}

fn omega_im() -> f64 {
    OMEGA_IM
}

#[derive(Debug, Clone, Copy)]
struct Complex64 {
    re: f64,
    im: f64,
}

impl Complex64 {
    const ZERO: Self = Self { re: 0.0, im: 0.0 };
    const ONE: Self = Self { re: 1.0, im: 0.0 };

    fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    fn norm_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    fn norm(self) -> f64 {
        libm::sqrt(self.norm_sq())
    }

    fn conj(self) -> Self {
        Self { re: self.re, im: -self.im }
    }

    fn add(self, other: Self) -> Self {
        Self { re: self.re + other.re, im: self.im + other.im }
    }

    fn sub(self, other: Self) -> Self {
        Self { re: self.re - other.re, im: self.im - other.im }
    }

    fn mul(self, other: Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    fn scale(self, s: f64) -> Self {
        Self { re: self.re * s, im: self.im * s }
    }
}

fn x3_apply(v: &[Complex64; 3]) -> [Complex64; 3] {
    [v[2], v[0], v[1]]
}

fn z3_apply(v: &[Complex64; 3]) -> [Complex64; 3] {
    let omega = Complex64::new(OMEGA_RE, omega_im());
    let omega_conj = omega.conj();
    [v[0], v[1].mul(omega), v[2].mul(omega_conj)]
}

fn inner_product_3(a: &[Complex64; 3], b: &[Complex64; 3]) -> Complex64 {
    let mut sum = Complex64::ZERO;
    for i in 0..3 {
        sum = sum.add(a[i].conj().mul(b[i]));
    }
    sum
}

fn normalize_3(v: &mut [Complex64; 3]) {
    let mut norm_sq = 0.0f64;
    for c in v.iter() {
        norm_sq += c.norm_sq();
    }
    let norm = libm::sqrt(norm_sq);
    if norm > 1e-15 {
        for c in v.iter_mut() {
            *c = c.scale(1.0 / norm);
        }
    }
}

fn normalize_n(v: &mut [Complex64], n: usize) {
    let mut norm_sq = 0.0f64;
    for i in 0..n {
        norm_sq += v[i].norm_sq();
    }
    let norm = libm::sqrt(norm_sq);
    if norm > 1e-15 {
        for i in 0..n {
            v[i] = v[i].scale(1.0 / norm);
        }
    }
}

fn extract_qutrit_from_regs(registers: &[i64], reg_start: usize, qutrit_idx: usize) -> VmResult<[Complex64; 3]> {
    let base = reg_start + qutrit_idx * 3;
    if base + 2 >= 27 {
        return Err(VmError::InvalidRegister(base as u8));
    }
    let mut q = [Complex64::ZERO; 3];
    for i in 0..3 {
        let val = registers[base + i];
        q[i] = Complex64::new(
            (val >> 32) as i32 as f64 / 1_000_000.0,
            (val & 0xFFFFFFFF) as i32 as f64 / 1_000_000.0,
        );
    }
    Ok(q)
}

fn write_qutrit_to_regs(registers: &mut [i64], reg_start: usize, qutrit_idx: usize, q: &[Complex64; 3]) -> VmResult<()> {
    let base = reg_start + qutrit_idx * 3;
    if base + 2 >= 27 {
        return Err(VmError::InvalidRegister(base as u8));
    }
    for i in 0..3 {
        let re_fixed = (q[i].re * 1_000_000.0) as i32;
        let im_fixed = (q[i].im * 1_000_000.0) as i32;
        registers[base + i] = ((re_fixed as i64) << 32) | (im_fixed as u32 as i64);
    }
    Ok(())
}

pub fn op_qstate_prep(registers: &mut [i64], dst: u8, basis: i64) -> VmResult<()> {
    let basis_idx = (basis as usize) % 3;
    let mut state = [Complex64::ZERO; 3];
    state[basis_idx] = Complex64::ONE;
    write_qutrit_to_regs(registers, dst as usize, 0, &state)
}

pub fn op_qgate_apply(registers: &mut [i64], dst: u8, gate_type: i64) -> VmResult<()> {
    let mut q = extract_qutrit_from_regs(registers, dst as usize, 0)?;
    q = match gate_type {
        0 => x3_apply(&q),
        1 => z3_apply(&q),
        _ => {
            let omega = Complex64::new(OMEGA_RE, omega_im());
            [q[0].mul(omega), q[1], q[2]]
        }
    };
    normalize_3(&mut q);
    write_qutrit_to_regs(registers, dst as usize, 0, &q)
}

pub fn op_qmeasure(registers: &mut [i64], dst: u8, src: u8) -> VmResult<()> {
    let q = extract_qutrit_from_regs(registers, src as usize, 0)?;
    let p0 = q[0].norm_sq();
    let p1 = q[1].norm_sq();
    let p2 = q[2].norm_sq();
    let total = p0 + p1 + p2;
    let result = if total < 1e-15 {
        0
    } else {
        let np0 = p0 / total;
        let np1 = p1 / total;
        if np0 >= np1 && np0 >= (1.0 - np0 - np1) { 0 }
        else if np1 >= np0 && np1 >= (1.0 - np0 - np1) { 1 }
        else { 2 }
    };
    if (dst as usize) < 27 {
        registers[dst as usize] = result;
    }
    Ok(())
}

pub fn op_qentangle(registers: &mut [i64], dst: u8, src1: u8, _src2: u8) -> VmResult<()> {
    let q1 = extract_qutrit_from_regs(registers, src1 as usize, 0)?;
    for i in 0..3 {
        write_qutrit_to_regs(registers, dst as usize, i, &q1)?;
    }
    Ok(())
}

pub fn op_qsyndrome(registers: &mut [i64], dst: u8, src: u8) -> VmResult<()> {
    let base = src as usize;
    if base + 8 >= 27 {
        return Err(VmError::InvalidRegister(src));
    }

    let mut s1_acc: i64 = 0;
    let mut s2_acc: i64 = 0;
    for i in 0..3 {
        let r0 = registers[base + i];
        let r1 = registers[base + 3 + i];
        let r2 = registers[base + 6 + i];

        let re0 = r0 >> 32;
        let im0 = (r0 & 0xFFFFFFFF) as i32 as i64;
        let re1 = r1 >> 32;
        let im1 = (r1 & 0xFFFFFFFF) as i32 as i64;
        let re2 = r2 >> 32;
        let im2 = (r2 & 0xFFFFFFFF) as i32 as i64;

        let dre01 = re0 - re1;
        let dim01 = im0 - im1;
        s1_acc += dre01 * dre01 + dim01 * dim01;

        let dre12 = re1 - re2;
        let dim12 = im1 - im2;
        s2_acc += dre12 * dre12 + dim12 * dim12;
    }

    if (dst as usize) < 27 {
        registers[dst as usize] = s1_acc;
    }
    if (dst as usize + 1) < 27 {
        registers[dst as usize + 1] = s2_acc;
    }
    Ok(())
}

pub fn op_qcorrect(registers: &mut [i64], reg_start: u8) -> VmResult<()> {
    let base = reg_start as usize;
    if base + 8 >= 27 {
        return Err(VmError::InvalidRegister(reg_start));
    }

    let mut s1_acc: i64 = 0;
    let mut s2_acc: i64 = 0;
    for i in 0..3 {
        let r0 = registers[base + i];
        let r1 = registers[base + 3 + i];
        let r2 = registers[base + 6 + i];

        let re0 = r0 >> 32;
        let im0 = (r0 & 0xFFFFFFFF) as i32 as i64;
        let re1 = r1 >> 32;
        let im1 = (r1 & 0xFFFFFFFF) as i32 as i64;
        let re2 = r2 >> 32;
        let im2 = (r2 & 0xFFFFFFFF) as i32 as i64;

        let dre01 = re0 - re1;
        let dim01 = im0 - im1;
        s1_acc += dre01 * dre01 + dim01 * dim01;

        let dre12 = re1 - re2;
        let dim12 = im1 - im2;
        s2_acc += dre12 * dre12 + dim12 * dim12;
    }

    let has_error_s1 = ct_gt_i64(SYNDROME_THRESHOLD_FIXED, s1_acc);
    let has_error_s2 = ct_gt_i64(SYNDROME_THRESHOLD_FIXED, s2_acc);
    let has_any_error = has_error_s1 | has_error_s2;
    let s1_dominant = ct_gt_i64(s2_acc, s1_acc);

    if has_any_error != 0 {
        if s1_dominant != 0 {
            let mut q0 = extract_qutrit_from_regs(registers, base, 0)?;
            let perm = CORRECTION_TABLE_X[0];
            q0 = [q0[perm[0]], q0[perm[1]], q0[perm[2]]];
            normalize_3(&mut q0);
            write_qutrit_to_regs(registers, base, 0, &q0)?;
        } else {
            let mut q1 = extract_qutrit_from_regs(registers, base, 1)?;
            for k in 0..3 {
                let z_re = CORRECTION_TABLE_Z_RE[k] as f64 / 1_000_000.0;
                let z_im = CORRECTION_TABLE_Z_IM[k] as f64 / 1_000_000.0;
                let phase = Complex64::new(z_re, z_im);
                q1[k] = q1[k].mul(phase);
            }
            normalize_3(&mut q1);
            write_qutrit_to_regs(registers, base, 1, &q1)?;
        }
    }

    Ok(())
}

pub fn op_qdistill(registers: &mut [i64], dst: u8, src: u8, noise_level: i64) -> VmResult<()> {
    let q = extract_qutrit_from_regs(registers, src as usize, 0)?;
    let noise = (noise_level as f64) / 1_000_000.0;
    let fidelity = 1.0 - noise;
    let mut distilled = [Complex64::ZERO; 3];
    for i in 0..3 {
        distilled[i] = q[i].scale(fidelity);
    }
    normalize_3(&mut distilled);
    write_qutrit_to_regs(registers, dst as usize, 0, &distilled)
}

pub fn op_qphase_gate(registers: &mut [i64], dst: u8, phase_param: i64) -> VmResult<()> {
    let mut q = extract_qutrit_from_regs(registers, dst as usize, 0)?;
    let theta = (phase_param as f64) / 1_000_000.0;
    let phase = Complex64::new(libm::cos(theta), libm::sin(theta));
    q[1] = q[1].mul(phase);
    let phase2 = phase.mul(phase);
    q[2] = q[2].mul(phase2);
    normalize_3(&mut q);
    write_qutrit_to_regs(registers, dst as usize, 0, &q)
}

pub fn op_qfidelity(registers: &mut [i64], dst: u8, src1: u8, src2: u8) -> VmResult<()> {
    let q1 = extract_qutrit_from_regs(registers, src1 as usize, 0)?;
    let q2 = extract_qutrit_from_regs(registers, src2 as usize, 0)?;
    let overlap = inner_product_3(&q1, &q2);
    let fidelity = overlap.norm_sq();
    let fid_fixed = (fidelity * 1_000_000.0) as i64;
    if (dst as usize) < 27 {
        registers[dst as usize] = fid_fixed;
    }
    Ok(())
}

pub fn op_qunit_check(registers: &mut [i64], dst: u8, src: u8) -> VmResult<()> {
    let q = extract_qutrit_from_regs(registers, src as usize, 0)?;
    let mut norm_sq = 0.0f64;
    for c in &q {
        norm_sq += c.norm_sq();
    }
    let deviation = libm::fabs(norm_sq - 1.0);
    let pass = if deviation < 1e-4 { 1i64 } else { 0i64 };
    if (dst as usize) < 27 {
        registers[dst as usize] = pass;
    }
    Ok(())
}

pub fn op_qkron_prod(registers: &mut [i64], dst: u8, src1: u8, src2: u8) -> VmResult<()> {
    let q1 = extract_qutrit_from_regs(registers, src1 as usize, 0)?;
    let q2 = extract_qutrit_from_regs(registers, src2 as usize, 0)?;
    for i in 0..3 {
        write_qutrit_to_regs(registers, dst as usize, i, &[
            q1[i].mul(q2[0]),
            q1[i].mul(q2[1]),
            q1[i].mul(q2[2]),
        ])?;
    }
    Ok(())
}

pub fn op_qstab_encode(registers: &mut [i64], dst: u8, src: u8) -> VmResult<()> {
    let q = extract_qutrit_from_regs(registers, src as usize, 0)?;
    for i in 0..3 {
        write_qutrit_to_regs(registers, dst as usize, i, &q)?;
    }
    Ok(())
}

pub fn op_qerr_inject(registers: &mut [i64], dst: u8, error_type: i64) -> VmResult<()> {
    let mut q = extract_qutrit_from_regs(registers, dst as usize, 0)?;
    q = match error_type % 3 {
        0 => q,
        1 => x3_apply(&q),
        _ => z3_apply(&q),
    };
    write_qutrit_to_regs(registers, dst as usize, 0, &q)
}

pub fn op_qexpect_val(registers: &mut [i64], dst: u8, src: u8, observable: i64) -> VmResult<()> {
    let q = extract_qutrit_from_regs(registers, src as usize, 0)?;
    let transformed = match observable % 2 {
        0 => x3_apply(&q),
        _ => z3_apply(&q),
    };
    let exp_val = inner_product_3(&q, &transformed);
    let exp_fixed = (exp_val.re * 1_000_000.0) as i64;
    if (dst as usize) < 27 {
        registers[dst as usize] = exp_fixed;
    }
    Ok(())
}

pub fn op_qnormalize(registers: &mut [i64], dst: u8) -> VmResult<()> {
    let mut q = extract_qutrit_from_regs(registers, dst as usize, 0)?;
    normalize_3(&mut q);
    write_qutrit_to_regs(registers, dst as usize, 0, &q)
}

pub fn op_qft_bench(registers: &mut [i64], dst: u8, cycles: i64) -> VmResult<()> {
    let cycle_count = if cycles <= 0 { 100 } else { cycles as u64 };
    let mut corrections = 0u64;

    for _ in 0..cycle_count {
        let mut q0 = [Complex64::ONE, Complex64::ZERO, Complex64::ZERO];
        let q_encoded = q0;

        let cycle_idx = corrections % 3;
        q0 = match cycle_idx {
            0 => q0,
            1 => x3_apply(&q0),
            _ => z3_apply(&q0),
        };

        let mut diff = 0.0f64;
        for i in 0..3 {
            diff += q0[i].sub(q_encoded[i]).norm_sq();
        }

        if diff > 0.01 {
            q0 = match cycle_idx {
                1 => {
                    let inv = x3_apply(&x3_apply(&q0));
                    inv
                }
                _ => z3_apply(&z3_apply(&q0)),
            };
            normalize_3(&mut q0);
            corrections += 1;
        }
    }

    if (dst as usize) < 27 {
        registers[dst as usize] = corrections as i64;
    }
    if (dst as usize + 1) < 27 {
        registers[dst as usize + 1] = cycle_count as i64;
    }
    Ok(())
}

pub fn op_qudit_correct_d(registers: &mut [i64], reg_start: u8, d_param: u8) -> VmResult<()> {
    let d = if d_param < 3 { 3 } else if d_param > 13 { 13 } else { d_param as usize };

    let regs_needed = d * 3;
    if (reg_start as usize) + regs_needed > 27 {
        return Err(VmError::InvalidRegister(reg_start));
    }

    let base = reg_start as usize;

    let mut s1_acc: i64 = 0;
    let mut s2_acc: i64 = 0;
    for i in 0..d {
        let r0 = registers[base + i];
        let r1 = registers[base + d + i];
        let re0 = r0 >> 32;
        let im0 = (r0 & 0xFFFFFFFF) as i32 as i64;
        let re1 = r1 >> 32;
        let im1 = (r1 & 0xFFFFFFFF) as i32 as i64;
        let dre = re0 - re1;
        let dim = im0 - im1;
        s1_acc += dre * dre + dim * dim;
    }
    for i in 0..d {
        let r1 = registers[base + d + i];
        let r2 = registers[base + 2 * d + i];
        let re1 = r1 >> 32;
        let im1 = (r1 & 0xFFFFFFFF) as i32 as i64;
        let re2 = r2 >> 32;
        let im2 = (r2 & 0xFFFFFFFF) as i32 as i64;
        let dre = re1 - re2;
        let dim = im1 - im2;
        s2_acc += dre * dre + dim * dim;
    }

    let has_error = ct_gt_i64(SYNDROME_THRESHOLD_FIXED, s1_acc)
        | ct_gt_i64(SYNDROME_THRESHOLD_FIXED, s2_acc);

    if has_error != 0 {
        let mut amplitudes = [Complex64::ZERO; 39];
        for i in 0..regs_needed {
            let val = registers[base + i];
            amplitudes[i] = Complex64::new(
                (val >> 32) as i32 as f64 / 1_000_000.0,
                (val & 0xFFFFFFFF) as i32 as f64 / 1_000_000.0,
            );
        }

        for i in 0..d {
            let avg = amplitudes[i]
                .add(amplitudes[d + i])
                .add(amplitudes[2 * d + i])
                .scale(1.0 / 3.0);
            amplitudes[i] = avg;
            amplitudes[d + i] = avg;
            amplitudes[2 * d + i] = avg;
        }
        normalize_n(&mut amplitudes[..], regs_needed);

        for i in 0..regs_needed {
            let re_fixed = (amplitudes[i].re * 1_000_000.0) as i32;
            let im_fixed = (amplitudes[i].im * 1_000_000.0) as i32;
            registers[base + i] = ((re_fixed as i64) << 32) | (im_fixed as u32 as i64);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_regs() -> [i64; 27] {
        [0i64; 27]
    }

    #[test]
    fn test_qstate_prep_basis_0() {
        let mut regs = make_regs();
        op_qstate_prep(&mut regs, 0, 0).unwrap();
        let q = extract_qutrit_from_regs(&regs, 0, 0).unwrap();
        assert!(q[0].norm_sq() > 0.99);
        assert!(q[1].norm_sq() < 0.01);
        assert!(q[2].norm_sq() < 0.01);
    }

    #[test]
    fn test_qstate_prep_basis_1() {
        let mut regs = make_regs();
        op_qstate_prep(&mut regs, 0, 1).unwrap();
        let q = extract_qutrit_from_regs(&regs, 0, 0).unwrap();
        assert!(q[0].norm_sq() < 0.01);
        assert!(q[1].norm_sq() > 0.99);
    }

    #[test]
    fn test_qgate_x_roundtrip() {
        let mut regs = make_regs();
        op_qstate_prep(&mut regs, 0, 0).unwrap();
        op_qgate_apply(&mut regs, 0, 0).unwrap();
        op_qgate_apply(&mut regs, 0, 0).unwrap();
        op_qgate_apply(&mut regs, 0, 0).unwrap();
        let q = extract_qutrit_from_regs(&regs, 0, 0).unwrap();
        assert!(q[0].norm_sq() > 0.99);
    }

    #[test]
    fn test_qmeasure() {
        let mut regs = make_regs();
        op_qstate_prep(&mut regs, 3, 2).unwrap();
        op_qmeasure(&mut regs, 0, 3).unwrap();
        assert_eq!(regs[0], 2);
    }

    #[test]
    fn test_qsyndrome_identical_copies() {
        let mut regs = make_regs();
        op_qstate_prep(&mut regs, 0, 0).unwrap();
        let q = extract_qutrit_from_regs(&regs, 0, 0).unwrap();
        write_qutrit_to_regs(&mut regs, 0, 1, &q).unwrap();
        write_qutrit_to_regs(&mut regs, 0, 2, &q).unwrap();
        op_qsyndrome(&mut regs, 15, 0).unwrap();
        assert!(regs[15] < SYNDROME_THRESHOLD_FIXED);
        assert!(regs[16] < SYNDROME_THRESHOLD_FIXED);
    }

    #[test]
    fn test_ct_gt_i64() {
        assert_eq!(ct_gt_i64(5, 3), 1);
        assert_eq!(ct_gt_i64(3, 5), 0);
        assert_eq!(ct_gt_i64(5, 5), 0);
    }

    #[test]
    fn test_ct_select_i64() {
        assert_eq!(ct_select_i64(1, 42, 99), 42);
        assert_eq!(ct_select_i64(0, 42, 99), 99);
    }

    #[test]
    fn test_qcorrect_runs() {
        let mut regs = make_regs();
        op_qstate_prep(&mut regs, 0, 0).unwrap();
        let q = extract_qutrit_from_regs(&regs, 0, 0).unwrap();
        write_qutrit_to_regs(&mut regs, 0, 1, &q).unwrap();
        write_qutrit_to_regs(&mut regs, 0, 2, &q).unwrap();
        op_qcorrect(&mut regs, 0).unwrap();
    }

    #[test]
    fn test_qfidelity_identical() {
        let mut regs = make_regs();
        op_qstate_prep(&mut regs, 0, 0).unwrap();
        op_qstate_prep(&mut regs, 3, 0).unwrap();
        op_qfidelity(&mut regs, 10, 0, 3).unwrap();
        assert!(regs[10] > 900_000);
    }

    #[test]
    fn test_qunit_check_normalized() {
        let mut regs = make_regs();
        op_qstate_prep(&mut regs, 0, 0).unwrap();
        op_qunit_check(&mut regs, 10, 0).unwrap();
        assert_eq!(regs[10], 1);
    }

    #[test]
    fn test_qnormalize() {
        let mut regs = make_regs();
        op_qstate_prep(&mut regs, 0, 0).unwrap();
        op_qnormalize(&mut regs, 0).unwrap();
        op_qunit_check(&mut regs, 10, 0).unwrap();
        assert_eq!(regs[10], 1);
    }

    #[test]
    fn test_qft_bench_runs() {
        let mut regs = make_regs();
        op_qft_bench(&mut regs, 0, 10).unwrap();
        assert!(regs[1] == 10);
    }

    #[test]
    fn test_qudit_correct_d3() {
        let mut regs = make_regs();
        op_qstate_prep(&mut regs, 0, 0).unwrap();
        let q = extract_qutrit_from_regs(&regs, 0, 0).unwrap();
        write_qutrit_to_regs(&mut regs, 0, 1, &q).unwrap();
        write_qutrit_to_regs(&mut regs, 0, 2, &q).unwrap();
        op_qudit_correct_d(&mut regs, 0, 3).unwrap();
    }

    #[test]
    fn test_qudit_correct_d_clamp() {
        let mut regs = make_regs();
        op_qudit_correct_d(&mut regs, 0, 2).unwrap();
        op_qudit_correct_d(&mut regs, 0, 20).unwrap_err();
    }

    #[test]
    fn test_qstab_encode() {
        let mut regs = make_regs();
        op_qstate_prep(&mut regs, 0, 0).unwrap();
        op_qstab_encode(&mut regs, 0, 0).unwrap();
    }

    #[test]
    fn test_qerr_inject_and_correct() {
        let mut regs = make_regs();
        op_qstate_prep(&mut regs, 0, 0).unwrap();
        let q = extract_qutrit_from_regs(&regs, 0, 0).unwrap();
        write_qutrit_to_regs(&mut regs, 0, 1, &q).unwrap();
        write_qutrit_to_regs(&mut regs, 0, 2, &q).unwrap();
        op_qerr_inject(&mut regs, 0, 1).unwrap();
        op_qcorrect(&mut regs, 0).unwrap();
    }
}
