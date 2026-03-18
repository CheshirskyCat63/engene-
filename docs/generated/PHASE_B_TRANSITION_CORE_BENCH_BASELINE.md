# Phase B Transition Core Bench Baseline (Batch 11)

## Harness
- Bench target: `simulation_transition_core` (criterion).
- File: `benches/simulation_transition_core.rs`.
- Regression gate script: `scripts/check_transition_speed_law.sh`.

## Covered surfaces + gate semantics
Each surface must satisfy both checks:
1) mean latency <= max-ms threshold
2) throughput >= min transitions/sec threshold

| surface | transitions/iter | max-ms | min transitions/sec |
|---|---:|---:|---:|
| `simulation_core/classify_only_single_thread` | 131072 | 0.20 | 800000 |
| `simulation_core/classify_only_multi_thread_x8` | 131072 | 0.50 | 2000000 |
| `simulation_core/classify_materialize_single_thread` | 131072 | 0.35 | 500000 |
| `simulation_core/classify_materialize_multi_thread_x8` | 131072 | 0.50 | 1200000 |
| `simulation_core/classify_order` | 16384 | 0.50 | 500000 |
| `simulation_core/classify_order_resolve` | 16384 | 0.50 | 500000 |
| `simulation_core/deferred_queue_replay` | 1024 | 0.50 | 500000 |
| `simulation_core/deterministic_merge_prep` | 131072 | 0.50 | 500000 |

## Fair scaling pairs (workload-equivalent)
- Pair A (`classify_only_*`): classify distances into preallocated `SimulationLevel` buffers only.
- Pair B (`classify_materialize_*`): classify + materialize transition requests with equivalent request construction work on ST/MT paths.

Scaling checks in gate:
- `classify_only_x8 = classify_only_single_thread_mean / classify_only_multi_thread_x8_mean`
  - required minimum: `>= 3.0x`
  - target visibility: `>= 4.5x`
- `classify_materialize_x8 = classify_materialize_single_thread_mean / classify_materialize_multi_thread_x8_mean`
  - required minimum: `>= 3.0x`
  - target visibility: `>= 4.5x`

## Deterministic merge-prep surface semantics
- `simulation_core/deterministic_merge_prep` measures merge-prep only.
- Materialized shard outputs are prepared outside measured closure.
- Measured closure performs deterministic `merge_shard_outputs(...)` with reusable output/scratch buffers.

## Isolation truth split
- Compatibility compile check (Cargo bench behavior):
  - `cargo bench --bench simulation_transition_core --no-run`
- Isolated compile truth (no root runtime bin compile hits allowed):
  - `bash scripts/check_transition_speed_law.sh --proof-cold-build`
  - `bash scripts/check_transition_speed_law.sh --proof-warm-build`
  - internally executes `cargo rustc --bench simulation_transition_core --profile bench -vv -- -C debuginfo=0`

## Runtime proof commands
- Full runtime bench + gate:
  - `bash scripts/check_transition_speed_law.sh`
- Gate from existing Criterion output:
  - `bash scripts/check_transition_speed_law.sh --from-existing`

## Operational expectations
- no steady-state heap growth in classify/materialize/merge hot path buffers
- no string formatting in transition-core hot path
- deterministic ordering + merge policy maintained


## Batch 11 measured result snapshot
- Gate run (`bash scripts/check_transition_speed_law.sh --from-existing`) currently reports PASS for classify_only_single_thread, classify_only_multi_thread_x8, classify_order, classify_order_resolve, deferred_queue_replay.
- Current FAIL set: classify_materialize_single_thread, classify_materialize_multi_thread_x8, deterministic_merge_prep, classify_only_x8 scaling, classify_materialize_x8 scaling.
