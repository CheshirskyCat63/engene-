use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use engene::testsupport::perf_harness::{run_tick_pressure, TickPressureConfig};

fn bench_tick_pressure(c: &mut Criterion) {
    let mut group = c.benchmark_group("foundation_tick_pressure");

    for budget_ns in [500_000u64, 1_000_000u64, 2_000_000u64] {
        group.bench_with_input(
            BenchmarkId::new("budget_ns", budget_ns),
            &budget_ns,
            |b, &budget| {
                b.iter_batched(
                    || TickPressureConfig {
                        ticks: 16,
                        producer_threads: 4,
                        events_per_thread_per_tick: 2_000,
                        tick_budget_ns: budget,
                    },
                    |cfg| run_tick_pressure(&cfg),
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_tick_pressure);
criterion_main!(benches);
