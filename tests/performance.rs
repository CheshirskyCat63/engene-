#[test]
fn budget_registry_creation() {
    use engene::core::budget_registry::create_default_registry;

    let registry = create_default_registry();
    let entries = registry.entries();
    assert!(!entries.is_empty());
}

#[test]
fn quality_governor_initial_state() {
    use engene::core::quality_governor::QualityGovernor;

    let governor = QualityGovernor::new(60);
    assert_eq!(governor.frame_budget_us, 1_000_000 / 60);
}

#[test]
fn quality_governor_pressure_response() {
    use engene::core::quality_governor::{PressureLevel, QualityGovernor};

    let mut governor = QualityGovernor::new(60);
    for _ in 0..20 {
        governor.update(50_000);
    }
    assert!(
        governor.pressure_level != PressureLevel::Normal
            || governor.max_dirty_surface_uploads() < 16
    );
}

#[test]
fn runtime_config_default_profile() {
    use engene::core::runtime_config::{RuntimeConfig, RuntimeProfile};

    let config = RuntimeConfig::default();
    assert_eq!(config.profile, RuntimeProfile::Game);
}

#[test]
fn asset_budget_default_limits() {
    use engene::content::asset_budget::{AssetBudget, ChunkBudgetLimits};

    let _budget = AssetBudget::default();
    let limits = ChunkBudgetLimits::default();
    assert!(limits.max_draw_calls > 0);
    assert!(limits.max_triangles > 0);
    assert!(limits.max_vram_bytes > 0);
}
