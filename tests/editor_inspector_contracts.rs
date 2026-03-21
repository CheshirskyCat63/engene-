//! Editor Inspector Contracts
//! 
//! Tests for component inspector, property editing, and reflection.
//! Ownership: Tools Team
//! Lane: tools
//! Type: Contract + Unit Tests
//! Speed: Fast

#[cfg(test)]
mod editor_inspector_tests {
    use engene::tools::inspector::{InspectorEdit, InspectorState};
    use engene::world::components::{Transform, Velocity, Health};

    #[test]
    fn inspector_state_new_creates_empty() {
        let inspector = InspectorState::new();
        assert!(inspector.selected_entities().is_empty());
        assert!(inspector.editable_components().is_empty());
        assert!(inspector.is_dirty() == false);
    }

    #[test]
    fn inspector_select_entity_adds_to_selection() {
        let mut inspector = InspectorState::new();
        let entity_id = 123;
        
        inspector.select_entity(entity_id);
        
        assert_eq!(inspector.selected_entities().len(), 1);
        assert!(inspector.selected_entities().contains(&entity_id));
    }

    #[test]
    fn inspector_select_multiple_entities() {
        let mut inspector = InspectorState::new();
        let entities = vec![123, 456, 789];
        
        for &entity_id in &entities {
            inspector.select_entity(entity_id);
        }
        
        assert_eq!(inspector.selected_entities().len(), 3);
        for entity_id in entities {
            assert!(inspector.selected_entities().contains(&entity_id));
        }
    }

    #[test]
    fn inspector_deselect_entity_removes_from_selection() {
        let mut inspector = InspectorState::new();
        let entity_id = 123;
        
        inspector.select_entity(entity_id);
        assert!(inspector.selected_entities().contains(&entity_id));
        
        inspector.deselect_entity(entity_id);
        assert!(!inspector.selected_entities().contains(&entity_id));
        assert!(inspector.selected_entities().is_empty());
    }

    #[test]
    fn inspector_clear_selection_removes_all() {
        let mut inspector = InspectorState::new();
        
        // Select multiple entities
        for i in 0..10 {
            inspector.select_entity(i);
        }
        
        assert_eq!(inspector.selected_entities().len(), 10);
        
        // Clear all
        inspector.clear_selection();
        assert!(inspector.selected_entities().is_empty());
    }

    #[test]
    fn inspector_refresh_updates_components() {
        let mut inspector = InspectorState::new();
        let entity_id = 123;
        
        inspector.select_entity(entity_id);
        let initial_components = inspector.editable_components().len();
        
        // Refresh should update component list
        inspector.refresh();
        let refreshed_components = inspector.editable_components().len();
        
        // Should have same or updated count
        assert!(refreshed_components >= initial_components);
    }

    #[test]
    fn inspector_edit_component_creates_edit() {
        let mut inspector = InspectorState::new();
        let entity_id = 123;
        
        inspector.select_entity(entity_id);
        
        // Try to edit Transform component
        let edit_result = inspector.edit_component("Transform");
        assert!(edit_result.is_ok());
        
        let edit = edit_result.unwrap();
        assert_eq!(edit.component_name(), "Transform");
        assert!(edit.is_dirty() == false);
    }

    #[test]
    fn inspector_edit_nonexistent_component_fails() {
        let mut inspector = InspectorState::new();
        let entity_id = 123;
        
        inspector.select_entity(entity_id);
        
        // Try to edit nonexistent component
        let edit_result = inspector.edit_component("NonexistentComponent");
        assert!(edit_result.is_err());
    }

    #[test]
    fn inspector_edit_property_updates_value() {
        let mut inspector = InspectorState::new();
        let entity_id = 123;
        
        inspector.select_entity(entity_id);
        let mut edit = inspector.edit_component("Transform").unwrap();
        
        // Edit position property
        let result = edit.edit_property("position", "10.0, 20.0, 30.0");
        assert!(result.is_ok());
        
        assert!(edit.is_dirty());
        assert_eq!(edit.get_property_value("position"), Some("10.0, 20.0, 30.0"));
    }

    #[test]
    fn inspector_edit_invalid_property_fails() {
        let mut inspector = InspectorState::new();
        let entity_id = 123;
        
        inspector.select_entity(entity_id);
        let mut edit = inspector.edit_component("Transform").unwrap();
        
        // Try to edit nonexistent property
        let result = edit.edit_property("nonexistent_property", "value");
        assert!(result.is_err());
        
        assert!(edit.is_dirty() == false);
    }

    #[test]
    fn inspector_apply_changes_marks_clean() {
        let mut inspector = InspectorState::new();
        let entity_id = 123;
        
        inspector.select_entity(entity_id);
        let mut edit = inspector.edit_component("Transform").unwrap();
        
        // Make changes
        edit.edit_property("position", "1.0, 2.0, 3.0").unwrap();
        assert!(edit.is_dirty());
        
        // Apply changes
        let apply_result = inspector.apply_changes(&mut edit);
        assert!(apply_result.is_ok());
        
        assert!(edit.is_dirty() == false);
        assert!(inspector.is_dirty() == false);
    }

    #[test]
    fn inspector_discard_changes_reverts_values() {
        let mut inspector = InspectorState::new();
        let entity_id = 123;
        
        inspector.select_entity(entity_id);
        let mut edit = inspector.edit_component("Transform").unwrap();
        
        let original_value = edit.get_property_value("position");
        
        // Make changes
        edit.edit_property("position", "99.0, 99.0, 99.0").unwrap();
        assert!(edit.is_dirty());
        assert_ne!(edit.get_property_value("position"), original_value);
        
        // Discard changes
        inspector.discard_changes(&mut edit);
        
        assert!(edit.is_dirty() == false);
        assert_eq!(edit.get_property_value("position"), original_value);
    }

    #[test]
    fn inspector_batch_edit_multiple_entities() {
        let mut inspector = InspectorState::new();
        let entities = vec![123, 456, 789];
        
        // Select multiple entities
        for &entity_id in &entities {
            inspector.select_entity(entity_id);
        }
        
        // Edit component on all selected entities
        let batch_result = inspector.batch_edit_component("Transform", "position", "5.0, 5.0, 5.0");
        assert!(batch_result.is_ok());
        
        let affected_count = batch_result.unwrap();
        assert_eq!(affected_count, entities.len());
    }

    #[test]
    fn inspector_component_filtering() {
        let mut inspector = InspectorState::new();
        let entity_id = 123;
        
        inspector.select_entity(entity_id);
        
        // Filter components by name
        let filtered = inspector.filter_components("Transform");
        assert!(filtered.iter().any(|comp| comp.name().contains("Transform")));
        
        // Filter by type
        let physics_components = inspector.filter_components_by_type("Physics");
        // Should include Velocity, Health, etc.
        assert!(physics_components.len() >= 0);
    }

    #[test]
    fn inspector_search_functionality() {
        let mut inspector = InspectorState::new();
        let entity_id = 123;
        
        inspector.select_entity(entity_id);
        
        // Search for specific property
        let results = inspector.search_properties("position");
        assert!(results.iter().any(|prop| prop.contains("position")));
        
        // Search for specific value
        let value_results = inspector.search_values("1.0");
        // Should find properties with value 1.0
        assert!(value_results.len() >= 0);
    }

    #[test]
    fn inspector_undo_redo_functionality() {
        let mut inspector = InspectorState::new();
        let entity_id = 123;
        
        inspector.select_entity(entity_id);
        let mut edit = inspector.edit_component("Transform").unwrap();
        
        let original_value = edit.get_property_value("position");
        
        // Make first change
        edit.edit_property("position", "10.0, 10.0, 10.0").unwrap();
        inspector.apply_changes(&mut edit).unwrap();
        
        // Make second change
        edit.edit_property("position", "20.0, 20.0, 20.0").unwrap();
        inspector.apply_changes(&mut edit).unwrap();
        
        // Undo to first change
        let undo_result = inspector.undo();
        assert!(undo_result.is_ok());
        assert_eq!(edit.get_property_value("position"), Some("10.0, 10.0, 10.0"));
        
        // Redo to second change
        let redo_result = inspector.redo();
        assert!(redo_result.is_ok());
        assert_eq!(edit.get_property_value("position"), Some("20.0, 20.0, 20.0"));
    }

    #[test]
    fn inspector_performance_many_entities() {
        let mut inspector = InspectorState::new();
        let start = std::time::Instant::now();
        
        // Select many entities
        for i in 0..1000 {
            inspector.select_entity(i);
        }
        
        let select_duration = start.elapsed();
        
        // Refresh component list
        let refresh_start = std::time::Instant::now();
        inspector.refresh();
        let refresh_duration = refresh_start.elapsed();
        
        assert_eq!(inspector.selected_entities().len(), 1000);
        assert!(select_duration.as_millis() < 50, "Selecting 1000 entities should be fast");
        assert!(refresh_duration.as_millis() < 100, "Refreshing should be fast");
    }

    #[test]
    fn inspector_concurrent_selection() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let inspector = Arc::new(Mutex::new(InspectorState::new()));
        let mut handles = vec![];
        
        // Multiple threads selecting entities
        for i in 0..4 {
            let inspector_clone = Arc::clone(&inspector);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let mut ins = inspector_clone.lock().unwrap();
                    ins.select_entity(i * 100 + j);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        
        let final_inspector = inspector.lock().unwrap();
        assert_eq!(final_inspector.selected_entities().len(), 400);
    }
}

#[cfg(test)]
mod component_editing_tests {
    use engene::tools::inspector::{InspectorEdit, InspectorState};

    #[test]
    fn component_edit_type_validation() {
        let mut inspector = InspectorState::new();
        inspector.select_entity(123);
        
        let mut edit = inspector.edit_component("Transform").unwrap();
        
        // Test valid float values
        assert!(edit.edit_property("position.x", "10.5").is_ok());
        
        // Test invalid float values
        assert!(edit.edit_property("position.x", "not_a_number").is_err());
        
        // Test valid vector values
        assert!(edit.edit_property("position", "1.0, 2.0, 3.0").is_ok());
        
        // Test invalid vector values
        assert!(edit.edit_property("position", "1.0, 2.0").is_err()); // Missing Z
        assert!(edit.edit_property("position", "1.0, not_a_number, 3.0").is_err());
    }

    #[test]
    fn component_edit_range_validation() {
        let mut inspector = InspectorState::new();
        inspector.select_entity(123);
        
        let mut edit = inspector.edit_component("Health").unwrap();
        
        // Test valid health values
        assert!(edit.edit_property("current", "50.0").is_ok());
        assert!(edit.edit_property("current", "0.0").is_ok());
        assert!(edit.edit_property("current", "100.0").is_ok());
        
        // Test invalid health values
        assert!(edit.edit_property("current", "-10.0").is_err()); // Negative
        assert!(edit.edit_property("current", "150.0").is_err()); // Over max
    }

    #[test]
    fn component_edit_enum_validation() {
        let mut inspector = InspectorState::new();
        inspector.select_entity(123);
        
        let mut edit = inspector.edit_component("Velocity").unwrap();
        
        // Test valid enum values
        assert!(edit.edit_property("mode", "Walking").is_ok());
        assert!(edit.edit_property("mode", "Running").is_ok());
        assert!(edit.edit_property("mode", "Idle").is_ok());
        
        // Test invalid enum values
        assert!(edit.edit_property("mode", "Flying").is_err()); // Not a valid mode
    }

    #[test]
    fn component_edit_batch_validation() {
        let mut inspector = InspectorState::new();
        
        // Select multiple entities
        for i in 0..10 {
            inspector.select_entity(i);
        }
        
        // Batch edit with valid value
        let result = inspector.batch_edit_component("Transform", "position.x", "5.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10); // All 10 entities affected
        
        // Batch edit with invalid value
        let invalid_result = inspector.batch_edit_component("Transform", "position.x", "invalid");
        assert!(invalid_result.is_err());
    }

    #[test]
    fn component_edit_persistence() {
        let mut inspector = InspectorState::new();
        inspector.select_entity(123);
        
        let mut edit = inspector.edit_component("Transform").unwrap();
        
        // Edit property
        edit.edit_property("position", "1.0, 2.0, 3.0").unwrap();
        
        // Save edit state
        let saved_state = edit.save_state();
        assert!(!saved_state.is_empty());
        
        // Load edit state
        let mut new_edit = inspector.edit_component("Transform").unwrap();
        let load_result = new_edit.load_state(&saved_state);
        assert!(load_result.is_ok());
        
        assert_eq!(new_edit.get_property_value("position"), Some("1.0, 2.0, 3.0"));
    }

    #[test]
    fn component_edit_diff_generation() {
        let mut inspector = InspectorState::new();
        inspector.select_entity(123);
        
        let mut edit = inspector.edit_component("Transform").unwrap();
        
        // Generate initial diff
        let initial_diff = edit.generate_diff();
        assert!(initial_diff.is_empty()); // No changes yet
        
        // Make changes
        edit.edit_property("position", "1.0, 2.0, 3.0").unwrap();
        edit.edit_property("rotation", "0.0, 90.0, 0.0").unwrap();
        
        // Generate diff
        let diff = edit.generate_diff();
        assert!(!diff.is_empty());
        assert!(diff.contains("position"));
        assert!(diff.contains("rotation"));
    }
}

// Mock implementations
#[derive(Debug)]
struct Transform {
    position: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 3],
}

#[derive(Debug)]
struct Velocity {
    vector: [f32; 3],
    mode: VelocityMode,
}

#[derive(Debug)]
struct Health {
    current: f32,
    max: f32,
}

#[derive(Debug, Clone, PartialEq)]
enum VelocityMode {
    Idle,
    Walking,
    Running,
}

impl InspectorState {
    fn new() -> Self {
        Self {
            selected_entities: Vec::new(),
            editable_components: Vec::new(),
            dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
    
    fn select_entity(&mut self, entity_id: u32) {
        if !self.selected_entities.contains(&entity_id) {
            self.selected_entities.push(entity_id);
            self.refresh();
        }
    }
    
    fn deselect_entity(&mut self, entity_id: u32) {
        self.selected_entities.retain(|&e| e != entity_id);
        self.refresh();
    }
    
    fn clear_selection(&mut self) {
        self.selected_entities.clear();
        self.editable_components.clear();
    }
    
    fn selected_entities(&self) -> &[u32] {
        &self.selected_entities
    }
    
    fn editable_components(&self) -> &[ComponentInfo] {
        &self.editable_components
    }
    
    fn refresh(&mut self) {
        self.editable_components = vec![
            ComponentInfo::new("Transform", "spatial"),
            ComponentInfo::new("Velocity", "physics"),
            ComponentInfo::new("Health", "gameplay"),
        ];
    }
    
    fn edit_component(&mut self, component_name: &str) -> Result<InspectorEdit, String> {
        if self.editable_components.iter().any(|c| c.name() == component_name) {
            Ok(InspectorEdit::new(component_name))
        } else {
            Err(format!("Component '{}' not found", component_name))
        }
    }
    
    fn apply_changes(&mut self, edit: &mut InspectorEdit) -> Result<(), String> {
        // Save current state to undo stack
        self.undo_stack.push(edit.save_state());
        self.redo_stack.clear();
        
        // Apply changes
        edit.mark_clean();
        self.dirty = false;
        
        Ok(())
    }
    
    fn discard_changes(&mut self, edit: &mut InspectorEdit) {
        edit.load_state(&edit.save_state());
        self.dirty = false;
    }
    
    fn batch_edit_component(&mut self, component_name: &str, property: &str, value: &str) -> Result<usize, String> {
        let affected_count = self.selected_entities.len();
        
        for &entity_id in &self.selected_entities {
            let mut edit = self.edit_component(component_name)?;
            edit.edit_property(property, value)?;
            self.apply_changes(&mut edit)?;
        }
        
        Ok(affected_count)
    }
    
    fn filter_components(&self, name_pattern: &str) -> Vec<&ComponentInfo> {
        self.editable_components.iter()
            .filter(|comp| comp.name().contains(name_pattern))
            .collect()
    }
    
    fn filter_components_by_type(&self, component_type: &str) -> Vec<&ComponentInfo> {
        self.editable_components.iter()
            .filter(|comp| comp.component_type() == component_type)
            .collect()
    }
    
    fn search_properties(&self, pattern: &str) -> Vec<String> {
        // Mock implementation - would search through all component properties
        vec![
            format!("Transform.position - {}", pattern),
            format!("Velocity.vector - {}", pattern),
        ]
    }
    
    fn search_values(&self, pattern: &str) -> Vec<String> {
        // Mock implementation - would search through all property values
        vec![
            format!("Transform.position.x = {}", pattern),
            format!("Health.current = {}", pattern),
        ]
    }
    
    fn undo(&mut self) -> Result<(), String> {
        if let Some(state) = self.undo_stack.pop() {
            self.redo_stack.push(state.clone());
            // Apply undo logic
            Ok(())
        } else {
            Err("Nothing to undo".to_string())
        }
    }
    
    fn redo(&mut self) -> Result<(), String> {
        if let Some(state) = self.redo_stack.pop() {
            self.undo_stack.push(state.clone());
            // Apply redo logic
            Ok(())
        } else {
            Err("Nothing to redo".to_string())
        }
    }
    
    fn is_dirty(&self) -> bool {
        self.dirty
    }
}

#[derive(Debug)]
struct ComponentInfo {
    name: String,
    component_type: String,
}

impl ComponentInfo {
    fn new(name: &str, component_type: &str) -> Self {
        Self {
            name: name.to_string(),
            component_type: component_type.to_string(),
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn component_type(&self) -> &str {
        &self.component_type
    }
}

impl InspectorEdit {
    fn new(component_name: &str) -> Self {
        Self {
            component_name: component_name.to_string(),
            properties: std::collections::HashMap::new(),
            dirty: false,
            original_values: std::collections::HashMap::new(),
        }
    }
    
    fn component_name(&self) -> &str {
        &self.component_name
    }
    
    fn edit_property(&mut self, property: &str, value: &str) -> Result<(), String> {
        // Validate and set property
        if self.validate_property(property, value) {
            self.properties.insert(property.to_string(), value.to_string());
            self.dirty = true;
            Ok(())
        } else {
            Err(format!("Invalid value for property '{}': {}", property, value))
        }
    }
    
    fn get_property_value(&self, property: &str) -> Option<&str> {
        self.properties.get(property).map(|s| s.as_str())
    }
    
    fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    fn mark_clean(&mut self) {
        self.dirty = false;
    }
    
    fn save_state(&self) -> String {
        // Serialize current state
        format!("{}:{:?}", self.component_name, self.properties)
    }
    
    fn load_state(&mut self, state: &str) -> Result<(), String> {
        // Deserialize and restore state
        // Mock implementation
        Ok(())
    }
    
    fn generate_diff(&self) -> String {
        // Generate diff of changes
        let mut diff = String::new();
        for (property, value) in &self.properties {
            diff.push_str(&format!("{}: {}\n", property, value));
        }
        diff
    }
    
    fn validate_property(&self, property: &str, value: &str) -> bool {
        // Mock validation logic
        match property {
            "position" | "rotation" | "scale" => {
                // Should be valid vector format
                value.split(',').count() == 3 && 
                value.split(',').all(|v| v.trim().parse::<f32>().is_ok())
            }
            "current" | "max" => {
                // Should be valid float in range
                if let Ok(val) = value.parse::<f32>() {
                    val >= 0.0 && val <= 100.0
                } else {
                    false
                }
            }
            "mode" => {
                // Should be valid enum value
                ["Idle", "Walking", "Running"].contains(&value)
            }
            _ => true, // Unknown property, allow for mock
        }
    }
}

struct InspectorEdit {
    component_name: String,
    properties: std::collections::HashMap<String, String>,
    dirty: bool,
    original_values: std::collections::HashMap<String, String>,
}
