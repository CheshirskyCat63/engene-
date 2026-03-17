use rand::Rng;

use crate::core::ecs::{Ecs, Entity};
use crate::world::components::*;

pub fn offline_combat(ecs: &mut Ecs, attacker: Entity, defender: Entity) -> bool {
    let atk_power = combat_power(ecs, attacker);
    let def_power = combat_power(ecs, defender);

    let mut rng = rand::thread_rng();
    let roll: f32 = rng.gen_range(0.0..1.0);
    let chance = atk_power / (atk_power + def_power + 0.01);

    roll < chance
}

fn combat_power(ecs: &Ecs, entity: Entity) -> f32 {
    let hp = ecs.get_needs(entity).map_or(1.0, |pn| pn.health);
    let energy = ecs.get_needs(entity).map_or(0.5, |pn| pn.energy);

    let trait_bonus = if let Some(t) = ecs.get_npc_traits(entity) {
        t.bravery * 0.3 + t.aggressiveness * 0.4
    } else if let Some(t) = ecs.get_monster_traits(entity) {
        t.bravery * 0.3 + t.aggressiveness * 0.4
    } else {
        0.3
    };

    (hp * 0.4 + energy * 0.3 + trait_bonus * 0.3).max(0.05)
}

pub fn offline_npc_work(ecs: &mut Ecs, entity: Entity, elapsed: f32) {
    let income = {
        let econ = match ecs.get_npc_economy(entity) {
            Some(e) => e,
            None => return,
        };
        let rate = match econ.job {
            Job::Guard => 0.8,
            Job::Trader => 1.0,
            Job::ArtifactHunter => 1.5,
            Job::Bandit => 1.8,
            Job::Hunter => 1.2,
            Job::Scavenger => 0.9,
            Job::Courier => 0.7,
            Job::Resident => 0.3,
            Job::Unemployed => 0.0,
        };
        elapsed * rate
    };

    if let Some(econ) = ecs.get_npc_economy_mut(entity) {
        econ.money += income;
    }
}
