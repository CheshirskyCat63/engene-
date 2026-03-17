use std::collections::HashMap;

use crate::core::ecs::Ecs;
use crate::world::cell::GRID_SIZE;
use crate::world::components::{EntityKind, MonsterSpecies};

pub type TerritoryMap = HashMap<(u32, u32), MonsterSpecies>;

/// Compute territory map based on monster positions.
/// 
/// Uses helper methods instead of direct storage access.
pub fn compute_territory(ecs: &Ecs) -> TerritoryMap {
    let mut counts: HashMap<(u32, u32, MonsterSpecies), u32> = HashMap::new();

    for &e in &ecs.alive {
        if let Some(EntityKind::Monster(species)) = ecs.get_kind(e) {
            if let Some(t) = ecs.get_transform(e) {
                let key = (t.cell_x.min(GRID_SIZE - 1), t.cell_y.min(GRID_SIZE - 1), *species);
                *counts.entry(key).or_default() += 1;
            }
        }
    }

    let mut territory = TerritoryMap::new();
    let mut best: HashMap<(u32, u32), (MonsterSpecies, u32)> = HashMap::new();

    for (&(cx, cy, sp), &cnt) in &counts {
        let entry = best.entry((cx, cy)).or_insert((sp, 0));
        if cnt > entry.1 {
            *entry = (sp, cnt);
        }
    }

    for ((cx, cy), (sp, _)) in best {
        territory.insert((cx, cy), sp);
    }

    territory
}
