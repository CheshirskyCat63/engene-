use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use engene::testsupport::perf_harness::{run_kernel_throughput, KernelThroughputConfig};

fn bench_kernel_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("foundation_kernel_throughput");
    group.sample_size(10);

    for threads in [1usize, 2, 4, 8, 16] {
        group.bench_with_input(BenchmarkId::new("threads", threads), &threads, |b, &t| {
            b.iter_batched(
                || KernelThroughputConfig {
                    events_per_thread: 2_000,
                    thread_levels: vec![t],
                    channel_capacity: 100_000,
                },
                |cfg| run_kernel_throughput(&cfg),
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_kernel_throughput);
criterion_main!(benches);
