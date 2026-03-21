# CURRENT_BRANCH_STATE

## Status label

**Post-root removal. Pure workspace established. Transitional cleanup in progress.**

## Current package truth

- Workspace is pure workspace root without package.
- Root package `engene` has been removed.
- Workspace contains only engine crates, role crates, and `apps/*`.
- Root feature model migrated to workspace-level dependencies.
- Role crates exist but still have transitional imports from removed root.

## Current entrypoint truth

| Runtime role | Binary entrypoint | First called crate | Current runner body location | Canonical owner | Status |
|---|---|---|---|---|---|
| Game | `cargo run -p engene_game` | `game_framework` | `crates/game_framework/src/lib.rs` | `game_framework` | transitional role crate |
| SDK | `cargo run -p engene_sdk` | `sdk_app` | `crates/sdk_app/src/lib.rs` | `sdk_app` | transitional role crate |
| Headless | `cargo run -p engene_headless -- --ticks 1200` | `game_framework` | `crates/game_framework/src/lib.rs` | `game_framework` | transitional role crate |
| Bootstrap | `cargo run -p engene_bootstrap` | `engene_bootstrap` | `apps/engene_bootstrap/src/main.rs` | `engene_bootstrap` | app shell |

## Current architecture truth

- Root is now pure workspace root (no package).
- Apps are canonical launch truth.
- Role crates are runtime owners but still have transitional dependencies.
- Role crates still depend on removed root for:
  - `engene::runtime::bootstrap::*` (needs migration to engine_runtime)
  - `engene::world::*` (needs migration to engine_world)
  - `engene::graphics::*` (needs migration to engine_render)
  - `engene::core::*` (needs migration to engine_core)
  - `engene::app::*` (needs migration to engine_runtime)
- `engine_ecs` and `engine_world` are ownership crates.
- Transitional imports need systematic replacement.

## Explicit current truths

- Root package removal is complete.
- Workspace is valid and functional.
- Engine core compiles successfully.
- Role crates need import migration.
- Apps are canonical entrypoints.

## Rule

This file is a snapshot of current truth, not a roadmap.
