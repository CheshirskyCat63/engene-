const SHIFT_THRESHOLD: f32 = 5000.0;

pub struct OriginShift {
    pub accumulated_offset: [f64; 3],
    pub shift_count: u32,
}

impl OriginShift {
    pub fn new() -> Self {
        Self {
            accumulated_offset: [0.0; 3],
            shift_count: 0,
        }
    }

    pub fn needs_shift(cam_x: f32, cam_z: f32) -> bool {
        let dist = (cam_x * cam_x + cam_z * cam_z).sqrt();
        dist > SHIFT_THRESHOLD
    }

    pub fn compute_shift(cam_x: f32, cam_z: f32) -> (f32, f32) {
        (-cam_x, -cam_z)
    }

    pub fn apply_shift(&mut self, shift_x: f32, shift_z: f32) {
        self.accumulated_offset[0] += shift_x as f64;
        self.accumulated_offset[2] += shift_z as f64;
        self.shift_count += 1;
    }

    pub fn world_to_absolute(&self, local_x: f32, local_z: f32) -> (f64, f64) {
        (
            local_x as f64 - self.accumulated_offset[0],
            local_z as f64 - self.accumulated_offset[2],
        )
    }

    pub fn shift_entities(
        transforms: &mut crate::core::sparse_set::SparseSet<crate::world::components::Transform>,
        alive: &[crate::core::ecs::Entity],
        shift_x: f32,
        shift_z: f32,
    ) {
        for &e in alive {
            if let Some(t) = transforms.get_mut(&e) {
                t.x += shift_x;
                t.y += shift_z;
                t.cell_x = ((t.x / crate::world::cell::CELL_SIZE) as u32)
                    .min(crate::world::cell::GRID_SIZE - 1);
                t.cell_y = ((t.y / crate::world::cell::CELL_SIZE) as u32)
                    .min(crate::world::cell::GRID_SIZE - 1);
            }
        }
    }
}
