#[derive(Clone, Debug)]
pub struct Checkpoint {
    pub tick: u64,
    pub ecs_snapshot: Vec<u8>,
    pub resource_snapshot: Vec<u8>,
    pub event_state: Vec<u8>,
}

pub struct CheckpointManager {
    checkpoints: Vec<Checkpoint>,
    interval_ticks: u64,
    max_checkpoints: usize,
}

impl CheckpointManager {
    pub fn new(interval_ticks: u64, max_checkpoints: usize) -> Self {
        Self {
            checkpoints: Vec::new(),
            interval_ticks,
            max_checkpoints,
        }
    }

    pub fn should_checkpoint(&self, tick: u64) -> bool {
        tick % self.interval_ticks == 0
    }

    pub fn save(&mut self, checkpoint: Checkpoint) {
        if self.checkpoints.len() >= self.max_checkpoints {
            self.checkpoints.remove(0);
        }
        self.checkpoints.push(checkpoint);
    }

    pub fn nearest_before(&self, tick: u64) -> Option<&Checkpoint> {
        self.checkpoints.iter().rev().find(|cp| cp.tick <= tick)
    }

    pub fn latest(&self) -> Option<&Checkpoint> {
        self.checkpoints.last()
    }

    pub fn count(&self) -> usize {
        self.checkpoints.len()
    }
}
