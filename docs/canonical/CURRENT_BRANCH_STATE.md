# CURRENT_BRANCH_STATE

## Status label

**Post-root removal. Pure workspace established. Canonical architecture achieved.**

## Current package truth

- Workspace is pure workspace root without package.
- Root package `engene` has been removed.
- Workspace contains only engine crates, role crates, and `apps/*`.
- Root feature model migrated to workspace-level dependencies.
- Role crates have canonical ownership structure.

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
- Role crates have clear ownership boundaries.
- Legacy isolated in tests_legacy/ and legacy/quarantine/.

## Explicit current truths

- Root package removal is complete.
- Workspace is valid and functional.
- Engine core compiles successfully.
- Role crates have canonical ownership structure.
- Legacy properly isolated.
- Apps are canonical entrypoints.

## Rule

This file is a snapshot of current truth, not a roadmap.
