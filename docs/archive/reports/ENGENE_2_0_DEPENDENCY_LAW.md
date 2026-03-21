# ENGENE 2.0 Dependency Law

## Purpose
This document defines strict crate-level dependency direction to prevent a new monolith from reappearing after workspace split.

## Global laws
1. Dependency direction is one-way: `core -> ecs/world/runtime -> feature systems -> apps`.
2. Engine crates must never depend on `game_framework` or `sdk_app`.
3. `sdk_app` and `game_framework` may depend on engine crates, never inverse.
4. Binaries in `apps/*` can use only public crate APIs; no private module path reaches.
5. Any exception requires architecture review and law update.

## Crate dependency matrix

| crate | can depend on | must never depend on |
|---|---|---|
| `engine_core` | std + third-party libs only | any ENGENE higher-level crate |
| `engine_ecs` | `engine_core` | `engine_world`, `engine_runtime`, `game_framework`, `sdk_app` |
| `engine_content` | `engine_core` | `engine_render`, `engine_physics`, `game_framework`, `sdk_app` |
| `engine_world` | `engine_core`, `engine_ecs`, `engine_content` | `game_framework`, `sdk_app`, `engine_tools` |
| `engine_runtime` | `engine_core`, `engine_ecs`, `engine_content`, `engine_world` | `game_framework`, `sdk_app` |
| `engine_physics` | `engine_core`, `engine_ecs`, `engine_content`, `engine_world` | `game_framework`, `sdk_app` |
| `engine_render` | `engine_core`, `engine_ecs`, `engine_content`, `engine_world`, `engine_runtime` contracts | `game_framework`, `sdk_app` |
| `engine_audio` | `engine_core`, `engine_ecs`, `engine_content`, `engine_world`, `engine_runtime` contracts | `game_framework`, `sdk_app` |
| `engine_tools` | `engine_core`, `engine_ecs`, `engine_content`, `engine_world`, `engine_runtime`, `engine_render`, `engine_audio`, `engine_physics` | `game_framework` runtime-private modules |
| `game_framework` | all engine runtime-facing public crates | `sdk_app` |
| `sdk_app` | `engine_tools`, engine public crates, optional `game_framework` debug adapters | engine private modules |
| `apps/engene_*` | public APIs of engine/game/sdk crates only | direct `src/*` module internals |

## Public API contract
- Each crate must expose a single `pub mod api` (or equivalent stable facade).
- Internal modules stay `pub(crate)` by default.
- Cross-crate usage through re-exported stable types only.

## Shared type registry
Shared contracts are centralized in:
- `engine_core::types` for fundamental primitives.
- `engine_content::schema` for canonical content descriptors.
- `engine_runtime::events` for cross-system runtime signals.

Duplicate “nearly same” structs across crates are forbidden.

## Cycle prevention rules
- CI must run `cargo metadata`-based cycle check.
- New crate introduces a dependency RFC section: allowed edges, forbidden edges.
- Feature flags cannot hide reverse dependency violations.

## Phase A completion criteria additions
Phase A is complete only when all are true:
1. Workspace compiles across all member crates.
2. Legacy reverse imports are zero.
3. Every moved module has explicit owner crate.
4. No duplicate shared contract types across crates.
5. Crate-level dependency-direction gate passes.
6. All `apps/*` launch via public APIs only.
7. No bin/app crate reaches engine private modules directly.
