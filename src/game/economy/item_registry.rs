use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemCategory {
    Weapon,
    Armor,
    Medkit,
    Food,
    Ammo,
    Artifact,
    Quest,
    Junk,
    Tool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Unique,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemTemplate {
    pub id: String,
    pub name: String,
    pub category: ItemCategory,
    pub rarity: ItemRarity,
    pub base_value: f32,
    pub weight: f32,
    pub max_stack: u32,
    pub max_durability: f32,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemInstance {
    pub template_id: String,
    pub stack_count: u32,
    pub durability: f32,
    pub modifications: Vec<String>,
}

pub struct ItemRegistry {
    templates: std::collections::HashMap<String, ItemTemplate>,
}

impl ItemRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            templates: std::collections::HashMap::new(),
        };
        reg.register_defaults();
        reg
    }

    fn register_defaults(&mut self) {
        let defaults = vec![
            ItemTemplate {
                id: "medkit".into(),
                name: "Medkit".into(),
                category: ItemCategory::Medkit,
                rarity: ItemRarity::Common,
                base_value: 50.0,
                weight: 0.3,
                max_stack: 5,
                max_durability: 1.0,
                description: "Basic medical supplies".into(),
            },
            ItemTemplate {
                id: "army_medkit".into(),
                name: "Army Medkit".into(),
                category: ItemCategory::Medkit,
                rarity: ItemRarity::Uncommon,
                base_value: 150.0,
                weight: 0.5,
                max_stack: 3,
                max_durability: 1.0,
                description: "Military-grade medical kit".into(),
            },
            ItemTemplate {
                id: "bread".into(),
                name: "Bread".into(),
                category: ItemCategory::Food,
                rarity: ItemRarity::Common,
                base_value: 15.0,
                weight: 0.2,
                max_stack: 10,
                max_durability: 1.0,
                description: "Simple bread".into(),
            },
            ItemTemplate {
                id: "canned_food".into(),
                name: "Canned Food".into(),
                category: ItemCategory::Food,
                rarity: ItemRarity::Common,
                base_value: 25.0,
                weight: 0.4,
                max_stack: 5,
                max_durability: 1.0,
                description: "Preserved food".into(),
            },
            ItemTemplate {
                id: "vodka".into(),
                name: "Vodka".into(),
                category: ItemCategory::Food,
                rarity: ItemRarity::Common,
                base_value: 30.0,
                weight: 0.5,
                max_stack: 3,
                max_durability: 1.0,
                description: "Reduces radiation, impairs aim".into(),
            },
            ItemTemplate {
                id: "pistol_ammo".into(),
                name: "9mm Ammo".into(),
                category: ItemCategory::Ammo,
                rarity: ItemRarity::Common,
                base_value: 5.0,
                weight: 0.01,
                max_stack: 60,
                max_durability: 1.0,
                description: "Standard pistol rounds".into(),
            },
            ItemTemplate {
                id: "rifle_ammo".into(),
                name: "5.56mm Ammo".into(),
                category: ItemCategory::Ammo,
                rarity: ItemRarity::Common,
                base_value: 8.0,
                weight: 0.02,
                max_stack: 60,
                max_durability: 1.0,
                description: "Standard rifle rounds".into(),
            },
            ItemTemplate {
                id: "artifact_moonlight".into(),
                name: "Moonlight".into(),
                category: ItemCategory::Artifact,
                rarity: ItemRarity::Rare,
                base_value: 500.0,
                weight: 0.5,
                max_stack: 1,
                max_durability: 1.0,
                description: "Glowing artifact, restores health".into(),
            },
            ItemTemplate {
                id: "artifact_flame".into(),
                name: "Flame".into(),
                category: ItemCategory::Artifact,
                rarity: ItemRarity::Rare,
                base_value: 800.0,
                weight: 0.3,
                max_stack: 1,
                max_durability: 1.0,
                description: "Hot artifact, increases stamina".into(),
            },
            ItemTemplate {
                id: "bolts".into(),
                name: "Bolts".into(),
                category: ItemCategory::Tool,
                rarity: ItemRarity::Common,
                base_value: 1.0,
                weight: 0.05,
                max_stack: 20,
                max_durability: 1.0,
                description: "Used to detect anomalies".into(),
            },
            ItemTemplate {
                id: "repair_kit".into(),
                name: "Repair Kit".into(),
                category: ItemCategory::Tool,
                rarity: ItemRarity::Uncommon,
                base_value: 100.0,
                weight: 1.0,
                max_stack: 1,
                max_durability: 3.0,
                description: "Repairs weapons and armor".into(),
            },
        ];
        for t in defaults {
            self.templates.insert(t.id.clone(), t);
        }
    }

    pub fn register(&mut self, template: ItemTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    pub fn get(&self, id: &str) -> Option<&ItemTemplate> {
        self.templates.get(id)
    }

    pub fn all(&self) -> impl Iterator<Item = &ItemTemplate> {
        self.templates.values()
    }

    pub fn by_category(&self, cat: ItemCategory) -> Vec<&ItemTemplate> {
        self.templates
            .values()
            .filter(|t| t.category == cat)
            .collect()
    }

    pub fn create_instance(&self, id: &str, count: u32) -> Option<ItemInstance> {
        self.get(id).map(|t| ItemInstance {
            template_id: t.id.clone(),
            stack_count: count.min(t.max_stack),
            durability: t.max_durability,
            modifications: Vec::new(),
        })
    }
}

impl Default for ItemRegistry {
    fn default() -> Self {
        Self::new()
    }
}
