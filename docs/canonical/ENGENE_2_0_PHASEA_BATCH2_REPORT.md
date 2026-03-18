# ENGENE 2.0 Phase A — Batch 2 Execution Report

## Concise Phase A batch 2 plan
1. Extract additional low-risk, contract-level leaf modules to `engine_core` and `engine_content`.
2. Preserve root-package compatibility via temporary re-exports from existing monolith module paths.
3. Keep migration scope limited to API/contracts and content-schema helpers (no heavy subsystem logic).
4. Update dependency/API reports and validate workspace build/gates.

## Exact moved-module list
Moved into `engine_core`:
- `src/core/runtime_config.rs` -> `crates/engine_core/src/runtime_config.rs`

Moved into `engine_content`:
- `src/content/asset_budget.rs` -> `crates/engine_content/src/asset_budget.rs`
- `src/content/schema_governance.rs` -> `crates/engine_content/src/schema_governance.rs`
- `src/content/import/hashes.rs` -> `crates/engine_content/src/import/hashes.rs`
- `src/content/cooking/dependency_graph.rs` -> `crates/engine_content/src/cooking/dependency_graph.rs`

Compatibility bridges applied:
- `src/core/mod.rs` re-exports `engine_core::runtime_config`.
- `src/content/mod.rs` re-exports `engine_content::{asset_budget, schema_governance}`.
- `src/content/import/mod.rs` re-exports `engine_content::import::hashes`.
- `src/content/cooking/mod.rs` re-exports `engine_content::cooking::dependency_graph`.

## Law-compliance note
- No new features introduced.
- No ECS/world/render/audio/gameplay redesign performed.
- Migration is leaf-first and contract-oriented (runtime/profile config and content schema/budget/hash/dependency contracts only).
- Root package remains active and existing call sites preserved through temporary compatibility re-exports.
- Dependency-direction/product-boundary posture preserved.

## Validation summary
- Workspace and dependency-direction checks executed and passing for this batch.
