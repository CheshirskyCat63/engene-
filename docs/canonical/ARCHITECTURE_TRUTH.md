# Architecture Truth (FACTUAL - VERIFIED)

## Phase Execution Ownership

**Canonical owner**: `engine_runtime`

- Phase execution lives in `engine_runtime::phase`
- Active phases: tick, streaming, persistence, spatial, audio, editor, render
- game_framework uses engine_runtime::phase::* entrypoints
- **Stateful loop**: game_framework passes known_loaded_chunks to streaming

## World Truth/Data Ownership

**Canonical owner**: `engine_world` (DATA LAYER ONLY)

- **CANONICAL**: coords, chunk, world_state, terrain
- **REMOVED**: population, material_bridge, terrain_truth, spatial_index, hierarchical_spatial (physically deleted)
- **KEPT TEMP**: heightmap (test consumers), material_truth (engine_core consumers)
- These modules exist but are NOT public API - checking for hidden consumers

## Role/Composition Crates

| Crate | Role | Status |
|-------|------|--------|
| `game_framework` | Game composition | ✅ Compiles |
| `sdk_app` | Editor composition | ✅ Compiles |
| `engine_runtime` | Phase execution | ✅ Compiles (cleaned) |
| `engine_world` | World data | ✅ Compiles (data-only, cleaned) |
| `engine_core` | Core primitives | ✅ Compiles |
| `engine_ecs` | ECS | ✅ Compiles |
| `engine_startup` | Startup | ✅ Compiles |
| `engine_tools` | Tooling | ✅ Compiles |
| `engine_content` | Content | ✅ Compiles |
| `engine_render` | Render | ❌ 137 errors (existing deps issue) |
| `engine_audio` | Audio | ❌ 17 errors (existing) |

## Runtime Surface (CLEANED)

**engine_runtime canonical exports:**
- Phase execution contract (tick, streaming, persistence, spatial, audio, editor, render)
- PhaseContext, PhaseResult, PhaseTrait
- EngineRuntimeAssembly (minimal stub)
- **REMOVED**: async_services, job_graph, phase_runner, jobs/, streaming/ (physically deleted)
- **KEPT**: perf/ (test consumers), wiring/ (test consumers)

## World Surface (CLEANED)

**engine_world canonical exports:**
- coords, chunk, world_state, terrain
- **REMOVED**: population, material_bridge, terrain_truth, spatial_index, hierarchical_spatial
- **KEPT TEMP**: heightmap (test consumers), material_truth (engine_core consumers)

## Render Surface (NARROWED)

**engine_render canonical exports:**
- renderer, render_system, camera, mesh, lighting, pbr, shadow, shader_loader, sky
- **REMOVED**: decal_system, decals, model_loader (physically deleted)
- **KEPT**: terrain.rs, vegetation.rs (potentially used)
- **KEPT TEMP**: destruction_occlusion.rs, gore_mesh.rs (active consumers)

## Removed/Forbidden (VERIFIED)

- `*_enhanced` modules - removed
- `streaming_owner`, `chunk_persistence` - REMOVED (checking for consumers)
- `bootstrap/`, `simulation_core/`, `performance_law.rs`, `minimal_runtime_test.rs` - removed
- Render modules removed: art_direction, atmosphere, gore_mesh, etc. - 26 modules not exported

## Current Phase Loop (game_framework)

```
tick → streaming (with known_loaded_chunks) → persistence (with completed_loads/unloads)
```

---

**Last updated**: Engine core physical residue removed - 21 dead files deleted, component_registry.rs removed from engine_ecs
