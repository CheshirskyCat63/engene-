use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRules {
    pub systems: Vec<SystemEntry>,
    pub simulation_radii: SimRadii,
    pub replicated_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEntry {
    pub name: String,
    pub frequency: TickFreqConfig,
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TickFreqConfig {
    EveryFrame,
    EveryN(u32),
    OnEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimRadii {
    pub l0: f32,
    pub l1: f32,
    pub l2: f32,
    pub l0_tick: u32,
    pub l1_tick: u32,
    pub l2_tick: u32,
}

impl Default for GameRules {
    fn default() -> Self {
        Self {
            systems: vec![
                SystemEntry { name: "Simulation".into(), frequency: TickFreqConfig::EveryFrame, order: 0 },
                SystemEntry { name: "WorldTick".into(), frequency: TickFreqConfig::EveryFrame, order: 1 },
                SystemEntry { name: "AI".into(), frequency: TickFreqConfig::EveryFrame, order: 2 },
                SystemEntry { name: "Physics".into(), frequency: TickFreqConfig::EveryFrame, order: 3 },
                SystemEntry { name: "Economy".into(), frequency: TickFreqConfig::EveryFrame, order: 4 },
            ],
            simulation_radii: SimRadii {
                l0: 300.0, l1: 5000.0, l2: 50000.0,
                l0_tick: 1, l1_tick: 12, l2_tick: 60,
            },
            replicated_events: vec![
                "ShotFired".into(),
                "Impact".into(),
                "EntityDied".into(),
                "NewDay".into(),
                "NewMonth".into(),
            ],
        }
    }
}
