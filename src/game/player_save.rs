use serde::{Deserialize, Serialize};

use crate::game::player::{CameraMode, PlayerController, PlayerState};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerSave {
    pub position: [f32; 3],
    pub rotation: [f32; 2],
    pub health: f32,
    pub max_health: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub bleeding: f32,
    pub camera_mode: CameraMode,
    pub inventory: Vec<PlayerItem>,
    pub equipped_weapon: Option<String>,
    pub money: f32,
    pub active_quests: Vec<u32>,
    pub completed_quests: Vec<u32>,
    pub faction_reputation: Vec<(String, f32)>,
    pub play_time_seconds: f64,
    pub save_day: u32,
    pub save_month: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerItem {
    pub name: String,
    pub quantity: u32,
    pub weight: f32,
    pub item_type: PlayerItemType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerItemType {
    Weapon,
    Ammo,
    Medkit,
    Food,
    Artifact,
    Armor,
    Junk,
    Tool,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerInventory {
    pub items: Vec<PlayerItem>,
    pub money: f32,
    pub equipped_weapon: Option<String>,
    pub weight_carried: f32,
    pub max_weight: f32,
}

impl PlayerInventory {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            money: 500.0,
            equipped_weapon: None,
            weight_carried: 0.0,
            max_weight: 50.0,
        }
    }

    pub fn add_item(&mut self, item: PlayerItem) -> bool {
        let new_weight = self.weight_carried + item.weight * item.quantity as f32;
        if new_weight > self.max_weight {
            return false;
        }
        self.weight_carried = new_weight;
        if let Some(existing) = self.items.iter_mut().find(|i| i.name == item.name) {
            existing.quantity += item.quantity;
        } else {
            self.items.push(item);
        }
        true
    }

    pub fn remove_item(&mut self, name: &str, count: u32) -> bool {
        if let Some(idx) = self.items.iter().position(|i| i.name == name) {
            if self.items[idx].quantity < count {
                return false;
            }
            let weight = self.items[idx].weight * count as f32;
            self.items[idx].quantity -= count;
            self.weight_carried = (self.weight_carried - weight).max(0.0);
            if self.items[idx].quantity == 0 {
                self.items.remove(idx);
            }
            true
        } else {
            false
        }
    }

    pub fn has_item(&self, name: &str) -> bool {
        self.items.iter().any(|i| i.name == name && i.quantity > 0)
    }

    pub fn item_count(&self, name: &str) -> u32 {
        self.items.iter().find(|i| i.name == name).map(|i| i.quantity).unwrap_or(0)
    }
}

impl PlayerSave {
    pub fn from_state(
        controller: &PlayerController,
        inventory: &PlayerInventory,
        active_quests: Vec<u32>,
        completed_quests: Vec<u32>,
        play_time: f64,
        day: u32,
        month: u32,
    ) -> Self {
        Self {
            position: controller.position,
            rotation: controller.rotation,
            health: controller.health,
            max_health: controller.max_health,
            stamina: controller.stamina,
            max_stamina: controller.max_stamina,
            bleeding: controller.bleeding,
            camera_mode: controller.camera_mode.clone(),
            inventory: inventory.items.clone(),
            equipped_weapon: inventory.equipped_weapon.clone(),
            money: inventory.money,
            active_quests,
            completed_quests,
            faction_reputation: Vec::new(),
            play_time_seconds: play_time,
            save_day: day,
            save_month: month,
        }
    }

    pub fn restore_controller(&self) -> PlayerController {
        PlayerController {
            position: self.position,
            rotation: self.rotation,
            health: self.health,
            max_health: self.max_health,
            stamina: self.stamina,
            max_stamina: self.max_stamina,
            bleeding: self.bleeding,
            camera_mode: self.camera_mode.clone(),
            state: if self.health > 0.0 { PlayerState::Alive } else { PlayerState::Dead },
            move_speed: 4.0,
            sprint_speed: 7.0,
            interaction_range: 3.0,
            weight_carried: self.inventory.iter().map(|i| i.weight * i.quantity as f32).sum(),
            max_weight: 50.0,
            weapon_slots: [
                Some("Makarov".to_string()),
                Some("AK74".to_string()),
                Some("Shotgun".to_string()),
                Some("Grenade".to_string()),
            ],
            current_weapon_slot: 0,
            fire_cooldown: 0.0,
        }
    }

    pub fn restore_inventory(&self) -> PlayerInventory {
        let weight = self.inventory.iter().map(|i| i.weight * i.quantity as f32).sum();
        PlayerInventory {
            items: self.inventory.clone(),
            money: self.money,
            equipped_weapon: self.equipped_weapon.clone(),
            weight_carried: weight,
            max_weight: 50.0,
        }
    }

    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let data = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, data)
    }

    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        ron::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}
