use std::collections::HashMap;
use std::fmt;
pub type Entity = u64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct DuplicateIdError {
    pub persistent_id: PersistentEntityId,
    pub existing_entity: Entity,
    pub new_entity: Entity,
}

impl fmt::Display for DuplicateIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "duplicate PersistentEntityId({}) — already live as Entity {}, attempted by Entity {}",
            self.persistent_id.0, self.existing_entity, self.new_entity
        )
    }
}

/// Stable identity that survives unload/reload/migration.
/// Unlike runtime Entity (u64), PersistentEntityId is assigned once
/// and never changes for the lifetime of an entity in the world.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistentEntityId(pub u64);

/// State of an entity that may not be currently loaded
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityPresence {
    /// Entity is alive and loaded in ECS with the given runtime handle
    Live(Entity),
    /// Entity exists but is in an unloaded chunk (background sim)
    Unloaded,
    /// Entity is permanently dead (tombstone kept for reference resolution)
    Dead,
}

/// Bidirectional mapping between stable PersistentEntityId and runtime Entity handles.
/// This is the single source of truth for "who is who" across streaming boundaries.
pub struct IdentityRegistry {
    next_persistent_id: u64,
    /// persistent -> presence
    persistent_to_presence: HashMap<PersistentEntityId, EntityPresence>,
    /// live entity -> persistent (only for Live entities)
    live_to_persistent: HashMap<Entity, PersistentEntityId>,
    /// Tombstones: dead entities kept for reference resolution
    tombstone_tick: HashMap<PersistentEntityId, u64>,
}

impl IdentityRegistry {
    pub fn new() -> Self {
        Self {
            next_persistent_id: 1, // 0 reserved as "null"
            persistent_to_presence: HashMap::new(),
            live_to_persistent: HashMap::new(),
            tombstone_tick: HashMap::new(),
        }
    }

    /// Assign a new PersistentEntityId to a freshly spawned entity
    pub fn register_new(&mut self, entity: Entity) -> PersistentEntityId {
        let pid = PersistentEntityId(self.next_persistent_id);
        self.next_persistent_id += 1;
        self.persistent_to_presence
            .insert(pid, EntityPresence::Live(entity));
        self.live_to_persistent.insert(entity, pid);
        pid
    }

    /// Register an entity restored from a snapshot with its known persistent ID.
    /// Returns `Err(DuplicateIdError)` if another live entity already holds this PID.
    pub fn register_restored(
        &mut self,
        pid: PersistentEntityId,
        entity: Entity,
    ) -> Result<(), DuplicateIdError> {
        if let Some(EntityPresence::Live(existing)) = self.persistent_to_presence.get(&pid) {
            if *existing != entity {
                return Err(DuplicateIdError {
                    persistent_id: pid,
                    existing_entity: *existing,
                    new_entity: entity,
                });
            }
        }
        self.persistent_to_presence
            .insert(pid, EntityPresence::Live(entity));
        self.live_to_persistent.insert(entity, pid);
        self.tombstone_tick.remove(&pid);
        if pid.0 >= self.next_persistent_id {
            self.next_persistent_id = pid.0 + 1;
        }
        Ok(())
    }

    /// Mark entity as unloaded (chunk unload). Removes live mapping but keeps persistent record.
    pub fn mark_unloaded(&mut self, pid: PersistentEntityId) {
        if let Some(EntityPresence::Live(entity)) = self.persistent_to_presence.get(&pid) {
            self.live_to_persistent.remove(entity);
        }
        self.persistent_to_presence
            .insert(pid, EntityPresence::Unloaded);
    }

    /// Mark entity as permanently dead. Creates tombstone.
    pub fn mark_dead(&mut self, pid: PersistentEntityId, current_tick: u64) {
        if let Some(EntityPresence::Live(entity)) = self.persistent_to_presence.get(&pid) {
            self.live_to_persistent.remove(entity);
        }
        self.persistent_to_presence
            .insert(pid, EntityPresence::Dead);
        self.tombstone_tick.insert(pid, current_tick);
    }

    /// Resolve a PersistentEntityId to its current runtime Entity, if live
    pub fn resolve(&self, pid: PersistentEntityId) -> Option<Entity> {
        match self.persistent_to_presence.get(&pid) {
            Some(EntityPresence::Live(e)) => Some(*e),
            _ => None,
        }
    }

    /// Get the presence state of a persistent entity
    pub fn presence(&self, pid: PersistentEntityId) -> EntityPresence {
        self.persistent_to_presence
            .get(&pid)
            .copied()
            .unwrap_or(EntityPresence::Dead)
    }

    /// Look up the persistent ID for a live entity
    pub fn persistent_id_of(&self, entity: Entity) -> Option<PersistentEntityId> {
        self.live_to_persistent.get(&entity).copied()
    }

    /// Get all currently live entity mappings
    pub fn live_entities(&self) -> impl Iterator<Item = (PersistentEntityId, Entity)> + '_ {
        self.live_to_persistent.iter().map(|(&e, &pid)| (pid, e))
    }

    /// Garbage-collect tombstones older than max_age ticks
    pub fn gc_tombstones(&mut self, current_tick: u64, max_age: u64) {
        let expired: Vec<PersistentEntityId> = self
            .tombstone_tick
            .iter()
            .filter(|(_, &tick)| current_tick.saturating_sub(tick) > max_age)
            .map(|(&pid, _)| pid)
            .collect();
        for pid in expired {
            self.tombstone_tick.remove(&pid);
            self.persistent_to_presence.remove(&pid);
        }
    }

    pub fn live_count(&self) -> usize {
        self.live_to_persistent.len()
    }

    pub fn total_count(&self) -> usize {
        self.persistent_to_presence.len()
    }

    pub fn tombstone_count(&self) -> usize {
        self.tombstone_tick.len()
    }
}

impl Default for IdentityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A reference to another entity that uses PersistentEntityId for stability.
/// When the target is unloaded, resolution returns None but the reference remains valid.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRef(pub PersistentEntityId);

impl EntityRef {
    pub fn new(pid: PersistentEntityId) -> Self {
        Self(pid)
    }

    /// Resolve to live Entity handle. Returns None if target is unloaded or dead.
    pub fn resolve(&self, registry: &IdentityRegistry) -> Option<Entity> {
        registry.resolve(self.0)
    }

    /// Check if the referenced entity is permanently dead
    pub fn is_dead(&self, registry: &IdentityRegistry) -> bool {
        matches!(registry.presence(self.0), EntityPresence::Dead)
    }

    /// Check if the referenced entity is simply unloaded (not dead)
    pub fn is_unloaded(&self, registry: &IdentityRegistry) -> bool {
        matches!(registry.presence(self.0), EntityPresence::Unloaded)
    }
}

/// Reference relink context used after chunk load to update stale runtime Entity handles
pub struct RelinkContext<'a> {
    pub registry: &'a IdentityRegistry,
}

impl<'a> RelinkContext<'a> {
    /// Given a stale Entity from a previous session, find its current live handle
    /// via the persistent ID stored in the snapshot
    pub fn resolve_persistent(&self, pid: PersistentEntityId) -> Option<Entity> {
        self.registry.resolve(pid)
    }
}

// ── Reference Hygiene Audit ──────────────────────────────────────────
//
// Classification of all raw Entity references in the codebase:
//
// MUST MIGRATE to PersistentEntityId (cross-session/cross-chunk references):
//   - src/ai/memory.rs: Memory.entities: HashMap<Entity, EntityOpinion>
//     -> EventMemory.other: Option<Entity>
//   - src/ai/social.rs: try_trade(buyer: Entity, seller: Entity) params are fine
//     (transient within tick), but stored opinions use Entity keys
//   - src/ai/groups.rs: Group { leader: Entity, members: Vec<Entity> }
//   - src/body/: BodyStateStore keyed by Entity
//   - src/core/events/canonical.rs: event target fields (entity, attacker, etc.)
//
// CAN STAY as runtime Entity (rebuilt on load, transient):
//   - src/world/spatial_index.rs: spatial lookups (rebuilt per tick)
//   - src/graphics/*: render instance data (rebuilt each frame)
//   - src/physics/rapier_world.rs: physics handles (rebuilt on entity creation)
//   - src/core/ecs.rs: alive/alive_set (runtime state)
//   - src/core/commands.rs: CommandBuffer targets (within-tick only)
//
// NEEDS DEFERRED RESOLUTION (cross-chunk):
//   - AI memory opinions about entities in unloaded chunks
//   - Group membership when leader/member is in different chunk
//   - Social ties to NPCs in unloaded regions
//
// REFERENCE FORMAT POLICY:
//   - New persistent cross-entity references MUST use EntityRef
//   - Within-tick transient references MAY use raw Entity
//   - Serialized/saved references MUST include PersistentEntityId
//   - Event fields that reference entities should include PersistentEntityId
//     for replay/causality, raw Entity for within-tick dispatch
