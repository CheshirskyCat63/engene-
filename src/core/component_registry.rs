//! Phase 7: Component registry metadata for simulation, replay, and tooling.

use std::any::TypeId;

use engine_core::ai_emotions::Emotions;
use engine_core::ai_memory::Memory;
use engine_core::ai_plan::Plan;
use engine_world::components::*;
use engine_world::extension_components::{Attributes, Blackboard, EntityTags, StatusEffects};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataTemperature {
    Hot,
    Cold,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateFrequency {
    PerTick,
    PerLevel,
    OnEvent,
    EditorOnly,
    SerializationOnly,
}

#[derive(Clone, Debug)]
pub struct ComponentMeta {
    pub type_id: TypeId,
    pub name: &'static str,
    pub temperature: DataTemperature,
    pub serializable: bool,
    pub authoritative: bool,
    pub prefab_allowed: bool,
    pub replay_relevant: bool,
    pub inspectable: bool,
    pub domain_tags: Vec<&'static str>,
    pub update_frequency: UpdateFrequency,
}

pub struct ComponentRegistry {
    entries: Vec<ComponentMeta>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn register<T: 'static>(
        &mut self,
        name: &'static str,
        temperature: DataTemperature,
        serializable: bool,
        authoritative: bool,
        prefab_allowed: bool,
        replay_relevant: bool,
        inspectable: bool,
        domain_tags: Vec<&'static str>,
        update_frequency: UpdateFrequency,
    ) {
        self.entries.push(ComponentMeta {
            type_id: TypeId::of::<T>(),
            name,
            temperature,
            serializable,
            authoritative,
            prefab_allowed,
            replay_relevant,
            inspectable,
            domain_tags,
            update_frequency,
        });
    }

    pub fn get_by_name(&self, name: &str) -> Option<&ComponentMeta> {
        self.entries.iter().find(|e| e.name == name)
    }

    pub fn hot_components(&self) -> impl Iterator<Item = &ComponentMeta> {
        self.entries
            .iter()
            .filter(|e| matches!(e.temperature, DataTemperature::Hot))
    }

    pub fn cold_components(&self) -> impl Iterator<Item = &ComponentMeta> {
        self.entries
            .iter()
            .filter(|e| matches!(e.temperature, DataTemperature::Cold))
    }

    pub fn replay_relevant(&self) -> impl Iterator<Item = &ComponentMeta> {
        self.entries.iter().filter(|e| e.replay_relevant)
    }

    pub fn default_registry() -> Self {
        let mut reg = Self::new();

        reg.register::<Transform>(
            "Transform",
            DataTemperature::Hot,
            true,
            true,
            false,
            true,
            true,
            vec!["physics", "core"],
            UpdateFrequency::PerTick,
        );
        reg.register::<EntityKind>(
            "EntityKind",
            DataTemperature::Hot,
            true,
            true,
            false,
            true,
            true,
            vec!["core"],
            UpdateFrequency::OnEvent,
        );
        reg.register::<SimLevel>(
            "SimLevel",
            DataTemperature::Hot,
            true,
            true,
            false,
            false,
            true,
            vec!["simulation"],
            UpdateFrequency::PerTick,
        );
        reg.register::<AiState>(
            "AiState",
            DataTemperature::Hot,
            true,
            true,
            false,
            true,
            true,
            vec!["ai"],
            UpdateFrequency::PerTick,
        );
        reg.register::<PersonalNeeds>(
            "PersonalNeeds",
            DataTemperature::Hot,
            true,
            true,
            false,
            false,
            true,
            vec!["ai"],
            UpdateFrequency::PerTick,
        );
        reg.register::<Plan>(
            "Plan",
            DataTemperature::Hot,
            true,
            true,
            false,
            false,
            true,
            vec!["ai"],
            UpdateFrequency::PerTick,
        );
        reg.register::<Memory>(
            "Memory",
            DataTemperature::Cold,
            true,
            true,
            false,
            true,
            true,
            vec!["ai"],
            UpdateFrequency::OnEvent,
        );
        reg.register::<Emotions>(
            "Emotions",
            DataTemperature::Hot,
            true,
            true,
            false,
            false,
            true,
            vec!["ai"],
            UpdateFrequency::PerTick,
        );
        reg.register::<Inventory>(
            "Inventory",
            DataTemperature::Cold,
            true,
            true,
            false,
            false,
            true,
            vec!["economy"],
            UpdateFrequency::OnEvent,
        );
        reg.register::<NpcEconomy>(
            "NpcEconomy",
            DataTemperature::Cold,
            true,
            true,
            false,
            false,
            true,
            vec!["economy"],
            UpdateFrequency::OnEvent,
        );
        reg.register::<NpcTraits>(
            "NpcTraits",
            DataTemperature::Cold,
            true,
            true,
            true,
            false,
            true,
            vec!["ai"],
            UpdateFrequency::SerializationOnly,
        );
        reg.register::<MonsterTraits>(
            "MonsterTraits",
            DataTemperature::Cold,
            true,
            true,
            true,
            false,
            true,
            vec!["ai"],
            UpdateFrequency::SerializationOnly,
        );
        reg.register::<LifeInfo>(
            "LifeInfo",
            DataTemperature::Hot,
            true,
            true,
            false,
            true,
            true,
            vec!["core"],
            UpdateFrequency::PerTick,
        );
        reg.register::<Flammable>(
            "Flammable",
            DataTemperature::Cold,
            true,
            true,
            false,
            false,
            true,
            vec!["physics"],
            UpdateFrequency::OnEvent,
        );
        reg.register::<EntityTags>(
            "EntityTags",
            DataTemperature::Cold,
            true,
            false,
            false,
            false,
            true,
            vec!["tools"],
            UpdateFrequency::EditorOnly,
        );
        reg.register::<Attributes>(
            "Attributes",
            DataTemperature::Cold,
            true,
            true,
            false,
            false,
            true,
            vec!["gameplay"],
            UpdateFrequency::OnEvent,
        );
        reg.register::<StatusEffects>(
            "StatusEffects",
            DataTemperature::Hot,
            true,
            true,
            false,
            true,
            true,
            vec!["gameplay"],
            UpdateFrequency::PerTick,
        );
        reg.register::<Blackboard>(
            "Blackboard",
            DataTemperature::Cold,
            true,
            false,
            false,
            false,
            true,
            vec!["ai"],
            UpdateFrequency::OnEvent,
        );
        reg.register::<ClothComponent>(
            "ClothComponent",
            DataTemperature::Cold,
            true,
            false,
            false,
            false,
            true,
            vec!["physics"],
            UpdateFrequency::OnEvent,
        );
        reg.register::<SocialNeeds>(
            "SocialNeeds",
            DataTemperature::Hot,
            true,
            true,
            false,
            false,
            true,
            vec!["ai"],
            UpdateFrequency::PerTick,
        );
        reg.register::<EcosystemNeeds>(
            "EcosystemNeeds",
            DataTemperature::Hot,
            true,
            true,
            false,
            false,
            true,
            vec!["ecosystem"],
            UpdateFrequency::PerTick,
        );

        reg
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::default_registry()
    }
}
