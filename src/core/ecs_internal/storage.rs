//! ECS Storage Layer - Component storage abstractions.
//! Phase 2a.2: TRANSITIONAL - depends on world::components and AI types.
//! This module provides component storage containers.
//!
//! BLOCKED: Cannot move to engine_ecs until world::components moves to engine_world.

use engine_ecs::sparse_set::SparseSet;
use crate::world::components::*;
use crate::world::extension_components::{Attributes, Blackboard, EntityTags, StatusEffects};
use crate::core::ai_emotions::Emotions;
use crate::core::ai_memory::Memory;
use crate::core::ai_plan::Plan;
use crate::world::spatial_index::SpatialIndex;

pub use engine_ecs::Entity;

/// Storage subsystem - all component storages.
/// TRANSITIONAL: Depends on world::components types.
/// Removal condition: when world::components moves to engine_world.
pub struct StorageSubsystem {
    // Core components - depend on world::components
    pub transforms: SparseSet<Transform>,
    pub kinds: SparseSet<EntityKind>,
    pub names: SparseSet<Name>,
    pub npc_traits: SparseSet<NpcTraits>,
    pub monster_traits: SparseSet<MonsterTraits>,
    pub personal_needs: SparseSet<PersonalNeeds>,
    pub social_needs: SparseSet<SocialNeeds>,
    pub ecosystem_needs: SparseSet<EcosystemNeeds>,
    pub npc_economies: SparseSet<NpcEconomy>,
    pub sim_levels: SparseSet<SimLevel>,
    pub ai_states: SparseSet<AiState>,
    pub inventories: SparseSet<Inventory>,
    
    // AI components - depend on root AI modules
    pub memories: SparseSet<Memory>,
    pub emotions: SparseSet<Emotions>,
    pub plans: SparseSet<Plan>,
    
    // Other gameplay components
    pub life_info: SparseSet<LifeInfo>,
    pub flammables: SparseSet<Flammable>,
    pub cloth_components: SparseSet<ClothComponent>,
    pub tags: SparseSet<EntityTags>,
    pub attributes: SparseSet<Attributes>,
    pub status_effects: SparseSet<StatusEffects>,
    pub blackboard: SparseSet<Blackboard>,
    pub equipment: SparseSet<EquipmentSlots>,
    pub faction_memberships: SparseSet<FactionMembership>,
    
    // Spatial - depends on world module
    pub spatial: SpatialIndex,
}

impl StorageSubsystem {
    pub fn new() -> Self {
        Self {
            transforms: SparseSet::new(),
            kinds: SparseSet::new(),
            names: SparseSet::new(),
            npc_traits: SparseSet::new(),
            monster_traits: SparseSet::new(),
            personal_needs: SparseSet::new(),
            social_needs: SparseSet::new(),
            ecosystem_needs: SparseSet::new(),
            npc_economies: SparseSet::new(),
            sim_levels: SparseSet::new(),
            ai_states: SparseSet::new(),
            inventories: SparseSet::new(),
            memories: SparseSet::new(),
            emotions: SparseSet::new(),
            plans: SparseSet::new(),
            life_info: SparseSet::new(),
            flammables: SparseSet::new(),
            cloth_components: SparseSet::new(),
            tags: SparseSet::new(),
            attributes: SparseSet::new(),
            status_effects: SparseSet::new(),
            blackboard: SparseSet::new(),
            equipment: SparseSet::new(),
            faction_memberships: SparseSet::new(),
            spatial: SpatialIndex::new(),
        }
    }

    /// Remove all components for an entity.
    pub fn remove_all(&mut self, entity: u64) {
        self.transforms.remove(&entity);
        self.kinds.remove(&entity);
        self.names.remove(&entity);
        self.npc_traits.remove(&entity);
        self.monster_traits.remove(&entity);
        self.personal_needs.remove(&entity);
        self.social_needs.remove(&entity);
        self.ecosystem_needs.remove(&entity);
        self.npc_economies.remove(&entity);
        self.sim_levels.remove(&entity);
        self.ai_states.remove(&entity);
        self.inventories.remove(&entity);
        self.memories.remove(&entity);
        self.emotions.remove(&entity);
        self.plans.remove(&entity);
        self.life_info.remove(&entity);
        self.flammables.remove(&entity);
        self.cloth_components.remove(&entity);
        self.tags.remove(&entity);
        self.attributes.remove(&entity);
        self.status_effects.remove(&entity);
        self.blackboard.remove(&entity);
        self.equipment.remove(&entity);
        self.faction_memberships.remove(&entity);
    }

    /// Get list of all component storage names (for debugging/inspection).
    pub fn storage_names(&self) -> Vec<&'static str> {
        vec![
            "transforms", "kinds", "names", "npc_traits", "monster_traits",
            "personal_needs", "social_needs", "ecosystem_needs", "npc_economies",
            "sim_levels", "ai_states", "inventories", "memories", "emotions",
            "plans", "life_info", "flammables", "cloth_components", "tags",
            "attributes", "status_effects", "blackboard", "equipment", "faction_memberships",
        ]
    }
}

impl Default for StorageSubsystem {
    fn default() -> Self {
        Self::new()
    }
}
