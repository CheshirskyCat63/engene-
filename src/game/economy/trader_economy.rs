use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraderInventorySlot {
    pub item_id: String,
    pub quantity: u32,
    pub buy_price: f32,
    pub sell_price: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraderState {
    pub name: String,
    pub faction: String,
    pub capital: f32,
    pub inventory: Vec<TraderInventorySlot>,
    pub reputation_modifier: f32,
    pub restock_timer: f32,
    pub restock_interval: f32,
    pub demand_modifiers: std::collections::HashMap<String, f32>,
}

impl TraderState {
    pub fn new(name: &str, faction: &str, capital: f32) -> Self {
        Self {
            name: name.to_string(),
            faction: faction.to_string(),
            capital,
            inventory: Vec::new(),
            reputation_modifier: 1.0,
            restock_timer: 0.0,
            restock_interval: 48.0,
            demand_modifiers: std::collections::HashMap::new(),
        }
    }

    pub fn effective_buy_price(&self, base_value: f32, item_id: &str) -> f32 {
        let demand = self.demand_modifiers.get(item_id).copied().unwrap_or(1.0);
        base_value * 0.5 * demand * self.reputation_modifier
    }

    pub fn effective_sell_price(&self, base_value: f32, item_id: &str) -> f32 {
        let demand = self.demand_modifiers.get(item_id).copied().unwrap_or(1.0);
        base_value * 1.2 * demand / self.reputation_modifier
    }

    pub fn tick_restock(&mut self, delta_hours: f32) {
        self.restock_timer += delta_hours;
        if self.restock_timer >= self.restock_interval {
            self.restock_timer = 0.0;
            self.restock();
        }
    }

    fn restock(&mut self) {
        for slot in &mut self.inventory {
            slot.quantity = (slot.quantity + 2).min(20);
        }
    }

    pub fn can_buy(&self, item_id: &str) -> Option<&TraderInventorySlot> {
        self.inventory
            .iter()
            .find(|s| s.item_id == item_id && s.quantity > 0)
    }
}
