# ENGENE 2.0 Phase B Batch 10 Report

## Scope
Batch 10 continues transition-core structural performance work with unchanged benchmark surfaces, workloads, and thresholds.

## Priority-ordered structural work completed
1) `merge_shard_outputs` rewrite:
- removed heap allocation from fast path cursor state
- replaced O(k)-per-output-element full-scan selection with fixed-size heap (O(log k) frontier updates)
- preserved deterministic comparator and shard-id tie-break
- retained full-sort fallback when shard-local order precondition is not satisfied

2) `classify_materialize` layout rewrite:
- introduced contiguous request arena + deterministic shard ranges
- ST and MT both classify + materialize identical request payloads
- shard outputs are filled from the same arena ranges (no lighter MT semantics)

3) `classify_only` MT rewrite:
- retained fixed thread-pool reuse
- switched to deterministic fixed ranges + scope fan-out with reduced coordination logic

4) gate severity output:
- script now emits PASS / NEAR_MISS / MATERIAL_REGRESSION / CRITICAL_FAILURE directly
- latency failures include max-ratio (`mean/max`)
- scaling critical failures include explicit "MT is Xx slower than ST"

## Current truth (measured)
From `bash scripts/check_transition_speed_law.sh --from-existing`:
- PASS: `classify_only_single_thread`
- PASS: `classify_only_multi_thread_x8`
- CRITICAL_FAILURE: `classify_materialize_single_thread`
- CRITICAL_FAILURE: `classify_materialize_multi_thread_x8`
- PASS: `classify_order`
- PASS: `classify_order_resolve`
- PASS: `deferred_queue_replay`
- CRITICAL_FAILURE: `deterministic_merge_prep`
- CRITICAL_FAILURE: `classify_only_x8` scaling
- CRITICAL_FAILURE: `classify_materialize_x8` scaling

## Acceptance
- Batch 9 remains NOT ACCEPTED AS FINAL.
- Batch 10 is also NOT ACCEPTED AS FINAL because the runtime gate still fails.
