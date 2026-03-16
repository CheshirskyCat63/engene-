use crate::body::body_store::BodyStateStore;
use crate::physics::damage_pipeline::response_aggregator::BodyZone;

pub fn check_dismemberment(store: &mut BodyStateStore, store_id: u32, joint_id: u8) -> bool {
    let body = match store.get_mut(store_id) {
        Some(b) => b,
        None => return false,
    };

    let joint = match body.joints.iter_mut().find(|j| j.id == joint_id) {
        Some(j) => j,
        None => return false,
    };

    if joint.broken {
        let zone = joint.zone;
        if let Some(z) = body.zones.iter_mut().find(|z| z.zone == zone) {
            z.is_severed = true;
            z.integrity = 0.0;
        }
        return true;
    }
    false
}

pub fn severed_zones(store: &BodyStateStore, store_id: u32) -> Vec<BodyZone> {
    match store.get(store_id) {
        Some(body) => body
            .zones
            .iter()
            .filter(|z| z.is_severed)
            .map(|z| z.zone)
            .collect(),
        None => Vec::new(),
    }
}
