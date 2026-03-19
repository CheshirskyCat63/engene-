# ENGENE 2.0 Phase B Batch 7 Report

## Scope
Phase B Batch 7 is a truth-and-proof cleanup pass for transition-core speed-law validation. It does not expand gameplay or subsystem ownership.

## What changed
- Replaced prior classify scaling surfaces with workload-equivalent ST/MT pairs:
  - `simulation_core/classify_only_single_thread`
  - `simulation_core/classify_only_multi_thread_x8`
  - `simulation_core/classify_materialize_single_thread`
  - `simulation_core/classify_materialize_multi_thread_x8`
- Kept transition-core regression surfaces:
  - `simulation_core/classify_order`
  - `simulation_core/classify_order_resolve`
  - `simulation_core/deferred_queue_replay`
  - `simulation_core/deterministic_merge_prep`
- Tightened deterministic merge semantics:
  - `deterministic_merge_prep` now measures merge preparation only on pre-materialized shard outputs.
- Upgraded gate semantics in `scripts/check_transition_speed_law.sh`:
  - per-surface mean latency + throughput enforcement for every surface,
  - separate scaling checks for fair pairs only (`classify_only_x8`, `classify_materialize_x8`).

## Exact thresholds in gate (must match baseline)
- `simulation_core/classify_only_single_thread`: max 0.20 ms, min 800000 transitions/sec
- `simulation_core/classify_only_multi_thread_x8`: max 0.50 ms, min 2000000 transitions/sec
- `simulation_core/classify_materialize_single_thread`: max 0.35 ms, min 500000 transitions/sec
- `simulation_core/classify_materialize_multi_thread_x8`: max 0.50 ms, min 1200000 transitions/sec
- `simulation_core/classify_order`: max 0.50 ms, min 500000 transitions/sec
- `simulation_core/classify_order_resolve`: max 0.50 ms, min 500000 transitions/sec
- `simulation_core/deferred_queue_replay`: max 0.50 ms, min 500000 transitions/sec
- `simulation_core/deterministic_merge_prep`: max 0.50 ms, min 500000 transitions/sec

Scaling thresholds:
- `classify_only_x8` minimum `>= 3.0x`, target `>= 4.5x`
- `classify_materialize_x8` minimum `>= 3.0x`, target `>= 4.5x`

## Compile proof truth split
- Compatibility compile check:
  - `cargo bench --bench simulation_transition_core --no-run`
- Isolation compile truth:
  - `cargo rustc --bench simulation_transition_core --profile bench -vv -- -C debuginfo=0`
  - executed through:
    - `bash scripts/check_transition_speed_law.sh --proof-cold-build`
    - `bash scripts/check_transition_speed_law.sh --proof-warm-build`
  - fails when compile log includes root runtime bins (`src/bin/engene_{game,sdk,headless,test}.rs`).

## Law-compliance notes
- No gameplay/system expansion.
- No world/game/render/audio/physics ownership changes.
- Transition-core proof surfaces remain deterministic and allocation-disciplined with debug-only no-growth assertions on reusable materialization buffers.

## Validation commands
- `cargo fmt --all`
- `bash scripts/check_dependency_direction.sh`
- `bash scripts/check_ecs_direct_access.sh`
- `cargo check --workspace -q`
- `cargo bench --bench simulation_transition_core --no-run`
- `bash scripts/check_transition_speed_law.sh --proof-warm-build`
- `bash scripts/check_transition_speed_law.sh --from-existing`
- `bash scripts/check_transition_speed_law.sh --proof-cold-build` (if environment time budget allows)
