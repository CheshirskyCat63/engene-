use engine_world::components::Goal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub goal: Goal,
    pub target_pos: Option<(f32, f32)>,
    pub started_tick: u64,
    pub max_duration: f32,
    pub elapsed: f32,
}

impl Plan {
    pub fn new(goal: Goal, target: Option<(f32, f32)>, tick: u64) -> Self {
        let max_duration = match goal {
            Goal::SeekFood | Goal::SeekWater => 120.0,
            Goal::Hunt => 90.0,
            Goal::Flee => 40.0,
            Goal::Explore => 180.0,
            Goal::Socialize => 100.0,
            Goal::Work | Goal::Trade => 150.0,
            Goal::Rest => 200.0,
            Goal::Migrate => 300.0,
            Goal::DefendTerritory => 80.0,
            Goal::FollowPack => 120.0,
            Goal::StealOrRob => 60.0,
            Goal::Mate => 80.0,
            Goal::SeekShelter => 100.0,
            Goal::Sleep => 250.0,
            Goal::DoQuest => 200.0,
            Goal::StayAtPost => 300.0,
            Goal::RepairEquipment => 80.0,
            Goal::BuySupplies => 80.0,
        };
        Self {
            goal,
            target_pos: target,
            started_tick: tick,
            max_duration,
            elapsed: 0.0,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.elapsed >= self.max_duration
    }

    pub fn tick(&mut self, delta: f32) {
        self.elapsed += delta;
    }

    pub fn reached_target(&self, my_x: f32, my_y: f32) -> bool {
        match self.target_pos {
            Some((tx, ty)) => (tx - my_x).powi(2) + (ty - my_y).powi(2) < 20.0 * 20.0,
            None => false,
        }
    }
}
