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
use plenumnet_kernel::ternary::{Trit, Tryte, TernaryWord, information_density};

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }

    let decimal_a = ((data[0] as u16) << 8 | data[1] as u16) % 730;

    if decimal_a < 729 {
        let tryte_a = Tryte::from_decimal(decimal_a).unwrap();
        let roundtrip = tryte_a.to_decimal();
        assert_eq!(roundtrip, decimal_a, "Tryte decimal roundtrip failed: {} != {}", roundtrip, decimal_a);

        let not_tryte = tryte_a.not();
        let not_not = not_tryte.not();
        assert_eq!(not_not.to_decimal(), decimal_a, "Tryte NOT involution failed");

        for trit in tryte_a.trits() {
            let a = trit.to_a();
            assert!(a >= -1 && a <= 1, "Tryte contains invalid trit value: {}", a);
        }
    }

    assert!(Tryte::from_decimal(729).is_none(), "Tryte accepted 729");
    assert!(Tryte::from_decimal(1000).is_none(), "Tryte accepted 1000");

    if data.len() >= 4 {
        let decimal_b = ((data[2] as u16) << 8 | data[3] as u16) % 729;
        let tryte_a = Tryte::from_decimal(decimal_a.min(728)).unwrap();
        let tryte_b = Tryte::from_decimal(decimal_b).unwrap();

        let sum = tryte_a.add(&tryte_b);
        let sum_decimal = sum.to_decimal();
        assert!(sum_decimal < 729, "Tryte add produced out-of-range decimal: {}", sum_decimal);

        for trit in sum.trits() {
            let a = trit.to_a();
            assert!(a >= -1 && a <= 1, "Tryte add result contains invalid trit");
        }
    }

    if data.len() >= 6 {
        let mut trits = [Trit::from_a(0).unwrap(); 6];
        for i in 0..6 {
            let val = (data[i % data.len()] % 3) as i8 - 1;
            trits[i] = Trit::from_a(val).unwrap();
        }
        let tryte = Tryte::new(trits);
        let decimal = tryte.to_decimal();
        assert!(decimal < 729, "Manual tryte has invalid decimal: {}", decimal);
    }

    if data.len() >= 6 {
        let mut trytes = [Tryte::from_decimal(0).unwrap(), Tryte::from_decimal(0).unwrap(), Tryte::from_decimal(0).unwrap()];
        for i in 0..3 {
            let idx = i * 2;
            if idx + 1 < data.len() {
                let val = ((data[idx] as u16) << 8 | data[idx + 1] as u16) % 729;
                trytes[i] = Tryte::from_decimal(val).unwrap();
            }
        }
        let word = TernaryWord::new(trytes);
        assert_eq!(word.trytes().len(), 3);
    }

    let trit_count = (data[0] as u32 % 32) + 1;
    let density = information_density(trit_count);
    assert_eq!(density.trit_count, trit_count);
    assert!(density.ternary_states > 0, "Zero ternary states");
    assert!(density.binary_states > 0, "Zero binary states");
    assert!(density.efficiency_gain > 0.0, "Non-positive efficiency gain");
    assert!(density.equivalent_bits > 0.0, "Non-positive equivalent bits");
});
