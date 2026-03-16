use glam::Vec3;
use crate::core::ecs::Entity;

#[derive(Clone, Debug)]
pub struct DeformationRegion {
    pub entity: Entity,
    pub local_offset: Vec3,
    pub radius: f32,
    pub depth: f32,
    pub age: f32,
}

pub struct DeformationShaderSystem {
    regions: Vec<DeformationRegion>,
    max_regions: usize,
}

impl DeformationShaderSystem {
    pub fn new(max_regions: usize) -> Self {
        Self {
            regions: Vec::with_capacity(max_regions),
            max_regions,
        }
    }

    pub fn add_deformation(&mut self, region: DeformationRegion) {
        if self.regions.len() >= self.max_regions {
            self.regions.remove(0);
        }
        self.regions.push(region);
    }

    pub fn update(&mut self, dt: f32) {
        for r in &mut self.regions {
            r.age += dt;
        }
    }

    pub fn active_regions(&self) -> &[DeformationRegion] {
        &self.regions
    }
}
