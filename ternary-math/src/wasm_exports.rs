// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// WASM exports — binds the Rust kernel to JavaScript via wasm-bindgen.
//
// §A  TL-Sponge-385 operations (server/crypto/sponge-wasm-bridge.ts)
//     TLSponge-385: 9 full rounds, 729-trit state (54-trit rate, 675-trit capacity)
//     TIS-27: 4-round fast-path for scan hash / identity / HMAC
// §B  Unified constants from constants.rs — single source of truth
//
// Build: see Makefile target `make wasm` (wasm-pack >= 0.13.1 required)
// Output: pkg/ternary_math.js, pkg/ternary_math_bg.wasm, pkg/ternary_math.d.ts
//
// PROTOCOL NOTE — sponge_duplex_encrypt:
//   This is a LOW-LEVEL sponge duplex primitive, NOT the full T-AE-MAC
//   authenticated encryption protocol. T-AE-MAC (which enforces INVARIANT 9
//   Rep C address binding, v1/v2/v3 version dispatch, and auto-gated rayon
//   parallelism) must be implemented in the caller layer. The TypeScript
//   bridge (server/crypto/sponge-wasm-bridge.ts) is responsible for:
//     1. Binding Rep C addresses into the domain parameter before calling
//     2. Using a canonical context string as the domain parameter
//     3. Enforcing INVARIANT 9 (Rep C address inclusion in associated data)
//   See phase_encryption.rs for the full T-AE-MAC implementation.

use wasm_bindgen::prelude::*;
use zeroize::Zeroize;
use crate::sponge;
use crate::constants;

// ═══════════════════════════════════════════════════════════════════════
// SAFETY BOUNDS
// ═══════════════════════════════════════════════════════════════════════

const MAX_TRIT_OUTPUT: usize = 8192;
const MAX_BYTE_INPUT: usize = 65536;

const CANONICAL_CONTEXTS: &[&[u8]] = &[
    b"PlenumNET-Phase-v2",
    b"PlenumNET-KEM-v1",
    b"PlenumNET-DSA-v1",
    b"PlenumNET-TIS27-v1",
    b"PlenumNET-Duplex-v1",
];

fn validate_len(len: usize, max: usize, _name: &str) -> Result<(), &'static str> {
    if len > max { Err("exceeds maximum") } else { Ok(()) }
}

fn validate_context(domain: &[u8]) -> Result<(), &'static str> {
    if CANONICAL_CONTEXTS.iter().any(|c| *c == domain) {
        Ok(())
    } else {
        Err("domain must be one of: PlenumNET-Phase-v2, PlenumNET-KEM-v1, \
             PlenumNET-DSA-v1, PlenumNET-TIS27-v1, PlenumNET-Duplex-v1")
    }
}

fn check_len(len: usize, max: usize, name: &str) -> Result<(), JsValue> {
    validate_len(len, max, name).map_err(|e| JsValue::from_str(&format!("{} {} ({} > {})", name, e, len, max)))
}

fn check_context(domain: &[u8]) -> Result<(), JsValue> {
    validate_context(domain).map_err(|e| JsValue::from_str(e))
}

// ═══════════════════════════════════════════════════════════════════════
// §A  TL-SPONGE-385 (9-round full mode)
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen]
pub fn sponge_hash(input: &[u8], output_len: usize) -> Result<Vec<u8>, JsValue> {
    check_len(input.len(), MAX_BYTE_INPUT, "input_len")?;
    check_len(output_len, MAX_TRIT_OUTPUT, "output_len")?;
    Ok(sponge::hash(input, output_len))
}

#[wasm_bindgen]
pub fn sponge_derive_key(context: &[u8], material: &[u8], key_len: usize) -> Result<Vec<u8>, JsValue> {
    check_len(context.len(), MAX_BYTE_INPUT, "context_len")?;
    check_len(material.len(), MAX_BYTE_INPUT, "material_len")?;
    check_len(key_len, MAX_TRIT_OUTPUT, "key_len")?;
    Ok(sponge::derive_key(context, material, key_len))
}

/// Generate a keystream from a domain seed.
/// Context validation is enforced — domain must be a canonical context string.
#[wasm_bindgen]
pub fn sponge_keystream(domain: &[u8], trit_count: usize) -> Result<Vec<i8>, JsValue> {
    check_len(domain.len(), MAX_BYTE_INPUT, "domain_len")?;
    check_len(trit_count, MAX_TRIT_OUTPUT, "trit_count")?;
    check_context(domain)?;
    let mut trits = sponge::bytes_to_trits_pub(domain);
    let mut s = sponge::Sponge385Pub::new();
    s.absorb(&trits);
    let result = s.squeeze(trit_count);
    trits.zeroize();
    // TODO(R1-A1-3): Sponge385Pub must implement Zeroize + Drop in tlsponge385.rs.
    Ok(result)
}

/// Sponge duplex encryption — LOW-LEVEL PRIMITIVE (not T-AE-MAC).
///
/// Return format (12-byte header + payload):
///   Bytes 0..4:   ks1.len() as little-endian u32, bit-cast to i8
///   Bytes 4..8:   ks2.len() as little-endian u32, bit-cast to i8
///   Bytes 8..12:  mac.len() as little-endian u32, bit-cast to i8
///   Bytes 12..:   ks1 ++ ks2 ++ mac (concatenated trit streams)
///
/// The JavaScript receiver MUST reinterpret the first 12 i8 bytes as
/// unsigned (mask with & 0xFF) for correct u32 parsing.
#[wasm_bindgen]
pub fn sponge_duplex_encrypt(
    domain: &[u8],
    keystream_len: usize,
    switch_marker: &[u8],
    keystream2_len: usize,
    cipher1: &[u8],
    cipher2: &[u8],
    mac_trits: usize,
) -> Result<Vec<i8>, JsValue> {
    check_len(domain.len(), MAX_BYTE_INPUT, "domain_len")?;
    check_len(switch_marker.len(), MAX_BYTE_INPUT, "switch_marker_len")?;
    check_len(cipher1.len(), MAX_BYTE_INPUT, "cipher1_len")?;
    check_len(cipher2.len(), MAX_BYTE_INPUT, "cipher2_len")?;
    check_len(keystream_len, MAX_TRIT_OUTPUT, "keystream_len")?;
    check_len(keystream2_len, MAX_TRIT_OUTPUT, "keystream2_len")?;
    check_len(mac_trits, MAX_TRIT_OUTPUT, "mac_trits")?;
    check_context(domain)?;

    let mut domain_trits = sponge::bytes_to_trits_pub(domain);
    let mut switch_trits = sponge::bytes_to_trits_pub(switch_marker);
    let mut cipher1_trits = sponge::bytes_to_trits_pub(cipher1);
    let mut cipher2_trits = sponge::bytes_to_trits_pub(cipher2);

    let mut s = sponge::Sponge385Pub::new();
    s.absorb(&domain_trits);
    let ks1 = s.squeeze(keystream_len);
    s.absorb(&switch_trits);
    let ks2 = s.squeeze(keystream2_len);
    s.absorb(&cipher1_trits);
    s.absorb(&cipher2_trits);
    let mac = s.squeeze(mac_trits);

    domain_trits.zeroize();
    switch_trits.zeroize();
    cipher1_trits.zeroize();
    cipher2_trits.zeroize();
    // TODO(R1-A1-3): Sponge385Pub must implement Zeroize + Drop in tlsponge385.rs.

    let header_bytes: [u8; 12] = {
        let mut h = [0u8; 12];
        h[0..4].copy_from_slice(&(ks1.len() as u32).to_le_bytes());
        h[4..8].copy_from_slice(&(ks2.len() as u32).to_le_bytes());
        h[8..12].copy_from_slice(&(mac.len() as u32).to_le_bytes());
        h
    };
    let mut result: Vec<i8> = Vec::with_capacity(12 + ks1.len() + ks2.len() + mac.len());
    for &b in &header_bytes {
        result.push(b as i8);
    }
    result.extend(ks1);
    result.extend(ks2);
    result.extend(mac);
    Ok(result)
}

// ═══════════════════════════════════════════════════════════════════════
// §A.2  TIS-27 FAST-PATH (4-round mode)
// Requires hash_tis() in tlsponge385.rs (Step 1 above).
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen]
pub fn sponge_hash_tis(input: &[u8], output_len: usize) -> Result<Vec<u8>, JsValue> {
    check_len(input.len(), MAX_BYTE_INPUT, "input_len")?;
    check_len(output_len, MAX_TRIT_OUTPUT, "output_len")?;
    Ok(sponge::hash_tis(input, output_len))
}

#[wasm_bindgen]
pub fn sponge_derive_key_tis(context: &[u8], material: &[u8], key_len: usize) -> Result<Vec<u8>, JsValue> {
    check_len(context.len(), MAX_BYTE_INPUT, "context_len")?;
    check_len(material.len(), MAX_BYTE_INPUT, "material_len")?;
    check_len(key_len, MAX_TRIT_OUTPUT, "key_len")?;
    Ok(sponge::derive_key_tis(context, material, key_len))
}

// ═══════════════════════════════════════════════════════════════════════
// §B  UNIFIED CONSTANTS — direct from crate::constants
// ═══════════════════════════════════════════════════════════════════════

// §1 Repunit family
#[wasm_bindgen] pub fn ternary_base() -> u32 { constants::TERNARY_BASE }
#[wasm_bindgen] pub fn repunit_1() -> u32 { constants::REPUNIT_1 }
#[wasm_bindgen] pub fn repunit_2() -> u32 { constants::REPUNIT_2 }
#[wasm_bindgen] pub fn repunit_3() -> u32 { constants::REPUNIT_3 }
#[wasm_bindgen] pub fn repunit_4() -> u32 { constants::REPUNIT_4 }
#[wasm_bindgen] pub fn repunit_5() -> u32 { constants::REPUNIT_5 }
#[wasm_bindgen] pub fn repunit_6() -> u32 { constants::REPUNIT_6 }

// §2 Circle quadratic
#[wasm_bindgen] pub fn quad_sum() -> u32 { constants::QUAD_SUM }
#[wasm_bindgen] pub fn quad_product() -> u32 { constants::QUAD_PRODUCT }
#[wasm_bindgen] pub fn discriminant() -> u32 { constants::DISCRIMINANT }
#[wasm_bindgen] pub fn discriminant_sqrt() -> u32 { constants::DISCRIMINANT_SQRT }
#[wasm_bindgen] pub fn root_x1() -> u32 { constants::ROOT_X1 }
#[wasm_bindgen] pub fn root_x2() -> u32 { constants::ROOT_X2 }

// §3 Unified equation
#[wasm_bindgen] pub fn unified_linear() -> u32 { constants::UNIFIED_LINEAR }
#[wasm_bindgen] pub fn unified_constant() -> u32 { constants::UNIFIED_CONSTANT }
#[wasm_bindgen] pub fn unified_factor() -> u32 { constants::UNIFIED_FACTOR }
#[wasm_bindgen] pub fn unified_disc() -> u32 { constants::UNIFIED_DISC }
#[wasm_bindgen] pub fn unified_disc_sqrt() -> u32 { constants::UNIFIED_DISC_SQRT }
#[wasm_bindgen] pub fn arc_root_semi() -> u32 { constants::ARC_ROOT_SEMI }
#[wasm_bindgen] pub fn arc_root_comp() -> u32 { constants::ARC_ROOT_COMP }
#[wasm_bindgen] pub fn green_arc_eff() -> u32 { constants::GREEN_ARC_EFF }
#[wasm_bindgen] pub fn center() -> u32 { constants::CENTER }
#[wasm_bindgen] pub fn discriminant_2() -> u32 { constants::DISCRIMINANT_2 }
#[wasm_bindgen] pub fn discriminant_2_sqrt() -> u32 { constants::DISCRIMINANT_2_SQRT }
#[wasm_bindgen] pub fn magic_constant() -> u32 { constants::MAGIC_CONSTANT }
#[wasm_bindgen] pub fn circumference() -> u32 { constants::CIRCUMFERENCE }

// §5 Angular conversion
#[wasm_bindgen] pub fn std_circle_deg() -> u32 { constants::STD_CIRCLE_DEG }
#[wasm_bindgen] pub fn angular_conv_num() -> u32 { constants::ANGULAR_CONV_NUM }
#[wasm_bindgen] pub fn angular_conv_den() -> u32 { constants::ANGULAR_CONV_DEN }

// §6 UV spectral wavelengths
#[wasm_bindgen] pub fn lambda_euv() -> u32 { constants::LAMBDA_EUV }
#[wasm_bindgen] pub fn lambda_uvc() -> u32 { constants::LAMBDA_UVC }
#[wasm_bindgen] pub fn lambda_uvb() -> u32 { constants::LAMBDA_UVB }
#[wasm_bindgen] pub fn lambda_uva() -> u32 { constants::LAMBDA_UVA }
#[wasm_bindgen] pub fn lambda_far_uvc() -> u32 { constants::LAMBDA_FAR_UVC }
#[wasm_bindgen] pub fn lambda_excimer() -> u32 { constants::LAMBDA_EXCIMER }
#[wasm_bindgen] pub fn lambda_nb_uvb() -> u32 { constants::LAMBDA_NB_UVB }
#[wasm_bindgen] pub fn boundary_euv_uvc() -> u32 { constants::BOUNDARY_EUV_UVC }
#[wasm_bindgen] pub fn boundary_uvc_uvb() -> u32 { constants::BOUNDARY_UVC_UVB }
#[wasm_bindgen] pub fn boundary_uvb_uva() -> u32 { constants::BOUNDARY_UVB_UVA }
#[wasm_bindgen] pub fn boundary_uv_vis() -> u32 { constants::BOUNDARY_UV_VIS }
#[wasm_bindgen] pub fn vacuum_bias_num() -> u32 { constants::VACUUM_BIAS_NUM }
#[wasm_bindgen] pub fn vacuum_bias_den() -> u32 { constants::VACUUM_BIAS_DEN }

// §7 Coprime walk
#[wasm_bindgen] pub fn pentadecagon() -> u32 { constants::PENTADECAGON }
#[wasm_bindgen] pub fn lcm_primary() -> u32 { constants::LCM_PRIMARY }
#[wasm_bindgen] pub fn lcm_sext_max() -> u32 { constants::LCM_SEXT_MAX }
#[wasm_bindgen] pub fn geometric_spectral_product() -> u32 { constants::GEOMETRIC_SPECTRAL_PRODUCT }

// §8 CCP bridge
#[wasm_bindgen] pub fn null_harmonic_deficit() -> u32 { constants::NULL_HARMONIC_DEFICIT }
#[wasm_bindgen] pub fn bridge_ratio_num() -> u32 { constants::BRIDGE_RATIO_NUM }
#[wasm_bindgen] pub fn bridge_ratio_den() -> u32 { constants::BRIDGE_RATIO_DEN }
#[wasm_bindgen] pub fn deficit_rate_num() -> u32 { constants::DEFICIT_RATE_NUM }
#[wasm_bindgen] pub fn deficit_rate_den() -> u32 { constants::DEFICIT_RATE_DEN }

// §10 HModal signal
#[wasm_bindgen] pub fn alpha_num() -> u32 { constants::ALPHA_NUM }
#[wasm_bindgen] pub fn alpha_den() -> u32 { constants::ALPHA_DEN }
#[wasm_bindgen] pub fn beta_num() -> u32 { constants::BETA_NUM }
#[wasm_bindgen] pub fn beta_den() -> u32 { constants::BETA_DEN }
#[wasm_bindgen] pub fn gamma_num() -> u32 { constants::GAMMA_NUM }
#[wasm_bindgen] pub fn gamma_den() -> u32 { constants::GAMMA_DEN }
#[wasm_bindgen] pub fn duty_num() -> u32 { constants::DUTY_NUM }
#[wasm_bindgen] pub fn duty_den() -> u32 { constants::DUTY_DEN }
#[wasm_bindgen] pub fn dc_num() -> u32 { constants::DC_NUM }
#[wasm_bindgen] pub fn dc_den() -> u32 { constants::DC_DEN }

// §11 Channel architecture
#[wasm_bindgen] pub fn null_channel_mod() -> u32 { constants::NULL_CHANNEL_MOD }
#[wasm_bindgen] pub fn sin_period() -> u32 { constants::SIN_PERIOD }

// §12 Polygon geometry
#[wasm_bindgen] pub fn polygon_count() -> u32 { constants::POLYGON_COUNT }
#[wasm_bindgen] pub fn central_angle_square() -> u32 { constants::CENTRAL_ANGLE_SQUARE }
#[wasm_bindgen] pub fn central_angle_heptagon() -> u32 { constants::CENTRAL_ANGLE_HEPTAGON }
#[wasm_bindgen] pub fn central_angle_tridecagon() -> u32 { constants::CENTRAL_ANGLE_TRIDECAGON }
#[wasm_bindgen] pub fn central_angle_tetradecagon() -> u32 { constants::CENTRAL_ANGLE_TETRADECAGON }
#[wasm_bindgen] pub fn bezier_c182_angle() -> u32 { constants::BEZIER_C182_ANGLE }
#[wasm_bindgen] pub fn bezier_c650_angle() -> u32 { constants::BEZIER_C650_ANGLE }
#[wasm_bindgen] pub fn rim_vertices() -> u32 { constants::RIM_VERTICES }
#[wasm_bindgen] pub fn interior_intersections() -> u32 { constants::INTERIOR_INTERSECTIONS }
#[wasm_bindgen] pub fn total_nodes() -> u32 { constants::TOTAL_NODES }

// §14 Triangular numbers
#[wasm_bindgen] pub fn tri_3() -> u32 { constants::TRI_3 }
#[wasm_bindgen] pub fn tri_7() -> u32 { constants::TRI_7 }
#[wasm_bindgen] pub fn tri_10() -> u32 { constants::TRI_10 }
#[wasm_bindgen] pub fn tri_13() -> u32 { constants::TRI_13 }

// §15 Torus knot parameters
#[wasm_bindgen] pub fn crossing_11_14() -> u32 { constants::CROSSING_11_14 }
#[wasm_bindgen] pub fn crossing_13_14() -> u32 { constants::CROSSING_13_14 }
#[wasm_bindgen] pub fn crossing_13_15() -> u32 { constants::CROSSING_13_15 }
#[wasm_bindgen] pub fn crossing_14_15() -> u32 { constants::CROSSING_14_15 }

// §17 Plenum square
#[wasm_bindgen] pub fn plenum_square_step() -> u32 { constants::PLENUM_SQUARE_STEP }
#[wasm_bindgen] pub fn plenum_square_min() -> u32 { constants::PLENUM_SQUARE_MIN }

// §19a Fibonacci-physical bridge
#[wasm_bindgen] pub fn fibonacci_pi() -> u32 { constants::FIBONACCI_PI }
#[wasm_bindgen] pub fn fibonacci_12() -> u32 { constants::FIBONACCI_12 }
#[wasm_bindgen] pub fn fibonacci_13() -> u32 { constants::FIBONACCI_13 }

// Z28 cyclic group
#[wasm_bindgen] pub fn cyclic_order() -> u32 { constants::CYCLIC_ORDER }
#[wasm_bindgen] pub fn radians_per_circle() -> u32 { constants::RADIANS_PER_CIRCLE }

// §4 Squared circle
#[wasm_bindgen] pub fn unit_circle_area() -> u32 { constants::UNIT_CIRCLE_AREA }
#[wasm_bindgen] pub fn radian_circle_area() -> u32 { constants::RADIAN_CIRCLE_AREA }
#[wasm_bindgen] pub fn radius_num() -> u32 { constants::RADIUS_NUM }
#[wasm_bindgen] pub fn radius_den() -> u32 { constants::RADIUS_DEN }

// §B.2 f64 constants
#[wasm_bindgen] pub fn full_circle_deg() -> f64 { constants::FULL_CIRCLE_DEG }
#[wasm_bindgen] pub fn pi_ternary() -> f64 { constants::PI_TERNARY }
#[wasm_bindgen] pub fn two_pi_ternary() -> f64 { constants::TWO_PI_TERNARY }
#[wasm_bindgen] pub fn radian_deg() -> f64 { constants::RADIAN_DEG }
#[wasm_bindgen] pub fn bridge_coeff() -> f64 { constants::BRIDGE_COEFF }
#[wasm_bindgen] pub fn tau_tribonacci() -> f64 { constants::TAU_TRIBONACCI }
#[wasm_bindgen] pub fn tau_squared() -> f64 { constants::TAU_SQUARED }
#[wasm_bindgen] pub fn tau_cubed() -> f64 { constants::TAU_CUBED }
#[wasm_bindgen] pub fn tribonacci_golden_angle_deg() -> f64 { constants::TRIBONACCI_GOLDEN_ANGLE_DEG }

// §19b CODATA 2022
#[wasm_bindgen] pub fn codata_rydberg_const() -> f64 { constants::CODATA_RYDBERG_CONST }
#[wasm_bindgen] pub fn codata_me_over_mp() -> f64 { constants::CODATA_ME_OVER_MP }
#[wasm_bindgen] pub fn codata_balmer_constant_nm() -> f64 { constants::CODATA_BALMER_CONSTANT_NM }
#[wasm_bindgen] pub fn codata_lyman_limit_nm() -> f64 { constants::CODATA_LYMAN_LIMIT_NM }
#[wasm_bindgen] pub fn codata_lyman_alpha_nm() -> f64 { constants::CODATA_LYMAN_ALPHA_NM }
#[wasm_bindgen] pub fn codata_h_alpha_nm() -> f64 { constants::CODATA_H_ALPHA_NM }
#[wasm_bindgen] pub fn codata_z0_ohm() -> f64 { constants::CODATA_Z0_OHM }
#[wasm_bindgen] pub fn codata_hartree_ev() -> f64 { constants::CODATA_HARTREE_EV }
#[wasm_bindgen] pub fn codata_rydberg_ev() -> f64 { constants::CODATA_RYDBERG_EV }
#[wasm_bindgen] pub fn codata_inv_alpha() -> f64 { constants::CODATA_INV_ALPHA }
#[wasm_bindgen] pub fn h_alpha_residual() -> f64 { constants::H_ALPHA_RESIDUAL }

// §19c Tolerance bounds
#[wasm_bindgen] pub fn codata_tier_1_tolerance() -> f64 { constants::CODATA_TIER_1_TOLERANCE }
#[wasm_bindgen] pub fn codata_tier_2_tolerance() -> f64 { constants::CODATA_TIER_2_TOLERANCE }
#[wasm_bindgen] pub fn codata_tier_2b_tolerance() -> f64 { constants::CODATA_TIER_2B_TOLERANCE }

// Superhub zones
#[wasm_bindgen] pub fn superhub_x_left() -> f64 { constants::SUPERHUB_X_LEFT }
#[wasm_bindgen] pub fn superhub_x_right() -> f64 { constants::SUPERHUB_X_RIGHT }
#[wasm_bindgen] pub fn superhub_y_ab() -> f64 { constants::SUPERHUB_Y_AB }
#[wasm_bindgen] pub fn superhub_y_cd() -> f64 { constants::SUPERHUB_Y_CD }

// §B.3 Conversion utilities
// NOTE: NaN/infinity inputs produce NaN output (IEEE 754 propagation).
#[wasm_bindgen] pub fn ternary_deg_to_std_deg(v: f64) -> f64 { constants::ternary_deg_to_std_deg(v) }
#[wasm_bindgen] pub fn std_deg_to_ternary_deg(v: f64) -> f64 { constants::std_deg_to_ternary_deg(v) }
#[wasm_bindgen] pub fn ternary_rad_to_std_rad(v: f64) -> f64 { constants::ternary_rad_to_std_rad(v) }
#[wasm_bindgen] pub fn std_rad_to_ternary_rad(v: f64) -> f64 { constants::std_rad_to_ternary_rad(v) }
#[wasm_bindgen] pub fn ternary_deg_to_ternary_rad(v: f64) -> f64 { constants::ternary_deg_to_ternary_rad(v) }
#[wasm_bindgen] pub fn ternary_rad_to_ternary_deg(v: f64) -> f64 { constants::ternary_rad_to_ternary_deg(v) }

// §B.4 Computed predictions
#[wasm_bindgen]
pub fn predict_lyman(n: u32) -> f64 {
    let r6 = constants::QUAD_PRODUCT as f64;
    let n2 = (n * n) as f64;
    r6 * n2 / (4.0 * (n2 - 1.0))
}

#[wasm_bindgen]
pub fn predict_balmer(n: u32) -> f64 {
    let r6 = constants::QUAD_PRODUCT as f64;
    let n2 = (n * n) as f64;
    r6 * n2 / (n2 - 4.0)
}

#[wasm_bindgen]
pub fn predict_paschen(n: u32) -> f64 {
    let r6 = constants::QUAD_PRODUCT as f64;
    let n2 = (n * n) as f64;
    r6 * 9.0 * n2 / (4.0 * (n2 - 9.0))
}

#[wasm_bindgen]
pub fn predict_hydrogen(m: u32, n: u32) -> f64 {
    let r6 = constants::QUAD_PRODUCT as f64;
    let m2 = (m * m) as f64;
    let n2 = (n * n) as f64;
    r6 * m2 * n2 / (4.0 * (n2 - m2))
}

#[wasm_bindgen]
pub fn rel_error(predicted: f64, measured: f64) -> f64 {
    (predicted - measured).abs() / measured
}

// ═══════════════════════════════════════════════════════════════════════
// PHASE 5: TritInt and Trit WASM Exports
// ═══════════════════════════════════════════════════════════════════════

use crate::trit_int::TritInt;
use crate::trit::Trit;

#[wasm_bindgen]
pub fn trit_int_from_u64(val: u64) -> Vec<u8> {
    TritInt::from_u64(val).to_repr_c()
}

#[wasm_bindgen]
pub fn trit_int_to_decimal(repr_c: &[u8]) -> Result<u64, JsValue> {
    let t = TritInt::try_from_repr_c(repr_c)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    t.to_u64().map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn trit_int_display(repr_c: &[u8]) -> Result<String, JsValue> {
    let t = TritInt::try_from_repr_c(repr_c)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(format!("{}", t))
}

#[wasm_bindgen]
pub fn trit_int_to_repr_c(val: u64) -> Vec<u8> {
    TritInt::from_u64(val).to_repr_c()
}

#[wasm_bindgen]
pub fn trit_int_from_repr_c(repr_c: &[u8]) -> Result<Vec<u8>, JsValue> {
    let t = TritInt::try_from_repr_c(repr_c)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(t.to_repr_c())
}

#[wasm_bindgen]
pub fn trit_new_scalar(val: u64) -> String {
    let t = Trit::from_u64(val);
    trit_to_json(&t)
}

#[wasm_bindgen]
pub fn trit_new_golden(a: u64, b: u64) -> String {
    let t = Trit::golden(TritInt::from_u64(a), TritInt::from_u64(b));
    trit_to_json(&t)
}

#[wasm_bindgen]
pub fn trit_display(json: &str) -> Result<String, JsValue> {
    let t = trit_from_json(json)?;
    Ok(format!("{}", t))
}

#[wasm_bindgen]
pub fn trit_to_f64(json: &str) -> Result<f64, JsValue> {
    let t = trit_from_json(json)?;
    Ok(t.to_f64())
}

#[wasm_bindgen]
pub fn trit_add(a_json: &str, b_json: &str) -> Result<String, JsValue> {
    let a = trit_from_json(a_json)?;
    let b = trit_from_json(b_json)?;
    Ok(trit_to_json(&Trit::add(&a, &b)))
}

#[wasm_bindgen]
pub fn trit_mul_golden(a_json: &str, b_json: &str) -> Result<String, JsValue> {
    let a = trit_from_json(a_json)?;
    let b = trit_from_json(b_json)?;
    Ok(trit_to_json(&a.mul_golden(&b)))
}

#[wasm_bindgen]
pub fn trit_norm_golden(json: &str) -> Result<String, JsValue> {
    let t = trit_from_json(json)?;
    Ok(trit_to_json(&t.norm_golden()))
}

fn trit_to_json(t: &Trit) -> String {
    let v0 = t.v[0].to_repr_c();
    let v1 = t.v[1].to_repr_c();
    let v2 = t.v[2].to_repr_c();
    format!("{{\"v\":[{},{},{}]}}",
        format_repr_c(&v0), format_repr_c(&v1), format_repr_c(&v2))
}

fn format_repr_c(repr: &[u8]) -> String {
    let digits: Vec<String> = repr.iter().map(|d| d.to_string()).collect();
    format!("[{}]", digits.join(","))
}

fn trit_from_json(json: &str) -> Result<Trit, JsValue> {
    parse_trit_json_minimal(json)
        .map_err(|e| JsValue::from_str(&e))
}

fn parse_trit_json_minimal(json: &str) -> Result<Trit, String> {
    let s = json.trim();
    let v_start = s.find("\"v\":[")
        .ok_or("missing \"v\" field")?;
    let inner = &s[v_start + 4..];
    let inner = inner.trim_start_matches('[');

    let mut arrays: Vec<Vec<u8>> = Vec::new();
    let mut depth = 0;
    let mut current = String::new();
    for ch in inner.chars() {
        match ch {
            '[' => { depth += 1; current.push(ch); }
            ']' => {
                if depth == 0 { break; }
                depth -= 1;
                current.push(ch);
                if depth == 0 {
                    arrays.push(parse_u8_array(&current)?);
                    current.clear();
                }
            }
            ',' if depth == 0 => {}
            _ => current.push(ch),
        }
    }

    if arrays.len() != 3 {
        return Err(format!("expected 3 arrays, got {}", arrays.len()));
    }

    let v0 = TritInt::try_from_repr_c(&arrays[0])
        .map_err(|e| e.to_string())?;
    let v1 = TritInt::try_from_repr_c(&arrays[1])
        .map_err(|e| e.to_string())?;
    let v2 = TritInt::try_from_repr_c(&arrays[2])
        .map_err(|e| e.to_string())?;

    Ok(Trit::new(v0, v1, v2))
}

fn parse_u8_array(s: &str) -> Result<Vec<u8>, String> {
    let inner = s.trim().trim_start_matches('[').trim_end_matches(']');
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    inner.split(',')
        .map(|d| d.trim().parse::<u8>().map_err(|e| e.to_string()))
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn verify_repunits() {
        assert_eq!(ternary_base(), constants::TERNARY_BASE);
        assert_eq!(repunit_3(), constants::REPUNIT_3);
        assert_eq!(repunit_6(), constants::REPUNIT_6);
    }

    #[test] fn verify_circle_quadratic() {
        assert_eq!(quad_product(), constants::QUAD_PRODUCT);
        assert_eq!(root_x1(), constants::ROOT_X1);
        assert_eq!(root_x2(), constants::ROOT_X2);
        assert_eq!(discriminant(), constants::DISCRIMINANT);
    }

    #[test] fn verify_unified() {
        assert_eq!(arc_root_semi(), constants::ARC_ROOT_SEMI);
        assert_eq!(arc_root_comp(), constants::ARC_ROOT_COMP);
        assert_eq!(green_arc_eff(), constants::GREEN_ARC_EFF);
        assert_eq!(center(), constants::CENTER);
        assert_eq!(discriminant_2_sqrt(), constants::DISCRIMINANT_2_SQRT);
    }

    #[test] fn verify_uv() {
        assert_eq!(lambda_euv(), constants::LAMBDA_EUV);
        assert_eq!(lambda_uvc(), constants::LAMBDA_UVC);
        assert_eq!(lambda_uvb(), constants::LAMBDA_UVB);
        assert_eq!(lambda_uva(), constants::LAMBDA_UVA);
        assert_eq!(vacuum_bias_num(), constants::VACUUM_BIAS_NUM);
    }

    #[test] fn verify_fibonacci() {
        assert_eq!(fibonacci_pi(), constants::FIBONACCI_PI);
        assert_eq!(fibonacci_12(), constants::FIBONACCI_12);
    }

    #[test] fn verify_codata() {
        assert_eq!(codata_balmer_constant_nm(), constants::CODATA_BALMER_CONSTANT_NM);
        assert_eq!(codata_z0_ohm(), constants::CODATA_Z0_OHM);
        assert_eq!(codata_h_alpha_nm(), constants::CODATA_H_ALPHA_NM);
    }

    #[test] fn lyman_series() {
        assert!((predict_lyman(2) - 121.333333).abs() < 0.001);
        assert!((predict_lyman(1000) - 91.0).abs() < 0.01);
    }

    #[test] fn balmer_series() {
        assert!((predict_balmer(3) - 655.2).abs() < 0.001);
        assert!((predict_balmer(1000) - 364.0).abs() < 0.01);
    }

    #[test] fn hydrogen_general_consistency() {
        for n in 2..8 { assert!((predict_hydrogen(1, n) - predict_lyman(n)).abs() < 1e-10); }
        for n in 3..8 { assert!((predict_hydrogen(2, n) - predict_balmer(n)).abs() < 1e-10); }
        for n in 4..8 { assert!((predict_hydrogen(3, n) - predict_paschen(n)).abs() < 1e-10); }
    }

    #[test] fn codata_tier_validation() {
        assert!(rel_error(364.0, constants::CODATA_BALMER_CONSTANT_NM) < constants::CODATA_TIER_1_TOLERANCE);
        assert!(rel_error(constants::FIBONACCI_PI as f64, constants::CODATA_Z0_OHM) < constants::CODATA_TIER_2_TOLERANCE);
    }

    #[test] fn reject_oversized_len() {
        assert!(validate_len(MAX_TRIT_OUTPUT + 1, MAX_TRIT_OUTPUT, "test").is_err());
        assert!(validate_len(MAX_TRIT_OUTPUT, MAX_TRIT_OUTPUT, "test").is_ok());
        assert!(validate_len(0, MAX_TRIT_OUTPUT, "test").is_ok());
    }

    #[test] fn reject_invalid_context() {
        assert!(validate_context(b"invalid").is_err());
        assert!(validate_context(b"PlenumNET-Phase-v2").is_ok());
        assert!(validate_context(b"PlenumNET-KEM-v1").is_ok());
        assert!(validate_context(b"PlenumNET-DSA-v1").is_ok());
        assert!(validate_context(b"PlenumNET-TIS27-v1").is_ok());
        assert!(validate_context(b"PlenumNET-Duplex-v1").is_ok());
    }

    #[test] fn reject_empty_context() {
        assert!(validate_context(b"").is_err());
    }

    #[test] fn canonical_context_list_is_five() {
        assert_eq!(CANONICAL_CONTEXTS.len(), 5);
    }
}
