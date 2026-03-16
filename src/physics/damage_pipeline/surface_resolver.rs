use crate::physics::damage_pipeline::response_aggregator::{DamageResponse, DecalType, SoundClass};
use crate::physics::impact_event::{ImpactEvent, StressEvent};
use crate::physics::damage_taxonomy::DamageClass;
use crate::world::surface_db::SurfaceDB;

pub struct SurfaceResolver;

impl SurfaceResolver {
    pub fn resolve_impact(
        event: &ImpactEvent,
        surface_db: &SurfaceDB,
        responses: &mut Vec<DamageResponse>,
    ) {
        let mat = match surface_db.get(event.material_hit) {
            Some(m) => m,
            None => return,
        };

        let stress = event.energy / event.contact_area.max(0.001);
        let intensity = (stress / mat.compressive_strength.max(1.0)).min(1.0);

        let decal_type = match event.damage_class {
            DamageClass::Ballistic | DamageClass::Piercing => DecalType::BulletHole,
            DamageClass::Explosive | DamageClass::Fragmentation => DecalType::Crack,
            DamageClass::Blunt => DecalType::Dent,
            DamageClass::Shear => DecalType::Crack,
            DamageClass::Thermal => DecalType::Scorch,
            _ => DecalType::Dent,
        };

        responses.push(DamageResponse::SurfaceMarked {
            position: event.position,
            material: event.material_hit,
            decal_type,
            intensity,
        });

        let sound_class = if mat.is_biological {
            SoundClass::FleshHit
        } else if mat.hardness > 6.0 {
            SoundClass::ConcreteBullet
        } else if mat.hardness > 3.0 {
            SoundClass::WoodBullet
        } else {
            SoundClass::DirtImpact
        };

        responses.push(DamageResponse::AudioTrigger {
            position: event.position,
            sound_class,
            intensity,
        });
    }

    pub fn resolve_stress(
        _event: &StressEvent,
        _surface_db: &SurfaceDB,
        _responses: &mut Vec<DamageResponse>,
    ) {
        // Cumulative stress only marks surfaces if intensity is high enough
        // Thermal -> scorch marks, Hydraulic -> water stains, etc.
        // Handled via SurfaceStateUpdated in Phase 15 integration
    }
}
