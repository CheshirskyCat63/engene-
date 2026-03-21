# ENGINE-ONLY SCOPE DEFINITION

## ENGINE CORE PATH - CRITICAL

**Allowed in engine path:**
- `engine_core` - time, determinism, failure taxonomy, runtime config, quality/budget policy, registry/ownership contracts
- `engine_ecs` - entity lifecycle, component storage, command/application model, access control, system descriptor/ordering metadata  
- `engine_runtime` - scheduler, tick loop, fixed tick context, world tick orchestration, runtime phases (core only)
- `engine_world` - ONLY minimum truth: basic world data/types, spatial/state surfaces for core needs
- `engine_physics` - ONLY minimum boundary: pure simulation boundary, deterministic update contracts

## QUARANTINE LIST - FORBIDDEN IN ENGINE PATH

**Immediately quarantined:**
- `sdk_app` - SDK, requires window/event/render/editor boot
- `game_framework` - Game framework, app glue
- `engine_tools` - Tooling, editor-related
- `engine_render` - Render luxury
- `engine_audio` - Audio comfort
- `engine_content` - Content carnival
- `engine_startup` - App entrypoint glue
- `apps/*` - All applications (engene_game, engene_sdk, engene_headless, engene_bootstrap)
- `docs/testing vanity files` - Documentation vanity
- `editor-related tests` - Editor tests requiring UI/render

**Quarantine rules:**
1. No engine decisions influenced by quarantined crates
2. No architectural compromises for SDK/editor comfort
3. No render/UI dependencies in core path
4. No app boot requirements for engine compilation

## ARCHITECTURAL LAWS

**Law A - Core is tiny:**
engine_core cannot pull: app logic, editor logic, rendering logic, world/game orchestration, fake bootstrap junk

**Law B - Runtime is orchestration, not dump:**
engine_runtime owns: scheduler, tick, system execution contracts, phase ordering
engine_runtime does NOT own: game semantics, editor semantics, SDK behavior

**Law C - ECS is mechanics only:**
engine_ecs owns: entities, lifecycle, storage, access rules, command model
engine_ecs does NOT own: world gameplay meaning

**Law D - No root fantasy:**
No more `engene::...` as hidden miracle bag. No hidden root ownership.

**Law E - Every type has one home:**
If type lacks obvious owner crate → architecture bug.

## COMPILE SPINE ORDER

Critical compilation order:
1. `engine_core` 
2. `engine_ecs`
3. `engine_runtime` 
4. `engine_world` (minimum only)
5. `engine_physics` (boundary only)

**NO REVERSE ORDER ALLOWED**

## SUCCESS CRITERIA

Engine "alive" when:
- engine_core, engine_ecs, engine_runtime compile clean
- minimal engine-only execution path exists  
- deterministic replay/repeatable tick passes
- smoke/contract/integration/perf engine matrix green
- hot paths measured
- root/legacy/transitional imports don't dictate architecture
- game/sdk/tools not needed for engine path existence
