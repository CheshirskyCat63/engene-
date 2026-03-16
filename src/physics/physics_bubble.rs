use crate::core::ecs::{Ecs, Entity};
use crate::world::components::SimulationLevel;

pub fn collect_l0(ecs: &Ecs) -> Vec<Entity> {
    ecs.alive
        .iter()
        .copied()
        .filter(|e| {
            ecs.sim_levels
                .get(e)
                .map_or(false, |s| s.level == SimulationLevel::L0)
        })
        .collect()
}
