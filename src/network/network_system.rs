// LEGACY IMPORTS - Use canonical crates instead
use engine_runtime::simulation_core::systems::engine_system::{EngineSystem, FixedTickContext};
use engine_ecs::system_descriptor::{DeterminismTier, SystemDescriptor};
use crate::core::system::EngineSystem as LegacyEngineSystem;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkMode {
    Offline,
    Client,
    Server,
    ListenServer,
}

pub struct NetworkState {
    pub mode: NetworkMode,
    pub connected: bool,
    pub ping_ms: f32,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

impl NetworkState {
    pub fn offline() -> Self {
        Self {
            mode: NetworkMode::Offline,
            connected: false,
            ping_ms: 0.0,
            bytes_sent: 0,
            bytes_received: 0,
        }
    }
}

pub struct NetworkSystem {
    pub mode: NetworkMode,
}

impl NetworkSystem {
    pub fn new() -> Self {
        Self {
            mode: NetworkMode::Offline,
        }
    }
}

impl LegacyEngineSystem for NetworkSystem {
    fn name(&self) -> &str {
        "NetworkSystem"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("NetworkSystem")
            .with_determinism(DeterminismTier::NonDeterministic)
            .with_headless(true)
    }

    fn register_resources(&mut self, res: &mut crate::core::registry::Resources) {
        res.insert(NetworkState::offline());
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let Some(state) = ctx.resources.get_mut::<NetworkState>() else {
            return;
        };
        if state.mode == NetworkMode::Offline {
            return;
        }
        let _ = state.ping_ms;
    }
}
