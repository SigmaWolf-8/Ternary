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

extern crate alloc;

use plenumnet_kernel::ternary::{Trit, Tryte, Representation, convert_representation, information_density};
use plenumnet_kernel::compat::gateway::{
    binary_to_balanced_ternary, balanced_ternary_to_binary,
    binary_bytes_to_ternary, ternary_to_binary_bytes,
    binary_u8_to_representation_b, representation_b_to_binary_u8,
    BinaryTernaryGateway, GatewayMode,
};

#[cfg(test)]
mod proptest_ternary {
    use super::*;
    use proptest::prelude::*;

    fn arb_trit_value() -> impl Strategy<Value = i8> {
        prop_oneof![Just(-1i8), Just(0i8), Just(1i8)]
    }

    fn arb_trit() -> impl Strategy<Value = Trit> {
        arb_trit_value().prop_map(|v| Trit::from_a(v).unwrap())
    }

    proptest! {
        #[test]
        fn gf3_add_commutative(a in arb_trit(), b in arb_trit()) {
            prop_assert_eq!(a.add(&b).to_a(), b.add(&a).to_a());
        }

        #[test]
        fn gf3_add_associative(a in arb_trit(), b in arb_trit(), c in arb_trit()) {
            let lhs = a.add(&b).add(&c);
            let rhs = a.add(&b.add(&c));
            prop_assert_eq!(lhs.to_a(), rhs.to_a());
        }

        #[test]
        fn gf3_add_identity(a in arb_trit()) {
            let zero = Trit::from_a(0).unwrap();
            prop_assert_eq!(a.add(&zero).to_a(), a.to_a());
        }

        #[test]
        fn gf3_mul_commutative(a in arb_trit(), b in arb_trit()) {
            prop_assert_eq!(a.multiply(&b).to_a(), b.multiply(&a).to_a());
        }

        #[test]
        fn gf3_mul_associative(a in arb_trit(), b in arb_trit(), c in arb_trit()) {
            let lhs = a.multiply(&b).multiply(&c);
            let rhs = a.multiply(&b.multiply(&c));
            prop_assert_eq!(lhs.to_a(), rhs.to_a());
        }

        #[test]
        fn gf3_mul_identity(a in arb_trit()) {
            let one = Trit::from_a(1).unwrap();
            prop_assert_eq!(a.multiply(&one).to_a(), a.to_a());
        }

        #[test]
        fn gf3_mul_absorbing(a in arb_trit()) {
            let zero = Trit::from_a(0).unwrap();
            prop_assert_eq!(a.multiply(&zero).to_a(), 0);
        }

        #[test]
        fn gf3_distributive(a in arb_trit(), b in arb_trit(), c in arb_trit()) {
            let lhs = a.multiply(&b.add(&c));
            let rhs = a.multiply(&b).add(&a.multiply(&c));
            prop_assert_eq!(lhs.to_a(), rhs.to_a());
        }

        #[test]
        fn not_involution(a in arb_trit()) {
            prop_assert_eq!(a.not().not().to_a(), a.to_a());
        }

        #[test]
        fn rotate_period_3(a in arb_trit()) {
            prop_assert_eq!(a.rotate().rotate().rotate().to_a(), a.to_a());
        }

        #[test]
        fn rotate_inverse_cancels(a in arb_trit()) {
            prop_assert_eq!(a.rotate().rotate_inverse().to_a(), a.to_a());
            prop_assert_eq!(a.rotate_inverse().rotate().to_a(), a.to_a());
        }

        #[test]
        fn xor_commutative(a in arb_trit(), b in arb_trit()) {
            prop_assert_eq!(a.xor(&b).to_a(), b.xor(&a).to_a());
        }

        #[test]
        fn xor_associative(a in arb_trit(), b in arb_trit(), c in arb_trit()) {
            let lhs = a.xor(&b).xor(&c);
            let rhs = a.xor(&b.xor(&c));
            prop_assert_eq!(lhs.to_a(), rhs.to_a());
        }

        #[test]
        fn bijection_a_b_roundtrip(a in arb_trit()) {
            let b_val = a.to_b();
            let reconstructed = Trit::from_b(b_val).unwrap();
            prop_assert_eq!(reconstructed.to_a(), a.to_a());
        }

        #[test]
        fn bijection_a_c_roundtrip(a in arb_trit()) {
            let c_val = a.to_c();
            let reconstructed = Trit::from_c(c_val).unwrap();
            prop_assert_eq!(reconstructed.to_a(), a.to_a());
        }

        #[test]
        fn bijection_formula_a_to_b(a in arb_trit()) {
            prop_assert_eq!(a.to_b() as i8, a.to_a() + 1);
        }

        #[test]
        fn bijection_formula_a_to_c(a in arb_trit()) {
            prop_assert_eq!(a.to_c() as i8, a.to_a() + 2);
        }

        #[test]
        fn trit_range_invariant(a in arb_trit()) {
            let val = a.to_a();
            prop_assert!(val >= -1 && val <= 1);
            let val_b = a.to_b();
            prop_assert!(val_b <= 2);
            let val_c = a.to_c();
            prop_assert!(val_c >= 1 && val_c <= 3);
        }

        #[test]
        fn add_result_range(a in arb_trit(), b in arb_trit()) {
            let r = a.add(&b).to_a();
            prop_assert!(r >= -1 && r <= 1);
        }

        #[test]
        fn mul_result_range(a in arb_trit(), b in arb_trit()) {
            let r = a.multiply(&b).to_a();
            prop_assert!(r >= -1 && r <= 1);
        }

        #[test]
        fn convert_representation_roundtrip(
            a in arb_trit_value(),
            from_repr in prop_oneof![Just(Representation::A), Just(Representation::B), Just(Representation::C)],
            to_repr in prop_oneof![Just(Representation::A), Just(Representation::B), Just(Representation::C)]
        ) {
            let converted = convert_representation(a, Representation::A, from_repr);
            let back = convert_representation(converted, from_repr, Representation::A);
            prop_assert_eq!(back, a);
        }
    }
}

#[cfg(test)]
mod proptest_tryte {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn tryte_decimal_roundtrip(val in 0u16..729) {
            let tryte = Tryte::from_decimal(val).unwrap();
            prop_assert_eq!(tryte.to_decimal(), val);
        }

        #[test]
        fn tryte_not_involution(val in 0u16..729) {
            let tryte = Tryte::from_decimal(val).unwrap();
            prop_assert_eq!(tryte.not().not().to_decimal(), val);
        }

        #[test]
        fn tryte_add_closed(a in 0u16..729, b in 0u16..729) {
            let ta = Tryte::from_decimal(a).unwrap();
            let tb = Tryte::from_decimal(b).unwrap();
            let sum = ta.add(&tb);
            prop_assert!(sum.to_decimal() < 729);
        }

        #[test]
        fn tryte_invalid_rejected(val in 729u16..2000) {
            prop_assert!(Tryte::from_decimal(val).is_none());
        }

        #[test]
        fn tryte_trit_values_valid(val in 0u16..729) {
            let tryte = Tryte::from_decimal(val).unwrap();
            for trit in tryte.trits() {
                let a = trit.to_a();
                prop_assert!(a >= -1 && a <= 1);
            }
        }
    }
}

#[cfg(test)]
mod proptest_gateway {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn balanced_ternary_i64_roundtrip(value in -100000i64..100000) {
            let trits = binary_to_balanced_ternary(value);
            for &t in &trits {
                prop_assert!(t >= -1 && t <= 1);
            }
            if value == 0 {
                prop_assert!(trits.is_empty());
            } else {
                let back = balanced_ternary_to_binary(&trits).unwrap();
                prop_assert_eq!(back, value);
            }
        }

        #[test]
        fn balanced_ternary_large_roundtrip(value in prop::num::i64::ANY) {
            let trits = binary_to_balanced_ternary(value);
            for &t in &trits {
                prop_assert!(t >= -1 && t <= 1);
            }
            if value == 0 {
                prop_assert!(trits.is_empty());
            } else {
                let back = balanced_ternary_to_binary(&trits);
                match back {
                    Ok(recovered) => prop_assert_eq!(recovered, value),
                    Err(_) => {} // overflow is acceptable for extreme values
                }
            }
        }

        #[test]
        fn binary_bytes_roundtrip(data in proptest::collection::vec(any::<u8>(), 0..256)) {
            let trits = binary_bytes_to_ternary(&data);
            prop_assert_eq!(trits.len(), data.len() * 6);
            for &t in &trits {
                prop_assert!(t >= -1 && t <= 1);
            }
            let recovered = ternary_to_binary_bytes(&trits).unwrap();
            prop_assert_eq!(recovered, data);
        }

        #[test]
        fn representation_b_u8_roundtrip(byte in any::<u8>()) {
            let rep_b = binary_u8_to_representation_b(byte);
            for &d in &rep_b {
                prop_assert!(d <= 2);
            }
            let back = representation_b_to_binary_u8(&rep_b).unwrap();
            prop_assert_eq!(back, byte);
        }

        #[test]
        fn gateway_bytes_roundtrip(
            data in proptest::collection::vec(any::<u8>(), 1..128),
            mode in prop_oneof![Just(GatewayMode::Strict), Just(GatewayMode::Lossy), Just(GatewayMode::Balanced)]
        ) {
            let mut gw = BinaryTernaryGateway::new(mode);
            let trits = gw.convert_to_ternary(&data).unwrap();
            let back = gw.convert_to_binary(&trits).unwrap();
            prop_assert_eq!(back, data);
            let stats = gw.stats();
            prop_assert_eq!(stats.conversions, 2);
        }

        #[test]
        fn gateway_i64_roundtrip(value in -50000i64..50000) {
            let mut gw = BinaryTernaryGateway::new(GatewayMode::Balanced);
            let trits = gw.convert_i64_to_ternary(value);
            if value != 0 {
                let back = gw.convert_ternary_to_i64(&trits).unwrap();
                prop_assert_eq!(back, value);
            }
        }

        #[test]
        fn invalid_trit_rejected(
            data in proptest::collection::vec(-5i8..5, 1..20)
        ) {
            let has_invalid = data.iter().any(|&t| t < -1 || t > 1);
            let result = balanced_ternary_to_binary(&data);
            if has_invalid {
                prop_assert!(result.is_err());
            }
        }

        #[test]
        fn ternary_to_binary_rejects_wrong_length(
            len in (1usize..100).prop_filter("not multiple of 6", |l| l % 6 != 0)
        ) {
            let trits: Vec<i8> = (0..len).map(|i| (i as i8 % 3) - 1).collect();
            prop_assert!(ternary_to_binary_bytes(&trits).is_err());
        }
    }
}

#[cfg(test)]
mod proptest_density {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn density_monotonic(a in 1u32..30, b in 1u32..30) {
            let da = information_density(a);
            let db = information_density(b);
            if a > b {
                prop_assert!(da.ternary_states >= db.ternary_states);
            }
        }

        #[test]
        fn density_states_correct(n in 1u32..20) {
            let d = information_density(n);
            prop_assert_eq!(d.ternary_states, 3u128.pow(n));
            prop_assert_eq!(d.trit_count, n);
        }

        #[test]
        fn density_positive(n in 1u32..30) {
            let d = information_density(n);
            prop_assert!(d.ternary_states > 0);
            prop_assert!(d.binary_states > 0);
            prop_assert!(d.efficiency_gain > 0.0);
        }
    }
}
