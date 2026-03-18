//! Material truth service: unified query layer over MaterialBridge with fallbacks.

use crate::world::material_bridge::{
    AudioMaterialMapping, MaterialBridge, MaterialBridgeData, ParticleMaterialMapping,
    RenderMaterialMapping,
};
use crate::world::surface_db::{MaterialId, SurfaceDB};

pub struct MaterialTruthService {
    bridge: MaterialBridge,
    fallback_render: RenderMaterialMapping,
    fallback_audio: AudioMaterialMapping,
    fallback_particle: ParticleMaterialMapping,
}

impl MaterialTruthService {
    pub fn new(bridge: MaterialBridge) -> Self {
        let fallback_render = RenderMaterialMapping {
            material_id: 0,
            base_albedo_tint: [0.5, 0.5, 0.5],
            roughness_range: (0.5, 0.8),
            metallic: 0.0,
            normal_intensity: 1.0,
            subsurface: 0.0,
        };
        let fallback_audio = AudioMaterialMapping {
            material_id: 0,
            impact_sound_class: "GenericImpact".into(),
            footstep_sound_class: "GenericFootstep".into(),
            scrape_sound_class: "GenericScrape".into(),
            break_sound_class: "GenericBreak".into(),
        };
        let fallback_particle = ParticleMaterialMapping {
            material_id: 0,
            debris_color: [0.4, 0.4, 0.4],
            debris_size_range: (0.05, 0.2),
            dust_color: [0.6, 0.6, 0.6],
            dust_density: 0.5,
            spark_on_impact: false,
        };
        Self {
            bridge,
            fallback_render,
            fallback_audio,
            fallback_particle,
        }
    }

    /// Create an empty MaterialTruthService (no mappings, uses fallbacks for all queries).
    pub fn empty() -> Self {
        Self::new(MaterialBridge::from_data(MaterialBridgeData {
            render: vec![],
            audio: vec![],
            particle: vec![],
        }))
    }

    pub fn query_render(&self, material_id: MaterialId) -> &RenderMaterialMapping {
        self.bridge
            .get_render(material_id)
            .unwrap_or(&self.fallback_render)
    }

    pub fn query_audio(&self, material_id: MaterialId) -> &AudioMaterialMapping {
        self.bridge
            .get_audio(material_id)
            .unwrap_or(&self.fallback_audio)
    }

    pub fn query_particle(&self, material_id: MaterialId) -> &ParticleMaterialMapping {
        self.bridge
            .get_particle(material_id)
            .unwrap_or(&self.fallback_particle)
    }

    /// Validates bridge against SurfaceDB; returns list of validation errors.
    pub fn validate(&self, surface_db: &SurfaceDB) -> Vec<String> {
        match crate::world::material_bridge::validate_material_bridge(surface_db, &self.bridge) {
            Ok(()) => vec![],
            Err(e) => e,
        }
    }
}
