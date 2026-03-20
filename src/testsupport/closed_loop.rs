//! Phase 10: Closed-Loop Simulation Validation
//! Runs headless simulation for N ticks and validates invariants.

use std::collections::HashMap;

use engine_ecs::Ecs;
use engine_runtime::engine::Engine;
// LEGACY IMPORTS - Use canonical crates instead
use engine_runtime::simulation_core::systems::engine_system::{EngineSystem, FixedTickContext};
use engine_ecs::system_descriptor::SystemDescriptor;
use engine_core::events::canonical::{EntityDied, ImpactEvent, NavUpdated, WorldTopologyChanged};
use crate::core::system::EngineSystem as LegacyEngineSystem;
use crate::game::economy::resource_flow;

const SIM_DT: f32 = 1.0 / 20.0;

/// Accumulated counts during closed-loop run.
#[derive(Default, Clone)]
pub struct ClosedLoopRecorder {
    pub deaths: u32,
    pub births: u32,
    pub topology_changes: u32,
    pub nav_updates: u32,
    pub impacts: u32,
    pub events_processed: HashMap<String, u64>,
}

/// System that records canonical events each tick (runs in fixed_tick, before events are cleared).
struct ClosedLoopRecorderSystem;

impl LegacyEngineSystem for ClosedLoopRecorderSystem {
    fn name(&self) -> &str {
        "ClosedLoopRecorder"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("ClosedLoopRecorder")
            .reads_event::<EntityDied>()
            .reads_event::<ImpactEvent>()
            .reads_event::<NavUpdated>()
            .reads_event::<WorldTopologyChanged>()
            .after("NavDirtyTick")
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let Some(recorder) = ctx.resources.get_mut::<ClosedLoopRecorder>() else {
            return;
        };

        let deaths = ctx.events.read::<EntityDied>();
        recorder.deaths += deaths.len() as u32;
        let e = recorder
            .events_processed
            .entry("EntityDied".into())
            .or_insert(0);
        *e = e.saturating_add(deaths.len() as u64);

        let impacts = ctx.events.read::<ImpactEvent>();
        recorder.impacts += impacts.len() as u32;
        let e = recorder
            .events_processed
            .entry("ImpactEvent".into())
            .or_insert(0);
        *e = e.saturating_add(impacts.len() as u64);

        let nav = ctx.events.read::<NavUpdated>();
        recorder.nav_updates += nav.len() as u32;
        let e = recorder
            .events_processed
            .entry("NavUpdated".into())
            .or_insert(0);
        *e = e.saturating_add(nav.len() as u64);

        let topo = ctx.events.read::<WorldTopologyChanged>();
        recorder.topology_changes += topo.len() as u32;
        let e = recorder
            .events_processed
            .entry("WorldTopologyChanged".into())
            .or_insert(0);
        *e = e.saturating_add(topo.len() as u64);
    }
}

/// Result of running closed-loop validation.
#[derive(Debug, Clone)]
pub struct ClosedLoopReport {
    pub ticks: u64,
    pub npc_count_start: usize,
    pub npc_count_end: usize,
    pub monster_count_start: usize,
    pub monster_count_end: usize,
    pub total_money_start: f64,
    pub total_money_end: f64,
    pub deaths: u32,
    pub births: u32,
    pub topology_changes: u32,
    pub nav_updates: u32,
    pub impacts: u32,
    pub events_processed: HashMap<String, u64>,
    pub degraded_but_truthful: bool,
}

/// Build headless engine with ClosedLoopRecorder for validation.
fn build_headless_with_recorder(biomes: &[crate::world::biome::Biome]) -> Engine {
    let grid = crate::world::world::WorldGrid::generate();
    let resource_grid = crate::world::resources::ResourceGrid::new(biomes);
    let heightmap = std::sync::Arc::new(crate::world::heightmap::Heightmap::generate(biomes));

    let mut builder = EngineBuilder::new();
    builder.insert_resource(resource_grid);
    builder.insert_resource(ClosedLoopRecorder::default());

    let mut world_fields = crate::world::fields::WorldFields::new();
    world_fields
        .anomaly
        .zones
        .push(crate::world::fields::AnomalyZone {
            center: glam::Vec3::new(
                crate::world::cell::WORLD_SIZE * 0.7,
                0.0,
                crate::world::cell::WORLD_SIZE * 0.3,
            ),
            radius: 80.0,
            force_strength: 15.0,
            force_type: crate::world::fields::AnomalyForceType::Vortex,
        });
    builder.insert_resource(world_fields);
    builder.insert_resource(crate::physics::destruction::DestructionSystem::new());
    builder.insert_resource(crate::world::terrain_deformation::TerrainDeformationSystem::new());

    builder.add_plugin(crate::game::stalker_plugin::StalkerPlugin::from_config(
        "game/data",
    ));
    builder.add_plugin(crate::game::weapons_plugin::WeaponsPlugin::new("game/data"));

    {
        use crate::core::config::{load_config, ConfigEnvelope};
        use crate::game::ai::combat_tactics::tactics::TacticProfile;
        use std::collections::HashMap;
        let tactics: HashMap<String, TacticProfile> =
            load_config::<ConfigEnvelope<HashMap<String, TacticProfile>>>("game/data/tactics.ron")
                .map(|e| e.data)
                .unwrap_or_default();
        builder.insert_resource(tactics);
    }

    {
        let hm_ref = heightmap.clone();
        let cover_map = crate::navigation::cover_map::CoverMap::precompute(
            crate::world::cell::WORLD_SIZE,
            20.0,
            &|x, z| hm_ref.sample(x, z),
        );
        builder.insert_resource(cover_map);
    }

    builder.insert_resource(crate::world::streaming::WorldStreamer::new(3000.0, 4000.0));
    builder.insert_resource(std::sync::Mutex::new(
        crate::memory::asset_manager::AssetManager::new(),
    ));
    builder.insert_resource(crate::audio::audio::AudioEngine::new());
    builder.insert_resource(crate::world::hierarchical_spatial::HierarchicalSpatialIndex::new());
    builder.insert_resource(crate::navigation::hpa_star::HpaGraph::build());
    builder.insert_resource(crate::world::origin_shift::OriginShift::new());
    builder.insert_resource(crate::navigation::dynamic_nav_update::NavDirtyTracker::new());
    builder
        .insert_resource(crate::graphics::destruction_occlusion::DestructionOcclusionSystem::new());
    builder.insert_resource(crate::graphics::gore_mesh::GoreMeshSystem::new(256));
    builder.insert_resource(crate::core::component_registry::ComponentRegistry::default_registry());
    builder.insert_resource(crate::core::material_truth::MaterialTruthService::empty());
    builder.insert_resource(crate::graphics::surface_state_render::SurfaceStateRenderSystem::new());

    let center = crate::world::cell::WORLD_SIZE * 0.5;
    builder.add_system_default(Box::new(
        crate::simulation::simulation::SimulationSystem::new(center, center),
    ));
    builder.add_system_default(Box::new(
        crate::runtime::wiring::world_tick::WorldTickSystem::new(grid),
    ));
    builder.add_system_default(Box::new(crate::game::ai::ai::AiSystem::new()));
    builder.add_system_default(Box::new(crate::physics::physics::PhysicsSystem::new(
        heightmap,
    )));
    builder.add_system_default(Box::new(crate::game::economy::economy::EconomySystem));
    builder.add_system_default(Box::new(
        crate::runtime::wiring::integration::BallisticsTickSystem,
    ));
    builder.add_system_default(Box::new(
        crate::runtime::wiring::integration::DamageDispatchSystem,
    ));
    builder.add_system_default(Box::new(
        crate::runtime::wiring::integration::DestructionTickSystem,
    ));
    builder.add_system_default(Box::new(
        crate::runtime::wiring::integration::TerrainDeformationTickSystem,
    ));
    builder.add_system_default(Box::new(
        crate::runtime::wiring::integration::NavDirtyTickSystem,
    ));
    builder.add_system_default(Box::new(
        crate::runtime::wiring::integration::OcclusionWireSystem,
    ));
    builder.add_system_default(Box::new(
        crate::runtime::wiring::integration::GoreWireSystem,
    ));
    builder.add_system_default(Box::new(
        crate::runtime::wiring::animation::AnimationIntegrationSystem::new(),
    ));
    builder.add_system_default(Box::new(
        crate::audio::audio_integration::AudioIntegrationSystem::new(),
    ));
    builder.add_system_default(Box::new(ClosedLoopRecorderSystem));

    let mut ecs = Ecs::new();
    crate::world::population::spawn_npcs(&mut ecs);
    crate::world::population::spawn_monsters(&mut ecs);

    let (systems, resources) = builder.build(&mut ecs);
    let mut engine = Engine::from_builder(systems, resources, ecs);
    engine.init();
    engine
}

/// Run headless simulation for N ticks and produce a ClosedLoopReport.
pub fn run_closed_loop(ticks: u64) -> ClosedLoopReport {
    let grid = crate::world::world::WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();

    let mut engine = build_headless_with_recorder(&biomes);

    let npc_count_start = engine.ecs.count_npcs();
    let monster_count_start = engine.ecs.monsters().len();
    let snap_start = resource_flow::snapshot(&engine.ecs);
    let total_money_start = snap_start.total_npc_money as f64;

    for _ in 0..ticks {
        engine.tick(SIM_DT);
    }

    let npc_count_end = engine.ecs.count_npcs();
    let monster_count_end = engine.ecs.monsters().len();
    let snap_end = resource_flow::snapshot(&engine.ecs);
    let total_money_end = snap_end.total_npc_money as f64;

    let recorder = engine
        .resources
        .get::<ClosedLoopRecorder>()
        .cloned()
        .unwrap_or_default();

    let births = (npc_count_end + monster_count_end)
        .saturating_sub(npc_count_start + monster_count_start)
        .saturating_add(recorder.deaths as usize) as u32;

    let event_drops = engine.events.total_dropped();
    let degraded_but_truthful = event_drops > 0;

    ClosedLoopReport {
        ticks,
        npc_count_start,
        npc_count_end,
        monster_count_start,
        monster_count_end,
        total_money_start,
        total_money_end,
        deaths: recorder.deaths,
        births,
        topology_changes: recorder.topology_changes,
        nav_updates: recorder.nav_updates,
        impacts: recorder.impacts,
        events_processed: recorder.events_processed,
        degraded_but_truthful,
    }
}

/// Validation result for closed-loop run.
#[derive(Debug, Clone)]
pub struct ClosedLoopValidation {
    pub passed: bool,
    pub messages: Vec<String>,
}

/// Validate report: money circulated, at least one death in long sim, destruction occurred,
/// nav updates occurred, no entity count explosion.
pub fn validate_closed_loop(
    report: &ClosedLoopReport,
    min_ticks_for_death: u64,
) -> ClosedLoopValidation {
    let mut messages = Vec::new();
    let mut passed = true;

    let money_changed = (report.total_money_end - report.total_money_start).abs() > 0.01;
    if !money_changed && report.ticks > 10 {
        messages.push("Money did not circulate (total unchanged)".to_string());
        passed = false;
    } else if money_changed {
        messages.push(format!(
            "Money circulated: ${:.1} -> ${:.1}",
            report.total_money_start, report.total_money_end
        ));
    }

    if report.ticks >= min_ticks_for_death && report.deaths == 0 {
        messages.push(format!(
            "Expected at least one death in {} ticks but got 0",
            report.ticks
        ));
        passed = false;
    } else if report.deaths > 0 {
        messages.push(format!("Deaths occurred: {}", report.deaths));
    }

    if report.topology_changes == 0 && report.ticks > 100 {
        messages.push("No destruction/topology events in long simulation".to_string());
        passed = false;
    } else if report.topology_changes > 0 {
        messages.push(format!("Destruction events: {}", report.topology_changes));
    }

    if report.nav_updates == 0 && report.ticks > 50 {
        messages.push("No nav updates in simulation".to_string());
        passed = false;
    } else if report.nav_updates > 0 {
        messages.push(format!("Nav updates: {}", report.nav_updates));
    }

    let total_start = report.npc_count_start + report.monster_count_start;
    let total_end = report.npc_count_end + report.monster_count_end;
    if total_end > total_start * 10 && report.ticks > 100 {
        messages.push(format!(
            "Entity count explosion: {} -> {}",
            total_start, total_end
        ));
        passed = false;
    }

    ClosedLoopValidation { passed, messages }
}
