use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpcRole {
    Guard,
    Hunter,
    Trader,
    Scavenger,
    Courier,
    Bandit,
    IdleResident,
    Mechanic,
    Medic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoleBehavior {
    pub role: NpcRole,
    pub daily_income: f32,
    pub danger_exposure: f32,
    pub social_interaction: f32,
    pub required_equipment: Vec<String>,
}

impl RoleBehavior {
    pub fn for_role(role: NpcRole) -> Self {
        match role {
            NpcRole::Guard => Self {
                role,
                daily_income: 15.0,
                danger_exposure: 0.6,
                social_interaction: 0.3,
                required_equipment: vec!["weapon".into(), "armor".into()],
            },
            NpcRole::Hunter => Self {
                role,
                daily_income: 25.0,
                danger_exposure: 0.8,
                social_interaction: 0.1,
                required_equipment: vec!["weapon".into()],
            },
            NpcRole::Trader => Self {
                role,
                daily_income: 30.0,
                danger_exposure: 0.2,
                social_interaction: 0.9,
                required_equipment: vec![],
            },
            NpcRole::Scavenger => Self {
                role,
                daily_income: 20.0,
                danger_exposure: 0.7,
                social_interaction: 0.2,
                required_equipment: vec!["weapon".into()],
            },
            NpcRole::Courier => Self {
                role,
                daily_income: 18.0,
                danger_exposure: 0.5,
                social_interaction: 0.4,
                required_equipment: vec![],
            },
            NpcRole::Bandit => Self {
                role,
                daily_income: 35.0,
                danger_exposure: 0.9,
                social_interaction: 0.2,
                required_equipment: vec!["weapon".into()],
            },
            NpcRole::IdleResident => Self {
                role,
                daily_income: 5.0,
                danger_exposure: 0.1,
                social_interaction: 0.5,
                required_equipment: vec![],
            },
            NpcRole::Mechanic => Self {
                role,
                daily_income: 22.0,
                danger_exposure: 0.1,
                social_interaction: 0.6,
                required_equipment: vec![],
            },
            NpcRole::Medic => Self {
                role,
                daily_income: 20.0,
                danger_exposure: 0.1,
                social_interaction: 0.7,
                required_equipment: vec![],
            },
        }
    }

    pub fn matches_job(job: &crate::world::components::Job) -> NpcRole {
        match job {
            crate::world::components::Job::Guard => NpcRole::Guard,
            crate::world::components::Job::ArtifactHunter => NpcRole::Hunter,
            crate::world::components::Job::Trader => NpcRole::Trader,
            crate::world::components::Job::Bandit => NpcRole::Bandit,
            crate::world::components::Job::Hunter => NpcRole::Hunter,
            crate::world::components::Job::Scavenger => NpcRole::Scavenger,
            crate::world::components::Job::Courier => NpcRole::Courier,
            crate::world::components::Job::Resident | crate::world::components::Job::Unemployed => {
                NpcRole::IdleResident
            }
        }
    }
}
