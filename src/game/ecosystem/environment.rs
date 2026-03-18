use crate::core::ecs::Ecs;
use crate::world::cell::GRID_SIZE;
use crate::world::components::EntityKind;
use crate::world::world::WorldGrid;

/// Update cell danger levels based on monster positions.
///
/// Uses helper methods instead of direct storage access.
pub fn update_cell_danger(ecs: &Ecs, grid: &mut WorldGrid) {
    for cell in &mut grid.cells {
        cell.danger = cell.biome.danger_level();
    }

    for &e in &ecs.alive {
        if let Some(EntityKind::Monster(species)) = ecs.get_kind(e) {
            if let Some(t) = ecs.get_transform(e) {
                let cx = t.cell_x.min(GRID_SIZE - 1);
                let cy = t.cell_y.min(GRID_SIZE - 1);
                let cell = grid.get_mut(cx, cy);
                let threat = match species {
                    crate::world::components::MonsterSpecies::Wolf => 0.05,
                    crate::world::components::MonsterSpecies::Boar => 0.01,
                    crate::world::components::MonsterSpecies::Bloodsucker => 0.15,
                };
                cell.danger = (cell.danger + threat).min(1.0);
            }
        }
    }
}

pub fn update_cell_danger_scaled(ecs: &Ecs, grid: &mut WorldGrid, danger_mult: f32) {
    for cell in &mut grid.cells {
        cell.danger = cell.biome.danger_level() * danger_mult;
    }

    for &e in &ecs.alive {
        if let Some(EntityKind::Monster(species)) = ecs.get_kind(e) {
            if let Some(t) = ecs.get_transform(e) {
                let cx = t.cell_x.min(GRID_SIZE - 1);
                let cy = t.cell_y.min(GRID_SIZE - 1);
                let cell = grid.get_mut(cx, cy);
                let threat = match species {
                    crate::world::components::MonsterSpecies::Wolf => 0.05,
                    crate::world::components::MonsterSpecies::Boar => 0.01,
                    crate::world::components::MonsterSpecies::Bloodsucker => 0.15,
                };
                cell.danger = (cell.danger + threat * danger_mult).min(1.0);
            }
        }
    }
}

pub fn consume_food(ecs: &Ecs, grid: &mut WorldGrid) {
    for &e in &ecs.alive {
        if let Some(t) = ecs.get_transform(e) {
            let cx = t.cell_x.min(GRID_SIZE - 1);
            let cy = t.cell_y.min(GRID_SIZE - 1);
            let cell = grid.get_mut(cx, cy);
            cell.food = (cell.food - 0.002).max(0.0);
        }
    }
}
