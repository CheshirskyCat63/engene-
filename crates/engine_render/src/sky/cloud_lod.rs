//! Phase 3.5: Cloud LOD system (full volumetric / simplified / impostor).

/// LOD level for cloud rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudLodLevel {
    /// Full volumetric raymarching (highest quality).
    Full,
    /// Simplified volumetric (fewer steps, lower resolution).
    Simplified,
    /// Billboard impostor or baked cloud.
    Impostor,
}

/// Distance-based LOD configuration for clouds.
#[derive(Debug, Clone)]
pub struct CloudLodConfig {
    /// Distance threshold: below this, use Full LOD.
    pub full_threshold: f32,
    /// Distance threshold: below this but above full_threshold, use Simplified LOD.
    pub simplified_threshold: f32,
    // Above simplified_threshold, use Impostor LOD.
}

impl Default for CloudLodConfig {
    fn default() -> Self {
        Self {
            full_threshold: 5000.0,
            simplified_threshold: 15000.0,
        }
    }
}

/// Returns the LOD level for a given distance using the default config.
pub fn get_lod_level(distance: f32) -> CloudLodLevel {
    CloudLodConfig::default().lod_from_distance(distance)
}

/// Returns the number of raymarch steps for the given LOD level.
pub fn get_ray_steps(level: CloudLodLevel) -> u32 {
    CloudLodConfig::default().steps_for_lod(level)
}

impl CloudLodConfig {
    /// Returns the LOD level for a given distance to the cloud layer.
    pub fn lod_from_distance(&self, distance: f32) -> CloudLodLevel {
        if distance < self.full_threshold {
            CloudLodLevel::Full
        } else if distance < self.simplified_threshold {
            CloudLodLevel::Simplified
        } else {
            CloudLodLevel::Impostor
        }
    }

    /// Number of raymarch steps for the given LOD level.
    pub fn steps_for_lod(&self, lod: CloudLodLevel) -> u32 {
        match lod {
            CloudLodLevel::Full => 64,
            CloudLodLevel::Simplified => 24,
            CloudLodLevel::Impostor => 0,
        }
    }

    /// Step size multiplier for the given LOD level (higher = coarser).
    pub fn step_scale_for_lod(&self, lod: CloudLodLevel) -> f32 {
        match lod {
            CloudLodLevel::Full => 1.0,
            CloudLodLevel::Simplified => 2.5,
            CloudLodLevel::Impostor => 1.0,
        }
    }
}
