use std::sync::Arc;

use engine_core::component_registry::ComponentRegistry;
use engine_core::material_truth::MaterialTruthService;
use engine_core::plugin::EngineBuilder;
use engine_navigation::dynamic_nav_update::NavDirtyTracker;
use engine_navigation::hpa_star::HpaGraph;
use crate::runtime::bootstrap::common::insert_runtime_core;
use engine_world::fields::{AnomalyForceType, AnomalyZone, WorldFields};
use engine_world::heightmap::Heightmap;
use engine_world::hierarchical_spatial::HierarchicalSpatialIndex;
use engine_world::origin_shift::OriginShift;
use engine_world::streaming::WorldStreamer;
use engine_world::surface_state::SurfaceStateStore;
use engine_world::terrain_deformation::TerrainDeformationSystem;
use engine_world::terrain_truth::TerrainTruth;
use engine_audio::audio::AudioEngine;
use engine_content::asset_budget::AssetBudget;
use engine_content::prefabs::prefab_registry::PrefabRegistry;
use engine_physics::damage_pipeline::DamageOrchestrator;
use engine_physics::destruction::DestructionSystem;

pub(super) fn insert_world_and_damage_resources(builder: &mut EngineBuilder) {
    let mut world_fields = WorldFields::new();
    world_fields.anomaly.zones.push(AnomalyZone {
        center: glam::Vec3::new(
            engine_world::cell::WORLD_SIZE * 0.7,
            0.0,
            engine_world::cell::WORLD_SIZE * 0.3,
        ),
        radius: 80.0,
        force_strength: 15.0,
        force_type: AnomalyForceType::Vortex,
    });
    builder.insert_resource(world_fields);
    builder.insert_resource(DestructionSystem::new());
    builder.insert_resource(TerrainDeformationSystem::new());
}

fn canonical_material_truth_service(builder: &EngineBuilder) -> MaterialTruthService {
    let cfg = builder
        .resources
        .get::<engine_core::game_config::GameConfig>()
        .expect("MaterialTruthService requires explicitly inserted GameConfig");
    MaterialTruthService::from_game_config(cfg)
}

pub(super) fn insert_vertical_runtime_resources(
    builder: &mut EngineBuilder,
    heightmap: &Arc<Heightmap>,
    authority_entries: Vec<engine_core::world_state_authority::StateAuthorityEntry>,
) {
    builder.insert_resource(heightmap.clone());
    builder.insert_resource(WorldStreamer::new(3000.0, 4000.0));
    builder.insert_resource(std::sync::Mutex::new(
        engine_memory::asset_manager::AssetManager::new(),
    ));
    builder.insert_resource(AudioEngine::new());
    builder.insert_resource(HierarchicalSpatialIndex::new());
    builder.insert_resource(HpaGraph::build());
    builder.insert_resource(OriginShift::new());
    builder.insert_resource(NavDirtyTracker::new());
    builder
        .insert_resource(engine_render::destruction_occlusion::DestructionOcclusionSystem::new());
    builder.insert_resource(engine_render::gore_mesh::GoreMeshSystem::new(256));
    builder.insert_resource(ComponentRegistry::default_registry());
    builder.insert_resource(canonical_material_truth_service(builder));
    builder.insert_resource(engine_render::surface_state_render::SurfaceStateRenderSystem::new());

    builder.insert_resource(authority_entries);
    builder.insert_resource(OwnershipMap::new());
    insert_runtime_core(builder, engine_core::runtime_config::RuntimeConfig::game());
    builder.insert_resource(TerrainTruth::grassland());
    builder.insert_resource(SurfaceStateStore::new());
    builder.insert_resource(DamageOrchestrator::new());
    builder.insert_resource(PrefabRegistry::new());
    builder.insert_resource(AssetBudget::default());
}

pub(super) fn insert_headless_runtime_resources(
    builder: &mut EngineBuilder,
    heightmap: &Arc<Heightmap>,
    authority_entries: Vec<engine_core::world_state_authority::StateAuthorityEntry>,
) {
    builder.insert_resource(heightmap.clone());
    builder.insert_resource(WorldStreamer::new(3000.0, 4000.0));
    builder.insert_resource(std::sync::Mutex::new(
        engine_memory::asset_manager::AssetManager::new(),
    ));
    builder.insert_resource(HierarchicalSpatialIndex::new());
    builder.insert_resource(HpaGraph::build());
    builder.insert_resource(OriginShift::new());
    builder.insert_resource(NavDirtyTracker::new());
    builder.insert_resource(ComponentRegistry::default_registry());
    builder.insert_resource(canonical_material_truth_service(builder));

    builder.insert_resource(authority_entries);
    builder.insert_resource(OwnershipMap::new());
    insert_runtime_core(builder, engine_core::runtime_config::RuntimeConfig::game());
    builder.insert_resource(TerrainTruth::grassland());
    builder.insert_resource(SurfaceStateStore::new());
    builder.insert_resource(DamageOrchestrator::new());
    builder.insert_resource(PrefabRegistry::new());
    builder.insert_resource(AssetBudget::default());
}

pub(super) fn insert_game_tail_resources(builder: &mut EngineBuilder, with_tools_metrics: bool) {
    builder.insert_resource(engine_game::economy::item_registry::ItemRegistry::new());
    builder.insert_resource(Vec::<engine_simulation::camp_simulation::CampState>::new());
    builder.insert_resource(engine_audio::sound_bank::SoundBank::new());
    builder.insert_resource(engine_core::perf::perf_budget::PerfBudgetManager::new(60));
    builder.insert_resource(engine_simulation::world_milestones::WorldMilestoneTracker::new());
    builder.insert_resource(engine_body::death_pipeline::CorpseManager::new());
    builder.insert_resource(engine_animation::clip_map::ClipMap::new());
    builder.insert_resource(engine_animation::animation_ladder::AnimationLadder::default());
    builder.insert_resource(engine_game::gameplay::factions::FactionRelations::new());
    builder
        .insert_resource(engine_core::determinism_policy::DeterminismPolicyMatrix::build_default());
    builder.insert_resource(engine_core::build_manifest::SchemaMigrationRegistry::new());
    if with_tools_metrics {
        builder
            .insert_resource(engine_tools::sim_metrics_dashboard::SimMetricsDashboard::default());
    }
    builder.insert_resource(engine_core::perf::sim_telemetry::SimTelemetry::new());
    builder.insert_resource(engine_core::dirty_set::ChunkDirtySet::default());
    builder.insert_resource(engine_core::dirty_set::NavDirtySet::default());
    builder.insert_resource(engine_core::dirty_set::EntityDirtySet::default());
    builder.insert_resource(engine_core::perf::low_spec_cert::LowSpecCertifier::default());
}
