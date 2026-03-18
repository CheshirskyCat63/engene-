//! World Map Panel — 2D overview showing chunks, entities, danger zones, biomes.

use crate::core::ecs::Ecs;
use crate::world::cell::{CELL_SIZE, GRID_SIZE};
use crate::world::components::EntityKind;

#[derive(Debug, Clone)]
pub struct WorldMapCell {
    pub x: u32,
    pub y: u32,
    pub npc_count: u32,
    pub monster_count: u32,
    pub danger: f32,
}

pub struct WorldMapPanel {
    pub cells: Vec<WorldMapCell>,
    pub grid_size: u32,
    pub cell_size: f32,
}

impl WorldMapPanel {
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            grid_size: GRID_SIZE,
            cell_size: CELL_SIZE,
        }
    }

    pub fn update_from_ecs(&mut self, ecs: &Ecs) {
        let gs = self.grid_size as usize;
        let mut npc_counts = vec![0u32; gs * gs];
        let mut monster_counts = vec![0u32; gs * gs];

        for &e in &ecs.alive {
            if let Some(t) = ecs.get_transform(e) {
                let cx = (t.x / self.cell_size)
                    .min(self.grid_size as f32 - 1.0)
                    .max(0.0) as usize;
                let cy = (t.y / self.cell_size)
                    .min(self.grid_size as f32 - 1.0)
                    .max(0.0) as usize;
                let idx = cy * gs + cx;
                match ecs.get_kind(e) {
                    Some(EntityKind::Npc) => npc_counts[idx] += 1,
                    Some(EntityKind::Monster(_)) => monster_counts[idx] += 1,
                    _ => {}
                }
            }
        }

        self.cells.clear();
        for y in 0..self.grid_size {
            for x in 0..self.grid_size {
                let idx = y as usize * gs + x as usize;
                if npc_counts[idx] > 0 || monster_counts[idx] > 0 {
                    self.cells.push(WorldMapCell {
                        x,
                        y,
                        npc_count: npc_counts[idx],
                        monster_count: monster_counts[idx],
                        danger: 0.0,
                    });
                }
            }
        }
    }

    pub fn total_entities(&self) -> u32 {
        self.cells
            .iter()
            .map(|c| c.npc_count + c.monster_count)
            .sum()
    }
}

impl WorldMapPanel {
    pub fn draw_ui(&self, ctx: &egui::Context) {
        egui::Window::new("World Map")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.label(format!(
                    "Grid: {}x{} cells @ {:.0}m",
                    self.grid_size, self.grid_size, self.cell_size
                ));
                ui.label(format!(
                    "Populated cells: {}  Entities: {}",
                    self.cells.len(),
                    self.total_entities()
                ));
                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .show(ui, |ui| {
                        for cell in &self.cells {
                            ui.horizontal(|ui| {
                                ui.monospace(format!("[{},{}]", cell.x, cell.y));
                                ui.label(format!(
                                    "NPCs:{} Monsters:{}",
                                    cell.npc_count, cell.monster_count
                                ));
                            });
                        }
                    });
            });
    }
}

impl Default for WorldMapPanel {
    fn default() -> Self {
        Self::new()
    }
}
