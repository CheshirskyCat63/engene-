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
| Game | `cargo run -p engene_game` | `engene_game` | `apps/engene_game/src/main.rs` | `engene_game` | app entrypoint |
| SDK | `cargo run -p engene_sdk` | `engene_sdk` | `apps/engene_sdk/src/main.rs` | `engene_sdk` | app entrypoint |
| Headless | `cargo run -p engene_run -- --ticks 1200` | `engene_run` | `apps/engene_run/src/main.rs` | `engene_run` | app entrypoint |

## Current architecture truth

- Root is now pure workspace root (no package).
- Apps are canonical launch truth.
- Legacy archived in archive/tests_legacy_pre_2024/ (separate package with historical dependencies).
- Role crates have canonical ownership structure.
- CI reflects current architecture.
- Workspace is valid and functional.
- Engine core compiles successfully.
- Role crates have canonical ownership structure.
- Legacy properly isolated.
- Apps are canonical entrypoints.

## Explicit current truths

- Root package removal is complete.
- Workspace is valid and functional.
- Engine core compiles successfully.
- Role crates have canonical ownership structure.
- Legacy properly isolated.
- Apps are canonical entrypoints.
- CI reflects current architecture.

## Rule

This file is a snapshot of current truth, not a roadmap.
