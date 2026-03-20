//! Life cycle components.

use super::entity_kind::MonsterSpecies;
use serde::{Deserialize, Serialize};

/// Life stage enumeration.
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum LifeStage {
    Young,
    Adult,
    Old,
}

impl LifeStage {
    pub fn speed_mult(&self) -> f32 {
        match self {
            Self::Young => 1.2,
            Self::Adult => 1.0,
            Self::Old => 0.7,
        }
    }

    pub fn combat_mult(&self) -> f32 {
        match self {
            Self::Young => 0.7,
            Self::Adult => 1.0,
            Self::Old => 0.8,
        }
    }

    pub fn learning_mult(&self) -> f32 {
        match self {
            Self::Young => 1.5,
            Self::Adult => 1.0,
            Self::Old => 0.5,
        }
    }
}

/// Entity life information (age, reproduction).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifeInfo {
    pub age: f32,
    pub max_age: f32,
    pub last_mate_day: u32,
    pub mate_cooldown_days: u32,
}

impl LifeInfo {
    pub fn new_npc(max_age: f32) -> Self {
        Self {
            age: max_age * 0.25,
            max_age,
            last_mate_day: 0,
            mate_cooldown_days: 60,
        }
    }

    pub fn new_monster(species: MonsterSpecies) -> Self {
        let (max_age, cooldown) = match species {
            MonsterSpecies::Wolf => (200.0, 30),
            MonsterSpecies::Boar => (180.0, 45),
            MonsterSpecies::Bloodsucker => (500.0, 90),
        };
        Self {
            age: max_age * 0.3,
            max_age,
            last_mate_day: 0,
            mate_cooldown_days: cooldown,
        }
    }

    pub fn life_stage(&self) -> LifeStage {
        let ratio = self.age / self.max_age;
        if ratio < 0.2 {
            LifeStage::Young
        } else if ratio < 0.7 {
            LifeStage::Adult
        } else {
            LifeStage::Old
        }
    }

    pub fn can_mate(&self, current_day: u32) -> bool {
        current_day.saturating_sub(self.last_mate_day) >= self.mate_cooldown_days
    }

    pub fn with_age(mut self, age: f32) -> Self {
        self.age = age;
        self
    }
}
