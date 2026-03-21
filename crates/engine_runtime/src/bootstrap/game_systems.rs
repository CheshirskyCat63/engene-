use std::sync::Arc;

use engine_core::plugin::EngineBuilder;
use crate::wiring::animation::AnimationIntegrationSystem;
use crate::wiring::integration::{
    AiDecisionWireSystem, AnimationWireSystem, BallisticsTickSystem, DamageDispatchSystem,
    DestructionTickSystem, GoreWireSystem, NavDirtyTickSystem, OcclusionWireSystem,
    TerrainDeformationTickSystem,
};
use crate::wiring::world_tick::WorldTickSystem;
use engine_simulation::simulation::SimulationSystem;
use engine_world::heightmap::Heightmap;
use engine_world::world::WorldGrid;
use engine_physics::physics::PhysicsSystem;

pub(super) fn register_vertical_systems(
    builder: &mut EngineBuilder,
    grid: WorldGrid,
    heightmap: Arc<Heightmap>,
) {
    let center = engine_world::cell::WORLD_SIZE * 0.5;
    builder.add_system_default(Box::new(SimulationSystem::new(center, center)));
    builder.add_system_default(Box::new(WorldTickSystem::new()));
    builder.add_system_default(Box::new(engine_game::ai::ai::AiSystem::new()));
    builder.add_system_default(Box::new(AiDecisionWireSystem));
    builder.add_system_default(Box::new(PhysicsSystem::new(heightmap)));
    builder.add_system_default(Box::new(BallisticsTickSystem));
    builder.add_system_default(Box::new(DamageDispatchSystem));
    builder.add_system_default(Box::new(DestructionTickSystem));
    builder.add_system_default(Box::new(TerrainDeformationTickSystem));
    builder.add_system_default(Box::new(NavDirtyTickSystem));
    builder.add_system_default(Box::new(OcclusionWireSystem));
    builder.add_system_default(Box::new(GoreWireSystem));
    builder.add_system_default(Box::new(AnimationIntegrationSystem::new()));
    builder.add_system_default(Box::new(AnimationWireSystem::new()));
    builder.add_system_default(Box::new(
        engine_audio::audio_integration::AudioIntegrationSystem::new(),
    ));
    builder.add_system_default(Box::new(
        engine_game::gameplay::quest_system::QuestSystem::new(),
    ));
    builder.add_system_default(Box::new(engine_body::body_system::BodySystem));
    builder.add_system_default(Box::new(engine_render::render_system::RenderSystem));
    builder.add_system_default(Box::new(engine_audio::playback::AudioPlaybackBridge::new()));
    builder.add_system_default(Box::new(
        engine_input::input_system::InputActionSystem::new(),
    ));
    #[cfg(feature = "networking")]
    builder.add_system_default(Box::new(
        engine_network::network_system::NetworkSystem::new(),
    ));
}

pub(super) fn register_headless_systems(
    builder: &mut EngineBuilder,
    grid: WorldGrid,
    heightmap: Arc<Heightmap>,
) {
    let center = engine_world::cell::WORLD_SIZE * 0.5;
    builder.add_system_default(Box::new(SimulationSystem::new(center, center)));
    builder.add_system_default(Box::new(WorldTickSystem::new()));
    builder.add_system_default(Box::new(engine_game::ai::ai::AiSystem::new()));
    builder.add_system_default(Box::new(AiDecisionWireSystem));
    builder.add_system_default(Box::new(PhysicsSystem::new(heightmap)));
    builder.add_system_default(Box::new(BallisticsTickSystem));
    builder.add_system_default(Box::new(DamageDispatchSystem));
    builder.add_system_default(Box::new(DestructionTickSystem));
    builder.add_system_default(Box::new(TerrainDeformationTickSystem));
    builder.add_system_default(Box::new(NavDirtyTickSystem));
    builder.add_system_default(Box::new(OcclusionWireSystem));
    builder.add_system_default(Box::new(GoreWireSystem));
    builder.add_system_default(Box::new(AnimationIntegrationSystem::new()));
    builder.add_system_default(Box::new(AnimationWireSystem::new()));
    builder.add_system_default(Box::new(
        engine_game::gameplay::quest_system::QuestSystem::new(),
    ));
    builder.add_system_default(Box::new(engine_body::body_system::BodySystem));
    #[cfg(feature = "networking")]
    builder.add_system_default(Box::new(
        engine_network::network_system::NetworkSystem::new(),
    ));
}
