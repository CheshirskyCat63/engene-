# ENTRYPOINT_TRUTH

## Rule

There must be one canonical description of how ENGENE starts in the current branch.

## Current canonical entrypoints

| Runtime role | Binary entrypoint | First called crate | Current runner body location | Canonical owner | Status |
|---|---|---|---|---|---|
| Game | `cargo run -p engene_game` | `runtime_headless` | `crates/runtime_headless/src/lib.rs` | `runtime_headless` | headless mode (game window pending) |
| SDK | `cargo run -p engene_sdk` | `sdk_app` | `crates/sdk_app/src/lib.rs` | `sdk_app` | app entrypoint |
| Headless | `cargo run -p engene_run -- --ticks 1200` | `runtime_headless` | `crates/runtime_headless/src/lib.rs` | `runtime_headless` | app entrypoint |

## Current truth

Package-level execution through `apps/*` is canonical launch truth.
Apps are real entrypoints, not transitional shells.
Root bins are not current operator truth.
Headless runtime ownership is `runtime_headless`, not `game_framework`.
`game_framework` is reserved for future game-specific launch paths.

## Transition rule

Package-level runtime ownership is real. This file reflects current truth.
