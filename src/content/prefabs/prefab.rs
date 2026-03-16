use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PrefabValue {
    Float(f32),
    Int(i64),
    Bool(bool),
    String(String),
    Vec3([f32; 3]),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentData {
    pub component_type: String,
    pub fields: HashMap<String, PrefabValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrefabEntity {
    pub name: Option<String>,
    pub components: Vec<ComponentData>,
    pub children: Vec<PrefabEntity>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrefabDescriptor {
    pub name: String,
    pub base: Option<String>,
    pub spec_variant: Option<SpecVariant>,
    pub root: PrefabEntity,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SpecVariant {
    Full,
    Low,
}

impl PrefabDescriptor {
    pub fn entity_count(&self) -> usize {
        count_entities(&self.root)
    }
}

fn count_entities(entity: &PrefabEntity) -> usize {
    1 + entity.children.iter().map(|c| count_entities(c)).sum::<usize>()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrefabOverrides {
    pub overrides: HashMap<String, PrefabValue>,
}
