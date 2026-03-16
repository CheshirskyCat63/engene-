use glam::Vec3;
use std::collections::VecDeque;

use crate::core::ecs::Entity;
use crate::physics::damage_taxonomy::DamageClass;

const MAX_CHAIN_DEPTH: u32 = 4;
const MAX_EVENTS_PER_FRAME: usize = 32;
const ENERGY_FLOOR: f32 = 10.0;

#[derive(Clone, Debug)]
pub struct ChainEvent {
    pub source_entity: Option<Entity>,
    pub position: Vec3,
    pub energy: f32,
    pub damage_class: DamageClass,
    pub depth: u32,
}

pub struct ChainReactionQueue {
    queue: VecDeque<ChainEvent>,
    processed_this_frame: usize,
}

impl ChainReactionQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(MAX_EVENTS_PER_FRAME),
            processed_this_frame: 0,
        }
    }

    pub fn submit(&mut self, event: ChainEvent) {
        if event.depth >= MAX_CHAIN_DEPTH {
            return;
        }
        if event.energy < ENERGY_FLOOR {
            return;
        }
        if self.queue.len() < MAX_EVENTS_PER_FRAME * 2 {
            self.queue.push_back(event);
        }
    }

    pub fn drain_batch(&mut self) -> Vec<ChainEvent> {
        self.processed_this_frame = 0;
        let count = self.queue.len().min(MAX_EVENTS_PER_FRAME);
        let batch: Vec<_> = self.queue.drain(..count).collect();
        self.processed_this_frame = batch.len();
        batch
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }
}
