//! PlenumNET Ternary Hardware Description Language (THDL)
//!
//! A hardware description language for synthesizing ternary logic
//! circuits targeting FPGA and ASIC implementations.
//!
//! # Features
//! - Ternary gate primitives
//! - Timing constraint specification
//! - Multi-target synthesis (FPGA, ASIC)
//! - Optimization passes
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

pub mod ir;
pub mod optimizer;
pub mod synthesizer;
pub mod timing;

/// THDL version
pub const THDL_VERSION: &str = "0.1.0";

/// Supported synthesis targets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Xilinx FPGA family
    XilinxFpga,
    /// Intel/Altera FPGA family
    IntelFpga,
    /// Lattice FPGA family
    LatticeFpga,
    /// Custom ASIC
    Asic,
    /// Simulation only
    Simulation,
}

/// Synthesis options
#[derive(Debug, Clone)]
pub struct SynthesisOptions {
    pub target: Target,
    pub optimize_area: bool,
    pub optimize_speed: bool,
    pub optimize_power: bool,
    pub timing_constraints: TimingConstraints,
}

/// Timing constraints for synthesis
#[derive(Debug, Clone, Default)]
pub struct TimingConstraints {
    /// Maximum clock period in picoseconds
    pub max_clock_period_ps: u64,
    /// Setup time margin in picoseconds
    pub setup_margin_ps: u64,
    /// Hold time margin in picoseconds
    pub hold_margin_ps: u64,
}

impl Default for SynthesisOptions {
    fn default() -> Self {
        Self {
            target: Target::Simulation,
            optimize_area: true,
            optimize_speed: true,
            optimize_power: false,
            timing_constraints: TimingConstraints::default(),
        }
    }
}

/// Synthesize THDL to target
pub fn synthesize(thdl_source: &str, options: &SynthesisOptions) -> Result<SynthesisResult, SynthesisError> {
    // Parse THDL
    let ir = ir::parse(thdl_source)?;
    
    // Run optimization passes
    let optimized = optimizer::optimize(&ir, options)?;
    
    // Generate target-specific output
    let output = synthesizer::generate(&optimized, options)?;
    
    Ok(output)
}

/// Synthesis result
#[derive(Debug, Clone)]
pub struct SynthesisResult {
    pub output: String,
    pub statistics: SynthesisStats,
}

/// Synthesis statistics
#[derive(Debug, Clone, Default)]
pub struct SynthesisStats {
    pub trit_cells: usize,
    pub gates: usize,
    pub flip_flops: usize,
    pub estimated_area_um2: f64,
    pub estimated_power_mw: f64,
    pub critical_path_ps: u64,
}

/// Synthesis error
#[derive(Debug, Clone)]
pub enum SynthesisError {
    ParseError(String),
    OptimizationError(String),
    GenerationError(String),
    TimingViolation { required_ps: u64, actual_ps: u64 },
}
