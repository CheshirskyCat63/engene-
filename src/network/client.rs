use std::collections::VecDeque;
use std::net::UdpSocket;

use crate::core::ecs::Entity;
use crate::memory::component_delta::{apply_transform_deltas, COMP_TRANSFORM};
use crate::network::interpolation::InterpolationBuffer;
use crate::network::protocol::*;

pub struct GameClient {
    socket: Option<UdpSocket>,
    pub server_addr: String,
    pub client_id: u64,
    pub player_entity: Entity,
    pub server_tick: u64,
    pub pending_inputs: VecDeque<PlayerInput>,
    pub interpolation: InterpolationBuffer,
    connected: bool,
}

impl GameClient {
    pub fn new() -> Self {
        Self {
            socket: None,
            server_addr: String::new(),
            client_id: 0,
            player_entity: 0,
            server_tick: 0,
            pending_inputs: VecDeque::with_capacity(64),
            interpolation: InterpolationBuffer::new(),
            connected: false,
        }
    }

    pub fn connect(&mut self, server_addr: &str) -> Result<(), String> {
        let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("bind: {e}"))?;
        socket.set_nonblocking(true).map_err(|e| format!("nonblocking: {e}"))?;
        self.server_addr = server_addr.to_string();

        let msg = ClientMessage::Connect {
            player_name: "Player".into(),
        };
        let data = bincode::serialize(&msg).map_err(|e| format!("serialize: {e}"))?;
        socket.send_to(&data, server_addr).map_err(|e| format!("send: {e}"))?;

        self.socket = Some(socket);
        tracing::info!("connecting to {}", server_addr);
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn send_input(&mut self, input: PlayerInput) {
        self.pending_inputs.push_back(input.clone());
        if self.pending_inputs.len() > 60 {
            self.pending_inputs.pop_front();
        }

        if let Some(socket) = &self.socket {
            let msg = ClientMessage::Input(input);
            if let Ok(data) = bincode::serialize(&msg) {
                let _ = socket.send_to(&data, &self.server_addr);
            }
        }
    }

    pub fn receive(
        &mut self,
        transforms: &mut crate::core::sparse_set::SparseSet<crate::world::components::Transform>,
    ) {
        if self.socket.is_none() {
            return;
        }

        let mut messages = Vec::new();
        let mut buf = [0u8; 65536];
        loop {
            let recv = self.socket.as_ref().unwrap().recv_from(&mut buf);
            match recv {
                Ok((len, _addr)) => {
                    if let Ok(msg) = bincode::deserialize::<ServerMessage>(&buf[..len]) {
                        messages.push(msg);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }

        for msg in messages {
            self.handle_message(msg, transforms);
        }
    }

    fn handle_message(
        &mut self,
        msg: ServerMessage,
        transforms: &mut crate::core::sparse_set::SparseSet<crate::world::components::Transform>,
    ) {
        match msg {
            ServerMessage::Welcome { client_id, player_entity, tick } => {
                self.client_id = client_id;
                self.player_entity = player_entity;
                self.server_tick = tick;
                self.connected = true;
                tracing::info!("connected as client {} entity {}", client_id, player_entity);
            }
            ServerMessage::StateUpdate { tick, deltas } => {
                self.server_tick = tick;
                for delta in &deltas {
                    match delta.component_type_id {
                        t if t == COMP_TRANSFORM as u16 => {
                            self.interpolation.push_snapshot(tick, delta.clone());
                            apply_transform_deltas(delta, transforms);
                        }
                        _ => {}
                    }
                }

                while self.pending_inputs.front().map_or(false, |i| i.tick <= tick) {
                    self.pending_inputs.pop_front();
                }
            }
            ServerMessage::InputAck { tick } => {
                while self.pending_inputs.front().map_or(false, |i| i.tick <= tick) {
                    self.pending_inputs.pop_front();
                }
            }
            ServerMessage::EntitySpawn { .. } => {}
            ServerMessage::EntityDespawn { .. } => {}
        }
    }
}
