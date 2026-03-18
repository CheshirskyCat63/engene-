use crate::body::body_store::BodyStateStore;
use crate::physics::damage_pipeline::body_resolver::BodyResolver;
use crate::physics::damage_pipeline::layer_resolver::LayerResolver;
use crate::physics::damage_pipeline::response_aggregator::DamageResponse;
use crate::physics::damage_pipeline::structural_resolver::StructuralResolver;
use crate::physics::damage_pipeline::surface_resolver::SurfaceResolver;
use crate::physics::damage_taxonomy::DamageCapability;
use crate::physics::destruction::DestructionSystem;
use crate::physics::impact_event::{ImpactEvent, StressEvent};
use crate::physics::layered_damage::DamageableStore;
use crate::world::surface_db::SurfaceDB;

const MAX_IMPACTS_PER_FRAME: usize = 64;
const MAX_STRESS_PER_FRAME: usize = 128;

pub struct DamageOrchestrator {
    impact_queue: Vec<ImpactEvent>,
    stress_queue: Vec<StressEvent>,
    pub responses: Vec<DamageResponse>,
}

impl DamageOrchestrator {
    pub fn new() -> Self {
        Self {
            impact_queue: Vec::with_capacity(MAX_IMPACTS_PER_FRAME),
            stress_queue: Vec::with_capacity(MAX_STRESS_PER_FRAME),
            responses: Vec::with_capacity(256),
        }
    }

    pub fn submit_impact(&mut self, event: ImpactEvent) {
        if self.impact_queue.len() < MAX_IMPACTS_PER_FRAME {
            self.impact_queue.push(event);
        }
    }

    pub fn submit_stress(&mut self, event: StressEvent) {
        if self.stress_queue.len() < MAX_STRESS_PER_FRAME {
            self.stress_queue.push(event);
        }
    }

    pub fn resolve_all(
        &mut self,
        surface_db: &SurfaceDB,
        damageable_store: &mut DamageableStore,
        destruction_sys: &mut DestructionSystem,
        body_store: &mut BodyStateStore,
        capability_lookup: &dyn Fn(u64) -> DamageCapability,
    ) {
        self.responses.clear();

        for event in self.impact_queue.drain(..) {
            let caps = event
                .target_entity
                .map(|e| capability_lookup(e))
                .unwrap_or(DamageCapability::SURFACE);

            SurfaceResolver::resolve_impact(&event, surface_db, &mut self.responses);

            if caps.contains(DamageCapability::LAYERED) {
                LayerResolver::resolve_impact(
                    &event,
                    surface_db,
                    damageable_store,
                    &mut self.responses,
                );
            }

            if caps.contains(DamageCapability::STRUCTURAL) {
                StructuralResolver::resolve_impact(&event, destruction_sys, &mut self.responses);
            }

            if caps.intersects(DamageCapability::ANATOMICAL) {
                BodyResolver::resolve_impact(&event, body_store, &mut self.responses);
            }
        }

        for event in self.stress_queue.drain(..) {
            SurfaceResolver::resolve_stress(&event, surface_db, &mut self.responses);

            let caps = capability_lookup(event.target_entity);
            if caps.contains(DamageCapability::LAYERED) {
                LayerResolver::resolve_stress(
                    &event,
                    surface_db,
                    damageable_store,
                    &mut self.responses,
                );
            }
        }
    }

    pub fn drain_responses(&mut self) -> Vec<DamageResponse> {
        std::mem::take(&mut self.responses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::damage_taxonomy::DamageClass;

    #[test]
    fn test_submit_impact() {
        let mut orch = DamageOrchestrator::new();

        let impact = ImpactEvent {
            position: glam::Vec3::ZERO,
            direction: glam::Vec3::Y,
            impulse: 100.0,
            energy: 100.0,
            contact_area: 0.1,
            damage_class: DamageClass::Ballistic,
            material_hit: 0u16,
            instigator: None,
            target_entity: None,
            projectile_info: None,
        };

        orch.submit_impact(impact);
        assert_eq!(orch.impact_queue.len(), 1);
    }

    #[test]
    fn test_submit_stress() {
        let mut orch = DamageOrchestrator::new();

        let stress = StressEvent {
            target_entity: 1,
            damage_class: DamageClass::Blunt,
            intensity: 50.0,
            duration: 1.0,
            position: Some(glam::Vec3::ZERO),
            source_direction: Some(glam::Vec3::Y),
        };

        orch.submit_stress(stress);
        assert_eq!(orch.stress_queue.len(), 1);
    }

    #[test]
    fn test_max_impacts_limit() {
        let mut orch = DamageOrchestrator::new();

        let impact = ImpactEvent {
            position: glam::Vec3::ZERO,
            direction: glam::Vec3::Y,
            impulse: 100.0,
            energy: 100.0,
            contact_area: 0.1,
            damage_class: DamageClass::Ballistic,
            material_hit: 0u16,
            instigator: None,
            target_entity: None,
            projectile_info: None,
        };

        // Submit more than limit
        for _ in 0..100 {
            orch.submit_impact(impact.clone());
        }

        assert_eq!(orch.impact_queue.len(), MAX_IMPACTS_PER_FRAME);
    }

    #[test]
    fn test_drain_responses() {
        let mut orch = DamageOrchestrator::new();

        // Manually add a response for test
        orch.responses.push(DamageResponse::SurfaceMarked {
            position: glam::Vec3::ZERO,
            material: 0u16,
            decal_type: crate::physics::damage_pipeline::response_aggregator::DecalType::BulletHole,
            intensity: 0.5,
        });

        let responses = orch.drain_responses();
        assert_eq!(responses.len(), 1);
        assert!(orch.responses.is_empty());
    }

    #[test]
    fn test_queues_cleared_after_resolve() {
        let mut orch = DamageOrchestrator::new();

        let impact = ImpactEvent {
            position: glam::Vec3::ZERO,
            direction: glam::Vec3::Y,
            impulse: 100.0,
            energy: 100.0,
            contact_area: 0.1,
            damage_class: DamageClass::Ballistic,
            material_hit: 0u16,
            instigator: None,
            target_entity: None,
            projectile_info: None,
        };

        orch.submit_impact(impact);
        assert_eq!(orch.impact_queue.len(), 1);

        // Resolve with empty stores (surface-only damage)
        let surface_db = SurfaceDB::new(Vec::new());
        let mut damageable_store = DamageableStore::new();
        let mut destruction_sys = DestructionSystem::new();
        let mut body_store = BodyStateStore::new();

        orch.resolve_all(
            &surface_db,
            &mut damageable_store,
            &mut destruction_sys,
            &mut body_store,
            &|_| DamageCapability::SURFACE,
        );

        // Queue should be drained
        assert!(orch.impact_queue.is_empty());
    }
}
