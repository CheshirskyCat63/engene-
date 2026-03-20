/// Vegetation as World Semantics (Phase B.4 V1)
/// Vegetation is not just render -- it has gameplay meaning.
use serde::{Deserialize, Serialize};

/// How vegetation affects line of sight
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OcclusionClass {
    /// Does not block LOS (short grass)
    None,
    /// Partially blocks LOS (bushes, tall grass)
    Partial,
    /// Fully blocks LOS (dense trees, thick bushes)
    Full,
}

/// Vegetation semantic properties for a vegetation instance or cluster
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VegetationSemantics {
    /// How this vegetation affects line of sight
    pub occlusion: OcclusionClass,
    /// Concealment score: reduces AI detection range (0.0 = none, 1.0 = hidden)
    pub concealment_score: f32,
    /// Movement penalty when passing through (0.0 = none, 1.0 = impassable)
    pub movement_penalty: f32,
    /// Flammability: fire spread rate multiplier (0.0 = fireproof, 2.0 = very flammable)
    pub flammability: f32,
    /// Sound response: volume of rustling when entities pass through
    pub sound_response: f32,
    /// Whether this vegetation can be trampled/destroyed
    pub destructible: bool,
    /// Current damage state (0.0 = pristine, 1.0 = destroyed)
    pub damage: f32,
}

impl VegetationSemantics {
    /// Short grass -- minimal gameplay impact
    pub fn short_grass() -> Self {
        Self {
            occlusion: OcclusionClass::None,
            concealment_score: 0.05,
            movement_penalty: 0.0,
            flammability: 1.0,
            sound_response: 0.1,
            destructible: true,
            damage: 0.0,
        }
    }

    /// Tall grass / bushes -- good concealment
    pub fn tall_grass() -> Self {
        Self {
            occlusion: OcclusionClass::Partial,
            concealment_score: 0.5,
            movement_penalty: 0.1,
            flammability: 1.2,
            sound_response: 0.4,
            destructible: true,
            damage: 0.0,
        }
    }

    /// Dense bush -- blocks sight, slows movement
    pub fn dense_bush() -> Self {
        Self {
            occlusion: OcclusionClass::Full,
            concealment_score: 0.8,
            movement_penalty: 0.3,
            flammability: 0.8,
            sound_response: 0.6,
            destructible: true,
            damage: 0.0,
        }
    }

    /// Tree -- blocks sight at trunk, provides cover
    pub fn tree() -> Self {
        Self {
            occlusion: OcclusionClass::Full,
            concealment_score: 0.7,
            movement_penalty: 0.0,
            flammability: 0.6,
            sound_response: 0.2,
            destructible: false,
            damage: 0.0,
        }
    }

    /// Dead tree -- less concealment, more flammable
    pub fn dead_tree() -> Self {
        Self {
            occlusion: OcclusionClass::Partial,
            concealment_score: 0.3,
            movement_penalty: 0.0,
            flammability: 1.5,
            sound_response: 0.1,
            destructible: true,
            damage: 0.0,
        }
    }

    /// Apply trampling damage
    pub fn trample(&mut self, amount: f32) {
        if self.destructible {
            self.damage = (self.damage + amount).min(1.0);
            self.concealment_score *= 1.0 - self.damage * 0.5;
            self.sound_response *= 1.0 - self.damage * 0.3;
        }
    }

    /// Check if this vegetation is effectively destroyed
    pub fn is_destroyed(&self) -> bool {
        self.damage >= 0.95
    }
}
