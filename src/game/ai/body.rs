use crate::world::components::{LifeStage, PersonalNeeds};

#[derive(Clone, Debug)]
pub struct BodyState {
    pub move_speed_mult: f32,
    pub perception_radius_mult: f32,
    pub combat_power_mult: f32,
    pub work_efficiency_mult: f32,
}

impl BodyState {
    pub fn compute(pn: &PersonalNeeds) -> Self {
        Self::compute_with_stage(pn, LifeStage::Adult)
    }

    pub fn compute_with_stage(pn: &PersonalNeeds, stage: LifeStage) -> Self {
        let fatigue = (1.0 - pn.energy).max(0.0);
        let starving = (pn.hunger - 0.5).max(0.0) * 2.0;
        let sleepy = (pn.sleep - 0.5).max(0.0) * 2.0;
        let hurt = (1.0 - pn.health).max(0.0);

        Self {
            move_speed_mult: ((1.0 - fatigue * 0.3 - starving * 0.2 - hurt * 0.4 - sleepy * 0.2)
                * stage.speed_mult())
            .max(0.2),
            perception_radius_mult: (1.0 - sleepy * 0.4 - fatigue * 0.15).max(0.3),
            combat_power_mult: ((1.0 - fatigue * 0.25 - starving * 0.15 - hurt * 0.3)
                * stage.combat_mult())
            .max(0.15),
            work_efficiency_mult: (1.0 - fatigue * 0.3 - sleepy * 0.3 - pn.discomfort * 0.2)
                .max(0.1),
        }
    }
}

pub fn is_night(day_progress: f32) -> bool {
    day_progress > 0.75 || day_progress < 0.2
}

pub fn time_of_day_mult(day_progress: f32, nocturnal: bool) -> f32 {
    let night = is_night(day_progress);
    if nocturnal {
        if night {
            1.3
        } else {
            0.7
        }
    } else {
        if night {
            0.6
        } else {
            1.0
        }
    }
}
