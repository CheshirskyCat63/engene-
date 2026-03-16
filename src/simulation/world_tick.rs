use crate::core::mutation_policy::*;
use crate::core::scheduler::Scheduler;
use crate::core::system::EngineSystem;
use crate::core::system_descriptor::{DeterminismTier, SystemDescriptor};
use crate::ecosystem::environment;
use crate::ecosystem::migration;
use crate::simulation::background_world;
use crate::world::resources::ResourceGrid;
use crate::world::world::WorldGrid;

#[derive(Clone, Debug)]
pub struct NewDay {
    pub day: u32,
    pub month: u32,
}

#[derive(Clone, Debug)]
pub struct NewMonth(pub u32);

pub struct WorldTickSystem {
    pub grid: WorldGrid,
    scheduler: Scheduler,
}

impl WorldTickSystem {
    pub fn new(grid: WorldGrid) -> Self {
        Self {
            grid,
            scheduler: Scheduler::new(5.0),
        }
    }
}

impl EngineSystem for WorldTickSystem {
    fn name(&self) -> &str {
        "WorldTick"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("WorldTick")
            .with_determinism(DeterminismTier::Hard)
            .after("Simulation")
            .before("AI")
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        if !self.scheduler.accumulate(ctx.time.delta) {
            return;
        }

        let season = ctx.time.season();
        let food_mult = season.food_regen_mult();
        let danger_mult = season.danger_mult();

        self.grid.regenerate_food_scaled(5.0, food_mult);
        environment::consume_food(ctx.ecs, &mut self.grid);
        environment::update_cell_danger_scaled(ctx.ecs, &mut self.grid, danger_mult);
        migration::process_migration(ctx.ecs, ctx.events, &self.grid);
        background_world::process_l1_entities(ctx.ecs, 5.0);
        background_world::process_l2_entities(ctx.ecs, 5.0);

        let biomes: Vec<_> = self.grid.cells.iter().map(|c| c.biome).collect();
        if let Some(resources) = ctx.resources.get_mut::<ResourceGrid>() {
            resources.tick(5.0, &biomes);
        }
    }
}
