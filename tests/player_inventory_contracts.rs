//! Player Inventory Contracts
//! 
//! Tests for player inventory, equipment slots, and item management.
//! Ownership: Game Team
//! Lane: game_world_physics_audio
//! Type: Contract + Integration Tests
//! Speed: Medium

#[cfg(test)]
mod player_inventory_tests {
    use engene::game::player_save::{PlayerInventory, PlayerItem, PlayerItemType, PlayerSave};
    use engene::world::components::EquipmentSlots;

    #[test]
    fn player_inventory_new_creates_empty() {
        let inventory = PlayerInventory::new();
        assert_eq!(inventory.items().len(), 0);
        assert_eq!(inventory.capacity(), 20); // Default capacity
        assert!(inventory.is_empty());
    }

    #[test]
    fn player_inventory_add_item_success() {
        let mut inventory = PlayerInventory::new();
        let item = PlayerItem::new(PlayerItemType::Weapon, "Sword".to_string(), 1);
        
        let result = inventory.add_item(item);
        assert!(result.is_ok());
        assert_eq!(inventory.items().len(), 1);
        assert!(!inventory.is_empty());
    }

    #[test]
    fn player_inventory_add_item_fails_when_full() {
        let mut inventory = PlayerInventory::new_with_capacity(1);
        let item1 = PlayerItem::new(PlayerItemType::Weapon, "Sword".to_string(), 1);
        let item2 = PlayerItem::new(PlayerItemType::Armor, "Shield".to_string(), 1);
        
        // Add first item
        assert!(inventory.add_item(item1).is_ok());
        
        // Second item should fail
        assert!(inventory.add_item(item2).is_err());
        assert_eq!(inventory.items().len(), 1);
    }

    #[test]
    fn player_inventory_remove_item_success() {
        let mut inventory = PlayerInventory::new();
        let item = PlayerItem::new(PlayerItemType::Weapon, "Sword".to_string(), 1);
        let item_id = item.id;
        
        inventory.add_item(item).unwrap();
        let removed = inventory.remove_item(item_id);
        
        assert!(removed.is_some());
        assert_eq!(inventory.items().len(), 0);
        assert!(inventory.is_empty());
    }

    #[test]
    fn player_inventory_remove_item_fails_when_not_found() {
        let mut inventory = PlayerInventory::new();
        let fake_id = 999;
        
        let removed = inventory.remove_item(fake_id);
        assert!(removed.is_none());
        assert_eq!(inventory.items().len(), 0);
    }

    #[test]
    fn player_inventory_find_item_by_type() {
        let mut inventory = PlayerInventory::new();
        let weapon = PlayerItem::new(PlayerItemType::Weapon, "Sword".to_string(), 1);
        let armor = PlayerItem::new(PlayerItemType::Armor, "Shield".to_string(), 1);
        
        inventory.add_item(weapon).unwrap();
        inventory.add_item(armor).unwrap();
        
        let weapons = inventory.find_by_type(PlayerItemType::Weapon);
        assert_eq!(weapons.len(), 1);
        assert_eq!(weapons[0].item_type, PlayerItemType::Weapon);
        
        let armors = inventory.find_by_type(PlayerItemType::Armor);
        assert_eq!(armors.len(), 1);
        assert_eq!(armors[0].item_type, PlayerItemType::Armor);
    }

    #[test]
    fn player_inventory_find_item_by_name() {
        let mut inventory = PlayerInventory::new();
        let sword = PlayerItem::new(PlayerItemType::Weapon, "Excalibur".to_string(), 1);
        let dagger = PlayerItem::new(PlayerItemType::Weapon, "Dagger".to_string(), 1);
        
        inventory.add_item(sword).unwrap();
        inventory.add_item(dagger).unwrap();
        
        let excalibur = inventory.find_by_name("Excalibur");
        assert!(excalibur.is_some());
        assert_eq!(excalibur.unwrap().name, "Excalibur");
        
        let nonexistent = inventory.find_by_name("Nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn player_inventory_count_by_type() {
        let mut inventory = PlayerInventory::new();
        
        for i in 0..5 {
            let weapon = PlayerItem::new(PlayerItemType::Weapon, format!("Weapon{}", i), 1);
            inventory.add_item(weapon).unwrap();
        }
        
        for i in 0..3 {
            let armor = PlayerItem::new(PlayerItemType::Armor, format!("Armor{}", i), 1);
            inventory.add_item(armor).unwrap();
        }
        
        assert_eq!(inventory.count_by_type(PlayerItemType::Weapon), 5);
        assert_eq!(inventory.count_by_type(PlayerItemType::Armor), 3);
        assert_eq!(inventory.count_by_type(PlayerItemType::Consumable), 0);
    }

    #[test]
    fn player_inventory_total_weight_calculation() {
        let mut inventory = PlayerInventory::new();
        
        let heavy_item = PlayerItem::new_with_weight(PlayerItemType::Weapon, "Hammer".to_string(), 1, 10.0);
        let light_item = PlayerItem::new_with_weight(PlayerItemType::Consumable, "Potion".to_string(), 1, 0.5);
        
        inventory.add_item(heavy_item).unwrap();
        inventory.add_item(light_item).unwrap();
        
        assert_eq!(inventory.total_weight(), 10.5);
    }

    #[test]
    fn player_inventory_weight_limit_enforced() {
        let mut inventory = PlayerInventory::new_with_weight_limit(5.0);
        
        let light_item = PlayerItem::new_with_weight(PlayerItemType::Consumable, "Potion".to_string(), 1, 1.0);
        let heavy_item = PlayerItem::new_with_weight(PlayerItemType::Weapon, "Hammer".to_string(), 1, 10.0);
        
        // Light item should fit
        assert!(inventory.add_item(light_item).is_ok());
        assert_eq!(inventory.total_weight(), 1.0);
        
        // Heavy item should exceed weight limit
        assert!(inventory.add_item(heavy_item).is_err());
        assert_eq!(inventory.total_weight(), 1.0);
    }

    #[test]
    fn player_inventory_stack_management() {
        let mut inventory = PlayerInventory::new();
        
        // Add stackable items
        let potion1 = PlayerItem::new_stackable(PlayerItemType::Consumable, "Health Potion".to_string(), 5);
        let potion2 = PlayerItem::new_stackable(PlayerItemType::Consumable, "Health Potion".to_string(), 3);
        
        inventory.add_item(potion1).unwrap();
        inventory.add_item(potion2).unwrap();
        
        // Should stack into single item
        let potions = inventory.find_by_name("Health Potion");
        assert!(potions.is_some());
        assert_eq!(potions.unwrap().quantity, 8);
        
        assert_eq!(inventory.items().len(), 1);
    }

    #[test]
    fn player_inventory_stack_split() {
        let mut inventory = PlayerInventory::new();
        let stack = PlayerItem::new_stackable(PlayerItemType::Consumable, "Arrows".to_string(), 20);
        let stack_id = stack.id;
        
        inventory.add_item(stack).unwrap();
        
        let split_result = inventory.split_stack(stack_id, 5);
        assert!(split_result.is_ok());
        
        let arrows = inventory.find_by_name("Arrows");
        assert_eq!(arrows.unwrap().quantity, 15); // Original stack reduced
        
        // Should have new stack
        assert_eq!(inventory.items().len(), 2);
    }

    #[test]
    fn player_inventory_merge_stacks() {
        let mut inventory = PlayerInventory::new();
        
        let stack1 = PlayerItem::new_stackable(PlayerItemType::Consumable, "Bullets".to_string(), 10);
        let stack2 = PlayerItem::new_stackable(PlayerItemType::Consumable, "Bullets".to_string(), 15);
        
        inventory.add_item(stack1).unwrap();
        inventory.add_item(stack2).unwrap();
        
        // Should automatically merge
        let bullets = inventory.find_by_name("Bullets");
        assert_eq!(bullets.unwrap().quantity, 25);
        assert_eq!(inventory.items().len(), 1);
    }

    #[test]
    fn player_inventory_performance_many_items() {
        let mut inventory = PlayerInventory::new_with_capacity(1000);
        let start = std::time::Instant::now();
        
        // Add 1000 items
        for i in 0..1000 {
            let item = PlayerItem::new(
                if i % 3 == 0 { PlayerItemType::Weapon } 
                else if i % 3 == 1 { PlayerItemType::Armor } 
                else { PlayerItemType::Consumable },
                format!("Item{}", i),
                1
            );
            inventory.add_item(item).unwrap();
        }
        
        let add_duration = start.elapsed();
        
        // Search operations
        let search_start = std::time::Instant::now();
        let weapons = inventory.find_by_type(PlayerItemType::Weapon);
        let armors = inventory.find_by_type(PlayerItemType::Armor);
        let consumables = inventory.find_by_type(PlayerItemType::Consumable);
        let search_duration = search_start.elapsed();
        
        assert_eq!(inventory.items().len(), 1000);
        assert_eq!(weapons.len(), 334);
        assert_eq!(armors.len(), 333);
        assert_eq!(consumables.len(), 333);
        
        assert!(add_duration.as_millis() < 50, "Adding 1000 items should be fast");
        assert!(search_duration.as_millis() < 10, "Searching should be very fast");
    }
}

#[cfg(test)]
mod equipment_slots_tests {
    use engene::world::components::EquipmentSlots;
    use engene::game::player_save::PlayerItem;

    #[test]
    fn equipment_slots_new_creates_empty() {
        let slots = EquipmentSlots::new();
        assert!(slots.is_slot_empty(EquipmentSlotType::Head));
        assert!(slots.is_slot_empty(EquipmentSlotType::Chest));
        assert!(slots.is_slot_empty(EquipmentSlotType::Weapon));
    }

    #[test]
    fn equipment_slots_equip_item_success() {
        let mut slots = EquipmentSlots::new();
        let weapon = PlayerItem::new(PlayerItemType::Weapon, "Sword".to_string(), 1);
        
        let result = slots.equip_item(EquipmentSlotType::Weapon, weapon);
        assert!(result.is_ok());
        assert!(!slots.is_slot_empty(EquipmentSlotType::Weapon));
    }

    #[test]
    fn equipment_slots_equip_item_fails_wrong_type() {
        let mut slots = EquipmentSlots::new();
        let armor = PlayerItem::new(PlayerItemType::Armor, "Helmet".to_string(), 1);
        
        // Try to equip armor in weapon slot
        let result = slots.equip_item(EquipmentSlotType::Weapon, armor);
        assert!(result.is_err());
        assert!(slots.is_slot_empty(EquipmentSlotType::Weapon));
    }

    #[test]
    fn equipment_slots_equip_item_fails_slot_occupied() {
        let mut slots = EquipmentSlots::new();
        let weapon1 = PlayerItem::new(PlayerItemType::Weapon, "Sword".to_string(), 1);
        let weapon2 = PlayerItem::new(PlayerItemType::Weapon, "Axe".to_string(), 1);
        
        // Equip first weapon
        assert!(slots.equip_item(EquipmentSlotType::Weapon, weapon1).is_ok());
        
        // Try to equip second weapon in same slot
        let result = slots.equip_item(EquipmentSlotType::Weapon, weapon2);
        assert!(result.is_err());
    }

    #[test]
    fn equipment_slots_unequip_item_success() {
        let mut slots = EquipmentSlots::new();
        let weapon = PlayerItem::new(PlayerItemType::Weapon, "Sword".to_string(), 1);
        let weapon_id = weapon.id;
        
        slots.equip_item(EquipmentSlotType::Weapon, weapon).unwrap();
        let unequipped = slots.unequip_item(EquipmentSlotType::Weapon);
        
        assert!(unequipped.is_some());
        assert_eq!(unequipped.unwrap().id, weapon_id);
        assert!(slots.is_slot_empty(EquipmentSlotType::Weapon));
    }

    #[test]
    fn equipment_slots_unequip_item_fails_when_empty() {
        let mut slots = EquipmentSlots::new();
        
        let unequipped = slots.unequip_item(EquipmentSlotType::Weapon);
        assert!(unequipped.is_none());
    }

    #[test]
    fn equipment_slots_swap_items() {
        let mut slots = EquipmentSlots::new();
        let weapon1 = PlayerItem::new(PlayerItemType::Weapon, "Sword".to_string(), 1);
        let weapon2 = PlayerItem::new(PlayerItemType::Weapon, "Axe".to_string(), 1);
        
        // Equip first weapon
        slots.equip_item(EquipmentSlotType::Weapon, weapon1).unwrap();
        
        // Swap with second weapon
        let old_item = slots.equip_item(EquipmentSlotType::Weapon, weapon2).unwrap();
        
        assert!(old_item.is_some());
        assert_eq!(old_item.unwrap().name, "Sword");
        assert!(!slots.is_slot_empty(EquipmentSlotType::Weapon));
        
        let current = slots.get_item(EquipmentSlotType::Weapon);
        assert!(current.is_some());
        assert_eq!(current.unwrap().name, "Axe");
    }

    #[test]
    fn equipment_slots_get_equipped_items() {
        let mut slots = EquipmentSlots::new();
        let helmet = PlayerItem::new(PlayerItemType::Armor, "Helmet".to_string(), 1);
        let chestplate = PlayerItem::new(PlayerItemType::Armor, "Chestplate".to_string(), 1);
        let sword = PlayerItem::new(PlayerItemType::Weapon, "Sword".to_string(), 1);
        
        slots.equip_item(EquipmentSlotType::Head, helmet).unwrap();
        slots.equip_item(EquipmentSlotType::Chest, chestplate).unwrap();
        slots.equip_item(EquipmentSlotType::Weapon, sword).unwrap();
        
        let equipped = slots.get_all_equipped();
        assert_eq!(equipped.len(), 3);
        
        let weapons = equipped.iter().filter(|item| item.item_type == PlayerItemType::Weapon).count();
        let armors = equipped.iter().filter(|item| item.item_type == PlayerItemType::Armor).count();
        
        assert_eq!(weapons, 1);
        assert_eq!(armors, 2);
    }

    #[test]
    fn equipment_slots_clear_all() {
        let mut slots = EquipmentSlots::new();
        let items = vec![
            PlayerItem::new(PlayerItemType::Armor, "Helmet".to_string(), 1),
            PlayerItem::new(PlayerItemType::Armor, "Chestplate".to_string(), 1),
            PlayerItem::new(PlayerItemType::Weapon, "Sword".to_string(), 1),
        ];
        
        slots.equip_item(EquipmentSlotType::Head, items[0].clone()).unwrap();
        slots.equip_item(EquipmentSlotType::Chest, items[1].clone()).unwrap();
        slots.equip_item(EquipmentSlotType::Weapon, items[2].clone()).unwrap();
        
        let cleared = slots.clear_all();
        assert_eq!(cleared.len(), 3);
        
        assert!(slots.is_slot_empty(EquipmentSlotType::Head));
        assert!(slots.is_slot_empty(EquipmentSlotType::Chest));
        assert!(slots.is_slot_empty(EquipmentSlotType::Weapon));
    }

    #[test]
    fn equipment_slots_validation() {
        let mut slots = EquipmentSlots::new();
        let weapon = PlayerItem::new(PlayerItemType::Weapon, "Sword".to_string(), 1);
        
        // Valid equipment
        assert!(slots.can_equip(EquipmentSlotType::Weapon, &weapon));
        
        // Invalid equipment (wrong type)
        let armor = PlayerItem::new(PlayerItemType::Armor, "Helmet".to_string(), 1);
        assert!(!slots.can_equip(EquipmentSlotType::Weapon, &armor));
        
        // Slot occupied
        slots.equip_item(EquipmentSlotType::Weapon, weapon).unwrap();
        let weapon2 = PlayerItem::new(PlayerItemType::Weapon, "Axe".to_string(), 1);
        assert!(!slots.can_equip(EquipmentSlotType::Weapon, &weapon2));
    }
}

// Mock implementations
#[derive(Debug, Clone, Copy, PartialEq)]
enum EquipmentSlotType {
    Head,
    Chest,
    Weapon,
    Legs,
    Boots,
}

#[derive(Debug, Clone, PartialEq)]
enum PlayerItemType {
    Weapon,
    Armor,
    Consumable,
}

#[derive(Debug, Clone)]
struct PlayerItem {
    id: u32,
    item_type: PlayerItemType,
    name: String,
    quantity: u32,
    weight: f32,
    stackable: bool,
}

impl PlayerItem {
    fn new(item_type: PlayerItemType, name: String, quantity: u32) -> Self {
        Self {
            id: rand::random::<u32>(),
            item_type,
            name,
            quantity,
            weight: 1.0,
            stackable: false,
        }
    }
    
    fn new_with_weight(item_type: PlayerItemType, name: String, quantity: u32, weight: f32) -> Self {
        Self {
            id: rand::random::<u32>(),
            item_type,
            name,
            quantity,
            weight,
            stackable: false,
        }
    }
    
    fn new_stackable(item_type: PlayerItemType, name: String, quantity: u32) -> Self {
        Self {
            id: rand::random::<u32>(),
            item_type,
            name,
            quantity,
            weight: 0.1,
            stackable: true,
        }
    }
}

struct PlayerInventory {
    items: Vec<PlayerItem>,
    capacity: usize,
    weight_limit: f32,
}

impl PlayerInventory {
    fn new() -> Self {
        Self::new_with_capacity(20)
    }
    
    fn new_with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::new(),
            capacity,
            weight_limit: f32::INFINITY,
        }
    }
    
    fn new_with_weight_limit(weight_limit: f32) -> Self {
        Self {
            items: Vec::new(),
            capacity: 20,
            weight_limit,
        }
    }
    
    fn add_item(&mut self, item: PlayerItem) -> Result<(), String> {
        if self.items.len() >= self.capacity {
            return Err("Inventory full".to_string());
        }
        
        if self.total_weight() + item.weight > self.weight_limit {
            return Err("Weight limit exceeded".to_string());
        }
        
        // Try to stack with existing items
        if item.stackable {
            if let Some(existing) = self.items.iter_mut().find(|i| i.name == item.name && i.item_type == item.item_type) {
                existing.quantity += item.quantity;
                return Ok(());
            }
        }
        
        self.items.push(item);
        Ok(())
    }
    
    fn remove_item(&mut self, item_id: u32) -> Option<PlayerItem> {
        let index = self.items.iter().position(|item| item.id == item_id)?;
        Some(self.items.remove(index))
    }
    
    fn find_by_type(&self, item_type: PlayerItemType) -> Vec<&PlayerItem> {
        self.items.iter().filter(|item| item.item_type == item_type).collect()
    }
    
    fn find_by_name(&self, name: &str) -> Option<&PlayerItem> {
        self.items.iter().find(|item| item.name == name)
    }
    
    fn count_by_type(&self, item_type: PlayerItemType) -> usize {
        self.items.iter().filter(|item| item.item_type == item_type).count()
    }
    
    fn total_weight(&self) -> f32 {
        self.items.iter().map(|item| item.weight * item.quantity as f32).sum()
    }
    
    fn items(&self) -> &[PlayerItem] {
        &self.items
    }
    
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    fn split_stack(&mut self, item_id: u32, quantity: u32) -> Result<(), String> {
        if self.items.len() >= self.capacity {
            return Err("Inventory full".to_string());
        }
        
        let item = self.items.iter_mut().find(|i| i.id == item_id && i.stackable);
        if let Some(item) = item {
            if item.quantity <= quantity {
                return Err("Insufficient quantity".to_string());
            }
            
            item.quantity -= quantity;
            
            let new_stack = PlayerItem {
                id: rand::random::<u32>(),
                item_type: item.item_type,
                name: item.name.clone(),
                quantity,
                weight: item.weight,
                stackable: true,
            };
            
            self.items.push(new_stack);
            Ok(())
        } else {
            Err("Item not found or not stackable".to_string())
        }
    }
}

struct EquipmentSlots {
    slots: std::collections::HashMap<EquipmentSlotType, Option<PlayerItem>>,
}

impl EquipmentSlots {
    fn new() -> Self {
        let mut slots = std::collections::HashMap::new();
        slots.insert(EquipmentSlotType::Head, None);
        slots.insert(EquipmentSlotType::Chest, None);
        slots.insert(EquipmentSlotType::Weapon, None);
        slots.insert(EquipmentSlotType::Legs, None);
        slots.insert(EquipmentSlotType::Boots, None);
        Self { slots }
    }
    
    fn equip_item(&mut self, slot_type: EquipmentSlotType, item: PlayerItem) -> Result<Option<PlayerItem>, String> {
        if !self.can_equip(slot_type, &item) {
            return Err("Cannot equip item".to_string());
        }
        
        let old_item = self.slots.insert(slot_type, Some(item)).unwrap_or(None);
        Ok(old_item)
    }
    
    fn unequip_item(&mut self, slot_type: EquipmentSlotType) -> Option<PlayerItem> {
        self.slots.insert(slot_type, None).unwrap_or(None)
    }
    
    fn get_item(&self, slot_type: EquipmentSlotType) -> Option<&PlayerItem> {
        self.slots.get(&slot_type).and_then(|item| item.as_ref())
    }
    
    fn get_all_equipped(&self) -> Vec<&PlayerItem> {
        self.slots.values().filter_map(|item| item.as_ref()).collect()
    }
    
    fn clear_all(&mut self) -> Vec<PlayerItem> {
        let mut cleared = Vec::new();
        for item in self.slots.values_mut() {
            if let Some(removed) = item.take() {
                cleared.push(removed);
            }
        }
        cleared
    }
    
    fn is_slot_empty(&self, slot_type: EquipmentSlotType) -> bool {
        self.slots.get(&slot_type).map(|item| item.is_none()).unwrap_or(true)
    }
    
    fn can_equip(&self, slot_type: EquipmentSlotType, item: &PlayerItem) -> bool {
        // Check if slot is occupied
        if !self.is_slot_empty(slot_type) {
            return false;
        }
        
        // Check item type compatibility (simplified)
        match (slot_type, item.item_type) {
            (EquipmentSlotType::Weapon, PlayerItemType::Weapon) => true,
            (EquipmentSlotType::Head, PlayerItemType::Armor) => true,
            (EquipmentSlotType::Chest, PlayerItemType::Armor) => true,
            (EquipmentSlotType::Legs, PlayerItemType::Armor) => true,
            (EquipmentSlotType::Boots, PlayerItemType::Armor) => true,
            _ => false,
        }
    }
}

// Mock rand for testing
mod rand {
    use std::sync::atomic::{AtomicU32, Ordering};
    
    static COUNTER: AtomicU32 = AtomicU32::new(1);
    
    pub fn random<T>() -> T 
    where 
        T: From<u32>
    {
        T::from(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}
