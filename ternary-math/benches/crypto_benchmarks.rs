// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Patent(s) Pending — All Rights Reserved
// Applied Physics Division
//
// PROPRIETARY AND CONFIDENTIAL
// This file is part of the Salvi Framework / PlenumNET platform.
// See LICENSE in the repository root for full terms.
//
// Crypto Benchmark Suite — Criterion-based statistical benchmarks
// Per TM-2026-020.1-PREREQ §7
//
// Covers: TIS-27, TLSponge-385, TL-DSA (44/65/87), TL-KEM (512/768/1024),
//         Phase Encryption v3 (all 4 modes × 1KB/64KB/1MB),
//         Raw sponge permutation (v1 vs v2, SIMD-dispatched)

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, BatchSize, Throughput};

use ternary_math::tlsponge385;
use ternary_math::tl_dsa::{self, TlDsaVariant};
use ternary_math::tl_kem::{self, TlKemVariant};
use ternary_math::phase_encryption::{self, EncryptionMode};

fn make_input(size: usize) -> Vec<u8> {
    (0..size).map(|i| ((i * 7 + 13) % 256) as u8).collect()
}

fn bench_tis27(c: &mut Criterion) {
    let mut group = c.benchmark_group("TIS-27");

    let input_48 = make_input(48);
    group.bench_function("hash_hex_tis/48B", |b| {
        b.iter(|| tlsponge385::hash_hex_tis(black_box(&input_48)))
    });

    for &(size, label) in &[(1024usize, "1KB"), (65536, "64KB"), (1048576, "1MB")] {
        let input = make_input(size);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::new("derive_key_tis", label),
            &input,
            |b, data| b.iter(|| tlsponge385::derive_key_tis(
                black_box(b"TIS-27-BENCH"), black_box(data), 32)),
        );
    }

    let domains: Vec<&[u8]> = (0..26).map(|_| b"TIS-BATCH" as &[u8]).collect();
    let materials: Vec<Vec<u8>> = (0..26).map(|i| (i as u32).to_le_bytes().to_vec()).collect();
    let refs: Vec<&[u8]> = materials.iter().map(|m| m.as_slice()).collect();
    group.bench_function("batch_x26", |b| {
        b.iter(|| tlsponge385::derive_key_batch_tis(
            black_box(&domains), black_box(&refs), 32))
    });

    group.finish();
}

fn bench_tlsponge385(c: &mut Criterion) {
    let mut group = c.benchmark_group("TLSponge-385");

    let input_48 = make_input(48);
    group.bench_function("hash/48B", |b| {
        b.iter(|| tlsponge385::hash(black_box(&input_48), 48))
    });

    group.bench_function("hash_hex/48B", |b| {
        b.iter(|| tlsponge385::hash_hex(black_box(&input_48)))
    });

    group.bench_function("derive_key/48B", |b| {
        b.iter(|| tlsponge385::derive_key(
            black_box(b"SPONGE-BENCH"), black_box(&input_48), 48))
    });

    for &size in &[1024usize, 65536, 1048576] {
        let input = make_input(size);
        let label = match size {
            1024 => "1KB",
            65536 => "64KB",
            _ => "1MB",
        };

        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::new("hash", label),
            &input,
            |b, data| b.iter(|| tlsponge385::hash(black_box(data), 48)),
        );

        group.bench_with_input(
            BenchmarkId::new("derive_key", label),
            &input,
            |b, data| b.iter(|| tlsponge385::derive_key(
                black_box(b"SPONGE-BENCH"), black_box(data), 48)),
        );

        group.bench_with_input(
            BenchmarkId::new("derive_key_bulk", label),
            &input,
            |b, data| b.iter(|| tlsponge385::derive_key_bulk(
                black_box(b"BULK-BENCH"), black_box(data), 48)),
        );
    }

    group.finish();
}

fn bench_sponge_full_vs_tis(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sponge-Full-vs-TIS");

    let input_48 = make_input(48);

    group.bench_function("hash_hex_full/48B", |b| {
        b.iter(|| tlsponge385::hash_hex(black_box(&input_48)))
    });

    group.bench_function("hash_hex_tis/48B", |b| {
        b.iter(|| tlsponge385::hash_hex_tis(black_box(&input_48)))
    });

    for &(size, label) in &[(1024usize, "1KB"), (65536, "64KB")] {
        let input = make_input(size);
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::new("derive_key_full", label),
            &input,
            |b, data| b.iter(|| tlsponge385::derive_key(
                black_box(b"FULL-BENCH"), black_box(data), 48)),
        );

        group.bench_with_input(
            BenchmarkId::new("derive_key_tis", label),
            &input,
            |b, data| b.iter(|| tlsponge385::derive_key_tis(
                black_box(b"TIS-BENCH"), black_box(data), 48)),
        );
    }

    group.finish();
}

fn bench_tl_dsa(c: &mut Criterion) {
    let mut group = c.benchmark_group("TL-DSA");
    let msg = make_input(59);

    for &(variant, name) in &[
        (TlDsaVariant::TlDsa44, "44"),
        (TlDsaVariant::TlDsa65, "65"),
        (TlDsaVariant::TlDsa87, "87"),
    ] {
        group.bench_function(BenchmarkId::new("keygen", name), |b| {
            b.iter(|| tl_dsa::keygen(black_box(variant), Some(b"bench-seed")))
        });

        let kp = tl_dsa::keygen(variant, Some(b"bench-seed"));

        group.bench_function(BenchmarkId::new("sign", name), |b| {
            b.iter(|| tl_dsa::sign(
                black_box(&kp.secret_key), black_box(&msg), black_box(variant)))
        });

        let sig = tl_dsa::sign(&kp.secret_key, &msg, variant);

        group.bench_function(BenchmarkId::new("verify", name), |b| {
            b.iter(|| tl_dsa::verify(
                black_box(&kp.public_key), black_box(&msg),
                black_box(&sig), black_box(variant)))
        });
    }

    group.finish();
}

fn bench_tl_kem(c: &mut Criterion) {
    let mut group = c.benchmark_group("TL-KEM");

    let seed: Vec<i8> = (0..32).map(|i| ((i * 7 + 3) % 3) as i8 - 1).collect();
    let randomness: Vec<i8> = (0..32).map(|i| ((i * 11 + 5) % 3) as i8 - 1).collect();

    for &(variant, name) in &[
        (TlKemVariant::TlKem512, "512"),
        (TlKemVariant::TlKem768, "768"),
        (TlKemVariant::TlKem1024, "1024"),
    ] {
        group.bench_function(BenchmarkId::new("keygen", name), |b| {
            b.iter(|| tl_kem::keygen_with_seed(black_box(variant), black_box(&seed)))
        });

        let (pk, sk) = tl_kem::keygen_with_seed(variant, &seed).expect("KEM keygen");

        group.bench_function(BenchmarkId::new("encapsulate", name), |b| {
            b.iter(|| tl_kem::encapsulate_with_randomness(black_box(&pk), black_box(&randomness)))
        });

        let (ct, _ss) = tl_kem::encapsulate_with_randomness(&pk, &randomness).expect("KEM encaps");

        group.bench_function(BenchmarkId::new("decapsulate", name), |b| {
            b.iter(|| tl_kem::decapsulate(black_box(&ct), black_box(&sk)))
        });
    }

    group.finish();
}

fn bench_phase_encryption(c: &mut Criterion) {
    let mut group = c.benchmark_group("PhaseEncryption-v3");
    group.sample_size(10);

    let secret = make_input(32);
    let key = phase_encryption::derive_key_from_secret(&secret);
    let nonce: [u8; 32] = {
        let v = make_input(32);
        let mut n = [0u8; 32];
        n.copy_from_slice(&v);
        n
    };

    let modes = [
        (EncryptionMode::HighSecurity, "HighSec"),
        (EncryptionMode::Balanced, "Balanced"),
        (EncryptionMode::Performance, "Perf"),
        (EncryptionMode::Adaptive, "Adaptive"),
    ];

    for &(mode, mode_label) in &modes {
        for &(size, size_label) in &[(1024usize, "1KB"), (65536, "64KB"), (1048576, "1MB")] {
            let plaintext = make_input(size);
            let bench_id = format!("{}/{}", mode_label, size_label);

            group.throughput(Throughput::Bytes(size as u64));

            group.bench_with_input(
                BenchmarkId::new("encrypt", &bench_id),
                &plaintext,
                |b, data| b.iter(|| phase_encryption::encrypt_with_nonce(
                    black_box(data), black_box(&key),
                    black_box(mode), black_box(&nonce))
                ),
            );

            let ct = phase_encryption::encrypt_with_nonce(
                &plaintext, &key, mode, &nonce)
                .expect("phase encrypt");

            group.bench_with_input(
                BenchmarkId::new("decrypt", &bench_id),
                &ct,
                |b, ciphertext| b.iter(|| phase_encryption::decrypt(
                    black_box(ciphertext), black_box(&key), black_box(mode))
                ),
            );
        }
    }

    group.finish();
}

fn bench_sponge_permutation(c: &mut Criterion) {
    let mut group = c.benchmark_group("SpongePermutation");

    let init_state = {
        let mut s = [0i8; 729];
        for i in 0..729 { s[i] = ((i * 7 + 3) % 3) as i8 - 1; }
        s
    };

    group.bench_function("v2_chi_9rounds", |b| {
        b.iter_batched(
            || init_state,
            |mut state| { tlsponge385::sponge_permutation(black_box(&mut state)); state },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("v1_no_chi_9rounds", |b| {
        b.iter_batched(
            || init_state,
            |mut state| { tlsponge385::sponge_permutation_v1(black_box(&mut state)); state },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_tis27,
    bench_tlsponge385,
    bench_sponge_full_vs_tis,
    bench_tl_dsa,
    bench_tl_kem,
    bench_phase_encryption,
    bench_sponge_permutation,
);
criterion_main!(benches);
