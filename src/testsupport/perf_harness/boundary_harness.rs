use std::hint::black_box;
use std::time::Instant;

use super::metrics::BoundaryMetrics;

#[derive(Clone, Debug)]
pub struct BoundaryConfig {
    pub events: usize,
    pub batch_size: usize,
}

#[derive(Clone, Debug)]
struct RuntimeEnvelope {
    id: u64,
    routed: u64,
}

pub fn run_boundary_cost(config: &BoundaryConfig) -> BoundaryMetrics {
    let events: Vec<u64> = (0..config.events as u64).collect();

    let baseline_start = Instant::now();
    let mut baseline_acc = 0u64;
    for &e in &events {
        baseline_acc = baseline_acc.wrapping_add(black_box(e));
    }
    black_box(baseline_acc);
    let baseline_ns = baseline_start.elapsed().as_nanos();

    let k2r_start = Instant::now();
    let runtime_batch: Vec<RuntimeEnvelope> = events
        .chunks(config.batch_size.max(1))
        .flat_map(|chunk| {
            chunk
                .iter()
                .map(|id| RuntimeEnvelope { id: *id, routed: 0 })
        })
        .collect();
    let kernel_to_runtime_ns = k2r_start.elapsed().as_nanos();

    let routing_start = Instant::now();
    let routed_batch: Vec<RuntimeEnvelope> = runtime_batch
        .into_iter()
        .map(|event| RuntimeEnvelope {
            id: event.id,
            routed: event.id ^ 0x9E37_79B9,
        })
        .collect();
    let runtime_routing_ns = routing_start.elapsed().as_nanos();

    let r2o_start = Instant::now();
    let mut observer_acc = 0u64;
    for event in &routed_batch {
        observer_acc = observer_acc.wrapping_add(black_box(event.id ^ event.routed));
    }
    black_box(observer_acc);
    let runtime_to_observer_ns = r2o_start.elapsed().as_nanos();

    let events_f = config.events.max(1) as f64;
    let baseline_ns_per_event = baseline_ns as f64 / events_f;
    let kernel_to_runtime_ns_per_event = kernel_to_runtime_ns as f64 / events_f;
    let runtime_routing_ns_per_event = runtime_routing_ns as f64 / events_f;
    let runtime_to_observer_ns_per_event = runtime_to_observer_ns as f64 / events_f;
    let boundary_total_ns_per_event =
        (kernel_to_runtime_ns + runtime_routing_ns + runtime_to_observer_ns) as f64 / events_f;
    let relative_slowdown_vs_baseline = if baseline_ns_per_event > 0.0 {
        boundary_total_ns_per_event / baseline_ns_per_event
    } else {
        0.0
    };

    BoundaryMetrics {
        events: config.events,
        baseline_ns_per_event,
        kernel_to_runtime_ns_per_event,
        runtime_routing_ns_per_event,
        runtime_to_observer_ns_per_event,
        boundary_total_ns_per_event,
        boundary_batch_total_ns: kernel_to_runtime_ns + runtime_routing_ns + runtime_to_observer_ns,
        relative_slowdown_vs_baseline,
    }
}
