# APP_ENTRYPOINTS

Date: 2026-03-17

## Thin bootstrap status
- `src/bin/engene_game.rs`: **YES** (delegates directly to `app::game_runner::run_from_env_args`).
- `src/bin/engene_sdk.rs`: **YES** (minimal arg parsing + delegation to `app::sdk_runner::run`).
- `src/bin/engene_headless.rs`: **YES** (delegates directly to `app::headless_runner::run_from_env_args`).

## Ownership model
- Binaries: startup mode selection and handoff only.
- App runners (`src/app/*_runner.rs`): loop/orchestration.
- Engine modules: reusable infrastructure.
- Game modules (`src/game/*`): game semantics/runtime behavior.
- SDK/editor/tooling: `src/tools/*` with SDK runner integration.

## Consistency note
This document is aligned with:
- `ENGENE_FINAL_STATUS_1_0.md`
- `ENGINE_RESTRUCTURE_REPORT.md`
- `RELEASE_BLOCKERS.md`
