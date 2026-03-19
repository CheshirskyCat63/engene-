# ENGENE 2.0 Phase A — Batch 7 Execution Report

## Concise Batch 7 plan
1. Finish the next safe runtime-neutral ECS query contract carve-outs from mixed `core/query`.
2. Keep gameplay/world-coupled query filters and bundled iterators in monolith.
3. Keep compatibility bridges centralized/thin and avoid new shim proliferation.
4. Produce strict closure-oriented Phase A assessment with explicit must-complete vs deferred lists.

## Exact moved-module list
### Moved into `engine_ecs`
- Extended `crates/engine_ecs/src/query_contract.rs` with additional runtime-neutral query contracts:
  - `ReadComponent<'a, T>`
  - `WriteComponent<'a, T>`
  - `collect_matching(...)`
  - `count_matching(...)`

### Monolith carve-out usage updates
- `src/core/query.rs` now consumes the above runtime-neutral query contracts from `engine_ecs::query_contract`.
- No gameplay-specific filters or world-coupled query bundles were moved.

## Exact compatibility shims added / removed / simplified
- `src/core/query.rs` now re-exports `ReadComponent`/`WriteComponent` from `engine_ecs::query_contract` instead of owning local definitions.
- `query_collect` / `query_count` in `src/core/query.rs` now delegate to crate-owned generic helpers (`collect_matching` / `count_matching`).
- No new compatibility bridge files added in Batch 7.
- Existing centralized content bridge from Batch 6 remains unchanged (`src/content/mod.rs`).

## Exact Cargo/dependency/workspace changes
- No new workspace members.
- No new third-party dependencies.
- `crates/engine_ecs/src/lib.rs` already exports `query_contract`; no additional crate graph changes needed this batch.

## Law-compliance note
- No new features.
- No gameplay/system redesign.
- No deep world/runtime/render/audio/AI migration.
- Root `engene` package remains active and canonical entrypoints unchanged.
- Dependency direction and ECS direct-access laws remain clean and validated.

## Updated dependency/API report note
- Updated Batch 7 generated snapshots:
  - `docs/generated/PHASE_A_DEPENDENCY_GRAPH.md`
  - `docs/generated/PHASE_A_API_SURFACE_REPORT.md`
- API report now reflects deeper runtime-neutral content of `engine_ecs::query_contract` (wrappers + helpers), not only matcher/iterator skeleton.

## Strict Phase A closure assessment
### Must-complete-before-close
1. **One final precision carve-out in `core/query`:** move remaining domain-neutral query helper abstractions that are still monolith-owned but independent of gameplay/world-specific component bundles.
2. Remove any residual redundant compatibility bridge layers that still duplicate crate-owned surfaces (if discovered during final pass).

### Deferred-to-later-phases
1. Gameplay-specific/component-bound query filters and bundled iterators (NPC/monster/AI flavored data bundles) that are tightly coupled to world/gameplay components.
2. Deep world/runtime/render/audio/AI ownership migrations.
3. Non-contract optimizations or behavioral/perf refactors.

### Direct closure recommendation
- **Phase A still cannot close after this batch.**
- Reason: there is still a small but real set of runtime-neutral query helper abstractions in `core/query` that should be crate-owned before claiming closure.

## Smallest safe closure batch definition (if still open)
- **Batch 8 (closure batch):**
  1. Extract only the last runtime-neutral query helper subset from `core/query` into `engine_ecs::query_contract`.
  2. Verify no redundant monolith bridge duplication remains for already-owned contracts.
  3. Regenerate reports + re-run gates + publish final Phase A closure report.
