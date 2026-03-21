# Architecture Truth

## Phase Execution Ownership

**Canonical owner**: `engine_runtime`

- Phase execution lives in `engine_runtime::phase`
- Current active phases: tick, streaming, persistence, spatial, audio, editor, render
- All phase entrypoints are public APIs in `engine_runtime`

## World Truth/Data Ownership

**Canonical owner**: `engine_world`

- World data lives in `engine_world`
- Core modules: coords, chunk, world_state, terrain
- Note: engine_world has compile errors (132+) - needs separate fix

## Role/Composition Crates

| Crate | Role |
|-------|------|
| `game_framework` | Game composition, launches runtime |
| `sdk_app` | Editor composition, launches runtime |
| `engine_runtime` | Phase execution contract |
| `engine_world` | World data/truth |
| `engine_core` | Core primitives |
| `engine_ecs` | Entity component system |

## Forbidden Patterns

- `*_enhanced` modules - removed (no consumers)
- Direct phase orchestration in game_framework - use engine_runtime
- engine_world depending on runtime concerns - forbidden
- engine_runtime::phase modules depending on engine_world when broken - temporary

## Current Status

- ✅ tick, streaming, persistence phases extracted to engine_runtime
- ✅ game_framework uses engine_runtime::phase::* entrypoints
- ⚠️ engine_world has 132+ compile errors (separate fix needed)
- ⚠️ engine_render, engine_audio have existing issues

---

**Last updated**: Phase extraction stabilization
