//! Equipment component for NPCs.

use serde::{Deserialize, Serialize};

/// Equipment carried by NPCs. Degrades per combat/time, must be repaired at trader.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EquipmentSlots {
    pub weapon_condition: f32,
    pub armor_condition: f32,
    pub medkits: u32,
    pub food_rations: u32,
    pub ammo: u32,
    pub total_weight: f32,
}

impl EquipmentSlots {
    pub fn default_stalker() -> Self {
        Self {
            weapon_condition: 0.8,
            armor_condition: 0.7,
            medkits: 2,
            food_rations: 3,
            ammo: 30,
            total_weight: 15.0,
        }
    }

    pub fn empty() -> Self {
        Self {
            weapon_condition: 0.0,
            armor_condition: 0.0,
            medkits: 0,
            food_rations: 0,
            ammo: 0,
            total_weight: 0.0,
        }
    }

    pub fn combat_effectiveness(&self) -> f32 {
        let weapon = if self.weapon_condition > 0.1 && self.ammo > 0 {
            self.weapon_condition
        } else {
            0.2
        };
        let armor = self.armor_condition * 0.5 + 0.5;
        weapon * armor
    }

    pub fn degrade_combat(&mut self) {
        self.weapon_condition = (self.weapon_condition - 0.05).max(0.0);
        self.armor_condition = (self.armor_condition - 0.03).max(0.0);
        if self.ammo > 0 {
            self.ammo = self.ammo.saturating_sub(5);
        }
    }

    pub fn degrade_time(&mut self, dt: f32) {
        self.weapon_condition = (self.weapon_condition - dt * 0.001).max(0.0);
        self.armor_condition = (self.armor_condition - dt * 0.0005).max(0.0);
    }

    pub fn repair_cost(&self) -> f32 {
        let weapon_repair = (1.0 - self.weapon_condition) * 30.0;
        let armor_repair = (1.0 - self.armor_condition) * 20.0;
        weapon_repair + armor_repair
    }

    pub fn needs_repair(&self) -> bool {
        self.weapon_condition < 0.3 || self.armor_condition < 0.3
    }

    pub fn needs_supplies(&self) -> bool {
        self.medkits == 0 || self.food_rations < 2 || self.ammo < 10
    }

    pub fn use_medkit(&mut self) -> bool {
        if self.medkits > 0 {
            self.medkits -= 1;
            true
        } else {
            false
        }
    }

    pub fn eat_ration(&mut self) -> bool {
        if self.food_rations > 0 {
            self.food_rations -= 1;
            true
        } else {
            false
        }
    }
}

impl Default for EquipmentSlots {
    fn default() -> Self {
        Self::default_stalker()
    }
}
