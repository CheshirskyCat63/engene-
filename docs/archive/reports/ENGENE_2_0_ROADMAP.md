# ENGENE 2.0 Roadmap

## Phase 0 — Cleanup / Archaeology / Repository Sanitation
- Goals:
  - Remove dead code, stale assets, orphan scripts, obsolete binaries, and duplicate contract types before split.
  - Establish readable handoff-friendly baseline.
- Likely touched: legacy module paths, scripts, obsolete bins, docs indexes, archive/quarantine folders.
- Deliverables:
  - Dead code inventory.
  - Unused asset/script inventory.
  - Legacy path map.
  - Duplicate type/utility inventory.
  - Archive/delete decision list.
  - Folder naming normalization plan.
  - Ownership/readability stubs.
- Acceptance:
  - Unused modules removed or explicitly quarantined.
  - Orphan scripts/tools deleted or moved with status markers.
  - Duplicate utility/type definitions inventoried.
  - Legacy launch paths mapped and classified.
  - Handoff-oriented repository overview exists.
- Mini-tooling slice:
  - Unused module/file scan.
  - Import/reference graph report.
  - Duplicate symbol/type heuristic scan.
  - "last referenced / owner / status" manifest generator.
- Evidence commands:
  - `cargo check --workspace`
  - `rg "TODO\(legacy\)|deprecated|obsolete" src docs scripts`
  - `bash scripts/check_dependency_direction.sh`
- Docs updates: repo hygiene/handoff + product boundaries + required output files.
- Demo at end: clean repo map with canonical entrypoints only.

## Phase A — Workspace / ownership split
- Goals: introduce crate/workspace boundaries, move thin apps, preserve current behavior.
- Likely touched: `Cargo.toml`, `src/lib.rs`, `src/app/*`, `src/bin/*`, new `crates/*`, `apps/*`.
- Blockers: API boundary definition, shared serialization contracts.
- Acceptance:
  - Workspace compiles.
  - Legacy reverse imports = 0.
  - Each moved module has owner crate.
  - No duplicate shared contract types across crates.
  - Crate-level dependency-direction gate works.
  - Apps launch through public APIs only.
- Mini-tooling slice:
  - Dependency graph report generator.
  - Public API surface report for each crate.
  - Hot-path inventory baseline.
  - Allocation hotspot baseline.
- Evidence commands:
  - `cargo check --workspace`
  - `bash scripts/check_dependency_direction.sh`
  - `bash scripts/check_ecs_direct_access.sh`
- Docs updates: architecture + dependency law + engine constitution.
- Demo at end: old executables still launch via new app crates.

## Phase B — Simulation core and world model
- Goals: implement L0-L3 scheduler, promotion/demotion, persistence handoff.
- Likely touched: `crates/engine_runtime`, `crates/engine_world`, migrated simulation modules.
- Blockers: deterministic reconciliation rules.
- Acceptance: stable world continuity through streaming transitions with documented authority rules.
- Mini-tooling slice:
  - Sim-level overlay minimum.
  - Promotion/demotion trace inspector minimum.
  - Promotion/demotion cost profiler.
  - Region/state merge cost breakdown.
- Evidence commands:
  - `cargo test --test world_streaming -- --nocapture`
  - `cargo test --test persistence_full -- --nocapture`
- Docs: simulation strategy + AI architecture + event model + entity lifecycle + determinism policy.
- Demo: moving across zones visibly changes simulation levels.

## Phase C — Destruction and material system
- Goals: layered penetration + hybrid fracture + structural collapse.
- Likely touched: `crates/engine_physics`, `crates/engine_world`, material schemas.
- Blockers: canonical schema law adoption.
- Acceptance: tile/support/concrete sequence and explosive collapse showcased.
- Mini-tooling slice:
  - Penetration trace viewer minimum.
  - Structural support state overlay minimum.
  - Penetration batch benchmark.
  - Destruction locality benchmark.
  - Debris pool utilization monitor.
- Evidence commands:
  - `cargo test --test physics_body_combat -- --nocapture`
  - `cargo test --test vertical_slice -- --nocapture`
- Docs: destruction program + schema law + data layout policy.
- Demo: ballistic and explosive destruction sequence.

## Phase D — Fire / water / weather
- Goals: coupled fire-wetness-weather with local/far-field split.
- Likely touched: `crates/engine_world`, `crates/engine_render`, content schemas.
- Blockers: weather front representation and persistence model.
- Acceptance: single-source-of-truth SkyTime/WeatherController + immutable frame snapshot contract + weather edge-event policy, then wind-driven fire, rain suppression, leak behavior, distant storms.
- Mini-tooling slice:
  - Fire/wetness/weather debug overlays minimum.
  - Active-cell update heatmap.
  - Wetness/fire memory bandwidth benchmark.
- Evidence commands:
  - `cargo test --test runtime_systems -- --nocapture`
  - `cargo test --test render_pipeline -- --nocapture`
- Docs: environment program + content pipeline + runtime truth model + memory budgets + architecture.
- Demo: dynamic weather shift altering fire/wetness outcomes.

## Phase E — Audio and perception
- Goals: occlusion/obstruction, material impact audio, AI hearing hooks.
- Likely touched: `crates/engine_audio`, `crates/game_framework`, runtime events.
- Blockers: geometry query cost and voice management.
- Acceptance: audible occlusion and AI hearing debug proof.
- Mini-tooling slice:
  - Audio rays/hearing monitor minimum.
  - Audio propagation throughput benchmark.
  - Hearing query batch profiler.
- Evidence commands:
  - `cargo test --test gameplay_and_ai -- --nocapture`
  - `cargo test --test engine_contracts -- --nocapture`
- Docs: audio program + event model + concurrency model.
- Demo: same shot sounds/reacts differently by obstruction path.

## Phase F — SDK workstation
- Goals: full authoring/debug/profiling suite for flagship systems.
- Likely touched: `crates/engine_tools`, `crates/sdk_app`, `apps/engene_sdk`.
- Blockers: runtime telemetry and tooling API stability.
- Acceptance: all system overlays/editors present and usable.
- Mini-tooling slice:
  - Cross-subsystem performance dashboard.
  - Frame-phase timing waterfall.
  - Memory + sync contention dashboard.
- Evidence commands:
  - `cargo test --test sdk_editor_gui -- --nocapture`
  - `cargo test --test determinism_and_sdk -- --nocapture`
- Docs: SDK program + content pipeline + budget contracts.
- Demo: operator can configure and inspect showcase from SDK only.

## Phase G — Showcase scene / Game 1.0 Tech Demo
- Goals: integrate content + systems into one executable validated tech-demo scene.
- Likely touched: `game/scenarios`, `game/content`, `crates/game_framework`, `apps/engene_game`.
- Blockers: content production and balancing throughput.
- Acceptance: all flagship bullets demoable in single session under truth rules.
- Mini-tooling slice:
  - Worst-case showcase trigger pack.
  - Spike recovery validation scenario.
- Evidence commands:
  - `cargo test --test vertical_slice -- --nocapture`
  - `cargo test --test production_candidate -- --nocapture`
- Docs: game tech-demo plan + showcase truth rules.
- Demo: industrial district breach scenario fully runnable.

## Phase H — Hardening / perf / release
- Goals: budget lock, stability, documentation completeness, release gate.
- Likely touched: perf configs, tests/benches, release scripts, docs.
- Blockers: long-tail regressions and content edge cases.
- Acceptance: release gates pass; no P0 blockers.
- Mini-tooling slice:
  - Multithread determinism stress suite.
  - Allocator steady-state proof.
  - Cache-sensitive hot-path review sign-off.
- Evidence commands:
  - `cargo test --no-run -q`
  - `cargo bench --profile dev --bench engine_benchmarks`
  - `cargo bench --profile dev --bench hot_paths -- --sample-size 10`
- Docs: performance strategy + CPU performance law + memory budgets + budget contracts + risk register + release gates.
- Demo: 20-minute showcase run with captured metrics package.
