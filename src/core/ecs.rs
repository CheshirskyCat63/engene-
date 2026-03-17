use std::collections::{HashMap, HashSet};

use slotmap::SlotMap;

use crate::core::ai_emotions::Emotions;
use crate::core::ai_memory::Memory;
use crate::core::ai_plan::Plan;
use crate::core::persistent_id::{DuplicateIdError, IdentityRegistry, PersistentEntityId};
use crate::core::sparse_set::SparseSet;
use crate::world::components::*;
use crate::world::extension_components::{Attributes, Blackboard, EntityTags, StatusEffects};
use crate::world::spatial_index::SpatialIndex;

pub type Entity = u64;

slotmap::new_key_type! {
    pub struct GenEntity;
}

pub struct Ecs {
    next_id: u64,
    pub tick: u64,
    pub gen_alloc: SlotMap<GenEntity, Entity>,
    pub alive: Vec<Entity>,
    alive_set: HashSet<Entity>,
    pub identity: IdentityRegistry,
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
    pub memories: SparseSet<Memory>,
    pub emotions: SparseSet<Emotions>,
    pub plans: SparseSet<Plan>,
    pub life_info: SparseSet<LifeInfo>,
    pub flammables: SparseSet<Flammable>,
    pub cloth_components: SparseSet<ClothComponent>,
    pub tags: SparseSet<EntityTags>,
    pub attributes: SparseSet<Attributes>,
    pub status_effects: SparseSet<StatusEffects>,
    pub blackboard: SparseSet<Blackboard>,
    pub equipment: SparseSet<EquipmentSlots>,
    pub faction_memberships: SparseSet<FactionMembership>,
    pub territory: HashMap<(u32, u32), MonsterSpecies>,
    pub spatial: SpatialIndex,
}

impl Ecs {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            tick: 0,
            gen_alloc: SlotMap::with_key(),
            alive: Vec::new(),
            alive_set: HashSet::new(),
            identity: IdentityRegistry::new(),
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
            territory: HashMap::new(),
            spatial: SpatialIndex::new(),
        }
    }

    /// Low-level spawn for transient/internal entities (particles, render-only).
    /// Does NOT assign a PersistentEntityId.
    pub fn spawn(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        self.alive.push(id);
        self.alive_set.insert(id);
        self.gen_alloc.insert(id);
        debug_assert!(self.validate_invariants().is_ok(), "ECS invariant violation after spawn");
        id
    }

    /// Spawn a new gameplay entity with a fresh PersistentEntityId.
    pub fn spawn_new(&mut self) -> (Entity, PersistentEntityId) {
        let entity = self.spawn();
        let pid = self.identity.register_new(entity);
        (entity, pid)
    }

    /// Restore an entity from persistence with its known PersistentEntityId.
    /// Returns `Err(DuplicateIdError)` if another live entity already claims this PID.
    pub fn spawn_restored(
        &mut self,
        pid: PersistentEntityId,
    ) -> Result<Entity, DuplicateIdError> {
        let entity = self.spawn();
        self.identity.register_restored(pid, entity)?;
        Ok(entity)
    }

    /// Remove entity from ECS for chunk unload. Marks as Unloaded (not Dead)
    /// so references can still resolve after reload.
    pub fn unload_entity(&mut self, entity: Entity) {
        if let Some(pid) = self.identity.persistent_id_of(entity) {
            self.identity.mark_unloaded(pid);
        }
        self.remove_entity_data(entity);
        debug_assert!(self.validate_invariants().is_ok(), "ECS invariant violation after unload");
    }

    pub fn despawn(&mut self, entity: Entity) {
        if let Some(pid) = self.identity.persistent_id_of(entity) {
            self.identity.mark_dead(pid, self.tick);
        }
        self.remove_entity_data(entity);
        debug_assert!(self.validate_invariants().is_ok(), "ECS invariant violation after despawn");
    }

    pub fn validate_invariants(&self) -> Result<(), String> {
        if self.alive.len() != self.alive_set.len() {
            return Err(format!(
                "alive/alive_set mismatch: {} vs {}",
                self.alive.len(),
                self.alive_set.len()
            ));
        }

        for &e in &self.alive {
            if !self.alive_set.contains(&e) {
                return Err(format!("alive entity {e} missing from alive_set"));
            }
        }

        let check_storage = |name: &str, entities: &[u64], alive_set: &HashSet<u64>| -> Result<(), String> {
            for &e in entities {
                if !alive_set.contains(&e) {
                    return Err(format!("storage '{name}' contains non-alive entity {e}"));
                }
            }
            Ok(())
        };

        check_storage("transforms", self.transforms.entities(), &self.alive_set)?;
        check_storage("kinds", self.kinds.entities(), &self.alive_set)?;
        check_storage("names", self.names.entities(), &self.alive_set)?;
        check_storage("npc_traits", self.npc_traits.entities(), &self.alive_set)?;
        check_storage("monster_traits", self.monster_traits.entities(), &self.alive_set)?;
        check_storage("personal_needs", self.personal_needs.entities(), &self.alive_set)?;
        check_storage("social_needs", self.social_needs.entities(), &self.alive_set)?;
        check_storage("ecosystem_needs", self.ecosystem_needs.entities(), &self.alive_set)?;
        check_storage("npc_economies", self.npc_economies.entities(), &self.alive_set)?;
        check_storage("sim_levels", self.sim_levels.entities(), &self.alive_set)?;
        check_storage("ai_states", self.ai_states.entities(), &self.alive_set)?;
        check_storage("inventories", self.inventories.entities(), &self.alive_set)?;
        check_storage("memories", self.memories.entities(), &self.alive_set)?;
        check_storage("emotions", self.emotions.entities(), &self.alive_set)?;
        check_storage("plans", self.plans.entities(), &self.alive_set)?;
        check_storage("life_info", self.life_info.entities(), &self.alive_set)?;
        check_storage("equipment", self.equipment.entities(), &self.alive_set)?;
        check_storage("faction_memberships", self.faction_memberships.entities(), &self.alive_set)?;
        // Identity consistency: live identity mappings must point to alive entities and be bijective.
        let mut seen = HashSet::new();
        for (pid, entity) in self.identity.live_entities() {
            if !seen.insert(pid) {
                return Err(format!("duplicate live PersistentEntityId {}", pid.0));
            }
            if !self.alive_set.contains(&entity) {
                return Err(format!("identity live entity {} not in alive_set", entity));
            }
            if self.identity.persistent_id_of(entity) != Some(pid) {
                return Err(format!("identity reverse mapping mismatch for entity {}", entity));
            }
        }

        Ok(())
    }

    fn remove_entity_data(&mut self, entity: Entity) {
        self.alive_set.remove(&entity);
        if let Some(pos) = self.alive.iter().position(|&e| e == entity) {
            self.alive.swap_remove(pos);
        }
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

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.alive_set.contains(&entity)
    }

    pub fn npcs(&self) -> Vec<Entity> {
        self.alive
            .iter()
            .copied()
            .filter(|e| matches!(self.kinds.get(e), Some(EntityKind::Npc)))
            .collect()
    }

    pub fn monsters(&self) -> Vec<Entity> {
        self.alive
            .iter()
            .copied()
            .filter(|e| matches!(self.kinds.get(e), Some(EntityKind::Monster(_))))
            .collect()
    }

    pub fn rebuild_spatial(&mut self) {
        self.spatial.clear();
        for &e in &self.alive {
            if let Some(t) = self.transforms.get(&e) {
                let (x, y) = (t.x, t.y);
                self.spatial.insert(e, x, y);
            }
        }
    }

    pub fn count_species(&self, species: MonsterSpecies) -> usize {
        self.alive.iter().filter(|&&e| {
            matches!(self.kinds.get(&e), Some(EntityKind::Monster(s)) if *s == species)
        }).count()
    }

    pub fn count_npcs(&self) -> usize {
        self.alive.iter().filter(|&&e| {
            matches!(self.kinds.get(&e), Some(EntityKind::Npc))
        }).count()
    }

    // ========================================================================
    // QUERY API HELPERS - Use these instead of direct storage access
    // ========================================================================

    /// Get monster traits if entity is a monster.
    #[inline]
    pub fn get_monster_traits(&self, entity: Entity) -> Option<&MonsterTraits> {
        self.monster_traits.get(&entity)
    }

    /// Get monster traits mutable if entity is a monster.
    #[inline]
    pub fn get_monster_traits_mut(&mut self, entity: Entity) -> Option<&mut MonsterTraits> {
        self.monster_traits.get_mut(&entity)
    }

    /// Get entity display name.
    #[inline]
    pub fn get_name(&self, entity: Entity) -> Option<&Name> {
        self.names.get(&entity)
    }

    #[inline]
    pub fn name(&self, entity: Entity) -> Option<&Name> {
        self.get_name(entity)
    }

    /// Get entity display name mutable.
    #[inline]
    pub fn get_name_mut(&mut self, entity: Entity) -> Option<&mut Name> {
        self.names.get_mut(&entity)
    }

    #[inline]
    pub fn name_mut(&mut self, entity: Entity) -> Option<&mut Name> {
        self.get_name_mut(entity)
    }

    /// Get NPC traits if entity is an NPC.
    #[inline]
    pub fn get_npc_traits(&self, entity: Entity) -> Option<&NpcTraits> {
        self.npc_traits.get(&entity)
    }

    /// Get NPC traits mutable if entity is an NPC.
    #[inline]
    pub fn get_npc_traits_mut(&mut self, entity: Entity) -> Option<&mut NpcTraits> {
        self.npc_traits.get_mut(&entity)
    }

    /// Get ecosystem needs for a monster.
    #[inline]
    pub fn get_ecosystem_needs(&self, entity: Entity) -> Option<&EcosystemNeeds> {
        self.ecosystem_needs.get(&entity)
    }

    /// Get ecosystem needs mutable for a monster.
    #[inline]
    pub fn get_ecosystem_needs_mut(&mut self, entity: Entity) -> Option<&mut EcosystemNeeds> {
        self.ecosystem_needs.get_mut(&entity)
    }

    /// Get social needs for an NPC.
    #[inline]
    pub fn get_social_needs(&self, entity: Entity) -> Option<&SocialNeeds> {
        self.social_needs.get(&entity)
    }

    /// Get social needs mutable for an NPC.
    #[inline]
    pub fn get_social_needs_mut(&mut self, entity: Entity) -> Option<&mut SocialNeeds> {
        self.social_needs.get_mut(&entity)
    }

    /// Get memory for an entity.
    #[inline]
    pub fn get_memory(&self, entity: Entity) -> Option<&Memory> {
        self.memories.get(&entity)
    }

    /// Get memory mutable for an entity.
    #[inline]
    pub fn get_memory_mut(&mut self, entity: Entity) -> Option<&mut Memory> {
        self.memories.get_mut(&entity)
    }

    /// Get emotions for an entity.
    #[inline]
    pub fn get_emotions(&self, entity: Entity) -> Option<&Emotions> {
        self.emotions.get(&entity)
    }

    /// Get emotions mutable for an entity.
    #[inline]
    pub fn get_emotions_mut(&mut self, entity: Entity) -> Option<&mut Emotions> {
        self.emotions.get_mut(&entity)
    }

    /// Get plan for an entity.
    #[inline]
    pub fn get_plan(&self, entity: Entity) -> Option<&Plan> {
        self.plans.get(&entity)
    }

    /// Get plan mutable for an entity.
    #[inline]
    pub fn get_plan_mut(&mut self, entity: Entity) -> Option<&mut Plan> {
        self.plans.get_mut(&entity)
    }

    /// Get life info for an entity.
    #[inline]
    pub fn get_life_info(&self, entity: Entity) -> Option<&LifeInfo> {
        self.life_info.get(&entity)
    }

    /// Get life info mutable for an entity.
    #[inline]
    pub fn get_life_info_mut(&mut self, entity: Entity) -> Option<&mut LifeInfo> {
        self.life_info.get_mut(&entity)
    }

    /// Get transform mutable.
    #[inline]
    pub fn get_transform_mut(&mut self, entity: Entity) -> Option<&mut Transform> {
        self.transforms.get_mut(&entity)
    }

    #[inline]
    pub fn transform(&self, entity: Entity) -> Option<&Transform> {
        self.transforms.get(&entity)
    }

    #[inline]
    pub fn transform_mut(&mut self, entity: Entity) -> Option<&mut Transform> {
        self.transforms.get_mut(&entity)
    }

    #[inline]
    pub fn kind(&self, entity: Entity) -> Option<&EntityKind> {
        self.kinds.get(&entity)
    }

    #[inline]
    pub fn kind_mut(&mut self, entity: Entity) -> Option<&mut EntityKind> {
        self.kinds.get_mut(&entity)
    }

    #[inline]
    pub fn set_transform(&mut self, entity: Entity, value: Transform) {
        self.transforms.insert(entity, value);
    }

    #[inline]
    pub fn set_kind(&mut self, entity: Entity, value: EntityKind) {
        self.kinds.insert(entity, value);
    }

    #[inline]
    pub fn set_name(&mut self, entity: Entity, value: Name) {
        self.names.insert(entity, value);
    }

    #[inline]
    pub fn set_personal_needs(&mut self, entity: Entity, value: PersonalNeeds) {
        self.personal_needs.insert(entity, value);
    }

    #[inline]
    pub fn set_social_needs(&mut self, entity: Entity, value: SocialNeeds) {
        self.social_needs.insert(entity, value);
    }

    #[inline]
    pub fn set_npc_economy(&mut self, entity: Entity, value: NpcEconomy) {
        self.npc_economies.insert(entity, value);
    }

    #[inline]
    pub fn set_sim_level(&mut self, entity: Entity, value: SimLevel) {
        self.sim_levels.insert(entity, value);
    }

    #[inline]
    pub fn set_ai_state(&mut self, entity: Entity, value: AiState) {
        self.ai_states.insert(entity, value);
    }

    #[inline]
    pub fn set_inventory(&mut self, entity: Entity, value: Inventory) {
        self.inventories.insert(entity, value);
    }

    #[inline]
    pub fn set_memory(&mut self, entity: Entity, value: Memory) {
        self.memories.insert(entity, value);
    }

    #[inline]
    pub fn set_emotions(&mut self, entity: Entity, value: Emotions) {
        self.emotions.insert(entity, value);
    }

    #[inline]
    pub fn set_life_info(&mut self, entity: Entity, value: LifeInfo) {
        self.life_info.insert(entity, value);
    }

    #[inline]
    pub fn set_plan(&mut self, entity: Entity, value: Plan) {
        self.plans.insert(entity, value);
    }

    #[inline]
    pub fn take_plan(&mut self, entity: Entity) -> Option<Plan> {
        self.plans.remove(&entity)
    }

    #[inline]
    pub fn territory_owner(&self, cell_x: u32, cell_y: u32) -> Option<MonsterSpecies> {
        self.territory.get(&(cell_x, cell_y)).copied()
    }

    /// Get inventory for an entity.
    #[inline]
    pub fn get_inventory(&self, entity: Entity) -> Option<&Inventory> {
        self.inventories.get(&entity)
    }

 /// Get inventory mutable for an entity.
 #[inline]
 pub fn get_inventory_mut(&mut self, entity: Entity) -> Option<&mut Inventory> {
 self.inventories.get_mut(&entity)
 }

 /// Get simulation level for an entity.
 #[inline]
 pub fn get_sim_level(&self, entity: Entity) -> Option<&SimLevel> {
 self.sim_levels.get(&entity)
 }

 /// Get simulation level mutable for an entity.
 #[inline]
 pub fn get_sim_level_mut(&mut self, entity: Entity) -> Option<&mut SimLevel> {
 self.sim_levels.get_mut(&entity)
 }

 /// Get NPC economy for an NPC.
 #[inline]
 pub fn get_npc_economy(&self, entity: Entity) -> Option<&NpcEconomy> {
 self.npc_economies.get(&entity)
 }

    /// Get NPC economy mutable for an NPC.
    #[inline]
    pub fn get_npc_economy_mut(&mut self, entity: Entity) -> Option<&mut NpcEconomy> {
        self.npc_economies.get_mut(&entity)
    }

    /// Check if entity is a monster of specific species.
    #[inline]
    pub fn is_monster_of(&self, entity: Entity, species: MonsterSpecies) -> bool {
        matches!(self.kinds.get(&entity), Some(EntityKind::Monster(s)) if *s == species)
    }

    /// Check if entity is a nocturnal monster (Bloodsucker).
    #[inline]
    pub fn is_nocturnal(&self, entity: Entity) -> bool {
        matches!(self.kinds.get(&entity), Some(EntityKind::Monster(MonsterSpecies::Bloodsucker)))
    }

    /// Get entity kind name for debugging.
    pub fn get_kind_name(&self, entity: Entity) -> &'static str {
        match self.kinds.get(&entity) {
            Some(EntityKind::Npc) => "NPC",
            Some(EntityKind::Monster(MonsterSpecies::Wolf)) => "Wolf",
            Some(EntityKind::Monster(MonsterSpecies::Boar)) => "Boar",
            Some(EntityKind::Monster(MonsterSpecies::Bloodsucker)) => "Bloodsucker",
            _ => "Unknown",
        }
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
        assert_eq!(ecs.alive.len(), 3);
    }

    #[test]
    fn test_spawn_new_assigns_persistent_id() {
        let mut ecs = Ecs::new();
        let (e1, pid1) = ecs.spawn_new();
        let (e2, pid2) = ecs.spawn_new();
        
        assert_ne!(pid1, pid2);
        assert_eq!(ecs.identity.persistent_id_of(e1), Some(pid1));
        assert_eq!(ecs.identity.persistent_id_of(e2), Some(pid2));
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
        assert_eq!(ecs.alive.len(), 1);
    }

    #[test]
    fn test_component_crud() {
        let mut ecs = Ecs::new();
        let e = ecs.spawn();
        
        // Insert
        ecs.transforms.insert(e, Transform { x: 10.0, y: 20.0, cell_x: 0, cell_y: 0 });
        ecs.kinds.insert(e, EntityKind::Npc);
        
        // Read
        let t = ecs.transforms.get(&e).unwrap();
        assert_eq!(t.x, 10.0);
        assert_eq!(t.y, 20.0);
        
        // Update
        ecs.transforms.insert(e, Transform { x: 30.0, y: 40.0, cell_x: 1, cell_y: 1 });
        let t2 = ecs.transforms.get(&e).unwrap();
        assert_eq!(t2.x, 30.0);
        
        // Delete
        ecs.transforms.remove(&e);
        assert!(ecs.transforms.get(&e).is_none());
    }

    #[test]
    fn test_despawn_removes_all_components() {
        let mut ecs = Ecs::new();
        let e = ecs.spawn();
        
        ecs.transforms.insert(e, Transform { x: 0.0, y: 0.0, cell_x: 0, cell_y: 0 });
        ecs.kinds.insert(e, EntityKind::Npc);
        ecs.names.insert(e, Name("Test".into()));
        
        ecs.despawn(e);
        
        assert!(ecs.transforms.get(&e).is_none());
        assert!(ecs.kinds.get(&e).is_none());
        assert!(ecs.names.get(&e).is_none());
    }

    #[test]
    fn test_npcs_and_monsters_filters() {
        let mut ecs = Ecs::new();
        
        let npc1 = ecs.spawn();
        ecs.kinds.insert(npc1, EntityKind::Npc);
        
        let npc2 = ecs.spawn();
        ecs.kinds.insert(npc2, EntityKind::Npc);
        
        let monster1 = ecs.spawn();
        ecs.kinds.insert(monster1, EntityKind::Monster(MonsterSpecies::Wolf));
        
        let monster2 = ecs.spawn();
        ecs.kinds.insert(monster2, EntityKind::Monster(MonsterSpecies::Boar));
        
        assert_eq!(ecs.npcs().len(), 2);
        assert_eq!(ecs.monsters().len(), 2);
        assert_eq!(ecs.count_npcs(), 2);
        assert_eq!(ecs.count_species(MonsterSpecies::Wolf), 1);
        assert_eq!(ecs.count_species(MonsterSpecies::Boar), 1);
    }

    #[test]
    fn test_rebuild_spatial() {
        let mut ecs = Ecs::new();
        
        let e1 = ecs.spawn();
        ecs.transforms.insert(e1, Transform { x: 100.0, y: 100.0, cell_x: 0, cell_y: 0 });
        
        let e2 = ecs.spawn();
        ecs.transforms.insert(e2, Transform { x: 200.0, y: 200.0, cell_x: 0, cell_y: 0 });
        
        ecs.rebuild_spatial();
        
        let near = ecs.spatial.candidates_in_radius(100.0, 100.0, 50.0);
        assert_eq!(near.len(), 1);
        assert!(near.contains(&e1));
    }
}
