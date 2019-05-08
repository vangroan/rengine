#[macro_use]
extern crate criterion;
extern crate rengine;

use criterion::Criterion;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("voxel iteration", |b| b.iter(|| {}));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
