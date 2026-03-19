# ENGENE 2.0 Dependency Ownership Map

Date: 2026-03-18
Scope: Direct workspace dependencies (not transitive lockfile graph).

## Direct external dependencies (root package `Cargo.toml`)
| Dependency | Where used (representative) | Bucket | Why needed | Status |
|---|---|---|---|---|
| `rand` | AI/world stochastic helpers (`src/game/*`, `src/world/*`) | runtime/sim | deterministic/randomized simulation decisions | KEEP |
| `wgpu` | renderer backend (`src/graphics/renderer.rs`) | graphics/projection | GPU rendering backend | KEEP |
| `winit` | app event loops (`src/app/game_runner/app_loop.rs`, `src/app/sdk_runner.rs`) | graphics/projection | window/input/event loop | KEEP |
| `glam` | math across physics/graphics/world | kernel/core + runtime/sim | vector/matrix math | KEEP |
| `bytemuck` | GPU buffer casts (`src/graphics/*`) | graphics/projection | POD-safe casts for GPU upload | KEEP |
| `pollster` | startup async blocking init (`src/graphics/renderer.rs`) | graphics/projection | sync bootstrap of async GPU setup | KEEP FOR NOW |
| `rapier3d` | physics internals (`src/physics/rapier_world.rs`) | runtime/sim | rigid-body/collision solver integration | KEEP |
| `gltf` | asset/model loading (`src/graphics/model_loader.rs`) | graphics/projection | GLTF content ingestion | KEEP FOR NOW |
| `image` | texture/image IO (`src/graphics/*`, content) | graphics/projection | texture decoding and preprocessing | KEEP |
| `rayon` | parallel jobs (`src/core/jobs`, heavy systems) | kernel/core | CPU parallelism | KEEP |
| `parking_lot` | lock primitives (`src/core/*`, tools/runtime`) | kernel/core | fast synchronization | KEEP FOR NOW |
| `pathfinding` | nav/path systems (`src/navigation/*`) | runtime/sim | path search utilities | KEEP FOR NOW |
| `slotmap` | handles/IDs in subsystems | runtime/sim | stable handle maps | KEEP FOR NOW |
| `tracing` | instrumentation/logging (`src/*`) | kernel/core | structured tracing | KEEP |
| `tracing-subscriber` | startup logging config (`src/app/*`) | tools/sdk + runtime | log subscriber setup | KEEP |
| `serde` | data schema serialization (`src/world/*`, `src/core/*`) | kernel/core | serialization traits | KEEP |
| `serde_json` | tools/report output (`src/tools/*`) | tools/sdk | JSON output/reporting | KEEP FOR NOW |
| `bincode` | binary persistence (`src/world/chunk_schema.rs`) | runtime/sim | compact binary storage | KEEP |
| `ron` | authored config/content (`game/data/*.ron`, loaders) | game/product | human-readable authored config format | KEEP |
| `puffin` | perf profiling hooks (`src/core/perf/*`) | tools/sdk | profiler integration | KEEP FOR NOW |
| `bitflags` | flags bitmasks (`src/*`) | kernel/core | compact flag types | KEEP |
| `egui` | editor/tool UI (`src/tools/*`) | tools/sdk | immediate-mode UI | KEEP |
| `egui-wgpu` | egui render bridge | tools/sdk | egui + wgpu integration | KEEP |
| `egui-winit` | egui input/window bridge | tools/sdk | egui + winit integration | KEEP |
| `rodio` (optional) | audio playback path (`src/audio/playback.rs`) | audio | runtime audio output | KEEP FOR NOW |
| `walkdir` | content/doc scans (`src/tools/doc_generator.rs`, content) | tools/sdk + content | recursive file traversal | KEEP |

## Dev-only dependencies
| Dependency | Where used | Bucket | Status |
|---|---|---|---|
| `criterion` | benches | tests/dev-only | KEEP |
| `glob` | test file matching helpers | tests/dev-only | KEEP FOR NOW |
| `tempfile` | persistence/content tests | tests/dev-only | KEEP |

## Workspace crate-level external deps
- `crates/engine_core`: `serde`, `ron`, `puffin`.
- `crates/engine_content`: `serde`, `walkdir`, `ron` (+ `tempfile` dev).
- `crates/engine_ecs`: `serde`.
- App wrappers (`apps/*`) depend only on root `engene` crate.

## Forbidden-in-kernel set (must stay out of pure kernel surface)
- `wgpu`, `winit`, `egui`, `egui-wgpu`, `egui-winit`, `rodio`, `gltf`, `image`.

## Tools-only preferred set
- `egui`, `egui-wgpu`, `egui-winit`, `puffin`, `serde_json`, `walkdir` (when used for tooling/documentation scans).

## Game-only preferred set
- `ron` authored game data loading paths, gameplay content data pipelines.

## Tests/dev-only set
- `criterion`, `glob`, `tempfile`.

## Feature leakage notes
- Current root crate still links graphics/audio/tooling dependencies in one package surface; this is acceptable for monolithic phase but a split target.
- `rodio` is optional, but wider dependency consolidation is still pending package-level role split.
