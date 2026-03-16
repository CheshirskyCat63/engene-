use std::collections::HashMap;
use std::path::Path;
use super::prefab::PrefabDescriptor;

pub struct PrefabRegistry {
    prefabs: HashMap<String, PrefabDescriptor>,
}

impl PrefabRegistry {
    pub fn new() -> Self {
        Self { prefabs: HashMap::new() }
    }

    pub fn register(&mut self, prefab: PrefabDescriptor) {
        self.prefabs.insert(prefab.name.clone(), prefab);
    }

    pub fn get(&self, name: &str) -> Option<&PrefabDescriptor> {
        self.prefabs.get(name)
    }

    pub fn load_from_dir(&mut self, dir: &Path) -> usize {
        let mut count = 0;
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "ron") {
                    if let Ok(contents) = std::fs::read_to_string(&path) {
                        match ron::from_str::<PrefabDescriptor>(&contents) {
                            Ok(prefab) => {
                                self.register(prefab);
                                count += 1;
                            }
                            Err(e) => {
                                println!("[prefab] failed to load {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }
        count
    }

    pub fn resolve_inheritance(&self, name: &str) -> Option<PrefabDescriptor> {
        let prefab = self.prefabs.get(name)?;
        if let Some(ref base_name) = prefab.base {
            let base = self.resolve_inheritance(base_name)?;
            Some(merge_prefabs(&base, prefab))
        } else {
            Some(prefab.clone())
        }
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.prefabs.keys().map(|s| s.as_str())
    }

    pub fn count(&self) -> usize {
        self.prefabs.len()
    }
}

fn merge_prefabs(base: &PrefabDescriptor, override_: &PrefabDescriptor) -> PrefabDescriptor {
    PrefabDescriptor {
        name: override_.name.clone(),
        base: None,
        spec_variant: override_.spec_variant.clone().or(base.spec_variant.clone()),
        root: override_.root.clone(),
        tags: {
            let mut tags = base.tags.clone();
            for tag in &override_.tags {
                if !tags.contains(tag) {
                    tags.push(tag.clone());
                }
            }
            tags
        },
    }
}

impl Default for PrefabRegistry {
    fn default() -> Self { Self::new() }
}
