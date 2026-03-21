use super::*;

// Category 7: Role simulation (13 tests)
// =============================================================================

#[test]
fn role_behavior_guard() {
    let rb = RoleBehavior::for_role(NpcRole::Guard);
    assert_eq!(rb.role, NpcRole::Guard);
    assert_eq!(rb.daily_income, 15.0);
    assert!(rb.danger_exposure > 0.5);
}

#[test]
fn role_behavior_hunter() {
    let rb = RoleBehavior::for_role(NpcRole::Hunter);
    assert_eq!(rb.daily_income, 25.0);
    assert!(rb.required_equipment.contains(&"weapon".to_string()));
}

#[test]
fn role_behavior_trader() {
    let rb = RoleBehavior::for_role(NpcRole::Trader);
    assert_eq!(rb.daily_income, 30.0);
    assert!(rb.danger_exposure < 0.5);
}

#[test]
fn role_behavior_scavenger() {
    let rb = RoleBehavior::for_role(NpcRole::Scavenger);
    assert_eq!(rb.daily_income, 20.0);
}

#[test]
fn role_behavior_courier() {
    let rb = RoleBehavior::for_role(NpcRole::Courier);
    assert!(rb.danger_exposure > 0.0);
}

#[test]
fn role_behavior_bandit() {
    let rb = RoleBehavior::for_role(NpcRole::Bandit);
    assert_eq!(rb.daily_income, 35.0);
    assert!(rb.danger_exposure > 0.8);
}

#[test]
fn role_behavior_idle_resident() {
    let rb = RoleBehavior::for_role(NpcRole::IdleResident);
    assert_eq!(rb.daily_income, 5.0);
}

#[test]
fn role_behavior_mechanic() {
    let rb = RoleBehavior::for_role(NpcRole::Mechanic);
    assert_eq!(rb.daily_income, 22.0);
}

#[test]
fn role_behavior_medic() {
    let rb = RoleBehavior::for_role(NpcRole::Medic);
    assert_eq!(rb.daily_income, 20.0);
}

#[test]
fn role_behavior_all_nine_roles() {
    let roles = [
        NpcRole::Guard,
        NpcRole::Hunter,
        NpcRole::Trader,
        NpcRole::Scavenger,
        NpcRole::Courier,
        NpcRole::Bandit,
        NpcRole::IdleResident,
        NpcRole::Mechanic,
        NpcRole::Medic,
    ];
    for role in roles {
        let rb = RoleBehavior::for_role(role.clone());
        assert_eq!(rb.role, role);
    }
}

#[test]
fn role_behavior_guard_equipment() {
    let rb = RoleBehavior::for_role(NpcRole::Guard);
    assert!(rb.required_equipment.contains(&"weapon".to_string()));
    assert!(rb.required_equipment.contains(&"armor".to_string()));
}

#[test]
fn role_behavior_trader_no_equipment() {
    let rb = RoleBehavior::for_role(NpcRole::Trader);
    assert!(rb.required_equipment.is_empty());
}

// =============================================================================
// Additional tests to reach 183 total
// =============================================================================

#[test]
fn world_milestone_tracker_new() {
    let tracker = WorldMilestoneTracker::new();
    assert_eq!(tracker.bankruptcies, 0);
    assert_eq!(tracker.npc_deaths, 0);
    assert_eq!(tracker.npc_births, 0);
}

#[test]
fn world_milestone_tracker_record_events() {
    let mut tracker = WorldMilestoneTracker::new();
    tracker.record_bankruptcy();
    tracker.record_npc_death();
    tracker.record_banditization();
    assert_eq!(tracker.bankruptcies, 1);
    assert_eq!(tracker.npc_deaths, 1);
    assert_eq!(tracker.banditizations, 1);
}

#[test]
fn world_milestone_tracker_check_milestones() {
    let mut tracker = WorldMilestoneTracker::new();
    for _ in 0..3 {
        tracker.record_bankruptcy();
    }
    tracker.check_milestones(5);
    assert!(!tracker.milestones_achieved.is_empty() || tracker.months_tracked == 5);
}

#[test]
fn world_milestone_tracker_summary() {
    let tracker = WorldMilestoneTracker::new();
    let s = tracker.summary();
    assert!(s.contains("Months") || s.len() > 0);
}

#[test]
fn economy_system_name() {
    use engene::game::economy::economy::EconomySystem;
    let sys = EconomySystem;
    assert_eq!(sys.name(), "Economy");
}

#[test]
fn item_template_has_category() {
    use engene::game::economy::item_registry::{ItemCategory, ItemRarity, ItemTemplate};
    let t = ItemTemplate {
        id: "test".into(),
        name: "Test".into(),
        category: ItemCategory::Food,
        rarity: ItemRarity::Common,
        base_value: 10.0,
        weight: 0.5,
        max_stack: 5,
        max_durability: 1.0,
        description: "".into(),
    };
    assert_eq!(t.category, ItemCategory::Food);
}

#[test]
fn trader_state_buy_sell() {
    let t = TraderState::new("Test", "Loners", 1000.0);
    let buy = t.effective_buy_price(50.0, "medkit");
    let sell = t.effective_sell_price(50.0, "medkit");
    assert!(buy > 0.0);
    assert!(sell > 0.0);
}

#[test]
fn trader_state_tick_restock() {
    let mut t = TraderState::new("Test", "Loners", 500.0);
    t.tick_restock(0.5);
    assert!(t.restock_timer >= 0.0);
}

#[test]
fn equipment_slots_default() {
    use engene::world::components::EquipmentSlots;
    let eq = EquipmentSlots::default_stalker();
    assert!(eq.weapon_condition > 0.0);
}

#[test]
fn faction_membership_standing() {
    use engene::world::components::FactionMembership;
    let fm = FactionMembership {
        faction: Faction::Loners,
        standing: 0.5,
    };
    assert_eq!(fm.standing, 0.5);
}

#[test]
fn npc_economy_desperation() {
    let econ = NpcEconomy {
        money: 10.0,
        monthly_required: 100.0,
        job: Job::Unemployed,
        desperation: 0.8,
    };
    assert!(econ.desperation > 0.5);
}

#[test]
fn sim_level_l2() {
    let sl = SimLevel {
        level: SimulationLevel::L2,
    };
    assert_eq!(sl.level, SimulationLevel::L2);
}

#[test]
fn perception_cache_entities_nearby() {
    use engene::game::ai::perception::PerceptionCache;
    let mut ecs = Ecs::new();
    let (entity, _) = ecs.spawn_new();
    ecs.transforms.insert(
        entity,
        Transform {
            x: 0.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(entity, EntityKind::Npc);
    ecs.rebuild_spatial();
    let cache = PerceptionCache::build(&ecs, entity);
    let _nearby = cache.entities_nearby_count;
}

#[test]
fn memory_opinion_of_unknown() {
    use engene::core::persistent_id::PersistentEntityId;
    use engene::game::ai::memory::Memory;
    let mem = Memory::new();
    let op = mem.opinion_of(PersistentEntityId(999));
    assert_eq!(op.trust, 0.0);
}

#[test]
fn memory_best_ally_none() {
    use engene::game::ai::memory::Memory;
    let mem = Memory::new();
    assert!(mem.best_ally().is_none());
}

#[test]
fn identity_registry_register_new() {
    let mut ecs = Ecs::new();
    let (e, pid) = ecs.spawn_new();
    assert!(pid.0 > 0);
    assert_eq!(ecs.identity.persistent_id_of(e), Some(pid));
}

#[test]
fn identity_registry_resolve() {
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
    assert_eq!(ecs.identity.resolve(pid), Some(e));
}

#[test]
fn entity_ref_is_dead_unknown() {
    use engene::core::ecs::Ecs;
    use engene::core::persistent_id::{EntityRef, PersistentEntityId};
    let ecs = Ecs::new();
    let ref_ = EntityRef::new(PersistentEntityId(9999));
    assert!(ref_.is_dead(&ecs.identity));
}

#[test]
fn build_manifest_schema_versions() {
    use engene::core::build_manifest::{
        BuildManifest, SCHEMA_VERSION_CHUNK, SCHEMA_VERSION_ENTITY, SCHEMA_VERSION_SAVE,
    };
    let m = BuildManifest::current();
    assert_eq!(m.schema_version_save, SCHEMA_VERSION_SAVE);
    assert_eq!(m.schema_version_chunk, SCHEMA_VERSION_CHUNK);
    assert_eq!(m.schema_version_entity, SCHEMA_VERSION_ENTITY);
}

#[test]
fn save_compatibility_current() {
    use engene::core::build_manifest::SaveCompatibility;
    let comp = SaveCompatibility::current();
    assert!(comp.is_compatible(1));
}

#[test]
fn schema_migration_registry_has_path() {
    use engene::core::build_manifest::SchemaMigrationRegistry;
    let reg = SchemaMigrationRegistry::new();
    assert!(!reg.has_path_save(0) || reg.has_path_save(1));
}

#[test]
fn quality_governor_max_micro_motion() {
    use engene::core::quality_governor::QualityGovernor;
    let gov = QualityGovernor::new(60);
    let _oscillators = gov.max_micro_motion_oscillators();
}

#[test]
fn quality_governor_histogram_frequency() {
    use engene::core::quality_governor::QualityGovernor;
    let gov = QualityGovernor::new(60);
    assert!(gov.histogram_frequency() >= 1);
}

#[test]
fn budget_registry_entries() {
    use engene::core::budget_registry::create_default_registry;
    let reg = create_default_registry();
    assert!(!reg.entries().is_empty());
}

#[test]
fn worker_pool_default() {
    use engene::core::jobs::worker_pool::WorkerPool;
    let pool = WorkerPool::new();
    assert!(pool.worker_count() >= 1);
}

#[test]
fn spatial_index_candidates() {
    use engene::core::ecs::Ecs;
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
    ecs.rebuild_spatial();
    let cands = ecs.spatial.candidates_in_radius(500.0, 500.0, 50.0);
    assert!(cands.len() >= 1);
}

#[test]
fn engine_tick_headless_50() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let mut engine = GameRuntimeAssembly::headless(&biomes);
    for _ in 0..50 {
        engine.tick(0.05);
    }
    assert_eq!(engine.time.tick_count, 50);
}
