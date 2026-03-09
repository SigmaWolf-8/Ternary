use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use plenumnet_kernel::crypto::tl_dsa::{self, TlDsaVariant};

fn bench_tl_dsa(c: &mut Criterion) {
    let mut group = c.benchmark_group("tl_dsa");
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
                    criterion::black_box(tl_dsa::keygen(variant, &seed).unwrap())
                });
            },
        );

        let (pk, sk) = tl_dsa::keygen(variant, &seed).unwrap();

        group.bench_function(
            BenchmarkId::new("sign", label),
            |b| {
                b.iter(|| {
                    criterion::black_box(tl_dsa::sign(&sk, &msg).unwrap())
                });
            },
        );

        let sig = tl_dsa::sign(&sk, &msg).unwrap();

        group.bench_function(
            BenchmarkId::new("verify", label),
            |b| {
                b.iter(|| {
                    criterion::black_box(tl_dsa::verify(&pk, &msg, &sig).unwrap())
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
                    criterion::black_box(valid)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_tl_dsa);
criterion_main!(benches);
