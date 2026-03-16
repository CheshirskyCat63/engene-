use serde::{Deserialize, Serialize};

use crate::world::surface_db::MaterialId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectionDescriptor {
    pub id: u32,
    pub section_type: SectionType,
    pub material: MaterialId,
    pub thickness: f32,
    pub hero: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SectionType {
    Wall,
    Column,
    Roof,
    Floor,
    Door,
    Window,
    Foundation,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingDescriptor {
    pub sections: Vec<SectionDescriptor>,
    pub section_adjacency: Vec<(u32, u32, f32)>,
    pub hero_objects: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct StructuralSection {
    pub id: u32,
    pub node_ids: Vec<u32>,
    pub section_type: SectionType,
    pub integrity: f32,
    pub neighbors: Vec<SectionNeighbor>,
}

#[derive(Clone, Debug)]
pub struct SectionNeighbor {
    pub section_id: u32,
    pub load_transfer: f32,
    pub collapse_priority: u8,
}
