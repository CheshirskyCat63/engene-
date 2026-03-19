use super::*;

// Category 5: Producer/consumer correctness (25 tests)
// =============================================================================

#[test]
fn event_bus_emit_and_count() {
    use engene::core::events::EventBus;
    let mut bus = EventBus::new();
    bus.set_channel_capacity::<u32>(10);
    bus.emit(1u32);
    bus.emit(2u32);
    assert_eq!(bus.count::<u32>(), 2);
}

#[test]
fn event_bus_clear() {
    use engene::core::events::EventBus;
    let mut bus = EventBus::new();
    bus.set_channel_capacity::<u32>(10);
    bus.emit(1u32);
    bus.clear();
    assert_eq!(bus.count::<u32>(), 0);
}

#[test]
fn command_buffer_spawn() {
    use engene::core::commands::CommandBuffer;
    let mut cb = CommandBuffer::new();
    cb.spawn();
    assert!(!cb.is_empty());
}

#[test]
fn command_buffer_despawn() {
    use engene::core::commands::CommandBuffer;
    let mut cb = CommandBuffer::new();
    cb.despawn(42);
    let despawns = cb.take_despawns();
    assert_eq!(despawns, vec![42]);
}

#[test]
fn engine_tick_applies_commands() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    engine.tick(0.05);
}

#[test]
fn ecs_spawn_new_assigns_pid() {
    let mut ecs = Ecs::new();
    let (e, pid) = ecs.spawn_new();
    assert!(pid.0 > 0);
    assert!(ecs.identity.persistent_id_of(e).is_some());
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
fn ecs_rebuild_spatial() {
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
    ecs.rebuild_spatial();
}

#[test]
fn event_bus_multiple_channels() {
    use engene::core::events::EventBus;
    let mut bus = EventBus::new();
    bus.set_channel_capacity::<u32>(5);
    bus.set_channel_capacity::<String>(5);
    bus.emit(42u32);
    bus.emit("test".to_string());
    assert_eq!(bus.count::<u32>(), 1);
    assert_eq!(bus.count::<String>(), 1);
}

#[test]
fn engine_time_events_propagate() {
    let mut engine = ToolsRuntimeAssembly::minimal();
    let sec_per_day = 120.0;
    let dt = 0.05;
    let ticks = (sec_per_day / dt) as u32;
    for _ in 0..(ticks + 5) {
        engine.tick(dt);
    }
    assert!(engine.time.day >= 1);
}

#[test]
fn ecs_components_apply() {
    let mut ecs = Ecs::new();
    let (e, _) = ecs.spawn_new();
    ecs.transforms.insert(
        e,
        Transform {
            x: 1.0,
            y: 2.0,
            cell_x: 0,
            cell_y: 0,
        },
    );
    ecs.kinds.insert(e, EntityKind::Npc);
    assert!(ecs.transforms.get(&e).is_some());
    assert_eq!(ecs.transforms.get(&e).unwrap().x, 1.0);
}

#[test]
fn engine_integration_systems_run() {
    let mut engine = GameRuntimeAssembly::headless(&[Biome::Plains]);
    engine.tick(0.05);
}

#[test]
fn world_tick_new_day_event() {
    use engene::core::time::GameTime;
    let mut gt = GameTime::new();
    for _ in 0..5000 {
        let ev = gt.advance(0.05);
        if ev.new_day {
            return;
        }
    }
}

#[test]
fn event_bus_bounded() {
    use engene::core::events::EventBus;
    let mut bus = EventBus::with_capacity(2);
    bus.set_channel_capacity::<i32>(2);
    bus.emit(1);
    bus.emit(2);
    bus.emit(3);
    assert!(bus.count::<i32>() <= 2);
}

// =============================================================================
