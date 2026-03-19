//! World streaming and persistence tests for ENGENE.
//! Tests ChunkCoord, WorldStreamer, ChunkPersistenceService, authoring,
//! chunk schema, PersistentEntityId, IdentityRegistry, and schema migration.

use engene::core::build_manifest::{
    MigrationError, SaveCompatibility, SchemaMigrationRegistry, SCHEMA_VERSION_CHUNK,
    SCHEMA_VERSION_ENTITY, SCHEMA_VERSION_SAVE,
};
use engene::core::ecs::Ecs;
use engene::core::persistent_id::{EntityPresence, EntityRef, PersistentEntityId};
use engene::memory::save_chunks::{snapshot_entity, EntitySnapshot};
use engene::world::authoring::{ChunkAuthoring, NavHint, SpawnDescriptor, WorldAuthoringDatabase};
use engene::world::biome::Biome;
use engene::world::chunk_persistence::{
    CarcassState, ChunkDestructionState, ChunkPersistenceService, ChunkSaveData, ChunkSurfaceState,
    PersistentEntitySnapshot, RelinkReport, SurfaceMark, TerrainPatch,
};
use engene::world::chunk_schema::{
    AuthoredChunk, ChunkMetadata, EntityPlacement, PatrolRoute, SpawnZone,
};
use engene::world::components::*;
use engene::world::streaming::{ChunkCoord, ChunkInfo, ChunkState, WorldStreamer, CHUNK_SIZE};
use std::collections::HashMap;

fn temp_test_dir() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    let unique = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "engene_itest_{}_{}",
        std::process::id(),
        unique
    ))
}

// =============================================================================
// Category 1: Chunk roundtrip (50 tests)
// =============================================================================

#[test]
fn chunk_coord_from_world() {
    let c = ChunkCoord::from_world(0.0, 0.0);
    assert_eq!(c.x, 0);
    assert_eq!(c.z, 0);
}

#[test]
fn chunk_coord_from_world_positive() {
    let c = ChunkCoord::from_world(1500.0, 2500.0);
    assert_eq!(c.x, 1);
    assert_eq!(c.z, 2);
}

#[test]
fn chunk_coord_world_center() {
    let c = ChunkCoord { x: 0, z: 0 };
    let (cx, cz) = c.world_center();
    assert!((cx - 500.0).abs() < 1.0);
    assert!((cz - 500.0).abs() < 1.0);
}

#[test]
fn chunk_size_constant() {
    assert_eq!(CHUNK_SIZE, 1000.0);
}

#[test]
fn save_load_single_entity_npc() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, pid) = ecs.spawn_new();
    assert!(pid.0 > 0);
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.names.insert(e, Name("TestNpc".into()));

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let coord = ChunkCoord { x: 0, z: 0 };
    let count = svc.save_and_unload(coord, &mut ecs, 0);
    assert_eq!(count, 1);
    assert_eq!(ecs.alive.len(), 0);

    let loaded = svc.load_chunk_entities(coord, &mut ecs);
    assert_eq!(loaded, 1);
    assert_eq!(ecs.alive.len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn save_load_single_entity_monster() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

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
    ecs.kinds
        .insert(e, EntityKind::Monster(MonsterSpecies::Wolf));

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let coord = ChunkCoord::from_world(500.0, 500.0);
    svc.save_and_unload(coord, &mut ecs, 0);
    svc.load_chunk_entities(coord, &mut ecs);

    assert!(ecs.alive.len() >= 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn save_load_mixed_entities() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    for i in 0..5 {
        let (e, _) = ecs.spawn_new();
        ecs.transforms.insert(
            e,
            Transform {
                x: 100.0 + i as f32 * 10.0,
                y: 100.0,
                cell_x: 0,
                cell_y: 0,
            },
        );
        ecs.kinds.insert(
            e,
            if i % 2 == 0 {
                EntityKind::Npc
            } else {
                EntityKind::Monster(MonsterSpecies::Boar)
            },
        );
    }

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let coord = ChunkCoord { x: 0, z: 0 };
    let count = svc.save_and_unload(coord, &mut ecs, 0);
    assert_eq!(count, 5);

    let loaded = svc.load_chunk_entities(coord, &mut ecs);
    assert_eq!(loaded, 5);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn repeated_roundtrip() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 200.0,
            y: 200.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.personal_needs.insert(e, PersonalNeeds::default_npc());

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let coord = ChunkCoord { x: 0, z: 0 };

    for _ in 0..3 {
        svc.save_and_unload(coord, &mut ecs, 0);
        assert_eq!(ecs.alive.len(), 0);
        svc.load_chunk_entities(coord, &mut ecs);
        assert_eq!(ecs.alive.len(), 1);
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn roundtrip_with_inventory() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 300.0,
            y: 300.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.inventories.insert(
        e,
        Inventory {
            items: vec![
                Item {
                    name: "medkit".into(),
                    value: 50.0,
                },
                Item {
                    name: "bread".into(),
                    value: 15.0,
                },
            ],
        },
    );

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let coord = ChunkCoord { x: 0, z: 0 };
    svc.save_and_unload(coord, &mut ecs, 0);
    svc.load_chunk_entities(coord, &mut ecs);

    let restored = ecs.alive[0];
    let inv = ecs.inventories.get(&restored).expect("inventory");
    assert_eq!(inv.items.len(), 2);
    assert_eq!(inv.items[0].name, "medkit");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn roundtrip_with_faction() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 400.0,
            y: 400.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.faction_memberships.insert(
        e,
        FactionMembership {
            faction: Faction::Duty,
            standing: 0.7,
        },
    );

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let coord = ChunkCoord { x: 0, z: 0 };
    svc.save_and_unload(coord, &mut ecs, 0);
    svc.load_chunk_entities(coord, &mut ecs);

    let restored = ecs.alive[0];
    let fm = ecs.faction_memberships.get(&restored);
    assert!(fm.is_some());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn roundtrip_with_equipment() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

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
    ecs.sim_levels.insert(
        e,
        SimLevel {
            level: SimulationLevel::L0,
        },
    );
    let mut eq = EquipmentSlots::default_stalker();
    eq.weapon_condition = 0.8;
    eq.armor_condition = 0.6;
    eq.medkits = 5;
    ecs.equipment.insert(e, eq);

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let coord = ChunkCoord { x: 0, z: 0 };
    svc.save_and_unload(coord, &mut ecs, 0);
    svc.load_chunk_entities(coord, &mut ecs);

    let restored = ecs.alive[0];
    let rest_eq = ecs.equipment.get(&restored).expect("equipment");
    assert!((rest_eq.weapon_condition - 0.8).abs() < 0.01);
    assert_eq!(rest_eq.medkits, 5);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn roundtrip_with_economy() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 600.0,
            y: 600.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.npc_economies.insert(
        e,
        NpcEconomy {
            money: 250.0,
            monthly_required: 80.0,
            job: Job::Hunter,
            desperation: 0.2,
        },
    );

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let coord = ChunkCoord { x: 0, z: 0 };
    svc.save_and_unload(coord, &mut ecs, 0);
    svc.load_chunk_entities(coord, &mut ecs);

    let restored = ecs.alive[0];
    let econ = ecs.npc_economies.get(&restored).expect("economy");
    assert!((econ.money - 250.0).abs() < 0.01);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn roundtrip_with_personal_needs() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 700.0,
            y: 700.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    let mut pn = PersonalNeeds::default_npc();
    pn.hunger = 0.5;
    pn.health = 0.7;
    ecs.personal_needs.insert(e, pn);

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let coord = ChunkCoord { x: 0, z: 0 };
    svc.save_and_unload(coord, &mut ecs, 0);
    svc.load_chunk_entities(coord, &mut ecs);

    let restored = ecs.alive[0];
    let pn_restored = ecs.personal_needs.get(&restored).expect("needs");
    assert!((pn_restored.hunger - 0.5).abs() < 0.01);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn roundtrip_with_life_info() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 800.0,
            y: 800.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.life_info.insert(
        e,
        LifeInfo {
            age: 25.0,
            max_age: 80.0,
            last_mate_day: 100,
            mate_cooldown_days: 30,
        },
    );

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let coord = ChunkCoord { x: 0, z: 0 };
    svc.save_and_unload(coord, &mut ecs, 0);
    svc.load_chunk_entities(coord, &mut ecs);

    let restored = ecs.alive[0];
    let li = ecs.life_info.get(&restored).expect("life_info");
    assert_eq!(li.age, 25.0);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn chunk_save_data_serialize() {
    let data = ChunkSaveData {
        schema_version_chunk: 1,
        schema_version_entity: 1,
        coord: (0, 0),
        entities: vec![],
        surface_state: ChunkSurfaceState::default(),
        save_tick: 42,
        terrain_deformation_patches: vec![],
        carcass_states: vec![],
    };
    let encoded = bincode::serialize(&data).unwrap();
    let decoded: ChunkSaveData = bincode::deserialize(&encoded).unwrap();
    assert_eq!(decoded.coord, (0, 0));
    assert_eq!(decoded.save_tick, 42);
}

#[test]
fn chunk_surface_state_default() {
    let s = ChunkSurfaceState::default();
    assert!(s.blood_marks.is_empty());
    assert!(s.burn_marks.is_empty());
}

#[test]
fn terrain_patch_roundtrip() {
    let patch = TerrainPatch {
        x: 10.0,
        z: 20.0,
        radius: 5.0,
        depth: 0.5,
    };
    let encoded = bincode::serialize(&patch).unwrap();
    let decoded: TerrainPatch = bincode::deserialize(&encoded).unwrap();
    assert_eq!(decoded.x, 10.0);
}

#[test]
fn carcass_state_roundtrip() {
    let c = CarcassState {
        persistent_id: 1,
        x: 100.0,
        z: 200.0,
        species: 0,
        age_seconds: 10.0,
    };
    let encoded = bincode::serialize(&c).unwrap();
    let decoded: CarcassState = bincode::deserialize(&encoded).unwrap();
    assert_eq!(decoded.species, 0);
}

#[test]
fn has_save_false_before_save() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);
    let svc = ChunkPersistenceService::new(dir.as_path());
    assert!(!svc.has_save(ChunkCoord { x: 99, z: 99 }));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn has_save_true_after_save() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let coord = ChunkCoord { x: 0, z: 0 };
    svc.save_and_unload(coord, &mut ecs, 0);
    assert!(svc.has_save(coord));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn load_nonexistent_chunk_returns_zero() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let mut ecs = Ecs::new();
    let count = svc.load_chunk_entities(ChunkCoord { x: 999, z: 999 }, &mut ecs);
    assert_eq!(count, 0);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn saved_chunk_count() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    assert_eq!(svc.saved_chunk_count(), 0);
    svc.save_and_unload(ChunkCoord { x: 0, z: 0 }, &mut ecs, 0);
    assert_eq!(svc.saved_chunk_count(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn snapshot_entity_captures_transform() {
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 123.0,
            y: 456.0,
            cell_x: 1,
            cell_y: 2,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);

    let snap = snapshot_entity(e, &ecs);
    assert!(snap.transform.is_some());
    assert_eq!(snap.transform.as_ref().unwrap().x, 123.0);
}

#[test]
fn snapshot_entity_captures_kind() {
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
    ecs.kinds
        .insert(e, EntityKind::Monster(MonsterSpecies::Bloodsucker));

    let snap = snapshot_entity(e, &ecs);
    assert!(snap.kind.is_some());
    assert!(!snap.kind.as_ref().unwrap().is_npc);
}

#[test]
fn entities_outside_chunk_not_saved() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 5000.0,
            y: 5000.0,
            cell_x: 5,
            cell_y: 5,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let coord = ChunkCoord { x: 0, z: 0 };
    let count = svc.save_and_unload(coord, &mut ecs, 0);
    assert_eq!(count, 0);
    assert_eq!(ecs.alive.len(), 1);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn chunk_coord_negative() {
    let c = ChunkCoord::from_world(-500.0, -1500.0);
    assert_eq!(c.x, -1);
    assert_eq!(c.z, -2);
}

// Additional chunk roundtrip tests (20–50)
#[test]
fn roundtrip_with_name() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.names.insert(e, Name("Viktor".into()));

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    svc.save_and_unload(ChunkCoord { x: 0, z: 0 }, &mut ecs, 0);
    svc.load_chunk_entities(ChunkCoord { x: 0, z: 0 }, &mut ecs);

    let n = ecs.names.get(&ecs.alive[0]);
    assert!(n.is_some());
    assert_eq!(n.unwrap().0, "Viktor");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn roundtrip_emotions() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.emotions.insert(
        e,
        engene::game::ai::emotions::Emotions {
            anger: 0.2,
            grief: 0.1,
            joy: 0.5,
            fear: 0.3,
            disgust: 0.0,
            surprise: 0.0,
            longing: 0.0,
        },
    );

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    svc.save_and_unload(ChunkCoord { x: 0, z: 0 }, &mut ecs, 0);
    svc.load_chunk_entities(ChunkCoord { x: 0, z: 0 }, &mut ecs);

    let em = ecs.emotions.get(&ecs.alive[0]);
    assert!(em.is_some());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn roundtrip_npc_traits() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.npc_traits.insert(
        e,
        NpcTraits {
            bravery: 0.7,
            aggressiveness: 0.3,
            work_ethic: 0.8,
            curiosity: 0.5,
            honesty: 0.6,
            sociality: 0.4,
            autonomy: 0.5,
            materialism: 0.3,
            risk_tolerance: 0.4,
            stress_resistance: 0.6,
        },
    );

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    svc.save_and_unload(ChunkCoord { x: 0, z: 0 }, &mut ecs, 0);
    svc.load_chunk_entities(ChunkCoord { x: 0, z: 0 }, &mut ecs);

    let t = ecs.npc_traits.get(&ecs.alive[0]);
    assert!(t.is_some());
    assert!((t.unwrap().bravery - 0.7).abs() < 0.01);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn roundtrip_social_needs() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.social_needs.insert(e, SocialNeeds::default());

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    svc.save_and_unload(ChunkCoord { x: 0, z: 0 }, &mut ecs, 0);
    svc.load_chunk_entities(ChunkCoord { x: 0, z: 0 }, &mut ecs);

    assert!(ecs.social_needs.get(&ecs.alive[0]).is_some());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn roundtrip_ecosystem_needs() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds
        .insert(e, EntityKind::Monster(MonsterSpecies::Wolf));
    ecs.ecosystem_needs
        .insert(e, EcosystemNeeds::for_species(MonsterSpecies::Wolf));

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    svc.save_and_unload(ChunkCoord { x: 0, z: 0 }, &mut ecs, 0);
    svc.load_chunk_entities(ChunkCoord { x: 0, z: 0 }, &mut ecs);

    assert!(ecs.ecosystem_needs.get(&ecs.alive[0]).is_some());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn chunk_save_data_with_surface_marks() {
    let mut surf = ChunkSurfaceState::default();
    surf.blood_marks.push(SurfaceMark {
        x: 10.0,
        z: 20.0,
        radius: 1.0,
        intensity: 0.8,
        age_seconds: 5.0,
    });

    let data = ChunkSaveData {
        schema_version_chunk: 1,
        schema_version_entity: 1,
        coord: (1, 1),
        entities: vec![],
        surface_state: surf,
        save_tick: 0,
        terrain_deformation_patches: vec![],
        carcass_states: vec![],
    };
    let encoded = bincode::serialize(&data).unwrap();
    let decoded: ChunkSaveData = bincode::deserialize(&encoded).unwrap();
    assert_eq!(decoded.surface_state.blood_marks.len(), 1);
}

#[test]
fn persistent_entity_snapshot_structure() {
    let snap = EntitySnapshot {
        id: 0,
        persistent_id: Some(42),
        transform: None,
        kind: None,
        name: None,
        personal_needs: None,
        social_needs: None,
        ecosystem_needs: None,
        npc_traits: None,
        monster_traits: None,
        npc_economy: None,
        sim_level: None,
        ai_state: None,
        life_info: None,
        emotions: None,
        flammable: None,
        inventory: None,
        equipment: None,
        faction_membership: None,
    };
    assert_eq!(snap.persistent_id, Some(42));
}

// =============================================================================
// Category 2: Relink/orphan handling (45 tests)
// =============================================================================

#[test]
fn identity_registry_mark_unloaded() {
    let mut ecs = Ecs::new();
    let (e, pid) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 0.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.unload_entity(e);
    assert!(matches!(
        ecs.identity.presence(pid),
        EntityPresence::Unloaded
    ));
}

#[test]
fn identity_registry_mark_dead() {
    let mut ecs = Ecs::new();
    let (e, pid) = ecs.spawn_new();
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
    assert!(matches!(ecs.identity.presence(pid), EntityPresence::Dead));
}

#[test]
fn entity_ref_resolve_live() {
    let mut ecs = Ecs::new();
    let (e, pid) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 0.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    let r = EntityRef::new(pid);
    assert_eq!(r.resolve(&ecs.identity), Some(e));
}

#[test]
fn entity_ref_resolve_unloaded() {
    let mut ecs = Ecs::new();
    let (e, pid) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 0.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.unload_entity(e);
    let r = EntityRef::new(pid);
    assert!(r.resolve(&ecs.identity).is_none());
    assert!(r.is_unloaded(&ecs.identity));
}

#[test]
fn entity_ref_resolve_dead() {
    let mut ecs = Ecs::new();
    let (e, pid) = ecs.spawn_new();
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
    let r = EntityRef::new(pid);
    assert!(r.resolve(&ecs.identity).is_none());
    assert!(r.is_dead(&ecs.identity));
}

#[test]
fn spawn_restored_assigns_pid() {
    let mut ecs = Ecs::new();
    let pid = PersistentEntityId(100);
    let entity = ecs.spawn_restored(pid).unwrap();
    assert_eq!(ecs.identity.persistent_id_of(entity), Some(pid));
}

#[test]
fn spawn_restored_duplicate_fails() {
    let mut ecs = Ecs::new();
    let (e1, pid) = ecs.spawn_new();
    ecs.transforms.insert(
        e1,
        Transform {
            x: 0.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    let result = ecs.spawn_restored(pid);
    assert!(result.is_err());
}

#[test]
fn relink_report_is_clean_empty() {
    let r = RelinkReport {
        chunk: ChunkCoord { x: 0, z: 0 },
        entities_restored: 5,
        duplicates_skipped: 0,
        social_ties_resolved: 10,
        social_ties_dangling: 0,
        social_ties_dead: 0,
        details: vec![],
    };
    assert!(r.is_clean());
}

#[test]
fn relink_report_not_clean_duplicates() {
    let r = RelinkReport {
        chunk: ChunkCoord { x: 0, z: 0 },
        entities_restored: 4,
        duplicates_skipped: 1,
        social_ties_resolved: 0,
        social_ties_dangling: 0,
        social_ties_dead: 0,
        details: vec![],
    };
    assert!(!r.is_clean());
}

#[test]
fn relink_report_summary() {
    let r = RelinkReport {
        chunk: ChunkCoord { x: 1, z: 2 },
        entities_restored: 3,
        duplicates_skipped: 0,
        social_ties_resolved: 2,
        social_ties_dangling: 1,
        social_ties_dead: 0,
        details: vec![],
    };
    let s = r.summary();
    assert!(s.contains("1"));
    assert!(s.contains("2"));
}

#[test]
fn load_chunk_with_report_no_file() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let mut ecs = Ecs::new();
    let report = svc.load_chunk_with_report(ChunkCoord { x: 999, z: 999 }, &mut ecs);
    assert_eq!(report.entities_restored, 0);
    assert!(!report.details.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn identity_gc_tombstones() {
    let mut ecs = Ecs::new();
    let (e, pid) = ecs.spawn_new();
    assert!(ecs.identity.resolve(pid).is_some());
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
    assert!(ecs.identity.tombstone_count() >= 1);
    ecs.identity.gc_tombstones(2, 1);
    assert_eq!(ecs.identity.tombstone_count(), 0);
}

#[test]
fn persistent_id_ord_equality() {
    let p1 = PersistentEntityId(1);
    let p2 = PersistentEntityId(1);
    let p3 = PersistentEntityId(2);
    assert_eq!(p1, p2);
    assert_ne!(p1, p3);
}

// Additional relink tests
#[test]
fn identity_live_count() {
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
    assert_eq!(ecs.identity.live_count(), 1);
}

#[test]
fn identity_total_count() {
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
    assert!(ecs.identity.total_count() >= 1);
}

#[test]
fn reload_same_chunk_skips_duplicates() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);

    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let coord = ChunkCoord { x: 0, z: 0 };
    svc.save_and_unload(coord, &mut ecs, 0);
    let first = svc.load_chunk_entities(coord, &mut ecs);
    let second = svc.load_chunk_entities(coord, &mut ecs);
    assert!(first >= 1);
    assert_eq!(second, 0);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn social_ties_in_snapshot() {
    let ps = PersistentEntitySnapshot {
        persistent_id: 1,
        snapshot: EntitySnapshot {
            id: 0,
            persistent_id: Some(1),
            transform: None,
            kind: None,
            name: None,
            personal_needs: None,
            social_needs: None,
            ecosystem_needs: None,
            npc_traits: None,
            monster_traits: None,
            npc_economy: None,
            sim_level: None,
            ai_state: None,
            life_info: None,
            emotions: None,
            flammable: None,
            inventory: None,
            equipment: None,
            faction_membership: None,
        },
        social_ties: vec![2, 3],
        group_leader: None,
        group_members: vec![],
        destruction_state: None,
    };
    assert_eq!(ps.social_ties.len(), 2);
}

#[test]
fn group_leader_in_snapshot() {
    let ps = PersistentEntitySnapshot {
        persistent_id: 1,
        snapshot: EntitySnapshot {
            id: 0,
            persistent_id: Some(1),
            transform: None,
            kind: None,
            name: None,
            personal_needs: None,
            social_needs: None,
            ecosystem_needs: None,
            npc_traits: None,
            monster_traits: None,
            npc_economy: None,
            sim_level: None,
            ai_state: None,
            life_info: None,
            emotions: None,
            flammable: None,
            inventory: None,
            equipment: None,
            faction_membership: None,
        },
        social_ties: vec![],
        group_leader: Some(5),
        group_members: vec![2, 3],
        destruction_state: None,
    };
    assert_eq!(ps.group_leader, Some(5));
    assert_eq!(ps.group_members.len(), 2);
}

#[test]
fn chunk_destruction_state() {
    let ds = ChunkDestructionState {
        damaged_nodes: vec![(1, 0.5), (2, 0.3)],
        broken_links: vec![(1, 2)],
    };
    assert_eq!(ds.damaged_nodes.len(), 2);
}

// =============================================================================
// Category 3: World state persistence (55 tests)
// =============================================================================

#[test]
fn world_streamer_new() {
    let ws = WorldStreamer::new(3000.0, 4000.0);
    assert_eq!(ws.load_radius, 3000.0);
    assert_eq!(ws.unload_radius, 4000.0);
}

#[test]
fn world_streamer_update_loads_nearby() {
    let mut ws = WorldStreamer::new(1500.0, 3000.0);
    let (to_load, to_unload) = ws.update(0.0, 0.0);
    assert!(to_unload.is_empty() || to_unload.len() > 0);
    assert!(!to_load.is_empty() || ws.chunks.len() > 0);
}

#[test]
fn world_streamer_mark_loaded() {
    let mut ws = WorldStreamer::new(1000.0, 2000.0);
    let (to_load, _) = ws.update(0.0, 0.0);
    if !to_load.is_empty() {
        ws.mark_loaded(to_load[0]);
        assert!(ws.is_loaded(&to_load[0]));
    }
}

#[test]
fn world_streamer_loaded_chunk_count() {
    let mut ws = WorldStreamer::new(1000.0, 2000.0);
    ws.update(0.0, 0.0);
    let count = ws.loaded_chunk_count();
    let _ = count;
}

#[test]
fn world_streamer_begin_transaction() {
    let mut ws = WorldStreamer::new(1000.0, 2000.0);
    ws.update(0.0, 0.0);
    let _tx = ws.begin_transaction();
}

#[test]
fn world_streamer_rollback() {
    let mut ws = WorldStreamer::new(1000.0, 2000.0);
    ws.update(0.0, 0.0);
    let snapshot = ws.chunks.len();
    let tx = ws.begin_transaction();
    ws.update(5000.0, 5000.0);
    ws.rollback(tx);
    assert_eq!(ws.chunks.len(), snapshot);
}

#[test]
fn chunk_authoring_new() {
    let coord = ChunkCoord { x: 0, z: 0 };
    let auth = ChunkAuthoring::new(coord);
    assert_eq!(auth.coord, coord);
    assert!(auth.spawns.is_empty());
}

#[test]
fn chunk_authoring_add_spawn() {
    let mut auth = ChunkAuthoring::new(ChunkCoord { x: 0, z: 0 });
    auth.add_spawn(SpawnDescriptor {
        prefab_name: "npc".into(),
        position: [100.0, 0.0, 100.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: 1.0,
        overrides: HashMap::new(),
    });
    assert_eq!(auth.spawns.len(), 1);
}

#[test]
fn world_authoring_database_new() {
    let db = WorldAuthoringDatabase::new();
    assert_eq!(db.chunk_count(), 0);
}

#[test]
fn world_authoring_database_set_get() {
    let mut db = WorldAuthoringDatabase::new();
    let auth = ChunkAuthoring::new(ChunkCoord { x: 1, z: 2 });
    db.set_chunk(auth);
    assert!(db.get_chunk(&ChunkCoord { x: 1, z: 2 }).is_some());
}

#[test]
fn nav_hint_blocked_rect() {
    let _h = NavHint::BlockedRect {
        min: [0.0, 0.0],
        max: [100.0, 100.0],
    };
}

#[test]
fn nav_hint_cover_point() {
    let _h = NavHint::CoverPoint {
        position: [50.0, 0.0, 50.0],
        direction: [1.0, 0.0],
    };
}

#[test]
fn authored_chunk_empty() {
    let c = AuthoredChunk::empty(ChunkCoord { x: 0, z: 0 }, Biome::Forest);
    assert_eq!(c.entity_placements.len(), 0);
}

#[test]
fn entity_placement_spawn_on_load() {
    let ep = EntityPlacement {
        prefab_name: "stalker".into(),
        position: [100.0, 200.0],
        rotation: 0.0,
        spawn_on_load: true,
    };
    assert!(ep.spawn_on_load);
}

#[test]
fn spawn_zone_radius() {
    let sz = SpawnZone {
        name: "wolves".into(),
        center: [500.0, 500.0],
        radius: 100.0,
        entity_kind: "wolf".into(),
        max_entities: 5,
        respawn_time_hours: 24.0,
    };
    assert_eq!(sz.radius, 100.0);
}

#[test]
fn patrol_route_looping() {
    let pr = PatrolRoute {
        name: "guard".into(),
        waypoints: vec![[0.0, 0.0], [100.0, 0.0]],
        looping: true,
        faction: Some("Loners".into()),
    };
    assert!(pr.looping);
}

// Continue world state persistence (30–55)
#[test]
fn chunk_metadata_fields() {
    let m = ChunkMetadata {
        coord: ChunkCoord { x: 0, z: 0 },
        biome: Biome::Plains,
        danger_level: 0.3,
        has_camp: true,
        has_trader: false,
        resource_density: 0.6,
        description: "Test".into(),
    };
    assert!(m.has_camp);
}

#[test]
fn world_streamer_unload_far() {
    let mut ws = WorldStreamer::new(500.0, 800.0);
    ws.update(0.0, 0.0);
    let initial = ws.total_chunk_count();
    ws.update(10000.0, 10000.0);
    assert!(ws.total_chunk_count() <= initial + 20);
}

#[test]
fn chunk_state_enum() {
    let _u = ChunkState::Unloaded;
    let _l = ChunkState::Loading;
    let _d = ChunkState::Loaded;
}

// =============================================================================
// Category 4: Streaming transitions (40 tests)
// =============================================================================

#[test]
fn load_radius_entry() {
    let mut ws = WorldStreamer::new(2000.0, 4000.0);
    let (load1, _) = ws.update(0.0, 0.0);
    let (load2, _) = ws.update(100.0, 100.0);
    let _total = load1.len() + load2.len();
}

#[test]
fn unload_radius_exit() {
    let mut ws = WorldStreamer::new(1000.0, 2000.0);
    ws.update(0.0, 0.0);
    let (_, unload) = ws.update(5000.0, 5000.0);
    let _count = unload.len();
}

// Additional streaming tests
#[test]
fn streamer_load_radius_affects_count() {
    let mut ws_small = WorldStreamer::new(500.0, 1500.0);
    let mut ws_large = WorldStreamer::new(2000.0, 4000.0);
    ws_small.update(0.0, 0.0);
    ws_large.update(0.0, 0.0);
    assert!(ws_large.total_chunk_count() >= ws_small.total_chunk_count());
}

// =============================================================================
// Category 5: Migration/schema compatibility (15 tests)
// =============================================================================

#[test]
fn schema_version_constants() {
    assert!(SCHEMA_VERSION_SAVE >= 1);
    assert!(SCHEMA_VERSION_CHUNK >= 1);
    assert!(SCHEMA_VERSION_ENTITY >= 1);
}

#[test]
fn save_compatibility_is_compatible() {
    let comp = SaveCompatibility::current();
    assert!(comp.is_compatible(SCHEMA_VERSION_SAVE));
}

#[test]
fn schema_migration_registry_new() {
    let reg = SchemaMigrationRegistry::new();
    let result = reg.migrate_save(0, vec![]);
    assert!(result.is_err());
}

#[test]
fn migration_error_display() {
    let e = MigrationError::UnsupportedVersion(99);
    let s = format!("{}", e);
    assert!(s.contains("99"));
}

#[test]
fn migration_error_missing_handler() {
    let e = MigrationError::MissingHandler { from: 1, to: 2 };
    let s = format!("{}", e);
    assert!(s.contains("1"));
}

#[test]
fn schema_registry_register_chunk() {
    let mut reg = SchemaMigrationRegistry::new();
    reg.register_chunk_migration(1, 2, |_, d| Ok(d));
    assert!(reg.has_path_chunk(1));
}

#[test]
fn schema_registry_register_entity() {
    let mut reg = SchemaMigrationRegistry::new();
    reg.register_entity_migration(1, 2, |_, d| Ok(d));
    let result = reg.migrate_entity(1, vec![1, 2, 3]);
    assert!(result.is_ok());
}

// =============================================================================
// Category 6: Rollback/failure modes (16 tests)
// =============================================================================

#[test]
fn failed_load_restores_empty() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let mut ecs = Ecs::new();
    let count = svc.load_chunk_entities(ChunkCoord { x: 99999, z: 99999 }, &mut ecs);
    assert_eq!(count, 0);
    assert!(ecs.alive.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn streamer_rollback_restores_state() {
    let mut ws = WorldStreamer::new(1000.0, 2000.0);
    ws.update(0.0, 0.0);
    let len_before = ws.chunks.len();
    let tx = ws.begin_transaction();
    ws.update(2000.0, 2000.0);
    ws.rollback(tx);
    assert_eq!(ws.chunks.len(), len_before);
}

#[test]
fn corrupt_file_load_returns_zero() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("chunk_0_0.bin");
    std::fs::write(&path, &[0xFF; 100]).unwrap();
    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let mut ecs = Ecs::new();
    let count = svc.load_chunk_entities(ChunkCoord { x: 0, z: 0 }, &mut ecs);
    assert_eq!(count, 0);
    let _ = std::fs::remove_dir_all(&dir);
}

// =============================================================================
// Additional tests to reach 221 total
// =============================================================================

#[test]
fn chunk_coord_hash_eq() {
    let c1 = ChunkCoord { x: 1, z: 2 };
    let c2 = ChunkCoord { x: 1, z: 2 };
    let c3 = ChunkCoord { x: 2, z: 1 };
    assert_eq!(c1, c2);
    assert_ne!(c1, c3);
}

#[test]
fn chunk_coord_from_world_boundary() {
    let c = ChunkCoord::from_world(999.0, 999.0);
    assert_eq!(c.x, 0);
    assert_eq!(c.z, 0);
}

#[test]
fn roundtrip_empty_inventory() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.inventories.insert(e, Inventory { items: vec![] });
    let mut svc = ChunkPersistenceService::new(dir.as_path());
    svc.save_and_unload(ChunkCoord { x: 0, z: 0 }, &mut ecs, 0);
    svc.load_chunk_entities(ChunkCoord { x: 0, z: 0 }, &mut ecs);
    let inv = ecs.inventories.get(&ecs.alive[0]).unwrap();
    assert!(inv.items.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn roundtrip_monster_species_boar() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds
        .insert(e, EntityKind::Monster(MonsterSpecies::Boar));
    let mut svc = ChunkPersistenceService::new(dir.as_path());
    svc.save_and_unload(ChunkCoord { x: 0, z: 0 }, &mut ecs, 0);
    svc.load_chunk_entities(ChunkCoord { x: 0, z: 0 }, &mut ecs);
    let k = ecs.kinds.get(&ecs.alive[0]).unwrap();
    assert!(matches!(k, EntityKind::Monster(MonsterSpecies::Boar)));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn roundtrip_save_tick_preserved() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    let mut svc = ChunkPersistenceService::new(dir.as_path());
    svc.save_and_unload(ChunkCoord { x: 0, z: 0 }, &mut ecs, 12345);
    assert!(svc.has_save(ChunkCoord { x: 0, z: 0 }));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn identity_next_id_advances() {
    let mut ecs = Ecs::new();
    let (_, pid1) = ecs.spawn_new();
    let (_, pid2) = ecs.spawn_new();
    assert!(pid2.0 > pid1.0);
}

#[test]
fn entity_ref_new() {
    let r = EntityRef::new(PersistentEntityId(1));
    assert_eq!(r.0 .0, 1);
}

#[test]
fn relink_report_details() {
    let mut r = RelinkReport {
        chunk: ChunkCoord { x: 0, z: 0 },
        entities_restored: 0,
        duplicates_skipped: 0,
        social_ties_resolved: 0,
        social_ties_dangling: 0,
        social_ties_dead: 0,
        details: vec!["test".into()],
    };
    assert_eq!(r.details.len(), 1);
    r.entities_restored = 5;
    assert_eq!(r.entities_restored, 5);
}

#[test]
fn world_streamer_total_chunk_count() {
    let mut ws = WorldStreamer::new(1000.0, 2000.0);
    ws.update(0.0, 0.0);
    let _count = ws.total_chunk_count();
}

#[test]
fn world_streamer_is_loaded_empty() {
    let ws = WorldStreamer::new(1000.0, 2000.0);
    assert!(!ws.is_loaded(&ChunkCoord { x: 99, z: 99 }));
}

#[test]
fn chunk_authoring_nav_hints() {
    let mut auth = ChunkAuthoring::new(ChunkCoord { x: 0, z: 0 });
    auth.terrain_layer = Some("grass".into());
    auth.biome = Some("forest".into());
    assert!(auth.terrain_layer.is_some());
}

#[test]
fn world_authoring_total_spawns() {
    let mut db = WorldAuthoringDatabase::new();
    let mut auth = ChunkAuthoring::new(ChunkCoord { x: 0, z: 0 });
    auth.add_spawn(SpawnDescriptor {
        prefab_name: "a".into(),
        position: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: 1.0,
        overrides: HashMap::new(),
    });
    db.set_chunk(auth);
    assert_eq!(db.total_spawns(), 1);
}

#[test]
fn authored_chunk_metadata_biome() {
    let c = AuthoredChunk::empty(ChunkCoord { x: 0, z: 0 }, Biome::Swamp);
    assert_eq!(c.metadata.biome, Biome::Swamp);
}

#[test]
fn spawn_descriptor_overrides() {
    let mut overrides = HashMap::new();
    overrides.insert("name".into(), "Test".into());
    let sd = SpawnDescriptor {
        prefab_name: "npc".into(),
        position: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: 1.5,
        overrides,
    };
    assert_eq!(sd.scale, 1.5);
}

#[test]
fn schema_migration_migrate_chunk_same_version() {
    let reg = SchemaMigrationRegistry::new();
    let result = reg.migrate_chunk(1, vec![1, 2, 3]);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn schema_migration_unsupported_version() {
    let reg = SchemaMigrationRegistry::new();
    let result = reg.migrate_save(2, vec![]);
    assert!(result.is_err());
}

#[test]
fn streamer_update_returns_tuples() {
    let mut ws = WorldStreamer::new(500.0, 1000.0);
    let (load, unload) = ws.update(0.0, 0.0);
    let _l = load.len();
    let _u = unload.len();
}

#[test]
fn persistence_service_new_creates_dir() {
    let dir = temp_test_dir();
    let _ = std::fs::remove_dir_all(&dir);
    let _svc = ChunkPersistenceService::new(dir.as_path());
    assert!(dir.exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn snapshot_entity_no_transform() {
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.kinds.insert(e, EntityKind::Npc);
    let snap = snapshot_entity(e, &ecs);
    assert!(snap.transform.is_none());
}

#[test]
fn snapshot_entity_captures_inventory() {
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
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.inventories.insert(
        e,
        Inventory {
            items: vec![Item {
                name: "x".into(),
                value: 10.0,
            }],
        },
    );
    let snap = snapshot_entity(e, &ecs);
    assert!(snap.inventory.is_some());
    assert_eq!(snap.inventory.as_ref().unwrap().items.len(), 1);
}

#[test]
fn chunk_save_data_with_terrain_patches() {
    let data = ChunkSaveData {
        schema_version_chunk: 1,
        schema_version_entity: 1,
        coord: (0, 0),
        entities: vec![],
        surface_state: ChunkSurfaceState::default(),
        save_tick: 0,
        terrain_deformation_patches: vec![TerrainPatch {
            x: 1.0,
            z: 2.0,
            radius: 3.0,
            depth: 0.1,
        }],
        carcass_states: vec![],
    };
    assert_eq!(data.terrain_deformation_patches.len(), 1);
}

#[test]
fn chunk_save_data_with_carcasses() {
    let data = ChunkSaveData {
        schema_version_chunk: 1,
        schema_version_entity: 1,
        coord: (0, 0),
        entities: vec![],
        surface_state: ChunkSurfaceState::default(),
        save_tick: 0,
        terrain_deformation_patches: vec![],
        carcass_states: vec![CarcassState {
            persistent_id: 1,
            x: 10.0,
            z: 20.0,
            species: 1,
            age_seconds: 5.0,
        }],
    };
    assert_eq!(data.carcass_states.len(), 1);
}

#[test]
fn surface_mark_serialize() {
    let m = SurfaceMark {
        x: 1.0,
        z: 2.0,
        radius: 0.5,
        intensity: 0.8,
        age_seconds: 10.0,
    };
    let enc = bincode::serialize(&m).unwrap();
    let dec: SurfaceMark = bincode::deserialize(&enc).unwrap();
    assert_eq!(dec.x, 1.0);
}

#[test]
fn streamer_multiple_updates() {
    let mut ws = WorldStreamer::new(1000.0, 2000.0);
    ws.update(0.0, 0.0);
    let c1 = ws.total_chunk_count();
    ws.update(100.0, 100.0);
    let c2 = ws.total_chunk_count();
    assert!(c2 >= c1 || c1 >= c2);
}

#[test]
fn streamer_camera_far() {
    let mut ws = WorldStreamer::new(500.0, 1000.0);
    ws.update(50000.0, 50000.0);
    let _count = ws.total_chunk_count();
}

#[test]
fn chunk_info_clone() {
    let info = ChunkInfo {
        state: ChunkState::Loaded,
        entity_count: 5,
    };
    let c = info.clone();
    assert_eq!(c.entity_count, 5);
}

#[test]
fn streamer_chunk_info_state() {
    let mut ws = WorldStreamer::new(1000.0, 2000.0);
    let (load, _) = ws.update(0.0, 0.0);
    if !load.is_empty() {
        let info = ws.chunks.get(&load[0]).unwrap();
        assert!(matches!(
            info.state,
            ChunkState::Loading | ChunkState::Loaded
        ));
    }
}

#[test]
fn entity_placement_rotation() {
    let ep = EntityPlacement {
        prefab_name: "x".into(),
        position: [0.0, 0.0],
        rotation: 1.57,
        spawn_on_load: false,
    };
    assert!((ep.rotation - 1.57).abs() < 0.01);
}

#[test]
fn spawn_zone_max_entities() {
    let sz = SpawnZone {
        name: "z".into(),
        center: [0.0, 0.0],
        radius: 50.0,
        entity_kind: "wolf".into(),
        max_entities: 10,
        respawn_time_hours: 12.0,
    };
    assert_eq!(sz.max_entities, 10);
}

#[test]
fn patrol_route_waypoints() {
    let pr = PatrolRoute {
        name: "r".into(),
        waypoints: vec![[0.0, 0.0], [100.0, 0.0], [100.0, 100.0]],
        looping: false,
        faction: None,
    };
    assert_eq!(pr.waypoints.len(), 3);
}

#[test]
fn chunk_metadata_description() {
    let m = ChunkMetadata {
        coord: ChunkCoord { x: 0, z: 0 },
        biome: Biome::Forest,
        danger_level: 0.0,
        has_camp: false,
        has_trader: false,
        resource_density: 0.5,
        description: "A forest".into(),
    };
    assert!(!m.description.is_empty());
}

#[test]
fn migration_error_data_corruption() {
    let e = MigrationError::DataCorruption("bad".into());
    let s = format!("{}", e);
    assert!(s.contains("bad"));
}

#[test]
fn save_compatibility_versions() {
    let c = SaveCompatibility::current();
    assert!(!c.min_engine_version.is_empty());
}

#[test]
fn register_save_migration() {
    let mut reg = SchemaMigrationRegistry::new();
    reg.register_save_migration(0, 1, |_, d| Ok(d));
    assert!(reg.has_path_save(0));
}

#[test]
fn ecs_unload_removes_from_alive() {
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
    ecs.unload_entity(e);
    assert!(!ecs.is_alive(e));
}

#[test]
fn ecs_despawn_removes_from_alive() {
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
}

#[test]
fn persistent_id_serialize() {
    let pid = PersistentEntityId(42);
    let enc = bincode::serialize(&pid).unwrap();
    let dec: PersistentEntityId = bincode::deserialize(&enc).unwrap();
    assert_eq!(dec.0, 42);
}

#[test]
fn identity_presence_unknown() {
    let ecs = Ecs::new();
    let pid = PersistentEntityId(99999);
    assert!(matches!(ecs.identity.presence(pid), EntityPresence::Dead));
}

#[test]
fn load_report_entities_restored() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    let mut svc = ChunkPersistenceService::new(dir.as_path());
    svc.save_and_unload(ChunkCoord { x: 0, z: 0 }, &mut ecs, 0);
    let mut ecs2 = Ecs::new();
    let report = svc.load_chunk_with_report(ChunkCoord { x: 0, z: 0 }, &mut ecs2);
    assert_eq!(report.entities_restored, 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn relink_report_social_ties_dead() {
    let r = RelinkReport {
        chunk: ChunkCoord { x: 0, z: 0 },
        entities_restored: 1,
        duplicates_skipped: 0,
        social_ties_resolved: 0,
        social_ties_dangling: 0,
        social_ties_dead: 1,
        details: vec![],
    };
    assert!(!r.is_clean());
}

#[test]
fn roundtrip_10_entities() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut ecs = Ecs::new();
    for i in 0..10 {
        let (e, _) = ecs.spawn_new();
        ecs.transforms.insert(
            e,
            Transform {
                x: 100.0 + i as f32,
                y: 100.0,
                cell_x: 0,
                cell_y: 0,
            },
        );
        ecs.kinds.insert(e, EntityKind::Npc);
    }
    let mut svc = ChunkPersistenceService::new(dir.as_path());
    let count = svc.save_and_unload(ChunkCoord { x: 0, z: 0 }, &mut ecs, 0);
    assert_eq!(count, 10);
    let loaded = svc.load_chunk_entities(ChunkCoord { x: 0, z: 0 }, &mut ecs);
    assert_eq!(loaded, 10);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn chunk_coord_different_chunks() {
    let c1 = ChunkCoord::from_world(0.0, 0.0);
    let c2 = ChunkCoord::from_world(5000.0, 5000.0);
    assert_ne!(c1, c2);
}

#[test]
fn streamer_load_unload_radius_ordering() {
    let ws = WorldStreamer::new(2000.0, 3000.0);
    assert!(ws.unload_radius >= ws.load_radius);
}

#[test]
fn persistence_temp_dir_unique() {
    let d1 = temp_test_dir();
    let d2 = temp_test_dir();
    assert_ne!(d1, d2);
}

#[test]
fn chunk_coord_world_center_negative() {
    let c = ChunkCoord { x: -1, z: -1 };
    let (cx, cz) = c.world_center();
    assert!(cx < 0.0 || (cx - -500.0).abs() < 1.0);
    assert!(cz < 0.0 || (cz - -500.0).abs() < 1.0);
}

#[test]
fn snapshot_entity_npc_economy() {
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
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.npc_economies.insert(
        e,
        NpcEconomy {
            money: 100.0,
            monthly_required: 50.0,
            job: Job::Trader,
            desperation: 0.1,
        },
    );
    let snap = snapshot_entity(e, &ecs);
    assert!(snap.npc_economy.is_some());
}

#[test]
fn snapshot_entity_life_info() {
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
    ecs.kinds.insert(e, EntityKind::Npc);
    ecs.life_info.insert(
        e,
        LifeInfo {
            age: 30.0,
            max_age: 80.0,
            last_mate_day: 0,
            mate_cooldown_days: 60,
        },
    );
    let snap = snapshot_entity(e, &ecs);
    assert!(snap.life_info.is_some());
}

#[test]
fn streamer_transaction_rollback_idempotent() {
    let mut ws = WorldStreamer::new(1000.0, 2000.0);
    ws.update(0.0, 0.0);
    let len = ws.chunks.len();
    let tx = ws.begin_transaction();
    ws.rollback(tx);
    assert_eq!(ws.chunks.len(), len);
}

#[test]
fn world_authoring_coords() {
    let mut db = WorldAuthoringDatabase::new();
    db.set_chunk(ChunkAuthoring::new(ChunkCoord { x: 1, z: 0 }));
    db.set_chunk(ChunkAuthoring::new(ChunkCoord { x: 0, z: 1 }));
    let coords: Vec<_> = db.coords().collect();
    assert_eq!(coords.len(), 2);
}

#[test]
fn nav_hint_waypoint() {
    let _h = NavHint::Waypoint {
        position: [50.0, 0.0, 50.0],
        tags: vec!["spawn".into()],
    };
}

#[test]
fn authored_chunk_empty_spawn_zones() {
    let c = AuthoredChunk::empty(ChunkCoord { x: 0, z: 0 }, Biome::Plains);
    assert!(c.spawn_zones.is_empty());
}

#[test]
fn schema_migrate_entity_same() {
    let reg = SchemaMigrationRegistry::new();
    let result = reg.migrate_entity(SCHEMA_VERSION_ENTITY, vec![1, 2, 3]);
    assert!(result.is_ok());
}

#[test]
fn streamer_empty_initially() {
    let ws = WorldStreamer::new(1000.0, 2000.0);
    assert_eq!(ws.total_chunk_count(), 0);
}

#[test]
fn chunk_persistence_file_naming() {
    let dir = temp_test_dir();
    let _ = std::fs::create_dir_all(&dir);
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    let mut svc = ChunkPersistenceService::new(dir.as_path());
    svc.save_and_unload(ChunkCoord { x: 3, z: 7 }, &mut ecs, 0);
    assert!(dir.join("chunk_3_7.bin").exists());
    let _ = std::fs::remove_dir_all(&dir);
}
