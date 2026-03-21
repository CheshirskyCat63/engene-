# OWNERSHIP_HANDOFF_ORDER

**Status**: Law
**Purpose**: Define the exact order and scope of ownership handoff from root to role crates.

---

## Handoff Order

```
Step 1: sdk_app ownership
Step 2: game_framework ownership  
Step 3: engine_runtime ownership
Step 4: engine_world ownership
Step 5: root cleanup
Step 6: re-export cleanup
Step 7: removal pass
```

---

## Step 1: SDK Ownership Handoff

### Current State
- `sdk_app` is a transitional shell
- Real SDK orchestration lives in `src/app/sdk_runner.rs`
- Editor shell, inspectors, dashboards depend on root

### Target State
- `sdk_app` owns all editor/runtime composition
- `sdk_runner.rs` decomposed into phase methods
- Zero root dependencies in sdk_app

### Files/Modules to Move

| Current Location | Target Location | Type |
|------------------|-----------------|------|
| `src/app/sdk_runner.rs` orchestration | `crates/sdk_app/src/` | Rewrite |
| Editor shell modules | `crates/sdk_app/src/editor/` | Move |
| Inspector surfaces | `crates/sdk_app/src/inspector/` | Move |
| Dashboard logic | `crates/sdk_app/src/dashboard/` | Move |

### Tests to Update
- `tests/sdk_editor_gui.rs` - verify SDK ownership
- `tests/runtime_phase_contracts.rs` - rewrite to invariant test

### Forbidden Regressions
- SDK may NOT own simulation truth
- SDK may NOT own world truth
- SDK may NOT own game rules

### Exit Criteria
- [ ] `sdk_app` has all editor tooling code
- [ ] `sdk_runner.rs` removed or became thin wrapper
- [ ] Zero root imports in sdk_app crates
- [ ] Tests pass with new ownership

---

## Step 2: Game Framework Ownership

### Current State
- `game_framework` is transitional shell
- Game bootstrap lives in root
- Game composition depends on root for types

### Target State
- `game_framework` owns playable runtime
- Game bootstrap fully in role crate
- Zero root dependencies

### Files/Modules to Move

| Current Location | Target Location | Type |
|------------------|-----------------|------|
| Game bootstrap types | `crates/game_framework/src/bootstrap/` | Move |
| Game composition logic | `crates/game_framework/src/composition/` | Move |
| Player/game entity types | `crates/game_framework/src/entities/` | Move |

### Tests to Update
- `tests/vertical_slice.rs` - verify game ownership
- `tests/e2e_regression.rs` - player lifecycle tests

### Forbidden Regressions
- Game may NOT own editor-only logic
- Game may NOT own SDK tooling

### Exit Criteria
- [ ] `game_framework` owns all game runtime
- [ ] Zero root dependencies
- [ ] Tests pass

---

## Step 3: Engine Runtime Ownership

### Current State
- `engine_runtime` exists but shares space with product orchestration
- Phase order documented but not enforced at code level

### Target State
- `engine_runtime` owns ONLY:
  - Phase graph
  - Execution sequencing
  - Tick/runtime laws
  - Scheduler contract surface

### Files/Modules to Retain

| Module | Owner | Status |
|--------|-------|--------|
| Phase definitions | engine_runtime | Keep |
| Tick contract | engine_runtime | Keep |
| Scheduler surface | engine_runtime | Keep |
| Execution graph | engine_runtime | Keep |

### Files/Modules to Remove from engine_runtime

| Module | Move To | Reason |
|--------|---------|--------|
| Editor-specific phases | sdk_app | Not runtime law |
| Game bootstrap | game_framework | Not runtime law |
| Editor UX rendering | sdk_app | Not runtime law |

### Exit Criteria
- [ ] engine_runtime has zero game/SDK specific code
- [ ] Phase contract is compile-time or test-enforced
- [ ] Tests verify phase order

---

## Step 4: Engine World Ownership

### Current State
- `engine_world` owns spatial, persistence, streaming
- But mutation boundaries not strictly enforced

### Target State
- `engine_world` is THE owner of:
  - Spatial truth
  - Streaming truth
  - Persistence truth
  - Dirty-input sources
  - Residency/state truth

### Ownership Rules

| Data Type | Owner | Access Pattern |
|-----------|-------|----------------|
| World truth | engine_world | Mutable only via world contract |
| Spatial index | engine_world | Read-only for render/audio |
| Chunk state | engine_world | Streaming contract |
| Transform state | engine_world | Mutation via commands |

### Contracts to Enforce

```
render -> engine_world: READ-ONLY spatial queries
audio -> engine_world: READ-ONLY spatial queries  
sdk_app -> engine_world: READ-ONLY + dirty input
game_framework -> engine_world: READ-ONLY + commands
```

### Exit Criteria
- [ ] All mutations go through world boundaries
- [ ] Render only reads spatial data
- [ ] Audio only reads spatial data
- [ ] Tests verify ownership

---

## Step 5: Root Cleanup

### Current State
- Root holds active bins
- Root re-exports broad module tree
- Root is "hidden brain" of engine

### Target State
- Root is thin compatibility shell OR removed
- Zero new architecture born in root

### Files to Deprecate/Remove

| File/Module | Action | Condition |
|-------------|--------|-----------|
| Root bins | Deprecate | After apps/* become canonical |
| Broad re-exports | Remove | After crates depend directly |
| Legacy modules | Archive | After handoff complete |

### Forbidden Regressions
- NO new domain ownership in root
- NO new orchestration in root
- NO new catch-all exports

### Exit Criteria
- [ ] Root has zero active runtime code
- [ ] Root is only compatibility layer
- [ ] All bins moved to apps/*

---

## Step 6: Re-export Cleanup

### Current State
- `src/lib.rs` re-exports everything
- Crates depend on root re-exports instead of each other

### Target State
- Direct crate-to-crate dependencies
- No root re-exports needed

### Required Changes

| Old Dependency | New Dependency |
|----------------|----------------|
| `engene::world::Heightmap` | `engine_world::Heightmap` |
| `engene::core::RuntimeConfig` | `engine_core::RuntimeConfig` |
| `engene::runtime::bootstrap::*` | `engine_runtime::bootstrap::*` |

### Exit Criteria
- [ ] All crates import directly
- [ ] Root re-exports removed
- [ ] No circular dependencies

---

## Step 7: Removal Pass

### Current State
- Transition scaffolds still exist
- Migration tests protect old code

### Target State
- All temporary code removed
- Migration tests deleted or rewritten

### Items to Remove

| Item | Removal Condition |
|------|-------------------|
| `sdk_runner.rs` old orchestration | After phase methods exist |
| Legacy root modules | After direct imports work |
| Migration tests | After handoff verified |
| Old entrypoint docs | After apps/* canonical |

### Exit Criteria
- [ ] Zero transition scaffolds
- [ ] All migration tests rewritten or deleted
- [ ] Clean build without legacy code

---

## First Slice: sdk_runner.rs Decomposition

**Priority**: Highest - this is the main symptom of "hidden brain"

### Current Problem
`src/app/sdk_runner.rs` contains:
- camera/input update
- simulation stepping
- asset polling
- world streaming
- chunk persistence
- full spatial rebuild
- audio listener update
- dashboard updates
- inspector mutation
- render preparation
- render error handling

### Target Structure

```
src/app/sdk_runner.rs (THIN WRAPPER)
    └── calls phase methods

crates/sdk_app/src/
    ├── editor/
    │   ├── dashboard.rs
    │   ├── inspector.rs
    │   └── overlays.rs
    └── composition.rs

crates/engine_runtime/src/
    ├── phase/
    │   ├── tick.rs
    │   ├── streaming.rs
    │   ├── persistence.rs
    │   ├── spatial.rs
    │   ├── audio.rs
    │   └── render.rs
    └── scheduler.rs

crates/engine_world/src/
    ├── spatial/
    │   └── dirty_input.rs
    ├── streaming/
    └── persistence/
```

### Slice 1 Tasks

1. [ ] Extract phase methods to `engine_runtime::phase::*`
2. [ ] Move editor-specific to `sdk_app::editor::*`
3. [ ] Move spatial to `engine_world::spatial`
4. [ ] Rewrite `runtime_phase_contracts.rs` as invariant test
5. [ ] Verify zero root dependencies after move

---

## Relationship to Other Documents

- **CURRENT_RUNTIME_TRUTH.md** — current state
- **GAP_MAP_CURRENT_TO_GOD_TIER.md** — blockers
- **ENGENE_2_0_GOD_TIER_ARCHITECTURE.md** — target
- **TEST_CLASSIFICATION_MATRIX.md** — test classification
- **OWNERSHIP_HANDOFF_ORDER.md** — this file
