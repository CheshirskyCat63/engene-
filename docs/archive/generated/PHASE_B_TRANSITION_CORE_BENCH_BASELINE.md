# Phase B Transition Core Bench Baseline (Batch 15)

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

## Batch 15 validated >=8 CPU host snapshot (provided rerun truth)
- `classify_only_single_thread` mean=`0.038812ms`
- `classify_only_multi_thread_x8` mean=`0.016695ms`
- `classify_only_x8` ratio=`2.325x` (`MATERIAL_REGRESSION`, required `>=3.0x`)
- `classify_materialize_single_thread` mean=`0.136389ms`
- `classify_materialize_multi_thread_x8` mean=`0.046846ms`
- `classify_materialize_x8` ratio=`2.911x` (`NEAR_MISS`, required `>=3.0x`)
- Other surfaces: PASS

## Batch 15 measured container snapshot (`--from-existing` on 2026-03-18)
- NEAR_MISS: `deterministic_merge_prep` (0.500151ms, 262,064,737/s)
- PASS: `classify_only_single_thread` (0.084338ms, 1,554,132,345/s)
- PASS: `classify_only_multi_thread_x8` (0.092521ms, 1,416,680,097/s)
- PASS: `classify_materialize_single_thread` (0.285725ms, 458,734,041/s)
- PASS: `classify_materialize_multi_thread_x8` (0.288009ms, 455,097,164/s)
- PASS: `classify_order` (0.034892ms, 469,569,464/s)
- PASS: `classify_order_resolve` (0.174255ms, 94,023,222/s)
- PASS: `deferred_queue_replay` (0.007100ms, 144,228,505/s)
- CRITICAL_FAILURE: scaling proof blocked on this host by environment guard (`available_cpus=3`, `required>=8`) before `classify_only_x8` / `classify_materialize_x8` evaluation.

## Batch 12 measured result snapshot (`--from-existing` on 2026-03-18)
- PASS: `classify_only_single_thread` (0.058003ms, 2,259,757,508/s)
- PASS: `classify_only_multi_thread_x8` (0.114272ms, 1,147,020,746/s)
- PASS: `classify_materialize_single_thread` (0.193818ms, 676,264,673/s)
- PASS: `classify_materialize_multi_thread_x8` (0.224837ms, 582,965,463/s)
- PASS: `classify_order` (0.024838ms, 659,638,310/s)
- PASS: `classify_order_resolve` (0.125528ms, 130,520,602/s)
- PASS: `deferred_queue_replay` (0.005105ms, 200,577,306/s)
- PASS: `deterministic_merge_prep` (0.398503ms, 328,910,609/s)
- CRITICAL_FAILURE: scaling proof blocked on this host by environment guard (`available_cpus=3`, `required>=8`) before `classify_only_x8` / `classify_materialize_x8` evaluation.

Closure status: **Phase B remains open** until rerun on an environment with at least 8 logical CPUs for honest x8 scaling validation.
