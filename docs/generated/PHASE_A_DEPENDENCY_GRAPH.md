# Phase A/Phase B Dependency Graph Snapshot (Batch 8 closure + Phase B Batch 15)

## Workspace members
- root package `engene`
- crates: `engine_core`, `engine_ecs`, `engine_world`, `engine_runtime`, `engine_render`, `engine_physics`, `engine_audio`, `engine_content`, `engine_tools`, `sdk_app`, `game_framework`
- apps: `app_engene_game`, `app_engene_sdk`, `app_engene_headless`

## Current dependency edges (declared in Cargo.toml)
- `engene` -> `engine_core`
- `engene` -> `engine_content`
- `engene` -> `engine_ecs`
- `engene` -> `engine_runtime`
- `engine_ecs` -> `engine_core`
- `engine_world` -> `engine_core`
- `engine_runtime` -> `engine_core`
- `engine_render` -> `engine_core`
- `engine_physics` -> `engine_core`
- `engine_audio` -> `engine_core`
- `engine_content` -> `engine_core`
- `engine_tools` -> `engine_core`
- `sdk_app` -> `engine_core`
- `game_framework` -> `engine_core`
- `app_engene_game` -> `engene`
- `app_engene_sdk` -> `engene`
- `app_engene_headless` -> `engene`

## Reverse import gate
- Engine->game reverse dependency gate: PASS.
- ECS direct storage gate: PASS.

## Notes
Phase B Batch 15 continues narrow transition-core bench coordination optimization under unchanged benchmark semantics/thresholds and severity-tagged runtime gate output, with no crate-direction law changes and no reverse dependencies added.
- Transition-core x8 scaling proof is now environment-guarded (requires >=8 logical CPUs) before scaling ratio evaluation.
