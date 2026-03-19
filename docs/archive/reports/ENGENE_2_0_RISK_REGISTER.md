# ENGENE 2.0 Risk Register

## Major technical risks
1. Hidden coupling in monolithic crate blocks clean workspace split.
2. Destruction + nav + streaming reconciliation inconsistencies.
3. Multi-level simulation desync causing implausible state jumps.
4. Audio propagation cost explosion in dense urban scenes.

## Architectural risks
- Boundary violations between engine/sdk/game reintroduced under delivery pressure.
- Game-specific rules leaking into engine crates under showcase pressure.
- SDK-specific debug/editor assumptions leaking into shipping runtime pathways.
- Legacy code migrated into new crates without ownership cleanup.
- Data model divergence between physics materials and audio/environment materials.
- `engine_runtime` becoming a new monolithic black hole.
- `engine_tools` becoming a generic misc sink.

## Performance risks
- Promotion/demotion storms causing reconstruction thrash and frame spikes.
- Excessive job fragmentation where scheduler overhead exceeds useful work.
- False sharing on packed arrays under multithread load.
- Event burst traffic causing allocation/backpressure spikes.
- Hot-loop random sparse access causing cache-miss dominated execution.
- Over-synchronization reducing net multithread gain.
- Debug/telemetry leakage into shipping hot paths.
- Insufficiently packed cooked data causing runtime parse/lookup overhead.

## Repository and handoff risks
- Folder/module structure technically valid but operationally unreadable for a new engineer.
- Dead scripts/binaries survive migration and create false entrypoints.
- Multiple “almost canonical” run/build/launch paths confuse ownership.

## Content-authoring risks
- Incomplete material authoring causes unrealistic or unstable outcomes.
- Break-state assets missing for many structures, forcing expensive runtime fallback.

## Tooling risks
- SDK lagging behind runtime features makes systems untestable by designers.
- Missing validators allows invalid content into showcase.

## Scope-creep risks
- Attempting exact simulation everywhere.
- Expanding from showcase-ready to full game completion within 2.0 window.

## Non-negotiables (must never be compromised)
- Dependency direction and ECS boundary discipline.
- Budget instrumentation before feature lock-in.
- Deterministic-enough persistence transitions L0<->L3.
- Showcase scene proving integrated behavior, not isolated tech demos.
- No hot-path heap churn without explicit budget and profiling evidence.
- No subsystem may rely on global lock contention for normal correctness.
- Hot runtime data must remain packed and batch-processable.
- Multithread speedups must not violate determinism classes.
- Engine, SDK, and Game remain product-distinct in ownership and structure.
- Legacy cleanup precedes major crate split.
- Repo readability and handoff clarity are release-quality concerns.

## What can kill 2.0 if done wrong
- No enforced multi-level simulation (15k actor target becomes impossible).
- No hybrid destruction strategy (either fake-only or full-physics-only extremes fail).
- SDK deficits preventing authoring/diagnosis.
- Performance law ignored until late integration.
- Product boundaries eroded by demo-pressure shortcuts.

## What must be phased, not rushed
- Workspace split before heavy feature growth.
- Material schema unification before ballistic/audio/environment coupling.
- Population abstraction before showcase scale targets.
- Concurrency/data-layout/CPU-law baseline before deep feature escalation.
- Repository sanitation before workspace split.
