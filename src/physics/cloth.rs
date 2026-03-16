use crate::world::components::SimulationLevel;

#[derive(Clone, Debug)]
pub struct ClothParticle {
    pub pos: [f32; 3],
    pub prev_pos: [f32; 3],
    pub inv_mass: f32,
}

#[derive(Clone, Debug)]
pub struct DistanceConstraint {
    pub a: usize,
    pub b: usize,
    pub rest_length: f32,
    pub stiffness: f32,
}

pub struct ClothSim {
    pub id: u32,
    pub particles: Vec<ClothParticle>,
    pub constraints: Vec<DistanceConstraint>,
    pub width: u32,
    pub height: u32,
    pub wind: [f32; 3],
    pub gravity: [f32; 3],
}

impl ClothSim {
    pub fn new(id: u32, width: u32, height: u32, origin: [f32; 3], spacing: f32) -> Self {
        let mut particles = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                let px = origin[0] + x as f32 * spacing;
                let py = origin[1];
                let pz = origin[2] + y as f32 * spacing;
                let inv_mass = if y == 0 { 0.0 } else { 1.0 };
                particles.push(ClothParticle {
                    pos: [px, py, pz],
                    prev_pos: [px, py, pz],
                    inv_mass,
                });
            }
        }

        let mut constraints = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                if x + 1 < width {
                    constraints.push(DistanceConstraint {
                        a: idx,
                        b: idx + 1,
                        rest_length: spacing,
                        stiffness: 0.8,
                    });
                }
                if y + 1 < height {
                    constraints.push(DistanceConstraint {
                        a: idx,
                        b: idx + width as usize,
                        rest_length: spacing,
                        stiffness: 0.8,
                    });
                }
                if x + 1 < width && y + 1 < height {
                    let diag = spacing * std::f32::consts::SQRT_2;
                    constraints.push(DistanceConstraint {
                        a: idx,
                        b: idx + width as usize + 1,
                        rest_length: diag,
                        stiffness: 0.5,
                    });
                }
            }
        }

        Self {
            id,
            particles,
            constraints,
            width,
            height,
            wind: [0.0, 0.0, 0.0],
            gravity: [0.0, -9.81, 0.0],
        }
    }

    pub fn step(&mut self, dt: f32, iterations: u32) {
        if iterations == 0 {
            return;
        }

        for p in &mut self.particles {
            if p.inv_mass == 0.0 {
                continue;
            }
            let vel = [
                p.pos[0] - p.prev_pos[0],
                p.pos[1] - p.prev_pos[1],
                p.pos[2] - p.prev_pos[2],
            ];
            p.prev_pos = p.pos;
            p.pos[0] += vel[0] + (self.gravity[0] + self.wind[0]) * dt * dt;
            p.pos[1] += vel[1] + (self.gravity[1] + self.wind[1]) * dt * dt;
            p.pos[2] += vel[2] + (self.gravity[2] + self.wind[2]) * dt * dt;
        }

        for _ in 0..iterations {
            for c in &self.constraints {
                let pa = self.particles[c.a].pos;
                let pb = self.particles[c.b].pos;
                let dx = pb[0] - pa[0];
                let dy = pb[1] - pa[1];
                let dz = pb[2] - pa[2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.0001);
                let diff = (dist - c.rest_length) / dist;
                let w_sum = self.particles[c.a].inv_mass + self.particles[c.b].inv_mass;
                if w_sum == 0.0 {
                    continue;
                }
                let corr = diff * c.stiffness;
                let wa = self.particles[c.a].inv_mass / w_sum;
                let wb = self.particles[c.b].inv_mass / w_sum;
                self.particles[c.a].pos[0] += dx * corr * wa;
                self.particles[c.a].pos[1] += dy * corr * wa;
                self.particles[c.a].pos[2] += dz * corr * wa;
                self.particles[c.b].pos[0] -= dx * corr * wb;
                self.particles[c.b].pos[1] -= dy * corr * wb;
                self.particles[c.b].pos[2] -= dz * corr * wb;
            }
        }

        for p in &mut self.particles {
            if p.pos[1] < 0.0 {
                p.pos[1] = 0.0;
            }
        }
    }

    pub fn positions_flat(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.particles.len() * 3);
        for p in &self.particles {
            out.push(p.pos[0]);
            out.push(p.pos[1]);
            out.push(p.pos[2]);
        }
        out
    }
}

pub struct ClothWorld {
    pub cloths: Vec<ClothSim>,
    next_id: u32,
}

impl ClothWorld {
    pub fn new() -> Self {
        Self {
            cloths: Vec::new(),
            next_id: 0,
        }
    }

    pub fn add_cloth(&mut self, width: u32, height: u32, origin: [f32; 3], spacing: f32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.cloths.push(ClothSim::new(id, width, height, origin, spacing));
        id
    }

    pub fn update(&mut self, dt: f32, sim_level: SimulationLevel) {
        let iterations = crate::physics::sim_lod::PhysicsLod::from_sim_level(sim_level)
            .cloth_iterations();
        for cloth in &mut self.cloths {
            cloth.step(dt, iterations);
        }
    }
}
