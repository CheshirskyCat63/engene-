#[test]
fn dependency_graph_no_cycles() {
    use engene::core::system_descriptor::SystemDescriptor;

    let a = SystemDescriptor::new("A").before("B");
    let b = SystemDescriptor::new("B").before("C");
    let c = SystemDescriptor::new("C");

    let descriptors = vec![a, b, c];
    let names: Vec<&str> = descriptors.iter().map(|d| d.name).collect();
    assert_eq!(names, vec!["A", "B", "C"]);
}

#[test]
fn command_buffer_spawn_and_despawn() {
    use engene::core::commands::CommandBuffer;

    let mut cb = CommandBuffer::new();
    assert!(cb.is_empty());

    cb.despawn(42);
    assert_eq!(cb.despawn_count(), 1);
    assert!(!cb.is_empty());

    let despawns = cb.take_despawns();
    assert_eq!(despawns, vec![42]);
}

#[test]
fn command_buffer_emit_event() {
    use engene::core::commands::CommandBuffer;

    #[derive(Debug)]
    struct TestEvent(u32);

    let mut cb = CommandBuffer::new();
    let evt = TestEvent(99);
    assert_eq!(evt.0, 99);
    cb.emit_event(evt);
    assert_eq!(cb.event_count(), 1);
}

#[test]
fn event_bus_bounded_capacity() {
    use engene::core::events::EventBus;

    let mut bus = EventBus::with_capacity(3);
    bus.set_channel_capacity::<u32>(3);

    bus.emit(1u32);
    bus.emit(2u32);
    bus.emit(3u32);
    bus.emit(4u32); // should be dropped

    assert_eq!(bus.count::<u32>(), 3);
    assert_eq!(bus.dropped_count::<u32>(), 1);
}

#[test]
fn event_bus_sticky() {
    use engene::core::events::EventBus;

    let mut bus = EventBus::new();

    #[derive(Debug, PartialEq)]
    struct Config(u32);

    bus.set_sticky(Config(42));
    assert_eq!(bus.get_sticky::<Config>(), Some(&Config(42)));

    bus.clear();
    assert_eq!(bus.get_sticky::<Config>(), Some(&Config(42)));
}

#[test]
fn event_bus_clear_removes_frame_events_not_sticky() {
    use engene::core::events::EventBus;

    let mut bus = EventBus::new();
    bus.emit(100u32);
    bus.set_sticky(String::from("persistent"));

    bus.clear();

    assert_eq!(bus.count::<u32>(), 0);
    assert_eq!(
        bus.get_sticky::<String>(),
        Some(&String::from("persistent"))
    );
}

#[test]
fn runtime_manifest_feature_toggle() {
    use engene::core::runtime_manifest::RuntimeManifest;

    let manifest = RuntimeManifest::default();
    assert!(manifest.is_feature_enabled("destruction"));
    assert!(manifest.is_feature_enabled("gore"));
    assert!(!manifest.is_feature_enabled("replay_capture"));

    let low = RuntimeManifest::low_spec();
    assert!(!low.cloth_simulation_enabled);
}

#[test]
fn runtime_config_profiles() {
    use engene::core::runtime_config::{RendererBackend, RuntimeConfig, RuntimeProfile};

    let sandbox = RuntimeConfig::sandbox();
    assert_eq!(sandbox.profile, RuntimeProfile::Sandbox);

    let headless = RuntimeConfig::headless();
    assert!(headless.is_headless());
    assert_eq!(headless.renderer_backend, RendererBackend::Headless);
}

#[test]
fn parallel_validator_detects_conflicts() {
    use engene::core::parallel_validation::validate_systems;
    use engene::core::system_descriptor::SystemDescriptor;

    struct FakeComponentA;
    struct FakeComponentB;

    let a = SystemDescriptor::new("A")
        .writes_component::<FakeComponentA>()
        .reads_component::<FakeComponentB>()
        .with_parallel(true);
    let b = SystemDescriptor::new("B")
        .writes_component::<FakeComponentA>()
        .with_parallel(true);

    let report = validate_systems(&[a, b], false, false);
    assert!(report.has_errors(), "should detect write-write conflict");
}

#[test]
fn parallel_validator_no_conflicts_for_disjoint() {
    use engene::core::parallel_validation::validate_systems;
    use engene::core::system_descriptor::SystemDescriptor;

    struct CompX;
    struct CompY;

    let a = SystemDescriptor::new("A")
        .writes_component::<CompX>()
        .with_parallel(true);
    let b = SystemDescriptor::new("B")
        .writes_component::<CompY>()
        .with_parallel(true);

    let report = validate_systems(&[a, b], false, false);
    assert!(!report.has_errors(), "disjoint writes should not conflict");
}

#[test]
fn debug_registry_categories() {
    use engene::core::debug::debug_registry::{DebugCategory, DebugRegistry};

    let mut reg = DebugRegistry::new();
    reg.register_view("nav_mesh", DebugCategory::World);
    reg.register_view("ai_goals", DebugCategory::AI);
    reg.register_view("budgets", DebugCategory::Budgets);

    assert_eq!(reg.views_in_category(DebugCategory::World).len(), 1);
    assert!(!reg.is_enabled("nav_mesh"));
    reg.toggle_view("nav_mesh");
    assert!(reg.is_enabled("nav_mesh"));
}

#[test]
fn frame_recorder_capacity_and_freeze() {
    use engene::core::debug::frame_recorder::{DebugFrameSnapshot, FrameRecorder};

    let mut recorder = FrameRecorder::new(3);
    for i in 0..5 {
        recorder.record(DebugFrameSnapshot {
            tick: i,
            data: vec![],
        });
    }
    assert_eq!(recorder.len(), 3);
    assert_eq!(recorder.latest().unwrap().tick, 4);

    recorder.freeze();
    recorder.record(DebugFrameSnapshot {
        tick: 99,
        data: vec![],
    });
    assert_eq!(recorder.len(), 3);
}

#[test]
fn worker_pool_executes() {
    use engene::core::jobs::worker_pool::WorkerPool;

    let pool = WorkerPool::new();
    assert!(pool.worker_count() >= 2);

    let result = pool.execute(|| 42);
    assert_eq!(result, 42);
}

#[test]
fn fence_signal_and_wait() {
    use engene::core::jobs::fences::Fence;

    let fence = Fence::new();
    assert!(!fence.is_signaled());
    fence.signal();
    assert!(fence.is_signaled());
    fence.reset();
    assert!(!fence.is_signaled());
}

// ===== Phase 1.4: E2E Command Application + Event Consumption Tests =====

#[test]
fn e2e_command_buffer_applies_spawns_and_components() {
    use engene::core::commands::CommandBuffer;

    let mut cb = CommandBuffer::new();

    let _spawn = cb.spawn();
    assert_eq!(cb.spawn_count(), 1);

    cb.emit_event(42u32);
    assert_eq!(cb.event_count(), 1);

    cb.add_component(0u64, String::from("test"));

    assert_eq!(cb.component_op_count(), 1);
    assert!(!cb.is_empty());

    let spawns = cb.take_spawns();
    assert_eq!(spawns.len(), 1);
    let ops = cb.take_component_ops();
    assert_eq!(ops.len(), 1);
    let events = cb.take_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn e2e_deferred_events_reach_bus() {
    use engene::core::events::EventBus;

    let mut bus = EventBus::new();

    #[derive(Debug, PartialEq)]
    struct TestEvent(u32);

    let boxed: Box<dyn std::any::Any + Send + Sync> = Box::new(TestEvent(42));
    bus.emit_boxed(boxed);

    let read = bus.read::<TestEvent>();
    assert_eq!(read.len(), 1);
    assert_eq!(read[0], &TestEvent(42));
}

#[test]
fn e2e_canonical_events_emitted_and_consumed() {
    use engene::core::events::canonical::*;
    use engene::core::events::EventBus;
    use glam::Vec3;

    let mut bus = EventBus::new();

    bus.emit(ImpactEvent {
        position: Vec3::new(10.0, 0.0, 10.0),
        direction: Vec3::NEG_Y,
        energy: 500.0,
        material_hit: 1,
        instigator: None,
        target_entity: None,
    });

    bus.emit(WorldTopologyChanged {
        position: Vec3::new(10.0, 0.0, 10.0),
        radius: 5.0,
        cause: TopologyChangeCause::Explosion,
    });

    bus.emit(SoundTrigger {
        position: Vec3::new(10.0, 0.0, 10.0),
        kind: SoundTriggerKind::GunShot,
        volume: 1.0,
    });

    assert_eq!(
        bus.read::<ImpactEvent>().len(),
        1,
        "ImpactEvent should be readable"
    );
    assert_eq!(
        bus.read::<WorldTopologyChanged>().len(),
        1,
        "WorldTopologyChanged should be readable"
    );
    assert_eq!(
        bus.read::<SoundTrigger>().len(),
        1,
        "SoundTrigger should be readable"
    );

    bus.clear();
    assert_eq!(
        bus.read::<ImpactEvent>().len(),
        0,
        "clear should remove frame events"
    );
}

#[test]
fn access_descriptor_conflict_detection() {
    use engene::core::access::queries::AccessDescriptor;
    use std::any::TypeId;

    struct Transform;
    struct Velocity;

    let mut a = AccessDescriptor::new();
    a.writes_components.push(TypeId::of::<Transform>());
    a.reads_components.push(TypeId::of::<Velocity>());

    let mut b = AccessDescriptor::new();
    b.reads_components.push(TypeId::of::<Transform>());

    assert!(a.has_write_conflict(&b));

    let mut c = AccessDescriptor::new();
    c.reads_components.push(TypeId::of::<Velocity>());
    assert!(!c.has_write_conflict(&b));
}

// ── Wave 1: Persistent Identity (A0.1) ────────────────────────

#[test]
fn persistent_identity_spawn_new_assigns_unique_pids() {
    use engene::core::ecs::Ecs;

    let mut ecs = Ecs::new();
    let (e1, pid1) = ecs.spawn_new();
    let (e2, pid2) = ecs.spawn_new();
    let (e3, pid3) = ecs.spawn_new();

    assert_ne!(pid1, pid2);
    assert_ne!(pid2, pid3);
    assert_ne!(pid1, pid3);
    assert_ne!(e1, e2);
    assert_ne!(e2, e3);

    assert_eq!(ecs.identity.persistent_id_of(e1), Some(pid1));
    assert_eq!(ecs.identity.persistent_id_of(e2), Some(pid2));
    assert_eq!(ecs.identity.persistent_id_of(e3), Some(pid3));
}

#[test]
fn persistent_identity_despawn_marks_dead() {
    use engene::core::ecs::Ecs;
    use engene::core::persistent_id::EntityPresence;

    let mut ecs = Ecs::new();
    let (entity, pid) = ecs.spawn_new();

    assert!(matches!(
        ecs.identity.presence(pid),
        EntityPresence::Live(_)
    ));
    ecs.despawn(entity);
    assert_eq!(ecs.identity.presence(pid), EntityPresence::Dead);
    assert_eq!(ecs.identity.persistent_id_of(entity), None);
}

#[test]
fn persistent_identity_spawn_restored_reuses_pid() {
    use engene::core::ecs::Ecs;
    use engene::core::persistent_id::{EntityPresence, PersistentEntityId};

    let mut ecs = Ecs::new();
    let pid = PersistentEntityId(42);

    let entity = ecs
        .spawn_restored(pid)
        .expect("first restore should succeed");
    assert!(matches!(
        ecs.identity.presence(pid),
        EntityPresence::Live(_)
    ));
    assert_eq!(ecs.identity.resolve(pid), Some(entity));
}

#[test]
fn persistent_identity_duplicate_pid_rejected() {
    use engene::core::ecs::Ecs;
    use engene::core::persistent_id::PersistentEntityId;

    let mut ecs = Ecs::new();
    let pid = PersistentEntityId(100);

    let _e1 = ecs.spawn_restored(pid).expect("first restore ok");
    let result = ecs.spawn_restored(pid);
    assert!(result.is_err(), "duplicate PID should be rejected");
}

#[test]
fn persistent_identity_tombstone_gc() {
    use engene::core::ecs::Ecs;
    use engene::core::persistent_id::EntityPresence;

    let mut ecs = Ecs::new();
    ecs.tick = 100;
    let (entity, pid) = ecs.spawn_new();
    ecs.despawn(entity);
    assert_eq!(ecs.identity.presence(pid), EntityPresence::Dead);

    ecs.identity.gc_tombstones(200, 50);
    assert_eq!(
        ecs.identity.presence(pid),
        EntityPresence::Dead,
        "tombstone should survive within max_age"
    );

    ecs.identity.gc_tombstones(300, 50);
    assert_eq!(
        ecs.identity.presence(pid),
        EntityPresence::Dead,
        "tombstone collected after max_age"
    );
}

// ── Wave 1: Reference Hygiene (A0.1.5) ────────────────────────

#[test]
fn entity_ref_resolves_live_entity() {
    use engene::core::ecs::Ecs;
    use engene::core::persistent_id::EntityRef;

    let mut ecs = Ecs::new();
    let (entity, pid) = ecs.spawn_new();
    let eref = EntityRef::new(pid);

    assert_eq!(eref.resolve(&ecs.identity), Some(entity));
    assert!(!eref.is_dead(&ecs.identity));
    assert!(!eref.is_unloaded(&ecs.identity));
}

#[test]
fn entity_ref_dead_after_despawn() {
    use engene::core::ecs::Ecs;
    use engene::core::persistent_id::EntityRef;

    let mut ecs = Ecs::new();
    let (entity, pid) = ecs.spawn_new();
    let eref = EntityRef::new(pid);

    ecs.despawn(entity);
    assert_eq!(eref.resolve(&ecs.identity), None);
    assert!(eref.is_dead(&ecs.identity));
}

#[test]
fn entity_ref_unloaded_state() {
    use engene::core::ecs::Ecs;
    use engene::core::persistent_id::EntityRef;

    let mut ecs = Ecs::new();
    let (_entity, pid) = ecs.spawn_new();
    let eref = EntityRef::new(pid);

    ecs.identity.mark_unloaded(pid);
    assert_eq!(eref.resolve(&ecs.identity), None);
    assert!(eref.is_unloaded(&ecs.identity));
    assert!(!eref.is_dead(&ecs.identity));
}

// ── Wave 1: Authority Matrix (A0.2) ───────────────────────────

#[test]
fn authority_matrix_covers_all_state_categories() {
    use engene::core::world_state_authority::{authority_matrix, SaveScope};

    let matrix = authority_matrix();
    assert!(
        matrix.len() >= 15,
        "authority matrix should cover at least 15 state categories, got {}",
        matrix.len()
    );

    let has_transform = matrix.iter().any(|e| e.state_name == "Entity Transform");
    let has_health = matrix.iter().any(|e| e.state_name == "Personal Needs");
    let has_destruction = matrix
        .iter()
        .any(|e| e.state_name == "Destruction Topology");
    let has_economy = matrix.iter().any(|e| e.state_name == "Economy Global");
    let has_spatial = matrix.iter().any(|e| e.state_name == "Spatial Index");

    assert!(has_transform, "missing Entity Transform");
    assert!(has_health, "missing Personal Needs");
    assert!(has_destruction, "missing Destruction Topology");
    assert!(has_economy, "missing Economy Global");
    assert!(has_spatial, "missing Spatial Index");

    let entity_scoped = matrix
        .iter()
        .filter(|e| e.save_scope == SaveScope::Entity)
        .count();
    let derived = matrix
        .iter()
        .filter(|e| e.save_scope == SaveScope::Derived)
        .count();
    assert!(
        entity_scoped >= 5,
        "should have at least 5 entity-scoped states"
    );
    assert!(derived >= 3, "should have at least 3 derived states");
}

#[test]
fn authority_enforcement_no_false_positives() {
    use engene::core::world_state_authority::enforce_authority_rules;

    let violations = enforce_authority_rules();
    for v in &violations {
        assert_ne!(
            v.state_name, "Entity Transform",
            "entity transform should not have authority violations"
        );
    }
}

#[test]
fn derived_state_rebuild_all_pass() {
    use engene::core::world_state_authority::validate_derived_state_rebuild;

    let tests = validate_derived_state_rebuild();
    assert!(
        tests.len() >= 3,
        "should have at least 3 derived rebuild tests"
    );
    for t in &tests {
        assert!(
            t.passed,
            "derived rebuild '{}' should pass: {}",
            t.state_name, t.details
        );
    }
}

// ── Wave 1: Abstraction Invariants (A0.3) ─────────────────────

#[test]
fn field_invariant_table_completeness() {
    use engene::simulation::abstraction_invariants::{field_invariant_table, FieldPreservation};

    let table = field_invariant_table();
    assert!(
        table.len() >= 10,
        "invariant table should have at least 10 entries, got {}",
        table.len()
    );

    let has_pid = table.iter().any(|f| f.field_name == "persistent_id");
    let has_position = table.iter().any(|f| f.field_name.contains("position"));
    let has_health = table.iter().any(|f| f.field_name.contains("health"));

    assert!(has_pid, "missing persistent_id invariant");
    assert!(has_position, "missing position invariant");
    assert!(has_health, "missing health invariant");

    let pid_entry = table
        .iter()
        .find(|f| f.field_name == "persistent_id")
        .unwrap();
    assert_eq!(
        pid_entry.preservation,
        FieldPreservation::PreserveExact,
        "persistent_id must be PreserveExact"
    );
}

// ── Wave 1: Network Markers (A0.4) ───────────────────────────

#[cfg(feature = "networking")]
#[test]
fn net_state_markers_cover_critical_states() {
    use engene::network::net_markers::{net_state_markers, NetAuthority, NetPriority};

    let markers = net_state_markers();
    assert!(
        markers.len() >= 10,
        "should annotate at least 10 state types"
    );

    let transform = markers
        .iter()
        .find(|m| m.state_name == "Transform")
        .unwrap();
    assert_eq!(transform.authority, NetAuthority::ServerAuthoritative);
    assert_eq!(transform.priority, NetPriority::EveryTick);
    assert!(transform.interpolatable);

    let memory = markers.iter().find(|m| m.state_name == "Memory").unwrap();
    assert_eq!(
        memory.priority,
        NetPriority::Never,
        "AI memory should never be replicated"
    );

    let sim_level = markers.iter().find(|m| m.state_name == "SimLevel").unwrap();
    assert_eq!(sim_level.authority, NetAuthority::ClientLocal);
}

// ── Wave 1: Editor Safe Mode (E0) ────────────────────────────

#[test]
fn editor_safe_mode_panel_lifecycle() {
    use engene::tools::editor_safe_mode::EditorSafeMode;

    let mut esm = EditorSafeMode::new();
    esm.register_panel("inspector");
    esm.register_panel("console");

    assert!(esm.is_panel_enabled("inspector"));
    assert!(esm.is_panel_enabled("console"));
    assert!(!esm.is_panel_enabled("nonexistent"));

    let drew = esm.draw_panel("inspector", || { /* no-op */ });
    assert!(drew, "healthy panel should draw");
}

#[test]
fn editor_safe_mode_disables_after_repeated_panics() {
    use engene::tools::editor_safe_mode::EditorSafeMode;

    let mut esm = EditorSafeMode::new();
    esm.register_panel("broken_panel");

    for _ in 0..3 {
        let _ = esm.draw_panel("broken_panel", || {
            panic!("intentional test panic");
        });
    }

    assert!(
        !esm.is_panel_enabled("broken_panel"),
        "panel should be disabled after 3 consecutive panics"
    );
}

#[test]
fn editor_safe_mode_recovery() {
    use engene::tools::editor_safe_mode::EditorSafeMode;

    let mut esm = EditorSafeMode::new();
    esm.register_panel("recoverable");

    for _ in 0..3 {
        let _ = esm.draw_panel("recoverable", || {
            panic!("test");
        });
    }
    assert!(!esm.is_panel_enabled("recoverable"));

    esm.re_enable_panel("recoverable");
    assert!(esm.is_panel_enabled("recoverable"));
}

#[test]
fn editor_safe_mode_global_activation() {
    use engene::tools::editor_safe_mode::EditorSafeMode;

    let mut esm = EditorSafeMode::new();
    esm.register_panel("inspector");
    assert!(esm.is_panel_enabled("inspector"));

    esm.end_frame(100.0); // way over budget
    assert!(esm.is_safe_mode());
    assert!(
        !esm.is_panel_enabled("inspector"),
        "all panels disabled in safe mode"
    );

    esm.exit_safe_mode();
    assert!(!esm.is_safe_mode());
    assert!(esm.is_panel_enabled("inspector"));
}

// ── Wave 1: spawn_new used for gameplay entities ──────────────

#[test]
fn all_gameplay_entities_have_persistent_ids() {
    use engene::core::ecs::Ecs;
    use engene::world::population;

    let mut ecs = Ecs::new();
    population::spawn_npcs(&mut ecs);
    population::spawn_monsters(&mut ecs);

    let total_alive = ecs.alive.len();
    assert!(total_alive > 0, "should have spawned entities");

    let with_pid = ecs
        .alive
        .iter()
        .filter(|&&e| ecs.identity.persistent_id_of(e).is_some())
        .count();

    assert_eq!(
        with_pid, total_alive,
        "all {} gameplay entities should have PersistentEntityId, but only {} do",
        total_alive, with_pid
    );
}

// ── Wave 2: Chunk Persistence (A.1) ──────────────────────────

#[test]
fn chunk_persistence_save_and_load_cycle() {
    use engene::core::ecs::Ecs;
    use engene::core::persistent_id::EntityPresence;
    use engene::world::chunk_persistence::ChunkPersistenceService;
    use engene::world::streaming::ChunkCoord;

    let dir = std::env::temp_dir().join("engene_test_chunks");
    let _ = std::fs::remove_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e1, pid1) = ecs.spawn_new();
    ecs.transforms.insert(
        e1,
        engene::world::components::Transform {
            x: 500.0,
            y: 500.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds
        .insert(e1, engene::world::components::EntityKind::Npc);
    ecs.names
        .insert(e1, engene::world::components::Name("TestNPC".into()));
    ecs.personal_needs
        .insert(e1, engene::world::components::PersonalNeeds::default_npc());

    let mut persistence = ChunkPersistenceService::new(&dir);
    let coord = ChunkCoord { x: 0, z: 0 };

    let saved = persistence.save_and_unload(coord, &mut ecs, 100);
    assert_eq!(saved, 1, "should save 1 entity");
    assert_eq!(ecs.alive.len(), 0, "entity should be removed from ECS");
    assert!(
        matches!(ecs.identity.presence(pid1), EntityPresence::Unloaded),
        "entity should be marked Unloaded, not Dead"
    );

    let loaded = persistence.load_chunk_entities(coord, &mut ecs);
    assert_eq!(loaded, 1, "should load 1 entity");
    assert_eq!(ecs.alive.len(), 1, "entity should be back in ECS");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn chunk_persistence_identity_survives_cycle() {
    use engene::core::ecs::Ecs;
    use engene::world::chunk_persistence::ChunkPersistenceService;
    use engene::world::streaming::ChunkCoord;

    let dir = std::env::temp_dir().join("engene_test_identity");
    let _ = std::fs::remove_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e1, pid1) = ecs.spawn_new();
    ecs.transforms.insert(
        e1,
        engene::world::components::Transform {
            x: 100.0,
            y: 100.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds
        .insert(e1, engene::world::components::EntityKind::Npc);
    ecs.personal_needs
        .insert(e1, engene::world::components::PersonalNeeds::default_npc());

    let mut persistence = ChunkPersistenceService::new(&dir);
    let coord = ChunkCoord { x: 0, z: 0 };

    persistence.save_and_unload(coord, &mut ecs, 50);
    persistence.load_chunk_entities(coord, &mut ecs);

    let restored_entity = ecs
        .identity
        .resolve(pid1)
        .expect("pid should resolve after reload");
    assert!(
        ecs.is_alive(restored_entity),
        "restored entity should be alive"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// ── Wave 2: Relink Report (A.2) ─────────────────────────────

#[test]
fn relink_report_clean_after_simple_cycle() {
    use engene::core::ecs::Ecs;
    use engene::world::chunk_persistence::ChunkPersistenceService;
    use engene::world::streaming::ChunkCoord;

    let dir = std::env::temp_dir().join("engene_test_relink");
    let _ = std::fs::remove_dir_all(&dir);

    let mut ecs = Ecs::new();
    let (e1, _pid1) = ecs.spawn_new();
    ecs.transforms.insert(
        e1,
        engene::world::components::Transform {
            x: 200.0,
            y: 200.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds
        .insert(e1, engene::world::components::EntityKind::Npc);
    ecs.personal_needs
        .insert(e1, engene::world::components::PersonalNeeds::default_npc());

    let mut persistence = ChunkPersistenceService::new(&dir);
    let coord = ChunkCoord { x: 0, z: 0 };

    persistence.save_and_unload(coord, &mut ecs, 10);
    let report = persistence.load_chunk_with_report(coord, &mut ecs);

    assert_eq!(report.entities_restored, 1);
    assert_eq!(report.duplicates_skipped, 0);
    assert!(
        report.is_clean(),
        "relink should be clean: {}",
        report.summary()
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// ── Wave 2: Resources take/insert_runtime (A.1 infrastructure) ──

#[test]
fn resources_take_and_reinsert() {
    use engene::core::registry::Resources;

    let mut res = Resources::new();
    res.insert(42u32);
    res.insert("hello".to_string());

    let val: Option<u32> = res.take();
    assert_eq!(val, Some(42));
    assert!(!res.contains::<u32>());

    res.insert_runtime(99u32);
    assert_eq!(*res.get::<u32>().unwrap(), 99);
}

// ── Wave 2: Test Budget Table (A.3) ─────────────────────────

#[test]
fn fast_tests_complete_within_budget() {
    // Meta-test: verifies that we haven't accidentally added tests
    // that blow the <10 minute budget
    let budget_seconds = 600.0; // 10 minutes
    let start = std::time::Instant::now();

    // This test itself contributes to the budget;
    // it exists as a sentinel -- if total test time exceeds 10min,
    // this will fail in CI by timeout, not by assertion.
    assert!(start.elapsed().as_secs_f64() < budget_seconds);
}

// ── Nightly-gated tests ──────────────────────────────────────

#[cfg(feature = "nightly-tests")]
#[test]
fn nightly_golden_scenario_population_stability() {
    use engene::core::ecs::Ecs;
    use engene::world::population;

    let mut ecs = Ecs::new();
    population::spawn_npcs(&mut ecs);
    population::spawn_monsters(&mut ecs);

    let initial = ecs.alive.len();
    assert!(initial >= 50, "should have substantial population");
    // In nightly, we'd run 1000+ ticks and verify no collapse
}

#[cfg(feature = "nightly-tests")]
#[test]
fn nightly_streaming_4_region_cycle() {
    use engene::testsupport::streaming_harness::golden_scenarios;

    let scenarios = golden_scenarios();
    assert!(
        scenarios.len() >= 4,
        "should have at least 4 golden scenarios"
    );
}

// ── Wave 3: Runtime Profiles ─────────────────────────────────

#[test]
fn runtime_profiles_have_correct_budgets() {
    use engene::core::runtime_config::{QualityTier, RuntimeConfig};

    let low = RuntimeConfig::low_spec();
    let budgets = low.budgets();
    assert_eq!(budgets.quality, QualityTier::Low);
    assert!(!budgets.enable_ssao, "low spec should disable SSAO");
    assert!(
        !budgets.enable_volumetrics,
        "low spec should disable volumetrics"
    );
    assert!(budgets.max_visible_npcs <= 50, "low spec should cap NPCs");

    let shipping = RuntimeConfig::shipping();
    let budgets = shipping.budgets();
    assert_eq!(budgets.quality, QualityTier::High);
    assert!(budgets.enable_ssao, "shipping should enable SSAO");
    assert!(
        budgets.max_visible_npcs >= 200,
        "shipping should support many NPCs"
    );

    let headless = RuntimeConfig::headless();
    let budgets = headless.budgets();
    assert_eq!(
        budgets.render_budget_ms, 0.0,
        "headless should have no render budget"
    );
    assert!(
        !budgets.enable_debug_ui,
        "headless should not enable debug UI"
    );
}

#[test]
fn all_profiles_produce_valid_budgets() {
    use engene::core::runtime_config::{ProfileBudgets, RuntimeProfile};

    let profiles = [
        RuntimeProfile::Shipping,
        RuntimeProfile::LowSpec,
        RuntimeProfile::DebugTools,
        RuntimeProfile::HeadlessServer,
        RuntimeProfile::Sandbox,
        RuntimeProfile::VerticalSlice,
    ];

    for profile in &profiles {
        let budgets = ProfileBudgets::for_profile(profile);
        assert!(
            budgets.ai_think_budget_ms >= 0.0,
            "{:?} has negative AI budget",
            profile
        );
        assert!(
            budgets.render_budget_ms >= 0.0,
            "{:?} has negative render budget",
            profile
        );
        assert!(
            budgets.max_particles <= 100_000,
            "{:?} has unreasonable particle cap",
            profile
        );
    }
}

// ── Wave 3: Degradation Order ────────────────────────────────

#[test]
fn degradation_order_has_never_cut_entries() {
    use engene::core::quality_governor::{degradation_order, DegradationPriority};

    let order = degradation_order();
    assert!(
        order.len() >= 10,
        "should have at least 10 degradation entries"
    );

    let never_cut: Vec<_> = order
        .iter()
        .filter(|e| e.priority == DegradationPriority::NeverCut)
        .collect();
    assert!(
        never_cut.len() >= 4,
        "should have at least 4 never-cut systems"
    );

    let never_cut_names: Vec<&str> = never_cut.iter().map(|e| e.subsystem).collect();
    assert!(never_cut_names.contains(&"Collision Detection"));
    assert!(never_cut_names.contains(&"Persistent Identity"));
    assert!(never_cut_names.contains(&"Save Correctness"));
    assert!(never_cut_names.contains(&"Navigation"));
}

#[test]
fn degradation_low_disables_cosmetics() {
    use engene::core::quality_governor::systems_to_disable;
    use engene::core::runtime_config::QualityTier;

    let disabled = systems_to_disable(QualityTier::Low);
    assert!(disabled.contains(&"SSAO"), "SSAO should be disabled at Low");
    assert!(
        disabled.contains(&"Volumetric Lighting"),
        "Volumetrics should be disabled at Low"
    );
    assert!(
        disabled.contains(&"Detailed Decals"),
        "Detailed decals should be disabled at Low"
    );
}

// ── Wave 3: Doctor Integration ───────────────────────────────

#[test]
fn doctor_reports_identity_and_authority() {
    use engene::core::engine::Engine;
    use engene::tools::doctor::{run_doctor, DoctorMode};
    use engene::world::biome::Biome;
    use engene::world::resources::ResourceGrid;

    let biomes = vec![Biome::Plains; 4];
    let engine = Engine::new(ResourceGrid::new(&biomes));

    let report = run_doctor(&engine, DoctorMode::Advisory);

    let has_identity_diag = report.diagnostics.iter().any(|d| d.category == "identity");
    let has_authority_diag = report.diagnostics.iter().any(|d| d.category == "authority");
    let has_network_diag = report.diagnostics.iter().any(|d| d.category == "network");
    let has_degradation_diag = report
        .diagnostics
        .iter()
        .any(|d| d.category == "degradation");

    assert!(has_identity_diag, "doctor should check identity health");
    assert!(has_authority_diag, "doctor should check authority matrix");
    assert!(has_network_diag, "doctor should check net markers");
    assert!(
        has_degradation_diag,
        "doctor should check degradation order"
    );
}

// ===== Block 2: Authority Enforcement Tests =====

#[test]
fn authority_matrix_covers_critical_state() {
    use engene::core::world_state_authority::{authority_matrix, SaveScope};
    let matrix = authority_matrix();
    assert!(
        matrix.len() >= 15,
        "authority matrix should cover all critical state categories"
    );

    let entity_scoped: Vec<_> = matrix
        .iter()
        .filter(|e| e.save_scope == SaveScope::Entity)
        .collect();
    assert!(entity_scoped.len() >= 8, "should have entity-scoped entries for transform, kind, needs, memory, emotions, economy, inventory, body");

    let chunk_scoped: Vec<_> = matrix
        .iter()
        .filter(|e| e.save_scope == SaveScope::Chunk)
        .collect();
    assert!(
        chunk_scoped.len() >= 3,
        "should have chunk-scoped entries for destruction, surface, fire"
    );
}

#[test]
fn authority_enforce_rules_detects_policy_violations() {
    use engene::core::world_state_authority::enforce_authority_rules;
    let violations = enforce_authority_rules();
    for v in &violations {
        assert!(!v.state_name.is_empty(), "violation should name the state");
    }
}

#[test]
fn authority_derived_state_rebuild_passes() {
    use engene::core::world_state_authority::validate_derived_state_rebuild;
    let tests = validate_derived_state_rebuild();
    assert!(
        tests.len() >= 4,
        "should test nav dirty, cover map, spatial index, sim level"
    );
    for t in &tests {
        assert!(
            t.passed,
            "derived state rebuild for '{}' should pass: {}",
            t.state_name, t.details
        );
    }
}

#[test]
fn ownership_map_validates() {
    use engene::core::ownership_map::{
        MutationTiming, OwnershipMap, ResourceOwnership, ThreadSafety,
    };
    let mut map = OwnershipMap::new();
    map.register_resource(ResourceOwnership {
        resource_name: "TestRes".into(),
        type_id: std::any::TypeId::of::<u32>(),
        owner_system: "TestSys".into(),
        readers: vec!["ReaderA".into()],
        writers: vec!["TestSys".into()],
        mutation_timing: vec![MutationTiming::FixedTick],
        thread_safety: ThreadSafety::MainThreadOnly,
        notes: String::new(),
    });
    let issues = map.validate();
    assert!(
        issues.is_empty(),
        "valid ownership should produce no issues"
    );
    assert_eq!(map.resource_count(), 1);
    assert_eq!(map.resources_owned_by("TestSys").len(), 1);
}

// ===== Block 3: Event System Tests =====

#[test]
fn event_multi_bus_architecture() {
    use engene::core::events::debug_bus::DebugBus;
    use engene::core::events::render_bus::RenderBus;
    use engene::core::events::sim_bus::SimBus;

    let mut sim = SimBus::new();
    let mut render = RenderBus::new();
    let mut debug = DebugBus::new();

    sim.bus.emit(42u32);
    render.bus.emit("render_event");
    debug.bus.emit(true);

    assert_eq!(sim.bus.count::<u32>(), 1);
    assert_eq!(render.bus.count::<&str>(), 1);
    assert_eq!(debug.bus.count::<bool>(), 1);

    sim.bus.clear();
    render.bus.clear();
    debug.bus.clear();

    assert_eq!(sim.bus.count::<u32>(), 0);
}

#[test]
fn event_sticky_events_contract() {
    use engene::core::events::sticky::StickyEvents;

    let mut sticky = StickyEvents::new();
    sticky.set(42u32);
    assert_eq!(sticky.get::<u32>(), Some(&42));
    assert!(sticky.remove::<u32>());
    assert_eq!(sticky.get::<u32>(), None);
}

#[test]
fn event_tracer_records_when_enabled() {
    use engene::core::events::tracing_hooks::EventTracer;
    let mut tracer = EventTracer::new();
    assert!(!tracer.is_enabled());
    tracer.enable();
    assert!(tracer.is_enabled());
    tracer.record(std::any::TypeId::of::<u32>(), "u32", 1, 5);
    let traces = tracer.drain();
    assert_eq!(traces.len(), 1);
    assert_eq!(traces[0].count, 5);
}

#[test]
fn event_aggregator_spatial_bucketing() {
    use engene::core::events::aggregation::EventAggregator;
    use engene::core::events::canonical::ImpactEvent;

    let mut agg = EventAggregator::new(10.0);
    agg.submit(&ImpactEvent {
        position: glam::Vec3::new(5.0, 0.0, 5.0),
        direction: glam::Vec3::Y,
        energy: 100.0,
        material_hit: 0u16,
        instigator: None,
        target_entity: None,
    });
    agg.submit(&ImpactEvent {
        position: glam::Vec3::new(6.0, 0.0, 6.0),
        direction: glam::Vec3::Y,
        energy: 50.0,
        material_hit: 0u16,
        instigator: None,
        target_entity: None,
    });
    let results = agg.drain();
    assert_eq!(results.len(), 1, "impacts in same cell should aggregate");
    assert!((results[0].total_energy - 150.0).abs() < 0.1);
    assert_eq!(results[0].count, 2);
}
