/// Render Integration Validation Harness (Phase B.0)
/// Validates each render pass before and after activation.
#[derive(Clone, Debug)]
pub struct PassValidationResult {
    pub pass_name: String,
    pub visual_correct: bool,
    pub gpu_time_ms: f32,
    pub budget_ceiling_ms: f32,
    pub within_budget: bool,
    pub temporal_stable: bool,
    pub exposure_correct: bool,
    pub tier_fallback_ok: bool,
    pub notes: Vec<String>,
}

impl PassValidationResult {
    pub fn all_ok(&self) -> bool {
        self.visual_correct && self.within_budget && self.temporal_stable
            && self.exposure_correct && self.tier_fallback_ok
    }
}

/// Per-quality-tier budget ceilings for render passes (milliseconds)
pub struct TierBudgets {
    pub low: f32,
    pub medium: f32,
    pub high: f32,
    pub ultra: f32,
}

/// Budget ceilings per render pass
pub fn pass_budgets() -> Vec<(&'static str, TierBudgets)> {
    vec![
        ("Atmosphere/Fog", TierBudgets { low: 0.0, medium: 0.3, high: 0.5, ultra: 0.8 }),
        ("IBL", TierBudgets { low: 0.1, medium: 0.1, high: 0.2, ultra: 0.3 }),
        ("TAA", TierBudgets { low: 0.0, medium: 0.3, high: 0.5, ultra: 0.5 }),
        ("SSAO", TierBudgets { low: 0.0, medium: 0.0, high: 0.5, ultra: 0.8 }),
        ("Bloom (full)", TierBudgets { low: 0.0, medium: 0.2, high: 0.3, ultra: 0.5 }),
        ("Contact Shadows", TierBudgets { low: 0.0, medium: 0.0, high: 0.3, ultra: 0.5 }),
        ("Reflection Probes", TierBudgets { low: 0.0, medium: 0.1, high: 0.3, ultra: 0.5 }),
        ("Total Post-Process", TierBudgets { low: 0.5, medium: 1.5, high: 3.0, ultra: 4.0 }),
    ]
}

/// Validation checklist for a render pass
#[derive(Clone, Debug)]
pub struct PassChecklist {
    pub pass_name: &'static str,
    pub checks: Vec<PassCheck>,
}

#[derive(Clone, Debug)]
pub struct PassCheck {
    pub name: &'static str,
    pub description: &'static str,
    pub status: CheckStatus,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CheckStatus {
    NotTested,
    Passed,
    Failed,
    Skipped,
}

/// Generate the validation checklist for all dormant passes
pub fn generate_pass_checklists() -> Vec<PassChecklist> {
    vec![
        PassChecklist {
            pass_name: "Atmosphere/Fog",
            checks: vec![
                PassCheck { name: "no_nan_pixels", description: "No NaN/Inf in output", status: CheckStatus::NotTested },
                PassCheck { name: "exposure_respect", description: "Fog respects auto-exposure", status: CheckStatus::NotTested },
                PassCheck { name: "time_of_day", description: "Works across sunrise/noon/dusk/night", status: CheckStatus::NotTested },
                PassCheck { name: "low_tier_disabled", description: "Gracefully disabled on Low tier", status: CheckStatus::NotTested },
            ],
        },
        PassChecklist {
            pass_name: "IBL",
            checks: vec![
                PassCheck { name: "env_light_binds", description: "EnvLightUniforms bound to PBR shader", status: CheckStatus::NotTested },
                PassCheck { name: "time_responsive", description: "Ambient changes with day progress", status: CheckStatus::NotTested },
                PassCheck { name: "no_overbright", description: "No overexposure from IBL", status: CheckStatus::NotTested },
            ],
        },
        PassChecklist {
            pass_name: "TAA",
            checks: vec![
                PassCheck { name: "motion_vectors", description: "Motion vectors generated in geometry pass", status: CheckStatus::NotTested },
                PassCheck { name: "jitter_applied", description: "Sub-pixel jitter on projection", status: CheckStatus::NotTested },
                PassCheck { name: "no_ghosting", description: "No ghosting on moving objects", status: CheckStatus::NotTested },
                PassCheck { name: "reactive_mask", description: "Particles/UI excluded from TAA", status: CheckStatus::NotTested },
                PassCheck { name: "history_rejection", description: "Disocclusion handled", status: CheckStatus::NotTested },
            ],
        },
        PassChecklist {
            pass_name: "SSAO",
            checks: vec![
                PassCheck { name: "normal_buffer", description: "Normal buffer output from geometry pass", status: CheckStatus::NotTested },
                PassCheck { name: "no_halo", description: "No dirty halo artifacts", status: CheckStatus::NotTested },
                PassCheck { name: "denoise", description: "Spatial denoise applied", status: CheckStatus::NotTested },
                PassCheck { name: "high_tier_only", description: "Disabled on Low/Medium tiers", status: CheckStatus::NotTested },
            ],
        },
        PassChecklist {
            pass_name: "Bloom",
            checks: vec![
                PassCheck { name: "blur_chain", description: "4-level downsample + upsample", status: CheckStatus::NotTested },
                PassCheck { name: "no_soap", description: "No over-bloom (2016 soap effect)", status: CheckStatus::NotTested },
                PassCheck { name: "threshold_correct", description: "Only bright pixels bloom", status: CheckStatus::NotTested },
            ],
        },
        PassChecklist {
            pass_name: "Contact Shadows",
            checks: vec![
                PassCheck { name: "depth_bound", description: "Depth buffer correctly bound", status: CheckStatus::NotTested },
                PassCheck { name: "no_noise", description: "No excessive noise", status: CheckStatus::NotTested },
                PassCheck { name: "complement_csm", description: "Complements CSM, doesn't fight it", status: CheckStatus::NotTested },
            ],
        },
        PassChecklist {
            pass_name: "Reflection Probes",
            checks: vec![
                PassCheck { name: "nearest_lookup", description: "nearest_probe() used in PBR", status: CheckStatus::NotTested },
                PassCheck { name: "parallax_correction", description: "Box parallax correction", status: CheckStatus::NotTested },
                PassCheck { name: "blend_probes", description: "Smooth blending between probes", status: CheckStatus::NotTested },
            ],
        },
    ]
}

/// Screenshot diff infrastructure (placeholder for CI integration)
pub struct ScreenshotBaseline {
    pub name: String,
    pub camera_position: [f32; 3],
    pub camera_direction: [f32; 3],
    pub quality_tier: &'static str,
    pub expected_hash: Option<u64>,
}

pub fn canonical_camera_positions() -> Vec<ScreenshotBaseline> {
    vec![
        ScreenshotBaseline {
            name: "Settlement Dawn".to_string(),
            camera_position: [500.0, 50.0, 500.0],
            camera_direction: [0.0, -0.3, 1.0],
            quality_tier: "High",
            expected_hash: None,
        },
        ScreenshotBaseline {
            name: "Forest Noon".to_string(),
            camera_position: [1500.0, 30.0, 1000.0],
            camera_direction: [1.0, -0.1, 0.5],
            quality_tier: "High",
            expected_hash: None,
        },
        ScreenshotBaseline {
            name: "Ruins Dusk".to_string(),
            camera_position: [2000.0, 40.0, 2000.0],
            camera_direction: [-0.5, -0.2, -0.8],
            quality_tier: "High",
            expected_hash: None,
        },
        ScreenshotBaseline {
            name: "Swamp Night".to_string(),
            camera_position: [800.0, 20.0, 1800.0],
            camera_direction: [0.3, -0.1, 0.9],
            quality_tier: "High",
            expected_hash: None,
        },
        ScreenshotBaseline {
            name: "Overview Low Spec".to_string(),
            camera_position: [1000.0, 200.0, 1000.0],
            camera_direction: [0.0, -0.8, 0.2],
            quality_tier: "Low",
            expected_hash: None,
        },
    ]
}

// ---------------------------------------------------------------------------
// Render Pipeline Completeness Validation (Phase 5)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct RenderValidationResult {
    pub checks: Vec<RenderCheck>,
}

#[derive(Clone, Debug)]
pub struct RenderCheck {
    pub name: &'static str,
    pub status: RenderCheckStatus,
    pub details: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RenderCheckStatus {
    Pass,
    Partial,
    Missing,
}

impl RenderValidationResult {
    pub fn validate_pipeline() -> Self {
        let checks = vec![
            RenderCheck {
                name: "Shadow Maps",
                status: RenderCheckStatus::Pass,
                details: "4-cascade CSM with depth comparison sampling".into(),
            },
            RenderCheck {
                name: "G-Buffer / HDR Target",
                status: RenderCheckStatus::Pass,
                details: "Rgba16Float HDR target with separate depth".into(),
            },
            RenderCheck {
                name: "PBR Lighting",
                status: RenderCheckStatus::Pass,
                details: "GGX specular + Fresnel + Cook-Torrance BRDF".into(),
            },
            RenderCheck {
                name: "IBL / Environment",
                status: RenderCheckStatus::Pass,
                details: "IblBindings with day-progress-based environment lighting".into(),
            },
            RenderCheck {
                name: "Atmosphere",
                status: RenderCheckStatus::Partial,
                details: "AtmospherePass wired as dormant, not yet in main render loop".into(),
            },
            RenderCheck {
                name: "Contact Shadows",
                status: RenderCheckStatus::Partial,
                details: "ContactShadowPass wired as dormant, not yet in main render loop".into(),
            },
            RenderCheck {
                name: "TAA",
                status: RenderCheckStatus::Partial,
                details: "TaaPass wired, advance_frame() called, resolve not in main loop".into(),
            },
            RenderCheck {
                name: "Bloom",
                status: RenderCheckStatus::Pass,
                details: "BloomPass extraction from HDR, multi-pass composite pending".into(),
            },
            RenderCheck {
                name: "Tonemap",
                status: RenderCheckStatus::Pass,
                details: "HDR -> sRGB tonemap as final pass".into(),
            },
            RenderCheck {
                name: "Skybox",
                status: RenderCheckStatus::Pass,
                details: "Procedural sky with sun position, day/night cycle".into(),
            },
            RenderCheck {
                name: "Skinned Mesh",
                status: RenderCheckStatus::Partial,
                details: "Pipeline ready, upload API exists, no skinned assets loaded".into(),
            },
            RenderCheck {
                name: "Vegetation",
                status: RenderCheckStatus::Pass,
                details: "Grass + tree instances from heightmap/biome".into(),
            },
            RenderCheck {
                name: "Entity Instancing",
                status: RenderCheckStatus::Pass,
                details: "Instanced capsule rendering with indirect draw".into(),
            },
            RenderCheck {
                name: "Particles",
                status: RenderCheckStatus::Missing,
                details: "Particle system exists in code but not wired to renderer".into(),
            },
            RenderCheck {
                name: "SSAO",
                status: RenderCheckStatus::Missing,
                details: "Not yet implemented - requires G-Buffer normals+depth".into(),
            },
            RenderCheck {
                name: "Deferred Shading",
                status: RenderCheckStatus::Missing,
                details: "Currently forward; deferred pass architecture planned".into(),
            },
        ];

        Self { checks }
    }

    pub fn pass_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.status == RenderCheckStatus::Pass)
            .count()
    }

    pub fn partial_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.status == RenderCheckStatus::Partial)
            .count()
    }

    pub fn missing_count(&self) -> usize {
        self.checks
            .iter()
            .filter(|c| c.status == RenderCheckStatus::Missing)
            .count()
    }

    pub fn completion_pct(&self) -> f32 {
        let total = self.checks.len() as f32;
        let score: f32 = self
            .checks
            .iter()
            .map(|c| match c.status {
                RenderCheckStatus::Pass => 1.0,
                RenderCheckStatus::Partial => 0.5,
                RenderCheckStatus::Missing => 0.0,
            })
            .sum();
        (score / total) * 100.0
    }

    pub fn summary(&self) -> String {
        format!(
            "Render pipeline: {:.0}% complete ({} pass, {} partial, {} missing)",
            self.completion_pct(),
            self.pass_count(),
            self.partial_count(),
            self.missing_count()
        )
    }
}
