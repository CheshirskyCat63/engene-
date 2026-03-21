# ENGENE 2.0 Performance Strategy

## Budget model (frame-time percentages, target 60 FPS class)
- Render + post: 30-40%
- Physics + destruction: 20-25%
- AI/population simulation: 15-20%
- Environment (fire/water/weather): 8-12%
- Audio: 5-8%
- Streaming/persistence/background jobs: 8-12%

## Worst-case stress budgets
- Combined destruction + fire + rain + mass combat scene must remain within degradation policy.
- Frame-time spike above threshold is tolerated only for bounded windows and must recover automatically.

## CPU hot-path priorities
1. L0 simulation truth.
2. Player-near collision/ballistics.
3. Promotion/demotion safety work.
4. Active combat AI perception/decision.
5. Event merge/apply.
6. Local environment updates.
7. Audio hearing/critical propagation.
8. Far-field summaries.
9. Debug/telemetry export.

## Frame spike policy
1. Detect spike source per subsystem.
2. Apply subsystem-specific degradation ladder immediately.
3. Record spike marker and mitigation action in metrics stream.
4. Recover quality gradually after stability window.

## Synchronization budget
- Hard cap on barrier count per frame class.
- Hard cap on blocking waits in gameplay-critical phases.
- Lock contention above threshold emits P1 performance signal.
- No lock acquisition inside hottest loop set from baseline telemetry.

## Scheduler priority rules
1. Gameplay-critical L0 simulation.
2. Player-near collision and ballistics.
3. Audio critical events and AI hearing hooks.
4. L1 simulation and environment updates.
5. L2/L3 aggregate simulation.
6. Non-critical visual polish updates.

## Simulation-level budgets
- L0 entities hard capped by active bubble budget.
- L1 updates at reduced cadence.
- L2 region sim on fixed interval ticks.
- L3 global model updated at coarse cadence.

## Destruction budgets
- Max concurrent structural events per frame.
- Debris actor cap + lifetime decay.
- Fallback to decal/rubble proxies when cap exceeded.

## Particle/debris budgets
- Tiered emitters by camera relevance.
- Weather/impact particles share pooled allocators.

## Audio budgets
- Voice cap by class (critical gameplay, ambience, foley).
- Event-rate limiter for burst destruction/fire scenes.
- Prioritized voice stealing with debugging telemetry.

## Weather/fire/water budgets
- Active cell cap per loaded region.
- Dirty-region updates only.
- Far-field weather rendered from precomputed/front-parametric model.

## Population budgets
- L0 actor cap + promotion throttling.
- L2/L3 aggregated updates in batched jobs.
- Soft degradation: behavior simplification before despawn.

## Allocation policy
- Zero/near-zero steady-state allocations on main update path.
- Pooled allocations for burst systems.
- Allocator activity on hot frame path is telemetry-visible.

## Cache and bandwidth risk policy
- Benchmark dense urban + fire + debris + mass AI combined stress case.
- Track memory-bandwidth-sensitive counters and top random-access loops.
- Require locality optimization before adding fidelity in same subsystem.

## Per-subsystem first degradation under stress
- Render: reduce volumetric resolution first.
- Physics/destruction: reduce non-critical debris fidelity first.
- AI: reduce update frequency for low-priority L1 agents first.
- Environment: coarsen far-field cell updates first.
- Audio: reduce ambience/polyphony before critical gameplay cues.

## Renderer degradation ladder
1. Reduce volumetrics resolution.
2. Reduce shadow cascade quality.
3. Reduce debris/particle draw density.
4. Increase far-field impostor substitution.
5. Drop non-critical post effects.

## Observability export format
- Structured JSON lines per frame for timings and counters.
- Snapshot bundles with scenario ID, build hash, content hash, platform profile.
- CSV summary export for budget pass/fail dashboards.

## Observability requirements
- Per-subsystem timings in runtime HUD and SDK dashboard.
- Budget-overrun alerts with frame captures.
- Scenario-level benchmark scripts with pass/fail thresholds.
