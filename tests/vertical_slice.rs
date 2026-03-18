#[test]
fn vertical_slice_world_loads() {
    use engene::world::authored_sets::WorldLayout;

    let layout = WorldLayout::vertical_slice_skeleton();
    assert!(layout.camps.len() >= 2);
    assert!(layout.habitats.len() >= 2);
    assert!(layout.roads.len() >= 1);
    assert!(layout.traders.len() >= 1);
}

#[test]
fn vertical_slice_engine_boots() {
    use engene::app::runtime_assembly::RuntimeAssembly;
    use engene::tools::doctor;
    use engene::world::heightmap::Heightmap;
    use engene::world::world::WorldGrid;
    use std::sync::Arc;

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = RuntimeAssembly::vertical_slice(heightmap, &biomes);

    assert!(engine.ecs.alive.len() > 0);
    assert!(engine.ecs.npcs().len() > 0);

    let report = doctor::run_doctor(&engine, doctor::DoctorMode::Advisory);
    assert_eq!(
        report.error_count(),
        0,
        "vertical slice should boot with 0 doctor errors"
    );
}

#[test]
fn vertical_slice_simulation_stable() {
    use engene::app::runtime_assembly::RuntimeAssembly;
    use engene::world::heightmap::Heightmap;
    use engene::world::world::WorldGrid;
    use std::sync::Arc;

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let mut engine = RuntimeAssembly::vertical_slice(heightmap, &biomes);

    let initial_entities = engine.ecs.alive.len();
    let sim_dt = 1.0 / 20.0;

    for _ in 0..200 {
        engine.tick(sim_dt);
    }

    let final_entities = engine.ecs.alive.len();
    let drift = (final_entities as f64 - initial_entities as f64).abs() / initial_entities as f64;
    assert!(
        drift < 0.5,
        "entity count drifted {:.0}% after 200 ticks",
        drift * 100.0
    );
}

#[test]
fn vertical_slice_player_controller() {
    use engene::game::player::PlayerController;

    let mut player = PlayerController::new([500.0, 10.0, 500.0]);
    assert!(player.is_alive());
    assert!(player.can_move());

    player.tick(0.1, false);
    assert!(player.health_fraction() > 0.99);

    player.take_damage(30.0);
    assert!((player.health - 70.0).abs() < 0.01);
    assert!(player.is_alive());

    player.take_damage(80.0);
    assert!(!player.is_alive());

    player.respawn([100.0, 5.0, 100.0]);
    assert!(player.is_alive());
    assert!(player.health_fraction() > 0.99);
}

#[test]
fn vertical_slice_hud() {
    use engene::game::hud::{HudState, NotificationKind};

    let mut hud = HudState::new();
    assert!(hud.show_health);
    assert!(hud.show_crosshair);

    hud.push_notification("Item picked up", NotificationKind::ItemPickup);
    assert_eq!(hud.notification_queue.len(), 1);

    hud.tick(4.0);
    assert_eq!(hud.notification_queue.len(), 0);
}

#[test]
fn vertical_slice_item_registry() {
    use engene::game::economy::item_registry::ItemRegistry;

    let registry = ItemRegistry::new();
    assert!(registry.get("medkit").is_some());
    assert!(registry.get("bread").is_some());
    assert!(registry.get("artifact_moonlight").is_some());

    let instance = registry.create_instance("medkit", 3);
    assert!(instance.is_some());
    let inst = instance.unwrap();
    assert_eq!(inst.stack_count, 3);
}
