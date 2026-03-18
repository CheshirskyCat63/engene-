use crate::core::engine::Engine;
use crate::core::query::WithMonster;
use crate::game::ai::body as ai_body;
use crate::game::economy::resource_flow;
use crate::graphics::lod::{LodConfig, LodLevel};
use crate::graphics::mesh::EntityInstance;
use crate::graphics::visibility::Frustum;
use crate::world::components::{AiState, EntityKind, MonsterSpecies};
use crate::world::heightmap::Heightmap;

pub(super) fn print_brief(engine: &Engine, frame: u64) {
    let npcs: Vec<_> = engine.ecs.iter_npcs().collect();
    let monsters: Vec<_> = engine.ecs.query_filter(WithMonster).collect();
    let snap = resource_flow::snapshot(&engine.ecs);

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
        frame,
        engine.time.day,
        engine.time.month,
        season,
        night,
        npcs.len(),
        wolves,
        boars,
        bloods,
        engine.ecs.alive.len(),
        carcasses,
        snap.total_npc_money,
        snap.average_desperation,
    );
}

pub(super) fn collect_entity_instances(
    ecs: &crate::core::ecs::Ecs,
    heightmap: &Heightmap,
    camera_pos: [f32; 3],
    frustum: &Frustum,
) -> Vec<EntityInstance> {
    let lod_config = LodConfig::default();
    let cam = glam::Vec3::from(camera_pos);
    let mut out = Vec::with_capacity(ecs.alive.len());

    for (_entity, transform, kind) in ecs.iter_transform_kind() {
        let y = heightmap.sample(transform.x, transform.y) + 1.0;
        let pos = glam::Vec3::new(transform.x, y, transform.y);
        let dist = (pos - cam).length();
        if lod_config.compute_lod(dist) == LodLevel::Culled || !frustum.test_sphere(pos, 2.0) {
            continue;
        }
        let color = match kind {
            EntityKind::Npc => [0.16, 0.47, 1.0],
            EntityKind::Monster(MonsterSpecies::Wolf) => [0.9, 0.9, 0.9],
            EntityKind::Monster(MonsterSpecies::Boar) => [0.55, 0.43, 0.39],
            EntityKind::Monster(MonsterSpecies::Bloodsucker) => [0.83, 0.0, 0.0],
        };
        out.push(EntityInstance {
            position: [transform.x, y, transform.y],
            color,
        });
    }
    out
}

pub(super) fn print_economy(engine: &Engine) {
    println!("--- NPCs ({}) ---", engine.ecs.count_npcs());
    for npc in engine.ecs.iter_npcs() {
        let name = npc.name.0.as_str();
        let goal = match engine.ecs.get_ai_state(npc.entity) {
            Some(AiState::Executing(g)) => format!("{}", g),
            _ => "Idle".into(),
        };
        let hp = npc.needs.map_or(1.0, |p| p.health);
        let hunger = npc.needs.map_or(0.0, |p| p.hunger);
        let money = npc.economy.map_or(0.0, |e| e.money);
        println!(
            "  {} | {} | hp:{:.0}% hunger:{:.0}% ${:.0}",
            name,
            goal,
            hp * 100.0,
            hunger * 100.0,
            money,
        );
    }
}
