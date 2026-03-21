use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use engine_runtime::simulation_core::{
    DeferredTransitionPolicy, DeferredTransitionQueue, PromotionRequest, SimulationLevel,
    SimulationPolicyProfile, SimulationSubject, TransitionExecutionContext,
    TransitionMetricsSnapshot, TransitionOrchestrator, TransitionReason, TransitionRequest,
    TransitionShardMergePolicy, TransitionShardOutput,
};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

const CLASSIFY_COUNT: usize = 131_072;
const REQUEST_COUNT: usize = 16_384;
const DEFERRED_REPLAY_COUNT: usize = 1_024;
const SHARDS: usize = 8;

fn make_distances(count: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        out.push((i % 120_000) as f32);
    }
    out
}

fn make_requests(orchestrator: TransitionOrchestrator, count: usize) -> Vec<TransitionRequest> {
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let subject = SimulationSubject {
            entity_id: i as u64,
        };
        let from = if i % 2 == 0 {
            SimulationLevel::L1
        } else {
            SimulationLevel::L2
        };
        let to = if i % 2 == 0 {
            SimulationLevel::L0
        } else {
            SimulationLevel::L3
        };
        let req = if i % 2 == 0 {
            TransitionRequest::Promote(orchestrator.make_promotion_request(
                subject,
                from,
                to,
                TransitionReason::PlayerProximity,
            ))
        } else {
            TransitionRequest::Demote(orchestrator.make_demotion_request(
                subject,
                from,
                to,
                TransitionReason::InteractionEntropyLow,
            ))
        };
        out.push(req);
    }
    out
}

#[inline(always)]
fn request_for_entity(idx: usize, to: SimulationLevel) -> TransitionRequest {
    TransitionRequest::Promote(PromotionRequest {
        subject: SimulationSubject {
            entity_id: idx as u64,
        },
        from: SimulationLevel::L1,
        to,
        reason: TransitionReason::PlayerProximity,
    })
}

#[derive(Clone)]
struct ShardRanges {
    ranges: [(usize, usize); SHARDS],
}

#[derive(Clone, Copy)]
struct ShardRange {
    start: usize,
    end: usize,
}

struct SharedEpochState {
    epoch: AtomicU64,
    done_count: AtomicUsize,
    stop: AtomicBool,
}

impl SharedEpochState {
    fn new() -> Self {
        Self {
            epoch: AtomicU64::new(0),
            done_count: AtomicUsize::new(0),
            stop: AtomicBool::new(false),
        }
    }
}

struct ClassifyOnlySharedState {
    sync: SharedEpochState,
    distances_addr: AtomicUsize,
    out_addr: AtomicUsize,
}

struct PersistentClassifyOnlyPool {
    shared: Arc<ClassifyOnlySharedState>,
    handles: Vec<JoinHandle<()>>,
}

impl PersistentClassifyOnlyPool {
    fn new(orchestrator: TransitionOrchestrator, ranges: &ShardRanges) -> Self {
        let shared = Arc::new(ClassifyOnlySharedState {
            sync: SharedEpochState::new(),
            distances_addr: AtomicUsize::new(0),
            out_addr: AtomicUsize::new(0),
        });
        let mut handles = Vec::with_capacity(SHARDS);
        for sid in 0..SHARDS {
            let (start, end) = ranges.get(sid);
            let shard = ShardRange { start, end };
            let shared = Arc::clone(&shared);
            let handle = std::thread::spawn(move || {
                let mut observed_epoch = 0u64;
                loop {
                    wait_for_next_epoch(&shared.sync, &mut observed_epoch);
                    if shared.sync.stop.load(Ordering::Acquire) {
                        return;
                    }

                    if shard.start < shard.end {
                        let distances_ptr =
                            shared.distances_addr.load(Ordering::Relaxed) as *const f32;
                        let out_ptr =
                            shared.out_addr.load(Ordering::Relaxed) as *mut SimulationLevel;
                        for idx in shard.start..shard.end {
                            // SAFETY: pointers are valid for run duration and shard ranges are disjoint.
                            unsafe {
                                *out_ptr.add(idx) =
                                    orchestrator.classify_distance(*distances_ptr.add(idx));
                            }
                        }
                    }
                    shared.sync.done_count.fetch_add(1, Ordering::Release);
                }
            });
            handles.push(handle);
        }
        Self { shared, handles }
    }

    fn run(&self, distances: &[f32], out_levels: &mut [SimulationLevel]) {
        self.shared
            .distances_addr
            .store(distances.as_ptr() as usize, Ordering::Relaxed);
        self.shared
            .out_addr
            .store(out_levels.as_mut_ptr() as usize, Ordering::Relaxed);
        run_epoch_and_wait(&self.shared.sync, SHARDS);
    }
}

impl Drop for PersistentClassifyOnlyPool {
    fn drop(&mut self) {
        self.shared.sync.stop.store(true, Ordering::Release);
        self.shared.sync.epoch.fetch_add(1, Ordering::Release);
        while let Some(handle) = self.handles.pop() {
            let _ = handle.join();
        }
    }
}

struct MaterializeSharedState {
    sync: SharedEpochState,
    distances_addr: AtomicUsize,
    arena_addr: AtomicUsize,
}

struct PersistentMaterializePool {
    shared: Arc<MaterializeSharedState>,
    handles: Vec<JoinHandle<()>>,
}

impl PersistentMaterializePool {
    fn new(orchestrator: TransitionOrchestrator, ranges: &ShardRanges) -> Self {
        let shared = Arc::new(MaterializeSharedState {
            sync: SharedEpochState::new(),
            distances_addr: AtomicUsize::new(0),
            arena_addr: AtomicUsize::new(0),
        });
        let mut handles = Vec::with_capacity(SHARDS);
        for sid in 0..SHARDS {
            let (start, end) = ranges.get(sid);
            let shard = ShardRange { start, end };
            let shared = Arc::clone(&shared);
            let handle = std::thread::spawn(move || {
                let mut observed_epoch = 0u64;
                loop {
                    wait_for_next_epoch(&shared.sync, &mut observed_epoch);
                    if shared.sync.stop.load(Ordering::Acquire) {
                        return;
                    }

                    if shard.start < shard.end {
                        let distances_ptr =
                            shared.distances_addr.load(Ordering::Relaxed) as *const f32;
                        let arena_ptr =
                            shared.arena_addr.load(Ordering::Relaxed) as *mut TransitionRequest;
                        for idx in shard.start..shard.end {
                            // SAFETY: pointers are valid for run duration and shard ranges are disjoint.
                            unsafe {
                                let level = orchestrator.classify_distance(*distances_ptr.add(idx));
                                *arena_ptr.add(idx) =
                                    TransitionRequest::Promote(PromotionRequest {
                                        subject: SimulationSubject {
                                            entity_id: idx as u64,
                                        },
                                        from: SimulationLevel::L1,
                                        to: level,
                                        reason: TransitionReason::PlayerProximity,
                                    });
                            }
                        }
                    }
                    shared.sync.done_count.fetch_add(1, Ordering::Release);
                }
            });
            handles.push(handle);
        }
        Self { shared, handles }
    }

    fn run(&self, distances: &[f32], arena: &mut [TransitionRequest]) {
        self.shared
            .distances_addr
            .store(distances.as_ptr() as usize, Ordering::Relaxed);
        self.shared
            .arena_addr
            .store(arena.as_mut_ptr() as usize, Ordering::Relaxed);
        run_epoch_and_wait(&self.shared.sync, SHARDS);
    }
}

impl Drop for PersistentMaterializePool {
    fn drop(&mut self) {
        self.shared.sync.stop.store(true, Ordering::Release);
        self.shared.sync.epoch.fetch_add(1, Ordering::Release);
        while let Some(handle) = self.handles.pop() {
            let _ = handle.join();
        }
    }
}

#[inline(always)]
fn wait_for_next_epoch(sync: &SharedEpochState, observed_epoch: &mut u64) {
    let mut spins = 0u32;
    loop {
        if sync.stop.load(Ordering::Acquire) {
            return;
        }
        let epoch = sync.epoch.load(Ordering::Acquire);
        if epoch != *observed_epoch {
            *observed_epoch = epoch;
            return;
        }
        if spins < 256 {
            std::hint::spin_loop();
            spins += 1;
        } else {
            std::thread::yield_now();
        }
    }
}

#[inline(always)]
fn run_epoch_and_wait(sync: &SharedEpochState, expected_done: usize) {
    sync.done_count.store(0, Ordering::Relaxed);
    sync.epoch.fetch_add(1, Ordering::Release);
    let mut spins = 0u32;
    while sync.done_count.load(Ordering::Acquire) != expected_done {
        if spins < 256 {
            std::hint::spin_loop();
            spins += 1;
        } else {
            std::thread::yield_now();
        }
    }
}

impl ShardRanges {
    fn new(total: usize) -> Self {
        let mut ranges = [(0usize, 0usize); SHARDS];
        let chunk = total.div_ceil(SHARDS);
        for (sid, slot) in ranges.iter_mut().enumerate() {
            let start = sid * chunk;
            let end = (start + chunk).min(total);
            *slot = (start, end);
        }
        Self { ranges }
    }

    fn to_shard_outputs(&self, arena: &[TransitionRequest]) -> Vec<TransitionShardOutput> {
        let mut out = Vec::with_capacity(SHARDS);
        for (sid, (start, end)) in self.ranges.iter().copied().enumerate() {
            out.push(TransitionShardOutput {
                shard_id: sid as u16,
                requests: arena[start..end].to_vec(),
            });
        }
        out
    }

    #[inline(always)]
    fn get(&self, sid: usize) -> (usize, usize) {
        self.ranges[sid]
    }
}

fn classify_only_single_thread(
    orchestrator: TransitionOrchestrator,
    distances: &[f32],
    out_levels: &mut [SimulationLevel],
) {
    debug_assert_eq!(distances.len(), out_levels.len());
    for idx in 0..distances.len() {
        out_levels[idx] = orchestrator.classify_distance(distances[idx]);
    }
}

fn classify_only_multi_thread_x8(
    distances: &[f32],
    out_levels: &mut [SimulationLevel],
    workers: &PersistentClassifyOnlyPool,
) {
    debug_assert_eq!(distances.len(), out_levels.len());
    workers.run(distances, out_levels);
}

fn classify_materialize_single_thread(
    orchestrator: TransitionOrchestrator,
    distances: &[f32],
    arena: &mut [TransitionRequest],
) {
    for idx in 0..distances.len() {
        let level = orchestrator.classify_distance(distances[idx]);
        arena[idx] = TransitionRequest::Promote(PromotionRequest {
            subject: SimulationSubject {
                entity_id: idx as u64,
            },
            from: SimulationLevel::L1,
            to: level,
            reason: TransitionReason::PlayerProximity,
        });
    }
}

fn classify_materialize_multi_thread_x8(
    distances: &[f32],
    arena: &mut [TransitionRequest],
    workers: &PersistentMaterializePool,
) {
    workers.run(distances, arena);
}

fn bench_classify_only_single_thread(c: &mut Criterion) {
    let orchestrator = TransitionOrchestrator::default();
    let distances = make_distances(CLASSIFY_COUNT);
    let mut out_levels = vec![SimulationLevel::L3; CLASSIFY_COUNT];

    c.bench_function("simulation_core/classify_only_single_thread", |b| {
        b.iter(|| {
            classify_only_single_thread(orchestrator, &distances, &mut out_levels);
            black_box(out_levels[0])
        })
    });
}

fn bench_classify_only_multi_thread_x8(c: &mut Criterion) {
    let orchestrator = TransitionOrchestrator::default();
    let distances = make_distances(CLASSIFY_COUNT);
    let ranges = ShardRanges::new(CLASSIFY_COUNT);
    let workers = PersistentClassifyOnlyPool::new(orchestrator, &ranges);
    let mut out_levels = vec![SimulationLevel::L3; CLASSIFY_COUNT];

    c.bench_function("simulation_core/classify_only_multi_thread_x8", |b| {
        b.iter(|| {
            classify_only_multi_thread_x8(&distances, &mut out_levels, &workers);
            black_box(out_levels[0])
        })
    });
}

fn bench_classify_materialize_single_thread(c: &mut Criterion) {
    let orchestrator = TransitionOrchestrator::default();
    let distances = make_distances(CLASSIFY_COUNT);
    let placeholder = request_for_entity(0, SimulationLevel::L1);
    let mut arena = vec![placeholder; CLASSIFY_COUNT];

    c.bench_function("simulation_core/classify_materialize_single_thread", |b| {
        b.iter(|| {
            classify_materialize_single_thread(orchestrator, &distances, &mut arena);
            black_box(arena[0])
        })
    });
}

fn bench_classify_materialize_multi_thread_x8(c: &mut Criterion) {
    let orchestrator = TransitionOrchestrator::default();
    let distances = make_distances(CLASSIFY_COUNT);
    let ranges = ShardRanges::new(CLASSIFY_COUNT);
    let workers = PersistentMaterializePool::new(orchestrator, &ranges);
    let placeholder = request_for_entity(0, SimulationLevel::L1);
    let mut arena = vec![placeholder; CLASSIFY_COUNT];

    c.bench_function(
        "simulation_core/classify_materialize_multi_thread_x8",
        |b| {
            b.iter(|| {
                classify_materialize_multi_thread_x8(&distances, &mut arena, &workers);
                black_box(arena[0])
            })
        },
    );
}

fn bench_classify_order(c: &mut Criterion) {
    let orchestrator = TransitionOrchestrator::default();
    let requests = make_requests(orchestrator, REQUEST_COUNT);

    c.bench_function("simulation_core/classify_order", |b| {
        b.iter_batched(
            || requests.clone(),
            |mut batch| {
                orchestrator.order_batch(&mut batch);
                black_box(batch.len())
            },
            BatchSize::LargeInput,
        )
    });
}

fn bench_classify_order_resolve(c: &mut Criterion) {
    let orchestrator = TransitionOrchestrator::default();
    let requests = make_requests(orchestrator, REQUEST_COUNT);
    c.bench_function("simulation_core/classify_order_resolve", |b| {
        b.iter_batched(
            || {
                (
                    requests.clone(),
                    DeferredTransitionQueue::with_policy(DeferredTransitionPolicy::default()),
                    TransitionExecutionContext::from_policy(
                        SimulationPolicyProfile::default(),
                        10,
                        10,
                    ),
                    TransitionMetricsSnapshot::default(),
                )
            },
            |(mut batch, mut queue, mut context, mut metrics)| {
                orchestrator.resolve_ordered_batch(
                    &mut batch,
                    &mut context,
                    &mut queue,
                    &mut metrics,
                );
                black_box((metrics.applied, queue.snapshot().queued))
            },
            BatchSize::LargeInput,
        )
    });
}

fn bench_deferred_queue_replay(c: &mut Criterion) {
    let orchestrator = TransitionOrchestrator::default();
    let requests = make_requests(orchestrator, REQUEST_COUNT);
    c.bench_function("simulation_core/deferred_queue_replay", |b| {
        b.iter_batched(
            || {
                let mut queue =
                    DeferredTransitionQueue::with_policy(DeferredTransitionPolicy::default());
                for req in requests.iter().take(DEFERRED_REPLAY_COUNT).copied() {
                    queue.enqueue_new(
                        req,
                        1,
                        1,
                        engine_runtime::simulation_core::TransitionDisposition::DeferredByPromotionBudget,
                    );
                }
                (queue, Vec::with_capacity(DEFERRED_REPLAY_COUNT))
            },
            |(mut queue, mut out)| {
                queue.drain_ready_into(&mut out, 2, DEFERRED_REPLAY_COUNT);
                black_box(out.len())
            },
            BatchSize::LargeInput,
        )
    });
}

fn bench_deterministic_merge_prep(c: &mut Criterion) {
    let orchestrator = TransitionOrchestrator::default();
    let distances = make_distances(CLASSIFY_COUNT);
    let ranges = ShardRanges::new(CLASSIFY_COUNT);
    let placeholder = request_for_entity(0, SimulationLevel::L1);
    let mut arena = vec![placeholder; CLASSIFY_COUNT];
    classify_materialize_single_thread(orchestrator, &distances, &mut arena);
    let shards = ranges.to_shard_outputs(&arena);

    c.bench_function("simulation_core/deterministic_merge_prep", |b| {
        let mut out = Vec::<TransitionRequest>::with_capacity(CLASSIFY_COUNT);
        let mut scratch = Vec::<(u16, TransitionRequest)>::with_capacity(CLASSIFY_COUNT);
        b.iter(|| {
            orchestrator.merge_shard_outputs(
                &shards,
                TransitionShardMergePolicy::default(),
                &mut out,
                &mut scratch,
            );
            black_box(out.len())
        })
    });
}

criterion_group!(
    benches,
    bench_classify_only_single_thread,
    bench_classify_only_multi_thread_x8,
    bench_classify_materialize_single_thread,
    bench_classify_materialize_multi_thread_x8,
    bench_classify_order,
    bench_classify_order_resolve,
    bench_deferred_queue_replay,
    bench_deterministic_merge_prep,
);
criterion_main!(benches);
