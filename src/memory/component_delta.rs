use crate::core::ecs::Entity;
use crate::core::sparse_set::SparseSet;
use crate::world::components::Transform;
use serde::{Deserialize, Serialize};

pub const COMP_TRANSFORM: u8 = 0;
pub const NUM_COMPONENT_TYPES: usize = 18;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentDelta {
    pub component_type_id: u16,
    pub entity_ids: Vec<Entity>,
    pub data: Vec<u8>,
}

pub struct DirtyFlags {
    flags: Vec<u32>,
}

impl DirtyFlags {
    pub fn new(capacity: usize) -> Self {
        Self {
            flags: vec![0u32; capacity],
        }
    }

    pub fn default_capacity() -> Self {
        Self::new(16384)
    }

    pub fn mark(&mut self, entity_idx: usize, component_bit: u8) {
        if entity_idx < self.flags.len() {
            self.flags[entity_idx] |= 1 << component_bit;
        }
    }

    pub fn is_dirty(&self, entity_idx: usize, component_bit: u8) -> bool {
        if entity_idx >= self.flags.len() {
            return false;
        }
        (self.flags[entity_idx] & (1 << component_bit)) != 0
    }

    pub fn any_dirty(&self, entity_idx: usize) -> bool {
        entity_idx < self.flags.len() && self.flags[entity_idx] != 0
    }

    pub fn clear_all(&mut self) {
        for f in &mut self.flags {
            *f = 0;
        }
    }

    pub fn ensure_capacity(&mut self, cap: usize) {
        if cap > self.flags.len() {
            self.flags.resize(cap, 0);
        }
    }

    pub fn dirty_entities(&self) -> Vec<usize> {
        self.flags
            .iter()
            .enumerate()
            .filter(|(_, &f)| f != 0)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn component_bits(&self, entity_idx: usize) -> u32 {
        if entity_idx < self.flags.len() {
            self.flags[entity_idx]
        } else {
            0
        }
    }

    pub fn dirty_entities_for(&self, component_bit: u8) -> Vec<usize> {
        self.flags
            .iter()
            .enumerate()
            .filter(|(_, &f)| (f & (1 << component_bit)) != 0)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn clear(&mut self) {
        self.clear_all();
    }
}

impl Default for DirtyFlags {
    fn default() -> Self {
        Self::default_capacity()
    }
}

pub mod component_ids {
    pub const TRANSFORM: u8 = 0;
    pub const KIND: u8 = 1;
    pub const NAME: u8 = 2;
    pub const PERSONAL_NEEDS: u8 = 3;
    pub const SOCIAL_NEEDS: u8 = 4;
    pub const ECOSYSTEM_NEEDS: u8 = 5;
    pub const NPC_TRAITS: u8 = 6;
    pub const MONSTER_TRAITS: u8 = 7;
    pub const NPC_ECONOMY: u8 = 8;
    pub const SIM_LEVEL: u8 = 9;
    pub const AI_STATE: u8 = 10;
    pub const INVENTORY: u8 = 11;
    pub const MEMORY: u8 = 12;
    pub const EMOTIONS: u8 = 13;
    pub const PLAN: u8 = 14;
    pub const LIFE_INFO: u8 = 15;
    pub const FLAMMABLE: u8 = 16;
    pub const CLOTH: u8 = 17;
}

pub fn collect_dirty_transforms(
    dirty: &DirtyFlags,
    entities: &[Entity],
    ecs: &crate::core::ecs::Ecs,
) -> Option<ComponentDelta> {
    let mut ids = Vec::new();
    let mut data = Vec::new();

    for (idx, &entity) in entities.iter().enumerate() {
        if dirty.is_dirty(idx, component_ids::TRANSFORM) {
            if let Some(t) = ecs.transforms.get(&entity) {
                ids.push(entity);
                data.extend_from_slice(&t.x.to_le_bytes());
                data.extend_from_slice(&t.y.to_le_bytes());
                data.extend_from_slice(&t.cell_x.to_le_bytes());
                data.extend_from_slice(&t.cell_y.to_le_bytes());
            }
        }
    }

    if ids.is_empty() {
        None
    } else {
        Some(ComponentDelta {
            component_type_id: component_ids::TRANSFORM as u16,
            entity_ids: ids,
            data,
        })
    }
}

pub fn collect_dirty_needs(
    dirty: &DirtyFlags,
    entities: &[Entity],
    ecs: &crate::core::ecs::Ecs,
) -> Option<ComponentDelta> {
    let mut ids = Vec::new();
    let mut data = Vec::new();

    for (idx, &entity) in entities.iter().enumerate() {
        if dirty.is_dirty(idx, component_ids::PERSONAL_NEEDS) {
            if let Some(pn) = ecs.personal_needs.get(&entity) {
                ids.push(entity);
                data.extend_from_slice(&pn.hunger.to_le_bytes());
                data.extend_from_slice(&pn.thirst.to_le_bytes());
                data.extend_from_slice(&pn.health.to_le_bytes());
                data.extend_from_slice(&pn.energy.to_le_bytes());
                data.extend_from_slice(&pn.fear.to_le_bytes());
            }
        }
    }

    if ids.is_empty() {
        None
    } else {
        Some(ComponentDelta {
            component_type_id: component_ids::PERSONAL_NEEDS as u16,
            entity_ids: ids,
            data,
        })
    }
}

pub fn collect_transform_deltas(
    dirty_indices: &[usize],
    transforms: &SparseSet<Transform>,
) -> ComponentDelta {
    let mut ids = Vec::new();
    let mut data = Vec::new();

    for &idx in dirty_indices {
        let entity = idx as Entity;
        if let Some(t) = transforms.get(&entity) {
            ids.push(entity);
            data.extend_from_slice(&t.x.to_le_bytes());
            data.extend_from_slice(&t.y.to_le_bytes());
            data.extend_from_slice(&t.cell_x.to_le_bytes());
            data.extend_from_slice(&t.cell_y.to_le_bytes());
        }
    }

    ComponentDelta {
        component_type_id: COMP_TRANSFORM as u16,
        entity_ids: ids,
        data,
    }
}

pub fn apply_transform_deltas(delta: &ComponentDelta, transforms: &mut SparseSet<Transform>) {
    let stride = 16;
    for (i, &entity) in delta.entity_ids.iter().enumerate() {
        let off = i * stride;
        if off + stride > delta.data.len() {
            break;
        }
        let x = f32::from_le_bytes(delta.data[off..off + 4].try_into().unwrap_or([0; 4]));
        let y = f32::from_le_bytes(delta.data[off + 4..off + 8].try_into().unwrap_or([0; 4]));
        let cx = u32::from_le_bytes(delta.data[off + 8..off + 12].try_into().unwrap_or([0; 4]));
        let cy = u32::from_le_bytes(delta.data[off + 12..off + 16].try_into().unwrap_or([0; 4]));

        if let Some(t) = transforms.get_mut(&entity) {
            t.x = x;
            t.y = y;
            t.cell_x = cx;
            t.cell_y = cy;
        } else {
            transforms.insert(
                entity,
                Transform {
                    x,
                    y,
                    cell_x: cx,
                    cell_y: cy,
                },
            );
        }
    }
}
