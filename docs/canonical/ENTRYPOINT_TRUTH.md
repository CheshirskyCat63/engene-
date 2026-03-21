# ENTRYPOINT_TRUTH

## Rule

There must be one canonical description of how ENGENE starts in the current branch.

## Current canonical entrypoints

| Runtime role | Binary entrypoint | First called crate | Current runner body location | Canonical owner | Status |
|---|---|---|---|---|---|
| Game | `cargo run -p engene_game` | `game_framework` | `crates/game_framework/src/lib.rs` | `game_framework` | app entrypoint |
| SDK | `cargo run -p engene_sdk` | `sdk_app` | `crates/sdk_app/src/lib.rs` | `sdk_app` | app entrypoint |
| Headless | `cargo run -p engene_run -- --ticks 1200` | `game_framework` | `crates/game_framework/src/lib.rs` | `game_framework` | app entrypoint |

## Current truth

Package-level execution through `apps/*` is canonical launch truth.
Apps are real entrypoints, not transitional shells.
Root bins are not current operator truth.
Ownership handoff is complete.

## Explicit note

Role crates still depend on root for:
- `engene::core::build_manifest::BuildManifest`
- `engene::core::crash_telemetry`
- `engene::runtime::bootstrap::*`
- `engene::world::heightmap::Heightmap`
- `engene::world::world::WorldGrid`
- `engene::app::game_runner::GameApp`
- `engene::app::spatial_dirty_journal::SpatialDirtyJournal`
- `engene::world::components::*`
- `engene::world::hierarchical_spatial::SpatialUpdatePath`

## Transition rule

When package-level runtime ownership becomes real, this file changes in one step.
Until then, no document may advertise future package-level commands as current truth.
