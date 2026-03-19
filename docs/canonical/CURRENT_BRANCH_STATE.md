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

**Smoke lane:**
- `just smoke`
- `cargo smoke`
- `scripts/test/smoke.sh`
- `scripts/test/smoke.ps1`

**Contract lane:**
- `just contracts`
- `cargo contracts`
- `scripts/test/contracts.sh`
- `scripts/test/contracts.ps1`

**Certification lane:**
- `just certification`
- `cargo cert`
- `scripts/test/certification.sh`
- `scripts/test/certification.ps1`

**Perf lane:**
- `just perf`
- `scripts/test/perf.sh`
- `scripts/test/perf.ps1`

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
- `crates/engine_physics` — stub placeholder, not real physics runtime
- `crates/engine_tools` — stub placeholder, not real tools implementation

## Current doc truth

- `docs/canonical/PLATFORM_AUDIT.md` — platform audit (NEW)
- `docs/canonical/PHYSICS_CORE_BOUNDARY.md` — physics boundary contract (NEW)
- `docs/canonical/PHYSICS_BOOTSTRAP_CONTRACT.md` — physics bootstrap contract (NEW)

## Current test truth

- `tests/physics_core_boundary_contracts.rs` — physics boundary tests (NEW)
- `tests/physics_bootstrap_contracts.rs` — physics bootstrap tests (NEW)
- `tests/runtime_phase_contracts.rs` — now includes real production calls

## Explicit not-yet-true statements

- `apps/*` packages are declared in workspace but are **not** current canonical launch truth
- `engene_test` bin does **not** exist in current Cargo
- Split is declared but **not** fully enforced
- Package-level entrypoints are **not** yet operator truth
- `sdk_runner.rs` orchestration is **not** yet split into phase methods
- `integration.rs` wiring is **not** yet decomposed into boundary modules
- Full spatial rebuild in SDK redraw is **not** yet replaced by dirty-path incremental
- `engine_physics` is **not** a real physics runtime, only a stub

## Rule

This file is a snapshot, not a roadmap.
Update it when branch truth changes, not when intentions change.
