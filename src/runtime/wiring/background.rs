use engine_game::ai::offline_simulation;
use engine_game::ecosystem::food_chain;
use engine_world::components::{EntityKind, SimulationLevel};
use engine_ecs::Ecs;

pub fn process_l1_entities(ecs: &mut Ecs, elapsed: f32) {
    let entities: Vec<_> = ecs
        .alive
        .iter()
        .copied()
        .filter(|e| {
            ecs.sim_levels
                .get(e)
                .map_or(false, |s| s.level == SimulationLevel::L1)
        })
        .collect();

    for entity in entities {
        match ecs.get_kind(entity).cloned() {
            Some(EntityKind::Npc) => {
                offline_simulation::offline_npc_work(ecs, entity, elapsed);
            }
            Some(EntityKind::Monster(species)) => {
                let rank = food_chain::food_chain_rank(species);
                let hunger_rate = 0.005 * (1.0 + rank as f32 * 0.1);
                let energy_rate = 0.003 * (1.0 + rank as f32 * 0.05);
                if let Some(pn) = ecs.get_needs_mut(entity) {
                    pn.hunger = (pn.hunger + elapsed * hunger_rate).min(1.0);
                    pn.energy = (pn.energy - elapsed * energy_rate).max(0.0);
                }
            }
            None => {}
        }
    }
}

pub fn process_l2_entities(ecs: &mut Ecs, elapsed: f32) {
    let entities: Vec<_> = ecs
        .alive
        .iter()
        .copied()
        .filter(|e| {
            ecs.sim_levels
                .get(e)
                .map_or(false, |s| s.level == SimulationLevel::L2)
        })
        .collect();

    for entity in entities {
        if let Some(pn) = ecs.get_needs_mut(entity) {
            pn.hunger = (pn.hunger + elapsed * 0.002).min(1.0);
            pn.energy = (pn.energy - elapsed * 0.001).max(0.0);
            if pn.hunger > 0.8 {
                pn.hunger -= 0.3;
                pn.health = (pn.health + 0.05).min(1.0);
            }
        }

        if let Some(econ) = ecs.get_npc_economy_mut(entity) {
            econ.money += elapsed * 0.01;
        }
    }
}
