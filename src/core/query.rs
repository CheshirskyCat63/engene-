//! Query Layer for ECS - provides type-safe, filterable queries over entities.
//!
//! # Status: production
//! # Integration: enabled
//! # Tests: unit
//!
//! This module implements the Query API pattern for ECS access,
//! replacing direct component storage access with composable queries.

use crate::core::ecs::Ecs;
use crate::world::components::*;

/// Trait for filtering entities in queries.
pub trait QueryFilter {
    /// Returns true if the entity matches the filter criteria.
    fn matches(&self, ecs: &Ecs, entity: u64) -> bool;
}

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

/// Bundled NPC data for efficient query results.
#[derive(Clone, Debug)]
pub struct NpcQueryItem<'a> {
    pub entity: u64,
    pub transform: &'a Transform,
    pub name: &'a Name,
    pub needs: Option<&'a PersonalNeeds>,
    pub economy: Option<&'a NpcEconomy>,
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

    /// Collect all entities matching a filter into a Vec.
    pub fn query_collect<F: QueryFilter>(&self, filter: F) -> Vec<u64> {
        self.query_filter(filter).collect()
    }

    /// Count entities matching a filter.
    pub fn query_count<F: QueryFilter>(&self, filter: F) -> usize {
        self.query_filter(filter).count()
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
