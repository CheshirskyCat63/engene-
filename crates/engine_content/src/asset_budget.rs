/// Estimated resource cost of a single asset
#[derive(Clone, Debug, Default)]
pub struct AssetBudget {
    pub vram_bytes: u64,
    pub cpu_skinning_ms: f32,
    pub draw_calls: u32,
    pub triangle_count: u32,
    pub overdraw_risk: OverdrawRisk,
    pub shadow_cost: ShadowCost,
    pub streaming_size_bytes: u64,
    pub destruction_cost: DestructionCost,
    pub audio_concurrency: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverdrawRisk {
    #[default]
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShadowCost {
    #[default]
    None,
    Light,
    Medium,
    Heavy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DestructionCost {
    #[default]
    None,
    Light,
    Medium,
    Heavy,
}

/// Budget limits for a single chunk
#[derive(Clone, Debug)]
pub struct ChunkBudgetLimits {
    pub max_draw_calls: u32,
    pub max_triangles: u32,
    pub max_vram_bytes: u64,
    pub max_memory_bytes: u64,
    pub max_nav_polygons: u32,
    pub max_ai_entities: u32,
    pub max_streaming_bytes: u64,
}

impl Default for ChunkBudgetLimits {
    fn default() -> Self {
        Self {
            max_draw_calls: 2000,
            max_triangles: 500_000,
            max_vram_bytes: 256 * 1024 * 1024,
            max_memory_bytes: 128 * 1024 * 1024,
            max_nav_polygons: 10_000,
            max_ai_entities: 50,
            max_streaming_bytes: 32 * 1024 * 1024,
        }
    }
}

/// Result of validating a chunk against budgets
#[derive(Clone, Debug)]
pub struct ChunkBudgetReport {
    pub chunk_coord: (i32, i32),
    pub total_draw_calls: u32,
    pub total_triangles: u32,
    pub total_vram: u64,
    pub total_streaming: u64,
    pub ai_entity_count: u32,
    pub violations: Vec<BudgetViolation>,
}

#[derive(Clone, Debug)]
pub struct BudgetViolation {
    pub metric: String,
    pub actual: f64,
    pub limit: f64,
    pub severity: BudgetSeverity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BudgetSeverity {
    Warning,
    Error,
}

pub fn validate_chunk_budget(
    coord: (i32, i32),
    assets: &[AssetBudget],
    ai_count: u32,
    limits: &ChunkBudgetLimits,
) -> ChunkBudgetReport {
    let total_dc: u32 = assets.iter().map(|a| a.draw_calls).sum();
    let total_tri: u32 = assets.iter().map(|a| a.triangle_count).sum();
    let total_vram: u64 = assets.iter().map(|a| a.vram_bytes).sum();
    let total_stream: u64 = assets.iter().map(|a| a.streaming_size_bytes).sum();

    let mut violations = Vec::new();

    if total_dc > limits.max_draw_calls {
        violations.push(BudgetViolation {
            metric: "draw_calls".into(),
            actual: total_dc as f64,
            limit: limits.max_draw_calls as f64,
            severity: BudgetSeverity::Error,
        });
    }
    if total_tri > limits.max_triangles {
        violations.push(BudgetViolation {
            metric: "triangles".into(),
            actual: total_tri as f64,
            limit: limits.max_triangles as f64,
            severity: BudgetSeverity::Error,
        });
    }
    if total_vram > limits.max_vram_bytes {
        violations.push(BudgetViolation {
            metric: "vram_bytes".into(),
            actual: total_vram as f64,
            limit: limits.max_vram_bytes as f64,
            severity: BudgetSeverity::Error,
        });
    }
    if ai_count > limits.max_ai_entities {
        violations.push(BudgetViolation {
            metric: "ai_entities".into(),
            actual: ai_count as f64,
            limit: limits.max_ai_entities as f64,
            severity: BudgetSeverity::Warning,
        });
    }

    ChunkBudgetReport {
        chunk_coord: coord,
        total_draw_calls: total_dc,
        total_triangles: total_tri,
        total_vram,
        total_streaming: total_stream,
        ai_entity_count: ai_count,
        violations,
    }
}

/// Production chunk standard checklist
#[derive(Clone, Debug)]
pub struct ProductionChunkChecklist {
    pub has_nav_coverage: bool,
    pub has_cover_coverage: bool,
    pub has_ambience_config: bool,
    pub has_material_truth: bool,
    pub has_destruction_tags: bool,
    pub has_prefab_validation: bool,
    pub has_streaming_budget: bool,
    pub has_low_spec_fallback: bool,
    pub has_test_patrol_route: bool,
    pub has_replay_spawn_anchor: bool,
}

impl ProductionChunkChecklist {
    pub fn is_production_ready(&self) -> bool {
        self.has_nav_coverage
            && self.has_cover_coverage
            && self.has_material_truth
            && self.has_prefab_validation
            && self.has_streaming_budget
    }

    pub fn missing_items(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if !self.has_nav_coverage {
            missing.push("nav_coverage");
        }
        if !self.has_cover_coverage {
            missing.push("cover_coverage");
        }
        if !self.has_ambience_config {
            missing.push("ambience_config");
        }
        if !self.has_material_truth {
            missing.push("material_truth");
        }
        if !self.has_destruction_tags {
            missing.push("destruction_tags");
        }
        if !self.has_prefab_validation {
            missing.push("prefab_validation");
        }
        if !self.has_streaming_budget {
            missing.push("streaming_budget");
        }
        if !self.has_low_spec_fallback {
            missing.push("low_spec_fallback");
        }
        if !self.has_test_patrol_route {
            missing.push("test_patrol_route");
        }
        if !self.has_replay_spawn_anchor {
            missing.push("replay_spawn_anchor");
        }
        missing
    }
}
