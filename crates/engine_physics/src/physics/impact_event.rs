use glam::Vec3;

use crate::core::ecs::Entity;
use crate::physics::damage_taxonomy::DamageClass;
use crate::world::surface_db::MaterialId;

#[derive(Clone, Debug)]
pub struct ProjectileInfo {
    pub caliber: f32,
    pub velocity: Vec3,
    pub mass: f32,
    pub fragmentation: bool,
}

#[derive(Clone, Debug)]
pub struct ImpactEvent {
    pub position: Vec3,
    pub direction: Vec3,
    pub impulse: f32,
    pub energy: f32,
    pub contact_area: f32,
    pub damage_class: DamageClass,
    pub instigator: Option<Entity>,
    pub material_hit: MaterialId,
    pub target_entity: Option<Entity>,
    pub projectile_info: Option<ProjectileInfo>,
}

#[derive(Clone, Debug)]
pub struct StressEvent {
    pub target_entity: Entity,
    pub damage_class: DamageClass,
    pub intensity: f32,
    pub duration: f32,
    pub position: Option<Vec3>,
    pub source_direction: Option<Vec3>,
}
