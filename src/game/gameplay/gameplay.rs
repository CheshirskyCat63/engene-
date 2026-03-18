//! Phase 11: Player interaction with economy, NPC, combat.
//!
//! Provides the player-facing interaction layer for trading, combat, and NPC communication.

use crate::core::ecs::Ecs;
use crate::game::economy::trading::TradeResult;
use crate::world::components::*;

/// Result of a player interaction attempt.
#[derive(Debug, Clone)]
pub enum InteractionResult {
    Success(String),
    Failed(String),
    NotAllowed(String),
    NotFound,
}

/// Player interaction system - handles all player-initiated actions.
///
/// Uses helper methods instead of direct storage access.
pub struct PlayerInteractionSystem {
    /// Currently targeted entity (if any)
    pub target_entity: Option<u64>,
    /// Interaction range in world units
    pub interaction_range: f32,
    /// Combat range
    pub combat_range: f32,
    /// Trade range
    pub trade_range: f32,
}

impl Default for PlayerInteractionSystem {
    fn default() -> Self {
        Self {
            target_entity: None,
            interaction_range: 50.0,
            combat_range: 30.0,
            trade_range: 20.0,
        }
    }
}

impl PlayerInteractionSystem {
    /// Create a new player interaction system.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the current target entity.
    pub fn set_target(&mut self, entity: Option<u64>) {
        self.target_entity = entity;
    }

    /// Check if an entity is within interaction range.
    pub fn is_in_range(&self, player_pos: (f32, f32), entity_pos: (f32, f32), range: f32) -> bool {
        let dx = player_pos.0 - entity_pos.0;
        let dy = player_pos.1 - entity_pos.1;
        let dist = (dx * dx + dy * dy).sqrt();
        dist <= range
    }

    /// Attempt to interact with the current target.
    pub fn try_interact(&self, ecs: &Ecs, player_pos: (f32, f32)) -> InteractionResult {
        let target = match self.target_entity {
            Some(t) => t,
            None => return InteractionResult::NotFound,
        };

        // Check if target exists
        if !ecs.alive.contains(&target) {
            return InteractionResult::NotFound;
        }

        // Get target position
        let target_pos = match ecs.get_transform(target) {
            Some(t) => (t.x, t.y),
            None => return InteractionResult::NotFound,
        };

        // Check range
        if !self.is_in_range(player_pos, target_pos, self.interaction_range) {
            return InteractionResult::Failed("Target is too far away".into());
        }

        // Determine interaction type based on entity kind
        match ecs.get_kind(target) {
            Some(EntityKind::Npc) => self.interact_with_npc(ecs, target),
            Some(EntityKind::Monster(species)) => self.interact_with_monster(ecs, target, species),
            None => InteractionResult::Failed("Cannot interact with this entity".into()),
        }
    }

    /// Interact with an NPC (trade, talk, etc.)
    fn interact_with_npc(&self, ecs: &Ecs, target: u64) -> InteractionResult {
        let name = ecs.get_name(target).map(|n| n.0.as_str()).unwrap_or("NPC");

        // Check if NPC has economy data
        if ecs.npc_economies.contains_key(&target) {
            InteractionResult::Success(format!("You can trade with {}.", name))
        } else {
            InteractionResult::Success(format!("{} greets you.", name))
        }
    }

    /// Interact with a monster (combat or avoidance)
    fn interact_with_monster(
        &self,
        ecs: &Ecs,
        target: u64,
        species: &MonsterSpecies,
    ) -> InteractionResult {
        let species_name = match species {
            MonsterSpecies::Wolf => "Wolf",
            MonsterSpecies::Boar => "Boar",
            MonsterSpecies::Bloodsucker => "Bloodsucker",
        };

        // Check monster health
        if let Some(needs) = ecs.get_needs(target) {
            if needs.health <= 0.0 {
                return InteractionResult::Success(format!("The {} is dead.", species_name));
            }
        }

        InteractionResult::Failed(format!(
            "The {} is hostile! Combat initiated.",
            species_name
        ))
    }

    /// Attempt to trade with current target.
    pub fn try_trade(&self, ecs: &mut Ecs, player_money: f32, amount: f32) -> TradeResult {
        let target = match self.target_entity {
            Some(t) => t,
            None => return TradeResult::Failed("No target selected".into()),
        };

        // Check if target is an NPC with economy
        if !matches!(ecs.get_kind(target), Some(EntityKind::Npc)) {
            return TradeResult::Failed("Cannot trade with this entity".into());
        }

        // Check if NPC has economy
        let _npc_economy = match ecs.get_npc_economy(target) {
            Some(e) => e.clone(),
            None => return TradeResult::Failed("This NPC cannot trade".into()),
        };

        // Simple trade: player pays, NPC receives
        if player_money < amount {
            return TradeResult::Failed("Not enough money".into());
        }

        // Update NPC economy
        if let Some(economy) = ecs.get_npc_economy_mut(target) {
            economy.money += amount;
        }

        TradeResult::Success {
            player_paid: amount,
            npc_received: amount,
        }
    }

    /// Initiate combat with current target.
    pub fn initiate_combat(&self, ecs: &Ecs) -> InteractionResult {
        let target = match self.target_entity {
            Some(t) => t,
            None => return InteractionResult::NotFound,
        };

        // Check if target is hostile
        match ecs.get_kind(target) {
            Some(EntityKind::Monster(_)) => InteractionResult::Success("Combat initiated!".into()),
            Some(EntityKind::Npc) => {
                InteractionResult::NotAllowed("Cannot attack friendly NPCs".into())
            }
            None => InteractionResult::NotFound,
        }
    }

    /// Get target info for UI display.
    pub fn get_target_info(&self, ecs: &Ecs) -> Option<TargetInfo> {
        let target = self.target_entity?;

        let name = ecs
            .get_name(target)
            .map(|n| n.0.clone())
            .unwrap_or_else(|| "Unknown".into());

        let kind = ecs.get_kind(target).cloned();

        let health = ecs.get_needs(target).map(|n| n.health).unwrap_or(1.0);

        let position = ecs
            .get_transform(target)
            .map(|t| (t.x, t.y))
            .unwrap_or((0.0, 0.0));

        Some(TargetInfo {
            entity: target,
            name,
            kind,
            health,
            position,
        })
    }
}

/// Information about the current target for UI display.
#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub entity: u64,
    pub name: String,
    pub kind: Option<EntityKind>,
    pub health: f32,
    pub position: (f32, f32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_range() {
        let system = PlayerInteractionSystem::new();

        assert!(system.is_in_range((0.0, 0.0), (10.0, 10.0), 20.0));
        assert!(!system.is_in_range((0.0, 0.0), (100.0, 100.0), 20.0));
    }

    #[test]
    fn test_no_target() {
        let system = PlayerInteractionSystem::new();
        let ecs = Ecs::new();

        match system.try_interact(&ecs, (0.0, 0.0)) {
            InteractionResult::NotFound => {}
            _ => panic!("Expected NotFound"),
        }
    }

    #[test]
    fn test_npc_interaction() {
        let mut system = PlayerInteractionSystem::new();
        let mut ecs = Ecs::new();

        let npc = ecs.spawn();
        ecs.set_kind(npc, EntityKind::Npc);
        ecs.set_name(npc, Name("Trader".into()));
        ecs.set_transform(
            npc,
            Transform {
                x: 5.0,
                y: 5.0,
                cell_x: 0,
                cell_y: 0,
            },
        );
        ecs.set_npc_economy(
            npc,
            NpcEconomy {
                money: 100.0,
                monthly_required: 50.0,
                job: Job::Trader,
                desperation: 0.0,
            },
        );

        system.set_target(Some(npc));

        match system.try_interact(&ecs, (0.0, 0.0)) {
            InteractionResult::Success(msg) => assert!(msg.contains("trade")),
            _ => panic!("Expected Success"),
        }
    }
}
