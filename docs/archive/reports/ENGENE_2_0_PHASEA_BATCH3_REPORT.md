# ENGENE 2.0 Phase A — Batch 3 Execution Report

## Concise Batch 3 plan
1. Execute a broader leaf-first extraction wave across `engine_core`, `engine_content`, and `engine_ecs`.
2. Move coherent contract/helper module groups (not isolated files) while avoiding heavy subsystem logic.
3. Keep root `engene` active with temporary compatibility shims for all moved module paths.
4. Refresh dependency/API migration snapshots and validate workspace + law gates.

## Exact moved-module list
### Moved into `engine_core`
- `src/core/build_manifest.rs` -> `crates/engine_core/src/build_manifest.rs`
- `src/core/runtime_manifest.rs` -> `crates/engine_core/src/runtime_manifest.rs`
- `src/core/determinism_policy.rs` -> `crates/engine_core/src/determinism_policy.rs`
- `src/core/data_policy.rs` -> `crates/engine_core/src/data_policy.rs`

### Moved into `engine_content`
- `src/content/pipeline.rs` -> `crates/engine_content/src/pipeline.rs`
- `src/content/import/asset_pipeline.rs` -> `crates/engine_content/src/import/asset_pipeline.rs`
- `src/content/cooking/cook_pipeline.rs` -> `crates/engine_content/src/cooking/cook_pipeline.rs`
- `src/content/prefabs/prefab.rs` -> `crates/engine_content/src/prefabs/prefab.rs`
- `src/content/prefabs/prefab_registry.rs` -> `crates/engine_content/src/prefabs/prefab_registry.rs`
- `src/content/validation/content_validator.rs` -> `crates/engine_content/src/validation/content_validator.rs`

### Moved into `engine_ecs`
- `src/core/sparse_set.rs` -> `crates/engine_ecs/src/sparse_set.rs`
- `src/core/access/mod.rs` -> `crates/engine_ecs/src/access/mod.rs`
- `src/core/access/ecs_view.rs` -> `crates/engine_ecs/src/access/ecs_view.rs`
- `src/core/access/resource_view.rs` -> `crates/engine_ecs/src/access/resource_view.rs`
- `src/core/access/queries.rs` -> `crates/engine_ecs/src/access/queries.rs`

## Exact compatibility shims added
- `src/core/mod.rs` now re-exports:
  - `engine_core::{build_manifest, runtime_manifest, determinism_policy, data_policy}`
  - `engine_ecs::{sparse_set, access}`
- `src/core/access/mod.rs` shim:
  - `pub use engine_ecs::access;`
- `src/content/pipeline.rs` shim:
  - `pub use engine_content::pipeline::*;`
- `src/content/import/mod.rs` shims:
  - `pub use engine_content::import::{asset_pipeline, hashes};`
- `src/content/import/asset_pipeline.rs` shim:
  - `pub use engine_content::import::asset_pipeline::*;`
- `src/content/cooking/mod.rs` shims:
  - `pub use engine_content::cooking::{cook_pipeline, dependency_graph};`
- `src/content/cooking/cook_pipeline.rs` shim:
  - `pub use engine_content::cooking::cook_pipeline::*;`
- `src/content/prefabs/mod.rs` shims:
  - `pub use engine_content::prefabs::{prefab, prefab_registry};`
- `src/content/prefabs/prefab.rs` shim:
  - `pub use engine_content::prefabs::prefab::*;`
- `src/content/prefabs/prefab_registry.rs` shim:
  - `pub use engine_content::prefabs::prefab_registry::*;`
- `src/content/validation/mod.rs` shim:
  - `pub use engine_content::validation::content_validator;`
- `src/content/validation/content_validator.rs` shim:
  - `pub use engine_content::validation::content_validator::*;`

## Cargo/dependency changes
- Root crate dependencies:
  - added `engine_ecs = { path = "crates/engine_ecs" }`
- `crates/engine_core/Cargo.toml`:
  - added `serde` and `ron` dependencies (required by moved build/runtime manifest contracts)
  - added empty feature declarations matching build-manifest feature probes (`physics`, `render`, `ai`, `audio`, `debug_ui`, `headless`, `sdk_tools`, `low_spec`, `networking`, `body_sim`, `audio_playback`) to keep check-cfg clean
- `crates/engine_content/Cargo.toml`:
  - added `ron` dependency (used by moved prefab registry)

## Law-compliance note
- No gameplay/system redesign.
- No heavy world/render/audio/AI migration.
- No new feature delivery; this batch is ownership extraction + compatibility bridging.
- Root package remains active and canonical entrypoints unchanged.
- Dependency direction and ECS direct-access gates remain enforced/passing.

## Updated dependency/API report note
- Updated `docs/generated/PHASE_A_DEPENDENCY_GRAPH.md` and `docs/generated/PHASE_A_API_SURFACE_REPORT.md` to Batch 3 state, reflecting expanded real ownership in `engine_core`, `engine_content`, and `engine_ecs`.

## Batch 4 high-value candidates
1. `engine_ecs`: contract-side extraction of stable entity identity/query contracts that are still runtime-neutral (e.g. selected `query` contract subsets after decoupling from gameplay component types).
2. `engine_core`: additional runtime-neutral audit/registry contracts (`integration_matrix`, selected perf contract DTOs) where fan-in is broad but behavior risk is low.
3. `engine_content`: remaining safe content orchestration helpers with minimal world coupling, plus stricter separation between content DTO contracts and monolith wiring.
