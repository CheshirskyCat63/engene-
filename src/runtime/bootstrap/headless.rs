use crate::core::plugin::EngineBuilder;
use crate::runtime::bootstrap::common::{finalize_builder, insert_runtime_core};
use crate::runtime::bootstrap::EngineRuntimeAssembly;
use crate::simulation::simulation::SimulationSystem;

impl EngineRuntimeAssembly {
    /// Engine-kernel bootstrap only. No game plugins, no game-data I/O, no world generation.
    pub fn kernel_headless() -> engine_runtime::engine::Engine {
        let mut builder = EngineBuilder::new();
        insert_runtime_core(
            &mut builder,
            crate::core::runtime_config::RuntimeConfig::headless(),
        );

        let center = crate::world::cell::WORLD_SIZE * 0.5;
        builder.add_system_default(Box::new(SimulationSystem::new(center, center)));

        finalize_builder(builder)
    }
}
