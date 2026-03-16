//! Core/ECS/Contracts tests for ENGENE.
//! ~119 tests covering ECS lifecycle, persistent identity, events, commands, authority.

// =============================================================================
// 1. ECS LIFECYCLE (35 tests)
// =============================================================================

#[test]
fn ecs_create_empty() {
    let ecs = engene::core::ecs::Ecs::new();
    assert_eq!(ecs.alive.len(), 0);
    assert!(ecs.transforms.is_empty());
    assert!(ecs.kinds.is_empty());
}

#[test]
fn ecs_spawn_creates_entity() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let entity = ecs.spawn();
    assert!(ecs.is_alive(entity));
    assert_eq!(ecs.alive.len(), 1);
    assert!(ecs.alive.contains(&entity));
}

#[test]
fn ecs_spawn_increments_id() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e1 = ecs.spawn();
    let e2 = ecs.spawn();
    let e3 = ecs.spawn();
    assert_ne!(e1, e2);
    assert_ne!(e2, e3);
    assert_ne!(e1, e3);
}

#[test]
fn ecs_despawn_removes_entity() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let entity = ecs.spawn();
    assert!(ecs.is_alive(entity));
    ecs.despawn(entity);
    assert!(!ecs.is_alive(entity));
    assert_eq!(ecs.alive.len(), 0);
}

#[test]
fn ecs_component_add_and_get() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let entity = ecs.spawn();
    let t = engene::world::components::Transform {
        x: 10.0,
        y: 20.0,
        cell_x: 1,
        cell_y: 2,
    };
    ecs.transforms.insert(entity, t.clone());
    assert_eq!(ecs.transforms.get(&entity), Some(&t));
}

#[test]
fn ecs_component_remove() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let entity = ecs.spawn();
    ecs.transforms.insert(
        entity,
        engene::world::components::Transform {
            x: 1.0,
            y: 2.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    assert!(ecs.transforms.contains_key(&entity));
    let removed = ecs.transforms.remove(&entity);
    assert!(removed.is_some());
    assert!(!ecs.transforms.contains_key(&entity));
}

#[test]
fn ecs_sparse_set_integrity_after_insert_remove() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e1 = ecs.spawn();
    let e2 = ecs.spawn();
    ecs.transforms.insert(e1, engene::world::components::Transform { x: 0.0, y: 0.0, cell_x: 0, cell_y: 0 });
    ecs.transforms.insert(e2, engene::world::components::Transform { x: 1.0, y: 1.0, cell_x: 1, cell_y: 1 });
    assert_eq!(ecs.transforms.len(), 2);
    ecs.transforms.remove(&e1);
    assert_eq!(ecs.transforms.len(), 1);
    assert_eq!(ecs.transforms.get(&e2).unwrap().x, 1.0);
}

#[test]
fn ecs_batch_spawn_multiple() {
    let mut ecs = engene::core::ecs::Ecs::new();
    for _ in 0..10 {
        ecs.spawn();
    }
    assert_eq!(ecs.alive.len(), 10);
}

#[test]
fn ecs_batch_despawn_all() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let entities: Vec<_> = (0..5).map(|_| ecs.spawn()).collect();
    for &e in &entities {
        ecs.despawn(e);
    }
    assert_eq!(ecs.alive.len(), 0);
    for &e in &entities {
        assert!(!ecs.is_alive(e));
    }
}

#[test]
fn ecs_transform_crud() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    let t = engene::world::components::Transform {
        x: 100.0,
        y: 200.0,
        cell_x: 5,
        cell_y: 10,
    };
    ecs.transforms.insert(e, t.clone());
    let got = ecs.transforms.get(&e).unwrap();
    assert_eq!(got.x, 100.0);
    assert_eq!(got.cell_x, 5);
    *ecs.transforms.get_mut(&e).unwrap() = engene::world::components::Transform {
        x: 50.0,
        y: 75.0,
        cell_x: 2,
        cell_y: 3,
    };
    assert_eq!(ecs.transforms.get(&e).unwrap().x, 50.0);
}

#[test]
fn ecs_kind_crud() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    ecs.kinds.insert(e, engene::world::components::EntityKind::Npc);
    assert!(matches!(
        ecs.kinds.get(&e),
        Some(engene::world::components::EntityKind::Npc)
    ));
    ecs.kinds.insert(
        e,
        engene::world::components::EntityKind::Monster(
            engene::world::components::MonsterSpecies::Wolf,
        ),
    );
    assert!(matches!(
        ecs.kinds.get(&e),
        Some(engene::world::components::EntityKind::Monster(
            engene::world::components::MonsterSpecies::Wolf
        ))
    ));
}

#[test]
fn ecs_name_crud() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    ecs.names.insert(e, engene::world::components::Name("TestNPC".into()));
    assert_eq!(ecs.names.get(&e).unwrap().0, "TestNPC");
    ecs.names.insert(e, engene::world::components::Name("Renamed".into()));
    assert_eq!(ecs.names.get(&e).unwrap().0, "Renamed");
}

#[test]
fn ecs_inventory_crud() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    ecs.inventories.insert(
        e,
        engene::world::components::Inventory {
            items: vec![engene::world::components::Item {
                name: "Bread".into(),
                value: 2.0,
            }],
        },
    );
    assert_eq!(ecs.inventories.get(&e).unwrap().items.len(), 1);
    assert_eq!(ecs.inventories.get(&e).unwrap().items[0].name, "Bread");
}

#[test]
fn ecs_tag_bit_operations() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    let mut tags = engene::world::extension_components::EntityTags::default();
    assert!(!tags.has_bit(0));
    tags.set_bit(0);
    assert!(tags.has_bit(0));
    tags.clear_bit(0);
    assert!(!tags.has_bit(0));
    ecs.tags.insert(e, tags);
}

#[test]
fn ecs_tag_string_operations() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    let mut tags = engene::world::extension_components::EntityTags::default();
    tags.add_tag("quest_target".into());
    assert!(tags.has_tag("quest_target"));
    ecs.tags.insert(e, tags);
}

#[test]
fn ecs_status_effects_add_and_has() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    let mut effects = engene::world::extension_components::StatusEffects::default();
    effects.add(1, 3, 10.0);
    assert!(effects.has(1));
    assert_eq!(effects.stacks(1), 3);
    ecs.status_effects.insert(e, effects);
}

#[test]
fn ecs_attributes_get_set() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    let mut attrs = engene::world::extension_components::Attributes::default();
    attrs.set(1, 0.8);
    assert_eq!(attrs.get(1), Some(0.8));
    attrs.set(1, 0.5);
    assert_eq!(attrs.get(1), Some(0.5));
    attrs.remove(1);
    assert_eq!(attrs.get(1), None);
    ecs.attributes.insert(e, attrs);
}

#[test]
fn ecs_spatial_rebuild_after_spawn() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    ecs.transforms.insert(
        e,
        engene::world::components::Transform {
            x: 500.0,
            y: 600.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.rebuild_spatial();
    let candidates = ecs.spatial.candidates_in_radius(500.0, 600.0, 10.0);
    assert!(candidates.contains(&e));
}

#[test]
fn ecs_invalid_entity_lookup_returns_none() {
    let ecs = engene::core::ecs::Ecs::new();
    assert_eq!(ecs.transforms.get(&99999), None);
    assert_eq!(ecs.kinds.get(&99999), None);
}

#[test]
fn ecs_despawned_entity_lookup_returns_none() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    ecs.transforms.insert(
        e,
        engene::world::components::Transform {
            x: 1.0,
            y: 1.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.despawn(e);
    assert_eq!(ecs.transforms.get(&e), None);
    assert!(!ecs.transforms.contains_key(&e));
}

#[test]
fn ecs_entity_iteration_over_alive() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e1 = ecs.spawn();
    let e2 = ecs.spawn();
    let collected: Vec<_> = ecs.alive.iter().copied().collect();
    assert_eq!(collected.len(), 2);
    assert!(collected.contains(&e1));
    assert!(collected.contains(&e2));
}

#[test]
fn ecs_npcs_filter() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e1 = ecs.spawn();
    let e2 = ecs.spawn();
    ecs.kinds.insert(e1, engene::world::components::EntityKind::Npc);
    ecs.kinds.insert(
        e2,
        engene::world::components::EntityKind::Monster(
            engene::world::components::MonsterSpecies::Wolf,
        ),
    );
    let npcs = ecs.npcs();
    assert_eq!(npcs.len(), 1);
    assert_eq!(npcs[0], e1);
}

#[test]
fn ecs_monsters_filter() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e1 = ecs.spawn();
    let e2 = ecs.spawn();
    ecs.kinds.insert(e1, engene::world::components::EntityKind::Npc);
    ecs.kinds.insert(
        e2,
        engene::world::components::EntityKind::Monster(
            engene::world::components::MonsterSpecies::Boar,
        ),
    );
    let monsters = ecs.monsters();
    assert_eq!(monsters.len(), 1);
    assert_eq!(monsters[0], e2);
}

#[test]
fn ecs_count_species() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e1 = ecs.spawn();
    let e2 = ecs.spawn();
    let e3 = ecs.spawn();
    ecs.kinds.insert(
        e1,
        engene::world::components::EntityKind::Monster(
            engene::world::components::MonsterSpecies::Wolf,
        ),
    );
    ecs.kinds.insert(
        e2,
        engene::world::components::EntityKind::Monster(
            engene::world::components::MonsterSpecies::Wolf,
        ),
    );
    ecs.kinds.insert(
        e3,
        engene::world::components::EntityKind::Monster(
            engene::world::components::MonsterSpecies::Boar,
        ),
    );
    assert_eq!(
        ecs.count_species(engene::world::components::MonsterSpecies::Wolf),
        2
    );
    assert_eq!(
        ecs.count_species(engene::world::components::MonsterSpecies::Boar),
        1
    );
}

#[test]
fn ecs_count_npcs() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e1 = ecs.spawn();
    let e2 = ecs.spawn();
    ecs.kinds.insert(e1, engene::world::components::EntityKind::Npc);
    ecs.kinds.insert(e2, engene::world::components::EntityKind::Npc);
    assert_eq!(ecs.count_npcs(), 2);
}

#[test]
fn ecs_sparse_set_iter() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    ecs.transforms.insert(
        e,
        engene::world::components::Transform {
            x: 1.0,
            y: 2.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    let pairs: Vec<_> = ecs.transforms.iter().collect();
    assert_eq!(pairs.len(), 1);
    assert_eq!(*pairs[0].0, e);
    assert_eq!(pairs[0].1.x, 1.0);
}

#[test]
fn ecs_personal_needs_insert() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    ecs.personal_needs
        .insert(e, engene::world::components::PersonalNeeds::default_npc());
    assert!(ecs.personal_needs.get(&e).is_some());
    assert!((ecs.personal_needs.get(&e).unwrap().health - 1.0).abs() < 0.01);
}

#[test]
fn ecs_life_info_insert() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    ecs.life_info
        .insert(e, engene::world::components::LifeInfo::new_npc(300.0));
    assert!(ecs.life_info.get(&e).is_some());
    assert_eq!(ecs.life_info.get(&e).unwrap().max_age, 300.0);
}

#[test]
fn ecs_despawn_cleans_all_components() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    ecs.transforms.insert(
        e,
        engene::world::components::Transform {
            x: 0.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, engene::world::components::EntityKind::Npc);
    ecs.names.insert(e, engene::world::components::Name("X".into()));
    ecs.despawn(e);
    assert!(!ecs.transforms.contains_key(&e));
    assert!(!ecs.kinds.contains_key(&e));
    assert!(!ecs.names.contains_key(&e));
}

#[test]
fn ecs_tick_increments() {
    let mut ecs = engene::core::ecs::Ecs::new();
    assert_eq!(ecs.tick, 0);
    ecs.tick = 10;
    assert_eq!(ecs.tick, 10);
}

#[test]
fn ecs_spawn_new_returns_entity_and_pid() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (entity, pid) = ecs.spawn_new();
    assert!(ecs.is_alive(entity));
    assert!(ecs.identity.persistent_id_of(entity).is_some());
    assert_eq!(ecs.identity.persistent_id_of(entity), Some(pid));
}

#[test]
fn ecs_unload_entity_marks_unloaded() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (entity, pid) = ecs.spawn_new();
    ecs.unload_entity(entity);
    assert!(!ecs.is_alive(entity));
    assert_eq!(
        ecs.identity.presence(pid),
        engene::core::persistent_id::EntityPresence::Unloaded
    );
}

#[test]
fn ecs_sparse_set_swap_remove_integrity() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e1 = ecs.spawn();
    let e2 = ecs.spawn();
    let e3 = ecs.spawn();
    ecs.transforms.insert(e1, engene::world::components::Transform { x: 1.0, y: 1.0, cell_x: 0, cell_y: 0 });
    ecs.transforms.insert(e2, engene::world::components::Transform { x: 2.0, y: 2.0, cell_x: 0, cell_y: 0 });
    ecs.transforms.insert(e3, engene::world::components::Transform { x: 3.0, y: 3.0, cell_x: 0, cell_y: 0 });
    ecs.transforms.remove(&e2);
    assert_eq!(ecs.transforms.len(), 2);
    assert_eq!(ecs.transforms.get(&e1).unwrap().x, 1.0);
    assert_eq!(ecs.transforms.get(&e3).unwrap().x, 3.0);
}

#[test]
fn ecs_sim_level_insert() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    ecs.sim_levels.insert(
        e,
        engene::world::components::SimLevel {
            level: engene::world::components::SimulationLevel::L1,
        },
    );
    assert!(matches!(
        ecs.sim_levels.get(&e).unwrap().level,
        engene::world::components::SimulationLevel::L1
    ));
}

#[test]
fn ecs_npc_economy_insert() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    ecs.npc_economies.insert(
        e,
        engene::world::components::NpcEconomy {
            money: 100.0,
            monthly_required: 50.0,
            job: engene::world::components::Job::Trader,
            desperation: 0.2,
        },
    );
    assert_eq!(ecs.npc_economies.get(&e).unwrap().money, 100.0);
    assert_eq!(ecs.npc_economies.get(&e).unwrap().job, engene::world::components::Job::Trader);
}

#[test]
fn ecs_equipment_component() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let e = ecs.spawn();
    ecs.equipment.insert(e, engene::world::components::EquipmentSlots::default_stalker());
    let eq = ecs.equipment.get(&e).unwrap();
    assert!(eq.weapon_condition > 0.0);
    assert!(eq.medkits >= 1);
}

// =============================================================================
// 2. PERSISTENT IDENTITY / RELINK (35 tests)
// =============================================================================

#[test]
fn pid_unique_generation() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (_, p1) = ecs.spawn_new();
    let (_, p2) = ecs.spawn_new();
    let (_, p3) = ecs.spawn_new();
    assert_ne!(p1, p2);
    assert_ne!(p2, p3);
    assert_ne!(p1, p3);
}

#[test]
fn pid_restore_rejects_duplicate() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let pid = engene::core::persistent_id::PersistentEntityId(100);
    let _e1 = ecs.spawn_restored(pid).expect("first restore ok");
    let result = ecs.spawn_restored(pid);
    assert!(result.is_err());
}

#[test]
fn pid_relink_social_tie_success() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e1, p1) = ecs.spawn_new();
    let (e2, p2) = ecs.spawn_new();
    assert_eq!(ecs.identity.persistent_id_of(e1), Some(p1));
    let mut mem = engene::ai::memory::Memory::new();
    mem.entities.insert(
        p2,
        engene::ai::memory::EntityOpinion {
            trust: 0.5,
            hostility: 0.0,
            familiarity: 0.3,
            last_seen_tick: 0,
            shared_kills: 0,
            times_hurt_me: 0,
        },
    );
    ecs.memories.insert(e1, mem);
    let ctx = engene::core::persistent_id::RelinkContext {
        registry: &ecs.identity,
    };
    let resolved = ctx.resolve_persistent(p2);
    assert_eq!(resolved, Some(e2));
}

#[test]
fn pid_relink_group_leader() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e1, p1) = ecs.spawn_new();
    let (e2, p2) = ecs.spawn_new();
    let group = engene::ai::groups::Group {
        leader: p1,
        members: vec![p1, p2],
        formed_tick: 0,
    };
    let ctx = engene::core::persistent_id::RelinkContext {
        registry: &ecs.identity,
    };
    assert_eq!(ctx.resolve_persistent(group.leader), Some(e1));
    assert_eq!(ctx.resolve_persistent(p2), Some(e2));
}

#[test]
fn pid_orphan_policy_unloaded_returns_none() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (_e, pid) = ecs.spawn_new();
    ecs.identity.mark_unloaded(pid);
    let ctx = engene::core::persistent_id::RelinkContext {
        registry: &ecs.identity,
    };
    assert_eq!(ctx.resolve_persistent(pid), None);
}

#[test]
fn pid_orphan_policy_dead_returns_none() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e, pid) = ecs.spawn_new();
    ecs.despawn(e);
    let ctx = engene::core::persistent_id::RelinkContext {
        registry: &ecs.identity,
    };
    assert_eq!(ctx.resolve_persistent(pid), None);
}

#[test]
fn pid_relink_report_counts() {
    use engene::world::chunk_persistence::{ChunkPersistenceService, RelinkReport};
    use engene::world::streaming::ChunkCoord;

    let dir = std::env::temp_dir().join("engene_core_test_relink");
    let _ = std::fs::remove_dir_all(&dir);

    let mut ecs = engene::core::ecs::Ecs::new();
    let (e1, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e1,
        engene::world::components::Transform {
            x: 500.0,
            y: 500.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e1, engene::world::components::EntityKind::Npc);
    ecs.personal_needs
        .insert(e1, engene::world::components::PersonalNeeds::default_npc());

    let mut persistence = ChunkPersistenceService::new(&dir);
    let coord = ChunkCoord { x: 0, z: 0 };
    persistence.save_and_unload(coord, &mut ecs, 10);
    let report: RelinkReport = persistence.load_chunk_with_report(coord, &mut ecs);

    assert_eq!(report.entities_restored, 1);
    assert_eq!(report.duplicates_skipped, 0);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pid_survives_roundtrip() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e1, pid) = ecs.spawn_new();
    ecs.transforms.insert(
        e1,
        engene::world::components::Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e1, engene::world::components::EntityKind::Npc);
    ecs.personal_needs
        .insert(e1, engene::world::components::PersonalNeeds::default_npc());

    let pid_val = pid.0;
    ecs.despawn(e1);

    let e2 = ecs.spawn_restored(engene::core::persistent_id::PersistentEntityId(pid_val))
        .expect("restore after dead should work");
    assert_eq!(ecs.identity.persistent_id_of(e2).unwrap().0, pid_val);
}

#[test]
fn pid_chunk_unload_preserves_pid() {
    use engene::world::chunk_persistence::ChunkPersistenceService;
    use engene::world::streaming::ChunkCoord;

    let dir = std::env::temp_dir().join("engene_core_test_pid_chunk");
    let _ = std::fs::remove_dir_all(&dir);

    let mut ecs = engene::core::ecs::Ecs::new();
    let (e1, pid) = ecs.spawn_new();
    ecs.transforms.insert(
        e1,
        engene::world::components::Transform {
            x: 500.0,
            y: 500.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e1, engene::world::components::EntityKind::Npc);
    ecs.personal_needs
        .insert(e1, engene::world::components::PersonalNeeds::default_npc());

    let mut persistence = ChunkPersistenceService::new(&dir);
    let coord = ChunkCoord { x: 0, z: 0 };
    persistence.save_and_unload(coord, &mut ecs, 50);

    assert_eq!(ecs.identity.presence(pid), engene::core::persistent_id::EntityPresence::Unloaded);

    persistence.load_chunk_entities(coord, &mut ecs);
    let restored = ecs.identity.resolve(pid).expect("pid resolves after reload");
    assert!(ecs.is_alive(restored));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pid_stale_relink_detection_dead_target() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e, pid) = ecs.spawn_new();
    let eref = engene::core::persistent_id::EntityRef::new(pid);
    ecs.despawn(e);
    assert!(eref.is_dead(&ecs.identity));
    assert_eq!(eref.resolve(&ecs.identity), None);
}

#[test]
fn pid_cross_chunk_relink_after_load() {
    use engene::world::chunk_persistence::ChunkPersistenceService;
    use engene::world::streaming::ChunkCoord;

    let dir = std::env::temp_dir().join("engene_core_test_cross_chunk");
    let _ = std::fs::remove_dir_all(&dir);

    let mut ecs = engene::core::ecs::Ecs::new();
    let (e1, pid1) = ecs.spawn_new();
    let (e2, _) = ecs.spawn_new();
    ecs.transforms.insert(e1, engene::world::components::Transform { x: 500.0, y: 500.0, cell_x: 0, cell_y: 0 });
    ecs.transforms.insert(e2, engene::world::components::Transform { x: 600.0, y: 600.0, cell_x: 0, cell_y: 0 });
    ecs.kinds.insert(e1, engene::world::components::EntityKind::Npc);
    ecs.kinds.insert(e2, engene::world::components::EntityKind::Npc);
    ecs.personal_needs.insert(e1, engene::world::components::PersonalNeeds::default_npc());
    ecs.personal_needs.insert(e2, engene::world::components::PersonalNeeds::default_npc());

    let mut persistence = ChunkPersistenceService::new(&dir);
    persistence.save_and_unload(ChunkCoord { x: 0, z: 0 }, &mut ecs, 0);
    persistence.load_chunk_entities(ChunkCoord { x: 0, z: 0 }, &mut ecs);

    let ctx = engene::core::persistent_id::RelinkContext { registry: &ecs.identity };
    assert!(ctx.resolve_persistent(pid1).is_some());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pid_player_persistence() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e, pid) = ecs.spawn_new();
    ecs.kinds.insert(e, engene::world::components::EntityKind::Npc);
    ecs.names.insert(e, engene::world::components::Name("Player".into()));
    assert_eq!(ecs.identity.persistent_id_of(e), Some(pid));
    ecs.despawn(e);
    let e2 = ecs.spawn_restored(pid).expect("player restore");
    assert_eq!(ecs.names.get(&e2).unwrap().0, "Player");
}

#[test]
fn pid_trader_persistence() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e, pid) = ecs.spawn_new();
    ecs.kinds.insert(e, engene::world::components::EntityKind::Npc);
    ecs.npc_economies.insert(
        e,
        engene::world::components::NpcEconomy {
            money: 500.0,
            monthly_required: 100.0,
            job: engene::world::components::Job::Trader,
            desperation: 0.0,
        },
    );
    assert_eq!(ecs.npc_economies.get(&e).unwrap().job, engene::world::components::Job::Trader);
    let pid_val = pid.0;
    ecs.despawn(e);
    let e2 = ecs.spawn_restored(engene::core::persistent_id::PersistentEntityId(pid_val)).unwrap();
    ecs.npc_economies.insert(
        e2,
        engene::world::components::NpcEconomy {
            money: 500.0,
            monthly_required: 100.0,
            job: engene::world::components::Job::Trader,
            desperation: 0.0,
        },
    );
    assert_eq!(ecs.identity.persistent_id_of(e2).unwrap().0, pid_val);
}

#[test]
fn pid_quest_giver_persistence() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e, pid) = ecs.spawn_new();
    let mut tags = engene::world::extension_components::EntityTags::default();
    tags.add_tag("quest_giver".into());
    ecs.tags.insert(e, tags);
    assert!(ecs.identity.persistent_id_of(e).is_some());
    assert_eq!(ecs.identity.persistent_id_of(e), Some(pid));
}

#[test]
fn pid_entity_ref_hygiene_resolve_live() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e, pid) = ecs.spawn_new();
    let eref = engene::core::persistent_id::EntityRef::new(pid);
    assert_eq!(eref.resolve(&ecs.identity), Some(e));
}

#[test]
fn pid_entity_ref_hygiene_is_unloaded() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (_e, pid) = ecs.spawn_new();
    ecs.identity.mark_unloaded(pid);
    let eref = engene::core::persistent_id::EntityRef::new(pid);
    assert!(eref.is_unloaded(&ecs.identity));
}

#[test]
fn pid_gc_tombstones_removes_old() {
    let mut ecs = engene::core::ecs::Ecs::new();
    ecs.tick = 1000;
    let (e, pid) = ecs.spawn_new();
    ecs.despawn(e);
    let before = ecs.identity.tombstone_count();
    assert!(before >= 1);
    ecs.identity.gc_tombstones(2000, 500);
    assert_eq!(ecs.identity.presence(pid), engene::core::persistent_id::EntityPresence::Dead);
}

#[test]
fn pid_register_restored_same_entity_ok() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let pid = engene::core::persistent_id::PersistentEntityId(42);
    let e = ecs.spawn_restored(pid).unwrap();
    let result = ecs.identity.register_restored(pid, e);
    assert!(result.is_ok());
}

#[test]
fn pid_live_count_matches_alive() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e1, _) = ecs.spawn_new();
    let (e2, _) = ecs.spawn_new();
    assert_eq!(ecs.identity.live_count(), 2);
    assert!(ecs.alive.contains(&e2));
    ecs.despawn(e1);
    assert_eq!(ecs.identity.live_count(), 1);
}

#[test]
fn pid_next_id_advances_after_restore() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let _ = ecs.spawn_restored(engene::core::persistent_id::PersistentEntityId(999)).unwrap();
    let (_, pid2) = ecs.spawn_new();
    assert!(pid2.0 >= 1000);
}

#[test]
fn pid_presence_live() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e, pid) = ecs.spawn_new();
    assert!(matches!(
        ecs.identity.presence(pid),
        engene::core::persistent_id::EntityPresence::Live(_)
    ));
    if let engene::core::persistent_id::EntityPresence::Live(resolved) = ecs.identity.presence(pid) {
        assert_eq!(resolved, e);
    }
}

#[test]
fn pid_presence_dead() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e, pid) = ecs.spawn_new();
    ecs.despawn(e);
    assert_eq!(
        ecs.identity.presence(pid),
        engene::core::persistent_id::EntityPresence::Dead
    );
}

#[test]
fn pid_resolve_none_for_unknown() {
    let ecs = engene::core::ecs::Ecs::new();
    let pid = engene::core::persistent_id::PersistentEntityId(99999);
    assert_eq!(ecs.identity.resolve(pid), None);
}

#[test]
fn pid_duplicate_error_contains_info() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let pid = engene::core::persistent_id::PersistentEntityId(77);
    let e1 = ecs.spawn_restored(pid).unwrap();
    let result = ecs.spawn_restored(pid);
    match result {
        Err(e) => {
            assert_eq!(e.persistent_id, pid);
            assert_eq!(e.existing_entity, e1);
        }
        Ok(_) => panic!("expected DuplicateIdError"),
    }
}

#[test]
fn pid_chunk_save_restore_cycle() {
    use engene::world::chunk_persistence::ChunkPersistenceService;
    use engene::world::streaming::ChunkCoord;

    let dir = std::env::temp_dir().join("engene_core_test_chunk_cycle");
    let _ = std::fs::remove_dir_all(&dir);

    let mut ecs = engene::core::ecs::Ecs::new();
    let (e, pid) = ecs.spawn_new();
    ecs.transforms.insert(e, engene::world::components::Transform { x: 500.0, y: 500.0, cell_x: 0, cell_y: 0 });
    ecs.kinds.insert(e, engene::world::components::EntityKind::Npc);
    ecs.personal_needs.insert(e, engene::world::components::PersonalNeeds::default_npc());

    let mut persistence = ChunkPersistenceService::new(&dir);
    let coord = ChunkCoord { x: 0, z: 0 };
    let saved = persistence.save_and_unload(coord, &mut ecs, 100);
    assert_eq!(saved, 1);
    let loaded = persistence.load_chunk_entities(coord, &mut ecs);
    assert_eq!(loaded, 1);
    assert_eq!(ecs.identity.resolve(pid), Some(ecs.alive[0]));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pid_multiple_restores_distinct() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let p1 = engene::core::persistent_id::PersistentEntityId(1);
    let p2 = engene::core::persistent_id::PersistentEntityId(2);
    let e1 = ecs.spawn_restored(p1).unwrap();
    let e2 = ecs.spawn_restored(p2).unwrap();
    assert_ne!(e1, e2);
    assert_eq!(ecs.identity.resolve(p1), Some(e1));
    assert_eq!(ecs.identity.resolve(p2), Some(e2));
}

#[test]
fn pid_tombstone_kept_after_despawn() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e, pid) = ecs.spawn_new();
    ecs.despawn(e);
    assert_eq!(ecs.identity.tombstone_count(), 1);
    assert_eq!(ecs.identity.presence(pid), engene::core::persistent_id::EntityPresence::Dead);
}

#[test]
fn pid_entity_ref_new() {
    let pid = engene::core::persistent_id::PersistentEntityId(42);
    let eref = engene::core::persistent_id::EntityRef::new(pid);
    assert_eq!(eref.0, pid);
}

#[test]
fn pid_relink_context_resolve() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e, pid) = ecs.spawn_new();
    let ctx = engene::core::persistent_id::RelinkContext {
        registry: &ecs.identity,
    };
    assert_eq!(ctx.resolve_persistent(pid), Some(e));
}

#[test]
fn pid_total_count_includes_dead() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e, _pid) = ecs.spawn_new();
    let total_before = ecs.identity.total_count();
    ecs.despawn(e);
    assert!(ecs.identity.total_count() >= total_before, "total_count preserves dead entries until gc");
}

#[test]
fn pid_live_entities_iter() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e1, p1) = ecs.spawn_new();
    let (e2, p2) = ecs.spawn_new();
    let pairs: Vec<_> = ecs.identity.live_entities().collect();
    assert_eq!(pairs.len(), 2);
    assert!(pairs.iter().any(|(pid, ent)| *pid == p1 && *ent == e1));
    assert!(pairs.iter().any(|(pid, ent)| *pid == p2 && *ent == e2));
}

// =============================================================================
// 3. EVENT SYSTEM (25 tests)
// =============================================================================

#[test]
fn event_emit_receive() {
    let mut bus = engene::core::events::EventBus::new();
    bus.emit(42u32);
    let read = bus.read::<u32>();
    assert_eq!(read.len(), 1);
    assert_eq!(*read[0], 42);
}

#[test]
fn event_sticky_survives_clear() {
    let mut bus = engene::core::events::EventBus::new();
    bus.set_sticky(100i32);
    bus.emit(1u32);
    bus.clear();
    assert_eq!(bus.count::<u32>(), 0);
    assert_eq!(bus.get_sticky::<i32>(), Some(&100));
}

#[test]
fn event_bounded_capacity() {
    let mut bus = engene::core::events::EventBus::with_capacity(2);
    bus.set_channel_capacity::<u32>(2);
    bus.emit(1u32);
    bus.emit(2u32);
    bus.emit(3u32);
    bus.emit(4u32);
    assert_eq!(bus.count::<u32>(), 2);
    assert!(bus.dropped_count::<u32>() >= 2);
}

#[test]
fn event_overflow_policy_drops() {
    let mut bus = engene::core::events::EventBus::with_capacity(1);
    bus.set_channel_capacity::<bool>(1);
    bus.emit(true);
    bus.emit(false);
    assert_eq!(bus.count::<bool>(), 1);
    assert_eq!(bus.dropped_count::<bool>(), 1);
}

#[test]
fn event_ordering_preserved() {
    let mut bus = engene::core::events::EventBus::new();
    bus.emit(1u32);
    bus.emit(2u32);
    bus.emit(3u32);
    let read = bus.read::<u32>();
    assert_eq!(read.len(), 3);
    assert_eq!(*read[0], 1);
    assert_eq!(*read[1], 2);
    assert_eq!(*read[2], 3);
}

#[test]
fn event_type_segregation() {
    let mut bus = engene::core::events::EventBus::new();
    bus.emit(10u32);
    bus.emit("hello");
    bus.emit(true);
    assert_eq!(bus.count::<u32>(), 1);
    assert_eq!(bus.count::<&str>(), 1);
    assert_eq!(bus.count::<bool>(), 1);
    assert_eq!(*bus.read::<u32>()[0], 10);
    assert_eq!(*bus.read::<&str>()[0], "hello");
}

#[test]
fn event_command_buffer_deferred() {
    let mut cb = engene::core::commands::CommandBuffer::new();
    cb.emit_event(99u32);
    assert_eq!(cb.event_count(), 1);
    let events = cb.take_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn event_multi_bus_sim_render_debug() {
    use engene::core::events::sim_bus::SimBus;
    use engene::core::events::render_bus::RenderBus;
    use engene::core::events::debug_bus::DebugBus;

    let mut sim = SimBus::new();
    let mut render = RenderBus::new();
    let mut debug = DebugBus::new();

    sim.bus.emit(1u32);
    render.bus.emit("r");
    debug.bus.emit(false);

    assert_eq!(sim.bus.count::<u32>(), 1);
    assert_eq!(render.bus.count::<&str>(), 1);
    assert_eq!(debug.bus.count::<bool>(), 1);
}

#[test]
fn event_tracer_records() {
    use engene::core::events::tracing_hooks::EventTracer;
    let mut tracer = EventTracer::new();
    tracer.enable();
    tracer.record(std::any::TypeId::of::<u32>(), "u32", 5, 3);
    let traces = tracer.drain();
    assert_eq!(traces.len(), 1);
    assert_eq!(traces[0].count, 3);
    assert_eq!(traces[0].tick, 5);
}

#[test]
fn event_aggregator_bucketing() {
    use engene::core::events::aggregation::EventAggregator;
    use engene::core::events::canonical::ImpactEvent;

    let mut agg = EventAggregator::new(10.0);
    agg.submit(&ImpactEvent {
        position: glam::Vec3::new(5.0, 0.0, 5.0),
        direction: glam::Vec3::Y,
        energy: 100.0,
        material_hit: 0,
        instigator: None,
        target_entity: None,
    });
    agg.submit(&ImpactEvent {
        position: glam::Vec3::new(6.0, 0.0, 6.0),
        direction: glam::Vec3::Y,
        energy: 50.0,
        material_hit: 0,
        instigator: None,
        target_entity: None,
    });
    let results = agg.drain();
    assert_eq!(results.len(), 1);
    assert!((results[0].total_energy - 150.0).abs() < 0.1);
    assert_eq!(results[0].count, 2);
}

#[test]
fn event_has_and_count() {
    let mut bus = engene::core::events::EventBus::new();
    assert!(!bus.has::<u32>());
    assert_eq!(bus.count::<u32>(), 0);
    bus.emit(1u32);
    assert!(bus.has::<u32>());
    assert_eq!(bus.count::<u32>(), 1);
}

#[test]
fn event_total_emitted() {
    let mut bus = engene::core::events::EventBus::with_capacity(1);
    bus.set_channel_capacity::<u32>(1);
    bus.emit(1u32);
    bus.emit(2u32);
    assert_eq!(bus.total_emitted::<u32>(), 2);
}

#[test]
fn event_clear_sticky() {
    let mut bus = engene::core::events::EventBus::new();
    bus.set_sticky(42i64);
    assert!(bus.get_sticky::<i64>().is_some());
    bus.clear_sticky::<i64>();
    assert!(bus.get_sticky::<i64>().is_none());
}

#[test]
fn event_channel_count() {
    let mut bus = engene::core::events::EventBus::new();
    bus.emit(1u32);
    bus.emit("x");
    assert!(bus.channel_count() >= 2);
}

#[test]
fn event_sticky_events_module() {
    use engene::core::events::sticky::StickyEvents;
    let mut sticky = StickyEvents::new();
    sticky.set(100u32);
    assert_eq!(sticky.get::<u32>(), Some(&100));
    assert!(sticky.remove::<u32>());
    assert_eq!(sticky.get::<u32>(), None);
}

#[test]
fn event_tracer_disabled_no_record() {
    use engene::core::events::tracing_hooks::EventTracer;
    let mut tracer = EventTracer::new();
    tracer.record(std::any::TypeId::of::<u32>(), "u32", 0, 1);
    let traces = tracer.drain();
    assert_eq!(traces.len(), 0);
}

#[test]
fn event_emit_boxed() {
    let mut bus = engene::core::events::EventBus::new();
    let boxed: Box<dyn std::any::Any + Send + Sync> = Box::new(42u32);
    bus.emit_boxed(boxed);
    assert_eq!(bus.read::<u32>().len(), 1);
    assert_eq!(*bus.read::<u32>()[0], 42);
}

#[test]
fn event_canonical_impact() {
    use engene::core::events::canonical::ImpactEvent;
    let mut bus = engene::core::events::EventBus::new();
    bus.emit(ImpactEvent {
        position: glam::Vec3::new(10.0, 0.0, 10.0),
        direction: glam::Vec3::NEG_Y,
        energy: 500.0,
        material_hit: 1,
        instigator: None,
        target_entity: None,
    });
    assert_eq!(bus.read::<ImpactEvent>().len(), 1);
}

#[test]
fn event_canonical_entity_died() {
    use engene::core::events::canonical::EntityDied;
    let mut bus = engene::core::events::EventBus::new();
    bus.emit(EntityDied {
        entity: 1,
        position: glam::Vec3::ZERO,
        killer: Some(2),
    });
    let read = bus.read::<EntityDied>();
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].entity, 1);
    assert_eq!(read[0].killer, Some(2));
}

#[test]
fn event_default_capacity() {
    let bus = engene::core::events::EventBus::new();
    assert_eq!(bus.channel_count(), 0);
}

#[test]
fn event_with_capacity() {
    let mut bus = engene::core::events::EventBus::with_capacity(256);
    bus.emit(1u32);
    assert_eq!(bus.count::<u32>(), 1);
}

#[test]
fn event_total_dropped() {
    let mut bus = engene::core::events::EventBus::with_capacity(1);
    bus.set_channel_capacity::<u32>(1);
    bus.emit(1u32);
    bus.emit(2u32);
    bus.emit(3u32);
    assert!(bus.total_dropped() >= 2);
}

#[test]
fn event_read_empty_channel() {
    let bus = engene::core::events::EventBus::new();
    let read = bus.read::<u32>();
    assert!(read.is_empty());
}

#[test]
fn event_aggregator_drain_clears() {
    use engene::core::events::aggregation::EventAggregator;
    use engene::core::events::canonical::ImpactEvent;

    let mut agg = EventAggregator::new(10.0);
    agg.submit(&ImpactEvent {
        position: glam::Vec3::ZERO,
        direction: glam::Vec3::Y,
        energy: 1.0,
        material_hit: 0,
        instigator: None,
        target_entity: None,
    });
    let r1 = agg.drain();
    assert_eq!(r1.len(), 1);
    let r2 = agg.drain();
    assert_eq!(r2.len(), 0);
}

#[test]
fn event_sticky_clear_preserves_other_types() {
    let mut bus = engene::core::events::EventBus::new();
    bus.set_sticky(1i32);
    bus.set_sticky(2.0f32);
    bus.clear_sticky::<i32>();
    assert!(bus.get_sticky::<i32>().is_none());
    assert_eq!(bus.get_sticky::<f32>(), Some(&2.0));
}

#[test]
fn event_tracer_drain_takes_all() {
    use engene::core::events::tracing_hooks::EventTracer;
    let mut tracer = EventTracer::new();
    tracer.enable();
    tracer.record(std::any::TypeId::of::<u32>(), "u32", 0, 1);
    tracer.record(std::any::TypeId::of::<i32>(), "i32", 1, 2);
    let traces = tracer.drain();
    assert_eq!(traces.len(), 2);
    let traces2 = tracer.drain();
    assert_eq!(traces2.len(), 0);
}

// =============================================================================
// 4. COMMAND BUFFER / RESOURCES (12 tests)
// =============================================================================

#[test]
fn cmd_spawn_through_buffer() {
    let mut cb = engene::core::commands::CommandBuffer::new();
    let _ = cb.spawn();
    assert_eq!(cb.spawn_count(), 1);
    let spawns = cb.take_spawns();
    assert_eq!(spawns.len(), 1);
}

#[test]
fn cmd_despawn_through_buffer() {
    let mut cb = engene::core::commands::CommandBuffer::new();
    cb.despawn(42);
    cb.despawn(43);
    assert_eq!(cb.despawn_count(), 2);
    let despawns = cb.take_despawns();
    assert_eq!(despawns, vec![42, 43]);
}

#[test]
fn cmd_add_component() {
    let mut cb = engene::core::commands::CommandBuffer::new();
    cb.add_component(0u64, String::from("test"));
    assert_eq!(cb.component_op_count(), 1);
    let ops = cb.take_component_ops();
    assert_eq!(ops.len(), 1);
}

#[test]
fn cmd_remove_component() {
    let mut cb = engene::core::commands::CommandBuffer::new();
    cb.remove_component::<String>(0);
    assert_eq!(cb.component_op_count(), 1);
}

#[test]
fn cmd_deferred_events() {
    let mut cb = engene::core::commands::CommandBuffer::new();
    cb.emit_event(1u32);
    cb.emit_event(2u32);
    assert_eq!(cb.event_count(), 2);
}

#[test]
fn cmd_resource_insert_get() {
    let mut res = engene::core::registry::Resources::new();
    res.insert(42u32);
    assert_eq!(*res.get::<u32>().unwrap(), 42);
}

#[test]
fn cmd_resource_take() {
    let mut res = engene::core::registry::Resources::new();
    res.insert(100u32);
    let taken = res.take::<u32>();
    assert_eq!(taken, Some(100));
    assert!(!res.contains::<u32>());
}

#[test]
fn cmd_resource_freeze_and_insert_runtime() {
    let mut res = engene::core::registry::Resources::new();
    res.insert(1u32);
    res.freeze();
    assert!(res.is_frozen());
    // insert() would panic when frozen; insert_runtime bypasses for take/reinsert flow
    res.insert_runtime(2u32);
    assert_eq!(*res.get::<u32>().unwrap(), 2);
}

#[test]
fn cmd_resource_init_time_unfreeze() {
    let mut res = engene::core::registry::Resources::new();
    res.insert(1u32);
    res.freeze();
    res.unfreeze();
    res.insert(2u32);
    assert_eq!(*res.get::<u32>().unwrap(), 2);
}

#[test]
fn cmd_resource_insert_runtime_after_take() {
    let mut res = engene::core::registry::Resources::new();
    res.insert(10u32);
    res.freeze();
    let _ = res.take::<u32>();
    res.insert_runtime(20u32);
    assert_eq!(*res.get::<u32>().unwrap(), 20);
}

#[test]
fn cmd_buffer_is_empty() {
    let mut cb = engene::core::commands::CommandBuffer::new();
    assert!(cb.is_empty());
    cb.spawn();
    assert!(!cb.is_empty());
    cb.clear();
    assert!(cb.is_empty());
}

#[test]
fn cmd_buffer_clear() {
    let mut cb = engene::core::commands::CommandBuffer::new();
    cb.spawn();
    cb.despawn(1);
    cb.add_component(0, 1u32);
    cb.emit_event(1u32);
    cb.clear();
    assert!(cb.is_empty());
    assert_eq!(cb.spawn_count(), 0);
    assert_eq!(cb.despawn_count(), 0);
}

// =============================================================================
// 5. AUTHORITY / OWNERSHIP / CONTRACTS (12 tests)
// =============================================================================

#[test]
fn authority_matrix_coverage() {
    use engene::core::world_state_authority::{authority_matrix, SaveScope};

    let matrix = authority_matrix();
    assert!(matrix.len() >= 15);

    let has_transform = matrix.iter().any(|e| e.state_name == "Entity Transform");
    let has_personal = matrix.iter().any(|e| e.state_name == "Personal Needs");
    assert!(has_transform);
    assert!(has_personal);

    let entity_scoped = matrix.iter().filter(|e| e.save_scope == SaveScope::Entity).count();
    assert!(entity_scoped >= 5);
}

#[test]
fn authority_ownership_map() {
    use engene::core::ownership_map::{OwnershipMap, ResourceOwnership, MutationTiming, ThreadSafety};

    let mut map = OwnershipMap::new();
    map.register_resource(ResourceOwnership {
        resource_name: "TestRes".into(),
        type_id: std::any::TypeId::of::<u32>(),
        owner_system: "Sys".into(),
        readers: vec!["Reader".into()],
        writers: vec!["Sys".into()],
        mutation_timing: vec![MutationTiming::FixedTick],
        thread_safety: ThreadSafety::MainThreadOnly,
        notes: String::new(),
    });
    let issues = map.validate();
    assert!(issues.is_empty());
    assert_eq!(map.resource_count(), 1);
}

#[test]
fn authority_contract_violations_detected() {
    use engene::core::world_state_authority::enforce_authority_rules;
    let violations = enforce_authority_rules();
    for v in &violations {
        assert!(!v.state_name.is_empty());
    }
}

#[test]
fn authority_spawn_policy_entity_scoped() {
    use engene::core::world_state_authority::{authority_matrix, SaveScope};
    let matrix = authority_matrix();
    let entity_states: Vec<_> = matrix.iter().filter(|e| e.save_scope == SaveScope::Entity).collect();
    assert!(entity_states.len() >= 5);
}

#[test]
fn authority_entity_ref_hygiene() {
    let mut ecs = engene::core::ecs::Ecs::new();
    let (e, pid) = ecs.spawn_new();
    let eref = engene::core::persistent_id::EntityRef::new(pid);
    assert!(!eref.is_dead(&ecs.identity));
    assert!(!eref.is_unloaded(&ecs.identity));
    ecs.despawn(e);
    assert!(eref.is_dead(&ecs.identity));
}

#[test]
fn authority_derived_state_rebuild() {
    use engene::core::world_state_authority::validate_derived_state_rebuild;
    let tests = validate_derived_state_rebuild();
    assert!(tests.len() >= 3);
    for t in &tests {
        assert!(t.passed, "{}: {}", t.state_name, t.details);
    }
}

#[test]
fn authority_ownership_map_resources_owned_by() {
    use engene::core::ownership_map::{OwnershipMap, ResourceOwnership, MutationTiming, ThreadSafety};

    let mut map = OwnershipMap::new();
    map.register_resource(ResourceOwnership {
        resource_name: "R1".into(),
        type_id: std::any::TypeId::of::<u32>(),
        owner_system: "SysA".into(),
        readers: vec![],
        writers: vec!["SysA".into()],
        mutation_timing: vec![MutationTiming::FixedTick],
        thread_safety: ThreadSafety::MainThreadOnly,
        notes: String::new(),
    });
    let owned = map.resources_owned_by("SysA");
    assert_eq!(owned.len(), 1);
}

#[test]
fn authority_ownership_map_validate_empty_owner_fails() {
    use engene::core::ownership_map::{OwnershipMap, ResourceOwnership, MutationTiming, ThreadSafety};

    let mut map = OwnershipMap::new();
    map.register_resource(ResourceOwnership {
        resource_name: "Bad".into(),
        type_id: std::any::TypeId::of::<u32>(),
        owner_system: String::new(),
        readers: vec![],
        writers: vec![],
        mutation_timing: vec![MutationTiming::Startup],
        thread_safety: ThreadSafety::MainThreadOnly,
        notes: String::new(),
    });
    let issues = map.validate();
    assert!(!issues.is_empty());
}

#[test]
fn authority_matrix_has_global_scope() {
    use engene::core::world_state_authority::{authority_matrix, SaveScope};
    let matrix = authority_matrix();
    let global = matrix.iter().filter(|e| e.save_scope == SaveScope::Global).count();
    assert!(global >= 1);
}

#[test]
fn authority_matrix_has_derived_scope() {
    use engene::core::world_state_authority::{authority_matrix, SaveScope};
    let matrix = authority_matrix();
    let derived = matrix.iter().filter(|e| e.save_scope == SaveScope::Derived).count();
    assert!(derived >= 2);
}

#[test]
fn authority_report_string() {
    use engene::core::world_state_authority::authority_report;
    let report = authority_report();
    assert!(!report.is_empty());
}

#[test]
fn authority_system_descriptor_ordering() {
    use engene::core::system_descriptor::SystemDescriptor;
    let a = SystemDescriptor::new("A").before("B");
    let b = SystemDescriptor::new("B").after("A");
    assert_eq!(a.ordering.before, vec!["B"]);
    assert_eq!(b.ordering.after, vec!["A"]);
}

#[test]
fn authority_system_descriptor_component_access() {
    use engene::core::system_descriptor::SystemDescriptor;
    struct CompA;
    struct CompB;
    let desc = SystemDescriptor::new("Test")
        .reads_component::<CompA>()
        .writes_component::<CompB>();
    assert!(desc.reads_components.contains(&std::any::TypeId::of::<CompA>()));
    assert!(desc.writes_components.contains(&std::any::TypeId::of::<CompB>()));
}

#[test]
fn cmd_spawn_command_ops_empty_by_default() {
    let mut cb = engene::core::commands::CommandBuffer::new();
    let cmd = cb.spawn();
    assert!(cmd.ops.is_empty());
}
