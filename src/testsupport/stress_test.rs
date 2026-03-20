//! Phase 11: Vertical Slice + Stress Test
//! Spawns extra entities and fires many ballistic events to stress the simulation.

use std::time::Instant;

use glam::Vec3;
use rand::Rng;

use crate::runtime::bootstrap::GameRuntimeAssembly;
use crate::world::cell::{CELL_SIZE, GRID_SIZE, WORLD_SIZE};
use crate::world::components::MonsterSpecies;
use crate::world::population;
use engine_physics::ballistics::BallisticsSystem;
use engine_runtime::engine::Engine;

const SIM_DT: f32 = 1.0 / 20.0;

/// Result of stress test run.
#[derive(Debug, Clone)]
pub struct StressReport {
    pub ticks: u64,
    pub avg_tick_ms: f64,
    pub worst_tick_ms: f64,
    pub p99_tick_ms: f64,
    pub peak_entity_count: usize,
    pub peak_event_count: usize,
    pub peak_destruction_objects: usize,
    pub governor_activations: u32,
    pub event_drops: u64,
    pub memory_peak_mb: f64,
}

/// Build headless and spawn extra entities for stress.
fn build_stress_world(extra_npcs: usize, extra_monsters_per_species: usize) -> Engine {
    let grid = crate::world::world::WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let mut engine = GameRuntimeAssembly::headless(&biomes);

    let mut rng = rand::thread_rng();
    for _ in 0..extra_npcs {
        let (e, _pid) = engine.ecs.spawn_new();
        let cx = rng.gen_range(0..GRID_SIZE);
        let cy = rng.gen_range(0..GRID_SIZE);
        engine.ecs.set_transform(
            e,
            crate::world::components::Transform {
                x: cx as f32 * CELL_SIZE + rng.gen_range(0.0..CELL_SIZE),
                y: cy as f32 * CELL_SIZE + rng.gen_range(0.0..CELL_SIZE),
                cell_x: cx,
                cell_y: cy,
            },
        );
        engine
            .ecs
            .set_kind(e, crate::world::components::EntityKind::Npc);
        engine
            .ecs
            .set_name(e, crate::world::components::Name("StressNPC".into()));
        engine
            .ecs
            .set_personal_needs(e, crate::world::components::PersonalNeeds::default_npc());
        engine.ecs.set_npc_economy(
            e,
            crate::world::components::NpcEconomy {
                money: 50.0,
                monthly_required: 50.0,
                job: crate::world::components::Job::Guard,
                desperation: 0.0,
            },
        );
        engine.ecs.set_sim_level(
            e,
            crate::world::components::SimLevel {
                level: crate::world::components::SimulationLevel::L1,
            },
        );
        engine
            .ecs
            .set_ai_state(e, crate::world::components::AiState::Idle);
        engine
            .ecs
            .set_inventory(e, crate::world::components::Inventory { items: Vec::new() });
        engine
            .ecs
            .set_memory(e, crate::core::ai_memory::Memory::new());
        engine
            .ecs
            .set_emotions(e, crate::core::ai_emotions::Emotions::new());
        engine.ecs.set_life_info(
            e,
            crate::world::components::LifeInfo {
                age: 100.0,
                max_age: 400.0,
                last_mate_day: 0,
                mate_cooldown_days: 60,
            },
        );
    }

    for species in [
        MonsterSpecies::Wolf,
        MonsterSpecies::Boar,
        MonsterSpecies::Bloodsucker,
    ] {
        for _ in 0..extra_monsters_per_species {
            population::spawn_single_monster(&mut engine.ecs, species, &mut rng);
        }
    }

    engine
}

/// Run stress test: headless with extra entities, fire many ballistics per tick.
pub fn run_meat_grinder(ticks: u64) -> StressReport {
    let grid = crate::world::world::WorldGrid::generate();
    let _biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let mut engine = build_stress_world(50, 20);

    let mut tick_times: Vec<f64> = Vec::with_capacity(ticks as usize);
    let mut peak_entity_count = engine.ecs.alive.len();
    let mut peak_destruction_objects = 0usize;

    let weapons = engine
        .resources
        .get::<crate::game::weapons_plugin::WeaponRegistry>()
        .cloned();
    let weapon_id = weapons
        .as_ref()
        .and_then(|w| w.get_id("ak47"))
        .or_else(|| {
            weapons
                .as_ref()
                .and_then(|w| w.name_to_id.values().next().copied())
        })
        .unwrap_or(0);

    for i in 0..ticks {
        let t0 = Instant::now();

        if let Some(ballistics) = engine.resources.get_mut::<BallisticsSystem>() {
            let center = WORLD_SIZE * 0.5;
            for j in 0..20 {
                let angle = (i as f32 * 0.5 + j as f32 * 0.3) % (2.0 * std::f32::consts::PI);
                let origin = Vec3::new(center, 10.0, center);
                let dir = Vec3::new(angle.cos(), 0.0, angle.sin());
                ballistics.fire(
                    origin,
                    dir,
                    400.0,
                    0.01,
                    0.001,
                    0,
                    weapon_id,
                    (i * 20 + j as u64) as u32,
                );
            }
        }

        engine.tick(SIM_DT);

        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
        tick_times.push(elapsed_ms);

        peak_entity_count = peak_entity_count.max(engine.ecs.alive.len());

        if let Some(dest) = engine
            .resources
            .get::<engine_physics::destruction::DestructionSystem>()
        {
            let obj_count = dest.objects.len();
            peak_destruction_objects = peak_destruction_objects.max(obj_count);
        }
    }

    let event_drops = engine.events.total_dropped();

    tick_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = tick_times.len();
    let avg_tick_ms = if n > 0 {
        tick_times.iter().sum::<f64>() / n as f64
    } else {
        0.0
    };
    let worst_tick_ms = tick_times.last().copied().unwrap_or(0.0);
    let p99_idx = (n as f64 * 0.99) as usize;
    let p99_tick_ms = tick_times
        .get(p99_idx.min(n.saturating_sub(1)))
        .copied()
        .unwrap_or(0.0);

    StressReport {
        ticks,
        avg_tick_ms,
        worst_tick_ms,
        p99_tick_ms,
        peak_entity_count,
        peak_event_count: event_drops as usize,
        peak_destruction_objects,
        governor_activations: 0,
        event_drops,
        memory_peak_mb: 0.0,
    }
}
