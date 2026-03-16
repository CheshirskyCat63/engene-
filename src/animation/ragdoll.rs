use glam::{Mat4, Vec3};
use rapier3d::prelude::*;

use crate::animation::animation::Skeleton;

pub struct JointLimit {
    pub joint_index: usize,
    pub swing_limit: f32,
    pub twist_limit: f32,
}

pub struct RagdollConfig {
    pub joint_limits: Vec<JointLimit>,
    pub bone_lengths: Vec<f32>,
    pub bone_radii: Vec<f32>,
}

impl RagdollConfig {
    pub fn from_skeleton(skeleton: &Skeleton) -> Self {
        let n = skeleton.joint_count();
        let world = skeleton.compute_world_transforms(
            &vec![Mat4::IDENTITY; n],
        );

        let mut bone_lengths = vec![0.3f32; n];
        let mut bone_radii = vec![0.06f32; n];
        let mut limits = Vec::with_capacity(n);

        for i in 0..n {
            let pos_i = world[i].col(3).truncate();
            let mut min_child_dist = f32::MAX;
            for j in 0..n {
                if skeleton.joints[j].parent == Some(i) {
                    let pos_j = world[j].col(3).truncate();
                    let d = (pos_j - pos_i).length();
                    if d < min_child_dist {
                        min_child_dist = d;
                    }
                }
            }
            if min_child_dist < f32::MAX && min_child_dist > 0.01 {
                bone_lengths[i] = min_child_dist;
                bone_radii[i] = (min_child_dist * 0.2).clamp(0.03, 0.12);
            }

            let name_lower = skeleton.joints[i].name.to_lowercase();
            let (swing, twist) = if name_lower.contains("spine") || name_lower.contains("neck") {
                (0.35, 0.3)
            } else if name_lower.contains("shoulder") || name_lower.contains("arm") {
                (std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_4)
            } else if name_lower.contains("hip") || name_lower.contains("thigh") || name_lower.contains("leg") {
                (std::f32::consts::FRAC_PI_3, 0.3)
            } else if name_lower.contains("knee") || name_lower.contains("elbow") {
                (std::f32::consts::FRAC_PI_3, 0.1)
            } else if name_lower.contains("head") {
                (0.5, 0.4)
            } else {
                (std::f32::consts::FRAC_PI_4, std::f32::consts::FRAC_PI_6)
            };

            limits.push(JointLimit {
                joint_index: i,
                swing_limit: swing,
                twist_limit: twist,
            });
        }

        Self {
            joint_limits: limits,
            bone_lengths,
            bone_radii,
        }
    }
}

pub struct RagdollBody {
    pub body_handles: Vec<RigidBodyHandle>,
    pub collider_handles: Vec<ColliderHandle>,
    pub joint_handles: Vec<ImpulseJointHandle>,
}

impl RagdollBody {
    pub fn spawn(
        config: &RagdollConfig,
        skeleton: &Skeleton,
        root_position: Vec3,
        bodies: &mut RigidBodySet,
        colliders: &mut ColliderSet,
        impulse_joints: &mut ImpulseJointSet,
    ) -> Self {
        let n = skeleton.joint_count();
        let local_transforms: Vec<Mat4> = skeleton
            .joints
            .iter()
            .map(|j| j.local_bind_transform)
            .collect();
        let world = skeleton.compute_world_transforms(&local_transforms);

        let mut body_handles = Vec::with_capacity(n);
        let mut collider_handles = Vec::with_capacity(n);
        let mut joint_handles = Vec::new();

        for i in 0..n {
            let bone_pos = world[i].col(3).truncate() + root_position;
            let (_, rot, _) = world[i].to_scale_rotation_translation();

            let half_len = (config.bone_lengths[i] * 0.5).max(0.02);
            let radius = config.bone_radii[i];

            let rb = RigidBodyBuilder::dynamic()
                .translation(bone_pos)
                .rotation(rot.to_scaled_axis())
                .linear_damping(0.3)
                .angular_damping(0.5)
                .build();
            let handle = bodies.insert(rb);

            let col = ColliderBuilder::capsule_y(half_len, radius)
                .mass(0.5 + (radius * 10.0))
                .friction(0.6)
                .build();
            let ch = colliders.insert_with_parent(col, handle, bodies);

            body_handles.push(handle);
            collider_handles.push(ch);
        }

        for i in 0..n {
            if let Some(parent_idx) = skeleton.joints[i].parent {
                let parent_handle = body_handles[parent_idx];
                let child_handle = body_handles[i];

                let parent_world_pos = world[parent_idx].col(3).truncate();
                let child_world_pos = world[i].col(3).truncate();
                let (_, parent_rot, _) = world[parent_idx].to_scale_rotation_translation();
                let (_, child_rot, _) = world[i].to_scale_rotation_translation();

                let anchor_world = child_world_pos;
                let anchor_in_parent = parent_rot.inverse() * (anchor_world - parent_world_pos);
                let anchor_in_child = child_rot.inverse() * (anchor_world - child_world_pos);

                let limit = &config.joint_limits[i];
                let joint = SphericalJointBuilder::new()
                    .local_anchor1(anchor_in_parent.into())
                    .local_anchor2(anchor_in_child.into())
                    .limits(JointAxis::AngX, [-limit.twist_limit, limit.twist_limit])
                    .limits(JointAxis::AngY, [-limit.swing_limit, limit.swing_limit])
                    .limits(JointAxis::AngZ, [-limit.swing_limit, limit.swing_limit])
                    .build();

                let jh = impulse_joints.insert(parent_handle, child_handle, joint, true);
                joint_handles.push(jh);
            }
        }

        Self {
            body_handles,
            collider_handles,
            joint_handles,
        }
    }

    pub fn read_physics_transforms(
        &self,
        bodies: &RigidBodySet,
        root_position: Vec3,
    ) -> Vec<Mat4> {
        self.body_handles
            .iter()
            .map(|&h| {
                if let Some(body) = bodies.get(h) {
                    let pos = body.translation() - root_position;
                    let rot = *body.rotation();
                    Mat4::from_rotation_translation(rot, pos)
                } else {
                    Mat4::IDENTITY
                }
            })
            .collect()
    }

    pub fn remove(
        self,
        bodies: &mut RigidBodySet,
        colliders: &mut ColliderSet,
        impulse_joints: &mut ImpulseJointSet,
        islands: &mut IslandManager,
        multibody_joints: &mut MultibodyJointSet,
    ) {
        for jh in self.joint_handles {
            impulse_joints.remove(jh, true);
        }
        for bh in self.body_handles {
            bodies.remove(bh, islands, colliders, impulse_joints, multibody_joints, true);
        }
    }
}

pub struct RagdollState {
    pub active: bool,
    pub blend_weight: f32,
    pub physics_transforms: Vec<Mat4>,
    pub transition_speed: f32,
}

impl RagdollState {
    pub fn new(joint_count: usize) -> Self {
        Self {
            active: false,
            blend_weight: 0.0,
            physics_transforms: vec![Mat4::IDENTITY; joint_count],
            transition_speed: 3.0,
        }
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.blend_weight = 0.0;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn update(&mut self, dt: f32) {
        if self.active {
            self.blend_weight = (self.blend_weight + dt * self.transition_speed).min(1.0);
        } else {
            self.blend_weight = (self.blend_weight - dt * self.transition_speed * 0.5).max(0.0);
        }
    }

    pub fn blend(&self, animation_transforms: &[Mat4]) -> Vec<Mat4> {
        if self.blend_weight < 0.001 {
            return animation_transforms.to_vec();
        }
        animation_transforms
            .iter()
            .enumerate()
            .map(|(i, anim)| {
                let phys = if i < self.physics_transforms.len() {
                    self.physics_transforms[i]
                } else {
                    *anim
                };
                let w = self.blend_weight;
                Mat4::from_cols(
                    anim.col(0).lerp(phys.col(0), w),
                    anim.col(1).lerp(phys.col(1), w),
                    anim.col(2).lerp(phys.col(2), w),
                    anim.col(3).lerp(phys.col(3), w),
                )
            })
            .collect()
    }
}
