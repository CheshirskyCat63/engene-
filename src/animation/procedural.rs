use glam::Mat4;

pub struct PerJointWeight {
    pub weights: Vec<f32>,
}

impl PerJointWeight {
    pub fn uniform(joint_count: usize, w: f32) -> Self {
        Self {
            weights: vec![w; joint_count],
        }
    }

    pub fn from_name_fn(joint_count: usize, names: &[String], f: impl Fn(&str) -> f32) -> Self {
        let mut weights = vec![0.5; joint_count];
        for (i, name) in names.iter().enumerate() {
            weights[i] = f(name);
        }
        Self { weights }
    }
}

pub struct PoseBlender {
    pub joint_weights: PerJointWeight,
    pub global_ragdoll_weight: f32,
}

impl PoseBlender {
    pub fn new(joint_count: usize) -> Self {
        Self {
            joint_weights: PerJointWeight::uniform(joint_count, 0.0),
            global_ragdoll_weight: 0.0,
        }
    }

    pub fn set_global_weight(&mut self, w: f32) {
        self.global_ragdoll_weight = w.clamp(0.0, 1.0);
    }

    pub fn set_joint_weight(&mut self, joint_index: usize, w: f32) {
        if joint_index < self.joint_weights.weights.len() {
            self.joint_weights.weights[joint_index] = w.clamp(0.0, 1.0);
        }
    }

    pub fn blend_poses(
        &self,
        animation_pose: &[Mat4],
        physics_pose: &[Mat4],
    ) -> Vec<Mat4> {
        let n = animation_pose.len().min(physics_pose.len());
        let mut result = Vec::with_capacity(n);

        for i in 0..n {
            let per_joint_w = if i < self.joint_weights.weights.len() {
                self.joint_weights.weights[i]
            } else {
                0.0
            };

            let w = (per_joint_w * self.global_ragdoll_weight).clamp(0.0, 1.0);

            if w < 0.001 {
                result.push(animation_pose[i]);
            } else if w > 0.999 {
                result.push(physics_pose[i]);
            } else {
                result.push(lerp_transform(animation_pose[i], physics_pose[i], w));
            }
        }

        for i in n..animation_pose.len() {
            result.push(animation_pose[i]);
        }

        result
    }
}

fn lerp_transform(a: Mat4, b: Mat4, t: f32) -> Mat4 {
    let (sa, ra, ta) = a.to_scale_rotation_translation();
    let (sb, rb, tb) = b.to_scale_rotation_translation();

    let pos = ta.lerp(tb, t);
    let rot = ra.slerp(rb, t);
    let scale = sa.lerp(sb, t);

    Mat4::from_scale_rotation_translation(scale, rot, pos)
}

pub struct ProceduralLayer {
    pub blender: PoseBlender,
    pub impact_weight_decay: f32,
}

impl ProceduralLayer {
    pub fn new(joint_count: usize) -> Self {
        Self {
            blender: PoseBlender::new(joint_count),
            impact_weight_decay: 2.0,
        }
    }

    pub fn on_heavy_impact(&mut self, force: f32, threshold: f32) {
        let ratio = (force / threshold).clamp(0.0, 3.0) / 3.0;
        let target_weight = ratio;
        if target_weight > self.blender.global_ragdoll_weight {
            self.blender.global_ragdoll_weight = target_weight;
        }
    }

    pub fn update(&mut self, dt: f32, balance_strength: f32) {
        let target = 1.0 - balance_strength;
        let current = self.blender.global_ragdoll_weight;
        if current > target {
            self.blender.global_ragdoll_weight =
                (current - dt * self.impact_weight_decay).max(target);
        } else {
            self.blender.global_ragdoll_weight =
                (current + dt * self.impact_weight_decay * 0.5).min(target);
        }
    }

    pub fn compute_final_pose(
        &self,
        animation_pose: &[Mat4],
        physics_pose: &[Mat4],
    ) -> Vec<Mat4> {
        self.blender.blend_poses(animation_pose, physics_pose)
    }

    pub fn configure_upper_lower_split(&mut self, joint_names: &[String]) {
        for (i, name) in joint_names.iter().enumerate() {
            let lower = name.to_lowercase();
            let w = if lower.contains("leg") || lower.contains("hip")
                || lower.contains("knee") || lower.contains("ankle")
                || lower.contains("foot") || lower.contains("toe")
            {
                1.0
            } else if lower.contains("spine") || lower.contains("pelvis") {
                0.6
            } else {
                0.3
            };
            self.blender.set_joint_weight(i, w);
        }
    }
}
