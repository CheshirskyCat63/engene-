# Phase B Transition Core Bench Baseline (Batch 14)

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

## Batch 14 measured result snapshot (`--from-existing` on 2026-03-18, container)
- NEAR_MISS: `deterministic_merge_prep` (0.510372ms, 256,816,469/s)
- PASS: `classify_only_single_thread` (0.081967ms, 1,599,079,597/s)
- PASS: `classify_only_multi_thread_x8` (0.136433ms, 960,708,913/s)
- PASS: `classify_materialize_single_thread` (0.266687ms, 491,483,056/s)
- PASS: `classify_materialize_multi_thread_x8` (0.231011ms, 567,384,203/s)
- PASS: `classify_order` (0.034100ms, 480,472,320/s)
- PASS: `classify_order_resolve` (0.195670ms, 83,732,994/s)
- PASS: `deferred_queue_replay` (0.007859ms, 130,303,396/s)
- CRITICAL_FAILURE: scaling proof blocked on this host by environment guard (`available_cpus=3`, `required>=8`) before x8 scaling ratio evaluation.

## Prior Batch 12 measured result snapshot (`--from-existing` on 2026-03-18)
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
