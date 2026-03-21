# ENGENE 2.0 Phase A — Batch 4 Execution Report

## Concise Batch 4 plan
1. Prioritize `engine_ecs` by extracting the next runtime-neutral ECS identity/descriptor contract wave.
2. Continue `engine_core` ownership reduction by moving shared runtime-neutral registry/audit/policy helpers.
3. Keep `engine_content` separation clean (no runtime-heavy migrations in this batch).
4. Preserve compatibility through targeted temporary bridges and keep law/build gates green.

## Exact moved-module list
### Moved into `engine_ecs`
- `src/core/persistent_id.rs` -> `crates/engine_ecs/src/persistent_id.rs`
- `src/core/system_descriptor.rs` -> `crates/engine_ecs/src/system_descriptor.rs`

### Moved into `engine_core`
- `src/core/registry.rs` -> `crates/engine_core/src/registry.rs`
- `src/core/metrics_registry.rs` -> `crates/engine_core/src/metrics_registry.rs`
- `src/core/ownership_map.rs` -> `crates/engine_core/src/ownership_map.rs`
- `src/core/integration_matrix.rs` -> `crates/engine_core/src/integration_matrix.rs`
- `src/core/profiler.rs` -> `crates/engine_core/src/profiler.rs`

### `engine_content` batch note
- No additional runtime-heavy content module migrations were performed in Batch 4; current extracted contract/DTO ownership from Batch 3 was retained.

## Exact compatibility shims added or removed
- **Removed monolith-owned modules** in `src/core/mod.rs` for moved files:
  - `integration_matrix`, `metrics_registry`, `ownership_map`, `profiler`, `registry`, `persistent_id`, `system_descriptor`
- **Added temporary compatibility re-exports** in `src/core/mod.rs`:
  - `pub use engine_core::{integration_matrix, metrics_registry, ownership_map, profiler, registry};`
  - `pub use engine_ecs::{persistent_id, system_descriptor};`
- No extra shim files were added for these moved modules; compatibility is handled directly through `src/core/mod.rs` re-exports to avoid needless layering.

## Exact Cargo/dependency/workspace changes
- `crates/engine_ecs/Cargo.toml`:
  - added `serde = { version = "1", features = ["derive"] }` (required by moved `persistent_id` serde derives)
- `crates/engine_ecs/src/lib.rs`:
  - expanded exported contract surface with `persistent_id` and `system_descriptor`
- `crates/engine_core/src/lib.rs`:
  - expanded exported contract surface with `registry`, `metrics_registry`, `ownership_map`, `integration_matrix`, `profiler`
- No workspace member additions/removals in this batch.

## Law-compliance note
- No new features.
- No gameplay/system redesign.
- No deep world/render/audio/AI migration.
- No legacy junk migration.
- Root package remains active and canonical entrypoints unchanged.
- Dependency and ECS direct-access laws remain enforced and validated.

## Updated dependency/API report note
- `docs/generated/PHASE_A_DEPENDENCY_GRAPH.md` and `docs/generated/PHASE_A_API_SURFACE_REPORT.md` updated to reflect expanded Batch 4 ownership in `engine_ecs` and `engine_core`.

## Remaining highest-value candidates for Batch 5
1. `engine_ecs`: evaluate extraction of runtime-neutral query contracts after splitting gameplay-coupled filters/data bundles from `core/query`.
2. `engine_core`: consider migration of low-risk topology/report contracts with manageable coupling (`job_topology_report` after interface decoupling if needed).
3. `engine_content`: continue tightening by extracting any remaining pure content contract helpers while avoiding world-coupled runtime behavior.
