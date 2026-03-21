# ENGENE Workspace Status

**Current HEAD:** engene-2.0-transition  
**Last Updated:** 2026-03-21 (role cleanup completed)

## Stable Crates (✅ Compile Clean)

| Crate | Status | Notes |
|-------|--------|-------|
| engine_core | ✅ Compiles | Core primitives, determinism, failure taxonomy |
| engine_ecs | ✅ Compiles | Entity lifecycle, component storage, system contracts |
| engine_runtime | ✅ Compiles | Phase execution spine (clean role) |
| engine_world | ✅ Compiles | World data contracts (data-only) |
| engine_startup | ✅ Compiles | Startup contracts, panic hooks |
| game_framework | ✅ Compiles | Game composition, headless runtime |

## Transitional Crates (⚠️ Need Work)

| Crate | Status | Issues |
|-------|--------|--------|
| sdk_app | ⚠️ Compiles | Legacy imports, needs migration |
| engine_tools | ⚠️ Compiles | Tooling utilities, stable enough |

## Broken Crates (❌ Dependency Issues)

| Crate | Status | Error Count | Root Cause |
|-------|--------|------------|------------|
| engine_render | ❌ Broken | 137 errors | Missing egui, pollster, image dependencies |
| engine_audio | ❌ Broken | 17 errors | Missing internal modules, legacy imports |

## Canonical Core (Эталонное ядро)

**Phase Execution Owner:** `engine_runtime`
- Phase execution contract (tick, streaming, persistence, spatial, audio, editor, render)
- Clean separation: phase spine in lib.rs, assembly in internal module

**World Data Owner:** `engine_world`  
- Data-only layer: coords, chunk, world_state, terrain
- Removed: population, material_bridge, terrain_truth, spatial_index, hierarchical_spatial
- Kept temp: heightmap (test consumers), material_truth (engine_core consumers)

**Composition Owner:** `game_framework`
- Uses engine_runtime::assembly for runtime state
- Uses engine_runtime::phase for phase execution
- Headless game loop verified

## Quarantined Zones

**engine_runtime:**
- perf/ (test consumers)
- wiring/ (test consumers)
- assembly/ (internal bootstrap glue)

**engine_world:**
- heightmap.rs (test consumers)
- material_truth.rs (engine_core consumers)
- events/, fields.rs, resources.rs (support systems)

**engine_render:**
- terrain.rs, vegetation.rs (potentially used)
- destruction_occlusion.rs, gore_mesh.rs (active consumers)

## Next Cleanup Targets

1. **engine_render fix campaign** - Resolve dependency issues (egui, pollster, image)
2. **engine_audio fix campaign** - Resolve missing internal modules  
3. **sdk_app migration** - Remove legacy imports, migrate to engine_* crates
4. **CI normalization** - Create core/extended/broken lanes

## Architecture Laws Enforced

✅ **Law A - Core is tiny:** engine_core has no app/editor/render logic  
✅ **Law B - Runtime is orchestration:** engine_runtime owns scheduler/tick, not game semantics  
✅ **Law C - ECS is mechanics only:** engine_ecs owns entities/lifecycle, not world meaning  
✅ **Law D - No root fantasy:** No hidden engene:: root ownership  
✅ **Law E - Every type has one home:** Clear crate ownership established

## Compilation Summary

```
cargo check -p engine_core -p engine_ecs -p engine_runtime -p engine_world -p engine_startup -p game_framework
# ✅ SUCCESS - All core crates compile clean

cargo check -p engine_render -p engine_audio  
# ❌ FAILED - Dependency issues (separate fix campaign needed)
```

---

**Status:** Core engine spine is stable and role-clean. Render/audio need separate dependency fix campaign before reintegration.
