use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampState {
    pub name: String,
    pub faction: String,
    pub population: u32,
    pub food_supply: f32,
    pub security_level: f32,
    pub mood: f32,
    pub danger_memory: f32,
    pub trade_volume: f32,
    pub services: Vec<CampService>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CampService {
    Trader {
        name: String,
        stock_value: f32,
    },
    Mechanic {
        repair_quality: f32,
    },
    Medic {
        heal_quality: f32,
    },
    BarKeep {
        morale_boost: f32,
    },
}

impl CampState {
    pub fn new(name: &str, faction: &str, population: u32) -> Self {
        Self {
            name: name.to_string(),
            faction: faction.to_string(),
            population,
            food_supply: 1.0,
            security_level: 0.6,
            mood: 0.5,
            danger_memory: 0.0,
            trade_volume: 0.0,
            services: Vec::new(),
        }
    }

    pub fn tick_daily(&mut self) {
        let food_consumption = self.population as f32 * 0.1;
        self.food_supply = (self.food_supply - food_consumption / 100.0).max(0.0);

        if self.food_supply < 0.3 {
            self.mood = (self.mood - 0.05).max(0.0);
        }

        self.danger_memory = (self.danger_memory * 0.95).max(0.0);

        self.mood = (self.mood * 0.98 + 0.01).clamp(0.0, 1.0);
        if self.security_level > 0.7 {
            self.mood = (self.mood + 0.01).min(1.0);
        }
    }

    pub fn report_danger(&mut self, severity: f32) {
        self.danger_memory = (self.danger_memory + severity).min(1.0);
        self.mood = (self.mood - severity * 0.2).max(0.0);
    }

    pub fn resupply(&mut self, food: f32) {
        self.food_supply = (self.food_supply + food).min(2.0);
    }

    pub fn is_safe(&self) -> bool {
        self.security_level > 0.5 && self.danger_memory < 0.3
    }

    pub fn pressure_level(&self) -> f32 {
        let food_pressure = (1.0 - self.food_supply).max(0.0);
        let danger_pressure = self.danger_memory;
        let morale_pressure = (1.0 - self.mood).max(0.0);
        (food_pressure + danger_pressure + morale_pressure) / 3.0
    }
}
