use crate::memory::component_delta::ComponentDelta;
use std::collections::VecDeque;

const BUFFER_SIZE: usize = 4;
const INTERP_DELAY_MS: f32 = 100.0;

#[derive(Clone)]
struct Snapshot {
    tick: u64,
    time_received: f32,
    deltas: Vec<ComponentDelta>,
}

pub struct InterpolationBuffer {
    snapshots: VecDeque<Snapshot>,
    current_time: f32,
}

impl InterpolationBuffer {
    pub fn new() -> Self {
        Self {
            snapshots: VecDeque::with_capacity(BUFFER_SIZE + 1),
            current_time: 0.0,
        }
    }

    pub fn push_snapshot(&mut self, tick: u64, delta: ComponentDelta) {
        if let Some(last) = self.snapshots.back_mut() {
            if last.tick == tick {
                last.deltas.push(delta);
                return;
            }
        }
        self.snapshots.push_back(Snapshot {
            tick,
            time_received: self.current_time,
            deltas: vec![delta],
        });
        while self.snapshots.len() > BUFFER_SIZE {
            self.snapshots.pop_front();
        }
    }

    pub fn advance(&mut self, dt: f32) {
        self.current_time += dt;
    }

    pub fn interpolation_factor(&self) -> f32 {
        if self.snapshots.len() < 2 {
            return 1.0;
        }
        let render_time = self.current_time - INTERP_DELAY_MS / 1000.0;
        let a = &self.snapshots[self.snapshots.len() - 2];
        let b = &self.snapshots[self.snapshots.len() - 1];
        let dt = b.time_received - a.time_received;
        if dt <= 0.0 {
            return 1.0;
        }
        ((render_time - a.time_received) / dt).clamp(0.0, 1.0)
    }

    pub fn latest_tick(&self) -> u64 {
        self.snapshots.back().map_or(0, |s| s.tick)
    }

    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }
}
