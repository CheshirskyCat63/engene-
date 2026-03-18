use engene::testsupport::perf_harness::{run_tick_pressure, TickPressureConfig};

const TICKS: usize = 24;
const PRODUCER_THREADS: usize = 4;
const EVENTS_PER_THREAD_PER_TICK: usize = 3_000;
const TICK_BUDGET_NS: u64 = 1_000_000; // 1ms

// Temporary threshold: fixed-budget harness must remain draining and bounded.
const MIN_DRAIN_EFFICIENCY: f64 = 0.50;

#[test]
fn fixed_tick_pipeline_pressure_contract_holds() {
    let metrics = run_tick_pressure(&TickPressureConfig {
        ticks: TICKS,
        producer_threads: PRODUCER_THREADS,
        events_per_thread_per_tick: EVENTS_PER_THREAD_PER_TICK,
        tick_budget_ns: TICK_BUDGET_NS,
    });

    assert_eq!(metrics.ticks, TICKS);
    assert!(metrics.produced_events > 0);
    assert!(metrics.drained_events > 0);
    assert!(metrics.max_backlog > 0);

    assert!(metrics.mean_tick_ns.is_finite());
    assert!(metrics.max_tick_ns > 0);
    assert!(metrics.drain_efficiency.is_finite());

    assert!(metrics.drain_efficiency >= MIN_DRAIN_EFFICIENCY);
    assert!(metrics.drain_efficiency <= 1.0);
    assert!(metrics.max_sustainable_ops_per_tick > 0);
    assert!(metrics.deadline_miss_ticks <= metrics.ticks);
}
