use engene::testsupport::perf_harness::{run_boundary_cost, BoundaryConfig};

const EVENTS: usize = 200_000;
const BATCH_SIZE: usize = 512;

// Binding: metric integrity (finite/non-zero) and monotonic boundary aggregation.
#[test]
fn boundary_overhead_is_measured_and_meaningful() {
    let metrics = run_boundary_cost(&BoundaryConfig {
        events: EVENTS,
        batch_size: BATCH_SIZE,
    });

    assert_eq!(metrics.events, EVENTS);
    assert!(metrics.baseline_ns_per_event.is_finite());
    assert!(metrics.kernel_to_runtime_ns_per_event.is_finite());
    assert!(metrics.runtime_routing_ns_per_event.is_finite());
    assert!(metrics.runtime_to_observer_ns_per_event.is_finite());
    assert!(metrics.boundary_total_ns_per_event.is_finite());
    assert!(metrics.relative_slowdown_vs_baseline.is_finite());

    assert!(metrics.boundary_total_ns_per_event > 0.0);
    assert!(metrics.boundary_batch_total_ns > 0);

    let split_sum = metrics.kernel_to_runtime_ns_per_event
        + metrics.runtime_routing_ns_per_event
        + metrics.runtime_to_observer_ns_per_event;
    assert!(metrics.boundary_total_ns_per_event <= split_sum + 1e-6);
    assert!(metrics.relative_slowdown_vs_baseline >= 1.0);
}
