# ENGENE 2.0 Phase A — Batch 6 Execution Report

## Concise Batch 6 plan
1. Carve a runtime-neutral query contract subset from mixed `core/query` into `engine_ecs`.
2. Keep gameplay/world-coupled query filters and bundles in monolith for safety.
3. Simplify `content` compatibility bridges by centralizing re-exports in `src/content/mod.rs`.
4. Refresh Phase A dependency/API snapshots and closure-readiness assessment.

## Exact moved-module list
### Moved into `engine_ecs`
- **New extracted module:** `crates/engine_ecs/src/query_contract.rs`
  - runtime-neutral matcher contract (`EntityMatcher`)
  - generic `AndMatcher`
  - generic filtered iterator (`FilteredEntityIter`)

### `engine_content` ownership / bridge cleanup
- No new content runtime module body was moved in Batch 6.
- Compatibility bridge ownership was simplified by removing redundant shim files under `src/content/*` and centralizing bridge exports in `src/content/mod.rs`.

## Exact compatibility shims added / removed / simplified
### Added / expanded
- `src/core/query.rs` now bridges onto `engine_ecs::query_contract` primitives:
  - `QueryFilter` now extends runtime-neutral `EntityMatcher<Ecs>`
  - `And` now aliases `engine_ecs::query_contract::AndMatcher`
  - `QueryIter` now aliases `engine_ecs::query_contract::FilteredEntityIter`

### Removed / simplified
- Removed redundant file-level content shim modules:
  - `src/content/pipeline.rs`
  - `src/content/import/mod.rs`
  - `src/content/import/asset_pipeline.rs`
  - `src/content/cooking/mod.rs`
  - `src/content/cooking/cook_pipeline.rs`
  - `src/content/prefabs/mod.rs`
  - `src/content/prefabs/prefab.rs`
  - `src/content/prefabs/prefab_registry.rs`
  - `src/content/validation/mod.rs`
  - `src/content/validation/content_validator.rs`
- `src/content/mod.rs` now acts as the single thin centralized bridge via direct `pub use engine_content::{...}` exports.

## Exact Cargo/dependency/workspace changes
- `crates/engine_ecs/src/lib.rs`:
  - added export: `pub mod query_contract;`
  - added API facade export: `pub use crate::query_contract;`
- No new workspace members.
- No new third-party dependencies added in Batch 6.

## Law-compliance note
- No new features.
- No gameplay/system redesign.
- No deep world/runtime/render/audio/AI migration.
- Root package remains active and canonical entrypoints remain intact.
- Dependency direction and ECS direct-access laws remain clean and validated.

## Updated dependency/API report note
- Updated:
  - `docs/generated/PHASE_A_DEPENDENCY_GRAPH.md`
  - `docs/generated/PHASE_A_API_SURFACE_REPORT.md`
- Batch 6 API snapshot now includes `engine_ecs::query_contract` and records centralized content bridge simplification.

## Phase A closure-readiness note
### Must-move-before-Phase-A-close
1. Additional runtime-neutral carve-out from `core/query` (generic-only parts beyond this batch) so monolith no longer owns primary query contracts.
2. Remaining monolith-owned ECS contract surfaces that are still runtime-neutral and reusable (after coupling split) should be crate-owned.
3. Monolith should converge to orchestration/assembly + temporary compatibility, not primary owner of contract APIs.

### Safe-to-defer-to-later-phases
1. Gameplay-specific filters and component-bound query bundles in `core/query` (coupled to world/gameplay components).
2. Deep runtime/world/render/audio/AI ownership migrations.
3. Non-contract performance/content polish refactors.

## Remaining highest-value candidates for Batch 7 or Phase A closure
1. Continue precision split of `core/query`: move additional generic filter composition / query helper abstractions that are domain-neutral.
2. Evaluate extraction of low-risk ECS-facing context/policy contracts only after removing world/gameplay coupling.
3. Keep shrinking monolith bridge surface by centralizing remaining compatibility layers and removing redundant shim files when safe.
