// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// PUV v1.0 — UV Spectral Protocol — src/kernel/src/distributor/puv_spectral.rs
// Reference: TM-2026-026 v1.2
//
// Add `pub mod puv_spectral;` to distributor/mod.rs.

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Core constants (nm, exact integers from the axiom π = 14, radian = 13)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// The radian unit — gcd of all system wavelengths.
pub const RADIAN: u32 = 13;

/// π in the PlenumNET system.
pub const PI: u32 = 14;

/// Full circle in custom degrees.
pub const FULL_CIRCLE: u32 = 364;

/// Ionization threshold — quarter-turn = 7 × 13.
pub const LAMBDA_EUV: u32 = 91;

/// O₂ molecular absorption wall — half-turn = 14 × 13.
pub const LAMBDA_UVC: u32 = 182;

/// Ozone bridge — green arc effective = 22 × 13.
pub const LAMBDA_UVB: u32 = 286;

/// Full transmission — full circle = 28 × 13.
pub const LAMBDA_UVA: u32 = 364;

/// Far-UVC germicidal wavelength = 2 × center = 2 × 111.
pub const LAMBDA_FAR_UVC: u32 = 222;

/// XeCl excimer therapeutic = 4 × 7 × 11.
pub const LAMBDA_EXCIMER: u32 = 308;

/// Narrowband UVB therapeutic = e₂ = pq + pr + qr.
pub const LAMBDA_NB_UVB: u32 = 311;

/// Center constant of the unified equation = (182 + 40) / 2.
pub const CENTER: u32 = 111;

/// Hamiltonian cycle length = 7 × 11 × 13.
pub const HAMILTONIAN_LENGTH: u32 = 1001;

/// Coprime triple generating the spectral partition.
pub const COPRIME_TRIPLE: [u32; 3] = [7, 11, 13];

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Band boundaries — arithmetic means of adjacent system wavelengths
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// EUV|UVC boundary = (91 + 182) / 2 = 136.5 → 136 (floor).
pub const BOUNDARY_EUV_UVC: u32 = 136;

/// UVC|UVB boundary = (182 + 286) / 2 = 234.
pub const BOUNDARY_UVC_UVB: u32 = 234;

/// UVB|UVA boundary = (286 + 364) / 2 = 325.
pub const BOUNDARY_UVB_UVA: u32 = 325;

/// UV|Visible boundary.
pub const BOUNDARY_UV_VIS: u32 = 400;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Bias constants
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Hydrogen-specific offset: +0.194%. Applies to hydrogen-referenced measurements.
pub const VACUUM_BIAS_NUM: u32 = 194;
pub const VACUUM_BIAS_DEN: u32 = 100_000;

/// Universal offset: +0.139%. Applies to infinite-mass limit (R_∞).
pub const UNIVERSAL_BIAS_NUM: u32 = 139;
pub const UNIVERSAL_BIAS_DEN: u32 = 100_000;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// PuvBand — deterministic, gap-free, overlap-free UV band classification
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PuvBand {
    Euv,
    UvC,
    UvB,
    UvA,
    Visible,
}

impl PuvBand {
    pub fn classify(lambda_nm: u32) -> Self {
        if lambda_nm <= BOUNDARY_EUV_UVC {
            PuvBand::Euv
        } else if lambda_nm <= BOUNDARY_UVC_UVB {
            PuvBand::UvC
        } else if lambda_nm <= BOUNDARY_UVB_UVA {
            PuvBand::UvB
        } else if lambda_nm <= BOUNDARY_UV_VIS {
            PuvBand::UvA
        } else {
            PuvBand::Visible
        }
    }

    pub fn anchor(&self) -> Option<u32> {
        match self {
            PuvBand::Euv => Some(LAMBDA_EUV),
            PuvBand::UvC => Some(LAMBDA_UVC),
            PuvBand::UvB => Some(LAMBDA_UVB),
            PuvBand::UvA => Some(LAMBDA_UVA),
            PuvBand::Visible => None,
        }
    }

    pub fn transmission(&self) -> &'static str {
        match self {
            PuvBand::Euv => "0% — absorbed by atomic O, N in thermosphere",
            PuvBand::UvC => "0% — absorbed by O₂ Schumann-Runge continuum",
            PuvBand::UvB => "~0.4% — attenuated by O₃ Hartley band",
            PuvBand::UvA => "~80% — near-complete atmospheric passage",
            PuvBand::Visible => "~100% — full atmospheric transmission",
        }
    }

    pub fn custom_radians(&self) -> Option<u32> {
        self.anchor().map(|a| a / RADIAN)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Plenum ↔ Vacuum wavelength conversion
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Vacuum bias as ratio: VACUUM_BIAS_NUM / VACUUM_BIAS_DEN = 0.00194.
const BIAS_FACTOR_NUM: u64 = (VACUUM_BIAS_DEN + VACUUM_BIAS_NUM) as u64;
const BIAS_FACTOR_DEN: u64 = VACUUM_BIAS_DEN as u64;

/// Convert plenum-exact wavelength to vacuum-measured wavelength (integer-scaled ×100_000).
pub fn plenum_to_vacuum_scaled(lambda_plenum: u32) -> u64 {
    (lambda_plenum as u64) * BIAS_FACTOR_NUM
}

/// Convert plenum-exact wavelength (nm) to vacuum-measured wavelength (nm, f64).
pub fn plenum_to_vacuum(lambda_plenum: u32) -> u64 {
    let num = (lambda_plenum as u64) * BIAS_FACTOR_NUM;
    (num + BIAS_FACTOR_DEN / 2) / BIAS_FACTOR_DEN
}

/// Convert vacuum-measured wavelength (nm) to plenum-exact wavelength (nm, rounded).
pub fn vacuum_to_plenum(lambda_vacuum: u32) -> u32 {
    let num = (lambda_vacuum as u64) * BIAS_FACTOR_DEN;
    ((num + BIAS_FACTOR_NUM / 2) / BIAS_FACTOR_NUM) as u32
}

/// Classify a vacuum-measured wavelength by converting to plenum first.
pub fn classify_vacuum(lambda_vacuum: u32) -> PuvBand {
    PuvBand::classify(vacuum_to_plenum(lambda_vacuum))
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Measurement frame
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementFrame {
    Plenum,
    Vacuum,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// PUV spectral measurement packet
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub struct PuvMeasurement {
    pub lambda_nm: u32,
    pub irradiance: u64,
    pub band: PuvBand,
    pub frame: MeasurementFrame,
    pub timestamp_ns: u64,
    pub source_node: [u8; 7],
    pub integrity: [u8; 27],
}

impl PuvMeasurement {
    pub fn new(
        lambda_nm: u32,
        irradiance: u64,
        frame: MeasurementFrame,
        timestamp_ns: u64,
        source_node: [u8; 7],
    ) -> Self {
        let effective_lambda = match frame {
            MeasurementFrame::Plenum => lambda_nm,
            MeasurementFrame::Vacuum => vacuum_to_plenum(lambda_nm),
        };
        Self {
            lambda_nm,
            irradiance,
            band: PuvBand::classify(effective_lambda),
            frame,
            timestamp_ns,
            source_node,
            integrity: [0u8; 27],
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// PUV spectral response function
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub struct PuvSpectralResponse {
    pub device_node: [u8; 7],
    pub response_curve: alloc::vec::Vec<(u32, u32)>,
    pub active_bands: alloc::vec::Vec<PuvBand>,
    pub calibrated_at: u64,
    pub calibration_authority: [u8; 7],
}

impl PuvSpectralResponse {
    pub fn band_count(&self, band: PuvBand) -> usize {
        self.response_curve
            .iter()
            .filter(|(lambda, _)| PuvBand::classify(*lambda) == band)
            .count()
    }

    pub fn band_sensitivity(&self, band: PuvBand) -> u64 {
        self.response_curve
            .iter()
            .filter(|(lambda, _)| PuvBand::classify(*lambda) == band)
            .map(|(_, sensitivity)| *sensitivity as u64)
            .sum()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Compile-time helper
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 { let t = b; b = a % b; a = t; } a
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Compile-time assertions (zero runtime cost)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

const _: () = assert!(LAMBDA_EUV == 7 * RADIAN);
const _: () = assert!(LAMBDA_UVC == PI * RADIAN);
const _: () = assert!(LAMBDA_UVB == 22 * RADIAN);
const _: () = assert!(LAMBDA_UVA == 28 * RADIAN);
const _: () = assert!(LAMBDA_UVA == FULL_CIRCLE);
const _: () = assert!(LAMBDA_UVC == LAMBDA_EUV * 2);
const _: () = assert!(LAMBDA_UVA == LAMBDA_EUV * 4);
const _: () = assert!(gcd(LAMBDA_EUV, gcd(LAMBDA_UVC, gcd(LAMBDA_UVB, LAMBDA_UVA))) == RADIAN);

const _: () = assert!(LAMBDA_FAR_UVC == 2 * CENTER);
const _: () = assert!(LAMBDA_EXCIMER == 4 * 7 * 11);
const _: () = assert!(LAMBDA_NB_UVB == 7 * 11 + 7 * 13 + 11 * 13);

const _: () = assert!(CENTER == (LAMBDA_UVC + 40) / 2);
const _: () = assert!(HAMILTONIAN_LENGTH == 7 * 11 * 13);

const _: () = assert!(BOUNDARY_EUV_UVC == (LAMBDA_EUV + LAMBDA_UVC) / 2);
const _: () = assert!(BOUNDARY_UVC_UVB == (LAMBDA_UVC + LAMBDA_UVB) / 2);
const _: () = assert!(BOUNDARY_UVB_UVA == (LAMBDA_UVB + LAMBDA_UVA) / 2);

const _: () = assert!(LAMBDA_UVB * 7 == LAMBDA_EUV * 22);
const _: () = assert!(LAMBDA_UVB * 7 == LAMBDA_UVC * 11);
const _: () = assert!(LAMBDA_UVA * 11 == LAMBDA_UVB * 14);

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Runtime tests
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn quarter_turn() { assert_eq!(LAMBDA_EUV, 7 * 13); }
    #[test] fn half_turn() { assert_eq!(LAMBDA_UVC, 14 * 13); }
    #[test] fn green_arc() { assert_eq!(LAMBDA_UVB, 22 * 13); }
    #[test] fn full_circle() { assert_eq!(LAMBDA_UVA, 28 * 13); }
    #[test] fn gcd_all() { assert_eq!(gcd(91, gcd(182, gcd(286, 364))), 13); }

    #[test] fn ratio_uvc_euv() { assert_eq!(LAMBDA_UVC / LAMBDA_EUV, 2); }
    #[test] fn ratio_uvb_euv() { assert_eq!(LAMBDA_UVB * 7, LAMBDA_EUV * 22); }
    #[test] fn ratio_uva_euv() { assert_eq!(LAMBDA_UVA / LAMBDA_EUV, 4); }
    #[test] fn ratio_uvb_uvc() { assert_eq!(LAMBDA_UVB * 7, LAMBDA_UVC * 11); }
    #[test] fn ratio_uva_uvb() { assert_eq!(LAMBDA_UVA * 11, LAMBDA_UVB * 14); }

    #[test] fn secondary_222() { assert_eq!(LAMBDA_FAR_UVC, 2 * 111); }
    #[test] fn secondary_308() { assert_eq!(LAMBDA_EXCIMER, 4 * 77); }
    #[test] fn secondary_311() { assert_eq!(LAMBDA_NB_UVB, 77 + 91 + 143); }

    #[test] fn classify_euv() { assert_eq!(PuvBand::classify(91), PuvBand::Euv); }
    #[test] fn classify_uvc() { assert_eq!(PuvBand::classify(182), PuvBand::UvC); }
    #[test] fn classify_uvb() { assert_eq!(PuvBand::classify(286), PuvBand::UvB); }
    #[test] fn classify_uva() { assert_eq!(PuvBand::classify(364), PuvBand::UvA); }
    #[test] fn classify_visible() { assert_eq!(PuvBand::classify(500), PuvBand::Visible); }
    #[test] fn classify_boundary_euv() { assert_eq!(PuvBand::classify(136), PuvBand::Euv); }
    #[test] fn classify_boundary_uvc() { assert_eq!(PuvBand::classify(137), PuvBand::UvC); }
    #[test] fn classify_boundary_uvb() { assert_eq!(PuvBand::classify(235), PuvBand::UvB); }
    #[test] fn classify_boundary_uva() { assert_eq!(PuvBand::classify(326), PuvBand::UvA); }
    #[test] fn classify_boundary_vis() { assert_eq!(PuvBand::classify(401), PuvBand::Visible); }

    #[test] fn anchor_euv() { assert_eq!(PuvBand::Euv.anchor(), Some(91)); }
    #[test] fn anchor_uvc() { assert_eq!(PuvBand::UvC.anchor(), Some(182)); }
    #[test] fn anchor_uvb() { assert_eq!(PuvBand::UvB.anchor(), Some(286)); }
    #[test] fn anchor_uva() { assert_eq!(PuvBand::UvA.anchor(), Some(364)); }
    #[test] fn anchor_vis() { assert_eq!(PuvBand::Visible.anchor(), None); }

    #[test] fn radians_euv() { assert_eq!(PuvBand::Euv.custom_radians(), Some(7)); }
    #[test] fn radians_uvc() { assert_eq!(PuvBand::UvC.custom_radians(), Some(14)); }
    #[test] fn radians_uvb() { assert_eq!(PuvBand::UvB.custom_radians(), Some(22)); }
    #[test] fn radians_uva() { assert_eq!(PuvBand::UvA.custom_radians(), Some(28)); }

    #[test] fn plenum_to_vacuum() {
        let scaled = plenum_to_vacuum_scaled(91);
        assert_eq!(scaled, 91 * 100_194);
    }
    #[test] fn vacuum_roundtrip() {
        let scaled = plenum_to_vacuum_scaled(182);
        let back = vacuum_to_plenum(scaled);
        assert_eq!(back, 182);
    }

    #[test] fn measurement_plenum() {
        let m = PuvMeasurement::new(286, 1000, MeasurementFrame::Plenum, 0, [0u8; 7]);
        assert_eq!(m.band, PuvBand::UvB);
    }

    #[test] fn hamiltonian() { assert_eq!(7 * 11 * 13, HAMILTONIAN_LENGTH); }
    #[test] fn center_eq() { assert_eq!((182 + 40) / 2, CENTER); }

    #[test] fn e2_is_prime() {
        let n = LAMBDA_NB_UVB;
        let mut is_prime = n > 1;
        let mut d = 2u32;
        while d * d <= n { if n % d == 0 { is_prime = false; break; } d += 1; }
        assert!(is_prime);
    }
    #[test] fn e1_is_prime() {
        let n = 7 + 11 + 13;
        assert_eq!(n, 31);
        let mut is_prime = n > 1;
        let mut d = 2u32;
        while d * d <= n { if n % d == 0 { is_prime = false; break; } d += 1; }
        assert!(is_prime);
    }
}
