// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file contains trade secrets of Capomastro Holdings Ltd.
// Unauthorized copying, distribution, or use is strictly prohibited.
//
// coprime.rs — Canonical coprime operations for the Salvi Framework.
// ONE source for coprimality, totient, walks, combinations, and CRT.
// All public API is TritInt. No u64 in any public signature.

use crate::trit_int::TritInt;

// ══════════════════════════════════════════════════════════════
// COPRIMALITY PREDICATE
// ══════════════════════════════════════════════════════════════

/// True if gcd(a, b) == 1. Uses TritInt::gcd (Phase 1).
pub fn is_coprime(a: &TritInt, b: &TritInt) -> bool {
    TritInt::gcd(a, b) == TritInt::one()
}

// ══════════════════════════════════════════════════════════════
// EULER TOTIENT
// ══════════════════════════════════════════════════════════════

/// Euler's totient function φ(n) via trial division in TritInt arithmetic.
///
/// φ(n) = n × ∏_{p|n} (1 − 1/p)
///
/// Matches the approach at coprime_polygon_pair.rs:125-133.
/// φ(1) = 1. φ(p) = p − 1 for prime p.
pub fn euler_totient(n: &TritInt) -> TritInt {
    if *n <= TritInt::one() {
        return n.clone();
    }
    let mut result = n.clone();
    let mut temp = n.clone();
    let one = TritInt::one();
    let mut p = TritInt::from_u64(2);

    loop {
        let p_sq = TritInt::mul(&p, &p);
        if p_sq > temp { break; }
        let (_, rem) = temp.div_mod(&p);
        if rem.is_zero() {
            // Divide out all factors of p from temp
            loop {
                let (q, r) = temp.div_mod(&p);
                if r.is_zero() {
                    temp = q;
                } else {
                    break;
                }
            }
            // result -= result / p
            let (quot, _) = result.div_mod(&p);
            result = TritInt::sub(&result, &quot);
        }
        p = TritInt::add(&p, &one);
    }
    // If temp > 1, then temp is a remaining prime factor
    if temp > one {
        let (quot, _) = result.div_mod(&temp);
        result = TritInt::sub(&result, &quot);
    }
    result
}

// ══════════════════════════════════════════════════════════════
// COPRIME OPTIONS
// ══════════════════════════════════════════════════════════════

/// All values in [min, max] coprime to `axis`.
pub fn coprime_options(axis: &TritInt, min: &TritInt, max: &TritInt) -> Vec<TritInt> {
    let one = TritInt::one();
    let mut result = Vec::new();
    let mut val = min.clone();
    while val <= *max {
        if is_coprime(&val, axis) {
            result.push(val.clone());
        }
        val = TritInt::add(&val, &one);
    }
    result
}

// ══════════════════════════════════════════════════════════════
// 2D COPRIME WALK
// ══════════════════════════════════════════════════════════════

/// 2D coprime walk on Z_a × Z_b. Requires gcd(a,b) = 1.
/// Returns all a×b distinct (x mod a, y mod b) coordinate pairs.
///
/// Walk step: smallest integer ≥ 2 coprime to a×b. This spreads
/// adjacent walk positions apart on the torus (better entropy
/// distribution than step=1 sequential enumeration).
///
/// The step cannot be a or b (both divide a×b). For coprime_walk(11, 13),
/// step=2 (smallest coprime to 143). For custom generators per dimension,
/// use `multidim_walk`.
pub fn coprime_walk(a: &TritInt, b: &TritInt) -> Vec<(TritInt, TritInt)> {
    assert!(is_coprime(a, b), "coprime_walk requires gcd(a,b)=1");
    let n = TritInt::mul(a, b);
    let one = TritInt::one();

    // Find smallest step ≥ 2 coprime to n
    let mut step = TritInt::from_u64(2);
    while !is_coprime(&step, &n) {
        step = TritInt::add(&step, &one);
    }

    let n_count = n.to_decimal() as usize; // host control flow
    let mut result = Vec::with_capacity(n_count);
    let mut pos = TritInt::zero();
    for _ in 0..n_count {
        let (_, x) = pos.div_mod(a);
        let (_, y) = pos.div_mod(b);
        result.push((x, y));
        pos = TritInt::add(&pos, &step);
    }
    result
}

// ══════════════════════════════════════════════════════════════
// COPRIME COMBINATIONS (MAXIMAL SUBSETS)
// ══════════════════════════════════════════════════════════════

/// Enumerate all maximal pairwise-coprime subsets of `moduli`.
///
/// Maximal = no element from `moduli` can be added without breaking
/// pairwise coprimality. Bounded: practical input size |moduli| ≤ ~15.
///
/// For the 13-polygon set {3,4,5,7,8,9,11,13}, produces the four
/// sextuples from TM-2026-028a §3.3, including the maximum
/// {5,7,8,9,11,13} with product 360,360.
pub fn coprime_combinations(moduli: &[TritInt]) -> Vec<Vec<TritInt>> {
    let mut results = Vec::new();
    let mut current = Vec::new();
    find_maximal(moduli, 0, &mut current, &mut results);
    results
}

fn find_maximal(
    moduli: &[TritInt],
    start: usize,
    current: &mut Vec<TritInt>,
    results: &mut Vec<Vec<TritInt>>,
) {
    let mut extended = false;
    for i in start..moduli.len() {
        if current.iter().all(|c| is_coprime(c, &moduli[i])) {
            current.push(moduli[i].clone());
            find_maximal(moduli, i + 1, current, results);
            current.pop();
            extended = true;
        }
    }
    if !extended && !current.is_empty() {
        let truly_maximal = moduli.iter().all(|m| {
            current.contains(m) || !current.iter().all(|c| is_coprime(c, m))
        });
        if truly_maximal {
            results.push(current.clone());
        }
    }
}

// ══════════════════════════════════════════════════════════════
// k-DIMENSIONAL HAMILTONIAN WALK
// ══════════════════════════════════════════════════════════════

/// k-dimensional Hamiltonian cycle on Z_{m_1} × ... × Z_{m_k}.
/// Each generator[i] must be coprime to moduli[i].
/// Cycle length = product of all moduli.
///
/// Panics if lengths mismatch or any generator is not coprime to its modulus.
pub fn multidim_walk(moduli: &[TritInt], generators: &[TritInt]) -> Vec<Vec<TritInt>> {
    assert_eq!(moduli.len(), generators.len(),
        "multidim_walk: moduli and generators must have same length");
    for i in 0..moduli.len() {
        assert!(is_coprime(&generators[i], &moduli[i]),
            "generator must be coprime to modulus");
    }

    let mut cycle_len: u64 = 1;
    for m in moduli {
        cycle_len *= m.to_decimal(); // host control flow
    }

    let k = moduli.len();
    let mut result = Vec::with_capacity(cycle_len as usize);
    let mut pos: Vec<TritInt> = vec![TritInt::zero(); k];

    for _ in 0..cycle_len {
        result.push(pos.clone());
        for j in 0..k {
            let sum = TritInt::add(&pos[j], &generators[j]);
            let (_, rem) = sum.div_mod(&moduli[j]);
            pos[j] = rem;
        }
    }
    result
}

// ══════════════════════════════════════════════════════════════
// CRT — STANDALONE CHINESE REMAINDER THEOREM
//
// These serve both the AGS (currently uses its own crt_project/
// crt_reconstruct on the Ags struct) and the Coprime Index
// (TM-2026-028a). Standalone: not tied to any dynamic generator set.
// ══════════════════════════════════════════════════════════════

/// CRT combination: given residues and pairwise-coprime moduli, recover
/// the unique value in [0, product(moduli)) that produces those residues.
///
/// Uses Garner's algorithm (successive substitution).
///
/// Panics if residues.len() != moduli.len() or moduli are not pairwise coprime.
pub fn crt_combine(residues: &[TritInt], moduli: &[TritInt]) -> TritInt {
    assert_eq!(residues.len(), moduli.len(),
        "crt_combine: residues and moduli must have same length");
    assert!(!moduli.is_empty(), "crt_combine: empty moduli");

    // Verify pairwise coprimality
    for i in 0..moduli.len() {
        for j in (i + 1)..moduli.len() {
            assert!(is_coprime(&moduli[i], &moduli[j]),
                "crt_combine requires pairwise coprime moduli");
        }
    }

    if moduli.len() == 1 {
        return residues[0].div_mod(&moduli[0]).1;
    }

    // Garner's algorithm: successive substitution
    let mut result = residues[0].clone();
    let mut product = moduli[0].clone();

    for i in 1..moduli.len() {
        // Find inverse of product mod moduli[i] via extended_gcd
        let (_, inv, _) = TritInt::extended_gcd(&product, &moduli[i]);

        // diff = (residues[i] - result) mod moduli[i]
        // Since TritInt is unsigned, handle the subtraction carefully
        let result_mod_mi = result.div_mod(&moduli[i]).1;
        let diff = if residues[i] >= result_mod_mi {
            TritInt::sub(&residues[i], &result_mod_mi)
        } else {
            let gap = TritInt::sub(&result_mod_mi, &residues[i]);
            TritInt::sub(&moduli[i], &gap)
        };

        // step = (diff * inv) mod moduli[i]
        let step = TritInt::mul(&diff, &inv).div_mod(&moduli[i]).1;

        // result += product * step
        result = TritInt::add(&result, &TritInt::mul(&product, &step));

        // product *= moduli[i]
        product = TritInt::mul(&product, &moduli[i]);
    }

    // Reduce to canonical range [0, product)
    result.div_mod(&product).1
}

/// CRT decomposition: split a value into its residues mod each modulus.
///
/// Inverse of crt_combine:
///   crt_combine(crt_split(v, m), m) == v   for v < product(m).
pub fn crt_split(value: &TritInt, moduli: &[TritInt]) -> Vec<TritInt> {
    moduli.iter().map(|m| value.div_mod(m).1).collect()
}

// ══════════════════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_coprime() {
        let v11 = TritInt::from_u64(11);
        let v13 = TritInt::from_u64(13);
        let v28 = TritInt::from_u64(28);
        let v364 = TritInt::repunit(6); // 364 = R₆
        assert!(is_coprime(&v11, &v364));     // 11 generates Z₃₆₄
        assert!(!is_coprime(&v13, &v364));     // gcd(13, 364) = 13
        assert!(is_coprime(&v13, &v28));       // gcd(13, 28) = 1
        assert!(is_coprime(&TritInt::one(), &v364)); // 1 coprime to everything
    }

    #[test]
    fn test_totient_known_values() {
        // φ(364) = 144 — from constants.rs
        assert_eq!(euler_totient(&TritInt::repunit(6)).to_decimal(), 144);
        // φ(143) = 120 — coprime_polygon_pair.rs:102
        assert_eq!(euler_totient(&TritInt::from_u64(143)).to_decimal(), 120);
        // φ(7) = 6 — prime → p−1
        assert_eq!(euler_totient(&TritInt::from_u64(7)).to_decimal(), 6);
        // φ(1) = 1
        assert_eq!(euler_totient(&TritInt::one()).to_decimal(), 1);
        // φ(360360) = 2³×3²×5×7×11×13, φ = 360360 × (1−1/2)(1−1/3)(1−1/5)(1−1/7)(1−1/11)(1−1/13)
        //           = 360360 × 1/2 × 2/3 × 4/5 × 6/7 × 10/11 × 12/13 = 69120
        assert_eq!(euler_totient(&TritInt::from_u64(360_360)).to_decimal(), 69_120);
    }

    #[test]
    fn test_coprime_walk_covers_all() {
        let a = TritInt::from_u64(11);
        let b = TritInt::from_u64(13);
        let walk = coprime_walk(&a, &b);
        assert_eq!(walk.len(), 143); // 11 × 13 = 143 = COPRIME_ARC
        let mut set = std::collections::HashSet::new();
        for pair in &walk {
            set.insert((pair.0.to_decimal(), pair.1.to_decimal()));
        }
        assert_eq!(set.len(), 143); // all pairs distinct
    }

    #[test]
    fn test_coprime_walk_order() {
        let a = TritInt::from_u64(11);
        let b = TritInt::from_u64(13);
        let walk = coprime_walk(&a, &b);
        // Step=2 (smallest coprime to 143=11×13): pos 0, 2, 4...
        assert_eq!(walk[0], (TritInt::zero(), TritInt::zero()));
        // pos=2: (2%11, 2%13) = (2, 2)
        assert_eq!(walk[1], (TritInt::from_u64(2), TritInt::from_u64(2)));
        // pos=4: (4%11, 4%13) = (4, 4)
        assert_eq!(walk[2], (TritInt::from_u64(4), TritInt::from_u64(4)));
        // Verify NOT sequential (step≠1)
        assert_ne!(walk[1], (TritInt::one(), TritInt::one()));
    }

    #[test]
    fn test_coprime_options_for_364() {
        let v364 = TritInt::repunit(6);
        let opts = coprime_options(&v364, &TritInt::one(), &TritInt::from_u64(30));
        let vals: Vec<u64> = opts.iter().map(|t| t.to_decimal()).collect();
        assert!(vals.contains(&11));  // 11 generates Z₃₆₄
        assert!(!vals.contains(&13)); // gcd(13, 364) = 13
        assert!(!vals.contains(&14)); // gcd(14, 364) = 14
        assert!(vals.contains(&1));   // 1 coprime to everything
    }

    #[test]
    fn test_multidim_walk_full_cycle() {
        let moduli = [TritInt::from_u64(5), TritInt::from_u64(7)];
        let gens = [TritInt::from_u64(2), TritInt::from_u64(3)];
        let walk = multidim_walk(&moduli, &gens);
        assert_eq!(walk.len(), 35); // 5 × 7
        let mut set = std::collections::HashSet::new();
        for pos in &walk {
            set.insert((pos[0].to_decimal(), pos[1].to_decimal()));
        }
        assert_eq!(set.len(), 35); // all positions distinct
    }

    #[test]
    fn test_coprime_combinations_pairwise() {
        let input: Vec<TritInt> = [5, 7, 10, 11].iter()
            .map(|&v| TritInt::from_u64(v)).collect();
        let combos = coprime_combinations(&input);
        // Every pair in every combo must be coprime
        for combo in &combos {
            for i in 0..combo.len() {
                for j in (i + 1)..combo.len() {
                    assert!(is_coprime(&combo[i], &combo[j]));
                }
            }
        }
    }

    #[test]
    fn test_coprime_combinations_are_maximal() {
        let input: Vec<TritInt> = [5, 7, 10, 11].iter()
            .map(|&v| TritInt::from_u64(v)).collect();
        let combos = coprime_combinations(&input);
        // No element from input can extend any returned subset
        for combo in &combos {
            for m in &input {
                if combo.contains(m) { continue; }
                let can_add = combo.iter().all(|c| is_coprime(c, m));
                assert!(!can_add, "subset not maximal");
            }
        }
    }

    #[test]
    fn test_coprime_combinations_sextuple() {
        // TM-2026-028a §3.2: maximum sextuple from 13-polygon set
        let polygon_set: Vec<TritInt> = [3, 4, 5, 7, 8, 9, 11, 13].iter()
            .map(|&v| TritInt::from_u64(v)).collect();
        let combos = coprime_combinations(&polygon_set);
        // Must contain {5,7,8,9,11,13} with product 360,360
        let max_product: u64 = combos.iter().map(|c| {
            c.iter().map(|t| t.to_decimal()).product::<u64>()
        }).max().unwrap();
        assert_eq!(max_product, 360_360);
        // All combos must be sextuples (size 6) for this input
        for combo in &combos {
            assert_eq!(combo.len(), 6, "all maximal subsets of this set have size 6");
        }
    }

    #[test]
    fn test_crt_roundtrip() {
        let moduli = [
            TritInt::from_u64(5), TritInt::from_u64(7),
            TritInt::from_u64(11), TritInt::from_u64(13),
        ];
        let val = TritInt::from_u64(42);
        let residues = crt_split(&val, &moduli);
        let recovered = crt_combine(&residues, &moduli);
        assert_eq!(recovered, val);
    }

    #[test]
    fn test_crt_sextuple_roundtrip() {
        // TM-2026-028a sextuple: (5,7,8,9,11,13), M = 360,360
        let moduli: Vec<TritInt> = [5, 7, 8, 9, 11, 13].iter()
            .map(|&v| TritInt::from_u64(v)).collect();
        for &v in &[0u64, 1, 42, 1001, 15015, 360_359] {
            let val = TritInt::from_u64(v);
            let residues = crt_split(&val, &moduli);
            let recovered = crt_combine(&residues, &moduli);
            assert_eq!(recovered.to_decimal(), v, "CRT roundtrip failed for {}", v);
        }
    }
}
