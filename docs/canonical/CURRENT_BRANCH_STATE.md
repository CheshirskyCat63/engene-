# CURRENT_BRANCH_STATE

## Status label

**Post-root removal. Pure workspace established. Role drift resolved.**

## Current package truth

- Workspace is pure workspace root without package.
- Root package `engene` has been removed.
- Workspace contains only engine crates, role crates, and `apps/*`.

## Current entrypoint truth

| Runtime role | Binary entrypoint | First called crate | Current runner body location | Canonical owner | Status |
|---|---|---|---|---|---|
| Game | `cargo run -p engene_game` | `runtime_headless` | `crates/runtime_headless/src/lib.rs` | `runtime_headless` | app entrypoint |
| SDK | `cargo run -p engene_sdk` | `sdk_app` | `crates/sdk_app/src/lib.rs` | `sdk_app` | app entrypoint |
| Headless | `cargo run -p engene_run -- --ticks 1200` | `runtime_headless` | `crates/runtime_headless/src/lib.rs` | `runtime_headless` | app entrypoint |

## Current architecture truth

- Root is pure workspace root (no package).
- Headless runtime ownership is `runtime_headless` (not `game_framework`).
- `game_framework` is reserved for future game-specific launch paths.
- Game, SDK, and Headless apps are canonical entrypoints.
- Legacy archived in archive/tests_legacy_pre_2024/.
- CI has narrow_check gate before broad jobs.

## Rule

This file is a snapshot of current truth, not a roadmap.
