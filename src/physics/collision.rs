use crate::core::ecs::{Ecs, Entity};

const ENTITY_RADIUS: f32 = 1.5;

pub fn resolve_collisions(ecs: &mut Ecs, entities: &[Entity]) {
    let positions: Vec<(Entity, f32, f32)> = entities
        .iter()
        .filter_map(|&e| ecs.transforms.get(&e).map(|t| (e, t.x, t.y)))
        .collect();

    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let (ea, ax, ay) = positions[i];
            let (eb, bx, by) = positions[j];

            let dx = bx - ax;
            let dy = by - ay;
            let dist_sq = dx * dx + dy * dy;
            let min_dist = ENTITY_RADIUS * 2.0;

            if dist_sq < min_dist * min_dist && dist_sq > 0.0001 {
                let dist = dist_sq.sqrt();
                let overlap = (min_dist - dist) * 0.5;
                let nx = dx / dist;
                let ny = dy / dist;

                if let Some(ta) = ecs.transforms.get_mut(&ea) {
                    ta.x -= nx * overlap;
                    ta.y -= ny * overlap;
                }
                if let Some(tb) = ecs.transforms.get_mut(&eb) {
                    tb.x += nx * overlap;
                    tb.y += ny * overlap;
                }
            }
        }
    }
}

pub fn check_overlap(ax: f32, ay: f32, bx: f32, by: f32, radius: f32) -> bool {
    let dx = bx - ax;
    let dy = by - ay;
    dx * dx + dy * dy < (radius * 2.0).powi(2)
}
