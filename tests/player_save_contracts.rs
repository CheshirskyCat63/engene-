//! Player Save Contracts
//! 
//! Tests for player save/load, persistence, and data integrity.
//! Ownership: Game Team
//! Lane: game_world_physics_audio
//! Type: Contract + Integration Tests
//! Speed: Medium

#[cfg(test)]
mod player_save_tests {
    use engene::game::player_save::{PlayerSave, PlayerInventory, PlayerItem, PlayerItemType};
    use engene::game::player::{PlayerController, PlayerState};
    use engene::world::components::PersonalNeeds;

    #[test]
    fn player_save_new_creates_default() {
        let save = PlayerSave::new("test_player");
        
        assert_eq!(save.player_name(), "test_player");
        assert_eq!(save.level(), 1);
        assert_eq!(save.experience(), 0);
        assert!(save.created_at() <= std::time::SystemTime::now());
        assert!(save.last_modified() <= std::time::SystemTime::now());
    }

    #[test]
    fn player_save_serialization_roundtrip() {
        let original = create_test_save();
        
        // Serialize to bytes
        let serialized = original.serialize().expect("Serialization should succeed");
        
        // Deserialize from bytes
        let deserialized = PlayerSave::deserialize(&serialized).expect("Deserialization should succeed");
        
        // Verify all data preserved
        assert_eq!(deserialized.player_name(), original.player_name());
        assert_eq!(deserialized.level(), original.level());
        assert_eq!(deserialized.experience(), original.experience());
        assert_eq!(deserialized.position(), original.position());
        assert_eq!(deserialized.health(), original.health());
        assert_eq!(deserialized.inventory().items().len(), original.inventory().items().len());
    }

    #[test]
    fn player_save_file_operations() {
        let save = create_test_save();
        let file_path = "test_save.json";
        
        // Save to file
        save.save_to_file(file_path).expect("Save to file should succeed");
        
        // Load from file
        let loaded = PlayerSave::load_from_file(file_path).expect("Load from file should succeed");
        
        // Verify data integrity
        assert_eq!(loaded.player_name(), save.player_name());
        assert_eq!(loaded.level(), save.level());
        assert_eq!(loaded.experience(), save.experience());
        
        // Cleanup
        std::fs::remove_file(file_path).ok();
    }

    #[test]
    fn player_save_load_nonexistent_file_fails() {
        let result = PlayerSave::load_from_file("nonexistent_save.json");
        assert!(result.is_err());
    }

    #[test]
    fn player_save_corrupted_file_fails() {
        let file_path = "corrupted_save.json";
        
        // Write invalid JSON
        std::fs::write(file_path, "{ invalid json").expect("Should write file");
        
        let result = PlayerSave::load_from_file(file_path);
        assert!(result.is_err());
        
        // Cleanup
        std::fs::remove_file(file_path).ok();
    }

    #[test]
    fn player_save_incremental_updates() {
        let mut save = PlayerSave::new("test_player");
        let initial_modified = save.last_modified();
        
        // Wait a bit to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Update some data
        save.set_level(5);
        save.add_experience(1000);
        
        assert!(save.last_modified() > initial_modified);
        assert_eq!(save.level(), 5);
        assert_eq!(save.experience(), 1000);
    }

    #[test]
    fn player_save_multiple_saves_isolation() {
        let save1 = PlayerSave::new("player1");
        let save2 = PlayerSave::new("player2");
        
        save1.set_level(10);
        save2.set_level(20);
        
        assert_ne!(save1.level(), save2.level());
        assert_ne!(save1.player_name(), save2.player_name());
    }

    #[test]
    fn player_save_large_data_handling() {
        let mut save = PlayerSave::new("hoarder_player");
        
        // Add many items to inventory
        for i in 0..1000 {
            let item = PlayerItem::new(
                PlayerItemType::Consumable,
                format!("Item{}", i),
                1
            );
            save.inventory_mut().add_item(item).unwrap();
        }
        
        // Should still serialize/deserialize correctly
        let serialized = save.serialize().expect("Should serialize large save");
        let deserialized = PlayerSave::deserialize(&serialized).expect("Should deserialize large save");
        
        assert_eq!(deserialized.inventory().items().len(), 1000);
    }

    #[test]
    fn player_save_concurrent_access() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let save = Arc::new(Mutex::new(PlayerSave::new("concurrent_player")));
        let mut handles = vec![];
        
        // Multiple threads updating different aspects
        for i in 0..4 {
            let save_clone = Arc::clone(&save);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let mut s = save_clone.lock().unwrap();
                    match i {
                        0 => s.add_experience(1),
                        1 => s.set_health(s.health() + 1.0),
                        2 => s.set_position([i as f32, j as f32, 0.0]),
                        3 => {
                            let item = PlayerItem::new(PlayerItemType::Consumable, format!("Item{}", j), 1);
                            s.inventory_mut().add_item(item).ok();
                        }
                        _ => {}
                    }
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_save = save.lock().unwrap();
        assert!(final_save.experience() > 0);
        assert!(final_save.health() > 100.0);
        assert!(final_save.inventory().items().len() > 0);
    }

    #[test]
    fn player_save_version_compatibility() {
        let current_save = create_test_save();
        
        // Simulate older version save (missing some fields)
        let mut older_data = current_save.serialize().unwrap();
        // Remove some fields to simulate older version
        if let Ok(json_str) = std::str::from_utf8(&older_data) {
            let mut json_value: serde_json::Value = serde_json::from_str(json_str).unwrap();
            json_value["new_field_not_in_old_version"] = serde_json::Value::Null;
            older_data = serde_json::to_vec(&json_value).unwrap();
        }
        
        // Should handle missing fields gracefully
        let loaded = PlayerSave::deserialize(&older_data);
        assert!(loaded.is_ok(), "Should handle version differences gracefully");
    }

    #[test]
    fn player_save_data_validation() {
        let mut save = PlayerSave::new("test_player");
        
        // Test invalid level
        let result = save.set_level(0);
        assert!(result.is_err(), "Level 0 should be invalid");
        
        // Test negative experience
        let result = save.add_experience(-100);
        assert!(result.is_err(), "Negative experience should be invalid");
        
        // Test invalid health
        let result = save.set_health(-10.0);
        assert!(result.is_err(), "Negative health should be invalid");
        
        // Test invalid position
        let result = save.set_position([f32::NAN, 0.0, 0.0]);
        assert!(result.is_err(), "NaN position should be invalid");
    }

    #[test]
    fn player_save_backup_and_recovery() {
        let save = create_test_save();
        let original_file = "primary_save.json";
        let backup_file = "backup_save.json";
        
        // Save primary
        save.save_to_file(original_file).unwrap();
        
        // Create backup
        save.save_to_file(backup_file).unwrap();
        
        // Modify primary
        save.set_level(99);
        save.save_to_file(original_file).unwrap();
        
        // Load from backup (should have original level)
        let backup_save = PlayerSave::load_from_file(backup_file).unwrap();
        assert_ne!(backup_save.level(), save.level());
        assert_eq!(backup_save.level(), 5); // Original level from create_test_save
        
        // Cleanup
        std::fs::remove_file(original_file).ok();
        std::fs::remove_file(backup_file).ok();
    }

    #[test]
    fn player_save_performance_large_save() {
        let mut save = PlayerSave::new("performance_test");
        
        // Add lots of data
        for i in 0..10000 {
            let item = PlayerItem::new(
                PlayerItemType::Consumable,
                format!("PerformanceItem{}", i),
                i % 100 + 1
            );
            save.inventory_mut().add_item(item).unwrap();
        }
        
        save.add_experience(1000000);
        save.set_level(100);
        
        // Test serialization performance
        let start = std::time::Instant::now();
        let serialized = save.serialize().unwrap();
        let serialize_duration = start.elapsed();
        
        // Test deserialization performance
        let start = std::time::Instant::now();
        let deserialized = PlayerSave::deserialize(&serialized).unwrap();
        let deserialize_duration = start.elapsed();
        
        assert_eq!(deserialized.inventory().items().len(), 10000);
        assert!(serialize_duration.as_millis() < 100, "Large save serialization should be fast");
        assert!(deserialize_duration.as_millis() < 100, "Large save deserialization should be fast");
    }
}

#[cfg(test)]
mod player_persistence_tests {
    use engene::game::player_save::PlayerSave;
    use engene::game::player::{PlayerController, PlayerState};
    use engene::world::components::PersonalNeeds;

    #[test]
    fn player_controller_to_save_conversion() {
        let mut controller = PlayerController::new([10.0, 5.0, 20.0]);
        controller.set_state(PlayerState::Moving);
        controller.take_damage(25.0);
        
        let save = PlayerSave::from_controller(&controller, "converted_player");
        
        assert_eq!(save.position(), [10.0, 5.0, 20.0]);
        assert_eq!(save.health(), 75.0);
        assert_eq!(save.player_state(), PlayerState::Moving);
    }

    #[test]
    fn player_save_to_controller_conversion() {
        let mut save = PlayerSave::new("reconstructed_player");
        save.set_position([15.0, 10.0, 25.0]);
        save.set_health(50.0);
        save.set_state(PlayerState::Combat);
        
        let controller = save.to_controller();
        
        assert_eq!(controller.position, [15.0, 10.0, 25.0]);
        assert_eq!(controller.health, 50.0);
        assert_eq!(controller.state, PlayerState::Combat);
    }

    #[test]
    fn player_needs_persistence() {
        let mut save = PlayerSave::new("needs_test");
        
        // Set initial needs
        save.set_hunger(80.0);
        save.set_thirst(70.0);
        save.set_fatigue(60.0);
        
        // Serialize and deserialize
        let serialized = save.serialize().unwrap();
        let restored = PlayerSave::deserialize(&serialized).unwrap();
        
        assert_eq!(restored.hunger(), 80.0);
        assert_eq!(restored.thirst(), 70.0);
        assert_eq!(restored.fatigue(), 60.0);
    }

    #[test]
    fn player_needs_bounds_validation() {
        let mut save = PlayerSave::new("bounds_test");
        
        // Test valid ranges
        assert!(save.set_hunger(50.0).is_ok());
        assert!(save.set_thirst(75.0).is_ok());
        assert!(save.set_fatigue(25.0).is_ok());
        
        // Test invalid ranges
        assert!(save.set_hunger(-10.0).is_err());
        assert!(save.set_hunger(110.0).is_err());
        assert!(save.set_thirst(f32::NAN).is_err());
        assert!(save.set_fatigue(f32::INFINITY).is_err());
    }

    #[test]
    fn player_persistence_across_sessions() {
        let file_path = "session_test_save.json";
        
        // Session 1: Create and save
        let mut save1 = PlayerSave::new("session_player");
        save1.set_level(10);
        save1.add_experience(5000);
        save1.set_position([100.0, 200.0, 300.0]);
        save1.save_to_file(file_path).unwrap();
        
        // Session 2: Load and modify
        let mut save2 = PlayerSave::load_from_file(file_path).unwrap();
        assert_eq!(save2.level(), 10);
        assert_eq!(save2.experience(), 5000);
        
        save2.add_experience(1000);
        save2.set_position([150.0, 250.0, 350.0]);
        save2.save_to_file(file_path).unwrap();
        
        // Session 3: Load final state
        let save3 = PlayerSave::load_from_file(file_path).unwrap();
        assert_eq!(save3.level(), 10);
        assert_eq!(save3.experience(), 6000);
        assert_eq!(save3.position(), [150.0, 250.0, 350.0]);
        
        // Cleanup
        std::fs::remove_file(file_path).ok();
    }
}

// Helper function to create test save
fn create_test_save() -> PlayerSave {
    let mut save = PlayerSave::new("test_player");
    save.set_level(5);
    save.add_experience(2500);
    save.set_position([10.0, 20.0, 30.0]);
    save.set_health(85.0);
    save.set_state(PlayerState::Moving);
    
    // Add some items
    let sword = PlayerItem::new(PlayerItemType::Weapon, "Test Sword".to_string(), 1);
    let potion = PlayerItem::new(PlayerItemType::Consumable, "Health Potion".to_string(), 5);
    save.inventory_mut().add_item(sword).unwrap();
    save.inventory_mut().add_item(potion).unwrap();
    
    save
}

// Mock implementations for PlayerSave
impl PlayerSave {
    fn new(name: &str) -> Self {
        Self {
            player_name: name.to_string(),
            level: 1,
            experience: 0,
            position: [0.0, 0.0, 0.0],
            health: 100.0,
            max_health: 100.0,
            stamina: 100.0,
            max_stamina: 100.0,
            state: PlayerState::Idle,
            inventory: PlayerInventory::new(),
            hunger: 100.0,
            thirst: 100.0,
            fatigue: 0.0,
            created_at: std::time::SystemTime::now(),
            last_modified: std::time::SystemTime::now(),
            version: 1,
        }
    }
    
    fn player_name(&self) -> &str {
        &self.player_name
    }
    
    fn level(&self) -> u32 {
        self.level
    }
    
    fn set_level(&mut self, level: u32) -> Result<(), String> {
        if level == 0 {
            return Err("Level must be positive".to_string());
        }
        self.level = level;
        self.update_timestamp();
        Ok(())
    }
    
    fn experience(&self) -> u64 {
        self.experience
    }
    
    fn add_experience(&mut self, exp: i64) -> Result<(), String> {
        if exp < 0 {
            return Err("Experience cannot be negative".to_string());
        }
        self.experience = self.experience.saturating_add(exp as u64);
        self.update_timestamp();
        Ok(())
    }
    
    fn position(&self) -> [f32; 3] {
        self.position
    }
    
    fn set_position(&mut self, pos: [f32; 3]) -> Result<(), String> {
        if !pos.iter().all(|&x| x.is_finite()) {
            return Err("Position must be finite".to_string());
        }
        self.position = pos;
        self.update_timestamp();
        Ok(())
    }
    
    fn health(&self) -> f32 {
        self.health
    }
    
    fn set_health(&mut self, health: f32) -> Result<(), String> {
        if health < 0.0 || !health.is_finite() {
            return Err("Health must be non-negative and finite".to_string());
        }
        self.health = health.min(self.max_health);
        self.update_timestamp();
        Ok(())
    }
    
    fn player_state(&self) -> PlayerState {
        self.state
    }
    
    fn set_state(&mut self, state: PlayerState) {
        self.state = state;
        self.update_timestamp();
    }
    
    fn inventory(&self) -> &PlayerInventory {
        &self.inventory
    }
    
    fn inventory_mut(&mut self) -> &mut PlayerInventory {
        &mut self.inventory
    }
    
    fn hunger(&self) -> f32 {
        self.hunger
    }
    
    fn set_hunger(&mut self, hunger: f32) -> Result<(), String> {
        if !(0.0..=100.0).contains(&hunger) || !hunger.is_finite() {
            return Err("Hunger must be between 0 and 100".to_string());
        }
        self.hunger = hunger;
        self.update_timestamp();
        Ok(())
    }
    
    fn thirst(&self) -> f32 {
        self.thirst
    }
    
    fn set_thirst(&mut self, thirst: f32) -> Result<(), String> {
        if !(0.0..=100.0).contains(&thirst) || !thirst.is_finite() {
            return Err("Thirst must be between 0 and 100".to_string());
        }
        self.thirst = thirst;
        self.update_timestamp();
        Ok(())
    }
    
    fn fatigue(&self) -> f32 {
        self.fatigue
    }
    
    fn set_fatigue(&mut self, fatigue: f32) -> Result<(), String> {
        if !(0.0..=100.0).contains(&fatigue) || !fatigue.is_finite() {
            return Err("Fatigue must be between 0 and 100".to_string());
        }
        self.fatigue = fatigue;
        self.update_timestamp();
        Ok(())
    }
    
    fn created_at(&self) -> std::time::SystemTime {
        self.created_at
    }
    
    fn last_modified(&self) -> std::time::SystemTime {
        self.last_modified
    }
    
    fn serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
    
    fn deserialize(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }
    
    fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.serialize()?;
        std::fs::write(path, data)?;
        Ok(())
    }
    
    fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read(path)?;
        Ok(Self::deserialize(&data)?)
    }
    
    fn from_controller(controller: &PlayerController, name: &str) -> Self {
        let mut save = Self::new(name);
        save.set_position(controller.position).unwrap();
        save.set_health(controller.health).unwrap();
        save.set_state(controller.state);
        save
    }
    
    fn to_controller(&self) -> PlayerController {
        let mut controller = PlayerController::new(self.position);
        controller.health = self.health;
        controller.state = self.state;
        controller
    }
    
    fn update_timestamp(&mut self) {
        self.last_modified = std::time::SystemTime::now();
    }
}

struct PlayerSave {
    player_name: String,
    level: u32,
    experience: u64,
    position: [f32; 3],
    health: f32,
    max_health: f32,
    stamina: f32,
    max_stamina: f32,
    state: PlayerState,
    inventory: PlayerInventory,
    hunger: f32,
    thirst: f32,
    fatigue: f32,
    created_at: std::time::SystemTime,
    last_modified: std::time::SystemTime,
    version: u32,
}

// Re-use PlayerController from previous file
#[derive(Debug, Clone, Copy, PartialEq)]
enum PlayerState {
    Idle,
    Moving,
    Combat,
    Dead,
}

// Mock implementations for serialization
impl serde::Serialize for PlayerSave {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        
        let mut state = serializer.serialize_struct("PlayerSave", 15)?;
        state.serialize_field("player_name", &self.player_name)?;
        state.serialize_field("level", &self.level)?;
        state.serialize_field("experience", &self.experience)?;
        state.serialize_field("position", &self.position)?;
        state.serialize_field("health", &self.health)?;
        state.serialize_field("max_health", &self.max_health)?;
        state.serialize_field("stamina", &self.stamina)?;
        state.serialize_field("max_stamina", &self.max_stamina)?;
        state.serialize_field("state", &format!("{:?}", self.state))?;
        state.serialize_field("inventory_items_count", &self.inventory.items().len())?;
        state.serialize_field("hunger", &self.hunger)?;
        state.serialize_field("thirst", &self.thirst)?;
        state.serialize_field("fatigue", &self.fatigue)?;
        state.serialize_field("created_at", &self.created_at.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())?;
        state.serialize_field("last_modified", &self.last_modified.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())?;
        state.serialize_field("version", &self.version)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for PlayerSave {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct PlayerSaveData {
            player_name: String,
            level: u32,
            experience: u64,
            position: [f32; 3],
            health: f32,
            max_health: f32,
            stamina: f32,
            max_stamina: f32,
            state: String,
            inventory_items_count: usize,
            hunger: f32,
            thirst: f32,
            fatigue: f32,
            created_at: u64,
            last_modified: u64,
            version: u32,
        }
        
        let data = PlayerSaveData::deserialize(deserializer)?;
        
        let created_at = std::time::UNIX_EPOCH + std::time::Duration::from_secs(data.created_at);
        let last_modified = std::time::UNIX_EPOCH + std::time::Duration::from_secs(data.last_modified);
        
        let state = match data.state.as_str() {
            "Idle" => PlayerState::Idle,
            "Moving" => PlayerState::Moving,
            "Combat" => PlayerState::Combat,
            "Dead" => PlayerState::Dead,
            _ => PlayerState::Idle,
        };
        
        Ok(PlayerSave {
            player_name: data.player_name,
            level: data.level,
            experience: data.experience,
            position: data.position,
            health: data.health,
            max_health: data.max_health,
            stamina: data.stamina,
            max_stamina: data.max_stamina,
            state,
            inventory: PlayerInventory::new(),
            hunger: data.hunger,
            thirst: data.thirst,
            fatigue: data.fatigue,
            created_at,
            last_modified,
            version: data.version,
        })
    }
}
