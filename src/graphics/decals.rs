use glam::Vec3;

use crate::physics::damage_pipeline::response_aggregator::DecalType;
use crate::world::surface_db::MaterialId;

#[derive(Clone, Debug)]
pub struct Decal {
    pub position: Vec3,
    pub normal: Vec3,
    pub size: f32,
    pub decal_type: DecalType,
    pub material: MaterialId,
    pub intensity: f32,
    pub age: f32,
    pub lifetime: f32,
}

pub struct DecalSystem {
    active_decals: Vec<Decal>,
    max_decals: usize,
}

impl DecalSystem {
    pub fn new(max_decals: usize) -> Self {
        Self {
            active_decals: Vec::with_capacity(max_decals),
            max_decals,
        }
    }

    pub fn add_decal(&mut self, decal: Decal) {
        if self.active_decals.len() >= self.max_decals {
            self.active_decals.remove(0);
        }
        self.active_decals.push(decal);
    }

    pub fn update(&mut self, dt: f32) {
        for d in &mut self.active_decals {
            d.age += dt;
        }
        self.active_decals.retain(|d| d.age < d.lifetime);
    }

    pub fn visible_decals(&self) -> &[Decal] {
        &self.active_decals
    }

    pub fn len(&self) -> usize {
        self.active_decals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.active_decals.is_empty()
    }
}
