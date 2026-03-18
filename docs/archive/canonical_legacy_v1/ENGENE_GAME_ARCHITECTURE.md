# ENGENE_GAME_ARCHITECTURE

Status: IMPLEMENTED

Game ownership center: `src/game/*`
- AI/economy/ecosystem/gameplay rules
- runtime integration systems
- game runtime assembly

## Runtime-specific modules moved in this closure
- `src/game/runtime_world_tick.rs`
- `src/game/runtime_background.rs`
- `src/game/animation_integration.rs`

These own game semantics while consuming engine services.
