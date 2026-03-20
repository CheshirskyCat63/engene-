# CURRENT_BRANCH_STATE

## Status label

**Blocked transition. Structural split exists, but ownership handoff is incomplete. Root remains the active migration shell.**

## Current package truth

- Workspace is declared with root package, engine crates, role crates, and `apps/*`.
- Root package `engene` is still active and hosts canonical bins.
- Root feature model now has explicit target taxonomy plus compatibility aliases.
- Role crates exist, but ownership is not fully handed off yet.

## Current entrypoint truth

| Runtime role | Canonical command | Current owner |
|---|---|---|
| Game | `cargo run -p app_engene_game` | apps/engene_game |
| SDK | `cargo run -p app_engene_sdk` | apps/engene_sdk |
| Headless | `cargo run -p app_engene_headless -- --ticks 1200` | apps/engene_headless |
| Bootstrap | `cargo run -p engene_bootstrap` | apps/engene_bootstrap |

## Current architecture truth

- Root is still the active migration shell.
- Root bins are still the canonical operator surface.
- `apps/*` are not canonical launch truth.
- Role crates are not yet full runtime owners.
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
