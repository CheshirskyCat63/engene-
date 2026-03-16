use rand::Rng;

use crate::core::ecs::Ecs;
use crate::core::events::EventBus;
use crate::world::cell::{CELL_SIZE, GRID_SIZE};
use crate::world::components::*;
use crate::world::world::WorldGrid;

pub fn process_migration(ecs: &mut Ecs, events: &mut EventBus, grid: &WorldGrid) {
    let monsters: Vec<_> = ecs.monsters();
    let mut rng = rand::thread_rng();

    for entity in monsters {
        let should_migrate = {
            let eco = match ecs.ecosystem_needs.get(&entity) {
                Some(e) => e,
                None => continue,
            };
            let pn = match ecs.personal_needs.get(&entity) {
                Some(p) => p,
                None => continue,
            };
            eco.migration_urge > 0.6 || pn.hunger > 0.7
        };

        if !should_migrate {
            continue;
        }

        let (old_cx, old_cy, species) = {
            let t = match ecs.transforms.get(&entity) {
                Some(t) => t,
                None => continue,
            };
            let sp = match ecs.kinds.get(&entity) {
                Some(EntityKind::Monster(s)) => *s,
                _ => continue,
            };
            (t.cell_x, t.cell_y, sp)
        };

        let mut best_cell = (old_cx, old_cy);
        let mut best_food = grid.get(old_cx, old_cy).food;

        for dx in -2i32..=2 {
            for dy in -2i32..=2 {
                let nx = (old_cx as i32 + dx).clamp(0, GRID_SIZE as i32 - 1) as u32;
                let ny = (old_cy as i32 + dy).clamp(0, GRID_SIZE as i32 - 1) as u32;
                let cell = grid.get(nx, ny);
                if cell.food > best_food {
                    best_food = cell.food;
                    best_cell = (nx, ny);
                }
            }
        }

        if best_cell != (old_cx, old_cy) {
            if let Some(t) = ecs.transforms.get_mut(&entity) {
                t.cell_x = best_cell.0;
                t.cell_y = best_cell.1;
                t.x = best_cell.0 as f32 * CELL_SIZE + rng.gen_range(0.0..CELL_SIZE);
                t.y = best_cell.1 as f32 * CELL_SIZE + rng.gen_range(0.0..CELL_SIZE);
            }
            if let Some(eco) = ecs.ecosystem_needs.get_mut(&entity) {
                eco.migration_urge = (eco.migration_urge - 0.4).max(0.0);
            }

            events.emit(MigrationOccurred {
                species,
                from: (old_cx, old_cy),
                to: best_cell,
            });
        }
    }
}
