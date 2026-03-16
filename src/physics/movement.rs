use crate::core::ecs::{Ecs, Entity};
use crate::world::cell;

#[derive(Clone, Debug)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,
}

impl Default for Velocity {
    fn default() -> Self {
        Self { vx: 0.0, vy: 0.0 }
    }
}

pub fn apply_velocity(ecs: &mut Ecs, entity: Entity, _delta: f32) {
    let t = match ecs.transforms.get_mut(&entity) {
        Some(t) => t,
        None => return,
    };

    t.x = (t.x).clamp(0.0, cell::GRID_SIZE as f32 * cell::CELL_SIZE);
    t.y = (t.y).clamp(0.0, cell::GRID_SIZE as f32 * cell::CELL_SIZE);

    let (cx, cy) = cell::pos_to_cell(t.x, t.y);
    t.cell_x = cx;
    t.cell_y = cy;
}

pub fn move_toward(
    ecs: &mut Ecs,
    entity: Entity,
    target_x: f32,
    target_y: f32,
    speed: f32,
    delta: f32,
) -> bool {
    let t = match ecs.transforms.get_mut(&entity) {
        Some(t) => t,
        None => return false,
    };

    let dx = target_x - t.x;
    let dy = target_y - t.y;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < speed * delta {
        t.x = target_x;
        t.y = target_y;
        let (cx, cy) = cell::pos_to_cell(t.x, t.y);
        t.cell_x = cx;
        t.cell_y = cy;
        return true;
    }

    let step = speed * delta / dist;
    t.x += dx * step;
    t.y += dy * step;
    let (cx, cy) = cell::pos_to_cell(t.x, t.y);
    t.cell_x = cx;
    t.cell_y = cy;
    false
}
