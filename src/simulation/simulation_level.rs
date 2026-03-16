use crate::world::components::SimulationLevel;

pub const L0_RADIUS: f32 = 300.0;
pub const L1_RADIUS: f32 = 5000.0;
pub const L2_RADIUS: f32 = 50000.0;

pub fn level_for_distance(distance: f32) -> SimulationLevel {
    if distance <= L0_RADIUS {
        SimulationLevel::L0
    } else if distance <= L1_RADIUS {
        SimulationLevel::L1
    } else if distance <= L2_RADIUS {
        SimulationLevel::L2
    } else {
        SimulationLevel::L3
    }
}

pub const L0_TICK_INTERVAL: u64 = 1;
pub const L1_TICK_INTERVAL: u64 = 12;
pub const L2_TICK_INTERVAL: u64 = 60;

pub fn should_tick(level: SimulationLevel, frame: u64) -> bool {
    match level {
        SimulationLevel::L0 => true,
        SimulationLevel::L1 => frame % L1_TICK_INTERVAL == 0,
        SimulationLevel::L2 => frame % L2_TICK_INTERVAL == 0,
        SimulationLevel::L3 => false, // strategic level ticks on events only
    }
}

// =============================================================================
// Physics LOD (Phase C.4)
// =============================================================================

/// Fire simulation LOD
pub fn should_tick_fire(level: SimulationLevel, frame: u64) -> bool {
    match level {
        SimulationLevel::L0 => true,
        SimulationLevel::L1 => frame % 4 == 0, // Every 4th frame
        SimulationLevel::L2 => frame % 12 == 0, // Every 12th frame
        SimulationLevel::L3 => false,
    }
}

/// Water simulation LOD
pub fn should_tick_water(level: SimulationLevel, frame: u64) -> bool {
    match level {
        SimulationLevel::L0 => true,
        SimulationLevel::L1 => frame % 6 == 0,
        SimulationLevel::L2 => frame % 24 == 0,
        SimulationLevel::L3 => false,
    }
}

/// Cloth simulation LOD
pub fn should_tick_cloth(level: SimulationLevel) -> bool {
    matches!(level, SimulationLevel::L0)
}

/// Number of cloth solver iterations based on LOD
pub fn cloth_iterations(level: SimulationLevel) -> usize {
    match level {
        SimulationLevel::L0 => 5,
        SimulationLevel::L1 => 2,
        SimulationLevel::L2 => 1,
        SimulationLevel::L3 => 0,
    }
}

/// Ragdoll simulation LOD
pub fn should_tick_ragdoll(level: SimulationLevel) -> bool {
    matches!(level, SimulationLevel::L0 | SimulationLevel::L1)
}

/// Number of ragdoll solver iterations
pub fn ragdoll_iterations(level: SimulationLevel) -> usize {
    match level {
        SimulationLevel::L0 => 4,
        SimulationLevel::L1 => 2,
        _ => 0,
    }
}

/// Debris simulation LOD
pub fn should_tick_debris(level: SimulationLevel, frame: u64) -> bool {
    match level {
        SimulationLevel::L0 => true,
        SimulationLevel::L1 => frame % 2 == 0,
        SimulationLevel::L2 => frame % 6 == 0,
        SimulationLevel::L3 => false,
    }
}

/// Maximum active fire cells based on LOD
pub fn max_fire_cells(level: SimulationLevel) -> usize {
    match level {
        SimulationLevel::L0 => 1000,
        SimulationLevel::L1 => 200,
        SimulationLevel::L2 => 50,
        SimulationLevel::L3 => 0,
    }
}

/// Maximum active water cells
pub fn max_water_cells(level: SimulationLevel) -> usize {
    match level {
        SimulationLevel::L0 => 500,
        SimulationLevel::L1 => 100,
        SimulationLevel::L2 => 20,
        SimulationLevel::L3 => 0,
    }
}

/// Maximum cloth particles
pub fn max_cloth_particles(level: SimulationLevel) -> usize {
    match level {
        SimulationLevel::L0 => 500,
        SimulationLevel::L1 => 100,
        _ => 0,
    }
}

/// Physics bubble radius for Rapier (only L0 entities get full physics)
pub const PHYSICS_BUBBLE_RADIUS: f32 = L0_RADIUS;

/// Check if entity should be in Rapier simulation
pub fn in_physics_bubble(distance: f32) -> bool {
    distance <= PHYSICS_BUBBLE_RADIUS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_for_distance() {
        assert_eq!(level_for_distance(100.0), SimulationLevel::L0);
        assert_eq!(level_for_distance(1000.0), SimulationLevel::L1);
        assert_eq!(level_for_distance(10000.0), SimulationLevel::L2);
        assert_eq!(level_for_distance(100000.0), SimulationLevel::L3);
    }

    #[test]
    fn test_should_tick_l0() {
        assert!(should_tick(SimulationLevel::L0, 0));
        assert!(should_tick(SimulationLevel::L0, 1));
        assert!(should_tick(SimulationLevel::L0, 100));
    }

    #[test]
    fn test_should_tick_l1() {
        assert!(should_tick(SimulationLevel::L1, 0));
        assert!(!should_tick(SimulationLevel::L1, 1));
        assert!(should_tick(SimulationLevel::L1, L1_TICK_INTERVAL));
    }

    #[test]
    fn test_fire_lod() {
        assert!(should_tick_fire(SimulationLevel::L0, 1));
        assert!(!should_tick_fire(SimulationLevel::L1, 1));
        assert!(should_tick_fire(SimulationLevel::L1, 4));
    }

    #[test]
    fn test_cloth_lod() {
        assert!(should_tick_cloth(SimulationLevel::L0));
        assert!(!should_tick_cloth(SimulationLevel::L1));
        assert_eq!(cloth_iterations(SimulationLevel::L0), 5);
        assert_eq!(cloth_iterations(SimulationLevel::L1), 2);
    }

    #[test]
    fn test_physics_bubble() {
        assert!(in_physics_bubble(100.0));
        assert!(in_physics_bubble(300.0));
        assert!(!in_physics_bubble(400.0));
    }

    #[test]
    fn test_max_fire_cells() {
        assert_eq!(max_fire_cells(SimulationLevel::L0), 1000);
        assert_eq!(max_fire_cells(SimulationLevel::L3), 0);
    }
}
