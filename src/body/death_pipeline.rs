use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeathState {
    Alive,
    Dying,
    Dead,
    Corpse { decay_ticks: u32 },
    Skeleton,
    Removed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorpseState {
    pub entity: u64,
    pub position: [f32; 2],
    pub cause_of_death: String,
    pub lootable: bool,
    pub items_remaining: Vec<String>,
    pub decay_timer: f32,
    pub max_decay: f32,
}

impl CorpseState {
    pub fn new(entity: u64, position: [f32; 2], cause: &str) -> Self {
        Self {
            entity,
            position,
            cause_of_death: cause.to_string(),
            lootable: true,
            items_remaining: Vec::new(),
            decay_timer: 0.0,
            max_decay: 300.0,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.decay_timer += dt;
    }

    pub fn decay_progress(&self) -> f32 {
        (self.decay_timer / self.max_decay).min(1.0)
    }

    pub fn should_remove(&self) -> bool {
        self.decay_timer >= self.max_decay
    }

    pub fn loot_item(&mut self, index: usize) -> Option<String> {
        if index < self.items_remaining.len() {
            Some(self.items_remaining.remove(index))
        } else {
            None
        }
    }
}

pub struct CorpseManager {
    corpses: Vec<CorpseState>,
}

impl CorpseManager {
    pub fn new() -> Self {
        Self {
            corpses: Vec::new(),
        }
    }

    pub fn register_death(
        &mut self,
        entity: u64,
        position: [f32; 2],
        cause: &str,
        items: Vec<String>,
    ) {
        let mut corpse = CorpseState::new(entity, position, cause);
        corpse.items_remaining = items;
        self.corpses.push(corpse);
    }

    pub fn tick_all(&mut self, dt: f32) {
        for corpse in &mut self.corpses {
            corpse.tick(dt);
        }
        self.corpses.retain(|c| !c.should_remove());
    }

    pub fn corpses_near(&self, x: f32, y: f32, radius: f32) -> Vec<&CorpseState> {
        let r2 = radius * radius;
        self.corpses
            .iter()
            .filter(|c| {
                let dx = c.position[0] - x;
                let dy = c.position[1] - y;
                dx * dx + dy * dy <= r2
            })
            .collect()
    }

    pub fn corpse_count(&self) -> usize {
        self.corpses.len()
    }
}

impl Default for CorpseManager {
    fn default() -> Self {
        Self::new()
    }
}
