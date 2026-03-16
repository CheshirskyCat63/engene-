//! Need components for NPC and monster behavior.

use serde::{Deserialize, Serialize};
use super::entity_kind::MonsterSpecies;

/// Personal needs shared by NPCs and monsters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonalNeeds {
    pub hunger: f32,
    pub thirst: f32,
    pub sleep: f32,
    pub health: f32,
    pub energy: f32,
    pub fear: f32,
    pub curiosity: f32,
    pub ambitions: f32,
    pub discomfort: f32,
}

impl PersonalNeeds {
    pub fn default_npc() -> Self {
        Self {
            hunger: 0.2,
            thirst: 0.2,
            sleep: 0.1,
            health: 1.0,
            energy: 0.8,
            fear: 0.0,
            curiosity: 0.3,
            ambitions: 0.4,
            discomfort: 0.1,
        }
    }

    pub fn default_monster() -> Self {
        Self {
            hunger: 0.3,
            thirst: 0.2,
            sleep: 0.1,
            health: 1.0,
            energy: 0.9,
            fear: 0.0,
            curiosity: 0.2,
            ambitions: 0.0,
            discomfort: 0.1,
        }
    }
}

/// Social needs (NPC only).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SocialNeeds {
    pub family: f32,
    pub money: f32,
    pub reputation: f32,
    pub friendship: f32,
    pub faction_loyalty: f32,
    pub fear_of_punishment: f32,
    pub competition: f32,
    pub loneliness: f32,
    pub entertainment: f32,
}

impl Default for SocialNeeds {
    fn default() -> Self {
        Self {
            family: 0.3,
            money: 0.5,
            reputation: 0.3,
            friendship: 0.3,
            faction_loyalty: 0.2,
            fear_of_punishment: 0.1,
            competition: 0.2,
            loneliness: 0.2,
            entertainment: 0.3,
        }
    }
}

/// Ecosystem needs (monster only).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EcosystemNeeds {
    pub hunting: f32,
    pub predator_avoidance: f32,
    pub food_chain_position: f32,
    pub territory_control: f32,
    pub migration_urge: f32,
    pub resource_competition: f32,
    pub pack_following: f32,
    pub shelter_seeking: f32,
    pub world_event_reaction: f32,
    pub prey_selection: f32,
}

impl EcosystemNeeds {
    pub fn for_species(species: MonsterSpecies) -> Self {
        match species {
            MonsterSpecies::Wolf => Self {
                hunting: 0.6,
                predator_avoidance: 0.2,
                food_chain_position: 0.7,
                territory_control: 0.5,
                migration_urge: 0.1,
                resource_competition: 0.3,
                pack_following: 0.8,
                shelter_seeking: 0.2,
                world_event_reaction: 0.1,
                prey_selection: 0.5,
            },
            MonsterSpecies::Boar => Self {
                hunting: 0.0,
                predator_avoidance: 0.7,
                food_chain_position: 0.3,
                territory_control: 0.3,
                migration_urge: 0.4,
                resource_competition: 0.5,
                pack_following: 0.4,
                shelter_seeking: 0.5,
                world_event_reaction: 0.3,
                prey_selection: 0.0,
            },
            MonsterSpecies::Bloodsucker => Self {
                hunting: 0.8,
                predator_avoidance: 0.1,
                food_chain_position: 0.9,
                territory_control: 0.7,
                migration_urge: 0.2,
                resource_competition: 0.2,
                pack_following: 0.1,
                shelter_seeking: 0.6,
                world_event_reaction: 0.2,
                prey_selection: 0.7,
            },
        }
    }
}
