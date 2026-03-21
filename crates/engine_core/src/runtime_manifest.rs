#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum QualityTier {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Clone, Debug)]
pub struct RuntimeManifest {
    pub destruction_enabled: bool,
    pub gore_enabled: bool,
    pub terrain_deformation_enabled: bool,
    pub micro_motion_quality: QualityTier,
    pub replay_capture: bool,
    pub debug_overlays_allowed: bool,
    pub governor_aggressiveness: f32,
    pub fire_simulation_enabled: bool,
    pub cloth_simulation_enabled: bool,
    pub surface_state_enabled: bool,
}

impl Default for RuntimeManifest {
    fn default() -> Self {
        Self {
            destruction_enabled: true,
            gore_enabled: true,
            terrain_deformation_enabled: true,
            micro_motion_quality: QualityTier::High,
            replay_capture: false,
            debug_overlays_allowed: true,
            governor_aggressiveness: 1.0,
            fire_simulation_enabled: true,
            cloth_simulation_enabled: true,
            surface_state_enabled: true,
        }
    }
}

impl RuntimeManifest {
    pub fn low_spec() -> Self {
        Self {
            micro_motion_quality: QualityTier::Low,
            governor_aggressiveness: 1.5,
            cloth_simulation_enabled: false,
            ..Default::default()
        }
    }

    pub fn headless() -> Self {
        Self {
            gore_enabled: false,
            micro_motion_quality: QualityTier::Low,
            debug_overlays_allowed: false,
            cloth_simulation_enabled: false,
            surface_state_enabled: false,
            ..Default::default()
        }
    }

    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "destruction" => self.destruction_enabled,
            "gore" => self.gore_enabled,
            "terrain_deformation" => self.terrain_deformation_enabled,
            "fire" => self.fire_simulation_enabled,
            "cloth" => self.cloth_simulation_enabled,
            "surface_state" => self.surface_state_enabled,
            "replay_capture" => self.replay_capture,
            "debug_overlays" => self.debug_overlays_allowed,
            _ => false,
        }
    }
}
