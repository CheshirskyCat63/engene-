// MIGRATION LAYER - Use engine_runtime::simulation_core::systems::engine_system::EngineSystem
// This provides a minimal compatibility alias during migration

pub use engine_ecs::system_descriptor::SystemDescriptor;
pub use engine_runtime::simulation_core::systems::engine_system::{
    SystemTickContext, 
    FixedTickContext, 
    RenderTickContext
};

// Re-export the canonical trait directly
pub use engine_runtime::simulation_core::systems::engine_system::EngineSystem;
