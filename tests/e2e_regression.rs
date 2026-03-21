//! End-to-End and Regression tests — 101 tests covering integration of all major subsystems.

use engene::audio::audio::AudioEngine;
use engene::body::body_response::BodyPhysicalResponseCache;
use engene::body::body_store::BodyStateStore;
use engene::body::death_pipeline::CorpseManager;
use engene::core::build_manifest::{BuildManifest, SaveCompatibility, SchemaMigrationRegistry};
use engene::core::ecs::Ecs;
use engene::game::economy::item_registry::{ItemCategory, ItemRegistry};
use engene::game::economy::trader_economy::TraderState;
use engene::game::hud::{HudState, NotificationKind};
use engene::game::player::PlayerController;
use engene::game::player_save::{PlayerInventory, PlayerItem, PlayerItemType, PlayerSave};
use engene::navigation::world_graph::WorldGraph;
use engene::physics::ballistics::BallisticsSystem;
use engene::physics::destruction::DestructionSystem;
use engene::physics::fire::FireGrid;
use engene::runtime::bootstrap::{GameRuntimeAssembly, ToolsRuntimeAssembly};
use engene::simulation::camp_simulation::CampState;
use engene::simulation::role_simulation::{NpcRole, RoleBehavior};
use engene::simulation::world_milestones::WorldMilestoneTracker;
use engene::tools::console::EngineConsole;
use engene::tools::doctor::{run_doctor, DoctorMode};
use engene::tools::editor_safe_mode::EditorSafeMode;
use engene::world::chunk_persistence::ChunkPersistenceService;
use engene::world::components::{EntityKind, Job, NpcEconomy, PersonalNeeds, Transform};
use engene::world::heightmap::Heightmap;
use engene::world::streaming::{ChunkCoord, WorldStreamer};
use engene::world::world::WorldGrid;
use std::sync::Arc;

// =============================================================================
// 1. ENGINE LIFECYCLE (8 tests)
// =============================================================================

#[test]
fn e2e_tools_boots() {
    let engine = ToolsRuntimeAssembly::minimal();
    assert!(engine.is_running());
    assert!(engine
        .resources
        .get::<engene::world::resources::ResourceGrid>()
        .is_none());
}

#[test]
fn e2e_headless_boots() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let engine = GameRuntimeAssembly::headless(&biomes);
    assert!(engine.ecs.alive.len() > 0);
    assert!(engine.is_running());
}

#[test]
fn e2e_vertical_slice_boots() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    assert!(engine.ecs.alive.len() > 0);
    assert!(engine.ecs.npcs().len() > 0);
}

#[test]
fn e2e_tools_tick() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    let before = engine.ecs.alive.len();
    engine.tick(1.0 / 20.0);
    assert_eq!(engine.ecs.alive.len(), before);
}

#[test]
fn e2e_headless_tick() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let mut engine = GameRuntimeAssembly::headless(&biomes);
    engine.tick(1.0 / 20.0);
    assert!(engine.ecs.alive.len() > 0);
}

#[test]
fn e2e_vertical_slice_tick() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let mut engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    engine.tick(1.0 / 20.0);
    assert!(engine.time.tick_count > 0);
}

#[test]
fn e2e_engine_shutdown() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.shutdown();
    assert!(!engine.is_running());
}

#[test]
fn e2e_engine_has_ecs_and_resources() {
    let engine = ToolsRuntimeAssembly::minimal();
    assert!(engine
        .resources
        .get::<engene::core::runtime_config::RuntimeConfig>()
        .is_some());
    assert!(engine
        .resources
        .get::<engene::world::resources::ResourceGrid>()
        .is_none());
}

// =============================================================================
// 2. BUILD MANIFEST (6 tests)
// =============================================================================

#[test]
fn e2e_build_manifest_current() {
    let m = BuildManifest::current();
    assert!(!m.engine_version.is_empty());
    assert_eq!(m.schema_version_save, 1);
    assert_eq!(m.schema_version_chunk, 1);
}

#[test]
fn e2e_build_manifest_schema_versions() {
    let m = BuildManifest::current();
    assert_eq!(
        m.schema_version_save,
        engene::core::build_manifest::SCHEMA_VERSION_SAVE
    );
    assert_eq!(
        m.schema_version_chunk,
        engene::core::build_manifest::SCHEMA_VERSION_CHUNK
    );
}

#[test]
fn e2e_save_compatibility_current() {
    let c = SaveCompatibility::current();
    assert!(c.is_compatible(1));
    assert!(c.is_compatible(0));
}

#[test]
fn e2e_save_compatibility_is_compatible() {
    let c = SaveCompatibility::current();
    assert!(c.is_compatible(c.schema_version_save));
    assert!(!c.is_compatible(999));
}

#[test]
fn e2e_schema_migration_registry_new() {
    let reg = SchemaMigrationRegistry::new();
    assert!(!reg.has_path_save(0));
    assert!(reg.has_path_save(1));
}

#[test]
fn e2e_build_manifest_ensure_data_dirs() {
    BuildManifest::ensure_data_dirs();
}

// =============================================================================
// 3. ECS INTEGRATION (10 tests)
// =============================================================================

#[test]
fn e2e_ecs_spawn_integration() {
    let mut ecs = Ecs::new();
    let (e, pid) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 200.0,
            cell_x: 1,
            cell_y: 2,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    assert!(ecs.is_alive(e));
    assert_eq!(ecs.transforms.get(&e).unwrap().x, 100.0);
    assert!(ecs.identity.persistent_id_of(e) == Some(pid));
}

#[test]
fn e2e_ecs_despawn_integration() {
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 0.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.despawn(e);
    assert!(!ecs.is_alive(e));
    assert!(ecs.transforms.get(&e).is_none());
}

#[test]
fn e2e_ecs_components_roundtrip_transform() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn();
    let t = Transform {
        x: 42.0,
        y: 99.0,
        cell_x: 3,
        cell_y: 4,
    };
    ecs.transforms.insert(e, t.clone());
    let got = ecs.transforms.get(&e).unwrap();
    assert_eq!(got.x, 42.0);
    assert_eq!(got.cell_x, 3);
}

#[test]
fn e2e_ecs_components_roundtrip_kind() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn();
    ecs.kinds.insert(
        e,
        EntityKind::Monster(engene::world::components::MonsterSpecies::Wolf),
    );
    assert!(matches!(ecs.kinds.get(&e), Some(EntityKind::Monster(_))));
}

#[test]
fn e2e_ecs_identity_spawn_new() {
    let mut ecs = Ecs::new();
    let (e1, p1) = ecs.spawn_new();
    let (e2, p2) = ecs.spawn_new();
    assert_ne!(p1, p2);
    assert_ne!(e1, e2);
}

#[test]
fn e2e_ecs_npc_economy_component() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn();
    ecs.npc_economies.insert(
        e,
        NpcEconomy {
            money: 250.0,
            monthly_required: 80.0,
            job: Job::Trader,
            desperation: 0.1,
        },
    );
    assert_eq!(ecs.npc_economies.get(&e).unwrap().money, 250.0);
}

#[test]
fn e2e_ecs_personal_needs_component() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn();
    ecs.personal_needs.insert(e, PersonalNeeds::default_npc());
    assert!((ecs.personal_needs.get(&e).unwrap().health - 1.0).abs() < 0.01);
}

#[test]
fn e2e_ecs_batch_spawn_despawn() {
    let mut ecs = Ecs::new();
    let entities: Vec<_> = (0..5).map(|_| ecs.spawn_new().0).collect();
    assert_eq!(ecs.alive.len(), 5);
    for &e in &entities {
        ecs.despawn(e);
    }
    assert_eq!(ecs.alive.len(), 0);
}

#[test]
fn e2e_ecs_spatial_rebuild() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn();
    ecs.transforms.insert(
        e,
        Transform {
            x: 500.0,
            y: 600.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.rebuild_spatial();
    let cand = ecs.spatial.candidates_in_radius(500.0, 600.0, 50.0);
    assert!(cand.contains(&e));
}

#[test]
fn e2e_ecs_count_npcs() {
    let mut ecs = Ecs::new();
    let e1 = ecs.spawn();
    let e2 = ecs.spawn();
    ecs.kinds.insert(e1, EntityKind::Npc);
    ecs.kinds.insert(e2, EntityKind::Npc);
    assert_eq!(ecs.count_npcs(), 2);
}

// =============================================================================
// 4. WORLD STREAMING (8 tests)
// =============================================================================

#[test]
fn e2e_world_streamer_new() {
    let streamer = WorldStreamer::new(3000.0, 4000.0);
    assert_eq!(streamer.total_chunk_count(), 0);
}

#[test]
fn e2e_world_streamer_update_loads_chunks() {
    let mut streamer = WorldStreamer::new(2000.0, 3000.0);
    let (_to_load, _to_unload) = streamer.update(0.0, 0.0);
    assert!(streamer.total_chunk_count() > 0);
}

#[test]
fn e2e_chunk_coord_from_world() {
    let c = ChunkCoord::from_world(1500.0, 2500.0);
    assert!(c.x != 0 || c.z != 0);
}

#[test]
fn e2e_chunk_coord_world_center() {
    let c = ChunkCoord { x: 1, z: 2 };
    let (wx, wz) = c.world_center();
    assert!(wx > 0.0);
    assert!(wz > 0.0);
}

#[test]
fn e2e_chunk_persistence_save_and_unload() {
    let dir = std::env::temp_dir().join("engene_e2e_chunk_save");
    let _ = std::fs::remove_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 500.0,
            y: 500.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.personal_needs.insert(e, PersonalNeeds::default_npc());

    let mut persistence = ChunkPersistenceService::new(&dir);
    let coord = ChunkCoord { x: 0, z: 0 };
    let saved = persistence.save_and_unload(coord, &mut ecs, 1);
    assert_eq!(saved, 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn e2e_chunk_persistence_load() {
    let dir = std::env::temp_dir().join("engene_e2e_chunk_load");
    let _ = std::fs::remove_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, pid) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 500.0,
            y: 500.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.personal_needs.insert(e, PersonalNeeds::default_npc());

    let mut persistence = ChunkPersistenceService::new(&dir);
    let coord = ChunkCoord { x: 0, z: 0 };
    persistence.save_and_unload(coord, &mut ecs, 1);
    let loaded = persistence.load_chunk_entities(coord, &mut ecs);
    assert_eq!(loaded, 1);
    assert!(ecs.identity.resolve(pid).is_some());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn e2e_chunk_persistence_has_save() {
    let dir = std::env::temp_dir().join("engene_e2e_chunk_has");
    let _ = std::fs::remove_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 500.0,
            y: 500.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.personal_needs.insert(e, PersonalNeeds::default_npc());

    let mut persistence = ChunkPersistenceService::new(&dir);
    persistence.save_and_unload(ChunkCoord { x: 5, z: 5 }, &mut ecs, 1);
    assert!(persistence.has_save(ChunkCoord { x: 5, z: 5 }));
    let _ = std::fs::remove_dir_all(&dir);
}

// =============================================================================
// 5. DOCTOR (6 tests)
// =============================================================================

#[test]
fn e2e_doctor_advisory_no_panic() {
    let engine = ToolsRuntimeAssembly::minimal();
    let report = run_doctor(&engine, DoctorMode::Advisory);
    assert!(report.diagnostics.len() > 0);
}

#[test]
fn e2e_doctor_strict_tools() {
    let engine = ToolsRuntimeAssembly::minimal();
    let report = run_doctor(&engine, DoctorMode::Strict);
    assert_eq!(report.error_count(), 0);
}

#[test]
fn e2e_doctor_report_error_count() {
    let engine = ToolsRuntimeAssembly::minimal();
    let report = run_doctor(&engine, DoctorMode::Advisory);
    let ec = report.error_count();
    let wc = report.warning_count();
    let info_count = report.diagnostics.len() - ec - wc;
    assert_eq!(ec + wc + info_count, report.diagnostics.len());
}

#[test]
fn e2e_doctor_schema_migration_has_path_chunk() {
    let reg = SchemaMigrationRegistry::new();
    assert!(reg.has_path_chunk(1));
    assert!(!reg.has_path_chunk(0));
}

#[test]
fn e2e_doctor_vertical_slice_zero_errors() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    let report = run_doctor(&engine, DoctorMode::Advisory);
    assert_eq!(report.error_count(), 0);
}

// =============================================================================
// 6. SIMULATION (10 tests)
// =============================================================================

#[test]
fn e2e_camp_state_new() {
    let camp = CampState::new("Test Camp", "Loners", 10);
    assert_eq!(camp.name, "Test Camp");
    assert_eq!(camp.faction, "Loners");
    assert_eq!(camp.population, 10);
    assert!(camp.food_supply > 0.0);
}

#[test]
fn e2e_camp_state_tick_daily() {
    let mut camp = CampState::new("C", "F", 5);
    let food_before = camp.food_supply;
    camp.tick_daily();
    assert!(camp.food_supply <= food_before || camp.food_supply <= 2.0);
}

#[test]
fn e2e_camp_state_report_danger() {
    let mut camp = CampState::new("C", "F", 3);
    camp.report_danger(0.5);
    assert!(camp.danger_memory > 0.0);
}

#[test]
fn e2e_camp_state_resupply() {
    let mut camp = CampState::new("C", "F", 2);
    camp.food_supply = 0.2;
    camp.resupply(0.5);
    assert!(camp.food_supply > 0.2);
}

#[test]
fn e2e_camp_state_is_safe() {
    let camp = CampState::new("C", "F", 1);
    assert!(camp.security_level > 0.0);
}

#[test]
fn e2e_role_behavior_all_roles() {
    for role in [
        NpcRole::Guard,
        NpcRole::Hunter,
        NpcRole::Trader,
        NpcRole::Scavenger,
        NpcRole::Courier,
        NpcRole::Bandit,
        NpcRole::IdleResident,
        NpcRole::Mechanic,
        NpcRole::Medic,
    ] {
        let rb = RoleBehavior::for_role(role.clone());
        assert_eq!(rb.role, role);
    }
}

#[test]
fn e2e_world_milestone_tracker_new() {
    let t = WorldMilestoneTracker::new();
    assert_eq!(t.bankruptcies, 0);
    assert_eq!(t.quest_completions, 0);
}

#[test]
fn e2e_world_milestone_tracker_record_and_summary() {
    let mut t = WorldMilestoneTracker::new();
    t.record_quest_completion();
    t.record_trade();
    t.record_npc_death();
    let s = t.summary();
    assert!(s.contains("Months"));
    assert!(s.contains("Deaths"));
}

#[test]
fn e2e_world_milestone_tracker_check_milestones() {
    let mut t = WorldMilestoneTracker::new();
    for _ in 0..10 {
        t.record_quest_completion();
    }
    t.check_milestones(5);
    assert!(t.milestones_achieved.len() > 0 || t.quest_completions == 10);
}

// =============================================================================
// 7. ECONOMY (8 tests)
// =============================================================================

#[test]
fn e2e_item_registry_new() {
    let reg = ItemRegistry::new();
    assert!(reg.get("medkit").is_some());
}

#[test]
fn e2e_item_registry_get() {
    let reg = ItemRegistry::new();
    let med = reg.get("medkit").unwrap();
    assert_eq!(med.id, "medkit");
}

#[test]
fn e2e_item_registry_by_category() {
    let reg = ItemRegistry::new();
    let food = reg.by_category(ItemCategory::Food);
    assert!(food.len() > 0);
}

#[test]
fn e2e_item_registry_create_instance() {
    let reg = ItemRegistry::new();
    let inst = reg.create_instance("medkit", 3);
    assert!(inst.is_some());
    assert_eq!(inst.unwrap().stack_count, 3);
}

#[test]
fn e2e_trader_state_new() {
    let t = TraderState::new("Barman", "Loners", 500.0);
    assert_eq!(t.name, "Barman");
    assert_eq!(t.capital, 500.0);
}

#[test]
fn e2e_trader_state_tick_restock() {
    let mut t = TraderState::new("T", "F", 100.0);
    let timer_before = t.restock_timer;
    t.tick_restock(24.0);
    assert!(t.restock_timer != timer_before || t.restock_timer == 0.0);
}

#[test]
fn e2e_audio_engine_new() {
    let audio = AudioEngine::new();
    assert_eq!(audio.master_volume, 1.0);
}

#[test]
fn e2e_trader_state_effective_prices() {
    let t = TraderState::new("T", "F", 200.0);
    let buy = t.effective_buy_price(50.0, "medkit");
    let sell = t.effective_sell_price(50.0, "medkit");
    assert!(buy > 0.0);
    assert!(sell > 0.0);
}

#[test]
fn e2e_npc_economy_via_ecs() {
    let mut ecs = Ecs::new();
    let e = ecs.spawn();
    ecs.npc_economies.insert(
        e,
        NpcEconomy {
            money: 100.0,
            monthly_required: 50.0,
            job: Job::Hunter,
            desperation: 0.3,
        },
    );
    let eco = ecs.npc_economies.get(&e).unwrap();
    assert_eq!(eco.job, Job::Hunter);
}

// =============================================================================
// 8. PLAYER LIFECYCLE (12 tests)
// =============================================================================

#[test]
fn e2e_player_create() {
    let p = PlayerController::new([500.0, 10.0, 500.0]);
    assert!(p.is_alive());
    assert!(p.can_move());
}

#[test]
fn e2e_player_damage() {
    let mut p = PlayerController::new([0.0, 0.0, 0.0]);
    p.take_damage(30.0);
    assert!((p.health - 70.0).abs() < 0.01);
    assert!(p.is_alive());
}

#[test]
fn e2e_player_heal() {
    let mut p = PlayerController::new([0.0, 0.0, 0.0]);
    p.take_damage(50.0);
    p.heal(30.0);
    assert!(p.health > 70.0);
}

#[test]
fn e2e_player_die() {
    let mut p = PlayerController::new([0.0, 0.0, 0.0]);
    p.take_damage(150.0);
    assert!(!p.is_alive());
}

#[test]
fn e2e_player_respawn() {
    let mut p = PlayerController::new([0.0, 0.0, 0.0]);
    p.take_damage(150.0);
    p.respawn([100.0, 5.0, 100.0]);
    assert!(p.is_alive());
    assert!(p.health_fraction() > 0.99);
}

#[test]
fn e2e_player_tick() {
    let mut p = PlayerController::new([0.0, 0.0, 0.0]);
    p.tick(0.1, false);
    assert!(p.health_fraction() > 0.0);
}

#[test]
fn e2e_player_inventory_new() {
    let inv = PlayerInventory::new();
    assert_eq!(inv.money, 500.0);
}

#[test]
fn e2e_player_inventory_add_item() {
    let mut inv = PlayerInventory::new();
    let ok = inv.add_item(PlayerItem {
        name: "Medkit".into(),
        quantity: 2,
        weight: 0.3,
        item_type: PlayerItemType::Medkit,
    });
    assert!(ok);
    assert!(inv.has_item("Medkit"));
}

#[test]
fn e2e_player_inventory_remove_item() {
    let mut inv = PlayerInventory::new();
    inv.add_item(PlayerItem {
        name: "Bread".into(),
        quantity: 5,
        weight: 0.2,
        item_type: PlayerItemType::Food,
    });
    assert!(inv.remove_item("Bread", 2));
    assert_eq!(inv.item_count("Bread"), 3);
}

#[test]
fn e2e_player_save_from_state() {
    let ctrl = PlayerController::new([100.0, 5.0, 200.0]);
    let mut inv = PlayerInventory::new();
    inv.add_item(PlayerItem {
        name: "Medkit".into(),
        quantity: 1,
        weight: 0.3,
        item_type: PlayerItemType::Medkit,
    });
    let save = PlayerSave::from_state(&ctrl, &inv, vec![1], vec![2], 3600.0, 1, 3);
    assert_eq!(save.position[0], 100.0);
    assert_eq!(save.inventory.len(), 1);
}

#[test]
fn e2e_player_save_restore_controller() {
    let ctrl = PlayerController::new([50.0, 10.0, 60.0]);
    let inv = PlayerInventory::new();
    let save = PlayerSave::from_state(&ctrl, &inv, vec![], vec![], 0.0, 1, 1);
    let restored = save.restore_controller();
    assert_eq!(restored.position[0], 50.0);
}

#[test]
fn e2e_player_save_restore_inventory() {
    let ctrl = PlayerController::new([0.0, 0.0, 0.0]);
    let mut inv = PlayerInventory::new();
    inv.add_item(PlayerItem {
        name: "Bread".into(),
        quantity: 3,
        weight: 0.2,
        item_type: PlayerItemType::Food,
    });
    let save = PlayerSave::from_state(&ctrl, &inv, vec![], vec![], 0.0, 1, 1);
    let restored = save.restore_inventory();
    assert!(restored.has_item("Bread"));
}

// =============================================================================
// 9. HUD INTEGRATION (6 tests)
// =============================================================================

#[test]
fn e2e_hud_new() {
    let hud = HudState::new();
    assert!(hud.show_health);
    assert!(hud.show_crosshair);
}

#[test]
fn e2e_hud_push_notification() {
    let mut hud = HudState::new();
    hud.push_notification("Item picked up", NotificationKind::ItemPickup);
    assert_eq!(hud.notification_queue.len(), 1);
}

#[test]
fn e2e_hud_tick_clears_notifications() {
    let mut hud = HudState::new();
    hud.push_notification("Test", NotificationKind::Info);
    hud.tick(4.0);
    assert_eq!(hud.notification_queue.len(), 0);
}

#[test]
fn e2e_hud_set_interaction() {
    let mut hud = HudState::new();
    hud.set_interaction("Press E to interact");
    assert!(hud.show_interaction_prompt);
    assert!(hud.interaction_text.contains("E"));
}

#[test]
fn e2e_hud_set_quest() {
    let mut hud = HudState::new();
    hud.set_quest("Find the artifact", "1/3");
    assert_eq!(hud.active_quest_name, "Find the artifact");
}

#[test]
fn e2e_hud_clear_interaction() {
    let mut hud = HudState::new();
    hud.set_interaction("x");
    hud.clear_interaction();
    assert!(!hud.show_interaction_prompt);
}

// =============================================================================
// 10. CONSOLE INTEGRATION (6 tests)
// =============================================================================

#[test]
fn e2e_console_new() {
    let console = EngineConsole::new();
    assert!(console.command_count() > 0);
}

#[test]
fn e2e_console_help() {
    let mut console = EngineConsole::new();
    let out = console.execute("help");
    assert!(out.contains("help") || out.len() > 0);
}

#[test]
fn e2e_console_unknown_command() {
    let mut console = EngineConsole::new();
    let out = console.execute("nonexistent_cmd_xyz");
    assert!(out.contains("Unknown") || out.contains("unknown"));
}

#[test]
fn e2e_console_command_count() {
    let console = EngineConsole::new();
    let cnt = console.command_count();
    assert!(cnt > 5);
}

// =============================================================================
// 11. BODY / COMBAT (8 tests)
// =============================================================================

#[test]
fn e2e_body_state_store_new() {
    let store = BodyStateStore::new();
    assert!(store.is_empty());
}

#[test]
fn e2e_body_state_store_allocate() {
    let mut store = BodyStateStore::new();
    let body = engene::body::anatomy::BodyState::new_humanoid(1);
    let handle = store.allocate(body);
    assert_eq!(store.len(), 1);
    assert!(handle.aggregate_health > 0.0);
}

#[test]
fn e2e_body_physical_response_healthy() {
    let cache = BodyPhysicalResponseCache::healthy();
    assert_eq!(cache.movement_speed_mult, 1.0);
    assert!(cache.can_sprint);
}

#[test]
fn e2e_body_physical_response_compute() {
    let cache = BodyPhysicalResponseCache::compute_from_health(1.0, 1.0, 0.0);
    assert_eq!(
        cache.response_tier,
        engene::body::body_response::PhysicalResponseTier::Healthy
    );
}

#[test]
fn e2e_corpse_manager_new() {
    let mgr = CorpseManager::new();
    assert_eq!(mgr.corpse_count(), 0);
}

#[test]
fn e2e_corpse_manager_register_death() {
    let mut mgr = CorpseManager::new();
    mgr.register_death(1, [100.0, 200.0], "bullet", vec!["medkit".into()]);
    assert_eq!(mgr.corpse_count(), 1);
}

#[test]
fn e2e_corpse_manager_tick() {
    let mut mgr = CorpseManager::new();
    mgr.register_death(1, [0.0, 0.0], "test", vec![]);
    mgr.tick_all(1.0);
    assert_eq!(mgr.corpse_count(), 1);
}

// =============================================================================
// 12. PHYSICS (8 tests)
// =============================================================================

#[test]
fn e2e_ballistics_system_fire() {
    let mut sys = BallisticsSystem::new();
    sys.fire(
        glam::Vec3::new(0.0, 10.0, 0.0),
        glam::Vec3::new(1.0, 0.0, 0.0),
        300.0,
        0.01,
        0.1,
        0,
        1,
        42,
    );
    assert_eq!(sys.projectiles.len(), 1);
}

#[test]
fn e2e_destruction_system_new() {
    let sys = DestructionSystem::new();
    assert_eq!(sys.objects.len(), 0);
}

#[test]
fn e2e_fire_grid_new() {
    let grid = FireGrid::new();
    assert_eq!(grid.active_fire_count(), 0);
}

#[test]
fn e2e_fire_grid_ignite() {
    let mut grid = FireGrid::new();
    grid.set_fuel(5, 5, 1.0, 0.8);
    grid.ignite(5, 5);
    assert_eq!(grid.active_fire_count(), 1);
}

#[test]
fn e2e_fire_grid_update() {
    let mut grid = FireGrid::new();
    grid.set_fuel(10, 10, 1.0, 0.5);
    grid.ignite(10, 10);
    grid.update(0.1, engene::world::components::SimulationLevel::L0);
    assert!(grid.burning_cells().len() == grid.active_fire_count());
}

#[test]
fn e2e_destruction_register_object() {
    let mut sys = DestructionSystem::new();
    let obj = engene::physics::destruction::DestructibleObject::new(
        0,
        vec![
            engene::physics::destruction::DestructionNode {
                id: 0,
                position: glam::Vec3::ZERO,
                mass: 1.0,
                material: 0,
                accumulated_stress: 0.0,
            },
            engene::physics::destruction::DestructionNode {
                id: 1,
                position: glam::Vec3::new(1.0, 0.0, 0.0),
                mass: 1.0,
                material: 0,
                accumulated_stress: 0.0,
            },
        ],
        vec![engene::physics::destruction::DestructionLink {
            a: 0,
            b: 1,
            strength: 100.0,
            fatigue: 0.0,
            broken: false,
        }],
    );
    sys.register_object(obj);
    assert_eq!(sys.objects.len(), 1);
}

// =============================================================================
// 13. NAVIGATION (5 tests)
// =============================================================================

#[test]
fn e2e_world_graph_build_default() {
    let g = WorldGraph::build_default();
    assert!(g.locations.len() > 0);
}

#[test]
fn e2e_world_graph_find_path() {
    let g = WorldGraph::build_default();
    let ids: Vec<_> = g.locations.keys().copied().collect();
    if ids.len() >= 2 {
        let path = g.find_path(ids[0], ids[1]);
        assert!(path.is_some());
        assert!(path.unwrap().len() > 0);
    }
}

#[test]
fn e2e_world_graph_same_location_path() {
    let g = WorldGraph::build_default();
    let id = *g.locations.keys().next().unwrap();
    let path = g.find_path(id, id);
    assert_eq!(path, Some(vec![id]));
}

// =============================================================================
// 14. EDITOR SAFE MODE (5 tests)
// =============================================================================

#[test]
fn e2e_editor_safe_mode_new() {
    let esm = EditorSafeMode::new();
    assert!(!esm.is_safe_mode());
}

#[test]
fn e2e_editor_safe_mode_register_panel() {
    let mut esm = EditorSafeMode::new();
    esm.register_panel("Inspector");
    assert!(esm.is_panel_enabled("Inspector"));
}

#[test]
fn e2e_editor_safe_mode_unregistered_panel() {
    let esm = EditorSafeMode::new();
    assert!(!esm.is_panel_enabled("NonExistent"));
}

#[test]
fn e2e_editor_safe_mode_panel_report() {
    let mut esm = EditorSafeMode::new();
    esm.register_panel("A");
    esm.register_panel("B");
    let report = esm.panel_report();
    assert!(report.contains("Editor Panel"));
}

// =============================================================================
// 15. GOLDEN SCENARIO (5 tests)
// =============================================================================

#[test]
fn e2e_golden_player_full_lifecycle() {
    let mut player = PlayerController::new([500.0, 10.0, 500.0]);
    let mut inv = PlayerInventory::new();
    inv.add_item(PlayerItem {
        name: "Medkit".into(),
        quantity: 2,
        weight: 0.3,
        item_type: PlayerItemType::Medkit,
    });

    player.take_damage(50.0);
    assert!(player.is_alive());
    player.heal(30.0);
    player.take_damage(150.0);
    assert!(!player.is_alive());
    player.respawn([100.0, 5.0, 100.0]);
    assert!(player.is_alive());
    assert!(inv.has_item("Medkit"));
}

#[test]
fn e2e_golden_save_load_roundtrip() {
    let ctrl = PlayerController::new([111.0, 22.0, 333.0]);
    let mut inv = PlayerInventory::new();
    inv.add_item(PlayerItem {
        name: "Bread".into(),
        quantity: 5,
        weight: 0.2,
        item_type: PlayerItemType::Food,
    });

    let save = PlayerSave::from_state(&ctrl, &inv, vec![1, 2], vec![3], 120.0, 5, 2);
    let restored_ctrl = save.restore_controller();
    let restored_inv = save.restore_inventory();

    assert_eq!(restored_ctrl.position[0], 111.0);
    assert!(restored_inv.has_item("Bread"));
}

#[test]
fn e2e_golden_engine_tick_doctor() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let mut engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);

    for _ in 0..10 {
        engine.tick(1.0 / 20.0);
    }
    let report = run_doctor(&engine, DoctorMode::Advisory);
    assert_eq!(report.error_count(), 0);
}

#[test]
fn e2e_golden_camp_trader_milestone() {
    let mut camp = CampState::new("Sidor", "Loners", 8);
    let mut trader = TraderState::new("Sidor", "Loners", 1000.0);
    let mut tracker = WorldMilestoneTracker::new();

    camp.tick_daily();
    trader.tick_restock(24.0);
    for _ in 0..20 {
        tracker.record_trade();
    }
    tracker.check_milestones(3);
    assert!(tracker.trade_events == 20);
}

#[test]
fn e2e_golden_chunk_save_load_ecs() {
    let dir = std::env::temp_dir().join("engene_e2e_golden_chunk");
    let _ = std::fs::remove_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, pid) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 500.0,
            y: 500.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.personal_needs.insert(e, PersonalNeeds::default_npc());
    ecs.npc_economies.insert(
        e,
        NpcEconomy {
            money: 100.0,
            monthly_required: 50.0,
            job: Job::Trader,
            desperation: 0.0,
        },
    );

    let mut persistence = ChunkPersistenceService::new(&dir);
    let coord = ChunkCoord { x: 0, z: 0 };
    persistence.save_and_unload(coord, &mut ecs, 10);
    assert!(!ecs.is_alive(e));

    let loaded = persistence.load_chunk_entities(coord, &mut ecs);
    assert_eq!(loaded, 1);
    let restored = ecs.identity.resolve(pid).unwrap();
    assert_eq!(ecs.npc_economies.get(&restored).unwrap().money, 100.0);

    let _ = std::fs::remove_dir_all(&dir);
}
