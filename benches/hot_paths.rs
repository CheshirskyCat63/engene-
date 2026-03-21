//! Benchmarks for ENGENE hot paths (Phase C.6)
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// =============================================================================
// Spatial Index Benchmarks
// =============================================================================

fn bench_spatial_index(c: &mut Criterion) {
    use std::collections::HashMap;

    // Simulate HierarchicalSpatialIndex
    fn cell_key(x: f32, z: f32, cell_size: f32) -> (i32, i32) {
        (
            (x / cell_size).floor() as i32,
            (z / cell_size).floor() as i32,
        )
    }

    struct Entity {
        id: u64,
    }
    struct SpatialLevel {
        cell_size: f32,
        cells: HashMap<(i32, i32), Vec<(Entity, f32, f32)>>,
    }

    impl SpatialLevel {
        fn new(cell_size: f32) -> Self {
            Self {
                cell_size,
                cells: HashMap::new(),
            }
        }

        fn insert(&mut self, entity: Entity, x: f32, z: f32) {
            let key = cell_key(x, z, self.cell_size);
            self.cells.entry(key).or_default().push((entity, x, z));
        }

        fn query_radius(&self, x: f32, z: f32, radius: f32) -> usize {
            let r2 = radius * radius;
            let min_key = cell_key(x - radius, z - radius, self.cell_size);
            let max_key = cell_key(x + radius, z + radius, self.cell_size);

            let mut count = 0;
            for cy in min_key.1..=max_key.1 {
                for cx in min_key.0..=max_key.0 {
                    if let Some(entities) = self.cells.get(&(cx, cy)) {
                        for (_, ex, ez) in entities {
                            let dx = ex - x;
                            let dz = ez - z;
                            if dx * dx + dz * dz <= r2 {
                                count += 1;
                            }
                        }
                    }
                }
            }
            count
        }
    }

    let mut group = c.benchmark_group("spatial_index");

    for size in [100, 1000, 10000].iter() {
        // Insert benchmark
        group.bench_with_input(BenchmarkId::new("insert", size), size, |b, &size| {
            b.iter(|| {
                let mut level = SpatialLevel::new(10.0);
                for i in 0..size {
                    level.insert(
                        Entity { id: i as u64 },
                        (i as f32 * 1.5) % 1000.0,
                        (i as f32 * 2.3) % 1000.0,
                    );
                }
                black_box(&level);
            });
        });

        // Query benchmark
        let mut level = SpatialLevel::new(10.0);
        for i in 0..*size {
            level.insert(
                Entity { id: i as u64 },
                (i as f32 * 1.5) % 1000.0,
                (i as f32 * 2.3) % 1000.0,
            );
        }

        group.bench_with_input(BenchmarkId::new("query_radius_50", size), size, |b, _| {
            b.iter(|| black_box(level.query_radius(500.0, 500.0, 50.0)));
        });

        group.bench_with_input(BenchmarkId::new("query_radius_200", size), size, |b, _| {
            b.iter(|| black_box(level.query_radius(500.0, 500.0, 200.0)));
        });
    }

    group.finish();
}

// =============================================================================
// ECS Component Access Benchmarks
// =============================================================================

fn bench_ecs_component_access(c: &mut Criterion) {
    use std::collections::HashMap;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct Entity {
        id: u64,
        generation: u32,
    }

    #[derive(Clone, Copy, Default)]
    struct Transform {
        x: f32,
        y: f32,
        z: f32,
    }

    #[derive(Clone, Copy, Default)]
    struct Velocity {
        vx: f32,
        vy: f32,
        vz: f32,
    }

    struct Ecs {
        transforms: HashMap<Entity, Transform>,
        velocities: HashMap<Entity, Velocity>,
        alive: Vec<Entity>,
    }

    let mut group = c.benchmark_group("ecs_component_access");

    for size in [100, 1000, 10000].iter() {
        let mut ecs = Ecs {
            transforms: HashMap::new(),
            velocities: HashMap::new(),
            alive: Vec::with_capacity(*size),
        };

        for i in 0..*size {
            let e = Entity {
                id: i as u64,
                generation: 0,
            };
            ecs.alive.push(e);
            ecs.transforms.insert(
                e,
                Transform {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },
            );
            ecs.velocities.insert(
                e,
                Velocity {
                    vx: 1.0,
                    vy: 0.0,
                    vz: 0.0,
                },
            );
        }

        // Read-only iteration
        group.bench_with_input(BenchmarkId::new("read_iteration", size), size, |b, _| {
            b.iter(|| {
                let mut sum = 0.0f32;
                for &e in &ecs.alive {
                    if let Some(t) = ecs.transforms.get(&e) {
                        sum += t.x;
                    }
                }
                black_box(sum)
            });
        });

        // Read-write iteration
        group.bench_with_input(
            BenchmarkId::new("read_write_iteration", size),
            size,
            |b, _| {
                b.iter(|| {
                    for &e in &ecs.alive {
                        if let Some(t) = ecs.transforms.get(&e) {
                            if let Some(v) = ecs.velocities.get(&e) {
                                let new_x = t.x + v.vx;
                                if let Some(t) = ecs.transforms.get_mut(&e) {
                                    t.x = new_x;
                                }
                            }
                        }
                    }
                    black_box(ecs.transforms.len())
                });
            },
        );
    }

    group.finish();
}

// =============================================================================
// Event Bus Benchmarks
// =============================================================================

fn bench_event_bus(c: &mut Criterion) {
    use std::collections::VecDeque;

    struct EventBus<T> {
        queue: VecDeque<T>,
    }

    impl<T> EventBus<T> {
        fn new() -> Self {
            Self {
                queue: VecDeque::new(),
            }
        }
        fn emit(&mut self, event: T) {
            self.queue.push_back(event);
        }
        fn drain(&mut self) -> impl Iterator<Item = T> + '_ {
            self.queue.drain(..)
        }
    }

    #[derive(Clone, Copy)]
    struct TestEvent {
        value: u32,
    }

    let mut group = c.benchmark_group("event_bus");

    for count in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("emit", count), count, |b, &count| {
            let mut bus = EventBus::new();
            b.iter(|| {
                for i in 0..count {
                    bus.emit(TestEvent { value: i });
                }
                black_box(&bus);
            });
        });

        group.bench_with_input(
            BenchmarkId::new("emit_and_drain", count),
            count,
            |b, &count| {
                let mut bus = EventBus::new();
                b.iter(|| {
                    for i in 0..count {
                        bus.emit(TestEvent { value: i });
                    }
                    let sum: u32 = bus.drain().map(|e| e.value).sum();
                    black_box(sum)
                });
            },
        );
    }

    group.finish();
}

// =============================================================================
// Metrics Registry Benchmarks
// =============================================================================

fn bench_metrics_registry(c: &mut Criterion) {
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::RwLock;

    struct MetricsRegistry {
        counters: RwLock<HashMap<String, AtomicU64>>,
    }

    impl MetricsRegistry {
        fn new() -> Self {
            Self {
                counters: RwLock::new(HashMap::new()),
            }
        }

        fn register(&self, name: &str) {
            let mut counters = self.counters.write().unwrap();
            counters.insert(name.to_string(), AtomicU64::new(0));
        }

        fn increment(&self, name: &str, delta: u64) {
            let counters = self.counters.read().unwrap();
            if let Some(counter) = counters.get(name) {
                counter.fetch_add(delta, Ordering::Relaxed);
            }
        }

        fn get(&self, name: &str) -> u64 {
            let counters = self.counters.read().unwrap();
            counters
                .get(name)
                .map(|c| c.load(Ordering::Relaxed))
                .unwrap_or(0)
        }
    }

    let mut group = c.benchmark_group("metrics_registry");

    let registry = MetricsRegistry::new();
    registry.register("counter1");
    registry.register("counter2");

    group.bench_function("increment_single", |b| {
        b.iter(|| {
            registry.increment("counter1", 1);
        });
    });

    group.bench_function("increment_multi", |b| {
        b.iter(|| {
            registry.increment("counter1", 1);
            registry.increment("counter2", 1);
        });
    });

    group.bench_function("read_after_writes", |b| {
        b.iter(|| {
            registry.increment("counter1", 1);
            black_box(registry.get("counter1"))
        });
    });

    group.finish();
}

// =============================================================================
// AI Scheduler Benchmarks
// =============================================================================

fn bench_ai_scheduler(c: &mut Criterion) {
    use std::collections::HashMap;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct Entity {
        id: u64,
    }

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum AiPriority {
        Idle = 0,
        Normal = 1,
        Urgent = 2,
        Critical = 3,
    }

    struct EntitySchedule {
        last_tick_frame: u64,
        skip_count: u32,
        priority: AiPriority,
    }

    struct AiScheduler {
        schedules: HashMap<Entity, EntitySchedule>,
        frame: u64,
        budget_us: u64,
    }

    impl AiScheduler {
        fn new() -> Self {
            Self {
                schedules: HashMap::new(),
                frame: 0,
                budget_us: 4000,
            }
        }

        fn update_entity(&mut self, entity: Entity, priority: AiPriority) {
            self.schedules
                .entry(entity)
                .and_modify(|s| s.priority = priority)
                .or_insert(EntitySchedule {
                    last_tick_frame: 0,
                    skip_count: 0,
                    priority,
                });
        }

        fn should_tick(&mut self, entity: Entity, elapsed_us: u64) -> bool {
            let schedule = self.schedules.get_mut(&entity);
            let schedule = match schedule {
                Some(s) => s,
                None => return true,
            };

            if schedule.skip_count >= 3 {
                schedule.skip_count = 0;
                schedule.last_tick_frame = self.frame;
                return true;
            }

            if elapsed_us >= self.budget_us && schedule.priority < AiPriority::Critical {
                return false;
            }

            schedule.priority >= AiPriority::Normal
        }

        fn begin_frame(&mut self) {
            self.frame += 1;
        }
    }

    let mut group = c.benchmark_group("ai_scheduler");

    for size in [100, 1000, 10000].iter() {
        let mut scheduler = AiScheduler::new();

        for i in 0..*size {
            let priority = match i % 4 {
                0 => AiPriority::Idle,
                1 => AiPriority::Normal,
                2 => AiPriority::Urgent,
                _ => AiPriority::Critical,
            };
            scheduler.update_entity(Entity { id: i as u64 }, priority);
        }

        group.bench_with_input(BenchmarkId::new("should_tick", size), size, |b, _| {
            b.iter(|| {
                scheduler.begin_frame();
                let mut tick_count = 0;
                for i in 0..*size {
                    if scheduler.should_tick(Entity { id: i as u64 }, 0) {
                        tick_count += 1;
                    }
                }
                black_box(tick_count)
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_spatial_index,
    bench_ecs_component_access,
    bench_event_bus,
    bench_metrics_registry,
    bench_ai_scheduler,
);

criterion_main!(benches);
