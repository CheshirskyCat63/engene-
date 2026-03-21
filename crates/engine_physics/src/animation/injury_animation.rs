/// Injury state that drives animation modifications
#[derive(Clone, Debug, Default)]
pub struct InjuryAnimationState {
    pub left_leg_damaged: bool,
    pub right_leg_damaged: bool,
    pub left_arm_damaged: bool,
    pub right_arm_damaged: bool,
    pub low_blood: bool,
    pub high_pain: bool,
    pub exhausted: bool,
}

/// Animation modifiers derived from injury state
#[derive(Clone, Debug)]
pub struct InjuryAnimationModifiers {
    /// Speed multiplier (1.0 = normal)
    pub speed_multiplier: f32,
    /// Whether to use limping animation
    pub limp: bool,
    /// Sway/wobble intensity (0.0 = none)
    pub sway_intensity: f32,
    /// Whether weapon handling is impaired
    pub weapon_impaired: bool,
    /// Transition time multiplier (higher = slower reactions)
    pub transition_speed: f32,
    /// Posture degradation amount
    pub posture_droop: f32,
}

impl InjuryAnimationState {
    pub fn compute_modifiers(&self) -> InjuryAnimationModifiers {
        let mut speed = 1.0f32;
        let mut limp = false;
        let mut sway = 0.0f32;
        let mut weapon_impaired = false;
        let mut transition = 1.0f32;
        let mut posture = 0.0f32;

        if self.left_leg_damaged || self.right_leg_damaged {
            speed *= 0.6;
            limp = true;
        }
        if self.left_leg_damaged && self.right_leg_damaged {
            speed *= 0.3;
        }
        if self.left_arm_damaged || self.right_arm_damaged {
            weapon_impaired = true;
        }
        if self.low_blood {
            sway += 0.4;
            speed *= 0.8;
            transition *= 1.5;
        }
        if self.high_pain {
            speed *= 0.7;
            transition *= 1.3;
            posture += 0.2;
        }
        if self.exhausted {
            speed *= 0.6;
            posture += 0.3;
            sway += 0.2;
        }

        InjuryAnimationModifiers {
            speed_multiplier: speed.max(0.1),
            limp,
            sway_intensity: sway.min(1.0),
            weapon_impaired,
            transition_speed: transition,
            posture_droop: posture.min(1.0),
        }
    }
}
