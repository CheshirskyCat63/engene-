use rand::Rng;
use std::hint::black_box;
use std::time::Instant;

fn bench_command_buffer_throughput() {
    use engene::core::commands::CommandBuffer;

    let iterations = 10_000;
    let start = Instant::now();
    let mut cb = CommandBuffer::new();
    for i in 0..iterations {
        cb.despawn(i);
    }
    let elapsed = start.elapsed();
    println!(
        "[CommandBuffer] {} ops in {:.2?}, {:.0} ops/sec",
        iterations,
        elapsed,
        iterations as f64 / elapsed.as_secs_f64()
    );
}

fn bench_event_bus_throughput() {
    use engene::core::events::EventBus;

    let iterations = 50_000;
    let start = Instant::now();
    let mut bus = EventBus::new();
    for i in 0u64..iterations {
        bus.emit(i);
    }
    let elapsed = start.elapsed();
    println!(
        "[EventBus] {} emits in {:.2?}, {:.0} ops/sec",
        iterations,
        elapsed,
        iterations as f64 / elapsed.as_secs_f64()
    );
}

fn bench_worker_pool_fanout() {
    use engene::core::jobs::worker_pool::WorkerPool;

    let pool = WorkerPool::new();
    let task_count = 100;
    let start = Instant::now();

    let results: Vec<u64> = (0..task_count)
        .map(|_i| {
            pool.execute(move || {
                let mut sum = 0u64;
                for j in 0..10_000 {
                    sum += black_box(j);
                }
                sum
            })
        })
        .collect();

    let elapsed = start.elapsed();
    let total: u64 = results.iter().sum();
    println!(
        "[WorkerPool] {} tasks in {:.2?}, total={}, {:.0} tasks/sec",
        task_count,
        elapsed,
        total,
        task_count as f64 / elapsed.as_secs_f64()
    );
}

fn bench_dependency_graph_invalidation() {
    use engene::content::cooking::dependency_graph::ContentDependencyGraph;

    let node_count = 1000;
    let mut graph = ContentDependencyGraph::new();
    for i in 1..node_count {
        graph.add_dependency(i, i - 1);
    }

    let start = Instant::now();
    let affected = graph.invalidate(0);
    let elapsed = start.elapsed();
    println!(
        "[DependencyGraph] invalidate root of {} nodes -> {} affected in {:.2?}",
        node_count,
        affected.len(),
        elapsed
    );
}

fn bench_prefab_registry_lookup() {
    use engene::content::prefabs::prefab::{PrefabDescriptor, PrefabEntity};
    use engene::content::prefabs::prefab_registry::PrefabRegistry;

    let mut registry = PrefabRegistry::new();
    for i in 0..500 {
        registry.register(PrefabDescriptor {
            name: format!("prefab_{}", i),
            base: if i > 0 {
                Some(format!("prefab_{}", i - 1))
            } else {
                None
            },
            spec_variant: None,
            root: PrefabEntity {
                name: Some(format!("Entity_{}", i)),
                components: vec![],
                children: vec![],
            },
            tags: vec![],
        });
    }

    let start = Instant::now();
    let iterations = 1000;
    for _ in 0..iterations {
        let _ = black_box(registry.get("prefab_250"));
    }
    let elapsed = start.elapsed();
    println!(
        "[PrefabRegistry] {} lookups in {:.2?}, {:.0} lookups/sec",
        iterations,
        elapsed,
        iterations as f64 / elapsed.as_secs_f64()
    );
}

fn bench_io_budget_streamer() {
    use engene::world::chunk_package::{BundleKind, StreamPriority};
    use engene::world::io_budget::*;
    use engene::world::streaming::ChunkCoord;

    let budget = IoBudget::default();
    let mut streamer = BudgetedStreamer::new(budget);

    for i in 0..200 {
        streamer.enqueue(StreamRequest {
            coord: ChunkCoord { x: i, z: 0 },
            bundle_kind: if i % 3 == 0 {
                BundleKind::Collision
            } else {
                BundleKind::Vegetation
            },
            priority: if i % 3 == 0 {
                StreamPriority::Critical
            } else {
                StreamPriority::Cosmetic
            },
            size_bytes: 50_000,
            compressed_bytes: 25_000,
            distance_sq: (i * i) as f32,
        });
    }

    let start = Instant::now();
    let mut total_loaded = 0;
    for _ in 0..60 {
        let result = streamer.process_frame();
        total_loaded += result.loaded.len();
    }
    let elapsed = start.elapsed();
    println!(
        "[BudgetedStreamer] 60 frames processed in {:.2?}, {} bundles loaded",
        elapsed, total_loaded
    );
}

fn bench_world_tick_100_entities() {
    use engene::core::ecs::Ecs;
    use engene::world::components::*;
    let mut ecs = Ecs::new();
    for i in 0..100u64 {
        let (entity, _pid) = ecs.spawn_new();
        ecs.transforms.insert(
            entity,
            Transform {
                x: (i as f32) * 20.0,
                y: (i as f32) * 20.0,
                cell_x: 0,
                cell_y: 0,
            },
        );
        ecs.kinds.insert(entity, EntityKind::Npc);
        ecs.personal_needs
            .insert(entity, PersonalNeeds::default_npc());
        ecs.npc_economies.insert(
            entity,
            NpcEconomy {
                money: 500.0,
                monthly_required: 100.0,
                job: Job::ArtifactHunter,
                desperation: 0.0,
            },
        );
        ecs.sim_levels.insert(
            entity,
            SimLevel {
                level: SimulationLevel::L0,
            },
        );
    }

    let player_x = 1000.0;
    let player_y = 1000.0;
    let start = Instant::now();
    let iterations = 1000;
    let mut deferred_queue = engine_runtime::simulation_core::DeferredTransitionQueue::with_policy(
        engine_runtime::simulation_core::DeferredTransitionPolicy::default(),
    );
    let mut transition_batch = Vec::with_capacity(deferred_queue.policy().max_queue_capacity);
    for i in 0..iterations {
        let _ = engene::simulation::activation::update_simulation_levels(
            &mut ecs,
            player_x,
            player_y,
            i as u64,
            i as u64,
            &mut deferred_queue,
            &mut transition_batch,
        );
    }
    let elapsed = start.elapsed();
    println!(
        "[WorldTick100] {} iterations in {:.2?}, {:.0} iter/sec",
        iterations,
        elapsed,
        iterations as f64 / elapsed.as_secs_f64()
    );
}

fn bench_ai_batch_50_npcs() {
    use engene::core::ecs::Ecs;
    use engene::game::ai::decision;
    use engene::game::ai::emotions::Emotions;
    use engene::game::ai::memory::Memory;
    use engene::world::components::*;

    let mut ecs = Ecs::new();
    let mut rng = rand::thread_rng();
    for i in 0..50u64 {
        let (entity, _pid) = ecs.spawn_new();
        ecs.transforms.insert(
            entity,
            Transform {
                x: (i as f32) * 30.0,
                y: (i as f32) * 30.0,
                cell_x: 0,
                cell_y: 0,
            },
        );
        ecs.kinds.insert(entity, EntityKind::Npc);
        ecs.npc_traits.insert(
            entity,
            NpcTraits {
                bravery: rng.gen(),
                aggressiveness: rng.gen(),
                work_ethic: rng.gen(),
                curiosity: rng.gen(),
                honesty: rng.gen(),
                sociality: rng.gen(),
                autonomy: rng.gen(),
                materialism: rng.gen(),
                risk_tolerance: rng.gen(),
                stress_resistance: rng.gen(),
            },
        );
        ecs.personal_needs
            .insert(entity, PersonalNeeds::default_npc());
        ecs.ai_states.insert(entity, AiState::Idle);
        ecs.emotions.insert(entity, Emotions::new());
        ecs.memories.insert(entity, Memory::new());
        ecs.sim_levels.insert(
            entity,
            SimLevel {
                level: SimulationLevel::L0,
            },
        );
    }

    let start = Instant::now();
    let iterations = 500;
    for _ in 0..iterations {
        for &entity in &ecs.alive.clone() {
            if let Some(EntityKind::Npc) = ecs.kinds.get(&entity) {
                let traits = ecs.npc_traits.get(&entity).cloned().unwrap_or(NpcTraits {
                    bravery: 0.5,
                    aggressiveness: 0.5,
                    work_ethic: 0.5,
                    curiosity: 0.5,
                    honesty: 0.5,
                    sociality: 0.5,
                    autonomy: 0.5,
                    materialism: 0.5,
                    risk_tolerance: 0.5,
                    stress_resistance: 0.5,
                });
                let needs = ecs
                    .personal_needs
                    .get(&entity)
                    .cloned()
                    .unwrap_or(PersonalNeeds::default_npc());
                let social = ecs.social_needs.get(&entity).cloned().unwrap_or_default();
                let economy = ecs
                    .npc_economies
                    .get(&entity)
                    .cloned()
                    .unwrap_or(NpcEconomy {
                        money: 100.0,
                        monthly_required: 50.0,
                        job: Job::Guard,
                        desperation: 0.0,
                    });
                let _goal = decision::decide_npc(&traits, &needs, &social, &economy);
            }
        }
    }
    let elapsed = start.elapsed();
    println!(
        "[AiBatch50] {} iterations in {:.2?}, {:.0} decisions/sec",
        iterations,
        elapsed,
        (iterations * 50) as f64 / elapsed.as_secs_f64()
    );
}

fn bench_chunk_save_load_roundtrip() {
    use engene::core::ecs::Ecs;
    use engene::world::chunk_persistence::ChunkPersistenceService;
    use engene::world::components::*;
    use engene::world::streaming::ChunkCoord;

    let test_dir = std::env::temp_dir().join("engene_bench_chunks");
    let _ = std::fs::remove_dir_all(&test_dir);

    let mut ecs = Ecs::new();
    let coord = ChunkCoord { x: 0, z: 0 };
    for i in 0..20u64 {
        let (entity, _pid) = ecs.spawn_new();
        ecs.transforms.insert(
            entity,
            Transform {
                x: (i as f32) * 40.0,
                y: (i as f32) * 40.0,
                cell_x: 0,
                cell_y: 0,
            },
        );
        ecs.kinds.insert(entity, EntityKind::Npc);
        ecs.personal_needs
            .insert(entity, PersonalNeeds::default_npc());
        ecs.npc_economies.insert(
            entity,
            NpcEconomy {
                money: 300.0,
                monthly_required: 80.0,
                job: Job::Guard,
                desperation: 0.1,
            },
        );
        ecs.inventories.insert(
            entity,
            Inventory {
                items: vec![Item {
                    name: "Medkit".into(),
                    value: 50.0,
                }],
            },
        );
        ecs.equipment
            .insert(entity, EquipmentSlots::default_stalker());
        ecs.sim_levels.insert(
            entity,
            SimLevel {
                level: SimulationLevel::L0,
            },
        );
    }

    let mut persistence = ChunkPersistenceService::new(test_dir.to_str().unwrap());

    let start = Instant::now();
    let iterations = 50;
    for _ in 0..iterations {
        persistence.save_and_unload(coord, &mut ecs, 0);
        persistence.load_chunk_entities(coord, &mut ecs);
    }
    let elapsed = start.elapsed();
    println!(
        "[ChunkSaveLoad] {} roundtrips in {:.2?}, {:.1} ms/roundtrip",
        iterations,
        elapsed,
        elapsed.as_millis() as f64 / iterations as f64
    );

    let _ = std::fs::remove_dir_all(&test_dir);
}

fn bench_frustum_cull_1000_instances() {
    use engene::graphics::visibility::Frustum;

    let vp = glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 16.0 / 9.0, 0.1, 1000.0)
        * glam::Mat4::look_at_rh(
            glam::Vec3::new(500.0, 100.0, 500.0),
            glam::Vec3::new(500.0, 0.0, 500.0),
            glam::Vec3::Y,
        );
    let frustum = Frustum::from_view_projection(&vp);

    let positions: Vec<glam::Vec3> = (0..1000)
        .map(|i| glam::Vec3::new((i % 32) as f32 * 32.0, 0.0, (i / 32) as f32 * 32.0))
        .collect();

    let start = Instant::now();
    let iterations = 5000;
    let mut visible_total = 0usize;
    for _ in 0..iterations {
        let mut visible = 0;
        for pos in &positions {
            if frustum.test_sphere(*pos, 2.0) {
                visible += 1;
            }
        }
        visible_total += visible;
    }
    let elapsed = start.elapsed();
    println!(
        "[FrustumCull1000] {} iterations in {:.2?}, avg {:.0} visible, {:.0}M tests/sec",
        iterations,
        elapsed,
        visible_total as f64 / iterations as f64,
        (iterations as f64 * 1000.0) / elapsed.as_secs_f64() / 1_000_000.0
    );
}

fn main() {
    println!("=== ENGENE Benchmark Suite ===\n");

    bench_command_buffer_throughput();
    bench_event_bus_throughput();
    bench_worker_pool_fanout();
    bench_dependency_graph_invalidation();
    bench_prefab_registry_lookup();
    bench_io_budget_streamer();
    bench_world_tick_100_entities();
    bench_ai_batch_50_npcs();
    bench_chunk_save_load_roundtrip();
    bench_frustum_cull_1000_instances();

    println!("\n=== Done ===");
}
