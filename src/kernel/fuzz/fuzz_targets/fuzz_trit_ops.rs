// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

#![no_main]
use libfuzzer_sys::fuzz_target;
use plenumnet_kernel::ternary::{Trit, Representation, convert_representation};

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }

    let a_raw = (data[0] % 3) as i8 - 1;
    let b_raw = (data[1] % 3) as i8 - 1;

    let a = Trit::from_a(a_raw).unwrap();
    let b = Trit::from_a(b_raw).unwrap();

    let sum = a.add(&b);
    assert!(sum.to_a() >= -1 && sum.to_a() <= 1, "GF(3) add produced out-of-range trit");

    let sum2 = b.add(&a);
    assert_eq!(sum.to_a(), sum2.to_a(), "GF(3) add not commutative");

    let product = a.multiply(&b);
    assert!(product.to_a() >= -1 && product.to_a() <= 1, "GF(3) mul produced out-of-range trit");

    let product2 = b.multiply(&a);
    assert_eq!(product.to_a(), product2.to_a(), "GF(3) mul not commutative");

    let xor_ab = a.xor(&b);
    let xor_ba = b.xor(&a);
    assert_eq!(xor_ab.to_a(), xor_ba.to_a(), "XOR not commutative");

    let not_a = a.not();
    assert_eq!(not_a.not().to_a(), a.to_a(), "NOT is not involution");

    let rotated = a.rotate().rotate().rotate();
    assert_eq!(rotated.to_a(), a.to_a(), "Rotation period != 3");

    let inv_cancel = a.rotate().rotate_inverse();
    assert_eq!(inv_cancel.to_a(), a.to_a(), "rotate_inverse doesn't cancel rotate");

    let zero = Trit::from_a(0).unwrap();
    assert_eq!(a.add(&zero).to_a(), a.to_a(), "0 is not additive identity");

    let one = Trit::from_a(1).unwrap();
    assert_eq!(a.multiply(&one).to_a(), a.to_a(), "1 is not multiplicative identity");
    assert_eq!(a.multiply(&zero).to_a(), 0, "0 is not multiplicative absorbing");

    assert_eq!(a.to_b() as i8, a.to_a() + 1, "A->B bijection violated");
    assert_eq!(a.to_c() as i8, a.to_a() + 2, "A->C bijection violated");

    if data.len() >= 3 {
        let c_raw = (data[2] % 3) as i8 - 1;
        let c = Trit::from_a(c_raw).unwrap();

        let assoc1 = a.add(&b).add(&c);
        let assoc2 = a.add(&b.add(&c));
        assert_eq!(assoc1.to_a(), assoc2.to_a(), "GF(3) add not associative");

        let mul_assoc1 = a.multiply(&b).multiply(&c);
        let mul_assoc2 = a.multiply(&b.multiply(&c));
        assert_eq!(mul_assoc1.to_a(), mul_assoc2.to_a(), "GF(3) mul not associative");

        let distrib1 = a.multiply(&b.add(&c));
        let distrib2 = a.multiply(&b).add(&a.multiply(&c));
        assert_eq!(distrib1.to_a(), distrib2.to_a(), "GF(3) distributive law violated");
    }

    for raw in 0..data.len().min(16) {
        let val = data[raw] as i8;
        let from_a = Trit::from_a(val);
        if val < -1 || val > 1 {
            assert!(from_a.is_none(), "from_a accepted invalid value {}", val);
        }
        let from_b = Trit::from_b(data[raw]);
        if data[raw] > 2 {
            assert!(from_b.is_none(), "from_b accepted invalid value {}", data[raw]);
        }
    }

    let repr_vals = [
        (Representation::A, Representation::B),
        (Representation::A, Representation::C),
        (Representation::B, Representation::C),
    ];
    for (from, to) in repr_vals {
        let converted = convert_representation(a_raw, Representation::A, from);
        let back = convert_representation(converted, from, Representation::A);
        assert_eq!(back, a_raw, "Representation roundtrip failed: A -> {:?} -> A", from);
    }
});
