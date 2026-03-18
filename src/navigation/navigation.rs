use crate::core::ecs::{Ecs, Entity};
use crate::navigation::world_graph::WorldGraph;
use crate::world::cell::CELL_SIZE;

pub fn travel_toward(
    ecs: &mut Ecs,
    entity: Entity,
    target_cx: u32,
    target_cy: u32,
    speed: f32,
    delta: f32,
) -> bool {
    let t = match ecs.get_transform_mut(entity) {
        Some(t) => t,
        None => return false,
    };

    let tx = target_cx as f32 * CELL_SIZE + CELL_SIZE * 0.5;
    let ty = target_cy as f32 * CELL_SIZE + CELL_SIZE * 0.5;
    let dx = tx - t.x;
    let dy = ty - t.y;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist < speed * delta {
        t.x = tx;
        t.y = ty;
        t.cell_x = target_cx;
        t.cell_y = target_cy;
        return true;
    }

    let step = speed * delta / dist;
    t.x += dx * step;
    t.y += dy * step;
    let (cx, cy) = crate::world::cell::pos_to_cell(t.x, t.y);
    t.cell_x = cx;
    t.cell_y = cy;
    false
}

pub fn estimated_travel_time(graph: &WorldGraph, from: u32, to: u32, speed: f32) -> f32 {
    let dist = graph.distance_between(from, to);
    dist / speed.max(0.01)
}
