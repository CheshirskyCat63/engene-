//! Entity-related events.

use super::entity_kind::MonsterSpecies;
use crate::core::ecs::Entity;

/// Emitted when an entity dies.
#[derive(Clone, Debug)]
pub struct EntityDied {
    pub entity: Entity,
    pub killer: Option<Entity>,
}

/// Emitted when an NPC goes bankrupt.
#[derive(Clone, Debug)]
pub struct NpcWentBankrupt(pub Entity);

/// Emitted when combat occurs.
#[derive(Clone, Debug)]
pub struct CombatOccurred {
    pub attacker: Entity,
    pub defender: Entity,
    pub attacker_won: bool,
}

/// Emitted when monsters migrate between cells.
#[derive(Clone, Debug)]
pub struct MigrationOccurred {
    pub species: MonsterSpecies,
    pub from: (u32, u32),
    pub to: (u32, u32),
}
