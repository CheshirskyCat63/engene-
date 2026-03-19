# ENGENE 2.0 Phase A — Batch 5 Execution Report

## Concise Batch 5 plan
1. Expand `engine_ecs` with the next runtime-neutral ECS-facing contract helpers (identity/query-adjacent command+validation layer).
2. Expand `engine_content` with additional safe content contract/helper ownership.
3. Keep compatibility bridges thin and centralized (prefer `src/core/mod.rs` / existing module shims over extra bridge files).
4. Update reports and re-run dependency/ECS/build checks.

## Exact moved-module list
### Moved into `engine_ecs`
- `src/core/commands.rs` -> `crates/engine_ecs/src/commands.rs`
- `src/core/parallel_validation.rs` -> `crates/engine_ecs/src/parallel_validation.rs`

### Moved into `engine_content`
- `src/tools/content_hash.rs` -> `crates/engine_content/src/content_hash.rs`

### `engine_core` note
- No additional Batch 5 module moves to `engine_core` (kept disciplined per priority/risk).

## Exact compatibility shims added / removed / simplified
- Removed monolith-owned `pub mod commands;` and `pub mod parallel_validation;` from `src/core/mod.rs`.
- Added direct compatibility re-exports in `src/core/mod.rs`:
  - `pub use engine_ecs::commands;`
  - `pub use engine_ecs::parallel_validation;`
- Simplified tools bridge:
  - `src/tools/content_hash.rs` now a thin bridge `pub use engine_content::content_hash::*;`
- No new extra bridge modules were introduced beyond these minimal adapters.

## Exact Cargo/dependency/workspace changes
- `crates/engine_ecs/src/lib.rs` now exports:
  - `commands`
  - `parallel_validation`
- `crates/engine_content/src/lib.rs` now exports:
  - `content_hash`
- `crates/engine_content/Cargo.toml`:
  - added `[dev-dependencies] tempfile = "3"` for moved content-hash tests.
- `Cargo.lock` updated due crate-level dependency graph updates.

## Law-compliance note
- No new features.
- No gameplay/system redesign.
- No deep world/runtime/render/audio/AI migration.
- Root `engene` package remains active; canonical entrypoints unchanged.
- Dependency direction and ECS direct-access laws remain clean and validated.

## Updated dependency/API report note
- Updated Batch 5 generated snapshots:
  - `docs/generated/PHASE_A_DEPENDENCY_GRAPH.md`
  - `docs/generated/PHASE_A_API_SURFACE_REPORT.md`
- Reports now reflect expanded `engine_ecs` contract ownership and additional `engine_content` helper ownership.

## Phase A status after Batch 5
- **Phase A is still open.**
- Reason: monolith still owns large mixed ECS query logic (`core/query`) and several cross-cutting runtime modules where ownership boundaries are not yet fully separated into stable crate contracts.

## Remaining highest-value candidates for Batch 6 or Phase A closure
1. `engine_ecs`: carve runtime-neutral subset from `core/query` (generic query iterator/filter contracts), keeping gameplay-coupled query bundles in monolith until decoupled.
2. `engine_ecs`: evaluate safe extraction of additional ECS-facing policy/context contracts only if they can avoid coupling to world/gameplay component details.
3. `engine_content`: continue pulling pure content-side helper utilities still outside crate ownership (if any remain), while avoiding world-coupled runtime behavior.
4. Phase A closure criteria: dependency law clean, ECS direct-access clean, and monolith reduced to intentional compatibility/assembly surfaces rather than owning primary contracts.
