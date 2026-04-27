// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL - All Rights Reserved.
// Patent(s) Pending.
//
// This file is part of the Salvi Framework / PlenumNET platform.
// Unauthorized copying, modification, distribution, or use of this file,
// via any medium, is strictly prohibited without the prior written
// permission of Capomastro Holdings Ltd.
//
// See LICENSE in the repository root for full terms.

//! Salvi Framework Performance Benchmark Suite
//!
//! Measures three quantitative claims:
//!
//! 1. **TVM Throughput** — ops/sec by security tier (Fortified/Verified/Basic)
//! 2. **Ternary vs Binary Efficiency** — GF(3) vs GF(2) operation cost ratio
//! 3. **Information Density** — trit-per-bit storage density
//!
//! # Running
//!
//! ```sh
//! cd src/kernel
//! cargo run --bin salvi-bench --features bench-tools --release
//! ```

use std::hint::black_box;
use std::time::{Duration, Instant};

use plenumnet_kernel::ternary::{Trit, KernelTritExt, Representation, convert_representation, pack_trits, unpack_trits};
use plenumnet_kernel::crypto::sponge::TernarySponge;
use plenumnet_kernel::crypto::tl_kem::{self, TlKemVariant};
use plenumnet_kernel::crypto::tl_dsa::{self, TlDsaVariant};
use plenumnet_kernel::crypto::ternary_lattice::TernaryPolynomial;
use plenumnet_kernel::vm::engine::TernaryVm;
use plenumnet_kernel::vm::instruction::{Instruction, Opcode, Program};
use plenumnet_kernel::timing::SimulatedHptp;

const WARMUP_ITERS: u32 = 5;
const MIN_SAMPLES: u32 = 20;
const MIN_DURATION_MS: u128 = 200;

fn bench<F: FnMut()>(name: &str, mut f: F) -> (f64, Duration) {
    for _ in 0..WARMUP_ITERS { f(); }

    let mut total = Duration::ZERO;
    let mut iters = 0u64;
    while iters < MIN_SAMPLES as u64 || total.as_millis() < MIN_DURATION_MS {
        let start = Instant::now();
        f();
        total += start.elapsed();
        iters += 1;
    }

    let avg = total / iters as u32;
    let avg_ns = avg.as_nanos() as f64;
    let ops_per_sec = if avg_ns > 0.0 { 1_000_000_000.0 / avg_ns } else { f64::INFINITY };

    println!("  {:<50} {:>12.1} ns/op  {:>12.0} ops/sec  ({} samples)",
        name, avg_ns, ops_per_sec, iters);
    (avg_ns, avg)
}

fn bench_n<F: FnMut()>(name: &str, ops_per_call: u64, mut f: F) -> f64 {
    for _ in 0..WARMUP_ITERS { f(); }

    let mut total = Duration::ZERO;
    let mut iters = 0u64;
    while iters < MIN_SAMPLES as u64 || total.as_millis() < MIN_DURATION_MS {
        let start = Instant::now();
        f();
        total += start.elapsed();
        iters += 1;
    }

    let total_ops = iters * ops_per_call;
    let ns_per_op = total.as_nanos() as f64 / total_ops as f64;
    let ops_per_sec = if ns_per_op > 0.0 { 1_000_000_000.0 / ns_per_op } else { f64::INFINITY };

    println!("  {:<50} {:>12.1} ns/op  {:>12.0} ops/sec  ({} × {} ops)",
        name, ns_per_op, ops_per_sec, iters, ops_per_call);
    ns_per_op
}

fn make_program(opcodes: &[(Opcode, u8, u8, u8)]) -> Program {
    let mut prog = Program::new("bench");
    for &(op, dst, s1, s2) in opcodes {
        prog.add_instruction(Instruction::new(op, dst, s1, s2, 0));
    }
    prog.add_instruction(Instruction::new(Opcode::Halt, 0, 0, 0, 0));
    prog
}

fn make_vm() -> TernaryVm {
    TernaryVm::new(65536, Box::new(SimulatedHptp::new()))
}

fn ternary_alu_program(n: usize) -> Program {
    let mut ops = Vec::with_capacity(n);
    for i in 0..n {
        let opcode = match i % 5 {
            0 => Opcode::TAdd,
            1 => Opcode::TMul,
            2 => Opcode::TNeg,
            3 => Opcode::TRot,
            _ => Opcode::TXor,
        };
        ops.push((opcode, 0, 1, 2));
    }
    make_program(&ops)
}

fn binary_alu_program(n: usize) -> Program {
    let mut ops = Vec::with_capacity(n);
    for i in 0..n {
        let opcode = match i % 4 {
            0 => Opcode::Add,
            1 => Opcode::Sub,
            2 => Opcode::Mul,
            _ => Opcode::Neg,
        };
        ops.push((opcode, 0, 1, 2));
    }
    make_program(&ops)
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════════════════════╗");
    println!("║         SALVI FRAMEWORK PERFORMANCE BENCHMARK SUITE                            ║");
    println!("║         PlenumNET Kernel · Post-Quantum Ternary Computing                      ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════════╝");
    println!();

    // ═══════════════════════════════════════════════════════════════════
    // GROUP 1: TVM THROUGHPUT BY SECURITY TIER
    // ═══════════════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("GROUP 1: TVM THROUGHPUT");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    println!("  1.1 — Raw TVM ALU throughput (ternary vs binary)");
    println!("  ────────────────────────────────────────────────────────────────");
    let batch = 1000usize;
    let t_prog = ternary_alu_program(batch);
    let b_prog = binary_alu_program(batch);

    let ternary_ns = bench_n("Ternary ALU (TAdd/TMul/TNeg/TRot/TXor) × 1000", batch as u64, || {
        let mut vm = make_vm();
        vm.load_program(t_prog.clone()).unwrap();
        vm.max_cycles = batch as u64 + 10;
        let _ = black_box(vm.run());
    });

    let binary_ns = bench_n("Binary ALU (Add/Sub/Mul/Neg) × 1000", batch as u64, || {
        let mut vm = make_vm();
        vm.load_program(b_prog.clone()).unwrap();
        vm.max_cycles = batch as u64 + 10;
        let _ = black_box(vm.run());
    });

    let ternary_binary_ratio = ternary_ns / binary_ns;
    println!();
    println!("  ► Ternary/Binary ALU cost ratio: {:.2}x", ternary_binary_ratio);
    println!();

    println!("  1.2 — Throughput by security tier");
    println!("  ────────────────────────────────────────────────────────────────");

    let seed: Vec<i8> = vec![0, 1, -1, 0, 1, -1, 0, 1, -1];
    let msg: Vec<i8> = vec![1, 0, -1, 1, 0, -1, 1, 0, -1];
    let rand_bytes: Vec<i8> = vec![1, 0, -1, 1, 0, -1, 1, 0];

    let (basic_ns, _) = bench("Basic tier (1000 ALU ops only)", || {
        let mut vm = make_vm();
        vm.load_program(t_prog.clone()).unwrap();
        vm.max_cycles = batch as u64 + 10;
        let _ = black_box(vm.run());
    });

    let (verified_ns, _) = bench("Verified tier (1000 ALU + sponge hash)", || {
        let mut vm = make_vm();
        vm.load_program(t_prog.clone()).unwrap();
        vm.max_cycles = batch as u64 + 10;
        let _ = vm.run();
        let mut sponge = TernarySponge::new();
        sponge.absorb(&msg);
        black_box(sponge.squeeze(243));
    });

    let (fortified_ns, _) = bench("Fortified tier (1000 ALU + TL-DSA-44 sign+verify)", || {
        let mut vm = make_vm();
        vm.load_program(t_prog.clone()).unwrap();
        vm.max_cycles = batch as u64 + 10;
        let _ = vm.run();
        let (pk, sk) = tl_dsa::keygen(TlDsaVariant::TlDsa44, &seed).unwrap();
        let sig = tl_dsa::sign(&sk, &msg).unwrap();
        black_box(tl_dsa::verify(&pk, &msg, &sig).unwrap());
    });

    println!();
    println!("  ► Verified overhead vs Basic:   {:.1}x ({:.0} ns added)",
        verified_ns / basic_ns, verified_ns - basic_ns);
    println!("  ► Fortified overhead vs Basic:   {:.1}x ({:.0} ns added)",
        fortified_ns / basic_ns, fortified_ns - basic_ns);
    println!();

    // ═══════════════════════════════════════════════════════════════════
    // GROUP 2: TERNARY VS BINARY EFFICIENCY
    // ═══════════════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("GROUP 2: TERNARY VS BINARY EFFICIENCY RATIO");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    println!("  2.1 — GF(3) vs GF(2) scalar operations");
    println!("  ────────────────────────────────────────────────────────────────");
    let scalar_ops = 10_000u64;
    let ta = Trit::from_a(1).unwrap();
    let tb = Trit::from_a(-1).unwrap();
    let ba: u8 = 0xAB;
    let bb: u8 = 0xCD;

    let gf3_add = bench_n("GF(3) trit add × 10k", scalar_ops, || {
        let mut acc = ta;
        for _ in 0..scalar_ops { acc = acc.add(tb); }
        black_box(acc);
    });

    let gf2_add = bench_n("GF(2) u8 wrapping_add × 10k", scalar_ops, || {
        let mut acc = ba;
        for _ in 0..scalar_ops { acc = acc.wrapping_add(bb); }
        black_box(acc);
    });

    let gf3_mul = bench_n("GF(3) trit mul × 10k", scalar_ops, || {
        let mut acc = ta;
        for _ in 0..scalar_ops { acc = acc.multiply(&tb); }
        black_box(acc);
    });

    let gf2_mul = bench_n("GF(2) u8 wrapping_mul × 10k", scalar_ops, || {
        let mut acc = ba;
        for _ in 0..scalar_ops { acc = acc.wrapping_mul(bb); }
        black_box(acc);
    });

    let gf3_not = bench_n("GF(3) trit not (negation) × 10k", scalar_ops, || {
        let mut acc = ta;
        for _ in 0..scalar_ops { acc = acc.not(); }
        black_box(acc);
    });

    let gf2_not = bench_n("GF(2) u8 bitwise NOT × 10k", scalar_ops, || {
        let mut acc = ba;
        for _ in 0..scalar_ops { acc = !acc; }
        black_box(acc);
    });

    println!();
    println!("  ► GF(3) add / GF(2) add cost ratio: {:.2}x", gf3_add / gf2_add);
    println!("  ► GF(3) mul / GF(2) mul cost ratio: {:.2}x", gf3_mul / gf2_mul);
    println!("  ► GF(3) not / GF(2) not cost ratio: {:.2}x", gf3_not / gf2_not);
    println!();

    println!("  2.2 — Ring multiplication: R₃[x]/(x^n+1) vs Z_q[x]/(x^n+1)");
    println!("  ────────────────────────────────────────────────────────────────");
    let poly_n = 256;
    let coeffs_a: Vec<i8> = (0..poly_n).map(|i| ((i % 3) as i8 - 1)).collect();
    let coeffs_b: Vec<i8> = (0..poly_n).map(|i| (((i + 1) % 3) as i8 - 1)).collect();
    let poly_a = TernaryPolynomial::from_coeffs_unchecked(coeffs_a.clone());
    let poly_b = TernaryPolynomial::from_coeffs_unchecked(coeffs_b.clone());

    let coeffs_u16_a: Vec<u16> = coeffs_a.iter().map(|&x| (x as i16 + 3329) as u16 % 3329).collect();
    let coeffs_u16_b: Vec<u16> = coeffs_b.iter().map(|&x| (x as i16 + 3329) as u16 % 3329).collect();

    let (r3_ns, _) = bench("R₃ schoolbook ring_mul n=256", || {
        black_box(poly_a.ring_mul(&poly_b).unwrap());
    });

    let (zq_ns, _) = bench("Z_3329 schoolbook ring_mul n=256", || {
        let q: u32 = 3329;
        let mut result = vec![0u32; poly_n];
        for i in 0..poly_n {
            if coeffs_u16_a[i] == 0 { continue; }
            for j in 0..poly_n {
                if coeffs_u16_b[j] == 0 { continue; }
                let product = coeffs_u16_a[i] as u32 * coeffs_u16_b[j] as u32;
                let pos = i + j;
                if pos < poly_n {
                    result[pos] = (result[pos] + product) % q;
                } else {
                    result[pos - poly_n] = (result[pos - poly_n] + q - (product % q)) % q;
                }
            }
        }
        black_box(result);
    });

    println!();
    println!("  ► R₃ / Z_q ring_mul cost ratio: {:.2}x", r3_ns / zq_ns);
    println!();

    println!("  2.3 — Hash comparison: TL-Sponge-385 vs SHA-384");
    println!("  ────────────────────────────────────────────────────────────────");

    for &input_len in &[16usize, 64, 243, 729] {
        let trit_input: Vec<i8> = (0..input_len).map(|i| ((i % 3) as i8 - 1)).collect();
        let byte_input: Vec<u8> = (0..input_len).map(|i| (i % 256) as u8).collect();

        let (sponge_ns, _) = bench(&format!("TL-Sponge-385 (input={} trits)", input_len), || {
            let mut sponge = TernarySponge::new();
            sponge.absorb(&trit_input);
            black_box(sponge.squeeze(243));
        });

        let (sha_ns, _) = bench(&format!("SHA-384 reference (input={} bytes)", input_len), || {
            black_box(plenumnet_kernel::crypto::sha2::sha384(&byte_input));
        });

        println!("    → sponge/SHA-384 ratio at len {}: {:.2}x", input_len, sponge_ns / sha_ns);
        println!();
    }

    println!("  2.4 — Post-quantum crypto: TL-KEM roundtrip");
    println!("  ────────────────────────────────────────────────────────────────");

    for &(variant, label) in &[
        (TlKemVariant::TlKem512, "TL-KEM-512"),
        (TlKemVariant::TlKem768, "TL-KEM-768"),
        (TlKemVariant::TlKem1024, "TL-KEM-1024"),
    ] {
        bench(&format!("{} keygen", label), || {
            black_box(tl_kem::keygen(variant, &seed).unwrap());
        });

        let (pk, sk) = tl_kem::keygen(variant, &seed).unwrap();
        bench(&format!("{} encapsulate", label), || {
            black_box(tl_kem::encapsulate(&pk, &rand_bytes).unwrap());
        });

        let (ct, _ss) = tl_kem::encapsulate(&pk, &rand_bytes).unwrap();
        bench(&format!("{} decapsulate", label), || {
            black_box(tl_kem::decapsulate(&sk, &ct).unwrap());
        });

        bench(&format!("{} full roundtrip (kg+enc+dec)", label), || {
            let (pk, sk) = tl_kem::keygen(variant, &seed).unwrap();
            let (ct, ss1) = tl_kem::encapsulate(&pk, &rand_bytes).unwrap();
            let ss2 = tl_kem::decapsulate(&sk, &ct).unwrap();
            black_box((ss1, ss2));
        });
        println!();
    }

    println!("  2.5 — Post-quantum crypto: TL-DSA roundtrip");
    println!("  ────────────────────────────────────────────────────────────────");

    for &(variant, label) in &[
        (TlDsaVariant::TlDsa44, "TL-DSA-44"),
        (TlDsaVariant::TlDsa65, "TL-DSA-65"),
        (TlDsaVariant::TlDsa87, "TL-DSA-87"),
    ] {
        bench(&format!("{} keygen", label), || {
            black_box(tl_dsa::keygen(variant, &seed).unwrap());
        });

        let (pk, sk) = tl_dsa::keygen(variant, &seed).unwrap();
        bench(&format!("{} sign", label), || {
            black_box(tl_dsa::sign(&sk, &msg).unwrap());
        });

        let sig = tl_dsa::sign(&sk, &msg).unwrap();
        bench(&format!("{} verify", label), || {
            black_box(tl_dsa::verify(&pk, &msg, &sig).unwrap());
        });

        bench(&format!("{} full roundtrip (kg+sign+verify)", label), || {
            let (pk, sk) = tl_dsa::keygen(variant, &seed).unwrap();
            let sig = tl_dsa::sign(&sk, &msg).unwrap();
            black_box(tl_dsa::verify(&pk, &msg, &sig).unwrap());
        });
        println!();
    }

    #[cfg(feature = "bench-tools")]
    {
        println!("  2.6 — TL-DSA timing breakdown (single-shot, per phase)");
        println!("  ────────────────────────────────────────────────────────────────");

        for &(variant, label) in &[
            (TlDsaVariant::TlDsa44, "TL-DSA-44"),
            (TlDsaVariant::TlDsa87, "TL-DSA-87"),
        ] {
            println!();
            println!("  {} breakdown:", label);
            let timings = tl_dsa::sign_verify_timing_breakdown(variant, &seed, &msg).unwrap();
            let total: std::time::Duration = timings.iter().map(|(_, d)| *d).sum();
            let total_us = total.as_micros() as f64;
            for (name, dur) in &timings {
                let us = dur.as_micros() as f64;
                let pct = if total_us > 0.0 { us / total_us * 100.0 } else { 0.0 };
                println!("    {:.<30} {:>10.0} µs  ({:>5.1}%)", name, us, pct);
            }
            println!("    {:.<30} {:>10.0} µs  (100.0%)", "TOTAL", total_us);
        }
        println!();
    }

    #[cfg(not(feature = "bench-tools"))]
    {
        println!("  2.6 — TL-DSA timing breakdown: skipped (requires --features bench-tools)");
        println!();
    }

    // ═══════════════════════════════════════════════════════════════════
    // GROUP 3: INFORMATION DENSITY
    // ═══════════════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("GROUP 3: INFORMATION DENSITY");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    println!("  3.1 — Trit packing/unpacking");
    println!("  ────────────────────────────────────────────────────────────────");

    let trits: Vec<Trit> = (0..27).map(|i| Trit::from_a((i % 3) as i8 - 1).unwrap()).collect();

    bench_n("pack_trits (27 trits → i64) × 1000", 1000, || {
        for _ in 0..1000 { black_box(pack_trits(&trits)); }
    });

    let packed = pack_trits(&trits);
    bench_n("unpack_trits (i64 → 27 trits) × 1000", 1000, || {
        for _ in 0..1000 { black_box(unpack_trits(packed)); }
    });

    bench_n("pack → unpack roundtrip × 1000", 1000, || {
        for _ in 0..1000 {
            let p = pack_trits(&trits);
            black_box(unpack_trits(p));
        }
    });

    println!();

    println!("  3.2 — Representation conversion");
    println!("  ────────────────────────────────────────────────────────────────");

    bench_n("A→B conversion × 10k", 10_000, || {
        for i in 0..10_000u32 {
            let v = (i % 3) as i8 - 1;
            black_box(convert_representation(v, Representation::A, Representation::B));
        }
    });

    bench_n("A→C conversion × 10k", 10_000, || {
        for i in 0..10_000u32 {
            let v = (i % 3) as i8 - 1;
            black_box(convert_representation(v, Representation::A, Representation::C));
        }
    });

    bench_n("B→A conversion × 10k", 10_000, || {
        for i in 0..10_000u32 {
            let v = (i % 3) as i8;
            black_box(convert_representation(v, Representation::B, Representation::A));
        }
    });

    println!();

    println!("  3.3 — Polynomial storage density");
    println!("  ────────────────────────────────────────────────────────────────");

    bench("TernaryPolynomial create n=256", || {
        let p = TernaryPolynomial::from_coeffs_unchecked(coeffs_a.clone());
        black_box(p);
    });

    bench("Vec<u16> create n=256 (Z_q reference)", || {
        black_box(coeffs_u16_a.clone());
    });

    println!();

    // ═══════════════════════════════════════════════════════════════════
    // DENSITY REPORT (computed, not benchmarked)
    // ═══════════════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("INFORMATION DENSITY REPORT");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    let log2_3: f64 = 3.0f64.log2();
    let gf3_bytes = poly_n;
    let zq_bytes = poly_n * 2;
    let gf3_info_bits = (poly_n as f64) * log2_3;
    let zq_info_bits = (poly_n as f64) * (3329.0f64).log2();

    println!("  GF(3) polynomial n=256:");
    println!("    Storage:      {} bytes ({} trits × 1 byte)", gf3_bytes, poly_n);
    println!("    Information:  {:.1} bits ({:.6} bits/trit)", gf3_info_bits, log2_3);
    println!("    Density:      {:.1}% of byte capacity", gf3_info_bits / (gf3_bytes as f64 * 8.0) * 100.0);
    println!();

    println!("  Z_q polynomial n=256, q=3329:");
    println!("    Storage:      {} bytes ({} coeffs × 2 bytes)", zq_bytes, poly_n);
    println!("    Information:  {:.1} bits ({:.3} bits/coeff)", zq_info_bits, 3329.0f64.log2());
    println!("    Density:      {:.1}% of u16 capacity", zq_info_bits / (zq_bytes as f64 * 8.0) * 100.0);
    println!();

    println!("  Trit packing (27 trits in i64):");
    println!("    Packed bits:  64");
    println!("    Information:  {:.1} bits (27 × {:.6})", 27.0 * log2_3, log2_3);
    println!("    Density:      {:.1}% of i64 capacity", 27.0 * log2_3 / 64.0 * 100.0);
    println!();

    println!("  Trit-per-bit ratio:   {:.6} (log₂3 ≈ 1.58496)", log2_3);
    println!("    → 1 trit stores {:.2}% more information than 1 bit", (log2_3 - 1.0) * 100.0);
    println!();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("KEY / CIPHERTEXT SIZE COMPARISON");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    let kem_sizes = [
        ("TL-KEM-512", 128, tl_kem::public_key_size(TlKemVariant::TlKem512),
         tl_kem::ciphertext_size(TlKemVariant::TlKem512),
         tl_kem::shared_secret_size(TlKemVariant::TlKem512)),
        ("TL-KEM-768", 192, tl_kem::public_key_size(TlKemVariant::TlKem768),
         tl_kem::ciphertext_size(TlKemVariant::TlKem768),
         tl_kem::shared_secret_size(TlKemVariant::TlKem768)),
        ("TL-KEM-1024", 256, tl_kem::public_key_size(TlKemVariant::TlKem1024),
         tl_kem::ciphertext_size(TlKemVariant::TlKem1024),
         tl_kem::shared_secret_size(TlKemVariant::TlKem1024)),
    ];

    let ml_kem_pk = [800usize, 1184, 1568];
    let ml_kem_ct = [768usize, 1088, 1568];
    let ml_kem_ss = [32usize, 32, 32];

    println!("  {:<16} {:>6} {:>6} {:>6}  │  {:<16} {:>6} {:>6} {:>6}",
        "TL-KEM", "pk", "ct", "ss", "ML-KEM (ref)", "pk", "ct", "ss");
    println!("  {}", "─".repeat(78));
    for (i, (name, sec, pk, ct, ss)) in kem_sizes.iter().enumerate() {
        println!("  {:<16} {:>5}t {:>5}t {:>5}t  │  ML-KEM-{:<8} {:>5}B {:>5}B {:>5}B",
            name, pk, ct, ss, sec, ml_kem_pk[i], ml_kem_ct[i], ml_kem_ss[i]);
    }
    println!();

    let dsa_sizes = [
        ("TL-DSA-44", 128, tl_dsa::public_key_size(TlDsaVariant::TlDsa44),
         tl_dsa::signature_size(TlDsaVariant::TlDsa44)),
        ("TL-DSA-65", 192, tl_dsa::public_key_size(TlDsaVariant::TlDsa65),
         tl_dsa::signature_size(TlDsaVariant::TlDsa65)),
        ("TL-DSA-87", 256, tl_dsa::public_key_size(TlDsaVariant::TlDsa87),
         tl_dsa::signature_size(TlDsaVariant::TlDsa87)),
    ];

    let ml_dsa_pk = [1312usize, 1952, 2592];
    let ml_dsa_sig = [2420usize, 3309, 4627];

    println!("  {:<16} {:>6} {:>6}  │  {:<16} {:>6} {:>6}",
        "TL-DSA", "pk", "sig", "ML-DSA (ref)", "pk", "sig");
    println!("  {}", "─".repeat(64));
    for (i, (name, sec, pk, sig)) in dsa_sizes.iter().enumerate() {
        println!("  {:<16} {:>5}t {:>5}t  │  ML-DSA-{:<8} {:>5}B {:>5}B",
            name, pk, sig, sec, ml_dsa_pk[i], ml_dsa_sig[i]);
    }
    println!();

    println!("═══════════════════════════════════════════════════════════════════════════════════");
    println!("BENCHMARK COMPLETE");
    println!("═══════════════════════════════════════════════════════════════════════════════════");
}
