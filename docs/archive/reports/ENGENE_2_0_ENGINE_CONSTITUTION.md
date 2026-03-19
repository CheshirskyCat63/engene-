# ENGENE 2.0 Engine Constitution

## Purpose
Meta-law for canonical 2.0 documents: precedence, conflict resolution, mandatory updates, and violation handling.

## Canonical constitutional docs list
- Scope lock, product boundaries, repo hygiene/handoff, legacy cleanup policy.
- Architecture, dependency law, schema law, event model.
- Content pipeline, simulation strategy, entity lifecycle, determinism policy.
- Concurrency model, data layout policy, CPU performance law, memory budgets.
- Budget contracts, runtime truth model, AI architecture.
- Showcase truth rules, release gates, roadmap.

## Precedence order
1. `ENGENE_2_0_SCOPE_LOCK.md` + `ENGENE_2_0_PRODUCT_BOUNDARIES.md`
2. Law docs (`DEPENDENCY_LAW`, `SCHEMA_LAW`, `EVENT_MODEL`, `ENTITY_LIFECYCLE`, `DETERMINISM_POLICY`, `RUNTIME_TRUTH_MODEL`, `CONCURRENCY_MODEL`, `DATA_LAYOUT_POLICY`, `CPU_PERFORMANCE_LAW`, `MEMORY_BUDGETS`)
3. Contracts (`BUDGET_CONTRACTS`, `RELEASE_GATES`, `SHOWCASE_TRUTH_RULES`)
4. Architecture and strategy docs
5. Program/roadmap docs

## Mandatory update rules
Any architecture-breaking or performance-breaking change requires same-PR updates to impacted canonical docs.

At minimum update:
- product ownership change -> product boundaries + architecture + roadmap.
- repo cleanup/handoff policy change -> repo hygiene/handoff + legacy cleanup policy.
- dependency edges -> dependency law + architecture.
- schema/content meaning -> schema law + content pipeline + data layout policy.
- signal routing semantics -> event model + runtime truth model + concurrency model.
- L0-L3 transitions -> simulation strategy + entity lifecycle + determinism policy.
- budgets/gates -> budget contracts + CPU performance law + memory budgets + release gates.

## Violation severity levels
- **P0**: release-blocking constitutional/law breach.
- **P1**: phase-blocking policy breach.
- **P2**: accepted debt with named owner and due date.
- **P3**: non-blocking imperfection.

## Emergency exception process
- Allowed only for P0 operational emergency.
- Must include owner, rollback plan, expiry date.
- Cannot waive scope lock or truth-for-showcase prohibitions.
- Cannot waive performance-first hot-path law for Tier A paths.

## Stale-doc policy
If implementation and canonical docs diverge, implementation is considered non-compliant until docs are aligned or code is corrected.
