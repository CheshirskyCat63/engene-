#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QualityTier {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Clone, Debug)]
pub struct RenderSettings {
    pub tier: QualityTier,
    pub shadow_resolution: u32,
    pub shadow_cascades: usize,
    pub ssao_enabled: bool,
    pub bloom_enabled: bool,
    pub vegetation_density: f32,
    pub lod_bias: f32,
    pub draw_distance: f32,
}

impl RenderSettings {
    pub fn from_tier(tier: QualityTier) -> Self {
        match tier {
            QualityTier::Low => Self {
                tier,
                shadow_resolution: 512,
                shadow_cascades: 1,
                ssao_enabled: false,
                bloom_enabled: false,
                vegetation_density: 0.1,
                lod_bias: 2.0,
                draw_distance: 500.0,
            },
            QualityTier::Medium => Self {
                tier,
                shadow_resolution: 1024,
                shadow_cascades: 2,
                ssao_enabled: false,
                bloom_enabled: true,
                vegetation_density: 0.3,
                lod_bias: 1.0,
                draw_distance: 1000.0,
            },
            QualityTier::High => Self {
                tier,
                shadow_resolution: 2048,
                shadow_cascades: 3,
                ssao_enabled: true,
                bloom_enabled: true,
                vegetation_density: 0.7,
                lod_bias: 0.5,
                draw_distance: 2000.0,
            },
            QualityTier::Ultra => Self {
                tier,
                shadow_resolution: 4096,
                shadow_cascades: 4,
                ssao_enabled: true,
                bloom_enabled: true,
                vegetation_density: 1.0,
                lod_bias: 0.0,
                draw_distance: 5000.0,
            },
        }
    }
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self::from_tier(QualityTier::High)
    }
}
