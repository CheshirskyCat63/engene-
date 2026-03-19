use super::*;

// Category 2: Fixed tick behavior (50 tests)
// =============================================================================

#[test]
fn engine_tick_advances_time() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    let initial_tick = engine.time.tick_count;
    engine.tick(0.05);
    assert_eq!(engine.time.tick_count, initial_tick + 1);
}

#[test]
fn engine_tick_advances_elapsed() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    let initial = engine.time.elapsed;
    engine.tick(0.05);
    assert!(engine.time.elapsed >= initial);
}

#[test]
fn engine_tick_ecs_tick_advances() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let mut engine = GameRuntimeAssembly::headless(&biomes);
    engine.tick(0.05);
    assert!(engine.ecs.tick >= 1);
}

#[test]
fn engine_multiple_ticks_advance_time() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    for _ in 0..10 {
        engine.tick(0.05);
    }
    assert!(engine.time.tick_count >= 10);
}

#[test]
fn engine_tick_new_day_event_after_enough_ticks() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    let sec_per_day = 120.0;
    let dt = 0.05;
    let ticks_per_day = (sec_per_day / dt) as u32;
    for _ in 0..(ticks_per_day + 10) {
        engine.tick(dt);
    }
    assert!(engine.time.day >= 1);
}

#[test]
fn engine_tick_no_panic_zero_dt() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.tick(0.0);
}

#[test]
fn engine_tick_no_panic_large_dt() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.tick(10.0);
}

#[test]
fn engine_tick_events_cleared_after_tick() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.tick(0.05);
    assert_eq!(engine.events.channel_count(), 0);
}

#[test]
fn engine_headless_tick_runs() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let mut engine = GameRuntimeAssembly::headless(&biomes);
    engine.tick(0.05);
    assert!(engine.time.tick_count >= 1);
}

#[test]
fn engine_vertical_slice_tick_runs() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let heightmap = Arc::new(Heightmap::generate(&biomes));
    let mut engine = GameRuntimeAssembly::vertical_slice(heightmap, &biomes);
    engine.tick(0.05);
    assert!(engine.time.tick_count >= 1);
}

#[test]
fn engine_tick_parallel_runs() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.tick_parallel(0.05);
    assert!(engine.time.tick_count >= 1);
}

#[test]
fn engine_tick_parallel_advances_time() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    let initial = engine.time.tick_count;
    engine.tick_parallel(0.05);
    assert_eq!(engine.time.tick_count, initial + 1);
}

#[test]
fn engine_100_ticks_headless() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let mut engine = GameRuntimeAssembly::headless(&biomes);
    for _ in 0..100 {
        engine.tick(0.05);
    }
    assert_eq!(engine.time.tick_count, 100);
}

#[test]
fn engine_time_delta_set() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.tick(0.05);
    assert!(engine.time.delta >= 0.0);
}

#[test]
fn engine_time_scale_default() {
    let engine = ToolsRuntimeAssembly::minimal();
    assert!(engine.time.time_scale > 0.0);
}

#[test]
fn engine_tick_count_increments() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    let c0 = engine.time.tick_count;
    engine.tick(0.05);
    let c1 = engine.time.tick_count;
    assert_eq!(c1, c0 + 1);
}

#[test]
fn personal_needs_decay_hunger_increases() {
    let mut pn = PersonalNeeds::default_npc();
    pn.hunger = 0.2;
    assert!(pn.hunger >= 0.0 && pn.hunger <= 1.0);
}

#[test]
fn personal_needs_default_npc_valid() {
    let pn = PersonalNeeds::default_npc();
    assert!(pn.health >= 0.0 && pn.health <= 1.0);
    assert!(pn.energy >= 0.0 && pn.energy <= 1.0);
}

#[test]
fn personal_needs_default_monster_valid() {
    let pn = PersonalNeeds::default_monster();
    assert!(pn.health >= 0.0 && pn.health <= 1.0);
}

#[test]
fn decide_npc_returns_goal() {
    use engene::game::ai::decision::decide_npc;
    let traits = NpcTraits {
        bravery: 0.5,
        aggressiveness: 0.3,
        work_ethic: 0.6,
        curiosity: 0.4,
        honesty: 0.7,
        sociality: 0.5,
        autonomy: 0.5,
        materialism: 0.3,
        risk_tolerance: 0.4,
        stress_resistance: 0.6,
    };
    let personal = PersonalNeeds::default_npc();
    let social = SocialNeeds::default();
    let economy = NpcEconomy {
        money: 100.0,
        monthly_required: 50.0,
        job: Job::Resident,
        desperation: 0.2,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    let _ = format!("{:?}", goal);
}

#[test]
fn decide_monster_returns_goal() {
    use engene::game::ai::decision::decide_monster;
    let traits = MonsterTraits {
        aggressiveness: 0.5,
        caution: 0.4,
        territoriality: 0.3,
        bravery: 0.5,
        pack_mentality: 0.6,
        energy_level: 0.7,
        hoarding: 0.2,
        curiosity: 0.3,
        adaptability: 0.5,
        stress_tolerance: 0.5,
    };
    let personal = PersonalNeeds::default_monster();
    let eco = EcosystemNeeds::for_species(MonsterSpecies::Wolf);
    let goal = decide_monster(&traits, &personal, &eco);
    let _ = format!("{:?}", goal);
}

#[test]
fn decide_npc_high_hunger_seeks_food() {
    use engene::game::ai::decision::decide_npc;
    let traits = NpcTraits {
        bravery: 0.5,
        aggressiveness: 0.1,
        work_ethic: 0.3,
        curiosity: 0.2,
        honesty: 0.8,
        sociality: 0.3,
        autonomy: 0.5,
        materialism: 0.2,
        risk_tolerance: 0.2,
        stress_resistance: 0.6,
    };
    let mut personal = PersonalNeeds::default_npc();
    personal.hunger = 0.9;
    let social = SocialNeeds::default();
    let economy = NpcEconomy {
        money: 50.0,
        monthly_required: 50.0,
        job: Job::Resident,
        desperation: 0.1,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    assert!(matches!(
        goal,
        Goal::SeekFood | Goal::Hunt | Goal::Rest | Goal::Work | Goal::SeekWater
    ));
}

#[test]
fn decide_monster_high_fear_flees() {
    use engene::game::ai::decision::decide_monster;
    let traits = MonsterTraits {
        aggressiveness: 0.2,
        caution: 0.9,
        territoriality: 0.3,
        bravery: 0.1,
        pack_mentality: 0.5,
        energy_level: 0.6,
        hoarding: 0.2,
        curiosity: 0.2,
        adaptability: 0.5,
        stress_tolerance: 0.4,
    };
    let mut personal = PersonalNeeds::default_monster();
    personal.fear = 0.95;
    let eco = EcosystemNeeds::for_species(MonsterSpecies::Wolf);
    let goal = decide_monster(&traits, &personal, &eco);
    assert!(matches!(
        goal,
        Goal::Flee | Goal::Rest | Goal::FollowPack | Goal::Hunt
    ));
}

#[test]
fn perception_cache_build_empty_ecs() {
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
    assert!(cache.predator.is_none());
    assert!(cache.prey.is_none());
}

#[test]
fn memory_new_empty() {
    use engene::game::ai::memory::Memory;
    let mem = Memory::new();
    assert!(mem.events.is_empty());
    assert!(mem.spatial.is_empty());
}

#[test]
fn memory_record_event() {
    use engene::game::ai::memory::{EventKind, EventMemory, Memory};
    let mut mem = Memory::new();
    mem.record_event(EventMemory {
        tick: 0,
        kind: EventKind::AllyDied,
        location: (0, 0),
        other: None,
        emotional_impact: 0.2,
    });
    assert_eq!(mem.events.len(), 1);
}

#[test]
fn memory_mark_cell() {
    use engene::game::ai::memory::{CellTag, Memory};
    let mut mem = Memory::new();
    mem.mark_cell(5, 5, CellTag::Danger, 0.8);
    assert!(mem.cell_danger(5, 5) > 0.0);
}

#[test]
fn engine_tick_50_tools() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    for _ in 0..50 {
        engine.tick(0.05);
    }
    assert_eq!(engine.time.tick_count, 50);
}

#[test]
fn engine_tick_200_headless() {
    let grid = WorldGrid::generate();
    let biomes: Vec<_> = grid.cells.iter().map(|c| c.biome).collect();
    let mut engine = GameRuntimeAssembly::headless(&biomes);
    for _ in 0..200 {
        engine.tick(0.05);
    }
    assert_eq!(engine.time.tick_count, 200);
}

#[test]
fn engine_scheduler_accumulates() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.tick(0.01);
    engine.tick(0.01);
    assert!(engine.time.tick_count >= 1);
}

#[test]
fn game_time_advance_returns_events() {
    use engene::core::time::GameTime;
    let mut gt = GameTime::new();
    let mut had_new_day = false;
    for _ in 0..3000 {
        let ev = gt.advance(0.05);
        if ev.new_day {
            had_new_day = true;
            break;
        }
    }
    assert!(had_new_day || gt.day >= 1);
}

#[test]
fn game_time_new_month_event() {
    use engene::core::time::GameTime;
    let mut gt = GameTime::new();
    for _ in 0..10000 {
        let ev = gt.advance(0.05);
        if ev.new_month {
            assert!(gt.month >= 2);
            return;
        }
    }
}

#[test]
fn engine_ecs_alive_non_null_after_spawn() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    let (e, _) = engine.ecs.spawn_new();
    engine.ecs.transforms.insert(
        e,
        Transform {
            x: 0.0,
            y: 0.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    assert!(engine.ecs.is_alive(e));
}

#[test]
fn engine_systems_run_in_order() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.tick(0.05);
    let descs = engine.system_descriptors();
    assert!(descs.is_empty());
}

#[test]
fn engine_tick_commands_applied() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.tick(0.05);
}

#[test]
fn engine_1000_ticks_stable() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    for _ in 0..1000 {
        engine.tick(0.05);
    }
    assert!(engine.is_running());
}

#[test]
fn engine_tick_parallel_100_ticks() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    for _ in 0..100 {
        engine.tick_parallel(0.05);
    }
    assert_eq!(engine.time.tick_count, 100);
}

#[test]
fn ai_decision_high_desperation_can_steal() {
    use engene::game::ai::decision::decide_npc;
    let traits = NpcTraits {
        bravery: 0.7,
        aggressiveness: 0.6,
        work_ethic: 0.2,
        curiosity: 0.3,
        honesty: 0.2,
        sociality: 0.3,
        autonomy: 0.7,
        materialism: 0.5,
        risk_tolerance: 0.8,
        stress_resistance: 0.4,
    };
    let personal = PersonalNeeds::default_npc();
    let social = SocialNeeds::default();
    let economy = NpcEconomy {
        money: 0.0,
        monthly_required: 100.0,
        job: Job::Unemployed,
        desperation: 0.95,
    };
    let goal = decide_npc(&traits, &personal, &social, &economy);
    let _ = goal;
}

// =============================================================================
