#[test]
fn determinism_audit_detects_headless_issue() {
    use engene::core::determinism_audit::audit_determinism_tiers;
    use engene::core::system_descriptor::{DeterminismTier, SystemDescriptor};

    let desc = SystemDescriptor::new("Sim")
        .with_determinism(DeterminismTier::Hard)
        .with_headless(false);

    let report = audit_determinism_tiers(&[desc]);
    assert!(report.has_issues());
    assert_eq!(report.hard_count, 1);
}

#[test]
fn determinism_audit_clean_report() {
    use engene::core::determinism_audit::audit_determinism_tiers;
    use engene::core::system_descriptor::{DeterminismTier, SystemDescriptor};

    let descriptors = vec![
        SystemDescriptor::new("Physics")
            .with_determinism(DeterminismTier::Hard)
            .with_parallel(true)
            .with_headless(true),
        SystemDescriptor::new("Particles")
            .with_determinism(DeterminismTier::Soft)
            .with_parallel(true),
        SystemDescriptor::new("Editor")
            .with_determinism(DeterminismTier::NonDeterministic)
            .with_headless(false),
    ];

    let report = audit_determinism_tiers(&descriptors);
    assert!(!report.has_issues());
    assert_eq!(report.hard_count, 1);
    assert_eq!(report.soft_count, 1);
    assert_eq!(report.non_count, 1);
}

#[test]
fn serialization_registry_validates() {
    use engene::core::serialization::*;

    let mut reg = SerializationRegistry::new();
    reg.register_schema(TypeSchema {
        type_name: "Transform".to_string(),
        version: SchemaVersion::new(1, 0, 0),
        authority: DataAuthority::Authoritative,
        deterministic: true,
        fields: vec![FieldSchema {
            name: "position".to_string(),
            type_name: "Vec3".to_string(),
            authority: DataAuthority::Authoritative,
            deterministic: true,
        }],
    });

    let issues = reg.validate_schemas();
    assert!(issues.is_empty());
    assert_eq!(reg.authoritative_types().len(), 1);
    assert_eq!(reg.deterministic_types().len(), 1);
}

#[test]
fn serialization_detects_contradictions() {
    use engene::core::serialization::*;

    let mut reg = SerializationRegistry::new();
    reg.register_schema(TypeSchema {
        type_name: "DebugState".to_string(),
        version: SchemaVersion::new(1, 0, 0),
        authority: DataAuthority::DebugOnly,
        deterministic: true,
        fields: vec![],
    });

    let issues = reg.validate_schemas();
    assert!(
        !issues.is_empty(),
        "deterministic + non-authoritative is contradictory"
    );
}

#[test]
fn ownership_map_validates() {
    use engene::core::ownership_map::*;
    use std::any::TypeId;

    struct Transform;

    let mut map = OwnershipMap::new();
    map.register_resource(ResourceOwnership {
        resource_name: "Transforms".to_string(),
        type_id: TypeId::of::<Transform>(),
        owner_system: "Physics".to_string(),
        readers: vec!["Render".to_string()],
        writers: vec!["Physics".to_string()],
        mutation_timing: vec![MutationTiming::FixedTick],
        thread_safety: ThreadSafety::ReadParallelWriteExclusive,
        notes: String::new(),
    });

    let issues = map.validate();
    assert!(issues.is_empty());
    assert_eq!(map.resource_count(), 1);
}

#[test]
fn ownership_map_detects_orphan_events() {
    use engene::core::ownership_map::*;
    use std::any::TypeId;

    struct DamageEvent;

    let mut map = OwnershipMap::new();
    map.register_event(EventOwnership {
        event_name: "DamageEvent".to_string(),
        type_id: TypeId::of::<DamageEvent>(),
        publishers: vec!["Physics".to_string()],
        subscribers: vec![],
        bus: "SimBus".to_string(),
        deterministic: true,
    });

    let issues = map.validate();
    assert!(issues.iter().any(|i| i.contains("no subscribers")));
}

#[test]
fn sdk_contract_validation() {
    use engene::core::sdk::*;

    let mut sdk = EngineSDK::new();
    sdk.register_plugin(PluginContract {
        name: "PhysicsPlugin".to_string(),
        version: "1.0".to_string(),
        provides_systems: vec!["Physics".to_string()],
        requires_systems: vec!["MissingSystem".to_string()],
        reads_resources: vec![],
        writes_resources: vec![],
        emits_events: vec![],
        reads_events: vec![],
        ordering_constraints: vec![],
    });

    let issues = sdk.validate_contracts();
    assert!(issues.iter().any(|i| i.contains("MissingSystem")));
}

#[test]
fn sdk_boundary_layers() {
    use engene::core::sdk::*;

    let mut sdk = EngineSDK::new();
    sdk.register_system(SystemContract {
        name: "Physics".to_string(),
        layer: BoundaryLayer::Engine,
        deterministic: true,
        parallel_safe: true,
        headless_compatible: true,
        public_api: vec!["apply_force".to_string()],
    });
    sdk.register_system(SystemContract {
        name: "QuestSystem".to_string(),
        layer: BoundaryLayer::Game,
        deterministic: false,
        parallel_safe: false,
        headless_compatible: false,
        public_api: vec![],
    });

    let engine_sys = sdk.engine_systems();
    let game_sys = sdk.game_systems();
    assert!(
        engine_sys.iter().any(|s| s.name == "Physics"),
        "Physics should be in engine layer"
    );
    assert!(
        game_sys.iter().any(|s| s.name == "QuestSystem"),
        "QuestSystem should be in game layer"
    );
}

#[test]
fn scene_hierarchy_filter() {
    use engene::tools::scene_hierarchy::SceneHierarchy;

    let mut hierarchy = SceneHierarchy::new();
    hierarchy.rebuild(&[
        (1, "Player".to_string(), None),
        (2, "Enemy_01".to_string(), None),
        (3, "PlayerWeapon".to_string(), Some(1)),
    ]);

    assert_eq!(hierarchy.entity_count(), 3);

    hierarchy.set_filter("player");
    let root = hierarchy.root_entities();
    let visible_count = root
        .iter()
        .filter(|&&id| hierarchy.get_node(id).map_or(false, |n| n.visible))
        .count();
    assert!(visible_count >= 1);
}

#[test]
fn console_execute_command() {
    use engene::tools::console::EngineConsole;

    let mut console = EngineConsole::new();
    console.register_command(
        "echo",
        "Echo input",
        "echo <text>",
        Box::new(|args| args.join(" ")),
    );

    let output = console.execute("echo hello world");
    assert_eq!(output, "hello world");
    assert_eq!(console.history().len(), 1);
}

#[test]
fn asset_browser_filter() {
    use engene::tools::asset_browser::{AssetBrowser, AssetEntry};

    let mut browser = AssetBrowser::new();
    browser.populate(vec![
        AssetEntry {
            id: 1,
            name: "wall.obj".into(),
            asset_type: "Model".into(),
            size_bytes: 1000,
            path: "/assets/wall.obj".into(),
            cooked: true,
        },
        AssetEntry {
            id: 2,
            name: "brick.png".into(),
            asset_type: "Texture".into(),
            size_bytes: 2000,
            path: "/assets/brick.png".into(),
            cooked: false,
        },
    ]);

    browser.set_filter_type(Some("Model".to_string()));
    assert_eq!(browser.visible_assets().len(), 1);

    browser.set_filter_type(None);
    browser.set_filter_text("brick");
    assert_eq!(browser.visible_assets().len(), 1);
}

#[test]
fn replay_recorder_and_player() {
    use engene::core::replay::recorder::{ReplayPlayer, ReplayRecorder};

    let mut recorder = ReplayRecorder::new(42, 60.0);
    recorder.start();
    recorder.record_frame(0, &[1, 2], &[3, 4]);
    recorder.record_frame(1, &[5, 6], &[7, 8]);
    recorder.stop();

    assert_eq!(recorder.frame_count(), 2);
    assert_eq!(recorder.header().seed, 42);

    let frames = recorder.frames().to_vec();
    let mut player = ReplayPlayer::new(frames);
    player.play();

    let f0 = player.advance().unwrap();
    assert_eq!(f0.tick, 0);
    let f1 = player.advance().unwrap();
    assert_eq!(f1.tick, 1);
    assert!(player.advance().is_none());
    assert!(player.is_finished());
}

#[test]
fn checkpoint_nearest_before() {
    use engene::core::replay::checkpoints::{Checkpoint, CheckpointManager};

    let mut mgr = CheckpointManager::new(100, 5);
    mgr.save(Checkpoint {
        tick: 0,
        ecs_snapshot: vec![],
        resource_snapshot: vec![],
        event_state: vec![],
    });
    mgr.save(Checkpoint {
        tick: 100,
        ecs_snapshot: vec![],
        resource_snapshot: vec![],
        event_state: vec![],
    });
    mgr.save(Checkpoint {
        tick: 200,
        ecs_snapshot: vec![],
        resource_snapshot: vec![],
        event_state: vec![],
    });

    let cp = mgr.nearest_before(150).unwrap();
    assert_eq!(cp.tick, 100);
}

#[test]
fn divergence_detector_severity() {
    use engene::core::replay::divergence::{DivergenceDetector, DivergenceSeverity};

    let mut detector = DivergenceDetector::new(0.01);
    let expected = vec![1u8; 1000];
    let mut actual = expected.clone();
    actual[0] = 0;

    detector.compare_snapshots(100, &expected, &actual);
    assert_eq!(detector.points().len(), 1);
    assert_eq!(detector.points()[0].severity, DivergenceSeverity::Minor);
}

// ===== Block 12: SDK Contract Tests =====

#[test]
fn sdk_plugin_contract_registration() {
    use engene::core::sdk::{EngineSDK, PluginContract};
    let mut sdk = EngineSDK::new();
    sdk.register_plugin(PluginContract {
        name: "TestPlugin".into(),
        version: "1.0.0".into(),
        provides_systems: vec!["TestSystem".into()],
        requires_systems: vec![],
        reads_resources: vec![],
        writes_resources: vec![],
        emits_events: vec!["TestEvent".into()],
        reads_events: vec![],
        ordering_constraints: vec![],
    });
    assert_eq!(sdk.plugin_count(), 1);
}

#[test]
fn sdk_system_contract_registration() {
    use engene::core::sdk::{BoundaryLayer, EngineSDK, SystemContract};
    let mut sdk = EngineSDK::new();
    let before = sdk.system_count();
    sdk.register_system(SystemContract {
        name: "CustomPhysicsSystem".into(),
        layer: BoundaryLayer::Engine,
        deterministic: true,
        parallel_safe: false,
        headless_compatible: true,
        public_api: vec!["apply_force".into(), "set_velocity".into()],
    });
    assert_eq!(
        sdk.system_count(),
        before + 1,
        "registering a new system should increase count by 1"
    );
}

// ===== Block 10: Low-Spec Discipline Tests =====

#[test]
fn runtime_config_profiles_all_valid() {
    use engene::core::runtime_config::{ProfileBudgets, RuntimeProfile};
    for profile in [
        RuntimeProfile::Game,
        RuntimeProfile::Game,
        RuntimeProfile::Tools,
        RuntimeProfile::Headless,
        RuntimeProfile::Tools,
        RuntimeProfile::Game,
    ] {
        let budgets = ProfileBudgets::for_profile(&profile);
        assert!(budgets.ai_think_budget_ms >= 0.0);
        assert!(budgets.render_budget_ms >= 0.0);
        assert!(budgets.physics_budget_ms >= 0.0);
    }
}

#[test]
fn quality_governor_responds_to_pressure() {
    use engene::core::quality_governor::QualityGovernor;
    let mut gov = QualityGovernor::new(60);
    for _ in 0..20 {
        gov.update(50_000);
    }
    use engene::core::quality_governor::PressureLevel;
    assert_ne!(
        gov.pressure_level,
        PressureLevel::Normal,
        "governor should detect pressure after many overruns"
    );
}

#[test]
fn budget_registry_tracks_overruns() {
    use engene::core::budget_registry::{BudgetEntry, BudgetRegistry};
    let mut reg = BudgetRegistry::new();
    reg.register(BudgetEntry {
        phase_name: "test_phase",
        owner_system: "TestSystem",
        cpu_budget_us: 1000,
        gpu_budget_us: 0,
        memory_budget_bytes: 0,
        profiler_category: "test",
        overrun_count: 0,
        last_measured_us: 0,
    });
    reg.record_measurement("test_phase", 2000);
    assert_eq!(reg.total_overruns(), 1);
}

#[test]
fn sdk_default_contracts_populated() {
    use engene::core::sdk::EngineSDK;
    let sdk = EngineSDK::new();
    assert!(
        sdk.system_count() >= 16,
        "SDK should auto-populate at least 16 system contracts"
    );
    assert!(
        sdk.plugin_count() == 0,
        "plugins are registered at runtime, not at construction"
    );
}

#[test]
fn sdk_contracts_validate_clean() {
    use engene::core::sdk::EngineSDK;
    let sdk = EngineSDK::new();
    let issues = sdk.validate_contracts();
    assert!(
        issues.is_empty(),
        "default SDK contracts should validate cleanly: {:?}",
        issues
    );
}

#[test]
fn sdk_boundary_layers_populated() {
    use engene::core::sdk::EngineSDK;
    let sdk = EngineSDK::new();
    assert!(
        !sdk.engine_systems().is_empty(),
        "engine boundary should have systems"
    );
    assert!(
        !sdk.game_systems().is_empty(),
        "game boundary should have systems"
    );
}

#[test]
fn editor_shell_default_initializes_all_panels() {
    use engene::tools::editor_shell::EditorShell;
    let shell = EditorShell::new();
    assert!(!shell.read_only, "editor shell defaults to writable mode");
    assert!(shell.debug_ui.show_inspector, "SDK defaults show inspector");
    assert_eq!(shell.crash_log.crash_count(), 0);
    assert!(shell.quest_board.entries.is_empty());
    assert!(shell.economy.latest().is_none());
    assert!(shell.sim_metrics.latest().is_none());
    assert_eq!(shell.persistence.total_saves, 0);
}
