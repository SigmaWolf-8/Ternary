// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL - All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! FPGA Synthesis Specifications for Ternary Crypto Accelerator
//!
//! Defines hardware description language (HDL) specifications, resource
//! estimates, and synthesis targets for implementing the PlenumNET ternary
//! cryptographic primitives in FPGA fabric.
//!
//! # Architecture Overview
//!
//! The Ternary Crypto Accelerator (TCA) implements the following in hardware:
//! 1. **GF(3) ALU**: Native ternary arithmetic (add, multiply, rotate) 
//! 2. **Sponge Permutation Engine**: 729-trit state with 27-round pipeline
//! 3. **Polynomial Multiplier**: Schoolbook ring multiplication for TL-KEM/DSA
//! 4. **AES-256 Core**: Bitsliced S-box for constant-time operation
//! 5. **Noise Sampler**: CBD ternary noise generation
//!
//! # Target Platforms
//!
//! | Platform | Family | LUTs | BRAMs | DSPs | MHz Target |
//! |----------|--------|------|-------|------|------------|
//! | Xilinx | Artix-7 (XC7A200T) | 134,600 | 365 | 740 | 200 |
//! | Xilinx | Kintex UltraScale+ | 663,360 | 1,080 | 3,528 | 500 |
//! | Intel | Stratix 10 | 933,120 | 11,721 | 5,760 | 500 |
//! | Lattice | CrossLink-NX | 53,000 | 208 | 56 | 300 |
//!
//! # Copyright
//! Copyright (c) 2026 Capomastro Holdings Ltd. All rights reserved.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FpgaFamily {
    XilinxArtix7,
    XilinxKintexUltraScale,
    IntelStratix10,
    LatticeCrossLinkNx,
}

impl FpgaFamily {
    pub fn name(&self) -> &'static str {
        match self {
            FpgaFamily::XilinxArtix7 => "Xilinx Artix-7 (XC7A200T)",
            FpgaFamily::XilinxKintexUltraScale => "Xilinx Kintex UltraScale+",
            FpgaFamily::IntelStratix10 => "Intel Stratix 10",
            FpgaFamily::LatticeCrossLinkNx => "Lattice CrossLink-NX",
        }
    }

    pub fn available_luts(&self) -> u32 {
        match self {
            FpgaFamily::XilinxArtix7 => 134_600,
            FpgaFamily::XilinxKintexUltraScale => 663_360,
            FpgaFamily::IntelStratix10 => 933_120,
            FpgaFamily::LatticeCrossLinkNx => 53_000,
        }
    }

    pub fn available_brams(&self) -> u32 {
        match self {
            FpgaFamily::XilinxArtix7 => 365,
            FpgaFamily::XilinxKintexUltraScale => 1_080,
            FpgaFamily::IntelStratix10 => 11_721,
            FpgaFamily::LatticeCrossLinkNx => 208,
        }
    }

    pub fn available_dsps(&self) -> u32 {
        match self {
            FpgaFamily::XilinxArtix7 => 740,
            FpgaFamily::XilinxKintexUltraScale => 3_528,
            FpgaFamily::IntelStratix10 => 5_760,
            FpgaFamily::LatticeCrossLinkNx => 56,
        }
    }

    pub fn target_mhz(&self) -> u32 {
        match self {
            FpgaFamily::XilinxArtix7 => 200,
            FpgaFamily::XilinxKintexUltraScale => 500,
            FpgaFamily::IntelStratix10 => 500,
            FpgaFamily::LatticeCrossLinkNx => 300,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceleratorModule {
    Gf3Alu,
    SpongeEngine,
    PolyMultiplier,
    Aes256Core,
    NoiseSampler,
    TlKemPipeline,
    TlDsaPipeline,
    ControlUnit,
    HostInterface,
}

impl AcceleratorModule {
    pub fn name(&self) -> &'static str {
        match self {
            AcceleratorModule::Gf3Alu => "GF(3) Arithmetic Logic Unit",
            AcceleratorModule::SpongeEngine => "Sponge Permutation Engine",
            AcceleratorModule::PolyMultiplier => "Polynomial Ring Multiplier",
            AcceleratorModule::Aes256Core => "AES-256-GCM Core",
            AcceleratorModule::NoiseSampler => "CBD Noise Sampler",
            AcceleratorModule::TlKemPipeline => "TL-KEM Pipeline",
            AcceleratorModule::TlDsaPipeline => "TL-DSA Pipeline",
            AcceleratorModule::ControlUnit => "Control Unit & Scheduler",
            AcceleratorModule::HostInterface => "Host Interface (AXI4/PCIe)",
        }
    }

    pub fn hdl_entity(&self) -> &'static str {
        match self {
            AcceleratorModule::Gf3Alu => "gf3_alu",
            AcceleratorModule::SpongeEngine => "sponge_engine_729",
            AcceleratorModule::PolyMultiplier => "poly_ring_mul_256",
            AcceleratorModule::Aes256Core => "aes256_gcm_core",
            AcceleratorModule::NoiseSampler => "cbd_noise_gen",
            AcceleratorModule::TlKemPipeline => "tl_kem_top",
            AcceleratorModule::TlDsaPipeline => "tl_dsa_top",
            AcceleratorModule::ControlUnit => "tca_ctrl",
            AcceleratorModule::HostInterface => "axi4_host_if",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceEstimate {
    pub module: AcceleratorModule,
    pub luts: u32,
    pub flip_flops: u32,
    pub brams: u32,
    pub dsps: u32,
    pub latency_cycles: u32,
    pub throughput_ops_per_sec: u64,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct FpgaSynthTarget {
    pub family: FpgaFamily,
    pub module_estimates: Vec<ResourceEstimate>,
    pub total_luts: u32,
    pub total_ffs: u32,
    pub total_brams: u32,
    pub total_dsps: u32,
    pub lut_utilization_pct: f64,
    pub bram_utilization_pct: f64,
    pub dsp_utilization_pct: f64,
    pub fits_on_target: bool,
    pub estimated_power_watts: f64,
}

#[derive(Debug, Clone)]
pub struct PipelineSpec {
    pub name: String,
    pub stages: Vec<PipelineStage>,
    pub total_latency_cycles: u32,
    pub initiation_interval: u32,
    pub throughput_description: String,
}

#[derive(Debug, Clone)]
pub struct PipelineStage {
    pub stage_num: u32,
    pub name: String,
    pub cycles: u32,
    pub module: AcceleratorModule,
    pub description: String,
}

pub fn gf3_alu_estimate() -> ResourceEstimate {
    ResourceEstimate {
        module: AcceleratorModule::Gf3Alu,
        luts: 2_400,
        flip_flops: 1_800,
        brams: 0,
        dsps: 0,
        latency_cycles: 1,
        throughput_ops_per_sec: 200_000_000,
        notes: String::from(
            "Single-cycle GF(3) ALU supporting add, multiply, rotate, negate. \
             243-trit wide datapath (parallel processing of full hash-width vectors). \
             Uses 2-bit encoding per trit ({00=-1, 01=0, 10=+1}). \
             Mod-3 arithmetic via 2-bit adder trees with carry-less reduction."
        ),
    }
}

pub fn sponge_engine_estimate() -> ResourceEstimate {
    ResourceEstimate {
        module: AcceleratorModule::SpongeEngine,
        luts: 18_500,
        flip_flops: 14_600,
        brams: 2,
        dsps: 0,
        latency_cycles: 27,
        throughput_ops_per_sec: 7_400_000,
        notes: String::from(
            "27-round sponge permutation over 729-trit state (1,458 FFs for double-buffered state). \
             Each round: substitution (S-box LUT per 3-trit group), position permutation \
             (hardwired routing), round constant injection. Fully pipelined with \
             initiation interval of 1 round. BRAMs store round constants."
        ),
    }
}

pub fn poly_multiplier_estimate() -> ResourceEstimate {
    ResourceEstimate {
        module: AcceleratorModule::PolyMultiplier,
        luts: 32_000,
        flip_flops: 24_000,
        brams: 8,
        dsps: 64,
        latency_cycles: 256,
        throughput_ops_per_sec: 780_000,
        notes: String::from(
            "Schoolbook polynomial multiplication in R_q = Z_3[X]/(X^256+1). \
             256x256 coefficient multiply-accumulate with mod-3 reduction. \
             Uses DSP blocks for parallel 2-bit multiply-add. \
             BRAMs store polynomial coefficients (2 bits x 256 per poly). \
             Supports k=2,3,4 vector multiplications via scheduling."
        ),
    }
}

pub fn aes256_core_estimate() -> ResourceEstimate {
    ResourceEstimate {
        module: AcceleratorModule::Aes256Core,
        luts: 12_000,
        flip_flops: 8_500,
        brams: 4,
        dsps: 0,
        latency_cycles: 14,
        throughput_ops_per_sec: 14_280_000,
        notes: String::from(
            "AES-256 with bitsliced S-box (no lookup tables, constant-time). \
             14-round fully pipelined architecture. GCM mode with GF(2^128) multiplier \
             (Karatsuba). IV generation and tag computation integrated. \
             BRAMs for expanded key schedule storage."
        ),
    }
}

pub fn noise_sampler_estimate() -> ResourceEstimate {
    ResourceEstimate {
        module: AcceleratorModule::NoiseSampler,
        luts: 4_200,
        flip_flops: 3_100,
        brams: 1,
        dsps: 0,
        latency_cycles: 8,
        throughput_ops_per_sec: 25_000_000,
        notes: String::from(
            "Centered Binomial Distribution (CBD) ternary noise sampler. \
             Generates eta-bounded ternary noise vectors for TL-KEM and TL-DSA. \
             PRNG core (sponge-based) feeds CBD computation. \
             Produces 256 ternary coefficients per invocation."
        ),
    }
}

pub fn tl_kem_pipeline_estimate() -> ResourceEstimate {
    ResourceEstimate {
        module: AcceleratorModule::TlKemPipeline,
        luts: 8_000,
        flip_flops: 6_000,
        brams: 4,
        dsps: 0,
        latency_cycles: 1_200,
        throughput_ops_per_sec: 166_000,
        notes: String::from(
            "Full TL-KEM orchestration: keygen, encapsulate, decapsulate. \
             Coordinates GF3 ALU, poly multiplier, sponge engine, and noise sampler. \
             FO transform comparison with constant-time select (cmov). \
             State machine with implicit rejection path."
        ),
    }
}

pub fn tl_dsa_pipeline_estimate() -> ResourceEstimate {
    ResourceEstimate {
        module: AcceleratorModule::TlDsaPipeline,
        luts: 9_500,
        flip_flops: 7_200,
        brams: 6,
        dsps: 0,
        latency_cycles: 2_400,
        throughput_ops_per_sec: 83_000,
        notes: String::from(
            "Full TL-DSA orchestration: keygen, sign, verify. \
             Sign pipeline includes reject-and-retry loop (max 256 attempts). \
             L-infinity norm checker with constant-time evaluation. \
             Challenge sampling via sponge engine. \
             Worst-case latency assumes max rejections."
        ),
    }
}

pub fn control_unit_estimate() -> ResourceEstimate {
    ResourceEstimate {
        module: AcceleratorModule::ControlUnit,
        luts: 3_800,
        flip_flops: 2_900,
        brams: 1,
        dsps: 0,
        latency_cycles: 0,
        throughput_ops_per_sec: 0,
        notes: String::from(
            "Master control FSM scheduling crypto operations. \
             Command queue (BRAM-backed, 64 entries). \
             Operation dispatch, result collection, error handling. \
             Interrupt generation for host notification."
        ),
    }
}

pub fn host_interface_estimate() -> ResourceEstimate {
    ResourceEstimate {
        module: AcceleratorModule::HostInterface,
        luts: 5_600,
        flip_flops: 4_200,
        brams: 2,
        dsps: 0,
        latency_cycles: 4,
        throughput_ops_per_sec: 0,
        notes: String::from(
            "AXI4-Lite control registers + AXI4-Stream data path. \
             DMA engine for bulk key/ciphertext transfer. \
             Memory-mapped register file for configuration and status. \
             Optional PCIe Gen3 x4 endpoint (for server deployments)."
        ),
    }
}

pub fn generate_synth_target(family: FpgaFamily) -> FpgaSynthTarget {
    let estimates = vec![
        gf3_alu_estimate(),
        sponge_engine_estimate(),
        poly_multiplier_estimate(),
        aes256_core_estimate(),
        noise_sampler_estimate(),
        tl_kem_pipeline_estimate(),
        tl_dsa_pipeline_estimate(),
        control_unit_estimate(),
        host_interface_estimate(),
    ];

    let total_luts: u32 = estimates.iter().map(|e| e.luts).sum();
    let total_ffs: u32 = estimates.iter().map(|e| e.flip_flops).sum();
    let total_brams: u32 = estimates.iter().map(|e| e.brams).sum();
    let total_dsps: u32 = estimates.iter().map(|e| e.dsps).sum();

    let lut_pct = (total_luts as f64 / family.available_luts() as f64) * 100.0;
    let bram_pct = (total_brams as f64 / family.available_brams() as f64) * 100.0;
    let dsp_pct = if family.available_dsps() > 0 {
        (total_dsps as f64 / family.available_dsps() as f64) * 100.0
    } else {
        0.0
    };

    let fits = lut_pct < 90.0 && bram_pct < 90.0 && dsp_pct < 90.0;

    let power = match family {
        FpgaFamily::XilinxArtix7 => 2.8,
        FpgaFamily::XilinxKintexUltraScale => 4.5,
        FpgaFamily::IntelStratix10 => 6.2,
        FpgaFamily::LatticeCrossLinkNx => 1.5,
    };

    FpgaSynthTarget {
        family,
        module_estimates: estimates,
        total_luts,
        total_ffs,
        total_brams,
        total_dsps,
        lut_utilization_pct: lut_pct,
        bram_utilization_pct: bram_pct,
        dsp_utilization_pct: dsp_pct,
        fits_on_target: fits,
        estimated_power_watts: power,
    }
}

pub fn tl_kem_pipeline_spec() -> PipelineSpec {
    PipelineSpec {
        name: String::from("TL-KEM Full Pipeline"),
        stages: vec![
            PipelineStage {
                stage_num: 1,
                name: String::from("Seed Expansion"),
                cycles: 27,
                module: AcceleratorModule::SpongeEngine,
                description: String::from("Expand seed to (rho, sigma) via sponge hash"),
            },
            PipelineStage {
                stage_num: 2,
                name: String::from("Matrix Generation"),
                cycles: 256,
                module: AcceleratorModule::PolyMultiplier,
                description: String::from("Generate matrix A from rho (k*k polynomials)"),
            },
            PipelineStage {
                stage_num: 3,
                name: String::from("Noise Sampling"),
                cycles: 32,
                module: AcceleratorModule::NoiseSampler,
                description: String::from("Sample secret s and error e vectors (CBD)"),
            },
            PipelineStage {
                stage_num: 4,
                name: String::from("Matrix-Vector Multiply"),
                cycles: 512,
                module: AcceleratorModule::PolyMultiplier,
                description: String::from("Compute t = As + e (k ring multiplications)"),
            },
            PipelineStage {
                stage_num: 5,
                name: String::from("Public Key Hash"),
                cycles: 27,
                module: AcceleratorModule::SpongeEngine,
                description: String::from("Hash public key for FO transform"),
            },
            PipelineStage {
                stage_num: 6,
                name: String::from("Encapsulation"),
                cycles: 300,
                module: AcceleratorModule::TlKemPipeline,
                description: String::from("Generate ciphertext and derive shared secret"),
            },
            PipelineStage {
                stage_num: 7,
                name: String::from("Compression"),
                cycles: 46,
                module: AcceleratorModule::Gf3Alu,
                description: String::from("Compress u and v for ciphertext output"),
            },
        ],
        total_latency_cycles: 1_200,
        initiation_interval: 1_200,
        throughput_description: String::from("1 full KEM operation per 1,200 cycles (166K ops/sec at 200 MHz)"),
    }
}

pub fn tl_dsa_pipeline_spec() -> PipelineSpec {
    PipelineSpec {
        name: String::from("TL-DSA Signing Pipeline"),
        stages: vec![
            PipelineStage {
                stage_num: 1,
                name: String::from("Key Material Load"),
                cycles: 16,
                module: AcceleratorModule::HostInterface,
                description: String::from("Load secret key and matrix seed from host memory"),
            },
            PipelineStage {
                stage_num: 2,
                name: String::from("Matrix Regeneration"),
                cycles: 256,
                module: AcceleratorModule::PolyMultiplier,
                description: String::from("Regenerate matrix A from seed (cached after first use)"),
            },
            PipelineStage {
                stage_num: 3,
                name: String::from("Message Hashing"),
                cycles: 54,
                module: AcceleratorModule::SpongeEngine,
                description: String::from("Compute mu = H(pk_hash || message)"),
            },
            PipelineStage {
                stage_num: 4,
                name: String::from("Masking Vector Sample"),
                cycles: 40,
                module: AcceleratorModule::NoiseSampler,
                description: String::from("Sample masking vector y from deterministic seed"),
            },
            PipelineStage {
                stage_num: 5,
                name: String::from("Commitment Compute"),
                cycles: 512,
                module: AcceleratorModule::PolyMultiplier,
                description: String::from("Compute w = Ay (matrix-vector multiply)"),
            },
            PipelineStage {
                stage_num: 6,
                name: String::from("Challenge Hash"),
                cycles: 27,
                module: AcceleratorModule::SpongeEngine,
                description: String::from("Compute challenge c = H(mu || w)"),
            },
            PipelineStage {
                stage_num: 7,
                name: String::from("Response Compute"),
                cycles: 256,
                module: AcceleratorModule::PolyMultiplier,
                description: String::from("Compute z = y + c*s1 with norm check"),
            },
            PipelineStage {
                stage_num: 8,
                name: String::from("Rejection Check"),
                cycles: 8,
                module: AcceleratorModule::Gf3Alu,
                description: String::from("Check ||z||_inf <= gamma, retry if rejected"),
            },
        ],
        total_latency_cycles: 1_169,
        initiation_interval: 1_169,
        throughput_description: String::from(
            "1 signing attempt per 1,169 cycles. Average ~2 attempts per valid signature. \
             ~85K valid signatures/sec at 200 MHz."
        ),
    }
}

pub fn generate_all_targets() -> Vec<FpgaSynthTarget> {
    vec![
        generate_synth_target(FpgaFamily::XilinxArtix7),
        generate_synth_target(FpgaFamily::XilinxKintexUltraScale),
        generate_synth_target(FpgaFamily::IntelStratix10),
        generate_synth_target(FpgaFamily::LatticeCrossLinkNx),
    ]
}

pub fn recommended_target() -> FpgaFamily {
    FpgaFamily::XilinxKintexUltraScale
}

pub fn synthesis_summary() -> SynthesisSummary {
    let targets = generate_all_targets();
    let fitting: Vec<_> = targets.iter().filter(|t| t.fits_on_target).collect();

    SynthesisSummary {
        total_modules: 9,
        total_targets: targets.len(),
        fitting_targets: fitting.len(),
        recommended: recommended_target(),
        total_estimated_luts: targets.first().map(|t| t.total_luts).unwrap_or(0),
        total_estimated_brams: targets.first().map(|t| t.total_brams).unwrap_or(0),
        total_estimated_dsps: targets.first().map(|t| t.total_dsps).unwrap_or(0),
        kem_pipeline_latency: 1_200,
        dsa_pipeline_latency: 1_169,
    }
}

#[derive(Debug, Clone)]
pub struct SynthesisSummary {
    pub total_modules: usize,
    pub total_targets: usize,
    pub fitting_targets: usize,
    pub recommended: FpgaFamily,
    pub total_estimated_luts: u32,
    pub total_estimated_brams: u32,
    pub total_estimated_dsps: u32,
    pub kem_pipeline_latency: u32,
    pub dsa_pipeline_latency: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_targets_generate() {
        let targets = generate_all_targets();
        assert_eq!(targets.len(), 4);
    }

    #[test]
    fn test_artix7_fits() {
        let target = generate_synth_target(FpgaFamily::XilinxArtix7);
        assert!(target.lut_utilization_pct < 100.0, "Design should fit on Artix-7");
    }

    #[test]
    fn test_kintex_fits() {
        let target = generate_synth_target(FpgaFamily::XilinxKintexUltraScale);
        assert!(target.fits_on_target, "Design should easily fit on Kintex UltraScale+");
        assert!(target.lut_utilization_pct < 30.0, "Kintex should have plenty of headroom");
    }

    #[test]
    fn test_stratix_fits() {
        let target = generate_synth_target(FpgaFamily::IntelStratix10);
        assert!(target.fits_on_target, "Design should easily fit on Stratix 10");
    }

    #[test]
    fn test_lattice_resource_check() {
        let target = generate_synth_target(FpgaFamily::LatticeCrossLinkNx);
        assert!(target.lut_utilization_pct > 50.0, "CrossLink-NX should be tight fit");
    }

    #[test]
    fn test_total_resources_reasonable() {
        let target = generate_synth_target(FpgaFamily::XilinxArtix7);
        assert!(target.total_luts > 50_000, "Total LUTs should be substantial");
        assert!(target.total_luts < 200_000, "Total LUTs should be reasonable");
        assert!(target.total_brams > 0);
        assert!(target.total_dsps > 0);
    }

    #[test]
    fn test_module_estimates_count() {
        let target = generate_synth_target(FpgaFamily::XilinxArtix7);
        assert_eq!(target.module_estimates.len(), 9);
    }

    #[test]
    fn test_kem_pipeline_spec() {
        let spec = tl_kem_pipeline_spec();
        assert_eq!(spec.stages.len(), 7);
        assert_eq!(spec.total_latency_cycles, 1_200);
    }

    #[test]
    fn test_dsa_pipeline_spec() {
        let spec = tl_dsa_pipeline_spec();
        assert_eq!(spec.stages.len(), 8);
        assert_eq!(spec.total_latency_cycles, 1_169);
    }

    #[test]
    fn test_fpga_family_names() {
        assert_eq!(FpgaFamily::XilinxArtix7.name(), "Xilinx Artix-7 (XC7A200T)");
        assert_eq!(FpgaFamily::IntelStratix10.name(), "Intel Stratix 10");
    }

    #[test]
    fn test_module_hdl_entities() {
        assert_eq!(AcceleratorModule::Gf3Alu.hdl_entity(), "gf3_alu");
        assert_eq!(AcceleratorModule::SpongeEngine.hdl_entity(), "sponge_engine_729");
        assert_eq!(AcceleratorModule::PolyMultiplier.hdl_entity(), "poly_ring_mul_256");
    }

    #[test]
    fn test_synthesis_summary() {
        let summary = synthesis_summary();
        assert_eq!(summary.total_modules, 9);
        assert_eq!(summary.total_targets, 4);
        assert!(summary.fitting_targets >= 2);
        assert_eq!(summary.recommended, FpgaFamily::XilinxKintexUltraScale);
    }

    #[test]
    fn test_power_estimates() {
        for family in [FpgaFamily::XilinxArtix7, FpgaFamily::XilinxKintexUltraScale,
                       FpgaFamily::IntelStratix10, FpgaFamily::LatticeCrossLinkNx] {
            let target = generate_synth_target(family);
            assert!(target.estimated_power_watts > 0.0);
            assert!(target.estimated_power_watts < 10.0);
        }
    }
}
