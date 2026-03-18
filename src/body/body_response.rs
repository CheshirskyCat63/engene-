use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BodyPhysicalResponseCache {
    pub movement_speed_mult: f32,
    pub combat_power_mult: f32,
    pub accuracy_mult: f32,
    pub pain_level: f32,
    pub blood_loss: f32,
    pub consciousness: f32,
    pub can_sprint: bool,
    pub can_aim_steady: bool,
    pub limp_severity: f32,
    pub response_tier: PhysicalResponseTier,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicalResponseTier {
    Healthy,
    MinorInjury,
    MajorInjury,
    Critical,
    Unconscious,
    Dead,
}

impl Default for PhysicalResponseTier {
    fn default() -> Self {
        Self::Healthy
    }
}

impl BodyPhysicalResponseCache {
    pub fn healthy() -> Self {
        Self {
            movement_speed_mult: 1.0,
            combat_power_mult: 1.0,
            accuracy_mult: 1.0,
            pain_level: 0.0,
            blood_loss: 0.0,
            consciousness: 1.0,
            can_sprint: true,
            can_aim_steady: true,
            limp_severity: 0.0,
            response_tier: PhysicalResponseTier::Healthy,
        }
    }

    pub fn compute_from_health(health: f32, blood: f32, pain: f32) -> Self {
        let consciousness =
            (health * 0.5 + (1.0 - blood) * 0.3 + (1.0 - pain) * 0.2).clamp(0.0, 1.0);

        let tier = if health <= 0.0 {
            PhysicalResponseTier::Dead
        } else if consciousness < 0.1 {
            PhysicalResponseTier::Unconscious
        } else if health < 0.25 {
            PhysicalResponseTier::Critical
        } else if health < 0.5 {
            PhysicalResponseTier::MajorInjury
        } else if health < 0.8 {
            PhysicalResponseTier::MinorInjury
        } else {
            PhysicalResponseTier::Healthy
        };

        let movement_mult = match tier {
            PhysicalResponseTier::Dead => 0.0,
            PhysicalResponseTier::Unconscious => 0.0,
            PhysicalResponseTier::Critical => 0.3,
            PhysicalResponseTier::MajorInjury => 0.6,
            PhysicalResponseTier::MinorInjury => 0.85,
            PhysicalResponseTier::Healthy => 1.0,
        };

        let combat_mult = match tier {
            PhysicalResponseTier::Dead | PhysicalResponseTier::Unconscious => 0.0,
            PhysicalResponseTier::Critical => 0.2,
            PhysicalResponseTier::MajorInjury => 0.5,
            PhysicalResponseTier::MinorInjury => 0.8,
            PhysicalResponseTier::Healthy => 1.0,
        };

        Self {
            movement_speed_mult: movement_mult,
            combat_power_mult: combat_mult,
            accuracy_mult: (1.0 - pain * 0.5).max(0.1),
            pain_level: pain,
            blood_loss: blood,
            consciousness,
            can_sprint: tier == PhysicalResponseTier::Healthy
                || tier == PhysicalResponseTier::MinorInjury,
            can_aim_steady: pain < 0.5 && consciousness > 0.5,
            limp_severity: if health < 0.5 {
                (0.5 - health) * 2.0
            } else {
                0.0
            },
            response_tier: tier,
        }
    }

    pub fn is_combat_capable(&self) -> bool {
        self.combat_power_mult > 0.1 && self.consciousness > 0.2
    }

    pub fn is_mobile(&self) -> bool {
        self.movement_speed_mult > 0.0
    }
}
