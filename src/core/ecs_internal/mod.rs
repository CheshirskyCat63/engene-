//! ECS Internal Modules - Phase 2a.2
//! Minimal composition layer.
//! 
//! Pure ECS mechanics live in engine_ecs::EcsMechanics.
//! This module only wires together transitional subsystems.

pub mod identity;   // Re-exports from engine_ecs
pub mod storage;    // TRANSITIONAL - depends on world::components
pub mod lifecycle;  // Uses engine_ecs::EcsMechanics + game-specific journals

pub use identity::IdentitySubsystem;
pub use storage::StorageSubsystem;
pub use lifecycle::LifecycleSubsystem;

// Re-export types from engine_ecs
pub use engine_ecs::Entity;
pub use engine_ecs::GenEntity;
pub use engine_ecs::persistent_id::{DuplicateIdError, PersistentEntityId};

// Alias for type used in composition
type EcsMechanics = engine_ecs::EcsMechanics;

/// Transitional Ecs struct - composes pure ECS mechanics with game-specific storage.
/// 
/// PHASE 2a.2 STATUS: 
/// - Pure ECS mechanics (identity, spawn, despawn, alive tracking) -> engine_ecs::EcsMechanics
/// - Component storage (world types) -> TRANSITIONAL in root
/// - Lifecycle journals -> TRANSITIONAL in root
pub struct Ecs {
    /// Pure ECS mechanics - identity, spawn, despawn, alive tracking
    pub ecs: EcsMechanics,
    
    /// Storage subsystem - all component storages (TRANSITIONAL - depends on world::components)
    pub storage: StorageSubsystem,
    
    /// Lifecycle subsystem - spatial journals, territory (TRANSITIONAL)
    pub lifecycle: LifecycleSubsystem,
}

impl Ecs {
    pub fn new() -> Self {
        Self {
            ecs: EcsMechanics::new(),
            storage: StorageSubsystem::new(),
            lifecycle: LifecycleSubsystem::new(),
        }
    }

    /// Spawn transient entity.
    pub fn spawn(&mut self) -> Entity {
        let entity = self.lifecycle.spawn();
        debug_assert!(self.validate_invariants().is_ok());
        entity
    }

    /// Spawn new gameplay entity with PersistentEntityId.
    pub fn spawn_new(&mut self) -> (Entity, PersistentEntityId) {
        self.lifecycle.spawn_new()
    }

    /// Restore entity from persistence.
    pub fn spawn_restored(&mut self, pid: PersistentEntityId) -> Result<Entity, DuplicateIdError> {
        self.ecs.spawn_restored(pid)
    }

    /// Unload entity (chunk boundary).
    pub fn unload_entity(&mut self, entity: Entity) {
        self.ecs.unload_entity(entity);
        self.storage.remove_all(entity);
    }

    /// Permanently remove entity.
    pub fn despawn(&mut self, entity: Entity) {
        self.lifecycle.despawn(entity);
        self.storage.remove_all(entity);
        debug_assert!(self.validate_invariants().is_ok());
    }

    /// Check if entity is alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.lifecycle.is_alive(entity)
    }

    /// Validate ECS invariants.
    pub fn validate_invariants(&self) -> Result<(), String> {
        self.lifecycle.validate_invariants()
    }

    /// Get tick.
    pub fn tick(&self) -> u64 {
        self.ecs.tick
    }

    /// Advance tick.
    pub fn advance_tick(&mut self) {
        self.ecs.advance_tick();
    }

    // ========================================================================
    // DIRECT STORAGE ACCESS - Legacy API for backward compatibility
    // These delegate to internal subsystems.
    // ========================================================================

    #[inline]
    pub fn transforms(&self) -> &engine_ecs::SparseSet<crate::world::components::Transform> {
        &self.storage.transforms
    }

    #[inline]
    pub fn transforms_mut(&mut self) -> &mut engine_ecs::SparseSet<crate::world::components::Transform> {
        &mut self.storage.transforms
    }

    #[inline]
    pub fn kinds(&self) -> &engine_ecs::SparseSet<crate::world::components::EntityKind> {
        &self.storage.kinds
    }

    #[inline]
    pub fn kinds_mut(&mut self) -> &mut engine_ecs::SparseSet<crate::world::components::EntityKind> {
        &mut self.storage.kinds
    }

    #[inline]
    pub fn names(&self) -> &engine_ecs::SparseSet<crate::world::components::Name> {
        &self.storage.names
    }

    #[inline]
    pub fn names_mut(&mut self) -> &mut engine_ecs::SparseSet<crate::world::components::Name> {
        &mut self.storage.names
    }

    /// Get alive entities.
    pub fn alive(&self) -> &[Entity] {
        self.lifecycle.alive()
    }

    /// Get spatial index.
    pub fn spatial(&self) -> &crate::world::spatial_index::SpatialIndex {
        &self.storage.spatial
    }

    /// Get territory map.
    pub fn territory(&self) -> &std::collections::HashMap<(u32, u32), crate::world::components::MonsterSpecies> {
        &self.lifecycle.territory
    }

    /// Get territory map mutable.
    pub fn territory_mut(&mut self) -> &mut std::collections::HashMap<(u32, u32), crate::world::components::MonsterSpecies> {
        &mut self.lifecycle.territory
    }

    /// Get entity kind.
    pub fn kind(&self, entity: Entity) -> Option<&crate::world::components::EntityKind> {
        self.storage.kinds.get(&entity)
    }

    /// Set entity transform.
    pub fn set_transform(&mut self, entity: Entity, value: crate::world::components::Transform) {
        if self.storage.transforms.get(&entity).is_some() {
            self.lifecycle.spatial_moved_journal.insert(entity);
        } else {
            self.lifecycle.spatial_inserted_journal.insert(entity);
            self.lifecycle.spatial_removed_journal.remove(&entity);
        }
        self.storage.transforms.insert(entity, value);
    }

    /// Get transform.
    pub fn transform(&self, entity: Entity) -> Option<&crate::world::components::Transform> {
        self.storage.transforms.get(&entity)
    }

    /// Get transform mutable.
    pub fn transform_mut(&mut self, entity: Entity) -> Option<&mut crate::world::components::Transform> {
        if self.lifecycle.is_alive(entity) {
            self.lifecycle.spatial_moved_journal.insert(entity);
        }
        self.storage.transforms.get_mut(&entity)
    }

    /// Set kind.
    #[inline]
    pub fn set_kind(&mut self, entity: Entity, value: crate::world::components::EntityKind) {
        self.storage.kinds.insert(entity, value);
    }

    /// Set name.
    #[inline]
    pub fn set_name(&mut self, entity: Entity, value: crate::world::components::Name) {
        self.storage.names.insert(entity, value);
    }

    /// Get name.
    #[inline]
    pub fn name(&self, entity: Entity) -> Option<&crate::world::components::Name> {
        self.storage.names.get(&entity)
    }

    /// Get memory.
    #[inline]
    pub fn memory(&self, entity: Entity) -> Option<&crate::core::ai_memory::Memory> {
        self.storage.memories.get(&entity)
    }

    /// Set memory.
    #[inline]
    pub fn set_memory(&mut self, entity: Entity, value: crate::core::ai_memory::Memory) {
        self.storage.memories.insert(entity, value);
    }

    /// Get emotions.
    #[inline]
    pub fn emotions(&self, entity: Entity) -> Option<&crate::core::ai_emotions::Emotions> {
        self.storage.emotions.get(&entity)
    }

    /// Set emotions.
    #[inline]
    pub fn set_emotions(&mut self, entity: Entity, value: crate::core::ai_emotions::Emotions) {
        self.storage.emotions.insert(entity, value);
    }

    /// Get plan.
    #[inline]
    pub fn plan(&self, entity: Entity) -> Option<&crate::core::ai_plan::Plan> {
        self.storage.plans.get(&entity)
    }

    /// Set plan.
    #[inline]
    pub fn set_plan(&mut self, entity: Entity, value: crate::core::ai_plan::Plan) {
        self.storage.plans.insert(entity, value);
    }

    /// Take plan (remove and return).
    #[inline]
    pub fn take_plan(&mut self, entity: Entity) -> Option<crate::core::ai_plan::Plan> {
        self.storage.plans.remove(&entity)
    }

    /// Get inventory.
    #[inline]
    pub fn inventory(&self, entity: Entity) -> Option<&crate::world::components::Inventory> {
        self.storage.inventories.get(&entity)
    }

    /// Set inventory.
    #[inline]
    pub fn set_inventory(&mut self, entity: Entity, value: crate::world::components::Inventory) {
        self.storage.inventories.insert(entity, value);
    }

    /// Get life info.
    #[inline]
    pub fn life_info(&self, entity: Entity) -> Option<&crate::world::components::LifeInfo> {
        self.storage.life_info.get(&entity)
    }

    /// Set life info.
    #[inline]
    pub fn set_life_info(&mut self, entity: Entity, value: crate::world::components::LifeInfo) {
        self.storage.life_info.insert(entity, value);
    }

    /// Get personal needs.
    #[inline]
    pub fn needs(&self, entity: Entity) -> Option<&crate::world::components::PersonalNeeds> {
        self.storage.personal_needs.get(&entity)
    }

    /// Set personal needs.
    #[inline]
    pub fn set_needs(&mut self, entity: Entity, value: crate::world::components::PersonalNeeds) {
        self.storage.personal_needs.insert(entity, value);
    }

    /// Get AI state.
    #[inline]
    pub fn ai_state(&self, entity: Entity) -> Option<&crate::world::components::AiState> {
        self.storage.ai_states.get(&entity)
    }

    /// Set AI state.
    #[inline]
    pub fn set_ai_state(&mut self, entity: Entity, value: crate::world::components::AiState) {
        self.storage.ai_states.insert(entity, value);
    }

    /// Get monster traits.
    #[inline]
    pub fn monster_traits(&self, entity: Entity) -> Option<&crate::world::components::MonsterTraits> {
        self.storage.monster_traits.get(&entity)
    }

    /// Get NPC traits.
    #[inline]
    pub fn npc_traits(&self, entity: Entity) -> Option<&crate::world::components::NpcTraits> {
        self.storage.npc_traits.get(&entity)
    }

    /// Get ecosystem needs.
    #[inline]
    pub fn ecosystem_needs(&self, entity: Entity) -> Option<&crate::world::components::EcosystemNeeds> {
        self.storage.ecosystem_needs.get(&entity)
    }

    /// Get social needs.
    #[inline]
    pub fn social_needs(&self, entity: Entity) -> Option<&crate::world::components::SocialNeeds> {
        self.storage.social_needs.get(&entity)
    }

    /// Set social needs.
    #[inline]
    pub fn set_social_needs(&mut self, entity: Entity, value: crate::world::components::SocialNeeds) {
        self.storage.social_needs.insert(entity, value);
    }

    /// Get NPC economy.
    #[inline]
    pub fn npc_economy(&self, entity: Entity) -> Option<&crate::world::components::NpcEconomy> {
        self.storage.npc_economies.get(&entity)
    }

    /// Set NPC economy.
    #[inline]
    pub fn set_npc_economy(&mut self, entity: Entity, value: crate::world::components::NpcEconomy) {
        self.storage.npc_economies.insert(entity, value);
    }

    /// Get simulation level.
    #[inline]
    pub fn sim_level(&self, entity: Entity) -> Option<&crate::world::components::SimLevel> {
        self.storage.sim_levels.get(&entity)
    }

    /// Set simulation level.
    #[inline]
    pub fn set_sim_level(&mut self, entity: Entity, value: crate::world::components::SimLevel) {
        self.storage.sim_levels.insert(entity, value);
    }

    /// Get equipment.
    #[inline]
    pub fn equipment(&self, entity: Entity) -> Option<&crate::world::components::EquipmentSlots> {
        self.storage.equipment.get(&entity)
    }

    /// Set equipment.
    #[inline]
    pub fn set_equipment(&mut self, entity: Entity, value: crate::world::components::EquipmentSlots) {
        self.storage.equipment.insert(entity, value);
    }

    /// Get faction memberships.
    #[inline]
    pub fn faction_memberships(&self, entity: Entity) -> Option<&crate::world::components::FactionMembership> {
        self.storage.faction_memberships.get(&entity)
    }

    /// Set faction memberships.
    #[inline]
    pub fn set_faction_memberships(&mut self, entity: Entity, value: crate::world::components::FactionMembership) {
        self.storage.faction_memberships.insert(entity, value);
    }

    /// Check if entity is monster of specific species.
    #[inline]
    pub fn is_monster_of(&self, entity: Entity, species: crate::world::components::MonsterSpecies) -> bool {
        matches!(self.storage.kinds.get(&entity), Some(crate::world::components::EntityKind::Monster(s)) if *s == species)
    }

    /// Get territory owner for cell.
    #[inline]
    pub fn territory_owner(&self, cell_x: u32, cell_y: u32) -> Option<crate::world::components::MonsterSpecies> {
        self.lifecycle.territory.get(&(cell_x, cell_y)).copied()
    }

    /// Rebuild spatial index.
    pub fn rebuild_spatial(&mut self) {
        self.storage.spatial.clear();
        for &e in self.lifecycle.alive() {
            if let Some(t) = self.storage.transforms.get(&e) {
                self.storage.spatial.insert(e, t.x, t.y);
            }
        }
    }

    /// Get all NPCs.
    pub fn npcs(&self) -> Vec<Entity> {
        self.lifecycle.alive()
            .iter()
            .copied()
            .filter(|e| matches!(self.storage.kinds.get(e), Some(crate::world::components::EntityKind::Npc)))
            .collect()
    }

    /// Get all monsters.
    pub fn monsters(&self) -> Vec<Entity> {
        self.lifecycle.alive()
            .iter()
            .copied()
            .filter(|e| matches!(self.storage.kinds.get(e), Some(crate::world::components::EntityKind::Monster(_))))
            .collect()
    }

    /// Count NPCs.
    pub fn count_npcs(&self) -> usize {
        self.npcs().len()
    }

    /// Count monsters of specific species.
    pub fn count_species(&self, species: crate::world::components::MonsterSpecies) -> usize {
        self.lifecycle.alive()
            .iter()
            .filter(|&&e| matches!(self.storage.kinds.get(&e), Some(crate::world::components::EntityKind::Monster(s)) if *s == species))
            .count()
    }

    /// Drain spatial inserted journal.
    pub fn drain_spatial_inserted_journal(&mut self) -> Vec<Entity> {
        self.lifecycle.drain_spatial_inserted()
    }

    /// Drain spatial moved journal.
    pub fn drain_spatial_moved_journal(&mut self) -> Vec<Entity> {
        self.lifecycle.drain_spatial_moved()
    }

    /// Drain spatial removed journal.
    pub fn drain_spatial_removed_journal(&mut self) -> Vec<Entity> {
        self.lifecycle.drain_spatial_removed()
    }

    // ========================================================================
    // COMPATIBILITY GETTERS - Added for backward compatibility
    // ========================================================================

    /// Get name (compatibility getter)
    pub fn get_name(&self, entity: Entity) -> Option<&crate::world::components::Name> {
        self.storage.names.get(&entity)
    }

    /// Get transform mutable (compatibility getter)
    pub fn get_transform_mut(&mut self, entity: Entity) -> Option<&mut crate::world::components::Transform> {
        self.storage.transforms.get_mut(&entity)
    }

    /// Get simulation level (compatibility getter)
    pub fn get_sim_level(&self, entity: Entity) -> Option<&crate::world::components::SimLevel> {
        self.storage.sim_levels.get(&entity)
    }

    /// Get emotions (compatibility getter)
    pub fn get_emotions(&self, entity: Entity) -> Option<&crate::core::ai_emotions::Emotions> {
        self.storage.emotions.get(&entity)
    }

    /// Get inventory (compatibility getter)
    pub fn get_inventory(&self, entity: Entity) -> Option<&crate::world::components::Inventory> {
        self.storage.inventories.get(&entity)
    }

    /// Get NPC economy (compatibility getter)
    pub fn get_npc_economy(&self, entity: Entity) -> Option<&crate::world::components::NpcEconomy> {
        self.storage.npc_economies.get(&entity)
    }

    // ========================================================================
    // IDENTITY ACCESSOR - Added for backward compatibility
    // ========================================================================

    /// Get identity subsystem (compatibility accessor)
    pub fn identity(&self) -> &crate::core::ecs_internal::identity::IdentitySubsystem {
        &self.storage.identity
    }

    /// Get mutable identity subsystem (compatibility accessor)
    pub fn identity_mut(&mut self) -> &mut crate::core::ecs_internal::identity::IdentitySubsystem {
        &mut self.storage.identity
    }

    // ========================================================================
    // READ-ONLY COMPATIBILITY ACCESSORS - Added for backward compatibility
    // ========================================================================

    /// Get alive entities (compatibility getter)
    pub fn alive(&self) -> &[Entity] {
        self.lifecycle.alive()
    }

    /// Get transforms (compatibility getter)
    pub fn transforms(&self) -> &engine_ecs::SparseSet<crate::world::components::Transform> {
        &self.storage.transforms
    }

    /// Get names (compatibility getter)
    pub fn names(&self) -> &engine_ecs::SparseSet<crate::world::components::Name> {
        &self.storage.names
    }

    /// Get kinds (compatibility getter)
    pub fn kinds(&self) -> &engine_ecs::SparseSet<crate::world::components::EntityKind> {
        &self.storage.kinds
    }
}

impl Default for Ecs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_assigns_unique_ids() {
        let mut ecs = Ecs::new();
        let e1 = ecs.spawn();
        let e2 = ecs.spawn();
        let e3 = ecs.spawn();

        assert_ne!(e1, e2);
        assert_ne!(e2, e3);
        assert_ne!(e1, e3);
        assert_eq!(ecs.lifecycle.count(), 3);
    }

    #[test]
    fn test_spawn_new_assigns_persistent_id() {
        let mut ecs = Ecs::new();
        let (e1, pid1) = ecs.spawn_new();
        let (e2, pid2) = ecs.spawn_new();

        assert_ne!(pid1, pid2);
        assert_eq!(ecs.ecs.identity.persistent_id_of(e1), Some(pid1));
        assert_eq!(ecs.ecs.identity.persistent_id_of(e2), Some(pid2));
    }

    #[test]
    fn test_despawn_removes_from_alive() {
        let mut ecs = Ecs::new();
        let e1 = ecs.spawn();
        let e2 = ecs.spawn();

        assert!(ecs.is_alive(e1));
        ecs.despawn(e1);
        assert!(!ecs.is_alive(e1));
        assert!(ecs.is_alive(e2));
        assert_eq!(ecs.lifecycle.count(), 1);
    }

    #[test]
    fn test_component_crud() {
        use crate::world::components::{Transform, EntityKind};

        let mut ecs = Ecs::new();
        let e = ecs.spawn();

        ecs.transforms_mut().insert(e, Transform { x: 10.0, y: 20.0, cell_x: 0, cell_y: 0 });
        ecs.kinds_mut().insert(e, EntityKind::Npc);

        let t = ecs.transforms().get(&e).unwrap();
        assert_eq!(t.x, 10.0);
        assert_eq!(t.y, 20.0);

        ecs.transforms_mut().insert(e, Transform { x: 30.0, y: 40.0, cell_x: 1, cell_y: 1 });
        let t2 = ecs.transforms().get(&e).unwrap();
        assert_eq!(t2.x, 30.0);

        ecs.transforms_mut().remove(&e);
        assert!(ecs.transforms().get(&e).is_none());
    }

    #[test]
    fn test_npcs_and_monsters_filters() {
        use crate::world::components::{EntityKind, MonsterSpecies};

        let mut ecs = Ecs::new();

        let npc1 = ecs.spawn();
        ecs.kinds_mut().insert(npc1, EntityKind::Npc);

        let npc2 = ecs.spawn();
        ecs.kinds_mut().insert(npc2, EntityKind::Npc);

        let monster1 = ecs.spawn();
        ecs.kinds_mut().insert(monster1, EntityKind::Monster(MonsterSpecies::Wolf));

        let monster2 = ecs.spawn();
        ecs.kinds_mut().insert(monster2, EntityKind::Monster(MonsterSpecies::Boar));

        assert_eq!(ecs.npcs().len(), 2);
        assert_eq!(ecs.monsters().len(), 2);
        assert_eq!(ecs.count_npcs(), 2);
        assert_eq!(ecs.count_species(MonsterSpecies::Wolf), 1);
        assert_eq!(ecs.count_species(MonsterSpecies::Boar), 1);
    }

    #[test]
    fn test_rebuild_spatial() {
        use crate::world::components::Transform;

        let mut ecs = Ecs::new();

        let e1 = ecs.spawn();
        ecs.transforms_mut().insert(e1, Transform { x: 100.0, y: 100.0, cell_x: 0, cell_y: 0 });

        let e2 = ecs.spawn();
        ecs.transforms_mut().insert(e2, Transform { x: 200.0, y: 200.0, cell_x: 0, cell_y: 0 });

        ecs.rebuild_spatial();

        let near = ecs.spatial().candidates_in_radius(100.0, 100.0, 50.0);
        assert_eq!(near.len(), 1);
        assert!(near.contains(&e1));
    }

    #[test]
    fn test_tick_advancement() {
        let mut ecs = Ecs::new();
        assert_eq!(ecs.tick(), 0);
        
        ecs.advance_tick();
        assert_eq!(ecs.tick(), 1);
    }

    #[test]
    fn test_unload_preserves_persistent_id() {
        use engine_ecs::persistent_id::EntityPresence;

        let mut ecs = Ecs::new();
        let (entity, pid) = ecs.spawn_new();
        
        ecs.unload_entity(entity);
        
        assert!(!ecs.is_alive(entity));
        assert_eq!(ecs.ecs.identity.presence(pid), EntityPresence::Unloaded);
    }
}