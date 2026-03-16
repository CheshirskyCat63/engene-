use crate::physics::damage_pipeline::response_aggregator::{BodyZone, DamageResponse};
use crate::physics::impact_event::ImpactEvent;
use crate::body::body_store::BodyStateStore;

pub struct BodyResolver;

impl BodyResolver {
    pub fn resolve_impact(
        event: &ImpactEvent,
        body_store: &mut BodyStateStore,
        responses: &mut Vec<DamageResponse>,
    ) {
        let entity = match event.target_entity {
            Some(e) => e,
            None => return,
        };

        let body = match body_store.get_mut_by_entity(entity) {
            Some(b) => b,
            None => return,
        };

        let zone = Self::position_to_zone(event.position.y - body.base_height);
        let zone_idx = zone as usize;
        if zone_idx >= body.zones.len() {
            return;
        }

        let raw_damage = event.energy * 0.01;
        let zone_state = &mut body.zones[zone_idx];
        zone_state.integrity -= raw_damage;
        zone_state.trauma += raw_damage * 0.5;

        body.total_trauma += raw_damage * 0.3;
        body.consciousness = (body.consciousness - raw_damage * 0.1).max(0.0);
        body.pain = (body.pain + raw_damage * 0.2).min(1.0);

        responses.push(DamageResponse::BodyZoneDamaged {
            entity,
            zone,
            damage: raw_damage,
            penetrated_layers: 1,
        });

        if zone_state.integrity <= 0.0 {
            zone_state.integrity = 0.0;
        }

        let near_joint = Self::is_near_joint(zone);
        if near_joint && raw_damage > 5.0 {
            for joint in body.joints.iter_mut() {
                if joint.zone == zone && !joint.broken {
                    joint.integrity -= raw_damage * 1.5;
                    if joint.integrity <= 0.0 {
                        joint.integrity = 0.0;
                        joint.broken = true;
                        responses.push(DamageResponse::JointBroken {
                            entity,
                            joint_id: joint.id,
                        });
                    }
                }
            }
        }

        if raw_damage > 2.0 && zone_state.integrity < 0.5 {
            let bleed_rate = raw_damage * 0.05;
            body.bleed_points.push(crate::body::anatomy::BleedPoint {
                zone,
                rate: bleed_rate,
                time_active: 0.0,
            });
            responses.push(DamageResponse::BleedStarted {
                entity,
                zone,
                rate: bleed_rate,
            });
        }
    }

    fn position_to_zone(relative_y: f32) -> BodyZone {
        if relative_y > 1.6 {
            BodyZone::Head
        } else if relative_y > 1.4 {
            BodyZone::Neck
        } else if relative_y > 0.9 {
            BodyZone::Torso
        } else if relative_y > 0.7 {
            BodyZone::Pelvis
        } else {
            BodyZone::LeftLeg
        }
    }

    fn is_near_joint(zone: BodyZone) -> bool {
        matches!(
            zone,
            BodyZone::Neck
                | BodyZone::LeftArm
                | BodyZone::RightArm
                | BodyZone::LeftLeg
                | BodyZone::RightLeg
                | BodyZone::Pelvis
        )
    }
}
