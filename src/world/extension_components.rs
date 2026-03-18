use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type AttrId = u16;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EntityTags {
    pub bits: u64,
    pub string_tags: Vec<String>,
}

impl EntityTags {
    pub fn has_bit(&self, bit: u32) -> bool {
        self.bits & (1u64 << bit) != 0
    }

    pub fn set_bit(&mut self, bit: u32) {
        self.bits |= 1u64 << bit;
    }

    pub fn clear_bit(&mut self, bit: u32) {
        self.bits &= !(1u64 << bit);
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.string_tags.iter().any(|t| t == tag)
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.has_tag(&tag) {
            self.string_tags.push(tag);
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Attributes {
    pub values: Vec<(AttrId, f32)>,
}

impl Attributes {
    pub fn get(&self, id: AttrId) -> Option<f32> {
        self.values.iter().find(|(k, _)| *k == id).map(|(_, v)| *v)
    }

    pub fn set(&mut self, id: AttrId, value: f32) {
        if let Some(entry) = self.values.iter_mut().find(|(k, _)| *k == id) {
            entry.1 = value;
        } else {
            self.values.push((id, value));
        }
    }

    pub fn remove(&mut self, id: AttrId) {
        self.values.retain(|(k, _)| *k != id);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusEffect {
    pub id: u16,
    pub stacks: u8,
    pub remaining: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StatusEffects {
    pub effects: Vec<StatusEffect>,
}

impl StatusEffects {
    pub fn add(&mut self, id: u16, stacks: u8, duration: f32) {
        if let Some(existing) = self.effects.iter_mut().find(|e| e.id == id) {
            existing.stacks = existing.stacks.saturating_add(stacks);
            existing.remaining = existing.remaining.max(duration);
        } else {
            self.effects.push(StatusEffect {
                id,
                stacks,
                remaining: duration,
            });
        }
    }

    pub fn tick(&mut self, dt: f32) {
        for effect in &mut self.effects {
            effect.remaining -= dt;
        }
        self.effects.retain(|e| e.remaining > 0.0);
    }

    pub fn has(&self, id: u16) -> bool {
        self.effects.iter().any(|e| e.id == id)
    }

    pub fn stacks(&self, id: u16) -> u8 {
        self.effects
            .iter()
            .find(|e| e.id == id)
            .map_or(0, |e| e.stacks)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Variant {
    Float(f32),
    Int(i64),
    Bool(bool),
    Str(String),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Blackboard {
    pub entries: HashMap<String, Variant>,
}

impl Blackboard {
    pub fn get_float(&self, key: &str) -> Option<f32> {
        match self.entries.get(key) {
            Some(Variant::Float(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.entries.get(key) {
            Some(Variant::Int(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.entries.get(key) {
            Some(Variant::Bool(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        match self.entries.get(key) {
            Some(Variant::Str(v)) => Some(v.as_str()),
            _ => None,
        }
    }

    pub fn set(&mut self, key: String, value: Variant) {
        self.entries.insert(key, value);
    }
}
