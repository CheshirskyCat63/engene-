# CURRENT_BRANCH_STATE

## Status label

**Transition-shaped repo. Split declared; split not fully enforced.**

## Current package truth

- Workspace is declared with root package, engine crates, SDK/game crates, and `apps/*`.
- Root package `engene` is still **active** and hosts canonical bins.
- Root feature model is role-blurry (domain, mode, tooling, bundle features mixed).

## Current entrypoint truth

| Runtime role | Canonical command | Owner |
|---|---|---|
| Game | `cargo run --bin engene_game` | root package bin |
| SDK | `cargo run --bin engene_sdk` | root package bin |
| Headless | `cargo run --bin engene_headless -- --ticks 1200` | root package bin |
| Tools | `cargo run --bin engene_tools` | root package bin |

## Current operator/test truth

### Test lanes
- `just smoke` / `cargo smoke` / `scripts/test/smoke.sh|ps1`
- `just contracts` / `cargo contracts` / `scripts/test/contracts.sh|ps1`
- `just certification` / `cargo cert` / `scripts/test/certification.sh|ps1`
- `just perf` / `scripts/test/perf.sh|ps1`

### Config
- `.config/nextest.toml` — default and ci profiles
- `.cargo/config.toml` — smoke/contracts/cert aliases
- `Justfile` — canonical operator entrypoint

## Current CI truth

- Branches: `main`, `engene-2.0-transition`
- Jobs: `fmt`, `clippy`, `build`, `smoke`, `contracts`
- Certification/perf: not in PR CI, operator-only lanes

## Current architecture truth

- `src/lib.rs` — broad monolith export surface (migration shell)
- `src/app/sdk_runner.rs` — multi-responsibility redraw orchestration
- `src/runtime/wiring/integration.rs` — mixed-domain wiring catch-all
- Root exports: animation, app, audio, body, content, core, engine, game, graphics, input, memory, navigation, network, physics, runtime, simulation, testsupport, tools, world

## Explicit not-yet-true statements

- `apps/*` packages are declared in workspace but are **not** current canonical launch truth
- `engene_test` bin does **not** exist in current Cargo
- Split is declared but **not** fully enforced
- Package-level entrypoints are **not** yet operator truth
- `sdk_runner.rs` orchestration is **not** yet split into phase methods
- `integration.rs` wiring is **not** yet decomposed into boundary modules
- Full spatial rebuild in SDK redraw is **not** yet replaced by dirty-path incremental

## Rule

This file is a snapshot, not a roadmap.
Update it when branch truth changes, not when intentions change.
