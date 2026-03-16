use glam::Vec3;
use std::collections::HashMap;

use crate::core::ecs::Entity;
use crate::world::fields::WorldFields;

#[derive(Clone, Debug)]
pub struct Projectile {
    pub pos: Vec3,
    pub prev_pos: Vec3,
    pub vel: Vec3,
    pub energy: f32,
    pub drag_coeff: f32,
    pub mass: f32,
    pub ttl: f32,
    pub seed: u32,
    pub owner: Entity,
    pub weapon_id: u16,
    pub distance_traveled: f32,
}

#[derive(Clone, Debug)]
pub struct MaterialProps {
    pub hardness: f32,
    pub penetration_resistance: f32,
    pub density: f32,
}

pub type MaterialId = u16;

pub struct MaterialTable {
    pub materials: HashMap<MaterialId, MaterialProps>,
    pub name_to_id: HashMap<String, MaterialId>,
}

impl MaterialTable {
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
            name_to_id: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, id: MaterialId, props: MaterialProps) {
        self.materials.insert(id, props);
        self.name_to_id.insert(name.to_string(), id);
    }

    pub fn get(&self, id: MaterialId) -> Option<&MaterialProps> {
        self.materials.get(&id)
    }
}

#[derive(Clone, Debug)]
pub enum ImpactResult {
    Stopped { pos: Vec3, normal: Vec3, material: MaterialId },
    Ricochet { pos: Vec3, new_vel: Vec3, energy_lost: f32 },
    Penetrated { pos: Vec3, exit_vel: Vec3, energy_lost: f32 },
}

#[derive(Clone, Debug)]
pub enum BallisticEvent {
    ShotFired { seed: u32, origin: Vec3, dir: Vec3, weapon_id: u16, owner: Entity },
    Impact { seed: u32, hit_pos: Vec3, normal: Vec3, material: MaterialId, damage: f32 },
    EntityHit { entity: Entity, damage: f32, hit_pos: Vec3, projectile_vel: Vec3 },
}

pub struct BallisticsSystem {
    pub projectiles: Vec<Projectile>,
    pub material_table: MaterialTable,
    pub events: Vec<BallisticEvent>,
    pub gravity: Vec3,
}

impl BallisticsSystem {
    pub fn new() -> Self {
        Self {
            projectiles: Vec::new(),
            material_table: MaterialTable::new(),
            events: Vec::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
        }
    }

    pub fn fire(&mut self, origin: Vec3, dir: Vec3, speed: f32, mass: f32, drag: f32, owner: Entity, weapon_id: u16, seed: u32) {
        let vel = dir.normalize() * speed;
        let energy = 0.5 * mass * speed * speed;
        self.projectiles.push(Projectile {
            pos: origin,
            prev_pos: origin,
            vel,
            energy,
            drag_coeff: drag,
            mass,
            ttl: 5.0,
            seed,
            owner,
            weapon_id,
            distance_traveled: 0.0,
        });
        self.events.push(BallisticEvent::ShotFired { seed, origin, dir, weapon_id, owner });
    }

    pub fn update(&mut self, dt: f32, fields: &WorldFields, terrain_height_fn: &dyn Fn(f32, f32) -> f32) {
        self.events.clear();
        let mut to_remove = Vec::new();

        for (i, proj) in self.projectiles.iter_mut().enumerate() {
            proj.ttl -= dt;
            if proj.ttl <= 0.0 {
                to_remove.push(i);
                continue;
            }

            let substeps = adaptive_substeps(proj.vel.length(), proj.distance_traveled);
            let sub_dt = dt / substeps as f32;

            for _ in 0..substeps {
                proj.prev_pos = proj.pos;

                let wind = fields.wind.sample(proj.pos, 0.0);
                let rho = fields.air_density.sample(proj.pos, 0.0);
                let v_rel = proj.vel - wind;
                let speed_rel = v_rel.length();

                let drag_force = if speed_rel > 0.001 {
                    -proj.drag_coeff * rho * speed_rel * v_rel
                } else {
                    Vec3::ZERO
                };

                let anomaly = fields.anomaly.sample(proj.pos);
                let accel = self.gravity + drag_force / proj.mass + anomaly.acceleration;
                proj.vel += accel * sub_dt;
                proj.pos += proj.vel * sub_dt;
                proj.distance_traveled += (proj.pos - proj.prev_pos).length();
                proj.energy = 0.5 * proj.mass * proj.vel.length_squared();

                let terrain_y = terrain_height_fn(proj.pos.x, proj.pos.z);
                if proj.pos.y <= terrain_y {
                    to_remove.push(i);
                    break;
                }
            }
        }

        to_remove.sort_unstable();
        to_remove.dedup();
        for &i in to_remove.iter().rev() {
            if i < self.projectiles.len() {
                self.projectiles.swap_remove(i);
            }
        }
    }

    pub fn resolve_impact(&self, proj: &Projectile, hit_normal: Vec3, mat_id: MaterialId) -> ImpactResult {
        let mat = match self.material_table.get(mat_id) {
            Some(m) => m,
            None => return ImpactResult::Stopped { pos: proj.pos, normal: hit_normal, material: mat_id },
        };

        let incident_dir = proj.vel.normalize();
        let cos_angle = (-incident_dir.dot(hit_normal)).abs();

        if cos_angle < 0.3 && mat.hardness > 0.5 {
            let reflected = incident_dir - 2.0 * incident_dir.dot(hit_normal) * hit_normal;
            let energy_lost = proj.energy * (1.0 - cos_angle) * mat.hardness;
            return ImpactResult::Ricochet {
                pos: proj.pos,
                new_vel: reflected * proj.vel.length() * 0.6,
                energy_lost,
            };
        }

        let pen_threshold = mat.penetration_resistance * 1000.0;
        if proj.energy > pen_threshold {
            let energy_lost = pen_threshold;
            let remaining_ratio = ((proj.energy - energy_lost) / proj.energy).sqrt();
            return ImpactResult::Penetrated {
                pos: proj.pos,
                exit_vel: proj.vel * remaining_ratio,
                energy_lost,
            };
        }

        ImpactResult::Stopped { pos: proj.pos, normal: hit_normal, material: mat_id }
    }

    pub fn analytical_trajectory(
        &self,
        origin: Vec3,
        dir: Vec3,
        speed: f32,
        mass: f32,
        drag: f32,
        fields: &WorldFields,
        max_distance: f32,
    ) -> Vec<Vec3> {
        let vel = dir.normalize() * speed;
        let mut pos = origin;
        let mut v = vel;
        let mut points = vec![pos];
        let mut dist = 0.0;
        let coarse_dt = 0.05;

        while dist < max_distance && v.length() > 10.0 {
            let wind = fields.wind.sample(pos, 0.0);
            let rho = fields.air_density.sample(pos, 0.0);
            let v_rel = v - wind;
            let speed_rel = v_rel.length();
            let drag_force = -drag * rho * speed_rel * v_rel;
            let accel = self.gravity + drag_force / mass;
            v += accel * coarse_dt;
            let prev = pos;
            pos += v * coarse_dt;
            dist += (pos - prev).length();
            points.push(pos);
        }

        points
    }

    pub fn drain_events(&mut self) -> Vec<BallisticEvent> {
        std::mem::take(&mut self.events)
    }
}

fn adaptive_substeps(speed: f32, distance: f32) -> u32 {
    if distance < 200.0 && speed > 300.0 {
        4
    } else if speed > 200.0 {
        2
    } else {
        1
    }
}
