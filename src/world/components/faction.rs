//! Faction membership component.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Faction {
    Loners,
    Duty,
    Freedom,
    Bandits,
    Military,
    Scientists,
    Traders,
}

/// Which faction an entity belongs to.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FactionMembership {
    pub faction: Faction,
    pub standing: f32,
}

impl Default for FactionMembership {
    fn default() -> Self {
        Self {
            faction: Faction::Loners,
            standing: 0.0,
        }
    }
}
