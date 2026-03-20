# ENTRYPOINT_TRUTH

## Rule

There must be one canonical description of how ENGENE starts in the current branch.

## Current canonical entrypoints

| Runtime role | Current command | Current owner |
|---|---|---|
| Game | `cargo run -p app_engene_game` | apps/engene_game |
| SDK | `cargo run -p app_engene_sdk` | apps/engene_sdk |
| Headless | `cargo run -p app_engene_headless -- --ticks 1200` | apps/engene_headless |
| Bootstrap | `cargo run -p engene_bootstrap` | apps/engene_bootstrap |

## Not current truth

The following are not canonical launch truths in this branch:
- package-level execution through `apps/*`
- any undocumented `test` / `sandbox` / `demo` bin

## Ownership statement

Root bins are the current operator truth.
Root is still a thin migration shell.
This does not mean root owns long-term runtime architecture.

## Transition rule

When package-level runtime ownership becomes real, this file changes in one step.
Until then, no document may advertise future package-level commands as current truth.
