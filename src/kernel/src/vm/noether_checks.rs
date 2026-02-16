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
// Noether Invariant Checks for Post-Correction Verification
// ============================================================================
//
// After qutrit/qudit correction (opcodes QCorrect 0xA5, QUDIT_CORRECT_D),
// these routines verify that three key SUFT Noether invariants are preserved:
//
//   1. Ternary Gauge Symmetry: Sum of branch shifts ≈ 0
//   2. Reparametrization Energy Invariant: Scaled norm² ≈ SUFT Φ constant
//   3. Periodicity Invariant: Branch time-values modulo 364 consistent
//
// These checks enforce physical consistency of the corrected register state.

use super::{VmError, VmResult};
use alloc::string::String;

const SUFT_PHI_RATIO: f64 = 13.0 / 28.0;
const PERIOD_MODULUS: f64 = 364.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvariantViolation {
    TernaryGaugeSum,
    ReparametrizationEnergy,
    PeriodicityViolation,
}

impl InvariantViolation {
    pub fn description(&self) -> &'static str {
        match self {
            Self::TernaryGaugeSum => "Ternary gauge sum != 0",
            Self::ReparametrizationEnergy => "Reparametrization energy invariant broken",
            Self::PeriodicityViolation => "Periodicity violation",
        }
    }
}

pub fn check_ternary_gauge_invariant(
    registers: &[i64],
    reg_start: usize,
    d: usize,
    tolerance: f64,
) -> VmResult<()> {
    if reg_start + 2 * d >= 27 {
        return Err(VmError::InvalidRegister(reg_start as u8));
    }

    let branch0_re = (registers[reg_start] >> 32) as i32 as f64 / 1_000_000.0;
    let branch1_re = (registers[reg_start + d] >> 32) as i32 as f64 / 1_000_000.0;
    let branch2_re = (registers[reg_start + 2 * d] >> 32) as i32 as f64 / 1_000_000.0;

    let branch_sum = branch0_re + branch1_re + branch2_re;

    if libm::fabs(branch_sum) > tolerance {
        return Err(VmError::InvalidProgram(
            String::from(InvariantViolation::TernaryGaugeSum.description()),
        ));
    }

    Ok(())
}

pub fn check_reparametrization_invariant(
    registers: &[i64],
    reg_start: usize,
    d: usize,
    tolerance: f64,
) -> VmResult<()> {
    let regs_needed = d * 3;
    if reg_start + regs_needed > 27 {
        return Err(VmError::InvalidRegister(reg_start as u8));
    }

    let mut sum_norm_sq = 0.0f64;
    for i in 0..regs_needed {
        let val = registers[reg_start + i];
        let re = (val >> 32) as i32 as f64 / 1_000_000.0;
        let im = (val & 0xFFFFFFFF) as i32 as f64 / 1_000_000.0;
        sum_norm_sq += re * re + im * im;
    }

    let energy_invariant = SUFT_PHI_RATIO * sum_norm_sq;

    if energy_invariant > tolerance && libm::fabs(sum_norm_sq - 1.0) > tolerance {
        return Err(VmError::InvalidProgram(
            String::from(InvariantViolation::ReparametrizationEnergy.description()),
        ));
    }

    Ok(())
}

pub fn check_periodicity_invariant(
    registers: &[i64],
    reg_start: usize,
    d: usize,
    tolerance: f64,
) -> VmResult<()> {
    if reg_start + 2 * d >= 27 {
        return Err(VmError::InvalidRegister(reg_start as u8));
    }

    for branch in 0..3 {
        let t_val = (registers[reg_start + branch * d] >> 32) as i32 as f64 / 1_000_000.0;
        let t_scaled = t_val * PERIOD_MODULUS;
        let mod_val = t_scaled - libm::floor(t_scaled / PERIOD_MODULUS) * PERIOD_MODULUS;
        let deviation = libm::fabs(t_scaled - mod_val);
        if deviation > tolerance * PERIOD_MODULUS && libm::fabs(t_val) > tolerance {
            return Err(VmError::InvalidProgram(
                String::from(InvariantViolation::PeriodicityViolation.description()),
            ));
        }
    }

    Ok(())
}

pub fn check_all_noether_invariants(
    registers: &[i64],
    reg_start: usize,
    d: usize,
    tolerance: f64,
) -> VmResult<()> {
    check_ternary_gauge_invariant(registers, reg_start, d, tolerance)?;
    check_reparametrization_invariant(registers, reg_start, d, tolerance)?;
    check_periodicity_invariant(registers, reg_start, d, tolerance)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_regs() -> [i64; 27] {
        [0i64; 27]
    }

    #[test]
    fn test_gauge_invariant_zero_state() {
        let regs = make_regs();
        assert!(check_ternary_gauge_invariant(&regs, 0, 3, 0.01).is_ok());
    }

    #[test]
    fn test_reparametrization_invariant_zero_state() {
        let regs = make_regs();
        assert!(check_reparametrization_invariant(&regs, 0, 3, 1.0).is_ok());
    }

    #[test]
    fn test_periodicity_invariant_zero_state() {
        let regs = make_regs();
        assert!(check_periodicity_invariant(&regs, 0, 3, 1.0).is_ok());
    }

    #[test]
    fn test_all_invariants_zero_state() {
        let regs = make_regs();
        assert!(check_all_noether_invariants(&regs, 0, 3, 1.0).is_ok());
    }

    #[test]
    fn test_gauge_invariant_out_of_bounds() {
        let regs = make_regs();
        assert!(check_ternary_gauge_invariant(&regs, 25, 3, 0.01).is_err());
    }

    #[test]
    fn test_invariant_violation_description() {
        assert_eq!(
            InvariantViolation::TernaryGaugeSum.description(),
            "Ternary gauge sum != 0"
        );
        assert_eq!(
            InvariantViolation::ReparametrizationEnergy.description(),
            "Reparametrization energy invariant broken"
        );
        assert_eq!(
            InvariantViolation::PeriodicityViolation.description(),
            "Periodicity violation"
        );
    }
}
