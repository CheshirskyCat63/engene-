# ENGENE Runtime Truth — Governing Document

> This is the single source of truth for module status across the engine.
> Updated as part of each Track completion. Violations are doctor errors.
>
> IRON RULE: Nothing gets deleted. All dormant code is either wired, feature-gated, or frozen with explanation.

## Module Status Legend

| Status | Meaning |
|--------|---------|
| `ACTIVE` | Wired into tick loop or render path, tested, production-used |
| `WIRED_PARTIAL` | Connected as resource/system but key methods still unused |
| `FROZEN` | Intentionally dormant, behind `#[cfg(feature)]` gate with explanation |
| `EDITOR_ONLY` | SDK/editor panels and tools, not in game binary |
| `TEST_ONLY` | Test infrastructure, harnesses, fixtures |
| `DECLARED` | Interface/struct exists, logic not yet enforced or implemented |

---

## ACTIVE Systems (16 in tick loop)

| System | Path | Notes |
|--------|------|-------|
| `SimulationSystem` | src/simulation/simulation.rs | L0-L3 simulation levels |
| `WorldTickSystem` | src/simulation/world_tick.rs | Ecosystem, food chain, danger |
| `AiSystem` | src/ai/ai.rs | NPC + monster AI, social, trade |
| `PhysicsSystem` | src/physics/physics.rs | Rapier3D integration |
| `EconomySystem` | src/economy/economy.rs | Trading, monthly payments |
| `BallisticsTickSystem` | src/core/integration_systems.rs | Projectile simulation |
| `DamageDispatchSystem` | src/core/integration_systems.rs | 5-resolver damage chain |
| `DestructionTickSystem` | src/core/integration_systems.rs | Structural collapse |
| `TerrainDeformationTickSystem` | src/core/integration_systems.rs | Crater/impact processing |
| `NavDirtyTickSystem` | src/core/integration_systems.rs | Navigation rebuild |
| `OcclusionWireSystem` | src/core/integration_systems.rs | Destruction occlusion |
| `GoreWireSystem` | src/core/integration_systems.rs | Gore mesh generation |
| `AnimationIntegrationSystem` | src/animation/animation_integration.rs | Animation pipeline |
| `AudioIntegrationSystem` | src/audio/audio_integration.rs | Audio event dispatch |
| `QuestSystem` | src/gameplay/quest_system.rs | Quest generation, tracking, completion |
| `BodySystem` | src/body/body_system.rs | Body state in ECS, injury processing |

## ACTIVE Infrastructure

| Module | Path | Notes |
|--------|------|-------|
| `Engine` / `EngineBuilder` | src/core/engine.rs, src/core/plugin.rs | Core tick loop |
| `Ecs` / `PersistentEntityId` | src/core/ecs.rs, src/core/persistent_id.rs | Entity management |
| `EventBus` (multi-bus) | src/core/events/ | SimBus, RenderBus, DebugBus, Sticky, Tracer, Aggregator |
| `CommandBuffer` | src/core/commands.rs | Deferred mutations |
| `QualityGovernor` | src/core/quality_governor.rs | Frame time -> pressure levels |
| `BudgetRegistry` | src/core/budget_registry.rs | Per-system budget tracking |
| `RuntimeConfig` | src/core/runtime_config.rs | Profile selection |
| `ChunkPersistence` | src/world/chunk_persistence.rs | Save/load with relink |
| `WorldStreamer` | src/world/streaming.rs | Chunk load/unload |
| `OwnershipMap` | src/core/ownership_map.rs | System write/read contracts |
| `BuildManifest` | src/core/build_manifest.rs | Version + schema tracking |
| `CrashTelemetry` | src/core/crash_telemetry.rs | Panic hook + crash bundles |
| `SchemaMigrationRegistry` | src/core/build_manifest.rs | Save version migration |

## ACTIVE Plugins

| Plugin | Path | SDK Contract |
|--------|------|-------------|
| `StalkerPlugin` | src/game/stalker_plugin.rs | Yes |
| `WeaponsPlugin` | src/game/weapons_plugin.rs | Yes |
| `CombatPlugin` | src/game/combat_plugin.rs | Registered |
| `AiConfigPlugin` | src/game/ai_config.rs | Registered |
| `EconomyPlugin` | src/game/economy_plugin.rs | Registered, adds EconomySystem via builder |
| `PopulationPlugin` | src/game/population_plugin.rs | Registered, spawns NPCs+monsters via builder |

## ACTIVE Renderer

| Pass | Path | Low-spec |
|------|------|----------|
| Shadow (4 cascades) | src/graphics/shadow.rs | Reduced cascade count |
| Geometry / PBR | src/graphics/pbr.rs | Simplified materials |
| Skybox | src/graphics/skybox.rs | Always on |
| Bloom | src/graphics/postprocess.rs | Disabled on low-spec |
| Tonemap | src/graphics/postprocess.rs | Always on |
| Terrain | src/graphics/terrain.rs | Vertex color fallback |
| Vegetation (instanced) | src/graphics/vegetation.rs | Reduced density |

## WIRED_PARTIAL

| Module | Path | What's Missing |
|--------|------|---------------|
| `BodySystem` | src/body/ | Anatomy defined but not in ECS tick loop. Track G wires it. |
| `CombatTactics` | src/ai/combat_tactics/ | Profiles loaded but not applied in resolve_group_combat(). Track G. |
| `AtmospherePass` | src/graphics/atmosphere.rs | Constructed but not in render() chain. Track F. |
| `ContactShadowPass` | src/graphics/contact_shadows.rs | Constructed but not in render() chain. Track F. |
| `TaaPass` | src/graphics/taa.rs | Constructed but not in render() chain. Track F. |
| `SsaoPass` | src/graphics/postprocess.rs | Constructed but not in render() chain. Track F. |
| `ParticleSystem` | src/graphics/particles.rs | Constructed but not dispatching. Track F. |
| `SkinningSystem` | src/graphics/skinning.rs | Pipeline created but no entities use it. Track F. |
| `ActiveRagdoll` | src/animation/active_ragdoll.rs | Logic exists but not triggered. Track G. |
| `AudioEngine` | src/audio/audio.rs | Logic-only mode, no real playback. Track J wires rodio. |
| `ContentPipeline` | src/content/ | Import/Cook registered but not processing real assets. Track E. |
| `PrefabRegistry` | src/content/prefabs/ | Registered but not used in spawn path. Track E. |
| `ReplayRecorder` | src/core/replay/ | Tested but not wired into engine tick. Track C. |

## FROZEN (feature-gated, IRON RULE compliant)

| Module | Feature Gate | Reason |
|--------|-------------|--------|
| `network/` | `#[cfg(feature = "networking")]` | FROZEN: networking deferred to post-1.0 milestone |
| `ai/decision.rs` | `#[cfg(feature = "scoring_ai")]` | FROZEN: alternative to desire-based AI |
| `body/` (advanced) | `#[cfg(feature = "body_sim")]` | FROZEN: until Track G wires body into ECS |
| `animation/` (advanced) | `#[cfg(feature = "advanced_anim")]` | FROZEN: ragdoll/procedural until Track G |

## EDITOR_ONLY

| Panel | Path | Status |
|-------|------|--------|
| Scene Hierarchy | src/tools/scene_hierarchy.rs | Working |
| Inspector | src/tools/inspector.rs | Working, needs edit mode |
| Overlays | src/tools/overlays.rs | Working |
| Profiler Dashboard | src/tools/profiler_dashboard.rs | Working, needs GPU timing |
| Doctor | src/tools/doctor.rs | Working |
| Event Monitor | src/tools/event_monitor.rs | Working |
| Console | src/tools/console.rs | Working, needs engine commands |
| Time Controls | src/tools/time_controls.rs | Working |
| Replay Browser | src/tools/replay_browser.rs | Working |
| Asset Browser | src/tools/asset_browser.rs | Shell only, needs real content |
| Game Tools | src/tools/game_tools.rs | Working |
| Runtime Truth Dashboard | src/tools/runtime_truth_dashboard.rs | Working — Track B |
| Sim Metrics Dashboard | src/tools/sim_metrics_dashboard.rs | Working — Track B |
| Persistence Dashboard | src/tools/persistence_dashboard.rs | Working — Track C |
| World Map | src/tools/world_map.rs | Working — Track D |
| Quest Board | src/tools/quest_board.rs | Working — Track H |
| Economy Dashboard | src/tools/economy_dashboard.rs | Working — Track H |
| Crash Log Viewer | src/tools/crash_log_viewer.rs | Working — Track A |

## DECLARED (interface exists, enforcement ongoing)

| Module | Path | Status |
|--------|------|--------|
| `FactionRelations` | src/gameplay/factions.rs | Active — 7 factions with stance matrix |
| `EquipmentSlots` | src/world/components.rs | Active — weapon/armor/medkit/food/ammo |
| `FactionMembership` | src/world/components.rs | Active — per-entity faction component |

## TEST_ONLY

| Module | Path |
|--------|------|
| `ClosedLoopHarness` | src/testsupport/closed_loop.rs |
| `FixtureWorld` | src/testsupport/fixture_world.rs |
| `StressTest` | src/testsupport/stress_test.rs |
| `StreamingHarness` | src/testsupport/streaming_harness.rs |

---

## Binary Targets

| Binary | Path | Description |
|--------|------|------------|
| `engene` (default) | src/main.rs | Thin dispatcher, runs game |
| `engene_game` | src/bin/engene_game.rs | Standalone playable world |
| `engene_sdk` | src/bin/engene_sdk.rs | Full editor with GUI |
| `engene_headless` | src/bin/engene_headless.rs | CI/balance testing |

## Warning Debt

| Metric | Count | Target |
|--------|-------|--------|
| Total warnings | ~935 (mostly unused methods) | Triage per-track |
| Errors | 0 | 0 |
| Tests | 203 | 200+ (PASSED) |
| Test files | 12 | 12 |
| Benchmarks | 6 | 6 |
| Wired dormant systems | 30+ | All connected |
| SDK contracts registered | 16 | All core systems |
| Plugin dogfooding | 6/6 | All through builder API |
| Editor panels | 17 | 17 (all wired) |

---

## Production Freeze Protocol (Track L, before GO/NO-GO)

Mandatory freeze sequence before shipping:

1. **Code freeze** (7 days before GO/NO-GO) — no new features, only bug fixes
2. **Content freeze** (5 days before) — no new prefabs/chunks/assets
3. **Perf freeze** (3 days before) — no optimization experiments, only proven fixes
4. **Bugfix-only window** (final 3 days) — only crash/data-loss fixes
5. **GO/NO-GO day** — run full Gate D checklist

### Gate D Checklist

- [ ] `cargo build --bin engene_game` — clean
- [ ] `cargo build --bin engene_sdk` — clean
- [ ] `cargo build --bin engene_headless` — clean
- [ ] `cargo test` — 200+ tests, 0 failures
- [ ] `cargo bench` — benchmarks run, no regressions
- [ ] 10-minute stability: `engene_headless --months 3` completes without crash
- [ ] Doctor strict mode: zero errors
- [ ] Save/load round-trip: entity count matches before/after
- [ ] Determinism: same seed → same state after 1000 ticks
- [ ] Low-spec: 30 FPS on equivalent of GT 1030
- [ ] Memory: RSS < budget for 2x2km world
- [ ] Draw calls: within Contract 9 ceiling
- [ ] All 4 previous gates (A, B, C, D) re-verified
- [ ] Freeze protocol fully completed
