#[test]
fn build_manifest_version() {
    use engene::core::build_manifest::BuildManifest;

    let manifest = BuildManifest::current();
    assert!(!manifest.engine_version.is_empty());
    assert!(!manifest.build_timestamp.is_empty());
    assert!(!manifest.profile.is_empty());
}

#[test]
fn schema_migration_registry_empty() {
    use engene::core::build_manifest::SchemaMigrationRegistry;

    let registry = SchemaMigrationRegistry::new();
    let result = registry.migrate_save(0, vec![]);
    assert!(result.is_err());
}

#[test]
fn persistence_dashboard_initial() {
    use engene::tools::persistence_dashboard::PersistenceDashboard;

    let dashboard = PersistenceDashboard::new();
    assert_eq!(dashboard.total_saves, 0);
    assert_eq!(dashboard.total_loads, 0);
}

#[test]
fn crash_bundle_creation() {
    use engene::core::crash_telemetry::CrashBundle;

    let bundle = CrashBundle {
        timestamp: "1234567890".to_string(),
        panic_message: "test panic".to_string(),
        location: Some("test.rs:10:5".to_string()),
        entity_count: Some(42),
        active_system: Some("TestSystem".to_string()),
        engine_version: "0.1.0".to_string(),
        profile: "dev".to_string(),
        recent_events: vec![],
        loaded_chunks: vec![],
    };
    assert_eq!(bundle.panic_message, "test panic");
    assert_eq!(bundle.engine_version, "0.1.0");
}

#[test]
fn determinism_policy_default() {
    use engene::core::determinism_policy::{DeterminismPolicyMatrix, DeterminismLevel};

    let matrix = DeterminismPolicyMatrix::build_default();
    assert_eq!(matrix.level_of("ECS tick order"), DeterminismLevel::Required);
    assert_eq!(matrix.level_of("Render order"), DeterminismLevel::Acceptable);
    assert_eq!(matrix.level_of("PhysicsSystem"), DeterminismLevel::Preferred);
}

#[test]
fn equipment_state_roundtrip() {
    use engene::core::ecs::Ecs;
    use engene::world::components::*;
    use engene::world::streaming::ChunkCoord;
    use engene::world::chunk_persistence::ChunkPersistenceService;
    let test_dir = std::env::temp_dir().join("engene_test_equip_rt");
    let _ = std::fs::remove_dir_all(&test_dir);

    let mut ecs = Ecs::new();
    let coord = ChunkCoord { x: 0, z: 0 };

    let (entity, pid) = ecs.spawn_new();
    assert!(pid.0 > 0, "PID should be positive");
    ecs.transforms.insert(entity, Transform { x: 100.0, y: 100.0, cell_x: 0, cell_y: 0 });
    ecs.kinds.insert(entity, EntityKind::Npc);
    let mut equip = EquipmentSlots::default_stalker();
    equip.weapon_condition = 0.75;
    equip.armor_condition = 0.5;
    equip.medkits = 3;
    equip.ammo = 45;
    ecs.equipment.insert(entity, equip);
    ecs.sim_levels.insert(entity, SimLevel { level: SimulationLevel::L0 });

    let mut persistence = ChunkPersistenceService::new(test_dir.to_str().unwrap());
    persistence.save_and_unload(coord, &mut ecs, 0);
    assert_eq!(ecs.alive.len(), 0);

    persistence.load_chunk_entities(coord, &mut ecs);
    assert_eq!(ecs.alive.len(), 1);

    let restored = ecs.alive[0];
    let restored_equip = ecs.equipment.get(&restored).expect("equipment missing after restore");
    assert!((restored_equip.weapon_condition - 0.75).abs() < 0.01);
    assert!((restored_equip.armor_condition - 0.5).abs() < 0.01);
    assert_eq!(restored_equip.medkits, 3);
    assert_eq!(restored_equip.ammo, 45);

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn faction_membership_roundtrip() {
    use engene::core::ecs::Ecs;
    use engene::world::components::*;
    use engene::world::streaming::ChunkCoord;
    use engene::world::chunk_persistence::ChunkPersistenceService;
    use engene::gameplay::factions::Faction;

    let test_dir = std::env::temp_dir().join("engene_test_faction_rt");
    let _ = std::fs::remove_dir_all(&test_dir);

    let mut ecs = Ecs::new();
    let coord = ChunkCoord { x: 0, z: 0 };

    let (entity, _pid) = ecs.spawn_new();
    ecs.transforms.insert(entity, Transform { x: 50.0, y: 50.0, cell_x: 0, cell_y: 0 });
    ecs.kinds.insert(entity, EntityKind::Npc);
    ecs.faction_memberships.insert(entity, FactionMembership {
        faction: Faction::Duty,
        standing: 0.8,
    });
    ecs.sim_levels.insert(entity, SimLevel { level: SimulationLevel::L0 });

    let mut persistence = ChunkPersistenceService::new(test_dir.to_str().unwrap());
    persistence.save_and_unload(coord, &mut ecs, 0);
    persistence.load_chunk_entities(coord, &mut ecs);

    let restored = ecs.alive[0];
    let fm = ecs.faction_memberships.get(&restored).expect("faction membership missing");
    assert_eq!(fm.faction, Faction::Duty);
    assert!((fm.standing - 0.8).abs() < 0.01);

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn surface_destruction_state_roundtrip() {
    use engene::world::streaming::ChunkCoord;
    use engene::world::chunk_persistence::ChunkPersistenceService;
    use engene::core::ecs::Ecs;
    use engene::world::components::*;

    let test_dir = std::env::temp_dir().join("engene_test_surface_rt");
    let _ = std::fs::remove_dir_all(&test_dir);

    let mut ecs = Ecs::new();
    let coord = ChunkCoord { x: 1, z: 1 };

    for i in 0..5u64 {
        let (entity, _) = ecs.spawn_new();
        ecs.transforms.insert(entity, Transform {
            x: 1000.0 + (i as f32) * 50.0,
            y: 1000.0 + (i as f32) * 50.0,
            cell_x: 1, cell_y: 1,
        });
        ecs.kinds.insert(entity, EntityKind::Npc);
        ecs.personal_needs.insert(entity, PersonalNeeds::default_npc());
        ecs.inventories.insert(entity, Inventory {
            items: vec![
                Item { name: format!("item_{}", i), value: 10.0 * (i as f32 + 1.0) },
            ],
        });
        ecs.sim_levels.insert(entity, SimLevel { level: SimulationLevel::L0 });
    }

    let mut persistence = ChunkPersistenceService::new(test_dir.to_str().unwrap());
    persistence.save_and_unload(coord, &mut ecs, 100);
    assert_eq!(ecs.alive.len(), 0);

    persistence.load_chunk_entities(coord, &mut ecs);
    assert_eq!(ecs.alive.len(), 5);

    for &e in &ecs.alive {
        assert!(ecs.transforms.get(&e).is_some(), "transform missing after roundtrip");
        assert!(ecs.kinds.get(&e).is_some(), "kind missing after roundtrip");
    }

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn runtime_truth_json_generation() {
    use engene::world::world::WorldGrid;
    use engene::world::heightmap::Heightmap;
    use engene::app::runtime_assembly::RuntimeAssembly;
    use engene::tools::doctor;
    use std::sync::Arc;

    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = RuntimeAssembly::vertical_slice(heightmap, &biomes);

    let json = doctor::generate_runtime_truth_json(&engine);
    assert!(json.contains("engine_version"));
    assert!(json.contains("entity_count"));
    assert!(json.contains("npc_count"));
    assert!(json.contains("economy"));
    assert!(json.contains("doctor"));
}
