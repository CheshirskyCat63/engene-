use std::sync::Arc;

use crate::core::mutation_policy::*;
use crate::core::system::EngineSystem;
use crate::core::system_descriptor::{DeterminismTier, SystemDescriptor};
use crate::physics::cloth::ClothWorld;
use crate::physics::damage_pipeline::response_aggregator::SurfaceMaskType;
use crate::physics::fire::FireGrid;
use crate::physics::rapier_world::RapierPhysics;
use crate::physics::sim_lod;
use crate::physics::water::WaterGrid;
use crate::world::fields::WorldFields;
use crate::world::heightmap::Heightmap;
use crate::world::surface_state::SurfaceStateStore;

pub struct PhysicsSystem {
    rapier: RapierPhysics,
    pub fire: FireGrid,
    pub water: WaterGrid,
    pub cloth: ClothWorld,
    cam_x: f32,
    cam_z: f32,
}

impl PhysicsSystem {
    pub fn new(heightmap: Arc<Heightmap>) -> Self {
        Self {
            rapier: RapierPhysics::new(heightmap),
            fire: FireGrid::new(),
            water: WaterGrid::new(),
            cloth: ClothWorld::new(),
            cam_x: 0.0,
            cam_z: 0.0,
        }
    }

    pub fn set_camera(&mut self, x: f32, z: f32) {
        self.cam_x = x;
        self.cam_z = z;
    }

    /// Apply canonical world-field truth into fire/water owner inputs.
    /// Returns sampled rain intensity used for this tick.
    pub fn apply_world_fields(&mut self, fields: &WorldFields, time: f32) -> f32 {
        let sampled_wind = fields.wind.sample(glam::Vec3::ZERO, time);
        self.fire.wind = [sampled_wind.x, sampled_wind.z];
        let rain = fields.rain.sample(glam::Vec3::ZERO, time).max(0.0);
        self.water.rain_intensity = rain;
        rain
    }
}

impl EngineSystem for PhysicsSystem {
    fn name(&self) -> &str {
        "Physics"
    }

    fn descriptor(&self) -> SystemDescriptor {
        SystemDescriptor::new("Physics")
            .with_determinism(DeterminismTier::Hard)
            .with_headless(true)
            .reads_resource::<WorldFields>()
            .writes_resource::<SurfaceStateStore>()
            .after("AI")
    }

    fn startup(&mut self, ctx: &mut StartupContext) {
        self.rapier.sync_entities(ctx.ecs);
    }

    fn fixed_tick(&mut self, ctx: &mut FixedTickContext) {
        self.rapier.sync_entities(ctx.ecs);
        self.rapier.update_kinematic(ctx.ecs);
        self.rapier.step();
        self.rapier.sync_dynamic_to_ecs(ctx.ecs);

        let world_center_x = crate::world::cell::WORLD_SIZE * 0.5;
        let world_center_z = crate::world::cell::WORLD_SIZE * 0.5;
        let region_level =
            sim_lod::region_sim_level(self.cam_x, self.cam_z, world_center_x, world_center_z);

        let dt = 1.0 / 20.0;
        let mut rain = 0.0;
        if let Some(fields) = ctx.resources.get::<WorldFields>() {
            rain = self.apply_world_fields(fields, 0.0);
            if rain > 0.0 {
                // Ensure canonical rain contributes to water truth even in coarse/far simulation levels.
                self.water.add_water(0, 0, rain * dt * 0.01);
            }
        }

        let fire_dt = dt * (1.0 - (rain * 0.5).min(0.8));
        self.fire.update(fire_dt, region_level);
        self.water.update(dt, region_level);
        self.cloth.update(dt, region_level);

        let water_present = self.water.has_any_water();
        if let Some(surface) = ctx.resources.get_mut::<SurfaceStateStore>() {
            let raining = rain > 0.0;
            surface.decay_tick(dt, raining);
            if raining {
                surface.apply_mask_delta(0, 0, SurfaceMaskType::Wetness, dt * rain.min(1.0));
            } else if water_present {
                surface.apply_mask_delta(0, 0, SurfaceMaskType::Wetness, dt * 0.02);
            }
        }

        apply_fire_fear(ctx.ecs, &self.fire);
    }
}

fn apply_fire_fear(ecs: &mut crate::core::ecs::Ecs, fire: &FireGrid) {
    if fire.active_fire_count() == 0 {
        return;
    }

    let fire_radius = 80.0;
    let entities = ecs.alive.clone();
    for e in entities {
        if let Some(t) = ecs.get_transform(e) {
            if fire.is_near_fire(t.x, t.y, fire_radius) {
                if let Some(pn) = ecs.get_needs_mut(e) {
                    pn.fear = (pn.fear + 0.1).min(1.0);
                }
                if let Some(emo) = ecs.get_emotions_mut(e) {
                    emo.fear = (emo.fear + 0.15).min(1.0);
                }
            }
        }
    }
}
