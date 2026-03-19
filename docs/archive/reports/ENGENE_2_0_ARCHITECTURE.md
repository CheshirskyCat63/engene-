# ENGENE 2.0 Architecture

## Target repository structure

```text
repo/
├── Cargo.toml
├── crates/
│   ├── engine_core/
│   ├── engine_ecs/
│   ├── engine_world/
│   ├── engine_runtime/
│   ├── engine_render/
│   ├── engine_physics/
│   ├── engine_audio/
│   ├── engine_content/
│   ├── engine_tools/
│   ├── sdk_app/
│   └── game_framework/
├── game/
│   ├── data/
│   ├── content/
│   ├── prefabs/
│   ├── materials/
│   ├── audio/
│   ├── weather/
│   └── scenarios/
├── apps/
│   ├── engene_game/
│   ├── engene_sdk/
│   └── engene_headless/
└── docs/
```

## Current-to-target mapping

### Current major modules and ownership
- `src/core`, `src/memory`, `src/input`, `src/engine.rs` -> **engine-owned** (`engine_core`, `engine_runtime`).
- `src/world`, `src/simulation`, `src/navigation` -> **engine-owned** (`engine_world`, `engine_runtime`).
- `src/physics`, `src/body` -> **engine-owned** (`engine_physics`).
- `src/graphics`, `src/animation` -> **engine-owned** (`engine_render`, partial `game_framework` hooks).
- `src/audio` -> **engine-owned** (`engine_audio`).
- `src/content` -> **mixed / needs split** (`engine_content` runtime schema + `sdk_app` authoring workflows).
- `src/tools`, `src/sdk.rs` -> **sdk-owned** (`engine_tools`, `sdk_app`).
- `src/game` -> **game-owned** (`game_framework` + `game/` authored content).
- `src/app`, `src/bin/*` -> **mixed / needs split** (`apps/*` thin launchers + runtime assembly in `engine_runtime`).

## Crate public API contract
- Every crate exposes a stable facade (`api` module or crate root re-exports).
- Internal wiring stays private (`pub(crate)` default).
- Apps and sibling crates can only call public API, never internal module paths.
- Hot-path API surfaces must prefer handles/IDs and batch slices over rich object graphs.

## Forbidden imports
- Engine crates importing `game_framework`.
- Engine crates importing `sdk_app`.
- `engine_render` importing gameplay logic directly.
- `engine_audio` importing gameplay logic directly.
- `apps/*` importing private modules from engine crates.

## Shared types registry
- Core primitives and IDs: `engine_core`.
- Canonical content/schema DTOs: `engine_content`.
- Runtime event contracts: `engine_runtime`.
- Duplicate cross-domain DTOs are prohibited.


## Product boundary guardrails
- `engine_*` crates define reusable runtime capability and canonical contracts.
- `sdk_app` + `engine_tools` define workstation/tooling capability, not gameplay truth.
- `game_framework` + `game/*` define tech-demo game rules/content, not engine ownership.
- Engine crates must not embed scenario-specific assumptions.
- SDK crates must not become mandatory for shipping runtime correctness.
- Game crates must not redefine canonical engine contracts.

## Performance ownership guardrails
- `engine_ecs` owns packed storage/query primitives, not gameplay logic.
- `engine_runtime` owns phase scheduling, batching, job dispatch, and budget enforcement.
- `engine_world` owns authoritative world state and packed environment data models.
- `engine_physics` owns compute-heavy local solvers and batch-friendly APIs.
- `engine_audio` owns propagation/hearing compute models and voice budget interfaces.
- `engine_tools` may inspect performance state but must not inject heavy dependencies into hot paths.
- `sdk_app` consumes telemetry snapshots, never live-pokes hot-loop internals.


## Sky/Weather cross-domain placement
- Weather truth state and simulation fields live in `engine_world`.
- Weather frame snapshot contract lives in `engine_runtime` APIs.
- Sky/weather presentation lives in `engine_render`.
- Weather authoring data lives in `engine_content`.
- SDK weather diagnostics live in `engine_tools`/`sdk_app`.
- SDK/tools are read-only over shipping runtime weather truth except explicit debug/authoring command paths.
- `game_framework` may emit triggers only; it does not own weather truth.
- `game_framework` must not set render sky state directly.

## Runtime ownership guardrails
`engine_runtime` owns:
- Scheduling/orchestration.
- Simulation-level transition coordination.
- Cross-domain event routing contract.

`engine_runtime` may orchestrate but must not define domain logic for:
- Physics material response rules.
- Audio propagation algorithms.
- World/environment simulation formulas.

Domain truth remains owned by `engine_physics`, `engine_audio`, `engine_world`, `engine_content` respectively.

## engine_tools guardrails
`engine_tools` must be partitioned into:
- Validation tooling.
- Inspection tooling.
- Profiling tooling.
- Authoring helpers.
- Replay/evidence tooling.

`engine_tools` must not become a generic runtime logic sink.

## Cycle prevention rules
- Crate graph checked in CI for cycles.
- Any new crate edge requires dependency-law review.
- Feature flags cannot be used to hide reverse dependencies.

## Migration order
1. Create workspace crates and move leaf modules with minimal API changes.
2. Extract `engine_core` + `engine_ecs` contracts, then rewire dependents.
3. Move world/simulation into `engine_world`/`engine_runtime`.
4. Split render/physics/audio/content.
5. Move tools/sdk runtime to `engine_tools` + `sdk_app`.
6. Move game logic to `game_framework`; keep authored assets under `game/`.
7. Convert `src/bin` to `apps/*` crates calling assembly APIs.

## Compile-risk hotspots
- Feature flag coupling in single-package `Cargo.toml` to be replaced by crate features.
- `src/app/runtime_assembly.rs` and `src/game/runtime_assembly.rs` likely high fan-in wiring points.
- Physics/world/render cross-calls may expose hidden cycles during split.
- Tooling code in `src/tools` that touches game/runtime internals may violate final dependency direction.
- Serialization types currently shared implicitly through module paths need explicit crate-level API contracts.

## Blockers
- Stable public API boundaries for ECS/world handles.
- Common data model for material layers used by physics, audio, and destruction.
- Unified event bus contract to avoid duplicate per-crate event types.
