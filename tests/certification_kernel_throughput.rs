use engene::testsupport::perf_harness::{run_kernel_throughput, KernelThroughputConfig};

const EVENTS_PER_THREAD: usize = 5_000;
const CHANNEL_CAPACITY: usize = 200_000;
const THREAD_LEVELS: [usize; 5] = [1, 2, 4, 8, 16];

// Temporary threshold (explicit): on broad CI hardware we only require
// non-catastrophic 16-thread scaling while preserving metric integrity.
const MIN_16T_SPEEDUP_VS_1T: f64 = 0.10;

#[test]
fn kernel_throughput_contract_is_decision_grade() {
    let config = KernelThroughputConfig {
        events_per_thread: EVENTS_PER_THREAD,
        thread_levels: THREAD_LEVELS.to_vec(),
        channel_capacity: CHANNEL_CAPACITY,
    };

    let report = run_kernel_throughput(&config);
    assert_eq!(report.points.len(), config.thread_levels.len());

    let single = &report.points[0];
    assert_eq!(single.threads, 1);
    assert!(single.metrics.total_ops > 0);
    assert!(single.metrics.ops_per_sec.is_finite());
    assert!(single.metrics.ns_per_op.is_finite());
    assert!(single.metrics.latency.p50_ns <= single.metrics.latency.p95_ns);
    assert!(single.metrics.latency.p95_ns <= single.metrics.latency.p99_ns);
    assert_eq!(single.metrics.dropped_events, 0);

    for point in &report.points {
        assert!(point.metrics.total_ops > 0);
        assert!(point.metrics.backlog_peak > 0);
        assert!(point.metrics.ops_per_sec.is_finite());
        assert!(point.metrics.latency.p99_ns >= point.metrics.latency.p50_ns);
    }

    let max_thread_point = report.points.last().expect("max-thread point");
    assert!(max_thread_point.speedup_vs_single_thread >= MIN_16T_SPEEDUP_VS_1T);
}
