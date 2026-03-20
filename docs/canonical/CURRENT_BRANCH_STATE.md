# CURRENT_BRANCH_STATE

## Status label

**Blocked transition. Structural split exists, but ownership handoff is incomplete. Root remains the active migration shell.**

## Current package truth

- Workspace is declared with root package, engine crates, role crates, and `apps/*`.
- Root package `engene` is still active and hosts canonical bins.
- Root feature model now has explicit target taxonomy plus compatibility aliases.
- Role crates exist, but ownership is not fully handed off yet.

## Current entrypoint truth

| Runtime role | Binary entrypoint | First called crate | Current runner body location | Canonical owner | Status |
|---|---|---|---|---|---|
| Game | `cargo run -p app_engene_game` | `game_framework` | `crates/game_framework/src/lib.rs` | `game_framework` | transitional role crate |
| SDK | `cargo run -p app_engene_sdk` | `sdk_app` | `crates/sdk_app/src/lib.rs` | `sdk_app` | transitional role crate |
| Headless | `cargo run -p app_engene_headless -- --ticks 1200` | `game_framework` | `crates/game_framework/src/lib.rs` | `game_framework` | transitional role crate |
| Bootstrap | `cargo run -p engene_bootstrap` | `engene_bootstrap` | `apps/engene_bootstrap/src/main.rs` | `engene_bootstrap` | app shell |

## Current architecture truth

- Root is still the active migration shell.
- Root bins are still the canonical operator surface.
- `apps/*` are not canonical launch truth.
- Role crates are not yet full runtime owners.
- Role crates still depend on root for:
  - `engene::core::build_manifest::BuildManifest`
  - `engene::core::crash_telemetry`
  - `engene::runtime::bootstrap::*`
  - `engene::world::heightmap::Heightmap`
  - `engene::world::world::WorldGrid`
  - `engene::app::game_runner::GameApp`
  - `engene::app::spatial_dirty_journal::SpatialDirtyJournal`
  - `engene::world::components::*`
  - `engene::world::hierarchical_spatial::SpatialUpdatePath`
- `engine_ecs` and `engine_world` are still incomplete ownership crates.
- Any temporarily disabled module must be recorded in `MIGRATION_LEDGER.md`.

## Explicit not-yet-true statements

- Migration is not finish-ready.
- Root is not yet removable.
- Package-level entrypoints are not current operator truth.
- Placeholder crates are not proof of completed ownership split.
- Compile-green shell state is not the same as domain recovery.

## Rule

This file is a snapshot of current truth, not a roadmap.
