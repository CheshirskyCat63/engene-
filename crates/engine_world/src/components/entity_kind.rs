//! Entity kind and species classification.

use serde::{Deserialize, Serialize};

/// Classification of entity type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EntityKind {
    Npc,
    Monster(MonsterSpecies),
}

/// Monster species variants.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy, Serialize, Deserialize)]
pub enum MonsterSpecies {
    Wolf,
    Boar,
    Bloodsucker,
}

impl std::fmt::Display for MonsterSpecies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wolf => write!(f, "Wolf"),
            Self::Boar => write!(f, "Boar"),
            Self::Bloodsucker => write!(f, "Bloodsucker"),
        }
    }
}

// Combat power levels: Wolf=3, Human=5, Boar=7, Bloodsucker=10
pub fn base_power(kind: &EntityKind) -> f32 {
    match kind {
        EntityKind::Monster(MonsterSpecies::Wolf) => 3.0,
        EntityKind::Npc => 5.0,
        EntityKind::Monster(MonsterSpecies::Boar) => 7.0,
        EntityKind::Monster(MonsterSpecies::Bloodsucker) => 10.0,
    }
}

pub fn is_prey_for(hunter: &EntityKind, target: &EntityKind) -> bool {
    base_power(hunter) > base_power(target)
}

pub fn food_value(kind: &EntityKind) -> f32 {
    match kind {
        EntityKind::Monster(MonsterSpecies::Wolf) => 0.4,
        EntityKind::Npc => 0.5,
        EntityKind::Monster(MonsterSpecies::Boar) => 0.6,
        EntityKind::Monster(MonsterSpecies::Bloodsucker) => 0.8,
    }
}
