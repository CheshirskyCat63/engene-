# ENGENE 2.0 Phase B Batch 8 Report

## Scope
Batch 8 is transition-core hot-path optimization only. No benchmark semantics, workloads, or thresholds were weakened.

## Optimization focus
- classify/materialize hot loops in `benches/simulation_transition_core.rs`
- deterministic merge-prep path in `engine_runtime::simulation_core::orchestrator`
- ordered resolve hot path in `engine_runtime::simulation_core::orchestrator`

## Code changes
- Materialization benches now pre-initialize per-shard request storage and overwrite by index (no per-iteration push growth).
- Merge prep now has a deterministic O(n) fast path when shard streams are already globally ordered; fallback keeps existing deterministic sort+tie-break behavior.
- Ordered resolve path uses a direct hot loop for `None` tracer/handoff usage to reduce temporary object churn while preserving queue/metrics semantics.

## Benchmark/gate truth (unchanged)
- Surfaces unchanged from Batch 7.
- Workloads unchanged from Batch 7.
- Thresholds unchanged from Batch 7.
- Scaling requirements unchanged from Batch 7:
  - `classify_only_x8 >= 3.0x` (target 4.5x)
  - `classify_materialize_x8 >= 3.0x` (target 4.5x)

## Measured outcomes (post-optimization, from gate)
- PASS: `classify_only_single_thread`
- PASS: `classify_materialize_single_thread`
- FAIL: `classify_materialize_multi_thread_x8` (slightly above max-ms)
- PASS: `classify_order`
- PASS: `classify_order_resolve`
- PASS: `deferred_queue_replay`
- FAIL: `deterministic_merge_prep` (still over max-ms)
- FAIL: `classify_only_x8` scaling
- FAIL: `classify_materialize_x8` scaling

## Isolation truth
- Warm isolated compile proof passes via:
  - `bash scripts/check_transition_speed_law.sh --proof-warm-build`
- This compile proof is separate from runtime performance gate success.
