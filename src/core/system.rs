// MIGRATION LAYER - Use engine_runtime::simulation_core::systems::engine_system::EngineSystem
// This re-exports the canonical EngineSystem for compatibility during migration

pub use engine_ecs::system_descriptor::SystemDescriptor;
pub use engine_runtime::simulation_core::systems::engine_system::{
    EngineSystem,
    SystemTickContext, 
    FixedTickContext, 
    RenderTickContext
};
