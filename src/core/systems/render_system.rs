//! Render system - collects entity instances for rendering.

use crate::core::ecs::Ecs;
use crate::graphics::camera::FlyCamera;
use crate::graphics::lod::{LodConfig, LodLevel};
use crate::graphics::mesh::EntityInstance;
use crate::graphics::visibility::Frustum;
use crate::world::components::{EntityKind, MonsterSpecies};
use crate::world::heightmap::Heightmap;
use glam::Vec3;

/// Collects entity instances for GPU instanced rendering.
pub struct RenderSystem {
    lod_config: LodConfig,
}

impl Default for RenderSystem {
    fn default() -> Self {
        Self {
            lod_config: LodConfig::default(),
        }
    }
}

impl RenderSystem {
    /// Create new render system.
    pub fn new() -> Self {
        Self::default()
    }

    /// Collect visible entity instances.
    pub fn collect_instances(
        &self,
        ecs: &Ecs,
        heightmap: &Heightmap,
        camera: &FlyCamera,
    ) -> Vec<EntityInstance> {
        let cam_pos = camera.position;
        let vp = camera.view_projection();
        let frustum = Frustum::from_view_projection(&vp);
        
        let mut instances = Vec::with_capacity(ecs.alive.len());
        
        for &e in &ecs.alive {
            let t = match ecs.get_transform(e) {
                Some(t) => t,
                None => continue,
            };

            let y = heightmap.sample(t.x, t.y) + 1.0;
            let pos = Vec3::new(t.x, y, t.y);
            let dist = (pos - cam_pos).length();

            // LOD culling
            if self.lod_config.compute_lod(dist) == LodLevel::Culled {
                continue;
            }

            // Frustum culling
            if !frustum.test_sphere(pos, 2.0) {
                continue;
            }

            // Entity color based on kind
            let color = match ecs.get_kind(e) {
                Some(EntityKind::Npc) => [0.16, 0.47, 1.0],
                Some(EntityKind::Monster(MonsterSpecies::Wolf)) => [0.9, 0.9, 0.9],
                Some(EntityKind::Monster(MonsterSpecies::Boar)) => [0.55, 0.43, 0.39],
                Some(EntityKind::Monster(MonsterSpecies::Bloodsucker)) => [0.83, 0.0, 0.0],
                None => [0.5, 0.5, 0.5],
            };

            instances.push(EntityInstance {
                position: [t.x, y, t.y],
                color,
            });
        }

        instances
    }
}

/// Helper functions for entity instance collection.
pub struct EntityInstanceCollector;

impl EntityInstanceCollector {
    /// Collect instances with custom LOD config.
    pub fn collect_with_lod(
        ecs: &Ecs,
        heightmap: &Heightmap,
        camera_pos: Vec3,
        frustum: &Frustum,
        lod_config: &LodConfig,
    ) -> Vec<EntityInstance> {
        let mut instances = Vec::with_capacity(ecs.alive.len());

        for &e in &ecs.alive {
            let t = match ecs.get_transform(e) {
                Some(t) => t,
                None => continue,
            };

            let y = heightmap.sample(t.x, t.y) + 1.0;
            let pos = Vec3::new(t.x, y, t.y);
            let dist = (pos - camera_pos).length();

            if lod_config.compute_lod(dist) == LodLevel::Culled {
                continue;
            }

            if !frustum.test_sphere(pos, 2.0) {
                continue;
            }

            let color = match ecs.get_kind(e) {
                Some(EntityKind::Npc) => [0.16, 0.47, 1.0],
                Some(EntityKind::Monster(MonsterSpecies::Wolf)) => [0.9, 0.9, 0.9],
                Some(EntityKind::Monster(MonsterSpecies::Boar)) => [0.55, 0.43, 0.39],
                Some(EntityKind::Monster(MonsterSpecies::Bloodsucker)) => [0.83, 0.0, 0.0],
                None => [0.5, 0.5, 0.5],
            };

            instances.push(EntityInstance {
                position: [t.x, y, t.y],
                color,
            });
        }

        instances
    }
}
