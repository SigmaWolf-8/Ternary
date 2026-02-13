// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division

use ternary_math::ternary_circle::Z28;
use ternary_math::clifford::ternary_circle_bridge::{
    z28_position_to_rotor, z28_clifford_walk,
    compose_canonical_tribonacci_walk, verify_z28_rotor_consistency,
    angular_step_rotor,
};
use ternary_math::torus::ternary_circle_bridge::{
    z28_to_torus_address, tribonacci_torus_walk,
    z28_torus_walk, tau_scaled_torus_distance,
    torus_address_to_z28,
};
use ternary_math::tribonacci::tribonacci_word;
use ternary_math::clifford::Multivector;
use ternary_math::gf3::Gf3;
use ternary_math::constants::{TAU_TRIBONACCI, CYCLIC_ORDER};

#[test]
fn clifford_identity_rotor_at_z28_zero() {
    let rotor = z28_position_to_rotor(Z28::zero());
    let identity = Multivector::scalar(Gf3::ONE);
    assert_eq!(rotor, identity,
        "Z₂₈(0) must produce the identity rotor (scalar=1, all others=0)");
}

#[test]
fn clifford_rotor_z28_produces_valid_rotors_all_784_pairs() {
    let identity = Multivector::scalar(Gf3::ONE);

    assert_eq!(z28_position_to_rotor(Z28::zero()), identity,
        "Z₂₈(0) must map to identity rotor");

    for a in 0..28u8 {
        let rotor = z28_position_to_rotor(Z28(a));
        let is_nonzero = rotor.components.iter().any(|&c| c != Gf3::ZERO);
        assert!(is_nonzero, "Z₂₈({}) produced zero multivector", a);
    }

    for a in 0..28u8 {
        for b in 0..28u8 {
            let ra = z28_position_to_rotor(Z28(a));
            let rb = z28_position_to_rotor(Z28(b));
            let composed = ra * rb;
            let is_nonzero = composed.components.iter().any(|&c| c != Gf3::ZERO);
            assert!(is_nonzero,
                "Product of Z₂₈({}) and Z₂₈({}) rotors is zero", a, b);
        }
    }

    for trit in 0..3u8 {
        let r = angular_step_rotor(trit);
        let product = r * identity;
        assert_eq!(product, r,
            "Rotor for trit {} is not right-identity-compatible", trit);
        let product2 = identity * r;
        assert_eq!(product2, r,
            "Rotor for trit {} is not left-identity-compatible", trit);
    }
}

#[test]
fn clifford_walk_gf3_consistency() {
    let word = tribonacci_word(50);
    let walk = z28_clifford_walk(&word);

    for (i, (z28_pos, _rotor)) in walk.iter().enumerate() {
        let residue = z28_pos.to_gf3_residue();
        assert!(residue <= 2,
            "GF(3) residue out of range at step {}: got {}", i, residue);
    }

    assert!(verify_z28_rotor_consistency(50),
        "Z₂₈ ↔ rotor consistency check failed for 50-step walk");
}

#[test]
fn torus_address_covers_positions_0_through_26_distinctly() {
    let dims = 3;
    let mut addresses: Vec<String> = Vec::with_capacity(27);

    for k in 0..27u8 {
        let addr = z28_to_torus_address(Z28(k), dims);
        assert_eq!(addr.dimensions(), dims,
            "Z₂₈({}) produced wrong dimension count", k);
        let key = format!("{}", addr);
        addresses.push(key);
    }

    for i in 0..27 {
        for j in (i+1)..27 {
            assert_ne!(addresses[i], addresses[j],
                "Z₂₈({}) and Z₂₈({}) map to the same torus address: {}",
                i, j, addresses[i]);
        }
    }

    let addr_0 = z28_to_torus_address(Z28(0), dims);
    let addr_27 = z28_to_torus_address(Z28(27), dims);
    let key_0 = format!("{}", addr_0);
    let key_27 = format!("{}", addr_27);
    assert_eq!(key_0, key_27,
        "Z₂₈(27) must collide with Z₂₈(0) in 3D torus (27 = 3³ ≡ 0 mod 3³): \
         got {} vs {}", key_0, key_27);
}

#[test]
fn torus_walk_and_z28_walk_position_consistency() {
    let dims = 3;
    let steps = 30;

    let z28_walk = z28_torus_walk(dims, steps);
    let torus_walk = tribonacci_torus_walk(dims, steps);

    assert_eq!(z28_walk.len(), steps,
        "z28_torus_walk returned wrong number of steps");
    assert_eq!(torus_walk.len(), steps + 1,
        "tribonacci_torus_walk returned wrong number of steps (should include origin)");

    for (i, (z28_pos, torus_addr)) in z28_walk.iter().enumerate() {
        let walk_addr = &torus_walk[i + 1];
        assert_eq!(torus_addr.dimensions(), walk_addr.dimensions(),
            "Dimension mismatch at step {}", i);
        for d in 0..dims {
            assert_eq!(torus_addr.trits.get(d), walk_addr.trits.get(d),
                "Torus coordinate mismatch at step {}, dim {}", i, d);
        }
    }
}

#[test]
fn tau_scaled_distance_decreases_with_step() {
    let addr_a = z28_to_torus_address(Z28(0), 3);
    let addr_b = z28_to_torus_address(Z28(13), 3);

    let d0 = tau_scaled_torus_distance(&addr_a, &addr_b, 0);
    let d1 = tau_scaled_torus_distance(&addr_a, &addr_b, 1);
    let d2 = tau_scaled_torus_distance(&addr_a, &addr_b, 2);

    assert!(d0 > 0.0, "Distance at step 0 should be positive");
    assert!(d1 < d0, "τ-scaled distance should decrease: d1={} >= d0={}", d1, d0);
    assert!(d2 < d1, "τ-scaled distance should decrease: d2={} >= d1={}", d2, d1);

    let ratio = d0 / d1;
    assert!((ratio - TAU_TRIBONACCI).abs() < 1e-10,
        "Distance ratio should equal τ: got {}, expected {}", ratio, TAU_TRIBONACCI);
}

#[test]
fn canonical_tribonacci_walk_composition() {
    let (rotor, word) = compose_canonical_tribonacci_walk(28);

    assert_eq!(word.len(), 28,
        "Canonical walk should produce 28 symbols");

    for &s in &word {
        assert!(s <= 2, "Tribonacci word contains invalid symbol: {}", s);
    }

    let walk = z28_clifford_walk(&word);
    assert_eq!(walk.len(), 28,
        "Clifford walk should have 28 steps");

    let final_rotor = &walk[27].1;
    assert_eq!(*final_rotor, rotor,
        "Final accumulated rotor should equal the composed rotor");
}

#[test]
fn z28_to_torus_roundtrip_within_range() {
    for i in 0..27u8 {
        let pos = Z28(i);
        let addr = z28_to_torus_address(pos, 3);
        let back = torus_address_to_z28(&addr);
        assert_eq!(back.value(), i,
            "Z₂₈({}) → torus(3D) → Z₂₈ roundtrip failed: got Z₂₈({})",
            i, back.value());
    }
    let addr_27 = z28_to_torus_address(Z28(27), 3);
    let back_27 = torus_address_to_z28(&addr_27);
    assert_eq!(back_27.value(), 0,
        "Z₂₈(27) should wrap to Z₂₈(0) via torus (27 mod 28 = 27, but 3³ = 27 ≡ 0 mod 28 is NOT 27; \
         expected wrap due to torus lattice size 3³=27 < 28)");
}

#[test]
fn gf3_fiber_sizes_10_9_9() {
    let mut counts = [0u32; 3];
    for k in 0..28u8 {
        counts[Z28(k).to_gf3_residue() as usize] += 1;
    }
    assert_eq!(counts[0], 10, "Residue 0 fiber should have 10 elements");
    assert_eq!(counts[1], 9, "Residue 1 fiber should have 9 elements");
    assert_eq!(counts[2], 9, "Residue 2 fiber should have 9 elements");
}

#[test]
fn angular_step_rotor_trit_values() {
    let r0 = angular_step_rotor(0);
    let r1 = angular_step_rotor(1);
    let r2 = angular_step_rotor(2);

    assert_eq!(r0, Multivector::scalar(Gf3::ONE),
        "Trit 0 should produce identity rotor");
    assert_ne!(r1, r0, "Trit 1 rotor should differ from identity");
    assert_ne!(r2, r0, "Trit 2 rotor should differ from identity");
    assert_ne!(r1, r2, "Trit 1 and Trit 2 rotors should differ");
}
