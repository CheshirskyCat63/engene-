# ENGENE 2.0 Public API Surface Map

Date: 2026-03-18
Status: Current public surface classification after foundation cleanup.

## Stable public API (intended externally reachable surface)
- `src/app/*` runners: process entry lifecycle contract for game/headless/sdk/tools binaries.
- `src/runtime/bootstrap/*`: explicit role bootstrap API (`GameRuntimeAssembly`, `EngineRuntimeAssembly`, `ToolsRuntimeAssembly`).
- `src/core/runtime_config` and `src/core/engine` contracts used by integration tests and runtime launch paths.
- `src/world/components/*` data contracts used across systems/tests.

## Public but suspicious (overexposed for legacy convenience)
- Root `src/lib.rs` currently exports all domain trees (`animation`, `audio`, `graphics`, `network`, etc.).
- `src/engine.rs` convenience umbrella re-exports almost every subsystem.
- `src/core/mod.rs` and `src/tools/mod.rs` expose broad module sets instead of narrow façade slices.
- `src/testsupport/*` is public for integration-test access, but not production API.

## Internal-only target (should move to narrower visibility in later split)
- runtime wiring internals (`src/runtime/wiring/*`)
- testsupport harness internals (`src/testsupport/*`)
- app loop internals (`src/app/game_runner/app_loop.rs`, diagnostics internals)
- high-volume domain implementation modules under `src/graphics/*`, `src/animation/*`, `src/audio/*`

## This batch API changes
1. Removed root `src/sdk.rs` compatibility surface to avoid duplicate/ambiguous SDK export namespace.
2. Kept role entrypoints explicit (`src/bin/*` -> `src/app/*` -> `src/runtime/bootstrap/*`).
3. Preserved broad root exports for now to avoid breaking existing workspace integration tests; classified as suspicious debt.

## Next API hardening targets
1. Introduce explicit `pub mod api` in root crate and migrate integration tests off deep `pub mod` graph.
2. Move `src/testsupport` behind test-only feature gate once remaining integration tests stop importing it directly.
3. Constrain `src/engine.rs` to curated re-exports only (kernel/runtime contracts first).
