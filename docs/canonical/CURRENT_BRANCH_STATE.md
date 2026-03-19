# CURRENT_BRANCH_STATE

## Status label

**Transition finish-ready. Structural split active; root constrained to thin compatibility shell.**

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

**Legacy recovery lane:**
- `just legacy-recovery`
- `scripts/test/legacy_recovery.sh`
- `scripts/test/legacy_recovery.ps1`

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

- `src/lib.rs` — reduced compatibility export surface (migration shell)
- `src/app/sdk_runner.rs` — phase driver shell with explicit redraw phase calls
- `src/app/sdk_runner/sdk_runner_phases/*` — phase modules (`tick`, `streaming`, `persistence`, `spatial`, `audio`, `editor`, `render`)
- `src/app/sdk_runner/sdk_runner_phases/spatial.rs` — explicit no-op / incremental / controlled full-rebuild spatial policy
- `src/runtime/wiring/integration/mod.rs` + `integration/*` — boundary module tree (old flat `integration.rs` removed)
- Root exports: animation, app, audio, body, content, core, game, graphics, input, memory, navigation, network, physics, runtime, simulation, testsupport, tools, world
- `crates/engine_physics` — real minimal physics seam with bootstrap validation path
- `crates/engine_tools` — stub placeholder, not real tools implementation

## Current doc truth

- `docs/canonical/PLATFORM_AUDIT.md` — platform audit (NEW)
- `docs/canonical/PHYSICS_CORE_BOUNDARY.md` — physics boundary contract (NEW)
- `docs/canonical/PHYSICS_BOOTSTRAP_CONTRACT.md` — physics bootstrap contract (NEW)

## Current test truth

- `tests/physics_core_boundary_contracts.rs` — physics boundary tests (NEW)
- `tests/physics_bootstrap_contracts.rs` — physics bootstrap tests (NEW)
- `tests/runtime_phase_contracts.rs` — now includes real production calls
- `tests/spatial_dirty_contracts.rs` — production spatial dirty path contract checks

## Explicit not-yet-true statements

- `apps/*` packages are declared in workspace but are **not** current canonical launch truth
- `engene_test` bin does **not** exist in current Cargo
- Split is active and compile-clean on current platform gate path
- Package-level entrypoints are **not** yet operator truth
- Spatial hot path uses journal-driven dirty tracking; no global ECS dirty scan each redraw
- Migration is **not** finished while root shell ownership is still active
- Root shell is active as compatibility shell and is not owner of new policy decisions
- Migration finish-ready state is reached; remaining work is post-migration handoff and optimization
- Legacy integration tests (`world_streaming`, `physics_body_combat`, `content_pipeline`, `persistence_full`, `runtime_systems`, `gameplay_and_ai`) are **not** part of current platform gate and are isolated in `tests_legacy/`

## Rule

This file is a snapshot, not a roadmap.
Update it when branch truth changes, not when intentions change.
