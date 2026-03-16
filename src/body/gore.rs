use crate::physics::damage_pipeline::response_aggregator::BodyZone;

#[derive(Clone, Debug)]
pub struct GoreState {
    pub exposed_zones: Vec<BodyZone>,
    pub flesh_chunks_spawned: u32,
    pub gore_level: f32,
}

impl GoreState {
    pub fn new() -> Self {
        Self {
            exposed_zones: Vec::new(),
            flesh_chunks_spawned: 0,
            gore_level: 0.0,
        }
    }

    pub fn expose_zone(&mut self, zone: BodyZone) {
        if !self.exposed_zones.contains(&zone) {
            self.exposed_zones.push(zone);
        }
        self.gore_level = (self.gore_level + 0.2).min(1.0);
    }

    pub fn spawn_flesh_chunks(&mut self, count: u32) {
        self.flesh_chunks_spawned += count;
    }
}
