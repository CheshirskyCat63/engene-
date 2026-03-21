# GAP_MAP_CURRENT_TO_GOD_TIER

**Status**: Transition blocker analysis
**Purpose**: Identify what stands between CURRENT_RUNTIME_TRUTH and ENGENE_2_0_GOD_TIER_ARCHITECTURE

---

## 1. Gap: Entrypoint Ownership

| Current state | Target state | Blocker |
|---|---|---|
| Root bins still active: `engene_game`, `engene_sdk`, `engene_headless`, `engene_tools` | `apps/*` as primary launch surface, root bins deprecated | Role crates depend on root for bootstrap types |

### Blocker details
- `game_framework` depends on `engene::runtime::bootstrap::*`
- `sdk_app` depends on `engene::app::game_runner::GameApp`
- No direct crate-to-crate ownership yet

### Required work
1. Move bootstrap types to `engine_runtime`
2. Move `GameApp` to appropriate role crate
3. Update entrypoint docs to use `apps/*`
4. Deprecate root bins with removal timeline

---

## 2. Gap: Root Crate Still Holds Active Binaries

| Current state | Target state | Blocker |
|---|---|---|
| Root package hosts active bins | Root is thin shell OR removed | Operator commands still point to root bins |

### Required work
1. Update CI to run `apps/*` instead of root bins
2. Update documentation to show `apps/*` as canonical
3. Add deprecation warnings to root bin paths
4. Remove root bin references from README_FIRST_RUN.md

---

## 3. Gap: Role Crates Still Depend on Root

| Current state | Target state | Blocker |
|---|---|---|
| `game_framework` imports: `engene::core::*`, `engene::runtime::*`, `engene::world::*` | Zero root dependencies | Bootstrap types live in root |

### Dependencies to move

```
game_framework -> root (9 imports):
- engene::core::build_manifest::BuildManifest    -> engine_core
- engene::core::crash_telemetry                   -> engine_core
- engene::runtime::bootstrap::*                   -> engine_runtime
- engene::world::heightmap::Heightmap             -> engine_world
- engene::world::world::WorldGrid                 -> engine_world
- engene::app::game_runner::GameApp               -> game_framework (own it)
- engene::app::spatial_dirty_journal::SpatialDirtyJournal -> engine_world
- engene::world::components::*                    -> engine_world
- engene::world::hierarchical_spatial::SpatialUpdatePath -> engine_world

sdk_app -> root (same dependencies, plus SDK-specific):
- All above, plus editor-specific types
```

### Required work
1. Create `engine_runtime::bootstrap` module
2. Move `BuildManifest`, `crash_telemetry` to `engine_core`
3. Move `Heightmap`, `WorldGrid`, components to `engine_world`
4. Move `SpatialDirtyJournal`, `SpatialUpdatePath` to `engine_world`
5. Own `GameApp` inside `game_framework` crate

---

## 4. Gap: sdk_runner.rs World-Kitchen

| Current state | Target state | Blocker |
|---|---|---|
| `src/app/sdk_runner.rs` performs multiple responsibilities in redraw: camera/input, simulation, asset polling, world streaming, chunk persistence, spatial rebuild, audio, dashboard, inspector, render | Phase driver with explicit phase methods | Phase order not yet enforced at code level |

### Current responsibilities in sdk_runner.rs redraw:
1. camera/input update
2. simulation stepping
3. asset polling
4. world streaming
5. chunk persistence handoff
6. full spatial rebuild
7. audio listener update
8. dashboard updates
9. inspector mutation
10. render preparation
11. render error handling

### Required work
1. Extract each responsibility into explicit phase methods
2. Enforce phase order at compile-time
3. Add phase order invariant tests
4. Replace full spatial rebuild with dirty-input incremental path
5. Add fallback observability

---

## 5. Gap: integration.rs Catch-All

| Current state | Target state | Blocker |
|---|---|---|
| `src/runtime/wiring/integration.rs` mixes multiple boundary domains | Boundary modules + re-export hub | Domains not yet separated |

### Current mixed domains:
- ballistics / damage
- destruction / terrain
- nav / occlusion
- gore
- AI wiring
- animation wiring

### Required work
1. Create `engine_runtime::wiring::ballistics` module
2. Create `engine_runtime::wiring::destruction` module
3. Create `engine_runtime::wiring::navigation` module
4. Create `engine_runtime::wiring::gore` module
5. Create `engine_runtime::wiring::ai` module
6. Create `engine_runtime::wiring::animation` module
7. Make `integration.rs` thin re-export hub

---

## 6. Gap: Feature Model Still Role-Blurry

| Current state | Target state | Blocker |
|---|---|---|
| Features mix: capability (`physics`, `render`), runtime mode (`headless`), tooling (`debug_ui`), quality (`low_spec`), bundle (`full`) | Role-first taxonomy: role / capability / tooling / profile | Cargo features not yet refactored |

### Required work
1. Rename features to role-first schema:
   - `role_game`, `role_sdk`, `role_headless`, `role_tools`
   - `cap_render`, `cap_physics`, `cap_ai`, `cap_audio`, `cap_networking`
   - `tool_debug_ui`, `tool_editor_inspection`, `tool_doctor`
   - `profile_low_spec`, `profile_ci`, `profile_release`
2. Update docs to use new feature taxonomy
3. Remove old ambiguous feature names

---

## 7. Gap: CI Not Covering Transition Branch

| Current state | Target state | Blocker |
|---|---|---|
| `.github/workflows/rust.yml` watches only `main` | CI covers transition branch on push/PR | Workflow not branch-aware |

### Required work
1. Add transition branch to CI triggers
2. Run smoke/contract/certification lanes on transition branch
3. Add branch-aware gating

---

## 8. Gap: Spatial Full-Rebuild Without Observability

| Current state | Target state | Blocker |
|---|---|---|
| Editor redraw triggers unconditional full spatial rebuild | Dirty-input incremental path + explicit fallback with telemetry | Equivalence tests not written |

### Required work
1. Implement explicit dirty input model
2. Add incremental update path
3. Add fallback observability (telemetry)
4. Write equivalence tests: `incremental(world, dirty) == rebuild(world_after)`
5. Add chunk load/unload invalidation tests

---

## 9. Gap: No Compile-Time Phase Order Enforcement

| Current state | Target state | Blocker |
|---|---|---|
| Phase order documented in PHASE_ORDER_CONTRACT.md | Compile-time or test-enforced phase order | No invariant tests yet |

### Required work
1. Write phase order invariant test
2. Write editor mutation propagation test
3. Write audio independent-of-render test
4. Write spatial equivalence test

---

## 10. Gap: Root Still Exports Broad Monolith Tree

| Current state | Target state | Blocker |
|---|---|---|
| `src/lib.rs` re-exports: animation, app, audio, body, content, core, engine, game, graphics, input, memory, navigation, network, physics, runtime, simulation, testsupport, tools, world | Thin re-export shell OR removed | Role crates still need compatibility |

### Required work
1. Track which exports are actually used
2. Remove unused re-exports
3. Replace with direct crate dependencies
4. Reduce to compatibility-only forwarding

---

## Gap Summary Table

| Gap | Severity | Effort | Blocker type |
|-----|----------|--------|--------------|
| Role crate root dependencies | 🔴 Critical | Medium | Code |
| Root active binaries | 🔴 Critical | Low | Docs/CI |
| sdk_runner.rs world-kitchen | 🔴 Critical | High | Code |
| integration.rs catch-all | 🟡 Medium | Medium | Code |
| Feature model role-blurry | 🟡 Medium | Medium | Cargo |
| CI not covering transition | 🟡 Medium | Low | CI |
| Spatial full-rebuild | 🟡 Medium | Medium | Code |
| Phase order not enforced | 🟡 Medium | Low | Tests |
| Root broad exports | 🟢 Low | Medium | Code |

---

## Exit Criteria

This document becomes obsolete when ALL gaps are resolved:
- [ ] Zero root dependencies in role crates
- [ ] `apps/*` as primary launch surface
- [ ] Root bins deprecated
- [ ] `sdk_runner.rs` replaced with phase methods
- [ ] `integration.rs` split into boundary modules
- [ ] Feature taxonomy cleaned
- [ ] CI covers transition branch
- [ ] Spatial dirty model implemented
- [ ] Phase order tests written
- [ ] Root exports reduced

---

## Relationship to Other Documents

- **CURRENT_RUNTIME_TRUTH.md** — where we are now
- **GAP_MAP_CURRENT_TO_GOD_TIER.md** — what blocks us (this file)
- **ENGENE_2_0_GOD_TIER_ARCHITECTURE.md** — where we're going
