use std::sync::Arc;

use crate::core::plugin::EngineBuilder;
use crate::physics::physics::PhysicsSystem;
use crate::runtime::wiring::animation::AnimationIntegrationSystem;
use crate::runtime::wiring::integration::{
    AiDecisionWireSystem, AnimationWireSystem, BallisticsTickSystem, DamageDispatchSystem,
    DestructionTickSystem, GoreWireSystem, NavDirtyTickSystem, OcclusionWireSystem,
    TerrainDeformationTickSystem,
};
use crate::runtime::wiring::world_tick::WorldTickSystem;
use crate::simulation::simulation::SimulationSystem;
use crate::world::heightmap::Heightmap;
use crate::world::world::WorldGrid;

pub(super) fn register_vertical_systems(
    builder: &mut EngineBuilder,
    grid: WorldGrid,
    heightmap: Arc<Heightmap>,
) {
    let center = crate::world::cell::WORLD_SIZE * 0.5;
    builder.add_system_default(Box::new(SimulationSystem::new(center, center)));
    builder.add_system_default(Box::new(WorldTickSystem::new(grid)));
    builder.add_system_default(Box::new(crate::game::ai::ai::AiSystem::new()));
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
        crate::audio::audio_integration::AudioIntegrationSystem::new(),
    ));
    builder.add_system_default(Box::new(
        crate::game::gameplay::quest_system::QuestSystem::new(),
    ));
    builder.add_system_default(Box::new(crate::body::body_system::BodySystem));
    builder.add_system_default(Box::new(crate::graphics::render_system::RenderSystem));
    builder.add_system_default(Box::new(crate::audio::playback::AudioPlaybackBridge::new()));
    builder.add_system_default(Box::new(
        crate::input::input_system::InputActionSystem::new(),
    ));
    #[cfg(feature = "networking")]
    builder.add_system_default(Box::new(
        crate::network::network_system::NetworkSystem::new(),
    ));
}

pub(super) fn register_headless_systems(
    builder: &mut EngineBuilder,
    grid: WorldGrid,
    heightmap: Arc<Heightmap>,
) {
    let center = crate::world::cell::WORLD_SIZE * 0.5;
    builder.add_system_default(Box::new(SimulationSystem::new(center, center)));
    builder.add_system_default(Box::new(WorldTickSystem::new(grid)));
    builder.add_system_default(Box::new(crate::game::ai::ai::AiSystem::new()));
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
        crate::game::gameplay::quest_system::QuestSystem::new(),
    ));
    builder.add_system_default(Box::new(crate::body::body_system::BodySystem));
    #[cfg(feature = "networking")]
    builder.add_system_default(Box::new(
        crate::network::network_system::NetworkSystem::new(),
    ));
}
