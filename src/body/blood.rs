use glam::Vec3;
use crate::physics::damage_pipeline::response_aggregator::BodyZone;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BloodLod {
    Full,
    Reduced,
    StateOnly,
}

#[derive(Clone, Debug)]
pub struct BloodEmitter {
    pub position: Vec3,
    pub zone: BodyZone,
    pub rate: f32,
    pub arterial: bool,
    pub lod: BloodLod,
}

pub fn compute_blood_lod(distance_sq: f32) -> BloodLod {
    if distance_sq < 200.0 * 200.0 {
        BloodLod::Full
    } else if distance_sq < 1000.0 * 1000.0 {
        BloodLod::Reduced
    } else {
        BloodLod::StateOnly
    }
}
