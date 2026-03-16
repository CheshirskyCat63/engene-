use crate::body::body_store::BodyStateStore;
use crate::physics::damage_pipeline::response_aggregator::BodyZone;

pub fn apply_zone_damage(store: &mut BodyStateStore, store_id: u32, zone: BodyZone, damage: f32) {
    let body = match store.get_mut(store_id) {
        Some(b) => b,
        None => return,
    };

    let zone_idx = zone as usize;
    if zone_idx >= body.zones.len() {
        return;
    }

    body.zones[zone_idx].integrity = (body.zones[zone_idx].integrity - damage).max(0.0);
    body.zones[zone_idx].trauma += damage * 0.5;
    body.total_trauma += damage * 0.3;
    body.consciousness = (body.consciousness - damage * 0.005).max(0.0);
    body.pain = (body.pain + damage * 0.01).min(1.0);
}
