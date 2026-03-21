PS> cd /d C:\workspaces\vela-prod-16434-1521-0-9bdac8f6-1d20-471c-bb3c-0c0f8a6b9c9e-x && cargo build --workspace 2>&1 | head -100PS> cd /d C:\workspaces\vela-prod-16434-1521-0-9bdac8f6-1d20-471c-bb3c-0c0f8a6b9c9e-x && cargo build --workspace 2>&1 | head -100PS> cd /d C:\workspaces\vela-prod-16434-1521-0-9bdac8f6-1d20-471c-bb3c-0c0f8a6b9c9e-x && cargo build --workspace 2>&1 | head -100PS> cd /d C:\workspaces\vela-prod-16434-1521-0-9bdac8f6-1d20-471c-bb3c-0c0f8a6b9c9e-x && cargo build --workspace 2>&1 | head -100# MIGRATION LEDGER

## Purpose

This file tracks all transitional/debt items in the engene-2.0-transition branch.
Every temporary thing must be logged here with removal condition.

Format: `item | current_location | temporary_owner | target_owner | why_temporary | removal_condition | blocker | status`

---

## Root Shell Temporary Exports

| Item | Current Location | Temp Owner | Target Owner | Why Temporary | Removal Condition | Blocker | Status |
|------|------------------|------------|--------------|---------------|-------------------|---------|--------|
| animation module | src/lib.rs | root | crates/engine_animation | legacy module in root | crates split complete | none | PENDING |
| app module | src/lib.rs | root | apps/* or crate | legacy module in root | apps ownership complete | apps not canonical yet | ACTIVE |
| audio module | src/lib.rs | root | crates/engine_audio | legacy module in root | crates split complete | none | PENDING |
| body module | src/lib.rs | root | crates/engine_physics | legacy module in root | crates split complete | none | PENDING |
| content module | src/lib.rs | root | crates/engine_content | legacy module in root | crates split complete | none | PENDING |
| core module | src/lib.rs | root | crates/engine_core | legacy module in root | crates split complete | none | PENDING |
| game module | src/lib.rs | root | crates/game_framework | legacy module in root | crates split complete | none | PENDING |
| graphics module | src/lib.rs | root | crates/engine_render | legacy module in root | crates split complete | none | PENDING |
| input module | src/lib.rs | root | crates/engine_input | legacy module in root | crates split complete | none | PENDING |
| memory module | src/lib.rs | root | TBD | legacy module in root | cleanup needed | none | PENDING |
| navigation module | src/lib.rs | root | crates/engine_navigation | legacy module in root | crates split complete | none | PENDING |
| network module | src/lib.rs | root | crates/engine_network | legacy module in root | crates split complete | none | PENDING |
| physics module | src/lib.rs | root | crates/engine_physics | legacy module in root | crates split complete | none | PENDING |
| runtime module | src/lib.rs | root | crates/engine_runtime | legacy module in root | crates split complete | none | PENDING |
| simulation module | src/lib.rs | root | TBD | legacy module in root | cleanup needed | none | PENDING |
| testsupport module | src/lib.rs | root | TBD | test helper in root | test cleanup | none | PENDING |
| tools module | src/lib.rs | root | crates/engine_tools | legacy module in root | crates split complete | none | PENDING |
| world module | src/lib.rs | root | crates/engine_world | legacy module in root | crates split complete | none | PENDING |

## Root Binaries (Current Canonical)

| Item | Current Location | Temp Owner | Target Owner | Why Temporary | Removal Condition | Blocker | Status |
|------|------------------|------------|--------------|---------------|-------------------|---------|--------|
| engene_game bin | Cargo.toml | root | apps/engene_game | migration shell | apps become canonical | apps are transitional wrappers, not yet canonical | ACTIVE |
| engene_sdk bin | Cargo.toml | root | apps/engene_sdk | migration shell | apps become canonical | apps are transitional wrappers, not yet canonical | ACTIVE |
| engene_headless bin | Cargo.toml | root | apps/engene_headless | migration shell | apps become canonical | apps are transitional wrappers, not yet canonical | ACTIVE |
| engene_tools bin | Cargo.toml | root | apps/engene_tools | migration shell | apps become canonical | apps are transitional shells | ACTIVE |

## Placeholder Crates

| Item | Current Location | Temp Owner | Target Owner | Why Temporary | Removal Condition | Blocker | Status |
|------|------------------|------------|--------------|---------------|-------------------|---------|--------|
| sdk_app crate | crates/sdk_app | root | TBD | placeholder facade | real ownership crate or removal | none | PENDING |
| game_framework crate | crates/game_framework | root | TBD | placeholder facade | real ownership crate or removal | none | PENDING |
| engine_tools crate | crates/engine_tools | root | TBD | placeholder facade | real ownership crate or removal | none | PENDING |

## Role Crates (Transitional Facades)

| Item | Current Location | Temp Owner | Target Owner | Why Temporary | Removal Condition | Blocker | Status |
|------|------------------|------------|--------------|---------------|-------------------|---------|--------|
| legacy feature aliases | Cargo.toml `[features]` | root package | explicit `role_*` / `cap_*` / `tool_*` / `profile_*` taxonomy | compatibility for existing cfg / operator habits | remove when all docs/tests/scripts/cfg use target taxonomy only | compatibility window | TRANSITIONAL |
| canonical root bins | src/bin/* | root package | role crates / runtime crates after real ownership handoff | current operator truth still enters through root | remove when package-level runtime entrypoints become real operator truth | runtime ownership handoff incomplete | TRANSITIONAL |
| role crates facade status | crates/sdk_app, crates/game_framework, crates/engine_tools | placeholder role crates | real role owners | crates exist before full behavior ownership handoff | remove when role runtime logic is physically owned outside root shell | runner ownership not fully migrated | TRANSITIONAL |
| root public export shell | src/lib.rs | root package | crate-owned public surfaces | keep internal modules mounted while shrinking external surface | remove private legacy mounts after ownership handoff | internal modules still rely on root mount points | TRANSITIONAL |
| root app shell | src/app/mod.rs | root package | role crates / runtime crates | canonical bins still dispatch through root app shell | remove when runtime role code is physically owned outside root | runner extraction not complete | TRANSITIONAL |
| sdk_app facade | crates/sdk_app/src/lib.rs | sdk_app | sdk_app runtime owner | facade exists before real SDK runtime ownership | remove transitional status when SDK startup/orchestration is no longer rooted in root shell | runner extraction not complete | TRANSITIONAL |
| game_framework facade | crates/game_framework/src/lib.rs | game_framework | game_framework runtime owner | facade exists before real Game runtime ownership | remove transitional status when Game startup/orchestration is no longer rooted in root shell | runner extraction not complete | TRANSITIONAL |
| engine_tools facade | crates/engine_tools/src/lib.rs | engine_tools | engine_tools runtime owner | facade exists before real tools runtime ownership | remove transitional status when Tools runtime no longer boots through root shell | runner extraction not complete | TRANSITIONAL |
| role crates root dependencies | crates/game_framework/src/lib.rs, crates/sdk_app/src/lib.rs | game_framework, sdk_app | game_framework, sdk_app | role crates still import from root crate | move dependencies to engine crates or role crates | engine crates not fully owned, world types not extracted | TRANSITIONAL |

## ECS Decomposition (Step 2 - Phase 2a.2 COMPLETED)

| Item | Current Location | Temp Owner | Target Owner | Why Temporary | Removal Condition | Blocker | Status |
|------|------------------|------------|--------------|---------------|-------------------|---------|--------|
| EcsMechanics ownership | crates/engine_ecs/src/ecs_mechanics.rs | engine_ecs | engine_ecs | PURE ECS mechanics now owned by engine_ecs | none - ownership transfer complete | none | PHASE_2A2_COMPLETE |
| IdentityRegistry ownership | crates/engine_ecs/src/persistent_id.rs | engine_ecs | engine_ecs | IdentityRegistry already in engine_ecs | none - ownership complete | none | PHASE_2A2_COMPLETE |
| Entity type ownership | crates/engine_ecs/src/entity.rs | engine_ecs | engine_ecs | Entity already in engine_ecs | none - ownership complete | none | PHASE_2A2_COMPLETE |
| SparseSet ownership | crates/engine_ecs/src/sparse_set.rs | engine_ecs | engine_ecs | Generic SparseSet in engine_ecs | none - ownership complete | none | PHASE_2A2_COMPLETE |
| Storage subsystem | src/core/ecs_internal/storage.rs | root | engine_ecs | TRANSITIONAL - depends on world::components types | move when world::components moves to engine_world | world::components in root | TRANSITIONAL |
| Lifecycle subsystem | src/core/ecs_internal/lifecycle.rs | root | engine_ecs | TRANSITIONAL - uses EcsMechanics + game-specific journals | move when world types decoupled | MonsterSpecies in world::components | TRANSITIONAL |
| Ecs composition layer | src/core/ecs_internal/mod.rs | root | engine_ecs | MINIMAL wiring only - delegates to engine_ecs for mechanics | remove when all consumers migrate | root consumers still depend on legacy API | TRANSITIONAL |
| Ecs transitional facade | src/core/ecs.rs | root | engine_ecs | re-exports from ecs_internal for backward compatibility | remove when all consumers migrate to new structure | many root consumers still use old API | TRANSITIONAL |
| component_registry dependency | src/core/component_registry.rs | root | engine_ecs | depends on world::components + ai_* modules | refactor after world component extraction | world types + AI modules still in root | BLOCKED |
| query module dependency | src/core/query.rs | root | engine_ecs | depends on monolithic Ecs + world components | refactor after Ecs decomposition complete | Ecs still has world component dependencies | BLOCKED |
| mutation_policy dependency | src/core/mutation_policy.rs | root | engine_ecs | depends on Ecs, CommandBuffer, EventBus, Resources | refactor after Ecs decomposition complete | root runtime types still coupled | BLOCKED |

## Core Slimming (Step 1 - COMPLETED)

| Item | Current Location | Temp Owner | Target Owner | Why Temporary | Removal Condition | Blocker | Status |
|------|------------------|------------|--------------|---------------|-------------------|---------|--------|
| engine_core slim to kernel | crates/engine_core | engine_core | engine_core | removed runtime/meta/tooling modules, now pure std | none - kernel is now clean | none | COMPLETED |
| runtime_config move | crates/engine_core/src/runtime_config | engine_core | engine_runtime | runtime config belongs to runtime container | move module to engine_runtime | none pending | PENDING |
| runtime_manifest move | crates/engine_core/src/runtime_manifest | engine_core | engine_runtime | manifest belongs to runtime container | move module to engine_runtime | none pending | PENDING |
| build_manifest move | crates/engine_core/src/build_manifest | engine_core | engine_runtime | build/asset manifest belongs to runtime container | move module to engine_runtime | uses serde/ron | PENDING |
| integration_matrix move | crates/engine_core/src/integration_matrix | engine_core | engine_runtime | integration config belongs to runtime container | move module to engine_runtime | none pending | PENDING |
| metrics_registry move | crates/engine_core/src/metrics_registry | engine_core | engine_runtime | metrics belong to runtime container | move module to engine_runtime | none pending | PENDING |
| ownership_map move | crates/engine_core/src/ownership_map | engine_core | engine_runtime | ownership tracking belongs to runtime container | move module to engine_runtime | none pending | PENDING |
| profiler move | crates/engine_core/src/profiler | engine_core | engine_runtime | profiler belongs to runtime container | move module to engine_runtime | none pending | PENDING |
| registry move | crates/engine_core/src/registry | engine_core | engine_runtime | registry belongs to runtime container | move module to engine_runtime | none pending | PENDING |

## Step 3 - engine_runtime Ownership (Phase 3 IN PROGRESS)

| Item | Current Location | Temp Owner | Target Owner | Why Temporary | Removal Condition | Blocker | Status |
|------|------------------|------------|--------------|---------------|-------------------|---------|--------|
| WorldTickSystem ownership | src/runtime/wiring/world_tick.rs → crates/engine_runtime/src/simulation_core/systems/ | engine_runtime | engine_runtime | Orchestration logic moved to engine_runtime | None - ownership transfer complete | none | PHASE_3_COMPLETE |
| TransitionOrchestrator ownership | crates/engine_runtime/src/simulation_core/orchestrator.rs | engine_runtime | engine_runtime | Already in engine_runtime (simulation transitions) | None | none | COMPLETE |
| Scheduler ownership | src/core/scheduler.rs → crates/engine_runtime/src/simulation_core/systems/ | engine_runtime | engine_runtime | Runtime scheduling logic | None - ownership transfer complete | none | PHASE_3_COMPLETE |
| EngineSystem trait ownership | src/core/system.rs → crates/engine_runtime/src/simulation_core/systems/ | engine_runtime | engine_runtime | System trait for runtime orchestration | None - ownership transfer complete | none | PHASE_3_COMPLETE |
| Runtime system registration | src/runtime/bootstrap/game_systems.rs | root | engine_runtime | App-shell glue - registers systems for different modes | Move when full system ownership resolved | root still has game-specific systems | TRANSITIONAL |
| Runtime bootstrap | src/runtime/bootstrap/mod.rs | root | root | App-shell glue - composition surface | None - stays in root as launch layer | none | ACTIVE |

**Verification:**
- `cargo check -p engine_runtime` ✅
- `cargo test -p engine_runtime` ✅ (5 tests passed)
- Root wiring now re-exports from engine_runtime ✅

## ECS Decomposition (Phase 2a.2 - PARTIAL OWNERSHIP TRANSFER COMPLETE)

| Item | Current Location | Temp Owner | Target Owner | Why Temporary | Removal Condition | Blocker | Status |
|------|------------------|------------|--------------|---------------|-------------------|---------|--------|
| EcsMechanics ownership | crates/engine_ecs/src/ecs_mechanics.rs | engine_ecs | engine_ecs | Pure ECS mechanics (spawn/despawn/unload, alive tracking, tick, dirty journals) | None - ownership transfer complete | none | COMPLETE |
| IdentityRegistry ownership | crates/engine_ecs/src/persistent_id.rs | engine_ecs | engine_ecs | Already in engine_ecs | None | none | COMPLETE |
| Entity type ownership | crates/engine_ecs/src/entity.rs | engine_ecs | engine_ecs | Already in engine_ecs | None | none | COMPLETE |
| GenEntity ownership | crates/engine_ecs/src/ecs_mechanics.rs | engine_ecs | engine_ecs | Already in engine_ecs | None | none | COMPLETE |
| SparseSet ownership | crates/engine_ecs/src/sparse_set.rs | engine_ecs | engine_ecs | Generic SparseSet in engine_ecs | None | none | COMPLETE |
| Storage subsystem | src/core/ecs_internal/storage.rs | root | engine_ecs | Depends on world::components types (Transform, EntityKind, etc.) | Move when world::components moves to engine_world | world::components in root | TRANSITIONAL |
| Lifecycle subsystem | src/core/ecs_internal/lifecycle.rs | root | engine_ecs | Uses EcsMechanics + game-specific journals (spatial, territory) | Move when MonsterSpecies moves to engine_world | MonsterSpecies in world::components | TRANSITIONAL |
| Ecs composition layer | src/core/ecs_internal/mod.rs | root | engine_ecs | Minimal wiring only - delegates to engine_ecs for mechanics | Remove when all consumers migrate | root consumers depend on legacy API | TRANSITIONAL |
| Ecs facade | src/core/ecs.rs | root | engine_ecs | Re-exports from ecs_internal for backward compatibility | Remove when all consumers migrate | root consumers use legacy API | TRANSITIONAL |

**Verification:**
- `cargo check -p engine_ecs` ✅
- `cargo test -p engine_ecs` ✅ (23 tests passed)
- No duplicate Entity/IdentityRegistry/EcsMechanics in root ✅
- Root uses engine_ecs via re-exports ✅

## Foundation Type Moves (Real Migrations)

| Item | Current Location | Temp Owner | Target Owner | Why Temporary | Removal Condition | Blocker | Status |
|------|------------------|------------|--------------|---------------|-------------------|---------|--------|
| canonical Entity alias | src/core/ecs.rs → crates/engine_ecs/src/entity.rs | engine_ecs | engine_ecs | foundation identity moved first without moving monolithic ECS container | complete when root code stops using root-owned Entity alias paths | root ECS container still monolithic | MOVED |
| world scale constants | src/world/cell.rs → crates/engine_world/src/cell.rs | engine_world | engine_world | constants are low-coupling and safe to extract before full Cell ownership | complete when root world code imports scale constants from engine_world | full Cell type still coupled to biome/entity ownership | MOVED |
| WorldFields | src/world/fields.rs → crates/engine_world/src/fields.rs | engine_world | engine_world | low-coupling state surface moved intact | complete when root/world users import WorldFields from engine_world | none for moved surface | MOVED |
| Heightmap extraction | src/world/heightmap.rs | root world | engine_world | delayed because `generate` still couples to biome ownership | move when biome ownership is explicit and move can remain near-verbatim | biome coupling | BLOCKED |
| monolithic Ecs container | src/core/ecs.rs | root runtime shell | engine_runtime + engine_ecs split surface | not safe to move as one block | remove when runtime container and ECS storage ownership are separated explicitly | monolithic coupling | BLOCKED |

## Apps (Transitional Shells)

| Item | Current Location | Temp Owner | Target Owner | Why Temporary | Removal Condition | Blocker | Status |
|------|------------------|------------|--------------|---------------|-------------------|---------|--------|
| app_engene_game | apps/engene_game | root | self | transitional wrapper | package execution replaces root bins | root still canonical, apps are transitional wrappers | ACTIVE |
| app_engene_sdk | apps/engene_sdk | root | self | transitional wrapper | package execution replaces root bins | root still canonical, apps are transitional wrappers | ACTIVE |
| app_engene_headless | apps/engene_headless | root | self | transitional wrapper | package execution replaces root bins | root still canonical, apps are transitional wrappers | ACTIVE |
| app_engene_bootstrap | apps/engene_bootstrap | root | self | transitional wrapper | package execution replaces root bins | root still canonical | ACTIVE |

## Feature Model Temporary Aliases

| Item | Current Location | Temp Owner | Target Owner | Why Temporary | Removal Condition | Blocker | Status |
|------|------------------|------------|--------------|---------------|-------------------|---------|--------|
| physics feature | Cargo.toml | root | cap_physics | mixed capability/role | feature rename to cap_* | risk of breaking users | PENDING |
| render feature | Cargo.toml | root | cap_render | mixed capability/role | feature rename to cap_* | risk of breaking users | PENDING |
| ai feature | Cargo.toml | root | cap_ai | mixed capability/role | feature rename to cap_* | risk of breaking users | PENDING |
| audio feature | Cargo.toml | root | cap_audio | mixed capability/role | feature rename to cap_* | risk of breaking users | PENDING |
| headless feature | Cargo.toml | root | role_headless | mixed capability/role | feature rename to role_* | risk of breaking users | PENDING |
| debug_ui feature | Cargo.toml | root | tool_debug_ui | mixed capability/role | feature rename to tool_* | risk of breaking users | PENDING |
| sdk_tools feature | Cargo.toml | root | role_sdk + tool_* | mixed capability/role | feature split to role_*/tool_* | risk of breaking users | PENDING |
| low_spec feature | Cargo.toml | root | profile_low_spec | mixed capability/role | feature rename to profile_* | risk of breaking users | PENDING |
| full feature | Cargo.toml | root | bundle alias | mixed capability/role | remove or document as bundle | risk of breaking users | PENDING |

## Legacy Test Surface

| Item | Current Location | Temp Owner | Target Owner | Why Temporary | Removal Condition | Blocker | Status |
|------|------------------|------------|--------------|---------------|-------------------|---------|--------|
| world_streaming tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | LEGACY |
| physics_body_combat tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | LEGACY |
| content_pipeline tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | LEGACY |
| persistence_full tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | LEGACY |
| runtime_systems tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | LEGACY |
| gameplay_and_ai tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | LEGACY |

## Temporary Disabled Features

| Item | Current Location | Temp Owner | Target Owner | Why Temporary | Removal Condition | Blocker | Status |
|------|------------------|------------|--------------|---------------|-------------------|---------|--------|
| physics module | crates/engine_physics/src/physics | engine_physics | engine_physics | 21 files depend on types (Ecs, Entity, WorldFields, Heightmap, SurfaceStateStore, SimulationLevel, MaterialId, etc.) that exist in root but NOT in crate structure | re-enable when engine_ecs exports Ecs/Entity and engine_world exports world types | needs engine_ecs Ecs/Entity + engine_world world types | TEMPORARY |
| model_loader module | crates/engine_content/src | engine_content | engine_graphics or engine_animation | code depends on animation/graphics types that should live in dedicated crates, not in engine_content | re-enable when engine_animation and engine_graphics crates exist with Skeleton, AnimationClip, SkinVertex types | needs dependent crates | TEMPORARY |

## Broken Crates (Critical Debt)

| Item | Current Location | Temp Owner | Target Owner | Why Temporary | Removal Condition | Blocker | Status |
|------|------------------|------------|--------------|---------------|-------------------|---------|--------|
| engine_physics crate | crates/engine_physics | FIXED | self | crate now compiles; physics/ module temporarily disabled (depends on Ecs, Entity, WorldFields from root) | re-enable physics/ when engine_ecs gets Ecs/Entity and engine_world gets world types | needs engine_ecs Ecs type + engine_world world types | TEMPORARY |
| engine_content crate | crates/engine_content | FIXED | self | crate now compiles; model_loader temporarily disabled (depends on animation/graphics from root) | re-enable model_loader when animation/graphics crates exist | needs engine_animation + engine_graphics crates | TEMPORARY |
| engine_world crate | crates/engine_world | unknown | unknown | not verified | verify or remove | unknown | UNKNOWN |
| engine_render crate | crates/engine_render | unknown | unknown | not verified | verify or remove | unknown | UNKNOWN |
| engine_audio crate | crates/engine_audio | unknown | unknown | not verified | verify or remove | unknown | UNKNOWN |
| engine_runtime crate | crates/engine_runtime | unknown | unknown | not verified | verify or remove | unknown | UNKNOWN |

---

## Rules

1. Any new temporary item must be added to this ledger immediately.
2. Items without removal condition are not allowed.
3. Status must be: ACTIVE, PENDING, LEGACY, ARCHIVED, or COMPLETE.
4. Blocker column must explain why item can't be removed now.
5. This file is the source of truth for migration debt.

## Exit Criteria

Migration is complete when:
- Root lib.rs exports only forwarding re-exports to crates
- All bins moved to apps/*
- All placeholder crates either filled or removed
- Feature model uses role_*/cap_*/tool_*/profile_* naming
- Legacy tests moved out of main test surface
- This ledger is empty (all items COMPLETE)
