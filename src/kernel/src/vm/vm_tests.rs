// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// All Rights Reserved.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

#[cfg(test)]
mod property_tests {
    use crate::ternary::*;
    use crate::vm::engine::TernaryVm;
    use crate::vm::instruction::*;
    use crate::timing::SimulatedHptp;
    use alloc::boxed::Box;

    const TRIT_VALUES: [i8; 3] = [-1, 0, 1];

    #[test]
    fn test_gf3_add_commutativity() {
        for &a in &TRIT_VALUES {
            for &b in &TRIT_VALUES {
                let ta = Trit::from_a(a).unwrap();
                let tb = Trit::from_a(b).unwrap();
                assert_eq!(ta.add(tb).to_a(), tb.add(ta).to_a(),
                    "Add commutativity failed for a={}, b={}", a, b);
            }
        }
    }

    #[test]
    fn test_gf3_mul_commutativity() {
        for &a in &TRIT_VALUES {
            for &b in &TRIT_VALUES {
                let ta = Trit::from_a(a).unwrap();
                let tb = Trit::from_a(b).unwrap();
                assert_eq!(ta.multiply(&tb).to_a(), tb.multiply(&ta).to_a(),
                    "Mul commutativity failed for a={}, b={}", a, b);
            }
        }
    }

    #[test]
    fn test_gf3_add_associativity() {
        for &a in &TRIT_VALUES {
            for &b in &TRIT_VALUES {
                for &c in &TRIT_VALUES {
                    let ta = Trit::from_a(a).unwrap();
                    let tb = Trit::from_a(b).unwrap();
                    let tc = Trit::from_a(c).unwrap();
                    let lhs = ta.add(tb).add(tc);
                    let rhs = ta.add(tb.add(tc));
                    assert_eq!(lhs.to_a(), rhs.to_a(),
                        "Add associativity failed for a={}, b={}, c={}", a, b, c);
                }
            }
        }
    }

    #[test]
    fn test_gf3_mul_associativity() {
        for &a in &TRIT_VALUES {
            for &b in &TRIT_VALUES {
                for &c in &TRIT_VALUES {
                    let ta = Trit::from_a(a).unwrap();
                    let tb = Trit::from_a(b).unwrap();
                    let tc = Trit::from_a(c).unwrap();
                    let lhs = ta.multiply(&tb).multiply(&tc);
                    let rhs = ta.multiply(&tb.multiply(&tc));
                    assert_eq!(lhs.to_a(), rhs.to_a(),
                        "Mul associativity failed for a={}, b={}, c={}", a, b, c);
                }
            }
        }
    }

    #[test]
    fn test_gf3_additive_identity() {
        let zero = Trit::from_a(0).unwrap();
        for &a in &TRIT_VALUES {
            let ta = Trit::from_a(a).unwrap();
            assert_eq!(ta.add(zero).to_a(), a, "0 is not additive identity for a={}", a);
        }
    }

    #[test]
    fn test_gf3_multiplicative_identity() {
        let one = Trit::from_a(1).unwrap();
        for &a in &TRIT_VALUES {
            let ta = Trit::from_a(a).unwrap();
            assert_eq!(ta.multiply(&one).to_a(), a, "1 is not multiplicative identity for a={}", a);
        }
    }

    #[test]
    fn test_gf3_additive_inverse() {
        for &a in &TRIT_VALUES {
            let ta = Trit::from_a(a).unwrap();
            let neg_a = ta.not();
            assert_eq!(ta.add(neg_a).to_a(), 0,
                "a + (-a) != 0 for a={}", a);
        }
    }

    #[test]
    fn test_gf3_multiplicative_inverse() {
        for &a in &[1i8, -1] {
            let ta = Trit::from_a(a).unwrap();
            let inv = ta.gf3_inverse_unchecked();
            assert_eq!(ta.multiply(&inv).to_a(), 1,
                "a * inv(a) != 1 for a={}", a);
        }
    }

    #[test]
    fn test_gf3_distributivity() {
        for &a in &TRIT_VALUES {
            for &b in &TRIT_VALUES {
                for &c in &TRIT_VALUES {
                    let ta = Trit::from_a(a).unwrap();
                    let tb = Trit::from_a(b).unwrap();
                    let tc = Trit::from_a(c).unwrap();
                    let lhs = ta.multiply(&tb.add(tc));
                    let rhs = ta.multiply(&tb).add(ta.multiply(&tc));
                    assert_eq!(lhs.to_a(), rhs.to_a(),
                        "Distributivity failed for a={}, b={}, c={}", a, b, c);
                }
            }
        }
    }

    #[test]
    fn test_gf3_double_negation() {
        for &a in &TRIT_VALUES {
            let ta = Trit::from_a(a).unwrap();
            assert_eq!(ta.not().not().to_a(), a, "Double negation failed for a={}", a);
        }
    }

    #[test]
    fn test_kleene_xor_idempotent() {
        for &a in &TRIT_VALUES {
            let ta = Trit::from_a(a).unwrap();
            assert_eq!(ta.xor(&ta).to_a(), a, "min(a,a) should be a for a={}", a);
        }
    }

    #[test]
    fn test_kleene_or_idempotent() {
        for &a in &TRIT_VALUES {
            let ta = Trit::from_a(a).unwrap();
            assert_eq!(ta.or(&ta).to_a(), a, "max(a,a) should be a for a={}", a);
        }
    }

    #[test]
    fn test_rotation_cycle() {
        for &a in &TRIT_VALUES {
            let ta = Trit::from_a(a).unwrap();
            let rotated = ta.rotate().rotate().rotate();
            assert_eq!(rotated.to_a(), a, "Triple rotation should be identity for a={}", a);
        }
    }

    #[test]
    fn test_rotation_inverse_cancels() {
        for &a in &TRIT_VALUES {
            let ta = Trit::from_a(a).unwrap();
            assert_eq!(ta.rotate().rotate_inverse().to_a(), a,
                "rot(rot_inv(a)) should be a for a={}", a);
            assert_eq!(ta.rotate_inverse().rotate().to_a(), a,
                "rot_inv(rot(a)) should be a for a={}", a);
        }
    }

    #[test]
    fn test_pack_unpack_roundtrip() {
        for &a in &TRIT_VALUES {
            for &b in &TRIT_VALUES {
                let trits = [Trit::from_a(a).unwrap(), Trit::from_a(b).unwrap()];
                let packed = pack_trits(&trits);
                let unpacked = unpack_trits(packed);
                assert_eq!(unpacked[0].to_a(), a);
                assert_eq!(unpacked[1].to_a(), b);
            }
        }
    }

    #[test]
    fn test_representation_roundtrip() {
        for &a in &TRIT_VALUES {
            let t = Trit::from_a(a).unwrap();
            let b = t.to_b();
            let back = Trit::from_b(b).unwrap();
            assert_eq!(back.to_a(), a, "A->B->A roundtrip failed for a={}", a);
            let c = t.to_c();
            let back2 = Trit::from_c(c).unwrap();
            assert_eq!(back2.to_a(), a, "A->C->A roundtrip failed for a={}", a);
        }
    }

    #[test]
    fn test_fuzz_all_binary_ops() {
        for &a in &TRIT_VALUES {
            for &b in &TRIT_VALUES {
                let ta = Trit::from_a(a).unwrap();
                let tb = Trit::from_a(b).unwrap();

                let add_result = ta.add(tb).to_a();
                assert!(add_result >= -1 && add_result <= 1, "add({},{}) out of range", a, b);

                let mul_result = ta.multiply(&tb).to_a();
                assert!(mul_result >= -1 && mul_result <= 1, "mul({},{}) out of range", a, b);

                let xor_result = ta.xor(&tb).to_a();
                assert!(xor_result >= -1 && xor_result <= 1, "xor({},{}) out of range", a, b);

                let and_result = ta.lukasiewicz_and(&tb).to_a();
                assert!(and_result >= -1 && and_result <= 1, "and({},{}) out of range", a, b);

                let or_result = ta.or(&tb).to_a();
                assert!(or_result >= -1 && or_result <= 1, "or({},{}) out of range", a, b);

                let sub_result = ta.sub(tb).to_a();
                assert!(sub_result >= -1 && sub_result <= 1, "sub({},{}) out of range", a, b);

                let cmp_result = ta.cmp_trit(&tb).to_a();
                assert!(cmp_result >= -1 && cmp_result <= 1, "cmp({},{}) out of range", a, b);
            }
        }
    }

    #[test]
    fn test_fuzz_all_unary_ops() {
        for &a in &TRIT_VALUES {
            let ta = Trit::from_a(a).unwrap();

            let neg = ta.not().to_a();
            assert!(neg >= -1 && neg <= 1, "not({}) out of range", a);

            let rot = ta.rotate().to_a();
            assert!(rot >= -1 && rot <= 1, "rotate({}) out of range", a);

            let roti = ta.rotate_inverse().to_a();
            assert!(roti >= -1 && roti <= 1, "rotate_inverse({}) out of range", a);

            if a != 0 {
                let inv = ta.gf3_inverse_unchecked().to_a();
                assert!(inv >= -1 && inv <= 1, "gf3_inverse({}) out of range", a);
            }
        }
    }

    #[test]
    fn test_fuzz_packed_operations() {
        for &a in &TRIT_VALUES {
            for &b in &TRIT_VALUES {
                let pa = pack_trits(&[Trit::from_a(a).unwrap()]);
                let pb = pack_trits(&[Trit::from_a(b).unwrap()]);

                let add = packed_zip(pa, pb, |x, y| x.add(*y));
                assert!(is_valid_packed(add), "packed add invalid for a={}, b={}", a, b);

                let mul = packed_zip(pa, pb, |x, y| x.multiply(y));
                assert!(is_valid_packed(mul), "packed mul invalid for a={}, b={}", a, b);

                let neg = packed_map(pa, |t| t.not());
                assert!(is_valid_packed(neg), "packed neg invalid for a={}", a);
            }
        }
    }

    #[test]
    fn test_benchmark_add_throughput() {
        let mut vm = TernaryVm::new(4096, Box::new(SimulatedHptp::new()));
        let mut prog = Program::new("bench_add");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        for _ in 0..100 {
            prog.add_instruction(Instruction::new(Opcode::TAdd, 2, 0, 1, 0));
        }
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        let cycles = vm.run().unwrap();
        assert_eq!(cycles, 103);
    }

    #[test]
    fn test_benchmark_mul_throughput() {
        let mut vm = TernaryVm::new(4096, Box::new(SimulatedHptp::new()));
        let mut prog = Program::new("bench_mul");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        for _ in 0..100 {
            prog.add_instruction(Instruction::new(Opcode::TMul, 2, 0, 1, 0));
        }
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        let cycles = vm.run().unwrap();
        assert_eq!(cycles, 103);
    }

    #[test]
    fn test_benchmark_hash_throughput() {
        let mut vm = TernaryVm::new(4096, Box::new(SimulatedHptp::new()));
        let mut prog = Program::new("bench_hash");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 42));
        for _ in 0..50 {
            prog.add_instruction(Instruction::new(Opcode::THash, 0, 0, 0, 0));
        }
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        let cycles = vm.run().unwrap();
        assert_eq!(cycles, 52);
    }

    #[test]
    fn test_benchmark_simd_throughput() {
        let mut vm = TernaryVm::new(4096, Box::new(SimulatedHptp::new()));
        let mut prog = Program::new("bench_simd");
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 0, 0, 0, 1));
        prog.add_instruction(Instruction::new(Opcode::LoadImm, 1, 0, 0, 1));
        for _ in 0..50 {
            prog.add_instruction(Instruction::new(Opcode::TAddV, 2, 0, 1, 0));
            prog.add_instruction(Instruction::new(Opcode::TMulV, 3, 0, 1, 0));
        }
        prog.add_instruction(Instruction::from_opcode(Opcode::Halt));
        vm.load_program(prog).unwrap();
        let cycles = vm.run().unwrap();
        assert_eq!(cycles, 103);
    }
}
