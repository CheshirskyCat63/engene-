# ENGENE 2.0 Budget Contracts

## Purpose
Convert performance strategy into enforceable subsystem contracts with owners, limits, and fail conditions.

## Contract fields (mandatory per subsystem)
- Hard budget.
- Soft budget.
- Burst budget.
- Recovery window.
- Degradation steps.
- Required telemetry fields.
- Fail-gate conditions.
- Mitigation owner.

## Physics/Destruction contract
- Hard: max active rigid debris and max collapse solves/frame.
- Soft: preferred collapse + debris cadence.
- Burst: temporary overshoot window during major explosions.
- Recovery: return under soft budget in bounded frames.
- Degrade first: non-critical debris fidelity and persistence length.
- Telemetry: active debris, collapse queue depth, solve time.
- Fail gate: sustained hard-budget breach without recovery.

## AI/Population contract
- Hard: max L0 combatants and max promotions/frame.
- Soft: target L1/L2 update cadence.
- Burst: tactical spike allowance near player events.
- Recovery: promotion throttle and demotion stabilization.
- Degrade first: low-priority L1 update frequency.
- Telemetry: per-level agent counts, promotion queue, decision time.
- Fail gate: runaway L0 population causing persistent budget breach.

## Audio contract
- Hard: voice caps per class and obstruction query caps per frame bucket.
- Soft: target ambience/foley distribution.
- Burst: short weapon/destruction event spikes.
- Recovery: priority stealing and ambience downshift.
- Degrade first: ambience richness before critical cues.
- Telemetry: active voices by class, dropped/aggregated events, query cost.
- Fail gate: critical gameplay cues dropped due to budget mismanagement.

## Environment contract
- Hard: max active wetness cells and fire spread updates/tick.
- Soft: preferred far-field weather cadence.
- Burst: storm-transition spike allowance.
- Recovery: coarsen far-field and reduce low-priority cells.
- Degrade first: distant weather resolution and non-critical cell updates.
- Telemetry: active cells, spread operations, weather tick cost.
- Fail gate: inability to maintain fire/wetness correctness in active bubble.

## Render contract
- Hard: frame-time envelope for render + post.
- Soft: quality tier target for platform profile.
- Burst: short over-budget during large scene transitions.
- Recovery: degrade ladder in strict order.
- Degrade first: volumetrics, then shadows, then non-critical post.
- Telemetry: pass timings, quality tier state, fallback counters.
- Fail gate: sustained frame collapse without automatic recovery.
