// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved
//
// Kani Proof Harnesses — Core Kernel Modules
//
// These are formal proofs, not tests. Kani explores ALL possible inputs
// within the bounded space and mathematically proves the properties hold.
// A passing Kani proof is stronger than any amount of testing.
//
// Run: cargo kani --harness <name>
// CI:  .github/workflows/formal-verification-rust.yml

#[cfg(kani)]
mod ternary_proofs {
    use crate::ternary::{Trit, KernelTritExt, pack_trits, unpack_trits, Representation, convert_representation};

    /// PROOF: Trit::from_a rejects all invalid inputs
    #[kani::proof]
    fn proof_trit_from_a_rejects_invalid() {
        let val: i8 = kani::any();
        let result = Trit::from_a(val);
        if val == -1 || val == 0 || val == 1 {
            assert!(result.is_some(), "Valid trit value must be accepted");
            assert_eq!(result.unwrap().to_a(), val);
        } else {
            assert!(result.is_none(), "Invalid trit value must be rejected");
        }
    }

    /// PROOF: Trit::from_b rejects all invalid inputs
    #[kani::proof]
    fn proof_trit_from_b_rejects_invalid() {
        let val: u8 = kani::any();
        let result = Trit::from_b(val);
        if val <= 2 {
            assert!(result.is_some());
        } else {
            assert!(result.is_none());
        }
    }

    /// PROOF: Trit::from_c rejects all invalid inputs
    #[kani::proof]
    fn proof_trit_from_c_rejects_invalid() {
        let val: u8 = kani::any();
        let result = Trit::from_c(val);
        if val >= 1 && val <= 3 {
            assert!(result.is_some());
        } else {
            assert!(result.is_none());
        }
    }

    /// PROOF: NOT is an involution — NOT(NOT(x)) == x for ALL trits
    #[kani::proof]
    fn proof_not_involution() {
        let val: i8 = kani::any();
        kani::assume(val >= -1 && val <= 1);
        let t = Trit::from_a(val).unwrap();
        assert_eq!(t.not().not().to_a(), t.to_a(), "NOT must be an involution");
    }

    /// PROOF: NOT negates — NOT(x) == -x for ALL trits
    #[kani::proof]
    fn proof_not_negates() {
        let val: i8 = kani::any();
        kani::assume(val >= -1 && val <= 1);
        let t = Trit::from_a(val).unwrap();
        assert_eq!(t.not().to_a(), -val, "NOT must negate the value");
    }

    /// PROOF: AND is commutative for ALL trit pairs
    #[kani::proof]
    fn proof_and_commutative() {
        let a: i8 = kani::any();
        let b: i8 = kani::any();
        kani::assume(a >= -1 && a <= 1);
        kani::assume(b >= -1 && b <= 1);
        let ta = Trit::from_a(a).unwrap();
        let tb = Trit::from_a(b).unwrap();
        assert_eq!(ta.and(&tb).to_a(), tb.and(&ta).to_a());
    }

    /// PROOF: OR is commutative for ALL trit pairs
    #[kani::proof]
    fn proof_or_commutative() {
        let a: i8 = kani::any();
        let b: i8 = kani::any();
        kani::assume(a >= -1 && a <= 1);
        kani::assume(b >= -1 && b <= 1);
        let ta = Trit::from_a(a).unwrap();
        let tb = Trit::from_a(b).unwrap();
        assert_eq!(ta.or(&tb).to_a(), tb.or(&ta).to_a());
    }

    /// PROOF: GF(3) addition is closed — result is always a valid trit
    #[kani::proof]
    fn proof_gf3_add_closure() {
        let a: i8 = kani::any();
        let b: i8 = kani::any();
        kani::assume(a >= -1 && a <= 1);
        kani::assume(b >= -1 && b <= 1);
        let ta = Trit::from_a(a).unwrap();
        let tb = Trit::from_a(b).unwrap();
        let result = ta.add(tb).to_a();
        assert!(result >= -1 && result <= 1, "GF(3) add must produce valid trit");
    }

    /// PROOF: GF(3) addition identity — a + 0 == a
    #[kani::proof]
    fn proof_gf3_add_identity() {
        let a: i8 = kani::any();
        kani::assume(a >= -1 && a <= 1);
        let ta = Trit::from_a(a).unwrap();
        let zero = Trit::from_a(0).unwrap();
        assert_eq!(ta.add(zero).to_a(), a, "Zero must be additive identity");
    }

    /// PROOF: GF(3) addition is commutative — a + b == b + a
    #[kani::proof]
    fn proof_gf3_add_commutative() {
        let a: i8 = kani::any();
        let b: i8 = kani::any();
        kani::assume(a >= -1 && a <= 1);
        kani::assume(b >= -1 && b <= 1);
        let ta = Trit::from_a(a).unwrap();
        let tb = Trit::from_a(b).unwrap();
        assert_eq!(ta.add(tb).to_a(), tb.add(ta).to_a());
    }

    /// PROOF: GF(3) multiplication is closed
    #[kani::proof]
    fn proof_gf3_mul_closure() {
        let a: i8 = kani::any();
        let b: i8 = kani::any();
        kani::assume(a >= -1 && a <= 1);
        kani::assume(b >= -1 && b <= 1);
        let ta = Trit::from_a(a).unwrap();
        let tb = Trit::from_a(b).unwrap();
        let result = ta.multiply(&tb).to_a();
        assert!(result >= -1 && result <= 1, "GF(3) mul must produce valid trit");
    }

    /// PROOF: GF(3) multiplication is commutative
    #[kani::proof]
    fn proof_gf3_mul_commutative() {
        let a: i8 = kani::any();
        let b: i8 = kani::any();
        kani::assume(a >= -1 && a <= 1);
        kani::assume(b >= -1 && b <= 1);
        let ta = Trit::from_a(a).unwrap();
        let tb = Trit::from_a(b).unwrap();
        assert_eq!(ta.multiply(&tb).to_a(), tb.multiply(&ta).to_a());
    }

    /// PROOF: GF(3) multiplication identity — a * 1 == a
    #[kani::proof]
    fn proof_gf3_mul_identity() {
        let a: i8 = kani::any();
        kani::assume(a >= -1 && a <= 1);
        let ta = Trit::from_a(a).unwrap();
        let one = Trit::from_a(1).unwrap();
        assert_eq!(ta.multiply(&one).to_a(), a, "One must be multiplicative identity");
    }

    /// PROOF: GF(3) zero annihilation — a * 0 == 0
    #[kani::proof]
    fn proof_gf3_mul_zero() {
        let a: i8 = kani::any();
        kani::assume(a >= -1 && a <= 1);
        let ta = Trit::from_a(a).unwrap();
        let zero = Trit::from_a(0).unwrap();
        assert_eq!(ta.multiply(&zero).to_a(), 0, "Zero must annihilate");
    }

    /// PROOF: Rotation cycles — rotate(rotate(rotate(x))) == x
    #[kani::proof]
    fn proof_rotate_cycles() {
        let a: i8 = kani::any();
        kani::assume(a >= -1 && a <= 1);
        let t = Trit::from_a(a).unwrap();
        let r3 = t.rotate().rotate().rotate();
        assert_eq!(r3.to_a(), t.to_a(), "Triple rotation must be identity");
    }

    /// PROOF: Rotate and rotate_inverse are inverses
    #[kani::proof]
    fn proof_rotate_inverse() {
        let a: i8 = kani::any();
        kani::assume(a >= -1 && a <= 1);
        let t = Trit::from_a(a).unwrap();
        assert_eq!(t.rotate().rotate_inverse().to_a(), t.to_a());
        assert_eq!(t.rotate_inverse().rotate().to_a(), t.to_a());
    }

    /// PROOF: Bijection A→B→A round-trip for ALL valid inputs
    #[kani::proof]
    fn proof_bijection_a_b_roundtrip() {
        let a: i8 = kani::any();
        kani::assume(a >= -1 && a <= 1);
        let t = Trit::from_a(a).unwrap();
        let b = t.to_b();
        let back = Trit::from_b(b).unwrap();
        assert_eq!(back.to_a(), a, "A->B->A must round-trip");
    }

    /// PROOF: Bijection A→C→A round-trip for ALL valid inputs
    #[kani::proof]
    fn proof_bijection_a_c_roundtrip() {
        let a: i8 = kani::any();
        kani::assume(a >= -1 && a <= 1);
        let t = Trit::from_a(a).unwrap();
        let c = t.to_c();
        let back = Trit::from_c(c).unwrap();
        assert_eq!(back.to_a(), a, "A->C->A must round-trip");
    }

    /// PROOF: convert_representation A→B→C→A round-trip
    #[kani::proof]
    fn proof_convert_representation_roundtrip() {
        let a: i8 = kani::any();
        kani::assume(a >= -1 && a <= 1);
        let b = convert_representation(a, Representation::A, Representation::B);
        let c = convert_representation(b, Representation::B, Representation::C);
        let back = convert_representation(c, Representation::C, Representation::A);
        assert_eq!(back, a, "A->B->C->A must round-trip");
    }

    /// PROOF: pack_trits/unpack_trits round-trip (bounded to 9 trits)
    #[kani::proof]
    #[kani::unwind(28)]
    fn proof_pack_unpack_roundtrip_9() {
        let mut trits = [Trit::from_a(0).unwrap(); 9];
        for i in 0..9 {
            let val: i8 = kani::any();
            kani::assume(val >= -1 && val <= 1);
            trits[i] = Trit::from_a(val).unwrap();
        }
        let packed = pack_trits(&trits);
        let unpacked = unpack_trits(packed);
        for i in 0..9 {
            assert_eq!(trits[i].to_a(), unpacked[i].to_a(),
                "Pack/unpack must round-trip at every position");
        }
    }
}

#[cfg(kani)]
mod timing_proofs {
    use crate::timing::{FemtosecondTimestamp, FS_PER_NS, FS_PER_MS, FS_PER_SECOND};
    use crate::SALVI_EPOCH_NS;

    /// PROOF: Timestamp seconds() never overflows for valid inputs
    #[kani::proof]
    fn proof_timestamp_seconds_no_overflow() {
        let fs: u128 = kani::any();
        // Bound to 1000 years of femtoseconds (realistic range)
        kani::assume(fs <= 1000 * 365 * 24 * 3600 * FS_PER_SECOND);
        let ts = FemtosecondTimestamp::new(fs);
        let _secs = ts.seconds(); // Must not panic
    }

    /// PROOF: sub_second_fs is always < 1 second
    #[kani::proof]
    fn proof_sub_second_bounded() {
        let fs: u128 = kani::any();
        kani::assume(fs <= 100 * FS_PER_SECOND);
        let ts = FemtosecondTimestamp::new(fs);
        assert!(ts.sub_second_fs() < FS_PER_SECOND,
            "Sub-second component must be less than 1 second");
    }

    /// PROOF: milliseconds() is always < 1000
    #[kani::proof]
    fn proof_milliseconds_bounded() {
        let fs: u128 = kani::any();
        kani::assume(fs <= 100 * FS_PER_SECOND);
        let ts = FemtosecondTimestamp::new(fs);
        assert!(ts.milliseconds() < 1000,
            "Milliseconds component must be < 1000");
    }

    /// PROOF: Precision constant relationships are correct
    #[kani::proof]
    fn proof_precision_constants() {
        assert_eq!(FS_PER_NS, 1_000_000);
        assert_eq!(FS_PER_MS, 1_000 * FS_PER_NS * 1_000);
        assert_eq!(FS_PER_SECOND, 1_000 * FS_PER_MS);
    }
}

#[cfg(kani)]
mod boot_proofs {
    use crate::arch::boot::{BootSequence, BootStage};
    use crate::arch::ArchId;

    /// PROOF: Boot sequence requires exactly 11 advances to reach Running
    #[kani::proof]
    #[kani::unwind(13)]
    fn proof_boot_sequence_length() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        let mut count = 0u32;
        while !seq.is_complete() {
            assert!(seq.advance().is_ok());
            count += 1;
            if count > 12 { break; } // safety bound
        }
        assert_eq!(count, 11, "Must take exactly 11 advances");
        assert!(seq.is_complete());
    }

    /// PROOF: Cannot advance past Running
    #[kani::proof]
    #[kani::unwind(13)]
    fn proof_boot_cannot_overrun() {
        let mut seq = BootSequence::new(ArchId::X86_64);
        for _ in 0..11 {
            let _ = seq.advance();
        }
        assert!(seq.is_complete());
        assert!(seq.advance().is_err(), "Must not advance past Running");
    }
}

#[cfg(kani)]
mod phase_proofs {
    use crate::phase::EncryptionMode;

    /// PROOF: All encryption modes have phase_count >= 3
    #[kani::proof]
    fn proof_phase_count_minimum() {
        let modes = [
            EncryptionMode::HighSecurity,
            EncryptionMode::Balanced,
            EncryptionMode::Performance,
            EncryptionMode::Adaptive,
        ];
        for mode in &modes {
            assert!(mode.phase_count() >= 3,
                "Every mode must split into at least 3 phases");
        }
    }

    /// PROOF: split_ratio is always in (0, 1) — valid proportion
    #[kani::proof]
    fn proof_split_ratio_bounded() {
        let modes = [
            EncryptionMode::HighSecurity,
            EncryptionMode::Balanced,
            EncryptionMode::Performance,
            EncryptionMode::Adaptive,
        ];
        for mode in &modes {
            let ratio = mode.split_ratio();
            assert!(ratio > 0.0 && ratio < 1.0,
                "Split ratio must be a valid proportion");
        }
    }
}
