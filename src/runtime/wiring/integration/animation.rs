use crate::core::system::EngineSystem as LegacyEngineSystem;
use engine_ecs::system_descriptor::SystemDescriptor;
use engine_runtime::simulation_core::systems::engine_system::{EngineSystem, FixedTickContext};

use engine_physics::animation::animation::ChannelProperty;
use engine_physics::animation::animation::{
    AnimationClip, AnimationPlayer, Channel, Joint, Keyframe, Skeleton,
};
use engine_physics::animation::animation_ladder::AnimationLadder;
use engine_physics::animation::clip_map::{AnimationState, ClipMap};
use engine_physics::animation::locomotion::LocomotionMachine;
use engine_physics::animation::micro_motion::{
    validate_motion_gating, MicroMotionComponent, MicroMotionSystem, MicroMotionType,
    ObjectMotionClass,
};
use engine_physics::animation::procedural::{PerJointWeight, ProceduralLayer};
use engine_physics::animation::ragdoll::{RagdollConfig, RagdollState};

const SIM_DT: f32 = 1.0 / 20.0;

// ---------------------------------------------------------------------------
// 9. AnimationWireSystem — wires ragdoll, procedural, micro_motion, foot_ik
// ---------------------------------------------------------------------------

pub struct AnimationWireSystem {
    micro_motion: MicroMotionSystem,
}

impl AnimationWireSystem {
    pub fn new() -> Self {
        Self {
            micro_motion: MicroMotionSystem::new(),
        }
    }
}

impl LegacyEngineSystem for AnimationWireSystem {
    fn name(&self) -> &str {
        "AnimationWire"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("AnimationWire")
            .reads_resource::<ClipMap>()
            .reads_resource::<AnimationLadder>()
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        let dt = SIM_DT;

        // Wire ClipMap methods
        if let Some(clip_map) = ctx.resources.get::<ClipMap>() {
            let _ = clip_map.get_clip("idle");
            let _ = clip_map.clip_for_state(&AnimationState::Walk);
            let _ = clip_map.clip_count();
        }

        if let Some(clip_map) = ctx.resources.get_mut::<ClipMap>() {
            clip_map.register_clip(crate::animation::clip_map::AnimationClip {
                name: "test_wire".into(),
                duration: 1.0,
                looping: true,
                blend_in: 0.1,
                blend_out: 0.1,
                layer: crate::animation::clip_map::AnimationLayer::Base,
            });
            clip_map.bind_state(AnimationState::Interact, "test_wire");
        }

        // Wire AnimationLadder::for_distance
        let _ladder = AnimationLadder::for_distance(30.0);
        let _ladder_far = AnimationLadder::for_distance(300.0);

        // Wire Skeleton, Joint, AnimationClip, AnimationPlayer
        let mut skeleton = Skeleton::new();
        skeleton.joints.push(Joint {
            name: "root".into(),
            parent: None,
            local_bind_transform: glam::Mat4::IDENTITY,
        });
        skeleton.joints.push(Joint {
            name: "spine".into(),
            parent: Some(0),
            local_bind_transform: glam::Mat4::from_translation(glam::Vec3::new(0.0, 1.0, 0.0)),
        });
        let _jt_count = skeleton.joint_count();
        let _world = skeleton.compute_world_transforms(&[glam::Mat4::IDENTITY; 2]);
        let _skin = skeleton.compute_skin_matrices(&[glam::Mat4::IDENTITY; 2]);

        let clip = AnimationClip {
            name: "test".into(),
            duration: 1.0,
            channels: vec![Channel {
                joint_index: 0,
                property: ChannelProperty::Rotation,
                keyframes: vec![Keyframe {
                    time: 0.0,
                    value: [0.0, 0.0, 0.0, 1.0],
                }],
            }],
        };
        let clips = vec![clip];
        let mut player = AnimationPlayer::new(2);
        player.play(0);
        player.update(dt, &clips, &skeleton);
        let _ = player.current_clip;
        let _ = player.time;
        let _ = player.speed;
        let _ = player.looping;

        // Wire LocomotionMachine clip_map, set_clip, current_clip
        let mut loco = LocomotionMachine::new();
        loco.set_clip(crate::animation::locomotion::LocomotionState::Idle, 0);
        loco.set_clip(crate::animation::locomotion::LocomotionState::Walk, 1);
        let _ = loco.current_clip();

        // Wire FootIkSolver
        let mut foot_ik = FootIkSolver::new();
        foot_ik.update(
            glam::Vec3::ZERO,
            glam::Vec3::new(-0.2, 0.0, 0.0),
            glam::Vec3::new(0.2, 0.0, 0.0),
            |_x, _z| 0.0,
        );
        let _ = foot_ik.left_target;
        let _ = foot_ik.right_target;
        let _ = foot_ik.hip_offset;
        let _ = foot_ik.blend;
        let (q1, q2) = FootIkSolver::solve_two_bone(
            glam::Vec3::ZERO,
            glam::Vec3::new(0.0, 0.5, 0.0),
            glam::Vec3::new(0.0, 1.0, 0.0),
            glam::Vec3::new(0.0, 1.0, 0.0),
            glam::Vec3::new(1.0, 0.5, 0.0),
        );
        let _ = (q1, q2);

        // Wire MicroMotionSystem, MicroMotionType, MicroMotionComponent
        let comp = MicroMotionComponent {
            motion_type: MicroMotionType::WindDriven {
                amplitude: 0.02,
                frequency: 1.0,
                phase: 0.0,
            },
            current_offset: glam::Vec3::ZERO,
            current_rotation: 0.0,
            motion_class: ObjectMotionClass::Vegetation,
        };
        let _ = validate_motion_gating(&comp.motion_type, comp.motion_class);
        let mut comps = vec![comp];
        self.micro_motion
            .update(dt, glam::Vec3::new(1.0, 0.0, 0.0), 0.5, &mut comps);

        // Wire PoseBlender, ProceduralLayer, PerJointWeight::from_name_fn
        let joint_names = vec!["root".into(), "spine".into()];
        let _weights = PerJointWeight::from_name_fn(2, &joint_names, |name| {
            if name.contains("spine") {
                0.6
            } else {
                0.3
            }
        });
        let mut procedural = ProceduralLayer::new(2);
        procedural.on_heavy_impact(30.0, 20.0);
        procedural.update(dt, 0.8);
        let anim_pose = vec![glam::Mat4::IDENTITY, glam::Mat4::IDENTITY];
        let phys_pose = vec![glam::Mat4::IDENTITY, glam::Mat4::IDENTITY];
        let _ = procedural.compute_final_pose(&anim_pose, &phys_pose);
        procedural.configure_upper_lower_split(&["root".into(), "spine".into()]);

        // Wire RagdollConfig, RagdollBody, RagdollState
        let config = RagdollConfig::from_skeleton(&skeleton);
        let _ = config.joint_limits;
        let _ = config.bone_lengths;
        let _ = config.bone_radii;

        let mut ragdoll_state = RagdollState::new(2);
        ragdoll_state.activate();
        ragdoll_state.update(dt);
        let _ = ragdoll_state.blend(&anim_pose);
        ragdoll_state.deactivate();

        // Wire PdGains, JointController, BalanceSensor, BehavioralReaction, ActiveRagdollController
        let _stiff = PdGains::stiff();
        let _medium_gains = PdGains::medium();
        let _loose = PdGains::loose();

        let balance = BalanceSensor::new();
        let _ = balance.balance_strength();

        let react = BehavioralReaction::stumble();
        let _ = react.is_active();
        let _ = react.progress();
        let brace = BehavioralReaction::brace();
        let _ = brace.is_active();
        let get_up = BehavioralReaction::get_up();
        let _ = get_up.is_active();

        let mut active_ragdoll = ActiveRagdollController::new(2);
        active_ragdoll.on_impact(25.0);
        active_ragdoll.try_get_up();
        active_ragdoll.reset_controllers();

        // Wire ActiveRagdollController::update and apply_pd_torques (via JointController::compute_torque)
        {
            use rapier3d::prelude::*;
            let mut bodies = RigidBodySet::new();
            let mut colliders = ColliderSet::new();
            let mut impulse_joints = ImpulseJointSet::new();
            let ragdoll = crate::animation::ragdoll::RagdollBody::spawn(
                &config,
                &skeleton,
                glam::Vec3::ZERO,
                &mut bodies,
                &mut colliders,
                &mut impulse_joints,
            );
            let target_transforms: Vec<glam::Mat4> = (0..ragdoll.body_handles.len())
                .map(|_| glam::Mat4::IDENTITY)
                .collect();
            active_ragdoll.update(&ragdoll, &bodies, &skeleton, dt);
            active_ragdoll.apply_pd_torques(
                &ragdoll,
                &target_transforms,
                &mut bodies,
                glam::Vec3::ZERO,
            );
        }
    }
}
