use glam::Vec3;

use crate::world::surface_db::{MaterialId, SurfaceDB};

pub struct MassProperties {
    pub mass: f32,
    pub center_of_mass: Vec3,
    pub inertia: Vec3,
}

pub fn compute_mass_from_material(
    surface_db: &SurfaceDB,
    material: MaterialId,
    volume: f32,
) -> MassProperties {
    let density = surface_db
        .get(material)
        .map(|m| m.density)
        .unwrap_or(1000.0);

    let mass = density * volume;
    let side = volume.cbrt();
    let i = mass * side * side / 6.0;

    MassProperties {
        mass,
        center_of_mass: Vec3::ZERO,
        inertia: Vec3::splat(i),
    }
}

pub fn compute_mass_box(
    surface_db: &SurfaceDB,
    material: MaterialId,
    half_extents: Vec3,
) -> MassProperties {
    let volume = half_extents.x * half_extents.y * half_extents.z * 8.0;
    let density = surface_db
        .get(material)
        .map(|m| m.density)
        .unwrap_or(1000.0);
    let mass = density * volume;

    let ix = mass * (half_extents.y * half_extents.y + half_extents.z * half_extents.z) / 3.0;
    let iy = mass * (half_extents.x * half_extents.x + half_extents.z * half_extents.z) / 3.0;
    let iz = mass * (half_extents.x * half_extents.x + half_extents.y * half_extents.y) / 3.0;

    MassProperties {
        mass,
        center_of_mass: Vec3::ZERO,
        inertia: Vec3::new(ix, iy, iz),
    }
}
