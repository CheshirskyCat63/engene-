use crate::core::mutation_policy::*;
use crate::core::system::EngineSystem;
use crate::core::system_descriptor::{DeterminismTier, SystemDescriptor};
use crate::simulation::activation;

pub struct SimulationSystem {
    player_x: f32,
    player_y: f32,
}

impl SimulationSystem {
    pub fn new(player_x: f32, player_y: f32) -> Self {
        Self { player_x, player_y }
    }
}

impl EngineSystem for SimulationSystem {
    fn name(&self) -> &str {
        "Simulation"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("Simulation")
            .with_parallel(true)
            .with_determinism(DeterminismTier::Hard)
            .before("AI")
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        activation::update_simulation_levels(ctx.ecs, self.player_x, self.player_y);
    }
}
