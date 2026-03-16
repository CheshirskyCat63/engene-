use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::Instant;

use crate::core::ecs::Entity;
use crate::memory::component_delta::{DirtyFlags, collect_transform_deltas, COMP_TRANSFORM};
use crate::network::protocol::*;

pub struct ConnectedClient {
    pub client_id: u64,
    pub player_entity: Entity,
    pub last_acked_tick: u64,
    pub connected_at: Instant,
}

pub struct GameServer {
    socket: Option<UdpSocket>,
    clients: HashMap<u64, ConnectedClient>,
    next_client_id: u64,
    pub dirty: DirtyFlags,
    tick: u64,
    running: bool,
}

impl GameServer {
    pub fn new() -> Self {
        Self {
            socket: None,
            clients: HashMap::new(),
            next_client_id: 1,
            dirty: DirtyFlags::default(),
            tick: 0,
            running: false,
        }
    }

    pub fn start(&mut self, port: u16) -> Result<(), String> {
        let addr = format!("0.0.0.0:{}", port);
        let socket = UdpSocket::bind(&addr).map_err(|e| format!("bind: {e}"))?;
        socket.set_nonblocking(true).map_err(|e| format!("nonblocking: {e}"))?;
        self.socket = Some(socket);
        self.running = true;
        tracing::info!("server started on {}", addr);
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn advance_tick(&mut self) {
        self.tick += 1;
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    pub fn receive(&mut self) {
        let socket = match &self.socket {
            Some(s) => s,
            None => return,
        };

        let mut buf = [0u8; 4096];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((len, addr)) => {
                    if let Ok(msg) = bincode::deserialize::<ClientMessage>(&buf[..len]) {
                        match msg {
                            ClientMessage::Connect { player_name } => {
                                let cid = self.next_client_id;
                                self.next_client_id += 1;
                                tracing::info!("client {} connected: {}", cid, player_name);
                                self.clients.insert(cid, ConnectedClient {
                                    client_id: cid,
                                    player_entity: cid,
                                    last_acked_tick: 0,
                                    connected_at: Instant::now(),
                                });
                                let welcome = ServerMessage::Welcome {
                                    client_id: cid,
                                    player_entity: cid,
                                    tick: self.tick,
                                };
                                if let Ok(data) = bincode::serialize(&welcome) {
                                    let _ = socket.send_to(&data, addr);
                                }
                            }
                            ClientMessage::Disconnect => {
                                tracing::info!("client disconnected from {}", addr);
                            }
                            ClientMessage::Input(input) => {
                                let _ = input;
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }
    }

    pub fn broadcast_deltas(
        &mut self,
        transforms: &crate::core::sparse_set::SparseSet<crate::world::components::Transform>,
    ) {
        let dirty_entities = self.dirty.dirty_entities_for(COMP_TRANSFORM);
        if dirty_entities.is_empty() {
            return;
        }

        let delta = collect_transform_deltas(&dirty_entities, transforms);
        let msg = ServerMessage::StateUpdate {
            tick: self.tick,
            deltas: vec![delta],
        };

        let data = match bincode::serialize(&msg) {
            Ok(d) => d,
            Err(_) => return,
        };

        if let Some(socket) = &self.socket {
            for _client in self.clients.values() {
                let _ = socket.send_to(&data, "127.0.0.1:0");
            }
        }

        self.dirty.clear();
    }
}
