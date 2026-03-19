# ENGENE 2.0 Phase A — Batch 8 Execution Report (Closure Batch)

## Concise Batch 8 plan
1. Extract final runtime-neutral query helper abstractions from mixed `core/query` into `engine_ecs::query_contract`.
2. Remove residual ownership duplication in monolith query helpers while preserving compatibility.
3. Publish final closure report and updated Phase A dependency/API snapshots.

## Exact moved-module list
### Runtime-neutral query helper subset completed
No new file move was required in Batch 8.

Extracted ownership into `engine_ecs::query_contract` by moving final neutral helper abstractions out of monolith-local ownership:
- `QueryFilter<Ctx>` trait alias (domain-neutral)
- `And<A, B>` alias (domain-neutral composition alias)
- `QueryIter<'a, Ctx, F>` alias (domain-neutral iterator alias)
- `ReadComponent<'a, T>` / `WriteComponent<'a, T>` wrappers
- `collect_matching(...)` / `count_matching(...)` helpers

Monolith `src/core/query.rs` now consumes/re-exports these crate-owned abstractions and keeps only ECS/world-coupled query filters/bundles.

## Exact compatibility shims added / removed / simplified
- Removed monolith-owned helper trait layer in `src/core/query.rs` (`QueryFilter` local trait definition removed).
- Removed monolith-owned local `And` helper alias ownership (now uses crate alias re-export).
- Removed monolith-owned local `ReadComponent` / `WriteComponent` definitions (now crate re-export).
- Simplified monolith query helper paths by importing/re-exporting crate-owned runtime-neutral query contracts.
- No new compatibility bridge files introduced.

## Exact Cargo/dependency/workspace changes
- No new workspace members.
- No new third-party dependencies.
- No Cargo dependency graph changes required.

## Law-compliance note
- No new features.
- No gameplay/system redesign.
- No deep world/runtime/render/audio/AI migration.
- Root `engene` package remains active.
- Canonical entrypoints unchanged.
- Dependency law and ECS direct-access law remain clean and validated.

## Updated dependency/API report note
- Updated:
  - `docs/generated/PHASE_A_DEPENDENCY_GRAPH.md`
  - `docs/generated/PHASE_A_API_SURFACE_REPORT.md`
- API report now records final runtime-neutral `engine_ecs::query_contract` helper ownership as part of Phase A closure state.
