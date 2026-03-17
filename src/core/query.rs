//! Query Layer for ECS - provides type-safe, filterable queries over entities.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: unit
//!
//! This module implements the Query API pattern for ECS access,
//! replacing direct component storage access with composable queries.
//!
//! ## Usage
//!
//! ```ignore
//! // Instead of direct access:
//! let needs = ecs.personal_needs.get(&entity); // DON'T DO THIS
//!
//! // Use query API:
//! for item in ecs.query_personal_needs() {
//!     item.needs.hunger += 0.1;
//! }
//! ```

use crate::core::ecs::Ecs;
use crate::world::components::*;

/// Trait for filtering entities in queries.
pub trait QueryFilter {
    /// Returns true if the entity matches the filter criteria.
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool;
}

// ============================================================================
// BASIC FILTERS
// ============================================================================

/// Filter that matches entities with a specific component type.
pub struct WithTransform;
impl QueryFilter for WithTransform {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        ecs.transforms.contains_key(&entity)
    }
}

/// Filter that matches entities with EntityKind component.
pub struct WithKind;
impl QueryFilter for WithKind {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        ecs.kinds.contains_key(&entity)
    }
}

/// Filter that matches NPC entities.
pub struct WithNpc;
impl QueryFilter for WithNpc {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        matches!(ecs.kinds.get(&entity), Some(EntityKind::Npc))
    }
}

/// Filter that matches Monster entities.
pub struct WithMonster;
impl QueryFilter for WithMonster {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        matches!(ecs.kinds.get(&entity), Some(EntityKind::Monster(_)))
    }
}

/// Filter that matches entities with PersonalNeeds component.
pub struct WithNeeds;
impl QueryFilter for WithNeeds {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        ecs.personal_needs.contains_key(&entity)
    }
}

/// Filter that matches entities with AiState component.
pub struct WithAiState;
impl QueryFilter for WithAiState {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        ecs.ai_states.contains_key(&entity)
    }
}

/// Filter that matches entities with Inventory component.
pub struct WithInventory;
impl QueryFilter for WithInventory {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        ecs.inventories.contains_key(&entity)
    }
}

/// Filter that matches entities with LifeInfo component.
pub struct WithLifeInfo;
impl QueryFilter for WithLifeInfo {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        ecs.life_info.contains_key(&entity)
    }
}

/// Filter that matches entities with Memory component.
pub struct WithMemory;
impl QueryFilter for WithMemory {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        ecs.memories.contains_key(&entity)
    }
}

/// Filter that matches entities with Emotions component.
pub struct WithEmotions;
impl QueryFilter for WithEmotions {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        ecs.emotions.contains_key(&entity)
    }
}

/// Filter that matches entities with Plan component.
pub struct WithPlan;
impl QueryFilter for WithPlan {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        ecs.plans.contains_key(&entity)
    }
}

/// Filter that matches entities with NpcEconomy component.
pub struct WithNpcEconomy;
impl QueryFilter for WithNpcEconomy {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        ecs.npc_economies.contains_key(&entity)
    }
}

/// Filter that matches entities with SocialNeeds component.
pub struct WithSocialNeeds;
impl QueryFilter for WithSocialNeeds {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        ecs.social_needs.contains_key(&entity)
    }
}

/// Filter that matches entities with EcosystemNeeds component.
pub struct WithEcosystemNeeds;
impl QueryFilter for WithEcosystemNeeds {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        ecs.ecosystem_needs.contains_key(&entity)
    }
}

/// Filter that matches entities with specific MonsterSpecies.
pub struct WithSpecies(pub MonsterSpecies);
impl QueryFilter for WithSpecies {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        matches!(ecs.kinds.get(&entity), Some(EntityKind::Monster(s)) if *s == self.0)
    }
}

/// Filter that matches alive entities (health > 0).
pub struct IsAlive;
impl QueryFilter for IsAlive {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        ecs.personal_needs.get(&entity).map_or(false, |pn| pn.health > 0.0)
    }
}

/// Filter that matches dead entities (health <= 0).
pub struct IsDead;
impl QueryFilter for IsDead {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        ecs.personal_needs.get(&entity).map_or(false, |pn| pn.health <= 0.0)
    }
}

/// Composite filter: matches if both A and B match.
pub struct And<A: QueryFilter, B: QueryFilter>(pub A, pub B);

impl<A: QueryFilter, B: QueryFilter> QueryFilter for And<A, B> {
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool {
        self.0.matches(ecs, entity) && self.1.matches(ecs, entity)
    }
}

/// Iterator over entities matching a query filter.
pub struct QueryIter<'a, F: QueryFilter> {
    ecs: &'a Ecs,
    entities: std::vec::IntoIter<u64>,
    filter: F,
}

impl<'a, F: QueryFilter> QueryIter<'a, F> {
    pub fn new(ecs: &'a Ecs, filter: F) -> Self {
        // Collect alive entities to a Vec for iteration
        let entities: Vec<u64> = ecs.alive.iter().cloned().collect();
        Self {
            ecs,
            entities: entities.into_iter(),
            filter,
        }
    }
}

impl<'a, F: QueryFilter> Iterator for QueryIter<'a, F> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entity = self.entities.next()?;  // entity: u64 (IntoIter владеет значениями)
            if self.filter.matches(self.ecs, entity) {
                return Some(entity);
            }
        }
    }
}

// ============================================================================
// QUERY ITEM TYPES - Bundled data for efficient iteration
// ============================================================================

/// Bundled NPC data for efficient query results.
#[derive(Clone, Debug)]
pub struct NpcQueryItem<'a> {
    pub entity: u64,
    pub transform: &'a Transform,
    pub name: &'a Name,
    pub needs: Option<&'a PersonalNeeds>,
    pub economy: Option<&'a NpcEconomy>,
}

/// Bundled Monster data for efficient query results.
#[derive(Clone, Debug)]
pub struct MonsterQueryItem<'a> {
    pub entity: u64,
    pub species: MonsterSpecies,
    pub transform: &'a Transform,
    pub needs: Option<&'a PersonalNeeds>,
    pub ecosystem: Option<&'a EcosystemNeeds>,
    pub ai_state: Option<&'a AiState>,
}

/// Bundled AI entity data (NPC or Monster).
#[derive(Clone, Debug)]
pub struct AiEntityQueryItem<'a> {
    pub entity: u64,
    pub kind: &'a EntityKind,
    pub transform: Option<&'a Transform>,
    pub needs: Option<&'a PersonalNeeds>,
    pub ai_state: Option<&'a AiState>,
    pub memory: Option<&'a crate::core::ai_memory::Memory>,
    pub emotions: Option<&'a crate::core::ai_emotions::Emotions>,
    pub plan: Option<&'a crate::core::ai_plan::Plan>,
}

/// Bundled data for combat participants.
#[derive(Clone, Debug)]
pub struct CombatantQueryItem<'a> {
    pub entity: u64,
    pub kind: &'a EntityKind,
    pub transform: Option<&'a Transform>,
    pub needs: Option<&'a PersonalNeeds>,
    pub life_info: Option<&'a LifeInfo>,
}

/// Read-only access to a single component.
#[derive(Clone, Copy)]
pub struct ReadComponent<'a, T> {
    pub entity: u64,
    pub component: &'a T,
}

/// Mutable access to a single component.
pub struct WriteComponent<'a, T> {
    pub entity: u64,
    pub component: &'a mut T,
}

/// Extension methods for Ecs to support query API.
impl Ecs {
    /// Create a query iterator with the given filter.
    pub fn query_filter<F: QueryFilter>(&self, filter: F) -> QueryIter<'_, F> {
        QueryIter::new(self, filter)
    }

    /// Iterate over all entities with Transform component.
    pub fn entities_with_transform(&self) -> impl Iterator<Item = (u64, &Transform)> {
        self.alive.iter()
            .filter_map(|e| self.transforms.get(e).map(|t| (*e, t)))
    }

    /// Iterate over all entities with Transform and EntityKind.
    pub fn iter_transform_kind(&self) -> impl Iterator<Item = (u64, &Transform, &EntityKind)> {
        self.alive.iter()
            .filter_map(|e| {
                let t = self.transforms.get(e)?;
                let k = self.kinds.get(e)?;
                Some((*e, t, k))
            })
    }

    /// Iterate over all NPCs with their data bundled.
    pub fn iter_npcs(&self) -> impl Iterator<Item = NpcQueryItem<'_>> {
        self.alive.iter()
            .filter(|e| matches!(self.kinds.get(e), Some(EntityKind::Npc)))
            .filter_map(move |e| {
                let transform = self.transforms.get(e)?;
                let name = self.names.get(e)?;
                Some(NpcQueryItem {
                    entity: *e,
                    transform,
                    name,
                    needs: self.personal_needs.get(e),
                    economy: self.npc_economies.get(e),
                })
            })
    }

    /// Iterate over all Monsters with their data bundled.
    pub fn iter_monsters(&self) -> impl Iterator<Item = MonsterQueryItem<'_>> {
        self.alive.iter()
            .filter_map(move |e| {
                let kind = self.kinds.get(e)?;
                let species = match kind {
                    EntityKind::Monster(s) => *s,
                    _ => return None,
                };
                let transform = self.transforms.get(e)?;
                Some(MonsterQueryItem {
                    entity: *e,
                    species,
                    transform,
                    needs: self.personal_needs.get(e),
                    ecosystem: self.ecosystem_needs.get(e),
                    ai_state: self.ai_states.get(e),
                })
            })
    }

    /// Iterate over all AI entities (NPCs and Monsters) with full AI data.
    pub fn iter_ai_entities(&self) -> impl Iterator<Item = AiEntityQueryItem<'_>> {
        self.alive.iter()
            .filter(|e| self.kinds.get(e).is_some())
            .filter_map(move |e| {
                let kind = self.kinds.get(e)?;
                Some(AiEntityQueryItem {
                    entity: *e,
                    kind,
                    transform: self.transforms.get(e),
                    needs: self.personal_needs.get(e),
                    ai_state: self.ai_states.get(e),
                    memory: self.memories.get(e),
                    emotions: self.emotions.get(e),
                    plan: self.plans.get(e),
                })
            })
    }

    /// Iterate over all combat-ready entities.
    pub fn iter_combatants(&self) -> impl Iterator<Item = CombatantQueryItem<'_>> {
        self.alive.iter()
            .filter(|e| self.kinds.get(e).is_some())
            .filter_map(move |e| {
                let kind = self.kinds.get(e)?;
                Some(CombatantQueryItem {
                    entity: *e,
                    kind,
                    transform: self.transforms.get(e),
                    needs: self.personal_needs.get(e),
                    life_info: self.life_info.get(e),
                })
            })
    }

    /// Collect entity ids that currently have PersonalNeeds.
    pub fn iter_needs_mut(&mut self) -> Vec<u64> {
        self.alive
            .iter()
            .copied()
            .filter(|e| self.personal_needs.get(e).is_some())
            .collect()
    }

    /// Get a single component for an entity (read-only).
    #[inline]
    pub fn get_transform(&self, entity: u64) -> Option<&Transform> {
        self.transforms.get(&entity)
    }

    #[inline]
    pub fn get_kind(&self, entity: u64) -> Option<&EntityKind> {
        self.kinds.get(&entity)
    }

    #[inline]
    pub fn get_needs(&self, entity: u64) -> Option<&PersonalNeeds> {
        self.personal_needs.get(&entity)
    }

    #[inline]
    pub fn get_needs_mut(&mut self, entity: u64) -> Option<&mut PersonalNeeds> {
        self.personal_needs.get_mut(&entity)
    }

    #[inline]
    pub fn get_ai_state(&self, entity: u64) -> Option<&AiState> {
        self.ai_states.get(&entity)
    }

    #[inline]
    pub fn get_ai_state_mut(&mut self, entity: u64) -> Option<&mut AiState> {
        self.ai_states.get_mut(&entity)
    }

    /// Collect all entities matching a filter into a Vec.
    pub fn query_collect<F: QueryFilter>(&self, filter: F) -> Vec<u64> {
        self.query_filter(filter).collect()
    }

    /// Count entities matching a filter.
    pub fn query_count<F: QueryFilter>(&self, filter: F) -> usize {
        self.query_filter(filter).count()
    }

    /// Get entity position if available.
    #[inline]
    pub fn get_position(&self, entity: u64) -> Option<(f32, f32)> {
        self.transforms.get(&entity).map(|t| (t.x, t.y))
    }

    /// Check if entity is alive (has health > 0).
    #[inline]
    pub fn is_entity_alive(&self, entity: u64) -> bool {
        self.personal_needs.get(&entity).map_or(false, |pn| pn.health > 0.0)
    }

    /// Get entity name if available.
    #[inline]
    pub fn get_entity_name(&self, entity: u64) -> Option<&str> {
        self.names.get(&entity).map(|n| n.0.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_transform_filter() {
        let mut ecs = Ecs::new();
        let e1 = ecs.spawn();
        let _e2 = ecs.spawn();  // entity without transform
        
        ecs.transforms.insert(e1, Transform { x: 0.0, y: 0.0, cell_x: 0, cell_y: 0 });
        
        let results: Vec<_> = ecs.query_filter(WithTransform).collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], e1);
    }

    #[test]
    fn test_with_npc_filter() {
        let mut ecs = Ecs::new();
        let e1 = ecs.spawn();
        let e2 = ecs.spawn();
        
        ecs.kinds.insert(e1, EntityKind::Npc);
        ecs.kinds.insert(e2, EntityKind::Monster(MonsterSpecies::Wolf));
        
        let results: Vec<_> = ecs.query_filter(WithNpc).collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], e1);
    }

    #[test]
    fn test_and_filter() {
        let mut ecs = Ecs::new();
        let e1 = ecs.spawn();
        let e2 = ecs.spawn();
        
        ecs.kinds.insert(e1, EntityKind::Npc);
        ecs.transforms.insert(e1, Transform { x: 0.0, y: 0.0, cell_x: 0, cell_y: 0 });
        ecs.kinds.insert(e2, EntityKind::Npc);
        // e2 has no transform
        
        let filter = And(WithNpc, WithTransform);
        let results: Vec<_> = ecs.query_filter(filter).collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], e1);
    }

    #[test]
    fn test_iter_npcs() {
        let mut ecs = Ecs::new();
        let e1 = ecs.spawn();
        
        ecs.kinds.insert(e1, EntityKind::Npc);
        ecs.transforms.insert(e1, Transform { x: 10.0, y: 20.0, cell_x: 0, cell_y: 0 });
        ecs.names.insert(e1, Name("Test NPC".into()));
        
        // Create PersonalNeeds with Default
        let needs = PersonalNeeds {
            health: 100.0,
            energy: 100.0,
            hunger: 0.0,
            thirst: 0.0,
            sleep: 0.0,
            fear: 0.0,
            curiosity: 0.0,
            ambitions: 0.0,
            discomfort: 0.0,
        };
        ecs.personal_needs.insert(e1, needs);
        
        let npcs: Vec<_> = ecs.iter_npcs().collect();
        assert_eq!(npcs.len(), 1);
        assert_eq!(npcs[0].entity, e1);
        assert_eq!(npcs[0].transform.x, 10.0);
        assert_eq!(npcs[0].name.0, "Test NPC");
        assert!(npcs[0].needs.is_some());
    }

    #[test]
    fn test_query_count() {
        let mut ecs = Ecs::new();
        
        for _ in 0..5 {
            let e = ecs.spawn();
            ecs.kinds.insert(e, EntityKind::Npc);
        }
        
        assert_eq!(ecs.query_count(WithNpc), 5);
    }
}
