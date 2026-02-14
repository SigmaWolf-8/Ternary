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

//! # Radix Economy — Quantifying Ternary Efficiency
//!
//! Provides functions to compute, compare, and benchmark the information-theoretic
//! efficiency of ternary vs binary representation.
//!
//! ## Key result
//!
//! Base 3 is the most efficient integer radix. The cost per unit of information
//! is b/ln(b), minimized at b = e ≈ 2.718. Since 3 is the closest integer to e,
//! ternary achieves ~5.7% better radix economy than binary.
//!
//! This module quantifies that advantage concretely for PlenumNET workloads:
//! opcode encoding, address spaces, and arithmetic operations.

use std::fmt;

/// Radix economy for representing integer N in base b.
/// E(b, N) = b × ⌈log_b(N)⌉
///
/// Returns None if N < 1 or b < 2.
pub fn radix_economy(base: u64, n: u64) -> Option<u64> {
    if n < 1 || base < 2 {
        return None;
    }
    if n == 1 {
        return Some(base); // 1 digit needed
    }
    // ⌈log_b(N)⌉ = number of digits
    let digits = digits_needed(base, n);
    Some(base * digits)
}

/// Number of digits needed to represent N in base b.
/// ⌈log_b(N)⌉, but computed with integer arithmetic to avoid float errors.
pub fn digits_needed(base: u64, n: u64) -> u64 {
    if n == 0 {
        return 1;
    }
    let mut digits = 0u64;
    let mut capacity = 1u64; // b^digits
    while capacity <= n {
        match capacity.checked_mul(base) {
            Some(next) => capacity = next,
            None => {
                digits += 1;
                break;
            }
        }
        digits += 1;
        if capacity > n {
            break;
        }
    }
    digits
}

/// Asymptotic efficiency: b / ln(b).
/// Lower is better. This is the cost per unit of information for large N.
pub fn asymptotic_cost(base: f64) -> f64 {
    base / base.ln()
}

/// Ratio of binary cost to ternary cost (asymptotic).
/// Values > 1.0 mean ternary is more efficient.
pub fn ternary_advantage_ratio() -> f64 {
    asymptotic_cost(2.0) / asymptotic_cost(3.0)
}

/// Information content per symbol in base b, measured in bits.
/// Each symbol carries log2(b) bits of information.
pub fn bits_per_symbol(base: f64) -> f64 {
    base.log2()
}

/// Compare radix economy across bases for a specific value.
pub fn compare_economies(n: u64) -> RadixComparison {
    RadixComparison {
        n,
        binary: radix_economy(2, n).unwrap_or(0),
        ternary: radix_economy(3, n).unwrap_or(0),
        quaternary: radix_economy(4, n).unwrap_or(0),
        decimal: radix_economy(10, n).unwrap_or(0),
        binary_digits: digits_needed(2, n),
        ternary_digits: digits_needed(3, n),
    }
}

/// Results of a radix economy comparison.
#[derive(Debug, Clone)]
pub struct RadixComparison {
    pub n: u64,
    pub binary: u64,
    pub ternary: u64,
    pub quaternary: u64,
    pub decimal: u64,
    pub binary_digits: u64,
    pub ternary_digits: u64,
}

impl RadixComparison {
    /// Ternary savings as a fraction (0.0 = equal, positive = ternary wins).
    pub fn ternary_savings_fraction(&self) -> f64 {
        if self.binary == 0 {
            return 0.0;
        }
        1.0 - (self.ternary as f64 / self.binary as f64)
    }

    /// Information density: bits carried per ternary symbol vs binary symbol.
    pub fn ternary_info_density_ratio(&self) -> f64 {
        // A trit carries log2(3) ≈ 1.585 bits. A bit carries 1 bit.
        // Per trit vs per bit: 1.585 / 1 = 1.585
        // But we use fewer trits than bits, so total info per digit count:
        if self.ternary_digits == 0 {
            return 0.0;
        }
        let ternary_bits = self.ternary_digits as f64 * 3.0f64.log2();
        let binary_bits = self.binary_digits as f64;
        ternary_bits / binary_bits
    }
}

impl fmt::Display for RadixComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Radix Economy for N = {}", self.n)?;
        writeln!(f, "  Binary  (base 2):  E = {} ({} digits)", self.binary, self.binary_digits)?;
        writeln!(f, "  Ternary (base 3):  E = {} ({} digits)", self.ternary, self.ternary_digits)?;
        writeln!(f, "  Quaternary (base 4): E = {}", self.quaternary)?;
        writeln!(f, "  Decimal (base 10):   E = {}", self.decimal)?;
        writeln!(f, "  Ternary savings: {:.1}%", self.ternary_savings_fraction() * 100.0)?;
        Ok(())
    }
}

// -- PlenumNET-specific benchmarks --------------------------------------------

/// Benchmark: opcode encoding efficiency for a given ISA size.
pub fn opcode_encoding_efficiency(num_opcodes: u64) -> OpcodeEfficiency {
    let binary_bits = digits_needed(2, num_opcodes);
    let ternary_trits = digits_needed(3, num_opcodes);
    let ternary_info_bits = ternary_trits as f64 * 3.0f64.log2();

    OpcodeEfficiency {
        num_opcodes,
        binary_bits,
        ternary_trits,
        binary_wasted_fraction: 1.0 - (num_opcodes as f64 / 2.0f64.powi(i32::try_from(binary_bits).unwrap())),
        ternary_wasted_fraction: 1.0 - (num_opcodes as f64 / 3.0f64.powi(i32::try_from(ternary_trits).unwrap())),
        ternary_info_bits,
        density_ratio: ternary_info_bits / binary_bits as f64,
    }
}

#[derive(Debug, Clone)]
pub struct OpcodeEfficiency {
    pub num_opcodes: u64,
    pub binary_bits: u64,
    pub ternary_trits: u64,
    pub binary_wasted_fraction: f64,
    pub ternary_wasted_fraction: f64,
    pub ternary_info_bits: f64,
    pub density_ratio: f64,
}

impl fmt::Display for OpcodeEfficiency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Opcode Encoding: {} opcodes", self.num_opcodes)?;
        writeln!(f, "  Binary:  {} bits  (waste: {:.1}%)", self.binary_bits, self.binary_wasted_fraction * 100.0)?;
        writeln!(f, "  Ternary: {} trits (waste: {:.1}%)", self.ternary_trits, self.ternary_wasted_fraction * 100.0)?;
        writeln!(f, "  Ternary carries {:.2} bits of info ({:.1}% vs binary)",
            self.ternary_info_bits, self.density_ratio * 100.0)?;
        Ok(())
    }
}

/// Benchmark: address space efficiency for a given node count.
pub fn address_space_efficiency(node_count: u64) -> AddressEfficiency {
    let binary_bits = digits_needed(2, node_count);
    let ternary_trits = digits_needed(3, node_count);
    let binary_capacity = 2u64.checked_pow(u32::try_from(binary_bits).unwrap()).unwrap_or(u64::MAX);
    let ternary_capacity = 3u64.checked_pow(u32::try_from(ternary_trits).unwrap()).unwrap_or(u64::MAX);

    AddressEfficiency {
        node_count,
        binary_bits,
        ternary_trits,
        binary_capacity,
        ternary_capacity,
        binary_utilization: node_count as f64 / binary_capacity as f64,
        ternary_utilization: node_count as f64 / ternary_capacity as f64,
    }
}

#[derive(Debug, Clone)]
pub struct AddressEfficiency {
    pub node_count: u64,
    pub binary_bits: u64,
    pub ternary_trits: u64,
    pub binary_capacity: u64,
    pub ternary_capacity: u64,
    pub binary_utilization: f64,
    pub ternary_utilization: f64,
}

impl fmt::Display for AddressEfficiency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Address Space: {} nodes", self.node_count)?;
        writeln!(f, "  Binary:  {} bits  → capacity {} (utilization {:.1}%)",
            self.binary_bits, self.binary_capacity, self.binary_utilization * 100.0)?;
        writeln!(f, "  Ternary: {} trits → capacity {} (utilization {:.1}%)",
            self.ternary_trits, self.ternary_capacity, self.ternary_utilization * 100.0)?;
        Ok(())
    }
}

/// Run the full PlenumNET benchmark suite and return a summary.
pub fn full_benchmark_report() -> String {
    let mut report = String::new();

    report.push_str("═══════════════════════════════════════════════════════\n");
    report.push_str("  TERNARY RADIX ECONOMY BENCHMARK — PlenumNET\n");
    report.push_str("═══════════════════════════════════════════════════════\n\n");

    // Asymptotic efficiency
    report.push_str(&format!(
        "Asymptotic cost (b/ln(b)):\n  Base 2: {:.4}\n  Base 3: {:.4}\n  Base e: {:.4}\n  Ternary advantage: {:.2}%\n\n",
        asymptotic_cost(2.0),
        asymptotic_cost(3.0),
        std::f64::consts::E,
        (ternary_advantage_ratio() - 1.0) * 100.0,
    ));

    // Opcode encoding for 55-opcode ISA
    report.push_str(&format!("{}\n", opcode_encoding_efficiency(55)));

    // Address spaces
    for &nodes in &[27, 81, 243, 729, 2187, 6561] {
        report.push_str(&format!("{}\n", address_space_efficiency(nodes)));
    }

    // Economy comparison for representative values
    report.push_str("Radix Economy Comparisons:\n");
    for &n in &[10, 100, 1000, 10000, 1_000_000] {
        let cmp = compare_economies(n);
        report.push_str(&format!("  N={:<10} Binary: {:<6} Ternary: {:<6} Savings: {:.1}%\n",
            n, cmp.binary, cmp.ternary, cmp.ternary_savings_fraction() * 100.0));
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_needed_basic() {
        assert_eq!(digits_needed(2, 1), 1);  // 1 in binary: "1" (1 digit)
        assert_eq!(digits_needed(2, 2), 2);  // 2 in binary: "10"
        assert_eq!(digits_needed(2, 7), 3);  // 7 in binary: "111"
        assert_eq!(digits_needed(2, 8), 4);  // 8 in binary: "1000"
        assert_eq!(digits_needed(3, 1), 1);
        assert_eq!(digits_needed(3, 2), 1);
        assert_eq!(digits_needed(3, 3), 2);  // 3 in ternary: "10"
        assert_eq!(digits_needed(3, 8), 2);  // 8 in ternary: "22"
        assert_eq!(digits_needed(3, 9), 3);  // 9 in ternary: "100"
        assert_eq!(digits_needed(3, 26), 3); // 26 in ternary: "222"
        assert_eq!(digits_needed(3, 27), 4); // 27 in ternary: "1000"
    }

    #[test]
    fn ternary_is_optimal_integer_radix() {
        let cost_2 = asymptotic_cost(2.0);
        let cost_3 = asymptotic_cost(3.0);
        let cost_4 = asymptotic_cost(4.0);
        let cost_5 = asymptotic_cost(5.0);

        assert!(cost_3 < cost_2, "Base 3 should beat base 2");
        assert!(cost_3 < cost_4, "Base 3 should beat base 4");
        assert!(cost_3 < cost_5, "Base 3 should beat base 5");
    }

    #[test]
    fn ternary_advantage_is_about_five_percent() {
        let ratio = ternary_advantage_ratio();
        // Should be approximately 1.057
        assert!(ratio > 1.05, "Advantage ratio too low: {ratio}");
        assert!(ratio < 1.07, "Advantage ratio too high: {ratio}");
    }

    #[test]
    fn opcode_35_isa() {
        let eff = opcode_encoding_efficiency(35);
        assert_eq!(eff.binary_bits, 6);  // ⌈log₂(35)⌉ = 6
        assert_eq!(eff.ternary_trits, 4); // ⌈log₃(35)⌉ = 4 (3⁴ = 81 > 35)
    }

    #[test]
    fn full_report_runs() {
        let report = full_benchmark_report();
        assert!(!report.is_empty());
        println!("{report}");
    }
}
