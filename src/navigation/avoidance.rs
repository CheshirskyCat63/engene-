use crate::core::ecs::Entity;

const NEIGHBOR_RADIUS: f32 = 5.0;
const TIME_HORIZON: f32 = 2.0;
const MAX_SPEED: f32 = 3.0;
const AGENT_RADIUS: f32 = 0.5;

pub struct RvoAgent {
    pub entity: Entity,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub preferred_velocity: [f32; 2],
    pub radius: f32,
}

pub struct RvoSystem {
    agents: Vec<RvoAgent>,
}

impl RvoSystem {
    pub fn new() -> Self {
        Self { agents: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.agents.clear();
    }

    pub fn add_agent(&mut self, entity: Entity, pos: [f32; 2], vel: [f32; 2], pref_vel: [f32; 2]) {
        self.agents.push(RvoAgent {
            entity,
            position: pos,
            velocity: vel,
            preferred_velocity: pref_vel,
            radius: AGENT_RADIUS,
        });
    }

    pub fn compute(&mut self) {
        let n = self.agents.len();
        let mut new_vels = vec![[0.0f32; 2]; n];

        for i in 0..n {
            let mut avoidance = [0.0f32; 2];
            let mut count = 0u32;

            for j in 0..n {
                if i == j {
                    continue;
                }

                let dx = self.agents[j].position[0] - self.agents[i].position[0];
                let dy = self.agents[j].position[1] - self.agents[i].position[1];
                let dist = (dx * dx + dy * dy).sqrt();

                if dist > NEIGHBOR_RADIUS || dist < 0.001 {
                    continue;
                }

                let combined_radius = self.agents[i].radius + self.agents[j].radius;
                let overlap = combined_radius - dist;

                if overlap > 0.0 {
                    let nx = -dx / dist;
                    let ny = -dy / dist;
                    let push = overlap / TIME_HORIZON;
                    avoidance[0] += nx * push;
                    avoidance[1] += ny * push;
                    count += 1;
                } else {
                    let rel_vel_x = self.agents[i].velocity[0] - self.agents[j].velocity[0];
                    let rel_vel_y = self.agents[i].velocity[1] - self.agents[j].velocity[1];

                    let t_closest = -(dx * rel_vel_x + dy * rel_vel_y)
                        / (rel_vel_x * rel_vel_x + rel_vel_y * rel_vel_y + 0.001);
                    let t_closest = t_closest.clamp(0.0, TIME_HORIZON);

                    let closest_dist_sq = {
                        let cx = dx + rel_vel_x * t_closest;
                        let cy = dy + rel_vel_y * t_closest;
                        cx * cx + cy * cy
                    };

                    if closest_dist_sq < combined_radius * combined_radius {
                        let nx = -dx / dist;
                        let ny = -dy / dist;
                        let urgency = 1.0 / (t_closest + 0.1);
                        avoidance[0] += nx * urgency * 0.5;
                        avoidance[1] += ny * urgency * 0.5;
                        count += 1;
                    }
                }
            }

            let pref = self.agents[i].preferred_velocity;
            if count > 0 {
                let scale = 1.0 / count as f32;
                new_vels[i][0] = pref[0] + avoidance[0] * scale;
                new_vels[i][1] = pref[1] + avoidance[1] * scale;
            } else {
                new_vels[i] = pref;
            }

            let speed = (new_vels[i][0] * new_vels[i][0] + new_vels[i][1] * new_vels[i][1]).sqrt();
            if speed > MAX_SPEED {
                let s = MAX_SPEED / speed;
                new_vels[i][0] *= s;
                new_vels[i][1] *= s;
            }
        }

        for (i, agent) in self.agents.iter_mut().enumerate() {
            agent.velocity = new_vels[i];
        }
    }

    pub fn get_velocity(&self, entity: Entity) -> Option<[f32; 2]> {
        self.agents
            .iter()
            .find(|a| a.entity == entity)
            .map(|a| a.velocity)
    }

    pub fn agents(&self) -> &[RvoAgent] {
        &self.agents
    }
}
