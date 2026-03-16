use std::collections::{HashMap, HashSet};

use slotmap::SlotMap;

use crate::ai::emotions::Emotions;
use crate::ai::memory::Memory;
use crate::ai::plan::Plan;
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
    }

    pub fn despawn(&mut self, entity: Entity) {
        if let Some(pid) = self.identity.persistent_id_of(entity) {
            self.identity.mark_dead(pid, self.tick);
        }
        self.remove_entity_data(entity);
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
