//! Debug output utilities for main.rs.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: none (output-only)

use crate::core::ecs::Ecs;
use crate::core::engine::Engine;
use crate::game::ai::body as ai_body;
use crate::graphics::lod::{LodConfig, LodLevel};
use crate::graphics::mesh::EntityInstance;
use crate::graphics::visibility::Frustum;
use crate::world::components::*;
use crate::world::heightmap::Heightmap;

/// Print brief status line every N frames.
pub fn print_brief(engine: &Engine, frame: u64) {
    let npcs = engine.ecs.npcs();
    let monsters = engine.ecs.monsters();
    let snap = crate::game::economy::resource_flow::snapshot(&engine.ecs);

    let mut wolves = 0u32;
    let mut boars = 0u32;
    let mut bloods = 0u32;
    for &m in &monsters {
        match engine.ecs.get_kind(m) {
            Some(EntityKind::Monster(MonsterSpecies::Wolf)) => wolves += 1,
            Some(EntityKind::Monster(MonsterSpecies::Boar)) => boars += 1,
            Some(EntityKind::Monster(MonsterSpecies::Bloodsucker)) => bloods += 1,
            _ => {}
        }
    }

    let carcasses = engine.resource_grid().carcasses.len();
    let night = if ai_body::is_night(engine.time.day_progress()) {
        "NIGHT"
    } else {
        "day"
    };
    let season = engine.time.season();

    println!(
        "[tick {}] day {} mo {} {} ({}) | NPC:{} W:{} B:{} BS:{} tot:{} carcass:{} | ${:.0} desp:{:.2}",
        frame, engine.time.day, engine.time.month, season, night,
        npcs.len(), wolves, boars, bloods, engine.ecs.alive.len(), carcasses,
        snap.total_npc_money, snap.average_desperation,
    );
}

/// Print detailed economy report on month change.
pub fn print_economy(engine: &Engine) {
    println!("--- NPCs ({}) ---", engine.ecs.count_npcs());
    for &e in &engine.ecs.alive {
        if !matches!(engine.ecs.get_kind(e), Some(EntityKind::Npc)) {
            continue;
        }
        print_entity_summary(&engine.ecs, e);
    }

    println!("--- Monsters ({}) ---", engine.ecs.monsters().len());
    for &e in &engine.ecs.alive {
        if !matches!(engine.ecs.get_kind(e), Some(EntityKind::Monster(_))) {
            continue;
        }
        print_entity_summary(&engine.ecs, e);
    }

    println!(
        "  carcasses on map: {}",
        engine.resource_grid().carcasses.len()
    );
}

fn print_entity_summary(ecs: &Ecs, e: u64) {
    let name = ecs.get_name(e).map(|n| n.0.as_str()).unwrap_or("?");
    let species = match ecs.get_kind(e) {
        Some(EntityKind::Monster(s)) => format!("({:?})", s),
        _ => String::new(),
    };

    let goal = match ecs.get_ai_state(e) {
        Some(AiState::Executing(g)) => format!("{}", g),
        _ => "Idle".into(),
    };

    let hp = ecs.get_needs(e).map_or(1.0, |p| p.health);
    let hunger = ecs.get_needs(e).map_or(0.0, |p| p.hunger);
    let emo_str = ecs
        .get_emotions(e)
        .map(|em| format!("{}", em.dominant()))
        .unwrap_or_else(|| "?".into());

    let mem_count = ecs.get_memory(e).map_or(0, |m| m.events.len());
    let relations = ecs.get_memory(e).map_or(0, |m| m.entities.len());
    let has_plan = ecs.plans.contains_key(&e);

    let money = ecs.get_npc_economy(e).map_or(0.0, |e| e.money);
    let age_str = ecs
        .get_life_info(e)
        .map(|li| format!("age:{:.0}/{:.0} {:?}", li.age, li.max_age, li.life_stage()))
        .unwrap_or_default();

    println!(
        "  {} {}| {} [{}] | hp:{:.0}% hunger:{:.0}% ${:.0} | {} | mem:{} rel:{} plan:{}",
        name,
        species,
        goal,
        emo_str,
        hp * 100.0,
        hunger * 100.0,
        money,
        age_str,
        mem_count,
        relations,
        has_plan,
    );
}

/// Collect entity instances for rendering.
pub fn collect_entity_instances(
    ecs: &Ecs,
    heightmap: &Heightmap,
    camera_pos: [f32; 3],
    frustum: &Frustum,
) -> Vec<EntityInstance> {
    let lod_config = LodConfig::default();
    let cam = glam::Vec3::from(camera_pos);
    let mut out = Vec::with_capacity(ecs.alive.len());

    for &e in &ecs.alive {
        let t = match ecs.get_transform(e) {
            Some(t) => t,
            None => continue,
        };

        let y = heightmap.sample(t.x, t.y) + 1.0;
        let pos = glam::Vec3::new(t.x, y, t.y);
        let dist = (pos - cam).length();

        if lod_config.compute_lod(dist) == LodLevel::Culled {
            continue;
        }
        if !frustum.test_sphere(pos, 2.0) {
            continue;
        }

        let color = match ecs.get_kind(e) {
            Some(EntityKind::Npc) => [0.16, 0.47, 1.0],
            Some(EntityKind::Monster(MonsterSpecies::Wolf)) => [0.9, 0.9, 0.9],
            Some(EntityKind::Monster(MonsterSpecies::Boar)) => [0.55, 0.43, 0.39],
            Some(EntityKind::Monster(MonsterSpecies::Bloodsucker)) => [0.83, 0.0, 0.0],
            None => [0.5, 0.5, 0.5],
        };

        out.push(EntityInstance {
            position: [t.x, y, t.y],
            color,
        });
    }

    out
}
