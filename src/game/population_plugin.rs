use crate::core::ecs::Ecs;
use crate::core::plugin::{EngineBuilder, Plugin};
use crate::core::registry::Resources;
use crate::world::population;

pub struct PopulationPlugin;

impl Plugin for PopulationPlugin {
    fn name(&self) -> &str {
        "PopulationPlugin"
    }

    fn build(&self, builder: &mut EngineBuilder) {
        builder.add_init_fn(|ecs: &mut Ecs, _res: &mut Resources| {
            population::spawn_npcs(ecs);
            population::spawn_monsters(ecs);
            println!("[population] spawned {} entities", ecs.alive.len());
        });
    }
}
