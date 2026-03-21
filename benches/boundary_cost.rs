use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use engene::testsupport::perf_harness::{run_boundary_cost, BoundaryConfig};

fn bench_boundary_cost(c: &mut Criterion) {
    c.bench_function("foundation_boundary_cost", |b| {
        b.iter_batched(
            || BoundaryConfig {
                events: 100_000,
                batch_size: 512,
            },
            |cfg| run_boundary_cost(&cfg),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_boundary_cost);
criterion_main!(benches);
