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
    use engene::runtime::bootstrap::GameRuntimeAssembly;
    use engene::tools::doctor;
    use engene::world::heightmap::Heightmap;
    use engene::world::world::WorldGrid;
    use std::sync::Arc;

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);

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
    use engene::runtime::bootstrap::GameRuntimeAssembly;
    use engene::world::heightmap::Heightmap;
    use engene::world::world::WorldGrid;
    use std::sync::Arc;

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let mut engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);

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

#[test]
fn vertical_slice_ballistics_material_truth_comes_from_surfaces() {
    use engene::core::config::{load_config, ConfigEnvelope};
    use engene::core::game_config::{GameConfig, MaterialConfig};
    use engene::physics::ballistics::BallisticsSystem;
    use engene::runtime::bootstrap::GameRuntimeAssembly;
    use engene::world::heightmap::Heightmap;
    use engene::world::world::WorldGrid;
    use std::collections::HashMap;
    use std::sync::Arc;

    let config = GameConfig::load_from_dir("game/data");
    let surface = config
        .surfaces
        .materials
        .iter()
        .find(|m| m.name == "concrete")
        .expect("surfaces.ron must contain concrete");

    let legacy =
        load_config::<ConfigEnvelope<HashMap<String, MaterialConfig>>>("game/data/materials.ron")
            .expect("materials.ron must parse")
            .data
            .get("Concrete")
            .cloned()
            .expect("materials.ron must contain Concrete");

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    let ballistics = engine
        .resources
        .get::<BallisticsSystem>()
        .expect("BallisticsSystem resource must exist");

    let mat_id = *ballistics
        .material_table
        .name_to_id
        .get("concrete")
        .expect("ballistics table must include concrete from surfaces");
    let props = ballistics.material_table.get(mat_id).unwrap();

    assert!(
        (props.hardness - surface.hardness).abs() < 0.001,
        "hardness must come from surfaces.ron canonical material truth"
    );
    assert!(
        (props.penetration_resistance - surface.penetration_resistance).abs() < 0.001,
        "penetration_resistance must come from surfaces.ron canonical material truth"
    );
    assert!(
        (props.density - surface.density).abs() < 0.001,
        "density must come from surfaces.ron canonical material truth"
    );

    assert!(
        (props.hardness - legacy.hardness).abs() > 0.5
            || (props.penetration_resistance - legacy.penetration_resistance).abs() > 1.0
            || (props.density - legacy.density).abs() > 1.0,
        "ballistics material properties must not silently fall back to materials.ron legacy values"
    );
}

#[test]
fn vertical_slice_material_truth_service_uses_authored_bridge_and_fallback_only_for_missing() {
    use engene::core::game_config::GameConfig;
    use engene::core::material_truth::MaterialTruthService;
    use engene::runtime::bootstrap::GameRuntimeAssembly;
    use engene::world::heightmap::Heightmap;
    use engene::world::world::WorldGrid;
    use std::sync::Arc;

    let cfg = GameConfig::load_from_dir("game/data");
    let render0 = cfg
        .material_bridge
        .render
        .iter()
        .find(|m| m.material_id == 0)
        .expect("material_bridge.ron must have render mapping for id 0");
    let audio0 = cfg
        .material_bridge
        .audio
        .iter()
        .find(|m| m.material_id == 0)
        .expect("material_bridge.ron must have audio mapping for id 0");
    let particle0 = cfg
        .material_bridge
        .particle
        .iter()
        .find(|m| m.material_id == 0)
        .expect("material_bridge.ron must have particle mapping for id 0");

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    let truth = engine
        .resources
        .get::<MaterialTruthService>()
        .expect("MaterialTruthService must exist in vertical_slice runtime");

    let render = truth.query_render(0);
    let audio = truth.query_audio(0);
    let particle = truth.query_particle(0);

    assert_eq!(
        render.base_albedo_tint,
        [
            render0.base_albedo_tint.0,
            render0.base_albedo_tint.1,
            render0.base_albedo_tint.2
        ],
        "runtime render mapping must come from authored material_bridge.ron"
    );
    assert_eq!(
        audio.impact_sound_class, audio0.impact_sound_class,
        "runtime audio mapping must come from authored material_bridge.ron"
    );
    assert_eq!(
        particle.spark_on_impact, particle0.spark_on_impact,
        "runtime particle mapping must come from authored material_bridge.ron"
    );

    let missing = truth.query_audio(u16::MAX);
    assert_eq!(
        missing.impact_sound_class, "GenericImpact",
        "fallback should be used only when mapping is missing"
    );
}
