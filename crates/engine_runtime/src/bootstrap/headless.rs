use engine_core::plugin::EngineBuilder;
use engine_runtime::bootstrap::common::{finalize_builder, insert_runtime_core};
use engine_runtime::bootstrap::EngineRuntimeAssembly;
use engine_simulation::simulation::SimulationSystem;

impl EngineRuntimeAssembly {
    /// Engine-kernel bootstrap only. No game plugins, no game-data I/O, no world generation.
    pub fn kernel_headless() -> engine_core::engine::Engine {
        let mut builder = EngineBuilder::new();
        insert_runtime_core(
            &mut builder,
            engine_core::runtime_config::RuntimeConfig::headless(),
        );

        let center = engine_world::cell::WORLD_SIZE * 0.5;
        builder.add_system_default(Box::new(SimulationSystem::new(center, center)));

        finalize_builder(builder)
    }
}
