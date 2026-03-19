use glam::Vec3;

use crate::physics::damage_taxonomy::DamageClass;
use crate::physics::impact_event::ImpactEvent;
use crate::world::surface_db::MaterialId;

const MAX_FRAGMENTS_PER_EXPLOSION: usize = 16;

pub struct DebrisFragment {
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f32,
    pub material: MaterialId,
    pub energy: f32,
}

pub fn generate_fragments(
    origin: Vec3,
    energy: f32,
    material: MaterialId,
    fragment_coeff: f32,
) -> Vec<DebrisFragment> {
    let count = ((fragment_coeff * 16.0) as usize).min(MAX_FRAGMENTS_PER_EXPLOSION);
    let mut fragments = Vec::with_capacity(count);

    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let elevation = (i as f32 * 0.3).sin() * 0.5 + 0.5;
        let speed = (energy * 0.1).sqrt().min(30.0);

        fragments.push(DebrisFragment {
            position: origin,
            velocity: Vec3::new(
                angle.cos() * speed,
                elevation * speed * 0.5,
                angle.sin() * speed,
            ),
            mass: 0.05 + (i as f32 * 0.02),
            material,
            energy: energy * 0.05 / count.max(1) as f32,
        });
    }
    fragments
}

pub fn fragment_to_impact(frag: &DebrisFragment) -> ImpactEvent {
    ImpactEvent {
        position: frag.position,
        direction: frag.velocity.normalize_or_zero(),
        impulse: frag.mass * frag.velocity.length(),
        energy: frag.energy,
        contact_area: 0.001,
        damage_class: DamageClass::Fragmentation,
        instigator: None,
        material_hit: frag.material,
        target_entity: None,
        projectile_info: None,
    }
}
