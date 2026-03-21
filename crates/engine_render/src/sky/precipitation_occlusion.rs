//! Phase 7.5: Sky visibility sampling and roof occlusion for precipitation.

/// Configuration for precipitation occlusion sampling.
#[derive(Clone, Debug)]
pub struct PrecipOcclusionConfig {
    pub resolution: u32,
    pub update_interval: u32,
}

impl Default for PrecipOcclusionConfig {
    fn default() -> Self {
        Self {
            resolution: 64,
            update_interval: 4,
        }
    }
}

/// Tracks frame counter for periodic occlusion updates.
pub struct PrecipOcclusionSystem {
    pub config: PrecipOcclusionConfig,
    pub last_updated_frame: u32,
}

impl PrecipOcclusionSystem {
    pub fn new(resolution: u32) -> Self {
        Self {
            config: PrecipOcclusionConfig {
                resolution,
                update_interval: 4,
            },
            last_updated_frame: 0,
        }
    }

    pub fn should_update(&self, frame: u32) -> bool {
        frame.saturating_sub(self.last_updated_frame) >= self.config.update_interval
    }

    pub fn mark_updated(&mut self, frame: u32) {
        self.last_updated_frame = frame;
    }
}
