use glam::{Mat4, Quat, Vec3};
use rapier3d::prelude::*;

use crate::animation::animation::Skeleton;
use crate::animation::ragdoll::RagdollBody;

pub struct PdGains {
    pub kp: f32,
    pub kd: f32,
}

impl PdGains {
    pub fn stiff() -> Self {
        Self {
            kp: 600.0,
            kd: 60.0,
        }
    }
    pub fn medium() -> Self {
        Self {
            kp: 300.0,
            kd: 40.0,
        }
    }
    pub fn loose() -> Self {
        Self { kp: 80.0, kd: 20.0 }
    }
}

pub struct JointController {
    pub gains: PdGains,
    pub strength: f32,
}

impl JointController {
    fn compute_torque(&self, current_rot: Quat, target_rot: Quat, angular_vel: Vec3) -> Vec3 {
        let error_quat = target_rot * current_rot.inverse();
        let (axis, angle) = error_quat.to_axis_angle();
        let angle = if angle > std::f32::consts::PI {
            angle - 2.0 * std::f32::consts::PI
        } else {
            angle
        };

        let proportional = axis * angle * self.gains.kp;
        let derivative = -angular_vel * self.gains.kd;
        (proportional + derivative) * self.strength
    }
}

pub struct BalanceSensor {
    pub center_of_mass: Vec3,
    pub support_center: Vec3,
    pub balance_error: f32,
    pub is_balanced: bool,
    pub fall_timer: f32,
}

impl BalanceSensor {
    pub fn new() -> Self {
        Self {
            center_of_mass: Vec3::ZERO,
            support_center: Vec3::ZERO,
            balance_error: 0.0,
            is_balanced: true,
            fall_timer: 0.0,
        }
    }

    pub fn update(
        &mut self,
        ragdoll: &RagdollBody,
        bodies: &RigidBodySet,
        skeleton: &Skeleton,
        dt: f32,
    ) {
        let mut total_mass = 0.0f32;
        let mut com = Vec3::ZERO;
        let mut foot_positions = Vec::new();

        for (i, &bh) in ragdoll.body_handles.iter().enumerate() {
            if let Some(body) = bodies.get(bh) {
                let mass = body.mass();
                com += body.translation() * mass;
                total_mass += mass;

                if i < skeleton.joint_count() {
                    let name = skeleton.joints[i].name.to_lowercase();
                    if name.contains("foot") || name.contains("toe") || name.contains("ankle") {
                        foot_positions.push(body.translation());
                    }
                }
            }
        }

        if total_mass > 0.01 {
            self.center_of_mass = com / total_mass;
        }

        if foot_positions.is_empty() {
            if ragdoll.body_handles.len() >= 2 {
                for &bh in ragdoll.body_handles.iter().rev().take(2) {
                    if let Some(body) = bodies.get(bh) {
                        foot_positions.push(body.translation());
                    }
                }
            }
        }

        if !foot_positions.is_empty() {
            self.support_center =
                foot_positions.iter().copied().sum::<Vec3>() / foot_positions.len() as f32;
        }

        let horizontal_offset = Vec3::new(
            self.center_of_mass.x - self.support_center.x,
            0.0,
            self.center_of_mass.z - self.support_center.z,
        );
        self.balance_error = horizontal_offset.length();

        const BALANCE_THRESHOLD: f32 = 0.3;
        const FALL_THRESHOLD: f32 = 0.6;

        if self.balance_error > FALL_THRESHOLD {
            self.is_balanced = false;
            self.fall_timer += dt;
        } else if self.balance_error < BALANCE_THRESHOLD {
            self.is_balanced = true;
            self.fall_timer = (self.fall_timer - dt * 2.0).max(0.0);
        } else {
            self.fall_timer += dt * 0.5;
        }
    }

    pub fn balance_strength(&self) -> f32 {
        if self.is_balanced {
            1.0
        } else {
            (1.0 - self.fall_timer * 2.0).clamp(0.0, 1.0)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReactionKind {
    None,
    Stumble,
    BraceForImpact,
    GetUp,
    FullRagdoll,
}

pub struct BehavioralReaction {
    pub kind: ReactionKind,
    pub timer: f32,
    pub duration: f32,
}

impl BehavioralReaction {
    pub fn none() -> Self {
        Self {
            kind: ReactionKind::None,
            timer: 0.0,
            duration: 0.0,
        }
    }

    pub fn stumble() -> Self {
        Self {
            kind: ReactionKind::Stumble,
            timer: 0.0,
            duration: 0.8,
        }
    }

    pub fn brace() -> Self {
        Self {
            kind: ReactionKind::BraceForImpact,
            timer: 0.0,
            duration: 0.5,
        }
    }

    pub fn get_up() -> Self {
        Self {
            kind: ReactionKind::GetUp,
            timer: 0.0,
            duration: 2.0,
        }
    }

    pub fn is_active(&self) -> bool {
        self.kind != ReactionKind::None && self.timer < self.duration
    }

    pub fn update(&mut self, dt: f32) {
        if self.kind != ReactionKind::None {
            self.timer += dt;
            if self.timer >= self.duration {
                self.kind = ReactionKind::None;
                self.timer = 0.0;
            }
        }
    }

    pub fn progress(&self) -> f32 {
        if self.duration > 0.0 {
            (self.timer / self.duration).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }
}

pub struct ActiveRagdollController {
    pub joint_controllers: Vec<JointController>,
    pub balance: BalanceSensor,
    pub reaction: BehavioralReaction,
    pub overall_strength: f32,
    pub impact_threshold: f32,
}

impl ActiveRagdollController {
    pub fn new(joint_count: usize) -> Self {
        let mut controllers = Vec::with_capacity(joint_count);
        for _ in 0..joint_count {
            controllers.push(JointController {
                gains: PdGains::stiff(),
                strength: 1.0,
            });
        }
        Self {
            joint_controllers: controllers,
            balance: BalanceSensor::new(),
            reaction: BehavioralReaction::none(),
            overall_strength: 1.0,
            impact_threshold: 50.0,
        }
    }

    pub fn apply_pd_torques(
        &self,
        ragdoll: &RagdollBody,
        target_transforms: &[Mat4],
        bodies: &mut RigidBodySet,
        root_position: Vec3,
    ) {
        for (i, &bh) in ragdoll.body_handles.iter().enumerate() {
            if i >= self.joint_controllers.len() || i >= target_transforms.len() {
                continue;
            }
            let controller = &self.joint_controllers[i];
            if controller.strength < 0.01 {
                continue;
            }

            if let Some(body) = bodies.get_mut(bh) {
                let (_, target_rot, target_pos) =
                    target_transforms[i].to_scale_rotation_translation();

                let current_rot = *body.rotation();
                let angular_vel = body.angvel();

                let torque = controller.compute_torque(current_rot, target_rot, angular_vel)
                    * self.overall_strength;

                body.apply_torque_impulse(torque * (1.0 / 60.0), true);

                let pos_error = (target_pos + root_position) - body.translation();
                let force = pos_error * controller.gains.kp * 0.1
                    - body.linvel() * controller.gains.kd * 0.1;
                body.add_force(force * self.overall_strength * controller.strength, true);
            }
        }
    }

    pub fn update(
        &mut self,
        ragdoll: &RagdollBody,
        bodies: &RigidBodySet,
        skeleton: &Skeleton,
        dt: f32,
    ) {
        self.balance.update(ragdoll, bodies, skeleton, dt);
        self.reaction.update(dt);

        let balance_str = self.balance.balance_strength();
        self.overall_strength = balance_str;

        match self.reaction.kind {
            ReactionKind::Stumble => {
                self.apply_stumble_modifiers(skeleton);
            }
            ReactionKind::BraceForImpact => {
                self.apply_brace_modifiers(skeleton);
            }
            ReactionKind::GetUp => {
                let progress = self.reaction.progress();
                self.overall_strength = progress * balance_str;
            }
            ReactionKind::FullRagdoll => {
                self.overall_strength = 0.0;
            }
            ReactionKind::None => {}
        }
    }

    pub fn on_impact(&mut self, force_magnitude: f32) {
        if force_magnitude > self.impact_threshold * 3.0 {
            self.reaction = BehavioralReaction {
                kind: ReactionKind::FullRagdoll,
                timer: 0.0,
                duration: 3.0,
            };
            self.overall_strength = 0.0;
        } else if force_magnitude > self.impact_threshold * 1.5 {
            if self.reaction.kind == ReactionKind::None {
                self.reaction = BehavioralReaction::brace();
            }
        } else if force_magnitude > self.impact_threshold {
            if self.reaction.kind == ReactionKind::None {
                self.reaction = BehavioralReaction::stumble();
            }
        }
    }

    pub fn try_get_up(&mut self) {
        if !self.balance.is_balanced && self.reaction.kind == ReactionKind::None {
            self.reaction = BehavioralReaction::get_up();
        }
    }

    fn apply_stumble_modifiers(&mut self, skeleton: &Skeleton) {
        let progress = self.reaction.progress();
        for (i, ctrl) in self.joint_controllers.iter_mut().enumerate() {
            if i < skeleton.joint_count() {
                let name = skeleton.joints[i].name.to_lowercase();
                if name.contains("leg")
                    || name.contains("hip")
                    || name.contains("knee")
                    || name.contains("ankle")
                    || name.contains("foot")
                {
                    ctrl.strength = 0.3 + 0.7 * progress;
                    ctrl.gains = PdGains::medium();
                } else {
                    ctrl.strength = 0.7 + 0.3 * progress;
                }
            }
        }
    }

    fn apply_brace_modifiers(&mut self, skeleton: &Skeleton) {
        let _progress = self.reaction.progress();
        for (i, ctrl) in self.joint_controllers.iter_mut().enumerate() {
            if i < skeleton.joint_count() {
                let name = skeleton.joints[i].name.to_lowercase();
                if name.contains("arm")
                    || name.contains("hand")
                    || name.contains("wrist")
                    || name.contains("elbow")
                    || name.contains("shoulder")
                {
                    ctrl.gains = PdGains::stiff();
                    ctrl.strength = 1.0;
                } else if name.contains("leg") || name.contains("knee") {
                    ctrl.strength = 0.4;
                    ctrl.gains = PdGains::loose();
                }
            }
        }
    }

    pub fn reset_controllers(&mut self) {
        for ctrl in &mut self.joint_controllers {
            ctrl.gains = PdGains::stiff();
            ctrl.strength = 1.0;
        }
        self.overall_strength = 1.0;
        self.reaction = BehavioralReaction::none();
    }
}
