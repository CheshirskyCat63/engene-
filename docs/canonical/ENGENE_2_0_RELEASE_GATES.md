# ENGENE 2.0 Release Gates

## Brutal readiness gates (all mandatory)
ENGENE 2.0 is NOT release-ready until every item passes:
1. Workspace split complete and enforced.
2. Dependency law enforced by CI.
3. Schema law enforced by validators/cooker.
4. Event model enforced in runtime + SDK observability.
5. All flagship systems have SDK tooling coverage.
6. Showcase scene passes uninterrupted 20-minute run.
7. Metrics evidence package exported (timings, budgets, counters, hashes).
8. Performance budgets respected in target showcase hardware profile.
9. Save/load continuity proven across L0-L3 transitions.
10. No open P0 blockers.
11. Hot-path allocation budget proven in showcase stress scenario.
12. Multithread execution remains within determinism policy.
13. Synchronization/barrier counts remain within budget.
14. Promotion/demotion spike stress tests pass.
15. Cooked runtime artifacts are locality-optimized and validated.
16. Engine/SDK/Game product boundaries are auditable and intact.
17. No dead legacy entrypoints remain on canonical shipping paths.
18. Repository handoff docs validated by fresh-reader walkthrough.
19. Canonical run/build/cook/debug entrypoints are unique and documented.
20. Quarantined legacy modules are not referenced by hot shipping paths.

## Severity levels
- **P0**: release blocker.
- **P1**: phase blocker.
- **P2**: accepted debt with owner and due date.
- **P3**: known non-blocking imperfection.

## Waiver policy
- P0 waivers are forbidden for final 2.0 release.
- P1 waivers require architecture + product sign-off with expiry date.
- P2/P3 can be accepted only with explicit owner and tracking artifact.

## Gate evidence package
- Build and test command logs.
- Benchmark/perf captures.
- Showcase run recording + metrics snapshot.
- Content validation and cook reports.
- Risk register with final status on each critical risk.
- Allocator activity captures.
- Per-phase/barrier timeline.
- Job system utilization captures.
- Contention report.
- Hot-path benchmark diff against baseline.
- Memory footprint by L0/L1/L2/L3.
- Product boundary audit report.
- Legacy cleanup status manifest.
- Handoff walkthrough checklist/results.

## P0 blocker definition
- Crash/data corruption.
- Determinism/persistence break causing invalid world state.
- Boundary law violation (dependency/schema/event/concurrency/data-layout/product boundaries).
- Showcase flagship system not demonstrable.
- Performance collapse beyond declared degradation ladder.
