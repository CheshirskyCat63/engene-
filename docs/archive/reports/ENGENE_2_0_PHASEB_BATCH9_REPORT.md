# ENGENE 2.0 Phase B Batch 9 Report

## Scope
Batch 9 performs structural transition-core performance surgery with unchanged benchmark surfaces, workloads, and thresholds.

## Structural changes
- Reworked multithread classify/materialize execution to fixed shard task fan-out via `ThreadPool::scope`, with shard-local mutable slices and deterministic shard ranges.
- Reworked materialize loops to direct indexed writes with inline request construction (`PromotionRequest`) on both ST/MT paths.
- Reworked deterministic merge-prep benchmark to reuse prepared shard inputs and reusable output/scratch buffers per iteration (no per-iteration shard clone setup).
- Reworked runtime merge implementation to a deterministic k-way merge fast path for sorted shard streams with shard-id tie break, with fallback flatten+sort for unsorted inputs.

## Truth guardrails (unchanged)
- Surfaces unchanged from Batch 8.
- Workloads unchanged from Batch 8.
- Thresholds unchanged from Batch 8.
- Scaling requirements unchanged from Batch 8.

## Measured outcome
From `bash scripts/check_transition_speed_law.sh --from-existing` after Batch 9 code changes:
- PASS: `classify_only_single_thread`
- PASS: `classify_only_multi_thread_x8`
- FAIL: `classify_materialize_single_thread`
- FAIL: `classify_materialize_multi_thread_x8`
- PASS: `classify_order`
- PASS: `classify_order_resolve`
- PASS: `deferred_queue_replay`
- FAIL: `deterministic_merge_prep`
- FAIL: `classify_only_x8` scaling
- FAIL: `classify_materialize_x8` scaling

## Notes
- `deterministic_merge_prep` improved materially vs prior run state but remains over threshold on this host.
- MT scaling remains below required minima on this host despite structural fan-out rewrite.
