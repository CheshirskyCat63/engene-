use crate::core::ecs::Ecs;
use crate::simulation::simulation_level::level_for_distance;
use crate::world::components::SimLevel;

pub fn update_simulation_levels(ecs: &mut Ecs, player_x: f32, player_y: f32) {
    let entities: Vec<_> = ecs.alive.clone();

    for entity in entities {
        let distance = {
            let t = match ecs.transforms.get(&entity) {
                Some(t) => t,
                None => continue,
            };
            ((t.x - player_x).powi(2) + (t.y - player_y).powi(2)).sqrt()
        };

        let new_level = level_for_distance(distance);
        ecs.sim_levels
            .insert(entity, SimLevel { level: new_level });
    }
}
