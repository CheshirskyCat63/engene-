use glam::Vec2;
use crate::core::ecs::{Ecs, Entity};

#[derive(Clone, Debug)]
pub struct CoverPoint {
    pub position: Vec2,
    pub quality: f32,
    pub direction: Vec2,
}

pub fn evaluate_cover_at(
    position: Vec2,
    threat_dir: Vec2,
    terrain_height_fn: &dyn Fn(f32, f32) -> f32,
) -> f32 {
    let sample_dist = 3.0;
    let behind = position - threat_dir.normalize() * sample_dist;
    let here = terrain_height_fn(position.x, position.y);
    let behind_h = terrain_height_fn(behind.x, behind.y);

    let height_diff = behind_h - here;
    (height_diff * 2.0).clamp(0.0, 1.0)
}

pub fn find_nearby_cover(
    ecs: &Ecs,
    entity: Entity,
    threat_pos: Vec2,
    search_radius: f32,
    terrain_height_fn: &dyn Fn(f32, f32) -> f32,
) -> Vec<CoverPoint> {
let my_pos = match ecs.get_transform(entity) {
 Some(t) => Vec2::new(t.x, t.y),
 None => return Vec::new(),
};

    let threat_dir = (threat_pos - my_pos).normalize_or_zero();
    let mut points = Vec::new();
    let step = 10.0;
    let steps = (search_radius / step) as i32;

    for dx in -steps..=steps {
        for dy in -steps..=steps {
            let candidate = my_pos + Vec2::new(dx as f32 * step, dy as f32 * step);
            let dist = (candidate - my_pos).length();
            if dist > search_radius || dist < 5.0 { continue; }

            let quality = evaluate_cover_at(candidate, threat_dir, terrain_height_fn);
            if quality > 0.2 {
                points.push(CoverPoint {
                    position: candidate,
                    quality,
                    direction: -threat_dir,
                });
            }
        }
    }

    points.sort_by(|a, b| b.quality.partial_cmp(&a.quality).unwrap_or(std::cmp::Ordering::Equal));
    points.truncate(5);
    points
}
