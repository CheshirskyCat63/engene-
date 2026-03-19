use glam::{Quat, Vec3};

pub struct FootIkSolver {
    pub left_target: Vec3,
    pub right_target: Vec3,
    pub hip_offset: f32,
    pub blend: f32,
}

impl FootIkSolver {
    pub fn new() -> Self {
        Self {
            left_target: Vec3::ZERO,
            right_target: Vec3::ZERO,
            hip_offset: 0.0,
            blend: 1.0,
        }
    }

    pub fn update(
        &mut self,
        root_pos: Vec3,
        left_foot_pos: Vec3,
        right_foot_pos: Vec3,
        terrain_sample: impl Fn(f32, f32) -> f32,
    ) {
        let left_ground = terrain_sample(left_foot_pos.x, left_foot_pos.z);
        let right_ground = terrain_sample(right_foot_pos.x, right_foot_pos.z);

        self.left_target = Vec3::new(left_foot_pos.x, left_ground, left_foot_pos.z);
        self.right_target = Vec3::new(right_foot_pos.x, right_ground, right_foot_pos.z);

        let lowest = left_ground.min(right_ground);
        let root_ground = terrain_sample(root_pos.x, root_pos.z);
        self.hip_offset = (lowest - root_ground).min(0.0);
    }

    pub fn solve_two_bone(
        root: Vec3,
        mid: Vec3,
        end: Vec3,
        target: Vec3,
        pole: Vec3,
    ) -> (Quat, Quat) {
        let a = (mid - root).length();
        let b = (end - mid).length();
        let c = (target - root).length().min(a + b - 0.001);

        let cos_angle_a = ((a * a + c * c - b * b) / (2.0 * a * c)).clamp(-1.0, 1.0);
        let cos_angle_b = ((a * a + b * b - c * c) / (2.0 * a * b)).clamp(-1.0, 1.0);

        let target_dir = (target - root).normalize_or_zero();
        let pole_dir = (pole - root).normalize_or_zero();

        let initial_dir = (mid - root).normalize_or_zero();
        let root_rot = Quat::from_rotation_arc(initial_dir, target_dir);

        let bend_axis = target_dir.cross(pole_dir).normalize_or_zero();
        let bend_angle = cos_angle_a.acos();
        let root_bend = Quat::from_axis_angle(bend_axis, bend_angle);

        let mid_angle = std::f32::consts::PI - cos_angle_b.acos();
        let mid_rot = Quat::from_axis_angle(bend_axis, -mid_angle);

        (root_rot * root_bend, mid_rot)
    }
}
