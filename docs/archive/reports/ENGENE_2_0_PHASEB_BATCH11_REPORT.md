# ENGENE 2.0 Phase B Batch 11 Report

## Scope
Batch 11 is a Phase B closure strike on transition-core performance with unchanged surfaces, workloads, thresholds, and severity rules.

## Structural changes
- `merge_shard_outputs` fast path now includes:
  - fixed-size frontier heap (no heap allocation in fast path),
  - no O(k) full-shard scan per emitted request,
  - deterministic comparator + shard-id tie-break preserved,
  - fallback full sort only when shard-local ordering precondition is not satisfied.
- `classify_materialize` bench surfaces now write into one contiguous request arena with deterministic shard ranges and equivalent ST/MT payload work.
- `classify_only` MT surface keeps fixed pool and uses deterministic fixed-range chunk writes.
- Gate script severity output remains direct in runtime output with latency ratio reporting and explicit `MT is Xx slower than ST` for critical scaling inversion.

## Measured truth (`bash scripts/check_transition_speed_law.sh --from-existing`)
- PASS: `classify_only_single_thread`
- PASS: `classify_only_multi_thread_x8`
- PASS: `classify_order`
- PASS: `classify_order_resolve`
- PASS: `deferred_queue_replay`
- MATERIAL_REGRESSION: `classify_materialize_multi_thread_x8`
- CRITICAL_FAILURE: `classify_materialize_single_thread`
- CRITICAL_FAILURE: `deterministic_merge_prep`
- CRITICAL_FAILURE: `classify_only_x8` (MT slower than ST)
- CRITICAL_FAILURE: `classify_materialize_x8` (MT slower than ST)

## Closure status
Phase B is NOT CLOSED because runtime gate remains failing.
