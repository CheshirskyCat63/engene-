use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::world::damage_profiles::{DecalProfileId, DebrisProfileId, FracturePatternId, GoreResponseId};

pub type MaterialId = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResponseClass {
    BrittleCeramic,
    BrittleGlassRadial,
    LayeredMasonry,
    AnisotropicWood,
    DuctileMetal,
    BiologicalSoft,
    BiologicalHard,
    Ite,
    Composite,
}

impl ResponseClass {
    pub fn is_stone_like(self) -> bool {
        matches!(self, Self::BrittleCeramic | Self::Ite | Self::LayeredMasonry)
    }

    pub fn is_biological(self) -> bool {
        matches!(self, Self::BiologicalSoft | Self::BiologicalHard)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SurfaceMaterial {
    pub id: MaterialId,
    pub name: String,
    pub response_class: ResponseClass,
    pub compressive_strength: f32,
    pub tensile_strength: f32,
    pub shear_strength: f32,
    pub brittleness: f32,
    pub density: f32,
    pub elasticity: f32,
    pub penetration_resistance: f32,
    pub hardness: f32,
    pub flammability: f32,
    pub fuel_content: f32,
    pub ignition_temp: f32,
    pub thermal_conductivity: f32,
    pub porosity: f32,
    pub erosion_resistance: f32,
    pub fragmentation_coeff: f32,
    pub fracture_pattern: FracturePatternId,
    pub debris_profile: DebrisProfileId,
    pub decal_profile: DecalProfileId,
    pub dust_intensity: f32,
    pub is_biological: bool,
    pub gore_response: GoreResponseId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SurfaceDbData {
    pub schema_version: u32,
    pub materials: Vec<SurfaceMaterial>,
}

pub struct SurfaceDB {
    materials: Vec<SurfaceMaterial>,
    name_to_id: HashMap<String, MaterialId>,
}

impl SurfaceDB {
    pub fn new(materials: Vec<SurfaceMaterial>) -> Self {
        let name_to_id = materials
            .iter()
            .map(|m| (m.name.clone(), m.id))
            .collect();
        Self {
            materials,
            name_to_id,
        }
    }

    pub fn get(&self, id: MaterialId) -> Option<&SurfaceMaterial> {
        self.materials.iter().find(|m| m.id == id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&SurfaceMaterial> {
        self.name_to_id
            .get(name)
            .and_then(|id| self.get(*id))
    }

    pub fn id_by_name(&self, name: &str) -> Option<MaterialId> {
        self.name_to_id.get(name).copied()
    }

    pub fn all_materials(&self) -> &[SurfaceMaterial] {
        &self.materials
    }

    pub fn len(&self) -> usize {
        self.materials.len()
    }

    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }
}
