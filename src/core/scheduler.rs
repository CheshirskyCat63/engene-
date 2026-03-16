pub struct Scheduler {
    world_tick_interval: f32,
    accumulator: f32,
}

impl Scheduler {
    pub fn new(world_tick_interval: f32) -> Self {
        Self {
            world_tick_interval,
            accumulator: 0.0,
        }
    }

    pub fn accumulate(&mut self, delta: f32) -> bool {
        self.accumulator += delta;
        if self.accumulator >= self.world_tick_interval {
            self.accumulator -= self.world_tick_interval;
            return true;
        }
        false
    }
}
