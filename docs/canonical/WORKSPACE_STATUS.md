# ENGENE Workspace Status

**Current HEAD:** engene-2.0-transition  
**Last Updated:** 2026-03-21 (engine_core physical residue removed - 21 dead files deleted)

## Stable Crates (✅ Compile Clean)

| Crate | Status | Notes |
|-------|--------|-------|
| engine_core | ✅ Compiles | Tiny core - only 5 modules, no external deps, physical residue removed |
| engine_ecs | ✅ Compiles | Entity lifecycle, component storage, system contracts (component_registry.rs removed) |
| engine_runtime | ✅ Compiles | Phase execution spine (clean role, assembly separated) |
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
- Role cleanup completed: no assembly logic inline in public API
- **REMOVED**: async_services, job_graph, phase_runner, jobs/, streaming/ (physically deleted)
- **KEPT**: perf/ (test consumers), wiring/ (test consumers)

**World Data Owner:** `engine_world`  
- Data-only layer: coords, chunk, world_state, terrain
- **REMOVED**: population, material_bridge, terrain_truth, spatial_index, hierarchical_spatial (physically deleted)
- **KEPT TEMP**: heightmap (test consumers), material_truth (engine_core consumers)

**Composition Owner:** `game_framework`
- Uses engine_runtime::assembly for runtime state
- Uses engine_runtime::phase for phase execution
- Headless game loop verified with stateful streaming loop

**Core Primitives Owner:** `engine_core`
- Tiny core: only 5 modules (data_policy, determinism_policy, deterministic_merge, failure_taxonomy, time)
- Zero external dependencies - uses only std
- All removed modules physically deleted - no quarantine terminology

## Module Status Summary

**engine_runtime:**
- **KEPT**: perf/ (test consumers)
- **KEPT**: wiring/ (test consumers)  
- **KEPT**: assembly/ (internal bootstrap glue)

**engine_world:**
- **KEPT**: heightmap.rs (test consumers)
- **KEPT**: material_truth.rs (engine_core consumers)
- **KEPT**: events/, fields.rs, resources.rs (support systems)

**engine_render:**
- **REMOVED**: decal_system, decals, model_loader (physically deleted)
- **KEPT**: terrain.rs, vegetation.rs (potentially used)
- **KEPT TEMP**: destruction_occlusion.rs, gore_mesh.rs (active consumers)

## Next Cleanup Targets

1. **engine_render fix campaign** - Resolve dependency issues (egui, pollster, image)
2. **engine_audio fix campaign** - Resolve missing internal modules  
3. **sdk_app migration** - Remove legacy imports, migrate to engine_* crates
4. **CI normalization** - Create core/extended/broken lanes

## Core Readiness Achieved ✅

**All core requirements met:**
- ✅ engine_core: Tiny core with 5 modules, zero external dependencies
- ✅ engine_ecs: Mechanics-only, unchanged
- ✅ engine_runtime: Clean phase spine + separated assembly
- ✅ game_framework: Uses assembly path, stateful streaming loop
- ✅ All core crates compile and test successfully
- ✅ Documentation synchronized with code reality

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
