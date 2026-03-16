//! Trait components defining entity personality.

use serde::{Deserialize, Serialize};

/// NPC personality traits (fixed at spawn).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpcTraits {
    pub bravery: f32,
    pub aggressiveness: f32,
    pub work_ethic: f32,
    pub curiosity: f32,
    pub honesty: f32,
    pub sociality: f32,
    pub autonomy: f32,
    pub materialism: f32,
    pub risk_tolerance: f32,
    pub stress_resistance: f32,
}

/// Monster personality traits (fixed at spawn).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterTraits {
    pub aggressiveness: f32,
    pub caution: f32,
    pub territoriality: f32,
    pub bravery: f32,
    pub pack_mentality: f32,
    pub energy_level: f32,
    pub hoarding: f32,
    pub curiosity: f32,
    pub adaptability: f32,
    pub stress_tolerance: f32,
}
