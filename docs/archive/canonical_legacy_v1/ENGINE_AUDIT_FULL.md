# ENGENE — Полный технический аудит игрового движка

> Дата аудита: 2026-03-15
> Версия движка: 0.1.0 (pre-release)
> Формат: инженерная карта системы + экспертный анализ

---

## Содержание

1. [Executive Summary](#1-executive-summary)
2. [Architecture Overview](#2-architecture-overview)
3. [Module Inventory](#3-module-inventory)
4. [Dependency Map](#4-dependency-map)
5. [Configuration & Variables Reference](#5-configuration--variables-reference)
6. [Entry Points & Lifecycle](#6-entry-points--lifecycle)
7. [Subsystem Documentation](#7-subsystem-documentation)
8. [Asset Pipeline & Tooling](#8-asset-pipeline--tooling)
9. [Build / Run / Deploy](#9-build--run--deploy)
10. [Repository Structure](#10-repository-structure)
11. [Risk Register & Tech Debt](#11-risk-register--tech-debt)
12. [Recommendations (Auditor)](#12-recommendations-auditor)
13. [Entry Points Summary](#13-entry-points-summary)
14. [Critical Files List](#14-critical-files-list)
15. [Gaps & Assumptions](#15-gaps--assumptions)
16. [General Verdict & Engine Maturity Assessment](#16-general-verdict--engine-maturity-assessment)
17. [Engine Purity Assessment (engine vs game coupling)](#17-engine-purity-assessment)
18. [Feature Maturity Matrix](#18-feature-maturity-matrix)
19. [Testability & Debugging Assessment](#19-testability--debugging-assessment)
20. [API Boundary Stability Analysis](#20-api-boundary-stability-analysis)
21. [Maintenance Burden & Bus Factor](#21-maintenance-burden--bus-factor)
22. [Architectural Philosophy & Product Core](#22-architectural-philosophy--product-core)
23. [True Weaknesses (5 Core Issues)](#23-true-weaknesses-5-core-issues)
24. [True Strengths (5 Core Assets)](#24-true-strengths-5-core-assets)
25. [Extended Recommendations & Roadmap](#25-extended-recommendations--roadmap)

---

## 1. EXECUTIVE SUMMARY

**ENGENE** — кастомный игровой движок на **Rust** (edition 2021, MSRV 1.75), ориентированный на survival-симулятор с открытым миром в духе S.T.A.L.K.E.R. Движок содержит **~8300 файлов**, 22 модуля верхнего уровня, 4 бинарных таргета, и предоставляет полный стек от ECS и физики до рендеринга (wgpu), AI с эмоциями/памятью/планированием, экономики, экосистемы, системы разрушений, баллистики и SDK-редактора с 23+ панелями.

**Ключевые характеристики:**

- Мир: 2×2 км (40×40 ячеек по 50 м), стриминг чанков, LOD-симуляция (L0/L1/L2)
- ECS: Кастомный на `SparseSet`, 24 компонентных хранилища, `Entity = u64`
- Рендеринг: wgpu 27, PBR, каскадные тени, HDR, volumetric clouds/fog, TAA, частицы
- Физика: Rapier3D + кастомная баллистика, огонь, вода, ткань, разрушения
- AI: Plan-based (не BT), желания/эмоции/память/тактика, NPC + 3 вида монстров
- Сеть: UDP client-server (20 Hz), 32 клиента (stub/early)
- Данные: 19 `.ron` конфигов, 8 чанков/префабов

---

## 2. ARCHITECTURE OVERVIEW

```
┌──────────────────────────────────────────────────────────────────┐
│                        BINARY TARGETS                            │
│  engene_game │ engene_sdk │ engene_headless │ engene_test        │
├──────────────┴────────────┴─────────────────┴────────────────────┤
│                    app::RuntimeAssembly                           │
│         (vertical_slice / headless / sandbox)                    │
├──────────────────────────────────────────────────────────────────┤
│  game (plugins)  │  tools (SDK/editor)  │  testsupport           │
├──────────────────┴──────────────────────┴────────────────────────┤
│                     GAMEPLAY LAYER                                │
│  ai │ economy │ ecosystem │ gameplay │ simulation │ animation    │
├──────────────────────────────────────────────────────────────────┤
│                     ENGINE SYSTEMS                                │
│  physics │ graphics │ audio │ navigation │ network │ input       │
├──────────────────────────────────────────────────────────────────┤
│                      CORE ENGINE                                 │
│  ecs │ events │ jobs │ registry │ commands │ time │ perf │ replay│
│  determinism │ debug │ access │ build_manifest │ crash_telemetry │
├──────────────────────────────────────────────────────────────────┤
│                    DATA / WORLD LAYER                             │
│  world │ body │ memory │ content (import/cook/prefabs)           │
├──────────────────────────────────────────────────────────────────┤
│                  EXTERNAL DEPENDENCIES                            │
│  wgpu│rapier3d│winit│glam│egui│rayon│serde│ron│pathfinding│etc.  │
└──────────────────────────────────────────────────────────────────┘
```

---

## 3. MODULE INVENTORY

| # | Модуль | Категория | Файлов | Назначение |
|---|--------|-----------|--------|------------|
| 1 | `core` | Core Engine | ~47 | ECS, события, задачи, реестры, время, детерминизм, перфоманс, replay, crash telemetry |
| 2 | `graphics` | Rendering | ~67 | wgpu рендер-пайплайн, PBR, тени, небо/погода, частицы, LOD, vegetation, debris |
| 3 | `physics` | Physics | ~31 | Rapier3D-обёртка, баллистика, огонь, вода, ткань, разрушения, damage pipeline |
| 4 | `ai` | AI | ~27 | Plan-based AI, желания, эмоции, память, восприятие, бой, размножение, тактика |
| 5 | `world` | World | ~30+ | Сетка мира, биомы, heightmap, стриминг, чанки, ресурсы, surface DB, origin shift |
| 6 | `navigation` | Navigation | ~11 | A* pathfinding, HPA*, cover map, RVO avoidance, world graph |
| 7 | `simulation` | Simulation | ~11 | LOD-симуляция (L0/L1/L2), world tick, лагеря, слухи, milestones |
| 8 | `body` | Body System | ~10 | Анатомия, зоны повреждений, кровотечение, расчленение, gore, death pipeline |
| 9 | `economy` | Economy | ~7 | Работы, торговля, деньги, отчаяние, item registry, trader state |
| 10 | `ecosystem` | Ecosystem | ~5 | Пищевая цепь, территории, миграция, потребление еды |
| 11 | `audio` | Audio | ~9 | 3D-звук, sound bank, occlusion/reverb, ambience, debug |
| 12 | `input` | Input | ~3 | Клавиатура/мышь, action bindings |
| 13 | `network` | Networking | ~7 | UDP client-server, интерполяция, net markers |
| 14 | `animation` | Animation | ~12 | Skeleton, locomotion, micro-motion, foot IK, ragdoll, active ragdoll |
| 15 | `memory` | Memory | ~6 | Entity pool, dirty flags, save/load, asset manager |
| 16 | `content` | Asset Pipeline | ~10 | Import, cooking, dependency graph, prefabs, validation |
| 17 | `gameplay` | Gameplay | ~5 | Фракции, квесты, quest system |
| 18 | `tools` | Tooling/SDK | ~23 | Editor shell, console, profiler, inspector, dashboards, doctor |
| 19 | `app` | App Bootstrap | ~2 | RuntimeAssembly — сборка движка под разные режимы |
| 20 | `game` | Game Logic | ~10 | Plugins (combat, economy, population, weapons, stalker), player, HUD |
| 21 | `testsupport` | Test Utils | ~4 | Closed-loop validation, stress test, streaming harness |
| 22 | `lib.rs` | Crate Root | 1 | Объявление всех 22 модулей |

---

## 4. DEPENDENCY MAP

### 4.1 Внешние зависимости

| Crate | Версия | Назначение | Где используется | Критичность | Замена |
|-------|--------|------------|------------------|-------------|--------|
| `wgpu` | 27 | GPU API (Vulkan/DX12/Metal) | `graphics/*`, `renderer.rs` | **Критическая** | Нет прямой замены (ash для Vulkan-only) |
| `winit` | 0.30 | Окна, events, input | `main.rs`, `bin/*` | **Критическая** | SDL2 (через bindgen) |
| `rapier3d` | 0.32 | Rigid-body физика | `physics/rapier_world.rs` only | Средняя | Bevy_rapier, custom |
| `glam` | 0.30 | Математика (Vec3, Mat4) | Повсеместно | **Критическая** | nalgebra, ultraviolet |
| `egui` | 0.33.3 | Immediate-mode GUI | `tools/*`, `bin/engene_sdk.rs` | Высокая (SDK) | imgui-rs |
| `egui-wgpu` | 0.33.3 | egui ↔ wgpu | `renderer.rs` | Высокая (SDK) | — |
| `egui-winit` | 0.33.3 | egui ↔ winit | `bin/engene_sdk.rs` | Высокая (SDK) | — |
| `rayon` | 1 | Thread pool | `core/jobs/worker_pool.rs` | Высокая | crossbeam, tokio |
| `parking_lot` | 0.12 | Быстрые мьютексы | `core/*`, `memory/*` | Средняя | std::sync::Mutex |
| `serde` | 1 (derive) | Сериализация | Повсеместно | **Критическая** | — |
| `ron` | 0.8 | RON формат данных | `game/data/*`, world config | Высокая | toml, json |
| `bincode` | 1 | Binary сериализация | `network/*`, `memory/save_chunks.rs` | Средняя | postcard, rmp |
| `pathfinding` | 4 | A* / Dijkstra | `navigation/*` | Средняя | petgraph, custom |
| `slotmap` | 1 | Generational arena | `core/ecs.rs` | Средняя | thunderdome |
| `tracing` | 0.1 | Structured logging | Повсеместно | Средняя | log |
| `tracing-subscriber` | 0.3 | Log output | `main.rs`, `bin/*` | Средняя | env_logger |
| `puffin` | 0.19 | Frame profiler | `core/profiler.rs`, `main.rs` | Низкая | tracy-client |
| `image` | 0.25 | Загрузка текстур | `memory/asset_manager.rs` | Средняя | — |
| `gltf` | 1 | 3D модели | `graphics/model_loader.rs` | Средняя | — |
| `bitflags` | 2 | Битовые флаги | `physics/damage_taxonomy.rs` | Низкая | enumflags2 |
| `bytemuck` | 1 (derive) | Safe casts | `graphics/*` (GPU буферы) | Средняя | — |
| `pollster` | 0.4 | Block on future | `renderer.rs` (wgpu init) | Низкая | futures::executor |
| `rand` | 0.8 | Рандом | AI, world gen, combat | Средняя | fastrand |
| `rodio` | 0.19 (opt.) | Audio playback | `audio/playback.rs` | Низкая (опц.) | kira, cpal |

### 4.2 Внутренние зависимости (граф)

```
core ← (всё)
world ← ai, simulation, physics, navigation, economy, ecosystem, memory, body, gameplay
ai ← simulation, body, world, economy
physics ← body, world
graphics ← world, input
navigation ← world, core
simulation ← world, ai, ecosystem, core
body ← physics, world, core
economy ← world, core, simulation
ecosystem ← world, core
audio ← core, world, graphics(occlusion)
animation ← core, ai, world, rapier3d
network ← core, memory, world
content ← (standalone, core for validation)
tools ← core, world, gameplay, content, graphics
game ← core, world, ai, economy, physics
app ← all systems
```

---

## 5. CONFIGURATION & VARIABLES REFERENCE

### 5.1 Build-time env variables (build.rs)

| Переменная | Источник | Назначение |
|------------|----------|------------|
| `ENGENE_BUILD_TIME` | `build.rs` | Unix timestamp сборки |
| `ENGENE_GIT_HASH` | `build.rs` (git rev-parse) | Short git hash |
| `ENGENE_BUILD_PROFILE` | `build.rs` (`$PROFILE`) | debug/release/etc. |
| `ENGENE_FEATURE_FLAGS` | `build.rs` (CARGO_FEATURE_*) | Список активных features |

### 5.2 Feature flags (Cargo.toml)

| Feature | Зависит от | Назначение | Влияние |
|---------|-----------|------------|---------|
| `default` = `full` | — | Полный движок | Включает всё |
| `engine_core` | — | Минимальное ядро | Только ECS, events, time |
| `physics` | engine_core | Физика | Rapier3D, огонь, вода |
| `render` | engine_core | Рендеринг | wgpu pipeline |
| `ai` | engine_core | AI | NPC/monster AI |
| `audio` | engine_core | Аудио | Sound bank |
| `headless` | engine_core, physics, ai | CI/серверный режим | Без окна и рендера |
| `debug_ui` | render | Debug GUI | egui панели |
| `full` | physics, render, ai, audio, debug_ui | Полный стек | — |
| `nightly-tests` | — | Ночные тесты | Доп. тесты |
| `audio_playback` | rodio | Реальный звук | Необязательно |
| `body_sim` | — | Симуляция тела | Body pipeline |
| `advanced_anim` | — | Продвинутая анимация | Ragdoll, IK |
| `networking` | — | Сеть | UDP server/client |
| `scoring_ai` | — | Scoring AI | decision.rs альтернатива |
| `low_spec` | full | Low-spec режим | Сертификация 30fps |
| `sdk_tools` | full, debug_ui | SDK-редактор | Все инструменты |

### 5.3 Build profiles

| Профиль | opt-level | LTO | Strip | Назначение |
|---------|-----------|-----|-------|------------|
| `dev` (default) | 0 | no | no | Разработка |
| `release` | 3 | thin | none | Релиз |
| `shipping` | 3 | fat | symbols | Финальная сборка |
| `low-spec` | 3 | thin | — | Слабые машины |
| `debug-tools` | 1 | no | no | Debug с оптимизацией |
| `sdk-tools` | 2 | thin | — | SDK editor |
| `headless-server` | 3 | thin | — | Серверный headless |

### 5.4 Runtime configuration constants (критические)

| Параметр | Значение | Файл | Влияние |
|----------|----------|------|---------|
| **SIM_TICK_RATE** | 20 Hz (dt=0.05s) | `core/runtime_config.rs`, `engine.rs` | Частота симуляции |
| **SECONDS_PER_DAY** | 120 | `core/time.rs` | Длительность игрового дня |
| **DAYS_PER_MONTH** | 30 | `core/time.rs` | Экономические циклы |
| **GRID_SIZE** | 40 | `world/cell.rs` | Размер мира в ячейках |
| **CELL_SIZE** | 50.0 м | `world/cell.rs` | Размер одной ячейки |
| **WORLD_SIZE** | 2000.0 м | `world/cell.rs` | Полный размер мира |
| **CHUNK_CELLS** | 4 | `world/cell.rs` | Ячеек на чанк |
| **L0_RADIUS** | 300 м | `simulation/simulation_level.rs` | Полная симуляция |
| **L1_RADIUS** | 5000 м | `simulation/simulation_level.rs` | Упрощённая |
| **L2_RADIUS** | 50000 м | `simulation/simulation_level.rs` | Фоновая |
| **L0_TICK_INTERVAL** | 1 | `simulation/simulation_level.rs` | Каждый тик |
| **L1_TICK_INTERVAL** | 12 | `simulation/simulation_level.rs` | Каждый 12-й тик |
| **L2_TICK_INTERVAL** | 60 | `simulation/simulation_level.rs` | Каждый 60-й тик |
| **MAX_PARTICLES** | 16384 | `graphics/particles.rs` | GPU частицы |
| **MAX_JOINTS** | 64 | `graphics/skinning.rs` | Скелет |
| **SHADOW_MAP_SIZE** | 2048 | `graphics/shadow.rs` | Разрешение теней |
| **CASCADE_COUNT** | 4 | `graphics/shadow.rs` | Каскады CSM |
| **CASCADE_SPLITS** | [0.05, 0.15, 0.4, 1.0] | `graphics/renderer.rs` | Разбиение каскадов |
| **DEPTH_FORMAT** | Depth32Float | `graphics/renderer.rs` | Формат z-буфера |
| **HDR_FORMAT** | Rgba16Float | `graphics/renderer.rs` | Формат HDR |
| **Event bus capacity** | 4096 | `core/events/mod.rs` | Макс. событий |
| **SimBus** | 2048 | `core/events/sim_bus.rs` | Sim events |
| **RenderBus** | 1024 | `core/events/render_bus.rs` | Render events |
| **DebugBus** | 512 | `core/events/debug_bus.rs` | Debug events |
| **MAX_CHAIN_DEPTH** | 4 | `physics/chain_reactions.rs` | Каскадных реакций |
| **MAX_EVENTS_PER_FRAME** | 32 | `physics/chain_reactions.rs` | Событий за кадр |
| **MAX_IMPACTS_PER_FRAME** | 64 | `physics/damage_pipeline/orchestrator.rs` | Ударов за кадр |
| **IGNITION_TEMP** | 100.0 | `physics/fire.rs` | Температура возгорания |
| **MAX_NPCS** | 30 | `ai/reproduction.rs` | Лимит NPC |
| **MAX_WOLVES** | 40 | `ai/reproduction.rs` | Лимит волков |
| **MAX_BOARS** | 40 | `ai/reproduction.rs` | Лимит кабанов |
| **MAX_BLOODSUCKERS** | 25 | `ai/reproduction.rs` | Лимит кровососов |
| **ENTITY_RADIUS** | 1.5 | `physics/collision.rs` | Радиус столкновения |
| **MAX_ACTIVE_PATCHES** | 4096 | `world/surface_state.rs` | Surface state patches |
| **MAX_DIRTY_UPLOADS/FRAME** | 16 | `world/surface_state.rs` | Upload budget |
| **SHIFT_THRESHOLD** | 5000.0 | `world/origin_shift.rs` | Origin shift порог |
| **MAX_CLIENTS** | 32 | `network/protocol.rs` | Сеть: клиентов |
| **SERVER_TICK_RATE** | 20 Hz | `network/protocol.rs` | Сеть: тик-рейт |

---

## 6. ENTRY POINTS & LIFECYCLE

### 6.1 Binary targets

| Binary | Path | Назначение | Ключевые features |
|--------|------|------------|-------------------|
| `engene_game` | `src/bin/engene_game.rs` | Играбельный клиент | full |
| `engene_sdk` | `src/bin/engene_sdk.rs` | Редактор с egui | sdk_tools |
| `engene_headless` | `src/bin/engene_headless.rs` | CI/balance тестирование | headless |
| `engene_test` | `src/bin/engene_test.rs` | Destruction sandbox | full |
| (default) | `src/main.rs` | Диспетчер → `engene_game` | full |

### 6.2 Bootstrap sequence (main.rs / engene_game)

```
1. install_panic_hook()           — crash telemetry
2. tracing_subscriber::init()     — logging
3. puffin::set_scopes_on(true)    — profiler
4. BuildManifest::current()       — print version/features
5. BuildManifest::ensure_data_dirs() — create game/world dirs
6. WorldGrid::generate()          — procedural world
7. Heightmap::generate()          — terrain
8. RuntimeAssembly::vertical_slice() — engine assembly:
   │
   ├── EngineBuilder::new()
   ├── register systems: SimulationSystem, WorldTickSystem,
   │   AiSystem, PhysicsSystem, QuestSystem, BodySystem,
   │   AudioIntegrationSystem, AudioPlaybackBridge,
   │   AnimationIntegrationSystem, RenderSystem,
   │   InputActionSystem, NetworkStateSystem,
   │   ClosedLoopRecorderSystem
   ├── register plugins: StalkerPlugin, EconomyPlugin,
   │   AiConfigPlugin, CombatPlugin, PopulationPlugin,
   │   WeaponsPlugin
   ├── insert resources: ResourceGrid, WorldStreamer,
   │   SurfaceDB, DamageableStore, DamageOrchestrator, etc.
   └── builder.build() → Engine
       ├── Engine::init()
       │   ├── startup(StartupContext) for each system
       │   └── resources.freeze()
       └── Engine ready
9. doctor::run_doctor()           — diagnostics
10. EventLoop::new() + run_app()  — winit event loop
```

### 6.3 Game loop (main.rs → RedrawRequested)

```
Per frame:
  1. Calculate dt
  2. camera.update(input, dt)
  3. input.end_frame()
  4. Fixed timestep accumulation (SIM_DT = 1/20):
     while accum >= SIM_DT:
       engine.tick(SIM_DT)     ← main sim tick
  5. AssetManager.poll()
  6. WorldStreamer.update() → load/unload chunks
  7. ChunkPersistenceService: save unloaded, load new
  8. HierarchicalSpatialIndex.rebuild()
  9. AudioEngine.set_listener() + update()
  10. Print economy reports (monthly)
  11. Renderer:
      a. Frustum cull entities
      b. Collect EntityInstances
      c. Update entity buffer
      d. renderer.render(camera)
```

### 6.4 Engine.tick() phases

```
engine.tick(real_delta):
  1. PRE_TICK phase (read-only ECS):
     - Run systems with Phase::PreTick
     - Apply command buffer
  2. FIXED_TICK phase (mutable ECS):
     - Run systems with Phase::FixedTick
     - Apply command buffer
  3. POST_TICK phase (read-only ECS):
     - Run systems with Phase::PostTick
     - Apply command buffer
  4. RENDER_EXTRACT + RENDER_PREPARE:
     - Extract data for rendering
  5. Post-tick cleanup:
     - QualityGovernor update
     - BudgetRegistry update
     - Event tracing
     - SimBus/RenderBus/DebugBus clear
     - Main EventBus clear
```

### 6.5 System update order (FixedTick)

```
Order 0: SimulationSystem     — L0/L1/L2 level updates
Order 1: WorldTickSystem      — day/month, food regen, migration, L1/L2 processing
Order 2: AiSystem             — NPC + monster AI ticks, respawn
Order 3: PhysicsSystem        — Rapier step, fire, water, cloth
Order 4: EconomySystem        — Monthly payments (on NewMonth)
Order 5: QuestSystem          — Quest expiry, generation, tracking
Order 6: BodySystem           — Body damage, death, health sync
Order 7: AudioIntegrationSystem
Order 8: AudioPlaybackBridge
Order 9: AnimationIntegrationSystem
Order 10: RenderSystem
Order 11: InputActionSystem
Order 12: NetworkStateSystem
Order 13: ClosedLoopRecorderSystem
```

### 6.6 Shutdown sequence

```
engine.shutdown():
  - Run each system's shutdown(ShutdownContext)
  - resources.unfreeze()
  - AsyncServices::shutdown_all()
```

---

## 7. SUBSYSTEM DOCUMENTATION

### 7.1 Core Engine (`src/core/`)

**Назначение:** Фундамент движка — ECS, события, задачи, время, ресурсы, replay, детерминизм, crash telemetry.

**Ключевые модули:**

| Модуль | Файл(ы) | Назначение |
|--------|---------|------------|
| ECS | `ecs.rs` | `Entity=u64`, 24 `SparseSet` хранилища компонентов |
| Engine | `engine.rs` | Tick loop, фазы, системы |
| Events | `events/` (8 файлов) | Типизированная шина событий, sticky, SimBus/RenderBus/DebugBus, canonical events |
| Jobs | `jobs/` (5 файлов) | `Job` trait, `WorkerPool` (rayon), `FrameGraph` (DAG), `Fence`, `TaskGroup` |
| Registry | `registry.rs` | `Resources` — type-id keyed хранилище с freeze |
| Commands | `commands.rs` | `CommandBuffer` — отложенные spawn/despawn/component ops |
| Time | `time.rs` | Игровое время: day, month, season, day_progress |
| Perf | `perf/` (10 файлов) | `PerfBudgetManager`, `MemoryBudgetRegistry`, `QualityGovernor`, `LowSpecCertifier` |
| Replay | `replay/` (3 файла) | `ReplayRecorder/Player`, `CheckpointManager`, `DivergenceDetector` |
| Determinism | `determinism_*.rs` (3 файла) | Аудит и классификация детерминизма систем |
| Access | `access/` (3 файла) | `Read<T>/Write<T>`, `AccessDescriptor`, conflict detection |
| Debug | `debug/` (2 файла) | `DebugRegistry`, `FrameRecorder` |
| BuildManifest | `build_manifest.rs` | Версии схем, метаданные сборки, миграции |
| CrashTelemetry | `crash_telemetry.rs` | Panic hook, crash reports в `crashes/` |
| Plugin | `plugin.rs` | Trait `Plugin { build(builder) }` |
| RuntimeConfig | `runtime_config.rs` | `RuntimeProfile`: Game, Editor, Headless, LowSpec; бюджеты per-profile |
| SDK | `sdk.rs` | SDK contracts, validation |

**ECS Component Storage (24 хранилища):**
`transforms`, `kinds`, `names`, `npc_traits`, `monster_traits`, `personal_needs`, `social_needs`, `ecosystem_needs`, `npc_economies`, `sim_levels`, `ai_states`, `inventories`, `memories`, `emotions`, `plans`, `life_info`, `flammables`, `cloth_components`, `tags`, `attributes`, `status_effects`, `blackboard`, `equipment`, `faction_memberships`

---

### 7.2 Rendering (`src/graphics/`)

**Назначение:** Полный GPU рендер-пайплайн на wgpu.

**Render Pipeline Order:**

```
Shadow Maps (4 cascades, 2048px) → Geometry (PBR terrain + entities)
→ Skybox → Volumetric Clouds (48-64 steps) → Fog → Stars → Moon
→ Precipitation → Bloom → Tonemapping → egui overlay
```

**Ключевые подсистемы:**

| Подсистема | Файлы | Описание |
|------------|-------|----------|
| PBR Renderer | `renderer.rs`, `pbr.rs` | Terrain + entity PBR шейдеры |
| Shadow Maps | `shadow.rs` | CSM, 4 каскада, 2048px |
| Sky/Weather | `sky/` (30 файлов) | Bruneton atmosphere, volumetric clouds, precipitation, fog, lightning, moon, stars |
| Particles | `particles.rs` | GPU compute + billboard, 16K max |
| Skinning | `skinning.rs` | 64 joints max |
| LOD | `lod.rs` | Full/Medium/Low/Billboard/Culled |
| GPU Culling | `gpu_culling.rs` | Compute shader frustum culling |
| Terrain | `terrain.rs` | Heightmap mesh generation |
| Vegetation | `vegetation.rs` | Grass/tree rendering |
| Decals | `decals.rs` | Bullet holes, cracks, scorch, blood |
| Gore | `gore_mesh.rs` | Gore mesh instances |
| Debris | `debris_instancing.rs` | Debris particles |
| Post-process | `postprocess.rs`, `taa.rs` | Bloom, tonemapping, TAA |
| IBL | `ibl.rs` | Image-based lighting |
| Contact Shadows | `contact_shadows.rs` | Screen-space contact shadows |
| Volumetric | `volumetric.rs` | Volumetric lighting |
| SSR | `ssr.rs` | Screen-space reflections |

**Все шейдеры:** Inline WGSL строки (25+ шейдеров). Нет внешних `.wgsl` файлов.

**Статус (из render_validation):**

- **Работает:** Shadow Maps, G-Buffer, PBR, IBL, Bloom, Tonemap, Skybox, Vegetation, Entity Instancing
- **Частично:** Atmosphere, Contact Shadows, TAA, Skinned Mesh
- **Отсутствует:** Particles в render loop, SSAO, Deferred Shading

---

### 7.3 Physics (`src/physics/`)

**Назначение:** Физика, баллистика, разрушения, элементальные симуляции.

**Architecture:**

```
PhysicsSystem
├── RapierPhysics (rapier_world.rs)  — rigid bodies, colliders, CCD
├── FireGrid (fire.rs)               — 2D огонь (ignition, spread, heat)
├── WaterGrid (water.rs)             — 2D вода (flow, evaporation, rain)
├── ClothWorld (cloth.rs)            — Verlet cloth simulation
└── sim_lod.rs                       — Physics LOD by distance

DamageOrchestrator (damage_pipeline/)
├── SurfaceResolver     → decals, sounds
├── LayerResolver       → layered material fracture
├── StructuralResolver  → structural collapse
└── BodyResolver        → anatomic damage (zones, joints, bleeding)
```

**Rapier3D использование:** Только в `rapier_world.rs` — rigid bodies (kinematic/dynamic), capsule colliders, ground plane, CCD. Баллистика, огонь, вода, ткань, collision resolution — **кастомные**.

**Ключевые системы:**

- **Ballistics:** Drag, wind, adaptive substeps (4 при >300 m/s), material penetration, ricochet
- **Destruction:** Node-link model, stress propagation, fragment spawning
- **Tile Wall Fracture:** 3-layer (tile→adhesive→support), weapon-specific impacts
- **Chain Reactions:** Max depth 4, 32 events/frame, energy floor 10.0

---

### 7.4 AI (`src/ai/`)

**Назначение:** Plan-based AI для NPC и 3 видов монстров (Wolf, Boar, Bloodsucker).

**Architecture (NO behavior trees):**

```
Per entity per tick:
  1. decay_and_sense()      — needs, emotions, memory decay
  2. desire_npc/monster()   — priority-based goal selection
  3. should_replan()        — check for higher-priority goal
  4. find_target_for_goal() — spatial search
  5. execute()              — movement + action for current goal
```

**Типы целей (Goal):** SeekFood, SeekWater, Hunt, Flee, Explore, Socialize, Work, Trade, Rest, Migrate, DefendTerritory, FollowPack, StealOrRob, Mate, SeekShelter, Sleep

**Пищевая цепь:** Boar(1) < Wolf(2) < Bloodsucker(3), NPC power=5

**Подсистемы:**

| Модуль | Назначение |
|--------|------------|
| `desire.rs` | Приоритетный выбор цели (голод/страх/здоровье/эмоции/черты) |
| `thresholds.rs` | Динамические пороги из traits + памяти |
| `plan.rs` | Активный план: goal + target + duration |
| `perception.rs` | Spatial cache: prey, predators, allies, mates |
| `memory.rs` | События, spatial marks, entity opinions, lessons (adaptive learning) |
| `emotions.rs` | Fear, anger, grief, joy, trust, loneliness; personality modulation |
| `combat.rs` | 1v1 и group combat, damage, death, rewards |
| `combat_tactics/` | Threat assessment, tactic selection, coordination, cover |
| `reproduction.rs` | Trait blending, population caps, offspring spawning |
| `witness.rs` | Kill/attack reactions, danger/rally communication |

---

### 7.5 World & Streaming (`src/world/`)

**Мир:** 2×2 км = 40×40 ячеек по 50 м = 10×10 чанков (200 м/chunk)

**Streaming:**

- Camera-based chunk loading/unloading
- `ChunkPersistenceService`: save entities on unload, restore on load
- `BudgetedStreamer`: read 2MB/frame, decompress 4MB/frame, upload 1MB/frame
- `OriginShift`: threshold 5000 м

**Resource Grid:** Food/water per cell, carcass tracking (decay 600s), seasonal regeneration

**Surface System:** Surface materials (16), damage profiles, terrain deformation (craters), surface state masks (dirt/wetness/scorch/blood)

---

### 7.6 Navigation (`src/navigation/`)

| Модуль | Алгоритм/Назначение |
|--------|---------------------|
| `navmesh.rs` | A* на heightmap-сетке |
| `hpa_star.rs` | Hierarchical A* (cluster size 8) |
| `world_graph.rs` | Абстрактный граф локаций |
| `cover_map.rs` | Precomputed cover quality |
| `avoidance.rs` | RVO local avoidance (radius 5, horizon 2s) |
| `path_cache.rs` | Кэш путей |
| `breach_analysis.rs` | Детекция пробоин в стенах |

---

### 7.7 Simulation (`src/simulation/`)

**LOD-симуляция:**

- **L0** (≤300 м): Полная — каждый тик, AI, физика, анимация
- **L1** (≤5000 м): Упрощённая — каждый 12-й тик, offline combat/work
- **L2** (≤50000 м): Фоновая — каждый 60-й тик, decay only

**Дополнительно:** Camp simulation, social propagation (слухи, mood, trust), world milestones

---

### 7.8 Audio (`src/audio/`)

3D spatial audio. SoundBank (40+ event types). Occlusion/reverb zones. Biome-based ambience. Optional `rodio` для реального playback.

---

### 7.9 Networking (`src/network/`)

UDP client-server. 20 Hz server tick, 32 max clients. `InterpolationBuffer` (4 snapshots, 100ms delay). `NetStateMarker` для ECS state replication. **Статус: early/partial implementation.**

---

### 7.10 Economy (`src/economy/`)

Monthly payment cycle. 9 jobs with income/danger/energy. Desperation → banditry (0.7 threshold). `TraderState` with restocking. `ItemRegistry` + `ItemInstance`.

---

### 7.11 Animation (`src/animation/`)

Skeleton (64 joints), locomotion state machine, clip maps, micro-motion (wind/pendulum), foot IK (two-bone), injury animation, ragdoll + active ragdoll (PD control), LOD (Full → Off by distance).

---

## 8. ASSET PIPELINE & TOOLING

### 8.1 Content Pipeline (`src/content/`)

```
Import (source hashing) → Cook (Dev/Shipping/LowSpec) → Validate → Prefab Registry
```

- **Import:** `ImportPipeline` — source registration, hash-based reimport detection
- **Cooking:** `CookPipeline` — 3 варианта (Dev, Shipping, LowSpec)
- **Dependency Graph:** `ContentDependencyGraph` — DAG, dirty propagation
- **Prefabs:** `PrefabRegistry` — inheritance, 22+ prefab definitions
- **Validation:** `ContentValidator` — prefab validation, `AssetBudget` (2000 DC, 500K tris, 256MB VRAM per chunk)

### 8.2 SDK/Editor (`src/tools/`)

23 инструментальных модуля в `EditorShell`:

| Инструмент | Назначение |
|------------|------------|
| `profiler_dashboard` | Frame timing, per-system timing |
| `inspector` | Entity inspector с edit mode |
| `console` | 16 встроенных команд |
| `doctor` | Diagnostics (entity counts, system health) |
| `economy_dashboard` | Wealth, trade, payments |
| `sim_metrics_dashboard` | Population, ecology |
| `event_monitor` | Event bus logging |
| `runtime_truth_dashboard` | Module status (24 entries) |
| `persistence_dashboard` | Chunk save/load status |
| `world_map` | 2D world overview |
| `quest_board` | Active quests |
| `scene_hierarchy` | Scene tree |
| `asset_browser` | Asset list |
| `prefab_placer` | Prefab placement (9 types, 16 materials) |
| `time_controls` | Sim time control |
| `crash_log_viewer` | Crash reports |
| `overlays` | World overlay toggles |
| `debug_ui` | Main menu & visibility |
| `editor_safe_mode` | Panel time budgets (2ms per panel, 8ms total) |
| `replay_browser` | Replay recordings |
| `game_tools` | Encounter/faction/spawn editors |

---

## 9. BUILD / RUN / DEPLOY

### 9.1 Сборка

```powershell
# Dev build
cargo build

# Release
cargo build --release

# SDK editor
cargo build --release --bin engene_sdk --features sdk_tools

# Headless
cargo build --release --bin engene_headless --features headless

# Low-spec
cargo build --profile low-spec --bin engene_game --features low_spec

# Shipping
cargo build --profile shipping --bin engene_game
```

### 9.2 Запуск

```powershell
# Game (default)
cargo run --bin engene_game

# SDK editor
cargo run --bin engene_sdk --features sdk_tools

# Headless (3 months)
cargo run --bin engene_headless -- --months 3

# Headless with truth dump
cargo run --bin engene_headless -- --months 1 --truth

# Destruction sandbox
cargo run --bin engene_game -- --layout destruction_sandbox_50x50

# Version info
cargo run -- --version
```

### 9.3 Scripts

| Скрипт | Назначение |
|--------|------------|
| `run_sdk.ps1` | Запуск SDK (с опц. build) |
| `run_game.ps1` | Запуск игры |
| `build_release.ps1` | Сборка всех бинарников |
| `package_release.ps1` | Упаковка в `dist/release/` (ENGENE_Game.exe, ENGENE_SDK.exe, ENGENE_Headless.exe, TEST.exe) |

### 9.4 Обязательные зависимости для сборки

- Rust toolchain ≥ 1.75.0
- GPU с поддержкой Vulkan/DX12/Metal (для рендера)
- Git (для build.rs git hash)
- Все crate-зависимости из `Cargo.toml`

---

## 10. REPOSITORY STRUCTURE

```
engene/
├── Cargo.toml                 # Package config, features, profiles, dependencies
├── build.rs                   # Build-time env vars (timestamp, git hash, features)
├── clippy.toml                # Linter config (MSRV 1.75, thresholds)
├── run_sdk.ps1                # SDK launch script
├── run_game.ps1               # Game launch script
├── build_release.ps1          # Release build script
├── package_release.ps1        # Package script → dist/
├── README_FIRST_RUN.md        # Quick start guide
│
├── src/
│   ├── lib.rs                 # Crate root — 22 module declarations
│   ├── main.rs                # Default binary (game dispatcher)
│   ├── bin/
│   │   ├── engene_game.rs     # Standalone game with player/save
│   │   ├── engene_sdk.rs      # Editor with egui
│   │   ├── engene_headless.rs # CI/headless simulation
│   │   └── engene_test.rs     # Destruction sandbox
│   │
│   ├── core/                  # (~47 files) ECS, events, jobs, perf, replay, debug
│   ├── graphics/              # (~37 files) Renderer, PBR, terrain, particles, LOD
│   │   └── sky/               # (~30 files) Atmosphere, clouds, weather, fog, moon
│   ├── physics/               # (~25 files) Rapier, ballistics, fire, water, cloth
│   │   └── damage_pipeline/   # (~6 files) Orchestrator, resolvers
│   ├── ai/                    # (~27 files) Plan-based AI, emotions, memory, combat
│   │   └── combat_tactics/    # (~5 files) Threat, tactics, coordination, cover
│   ├── world/                 # (~30 files) Grid, biomes, heightmap, streaming, surface DB
│   ├── navigation/            # (~11 files) A*, HPA*, RVO, cover map
│   ├── simulation/            # (~11 files) LOD sim, world tick, camps, social
│   ├── body/                  # (~10 files) Anatomy, gore, death, dismemberment
│   ├── economy/               # (~7 files) Jobs, trading, items, desperation
│   ├── ecosystem/             # (~5 files) Food chain, territory, migration
│   ├── audio/                 # (~9 files) 3D audio, sound bank, ambience
│   ├── input/                 # (~3 files) Keyboard/mouse, action bindings
│   ├── network/               # (~7 files) UDP server/client, interpolation
│   ├── animation/             # (~12 files) Skeleton, locomotion, ragdoll, IK
│   ├── memory/                # (~6 files) Save/load, asset manager, dirty flags
│   ├── content/               # (~10 files) Import, cook, prefabs, validation
│   ├── gameplay/              # (~5 files) Factions, quests
│   ├── tools/                 # (~23 files) SDK editor panels
│   ├── app/                   # (~2 files) RuntimeAssembly
│   ├── game/                  # (~10 files) Plugins, player, HUD
│   └── testsupport/           # (~4 files) Closed-loop, stress test
│
├── tests/                     # ~25 integration test files
├── benches/
│   └── engine_benchmarks.rs   # 10 micro-benchmarks
│
├── game/
│   ├── data/                  # 19 .ron config files
│   └── world/                 # Layouts, prefabs, chunks
│       ├── layouts/           # World layouts (.ron)
│       ├── prefabs/           # Prefab definitions (.ron)
│       └── chunks/            # Authored chunk data (.ron)
│
└── docs/
    └── canonical/             # Specs, reports, art direction
```

---

## 11. RISK REGISTER & TECH DEBT

### 11.1 Критические риски

| # | Риск | Severity | Файлы | Описание |
|---|------|----------|-------|----------|
| R1 | **RON configs не загружаются** | HIGH | `food_chain.ron`, `species.ron`, `rules.ron`, `surfaces.ron`, `material_bridge.ron` | Данные authored в `.ron`, но логика hardcoded. `MaterialTruthService::empty()` используется в runtime. Есть разрыв data↔code. |
| R2 | **ECS не generational** | MEDIUM | `core/ecs.rs` | `Entity=u64` с ручным генерационным tracking через `slotmap`. `SparseSet` — не thread-safe для concurrent read/write. |
| R3 | **Все шейдеры inline** | MEDIUM | `graphics/*.rs` | 25+ WGSL шейдеров как Rust строки. Нет hot reload, нет инструментов отладки шейдеров, усложняет итерацию. |
| R4 | **Networking — stub** | HIGH | `network/*` | UDP framework есть, но интеграция с ECS минимальна. `NetworkStateSystem` — placeholder. |
| R5 | **No SSAO, no deferred** | MEDIUM | `graphics/lighting.rs` | Deferred shading — stub. SSAO отсутствует. Forward-only рендер. |
| R6 | **gpu_jobs.rs — пустой stub** | LOW | `graphics/gpu_jobs.rs` | Помечен как Phase 9, содержит один комментарий. GPU compute jobs не реализованы. |
| R7 | **Particles не в render loop** | MEDIUM | `graphics/particles.rs`, `render_validation.rs` | `ParticleSystem` реализован (compute + billboard), но не подключён к основному render pass. |
| R8 | **Collision — 2D circles only** | MEDIUM | `physics/collision.rs` | `resolve_collisions()` — простое 2D circle overlap (radius 1.5). Rapier3D используется отдельно и не интегрирован с entity collision. |
| R9 | **Population caps hardcoded** | LOW | `ai/reproduction.rs` | MAX_NPCS=30, MAX_WOLVES=40 и т.д. дублируют значения из `population.ron`, но загружаются из кода, не из конфига. |
| R10 | **Memory stubs** | MEDIUM | `memory/entity_pool.rs`, `memory_manager.rs`, `streaming_cache.rs` | Три файла — пустые stubs. Memory management реально не реализован. |
| R11 | **Single-threaded tick** | HIGH | `core/engine.rs` | `tick()` выполняет все системы последовательно. `tick_parallel()` существует, но не используется в main loop. `WorkerPool` + `FrameGraph` готовы, но не задействованы. |
| R12 | **Save schema v1 — нет миграций** | MEDIUM | `core/build_manifest.rs` | `SchemaMigrationRegistry` реализован, но все версии = 1 и реальных миграций нет. Первое изменение схемы потребует ручной работы. |
| R13 | **Crash telemetry — global Mutex** | LOW | `core/crash_telemetry.rs` | `CRASH_CONTEXT: Mutex<CrashContext>` — глобальный static. При panic в panic возможен deadlock. |
| R14 | **Decision.rs не используется** | LOW | `ai/decision.rs`, `ai/goals.rs` | Альтернативная scoring-based система выбора целей. Не подключена к main tick — мёртвый код или эксперимент. |
| R15 | **Origin shift не протестирован** | MEDIUM | `world/origin_shift.rs` | Порог 5000 м, код реализован, но мир всего 2000 м — shift никогда не сработает при текущих параметрах. |

### 11.2 Технический долг

| Область | Описание | Критичность | Файлы |
|---------|----------|-------------|-------|
| **Data-code gap** | Многие `.ron` конфиги authored, но значения hardcoded в коде. Загрузчики есть для ~10 из 19 файлов, остальные не подключены. | HIGH | `food_chain.ron`, `species.ron`, `rules.ron`, `surfaces.ron`, `material_bridge.ron` |
| **Дублирование констант** | Радиусы L0/L1/L2, population caps, job incomes определены и в `.ron`, и в Rust-коде. Нет единого source of truth. | HIGH | `simulation_level.rs` vs `simulation.ron`, `reproduction.rs` vs `population.ron` |
| **Отсутствие ECS query system** | Нет typed query API (`Query<(&Transform, &AiState)>`). Компоненты достаются вручную через `ecs.transforms.get(&e)`. | MEDIUM | `core/ecs.rs`, все системы |
| **Forward-only rendering** | Нет deferred shading pipeline. При росте числа light sources будут проблемы с производительностью. | MEDIUM | `graphics/lighting.rs` (stub) |
| **Тестовое покрытие** | 25 integration test файлов — хорошо. Но unit-тесты внутри модулей не обнаружены при аудите. | MEDIUM | `tests/` |
| **Нет hot reload** | Шейдеры inline, `.ron` конфиги загружаются при старте. Нет механизма hot reload для итерации. | MEDIUM | Весь content pipeline |
| **Gameplay.rs — placeholder** | `gameplay/gameplay.rs` помечен как Phase 11, пустой. | LOW | `gameplay/gameplay.rs` |
| **Unused feature flags** | `body_sim`, `advanced_anim`, `networking`, `scoring_ai` объявлены, но не gate-ят код через `#[cfg(feature)]` проверки. | LOW | `Cargo.toml` |
| **No error propagation** | Многие функции используют `unwrap()` или просто игнорируют ошибки вместо `Result<>`. | MEDIUM | Разбросано по кодовой базе |

### 11.3 Узкие места производительности

| Bottleneck | Описание | Файл | Рекомендация |
|------------|----------|------|--------------|
| **Sequential tick** | Все системы выполняются в одном потоке, хотя `WorkerPool` и `FrameGraph` готовы | `engine.rs` | Включить `tick_parallel()` |
| **Spatial index rebuild per frame** | `HierarchicalSpatialIndex.clear()` + full rebuild каждый кадр в main.rs | `main.rs:246-252` | Incremental update |
| **AI per-entity полный проход** | Каждый NPC/monster проходит полный perception + desire + plan + execute каждый тик | `ai/ai.rs` | Stagger AI ticks, budget-based |
| **Fire/Water grid** | 2D симуляции на весь мир без spatial culling | `physics/fire.rs`, `water.rs` | Активировать только вблизи камеры (sim_lod частично есть) |
| **Surface state patches** | MAX_ACTIVE_PATCHES=4096, MAX_DIRTY_UPLOADS=16/frame | `world/surface_state.rs` | Может быть bottleneck при массовых разрушениях |
| **Cloud ray-marching** | 48-64 steps per pixel | `graphics/sky/cloud_renderer.rs` | Quality profiles уже есть (Ultra→Low) |

### 11.4 Места с высокой связанностью (coupling)

| Связка | Описание |
|--------|----------|
| `world::components` ↔ всё | 30+ типов компонентов в одном файле, от которого зависят AI, physics, simulation, economy, body, gameplay |
| `core::ecs` ↔ все системы | Все системы напрямую обращаются к `ecs.transforms`, `ecs.ai_states` и т.д. — нет абстракции query |
| `main.rs` — monolithic game loop | 514 строк, вручную оркестрирует streaming, audio, spatial index, rendering. Должно быть в системах. |
| `ai` ↔ `world` ↔ `economy` | AI вызывает economy::trading, economy знает про AiState, world::components содержит и NpcEconomy, и Goal |

---

## 12. RECOMMENDATIONS (AUDITOR)

### 12.1 Архитектурные

| # | Рекомендация | Приоритет | Обоснование |
|---|-------------|-----------|-------------|
| 1 | **Data-driven config loading** — подключить загрузку всех 19 `.ron` файлов, убрать hardcoded значения | P0 | Устранит data-code gap, позволит балансировать без перекомпиляции |
| 2 | **Включить parallel tick** — задействовать `tick_parallel()`, `FrameGraph`, `WorkerPool` | P0 | Раскроет многоядерность, уже реализовано но не используется |
| 3 | **Typed ECS queries** — добавить query API вместо прямого доступа к SparseSet | P1 | Уменьшит coupling, позволит автоматический conflict detection |
| 4 | **Извлечь game loop из main.rs** — перенести streaming, spatial rebuild, audio update в системы | P1 | main.rs станет тонким dispatcher |
| 5 | **Shader hot reload** — вынести WGSL в файлы, добавить file watcher | P2 | Ускорит итерацию по визуалу |
| 6 | **Deferred rendering path** — реализовать `lighting.rs` | P2 | Необходимо для масштабирования количества источников света |

### 12.2 Стабильность и качество

| # | Рекомендация | Приоритет |
|---|-------------|-----------|
| 7 | Добавить unit-тесты в каждый модуль (особенно `core/ecs`, `physics/damage_pipeline`, `ai/desire`) | P1 |
| 8 | Подключить particles к render loop | P1 |
| 9 | Заменить `unwrap()` на proper error handling в критических путях | P1 |
| 10 | Реализовать `memory_manager.rs` и `entity_pool.rs` (сейчас stubs) | P2 |
| 11 | Удалить или пометить `#[deprecated]` неиспользуемый `decision.rs` / `goals.rs` | P2 |

### 12.3 Масштабирование

| # | Рекомендация | Приоритет |
|---|-------------|-----------|
| 12 | AI tick staggering — не все NPC каждый кадр, а budget-based распределение | P1 |
| 13 | Incremental spatial index update вместо полного rebuild | P1 |
| 14 | Chunk LOD streaming — подключить `BudgetedStreamer` и `ChunkPackageRegistry` | P2 |
| 15 | Networking — определить scope (dedicated server? P2P?) и довести до production | P2 |
| 16 | Split `world::components.rs` — разделить 30+ типов по доменным модулям | P2 |

---

## 13. ENTRY POINTS SUMMARY

| Entry Point | Command | Описание |
|-------------|---------|----------|
| Default game | `cargo run` | `src/main.rs` → vertical_slice, 3D окно |
| Game binary | `cargo run --bin engene_game` | Полноценная игра с player controller, save/load |
| SDK editor | `cargo run --bin engene_sdk --features sdk_tools` | egui editor, 23 панели |
| Headless sim | `cargo run --bin engene_headless -- --months N` | CI, balance testing |
| Destruction test | `cargo run --bin engene_test` | 50×50 sandbox, баллистика |
| Benchmarks | `cargo bench` | 10 micro-benchmarks |
| Tests | `cargo test` | 25 integration tests |

---

## 14. CRITICAL FILES LIST

| Файл | Почему критичен |
|------|-----------------|
| `Cargo.toml` | Features, dependencies, profiles — вся конфигурация сборки |
| `build.rs` | Build-time env vars — без него нет версионирования |
| `src/core/engine.rs` | Сердце движка — tick loop, фазы, system execution |
| `src/core/ecs.rs` | Всё состояние мира — entities + 24 component storages |
| `src/core/events/mod.rs` | Межсистемная коммуникация — event bus |
| `src/core/registry.rs` | Resources container — все shared ресурсы |
| `src/app/runtime_assembly.rs` | Сборка движка — определяет какие системы и ресурсы загружаются |
| `src/main.rs` | Game loop — streaming, rendering, input, audio orchestration |
| `src/graphics/renderer.rs` | Рендер пайплайн — все проходы рисования |
| `src/physics/physics.rs` | Физический пайплайн — Rapier + fire + water + cloth |
| `src/ai/ai.rs` | AI system entry — NPC + monster tick dispatch |
| `src/ai/desire.rs` | Goal selection — определяет поведение всех существ |
| `src/world/components.rs` | 30+ типов компонентов — от Transform до LifeInfo |
| `src/world/cell.rs` | World geometry constants — GRID_SIZE, CELL_SIZE, WORLD_SIZE |
| `src/core/build_manifest.rs` | Schema versions — критично для save compatibility |
| `game/data/weapons.ron` | Оружие — баллистика зависит от этих значений |
| `game/data/economy.ron` | Экономика — пороги отчаяния, доходы |
| `game/data/population.ron` | Популяция — caps, respawn, имена |

---

## 15. GAPS & ASSUMPTIONS

| # | Тип | Описание | Требует верификации |
|---|-----|----------|---------------------|
| G1 | GAP | Не обнаружен `#[cfg(feature = "...")]` gating для features `body_sim`, `advanced_anim`, `networking`, `scoring_ai` | Проверить, действительно ли features влияют на компиляцию |
| G2 | GAP | `SurfaceDB` и `MaterialBridge` — не найден loader из `.ron` в runtime | Как именно заполняются эти структуры в production? |
| G3 | GAP | `world_layout.ron` — не найден загрузчик | Hardcoded в `authored_sets.rs`? |
| G4 | ASSUMPTION | `tick_parallel()` существует, но предполагается что не тестировался под нагрузкой | Нужен stress test |
| G5 | GAP | Нет LICENSE файла в корне проекта | Юридический risk |
| G6 | GAP | `gameplay/gameplay.rs` — Phase 11 placeholder | Не ясен scope player interaction |
| G7 | ASSUMPTION | `OriginShift` с порогом 5000 м при мире 2000 м — предположительно для будущего масштабирования | Подтвердить intention |
| G8 | GAP | `component_registry.rs`, `content_validation.rs`, `game_config.rs`, `integration_systems.rs`, `ownership_map.rs`, `parallel_validation.rs`, `world_state_authority.rs`, `runtime_manifest.rs` в `core/` — не были полностью прочитаны | Требуют отдельного аудита |

---

<!-- ================================================================ -->
<!-- ЧАСТЬ 2: ЭКСПЕРТНЫЙ АНАЛИЗ                                       -->
<!-- ================================================================ -->

# ЧАСТЬ 2: ЭКСПЕРТНЫЙ АНАЛИЗ И РАСШИРЕННЫЕ РЕКОМЕНДАЦИИ

> Надстройка над инженерным аудитом. Оценка зрелости, архитектурной философии,
> реальных сильных и слабых мест, и конкретный roadmap дальнейшей работы.

---

## 16. GENERAL VERDICT & ENGINE MATURITY ASSESSMENT

### 16.1 Общий вердикт

По данному аудиту движок **не выглядит ущербным**. Он выглядит как **амбициозный, перегруженный, местами недоведённый, но архитектурно живой проект**. Это важная разница.

**Ущербный движок** — это когда хаос, нет опорного каркаса, нет фаз, нет системной модели, нет сборки, нет границ между подсистемами, а всё держится на скотче и панике. Здесь каркас есть. Более того, каркас жирный.

Но есть и второй слой правды: движок выглядит как **"vertical slice engine", который притворяется production-grade платформой**. В нём уже очень много подсистем, фич и красивых слов, но часть из них явно не дошла до стадии полной эксплуатационной зрелости. Это типичная зверушка для инди/малой команды: великолепная инженерная жадность, а потом суровая физика времени.

### 16.2 Классификация

По описанию ENGENE — это:

- **Сильный R&D-движок / sandbox engine**
- **Не до конца зрелый production engine**
- **Точно не "мусор"**

У него видны хорошие инженерные инстинкты:

- Модульность
- Runtime assembly
- Фазный tick loop
- Separation на core / systems / gameplay / tools
- Попытка в determinism, replay, perf budgets, crash telemetry
- Нормальный toolchain и SDK
- Data pipeline
- Осознанность по LOD-симуляции и стримингу

Это не уровень "студент на коленке собрал 3 файла и назвал это движком". Тут уже зверь с зубами. Но зубы местами молочные.

### 16.3 Согласие с аудитором — что верно и почему

#### Data-code gap — действительно одна из главных проблем

Это очень серьёзно. Если `.ron`-конфиги существуют, но реальные значения захардкожены в Rust, то проект теряет сразу три вещи: **балансируемость, проверяемость, предсказуемость**. Данные есть "для красоты" или для будущего, а правда живёт в коде. Такая архитектура быстро начинает врать самой себе.

#### Single-threaded tick при готовой инфраструктуре — болезненно

Классическая ситуация: "мы построили космодром, но летаем на велосипеде". Если есть `WorkerPool`, `FrameGraph`, `tick_parallel()`, но основной runtime идёт последовательно, то:

- либо параллельный режим нестабилен,
- либо он не доведён,
- либо команда боится nondeterminism и гонок.

В любом случае это один из главных резервов.

#### Monolithic main loop — плохой знак зрелости

`main.rs`, который вручную оркестрирует полмира, — это запах. Не катастрофа, но запах. Чем толще main, тем меньше реально завершена системная композиция.

#### Inline shaders — очень неудобно

Не смертельно, но для реального рендерного пайплайна — боль. Итерация по шейдерам без hot reload — инженерная форма самоистязания.

#### Networking — пока не production

Наличие сетевого модуля и наличие сетевой архитектуры — это **разные биологические виды**.

#### Typed ECS queries действительно нужны

Когда все системы вручную лазят в `ecs.transforms`, `ecs.ai_states` и так далее — это быстро превращается в лес ручных зависимостей и хрупких доступов.

### 16.4 Где стоит быть осторожнее, чем аудитор

#### "ECS не generational" — не автоматически приговор

Если есть ручной generation tracking через slotmap-подобный подход, то вопрос не в названии паттерна, а в том:

- насколько надёжно предотвращаются stale entity handles,
- есть ли reuse safety,
- как ведут себя despawn/spawn под нагрузкой,
- есть ли тесты на invalid access.

Сам по себе этот пункт — не "ущербность", а **"место, где легко накосячить"**.

#### Origin shift при мире 2000 м и пороге 5000 м — не обязательно проблема

Это может быть: задел на будущий масштаб, общий код для sandbox и больших карт, просто унифицированный world-space policy. Не баг, а либо недоиспользованный механизм, либо инфраструктурный аванс в будущее.

#### Отсутствие deferred shading — не обязательно архитектурный провал

Forward+ или даже хорошо сделанный forward pipeline иногда нормален, особенно если источников света не тысячи, есть culling, есть clustered/tiled path или зачатки, упор на stylized/controlled scene budgets.

"Нет deferred" ≠ автоматически плохо. Плохо, если целитесь в тяжёлую ночную сцену S.T.A.L.K.E.R.-типа с тонной локальных источников, volumetrics, дождём, мокрыми материалами и кучей динамических объектов. Тогда forward-only может начать хрипеть.

#### Stub-файлы — сами по себе не ужас

Пустые `gpu_jobs.rs`, `memory_manager.rs` и т.д. — не катастрофа. Вопрос: они честно помечены как phase placeholders или создают ложное впечатление завершённости? Если второе — плохо. Если первое — нормально.

---

## 17. ENGINE PURITY ASSESSMENT

### 17.1 Проблема: engine vs game coupling

Судя по аудиту, ENGENE пока не совсем "движок", а **движок-с-очень-конкретной-игрой-внутри**. Game-specific logic сидит глубоко:

- Экономика (jobs, desperation, banditry)
- Экосистема (wolves, boars, bloodsuckers)
- Population (NPC names, species caps)
- AI goals (Hunt, StealOrRob, Mate, DefendTerritory)
- Монстры (Wolf/Boar/Bloodsucker hardcoded в компонентах)
- Фракции (Loners, Duty, Freedom, Bandits)
- Survival-логика (hunger, thirst, sleep, fear)

### 17.2 Классификация компонентов по чистоте

| Слой | Truly Engine-Agnostic | Game-Specific |
|------|----------------------|---------------|
| **Core** | ECS, Events, Jobs, Registry, Time, Perf, Commands, Replay, Determinism, BuildManifest, CrashTelemetry | — |
| **Graphics** | Renderer, PBR, Shadows, Particles, LOD, GPU Culling, Terrain, Sky/Weather | Gore mesh, vegetation semantics |
| **Physics** | RapierPhysics, Cloth, Water, Fire, Ballistics, Destruction, Damage Pipeline | Tile wall fracture (STALKER-specific), body zones |
| **AI** | — | **Всё** (desire, emotions, memory, perception, combat, reproduction, monster_ai, npc_ai, thresholds) |
| **World** | Heightmap, Streaming, Spatial Index, Origin Shift, Surface DB | Biomes (Forest/Swamp/etc.), Resource Grid (food/water/carcass), Components (NpcTraits, MonsterSpecies, Job, Goal) |
| **Navigation** | Navmesh, HPA*, RVO, Path Cache, World Graph | Cover map (tactical), breach analysis |
| **Simulation** | Simulation levels (L0/L1/L2) | Camp simulation, social propagation, milestones |
| **Economy** | — | **Всё** (jobs, trading, desperation, items, traders) |
| **Ecosystem** | — | **Всё** (food chain, territory, migration) |
| **Audio** | AudioEngine, SoundBank, Occlusion, Spatial | Biome ambience |
| **Body** | Anatomy framework, death pipeline | Zone-specific damage (humanoid-only), gore |
| **Animation** | Skeleton, Locomotion, Foot IK, Ragdoll | Injury states |
| **Networking** | Protocol, Server/Client, Interpolation | — |
| **Tooling** | Editor shell, Console, Profiler, Inspector, Doctor | Economy dashboard, sim metrics, quest board |

### 17.3 Вердикт

Проект может быть **очень хорошим движком для одной игры**, но **слабой общей платформой для разных игр**. Это не недостаток, если цель — сделать одну игру. Это недостаток, если кто-то всерьёз считает, что это "универсальный engine".

### 17.4 Рекомендации по чистоте

Что надо вынести в plugins / data packs для отделения engine от game:

1. Вся AI-логика (desire, thresholds, combat, reproduction) → `game::ai_pack`
2. Экономика → `game::economy_pack`
3. Экосистема → `game::ecosystem_pack`
4. Конкретные виды монстров → data-driven species definitions
5. Фракции → configurable, не hardcoded enum
6. World components → split на engine components (Transform, SimLevel) и game components (NpcTraits, Goal, Inventory)

---

## 18. FEATURE MATURITY MATRIX

Для каждой подсистемы — статус по модели реальной зрелости.

| Подсистема | Code Exists | Enabled in Runtime | Tested | Used by Default | Production-Ready |
|------------|:-----------:|:------------------:|:------:|:---------------:|:----------------:|
| **ECS** | ✅ | ✅ | ⚠️ integration only | ✅ | ⚠️ no query API |
| **Engine tick loop** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Event system** | ✅ | ✅ | ⚠️ | ✅ | ✅ |
| **Job system** | ✅ | ❌ not used | ❌ | ❌ | ❌ |
| **Perf budgets** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| **Replay system** | ✅ | ⚠️ partial | ⚠️ | ❌ not default | ❌ |
| **Determinism** | ✅ | ⚠️ audit only | ⚠️ | ❌ | ❌ |
| **Crash telemetry** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| **PBR Rendering** | ✅ | ✅ | ✅ | ✅ | ⚠️ forward-only |
| **Shadow Maps** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Sky/Weather** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ complex |
| **Particles** | ✅ | ❌ not in render loop | ❌ | ❌ | ❌ |
| **GPU Culling** | ✅ | ⚠️ | ⚠️ | ⚠️ | ⚠️ |
| **TAA** | ✅ | ⚠️ partial | ⚠️ | ⚠️ | ❌ |
| **SSAO** | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Deferred Shading** | ❌ stub | ❌ | ❌ | ❌ | ❌ |
| **Skinned Mesh** | ✅ | ⚠️ partial | ⚠️ | ⚠️ | ❌ |
| **Rapier3D physics** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| **Fire simulation** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| **Water simulation** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| **Cloth simulation** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| **Ballistics** | ✅ | ✅ | ✅ | ✅ (sandbox) | ⚠️ |
| **Destruction** | ✅ | ✅ | ✅ | ✅ (sandbox) | ⚠️ |
| **Damage pipeline** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| **AI (NPC)** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| **AI (Monster)** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| **Combat tactics** | ✅ | ⚠️ integration_systems | ⚠️ | ⚠️ | ❌ |
| **Navigation** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| **World streaming** | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| **LOD simulation** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| **Save/Load** | ✅ | ✅ | ✅ | ✅ | ⚠️ schema v1 |
| **Audio** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ rodio optional |
| **Input** | ✅ | ✅ | ⚠️ | ✅ | ✅ |
| **Networking** | ✅ | ❌ stub | ❌ | ❌ | ❌ |
| **Animation** | ✅ | ⚠️ partial | ⚠️ | ⚠️ | ❌ |
| **Economy** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| **Ecosystem** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| **Quests** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| **Content pipeline** | ✅ | ⚠️ | ✅ | ⚠️ | ❌ |
| **SDK/Editor** | ✅ | ✅ | ⚠️ | ✅ (sdk binary) | ⚠️ |
| **Memory management** | ❌ stub | ❌ | ❌ | ❌ | ❌ |
| **Body system** | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |

**Легенда:** ✅ = да, ⚠️ = частично / с оговорками, ❌ = нет

**Вывод:** Из ~40 подсистем только ~10 можно считать production-ready. Ещё ~20 в состоянии "работает, но с оговорками". ~10 — stubs или не интегрированы.

---

## 19. TESTABILITY & DEBUGGING ASSESSMENT

### 19.1 Что есть

| Инструмент | Статус | Описание |
|------------|--------|----------|
| Integration tests | ✅ 25 файлов | Покрывают основные подсистемы |
| Benchmarks | ✅ 10 штук | core, content, world, AI, graphics |
| Doctor diagnostics | ✅ | Runtime health check |
| Profiler dashboard | ✅ | Frame timing, per-system timing |
| Event monitor | ✅ | Event bus logging |
| Crash telemetry | ✅ | Panic hook + crash bundles |
| Replay system | ⚠️ | Recorder/Player есть, checkpoint есть, divergence detection есть |
| Closed-loop validation | ✅ | `ClosedLoopRecorder` + validation |
| Stress test | ✅ | `run_meat_grinder` |
| Streaming harness | ✅ | Multi-region test, golden scenarios |
| Console | ✅ | 16 built-in commands |

### 19.2 Что отсутствует — критические вопросы

| Вопрос | Ответ | Риск |
|--------|-------|------|
| Можно ли воспроизводить баги детерминированно? | ⚠️ Replay + determinism infra есть, но tick не детерминирован (single-thread only, parallel untested) | MEDIUM |
| Есть ли golden tests для симуляции? | ⚠️ `golden_scenarios` в streaming_harness, но не для полной симуляции | MEDIUM |
| Есть ли snapshot tests для data configs? | ❌ Не обнаружены | LOW |
| Есть ли render regression tests? | ❌ Нет screenshot comparison | MEDIUM |
| Есть ли fuzzing для content pipeline / save-load / network packets? | ❌ Не обнаружен | HIGH для сети |
| Есть ли soak tests на 1–6 часов headless runtime? | ⚠️ `--months N` в headless binary, `SimTestTier::Soak` = 864_000 тиков | Формально есть |
| Unit-тесты внутри модулей? | ❌ Не обнаружены при аудите | MEDIUM |

### 19.3 Рекомендации

Для survival/open-world движка отсутствие fuzzing, render regression tests и полноценных golden tests — это **не роскошь, а бомба замедленного действия**. Сложность начнёт жрать саму себя.

---

## 20. API BOUNDARY STABILITY ANALYSIS

### 20.1 Текущее состояние

| Граница | Стабильность | Blast Radius изменений |
|---------|-------------|----------------------|
| `core::ecs::Ecs` (struct fields) | **Низкая** — поля `pub`, добавление компонента = изменение struct | **Огромный** — ломает все системы |
| `world::components` | **Низкая** — монолитный файл, 30+ типов | **Огромный** — enum variant change ломает AI, physics, economy |
| `core::engine::Engine` (public API) | **Средняя** — `tick()`, `init()`, `shutdown()` стабильны | **Средний** |
| `core::events::EventBus` | **Высокая** — generic API | **Низкий** |
| `core::registry::Resources` | **Высокая** — type-erased API | **Низкий** |
| `graphics::renderer::Renderer` | **Средняя** | **Средний** — API стабилен, но добавление pass = внутренние изменения |
| `physics::damage_pipeline` | **Средняя** — через enums | **Средний** — добавление DamageResponse variant = cascade |
| `ai::desire` / `ai::plan` | **Низкая** — завязана на Goal enum | **Высокий** — новый Goal требует изменений в desire, plan, npc_ai, monster_ai |

### 20.2 Вывод

Coupling высоковат. Это значит:

- **Рефакторинг дорогой** — изменение одного enum может потребовать правок в 10+ файлах
- **Параллельная работа команды усложняется** — merge conflicts на горячих файлах
- **Внедрение новых людей будет медленнее** — нужно знать всю паутину зависимостей

---

## 21. MAINTENANCE BURDEN & BUS FACTOR

### 21.1 Стоимость сопровождения

Движок имеет риск стать **"инженерным дворцом с налогом на владение"**.

Признаки:

- Много подсистем (~22 модуля)
- Много ambition-heavy features
- Кастомный ECS
- Кастомная sim logic
- Кастомные destruction layers
- Полный toolchain
- Editor на 23+ панели
- Shaders inline
- Partly implemented networking

Каждое такое решение — **подписка на поддержку**.

### 21.2 Оценка по модулям

| Модуль | Сложность сопровождения | Причина |
|--------|------------------------|---------|
| `graphics/sky/` (30 файлов) | **Очень высокая** | Volumetric clouds, atmosphere LUTs, weather FSM — глубокая графика |
| `physics/damage_pipeline/` | **Высокая** | 4 resolver chain, 16+ response types, cross-system effects |
| `ai/` (27 файлов) | **Высокая** | Emotions, memory, perception, desire, combat — тесно связанная паутина |
| `core/engine.rs` | **Высокая** | Центральная точка, любое изменение фаз/порядка = каскад |
| `graphics/renderer.rs` | **Высокая** | 10+ render passes inline, все шейдеры тут |
| `world/components.rs` | **Средняя** | Простой код, но высокий blast radius |
| `tools/` (23 файла) | **Средняя** | Каждая панель независима, но editor_shell = оркестратор |
| `content/` | **Низкая** | Standalone, хорошие границы |
| `input/` | **Низкая** | 3 файла, минимальный scope |

### 21.3 Bus Factor

Если за каждый модуль отвечает один человек (или один человек отвечает за всё), то:

- **graphics/sky/** — bus factor = 1 (специфическая экспертиза в atmospheric rendering)
- **physics/damage_pipeline** — bus factor = 1
- **core/engine.rs** + **core/ecs.rs** — bus factor = 1 (архитектор движка)

### 21.4 Минимальная команда для устойчивого сопровождения

| Роль | Scope | Минимум |
|------|-------|---------|
| Engine architect | core, engine, ECS, jobs, events | 1 |
| Graphics engineer | graphics/, sky/, shaders | 1 |
| Systems programmer | physics, body, navigation, animation | 1 |
| Gameplay/AI programmer | ai, economy, ecosystem, simulation, gameplay | 1 |
| Tools/SDK engineer | tools, content, editor | 0.5 |
| QA / test automation | tests, benchmarks, CI | 0.5 |
| **Итого** | | **~5 человек** |

При команде меньше 3 человек проект будет неизбежно накапливать технический долг быстрее, чем его можно погашать.

---

## 22. ARCHITECTURAL PHILOSOPHY & PRODUCT CORE

### 22.1 Проблема множественной идентичности

Судя по аудиту, движок одновременно пытается быть:

- ECS engine
- Simulation engine
- Rendering engine
- AI sandbox
- Editor platform
- Networking base
- Survival framework

Это опасно. Не потому что нельзя. А потому что **без жёсткой иерархии приоритетов движок начинает распухать**.

### 22.2 Главный вопрос

**Что является продуктовым ядром ENGENE?**

| Кандидат | Аргументы за | Аргументы против |
|----------|-------------|-----------------|
| Симуляция мира | L0/L1/L2, ecosystem, economy, AI | Завязана на конкретную survival-игру |
| Immersive rendering | PBR, sky, weather, destruction | Forward-only, particles не подключены, shaders inline |
| Survival systems | AI, economy, needs, factions, quests | Hardcoded species и goals |
| Editor/SDK | 23 панели, doctor, profiler | Завязан на конкретный движок |
| Sandbox/R&D | Destruction sandbox, headless testing | Не масштабируется в production pipeline |

**Без ответа на этот вопрос даже хороший движок начинает расползаться.**

Рекомендация: определить **1-2 главных differentiator** и сфокусировать на них. Остальное — поддерживать, но не развивать вширь.

---

## 23. TRUE WEAKNESSES (5 CORE ISSUES)

Если отбросить театральные спецэффекты, вот 5 настоящих слабостей:

### W1. Несоответствие между заявленной полнотой и фактической интеграцией

**Самая опасная штука.** По бумаге всё есть, а по execution path — часть систем наполовину подключена. Particles есть, но не в render loop. Networking есть, но stub. Parallel tick есть, но не задействован. Data configs есть, но не грузятся как source of truth. Это рождает **иллюзию зрелости**.

### W2. Слишком высокая доменная связность

`world::components`, прямой доступ к ECS, толстый main loop, тесная спайка AI/economy/world — всё это делает **рефакторинг дорогим**.

### W3. Недостаточная data-driven дисциплина

Если баланс, лимиты, материалы и population truth живут partly in code, partly in `.ron`, система начинает **раздваивать личность**.

### W4. Недореализованная эксплуатационная многопоточность

Если проект такого размера крутится по сути последовательно, то это сильно бьёт по **масштабируемости**.

### W5. Опасность "feature garden"

Слишком много систем на разной стадии готовности. Это может стать **кладбищем почти-готовых подсистем**.

---

## 24. TRUE STRENGTHS (5 CORE ASSETS)

### S1. Есть инженерный каркас

Фазы тика, assembly, ресурсы, plugins, crash telemetry, replay, perf budgets — это всё признаки не хаоса, а **архитектурного мышления**.

### S2. Есть toolability

SDK/editor на 23 панели — **очень серьёзная заявка**. Много самодельных движков умирают именно потому, что без инструментария они непригодны для жизни.

### S3. Есть ориентация на runtime reality

LOD, world streaming, budgets, low-spec profile, headless — признаки того, что кто-то хотя бы иногда смотрел в бездну производительности, а не только рисовал красивые блок-схемы.

### S4. Есть попытка системной симуляции, а не скриптовой магии

AI, ecology, economy, body, destruction — это тяжёлый путь, но он **даёт глубину**.

### S5. Есть хороший потенциал vertical slice / sandbox proof

Для прототипирования и внутреннего R&D это уже **весьма мясистая база**.

---

## 25. EXTENDED RECOMMENDATIONS & ROADMAP

### 25.1 Приоритеты аудитора (подтверждённые)

**P0 — делать немедленно:**

- Сделать все конфиги реально data-driven
- Включить / довести parallel tick

**P1 — делать следом:**

- Нормализовать ECS query layer
- Вынести orchestration из `main.rs`
- Добить тесты и интеграцию незавершённых runtime paths

**P2 — делать планово:**

- Hot reload для шейдеров
- Deferred / lighting strategy
- Memory manager
- Networking scope

### 25.2 Расширенный roadmap (5 этапов)

#### Этап 1 — Убрать ложную сложность

- Провести feature maturity audit (см. раздел 18)
- Пометить всё как `shipping` / `experimental` / `stub` / `dormant`
- Убрать иллюзию, что "всё уже есть"
- Для каждой подсистемы ввести статус: `production`, `experimental`, `partial`, `planned`, `deprecated`

**Такая честность экономит месяцы жизни и килограммы нервов.**

#### Этап 2 — Восстановить единый источник правды

Все следующие параметры **должны иметь один source of truth** (в `.ron`):

- Популяционные лимиты
- Радиусы LOD (L0/L1/L2)
- Job incomes
- Materials / surfaces
- AI thresholds
- Economy rules
- Species definitions
- Weapon stats

Rust-код должен **только читать**, не дублировать.

#### Этап 3 — Ужесточить границы ядра

- Engine core — отдельно (ECS, events, jobs, time, perf, registry)
- Game-specific logic — отдельно (AI goals, economy, ecosystem, factions)
- Survival pack — отдельно (needs, inventory, traits)
- AI pack — отдельно (desire, perception, memory, combat)

#### Этап 4 — Добить runtime execution model

- Либо честно включить parallel tick
- Либо признать его экспериментом и не держать как фантомную мощь
- Если включить — обеспечить determinism gate (replay comparison)
- Stress test parallel mode under 200+ entities

#### Этап 5 — Architecture honesty

Для каждой подсистемы вести явный статус в документации:

| Статус | Значение |
|--------|----------|
| `production` | Работает, протестировано, используется в shipping path |
| `experimental` | Работает, но не протестировано под нагрузкой / не в default path |
| `partial` | Частично реализовано, есть known gaps |
| `planned` | Placeholder / stub, код есть но не функционирует |
| `deprecated` | Мёртвый код, кандидат на удаление |

### 25.3 Финальная фраза

> Это не ущербный движок. Это движок с хорошим скелетом, сильным R&D-геном и заметной склонностью к архитектурному разрастанию раньше полной эксплуатационной зрелости.

---

*Документ сгенерирован на основе полного аудита кодовой базы (~8300 файлов) и экспертного анализа.*
*Пробелы в данных явно помечены в разделе 15 (Gaps & Assumptions).*
*Для любых уточнений по конкретным модулям, файлам или переменным — обращайтесь к соответствующим разделам или запрашивайте детализированный аудит.*
