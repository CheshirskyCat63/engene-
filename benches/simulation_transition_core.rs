use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use engine_runtime::simulation_core::{
    DeferredTransitionPolicy, DeferredTransitionQueue, PromotionRequest, SimulationLevel,
    SimulationPolicyProfile, SimulationSubject, TransitionExecutionContext,
    TransitionMetricsSnapshot, TransitionOrchestrator, TransitionReason, TransitionRequest,
    TransitionShardMergePolicy, TransitionShardOutput,
};
use rayon::prelude::*;
use rayon::ThreadPool;

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

fn make_thread_pool(threads: usize) -> ThreadPool {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("thread pool")
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
    orchestrator: TransitionOrchestrator,
    distances: &[f32],
    out_levels: &mut [SimulationLevel],
    pool: &ThreadPool,
) {
    debug_assert_eq!(distances.len(), out_levels.len());
    let chunk = distances.len().div_ceil(SHARDS);
    pool.install(|| {
        out_levels
            .par_chunks_mut(chunk)
            .enumerate()
            .for_each(|(sid, out_chunk)| {
                let start = sid * chunk;
                for (i, slot) in out_chunk.iter_mut().enumerate() {
                    *slot = orchestrator.classify_distance(distances[start + i]);
                }
            });
    });
}

fn classify_materialize_single_thread(
    orchestrator: TransitionOrchestrator,
    distances: &[f32],
    arena: &mut [TransitionRequest],
) {
    for idx in 0..distances.len() {
        let level = orchestrator.classify_distance(distances[idx]);
        arena[idx] = request_for_entity(idx, level);
    }
}

fn classify_materialize_multi_thread_x8(
    orchestrator: TransitionOrchestrator,
    distances: &[f32],
    arena: &mut [TransitionRequest],
    pool: &ThreadPool,
) {
    let chunk = distances.len().div_ceil(SHARDS);
    pool.install(|| {
        arena
            .par_chunks_mut(chunk)
            .enumerate()
            .for_each(|(sid, out_chunk)| {
                let start = sid * chunk;
                for (i, slot) in out_chunk.iter_mut().enumerate() {
                    let idx = start + i;
                    let level = orchestrator.classify_distance(distances[idx]);
                    *slot = request_for_entity(idx, level);
                }
            });
    });
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
    let pool = make_thread_pool(SHARDS);
    let mut out_levels = vec![SimulationLevel::L3; CLASSIFY_COUNT];

    c.bench_function("simulation_core/classify_only_multi_thread_x8", |b| {
        b.iter(|| {
            classify_only_multi_thread_x8(orchestrator, &distances, &mut out_levels, &pool);
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
    let pool = make_thread_pool(SHARDS);
    let placeholder = request_for_entity(0, SimulationLevel::L1);
    let mut arena = vec![placeholder; CLASSIFY_COUNT];

    c.bench_function(
        "simulation_core/classify_materialize_multi_thread_x8",
        |b| {
            b.iter(|| {
                classify_materialize_multi_thread_x8(orchestrator, &distances, &mut arena, &pool);
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
