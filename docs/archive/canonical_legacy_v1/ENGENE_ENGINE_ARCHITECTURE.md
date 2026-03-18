# ENGENE_ENGINE_ARCHITECTURE

Status: IMPLEMENTED

## Engine structure
Engine runtime is exposed through `src/engine.rs` and implemented in reusable subsystems:
- `src/core`
- `src/world`
- `src/memory`
- `src/physics`
- `src/audio`
- `src/graphics`
- `src/navigation`
- `src/content`
- `src/simulation` (time events + generic simulation infra)
- `src/animation` (engine-generic animation runtime)
- `src/input`, `src/network`, `src/body`

## Runtime flow
- `Engine::tick*` advances time, emits `simulation::time_events`, runs scheduled systems, then render phase.
- Game-specific world tick and animation integration are now game-owned modules wired by game runtime assembly.

## Performance hardening in this pass
- warning/noise cleanup on core build targets
- reduced debug noise from unused imports/variables
- removed dead fields in core runtime structures
