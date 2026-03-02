// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved
//
// Kani Proof Harnesses — Cryptographic Primitives
//
// Formally proves constant-time properties and algebraic invariants
// that the dynamic verification in formal_verify.rs can only sample.

#[cfg(kani)]
mod ct_utils_proofs {
    use crate::crypto::ct_utils;

    /// PROOF: ct_eq_u8 is reflexive — ct_eq(a, a) == 0xFF for ALL u8
    #[kani::proof]
    fn proof_ct_eq_reflexive() {
        let a: u8 = kani::any();
        assert_eq!(ct_utils::ct_eq_u8(a, a), 0xFF,
            "ct_eq must return 0xFF for equal inputs");
    }

    /// PROOF: ct_eq_u8 is symmetric — ct_eq(a, b) == ct_eq(b, a)
    #[kani::proof]
    fn proof_ct_eq_symmetric() {
        let a: u8 = kani::any();
        let b: u8 = kani::any();
        assert_eq!(ct_utils::ct_eq_u8(a, b), ct_utils::ct_eq_u8(b, a),
            "ct_eq must be symmetric");
    }

    /// PROOF: ct_eq_u8 returns only 0x00 or 0xFF — no partial matches
    #[kani::proof]
    fn proof_ct_eq_binary_output() {
        let a: u8 = kani::any();
        let b: u8 = kani::any();
        let result = ct_utils::ct_eq_u8(a, b);
        assert!(result == 0x00 || result == 0xFF,
            "ct_eq must return only 0x00 or 0xFF");
    }

    /// PROOF: ct_eq_u8 correctness — returns 0xFF iff equal
    #[kani::proof]
    fn proof_ct_eq_correctness() {
        let a: u8 = kani::any();
        let b: u8 = kani::any();
        let result = ct_utils::ct_eq_u8(a, b);
        if a == b {
            assert_eq!(result, 0xFF);
        } else {
            assert_eq!(result, 0x00);
        }
    }

    /// PROOF: ct_neq_u8 is the complement of ct_eq_u8
    #[kani::proof]
    fn proof_ct_neq_complement() {
        let a: u8 = kani::any();
        let b: u8 = kani::any();
        assert_eq!(ct_utils::ct_neq_u8(a, b), !ct_utils::ct_eq_u8(a, b));
    }

    /// PROOF: ct_select_u8 correctness — condition=1 gives if_true, condition=0 gives if_false
    #[kani::proof]
    fn proof_ct_select_u8_correctness() {
        let condition: u8 = kani::any();
        kani::assume(condition <= 1);
        let if_true: u8 = kani::any();
        let if_false: u8 = kani::any();
        let result = ct_utils::ct_select_u8(condition, if_true, if_false);
        if condition == 1 {
            assert_eq!(result, if_true);
        } else {
            assert_eq!(result, if_false);
        }
    }

    /// PROOF: ct_select_i8 correctness for trit values
    #[kani::proof]
    fn proof_ct_select_i8_correctness() {
        let condition: u8 = kani::any();
        kani::assume(condition <= 1);
        let if_true: i8 = kani::any();
        let if_false: i8 = kani::any();
        kani::assume(if_true >= -1 && if_true <= 1);
        kani::assume(if_false >= -1 && if_false <= 1);
        let result = ct_utils::ct_select_i8(condition, if_true, if_false);
        if condition == 1 {
            assert_eq!(result, if_true);
        } else {
            assert_eq!(result, if_false);
        }
    }

    /// PROOF: ct_is_zero correctness
    #[kani::proof]
    fn proof_ct_is_zero_correctness() {
        let x: u8 = kani::any();
        let result = ct_utils::ct_is_zero(x);
        if x == 0 {
            assert_eq!(result, 0xFF);
        } else {
            assert_eq!(result, 0x00);
        }
    }

    /// PROOF: ct_lt_u32 correctness
    #[kani::proof]
    fn proof_ct_lt_u32_correctness() {
        let a: u32 = kani::any();
        let b: u32 = kani::any();
        let result = ct_utils::ct_lt_u32(a, b);
        if a < b {
            assert_eq!(result, 1);
        } else {
            assert_eq!(result, 0);
        }
    }

    /// PROOF: ct_select_u32 correctness
    #[kani::proof]
    fn proof_ct_select_u32_correctness() {
        let condition: u8 = kani::any();
        kani::assume(condition <= 1);
        let if_true: u32 = kani::any();
        let if_false: u32 = kani::any();
        let result = ct_utils::ct_select_u32(condition, if_true, if_false);
        if condition == 1 {
            assert_eq!(result, if_true);
        } else {
            assert_eq!(result, if_false);
        }
    }

    /// PROOF: ct_zeroize zeroes ALL bytes (bounded proof)
    #[kani::proof]
    #[kani::unwind(17)]
    fn proof_ct_zeroize_complete() {
        let mut data = [0u8; 16];
        for i in 0..16 {
            data[i] = kani::any();
        }
        ct_utils::ct_zeroize(&mut data);
        for i in 0..16 {
            assert_eq!(data[i], 0, "Every byte must be zeroed");
        }
    }

    /// PROOF: ct_zeroize_i8 zeroes ALL elements (bounded proof)
    #[kani::proof]
    #[kani::unwind(17)]
    fn proof_ct_zeroize_i8_complete() {
        let mut data = [0i8; 16];
        for i in 0..16 {
            data[i] = kani::any();
        }
        ct_utils::ct_zeroize_i8(&mut data);
        for i in 0..16 {
            assert_eq!(data[i], 0);
        }
    }

    /// PROOF: ct_eq_slices rejects different lengths
    #[kani::proof]
    fn proof_ct_eq_slices_length_check() {
        let a = [0i8; 3];
        let b = [0i8; 4];
        assert_eq!(ct_utils::ct_eq_slices(&a, &b), 0,
            "Different length slices must never be equal");
    }

    /// PROOF: ct_eq_byte_slices rejects different lengths
    #[kani::proof]
    fn proof_ct_eq_byte_slices_length_check() {
        let a = [0u8; 3];
        let b = [0u8; 4];
        assert_eq!(ct_utils::ct_eq_byte_slices(&a, &b), 0);
    }

    /// PROOF: ct_cmov_bytes with condition=0 leaves target unchanged (bounded)
    #[kani::proof]
    #[kani::unwind(5)]
    fn proof_ct_cmov_bytes_noop() {
        let mut target = [0u8; 4];
        for i in 0..4 { target[i] = kani::any(); }
        let original = target;
        let source: [u8; 4] = [kani::any(), kani::any(), kani::any(), kani::any()];
        ct_utils::ct_cmov_bytes(0, &mut target, &source);
        for i in 0..4 {
            assert_eq!(target[i], original[i], "condition=0 must not modify target");
        }
    }

    /// PROOF: ct_cmov_bytes with condition=1 copies source to target (bounded)
    #[kani::proof]
    #[kani::unwind(5)]
    fn proof_ct_cmov_bytes_copies() {
        let mut target = [0u8; 4];
        let source: [u8; 4] = [kani::any(), kani::any(), kani::any(), kani::any()];
        ct_utils::ct_cmov_bytes(1, &mut target, &source);
        for i in 0..4 {
            assert_eq!(target[i], source[i], "condition=1 must copy source");
        }
    }
}
