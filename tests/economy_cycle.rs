#[test]
fn economy_dashboard_empty() {
    use engene::tools::economy_dashboard::EconomyDashboard;

    let dashboard = EconomyDashboard::default();
    assert!(dashboard.latest().is_none());
}

#[test]
fn economy_dashboard_record_snapshot() {
    use engene::core::ecs::Ecs;
    use engene::tools::economy_dashboard::EconomyDashboard;
    use engene::world::components::{EntityKind, NpcEconomy, Job};

    let mut ecs = Ecs::new();
    let e = ecs.spawn();
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.npc_economies.insert(e, NpcEconomy {
        money: 100.0,
        monthly_required: 50.0,
        job: Job::Guard,
        desperation: 0.0,
    });

    let mut dashboard = EconomyDashboard::default();
    dashboard.record_snapshot(&ecs, 1);

    let latest = dashboard.latest();
    assert!(latest.is_some());
    let snap = latest.unwrap();
    assert_eq!(snap.npc_count, 1);
    assert!((snap.total_money - 100.0).abs() < 1e-6);
}

#[test]
fn economy_dashboard_wealth_trend() {
    use engene::core::ecs::Ecs;
    use engene::tools::economy_dashboard::EconomyDashboard;
    use engene::world::components::{EntityKind, NpcEconomy, Job};

    let mut ecs = Ecs::new();
    let e = ecs.spawn();
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.npc_economies.insert(e, NpcEconomy {
        money: 50.0,
        monthly_required: 50.0,
        job: Job::Guard,
        desperation: 0.0,
    });

    let mut dashboard = EconomyDashboard::new(10);
    dashboard.record_snapshot(&ecs, 1);

    ecs.npc_economies.get_mut(&e).unwrap().money = 80.0;
    dashboard.record_snapshot(&ecs, 2);

    ecs.npc_economies.get_mut(&e).unwrap().money = 120.0;
    dashboard.record_snapshot(&ecs, 3);

    let trend = dashboard.wealth_trend(3);
    assert_eq!(trend.len(), 3);
    assert!((trend[0] - 50.0).abs() < 1e-6);
    assert!((trend[1] - 80.0).abs() < 1e-6);
    assert!((trend[2] - 120.0).abs() < 1e-6);
}

#[test]
fn sim_telemetry_initial() {
    use engene::core::perf::sim_telemetry::SimTelemetry;

    let telemetry = SimTelemetry::new();
    assert_eq!(telemetry.entity_births, 0);
    assert_eq!(telemetry.entity_deaths, 0);
    assert_eq!(telemetry.combat_events, 0);
    assert_eq!(telemetry.quests_completed, 0);
    assert_eq!(telemetry.bankruptcies, 0);
    assert_eq!(telemetry.reproduction_events, 0);
}

#[test]
fn sim_metrics_dashboard_population_trend() {
    use engene::tools::sim_metrics_dashboard::SimMetricsDashboard;

    let dashboard = SimMetricsDashboard::default();
    let trend = dashboard.population_trend(5);
    assert!(trend.is_empty());
}
