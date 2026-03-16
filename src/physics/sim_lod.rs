use crate::world::components::SimulationLevel;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PhysicsLod {
    Full,
    Simplified,
    Statistical,
    Paused,
}

impl PhysicsLod {
    pub fn from_sim_level(level: SimulationLevel) -> Self {
        match level {
            SimulationLevel::L0 => Self::Full,
            SimulationLevel::L1 => Self::Simplified,
            SimulationLevel::L2 => Self::Statistical,
            SimulationLevel::L3 => Self::Paused,
        }
    }

    pub fn cloth_iterations(self) -> u32 {
        match self {
            Self::Full => 10,
            Self::Simplified => 3,
            Self::Statistical | Self::Paused => 0,
        }
    }

    pub fn water_active(self) -> bool {
        matches!(self, Self::Full | Self::Simplified)
    }

    pub fn fire_tick_interval(self) -> u64 {
        match self {
            Self::Full => 1,
            Self::Simplified => 4,
            Self::Statistical => 16,
            Self::Paused => u64::MAX,
        }
    }

    pub fn should_tick_fire(self, tick: u64) -> bool {
        let interval = self.fire_tick_interval();
        if interval == u64::MAX {
            return false;
        }
        tick % interval == 0
    }
}

pub fn region_sim_level(
    cam_x: f32,
    cam_z: f32,
    region_x: f32,
    region_z: f32,
) -> SimulationLevel {
    let dx = cam_x - region_x;
    let dz = cam_z - region_z;
    let dist = (dx * dx + dz * dz).sqrt();

    if dist < 300.0 {
        SimulationLevel::L0
    } else if dist < 5000.0 {
        SimulationLevel::L1
    } else if dist < 50000.0 {
        SimulationLevel::L2
    } else {
        SimulationLevel::L3
    }
}
