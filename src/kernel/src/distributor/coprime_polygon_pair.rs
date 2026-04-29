// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved — Applied Physics Division
//
// (11, 13) Coprime Polygon Pair — src/kernel/src/distributor/coprime_polygon_pair.rs
// Reference: TM-2026-025 v3
//
// Add `pub mod coprime_polygon_pair;` to distributor/mod.rs.

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Core constants
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Combined arc: 11 × 13 = 143° = 11 ternary radians.
pub const COPRIME_ARC: u32 = 143;
/// Hendecagon edges — generates Z₃₆₄. gcd(11, 364) = 1.
pub const HENDECAGON_EDGES: u32 = 11;
/// Tridecagon edges — generates Z₂₈. gcd(13, 28) = 1.
pub const TRIDECAGON_EDGES: u32 = 13;
/// Combined vertices: 11 + 13 − 1 = 23 = 11⁻¹ mod 28 = 143 − φ(143).
pub const COMBINED_VERTICES: u32 = 23;
/// φ(143) = φ(11)×φ(13) = 10×12 = 120. ARC_BLUE = 2 × this.
pub const EULER_TOTIENT_143: u32 = 120;
/// Palindromic interleave: 11 entries, sum = 12 = 13 − 1.
pub const INTERLEAVE_PATTERN: [u8; 11] = [1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1];
/// Bézout: 11 × 6 + 13 × (−5) = 1. Note: 6 = φ(7).
pub const BEZOUT_COEFFICIENTS: (i32, i32) = (6, -5);
/// CF: 364/143 = [2; 1, 1, 5]. Final convergent: 28/11.
pub const CONTINUED_FRACTION: [u32; 4] = [2, 1, 1, 5];
/// φ(364) − φ(143) = 24 = T₈.
pub const TOTIENT_GAP: u32 = 24;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// PlenumColor harmonic system (all derived from the (11, 13) pair)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// ARC_RED = 182 = 14 × 13 = πR₃ (semicircle root).
pub const ARC_RED: u32 = 182;
/// ARC_BLUE = 240 = 2 × φ(143) = 3⁵ − 3.
/// CRT in Z₇₅₆: (24, 16) = (T₈, 2⁴).
/// CRT in Z₃₆₄: (0, 2, 6) where 6 = φ(7) = Bézout[0].
pub const ARC_BLUE: u32 = 240;
/// ARC_COPRIME = 286 = 2 × 143 = 2 × 11 × 13.
/// Bridge: ARC_GREEN = FULL_CIRCLE + ARC_COPRIME.
pub const ARC_COPRIME: u32 = 286;
/// √Δ_arc = 468 = 36 × 13 = ARC_RED + ARC_COPRIME.
/// Roots: (832 ± 468) / 2 = {182, 650}.
pub const ARC_SQRT_DISCRIMINANT: u32 = 468;
/// ARC_GREEN = 650 = FULL_CIRCLE + ARC_COPRIME (rejected root).
pub const ARC_GREEN: u32 = 650;
/// Full ternary circle = 364 = 111111₃.
pub const FULL_CIRCLE_DEG: u32 = 364;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Compile-time helper
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Standard Euclidean greatest-common-divisor, available as a `const fn`
/// for compile-time coprimality assertions and exposed as `pub` for
/// downstream callers that need the same primitive.
pub const fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 { let t = b; b = a % b; a = t; } a
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Compile-time assertions (zero runtime cost)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// §1 Generator Duality
const _: () = assert!(HENDECAGON_EDGES * TRIDECAGON_EDGES == COPRIME_ARC);
const _: () = assert!(gcd(HENDECAGON_EDGES, 364) == 1);       // 11 generates Z₃₆₄
const _: () = assert!(gcd(TRIDECAGON_EDGES, 364) == 13);      // 13 does NOT
const _: () = assert!(gcd(TRIDECAGON_EDGES, 28) == 1);        // 13 generates Z₂₈
// §4 Combined vertices
const _: () = assert!(HENDECAGON_EDGES + TRIDECAGON_EDGES - 1 == COMBINED_VERTICES);
const _: () = assert!(COPRIME_ARC - EULER_TOTIENT_143 == COMBINED_VERTICES); // Theorem 4.1
// Sponge strides
const _: () = assert!(gcd(HENDECAGON_EDGES, 54) == 1);
const _: () = assert!(gcd(TRIDECAGON_EDGES, 54) == 1);
const _: () = assert!(gcd(COMBINED_VERTICES, 54) == 1);
// §3 CRT images
const _: () = assert!(COPRIME_ARC % 27 == 8);   // branch number
const _: () = assert!(COPRIME_ARC % 28 == 3);   // Rep C max
const _: () = assert!(COPRIME_ARC % 4 == 3);
const _: () = assert!(COPRIME_ARC % 7 == 3);
const _: () = assert!(COPRIME_ARC % 13 == 0);
// §8 Bézout
const _: () = assert!((HENDECAGON_EDGES as i32) * BEZOUT_COEFFICIENTS.0
                     + (TRIDECAGON_EDGES as i32) * BEZOUT_COEFFICIENTS.1 == 1);
// §5 Interleave
const _: () = assert!(INTERLEAVE_PATTERN.len() == HENDECAGON_EDGES as usize);

// §6 Unified equation
const _: () = assert!(ARC_COPRIME == 2 * COPRIME_ARC);
const _: () = assert!(ARC_GREEN - FULL_CIRCLE_DEG == ARC_COPRIME);
const _: () = assert!(ARC_SQRT_DISCRIMINANT == ARC_RED + ARC_COPRIME);
const _: () = assert!(ARC_SQRT_DISCRIMINANT == ARC_GREEN - ARC_RED);
const _: () = assert!(ARC_SQRT_DISCRIMINANT * ARC_SQRT_DISCRIMINANT == 832 * 832 - 4 * 118300);
const _: () = assert!(ARC_SQRT_DISCRIMINANT == 36 * 13);
const _: () = assert!((832 - ARC_SQRT_DISCRIMINANT) / 2 == ARC_RED);
const _: () = assert!((832 + ARC_SQRT_DISCRIMINANT) / 2 == ARC_GREEN);
const _: () = assert!(ARC_RED - COPRIME_ARC == 3 * 13);
const _: () = assert!(FULL_CIRCLE_DEG == 2 * ARC_RED);

// §6.1 ARC_BLUE derivation
const _: () = assert!(ARC_BLUE == 2 * EULER_TOTIENT_143);     // 240 = 2 × φ(143)
const _: () = assert!(ARC_BLUE == 243 - 3);                    // 240 = 3⁵ − 3
const _: () = assert!(ARC_COPRIME - ARC_BLUE == 2 * COMBINED_VERTICES); // 286 − 240 = 46
const _: () = assert!(ARC_BLUE % 27 == 24);                   // CRT: T₈ on Z₂₇
const _: () = assert!(ARC_BLUE % 28 == 16);                   // CRT: 2⁴ on Z₂₈
const _: () = assert!(ARC_BLUE % 13 == 6);                    // CRT: φ(7) on Z₁₃

// Vieta's formulas
const _: () = assert!(ARC_RED + ARC_GREEN == 832);
const _: () = assert!(ARC_RED * ARC_GREEN == 118300);

// Cross-reference: COPRIME_ARC = ternary_math::constants::COPRIME_PAIR_LCMS[3] = 143
// Cross-reference: ARC_RED = ternary_math::constants::COPRIME_PAIR_LCMS[6] = 182
// Cross-reference: ARC_COPRIME = ternary_math::constants::GREEN_ARC_EFF = 286
const _: () = assert!(COPRIME_ARC == 143);
const _: () = assert!(ARC_RED == 182);
const _: () = assert!(ARC_COPRIME == 286);

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Runtime tests
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(test)]
fn euler_totient(mut n: u32) -> u32 {
    let mut result = n; let mut p = 2u32; let mut temp = n;
    while p * p <= temp {
        if temp % p == 0 { while temp % p == 0 { temp /= p; } result -= result / p; }
        p += 1;
    }
    if temp > 1 { result -= result / temp; }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test] fn product() { assert_eq!(11 * 13, COPRIME_ARC); }
    #[test] fn gen_z364() { assert_eq!((0..364).map(|k| (11*k)%364).collect::<HashSet<_>>().len(), 364); }
    #[test] fn no_gen_z364_by_13() { assert_eq!((0..364).map(|k| (13*k)%364).collect::<HashSet<_>>().len(), 28); }
    #[test] fn both_gen_z28() {
        for s in [11u32,13] { assert_eq!((0..28).map(|k|(s*k)%28).collect::<HashSet<_>>().len(), 28); }
    }
    #[test] fn different_orders() {
        let w11: Vec<u32> = (0..28).map(|k|(11*k)%28).collect();
        let w13: Vec<u32> = (0..28).map(|k|(13*k)%28).collect();
        assert_ne!(w11, w13);
    }
    #[test] fn euclidean_ladder() {
        let (mut a, mut b) = (364u32, 143);
        while b != 0 { let r = a%b; if r != 0 { assert_eq!(r%13, 0); } a=b; b=r; }
        assert_eq!(a, 13);
    }
    #[test] fn cf_convergent() {
        let cf = CONTINUED_FRACTION;
        let (mut hp,mut hc) = (1u32, cf[0]); let (mut kp,mut kc) = (0u32, 1u32);
        for i in 1..cf.len() { let hn=cf[i]*hc+hp; let kn=cf[i]*kc+kp; hp=hc; hc=hn; kp=kc; kc=kn; }
        assert_eq!((hc, kc), (28, 11));
    }
    #[test] fn crt_364() { assert_eq!((COPRIME_ARC%4, COPRIME_ARC%7, COPRIME_ARC%13), (3,3,0)); }
    #[test] fn crt_756() { assert_eq!((COPRIME_ARC%27, COPRIME_ARC%28), (8,3)); }
    #[test] fn vertices() { assert_eq!(COMBINED_VERTICES, 11+13-1); }
    #[test] fn inverse_23() { assert_eq!((11u32*23)%28, 1); }
    #[test] fn self_inv_13() { assert_eq!((13u32*13)%28, 1); }
    #[test] fn inclusion_exclusion() { assert_eq!(COPRIME_ARC - EULER_TOTIENT_143, COMBINED_VERTICES); }
    #[test] fn palindrome() { let il = INTERLEAVE_PATTERN; for i in 0..il.len() { assert_eq!(il[i], il[10-i]); } }
    #[test] fn interleave_sum() { assert_eq!(INTERLEAVE_PATTERN.iter().map(|&x|x as u32).sum::<u32>(), 12); }
    #[test] fn bezout() { assert_eq!(11i32*6 + 13*(-5), 1); }
    #[test] fn totient_gap() { assert_eq!(euler_totient(364)-euler_totient(143), TOTIENT_GAP); }
    #[test] fn bezout_coeff_is_phi7() { assert_eq!(euler_totient(7), 6); }

    // PlenumColor harmonics
    #[test] fn green_eq_circle_plus_coprime() { assert_eq!(FULL_CIRCLE_DEG + ARC_COPRIME, ARC_GREEN); }
    #[test] fn red_plus_coprime_eq_sqrt_d() { assert_eq!(ARC_RED + ARC_COPRIME, ARC_SQRT_DISCRIMINANT); }
    #[test] fn coprime_eq_double_arc() { assert_eq!(ARC_COPRIME, 2*COPRIME_ARC); }
    #[test] fn sqrt_d_eq_root_diff() { assert_eq!(ARC_GREEN - ARC_RED, ARC_SQRT_DISCRIMINANT); }
    #[test] fn discriminant() { assert_eq!(468u32*468, 832*832 - 4*118300); }
    #[test] fn roots_from_formula() { assert_eq!(((832-468)/2, (832+468)/2), (182, 650)); }
    #[test] fn sqrt_d_factorization() { assert_eq!(ARC_SQRT_DISCRIMINANT, 36*13); }
    #[test] fn circle_double_red() { assert_eq!(FULL_CIRCLE_DEG, 2*ARC_RED); }
    #[test] fn vieta_sum() { assert_eq!(ARC_RED + ARC_GREEN, 832); }
    #[test] fn vieta_product() { assert_eq!(ARC_RED * ARC_GREEN, 118300); }
    #[test] fn half_circle_minus_arc() { assert_eq!(ARC_RED - COPRIME_ARC, 39); }
    #[test] fn rejected_decomposition() { assert_eq!(FULL_CIRCLE_DEG + 2*COPRIME_ARC, ARC_GREEN); }

    // ARC_BLUE derivation
    #[test] fn blue_eq_double_totient() { assert_eq!(ARC_BLUE, 2*EULER_TOTIENT_143); }
    #[test] fn blue_eq_3pow5_minus_3() { assert_eq!(ARC_BLUE, 243-3); }
    #[test] fn blue_totient_verified() { assert_eq!(euler_totient(143), EULER_TOTIENT_143); }
    #[test] fn coprime_minus_blue_eq_double_vertices() { assert_eq!(ARC_COPRIME - ARC_BLUE, 2*COMBINED_VERTICES); }
    #[test] fn blue_crt_z27() { assert_eq!(ARC_BLUE % 27, 24); } // T₈
    #[test] fn blue_crt_z28() { assert_eq!(ARC_BLUE % 28, 16); } // 2⁴
    #[test] fn blue_crt_z13() { assert_eq!(ARC_BLUE % 13, 6); }  // φ(7)
    #[test] fn two_times_scaling() {
        assert_eq!(2 * COPRIME_ARC, ARC_COPRIME);            // 2 × 143 = 286
        assert_eq!(2 * EULER_TOTIENT_143, ARC_BLUE);          // 2 × 120 = 240
        assert_eq!(2 * COMBINED_VERTICES, ARC_COPRIME - ARC_BLUE); // 2 × 23 = 46
    }
}
