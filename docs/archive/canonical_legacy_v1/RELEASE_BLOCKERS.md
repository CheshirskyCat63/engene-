# RELEASE_BLOCKERS

Date: 2026-03-17

## Final blocker table (release-gate pass)
| ID | Blocker | Status | Evidence |
|---|---|---|---|
| B-001 | ECS boundary incomplete | CLOSED | `bash scripts/check_ecs_direct_access.sh` passed in this pass. |
| B-002 | Engine->game dependency-direction violations | CLOSED | `bash scripts/check_dependency_direction.sh` passed in this pass. |
| B-003 | Thin bootstrap entrypoints incomplete | CLOSED | `cargo check --bins` passed; bootstrap binaries compile and remain thin wrappers. |
| B-004 | Core runtime/persistence/render test gate failing | CLOSED | `gameplay_and_ai`, `runtime_systems`, `persistence_full`, `save_load_torture`, `render_pipeline` all passed. |
| B-005 | Performance evidence not verified | CLOSED | `cargo bench --profile dev --bench engine_benchmarks` and `cargo bench --profile dev --bench hot_paths -- --sample-size 10` both completed successfully. |

## Open blockers
- None currently evidenced by the required release-gate commands.

## Accepted limitations (explicit)
1. This verdict is limited to verified engine+SDK gates and targeted gameplay integration suites.
2. This pass does not constitute a full game-content completion audit for all single-player scope claims.

## Final readiness verdict
**READY FOR ENGINE + SDK 1.0, GAME PARTIAL**
