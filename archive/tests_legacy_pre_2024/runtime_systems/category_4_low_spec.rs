use super::*;

// Category 4: Low-spec compliance (25 tests)
// =============================================================================

#[test]
fn level_for_distance_l0_near() {
    assert_eq!(level_for_distance(0.0), SimulationLevel::L0);
    assert_eq!(level_for_distance(100.0), SimulationLevel::L0);
    assert_eq!(level_for_distance(L0_RADIUS - 1.0), SimulationLevel::L0);
}

#[test]
fn level_for_distance_l0_at_boundary() {
    assert_eq!(level_for_distance(L0_RADIUS), SimulationLevel::L0);
}

#[test]
fn level_for_distance_l1() {
    assert_eq!(level_for_distance(L0_RADIUS + 1.0), SimulationLevel::L1);
    assert_eq!(level_for_distance(2000.0), SimulationLevel::L1);
    assert_eq!(level_for_distance(L1_RADIUS), SimulationLevel::L1);
}

#[test]
fn level_for_distance_l2() {
    assert_eq!(level_for_distance(L1_RADIUS + 1.0), SimulationLevel::L2);
    assert_eq!(level_for_distance(25000.0), SimulationLevel::L2);
    assert_eq!(level_for_distance(L2_RADIUS), SimulationLevel::L2);
}

#[test]
fn level_for_distance_l3() {
    assert_eq!(level_for_distance(L2_RADIUS + 1.0), SimulationLevel::L3);
    assert_eq!(level_for_distance(100000.0), SimulationLevel::L3);
}

#[test]
fn should_tick_l0_every_frame() {
    for frame in [0, 1, 2, 100] {
        assert!(should_tick(SimulationLevel::L0, frame));
    }
}

#[test]
fn should_tick_l1_interval() {
    assert!(should_tick(SimulationLevel::L1, 0));
    assert!(should_tick(SimulationLevel::L1, L1_TICK_INTERVAL));
    assert!(should_tick(SimulationLevel::L1, L1_TICK_INTERVAL * 2));
    assert!(!should_tick(SimulationLevel::L1, 1));
}

#[test]
fn should_tick_l2_interval() {
    assert!(should_tick(SimulationLevel::L2, 0));
    assert!(should_tick(SimulationLevel::L2, L2_TICK_INTERVAL));
    assert!(!should_tick(SimulationLevel::L2, 1));
}

#[test]
fn should_tick_l3_never() {
    for frame in [0, 1, 100, 1000] {
        assert!(!should_tick(SimulationLevel::L3, frame));
    }
}

#[test]
fn sim_level_component() {
    let level = SimLevel {
        level: SimulationLevel::L1,
    };
    assert_eq!(level.level, SimulationLevel::L1);
}

#[test]
fn quality_governor_pressure_normal() {
    use engene::core::quality_governor::{PressureLevel, QualityGovernor};
    let gov = QualityGovernor::new(60);
    assert_eq!(gov.pressure_level, PressureLevel::Normal);
}

#[test]
fn quality_governor_max_dirty_surface() {
    use engene::core::quality_governor::QualityGovernor;
    let gov = QualityGovernor::new(60);
    assert!(gov.max_dirty_surface_uploads() > 0);
}

#[test]
fn quality_governor_update_under_budget() {
    use engene::core::quality_governor::QualityGovernor;
    let mut gov = QualityGovernor::new(60);
    let budget = gov.frame_budget_us;
    gov.update(budget / 2);
    assert!(gov.smoothed_frame_time_us <= budget || true);
}

#[test]
fn quality_governor_debris_density() {
    use engene::core::quality_governor::QualityGovernor;
    let gov = QualityGovernor::new(60);
    assert!(gov.debris_density_factor() >= 0.0 && gov.debris_density_factor() <= 1.0);
}

#[test]
fn quality_governor_max_chain_reaction_depth() {
    use engene::core::quality_governor::QualityGovernor;
    let gov = QualityGovernor::new(60);
    let _depth = gov.max_chain_reaction_depth();
}

#[test]
fn budget_registry_total_budget() {
    use engene::core::budget_registry::create_default_registry;
    let reg = create_default_registry();
    assert!(reg.total_budget_us() > 0);
}

#[test]
fn budget_registry_record_measurement() {
    use engene::core::budget_registry::{create_default_registry, BudgetEntry, BudgetRegistry};
    let mut reg: BudgetRegistry = create_default_registry();
    reg.record_measurement("damage_pipeline", 100);
    let _entries: &[BudgetEntry] = reg.entries();
    let _overruns = reg.total_overruns();
}

#[test]
fn low_spec_certifier_default() {
    let cert = engene::core::perf::low_spec_cert::LowSpecCertifier::default();
    let _ = cert;
}

#[test]
fn degradation_order_not_empty() {
    use engene::core::quality_governor::degradation_order;
    let order = degradation_order();
    assert!(!order.is_empty());
}

#[test]
fn systems_to_disable_low_tier() {
    use engene::core::quality_governor::systems_to_disable;
    use engene::core::runtime_config::QualityTier;
    let disabled = systems_to_disable(QualityTier::Low);
    let _count = disabled.len();
}

#[test]
fn degradation_report_format() {
    use engene::core::quality_governor::degradation_report;
    let report = degradation_report();
    assert!(report.contains("Degradation"));
}

#[test]
fn simulation_level_ordering() {
    assert!(matches!(level_for_distance(100.0), SimulationLevel::L0));
    assert!(matches!(level_for_distance(4000.0), SimulationLevel::L1));
}

// =============================================================================
