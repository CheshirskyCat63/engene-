# ENGENE 2.0 Phase B Batch 12 Report

## Scope
Batch 12 is a narrow transition-core perf closure pass focused only on remaining bottleneck groups:
- `classify_materialize_single_thread`
- `classify_materialize_multi_thread_x8` (+ scaling)
- `deterministic_merge_prep`
- `classify_only_x8` scaling

No threshold/workload/surface/severity changes were made.

## Structural changes
- Bench MT classify/materialize paths moved from `par_chunks_mut(...).enumerate()` to fixed-range scoped shard jobs over precomputed ranges.
- MT classify/materialize writes use direct indexed writes into disjoint shard slices (no post-stitch stage).
- ST materialize path now performs direct inline payload construction in hot loop (no helper-call tax).
- Merge comparator path now compares unpacked fields directly and removes tuple-key construction/accessor churn in comparator/concat checks.

## Measured truth (`bash scripts/check_transition_speed_law.sh --from-existing`, 2026-03-18)
- PASS: `classify_only_single_thread` mean=0.058003ms throughput=2,259,757,508/s
- PASS: `classify_only_multi_thread_x8` mean=0.114272ms throughput=1,147,020,746/s
- PASS: `classify_materialize_single_thread` mean=0.193818ms throughput=676,264,673/s
- PASS: `classify_materialize_multi_thread_x8` mean=0.224837ms throughput=582,965,463/s
- PASS: `classify_order` mean=0.024838ms throughput=659,638,310/s
- PASS: `classify_order_resolve` mean=0.125528ms throughput=130,520,602/s
- PASS: `deferred_queue_replay` mean=0.005105ms throughput=200,577,306/s
- PASS: `deterministic_merge_prep` mean=0.398503ms throughput=328,910,609/s
- CRITICAL_FAILURE: environment invalid for x8 scaling proof (available_cpus=3, required>=8)
- `classify_only_x8` / `classify_materialize_x8` were intentionally not evaluated on this host

## Closure status
Phase B is **NOT CLOSED** in Batch 12.

Reason: runtime scaling proof is blocked by host capacity (<8 logical CPUs). Phase B remains open until rerun on >=8 CPU environment; no semantic/threshold cheating applied.

## Outside-scope notes
- No code outside allowed transition-core perimeter was modified in this batch.
- If broader scaling closure is required, likely next blocker is thread-orchestration overhead vs tiny per-element classify work at current workload size.
