//! AI state and simulation level components.

use serde::{Deserialize, Serialize};

/// Current AI behavior state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AiState {
    Idle,
    Executing(Goal),
}

/// High-level goals for AI planning.
#[derive(Clone, Debug, PartialEq, Copy, Serialize, Deserialize)]
pub enum Goal {
    SeekFood,
    SeekWater,
    Rest,
    Work,
    Hunt,
    Flee,
    Trade,
    Explore,
    Socialize,
    Migrate,
    DefendTerritory,
    FollowPack,
    StealOrRob,
    Mate,
    SeekShelter,
    Sleep,
    DoQuest,
    StayAtPost,
    RepairEquipment,
    BuySupplies,
}

impl std::fmt::Display for Goal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SeekFood => write!(f, "Seek Food"),
            Self::SeekWater => write!(f, "Seek Water"),
            Self::Rest => write!(f, "Rest"),
            Self::Work => write!(f, "Work"),
            Self::Hunt => write!(f, "Hunt"),
            Self::Flee => write!(f, "Flee"),
            Self::Trade => write!(f, "Trade"),
            Self::Explore => write!(f, "Explore"),
            Self::Socialize => write!(f, "Socialize"),
            Self::Migrate => write!(f, "Migrate"),
            Self::DefendTerritory => write!(f, "Defend Territory"),
            Self::FollowPack => write!(f, "Follow Pack"),
            Self::StealOrRob => write!(f, "Steal / Rob"),
            Self::Mate => write!(f, "Mate"),
            Self::SeekShelter => write!(f, "Seek Shelter"),
            Self::Sleep => write!(f, "Sleep"),
            Self::DoQuest => write!(f, "Do Quest"),
            Self::StayAtPost => write!(f, "Stay at Post"),
            Self::RepairEquipment => write!(f, "Repair Equipment"),
            Self::BuySupplies => write!(f, "Buy Supplies"),
        }
    }
}

/// Simulation detail level based on distance from player.
#[derive(Clone, Debug, PartialEq, Copy, Serialize, Deserialize)]
pub enum SimulationLevel {
    /// 0-300m: full physics, animation, audio, perception, AI
    L0,
    /// 300m-5km: AI decisions, movement, no physics/animation
    L1,
    /// 5-50km: economy, migration, social (agent-based)
    L2,
    /// 50km+: demographics, climate, ecosystem (strategic)
    L3,
}

/// Component storing current simulation level.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimLevel {
    pub level: SimulationLevel,
}
