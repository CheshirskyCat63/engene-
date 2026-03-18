use crate::physics::damage_pipeline::response_aggregator::BodyZone;

#[derive(Clone, Debug)]
pub struct ZoneState {
    pub zone: BodyZone,
    pub integrity: f32,
    pub trauma: f32,
    pub is_severed: bool,
}

impl ZoneState {
    pub fn new(zone: BodyZone) -> Self {
        Self {
            zone,
            integrity: 100.0,
            trauma: 0.0,
            is_severed: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct JointInfo {
    pub id: u8,
    pub zone: BodyZone,
    pub integrity: f32,
    pub broken: bool,
    pub dismember_threshold: f32,
}

#[derive(Clone, Debug)]
pub struct BleedPoint {
    pub zone: BodyZone,
    pub rate: f32,
    pub time_active: f32,
}

#[derive(Clone, Debug)]
pub struct BodyState {
    pub entity: u64,
    pub base_height: f32,
    pub zones: Vec<ZoneState>,
    pub joints: Vec<JointInfo>,
    pub blood_level: f32,
    pub total_trauma: f32,
    pub consciousness: f32,
    pub pain: f32,
    pub bleed_points: Vec<BleedPoint>,
}

impl BodyState {
    pub fn new_humanoid(entity: u64) -> Self {
        Self {
            entity,
            base_height: 0.0,
            zones: vec![
                ZoneState::new(BodyZone::Head),
                ZoneState::new(BodyZone::Neck),
                ZoneState::new(BodyZone::Torso),
                ZoneState::new(BodyZone::LeftArm),
                ZoneState::new(BodyZone::RightArm),
                ZoneState::new(BodyZone::Pelvis),
                ZoneState::new(BodyZone::LeftLeg),
                ZoneState::new(BodyZone::RightLeg),
            ],
            joints: vec![
                JointInfo {
                    id: 0,
                    zone: BodyZone::Neck,
                    integrity: 100.0,
                    broken: false,
                    dismember_threshold: 15.0,
                },
                JointInfo {
                    id: 1,
                    zone: BodyZone::LeftArm,
                    integrity: 100.0,
                    broken: false,
                    dismember_threshold: 20.0,
                },
                JointInfo {
                    id: 2,
                    zone: BodyZone::RightArm,
                    integrity: 100.0,
                    broken: false,
                    dismember_threshold: 20.0,
                },
                JointInfo {
                    id: 3,
                    zone: BodyZone::LeftLeg,
                    integrity: 100.0,
                    broken: false,
                    dismember_threshold: 25.0,
                },
                JointInfo {
                    id: 4,
                    zone: BodyZone::RightLeg,
                    integrity: 100.0,
                    broken: false,
                    dismember_threshold: 25.0,
                },
                JointInfo {
                    id: 5,
                    zone: BodyZone::Pelvis,
                    integrity: 100.0,
                    broken: false,
                    dismember_threshold: 30.0,
                },
            ],
            blood_level: 1.0,
            total_trauma: 0.0,
            consciousness: 1.0,
            pain: 0.0,
            bleed_points: Vec::new(),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.blood_level > 0.0 && self.zones[0].integrity > 0.0
    }

    pub fn aggregate_health(&self) -> f32 {
        let zone_avg: f32 =
            self.zones.iter().map(|z| z.integrity).sum::<f32>() / self.zones.len() as f32;
        (zone_avg * self.blood_level * self.consciousness).max(0.0)
    }
}
