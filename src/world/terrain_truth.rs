/// Terrain Truth Layer (Phase B.3)
/// Terrain is not just visual -- it is a semantic material of the world.
/// Each terrain cell has physical, acoustic, ecological, and navigational properties.

use serde::{Serialize, Deserialize};

/// Physical properties of terrain at a given point
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainPhysicalMaterial {
    /// Movement speed multiplier (1.0 = normal, 0.5 = half speed)
    pub traversal_cost: f32,
    /// Cover quality (0.0 = none, 1.0 = full cover)
    pub cover_quality: f32,
    /// Movement penalty from terrain type
    pub movement_penalty: f32,
    /// Whether vehicles can traverse (future use)
    pub vehicle_passable: bool,
}

/// Acoustic properties of terrain
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainAcousticMaterial {
    /// Footstep sound class index
    pub footstep_sound_class: FootstepSoundClass,
    /// Sound propagation modifier (1.0 = normal, 0.5 = muffled)
    pub sound_propagation: f32,
    /// Sound absorption (soft ground absorbs more)
    pub absorption: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FootstepSoundClass {
    Dirt,
    Grass,
    Stone,
    Wood,
    Metal,
    Water,
    Mud,
    Sand,
    Snow,
    Gravel,
}

/// Ecological properties
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainEcologyMaterial {
    /// Vegetation suitability (0.0 = barren, 1.0 = lush)
    pub vegetation_suitability: f32,
    /// Fire spread rate multiplier
    pub fire_spread_rate: f32,
    /// Food growth rate for ecosystem
    pub food_growth_rate: f32,
    /// Water retention (affects puddle formation)
    pub water_retention: f32,
}

/// Moisture and deformation behavior
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainMoistureBehavior {
    /// Current wetness (0.0 = dry, 1.0 = soaked)
    pub wetness: f32,
    /// How quickly wetness evaporates (per second)
    pub evaporation_rate: f32,
    /// How quickly tracks/footprints fade
    pub deformation_recovery_rate: f32,
    /// Maximum puddle depth at this point
    pub puddle_capacity: f32,
}

/// Complete terrain truth for a single cell/sample point
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainTruth {
    pub physical: TerrainPhysicalMaterial,
    pub acoustic: TerrainAcousticMaterial,
    pub ecology: TerrainEcologyMaterial,
    pub moisture: TerrainMoistureBehavior,
}

impl TerrainTruth {
    /// Generate terrain truth from biome type
    pub fn from_biome(biome: u8) -> Self {
        match biome {
            0 => Self::grassland(),
            1 => Self::forest(),
            2 => Self::swamp(),
            3 => Self::settlement(),
            4 => Self::mountain(),
            _ => Self::grassland(),
        }
    }

    pub fn grassland() -> Self {
        Self {
            physical: TerrainPhysicalMaterial {
                traversal_cost: 1.0, cover_quality: 0.1, movement_penalty: 0.0, vehicle_passable: true,
            },
            acoustic: TerrainAcousticMaterial {
                footstep_sound_class: FootstepSoundClass::Grass, sound_propagation: 1.0, absorption: 0.3,
            },
            ecology: TerrainEcologyMaterial {
                vegetation_suitability: 0.7, fire_spread_rate: 0.8, food_growth_rate: 1.0, water_retention: 0.3,
            },
            moisture: TerrainMoistureBehavior {
                wetness: 0.2, evaporation_rate: 0.05, deformation_recovery_rate: 0.02, puddle_capacity: 0.1,
            },
        }
    }

    pub fn forest() -> Self {
        Self {
            physical: TerrainPhysicalMaterial {
                traversal_cost: 1.3, cover_quality: 0.6, movement_penalty: 0.15, vehicle_passable: false,
            },
            acoustic: TerrainAcousticMaterial {
                footstep_sound_class: FootstepSoundClass::Dirt, sound_propagation: 0.7, absorption: 0.5,
            },
            ecology: TerrainEcologyMaterial {
                vegetation_suitability: 1.0, fire_spread_rate: 1.2, food_growth_rate: 0.8, water_retention: 0.6,
            },
            moisture: TerrainMoistureBehavior {
                wetness: 0.4, evaporation_rate: 0.02, deformation_recovery_rate: 0.01, puddle_capacity: 0.3,
            },
        }
    }

    pub fn swamp() -> Self {
        Self {
            physical: TerrainPhysicalMaterial {
                traversal_cost: 1.8, cover_quality: 0.3, movement_penalty: 0.4, vehicle_passable: false,
            },
            acoustic: TerrainAcousticMaterial {
                footstep_sound_class: FootstepSoundClass::Mud, sound_propagation: 0.6, absorption: 0.7,
            },
            ecology: TerrainEcologyMaterial {
                vegetation_suitability: 0.5, fire_spread_rate: 0.2, food_growth_rate: 0.6, water_retention: 0.9,
            },
            moisture: TerrainMoistureBehavior {
                wetness: 0.8, evaporation_rate: 0.01, deformation_recovery_rate: 0.005, puddle_capacity: 0.8,
            },
        }
    }

    pub fn settlement() -> Self {
        Self {
            physical: TerrainPhysicalMaterial {
                traversal_cost: 0.9, cover_quality: 0.4, movement_penalty: 0.0, vehicle_passable: true,
            },
            acoustic: TerrainAcousticMaterial {
                footstep_sound_class: FootstepSoundClass::Stone, sound_propagation: 1.2, absorption: 0.1,
            },
            ecology: TerrainEcologyMaterial {
                vegetation_suitability: 0.1, fire_spread_rate: 0.5, food_growth_rate: 0.2, water_retention: 0.1,
            },
            moisture: TerrainMoistureBehavior {
                wetness: 0.1, evaporation_rate: 0.1, deformation_recovery_rate: 0.0, puddle_capacity: 0.05,
            },
        }
    }

    pub fn mountain() -> Self {
        Self {
            physical: TerrainPhysicalMaterial {
                traversal_cost: 2.0, cover_quality: 0.5, movement_penalty: 0.5, vehicle_passable: false,
            },
            acoustic: TerrainAcousticMaterial {
                footstep_sound_class: FootstepSoundClass::Stone, sound_propagation: 1.3, absorption: 0.05,
            },
            ecology: TerrainEcologyMaterial {
                vegetation_suitability: 0.2, fire_spread_rate: 0.3, food_growth_rate: 0.3, water_retention: 0.1,
            },
            moisture: TerrainMoistureBehavior {
                wetness: 0.1, evaporation_rate: 0.08, deformation_recovery_rate: 0.0, puddle_capacity: 0.02,
            },
        }
    }
}
