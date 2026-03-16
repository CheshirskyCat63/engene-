//! Phase 9: Canonical Runtime Assembly
//! Builds the canonical engine configurations for VerticalSlice, Headless, and Sandbox.

use std::collections::HashMap;
use std::sync::Arc;

use crate::ai::ai::AiSystem;
use crate::ai::combat_tactics::tactics::TacticProfile;
use crate::audio::audio::AudioEngine;
use crate::core::component_registry::ComponentRegistry;
use crate::core::config::{load_config, ConfigEnvelope};
use crate::core::ecs::Ecs;
use crate::core::engine::Engine;
use crate::core::material_truth::MaterialTruthService;
use crate::core::plugin::EngineBuilder;
use crate::core::runtime_config::RuntimeProfile;
use crate::game::StalkerPlugin;
use crate::game::weapons_plugin::WeaponsPlugin;
use crate::memory::asset_manager::AssetManager;
use crate::navigation::cover_map::CoverMap;
use crate::navigation::dynamic_nav_update::NavDirtyTracker;
use crate::navigation::hpa_star::HpaGraph;
use crate::physics::destruction::DestructionSystem;
use crate::physics::physics::PhysicsSystem;
use crate::core::events::sim_bus::SimBus;
use crate::core::events::render_bus::RenderBus;
use crate::core::events::debug_bus::DebugBus;
use crate::core::events::sticky::StickyEvents;
use crate::core::events::tracing_hooks::EventTracer;
use crate::core::events::aggregation::EventAggregator;
use crate::core::budget_registry::create_default_registry;
use crate::core::quality_governor::QualityGovernor;
use crate::core::runtime_config::RuntimeConfig;
use crate::core::ownership_map::OwnershipMap;
use crate::core::world_state_authority;
use crate::core::sdk::EngineSDK;
use crate::world::terrain_truth::TerrainTruth;
use crate::world::surface_state::SurfaceStateStore;
use crate::physics::damage_pipeline::DamageOrchestrator;
use crate::content::prefabs::prefab_registry::PrefabRegistry;
use crate::content::asset_budget::AssetBudget;
use crate::world::authoring::WorldAuthoringDatabase;
use crate::world::chunk_persistence::ChunkPersistenceService;
use crate::game::combat_plugin::CombatPlugin;
use crate::game::ai_config::AiConfigPlugin;
use crate::game::economy_plugin::EconomyPlugin;
use crate::game::population_plugin::PopulationPlugin;
use crate::simulation::simulation::SimulationSystem;
use crate::simulation::world_tick::WorldTickSystem;
use crate::world::cell::WORLD_SIZE;
use crate::world::fields::{AnomalyForceType, AnomalyZone, WorldFields};
use crate::world::heightmap::Heightmap;
use crate::world::hierarchical_spatial::HierarchicalSpatialIndex;
use crate::world::origin_shift::OriginShift;
use crate::world::resources::ResourceGrid;
use crate::world::streaming::WorldStreamer;
use crate::world::terrain_deformation::TerrainDeformationSystem;
use crate::world::world::WorldGrid;

pub struct RuntimeAssembly {
    pub config: RuntimeConfig,
}

impl RuntimeAssembly {
    /// Builds the canonical VerticalSlice runtime with ALL systems registered.
    /// This is THE integration target.
    pub fn vertical_slice(
        heightmap: Arc<Heightmap>,
        biomes: &[crate::world::biome::Biome],
    ) -> Engine {
        let grid = WorldGrid::generate();
        let resource_grid = ResourceGrid::new(biomes);
        let authority_entries = world_state_authority::authority_matrix();

        let mut builder = EngineBuilder::new();
        builder.insert_resource(resource_grid);

        let mut world_fields = WorldFields::new();
        world_fields.anomaly.zones.push(AnomalyZone {
            center: glam::Vec3::new(WORLD_SIZE * 0.7, 0.0, WORLD_SIZE * 0.3),
            radius: 80.0,
            force_strength: 15.0,
            force_type: AnomalyForceType::Vortex,
        });
        builder.insert_resource(world_fields);
        builder.insert_resource(DestructionSystem::new());
        builder.insert_resource(TerrainDeformationSystem::new());

        builder.add_plugin(StalkerPlugin::from_config("game/data"));
        builder.add_plugin(WeaponsPlugin::new("game/data"));

        {
            let tactics: HashMap<String, TacticProfile> = load_config::<
                ConfigEnvelope<HashMap<String, TacticProfile>>,
            >("game/data/tactics.ron")
            .map(|e| e.data)
            .unwrap_or_default();
            builder.insert_resource(tactics);
        }

        {
            let hm_ref = heightmap.clone();
            let cover_map = CoverMap::precompute(
                crate::world::cell::WORLD_SIZE,
                20.0,
                &|x, z| hm_ref.sample(x, z),
            );
            builder.insert_resource(cover_map);
        }

        builder.insert_resource(heightmap.clone());
        builder.insert_resource(WorldStreamer::new(3000.0, 4000.0));
        builder.insert_resource(std::sync::Mutex::new(AssetManager::new()));
        builder.insert_resource(AudioEngine::new());
        builder.insert_resource(HierarchicalSpatialIndex::new());
        builder.insert_resource(HpaGraph::build());
        builder.insert_resource(OriginShift::new());
        builder.insert_resource(NavDirtyTracker::new());
        builder.insert_resource(crate::graphics::destruction_occlusion::DestructionOcclusionSystem::new());
        builder.insert_resource(crate::graphics::gore_mesh::GoreMeshSystem::new(256));
        builder.insert_resource(ComponentRegistry::default_registry());
        builder.insert_resource(MaterialTruthService::empty());
        builder.insert_resource(crate::graphics::surface_state_render::SurfaceStateRenderSystem::new());

        // Wired dormant systems
        builder.insert_resource(authority_entries);
        builder.insert_resource(OwnershipMap::new());
        builder.insert_resource(SimBus::new());
        builder.insert_resource(RenderBus::new());
        builder.insert_resource(DebugBus::new());
        builder.insert_resource(StickyEvents::new());
        let mut tracer = EventTracer::new();
        tracer.enable();
        builder.insert_resource(tracer);
        builder.insert_resource(EventAggregator::new(10.0));
        builder.insert_resource(RuntimeConfig::default());
        builder.insert_resource(QualityGovernor::new(60));
        builder.insert_resource(create_default_registry());
        builder.insert_resource(TerrainTruth::grassland());
        builder.insert_resource(SurfaceStateStore::new());
        builder.insert_resource(DamageOrchestrator::new());
        builder.insert_resource(PrefabRegistry::new());
        builder.insert_resource(AssetBudget::default());
        builder.insert_resource(EngineSDK::new());

        let center = crate::world::cell::WORLD_SIZE * 0.5;
        builder.add_system_default(Box::new(SimulationSystem::new(center, center)));
        builder.add_system_default(Box::new(WorldTickSystem::new(grid)));
        builder.add_system_default(Box::new(AiSystem::new()));
        builder.add_system_default(Box::new(crate::core::integration_systems::AiDecisionWireSystem));
        builder.add_system_default(Box::new(PhysicsSystem::new(heightmap)));
        builder.add_system_default(Box::new(crate::core::integration_systems::BallisticsTickSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::DamageDispatchSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::DestructionTickSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::TerrainDeformationTickSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::NavDirtyTickSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::OcclusionWireSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::GoreWireSystem));
        builder.add_system_default(Box::new(
            crate::animation::animation_integration::AnimationIntegrationSystem::new(),
        ));
        builder.add_system_default(Box::new(
            crate::core::integration_systems::AnimationWireSystem::new(),
        ));
        builder.add_system_default(Box::new(
            crate::audio::audio_integration::AudioIntegrationSystem::new(),
        ));
        builder.add_system_default(Box::new(
            crate::gameplay::quest_system::QuestSystem::new(),
        ));
        builder.add_system_default(Box::new(crate::body::body_system::BodySystem));
        builder.add_system_default(Box::new(
            crate::graphics::render_system::RenderSystem,
        ));
        builder.add_system_default(Box::new(
            crate::audio::playback::AudioPlaybackBridge::new(),
        ));
        builder.add_system_default(Box::new(
            crate::input::input_system::InputActionSystem::new(),
        ));
        #[cfg(feature = "networking")]
        builder.add_system_default(Box::new(
            crate::network::network_system::NetworkSystem::new(),
        ));

        // All game plugins go through the public Plugin API (L2 dogfooding)
        builder.add_plugin(CombatPlugin);
        builder.add_plugin(AiConfigPlugin);
        builder.add_plugin(EconomyPlugin);
        builder.add_plugin(PopulationPlugin);

        builder.insert_resource(crate::economy::item_registry::ItemRegistry::new());
        builder.insert_resource(Vec::<crate::simulation::camp_simulation::CampState>::new());
        builder.insert_resource(crate::audio::sound_bank::SoundBank::new());
        builder.insert_resource(crate::core::perf::perf_budget::PerfBudgetManager::new(60));
        builder.insert_resource(crate::simulation::world_milestones::WorldMilestoneTracker::new());
        builder.insert_resource(crate::body::death_pipeline::CorpseManager::new());
        builder.insert_resource(crate::animation::clip_map::ClipMap::new());
        builder.insert_resource(crate::animation::animation_ladder::AnimationLadder::default());
        builder.insert_resource(crate::gameplay::factions::FactionRelations::new());
        builder.insert_resource(crate::core::determinism_policy::DeterminismPolicyMatrix::build_default());
        builder.insert_resource(crate::core::build_manifest::SchemaMigrationRegistry::new());
        builder.insert_resource(crate::tools::sim_metrics_dashboard::SimMetricsDashboard::default());
        builder.insert_resource(crate::core::perf::sim_telemetry::SimTelemetry::new());
        builder.insert_resource(crate::core::dirty_set::ChunkDirtySet::default());
        builder.insert_resource(crate::core::dirty_set::NavDirtySet::default());
        builder.insert_resource(crate::core::dirty_set::EntityDirtySet::default());
        builder.insert_resource(crate::core::perf::low_spec_cert::LowSpecCertifier::default());

        let mut ecs = Ecs::new();

        let (systems, resources) = builder.build(&mut ecs);
        let mut engine = Engine::from_builder(systems, resources, ecs);
        engine.init();
        engine
    }

    /// Headless mode -- no renderer, no audio, pure simulation.
    /// Same as vertical_slice but omits renderer-only resources.
    pub fn headless(biomes: &[crate::world::biome::Biome]) -> Engine {
        let grid = WorldGrid::generate();
        let resource_grid = ResourceGrid::new(biomes);
        let heightmap = Arc::new(Heightmap::generate(biomes));

        let authority_entries = world_state_authority::authority_matrix();

        let mut builder = EngineBuilder::new();
        builder.insert_resource(resource_grid);

        let mut world_fields = WorldFields::new();
        world_fields.anomaly.zones.push(AnomalyZone {
            center: glam::Vec3::new(WORLD_SIZE * 0.7, 0.0, WORLD_SIZE * 0.3),
            radius: 80.0,
            force_strength: 15.0,
            force_type: AnomalyForceType::Vortex,
        });
        builder.insert_resource(world_fields);
        builder.insert_resource(DestructionSystem::new());
        builder.insert_resource(TerrainDeformationSystem::new());

        builder.add_plugin(StalkerPlugin::from_config("game/data"));
        builder.add_plugin(WeaponsPlugin::new("game/data"));

        {
            let tactics: HashMap<String, TacticProfile> = load_config::<
                ConfigEnvelope<HashMap<String, TacticProfile>>,
            >("game/data/tactics.ron")
            .map(|e| e.data)
            .unwrap_or_default();
            builder.insert_resource(tactics);
        }

        {
            let hm_ref = heightmap.clone();
            let cover_map = CoverMap::precompute(
                crate::world::cell::WORLD_SIZE,
                20.0,
                &|x, z| hm_ref.sample(x, z),
            );
            builder.insert_resource(cover_map);
        }

        builder.insert_resource(heightmap.clone());
        builder.insert_resource(WorldStreamer::new(3000.0, 4000.0));
        builder.insert_resource(std::sync::Mutex::new(AssetManager::new()));
        builder.insert_resource(AudioEngine::new());
        builder.insert_resource(HierarchicalSpatialIndex::new());
        builder.insert_resource(HpaGraph::build());
        builder.insert_resource(OriginShift::new());
        builder.insert_resource(NavDirtyTracker::new());
        builder.insert_resource(crate::graphics::destruction_occlusion::DestructionOcclusionSystem::new());
        builder.insert_resource(crate::graphics::gore_mesh::GoreMeshSystem::new(256));
        builder.insert_resource(ComponentRegistry::default_registry());
        builder.insert_resource(MaterialTruthService::empty());
        builder.insert_resource(crate::graphics::surface_state_render::SurfaceStateRenderSystem::new());

        // Wired dormant systems
        builder.insert_resource(authority_entries);
        builder.insert_resource(OwnershipMap::new());
        builder.insert_resource(SimBus::new());
        builder.insert_resource(RenderBus::new());
        builder.insert_resource(DebugBus::new());
        builder.insert_resource(StickyEvents::new());
        let mut tracer = EventTracer::new();
        tracer.enable();
        builder.insert_resource(tracer);
        builder.insert_resource(EventAggregator::new(10.0));
        builder.insert_resource(RuntimeConfig::default());
        builder.insert_resource(QualityGovernor::new(60));
        builder.insert_resource(create_default_registry());
        builder.insert_resource(TerrainTruth::grassland());
        builder.insert_resource(SurfaceStateStore::new());
        builder.insert_resource(DamageOrchestrator::new());
        builder.insert_resource(PrefabRegistry::new());
        builder.insert_resource(AssetBudget::default());
        builder.insert_resource(EngineSDK::new());

        let center = crate::world::cell::WORLD_SIZE * 0.5;
        builder.add_system_default(Box::new(SimulationSystem::new(center, center)));
        builder.add_system_default(Box::new(WorldTickSystem::new(grid)));
        builder.add_system_default(Box::new(AiSystem::new()));
        builder.add_system_default(Box::new(crate::core::integration_systems::AiDecisionWireSystem));
        builder.add_system_default(Box::new(PhysicsSystem::new(heightmap)));
        builder.add_system_default(Box::new(crate::core::integration_systems::BallisticsTickSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::DamageDispatchSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::DestructionTickSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::TerrainDeformationTickSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::NavDirtyTickSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::OcclusionWireSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::GoreWireSystem));
        builder.add_system_default(Box::new(
            crate::animation::animation_integration::AnimationIntegrationSystem::new(),
        ));
        builder.add_system_default(Box::new(
            crate::core::integration_systems::AnimationWireSystem::new(),
        ));
        builder.add_system_default(Box::new(
            crate::audio::audio_integration::AudioIntegrationSystem::new(),
        ));
        builder.add_system_default(Box::new(
            crate::gameplay::quest_system::QuestSystem::new(),
        ));
        builder.add_system_default(Box::new(crate::body::body_system::BodySystem));
        #[cfg(feature = "networking")]
        builder.add_system_default(Box::new(
            crate::network::network_system::NetworkSystem::new(),
        ));
        builder.add_system_default(Box::new(
            crate::input::input_system::InputActionSystem::new(),
        ));

        // All game plugins go through the public Plugin API (L2 dogfooding)
        builder.add_plugin(CombatPlugin);
        builder.add_plugin(AiConfigPlugin);
        builder.add_plugin(EconomyPlugin);
        builder.add_plugin(PopulationPlugin);

        builder.insert_resource(crate::economy::item_registry::ItemRegistry::new());
        builder.insert_resource(Vec::<crate::simulation::camp_simulation::CampState>::new());
        builder.insert_resource(crate::audio::sound_bank::SoundBank::new());
        builder.insert_resource(crate::core::perf::perf_budget::PerfBudgetManager::new(60));
        builder.insert_resource(crate::simulation::world_milestones::WorldMilestoneTracker::new());
        builder.insert_resource(crate::body::death_pipeline::CorpseManager::new());
        builder.insert_resource(crate::animation::clip_map::ClipMap::new());
        builder.insert_resource(crate::animation::animation_ladder::AnimationLadder::default());
        builder.insert_resource(crate::gameplay::factions::FactionRelations::new());
        builder.insert_resource(crate::core::determinism_policy::DeterminismPolicyMatrix::build_default());
        builder.insert_resource(crate::core::build_manifest::SchemaMigrationRegistry::new());
        builder.insert_resource(crate::core::perf::sim_telemetry::SimTelemetry::new());
        builder.insert_resource(crate::core::dirty_set::ChunkDirtySet::default());
        builder.insert_resource(crate::core::dirty_set::NavDirtySet::default());
        builder.insert_resource(crate::core::dirty_set::EntityDirtySet::default());
        builder.insert_resource(crate::core::perf::low_spec_cert::LowSpecCertifier::default());

        let mut ecs = Ecs::new();

        let (systems, resources) = builder.build(&mut ecs);
        let mut engine = Engine::from_builder(systems, resources, ecs);
        engine.init();
        engine
    }

    /// Destruction Sandbox 50x50 — full physics/render/audio runtime for the
    /// sandbox proving ground.  Based on vertical_slice but stripped of
    /// AI/economy/quest/network systems that are irrelevant for destruction
    /// testing.  Uses micro-streaming (1-4 chunks) so that the persistence
    /// path is exercised the same way as the full world.
    pub fn sandbox_50x50() -> Engine {
        let sandbox_size = 50.0_f32;

        let grid = WorldGrid::generate();
        let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
        let heightmap = Arc::new(Heightmap::flat(sandbox_size));
        let resource_grid = ResourceGrid::new(&biomes);
        let authority_entries = world_state_authority::authority_matrix();

        let mut builder = EngineBuilder::new();
        builder.insert_resource(resource_grid);
        builder.insert_resource(WorldFields::new());
        builder.insert_resource(DestructionSystem::new());
        builder.insert_resource(TerrainDeformationSystem::new());

        builder.add_plugin(WeaponsPlugin::new("game/data"));

        {
            let tactics: HashMap<String, TacticProfile> = load_config::<
                ConfigEnvelope<HashMap<String, TacticProfile>>,
            >("game/data/tactics.ron")
            .map(|e| e.data)
            .unwrap_or_default();
            builder.insert_resource(tactics);
        }

        {
            let hm_ref = heightmap.clone();
            let cover_map = CoverMap::precompute(sandbox_size, 5.0, &|x, z| hm_ref.sample(x, z));
            builder.insert_resource(cover_map);
        }

        builder.insert_resource(heightmap.clone());
        builder.insert_resource(WorldStreamer::new(50.0, 80.0));
        builder.insert_resource(std::sync::Mutex::new(AssetManager::new()));
        builder.insert_resource(AudioEngine::new());
        builder.insert_resource(HierarchicalSpatialIndex::new());
        builder.insert_resource(HpaGraph::build());
        builder.insert_resource(OriginShift::new());
        builder.insert_resource(NavDirtyTracker::new());
        builder.insert_resource(crate::graphics::destruction_occlusion::DestructionOcclusionSystem::new());
        builder.insert_resource(crate::graphics::gore_mesh::GoreMeshSystem::new(256));
        builder.insert_resource(ComponentRegistry::default_registry());
        builder.insert_resource(MaterialTruthService::empty());
        builder.insert_resource(crate::graphics::surface_state_render::SurfaceStateRenderSystem::new());

        // Reverb zones: basement (Zone C) gets interior reverb
        let mut audio_occlusion = crate::audio::occlusion::OcclusionSystem::new();
        audio_occlusion.zones.push(crate::audio::occlusion::ReverbZone {
            center: glam::Vec3::new(40.0, 1.5, 12.5),
            radius: 15.0,
            reverb_time: 2.0,
            early_reflections: 0.7,
            density: 0.8,
        });
        builder.insert_resource(audio_occlusion);

        builder.insert_resource(authority_entries);
        builder.insert_resource(OwnershipMap::new());
        builder.insert_resource(SimBus::new());
        builder.insert_resource(RenderBus::new());
        builder.insert_resource(DebugBus::new());
        builder.insert_resource(StickyEvents::new());
        let mut tracer = EventTracer::new();
        tracer.enable();
        builder.insert_resource(tracer);
        builder.insert_resource(EventAggregator::new(10.0));
        builder.insert_resource(RuntimeConfig::default());
        builder.insert_resource(QualityGovernor::new(60));
        builder.insert_resource(create_default_registry());
        builder.insert_resource(TerrainTruth::grassland());
        builder.insert_resource(SurfaceStateStore::new());
        builder.insert_resource(DamageOrchestrator::new());
        builder.insert_resource(PrefabRegistry::new());
        builder.insert_resource(AssetBudget::default());
        builder.insert_resource(EngineSDK::new());

        let center = sandbox_size * 0.5;
        builder.add_system_default(Box::new(SimulationSystem::new(center, center)));
        builder.add_system_default(Box::new(PhysicsSystem::new(heightmap)));
        builder.add_system_default(Box::new(crate::core::integration_systems::BallisticsTickSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::DamageDispatchSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::DestructionTickSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::TerrainDeformationTickSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::NavDirtyTickSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::OcclusionWireSystem));
        builder.add_system_default(Box::new(crate::core::integration_systems::GoreWireSystem));
        builder.add_system_default(Box::new(
            crate::animation::animation_integration::AnimationIntegrationSystem::new(),
        ));
        builder.add_system_default(Box::new(
            crate::core::integration_systems::AnimationWireSystem::new(),
        ));
        builder.add_system_default(Box::new(
            crate::audio::audio_integration::AudioIntegrationSystem::new(),
        ));
        builder.add_system_default(Box::new(crate::body::body_system::BodySystem));
        builder.add_system_default(Box::new(crate::graphics::render_system::RenderSystem));
        builder.add_system_default(Box::new(
            crate::audio::playback::AudioPlaybackBridge::new(),
        ));
        builder.add_system_default(Box::new(
            crate::input::input_system::InputActionSystem::new(),
        ));

        builder.add_plugin(CombatPlugin);

        builder.insert_resource(crate::economy::item_registry::ItemRegistry::new());
        builder.insert_resource(crate::audio::sound_bank::SoundBank::new());
        builder.insert_resource(crate::core::perf::perf_budget::PerfBudgetManager::new(60));
        builder.insert_resource(crate::body::death_pipeline::CorpseManager::new());
        builder.insert_resource(crate::animation::clip_map::ClipMap::new());
        builder.insert_resource(crate::animation::animation_ladder::AnimationLadder::default());
        builder.insert_resource(crate::gameplay::factions::FactionRelations::new());
        builder.insert_resource(crate::core::determinism_policy::DeterminismPolicyMatrix::build_default());
        builder.insert_resource(crate::core::build_manifest::SchemaMigrationRegistry::new());
        builder.insert_resource(crate::tools::sim_metrics_dashboard::SimMetricsDashboard::default());
        builder.insert_resource(crate::core::perf::sim_telemetry::SimTelemetry::new());
        builder.insert_resource(crate::core::dirty_set::ChunkDirtySet::default());
        builder.insert_resource(crate::core::dirty_set::NavDirtySet::default());
        builder.insert_resource(crate::core::dirty_set::EntityDirtySet::default());
        builder.insert_resource(crate::core::perf::low_spec_cert::LowSpecCertifier::default());
        builder.insert_resource(ChunkPersistenceService::new("game/saves/chunks"));

        builder.add_init_fn(|ecs: &mut Ecs, _res: &mut crate::core::registry::Resources| {
            let mut authoring = WorldAuthoringDatabase::new();
            let chunk_path = std::path::Path::new("game/world/chunks/chunk_sandbox_main.ron");
            if let Ok(contents) = std::fs::read_to_string(chunk_path) {
                match ron::from_str::<crate::world::authoring::ChunkAuthoring>(&contents) {
                    Ok(chunk) => { authoring.set_chunk(chunk); }
                    Err(e) => { println!("[sandbox] ERROR parsing chunk RON: {}", e); }
                }
            } else {
                println!("[sandbox] WARNING: chunk file not found: {:?}", chunk_path);
            }
            let chunk_count = authoring.chunk_count();
            println!("[sandbox] loaded {} authored chunk(s)", chunk_count);

            let mut spawned = 0u32;
            let coords: Vec<_> = authoring.coords().cloned().collect();
            for coord in coords {
                if let Some(chunk) = authoring.get_chunk(&coord) {
                    for spawn in &chunk.spawns {
                        let (entity, _pid) = ecs.spawn_new();
                        let world_x = spawn.position[0];
                        let world_z = spawn.position[2];

                        ecs.transforms.insert(
                            entity,
                            crate::world::components::Transform {
                                x: world_x,
                                y: world_z,
                                cell_x: (world_x / 50.0) as u32,
                                cell_y: (world_z / 50.0) as u32,
                            },
                        );

                        let label = spawn
                            .overrides
                            .get("label")
                            .cloned()
                            .unwrap_or_else(|| spawn.prefab_name.clone());
                        ecs.names.insert(entity, crate::world::components::Name(label));

                        spawned += 1;
                    }
                }
            }
            println!("[sandbox] spawned {} entities from authored content", spawned);
        });

        let mut ecs = Ecs::new();
        let (systems, resources) = builder.build(&mut ecs);
        let mut engine = Engine::from_builder(systems, resources, ecs);
        engine.init();
        engine
    }

    /// Minimal sandbox for testing individual systems.
    /// Only Ecs and SimulationSystem.
    pub fn sandbox() -> Engine {
        let grid = WorldGrid::generate();
        let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
        let resource_grid = ResourceGrid::new(&biomes);

        let mut builder = EngineBuilder::new();
        builder.insert_resource(resource_grid);

        let center = crate::world::cell::WORLD_SIZE * 0.5;
        builder.add_system_default(Box::new(SimulationSystem::new(center, center)));

        let mut ecs = Ecs::new();

        let (systems, resources) = builder.build(&mut ecs);
        let mut engine = Engine::from_builder(systems, resources, ecs);
        engine.init();
        engine
    }
}

impl Default for RuntimeAssembly {
    fn default() -> Self {
        Self {
            config: RuntimeConfig {
                profile: RuntimeProfile::VerticalSlice,
                ..Default::default()
            },
        }
    }
}
