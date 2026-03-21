# CURRENT_BRANCH_STATE

## Status label

**Post-root removal. Pure workspace established. Canonical architecture achieved.**

## Current package truth

- Workspace is pure workspace root without package.
- Root package `engene` has been removed.
- Workspace contains only engine crates, role crates, and `apps/*`.

## Current entrypoint truth

| Runtime role | Binary entrypoint | First called crate | Current runner body location | Canonical owner | Status |
|---|---|---|---|---|
| Game | `cargo run -p engene_game` | `game_framework` | `crates/game_framework/src/lib.rs` | `game_framework` | deprecated stub |
| SDK | `cargo run -p engene_sdk` | `sdk_app` | `crates/sdk_app/src/lib.rs` | `sdk_app` | app entrypoint |
| Headless | `cargo run -p engene_run -- --ticks 1200` | `game_framework` | `crates/game_framework/src/lib.rs` | `game_framework` | app entrypoint |

## Current architecture truth

- Root is now pure workspace root (no package).
- SDK and Headless apps are canonical launch truth.
- Game entrypoint is deprecated stub.
- Legacy archived in archive/tests_legacy_pre_2024/ (separate package with historical dependencies).
- Role crates have canonical ownership structure.
- CI reflects current architecture.
- Workspace is valid and functional.
- Engine core compiles successfully.
- Role crates have canonical ownership structure.
- Legacy properly isolated.
- SDK and Headless apps are canonical entrypoints.

## Explicit current truths

- Root package removal is complete.
- Workspace is valid and functional.
- Engine core compiles successfully.
- Role crates have canonical ownership structure.
- Legacy properly isolated.
- SDK and Headless apps are canonical entrypoints.
- CI reflects current architecture.

## Rule

This file is a snapshot of current truth, not a roadmap.
