use crate::ai::offline_simulation;
use crate::core::ecs::Ecs;
use crate::ecosystem::food_chain;
use crate::world::components::{EntityKind, SimulationLevel};

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
        match ecs.kinds.get(&entity).cloned() {
            Some(EntityKind::Npc) => {
                offline_simulation::offline_npc_work(ecs, entity, elapsed);
            }
            Some(EntityKind::Monster(species)) => {
                let rank = food_chain::food_chain_rank(species);
                let hunger_rate = 0.005 * (1.0 + rank as f32 * 0.1);
                let energy_rate = 0.003 * (1.0 + rank as f32 * 0.05);
                if let Some(pn) = ecs.personal_needs.get_mut(&entity) {
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
        if let Some(pn) = ecs.personal_needs.get_mut(&entity) {
            pn.hunger = (pn.hunger + elapsed * 0.002).min(1.0);
            pn.energy = (pn.energy - elapsed * 0.001).max(0.0);
            if pn.hunger > 0.8 {
                pn.hunger -= 0.3;
                pn.health = (pn.health + 0.05).min(1.0);
            }
        }

        if let Some(econ) = ecs.npc_economies.get_mut(&entity) {
            econ.money += elapsed * 0.01;
        }
    }
}
