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
| engene_game bin | Cargo.toml | root | apps/engene_game | migration shell | apps become canonical | apps are transitional shells | ACTIVE |
| engene_sdk bin | Cargo.toml | root | apps/engene_sdk | migration shell | apps become canonical | apps are transitional shells | ACTIVE |
| engene_headless bin | Cargo.toml | root | apps/engene_headless | migration shell | apps become canonical | apps are transitional shells | ACTIVE |
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
| app_engene_game | apps/engene_game | root | self | transitional wrapper | package execution replaces root bins | root still canonical | ACTIVE |
| app_engene_sdk | apps/engene_sdk | root | self | transitional wrapper | package execution replaces root bins | root still canonical | ACTIVE |
| app_engene_headless | apps/engene_headless | root | self | transitional wrapper | package execution replaces root bins | root still canonical | ACTIVE |
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
| world_streaming tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | QUARANTINE |
| physics_body_combat tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | QUARANTINE |
| content_pipeline tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | QUARANTINE |
| persistence_full tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | QUARANTINE |
| runtime_systems tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | QUARANTINE |
| gameplay_and_ai tests | tests_legacy/ | legacy | TBD | old integration tests | migrate or remove | API drift | QUARANTINE |

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
3. Status must be: ACTIVE, PENDING, QUARANTINE, or COMPLETE.
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
