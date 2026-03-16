use crate::core::ecs::Entity;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ObjectCondition {
    Intact,
    Wobbling { amplitude: f32, frequency: f32 },
    Sagging { amount: f32 },
    Cracked { severity: f32 },
    Leaning { angle_deg: f32 },
    Limping,
}

#[derive(Clone, Debug)]
pub struct SoftDamageState {
    pub entity: Entity,
    pub condition: ObjectCondition,
    pub accumulated_fatigue: f32,
    pub wobble_amplitude: f32,
    pub sag_amount: f32,
    pub lean_angle: f32,
}

impl SoftDamageState {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            condition: ObjectCondition::Intact,
            accumulated_fatigue: 0.0,
            wobble_amplitude: 0.0,
            sag_amount: 0.0,
            lean_angle: 0.0,
        }
    }

    pub fn apply_fatigue(&mut self, amount: f32) {
        self.accumulated_fatigue += amount;
        self.update_condition();
    }

    fn update_condition(&mut self) {
        if self.accumulated_fatigue < 0.1 {
            self.condition = ObjectCondition::Intact;
        } else if self.accumulated_fatigue < 0.3 {
            self.wobble_amplitude = self.accumulated_fatigue * 0.1;
            self.condition = ObjectCondition::Wobbling {
                amplitude: self.wobble_amplitude,
                frequency: 2.0,
            };
        } else if self.accumulated_fatigue < 0.6 {
            self.condition = ObjectCondition::Cracked {
                severity: self.accumulated_fatigue,
            };
        } else if self.accumulated_fatigue < 0.9 {
            self.sag_amount = (self.accumulated_fatigue - 0.6) * 0.5;
            self.condition = ObjectCondition::Sagging {
                amount: self.sag_amount,
            };
        } else {
            self.lean_angle = (self.accumulated_fatigue - 0.9) * 30.0;
            self.condition = ObjectCondition::Leaning {
                angle_deg: self.lean_angle,
            };
        }
    }
}
