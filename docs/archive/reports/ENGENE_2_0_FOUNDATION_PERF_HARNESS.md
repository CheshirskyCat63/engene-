# ENGENE 2.0 Foundation Performance Harness (Canonical)

Date: 2026-03-18

## Purpose
This harness provides decision-grade measurements for foundation architecture costs:
- pure kernel throughput (producer -> core event bus -> consumer)
- boundary crossing overhead (kernel -> runtime routing -> app/tools observer)
- fixed-tick pressure behavior under bounded budgets

It intentionally excludes renderer/gameplay noise from the kernel baseline.

## Canonical scenarios
1. **Pure kernel throughput**
   - Entry: `src/testsupport/perf_harness/event_bus_harness.rs`
   - Measures: total ops, ops/sec, ns/op, p50/p95/p99 latency, dropped events, backlog peak, scaling 1/2/4/8/16.
2. **Boundary cost**
   - Entry: `src/testsupport/perf_harness/boundary_harness.rs`
   - Measures: ns/event per hop (`kernel->runtime envelope`, `runtime routing`, `runtime->observer`), batch transfer cost, relative slowdown vs baseline.
3. **Fixed tick pressure**
   - Entry: `src/testsupport/perf_harness/tick_harness.rs`
   - Measures: tick time, deadline misses, max sustainable ops/tick, backlog growth, drain efficiency.

## Certification vs exploration split
### Certification tests (binding pass/fail)
- `tests/certification_kernel_throughput.rs`
- `tests/certification_boundary_overhead.rs`
- `tests/certification_tick_budget.rs`

Rules:
- **Binding thresholds:** metric integrity (finite/non-NaN/non-zero where required), monotonic percentile order, dropped-event/backlog invariants, bounded drain invariants.
- **Temporary thresholds (explicit):** conservative hardware-agnostic floors (`MIN_16T_SPEEDUP_VS_1T`, `MIN_DRAIN_EFFICIENCY`) used as non-catastrophic guards pending per-target calibration.

### Criterion benches (exploratory/trend)
- `benches/kernel_throughput.rs`
- `benches/boundary_cost.rs`
- `benches/tick_pressure.rs`

Rules:
- benches are for trend analysis and profiling, not certification verdicts.
- certification ownership remains in integration tests only.

## Regression-relevant metrics
- `ThroughputMetrics.ops_per_sec`, `ThroughputMetrics.ns_per_op`
- `ThroughputMetrics.latency.p95_ns`, `ThroughputMetrics.latency.p99_ns`
- `BoundaryMetrics.kernel_to_runtime_ns_per_event`
- `BoundaryMetrics.runtime_routing_ns_per_event`
- `BoundaryMetrics.runtime_to_observer_ns_per_event`
- `BoundaryMetrics.boundary_total_ns_per_event`
- `BoundaryMetrics.relative_slowdown_vs_baseline`
- `TickMetrics.deadline_miss_ticks`
- `TickMetrics.max_sustainable_ops_per_tick`
- `TickMetrics.drain_efficiency`

## Baseline classification
- **Pure kernel baseline:** `run_kernel_throughput` path that uses producer threads, MPSC transport, and core `EventBus` consumption only.
- **Boundary overhead:** additional cost measured by `run_boundary_cost` relative to direct kernel baseline iteration.
- **Tick pressure:** bounded fixed-tick drain measured by `run_tick_pressure` under explicit nanosecond budgets.

## Canonical evidence artifact
- `docs/canonical/ENGENE_2_0_FOUNDATION_PERF_EVIDENCE_2026-03-18.md`
