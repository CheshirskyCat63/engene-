//! Phase 13.8: Simulation determinism policy and seeded RNG.

pub struct WeatherDeterminism {
    pub master_seed: u64,
    state: u64,
}

impl WeatherDeterminism {
    pub fn new(seed: u64) -> Self {
        Self { master_seed: seed, state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() & 0x00FFFFFF) as f32 / 16777216.0
    }

    pub fn fork(&self, salt: u64) -> Self {
        Self::new(self.master_seed.wrapping_add(salt))
    }
}
