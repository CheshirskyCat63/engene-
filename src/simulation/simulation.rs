// LEGACY IMPORTS - Use canonical crates instead
use engine_runtime::simulation_core::systems::engine_system::{EngineSystem, FixedTickContext, ExtractContext};
use engine_ecs::system_descriptor::{DeterminismTier, SystemDescriptor};
use engine_runtime::simulation_core::{
    DeferredTransitionPolicy, DeferredTransitionQueue, TransitionRequest,
};
use crate::core::system::EngineSystem as LegacyEngineSystem;
use crate::simulation::activation;

pub struct SimulationSystem {
    player_x: f32,
    player_y: f32,
    deferred_queue: DeferredTransitionQueue,
    transition_batch: Vec<TransitionRequest>,
}

impl SimulationSystem {
    pub fn new(player_x: f32, player_y: f32) -> Self {
        let queue_policy = DeferredTransitionPolicy::default();
        let deferred_queue = DeferredTransitionQueue::with_policy(queue_policy);
        let transition_batch = Vec::with_capacity(queue_policy.max_queue_capacity);
        debug_assert!(transition_batch.capacity() >= queue_policy.max_queue_capacity);
        Self {
            player_x,
            player_y,
            deferred_queue,
            transition_batch,
        }
    }
}

impl LegacyEngineSystem for SimulationSystem {
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
        let _transition_metrics = activation::update_simulation_levels(
            ctx.ecs,
            self.player_x,
            self.player_y,
            ctx.time.tick_count,
            ctx.time.tick_count,
            &mut self.deferred_queue,
            &mut self.transition_batch,
        );
    }
}
