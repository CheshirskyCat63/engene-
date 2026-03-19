# ENGENE 2.0 CPU Performance Law

## Purpose
Define enforceable CPU-side hot-path contracts for frame stability under flagship 2.0 scenarios.

## CPU-first law
Feature correctness is insufficient. A subsystem is not production-ready unless it defines and proves:
- hot-path data packing,
- batching model,
- safe parallel execution,
- synchronization bounds,
- heap churn avoidance,
- degradation behavior under stress.

## Frame decomposition priorities (CPU)
1. L0 simulation truth updates.
2. Player-near collision and ballistic solving.
3. Promotion/demotion safety work.
4. Active combat AI perception/decision.
5. Event merge/apply for authoritative channels.
6. Local environment updates (fire/wetness).
7. Audio hearing and critical propagation.
8. Far-field summaries.
9. Debug/telemetry export.

## Hot-path classification
- Tier A: must remain within strict CPU budget every frame class.
- Tier B: bounded spikes allowed with recovery window.
- Tier C: degradable support work.

## Hot-loop prohibitions
- String/hash map lookup in Tier A loops unless pre-resolved.
- Virtual dispatch in Tier A loops unless profiled and justified.
- Per-entity heap allocation in update path.
- Unbounded indirect pointer chasing across many sparse storages.

## Synchronization budget law
- Hard cap on barrier count per frame class.
- Hard cap on blocking waits in gameplay-critical phases.
- Lock contention beyond threshold emits P1 perf signal.
- No lock acquisition inside N hottest loops (defined by telemetry baseline).

## Job dispatch overhead law
- Dispatch granularity must be measured against useful compute ratio.
- Excessive micro-jobs causing scheduler overhead > useful work are forbidden.
- Job batching/combining is required for sub-threshold workloads.

## Vectorization and batching targets
- Candidate Tier A loops must have vectorization feasibility review.
- Batch APIs are mandatory for physics, AI, environment, and audio hot compute paths.

## Subsystem-specific CPU laws
- ECS: narrowest dense storage drives iteration.
- AI: L0 batched by role/archetype, L2/L3 table-driven.
- Environment: chunk/tile updates with active/dirty masks.
- Audio: batched propagation/hearing queries with relevance culling.
- Destruction: solver chunking and staged collapse application.

## Allocation visibility law
- Allocator activity on hot frame path must be telemetry-visible.
- Steady-state showcase run must demonstrate near-zero Tier A allocation churn.

## Evidence requirements
- Phase waterfall timings.
- Barrier/wait timeline.
- Job utilization and overhead report.
- Allocation hotspot report.
- Baseline vs head benchmark diffs for canonical scenarios.
