use serde::{Deserialize, Serialize};

pub type FracturePatternId = u16;
pub type DebrisProfileId = u16;
pub type DecalProfileId = u16;
pub type GoreResponseId = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FracturePattern {
    BrittleRadial,
    BrittleClustered,
    AnisotropicSplit,
    DuctileDent,
    LayeredTear,
    BiologicalRupture,
    Shatter,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebrisProfile {
    pub id: DebrisProfileId,
    pub name: String,
    pub min_count: u16,
    pub max_count: u16,
    pub min_mass: f32,
    pub max_mass: f32,
    pub spread_angle: f32,
    pub velocity_factor: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecalProfile {
    pub id: DecalProfileId,
    pub name: String,
    pub min_size: f32,
    pub max_size: f32,
    pub lifetime_seconds: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoreResponse {
    pub id: GoreResponseId,
    pub flesh_chunk_count: u8,
    pub blood_spray_intensity: f32,
    pub bone_fragment: bool,
}
