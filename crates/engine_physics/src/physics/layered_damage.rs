use std::collections::HashMap;

use crate::core::ecs::Entity;
use crate::physics::damage_taxonomy::DamageCapability;
use crate::world::surface_db::MaterialId;

#[derive(Clone, Debug)]
pub struct DamageLayer {
    pub material: MaterialId,
    pub thickness: f32,
    pub integrity: f32,
    pub adhesion: f32,
    pub accumulated_stress: f32,
    pub thermal_damage: f32,
    pub moisture_damage: f32,
}

#[derive(Clone, Debug)]
pub struct DamageableObject {
    pub entity: Entity,
    pub capability: DamageCapability,
    pub layers: Vec<DamageLayer>,
    pub structural_section: Option<u32>,
}

pub struct DamageableStore {
    objects: HashMap<Entity, DamageableObject>,
}

impl DamageableStore {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }

    pub fn register(&mut self, obj: DamageableObject) {
        self.objects.insert(obj.entity, obj);
    }

    pub fn get(&self, entity: Entity) -> Option<&DamageableObject> {
        self.objects.get(&entity)
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut DamageableObject> {
        self.objects.get_mut(&entity)
    }

    pub fn remove(&mut self, entity: Entity) -> Option<DamageableObject> {
        self.objects.remove(&entity)
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn capability_of(&self, entity: Entity) -> DamageCapability {
        self.objects
            .get(&entity)
            .map(|o| o.capability)
            .unwrap_or(DamageCapability::empty())
    }
}
