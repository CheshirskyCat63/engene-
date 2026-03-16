//! Faction membership component.

use serde::{Deserialize, Serialize};

/// Which faction an entity belongs to.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FactionMembership {
    pub faction: crate::gameplay::factions::Faction,
    pub standing: f32,
}

impl Default for FactionMembership {
    fn default() -> Self {
        Self {
            faction: crate::gameplay::factions::Faction::Loners,
            standing: 0.0,
        }
    }
}
