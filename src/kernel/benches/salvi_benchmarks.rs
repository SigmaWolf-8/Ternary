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
//! Criterion-based benchmarks measuring three quantitative claims:
//!
//! 1. **TVM Throughput** — ops/sec by security tier (Fortified/Verified/Basic)
//! 2. **Ternary vs Binary Efficiency** — GF(3) vs GF(2) operation cost ratio
//! 3. **Information Density** — trit-per-bit storage density
//!
//! # Running
//!
//! ```sh
//! cd src/kernel
//! cargo bench --bench salvi_benchmarks
//! ```
//!
//! Results are written to `target/criterion/` with HTML reports.
//!
//! # Methodology
//!
//! All benchmarks use criterion's statistical framework (100+ samples,
//! warm-up, outlier detection). Wall-clock measurements on x86_64 host.
//! FPGA/ASIC cycle-accurate measurements require target hardware with
//! rdtsc or equivalent counters.

use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime,
    BenchmarkGroup, BenchmarkId, Criterion, Throughput,
};

use plenumnet_kernel::ternary::{
    Trit, Representation, convert_representation, pack_trits, unpack_trits,
};
use plenumnet_kernel::crypto::sponge::TernarySponge;
use plenumnet_kernel::crypto::tl_kem::{self, TlKemVariant};
use plenumnet_kernel::crypto::tl_dsa::{self, TlDsaVariant};
use plenumnet_kernel::crypto::ternary_lattice::TernaryPolynomial;
use plenumnet_kernel::vm::engine::TernaryVm;
use plenumnet_kernel::vm::instruction::{Instruction, Opcode};

// ===================================================================
// CONSTANTS
// ===================================================================

const POLY_DEGREE: usize = 256;
const SPONGE_RATE: usize = 243;
const SPONGE_OUTPUT: usize = 243;
const PACKED_TRITS: usize = 27;

// ===================================================================
// GROUP 1: TVM THROUGHPUT BY SECURITY TIER
// ===================================================================

fn make_vm_with_program(program: Vec<Instruction>) -> TernaryVm {
    let mut vm = TernaryVm::new(65536, 4096);
    vm.load_program(program);
    vm
}

fn tvm_ternary_alu_program(count: usize) -> Vec<Instruction> {
    let mut program = Vec::with_capacity(count + 1);
    for i in 0..count {
        let opcode = match i % 5 {
            0 => Opcode::TAdd,
            1 => Opcode::TMul,
            2 => Opcode::TNeg,
            3 => Opcode::TRot,
            _ => Opcode::TXor,
        };
        program.push(Instruction {
            opcode,
            dst: 0,
            src1: 1,
            src2: 2,
            immediate: 0,
            flags: 0,
        });
    }
    program.push(Instruction {
        opcode: Opcode::Halt,
        dst: 0,
        src1: 0,
        src2: 0,
        immediate: 0,
        flags: 0,
    });
    program
}

fn tvm_binary_alu_program(count: usize) -> Vec<Instruction> {
    let mut program = Vec::with_capacity(count + 1);
    for i in 0..count {
        let opcode = match i % 4 {
            0 => Opcode::Add,
            1 => Opcode::Sub,
            2 => Opcode::Mul,
            _ => Opcode::Neg,
        };
        program.push(Instruction {
            opcode,
            dst: 0,
            src1: 1,
            src2: 2,
            immediate: 0,
            flags: 0,
        });
    }
    program.push(Instruction {
        opcode: Opcode::Halt,
        dst: 0,
        src1: 0,
        src2: 0,
        immediate: 0,
        flags: 0,
    });
    program
}

fn bench_tvm_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("tvm_throughput");

    for &batch_size in &[100u64, 1000, 10_000] {
        let n = batch_size as usize;

        group.throughput(Throughput::Elements(batch_size));

        group.bench_with_input(
            BenchmarkId::new("ternary_alu", batch_size),
            &n,
            |b, &n| {
                b.iter_with_setup(
                    || make_vm_with_program(tvm_ternary_alu_program(n)),
                    |mut vm| {
                        let _ = black_box(vm.run());
                    },
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("binary_alu", batch_size),
            &n,
            |b, &n| {
                b.iter_with_setup(
                    || make_vm_with_program(tvm_binary_alu_program(n)),
                    |mut vm| {
                        let _ = black_box(vm.run());
                    },
                );
            },
        );
    }

    group.finish();
}

fn bench_tvm_tier_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("tvm_tier_throughput");

    let ops_per_iter = 1000u64;
    group.throughput(Throughput::Elements(ops_per_iter));

    let alu_program = tvm_ternary_alu_program(ops_per_iter as usize);

    group.bench_function("basic_tier", |b| {
        b.iter_with_setup(
            || make_vm_with_program(alu_program.clone()),
            |mut vm| {
                let _ = black_box(vm.run());
            },
        );
    });

    let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
    group.bench_function("verified_tier", |b| {
        b.iter(|| {
            let mut vm = make_vm_with_program(alu_program.clone());
            let _ = vm.run();
            let mut sponge = TernarySponge::new();
            sponge.absorb(&seed);
            black_box(sponge.squeeze(SPONGE_OUTPUT));
        });
    });

    group.bench_function("fortified_tier", |b| {
        b.iter(|| {
            let mut vm = make_vm_with_program(alu_program.clone());
            let _ = vm.run();
            let (pk, sk) = tl_dsa::keygen(TlDsaVariant::TlDsa44, &seed).unwrap();
            let msg = vec![1i8, 0, -1];
            let sig = tl_dsa::sign(&sk, &msg).unwrap();
            black_box(tl_dsa::verify(&pk, &msg, &sig).unwrap());
        });
    });

    group.finish();
}

// ===================================================================
// GROUP 2: TERNARY VS BINARY EFFICIENCY RATIO
// ===================================================================

fn bench_gf3_vs_gf2_scalar(c: &mut Criterion) {
    let mut group = c.benchmark_group("gf3_vs_gf2_scalar");
    let ops = 10_000u64;
    group.throughput(Throughput::Elements(ops));

    let ta = Trit::from_a(1).unwrap();
    let tb = Trit::from_a(-1).unwrap();
    let ba: u8 = 0xAB;
    let bb: u8 = 0xCD;

    group.bench_function("gf3_add", |b| {
        b.iter(|| {
            let mut acc = ta;
            for _ in 0..ops {
                acc = acc.add(&tb);
            }
            black_box(acc)
        });
    });

    group.bench_function("gf2_add", |b| {
        b.iter(|| {
            let mut acc = ba;
            for _ in 0..ops {
                acc = acc.wrapping_add(bb);
            }
            black_box(acc)
        });
    });

    group.bench_function("gf3_mul", |b| {
        b.iter(|| {
            let mut acc = ta;
            for _ in 0..ops {
                acc = acc.multiply(&tb);
            }
            black_box(acc)
        });
    });

    group.bench_function("gf2_mul", |b| {
        b.iter(|| {
            let mut acc = ba;
            for _ in 0..ops {
                acc = acc.wrapping_mul(bb);
            }
            black_box(acc)
        });
    });

    group.bench_function("gf3_rotate", |b| {
        b.iter(|| {
            let mut acc = ta;
            for _ in 0..ops {
                acc = acc.rotate();
            }
            black_box(acc)
        });
    });

    group.bench_function("gf2_shift", |b| {
        b.iter(|| {
            let mut acc = ba;
            for _ in 0..ops {
                acc = acc.rotate_left(1);
            }
            black_box(acc)
        });
    });

    group.bench_function("gf3_not", |b| {
        b.iter(|| {
            let mut acc = ta;
            for _ in 0..ops {
                acc = acc.not();
            }
            black_box(acc)
        });
    });

    group.bench_function("gf2_not", |b| {
        b.iter(|| {
            let mut acc = ba;
            for _ in 0..ops {
                acc = !acc;
            }
            black_box(acc)
        });
    });

    group.finish();
}

fn bench_ring_mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_mul");
    group.sample_size(20);

    let coeffs_a: Vec<i8> = (0..POLY_DEGREE).map(|i| ((i % 3) as i8 - 1)).collect();
    let coeffs_b: Vec<i8> = (0..POLY_DEGREE).map(|i| (((i + 1) % 3) as i8 - 1)).collect();
    let poly_a = TernaryPolynomial::new(coeffs_a.clone(), POLY_DEGREE).unwrap();
    let poly_b = TernaryPolynomial::new(coeffs_b.clone(), POLY_DEGREE).unwrap();

    group.bench_function("R3_schoolbook_n256", |b| {
        b.iter(|| {
            black_box(poly_a.ring_mul(&poly_b).unwrap())
        });
    });

    let coeffs_u16_a: Vec<u16> = coeffs_a.iter().map(|&x| (x as i16 + 3329) as u16 % 3329).collect();
    let coeffs_u16_b: Vec<u16> = coeffs_b.iter().map(|&x| (x as i16 + 3329) as u16 % 3329).collect();

    group.bench_function("Zq_schoolbook_n256_q3329", |b| {
        b.iter(|| {
            let n = POLY_DEGREE;
            let q: u32 = 3329;
            let mut result = vec![0u32; n];
            for i in 0..n {
                if coeffs_u16_a[i] == 0 { continue; }
                for j in 0..n {
                    if coeffs_u16_b[j] == 0 { continue; }
                    let product = coeffs_u16_a[i] as u32 * coeffs_u16_b[j] as u32;
                    let pos = i + j;
                    if pos < n {
                        result[pos] = (result[pos] + product) % q;
                    } else {
                        result[pos - n] = (result[pos - n] + q - (product % q)) % q;
                    }
                }
            }
            black_box(result)
        });
    });

    group.finish();
}

fn bench_sponge_vs_sha256(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_comparison");

    for &input_len in &[16usize, 64, 243, 729] {
        let trit_input: Vec<i8> = (0..input_len).map(|i| ((i % 3) as i8 - 1)).collect();
        let byte_input: Vec<u8> = (0..input_len).map(|i| (i % 256) as u8).collect();

        group.bench_with_input(
            BenchmarkId::new("ternary_sponge", input_len),
            &input_len,
            |b, _| {
                b.iter(|| {
                    let mut sponge = TernarySponge::new();
                    sponge.absorb(&trit_input);
                    black_box(sponge.squeeze(SPONGE_OUTPUT))
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("sha256_reference", input_len),
            &input_len,
            |b, _| {
                b.iter(|| {
                    let hash = plenumnet_kernel::crypto::sha2::sha256(&byte_input);
                    black_box(hash)
                });
            },
        );
    }

    group.finish();
}

fn bench_tl_kem_vs_ml_kem(c: &mut Criterion) {
    let mut group = c.benchmark_group("kem_operations");
    group.sample_size(10);

    let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1];
    let rand = vec![1i8, 0, -1, 1, 0, -1, 1, 0];

    for &(variant, label) in &[
        (TlKemVariant::TlKem512, "TL-KEM-512"),
        (TlKemVariant::TlKem768, "TL-KEM-768"),
        (TlKemVariant::TlKem1024, "TL-KEM-1024"),
    ] {
        group.bench_function(
            BenchmarkId::new("keygen", label),
            |b| {
                b.iter(|| {
                    black_box(tl_kem::keygen(variant, &seed).unwrap())
                });
            },
        );

        let (pk, sk) = tl_kem::keygen(variant, &seed).unwrap();

        group.bench_function(
            BenchmarkId::new("encaps", label),
            |b| {
                b.iter(|| {
                    black_box(tl_kem::encapsulate(&pk, &rand).unwrap())
                });
            },
        );

        let (ct, _ss) = tl_kem::encapsulate(&pk, &rand).unwrap();

        group.bench_function(
            BenchmarkId::new("decaps", label),
            |b| {
                b.iter(|| {
                    black_box(tl_kem::decapsulate(&sk, &ct).unwrap())
                });
            },
        );

        group.bench_function(
            BenchmarkId::new("full_roundtrip", label),
            |b| {
                b.iter(|| {
                    let (pk, sk) = tl_kem::keygen(variant, &seed).unwrap();
                    let (ct, ss1) = tl_kem::encapsulate(&pk, &rand).unwrap();
                    let ss2 = tl_kem::decapsulate(&sk, &ct).unwrap();
                    black_box((ss1, ss2))
                });
            },
        );
    }

    group.finish();
}

fn bench_tl_dsa_vs_ml_dsa(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsa_operations");
    group.sample_size(10);

    let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
    let msg = vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1];

    for &(variant, label) in &[
        (TlDsaVariant::TlDsa44, "TL-DSA-44"),
        (TlDsaVariant::TlDsa65, "TL-DSA-65"),
        (TlDsaVariant::TlDsa87, "TL-DSA-87"),
    ] {
        group.bench_function(
            BenchmarkId::new("keygen", label),
            |b| {
                b.iter(|| {
                    black_box(tl_dsa::keygen(variant, &seed).unwrap())
                });
            },
        );

        let (pk, sk) = tl_dsa::keygen(variant, &seed).unwrap();

        group.bench_function(
            BenchmarkId::new("sign", label),
            |b| {
                b.iter(|| {
                    black_box(tl_dsa::sign(&sk, &msg).unwrap())
                });
            },
        );

        let sig = tl_dsa::sign(&sk, &msg).unwrap();

        group.bench_function(
            BenchmarkId::new("verify", label),
            |b| {
                b.iter(|| {
                    black_box(tl_dsa::verify(&pk, &msg, &sig).unwrap())
                });
            },
        );

        group.bench_function(
            BenchmarkId::new("full_roundtrip", label),
            |b| {
                b.iter(|| {
                    let (pk, sk) = tl_dsa::keygen(variant, &seed).unwrap();
                    let sig = tl_dsa::sign(&sk, &msg).unwrap();
                    let valid = tl_dsa::verify(&pk, &msg, &sig).unwrap();
                    black_box(valid)
                });
            },
        );
    }

    group.finish();
}

// ===================================================================
// GROUP 3: INFORMATION DENSITY
// ===================================================================

fn bench_information_density(c: &mut Criterion) {
    let mut group = c.benchmark_group("information_density");
    let n = 1000u64;
    group.throughput(Throughput::Elements(n));

    let trits: Vec<Trit> = (0..PACKED_TRITS)
        .map(|i| Trit::from_a((i % 3) as i8 - 1).unwrap())
        .collect();

    group.bench_function("pack_27_trits", |b| {
        b.iter(|| {
            for _ in 0..n {
                black_box(pack_trits(&trits));
            }
        });
    });

    let packed = pack_trits(&trits);
    group.bench_function("unpack_27_trits", |b| {
        b.iter(|| {
            for _ in 0..n {
                black_box(unpack_trits(packed, PACKED_TRITS));
            }
        });
    });

    group.bench_function("pack_unpack_roundtrip", |b| {
        b.iter(|| {
            for _ in 0..n {
                let p = pack_trits(&trits);
                let u = unpack_trits(p, PACKED_TRITS);
                black_box(u);
            }
        });
    });

    let rep_a: Vec<i8> = (0..PACKED_TRITS).map(|i| (i % 3) as i8 - 1).collect();
    group.bench_function("repr_a_to_b", |b| {
        b.iter(|| {
            for _ in 0..n {
                let result = convert_representation(&rep_a, Representation::A, Representation::B);
                black_box(result);
            }
        });
    });

    group.bench_function("repr_a_to_c", |b| {
        b.iter(|| {
            for _ in 0..n {
                let result = convert_representation(&rep_a, Representation::A, Representation::C);
                black_box(result);
            }
        });
    });

    let rep_b = convert_representation(&rep_a, Representation::A, Representation::B);
    group.bench_function("repr_b_to_a", |b| {
        b.iter(|| {
            for _ in 0..n {
                let result = convert_representation(&rep_b, Representation::B, Representation::A);
                black_box(result);
            }
        });
    });

    group.finish();
}

fn bench_polynomial_storage(c: &mut Criterion) {
    let mut group = c.benchmark_group("polynomial_storage");

    let coeffs_gf3: Vec<i8> = (0..POLY_DEGREE).map(|i| ((i % 3) as i8 - 1)).collect();
    let coeffs_zq: Vec<u16> = (0..POLY_DEGREE).map(|i| (i % 3329) as u16).collect();

    group.bench_function("gf3_poly_create_n256", |b| {
        b.iter(|| {
            black_box(TernaryPolynomial::new(coeffs_gf3.clone(), POLY_DEGREE).unwrap())
        });
    });

    group.bench_function("zq_vec_create_n256", |b| {
        b.iter(|| {
            black_box(coeffs_zq.clone())
        });
    });

    let gf3_bytes = POLY_DEGREE;
    let zq_bytes = POLY_DEGREE * 2;
    let gf3_info_bits = (POLY_DEGREE as f64) * (3.0f64).log2();
    let zq_info_bits = (POLY_DEGREE as f64) * (3329.0f64).log2();

    println!("\n=== Information Density Report ===");
    println!("GF(3) polynomial n=256:");
    println!("  Storage:        {} bytes ({} trits × 1 byte)", gf3_bytes, POLY_DEGREE);
    println!("  Information:    {:.1} bits ({:.3} bits/trit)", gf3_info_bits, (3.0f64).log2());
    println!("  Density:        {:.1}% of byte capacity", gf3_info_bits / (gf3_bytes as f64 * 8.0) * 100.0);
    println!();
    println!("Z_q polynomial n=256, q=3329:");
    println!("  Storage:        {} bytes ({} coeffs × 2 bytes)", zq_bytes, POLY_DEGREE);
    println!("  Information:    {:.1} bits ({:.3} bits/coeff)", zq_info_bits, (3329.0f64).log2());
    println!("  Density:        {:.1}% of u16 capacity", zq_info_bits / (zq_bytes as f64 * 8.0) * 100.0);
    println!();
    println!("Trit packing (27 trits in i64):");
    println!("  Packed bits:    64");
    println!("  Information:    {:.1} bits (27 × {:.6})", 27.0 * (3.0f64).log2(), (3.0f64).log2());
    println!("  Density:        {:.1}% of i64 capacity", 27.0 * (3.0f64).log2() / 64.0 * 100.0);
    println!();
    println!("Trit-per-bit ratio:   {:.6} (log₂3 ≈ 1.58496)", (3.0f64).log2());
    println!("  → 1 trit stores {:.2}% more information than 1 bit", ((3.0f64).log2() - 1.0) * 100.0);
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

    println!("=== Key/Ciphertext Size Comparison (trits vs bytes) ===");
    println!("{:<16} {:>6} {:>6} {:>6}  |  {:<16} {:>6} {:>6} {:>6}",
        "TL-KEM", "pk", "ct", "ss", "ML-KEM (ref)", "pk", "ct", "ss");
    println!("{}", "-".repeat(80));
    for (i, (name, sec, pk, ct, ss)) in kem_sizes.iter().enumerate() {
        println!("{:<16} {:>5}t {:>5}t {:>5}t  |  ML-KEM-{:<8} {:>5}B {:>5}B {:>5}B",
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

    println!("{:<16} {:>6} {:>6}  |  {:<16} {:>6} {:>6}",
        "TL-DSA", "pk", "sig", "ML-DSA (ref)", "pk", "sig");
    println!("{}", "-".repeat(64));
    for (i, (name, sec, pk, sig)) in dsa_sizes.iter().enumerate() {
        println!("{:<16} {:>5}t {:>5}t  |  ML-DSA-{:<8} {:>5}B {:>5}B",
            name, pk, sig, sec, ml_dsa_pk[i], ml_dsa_sig[i]);
    }
    println!("=== End Density Report ===\n");

    group.finish();
}

fn bench_crypto_by_tier(c: &mut Criterion) {
    let mut group = c.benchmark_group("crypto_by_tier");
    group.sample_size(10);

    let seed = vec![0i8, 1, -1, 0, 1, -1, 0, 1, -1, 0, 1, -1];
    let msg = vec![1i8, 0, -1, 1, 0, -1, 1, 0, -1];
    let rand = vec![1i8, 0, -1, 1, 0, -1, 1, 0];

    group.bench_function("basic_sponge_only", |b| {
        b.iter(|| {
            let mut sponge = TernarySponge::new();
            sponge.absorb(&msg);
            black_box(sponge.squeeze(SPONGE_OUTPUT))
        });
    });

    group.bench_function("verified_kem_512", |b| {
        b.iter(|| {
            let (pk, sk) = tl_kem::keygen(TlKemVariant::TlKem512, &seed).unwrap();
            let (ct, ss1) = tl_kem::encapsulate(&pk, &rand).unwrap();
            let ss2 = tl_kem::decapsulate(&sk, &ct).unwrap();
            black_box((ss1, ss2))
        });
    });

    group.bench_function("fortified_dsa_44_sign_verify", |b| {
        b.iter(|| {
            let (pk, sk) = tl_dsa::keygen(TlDsaVariant::TlDsa44, &seed).unwrap();
            let sig = tl_dsa::sign(&sk, &msg).unwrap();
            let valid = tl_dsa::verify(&pk, &msg, &sig).unwrap();
            black_box(valid)
        });
    });

    group.bench_function("fortified_kem_1024_plus_dsa_87", |b| {
        b.iter(|| {
            let (kem_pk, kem_sk) = tl_kem::keygen(TlKemVariant::TlKem1024, &seed).unwrap();
            let (ct, ss) = tl_kem::encapsulate(&kem_pk, &rand).unwrap();
            let _ss2 = tl_kem::decapsulate(&kem_sk, &ct).unwrap();

            let (dsa_pk, dsa_sk) = tl_dsa::keygen(TlDsaVariant::TlDsa87, &seed).unwrap();
            let sig = tl_dsa::sign(&dsa_sk, &msg).unwrap();
            let valid = tl_dsa::verify(&dsa_pk, &msg, &sig).unwrap();
            black_box((ss, valid))
        });
    });

    group.finish();
}

// ===================================================================
// CRITERION HARNESS
// ===================================================================

criterion_group!(
    tvm_benches,
    bench_tvm_throughput,
    bench_tvm_tier_throughput
);

criterion_group!(
    ternary_vs_binary_benches,
    bench_gf3_vs_gf2_scalar,
    bench_ring_mul,
    bench_sponge_vs_sha256,
    bench_tl_kem_vs_ml_kem,
    bench_tl_dsa_vs_ml_dsa
);

criterion_group!(
    density_benches,
    bench_information_density,
    bench_polynomial_storage,
    bench_crypto_by_tier
);

criterion_main!(tvm_benches, ternary_vs_binary_benches, density_benches);
