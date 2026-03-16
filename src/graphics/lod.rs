use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LodLevel {
    Full,
    Medium,
    Low,
    Billboard,
    Culled,
}

pub struct LodConfig {
    pub full_distance: f32,
    pub medium_distance: f32,
    pub low_distance: f32,
    pub billboard_distance: f32,
    pub cull_distance: f32,
    pub animation_distance: f32,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            full_distance: 50.0,
            medium_distance: 150.0,
            low_distance: 300.0,
            billboard_distance: 600.0,
            cull_distance: 1200.0,
            animation_distance: 200.0,
        }
    }
}

impl LodConfig {
    pub fn compute_lod(&self, distance: f32) -> LodLevel {
        if distance < self.full_distance {
            LodLevel::Full
        } else if distance < self.medium_distance {
            LodLevel::Medium
        } else if distance < self.low_distance {
            LodLevel::Low
        } else if distance < self.billboard_distance {
            LodLevel::Billboard
        } else {
            LodLevel::Culled
        }
    }

    pub fn should_animate(&self, distance: f32) -> bool {
        distance < self.animation_distance
    }
}

pub struct EntityLod {
    pub entity_id: u64,
    pub lod: LodLevel,
    pub distance: f32,
    pub animate: bool,
}

pub fn compute_entity_lods(
    positions: &[(u64, Vec3)],
    camera_pos: Vec3,
    config: &LodConfig,
) -> Vec<EntityLod> {
    positions
        .iter()
        .filter_map(|&(id, pos)| {
            let dist = (pos - camera_pos).length();
            let lod = config.compute_lod(dist);
            if lod == LodLevel::Culled {
                return None;
            }
            Some(EntityLod {
                entity_id: id,
                lod,
                distance: dist,
                animate: config.should_animate(dist),
            })
        })
        .collect()
}
