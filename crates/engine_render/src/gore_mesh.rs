use glam::Vec3;

use crate::core::ecs::Entity;
use crate::physics::damage_pipeline::response_aggregator::BodyZone;

#[derive(Clone, Debug)]
pub struct GoreMeshInstance {
    pub entity: Entity,
    pub zone: BodyZone,
    pub position: Vec3,
    pub scale: f32,
    pub blood_intensity: f32,
}

pub struct GoreMeshSystem {
    instances: Vec<GoreMeshInstance>,
    max_instances: usize,
}

impl GoreMeshSystem {
    pub fn new(max_instances: usize) -> Self {
        Self {
            instances: Vec::with_capacity(max_instances),
            max_instances,
        }
    }

    pub fn add_gore(&mut self, instance: GoreMeshInstance) {
        if self.instances.len() >= self.max_instances {
            self.instances.remove(0);
        }
        self.instances.push(instance);
    }

    pub fn active_count(&self) -> usize {
        self.instances.len()
    }

    pub fn instances(&self) -> &[GoreMeshInstance] {
        &self.instances
    }
}
