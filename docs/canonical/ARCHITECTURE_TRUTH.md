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
- **QUARANTINE** (not exported): chunk_persistence, streaming_owner, ai_*, authored_*, etc.
- These modules exist but are NOT public API - checking for hidden consumers

## Role/Composition Crates

| Crate | Role | Status |
|-------|------|--------|
| `game_framework` | Game composition | ✅ Compiles |
| `sdk_app` | Editor composition | ✅ Compiles |
| `engine_runtime` | Phase execution | ✅ Compiles |
| `engine_world` | World data | ✅ Compiles (data-only) |
| `engine_core` | Core primitives | ✅ Compiles |
| `engine_ecs` | ECS | ✅ Compiles |
| `engine_startup` | Startup | ✅ Compiles |
| `engine_tools` | Tooling | ✅ Compiles |
| `engine_content` | Content | ✅ Compiles |
| `engine_render` | Render | ❌ 137 errors (existing) |
| `engine_audio` | Audio | ❌ 17 errors (existing) |

## Removed/Forbidden (VERIFIED)

- `*_enhanced` modules - removed
- `streaming_owner`, `chunk_persistence` - QUARANTINE (checking for consumers)
- `bootstrap/`, `simulation_core/`, `performance_law.rs`, `minimal_runtime_test.rs` - removed
- Render modules in quarantine: art_direction, atmosphere, gore_mesh, etc. - 26 modules not exported

## Current Phase Loop (game_framework)

```
tick → streaming (with known_loaded_chunks) → persistence (with completed_loads/unloads)
```

---

**Last updated**: After narrow public surface cleanup - world/data/runtime spine verified
