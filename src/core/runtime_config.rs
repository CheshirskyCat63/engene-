#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeProfile {
    Sandbox,
    VerticalSlice,
    /// Final game, all optimizations on, no debug overhead
    Shipping,
    /// Targets 10-year-old hardware: reduced draw distance, simplified AI, no SSAO
    LowSpec,
    /// Editor/tools mode with debug panels and profiler
    DebugTools,
    /// No GPU, pure simulation for dedicated server
    HeadlessServer,
    Tools,
}

/// What quality tier a system should use
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum QualityTier {
    /// Minimal: skip non-essential work
    Low,
    /// Balanced: good visuals, reasonable perf
    Medium,
    /// Full: all features enabled
    High,
    /// Everything on, debug overlays included
    Ultra,
}

/// Per-subsystem budgets derived from the runtime profile
#[derive(Clone, Debug)]
pub struct ProfileBudgets {
    pub quality: QualityTier,
    pub max_visible_npcs: usize,
    pub ai_think_budget_ms: f64,
    pub render_budget_ms: f64,
    pub physics_budget_ms: f64,
    pub streaming_radius: f32,
    pub shadow_cascades: u32,
    pub enable_ssao: bool,
    pub enable_volumetrics: bool,
    pub enable_detailed_decals: bool,
    pub enable_debug_ui: bool,
    pub max_particles: usize,
    pub far_animation_lod: bool,
}

impl ProfileBudgets {
    pub fn for_profile(profile: &RuntimeProfile) -> Self {
        match profile {
            RuntimeProfile::Shipping => Self {
                quality: QualityTier::High,
                max_visible_npcs: 200,
                ai_think_budget_ms: 4.0,
                render_budget_ms: 12.0,
                physics_budget_ms: 4.0,
                streaming_radius: 4000.0,
                shadow_cascades: 4,
                enable_ssao: true,
                enable_volumetrics: true,
                enable_detailed_decals: true,
                enable_debug_ui: false,
                max_particles: 10000,
                far_animation_lod: true,
            },
            RuntimeProfile::LowSpec => Self {
                quality: QualityTier::Low,
                max_visible_npcs: 50,
                ai_think_budget_ms: 2.0,
                render_budget_ms: 8.0,
                physics_budget_ms: 2.0,
                streaming_radius: 2000.0,
                shadow_cascades: 2,
                enable_ssao: false,
                enable_volumetrics: false,
                enable_detailed_decals: false,
                enable_debug_ui: false,
                max_particles: 2000,
                far_animation_lod: false,
            },
            RuntimeProfile::DebugTools => Self {
                quality: QualityTier::Medium,
                max_visible_npcs: 100,
                ai_think_budget_ms: 6.0,
                render_budget_ms: 10.0,
                physics_budget_ms: 4.0,
                streaming_radius: 3000.0,
                shadow_cascades: 3,
                enable_ssao: true,
                enable_volumetrics: false,
                enable_detailed_decals: true,
                enable_debug_ui: true,
                max_particles: 5000,
                far_animation_lod: true,
            },
            RuntimeProfile::HeadlessServer => Self {
                quality: QualityTier::Low,
                max_visible_npcs: 500,
                ai_think_budget_ms: 8.0,
                render_budget_ms: 0.0,
                physics_budget_ms: 6.0,
                streaming_radius: 8000.0,
                shadow_cascades: 0,
                enable_ssao: false,
                enable_volumetrics: false,
                enable_detailed_decals: false,
                enable_debug_ui: false,
                max_particles: 0,
                far_animation_lod: false,
            },
            _ => Self {
                quality: QualityTier::Medium,
                max_visible_npcs: 100,
                ai_think_budget_ms: 4.0,
                render_budget_ms: 10.0,
                physics_budget_ms: 4.0,
                streaming_radius: 3000.0,
                shadow_cascades: 3,
                enable_ssao: true,
                enable_volumetrics: false,
                enable_detailed_decals: true,
                enable_debug_ui: true,
                max_particles: 5000,
                far_animation_lod: true,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RendererBackend {
    Wgpu,
    Headless,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayMode {
    Off,
    Capture,
    Playback,
    Validate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkingMode {
    Standalone,
    Server,
    Client,
}

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub profile: RuntimeProfile,
    pub fixed_tick_rate: f32,
    pub enabled_plugins: Vec<String>,
    pub renderer_backend: RendererBackend,
    pub budget_profile: String,
    pub debug_channels: Vec<String>,
    pub replay_mode: ReplayMode,
    pub networking_mode: NetworkingMode,
    pub streaming_radius_override: Option<f32>,
    pub deterministic_seed: Option<u64>,
    pub world_size_override: Option<(u32, u32)>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            profile: RuntimeProfile::VerticalSlice,
            fixed_tick_rate: 20.0,
            enabled_plugins: Vec::new(),
            renderer_backend: RendererBackend::Wgpu,
            budget_profile: "default".to_string(),
            debug_channels: Vec::new(),
            replay_mode: ReplayMode::Off,
            networking_mode: NetworkingMode::Standalone,
            streaming_radius_override: None,
            deterministic_seed: None,
            world_size_override: None,
        }
    }
}

impl RuntimeConfig {
    pub fn sandbox() -> Self {
        Self {
            profile: RuntimeProfile::Sandbox,
            ..Default::default()
        }
    }

    pub fn headless() -> Self {
        Self {
            profile: RuntimeProfile::HeadlessServer,
            renderer_backend: RendererBackend::Headless,
            ..Default::default()
        }
    }

    pub fn tools() -> Self {
        Self {
            profile: RuntimeProfile::Tools,
            ..Default::default()
        }
    }

    pub fn shipping() -> Self {
        Self {
            profile: RuntimeProfile::Shipping,
            ..Default::default()
        }
    }

    pub fn low_spec() -> Self {
        Self {
            profile: RuntimeProfile::LowSpec,
            ..Default::default()
        }
    }

    pub fn debug_tools() -> Self {
        Self {
            profile: RuntimeProfile::DebugTools,
            ..Default::default()
        }
    }

    pub fn budgets(&self) -> ProfileBudgets {
        ProfileBudgets::for_profile(&self.profile)
    }

    pub fn is_headless(&self) -> bool {
        self.renderer_backend == RendererBackend::Headless
    }

    pub fn is_replay_active(&self) -> bool {
        self.replay_mode != ReplayMode::Off
    }
}
