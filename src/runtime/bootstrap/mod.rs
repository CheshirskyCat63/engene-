mod common;
pub mod game;
pub mod game_plugins;
pub mod game_resources;
pub mod game_systems;
pub mod headless;
pub mod tools;

pub struct EngineRuntimeAssembly;
pub struct GameRuntimeAssembly;
pub struct ToolsRuntimeAssembly;

#[cfg(feature = "physics")]
/// Helper to get a minimal physics bootstrap object for validation.
///
/// This is a small, real production seam that tests can call to ensure
/// physics bootstrap path is available in platform code.
pub fn minimal_physics_bootstrap_for_validation() -> engine_physics::bootstrap::PhysicsBootstrap {
    engine_physics::bootstrap::PhysicsBootstrap::enabled_minimal()
}

#[cfg(feature = "physics")]
/// Helper to get a disabled physics bootstrap object for validation.
pub fn disabled_physics_bootstrap_for_validation() -> engine_physics::bootstrap::PhysicsBootstrap {
    engine_physics::bootstrap::PhysicsBootstrap::disabled()
}

#[cfg(feature = "physics")]
/// Helper for minimal physics system registration in runtime wiring.
pub fn physics_tick_system_descriptor() -> crate::core::system_descriptor::SystemDescriptor {
    crate::core::system_descriptor::SystemDescriptor::new(
        engine_physics::systems::PHYSICS_TICK_SYSTEM_NAME,
    )
}
