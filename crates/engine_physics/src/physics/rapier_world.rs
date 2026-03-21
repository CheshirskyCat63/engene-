use std::collections::HashMap;
use std::sync::Arc;

use glam::Vec3;
use rapier3d::prelude::*;

use crate::core::ecs::{Ecs, Entity};
use crate::world::components::SimulationLevel;
use crate::world::heightmap::Heightmap;

pub struct RapierPhysics {
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    gravity: Vec3,
    integration_params: IntegrationParameters,
    pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,

    entity_handles: HashMap<Entity, RigidBodyHandle>,
    heightmap: Arc<Heightmap>,
}

impl RapierPhysics {
    pub fn new(heightmap: Arc<Heightmap>) -> Self {
        let mut bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();

        let ws = heightmap.world_size;

        // Large flat ground collider as terrain approximation
        let terrain_body = bodies.insert(
            RigidBodyBuilder::fixed()
                .translation(Vec3::new(ws / 2.0, -0.5, ws / 2.0))
                .build(),
        );
        let ground = ColliderBuilder::cuboid(ws / 2.0, 0.5, ws / 2.0)
            .friction(0.8)
            .build();
        colliders.insert_with_parent(ground, terrain_body, &mut bodies);

        Self {
            bodies,
            colliders,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            integration_params: IntegrationParameters::default(),
            pipeline: PhysicsPipeline::new(),
            islands: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            entity_handles: HashMap::new(),
            heightmap,
        }
    }

    pub fn sync_entities(&mut self, ecs: &Ecs) {
        let alive: std::collections::HashSet<Entity> = ecs.alive.iter().copied().collect();

        let removed: Vec<Entity> = self
            .entity_handles
            .keys()
            .copied()
            .filter(|e| {
                !alive.contains(e)
                    || ecs
                        .sim_levels
                        .get(e)
                        .map_or(false, |s| s.level != SimulationLevel::L0)
            })
            .collect();
        for e in removed {
            if let Some(handle) = self.entity_handles.remove(&e) {
                self.bodies.remove(
                    handle,
                    &mut self.islands,
                    &mut self.colliders,
                    &mut self.impulse_joints,
                    &mut self.multibody_joints,
                    true,
                );
            }
        }

        for &e in &ecs.alive {
            if self.entity_handles.contains_key(&e) {
                continue;
            }
            let is_l0 = ecs
                .sim_levels
                .get(&e)
                .map_or(true, |s| s.level == SimulationLevel::L0);
            if !is_l0 {
                continue;
            }
            if let Some(t) = ecs.get_transform(e) {
                let y = self.heightmap.sample(t.x, t.y) + 1.0;
                let body = RigidBodyBuilder::kinematic_position_based()
                    .translation(Vec3::new(t.x, y, t.y))
                    .build();
                let handle = self.bodies.insert(body);
                let collider = ColliderBuilder::capsule_y(0.5, 0.5).friction(0.5).build();
                self.colliders
                    .insert_with_parent(collider, handle, &mut self.bodies);
                self.entity_handles.insert(e, handle);
            }
        }
    }

    pub fn update_kinematic(&mut self, ecs: &Ecs) {
        for (&entity, &handle) in &self.entity_handles {
            if let Some(body) = self.bodies.get_mut(handle) {
                if body.is_kinematic() {
                    if let Some(t) = ecs.get_transform(entity) {
                        let y = self.heightmap.sample(t.x, t.y) + 1.0;
                        body.set_next_kinematic_translation(Vec3::new(t.x, y, t.y));
                    }
                }
            }
        }
    }

    pub fn step(&mut self) {
        self.pipeline.step(
            self.gravity,
            &self.integration_params,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            &(),
            &(),
        );
    }

    pub fn sync_dynamic_to_ecs(&self, ecs: &mut Ecs) {
        for (&entity, &handle) in &self.entity_handles {
            if let Some(body) = self.bodies.get(handle) {
                if body.is_dynamic() {
                    let pos = body.translation();
                    if let Some(t) = ecs.get_transform_mut(entity) {
                        t.x = pos.x;
                        t.y = pos.z;
                    }
                }
            }
        }
    }

    pub fn make_dynamic(&mut self, entity: Entity) {
        if let Some(&handle) = self.entity_handles.get(&entity) {
            if let Some(body) = self.bodies.get_mut(handle) {
                body.set_body_type(RigidBodyType::Dynamic, true);
            }
        }
    }
}
