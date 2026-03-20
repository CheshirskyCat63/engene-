//! Economy and inventory components.

use serde::{Deserialize, Serialize};

/// NPC economic state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpcEconomy {
    pub money: f32,
    pub monthly_required: f32,
    pub job: Job,
    pub desperation: f32,
}

/// Job types for NPCs.
#[derive(Clone, Debug, PartialEq, Copy, Serialize, Deserialize)]
pub enum Job {
    ArtifactHunter,
    Guard,
    Trader,
    Bandit,
    Unemployed,
    Hunter,
    Scavenger,
    Courier,
    Resident,
}

impl std::fmt::Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArtifactHunter => write!(f, "Artifact Hunter"),
            Self::Guard => write!(f, "Guard"),
            Self::Trader => write!(f, "Trader"),
            Self::Bandit => write!(f, "Bandit"),
            Self::Unemployed => write!(f, "Unemployed"),
            Self::Hunter => write!(f, "Hunter"),
            Self::Scavenger => write!(f, "Scavenger"),
            Self::Courier => write!(f, "Courier"),
            Self::Resident => write!(f, "Resident"),
        }
    }
}

/// Entity inventory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<Item>,
}

/// Inventory item.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    pub value: f32,
}
