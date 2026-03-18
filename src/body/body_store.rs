use crate::body::anatomy::BodyState;
use crate::core::ecs::Entity;

pub struct BodyHandle {
    pub store_id: u32,
    pub aggregate_health: f32,
    pub is_alive: bool,
    pub has_severed_limb: bool,
    pub bleed_rate: f32,
}

pub struct BodyStateStore {
    bodies: Vec<BodyState>,
    free_ids: Vec<u32>,
}

impl BodyStateStore {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            free_ids: Vec::new(),
        }
    }

    pub fn allocate(&mut self, body: BodyState) -> BodyHandle {
        let id = if let Some(free) = self.free_ids.pop() {
            self.bodies[free as usize] = body;
            free
        } else {
            let id = self.bodies.len() as u32;
            self.bodies.push(body);
            id
        };

        let b = &self.bodies[id as usize];
        BodyHandle {
            store_id: id,
            aggregate_health: b.aggregate_health(),
            is_alive: b.is_alive(),
            has_severed_limb: false,
            bleed_rate: 0.0,
        }
    }

    pub fn get(&self, id: u32) -> Option<&BodyState> {
        self.bodies.get(id as usize)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut BodyState> {
        self.bodies.get_mut(id as usize)
    }

    pub fn get_mut_by_entity(&mut self, entity: Entity) -> Option<&mut BodyState> {
        self.bodies.iter_mut().find(|b| b.entity == entity)
    }

    /// Returns store_id for an entity if it has an allocated body.
    pub fn store_id_for_entity(&self, entity: Entity) -> Option<u32> {
        for (i, b) in self.bodies.iter().enumerate() {
            if !self.free_ids.contains(&(i as u32)) && b.entity == entity {
                return Some(i as u32);
            }
        }
        None
    }

    pub fn free(&mut self, id: u32) {
        self.free_ids.push(id);
    }

    pub fn tick(&mut self, dt: f32) {
        for body in &mut self.bodies {
            let mut total_bleed = 0.0_f32;
            for bp in &mut body.bleed_points {
                bp.time_active += dt;
                let effective_rate = bp.rate * (1.0 - (bp.time_active * 0.01).min(0.5));
                total_bleed += effective_rate;
            }
            body.blood_level = (body.blood_level - total_bleed * dt * 0.01).max(0.0);
            body.pain = (body.pain - dt * 0.02).max(0.0);
            body.bleed_points
                .retain(|bp| bp.rate > 0.001 && bp.time_active < 300.0);
        }
    }

    pub fn len(&self) -> usize {
        self.bodies.len() - self.free_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
