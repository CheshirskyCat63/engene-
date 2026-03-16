use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::world::surface_db::{MaterialId, SurfaceDB};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationMode {
    Strict,
    ReportAll,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderMaterialMapping {
    pub material_id: MaterialId,
    pub base_albedo_tint: [f32; 3],
    pub roughness_range: (f32, f32),
    pub metallic: f32,
    pub normal_intensity: f32,
    pub subsurface: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioMaterialMapping {
    pub material_id: MaterialId,
    pub impact_sound_class: String,
    pub footstep_sound_class: String,
    pub scrape_sound_class: String,
    pub break_sound_class: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParticleMaterialMapping {
    pub material_id: MaterialId,
    pub debris_color: [f32; 3],
    pub debris_size_range: (f32, f32),
    pub dust_color: [f32; 3],
    pub dust_density: f32,
    pub spark_on_impact: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialBridgeData {
    pub render: Vec<RenderMaterialMapping>,
    pub audio: Vec<AudioMaterialMapping>,
    pub particle: Vec<ParticleMaterialMapping>,
}

pub struct MaterialBridge {
    pub render_map: HashMap<MaterialId, RenderMaterialMapping>,
    pub audio_map: HashMap<MaterialId, AudioMaterialMapping>,
    pub particle_map: HashMap<MaterialId, ParticleMaterialMapping>,
}

impl MaterialBridge {
    pub fn from_data(data: MaterialBridgeData) -> Self {
        Self {
            render_map: data.render.into_iter().map(|r| (r.material_id, r)).collect(),
            audio_map: data.audio.into_iter().map(|a| (a.material_id, a)).collect(),
            particle_map: data.particle.into_iter().map(|p| (p.material_id, p)).collect(),
        }
    }

    pub fn get_render(&self, id: MaterialId) -> Option<&RenderMaterialMapping> {
        self.render_map.get(&id)
    }

    pub fn get_audio(&self, id: MaterialId) -> Option<&AudioMaterialMapping> {
        self.audio_map.get(&id)
    }

    pub fn get_particle(&self, id: MaterialId) -> Option<&ParticleMaterialMapping> {
        self.particle_map.get(&id)
    }
}

pub fn validate_material_bridge(
    surface_db: &SurfaceDB,
    bridge: &MaterialBridge,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for mat in surface_db.all_materials() {
        if !bridge.render_map.contains_key(&mat.id) {
            errors.push(format!("Material '{}' (id={}) missing RenderMaterialMapping", mat.name, mat.id));
        }
        if !bridge.audio_map.contains_key(&mat.id) {
            errors.push(format!("Material '{}' (id={}) missing AudioMaterialMapping", mat.name, mat.id));
        }
        if !bridge.particle_map.contains_key(&mat.id) {
            errors.push(format!("Material '{}' (id={}) missing ParticleMaterialMapping", mat.name, mat.id));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validate_material_consistency(
    surface_db: &SurfaceDB,
    bridge: &MaterialBridge,
) -> Vec<String> {
    let mut warnings = Vec::new();
    for mat in surface_db.all_materials() {
        if let Some(render) = bridge.render_map.get(&mat.id) {
            if mat.response_class.is_biological() && render.metallic > 0.3 {
                warnings.push(format!(
                    "'{}': biological material with metallic={} (expected < 0.3)",
                    mat.name, render.metallic
                ));
            }
        }
        if let Some(particle) = bridge.particle_map.get(&mat.id) {
            if mat.brittleness > 0.8 && particle.dust_density < 0.1 {
                warnings.push(format!(
                    "'{}': highly brittle (brittleness={}) but low dust_density={}",
                    mat.name, mat.brittleness, particle.dust_density
                ));
            }
        }
        if let Some(audio) = bridge.audio_map.get(&mat.id) {
            if mat.response_class.is_stone_like() && audio.scrape_sound_class == "ClothScrape" {
                warnings.push(format!(
                    "'{}': stone-like material with cloth scrape sound",
                    mat.name
                ));
            }
        }
    }
    warnings
}
