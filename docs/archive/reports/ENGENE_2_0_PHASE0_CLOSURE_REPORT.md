# ENGENE 2.0 Phase 0 Closure Report
Date: 2026-03-18
Branch: `engene-2.0-transition`
Status: **Phase 0 closure candidate**

## A) Remaining root legacy files
These remain intentionally after cleanup and are classified for next sanitation pass:

- `GAME_QUICKSTART.md` — REFACTOR_THEN_MOVE.
- `SDK_QUICKSTART.md` — REFACTOR_THEN_MOVE.
- `agent_engine.py` — REFACTOR_THEN_MOVE.
- `build_release.ps1` — CANONICAL_KEEP (helper).
- `package_release.ps1` — CANONICAL_KEEP (helper).
- `run_game.ps1` — CANONICAL_KEEP (helper).
- `run_sdk.ps1` — CANONICAL_KEEP (helper).

## B) Canonical docs normalization result
- `docs/canonical/` now contains only `ENGENE_2_0_*.md` files (single naming baseline).
- Legacy/mixed canonical docs moved to `docs/archive/canonical_legacy_v1/` (count: 34).

### Canonical mapping old -> new baseline

| Old/mixed doc | Canonical ENGENE 2.0 replacement |
|---|---|
| `APP_ENTRYPOINTS.md` | `ENGENE_2_0_ENTRYPOINTS.md` |
| `ENGINE_TO_SDK_TO_GAME_EXECUTION_PLAN.md` | `ENGENE_2_0_PRODUCT_BOUNDARIES.md + ENGENE_2_0_ROADMAP.md` |
| `V1_SCOPE_LOCK.md` | `ENGENE_2_0_SCOPE_LOCK.md` |
| `ENGENE_ENGINE_ARCHITECTURE.md` | `ENGENE_2_0_ARCHITECTURE.md` |
| `ENGENE_SDK_ARCHITECTURE.md` | `ENGENE_2_0_SDK_PROGRAM.md + ENGENE_2_0_PRODUCT_BOUNDARIES.md` |
| `ENGENE_GAME_ARCHITECTURE.md` | `ENGENE_2_0_GAME_50_PLAN.md + ENGENE_2_0_PRODUCT_BOUNDARIES.md` |
| `RUNTIME_TRUTH.md` | `ENGENE_2_0_RUNTIME_TRUTH_MODEL.md` |
| `DEBUGGING_PLAN.md` | `ENGENE_2_0_REPO_HYGIENE_AND_HANDOFF.md + docs/HOW_TO_DEBUG_SHOWCASE.md` |
| `READINESS_REPORT.md` | `ENGENE_2_0_RELEASE_GATES.md` |
| `RELEASE_BLOCKERS.md` | `ENGENE_2_0_RELEASE_GATES.md + ENGENE_2_0_RISK_REGISTER.md` |
| `BENCHMARK_BASELINE.md` | `ENGENE_2_0_PERFORMANCE_STRATEGY.md + ENGENE_2_0_BUDGET_CONTRACTS.md` |
| `DATA_OWNERSHIP_MATRIX.md` | `ENGENE_2_0_PRODUCT_BOUNDARIES.md + ENGENE_2_0_SCHEMA_LAW.md` |
| `SANDBOX_MANUAL_TEST_PLAN.md` | `ENGENE_2_0_GAME_50_PLAN.md + ENGENE_2_0_SHOWCASE_TRUTH_RULES.md` |
| `SANDBOX_ACCEPTANCE_CHECKLIST.md` | `ENGENE_2_0_RELEASE_GATES.md + ENGENE_2_0_GAME_50_PLAN.md` |
| `ENGINE_AUDIT_FULL.md` | `ENGENE_2_0_PHASE0_CLEANUP_REPORT.md + ENGENE_2_0_PHASE0_CLOSURE_REPORT.md` |

## C) Archived / quarantined items
- Archived docs: `docs/archive/canonical_legacy_v1/*.md`
- Quarantined phase0 root artifacts: `legacy/quarantine/phase0/*`
- Quarantined historical crashes: `legacy/quarantine/crashes_legacy/*`
- Active crash directory preserved with `crashes/.gitkeep`.

## D) Handoff readability improvements
- Added `docs/REPO_MAP.md` (ownership and canonical path map).
- Added `docs/HOW_TO_RUN.md` (single canonical run paths).
- Added `docs/HOW_TO_VALIDATE_AND_COOK.md` (Phase 0 validation baseline).
- Added `docs/HOW_TO_DEBUG_SHOWCASE.md` (showcase debug/handoff pointers).
- Rewrote root quickstarts to prioritize canonical `cargo run --bin ...` entrypoints.

## E) Recommendation: may Phase A begin?
**YES, with minor tracked follow-ups.**

Rationale:
- Canonical naming baseline is normalized in `docs/canonical/`.
- Old mixed canonical docs are archived/quarantined explicitly.
- Primary entrypoint ambiguity in root docs is reduced and canonicalized.
- Product-boundary, dependency, and entrypoint laws remain respected.

Follow-ups before/early Phase A:
- Classify `agent_engine.py` ownership (`REFACTOR_THEN_MOVE` or quarantine).
- Decide whether quickstart docs stay root-level or migrate under `docs/` index.
- Add machine-readable quarantine manifest for automated stale checks.


## F) Phase A start gate (go / no-go)
- `docs/canonical/` is sole canonical truth set for 2.0 law/program docs: **YES**.
- Root repository entrypoints/readme path is non-ambiguous and cleaned from false primary binaries: **YES**.
- Canonical entrypoints are single and explicit (`cargo run --bin ...`): **YES**.

Phase A recommendation: **GO**.
