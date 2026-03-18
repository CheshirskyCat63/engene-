use std::time::Duration;

#[derive(Clone, Debug)]
pub struct LatencyStats {
    pub p50_ns: u64,
    pub p95_ns: u64,
    pub p99_ns: u64,
}

#[derive(Clone, Debug)]
pub struct ThroughputMetrics {
    pub total_ops: u64,
    pub elapsed: Duration,
    pub ops_per_sec: f64,
    pub ns_per_op: f64,
    pub dropped_events: u64,
    pub backlog_peak: usize,
    pub latency: LatencyStats,
}

#[derive(Clone, Debug)]
pub struct ScalingPoint {
    pub threads: usize,
    pub metrics: ThroughputMetrics,
    pub speedup_vs_single_thread: f64,
}

#[derive(Clone, Debug)]
pub struct BoundaryMetrics {
    pub events: usize,
    pub baseline_ns_per_event: f64,
    pub kernel_to_runtime_ns_per_event: f64,
    pub runtime_routing_ns_per_event: f64,
    pub runtime_to_observer_ns_per_event: f64,
    pub boundary_total_ns_per_event: f64,
    pub boundary_batch_total_ns: u128,
    pub relative_slowdown_vs_baseline: f64,
}

#[derive(Clone, Debug)]
pub struct TickMetrics {
    pub ticks: usize,
    pub produced_events: u64,
    pub drained_events: u64,
    pub max_backlog: usize,
    pub deadline_miss_ticks: usize,
    pub mean_tick_ns: f64,
    pub max_tick_ns: u128,
    pub max_sustainable_ops_per_tick: u64,
    pub drain_efficiency: f64,
}

pub fn percentile_ns(mut values: Vec<u64>, percentile: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    let len = values.len();
    let idx = ((len - 1) as f64 * percentile).round() as usize;
    values[idx.min(len - 1)]
}

pub fn to_latency_stats(values: &[u64]) -> LatencyStats {
    let data = values.to_vec();
    LatencyStats {
        p50_ns: percentile_ns(data.clone(), 0.50),
        p95_ns: percentile_ns(data.clone(), 0.95),
        p99_ns: percentile_ns(data, 0.99),
    }
}

pub fn throughput(total_ops: u64, elapsed: Duration) -> (f64, f64) {
    let secs = elapsed.as_secs_f64();
    let ops_per_sec = if secs > 0.0 {
        total_ops as f64 / secs
    } else {
        0.0
    };
    let ns_per_op = if total_ops > 0 {
        elapsed.as_nanos() as f64 / total_ops as f64
    } else {
        0.0
    };
    (ops_per_sec, ns_per_op)
}
