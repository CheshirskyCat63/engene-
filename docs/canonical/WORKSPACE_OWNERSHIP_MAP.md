# WORKSPACE_OWNERSHIP_MAP

## Workspace members currently declared

### Engine / platform crates
- `crates/engine_core`
- `crates/engine_ecs`
- `crates/engine_world`
- `crates/engine_runtime`
- `crates/engine_render`
- `crates/engine_physics`
- `crates/engine_audio`
- `crates/engine_content`
- `crates/engine_tools`

### App / product-side crates
- `crates/sdk_app`
- `crates/game_framework`

### App shells
- `apps/engene_game`
- `apps/engene_sdk`
- `apps/engene_headless`

### Migration shell
- root package `.` (`engene`)

## Ownership intent

| Area | Intended owner |
|---|---|
| core contracts / events / scheduling | `engine_core`, `engine_runtime` |
| ECS and entity lifecycle | `engine_ecs` |
| world truth / spatial / persistence / streaming | `engine_world` |
| rendering implementation | `engine_render` |
| physics runtime | `engine_physics` |
| audio runtime | `engine_audio` |
| content pipeline / assets | `engine_content` |
| maintenance tooling / doctor | `engine_tools` |
| SDK/editor experience | `sdk_app` |
| game runtime composition | `game_framework` |
| end-user app shells | `apps/*` |

## Current mismatch to keep visible

The root package still hosts active execution and broad exports.
That means the ownership map is **declared**, but not yet fully enforced.

## Rule

If a module or responsibility has no obvious owner in this table,
that is a design bug, not a reason to dump it into root.
