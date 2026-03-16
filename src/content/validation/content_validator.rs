use crate::content::prefabs::prefab_registry::PrefabRegistry;

#[derive(Debug)]
pub struct ContentValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ContentValidationResult {
    pub fn is_valid(&self) -> bool { self.errors.is_empty() }
}

pub fn validate_prefabs(registry: &PrefabRegistry) -> ContentValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for name in registry.names() {
        let prefab = registry.get(name).unwrap();

        if prefab.root.components.is_empty() {
            warnings.push(format!("Prefab '{}' has no components on root entity", name));
        }

        if let Some(ref base) = prefab.base {
            if registry.get(base).is_none() {
                errors.push(format!("Prefab '{}' inherits from '{}' which doesn't exist", name, base));
            }
        }

        let entity_count = prefab.entity_count();
        if entity_count > 100 {
            warnings.push(format!("Prefab '{}' has {} entities (consider splitting)", name, entity_count));
        }
    }

    ContentValidationResult { errors, warnings }
}
