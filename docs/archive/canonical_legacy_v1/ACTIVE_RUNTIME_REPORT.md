# ENGENE Active Runtime Report

Generated: 2026-03-10

---

## 1. Registered Runtime Systems

All systems below are registered via `RuntimeAssembly::vertical_slice()` in `src/app/runtime_assembly.rs`.

### Core Simulation Systems

| # | System Name | Module Path | Tick Phase | Frequency | Threading | Status |
|---|-------------|-------------|------------|-----------|-----------|--------|
| 1 | SimulationSystem | `src/simulation/simulation.rs` | fixed_tick | Every frame (20Hz sim) | Single | **ACTIVE** |
| 2 | WorldTickSystem | `src/simulation/world_tick.rs` | fixed_tick | Every frame | Single | **ACTIVE** |
| 3 | AiSystem | `src/ai/ai.rs` | fixed_tick | Every frame | Single | **ACTIVE** |
| 4 | PhysicsSystem | `src/physics/physics.rs` | fixed_tick | Every frame | Single | **ACTIVE** |
| 5 | BodySystem | `src/body/body_system.rs` | fixed_tick | Every frame | Single | **ACTIVE** |

### Damage & Destruction Pipeline

| # | System Name | Module Path | Tick Phase | Frequency | Threading | Status |
|---|-------------|-------------|------------|-----------|-----------|--------|
| 6 | BallisticsTickSystem | `src/core/integration_systems.rs` | fixed_tick | Every frame | Single | **ACTIVE** |
| 7 | DamageDispatchSystem | `src/core/integration_systems.rs` | fixed_tick | Every frame | Single | **ACTIVE** |
| 8 | DestructionTickSystem | `src/core/integration_systems.rs` | fixed_tick | Every frame | Single | **ACTIVE** |
| 9 | TerrainDeformationTickSystem | `src/core/integration_systems.rs` | fixed_tick | Every frame | Single | **ACTIVE** |

### Navigation & Spatial

| # | System Name | Module Path | Tick Phase | Frequency | Threading | Status |
|---|-------------|-------------|------------|-----------|-----------|--------|
| 10 | NavDirtyTickSystem | `src/core/integration_systems.rs` | fixed_tick | Every frame | Single | **ACTIVE** |
| 11 | OcclusionWireSystem | `src/core/integration_systems.rs` | fixed_tick | Every frame | Single | **ACTIVE** |
| 12 | GoreWireSystem | `src/core/integration_systems.rs` | fixed_tick | Every frame | Single | **ACTIVE** |

### Integration / Wires

| # | System Name | Module Path | Tick Phase | Frequency | Threading | Status |
|---|-------------|-------------|------------|-----------|-----------|--------|
| 13 | AiDecisionWireSystem | `src/core/integration_systems.rs` | fixed_tick | Every frame | Single | **ACTIVE** |
| 14 | AnimationIntegrationSystem | `src/animation/animation_integration.rs` | fixed_tick | Every frame | Single | **ACTIVE** |
| 15 | AnimationWireSystem | `src/core/integration_systems.rs` | fixed_tick | Every frame | Single | **ACTIVE** |
| 16 | AudioIntegrationSystem | `src/audio/audio_integration.rs` | fixed_tick | Every frame | Single | **ACTIVE** |

### Gameplay

| # | System Name | Module Path | Tick Phase | Frequency | Threading | Status |
|---|-------------|-------------|------------|-----------|-----------|--------|
| 17 | QuestSystem | `src/gameplay/quest_system.rs` | fixed_tick | Every frame | Single | **ACTIVE** |

### New Systems (Render, Audio, Input, Network)

| # | System Name | Module Path | Tick Phase | Frequency | Threading | Status |
|---|-------------|-------------|------------|-----------|-----------|--------|
| 18 | RenderSystem | (render pipeline) | render | Every frame | Single | **ACTIVE** |
| 19 | AudioPlaybackBridge | (audio) | fixed_tick | Every frame | Single | **ACTIVE** |
| 20 | InputActionSystem | (input) | variable_tick | Per input | Single | **ACTIVE** |
| 21 | NetworkSystem | (network) | fixed_tick | Every frame | Single | **ACTIVE** |

### Plugin-Registered Systems (6 plugins)

| Plugin | Module Path | Status |
|--------|------------|--------|
| StalkerPlugin | `src/game/mod.rs` | **ACTIVE** — registers stalker-specific entity spawning |
| WeaponsPlugin | `src/game/weapons_plugin.rs` | **ACTIVE** — registers weapon definitions |
| CombatPlugin | `src/game/combat_plugin.rs` | **ACTIVE** — wires combat resolution |
| AiConfigPlugin | `src/game/ai_config.rs` | **ACTIVE** — registers AI config |
| EconomyPlugin | `src/game/economy_plugin.rs` | **ACTIVE** — wires economy tick |
| PopulationPlugin | `src/game/population_plugin.rs` | **ACTIVE** — manages population spawning |

**Total: 21 systems + 6 plugins = 27 runtime components**

---

## 2. Runtime Resources (Active)

| Resource | Purpose | Status |
|----------|---------|--------|
| PerfBudgetManager | Per-system timing (active) | **ACTIVE** |
| QualityGovernor | Frame pressure tracking | **ACTIVE** |
| BudgetRegistry | Phase budgets | **ACTIVE** |
| DirtySet | Chunk/Nav/Entity dirty tracking | **ACTIVE** |
| LowSpecCertifier | Low-spec mode checks | **ACTIVE** |
| DeterminismPolicyMatrix | Determinism controls | **ACTIVE** |
| SimTelemetry | Simulation telemetry | **ACTIVE** |
| SimMetricsDashboard | Metrics aggregation | **ACTIVE** |
| ItemRegistry | Item definitions | **ACTIVE** |
| SoundBank | Audio asset registry | **ACTIVE** |
| CorpseManager | Death pipeline / corpse handling | **ACTIVE** |
| ClipMap | Animation clip registry | **ACTIVE** |
| AnimationLadder | Animation state definitions | **ACTIVE** |
| FactionRelations | Faction state | **ACTIVE** |
| SchemaMigrationRegistry | Persistence schema | **ACTIVE** |
| WorldMilestoneTracker | World event tracking | **ACTIVE** |

---

## 3. Runtime Status Classification

### ACTIVE — Fully wired and running every tick

All 21 systems and 6 plugins are **ACTIVE**:

- **Core:** SimulationSystem, WorldTickSystem, AiSystem, PhysicsSystem, BodySystem
- **Pipeline:** BallisticsTickSystem, DamageDispatchSystem, DestructionTickSystem, TerrainDeformationTickSystem
- **Nav:** NavDirtyTickSystem, OcclusionWireSystem, GoreWireSystem
- **Integration:** AiDecisionWireSystem, AnimationIntegrationSystem, AnimationWireSystem, AudioIntegrationSystem
- **Gameplay:** QuestSystem
- **New:** RenderSystem, AudioPlaybackBridge, InputActionSystem, NetworkSystem
- **Plugins:** StalkerPlugin, WeaponsPlugin, CombatPlugin, AiConfigPlugin, EconomyPlugin, PopulationPlugin

### DORMANT — Implemented in code but NOT registered as runtime systems

- **AtmospherePass** (`src/graphics/atmosphere.rs`): Constructed in Renderer, `advance_frame()` not called in render loop.
- **ContactShadowPass** (`src/graphics/contact_shadows.rs`): Constructed in Renderer, not in render pipeline.
- **TaaPass** (`src/graphics/taa.rs`): `advance_frame()` called, but TAA resolve not in render output chain.
- **BloomPass** (`src/graphics/postprocess.rs`): HDR extraction exists, multi-pass composite not wired.
- **ActiveRagdoll** (`src/animation/active_ragdoll.rs`): Full implementation, never invoked at runtime.
- **FootIK** (`src/animation/foot_ik.rs`): Full implementation, never invoked at runtime.
- **MicroMotion** (`src/animation/micro_motion.rs`): Full implementation, never invoked at runtime.
- **ProceduralAnimation** (`src/animation/procedural.rs`): Full implementation, never invoked at runtime.
- **CampSimulation** (`src/simulation/camp_simulation.rs`): Data types defined, not ticked by any system.
- **RoleSimulation** (`src/simulation/role_simulation.rs`): Data types defined, not ticked by any system.
- **TraderEconomy** (`src/economy/trader_economy.rs`): Data types defined, not used by trading system.
- **BodyPhysicalResponseCache** (`src/body/body_response.rs`): Data types defined, not read by AI or movement.
- **AudioDebugState** (`src/audio/audio_debug.rs`): Data types only.
- **RenderValidation** (`src/graphics/render_validation.rs`): Static checks only, no runtime usage.
- **ArtDirectionProfile** (`src/graphics/art_direction.rs`): Data types defined, not applied to renderer.

### FEATURE_GATED

- **SceneHierarchy panel**: Gated behind `#[cfg(feature = "debug_ui")]` in `editor_shell.rs`.
- **AdvancedAnimation**: Several animation features gated behind `advanced_anim` feature flag.

### TEST_ONLY

- **VerticalSlice tests** (`tests/vertical_slice.rs`): Test-time only.
- **ProductionCandidate tests** (`tests/production_candidate.rs`): Test-time only.
- **PersistenceFull tests** (`tests/persistence_full.rs`): Test-time only.

---

## 4. Event System Status

### Main EventBus (`src/core/events/mod.rs`)

- **Status**: **ACTIVE**
- Created in `Engine::from_builder()`
- Events emitted by: all 21 registered systems + Engine tick (NewDay, NewMonth)
- Events consumed by: all systems that `reads_event::<T>()`
- Cleared every tick via `events.clear()`

### Active Event Producers (confirmed emitters)

| Producer | Events Emitted |
|----------|---------------|
| Engine::tick | `NewDay`, `NewMonth` |
| BallisticsTickSystem | `ImpactEvent`, `SoundTrigger` |
| DamageDispatchSystem | `BodyZoneDamaged`, `TerrainDeformed`, `SurfaceDamaged`, `SoundTrigger` |
| DestructionTickSystem | `WorldTopologyChanged`, `StructuralCollapse` |
| TerrainDeformationTickSystem | `TerrainChanged` |
| NavDirtyTickSystem | `NavUpdated`, `CoverChanged` |
| GoreWireSystem | `GoreMeshSpawn` |
| CommandBuffer | Boxed events via `commands.take_events()` |

### Active Event Consumers (confirmed readers)

| Consumer | Events Read |
|----------|------------|
| DamageDispatchSystem | `ImpactEvent` |
| DestructionTickSystem | `ImpactEvent` |
| TerrainDeformationTickSystem | `TerrainDeformed` |
| NavDirtyTickSystem | `WorldTopologyChanged`, `TerrainChanged` |
| OcclusionWireSystem | `StructuralCollapse` |
| GoreWireSystem | `BodyZoneDamaged` |
| SimulationSystem | `NewDay`, `NewMonth` (via EventBus) |
| WorldTickSystem | `NewDay`, `NewMonth` |
| QuestSystem | Game events |

### Secondary Buses

| Bus | Status | Producers | Consumers |
|-----|--------|-----------|-----------|
| SimBus | **REGISTERED, CLEARED** — `bus.clear()` called in Engine::tick(). No systems explicitly emit to SimBus. | None confirmed | None confirmed |
| RenderBus | **REGISTERED, CLEARED** — Same pattern. | None confirmed | None confirmed |
| DebugBus | **REGISTERED, CLEARED** — Same pattern. | None confirmed | None confirmed |

**Verdict**: SimBus, RenderBus, and DebugBus are registered and cleared every tick, but no system actually emits or reads from them. They are infrastructure-ready but functionally empty.

### EventTracer

- **Status**: **ACTIVE** — `enable()` called on construction
- Records: `frame_event_summary` with total channel count per tick
- Does NOT record individual event types or per-system emit counts
- Accessible via `resources.get::<EventTracer>()`

### EventAggregator

- **Status**: **ACTIVE** — Used by DamageDispatchSystem for spatial bucketing of ImpactEvents
- `submit()` called per impact, `drain()` called per tick
- Aggregated results are discarded (assigned to `_aggregated`)

---

## 5. Engine Tick Pipeline

The Engine runs a 5-phase tick at 20Hz simulation rate:

1. **PRE_TICK** (read-only ECS) — systems observe state
2. **CommandBuffer apply** — spawns, component ops, despawns, events
3. **FIXED_TICK** (mutable ECS) — main simulation step for all 21 systems
4. **CommandBuffer apply**
5. **POST_TICK** (read-only ECS) — post-processing
6. **CommandBuffer apply**
7. **RENDER_EXTRACT + RENDER_PREPARE** — read-only extraction
8. **QualityGovernor update** — records frame time
9. **BudgetRegistry update** — records total frame measurement; **PerfBudgetManager** records per-system timing
10. **EventTracer recording** — logs channel count
11. **Bus clearing** — SimBus, RenderBus, DebugBus, main EventBus

### Threading

- All systems run **single-threaded sequentially** in `tick()`
- Both `tick()` and `tick_parallel()` have **per-system timing** via PerfBudgetManager
- `WorkerPool` is constructed and available for future parallel scheduling

---

## 6. Honest Assessment

### What is REAL and RUNNING:
- ECS with ~100+ entities spawned at startup
- AI decision-making for NPCs and monsters
- Physics movement and ballistics
- Body/health system
- Quest generation and tracking
- Economy: money, trading, desperation mechanics
- Day/night cycle with monthly world events
- Damage pipeline: ballistics -> impacts -> destruction -> terrain deformation -> nav rebuild -> gore
- World streaming with chunk save/load
- Spatial indexing rebuild per frame
- Quality governor frame time tracking
- Per-system PerfBudget recording
- PlayerController, HUD, interaction, save/load, death/respawn wired in game binary
- Most Phase 3-10 modules now wired (ItemRegistry, SoundBank, ClipMap, AnimationLadder, WorldMilestoneTracker, CorpseManager, PerfBudgetManager, etc.)

### What is CODE but NOT RUNTIME:
- Remaining dormant: CampSimulation, RoleSimulation, TraderEconomy, BodyPhysicalResponseCache, advanced animation (ragdoll, foot IK, micro motion, procedural), secondary event buses (SimBus, RenderBus, DebugBus), several render passes (atmosphere, contact shadows, TAA resolve, SSAO, deferred)

### What is VISUALLY BROKEN:
- None — egui versions aligned; SDK EditorShell (17+ panels) renders and is interactive.
