//! ECS Mechanics - Pure ECS without world/gameplay dependencies.
//! Phase 2a.2: Ownership transfer to engine_ecs.
//! This module contains ECS mechanics that don't depend on world::components or AI types.

use slotmap::SlotMap;
use std::collections::HashSet;

pub use crate::entity::Entity;
pub use crate::persistent_id::{DuplicateIdError, IdentityRegistry, PersistentEntityId};

slotmap::new_key_type! {
    pub struct GenEntity;
}

/// Pure ECS identity and lifecycle mechanics.
/// Does NOT depend on world::components or AI types.
pub struct EcsMechanics {
    pub next_id: u64,
    pub tick: u64,
    pub gen_alloc: SlotMap<GenEntity, Entity>,
    pub identity: IdentityRegistry,

    // Lifecycle - purely runtime, no world types
    alive: Vec<Entity>,
    alive_set: HashSet<Entity>,

    // Journals for dirty tracking (component-agnostic)
    dirty_inserted: HashSet<Entity>,
    dirty_moved: HashSet<Entity>,
    dirty_removed: HashSet<Entity>,
}

impl EcsMechanics {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            tick: 0,
            gen_alloc: SlotMap::with_key(),
            identity: IdentityRegistry::new(),
            alive: Vec::new(),
            alive_set: HashSet::new(),
            dirty_inserted: HashSet::new(),
            dirty_moved: HashSet::new(),
            dirty_removed: HashSet::new(),
        }
    }

    /// Allocate a new transient entity ID (no PersistentEntityId).
    pub fn spawn(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        self.alive.push(id);
        self.alive_set.insert(id);
        self.gen_alloc.insert(id);
        self.dirty_inserted.insert(id);
        self.dirty_removed.remove(&id);
        debug_assert!(
            self.validate_invariants().is_ok(),
            "ECS invariant violation after spawn"
        );
        id
    }

    /// Spawn new gameplay entity with fresh PersistentEntityId.
    pub fn spawn_new(&mut self) -> (Entity, PersistentEntityId) {
        let entity = self.spawn();
        let pid = self.identity.register_new(entity);
        (entity, pid)
    }

    /// Restore entity from persistence with known PersistentEntityId.
    pub fn spawn_restored(&mut self, pid: PersistentEntityId) -> Result<Entity, DuplicateIdError> {
        let entity = self.spawn();
        self.identity.register_restored(pid, entity)?;
        Ok(entity)
    }

    /// Unload entity (chunk boundary) - marks as Unloaded not Dead.
    pub fn unload_entity(&mut self, entity: Entity) {
        if let Some(pid) = self.identity.persistent_id_of(entity) {
            self.identity.mark_unloaded(pid);
        }
        self.remove_entity(entity);
        debug_assert!(
            self.validate_invariants().is_ok(),
            "ECS invariant violation after unload"
        );
    }

    /// Permanently remove entity.
    pub fn despawn(&mut self, entity: Entity) {
        if let Some(pid) = self.identity.persistent_id_of(entity) {
            self.identity.mark_dead(pid, self.tick);
        }
        self.remove_entity(entity);
        debug_assert!(
            self.validate_invariants().is_ok(),
            "ECS invariant violation after despawn"
        );
    }

    fn remove_entity(&mut self, entity: Entity) {
        self.alive_set.remove(&entity);
        if let Some(pos) = self.alive.iter().position(|&e| e == entity) {
            self.alive.swap_remove(pos);
        }
        self.dirty_removed.insert(entity);
        self.dirty_inserted.remove(&entity);
        self.dirty_moved.remove(&entity);
    }

    /// Check if entity is alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.alive_set.contains(&entity)
    }

    /// Get count of alive entities.
    pub fn alive_count(&self) -> usize {
        self.alive.len()
    }

    /// Get alive entities iterator.
    pub fn alive(&self) -> &[Entity] {
        &self.alive
    }

    /// Advance tick counter.
    pub fn advance_tick(&mut self) {
        self.tick += 1;
    }

    /// Set tick directly.
    pub fn set_tick(&mut self, tick: u64) {
        self.tick = tick;
    }

    /// Validate ECS invariants.
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

        // Verify identity consistency
        let mut seen = HashSet::new();
        for (pid, entity) in self.identity.live_entities() {
            if !seen.insert(pid) {
                return Err(format!("duplicate live PersistentEntityId {}", pid.0));
            }
            if !self.alive_set.contains(&entity) {
                return Err(format!("identity live entity {} not in alive_set", entity));
            }
            if self.identity.persistent_id_of(entity) != Some(pid) {
                return Err(format!(
                    "identity reverse mapping mismatch for entity {}",
                    entity
                ));
            }
        }

        Ok(())
    }

    /// Mark entity as dirty (component inserted/updated).
    pub fn mark_dirty(&mut self, entity: Entity) {
        if self.alive_set.contains(&entity) {
            self.dirty_moved.insert(entity);
        }
    }

    /// Mark entity as removed from dirty tracking.
    pub fn mark_removed(&mut self, entity: Entity) {
        self.dirty_moved.remove(&entity);
    }

    /// Drain dirty inserted journal.
    pub fn drain_dirty_inserted(&mut self) -> Vec<Entity> {
        self.dirty_inserted.drain().collect()
    }

    /// Drain dirty moved journal.
    pub fn drain_dirty_moved(&mut self) -> Vec<Entity> {
        self.dirty_moved.drain().collect()
    }

    /// Drain dirty removed journal.
    pub fn drain_dirty_removed(&mut self) -> Vec<Entity> {
        self.dirty_removed.drain().collect()
    }
}

impl Default for EcsMechanics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_assigns_unique_ids() {
        let mut ecs = EcsMechanics::new();
        let e1 = ecs.spawn();
        let e2 = ecs.spawn();
        let e3 = ecs.spawn();

        assert_ne!(e1, e2);
        assert_ne!(e2, e3);
        assert_ne!(e1, e3);
        assert_eq!(ecs.alive_count(), 3);
    }

    #[test]
    fn test_spawn_new_assigns_persistent_id() {
        let mut ecs = EcsMechanics::new();
        let (e1, pid1) = ecs.spawn_new();
        let (e2, pid2) = ecs.spawn_new();

        assert_ne!(pid1, pid2);
        assert_eq!(ecs.identity.persistent_id_of(e1), Some(pid1));
        assert_eq!(ecs.identity.persistent_id_of(e2), Some(pid2));
    }

    #[test]
    fn test_despawn_removes_from_alive() {
        let mut ecs = EcsMechanics::new();
        let e1 = ecs.spawn();
        let e2 = ecs.spawn();

        assert!(ecs.is_alive(e1));
        ecs.despawn(e1);
        assert!(!ecs.is_alive(e1));
        assert!(ecs.is_alive(e2));
        assert_eq!(ecs.alive_count(), 1);
    }

    #[test]
    fn test_unload_entity_keeps_persistent_id() {
        let mut ecs = EcsMechanics::new();
        let (entity, pid) = ecs.spawn_new();

        // Unload should keep persistent ID but remove from alive
        ecs.unload_entity(entity);

        assert!(!ecs.is_alive(entity));
        // Presence should be Unloaded, not Dead
        assert!(matches!(
            ecs.identity.presence(pid),
            crate::persistent_id::EntityPresence::Unloaded
        ));
    }

    #[test]
    fn test_spawn_restored_with_existing_pid() {
        let mut ecs = EcsMechanics::new();

        // First spawn and despawn
        let (e1, pid) = ecs.spawn_new();
        ecs.despawn(e1);

        // Restore with same PID
        let e2 = ecs.spawn_restored(pid).expect("restore should succeed");

        assert_eq!(ecs.identity.persistent_id_of(e2), Some(pid));
        assert!(ecs.is_alive(e2));
    }

    #[test]
    fn test_spawn_restored_duplicate_pid_fails() {
        let mut ecs = EcsMechanics::new();

        // Create two entities
        let (e1, pid) = ecs.spawn_new();
        let _e2 = ecs.spawn();

        // Try to restore second entity with same PID - should fail
        let result = ecs.spawn_restored(pid);
        assert!(result.is_err());
    }

    #[test]
    fn test_lifecycle_invariants_maintained() {
        let mut ecs = EcsMechanics::new();

        let e1 = ecs.spawn();
        let e2 = ecs.spawn();

        // Valid state
        assert!(ecs.validate_invariants().is_ok());

        // After despawn, validate should still pass
        ecs.despawn(e1);
        assert!(ecs.validate_invariants().is_ok());

        // Despawn all should still pass
        ecs.despawn(e2);
        assert!(ecs.validate_invariants().is_ok());
    }

    #[test]
    fn test_dirty_journals_work() {
        let mut ecs = EcsMechanics::new();

        let e1 = ecs.spawn();
        let e2 = ecs.spawn();

        // Initially should have inserted journal
        let inserted = ecs.drain_dirty_inserted();
        assert_eq!(inserted.len(), 2);
        assert!(inserted.contains(&e1));
        assert!(inserted.contains(&e2));

        // After draining, should be empty
        assert!(ecs.dirty_inserted.is_empty());

        // Mark dirty and drain
        ecs.mark_dirty(e1);
        let moved = ecs.drain_dirty_moved();
        assert_eq!(moved, vec![e1]);

        ecs.despawn(e2);
        let removed = ecs.drain_dirty_removed();
        assert_eq!(removed, vec![e2]);
    }

    #[test]
    fn test_tick_advancement() {
        let mut ecs = EcsMechanics::new();

        assert_eq!(ecs.tick, 0);

        ecs.advance_tick();
        assert_eq!(ecs.tick, 1);

        ecs.set_tick(100);
        assert_eq!(ecs.tick, 100);
    }

    #[test]
    fn test_identity_live_entities() {
        let mut ecs = EcsMechanics::new();

        let (e1, pid1) = ecs.spawn_new();
        let (e2, pid2) = ecs.spawn_new();

        let live: Vec<_> = ecs.identity.live_entities().collect();
        assert_eq!(live.len(), 2);

        // Check pid1 -> e1 mapping exists
        let has_e1 = live.iter().any(|(pid, e)| *pid == pid1 && *e == e1);
        assert!(has_e1, "should have pid1 -> e1 mapping");

        // Check pid2 -> e2 mapping exists
        let has_e2 = live.iter().any(|(pid, e)| *pid == pid2 && *e == e2);
        assert!(has_e2, "should have pid2 -> e2 mapping");

        ecs.despawn(e1);

        let live_after: Vec<_> = ecs.identity.live_entities().collect();
        assert_eq!(live_after.len(), 1);
        assert_eq!(live_after[0].1, e2);
    }
}
