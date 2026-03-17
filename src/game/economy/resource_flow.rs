use crate::core::ecs::Ecs;
use crate::world::components::EntityKind;

pub struct EconomySnapshot {
    pub total_npc_money: f32,
    pub average_desperation: f32,
    pub bandit_count: u32,
}

pub fn snapshot(ecs: &Ecs) -> EconomySnapshot {
    let mut total_money = 0.0_f32;
    let mut total_desp = 0.0_f32;
    let mut bandits = 0_u32;
    let mut count = 0_u32;

    for &e in &ecs.alive {
        if !matches!(ecs.get_kind(e), Some(EntityKind::Npc)) {
            continue;
        }
        if let Some(econ) = ecs.get_npc_economy(e) {
            total_money += econ.money;
            total_desp += econ.desperation;
            if econ.job == crate::world::components::Job::Bandit {
                bandits += 1;
            }
            count += 1;
        }
    }

    let n = count.max(1) as f32;
    EconomySnapshot {
        total_npc_money: total_money,
        average_desperation: total_desp / n,
        bandit_count: bandits,
    }
}
