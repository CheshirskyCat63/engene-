use serde::{Deserialize, Serialize};
use crate::core::ecs::Entity;
use crate::memory::component_delta::ComponentDelta;

pub const CHANNEL_RELIABLE: u8 = 0;
pub const CHANNEL_UNRELIABLE: u8 = 1;
pub const MAX_CLIENTS: usize = 32;
pub const SERVER_TICK_RATE: f32 = 20.0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Connect { player_name: String },
    Disconnect,
    Input(PlayerInput),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerInput {
    pub tick: u64,
    pub move_dir: [f32; 2],
    pub look_dir: [f32; 2],
    pub actions: u32,
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            tick: 0,
            move_dir: [0.0; 2],
            look_dir: [0.0; 2],
            actions: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Welcome {
        client_id: u64,
        player_entity: Entity,
        tick: u64,
    },
    StateUpdate {
        tick: u64,
        deltas: Vec<ComponentDelta>,
    },
    EntitySpawn {
        entity: Entity,
        kind: u8,
        x: f32,
        y: f32,
    },
    EntityDespawn {
        entity: Entity,
    },
    InputAck {
        tick: u64,
    },
}
