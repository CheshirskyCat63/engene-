use engene::testsupport::perf_harness::{
    run_boundary_cost, run_kernel_throughput, run_tick_pressure, BoundaryConfig,
    KernelThroughputConfig, TickPressureConfig,
};

#[test]
fn print_foundation_perf_snapshot() {
    let kernel = run_kernel_throughput(&KernelThroughputConfig {
        events_per_thread: 5_000,
        thread_levels: vec![1, 2, 4, 8, 16],
        channel_capacity: 200_000,
    });

    println!("KERNEL_POINTS_START");
    for p in &kernel.points {
        println!(
            "threads={} total_ops={} ops_per_sec={:.2} ns_per_op={:.2} p50={} p95={} p99={} dropped={} backlog_peak={} speedup={:.3}",
            p.threads,
            p.metrics.total_ops,
            p.metrics.ops_per_sec,
            p.metrics.ns_per_op,
            p.metrics.latency.p50_ns,
            p.metrics.latency.p95_ns,
            p.metrics.latency.p99_ns,
            p.metrics.dropped_events,
            p.metrics.backlog_peak,
            p.speedup_vs_single_thread,
        );
    }
    println!("KERNEL_POINTS_END");

    let boundary = run_boundary_cost(&BoundaryConfig {
        events: 200_000,
        batch_size: 512,
    });
    println!(
        "BOUNDARY baseline_ns_per_event={:.4} k2r_ns_per_event={:.4} routing_ns_per_event={:.4} r2o_ns_per_event={:.4} total_ns_per_event={:.4} slowdown={:.4}",
        boundary.baseline_ns_per_event,
        boundary.kernel_to_runtime_ns_per_event,
        boundary.runtime_routing_ns_per_event,
        boundary.runtime_to_observer_ns_per_event,
        boundary.boundary_total_ns_per_event,
        boundary.relative_slowdown_vs_baseline,
    );

    let tick = run_tick_pressure(&TickPressureConfig {
        ticks: 24,
        producer_threads: 4,
        events_per_thread_per_tick: 3_000,
        tick_budget_ns: 1_000_000,
    });
    println!(
        "TICK mean_tick_ns={:.2} max_tick_ns={} deadline_miss_ticks={} max_backlog={} drain_efficiency={:.4} max_sustainable_ops_per_tick={}",
        tick.mean_tick_ns,
        tick.max_tick_ns,
        tick.deadline_miss_ticks,
        tick.max_backlog,
        tick.drain_efficiency,
        tick.max_sustainable_ops_per_tick,
    );
}
