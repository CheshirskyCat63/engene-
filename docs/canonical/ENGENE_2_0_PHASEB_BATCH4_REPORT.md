# ENGENE 2.0 Phase B Batch 4 Report

## Scope
Phase B Batch 4 locks transition-core speed as explicit architecture law and aligns code/docs/contracts for deterministic, bounded, benchmarkable runtime orchestration.

## What was added
- Speed law contract surface:
  - `TransitionSpeedLawTargets` in runtime contracts.
- Canonical numeric sync:
  - policy defaults remain `max_promotions_per_frame = 64`, `max_demotions_per_frame = 256`.
  - queue policy defaults are now explicit (`capacity=4096`, `replay_batch_limit=512`, `max_retry_attempts=3`, `max_age_ticks=240`, `retry_cooldown_ticks=1`).
- Deterministic shard-merge prep:
  - `TransitionShardOutput`, `TransitionShardMergePolicy`.
  - `TransitionOrchestrator::merge_shard_outputs(...)` deterministic reduction rule:
    1) cap inputs by `max_merge_inputs`
    2) sort by transition ordering policy, tie-break by `shard_id`
    3) emit one bounded merged sequence.
- Deferred queue maturity:
  - deferred entry retry/age metadata (`retry_attempts`, `next_retry_tick`).
  - replay window (`drain_ready_into`) and bounded retention/drop semantics (`DroppedByQueuePolicy`).
- Hot-path discipline:
  - no formatted string creation in resolve hot path (`snapshot_key: None`).
  - reusable preallocated transition batch and bounded queue replay fill.
  - backpressure path uses queue deferral rather than `Vec` growth.
- Benchmark/proof harness:
  - `benches/simulation_transition_core.rs` with focused microbenches for classify/order/resolve/deferred/merge-prep.

## Transition-core speed law (hard)
- `update_simulation_levels` / transition-core typical: <= 0.20 ms.
- transition-core ceiling: <= 0.50 ms.
- hard-fail spike: > 0.90 ms.
- single-thread floor: >= 500k transitions/sec.
- single-thread target: >= 800k transitions/sec.
- multi-thread target profile: >= 2.0M transitions/sec.
- scaling on 8 logical workers: minimum >= 3.0x, target >= 4.5x.
- zero steady-state heap allocations on classify/order/resolve path.
- zero string formatting on hot path.
- zero `Vec` growth in steady state.
- zero global mutexes.
- zero blocking waits.
- max one deterministic merge barrier per frame phase.
- queue typical occupancy < 25% capacity.
- queue stress occupancy < 70% capacity.
- >= 90% occupancy is failure unless explicitly justified.
- dev metrics overhead < 5%.
- shipping-like metrics overhead < 1%.

## Law-compliance notes
- Runtime owns transition-core contracts/orchestration/queue/merge-prep/metrics surfaces only.
- Domain truth ownership remains with world/game/render/audio/physics.
- No gameplay features and no deep subsystem migrations.

## Validation
- `bash scripts/check_dependency_direction.sh`
- `bash scripts/check_ecs_direct_access.sh`
- `cargo check --workspace -q`
