use crate::physics::damage_pipeline::response_aggregator::DamageResponse;
use crate::physics::destruction::DestructionSystem;
use crate::physics::impact_event::ImpactEvent;

pub struct StructuralResolver;

impl StructuralResolver {
    pub fn resolve_impact(
        event: &ImpactEvent,
        destruction: &mut DestructionSystem,
        responses: &mut Vec<DamageResponse>,
    ) {
        let entity = match event.target_entity {
            Some(e) => e,
            None => return,
        };

        let section_id = destruction.find_section_at(entity, event.position);
        if let Some(sid) = section_id {
            let collapsed = destruction.apply_section_damage(entity, sid, event.energy);

            responses.push(DamageResponse::StructuralDamage {
                entity,
                section_id: sid,
                energy: event.energy,
            });

            if collapsed {
                responses.push(DamageResponse::CollapseTriggered {
                    entity,
                    section_id: sid,
                });
                responses.push(DamageResponse::WorldTopologyChanged {
                    position: event.position,
                    radius: 10.0,
                });
            }
        }
    }
}
