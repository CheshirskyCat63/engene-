# ENGENE 2.0 Foundation Structure Map

Date: 2026-03-18
Status: Canonical structural ownership map after foundation-structure closure batch.

## 1) Source tree by ownership
- `src/core/*` — kernel/core engine contracts (ECS/system/runtime-agnostic kernel services).
- `src/runtime/bootstrap/*` — runtime assembly composition by role (`game`, `headless`, `tools`) + shared bootstrap primitives.
- `src/runtime/wiring/*` — runtime glue systems that connect domain modules into tick/frame flow.
- `src/app/*` — product runners and launch lifecycle coordination (`game_runner`, `headless_runner`, `sdk_runner`, `tools_runner`).
- `src/game/*` — game domain systems/plugins/content logic only.
- `src/tools/*` — tools-domain/editor/doctor dashboards and support runtime utilities.
- `src/testsupport/*` — support-only harnesses and test fixtures; never runtime entrypoint ownership.
- `src/bin/*` — canonical executable entrypoints.

## 2) Runtime tree by role
- `src/runtime/bootstrap/game.rs` + `game_resources.rs` + `game_systems.rs` + `game_plugins.rs` => **Game runtime role**.
- `src/runtime/bootstrap/headless.rs` => **Headless/Kernel runtime role**.
- `src/runtime/bootstrap/tools.rs` => **Tools/SDK runtime role**.
- `src/runtime/bootstrap/common.rs` => shared bootstrap primitives only (no role-specific leaks).
- `src/runtime/wiring/*` => cross-role runtime glue (non-domain API ownership).

## 3) App tree by responsibility
- `src/app/game_runner/mod.rs` — game launch assembly + environment arg dispatch.
- `src/app/game_runner/app_loop.rs` — game event loop and per-frame orchestration.
- `src/app/game_runner/diagnostics.rs` — game-runner diagnostics outputs.
- `src/app/headless_runner.rs` — headless/kernel runner.
- `src/app/sdk_runner.rs` — SDK workstation runner.
- `src/app/tools_runner.rs` — tools diagnostics runner.

## 4) Tests tree by responsibility
- `tests/runtime_role_separation.rs` — runtime role boundaries and profile discipline.
- `tests/tools_runtime_purity.rs` — tools runtime purity invariants.
- `tests/certification_bootstrap_invariants.rs` — bootstrap ownership/coupling certification guards.
- `tests/runtime_systems.rs` + `tests/runtime_systems/*` — runtime systems contracts split by category.
- `tests/e2e_regression.rs`, `tests/production_candidate.rs` — end-to-end and release gate integration checks.
- `tests/persistence_full.rs`, `tests/save_load_torture.rs` — persistence contracts.
- `tests/perf_lowspec_mt.rs`, `tests/performance.rs` — performance/lowspec checks.

## 5) Docs tree by responsibility
- `docs/canonical/*` — canonical ownership laws, runtime/dependency baseline, entrypoints, and closure reports.
- `docs/generated/*` — generated evidence reports.
- `docs/REPO_MAP.md`, `docs/HOW_TO_RUN.md`, `docs/HOW_TO_DEBUG_SHOWCASE.md` — operator-facing guides (must not override canonical law).

## 6) Remaining heavy zones
- `tests/ai_social_economy.rs` — high-volume integration contracts across AI/social/economy surface.
- `tests/physics_body_combat.rs` — broad physics/body/combat integration matrix.
- `src/runtime/bootstrap/game_resources.rs` — large resource registration surface.
- `src/app/sdk_runner.rs` — mixed shell + runtime loop concerns.

## 7) Next split targets
1. `tests/ai_social_economy.rs` split by domain seam (`ai_social`, `economy`, `cross-domain`).
2. `tests/physics_body_combat.rs` split into `physics_core`, `body_pipeline`, `combat_integration`.
3. `src/runtime/bootstrap/game_resources.rs` split by ownership sub-buckets (`world`, `physics`, `audio`).
4. `src/app/sdk_runner.rs` split into startup/bootstrap and event-loop execution modules.
