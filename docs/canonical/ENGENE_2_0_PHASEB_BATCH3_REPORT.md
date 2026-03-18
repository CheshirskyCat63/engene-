# ENGENE 2.0 Phase B Batch 3 Report

## Scope
Phase B Batch 3 matures `engine_runtime::simulation_core` to a deterministic, bounded, multithread-ready transition pipeline shape with explicit queue semantics and explicit performance targets.

## What was added
- Deterministic batch ordering contract:
  - `TransitionOrderingPolicy` with explicit `SubjectThenFromToReason` compare rule.
  - `TransitionOrchestrator::order_batch(...)` now applies deterministic ordering before resolution.
- Runtime-owned deferred queue contract:
  - `DeferredTransitionQueue`, `DeferredTransitionEntry`, and `DeferredTransitionQueueSnapshot`.
  - deterministic FIFO replay via `drain_into(...)` and bounded capacity with overflow accounting.
- Transition metrics expanded with queue observability:
  - `queue_depth`, `queue_capacity`, `queue_overflow_dropped`.
- Hot-path allocation reduction:
  - reusable transition batch buffer in monolith simulation system.
  - removal of per-transition formatted snapshot key allocation (`snapshot_key: None` in hot path).
- Multithread-readiness prep:
  - explicit ordering contract and queue ownership surfaces allow future parallel classify + single deterministic merge/resolve point.

## Transition-core performance targets (baseline)
- `update_simulation_levels`: typical <= 0.20 ms, ceiling <= 0.50 ms, hard-fail spike > 0.90 ms.
- Classification throughput: >= 500k/sec single-thread floor, >= 800k/sec single-thread target, >= 2.0M/sec multi-thread target profile.
- Scaling expectation: >= 3.0x over single-thread minimum and >= 4.5x target on 8 logical workers for pure classification/budget pass.
- Default budgets: `max_promotions_per_frame = 64`, `max_demotions_per_frame = 256`.
- Allocation policy: zero steady-state heap allocations in per-entity transition decision path.
- Synchronization policy: zero blocking waits and zero global locks in per-entity hot classification path.
- Merge policy: max one bounded deterministic merge point per frame phase.
- Metrics overhead target: <5% dev builds, <1% shipping-like profile.

## Compatibility strategy
- `src/simulation/simulation.rs` now owns deferred queue and reusable batch buffer.
- `src/simulation/activation.rs` drains deferred queue, builds current-frame requests, orders deterministically, resolves bounded transitions, and re-enqueues deferred outcomes.

## Law-compliance notes
- Runtime owns transition orchestration contracts, ordering, queue semantics, and observability.
- Domain truth ownership remains in world/game/render/audio/physics systems.
- No gameplay feature additions and no deep subsystem migrations.

## Validation
- `bash scripts/check_dependency_direction.sh`
- `bash scripts/check_ecs_direct_access.sh`
- `cargo check --workspace -q`
