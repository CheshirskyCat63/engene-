//! ECS Lifecycle Layer - Entity spawn/despawn/unload.
//! Phase 2a.2: TRANSITIONAL - depends on world types, cannot move to engine_ecs yet.
//! This module handles entity lifecycle state transitions.

use std::collections::{HashMap, HashSet};
use engine_ecs::ecs_mechanics::EcsMechanics;

pub use engine_ecs::Entity;

/// Lifecycle subsystem - combines pure ECS mechanics with gameplay-specific tracking.
/// TRANSITIONAL: Depends on MonsterSpecies from world::components.
/// Removal condition: when MonsterSpecies moves to engine_world.
pub struct LifecycleSubsystem {
    /// Pure ECS mechanics from engine_ecs - handles identity, spawn, despawn, alive tracking
    pub ecs: EcsMechanics,
    
    // Journals for spatial tracking (gameplay-specific, not pure ECS)
    pub spatial_inserted_journal: HashSet<Entity>,
    pub spatial_moved_journal: HashSet<Entity>,
    pub spatial_removed_journal: HashSet<Entity>,
    
    // Territory mapping - BLOCKED: depends on MonsterSpecies from world::components
    pub territory: HashMap<(u32, u32), crate::world::components::MonsterSpecies>,
}

impl LifecycleSubsystem {
    pub fn new() -> Self {
        Self {
            ecs: EcsMechanics::new(),
            spatial_inserted_journal: HashSet::new(),
            spatial_moved_journal: HashSet::new(),
            spatial_removed_journal: HashSet::new(),
            territory: HashMap::new(),
        }
    }

    /// Delegate to EcsMechanics for spawn
    pub fn spawn(&mut self) -> Entity {
        let entity = self.ecs.spawn();
        self.spatial_inserted_journal.insert(entity);
        self.spatial_removed_journal.remove(&entity);
        entity
    }

    /// Delegate to EcsMechanics for spawn_new
    pub fn spawn_new(&mut self) -> (Entity, engine_ecs::PersistentEntityId) {
        self.ecs.spawn_new()
    }

    /// Delegate to EcsMechanics for despawn
    pub fn despawn(&mut self, entity: Entity) {
        self.ecs.despawn(entity);
        self.spatial_removed_journal.insert(entity);
        self.spatial_inserted_journal.remove(&entity);
        self.spatial_moved_journal.remove(&entity);
    }

    /// Delegate to EcsMechanics for is_alive
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.ecs.is_alive(entity)
    }

    /// Delegate to EcsMechanics for alive_count
    pub fn count(&self) -> usize {
        self.ecs.alive_count()
    }

    /// Delegate to EcsMechanics for alive
    pub fn alive(&self) -> &[Entity] {
        self.ecs.alive()
    }

    /// Delegate to EcsMechanics for validate_invariants
    pub fn validate_invariants(&self) -> Result<(), String> {
        self.ecs.validate_invariants()
    }

    /// Drain and clear spatial inserted journal.
    pub fn drain_spatial_inserted(&mut self) -> Vec<Entity> {
        self.spatial_inserted_journal.drain().collect()
    }

    /// Drain and clear spatial moved journal.
    pub fn drain_spatial_moved(&mut self) -> Vec<Entity> {
        self.spatial_moved_journal.drain().collect()
    }

    /// Drain and clear spatial removed journal.
    pub fn drain_spatial_removed(&mut self) -> Vec<Entity> {
        self.spatial_removed_journal.drain().collect()
    }
}

impl Default for LifecycleSubsystem {
    fn default() -> Self {
        Self::new()
    }
}
