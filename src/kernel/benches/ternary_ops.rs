use criterion::{criterion_group, criterion_main, Criterion};

fn ternary_benchmarks(_c: &mut Criterion) {
}

criterion_group!(benches, ternary_benchmarks);
criterion_main!(benches);
