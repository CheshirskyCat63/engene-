use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationQuality {
    Full,
    Reduced,
    Minimal,
    Off,
}

#[derive(Clone, Debug)]
pub struct AnimationLadder {
    pub quality: AnimationQuality,
    pub max_blended_layers: u32,
    pub micro_motion_enabled: bool,
    pub foot_ik_enabled: bool,
    pub procedural_overlays: bool,
    pub ragdoll_near_only: bool,
    pub update_rate_hz: u32,
}

impl AnimationLadder {
    pub fn for_quality(quality: AnimationQuality) -> Self {
        match quality {
            AnimationQuality::Full => Self {
                quality, max_blended_layers: 4, micro_motion_enabled: true,
                foot_ik_enabled: true, procedural_overlays: true,
                ragdoll_near_only: false, update_rate_hz: 60,
            },
            AnimationQuality::Reduced => Self {
                quality, max_blended_layers: 2, micro_motion_enabled: true,
                foot_ik_enabled: true, procedural_overlays: false,
                ragdoll_near_only: true, update_rate_hz: 30,
            },
            AnimationQuality::Minimal => Self {
                quality, max_blended_layers: 1, micro_motion_enabled: false,
                foot_ik_enabled: false, procedural_overlays: false,
                ragdoll_near_only: true, update_rate_hz: 15,
            },
            AnimationQuality::Off => Self {
                quality, max_blended_layers: 0, micro_motion_enabled: false,
                foot_ik_enabled: false, procedural_overlays: false,
                ragdoll_near_only: true, update_rate_hz: 0,
            },
        }
    }

    pub fn for_distance(distance: f32) -> Self {
        if distance < 50.0 {
            Self::for_quality(AnimationQuality::Full)
        } else if distance < 200.0 {
            Self::for_quality(AnimationQuality::Reduced)
        } else if distance < 500.0 {
            Self::for_quality(AnimationQuality::Minimal)
        } else {
            Self::for_quality(AnimationQuality::Off)
        }
    }
}

impl Default for AnimationLadder {
    fn default() -> Self { Self::for_quality(AnimationQuality::Full) }
}
