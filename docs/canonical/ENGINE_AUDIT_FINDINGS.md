# ENGINE-ONLY RECOVERY AUDIT

## FROZEN NON-CORE FINDINGS

**Non-engine crates (DO NOT TOUCH during recovery):**
- sdk_app - Still uses engene::* imports, lives in old architecture
- game_framework - Still pulls engene::runtime, engene::world, pub use engene::app
- engine_tools - Tooling, not core path
- apps/* - All applications, not engine path
- docs/vanity - Documentation vanity files

**Status: FROZEN until engine path is stable**

## ENGINE-ONLY TARGET SURFACE

**Core engine path (REAL TARGET):**
- engine_core - TO BE TRIMMED to tiny core
- engine_ecs - CLOSEST TO REAL CORE (keep as-is)
- engine_runtime - TO BE VERIFIED for real spine
- engine_world - TO BE AUDITED (stub vs truth)

**Temporarily excluded:**
- engine_render - Render luxury
- engine_audio - Audio comfort
- engine_physics - TO BE DECIDED (boundary vs core)
- engine_content - Content carnival

## AUDIT FINDINGS

### engine_ecs - MOST REAL
✅ **STATUS: CLOSEST TO REAL CORE**
- Normal crate surface: access, commands, ecs_mechanics, entity, parallel_validation
- Looks like real ECS mechanics, not decoration
- **ACTION: KEEP AS-IS**

### engine_core - OVERLOADED  
⚠️ **STATUS: TOO FAT**
- Current modules: build_manifest, game_config, integration_matrix, metrics_registry, mutation_policy, ownership_map, plugin, profiler, quality_governor, registry, runtime_config, runtime_manifest, serialization
- Should be: time, determinism, failure taxonomy, tiny config contract
- **ACTION: TRIM TO TINY CORE**

### engine_world - STUB THEATER
❌ **STATUS: HONEST STUB, NOT TRUTH**
- Current: Cell with position/biome_id, GRID_SIZE, WorldGrid as Vec<Cell>, generate() fills 10x10 with biome 0
- This is compile scaffold, not semantic world truth
- **ACTION: DECIDE: honest stub vs minimum truth**

### engine_runtime - TO BE VERIFIED
❓ **STATUS: NEEDS VERIFICATION**
- Has simulation_core and re-exports
- Cargo.toml pulls: engine_core, engine_ecs, engine_world, rapier3d
- Question: Is rapier3d needed for minimal core path?
- **ACTION: VERIFY REAL EXECUTION SPINE**

## CRITICAL ISSUES

### 1. engene:: imports still alive
- game_framework still: `use engene::runtime::bootstrap`, `use engene::world::heightmap`, `pub use engene::app::game_runner::GameApp`
- sdk_app still: massive engene::* imports for spatial_dirty_journal, engine, graphics, input, memory, runtime, tools, world
- **STATUS: ARCHITECTURALLY HANGING IN AIR**

### 2. Docs truth lagging behind
- CURRENT_BRANCH_STATE.md claims root package still active, root migration shell alive
- But root package already removed
- **STATUS: DOCS TRUTH STALE**

### 3. Fake compile fixes suspected
- plugin, registry, runtime_manifest, game_config may be stub additions
- **ACTION: AUDIT FOR FAKE TYPES**

## NEXT STEPS (HARDCODED ORDER)

1. **FREEZE** - Mark non-engine crates as out of current recovery scope
2. **DEFINE** - Fix real engine-only target surface  
3. **TRIM** - Cut engine_core to tiny core
4. **AUDIT** - Decide engine_world: stub vs truth
5. **VERIFY** - Confirm engine_runtime real spine
6. **CUT** - Remove engene:: imports from engine path
7. **TEST** - Build engine-only test matrix
8. **KILL** - Remove fake compile fixes

## CURRENT HONEST STATUS

**NOT Phase 3, but Phase 1.5/2:**
- Compile spine partially restored
- Part of spine held by placeholder world  
- External crates still live in old architecture universe
- Engine not "proven", only partially brought to runnable state

**REALITY CHECK:**
- ECS → closest to real core ✅
- Core → too fat ❌  
- World → stub theater ❌
- Runtime → needs verification ❓
- Consumers → old architecture ❌
