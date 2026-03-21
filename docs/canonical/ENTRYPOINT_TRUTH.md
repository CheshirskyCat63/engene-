# ENTRYPOINT_TRUTH

## Rule

There must be one canonical description of how ENGENE starts in the current branch.

## Current canonical entrypoints

| Runtime role | Binary entrypoint | First called crate | Current runner body location | Canonical owner | Status |
|---|---|---|---|---|---|
| Game | `cargo run -p app_engene_game` | `game_framework` | `crates/game_framework/src/lib.rs` | `game_framework` | transitional role crate |
| SDK | `cargo run -p app_engene_sdk` | `sdk_app` | `crates/sdk_app/src/lib.rs` | `sdk_app` | transitional role crate |
| Headless | `cargo run -p app_engene_headless -- --ticks 1200` | `game_framework` | `crates/game_framework/src/lib.rs` | `game_framework` | transitional role crate |
| Bootstrap | `cargo run -p engene_bootstrap` | `engene_bootstrap` | `apps/engene_bootstrap/src/main.rs` | `engene_bootstrap` | app shell |

## Not current truth

The following are not canonical launch truths in this branch:
- package-level execution through `apps/*` (apps are transitional shells)
- any undocumented `test` / `sandbox` / `demo` bin
- root bins as long-term operator truth

## Ownership statement

Root bins are the current operator truth.
Root is still a thin migration shell.
Role crates (`game_framework`, `sdk_app`) are transitional owners.
Ownership handoff is incomplete.

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
