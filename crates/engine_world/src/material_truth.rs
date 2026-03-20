//! Material truth service: unified query layer over MaterialBridge with fallbacks.

use crate::core::game_config::GameConfig;
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

    /// Build canonical runtime material truth from authored GameConfig bridge data.
    pub fn from_game_config(config: &GameConfig) -> Self {
        let data = MaterialBridgeData {
            render: config
                .material_bridge
                .render
                .iter()
                .map(|r| RenderMaterialMapping {
                    material_id: r.material_id as MaterialId,
                    base_albedo_tint: [
                        r.base_albedo_tint.0,
                        r.base_albedo_tint.1,
                        r.base_albedo_tint.2,
                    ],
                    roughness_range: r.roughness_range,
                    metallic: r.metallic,
                    normal_intensity: r.normal_intensity,
                    subsurface: r.subsurface,
                })
                .collect(),
            audio: config
                .material_bridge
                .audio
                .iter()
                .map(|a| AudioMaterialMapping {
                    material_id: a.material_id as MaterialId,
                    impact_sound_class: a.impact_sound_class.clone(),
                    footstep_sound_class: a.footstep_sound_class.clone(),
                    scrape_sound_class: a.scrape_sound_class.clone(),
                    break_sound_class: a.break_sound_class.clone(),
                })
                .collect(),
            particle: config
                .material_bridge
                .particle
                .iter()
                .map(|p| ParticleMaterialMapping {
                    material_id: p.material_id as MaterialId,
                    debris_color: [p.debris_color.0, p.debris_color.1, p.debris_color.2],
                    debris_size_range: p.debris_size_range,
                    dust_color: [p.dust_color.0, p.dust_color.1, p.dust_color.2],
                    dust_density: p.dust_density,
                    spark_on_impact: p.spark_on_impact,
                })
                .collect(),
        };
        Self::new(MaterialBridge::from_data(data))
    }

    /// Canonical wiring guard helper:
    /// true means all three channels for this material resolve to fallback.
    pub fn is_fallback_primary_for(&self, material_id: MaterialId) -> bool {
        let r = self.query_render(material_id);
        let a = self.query_audio(material_id);
        let p = self.query_particle(material_id);
        r.material_id == self.fallback_render.material_id
            && a.impact_sound_class == self.fallback_audio.impact_sound_class
            && p.material_id == self.fallback_particle.material_id
    }

    /// Validates bridge against SurfaceDB; returns list of validation errors.
    pub fn validate(&self, surface_db: &SurfaceDB) -> Vec<String> {
        match crate::world::material_bridge::validate_material_bridge(surface_db, &self.bridge) {
            Ok(()) => vec![],
            Err(e) => e,
        }
    }
}
